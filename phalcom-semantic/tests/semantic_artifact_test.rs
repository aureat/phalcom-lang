use phalcom_compiler_artifact::ArtifactError;
use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_compiler_artifact::header::{
    MODULE_SEMANTIC_ABI_V2, QUERY_FINGERPRINT_ABI_V2, SEMANTIC_ARTIFACT_ABI_V2, TELOSC_CONTAINER_SCHEMA_V2, TELOSC_MIN_READER_SCHEMA_V2, TeloscCapabilities,
    TeloscHeaderV2,
};
use phalcom_compiler_artifact::identity::TeloscPackageIdentity;
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::metadata::{MetadataPreflightLimits, MetadataSectionV1};
use phalcom_compiler_artifact::query::{
    DependencyEvidenceEntryV3, PortableQueryKeyV1, PortableQueryKindV1, PortableQueryOwnerKeyV1, QueryImageV3, QueryProductRefV2, QueryRootEntryV3,
};
use phalcom_compiler_artifact::semantic::aggregates::{DataInfoV1, DataShapeV1, EnumInfoV1, StructuralPrimaryContractsV1, VisibilityV1};
use phalcom_compiler_artifact::semantic::static_value::{ArtifactStaticValueId, StaticValueGraphV1, StaticValueV1};
use phalcom_compiler_artifact::semantic::type_store::{ArtifactKindId, ArtifactTypeId, KindV1, TypeEntryV1, TypeGraphV1, TypeV1};
use phalcom_compiler_artifact::semantic::world::SemanticWorldV3;
use phalcom_semantic::artifact::{
    CompilerArtifactImage, decode_and_validate_artifact, encode_artifact, import_compiler_image, import_compiler_image_bytes_with_project_catalog,
};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::{StableDeclarationRef, StableModuleRef, StableProjectRef};
use phalcom_type_meta::{
    ArtifactIdentityScheme, KindNode, KindNodeEntry, KindNodeId, MetadataFeatures, MetadataProfile, NATIVE_SURFACE_SCHEMA_VERSION, ProducerIdentity,
    SEMANTIC_MODEL_VERSION, SemanticMetadataBundle, SemanticMetadataHeader, TYPE_METADATA_SCHEMA_VERSION, TypeNode, TypeNodeEntry,
};
use std::sync::Arc;

fn sample_bundle() -> SemanticMetadataBundle {
    SemanticMetadataBundle {
        header: SemanticMetadataHeader {
            schema_version: TYPE_METADATA_SCHEMA_VERSION,
            semantic_model_version: SEMANTIC_MODEL_VERSION,
            producer: ProducerIdentity("phalcom-compiler".into()),
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

fn sample_image() -> CompilerArtifactImage {
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
        native_catalog_fingerprint: [7; 16],
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

    let universe = Arc::new(phalcom_modules::ProjectUniverse::new());
    let root = phalcom_modules::ModuleId::universe_root();
    let iface = phalcom_modules::UnlinkedModuleInterface {
        id: root.clone(),
        kind: phalcom_modules::ModuleKind::Package,
        declarations: std::collections::BTreeMap::new(),
        exports: std::collections::BTreeMap::new(),
        imports: vec![],
        exposed_children: std::collections::BTreeSet::new(),
        metadata: phalcom_modules::ModuleMetadata::default(),
    };
    let mut interfaces = std::collections::BTreeMap::new();
    interfaces.insert(root.clone(), Arc::new(iface.clone()));
    let linked_iface = phalcom_modules::LinkedModuleInterface {
        module: root.clone(),
        kind: phalcom_modules::ModuleKind::Package,
        exports: std::collections::BTreeMap::new(),
        metadata: phalcom_modules::ModuleMetadata::default(),
    };
    let mut modules = std::collections::BTreeMap::new();
    modules.insert(
        root.clone(),
        phalcom_modules::LinkedModule {
            interface: linked_iface,
            bindings: phalcom_modules::ModuleBindingLayout::default(),
            linked_reads: vec![],
            runtime_dependencies: vec![],
        },
    );
    let linked = Arc::new(phalcom_modules::LinkedProgram {
        universe: universe.clone(),
        modules,
        graphs: phalcom_modules::ModuleGraphs::default(),
        entry: root.clone(),
        initialization_order: vec![root.clone()],
    });
    let mut unlinked_map = std::collections::BTreeMap::new();
    unlinked_map.insert(root.clone(), iface);
    let topology = Arc::new(phalcom_modules::ModuleTopology::from_parts(
        phalcom_modules::ResolverGeneration(0),
        &universe,
        &unlinked_map,
        &std::collections::BTreeMap::new(),
    ));
    let external_world = phalcom_modules::canonical_products::CanonicalExternalModuleWorld::from_parts(linked, interfaces, topology).unwrap();
    let module_world = phalcom_modules::compiler_artifact::project_module_world(&external_world, &ValidationLimits::default()).unwrap();

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

    let query_image = QueryImageV3 {
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
            input_fingerprint: 0,
            product_fingerprint: 0,
            dependency_indices: vec![],
        }],
        dependency_evidence: vec![],
    };

    let preflight_limits = MetadataPreflightLimits {
        max_metadata_bytes: 65536,
        max_token_count: 10000,
        max_container_count: 10000,
        max_nesting_depth: 32,
    };
    let metadata = MetadataSectionV1::from_bundle(
        &sample_bundle(),
        pkg_id.digest().0,
        &preflight_limits,
        &phalcom_type_meta::ValidationLimits::default(),
    )
    .unwrap();

    CompilerArtifactImage {
        header,
        module_world,
        semantic_world,
        query_image,
        metadata,
    }
}

fn sample_module_world_with_core() -> phalcom_compiler_artifact::module_world::ModuleWorldV2 {
    let universe = Arc::new(phalcom_modules::ProjectUniverse::new());
    let root = phalcom_modules::ModuleId::universe_root();
    let core = phalcom_modules::ModuleId::universe(phalcom_modules::ModulePath::from_components(vec![
        phalcom_modules::ModuleComponent::from_identifier("core").expect("valid module component"),
    ]));
    let root_interface = phalcom_modules::UnlinkedModuleInterface {
        id: root.clone(),
        kind: phalcom_modules::ModuleKind::Package,
        declarations: std::collections::BTreeMap::new(),
        exports: std::collections::BTreeMap::new(),
        imports: vec![],
        exposed_children: [phalcom_modules::ModuleComponent::from_identifier("core").expect("valid module component")]
            .into_iter()
            .collect(),
        metadata: phalcom_modules::ModuleMetadata::default(),
    };
    let core_interface = phalcom_modules::UnlinkedModuleInterface {
        id: core.clone(),
        kind: phalcom_modules::ModuleKind::Module,
        declarations: std::collections::BTreeMap::new(),
        exports: std::collections::BTreeMap::new(),
        imports: vec![],
        exposed_children: std::collections::BTreeSet::new(),
        metadata: phalcom_modules::ModuleMetadata::default(),
    };
    let linked_root = phalcom_modules::LinkedModuleInterface {
        module: root.clone(),
        kind: phalcom_modules::ModuleKind::Package,
        exports: std::collections::BTreeMap::new(),
        metadata: phalcom_modules::ModuleMetadata::default(),
    };
    let linked_core = phalcom_modules::LinkedModuleInterface {
        module: core.clone(),
        kind: phalcom_modules::ModuleKind::Module,
        exports: std::collections::BTreeMap::new(),
        metadata: phalcom_modules::ModuleMetadata::default(),
    };
    let mut modules = std::collections::BTreeMap::new();
    for (id, interface) in [(root.clone(), linked_root.clone()), (core.clone(), linked_core.clone())] {
        modules.insert(
            id,
            phalcom_modules::LinkedModule {
                interface,
                bindings: phalcom_modules::ModuleBindingLayout::default(),
                linked_reads: vec![],
                runtime_dependencies: vec![],
            },
        );
    }
    let linked = Arc::new(phalcom_modules::LinkedProgram {
        universe: universe.clone(),
        modules,
        graphs: phalcom_modules::ModuleGraphs::default(),
        entry: root.clone(),
        initialization_order: vec![root.clone(), core.clone()],
    });
    let mut interfaces = std::collections::BTreeMap::new();
    interfaces.insert(root.clone(), Arc::new(root_interface.clone()));
    interfaces.insert(core.clone(), Arc::new(core_interface.clone()));
    let mut unlinked = std::collections::BTreeMap::new();
    unlinked.insert(root, root_interface);
    unlinked.insert(core, core_interface);
    let topology = Arc::new(phalcom_modules::ModuleTopology::from_parts(
        phalcom_modules::ResolverGeneration(0),
        &universe,
        &unlinked,
        &std::collections::BTreeMap::new(),
    ));
    let world = phalcom_modules::canonical_products::CanonicalExternalModuleWorld::from_parts(linked, interfaces, topology).unwrap();
    phalcom_modules::compiler_artifact::project_module_world(&world, &ValidationLimits::default()).unwrap()
}

#[test]
fn transactional_semantic_export_and_import_round_trip() {
    let image = sample_image();
    let bytes = encode_artifact(&image).expect("encode artifact");
    assert!(!bytes.is_empty());

    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let artifact = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget).expect("decode and validate artifact");

    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(artifact, &limits, &mut import_budget).expect("import compiler image");

    assert_eq!(imported.external_world.linked().modules.len(), 1);
    assert_eq!(imported.metadata_bundle.header.producer.0.as_ref(), "phalcom-compiler");
    assert_eq!(imported.semantic_world.type_graph.types.len(), 1);
    assert_eq!(imported.static_values.store().len(), 1);
    let imported_unit = imported.static_values.value(ArtifactStaticValueId(0)).expect("imported static unit row");
    assert!(matches!(
        imported.static_values.store().get(imported_unit),
        Ok(phalcom_semantic::static_value::StaticValueData::Unit)
    ));
    assert_eq!(imported.query_image.supported_roots.len(), 1);

    struct NoPackageResolver;
    impl phalcom_modules::TrustedPackageResolver for NoPackageResolver {
        fn package_authority_receipt(&self, _stable_project: &StableProjectRef) -> Option<phalcom_modules::ValidatedPackageAuthorityReceipt> {
            None
        }
    }
    let mut catalog_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported_with_catalog = import_compiler_image_bytes_with_project_catalog(
        &bytes,
        None,
        &limits,
        &mut catalog_budget,
        &phalcom_modules::ProjectUniverse::new(),
        &NoPackageResolver,
    )
    .expect("import through the caller's project catalog");
    assert!(
        Arc::ptr_eq(&imported_with_catalog.project_universe, &imported_with_catalog.external_world.linked().universe),
        "the imported image and module world must retain the same project graph"
    );
}

#[test]
fn structural_data_contract_reconstructs_as_native_semantic_product() {
    let mut image = sample_image();
    image.module_world = sample_module_world_with_core();
    let universe_project = StableProjectRef::Builtin {
        namespace: "universe".into(),
        version: "0.1.0".into(),
    };
    let owner = StableDeclarationRef {
        module: StableModuleRef {
            project: universe_project,
            path: vec!["core".into()].into_boxed_slice(),
        },
        path: vec!["Payload".into()].into_boxed_slice(),
    };
    image.semantic_world.type_graph.types.push(TypeEntryV1 {
        kind: ArtifactKindId(0),
        form: TypeV1::Nominal { declaration: owner.clone() },
    });
    image.semantic_world.structural_contracts.data_contracts.push(DataInfoV1 {
        owner: owner.clone(),
        root_form: ArtifactTypeId(1),
        generic_signature: None,
        static_data: None,
        shape: DataShapeV1::Tuple,
        components: vec![],
        constructor_visibility: VisibilityV1::Public,
        constructor_parameters: vec![],
        constructor_result_type: ArtifactTypeId(1),
        shape_contributions: vec![],
    });

    let bytes = encode_artifact(&image).expect("encode structural data artifact");
    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget).expect("validate structural data artifact");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(payload, &limits, &mut import_budget).expect("reconstruct structural data contract");
    let module = phalcom_modules::ModuleId::universe(phalcom_modules::ModulePath::from_components(vec![
        phalcom_modules::ModuleComponent::from_identifier("core").expect("valid module component"),
    ]));
    let owner = phalcom_semantic::identity::DeclarationId::new(module, "Payload".into());
    let info = imported
        .semantic_aggregates
        .data_semantics
        .get(&owner)
        .expect("native data declaration product");
    assert_eq!(info.owner, owner);
    assert!(info.components.is_empty());
    assert_eq!(info.shape, phalcom_semantic::data_semantics::DataShape::Tuple);
}

#[test]
fn structural_data_query_root_reconstructs_typed_product_and_rejects_stale_fingerprints() {
    let mut image = sample_image();
    image.module_world = sample_module_world_with_core();
    let stable_owner = StableDeclarationRef {
        module: StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "universe".into(),
                version: "0.1.0".into(),
            },
            path: vec!["core".into()].into_boxed_slice(),
        },
        path: vec!["CachedPayload".into()].into_boxed_slice(),
    };
    let stable_enum_owner = StableDeclarationRef {
        module: stable_owner.module.clone(),
        path: vec!["CachedChoice".into()].into_boxed_slice(),
    };
    image.semantic_world.type_graph.types.push(TypeEntryV1 {
        kind: ArtifactKindId(0),
        form: TypeV1::Nominal {
            declaration: stable_owner.clone(),
        },
    });
    image.semantic_world.type_graph.types.push(TypeEntryV1 {
        kind: ArtifactKindId(0),
        form: TypeV1::Nominal {
            declaration: stable_enum_owner.clone(),
        },
    });
    image.semantic_world.structural_contracts.data_contracts.push(DataInfoV1 {
        owner: stable_owner.clone(),
        root_form: ArtifactTypeId(1),
        generic_signature: None,
        static_data: None,
        shape: DataShapeV1::Tuple,
        components: vec![],
        constructor_visibility: VisibilityV1::Public,
        constructor_parameters: vec![],
        constructor_result_type: ArtifactTypeId(1),
        shape_contributions: vec![],
    });
    image.semantic_world.structural_contracts.enum_contracts.push(EnumInfoV1 {
        owner: stable_enum_owner.clone(),
        root_form: ArtifactTypeId(2),
        generic_signature: None,
        static_data: None,
        default_result_type: ArtifactTypeId(2),
        variants: vec![],
    });

    // First reconstruct the structural row without a DataDeclaration query root
    // so the test obtains the receiving compiler's canonical current-main
    // fingerprints rather than duplicating fingerprint logic in the fixture.
    let limits = ValidationLimits::default();
    let seed_bytes = encode_artifact(&image).expect("encode structural seed artifact");
    let mut seed_decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let seed_payload =
        decode_and_validate_artifact(&seed_bytes, &limits, None, &mut seed_decode_budget).expect("validate structural seed artifact");
    let mut seed_import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let seed_import = import_compiler_image(seed_payload, &limits, &mut seed_import_budget).expect("reconstruct structural seed");

    let module = phalcom_modules::ModuleId::universe(phalcom_modules::ModulePath::from_components(vec![
        phalcom_modules::ModuleComponent::from_identifier("core").expect("valid module component"),
    ]));
    let owner = phalcom_semantic::identity::DeclarationId::new(module, "CachedPayload".into());
    let info = seed_import
        .semantic_aggregates
        .data_semantics
        .get(&owner)
        .expect("reconstructed data declaration")
        .clone();
    let diagnostics: Arc<[phalcom_semantic::diagnostic::SemanticDiagnostic]> = Arc::from([]);
    let product = phalcom_semantic::db::product::DataDeclarationProduct {
        info,
        diagnostics: diagnostics.clone(),
    };
    let input_fingerprint = phalcom_semantic::db::fingerprint::data_declaration_input_fingerprint(&product).raw();
    let product_fingerprint = phalcom_semantic::db::fingerprint::data_declaration_product_fingerprint(&product).raw();

    let enum_owner = phalcom_semantic::identity::DeclarationId::new(
        owner.module.clone(),
        "CachedChoice".into(),
    );
    let enum_info = seed_import
        .semantic_aggregates
        .enum_semantics
        .enums
        .get(&enum_owner)
        .expect("reconstructed enum declaration")
        .clone();
    let enum_product = phalcom_semantic::db::product::EnumDeclarationProduct {
        info: enum_info,
        variants: Arc::from([]),
        diagnostics,
    };
    let enum_input_fingerprint = phalcom_semantic::db::fingerprint::enum_declaration_input_fingerprint(&enum_product).raw();
    let enum_product_fingerprint = phalcom_semantic::db::fingerprint::enum_declaration_product_fingerprint(&enum_product).raw();

    image.query_image.supported_roots.push(QueryRootEntryV3 {
        key: PortableQueryKeyV1 {
            kind: PortableQueryKindV1::DataDeclaration,
            owner: PortableQueryOwnerKeyV1::Declaration(stable_owner),
        },
        product: QueryProductRefV2 {
            kind: PortableQueryKindV1::DataDeclaration,
            row_index: 0,
        },
        input_fingerprint,
        product_fingerprint,
        dependency_indices: vec![],
    });
    image.query_image.supported_roots.push(QueryRootEntryV3 {
        key: PortableQueryKeyV1 {
            kind: PortableQueryKindV1::EnumDeclaration,
            owner: PortableQueryOwnerKeyV1::Declaration(stable_enum_owner),
        },
        product: QueryProductRefV2 {
            kind: PortableQueryKindV1::EnumDeclaration,
            row_index: 0,
        },
        input_fingerprint: enum_input_fingerprint,
        product_fingerprint: enum_product_fingerprint,
        dependency_indices: vec![],
    });

    let bytes = encode_artifact(&image).expect("encode fingerprinted structural artifact");
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget).expect("validate fingerprinted structural artifact");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(payload, &limits, &mut import_budget).expect("import fingerprinted structural product");
    assert_eq!(
        imported.query_products.len(),
        2,
        "DataDeclaration and EnumDeclaration must be staged as typed reusable products"
    );

    let mut db = phalcom_semantic::db::SemanticDb::new();
    imported
        .publish_query_products_into_empty_db(&mut db)
        .expect("publish structural product into fresh semantic db");
    let query_key = phalcom_semantic::db::QueryKey::DataDeclaration(owner.clone());
    let published = db
        .product(&query_key)
        .and_then(|product| product.as_data_declaration())
        .expect("typed DataDeclaration product must be Ready");
    assert_eq!(published.info.owner, owner);
    assert_eq!(
        db.ready_product_fingerprint(&query_key).map(|fingerprint| fingerprint.raw()),
        Some(product_fingerprint),
        "published database state must use the verified current-main product fingerprint"
    );
    let enum_query_key = phalcom_semantic::db::QueryKey::EnumDeclaration(enum_owner.clone());
    let published_enum = db
        .product(&enum_query_key)
        .and_then(|product| product.as_enum_declaration())
        .expect("typed EnumDeclaration product must be Ready");
    assert_eq!(published_enum.info.owner, enum_owner);
    assert_eq!(
        db.ready_product_fingerprint(&enum_query_key).map(|fingerprint| fingerprint.raw()),
        Some(enum_product_fingerprint),
        "published enum state must use the verified current-main product fingerprint"
    );
    assert!(
        imported.publish_query_products_into_empty_db(&mut db).is_err(),
        "artifact authority must not be merged into an already-populated semantic database"
    );

    let data_root = image
        .query_image
        .supported_roots
        .iter_mut()
        .find(|root| root.key.kind == PortableQueryKindV1::DataDeclaration)
        .expect("data query root");
    data_root.product_fingerprint ^= 1;

    let stale_bytes = encode_artifact(&image).expect("encode stale structural artifact");
    let mut stale_decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let stale_payload =
        decode_and_validate_artifact(&stale_bytes, &limits, None, &mut stale_decode_budget).expect("container remains structurally valid");
    let mut stale_import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let stale = import_compiler_image(stale_payload, &limits, &mut stale_import_budget);
    assert!(stale.is_err(), "stale query product fingerprint must fail closed");
    assert_eq!(
        stale_import_budget.requested(),
        0,
        "failed fingerprint validation must not commit staged allocation authority"
    );

    let data_root = image
        .query_image
        .supported_roots
        .iter_mut()
        .find(|root| root.key.kind == PortableQueryKindV1::DataDeclaration)
        .expect("data query root");
    data_root.product_fingerprint = product_fingerprint;
    let data_key = data_root.key.clone();
    image.query_image.dependency_evidence.push(DependencyEvidenceEntryV3 {
        key: data_key,
        stable_aggregate_owner: PortableQueryOwnerKeyV1::Declaration(StableDeclarationRef {
            module: StableModuleRef {
                project: StableProjectRef::Builtin {
                    namespace: "universe".into(),
                    version: "0.1.0".into(),
                },
                path: vec!["core".into()].into_boxed_slice(),
            },
            path: vec!["OtherPayload".into()].into_boxed_slice(),
        }),
        product_fingerprint,
    });

    let mismatched_bytes = encode_artifact(&image).expect("encode mismatched dependency owner artifact");
    let mut mismatch_decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let mismatch_payload =
        decode_and_validate_artifact(&mismatched_bytes, &limits, None, &mut mismatch_decode_budget).expect("wire owner remains structurally valid");
    let mut mismatch_import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let mismatch = import_compiler_image(mismatch_payload, &limits, &mut mismatch_import_budget);
    assert!(mismatch.is_err(), "dependency evidence must bind to the exact structural aggregate owner");
    assert_eq!(
        mismatch_import_budget.requested(),
        0,
        "mismatched dependency evidence must fail before committing import allocations"
    );
}

#[test]
fn exact_allocation_accounting_n_succeeds_n_minus_1_fails_and_retry_succeeds() {
    let image = sample_image();
    let bytes = encode_artifact(&image).unwrap();
    let limits = ValidationLimits::default();

    // 1. Measure exact allocation N required for import
    let mut measure_budget = AllocationBudget::new(64 * 1024 * 1024);
    let artifact = decode_and_validate_artifact(&bytes, &limits, None, &mut measure_budget).unwrap();

    let mut import_measure_budget = AllocationBudget::new(64 * 1024 * 1024);
    let measure_artifact = artifact.clone();
    let _imported = import_compiler_image(measure_artifact, &limits, &mut import_measure_budget).unwrap();
    let n = import_measure_budget.requested();
    assert!(n > 0, "import must require non-zero allocation budget");

    // 2. Exact N succeeds leaving exactly 0 remaining
    let mut exact_budget = AllocationBudget::new(n);
    let exact_artifact = artifact.clone();
    let imported = import_compiler_image(exact_artifact, &limits, &mut exact_budget);
    assert!(imported.is_ok(), "exact budget N must succeed");
    assert_eq!(exact_budget.remaining(), 0, "exact budget N must leave 0 remaining");

    // 3. N - 1 fails with typed AllocationError::Exhausted
    let mut insufficient_budget = AllocationBudget::new(n - 1);
    let insufficient_artifact = artifact.clone();
    let failed = import_compiler_image(insufficient_artifact, &limits, &mut insufficient_budget);
    assert!(failed.is_err(), "budget N - 1 must fail");
    let err = failed.unwrap_err();
    assert!(
        matches!(err, ArtifactError::Allocation(AllocationError::Exhausted)),
        "failure must be typed AllocationError::Exhausted, got {err:?}"
    );
    // Crucial: verify fail-closed atomicity: insufficient_budget left uncommitted
    assert_eq!(insufficient_budget.requested(), 0, "failed import must not commit allocations to caller budget");

    // 4. Retry with sufficient budget succeeds
    let mut retry_budget = AllocationBudget::new(n);
    let retry_artifact = artifact.clone();
    let retried = import_compiler_image(retry_artifact, &limits, &mut retry_budget);
    assert!(retried.is_ok(), "retry with N must succeed");
    assert_eq!(retry_budget.remaining(), 0);
}

#[test]
fn package_artifact_import_registers_trusted_receipt_in_retained_workspace_graph() {
    let mut image = sample_image();
    let stable_package = StableProjectRef::Package {
        package: "shared-package".into(),
        version: "2.4.1".into(),
        artifact_fingerprint: Fingerprint128([0x51; 16]),
    };
    image.header.package_identity = TeloscPackageIdentity::Package {
        package: "shared-package".into(),
        version: "2.4.1".into(),
    };
    image.module_world.external_projects = vec![stable_package.clone()];
    image
        .module_world
        .refresh_portable_topology_fingerprint()
        .expect("refresh package topology fingerprint");
    image.metadata.package_digest = image.header.package_identity.digest().0;
    let bytes = encode_artifact(&image).expect("encode package artifact");

    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let validated_payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget).expect("validate package bytes");
    let artifact_digest = validated_payload.header.artifact_digest;

    let temp = tempfile::TempDir::new().expect("create package project root");
    let project_root = temp.path().join("resolver-target");
    std::fs::create_dir_all(project_root.join("src")).expect("create source root");
    std::fs::write(
        project_root.join("project.toml"),
        "[project]\nname = \"resolver-target\"\nnamespace = \"resolver_target\"\nversion = \"1.0.0\"\n",
    )
    .expect("write project manifest");
    std::fs::write(project_root.join("src/package.ph"), "").expect("write package source");
    let mut projects = phalcom_modules::ProjectUniverse::new();
    let project_id = projects.load_root(project_root.join("project.toml")).expect("load persistent resolver target");
    let receipt = phalcom_modules::ValidatedPackageAuthorityReceipt::from_trusted_resolver(stable_package.clone(), artifact_digest, project_id)
        .expect("make trusted resolver receipt");
    struct Resolver(phalcom_modules::ValidatedPackageAuthorityReceipt);
    impl phalcom_modules::TrustedPackageResolver for Resolver {
        fn package_authority_receipt(&self, stable_project: &StableProjectRef) -> Option<phalcom_modules::ValidatedPackageAuthorityReceipt> {
            (self.0.stable_project() == stable_project).then(|| self.0.clone())
        }
    }
    let resolver = Resolver(receipt);

    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image_bytes_with_project_catalog(&bytes, None, &limits, &mut import_budget, &projects, &resolver)
        .expect("import package artifact with trusted resolver receipt");
    assert_eq!(
        imported.project_universe.resolve_stable_project(&stable_package),
        Some(phalcom_modules::ProjectIdentity::Resolved(project_id))
    );
    assert_eq!(
        imported.project_universe.projects().len(),
        projects.projects().len(),
        "receipt registration must not allocate IDs"
    );
    assert!(Arc::ptr_eq(&imported.project_universe, &imported.external_world.linked().universe));

    let unrelated_manifest = {
        let unrelated_root = temp.path().join("unrelated-project");
        std::fs::create_dir_all(unrelated_root.join("src")).expect("create unrelated source root");
        std::fs::write(
            unrelated_root.join("project.toml"),
            "[project]\nname = \"unrelated\"\nnamespace = \"unrelated\"\nversion = \"1.0.0\"\n",
        )
        .expect("write unrelated manifest");
        std::fs::write(unrelated_root.join("src/package.ph"), "").expect("write unrelated package source");
        unrelated_root.join("project.toml")
    };
    let mut projects_with_unrelated = projects.clone();
    projects_with_unrelated
        .load_root(unrelated_manifest)
        .expect("load unrelated persistent project");
    let mut second_import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported_with_unrelated =
        import_compiler_image_bytes_with_project_catalog(&bytes, None, &limits, &mut second_import_budget, &projects_with_unrelated, &resolver)
            .expect("same package artifact imports with an unrelated workspace project");
    assert_eq!(
        imported.project_universe.resolve_stable_project(&stable_package),
        imported_with_unrelated.project_universe.resolve_stable_project(&stable_package),
        "receipt still targets the same existing project"
    );
    assert_ne!(
        imported.external_world.topology().fingerprint,
        imported_with_unrelated.external_world.topology().fingerprint,
        "live topology fingerprints include the distinct receiving workspace graphs"
    );
}

#[test]
fn hostile_corrupted_metadata_fails_closed_without_publishing() {
    let image = sample_image();
    let bytes = encode_artifact(&image).unwrap();
    let limits = ValidationLimits::default();

    let mut budget = AllocationBudget::new(32 * 1024 * 1024);
    let mut artifact = decode_and_validate_artifact(&bytes, &limits, None, &mut budget).unwrap();

    // Corrupt the metadata section package digest
    artifact.metadata.package_digest = [0xff; 32];

    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let result = import_compiler_image(artifact, &limits, &mut import_budget);
    assert!(result.is_err(), "corrupted metadata digest must fail import");
    assert_eq!(import_budget.requested(), 0, "uncommitted import must debit 0 bytes");
}
