# Fresh-Session Handoff — Phalcom LSP Module Architecture (C1 Complete → C2 Next)

You are taking over the implementation of the **Phalcom LSP Module Architecture** roadmap.
Checkpoints **C0** and **C1** are complete, fully verified, and passing all negative/evidence gates.
Your immediate mission is to execute **Checkpoint C2 (Incremental Persistent Module Workspace, Tasks 9–13)**.

---

## 1. Primary Objectives and Grounding Documents

- **Authoritative Patch Plan**: `docs/impl/lsp/module/phalcom-lsp-module-architecture-patch-grade-implementation-plan.md`
- **Architecture Specification**: `docs/impl/lsp/module/lsp-module-architecture.md`
- **Live Implementation State**: `docs/work/modules/lsp-module-architecture-implementation-state.md`
- **Session Roadmap**: `/Users/altunhasanli/.gemini/antigravity/brain/f00f7b80-dbe7-45b2-9ce5-3a229f721c01/implementation_plan.md`

### Core Constraints & House Rules
1. **Sandboxed Git**: ALWAYS prefix `git` commands with `GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null`.
2. **Preserve Concurrent Work**: Do NOT revert or touch uncommitted working tree edits in:
   - `phalcom-semantic/src/checker/*`
   - `phalcom-core/tests/core/typing_integration/*`
   - `phalcom-ast/src/*`
   - `docs/agents/prompt--supervisor-implementer-workflow.md`
3. **Communication Style**: Terse smart-caveman (`AGENTS.md`). Drop articles/fluff, keep exact technical terms: `[thing] [action] [reason]. [next step].`
4. **Semantics Rule**: Never weaken type assertions or bypass package semantics. Only `package.ph` establishes a package identity; plain sibling files remain standalone modules.

---

## 2. Completed Work & Verified Evidence

### Pre-requisite / Checkpoint C0 (Resolved Baseline Incident)
- **Problem**: Pre-existing quadratic hang in `SourceSemanticIndex::attach_formal_analysis` triggered when running `modules_universe::standalone_package_has_no_project_binding`.
- **Fix**:
  1. Added `baseline_occurrences: Arc<OccurrenceIndex>` in `ModuleSourceIndex` (`phalcom-semantic/src/source_index/mod.rs`), preventing duplicate compounding occurrences across callable attachments.
  2. Deferred full workspace `rebuild_target_occurrences()` to a single batch call after callable analysis loop in `phalcom-semantic/src/session.rs`.
- **Evidence**: `RUSTFLAGS='' cargo test -p phalcom-core --test core modules_universe` passes all 16 tests in ~15s.

### Checkpoint C1: Canonical Topology and Module-Owned Fingerprints (Tasks 5–8)
- **Task 5 (Module-Owned Fingerprints)**:
  - Created `phalcom-modules/src/fingerprint.rs` defining canonical `InterfaceFingerprint(u64)` and `LinkedInterfaceFingerprint(u64)`.
  - Canonicalized `hash_unlinked_interface`, `hash_linked_interface`, `interface_fingerprint`, `linked_interface_fingerprint`, and input fingerprint variants.
  - Replaced duplicate hashing in `phalcom-semantic/src/db/fingerprint.rs` with delegation to `phalcom_modules::fingerprint`.
- **Task 6 (Canonical Topology & Fingerprint)**:
  - Created `phalcom-modules/src/topology.rs` defining `ModuleTopology`, `TopologyNode`, and `TopologyFingerprint(u64)`.
  - Hashing captures project boundaries, module existence, module kinds, and package exposure edges; excludes source method bodies, local expressions, and comments.
  - Added cycle detection (`detect_cycle`) and descendant hierarchy traversal (`descendants`).
- **Task 7 (Reusable Import Resolution Product)**:
  - In `phalcom-modules/src/resolver.rs`, introduced `ImportPathIdentity`, `ResolutionFingerprint(u64)`, `ResolutionTopologyDependencies`, and `ImportResolutionProduct`.
  - Added `ModuleResolver::resolve_import_product`.
- **Task 8 (Accelerated Query Facade)**:
  - In `phalcom-modules/src/query.rs`, enhanced `ModuleQueryFacade` with `.with_topology()` and `.with_reverse_imports()`.
  - Accelerated `module_children`, `external_import_children`, `module_for_source`, and `reverse_importers` using topology indexes.

### Evidence Matrix

| Gate | Command | Result |
|---|---|---|
| Core Modules Universe | `cargo test -p phalcom-core --test core modules_universe` | PASS (16/16) |
| Module Topology Suite | `cargo test -p phalcom-modules --test topology` | PASS (6/6) |
| Module Query Suite | `cargo test -p phalcom-modules --test query` | PASS (3/3) |
| Full Modules Crate | `cargo test -p phalcom-modules` | PASS (81/81) |
| Semantic Fingerprints | `cargo test -p phalcom-semantic --test semantic incremental::fingerprints` | PASS (32/32) |
| Negative Gate | `rg 'fn hash_import_path\|fn hash_metadata' phalcom-semantic/src/db/fingerprint.rs` | PASS (0 hits) |
| Whitespace Check | `git diff --check` | PASS (clean) |

---

## 3. Working Tree File Map

### Files Created in C0 / C1
- `phalcom-modules/src/fingerprint.rs` — Canonical interface fingerprints and interface hashing.
- `phalcom-modules/src/topology.rs` — `ModuleTopology`, `TopologyFingerprint`, `TopologyNode`.
- `phalcom-modules/tests/topology.rs` — Unit tests for topology fingerprints, exposure changes, cycle detection, resolution products.

### Files Modified in C0 / C1
- `phalcom-modules/src/lib.rs` — Re-exports for fingerprint, topology, and resolution product types.
- `phalcom-modules/src/error.rs` — Added `Eq` derives to `ModuleResolutionError`, `SourceError`, `ModuleLoadError`, `InterfaceError`.
- `phalcom-modules/src/interface.rs` — Added `.fingerprint()` methods on `UnlinkedModuleInterface` and `LinkedModuleInterface`.
- `phalcom-modules/src/query.rs` — Integrated topology and reverse import acceleration into `ModuleQueryFacade`.
- `phalcom-modules/src/resolver.rs` — Added `ImportResolutionProduct` and `resolve_import_product`.
- `phalcom-modules/tests/query.rs` — Added test for facade with topology and reverse imports.
- `phalcom-semantic/src/db/fingerprint.rs` — Delegated interface fingerprints to `phalcom-modules`.
- `phalcom-semantic/src/source_index/mod.rs` & `occurrence.rs` — `baseline_occurrences` and batch reverse indexing.
- `phalcom-semantic/src/session.rs` — Batch `rebuild_target_occurrences()`.
- `docs/work/modules/lsp-module-architecture-implementation-state.md` — Updated ledger and decisions D-06 through D-09.

---

## 4. Next Mission: Checkpoint C2 (Incremental Persistent Module Workspace)

Reference: `docs/impl/lsp/module/phalcom-lsp-module-architecture-patch-grade-implementation-plan.md` lines 1350–1670.

### Tasks to Implement

1. **Task 9: Delta Transaction in `WorkspaceModuleSession`**:
   - File: `phalcom-modules/src/session.rs`.
   - Implement `WorkspaceModuleTransaction` / delta updates instead of rebuilding the entire universe and session maps from scratch on each mutation.
   - Batch mutations into atomic commits.
2. **Task 10: Invalidation Domains in `FilesystemSourceProvider`**:
   - File: `phalcom-modules/src/source.rs`.
   - Separate content invalidation (source body modified) from topology invalidation (file created, deleted, or renamed; `package.ph` created or removed).
3. **Task 11: Retain `InterfaceProduct` and Provide to Semantic Workspace**:
   - Files: `phalcom-modules/src/session.rs`, `phalcom-semantic/src/session.rs`.
   - Do not re-extract unlinked interfaces for unmodified sources across transaction steps.
4. **Task 12: Reverse Dependency Invalidation & Product Reuse**:
   - Files: `phalcom-modules/src/session.rs`, `phalcom-modules/src/resolver.rs`.
   - Maintain a reverse dependency graph of import resolutions.
   - When a module's `InterfaceFingerprint` is unchanged after an edit, downstream modules' import resolution products and linked products are retained and reused.
5. **Task 13: C2 Verification Tests**:
   - Add work-count assertion tests in `phalcom-modules/tests/workspace_session.rs`: assert that body-only edits perform 1 parse and 0 downstream relinks.

---

## 5. Verification Commands for Handoff Check

Run these to verify baseline health before touching any files:

```bash
# 1. Check modules crate
RUSTFLAGS='' cargo test -p phalcom-modules

# 2. Check semantic incremental fingerprints
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::fingerprints

# 3. Check core modules universe
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_universe

# 4. Verify no git whitespace errors
GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null git diff --check
```
