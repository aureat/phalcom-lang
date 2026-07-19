# 14 — Lane E: Schedule Trace

> **Oracle:** pinned trace — **characterization**, not contract (see §4).
> **Closes:** [G7, G8](02-coverage-ledger.md#2-what-that-leaves-uncovered).
> **Status:** specified, not built.

## 1. The claim

Every concurrency fixture's oracle is stdout. Stdout is a **projection** of the
schedule, and the projection has a kernel: a resumer-chain corruption, a
ready-queue reordering, or a fiber resumed twice can all converge to the same
prints.

Cooperative determinism (ADR-0030) makes the schedule a pure function of the
program text. So the schedule is not a race to be sampled — it is a *value* that
can be observed and compared. Lane E observes it.

## 2. Mechanism

A flag-gated event log emitted to **stderr** (stdout is pinned by the corpus and
must not move):

```
FIBER  spawn   id=3 parent=1
FIBER  resume  id=3 from=1
FIBER  yield   id=3 to=1 value=<n>
SCHED  enqueue id=4
SCHED  dequeue id=4
FIBER  done    id=3
GC     collect swept=12 live=340
```

Requirements that make the trace usable rather than decorative:

- **Stable fiber ids** — sequential from a per-VM counter, not handle indices,
  which shift as the heap changes and would make every trace churn.
- **No wall-clock, no addresses, no handle values.** Anything non-deterministic
  in the trace defeats the point.
- **Off by default, zero cost when off.** A `PHALCOM_TRACE=sched,fiber,gc`
  filter, checked once at startup into a bitmask.
- **`GC collect` lines belong to Lane A, not here** — they are the mechanism by
  which a stress-mode failure can be localized to a safepoint. Include the
  category; exclude those lines from Lane E's pinned traces, since collection
  frequency is configuration-dependent.

## 3. What it catches that stdout does not

| Bug | stdout | trace |
|---|---|---|
| Fiber resumed by the wrong resumer, results still converge | silent | `resume from=` differs |
| Ready-queue drained LIFO where FIFO was intended, order-independent tasks | silent | `dequeue` order differs |
| Fiber resumed twice, second resume a no-op | silent | duplicate `resume` |
| Nested drain re-entering the queue during drain | usually silent | `enqueue`/`dequeue` interleave |
| Waiter scheduled twice on settle | silent (idempotent callback) | duplicate `enqueue` |

The last two are the practical motivation: `concurrency_sched_run_scheduled_drains_including_nested`
exercises nested drain today and asserts only its printed result.

## 4. Characterization, not contract — the load-bearing rule

**Scheduler fairness policy is OPEN** (concurrency.md §3, ADR-0030). The
overlay is explicit: the ready-queue exists as *mechanism*; no fairness *policy*
is specified.

`concurrency_sched_fifo_order` already pins FIFO. That test is not wrong, but it
currently converts an open design question into a de-facto contract by accident,
which is [G8](02-coverage-ledger.md#2-what-that-leaves-uncovered). Lane E would
multiply the problem across every traced fixture.

The rule:

> Trace fixtures live in `tests/lang/concurrency/trace/` and carry a header:
>
> ```
> # CHARACTERIZATION — pins observed scheduler behavior, not specified behavior.
> # Fairness policy is OPEN (concurrency.md §3, ADR-0030). A future fairness ADR
> # may rewrite these traces wholesale; that is a graduation, not a regression.
> ```

And, separately from this lane: `concurrency_sched_fifo_order` should be
**either** promoted (write FIFO into concurrency.md §3 as the specified policy,
making the test a genuine contract) **or** relabelled characterization. Leaving
it ambiguous is the actual defect, and it is a one-line fix that does not
require Lane E to exist. Do it independently, and sooner.

The distinction is not bookkeeping. A future implementer facing 40 red traces
needs to know in the first ten seconds whether they broke something or merely
changed something the spec never promised.

## 5. Cost and gating

Moderate: the trace emitter is real runtime work, and pinned traces are verbose
and churn under unrelated changes. That churn is the main cost and the main
argument for keeping the traced set small.

- Instrument a **selected subset** — the ~15 concurrency fixtures where the
  schedule is the point — not all 50, and certainly not all 652.
- **In the default green gate**, since it is fast once the traces exist.

## 6. Preclusion

- **Traces make scheduler order a compat surface.** Every pinned trace is a
  constraint on future scheduling. Work-stealing is precluded by ADR-0030
  already (single-threaded), but priority queues, fairness quanta, and
  starvation avoidance are all live options that traces would fight. The
  characterization header in §4 is the mitigation and it is mandatory, not
  advisory.
- **Trace output on stderr constrains diagnostics.** NEGATIVE corpus cases match
  diagnostics as a stderr substring. A trace-enabled run interleaves trace lines
  with diagnostics, so the two must never be enabled together in the same
  assertion — or the trace must go to a third stream (fd 3, or a file named by
  the env var). Prefer the file: it keeps stderr clean and makes trace
  comparison a file diff, which is also a better failure report.
- **Instrumenting the dispatch loop risks perturbing what it measures.** Keep
  the emit behind a branch on a startup-resolved bitmask, and keep it out of the
  hot path where the perf log's measured per-instruction cost would notice.
