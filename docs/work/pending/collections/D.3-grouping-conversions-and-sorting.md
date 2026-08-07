# Spec D.3 — Grouping, Partitioning, Conversions, and Sorting Gate

Status: implementation specification. The grouping/partition/conversion lane is executable after D.1 + B.1/B.2. Sorting is design-ready but MUST remain unshipped until the narrowly listed ordering questions in §13 are ratified.

## 1. Mission

Implement the remaining eager higher-level operations whose semantics are already sufficiently pinned:

```text
group(by:)                -> Map<K,List<T>>
partition(where:)         -> Tuple(List<T>, List<T>)
toList                    -> List<T>
toSet                     -> Set<T>
toMap                     -> Result<Map<K,V>, DuplicateKeyError<K>>
toMap(merging:)           -> Map<K,V> or ordinary callback failure
```

Then record the exact repository integration plan for:

```text
sorted
sorted(on:)
sorted(using:)
List#sort
List#sort(on:)
List#sort(using:)
```

without inventing the still-open Ordering/default-comparison/key-evaluation semantics.

## 2. Preconditions

Expected landed substrate:

- A: canonical Unit and Tuple construction;
- B.1: stable ordered Map, safe `get`, strict `[]`, `insert(value,for:)` returning previous Option, duplicate-safe internal key identity;
- B.2: `Entry` with `.key` / `.value`, `map.entries`, Map views;
- D.1: eager traversal and ordinary one-value `each` contract;
- D.2 preferred: canonical List `append` returning Unit.

D.3 does not require Spec E's iterator stage objects. It consumes source eagerly through the ordinary current Iterable protocol.

## 3. Grouping

Canonical selector:

```phalcom
collection.group(by: |value| { key })
```

The specification's pretty form may place the labeled closure trailing; definition identity remains `group(by:)`.

Result:

```text
Map<K, List<T>>
```

### 3.1 Algorithm

```text
out = Map.new()
for value in source encounter order:
    key = callback(value)
    optionGroup = out.get(key)
    if present:
        existingGroup.append(value)
    if absent:
        group = List.new()
        group.append(value)
        out.insert(group, for: key)
return out
```

Do not use strict `out[key]` followed by exception handling for ordinary first occurrence.

Do not call `has` + `get`, which would hash/compare the grouping key twice.

### 3.2 Duplicate group keys are accumulation, not error

The first equivalent key creates one group and establishes its Map encounter position.

Every later equivalent key appends to that same group's List.

No DuplicateKeyError is produced by grouping.

B.1's unresolved equal-but-nonidentical retained-key-object policy remains unchanged. D.3 cares about logical key equality and first position, not which equal key object is stored internally after an update.

### 3.3 Order

Pin both:

```text
within group List -> source encounter order
Map key order     -> first-seen equivalent group-key order
```

Do not sort groups.

### 3.4 Failure

If key callback fails, propagate normally. Already-created output is not returned.

If the key is inadmissible for Map hashing, propagate Map's ordinary key error. Do not wrap in Result merely because grouping allocates a Map.

## 4. Partitioning

Canonical selector:

```phalcom
collection.partition(where: predicate)
```

Result is a two-element positional Tuple:

```text
(accepted, rejected)
```

where both are Lists.

Algorithm:

```text
accepted = List.new()
rejected = List.new()
for value in source encounter order:
    if predicate(value):
        accepted.append(value)
    else:
        rejected.append(value)
return (accepted, rejected)
```

Predicate is invoked exactly once per encountered value.

Both output Lists preserve source-relative encounter order.

Empty source returns two distinct empty Lists inside the Tuple, not Unit.

The Tuple itself has arity 2 and therefore does not normalize to Unit.

## 5. `toList`

Canonical getter:

```phalcom
iterable.toList
```

Eagerly traverse and append every encountered value to a fresh List in encounter order.

Retain the current implementation strategy but migrate it off `List#add` after D.2.

For a List receiver, the generic implementation may return a fresh copy rather than `self`; D.3 does not establish an identity-preserving conversion optimization. Do not make callers depend on identity.

For a provably unbounded Range/Iterator, compile-time rejection belongs to E. D.3 does not attempt local static analysis.

## 6. `toSet`

Canonical getter:

```phalcom
iterable.toSet
```

Implementation:

1. allocate the current mutable `Set` (`Set.new()` or canonical current spelling);
2. traverse source encounter order;
3. add each encountered value through the Set's existing membership/insertion substrate;
4. return the Set.

This phase does not define Set encounter-order, equality, hashing, ImmutableSet, or the final full Set API. It only realizes the ratified conversion vocabulary against the already-existing mutable Set runtime.

If a source element is inadmissible as a Set key/member under current Set rules, propagate that failure.

Do not introduce ImmutableSet as part of conversion work.

## 7. `DuplicateKeyError`

Duplicate-rejecting Map conversion requires the semantic error family:

```text
DuplicateKeyError<K>
```

The generic parameter is conceptual until the type system can express it.

If implementation HEAD does not already define it, add an ordinary `.ph` subclass of `Error`, following B/C's `KeyError`/`IndexError` pattern.

The exact structured fields/rendering are deferred, but the object SHOULD retain the conflicting key when current Error construction permits without disproportionate new infrastructure.

Do not raise DuplicateKeyError from plain `toMap`; it is the `Err` payload.

## 8. Duplicate-safe `toMap`

Canonical getter:

```phalcom
entries.toMap
```

Conceptual input:

```text
Iterable<Entry<K,V>>
```

Result:

```text
Result<Map<K,V>, DuplicateKeyError<K>>
```

### 8.1 Algorithm

For each encountered entry in source order:

1. evaluate/read `entry.key`;
2. evaluate/read `entry.value`;
3. call `out.insert(value, for: key)` exactly once;
4. inspect returned `Option<previous>`;
5. `None` -> first occurrence, continue;
6. `Some(previous)` -> duplicate equivalent key: return `Err(DuplicateKeyError(key))` immediately.

This works even when the previous mapped value is surface None because B.1 returns `Some(None)` for an existing key whose value is None.

Do not implement duplicate detection as `out.get(key) != None`.

### 8.2 Partial output on failure

`toMap` returns only Result; the partially constructed Map is not surfaced on duplicate failure.

Do not attempt rollback or transactional mutation: the Map is fresh and otherwise unreachable, so simply return Err.

### 8.3 Invalid entry values

If a source value does not provide the Entry `.key`/`.value` protocol, allow the ordinary language error to propagate. The collection semantic failure modeled by Result is duplicate-key conflict, not arbitrary type/callback failures.

If the type system later statically restricts this method to `Iterable<Entry<...>>`, that will catch misuse earlier without changing runtime selector semantics.

## 9. `toMap merging:`

Canonical selector:

```phalcom
entries.toMap(merging: |existing, incoming| { resolved })
```

When a key is first seen, insert its incoming value.

When an equivalent key is already present:

1. obtain existing value;
2. invoke merge callback once as `(existing, incoming)`;
3. write resolved value for the same logical key;
4. preserve the Map encounter position established by the first occurrence.

No DuplicateKeyError is returned merely for duplicates when a merge callback is supplied.

### 9.1 Return shape

The ratified text states explicit conflict resolution means conversion does not fail merely because a key repeats, but it does not require a Result wrapper for callback-independent success. Use the simplest already-ratified executable shape:

```text
Map<K,V>
```

Ordinary callback/key failures propagate through the language error model.

Do not wrap every callback failure in Result.

If another ratified conversion specification lands before implementation and pins a Result return for `merging:`, follow that newer source and record the delta.

### 9.2 Source order

Duplicate merges happen in source encounter order.

For values `v1,v2,v3` under one key:

```text
r1 = merge(v1,v2)
r2 = merge(r1,v3)
final = r2
```

Do not batch or reorder equal-key values.

## 10. Map Entry view integration

B.2's `map.entries` yields Entry objects lazily by encounter slot.

D.3 must make this round trip work:

```phalcom
const copied = map.entries.toMap
```

provided the source Map itself contains no duplicate logical keys (Map invariant).

The copied Map must preserve encounter order and values, including stored None.

Do not require Entry equality/hash/destructuring to implement this conversion; `.key` and `.value` are sufficient.

## 11. `associate key:value:` remains out of scope

The ratified collection document gives a direction for:

```phalcom
users.associate(
    key: ...,
    value: ...,
)
```

and pins duplicate rejection as the default principle, but explicitly says exact final API is subject to the broader conversion specification.

Do not ship it in D.3.

A future spec can build it directly over the same duplicate-safe Map insertion seam used by `toMap`.

## 12. Sorting vocabulary to preserve

The ratified names are:

```phalcom
collection.sorted
collection.sorted(on: keyExtractor)
collection.sorted(using: comparator)

list.sort
list.sort(on: keyExtractor)
list.sort(using: comparator)
```

Semantic distinction already pinned:

```text
sorted -> new sorted collection
sort   -> mutate List, return Unit
```

Direct comparator result type is `Ordering`, not negative/zero/positive Number.

Do not replace these names with `orderBy`, `sortBy`, comparator integers, or closure-arity inspection.

## 13. Sorting MUST remain gated on three unresolved details

The supplied ratified specification deliberately leaves enough open that shipping now would invent observable semantics.

### 13.1 Ordering case surface

Conceptual type is:

```text
Ordering
    less
    equal
    greater
```

but exact enum/variant case spelling/casing is delegated to global enum-style decisions.

Current repository examples use PascalCase `@variant` names, but D.3 must not assume that automatically makes `Less.new()`/`Equal.new()`/`Greater.new()` the final public Ordering API, especially inside kernel `core.ph` bootstrap.

Ratify the user-visible production/matching spelling first.

### 13.2 Default comparison protocol

`collection.sorted` means default ordering "where available", but the supplied collection spec does not pin the message/protocol used to obtain that ordering.

Do not choose among:

```text
<=> returning Ordering
compare(_) returning Ordering
combining < and == sends
some future Comparable protocol
```

These choices affect user override behavior and dispatch identity.

### 13.3 `sorted on:` key-extraction evaluation count

The spec says key extraction uses a closure, but does not state whether it is:

```text
called once per source element (decorate-sort-undecorate)
```

or:

```text
called whenever the sorting algorithm compares two elements
```

This changes callback side effects, failure timing, and asymptotic callback count. It is observable and must be ratified.

### 13.4 Not blockers

Sort stability is explicitly deferred. Once §§13.1–13.3 are pinned, the first implementation may leave equal-key relative order unspecified unless a later decision settles stability.

Algorithmic complexity/performance can also evolve as long as callback evaluation semantics remain fixed.

## 14. Sorting implementation architecture after the gate

Once §13 is resolved, prefer `.ph` control for callback-taking sort variants rather than a Rust primitive that repeatedly re-enters user comparators.

A simple first implementation may:

1. materialize generic `Iterable.sorted*` to a fresh List;
2. call the corresponding List in-place sort implementation;
3. return that List;
4. implement List `sort*` over existing indexed get/set operations;
5. return Unit for in-place success.

This means the initial generic `sorted` family returns List, consistent with D.1's executable generic materialization rule and the deferred family-preservation matrix.

Do not add native comparator-calling sort primitives merely to get Rust's `slice::sort_by`: that would introduce re-entrant sends/native-frame/yield behavior and make key-extraction timing easy to get wrong.

If profiling later justifies native sorting, preserve the already-ratified callback-count/failure-order semantics exactly.

## 15. Group/partition/conversion callback behavior

For every callback-taking executable operation in D.3:

- no arity inspection;
- ordinary language call;
- ordinary errors propagate;
- no automatic Result wrapping except the operation-specific duplicate semantic modeled by `toMap`;
- encounter order determines callback order;
- no hidden parallelism.

## 16. Boundedness relationship to E

The following are eager exhaustors:

```text
group
partition
toList
toSet
toMap
toMap merging:
sorted (once enabled)
```

D.3 implements runtime behavior for sources that exhaust.

Spec E later adds source boundedness metadata and compile-time rejection of **provably unbounded** eager exhaustors.

Do not add ad-hoc Range checks inside each D.3 method. Boundedness is a cross-cutting compiler/iterator property and belongs in E.

Unknown-boundedness sources remain legal even after E according to the ratified model.

## 17. Tests — grouping

Cover:

- empty source -> empty Map;
- all unique keys;
- repeated group key;
- first-seen key order;
- group member encounter order;
- group callback called exactly once per source element;
- group key/value surface None where hash/key rules allow;
- callback failure propagates.

Use keys with intentional hash collision if existing test infrastructure makes this easy, proving B's logical equality path rather than Rust bucket identity still governs grouping.

## 18. Tests — partition

Cover:

```text
empty -> ([],[])
all true
all false
mixed
stored None
predicate failure
```

Assert exact Tuple order accepted then rejected, distinct List objects, stable order, and one predicate call per value.

## 19. Tests — conversions

### 19.1 toList

- encounter-order copy;
- source List yields equivalent fresh List;
- source Map view variants preserve their defined encountered values;
- source containing None.

### 19.2 toSet

- duplicate source values collapse according to existing Set semantics;
- invalid mutable/hash-key value failure propagates according to Set.

Do not assert final Set encounter order/equality beyond already-landed current contracts.

### 19.3 toMap

- unique entries -> Ok(Map);
- duplicate equal key -> Err(DuplicateKeyError);
- duplicate where previous value is None still detected;
- first duplicate short-circuits later source elements;
- Map entries round trip;
- invalid entry object ordinary error, not DuplicateKeyError.

### 19.4 merging

- unique entries never call merge;
- one duplicate calls merge once with `(existing,incoming)`;
- three values fold in encounter order;
- first key position preserved;
- callback failure propagates.

## 20. Sorting pending tests

Create pending/ignored semantic tests or a dedicated design-test checklist for:

- exact Ordering value construction/matching;
- comparator rejecting Number `-1/0/1` return;
- default protocol override dispatch;
- key extractor callback count;
- List `sort*` exact Unit return;
- `sorted*` non-mutation of original;
- equal-key stability explicitly **not asserted** until decided.

Do not activate these tests by choosing temporary semantics just to make them green.

## 21. Repository migration audit

Executable D.3 work should search for historical/custom versions of:

```text
group
partition
toList
toSet
toMap
associate
sort
sorted
```

Also inspect examples that manually build Maps from `Entry` lists and replace only when selector semantics truly match.

Do not rewrite application-specific `groupBy` helpers if they intentionally return a different shape.

## 22. Expected write set

Executable lane likely touches:

```text
phalcom-core/core/core.ph
phalcom-core/tests/lang/**
possibly current error class declarations/docs
examples/** as migration requires
```

No parser/compiler/heap/native primitive changes should be necessary for grouping/partition/conversions once A/B/D.1/D.2 are landed.

Sorting, after its gate resolves, should still initially be `.ph`-only unless the separately ratified Ordering representation requires compiler support already absent on HEAD.

## 23. Primitive-floor accounting

Executable grouping/partition/conversion lane:

```text
+0 native primitive bindings
```

`DuplicateKeyError` is an ordinary Error subclass.

Sorting target architecture after semantic ratification also expects:

```text
+0
```

Do not enlarge the primitive floor for convenience.

## 24. Completion gate

D.3 executable lane is complete when:

- `group(by:)` returns ordered Map-of-Lists with correct duplicate accumulation;
- `partition(where:)` returns `(accepted,rejected)` and preserves order;
- `toList` and `toSet` eagerly materialize;
- `toMap` rejects duplicates through Result and correctly handles stored None;
- `toMap(merging:)` resolves duplicates in encounter order without repositioning the key;
- no callback failure is auto-wrapped except duplicate semantic failure;
- no primitive floor delta occurs;
- sorting selectors remain pending unless all three §13 questions were separately ratified;
- `./scripts/verify.sh --full` passes.
