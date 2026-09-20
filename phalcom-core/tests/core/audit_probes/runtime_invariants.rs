//! Runtime invariants and VM execution audit probes.

use phalcom_core::{
    chunk::Chunk,
    heap::{ClassObject, Object},
    method::{MethodObject, SignatureKind},
    value::Value,
    vm::VM,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn recurse(vm: &mut VM, receiver: &Value, args: &[Value]) -> phalcom_core::error::PhResult<Value> {
    let n = args[0].as_int().unwrap();
    if n == 0 {
        return Ok(Value::int(40));
    }
    let selector = vm.get_or_intern("auditRecurse(_)");
    vm.send_dynamic(*receiver, selector, &[Value::int(n - 1)])
}

#[test]
#[ignore = "AUD-RUNTIME-001: chunk constant pool index overflow for indices > 65535"]
fn chunk_constant_pool_capacity() {
    let mut chunk = Chunk::new();
    for i in 0..=65536 {
        let index = chunk.add_constant(Value::int(i));
        if [0, 1, 65534, 65535, 65536].contains(&i) {
            assert_eq!(chunk.constants[index as usize].as_int(), Some(i));
        }
    }
}

#[test]
fn gc_lexical_owner_survives_collection() {
    let mut vm = VM::new_kernel();
    let module = vm.create_module("audit", "");
    let closure = vm.compile_closure(module, "42\n").expect("kernel literal compile");
    let owner = vm.heap.alloc_class(ClassObject::bare("AuditOwner"));
    vm.heap.closure_mut(closure).lexical_class = Some(owner);
    vm.push_root_for_test(Value::obj(closure));
    vm.force_gc();
    assert!(vm.heap.try_get(owner).is_some());
    vm.pop_root_for_test();
}

#[test]
fn native_recursion_depth() {
    let mut vm = VM::new_kernel();
    let class = vm.universe.classes.int_class;
    let selector = vm.get_or_intern("auditRecurse(_)");
    let method = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_primitive(
        selector,
        SignatureKind::Method(1),
        recurse,
        class,
    ))));
    vm.heap.class_mut(class).add_method(selector, method);
    let result = vm.send_dynamic(Value::int(1), selector, &[Value::int(40)]);
    assert_eq!(result.unwrap(), Value::int(40));
}

#[test]
#[ignore = "AUD-RUNTIME-004: native entry arity check does not panic/err on shipping dispatch"]
fn native_arity_mismatch_panics() {
    let mut vm = VM::new_native();
    let selector = vm.get_or_intern("+(_)");
    let result = catch_unwind(AssertUnwindSafe(|| vm.send_dynamic(Value::int(1), selector, &[])));
    assert!(result.is_err());
}
