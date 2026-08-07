# Spec D.1 — Eager Traversal, Queries, and Reduction

Status: implementation specification. Intended to run after Spec C; requires canonical Unit from A. B.2 should be landed first so Map pair traversal can migrate cleanly to `map.entries`.

## 1. Mission

Replace the repository's now-obsolete mixed eager/lazy Iterable surface with the ratified eager concrete-operation semantics.

D.1 establishes this public model:

```text
concrete collection receiver
    map/filter/flatMap -> execute now -> concrete result

explicit iterator receiver
    lazy stages        -> Spec E, not D
```

It also replaces the historical accumulation/query selectors with the ratified vocabulary:

```text
map(f)                       -> List
map(indexed: f)              -> List
filter(f)                    -> List
flatMap(f)                   -> List
each(f)                      -> Unit
each(indexed: f)             -> Unit
find(where: f)               -> Option<T>
index(where: f)              -> Option<Int>
any(where: f)                -> Bool
all(where: f)                -> Bool
none(where: f)               -> Bool
count(where: f)              -> Int
fold(initial: a, using: f)   -> A
reduce(using: f)             -> Option<T>
```

The complete cross-family return matrix is deferred. The generic eager transform implementation in this phase materializes Lists.

## 2. Repository behavior to replace

Current `phalcom-core/core/core.ph` has all of these legacy behaviors:

- `Iterable#map(f)` returns `MapView.new(self,f)` lazily;
- `filter(pred)` returns an eager List;
- `reduce(init,f)` is an explicit-initial fold;
- queries are positional (`all(f)`, `any(f)`, `count(f)`, `find(f)`);
- `where`, `skip`, and `take` allocate lazy view classes directly from a concrete collection receiver;
- `Map#each(f)` overrides the generic selector but passes two callback arguments `(key,value)`;
- `each` does not deliberately return canonical Unit;
- `flatMap`, `none where:`, `index where:`, indexed callback variants, and no-initial `reduce` do not exist in the ratified shape.

Do not preserve these contradictions under aliases just to reduce test churn. This is a language migration.

## 3. Keep iteration protocol, replace operation semantics

Keep the existing cursor protocol:

```text
iterate(cursor)      -> next cursor or None
iteratorValue(cursor)-> encountered value
```

All generic eager operations MUST drive that protocol through `for (x in self)` or equivalent `iterate`/`iteratorValue` sends.

Do not implement generic operations by:

- raw integer indexing;
- `size` + `at` assumptions;
- calling `self.each` internally;
- inspecting callback arity.

The `self.each` prohibition matters because historical Map overrides it. D.1 removes that override, but generic operations should still remain protocol-driven and independent of that fact.

## 4. Eager `map`

Canonical selector:

```phalcom
collection.map |value| { ... }
```

Definition-side selector identity is the ordinary one-positional-argument method `map(_)`.

Implementation:

1. allocate `List.new()`;
2. traverse source encounter order;
3. invoke callback exactly once per encountered value;
4. append the callback result to the output;
5. return the output after complete exhaustion.

Use the ratified D.2 `append` once available. During phase-local bootstrapping, it is acceptable to call the existing raw `push_` from core code if D.2 has not landed yet; do not depend on chainable `add`.

A callback result equal to surface `None` is an ordinary mapped element and must be stored.

The output is materialized before the send returns. It must not be `MapView`, `Iterator`, or any future stage object.

## 5. Eager `filter`

Canonical selector:

```phalcom
collection.filter |value| { ... }
```

For each encountered source value:

- callback result `true` -> append the original source value;
- callback result `false` -> skip it;
- non-Bool callback result -> ordinary Bool/type failure through the language's existing branch semantics.

Return a new List.

Filtering is stable: retained values preserve source encounter order.

## 6. Eager `flatMap`

Canonical selector:

```phalcom
collection.flatMap |value| { ... }
```

Algorithm:

```text
out = new List
for outerValue in source:
    inner = callback(outerValue)
    for innerValue in inner:
        append innerValue to out
return out
```

The callback is invoked exactly once per outer value. It is expected to return an iterable value. If the returned value does not implement the iteration protocol, let the ordinary message/type failure propagate; do not convert that failure into Result.

D.1 does not add boundedness analysis. A callback that returns an unbounded source can make `flatMap` fail to terminate. Spec E owns static rejection of provably unbounded eager exhaustion.

## 7. `each` is effect traversal returning Unit

Canonical selector:

```phalcom
collection.each |value| { ... }
```

Traverse once in encounter order and invoke callback once per value.

After successful completion return canonical Unit `()`.

Do not return:

- the receiver;
- the last callback value;
- None.

The callback's runtime return value is ignored. When optional typing can check the callback, its expected result type is Unit; D.1 does not add runtime inspection solely to enforce that static expectation.

## 8. Indexed callback variants

D.1 implements the two indexed variants explicitly named by the ratified specification:

```phalcom
collection.map(indexed: |index, value| { ... })
collection.each(indexed: |index, value| { ... })
```

Their selector identities differ from the ordinary forms because the sole argument is labeled `indexed:`.

Do not decide indexed behavior by reading closure arity.

### 8.1 Index meaning

The callback index is the zero-based **encounter ordinal**:

```text
0, 1, 2, ...
```

It is not the raw iteration cursor. This distinction is essential for iterables whose cursor is not a user-visible numeric index.

Use a local counter incremented after every encountered value.

### 8.2 Result behavior

- `map(indexed:)` returns a List of callback results;
- `each(indexed:)` returns Unit.

Do not add `filter(indexed:)`, `flatMap(indexed:)`, or a generalized indexed matrix in this phase; they were not ratified by the source specification.

## 9. Predicate-qualified queries use `where:`

Implement these selector shapes:

```phalcom
collection.find(where: predicate)
collection.index(where: predicate)
collection.any(where: predicate)
collection.all(where: predicate)
collection.none(where: predicate)
collection.count(where: predicate)
```

If labeled trailing-closure syntax is not yet landed on implementation HEAD, tests may call the same selectors with the callback inside parentheses. Do not change selector identity or invent an unlabeled compatibility overload merely because the prettier trailing syntax is pending.

Retire the historical positional predicate overloads (`find(_)`, `all(_)`, `any(_)`, `count(_)`) unless another current ratified non-collection specification independently requires one. Repository usages must migrate to `where:`.

## 10. `find where:`

Traverse in encounter order.

On the first predicate result `true`:

```text
return Some(encountered value)
```

On exhaustion without a match:

```text
return None
```

The predicate is not called after the first match.

A matched value equal to surface `None` MUST produce `Some(None)`, not bare `None`.

## 11. `index where:`

Traverse in encounter order while maintaining encounter ordinal.

On first match:

```text
return Some(index)
```

On miss:

```text
return None
```

This index is an encounter ordinal, not an opaque cursor and not a Map key.

It short-circuits at first match.

Until the numeric tower lands, the produced index uses HEAD's current Number representation for integer values. Do not add another Int/Float compatibility mechanism; reuse the project-wide numeric transition plan.

## 12. Quantifiers

### 12.1 `any where:`

Return `true` at first matching element. Return `false` on exhaustion.

Empty identity:

```text
false
```

### 12.2 `all where:`

Return `false` at first non-matching element. Return `true` on exhaustion.

Empty identity:

```text
true
```

### 12.3 `none where:`

Return `false` at first matching element. Return `true` on exhaustion.

Empty identity:

```text
true
```

Do not implement `none` as `not any(...)` if doing so would force creation of an extra closure or alter callback/trace behavior. A direct loop is clearer and preserves exact callback count.

## 13. `count where:`

Invoke the predicate for every encountered element unless source iteration or the callback fails.

Increment for every `true` result and return the final integer count.

There is no predicate short-circuit.

The existing nullary `count` getter may remain if independently useful/current, but D.1 does not use it as a substitute for `count(where:)` and does not make any new cross-family cardinality guarantee through it.

## 14. `fold(initial:using:)`

Canonical selector:

```phalcom
collection.fold(initial: accumulator, using: |acc, value| { ... })
```

Selector lane:

```text
0 positional arguments
labels in order: initial, using
```

This is important under Phalcom's order-significant labeled selector identity. Do not implement `fold(using:initial:)` as an alias.

Algorithm:

```text
acc = initial
for value in source encounter order:
    acc = callback(acc, value)
return acc
```

On empty input return the exact original initial value and do not invoke callback.

No Result wrapping is introduced for callback failures.

## 15. `reduce(using:)`

Canonical selector:

```phalcom
collection.reduce(using: |left, right| { ... })
```

No explicit initial accumulator is accepted under this selector.

Algorithm:

1. request first source value;
2. if source is empty -> return `None`;
3. set accumulator to first value;
4. request second and subsequent values in encounter order;
5. for each, `acc = callback(acc, value)`;
6. return `Some(acc)`.

Singleton input returns `Some(the original element)` with zero callback invocations.

A singleton element equal to surface `None` returns `Some(None)`.

### 15.1 Retire old `reduce(init,f)`

Current code and repository examples use the old two-positional-argument selector for explicit-initial reduction. Migrate them to `fold(initial:using:)`.

Do not leave `reduce(_,_)` as an alias. Its presence makes the conceptual distinction less clear and preserves a historical API the ratified design explicitly replaced.

## 16. First/last accessors — scoped application

The ratified design makes `first` and `last` safe Option-returning accessors where meaningful, but applicability across every collection family is deferred.

D.1 MAY install them on the clearly finite ordered built-ins already covered by C:

```text
List
Tuple
Bytes
```

Use:

```text
empty -> None
nonempty first -> Some(sequence[0])
nonempty last  -> Some(sequence[-1])
```

Do not put `first`/`last` on `Iterable` root in D.1: that would silently commit Set/Range/other future iterable applicability while their semantics remain deferred.

If implementation HEAD already gained a narrower ordered-sequence abstraction after these specs were written, place the methods there instead and document the move.

## 17. Remove direct lazy collection sugar

Current U-SEQ installed:

```text
Iterable#map      -> MapView
Iterable#where    -> WhereView
Iterable#skip     -> SkipView
Iterable#take     -> TakeView
```

D.1 must end with no direct concrete-collection selector that creates those lazy stages.

Actions:

1. replace `map` with §4's eager implementation;
2. remove `where`, `skip`, and `take` from `Iterable`;
3. remove `MapView`, `WhereView`, `SkipView`, and `TakeView` from `core.ph` if they have no remaining ratified caller;
4. migrate/delete U-SEQ tests that assert direct laziness;
5. do not pre-design E's replacement classes in D.

Spec E will introduce the explicit iterator/pipeline receiver and may reuse implementation ideas, but not the old public direct-collection surface.

## 18. Normalize Map `each`

After B.2, Map already has lightweight:

```phalcom
map.keys
map.values
map.entries
```

Current `Map#each(f)` passes two arguments to `f` for every entry. Remove that override.

Map should inherit the ordinary `Iterable#each(_)`, whose encountered value follows Map's ordinary iterator value on implementation HEAD (currently keys).

Users wanting key/value pairs traverse:

```phalcom
map.entries.each |entry| { ... }
```

or access `entry.key` / `entry.value`.

Do not keep a same-selector `each(_)` that changes callback invocation arity by receiver family. This is exactly the semantic ambiguity the selector-based callback rule forbids.

D.1 does not decide whether a future Map-specific labeled traversal convenience should exist; that would require a distinct selector.

## 19. Existing unrelated Iterable methods

Do not use D.1 as a pretext to redesign unrelated landed methods such as `join` unless migration makes a compile failure unavoidable.

`includes`/`isEmpty` likewise stay outside this phase unless another ratified spec supersedes them. D.1's mission is the explicitly ratified transform/query/reduction vocabulary.

## 20. Callback and failure behavior

Every D.1 callback runs as ordinary language code.

- thrown/raised errors propagate normally;
- no operation becomes Result merely because its callback can fail;
- Bool-requiring predicates use ordinary Bool semantics;
- no error is swallowed to continue traversal;
- no callback is retried;
- no callback is invoked after a short-circuit result is known.

Do not hold native heap borrows across callback execution. The intended implementation is `.ph`, so that hazard should not arise unless an implementer attempts an unnecessary native optimization.

Fiber-yield limitations of the current callable/native-frame machinery are pre-existing runtime work and are not solved by D.1.

## 21. Result-family rule for this phase

Because the full family-preservation matrix is explicitly deferred, the generic implementations of:

```text
map
map(indexed:)
filter
flatMap
```

return List in D.1.

Do not create speculative Tuple/Bytes/Map overrides merely to guess a future matrix.

This choice is intentionally easy to refine: a later per-family override can preserve a more natural family under the same selector without changing the generic semantic contract that the operation is eager.

## 22. Tests

Add focused language fixtures under a collection/eager-operations label or the repository's current nearest convention.

### 22.1 Eagerness

Prove callback side effects occur before `map` returns:

```text
counter = 0
result = [1,2,3].map { counter += 1; ... }
assert counter == 3 immediately
assert result is List
```

No `MapView` should be observable.

### 22.2 Transform order/content

Cover:

- map of empty/singleton/multiple;
- filter retain none/some/all;
- flatMap empty inner values, multiple inner values, nested encounter order;
- stored/mapped None.

### 22.3 Indexed variants

Use a custom Iterable whose cursor is not the numeric encounter ordinal if practical, and prove callback indices are `0,1,...` rather than the raw cursor.

At minimum test List plus one non-List iterable.

### 22.4 Query identities and short circuit

Use side-effect counters to prove:

- `any` stops after first true;
- `all` stops after first false;
- `none` stops after first true;
- `find` and `index` stop after first match;
- `count where:` visits all values;
- empty identities are exact.

### 22.5 Fold/reduce

Cover:

- fold empty returns exact initial object;
- fold order with three values;
- reduce empty -> None;
- reduce singleton -> Some(element), callback count 0;
- reduce multiple -> Some(result);
- reduce singleton surface None -> Some(None).

### 22.6 Map migration

After B.2, prove:

- `map.each` callback receives one value per iteration, not `(k,v)`;
- `map.entries.each` exposes Entry key/value pairs;
- generic `map`/queries over Map use its defined ordinary encounter value without dNU.

### 22.7 Retired selectors

Add negative or migration tests showing direct collection `where`/`skip`/`take` are no longer the lazy pipeline API after D.1. Do not assert E's future `.iter` syntax yet.

## 23. Repository migration audit

Search at minimum for:

```text
.map(
.reduce(
.find(
.all(
.any(
.count(
.where(
.skip(
.take(
MapView
WhereView
SkipView
TakeView
.each(
```

Inspect:

- `phalcom-core/core/core.ph`;
- `phalcom-core/tests/lang/**`;
- `examples/**`;
- `benchmarks/**`;
- docs/examples that are executed by verification tooling.

Particularly migrate old `reduce(init){...}` to `fold(initial:..., using:...)` and callers that relied on `map(...).toList` solely because `map` was lazy.

Do not mechanically replace `.map(...).toList` without checking intent: after D.1 the `.toList` may be redundant but still produce an extra copy.

## 24. Expected write set

Likely:

```text
phalcom-core/core/core.ph
phalcom-core/tests/lang/**
examples/**               # migration as required
benchmarks/**             # migration as required
phalcom-core/tests/lang/MANIFEST.md or equivalent
relevant current-spec/forge docs whose statements are superseded
```

No `primitive/*.rs`, heap representation, bytecode, parser, or compiler changes are required by D.1 itself.

If implementation HEAD has moved generic iteration behavior out of `core.ph`, follow the current architecture rather than forcing this old path.

## 25. Primitive-floor accounting

Expected native primitive binding delta:

```text
+0
```

All behavior is expressible through the existing iteration/call/List construction substrate.

Do not native-optimize transform loops in this phase merely for speed. Native user-callback loops would enlarge re-entrant VM/fiber review surface and violate the repository's established preference to keep block-taking control in `.ph` unless inexpressible.

## 26. Completion gate

D.1 is complete only when:

- direct concrete `map/filter/flatMap` are eager and materialized;
- `each` returns Unit;
- indexed variants use distinct selector identity and encounter ordinals;
- predicate queries use `where:` and satisfy short-circuit/empty identities;
- fold/reduce semantics and Option shape match this spec;
- old direct lazy-view selectors/classes no longer define the collection API;
- Map's same-selector two-callback-argument `each` override is gone;
- repository migrations are complete;
- floor census remains unchanged;
- `./scripts/verify.sh --full` passes.
