# 08 — Performance epistemology

> **Thesis:** almost every performance mistake in this project has been an *epistemic* mistake
> rather than an engineering one. The number was right and the cause was wrong; the harness was
> measuring a different program; the profile described a binary that no longer existed; a
> plausible mechanism was fitted to a single noisy observation. Optimization discipline is
> mostly a discipline about what you are entitled to believe.

**`[V]`** Primary sources: `docs/design-notes/optimization-method-and-harness-fidelity.md`
(findings O1–O5) and `docs/design-notes/bytecode-representation-and-borrowed-techniques.md`
(findings B1–B5), both grounded 2026-07-14, both governed by ADR-0051 (measure-first, tiered,
behavior-invariant). Per-cut numbers live in `docs/forge/perf-log/`; **that ledger is the only
citable source for a number**, and quoting a figure from memory is a known way to be wrong here.

---

## 1. A runtime filter cannot remove code

**`[V]`** Finding O1. Removing a per-opcode tracing span and three `debug!` calls from the dispatch
loop bought **−16.7%** on the arithmetic benchmark. The instructive part is what had been assumed:

> **No runtime filter can remove code.** `LevelFilter::OFF` was set the entire time
> (`bin/phalcom/main.rs:15`) and bought nothing. A filter suppresses *output*; the compiler still
> emits the callsite. Only `#[cfg]` removes instructions.

The generalization is a category rather than a quirk of one logging crate: **every** per-opcode
observability feature — counters, assertions, IC hit/miss statistics, allocation tallies — is free
to *write* and costs roughly eight percent of a dispatch-bound workload to *leave in*, switched on
or not.

And the second-order observation, which is the one that transfers furthest:

> The tax scales with opcodes executed, so it is nearly invisible on allocation-bound workloads
> (Skynet ≈ −1%) and brutal on dispatch-bound ones. **A cost that hides on the headline benchmark
> and bites on the micro-bench is exactly the cost that survives review.**

Costs are not caught in proportion to their size. They are caught in proportion to their
visibility on whatever benchmark the team habitually runs — so the survivors are precisely the
costs orthogonal to that benchmark. This is a selection effect on your own review process, and
the only defense is to know which axis your headline benchmark is blind to.

---

## 2. Attribution is not mechanism

**`[V]`** Finding O2, and the single most valuable item in the whole performance record.

A profile said "tracing span, 18.3% of arith." The **size was right** — 18.2% when measured
directly. Both natural readings of the **cause** were wrong:

| Theory | Why it was attractive | Measured |
|---|---|---|
| Subscriber misconfiguration causing a dynamic `enabled()` check per opcode | one-line fix, touches no VM code | **−0.4% — refuted** |
| The span *is* the cost (it is what the profiler named) | the profile literally says "span" | **half** — span −8.4%, the three `debug!`s −8.4%, together −18.2% |

The counterfactual is the point:

> A unit dispatched from the note's own framing ("Tier 1 — tracing span") would have gated the
> span, booked −8.4%, closed the candidate, and written the false mechanism down as fact.

The team would have shipped a real improvement, recorded a wrong explanation, and left half the
win on the table — while believing the candidate was closed. Note that nothing about that outcome
looks like a failure from the inside. The benchmark improved.

> A profiler names the **line where samples landed**, not the **cause**. Neighbours costing the
> same are invisible to it — the three `debug!`s sat two lines away and were never mentioned.

**`[V]`** The corrective is cheap and should be standard: **build one variant binary per suspect
and A/B them.** Four binaries, about ten minutes, and it both doubled the win and prevented a
false claim from entering the record. Stated as a generalization of the perf-log's existing rule:
reproduce the *mechanism*, not just the *number*.

This is the same failure mode as the citation incident in
[`00-provenance-and-citation-discipline.md`](00-provenance-and-citation-discipline.md) — a true
artifact (the 18.3% figure; the Conway bibliography) attached to a false account of where it came
from. In both cases the true part functions as a credential for the false part.

---

## 3. The harness is a subject, not an instrument

**`[V]`** Finding O3, three independent ways one repository's measurement setup misled, all found
in a single session.

**(a) The benchmark harness was a different program.** `benches/vm_bench.rs` drives the
interpreter directly and installs no tracing subscriber. Callsite interest therefore caches as
`never` — **a materially cheaper code path than the real binary**, which installs a registry. The
harness showed approximately no win where whole-process measurement showed −16.7%. Stated
starkly: *it would have vetoed a real win.* The bench is not a smaller version of the program; it
is a different program.

**(b) The saved baseline rolls.** A plain `cargo bench` run moves the previous run's `new/` to
`base/`, so the reported change always means "versus whatever ran last" — not versus a fixed
reference. Confidence intervals are computed within a run and say nothing about across-run drift.
Corollary: **a benchmark run destroys the comparison point it just used.**

**(c) The headline benchmark cannot resolve small effects.** Skynet's wall-clock spread was
**5.4 seconds within a single unmodified binary** (13.98 / 16.89 / 19.40). The variance lives in
`sys` — page faults, allocation — while `user` held to ±0.4 s across the same runs.

**`[V]`** The method that worked, and the default for anything under ~10%:

> build variant binaries, copy them aside, run them interleaved (A,B,A,B…) in one session, report
> `min` with `median` as a consistency check.

The justification is a distributional argument worth remembering: **wall-clock noise is one-sided**
— background load only ever *adds* time — so `min` is the cleanest estimate of the binary's own
cost, and the mean is systematically pessimistic by an amount that depends on machine load rather
than on the code. When `min` and `median` disagree, the honest output is *no number*.

**`[M]`** Two operational corollaries recorded elsewhere in this project and worth repeating,
because both were learned by losing a full round of measurements: never time on a loaded machine
(a load average of 7–10 on 8 cores voided an entire round), and note that a benchmark process can
**saturate the machine itself** — the guard must wait for load below ~0.5, not below ~1.5,
because the child adds roughly 1.0 and would otherwise abort its own measurement window.

---

## 4. Shares are shares of a denominator that keeps changing

**`[V]`** Finding O5, the subtlest of the set. An attribution table ranking the remaining
optimization tiers — malloc 19.7% of arith, dispatch lookup 13.9%, tracing 18.3% — came from **one
profile of a binary that no longer exists**. Two subsequent cuts removed large and *unequal*
pieces: the per-send argument vector (arith −41.5%, i.e. most of the malloc share) and the tracing
share outright.

> Percentages are shares of a total, so **removing 18% of the denominator re-normalizes every
> remaining share upward**.

The consequence is a live risk to planning, not a curiosity: the "next levers" ranking is inherited
from the original profile, so a unit may be sized and justified from a number describing a binary
two cuts in the past. The note's own arithmetic correction is offered "only to show the direction,
not as a figure to plan from" — itself a nice piece of discipline, refusing to replace a stale
number with a freshly computed but equally unfounded one.

**Generalizable:** a profile is a snapshot of a *ratio*, and every optimization you land
invalidates every other entry in the table. Re-profile after any cut large enough to move the
denominator, and treat an un-refreshed ranking as expired rather than approximate.

---

## 5. The catalogue of refuted hypotheses

Kept deliberately, per the rule in
[`00-provenance-and-citation-discipline.md`](00-provenance-and-citation-discipline.md) §R6: a dead
hypothesis with a cause of death attached is worth more than a live one with no test.

- **`[X]` Subscriber misconfiguration was the tracing cost.** Refuted at −0.4%. Attractive because
  it was a one-line fix touching no VM code — *the cheapness of a proposed fix is not evidence for
  its diagnosis*, though it reliably feels like evidence.
- **`[X]` Fiber pooling helps under high turnover.** **`[M]`** Measured against the `fiber_churn`
  benchmark that had been nominated as the pool's best case: +37% user and +72% RSS at one million
  fibers, +86% RSS at one hundred thousand. The pool is bounded at 100 entries yet adds ~450 bytes
  *per fiber* — a linear cost from a bounded structure, which is the tell. The mechanism is that
  recycled capacity is retained by shells that outlive their run, so the lever is the shell's
  lifetime, not the buffer's size. The earlier "null result" verdict was *too generous*; the
  feature is not neutral but actively negative, and stays off.
- **`[X]` Pre-sizing fiber vectors helps.** **`[M]`** +2.4% user on skynet, +20% user and +121% RSS
  on `fiber_churn`. Reverted. Same mechanism as the pool: ~640 bytes of retained capacity per
  GC-lifetime shell.
- **`[X]` Operand-folding superinstructions would help.** Refuted *statically*, by precondition
  analysis rather than measurement — there is no second fetch to delete. See
  [`07-borrowed-techniques-and-their-preconditions.md`](07-borrowed-techniques-and-their-preconditions.md).
- **`[X]` Two long-standing microarchitectural hypotheses about instruction-cache pressure and
  initialization cost.** Refuted statically: the relevant L1i is 192 KiB, so the loop cannot miss
  it, and the "128 byte init" was in fact 8 bytes. Both had been recorded as findings and had
  propagated into the scoreboard before correction.
- **`[O]` A ~37% fiber-spawn regression, observed once.** Deliberately *not* explained. The
  baseline was a previous run of unknown vintage that has since been overwritten; the interval was
  +21% to +62%; and the same tooling has certified noise on this hardware before. A tidy mechanism
  was available (a collector had begun running at safepoints that day) and was explicitly refused,
  because fitting a mechanism to one noisy observation is exactly the O2 error. Recorded as an open
  question.

The pattern across the list: **four of the six were killed by a check that cost minutes**, and in
several cases the check was cheaper than writing the hypothesis down. The bottleneck is not
measurement capacity. It is the habit of asking what would distinguish the hypothesis from its
neighbours before believing it.

---

## 6. The rules, compressed

1. **No number without a recorded before/after** from a reproducible in-repo benchmark. No oral
   numbers, ever — this project has a standing rule that speculation may not be filed in the
   measured ledger, because doing so corrupts the one artifact that is trustworthy.
2. **No mechanism without a variant binary.** A profile names a line; only an A/B names a cause.
3. **Instrumentation in a hot loop ships `#[cfg]`-gated from the first commit**, never "filtered
   off for now."
4. **Interleave A/B/A/B in one session, report `min`.** Noise is one-sided; the mean is not your
   friend.
5. **Check machine load before timing.** A shared or self-saturated box voids the round.
6. **Re-profile after any cut that moves the denominator.** Stale shares are expired, not
   approximate.
7. **Behavior-invariant.** A performance change that alters an observable is a specification
   change requiring its own decision record, not a performance sneak. The golden corpus stays
   byte-identical.
