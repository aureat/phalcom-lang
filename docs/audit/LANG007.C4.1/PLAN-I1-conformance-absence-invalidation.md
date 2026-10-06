# LANG007 I1 Conformance Absence Invalidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Certify and, if necessary, repair incremental invalidation when a conformance changes from absent to present or present to absent.

**Architecture:** Test the existing exact dependency graph first. If tests fail, add a dependency on the conformance-search authority or prove an existing aggregate key is sufficient. Do not restore broad workspace fingerprints as a shortcut.

**Tech Stack:** Rust nightly, `phalcom-semantic` incremental tests, cross-module workspace fixtures.

**Issue brief:** `docs/audit/LANG007.C4.1/AUD-I1-conformance-absence-invalidation.md`

---

### Task 1: Build a two-module fixture

**Files:**
- Modify: `phalcom-semantic/tests/semantic/incremental/traits.rs`

- [ ] Add provider and consumer modules with stable linked-program topology.
- [ ] Keep the consumer source and public signature unchanged across revisions.
- [ ] Ensure the conformance is the only semantic change in each transition.

### Task 2: Test absent-to-added

- [ ] Analyze revision N without the provider conformance and assert missing dispatch/evidence.
- [ ] Add only the provider conformance and update the same session.
- [ ] Assert consumer analysis, diagnostics, dispatch, and evidence refresh.
- [ ] Analyze the final workspace cold and compare callable analysis, selected target, evidence/completeness, diagnostics, and public signature.
- [ ] Assert current dependencies reflect the new result.

### Task 3: Test present-to-removed

- [ ] Start with a positive provider conformance and consumer.
- [ ] Remove only the provider conformance.
- [ ] Assert old positive evidence and dispatch are retracted.
- [ ] Compare the incremental result with a cold no-conformance workspace.

### Task 4: Repair only the owning boundary if tests fail

**Files:**
- Modify: `phalcom-semantic/src/db/query.rs` or `phalcom-semantic/src/session.rs` only after the failing test identifies the missing edge

- [ ] Trace the missing dependency from the negative query to its search authority.
- [ ] Add the smallest exact dependency or invalidation root covering addition and removal.
- [ ] Keep positive `ConformanceDispatch` dependencies and prior-client retraction intact.
- [ ] Re-run both regressions after repair.

### Task 5: Record evidence

- [ ] Record whether the architecture passed unchanged or required production changes.
- [ ] Record exact selectors and cold-parity assertions.
- [ ] If one serious semantic repair does not resolve the issue, stop and escalate rather than broadening fingerprints speculatively.

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::traits
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::associated_types
```
