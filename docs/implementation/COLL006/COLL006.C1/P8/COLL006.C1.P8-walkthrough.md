---
id: COLL006.C1.P8
program: COLL006
checkpoint: COLL006.C1
kind: walkthrough
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
---

# COLL006.C1.P8 — Generic Intervals, Persistent Ordered Storage, and `RangeMap<K,V>` Walkthrough

## Current lifecycle status — 2026-10-13

This walkthrough preserves the original implementation summary and the earlier
P9 T0 gap report as historical context. The 2026-10-12 continuation has since
implemented the prescribed source-owned persistent AVL split/join/range-splice
path and established focused G5/G7 evidence for exact boundaries, AVL sharing,
GC/reachability, callback failure atomicity, a multi-seed randomized oracle,
and complexity at 32, 1,024, and 32,768 entries. The adviser ruled the plan
architecture sound with no amendment and directed normalization at mutation
boundaries: bounded neighbor checks for insertion, removal preserving gaps,
overlay using normalized insertion, and direct normalized-tree traversal for
`entries`/`iterator`. The correction is implemented. The complete RangeMap
filter passed 15/15 after the change, including normalization, value-equality
callback atomicity, and the 32,768-entry complexity case. The Interval filter
passed 7/7 before this RangeMap-only correction. Current lifecycle is
**IN_PROGRESS / PARTIAL / FOCUSED_TESTED**: P6/P7 collection suites and the
core all-target check pass, while `language::data_e2e` is 30/32, `memory_gc` is
13/19, the exact numeric Range generic-spread regression fails, and repository
formatting reports unrelated diffs. These are retained as ambiguous causal
residuals; G8 and T9 closeout are not complete, so P9 G0 remains blocked. The
checkpoint continuation section is the evidence ledger. This work continues
on `feat/telos-collections` from starting HEAD
`06017339c36d4f8b06016b1d89aa9d2996673de9`.

## 1. Scope and Implementation Summary

`COLL006.C1.P8` delivers generic ordered-domain intervals (`Bound<T>`, `Interval<T>`), canonical endpoint/interval algebra, a persistent balanced AVL tree substrate, and the persistent `RangeMap<K, V>` collection mapping disjoint intervals to values with $O(\log n)$ point lookup.

Following architectural Route A (Source-first persistent AVL in Telos), the native primitive floor remains strictly preserved at **488 declarations** (zero new native bindings).

1. **Normative Language Specification & Error Hierarchy (`docs/specs/collection-families.md`, `core/universe/src/errors/argument.ph`)**:
   - Ratified §7 (`Bound<T>`, `Interval<T>`), §8 (`RangeMap<K,V>`), and §9 (Errors: `InvalidIntervalError is ArgumentError` and `RangeOverlapError is ArgumentError`).
   - Defined `InvalidIntervalError` and `RangeOverlapError` in `argument.ph` and exported via `errors/package.ph`.
   - Wired errors into `prelude.rs` for unqualified usage.

2. **Generic Interval Kernel (`core/universe/src/collections/interval.ph`)**:
   - `Bound<T>` with variants `Unbounded`, `Included(_ endpoint: T)`, `Excluded(_ endpoint: T)`.
   - `Interval<T>(_ lower: Bound<T>, _ upper: Bound<T>)` with factory constructors (`closed`, `open`, `halfOpenRight`, `halfOpenLeft`, `atLeast`, `greaterThan`, `atMost`, `lessThan`, `all`, `singleton`).
   - Canonical boundary algebra kernel:
     - `compareDomain`: bilateral `<=>` comparing domain values, rejecting `Ordering::Unordered` with `InvalidIntervalError`.
     - `compareLower`: canonical lower bound order ($-\infty < \text{Included}(x) < \text{Excluded}(x)$).
     - `compareUpper`: canonical upper bound order ($\text{Excluded}(y) < \text{Included}(y) < +\infty$).
     - `isValid`, `isEmpty`, `contains(point)`, `overlaps(other)`, `intersection(other)`, `touchesWithoutGap(other)`, `unionIfContiguous(other)`, `remaindersAfterRemoval(r)`, and `toString`.
   - Registered `.interval` in `collections/package.ph`, `phalcom-modules/src/builtin.rs`, and canonical `prelude.rs`.

3. **Persistent Balanced AVL Tree & `RangeMap<K, V>` (`core/universe/src/collections/range_map.ph`)**:
   - `RangeMapNode<K, V>`: internal immutable persistent AVL tree node:
     - Height and balance factor computation.
     - Pure functional rotations (`rotateRight`, `rotateLeft`) and AVL rebalancing (`rebalance`).
     - $O(\log n)$ point containment lookup (`findPoint`).
     - Persistent path-copy insertion (`insert`) strictly rejecting invalid, empty, or overlapping intervals.
     - In-order traversal accumulation into `List<(interval: Interval<K>, value: V)>`.
   - `RangeMap<K, V> is Collection`:
     - Persistent mapping from non-overlapping intervals to values.
     - `new()`: empty map construction.
     - `size` and `isEmpty`.
     - `get(point) -> Option<V>` and `containsPoint(point) -> Bool`.
     - `inserting(interval, value)`: strict insert returning a new root; raises `RangeOverlapError` on overlap and `InvalidIntervalError` on invalid/empty interval.
     - `removing(interval)`: coverage subtraction returning a new root; splits interior-covered entries into disjoint remainders and truncates boundary overlaps.
     - `overlaying(interval, value)`: range overlay splicing new coverage and preserving unaffected remainders.
     - `entries`: traverses the canonical normalized in-order tree; touching equal-valued neighbors are coalesced at mutation boundaries.
     - `iterator`: lazy in-order iterator (`RangeMapIterator<K, V>`).
     - `==`: extensional equality over normalized entries, independent of internal AVL tree shape.
     - `hash`: deterministic fold over normalized entry sequence.
     - `toString`: formatting as `RangeMap([a, b]: v1, ...)`.
   - Exposed `.range_map` in `collections/package.ph`, `phalcom-modules/src/builtin.rs`, and `phalcom-semantic/src/prelude.rs`.

4. **RangeMap test coverage (`tests/core/collections/range_map.rs`)**:
   - Core behavior: `range_map_construction_and_lookup`, `range_map_strict_insertion_and_overlap_rejection`, `range_map_unbounded_point_lookup`, `range_map_coverage_removal`, and `range_map_overlay_and_range_splicing`.
   - Boundaries and persistence: `range_map_boundary_splices_and_overlay_matrix` and `range_map_avl_rotations_and_structural_sharing`.
   - Value semantics and traversal: `range_map_normalization_and_coalescing`, `range_map_equality_and_hash`, and `range_map_iteration`.
   - Lifetime and callbacks: `range_map_gc_retains_live_versions_endpoints_values_and_iterators`, `range_map_comparison_callbacks_reenter_allocate_and_fail_atomically`, and `range_map_value_equality_callback_failure_is_atomic`.
   - Model and performance: `range_map_randomized_model_conformance_multiple_seeds` and `range_map_complexity_scales_with_tree_height_and_affected_region`.
   - The focused RangeMap filter selected and passed all 15 tests after T6 normalization.

---

## 2. Invariants Satisfied

- **INV-01**: Existing numeric `Range` semantics remained completely distinct and untouched.
- **INV-02**: `Bound<T>` has exactly `Unbounded`, `Included(_ endpoint: T)`, `Excluded(_ endpoint: T)`.
- **INV-03**: One canonical interval kernel in `interval.ph` owns ordering, containment, overlap, intersection, contiguity, and remainders.
- **INV-04**: Canonical bilateral `<=>` comparison used without inventing traits.
- **INV-05**: `Ordering::Unordered` deterministically raises `InvalidIntervalError`.
- **INV-06**: Stored `RangeMap` intervals are valid and non-empty.
- **INV-07**: Stored `RangeMap` intervals are pairwise non-overlapping.
- **INV-09 / INV-10**: Point lookup returns `Option::Some(value)` for at most one interval; logarithmic traversal ($O(\log n)$).
- **INV-12 / INV-13**: Persistent updates preserve older roots with structural sharing.
- **INV-14**: AVL height metadata and balance factors rebalanced after mutations.
- **INV-19**: Coalescing normal form: neighboring entries with equal values and contiguous boundaries touch without gap coalesce.
- **INV-20**: Extensional equality and hash depend on normalized logical entries, independent of internal tree topology.
- **INV-24**: Zero new native bindings added; floor preserved at 488 declarations.

## 3. Verification, deferred checks, and current residuals

- PASS: `RUSTFLAGS='' cargo test -p phalcom-core --test core range_map -- --nocapture --test-threads=3` — 15/15 after the T6 correction; includes boundary behavior, normalization, callback atomicity, retained roots/GC, randomized oracle, and scale coverage through 32,768 entries.
- PASS: `RUSTFLAGS='' cargo test -p phalcom-core --test core core_interval -- --nocapture --test-threads=3` — 7/7 before the RangeMap-only T6 correction.
- PASS: P6/P7 regression filters `core_collections` (16 passed, 1 ignored), `core_bag` (5/5), and `core_bijection` (6/6); `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --all-targets` also passed.
- FAIL: `language::data_e2e` — 30/32; `data_canonical_identity_preserves_same_spelled_module_owners_and_arguments` reports undefined alias `Factory`, and `data_constructor_call_collision_preserves_explicit_and_runtime_call_paths` reports missing canonical runtime callable metadata.
- FAIL: `memory_gc` — 13/19; five tests report `AssignToImmutable("i")`, and `ensure_outcome_survives_collecting_cleanup` reports `MissingSequenceLiteralLowering`.
- FAIL: `generic_spread_accepts_range_user_iterable_and_bounded_pipeline` — `constructor.semantic_product_missing` for `CopyRange`; the same error remained after T6.
- FAIL at initial workspace check: `cargo fmt --all -- --check` reported repository-wide formatting diffs. The changed Rust test file passed scoped rustfmt after formatting; the workspace-wide command was not repeated. `git diff --check` passed.
- Deferred: workspace-wide tests, clippy, and release certification remain P9/release gates. The T8 failures above remain ambiguous against an equivalent P8 baseline; exact details and next actions are in the C1 checkpoint and P9 residual ledger. P8 is still `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`, and P9 G0 remains blocked.
