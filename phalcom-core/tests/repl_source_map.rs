//! U-REPL §D2 — a compiled chunk resolves spans against *its own* source.
//!
//! One module accumulates one source entry per compiled unit. In the REPL that
//! is one entry per cell, so the module's "current" source is whatever ran
//! last. Binding diagnostics to the module rather than to the artifact made an
//! earlier cell's span render against a later cell's text, and `.unwrap()`ed a
//! missing source into a panic (`vm/dispatch.rs`, precondition 6).
//!
//! These tests pin the fix: [`Chunk::source_id`] indexes
//! [`ModuleObject::sources`], and an id the module never recorded degrades to
//! `None` instead of panicking.

use phalcom_core::vm::VM;

/// Two units compiled into one module must keep distinct source entries, each
/// resolving to the text that unit was compiled from.
///
/// This is the multi-cell REPL case: without it, `cell_one`'s spans would be
/// rendered against `cell_two`'s text.
#[test]
fn each_compiled_unit_keeps_its_own_source() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<repl>");

    // Deliberately different lengths: an offset valid in one cell would land
    // mid-token (or out of bounds) in the other.
    let cell_one = "let a = 1\n";
    let cell_two = "let considerably_longer_name = 2 + 3\n";

    let first = vm.compile_closure(module, cell_one).expect("first cell should compile");
    let second = vm.compile_closure(module, cell_two).expect("second cell should compile");

    let first_id = vm.heap.closure(first).callable.chunk.source_id;
    let second_id = vm.heap.closure(second).callable.chunk.source_id;

    assert_ne!(first_id, second_id, "each compiled unit must get its own source entry");

    let module_obj = vm.heap.module(module);
    assert_eq!(
        module_obj.source_at(first_id).map(|s| s.as_str()),
        Some(cell_one),
        "the first cell's chunk must resolve to the first cell's text"
    );
    assert_eq!(
        module_obj.source_at(second_id).map(|s| s.as_str()),
        Some(cell_two),
        "the second cell's chunk must resolve to its own text, not the module's latest"
    );
}

/// A source id the module never recorded resolves to `None`.
///
/// This is the path that used to be `.unwrap()`ed into a panic. A synthesized
/// chunk defaults to id `0`; on a module with no recorded source there is
/// nothing to resolve, and the traceback must degrade rather than crash.
#[test]
fn unrecorded_source_id_degrades_to_none() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<empty>");

    assert!(
        vm.heap.module(module).source_at(0).is_none(),
        "a module that compiled nothing has no source entry 0"
    );

    vm.compile_closure(module, "let a = 1\n").expect("cell should compile");

    assert!(vm.heap.module(module).source_at(0).is_some(), "compiling records entry 0");
    assert!(
        vm.heap.module(module).source_at(99).is_none(),
        "an out-of-range source id must resolve to None, never panic"
    );
}
