use phalcom_compiler_artifact::executable::compatibility::{CompilationProfileV1, ExecutableCompatibilityV1};
use phalcom_compiler_artifact::executable::constant::{
    ConstantRecipeV1, ContractRecipeV1, MethodBodyV1, MethodOriginV1, MethodRecipeV1, MethodSignatureKindV1, MethodSignatureV1, SelectorBaseV1,
    SelectorKindPatternV1, SelectorKindV1, SelectorPatternV1, SelectorSlotV1, StableClassRefV1, VisibilityV1,
};
use phalcom_compiler_artifact::executable::image::{ExecutableImageLimitsV1, ExecutableImageV1};
use phalcom_compiler_artifact::executable::instruction::PortableInstructionV1;
use phalcom_compiler_artifact::executable::module::{PortableBootstrapModuleV1, PortableBootstrapPlanV1, PortableModuleV1, PortableSourceMapV1};
use phalcom_compiler_artifact::executable::module::{PortableClassAllocationV1, PortableClassRecipeV1};
use phalcom_compiler_artifact::executable::prototype::{
    ConstantId, ExecutablePrototypeLimitsV1, ParameterShapeV1, PortablePrototypeV1, PrototypeId, PrototypeOwnerV1, RestKindV1, SemanticPoolId, SourceMapId,
    UpvalueCaptureV1,
};
use phalcom_compiler_artifact::executable::semantic::PortableSemanticPoolV1;
use phalcom_compiler_artifact::executable::staging::StagedExecutableSectionV1;
use phalcom_compiler_artifact::module_world::SpanV1;
use phalcom_compiler_artifact::{
    ArtifactDigest, ArtifactError, BinaryReader, CodecError, SectionFlags, SectionKind, TELOSC_CONTAINER_SCHEMA_V2, TELOSC_MIN_READER_SCHEMA_V2,
    TeloscCapabilities, TeloscContainerBuilder, TeloscContainerView, TeloscHeaderV2, TeloscPackageIdentity, ValidationLimits,
};
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::identity::{StableDeclarationRef, StableDispatchSide, StableModuleRef, StableProjectRef};

fn module_ref() -> StableModuleRef {
    StableModuleRef {
        project: StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        },
        path: Box::new(["test".into()]),
    }
}

fn empty_semantic_pool() -> PortableSemanticPoolV1 {
    PortableSemanticPoolV1 {
        type_recipes: Vec::new(),
        kind_recipes: Vec::new(),
        enum_specs: Vec::new(),
        variant_targets: Vec::new(),
        data_specs: Vec::new(),
        data_targets: Vec::new(),
        data_constructions: Vec::new(),
        anonymous_products: Vec::new(),
        call_environments: Vec::new(),
        associated_targets: Vec::new(),
        family_descriptors: Vec::new(),
        conditional_families: Vec::new(),
        family_operations: Vec::new(),
        family_candidate_sets: Vec::new(),
        trait_invocations: Vec::new(),
        trait_requirement_invocations: Vec::new(),
        type_parameter_class_receivers: Vec::new(),
        provenance: None,
    }
}

fn image() -> ExecutableImageV1 {
    let module = module_ref();
    let native_anchors = Vec::new();
    let primordial_roles = Vec::new();
    ExecutableImageV1 {
        compatibility: ExecutableCompatibilityV1::current(CompilationProfileV1::Debug, &native_anchors, &primordial_roles).unwrap(),
        modules: vec![PortableModuleV1 {
            identity: module.clone(),
            initializer: PrototypeId(0),
            linked_reads: Vec::new(),
            source: SourceMapId(0),
            eager: true,
        }],
        prototypes: vec![PortablePrototypeV1 {
            owner: PrototypeOwnerV1::ModuleInitializer { module: module.clone() },
            code: vec![PortableInstructionV1::Return {}],
            constants: Vec::<ConstantId>::new(),
            max_slots: 1,
            arity: 0,
            upvalues: Vec::new(),
            parameters: ParameterShapeV1 {
                fixed_positionals: 0,
                fixed_labels: Vec::new(),
                rest: None::<RestKindV1>,
            },
            name: "test initializer".into(),
            local_names: Vec::new(),
            source: SourceMapId(0),
            semantics: SemanticPoolId(0),
            spans: vec![SpanV1 { start: 0, end: 0 }],
        }],
        constants: Vec::new(),
        semantic_pools: vec![empty_semantic_pool()],
        classes: Vec::new(),
        native_anchors,
        primordial_roles,
        source_maps: vec![PortableSourceMapV1 {
            logical_uri: "universe://test.ph".into(),
            module: module.clone(),
            line_starts: vec![0],
            source_span: Some(SpanV1 { start: 0, end: 0 }),
        }],
        detached_methods: Vec::new(),
        bootstrap: PortableBootstrapPlanV1 {
            native_runtime_roles: Vec::new(),
            eager_modules: vec![PortableBootstrapModuleV1 {
                module,
                initializer: PrototypeId(0),
            }],
            deferred_modules: Vec::new(),
            required_declarations: Vec::new(),
        },
    }
}

fn container_header(capabilities: TeloscCapabilities) -> TeloscHeaderV2 {
    TeloscHeaderV2 {
        container_schema: TELOSC_CONTAINER_SCHEMA_V2,
        minimum_reader_schema: TELOSC_MIN_READER_SCHEMA_V2,
        package_identity: TeloscPackageIdentity::universe(),
        semantic_model_version: 1,
        stable_identity_schema: phalcom_compiler_artifact::header::STABLE_IDENTITY_SCHEMA_V1,
        module_semantic_abi: phalcom_compiler_artifact::header::MODULE_SEMANTIC_ABI_V2,
        semantic_artifact_abi: phalcom_compiler_artifact::header::SEMANTIC_ARTIFACT_ABI_V3,
        query_fingerprint_abi: phalcom_compiler_artifact::header::QUERY_FINGERPRINT_ABI_V3,
        native_surface_schema_version: 1,
        native_catalog_fingerprint: [0; 16],
        type_metadata_schema_version: 1,
        capabilities,
        source_digest: ArtifactDigest::ZERO,
        interface_digest: ArtifactDigest::ZERO,
        semantic_digest: ArtifactDigest::ZERO,
        query_digest: ArtifactDigest::ZERO,
        metadata_digest: ArtifactDigest::ZERO,
        artifact_digest: ArtifactDigest::ZERO,
        section_count: 0,
    }
}

#[test]
fn executable_image_encoding_is_deterministic_and_round_trips() {
    let original = image();
    let first = original.encode().unwrap();
    let second = original.encode().unwrap();
    assert_eq!(first, second);

    let mut budget = AllocationBudget::new(4 * 1024 * 1024);
    let decoded = ExecutableImageV1::decode(&first, &ValidationLimits::default(), &ExecutableImageLimitsV1::default(), &mut budget).unwrap();
    assert_eq!(decoded, original);
    assert!(budget.requested() > 0);
}

#[test]
fn executable_decode_budget_is_transactional_at_n_and_n_minus_one() {
    let bytes = image().encode().unwrap();
    let mut exact = AllocationBudget::new(4 * 1024 * 1024);
    ExecutableImageV1::decode(&bytes, &ValidationLimits::default(), &ExecutableImageLimitsV1::default(), &mut exact).unwrap();
    let requested = exact.requested();

    let mut one_short = AllocationBudget::new(requested - 1);
    let error = ExecutableImageV1::decode(&bytes, &ValidationLimits::default(), &ExecutableImageLimitsV1::default(), &mut one_short).unwrap_err();
    assert!(matches!(error, ArtifactError::Allocation(_)));
    assert_eq!(one_short.requested(), 0, "failed decode must not publish a partial charge");

    let mut exact_limit = AllocationBudget::new(requested);
    ExecutableImageV1::decode(&bytes, &ValidationLimits::default(), &ExecutableImageLimitsV1::default(), &mut exact_limit).unwrap();
    assert_eq!(exact_limit.requested(), requested);
}

#[test]
fn executable_decode_rejects_tight_dimensions_oversized_strings_and_truncation() {
    let bytes = image().encode().unwrap();
    let mut caps = ExecutableImageLimitsV1::default();
    caps.max_modules = 0;
    let mut budget = AllocationBudget::new(4 * 1024 * 1024);
    assert!(ExecutableImageV1::decode(&bytes, &ValidationLimits::default(), &caps, &mut budget).is_err());
    assert_eq!(budget.requested(), 0);

    let mut limits = ValidationLimits::default();
    limits.max_string_bytes = 2;
    let mut budget = AllocationBudget::new(4 * 1024 * 1024);
    assert!(ExecutableImageV1::decode(&bytes, &limits, &ExecutableImageLimitsV1::default(), &mut budget).is_err());
    assert_eq!(budget.requested(), 0);

    let mut budget = AllocationBudget::new(4 * 1024 * 1024);
    assert!(
        ExecutableImageV1::decode(
            &bytes[..bytes.len() - 1],
            &ValidationLimits::default(),
            &ExecutableImageLimitsV1::default(),
            &mut budget
        )
        .is_err()
    );
    assert_eq!(budget.requested(), 0);
}

#[test]
fn executable_constant_recipe_tags_round_trip_with_exact_scalar_bits_and_graph_edges() {
    let mut original = image();
    let module = module_ref();
    let class = StableClassRefV1 {
        declaration: StableDeclarationRef {
            module: module.clone(),
            path: vec!["Fixture".into()].into_boxed_slice(),
        },
        side: StableDispatchSide::Instance,
    };
    original.classes.push(PortableClassRecipeV1 {
        class: class.clone(),
        allocation: PortableClassAllocationV1::SourceDeclared,
        superclass: None,
        metaclass: None,
        fields: Vec::new(),
        field_count: 0,
        immutable_slots: Vec::new(),
        native_repr: false,
        sealed: false,
        is_abstract: false,
    });
    original.constants = vec![
        ConstantRecipeV1::None,
        ConstantRecipeV1::Unit,
        ConstantRecipeV1::EmptyRecord,
        ConstantRecipeV1::EmptyPack,
        ConstantRecipeV1::Boolean { value: true },
        ConstantRecipeV1::Integer { bits: (-42_i64) as u64 },
        ConstantRecipeV1::Float { bits: 0x7ff8_0000_0000_0042 },
        ConstantRecipeV1::Character { scalar: 0x1f680 },
        ConstantRecipeV1::Symbol { text: "symbol".into() },
        ConstantRecipeV1::String { text: "string".into() },
        ConstantRecipeV1::BigInteger {
            negative: true,
            magnitude: vec![0, 0, 0x80],
        },
        ConstantRecipeV1::Class { reference: class.clone() },
        ConstantRecipeV1::Module { module: module.clone() },
        ConstantRecipeV1::ClosureTemplate {
            prototype: PrototypeId(0),
            module: module.clone(),
            lexical_class: Some(class.clone()),
            super_anchor: Some(class.clone()),
        },
        ConstantRecipeV1::Method {
            recipe: MethodRecipeV1 {
                body: MethodBodyV1::Closure { template: ConstantId(13) },
                signature: MethodSignatureV1 {
                    selector: "run".into(),
                    kind: MethodSignatureKindV1::Method { slots: 1 },
                    positional_arity: 0,
                    rest: None,
                },
                holder: Some(class.clone()),
                access_owner: Some(class.clone()),
                visibility: VisibilityV1::Public,
                origin: MethodOriginV1 {
                    module: Some(module.clone()),
                    package: None,
                    project: None,
                },
                contracts: Some(vec![ContractRecipeV1 {
                    predicate: "pre".into(),
                    value: ConstantId(0),
                }]),
                attributes: vec![ConstantId(1)],
                attributes_frozen: true,
            },
        },
        ConstantRecipeV1::SelectorPattern {
            pattern: SelectorPatternV1 {
                base: SelectorBaseV1::Named { name: "run".into() },
                kind: SelectorKindPatternV1::Exact { kind: SelectorKindV1::Method },
                prefix: vec![SelectorSlotV1::Label { label: "input".into() }],
                suffix: vec![SelectorSlotV1::Positional],
                has_gap: true,
            },
        },
        ConstantRecipeV1::Vector { elements: vec![ConstantId(4)] },
        ConstantRecipeV1::FrozenArray { elements: vec![ConstantId(5)] },
        ConstantRecipeV1::Tuple {
            positionals: vec![ConstantId(6)],
            labels: vec!["label".into()],
            labeled_values: vec![ConstantId(7)],
        },
        ConstantRecipeV1::Record {
            labels: vec!["field".into()],
            values: vec![ConstantId(8)],
        },
        ConstantRecipeV1::Pack {
            positionals: vec![ConstantId(9)],
            labels: vec!["rest".into()],
            labeled_values: vec![ConstantId(10)],
        },
        ConstantRecipeV1::Nil,
    ];
    original.prototypes[0].constants = (0..u32::try_from(original.constants.len()).unwrap()).map(ConstantId).collect();

    let bytes = original.encode().unwrap();
    let mut budget = AllocationBudget::new(4 * 1024 * 1024);
    let decoded = ExecutableImageV1::decode(&bytes, &ValidationLimits::default(), &ExecutableImageLimitsV1::default(), &mut budget).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn prototype_round_trip_preserves_owner_parameters_upvalues_and_spans() {
    let mut prototype = image().prototypes.remove(0);
    prototype.owner = PrototypeOwnerV1::Nested {
        parent: PrototypeId(3),
        ordinal: 9,
    };
    prototype.max_slots = 3;
    prototype.arity = 2;
    prototype.upvalues = vec![UpvalueCaptureV1::Local(1), UpvalueCaptureV1::Upvalue(0)];
    prototype.parameters = ParameterShapeV1 {
        fixed_positionals: 2,
        fixed_labels: vec!["named".into()],
        rest: Some(RestKindV1::Split),
    };
    prototype.name = "nested callable".into();
    prototype.local_names = vec!["self".into(), "first".into(), "second".into()];
    prototype.spans = vec![SpanV1 { start: 4, end: 8 }];

    let bytes = prototype.encode().unwrap();
    let mut budget = AllocationBudget::new(1024 * 1024);
    let decoded = PortablePrototypeV1::decode(&bytes, &ValidationLimits::default(), &ExecutablePrototypeLimitsV1::default(), &mut budget).unwrap();
    assert_eq!(decoded, prototype);
}

#[test]
fn prototype_decoder_applies_string_limit_to_owner_identity() {
    let mut prototype = image().prototypes.remove(0);
    let PrototypeOwnerV1::ModuleInitializer { module } = &mut prototype.owner else {
        unreachable!();
    };
    *module = StableModuleRef {
        project: StableProjectRef::Builtin {
            namespace: "u".into(),
            version: "1".into(),
        },
        path: vec!["owner-string-that-exceeds-the-limit".into()].into_boxed_slice(),
    };
    prototype.name.clear();
    prototype.source = SourceMapId(0);
    let bytes = prototype.encode().unwrap();
    let mut common = ValidationLimits::default();
    common.max_string_bytes = 8;
    let mut budget = AllocationBudget::new(1024 * 1024);
    assert!(PortablePrototypeV1::decode(&bytes, &common, &ExecutablePrototypeLimitsV1::default(), &mut budget).is_err());
    assert_eq!(budget.requested(), 0);
}

#[test]
fn executable_image_rejects_dangling_bootstrap_and_prototype_references() {
    let mut bad_initializer = image();
    bad_initializer.modules[0].initializer = PrototypeId(1);
    assert!(bad_initializer.validate_structure(&ExecutableImageLimitsV1::default()).is_err());

    let mut bad_bootstrap = image();
    bad_bootstrap.bootstrap.eager_modules[0].initializer = PrototypeId(1);
    assert!(bad_bootstrap.validate_structure(&ExecutableImageLimitsV1::default()).is_err());
}

#[test]
fn staged_executable_section_is_optional_until_validation_authorizes_it() {
    let image = image();
    let staged = StagedExecutableSectionV1::from_image(&image).unwrap();
    let expected_payload = image.encode().unwrap();
    assert_eq!(staged.schema_version(), phalcom_compiler_artifact::executable::EXECUTABLE_SCHEMA_V1);
    assert_eq!(staged.payload(), expected_payload);

    let mut builder = TeloscContainerBuilder::new(container_header(TeloscCapabilities::empty()));
    builder.add_section(
        SectionKind::ModuleWorld,
        phalcom_compiler_artifact::module_world::MODULE_WORLD_SCHEMA_V2,
        SectionFlags::REQUIRED,
        Vec::new(),
    );
    builder.add_section(
        SectionKind::SemanticWorld,
        phalcom_compiler_artifact::semantic::world::SEMANTIC_WORLD_SCHEMA_V4,
        SectionFlags::REQUIRED,
        Vec::new(),
    );
    builder.stage_executable_section(staged).unwrap();
    let bytes = builder.build().unwrap();
    let view = TeloscContainerView::parse(&bytes, &ValidationLimits::default()).unwrap();
    assert!(!view.header().capabilities.contains(TeloscCapabilities::PORTABLE_EXECUTABLE));
    assert_eq!(view.get_section(SectionKind::Executable), Some(expected_payload.as_slice()));

    let mut authorized_header = container_header(TeloscCapabilities::PORTABLE_EXECUTABLE);
    authorized_header.section_count = 0;
    let mut builder = TeloscContainerBuilder::new(authorized_header);
    let error = builder
        .stage_executable_section(StagedExecutableSectionV1::from_image(&image).unwrap())
        .unwrap_err();
    assert!(matches!(
        error,
        ArtifactError::Container(phalcom_compiler_artifact::ContainerError::UnvalidatedExecutableCapability)
    ));
}

#[test]
fn portable_instruction_decoder_rejects_unknown_and_truncated_records() {
    let unknown_bytes = u16::MAX.to_le_bytes();
    let mut unknown = BinaryReader::new(&unknown_bytes);
    assert!(matches!(PortableInstructionV1::read_from(&mut unknown), Err(CodecError::InvalidTag { .. })));

    let mut truncated = BinaryReader::new(&[1, 0, 0]);
    assert!(matches!(PortableInstructionV1::read_from(&mut truncated), Err(CodecError::UnexpectedEof)));
}
