# Spec 01 Compiler-Owned Typing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** Implement Spec 01's foundational semantic infrastructure while preserving current two-axis typing and Phalcom runtime behavior.

**Architecture:** Extend `phalcom-semantic` with validated store-relative type handles, explicit relation/query terminal states, source-owned diagnostics, and immutable generation-tagged snapshots. Add a compiler-owned staged semantic database with dependency recording, deterministic invalidation/SCC scheduling, cancellation, and budgets; keep the current cold workspace analysis as its compatibility backend during migration. Make compiler/LSP consumers use the shared formal snapshot while retaining advisory `ValueShape` separately.

**Tech Stack:** Rust 2024 workspace, `phalcom-semantic`, `phalcom-modules`, `phalcom-core`, `phalcom-lsp`, `Arc`-owned immutable products, deterministic `BTreeMap`/worklists, focused Rust integration tests.

---

### Task 1: Store-relative identities and proper-type enforcement

**Files:**
- Modify: `phalcom-semantic/src/types/id.rs`, `types/store.rs`, `types/evidence.rs`, `types/mod.rs`
- Test: `phalcom-semantic/tests/kinds.rs`, `types/store.rs` unit tests, new cross-store tests

- [ ] Add `TypeStoreId`, `ProperTypeId`, and checked `TypeStore::proper_type` validation without changing dense internal `TypeId` representation.
- [ ] Make tuple, record, callable, union, and `TypeKnowledge::known` construction require validated proper types; preserve trusted internal constructors only as `pub(crate)`.
- [ ] Add negative tests proving wrong-kind and cross-store handles cannot enter published knowledge.

### Task 2: Explicit bounded relation outcomes

**Files:**
- Create: `phalcom-semantic/src/types/outcome.rs`
- Modify: `types/relation.rs`, `types/constraint.rs`, `types/mod.rs`, checker call sites
- Test: `phalcom-semantic/tests/checker.rs`, `types/relation.rs` tests, cycle/budget/cancellation tests

- [ ] Define query-specific outcomes distinguishing proven, refuted, dynamic boundary, blocked, cancelled, budget exceeded, and internal failure.
- [ ] Implement iterative/memoized relation evaluation with store validation, pair/step budgets, cancellation checks, deterministic evidence/refutation, and explicit recursive-cycle policy.
- [ ] Migrate production assignability/checker paths away from boolean/coarse success; keep compatibility only where callers still require local adaptation.
- [ ] Add deep, cyclic, dynamic, cancelled, and budget-exhausted negative coverage.

### Task 3: Stable project/snapshot identity and source-owned diagnostics

**Files:**
- Modify: `phalcom-semantic/src/identity.rs`, `diagnostic.rs`, `snapshot.rs`, `lib.rs`
- Modify: `phalcom-modules/src/identity.rs`, `project.rs`, `lib.rs`
- Modify: `phalcom-lsp/src/diagnostics.rs`
- Test: semantic and LSP cross-module diagnostic tests

- [ ] Add stable project/module keys, semantic revisions, snapshot/store IDs, and checked snapshot-relative type references.
- [ ] Convert primary and related diagnostic spans to owned module/source identities; add stable project, link, blocked, cancellation, budget, internal, cycle, and dynamic-boundary codes.
- [ ] Make snapshot products private behind coherent getters and preserve structural deterministic output.

### Task 4: Compiler-owned staged semantic database

**Files:**
- Create: `phalcom-semantic/src/db/{mod,key,state,dependency,scheduler,budget,metrics}.rs`
- Modify: `phalcom-semantic/src/lib.rs`, `workspace.rs`, `invalidation.rs`, `snapshot.rs`
- Test: new `phalcom-semantic/tests/db.rs` and incremental differential fixtures

- [ ] Add input/query tables, query states, fingerprints, dependency recorder, reverse index, deterministic scheduler, cancellation, budgets, and metrics.
- [ ] Wrap existing workspace phases as staged products: parse, interface, linked interface, declaration shell, semantic component, body, diagnostics, metadata.
- [ ] Publish immutable complete/partial snapshots atomically; never publish cancelled, budget-exhausted, or internal-failure products as success.
- [ ] Add cold-vs-incremental equivalence, no-op hit, body-only invalidation, interface reverse-closure, stale-result, and deterministic publication tests.

### Task 5: Partial workspace failures and runtime-cycle correctness

**Files:**
- Modify: `phalcom-semantic/src/workspace.rs`, `snapshot.rs`, `diagnostic.rs`
- Modify: `phalcom-modules/src/{linker,graph,error}.rs` only where source ownership is missing
- Modify: `phalcom-lsp/src/analysis_service.rs`
- Test: project-load/interface/link/runtime-cycle and cancellation fixtures

- [ ] Preserve project, parse, interface, import, link, and runtime-cycle failures as module states and source-owned diagnostics.
- [ ] Remove the LSP sorted-order fallback and prevent invalid runtime programs from reaching formal compilation.
- [ ] Publish valid independent modules as explicitly partial while strict entry reachability rejects blocked/invalid closure.

### Task 6: Formal consumer convergence and observability

**Files:**
- Modify: `phalcom-core/src/modules/compile.rs`, REPL session entry points discovered during implementation
- Modify: `phalcom-lsp/src/{analysis_service,backend,diagnostics}.rs` and `semantic/{engine,snapshot,mod}.rs`
- Test: compiler/LSP/REPL diagnostic convergence and performance/invalidation tests

- [ ] Route compiler and CLI analysis through `SemanticDb` snapshots and preserve one compile gate.
- [ ] Feed LSP source overlays into the compiler-owned database; retain advisory semantic engine facts as a separate layer.
- [ ] Add query/invalidation counters and cold/warm/edit/revert observations; report deferred compaction/parallelism or Spec 01.5 semantics explicitly.

### Verification

- [ ] Run focused semantic/module/core/LSP tests after each cohesive task.
- [ ] Run `cargo fmt --all -- --check`, `git diff --check`, relevant workspace tests, and broad feasible regression suites.
- [ ] Run `graphify update .` after code changes.
- [ ] Report passing, baseline/unrelated, deferred, and unverified scope separately.
