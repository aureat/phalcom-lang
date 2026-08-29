//! Compiler-owned formal control-flow and path analysis (Spec 04.5).

pub mod graph;
pub mod predicate;
pub mod state;
pub mod transfer;

pub use graph::{FlowEdge, FlowEdgeKind, FlowGraph, FlowNode, FlowNodeKind};
pub use predicate::{FlowPredicate, PredicateEntry, extract_predicate, extract_trusted_predicate};
pub use state::{FactSet, FieldContractValidity, FieldInitialization, FieldState, FlowInvariantFailure, FlowState, join_field_validity};

pub use transfer::apply_predicate;
