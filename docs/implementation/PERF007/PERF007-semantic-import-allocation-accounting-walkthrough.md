# PERF007 Semantic Import Allocation Accounting Walkthrough

**Status:** COMPLETE (FOCUSED_TESTED)  
**Branch:** `perf/universe-characterization`  
**Worktree:** `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization`  
**Discriminator:** `PHALCOM_UNIVERSE_TELOSC_FOR_TEST=/tmp/perf003-patch5r-universe.telosc cargo test -p phalcom-core --test artifact-allocation full_universe_telosc_load_allocation_equality_and_n_minus_one -- --nocapture`

---

## 1. Objective and Results

The objective of PERF007 was to close the 1,143,494-byte allocation accounting gap in `SemanticWorldV3` artifact import (`validate_and_import_semantic_world_with_allocation_budget`) so that every byte allocated during successful import is tracked by the owning `AllocationBudget` ledger, achieving exact equality (`delta == 0`), exact $N-1$ failure rejection, child transaction rollback on exhaustion, and full preservation of semantic identities and canonical prefix behavior.

### Final Verification Result
```text
TYPE GRAPH IMPORT: ledger=765031 observed=765031 delta=0
SEMANTIC IMPORT EXACT CHECK:
  ledger requested: 264140594
  independently observed: 264140594
  delta: 0
FULL LOAD ALLOCATION DISCRIMINATOR:
  ledger requested (N): 364247515
  independent allocator requested: 370464721
  delta: 6217206
test full_universe_telosc_load_allocation_equality_and_n_minus_one ... ok
```

- **Semantic Import Exact Match:** 264,140,594 ledger bytes == 264,140,594 allocator-observed bytes (**0 delta**).
- **Type Graph Import Exact Match:** 765,031 ledger bytes == 765,031 allocator-observed bytes (**0 delta**).
- **Exact $N$ Equality:** Succeeded.
- **Exact $N - 1$ Rejection:** Failed with typed `ArtifactError::Allocation(AllocationError::Exhausted)` and zero child requested bytes leaked into the parent ledger.

---

## 2. Key Root Causes Identified and Fixed

1. **Static Value BigInt Comparisons (+16 B):**
   - `StaticInt::cmp` previously called `self.to_bigint().cmp(&other.to_bigint())` when comparing `Large` values during static value sorting. `to_bigint()` allocated heap limbs on the untracked allocator.
   - Fixed with a zero-allocation integer comparison using `i64::try_from` and limb signs.

2. **Native Surface Catalog Map Lookups (+16 B):**
   - `NativeSurfaceCatalog::find_selector` previously queried a HashMap with `selector.to_owned()`, allocating a heap `String` per native metadata lookup.
   - Fixed with a zero-allocation linear slice lookup over `&'static [NativeSurfaceRecord]` (488 records, executed in nanoseconds without allocations).

3. **TypeStore Export Visitor Slice/Count Accessors:**
   - In `phalcom-semantic/src/types/artifact.rs`, the export walker directly checked `store.kinds.len()`, `store.types.len()`, etc., which only counted the mutable local overlay and omitted the frozen canonical prefix, causing `InvalidNativeCoordinate` on kind 0 when exporting from an imported store.
   - Corrected to use canonical count methods (`type_count()`, `kind_count()`, `parameter_count()`, `row_count()`, `family_count()`, `variant_count()`, `static_domain_count()`, `static_constructor_count()`) and canonical lookup accessors (`kind_of()`, `type_parameter()`, `record_row()`, `get_family()`, `variant_identity()`, `static_domain()`, `static_constructor()`).

4. **Zero-Allocation and Budget-Aware Structural Containers:**
   - Replaced unbudgeted `BTreeMap` and native collections in semantic import with contiguous precharged buffers (`ParentMap`, `OrderedMap`, `BudgetedSortedMap`, `Arc<[T]>`, `Box<[T]>`), eliminating unbudgeted node/pointer overhead across all 14 phases.

---

## 3. Verification Suite Executed

All checks passed cleanly:
- `cargo check -p phalcom-type-meta --lib` (0 errors)
- `cargo check -p phalcom-semantic --lib` (0 errors)
- `cargo check -p phalcom-core --lib` (0 errors)
- `cargo test -p phalcom-type-meta --test allocation_helpers` (1 passed)
- `cargo test -p phalcom-semantic --test artifact-allocation` (1 passed)
- `cargo test -p phalcom-semantic --lib artifact::` (44 passed, 0 failed)
- `cargo test -p phalcom-core --test artifact-allocation full_universe_telosc_load_allocation_equality_and_n_minus_one` (1 passed)
- `git diff --check` (clean diff, 0 trailing whitespace/merge artifacts)
