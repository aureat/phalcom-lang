//! Complete compiled artifact payload coordinating all four required sections:
//! MODULE_WORLD, SEMANTIC_WORLD, QUERY_IMAGE, and METADATA.

use crate::container::{TeloscContainerBuilder, TeloscContainerView};
use crate::digest::{ArtifactDigest, DOMAIN_METADATA, DOMAIN_MODULE_WORLD, DOMAIN_QUERY_IMAGE, DOMAIN_SEMANTIC_WORLD};
use crate::error::{ArtifactError, ValidationError};
use crate::header::TeloscHeaderV2;
use crate::identity::CanonicalCompatibilityIdentity;
use crate::limits::ValidationLimits;
use crate::metadata::{METADATA_SECTION_SCHEMA_V1, MetadataSectionV1};
use crate::module_world::{MODULE_WORLD_SCHEMA_V1, ModuleWorldV1};
use crate::query::{QUERY_IMAGE_SCHEMA_V1, QueryImageV1};
use crate::section::{SectionFlags, SectionKind};
use crate::semantic::world::{SEMANTIC_WORLD_SCHEMA_V3, SemanticWorldV3};
use phalcom_type_meta::allocation::AllocationBudget;

/// In-memory representation of a complete compiler image artifact payload.
#[derive(Clone, Debug, PartialEq)]
pub struct CompilerArtifactPayload {
    pub header: TeloscHeaderV2,
    pub module_world: ModuleWorldV1,
    pub semantic_world: SemanticWorldV3,
    pub query_image: QueryImageV1,
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
        builder.add_section(SectionKind::ModuleWorld, MODULE_WORLD_SCHEMA_V1, SectionFlags::REQUIRED, module_bytes);
        builder.add_section(SectionKind::SemanticWorld, SEMANTIC_WORLD_SCHEMA_V3, SectionFlags::REQUIRED, semantic_bytes);
        builder.add_section(SectionKind::QueryImage, QUERY_IMAGE_SCHEMA_V1, SectionFlags::REQUIRED, query_bytes);
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
        let view = TeloscContainerView::parse_with_allocation_budget(bytes, limits, budget)?;

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
        let module_world = ModuleWorldV1::decode_with_allocation_budget(module_data, limits, budget)?;
        let semantic_world = SemanticWorldV3::decode_with_allocation_budget(semantic_data, limits, budget)?;
        let query_image = QueryImageV1::decode_with_allocation_budget(query_data, limits, budget)?;
        let metadata = MetadataSectionV1::decode(metadata_data, limits)?;

        let payload = Self {
            header: view.header().clone(),
            module_world,
            semantic_world,
            query_image,
            metadata,
        };

        // Cross-section validation
        payload.validate_cross_section(limits)?;

        Ok(payload)
    }

    /// Cross-section referential integrity and consistency validation.
    pub fn validate_cross_section(&self, _limits: &ValidationLimits) -> Result<(), ArtifactError> {
        // 1. Verify that the metadata package digest matches package identity
        let expected_pkg_digest = self.header.package_identity.digest().0;
        if self.metadata.package_digest != expected_pkg_digest {
            return Err(ValidationError::IncompatibleWorld("metadata package digest mismatch with container header".into()).into());
        }

        // 2. Verify that all declaration references in semantic_world belong to modules
        //    defined in module_world.
        for decl in &self.semantic_world.static_definitions {
            let mod_path = &decl.id.module.path;
            if !self
                .module_world
                .modules
                .iter()
                .any(|m| m.path.len() == mod_path.len() && m.path.iter().zip(mod_path.iter()).all(|(a, b)| a.as_str() == b.as_ref()))
            {
                return Err(ValidationError::ReferentialIntegrity(format!("static definition owner {:?} is not in module world", decl.id)).into());
            }
        }

        // 3. Verify query root keys reference known module products
        for root in &self.query_image.supported_roots {
            match root.key.kind {
                crate::query::PortableQueryKindV1::UnlinkedInterface
                | crate::query::PortableQueryKindV1::ResolvedImports
                | crate::query::PortableQueryKindV1::LinkedInterface
                | crate::query::PortableQueryKindV1::ModuleStaticBindingCatalog
                | crate::query::PortableQueryKindV1::ModuleStaticBindings => {
                    let key_path = root.key.stable_owner_key.split('.').collect::<Vec<_>>();
                    let matches_any = self.module_world.modules.iter().any(|m| {
                        if m.path.is_empty() && root.key.stable_owner_key.is_empty() {
                            return true;
                        }
                        m.path.len() == key_path.len() && m.path.iter().zip(key_path.iter()).all(|(a, b)| a == b)
                    });
                    if !matches_any && !root.key.stable_owner_key.is_empty() {
                        return Err(
                            ValidationError::ReferentialIntegrity(format!("query root references unknown module {}", root.key.stable_owner_key)).into(),
                        );
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
