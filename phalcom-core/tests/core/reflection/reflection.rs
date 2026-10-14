use phalcom_core::heap::{ClassObject, ListObject, Object};
use phalcom_core::product::finish_tuple_value;
use phalcom_core::typing::context::TypingContextData;
use phalcom_core::typing::handle::{MetadataPoolId, RuntimeTypeRef};
use phalcom_core::typing::inspect;
use phalcom_core::typing::loader::LoadedSemanticMetadata;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_type_meta::bundle::SemanticMetadataBundle;
use phalcom_type_meta::declaration::{
    CallableSemanticRecord, DeclarationTypeFlags, DeclarationTypeRecord, FieldMutabilityRef, FieldSemanticRecord, PublishedTypeAuthority, PublishedTypeSlot,
};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::generic::{
    GenericConstraintRef, GenericSignatureRecord, GenericSignatureRecordId, StableTypeParameterOwnerRef, StableTypeParameterRef, TypeParameterRecord,
    VarianceRef,
};
use phalcom_type_meta::header::{
    ArtifactIdentityScheme, MetadataFeatures, MetadataProfile, NATIVE_SURFACE_SCHEMA_VERSION, ProducerIdentity, SEMANTIC_MODEL_VERSION, SemanticMetadataHeader,
    TYPE_METADATA_SCHEMA_VERSION,
};
use phalcom_type_meta::identity::{StableCallableRef, StableDeclarationRef, StableDispatchSide, StableFieldRef, StableModuleRef, StableProjectRef};
use phalcom_type_meta::kind::{KindNode, KindNodeEntry, KindNodeId};
use phalcom_type_meta::type_node::{RecordFieldRef, RecordRowRef, StaticArgumentRef, StructuralMemberPresenceRef, TypeNode, TypeNodeEntry, TypeNodeId};
use std::sync::Arc;

fn send0(vm: &mut VM, receiver: Value, selector: &str) -> Value {
    let selector = vm.get_or_intern(selector);
    vm.send_dynamic(receiver, selector, &[]).unwrap_or_else(|error| panic!("send failed: {error}"))
}

fn send1(vm: &mut VM, receiver: Value, selector: &str, arg: Value) -> Value {
    let selector = vm.get_or_intern(selector);
    vm.send_dynamic(receiver, selector, &[arg])
        .unwrap_or_else(|error| panic!("send failed: {error}"))
}

fn send2(vm: &mut VM, receiver: Value, selector: &str, first: Value, second: Value) -> Value {
    let selector = vm.get_or_intern(selector);
    vm.send_dynamic(receiver, selector, &[first, second])
        .unwrap_or_else(|error| panic!("send failed: {error}"))
}

fn send4(vm: &mut VM, receiver: Value, selector: &str, a1: Value, a2: Value, a3: Value, a4: Value) -> Value {
    let selector = vm.get_or_intern(selector);
    vm.send_dynamic(receiver, selector, &[a1, a2, a3, a4])
        .unwrap_or_else(|error| panic!("send failed: {error}"))
}

fn tuple(vm: &mut VM, values: Vec<Value>) -> Value {
    finish_tuple_value(vm, values, Vec::new()).expect("tuple construction")
}

fn tuple_values(vm: &VM, value: Value) -> Vec<Value> {
    vm.tuple_view(value.as_obj().expect("tuple value")).expect("tuple view").values()
}

fn test_module() -> StableModuleRef {
    StableModuleRef {
        project: StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "1.0".into(),
        },
        path: Box::new([]),
    }
}

fn build_test_bundle() -> SemanticMetadataBundle {
    let header = SemanticMetadataHeader {
        schema_version: TYPE_METADATA_SCHEMA_VERSION,
        semantic_model_version: SEMANTIC_MODEL_VERSION,
        producer: ProducerIdentity("test".into()),
        producer_version: "1.0".into(),
        native_surface_schema_version: NATIVE_SURFACE_SCHEMA_VERSION,
        profile: MetadataProfile::ToolingDebug,
        features: MetadataFeatures::default(),
        identity_scheme: ArtifactIdentityScheme::V1Standard,
        source_fingerprint: Fingerprint128::ZERO,
        interface_fingerprint: Fingerprint128::ZERO,
    };

    let kinds = Box::new([
        KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::RecordRow,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::Arrow {
                parameters: vec![KindNodeId(1)].into_boxed_slice(),
                result: KindNodeId(0),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);

    let decl_box = StableDeclarationRef {
        module: test_module(),
        path: vec!["Box".into()].into_boxed_slice(),
    };
    let decl_row_box = StableDeclarationRef {
        module: test_module(),
        path: vec!["RowBox".into()].into_boxed_slice(),
    };

    let types = Box::new([
        // 0: Unit
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 1: String
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Nominal {
                declaration: StableDeclarationRef {
                    module: test_module(),
                    path: vec!["String".into()].into_boxed_slice(),
                },
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 2: Parameter T
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Parameter(StableTypeParameterRef {
                owner: StableTypeParameterOwnerRef::Declaration(decl_box.clone()),
                index: 0,
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 3: Box<T>
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Nominal { declaration: decl_box.clone() },
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 4: RowBox
        TypeNodeEntry {
            kind: KindNodeId(2),
            form: TypeNode::Nominal {
                declaration: decl_row_box.clone(),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 5: Int
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Nominal {
                declaration: StableDeclarationRef {
                    module: test_module(),
                    path: vec!["Int".into()].into_boxed_slice(),
                },
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 6: RowBox<{payload: Int}>
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Applied {
                origin: TypeNodeId(4),
                arguments: vec![StaticArgumentRef::RecordRow(RecordRowRef {
                    fields: vec![RecordFieldRef {
                        key: phalcom_type_meta::field_key::FieldKeyRef::Symbol("payload".into()),
                        ty: TypeNodeId(5),
                        presence: StructuralMemberPresenceRef::Required,
                    }]
                    .into_boxed_slice(),
                    tail: None,
                    remainder: None,
                })]
                .into_boxed_slice(),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 7: RowBox<{payload: String, **R0}>
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Applied {
                origin: TypeNodeId(4),
                arguments: vec![StaticArgumentRef::RecordRow(RecordRowRef {
                    fields: vec![RecordFieldRef {
                        key: phalcom_type_meta::field_key::FieldKeyRef::Symbol("payload".into()),
                        ty: TypeNodeId(1),
                        presence: StructuralMemberPresenceRef::Required,
                    }]
                    .into_boxed_slice(),
                    tail: Some(StableTypeParameterRef {
                        owner: StableTypeParameterOwnerRef::Declaration(decl_row_box.clone()),
                        index: 0,
                    }),
                    remainder: None,
                })]
                .into_boxed_slice(),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);

    let param_t_ref = StableTypeParameterRef {
        owner: StableTypeParameterOwnerRef::Declaration(decl_box.clone()),
        index: 0,
    };
    let param_fs_ref = StableTypeParameterRef {
        owner: StableTypeParameterOwnerRef::Declaration(decl_row_box.clone()),
        index: 0,
    };

    let param_t = TypeParameterRecord {
        id: param_t_ref.clone(),
        name: "T".into(),
        kind: KindNodeId(0),
        variance: VarianceRef::Covariant,
        mode: Default::default(),
        source: None,
    };
    let param_fs = TypeParameterRecord {
        id: param_fs_ref.clone(),
        name: "Fs".into(),
        kind: KindNodeId(1),
        variance: VarianceRef::Invariant,
        mode: Default::default(),
        source: None,
    };

    let gen_sig = GenericSignatureRecord {
        owner: StableTypeParameterOwnerRef::Declaration(decl_box.clone()),
        parameters: vec![param_t_ref].into_boxed_slice(),
        constraints: vec![GenericConstraintRef::Subtype {
            lower: TypeNodeId(2),
            upper: TypeNodeId(0),
        }]
        .into_boxed_slice(),
    };
    let row_gen_sig = GenericSignatureRecord {
        owner: StableTypeParameterOwnerRef::Declaration(decl_row_box.clone()),
        parameters: vec![param_fs_ref].into_boxed_slice(),
        constraints: Box::new([]),
    };

    let callable_value = CallableSemanticRecord {
        callable: StableCallableRef {
            owner: phalcom_type_meta::identity::StableCallableOwnerRef::Declaration(decl_box.clone()),
            side: StableDispatchSide::Instance,
            selector: "value".into(),
        },
        generic_signature: None,
        parameters: Box::new([]),
        return_type: PublishedTypeSlot::Known {
            form: TypeNodeId(2), // returns T
            authority: PublishedTypeAuthority::DeclaredAnnotation,
        },
        source: None,
    };

    let decl_rec = DeclarationTypeRecord {
        declaration: decl_box.clone(),
        form: TypeNodeId(3),
        kind: KindNodeId(0),
        generic_signature: Some(GenericSignatureRecordId(0)),
        superclass_template: None,
        instance_callables: vec![callable_value.callable.clone()].into_boxed_slice(),
        class_callables: Box::new([]),
        instance_fields: Box::new([]),
        class_fields: Box::new([]),
        flags: DeclarationTypeFlags::default(),
        algebraic: None,
        source: None,
    };
    let row_decl_rec = DeclarationTypeRecord {
        declaration: decl_row_box,
        form: TypeNodeId(4),
        kind: KindNodeId(2),
        generic_signature: Some(GenericSignatureRecordId(1)),
        superclass_template: None,
        instance_callables: Box::new([]),
        class_callables: Box::new([]),
        instance_fields: Box::new([]),
        class_fields: Box::new([]),
        flags: DeclarationTypeFlags::default(),
        algebraic: None,
        source: None,
    };

    let field_val = FieldSemanticRecord {
        field: StableFieldRef {
            owner: decl_box.clone(),
            side: StableDispatchSide::Instance,
            name: "val".into(),
        },
        mutability: FieldMutabilityRef::Immutable,
        ty: PublishedTypeSlot::Known {
            form: TypeNodeId(2), // T
            authority: PublishedTypeAuthority::DeclaredAnnotation,
        },
        source: None,
    };

    SemanticMetadataBundle {
        header,
        kinds,
        types,
        scoped_types: Box::new([]),
        parameters: vec![param_t, param_fs].into_boxed_slice(),
        generic_signatures: vec![gen_sig, row_gen_sig].into_boxed_slice(),
        declarations: vec![decl_rec, row_decl_rec].into_boxed_slice(),
        static_domains: Box::new([]),
        aliases: Box::new([]),
        callables: vec![callable_value].into_boxed_slice(),
        fields: vec![field_val].into_boxed_slice(),
        module_roots: Box::new([]),
        runtime_roots: Box::new([]),
        occurrences: Box::new([]),
        static_values: Box::new([]),
        static_constants: Box::new([]),
        static_definitions: Box::new([]),
        extensions: Box::new([]),
    }
}

#[test]
fn metadata_loader_preserves_parameter_mode_and_kind_independently() {
    use phalcom_core::typing::loader::load_metadata_bundle;
    use phalcom_modules::{DeclarationId, ModuleComponent, ModuleId, ModulePath};
    use phalcom_semantic::metadata::export::MetadataExporter;
    use phalcom_semantic::types::parameter::GenericParameterMode;
    use phalcom_semantic::types::{GenericSignature, KindId, TypeParameterData, TypeParameterOwner, TypeStore};
    use phalcom_type_meta::generic::GenericParameterModeRef;
    use phalcom_type_meta::validate::ValidationLimits;

    let owner_ref = DeclarationId::new(
        ModuleId::universe(ModulePath::from_components(vec![ModuleComponent::from_identifier("mode_kind").unwrap()])),
        "ModeKind".into(),
    );
    let cases = [
        (
            KindId::LABELED_TYPE_PACK,
            GenericParameterMode::LabeledPack,
            phalcom_type_meta::kind::KindNode::LabeledTypePack,
            GenericParameterModeRef::LabeledPack,
        ),
        (
            KindId::RECORD_ROW,
            GenericParameterMode::LabeledPack,
            phalcom_type_meta::kind::KindNode::RecordRow,
            GenericParameterModeRef::LabeledPack,
        ),
        (
            KindId::RECORD_ROW,
            GenericParameterMode::Single,
            phalcom_type_meta::kind::KindNode::RecordRow,
            GenericParameterModeRef::Single,
        ),
    ];

    for (index, (kind, mode, expected_kind, expected_mode)) in cases.into_iter().enumerate() {
        let mut store = TypeStore::new();
        let owner = TypeParameterOwner::Declaration(owner_ref.clone());
        let parameter = store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "P", kind).with_mode(mode));
        let signature = GenericSignature::new(owner, Box::new([parameter])).with_parameter_modes(Box::new([mode]));
        let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
        exporter.export_generic_signature(&signature).unwrap();
        let bundle = exporter.build_bundle(&[]).unwrap();

        let loaded =
            load_metadata_bundle(MetadataPoolId(index as u32), Arc::new(bundle), &ValidationLimits::default()).expect("valid mode/kind pair should load");
        let parameter = &loaded.bundle.parameters[0];
        assert_eq!(loaded.bundle.kinds[parameter.kind.0 as usize].node, expected_kind);
        assert_eq!(parameter.mode, expected_mode);
    }
}

#[test]
fn exact_static_integer_metadata_reflection_preserves_signed_values_and_identity() {
    use phalcom_core::heap::Heap;
    use phalcom_core::typing::loader::load_metadata_bundle;
    use phalcom_core::typing::registry::RuntimeTypingRegistry;
    use phalcom_modules::{DeclarationId, ModuleId};
    use phalcom_semantic::KindId;
    use phalcom_semantic::metadata::MetadataExporter;
    use phalcom_semantic::types::store::{StaticArgument, TypeStore};
    use phalcom_type_meta::encode::{decode_metadata_json, encode_metadata_json};
    use phalcom_type_meta::validate::ValidationLimits;

    let decimal = "12345678901234567890123456789012345678901234567890";
    let mut store = TypeStore::new();
    let positive = store.const_int(phalcom_type_meta::StaticInt::new(decimal.parse().unwrap()));
    let negative = store.const_int(phalcom_type_meta::StaticInt::new(format!("-{decimal}").parse().unwrap()));
    let integer_kind = store.kind_of(positive);
    let constructor_kind = store.arrow_kind(Box::new([integer_kind]), KindId::TYPE);
    let constructor = store.nominal_form(DeclarationId::new(ModuleId::universe_root(), "SizedEvidence".into()), constructor_kind);
    let positive_application = store.apply_static_form(constructor, &[StaticArgument::Term(positive)]).unwrap();
    let negative_application = store.apply_static_form(constructor, &[StaticArgument::Term(negative)]).unwrap();
    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let positive_node = exporter.export_type_form(positive_application).unwrap();
    let negative_node = exporter.export_type_form(negative_application).unwrap();
    let bundle = exporter.build_bundle(&[]).unwrap();
    let json = encode_metadata_json(&bundle).unwrap();
    let limits = ValidationLimits::default();
    let mut registry = RuntimeTypingRegistry::new();
    let pool = registry.register_pool(load_metadata_bundle(MetadataPoolId(0), Arc::new(decode_metadata_json(&json, &limits).unwrap()), &limits).unwrap());
    let other_pool = registry.register_pool(load_metadata_bundle(MetadataPoolId(1), Arc::new(bundle), &limits).unwrap());
    let context = TypingContextData::new(Box::new([pool, other_pool]));
    let heap = Heap::new();
    let positive = RuntimeTypeRef::Base { pool, node: positive_node };
    let negative = RuntimeTypeRef::Base { pool, node: negative_node };
    assert_eq!(inspect::display(&context, &registry, &heap, positive), format!("SizedEvidence<{decimal}>"));
    assert_eq!(inspect::display(&context, &registry, &heap, negative), format!("SizedEvidence<-{decimal}>"));
    assert!(!inspect::equivalent(&context, &registry, positive, negative));
    assert!(inspect::equivalent(
        &context,
        &registry,
        positive,
        RuntimeTypeRef::Base {
            pool: other_pool,
            node: positive_node
        }
    ));
}

#[test]
fn spec03_bootstraps_typing_surface_and_context_capabilities() {
    let mut vm = VM::new();
    let typing_class = vm.universe.typing_classes.get("Typing").expect("Typing class");
    let context_class = vm.universe.typing_classes.get("TypingContext").expect("TypingContext class");

    let context = send0(&mut vm, Value::obj(typing_class), "current");
    assert_eq!(context.class(&vm), context_class);
    let profile = send0(&mut vm, context, "profile").as_symbol().ok().map(|symbol| vm.resolve_symbol(symbol));
    assert_eq!(profile, Some("RuntimePublic"));

    let capability_value = send0(&mut vm, context, "capabilities");
    let capabilities = tuple_values(&vm, capability_value);
    let capability_names = capabilities
        .iter()
        .filter_map(|value| value.as_symbol().ok())
        .map(|symbol| vm.resolve_symbol(symbol).to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        capability_names,
        ["OBSERVE_PUBLIC_TYPES", "OBSERVE_SIGNATURES", "CONSTRUCT_TYPE_FORMS", "EVALUATE_RELATIONS"]
    );

    let observe_public_types = Value::symbol(vm.get_or_intern("OBSERVE_PUBLIC_TYPES"));
    let allowed = tuple(&mut vm, vec![observe_public_types]);
    let restricted = send1(&mut vm, context, "restrictTo(_)", allowed);
    let restricted_capabilities = send0(&mut vm, restricted, "capabilities");
    assert_eq!(tuple_values(&vm, restricted_capabilities).len(), 1);
    let apply_selector = vm.get_or_intern("apply(_,_)");
    let empty_arguments = tuple(&mut vm, Vec::new());
    let list_class = vm.universe.classes.list_class;
    assert!(vm.send_dynamic(restricted, apply_selector, &[Value::obj(list_class), empty_arguments]).is_err());
}

#[test]
fn spec03_reifies_applied_type_forms_and_returns_bounded_results() {
    let mut vm = VM::new();
    let typing_class = vm.universe.typing_classes.get("Typing").expect("Typing class");
    let list_class = vm.universe.classes.list_class;
    let int_class = vm.universe.classes.int_class;
    let type_class = vm.universe.typing_classes.get("Type").expect("Type class");
    let descriptor_class = vm.universe.typing_classes.get("AppliedType").expect("AppliedType class");
    let known_class = vm.universe.typing_classes.get("TypingKnown").expect("TypingKnown class");
    let satisfied_class = vm.universe.typing_classes.get("RelationSatisfied").expect("RelationSatisfied class");

    let context = send0(&mut vm, Value::obj(typing_class), "current");
    let list_kind = send0(&mut vm, Value::obj(list_class), "kind");
    let list_kind_display = send0(&mut vm, list_kind, "display");
    let list_kind_display_text = vm.heap.string(list_kind_display.as_obj().expect("kind display string")).as_str().to_owned();
    assert_eq!(list_kind_display_text, "Type -> Type");
    assert_eq!(send0(&mut vm, list_kind, "argumentCount").as_int(), Some(1));
    assert!(send0(&mut vm, list_kind, "result").is_some());
    assert_eq!(send0(&mut vm, Value::obj(list_class), "remainingParameterCount").as_int(), Some(1));

    for (collection_class, expected_display) in [
        (vm.universe.classes.slice_class, "Slice<Int>"),
        (vm.universe.classes.buffer_class, "Buffer<Int>"),
    ] {
        let collection_kind = send0(&mut vm, Value::obj(collection_class), "kind");
        assert_eq!(send0(&mut vm, collection_kind, "argumentCount").as_int(), Some(1));
        assert_eq!(send0(&mut vm, Value::obj(collection_class), "remainingParameterCount").as_int(), Some(1));

        let collection_arguments = tuple(&mut vm, vec![Value::obj(int_class)]);
        let collection_known = send2(&mut vm, context, "apply(_,_)", Value::obj(collection_class), collection_arguments);
        assert_eq!(collection_known.class(&vm), known_class);

        let collection_descriptor = send0(&mut vm, collection_known, "value");
        assert_eq!(collection_descriptor.class(&vm), descriptor_class);
        assert_eq!(send0(&mut vm, collection_descriptor, "argumentCount").as_int(), Some(1));
        let collection_display = send0(&mut vm, collection_descriptor, "display");
        assert_eq!(
            vm.heap.string(collection_display.as_obj().expect("applied collection type display")).as_str(),
            expected_display
        );
    }

    let arguments = tuple(&mut vm, vec![Value::obj(int_class)]);
    let known = send2(&mut vm, context, "apply(_,_)", Value::obj(list_class), arguments);
    assert_eq!(known.class(&vm), known_class);

    let descriptor = send0(&mut vm, known, "value");
    assert_eq!(descriptor.class(&vm), descriptor_class);
    assert_eq!(send0(&mut vm, descriptor, "kind"), Value::obj(type_class));
    let display = send0(&mut vm, descriptor, "display");
    let display_text = vm.heap.string(display.as_obj().expect("display string")).as_str().to_owned();
    assert_eq!(display_text, "List<Int>");
    assert_eq!(send0(&mut vm, descriptor, "argumentCount").as_int(), Some(1));
    assert_eq!(send0(&mut vm, descriptor, "remainingParameterCount").as_int(), Some(0));

    let argument = send1(&mut vm, descriptor, "argumentAt(_)", Value::int(0));
    let argument_display = send0(&mut vm, argument, "display");
    let argument_display_text = vm.heap.string(argument_display.as_obj().expect("argument display string")).as_str().to_owned();
    assert_eq!(argument_display_text, "Int");

    let relation = send1(&mut vm, descriptor, "subtypeOf(_)", descriptor);
    assert_eq!(relation.class(&vm), satisfied_class);
    assert_eq!(send0(&mut vm, relation, "value").as_bool(), Some(true));

    let tuple_arguments = tuple(&mut vm, vec![Value::obj(int_class), Value::obj(list_class)]);
    let tuple_known = send1(&mut vm, context, "tupleOf(_)", tuple_arguments);
    let tuple_descriptor = send0(&mut vm, tuple_known, "value");
    assert_eq!(
        tuple_descriptor.class(&vm),
        vm.universe.typing_classes.get("TupleType").expect("TupleType class")
    );
    let tuple_display = send0(&mut vm, tuple_descriptor, "display");
    assert_eq!(vm.heap.string(tuple_display.as_obj().expect("tuple display string")).as_str(), "(Int, List)");

    let callable_parameters = tuple(&mut vm, vec![Value::obj(int_class)]);
    let callable_known = send2(&mut vm, context, "callable(_,_)", callable_parameters, Value::obj(int_class));
    let callable_descriptor = send0(&mut vm, callable_known, "value");
    assert_eq!(
        callable_descriptor.class(&vm),
        vm.universe.typing_classes.get("CallableType").expect("CallableType class")
    );
    let callable_display = send0(&mut vm, callable_descriptor, "display");
    assert_eq!(
        vm.heap.string(callable_display.as_obj().expect("callable display string")).as_str(),
        "(Int) -> Int"
    );
}

#[test]
fn applied_record_row_metadata_display_preserves_field_types_and_tail() {
    let mut vm = VM::new();
    let pool = vm
        .typing_registry
        .register_pool(LoadedSemanticMetadata::new(MetadataPoolId(0), Arc::new(build_test_bundle())));
    let context = TypingContextData::new(vec![pool].into_boxed_slice());

    let int_row = inspect::display(&context, &vm.typing_registry, &vm.heap, RuntimeTypeRef::Base { pool, node: TypeNodeId(6) });
    let string_row = inspect::display(&context, &vm.typing_registry, &vm.heap, RuntimeTypeRef::Base { pool, node: TypeNodeId(7) });

    assert_eq!(int_row, "RowBox<{payload: Int}>");
    assert_eq!(string_row, "RowBox<{payload: String, **R0}>");
    assert_ne!(int_row, string_row);
}

#[test]
fn spec03_reflects_generic_signatures_parameters_and_specialized_member_lookup() {
    let mut vm = VM::new();
    let bundle = build_test_bundle();
    let _pool_id = vm
        .typing_registry
        .register_pool(LoadedSemanticMetadata::new(MetadataPoolId(0), Arc::new(bundle)));

    // Create a class named "Box" in VM heap and bind nominal declaration to it
    let box_class_id = vm.heap.alloc_class(ClassObject::bare("Box"));
    let string_class_id = vm.heap.alloc_class(ClassObject::bare("String"));

    let decl_box = StableDeclarationRef {
        module: test_module(),
        path: vec!["Box".into()].into_boxed_slice(),
    };
    vm.typing_registry.register_nominal_binding(decl_box.clone(), box_class_id);
    vm.typing_registry.register_nominal_binding(
        StableDeclarationRef {
            module: test_module(),
            path: vec!["String".into()].into_boxed_slice(),
        },
        string_class_id,
    );

    let typing_class = vm.universe.typing_classes.get("Typing").expect("Typing class");
    let context = send0(&mut vm, Value::obj(typing_class), "current");

    // 1. GenericSignature inspection
    let known_sig = send1(&mut vm, context, "genericSignatureOf(_)", Value::obj(box_class_id));
    assert_eq!(known_sig.class(&vm), vm.universe.typing_classes.get("TypingKnown").unwrap());
    let sig = send0(&mut vm, known_sig, "value");
    assert_eq!(sig.class(&vm), vm.universe.typing_classes.get("GenericSignature").unwrap());
    assert_eq!(send0(&mut vm, sig, "parameterCount").as_int(), Some(1));

    let param = send1(&mut vm, sig, "parameterAt(_)", Value::int(0));
    assert_eq!(param.class(&vm), vm.universe.typing_classes.get("TypeParameter").unwrap());
    let param_name = send0(&mut vm, param, "name");
    assert_eq!(vm.resolve_symbol(param_name.as_symbol().unwrap()), "T");
    assert_eq!(send0(&mut vm, param, "index").as_int(), Some(0));
    let variance = send0(&mut vm, param, "variance");
    assert!(variance.is_some());

    // 2. Member lookup with specialization (Box<String>.value specialized to String)
    let args_tuple = tuple(&mut vm, vec![Value::obj(string_class_id)]);
    let applied_box_known = send2(&mut vm, context, "apply(_,_)", Value::obj(box_class_id), args_tuple);
    let applied_box = send0(&mut vm, applied_box_known, "value");

    let val_sel = Value::symbol(vm.get_or_intern("value"));
    let inst_side = Value::symbol(vm.get_or_intern("instance"));
    let norm_lookup = Value::symbol(vm.get_or_intern("normal"));

    let member_result = send4(&mut vm, context, "member(_,_,_,_)", applied_box, val_sel, inst_side, norm_lookup);
    assert_eq!(member_result.class(&vm), vm.universe.typing_classes.get("MemberFound").unwrap());

    let method_sig = send0(&mut vm, member_result, "value");
    assert_eq!(method_sig.class(&vm), vm.universe.typing_classes.get("CallableSignature").unwrap());

    let ret_type = send0(&mut vm, method_sig, "returnType");
    let ret_display = send0(&mut vm, ret_type, "display");
    assert_eq!(vm.heap.string(ret_display.as_obj().unwrap()).as_str(), "String");

    // 3. Proof refusal and capability enforcement
    // In default context, INSPECT_PROOFS is denied
    let proofs_selector = vm.get_or_intern("proofsOf(_)");
    assert!(vm.send_dynamic(context, proofs_selector, &[Value::obj(box_class_id)]).is_err());

    // In Proof context, proofsOf returns TypingUnavailable("proofs_deferred_to_spec05")
    let proof_sym = Value::symbol(vm.get_or_intern("proof"));
    let proof_ctx_known = send1(&mut vm, Value::obj(typing_class), "contextFor(_)", proof_sym);
    let proof_ctx = send0(&mut vm, proof_ctx_known, "value");
    let proofs = send1(&mut vm, proof_ctx, "proofsOf(_)", Value::obj(box_class_id));
    assert_eq!(proofs.class(&vm), vm.universe.typing_classes.get("TypingUnavailable").unwrap());
    let proof_reason = send0(&mut vm, proofs, "value");
    assert_eq!(vm.resolve_symbol(proof_reason.as_symbol().unwrap()), "proofs_deferred_to_spec05");
}

#[test]
fn spec03_reflects_direct_conformance_constraint_without_fabricated_right_type() {
    let mut vm = VM::new();
    let mut bundle = build_test_bundle();
    bundle.generic_signatures[0].constraints = vec![GenericConstraintRef::Conforms {
        subject: TypeNodeId(2),
        trait_application: phalcom_type_meta::generic::StableTraitApplicationRef {
            declaration: StableDeclarationRef {
                module: test_module(),
                path: vec!["Printable".into()].into_boxed_slice(),
            },
            arguments: Box::new([]),
        },
    }]
    .into_boxed_slice();
    vm.typing_registry
        .register_pool(LoadedSemanticMetadata::new(MetadataPoolId(0), Arc::new(bundle)));

    let box_class_id = vm.heap.alloc_class(ClassObject::bare("Box"));
    vm.typing_registry.register_nominal_binding(
        StableDeclarationRef {
            module: test_module(),
            path: vec!["Box".into()].into_boxed_slice(),
        },
        box_class_id,
    );
    let typing_class = vm.universe.typing_classes.get("Typing").expect("Typing class");
    let context = send0(&mut vm, Value::obj(typing_class), "current");
    let known_sig = send1(&mut vm, context, "genericSignatureOf(_)", Value::obj(box_class_id));
    let sig = send0(&mut vm, known_sig, "value");
    let constraint = send1(&mut vm, sig, "constraintAt(_)", Value::int(0));

    let relation = send0(&mut vm, constraint, "relation");
    assert_eq!(vm.resolve_symbol(relation.as_symbol().unwrap()), "conforms");
    let subject = send0(&mut vm, constraint, "subject");
    let subject_display = send0(&mut vm, subject, "display");
    assert_eq!(vm.heap.string(subject_display.as_obj().unwrap()).as_str(), "T0");
    let trait_declaration = send0(&mut vm, constraint, "traitDeclaration");
    assert_eq!(vm.resolve_symbol(trait_declaration.as_symbol().unwrap()), "Printable");
    let arguments = send0(&mut vm, constraint, "traitArguments");
    assert_eq!(arguments.to_string(&vm), "()");
    let right_selector = vm.get_or_intern("right");
    assert!(vm.send_dynamic(constraint, right_selector, &[]).is_err());
}

#[test]
fn spec03_record_reflection_and_runtime_validation() {
    let mut vm = VM::new();
    let typing_class = vm.universe.typing_classes.get("Typing").expect("Typing class");
    let debug_sym = Value::symbol(vm.get_or_intern("debug"));
    let context_known = send1(&mut vm, Value::obj(typing_class), "contextFor(_)", debug_sym);
    let context = send0(&mut vm, context_known, "value");
    let int_class = vm.universe.classes.int_class;

    // Create record type: {x: Int}
    let f_name = Value::symbol(vm.get_or_intern("x"));
    let pair = tuple(&mut vm, vec![f_name, Value::obj(int_class)]);
    let fields_tuple = tuple(&mut vm, vec![pair]);

    let record_known = send1(&mut vm, context, "recordOf(_)", fields_tuple);
    assert_eq!(record_known.class(&vm), vm.universe.typing_classes.get("TypingKnown").unwrap());
    let record_type = send0(&mut vm, record_known, "value");
    assert_eq!(record_type.class(&vm), vm.universe.typing_classes.get("RecordType").unwrap());
    assert_eq!(send0(&mut vm, record_type, "fieldCount").as_int(), Some(1));

    let f0_name = send1(&mut vm, record_type, "fieldNameAt(_)", Value::int(0));
    assert_eq!(vm.resolve_symbol(f0_name.as_symbol().unwrap()), "x");

    // Dynamic boundary on erased generic match
    let list_inst = vm.heap.alloc(Object::List(ListObject::new(vec![Value::int(1), Value::int(2)])));
    let list_class = vm.universe.classes.list_class;
    let list_args = tuple(&mut vm, vec![Value::obj(int_class)]);
    let applied_list_known = send2(&mut vm, context, "apply(_,_)", Value::obj(list_class), list_args);
    let applied_list = send0(&mut vm, applied_list_known, "value");
    let match_result = send2(&mut vm, context, "matches(_,_)", Value::obj(list_inst), applied_list);
    assert_eq!(match_result.class(&vm), vm.universe.typing_classes.get("RelationDynamicBoundary").unwrap());

    // Runtime validate list of ints against List<Int>
    let val_result = send2(&mut vm, context, "validate(_,_)", Value::obj(list_inst), applied_list);
    assert_eq!(val_result.class(&vm), vm.universe.typing_classes.get("RelationSatisfied").unwrap());
    assert_eq!(send0(&mut vm, val_result, "value").as_bool(), Some(true));

    // Reflective construct
    let bool_class = vm.universe.classes.bool_class;
    let bool_arg = tuple(&mut vm, vec![Value::int(1)]);
    let constructed_known = send2(&mut vm, context, "construct(_,_)", Value::obj(bool_class), bool_arg);
    assert_eq!(constructed_known.class(&vm), vm.universe.typing_classes.get("TypingKnown").unwrap());
    let constructed = send0(&mut vm, constructed_known, "value");
    assert_eq!(constructed.as_bool(), Some(true));
}

#[test]
fn spec03_reflection_relations_and_kinds() {
    let mut vm = VM::new();
    let int_class = vm.universe.classes.int_class;
    let number_class = vm.universe.classes.number_class;
    let object_class = vm.universe.classes.object_class;
    let option_class = vm.universe.classes.resolve(phalcom_native_meta::UniverseKey::Option);
    let some_class = vm.universe.classes.resolve(phalcom_native_meta::UniverseKey::Some);

    // Option.kind and Some.kind must be FunctionKind (Type -> Type)
    let option_kind = send0(&mut vm, Value::obj(option_class), "kind");
    let some_kind = send0(&mut vm, Value::obj(some_class), "kind");
    let option_display = send0(&mut vm, option_kind, "display");
    let some_display = send0(&mut vm, some_kind, "display");
    assert_eq!(vm.heap.string(option_display.as_obj().unwrap()).as_str(), "Type -> Type");
    assert_eq!(vm.heap.string(some_display.as_obj().unwrap()).as_str(), "Type -> Type");

    // Subtyping on behavior: Int <: Number, Int <: Object, but not Number <: Int
    let int_sub_num = send1(&mut vm, Value::obj(int_class), "subtypeOf(_)", Value::obj(number_class));
    assert_eq!(int_sub_num.class(&vm), vm.universe.typing_classes.get("RelationSatisfied").unwrap());
    assert_eq!(send0(&mut vm, int_sub_num, "value").as_bool(), Some(true));

    let int_sub_obj = send1(&mut vm, Value::obj(int_class), "subtypeOf(_)", Value::obj(object_class));
    assert_eq!(int_sub_obj.class(&vm), vm.universe.typing_classes.get("RelationSatisfied").unwrap());

    let num_sub_int = send1(&mut vm, Value::obj(number_class), "subtypeOf(_)", Value::obj(int_class));
    assert_eq!(num_sub_int.class(&vm), vm.universe.typing_classes.get("RelationRejected").unwrap());

    // Context subtype, assignable, consistent, conforms
    let typing_class = vm.universe.typing_classes.get("Typing").expect("Typing class");
    let debug_sym = Value::symbol(vm.get_or_intern("debug"));
    let context_known = send1(&mut vm, Value::obj(typing_class), "contextFor(_)", debug_sym);
    let context = send0(&mut vm, context_known, "value");

    let sub_res = send2(&mut vm, context, "subtype(_,_)", Value::obj(int_class), Value::obj(number_class));
    assert_eq!(sub_res.class(&vm), vm.universe.typing_classes.get("RelationSatisfied").unwrap());

    let assignable_res = send2(&mut vm, context, "assignable(_,_)", Value::obj(int_class), Value::obj(number_class));
    assert_eq!(assignable_res.class(&vm), vm.universe.typing_classes.get("TypingUnavailable").unwrap());

    let consistent_res = send2(&mut vm, context, "consistent(_,_)", Value::obj(int_class), Value::obj(number_class));
    assert_eq!(consistent_res.class(&vm), vm.universe.typing_classes.get("TypingUnavailable").unwrap());

    let conforms_res = send2(&mut vm, context, "conforms(_,_)", Value::obj(int_class), Value::obj(number_class));
    assert_eq!(conforms_res.class(&vm), vm.universe.typing_classes.get("TypingUnavailable").unwrap());
}
