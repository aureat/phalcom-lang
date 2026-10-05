//! Compiler-internal transient dispatch authority verification.
//!
//! Source non-forgeability is covered in `tests/outgoing_packs_completion.rs`.
//! This pins the VM half: compiler-internal privilege is a transient dispatch
//! depth, not ambient authority that remains after the operation finishes.

use phalcom_core::bytecode::Bytecode;
use phalcom_core::error::PhResult;
use phalcom_core::heap::Object;
use phalcom_core::method::MethodKind;
use phalcom_core::method::{MemberVisibility, MethodObject, SignatureKind};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

fn internal_probe_primitive(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::unit())
}

#[test]
fn compiler_internal_dispatch_authority_is_transient() {
    let mut vm = VM::new_kernel();
    let selector = vm.interner.intern("compilerAuthorityProbe()");
    let owner = vm.universe.classes.object_class;

    let mut method = MethodObject::new_positional_primitive(selector, SignatureKind::Method(0), internal_probe_primitive, owner);
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

#[test]
fn generated_constructor_marks_only_allocator_as_compiler_internal() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "constructor_authority");
    let closure = vm
        .compile_closure(module, "class Box { @constructor new() {} }\n")
        .expect("constructor source should compile");
    let constants = vm.heap.closure(closure).callable.chunk.constants.clone();

    let mut factory_code = None;
    let mut initializer_code = None;
    for constant in constants {
        let Some(method_ref) = constant.as_obj() else { continue };
        if !matches!(vm.heap.get(method_ref), Object::Method(_)) {
            continue;
        }
        let selector = vm.heap.method(method_ref).selector();
        let body = match &vm.heap.method(method_ref).kind {
            MethodKind::Closure(body) => *body,
            MethodKind::Primitive(_) => continue,
        };
        match vm.resolve_symbol(selector) {
            "new()" => factory_code = Some(vm.heap.closure(body).callable.chunk.code.clone()),
            "init new()" => initializer_code = Some(vm.heap.closure(body).callable.chunk.code.clone()),
            _ => {}
        }
    }

    let factory_code = factory_code.expect("generated constructor factory should be compiled");
    let initializer_code = initializer_code.expect("generated constructor initializer should be compiled");
    assert_eq!(
        factory_code
            .iter()
            .filter(|instruction| matches!(instruction, Bytecode::InvokeCompilerInternal(..)))
            .count(),
        1,
        "only the generated allocator send may carry compiler-internal authority: {factory_code:?}"
    );
    assert!(
        initializer_code
            .iter()
            .all(|instruction| !matches!(instruction, Bytecode::InvokeCompilerInternal(..))),
        "the hidden initializer must retain ordinary call authority: {initializer_code:?}"
    );
}
