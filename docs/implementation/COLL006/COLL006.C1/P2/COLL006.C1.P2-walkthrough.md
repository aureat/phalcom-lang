---
id: COLL006.C1.P2
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: walkthrough
title: "Array<T> Runtime Fixed-Extent Sequence — Walkthrough"
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# COLL006.C1.P2 — `Array<T>` Runtime Fixed-Extent Sequence — Walkthrough

## 1. Plan identity and final state

- **Plan ID:** `COLL006.C1.P2`
- **Owning Checkpoint:** `COLL006.C1`
- **Status:** `COMPLETE`
- **Completion:** `IMPLEMENTED`
- **Verification:** `FOCUSED_TESTED`

## 2. Objective and outcome

P2 introduced the canonical Telos collection family `Array<T>`: an owned, homogeneous, ordered sequence whose extent is runtime state permanently fixed after construction, while elements remain mutable.

Outcomes achieved:
1. Canonical `UniverseKey::Array` established in `phalcom-native-meta` with nominal superclass `Collection`, arity 1 type parameter `T: Type`, and re-export in `universe.collections.array` / `collections/package.ph`.
2. Dedicated runtime class `array_class` under `collection_class`, registered in `CoreClasses`, census, and parallel metaclass tower.
3. Dedicated heap representation `ArrayObject` backed by fixed `Box<[Value]>` and `frozen` boolean flag; passes object-size budget.
4. Minimal native storage floor implemented in `phalcom-core/src/primitive/array.rs` (`rawLength`, `rawAt(_)`, `rawSet(_,_)`, `toString`), regenerated into `phalcom-native-surface/src/generated.rs`, with census updated to 239 bindings.
5. Telos-authored sequence protocol in `phalcom-core/core/universe/src/collections/array.ph`:
   - `size`, `first`, `last`, `at(_)`, `get(_)`, subscript read `[_]`, subscript assignment `[_]=`, `at(_, put:)`
   - same-family ordered structural equality `==`, `!=`
   - `ArrayIterator<T>` conforming to `CanonicalIterator` and `CanonicalIterable` with live-view iteration
   - strict absence of structural resize methods (`append`, `prepend`, `insert`, `remove`, `clear`)
6. Integrated with mutable-key policy: `is_mutable_collection_key` rejects `Object::Array` as `Map`/`Set` keys.
7. Activated `Array` in semantic sequence-literal resolution (`synthesize_bracket_literal`):
   - Contextual `let a: Array<Int> = [1, 2, 3]` selects family `Array` with element `Int`
   - Contextual hole `let a: Array<_> = [1, 2, 3]` infers `Int` and publishes `Array<Int>`
   - Explicit head `<Int>[1, 2, 3]` selects family `Array` with element `Int`
   - Explicit hole `<_>[1, 2, 3]` creates solver variable, infers element `Int`, and publishes `Array<Int>`
   - Empty typed literals `<Int>[]` and `let a: Array<Int> = []` succeed
   - Unconstrained empty holes `<_>[]` and `let a: Array<_> = []` emit `GenericInferenceUnderconstrained`
   - Bare unconstrained literals `[1, 2, 3]` remain `List` for P1/P4 backwards compatibility
8. Implemented direct fixed Array literal lowering via new `BuildArray(u16)` bytecode without `List` staging.
9. Implemented dynamic-spread Array construction via private `ArrayLiteralBuilderObject` (`BeginArrayLiteral`, `ArrayLiteralAppend`, `ArrayTryExpandTuplePositionals`, `FinishArrayLiteral`), GC-traced and finalized directly to `ArrayObject`.

## 3. Important files changed

### Canonical identity & Universe source
- `phalcom-native-meta/src/universe.rs`: added `UniverseKey::Array`, class relation, name, bindings, type forms
- `phalcom-modules/src/builtin.rs`: registered `universe.collections.array`
- `phalcom-core/core/universe/src/collections/package.ph`: exposed `.array`
- `phalcom-core/core/universe/src/collections/array.ph`: created canonical `Array<T>` declaration and protocol
- `phalcom-semantic/src/core_surface/identity.rs`: added `CoreDeclarationIds.array`
- `phalcom-semantic/src/types/store.rs`: added `array_of` helper

### Runtime & Primitives
- `phalcom-core/src/heap/array.rs`: created `ArrayObject`, `ArrayMutationError`, `ArrayLiteralBuilderObject`
- `phalcom-core/src/heap/object.rs`: added `Object::Array(ArrayObject)` and `Object::ArrayLiteralBuilder(Box<...>)`
- `phalcom-core/src/heap/mod.rs`: re-exported array types, added `alloc_array` and `alloc_array_literal_builder`
- `phalcom-core/src/heap/accessors.rs`: added `array`, `array_mut`, `as_array`, builder accessors
- `phalcom-core/src/heap/trace.rs`: added GC tracing for `ArrayObject` and `ArrayLiteralBuilderObject`
- `phalcom-core/src/universe/core_classes.rs`: wired `array_class` into class tower and `each_handle`
- `phalcom-core/src/universe/invariants.rs`: added `Array` to census
- `phalcom-core/src/universe/primitives.rs`: registered `rawLength`, `rawAt(_)`, `rawSet(_,_)`, `toString`
- `phalcom-core/src/primitive/mod.rs`: added `expect_array`, included `Object::Array` in `is_mutable_collection_key`
- `phalcom-core/src/primitive/array.rs`: created primitive implementations
- `phalcom-native-surface/src/generated.rs`: regenerated primitive metadata

### Bytecode, Compiler & VM
- `phalcom-core/src/bytecode.rs`: added `BuildArray`, `BeginArrayLiteral`, `ArrayLiteralAppend`, `FinishArrayLiteral`, `ArrayTryExpandTuplePositionals`
- `phalcom-core/src/vm/dispatch.rs`: implemented execution for new bytecodes and positional expansion
- `phalcom-core/src/compiler/lib/expr.rs`: implemented direct fixed Array lowering and dynamic spread builder lowering

### Semantic Analysis
- `phalcom-semantic/src/checker/expected.rs`: made `inference_origin_matches` `pub(crate)`
- `phalcom-semantic/src/checker/expression.rs`: updated `synthesize_bracket_literal` to select `Array` via head or expected type, solve explicit holes, and publish `SequenceLiteralFamily::Array`

### Verification
- `phalcom-semantic/tests/semantic/foundations/coll006_c1_p2_array.rs`: 10 focused semantic tests
- `phalcom-core/tests/core/collections/array.rs`: 10 focused runtime tests
- `phalcom-core/tests/core/object_model/invariants.rs`: census and tower verification

## 4. Verification evidence

- **Semantic Suite:**
  `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic coll006` -> 15 passed (10 P2 tests + 5 P1 tests), 0 failed.
- **Core Runtime Suite:**
  `RUSTFLAGS='' cargo test -p phalcom-core --test core core_array` -> 10 passed, 0 failed.
- **Workspace Build:**
  `RUSTFLAGS='' cargo check --workspace` -> clean, zero errors.
- **Git Diff:**
  `git diff --check` -> clean, zero whitespace/lint issues.
