//! Tier 0 criterion micro-benches (U-BENCH, `performance.md` §4 Tier 0).
//!
//! Three per-mechanism benchmarks that give regression tripwires for later
//! tiers and separate dispatch cost from allocation cost per the §2 cost
//! model:
//!
//! - `bare_send` — a static, argument-free user-method send in a loop; no
//!   primitive on the path, so no per-send `Vec<Value>` allocation
//!   (`vm.rs:626`). Isolates the fixed dispatch tax
//!   (`lookup_method_in_hierarchy`, `class.rs:65`).
//! - `arith_send` — the same send count as `bare_send`, but each send is a
//!   primitive arithmetic op (`1 + 2`). Isolates per-send allocation cost
//!   on top of dispatch.
//! - `fiber_spawn` — fiber spawn + yield + `call()` in a loop. Layers
//!   fiber-object allocation and the fiber switch (already established as
//!   O(1), `fiber.rs:29-51`) on top of `arith_send`'s cost.
//!
//! Each iteration runs one full program on a **freshly bootstrapped**
//! [`Interpreter`] (`Interpreter::new` → `VM::new`, which itself
//! re-lexes/parses/compiles `core.ph` — Tier 5's target). Bootstrap is not
//! isolated out: each program's internal loop count is large enough that
//! bootstrap cost is a small, amortized fraction of the measured time, and
//! this matches how `benchmarks/vm/run.sh` invokes the same programs as
//! whole processes — the two numbers stay comparable.
//!
//! This is a micro-bench harness ([DEC-BENCH-A], `docs/forge/units/U-BENCH/plan.md`)
//! separate from `benchmarks/vm/run.sh`'s whole-process wall-clock timing of
//! Skynet: criterion's warmup/iteration model is appropriate for
//! sub-second per-mechanism numbers, but would mask the whole-process cost
//! that `performance.md` §2 says is the actual unit of measurement for
//! Skynet itself.

use criterion::{Criterion, criterion_group, criterion_main};
use phalcom_core::heap::Object;
use phalcom_core::interpret::Interpreter;
use phalcom_core::value::Value;
use std::hint::black_box;

/// Dispatch-bound source: `benchmarks/vm/bare_send.ph`, a static
/// argument-free send in a 200,000-iteration loop.
const BARE_SEND: &str = include_str!("../../benchmarks/vm/bare_send.ph");

/// Allocation-bound source: `benchmarks/vm/arith_send.ph`, a primitive
/// `1 + 2` send in a 200,000-iteration loop (same send count as
/// [`BARE_SEND`]).
const ARITH_SEND: &str = include_str!("../../benchmarks/vm/arith_send.ph");

/// Fiber-bound source: `benchmarks/vm/fiber_spawn.ph`, a fiber
/// spawn/yield/call cycle in a 20,000-iteration loop.
const FIBER_SPAWN: &str = include_str!("../../benchmarks/vm/fiber_spawn.ph");

/// Variadic-dispatch source: `benchmarks/vm/variadic_send.ph`, 2,000,000 sends
/// to a `sum(*args)` variadic — the only shape that reaches `Invoke`'s variadic
/// probe (IC -> exact probe -> variadic probe). Every other program here
/// dispatches before that line, which is why the probe was unmeasurable until
/// this program existed (perf-log 004).
const VARIADIC_SEND: &str = include_str!("../../benchmarks/vm/variadic_send.ph");

/// Runs `src` to completion on a freshly bootstrapped [`Interpreter`], then
/// asserts each `(global, expected)` pair holds in the `main` module.
///
/// The `checks` are what make a timing a *measurement* rather than a
/// stopwatch reading: an optimization that skips the loop, mis-dispatches to
/// the wrong method, or drops a fiber's result is fast and wrong, and a
/// harness that only asserts "it did not error" would book that as a win
/// (law P1, `docs/spec/v0.2/performance.md`). Every program therefore carries
/// a loop counter and a value checksum, checked here on each iteration —
/// they run inside the timed region, but each is a single hash lookup against
/// a program that just did 10^5–10^6 sends.
///
/// # Panics
///
/// Panics if `src` fails to compile, raises an uncaught runtime error, or
/// finishes with any checked global absent or holding the wrong number — a
/// benchmark program that does not compute its known answer is not a valid
/// baseline (U-BENCH plan §Tests: "the programs must execute — this is the
/// gate").
fn run_program(src: &str, checks: &[(&str, f64)]) {
    let mut interp = Interpreter::new();
    let main = interp.vm.create_module("main", "<bench>");
    interp
        .vm
        .interpret_source(main, src)
        .expect("benchmark program must execute cleanly");

    for &(name, expected) in checks {
        let sym = interp.vm.interner.intern(name);
        let module = match interp.vm.heap.get(main) {
            Object::Module(m) => m,
            _ => panic!("main module handle is not a Module"),
        };
        match module.get(sym) {
            Some(Value::Number(got)) => assert_eq!(
                got, expected,
                "benchmark program computed the wrong answer: `{name}` = {got}, expected {expected}"
            ),
            other => panic!("benchmark global `{name}` missing or not a Number: {other:?}"),
        }
    }
}

/// Benchmarks the dispatch-bound program ([`BARE_SEND`]).
///
/// Checks: the loop ran all 200,000 times, and `acc` holds the value the
/// dispatched `Empty.noop` returns — i.e. every send resolved to the intended
/// method body.
fn bench_bare_send(c: &mut Criterion) {
    c.bench_function("bare_send", |b| {
        b.iter(|| run_program(black_box(BARE_SEND), &[("i", 200_000.0), ("acc", 0.0)]))
    });
}

/// Benchmarks the allocation-bound program ([`ARITH_SEND`]).
///
/// Checks: the loop ran all 200,000 times, and `acc` holds `1 + 2` — i.e. the
/// `Number` `+` primitive received its argument and returned the sum.
fn bench_arith_send(c: &mut Criterion) {
    c.bench_function("arith_send", |b| {
        b.iter(|| run_program(black_box(ARITH_SEND), &[("i", 200_000.0), ("acc", 3.0)]))
    });
}

/// Benchmarks the fiber spawn/yield program ([`FIBER_SPAWN`]).
///
/// Checks: the loop ran all 20,000 times, and `acc` holds the value the last
/// fiber yielded back through `call()` — i.e. the fibers actually ran and
/// delivered their result to the resumer, rather than being spawned and
/// abandoned.
fn bench_fiber_spawn(c: &mut Criterion) {
    c.bench_function("fiber_spawn", |b| {
        b.iter(|| run_program(black_box(FIBER_SPAWN), &[("i", 20_000.0), ("acc", 0.0)]))
    });
}

/// Benchmarks the variadic-dispatch program ([`VARIADIC_SEND`]).
///
/// Checks: the loop ran all 2,000,000 times, and `acc` is `2_000_000 * 3` —
/// i.e. every call collapsed its three trailing arguments into the rest `List`
/// and the variadic probe resolved, rather than falling through to `dNU`.
fn bench_variadic_send(c: &mut Criterion) {
    c.bench_function("variadic_send", |b| {
        b.iter(|| run_program(black_box(VARIADIC_SEND), &[("i", 2_000_000.0), ("acc", 6_000_000.0)]))
    });
}

criterion_group!(vm_benches, bench_bare_send, bench_arith_send, bench_fiber_spawn, bench_variadic_send);
criterion_main!(vm_benches);
