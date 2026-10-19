//! P7 associated-authority reconstruction adapted to current-main semantic types.
//!
//! Certified PERF003 P7 reconstructed type-member declarations, associated
//! static domains, and associated surfaces as one semantic authority before
//! query-image publication. ARTF001 preserves that ownership model while using
//! project-qualified owner relocation and the current TypeStore representation.

use super::identity::StableOwnerCatalog;
use super::type_import::ReconstructedTypeGraph;
use crate::associated::{AssociatedFamilyInfo, AssociatedFamilyKind, AssociatedMemberId, AssociatedSurface};
use crate::db::product::{AssociatedStaticDomainDeclarationProduct, TypeMemberDeclarationProduct};
use crate::identity::{
    AssociatedFamilyId, AssociatedStaticDomainDeclarationId, DataComponentId, DataConstructorId, DeclarationId,
};
use crate::types::type_member::{
    IntrinsicTypeOperation, TypeMemberDeclarationId, TypeMemberDomain, TypeMemberParameter, TypeMemberSelector, TypeMemberSignature,
};
use phalcom_common::modifier::ImplementationKind;
use phalcom_common::range::SourceRange;
use phalcom_common::selector::SelectorBase;
use phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1;
use phalcom_compiler_artifact::semantic::world::{
    AssociatedFamilyKindV1, AssociatedMemberV1, AssociatedStaticDomainRecordV1, AssociatedSurfaceRecordV1, IntrinsicTypeOperationV1, SemanticWorldV3,
    TypeMemberDeclarationRecordV1, TypeMemberDomainV1, TypeMemberImplementationV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Current-main associated semantic products reconstructed from one validated
/// semantic-world image.
///
/// The sorted vectors are staging indexes, not a second semantic database.
/// They avoid constructing current-main workspace maps before T4's atomic
/// snapshot/session publication boundary while still giving query import exact
/// product identity and O(log N) lookup.
#[derive(Clone, Debug, Default)]
pub struct ImportedAssociatedAuthority {
    type_members: Box<[(TypeMemberDeclarationId, Arc<TypeMemberDeclarationProduct>)]>,
    static_domains: Box<[(AssociatedStaticDomainDeclarationId, Arc<AssociatedStaticDomainDeclarationProduct>)]>,
    surfaces: Box<[(DeclarationId, Arc<AssociatedSurface>)]>,
}

impl ImportedAssociatedAuthority {
    pub fn type_member_len(&self) -> usize {
        self.type_members.len()
    }

    pub fn static_domain_len(&self) -> usize {
        self.static_domains.len()
    }

    pub fn surface_len(&self) -> usize {
        self.surfaces.len()
    }

    pub(crate) fn type_member(&self, id: &TypeMemberDeclarationId) -> Option<&Arc<TypeMemberDeclarationProduct>> {
        self.type_members
            .binary_search_by(|(candidate, _)| candidate.cmp(id))
            .ok()
            .map(|index| &self.type_members[index].1)
    }

    pub(crate) fn static_domain(
        &self,
        id: &AssociatedStaticDomainDeclarationId,
    ) -> Option<&Arc<AssociatedStaticDomainDeclarationProduct>> {
        self.static_domains
            .binary_search_by(|(candidate, _)| candidate.cmp(id))
            .ok()
            .map(|index| &self.static_domains[index].1)
    }

    pub(crate) fn surface(&self, owner: &DeclarationId) -> Option<&Arc<AssociatedSurface>> {
        self.surfaces
            .binary_search_by(|(candidate, _)| candidate.cmp(owner))
            .ok()
            .map(|index| &self.surfaces[index].1)
    }
}

/// Transplants P7's `import_associated_authority` ownership boundary.
///
/// Unlike P7, stable identities are resolved through `StableOwnerCatalog` so
/// Package and SourceArtifact projects may relocate to arbitrary receiving
/// graph-local IDs without changing artifact identity.
pub(crate) fn reconstruct_associated_authority(
    world: &SemanticWorldV3,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedAssociatedAuthority, ArtifactError> {
    let mut type_members = allocations
        .vector::<(TypeMemberDeclarationId, Arc<TypeMemberDeclarationProduct>)>(world.type_member_declarations.len())
        .map_err(ArtifactError::Allocation)?;
    for row in &world.type_member_declarations {
        let product = reconstruct_type_member_product(row, owners, types, allocations)?;
        type_members.push((product.id.clone(), product));
    }
    type_members.sort_by(|left, right| left.0.cmp(&right.0));
    reject_duplicate_keys(&type_members, "duplicate type-member declaration after stable-owner relocation")?;

    let mut static_domains = allocations
        .vector::<(
            AssociatedStaticDomainDeclarationId,
            Arc<AssociatedStaticDomainDeclarationProduct>,
        )>(world.associated_static_domains.len())
        .map_err(ArtifactError::Allocation)?;
    for row in &world.associated_static_domains {
        let product = reconstruct_static_domain_product(row, owners, types, allocations)?;
        static_domains.push((product.id.clone(), product));
    }
    static_domains.sort_by(|left, right| left.0.cmp(&right.0));
    reject_duplicate_keys(
        &static_domains,
        "duplicate associated static-domain declaration after stable-owner relocation",
    )?;

    let mut surfaces = allocations
        .vector::<(DeclarationId, Arc<AssociatedSurface>)>(world.associated_surfaces.len())
        .map_err(ArtifactError::Allocation)?;
    for row in &world.associated_surfaces {
        let surface = reconstruct_associated_surface(row, owners, allocations)?;
        surfaces.push((surface.owner.clone(), surface));
    }
    surfaces.sort_by(|left, right| left.0.cmp(&right.0));
    reject_duplicate_keys(&surfaces, "duplicate associated surface after stable-owner relocation")?;

    Ok(ImportedAssociatedAuthority {
        type_members: allocations.boxed_slice(type_members).map_err(ArtifactError::Allocation)?,
        static_domains: allocations.boxed_slice(static_domains).map_err(ArtifactError::Allocation)?,
        surfaces: allocations.boxed_slice(surfaces).map_err(ArtifactError::Allocation)?,
    })
}

fn reject_duplicate_keys<K: Ord, V>(rows: &[(K, V)], message: &'static str) -> Result<(), ArtifactError> {
    if rows.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        Err(invalid(message))
    } else {
        Ok(())
    }
}

fn reconstruct_type_member_product(
    row: &TypeMemberDeclarationRecordV1,
    owners: &StableOwnerCatalog<'_>,
    types: &ReconstructedTypeGraph,
    allocations: &mut AllocationBudget,
) -> Result<Arc<TypeMemberDeclarationProduct>, ArtifactError> {
    let owner = owners.declaration(&row.owner)?;
    let selector = type_member_selector(&row.selector, allocations)?;
    let id = TypeMemberDeclarationId::new(owner, selector.clone());

    let mut parameters = allocations
        .vector::<TypeMemberParameter>(row.parameters.len())
        .map_err(ArtifactError::Allocation)?;
    for parameter in &row.parameters {
        parameters.push(TypeMemberParameter {
            name: allocations.boxed_str(&parameter.name).map_err(ArtifactError::Allocation)?,
            domain: type_member_domain(&parameter.domain, types)?,
        });
    }

    let intrinsic = row.intrinsic.as_ref().map(import_intrinsic_type_operation);
    let implementation = import_type_member_implementation(&row.implementation);
    allocations
        .arc(TypeMemberDeclarationProduct {
            id,
            signature: TypeMemberSignature {
                selector,
                parameters: allocations.boxed_slice(parameters).map_err(ArtifactError::Allocation)?,
                result_domain: type_member_domain(&row.result_domain, types)?,
                intrinsic,
                implementation,
            },
            // P7 serializes checked semantics, not presentation/source slices.
            // Source-backed overlays therefore derive a different input
            // fingerprint and invalidate this source-free baseline product.
            source_range: SourceRange::default(),
        })
        .map_err(ArtifactError::Allocation)
}

fn reconstruct_static_domain_product(
    row: &AssociatedStaticDomainRecordV1,
    owners: &StableOwnerCatalog<'_>,
    types: &ReconstructedTypeGraph,
    allocations: &mut AllocationBudget,
) -> Result<Arc<AssociatedStaticDomainDeclarationProduct>, ArtifactError> {
    let owner = owners.declaration(&row.declaration.owner)?;
    let id = AssociatedStaticDomainDeclarationId::new(
        owner,
        allocations.boxed_str(&row.declaration.name).map_err(ArtifactError::Allocation)?,
    );
    let domain = *types
        .static_domains
        .get(row.domain.0 as usize)
        .ok_or_else(|| invalid("associated static-domain row references an absent type-graph domain"))?;
    let mut constructors = allocations
        .vector::<crate::types::id::AlgebraicStaticConstructorId>(row.constructors.len())
        .map_err(ArtifactError::Allocation)?;
    for constructor in &row.constructors {
        constructors.push(
            *types
                .static_constructors
                .get(constructor.0 as usize)
                .ok_or_else(|| invalid("associated static-domain row references an absent constructor"))?,
        );
    }
    allocations
        .arc(AssociatedStaticDomainDeclarationProduct {
            id,
            domain,
            constructors: allocations.boxed_slice(constructors).map_err(ArtifactError::Allocation)?,
            source_range: SourceRange::default(),
        })
        .map_err(ArtifactError::Allocation)
}

fn reconstruct_associated_surface(
    row: &AssociatedSurfaceRecordV1,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<Arc<AssociatedSurface>, ArtifactError> {
    let owner = owners.declaration(&row.owner)?;
    let mut families = BTreeMap::new();

    for family in &row.families {
        let base = selector_base(&family.selector, allocations)?;
        let mut members = allocations
            .vector::<AssociatedMemberId>(family.members.len())
            .map_err(ArtifactError::Allocation)?;
        for member in &family.members {
            let imported = associated_member(member, &owner, owners, allocations)?;
            members.push(imported);
        }
        let info = AssociatedFamilyInfo {
            id: AssociatedFamilyId::new(owner.clone(), base.clone()),
            kind: associated_family_kind(&family.kind),
            members: allocations.boxed_slice(members).map_err(ArtifactError::Allocation)?,
        };
        if families.insert(base, info).is_some() {
            return Err(invalid("associated surface contains duplicate family selectors"));
        }
    }

    allocations
        .arc(AssociatedSurface { owner, families })
        .map_err(ArtifactError::Allocation)
}

fn associated_member(
    member: &AssociatedMemberV1,
    expected_owner: &DeclarationId,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<AssociatedMemberId, ArtifactError> {
    let imported = match member {
        AssociatedMemberV1::Variant { variant } => AssociatedMemberId::Variant(owners.variant(variant)?),
        AssociatedMemberV1::DataConstructor { owner } => {
            AssociatedMemberId::DataConstructor(DataConstructorId::new(owners.declaration(owner)?))
        }
        AssociatedMemberV1::DataComponent { owner, index } => {
            AssociatedMemberId::DataComponent(DataComponentId::new(owners.declaration(owner)?, *index))
        }
        AssociatedMemberV1::StaticDomain { declaration } => {
            AssociatedMemberId::StaticDomain(AssociatedStaticDomainDeclarationId::new(
                owners.declaration(&declaration.owner)?,
                allocations.boxed_str(&declaration.name).map_err(ArtifactError::Allocation)?,
            ))
        }
        AssociatedMemberV1::TypeMember { owner, selector } => AssociatedMemberId::TypeMember(
            TypeMemberDeclarationId::new(owners.declaration(owner)?, type_member_selector(selector, allocations)?),
        ),
    };

    let actual_owner = match &imported {
        AssociatedMemberId::Variant(variant) => &variant.owner,
        AssociatedMemberId::DataConstructor(constructor) => &constructor.owner,
        AssociatedMemberId::DataComponent(component) => &component.owner,
        AssociatedMemberId::StaticDomain(declaration) => &declaration.owner,
        AssociatedMemberId::TypeMember(member) => &member.owner,
    };
    if actual_owner != expected_owner {
        return Err(invalid("associated surface member belongs to another declaration"));
    }
    Ok(imported)
}

fn type_member_domain(domain: &TypeMemberDomainV1, types: &ReconstructedTypeGraph) -> Result<TypeMemberDomain, ArtifactError> {
    Ok(match domain {
        TypeMemberDomainV1::Kind { kind } => TypeMemberDomain::Kind(
            *types
                .kinds
                .get(kind.0 as usize)
                .ok_or_else(|| invalid("type-member domain references an absent kind row"))?,
        ),
        TypeMemberDomainV1::ValueType { ty } => TypeMemberDomain::ValueType(
            *types
                .types
                .get(ty.0 as usize)
                .ok_or_else(|| invalid("type-member domain references an absent type row"))?,
        ),
    })
}

fn type_member_selector(selector: &TypeMemberSelectorV1, allocations: &mut AllocationBudget) -> Result<TypeMemberSelector, ArtifactError> {
    Ok(match selector {
        TypeMemberSelectorV1::Named { name } => {
            if name.as_str() == "At" {
                return Err(invalid("noncanonical type-member selector aliases the subscript identity"));
            }
            TypeMemberSelector::Named(allocations.boxed_str(name).map_err(ArtifactError::Allocation)?)
        }
        TypeMemberSelectorV1::Subscript => TypeMemberSelector::Subscript,
    })
}

fn selector_base(selector: &TypeMemberSelectorV1, allocations: &mut AllocationBudget) -> Result<SelectorBase, ArtifactError> {
    Ok(match selector {
        TypeMemberSelectorV1::Named { name } => SelectorBase::Named(allocations.string(name).map_err(ArtifactError::Allocation)?),
        TypeMemberSelectorV1::Subscript => SelectorBase::Subscript,
    })
}

fn import_intrinsic_type_operation(operation: &IntrinsicTypeOperationV1) -> IntrinsicTypeOperation {
    match operation {
        IntrinsicTypeOperationV1::TuplePositionals => IntrinsicTypeOperation::TuplePositionals,
        IntrinsicTypeOperationV1::TupleLabeled => IntrinsicTypeOperation::TupleLabeled,
        IntrinsicTypeOperationV1::TupleEntries => IntrinsicTypeOperation::TupleEntries,
        IntrinsicTypeOperationV1::TupleElement => IntrinsicTypeOperation::TupleElement,
        IntrinsicTypeOperationV1::TupleAt => IntrinsicTypeOperation::TupleAt,
        IntrinsicTypeOperationV1::TupleMap => IntrinsicTypeOperation::TupleMap,
        IntrinsicTypeOperationV1::RecordFields => IntrinsicTypeOperation::RecordFields,
        IntrinsicTypeOperationV1::RecordValue => IntrinsicTypeOperation::RecordValue,
        IntrinsicTypeOperationV1::RecordMap => IntrinsicTypeOperation::RecordMap,
        IntrinsicTypeOperationV1::TypePackMap => IntrinsicTypeOperation::TypePackMap,
        IntrinsicTypeOperationV1::LabeledTypePackMap => IntrinsicTypeOperation::LabeledTypePackMap,
        IntrinsicTypeOperationV1::RecordRowMap => IntrinsicTypeOperation::RecordRowMap,
        IntrinsicTypeOperationV1::CompleteTypePackPositionals => IntrinsicTypeOperation::CompleteTypePackPositionals,
        IntrinsicTypeOperationV1::CompleteTypePackLabeled => IntrinsicTypeOperation::CompleteTypePackLabeled,
        IntrinsicTypeOperationV1::CompleteTypePackMap => IntrinsicTypeOperation::CompleteTypePackMap,
        IntrinsicTypeOperationV1::UnorderedShapeLookup => IntrinsicTypeOperation::UnorderedShapeLookup,
        IntrinsicTypeOperationV1::UnorderedShapeView => IntrinsicTypeOperation::UnorderedShapeView,
        IntrinsicTypeOperationV1::UnorderedShapeTail => IntrinsicTypeOperation::UnorderedShapeTail,
    }
}

fn import_type_member_implementation(implementation: &TypeMemberImplementationV1) -> ImplementationKind {
    match implementation {
        TypeMemberImplementationV1::Phalcom => ImplementationKind::Phalcom,
        TypeMemberImplementationV1::Native => ImplementationKind::Native,
        TypeMemberImplementationV1::Foreign => ImplementationKind::Foreign,
        TypeMemberImplementationV1::Intrinsic => ImplementationKind::Intrinsic,
    }
}

fn associated_family_kind(kind: &AssociatedFamilyKindV1) -> AssociatedFamilyKind {
    match kind {
        AssociatedFamilyKindV1::Variant => AssociatedFamilyKind::Variant,
        AssociatedFamilyKindV1::DataConstructor => AssociatedFamilyKind::DataConstructor,
        AssociatedFamilyKindV1::StaticDomain => AssociatedFamilyKind::StaticDomain,
        AssociatedFamilyKindV1::TypeMember => AssociatedFamilyKind::TypeMember,
        AssociatedFamilyKindV1::IntrinsicTypeMember => AssociatedFamilyKind::IntrinsicTypeMember,
    }
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
