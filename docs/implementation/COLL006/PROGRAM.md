---
id: COLL006
category: COLL
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
---

# COLL006 — Extended collection semantics, storage, and contextual sequence literals

COLL006 owns Telos's coherent extension of collection semantics across source syntax, contextual typing and inference, static evaluation, compiler lowering, native storage, collection protocols, persistent data structures, and standard-library collection APIs.

The current program state is `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED`. P1–P4 are complete at their bounded implementation scope; P5 Slice/Buffer has completed range/boundary, cross-family mutation, key, GC, representation, and conversion gates. `ListSource` has migrated to Array + Slice and P14 records are reconciled, but the parser runtime gate remains blocked by unchanged pre-P5 parse-package failures. The inherited computed-Record-key static-constant failures are tracked as D4 in the checkpoint. The checkpoint record owns the detailed state and remaining gates.

## 1. Program ownership

COLL006 owns implementation work for:

- contextual homogeneous bracket literals;
- inference holes used by contextual collection construction;
- `Array<T>`;
- explicit dynamically-resizable `List<T>` semantics after the literal cutover;
- `Vector<T, N>`;
- static homogeneous sequence values;
- `Slice<T>` and `Buffer<T>`;
- `Entry<K, V>`;
- `Bag<T>` and `MutableBag<T>`;
- `IndexedDict<K, V>` and `OrderedSet<T>`;
- `Bijection<K, V>`;
- `Bound<T>`, `Interval<T>`, and `RangeMap<K, V>`;
- reusable ordered-hash, persistent-hash, and persistent-ordered storage foundations;
- collection-specific compiler/runtime/GC/performance certification;
- migration from legacy bare-bracket-`List` semantics.

COLL006 consumes, rather than duplicates, the canonical Telos type/relation engine, generic inference solver, C4 static-value/materialization authority, `Collection` superclass, and existing `Iterator` / `Iterable` protocols.

## 2. Ratified semantic direction

```text
List<T>
    homogeneous
    mutable
    dynamically resizable
    runtime extent
    explicitly requested rather than the default bracket realization

Array<T>
    homogeneous
    mutable elements for ordinary runtime values
    fixed extent after construction
    runtime extent
    contiguous owned storage
    may also be explicitly requested as a const/static Array

Vector<T, N>
    homogeneous
    immutable
    exact static extent N
    contiguous owned value storage
    valid both as a runtime value and as a const/static value
```

Bracket literals become contextual homogeneous-sequence literals:

```telos
let runtimeDefault = [1, 2, 3]
// Array<Int>

const staticDefault = [1, 2, 3]
// Vector<Int, 3>

let dynamic: List<_> = [1, 2, 3]
// List<Int>

const explicitArray: Array<_> = [1, 2, 3]
// static Array<Int>

let explicitArray2 = <_>[1, 2, 3]
// Array<Int>

let explicitVector = <_, _>[1, 2, 3]
// Vector<Int, 3>
```

`_` is an inference hole. It is not `Dynamic`, `Any`, an existential wildcard, or an ignored argument.

## 3. Checkpoints

### COLL006.C1 — Extended collections foundation and first-generation collection family

**State:** `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED`

C1 owns the first complete extended-collections generation:

- neutral/contextual bracket-literal semantics;
- inference-bearing partial collection annotations;
- Array/List/Vector runtime and static semantics;
- direct compiler lowering for resolved collection literals;
- static sequence evaluation/materialization, including constant Array;
- Slice and Buffer;
- reusable ordered-hash storage and mutable ordered collection derivatives;
- persistent hash storage, Bag, and Bijection;
- generic interval values, persistent ordered storage, and RangeMap;
- migration, tests, GC and performance certification.

C1 is intentionally decomposed into multiple numbered plans. A new checkpoint must not be created merely because one C1 plan proves incomplete or requires corrective follow-up.

No later COLL006 checkpoint is defined by this record.

## 4. Intended C1 dependency shape

```text
P1  contextual literal + inference foundation
 │
 ├──────────────┐
 ▼              ▼
P2             P3
Array          Vector
 └──────┬───────┘
        ▼
        P4
 static sequences +
 bracket default cutover
        │
        ▼
        P5
 Slice / Buffer

P1 ─────────────► P6 ordered-hash family
                    │
                    ▼
                   P7 persistent-hash family

P1 ─────────────► P8 Interval / RangeMap

P2–P8 ──────────► P9 convergence/certification
```

Numbered plans are intent, not implementation truth. The canonical shared state for C1 is `COLL006.C1-CHECKPOINT.md`.

## 5. Cross-program boundaries

- `LANG007.C4.P3` Pack/rest semantics are a prerequisite for exact positional/cardinality reasoning used by Vector construction.
- C4 remains the authority for static-value identity, static evaluation, residual semantics, metadata durability, and static-to-runtime materialization rules.
- COLL006 may extend those C4 seams with collection-specific static values but must not create a second compile-time VM or static identity graph.
- Existing Map/Set hash/equality re-entry rules remain authoritative until a separate language-wide hashability/order trait upgrade is ratified.
- Existing `Iterator` and `Iterable` are reused.
- `Collection` remains the nominal superclass and must not be duplicated as a trait.

## 6. Program-wide architecture rules

1. Parser owns bracket syntax, not collection-family selection.
2. Semantic analysis is the sole authority selecting `Array`, `List`, or `Vector`.
3. Compiler lowering consumes checked semantic products and does not reconstruct collection semantics.
4. Evaluation phase influences defaults but does not replace expected-type reasoning.
5. Explicit literal heads and unique explicit context outrank phase defaults.
6. Multiple viable contextual collection families are ambiguous; phase defaulting is not an overload tie-breaker.
7. Runtime bare brackets default to `Array<T>`.
8. Static bare brackets default to `Vector<T, N>`.
9. `<T>[...]` explicitly selects `Array<T>`.
10. `<T, N>[...]` explicitly selects `Vector<T, N>`.
11. `List<T>` remains the general dynamically-resizable homogeneous sequence and is explicitly requested after cutover.
12. `Vector<T, N>` is immutable in every phase.
13. Constant Array is supported without a separate `ConstArray` classifier.
14. No unsolved inference variable may escape into durable semantic state.
15. Persistent updates never mutate storage reachable from older public roots.
16. Native collection implementations preserve precise GC tracing and re-entrant Telos hash/equality/ordering safety.

## 7. Lifecycle

Program lifecycle state is summarized in `STATUS.md`.

Checkpoint state, plan progress, durable architecture, verification evidence, consultations, deferred failures, and next action are owned by the checkpoint record:

```text
COLL006.C1-CHECKPOINT.md
```

Numbered plans describe intended work; walkthroughs record what actually shipped and was proved; handoffs carry the smallest complete takeover state.
