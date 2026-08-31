//! Exact associated reference reification tests (Part 4).

use phalcom_common::selector::SelectorSlot;
use phalcom_core::bytecode::Bytecode;
use phalcom_core::error::PhError;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::sync::Arc;

fn run_inline(source: &str) -> Result<(VM, phalcom_core::heap::ObjRef), PhError> {
    let mut vm = VM::new();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).map_err(PhError::from)?;
    vm.run_compiled(&program)?;
    let entry_id = program.initialization_order.last().expect("entry module");
    let module = vm.module_registry.get(entry_id).expect("entry module registered").object;
    Ok((vm, module))
}

#[test]
fn exact_variant_constructor_reference_reifies_callable_thunk() {
    let source = r#"
enum Option<T> {
  @variant Some(_ value: T)
  @variant None
}

let ctor = Option<Int>::Some::(_)
let some = ctor(42)
"#;
    let (vm, module) = run_inline(source).expect("constructor reference should execute");
    let some = vm
        .heap
        .module(module)
        .get(vm.interner.find("some").expect("some symbol"))
        .expect("some binding");
    let variant = vm.runtime_variant_of(some).expect("variant identity");
    assert_eq!(vm.case_payload_at(some, 0).expect("payload"), Value::int(42));
    assert_eq!(vm.adt_registry.variant_descriptor(variant).expect("descriptor").payload_arity, 1);
}

#[test]
fn constructor_reference_lowering_is_distinct_from_direct_construction() {
    let source = r#"
enum Option<T> {
  @variant Some(_ value: T)
}

let ctor = Option<Int>::Some::(_)
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("constructor reference should compile");
    let mut vm = VM::new();
    vm.materialize_program(&program).expect("program materializes");
    let closure = vm
        .compile_program_module_closure(&program.entry, source, &program)
        .expect("module closure compiles");
    let chunk = &vm.heap.closure(closure).callable.chunk;
    let spec = program.modules[&program.entry]
        .lowering
        .associated
        .values()
        .next()
        .expect("constructor reference lowering");
    let phalcom_core::modules::semantic_lowering::AssociatedLoweringSpec::MakeVariantConstructorThunk { operation, .. } = spec else {
        panic!("expected constructor thunk lowering, got {spec:?}");
    };
    assert_eq!(operation.slots.as_ref(), [SelectorSlot::Positional]);
    assert!(chunk.code.iter().any(|op| matches!(op, Bytecode::Closure(_))));
    assert!(!chunk.code.iter().any(|op| matches!(op, Bytecode::ConstructVariant { .. })));
}
