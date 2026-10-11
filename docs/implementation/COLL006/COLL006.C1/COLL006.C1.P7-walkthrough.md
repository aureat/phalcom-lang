---
id: COLL006.C1.P7
program: COLL006
checkpoint: COLL006.C1
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# COLL006.C1.P7 — Persistent Hash Collections Walkthrough

## 1. Scope and Implementation Summary

COLL006.C1.P7 delivers the persistent immutable hash trie substrate (`PersistentHashNode`),
the persistent multiset family `Bag<T>`, and the persistent bidirectional mapping `Bijection<K, V>`.

1. **Persistent Hash Trie Substrate (`heap/persistent_hash.rs`, `primitive/persistent_hash.rs`)**:
   - `PersistentHashNode` implemented with three GC-visible variants: `Leaf`, `Collision`, and `Branch`.
   - Boxed in `Object::PersistentHashNode(Box<PersistentHashNode>)` to strictly preserve `size_of::<Object>() == 48`.
   - Reusable HAMT algorithm implementing 32-way 5-bit branching with bitmapped indexing and path-copying persistence.
   - Strict two-phase execution:
     - Phase A preflight (`prepare_insert`, `prepare_remove`) inspects the trie, traverses path frames, and invokes user `hash` and `==` callbacks under VM temp roots without holding any live heap borrow.
     - Phase B commit (`commit_insert`, `commit_remove`, `commit_replace_payload`) executes purely on `&mut Heap` with zero user callbacks, guaranteeing failure atomicity and heap borrow safety.
   - Dedicated unit and property test suite across CV-01 through CV-17 and randomized property model comparisons against `std::collections::HashMap` (Gate G3 PASSED).

2. **Native Floor & Governance (TDR-0093 / Gate G4)**:
   - Authored consultation packet `COLL006.C1.P7-api-and-floor-consultation.md` and ratified `TDR-0093`.
   - Authorized exactly 14 native primitives (7 for `Bag`, 7 for `Bijection`).
   - Extended normative language specification in `docs/specs/collection-families.md` (§5 `Bag<T>`, §6 `Bijection<K,V>`, §7 error boundary).

3. **`Bag<T>` Persistent Multiset (`heap/bag.rs`, `primitive/bag.rs`, `collections/bag.ph`)**:
   - Backed by `BagObject` storing optional trie root (`Option<ObjRef>`), exact unbounded occurrence total (`total_count: BigInt`), and logical cardinality (`distinct_count: usize`).
   - 7 approved native bindings: `new()`, `rawSize`, `rawDistinctCount`, `rawCount(_)`, `rawAdding(_,_)`, `rawRemoving(_,_)`, `rawKeys`.
   - Telos source class `Bag<T>` with full multiset algebra: `+` (sum of multiplicities), `union` (maximum multiplicities), `intersection` (minimum multiplicities), and `difference` (saturating subtraction).
   - Lazy multiset iteration via `BagIterator<T>` retaining owner reference.
   - Order-independent equality and hash law implementation.
   - Rejects mutable collection keys via `is_mutable_collection_key`.
   - Certified across all 5 test scenarios in `tests/core/collections/bag.rs` (Gate G5 PASSED).

4. **`Bijection<K, V>` Persistent Bidirectional Mapping (`heap/bijection.rs`, `primitive/bijection.rs`, `collections/bijection.ph`)**:
   - Defined `BijectionConflictError` in `core/universe/src/errors/argument.ph` and exported via `errors/package.ph`.
   - Backed by `BijectionObject` storing dual trie roots (`forward_root`, `reverse_root`) and entry count (`len: usize`).
   - 7 approved native bindings: `new()`, `rawSize`, `rawGet(_)`, `rawAssociate(_,_)`, `rawRemoveKey(_)`, `rawInverse`, `rawKeys`.
   - Dual-root atomic preflight: validates that `key` does not map to a different value and `value` does not map to a different key; raises `BijectionConflictError` on either conflict; permits idempotent re-association of identical pairs.
   - Symmetrical O(1) inverse operation (`rawInverse`) via simple root pointer swap.
   - Symmetrical removals via `removingKey` and `removingValue`.
   - Telos source class `Bijection<K, V>` and iterator `BijectionIterator<K, V>`.
   - Certified across all 6 test scenarios in `tests/core/collections/bijection.rs` (Gate G6 PASSED).

5. **Cross-Family Invariants & Native Surface (Gate G7)**:
   - Native surface generator regenerated and verified current at **488 declarations** (`cargo run -p phalcom-native-surface-gen -- --root . --check`).
   - Aggregate collection suite `tests/core/collections/contract.rs` verified with 16 passed tests and zero regressions across all 12 collection families.

---

## 2. Verification Evidence

| Gate | Suite / Command | Result |
|---|---|---|
| **G2: Node GC Tracing** | `cargo test -p phalcom-core --test core memory_gc` | **PASSED** (all nodes traced, zero leaks) |
| **G3: Persistent HAMT Engine** | `cargo test -p phalcom-core --test core core_persistent_hash -- --nocapture` | **PASSED** (9 suites, CV-01..CV-17 + random property model) |
| **G4: Native Floor** | `TDR-0093` ratified; `cargo run -p phalcom-native-surface-gen -- --root . --check` | **PASSED** (488 declarations, current) |
| **G5: Bag Multiset** | `cargo test -p phalcom-core --test core core_bag -- --nocapture` | **PASSED** (5/5 suites: construction, removals, algebra, equality/hash, iteration) |
| **G6: Bijection Mapping** | `cargo test -p phalcom-core --test core core_bijection -- --nocapture` | **PASSED** (6/6 suites: lookup, conflict detection, inverse, removals, equality/hash, iteration) |
| **G7: Collections Aggregate** | `cargo test -p phalcom-core --test core core_collections -- --nocapture` | **PASSED** (16/16 active tests passed) |

---

## 3. Residuals & Baseline Observations

- `R-P7-NUMERIC-HASH-LAW`: Numeric hash divergence for float values $\ge 2^{53}$, large integer representations, and distinct NaNs documented and isolated in `contract.rs`.
- `R-PARSE-ASSOCIATED-INVOKE`: Cursor invocation in parser retained from P5; scheduled for P9.
