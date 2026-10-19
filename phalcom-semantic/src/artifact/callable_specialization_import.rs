//! Certified PERF003 P7 callable call-specialization reconstruction.
//!
//! Specialization is an expression sidecar: it binds canonical type/row/static
//! arguments and generic conformance evidence to the exact selected callable.
//! It is staged separately from base expression facts so no partial
//! ExpressionAnalysis is published before dispatch/proof authority is complete.

use super::callable_import::ImportedCallableAuthority;
use super::identity::StableOwnerCatalog;
use super::type_import::ReconstructedTypeGraph;
use super::witness_import::ImportedGenericCallConformanceBinding;
use crate::identity::{BodyId, CallableId, ExpressionId, LocalExpressionId};
use crate::types::id::TypeParameterId;
use crate::types::store::StaticArgument;
use phalcom_compiler_artifact::semantic::type_store::StaticArgumentV1;
use phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV1;
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;

/// Relocated P7 call specialization for one expression.
#[derive(Clone, Debug)]
pub struct ImportedCallSpecialization {
    pub expression: ExpressionId,
    pub callable: CallableId,
    pub bindings: Box<[(TypeParameterId, StaticArgument)]>,
    pub conformances: Box<[ImportedGenericCallConformanceBinding]>,
}

#[derive(Clone, Debug)]
pub struct ImportedCallableSpecializationOwner {
    pub callable: CallableId,
    pub body: BodyId,
    pub specializations: Box<[ImportedCallSpecialization]>,
}

/// Sparse callable-keyed specialization authority.
#[derive(Clone, Debug, Default)]
pub struct ImportedCallableSpecializationAuthority {
    owners: Box<[ImportedCallableSpecializationOwner]>,
}

impl ImportedCallableSpecializationAuthority {
    pub fn callable_len(&self) -> usize {
        self.owners.len()
    }

    pub fn specialization_len(&self) -> usize {
        self.owners
            .iter()
            .map(|owner| owner.specializations.len())
            .sum()
    }

    pub fn specialization(
        &self,
        callable: &CallableId,
        expression_ordinal: u32,
    ) -> Option<&ImportedCallSpecialization> {
        let owner = self
            .owners
            .binary_search_by(|owner| owner.callable.cmp(callable))
            .ok()
            .and_then(|index| self.owners.get(index))?;
        owner
            .specializations
            .binary_search_by_key(&LocalExpressionId(expression_ordinal), |specialization| {
                specialization.expression.local
            })
            .ok()
            .and_then(|index| owner.specializations.get(index))
    }
}

pub(crate) fn reconstruct_callable_specializations(
    records: &[CallableAnalysisRecordV1],
    callables: &ImportedCallableAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedCallableSpecializationAuthority, ArtifactError> {
    let mut imported = allocations
        .vector::<ImportedCallableSpecializationOwner>(records.len())
        .map_err(ArtifactError::Allocation)?;

    for (body_index, record) in records.iter().enumerate() {
        let owner = owners.callable(&record.callable)?;
        let body = BodyId(
            u32::try_from(body_index)
                .map_err(|_| invalid("callable body ordinal exceeds portable range"))?,
        );
        let mut specializations = allocations
            .vector::<ImportedCallSpecialization>(
                record
                    .expressions
                    .iter()
                    .filter(|expression| expression.call_specialization.is_some())
                    .count(),
            )
            .map_err(ArtifactError::Allocation)?;

        for (index, expression) in record.expressions.iter().enumerate() {
            if usize::try_from(expression.ordinal).ok() != Some(index) {
                return Err(invalid(
                    "call specialization owner expressions are not dense and ordinal-sorted",
                ));
            }
            let Some(record) = expression.call_specialization.as_ref() else {
                continue;
            };

            let callable = owners.callable(&record.callable)?;
            if callables.signature(&callable).is_none() {
                return Err(invalid(
                    "call specialization selected callable has no imported canonical signature",
                ));
            }

            let mut bindings = allocations
                .vector::<(TypeParameterId, StaticArgument)>(record.bindings.len())
                .map_err(ArtifactError::Allocation)?;
            for binding in &record.bindings {
                let parameter = *types
                    .parameters
                    .get(binding.parameter.0 as usize)
                    .ok_or_else(|| {
                        invalid("call specialization references an absent type parameter")
                    })?;
                if bindings.iter().any(|(existing, _)| *existing == parameter) {
                    return Err(invalid(
                        "call specialization contains duplicate relocated parameter bindings",
                    ));
                }
                let argument = match &binding.argument {
                    StaticArgumentV1::Term { term } => StaticArgument::Term(
                        *types
                            .types
                            .get(term.0 as usize)
                            .ok_or_else(|| {
                                invalid("call specialization term references an absent type")
                            })?,
                    ),
                    StaticArgumentV1::RecordRow { row } => StaticArgument::RecordRow(
                        *types
                            .rows
                            .get(row.0 as usize)
                            .ok_or_else(|| {
                                invalid("call specialization references an absent record row")
                            })?,
                    ),
                    StaticArgumentV1::DynamicType => StaticArgument::DynamicType,
                };
                bindings.push((parameter, argument));
            }

            let mut conformances = allocations
                .vector::<ImportedGenericCallConformanceBinding>(record.conformances.len())
                .map_err(ArtifactError::Allocation)?;
            for binding in &record.conformances {
                conformances.push(super::witness_import::reconstruct_conformance_binding(
                    binding,
                    types,
                    owners,
                    record_conformance_evidence_count(record, expression, records)?,
                    allocations,
                )?);
            }

            specializations.push(ImportedCallSpecialization {
                expression: ExpressionId::new(body, LocalExpressionId(expression.ordinal)),
                callable,
                bindings: allocations
                    .boxed_slice(bindings)
                    .map_err(ArtifactError::Allocation)?,
                conformances: allocations
                    .boxed_slice(conformances)
                    .map_err(ArtifactError::Allocation)?,
            });
        }

        imported.push(ImportedCallableSpecializationOwner {
            callable: owner,
            body,
            specializations: allocations
                .boxed_slice(specializations)
                .map_err(ArtifactError::Allocation)?,
        });
    }

    imported.sort_by(|left, right| left.callable.cmp(&right.callable));
    if imported
        .windows(2)
        .any(|pair| pair[0].callable == pair[1].callable)
    {
        return Err(invalid(
            "duplicate callable specialization owner after stable-owner relocation",
        ));
    }

    Ok(ImportedCallableSpecializationAuthority {
        owners: allocations
            .boxed_slice(imported)
            .map_err(ArtifactError::Allocation)?,
    })
}

/// Returns the evidence table cardinality for the exact owning analysis record.
///
/// Kept as a helper so a future split representation cannot accidentally use
/// another callable's ordinal graph.
fn record_conformance_evidence_count(
    _specialization: &phalcom_compiler_artifact::semantic::world::CallableCallSpecializationV1,
    expression: &phalcom_compiler_artifact::semantic::world::CallableExpressionRowV1,
    records: &[CallableAnalysisRecordV1],
) -> Result<usize, ArtifactError> {
    let owner = records
        .iter()
        .find(|record| {
            record
                .expressions
                .get(expression.ordinal as usize)
                .is_some_and(|candidate| std::ptr::eq(candidate, expression))
        })
        .ok_or_else(|| invalid("call specialization lost its owning analysis record"))?;
    Ok(owner.conformance_evidence.len())
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
