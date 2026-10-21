//! Transactional semantic artifact export, container encoding, validation, and import.
//!
//! (ARTF001.C1.P2)

// The current-schema producer/importer modules below are the production path.
// The compatibility staging importer and its family adapters are compiled
// only into unit tests; production exposes no second semantic reconstruction
// path.
pub mod export;
pub mod import;
pub mod operational;
pub mod query;
pub mod roots;
pub mod validate;

#[cfg(test)]
mod associated_import;
#[cfg(test)]
mod callable_binding_import;
#[cfg(test)]
mod callable_dispatch_import;
#[cfg(test)]
mod callable_evidence_import;
#[cfg(test)]
mod callable_expression_import;
#[cfg(test)]
mod callable_import;
#[cfg(test)]
mod callable_specialization_import;
#[cfg(test)]
mod conformance_evidence_import;
#[cfg(test)]
mod conformance_import;
#[cfg(test)]
mod constructor_import;
#[cfg(test)]
mod denotation_import;
#[cfg(test)]
mod derived_import;
#[cfg(test)]
mod diagnostic_import;
#[cfg(test)]
mod dispatch_import;
#[cfg(test)]
mod enum_requirement_import;
mod identity;
#[cfg(test)]
mod inherent_import;
pub mod manifest;
#[cfg(test)]
mod module_static_import;
#[cfg(test)]
mod query_import;
mod query_policy;
#[cfg(test)]
mod query_product_import;
#[cfg(test)]
mod semantic_import;
#[cfg(test)]
mod static_case_import;
#[cfg(test)]
mod static_constant_import;
#[cfg(test)]
mod static_definition_import;
#[cfg(test)]
mod static_dependency_import;
#[cfg(test)]
mod static_import;
#[cfg(test)]
mod static_program_import;
#[cfg(test)]
mod trait_import;
#[cfg(test)]
mod type_import;
#[cfg(test)]
mod witness_import;

#[cfg(test)]
pub use associated_import::ImportedAssociatedAuthority;
#[cfg(test)]
pub use callable_binding_import::ImportedCallableBindingAuthority;
#[cfg(test)]
pub use callable_dispatch_import::{ImportedCallableConditionalDispatchAuthority, ImportedConditionalCallCapability};
#[cfg(test)]
pub use callable_evidence_import::{ImportedCallableConformanceEvidence, ImportedCallableConformanceEvidenceAuthority};
#[cfg(test)]
pub use callable_expression_import::{ImportedCallableExpressionAuthority, ImportedCallableExpressionBase, ImportedExpressionDeferredAuthority};
#[cfg(test)]
pub use callable_import::ImportedCallableAuthority;
#[cfg(test)]
pub use callable_specialization_import::{ImportedCallSpecialization, ImportedCallableSpecializationAuthority};
#[cfg(test)]
pub use conformance_import::ImportedConformanceAuthority;
#[cfg(test)]
pub use constructor_import::ImportedConstructorInitializationAuthority;
#[cfg(test)]
pub use derived_import::ImportedDerivedConformanceAuthority;
#[cfg(test)]
pub use dispatch_import::ImportedDispatchAuthority;
#[cfg(test)]
pub use enum_requirement_import::ImportedEnumRequirementAuthority;
pub use identity::StableOwnerCatalog;
#[cfg(test)]
pub use inherent_import::ImportedConditionalInherentAuthority;
pub use manifest::{
    ArtifactAuthorityDisposition, COMPILER_ATTACHMENT_MANIFEST, CompilerAttachmentDispositionRow, CompilerAttachmentFamily, IMPORTED_ARTIFACT_FAMILY_MANIFEST,
    ImportedArtifactFamily, ImportedArtifactFamilyDispositionRow, QUERY_KIND_MANIFEST, QueryArtifactDisposition, QueryKindDispositionRow,
    SESSION_BASE_FAMILY_MANIFEST, SNAPSHOT_FIELD_MANIFEST, SessionBaseFamilyManifestRow, SnapshotFieldManifestRow, compiler_attachment_disposition,
    imported_family_disposition, query_kind_disposition, session_base_family_disposition, snapshot_field_disposition,
};
#[cfg(test)]
pub use module_static_import::ImportedModuleStaticBindingAuthority;
pub use query_policy::{
    CanonicalQueryPersistence, canonical_query_persistence, portable_query_kind, query_kind_from_portable, query_kind_profile_support, query_root_is_reusable,
};
pub(crate) use query_policy::{CanonicalQueryRootRequirement, canonical_query_root_requirement};
#[cfg(test)]
pub use query_product_import::ImportedQueryProducts;
#[cfg(test)]
pub use semantic_import::ImportedSemanticAggregates;
#[cfg(test)]
pub use static_case_import::ImportedStaticCaseAuthority;
#[cfg(test)]
pub use static_constant_import::ImportedStaticConstantAuthority;
#[cfg(test)]
pub use static_definition_import::ImportedStaticDefinitionAuthority;
#[cfg(test)]
pub use static_import::ImportedStaticValueGraph;
#[cfg(test)]
pub use static_program_import::{
    ImportedCanonicalCheckedStaticApplication, ImportedCanonicalStaticApplicationArgument, ImportedCanonicalStaticProgram, ImportedOperationalNode,
    ImportedOperationalTarget, ImportedStaticBinaryOp, ImportedStaticUnaryOp,
};
#[cfg(test)]
pub use trait_import::ImportedTraitAuthority;
#[cfg(test)]
pub use type_import::ImportedTypeGraph;
#[cfg(test)]
pub use witness_import::ImportedWitnessPreflightAuthority;

#[cfg(test)]
#[path = "compatibility_tests.rs"]
mod compatibility_tests;

use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_compiler_artifact::header::TeloscHeaderV2;
use phalcom_compiler_artifact::identity::CanonicalCompatibilityIdentity;
use phalcom_compiler_artifact::limits::ValidationLimits;
#[cfg(test)]
use phalcom_compiler_artifact::metadata::MetadataPreflightLimits;
use phalcom_compiler_artifact::metadata::MetadataSectionV1;
use phalcom_compiler_artifact::module_world::ModuleWorldV2;
use phalcom_compiler_artifact::payload::CompilerArtifactPayload;
use phalcom_compiler_artifact::query::QueryImageV4;
use phalcom_compiler_artifact::semantic::world::SemanticWorldV4;
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
#[cfg(test)]
use phalcom_modules::canonical_products::CanonicalExternalModuleWorld;
#[cfg(test)]
use phalcom_modules::compiler_artifact::{load_module_world_with_allocation_budget, load_module_world_with_project_catalog_and_allocation_budget};
#[cfg(test)]
use phalcom_modules::{ProjectUniverse, TrustedPackageResolver, stage_artifact_project_authorities};
#[cfg(test)]
use phalcom_type_meta::SemanticMetadataBundle;
use phalcom_type_meta::allocation::AllocationBudget;
use std::sync::Arc;

/// A completed compiler image ready to be encoded into a `.telosc` container.
#[derive(Clone, Debug, PartialEq)]
pub struct CompilerArtifactImage {
    pub header: TeloscHeaderV2,
    pub module_world: ModuleWorldV2,
    pub semantic_world: SemanticWorldV4,
    pub query_image: QueryImageV4,
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

/// Validation-only compatibility view of staged artifact data.
///
/// `semantic_world` and `query_image` remain portable wire DTOs here. Production
/// publication uses the current native importer to build one `SemanticSnapshot`,
/// one `SemanticDb`, and one `CanonicalUniverseFormalImage`.
#[derive(Clone, Debug)]
#[doc(hidden)]
#[cfg(test)]
pub struct ImportedCompilerImage {
    pub external_world: Arc<CanonicalExternalModuleWorld>,
    /// Project identity graph retained for subsequent semantic import and local overlays.
    pub project_universe: Arc<ProjectUniverse>,
    pub metadata_bundle: SemanticMetadataBundle,
    /// Relocated current-main type graph. The semantic/query DTOs below remain
    /// transport input until the corresponding product import stages complete.
    pub type_graph: Arc<ImportedTypeGraph>,
    /// Relocated current-main static-value graph. Semantic/query DTOs below
    /// remain transport input until their corresponding import stages complete.
    pub static_values: Arc<ImportedStaticValueGraph>,
    /// Certified-P7 checked static constants reconstructed against the same
    /// relocated type/static-value graphs and retained for T4 publication.
    pub static_constant_authority: Arc<ImportedStaticConstantAuthority>,
    /// Certified-P7 source-free static-definition programs staged as typed
    /// operational authority without fabricating current-main AST bodies.
    pub static_definition_authority: Arc<ImportedStaticDefinitionAuthority>,
    /// Reconstructed structural declaration products (data, enums, variants).
    pub semantic_aggregates: Arc<ImportedSemanticAggregates>,
    /// P7 associated semantic authority reconstructed once and shared by
    /// snapshot/session publication and reusable query-product publication.
    pub associated_authority: Arc<ImportedAssociatedAuthority>,
    /// P7 static-case catalogs/signatures reconstructed for snapshot authority.
    /// Their current query input identity remains source-backed, so they are
    /// staged semantically but are not published as unconditional Ready roots.
    pub static_case_authority: Arc<ImportedStaticCaseAuthority>,
    /// Certified-P7 callable signatures reconstructed into current-main live
    /// semantic products. Query Ready publication remains gated on current-main
    /// dependency closure.
    pub callable_authority: Arc<ImportedCallableAuthority>,
    /// Certified-P7 checked callable binding states retained independently of
    /// the still-incomplete expression/proof/flow analysis transplant.
    pub callable_binding_authority: Arc<ImportedCallableBindingAuthority>,
    /// P7 callable expression base facts relocated into current-main identity.
    /// Proof-bearing sidecars remain explicitly deferred per expression until
    /// the separate dispatch/evidence transplant is complete.
    pub callable_expression_authority: Arc<ImportedCallableExpressionAuthority>,
    /// P7 conditional-call proof capability sidecars, including inherent
    /// specialization and conformance provenance.
    pub callable_conditional_dispatch_authority: Arc<ImportedCallableConditionalDispatchAuthority>,
    /// Fully relocated callable-local P7 conformance-evidence graphs, staged
    /// after conformance heads/traits/witnesses for proof-bearing sidecars.
    pub callable_conformance_evidence_authority: Arc<ImportedCallableConformanceEvidenceAuthority>,
    /// P7 generic call specialization sidecars, including relocated static
    /// arguments and conformance-evidence references.
    pub callable_specialization_authority: Arc<ImportedCallableSpecializationAuthority>,
    /// Certified-P7 module-static namespace facts joined to the canonical
    /// imported callable-binding rows without duplicating BindingState.
    pub module_static_binding_authority: Arc<ImportedModuleStaticBindingAuthority>,
    /// Certified-P7 constructor initialization products staged for T4 snapshot
    /// publication. They are not unconditional reusable query roots.
    pub constructor_initialization_authority: Arc<ImportedConstructorInitializationAuthority>,
    /// Certified-P7 declaration dispatch members and field signatures staged
    /// without constructing unbudgeted live HashMap surfaces.
    pub dispatch_authority: Arc<ImportedDispatchAuthority>,
    /// Certified-P7 conditional inherent domains and members, staged without
    /// constructing the live selector HashMap before T4 publication.
    pub conditional_inherent_authority: Arc<ImportedConditionalInherentAuthority>,
    /// Certified-P7 explicit conformance heads, with authorization rechecked
    /// against the relocated current-main project graph.
    pub conformance_authority: Arc<ImportedConformanceAuthority>,
    /// Certified-P7 trait headers, requirements, associated types, and complete
    /// diagnostics staged for the one imported snapshot.
    pub trait_authority: Arc<ImportedTraitAuthority>,
    /// P7 witness plans after portable→live owner/header/requirement-view
    /// preflight. Recursive selection/evidence reconstruction is a separate
    /// proof-bearing import stage and is not implied by this value.
    pub witness_preflight_authority: Arc<ImportedWitnessPreflightAuthority>,
    /// Certified-P7 guarded derived-conformance rules and their relocated proof
    /// graphs, staged for the one imported semantic generation.
    pub derived_conformance_authority: Arc<ImportedDerivedConformanceAuthority>,
    /// Certified-P7 closed-enum requirements and per-case obligation statuses,
    /// staged for T4 snapshot publication. This authority is not independently
    /// promoted to a reusable SemanticDb root.
    pub enum_requirement_authority: Arc<ImportedEnumRequirementAuthority>,
    /// Reconstructed, canonical-fingerprint-validated reusable query products.
    /// T4 installs these into the imported generation's one SemanticDb.
    pub query_products: Arc<ImportedQueryProducts>,
    pub semantic_world: SemanticWorldV4,
    pub query_image: QueryImageV4,
}

#[cfg(test)]
impl ImportedCompilerImage {
    /// Publishes fingerprint-validated imported query products into a fresh
    /// current-main semantic database.
    ///
    /// T3 owns typed product reconstruction and database publication. T4 owns
    /// construction of the surrounding artifact-backed workspace generation;
    /// callers therefore may not merge this authority into a populated DB.
    pub fn publish_query_products_into_empty_db(&self, db: &mut crate::db::SemanticDb) -> Result<(), ArtifactError> {
        self.query_products.publish_into_empty_db(db)
    }
}

/// Transactional staging loader for compiler artifacts.
///
/// Ensures fail-closed semantics: allocations are tracked against a budget,
/// all stages are loaded into private uncommitted fields, and on any error,
/// staging structures are dropped without publishing partial state.
#[cfg(test)]
pub(crate) struct ArtifactImportTransaction<'a> {
    pub parent_budget: &'a mut AllocationBudget,
    pub child_ledger: Option<phalcom_type_meta::allocation::ChildAllocationBudget>,
    pub limits: ValidationLimits,
    pub staged_module_world: Option<Arc<CanonicalExternalModuleWorld>>,
    pub staged_metadata_bundle: Option<SemanticMetadataBundle>,
    pub staged_type_graph: Option<Arc<ImportedTypeGraph>>,
    pub staged_static_values: Option<Arc<ImportedStaticValueGraph>>,
    pub staged_static_constant_authority: Option<Arc<ImportedStaticConstantAuthority>>,
    pub staged_static_definition_authority: Option<Arc<ImportedStaticDefinitionAuthority>>,
    pub staged_semantic_aggregates: Option<Arc<ImportedSemanticAggregates>>,
    pub staged_associated_authority: Option<Arc<ImportedAssociatedAuthority>>,
    pub staged_static_case_authority: Option<Arc<ImportedStaticCaseAuthority>>,
    pub staged_callable_authority: Option<Arc<ImportedCallableAuthority>>,
    pub staged_callable_binding_authority: Option<Arc<ImportedCallableBindingAuthority>>,
    pub staged_callable_expression_authority: Option<Arc<ImportedCallableExpressionAuthority>>,
    pub staged_callable_conditional_dispatch_authority: Option<Arc<ImportedCallableConditionalDispatchAuthority>>,
    pub staged_callable_conformance_evidence_authority: Option<Arc<ImportedCallableConformanceEvidenceAuthority>>,
    pub staged_callable_specialization_authority: Option<Arc<ImportedCallableSpecializationAuthority>>,
    pub staged_module_static_binding_authority: Option<Arc<ImportedModuleStaticBindingAuthority>>,
    pub staged_constructor_initialization_authority: Option<Arc<ImportedConstructorInitializationAuthority>>,
    pub staged_dispatch_authority: Option<Arc<ImportedDispatchAuthority>>,
    pub staged_conditional_inherent_authority: Option<Arc<ImportedConditionalInherentAuthority>>,
    pub staged_conformance_authority: Option<Arc<ImportedConformanceAuthority>>,
    pub staged_trait_authority: Option<Arc<ImportedTraitAuthority>>,
    pub staged_witness_preflight_authority: Option<Arc<ImportedWitnessPreflightAuthority>>,
    pub staged_derived_conformance_authority: Option<Arc<ImportedDerivedConformanceAuthority>>,
    pub staged_enum_requirement_authority: Option<Arc<ImportedEnumRequirementAuthority>>,
    pub staged_query_products: Option<Arc<ImportedQueryProducts>>,
    pub staged_semantic_world: Option<SemanticWorldV4>,
    pub staged_query_image: Option<QueryImageV4>,
    project_catalog: Option<Arc<ProjectUniverse>>,
    trusted_package_resolver: Option<&'a dyn TrustedPackageResolver>,
    committed: bool,
}

#[cfg(test)]
impl<'a> ArtifactImportTransaction<'a> {
    pub fn new(budget: &'a mut AllocationBudget, limits: ValidationLimits) -> Self {
        let child_ledger = budget.child(usize::MAX);
        Self {
            parent_budget: budget,
            child_ledger: Some(child_ledger),
            limits,
            staged_module_world: None,
            staged_metadata_bundle: None,
            staged_type_graph: None,
            staged_static_values: None,
            staged_static_constant_authority: None,
            staged_static_definition_authority: None,
            staged_semantic_aggregates: None,
            staged_associated_authority: None,
            staged_static_case_authority: None,
            staged_callable_authority: None,
            staged_callable_binding_authority: None,
            staged_callable_expression_authority: None,
            staged_callable_conditional_dispatch_authority: None,
            staged_callable_conformance_evidence_authority: None,
            staged_callable_specialization_authority: None,
            staged_module_static_binding_authority: None,
            staged_constructor_initialization_authority: None,
            staged_dispatch_authority: None,
            staged_conditional_inherent_authority: None,
            staged_conformance_authority: None,
            staged_trait_authority: None,
            staged_witness_preflight_authority: None,
            staged_derived_conformance_authority: None,
            staged_enum_requirement_authority: None,
            staged_query_products: None,
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

        // 3. Relocate portable type-graph rows into the current semantic store.
        // The relocation is private until all semantic and query rows are staged.
        let projects = self
            .project_catalog
            .as_deref()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged project catalog".into())))?;
        let owners = identity::StableOwnerCatalog::new(&payload.module_world, projects)?;
        query_import::validate_query_owner_authority(&payload.query_image, &owners)?;
        query_import::validate_structural_root_row_ownership(&payload.query_image, &payload.semantic_world)?;
        let reconstructed = type_import::reconstruct_type_graph(&payload.semantic_world.type_graph, &self.limits, &owners, child)?;
        let static_values = static_import::reconstruct_static_values(&payload.semantic_world.static_values, &reconstructed, &owners, &self.limits, child)?;
        let static_constant_authority =
            static_constant_import::reconstruct_static_constants(&payload.semantic_world.static_constants, &static_values, &reconstructed, &owners, child)?;
        let static_definition_authority = static_definition_import::reconstruct_static_definitions(
            &payload.semantic_world.static_definitions,
            &payload.semantic_world.structural_contracts,
            &static_values,
            &reconstructed,
            &owners,
            &self.limits,
            child,
        )?;
        self.staged_static_values = Some(child.arc(static_values).map_err(ArtifactError::Allocation)?);
        self.staged_static_constant_authority = Some(child.arc(static_constant_authority).map_err(ArtifactError::Allocation)?);
        let static_definition_authority = child.arc(static_definition_authority).map_err(ArtifactError::Allocation)?;
        self.staged_static_definition_authority = Some(static_definition_authority.clone());
        let semantic_aggregates =
            semantic_import::reconstruct_structural_aggregates(&payload.semantic_world.structural_contracts, &reconstructed, &owners, &self.limits, child)?;
        let associated_authority = associated_import::reconstruct_associated_authority(&payload.semantic_world, &reconstructed, &owners, child)?;
        let static_case_authority = static_case_import::reconstruct_static_case_authority(
            &payload.semantic_world.static_case_catalogs,
            &payload.semantic_world.static_case_signatures,
            &reconstructed,
            &owners,
            child,
        )?;
        let callable_authority = callable_import::reconstruct_callable_authority(
            &payload.semantic_world.callable_signatures,
            &payload.semantic_world.callable_definitions,
            &reconstructed,
            &owners,
            child,
        )?;
        let callable_binding_authority = callable_binding_import::reconstruct_callable_bindings(
            &payload.semantic_world.callable_analyses,
            &reconstructed,
            &static_definition_authority,
            &owners,
            child,
        )?;
        let callable_expression_authority = callable_expression_import::reconstruct_callable_expression_bases(
            &payload.semantic_world.callable_analyses,
            &callable_binding_authority,
            &static_definition_authority,
            &reconstructed,
            &owners,
            child,
        )?;
        let callable_conditional_dispatch_authority = callable_dispatch_import::reconstruct_callable_conditional_dispatch(
            &payload.semantic_world.callable_analyses,
            &callable_authority,
            &reconstructed,
            &owners,
            child,
        )?;
        let callable_specialization_authority = callable_specialization_import::reconstruct_callable_specializations(
            &payload.semantic_world.callable_analyses,
            &callable_authority,
            &reconstructed,
            &owners,
            child,
        )?;
        let module_static_binding_authority = module_static_import::reconstruct_module_static_bindings(
            &payload.semantic_world.module_static_bindings,
            &callable_binding_authority,
            &owners,
            child,
        )?;
        let constructor_initialization_authority = constructor_import::reconstruct_constructor_initializations(
            &payload.semantic_world.constructor_initializations,
            &callable_authority,
            &owners,
            child,
        )?;
        let dispatch_authority =
            dispatch_import::reconstruct_dispatch_authority(&payload.semantic_world.dispatch_surfaces, &callable_authority, &reconstructed, &owners, child)?;
        let conditional_inherent_authority =
            inherent_import::reconstruct_conditional_inherent_authority(&payload.semantic_world.conditional_inherent_sets, &reconstructed, &owners, child)?;
        let conformance_authority =
            conformance_import::reconstruct_conformance_authority(&payload.semantic_world.conformance_heads, &reconstructed, &owners, child)?;
        let trait_authority = trait_import::reconstruct_trait_authority(&payload.semantic_world.trait_surfaces, &reconstructed, &owners, child)?;
        let witness_preflight_authority = witness_import::reconstruct_witness_preflight(
            &payload.semantic_world.witness_plans,
            &conformance_authority,
            &trait_authority,
            &callable_authority,
            &reconstructed,
            &owners,
            &self.limits,
            child,
        )?;
        let callable_conformance_evidence_authority = callable_evidence_import::reconstruct_callable_conformance_evidence(
            &payload.semantic_world.callable_analyses,
            &conformance_authority,
            &witness_preflight_authority,
            &trait_authority,
            &callable_authority,
            &reconstructed,
            &owners,
            &self.limits,
            child,
        )?;
        let derived_conformance_authority = derived_import::reconstruct_derived_conformance_authority(
            &payload.semantic_world.derived_conformance_rules,
            witness_preflight_authority.plans(),
            &conformance_authority,
            &trait_authority,
            &callable_authority,
            &reconstructed,
            &owners,
            &self.limits,
            child,
        )?;
        let enum_requirement_authority = enum_requirement_import::reconstruct_enum_requirement_authority(
            &payload.semantic_world.enum_requirements,
            &semantic_aggregates,
            &reconstructed,
            &owners,
            child,
        )?;
        let query_products = query_product_import::reconstruct_structural_query_products(
            &payload.query_image,
            &semantic_aggregates,
            &associated_authority,
            &callable_authority,
            &owners,
            &payload.semantic_world,
            child,
        )?;
        self.staged_semantic_aggregates = Some(child.arc(semantic_aggregates).map_err(ArtifactError::Allocation)?);
        self.staged_associated_authority = Some(child.arc(associated_authority).map_err(ArtifactError::Allocation)?);
        self.staged_static_case_authority = Some(child.arc(static_case_authority).map_err(ArtifactError::Allocation)?);
        self.staged_callable_authority = Some(child.arc(callable_authority).map_err(ArtifactError::Allocation)?);
        self.staged_callable_binding_authority = Some(child.arc(callable_binding_authority).map_err(ArtifactError::Allocation)?);
        self.staged_callable_expression_authority = Some(child.arc(callable_expression_authority).map_err(ArtifactError::Allocation)?);
        self.staged_callable_conditional_dispatch_authority = Some(child.arc(callable_conditional_dispatch_authority).map_err(ArtifactError::Allocation)?);
        self.staged_callable_specialization_authority = Some(child.arc(callable_specialization_authority).map_err(ArtifactError::Allocation)?);
        self.staged_module_static_binding_authority = Some(child.arc(module_static_binding_authority).map_err(ArtifactError::Allocation)?);
        self.staged_constructor_initialization_authority = Some(child.arc(constructor_initialization_authority).map_err(ArtifactError::Allocation)?);
        self.staged_dispatch_authority = Some(child.arc(dispatch_authority).map_err(ArtifactError::Allocation)?);
        self.staged_conditional_inherent_authority = Some(child.arc(conditional_inherent_authority).map_err(ArtifactError::Allocation)?);
        self.staged_conformance_authority = Some(child.arc(conformance_authority).map_err(ArtifactError::Allocation)?);
        self.staged_trait_authority = Some(child.arc(trait_authority).map_err(ArtifactError::Allocation)?);
        self.staged_witness_preflight_authority = Some(child.arc(witness_preflight_authority).map_err(ArtifactError::Allocation)?);
        self.staged_callable_conformance_evidence_authority = Some(child.arc(callable_conformance_evidence_authority).map_err(ArtifactError::Allocation)?);
        self.staged_derived_conformance_authority = Some(child.arc(derived_conformance_authority).map_err(ArtifactError::Allocation)?);
        self.staged_enum_requirement_authority = Some(child.arc(enum_requirement_authority).map_err(ArtifactError::Allocation)?);
        self.staged_query_products = Some(child.arc(query_products).map_err(ArtifactError::Allocation)?);
        self.staged_type_graph = Some(child.arc(ImportedTypeGraph { inner: reconstructed }).map_err(ArtifactError::Allocation)?);

        // 4. Stage remaining semantic world and query wire data.
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
        let type_graph = self
            .staged_type_graph
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged semantic type graph".into())))?;
        let static_values = self
            .staged_static_values
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged semantic static-value graph".into())))?;
        let static_constant_authority = self
            .staged_static_constant_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged static-constant authority".into())))?;
        let static_definition_authority = self
            .staged_static_definition_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged static-definition authority".into())))?;
        let semantic_aggregates = self
            .staged_semantic_aggregates
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged semantic aggregates".into())))?;
        let associated_authority = self
            .staged_associated_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged associated semantic authority".into())))?;
        let static_case_authority = self
            .staged_static_case_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged static-case semantic authority".into())))?;
        let callable_authority = self
            .staged_callable_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged callable semantic authority".into())))?;
        let callable_binding_authority = self
            .staged_callable_binding_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged callable binding authority".into())))?;
        let callable_expression_authority = self
            .staged_callable_expression_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged callable expression authority".into())))?;
        let callable_conditional_dispatch_authority = self.staged_callable_conditional_dispatch_authority.take().ok_or_else(|| {
            ArtifactError::Validation(ValidationError::ReferentialIntegrity(
                "no staged callable conditional-dispatch authority".into(),
            ))
        })?;
        let callable_specialization_authority = self
            .staged_callable_specialization_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged callable specialization authority".into())))?;
        let module_static_binding_authority = self
            .staged_module_static_binding_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged module static-binding authority".into())))?;
        let constructor_initialization_authority = self
            .staged_constructor_initialization_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged constructor-initialization authority".into())))?;
        let dispatch_authority = self
            .staged_dispatch_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged declaration dispatch authority".into())))?;
        let conditional_inherent_authority = self
            .staged_conditional_inherent_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged conditional inherent authority".into())))?;
        let conformance_authority = self
            .staged_conformance_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged conformance-head authority".into())))?;
        let trait_authority = self
            .staged_trait_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged trait semantic authority".into())))?;
        let witness_preflight_authority = self
            .staged_witness_preflight_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged witness-plan preflight authority".into())))?;
        let callable_conformance_evidence_authority = self.staged_callable_conformance_evidence_authority.take().ok_or_else(|| {
            ArtifactError::Validation(ValidationError::ReferentialIntegrity(
                "no staged callable conformance-evidence authority".into(),
            ))
        })?;
        let derived_conformance_authority = self
            .staged_derived_conformance_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged derived-conformance authority".into())))?;
        let enum_requirement_authority = self
            .staged_enum_requirement_authority
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged enum-requirement semantic authority".into())))?;
        let query_products = self
            .staged_query_products
            .take()
            .ok_or_else(|| ArtifactError::Validation(ValidationError::ReferentialIntegrity("no staged semantic query products".into())))?;
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
            type_graph,
            static_values,
            static_constant_authority,
            static_definition_authority,
            semantic_aggregates,
            associated_authority,
            static_case_authority,
            callable_authority,
            callable_binding_authority,
            callable_expression_authority,
            callable_conditional_dispatch_authority,
            callable_specialization_authority,
            module_static_binding_authority,
            constructor_initialization_authority,
            dispatch_authority,
            conditional_inherent_authority,
            conformance_authority,
            trait_authority,
            witness_preflight_authority,
            callable_conformance_evidence_authority,
            derived_conformance_authority,
            enum_requirement_authority,
            query_products,
            semantic_world,
            query_image,
        })
    }
}

#[cfg(test)]
impl Drop for ArtifactImportTransaction<'_> {
    fn drop(&mut self) {
        if !self.committed {
            self.child_ledger = None;
            self.staged_module_world = None;
            self.staged_metadata_bundle = None;
            self.staged_type_graph = None;
            self.staged_static_values = None;
            self.staged_static_constant_authority = None;
            self.staged_static_definition_authority = None;
            self.staged_semantic_aggregates = None;
            self.staged_associated_authority = None;
            self.staged_static_case_authority = None;
            self.staged_callable_authority = None;
            self.staged_callable_binding_authority = None;
            self.staged_callable_expression_authority = None;
            self.staged_callable_conditional_dispatch_authority = None;
            self.staged_callable_specialization_authority = None;
            self.staged_module_static_binding_authority = None;
            self.staged_constructor_initialization_authority = None;
            self.staged_dispatch_authority = None;
            self.staged_conditional_inherent_authority = None;
            self.staged_conformance_authority = None;
            self.staged_trait_authority = None;
            self.staged_witness_preflight_authority = None;
            self.staged_callable_conformance_evidence_authority = None;
            self.staged_derived_conformance_authority = None;
            self.staged_enum_requirement_authority = None;
            self.staged_query_products = None;
            self.staged_semantic_world = None;
            self.staged_query_image = None;
            self.project_catalog = None;
        }
    }
}

/// Helper converting general validation limits into metadata preflight limits.
#[cfg(test)]
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

/// Stages a compiler artifact into the validation-only compatibility view.
///
/// Production semantic/query publication is owned by the current native
/// importer and `CanonicalUniverseFormalImage` loader.
#[doc(hidden)]
#[cfg(test)]
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
/// trusted package resolver. The returned compatibility view retains the
/// resulting graph but is not a production semantic publication authority.
#[doc(hidden)]
#[cfg(test)]
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
