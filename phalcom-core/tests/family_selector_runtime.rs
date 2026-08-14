use phalcom_core::bytecode::{Bytecode, FamilySpecKind};
use phalcom_core::compiler::lib::UnitKind;
use phalcom_core::heap::Object;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

#[test]
fn make_family_uses_explicit_exact_discriminator_and_allows_future_method() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<family>");
    let closure = vm
        .compile_closure_as(module, "const f = 1::future()\n", UnitKind::File)
        .expect("exact family compiles");
    let chunk = &vm.heap.closure(closure).callable.chunk;
    assert!(chunk.code.iter().any(|opcode| matches!(
        opcode,
        Bytecode::MakeFamily {
            kind: FamilySpecKind::Exact,
            ..
        }
    )));
    vm.run_cell(module, closure).expect("family construction does not resolve target method");
}

#[test]
fn make_family_compiles_pattern_object_without_punctuation_heuristic() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<family>");
    let closure = vm
        .compile_closure_as(module, "const f = 1::future(...)\n", UnitKind::File)
        .expect("pattern family compiles");
    let chunk = &vm.heap.closure(closure).callable.chunk;
    let pattern = chunk.constants.iter().find_map(|constant| match constant {
        Value::Obj(id) if matches!(vm.heap.get(*id), Object::SelectorPattern(_)) => Some(*id),
        _ => None,
    });
    assert!(pattern.is_some(), "pattern must be a first-class immutable heap object");
    assert!(chunk.code.iter().any(|opcode| matches!(
        opcode,
        Bytecode::MakeFamily {
            kind: FamilySpecKind::Pattern,
            ..
        }
    )));
    vm.run_cell(module, closure).expect("pattern family construction succeeds");
}
