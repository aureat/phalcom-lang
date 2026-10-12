---
id: COLL006.C1.P6
program: COLL006
checkpoint: COLL006.C1
kind: handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# COLL006.C1.P6 — Handoff to P7/P9

`COLL006.C1.P6` is **COMPLETE / IMPLEMENTED / FOCUSED_TESTED** on
`feat/telos-collection-upgrade` in `.worktrees/telos-collections-upgrade`.
All P6 implementation tasks (T0–T7) and gates (G1–G7) are complete and verified.
The C1 checkpoint record remains the single durable shared state authority.

---

## 1. Stable Takeover Interfaces

The successor plan (`COLL006.C1.P7` — Persistent Hash Substrate, `Bag<T>`, `Bijection<K,V>`)
and future plans inherit the following stable authorities:

1. **Native Catalog and Floor (TDR-0092)**:
   - Exactly 29 native bindings registered: 9 for `MutableBag`, 10 for `IndexedDict`,
     and 10 for `OrderedSet`.
   - Native descriptor catalog verified at exactly 458 declarations in
     `phalcom-native-surface/src/generated.rs`.

2. **Heap Storage and Object Variants**:
   - `Object::MutableBag(MutableBagObject)` in `phalcom-core/src/heap/mutable_bag.rs`.
   - `Object::IndexedDict(IndexedDictObject)` in `phalcom-core/src/heap/indexed_dict.rs`.
   - `Object::OrderedSet(OrderedSetObject)` in `phalcom-core/src/heap/ordered_set.rs`.
   - Full GC tracing integration in `phalcom-core/src/heap/trace.rs`.

3. **Shared Key and Re-entry Engine**:
   - `phalcom-core/src/primitive/ordered_hash.rs` centralizes Telos hash dispatch,
     hash bucket enumeration, Telos equality dispatch, numeric SameValueZero
     equivalence, and receiver reentrancy counters across Map, Set, MutableBag,
     IndexedDict, and OrderedSet.
   - Invariant: No live Heap borrow is held across user callbacks.
   - Start-of-call logical snapshots are used for equality traversals.
   - Reentrant `IndexedDict` overwrites commit outer values while inner commits
     are not rolled back on outer failure.

4. **Canonical Key Admission**:
   - `phalcom-core/src/primitive/mod.rs::is_mutable_collection_key` is the central
     classifier rejecting:
     `List`, `Array`, `Buffer`, `Slice`, `Map`, `Set`, `MutableBag`, `IndexedDict`,
     `OrderedSet`, and `Bytes`.
   - `Vector<T, N>` and immutable scalars remain verified as valid keys.

5. **Universe Collections Surface**:
   - `phalcom-core/core/universe/src/collections/mutable_bag.ph`
   - `phalcom-core/core/universe/src/collections/indexed_dict.ph`
   - `phalcom-core/core/universe/src/collections/ordered_set.ph`
   - `IndexedDict.insert(entry, at:) -> Result<Unit, Error>` constructs concrete
     `DuplicateKeyError` and `IndexError` subclasses without typechecker or VM workarounds.

6. **Semantic Universe Hierarchy**:
   - `phalcom-semantic/src/db/query.rs` and `canonical_baseline.rs` share source
     superclass resolution for canonical classes.
   - `phalcom-semantic/src/checker/call.rs` seeds variant-constructor expected types
     prior to argument analysis.

---

## 2. Accepted Evidence

All focused verification gates passed:

```sh
# Native floor verification (458 declarations)
cargo run -p phalcom-native-surface-gen -- --root . --check

# Semantic hierarchy and generic application (20 passed)
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::canonical_universe_source_hierarchy
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic types::generic_application

# P6 combined contracts and GC (21 assertions passed)
RUSTFLAGS='' cargo test -p phalcom-core --test core collections::contract::tests::p6_mutable_hash_families_preserve_callback_snapshot_and_gc_contracts

# Central mutable-key admission matrix (50 rejections + Vector acceptance passed)
RUSTFLAGS='' cargo test -p phalcom-core --test core collections::contract::tests::p6_mutable_key_admission_matrix

# Focused collection test suites (13 passed across 3 suites)
RUSTFLAGS='' cargo test -p phalcom-core --test core collections::indexed_dict
RUSTFLAGS='' cargo test -p phalcom-core --test core collections::ordered_set
RUSTFLAGS='' cargo test -p phalcom-core --test core collections::mutable_bag

# Aggregate core collections module (15 passed)
RUSTFLAGS='' cargo test -p phalcom-core --test core core_collections

# Target and workspace compilation checks
cargo check -p phalcom-core --all-targets
cargo check --workspace --all-targets
git diff --check
```

---

## 3. Retained Residuals and Owner Obligations

The following known issues are outside P6 scope and remain assigned to P9 or their
owning subsystems:

1. **`R-PARSE-ASSOCIATED-INVOKE` (Compiler Layer / Assigned to P9)**:
   - In `packages/parse/tests/runtime/src/core/source-test.ph`, token-backed Cursor
     invocations of `current`/`advance` trigger an associated-family invocation lowering
     error in `phalcom-core/src/compiler/lib/associated.rs`.
   - P9 must rerun `cargo test -p phalcom-core --test core modules_parse_package`
     after the compiler associated-resolution owner work lands.

2. **`D1`, `D2`, `D4` (Pre-existing Static/Reflection Baseline / Assigned to P9)**:
   - Inherited static constant computed Record-key, reflection, and metadata failures
     remain baseline items.

3. **Language Corpus Range Failures**:
   - Pre-existing numeric Range iteration failures in corpus tests remain unchanged
     baseline.

---

## 4. Next Plan Guidance

Proceed to **`COLL006.C1.P7`**:
- Plan document: `docs/implementation/COLL006/COLL006.C1/COLL006.C1.P7-luna-patch-grade-implementation-plan.md`.
- Objective: Implement persistent hash storage, `Bag<T>`, and `Bijection<K,V>` with
  structural sharing and old-root immutability.
- Do not redesign or modify the 29 P6 native bindings or the shared `ordered_hash.rs` engine.
- Re-read P7 plan requirements and confirm baseline before editing.
