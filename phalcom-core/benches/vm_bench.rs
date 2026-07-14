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
use phalcom_core::interpret::Interpreter;
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

/// Runs `src` to completion on a freshly bootstrapped [`Interpreter`].
///
/// # Panics
///
/// Panics if `src` fails to compile or raises an uncaught runtime error —
/// a benchmark program that errors is not a valid baseline (U-BENCH plan
/// §Tests: "the programs must execute — this is the gate").
fn run_program(src: &str) {
    let mut interp = Interpreter::new();
    let main = interp.vm.create_module("main", "<bench>");
    interp
        .vm
        .interpret_source(main, src)
        .expect("benchmark program must execute cleanly");
}

/// Benchmarks the dispatch-bound program ([`BARE_SEND`]).
fn bench_bare_send(c: &mut Criterion) {
    c.bench_function("bare_send", |b| b.iter(|| run_program(black_box(BARE_SEND))));
}

/// Benchmarks the allocation-bound program ([`ARITH_SEND`]).
fn bench_arith_send(c: &mut Criterion) {
    c.bench_function("arith_send", |b| b.iter(|| run_program(black_box(ARITH_SEND))));
}

/// Benchmarks the fiber spawn/yield program ([`FIBER_SPAWN`]).
fn bench_fiber_spawn(c: &mut Criterion) {
    c.bench_function("fiber_spawn", |b| b.iter(|| run_program(black_box(FIBER_SPAWN))));
}

criterion_group!(vm_benches, bench_bare_send, bench_arith_send, bench_fiber_spawn);
criterion_main!(vm_benches);
