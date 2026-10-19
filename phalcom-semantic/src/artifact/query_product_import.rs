//! Reconstruction of reusable typed query products from validated artifact rows.
//!
//! This module deliberately starts with the source-independent structural
//! declaration families whose live query implementations have no dependency
//! edges: `DataDeclaration` and `EnumDeclaration`. Other query families remain
//! wire DTOs until their complete current-main dependency contract is imported.

use crate::artifact::identity::StableOwnerCatalog;
use crate::associated::{AssociatedFamilyInfo, AssociatedFamilyKind, AssociatedMemberId, AssociatedSurface};
use crate::db::fingerprint::{
    associated_surface_input_fingerprint, associated_surface_product_fingerprint, data_declaration_input_fingerprint,
    data_declaration_product_fingerprint, enum_declaration_input_fingerprint, enum_declaration_product_fingerprint,
};
use crate::db::product::{DataDeclarationProduct, EnumDeclarationProduct};
use crate::db::{InputFingerprint, ProductFingerprint, QueryKey, SemanticDb, SemanticProduct};
use crate::diagnostic::SemanticDiagnostic;
use crate::enum_semantics::VariantInfo;
use crate::identity::{
    AssociatedFamilyId, AssociatedStaticDomainDeclarationId, DataComponentId, DataConstructorId, DeclarationId,
};
use phalcom_compiler_artifact::query::{
    DependencyEvidenceEntryV3, PortableQueryKindV1 as Kind, PortableQueryOwnerKeyV1 as Owner, QueryImageV3, QueryRootEntryV3,
    portable_structural_dependency_product_fingerprint, portable_structural_root_fingerprints,
};
use phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1;
use phalcom_compiler_artifact::semantic::world::{AssociatedFamilyKindV1, AssociatedMemberV1, AssociatedSurfaceRecordV1, SemanticWorldV3};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;
use std::collections::BTreeMap;
use std::sync::Arc;

/// One typed, fingerprint-validated query product staged from an artifact.
///
/// Dependencies are intentionally absent from this initial structural slice:
/// the current-main DataDeclaration and EnumDeclaration queries publish no
/// dependency edges. A future family with dependencies must carry and validate
/// those edges explicitly rather than extending this representation implicitly.
#[derive(Clone, Debug)]
pub(crate) struct ImportedReadyQueryProduct {
    pub key: QueryKey,
    pub input_fingerprint: InputFingerprint,
    pub product_fingerprint: ProductFingerprint,
    pub product: SemanticProduct,
}

/// Typed artifact query products that have passed current-main reconstruction
/// and canonical fingerprint validation.
#[derive(Clone, Debug, Default)]
pub struct ImportedQueryProducts {
    ready: Vec<ImportedReadyQueryProduct>,
    /// Fingerprint-validated dependency products. These remain non-authoritative
    /// until a reusable root proves a closed dependency set that references them.
    evidence: Vec<ImportedReadyQueryProduct>,
}

impl ImportedQueryProducts {
    pub fn len(&self) -> usize {
        self.ready.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ready.is_empty()
    }

    /// Number of validated dependency-only products retained for future closed
    /// dependency-set promotion. This count does not imply reusable authority.
    pub fn validated_evidence_len(&self) -> usize {
        self.evidence.len()
    }

    /// Publishes the staged products into the one current-main semantic database.
    ///
    /// Imported generation publication is deliberately restricted to a fresh
    /// database. T4 owns construction of that database and its surrounding
    /// workspace session; this helper must not overlay artifact authority onto
    /// an unrelated live generation.
    pub(crate) fn publish_into_empty_db(&self, db: &mut SemanticDb) -> Result<(), ArtifactError> {
        if db.query_keys().next().is_some() {
            return Err(invalid("artifact query products require an empty semantic database"));
        }

        db.begin_transaction();
        let revision = db.revision();
        for ready in &self.ready {
            if db
                .publish_product_ready(
                    ready.key.clone(),
                    revision,
                    ready.input_fingerprint,
                    ready.product_fingerprint,
                    ready.product.clone(),
                    std::iter::empty(),
                )
                .is_err()
            {
                db.rollback_transaction();
                return Err(invalid("artifact query product publication became stale"));
            }
        }
        db.commit_transaction();
        Ok(())
    }
}

/// Reconstructs the first reusable structural query-product families.
///
/// Roots are independently bound to exact semantic rows before this function is
/// called. Here their portable wire fingerprints are validated first, then they
/// are relocated to live keys and rebuilt as current-main typed products. The
/// receiving SemanticDb gets freshly computed local input/product fingerprints.
///
/// Structural dependency evidence is also reconstructed and fingerprint-checked,
/// but evidence alone is never promoted to `Ready`. Later query families must
/// explicitly prove a closed dependency set reachable from an imported root
/// before those dependency products become database authority.
pub(crate) fn reconstruct_structural_query_products(
    image: &QueryImageV3,
    aggregates: &super::semantic_import::ImportedSemanticAggregates,
    owners: &StableOwnerCatalog<'_>,
    world: &SemanticWorldV3,
    allocations: &mut AllocationBudget,
) -> Result<ImportedQueryProducts, ArtifactError> {
    let root_count = image
        .supported_roots
        .iter()
        .filter(|root| is_structural_kind(root.key.kind))
        .count();
    let evidence_count = image.dependency_evidence.iter().filter(|evidence| is_structural_kind(evidence.key.kind)).count();

    if root_count == 0 && evidence_count == 0 {
        return Ok(ImportedQueryProducts::default());
    }

    let mut ready = allocations
        .vector::<ImportedReadyQueryProduct>(root_count)
        .map_err(ArtifactError::Allocation)?;
    let mut evidence_products = allocations
        .vector::<ImportedReadyQueryProduct>(evidence_count)
        .map_err(ArtifactError::Allocation)?;
    let empty_diagnostics = allocations
        .arc_slice(Vec::<SemanticDiagnostic>::new())
        .map_err(ArtifactError::Allocation)?;

    for root in &image.supported_roots {
        let imported = match root.key.kind {
            Kind::DataDeclaration => reconstruct_data_root(root, aggregates, owners, world, empty_diagnostics.clone(), allocations)?,
            Kind::EnumDeclaration => reconstruct_enum_root(root, aggregates, owners, world, empty_diagnostics.clone(), allocations)?,
            Kind::AssociatedSurface => reconstruct_associated_surface_root(root, owners, world, allocations)?,
            _ => continue,
        };
        ready.push(imported);
    }

    // Dependency evidence is validated but is not a reusable root by itself.
    // Future query-family import promotes only evidence reachable from a root
    // whose complete dependency contract has also been reconstructed.
    for evidence in &image.dependency_evidence {
        let imported = match evidence.key.kind {
            Kind::DataDeclaration => Some(reconstruct_data_evidence(
                evidence,
                aggregates,
                owners,
                world,
                empty_diagnostics.clone(),
                allocations,
            )?),
            Kind::EnumDeclaration => Some(reconstruct_enum_evidence(
                evidence,
                aggregates,
                owners,
                world,
                empty_diagnostics.clone(),
                allocations,
            )?),
            Kind::AssociatedSurface => Some(reconstruct_associated_surface_evidence(evidence, owners, world, allocations)?),
            _ => None,
        };
        if let Some(imported) = imported {
            evidence_products.push(imported);
        }
    }

    // Wire keys are unique already; these checks catch any stable-owner
    // relocation aliasing before either roots or dependency evidence can become
    // database authority.
    ready.sort_by(|left, right| left.key.cmp(&right.key));
    if ready.windows(2).any(|pair| pair[0].key == pair[1].key) {
        return Err(invalid("multiple artifact query roots relocate to the same live query key"));
    }
    evidence_products.sort_by(|left, right| left.key.cmp(&right.key));
    if evidence_products.windows(2).any(|pair| pair[0].key == pair[1].key) {
        return Err(invalid("multiple artifact dependency entries relocate to the same live query key"));
    }

    Ok(ImportedQueryProducts {
        ready,
        evidence: evidence_products,
    })
}

fn reconstruct_data_root(
    root: &QueryRootEntryV3,
    aggregates: &super::semantic_import::ImportedSemanticAggregates,
    owners: &StableOwnerCatalog<'_>,
    world: &SemanticWorldV3,
    diagnostics: Arc<[SemanticDiagnostic]>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_zero_dependency_contract(root)?;
    validate_portable_root_fingerprints(root, world)?;
    let key = QueryKey::from_portable_owner(root.key.kind, &root.key.owner, owners)?;
    let QueryKey::DataDeclaration(owner) = &key else {
        return Err(invalid("data query root did not relocate to DataDeclaration"));
    };
    let product = data_product(owner, aggregates, diagnostics, allocations)?;
    let input_fingerprint = data_declaration_input_fingerprint(&product);
    let product_fingerprint = data_declaration_product_fingerprint(&product);
    Ok(ready(key, input_fingerprint, product_fingerprint, SemanticProduct::DataDeclaration(product)))
}

fn reconstruct_enum_root(
    root: &QueryRootEntryV3,
    aggregates: &super::semantic_import::ImportedSemanticAggregates,
    owners: &StableOwnerCatalog<'_>,
    world: &SemanticWorldV3,
    diagnostics: Arc<[SemanticDiagnostic]>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_zero_dependency_contract(root)?;
    validate_portable_root_fingerprints(root, world)?;
    let key = QueryKey::from_portable_owner(root.key.kind, &root.key.owner, owners)?;
    let QueryKey::EnumDeclaration(owner) = &key else {
        return Err(invalid("enum query root did not relocate to EnumDeclaration"));
    };
    let product = enum_product(owner, aggregates, diagnostics, allocations)?;
    let input_fingerprint = enum_declaration_input_fingerprint(&product);
    let product_fingerprint = enum_declaration_product_fingerprint(&product);
    Ok(ready(key, input_fingerprint, product_fingerprint, SemanticProduct::EnumDeclaration(product)))
}

fn reconstruct_data_evidence(
    evidence: &DependencyEvidenceEntryV3,
    aggregates: &super::semantic_import::ImportedSemanticAggregates,
    owners: &StableOwnerCatalog<'_>,
    world: &SemanticWorldV3,
    diagnostics: Arc<[SemanticDiagnostic]>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_exact_structural_aggregate_owner(evidence)?;
    validate_portable_evidence_fingerprint(evidence, world)?;
    let key = QueryKey::from_portable_owner(evidence.key.kind, &evidence.key.owner, owners)?;
    let QueryKey::DataDeclaration(owner) = &key else {
        return Err(invalid("data dependency evidence did not relocate to DataDeclaration"));
    };
    let product = data_product(owner, aggregates, diagnostics, allocations)?;
    let input_fingerprint = data_declaration_input_fingerprint(&product);
    let product_fingerprint = data_declaration_product_fingerprint(&product);
    Ok(ready(key, input_fingerprint, product_fingerprint, SemanticProduct::DataDeclaration(product)))
}

fn reconstruct_enum_evidence(
    evidence: &DependencyEvidenceEntryV3,
    aggregates: &super::semantic_import::ImportedSemanticAggregates,
    owners: &StableOwnerCatalog<'_>,
    world: &SemanticWorldV3,
    diagnostics: Arc<[SemanticDiagnostic]>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_exact_structural_aggregate_owner(evidence)?;
    validate_portable_evidence_fingerprint(evidence, world)?;
    let key = QueryKey::from_portable_owner(evidence.key.kind, &evidence.key.owner, owners)?;
    let QueryKey::EnumDeclaration(owner) = &key else {
        return Err(invalid("enum dependency evidence did not relocate to EnumDeclaration"));
    };
    let product = enum_product(owner, aggregates, diagnostics, allocations)?;
    let input_fingerprint = enum_declaration_input_fingerprint(&product);
    let product_fingerprint = enum_declaration_product_fingerprint(&product);
    Ok(ready(key, input_fingerprint, product_fingerprint, SemanticProduct::EnumDeclaration(product)))
}

fn reconstruct_associated_surface_root(
    root: &QueryRootEntryV3,
    owners: &StableOwnerCatalog<'_>,
    world: &SemanticWorldV3,
    allocations: &mut AllocationBudget,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_zero_dependency_contract(root)?;
    validate_portable_root_fingerprints(root, world)?;
    let key = QueryKey::from_portable_owner(root.key.kind, &root.key.owner, owners)?;
    let QueryKey::AssociatedSurface(owner) = &key else {
        return Err(invalid("associated-surface query root did not relocate to AssociatedSurface"));
    };
    let row = world
        .associated_surfaces
        .get(root.product.row_index as usize)
        .ok_or_else(|| invalid("associated-surface query product row is out of bounds"))?;
    let surface = reconstruct_associated_surface(row, owner, owners, allocations)?;
    let input_fingerprint = associated_surface_input_fingerprint(&surface);
    let product_fingerprint = associated_surface_product_fingerprint(&surface);
    Ok(ready(
        key,
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::AssociatedSurface(surface),
    ))
}

fn reconstruct_associated_surface_evidence(
    evidence: &DependencyEvidenceEntryV3,
    owners: &StableOwnerCatalog<'_>,
    world: &SemanticWorldV3,
    allocations: &mut AllocationBudget,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_exact_structural_aggregate_owner(evidence)?;
    validate_portable_evidence_fingerprint(evidence, world)?;
    let key = QueryKey::from_portable_owner(evidence.key.kind, &evidence.key.owner, owners)?;
    let QueryKey::AssociatedSurface(owner) = &key else {
        return Err(invalid("associated-surface dependency evidence did not relocate to AssociatedSurface"));
    };
    let Owner::Declaration(stable_owner) = &evidence.key.owner else {
        return Err(invalid("associated-surface dependency evidence is not declaration-owned"));
    };
    let mut rows = world.associated_surfaces.iter().filter(|row| &row.owner == stable_owner);
    let row = rows
        .next()
        .ok_or_else(|| invalid("associated-surface dependency evidence has no semantic product row"))?;
    if rows.next().is_some() {
        return Err(invalid("associated-surface dependency evidence owner is ambiguous"));
    }
    let surface = reconstruct_associated_surface(row, owner, owners, allocations)?;
    let input_fingerprint = associated_surface_input_fingerprint(&surface);
    let product_fingerprint = associated_surface_product_fingerprint(&surface);
    Ok(ready(
        key,
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::AssociatedSurface(surface),
    ))
}

fn reconstruct_associated_surface(
    row: &AssociatedSurfaceRecordV1,
    expected_owner: &DeclarationId,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<Arc<AssociatedSurface>, ArtifactError> {
    let owner = owners.declaration(&row.owner)?;
    if &owner != expected_owner {
        return Err(invalid("associated-surface semantic row owner differs from relocated query owner"));
    }

    let mut families = BTreeMap::new();
    for family in &row.families {
        let base = selector_base(&family.selector, allocations)?;
        let kind = associated_family_kind(family.kind.clone());
        let mut members = allocations
            .vector::<AssociatedMemberId>(family.members.len())
            .map_err(ArtifactError::Allocation)?;
        for member in &family.members {
            let (member_kind, member) = associated_member(member, &owner, owners, allocations)?;
            if member_kind != kind {
                return Err(invalid("associated-surface member category differs from its family category"));
            }
            members.push(member);
        }
        let id = AssociatedFamilyId::new(owner.clone(), base.clone());
        let info = AssociatedFamilyInfo {
            id,
            kind,
            members: allocations.boxed_slice(members).map_err(ArtifactError::Allocation)?,
        };
        if families.insert(base, info).is_some() {
            return Err(invalid("associated-surface contains duplicate family selectors"));
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
) -> Result<(AssociatedFamilyKind, AssociatedMemberId), ArtifactError> {
    let pair = match member {
        AssociatedMemberV1::Variant { variant } => {
            let variant = owners.variant(variant)?;
            if &variant.owner != expected_owner {
                return Err(invalid("associated variant belongs to another declaration"));
            }
            (AssociatedFamilyKind::Variant, AssociatedMemberId::Variant(variant))
        }
        AssociatedMemberV1::DataConstructor { owner } => {
            let owner = owners.declaration(owner)?;
            if &owner != expected_owner {
                return Err(invalid("associated data constructor belongs to another declaration"));
            }
            (
                AssociatedFamilyKind::DataConstructor,
                AssociatedMemberId::DataConstructor(DataConstructorId::new(owner)),
            )
        }
        AssociatedMemberV1::DataComponent { .. } => {
            return Err(invalid("data components are not declaration associated-surface family members"));
        }
        AssociatedMemberV1::StaticDomain { declaration } => {
            let owner = owners.declaration(&declaration.owner)?;
            if &owner != expected_owner {
                return Err(invalid("associated static domain belongs to another declaration"));
            }
            let name = allocations.boxed_str(&declaration.name).map_err(ArtifactError::Allocation)?;
            (
                AssociatedFamilyKind::StaticDomain,
                AssociatedMemberId::StaticDomain(AssociatedStaticDomainDeclarationId::new(owner, name)),
            )
        }
        AssociatedMemberV1::TypeMember { owner, selector } => {
            let owner = owners.declaration(owner)?;
            if &owner != expected_owner {
                return Err(invalid("associated type member belongs to another declaration"));
            }
            let selector = type_member_selector(selector, allocations)?;
            (
                AssociatedFamilyKind::TypeMember,
                AssociatedMemberId::TypeMember(crate::types::type_member::TypeMemberDeclarationId::new(owner, selector)),
            )
        }
    };
    Ok(pair)
}

fn selector_base(selector: &TypeMemberSelectorV1, allocations: &mut AllocationBudget) -> Result<phalcom_common::selector::SelectorBase, ArtifactError> {
    Ok(match selector {
        TypeMemberSelectorV1::Named { name } => phalcom_common::selector::SelectorBase::Named(
            allocations.string(name).map_err(ArtifactError::Allocation)?,
        ),
        TypeMemberSelectorV1::Subscript => phalcom_common::selector::SelectorBase::Subscript,
    })
}

fn type_member_selector(
    selector: &TypeMemberSelectorV1,
    allocations: &mut AllocationBudget,
) -> Result<crate::types::type_member::TypeMemberSelector, ArtifactError> {
    Ok(match selector {
        TypeMemberSelectorV1::Named { name } => crate::types::type_member::TypeMemberSelector::Named(
            allocations.boxed_str(name).map_err(ArtifactError::Allocation)?,
        ),
        TypeMemberSelectorV1::Subscript => crate::types::type_member::TypeMemberSelector::Subscript,
    })
}

fn associated_family_kind(kind: AssociatedFamilyKindV1) -> AssociatedFamilyKind {
    match kind {
        AssociatedFamilyKindV1::Variant => AssociatedFamilyKind::Variant,
        AssociatedFamilyKindV1::DataConstructor => AssociatedFamilyKind::DataConstructor,
        AssociatedFamilyKindV1::StaticDomain => AssociatedFamilyKind::StaticDomain,
        AssociatedFamilyKindV1::TypeMember => AssociatedFamilyKind::TypeMember,
        AssociatedFamilyKindV1::IntrinsicTypeMember => AssociatedFamilyKind::IntrinsicTypeMember,
    }
}

fn data_product(
    owner: &DeclarationId,
    aggregates: &super::semantic_import::ImportedSemanticAggregates,
    diagnostics: Arc<[SemanticDiagnostic]>,
    allocations: &mut AllocationBudget,
) -> Result<Arc<DataDeclarationProduct>, ArtifactError> {
    let info = aggregates
        .data_semantics
        .data_info(owner)
        .cloned()
        .ok_or_else(|| invalid("data query identity has no reconstructed declaration product"))?;
    allocations
        .arc(DataDeclarationProduct { info, diagnostics })
        .map_err(ArtifactError::Allocation)
}

fn enum_product(
    owner: &DeclarationId,
    aggregates: &super::semantic_import::ImportedSemanticAggregates,
    diagnostics: Arc<[SemanticDiagnostic]>,
    allocations: &mut AllocationBudget,
) -> Result<Arc<EnumDeclarationProduct>, ArtifactError> {
    let info = aggregates
        .enum_semantics
        .enums
        .get(owner)
        .cloned()
        .ok_or_else(|| invalid("enum query identity has no reconstructed declaration product"))?;

    let mut variants = allocations
        .vector::<Arc<VariantInfo>>(info.variants.len())
        .map_err(ArtifactError::Allocation)?;
    for variant_id in info.variants.iter() {
        let variant = aggregates
            .enum_semantics
            .variants
            .get(variant_id)
            .ok_or_else(|| invalid("enum query identity references a missing reconstructed variant"))?;
        if &variant.id.owner != owner {
            return Err(invalid("enum query identity contains a variant owned by another declaration"));
        }
        variants.push(variant.clone());
    }
    let variants = allocations.arc_slice(variants).map_err(ArtifactError::Allocation)?;
    allocations
        .arc(EnumDeclarationProduct {
            info,
            variants,
            diagnostics,
        })
        .map_err(ArtifactError::Allocation)
}

fn ready(
    key: QueryKey,
    input_fingerprint: InputFingerprint,
    product_fingerprint: ProductFingerprint,
    product: SemanticProduct,
) -> ImportedReadyQueryProduct {
    ImportedReadyQueryProduct {
        key,
        input_fingerprint,
        product_fingerprint,
        product,
    }
}

fn is_structural_kind(kind: Kind) -> bool {
    matches!(kind, Kind::DataDeclaration | Kind::EnumDeclaration | Kind::AssociatedSurface)
}

fn require_zero_dependency_contract(root: &QueryRootEntryV3) -> Result<(), ArtifactError> {
    if !root.dependency_indices.is_empty() {
        return Err(invalid(format!(
            "artifact {:?} root carries dependency edges but the current-main query contract has none",
            root.key.kind
        )));
    }
    Ok(())
}

fn require_exact_structural_aggregate_owner(evidence: &DependencyEvidenceEntryV3) -> Result<(), ArtifactError> {
    let Owner::Declaration(key_owner) = &evidence.key.owner else {
        return Err(invalid("structural dependency evidence key is not declaration-owned"));
    };
    let Owner::Declaration(aggregate_owner) = &evidence.stable_aggregate_owner else {
        return Err(invalid("structural dependency evidence aggregate owner is not a declaration"));
    };
    if key_owner != aggregate_owner {
        return Err(invalid("structural dependency evidence points at a different aggregate owner"));
    }
    Ok(())
}

fn validate_portable_root_fingerprints(root: &QueryRootEntryV3, world: &SemanticWorldV3) -> Result<(), ArtifactError> {
    let Some((input, product)) = portable_structural_root_fingerprints(root, world)? else {
        return Err(invalid("portable structural fingerprint requested for a non-structural query root"));
    };
    if root.input_fingerprint != input {
        return Err(invalid(format!(
            "artifact {:?} portable input fingerprint is stale or does not match its canonical semantic row",
            root.key.kind
        )));
    }
    if root.product_fingerprint != product {
        return Err(invalid(format!(
            "artifact {:?} portable product fingerprint is stale or does not match its canonical semantic row",
            root.key.kind
        )));
    }
    Ok(())
}

fn validate_portable_evidence_fingerprint(evidence: &DependencyEvidenceEntryV3, world: &SemanticWorldV3) -> Result<(), ArtifactError> {
    let Some(product) = portable_structural_dependency_product_fingerprint(&evidence.key, world)? else {
        return Err(invalid("portable structural dependency fingerprint requested for a non-structural query key"));
    };
    if evidence.product_fingerprint != product {
        return Err(invalid(format!(
            "artifact {:?} portable dependency fingerprint is stale or does not match its canonical semantic row",
            evidence.key.kind
        )));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
