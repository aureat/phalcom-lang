# Spec B — Map Runtime, Views, Interoperability, and Brace Literals

This implementation unit realizes Phalcom's ratified default `Map` semantics and the collection-literal boundary that can be implemented without importing the deferred typing, reflection, iterator, expansion, or full Set designs.

The semantic authority is the supplied `collections-next` specification set, especially `collection-literals-and-map-spec.md` and the Map-related rules in `collections-core-semantics-spec.md`. Repository-era `U-COLL`, `U-COLLTYPES`, `map-and-set.md`, and the existing collection contract are implementation history only where they disagree with the new specification.

Spec B assumes Spec A has landed. In particular it relies on canonical `Unit`, first-class Symbol labels, positive immutable Record, and the Record encounter-order observers specified by A.3.

## Repository diagnosis

The current repository is not a blank Map implementation. HEAD already contains:

- native `Object::Map` / `MapObject` storage;
- insertion-ordered `entries: Vec<(Value, Value, bucket)>` plus a hash-bucket index;
- re-entrant Phalcom `hash` + `==` lookup with a mutation lock;
- raw Map primitives for allocation, size, get, put, has, remove, key-at, and value-at;
- a `.ph` `Map` facade with `at`, `[]`, mutation, equality, copied `keys`/`values`, and iteration;
- parser support for the narrow `{ identifier: value }` Map form, currently desugared to repeated `Map.new().at(..., put: ...)` sends.

Several of those shipped choices are now semantically obsolete and must be migrated rather than preserved:

1. `MapObject::remove_at` uses `Vec::swap_remove`. Removing a non-final entry therefore moves the last entry into the removed slot and destroys first-insertion encounter order. The new specification makes removal order observable and requires all surviving entries to keep their relative order.
2. `Map::get_` currently returns the raw stored value on hit and the `None` singleton on miss. That representation cannot distinguish an absent key from a present key whose value is `None`; the new safe lookup contract requires `Some(None)` for the latter.
3. `put_` and `remove_` currently return the receiver. The new public `insert(value, for: key)` and `remove(key)` return `Option<previous-or-removed>`.
4. `keys` and `values` currently allocate copied `List`s. The new contract requires lightweight ordered views and also adds `entries` / immutable `Entry`.
5. The old reusable `collections_contract.rs` treats Map as though it were a sequence by building numeric keys and testing total `at(i)`. The new Map is an association collection: `map[key]` is strict, `map.get(key)` is safe, and integer keys—including negative integers—are ordinary keys. Map must leave that obsolete sequence harness.
6. The current Map literal parser accepts only bare identifier keys, cannot express computed keys, silently overwrites duplicate logical keys through repeated mutation sends, and cannot make literal construction atomically duplicate-safe.

## A real grammar blocker discovered on HEAD

The authoritative collection specification says:

```text
{}          -> empty Map
{a, b, c}   -> Set
{a: b}      -> Map
```

while the existing Phalcom language uses the same brace forms for zero-parameter block literals, including `{} ` and `{ expr }`. Those forms are syntactically identical. In particular, a context-independent parser cannot simultaneously make bare `{}` both an empty block and an empty Map, or `{x}` both a one-expression block and a singleton Set.

This is not a type-context problem and must not be hidden behind expected-type inference. It is a direct surface-grammar collision between two already-specified features.

Therefore Spec B deliberately orders the work so useful Map implementation is not blocked by that unresolved syntax collision:

- B.1 and B.2 are fully dispatchable now.
- B.3a, covering unambiguous association Map literals such as `{name: v}` and `{[expr]: v}`, is dispatchable after B.1.
- B.3b, covering `{}` as Map and bare-brace Set literals, is **BLOCKED on a language-level block-vs-collection grammar decision**. An implementer must not invent contextual heuristics, silently retire block forms, or adopt an unratified trailing-comma convention.

This blocker should be resolved before claiming the complete brace-literal taxonomy implemented. Until then `Map.new()`/`Map()` and `Set.new()`/`Set()` remain explicit construction escape hatches according to whatever constructor-call spelling HEAD supports after the relevant syntax units.

## Target architecture

```text
Map runtime
  MapObject
    ordered dense entries
    hash bucket -> entry slots
    stable removal
    re-entrant Phalcom hash/== lookup

  raw boundary
    get_    -> Option<V>
    put_    -> Option<V> previous
    remove_ -> Option<V> removed
    has_    -> Bool
    keyAt_ / valueAt_ / size_

  .ph public surface
    get(key)                  -> Option<V>
    map[key]                  -> V or KeyError
    insert(value, for: key)   -> Option<V>
    remove(key)               -> Option<V>
    clear                     -> Unit
    keys / values / entries   -> lightweight live ordered views
    ==                        -> order-insensitive mapping equality

Map literal
  -> explicit MapLiteral AST
  -> BeginMapLiteral (hidden ordinary Map on VM stack)
  -> for each entry: key -> value -> unique insert
  -> FinishMapLiteral
  -> finalized mutable Map
```

## Phase order

### B.1 — Map Runtime Contract Migration

Fix stable encounter order, make raw lookup/mutation absence-safe, install the strict/safe public Map API, add `KeyError`, add the `insert(value, for: key)` selector including the required `for:` label parsing support, and replace the obsolete sequence-style Map conformance tests with association-specific tests.

Expected primitive-floor delta: **0**. Existing Map primitive bindings are reused with corrected semantics.

Artifact: `B.1-map-runtime-contract-migration.md`.

### B.2 — Map Views, Entry, and Record Interoperability

Replace copied List projections with lightweight live views, add immutable semantic `Entry`, add the `entries` view, and implement explicit `Map.from(record: record)` using A.3 Record observers while preserving encounter order and semantic independence.

Expected primitive-floor delta: **0**. The views and Entry are ordinary `.ph` classes over existing Map/Record raw observations.

Artifact: `B.2-map-views-entry-record-conversion.md`.

### B.3 — Brace-Literal AST and Atomic Map Construction

Replace parser desugaring for Map association literals with explicit AST + compiler/runtime construction, add computed keys, static/dynamic duplicate rejection, lexical evaluation order, and expansion-ready AST structure. This document also specifies the hard gate for `{}` and Set literal classification caused by the existing block syntax collision.

Expected primitive-floor delta: **0**. The literal-builder instructions are VM bytecodes, not surface primitives.

Artifact: `B.3-brace-literals-and-atomic-map-construction.md`.

## Cross-phase invariants

After B.1, no ordinary Map operation may disturb the relative encounter order of surviving entries, and safe lookup must distinguish absence from stored `None`.

After B.2, `map.keys`, `map.values`, and `map.entries` must not eagerly copy the Map into Lists; they must preserve encounter order through lightweight views. `Map.from(record:)` must return a distinct mutable Map and must not mutate or alias the Record semantically.

After B.3a, Map association literals must no longer be mutation-send chains. Duplicate logical keys must fail rather than overwrite, computed keys must remain arbitrary values with no String-to-Symbol coercion, and source evaluation order must be lexical.

Nothing in B may settle mutation-during-view-iteration behavior or equal-but-nonidentical key retention.

## Deliberate exclusions

Spec B does **not** implement:

- full `Set` / `ImmutableSet` semantics;
- the final block-vs-Set/empty-Map grammar decision;
- `*` or `**` literal expansion execution (Spec F);
- target-sensitive `**Map` labeled expansion (Spec F);
- generic `Iterable<Entry>.toMap` / `toMap merging:` or association combinators (Spec D);
- general subscript assignment evaluation order / setter-result discarding (Spec C);
- negative sequence indexing or slicing (Spec C; Map integer keys are never normalized);
- mutation-during-view-iteration semantics;
- equal-but-nonidentical Map-key retention/replacement policy;
- full Set order/equality/hash API;
- immutable map families, future `HashMap`, or future `OrderedMap`;
- final printing/serialization/cycle policy;
- typing, generic inference, variance, Record-to-Map type joins, or reflection.

## Verification gate

Every phase must inspect actual HEAD immediately before editing. The mandatory completion gate is:

```sh
./scripts/verify.sh --full
```

Also run focused crate/test lanes while iterating, but do not claim a phase complete until the full gate passes. Any phase changing lexer/parser AST snapshots must accept/review the resulting snapshots intentionally rather than bulk-accepting unrelated changes.
