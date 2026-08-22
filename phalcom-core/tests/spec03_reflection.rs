use phalcom_core::heap::{Object, TupleObject};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

fn send0(vm: &mut VM, receiver: Value, selector: &str) -> Value {
    vm.send_dynamic(receiver, vm.get_or_intern(selector), &[]).unwrap_or_else(|error| panic!("send `{selector}` failed: {error}"))
}

fn send1(vm: &mut VM, receiver: Value, selector: &str, arg: Value) -> Value {
    vm.send_dynamic(receiver, vm.get_or_intern(selector), &[arg]).unwrap_or_else(|error| panic!("send `{selector}` failed: {error}"))
}

fn send2(vm: &mut VM, receiver: Value, selector: &str, first: Value, second: Value) -> Value {
    vm.send_dynamic(receiver, vm.get_or_intern(selector), &[first, second]).unwrap_or_else(|error| panic!("send `{selector}` failed: {error}"))
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
    assert_eq!(send0(&mut vm, context, "profile").symbol_value().map(|symbol| vm.resolve_symbol(symbol)), Some("RuntimePublic"));

    let capabilities = tuple_values(&vm, send0(&mut vm, context, "capabilities"));
    let capability_names = capabilities
        .iter()
        .filter_map(Value::symbol_value)
        .map(|symbol| vm.resolve_symbol(symbol).to_string())
        .collect::<Vec<_>>();
    assert_eq!(capability_names, ["ObservePublicTypes", "ObserveSignatures", "ConstructTypeForms", "EvaluateRelations"]);

    let observe_public_types = Value::symbol(vm.get_or_intern("ObservePublicTypes"));
    let allowed = tuple(&mut vm, vec![observe_public_types]);
    let restricted = send1(&mut vm, context, "restrictTo(_)" , allowed);
    assert_eq!(tuple_values(&vm, send0(&mut vm, restricted, "capabilities")).len(), 1);
}

#[test]
fn spec03_reifies_applied_type_forms_and_returns_bounded_results() {
    let mut vm = VM::new();
    let typing_class = vm.universe.typing_classes.get("Typing").expect("Typing class");
    let list_class = vm.universe.classes.list_class;
    let int_class = vm.universe.classes.int_class;
    let type_class = vm.universe.typing_classes.get("Type").expect("Type class");
    let descriptor_class = vm.universe.typing_classes.get("TypeDescriptor").expect("TypeDescriptor class");
    let known_class = vm.universe.typing_classes.get("TypingKnown").expect("TypingKnown class");
    let satisfied_class = vm.universe.typing_classes.get("RelationSatisfied").expect("RelationSatisfied class");

    let context = send0(&mut vm, Value::obj(typing_class), "current");
    let arguments = tuple(&mut vm, vec![Value::obj(int_class)]);
    let known = send2(&mut vm, context, "apply(_,_)" , Value::obj(list_class), arguments);
    assert_eq!(known.class(&vm), known_class);

    let descriptor = send0(&mut vm, known, "value");
    assert_eq!(descriptor.class(&vm), descriptor_class);
    assert_eq!(send0(&mut vm, descriptor, "kind"), Value::obj(type_class));
    let display = send0(&mut vm, descriptor, "display");
    let display_text = vm.heap.string(display.as_obj().expect("display string")).as_str().to_owned();
    assert_eq!(display_text, "List<Int>");
    assert_eq!(send0(&mut vm, descriptor, "argumentCount").as_int(), Some(1));

    let argument = send1(&mut vm, descriptor, "argumentAt(_)" , Value::int(0));
    let argument_display = send0(&mut vm, argument, "display");
    let argument_display_text = vm.heap.string(argument_display.as_obj().expect("argument display string")).as_str().to_owned();
    assert_eq!(argument_display_text, "Int");

    let relation = send1(&mut vm, descriptor, "subtypeOf(_)" , descriptor);
    assert_eq!(relation.class(&vm), satisfied_class);
    assert_eq!(send0(&mut vm, relation, "value").as_bool(), Some(true));
}
