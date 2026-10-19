//! Reconstruction of reusable typed query products from validated artifact rows.
//!
//! This module deliberately starts with the source-independent structural
//! declaration families whose live query implementations have no dependency
//! edges: `DataDeclaration` and `EnumDeclaration`. Other query families remain
//! wire DTOs until their complete current-main dependency contract is imported.

use crate::artifact::identity::StableOwnerCatalog;
use crate::db::fingerprint::{
    data_declaration_input_fingerprint, data_declaration_product_fingerprint, enum_declaration_input_fingerprint,
    enum_declaration_product_fingerprint,
};
use crate::db::product::{DataDeclarationProduct, EnumDeclarationProduct};
use crate::db::{InputFingerprint, ProductFingerprint, QueryKey, SemanticDb, SemanticProduct};
use crate::diagnostic::SemanticDiagnostic;
use crate::enum_semantics::VariantInfo;
use crate::identity::DeclarationId;
use phalcom_compiler_artifact::query::{
    DependencyEvidenceEntryV3, PortableQueryKindV1 as Kind, PortableQueryOwnerKeyV1 as Owner, QueryImageV3, QueryRootEntryV3,
};
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
pub struct ImportedReadyQueryProduct {
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
}

impl ImportedQueryProducts {
    pub fn len(&self) -> usize {
        self.ready.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ready.is_empty()
    }

    pub(crate) fn ready(&self) -> &[ImportedReadyQueryProduct] {
        &self.ready
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
/// called. Here they are relocated to live keys, rebuilt as current-main typed
/// products, and compared against canonical current-main fingerprints.
///
/// Structural dependency evidence is also reconstructed. This is important for
/// later query families: a dependency-only DataDeclaration/EnumDeclaration must
/// become a real database product before a dependent root can be validated.
pub(crate) fn reconstruct_structural_query_products(
    image: &QueryImageV3,
    aggregates: &super::semantic_import::ImportedSemanticAggregates,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedQueryProducts, ArtifactError> {
    let candidate_count = image
        .supported_roots
        .iter()
        .filter(|root| is_structural_kind(root.key.kind))
        .count()
        .checked_add(image.dependency_evidence.iter().filter(|evidence| is_structural_kind(evidence.key.kind)).count())
        .ok_or_else(|| ArtifactError::Allocation(phalcom_type_meta::allocation::AllocationError::Overflow))?;

    if candidate_count == 0 {
        return Ok(ImportedQueryProducts::default());
    }

    let mut ready = allocations
        .vector::<ImportedReadyQueryProduct>(candidate_count)
        .map_err(ArtifactError::Allocation)?;
    let empty_diagnostics = allocations
        .arc_slice(Vec::<SemanticDiagnostic>::new())
        .map_err(ArtifactError::Allocation)?;

    for root in &image.supported_roots {
        let imported = match root.key.kind {
            Kind::DataDeclaration => reconstruct_data_root(root, aggregates, owners, empty_diagnostics.clone(), allocations)?,
            Kind::EnumDeclaration => reconstruct_enum_root(root, aggregates, owners, empty_diagnostics.clone(), allocations)?,
            _ => continue,
        };
        ready.push(imported);
    }

    for evidence in &image.dependency_evidence {
        let imported = match evidence.key.kind {
            Kind::DataDeclaration => reconstruct_data_evidence(evidence, aggregates, owners, empty_diagnostics.clone(), allocations)?,
            Kind::EnumDeclaration => reconstruct_enum_evidence(evidence, aggregates, owners, empty_diagnostics.clone(), allocations)?,
            _ => continue,
        };
        ready.push(imported);
    }

    // Portable validation rejects duplicate roots/evidence within their own
    // collections. A key may legitimately be both a root and dependency
    // evidence for another root, so relocated duplicates are accepted only when
    // their canonical fingerprints agree.
    ready.sort_by(|left, right| left.key.cmp(&right.key));
    for pair in ready.windows(2) {
        if pair[0].key == pair[1].key
            && (pair[0].input_fingerprint != pair[1].input_fingerprint || pair[0].product_fingerprint != pair[1].product_fingerprint)
        {
            return Err(invalid("artifact root and dependency evidence disagree for the same live query key"));
        }
    }
    ready.dedup_by(|left, right| left.key == right.key);

    Ok(ImportedQueryProducts { ready })
}

fn reconstruct_data_root(
    root: &QueryRootEntryV3,
    aggregates: &super::semantic_import::ImportedSemanticAggregates,
    owners: &StableOwnerCatalog<'_>,
    diagnostics: Arc<[SemanticDiagnostic]>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_zero_dependency_contract(root)?;
    require_relocation_stable_fingerprint_owner(&root.key.owner)?;
    let key = QueryKey::from_portable_owner(root.key.kind, &root.key.owner, owners)?;
    let QueryKey::DataDeclaration(owner) = &key else {
        return Err(invalid("data query root did not relocate to DataDeclaration"));
    };
    let product = data_product(owner, aggregates, diagnostics, allocations)?;
    let input_fingerprint = data_declaration_input_fingerprint(&product);
    let product_fingerprint = data_declaration_product_fingerprint(&product);
    validate_root_fingerprints(root, input_fingerprint, product_fingerprint)?;
    Ok(ready(key, input_fingerprint, product_fingerprint, SemanticProduct::DataDeclaration(product)))
}

fn reconstruct_enum_root(
    root: &QueryRootEntryV3,
    aggregates: &super::semantic_import::ImportedSemanticAggregates,
    owners: &StableOwnerCatalog<'_>,
    diagnostics: Arc<[SemanticDiagnostic]>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_zero_dependency_contract(root)?;
    require_relocation_stable_fingerprint_owner(&root.key.owner)?;
    let key = QueryKey::from_portable_owner(root.key.kind, &root.key.owner, owners)?;
    let QueryKey::EnumDeclaration(owner) = &key else {
        return Err(invalid("enum query root did not relocate to EnumDeclaration"));
    };
    let product = enum_product(owner, aggregates, diagnostics, allocations)?;
    let input_fingerprint = enum_declaration_input_fingerprint(&product);
    let product_fingerprint = enum_declaration_product_fingerprint(&product);
    validate_root_fingerprints(root, input_fingerprint, product_fingerprint)?;
    Ok(ready(key, input_fingerprint, product_fingerprint, SemanticProduct::EnumDeclaration(product)))
}

fn reconstruct_data_evidence(
    evidence: &DependencyEvidenceEntryV3,
    aggregates: &super::semantic_import::ImportedSemanticAggregates,
    owners: &StableOwnerCatalog<'_>,
    diagnostics: Arc<[SemanticDiagnostic]>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_exact_structural_aggregate_owner(evidence)?;
    require_relocation_stable_fingerprint_owner(&evidence.key.owner)?;
    let key = QueryKey::from_portable_owner(evidence.key.kind, &evidence.key.owner, owners)?;
    let QueryKey::DataDeclaration(owner) = &key else {
        return Err(invalid("data dependency evidence did not relocate to DataDeclaration"));
    };
    let product = data_product(owner, aggregates, diagnostics, allocations)?;
    let input_fingerprint = data_declaration_input_fingerprint(&product);
    let product_fingerprint = data_declaration_product_fingerprint(&product);
    validate_evidence_fingerprint(evidence, product_fingerprint)?;
    Ok(ready(key, input_fingerprint, product_fingerprint, SemanticProduct::DataDeclaration(product)))
}

fn reconstruct_enum_evidence(
    evidence: &DependencyEvidenceEntryV3,
    aggregates: &super::semantic_import::ImportedSemanticAggregates,
    owners: &StableOwnerCatalog<'_>,
    diagnostics: Arc<[SemanticDiagnostic]>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_exact_structural_aggregate_owner(evidence)?;
    require_relocation_stable_fingerprint_owner(&evidence.key.owner)?;
    let key = QueryKey::from_portable_owner(evidence.key.kind, &evidence.key.owner, owners)?;
    let QueryKey::EnumDeclaration(owner) = &key else {
        return Err(invalid("enum dependency evidence did not relocate to EnumDeclaration"));
    };
    let product = enum_product(owner, aggregates, diagnostics, allocations)?;
    let input_fingerprint = enum_declaration_input_fingerprint(&product);
    let product_fingerprint = enum_declaration_product_fingerprint(&product);
    validate_evidence_fingerprint(evidence, product_fingerprint)?;
    Ok(ready(key, input_fingerprint, product_fingerprint, SemanticProduct::EnumDeclaration(product)))
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
        .vector::<VariantInfo>(info.variants.len())
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
        variants.push((**variant).clone());
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
    matches!(kind, Kind::DataDeclaration | Kind::EnumDeclaration)
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

/// The existing SemanticDb structural fingerprint helpers hash live ModuleId
/// ownership. Resolved project IDs are graph-local, so their raw fingerprint is
/// not a portable artifact identity. Until QUERY_FINGERPRINT_ABI defines and
/// exports a stable-owner fingerprint for relocated project roots, only builtin
/// owners may use direct wire-vs-live fingerprint equality. Failing closed here
/// prevents package/source artifacts from being accidentally accepted because
/// their receiving graph happened to reuse the producer's numeric project ID.
fn require_relocation_stable_fingerprint_owner(owner: &Owner) -> Result<(), ArtifactError> {
    if !matches!(owner.project(), phalcom_type_meta::identity::StableProjectRef::Builtin { .. }) {
        return Err(invalid(
            "portable query fingerprint validation for relocated non-builtin project owners is not yet defined",
        ));
    }
    Ok(())
}

fn validate_root_fingerprints(
    root: &QueryRootEntryV3,
    input_fingerprint: InputFingerprint,
    product_fingerprint: ProductFingerprint,
) -> Result<(), ArtifactError> {
    if input_fingerprint.raw() != root.input_fingerprint {
        return Err(invalid(format!(
            "artifact {:?} input fingerprint is stale or does not match the reconstructed product",
            root.key.kind
        )));
    }
    if product_fingerprint.raw() != root.product_fingerprint {
        return Err(invalid(format!(
            "artifact {:?} product fingerprint is stale or does not match the reconstructed product",
            root.key.kind
        )));
    }
    Ok(())
}

fn validate_evidence_fingerprint(evidence: &DependencyEvidenceEntryV3, product_fingerprint: ProductFingerprint) -> Result<(), ArtifactError> {
    if product_fingerprint.raw() != evidence.product_fingerprint {
        return Err(invalid(format!(
            "artifact {:?} dependency evidence fingerprint is stale or does not match the reconstructed product",
            evidence.key.kind
        )));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
