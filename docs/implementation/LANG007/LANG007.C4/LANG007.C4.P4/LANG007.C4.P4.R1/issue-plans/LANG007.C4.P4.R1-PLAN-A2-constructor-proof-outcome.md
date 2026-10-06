# LANG007 A2 Constructor Proof Outcome Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Preserve the distinction between a refuted constructor and a constructor whose local proof is blocked.

**Architecture:** Introduce typed outcomes for local case proof and variant opening. Propagate `Refuted` to constructor omission and `Blocked` to `DomainDecomposition::Blocked`; retain the existing coverage engine's blocked propagation.

**Tech Stack:** Rust nightly, `phalcom-semantic`, ADT/match semantic tests, compiler lowering tests.

**Issue brief:** `../issue-audits/LANG007.C4.P4.R1-AUD-A2-constructor-proof-outcome.md`

---

### Task 1: Add failing outcome tests

**Files:**
- Modify: `phalcom-semantic/tests/semantic/adts/matching/gadt_refinement.rs`
- Modify: `phalcom-semantic/tests/semantic/adts/matching/exhaustiveness.rs`

- [ ] Assert a concrete incompatible GADT case is explicitly refuted.
- [ ] Assert a supported nested local-rigid case is reachable.
- [ ] Add a budget/cancellation or unsupported-local-shape fixture that must produce `ExhaustivenessResult::Blocked` and `MatchAnalysisBlocked`.
- [ ] Run the tests before changing production code and record the collapse.

### Task 2: Type local proof outcomes

**Files:**
- Modify: `phalcom-semantic/src/checker/gadt_proof.rs`
- Inspect: `phalcom-semantic/src/types/outcome.rs`

- [ ] Replace `LocalCaseProof = Option<...>` with a result carrying success bindings/equalities, `Refuted`, and `Blocked(BlockReason)`.
- [ ] Change `unify_local_types` and helpers to preserve the distinction.
- [ ] Classify explicit contradictions and occurs-check failures as `Refuted`.
- [ ] Classify unsupported representations, unresolved projections, cancellation, budgets, and recovery uncertainty as `Blocked`.
- [ ] Keep rigid-scope and materialization invariants unchanged.

### Task 3: Type variant opening and decomposition

**Files:**
- Modify: `phalcom-semantic/src/checker/coverage/domain.rs`
- Modify: `phalcom-semantic/src/checker/coverage/mod.rs`

- [ ] Add `VariantOpenOutcome::{Reachable, Refuted, Blocked}`.
- [ ] Map canonical `GadtProofResult::Refuted` to `Refuted`.
- [ ] Map local proof `Blocked` to `Blocked`; remove `?`-based loss of information.
- [ ] Return `Empty` only for refuted exact cases.
- [ ] In general enums, skip refuted variants, retain reachable variants, and return blocked if a potentially applicable variant cannot be classified.

### Task 4: Update consumers and verify

**Files:**
- Modify: `phalcom-semantic/src/checker/pattern.rs`
- Modify: `phalcom-semantic/src/checker/coverage/usefulness.rs` only if required

- [ ] Preserve blocked state in contextual singleton and variant-pattern resolution.
- [ ] Preserve `PatternUsefulness::Impossible` only for genuine refutation.
- [ ] Run semantic coverage tests and the directly affected compiler lowering test.
- [ ] Review every `open_variant_case` caller before completion.

```sh
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::matching::gadt_refinement
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::matching::exhaustiveness
RUSTFLAGS='' cargo test -p phalcom-semantic --lib checker::coverage
```
