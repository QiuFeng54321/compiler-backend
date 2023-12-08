use std::fmt::{Display, Formatter};
use std::ops::Deref;

use fixedbitset::FixedBitSet;

use crate::block::DataFlowGraph;
use crate::ir::{CodeBlockGraphWeight, IR};
use crate::{
    block::{BlockLattice, BlockTransfer},
    ir::CodeBlock,
    semilattice::SemiLattice,
};

#[derive(Debug)]
pub struct ReachLattice {
    value: FixedBitSet,
}

impl ReachLattice {
    pub fn new(capacity: usize) -> Self {
        Self {
            value: FixedBitSet::with_capacity(capacity),
        }
    }
    /**
    Only the current declaration is 1
     */
    pub fn gen_var(ir: &IR, code_block_graph_weight: &CodeBlockGraphWeight) -> Self {
        let mut set = FixedBitSet::with_capacity(code_block_graph_weight.assignment_count);
        match ir {
            IR::Quad(_, var, _, _, info) => match info.declaration_number {
                Some(declaration_number) => {
                    set.set(declaration_number, true);
                }
                _ => {}
            },
            _ => {}
        }
        Self { value: set }
    }
    /**
    All 1s, but killed declaration numbers are 0
     */
    pub fn kill_mask_var(ir: &IR, code_block_graph_weight: &CodeBlockGraphWeight) -> Self {
        let mut set = Self::kill_var(ir, code_block_graph_weight);
        set.value.toggle_range(..);
        set
    }
    /**
    All 0s, but killed declaration numbers are 1
     */
    pub fn kill_var(ir: &IR, code_block_graph_weight: &CodeBlockGraphWeight) -> Self {
        let mut set = FixedBitSet::with_capacity(code_block_graph_weight.assignment_count);
        match ir {
            IR::Quad(_, var, _, _, info) => {
                code_block_graph_weight
                    .variable_assignment_map
                    .get(var)
                    .expect("Variable not found")
                    .iter()
                    .for_each(|declaration_number| {
                        set.set(*declaration_number, true);
                    });
                set.set(
                    info.declaration_number.expect("No declaration number"),
                    false,
                );
            }
            IR::Jump(_, _, _, _, _) => {}
        }
        Self { value: set }
    }
}

impl PartialEq for ReachLattice {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Display for ReachLattice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
impl PartialOrd for ReachLattice {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl SemiLattice for ReachLattice {
    fn meet(&self, other: &Self) -> Self {
        let mut res_value = self.value.clone();
        res_value.union_with(&other.value);
        Self { value: res_value }
    }
}

impl BlockLattice<ReachLattice> for CodeBlock {
    fn get_in(&self) -> &ReachLattice {
        &self.reach_in
    }

    fn set_in(&mut self, value: ReachLattice) {
        self.reach_in = value
    }

    fn get_out(&self) -> &ReachLattice {
        &self.reach_out
    }

    fn set_out(&mut self, value: ReachLattice) {
        self.reach_out = value
    }
}

impl CodeBlock {
    fn transfer_reach(
        &self,
        input: ReachLattice,
        ir: &IR,
        graph: &DataFlowGraph<CodeBlock, CodeBlockGraphWeight>,
    ) {
    }
}

impl BlockTransfer<ReachLattice, CodeBlock, CodeBlockGraphWeight> for CodeBlock {
    fn transfer_forward(
        &self,
        in_value: &ReachLattice,
        graph: &DataFlowGraph<CodeBlock, CodeBlockGraphWeight>,
        self_index: petgraph::prelude::NodeIndex<u32>,
    ) -> ReachLattice {
        let mut current_gen = FixedBitSet::with_capacity(graph.weight.assignment_count);
        let mut current_kill = FixedBitSet::with_capacity(graph.weight.assignment_count);
        let mut current_kill_mask = FixedBitSet::with_capacity(graph.weight.assignment_count);
        current_kill_mask.toggle_range(..);
        self.irs_range.iter().rev().for_each(|ir| {
            let ir_kill = ReachLattice::kill_var(ir.borrow().deref(), &graph.weight);
            let mut ir_gen = ReachLattice::gen_var(ir.borrow().deref(), &graph.weight);
            current_kill.union_with(&ir_kill.value);
            let ir_kill_mask = ReachLattice::kill_mask_var(ir.borrow().deref(), &graph.weight);
            current_kill_mask.intersect_with(&ir_kill_mask.value);
            ir_gen.value.intersect_with(&current_kill_mask);
            current_gen.union_with(&ir_gen.value);
        });
        let mut res_out = in_value.value.clone();
        res_out.intersect_with(&current_kill_mask);
        res_out.union_with(&current_gen);

        ReachLattice { value: res_out }
    }

    fn transfer_backward(
        &self,
        out_value: &ReachLattice,
        graph: &DataFlowGraph<CodeBlock, CodeBlockGraphWeight>,
        self_index: petgraph::prelude::NodeIndex<u32>,
    ) -> ReachLattice {
        todo!()
    }

    fn entry_out(data_flow_graph: &DataFlowGraph<CodeBlock, CodeBlockGraphWeight>) -> ReachLattice {
        Self::top(data_flow_graph)
    }

    fn exit_in(data_flow_graph: &DataFlowGraph<CodeBlock, CodeBlockGraphWeight>) -> ReachLattice {
        unimplemented!("Invalid data flow")
    }

    fn top(data_flow_graph: &DataFlowGraph<CodeBlock, CodeBlockGraphWeight>) -> ReachLattice {
        ReachLattice::new(data_flow_graph.weight.assignment_count)
    }

    fn bottom(data_flow_graph: &DataFlowGraph<CodeBlock, CodeBlockGraphWeight>) -> ReachLattice {
        unimplemented!("Invalid data flow")
    }
}
