//! Disassembly golden for the `SuperSend` opcode (U-INH §3.4, ADR-0040).
//!
//! A `super.sel(…)` body must lower to [`Bytecode::SuperSend`] carrying the
//! defining-class constant, **not** an ordinary [`Bytecode::Invoke`]. Since the
//! send lives inside a method closure (not the top-level chunk), this walks the
//! compiled constants to the method bodies and inspects their opcodes directly.

use phalcom_core::bytecode::Bytecode;
use phalcom_core::heap::Object;
use phalcom_core::method::MethodKind;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

/// Collects the opcodes of every method-body closure reachable from the
/// top-level chunk's constant pool.
fn method_body_opcodes(vm: &VM, top_closure: phalcom_core::heap::ObjRef) -> Vec<Bytecode> {
    let mut ops = Vec::new();
    let constants = vm.heap.closure(top_closure).callable.chunk.constants.clone();
    for constant in constants {
        if let Value::Obj(obj) = constant
            && let Object::Method(method) = vm.heap.get(obj)
            && let MethodKind::Closure(body) = method.kind
        {
            ops.extend(vm.heap.closure(body).callable.chunk.code.iter().copied());
        }
    }
    ops
}

#[test]
fn super_send_emits_supersend_not_invoke() {
    let source = "\
class Animal {
  construct new() { }
  speak => \"generic\"
}
class Dog extends Animal {
  construct new() { }
  speak => super.speak
}
";
    let mut vm = VM::new();
    let module = vm.create_module("main", "<main>");
    let top = vm.compile_closure(module, source).expect("source should compile");

    let ops = method_body_opcodes(&vm, top);

    assert!(
        ops.iter().any(|op| matches!(op, Bytecode::SuperSend(..))),
        "a `super.speak` body must emit a SuperSend opcode; opcodes were: {ops:?}"
    );
}
