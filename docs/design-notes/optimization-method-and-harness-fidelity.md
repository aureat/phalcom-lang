# Optimization method + harness fidelity — findings (2026-07-14)

**Status: FINDINGS.** Grounded in the U-TRACE cut (`1ef999b`, [perf-log 003](../forge/perf-log/003-vm-trace-feature-gate.md),
finding [F9](../forge/perf-log/findings.md)) and the measurements taken around it. Governed by
[performance.md](../spec/v0.2/performance.md) + [ADR-0051](../adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md)
(measure-first, tiered, behavior-invariant).

Per-cut records live in `docs/forge/perf-log/`. **This file is the generalizable layer** — what the
session taught about *how to optimize this VM at all*, and which of those lessons outlive the cut.
Two items here (O4, O5) are **open** and gate later work; they are the reason this file exists
rather than being folded into perf-log 003.

---

## O1 — Hot-loop instrumentation is a category, not a `tracing` quirk

U-TRACE removed a per-opcode `tracing` span + three `debug!`s for **−16.7%** on arith. The
generalizable fact is not about `tracing`:

> **No runtime filter can remove code.** `LevelFilter::OFF` was set the entire time
> (`bin/phalcom/main.rs:15`) and bought nothing. A filter suppresses *output*; the compiler still
> emits the callsite. Only `#[cfg]` removes instructions.

This applies to **any** per-opcode observability we are tempted to add — counters, assertions,
`debug_assert!`-adjacent checks, IC hit/miss statistics, GC allocation tallies. Each is free to
*write* and costs ~8% of arith to *leave in*, whether or not it is switched on.

The tax scales with opcodes executed, so it is nearly invisible on allocation-bound workloads
(Skynet ≈ −1%) and brutal on dispatch-bound ones. A cost that hides on the headline benchmark and
bites on the micro-bench is exactly the cost that survives review.

**Proposed policy (no ADR yet — worth one if we add a second such gate):** any instrumentation
inside `vm/dispatch.rs`'s loop ships `#[cfg]`-gated from the first commit, never "filtered off for
now". `U-IC` will want hit/miss counters; that is the next place this bites.

## O2 — Attribution ≠ mechanism, and only mechanism gives the fix its shape

F1's profile said "tracing span, 18.3% arith". The **size was right** (18.2% measured). Both
natural readings of the **cause** were wrong:

| Theory | Why it was attractive | Measured |
|---|---|---|
| Subscriber misconfiguration (`Interest::sometimes()` → dynamic `enabled()` per opcode) | one-line fix, touches no VM code | **−0.4% — refuted** |
| The span is the cost (it is what the profiler named) | the profile literally says "span" | **half** — span −8.4%, the three `debug!`s −8.4%, both −18.2% |

A unit dispatched from the note's own framing ("Tier 1 — tracing span") would have gated the span,
booked −8.4%, closed the candidate, and written the false mechanism down as fact.

> A profiler names the **line where samples landed**, not the **cause**. Neighbours costing the
> same are invisible to it — the three `debug!`s sat two lines away and were never mentioned.

The discipline that caught it is cheap and should be standard: **build one variant per suspect and
A/B them.** Four binaries, ~10 minutes, and it both doubled the win and prevented a wrong claim
from entering the record. This is the concrete form of the perf-log README's existing rule
("reproduce the baseline observation before explaining it") — generalized: reproduce the
*mechanism*, not just the *number*.

## O3 — The harness's fidelity to the real binary is itself a claim requiring verification

Three **independent** ways this repo's measurement setup misleads, all found in one session. This
is the finding with the longest reach: we have been treating the harness as the instrument and the
VM as the subject, when the harness is also a subject.

**(a) Criterion is structurally blind to a whole class of cost.** `benches/vm_bench.rs` drives
`Interpreter` directly and installs **no subscriber**. Callsite interest therefore caches as
`never` — a *materially cheaper code path than the real binary*, which installs a registry.
Criterion showed ~no win where whole-process measured −16.7%. **It would have vetoed a real win.**
The bench is not a smaller version of the program; it is a *different* program.

**(b) Criterion's saved baseline rolls on every run.** Plain `cargo bench` moves the previous run's
`new/` to `base/`, so the reported `change: [±x%]` always means **"vs whatever ran last"** — not vs
a fixed reference. Its confidence intervals are computed within a run and say nothing about the
across-run drift the README already documents (noise certified at `p = 0.00` twice). Any
comparison meant to survive must use `--save-baseline <name>`. Corollary: **a `cargo bench` run
destroys the comparison point it just used** (see O4 — this happened).

**(c) Skynet cannot resolve small effects on `real`.** Its `real` spread was **5.4 s within a
single unmodified binary** (13.98 / 16.89 / 19.40). The variance lives in `sys` (page faults,
allocation); `user` held to ±0.4 s across the same runs. `BASELINE.md`'s own 13.7–15.6 s range
said this and was not acted on.

The method that worked, and should be the default for anything under ~10%: **build variant
binaries, copy them aside, run them interleaved (A,B,A,B…) in one session, report `min` with
`median` as a consistency check.** Wall-clock noise is one-sided — background load only ever adds
time — so `min` is the cleanest estimate of the binary's own cost. Every U-TRACE Δ agreed between
`min` and `median` to within 0.2pp; where they disagree (Skynet), the honest output is *no number*.

## O4 — OPEN: `fiber_spawn` regressed ~37% against the previous run; unattributed

Observed once, in this session's criterion run:

```
fiber_spawn  time: [24.331 ms 27.151 ms 31.846 ms]
             change: [+21.120% +37.578% +62.196%] (p = 0.00 < 0.05)
             Performance has regressed.
```

**Not claimed as a regression, and deliberately not explained.** What is and is not known:

- Per O3(b), `base` was the *previous vm_bench run* of unknown vintage — and that run's numbers
  have now been **overwritten by mine**, so the comparison is not reproducible from disk.
- The interval is enormous (+21% to +62%) and `p = 0.00` from criterion has certified noise on this
  hardware before (README, method §).
- **The plausible story is that it is real:** U-GC steps 3–4 landed today (`94b6bbf`) and collection
  now actually runs at safepoints, which a fiber spawn/yield/call loop would pay for. U-GC step 5's
  own A/B was a null result, so nothing there contradicts this.

That last bullet is exactly the shape of an O2 error — a tidy mechanism fitted to one noisy
observation. **Recorded as an open question, not a finding.** Resolving it: build binaries at
`94b6bbf^` and `94b6bbf`, interleaved whole-process A/B per O3. Worth doing before U-GC is closed,
since "the collector costs fiber-heavy code ~37%" would be a material fact about ADR-0050 — and
"it was noise" is equally worth knowing.

## O5 — OPEN: the attribution table that ranks the remaining tiers is now stale

`performance.md` §2 / F1's shares — malloc 19.7% arith, dispatch lookup 13.9% arith, tracing
18.3% — come from **one profile of a binary that no longer exists**. Two cuts have since removed
large, *unequal* pieces of it:

- **001** (U-PRIM-ABI) removed the per-send argument `Vec` — arith −41.5%, i.e. most of that
  19.7% malloc share.
- **003** (U-TRACE) removed the ~18.2% tracing share outright.

Percentages are shares of a total, so **removing 18% of the denominator re-normalizes every
remaining share upward**. Illustratively, dispatch lookup's 13.9% is ~17% of the post-003 total,
and larger still after 001 — but that arithmetic compounds two stale numbers and is offered only to
show the direction, not as a figure to plan from.

**Consequence for the dispatch order.** The "next levers" ranking (Tier 4 U-GC > Tier 3 U-IC) is
inherited from the original profile. It may well still hold — U-IC's real blocker is F4's
preconditions (`SelectorId`, `ClassObject` epoch), not its rank. But **U-IC should not be sized or
justified from 13.9%**; that number describes a binary two cuts in the past.

**Recommended before Tier 3/4 dispatch work:** re-profile arith + Skynet at current HEAD and
refresh `performance.md` §2 / `BASELINE.md`. This is cheap next to U-IC itself, and U-IC is the
unit most exposed to a stale denominator. It also gives the tiers a *clean* baseline for the first
time — pre-003 measurements carry an 18% constant that was masking everything else, including,
plausibly, whatever O4 is.

---

## What this does not preclude

- Nothing here changes committed semantics; U-TRACE was behavior-invariant and no ADR moved.
- O1's policy is a *proposal*. It binds nothing until an ADR says so, and one gate is not yet a
  pattern — revisit when `U-IC` adds the second.
- O4 and O5 are questions, not verdicts. Neither blocks U-GC's remaining close-out work
  (`DEFERRED.md` temp-root note, miri lane, reviewer gate); both should be answered before the
  numbers in them are used to justify a unit.
