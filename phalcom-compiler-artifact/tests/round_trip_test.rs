use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_compiler_artifact::header::{
    MODULE_SEMANTIC_ABI_V2, QUERY_FINGERPRINT_ABI_V2, SEMANTIC_ARTIFACT_ABI_V3, TELOSC_CONTAINER_SCHEMA_V2, TELOSC_MIN_READER_SCHEMA_V2,
    TeloscCapabilities, TeloscHeaderV2,
};
use phalcom_compiler_artifact::identity::{CanonicalCompatibilityIdentity, TeloscPackageIdentity};
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::metadata::{MetadataPreflightLimits, MetadataSectionV1};
use phalcom_compiler_artifact::module_world::{ArtifactModuleId, ModuleKindV1, ModuleV2, ModuleWorldV2};
use phalcom_compiler_artifact::payload::CompilerArtifactPayload;
use phalcom_compiler_artifact::query::{
    PortableQueryKeyV1, PortableQueryKindV1, PortableQueryOwnerKeyV1, QueryImageV3, QueryProductRefV2, QueryRootEntryV3,
};
use phalcom_compiler_artifact::semantic::aggregates::StructuralPrimaryContractsV1;
use phalcom_compiler_artifact::semantic::static_value::{StaticValueGraphV1, StaticValueV1};
use phalcom_compiler_artifact::semantic::type_store::{ArtifactKindId, KindV1, TypeEntryV1, TypeGraphV1, TypeV1};
use phalcom_compiler_artifact::semantic::world::SemanticWorldV3;
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::{StableModuleRef, StableProjectRef};
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

fn sample_metadata_section(pkg_identity: &TeloscPackageIdentity) -> MetadataSectionV1 {
    let bundle = sample_bundle();
    let limits = MetadataPreflightLimits {
        max_metadata_bytes: 65536,
        max_token_count: 10000,
        max_container_count: 10000,
        max_nesting_depth: 32,
    };
    MetadataSectionV1::from_bundle(&bundle, pkg_identity.digest().0, &limits, &phalcom_type_meta::ValidationLimits::default()).expect("create metadata section")
}

fn sample_module_world() -> ModuleWorldV2 {
    let universe_project = StableProjectRef::Builtin {
        namespace: "universe".into(),
        version: "0.1.0".into(),
    };
    let root_module = ModuleV2 {
        identity: StableModuleRef {
            project: universe_project.clone(),
            path: Box::new([]),
        },
        path: vec![],
        kind: ModuleKindV1::Package,
        declarations: vec![],
        exports: vec![],
        imports: vec![],
        exposed_children: vec!["core".into()],
        metadata: vec![],
        linked_exports: vec![],
        globals: vec![],
        import_bindings: vec![],
        linked_reads: vec![],
        runtime_dependencies: vec![],
        references: vec![],
        runtime_edges: vec![],
        topology_parent: None,
        topology_children: vec![ArtifactModuleId(1)],
        interface_fingerprint: 0,
        linked_interface_fingerprint: 0,
        dependency_fingerprint: 0,
    };

    let child_module = ModuleV2 {
        identity: StableModuleRef {
            project: universe_project,
            path: Box::new(["core".into()]),
        },
        path: vec!["core".into()],
        kind: ModuleKindV1::Module,
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
        topology_parent: Some(ArtifactModuleId(0)),
        topology_children: vec![],
        interface_fingerprint: 0,
        linked_interface_fingerprint: 0,
        dependency_fingerprint: 0,
    };

    let mut module_world = ModuleWorldV2 {
        entry: ArtifactModuleId(0),
        initialization_order: vec![ArtifactModuleId(0), ArtifactModuleId(1)],
        modules: vec![root_module, child_module],
        semantic_rows: vec![],
        external_projects: vec![],
        topology_fingerprint: 0,
    };
    module_world.refresh_portable_topology_fingerprint().unwrap();
    module_world
}

fn sample_semantic_world() -> SemanticWorldV3 {
    let type_graph = TypeGraphV1 {
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
    };

    let static_values = StaticValueGraphV1 {
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
    };

    SemanticWorldV3 {
        type_graph,
        structural_contracts: StructuralPrimaryContractsV1 {
            declaration_shells: vec![],
            hierarchy_and_aliases: vec![],
            generic_signatures: vec![],
            data_contracts: vec![],
            enum_contracts: vec![],
            variant_contracts: vec![],
        },
        enum_requirements: vec![],
        static_values,
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
    }
}

fn sample_query_image() -> QueryImageV3 {
    QueryImageV3 {
        supported_roots: vec![QueryRootEntryV3 {
            key: PortableQueryKeyV1 {
                kind: PortableQueryKindV1::LinkedInterface,
                owner: PortableQueryOwnerKeyV1::Module(StableModuleRef {
                    project: StableProjectRef::Builtin {
                        namespace: "universe".into(),
                        version: "0.1.0".into(),
                    },
                    path: Box::new([]),
                }),
            },
            product: QueryProductRefV2 {
                kind: PortableQueryKindV1::LinkedInterface,
                row_index: 0,
            },
            input_fingerprint: 1,
            product_fingerprint: 2,
            dependency_indices: vec![],
        }],
        dependency_evidence: vec![],
    }
}

fn sample_payload() -> CompilerArtifactPayload {
    let pkg_id = TeloscPackageIdentity::universe();
    let header = TeloscHeaderV2 {
        container_schema: TELOSC_CONTAINER_SCHEMA_V2,
        minimum_reader_schema: TELOSC_MIN_READER_SCHEMA_V2,
        package_identity: pkg_id.clone(),
        semantic_model_version: 1,
        stable_identity_schema: 1,
        module_semantic_abi: MODULE_SEMANTIC_ABI_V2,
        semantic_artifact_abi: SEMANTIC_ARTIFACT_ABI_V3,
        query_fingerprint_abi: QUERY_FINGERPRINT_ABI_V2,
        native_surface_schema_version: 1,
        native_catalog_fingerprint: [5; 16],
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

    CompilerArtifactPayload {
        header,
        module_world: sample_module_world(),
        semantic_world: sample_semantic_world().into(),
        query_image: sample_query_image(),
        metadata: sample_metadata_section(&pkg_id),
    }
}

#[test]
fn complete_compiler_artifact_round_trips_through_binary_container() {
    let payload = sample_payload();
    let encoded = payload.encode().expect("encode compiler artifact");
    assert!(!encoded.is_empty());
    assert_eq!(&encoded[0..6], b"TELOSC");

    let limits = ValidationLimits::default();
    let mut budget = AllocationBudget::new(32 * 1024 * 1024);
    let restored = CompilerArtifactPayload::decode_and_validate(&encoded, &limits, None, &mut budget).expect("decode and validate compiler artifact");

    assert_eq!(restored.header.package_identity, payload.header.package_identity);
    assert_eq!(restored.module_world.modules.len(), payload.module_world.modules.len());
    assert_eq!(restored.module_world.modules[0].path, payload.module_world.modules[0].path);
    assert_eq!(restored.module_world.modules[1].path, payload.module_world.modules[1].path);
    assert_eq!(restored.semantic_world.type_graph.types.len(), payload.semantic_world.type_graph.types.len());
    assert_eq!(restored.query_image.supported_roots.len(), payload.query_image.supported_roots.len());
    assert_eq!(restored.metadata.package_digest, payload.metadata.package_digest);
    assert!(budget.remaining() < 32 * 1024 * 1024);
}

#[test]
fn compatibility_identity_verifies_matching_and_rejects_foreign() {
    let payload = sample_payload();
    let encoded = payload.encode().expect("encode compiler artifact");

    let limits = ValidationLimits::default();
    let mut initial_budget = AllocationBudget::new(32 * 1024 * 1024);
    let initial = CompilerArtifactPayload::decode_and_validate(&encoded, &limits, None, &mut initial_budget).unwrap();

    let matching_identity = CanonicalCompatibilityIdentity {
        package: initial.header.package_identity.clone(),
        source_digest: initial.header.source_digest,
        interface_digest: initial.header.interface_digest,
        semantic_digest: initial.header.semantic_digest,
        semantic_model_version: initial.header.semantic_model_version,
        stable_identity_schema: initial.header.stable_identity_schema,
        module_semantic_abi: initial.header.module_semantic_abi,
        semantic_artifact_abi: initial.header.semantic_artifact_abi,
        query_fingerprint_abi: initial.header.query_fingerprint_abi,
        native_surface_schema_version: initial.header.native_surface_schema_version,
        native_catalog_fingerprint: initial.header.native_catalog_fingerprint,
        type_metadata_schema_version: initial.header.type_metadata_schema_version,
    };

    let mut budget = AllocationBudget::new(32 * 1024 * 1024);
    assert!(CompilerArtifactPayload::decode_and_validate(&encoded, &limits, Some(&matching_identity), &mut budget).is_ok());

    let mut foreign_identity = matching_identity;
    foreign_identity.native_catalog_fingerprint = [99; 16];
    let mut budget2 = AllocationBudget::new(32 * 1024 * 1024);
    let err = CompilerArtifactPayload::decode_and_validate(&encoded, &limits, Some(&foreign_identity), &mut budget2);
    assert!(err.is_err());
}
