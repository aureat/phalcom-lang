//! U-REPL §D4 — Two-set immutability across REPL cells.

use phalcom_core::compiler::lib::UnitKind;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

/// `const_rebinds_across_repl_cells`: Cell 1 `const x = 1`, cell 2 `x = 2` — succeeds under `Repl`.
#[test]
fn const_rebinds_across_repl_cells() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<repl>");

    let c1 = vm.compile_closure_as(module, "const x = 1\n", UnitKind::Repl).expect("cell 1 compiles");
    vm.run_cell(module, c1).expect("cell 1 succeeds");

    let c2 = vm
        .compile_closure_as(module, "x = 2\n", UnitKind::Repl)
        .expect("cell 2 re-assignment compiles in Repl mode");
    let res2 = vm.run_cell(module, c2).expect("cell 2 succeeds");
    assert_eq!(res2, Value::Number(2.0));
}

/// `const_redeclares_across_repl_cells`: Cell 1 `const x = 1`, cell 2 `const x = 2` — succeeds.
#[test]
fn const_redeclares_across_repl_cells() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<repl>");

    let c1 = vm.compile_closure_as(module, "const x = 1\n", UnitKind::Repl).expect("cell 1 compiles");
    vm.run_cell(module, c1).expect("cell 1 succeeds");

    let c2 = vm
        .compile_closure_as(module, "const x = 2\n", UnitKind::Repl)
        .expect("cell 2 const re-declaration compiles in Repl mode");
    let res2 = vm.run_cell(module, c2).expect("cell 2 succeeds");
    assert_eq!(res2, Value::Obj(vm.universe.classes.none_singleton));

    let c3 = vm.compile_closure_as(module, "x\n", UnitKind::Repl).expect("cell 3 compiles");
    let res3 = vm.run_cell(module, c3).expect("cell 3 succeeds");
    assert_eq!(res3, Value::Number(2.0));
}

/// `const_assignment_errors_in_one_unit`: Both statements in ONE unit — errors.
#[test]
fn const_assignment_errors_in_one_unit() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<repl>");

    let err = vm.compile_closure_as(module, "const x = 1\nx = 2\n", UnitKind::Repl);
    assert!(err.is_err(), "assignment to const in same unit MUST error");
}

/// `const_assignment_errors_in_file_mode`: Same source across units in `UnitKind::File` mode — errors.
///
/// Guards against over-broad exemption leaking into File units.
#[test]
fn const_assignment_errors_in_file_mode() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<file>");

    let c1 = vm.compile_closure_as(module, "const x = 1\n", UnitKind::File).expect("unit 1 compiles");
    vm.run_cell(module, c1).expect("unit 1 succeeds");

    let err = vm.compile_closure_as(module, "x = 2\n", UnitKind::File);
    assert!(err.is_err(), "cross-unit assignment to const in File mode MUST error");
}

/// `class_shadows_across_repl_cells`: Cell 2's `class Foo` does not trip `already_defined` in `global_bindings`.
#[test]
fn class_shadows_across_repl_cells() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<repl>");

    let c1 = vm.compile_closure_as(module, "class Foo {}\n", UnitKind::Repl).expect("cell 1 compiles");
    vm.run_cell(module, c1).expect("cell 1 succeeds");

    let c2 = vm
        .compile_closure_as(module, "class Foo {}\n", UnitKind::Repl)
        .expect("cell 2 compiles without class definition error");
    vm.run_cell(module, c2).expect("cell 2 succeeds");
}
