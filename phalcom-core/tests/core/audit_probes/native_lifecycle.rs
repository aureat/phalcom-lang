//! Native method call lifecycle and error traceback audit probes.

use phalcom_core::{
    error::{PhResult, RuntimeError},
    heap::Object,
    method::{ArgumentView, CallOutcome, MethodObject, Signature, SignatureKind},
    value::Value,
    vm::VM,
};

fn fail(_: &mut VM, _: &Value, _: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::Internal("audit outer/inner failure".into()).into())
}

fn ok(_: &mut VM, _: &Value, _: &[Value]) -> PhResult<Value> {
    Ok(Value::int(7))
}

fn outer_success_then_fail(vm: &mut VM, r: &Value, _: &[Value]) -> PhResult<Value> {
    let s = vm.get_or_intern("auditOk()");
    vm.send_dynamic(*r, s, &[])?;
    fail(vm, r, &[])
}

fn outer_catch_then_fail(vm: &mut VM, r: &Value, _: &[Value]) -> PhResult<Value> {
    let s = vm.get_or_intern("auditFail()");
    assert!(vm.send_dynamic(*r, s, &[]).is_err());
    fail(vm, r, &[])
}

fn outer_catch_then_ok(vm: &mut VM, r: &Value, _: &[Value]) -> PhResult<Value> {
    let s = vm.get_or_intern("auditFail()");
    assert!(vm.send_dynamic(*r, s, &[]).is_err());
    let s = vm.get_or_intern("auditOk()");
    vm.send_dynamic(*r, s, &[])
}

fn outer_error_after_success(vm: &mut VM, r: &Value, _: &[Value]) -> PhResult<Value> {
    let s = vm.get_or_intern("auditOk()");
    vm.send_dynamic(*r, s, &[])?;
    Err(RuntimeError::Internal("outer legacy failure".into()).into())
}

fn outer_error_after_failure(vm: &mut VM, r: &Value, _: &[Value]) -> PhResult<Value> {
    let s = vm.get_or_intern("auditFail()");
    vm.send_dynamic(*r, s, &[])?;
    unreachable!("nested failure must propagate")
}

fn shape_error_after_success(vm: &mut VM, receiver: Value, _args: ArgumentView) -> PhResult<CallOutcome> {
    let s = vm.get_or_intern("auditOk()");
    vm.send_dynamic(receiver, s, &[])?;
    Err(RuntimeError::Internal("outer shape failure".into()).into())
}

fn shape_error_after_failure(vm: &mut VM, receiver: Value, _args: ArgumentView) -> PhResult<CallOutcome> {
    let s = vm.get_or_intern("auditFail()");
    vm.send_dynamic(receiver, s, &[])?;
    unreachable!("nested failure must propagate")
}

#[test]
fn native_lifecycle_nested_dispatch_and_error_propagation() {
    let cases = [
        ("auditControl()", fail as fn(&mut VM, &Value, &[Value]) -> PhResult<Value>, true),
        ("auditOuterSuccessFail()", outer_success_then_fail, true),
        ("auditOuterCatchFail()", outer_catch_then_fail, true),
        ("auditOuterCatchOk()", outer_catch_then_ok, false),
    ];

    for (name, f, expect_err) in cases {
        let mut vm = VM::new_kernel();
        let class = vm.universe.classes.int_class;
        for (n, p) in [
            ("auditOk()", ok as fn(&mut VM, &Value, &[Value]) -> PhResult<Value>),
            ("auditFail()", fail),
            (name, f),
        ] {
            let s = vm.get_or_intern(n);
            let m = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_positional_primitive(
                s,
                SignatureKind::Method(0),
                p,
                class,
            ))));
            vm.heap.class_mut(class).add_method(s, m);
        }
        let s = vm.get_or_intern(name);
        let result = vm.send_dynamic(Value::int(1), s, &[]);
        assert_eq!(result.is_err(), expect_err, "case {name}");
        if !expect_err {
            assert_eq!(result.unwrap(), Value::int(7));
        }
    }
}

#[test]
fn native_error_provenance_follows_escaping_producer() {
    let mut vm = VM::new_kernel();
    let class = vm.universe.classes.int_class;
    for (name, f) in [
        ("auditOk()", ok as fn(&mut VM, &Value, &[Value]) -> PhResult<Value>),
        ("auditFail()", fail),
        ("auditOuterSuccessFail()", outer_error_after_success),
        ("auditOuterFailureFail()", outer_error_after_failure),
    ] {
        let selector = vm.get_or_intern(name);
        let method = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_positional_primitive(
            selector,
            SignatureKind::Method(0),
            f,
            class,
        ))));
        vm.heap.class_mut(class).add_method(selector, method);
    }
    let receiver = Value::int(1);
    let selector = vm.get_or_intern("auditOuterSuccessFail()");
    let error = vm.send_dynamic(receiver, selector, &[]).expect_err("outer failure");
    let origin = error.native_origin().expect("outer origin");
    assert_eq!(vm.resolve_symbol(origin.selector), "auditOuterSuccessFail()");
    assert_eq!(vm.resolve_symbol(origin.class), "Int");

    let selector = vm.get_or_intern("auditOuterFailureFail()");
    let error = vm.send_dynamic(receiver, selector, &[]).expect_err("inner failure");
    let origin = error.native_origin().expect("inner origin");
    assert_eq!(vm.resolve_symbol(origin.selector), "auditFail()");
    assert_eq!(vm.resolve_symbol(origin.class), "Int");

    let mut vm = VM::new_kernel();
    let class = vm.universe.classes.int_class;
    for (name, f) in [("auditOk()", ok as fn(&mut VM, &Value, &[Value]) -> PhResult<Value>), ("auditFail()", fail)] {
        let selector = vm.get_or_intern(name);
        let method = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_positional_primitive(
            selector,
            SignatureKind::Method(0),
            f,
            class,
        ))));
        vm.heap.class_mut(class).add_method(selector, method);
    }
    for (name, f) in [
        (
            "auditShapeSuccessFail()",
            shape_error_after_success as fn(&mut VM, Value, ArgumentView) -> PhResult<CallOutcome>,
        ),
        ("auditShapeFailureFail()", shape_error_after_failure),
    ] {
        let selector = vm.get_or_intern(name);
        let signature = Signature::new(selector, SignatureKind::Method(0), 0);
        let method = vm
            .heap
            .alloc(Object::Method(Box::new(MethodObject::new_shape_primitive(selector, signature, f, class))));
        vm.heap.class_mut(class).add_method(selector, method);
    }
    let selector = vm.get_or_intern("auditShapeSuccessFail()");
    let error = vm.send_dynamic(receiver, selector, &[]).expect_err("shape outer failure");
    let origin = error.native_origin().expect("shape outer origin");
    assert_eq!(vm.resolve_symbol(origin.selector), "auditShapeSuccessFail()");
    let selector = vm.get_or_intern("auditShapeFailureFail()");
    let error = vm.send_dynamic(receiver, selector, &[]).expect_err("shape inner failure");
    let origin = error.native_origin().expect("shape inner origin");
    assert_eq!(vm.resolve_symbol(origin.selector), "auditFail()");
}
