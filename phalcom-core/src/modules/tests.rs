use super::{EntrySelection, ModuleState, ProgramCompiler};
use crate::vm::VM;
use std::sync::Arc;

#[test]
fn second_materialize_of_same_program_preserves_module_object() {
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(
        "let value = 1\nexport value\n",
    )))
    .expect("inline program should compile");
    let mut vm = VM::new();

    vm.materialize_program(&program)
        .expect("first materialization should succeed");
    let first = vm
        .module_registry
        .get(&program.entry)
        .expect("entry should be registered")
        .object;

    vm.materialize_program(&program)
        .expect("second materialization of same program should be a no-op");
    let second = vm
        .module_registry
        .get(&program.entry)
        .expect("entry should remain registered")
        .object;

    assert_eq!(first, second, "idempotent materialization must preserve object identity");
}

#[test]
fn second_run_does_not_recompile_initialized_module() {
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(
        "let value = 1\nexport value\n",
    )))
    .expect("inline program should compile");
    let mut vm = VM::new();

    vm.run_compiled(&program).expect("first run should succeed");
    let record = vm.module_registry.get(&program.entry).expect("entry registered");
    assert_eq!(record.state, ModuleState::Initialized);
    let object = record.object;
    let closure = vm.heap.module(object).closure.expect("source closure compiled");
    let source_count = vm.heap.module(object).sources.len();

    vm.run_compiled(&program).expect("second run should be a semantic no-op");
    let record = vm.module_registry.get(&program.entry).expect("entry still registered");
    assert_eq!(record.state, ModuleState::Initialized);
    assert_eq!(record.object, object);
    assert_eq!(vm.heap.module(object).closure, Some(closure));
    assert_eq!(
        vm.heap.module(object).sources.len(),
        source_count,
        "second run must not push/recompile source"
    );
}

#[test]
fn second_program_with_same_semantic_ids_cannot_mutate_first_program() {
    let first_program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(
        "let value = 1\nexport value\n",
    )))
    .expect("inline program should compile");
    let second_program = ProgramCompiler::new(first_program.linked.clone())
        .compile_entry(EntrySelection::ModuleId(first_program.entry.clone()))
        .expect("same linked graph should compile as a distinct runtime program");
    let mut vm = VM::new();

    vm.materialize_program(&first_program)
        .expect("first program should materialize");
    let object = vm
        .module_registry
        .get(&first_program.entry)
        .expect("entry registered")
        .object;

    let err = vm
        .materialize_program(&second_program)
        .expect_err("different runtime program must not reuse the same semantic registry entries");
    assert!(err.to_string().contains("another runtime program"), "unexpected error: {err}");
    assert_eq!(
        vm.module_registry
            .get(&first_program.entry)
            .expect("first entry remains registered")
            .object,
        object,
        "ownership rejection must occur before mutating the existing record"
    );
}
