//! Certified PERF003 P7 callable expression base-fact reconstruction.
//!
//! P7 intentionally decomposed callable expression import into base facts,
//! proof/dispatch authority, call specialization, and other resolution indexes.
//! ARTF001 keeps that boundary explicit. This module reconstructs every base
//! fact into current-main identities and records an exact manifest of deferred
//! proof-bearing sidecars; it does not claim a complete `ExpressionAnalysis`.

use super::callable_binding_import::ImportedCallableBindingAuthority;
use super::identity::StableOwnerCatalog;
use super::static_definition_import::ImportedStaticDefinitionAuthority;
use super::type_import::ReconstructedTypeGraph;
use crate::checker::analysis::{AnalysisStatus};
use crate::checker::causal::{CausalInvalidity, SuppressionCause};
use crate::checker::flow::BindingReadSafety;
use crate::dispatch::ConditionalDispatchSelection;
use crate::identity::{
    BindingId, BodyId, CallableId, DiagnosticCauseId, ExplanationId, ExpressionId,
    LocalExpressionId,
};
use crate::types::denotation::SemanticDenotation;
use crate::types::evidence::{DynamicReason, TypeKnowledge};
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::semantic::aggregates::DynamicReasonV1;
use phalcom_compiler_artifact::semantic::world::{
    CallableAnalysisRecordV1, CallableBindingReadSafetyV1, CallableCausalInvalidityV1,
    CallableExpressionRowV1, CallableExpressionStatusV1, CallableSuppressionCauseV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;
use std::sync::Arc;

/// Exact presence/count manifest for expression authority that belongs to the
/// separate P7 proof/dispatch/specialization import stages.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ImportedExpressionDeferredAuthority {
    pub conditional_capability: bool,
    pub trait_dispatch: bool,
    pub trait_dispatch_candidate_count: u32,
    pub contextual_realization: bool,
    pub call_specialization: bool,
}

impl ImportedExpressionDeferredAuthority {
    pub fn is_empty(self) -> bool {
        !self.conditional_capability
            && !self.trait_dispatch
            && self.trait_dispatch_candidate_count == 0
            && !self.contextual_realization
            && !self.call_specialization
    }
}

/// Relocated non-proof portion of one P7 expression row.
#[derive(Clone, Debug)]
pub struct ImportedCallableExpressionBase {
    pub id: ExpressionId,
    pub range: SourceRange,
    pub knowledge: TypeKnowledge,
    pub callable: Option<CallableId>,
    pub conditional_dispatch: Option<ConditionalDispatchSelection>,
    pub denotation: Option<SemanticDenotation>,
    pub status: AnalysisStatus,
    pub causal_invalidity: CausalInvalidity,
    pub explanation: Option<ExplanationId>,
    pub binding_read_safety: Option<BindingReadSafety>,
    pub binding_read: Option<BindingId>,
    pub binding_write: Option<BindingId>,
    pub deferred: ImportedExpressionDeferredAuthority,
}

#[derive(Clone, Debug)]
pub struct ImportedCallableExpressionOwner {
    pub callable: CallableId,
    pub body: BodyId,
    pub expressions: Arc<[ImportedCallableExpressionBase]>,
}

/// Callable-keyed base expression authority. This value is intentionally not a
/// `CallableAnalysisTable` until every deferred sidecar is reconstructed.
#[derive(Clone, Debug, Default)]
pub struct ImportedCallableExpressionAuthority {
    owners: Box<[ImportedCallableExpressionOwner]>,
}

impl ImportedCallableExpressionAuthority {
    pub fn callable_len(&self) -> usize {
        self.owners.len()
    }

    pub fn expression_len(&self) -> usize {
        self.owners.iter().map(|owner| owner.expressions.len()).sum()
    }

    pub fn deferred_expression_len(&self) -> usize {
        self.owners
            .iter()
            .flat_map(|owner| owner.expressions.iter())
            .filter(|expression| !expression.deferred.is_empty())
            .count()
    }

    pub fn expression(
        &self,
        callable: &CallableId,
        ordinal: u32,
    ) -> Option<&ImportedCallableExpressionBase> {
        let owner = self
            .owners
            .binary_search_by(|owner| owner.callable.cmp(callable))
            .ok()
            .and_then(|index| self.owners.get(index))?;
        owner
            .expressions
            .get(ordinal as usize)
            .filter(|expression| expression.id.local == LocalExpressionId(ordinal))
    }
}

pub(crate) fn reconstruct_callable_expression_bases(
    records: &[CallableAnalysisRecordV1],
    bindings: &ImportedCallableBindingAuthority,
    static_definitions: &ImportedStaticDefinitionAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedCallableExpressionAuthority, ArtifactError> {
    let mut imported = allocations
        .vector::<ImportedCallableExpressionOwner>(records.len())
        .map_err(ArtifactError::Allocation)?;

    for (body_index, record) in records.iter().enumerate() {
        let callable = owners.callable(&record.callable)?;
        let body = BodyId(
            u32::try_from(body_index)
                .map_err(|_| invalid("callable body ordinal exceeds portable range"))?,
        );
        let expected_bindings = bindings
            .rows(&callable)
            .ok_or_else(|| invalid("callable expression owner has no imported binding authority"))?;
        if expected_bindings.len() != record.bindings.len() {
            return Err(invalid(
                "callable expression owner binding cardinality disagrees with imported binding authority",
            ));
        }

        let mut expressions = allocations
            .vector::<ImportedCallableExpressionBase>(record.expressions.len())
            .map_err(ArtifactError::Allocation)?;
        for (index, expression) in record.expressions.iter().enumerate() {
            if usize::try_from(expression.ordinal).ok() != Some(index) {
                return Err(invalid(
                    "callable expression rows are not dense and ordinal-sorted",
                ));
            }
            expressions.push(reconstruct_expression(
                expression,
                body,
                &callable,
                expected_bindings.len(),
                static_definitions,
                types,
                owners,
                allocations,
            )?);
        }
        imported.push(ImportedCallableExpressionOwner {
            callable,
            body,
            expressions: allocations
                .arc_slice(expressions)
                .map_err(ArtifactError::Allocation)?,
        });
    }

    imported.sort_by(|left, right| left.callable.cmp(&right.callable));
    if imported
        .windows(2)
        .any(|pair| pair[0].callable == pair[1].callable)
    {
        return Err(invalid(
            "duplicate callable expression authority after stable-owner relocation",
        ));
    }

    Ok(ImportedCallableExpressionAuthority {
        owners: allocations
            .boxed_slice(imported)
            .map_err(ArtifactError::Allocation)?,
    })
}

fn reconstruct_expression(
    expression: &CallableExpressionRowV1,
    body: BodyId,
    owner: &CallableId,
    binding_count: usize,
    static_definitions: &ImportedStaticDefinitionAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedCallableExpressionBase, ArtifactError> {
    let range = source_range(&expression.range, "callable expression")?;
    let knowledge =
        super::callable_import::import_type_knowledge(&expression.knowledge, types, allocations)?;
    let callable = expression
        .callable
        .as_ref()
        .map(|callable| owners.callable(callable))
        .transpose()?;
    let conditional_dispatch = expression
        .conditional_dispatch
        .as_ref()
        .map(|selection| super::denotation_import::conditional_selection(selection, owners))
        .transpose()?;
    let denotation = expression
        .denotation
        .as_ref()
        .map(|denotation| {
            super::denotation_import::reconstruct_semantic_denotation(
                denotation,
                types,
                static_definitions,
                owners,
                allocations,
            )
        })
        .transpose()?;

    let binding_read = binding_id(expression.binding_read, binding_count, "binding read")?;
    let binding_write = binding_id(expression.binding_write, binding_count, "binding write")?;

    if conditional_dispatch
        .as_ref()
        .is_some_and(|selection| selection.callable.module() != owner.module())
        && expression.conditional_capability.is_none()
    {
        // A cross-module conditional selection without its proof capability
        // must remain explicitly deferred rather than being treated as an
        // ordinary direct dispatch fact.
    }

    let deferred = ImportedExpressionDeferredAuthority {
        conditional_capability: expression.conditional_capability.is_some(),
        trait_dispatch: expression.trait_dispatch.is_some(),
        trait_dispatch_candidate_count: u32::try_from(
            expression
                .trait_dispatch_candidates
                .as_ref()
                .map_or(0, Vec::len),
        )
        .map_err(|_| invalid("trait dispatch candidate count exceeds u32"))?,
        contextual_realization: expression.contextual_realization.is_some(),
        call_specialization: expression.call_specialization.is_some(),
    };

    Ok(ImportedCallableExpressionBase {
        id: ExpressionId::new(body, LocalExpressionId(expression.ordinal)),
        range,
        knowledge,
        callable,
        conditional_dispatch,
        denotation,
        status: expression_status(&expression.status, types, owners, allocations)?,
        causal_invalidity: causal_invalidity(&expression.causal_invalidity),
        explanation: expression.explanation.map(ExplanationId),
        binding_read_safety: expression.binding_read_safety.as_ref().map(read_safety),
        binding_read,
        binding_write,
        deferred,
    })
}

fn expression_status(
    status: &CallableExpressionStatusV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<AnalysisStatus, ArtifactError> {
    Ok(match status {
        CallableExpressionStatusV1::Ready => AnalysisStatus::Ready,
        CallableExpressionStatusV1::Invalid { cause } => {
            AnalysisStatus::Invalid(DiagnosticCauseId(*cause))
        }
        CallableExpressionStatusV1::Suppressed { cause } => {
            AnalysisStatus::Suppressed(match cause {
                CallableSuppressionCauseV1::One { cause } => {
                    SuppressionCause::One(DiagnosticCauseId(*cause))
                }
                CallableSuppressionCauseV1::Multiple => SuppressionCause::Multiple,
            })
        }
        CallableExpressionStatusV1::Blocked { reason } => AnalysisStatus::Blocked(
            super::callable_binding_import::block_reason(reason, types, owners, allocations)?,
        ),
        CallableExpressionStatusV1::DynamicBoundary { reason } => {
            AnalysisStatus::DynamicBoundary(match reason {
                DynamicReasonV1::ExplicitEscape => DynamicReason::ExplicitEscape,
                DynamicReasonV1::DynamicRestPack => DynamicReason::DynamicRestPack,
                DynamicReasonV1::RuntimeReflection => DynamicReason::RuntimeReflection,
            })
        }
        CallableExpressionStatusV1::Cancelled => AnalysisStatus::Cancelled,
        CallableExpressionStatusV1::BudgetExceeded { report } => AnalysisStatus::BudgetExceeded(
            super::callable_binding_import::budget_report(report),
        ),
    })
}

fn causal_invalidity(value: &CallableCausalInvalidityV1) -> CausalInvalidity {
    match value {
        CallableCausalInvalidityV1::Clean => CausalInvalidity::Clean,
        CallableCausalInvalidityV1::One { cause } => {
            CausalInvalidity::One(DiagnosticCauseId(*cause))
        }
        CallableCausalInvalidityV1::Multiple => CausalInvalidity::Multiple,
    }
}

fn read_safety(value: &CallableBindingReadSafetyV1) -> BindingReadSafety {
    match value {
        CallableBindingReadSafetyV1::ProvenInitialized => BindingReadSafety::ProvenInitialized,
        CallableBindingReadSafetyV1::DefinitelyUninitialized => {
            BindingReadSafety::DefinitelyUninitialized
        }
        CallableBindingReadSafetyV1::RuntimeCheckRequired => {
            BindingReadSafety::RuntimeCheckRequired
        }
    }
}

fn binding_id(
    ordinal: Option<u32>,
    binding_count: usize,
    family: &'static str,
) -> Result<Option<BindingId>, ArtifactError> {
    ordinal
        .map(|ordinal| {
            if ordinal as usize >= binding_count {
                return Err(invalid(format!(
                    "callable expression {family} references an absent binding",
                )));
            }
            Ok(BindingId(ordinal))
        })
        .transpose()
}

fn source_range(
    range: &phalcom_compiler_artifact::module_world::SpanV1,
    family: &'static str,
) -> Result<SourceRange, ArtifactError> {
    if range.start > range.end {
        return Err(invalid(format!("{family} range is inverted")));
    }
    Ok(SourceRange {
        start: usize::try_from(range.start)
            .map_err(|_| invalid(format!("{family} start exceeds address space")))?,
        end: usize::try_from(range.end)
            .map_err(|_| invalid(format!("{family} end exceeds address space")))?,
    })
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
