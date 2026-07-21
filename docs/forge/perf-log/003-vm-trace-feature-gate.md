# 003 — U-TRACE: compile out the dispatch loop's per-opcode tracing (Tier 1)

Status: **landed** · Unit: [U-TRACE](../units/U-TRACE/plan.md) (Tier 1) · Spec: [performance.md §4 Tier 1](../../spec/current/performance.md), [ADR-0051](../../adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md) · Behavior-invariant (no ADR, no floor change)

Closes the "Tier 1 — tracing span (~18.3% arith)" candidate that
[`README.md`](README.md) had ranked next but never turned into a unit.

## The cost

`vm/dispatch.rs` carried four `tracing` callsites inside the opcode loop: a
`vm_opcode` span + `.enter()` guard, and three `debug!`s dumping stack state. The
span at `dispatch.rs:403` was the only `span!` in the workspace.

`bin/phalcom/main.rs:15` already pins every subscriber to `LevelFilter::OFF`, so
**no trace output was ever emitted** — and the loop paid for it anyway. This is the
point worth carrying forward: a runtime filter suppresses *output*, not the
*instructions* the compiler must emit at each callsite. No subscriber setting can
remove code.

## Method

Whole-process A/B (performance.md §2 — "whole-process lifetime is the unit of
measurement"), on `arith_send` scaled to 5M iterations so execution dominates
process startup. Variant binaries are built, copied aside, and then run
**interleaved** (A,B,A,B,…), 12–14 reps each, in one process-level loop.

`min` is the headline estimator, with `median` reported alongside as a
consistency check: wall-clock noise is one-sided (background load only ever adds
time), so `min` is the cleanest estimate of the binary's own cost. Every Δ below
agreed between `min` and `median` to within 0.2pp — where they disagree, the
result is noise and is reported as such (see Skynet, below).

## Attribution — measure before explaining

Each row is a separately-built binary, A/B'd against HEAD `876dce2`:

| Variant | Δ min vs HEAD |
|---|---|
| `LevelFilter::OFF` moved from a per-layer filter to the `Layered` stack | **−0.4% (null)** |
| Span removed, `debug!`s kept | −8.4% |
| `debug!`s removed, span kept | −8.4% |
| Both removed | **−18.2%** |

**A refuted theory, recorded because it was the attractive one.** The first
hypothesis was a subscriber misconfiguration: `registry().with(layer.with_filter(OFF))`
combines a `never` layer with the registry's default `always`, which *should* yield
`Interest::sometimes()` and force a dynamic `enabled()` check per opcode. It is a
tidy story and it is wrong — the interest already resolves to `never` in both
configs, and fixing the placement bought **−0.4%**. `main.rs` was reverted and left
alone. (Per README's own rule: reproduce the baseline observation before explaining
it.)

**The cost splits evenly and additively between the span and the `debug!`s.** So it
is generic per-callsite overhead, not a span-guard optimization barrier — the
mechanism the "tracing span" framing implied. This directly shaped the fix: **gating
only the span would have recovered half the win.**

## The cut

A `vm-trace` Cargo feature (`phalcom-core/Cargo.toml`), off by default, gating all
four callsites plus the now-conditional `use tracing::{debug, span, Level}` import.

```rust
#[cfg(feature = "vm-trace")]
let _opcode_span = {
    let entered = span!(Level::DEBUG, "vm_opcode", opcode = ?opcode).entered();
    debug!("Stack before: {:?}", self.stack);
    entered
};
```

`tracing` stays a **hard dependency** — the compiler and `vm/api.rs` instrument cold
paths, where this cost does not arise. `tracing/release_max_level_off` would have
been a one-line alternative but was rejected: Cargo feature unification would force
it on every downstream consumer of `phalcom-core` as a library.

## Result

| Metric | Before | After | Δ |
|---|---|---|---|
| `arith_send` 5M, whole-process, min of 14 | 1.124 s | 0.936 s | **−16.7%** |
| same, median of 14 | 1.181 s | 0.986 s | −16.5% |

Golden diff: **none** — all three `benchmarks/vm/*.ph` byte-identical before/after.
`cargo test --workspace`: 239 passed, 0 failed. Both feature states build clean.

**Skynet: no win measurable — and `real` is the wrong instrument for it.** A 3-pair
A/B:

| | run 1 | run 2 | run 3 | min |
|---|---|---|---|---|
| HEAD `real` | 16.89 | 19.40 | 13.98 | 13.98 s |
| gated `real` | 20.30 | 14.84 | 18.78 | 14.84 s |
| HEAD `user` | 9.11 | 9.50 | 9.09 | **9.09 s** |
| gated `user` | 9.68 | 9.00 | 9.38 | **9.00 s** |

`real` spans **5.4 s within HEAD alone** — it cannot resolve an effect of this size,
and the apparent "regression" on min-`real` is noise, not signal. `user` is the
tighter instrument here (the variance lives in `sys`: page faults and
allocation, per F1/F5), and on `user` the cut measures **≈ −1%**: real but
near-nothing.

That is the expected shape, not a disappointment — Skynet is allocation/GC-bound, so
it executes far fewer opcodes per unit of work than `arith_send`, and a per-opcode
tax is proportionally small. **Worth carrying forward as method**, though: for
Skynet, prefer `user` over `real`, and treat any single-digit-percent `real` claim on
it as unfounded. `benchmarks/vm/BASELINE.md`'s own 13.7–15.6 s spread said as much.

## Caveat carried from criterion

`benches/vm_bench.rs` drives `Interpreter` directly and installs **no subscriber**,
so its callsite interest caches as `never` — a *different, cheaper* path than the
real binary. The criterion micro-benches therefore **under-measure this cost and show
~no win**, which is why every number here is whole-process. Worth knowing before
trusting criterion for any future tracing- or subscriber-sensitive question.
