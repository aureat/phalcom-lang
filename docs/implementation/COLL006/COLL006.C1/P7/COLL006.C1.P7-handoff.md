---
id: COLL006.C1.P7
program: COLL006
checkpoint: COLL006.C1
kind: handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# COLL006.C1.P7 — Handoff to P8

`COLL006.C1.P7` is **COMPLETE / IMPLEMENTED / FOCUSED_TESTED** on branch `main`.
All P7 implementation tasks (T0–T8) and gates (G1–G7) are complete and verified.
The C1 checkpoint record (`docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md`) remains the single durable shared state authority.

---

## 1. Stable Takeover Interfaces

The successor plan (`COLL006.C1.P8` — Interval and Range Hierarchy: `Bound<T>`, `Interval<T>`, `RangeMap<K,V>`)
and future plans inherit the following stable authorities:

1. **Native Catalog and Floor (TDR-0093)**:
   - Exactly 14 persistent hash native bindings registered: 7 for `Bag`, 7 for `Bijection`.
   - Native descriptor catalog verified at exactly 488 declarations in
     `phalcom-native-surface/src/generated.rs`.
   - `cargo run -p phalcom-native-surface-gen -- --root . --check` is clean.

2. **Persistent HAMT Substrate**:
   - `PersistentHashNode` (`Leaf`, `Collision`, `Branch`) in `phalcom-core/src/heap/persistent_hash.rs`.
   - Mark-sweep GC tracing in `phalcom-core/src/heap/trace.rs` traces HAMT nodes recursively without cycles.
   - Core two-phase engine in `phalcom-core/src/primitive/persistent_hash.rs`:
     - Phase A preflights user `hash` and `==` dispatch while parking newly created nodes in VM temp roots.
     - Phase B commits node allocations and updates on `&mut Heap` with zero user callbacks.
   - Guaranteed structural sharing and immutability of existing roots.

3. **`Bag<T>` Collection Family**:
   - Heap representation: `Object::Bag(Box<BagObject>)` in `phalcom-core/src/heap/bag.rs`.
   - Memory layout invariant: `std::mem::size_of::<Object>() == 48` preserved via boxed payload.
   - Universe definition: `core/universe/src/collections/bag.ph` with `BagIterator<T>` retaining owner reference.
   - Multiset algebra: exact `Int` multiplicities, automatic key deletion when multiplicity reaches zero, `rawCount`, `rawAdding`, `rawRemoving`, `rawKeys`.

4. **`Bijection<K, V>` Collection Family**:
   - Heap representation: `Object::Bijection(Box<BijectionObject>)` in `phalcom-core/src/heap/bijection.rs`.
   - Dual-root persistent HAMTs (`forward`: K $\to$ V, `reverse`: V $\to$ K).
   - O(1) inverse operation via root pointer swap in `rawInverse`.
   - Conflict preflight: pre-checks forward key existence and reverse value existence before allocating new nodes, raising `BijectionConflictError` on conflicts with full rollback safety.
   - Universe definition: `core/universe/src/collections/bijection.ph` with `BijectionIterator<K, V>`.

5. **`BijectionConflictError` Error Hierarchy**:
   - Defined in `core/universe/src/errors/argument.ph` as subclass of `ArgumentError`.
   - Re-exported in `core/universe/src/errors/package.ph`.
   - Registered in prelude bindings and instantiated dynamically on association collision.

6. **Universe Invariants and Metadata**:
   - Core class census in `phalcom-core/src/universe/invariants.rs` updated to 45 canonical classes.
   - Metadata entries in `phalcom-native-meta/src/universe.rs` and semantic identity in `phalcom-semantic/src/core_surface/identity.rs`.

---

## 2. Accepted Evidence

All focused verification gates passed on `main`:

```sh
# Native floor verification (488 declarations)
RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-native-surface-gen -- --root . --check

# Persistent HAMT substrate verification (CV-01..CV-17 contract and property suites)
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::persistent_hash

# Focused Bag test suite (5 passed)
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_bag

# Focused Bijection test suite (6 passed)
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_bijection

# Aggregate core collections suite (16 passed, 1 ignored for numeric hash law)
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_collections

# Compilation and target hygiene
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --all-targets
git diff --check
```

---

## 3. Retained Residuals and Owner Obligations

1. **`R-P7-NUMERIC-HASH-LAW` (Isolated in Test / Not a Blocker)**:
   - For numbers $\ge 2^{53}$ and NaNs, `SameValueZero` equivalence vs `Number#hash` can diverge if values are not normalized before hashing.
   - Pinned in `phalcom-core/tests/core/collections/contract.rs` under `#[ignore]` until Telos unified number hashing specification is revised.

2. **`R-PARSE-ASSOCIATED-INVOKE` (Compiler Layer / Assigned to P9)**:
   - In `packages/parse/tests/runtime/src/core/source-test.ph`, token-backed Cursor invocations trigger an associated-family invocation lowering error.
   - Handled under P9.

3. **`D1`, `D2`, `D4` (Pre-existing Baseline Failures)**:
   - Baseline constant computed Record-key, reflection, and metadata failures remain unchanged.

---

## 4. Next Plan Guidance

Proceed to **`COLL006.C1.P8`**:
- Plan document: `docs/implementation/COLL006/COLL006.C1/COLL006.C1.P8-luna-patch-grade-implementation-plan.md` (or detailed implementation plan).
- Objective: Implement interval and range hierarchy (`Bound<T>`, `Interval<T>`, `RangeMap<K, V>`).
- Do not redesign or alter the 14 persistent hash bindings or the persistent HAMT substrate established in P7.
- Verify native floor increments adhere to TDR conventions before updating `phalcom-native-surface`.
