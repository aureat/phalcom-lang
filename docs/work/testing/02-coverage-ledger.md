# 02 — Coverage Ledger

> Census of what the tree tests **today**, and the named gap each lane closes.
> Counts are from the tree as of 2026-07-19 and will drift; re-derive with the
> commands in §5 rather than trusting the numbers.

## 1. What exists

### Acceptance corpus — `phalcom-core/tests/lang/`

652 `.ph` cases across 35 label directories, run through the real `phalcom` CLI
as a subprocess by [`tests/support/mod.rs`](../../phalcom-core/tests/support/mod.rs).
Three case kinds per [MANIFEST.md](../../phalcom-core/tests/lang/MANIFEST.md):
PASS (exact stdout), NEGATIVE (non-zero exit, `.expected` as stderr substring,
never a panic), PENDING (`<label>/pending/`, `#[ignore]`, pins intended future
output).

Concurrency label — 50 cases:

| Group | Count | Covers |
|---|---|---|
| `*fiber*` | 33 (incl. 9 negative) | `new`/`call`/`try`/`yield`/`resume`/`abort`/`isDone`/`error`, capture, nesting, two-way channel, Wren-suite port, dead-frame non-local return |
| `*future*` | 11 | settle-once, `then`/`map`/`catch` chains, rejected-skips-then, `async`/`await`, `value`/`error`/`isReady` |
| `*sched*` | 6 | FIFO order, empty queue, drain-including-nested, schedule-is-not-synchronous, root drive at exit, raising fiber does not abort host |

### GC — `phalcom-core/tests/gc.rs`

14 `#[test]`s driving the collector directly: sweep of unreachable objects,
cycle collection, kernel survival, transitive retention, deep-chain sweep
without stack overflow, handle stability across collection, `System.gc`
(returns/collects/idempotent), the alloc-latches-but-never-collects rule
(Invariant L), automatic safepoint firing, a bounded-churn workload,
`suspended_fiber_roots_its_stack`, and `verify_invariants` after an automatic
collection.

### Invariants — `phalcom-core/tests/invariants.rs` + `Universe::verify_invariants`

Structural assertions over the class/metaclass tower, the `nil`/`None`
boundary, the sealed hierarchy, and the core-class census (29 rows, `Fiber`
joined 2026-07-15).

## 2. What that leaves uncovered

Each row is the gap, not a complaint about the tests that exist — the existing
tests are correct and well-anchored. The gaps are structural: no oracle in the
tree can see them.

| # | Gap | Why nothing catches it | Closed by |
|---|---|---|---|
| G1 | **The 652-case corpus runs with GC effectively never firing.** Programs are small; `INITIAL_GC_THRESHOLD` is not crossed. So the corpus exercises ~zero root-set correctness despite being the largest body of Phalcom programs that exists. | Threshold policy, not test design. Nothing is asserted about collection because nothing collects. | **Lane A** |
| G2 | **Missed root under a specific safepoint.** `collect_roots` walks frames, stack, `current`, `open_upvalues`, `ready_queue`, modules, classes, `sealed_classes`, `checking`, and `universe.each_handle`. A future addition that forgets a root is invisible until GC lands at exactly the wrong safepoint. | `gc.rs` drives `force_gc()` at points the *test author chose*. It cannot choose the point an unwritten bug needs. | **Lane A** |
| G3 | **Retention (leak).** An object kept alive forever by a stale reference — a ready-queue residue, a decorator's receiver-keyed side table (the hazard ADR-0052 names and explicitly leaves to "written contract, not static analysis"), a fiber shell whose resumer link outlives it. | No test asserts that memory *returns*. `bounded_churn_real_workload` is the nearest and asserts a bound on one workload, not convergence across repeats. | **Lane B** |
| G4 | **A native re-entry that fails to raise `native_reentry_depth`.** The yield/resume guard is a single VM counter (§3), not a per-primitive check. Any new primitive that calls `run_until` without incrementing it silently disarms the guard for every fiber under it. | The 9 negative fiber fixtures test the guard *firing*, from the call sites that already increment. Nothing tests the set of increment sites is complete. | **Lane C** |
| G5 | **`native_reentry_depth` leaking on an error path.** All four increment sites use `let result = …; depth -= 1; result` rather than `?`. Correct today. A future `?` — or a panic — leaves the counter high, permanently disarming yields for the rest of the process. | No assertion that the counter returns to zero. | **Lane C** |
| G6 | **Unexercised `Future` state-machine cells.** 11 fixtures against a `{pending, fulfilled, rejected}` × `{then, map, catch, settle, reject, await}` × `{before-settle, after-settle}` space. Coverage was chosen by intuition. | Hand-authored fixtures over a state machine are always holey — see [13-future-conformance.md](13-future-conformance.md) §1 for the precedent and its cost. | **Lane D** |
| G7 | **Right answer, wrong schedule.** A resumer-chain or ready-queue reordering bug that converges to the same prints. | Every concurrency fixture's oracle is stdout, which is a projection of the schedule. | **Lane E** |
| G8 | **FIFO is asserted but not specified.** `concurrency_sched_fifo_order` pins FIFO ordering. Scheduler fairness policy is **OPEN** (concurrency.md §3, ADR-0030). The test currently converts an open question into a de-facto contract by accident. | Nothing distinguishes contract tests from characterization tests. | **Lane E** §4 |
| G9 | **Program shapes nobody wrote.** Deep fiber nesting under GC pressure, schedule-during-drain, abort mid-chain, yields interleaved with allocation churn. | 50 hand-written fixtures cover the shapes the author thought of. | **Lane F** |

## 3. The re-entrancy guard as it actually is

Worth stating precisely, because the obvious mental model is wrong and Lane C
depends on the correct one.

There is **no per-primitive guard**. There is one VM counter,
`VM::native_reentry_depth` ([`vm/mod.rs:91`](../../phalcom-core/src/vm/mod.rs)),
and two checks:

- `fiber_resume` ([`primitive/fiber.rs:248`](../../phalcom-core/src/primitive/fiber.rs))
  raises `CannotYieldAcrossNativeFrame` if the counter is nonzero.
- `fiber_yield` ([`primitive/fiber.rs:338`](../../phalcom-core/src/primitive/fiber.rs))
  raises if the counter differs from the current fiber's recorded `floor_depth`.

The counter is raised at exactly **four** sites, each wrapping a re-entrant
`run_until`:

| Site | Path |
|---|---|
| [`interpret.rs:269`](../../phalcom-core/src/interpret.rs) | the interpreter's own re-entry |
| [`primitive/block.rs:158`](../../phalcom-core/src/primitive/block.rs) | `Block#call` — the `.each { }` case |
| [`vm/send.rs:233`](../../phalcom-core/src/vm/send.rs) | `send_dynamic` — reflective `perform`, and `doesNotUnderstand` forwarding |
| [`vm/send.rs:276`](../../phalcom-core/src/vm/send.rs) | `invoke_method_object` — `Method#invokeOn`, `Method#bind` |

This is a **good** design: it centralizes the guard so `List#map`, `Map` key
hashing, sort comparators, `toString`, dNU, and decorator interception all
inherit it by routing through `block_call` or `send_dynamic` rather than each
re-deriving it. The testable consequence is not "does each primitive raise" but:

> **C-INV-1.** Every path from native Rust code into `run_until` passes through
> a site that increments `native_reentry_depth`.
>
> **C-INV-2.** `native_reentry_depth` is zero whenever the dispatch loop is at
> top level, on both the success and the error path.

Neither is asserted anywhere today. Both are mechanically checkable. That is
Lane C.

## 4. Priority rationale

Lane A is first by a wide margin, and the reason is arithmetic rather than
taste: it converts 652 existing verified programs into 652 GC tests for the cost
of one threshold knob and one CI job. Every lane below it requires authoring
proportional to its coverage. Lane A's coverage grows for free with every future
corpus addition — including additions made for unrelated reasons.

Lane B is second because G3 is the only gap in the list that is **permanently**
invisible: a leak never prints, never panics, and never fails a golden test, no
matter how many are written.

Lanes C through F are ordered by (gap severity ÷ authoring cost), and are
genuinely reorderable if circumstances change — for instance, C jumps to first
if a new re-entrant primitive is about to land.

## 5. Re-deriving these numbers

```sh
find phalcom-core/tests/lang -name '*.ph' | wc -l              # corpus size
find phalcom-core/tests/lang/concurrency -name '*.ph' | wc -l  # concurrency label
grep -c '^#\[test\]' phalcom-core/tests/gc.rs                  # GC test count
grep -rn 'native_reentry_depth += 1' phalcom-core/src          # guard increment sites
```

The last one is the load-bearing check: **if it returns more than the four sites
in §3, this ledger is stale and Lane C's census must be regenerated.**
