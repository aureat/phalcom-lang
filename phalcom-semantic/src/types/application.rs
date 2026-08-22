//! Canonical kind-checked type application.

use super::id::{KindId, TypeId};

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TypeApplicationError {
    #[error("type form is not applicable")]
    NotAConstructor { origin: TypeId, kind: KindId },

    #[error("too many type arguments: supplied {supplied}, accepted {accepted}")]
    TooManyArguments { supplied: usize, accepted: usize },

    #[error("type argument kind mismatch at index {index}")]
    ArgumentKindMismatch {
        index: usize,
        expected: KindId,
        actual: KindId,
    },
}
