// Standalone read-only runtime probe; see ../README.md for reproduction.
use phalcom_core::{compiler::lib::UnitKind, error::PhResult, heap::{Object, FiberStatus}, method::{MethodKind, PrimitiveFn}, value::Value, vm::VM};
fn inspect(vm: &mut VM, _: &Value, _: &[Value]) -> PhResult<Value> {
    let current = phalcom_core::primitive::fiber::fiber_current(vm, &Value::unit(), &[])?.as_obj().unwrap();
    let active = vm.heap.fiber(current);
    assert_eq!(active.status, FiberStatus::Running);
    assert!(active.stack.is_empty() && active.frames.is_empty());
    let parent = vm.heap.fiber(active.resumer.unwrap());
    assert_eq!(parent.status, FiberStatus::Running); // documents defect, not desired behavior
    assert!(!parent.stack.is_empty() && !parent.frames.is_empty());
    let count = vm.heap.iter_handles_for_test().iter().filter(|id| matches!(vm.heap.get(**id), Object::Fiber(f) if f.status == FiberStatus::Running)).count();
    assert_eq!(count, 2);
    println!("Running statuses: {count}; current buffers empty; parent buffers parked");
    Ok(Value::unit())
}
fn main() {
    let mut vm = VM::new();
    let module = vm.create_module("audit", "state-inspection");
    let closure = vm.compile_closure_as(module, "class Inspector { check() {} }\nconst f = Fiber.new || { Inspector.new().check(); Fiber.yield(5); 9 }\nf.call()\n", UnitKind::File).unwrap();
    let method = vm.heap.closure(closure).callable.chunk.constants.iter().find_map(|value| {
        let id = value.as_obj()?;
        (matches!(vm.heap.get(id), Object::Method(_)) && vm.resolve_symbol(vm.heap.method(id).selector()) == "check()").then_some(id)
    }).unwrap();
    vm.heap.method_mut(method).kind = MethodKind::Primitive(PrimitiveFn::Legacy(inspect));
    vm.run_cell(module, closure).unwrap();
}
