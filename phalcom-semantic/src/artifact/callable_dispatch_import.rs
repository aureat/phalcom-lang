//! Certified PERF003 P7 callable conditional-dispatch capability import.
//!
//! This is the first proof-bearing expression sidecar. It joins the exact
//! relocated conditional selection with the P7 inherent specialization and its
//! conformance provenance. Trait dispatch/contextual realization remain separate
//! sidecars and are not implied by this authority.

use super::callable_import::ImportedCallableAuthority;
use super::identity::StableOwnerCatalog;
use super::type_import::ReconstructedTypeGraph;
use super::witness_import::ImportedInherentSpecialization;
use crate::dispatch::ConditionalDispatchSelection;
use crate::identity::{BodyId, CallableId, ExpressionId, LocalExpressionId};
use phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2;
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;

/// Exact conditional-call capability for one expression.
#[derive(Clone, Debug)]
pub struct ImportedConditionalCallCapability {
    pub expression: ExpressionId,
    pub selection: ConditionalDispatchSelection,
    pub applicability: ImportedInherentSpecialization,
}

#[derive(Clone, Debug)]
pub struct ImportedCallableConditionalDispatchOwner {
    pub callable: CallableId,
    pub body: BodyId,
    pub capabilities: Box<[ImportedConditionalCallCapability]>,
}

/// Sparse callable-keyed conditional-dispatch capability authority.
#[derive(Clone, Debug, Default)]
pub struct ImportedCallableConditionalDispatchAuthority {
    owners: Box<[ImportedCallableConditionalDispatchOwner]>,
}

impl ImportedCallableConditionalDispatchAuthority {
    pub fn callable_len(&self) -> usize {
        self.owners.len()
    }

    pub fn capability_len(&self) -> usize {
        self.owners.iter().map(|owner| owner.capabilities.len()).sum()
    }

    pub fn capability(&self, callable: &CallableId, expression_ordinal: u32) -> Option<&ImportedConditionalCallCapability> {
        let owner = self
            .owners
            .binary_search_by(|owner| owner.callable.cmp(callable))
            .ok()
            .and_then(|index| self.owners.get(index))?;
        owner
            .capabilities
            .binary_search_by_key(&LocalExpressionId(expression_ordinal), |capability| capability.expression.local)
            .ok()
            .and_then(|index| owner.capabilities.get(index))
    }
}

pub(crate) fn reconstruct_callable_conditional_dispatch(
    records: &[CallableAnalysisRecordV2],
    callables: &ImportedCallableAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedCallableConditionalDispatchAuthority, ArtifactError> {
    let mut imported = allocations
        .vector::<ImportedCallableConditionalDispatchOwner>(records.len())
        .map_err(ArtifactError::Allocation)?;

    for (body_index, analysis) in records.iter().enumerate() {
        let owner = owners.callable(&analysis.callable)?;
        let body = BodyId(u32::try_from(body_index).map_err(|_| invalid("callable body ordinal exceeds portable range"))?);
        let mut capabilities = allocations
            .vector::<ImportedConditionalCallCapability>(
                analysis
                    .expressions
                    .iter()
                    .filter(|expression| expression.conditional_capability.is_some())
                    .count(),
            )
            .map_err(ArtifactError::Allocation)?;

        for (index, expression) in analysis.expressions.iter().enumerate() {
            if usize::try_from(expression.ordinal).ok() != Some(index) {
                return Err(invalid("conditional-dispatch owner expressions are not dense and ordinal-sorted"));
            }
            let Some(capability) = expression.conditional_capability.as_ref() else {
                continue;
            };
            if expression.conditional_dispatch.as_ref() != Some(&capability.selection) {
                return Err(invalid("conditional call capability disagrees with its expression selection"));
            }
            if capability.selection.impl_id != capability.applicability.impl_id {
                return Err(invalid("conditional call capability selection and applicability use different implementations"));
            }

            let selection = super::denotation_import::conditional_selection(&capability.selection, owners)?;
            if callables.signature(&selection.callable).is_none() {
                return Err(invalid("conditional call capability selected callable has no imported canonical signature"));
            }
            let applicability = super::witness_import::reconstruct_inherent_specialization(
                &capability.applicability,
                types,
                owners,
                analysis.conformance_evidence.len(),
                allocations,
            )?;
            if selection.impl_id != applicability.impl_id {
                return Err(invalid("relocated conditional selection disagrees with its inherent specialization"));
            }

            capabilities.push(ImportedConditionalCallCapability {
                expression: ExpressionId::new(body, LocalExpressionId(expression.ordinal)),
                selection,
                applicability,
            });
        }

        imported.push(ImportedCallableConditionalDispatchOwner {
            callable: owner,
            body,
            capabilities: allocations.boxed_slice(capabilities).map_err(ArtifactError::Allocation)?,
        });
    }

    imported.sort_by(|left, right| left.callable.cmp(&right.callable));
    if imported.windows(2).any(|pair| pair[0].callable == pair[1].callable) {
        return Err(invalid("duplicate conditional-dispatch owner after stable-owner relocation"));
    }

    Ok(ImportedCallableConditionalDispatchAuthority {
        owners: allocations.boxed_slice(imported).map_err(ArtifactError::Allocation)?,
    })
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
