//! Immediate Option object-model, constructor, and allocation regressions.

use phalcom_core::primitive::nil::{some_call, some_new};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

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
fn bootstrap_binds_immediate_none_and_zero_field_variants() {
    let mut vm = VM::new();
    let core = vm.core_module().expect("core module");
    let none_name = vm.interner.intern("None");

    assert_eq!(vm.heap.module(core).get(none_name), Some(Value::none()));
    assert_eq!(vm.heap.class(vm.universe.classes.some_class).field_count, 0);
    assert_eq!(vm.heap.class(vm.universe.classes.none_class).field_count, 0);
    assert_eq!(Value::none().class(&vm), vm.universe.classes.none_class);
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
