//! Bounded execution of checker-authorized static expressions.

mod error;
mod machine;

pub use error::{StaticEvalError, StaticEvalOutcome, StaticEvalTraceFrame};
pub use machine::{StaticArgument, StaticEvaluator};
