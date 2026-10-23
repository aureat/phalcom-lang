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
use phalcom_semantic::artifact::import::{
    validate_and_import_semantic_world_into_type_store_with_module_world_and_project_universe_and_allocation_budget,
    validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget,
};
use phalcom_semantic::artifact::query::reconstruct_query_image_v4_with_project_catalog_and_allocation_budget;
use phalcom_semantic::canonical_image::{
    CanonicalDependencyFingerprints, CanonicalSemanticQueryImage, CanonicalSemanticWorldId,
};
use phalcom_semantic::snapshot::SemanticSnapshot;
use phalcom_semantic::types::store::TypeStore;
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
    /// Plan an immutable artifact set before importing any native owner.
    ///
    /// No semantic store is published by this step. Dependencies are sorted
    /// by their exact stable package identity, and non-artifact source
    /// dependencies are not admitted through this source-free authority path.
    pub(crate) fn package_import_order(
        artifacts: &[Self],
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<Vec<usize>, TeloscStagingError> {
        use super::telosc_dependency_plan::{PackageDependencyNode, plan_package_dependencies};

        let mut plan_budget = allocations.child(allocations.remaining());
        let mut nodes = plan_budget.vector::<PackageDependencyNode<'_>>(artifacts.len())
            .map_err(ArtifactError::Allocation)?;
        for artifact in artifacts {
            if !matches!(&artifact.header.package_identity, TeloscPackageIdentity::Package { .. }) {
                return Err(ArtifactError::from(ValidationError::IncompatibleWorld(
                    "artifact dependency plan requires exact package identities".into(),
                )).into());
            }
            let wire = &artifact.module_world.previous.world;
            let root = wire.modules.get(wire.entry.0 as usize).ok_or_else(|| {
                ArtifactError::from(ValidationError::ReferentialIntegrity("missing package artifact entry module".into()))
            })?;
            if !root.identity.path.is_empty() || !artifact.header.package_identity.matches_project(&root.identity.project) {
                return Err(ArtifactError::from(ValidationError::ReferentialIntegrity(
                    "package artifact header does not own its root module".into(),
                )).into());
            }
            if wire.modules.iter().any(|module| module.identity.project != root.identity.project) {
                return Err(ArtifactError::from(ValidationError::ReferentialIntegrity(
                    "package artifact embeds a foreign module owner".into(),
                )).into());
            }
            nodes.push(PackageDependencyNode {
                owner: &root.identity.project,
                external_projects: &wire.external_projects,
            });
        }
        let order = plan_package_dependencies(&nodes, limits, &mut plan_budget)?;
        allocations.commit_child(plan_budget).map_err(ArtifactError::Allocation)?;
        Ok(order)
    }

    /// Decode one compiler-capable artifact without granting it semantic,
    /// module, or executable ownership. All allocations share the caller's
    /// cumulative transaction budget.
    pub(crate) fn decode(
        bytes: &[u8],
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, TeloscStagingError> {
        // Private decode transaction: any failure rolls back budget authority
        // as well as withholding all partially reconstructed wire products.
        let mut decode_budget = allocations.child(allocations.remaining());
        let container = TeloscContainerView::parse_with_allocation_budget(bytes, limits, &mut decode_budget)?;
        let sections = StagedCompilerSections::from_container(&container)?;
        let header = container.header().clone_with_allocation_budget(&mut decode_budget)
            .map_err(ArtifactError::Allocation)?;
        let module_world = ModuleWorldV4::decode_with_allocation_budget(sections.module_world, limits, &mut decode_budget)?;
        let semantic_world = SemanticWorldV4::decode_with_allocation_budget(sections.semantic_world, limits, &mut decode_budget)?;
        module_world.previous.validate_execution_authority(&semantic_world.module_execution_authority)?;

        let query_image = QueryImageV4::decode_with_allocation_budget(sections.query_image, limits, &mut decode_budget)?;
        query_image.validate(limits)?;
        query_image.validate_product_references(&module_world.previous.world, &semantic_world)?;
        query_image.validate_portable_structural_fingerprints_v4(&semantic_world)?;

        let metadata = MetadataSectionV1::decode_with_allocation_budget(sections.metadata, limits, &mut decode_budget)?;
        if metadata.package_digest != *header.package_identity.digest().as_bytes() {
            return Err(ArtifactError::from(ValidationError::IncompatibleWorld(
                "metadata package digest does not match artifact header identity".into(),
            )).into());
        }
        let staged = Self { header, module_world, semantic_world, query_image, metadata };
        allocations.commit_child(decode_budget).map_err(ArtifactError::Allocation)?;
        Ok(staged)
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
        let mut generation_budget = allocations.child(allocations.remaining());
        // Validate package receipts, foreign-module exclusion and exact native
        // project bindings before reconstructing semantic owners.
        let module_products = self.stage_package_modules(projects, resolver, limits, &mut generation_budget)?;
        let modules = generation_budget.arc(module_products).map_err(ArtifactError::Allocation)?;
        let parts = validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget(
            self.semantic_world,
            &self.module_world.previous.world,
            modules.linked().universe.as_ref(),
            limits,
            &mut generation_budget,
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
        let (queries, fingerprints) = reconstruct_query_image_v4_with_project_catalog_and_allocation_budget(
            self.query_image,
            &parts.snapshot,
            &world,
            &modules,
            &self.module_world.previous.world,
            limits,
            &mut generation_budget,
        )?;
        // A query-image reader is only valid for the exact imported native
        // type/static generation. Enforce that property before returning.
        queries.reader(&world, &parts.snapshot.store, &parts.snapshot.static_values)?;
        let staged = StagedPackageSemanticGeneration {
            world,
            modules,
            snapshot: parts.snapshot,
            queries,
            fingerprints,
        };
        allocations.commit_child(generation_budget).map_err(ArtifactError::Allocation)?;
        Ok(staged)
    }

    /// Completes one package import against a shared receiving type
    /// generation. The caller-visible receiver is replaced only after module,
    /// semantic, query, reader, and allocation-transaction validation succeed.
    ///
    /// Static values remain package-local in this A4 increment, so the returned
    /// generation is still private and not a composed workspace authority.
    pub(crate) fn stage_package_semantic_generation_into_type_store(
        self,
        projects: &ProjectUniverse,
        resolver: &dyn TrustedPackageResolver,
        receiving_types: &mut TypeStore,
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<StagedPackageSemanticGeneration, TeloscStagingError> {
        use phalcom_compiler_artifact::header::TeloscCapabilities;
        if self.header.capabilities.contains(TeloscCapabilities::PORTABLE_EXECUTABLE) {
            return Err(TeloscStagingError::Capability(
                "ordinary package executable authority is not supported by compiler-only staging",
            ));
        }

        let mut generation_budget = allocations.child(allocations.remaining());
        let mut type_candidate = receiving_types.clone();
        let module_products = self.stage_package_modules(projects, resolver, limits, &mut generation_budget)?;
        let modules = generation_budget.arc(module_products).map_err(ArtifactError::Allocation)?;
        let parts =
            validate_and_import_semantic_world_into_type_store_with_module_world_and_project_universe_and_allocation_budget(
                self.semantic_world,
                &self.module_world.previous.world,
                modules.linked().universe.as_ref(),
                &mut type_candidate,
                limits,
                &mut generation_budget,
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
        let (queries, fingerprints) = reconstruct_query_image_v4_with_project_catalog_and_allocation_budget(
            self.query_image,
            &parts.snapshot,
            &world,
            &modules,
            &self.module_world.previous.world,
            limits,
            &mut generation_budget,
        )?;
        queries.reader(&world, &parts.snapshot.store, &parts.snapshot.static_values)?;

        let staged = StagedPackageSemanticGeneration {
            world,
            modules,
            snapshot: parts.snapshot,
            queries,
            fingerprints,
        };
        allocations.commit_child(generation_budget).map_err(ArtifactError::Allocation)?;
        *receiving_types = type_candidate;
        Ok(staged)
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
        let mut module_budget = allocations.child(allocations.remaining());
        let portable = &self.module_world.previous.world;
        let candidate = phalcom_modules::compiler_artifact::stage_artifact_project_authorities(
            projects,
            portable,
            &self.header.package_identity,
            self.header.artifact_digest,
            resolver,
        )?;
        let modules = phalcom_modules::compiler_artifact::load_module_world_with_project_catalog_and_allocation_budget(
            portable, limits, &mut module_budget, candidate,
        )?;
        allocations.commit_child(module_budget).map_err(ArtifactError::Allocation)?;
        Ok(modules)
    }
}

#[cfg(test)]
mod staging_transaction_tests {
    use super::*;

    #[test]
    fn rejected_container_can_be_retried_without_spending_load_budget() {
        let mut allocations = AllocationBudget::new(1024 * 1024);
        let limits = ValidationLimits::default();
        let invalid = [0_u8; 256];
        let before = allocations.requested();
        for _ in 0..3 {
            assert!(StagedTeloscCompilerWire::decode(&invalid, &limits, &mut allocations).is_err());
            assert_eq!(allocations.requested(), before);
        }
    }

    #[test]
    fn empty_package_set_has_a_deterministic_empty_plan() {
        let mut allocations = AllocationBudget::new(1024);
        let limits = ValidationLimits::default();
        let order = StagedTeloscCompilerWire::package_import_order(&[], &limits, &mut allocations).unwrap();
        assert!(order.is_empty());
    }
}
