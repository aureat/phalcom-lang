//! CFG-based termination analysis checking acyclicity.

use super::TerminationEvidence;
use crate::checker::flow::graph::{FlowGraph, FlowNodeKind};

/// Checks if a FlowGraph is acyclic (contains no back-edges or loop headers).
pub fn check_cfg_acyclicity(graph: &FlowGraph) -> Option<TerminationEvidence> {
    let has_loop = graph.nodes.values().any(|n| matches!(n.kind, FlowNodeKind::LoopHeader));
    if !has_loop { Some(TerminationEvidence::AcyclicCfg) } else { None }
}
