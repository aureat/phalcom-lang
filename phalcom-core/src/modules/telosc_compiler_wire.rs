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
        Ok(Self { header, module_world, semantic_world, query_image, metadata })
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
        if !matches!(self.header.package_identity, TeloscPackageIdentity::Package { .. }) {
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
