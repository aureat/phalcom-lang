pub mod application;
mod error;
mod machine;
pub(crate) mod pattern;
pub mod residual;

pub use application::{
    CanonicalStaticProofFact, StaticApplicationKey, StaticArgumentIdentity, StaticArgumentSummary, StaticCycle, StaticCycleFrame,
    StaticMemoResult, StaticSpecializationArgument, StaticSpecializationEnvironmentError, StaticSpecializationEnvironmentKey,
};
pub use error::{StaticEvalError, StaticEvalOutcome, StaticEvalTraceFrame};
pub use machine::{StaticArgument, StaticEvaluator};
pub use residual::{
    StaticEvalTerm, StaticResidualApplication, StaticResidualData, StaticResidualId, StaticResidualMatch, StaticResidualMatchArm,
    StaticResidualParameter, StaticResidualRef, StaticResidualStore, StaticResidualStoreError, StaticResidualStoreId,
    StaticResidualTerm,
};
