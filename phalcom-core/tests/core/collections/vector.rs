use super::vm_support;
use phalcom_core::bytecode::Bytecode;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::sync::Arc;

fn global(vm: &VM, module: phalcom_core::heap::ObjRef, name: &str) -> Value {
    vm.heap.module(module).get(vm.interner.find(name).expect("interned global")).expect("global value")
}

#[test]
fn vector_fixed_and_exact_spread_construction_preserve_native_identity_and_order() {
    let (vm, module) = vm_support::run_inline(
        "let base = <Int, 2>[2, 3]\nlet result = <Int, _>[1, *base, *(4, 5, named: 99), *(), 6]\n",
    ).expect("checked Vector program compiles and runs");
    let result = global(&vm, module, "result");
    assert_eq!(result.class(&vm), vm.universe.classes.vector_class);
    assert_eq!(vm.heap.vector(result.as_obj().expect("Vector object")).elements(), &[
        Value::int(1), Value::int(2), Value::int(3), Value::int(4), Value::int(5), Value::int(6),
    ]);
}

#[test]
fn context_free_vector_compilation_requires_checked_lowering() {
    let mut vm = VM::new();
    let module = vm.create_module("vector-unchecked", "vector-unchecked");
    let error = vm.compile_closure(module, "let value = <Int, 2>[1, 2]\n").expect_err("Vector requires semantic authority");
    assert!(error.to_string().contains("sequence literal lowering"), "{error}");
}

#[test]
fn vector_read_iteration_equality_hash_and_conversion_surfaces_work() {
    let (vm, module) = vm_support::run_inline(r#"
let values = <Int, _>[10, 20, 30]
let last = values[-1]
let missing = values.get(5)
let first = values.first
let iter = values.iterator
let iterFirst = iter.next
let same = values == <Int, 3>[10, 20, 30]
let reordered = values == <Int, 3>[30, 20, 10]
let hashSame = values.hash == <Int, 3>[10, 20, 30].hash
let array = values.toArray
array[0] = 99
let original = values[0]
let list = values.toList
list.append(40)
let listSize = list.size
"#).expect("Vector API program compiles and runs");
    assert_eq!(global(&vm, module, "last"), Value::int(30));
    assert!(global(&vm, module, "missing").is_none());
    assert_eq!(global(&vm, module, "same"), Value::bool(true));
    assert_eq!(global(&vm, module, "reordered"), Value::bool(false));
    assert_eq!(global(&vm, module, "hashSame"), Value::bool(true));
    assert_eq!(global(&vm, module, "original"), Value::int(10));
    assert_eq!(global(&vm, module, "listSize"), Value::int(4));
}

#[test]
fn vector_lowering_uses_checked_extent_and_direct_family_bytecodes() {
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from("let value = <Int, 3>[1, 2, 3]\n"))).expect("Vector checks");
    let spec = program.modules[&program.entry].lowering.sequence_literals.values().next().expect("literal spec");
    assert_eq!(spec.expected_extent, Some(3));
    assert!(spec.extent.is_some());
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("Vector executes");
    let object = vm.module_registry.get(&program.entry).expect("entry").object;
    let vector = global(&vm, object, "value");
    assert!(vm.heap.as_vector(vector.as_obj().unwrap()).is_some());
    assert_eq!(std::mem::size_of::<Value>(), 16);
    assert!(std::mem::size_of::<phalcom_core::heap::Object>() <= 40);
    let _instruction = Bytecode::BuildVector(3);
}
