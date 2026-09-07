//! Immediate Option object-model, constructor, and allocation regressions.

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_core::adt::{RuntimeAdtRepresentation, RuntimeVariantShape};
use phalcom_core::primitive::option::{some_call, some_new};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_semantic::core_surface::CoreDeclarationIds;
use phalcom_semantic::identity::VariantId;

fn send0(vm: &mut VM, receiver: Value, selector: &str) -> Value {
    let selector = vm.get_or_intern(selector);
    vm.send_dynamic(receiver, selector, &[]).expect("send should succeed")
}

#[test]
fn constructors_return_same_immediate_representation() {
    let mut vm = VM::new();
    let some_class = Value::obj(vm.universe.classes.some_class);

    let call_selector = vm.get_or_intern("call(_)");
    let new_selector = vm.get_or_intern("new(_)");
    let via_call = vm.send_dynamic(some_class, call_selector, &[Value::int(42)]).expect("Some.call");
    let via_new = vm.send_dynamic(some_class, new_selector, &[Value::int(42)]).expect("Some.new");

    assert_eq!(via_call.option_depth(), 1);
    assert_eq!(via_call, via_new);
    assert!(via_call.as_obj().is_none());
}

#[test]
fn immediate_option_reflection_and_dispatch_are_ordinary() {
    let mut vm = VM::new();
    let classes = vm.universe.classes;
    let some = some_call(&mut vm, &Value::obj(classes.some_class), &[Value::int(7)]).expect("Some.call");
    let some_none = some_call(&mut vm, &Value::obj(classes.some_class), &[Value::none()]).expect("Some(None)");

    assert_eq!(some.class(&vm), classes.some_class);
    assert_eq!(some_none.class(&vm), classes.some_class);
    assert_eq!(Value::none().class(&vm), classes.none_class);
    assert_eq!(send0(&mut vm, Value::none(), "class"), Value::obj(classes.none_class));
    assert_ne!(send0(&mut vm, Value::none(), "class"), Value::none());
    assert_eq!(send0(&mut vm, some, "class"), Value::obj(classes.some_class));
    assert_eq!(send0(&mut vm, some, "isSome").as_bool(), Some(true));
    assert_eq!(send0(&mut vm, some, "isNone").as_bool(), Some(false));
    assert_eq!(send0(&mut vm, Value::none(), "isSome").as_bool(), Some(false));
    assert_eq!(send0(&mut vm, Value::none(), "isNone").as_bool(), Some(true));
    assert_eq!(send0(&mut vm, some, "toString").to_string(&vm), "Some(7)");
    assert_eq!(send0(&mut vm, some_none, "toString").to_string(&vm), "Some(None)");
    assert_ne!(some_none, Value::none());
}

#[test]
fn canonical_option_none_is_immediate_and_registered_as_singleton_variant() {
    let vm = VM::new();
    let option = CoreDeclarationIds::default().option;
    let none = VariantId::new(option, Selector::getter("None").expect("valid None getter"));
    let runtime_none = vm.adt_registry.variant_by_semantic(&none).expect("canonical Option::None must be registered");
    let descriptor = vm.adt_registry.variant_descriptor(runtime_none).expect("canonical Option::None descriptor");

    assert_eq!(descriptor.shape, RuntimeVariantShape::Singleton);
    assert_eq!(descriptor.payload_arity, 0);
    assert_eq!(descriptor.singleton, Some(Value::none()));
    assert_eq!(vm.runtime_variant_of(Value::none()), Some(runtime_none));
    assert_eq!(vm.heap.class(vm.universe.classes.some_class).field_count, 0);
    assert_eq!(vm.heap.class(vm.universe.classes.none_class).field_count, 0);

    // `none_class` is the hidden runtime behavior class for the
    // Option::None exact case. It is not a separate semantic declaration.
    assert_eq!(Value::none().class(&vm), vm.universe.classes.none_class);
}

#[test]
fn option_none_has_variant_identity_without_independent_class_identity() {
    let vm = VM::new();
    let ids = CoreDeclarationIds::default();
    let option = ids.option;
    let some = VariantId::new(
        option.clone(),
        Selector::method("Some", vec![SelectorSlot::Positional]).expect("valid Some constructor"),
    );
    let none = VariantId::new(option.clone(), Selector::getter("None").expect("valid None getter"));

    let enum_id = vm.adt_registry.enum_by_declaration(&option).expect("canonical Option enum must be registered");
    let enum_descriptor = vm.adt_registry.enum_descriptor(enum_id).expect("canonical Option enum descriptor");

    assert_eq!(enum_descriptor.representation, RuntimeAdtRepresentation::NativeOption);
    assert_eq!(enum_descriptor.root_class, vm.universe.classes.option_class);
    assert_eq!(enum_descriptor.variants.len(), 2);

    let runtime_some = vm
        .adt_registry
        .variant_by_semantic(&some)
        .expect("canonical Option::Some(_) must be registered");
    let runtime_none = vm.adt_registry.variant_by_semantic(&none).expect("canonical Option::None must be registered");
    assert!(enum_descriptor.variants.contains(&runtime_some));
    assert!(enum_descriptor.variants.contains(&runtime_none));

    let some_descriptor = vm.adt_registry.variant_descriptor(runtime_some).expect("canonical Option::Some(_) descriptor");
    assert_eq!(some_descriptor.semantic_id, some);
    assert_eq!(some_descriptor.enum_id, enum_id);
    assert_eq!(some_descriptor.shape, RuntimeVariantShape::Constructor);
    assert_eq!(some_descriptor.payload_arity, 1);
    assert_eq!(some_descriptor.behavior_class, vm.universe.classes.some_class);

    let none_descriptor = vm.adt_registry.variant_descriptor(runtime_none).expect("canonical Option::None descriptor");
    assert_eq!(none_descriptor.semantic_id, none);
    assert_eq!(none_descriptor.semantic_id.owner, option);
    assert_eq!(none_descriptor.enum_id, enum_id);
    assert_eq!(none_descriptor.shape, RuntimeVariantShape::Singleton);
    assert_eq!(none_descriptor.payload_arity, 0);
    assert_eq!(none_descriptor.behavior_class, vm.universe.classes.none_class);
    assert_eq!(none_descriptor.singleton, Some(Value::none()));
    assert_eq!(vm.runtime_variant_of(Value::none()), Some(runtime_none));

    assert_eq!(vm.case_behavior_class(Value::none()), Some(vm.universe.classes.none_class));
    assert_eq!(Value::none().class(&vm), vm.universe.classes.none_class);
    assert_ne!(vm.universe.classes.none_class, enum_descriptor.root_class);
}

#[test]
fn wrapping_never_allocates_and_deep_nesting_supported() {
    let mut vm = VM::new();
    let before = vm.heap.live_count();
    let some_class = Value::obj(vm.universe.classes.some_class);
    let mut value = Value::int(1);

    for expected_depth in 1..=100 {
        value = some_call(&mut vm, &some_class, &[value]).expect("bounded Some construction");
        assert_eq!(value.option_depth(), expected_depth);
        assert!(value.as_obj().is_none());
        assert_eq!(vm.heap.live_count(), before, "immediate wrapping must not allocate");
    }
}

#[test]
fn some_new_remains_compatibility_alias() {
    let mut vm = VM::new();
    let some_receiver = Value::obj(vm.universe.classes.some_class);
    let canonical = some_call(&mut vm, &some_receiver, &[Value::bool(true)]).expect("Some.call");
    let compatibility = some_new(&mut vm, &some_receiver, &[Value::bool(true)]).expect("Some.new");
    assert_eq!(canonical, compatibility);
}
