use phalcom_core::{diagnostics::{style::RenderConfig, traceback::render_traceback}, error::{PhResult, RuntimeError}, heap::Object, method::{MethodObject, SignatureKind}, value::Value, vm::VM};
fn fail(_: &mut VM, _: &Value, _: &[Value]) -> PhResult<Value> { Err(RuntimeError::Internal("audit outer/inner failure".into()).into()) }
fn ok(_: &mut VM, _: &Value, _: &[Value]) -> PhResult<Value> { Ok(Value::int(7)) }
fn state(vm: &VM, label: &str) { println!("{label}: selector={:?} class={:?}", vm.native_selector.map(|s| vm.resolve_symbol(s)), vm.native_class.map(|s| vm.resolve_symbol(s))); }
fn outer_success_then_fail(vm: &mut VM, r: &Value, _: &[Value]) -> PhResult<Value> {
    let s = vm.get_or_intern("auditOk()");
    vm.send_dynamic(*r, s, &[])?;
    state(vm, "inside outer after nested success");
    fail(vm, r, &[])
}
fn outer_catch_then_fail(vm: &mut VM, r: &Value, _: &[Value]) -> PhResult<Value> {
    let s = vm.get_or_intern("auditFail()");
    assert!(vm.send_dynamic(*r, s, &[]).is_err());
    state(vm, "inside outer after caught nested error");
    fail(vm, r, &[])
}
fn outer_catch_then_ok(vm: &mut VM, r: &Value, _: &[Value]) -> PhResult<Value> {
    let s = vm.get_or_intern("auditFail()");
    assert!(vm.send_dynamic(*r, s, &[]).is_err());
    let s = vm.get_or_intern("auditOk()");
    let result = vm.send_dynamic(*r, s, &[]);
    state(vm, "inside outer after caught error then success");
    result
}
fn main() {
    for (name, f) in [("auditControl()", fail as fn(&mut VM, &Value, &[Value]) -> PhResult<Value>), ("auditOuterSuccessFail()", outer_success_then_fail), ("auditOuterCatchFail()", outer_catch_then_fail), ("auditOuterCatchOk()", outer_catch_then_ok)] {
        let mut vm = VM::new_kernel();
        let class = vm.universe.classes.int_class;
        for (n, p) in [("auditOk()", ok as fn(&mut VM, &Value, &[Value]) -> PhResult<Value>), ("auditFail()", fail), (name, f)] {
            let s = vm.get_or_intern(n);
            let m = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_primitive(s, SignatureKind::Method(0), p, class))));
            vm.heap.class_mut(class).add_method(s, m);
        }
        let s = vm.get_or_intern(name);
        let result = vm.send_dynamic(Value::int(1), s, &[]);
        println!("CASE {name} result={result:?}");
        state(&vm, "after outer");
        if let Err(e) = result { render_traceback(&mut vm, &e, &RenderConfig::default(), false, true); }
    }
}
