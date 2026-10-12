---
id: COLL006.C1.P8
program: COLL006
checkpoint: COLL006.C1
kind: handoff
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
---

# COLL006.C1.P8 — Handoff Record (Tasks T0–T2 to Successor Implementer)

## 1. Executive Summary

`COLL006.C1.P8` (Generic Intervals, Persistent Ordered Storage, and `RangeMap<K,V>`) is currently **`IN_PROGRESS / PARTIAL / FOCUSED_TESTED`** on dedicated branch `feat/telos-collections` in worktree `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections`.

- **Architectural Route**: **Route A (Source-first persistent AVL in Telos)** confirmed by Gate G0 capability probes.
- **Native Floor**: Strictly frozen at **488 declarations** (zero new native bindings).
- **Existing Collections**: Numeric `Range` remains completely untouched. Memory layout `size_of::<Object>() == 48` preserved.
- **Universe Integration**: `collections.interval` is authored, exposed in `collections/package.ph`, registered in `phalcom-modules/src/builtin.rs`, and recognized in `PreludeTypeMap` in `phalcom-semantic/src/prelude.rs`. Canonical Universe compilation passes cleanly.

---

## 2. Stable Takeover Interfaces & Implemented Assets

The successor agent inherits the following verified authorities:

1. **Normative Specification Ratification (Gate G1 PASSED)**:
   - `docs/specs/collection-families.md`:
     - §7: `Bound<T>` and `Interval<T>` specification.
     - §8: `RangeMap<K,V>` specification (invariants, lookup $O(\log n)$, strict insert, coverage subtraction, overlay $O(\log n + k)$, coalescing, iteration, equality/hash).
     - §9: Errors: `InvalidIntervalError is ArgumentError` and `RangeOverlapError is ArgumentError`.

2. **Error Hierarchy Definitions**:
   - `phalcom-core/core/universe/src/errors/argument.ph`: `InvalidIntervalError` and `RangeOverlapError` declared and exported.
   - `phalcom-core/core/universe/src/errors/package.ph`: Exposed to `errors` package.

3. **Interval Kernel (`phalcom-core/core/universe/src/collections/interval.ph`)**:
   - `Bound<T>`:
     - Variants: `@variant Unbounded`, `@variant Included(_ value: T)`, `@variant Excluded(_ value: T)`.
     - Accessors: `isUnbounded -> Bool`, `isIncluded -> Bool`, `isExcluded -> Bool`, `value -> Option<T>`.
     - Factory methods: `@class unbounded`, `@class included(_ value: T)`, `@class excluded(_ value: T)`.
   - `Interval<T>(_ lower: Bound<T>, _ upper: Bound<T>)`:
     - Factory methods: `closed`, `open`, `halfOpenRight`, `halfOpenLeft`, `atLeast`, `greaterThan`, `atMost`, `lessThan`, `all`, `singleton`.
     - Endpoint algebra kernel:
       - `compareDomain<K>(_ a: K, _ b: K) -> Ordering`: uses `<=>`, rejects `Ordering::Unordered` with `InvalidIntervalError`.
       - `compareLower`: canonical lower bound total order (Unbounded < Included(x) < Excluded(x)).
       - `compareUpper`: canonical upper bound total order (Excluded(y) < Included(y) < Unbounded).
       - `complementLowerForLeftUpper`: transforms lower bound to upper complement.
       - `complementUpperForRightLower`: transforms upper bound to lower complement.
       - `isValid`: checks lower $\le$ upper and absence of unordered endpoints.
       - `isEmpty`: detects empty intervals (`[x, x)`, `(x, x]`, `(x, x)`).
       - `contains`: point containment query.
       - `intersection`: computes interval intersection, returning `Option<Interval<T>>`.
       - `overlaps`: boolean test for nonempty intersection.
       - `touchesWithoutGap`: tests whether two disjoint intervals share a boundary without an unmapped gap.
       - `unionIfContiguous`: merges overlapping or contiguous touching intervals.
       - `remaindersAfterRemoval`: coverage subtraction returning array of remaining pieces (`0`, `1`, or `2` intervals).
       - `toString`: standard bracket/parenthesis notation (`[a, b]`, `[a, b)`, `(-inf, b]`, etc.).

4. **Compiler & Prelude Integration**:
   - `phalcom-semantic/src/prelude.rs`: `Bound`, `Interval`, `InvalidIntervalError`, and `RangeOverlapError` registered in canonical `PreludeTypeMap`.
   - `phalcom-modules/src/builtin.rs`: `collections.interval` registered with provider and children.
   - `phalcom-core/tests/core/mod.rs`: `core_interval` and `core_range_map` modules registered.

---

## 3. Critical Technical Lessons & Gotchas

Future implementers must adhere to these findings:

1. **Telos Syntax Rules**:
   - **Match Arms**: Do NOT use trailing commas between arms (`=> expr` followed by newline).
   - **Negation**: Use `not expr`, never `!expr`.
   - **Statements in Match**: `return false` directly in an arm is a syntax error because `return` is a statement. Use `{ return false }` or use expressions.
   - **No Match on `<=>`**: Matching on `ord = a <=> b` can cause semantic checker reflection panics. Compare using `if ord == Ordering::Unordered` or `if ord == Ordering::Equal`.

2. **Generic Variant Construction & Lowering**:
   - **Positional Variant Args**: Variants with parameters must have an underscore `_` to accept positional arguments: `@variant Included(_ value: T)`, not `@variant Included(value: T)`.
   - **Specialized Constructor Syntax**: When invoking variant constructors on generic enums inside source code, use the specialized type form: `Bound<T>::Included(value)` or `Bound<T>::Unbounded`. Calling unspecialized `Bound::Included(value)` produces `AssociatedGenericUnderconstrained` or `AssociatedInvokeNotLoweredYet`.
   - **Unbounded in Unconstrained Positions**: In calls like `Interval(Bound<T>.unbounded, ...)`, use the class method `Bound<T>.unbounded` to give the compiler an explicit return type constraint.

3. **Immediate / Unboxed `Option<T>` Representation at Runtime**:
   - In the Phalcom VM, `Option<T>` is an immediate representation: `Some(v)` at runtime is the raw value `v` directly (e.g. `Some(10)` is the integer `10`). `None` is `Value::none()`.
   - Because `10` is an `Int` and does not inherit from `Option`, calling `.isSome` on `bIncluded.value` fails with `10 does not understand 'isSome'`.
   - In tests, to assert on an `Option<T>` return value:
     - Check `opt == None` for `None`.
     - Pattern match: `match opt { Option::Some(v) => v == expected, _ => false }`.
     - Or compare directly against the expected value if unwrapped: `opt == expected`.

4. **Working Directory (`Cwd`)**:
   - ALWAYS run commands with `Cwd: "/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections"`.

---

## 4. Immediate Next Tasks for the Successor Agent

### Step 1: Finalize Gate G2 (Interval Verification Suite)
- In `phalcom-core/tests/core/collections/interval.rs`:
  - Adjust `test_cv01_bound_construction_and_accessors` to eliminate `Option` method calls on raw integers (use pattern match `match b.value { Option::Some(v) => v == 10, _ => false }`). Verify it passes.
  - Implement the remaining CV-02 through CV-13 test cases:
    - `CV-02`: `compareLower` ordering & tie-breaking.
    - `CV-03`: `compareUpper` ordering & tie-breaking.
    - `CV-04`: `isValid` & `isEmpty` boundary matrix (`[x,x]`, `[x,x)`, `(x,x]`, `(x,x)`, descending).
    - `CV-05`: `contains` point query across bracket permutations and infinite bounds.
    - `CV-06`: `overlaps` and `intersection`.
    - `CV-07`: `touchesWithoutGap` and `unionIfContiguous`.
    - `CV-08`: `remaindersAfterRemoval` (0, 1, and 2 remainder cases).
    - `CV-09`: Custom non-numeric ordered domain (`TestDate`).
    - `CV-10`: `toString` representation formatting.
    - `CV-11`: `Ordering::Unordered` rejection (`InvalidIntervalError`).
    - `CV-12`: Comparator callback error propagation.
  - Run `cargo test -p phalcom-core --test core core_interval` and confirm Gate G2 passes.

### Step 2: Implement Task T3 & T4 (Persistent AVL Substrate & RangeMap Core)
- Create `phalcom-core/core/universe/src/collections/range_map.ph`:
  - Internal persistent tree node: `class _RangeMapNode<K, V>`:
    - Fields: `interval: Interval<K>`, `value: V`, `left: Option<_RangeMapNode<K, V>>`, `right: Option<_RangeMapNode<K, V>>`, `height: Int`.
    - Balancing: standard AVL balance factor, single rotations (LL, RR), double rotations (LR, RL).
    - Search: Point lookup `findPoint(point)` in $O(\log n)$ using `interval.contains(point)`.
    - Insertion: `insertByLower(interval, value)` enforcing AVL balance invariants and disjoint intervals.
  - `class RangeMap<K, V> is Collection`:
    - Public API: `new()`, `size`, `isEmpty`, `get(point) -> Option<V>`, `containsPoint(point) -> Bool`.
    - `inserting(interval: Interval<K>, value: V) -> RangeMap<K, V>`:
      - Reject invalid or empty interval with `InvalidIntervalError`.
      - Reject any overlap with existing entries with `RangeOverlapError`.
      - Return new persistent `RangeMap<K, V>` with updated root.
- Expose `.range_map` in `collections/package.ph` and register in `builtin.rs` and `prelude.rs`.
- Implement tests in `phalcom-core/tests/core/collections/range_map.rs` and verify Gates G3 & G4.

### Step 3: Implement Task T5 (Range Removal, Overlay, and Splicing)
- In `range_map.ph`:
  - Implement `removing(interval: Interval<K>) -> RangeMap<K, V>`:
    - Finds all intersecting entries in $O(\log n + k)$.
    - For each intersecting entry, replaces it with `entry.interval.remaindersAfterRemoval(interval)`.
  - Implement `overlaying(interval: Interval<K>, value: V) -> RangeMap<K, V>`:
    - Equivalent to `self.removing(interval).inserting(interval, value)` in $O(\log n + k)$.
- Add tests in `range_map.rs` covering edge trims, internal splits, multi-interval replacements, and verify Gate G5.

### Step 4: Implement Task T6 (Normalization, Iteration, Equality, and Hash)
- Coalescing: merge contiguous adjacent intervals with equal values (`==`).
- Iteration: in-order traversal iterator `RangeMapIterator<K, V>` yielding `(interval, value)`.
- Equality: logical entry sequence comparison after normalization.
- Hash: stable hashing over normalized logical entries.
- Verify Gate G6.

### Step 5: Tasks T7 & T8 (Hardening & Cross-Collection Regression)
- GC stress test, re-entrancy callback safety, randomized oracle testing.
- Verify Gate G7 and run aggregate collection suites (`core_collections`, `core_bag`, `core_bijection`).
- Close out plan, update `COLL006.C1-CHECKPOINT.md` to `COMPLETE`, author walkthrough and handoff to P9.
