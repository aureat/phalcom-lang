use phalcom_compiler_artifact::container::{TeloscContainerBuilder, TeloscContainerView};
use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_compiler_artifact::error::{ArtifactError, CodecError, ContainerError, ValidationError};
use phalcom_compiler_artifact::header::{TELOSC_CONTAINER_SCHEMA_V2, TELOSC_MIN_READER_SCHEMA_V2, TeloscCapabilities, TeloscHeaderV2};
use phalcom_compiler_artifact::identity::{CanonicalCompatibilityIdentity, TeloscPackageIdentity};
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::metadata::{MetadataPreflightLimits, MetadataSectionV1};
use phalcom_compiler_artifact::module_world::{ArtifactModuleId, MODULE_WORLD_SCHEMA_V1, ModuleKindV1, ModuleV1, ModuleWorldV1};
use phalcom_compiler_artifact::payload::CompilerArtifactPayload;
use phalcom_compiler_artifact::query::{PortableQueryKeyV1, PortableQueryKindV1, QueryImageV1, QueryRootEntryV1};
use phalcom_compiler_artifact::section::{SectionFlags, SectionKind};
use phalcom_compiler_artifact::semantic::aggregates::StructuralPrimaryContractsV1;
use phalcom_compiler_artifact::semantic::static_value::{StaticValueGraphV1, StaticValueV1};
use phalcom_compiler_artifact::semantic::type_store::{ArtifactKindId, KindV1, TypeEntryV1, TypeGraphV1, TypeV1};
use phalcom_compiler_artifact::semantic::world::{SEMANTIC_WORLD_SCHEMA_V3, SemanticWorldV3};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::{
    ArtifactIdentityScheme, KindNode, KindNodeEntry, KindNodeId, MetadataFeatures, MetadataProfile, NATIVE_SURFACE_SCHEMA_VERSION, ProducerIdentity,
    SEMANTIC_MODEL_VERSION, SemanticMetadataBundle, SemanticMetadataHeader, TYPE_METADATA_SCHEMA_VERSION, TypeNode, TypeNodeEntry,
};

fn sample_bundle() -> SemanticMetadataBundle {
    SemanticMetadataBundle {
        header: SemanticMetadataHeader {
            schema_version: TYPE_METADATA_SCHEMA_VERSION,
            semantic_model_version: SEMANTIC_MODEL_VERSION,
            producer: ProducerIdentity("test".into()),
            producer_version: "1.0.0".into(),
            native_surface_schema_version: NATIVE_SURFACE_SCHEMA_VERSION,
            profile: MetadataProfile::RuntimePublic,
            features: MetadataFeatures::default(),
            identity_scheme: ArtifactIdentityScheme::V1Standard,
            source_fingerprint: Fingerprint128::ZERO,
            interface_fingerprint: Fingerprint128::ZERO,
        },
        kinds: Box::new([KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        }]),
        types: Box::new([TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        }]),
        scoped_types: Box::new([]),
        parameters: Box::new([]),
        generic_signatures: Box::new([]),
        declarations: Box::new([]),
        static_domains: Box::new([]),
        aliases: Box::new([]),
        callables: Box::new([]),
        fields: Box::new([]),
        static_definitions: Box::new([]),
        static_constants: Box::new([]),
        static_values: Box::new([]),
        module_roots: Box::new([]),
        runtime_roots: Box::new([]),
        occurrences: Box::new([]),
        extensions: Box::new([]),
    }
}

fn sample_metadata(pkg_identity: &TeloscPackageIdentity) -> MetadataSectionV1 {
    let bundle = sample_bundle();
    let limits = MetadataPreflightLimits {
        max_metadata_bytes: 65536,
        max_token_count: 10000,
        max_container_count: 10000,
        max_nesting_depth: 32,
    };
    MetadataSectionV1::from_bundle(&bundle, pkg_identity.digest().0, &limits, &phalcom_type_meta::ValidationLimits::default()).unwrap()
}

fn sample_payload() -> CompilerArtifactPayload {
    let pkg_id = TeloscPackageIdentity::universe();
    let header = TeloscHeaderV2 {
        container_schema: TELOSC_CONTAINER_SCHEMA_V2,
        minimum_reader_schema: TELOSC_MIN_READER_SCHEMA_V2,
        package_identity: pkg_id.clone(),
        semantic_model_version: 1,
        stable_identity_schema: 1,
        module_semantic_abi: 1,
        semantic_artifact_abi: 1,
        query_fingerprint_abi: 1,
        native_surface_schema_version: 1,
        native_catalog_fingerprint: [1; 16],
        type_metadata_schema_version: 13,
        capabilities: TeloscCapabilities::CANONICAL_COMPILER_IMAGE_V1 | TeloscCapabilities::RUNTIME_PUBLIC_METADATA,
        source_digest: ArtifactDigest::ZERO,
        interface_digest: ArtifactDigest::ZERO,
        semantic_digest: ArtifactDigest::ZERO,
        query_digest: ArtifactDigest::ZERO,
        metadata_digest: ArtifactDigest::ZERO,
        artifact_digest: ArtifactDigest::ZERO,
        section_count: 4,
    };

    let root_module = ModuleV1 {
        path: vec![],
        kind: ModuleKindV1::Package,
        declarations: vec![],
        exports: vec![],
        imports: vec![],
        exposed_children: vec![],
        metadata: vec![],
        linked_exports: vec![],
        globals: vec![],
        import_bindings: vec![],
        linked_reads: vec![],
        runtime_dependencies: vec![],
        references: vec![],
        runtime_edges: vec![],
        topology_parent: None,
        topology_children: vec![],
        interface_fingerprint: 0,
        linked_interface_fingerprint: 0,
        dependency_fingerprint: 0,
    };

    let module_world = ModuleWorldV1 {
        entry: ArtifactModuleId(0),
        initialization_order: vec![ArtifactModuleId(0)],
        modules: vec![root_module],
        semantic_rows: vec![],
        topology_fingerprint: 0,
    };

    let semantic_world = SemanticWorldV3 {
        type_graph: TypeGraphV1 {
            types: vec![TypeEntryV1 {
                kind: ArtifactKindId(0),
                form: TypeV1::Unit,
            }],
            kinds: vec![KindV1::Type],
            parameters: vec![],
            current_parameters: vec![],
            rows: vec![],
            families: vec![],
            variants: vec![],
            static_domains: vec![],
            static_constructors: vec![],
            scoped: vec![],
            lambdas: vec![],
            variance_overrides: vec![],
        },
        structural_contracts: StructuralPrimaryContractsV1 {
            declaration_shells: vec![],
            hierarchy_and_aliases: vec![],
            generic_signatures: vec![],
            data_contracts: vec![],
            enum_contracts: vec![],
            variant_contracts: vec![],
        },
        enum_requirements: vec![],
        static_values: StaticValueGraphV1 {
            values: vec![StaticValueV1::Unit],
            strings: vec![],
            symbols: vec![],
            tuples: vec![],
            sequences: vec![],
            records: vec![],
            data_values: vec![],
            variant_values: vec![],
            pack_backings: vec![],
            packs: vec![],
        },
        static_definitions: vec![],
        static_constants: vec![],
        module_static_bindings: vec![],
        witness_plans: vec![],
        derived_conformance_rules: vec![],
        callable_signatures: vec![],
        callable_definitions: vec![],
        callable_analyses: vec![],
        trait_surfaces: vec![],
        dispatch_surfaces: vec![],
        conditional_inherent_sets: vec![],
        conformance_heads: vec![],
        constructor_initializations: vec![],
        type_member_declarations: vec![],
        associated_static_domains: vec![],
        static_case_catalogs: vec![],
        static_case_signatures: vec![],
        associated_surfaces: vec![],
    };

    let query_image = QueryImageV1 {
        supported_roots: vec![],
        dependency_evidence: vec![],
    };

    CompilerArtifactPayload {
        header,
        module_world,
        semantic_world,
        query_image,
        metadata: sample_metadata(&pkg_id),
    }
}

#[test]
fn missing_required_section_rejected() {
    let payload = sample_payload();
    let mut header = payload.header.clone();
    header.capabilities = TeloscCapabilities::CANONICAL_COMPILER_IMAGE_V1 | TeloscCapabilities::RUNTIME_PUBLIC_METADATA;

    // Build container missing MODULE_WORLD
    let mut builder = TeloscContainerBuilder::new(header);
    builder.add_section(
        SectionKind::SemanticWorld,
        SEMANTIC_WORLD_SCHEMA_V3,
        SectionFlags::REQUIRED,
        payload.semantic_world.encode().unwrap(),
    );
    let bytes = builder.build().unwrap();

    let limits = ValidationLimits::default();
    let err = TeloscContainerView::parse(&bytes, &limits).unwrap_err();
    assert!(matches!(
        err,
        ArtifactError::Container(ContainerError::MissingRequiredSection(SectionKind::ModuleWorld))
    ));
}

#[test]
fn duplicate_required_section_rejected() {
    let payload = sample_payload();
    let mut builder = TeloscContainerBuilder::new(payload.header.clone());
    let mw = payload.module_world.encode().unwrap();
    builder.add_section(SectionKind::ModuleWorld, MODULE_WORLD_SCHEMA_V1, SectionFlags::REQUIRED, mw.clone());
    builder.add_section(SectionKind::ModuleWorld, MODULE_WORLD_SCHEMA_V1, SectionFlags::REQUIRED, mw);
    let bytes = builder.build().unwrap();

    let limits = ValidationLimits::default();
    let err = TeloscContainerView::parse(&bytes, &limits).unwrap_err();
    assert!(matches!(
        err,
        ArtifactError::Container(ContainerError::DuplicateSection(SectionKind::ModuleWorld))
    ));
}

#[test]
fn truncated_section_rejected_with_codec_error() {
    let payload = sample_payload();
    let encoded = payload.encode().unwrap();
    let truncated = &encoded[..encoded.len() - 20];

    let limits = ValidationLimits::default();
    let err = TeloscContainerView::parse(truncated, &limits).unwrap_err();
    assert!(matches!(
        err,
        ArtifactError::Codec(CodecError::UnexpectedEof) | ArtifactError::Container(ContainerError::OutOfFileSection { .. })
    ));
}

#[test]
fn whole_container_digest_mismatch_rejected() {
    let payload = sample_payload();
    let mut encoded = payload.encode().unwrap();
    // Corrupt a byte in the middle of a section payload
    let last = encoded.len() - 1;
    encoded[last] ^= 0xff;

    let limits = ValidationLimits::default();
    let err = TeloscContainerView::parse(&encoded, &limits).unwrap_err();
    assert!(matches!(
        err,
        ArtifactError::Container(ContainerError::SectionDigestMismatch { .. }) | ArtifactError::Container(ContainerError::ArtifactDigestMismatch { .. })
    ));
}

#[test]
fn unsupported_schema_version_rejected() {
    let payload = sample_payload();
    let mut header = payload.header.clone();
    header.container_schema = 999;
    header.minimum_reader_schema = 999;

    let mut builder = TeloscContainerBuilder::new(header);
    builder.add_section(
        SectionKind::ModuleWorld,
        MODULE_WORLD_SCHEMA_V1,
        SectionFlags::REQUIRED,
        payload.module_world.encode().unwrap(),
    );
    let bytes = builder.build().unwrap();

    let limits = ValidationLimits::default();
    let err = TeloscContainerView::parse(&bytes, &limits).unwrap_err();
    assert!(matches!(
        err,
        ArtifactError::Container(ContainerError::UnsupportedContainerSchema { found: 999, .. })
    ));
}

#[test]
fn wrong_project_provenance_rejected() {
    let payload = sample_payload();
    let encoded = payload.encode().unwrap();

    let foreign_compat = CanonicalCompatibilityIdentity {
        package: TeloscPackageIdentity::Package {
            package: "foreign".into(),
            version: "1.0.0".into(),
        },
        source_digest: ArtifactDigest::ZERO,
        interface_digest: ArtifactDigest::ZERO,
        semantic_digest: ArtifactDigest::ZERO,
        semantic_model_version: 1,
        stable_identity_schema: 1,
        module_semantic_abi: 1,
        semantic_artifact_abi: 1,
        native_surface_schema_version: 1,
        native_catalog_fingerprint: [1; 16],
        type_metadata_schema_version: 13,
    };

    let limits = ValidationLimits::default();
    let mut budget = AllocationBudget::new(32 * 1024 * 1024);
    let err = CompilerArtifactPayload::decode_and_validate(&encoded, &limits, Some(&foreign_compat), &mut budget);
    assert!(err.is_err());
    assert!(matches!(err.unwrap_err(), ArtifactError::Validation(ValidationError::IncompatibleWorld(_))));
}

#[test]
fn duplicate_query_root_key_rejected() {
    let query_image = QueryImageV1 {
        supported_roots: vec![
            QueryRootEntryV1 {
                key: PortableQueryKeyV1 {
                    kind: PortableQueryKindV1::LinkedInterface,
                    stable_owner_key: "core".into(),
                },
                dependency_indices: vec![],
            },
            QueryRootEntryV1 {
                key: PortableQueryKeyV1 {
                    kind: PortableQueryKindV1::LinkedInterface,
                    stable_owner_key: "core".into(),
                },
                dependency_indices: vec![],
            },
        ],
        dependency_evidence: vec![],
    };

    let err = query_image.validate(&ValidationLimits::default());
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("duplicate query root key"));
}

#[test]
fn out_of_bounds_query_dependency_rejected() {
    let query_image = QueryImageV1 {
        supported_roots: vec![QueryRootEntryV1 {
            key: PortableQueryKeyV1 {
                kind: PortableQueryKindV1::LinkedInterface,
                stable_owner_key: "core".into(),
            },
            dependency_indices: vec![42],
        }],
        dependency_evidence: vec![],
    };

    let err = query_image.validate(&ValidationLimits::default());
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("query dependency index 42 is out of bounds"));
}

#[test]
fn cumulative_budget_exhaustion_and_retry_accounting() {
    let payload = sample_payload();
    let encoded = payload.encode().unwrap();
    let limits = ValidationLimits::default();

    // With budget too small: fails with AllocationError::Exhausted
    let mut insufficient_budget = AllocationBudget::new(100);
    let result = CompilerArtifactPayload::decode_and_validate(&encoded, &limits, None, &mut insufficient_budget);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ArtifactError::Allocation(AllocationError::Exhausted)));
    assert_eq!(insufficient_budget.requested(), 0, "failed decode must not commit allocations");

    // Retry with sufficient budget: succeeds
    let mut sufficient_budget = AllocationBudget::new(32 * 1024 * 1024);
    let restored = CompilerArtifactPayload::decode_and_validate(&encoded, &limits, None, &mut sufficient_budget);
    assert!(restored.is_ok());
    assert!(sufficient_budget.requested() > 0);
}
