//! Formal control-flow graph representation for callable bodies (Spec 04.5).

use crate::identity::{FlowEdgeId, FlowNodeId, PredicateId};
use phalcom_common::range::SourceRange;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FlowNodeKind {
    Entry,
    Exit,
    Statement(usize),
    BranchCondition,
    LoopHeader,
    Join,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowNode {
    pub id: FlowNodeId,
    pub kind: FlowNodeKind,
    pub range: SourceRange,
    pub predecessors: Vec<FlowEdgeId>,
    pub successors: Vec<FlowEdgeId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowEdge {
    pub id: FlowEdgeId,
    pub source: FlowNodeId,
    pub target: FlowNodeId,
    pub predicate: Option<PredicateId>,
}

/// Control-flow graph within a callable body.
#[derive(Clone, Debug, Default)]
pub struct FlowGraph {
    pub nodes: BTreeMap<FlowNodeId, FlowNode>,
    pub edges: BTreeMap<FlowEdgeId, FlowEdge>,
    pub entry: Option<FlowNodeId>,
    pub exits: Vec<FlowNodeId>,
    next_node_id: u32,
    next_edge_id: u32,
}

impl FlowGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, kind: FlowNodeKind, range: SourceRange) -> FlowNodeId {
        let id = FlowNodeId(self.next_node_id);
        self.next_node_id += 1;
        self.nodes.insert(
            id,
            FlowNode {
                id,
                kind,
                range,
                predecessors: Vec::new(),
                successors: Vec::new(),
            },
        );
        id
    }

    pub fn add_edge(&mut self, source: FlowNodeId, target: FlowNodeId, predicate: Option<PredicateId>) -> FlowEdgeId {
        let id = FlowEdgeId(self.next_edge_id);
        self.next_edge_id += 1;
        self.edges.insert(id, FlowEdge { id, source, target, predicate });
        if let Some(s) = self.nodes.get_mut(&source) {
            s.successors.push(id);
        }
        if let Some(t) = self.nodes.get_mut(&target) {
            t.predecessors.push(id);
        }
        id
    }
}
