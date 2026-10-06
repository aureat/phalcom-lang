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

- [x] Assert a concrete incompatible GADT case is explicitly refuted.
- [x] Assert a supported nested local-rigid case is reachable.
- [x] Add a budget/cancellation or unsupported-local-shape fixture that must produce `ExhaustivenessResult::Blocked` and `MatchAnalysisBlocked`.
- [x] Run the tests before changing production code and record the collapse.

### Task 2: Type local proof outcomes

**Files:**
- Modify: `phalcom-semantic/src/checker/gadt_proof.rs`
- Inspect: `phalcom-semantic/src/types/outcome.rs`

- [x] Replace `LocalCaseProof = Option<...>` with a result carrying success bindings/equalities, `Refuted`, and `Blocked(BlockReason)`.
- [x] Change `unify_local_types` and helpers to preserve the distinction.
- [x] Classify explicit contradictions and occurs-check failures as `Refuted`.
- [x] Classify unsupported representations, unresolved projections, cancellation, budgets, and recovery uncertainty as `Blocked`.
- [x] Keep rigid-scope and materialization invariants unchanged.

### Task 3: Type variant opening and decomposition

**Files:**
- Modify: `phalcom-semantic/src/checker/coverage/domain.rs`
- Modify: `phalcom-semantic/src/checker/coverage/mod.rs`

- [x] Add `VariantOpenOutcome::{Reachable, Refuted, Blocked}`.
- [x] Map canonical `GadtProofResult::Refuted` to `Refuted`.
- [x] Map local proof `Blocked` to `Blocked`; remove `?`-based loss of information.
- [x] Return `Empty` only for refuted exact cases.
- [x] In general enums, skip refuted variants, retain reachable variants, and return blocked if a potentially applicable variant cannot be classified.

### Task 4: Update consumers and verify

**Files:**
- Modify: `phalcom-semantic/src/checker/pattern.rs`
- Modify: `phalcom-semantic/src/checker/coverage/usefulness.rs` only if required

- [x] Preserve blocked state in contextual singleton and variant-pattern resolution.
- [x] Preserve `PatternUsefulness::Impossible` only for genuine refutation.
- [x] Run semantic coverage tests and the directly affected compiler lowering test.
- [x] Review every `open_variant_case` caller before completion.

```sh
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::matching::gadt_refinement
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::matching::exhaustiveness
RUSTFLAGS='' cargo test -p phalcom-semantic --lib checker::coverage
```

## Execution amendment and focused evidence — 2026-10-06

The owning canonical equality helper and `ProofMerge` also collapsed unsupported
terms into contradiction. They now retain typed `Blocked` outcomes, consumed by
coverage search and branch-proof extraction. Unequal opaque local bindings and
failed positional union pairings cannot establish contradiction. The existing
semantic proof owner, canonical IDs, rigid scopes, and materialization boundary
remain authoritative; no architecture redesign or external consultation occurred.

`ResolvedVariantPattern.blocked` retains uncertainty alongside declaration-owned
family/selector identity, including contextual singleton patterns with no proven
candidate. Nested pattern resolution forwards it to the owning coverage engine.
Inhabitation and pattern-space summaries pass the existing `CheckerControl` into
constructor opening; local equality recursion charges shared steps and checks
depth/cancellation. Union decomposition preserves local members and examines all
members so an open member cannot conceal a later blocked member.

The valid nested parent-rigid source regression failed before production changes
(`Proven` instead of `Blocked`) and passed after correction. Concrete refutation
and supported nested parent-rigid reachability are covered by direct proof tests
and the existing matching suite. Cancellation, step/depth budgets, unresolved
projections, occurs checks, and mixed exact/enum/union domains have focused unit
coverage. A projection-in-constructor-result source probe was rejected during
declaration formation (`AnnotationUnresolved` / `EnumVariantResultInvalid`); it
was removed as an invalid A2 fixture. Direct proof tests cover that uncertainty;
this work does not certify or repair that source producer.

All four production `open_variant_case` call sites were reviewed: contextual
singleton and explicit variant resolution, exact-case and general enum domain
decomposition. Required final verification and residual failures are recorded in
[the A2 walkthrough](../LANG007.C4.P4.R1-A2-walkthrough.md). Broad certification
remains V1 work. A2 implementation is committed on main as `1002ab70`, starting
from `4a1cd6c7`; the subsequent user request authorizes commit/push delivery.
