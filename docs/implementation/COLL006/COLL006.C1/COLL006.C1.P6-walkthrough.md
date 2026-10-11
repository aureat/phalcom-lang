---
id: COLL006.C1.P6
program: COLL006
checkpoint: COLL006.C1
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# COLL006.C1.P6 — Ordered Mutable Hash Family Walkthrough

## 1. Scope and Implementation Summary

COLL006.C1.P6 implements the ordered mutable hash collection families and factors
shared Telos key-lookup and re-entry machinery across the runtime:

1. **Shared Key Engine (`primitive/ordered_hash.rs`)**:
   - Centralizes Telos `hash` dispatch, hash-bucket candidate enumeration, Telos
     `==` dispatch, numeric SameValueZero comparisons, and receiver re-entry
     counters across Map, Set, MutableBag, IndexedDict, and OrderedSet.
   - Eliminates duplicate key-location logic in `primitive/map.rs` and `primitive/set.rs`
     while preserving existing Map and Set semantics without behavioral regression.
   - Enforces the strict rule that no live Heap borrow is held across user Telos
     equality or hash callbacks.

2. **`MutableBag<T>` (`heap/mutable_bag.rs`, `primitive/mutable_bag.rs`, `collections/mutable_bag.ph`)**:
   - Backed by `MutableBagObject` storing `Vec<MutableBagEntry>` with entry indices
     and cached bucket hashes.
   - Maintains exact Telos Int multiplicities, ensuring `total == sum(counts)`.
   - Prunes entries on zero count; handles hash collisions and re-entrant mutations;
     exposes iterator with owner-retention.

3. **`IndexedDict<K,V>` (`heap/indexed_dict.rs`, `primitive/indexed_dict.rs`, `collections/indexed_dict.ph`)**:
   - Backed by `IndexedDictObject` storing entries in insertion/positional order
     via `Vec<IndexedDictEntry>` alongside a hash index mapping keys to entry indices.
   - Disentangles key lookup from positional access (`entryAt`, `keyAt`, `valueAt`).
   - Implements atomic insertion: duplicate key rejects with `DuplicateKeyError` and
     invalid positional index rejects with `IndexError`, preserving dictionary state
     and ordering without partial mutation.
   - Obeys reentrant overwrite semantics where the outer write wins and commits
     its value while inner commits are not rolled back on outer failure.

4. **`OrderedSet<T>` (`heap/ordered_set.rs`, `primitive/ordered_set.rs`, `collections/ordered_set.ph`)**:
   - Backed by `OrderedSetObject` maintaining insertion/positional order with
     uniqueness deduplication.
   - Positional ordering participates in equality; candidate collisions remain
     distinct while preserving positional indices after removal or replacement.
   - Equality captures start-of-call logical content snapshots before running
     user comparisons.

5. **Native Floor (TDR-0092)**:
   - Registers exactly 29 approved native bindings: 9 for MutableBag, 10 for
     IndexedDict, and 10 for OrderedSet.
   - Verified by `phalcom-native-surface-gen -- --root . --check` at exactly
     458 catalog declarations.

6. **Central Mutable-Key Admission**:
   - Extended `primitive/mod.rs::is_mutable_collection_key` to classify
     `MutableBag`, `IndexedDict`, and `OrderedSet` alongside `List`, `Array`,
     `Buffer`, `Slice`, `Map`, `Set`, and `Bytes`.
   - Protects all 5 keyed families from admission of mutable collections as keys.
   - Guarantees `Vector<T, N>` and immutable scalars remain accepted.

---

## 2. Incident Investigations and Root-Cause Resolutions

### 2.1 Incident 1: Semantic Universe Hierarchy & Variant Expected Type
- **Symptom**: `IndexedDict.insert(entry, at:) -> Result<Unit, Error>` failed
  type checking when constructing concrete error classes `DuplicateKeyError` and `IndexError`.
- **Root Cause**: The immutable canonical Universe baseline in `phalcom-semantic`
  omitted source-authored superclass edges (`is Error`), and generic variant
  constructors required expected-type seeding prior to payload expression analysis.
- **Resolution**: Resolved in `phalcom-semantic` by sharing `resolve_source_class_superclass`
  between hierarchy queries and canonical baseline construction, and propagating
  variant-constructor expected types. Five hierarchy regressions and 15 generic
  application regressions pass.

### 2.2 Incident 2: `orderedCollision = false`
- **Symptom**: The combined P6 contract test reported `orderedCollision = false`.
- **Root Cause**: Test oracle defect in `phalcom-core/tests/core/collections/contract.rs`.
  Per normative specification (`docs/specs/collection-families.md` §4),
  `OrderedSet.at(_ index: Int) -> Option<T>`. The test compared `ordered.at(0) == first`,
  which evaluated `Option::Some(first) == first` (false).
- **Resolution**: Unwrapped the optional element via `.unwrapOr(fallbackPoint) == first`,
  confirming proper element equality and collision handling.

### 2.3 Incident 3: `mutableKeyPolicy = false`
- **Symptom**: The combined P6 contract test reported `mutableKeyPolicy = false`.
- **Root Cause**: Test oracle expectation defect. When rejecting mutable keys,
  `Map` reports `"Map key must be immutable..."`, whereas the test asserted it
  should contain `"MutableBag key must be immutable..."`.
- **Resolution**: Corrected the test oracle to match family-specific rejection
  messages. Added `p6_mutable_key_admission_matrix` verifying all 50 rejection
  pairs (5 keyed collections × 10 mutable collection types) and verifying Vector
  and scalar key acceptance.

---

## 3. Focused Verification Results

All Cargo commands were executed with `RUSTFLAGS=''` and the pinned toolchain.

| Gate | Subsystem / Test | Output / Result |
|---|---|---|
| Native Floor | `cargo run -p phalcom-native-surface-gen -- --root . --check` | 458 declarations, current & verified |
| Semantic Hierarchy | `cargo test -p phalcom-semantic --test semantic foundations::canonical_universe_source_hierarchy` | 5 passed, 0 failed |
| Semantic Generics | `cargo test -p phalcom-semantic --test semantic types::generic_application` | 15 passed, 0 failed |
| P6 Combined Contracts | `cargo test -p phalcom-core --test core collections::contract::tests::p6_mutable_hash_families_preserve_callback_snapshot_and_gc_contracts` | 21 assertions green, PASSED |
| Key Admission Matrix | `cargo test -p phalcom-core --test core collections::contract::tests::p6_mutable_key_admission_matrix` | 50 rejections + Vector acceptance, PASSED |
| IndexedDict (G4) | `cargo test -p phalcom-core --test core collections::indexed_dict` | 5 passed, 0 failed |
| OrderedSet (G5) | `cargo test -p phalcom-core --test core collections::ordered_set` | 5 passed, 0 failed |
| MutableBag (G3) | `cargo test -p phalcom-core --test core collections::mutable_bag` | 3 passed, 0 failed |
| Core Collections Module | `cargo test -p phalcom-core --test core core_collections` | 15 passed, 0 failed |
| Target Typecheck | `cargo check -p phalcom-core --all-targets` | 0 warnings, PASSED |
| Workspace Typecheck | `cargo check --workspace --all-targets` | exit code 0, PASSED |
| Scoped Formatting / Diff | `git diff --check` | clean |

---

## 4. Retained Residuals and Ownership

- **R-PARSE-ASSOCIATED-INVOKE**: Full-package execution in `modules_parse_package`
  remains blocked by the compiler associated-family invocation lowering fallback.
  This is an owning compiler-layer issue assigned to P9; it is unaffected by P6.
- **P4 D1/D2/D4**: Static constant computed Record-key, reflection, and metadata
  residuals remain inherited baseline items assigned to P9.
- **Language Corpus Baseline**: Pre-existing Range and corpus failures remain
  unrelated baseline items.

---

## 5. Lifecycle Disposition

`COLL006.C1.P6` is **COMPLETE / IMPLEMENTED / FOCUSED_TESTED**.

All gates G1–G7 are satisfied. The active plan transitions to `COLL006.C1.P7`
(Persistent hash substrate; `Bag<T>`; `Bijection<K,V>`).
