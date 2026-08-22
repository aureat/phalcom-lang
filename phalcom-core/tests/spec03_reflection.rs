use phalcom_core::heap::{Object, TupleObject};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

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

fn tuple(vm: &mut VM, values: Vec<Value>) -> Value {
    Value::obj(vm.heap.alloc(Object::Tuple(TupleObject::positional(values))))
}

fn tuple_values(vm: &VM, value: Value) -> Vec<Value> {
    vm.heap.tuple(value.as_obj().expect("tuple value")).values().to_vec()
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
