//! Staging of complete compiler wire sections before native semantic import.
//!
//! A staged wire bundle is not an external semantic base. It carries bounded,
//! cross-validated portable DTOs for exactly one artifact. Package authority is
//! obtained from the caller's trusted resolver, never from a wire display name.
//! In particular, no staged product is visible to a workspace session.

use super::telosc_staging::{StagedCompilerSections, TeloscStagingError};
use phalcom_compiler_artifact::container::TeloscContainerView;
use phalcom_compiler_artifact::header::TeloscHeaderV2;
use phalcom_compiler_artifact::identity::TeloscPackageIdentity;
use phalcom_compiler_artifact::metadata::MetadataSectionV1;
use phalcom_compiler_artifact::module_world::ModuleWorldV4;
use phalcom_compiler_artifact::query::QueryImageV4;
use phalcom_compiler_artifact::semantic::world::SemanticWorldV4;
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_modules::{CanonicalExternalModuleWorld, ProjectUniverse, TrustedPackageResolver};
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_semantic::artifact::import::validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget;
use phalcom_semantic::artifact::query::reconstruct_query_image_v4_with_allocation_budget;
use phalcom_semantic::canonical_image::{
    CanonicalDependencyFingerprints, CanonicalSemanticQueryImage, CanonicalSemanticWorldId,
};
use phalcom_semantic::snapshot::SemanticSnapshot;
use std::sync::Arc;

/// A single privately imported package generation. This is deliberately not
/// a workspace-publishable external base: independent instances still have
/// different TypeStore/StaticValueStore lineages. A composed external base
/// must relocate all dependencies into one receiving generation before
/// exposing any of these products to a SemanticWorkspaceSession.
pub(crate) struct StagedPackageSemanticGeneration {
    pub world: CanonicalSemanticWorldId,
    pub modules: Arc<CanonicalExternalModuleWorld>,
    pub snapshot: Arc<SemanticSnapshot>,
    pub queries: Arc<CanonicalSemanticQueryImage>,
    pub fingerprints: CanonicalDependencyFingerprints,
}

/// Validated portable compiler products, prior to native ID relocation.
/// Semantic products must be imported into one composed native generation
/// before the result can serve as immutable compiler/query authority.
pub(crate) struct StagedTeloscCompilerWire {
    header: TeloscHeaderV2,
    pub module_world: ModuleWorldV4,
    pub semantic_world: SemanticWorldV4,
    pub query_image: QueryImageV4,
    pub metadata: MetadataSectionV1,
}

impl StagedTeloscCompilerWire {
    /// Decode one compiler-capable artifact without granting it semantic,
    /// module, or executable ownership. All allocations share the caller's
    /// cumulative transaction budget.
    pub(crate) fn decode(
        bytes: &[u8],
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, TeloscStagingError> {
        let container = TeloscContainerView::parse_with_allocation_budget(bytes, limits, allocations)?;
        let sections = StagedCompilerSections::from_container(&container)?;
        let header = container.header().clone_with_allocation_budget(allocations)
            .map_err(ArtifactError::Allocation)?;
        let module_world = ModuleWorldV4::decode_with_allocation_budget(sections.module_world, limits, allocations)?;
        let semantic_world = SemanticWorldV4::decode_with_allocation_budget(sections.semantic_world, limits, allocations)?;
        module_world.previous.validate_execution_authority(&semantic_world.module_execution_authority)?;

        let query_image = QueryImageV4::decode_with_allocation_budget(sections.query_image, limits, allocations)?;
        query_image.validate(limits)?;
        query_image.validate_product_references(&module_world.previous.world, &semantic_world)?;
        query_image.validate_portable_structural_fingerprints_v4(&semantic_world)?;

        let metadata = MetadataSectionV1::decode_with_allocation_budget(sections.metadata, limits, allocations)?;
        if metadata.package_digest != *header.package_identity.digest().as_bytes() {
            return Err(ArtifactError::from(ValidationError::IncompatibleWorld(
                "metadata package digest does not match artifact header identity".into(),
            )).into());
        }
        Ok(Self { header, module_world, semantic_world, query_image, metadata })
    }

    /// Completes semantic and immutable-query reconstruction for exactly one
    /// package in one private transaction. On failure no session authority is
    /// published. The result must not be combined with another independently
    /// imported artifact as though their native store IDs shared a lineage.
    pub(crate) fn stage_package_semantic_generation(
        self,
        projects: &ProjectUniverse,
        resolver: &dyn TrustedPackageResolver,
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<StagedPackageSemanticGeneration, TeloscStagingError> {
        use phalcom_compiler_artifact::header::TeloscCapabilities;
        if self.header.capabilities.contains(TeloscCapabilities::PORTABLE_EXECUTABLE) {
            return Err(TeloscStagingError::Capability(
                "ordinary package executable authority is not supported by compiler-only staging",
            ));
        }
        // Validate package receipts, foreign-module exclusion and exact native
        // project bindings before reconstructing semantic owners.
        let module_products = self.stage_package_modules(projects, resolver, limits, allocations)?;
        let modules = allocations.arc(module_products).map_err(ArtifactError::Allocation)?;
        let parts = validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget(
            self.semantic_world,
            &self.module_world.previous.world,
            modules.linked().universe.as_ref(),
            limits,
            allocations,
        )?;
        let header = self.header;
        let compatibility = phalcom_compiler_artifact::CanonicalCompatibilityIdentity {
            package: header.package_identity,
            source_digest: header.source_digest,
            interface_digest: header.interface_digest,
            semantic_digest: header.semantic_digest,
            semantic_model_version: header.semantic_model_version,
            stable_identity_schema: header.stable_identity_schema,
            module_semantic_abi: header.module_semantic_abi,
            semantic_artifact_abi: header.semantic_artifact_abi,
            query_fingerprint_abi: header.query_fingerprint_abi,
            native_surface_schema_version: header.native_surface_schema_version,
            native_catalog_fingerprint: header.native_catalog_fingerprint,
            type_metadata_schema_version: header.type_metadata_schema_version,
        };
        let world = CanonicalSemanticWorldId::for_artifact_world(1, header.source_digest, compatibility);
        let (queries, fingerprints) = reconstruct_query_image_v4_with_allocation_budget(
            self.query_image,
            &parts.snapshot,
            &world,
            &modules,
            limits,
            allocations,
        )?;
        // A query-image reader is only valid for the exact imported native
        // type/static generation. Enforce that property before returning.
        queries.reader(&world, &parts.snapshot.store, &parts.snapshot.static_values)?;
        Ok(StagedPackageSemanticGeneration {
            world,
            modules,
            snapshot: parts.snapshot,
            queries,
            fingerprints,
        })
    }

    /// Privately relocate a package's module products using the caller's
    /// existing persistent project graph and trusted package receipts.
    ///
    /// This cannot be published as compiler authority on its own: the semantic
    /// wire products still require validation/relocation into the composed
    /// external semantic base. The caller owns the publication barrier.
    pub(crate) fn stage_package_modules(
        &self,
        projects: &ProjectUniverse,
        resolver: &dyn TrustedPackageResolver,
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<CanonicalExternalModuleWorld, TeloscStagingError> {
        if !matches!(&self.header.package_identity, TeloscPackageIdentity::Package { .. }) {
            return Err(ArtifactError::from(ValidationError::IncompatibleWorld(
                "ordinary package module staging requires a package artifact identity".into(),
            )).into());
        }
        let portable = &self.module_world.previous.world;
        let candidate = phalcom_modules::compiler_artifact::stage_artifact_project_authorities(
            projects,
            portable,
            &self.header.package_identity,
            self.header.artifact_digest,
            resolver,
        )?;
        let modules = phalcom_modules::compiler_artifact::load_module_world_with_project_catalog_and_allocation_budget(
            portable, limits, allocations, candidate,
        )?;
        Ok(modules)
    }
}
