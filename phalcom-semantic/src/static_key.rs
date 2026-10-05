//! Projection of checker-authorized static expressions to canonical Record keys.

use crate::db::{CancellationToken, QueryBudget, QueryOutcome};
use crate::field_key::FieldKey;
use crate::static_constant::{CheckedStaticExpression, StaticConstantId, StaticConstantTable};
use crate::static_definition::StaticDefinitionTable;
use crate::static_eval::{StaticEvalError, StaticEvalOutcome, StaticEvaluator};
use crate::static_value::{StaticValueStore, StaticValueStoreError};
use crate::types::store::TypeStore;
use std::sync::Arc;

/// Outcome of reducing a checked static expression at a Record-key site.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaticKeyProjection {
    /// Evaluation produced one canonical key from the closed FieldKey domain.
    Concrete(FieldKey),
    /// Evaluation is valid but requires a later static stage to reduce.
    DeferredToP7,
    /// The checked expression or reduced value cannot serve as a Record key.
    Invalid(StaticKeyProjectionError),
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum StaticKeyProjectionError {
    #[error("static key expression has checker diagnostics")]
    CheckerDiagnostics,
    #[error("selected static constant `{0}` has no reduced current value")]
    ConstantNotReduced(StaticConstantId),
    #[error("static key expression produced a kinded result")]
    KindedResult,
    #[error("static key evaluation failed: {0}")]
    Evaluation(StaticEvalError),
    #[error("static key evaluation was recursive")]
    Recursive,
    #[error("reduced static value is not a Record key: {0}")]
    InvalidFieldKey(StaticValueStoreError),
}

/// Reduces one ordinary-checker-approved expression to an owned canonical
/// `FieldKey`. Constant dependencies come from checker-selected binding IDs
/// retained in the checked expression; this function does no name lookup.
pub fn project_checked_static_key(
    checked: &CheckedStaticExpression,
    constants: &StaticConstantTable,
    definitions: &StaticDefinitionTable,
    types: &TypeStore,
    data_semantics: &crate::data_semantics::DataSemanticTable,
    enum_semantics: &crate::enum_semantics::EnumSemanticTable,
    values: &mut StaticValueStore,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
) -> QueryOutcome<StaticKeyProjection> {
    if !checked.diagnostics.is_empty() {
        return QueryOutcome::Ready(StaticKeyProjection::Invalid(StaticKeyProjectionError::CheckerDiagnostics));
    }

    for symbol in checked.static_constant_reads.iter() {
        let id = StaticConstantId(symbol.clone());
        let Some(product) = constants.get(&id) else {
            return QueryOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency);
        };
        if product.value.is_none() {
            return QueryOutcome::Ready(StaticKeyProjection::Invalid(StaticKeyProjectionError::ConstantNotReduced(id)));
        }
    }

    let outcome = {
        let mut evaluator = StaticEvaluator::for_checked_expression(checked, values, budget, cancellation)
            .with_static_constants(constants)
            .with_static_definitions(definitions)
            .with_type_store(types)
            .with_semantic_tables(types, data_semantics, enum_semantics);
        evaluator.evaluate()
    };

    match outcome {
        StaticEvalOutcome::Reduced(value) => match values.field_key_of(value) {
            Ok(key) => QueryOutcome::Ready(StaticKeyProjection::Concrete(key)),
            Err(error) => QueryOutcome::Ready(StaticKeyProjection::Invalid(StaticKeyProjectionError::InvalidFieldKey(error))),
        },
        StaticEvalOutcome::Symbolic => QueryOutcome::Ready(StaticKeyProjection::DeferredToP7),
        StaticEvalOutcome::ReducedKinded(_) => QueryOutcome::Ready(StaticKeyProjection::Invalid(StaticKeyProjectionError::KindedResult)),
        StaticEvalOutcome::Invalid(error) => QueryOutcome::Ready(StaticKeyProjection::Invalid(StaticKeyProjectionError::Evaluation(error))),
        StaticEvalOutcome::Recursive => QueryOutcome::Ready(StaticKeyProjection::Invalid(StaticKeyProjectionError::Recursive)),
        StaticEvalOutcome::BudgetExceeded(report) => QueryOutcome::BudgetExceeded(report),
        StaticEvalOutcome::Cancelled => QueryOutcome::Cancelled,
    }
}

/// Retains all checker-produced semantic dependencies for an owner query.
pub fn exact_static_key_dependencies(checked: &CheckedStaticExpression) -> Arc<[crate::checker::analysis::SemanticDependency]> {
    checked.dependencies.clone()
}
