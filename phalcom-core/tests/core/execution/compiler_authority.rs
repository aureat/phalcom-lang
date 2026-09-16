//! Compiler-internal transient dispatch authority verification.
//!
//! Source non-forgeability is covered in `tests/outgoing_packs_completion.rs`.
//! This pins the VM half: compiler-internal privilege is a transient dispatch
//! depth, not ambient authority that remains after the operation finishes.

use phalcom_core::error::PhResult;
use phalcom_core::heap::Object;
use phalcom_core::method::{MemberVisibility, MethodObject, SignatureKind};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

fn internal_probe_primitive(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::unit())
}

#[test]
fn compiler_internal_dispatch_authority_is_transient() {
    let mut vm = VM::new();
    let selector = vm.interner.intern("_$compilerAuthorityProbe()");
    let owner = vm.universe.classes.object_class;

    let mut method = MethodObject::new_primitive(selector, SignatureKind::Method(0), internal_probe_primitive, owner);
    method.visibility = MemberVisibility::Internal;
    let method = vm.heap.alloc(Object::Method(Box::new(method)));

    assert!(
        vm.authorize_method_access(method).is_err(),
        "ordinary execution must not authorize an Internal member"
    );

    vm.compiler_internal_dispatch_depth = 1;
    assert!(
        vm.authorize_method_access(method).is_ok(),
        "compiler-owned dynamic dispatch must be able to authorize its Internal target"
    );

    vm.compiler_internal_dispatch_depth = 0;
    assert!(
        vm.authorize_method_access(method).is_err(),
        "authority must disappear immediately after the compiler-owned dispatch operation"
    );
}
