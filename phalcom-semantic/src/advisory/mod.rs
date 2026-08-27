//! Compiler-owned advisory runtime-shape facts.
//!
//! Advisory facts help tooling describe likely runtime values. They are not
//! formal types and never participate in checker acceptance, diagnostics, or
//! proof discharge.

mod agreement;
mod analyzer;
mod fact;
mod flow;
mod formal;
mod parameters;
mod provenance;
mod query;
mod shape;
mod solver;
mod summary;
mod workspace;

pub use agreement::{AdvisoryAgreement, compare_expression, compare_known};
pub use analyzer::{AdvisoryBuiltins, AdvisoryCallArgument, AdvisoryCallObservation, AdvisoryExpressionContext, analyze_expr};
pub use fact::{AdvisoryConfidence, AdvisoryFact, AdvisoryLiteral};
pub use flow::{AdvisoryFlowContext, AdvisoryFlowProduct, analyze_statements};
pub use formal::{advisory_fact_from_formal, advisory_shape_from_formal, advisory_shape_from_formal_for_receiver};
pub use parameters::{AdvisoryContributionSource, AdvisoryParameterContributions, AdvisoryParameterFactDelta, AdvisoryParameterSlot};
pub use provenance::AdvisoryOrigin;
pub use query::AdvisoryQuery;
pub use shape::{CapturedMethodFamilyShape, MAX_SHAPE_UNION, ValueShape};
pub use solver::{AdvisorySolver, AdvisorySolverBudget, AdvisorySolverNode, AdvisorySolverResult};
pub use summary::{AdvisoryCallableSummary, AdvisoryProductStatus, AdvisorySummaryEffects};
pub use workspace::{AdvisoryModuleProduct, AdvisoryTargetResolution, AdvisoryWorkspace};
