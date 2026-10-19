use phalcom_compiler_artifact::container::{TeloscContainerBuilder, TeloscContainerView};
use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_compiler_artifact::error::{ArtifactError, ContainerError, ValidationError};
use phalcom_compiler_artifact::header::{
    MODULE_SEMANTIC_ABI_V2, QUERY_FINGERPRINT_ABI_V2, SEMANTIC_ARTIFACT_ABI_V2, TELOSC_CONTAINER_SCHEMA_V2, TELOSC_MIN_READER_SCHEMA_V2,
    TeloscCapabilities, TeloscHeaderV2,
};
use phalcom_compiler_artifact::identity::TeloscPackageIdentity;
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::metadata::{MetadataPreflightLimits, MetadataSectionV1};
use phalcom_compiler_artifact::module_world::{
    ArtifactModuleId, MODULE_WORLD_SCHEMA_V1, MODULE_WORLD_SCHEMA_V2, ModuleKindV1, ModuleV2, ModuleWorldV1, ModuleWorldV2,
};
use phalcom_compiler_artifact::payload::CompilerArtifactPayload;
use phalcom_compiler_artifact::query::{
    PortableQueryKeyV1, PortableQueryKindV1, PortableQueryOwnerKeyV1, QUERY_IMAGE_SCHEMA_V2, QUERY_IMAGE_SCHEMA_V3,
    QueryImageV2, QueryImageV3, QueryProductRefV2, QueryRootEntryV3,
};
use phalcom_compiler_artifact::section::{SectionFlags, SectionKind};
use phalcom_compiler_artifact::semantic::aggregates::StructuralPrimaryContractsV1;
use phalcom_compiler_artifact::semantic::static_value::StaticValueGraphV1;
use phalcom_compiler_artifact::semantic::type_store::TypeGraphV1;
use phalcom_compiler_artifact::semantic::world::{SEMANTIC_WORLD_SCHEMA_V3, SemanticWorldV3};
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::{StableDeclarationRef, StableModuleRef, StableProjectRef};
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

fn sample_metadata(pkg_id: &TeloscPackageIdentity) -> MetadataSectionV1 {
    let bundle = sample_bundle();
    let limits = MetadataPreflightLimits {
        max_metadata_bytes: 65536,
        max_token_count: 10000,
        max_container_count: 10000,
        max_nesting_depth: 32,
    };
    MetadataSectionV1::from_bundle(&bundle, pkg_id.digest().0, &limits, &phalcom_type_meta::ValidationLimits::default()).unwrap()
}

fn universe_project() -> StableProjectRef {
    StableProjectRef::Builtin {
        namespace: "universe".into(),
        version: "0.1.0".into(),
    }
}

fn external_project_a() -> StableProjectRef {
    StableProjectRef::Package {
        package: "pkg_a".into(),
        version: "1.0.0".into(),
        artifact_fingerprint: Fingerprint128::from_u128(0xaaaa_bbbb),
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
        semantic_artifact_abi: SEMANTIC_ARTIFACT_ABI_V2,
        query_fingerprint_abi: QUERY_FINGERPRINT_ABI_V2,
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

    let root_module = ModuleV2 {
        identity: StableModuleRef {
            project: universe_project(),
            path: Box::new([]),
        },
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

    let mut module_world = ModuleWorldV2 {
        entry: ArtifactModuleId(0),
        initialization_order: vec![ArtifactModuleId(0)],
        modules: vec![root_module],
        semantic_rows: vec![],
        external_projects: vec![],
        topology_fingerprint: 0,
    };
    module_world.refresh_portable_topology_fingerprint().unwrap();

    let semantic_world = SemanticWorldV3 {
        type_graph: TypeGraphV1 {
            types: vec![],
            kinds: vec![],
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
            values: vec![],
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

    let query_image = QueryImageV3 {
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

fn empty_module(identity: StableModuleRef, path: Vec<String>) -> ModuleV2 {
    ModuleV2 {
        identity,
        path,
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
    }
}

#[test]
fn same_path_in_distinct_projects_remains_distinct() {
    let mod_u = empty_module(
        StableModuleRef {
            project: universe_project(),
            path: Box::new(["math".into()]),
        },
        vec!["math".into()],
    );
    let mod_ext = empty_module(
        StableModuleRef {
            project: external_project_a(),
            path: Box::new(["math".into()]),
        },
        vec!["math".into()],
    );

    let mut world = ModuleWorldV2 {
        entry: ArtifactModuleId(0),
        initialization_order: vec![ArtifactModuleId(0)],
        modules: vec![mod_u, mod_ext],
        semantic_rows: vec![],
        external_projects: vec![external_project_a()],
        topology_fingerprint: 0,
    };
    world.refresh_portable_topology_fingerprint().unwrap();

    assert!(world.validate(&ValidationLimits::default()).is_ok());
}

#[test]
fn portable_topology_fingerprint_rejects_represented_identity_edge_and_exposure_changes() {
    let root_identity = StableModuleRef {
        project: universe_project(),
        path: Box::new([]),
    };
    let child_identity = StableModuleRef {
        project: universe_project(),
        path: Box::new(["math".into()]),
    };
    let mut root = empty_module(root_identity, vec![]);
    root.topology_children = vec![ArtifactModuleId(1)];
    root.exposed_children = vec!["math".into()];
    let mut child = empty_module(child_identity, vec!["math".into()]);
    child.topology_parent = Some(ArtifactModuleId(0));

    let mut world = ModuleWorldV2 {
        entry: ArtifactModuleId(0),
        initialization_order: vec![ArtifactModuleId(0)],
        modules: vec![root, child],
        semantic_rows: vec![],
        external_projects: vec![],
        topology_fingerprint: 0,
    };
    world.refresh_portable_topology_fingerprint().unwrap();
    assert!(world.validate(&ValidationLimits::default()).is_ok());

    let mut changed_identity = world.clone();
    changed_identity.modules[1].identity.project = external_project_a();
    changed_identity.external_projects = vec![external_project_a()];
    assert_portable_topology_mismatch(changed_identity);

    let mut changed_edge = world.clone();
    changed_edge.modules[0].topology_children.clear();
    assert_portable_topology_mismatch(changed_edge);

    let mut changed_exposure = world;
    changed_exposure.modules[0].exposed_children[0] = "other".into();
    assert_portable_topology_mismatch(changed_exposure);
}

fn assert_portable_topology_mismatch(world: ModuleWorldV2) {
    let error = world.validate(&ValidationLimits::default()).unwrap_err();
    assert!(
        error.to_string().contains("portable module topology fingerprint mismatch"),
        "unexpected error: {error}"
    );
}

#[test]
fn duplicate_full_module_identity_rejected() {
    let mod1 = empty_module(
        StableModuleRef {
            project: universe_project(),
            path: Box::new(["core".into()]),
        },
        vec!["core".into()],
    );
    let mod2 = empty_module(
        StableModuleRef {
            project: universe_project(),
            path: Box::new(["core".into()]),
        },
        vec!["core".into()],
    );

    let world = ModuleWorldV2 {
        entry: ArtifactModuleId(0),
        initialization_order: vec![ArtifactModuleId(0)],
        modules: vec![mod1, mod2],
        semantic_rows: vec![],
        external_projects: vec![],
        topology_fingerprint: 0,
    };

    let err = world.validate(&ValidationLimits::default()).unwrap_err();
    assert!(err.to_string().contains("duplicate module identity"));
}

#[test]
fn missing_internal_module_owner_rejected() {
    let mut payload = sample_payload();
    // A query root references an internal module that does not exist in module_world
    payload.query_image.supported_roots.push(QueryRootEntryV3 {
        key: PortableQueryKeyV1 {
            kind: PortableQueryKindV1::LinkedInterface,
            owner: PortableQueryOwnerKeyV1::Module(StableModuleRef {
                project: universe_project(),
                path: Box::new(["nonexistent".into()]),
            }),
        },
        product: QueryProductRefV2 {
            kind: PortableQueryKindV1::LinkedInterface,
            row_index: 0,
        },
        input_fingerprint: 0,
        product_fingerprint: 0,
        dependency_indices: vec![],
    });

    let err = payload.validate_cross_section(&ValidationLimits::default()).unwrap_err();
    assert!(err.to_string().contains("missing internal module owner"));
}

#[test]
fn unlisted_external_project_rejected() {
    let mut payload = sample_payload();
    // Query root references an external project that is NOT in external_projects
    payload.query_image.supported_roots.push(QueryRootEntryV3 {
        key: PortableQueryKeyV1 {
            kind: PortableQueryKindV1::LinkedInterface,
            owner: PortableQueryOwnerKeyV1::Module(StableModuleRef {
                project: external_project_a(),
                path: Box::new(["math".into()]),
            }),
        },
        product: QueryProductRefV2 {
            kind: PortableQueryKindV1::LinkedInterface,
            row_index: 0,
        },
        input_fingerprint: 0,
        product_fingerprint: 0,
        dependency_indices: vec![],
    });

    let err = payload.validate_cross_section(&ValidationLimits::default()).unwrap_err();
    assert!(err.to_string().contains("unlisted external project owner"));
}

#[test]
fn exact_allowlisted_external_project_accepted() {
    let mut payload = sample_payload();
    payload.module_world.external_projects = vec![external_project_a()];
    payload.query_image.supported_roots.push(QueryRootEntryV3 {
        key: PortableQueryKeyV1 {
            kind: PortableQueryKindV1::DeclarationSurface,
            owner: PortableQueryOwnerKeyV1::Declaration(StableDeclarationRef {
                module: StableModuleRef {
                    project: external_project_a(),
                    path: Box::new(["math".into()]),
                },
                path: Box::new(["Matrix".into()]),
            }),
        },
        product: QueryProductRefV2 {
            kind: PortableQueryKindV1::DeclarationSurface,
            row_index: 0,
        },
        input_fingerprint: 0,
        product_fingerprint: 0,
        dependency_indices: vec![],
    });

    // Provide a semantic row for DeclarationSurface
    payload.semantic_world.dispatch_surfaces.push(phalcom_compiler_artifact::semantic::world::DeclarationSurfaceRecordV1 {
        declaration: StableDeclarationRef {
            module: StableModuleRef {
                project: external_project_a(),
                path: Box::new(["math".into()]),
            },
            path: Box::new(["Matrix".into()]),
        },
        instance_callables: vec![],
        class_callables: vec![],
        fields: vec![],
    });

    assert!(payload.validate_cross_section(&ValidationLimits::default()).is_ok());
}

#[test]
fn external_project_fingerprint_mismatch_rejected() {
    let mut payload = sample_payload();
    // Allowlist contains external_project_a with fingerprint 0xaaaa_bbbb
    payload.module_world.external_projects = vec![external_project_a()];

    // Query root references external project with DIFFERENT fingerprint 0x1111_2222
    let mismatched = StableProjectRef::Package {
        package: "pkg_a".into(),
        version: "1.0.0".into(),
        artifact_fingerprint: Fingerprint128::from_u128(0x1111_2222),
    };

    payload.query_image.supported_roots.push(QueryRootEntryV3 {
        key: PortableQueryKeyV1 {
            kind: PortableQueryKindV1::DeclarationSurface,
            owner: PortableQueryOwnerKeyV1::Declaration(StableDeclarationRef {
                module: StableModuleRef {
                    project: mismatched,
                    path: Box::new(["math".into()]),
                },
                path: Box::new(["Matrix".into()]),
            }),
        },
        product: QueryProductRefV2 {
            kind: PortableQueryKindV1::DeclarationSurface,
            row_index: 0,
        },
        input_fingerprint: 0,
        product_fingerprint: 0,
        dependency_indices: vec![],
    });

    let err = payload.validate_cross_section(&ValidationLimits::default()).unwrap_err();
    assert!(err.to_string().contains("unlisted external project owner"));
}

#[test]
fn session_project_owner_rejected() {
    let session = StableProjectRef::Session {
        session_fingerprint: Fingerprint128::from_u128(999),
    };

    // 1. In module world
    let mod_session = empty_module(
        StableModuleRef {
            project: session.clone(),
            path: Box::new(["test".into()]),
        },
        vec!["test".into()],
    );
    let world = ModuleWorldV2 {
        entry: ArtifactModuleId(0),
        initialization_order: vec![ArtifactModuleId(0)],
        modules: vec![mod_session],
        semantic_rows: vec![],
        external_projects: vec![],
        topology_fingerprint: 0,
    };
    let err = world.validate(&ValidationLimits::default()).unwrap_err();
    assert!(err.to_string().contains("session project identity"));

    // 2. In query image
    let query_image = QueryImageV3 {
        supported_roots: vec![QueryRootEntryV3 {
            key: PortableQueryKeyV1 {
                kind: PortableQueryKindV1::LinkedInterface,
                owner: PortableQueryOwnerKeyV1::Module(StableModuleRef {
                    project: session,
                    path: Box::new(["test".into()]),
                }),
            },
            product: QueryProductRefV2 {
                kind: PortableQueryKindV1::LinkedInterface,
                row_index: 0,
            },
            input_fingerprint: 0,
            product_fingerprint: 0,
            dependency_indices: vec![],
        }],
        dependency_evidence: vec![],
    };
    let err = query_image.validate(&ValidationLimits::default()).unwrap_err();
    assert!(err.to_string().contains("session project"));
}

#[test]
fn query_owner_project_mismatch_rejected() {
    let mut payload = sample_payload();
    // Allowlist contains external project
    payload.module_world.external_projects = vec![external_project_a()];

    // A query root for LinkedInterface claims to describe module row 0 (which is in universe_project),
    // but the query root's owner specifies external_project_a!
    payload.query_image.supported_roots.push(QueryRootEntryV3 {
        key: PortableQueryKeyV1 {
            kind: PortableQueryKindV1::LinkedInterface,
            owner: PortableQueryOwnerKeyV1::Module(StableModuleRef {
                project: external_project_a(),
                path: Box::new([]),
            }),
        },
        product: QueryProductRefV2 {
            kind: PortableQueryKindV1::LinkedInterface,
            row_index: 0,
        },
        input_fingerprint: 0,
        product_fingerprint: 0,
        dependency_indices: vec![],
    });

    let err = payload.validate_cross_section(&ValidationLimits::default()).unwrap_err();
    assert!(err.to_string().contains("query root owner"));
    assert!(err.to_string().contains("does not match referenced module"));
}

#[test]
fn legacy_module_schema_rejected() {
    let mut builder = TeloscContainerBuilder::new(sample_payload().header);
    // Add legacy module world schema V1
    builder.add_section(
        SectionKind::ModuleWorld,
        MODULE_WORLD_SCHEMA_V1,
        SectionFlags::REQUIRED,
        vec![0u8; 16],
    );
    builder.add_section(
        SectionKind::SemanticWorld,
        SEMANTIC_WORLD_SCHEMA_V3,
        SectionFlags::REQUIRED,
        vec![0u8; 16],
    );
    builder.add_section(
        SectionKind::QueryImage,
        QUERY_IMAGE_SCHEMA_V3,
        SectionFlags::REQUIRED,
        vec![0u8; 16],
    );
    builder.add_section(
        SectionKind::Metadata,
        1,
        SectionFlags::REQUIRED,
        vec![0u8; 16],
    );
    let bytes = builder.build().unwrap();

    let limits = ValidationLimits::default();
    let err = TeloscContainerView::parse(&bytes, &limits).unwrap_err();
    assert!(matches!(
        err,
        ArtifactError::Container(ContainerError::UnsupportedSectionSchema {
            section: SectionKind::ModuleWorld,
            schema: 1,
        })
    ));

    // Also verify decode_with_allocation_budget rejects V1
    let v1_world = ModuleWorldV1 {
        entry: ArtifactModuleId(0),
        initialization_order: vec![],
        modules: vec![],
        semantic_rows: vec![],
        topology_fingerprint: 0,
    };
    let v1_bytes = v1_world.encode().unwrap();
    let mut budget = AllocationBudget::new(1024 * 1024);
    let err = ModuleWorldV2::decode_with_allocation_budget(&v1_bytes, &limits, &mut budget).unwrap_err();
    assert!(matches!(
        err,
        ArtifactError::Validation(ValidationError::IncompatibleAbi {
            kind: "module_world_schema",
            found: 1,
            expected: MODULE_WORLD_SCHEMA_V2,
        })
    ));
}

#[test]
fn legacy_query_schema_rejected() {
    let mut builder = TeloscContainerBuilder::new(sample_payload().header);
    builder.add_section(
        SectionKind::ModuleWorld,
        MODULE_WORLD_SCHEMA_V2,
        SectionFlags::REQUIRED,
        sample_payload().module_world.encode().unwrap(),
    );
    builder.add_section(
        SectionKind::SemanticWorld,
        SEMANTIC_WORLD_SCHEMA_V3,
        SectionFlags::REQUIRED,
        sample_payload().semantic_world.encode().unwrap(),
    );
    // Add legacy query image schema V2
    builder.add_section(
        SectionKind::QueryImage,
        QUERY_IMAGE_SCHEMA_V2,
        SectionFlags::REQUIRED,
        vec![0u8; 16],
    );
    builder.add_section(
        SectionKind::Metadata,
        1,
        SectionFlags::REQUIRED,
        vec![0u8; 16],
    );
    let bytes = builder.build().unwrap();

    let limits = ValidationLimits::default();
    let err = TeloscContainerView::parse(&bytes, &limits).unwrap_err();
    assert!(matches!(
        err,
        ArtifactError::Container(ContainerError::UnsupportedSectionSchema {
            section: SectionKind::QueryImage,
            schema: 2,
        })
    ));

    // Also verify decode_with_allocation_budget rejects V2
    let v2_query = QueryImageV2 {
        supported_roots: vec![],
        dependency_evidence: vec![],
    };
    let v2_bytes = v2_query.encode().unwrap();
    let mut budget = AllocationBudget::new(1024 * 1024);
    let err = QueryImageV3::decode_with_allocation_budget(&v2_bytes, &limits, &mut budget).unwrap_err();
    assert!(matches!(
        err,
        ArtifactError::Validation(ValidationError::IncompatibleAbi {
            kind: "query_image_schema",
            found: 2,
            expected: QUERY_IMAGE_SCHEMA_V3,
        })
    ));
}
