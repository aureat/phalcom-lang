use crate::identity::DeclarationId;
use crate::identity::{BindingId, ExpressionId};
use crate::types::outcome::BudgetReport;
use phalcom_common::range::SourceRange;
use thiserror::Error;

/// A fail-closed reason why a checked static body could not be reduced.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum StaticEvalError {
    #[error("static expression at source range {start}..{end} has ambiguous checked identities")]
    AmbiguousExpressionSite { start: usize, end: usize },
    #[error("static binding {0:?} has no supplied value or initialized local slot")]
    UnboundBinding(BindingId),
    #[error("static binding {0:?} was read before initialization")]
    UninitializedBinding(BindingId),
    #[error("static integer literal is malformed")]
    InvalidIntegerLiteral,
    #[error("static argument {index} does not match the checked parameter kind")]
    InvalidArgumentKind { index: u32 },
    #[error("static evaluator expected {expected} arguments but received {actual}")]
    InvalidArgumentCount { expected: usize, actual: usize },
    #[error("static operation {0:?} has no exact checker authorization")]
    UnsupportedOperation(ExpressionId),
    #[error("static operation received values outside its supported primitive domain")]
    InvalidOperationOperands,
    #[error("static expression form is not supported by this evaluator slice")]
    UnsupportedExpression,
    #[error("static control operation requires a Bool value")]
    ExpectedBool,
    #[error("static value store rejected an operation: {0}")]
    ValueStore(String),
}

/// Terminal state of one bounded static evaluation request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaticEvalOutcome {
    Reduced(crate::static_value::StaticValueRef),
    /// A checked canonical kinded result. TypeIds never enter the evaluator's
    /// ordinary StaticValueRef stack.
    ReducedKinded(crate::types::id::TypeId),
    Symbolic,
    Invalid(StaticEvalError),
    Recursive,
    BudgetExceeded(BudgetReport),
    Cancelled,
}

/// One bounded frame in the evaluator's diagnostic provenance path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaticEvalTraceFrame {
    /// Static definition currently executing.
    Definition(DeclarationId),
    /// Static constant currently executing.
    Constant(crate::static_constant::StaticConstantId),
    /// Exact checked application currently entering a target definition.
    Application { target: DeclarationId, source_range: SourceRange },
    /// Source expression currently being evaluated.
    Expression(SourceRange),
}
