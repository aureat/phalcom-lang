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
use phalcom_compiler_artifact::query::{PortableQueryKindV1 as Kind, QueryImageV3, QueryRootEntryV3};
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
/// Exact owner/row identity is checked again here even though the staging
/// transaction performs an earlier cross-row validation. Keeping the typed
/// reconstruction self-contained prevents a future caller from accidentally
/// treating row bounds as sufficient authority.
pub(crate) fn reconstruct_structural_query_products(
    image: &QueryImageV3,
    aggregates: &super::semantic_import::ImportedSemanticAggregates,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedQueryProducts, ArtifactError> {
    let structural_count = image
        .supported_roots
        .iter()
        .filter(|root| matches!(root.key.kind, Kind::DataDeclaration | Kind::EnumDeclaration))
        .count();
    let mut ready = allocations
        .vector::<ImportedReadyQueryProduct>(structural_count)
        .map_err(ArtifactError::Allocation)?;
    let empty_diagnostics = allocations
        .arc_slice(Vec::<SemanticDiagnostic>::new())
        .map_err(ArtifactError::Allocation)?;

    for root in &image.supported_roots {
        let imported = match root.key.kind {
            Kind::DataDeclaration => reconstruct_data_product(root, aggregates, owners, empty_diagnostics.clone(), allocations)?,
            Kind::EnumDeclaration => reconstruct_enum_product(root, aggregates, owners, empty_diagnostics.clone(), allocations)?,
            _ => continue,
        };
        ready.push(imported);
    }

    // Portable validation rejects duplicate wire keys. This second check is on
    // relocated live keys, closing any identity-aliasing gap introduced by
    // conversion. Sorting is in-place and requires no auxiliary allocation.
    ready.sort_by(|left, right| left.key.cmp(&right.key));
    if ready.windows(2).any(|pair| pair[0].key == pair[1].key) {
        return Err(invalid("multiple artifact query roots relocate to the same live query key"));
    }

    Ok(ImportedQueryProducts { ready })
}

fn reconstruct_data_product(
    root: &QueryRootEntryV3,
    aggregates: &super::semantic_import::ImportedSemanticAggregates,
    owners: &StableOwnerCatalog<'_>,
    diagnostics: Arc<[SemanticDiagnostic]>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_zero_dependency_contract(root)?;
    let key = QueryKey::from_portable_owner(root.key.kind, &root.key.owner, owners)?;
    let QueryKey::DataDeclaration(owner) = &key else {
        return Err(invalid("data query root did not relocate to DataDeclaration"));
    };
    let info = aggregates
        .data_semantics
        .data_info(owner)
        .cloned()
        .ok_or_else(|| invalid("data query root has no reconstructed declaration product"))?;

    let product = allocations
        .arc(DataDeclarationProduct { info, diagnostics })
        .map_err(ArtifactError::Allocation)?;
    let input_fingerprint = data_declaration_input_fingerprint(&product);
    let product_fingerprint = data_declaration_product_fingerprint(&product);
    validate_root_fingerprints(root, input_fingerprint, product_fingerprint)?;

    Ok(ImportedReadyQueryProduct {
        key,
        input_fingerprint,
        product_fingerprint,
        product: SemanticProduct::DataDeclaration(product),
    })
}

fn reconstruct_enum_product(
    root: &QueryRootEntryV3,
    aggregates: &super::semantic_import::ImportedSemanticAggregates,
    owners: &StableOwnerCatalog<'_>,
    diagnostics: Arc<[SemanticDiagnostic]>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedReadyQueryProduct, ArtifactError> {
    require_zero_dependency_contract(root)?;
    let key = QueryKey::from_portable_owner(root.key.kind, &root.key.owner, owners)?;
    let QueryKey::EnumDeclaration(owner) = &key else {
        return Err(invalid("enum query root did not relocate to EnumDeclaration"));
    };
    let info = aggregates
        .enum_semantics
        .enums
        .get(owner)
        .cloned()
        .ok_or_else(|| invalid("enum query root has no reconstructed declaration product"))?;

    let mut variants = allocations
        .vector::<VariantInfo>(info.variants.len())
        .map_err(ArtifactError::Allocation)?;
    for variant_id in info.variants.iter() {
        let variant = aggregates
            .enum_semantics
            .variants
            .get(variant_id)
            .ok_or_else(|| invalid("enum query root references a missing reconstructed variant"))?;
        if variant.id.owner != *owner {
            return Err(invalid("enum query root contains a variant owned by another declaration"));
        }
        variants.push((**variant).clone());
    }
    let variants = allocations.arc_slice(variants).map_err(ArtifactError::Allocation)?;
    let product = allocations
        .arc(EnumDeclarationProduct {
            info,
            variants,
            diagnostics,
        })
        .map_err(ArtifactError::Allocation)?;
    let input_fingerprint = enum_declaration_input_fingerprint(&product);
    let product_fingerprint = enum_declaration_product_fingerprint(&product);
    validate_root_fingerprints(root, input_fingerprint, product_fingerprint)?;

    Ok(ImportedReadyQueryProduct {
        key,
        input_fingerprint,
        product_fingerprint,
        product: SemanticProduct::EnumDeclaration(product),
    })
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

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
