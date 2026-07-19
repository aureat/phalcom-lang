# 10 — Lane A: GC Stress

> **Oracle:** metamorphic — `output(P, collect at every safepoint) ≡ output(P, default)`.
> **Closes:** [G1, G2](02-coverage-ledger.md#2-what-that-leaves-uncovered).
> **Status:** specified, not built.

## 1. The claim

ADR-0050 selects a **non-moving** precise mark-sweep collector over the handle
arena. Non-moving means object addresses never change; combined with ADR-0009's
handle indirection, it means **no correct program can observe that a collection
happened**. Identity, hashing, and equality are all handle-based, not
address-based.

That unobservability is not a nicety — it is a testable equation:

```
∀ P : output(P, gc at every safepoint) ≡ output(P, gc at default threshold)
```

Any program where the two differ has found a bug, and the bug is in the runtime,
not the program. The lane needs **no expected output of its own**: the corpus
already carries 652 hand-verified ones.

## 2. Mechanism

Collection is already latched-and-serviced, which is what makes this cheap.
[`Heap::alloc`](../../phalcom-core/src/heap/mod.rs) sets `gc_pending` when
`objects.len()` crosses `next_gc` — it **latches only, never collects**
(Invariant L). The dispatch loop's back-edge calls
[`service_gc_safepoint`](../../phalcom-core/src/vm/gc.rs), which collects if the
flag is set.

Stress mode is therefore one knob: **make every allocation latch.**

```rust
// Heap::new / Heap::alloc
next_gc: if gc_stress_enabled() { 0 } else { INITIAL_GC_THRESHOLD },
```

With `next_gc == 0`, `objects.len() >= next_gc` holds unconditionally, every
allocation latches, and every safepoint collects. The growth policy
(`GC_UNPRODUCTIVE_GROW_FACTOR`) must be suppressed in stress mode so the
threshold does not climb away from zero after the first unproductive cycle.

Critically, **the latch/service split is preserved**. Stress mode does not
collect from `alloc`; it only raises the frequency at which the *existing*
safepoint collects. Invariant L is untouched, so stress mode does not test a
different collector — it tests the same collector, more often. This is what
makes the metamorphic relation legitimate rather than aspirational.

### Configuration surface

Environment variable, read once at `Heap::new`:

| Value | Behavior |
|---|---|
| unset / `0` | Default threshold policy. |
| `1` | Collect at every safepoint (`next_gc = 0`, growth suppressed). |
| `N` (>1) | Collect every Nth safepoint. A middle gear for bisecting a stress failure that is too slow at `1`. |

Environment rather than a CLI flag because the corpus harness spawns the real
`phalcom` binary as a subprocess and asserts on its stdout; an env var threads
through `Command::env` without perturbing argv, which the NEGATIVE cases pin.

## 3. Test construction

No new fixtures. The existing corpus runner is parameterized:

```rust
// tests/support/mod.rs
fn run_case(path: &Path) -> Output {
    let mut cmd = Command::new(phalcom_bin());
    cmd.arg(path);
    if let Ok(v) = std::env::var("PHALCOM_GC_STRESS") {
        cmd.env("PHALCOM_GC_STRESS", v);   // propagate to the child
    }
    cmd.output().expect("failed to spawn the `phalcom` binary")
}
```

The lane is then a CI invocation, not a code path:

```sh
PHALCOM_GC_STRESS=1 cargo test -p phalcom-core --test lang
PHALCOM_GC_STRESS=1 cargo test -p phalcom-core --test golden
```

Assertions are unchanged — the same `.expected` comparison, the same
no-panic check. A stress failure surfaces as an ordinary corpus failure naming
the exact `.ph` file, which is already a minimal reproducer.

### Targeted additions

The corpus reuse is the bulk of the value, but four shapes deserve dedicated
fixtures because they are underrepresented and are precisely where a missed root
would hide. These go in `tests/lang/concurrency/` and run in **both** modes.

| Fixture | Shape | Root it probes |
|---|---|---|
| `gcstress_scheduled_fiber_sole_root` | schedule a fiber, drop every `.ph` reference to it, allocate churn, then drain | `ready_queue` — a fiber reachable *only* from the scheduler |
| `gcstress_suspended_fiber_upvalues` | fiber captures an enclosing local, suspends, churn, resume and read the capture | `open_upvalues` + a suspended fiber's stored frames |
| `gcstress_future_waiter_chain` | long `then`/`map` chain, drop intermediates, churn, settle | waiter lists held by unsettled futures |
| `gcstress_dnu_reentrant_alloc` | `doesNotUnderstand` forward that allocates, under churn | native locals held across the `send_dynamic` re-entry |

The last one is the sharp case: it probes whether a native primitive holding a
fresh `ObjRef` in a Rust local across a re-entrant send has that handle rooted.
`force_gc`'s own doc names this as the `temp_roots` escape hatch's reason for
existing. Under default thresholds it essentially never fires; under stress it
fires on every allocation.

## 4. Cost and gating

Expect **100×–1000×** wall-clock on the corpus. Precedent: Ruby's `GC.stress`
and SpiderMonkey's `gcZeal` both land in that band, and both are run as
dedicated jobs for exactly that reason.

Mitigations, in order of preference:

1. **Not in the default green gate.** Nightly CI job plus an on-demand trigger.
2. **`PHALCOM_GC_STRESS=N` for bisection.** A failure at `1` that takes minutes
   to reproduce often reproduces at `N=100` in seconds.
3. **Shard by label.** The corpus is already label-partitioned; a matrix job
   over labels parallelizes without any new test infrastructure.

Do **not** mitigate by sampling the corpus. The corpus's value here is
exhaustiveness — a sampled stress lane is a worse version of Lane F.

## 5. Failure interpretation

A stress-mode failure is one of exactly three things. The triage order matters,
because the third is the one people reach for first and it is almost always
wrong.

1. **A missed root.** Output differs, or the process fails on a stale handle.
   The swept object was live. Fix `collect_roots` or add a temp root.
2. **A safepoint placed mid-opcode.** Collection ran while a value was popped or
   `split_off` the stack and held only in a Rust local — the window
   `service_gc_safepoint`'s doc warns about. Fix the safepoint placement, never
   the collector.
3. **The program was wrong to begin with.** Vanishingly rare, and only possible
   if the program observes allocation timing. Suspect this last.

The relation gives a bisection bonus: because the two runs are *supposed* to be
identical, any divergence is a first-order signal, and the diff between the two
stdouts points directly at the first observation that went wrong.

## 6. Preclusion — read before amending ADR-0050

**This lane is an executable encoding of "GC is behavior-invariant."** It is
sound only under a non-moving collector.

If the moving/compacting alternative ADR-0050 rejects is ever revived:

- Object identity or hashing derived from address would make collection
  observable, and the metamorphic relation would become **false**.
- The lane would not go red. It would go *wrong* — reporting failures for
  correct behavior, or, worse, passing because both runs moved objects the same
  way.

Therefore: the invariance assumption **must be written into the lane's own
module doc**, citing ADR-0050, so that anyone amending the collector finds the
dependency at the amendment site rather than in this directory. A moving
collector requires re-deriving the relation (likely to "identity-preserving
projections of the output are equal"), not merely re-running the lane.

Secondary preclusion: parameterizing the corpus harness on an env var makes
runtime configuration part of the harness contract. Future lanes wanting their
own knob should extend the same mechanism rather than forking the runner —
otherwise the corpus acquires N incompatible runners and stops being reusable,
which would forfeit the exact property that makes this lane worth building.
