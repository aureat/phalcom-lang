//! Exact semantic authorization for compile-time intrinsic operations.

use crate::checker::analysis::{AnalysisStatus, ExpressionAnalysisIndex};
use crate::identity::{CallableId, DispatchSide, ExpressionId};
use phalcom_common::selector::Selector;
use phalcom_native_meta::{NativeDispatch, UniverseKey};
use std::collections::BTreeMap;
use thiserror::Error;

/// A compiler-owned operation that the static evaluator may execute.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StaticIntrinsicOperation {
    /// Addition selected through the canonical `Number.+(_)` callable.
    NumberAdd,
}

/// A statically executable operation attached to one exact selected callable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticOperationResolution {
    pub expression: ExpressionId,
    pub callable: CallableId,
    pub operation: StaticIntrinsicOperation,
}

/// Per-expression static operation facts derived from ready ordinary analysis.
pub type StaticOperationResolutionIndex = BTreeMap<ExpressionId, StaticOperationResolution>;

/// Registry construction failed because the trusted canonical native surface
/// no longer contains a required operation with its expected selector shape.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum StaticIntrinsicRegistryError {
    #[error("canonical Number addition surface is missing")]
    MissingNumberAddition,
    #[error("canonical Number addition selector is malformed: {0}")]
    InvalidNumberAdditionSelector(String),
}

/// Compiler-owned allowlist keyed by canonical callable identity.
///
/// Construction is intentionally limited to the canonical native surface;
/// callers can query membership but cannot add user-selected identities.
#[derive(Clone, Debug, Default)]
pub struct StaticIntrinsicRegistry {
    operations: BTreeMap<CallableId, StaticIntrinsicOperation>,
}

impl StaticIntrinsicRegistry {
    /// Builds the initial P2 allowlist from an exact canonical Universe row.
    /// The native surface selector is used only to construct the canonical
    /// callable key; subsequent authorization compares the complete `CallableId`.
    pub fn from_canonical_core() -> Result<Self, StaticIntrinsicRegistryError> {
        let row = phalcom_native_surface::find_native_surface(UniverseKey::Number, NativeDispatch::Instance, "+(_)")
            .ok_or(StaticIntrinsicRegistryError::MissingNumberAddition)?;
        let selector =
            Selector::try_decode_exact(row.selector()).map_err(|error| StaticIntrinsicRegistryError::InvalidNumberAdditionSelector(error.to_string()))?;
        let callable = CallableId::method(crate::core_surface::universe_declaration(UniverseKey::Number), selector, DispatchSide::Instance);
        Ok(Self {
            operations: BTreeMap::from([(callable, StaticIntrinsicOperation::NumberAdd)]),
        })
    }

    /// Returns the operation registered for this complete callable identity.
    pub fn operation_for(&self, callable: &CallableId) -> Option<StaticIntrinsicOperation> {
        self.operations.get(callable).copied()
    }

    /// Publishes an authorization fact only for a ready expression whose
    /// ordinary checker recorded an exact callable selected by dispatch.
    pub(crate) fn resolve_ready_expressions(&self, expressions: &ExpressionAnalysisIndex) -> StaticOperationResolutionIndex {
        expressions
            .iter()
            .filter_map(|(expression, analysis)| {
                if !matches!(analysis.status, AnalysisStatus::Ready) {
                    return None;
                }
                let callable = analysis.callable.as_ref()?;
                let operation = self.operation_for(callable)?;
                Some((
                    *expression,
                    StaticOperationResolution {
                        expression: *expression,
                        callable: callable.clone(),
                        operation,
                    },
                ))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::analysis::ExpressionAnalysis;
    use crate::identity::{BodyId, DiagnosticCauseId, LocalExpressionId};
    use crate::types::evidence::{EvidenceOrigin, TypeKnowledge};
    use crate::types::id::TypeId;
    use phalcom_common::range::SourceRange;

    #[test]
    fn ready_analysis_projection_keeps_exact_callable_and_skips_invalid_sites() {
        let registry = StaticIntrinsicRegistry::from_canonical_core().unwrap();
        let operation = registry.operations.keys().next().unwrap().clone();
        let ready_id = ExpressionId::new(BodyId(4), LocalExpressionId(0));
        let invalid_id = ExpressionId::new(BodyId(4), LocalExpressionId(1));
        let make_analysis = |id| {
            ExpressionAnalysis::ready(
                id,
                SourceRange { start: 0, end: 1 },
                TypeKnowledge::established(TypeId(0), EvidenceOrigin::CallableSignature),
            )
        };
        let mut ready = make_analysis(ready_id);
        ready.callable = Some(operation.clone());
        let mut invalid = make_analysis(invalid_id);
        invalid.callable = Some(operation.clone());
        invalid.status = AnalysisStatus::Invalid(DiagnosticCauseId(1));
        let expressions = ExpressionAnalysisIndex::from([(ready_id, ready), (invalid_id, invalid)]);

        let resolved = registry.resolve_ready_expressions(&expressions);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[&ready_id].callable, operation);
        assert!(!resolved.contains_key(&invalid_id));
    }
}
