use phalcom_compiler_artifact::ArtifactError;
use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_compiler_artifact::header::{
    MODULE_SEMANTIC_ABI_V2, QUERY_FINGERPRINT_ABI_V2, SEMANTIC_ARTIFACT_ABI_V2, TELOSC_CONTAINER_SCHEMA_V2, TELOSC_MIN_READER_SCHEMA_V2, TeloscCapabilities,
    TeloscHeaderV2,
};
use phalcom_compiler_artifact::identity::TeloscPackageIdentity;
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::metadata::{MetadataPreflightLimits, MetadataSectionV1};
use phalcom_compiler_artifact::module_world::SpanV1;
use phalcom_compiler_artifact::query::{
    DependencyEvidenceEntryV3, PortableQueryKeyV1, PortableQueryKindV1, PortableQueryOwnerKeyV1, QueryImageV3, QueryProductRefV2, QueryRootEntryV3,
};
use phalcom_compiler_artifact::semantic::aggregates::{
    DataInfoV1, DataShapeV1, DeclaredTypeBasisV1, DeclaredTypeFactV1, DeclaredTypeStateV1, EnumInfoV1, StructuralPrimaryContractsV1,
    UnknownReasonV1, VisibilityV1,
};
use phalcom_compiler_artifact::semantic::static_value::{ArtifactStaticValueId, StaticValueGraphV1, StaticValueV1};
use phalcom_compiler_artifact::semantic::type_store::{
    ArtifactKindId, ArtifactTypeId, KindV1, StaticDomainV1, TypeEntryV1, TypeGraphV1, TypeMemberSelectorV1, TypeV1,
};
use phalcom_compiler_artifact::semantic::world::{
    AssociatedFamilyKindV1, AssociatedFamilyRecordV1, AssociatedMemberV1, AssociatedSurfaceRecordV1,
    CallableDefinitionBodyProvenanceV1, CallableDefinitionOriginV1, CallableDefinitionRecordV1, CallableSignatureRecordV1,
    ConstructorInitializationRecordV1, DeclarationSurfaceRecordV1, DispatchCallableFactV1, EffectSpecV1,
    EnumRequirementOwnerV1, EnumRequirementRecordV1, ExactSelectorV1, FieldSignatureV1, ImplementationKindV1,
    NativeLifecycleSpecV1,
    RaisesSpecV1, ReturnContractValidationV1,
    ReturnFlowV1, SelectorKindV1, SemanticWorldV3, TypeKnowledgeV1, StaticCaseDomainV1, StaticCaseHeaderV1, StaticCaseShapeV1,
    StaticDomainCaseCatalogRecordV1, TypeMemberDeclarationRecordV1, TypeMemberDomainV1, TypeMemberImplementationV1,
};
use phalcom_semantic::artifact::{
    CompilerArtifactImage, decode_and_validate_artifact, encode_artifact, import_compiler_image, import_compiler_image_bytes_with_project_catalog,
};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::{
    StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide, StableModuleRef, StableProjectRef, StableTypeMemberRef,
};
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
    image.semantic_world.associated_surfaces.push(AssociatedSurfaceRecordV1 {
        owner: stable_owner.clone(),
        families: vec![AssociatedFamilyRecordV1 {
            selector: TypeMemberSelectorV1::Named {
                name: "Construct".to_string(),
            },
            kind: AssociatedFamilyKindV1::DataConstructor,
            members: vec![AssociatedMemberV1::DataConstructor {
                owner: stable_owner.clone(),
            }],
        }],
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
    assert_eq!(
        seed_import.associated_authority.surface_len(),
        1,
        "P7 associated semantic authority must be reconstructed independently of query-root selection"
    );
    assert_eq!(
        seed_import.query_products.len(),
        0,
        "an associated semantic row is not reusable query authority until QUERY_IMAGE selects it"
    );

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
            owner: PortableQueryOwnerKeyV1::Declaration(stable_owner.clone()),
        },
        product: QueryProductRefV2 {
            kind: PortableQueryKindV1::DataDeclaration,
            row_index: 0,
        },
        input_fingerprint: 0,
        product_fingerprint: 0,
        dependency_indices: vec![],
    });
    image.query_image.supported_roots.push(QueryRootEntryV3 {
        key: PortableQueryKeyV1 {
            kind: PortableQueryKindV1::EnumDeclaration,
            owner: PortableQueryOwnerKeyV1::Declaration(stable_enum_owner.clone()),
        },
        product: QueryProductRefV2 {
            kind: PortableQueryKindV1::EnumDeclaration,
            row_index: 0,
        },
        input_fingerprint: 0,
        product_fingerprint: 0,
        dependency_indices: vec![],
    });
    image.query_image.supported_roots.push(QueryRootEntryV3 {
        key: PortableQueryKeyV1 {
            kind: PortableQueryKindV1::AssociatedSurface,
            owner: PortableQueryOwnerKeyV1::Declaration(stable_owner.clone()),
        },
        product: QueryProductRefV2 {
            kind: PortableQueryKindV1::AssociatedSurface,
            row_index: 0,
        },
        input_fingerprint: 0,
        product_fingerprint: 0,
        dependency_indices: vec![],
    });
    image.query_image.dependency_evidence.push(DependencyEvidenceEntryV3 {
        key: PortableQueryKeyV1 {
            kind: PortableQueryKindV1::DataDeclaration,
            owner: PortableQueryOwnerKeyV1::Declaration(stable_owner.clone()),
        },
        stable_aggregate_owner: PortableQueryOwnerKeyV1::Declaration(stable_owner.clone()),
        product_fingerprint: 0,
    });
    image
        .query_image
        .refresh_portable_structural_fingerprints(&image.semantic_world)
        .expect("refresh portable structural fingerprints");

    let data_portable_product_fingerprint = image
        .query_image
        .supported_roots
        .iter()
        .find(|root| root.key.kind == PortableQueryKindV1::DataDeclaration)
        .expect("data query root")
        .product_fingerprint;

    let bytes = encode_artifact(&image).expect("encode fingerprinted structural artifact");
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget).expect("validate fingerprinted structural artifact");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(payload, &limits, &mut import_budget).expect("import fingerprinted structural product");
    assert_eq!(
        imported.query_products.len(),
        3,
        "DataDeclaration, EnumDeclaration, and AssociatedSurface must be staged as typed reusable products"
    );
    assert_eq!(
        imported.query_products.validated_evidence_len(),
        1,
        "dependency-only DataDeclaration evidence is retained but not counted as reusable root authority"
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
    let associated_query_key = phalcom_semantic::db::QueryKey::AssociatedSurface(owner.clone());
    let associated = db
        .product(&associated_query_key)
        .and_then(|product| product.as_associated_surface())
        .expect("typed AssociatedSurface product must be Ready");
    assert_eq!(associated.owner, owner);
    assert!(
        associated
            .families
            .contains_key(&phalcom_common::selector::SelectorBase::Named("Construct".to_string())),
        "reconstructed associated namespace must retain the data constructor family"
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
    let stale = decode_and_validate_artifact(&stale_bytes, &limits, None, &mut stale_decode_budget);
    assert!(
        stale.is_err(),
        "stale portable query product fingerprint must fail during cross-section artifact validation"
    );
    assert_eq!(
        stale_decode_budget.requested(),
        0,
        "failed portable fingerprint validation must not commit decode allocations"
    );

    let data_root = image
        .query_image
        .supported_roots
        .iter_mut()
        .find(|root| root.key.kind == PortableQueryKindV1::DataDeclaration)
        .expect("data query root");
    data_root.product_fingerprint = data_portable_product_fingerprint;
    image.query_image.dependency_evidence[0].stable_aggregate_owner =
        PortableQueryOwnerKeyV1::Declaration(StableDeclarationRef {
            module: StableModuleRef {
                project: StableProjectRef::Builtin {
                    namespace: "universe".into(),
                    version: "0.1.0".into(),
                },
                path: vec!["core".into()].into_boxed_slice(),
            },
            path: vec!["OtherPayload".into()].into_boxed_slice(),
        });

    let mismatched_bytes = encode_artifact(&image).expect("encode mismatched dependency owner artifact");
    let mut mismatch_decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let mismatch = decode_and_validate_artifact(&mismatched_bytes, &limits, None, &mut mismatch_decode_budget);
    assert!(
        mismatch.is_err(),
        "dependency evidence must bind to the exact structural aggregate owner during cross-section validation"
    );
    assert_eq!(
        mismatch_decode_budget.requested(),
        0,
        "mismatched dependency evidence must fail before committing decode allocations"
    );
}

#[test]
fn p7_type_member_product_imports_as_current_main_ready_query() {
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
        path: vec!["TypeOwner".into()].into_boxed_slice(),
    };
    image
        .semantic_world
        .type_member_declarations
        .push(TypeMemberDeclarationRecordV1 {
            owner: stable_owner.clone(),
            selector: TypeMemberSelectorV1::Named {
                name: "Element".into(),
            },
            parameters: vec![],
            result_domain: TypeMemberDomainV1::Kind {
                kind: ArtifactKindId(0),
            },
            intrinsic: None,
            implementation: TypeMemberImplementationV1::Phalcom,
        });

    image.query_image.supported_roots.push(QueryRootEntryV3 {
        key: PortableQueryKeyV1 {
            kind: PortableQueryKindV1::TypeMemberDeclaration,
            owner: PortableQueryOwnerKeyV1::TypeMember(StableTypeMemberRef {
                owner: stable_owner,
                selector: "Element".into(),
            }),
        },
        product: QueryProductRefV2 {
            kind: PortableQueryKindV1::TypeMemberDeclaration,
            row_index: 0,
        },
        input_fingerprint: 0,
        product_fingerprint: 0,
        dependency_indices: vec![],
    });
    let type_member_owner = PortableQueryOwnerKeyV1::TypeMember(StableTypeMemberRef {
        owner: StableDeclarationRef {
            module: StableModuleRef {
                project: StableProjectRef::Builtin {
                    namespace: "universe".into(),
                    version: "0.1.0".into(),
                },
                path: vec!["core".into()].into_boxed_slice(),
            },
            path: vec!["TypeOwner".into()].into_boxed_slice(),
        },
        selector: "Element".into(),
    });
    image.query_image.dependency_evidence.push(DependencyEvidenceEntryV3 {
        key: PortableQueryKeyV1 {
            kind: PortableQueryKindV1::TypeMemberDeclaration,
            owner: type_member_owner.clone(),
        },
        stable_aggregate_owner: type_member_owner,
        product_fingerprint: 0,
    });
    image
        .query_image
        .dependency_evidence
        .sort();
    image
        .query_image
        .supported_roots
        .sort_by(|left, right| left.key.cmp(&right.key));
    image
        .query_image
        .refresh_portable_structural_fingerprints(&image.semantic_world)
        .expect("refresh type-member portable fingerprint");

    let bytes = encode_artifact(&image).expect("encode type-member artifact");
    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget)
        .expect("validate type-member artifact");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(payload, &limits, &mut import_budget)
        .expect("import P7 type-member semantic product");

    assert_eq!(imported.associated_authority.type_member_len(), 1);
    assert_eq!(imported.query_products.len(), 1, "only the reusable type-member root is reconstructed");
    assert_eq!(
        imported.query_products.validated_evidence_len(),
        1,
        "P7 type-member dependency evidence is validated and retained without becoming an extra reusable root"
    );
    let module = phalcom_modules::ModuleId::universe(phalcom_modules::ModulePath::from_components(vec![
        phalcom_modules::ModuleComponent::from_identifier("core").expect("valid module component"),
    ]));
    let owner = phalcom_semantic::identity::DeclarationId::new(module, "TypeOwner".into());
    let id = phalcom_semantic::types::type_member::TypeMemberDeclarationId::new(
        owner,
        phalcom_semantic::types::type_member::TypeMemberSelector::Named("Element".into()),
    );

    let mut db = phalcom_semantic::db::SemanticDb::new();
    imported
        .publish_query_products_into_empty_db(&mut db)
        .expect("publish imported type-member query");
    let product = db
        .product(&phalcom_semantic::db::QueryKey::TypeMemberDeclaration(id.clone()))
        .and_then(|product| product.as_type_member_declaration())
        .expect("typed TypeMemberDeclaration product must be Ready");
    assert_eq!(product.id, id);
    assert!(product.signature.parameters.is_empty());
    assert!(matches!(
        product.signature.result_domain,
        phalcom_semantic::types::type_member::TypeMemberDomain::Kind(phalcom_semantic::types::id::KindId::TYPE)
    ));
}

#[test]
fn p7_static_case_catalog_reconstructs_as_semantic_authority_without_ready_query() {
    let mut image = sample_image();
    image.module_world = sample_module_world_with_core();
    let owner = StableDeclarationRef {
        module: StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "universe".into(),
                version: "0.1.0".into(),
            },
            path: vec!["core".into()].into_boxed_slice(),
        },
        path: vec!["StaticChoice".into()].into_boxed_slice(),
    };
    image
        .semantic_world
        .type_graph
        .static_domains
        .push(StaticDomainV1::Nominal {
            declaration: owner.clone(),
        });
    image
        .semantic_world
        .static_case_catalogs
        .push(StaticDomainCaseCatalogRecordV1 {
            domain: StaticCaseDomainV1::Nominal {
                declaration: owner,
            },
            cases: vec![StaticCaseHeaderV1 {
                selector: ExactSelectorV1 {
                    base: TypeMemberSelectorV1::Named {
                        name: "Ready".into(),
                    },
                    kind: SelectorKindV1::Getter,
                    slots: vec![],
                },
                shape: StaticCaseShapeV1::Singleton,
                source: SpanV1 { start: 4, end: 9 },
            }],
        });

    let bytes = encode_artifact(&image).expect("encode P7 static-case authority");
    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload =
        decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget).expect("validate P7 static-case authority");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported =
        import_compiler_image(payload, &limits, &mut import_budget).expect("reconstruct P7 static-case authority");

    assert_eq!(imported.static_case_authority.catalog_len(), 1);
    assert_eq!(imported.static_case_authority.signature_len(), 0);
    assert_eq!(
        imported.query_products.len(),
        0,
        "source-backed static-case input identity must not be fabricated into a reusable Ready query"
    );
}

#[test]
fn p7_callable_signature_reconstructs_as_semantic_authority_without_ready_query() {
    let mut image = sample_image();
    image.module_world = sample_module_world_with_core();
    let owner = StableDeclarationRef {
        module: StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "universe".into(),
                version: "0.1.0".into(),
            },
            path: vec!["core".into()].into_boxed_slice(),
        },
        path: vec!["CallableOwner".into()].into_boxed_slice(),
    };
    let callable = StableCallableRef {
        owner: StableCallableOwnerRef::Declaration(owner.clone()),
        side: StableDispatchSide::Instance,
        selector: "run".into(),
    };
    let signature_record = CallableSignatureRecordV1 {
        callable: callable.clone(),
        owner,
        side: 0,
        selector: "run".into(),
        generics: None,
        parameters: vec![],
        declared_return: DeclaredTypeFactV1 {
            state: DeclaredTypeStateV1::Unknown {
                reason: UnknownReasonV1::NoTypeEvidence,
            },
            basis: DeclaredTypeBasisV1::Unspecified,
        },
        return_validation: ReturnContractValidationV1::NotApplicable,
        inferred_return: None,
        source: None,
        implementation: ImplementationKindV1::Phalcom,
        native_id: None,
        effects: EffectSpecV1::Unknown,
        raises: RaisesSpecV1::Unknown,
        flow: ReturnFlowV1::Unknown,
        lifecycle: NativeLifecycleSpecV1 {
            since: None,
            deprecated_since: None,
            replacement: None,
        },
    };
    image.semantic_world.callable_signatures.push(signature_record.clone());
    image.semantic_world.callable_definitions.push(CallableDefinitionRecordV1 {
        callable: signature_record.callable.clone(),
        origin: CallableDefinitionOriginV1::PrimaryDeclaration,
        source_member_index: 0,
        signature: signature_record,
        body_provenance: CallableDefinitionBodyProvenanceV1::SourceBody,
        inherent_body_context: None,
    });
    image.query_image.supported_roots.push(QueryRootEntryV3 {
        key: PortableQueryKeyV1 {
            kind: PortableQueryKindV1::CallableDefinition,
            owner: PortableQueryOwnerKeyV1::Callable(callable),
        },
        product: QueryProductRefV2 {
            kind: PortableQueryKindV1::CallableDefinition,
            row_index: 0,
        },
        input_fingerprint: 0,
        product_fingerprint: 0,
        dependency_indices: vec![],
    });
    image.query_image.supported_roots.sort_by(|left, right| left.key.cmp(&right.key));
    image
        .query_image
        .refresh_portable_structural_fingerprints(&image.semantic_world)
        .expect("refresh P7 callable-definition portable evidence");

    let bytes = encode_artifact(&image).expect("encode P7 callable-signature authority");
    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload =
        decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget).expect("validate P7 callable-signature authority");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported =
        import_compiler_image(payload, &limits, &mut import_budget).expect("reconstruct P7 callable-signature authority");

    assert_eq!(imported.callable_authority.signature_len(), 1);
    assert_eq!(imported.callable_authority.definition_len(), 1);
    assert_eq!(
        imported.query_products.len(),
        1,
        "zero-dependency CallableDefinition may become Ready, while CallableSignature remains semantic-only until its dependency closure is imported"
    );
    let mut db = phalcom_semantic::db::SemanticDb::new();
    imported
        .publish_query_products_into_empty_db(&mut db)
        .expect("publish imported callable definition");
    assert!(
        db.query_keys()
            .any(|key| matches!(key, phalcom_semantic::db::QueryKey::CallableDefinition(_))),
        "the reusable root must publish the exact current-main CallableDefinition family"
    );
}

#[test]
fn p7_enum_requirements_reconstruct_as_snapshot_authority_without_ready_query() {
    let mut image = sample_image();
    image.module_world = sample_module_world_with_core();

    let enum_owner = StableDeclarationRef {
        module: StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "universe".into(),
                version: "0.1.0".into(),
            },
            path: vec!["core".into()].into_boxed_slice(),
        },
        path: vec!["RequirementEnum".into()].into_boxed_slice(),
    };
    image.semantic_world.type_graph.types.push(TypeEntryV1 {
        kind: ArtifactKindId(0),
        form: TypeV1::Nominal {
            declaration: enum_owner.clone(),
        },
    });
    image.semantic_world.structural_contracts.enum_contracts.push(EnumInfoV1 {
        owner: enum_owner.clone(),
        root_form: ArtifactTypeId(1),
        generic_signature: None,
        static_data: None,
        default_result_type: ArtifactTypeId(1),
        variants: vec![],
    });

    let requirement = StableCallableRef {
        owner: StableCallableOwnerRef::Declaration(enum_owner.clone()),
        side: StableDispatchSide::Instance,
        selector: "render".into(),
    };
    let signature = CallableSignatureRecordV1 {
        callable: requirement.clone(),
        owner: enum_owner.clone(),
        side: 0,
        selector: "render".into(),
        generics: None,
        parameters: vec![],
        declared_return: DeclaredTypeFactV1 {
            state: DeclaredTypeStateV1::Unknown {
                reason: UnknownReasonV1::NoTypeEvidence,
            },
            basis: DeclaredTypeBasisV1::Unspecified,
        },
        return_validation: ReturnContractValidationV1::NotApplicable,
        inferred_return: None,
        source: None,
        implementation: ImplementationKindV1::Phalcom,
        native_id: None,
        effects: EffectSpecV1::Unknown,
        raises: RaisesSpecV1::Unknown,
        flow: ReturnFlowV1::Unknown,
        lifecycle: NativeLifecycleSpecV1 {
            since: None,
            deprecated_since: None,
            replacement: None,
        },
    };
    image.semantic_world.callable_signatures.push(signature.clone());
    image.semantic_world.enum_requirements.push(EnumRequirementOwnerV1 {
        owner: enum_owner,
        requirements: vec![EnumRequirementRecordV1 {
            requirement,
            signature,
            source_order: 0,
            source: None,
        }],
        case_statuses: vec![],
    });

    let bytes = encode_artifact(&image).expect("encode P7 enum-requirement authority");
    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget)
        .expect("validate P7 enum-requirement authority");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(payload, &limits, &mut import_budget)
        .expect("reconstruct P7 enum-requirement authority");

    assert_eq!(imported.enum_requirement_authority.owner_len(), 1);
    assert_eq!(imported.enum_requirement_authority.requirement_len(), 1);
    assert_eq!(imported.enum_requirement_authority.case_status_len(), 0);
    assert_eq!(
        imported.query_products.len(),
        0,
        "EnumRequirements is snapshot authority only and must not be fabricated into a reusable Ready query"
    );
}

#[test]
fn p7_constructor_initialization_reconstructs_as_snapshot_authority_without_ready_query() {
    let mut image = sample_image();
    image.module_world = sample_module_world_with_core();

    let owner = StableDeclarationRef {
        module: StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "universe".into(),
                version: "0.1.0".into(),
            },
            path: vec!["core".into()].into_boxed_slice(),
        },
        path: vec!["Constructed".into()].into_boxed_slice(),
    };
    let factory = StableCallableRef {
        owner: StableCallableOwnerRef::Declaration(owner.clone()),
        side: StableDispatchSide::Class,
        selector: "new".into(),
    };
    let initializer = StableCallableRef {
        owner: StableCallableOwnerRef::Declaration(owner.clone()),
        side: StableDispatchSide::Instance,
        selector: "initialize".into(),
    };
    let signature = |callable: StableCallableRef, side: u8, selector: &str| CallableSignatureRecordV1 {
        callable,
        owner: owner.clone(),
        side,
        selector: selector.into(),
        generics: None,
        parameters: vec![],
        declared_return: DeclaredTypeFactV1 {
            state: DeclaredTypeStateV1::Unknown {
                reason: UnknownReasonV1::NoTypeEvidence,
            },
            basis: DeclaredTypeBasisV1::Unspecified,
        },
        return_validation: ReturnContractValidationV1::NotApplicable,
        inferred_return: None,
        source: None,
        implementation: ImplementationKindV1::Phalcom,
        native_id: None,
        effects: EffectSpecV1::Unknown,
        raises: RaisesSpecV1::Unknown,
        flow: ReturnFlowV1::Unknown,
        lifecycle: NativeLifecycleSpecV1 {
            since: None,
            deprecated_since: None,
            replacement: None,
        },
    };
    image
        .semantic_world
        .callable_signatures
        .push(signature(factory.clone(), 1, "new"));
    image
        .semantic_world
        .callable_signatures
        .push(signature(initializer.clone(), 0, "initialize"));
    image.semantic_world.constructor_initializations.push(ConstructorInitializationRecordV1 {
        factory,
        initializer,
        origin: 1,
        field_input_fingerprint: 0xA11CE,
        default_fields: vec![],
        parameter_fields: vec![],
    });

    let bytes = encode_artifact(&image).expect("encode P7 constructor authority");
    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget)
        .expect("validate P7 constructor authority");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(payload, &limits, &mut import_budget)
        .expect("reconstruct P7 constructor authority");

    assert_eq!(imported.constructor_initialization_authority.len(), 1);
    assert_eq!(
        imported.query_products.len(),
        0,
        "ConstructorInitialization remains snapshot authority until its current query reuse contract is explicitly imported"
    );
}

#[test]
fn p7_declaration_dispatch_reconstructs_as_budgeted_snapshot_authority() {
    let mut image = sample_image();
    image.module_world = sample_module_world_with_core();

    let owner = StableDeclarationRef {
        module: StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "universe".into(),
                version: "0.1.0".into(),
            },
            path: vec!["core".into()].into_boxed_slice(),
        },
        path: vec!["DispatchOwner".into()].into_boxed_slice(),
    };
    let callable = StableCallableRef {
        owner: StableCallableOwnerRef::Declaration(owner.clone()),
        side: StableDispatchSide::Instance,
        selector: "run".into(),
    };
    image.semantic_world.callable_signatures.push(CallableSignatureRecordV1 {
        callable: callable.clone(),
        owner: owner.clone(),
        side: 0,
        selector: "run".into(),
        generics: None,
        parameters: vec![],
        declared_return: DeclaredTypeFactV1 {
            state: DeclaredTypeStateV1::Unknown {
                reason: UnknownReasonV1::NoTypeEvidence,
            },
            basis: DeclaredTypeBasisV1::Unspecified,
        },
        return_validation: ReturnContractValidationV1::NotApplicable,
        inferred_return: None,
        source: None,
        implementation: ImplementationKindV1::Phalcom,
        native_id: None,
        effects: EffectSpecV1::Unknown,
        raises: RaisesSpecV1::Unknown,
        flow: ReturnFlowV1::Unknown,
        lifecycle: NativeLifecycleSpecV1 {
            since: None,
            deprecated_since: None,
            replacement: None,
        },
    });
    image.semantic_world.dispatch_surfaces.push(DeclarationSurfaceRecordV1 {
        declaration: owner.clone(),
        instance_callables: vec![DispatchCallableFactV1 {
            callable,
            knowledge: TypeKnowledgeV1::Unknown {
                reason: UnknownReasonV1::NoTypeEvidence,
            },
            visibility: VisibilityV1::Public,
        }],
        class_callables: vec![],
        fields: vec![FieldSignatureV1 {
            owner,
            name: "value".into(),
            declared_type: None,
            knowledge: TypeKnowledgeV1::Unknown {
                reason: UnknownReasonV1::NoTypeEvidence,
            },
            mutable: None,
            side: 0,
            visibility: VisibilityV1::Private,
            source: None,
        }],
    });

    let bytes = encode_artifact(&image).expect("encode P7 dispatch authority");
    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget)
        .expect("validate P7 dispatch authority");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(payload, &limits, &mut import_budget)
        .expect("reconstruct P7 dispatch authority");

    assert_eq!(imported.dispatch_authority.surface_len(), 1);
    assert_eq!(imported.dispatch_authority.callable_len(), 1);
    assert_eq!(imported.dispatch_authority.field_len(), 1);
    assert_eq!(imported.dispatch_authority.field_signature_len(), 0);
    assert_eq!(
        imported.query_products.len(),
        0,
        "declaration dispatch staging must not fabricate DeclarationSurface/FieldSignature Ready roots"
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

    let stable_cached_owner = StableDeclarationRef {
        module: StableModuleRef {
            project: stable_package.clone(),
            path: Box::new([]),
        },
        path: vec!["CachedPackagePayload".into()].into_boxed_slice(),
    };
    image.semantic_world.type_graph.types.push(TypeEntryV1 {
        kind: ArtifactKindId(0),
        form: TypeV1::Nominal {
            declaration: stable_cached_owner.clone(),
        },
    });
    image.semantic_world.structural_contracts.data_contracts.push(DataInfoV1 {
        owner: stable_cached_owner.clone(),
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
    image.query_image.supported_roots.push(QueryRootEntryV3 {
        key: PortableQueryKeyV1 {
            kind: PortableQueryKindV1::DataDeclaration,
            owner: PortableQueryOwnerKeyV1::Declaration(stable_cached_owner),
        },
        product: QueryProductRefV2 {
            kind: PortableQueryKindV1::DataDeclaration,
            row_index: 0,
        },
        input_fingerprint: 0,
        product_fingerprint: 0,
        dependency_indices: vec![],
    });
    image
        .query_image
        .refresh_portable_structural_fingerprints(&image.semantic_world)
        .expect("refresh package query fingerprint");

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
    assert_eq!(imported.query_products.len(), 1, "package-owned DataDeclaration root must survive portable fingerprint validation");
    let package_owner = phalcom_semantic::identity::DeclarationId::new(
        phalcom_modules::ModuleId::resolved(project_id, phalcom_modules::ModulePath::root()),
        "CachedPackagePayload".into(),
    );
    let mut package_db = phalcom_semantic::db::SemanticDb::new();
    imported
        .publish_query_products_into_empty_db(&mut package_db)
        .expect("publish package-owned query product");
    assert!(
        package_db
            .product(&phalcom_semantic::db::QueryKey::DataDeclaration(package_owner.clone()))
            .and_then(|product| product.as_data_declaration())
            .is_some(),
        "first receiving graph must publish the package-owned typed product"
    );

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
        .load_root(unrelated_manifest.clone())
        .expect("load unrelated persistent project");
    let mut second_import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported_with_unrelated =
        import_compiler_image_bytes_with_project_catalog(&bytes, None, &limits, &mut second_import_budget, &projects_with_unrelated, &resolver)
            .expect("same package artifact imports with an unrelated workspace project");
    assert_eq!(
        imported.project_universe.resolve_stable_project(&stable_package),
        imported_with_unrelated.project_universe.resolve_stable_project(&stable_package),
        "adding an unrelated project after the resolver target preserves its existing graph identity"
    );
    assert_ne!(
        imported.external_world.topology().fingerprint,
        imported_with_unrelated.external_world.topology().fingerprint,
        "live topology fingerprints include the distinct receiving workspace graphs"
    );

    // Build an independent receiving graph with an unrelated project allocated
    // first so the exact same stable package is bound to a different local
    // ResolvedProjectId. Portable query fingerprints must remain valid.
    let second_target_root = temp.path().join("resolver-target-second");
    std::fs::create_dir_all(second_target_root.join("src")).expect("create second source root");
    std::fs::write(
        second_target_root.join("project.toml"),
        "[project]\nname = \"resolver-target-second\"\nnamespace = \"resolver_target_second\"\nversion = \"1.0.0\"\n",
    )
    .expect("write second project manifest");
    std::fs::write(second_target_root.join("src/package.ph"), "").expect("write second package source");

    let mut relocated_projects = phalcom_modules::ProjectUniverse::new();
    relocated_projects
        .load_root(unrelated_manifest)
        .expect("allocate unrelated graph node first");
    let relocated_project_id = relocated_projects
        .load_root(second_target_root.join("project.toml"))
        .expect("load relocated resolver target");
    assert_ne!(project_id, relocated_project_id, "test must exercise different graph-local project IDs");

    let relocated_receipt =
        phalcom_modules::ValidatedPackageAuthorityReceipt::from_trusted_resolver(stable_package.clone(), artifact_digest, relocated_project_id)
            .expect("make relocated trusted resolver receipt");
    let relocated_resolver = Resolver(relocated_receipt);
    let mut relocated_import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let relocated = import_compiler_image_bytes_with_project_catalog(
        &bytes,
        None,
        &limits,
        &mut relocated_import_budget,
        &relocated_projects,
        &relocated_resolver,
    )
    .expect("portable query fingerprint must survive package project-ID relocation");
    assert_eq!(relocated.query_products.len(), 1);

    let relocated_owner = phalcom_semantic::identity::DeclarationId::new(
        phalcom_modules::ModuleId::resolved(relocated_project_id, phalcom_modules::ModulePath::root()),
        "CachedPackagePayload".into(),
    );
    let mut relocated_db = phalcom_semantic::db::SemanticDb::new();
    relocated
        .publish_query_products_into_empty_db(&mut relocated_db)
        .expect("publish relocated package-owned product");
    assert!(
        relocated_db
            .product(&phalcom_semantic::db::QueryKey::DataDeclaration(relocated_owner))
            .and_then(|product| product.as_data_declaration())
            .is_some(),
        "second receiving graph must publish the same portable root under its own local owner"
    );
    assert_ne!(
        package_owner.module.project,
        phalcom_modules::ProjectIdentity::Resolved(relocated_project_id),
        "local DB identities differ even though portable artifact authority is identical"
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
