//! Immediate Option object-model, constructor, and allocation regressions.

use phalcom_core::heap::CORE_MODULE_NAME;
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
    let some_class = Value::Obj(vm.universe.classes.some_class);

    let call_selector = vm.get_or_intern("call(_)");
    let new_selector = vm.get_or_intern("new(_)");
    let via_call = vm.send_dynamic(some_class, call_selector, &[Value::Int(42)]).expect("Some.call");
    let via_new = vm.send_dynamic(some_class, new_selector, &[Value::Int(42)]).expect("Some.new");

    assert!(matches!(via_call, Value::Some1(_)));
    assert_eq!(via_call, via_new);
    assert!(via_call.as_obj().is_none());
}

#[test]
fn immediate_option_reflection_and_dispatch_are_ordinary() {
    let mut vm = VM::new();
    let classes = vm.universe.classes;
    let some = some_call(&mut vm, &Value::Obj(classes.some_class), &[Value::Int(7)]).expect("Some.call");
    let some_none = some_call(&mut vm, &Value::Obj(classes.some_class), &[Value::None]).expect("Some(None)");

    assert_eq!(some.class(&vm), classes.some_class);
    assert_eq!(some_none.class(&vm), classes.some_class);
    assert_eq!(Value::None.class(&vm), classes.none_class);
    assert_eq!(send0(&mut vm, Value::None, "class"), Value::Obj(classes.none_class));
    assert_ne!(send0(&mut vm, Value::None, "class"), Value::None);
    assert_eq!(send0(&mut vm, some, "class"), Value::Obj(classes.some_class));
    assert!(matches!(send0(&mut vm, some, "isSome"), Value::Bool(true)));
    assert!(matches!(send0(&mut vm, some, "isNone"), Value::Bool(false)));
    assert!(matches!(send0(&mut vm, Value::None, "isSome"), Value::Bool(false)));
    assert!(matches!(send0(&mut vm, Value::None, "isNone"), Value::Bool(true)));
    assert_eq!(send0(&mut vm, some, "toString").to_string(&vm), "Some(7)");
    assert_eq!(send0(&mut vm, some_none, "toString").to_string(&vm), "Some(None)");
    assert_ne!(some_none, Value::None);
}

#[test]
fn bootstrap_binds_immediate_none_and_zero_field_variants() {
    let mut vm = VM::new();
    let core = vm.get_module_from_str(CORE_MODULE_NAME).expect("core module");
    let none_name = vm.interner.intern("None");

    assert_eq!(vm.heap.module(core).get(none_name), Some(Value::None));
    assert_eq!(vm.heap.class(vm.universe.classes.some_class).field_count, 0);
    assert_eq!(vm.heap.class(vm.universe.classes.none_class).field_count, 0);
    assert_eq!(Value::None.class(&vm), vm.universe.classes.none_class);
}

#[test]
fn wrapping_never_allocates_and_depth_eight_is_rejected() {
    let mut vm = VM::new();
    let before = vm.heap.live_count();
    let some_class = Value::Obj(vm.universe.classes.some_class);
    let mut value = Value::Int(1);

    for expected_depth in 1..=7 {
        value = some_call(&mut vm, &some_class, &[value]).expect("bounded Some construction");
        assert_eq!(value.option_depth(), expected_depth);
        assert!(value.as_obj().is_none());
        assert_eq!(vm.heap.live_count(), before, "immediate wrapping must not allocate");
    }

    let error = some_call(&mut vm, &some_class, &[value]).expect_err("eighth layer must fail");
    assert!(error.to_string().contains("Option nesting limit exceeded (7)"));
}

#[test]
fn some_new_remains_compatibility_alias() {
    let mut vm = VM::new();
    let some_receiver = Value::Obj(vm.universe.classes.some_class);
    let canonical = some_call(&mut vm, &some_receiver, &[Value::Bool(true)]).expect("Some.call");
    let compatibility = some_new(&mut vm, &some_receiver, &[Value::Bool(true)]).expect("Some.new");
    assert_eq!(canonical, compatibility);
}
