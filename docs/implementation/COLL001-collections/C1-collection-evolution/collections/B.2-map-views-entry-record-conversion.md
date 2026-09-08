# Spec B.2 — Map Views, Entry, and Record Interoperability

Status: implementation specification. Requires B.1 and Spec A.3 landed. This phase is pure surface/core-library work over native Map and Record observations unless actual HEAD reveals a missing underivable operation.

## 1. Mission

Replace eager copied Map projections with lightweight ordered views, introduce the immutable semantic `Entry` value exposed by `map.entries`, and implement the explicit Record-to-Map conversion:

```phalcom
Map.from(record: record)
```

All behavior must preserve Map encounter order and Record encounter order while keeping the resulting Map semantically independent and mutable.

Do not implement general `Iterable<Entry>.toMap`, expansion, destructuring, or mutation-during-view-iteration policy here.

## 2. Required semantic outcomes

At the end of B.2:

```text
map.keys
map.values
map.entries
```

are lightweight ordered view objects, not copied Lists.

For a Map whose encounter order is:

```text
K0, K1, K2
```

views yield:

```text
keys:    K0, K1, K2
values:  V0, V1, V2
entries: Entry(K0,V0), Entry(K1,V1), Entry(K2,V2)
```

`Entry` exposes:

```phalcom
entry.key
entry.value
```

and does not expose mutation selectors.

For Record conversion:

```text
Record fields in encounter order
    -> Symbol Map keys in same initial encounter order
    -> same field values
    -> fresh mutable Map
```

Mutating the Map does not mutate the Record.

## 3. Repository baseline

HEAD's `class Map` currently implements `keys` and `values` by allocating `List.new()` and copying each `keyAt_` / `valueAt_` slot. No `entries` view exists.

A.3 establishes Record's required observation floor:

```text
Record::size_
Record::labelAt_(i)
Record::valueAt_(i)
```

Those operations are exactly enough for `Map.from(record:)`. Do not add a Record-to-Map native conversion primitive.

The existing `Iterable` class already supplies cursor iteration from `size` plus `iteratorValue(cursor)`, so the simplest view design can remain pure `.ph` and reuse that protocol.

## 4. Implement views as ordinary `.ph` classes

Prefer ordinary heap instances authored in `phalcom-core/core/core.ph` rather than adding `Object::MapKeysView`, `Object::MapValuesView`, or `Object::Entry` native variants.

Suggested classes:

```text
MapKeysView
MapValuesView
MapEntriesView
Entry
```

The exact names may follow repository core naming conventions but must be stable user-visible class names if surfaced through `.class` or errors.

### 4.1 Why ordinary instances

A view needs only one retained `Map` value and behavior expressible via existing sends. An `Entry` needs only key/value fields and getters. The ordinary class/field/GC machinery already handles those references.

Adding native variants would enlarge:

- `Object` exhaustive matches;
- GC tracing;
- class dispatch plumbing;
- primitive floor;
- heap accessors;

for no underivable state. Do not do that.

## 5. `MapKeysView`

Conceptual implementation:

```phalcom
class MapKeysView is Iterable {
  @constructor
  new(map) { _map = map }

  size => _map.size

  iteratorValue(cursor) => _map.keyAt_(cursor)
}
```

If `Iterable.iterate` on HEAD is already the generic `0..size` cursor, inherit it. Do not duplicate cursor advancement.

The view is live in the minimal sense that `size` and slot reads delegate to the current Map rather than a copied snapshot. **Do not specify what happens if the Map is structurally mutated during an active view iteration.** That remains deferred.

The view itself must not provide Map mutation methods.

## 6. `MapValuesView`

Same structure, with:

```phalcom
iteratorValue(cursor) => _map.valueAt_(cursor)
```

It must preserve exactly the same encounter slots as keys.

A stored `None` is yielded as the ordinary surface `None` value; view traversal is not safe-lookup and therefore has no Option wrapping.

## 7. `Entry` and `MapEntriesView`

### 7.1 Entry

Implement as an ordinary immutable-by-surface `.ph` value:

```phalcom
class Entry {
  @constructor
  new(key, value) {
    _key = key
    _value = value
  }

  key => _key
  value => _value
}
```

Do not add setters.

Do not decide `Entry#==`, `Entry#hash`, tuple equivalence, printing format, or destructuring protocol in B.2 unless another already-ratified repository contract requires them. The collection specification requires semantic key/value access, not a full value-object protocol here.

### 7.2 Entries view

Conceptually:

```phalcom
class MapEntriesView is Iterable {
  @constructor
  new(map) { _map = map }

  size => _map.size

  iteratorValue(cursor) => Entry.new(
    _map.keyAt_(cursor),
    _map.valueAt_(cursor),
  )
}
```

Creating an `Entry` lazily per visited slot is acceptable. Do not pre-materialize all entries.

A later optimization may reuse entry cursors or another representation if it remains observationally equivalent.

## 8. Rewrite Map projection properties

Replace current copied List implementations with view allocation:

```phalcom
keys    => MapKeysView.new(self)
values  => MapValuesView.new(self)
entries => MapEntriesView.new(self)
```

Repeated property reads may return distinct lightweight view objects. The spec does not require view identity canonicalization.

Do not retain hidden copied-List compatibility behind these names. Code that truly needs a List can explicitly materialize through the ordinary iterable conversion once that surface is available on HEAD, e.g. `map.keys.toList` where supported.

## 9. Record-to-Map conversion

### 9.1 Canonical surface

Install class-side:

```phalcom
Map.from(record: record)
```

on the native `Map` row through the normal `core.ph` stub-completion mechanism.

This is a labeled selector. `record:` is an ordinary identifier label and requires no additional parser rule beyond existing label support.

### 9.2 Validation

The argument must be a Record value. If it is not, raise `ArgumentError` (or the current repository's canonical argument-contract error) rather than relying on a raw primitive panic.

Remember that `#{}` is Unit after Spec A, not an observable zero-field Record object. Therefore:

```phalcom
Map.from(record: #{})
```

passes Unit at runtime. The product specification says closed empty Record syntax normalizes to Unit, so there is no distinct empty Record to inspect. B.2 must choose the behavior consistent with that canonicalization:

- accept `Unit` as the zero-field Record form and return a new empty mutable Map;
- accept positive `Record` normally;
- reject unrelated values.

Do not require a hidden empty Record allocation merely to make this API convenient.

This special case is semantic canonicalization, not a type-system rule.

### 9.3 Construction algorithm

Conceptually:

```text
m = Map.new()
for i in 0 .. record.size_:
    key = record.labelAt_(i)   # already a Symbol
    val = record.valueAt_(i)
    insert val for key
return m
```

For Unit, skip the loop and return a fresh empty Map.

Use ordinary Map insertion so the result inherits B.1 key semantics and initial encounter order. Record fields are unique by construction, so duplicate conflict cannot occur.

Avoid converting labels to Strings and back; the Record already stores canonical Symbols.

### 9.4 Semantic copy, implementation freedom

The result must be independently mutable:

```phalcom
const r = #{ name: "Ada" }
const m = Map.from(record: r)
m[#name] = "Grace"
```

must not alter `r`.

The specification permits copy-on-write/shared immutable backing as a future optimization, but B.2 should not implement it. Current Record and Map layouts are different enough that a straightforward O(n) semantic copy is smaller and easier to verify.

Do not introduce shared backing solely because the specification says it is allowed.

## 10. Encounter-order tests

Construct a Record in a deliberate nonalphabetic order, e.g. fields `c`, `a`, `b`, convert it, and assert Map keys iterate:

```text
#c, #a, #b
```

Then update `#a` and assert the order remains unchanged. Remove/reinsert `#c` and assert B.1 behavior moves it to the end.

This proves conversion establishes initial order but the resulting Map subsequently follows ordinary Map mutation semantics.

## 11. View tests

### 11.1 Not copied Lists

Assert:

- `map.keys.class` is the view class, not `List`;
- same for values and entries;
- view `size` matches Map size;
- iteration/order is correct.

Do not use internal allocation counts as the language contract; the relevant rule is that the properties are lightweight views, not eager Lists.

### 11.2 Live observation, without settling active-iteration mutation

It is safe to test:

```text
view = map.keys
mutate map while no iteration is active
then iterate/read view
```

and require it observes current Map contents/order. This follows the view retaining Map rather than a snapshot.

Do **not** add a conformance test for mutation halfway through an active `for` over the view. That exact behavior remains deferred.

### 11.3 Entries

For each emitted Entry assert `.key` and `.value` pair correspond to the same encounter slot. Include a stored `None` value.

## 12. Repository migration audit

Search current repository uses of:

```text
.keys
.values
```

on Maps. Existing code may assume a `List` and call List-only mutation/indexing APIs.

For each hit:

- if only iteration/size/combinators are used, the view should remain source-compatible through `Iterable`;
- if a real List is required, materialize explicitly;
- if code mutates the returned collection assuming Map stays unchanged, preserve that intent by converting to a List first rather than making the view mutable.

Do not add List-specific methods to the view classes to avoid updating stale callers.

## 13. Files expected to change

Likely:

```text
phalcom-core/core/core.ph
phalcom-core/tests/lang/collections/...
phalcom-core/tests/... Map runtime/view tests
examples/benchmarks that assumed keys/values are Lists
relevant docs/status files required by repository process
```

No new native file, heap variant, bytecode, or primitive is expected.

If implementation discovers that ordinary `.ph` classes cannot retain a native Map through fields, that is a bug in ordinary object/GC behavior to fix at the appropriate abstraction, not a reason to make view objects native.

## 14. Explicit non-goals

Do not implement:

- active mutation-during-view-iteration policy;
- `Entry` destructuring;
- `Entry` full equality/hash/printing policy;
- `entries.toMap` / `toMap merging:`;
- grouping/association constructors;
- `**` expansion of Map/Record;
- Record update/merge/field-access syntax;
- copy-on-write Record/Map backing;
- generic typing/inference/reflection.

## 15. Completion gate

Run:

```sh
./scripts/verify.sh --full
```

Implementation report must include:

- concrete view class definitions;
- proof they are not List copies;
- Entry construction/access shape;
- `Map.from(record:)` handling of positive Record and canonical Unit/`#{}`;
- encounter-order conversion tests;
- repository `.keys`/`.values` migration summary;
- primitive-floor delta, expected `0`;
- final verification tail.
