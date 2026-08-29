//! Epistemic explanation and diagnostic cause DAG system (Spec 04.5).

pub mod arena;
pub mod node;
pub mod slice;

pub use arena::ExplanationArena;
pub use node::{
    CallShapeExplanation, CollectionKind, DerivationRule, EvidenceRef, ExplanationNode, ExplanationStep, GenericConstraintOrigin, GenericConstraintRelation,
    PredicateKind,
};
pub use slice::{causal_slice, causal_trace};
