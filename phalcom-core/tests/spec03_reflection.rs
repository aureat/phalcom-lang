use phalcom_core::heap::{ClassObject, ListObject, Object, TupleObject};
use phalcom_core::typing::handle::MetadataPoolId;
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
use phalcom_type_meta::type_node::{TypeNode, TypeNodeEntry, TypeNodeId};
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
    Value::obj(vm.heap.alloc(Object::Tuple(TupleObject::positional(values))))
}

fn tuple_values(vm: &VM, value: Value) -> Vec<Value> {
    vm.heap.tuple(value.as_obj().expect("tuple value")).values().to_vec()
}

fn test_module() -> StableModuleRef {
    StableModuleRef {
        project: StableProjectRef::Builtin {
            namespace: "core".into(),
            version: "1.0".into(),
        },
        path: vec!["core".into()].into_boxed_slice(),
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

    let kinds = Box::new([KindNodeEntry {
        node: KindNode::Type,
        structural_fingerprint: Fingerprint128::ZERO,
    }]);

    let decl_box = StableDeclarationRef {
        module: test_module(),
        path: vec!["Box".into()].into_boxed_slice(),
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
    ]);

    let param_t_ref = StableTypeParameterRef {
        owner: StableTypeParameterOwnerRef::Declaration(decl_box.clone()),
        index: 0,
    };

    let param_t = TypeParameterRecord {
        id: param_t_ref.clone(),
        name: "T".into(),
        kind: KindNodeId(0),
        variance: VarianceRef::Covariant,
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

    let callable_value = CallableSemanticRecord {
        callable: StableCallableRef {
            owner: decl_box.clone(),
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
        parameters: vec![param_t].into_boxed_slice(),
        generic_signatures: vec![gen_sig].into_boxed_slice(),
        declarations: vec![decl_rec].into_boxed_slice(),
        aliases: Box::new([]),
        callables: vec![callable_value].into_boxed_slice(),
        fields: vec![field_val].into_boxed_slice(),
        module_roots: Box::new([]),
        runtime_roots: Box::new([]),
        occurrences: Box::new([]),
        extensions: Box::new([]),
    }
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
