use super::vm_support;
use phalcom_core::bytecode::Bytecode;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::sync::Arc;

fn global(vm: &VM, module: phalcom_core::heap::ObjRef, name: &str) -> Value {
    vm.heap
        .module(module)
        .get(vm.interner.find(name).expect("interned global"))
        .expect("global value")
}

#[test]
fn vector_fixed_and_exact_spread_construction_preserve_native_identity_and_order() {
    let (vm, module) = vm_support::run_inline("let base = <Int, 2>[2, 3]\nlet result = <Int, _>[1, *base, *(4, 5, named: 99), *(), 6]\n")
        .expect("checked Vector program compiles and runs");
    let result = global(&vm, module, "result");
    assert_eq!(result.class(&vm), vm.universe.classes.vector_class);
    assert_eq!(
        vm.heap.vector(result.as_obj().expect("Vector object")).elements(),
        &[Value::int(1), Value::int(2), Value::int(3), Value::int(4), Value::int(5), Value::int(6),]
    );
}

#[test]
fn context_free_vector_compilation_requires_checked_lowering() {
    let mut vm = VM::new();
    let module = vm.create_module("vector-unchecked", "vector-unchecked");
    let error = vm
        .compile_closure(module, "let value = <Int, 2>[1, 2]\n")
        .expect_err("Vector requires semantic authority");
    assert!(error.to_string().contains("sequence literal lowering"), "{error}");
}

#[test]
fn vector_read_iteration_equality_hash_and_conversion_surfaces_work() {
    let (vm, module) = vm_support::run_inline(
        r#"
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
"#,
    )
    .expect("Vector API program compiles and runs");
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
    let spec = program.modules[&program.entry]
        .lowering
        .sequence_literals
        .values()
        .next()
        .expect("literal spec");
    assert_eq!(spec.expected_extent, Some(3));
    assert!(spec.extent.is_some());
    let mut vm = VM::new();
    vm.materialize_program(&program).expect("materialized program");
    let object = vm.module_registry.get(&program.entry).expect("entry").object;
    assert_eq!(std::mem::size_of::<Value>(), 16);
    // FamilyObject was already the largest inline payload before Vector.
    // Vector must fit below that existing enum layout requirement.
    assert!(std::mem::size_of::<phalcom_core::heap::VectorObject>() <= std::mem::size_of::<phalcom_core::heap::FamilyObject>());
    let closure = vm
        .compile_program_module_closure(&program.entry, "let value = <Int, 3>[1, 2, 3]\n", &program)
        .expect("checked closure");
    let code = &vm.heap.closure(closure).callable.chunk.code;
    assert!(code.iter().any(|op| matches!(op, Bytecode::BuildVector(3))), "{code:?}");
    assert!(
        !code.iter().any(|op| matches!(op, Bytecode::BuildArray(_) | Bytecode::BuildList(_))),
        "{code:?}"
    );
    vm.run_in_module(object, closure).expect("Vector executes");
    let vector = global(&vm, object, "value");
    assert!(vm.heap.as_vector(vector.as_obj().unwrap()).is_some());
}

#[test]
fn vector_is_immutable_and_nested_storage_survives_gc() {
    let (mut vm, module) = vm_support::run_inline(
        r#"
let values = <String, 2>["retained-first", "retained-second"]
let nested = <Vector<String, 2>, 1>[values]
let iterator = <String, 1>["iterator-owned"].iterator
"#,
    )
    .expect("nested Vector program");
    let nested = global(&vm, module, "nested").as_obj().expect("nested Vector");
    let values = vm.heap.vector(nested).get(0).unwrap();
    let string = vm.heap.vector(values.as_obj().unwrap()).get(0).unwrap().as_obj().unwrap();
    for (selector, arguments) in [
        ("[_]=(_)", vec![Value::int(0), Value::int(9)]),
        ("at(_,put:)", vec![Value::int(0), Value::int(9)]),
        ("append(_)", vec![Value::int(9)]),
        ("remove(at:)", vec![Value::int(0)]),
    ] {
        let selector = vm.get_or_intern(selector);
        assert!(vm.send_dynamic(values, selector, &arguments).is_err());
    }
    vm.force_gc();
    assert!(vm.heap.as_vector(nested).is_some());
    assert!(matches!(vm.heap.get(string), phalcom_core::heap::Object::Str(_)));
    let iterator = global(&vm, module, "iterator");
    let next = vm.get_or_intern("next");
    assert!(!vm.send_dynamic(iterator, next, &[]).expect("iterator retains its Vector owner").is_none());
    assert!(vm.send_dynamic(iterator, next, &[]).expect("exhausted iterator").is_none());
}

#[test]
fn vector_structural_keys_preserve_map_and_set_laws() {
    let (vm, module) = vm_support::run_inline(
        r#"
let first = <Int, 2>[1, 2]
let equal = <Int, 2>[1, 2]
let different = <Int, 2>[2, 1]
let map = Map<Vector<Int, 2>, Int>.new()
map[first] = 10
map[equal] = 20
map[different] = 30
let mapSize = map.size
let set = Set<Vector<Int, 2>>.new()
set.add(first)
set.add(equal)
set.add(different)
let setSize = set.size
let crossArray = first == <Int>[1, 2]
let crossList = first == [1, 2]
"#,
    )
    .expect("Vector key program");
    assert_eq!(global(&vm, module, "mapSize"), Value::int(2));
    assert_eq!(global(&vm, module, "setSize"), Value::int(2));
    assert_eq!(global(&vm, module, "crossArray"), Value::bool(false));
    assert_eq!(global(&vm, module, "crossList"), Value::bool(false));
}

#[test]
fn vector_symbolic_identity_spread_copies_native_storage() {
    let source = r#"
class Copier<N: Int> {
  @class copy(_ source: Vector<Int, N>) -> Vector<Int, N> { <Int, N>[*source] }
}
let original = <Int, 2>[7, 8]
let copied = Copier<2>.copy(original)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("symbolic identity copy compiles and runs");
    let original = global(&vm, module, "original").as_obj().unwrap();
    let copied = global(&vm, module, "copied").as_obj().unwrap();
    assert_ne!(original, copied);
    assert_eq!(vm.heap.vector(original).elements(), vm.heap.vector(copied).elements());
}

#[test]
fn vector_rejects_inconsistent_exact_spread_and_final_extent() {
    let source = "let result = <Int, 2>[*(1, 2)]\n";
    let (mut vm, program, first_closure) = vm_support::compile_inline(source).expect("checked concrete spread");
    let module = vm.module_registry.get(&program.entry).expect("entry").object;
    let original_code = vm.heap.closure(first_closure).callable.chunk.code.clone();
    for tamper_spread in [true, false] {
        let closure = first_closure;
        let code = &mut std::rc::Rc::make_mut(&mut vm.heap.closure_mut(closure).callable).chunk.code;
        *code = original_code.clone();
        let mut changed = false;
        for op in code {
            match op {
                Bytecode::VectorExpandExactPositionals { expected, .. } if tamper_spread => {
                    *expected = 3;
                    changed = true;
                }
                Bytecode::FinishVectorLiteral(expected) if !tamper_spread => {
                    *expected = 3;
                    changed = true;
                }
                _ => {}
            }
        }
        assert!(changed, "selected actual cardinality check");
        let error = vm.run_in_module(module, closure).expect_err("inconsistent proof must fail closed");
        assert!(format!("{error:?}").contains(if tamper_spread { "cardinality" } else { "extent" }));
        let result = vm.interner.find("result").expect("global symbol");
        assert!(vm.heap.module(module).get(result).is_none(), "failure cannot publish a Vector result");
        vm.force_gc();
    }
}
