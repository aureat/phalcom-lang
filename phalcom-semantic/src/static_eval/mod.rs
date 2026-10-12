pub mod application;
mod error;
pub mod machine;
pub(crate) mod pattern;
pub mod residual;
pub mod specialize;

pub use application::{
    CanonicalStaticProofFact, StaticApplicationEnvironmentKey, StaticApplicationKey, StaticArgumentIdentity, StaticArgumentSummary,
    StaticCycle, StaticCycleFrame, StaticMemoResult, StaticSpecializationArgument, StaticSpecializationEnvironmentError,
    StaticSpecializationEnvironmentKey,
};
pub use error::{StaticEvalError, StaticEvalOutcome, StaticEvalTraceFrame};
pub use machine::{StaticArgument, StaticEvaluator};
pub use residual::{
    StaticEvalTerm, StaticResidualApplication, StaticResidualData, StaticResidualId, StaticResidualMatch, StaticResidualMatchArm,
    StaticResidualParameter, StaticResidualRef, StaticResidualStore, StaticResidualStoreError, StaticResidualStoreId,
    StaticResidualTerm,
};
pub use specialize::specialize_residual;
