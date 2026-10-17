---
id: PERF003.C2.P7.PATCH5
kind: handoff
program: PERF003
checkpoint: PERF003.C2
parent_phase: PERF003.C2.P7
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
branch: perf/universe-characterization
prepared_against_revision: a134a0650461b2dd6760a8d42a2e944be505849f
execution_agent: Luna
---

# PERF003.C2.P7.PATCH5 Handoff — P7 Allocation & Evidence Hardening

## 1. Operating Environment & Worktree Anchor

- **Repository Root:** `/Users/altunhasanli/dev/phalcom/phalcom`
- **Active Worktree:** `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization`
- **Active Branch:** `perf/universe-characterization`
- **Plan Document:** `docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/PERF003.C2.P7.PATCH5-Luna-Patch-Grade-Implementation-Plan.md`
- **Target Subsystems:**
  - `phalcom-type-meta` (exact layout math and budgeting)
  - `phalcom-modules` (module-world loaded authority & frozen ordered storage)
  - `phalcom-semantic` (TypeStore, StaticValueStore, semantic owner closure, query image)
  - `phalcom-core` (whole-artifact counting allocator discriminator & publication)
  - `packages/slotmap` (compiler parity discriminator)

---

## 2. Executive Summary & Progress

This patch closes exact cumulative allocation accounting for the loaded compiler artifact (`.telosc`), normalizes `packages/slotmap` for source vs artifact compiler parity, and repairs the P7 audit harness.

- **Overall Progress:** ~50% complete.
- **Wave 0 (Proof Harness & Exact Helpers):** **COMPLETE (T0, T1)**
  - Dedicated counting allocator test target created in `phalcom-core/tests/artifact_allocation.rs`.
  - Exact layout helpers added to `phalcom-type-meta/src/allocation.rs` (`boxed_slice`, `arc_slice`, `arc_str`, `vector`, `string`, `boxed_str`, `arc`).
  - Unit tests verified in `phalcom-type-meta/tests/allocation_helpers.rs` (`delta = 0`).
- **Wave 1 (Module-World Authority):** **COMPLETE (T2, T3)**
  - `ModuleAuthorityMap<K, V>` and `ModuleAuthoritySet<T>` implemented in `phalcom-modules/src/authority.rs`.
  - Replaced native maps/sets with frozen budgeted sorted arrays for all loaded module authority.
  - Runtime dependency graph ordering preserved to match linked topological order.
  - Artifact loads module world cleanly without unbudgeted allocations.
- **Wave 2 (Semantic Store & Owner Closure):** **IN PROGRESS (~80% done)**
  - Added `Ord, PartialOrd` derives to all underlying semantic type data and static value structs (`KindData`, `RecordRowData`, `FamilyType`, `ScopedTypeData`, `TypeLambdaData`, `TypeData`, `StaticValueData`, etc.).
  - Implemented `FrozenTypeLambdaPrefix` in `phalcom-semantic/src/types/type_lambda.rs`.
  - Implemented `FrozenTypeStorePrefix` in `phalcom-semantic/src/types/store.rs` along with `TypeStore::from_frozen(...)` and layered lookups.
  - Implemented `TypeStore::import_type_graph_with_allocation_budget` in `phalcom-semantic/src/types/artifact.rs` (T4 Complete).
  - Implemented `FrozenStaticValuePrefix` in `phalcom-semantic/src/static_value/store.rs` and dense boxed slice relocations in `phalcom-semantic/src/static_value/artifact.rs` (T5 Complete).
  - Cleaned up test and fixture consumers; verified `cargo check --lib -p phalcom-type-meta -p phalcom-modules -p phalcom-semantic -p phalcom-core` compiles clean.
  - Verified `cargo test -p phalcom-semantic --test artifact-allocation` passes.
  - Ready to wire `validate_and_import_semantic_world_inner` (T6).

---

## 3. Stable Interface & Takeover Map

The following interfaces and types are established in the live tree and must be inherited:

| Symbol / Type | Location | Purpose |
|---|---|---|
| `AllocationBudget::boxed_slice` | `phalcom-type-meta/src/allocation.rs` | Allocates `Box<[T]>` with exact layout accounting |
| `AllocationBudget::arc_slice` | `phalcom-type-meta/src/allocation.rs` | Allocates `Arc<[T]>` with exact layout accounting |
| `AllocationBudget::arc_str` | `phalcom-type-meta/src/allocation.rs` | Allocates `Arc<str>` with exact layout accounting |
| `ModuleAuthorityMap<K, V>` | `phalcom-modules/src/authority.rs` | Dual-backing map: `Source(BTreeMap)` / `Frozen(Arc<BudgetedSortedMap>)` |
| `ModuleAuthoritySet<T>` | `phalcom-modules/src/authority.rs` | Dual-backing set: `Source(BTreeSet)` / `Frozen(Arc<BudgetedSortedSet>)` |
| `FrozenTypeStorePrefix` | `phalcom-semantic/src/types/store.rs` | Contiguous frozen prefix storing artifact-loaded type graph |
| `TypeStore::from_frozen` | `phalcom-semantic/src/types/store.rs` | Wraps frozen prefix with mutable `im::Vector`/`im::HashMap` overlay for user sessions |
| `FrozenTypeLambdaPrefix` | `phalcom-semantic/src/types/type_lambda.rs` | Contiguous frozen prefix for scoped types and type lambdas |
| `TypeLambdaArena::from_frozen`| `phalcom-semantic/src/types/type_lambda.rs` | Wraps frozen lambda prefix for user sessions |
| `FrozenStaticValuePrefix` | `phalcom-semantic/src/static_value/store.rs` | Contiguous frozen prefix for static values, symbols, and records |
| `StaticValueStore::from_frozen` | `phalcom-semantic/src/static_value/store.rs` | Wraps frozen static value prefix for user sessions |
| `import_static_value_graph_with_allocation_budget` | `phalcom-semantic/src/static_value/artifact.rs` | Imports static value graph with exact layout accounting into dense frozen prefix |
| `load_universe_telosc_from_bytes_with_allocation_summary` | `phalcom-core/src/modules/canonical_artifact.rs` | Measures ledger-requested bytes `N` for discovery |

---

## 4. Invariants That Must NOT Be Redesigned

1. **Exact Allocation Accounting:**
   - Every allocator request owned by the loaded compiler artifact must be debited against the single `AllocationBudget` before the allocator request occurs.
   - Do NOT estimate `BTreeMap`, `BTreeSet`, `im::Vector`, or `im::HashMap` node memory.
   - Do NOT use global allocator interception in production code (only in isolated test harnesses).
2. **Read-Authority Invariance:**
   - Source compilation keeps existing mutable collections (`BTreeMap`, `im::Vector`, etc.).
   - Artifact-loaded authority uses frozen contiguous sorted buffers (`BudgetedSortedMap`, `FrozenTypeStorePrefix`, etc.) behind common read interfaces.
3. **COW / Fork Semantics:**
   - `fork_from_baseline()` and `extends_canonical_prefix()` must remain fully functional. User compilation must extend the canonical loaded prefix without mutation or deep re-interning.
4. **Scope Boundaries:**
   - Do NOT implement P8 executable-Universe / VM bytecode bootstrap work. Stop at the loaded compiler image.

---

## 5. Immediate Next Steps (Priority Order)

### Step 1: Implement T6 (Semantic Owner Closure)
- **File:** `phalcom-semantic/src/artifact/import.rs`
- **Actions:**
  1. Thread `allocations` to `TypeStore::import_type_graph_with_allocation_budget` and `import_static_value_graph_with_allocation_budget` in `validate_and_import_semantic_world_inner`.
  2. Remove `require_semantic_import_allocation_accounting()` placeholder.
  3. Ensure all remaining semantic world tables (callable signatures, trait surfaces, dispatch resolver, field signatures) use frozen budgeted storage.
  4. Run `RUSTFLAGS='' cargo check -p phalcom-semantic --lib`.

### Step 4: Implement Wave 3 (T7 & T8 — Query Image & Publication)
- **Files:** `phalcom-semantic/src/artifact/query.rs`, `phalcom-semantic/src/canonical_image.rs`
- **Actions:**
  1. Thread `AllocationBudget` through `reconstruct_query_image_with_allocation_budget` and `product_and_fingerprints`.
  2. Charge final `Arc<SemanticProduct>`, `ResolvedImportsProduct`, and query entry sorted arrays.
  3. Implement `CanonicalUniverseFormalImage::from_loaded_artifact_with_allocation_budget`.

### Step 5: Wave 4 & 5 (T9–T12 — Verification, Parity & Audit Evidence)
- **Actions:**
  1. Run `cargo test -p phalcom-core --test artifact_allocation -- --nocapture`. Verify `delta = 0` between ledger requested bytes and counting allocator requested bytes.
  2. In `packages/slotmap`, update syntax (`for entry in ...`) and rename test files to kebab-case (`basic-test.ph`, `dense-test.ph`).
  3. Update `phalcom-core/examples/perf_audit_p7.rs` (CSV escaping, BLAKE3-256 terminology, fresh-process RSS measurement).
  4. Produce `PERF003.C2.P7.PATCH5-walkthrough.md`.

---

## 6. Known Repository Drift & Notes for Tests

- **Legacy Test Fixtures:** In `phalcom-semantic/tests/semantic.rs`, integration test fixtures directly construct `LinkedModule` struct literals with raw `BTreeMap` fields. `From<BTreeMap<K, V>> for ModuleAuthorityMap<K, V>` or test helper adaptations can be added if broader legacy test suites are run. For BUILD mode, follow the plan:
  ```sh
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta --lib
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules --lib
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact_allocation
  ```

---

## 7. Must-Read Documents

1. `docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/PERF003.C2.P7.PATCH5-Luna-Patch-Grade-Implementation-Plan.md`
2. `docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-allocation-materialization-architecture-ruling-2026-10-15.md`
3. `phalcom-type-meta/src/allocation.rs`
4. `phalcom-modules/src/authority.rs`
5. `phalcom-semantic/src/types/store.rs`
6. `phalcom-semantic/src/types/type_lambda.rs`
7. `phalcom-core/tests/artifact_allocation.rs`
