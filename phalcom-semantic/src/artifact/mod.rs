//! Transactional semantic artifact export, container encoding, validation, and import.
//!
//! (ARTF001.C1.P2)

mod query_policy;
pub use query_policy::{portable_query_kind, query_kind_profile_support};

use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_compiler_artifact::header::TeloscHeaderV2;
use phalcom_compiler_artifact::identity::CanonicalCompatibilityIdentity;
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::metadata::{MetadataPreflightLimits, MetadataSectionV1};
use phalcom_compiler_artifact::module_world::ModuleWorldV2;
use phalcom_compiler_artifact::payload::CompilerArtifactPayload;
use phalcom_compiler_artifact::query::QueryImageV3;
use phalcom_compiler_artifact::semantic::world::SemanticWorldV3;
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_modules::canonical_products::CanonicalExternalModuleWorld;
use phalcom_modules::compiler_artifact::{load_module_world_with_allocation_budget, load_module_world_with_project_catalog_and_allocation_budget};
use phalcom_modules::{ProjectUniverse, TrustedPackageResolver, stage_artifact_project_authorities};
use phalcom_type_meta::SemanticMetadataBundle;
use phalcom_type_meta::allocation::AllocationBudget;
use std::sync::Arc;

/// A completed compiler image ready to be encoded into a `.telosc` container.
#[derive(Clone, Debug, PartialEq)]
pub struct CompilerArtifactImage {
    pub header: TeloscHeaderV2,
    pub module_world: ModuleWorldV2,
    pub semantic_world: SemanticWorldV3,
    pub query_image: QueryImageV3,
    pub metadata: MetadataSectionV1,
}

impl CompilerArtifactImage {
    /// Encodes this image into a validated `.telosc` container byte vector.
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        let payload = CompilerArtifactPayload {
            header: self.header.clone(),
            module_world: self.module_world.clone(),
            semantic_world: self.semantic_world.clone(),
            query_image: self.query_image.clone(),
            metadata: self.metadata.clone(),
        };
        payload.encode()
    }
}

/// Validated artifact data staged for a future live semantic import.
///
/// `semantic_world` and `query_image` remain portable wire DTOs here. This value is not a
/// `SemanticSnapshot`, and it is not published into `SemanticDb`.
#[derive(Clone, Debug)]
pub struct ImportedCompilerImage {
    pub external_world: Arc<CanonicalExternalModuleWorld>,
    /// Project identity graph retained for subsequent semantic import and local overlays.
    pub project_universe: Arc<ProjectUniverse>,
    pub metadata_bundle: SemanticMetadataBundle,
    pub semantic_world: SemanticWorldV3,
    pub query_image: QueryImageV3,
}

/// Transactional staging loader for compiler artifacts.
///
/// Ensures fail-closed semantics: allocations are tracked against a budget,
/// all stages are loaded into private uncommitted fields, and on any error,
/// staging structures are dropped without publishing partial state.
pub struct ArtifactImportTransaction<'a> {
    pub parent_budget: &'a mut AllocationBudget,
    pub child_ledger: Option<phalcom_type_meta::allocation::ChildAllocationBudget>,
    pub limits: ValidationLimits,
    pub staged_module_world: Option<Arc<CanonicalExternalModuleWorld>>,
    pub staged_metadata_bundle: Option<SemanticMetadataBundle>,
    pub staged_semantic_world: Option<SemanticWorldV3>,
    pub staged_query_image: Option<QueryImageV3>,
    project_catalog: Option<Arc<ProjectUniverse>>,
    trusted_package_resolver: Option<&'a dyn TrustedPackageResolver>,
    committed: bool,
}

impl<'a> ArtifactImportTransaction<'a> {
    pub fn new(budget: &'a mut AllocationBudget, limits: ValidationLimits) -> Self {
        let child_ledger = budget.child(usize::MAX);
        Self {
            parent_budget: budget,
            child_ledger: Some(child_ledger),
            limits,
            staged_module_world: None,
            staged_metadata_bundle: None,
            staged_semantic_world: None,
            staged_query_image: None,
            project_catalog: None,
            trusted_package_resolver: None,
            committed: false,
        }
    }

    /// Creates an import transaction bound to the caller's existing project graph.
    /// Package authority is obtained from `trusted_package_resolver` and registered
    /// only in the transaction's private project-catalog candidate.
    pub fn with_project_catalog(
        budget: &'a mut AllocationBudget,
        limits: ValidationLimits,
        projects: &ProjectUniverse,
        trusted_package_resolver: &'a dyn TrustedPackageResolver,
    ) -> Self {
        let mut transaction = Self::new(budget, limits);
        transaction.project_catalog = Some(Arc::new(projects.clone()));
        transaction.trusted_package_resolver = Some(trusted_package_resolver);
        transaction
    }

    /// Stages an already-parsed and verified `CompilerArtifactPayload`.
    pub fn stage_payload(&mut self, payload: CompilerArtifactPayload) -> Result<&mut Self, ArtifactError> {
        self.stage_payload_with_digest(payload, None)
    }

    fn stage_payload_with_digest(
        &mut self,
        payload: CompilerArtifactPayload,
        validated_artifact_digest: Option<ArtifactDigest>,
    ) -> Result<&mut Self, ArtifactError> {
        payload.validate_cross_section(&self.limits)?;

        let child = self
            .child_ledger
            .as_mut()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("transaction already committed".into())))?;

        // 1. Resolve and stage exact project authority, then reconstruct the module world.
        let external = if let Some(projects) = self.project_catalog.as_ref() {
            let package_refs_present = payload
                .module_world
                .external_projects
                .iter()
                .chain(payload.module_world.modules.iter().map(|module| &module.identity.project))
                .any(|project| matches!(project, phalcom_type_meta::identity::StableProjectRef::Package { .. }));
            if package_refs_present && validated_artifact_digest.is_none() {
                return Err(ArtifactError::Validation(ValidationError::ReferentialIntegrity(
                    "package project authority requires a validated artifact-byte digest".into(),
                )));
            }
            let resolver = self.trusted_package_resolver.ok_or_else(|| {
                ArtifactError::Validation(ValidationError::ReferentialIntegrity(
                    "package project authority requires the caller's trusted resolver".into(),
                ))
            })?;
            let project_universe = stage_artifact_project_authorities(
                projects,
                &payload.module_world,
                &payload.header.package_identity,
                validated_artifact_digest.unwrap_or(ArtifactDigest::ZERO),
                resolver,
            )?;
            load_module_world_with_project_catalog_and_allocation_budget(&payload.module_world, &self.limits, child, project_universe)?
        } else {
            let package_refs_present = payload
                .module_world
                .external_projects
                .iter()
                .chain(payload.module_world.modules.iter().map(|module| &module.identity.project))
                .any(|project| matches!(project, phalcom_type_meta::identity::StableProjectRef::Package { .. }));
            if package_refs_present {
                return Err(ArtifactError::Validation(ValidationError::ReferentialIntegrity(
                    "package project authority requires a caller-supplied project catalog and trusted resolver".into(),
                )));
            }
            load_module_world_with_allocation_budget(&payload.module_world, &self.limits, child)?
        };
        self.staged_module_world = Some(child.arc(external).map_err(ArtifactError::Allocation)?);
        self.project_catalog = Some(self.staged_module_world.as_ref().expect("staged above").linked().universe.clone());

        // 2. Stage metadata bundle under child ledger
        let preflight = metadata_preflight_limits(&self.limits);
        let bundle = payload
            .metadata
            .decode_bundle_with_allocation_budget(
                &preflight,
                &phalcom_type_meta::ValidationLimits::default(),
                &payload.metadata.package_digest,
                child,
            )
            .map_err(|e| match e {
                phalcom_compiler_artifact::metadata::MetadataTransportError::Decode(phalcom_type_meta::MetadataDecodeError::Budgeted(
                    phalcom_type_meta::budgeted::DecodeError::Allocation(err),
                )) => ArtifactError::Allocation(err),
                _ => ArtifactError::Validation(ValidationError::ReferentialIntegrity(format!("{e:?}"))),
            })?;
        self.staged_metadata_bundle = Some(bundle);

        // 3. Stage semantic world and query image
        self.staged_semantic_world = Some(payload.semantic_world);
        self.staged_query_image = Some(payload.query_image);

        Ok(self)
    }

    /// Stages a raw `.telosc` binary buffer.
    pub fn stage_bytes(&mut self, bytes: &[u8], expected_identity: Option<&CanonicalCompatibilityIdentity>) -> Result<&mut Self, ArtifactError> {
        let payload = {
            let child = self
                .child_ledger
                .as_mut()
                .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("transaction already committed".into())))?;
            CompilerArtifactPayload::decode_and_validate(bytes, &self.limits, expected_identity, child)?
        };
        let artifact_digest = payload.header.artifact_digest;
        self.stage_payload_with_digest(payload, Some(artifact_digest))
    }

    /// Commits staged transport data into an `ImportedCompilerImage` value.
    ///
    /// This does not publish semantic products into a workspace session.
    pub fn commit(mut self) -> Result<ImportedCompilerImage, ArtifactError> {
        let external_world = self
            .staged_module_world
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged module world".into())))?;
        let metadata_bundle = self
            .staged_metadata_bundle
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged metadata bundle".into())))?;
        let semantic_world = self
            .staged_semantic_world
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged semantic world".into())))?;
        let query_image = self
            .staged_query_image
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged query image".into())))?;
        let project_universe = self
            .project_catalog
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged project catalog".into())))?;
        let child = self
            .child_ledger
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no child ledger".into())))?;

        self.parent_budget.commit_child(child).map_err(ArtifactError::Allocation)?;

        self.committed = true;
        Ok(ImportedCompilerImage {
            external_world,
            project_universe,
            metadata_bundle,
            semantic_world,
            query_image,
        })
    }
}

impl Drop for ArtifactImportTransaction<'_> {
    fn drop(&mut self) {
        if !self.committed {
            self.child_ledger = None;
            self.staged_module_world = None;
            self.staged_metadata_bundle = None;
            self.staged_semantic_world = None;
            self.staged_query_image = None;
            self.project_catalog = None;
        }
    }
}

/// Helper converting general validation limits into metadata preflight limits.
pub fn metadata_preflight_limits(limits: &ValidationLimits) -> MetadataPreflightLimits {
    MetadataPreflightLimits {
        max_metadata_bytes: limits.max_string_bytes,
        max_token_count: limits.max_semantic_entries_per_family as u32,
        max_container_count: limits.max_semantic_entries_per_family as u32,
        max_nesting_depth: limits.max_recursion_depth as u16,
    }
}

/// Encodes a `CompilerArtifactImage` into `.telosc` binary bytes.
pub fn encode_artifact(image: &CompilerArtifactImage) -> Result<Vec<u8>, ArtifactError> {
    image.encode()
}

/// Decodes, verifies framing, digests, schemas, and validates cross-section invariants.
pub fn decode_and_validate_artifact(
    bytes: &[u8],
    limits: &ValidationLimits,
    expected_identity: Option<&CanonicalCompatibilityIdentity>,
    budget: &mut AllocationBudget,
) -> Result<CompilerArtifactPayload, ArtifactError> {
    CompilerArtifactPayload::decode_and_validate(bytes, limits, expected_identity, budget)
}

/// Transactionally stages a compiler artifact payload into an `ImportedCompilerImage` value.
///
/// Semantic/query DTO reconstruction and workspace publication are not implemented yet.
pub fn import_compiler_image(
    artifact: CompilerArtifactPayload,
    limits: &ValidationLimits,
    budget: &mut AllocationBudget,
) -> Result<ImportedCompilerImage, ArtifactError> {
    let mut tx = ArtifactImportTransaction::new(budget, *limits);
    tx.stage_payload(artifact)?;
    tx.commit()
}

/// Decodes and stages artifact bytes against an existing workspace project graph.
/// Package references require validated bytes and receipts from the supplied
/// trusted package resolver. The returned image retains the resulting graph.
pub fn import_compiler_image_bytes_with_project_catalog(
    bytes: &[u8],
    expected_identity: Option<&CanonicalCompatibilityIdentity>,
    limits: &ValidationLimits,
    budget: &mut AllocationBudget,
    projects: &ProjectUniverse,
    trusted_package_resolver: &dyn TrustedPackageResolver,
) -> Result<ImportedCompilerImage, ArtifactError> {
    let mut transaction = ArtifactImportTransaction::with_project_catalog(budget, *limits, projects, trusted_package_resolver);
    transaction.stage_bytes(bytes, expected_identity)?;
    transaction.commit()
}
