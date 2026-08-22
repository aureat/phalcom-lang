//! Kind models distinguishing value types from type constructors.

use super::id::KindId;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum KindData {
    /// Fully saturated value type (`*` or `Type`).
    Type,
    /// Record row kind (`RecordRow`).
    RecordRow,
    /// Type constructor (`Kind -> Kind` or `(Kind, ...) -> Kind`).
    Arrow { parameters: Box<[KindId]>, result: KindId },
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum KindApplicationError {
    #[error("kind is not applicable: {kind:?}")]
    NotApplicable { kind: KindId },
    #[error("too many kind arguments: supplied {supplied}, accepted {accepted}")]
    TooManyArguments { supplied: usize, accepted: usize },
    #[error("argument kind mismatch at index {index}: expected {expected:?}, actual {actual:?}")]
    ArgumentKindMismatch { index: usize, expected: KindId, actual: KindId },
}
