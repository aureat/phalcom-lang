//! Native boundary admission and activation-depth audit probes.

use phalcom_core::{
    error::{PhError, PhResult, RuntimeError},
    heap::Object,
    method::{MethodObject, SignatureKind},
    value::Value,
    vm::{MAX_NATIVE_ACTIVATION_DEPTH, VM},
};

fn recurse_native(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let depth = args[0].as_int().expect("recursion depth is an integer");
    let selector = vm.get_or_intern("auditNativeRecurse(_)");
    vm.send_dynamic(*receiver, selector, &[Value::int(depth + 1)])
}

#[test]
fn public_native_arity_failure_is_controlled() {
    let mut vm = VM::new_native();
    let selector = vm.get_or_intern("+(_)");

    let result = vm.send_dynamic(Value::int(1), selector, &[]);

    assert!(matches!(result, Err(PhError::Runtime(RuntimeError::Arity { .. }))));

    let valid_selector = vm.get_or_intern("+(_)");
    assert_eq!(
        vm.send_dynamic(Value::int(1), valid_selector, &[Value::int(2)])
            .expect("a subsequent valid send must still execute"),
        Value::int(3)
    );
}

#[test]
fn native_only_recursion_is_bounded_separately() {
    let mut vm = VM::new_kernel();
    let class = vm.universe.classes.int_class;
    let selector = vm.get_or_intern("auditNativeRecurse(_)");
    let method = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_positional_primitive(
        selector,
        SignatureKind::Method(1),
        recurse_native,
        class,
    ))));
    vm.heap.class_mut(class).add_method(selector, method);

    let result = vm.send_dynamic(Value::int(1), selector, &[Value::int(0)]);

    let err = result.expect_err("native recursion must be bounded");
    let (underlying, origin) = err.into_underlying();
    assert!(origin.is_some());
    assert!(matches!(
        underlying,
        PhError::Runtime(RuntimeError::DepthExceeded {
            what: "native activation depth",
            limit: MAX_NATIVE_ACTIVATION_DEPTH,
        })
    ));
}
