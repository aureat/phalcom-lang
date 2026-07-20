//! Rust-level proof of U-ANNOT-CONTRACTS plan §3.6's metadata-stripping axis
//! (`MethodObject::contracts`, plan §3.5, D-contract-1).
//!
//! The two `.ph` goldens in `tests/lang/errors/`
//! (`contracts_release_stripped.ph`, `contracts_unchecked_metadata_stripped.ph`)
//! prove the *guard*-stripping axis end to end (a call that would violate a
//! contract in `Debug` mode runs without raising once the guard is
//! stripped). The metadata axis has no `.ph`-visible surface yet — no
//! `Method>>contracts` reflection primitive exists on the core `Method`
//! class (out of this unit's write-set; the plan's own §3.5 scopes only the
//! Rust-side field) — so it is verified here directly against
//! `MethodObject::contracts` through the crate's public API instead.

use phalcom_core::compiler::attributes::CompileMode;
use phalcom_core::heap::lookup_method_in_hierarchy;
use phalcom_core::method::{encode_selector, SignatureKind};
use phalcom_core::vm::VM;

/// Compiles `source` under `mode`/`strip_contract_metadata`, looks up
/// `Box>>#foo(_)`'s compiled [`phalcom_core::method::MethodObject`], and
/// returns whether its `contracts` field is populated.
fn compiled_foo_has_contracts(mode: CompileMode, strip_contract_metadata: bool) -> bool {
    let source = r#"
        class Box {
          construct new() { }

          @requires(x > 0)
          foo(x) {
            return x
          }
        }
    "#;

    let mut vm = VM::new();
    vm.compile_mode = mode;
    vm.strip_contract_metadata = strip_contract_metadata;

    let module = vm.create_module("main", "<contracts_metadata_test>");
    let closure = vm.compile_closure(module, source).expect("Box compiles");
    // Run the module so `Box` is defined as a global before lookup — the
    // class only exists on `vm.classes` after `Bytecode::DefineGlobal` runs.
    vm.run_in_module(module, closure).expect("Box's top level runs without error");

    let class_sym = vm.interner.find("Box").expect("Box was interned while compiling");
    let class_id = *vm.classes.get(&phalcom_core::vm::ClassKey { module, name: class_sym }).expect("Box is a defined global class");

    let selector = encode_selector("foo", &[None], SignatureKind::Method(1));
    let selector_sym = vm.interner.find(&selector).expect("foo's selector was interned while compiling");
    let method_ref = lookup_method_in_hierarchy(&vm.heap, class_id, selector_sym).expect("Box declares foo(_)");

    vm.heap.method(method_ref).contracts.is_some()
}

#[test]
fn debug_mode_retains_contract_metadata() {
    assert!(compiled_foo_has_contracts(CompileMode::Debug, false));
}

#[test]
fn release_mode_retains_contract_metadata_by_default() {
    assert!(compiled_foo_has_contracts(CompileMode::Release, false));
}

#[test]
fn release_mode_strips_contract_metadata_when_flagged() {
    assert!(!compiled_foo_has_contracts(CompileMode::Release, true));
}

#[test]
fn unchecked_mode_strips_contract_metadata_by_default() {
    assert!(!compiled_foo_has_contracts(CompileMode::Unchecked, false));
}
