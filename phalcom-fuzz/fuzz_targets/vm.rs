#![no_main]

use libfuzzer_sys::fuzz_target;
use phalcom_core::vm::VM;

// Compile and execute arbitrary source, then collect garbage and verify the
// object-model invariants. Compile and runtime errors are valid outcomes for
// malformed programs; panics, sanitizer failures, timeouts, and invariant
// failures are fuzz findings.
fuzz_target!(|data: &str| {
    let mut vm = VM::new();
    let module = vm.create_module("fuzz", "fuzz.ph");

    if let Ok(closure) = vm.compile_closure(module, data) {
        let _ = vm.run_in_module(module, closure);
    }

    vm.force_gc();
    assert!(
        vm.universe.verify_invariants(&vm.heap).is_ok(),
        "VM invariants failed after fuzz input"
    );
});
