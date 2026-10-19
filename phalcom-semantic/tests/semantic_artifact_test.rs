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
use phalcom_compiler_artifact::semantic::operational::OperationalOpV1;
use phalcom_compiler_artifact::semantic::type_store::{
    ArtifactKindId, ArtifactTypeId, KindV1, StaticDomainV1, TypeEntryV1, TypeGraphV1, TypeMemberSelectorV1, TypeV1,
};
use phalcom_compiler_artifact::semantic::world::{
    AssociatedFamilyKindV1, AssociatedFamilyRecordV1, AssociatedMemberV1, AssociatedSurfaceRecordV1,
    CallableAnalysisRecordV1, CallableAnalysisSourceOwnerV1, CallableAnalysisStatusV1,
    CallableBindingAuthorityV1, CallableBindingConsistencyV1, CallableBindingRowV1, CallableCausalInvalidityV1,
    CallableDefinitionBodyProvenanceV1, CallableDefinitionOriginV1, CallableDefinitionRecordV1,
    CallableBindingReadSafetyV1, CallableCallSpecializationV1, CallableDiagnosticSeverityV1, CallableEvidenceDiagnosticV1,
    CallableExpressionRowV1, CallableExpressionStatusV1, CallableFlowGraphV1, CallableFlowStateSummaryV1,
    CallableGeneratedBindingReasonV1, CallableInitializationStateV1, CallableSignatureRecordV1, SemanticDenotationV1,
    ConditionalInherentMemberV1, ConditionalInherentSetV1, ConformanceHeadRecordV1, ConformanceTargetV1,
    ConstructorInitializationRecordV1, DeclarationSurfaceRecordV1, DispatchCallableFactV1, EffectSpecV1,
    EnumRequirementOwnerV1, EnumRequirementRecordV1, EnumRequirementSourceV1,
    ExactSelectorV1, FieldSignatureV1, ImplementationKindV1, InherentImplDomainV1, InherentImplTargetV1,
    ModuleStaticBindingIndexV1, ModuleStaticBindingPhaseV1, ModuleStaticBindingRecordV1, ModuleStaticBindingVisibleNameV1,
    NativeLifecycleSpecV1,
    RaisesSpecV1, ReturnContractValidationV1,
    CanonicalStaticProgramV1, ReturnFlowV1, SelectorKindV1, SemanticWorldV3,
    StaticConstantOutcomeV1, StaticConstantRecordV1, StaticConstantSymbolV1, StaticDefinitionRecordV1,
    StaticDefinitionSignatureV1, StaticValueDomainV1, TraitRefV1, TraitSurfaceRecordV1, TypeKnowledgeV1,
    CallableConformanceAssociatedTypePlanV1, CallableConformanceCompletenessV1, CanonicalCheckedStaticApplicationV1, WitnessPlanV1,
    StaticCaseDomainV1, StaticCaseHeaderV1, StaticCaseShapeV1,
    StaticDomainCaseCatalogRecordV1, TypeMemberDeclarationRecordV1, TypeMemberDomainV1, TypeMemberImplementationV1,
};
use phalcom_semantic::artifact::{
    CompilerArtifactImage, ImportedOperationalNode, decode_and_validate_artifact, encode_artifact, import_compiler_image,
    import_compiler_image_bytes_with_project_catalog,
};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::{
    StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide, StableImplRef, StableModuleRef,
    StableProjectRef, StableTypeMemberRef,
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
fn p7_conditional_inherent_dispatch_reconstructs_without_live_selector_index() {
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
        path: vec!["ConditionalOwner".into()].into_boxed_slice(),
    };
    image.semantic_world.type_graph.types.push(TypeEntryV1 {
        kind: ArtifactKindId(0),
        form: TypeV1::Nominal {
            declaration: owner.clone(),
        },
    });

    let implementation = StableImplRef {
        module: owner.module.clone(),
        local_id: 7,
    };
    let target = InherentImplTargetV1::Declaration {
        declaration: owner.clone(),
    };
    let callable = StableCallableRef {
        owner: StableCallableOwnerRef::Declaration(owner.clone()),
        side: StableDispatchSide::Instance,
        selector: "special".into(),
    };
    let signature = CallableSignatureRecordV1 {
        callable: callable.clone(),
        owner: owner.clone(),
        side: 0,
        selector: "special".into(),
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
    image.semantic_world.conditional_inherent_sets.push(ConditionalInherentSetV1 {
        target: target.clone(),
        domains: vec![InherentImplDomainV1 {
            impl_id: implementation.clone(),
            target,
            head_type: ArtifactTypeId(1),
            generic_signature: None,
            constraints: vec![],
        }],
        members: vec![ConditionalInherentMemberV1 {
            impl_id: implementation,
            member_order: 0,
            is_conditionally_applicable: true,
            callable,
            signature,
            visibility: VisibilityV1::Public,
            source_member_index: 3,
            is_requirement: false,
        }],
    });

    let bytes = encode_artifact(&image).expect("encode P7 conditional inherent authority");
    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget)
        .expect("validate P7 conditional inherent authority");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(payload, &limits, &mut import_budget)
        .expect("reconstruct P7 conditional inherent authority");

    assert_eq!(imported.conditional_inherent_authority.set_len(), 1);
    assert_eq!(imported.conditional_inherent_authority.domain_len(), 1);
    assert_eq!(imported.conditional_inherent_authority.member_len(), 1);
    assert_eq!(
        imported.query_products.len(),
        0,
        "conditional inherent staging must not fabricate open-world query authority"
    );
}

#[test]
fn p7_conformance_head_reconstructs_with_current_authorization_check() {
    let mut image = sample_image();
    image.module_world = sample_module_world_with_core();

    let module = StableModuleRef {
        project: StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        },
        path: vec!["core".into()].into_boxed_slice(),
    };
    let target = StableDeclarationRef {
        module: module.clone(),
        path: vec!["ConformanceTarget".into()].into_boxed_slice(),
    };
    let trait_owner = StableDeclarationRef {
        module: module.clone(),
        path: vec!["Protocol".into()].into_boxed_slice(),
    };
    image.semantic_world.type_graph.types.push(TypeEntryV1 {
        kind: ArtifactKindId(0),
        form: TypeV1::Nominal {
            declaration: target.clone(),
        },
    });
    image.semantic_world.conformance_heads.push(ConformanceHeadRecordV1 {
        impl_id: StableImplRef {
            module: module.clone(),
            local_id: 9,
        },
        source_module: module,
        target: ConformanceTargetV1::Declaration {
            declaration: target,
        },
        target_type: ArtifactTypeId(1),
        trait_ref: TraitRefV1 {
            declaration: trait_owner,
            arguments: vec![],
        },
        generic_signature: None,
        authorized: true,
        eligible: true,
    });

    let bytes = encode_artifact(&image).expect("encode P7 conformance head");
    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget)
        .expect("validate P7 conformance head");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(payload, &limits, &mut import_budget)
        .expect("reconstruct P7 conformance head");

    assert_eq!(imported.conformance_authority.len(), 1);
    assert_eq!(
        imported.query_products.len(),
        0,
        "ConformanceDispatch is open-world and must not become unconditional Ready authority"
    );
}

#[test]
fn p7_trait_surface_reconstructs_complete_diagnostic_payload() {
    let mut image = sample_image();
    image.module_world = sample_module_world_with_core();

    let module = StableModuleRef {
        project: StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        },
        path: vec!["core".into()].into_boxed_slice(),
    };
    let declaration = StableDeclarationRef {
        module: module.clone(),
        path: vec!["ProtocolWithDiagnostic".into()].into_boxed_slice(),
    };
    let source = EnumRequirementSourceV1 {
        module: module.clone(),
        range: SpanV1 { start: 4, end: 8 },
    };
    image.semantic_world.trait_surfaces.push(TraitSurfaceRecordV1 {
        declaration,
        generic_signature: None,
        header_source: source.clone(),
        members: vec![],
        associated_types: vec![],
        diagnostics: vec![CallableEvidenceDiagnosticV1 {
            code: "trait.member.conflict".into(),
            severity: CallableDiagnosticSeverityV1::Warning,
            message: "conflicting trait member".into(),
            primary: source,
            primary_range: SpanV1 { start: 4, end: 8 },
            labels: vec![],
            notes: vec!["retained P7 note".into()],
            helps: vec!["retained P7 help".into()],
            guidance: vec![],
            explanations: vec![],
            fixes: vec![],
            root_cause: None,
        }],
    });

    let bytes = encode_artifact(&image).expect("encode P7 trait authority");
    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget)
        .expect("validate P7 trait authority");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(payload, &limits, &mut import_budget)
        .expect("reconstruct P7 trait authority");

    assert_eq!(imported.trait_authority.surface_len(), 1);
    assert_eq!(imported.trait_authority.member_len(), 0);
    assert_eq!(imported.trait_authority.associated_type_len(), 0);
    assert_eq!(imported.trait_authority.diagnostic_len(), 1);
    assert_eq!(
        imported.query_products.len(),
        0,
        "trait snapshot authority must not bypass TraitHeader/TraitSurface dependency closure"
    );
}

#[test]
fn p7_witness_preflight_binds_portable_plan_to_live_conformance_and_trait_authority() {
    let mut image = sample_image();
    image.module_world = sample_module_world_with_core();

    let module = StableModuleRef {
        project: StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        },
        path: vec!["core".into()].into_boxed_slice(),
    };
    let target = StableDeclarationRef {
        module: module.clone(),
        path: vec!["WitnessTarget".into()].into_boxed_slice(),
    };
    let trait_owner = StableDeclarationRef {
        module: module.clone(),
        path: vec!["WitnessProtocol".into()].into_boxed_slice(),
    };
    let implementation = StableImplRef {
        module: module.clone(),
        local_id: 11,
    };

    image.semantic_world.type_graph.types.push(TypeEntryV1 {
        kind: ArtifactKindId(0),
        form: TypeV1::Nominal {
            declaration: target.clone(),
        },
    });
    image.semantic_world.trait_surfaces.push(TraitSurfaceRecordV1 {
        declaration: trait_owner.clone(),
        generic_signature: None,
        header_source: EnumRequirementSourceV1 {
            module: module.clone(),
            range: SpanV1 { start: 0, end: 0 },
        },
        members: vec![],
        associated_types: vec![],
        diagnostics: vec![],
    });
    image.semantic_world.conformance_heads.push(ConformanceHeadRecordV1 {
        impl_id: implementation.clone(),
        source_module: module.clone(),
        target: ConformanceTargetV1::Declaration {
            declaration: target,
        },
        target_type: ArtifactTypeId(1),
        trait_ref: TraitRefV1 {
            declaration: trait_owner.clone(),
            arguments: vec![],
        },
        generic_signature: None,
        authorized: true,
        eligible: true,
    });
    image.semantic_world.witness_plans.push(WitnessPlanV1 {
        impl_id: implementation.clone(),
        source_module: module,
        trait_ref_template: TraitRefV1 {
            declaration: trait_owner,
            arguments: vec![],
        },
        target_template: ArtifactTypeId(1),
        generic_signature: None,
        requirement_views: vec![],
        requirements: vec![],
        associated_type_plan: CallableConformanceAssociatedTypePlanV1 {
            impl_id: implementation,
            bindings: vec![],
            failures: vec![],
            diagnostics: vec![],
        },
        invalid_explicit_members: false,
        completeness: CallableConformanceCompletenessV1::Complete,
        diagnostics: vec![],
        conformance_evidence: vec![],
        fingerprint: 0,
        ordered_selected_callables: vec![],
        slot_layout_fingerprint: [0; 32],
    });

    let bytes = encode_artifact(&image).expect("encode P7 witness preflight");
    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget)
        .expect("validate P7 witness preflight");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(payload, &limits, &mut import_budget)
        .expect("bind P7 witness plan to live authorities");

    assert_eq!(imported.witness_preflight_authority.len(), 1);
    assert_eq!(imported.witness_preflight_authority.requirement_view_len(), 0);
    assert_eq!(imported.witness_preflight_authority.evidence_node_len(), 0);
    assert_eq!(
        imported.query_products.len(),
        0,
        "witness preflight is proof staging and must not make ConformanceDispatch Ready"
    );
}

#[test]
fn p7_callable_binding_state_reconstructs_without_claiming_full_callable_analysis() {
    let mut image = sample_image();
    image.module_world = sample_module_world_with_core();

    let stable_module = StableModuleRef {
        project: StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        },
        path: vec!["core".into()].into_boxed_slice(),
    };
    let owner = StableDeclarationRef {
        module: stable_module.clone(),
        path: vec!["BindingOwner".into()].into_boxed_slice(),
    };
    let callable = StableCallableRef {
        owner: StableCallableOwnerRef::Declaration(owner.clone()),
        side: StableDispatchSide::Instance,
        selector: "run".into(),
    };
    image.semantic_world.callable_signatures.push(CallableSignatureRecordV1 {
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
    });
    image.semantic_world.callable_analyses.push(CallableAnalysisRecordV1 {
        callable: callable.clone(),
        body_source_owner: Some(CallableAnalysisSourceOwnerV1::Callable {
            callable: callable.clone(),
        }),
        body_range: SpanV1 { start: 0, end: 0 },
        status: CallableAnalysisStatusV1::Complete,
        exits_unreachable: false,
        flow_graph: CallableFlowGraphV1 {
            nodes: vec![],
            edges: vec![],
            entry: None,
            exits: vec![],
        },
        constructor_super_initializer: None,
        expressions: vec![CallableExpressionRowV1 {
            ordinal: 0,
            range: SpanV1 { start: 1, end: 2 },
            knowledge: TypeKnowledgeV1::Unknown {
                reason: UnknownReasonV1::NoTypeEvidence,
            },
            callable: Some(callable.clone()),
            conditional_dispatch: None,
            conditional_capability: None,
            trait_dispatch: None,
            trait_dispatch_candidates: None,
            contextual_realization: None,
            status: CallableExpressionStatusV1::Ready,
            causal_invalidity: CallableCausalInvalidityV1::Clean,
            explanation: None,
            binding_read_safety: Some(CallableBindingReadSafetyV1::ProvenInitialized),
            binding_read: Some(1),
            binding_write: None,
            denotation: Some(SemanticDenotationV1::TypeForm {
                ty: ArtifactTypeId(0),
            }),
            call_specialization: Some(CallableCallSpecializationV1 {
                callable: callable.clone(),
                bindings: vec![],
                conformances: vec![],
            }),
        }],
        conformance_evidence: vec![],
        bindings: vec![
            CallableBindingRowV1 {
                ordinal: 0,
                name: "self".into(),
                parameter: None,
                range: SpanV1 { start: 0, end: 0 },
                contract: None,
                current: TypeKnowledgeV1::Unknown {
                    reason: UnknownReasonV1::NoTypeEvidence,
                },
                denotation: Some(SemanticDenotationV1::TypeForm {
                    ty: ArtifactTypeId(0),
                }),
                consistency: CallableBindingConsistencyV1::Unconstrained,
                causal_invalidity: CallableCausalInvalidityV1::Clean,
                mutable: false,
                initialization: CallableInitializationStateV1::DefinitelyInitialized,
                version: 0,
                explanation: None,
                authority: CallableBindingAuthorityV1::BodyLocalGenerated {
                    reason: CallableGeneratedBindingReasonV1::ImplicitReceiver,
                },
            },
            CallableBindingRowV1 {
                ordinal: 1,
                name: "cached".into(),
                parameter: None,
                range: SpanV1 { start: 1, end: 2 },
                contract: None,
                current: TypeKnowledgeV1::Unknown {
                    reason: UnknownReasonV1::NoTypeEvidence,
                },
                denotation: None,
                consistency: CallableBindingConsistencyV1::Unconstrained,
                causal_invalidity: CallableCausalInvalidityV1::Clean,
                mutable: false,
                initialization: CallableInitializationStateV1::DefinitelyInitialized,
                version: 0,
                explanation: None,
                authority: CallableBindingAuthorityV1::BodyLocalSource {
                    site_owner: CallableAnalysisSourceOwnerV1::Callable {
                        callable: callable.clone(),
                    },
                    site_local: 1,
                    declaration_range: SpanV1 { start: 1, end: 2 },
                },
            },
        ],
        for_iteration_lanes: vec![],
        positional_expansions: vec![],
        known_invalid_positional_expansions: vec![],
        sequence_literals: vec![],
        associated_resolutions: vec![],
        callable_reference_resolutions: vec![],
        family_applications: vec![],
        local_proof_graph: None,
        match_resolutions: vec![],
        pattern_site_resolutions: vec![],
        entry_flow: CallableFlowStateSummaryV1 {
            bindings: vec![],
            fields: vec![],
            fact_count: 0,
        },
        normal_returns: vec![],
        throws: vec![],
        return_validation: ReturnContractValidationV1::NotApplicable,
        explanations: vec![],
        return_explanation: None,
        diagnostic_cause_count: 0,
        diagnostics: vec![],
        dependencies: vec![],
        semantic_dependencies: vec![],
    });

    image.semantic_world.module_static_bindings.push(ModuleStaticBindingIndexV1 {
        module: stable_module.clone(),
        bindings: vec![ModuleStaticBindingRecordV1 {
            symbol: StaticConstantSymbolV1 {
                module: stable_module.clone(),
                name: "cached".into(),
            },
            source_body: callable,
            source_binding_ordinal: 1,
            phase: ModuleStaticBindingPhaseV1::StaticConstant,
            declaration_source: SpanV1 { start: 1, end: 2 },
            initializer_source: Some(SpanV1 { start: 2, end: 2 }),
        }],
        visible_names: vec![ModuleStaticBindingVisibleNameV1 {
            name: "cached".into(),
            symbol: StaticConstantSymbolV1 {
                module: stable_module,
                name: "cached".into(),
            },
        }],
    });

    let bytes = encode_artifact(&image).expect("encode P7 callable binding authority");
    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget)
        .expect("validate P7 callable binding authority");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(payload, &limits, &mut import_budget)
        .expect("reconstruct P7 callable binding authority");

    assert_eq!(imported.callable_binding_authority.callable_len(), 1);
    assert_eq!(imported.callable_binding_authority.binding_len(), 2);
    assert_eq!(imported.callable_expression_authority.callable_len(), 1);
    assert_eq!(imported.callable_expression_authority.expression_len(), 1);
    assert_eq!(imported.callable_expression_authority.deferred_expression_len(), 1);
    assert_eq!(imported.callable_conformance_evidence_authority.callable_len(), 1);
    assert_eq!(imported.callable_conformance_evidence_authority.evidence_node_len(), 0);
    assert_eq!(imported.callable_specialization_authority.callable_len(), 1);
    assert_eq!(imported.callable_specialization_authority.specialization_len(), 1);
    let module = phalcom_modules::ModuleId::universe(phalcom_modules::ModulePath::from_components(vec![
        phalcom_modules::ModuleComponent::from_identifier("core").expect("valid module component"),
    ]));
    let callable_id = phalcom_semantic::identity::CallableId::new(
        phalcom_semantic::identity::DeclarationId::new(module, "BindingOwner".into()),
        phalcom_common::selector::Selector::decode("run"),
        phalcom_semantic::identity::DispatchSide::Instance,
    );
    let expression = imported
        .callable_expression_authority
        .expression(&callable_id, 0)
        .expect("relocated base expression");
    assert_eq!(expression.binding_read, Some(phalcom_semantic::identity::BindingId(1)));
    assert!(matches!(expression.status, phalcom_semantic::checker::analysis::AnalysisStatus::Ready));
    assert!(matches!(
        expression.binding_read_safety,
        Some(phalcom_semantic::checker::flow::BindingReadSafety::ProvenInitialized)
    ));
    assert!(matches!(
        expression.denotation,
        Some(phalcom_semantic::types::denotation::SemanticDenotation::TypeForm(_))
    ));
    assert!(expression.deferred.call_specialization);
    let specialization = imported
        .callable_specialization_authority
        .specialization(&callable_id, 0)
        .expect("relocated call specialization");
    assert_eq!(specialization.callable, callable_id);
    assert!(specialization.bindings.is_empty());
    assert!(specialization.conformances.is_empty());
    assert_eq!(imported.module_static_binding_authority.module_len(), 1);
    assert_eq!(imported.module_static_binding_authority.binding_len(), 1);
    assert_eq!(imported.module_static_binding_authority.visible_name_len(), 1);
    assert_eq!(
        imported.query_products.len(),
        0,
        "binding-state staging must not pretend a complete CallableBody query is Ready"
    );
}

#[test]
fn p7_static_constant_reconstructs_typed_authority_without_checked_body_fabrication() {
    let mut image = sample_image();
    image.module_world = sample_module_world_with_core();
    let stable_module = StableModuleRef {
        project: StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        },
        path: vec!["core".into()].into_boxed_slice(),
    };
    image.semantic_world.static_constants.push(StaticConstantRecordV1 {
        symbol: StaticConstantSymbolV1 {
            module: stable_module,
            name: "cachedUnit".into(),
        },
        declaration_source: SpanV1 { start: 4, end: 12 },
        initializer_source: SpanV1 { start: 10, end: 12 },
        input_fingerprint: 0x51A71C,
        dependencies: vec![],
        static_constant_reads: vec![],
        record_key_static_constant_reads: vec![],
        result_type: Some(ArtifactTypeId(0)),
        outcome: StaticConstantOutcomeV1::Reduced {
            value: ArtifactStaticValueId(0),
        },
        lkg_value: Some(ArtifactStaticValueId(0)),
        diagnostics: vec![],
    });

    let bytes = encode_artifact(&image).expect("encode P7 static constant authority");
    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget)
        .expect("validate P7 static constant authority");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(payload, &limits, &mut import_budget)
        .expect("reconstruct P7 static constant authority");

    assert_eq!(imported.static_constant_authority.len(), 1);
    assert_eq!(
        imported.query_products.len(),
        0,
        "static-constant authority must not invent a CheckedStaticBody or reusable StaticConstant query root"
    );
}

#[test]
fn p7_static_definition_preserves_source_free_operational_program_without_ast() {
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
        path: vec!["CachedDefinition".into()].into_boxed_slice(),
    };
    let signature = StaticDefinitionSignatureV1 {
        parameters: vec![],
        result_domain: StaticValueDomainV1::ValueType {
            ty: ArtifactTypeId(0),
        },
        generic_signature: None,
    };
    image.semantic_world.static_definitions.push(StaticDefinitionRecordV1 {
        id: owner.clone(),
        signature,
        source: SpanV1 { start: 20, end: 32 },
        alias_info: None,
        canonical_program: CanonicalStaticProgramV1 {
            owner: owner.clone(),
            parameter_count: 0,
            local_count: 0,
            root: OperationalOpV1::Constant {
                value: ArtifactStaticValueId(0),
            },
            applications: vec![],
            kinded_parameters: vec![],
            static_constant_reads: vec![],
            dependencies: vec![],
            result: StaticValueDomainV1::ValueType {
                ty: ArtifactTypeId(0),
            },
            input_fingerprint: 0x57A71C,
            module_binding_fingerprint: None,
        },
        diagnostics: vec![],
    });

    let bytes = encode_artifact(&image).expect("encode P7 static definition authority");
    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget)
        .expect("validate P7 static definition authority");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(payload, &limits, &mut import_budget)
        .expect("reconstruct P7 static definition authority");

    assert_eq!(imported.static_definition_authority.len(), 1);
    let module = phalcom_modules::ModuleId::universe(phalcom_modules::ModulePath::from_components(vec![
        phalcom_modules::ModuleComponent::from_identifier("core").expect("valid module component"),
    ]));
    let id = phalcom_semantic::identity::DeclarationId::new(module, "CachedDefinition".into());
    let definition = imported
        .static_definition_authority
        .get(&id)
        .expect("relocated static definition");
    assert_eq!(definition.canonical_program.owner.0, id);
    assert!(matches!(
        definition.canonical_program.root,
        ImportedOperationalNode::Constant(_)
    ));
    assert_eq!(
        imported.query_products.len(),
        0,
        "source-free operational authority must not fabricate an AST-backed StaticDefinition query root"
    );
}

#[test]
fn p7_static_application_denotation_binds_to_imported_checked_application_identity() {
    let mut image = sample_image();
    image.module_world = sample_module_world_with_core();

    let stable_module = StableModuleRef {
        project: StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        },
        path: vec!["core".into()].into_boxed_slice(),
    };
    let static_owner = StableDeclarationRef {
        module: stable_module.clone(),
        path: vec!["StaticBody".into()].into_boxed_slice(),
    };
    let static_signature = StaticDefinitionSignatureV1 {
        parameters: vec![],
        result_domain: StaticValueDomainV1::ValueType {
            ty: ArtifactTypeId(0),
        },
        generic_signature: None,
    };
    image.semantic_world.static_definitions.push(StaticDefinitionRecordV1 {
        id: static_owner.clone(),
        signature: static_signature.clone(),
        source: SpanV1 { start: 40, end: 52 },
        alias_info: None,
        canonical_program: CanonicalStaticProgramV1 {
            owner: static_owner.clone(),
            parameter_count: 0,
            local_count: 0,
            root: OperationalOpV1::Constant {
                value: ArtifactStaticValueId(0),
            },
            applications: vec![CanonicalCheckedStaticApplicationV1 {
                ordinal: 0,
                source: SpanV1 { start: 44, end: 48 },
                target: static_owner.clone(),
                signature: static_signature,
                arguments: vec![],
            }],
            kinded_parameters: vec![],
            static_constant_reads: vec![],
            dependencies: vec![],
            result: StaticValueDomainV1::ValueType {
                ty: ArtifactTypeId(0),
            },
            input_fingerprint: 0xA770,
            module_binding_fingerprint: None,
        },
        diagnostics: vec![],
    });

    let callable_owner = StableDeclarationRef {
        module: stable_module,
        path: vec!["StaticUser".into()].into_boxed_slice(),
    };
    let callable = StableCallableRef {
        owner: StableCallableOwnerRef::Declaration(callable_owner.clone()),
        side: StableDispatchSide::Instance,
        selector: "run".into(),
    };
    image.semantic_world.callable_signatures.push(CallableSignatureRecordV1 {
        callable: callable.clone(),
        owner: callable_owner,
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
    image.semantic_world.callable_analyses.push(CallableAnalysisRecordV1 {
        callable: callable.clone(),
        body_source_owner: Some(CallableAnalysisSourceOwnerV1::Callable {
            callable: callable.clone(),
        }),
        body_range: SpanV1 { start: 0, end: 1 },
        status: CallableAnalysisStatusV1::Complete,
        exits_unreachable: false,
        flow_graph: CallableFlowGraphV1 {
            nodes: vec![],
            edges: vec![],
            entry: None,
            exits: vec![],
        },
        constructor_super_initializer: None,
        expressions: vec![],
        conformance_evidence: vec![],
        bindings: vec![CallableBindingRowV1 {
            ordinal: 0,
            name: "resolved".into(),
            parameter: None,
            range: SpanV1 { start: 0, end: 1 },
            contract: None,
            current: TypeKnowledgeV1::Unknown {
                reason: UnknownReasonV1::NoTypeEvidence,
            },
            denotation: Some(SemanticDenotationV1::StaticApplication {
                body: static_owner.clone(),
                ordinal: 0,
            }),
            consistency: CallableBindingConsistencyV1::Unconstrained,
            causal_invalidity: CallableCausalInvalidityV1::Clean,
            mutable: false,
            initialization: CallableInitializationStateV1::DefinitelyInitialized,
            version: 0,
            explanation: None,
            authority: CallableBindingAuthorityV1::BodyLocalGenerated {
                reason: CallableGeneratedBindingReasonV1::ImplicitReceiver,
            },
        }],
        for_iteration_lanes: vec![],
        positional_expansions: vec![],
        known_invalid_positional_expansions: vec![],
        sequence_literals: vec![],
        associated_resolutions: vec![],
        callable_reference_resolutions: vec![],
        family_applications: vec![],
        local_proof_graph: None,
        match_resolutions: vec![],
        pattern_site_resolutions: vec![],
        entry_flow: CallableFlowStateSummaryV1 {
            bindings: vec![],
            fields: vec![],
            fact_count: 0,
        },
        normal_returns: vec![],
        throws: vec![],
        return_validation: ReturnContractValidationV1::NotApplicable,
        explanations: vec![],
        return_explanation: None,
        diagnostic_cause_count: 0,
        diagnostics: vec![],
        dependencies: vec![],
        semantic_dependencies: vec![],
    });

    let bytes = encode_artifact(&image).expect("encode P7 static-application denotation");
    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let payload = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget)
        .expect("validate P7 static-application denotation");
    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(payload, &limits, &mut import_budget)
        .expect("import P7 static-application denotation");

    let module = phalcom_modules::ModuleId::universe(phalcom_modules::ModulePath::from_components(vec![
        phalcom_modules::ModuleComponent::from_identifier("core").expect("valid module component"),
    ]));
    let callable_id = phalcom_semantic::identity::CallableId::new(
        phalcom_semantic::identity::DeclarationId::new(module.clone(), "StaticUser".into()),
        phalcom_common::selector::Selector::decode("run"),
        phalcom_semantic::identity::DispatchSide::Instance,
    );
    let binding = imported
        .callable_binding_authority
        .binding(&callable_id, 0)
        .expect("imported callable binding");
    let phalcom_semantic::types::denotation::SemanticDenotation::StaticApplication(application) =
        binding.state.denotation.as_ref().expect("static application denotation")
    else {
        panic!("binding denotation must remain a checked static application");
    };
    assert_eq!(application.ordinal, 0);
    assert_eq!(
        application.body.0,
        phalcom_semantic::identity::DeclarationId::new(module, "StaticBody".into())
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
