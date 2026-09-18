---
id: LANG005.C6.P5
kind: implementation-handoff
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
revision: 715d0b16f3938495bc20c159ce4e1fbbcd699a22
prepared: 2026-09-18
---

# LANG005.C6.P5 Handoff: Production Stateful Iteration & T14 Migration

## 1. Executive Summary & Current State

- **Active Plan**: `LANG005.C6.P5` (Production Stateful Iteration, Evidence-Driven `for`, Iterator Toolbox, and Cursor Retirement).
- **Incident & Blocker (Fix 1) — RESOLVED & VERIFIED**:
  - **Issue**: Invoking instance methods on `ListIterator<T>` / `SetIterator<T>` instantiated unparameterized collection frames against an empty parent type environment (`parent []`). The VM's `resolve_call_environment` raised a generic call-entry type recipe instantiation error.
  - **Root Cause**: `instantiate_type_ref` in `phalcom-core/src/typing/environment.rs` returned `None` when `TypeNode::Parameter(param)` was missing from `env`.
  - **Fix Applied**: `instantiate_type_ref` now degrades unbound parameters to `Dynamic` (`RuntimeOverlayTypeNode::Special("Dynamic")`), which is the semantically correct erasure fallback for unspecialized runtime frames.
  - **Verification**: `cargo test -p phalcom-core --test core language::iteration` went from **5 passing / 6 failing** to **11 passing / 0 failing / 2 ignored baseline**.
- **Active Task**: **Task T14** (Retire legacy runtime `Iterable`/`Iterator` classes and reparent collections to intermediary abstract `Collection` class).

---

## 2. In-Flight Work: Task T14 & Intermediary `Collection` Hierarchy

The user approved introducing an intermediary abstract `Collection` class between `Object` and the concrete collections (`List`, `Map`, `Set`, `Tuple`, `Range`, `Bytes`).

### Target Hierarchy
```text
Object
 └── Collection   (abstract, UniverseKey::Collection)
      ├── List
      ├── Map
      ├── Set
      ├── Tuple
      ├── Range
      └── Bytes
```
*(Note: `Record` remains directly under `Object` as it is an immutable product representation without iteration).*

### Completed in T14 so far:
1. `phalcom-native-meta/src/universe.rs`:
   - Replaced `UniverseKey::Iterable` with `UniverseKey::Collection`.
   - Updated `UNIVERSE_CLASS_RELATIONS` to wire `Collection` under `Object`, and `List`, `Map`, `Set`, `Tuple`, `Range`, `Bytes` under `Collection`.
   - Updated `UNIVERSE_BINDINGS` to export `Collection` (`prelude = true`).
   - Updated `name()` for `UniverseKey::Collection`.

### Remaining Steps to Complete T14:
1. **Finish `phalcom-native-meta/src/universe.rs`**:
   - Update `source_path()`: map `Self::Collection => &["collections", "list"]` (or `&["collections", "package"]`).
   - Update `from_name()`: map `"Collection" => Some(Self::Collection)`.
2. **Update Core Classes & Runtime Invariants**:
   - `phalcom-core/src/universe/core_classes.rs`:
     - Rename `iterable_class` -> `collection_class` across `CoreClasses` struct, `make_core_class`, `create_core_classes`, `resolve`, `each_handle`.
     - Wire `collection_class = make_core_class(heap, "Collection", object_class, metaclass_class)` with `is_abstract = true`.
     - Wire `list_class`, `map_class`, `set_class`, `tuple_class`, `range_class`, `bytes_class` with superclass `collection_class`.
   - `phalcom-core/src/universe/invariants.rs`: Replace `("Iterable", c.iterable_class)` with `("Collection", c.collection_class)`.
   - `phalcom-core/src/universe/primitives.rs`: Replace `classes.iterable_class` with `classes.collection_class`.
   - `phalcom-core/src/vm/bootstrap.rs`: Replace `add_class!(iterable_class)` with `add_class!(collection_class)`.
3. **Update Source `.ph` Declarations in `phalcom-core/core/universe/src/collections/`**:
   - `list.ph`: `class List<T> is Collection`
   - `map.ph`: `class Map<K, V> is Collection`
   - `set.ph`: `class Set<T> is Collection`
   - `tuple.ph`: `class Tuple is Collection`
   - `range.ph`: `class Range is Collection`
   - `bytes.ph`: `class Bytes is Collection`
   - `iterable.ph`: Delete or remove obsolete cursor-era classes (`class Iterable`, `IndexedIterable`, `ZippedIterable`, `class Iterator`, `SourceIterator`, `MapIterator`, `FilterIterator`, `SkipIterator`, `TakeIterator`, `TakeWhileIterator`, `FlatMapIterator`).
   - `package.ph`: Export `Collection` and ensure `Iterable` / `Iterator` public names resolve directly to `iterator_laws.ph` traits.
4. **Update Legacy Use Sites**:
   - `examples/sheetcalc/src/grid/ref-range.ph`: update `is Iterable` to `is Collection` or trait conformance.
5. **Verification Commands**:
   - `RUSTFLAGS='' cargo check -p phalcom-native-meta`
   - `RUSTFLAGS='' cargo check -p phalcom-core`
   - `RUSTFLAGS='' cargo test -p phalcom-core --test core language::iteration`
   - `RUSTFLAGS='' cargo test -p phalcom-core --test core language::derived_conformance`
   - Final audit: `rg -n 'iterable_class|class Iterable|class Iterator|SourceIterator' -g '!target/**'`

---

## 3. Key Invariants & Architectural Rules

- **Semantic Authority**: Conformance proofs and associated type projections stay in `phalcom-semantic`.
- **Runtime Proof Free**: The VM executes pre-resolved recipes; no dynamic trait search or conformance proof occurs in bytecode dispatch.
- **Dynamic Fallback**: Unbound type parameters at call sites degrade gracefully to `Dynamic` overlay types rather than panicking.
- **Stateful Iteration**: All iterators own traversal state and adhere to sticky exhaustion on `None`.

