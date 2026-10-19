//! Certified PERF003 P7 enum-requirement reconstruction adapted to ARTF001.
//!
//! Enum requirements are snapshot semantic authority, not independently reusable
//! query roots in the current artifact profile. This importer preserves P7's
//! owner/order/status validation while staging immutable rows for T4 publication.

use super::identity::StableOwnerCatalog;
use super::semantic_import::ImportedSemanticAggregates;
use super::type_import::ReconstructedTypeGraph;
use crate::diagnostic::SemanticSourceSpan;
use crate::enum_requirements::{
    CaseRequirementResult, CaseRequirementStatus, EnumRequirement, EnumRequirementId,
};
use crate::identity::{CallableOwnerId, DeclarationId, DispatchSide};
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::semantic::world::{
    CaseRequirementStatusV1, EnumRequirementOwnerV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;
use std::sync::Arc;

#[derive(Clone, Debug)]
struct ImportedEnumRequirementOwner {
    owner: DeclarationId,
    requirements: Arc<[EnumRequirement]>,
    case_statuses: Arc<[CaseRequirementResult]>,
}

/// Current-main closed-enum requirement authority reconstructed from P7 rows.
///
/// Rows stay in a sorted immutable staging index until T4 constructs the one
/// imported semantic generation. This is not a second query database.
#[derive(Clone, Debug, Default)]
pub struct ImportedEnumRequirementAuthority {
    owners: Box<[ImportedEnumRequirementOwner]>,
}

impl ImportedEnumRequirementAuthority {
    pub fn owner_len(&self) -> usize {
        self.owners.len()
    }

    pub fn requirement_len(&self) -> usize {
        self.owners.iter().map(|row| row.requirements.len()).sum()
    }

    pub fn case_status_len(&self) -> usize {
        self.owners.iter().map(|row| row.case_statuses.len()).sum()
    }

    pub(crate) fn requirements(&self, owner: &DeclarationId) -> Option<&Arc<[EnumRequirement]>> {
        self.owners
            .binary_search_by(|row| row.owner.cmp(owner))
            .ok()
            .map(|index| &self.owners[index].requirements)
    }

    pub(crate) fn case_statuses(&self, owner: &DeclarationId) -> Option<&Arc<[CaseRequirementResult]>> {
        self.owners
            .binary_search_by(|row| row.owner.cmp(owner))
            .ok()
            .map(|index| &self.owners[index].case_statuses)
    }
}

/// Port of certified P7 `import_enum_requirements`.
///
/// The current branch keeps the transport ordering contract but delays creation
/// of the live HashMap-backed `EnumRequirementTable` until T4's publication
/// boundary. Every stable owner is relocated through the exact project catalog.
pub(crate) fn reconstruct_enum_requirement_authority(
    records: &[EnumRequirementOwnerV1],
    aggregates: &ImportedSemanticAggregates,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedEnumRequirementAuthority, ArtifactError> {
    let mut imported = allocations
        .vector::<ImportedEnumRequirementOwner>(records.len())
        .map_err(ArtifactError::Allocation)?;

    for owner_record in records {
        let owner = owners.declaration(&owner_record.owner)?;
        let enum_info = aggregates
            .enum_semantics
            .enums
            .get(&owner)
            .ok_or_else(|| invalid("enum requirement owner has no reconstructed enum authority"))?;

        let mut ordered_requirements = allocations
            .vector::<Option<EnumRequirement>>(owner_record.requirements.len())
            .map_err(ArtifactError::Allocation)?;
        ordered_requirements.resize_with(owner_record.requirements.len(), || None);

        for record in &owner_record.requirements {
            let source_order = usize::try_from(record.source_order)
                .map_err(|_| invalid("enum requirement source order exceeds address space"))?;
            let slot = ordered_requirements
                .get_mut(source_order)
                .ok_or_else(|| invalid("enum requirement source order is outside its owner table"))?;
            if slot.is_some() {
                return Err(invalid("enum requirement source order is duplicated"));
            }

            let callable = owners.callable(&record.requirement)?;
            let CallableOwnerId::Declaration(callable_owner) = &callable.owner else {
                return Err(invalid("enum root requirement is not declaration-owned"));
            };
            if callable_owner != &owner || callable.side != DispatchSide::Instance {
                return Err(invalid("enum root requirement callable identity disagrees with its enum owner"));
            }

            let signature = super::callable_import::reconstruct_callable_signature(
                &record.signature,
                types,
                owners,
                allocations,
            )?;
            if signature.callable != callable
                || signature.owner != owner
                || signature.side != DispatchSide::Instance
                || signature.selector != callable.selector
            {
                return Err(invalid("inline enum requirement signature disagrees with its exact owner"));
            }

            let source = record
                .source
                .as_ref()
                .map(|source| -> Result<SemanticSourceSpan, ArtifactError> {
                    let module = owners.module(&source.module)?;
                    if module != owner.module || source.range.start > source.range.end {
                        return Err(invalid("enum requirement source span disagrees with its enum owner"));
                    }
                    Ok(SemanticSourceSpan::new(
                        module,
                        SourceRange {
                            start: usize::try_from(source.range.start)
                                .map_err(|_| invalid("enum requirement source start exceeds address space"))?,
                            end: usize::try_from(source.range.end)
                                .map_err(|_| invalid("enum requirement source end exceeds address space"))?,
                        },
                    ))
                })
                .transpose()?;

            *slot = Some(EnumRequirement {
                id: EnumRequirementId::new(owner.clone(), callable.selector.clone()),
                signature,
                source,
            });
        }

        let mut requirements = allocations
            .vector::<EnumRequirement>(owner_record.requirements.len())
            .map_err(ArtifactError::Allocation)?;
        for requirement in ordered_requirements {
            requirements.push(requirement.ok_or_else(|| invalid("enum requirement source order has a gap"))?);
        }

        let expected_status_count = requirements
            .len()
            .checked_mul(enum_info.variants.len())
            .ok_or_else(|| invalid("enum requirement case status count overflow"))?;
        if owner_record.case_statuses.len() != expected_status_count {
            return Err(invalid("enum requirement case status count disagrees with owner products"));
        }

        let mut statuses = allocations
            .vector::<CaseRequirementResult>(owner_record.case_statuses.len())
            .map_err(ArtifactError::Allocation)?;
        if !enum_info.variants.is_empty() {
            for (index, record) in owner_record.case_statuses.iter().enumerate() {
                let requirement_order = index / enum_info.variants.len();
                let variant_order = index % enum_info.variants.len();
                let requirement = requirements
                    .get(requirement_order)
                    .ok_or_else(|| invalid("enum case status requirement order is missing"))?;
                let variant = owners.variant(&record.variant)?;

                let stable_requirement = owner_record
                    .requirements
                    .get(requirement_order)
                    .ok_or_else(|| invalid("enum case status requirement wire row is missing"))?;
                if record.requirement_order as usize != requirement_order
                    || record.variant_order as usize != variant_order
                    || record.requirement != stable_requirement.requirement
                    || variant != enum_info.variants[variant_order]
                {
                    return Err(invalid("enum case status order or owner identity disagrees with enum semantics"));
                }

                let status = match &record.status {
                    CaseRequirementStatusV1::Satisfied { implementation } => {
                        let implementation = owners.callable(implementation)?;
                        validate_case_implementation(&implementation, &variant, requirement)?;
                        CaseRequirementStatus::Satisfied { implementation }
                    }
                    CaseRequirementStatusV1::Missing => CaseRequirementStatus::Missing,
                    CaseRequirementStatusV1::Incompatible { implementation } => {
                        let implementation = owners.callable(implementation)?;
                        validate_case_implementation(&implementation, &variant, requirement)?;
                        CaseRequirementStatus::Incompatible { implementation }
                    }
                    CaseRequirementStatusV1::Blocked => CaseRequirementStatus::Blocked,
                };
                statuses.push(CaseRequirementResult {
                    variant,
                    requirement: requirement.id.clone(),
                    status,
                });
            }
        }

        imported.push(ImportedEnumRequirementOwner {
            owner,
            requirements: allocations.arc_slice(requirements).map_err(ArtifactError::Allocation)?,
            case_statuses: allocations.arc_slice(statuses).map_err(ArtifactError::Allocation)?,
        });
    }

    imported.sort_by(|left, right| left.owner.cmp(&right.owner));
    if imported.windows(2).any(|pair| pair[0].owner == pair[1].owner) {
        return Err(invalid("duplicate enum requirement owner after stable-owner relocation"));
    }

    Ok(ImportedEnumRequirementAuthority {
        owners: allocations.boxed_slice(imported).map_err(ArtifactError::Allocation)?,
    })
}

fn validate_case_implementation(
    implementation: &crate::identity::CallableId,
    variant: &crate::identity::VariantId,
    requirement: &EnumRequirement,
) -> Result<(), ArtifactError> {
    if !matches!(&implementation.owner, CallableOwnerId::Variant(owner) if owner == variant)
        || implementation.side != DispatchSide::Instance
        || implementation.selector != requirement.id.selector
    {
        return Err(invalid("enum requirement implementation has the wrong case or selector"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
