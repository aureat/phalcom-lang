//! Epistemic explanation and diagnostic cause DAG system (Spec 04.5).

pub mod arena;
pub mod node;
pub mod slice;

pub use arena::ExplanationArena;
pub use node::{ExplanationNode, ExplanationStep};
pub use slice::causal_slice;
