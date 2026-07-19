# 15 — Lane F: Invariant Fuzz

> **Oracle:** invariant-only — no expected output exists or is needed.
> **Closes:** [G9](02-coverage-ledger.md#2-what-that-leaves-uncovered).
> **Status:** specified, not built. Lowest priority; build after A–D.

## 1. The claim

Fifty hand-written concurrency fixtures cover the program shapes their authors
thought of. The shapes that break runtimes are the ones nobody pictures: fiber
nesting six deep under allocation churn, a fiber scheduled from inside a drain
that schedules another, an abort mid-`then`-chain, a yield between two
allocations that straddle a safepoint.

Generation reaches those. The obstacle is the oracle — nobody can hand-verify a
thousand generated programs. The answer, per
[01 §"generated input demands an oracle of kind 2 or 3"](01-oracle-model.md#corollary-what-to-do-with-generated-inputs),
is to assert **invariants only** and never expected output. That makes the lane
cheap to build, because the entire cost of generative testing is normally the
oracle.

## 2. The grammar

A small program generator over concurrency-relevant operations:

```
prog  := stmt*
stmt  := spawn | resume | yield | schedule | drain | alloc | settle | await
       | abort | try_block | nest(prog)
```

Constraints keeping generated programs meaningful rather than merely legal:

- **Well-formed by construction.** Generate against a model of what is legal at
  each point (do not resume a `Done` fiber, do not yield from the root) so the
  bulk of runs exercise the happy path rather than re-deriving the error surface
  the corpus already pins. Reserve a tunable fraction — say 10% — for
  deliberately illegal operations, whose invariant is "raises a defined error,
  never panics."
- **Bounded depth and length.** Nesting ≤ 6, ≤ 200 statements. Unbounded
  generation finds stack overflow, which is a known and boring answer.
- **Deterministic from a seed**, printed on failure. Determinism is inherited
  from ADR-0030 — a seed reproduces the failure exactly, with no interleaving
  luck involved. This is the property that makes fuzzing a cooperative runtime
  far more tractable than fuzzing a preemptive one.
- **Allocation churn interleaved** so generated programs cross GC thresholds.
  Run the lane with `PHALCOM_GC_STRESS` on a fraction of seeds; that composition
  is where the deepest bugs are, since it crosses fiber state with collection
  timing.

## 3. The invariants

No expected output. Every assertion is program-independent:

| # | Invariant |
|---|---|
| F1 | **No Rust panic.** Exit code ≠ 101, no `panicked at` on stderr. Any panic is a bug regardless of the program. |
| F2 | **No stale handle resolution.** A swept handle must resolve to nothing, never to a different object. |
| F3 | **`Universe::verify_invariants` holds** after the run and after any collection during it. |
| F4 | **Resumer chain is acyclic**, and every suspended fiber's resumer is live. |
| F5 | **`native_reentry_depth == 0`** at top level ([C-INV-2](12-reentrancy-census.md#2-c-inv-2--balance)). |
| F6 | **Ready queue contains no `Done`/`Failed` fiber** after a drain. |
| F7 | **Every error is a defined Phalcom error**, never an unwrapped Rust error or an unhandled `todo!()`. |
| F8 | **Termination** within a step budget — no infinite drain loop. Budget exceeded is a finding, not a timeout to raise. |

F1 and F7 together are most of the value at the lowest cost. They need no model
of the program's intent whatsoever, which is exactly why this lane is cheap.

## 4. Relationship to the other lanes

Lane F is the **residual** lane: it looks for what the targeted lanes miss. It
is deliberately last, because a bug findable by Lane A or C should be found
there — with a named fixture and a clear diagnosis — rather than as a
seed number and a stack trace.

A useful discipline: when Lane F finds something, **shrink it, then promote it
to a named fixture** in the appropriate lane. The fuzz lane is a discovery
mechanism, not a home. A corpus of retained fuzz seeds is a worse version of the
corpus that already exists.

## 5. Cost and gating

- Authoring: moderate — the generator plus the invariant harness.
- Runtime: tunable by seed count.
- **Not in the default green gate.** Nightly with a fixed seed range, plus a
  longer-running job on a schedule. A green gate that generates new inputs per
  run makes CI failures non-reproducible from the commit alone, which is the
  thing everyone hates about fuzz-in-CI.

Regression seeds — those that once found a bug — are pinned in a committed list
and *do* run in the green gate, since they are deterministic and cheap. This is
the standard split and it is the only part of the lane that gates merges.

## 6. Preclusion

- **Invariant-only oracles cannot catch wrong answers**, only broken states. A
  generated program that computes the wrong number silently passes. That is
  accepted: correctness of results is Lanes A/D's job, and trying to give Lane F
  a value oracle would require the reference model rejected in
  [README §Lane inventory](README.md#lane-inventory).
- **The generator encodes what is legal**, so it inherits every open question.
  It cannot generate cancellation or `select`/`race` (both **OPEN**), and when
  those land the grammar needs extending in the same change — or the lane
  silently stops covering the newest, least-tested surface, which is the worst
  possible failure mode for a residual lane.
- **Seed-reproducibility depends on determinism.** ADR-0030's cooperative,
  single-threaded model is what makes a seed a complete reproducer. If
  preemption is ever admitted, seeds stop reproducing, and this lane needs a
  recorded-schedule mechanism to stay useful at all.
