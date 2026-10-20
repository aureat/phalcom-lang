//! Reconstruction of reusable typed query products from validated artifact rows.
//!
//! This module deliberately starts with the source-independent structural
//! declaration families whose live query implementations have no dependency
//! edges: `DataDeclaration`, `EnumDeclaration`, `AssociatedSurface`, and
//! `TypeMemberDeclaration`. Other query families remain
//! wire DTOs until their complete current-main dependency contract is imported.

use crate::artifact::identity::StableOwnerCatalog;
use crate::db::fingerprint::{
    associated_surface_input_fingerprint, associated_surface_product_fingerprint, callable_definition_input_fingerprint,
    callable_definition_product_fingerprint, data_declaration_input_fingerprint, data_declaration_product_fingerprint,
    enum_declaration_input_fingerprint, enum_declaration_product_fingerprint, type_member_declaration_input_fingerprint,
    type_member_declaration_product_fingerprint,
};
use crate::db::product::{DataDeclarationProduct, EnumDeclarationProduct};
use crate::db::{InputFingerprint, ProductFingerprint, QueryKey, SemanticDb, SemanticProduct};
use crate::diagnostic::SemanticDiagnostic;
use crate::enum_semantics::VariantInfo;
use crate::identity::DeclarationId;
use phalcom_compiler_artifact::query::{
    DependencyEvidenceEntryV3, PortableQueryKindV1 as Kind, QueryImageV3, QueryRootEntryV3,
    portable_structural_dependency_product_fingerprint_v4, portable_structural_root_fingerprints_v4,
};
use phalcom_compiler_artifact::semantic::world::SemanticWorldV4;
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;
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
    associated: &super::associated_import::ImportedAssociatedAuthority,
    callables: &super::callable_import::ImportedCallableAuthority,
    owners: &StableOwnerCatalog<'_>,
    world: &SemanticWorldV4,
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
            Kind::AssociatedSurface => reconstruct_associated_surface_root(root, associated, owners, world)?,
            Kind::TypeMemberDeclaration => reconstruct_type_member_root(root, associated, owners, world)?,
            Kind::CallableDefinition => reconstruct_callable_definition_root(root, callables, owners, world)?,
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
            Kind::AssociatedSurface => Some(reconstruct_associated_surface_evidence(evidence, associated, owners, world)?),
            Kind::TypeMemberDeclaration => Some(reconstruct_type_member_evidence(evidence, associated, owners, world)?),
            Kind::CallableDefinition => Some(reconstruct_callable_definition_evidence(evidence, callables, owners, world)?),
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
    world: &SemanticWorldV4,
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
    world: &SemanticWorldV4,
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
    world: &SemanticWorldV4,
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
    world: &SemanticWorldV4,
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
    associated: &super::associated_import::ImportedAssociatedAuthority,
    owners: &StableOwnerCatalog<'_>,
    world: &SemanticWorldV4,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_zero_dependency_contract(root)?;
    validate_portable_root_fingerprints(root, world)?;
    let key = QueryKey::from_portable_owner(root.key.kind, &root.key.owner, owners)?;
    let QueryKey::AssociatedSurface(owner) = &key else {
        return Err(invalid("associated-surface query root did not relocate to AssociatedSurface"));
    };
    let surface = associated
        .surface(owner)
        .cloned()
        .ok_or_else(|| invalid("associated-surface query owner has no reconstructed P7 semantic authority"))?;
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
    associated: &super::associated_import::ImportedAssociatedAuthority,
    owners: &StableOwnerCatalog<'_>,
    world: &SemanticWorldV4,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_exact_structural_aggregate_owner(evidence)?;
    validate_portable_evidence_fingerprint(evidence, world)?;
    let key = QueryKey::from_portable_owner(evidence.key.kind, &evidence.key.owner, owners)?;
    let QueryKey::AssociatedSurface(owner) = &key else {
        return Err(invalid("associated-surface dependency evidence did not relocate to AssociatedSurface"));
    };
    let surface = associated
        .surface(owner)
        .cloned()
        .ok_or_else(|| invalid("associated-surface dependency owner has no reconstructed P7 semantic authority"))?;
    let input_fingerprint = associated_surface_input_fingerprint(&surface);
    let product_fingerprint = associated_surface_product_fingerprint(&surface);
    Ok(ready(
        key,
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::AssociatedSurface(surface),
    ))
}

fn reconstruct_type_member_root(
    root: &QueryRootEntryV3,
    associated: &super::associated_import::ImportedAssociatedAuthority,
    owners: &StableOwnerCatalog<'_>,
    world: &SemanticWorldV4,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_zero_dependency_contract(root)?;
    validate_portable_root_fingerprints(root, world)?;
    let key = QueryKey::from_portable_owner(root.key.kind, &root.key.owner, owners)?;
    let QueryKey::TypeMemberDeclaration(id) = &key else {
        return Err(invalid("type-member query root did not relocate to TypeMemberDeclaration"));
    };
    let product = associated
        .type_member(id)
        .cloned()
        .ok_or_else(|| invalid("type-member query owner has no reconstructed P7 semantic authority"))?;
    let input_fingerprint = type_member_declaration_input_fingerprint(None, &product);
    let product_fingerprint = type_member_declaration_product_fingerprint(&product);
    Ok(ready(
        key,
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::TypeMemberDeclaration(product),
    ))
}

fn reconstruct_type_member_evidence(
    evidence: &DependencyEvidenceEntryV3,
    associated: &super::associated_import::ImportedAssociatedAuthority,
    owners: &StableOwnerCatalog<'_>,
    world: &SemanticWorldV4,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_exact_structural_aggregate_owner(evidence)?;
    validate_portable_evidence_fingerprint(evidence, world)?;
    let key = QueryKey::from_portable_owner(evidence.key.kind, &evidence.key.owner, owners)?;
    let QueryKey::TypeMemberDeclaration(id) = &key else {
        return Err(invalid("type-member dependency evidence did not relocate to TypeMemberDeclaration"));
    };
    let product = associated
        .type_member(id)
        .cloned()
        .ok_or_else(|| invalid("type-member dependency owner has no reconstructed P7 semantic authority"))?;
    let input_fingerprint = type_member_declaration_input_fingerprint(None, &product);
    let product_fingerprint = type_member_declaration_product_fingerprint(&product);
    Ok(ready(
        key,
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::TypeMemberDeclaration(product),
    ))
}

fn reconstruct_callable_definition_root(
    root: &QueryRootEntryV3,
    callables: &super::callable_import::ImportedCallableAuthority,
    owners: &StableOwnerCatalog<'_>,
    world: &SemanticWorldV4,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_zero_dependency_contract(root)?;
    validate_portable_root_fingerprints(root, world)?;
    let key = QueryKey::from_portable_owner(root.key.kind, &root.key.owner, owners)?;
    let QueryKey::CallableDefinition(callable) = &key else {
        return Err(invalid("callable-definition query root did not relocate to CallableDefinition"));
    };
    let product = callables
        .definition(callable)
        .cloned()
        .ok_or_else(|| invalid("callable-definition query owner has no reconstructed P7 semantic authority"))?;
    let input_fingerprint = callable_definition_input_fingerprint(&product);
    let product_fingerprint = callable_definition_product_fingerprint(&product);
    Ok(ready(
        key,
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::CallableDefinition(product),
    ))
}

fn reconstruct_callable_definition_evidence(
    evidence: &DependencyEvidenceEntryV3,
    callables: &super::callable_import::ImportedCallableAuthority,
    owners: &StableOwnerCatalog<'_>,
    world: &SemanticWorldV4,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_exact_structural_aggregate_owner(evidence)?;
    validate_portable_evidence_fingerprint(evidence, world)?;
    let key = QueryKey::from_portable_owner(evidence.key.kind, &evidence.key.owner, owners)?;
    let QueryKey::CallableDefinition(callable) = &key else {
        return Err(invalid("callable-definition dependency evidence did not relocate to CallableDefinition"));
    };
    let product = callables
        .definition(callable)
        .cloned()
        .ok_or_else(|| invalid("callable-definition dependency owner has no reconstructed P7 semantic authority"))?;
    let input_fingerprint = callable_definition_input_fingerprint(&product);
    let product_fingerprint = callable_definition_product_fingerprint(&product);
    Ok(ready(
        key,
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::CallableDefinition(product),
    ))
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
    matches!(
        kind,
        Kind::DataDeclaration
            | Kind::EnumDeclaration
            | Kind::AssociatedSurface
            | Kind::TypeMemberDeclaration
            | Kind::CallableDefinition
    )
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
    if evidence.key.owner != evidence.stable_aggregate_owner {
        return Err(invalid("structural dependency evidence points at a different exact stable owner"));
    }
    Ok(())
}

fn validate_portable_root_fingerprints(root: &QueryRootEntryV3, world: &SemanticWorldV4) -> Result<(), ArtifactError> {
    let Some((input, product)) = portable_structural_root_fingerprints_v4(root, world)? else {
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

fn validate_portable_evidence_fingerprint(evidence: &DependencyEvidenceEntryV3, world: &SemanticWorldV4) -> Result<(), ArtifactError> {
    let Some(product) = portable_structural_dependency_product_fingerprint_v4(&evidence.key, world)? else {
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
