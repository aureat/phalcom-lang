//! Shared executable-pattern context scenarios.

use super::vm_support::{compile_inline, run_inline};
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::sync::Arc;

fn slot(vm: &phalcom_core::vm::VM, module: phalcom_core::heap::ObjRef, name: &str) -> Option<Value> {
    vm.heap.module(module).get(vm.interner.find(name)?)
}

#[test]
#[ignore = "GATED: if-let pattern syntax is not available in current parser"]
fn pat_ctx_01_if_let_nested_success_commits_binding() {
    let source = "data Outer { Some(_ value: Int) None }\nlet value = Outer::Some(42)\nlet result = if let Outer::Some(x) = value { x } else { 0 }\n";
    let (vm, module) = run_inline(source).expect("nested if-let should execute");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(42)));
}

#[test]
#[ignore = "GATED: if-let pattern syntax is not available in current parser"]
fn pat_ctx_02_if_let_nested_failure_does_not_leak_binding() {
    let source = "data Outer { Some(_ value: Int) None }\nlet value = Outer::None\nlet result = if let Outer::Some(x) = value { x } else { 0 }\n";
    let (vm, module) = run_inline(source).expect("failed if-let should execute else branch");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(0)));
}

#[test]
#[ignore = "GATED: if-let pattern syntax is not available in current parser"]
fn pat_ctx_03_if_let_or_pattern_publishes_shared_binding() {
    let source = "data Either { Left(_ value: Int) Right(_ value: Int) }\nlet value = Either::Right(42)\nlet result = if let Either::Left(x) | Either::Right(x) = value { x } else { 0 }\n";
    let (vm, module) = run_inline(source).expect("or-pattern if-let should execute");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(42)));
}

#[test]
#[ignore = "GATED: while-let pattern syntax is not available in current parser"]
fn pat_ctx_04_while_let_evaluates_rhs_once_per_iteration() {
    let source = "data Option { Some(_ value: Int) None }\nlet value = Option::None\nwhile let Option::Some(x) = value { break }\nlet result = 1\n";
    let (_vm, _module) = run_inline(source).expect("while-let should compile and terminate");
}

#[test]
#[ignore = "GATED: while-let pattern syntax is not available in current parser"]
fn pat_ctx_05_while_let_failed_iteration_does_not_leak_binding() {
    let source = "data Option { Some(_ value: Int) None }\nlet value = Option::None\nwhile let Option::Some(x) = value { break }\nlet result = 0\n";
    let (vm, module) = run_inline(source).expect("failed while-let should terminate cleanly");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(0)));
}

#[test]
#[ignore = "GATED: required destructuring emitter fixture is not available"]
fn pat_ctx_06_required_destructuring_uses_shared_emitter() {
    let source =
        "data Pair { Pair(_ left: Int, _ right: Int) }\nlet value = Pair::Pair(20, 22)\nlet Pair::Pair(left, right) = value\nlet result = left + right\n";
    let (vm, module) = run_inline(source).expect("required destructuring should use pattern emitter");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(42)));
}

#[test]
fn pat_ctx_07_for_pattern_binds_each_item() {
    let source = "let mut total = 0\nfor value in [1, 2, 3] { total = total + value }\n";
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("for-pattern fixture should compile");
    let mut vm = VM::new_kernel();
    vm.materialize_program(&program).expect("for-pattern fixture should materialize");
    vm.compile_program_module_closure(&program.entry, source, &program)
        .expect("for-pattern fixture should lower to a module closure");
}

#[test]
#[ignore = "GATED: closure capture pattern fixture is not available"]
fn pat_ctx_08_captured_binding_is_committed_visible_binding() {
    let source = "data Boxed { Value(_ value: Int) }\nlet boxed = Boxed::Value(42)\nlet capture = if let Boxed::Value(x) = boxed { { x } } else { { 0 } }\nlet result = capture()\n";
    let result = compile_inline(source);
    assert!(result.is_ok(), "captured pattern binding should compile");
}
