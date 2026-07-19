# 11 — Lane B: Steady State (Leak Detection)

> **Oracle:** invariant — live-object count converges across repeated identical workloads.
> **Closes:** [G3](02-coverage-ledger.md#2-what-that-leaves-uncovered).
> **Status:** specified, not built.

## 1. The claim

Lane A proves the collector does not free too much. Nothing proves it frees
**enough**. Retention is the only failure class in this specification that is
*permanently* invisible: a leaked object never prints, never panics, never fails
a golden test, and never will, no matter how many fixtures are written. It
degrades a long-running process and nothing else.

The invariant that catches it:

> **B-INV-1.** For a workload `W` with no intentional accumulation, the live
> object count after `k` iterations followed by a full collection is constant
> for all `k ≥ k₀`.

The `k₀` allowance is not slack — it absorbs legitimate one-time effects:
interned symbols, memoized module handles, lazily-created classes, the
canonical-path import memo. Those grow once and stop. A leak grows per
iteration, forever. The distinction is the whole test.

## 2. Why this is not `bounded_churn_real_workload`

[`gc.rs`](../../phalcom-core/tests/gc.rs) already has
`bounded_churn_real_workload`, which asserts a *bound* on one workload. That
catches catastrophic retention. It does not catch a per-iteration leak of a
handful of objects, because a small leak stays under any bound generous enough
to accommodate the one-time effects in §1.

Convergence is strictly stronger than boundedness, and is what distinguishes
"grew once" from "grows always" without needing to know the right absolute
number for either.

## 3. Mechanism

Build on the `settled_vm()` idiom already in `gc.rs` — it exists because a
freshly bootstrapped VM is not garbage-free (bootstrap leaves `core.ph`'s
top-level closure unreachable), so any exact live count must baseline *after* a
collection.

```rust
/// Runs `source` `iterations` times in one VM, sampling the settled live count
/// after each. Returns one sample per iteration.
fn live_counts_over(source: &str, iterations: usize) -> Vec<usize> {
    let mut vm = VM::new();
    vm.force_gc();                     // settle bootstrap garbage first
    (0..iterations)
        .map(|_| {
            vm.run_source(source).expect("workload must succeed");
            vm.force_gc();
            vm.heap.live_count()
        })
        .collect()
}
```

The assertion is on the **tail**, not the whole series:

```rust
/// Asserts the live count stops growing: the last `tail` samples are all equal.
fn assert_converges(samples: &[usize], tail: usize) {
    let window = &samples[samples.len() - tail..];
    assert!(
        window.iter().all(|n| *n == window[0]),
        "live count still growing after warmup — leak. samples: {samples:?}"
    );
}
```

Defaults: 12 iterations, tail of 5. Tune per workload; record the reason in the
test's doc comment when a workload needs a longer warmup, since an unexplained
long warmup is itself evidence of slow accumulation.

### Diagnosis mode

A bare "it grew" is a weak report. When a workload fails, the per-iteration
delta is usually the whole diagnosis — a delta that equals the number of
scheduled fibers points at the ready queue; a delta of one per `then` points at
a waiter list. Emit the sample series in the failure message (as above) and, if
the heap can cheaply report a per-`Object`-variant census, emit the delta by
variant. That turns a leak hunt into a lookup.

## 4. Workload catalog

Each workload must **return the heap to its starting shape** by construction —
allocate, use, drop. A workload that intentionally accumulates cannot be tested
this way and belongs in a different lane.

| Workload | Probes | Leak it would catch |
|---|---|---|
| `fiber_create_run_complete` | fiber shell lifecycle | fiber objects retained after `Done` |
| `fiber_create_abort` | abort path | shells retained on the abnormal exit |
| `fiber_suspend_never_resumed` | abandoned fibers | a suspended fiber nothing will resume, still rooted from `open_upvalues` or a stale resumer link |
| `schedule_drain_repeat` | ready queue | queue entries not cleared after drain |
| `future_settle_and_drop` | waiter lists | waiters retained past settle |
| `future_chain_then_drop` | chain teardown | intermediate futures in a `then`/`map` chain |
| `closure_capture_release` | upvalues | `open_upvalues` entries not closed |
| `dnu_forward_repeat` | reflective dispatch | `Message`/args allocation per forward |
| `import_same_module_repeat` | module memo | should converge at `k=1` — a canary that the harness measures what it thinks |
| `decorated_method_invoke` | ADR-0052 hazard | **per-receiver decorator state in a side table** |

The last row is the one this lane exists for. ADR-0052 requires per-receiver
decorator state to live in a Layout-tier reserved slot, never an Install-tier
receiver-keyed side table, because a side table is a mark-sweep leak — and it
records that this is enforced as "written contract + golden-test snapshot, not
static analysis." A golden snapshot cannot see a leak. **This lane is what makes
ADR-0052's constraint actually enforced**, and the workload should cite the ADR
in its doc comment so the connection survives.

## 5. Cost and gating

Cheap: no stress mode, small workloads, ~12 iterations each. Full lane should
run in seconds. **In the default green gate.**

Runs as a Rust integration test (`tests/steady_state.rs`) rather than through
the CLI corpus, because it needs in-process access to `heap.live_count()`
between iterations — the subprocess harness cannot observe it.

## 6. Interaction with Lane A

Complementary, and worth stating because it is tempting to fold them together:

- Lane A: **does the collector free things that are still live?** (soundness)
- Lane B: **does the collector free everything that is dead?** (completeness)

They fail in opposite directions and share no machinery. Running Lane B *under*
`PHALCOM_GC_STRESS=1` is redundant — Lane B already forces a collection per
iteration, so stress adds runtime without adding an oracle.

## 7. Preclusion

- **Asserting exact live counts** would couple the suite to bootstrap object
  count, which changes whenever a kernel class is added. Assert *convergence*
  and *deltas*, never absolute totals. `settled_vm()`'s existing doc already
  makes this argument for its own baseline; the same reasoning applies here and
  is easy to violate accidentally when writing a new workload.
- **`live_count` as the metric** ties the lane to object count rather than
  bytes. Fine under the current uniform-handle heap; if variable-size or
  off-heap payloads (large strings, native buffers) ever land, a workload could
  converge in count while growing in bytes. The lane would need a byte metric
  alongside — not instead, since count is the better diagnostic of the two.
- **Convergence as the invariant** presumes workloads that return to their
  starting shape. It cannot test anything with legitimate unbounded growth. That
  is a real limit, not a defect: such a subsystem needs an explicit retention
  bound, and that bound is what should be asserted instead.
