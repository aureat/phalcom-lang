//! PDR-0007 — bounded call depth and native re-entrancy.
//!
//! Before these limits existed, unbounded `.ph` recursion did not fail at all:
//! frames live in a heap `Vec`, so it grew until the OS killed the process —
//! measured at over five minutes with no diagnostic, which is strictly worse than
//! a stack overflow because there is no defined outcome to catch.

use phalcom_core::compiler::lib::UnitKind;
use phalcom_core::error::{PhError, RuntimeError};
use phalcom_core::value::Value;
use phalcom_core::vm::{MAX_CALL_DEPTH, MAX_NATIVE_REENTRY, VM};

/// Runs `source` as a module and returns the outcome.
fn run(source: &str) -> Result<(), PhError> {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<test>");
    let closure = vm.compile_closure_as(module, source, UnitKind::File)?;
    vm.run_in_module(module, closure)
}

/// Asserts the error is a `DepthExceeded` naming `what`, with the ceiling `limit`.
fn assert_depth_exceeded(err: PhError, what: &str, limit: usize) {
    match err {
        PhError::Runtime(RuntimeError::DepthExceeded {
            what: got_what,
            limit: got_limit,
        }) => {
            assert_eq!(got_what, what, "wrong counter tripped");
            assert_eq!(got_limit, limit, "message must name the actual ceiling");
        }
        other => panic!("expected DepthExceeded, got {other:?}"),
    }
}

#[test]
fn ph_call_depth_is_bounded() {
    // Unbounded `.ph` recursion: the case that used to hang.
    let err = run("class Boom {\n@constructor\nnew() {}\n  go(_ n) { return self.go(n + 1) }\n}\nBoom.new().go(0)\n")
        .expect_err("infinite recursion must fail, not hang");
    assert_depth_exceeded(err, "call depth", MAX_CALL_DEPTH);
}

#[test]
fn native_reentry_is_bounded() {
    // A separate counter, and it must be the one that trips here. `perform` routes
    // through `send_dynamic`, which drives the dispatch loop recursively on the
    // *Rust* stack — overflowing that aborts the process rather than unwinding, so
    // this ceiling is two orders of magnitude tighter and must fire first.
    let err = run("class P {\n@constructor\nnew() {}\n  go { return self.perform(#go) }\n}\nP.new().go\n").expect_err("unbounded native re-entrancy must fail");
    assert_depth_exceeded(err, "native re-entrancy depth", MAX_NATIVE_REENTRY);
}

#[test]
fn dnu_chain_trips_the_ph_counter_not_the_native_one() {
    // A `doesNotUnderstand` chain looks like it should be native re-entrancy, but
    // it pushes ordinary `.ph` frames and trips the call-depth ceiling instead.
    // Pinning this stops someone "fixing" the native counter to catch a case that
    // was never on its path.
    let err = run("class Deep {\n@constructor\nnew() {}\n  doesNotUnderstand(_ msg) { return self.alsoMissing() }\n}\nDeep.new().missing()\n")
        .expect_err("a runaway dNU chain must fail");
    assert_depth_exceeded(err, "call depth", MAX_CALL_DEPTH);
}

#[test]
fn depth_error_is_an_ordinary_catchable_raise() {
    // PDR-0007 §2 — it must be catchable, so ADR-0008's terminating unwind applies
    // and `ensure` still runs.
    //
    // This nearly did not hold. `Block#on` ran its `isA` probe — a full dynamic
    // send needing a frame of its own — *before* unwinding the failed block's
    // frames, so at the call-depth ceiling the handler could never get a frame and
    // the depth error escaped uncatchable. The unwind now happens first.
    let mut vm = VM::new();
    let module = vm.create_module("main", "<test>");
    let source = "class Boom {\n@constructor\nnew() {}\n  go(_ n) { return self.go(n + 1) }\n}\n\
                  let caught = false\n\
                  try {\n  Boom.new().go(0)\n} catch e {\n  caught = true\n}\n";
    let closure = vm.compile_closure_as(module, source, UnitKind::File).expect("compiles");
    vm.run_in_module(module, closure).expect("the depth error must be caught, not propagate");

    let caught_sym = vm.get_or_intern("caught");
    let module_obj = vm.heap.module(module);
    let slot = module_obj.slot_of(caught_sym).expect("`caught` must be bound");
    assert_eq!(module_obj.globals[slot], Value::Bool(true), "the handler must have run");
}

#[test]
fn traceback_survives_a_frame_that_executed_nothing() {
    // Regression guard. `runtime_error` reads `chunk.span_at(frame.ip - 1)`
    // (`chunk.spans[frame.ip - 1]` before U-TRACE T1 centralized the clamp),
    // which underflows for a frame whose `ip` is still 0. That was unreachable
    // while tracebacks were built after the frames had been discarded; reporting
    // before the unwind (PDR-0008 §2) makes it reachable, and a depth-limit raise
    // hits it immediately — it panicked with "attempt to subtract with overflow"
    // before `saturating_sub`. A panic reachable from ordinary source is a
    // robustness bug.
    let err = run("class P {\n@constructor\nnew() {}\n  go { return self.perform(#go) }\n}\nP.new().go\n").expect_err("must raise");
    // Reaching this line at all means the traceback walk did not panic.
    assert_depth_exceeded(err, "native re-entrancy depth", MAX_NATIVE_REENTRY);
}
