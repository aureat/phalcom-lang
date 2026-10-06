# LANG007 V1 Broad Certification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Establish truthful current-head verification evidence for C4.1 after focused semantic obligations are complete.

**Architecture:** Run narrow gates first, then directly affected compiler/core lanes, then CI-equivalent workspace gates. Record each result and classify baseline failures without repairing unrelated work.

**Tech Stack:** Pinned nightly `nightly-2026-07-10`, Cargo, `phalcom-semantic`, `phalcom-core`, CI lanes in `.github/workflows/ci.yml`.

**Issue brief:** `../issue-audits/LANG007.C4.P4.R1-AUD-V1-broad-certification.md`

---

### Task 1: Confirm prerequisites

- [ ] Confirm A1 and A2 are complete or explicitly deferred with an architectural decision.
- [ ] Confirm I1, T2, and T3 regressions have run or are classified.
- [ ] Confirm the working tree contains only authorized changes.

### Task 2: Run focused semantic gates

```sh
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::matching::gadt_refinement
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::matching::exhaustiveness
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic impls::bodies
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic impls::signatures
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::traits
```

- [ ] Confirm each selector ran at least one test.
- [ ] Record exact counts and any ignored tests.

### Task 3: Run directly affected compiler/core gates

- [ ] Locate the existing compiler match-lowering and semantic body projection tests.
- [ ] Run the smallest affected core targets and compare blocked/proven match behavior.
- [ ] Record any unrelated failures separately.

### Task 4: Run current-head workspace certification

```sh
cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
```

- [ ] Run extension-host and Miri lanes when the environment supports them.
- [ ] Report incomplete lanes as incomplete; do not call them passed.
- [ ] Classify baseline formatting, test, and lint failures under the repository workflow.

### Task 5: Update durable records

**Files:**
- Modify: `../LANG007.C4.P4.R1-walkthrough.md`
- Modify: `../LANG007.C4.P4.R1-handoff.md`

- [ ] Record focused and broad commands with results.
- [ ] Preserve `FOCUSED_TESTED` versus `RELEASE_COMPLETE` distinction.
- [ ] Record residual baseline failures and the next action.
