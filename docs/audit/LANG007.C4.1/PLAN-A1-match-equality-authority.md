# LANG007 A1 Match Equality Authority Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Make match coverage consume canonical generic evidence instead of maintaining a private equality-normalization solver.

**Architecture:** Add one shared evidence-backed refinement operation at the generic-evidence boundary. Match synthesis consumes its typed result and preserves parameter ownership, local rigid isolation, projection outcomes, and blocked states.

**Tech Stack:** Rust nightly, `phalcom-semantic`, semantic integration tests.

**Issue brief:** `docs/audit/LANG007.C4.1/AUD-A1-match-equality-authority.md`

---

### Task 1: Establish failing parity tests

**Files:**
- Modify: `phalcom-semantic/tests/semantic/adts/matching/exhaustiveness.rs`
- Modify: `phalcom-semantic/tests/semantic/adts/matching/gadt_refinement.rs`

- [ ] Add direct, reversed, and chained equality fixtures whose formal match resolution must use the same refined root as ordinary generic checking.
- [ ] Add a negative concrete-mismatch fixture that remains `PatternUsefulness::Impossible`.
- [ ] Add a projection or underconstrained fixture that remains blocked/non-proven rather than silently normalized.
- [ ] Run the tests and record the pre-fix failure or missing observation.

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::matching::exhaustiveness
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::matching::gadt_refinement
```

### Task 2: Define and test the shared refinement result

**Files:**
- Modify: `phalcom-semantic/src/generic_evidence.rs`
- Inspect: `phalcom-semantic/src/types/projection.rs`
- Inspect: `phalcom-semantic/src/types/outcome.rs`

- [ ] Define a result distinguishing unchanged/known, proven refined, and blocked/unavailable refinement.
- [ ] Reuse `GenericEquivalenceProof`, projection outcomes, and `BlockReason`; do not add a second equivalence graph.
- [ ] Traverse equivalent-term components deterministically.
- [ ] Preserve `TypeParameterId` ownership; do not materialize inference variables or local rigids.
- [ ] Add unit tests for direct, reversed, transitive, projection, reordered, and blocked evidence.

### Task 3: Replace match-local normalization

**Files:**
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Modify: `phalcom-semantic/src/checker/context.rs` only if required by the shared query

- [ ] Replace `apply_ambient_equalities` in `synthesize_match_expr` with the shared operation.
- [ ] Remove the private helper after all callers are gone.
- [ ] On blocked refinement, retain a sound root and mark coverage blocked through its existing API.
- [ ] Keep `ambient_constraints` for unrelated APIs until a separate migration proves it can be removed.

### Task 4: Verify and document

- [ ] Run focused A1 tests and `cargo check -p phalcom-semantic`.
- [ ] Confirm no private equality solver remains in match synthesis.
- [ ] Review the diff for changes to `Self`, local rigid, or dispatch semantics.
- [ ] If one serious semantic correction does not resolve the issue, stop and escalate under the repository protocol.

```sh
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::matching::exhaustiveness
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::matching::gadt_refinement
```
