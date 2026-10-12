---
id: COLL006.C1.P8
program: COLL006
checkpoint: COLL006.C1
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# COLL006.C1.P8 — Generic Intervals, Persistent Ordered Storage, and `RangeMap<K,V>` Walkthrough

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
   - `_RangeMapNode<K, V>`: internal immutable persistent AVL tree node:
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
     - `entries`: returns normalized in-order entry list, coalescing adjacent contiguous intervals with equal values (`touchesWithoutGap` + `value == value`).
     - `iterator`: lazy in-order iterator (`RangeMapIterator<K, V>`).
     - `==`: extensional equality over normalized entries, independent of internal AVL tree shape.
     - `hash`: deterministic fold over normalized entry sequence.
     - `toString`: formatting as `RangeMap([a, b]: v1, ...)`.
   - Exposed `.range_map` in `collections/package.ph`, `phalcom-modules/src/builtin.rs`, and `phalcom-semantic/src/prelude.rs`.

4. **Comprehensive Test Suites (`tests/core/collections/range_map.rs`, `tests/core/collections/interval.rs`)**:
   - `range_map_construction_and_lookup`: empty map, single/multiple disjoint insertions, point lookup inside/outside, and old-root immutability.
   - `range_map_strict_insertion_and_overlap_rejection`: overlap left/right/full/interior rejection with `RangeOverlapError`, invalid/empty rejection, and adjoining interval acceptance.
   - `range_map_unbounded_point_lookup`: point lookup in unbounded intervals (`lessThan`, `singleton`, `greaterThan`).
   - `range_map_coverage_removal`: coverage removal with interior hole splitting, truncation, empty interval identity, and immutability.
   - `range_map_overlay_and_range_splicing`: range overlay replacing middle segment and preserving remainders.
   - `range_map_normalization_and_coalescing`: verification that contiguous intervals with equal values coalesce, while gaps or differing values remain distinct.
   - `range_map_equality_and_hash`: order- and topology-independent structural equality and hash law.
   - `range_map_iteration`: lazy in-order traversal of entries.

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
