# LANG007 T2 Public Signature Coordinate Purity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Lock down that published covering-inherent callable signatures contain no current-impl-owned type parameters in any recursive type position.

**Architecture:** Add a read-only recursive test walker over canonical `TypeStore` products and generic constraints. Test publication separately from source-body inverse covering.

**Tech Stack:** Rust nightly, `phalcom-semantic` semantic integration tests.

**Issue brief:** `../issue-audits/LANG007.C4.P4.R1-AUD-T2-public-signature-coordinate-purity.md`

---

### Task 1: Implement the recursive ownership walker

**Files:**
- Modify: `phalcom-semantic/tests/semantic/impls/signatures.rs`

- [ ] Return an offending type path and `TypeParameterId` for the current `ImplId`.
- [ ] Recurse through parameters, result, applied origins/arguments, projections, unions, exact cases, tuples, records/rows/tails, packs, callables, and pack forms.
- [ ] Walk generic constraints and trait-reference arguments.
- [ ] Preserve static `RecordRow` arguments and row projection tails.
- [ ] Make paths deterministic and actionable.

### Task 2: Add the covering publication matrix

**Files:**
- Modify: `phalcom-semantic/tests/semantic/impls/signatures.rs`

- [ ] Construct a covering impl with distinct declaration and impl binders and a permutation target.
- [ ] Exercise every currently supported nested carrier form available through live signature APIs.
- [ ] Assert no published type contains `TypeParameterOwner::Impl(current_impl)`.
- [ ] Assert declaration-owned parameters remain in expected positions.
- [ ] Assert callable-local parameters remain distinct.

### Task 3: Add a negative walker test and verify

- [ ] Construct a test-only malformed signature containing an impl parameter in a nested carrier.
- [ ] Assert the walker reports the exact path.
- [ ] Run signature and body groups and classify unsupported carriers instead of inventing syntax.

```sh
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic impls::signatures
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic impls::bodies
```
