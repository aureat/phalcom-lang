# Phalcom Runtime Testing Specification

**Status:** Draft 0.1 — design baseline. No lane below is built yet except where
the Coverage Ledger says otherwise.

This directory specifies how Phalcom's **runtime subsystems** are tested: the
garbage collector, fibers, futures, the scheduler, and the boundary guards that
hold them apart. It is a companion to, not a replacement for, the language
acceptance corpus (`phalcom-core/tests/lang/`, see
[MANIFEST.md](../../phalcom-core/tests/lang/MANIFEST.md)).

## Why this document exists

The acceptance corpus asserts **exact stdout for a program**. That is the right
oracle for syntax, dispatch, arithmetic, and error messages. It is the *wrong*
oracle — structurally, not incidentally — for every subsystem in scope here:

| Subsystem | What can go wrong | Visible in stdout? |
|---|---|---|
| GC | a root is missed; an object is swept while live | No — until the sweep lands at a safepoint the fixture never reaches |
| GC | an object is retained forever (leak) | No — never |
| Fibers | the resumer chain is corrupted but the prints still line up | No |
| Scheduler | tasks run in a different order that happens to converge | Sometimes |
| Futures | a state-machine cell no fixture exercises settles twice | Only for the cells someone thought to write |
| Native-frame guard | a re-entrant primitive lets a yield through | No — it corrupts, it does not print |

Every one of those is a **silent** failure under the corpus. This specification
defines the oracles that make them loud.

## Doctrine

1. **No oracle, no lane.** A test lane must state what it compares against —
   pinned output, an invariant, a metamorphic relation, or a model. A lane with
   no stated oracle is a smoke test; say so and price it accordingly.
2. **Reuse the corpus before writing fixtures.** Phalcom already has ~700
   verified `.ph` programs. A lane that re-runs them under a different runtime
   configuration buys coverage at near-zero authoring cost. Prefer that to hand
   authoring, always.
3. **Enumerate, do not intuit.** Where the state space is finite and known
   (future state machine, re-entrant native frames), generate the cases from the
   space. Hand-picked coverage of a state machine is always holey — see
   [13-future-conformance.md](13-future-conformance.md) for the precedent.
4. **Determinism is an asset; spend it.** Phalcom is cooperative and
   single-threaded (ADR-0030). There are no races to chase and no flaky
   interleavings. Every failure here is reproducible from the program text
   alone. This is what makes stress and fuzz lanes cheap.
5. **Never assert unspecified behavior as contract.** Where the spec is silent
   (scheduler fairness, cancellation propagation), a passing test still pins
   behavior. Tag those tests **characterization** so a future ADR can rewrite
   them without the diff reading as a regression. See
   [14-schedule-trace.md](14-schedule-trace.md) §4.

## Reading order

| Doc | Covers |
|---|---|
| [01-oracle-model.md](01-oracle-model.md) | The four oracle kinds, the decision rule for picking one, and why exact-output is a minority tool here |
| [02-coverage-ledger.md](02-coverage-ledger.md) | Census of what the tree tests **today**, grounded in file counts, and the named gaps each lane closes |
| [10-gc-stress.md](10-gc-stress.md) | **Lane A** — collect at every safepoint, re-run the whole corpus, assert byte-identical output |
| [11-steady-state.md](11-steady-state.md) | **Lane B** — leak detection by live-count convergence over repeated workloads |
| [12-reentrancy-census.md](12-reentrancy-census.md) | **Lane C** — one fixture per native frame that can re-enter `.ph`, enumerated from `src/primitive/` |
| [13-future-conformance.md](13-future-conformance.md) | **Lane D** — the `Future` state-machine cross-product, generated |
| [14-schedule-trace.md](14-schedule-trace.md) | **Lane E** — assert the schedule itself, not its side effects |
| [15-invariant-fuzz.md](15-invariant-fuzz.md) | **Lane F** — generated concurrency programs with an invariant-only oracle |

## Lane inventory

Priority order is build order. Each lane's cost is authoring cost; the runtime
cost is in the lane's own doc.

| Lane | Name | Oracle kind | Reuses corpus | Priority |
|---|---|---|---|---|
| A | GC stress | Metamorphic (`gc_stress(P) ≡ P`) | Yes — all of it | 1 |
| B | Steady state | Invariant (live-count converges) | Partly | 2 |
| C | Re-entrancy census | Pinned diagnostic, enumerated | New fixtures | 3 |
| D | Future conformance | Generated cross-product + algebraic law | New, generated | 4 |
| E | Schedule trace | Pinned trace (characterization) | Instruments corpus | 5 |
| F | Invariant fuzz | Invariant-only, no expected output | Generated | 6 |

**Explicitly out of scope**, with reasons:

- **ThreadSanitizer / race detection** — Phalcom is single-threaded by
  construction (ADR-0030). There is no race to detect. Revisit only if
  preemption or real threads are ever admitted.
- **Miri beyond a token lane** — ADR-0009 puts the object graph behind a handle
  arena with no `unsafe`. A cheap Miri lane over the arena is worth wiring; a
  broad one has nothing to find.
- **Differential testing against a reference model** — under determinism the
  program *is* its own schedule, so a second implementation buys far less than
  it costs to keep honest. Reconsider only if a fairness policy lands and the
  schedule stops being a pure function of the program.
- **Cancellation, `select`/`race`, fairness policy** — all **OPEN** in the spec
  (`docs/spec/current/concurrency.md` §3, ADR-0030). Unspecified behavior is not
  testable; testing it would only pin an accident. Blocked on an ADR, not on
  test engineering.
- **Generators** — the feature does not exist. ADR-0033 (`CallBlock`
  trampoline) is **Deferred**. What *is* testable today is the guard that stands
  in for it; that is Lane C, not a generator lane.

## Load-bearing assumptions

These lanes are sound only while the following hold. Each is an ADR commitment,
not an implementation detail — if one is ever amended, the lane it supports must
be re-derived, not merely re-run.

1. **GC is behavior-invariant.** ADR-0050 selects a **non-moving** mark-sweep
   collector. Object identity and hashing do not depend on address, so
   collection cannot be observed by a correct program. Lane A *is* this
   assumption. If the rejected moving alternative ever revives, Lane A silently
   becomes wrong rather than red — see [10-gc-stress.md](10-gc-stress.md) §6.
2. **No native fiber stacks.** ADR-0030 keeps frames on the heap, which is what
   makes the *stackful fibers ⊗ moving GC* hazard not arise, and what lets Lane
   A trace a suspended fiber's roots at all.
3. **Cooperative, single-threaded, no preemption.** ADR-0030. Makes every lane
   deterministic and every failure reproducible.
4. **Yielding across a native frame raises.** ADR-0030 §4's restricted
   execution model. Lane C encodes this as a contract — while noting it is a
   snapshot of a *deferred* decision (ADR-0033), not an invariant.

## Command surface

Target shape once the lanes land. Names are provisional; each lane's doc owns
its own final spelling.

```sh
cargo test -p phalcom-core                       # default green gate (fast lanes)
PHALCOM_GC_STRESS=1 cargo test -p phalcom-core --test lang    # Lane A
cargo test -p phalcom-core --test steady_state   # Lane B
cargo test -p phalcom-core --test lang reentrancy # Lane C
cargo test -p phalcom-core --test future_conformance # Lane D
```

Lanes A and F are **not** part of the default green gate — A for runtime cost,
F for nondeterminism in what it generates. Both run in CI on their own schedule.
See each lane's §5 for the gating rule.
