# U-TRACE — compile out the dispatch loop's per-opcode tracing

_Tier 1 of the [performance strategy](../../../spec/current/performance.md)
([ADR-0051](../../../adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md)).
Written 2026-07-14 against HEAD `876dce2`, **after** the measurement — every number below is
reproduced in [perf-log 003](../../perf-log/003-vm-trace-feature-gate.md), not predicted._

## 0. Why this unit existed only as a one-line note

[`perf-log/README.md`](../../perf-log/README.md) ranked "Tier 1 — tracing span (~18.3% arith)"
as the next candidate but no unit directory, plan, or IMPL-SPEC was ever written. This document
closes that gap. The unit is small enough that plan and execution landed in one pass.

## 1. Problem

`vm/dispatch.rs` instruments **every opcode** with four `tracing` callsites:

| Line (HEAD `876dce2`) | Callsite | Frequency |
|---|---|---|
| 403 | `span!(Level::DEBUG, "vm_opcode", opcode = ?opcode)` + `.enter()` | every opcode |
| 405 | `debug!("Stack before: {:?}", self.stack)` | every opcode |
| 412 | `debug!("Pushing constant: {:?}", constant)` | every `Constant` |
| 982 | `debug!("Stack after opcode {:?}: {:?}", …)` | every opcode |

These are the only `span!` in the whole workspace, so the blast radius is one file.

**The cost is not removable at runtime.** `bin/phalcom/main.rs:15` already filters every
subscriber to `LevelFilter::OFF`, so no trace output is ever emitted — and the loop still pays
the cost, because the compiler must emit the callsite code regardless of what a subscriber later
decides. Filtering suppresses *output*, not *instructions*.

## 2. Measured attribution (the reason the fix has the shape it has)

Whole-process A/B, 5M-iteration `arith_send`, interleaved runs, `min` of 12–14 (see perf-log 003
for method and why `min` is the estimator):

| Variant | Δ vs HEAD |
|---|---|
| Move `LevelFilter::OFF` from a per-layer filter to the `Layered` stack (hint = `OFF`) | **−0.4% — null result** |
| Remove the span only, keep the `debug!`s | −8.4% |
| Remove the `debug!`s only, keep the span | −8.4% |
| Remove both | **−18.2%** |

Two findings shaped the unit:

- **The subscriber-configuration theory is refuted.** The plausible story — that
  `registry().with(layer.with_filter(OFF))` yields `Interest::sometimes()` and forces a dynamic
  `enabled()` check per opcode — was tested first and bought nothing. `main.rs` is left alone.
- **Cost is split evenly and additively between span and `debug!`.** So this is generic
  per-callsite overhead, not a span-guard-specific optimization barrier. **A fix that gates only
  the span recovers half the win** — which is exactly what perf-log README's "tracing span"
  framing would have produced. All four callsites must be gated.

## 3. Fix

A `vm-trace` Cargo feature (`phalcom-core/Cargo.toml`), **off by default**, gating all four
callsites and the now-conditional `use tracing::{debug, span, Level}` import in `dispatch.rs`.

`tracing` stays a hard (non-optional) dependency: the compiler (`compiler/lib/scope.rs`,
`compiler/lib/class_decl.rs`) and `vm/api.rs` instrument **cold** paths, where the same cost does
not arise. This unit's claim is about the dispatch loop only.

Rejected alternatives:

- **Delete the instrumentation.** Recovers the same time but destroys a real debugging tool for a
  loop that is hard to debug any other way.
- **`tracing/release_max_level_off`.** One line, no source churn — but it is a feature on a
  dependency, so Cargo feature unification would force it on **every** downstream consumer of
  `phalcom-core` as a library. Enabling `release_max_level_*` from a library is the documented
  anti-pattern.
- **Fix the subscriber config.** Measured: −0.4%. Does not work (§2).

## 4. Result

| Metric | Before | After | Δ |
|---|---|---|---|
| `arith_send` 5M, whole-process (min of 14) | 1.124 s | 0.936 s | **−16.7%** |

**Behavior-invariant**: all three `benchmarks/vm/*.ph` produce byte-identical stdout before and
after; `cargo test --workspace` = 239 passed / 0 failed. Both feature states build clean
(`cargo build -p phalcom-core` and `--features vm-trace`).

**Not attributable to this unit:** `tests/lang` `indexing` + `indexing_negative` fail at HEAD
`876dce2` with and without this change — pre-existing, U-INDEX's, verified by stashing.

## 5. What this does not preclude

- `U-IC` / `U-HOTPATH` still own the dispatch loop's *structure*; this only removes code from it,
  and makes their future measurements less noisy by removing an 18% constant.
- Debugging the dispatch loop stays available via `--features vm-trace`.

## Tests

Behavior invariance is the whole test strategy — there is no new behavior to test. The gate is
the existing golden `.ph` corpus + `cargo test --workspace`, plus a build of **both** feature
states (a `#[cfg]` that only compiles one way is the failure mode this unit could introduce).
