//! Control facts and exit summaries (Spec 05).

use crate::checker::flow::graph::{FlowGraph, FlowNodeKind};
use crate::identity::FlowNodeId;
use crate::types::id::TypeId;

/// Lower-level control product. ExitSummary is derived after termination is known.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControlFacts {
    pub may_return_normally: bool,
    pub raises: RaiseKnowledge,
    pub cycle_candidates: Vec<FlowNodeId>, // LoopHeader nodes from FlowGraph
    pub may_exit_process: bool,
    pub may_suspend: bool,
}

impl ControlFacts {
    /// Extracts control facts from a FlowGraph.
    pub fn from_flow_graph(graph: &FlowGraph) -> Self {
        let mut cycle_candidates = Vec::new();
        let may_return_normally = !graph.exits.is_empty();

        for node in graph.nodes.values() {
            if matches!(node.kind, FlowNodeKind::LoopHeader) {
                cycle_candidates.push(node.id);
            }
        }

        Self {
            may_return_normally,
            raises: RaiseKnowledge::None,
            cycle_candidates,
            may_exit_process: false,
            may_suspend: false,
        }
    }
}

/// Final exit summary derived after termination analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExitSummary {
    pub may_return_normally: bool,
    pub raises: RaiseKnowledge,
    pub divergence: DivergenceKnowledge, // set AFTER termination analysis
    pub may_exit_process: bool,
    pub may_suspend: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RaiseKnowledge {
    None,
    Known(Box<[TypeId]>), // sorted canonical exception types
    Opaque(RaiseOpaqueReason),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RaiseOpaqueReason {
    DynamicRaise,
    UnanalyzedNative,
    ForeignBoundary,
    UnknownDependency,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DivergenceKnowledge {
    ProvenAbsent, // ONLY after TerminationKnowledge::Proven
    Possible,
    Opaque(DivergenceOpaqueReason),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DivergenceOpaqueReason {
    UnsupportedRecursion,
    UnanalyzedNative,
    ForeignBoundary,
    UnknownDependency,
}
