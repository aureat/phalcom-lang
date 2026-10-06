# LANG007 T3 Nested Covering Source Certification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Certify supported nested `Self` and generic covering forms through parsing, publication, inverse body materialization, and body checking.

**Architecture:** Add a small source-level matrix in existing impl-body tests. Each fixture isolates one carrier and asserts declaration-canonical publication plus source-relative body semantics.

**Tech Stack:** Rust nightly, `phalcom-semantic` source fixtures and semantic products.

**Issue brief:** `docs/audit/LANG007.C4.1/AUD-T3-nested-covering-source-certification.md`

---

### Task 1: Inventory supported syntax

**Files:**
- Inspect: `phalcom-ast/src/ast/`
- Inspect: `phalcom-semantic/src/types/annotation.rs`
- Inspect: `phalcom-semantic/tests/semantic/impls/bodies.rs`

- [ ] List exact syntax supported for nested applications, tuples, records, callable types, projections, packs, and rows.
- [ ] Classify each requested carrier as supported, partial, or unavailable.
- [ ] Do not add fixtures for unsupported language syntax.

### Task 2: Add positive nested source fixtures

**Files:**
- Modify: `phalcom-semantic/tests/semantic/impls/bodies.rs`

- [ ] Add one nested nominal application case.
- [ ] Add one tuple or record carrier case.
- [ ] Add one callable or projection carrier if supported.
- [ ] Assert parse success, diagnostics, exact callable identity, public coordinates, and body expression types.
- [ ] Use distinct impl and callable binder identities.

### Task 3: Add permutation and negative binder coverage

- [ ] Compare identity and permuted covering targets with equivalent body structure.
- [ ] Assert the public contract and body context follow their separate coordinate systems.
- [ ] Retain a callable-local `U` negative case that cannot unify with impl `T`.
- [ ] Reuse `(ImplId, source_member_index)` provenance assertions.

### Task 4: Verify and record unsupported forms

- [ ] Compare body products with a cold analysis for a context-only target permutation.
- [ ] Record unsupported carriers and their owning subsystem in the issue brief.
- [ ] Run the relevant semantic groups.

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic impls::bodies
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic impls::signatures
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic impls::queries
```
