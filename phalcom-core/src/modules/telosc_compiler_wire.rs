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
use phalcom_modules::{CanonicalExternalModuleWorld, ExternalModuleAuthority, ExternalModuleAuthoritySet, ModuleId, ProjectUniverse, TrustedPackageResolver};
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_semantic::artifact::import::{
    validate_and_import_semantic_world_into_shared_stores_with_module_world_and_project_universe_and_allocation_budget,
    validate_and_import_semantic_world_into_type_store_with_module_world_and_project_universe_and_allocation_budget,
    validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget,
};
use phalcom_semantic::artifact::query::reconstruct_query_image_v4_with_project_catalog_and_allocation_budget;
use phalcom_semantic::canonical_image::{
    CanonicalDependencyFingerprints, CanonicalSemanticQueryImage, CanonicalSemanticWorldId,
    CanonicalUniverseFormalImage, ExternalSemanticQueryCatalogue, ExternalSemanticQueryGeneration,
};
use phalcom_semantic::{ExternalSemanticSessionSeed, SemanticWorkspaceSession};
use phalcom_semantic::snapshot::SemanticSnapshot;
use phalcom_semantic::static_value::StaticValueStore;
use phalcom_semantic::types::store::TypeStore;
use std::sync::Arc;

/// A single privately imported package generation. This is deliberately not
/// a workspace-publishable external base on its own. It may have been imported
/// into shared receiving stores, but only the composed-base publication barrier
/// proves all package generations, module owners and query images compatible
/// with the final native generation.
pub(crate) struct StagedArtifactSemanticGeneration {
    pub world: CanonicalSemanticWorldId,
    pub modules: Arc<CanonicalExternalModuleWorld>,
    pub snapshot: Arc<SemanticSnapshot>,
    pub queries: Arc<CanonicalSemanticQueryImage>,
    pub fingerprints: Arc<CanonicalDependencyFingerprints>,
}

/// Fully validated immutable package-artifact base prior to workspace
/// publication. Construction is all-or-nothing: project receipts, module
/// authority, type/static generations, semantic snapshots and every retained
/// query image have all been checked against the final shared generation.
pub(crate) struct ComposedArtifactSemanticBase {
    projects: Arc<ProjectUniverse>,
    modules: Arc<ExternalModuleAuthoritySet>,
    type_store: Arc<TypeStore>,
    static_values: Arc<StaticValueStore>,
    query_catalogue: Arc<ExternalSemanticQueryCatalogue>,
    generations: Vec<StagedArtifactSemanticGeneration>,
}

impl ComposedArtifactSemanticBase {
    pub(crate) fn projects(&self) -> &Arc<ProjectUniverse> { &self.projects }
    pub(crate) fn modules(&self) -> &Arc<ExternalModuleAuthoritySet> { &self.modules }
    pub(crate) fn type_store(&self) -> &Arc<TypeStore> { &self.type_store }
    pub(crate) fn static_values(&self) -> &Arc<StaticValueStore> { &self.static_values }
    pub(crate) fn query_catalogue(&self) -> &Arc<ExternalSemanticQueryCatalogue> { &self.query_catalogue }

    /// Constructs a fresh mutable semantic database over this immutable
    /// composed package authority. External products remain immutable leaves;
    /// only local workspace queries enter mutable database state.
    pub(crate) fn semantic_db(&self, workspace: phalcom_semantic::identity::WorkspaceId) -> phalcom_semantic::db::SemanticDb {
        phalcom_semantic::db::SemanticDb::with_external_query_catalogue(
            workspace,
            Arc::clone(&self.query_catalogue),
        )
    }

    pub(crate) fn generations(&self) -> &[StagedArtifactSemanticGeneration] { &self.generations }

    /// Universe semantic snapshot retained inside the same composed authority.
    /// Production composition guarantees exactly one Universe-owned generation.
    pub(crate) fn universe_snapshot(&self) -> Option<&Arc<SemanticSnapshot>> {
        self.generations
            .iter()
            .find(|generation| generation.modules.owns_project(phalcom_modules::ProjectIdentity::Universe))
            .map(|generation| &generation.snapshot)
    }

    /// Creates a mutable workspace whose immutable compiler authority is this
    /// entire composed package base. Artifact snapshots retain independent
    /// query provenance; the semantic seed is an exact owner-composed view.
    pub(crate) fn semantic_session(
        &self,
        workspace: phalcom_semantic::identity::WorkspaceId,
    ) -> Result<SemanticWorkspaceSession, phalcom_semantic::canonical_image::CanonicalImageError> {
        let seed = ExternalSemanticSessionSeed::from_validated_authorities(
            Arc::clone(&self.modules),
            Arc::clone(&self.query_catalogue),
            Arc::clone(&self.type_store),
            Arc::clone(&self.static_values),
        )?;
        Ok(SemanticWorkspaceSession::from_external_artifact_base(workspace, Arc::new(seed)))
    }

    /// Exact semantic generation owning one externally provided module.
    pub(crate) fn generation_for_module(&self, module: &ModuleId) -> Option<&StagedArtifactSemanticGeneration> {
        let origin = self.modules.origin(module)?;
        self.generations.get(origin.index())
    }
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

    /// Dependency-orders package artifacts and composes them over the loaded
    /// precompiled Universe as one immutable native semantic authority.
    ///
    /// Universe is generation zero of the same module/query/store authority;
    /// there is no parallel canonical-image lookup and no Universe source
    /// fallback once this base is published.
    pub(crate) fn compose_artifact_semantic_base(
        artifacts: Vec<Self>,
        universe: Arc<CanonicalUniverseFormalImage>,
        resolver: &dyn TrustedPackageResolver,
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<ComposedArtifactSemanticBase, TeloscStagingError> {
        Self::compose_artifact_semantic_base_impl(
            artifacts,
            universe.modules().projects().as_ref(),
            Some(universe),
            resolver,
            limits,
            allocations,
        )
    }

    #[cfg(test)]
    fn compose_package_semantic_base_without_universe_for_test(
        artifacts: Vec<Self>,
        projects: &ProjectUniverse,
        resolver: &dyn TrustedPackageResolver,
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<ComposedArtifactSemanticBase, TeloscStagingError> {
        Self::compose_artifact_semantic_base_impl(
            artifacts,
            projects,
            None,
            resolver,
            limits,
            allocations,
        )
    }

    /// Internal composition transaction shared by the production Universe-rooted
    /// path and the empty-base unit discriminator. Production callers must pass
    /// a Universe image.
    fn compose_artifact_semantic_base_impl(
        artifacts: Vec<Self>,
        projects: &ProjectUniverse,
        universe: Option<Arc<CanonicalUniverseFormalImage>>,
        resolver: &dyn TrustedPackageResolver,
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<ComposedArtifactSemanticBase, TeloscStagingError> {
        let mut composition_budget = allocations.child(allocations.remaining());
        let order = Self::package_import_order(&artifacts, limits, &mut composition_budget)?;

        let artifact_count = artifacts.len();
        let generation_count = artifact_count
            .checked_add(usize::from(universe.is_some()))
            .ok_or_else(|| ArtifactError::from(ValidationError::ReferentialIntegrity(
                "artifact generation count overflow".into(),
            )))?;
        let mut pending = composition_budget
            .vector::<Option<Self>>(artifact_count)
            .map_err(ArtifactError::Allocation)?;
        for artifact in artifacts {
            pending.push(Some(artifact));
        }

        let mut project_candidate = projects.clone();
        let mut receiving_types = universe
            .as_ref()
            .map_or_else(TypeStore::new, |image| image.snapshot().store.as_ref().clone());
        let mut receiving_static_values = universe
            .as_ref()
            .map_or_else(StaticValueStore::new, |image| image.snapshot().static_values.as_ref().clone());
        let mut generations = composition_budget
            .vector::<StagedArtifactSemanticGeneration>(generation_count)
            .map_err(ArtifactError::Allocation)?;
        let mut worlds = composition_budget
            .vector::<Arc<CanonicalExternalModuleWorld>>(generation_count)
            .map_err(ArtifactError::Allocation)?;

        if let Some(universe) = universe {
            if !project_candidate.preserves_authority_of(universe.modules().projects())
                || !universe.modules().projects().preserves_authority_of(&project_candidate)
            {
                return Err(ArtifactError::from(ValidationError::ReferentialIntegrity(
                    "loaded Universe artifact project authority does not match composition base".into(),
                )).into());
            }
            let fingerprints = universe
                .dependency_fingerprints()
                .clone_with_allocation_budget(&mut composition_budget)?;
            let fingerprints = composition_budget
                .arc(fingerprints)
                .map_err(ArtifactError::Allocation)?;
            worlds.push(Arc::clone(universe.modules()));
            generations.push(StagedArtifactSemanticGeneration {
                world: universe.world().clone(),
                modules: Arc::clone(universe.modules()),
                snapshot: Arc::clone(universe.snapshot()),
                queries: Arc::clone(universe.query_image()),
                fingerprints,
            });
        }

        for index in order {
            let artifact = pending
                .get_mut(index)
                .and_then(Option::take)
                .ok_or_else(|| ArtifactError::from(ValidationError::ReferentialIntegrity(
                    "package artifact dependency plan selected an invalid or repeated slot".into(),
                )))?;
            let generation = artifact.stage_package_semantic_generation_into_shared_stores(
                &project_candidate,
                resolver,
                &mut receiving_types,
                &mut receiving_static_values,
                limits,
                &mut composition_budget,
            )?;
            let next_projects = generation.modules.linked().universe.as_ref().clone();
            if !next_projects.preserves_authority_of(&project_candidate) {
                return Err(ArtifactError::from(ValidationError::ReferentialIntegrity(
                    "package artifact import replaced previously validated project authority".into(),
                )).into());
            }
            project_candidate = next_projects;
            worlds.push(Arc::clone(&generation.modules));
            generations.push(generation);
        }

        if pending.iter().any(Option::is_some) {
            return Err(ArtifactError::from(ValidationError::ReferentialIntegrity(
                "package artifact dependency plan did not consume every artifact".into(),
            )).into());
        }

        let final_projects = composition_budget
            .arc(project_candidate)
            .map_err(ArtifactError::Allocation)?;
        let module_set = ExternalModuleAuthoritySet::from_worlds(Arc::clone(&final_projects), worlds)
            .map_err(|error| ArtifactError::from(ValidationError::ReferentialIntegrity(error)))?;
        let modules = composition_budget.arc(module_set).map_err(ArtifactError::Allocation)?;
        let type_store = composition_budget.arc(receiving_types).map_err(ArtifactError::Allocation)?;
        let static_values = composition_budget.arc(receiving_static_values).map_err(ArtifactError::Allocation)?;

        // Every earlier immutable snapshot is a strict prefix of the final
        // shared stores. Revalidate all retained query images against that
        // final lineage before publishing any composed authority.
        for generation in &generations {
            generation.queries.reader(
                &generation.world,
                type_store.as_ref(),
                static_values.as_ref(),
            )?;
        }

        // Query ownership is exact and singular just like module ownership.
        // Dependency-only evidence may overlap through the catalogue iff its
        // fingerprints agree; actual query roots may have exactly one owner.
        for left_index in 0..generations.len() {
            for right_index in (left_index + 1)..generations.len() {
                let mut left = generations[left_index].queries.entries().peekable();
                let mut right = generations[right_index].queries.entries().peekable();
                loop {
                    match (left.peek(), right.peek()) {
                        (Some((left_key, _)), Some((right_key, _))) => {
                            match left_key.cmp(right_key) {
                                std::cmp::Ordering::Less => { left.next(); }
                                std::cmp::Ordering::Greater => { right.next(); }
                                std::cmp::Ordering::Equal => {
                                    return Err(ArtifactError::from(ValidationError::ReferentialIntegrity(
                                        "query key is owned by multiple immutable artifacts".into(),
                                    )).into());
                                }
                            }
                        }
                        _ => break,
                    }
                }
            }
        }

        let mut query_generations = composition_budget
            .vector::<ExternalSemanticQueryGeneration>(generations.len())
            .map_err(ArtifactError::Allocation)?;
        for generation in &generations {
            query_generations.push(
                ExternalSemanticQueryGeneration::from_validated_parts_with_allocation_budget(
                    &generation.world,
                    Arc::clone(&generation.snapshot),
                    Arc::clone(&generation.queries),
                    Arc::clone(&generation.fingerprints),
                    type_store.as_ref(),
                    static_values.as_ref(),
                    &mut composition_budget,
                )?,
            );
        }
        let query_catalogue = ExternalSemanticQueryCatalogue::from_generations_with_allocation_budget(
            query_generations,
            &mut composition_budget,
        )?;
        let query_catalogue = composition_budget.arc(query_catalogue).map_err(ArtifactError::Allocation)?;

        let base = ComposedArtifactSemanticBase {
            projects: final_projects,
            modules,
            type_store,
            static_values,
            query_catalogue,
            generations,
        };
        allocations.commit_child(composition_budget).map_err(ArtifactError::Allocation)?;
        Ok(base)
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
    ) -> Result<StagedArtifactSemanticGeneration, TeloscStagingError> {
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
        let fingerprints = generation_budget.arc(fingerprints).map_err(ArtifactError::Allocation)?;
        let staged = StagedArtifactSemanticGeneration {
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
    ) -> Result<StagedArtifactSemanticGeneration, TeloscStagingError> {
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

        let fingerprints = generation_budget.arc(fingerprints).map_err(ArtifactError::Allocation)?;
        let staged = StagedArtifactSemanticGeneration {
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

    /// Completes one package import against shared receiving type and
    /// static-value generations. Both caller-visible stores are replaced only
    /// after module, semantic, query, reader, and allocation validation commit.
    pub(crate) fn stage_package_semantic_generation_into_shared_stores(
        self,
        projects: &ProjectUniverse,
        resolver: &dyn TrustedPackageResolver,
        receiving_types: &mut TypeStore,
        receiving_static_values: &mut StaticValueStore,
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<StagedArtifactSemanticGeneration, TeloscStagingError> {
        use phalcom_compiler_artifact::header::TeloscCapabilities;
        if self.header.capabilities.contains(TeloscCapabilities::PORTABLE_EXECUTABLE) {
            return Err(TeloscStagingError::Capability(
                "ordinary package executable authority is not supported by compiler-only staging",
            ));
        }

        let mut generation_budget = allocations.child(allocations.remaining());
        let mut type_candidate = receiving_types.clone();
        let mut static_candidate = receiving_static_values.clone();
        let module_products = self.stage_package_modules(projects, resolver, limits, &mut generation_budget)?;
        let modules = generation_budget.arc(module_products).map_err(ArtifactError::Allocation)?;
        let parts =
            validate_and_import_semantic_world_into_shared_stores_with_module_world_and_project_universe_and_allocation_budget(
                self.semantic_world,
                &self.module_world.previous.world,
                modules.linked().universe.as_ref(),
                &mut type_candidate,
                &mut static_candidate,
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

        let fingerprints = generation_budget.arc(fingerprints).map_err(ArtifactError::Allocation)?;
        let staged = StagedArtifactSemanticGeneration {
            world,
            modules,
            snapshot: parts.snapshot,
            queries,
            fingerprints,
        };
        allocations.commit_child(generation_budget).map_err(ArtifactError::Allocation)?;
        *receiving_types = type_candidate;
        *receiving_static_values = static_candidate;
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

    struct NoPackageReceipts;

    impl TrustedPackageResolver for NoPackageReceipts {
        fn package_authority_receipt(
            &self,
            _: &phalcom_type_meta::identity::StableProjectRef,
        ) -> Option<phalcom_modules::ValidatedPackageAuthorityReceipt> {
            None
        }
    }

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

    #[test]
    fn empty_package_set_test_discriminator_publishes_closed_empty_shared_generation() {
        let projects = ProjectUniverse::new();
        let limits = ValidationLimits::default();
        let mut allocations = AllocationBudget::new(64 * 1024);
        let base = StagedTeloscCompilerWire::compose_package_semantic_base_without_universe_for_test(
            Vec::new(),
            &projects,
            &NoPackageReceipts,
            &limits,
            &mut allocations,
        )
        .unwrap();

        assert!(base.generations().is_empty());
        assert!(base.projects().preserves_authority_of(&projects));
        assert!(projects.preserves_authority_of(base.projects()));
        assert_eq!(base.type_store().id(), base.type_store().id());
        assert_eq!(base.static_values().id(), base.static_values().id());
        let db = base.semantic_db(phalcom_semantic::identity::WorkspaceId::from_raw(77));
        assert!(db.canonical_image().is_none());
        assert!(db.immutable_query_authority().is_some());
        assert_eq!(base.query_catalogue().generations().len(), 0);

        let session = base
            .semantic_session(phalcom_semantic::identity::WorkspaceId::from_raw(78))
            .expect("empty composed artifact base seeds a workspace");
        assert!(session.last_snapshot().is_none(), "external authority is not a local workspace publication");
        assert!(session.module_session().universe().preserves_authority_of(base.projects()));
    }
}
