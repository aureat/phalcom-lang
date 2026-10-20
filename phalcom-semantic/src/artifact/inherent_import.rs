//! Certified PERF003 P7 conditional-inherent dispatch reconstruction adapted to ARTF001.
//!
//! Current-main's live `ConditionalInherentMemberSet` owns an unbudgeted
//! HashMap selector index. P2 therefore reconstructs the exact semantic domains
//! and members into immutable sorted staging rows. T4 materializes the live
//! selector index only at the single generation-publication boundary.

use super::identity::StableOwnerCatalog;
use super::type_import::ReconstructedTypeGraph;
use crate::identity::{CallableOwnerId, ImplId};
use crate::impls::{ConditionalInherentMember, InherentImplDomain, InherentImplTarget};
use crate::types::id::TypeId;
use phalcom_common::modifier::Visibility;
use phalcom_compiler_artifact::semantic::aggregates::VisibilityV1;
use phalcom_compiler_artifact::semantic::world::{ConditionalInherentMemberV1, ConditionalInherentSetV1, InherentImplDomainV1, InherentImplTargetV1};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ImportedConditionalInherentSet {
    pub target: InherentImplTarget,
    pub domains: Box<[(ImplId, Arc<InherentImplDomain>)]>,
    pub members: Box<[ConditionalInherentMember]>,
}

/// Exact conditional-inherent semantic authority staged from P7 rows.
#[derive(Clone, Debug, Default)]
pub struct ImportedConditionalInherentAuthority {
    sets: Box<[ImportedConditionalInherentSet]>,
}

impl ImportedConditionalInherentAuthority {
    pub fn set_len(&self) -> usize {
        self.sets.len()
    }

    pub fn domain_len(&self) -> usize {
        self.sets.iter().map(|set| set.domains.len()).sum()
    }

    pub fn member_len(&self) -> usize {
        self.sets.iter().map(|set| set.members.len()).sum()
    }

    pub(crate) fn set(&self, target: &InherentImplTarget) -> Option<&ImportedConditionalInherentSet> {
        self.sets.binary_search_by(|set| set.target.cmp(target)).ok().map(|index| &self.sets[index])
    }
}

pub(crate) fn reconstruct_conditional_inherent_authority(
    records: &[ConditionalInherentSetV1],
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedConditionalInherentAuthority, ArtifactError> {
    let mut sets = allocations
        .vector::<ImportedConditionalInherentSet>(records.len())
        .map_err(ArtifactError::Allocation)?;

    for record in records {
        let target = reconstruct_target(&record.target, owners)?;

        let mut domains = allocations
            .vector::<(ImplId, Arc<InherentImplDomain>)>(record.domains.len())
            .map_err(ArtifactError::Allocation)?;
        for domain in &record.domains {
            let imported = reconstruct_domain(domain, &target, types, owners, allocations)?;
            domains.push((imported.impl_id.clone(), allocations.arc(imported).map_err(ArtifactError::Allocation)?));
        }
        domains.sort_by(|left, right| left.0.cmp(&right.0));
        if domains.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(invalid("conditional inherent domain is duplicated after stable-owner relocation"));
        }

        let mut ordered_members = allocations
            .vector::<Option<ConditionalInherentMember>>(record.members.len())
            .map_err(ArtifactError::Allocation)?;
        ordered_members.resize_with(record.members.len(), || None);
        for member in &record.members {
            let order = usize::try_from(member.member_order).map_err(|_| invalid("conditional inherent member order exceeds address space"))?;
            let slot = ordered_members
                .get_mut(order)
                .ok_or_else(|| invalid("conditional inherent member order is outside its set"))?;
            if slot.is_some() {
                return Err(invalid("conditional inherent member order is duplicated"));
            }
            *slot = Some(reconstruct_member(member, &target, &domains, types, owners, allocations)?);
        }

        let mut members = allocations
            .vector::<ConditionalInherentMember>(record.members.len())
            .map_err(ArtifactError::Allocation)?;
        for member in ordered_members {
            members.push(member.ok_or_else(|| invalid("conditional inherent member order has a gap"))?);
        }

        sets.push(ImportedConditionalInherentSet {
            target,
            domains: allocations.boxed_slice(domains).map_err(ArtifactError::Allocation)?,
            members: allocations.boxed_slice(members).map_err(ArtifactError::Allocation)?,
        });
    }

    sets.sort_by(|left, right| left.target.cmp(&right.target));
    if sets.windows(2).any(|pair| pair[0].target == pair[1].target) {
        return Err(invalid("conditional inherent target is duplicated after stable-owner relocation"));
    }

    Ok(ImportedConditionalInherentAuthority {
        sets: allocations.boxed_slice(sets).map_err(ArtifactError::Allocation)?,
    })
}

fn reconstruct_domain(
    row: &InherentImplDomainV1,
    expected_target: &InherentImplTarget,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<InherentImplDomain, ArtifactError> {
    let impl_id = owners.implementation(&row.impl_id)?;
    let target = reconstruct_target(&row.target, owners)?;
    if &target != expected_target {
        return Err(invalid("conditional inherent domain target disagrees with its enclosing set"));
    }
    let head_type: TypeId = *types
        .types
        .get(row.head_type.0 as usize)
        .ok_or_else(|| invalid("conditional inherent domain references an absent head type"))?;
    let generic_signature = row
        .generic_signature
        .as_ref()
        .map(|signature| super::semantic_import::reconstruct_signature(signature, types, owners, allocations))
        .transpose()?;
    let mut constraints = allocations.vector(row.constraints.len()).map_err(ArtifactError::Allocation)?;
    for constraint in &row.constraints {
        constraints.push(super::semantic_import::reconstruct_constraint(constraint, types, owners, allocations)?);
    }
    Ok(InherentImplDomain {
        impl_id,
        target,
        head_type,
        generic_signature,
        constraints: allocations.boxed_slice(constraints).map_err(ArtifactError::Allocation)?,
    })
}

fn reconstruct_member(
    row: &ConditionalInherentMemberV1,
    target: &InherentImplTarget,
    domains: &[(ImplId, Arc<InherentImplDomain>)],
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ConditionalInherentMember, ArtifactError> {
    let impl_id = owners.implementation(&row.impl_id)?;
    let domain = domains
        .binary_search_by(|(candidate, _)| candidate.cmp(&impl_id))
        .ok()
        .and_then(|index| domains.get(index))
        .map(|(_, domain)| Arc::clone(domain))
        .ok_or_else(|| invalid("conditional inherent member has no reconstructed domain"))?;

    let callable = owners.callable(&row.callable)?;
    let expected_owner: CallableOwnerId = target.clone().into();
    if callable.owner != expected_owner {
        return Err(invalid("conditional inherent callable owner disagrees with its target"));
    }
    let signature_template = super::callable_import::reconstruct_callable_signature(&row.signature, types, owners, allocations)?;
    if signature_template.callable != callable
        || signature_template.owner != *target.declaration()
        || signature_template.side != callable.side
        || signature_template.selector != callable.selector
    {
        return Err(invalid("conditional inherent signature disagrees with its callable identity"));
    }

    Ok(ConditionalInherentMember {
        impl_id,
        domain,
        is_conditionally_applicable: row.is_conditionally_applicable,
        callable,
        signature_template,
        visibility: visibility(&row.visibility),
        source_member: usize::try_from(row.source_member_index).map_err(|_| invalid("conditional inherent source member index exceeds address space"))?,
        is_requirement: row.is_requirement,
    })
}

fn reconstruct_target(target: &InherentImplTargetV1, owners: &StableOwnerCatalog<'_>) -> Result<InherentImplTarget, ArtifactError> {
    Ok(match target {
        InherentImplTargetV1::Declaration { declaration } => InherentImplTarget::Declaration(owners.declaration(declaration)?),
        InherentImplTargetV1::ExactEnumCase { variant } => InherentImplTarget::ExactEnumCase(owners.variant(variant)?),
    })
}

fn visibility(value: &VisibilityV1) -> Visibility {
    match value {
        VisibilityV1::Public => Visibility::Public,
        VisibilityV1::Private => Visibility::Private,
        VisibilityV1::Protected => Visibility::Protected,
        VisibilityV1::Module => Visibility::Module,
        VisibilityV1::Package => Visibility::Package,
        VisibilityV1::Internal => Visibility::Internal,
    }
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
