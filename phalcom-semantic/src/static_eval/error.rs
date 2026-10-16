use crate::identity::DeclarationId;
use crate::identity::{BindingId, ExpressionId};
use crate::types::evidence::UnknownReason;
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
    #[error("canonical static local slot {slot} is out of range")]
    InvalidLocalSlot { slot: u32 },
    #[error("canonical static local slot {slot} was read before initialization")]
    UninitializedLocalSlot { slot: u32 },
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
    #[error("missing checked match resolution for expression {0:?}")]
    MissingMatchResolution(ExpressionId),
    #[error("missing checked static iteration resolution for expression {0:?}")]
    MissingStaticIterationResolution(ExpressionId),
    #[error("checked member projection failed: {0}")]
    MemberProjection(Box<str>),
    #[error("static iteration is not finite")]
    StaticIterationNotFinite,
    #[error("exhaustive static match had no matching branch")]
    UnmatchedExhaustiveStaticMatch,
    #[error("static specialization environment error: {0}")]
    SpecializationEnvironment(#[from] crate::static_eval::application::StaticSpecializationEnvironmentError),
}

/// Terminal state of one bounded static evaluation request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaticEvalOutcome {
    Reduced(crate::static_value::StaticValueRef),
    /// A checked canonical kinded result. TypeIds never enter the evaluator's
    /// ordinary StaticValueRef stack.
    ReducedKinded(crate::types::id::TypeId),
    Symbolic(crate::static_eval::residual::StaticResidualRef),
    Invalid(StaticEvalError),
    /// The checked static operation could not be resolved with available evidence.
    Unknown(UnknownReason),
    /// A canonical dependency or analysis state prevents this request from completing.
    Blocked(crate::types::outcome::BlockReason),
    /// The request crosses a dynamic language boundary.
    Dynamic(crate::types::outcome::DynamicBoundaryObligation),
    Recursive(crate::static_eval::application::StaticCycle),
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
