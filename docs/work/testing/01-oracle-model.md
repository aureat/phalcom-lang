# 01 — The Oracle Model

> How a lane decides whether a run passed. Read this before writing any lane;
> picking the wrong oracle is the failure mode that produces suites which are
> large, green, and blind.

## The problem

A test needs something to compare against. The acceptance corpus uses one
oracle — a pinned `.expected` stdout — and applies it universally. That works
because for syntax, dispatch, and arithmetic, *the output is the behavior*.

For the runtime subsystems in this directory, the output is a **projection** of
the behavior, and the interesting failures live in the kernel of that
projection. A missed GC root, a leaked object, a corrupted resumer chain, and a
schedule reordering can all map to identical stdout. Pinning stdout harder does
not help; it is the wrong function.

## The four kinds

### 1. Pinned output (exact-value oracle)

Compare the run's observable output to a hand-verified constant.

- **Requires:** someone knows the right answer and can write it down.
- **Strength:** total. Catches any deviation in what it observes.
- **Blind to:** everything it does not observe. That is the whole problem here.
- **Used by:** the acceptance corpus; Lane C (pinned *diagnostic*, not value).

### 2. Invariant (property of a single run)

Assert a predicate over runtime state that must hold at defined points,
regardless of the program.

- **Requires:** the invariant to be stated precisely and checkable cheaply.
- **Strength:** program-independent — one invariant covers every program that
  runs under it, including programs not yet written.
- **Blind to:** anything outside the predicate. An invariant suite is only as
  good as its census of invariants.
- **Used by:** Lane B (live-count converges), Lane F (no panic, no stale handle,
  resumer chain acyclic), and the existing
  [`tests/invariants.rs`](../../phalcom-core/tests/invariants.rs) /
  `Universe::verify_invariants`.

### 3. Metamorphic relation (property between runs)

Assert a relation between the outputs of two runs that differ in a way that
*must not* matter.

- **Requires:** a transformation the semantics guarantee is invisible.
- **Strength:** **needs no expected output.** This is what makes it the highest
  leverage tool in this directory — an existing corpus of N programs with
  verified outputs becomes N new tests for free, and so does every program added
  later.
- **Blind to:** bugs present identically in both runs. A metamorphic lane
  proves *consistency*, never *correctness*.
- **Used by:** Lane A (`gc_stress(P) ≡ P`), Lane D's algebraic laws.

The relations Phalcom's semantics license:

| Relation | Licensed by |
|---|---|
| `output(P, collect at every safepoint) ≡ output(P, default threshold)` | ADR-0050 non-moving mark-sweep — collection is unobservable |
| `await(async { e }) ≡ e`, for `e` free of scheduler-visible effects | ADR-0030 §1 |
| `f.then(g).then(h) ≡ f.then(x => h(g(x)))` | `Future` functor law, concurrency.md §2 |
| computation split across N yielding fibers ≡ the same computation inline | ADR-0030 §1, cooperative determinism |

### 4. Model / differential (property against a second implementation)

Run the same input through an independent implementation and compare.

- **Requires:** building and maintaining the second implementation.
- **Strength:** catches deep semantic divergence nothing else will.
- **Cost:** the model rots, and a model bug reads as a runtime bug.
- **Verdict for Phalcom: rejected for now.** See
  [README.md](README.md#lane-inventory) — determinism means the program is its
  own schedule, so a model would restate the interpreter rather than check it.

## Decision rule

Apply in order. Take the first that fits.

1. Is the failure **visible in output**, and does someone know the right answer?
   → **Pinned output**. (Corpus, Lane C.)
2. Is there a transformation that **must not change** the output? → **Metamorphic**.
   Prefer this over authoring fixtures — it converts existing coverage into new
   coverage. (Lanes A, D.)
3. Is there a predicate that must hold **regardless of program**? → **Invariant**.
   Prefer this over pinned output for anything generated, since generated inputs
   have no known answer. (Lanes B, F.)
4. Only then consider a **model**, and justify the maintenance cost explicitly.

## Corollary: what to do with generated inputs

Generation and pinned output are incompatible — nobody can hand-verify a
thousand generated programs. This is why Lane F asserts **invariants only**. It
is also why Lane D generates the *inputs* from the state-machine cross-product
but derives the *expectations* from the state machine's own transition rules
rather than from hand-written `.expected` files.

The general rule: **generated input demands an oracle of kind 2 or 3.** A
generative lane that needs hand-written expectations is not generative; it is a
fixture suite with extra steps.

## Anti-patterns

- **Assertion-free smoke lanes.** "It ran without panicking" is an invariant —
  a weak but legitimate one. State it as such. Do not let it masquerade as
  coverage of the feature the program happens to exercise.
- **Pinning a projection and calling it the behavior.** A fiber test whose only
  assertion is stdout tests the prints, not the schedule. If the schedule is
  what matters, observe the schedule (Lane E).
- **Pinning unspecified behavior.** A green test over an OPEN question converts
  an accident into a contract. See [README.md](README.md) doctrine §5 and
  [14-schedule-trace.md](14-schedule-trace.md) §4.
- **Metamorphic relations that are not actually licensed.** Every relation in
  the table above cites the ADR that makes it true. A relation without such a
  citation is a guess, and when it breaks you will not know whether the runtime
  or the relation was wrong.
