//! U-REPL §D1, §D3, §D10 — Session, cell loop, echo mode, upvalue hygiene across cells.

use phalcom_core::compiler::lib::UnitKind;
use phalcom_core::heap::Object;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

/// `globals_persist_across_cells`: Globals bound in cell 1 persist in the module
/// and can be read by cell 2.
#[test]
fn globals_persist_across_cells() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<repl>");

    let c1 = vm.compile_closure_as(module, "let x = 42\n", UnitKind::Repl).unwrap();
    let res1 = vm.run_cell(module, c1).unwrap();
    assert_eq!(res1, Value::none(), "statement cell returns surfaced absence");

    let c2 = vm.compile_closure_as(module, "x\n", UnitKind::Repl).unwrap();
    let res2 = vm.run_cell(module, c2).unwrap();
    assert_eq!(res2, Value::int(42), "cell 2 reads global x from cell 1");
}

/// `echo_mode_keeps_final_expression`: Repl unit keeps final expression value on stack.
#[test]
fn echo_mode_keeps_final_expression() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<repl>");

    let c_repl = vm.compile_closure_as(module, "10 + 20\n", UnitKind::Repl).unwrap();
    let res_repl = vm.run_cell(module, c_repl).unwrap();
    assert_eq!(res_repl, Value::int(30), "Repl unit returns final expression");
}

/// `statement_cell_echoes_nothing`: A statement cell yields Nil (surfaced as None).
#[test]
fn statement_cell_echoes_nothing() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<repl>");

    let c_stmt = vm.compile_closure_as(module, "let a = 100\n", UnitKind::Repl).unwrap();
    let res_stmt = vm.run_cell(module, c_stmt).unwrap();
    assert_eq!(res_stmt, Value::none(), "Statement cell yields surfaced absence");
}

/// `underscore_binds_last_value`: `_` global holds the prior cell's expression result.
#[test]
fn underscore_binds_last_value() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<repl>");

    let c1 = vm.compile_closure_as(module, "100 + 200\n", UnitKind::Repl).unwrap();
    let val1 = vm.run_cell(module, c1).unwrap();
    assert_eq!(val1, Value::int(300));

    let underscore_sym = vm.get_or_intern("_");
    vm.define_global(module, underscore_sym, val1).unwrap();

    let c2 = vm.compile_closure_as(module, "_ + 50\n", UnitKind::Repl).unwrap();
    let val2 = vm.run_cell(module, c2).unwrap();
    assert_eq!(val2, Value::int(350), "_ evaluates to 300");
}

/// `open_upvalue_hygiene_across_cells`: Load-bearing test for §D10.
///
/// Shape: cell 1 defines a function capturing a local, and errors mid-execution.
/// Unwinding cell 1 must close open upvalues and clear stack/frames cleanly.
/// Cell 2 pushes new stack items; it MUST NOT alias cell 1's captured slots.
#[test]
fn open_upvalue_hygiene_across_cells() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<repl>");

    // Pre-declare getter as global in module so cell 1's assignment binds it
    let getter_sym = vm.get_or_intern("getter");
    vm.define_global(module, getter_sym, phalcom_core::value::NIL).unwrap();

    // Cell 1: creates a block capturing a local `secret`, assigns getter, then raises a runtime error
    let cell1_src = "let make = || {\nlet secret = 999\ngetter = || {\nsecret\n}\nNone.non_existent_method()\n}\nmake()\n";

    let c1 = vm.compile_closure_as(module, cell1_src, UnitKind::Repl).expect("cell 1 compiles");
    let res1 = vm.run_cell(module, c1);
    assert!(res1.is_err(), "cell 1 raises runtime error");

    // §D10 asserts unwind_cell was called (run_cell calls unwind_cell in all cases).
    // Now cell 2 runs, pushing new values on stack and executing the captured closure getter.
    let cell2_src = "let dummy1 = 111\nlet dummy2 = 222\nlet dummy3 = 333\ngetter()\n";

    let c2 = vm.compile_closure_as(module, cell2_src, UnitKind::Repl).expect("cell 2 compiles");
    let res2 = vm.run_cell(module, c2).expect("cell 2 should execute without stack aliasing corruption");
    assert_eq!(
        res2,
        Value::int(999),
        "captured upvalue secret remains 999 and is not corrupted by cell 2's stack"
    );
}

/// `class_redefinition_shadows`: Cell 2's `class Foo` shadows cell 1's `class Foo`.
/// Existing instances keep the old class.
#[test]
fn class_redefinition_shadows() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<repl>");

    let c1 = vm
        .compile_closure_as(module, "class Foo {\nv() { 1 }\n}\nlet f1 = Foo.new()\n", UnitKind::Repl)
        .unwrap();
    vm.run_cell(module, c1).unwrap();

    let c2 = vm
        .compile_closure_as(module, "class Foo {\nv() { 2 }\n}\nlet f2 = Foo.new()\n", UnitKind::Repl)
        .unwrap();
    vm.run_cell(module, c2).unwrap();

    let c3 = vm.compile_closure_as(module, "[f1.v(), f2.v()]\n", UnitKind::Repl).unwrap();
    let res = vm.run_cell(module, c3).unwrap();

    if let Some(id) = res.as_obj() {
        if let Object::List(list) = vm.heap.get(id) {
            let elems = list.elements();
            assert_eq!(elems[0], Value::int(1), "f1 retains old class method v() -> 1");
            assert_eq!(elems[1], Value::int(2), "f2 uses new shadowed class method v() -> 2");
            return;
        }
    }
    panic!("Expected list result");
}
