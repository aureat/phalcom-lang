//! Complete compiled artifact payload coordinating all four required sections:
//! MODULE_WORLD, SEMANTIC_WORLD, QUERY_IMAGE, and METADATA.

use crate::container::{TeloscContainerBuilder, TeloscContainerView};
use crate::digest::{ArtifactDigest, DOMAIN_METADATA, DOMAIN_MODULE_WORLD, DOMAIN_QUERY_IMAGE, DOMAIN_SEMANTIC_WORLD};
use crate::error::{ArtifactError, ValidationError};
use crate::header::TeloscHeaderV2;
use crate::identity::CanonicalCompatibilityIdentity;
use crate::limits::ValidationLimits;
use crate::metadata::{METADATA_SECTION_SCHEMA_V1, MetadataSectionV1};
use crate::module_world::{MODULE_WORLD_SCHEMA_V2, ModuleWorldV2};
use crate::query::{QUERY_IMAGE_SCHEMA_V4, QueryImageV4};
use crate::section::{SectionFlags, SectionKind};
use crate::semantic::world::{DependencyPhaseV1, RuntimeRequirementTargetV1, SEMANTIC_WORLD_SCHEMA_V4, SemanticWorldV4};
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::identity::{StableModuleRef, StableProjectRef};

/// In-memory representation of a complete compiler image artifact payload.
#[derive(Clone, Debug, PartialEq)]
pub struct CompilerArtifactPayload {
    pub header: TeloscHeaderV2,
    pub module_world: ModuleWorldV2,
    pub semantic_world: SemanticWorldV4,
    pub query_image: QueryImageV4,
    pub metadata: MetadataSectionV1,
}

impl CompilerArtifactPayload {
    /// Serializes and packages the complete `.telosc` binary container.
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        let module_bytes = self.module_world.encode()?;
        let semantic_bytes = self.semantic_world.encode()?;
        let query_bytes = self.query_image.encode()?;
        let metadata_bytes = self.metadata.encode()?;

        let mut header = self.header.clone();
        header.interface_digest = ArtifactDigest::digest_domain(DOMAIN_MODULE_WORLD, &module_bytes);
        header.semantic_digest = ArtifactDigest::digest_domain(DOMAIN_SEMANTIC_WORLD, &semantic_bytes);
        header.query_digest = ArtifactDigest::digest_domain(DOMAIN_QUERY_IMAGE, &query_bytes);
        header.metadata_digest = ArtifactDigest::digest_domain(DOMAIN_METADATA, &metadata_bytes);

        let mut builder = TeloscContainerBuilder::new(header);
        builder.add_section(SectionKind::ModuleWorld, MODULE_WORLD_SCHEMA_V2, SectionFlags::REQUIRED, module_bytes);
        builder.add_section(SectionKind::SemanticWorld, SEMANTIC_WORLD_SCHEMA_V4, SectionFlags::REQUIRED, semantic_bytes);
        builder.add_section(SectionKind::QueryImage, QUERY_IMAGE_SCHEMA_V4, SectionFlags::REQUIRED, query_bytes);
        builder.add_section(SectionKind::Metadata, METADATA_SECTION_SCHEMA_V1, SectionFlags::REQUIRED, metadata_bytes);

        builder.build()
    }

    /// Decodes, validates integrity, verifies section schemas and compatibility,
    /// and performs cross-section referential integrity checks under a transaction budget.
    pub fn decode_and_validate(
        bytes: &[u8],
        limits: &ValidationLimits,
        expected_identity: Option<&CanonicalCompatibilityIdentity>,
        budget: &mut AllocationBudget,
    ) -> Result<Self, ArtifactError> {
        let mut load_budget = budget.clone();
        let view = TeloscContainerView::parse_with_allocation_budget(bytes, limits, &mut load_budget)?;

        // Compatibility checks
        if let Some(expected) = expected_identity {
            let actual_identity = CanonicalCompatibilityIdentity {
                package: view.header().package_identity.clone(),
                source_digest: view.header().source_digest,
                interface_digest: view.header().interface_digest,
                semantic_digest: view.header().semantic_digest,
                semantic_model_version: view.header().semantic_model_version,
                stable_identity_schema: view.header().stable_identity_schema,
                module_semantic_abi: view.header().module_semantic_abi,
                semantic_artifact_abi: view.header().semantic_artifact_abi,
                query_fingerprint_abi: view.header().query_fingerprint_abi,
                native_surface_schema_version: view.header().native_surface_schema_version,
                native_catalog_fingerprint: view.header().native_catalog_fingerprint,
                type_metadata_schema_version: view.header().type_metadata_schema_version,
            };
            actual_identity.check_compatible(expected)?;
        }

        // Section lookups
        let module_data = view.require_section(SectionKind::ModuleWorld)?;
        let semantic_data = view.require_section(SectionKind::SemanticWorld)?;
        let query_data = view.require_section(SectionKind::QueryImage)?;
        let metadata_data = view.require_section(SectionKind::Metadata)?;

        // Decodes under budget
        let module_world = ModuleWorldV2::decode_with_allocation_budget(module_data, limits, &mut load_budget)?;
        let semantic_world = SemanticWorldV4::decode_with_allocation_budget(semantic_data, limits, &mut load_budget)?;
        let query_image = QueryImageV4::decode_with_allocation_budget(query_data, limits, &mut load_budget)?;
        let metadata = MetadataSectionV1::decode_with_allocation_budget(metadata_data, limits, &mut load_budget)?;

        let payload = Self {
            header: view
                .header()
                .clone_with_allocation_budget(&mut load_budget)
                .map_err(ArtifactError::Allocation)?,
            module_world,
            semantic_world,
            query_image,
            metadata,
        };

        // Cross-section validation
        payload.validate_cross_section(limits)?;

        *budget = load_budget;
        Ok(payload)
    }

    /// Cross-section referential integrity and consistency validation.
    pub fn validate_cross_section(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        self.query_image.validate(limits)?;

        // 1. Verify that the metadata package digest matches package identity
        let expected_pkg_digest = self.header.package_identity.digest().0;
        if self.metadata.package_digest != expected_pkg_digest {
            return Err(ValidationError::IncompatibleWorld("metadata package digest mismatch with container header".into()).into());
        }

        let validate_project = |proj: &StableProjectRef| -> Result<(), ArtifactError> {
            if matches!(proj, StableProjectRef::Session { .. }) {
                return Err(ValidationError::ReferentialIntegrity("session project owner is forbidden".into()).into());
            }
            if self.module_world.modules.iter().any(|module| &module.identity.project == proj) {
                return Ok(());
            }
            if self.module_world.external_projects.contains(proj) {
                return Ok(());
            }
            Err(ValidationError::ReferentialIntegrity(format!("unlisted external project owner: {proj:?}")).into())
        };

        let validate_module = |module: &StableModuleRef| -> Result<(), ArtifactError> {
            validate_project(&module.project)?;
            if self.module_world.modules.iter().any(|row| row.identity.project == module.project) {
                if !self.module_world.modules.iter().any(|m| m.identity == *module) {
                    return Err(ValidationError::ReferentialIntegrity(format!("missing internal module owner: {module:?}")).into());
                }
            }
            Ok(())
        };

        // 2. Verify all module rows in module_world have valid project ownership
        for m in &self.module_world.modules {
            validate_project(&m.identity.project)?;
        }

        // 2a. Module execution authority is keyed by the source-import rows in
        // MODULE_WORLD. These ordinals are dense and module-relative; they are
        // not executable read slots.
        let execution = &self.semantic_world.module_execution_authority;
        if execution.import_phase_products.len() != self.module_world.modules.len() {
            return Err(ValidationError::ReferentialIntegrity("module execution authority must contain one phase product for every module".into()).into());
        }
        for module in &self.module_world.modules {
            let product = execution
                .import_phase_products
                .iter()
                .find(|product| product.module == module.identity)
                .ok_or_else(|| {
                    ArtifactError::Validation(ValidationError::ReferentialIntegrity(format!(
                        "missing module import phase product for {:?}",
                        module.identity
                    )))
                })?;
            let expected_binding_count = self
                .module_world
                .source_import_binding_count(&module.identity)?
                .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("module disappeared from module world".into())))?;
            if u32::try_from(product.bindings.len()).ok() != Some(expected_binding_count) {
                return Err(ValidationError::ReferentialIntegrity(format!(
                    "module import phase binding count disagrees with source import rows for {:?}",
                    module.identity
                ))
                .into());
            }
            for (ordinal, binding) in product.bindings.iter().enumerate() {
                if usize::try_from(binding.binding).ok() != Some(ordinal) {
                    return Err(ValidationError::ReferentialIntegrity(format!(
                        "module import phase bindings are not the dense source ordinal range for {:?}",
                        module.identity
                    ))
                    .into());
                }
            }
        }

        for analysis in &self.semantic_world.callable_analyses {
            let callable_module = match &analysis.callable.owner {
                phalcom_type_meta::identity::StableCallableOwnerRef::Declaration(owner) => &owner.module,
                phalcom_type_meta::identity::StableCallableOwnerRef::Variant(owner) => &owner.owner.module,
                phalcom_type_meta::identity::StableCallableOwnerRef::Conformance(owner) => &owner.module,
            };
            validate_module(callable_module)?;
            let product = execution
                .import_phase_products
                .iter()
                .find(|product| product.module == *callable_module)
                .ok_or_else(|| {
                    ArtifactError::Validation(ValidationError::ReferentialIntegrity(format!(
                        "callable phase evidence has no module product for {:?}",
                        callable_module
                    )))
                })?;
            for observed in &analysis.observed_import_phases {
                let summary = product
                    .bindings
                    .get(
                        usize::try_from(observed.binding)
                            .map_err(|_| ArtifactError::Validation(ValidationError::ReferentialIntegrity("callable import binding ordinal overflow".into())))?,
                    )
                    .filter(|summary| summary.binding == observed.binding)
                    .ok_or_else(|| {
                        ArtifactError::Validation(ValidationError::ReferentialIntegrity(format!(
                            "callable import phase references absent source binding {} in {:?}",
                            observed.binding, callable_module
                        )))
                    })?;
                if !summary.summary.observed {
                    return Err(ValidationError::ReferentialIntegrity(format!(
                        "callable import phase references an unobserved module binding {} in {:?}",
                        observed.binding, callable_module
                    ))
                    .into());
                }
                if observed.phase == DependencyPhaseV1::Runtime && summary.summary.phase != DependencyPhaseV1::Runtime {
                    return Err(ValidationError::ReferentialIntegrity(format!(
                        "module import phase does not cover callable runtime observation {} in {:?}",
                        observed.binding, callable_module
                    ))
                    .into());
                }
            }
        }

        for requirement in &execution.runtime_requirements {
            validate_module(&requirement.importer)?;
            validate_module(&requirement.dependency)?;
            validate_module(&requirement.provenance.consumer)?;
            match &requirement.provenance.target {
                RuntimeRequirementTargetV1::Module { module } | RuntimeRequirementTargetV1::Binding { module, .. } => validate_module(module)?,
            }
            if let Some(target) = &requirement.read_target {
                match target {
                    crate::semantic::world::LinkedReadSpecV1::Module { module } => validate_module(module)?,
                    crate::semantic::world::LinkedReadSpecV1::Binding { symbol } => validate_module(&symbol.module)?,
                }
            }
        }

        // 3. Verify static definitions and static constants in semantic_world
        for decl in &self.semantic_world.static_definitions {
            validate_module(&decl.id.module)?;
        }
        for constant in &self.semantic_world.static_constants {
            validate_module(&constant.symbol.module)?;
        }

        // 4. Verify all query root owners and dependency evidence owners
        for root in &self.query_image.supported_roots {
            validate_project(root.key.owner.project())?;
            if let Some(m) = root.key.owner.module() {
                validate_module(m)?;
            }
        }
        for ev in &self.query_image.dependency_evidence {
            validate_project(ev.key.owner.project())?;
            validate_project(ev.stable_aggregate_owner.project())?;
            if let Some(m) = ev.key.owner.module() {
                validate_module(m)?;
            }
            if let Some(m) = ev.stable_aggregate_owner.module() {
                validate_module(m)?;
            }
        }

        // 5. Verify product references and portable QUERY_FINGERPRINT_ABI_V3 evidence.
        let modules = crate::module_world_v5::ModuleWorldV5::from(self.module_world.clone());
        self.query_image.validate_product_references(&modules, &self.semantic_world)?;
        self.query_image.validate_portable_structural_fingerprints_v4(&self.semantic_world)?;

        Ok(())
    }
}
