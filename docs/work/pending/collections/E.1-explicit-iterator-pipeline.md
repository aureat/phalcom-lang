# Spec E.1 — Explicit Iterator Pipeline Runtime

Status: implementation specification. Requires D.1's eager direct-operation semantics and the existing bare-cursor iteration protocol.

## 1. Mission

Introduce an explicit lazy receiver without replacing Phalcom's current iteration protocol.

The public semantic split is:

```text
concrete Iterable receiver
    map/filter/flatMap -> eager (D)

explicit Iterator receiver
    map/filter/flatMap -> lazy (E)
```

Canonical use:

```phalcom
const pipeline =
    users.iter
        .map { user => user.name }
        .filter { name => not name.empty? }

const names = pipeline.toList
```

Nothing before `toList` consumes `users` or invokes the callbacks.

E.1 is deliberately a `.ph` library layer over the already-landed cursor protocol. Do not add a new `Value` arm, heap-native iterator object, bytecode instruction, or native primitive merely to represent a lazy stage.

Expected primitive-floor delta: **0**.

## 2. Preserve the existing cursor protocol

Keep:

```text
iterable.iterate(cursor)
    -> next live cursor
    -> None at exhaustion

iterable.iteratorValue(cursor)
    -> value associated with that live cursor
```

Keep `None` as the start/exhaustion sentinel expected by the compiler's existing `for` lowering.

A pipeline stage is itself an `Iterable` and implements those two sends. This is the core architectural leverage: `for`, D's eager generic methods, D.3's materializers, and user iteration require no VM special case for pipeline objects.

Do not switch the protocol to:

```text
next() -> Option<Value>
```

or to a mutable Rust iterator hidden behind a native object. That would unnecessarily fork the existing language model.

## 3. Add an explicit Iterator root

Recommended core shape:

```phalcom
class Iterator is Iterable {
    iter => self

    map(f) => MapIterator.new(self, f)
    filter(pred) => FilterIterator.new(self, pred)
    flatMap(f) => FlatMapIterator.new(self, f)

    skip(n) => SkipIterator.new(self, n)
    take(n) => TakeIterator.new(self, n)
    takeWhile(pred) => TakeWhileIterator.new(self, pred)
}
```

Use current HEAD's constructor/field-declaration syntax rather than copying this sketch literally if class-layout rules changed.

`Iterator` is the semantic lazy-pipeline receiver root. It intentionally overrides only operations whose meaning differs from D's eager `Iterable` implementation.

Everything else can remain inherited unless a later iterator-terminal specification says otherwise.

In particular, after D:

```text
Iterator#each
Iterator#find(where:)
Iterator#any(where:)
Iterator#all(where:)
Iterator#none(where:)
Iterator#count(where:)
Iterator#fold(initial:,using:)
Iterator#reduce(using:)
Iterator#toList
Iterator#toSet
Iterator#toMap
```

may use the generic eager/terminal implementation inherited from `Iterable`.

This gives the desired receiver-driven rule without parallel names such as `lazyMap`.

## 4. `.iter` on ordinary Iterable

Add to the ordinary `Iterable` root:

```phalcom
iter => SourceIterator.new(self)
```

`SourceIterator` is a lightweight lazy wrapper that retains the source and delegates the cursor protocol:

```text
SourceIterator.iterate(cursor)
    -> source.iterate(cursor)

SourceIterator.iteratorValue(cursor)
    -> source.iteratorValue(cursor)
```

`Iterator#iter => self` prevents nested wrappers:

```phalcom
xs.iter.iter
```

is semantically the same pipeline object as:

```phalcom
xs.iter
```

Do not make `.iter` copy/materialize the source.

Do not probe `size`.

Do not execute a first `iterate(None)` call eagerly.

## 5. Concrete stage class names

Recommended implementation classes:

```text
SourceIterator
MapIterator
FilterIterator
FlatMapIterator
SkipIterator
TakeIterator
TakeWhileIterator
```

Every stage except `SourceIterator` stores an upstream `Iterator`; `SourceIterator` stores an arbitrary `Iterable`.

These concrete names are implementation architecture, not selector identity. The stable language behavior is the `.iter` boundary and the iterator operations. If Phalcom reflection exposes the concrete classes, do not document user programs as depending on a particular stage-class name or field layout.

Do not reuse the old public behavior:

```text
MapView
WhereView
SkipView
TakeView
```

as direct collection-return values. D.1 retired that API. It is acceptable to reuse internal algorithmic code while migrating class names/placement.

## 6. Pipeline values are traversal descriptors, not consumed cursors

A pipeline object stores configuration:

```text
source
callback/predicate/count
```

It does **not** store "current position" as mutable instance state.

Traversal position belongs in the cursor argument.

Consequences:

```phalcom
const p = xs.iter.map { x => f(x) }

for (x in p) { ... }
for (x in p) { ... }
```

must perform two fresh traversals.

Callbacks run again during the second traversal.

Do not memoize mapped values or accepted filter positions.

Do not mutate a `_current`, `_index`, `_taken`, `_done`, or similar stage instance field from `iterate`.

This preserves the existing repeatable cursor protocol and avoids the historical one-shot `TakeView` bug that U-SEQ already identified.

## 7. Lazy `map`

`MapIterator` retains:

```text
source: Iterator
fn: callable
```

Cursor behavior:

```text
iterate(cursor)
    -> source.iterate(cursor)

iteratorValue(cursor)
    -> fn(source.iteratorValue(cursor))
```

Properties:

- stage construction invokes `fn` zero times;
- advancing the cursor does not invoke `fn`;
- retrieving the current value invokes `fn`;
- ordinary `for`/materialization therefore invokes `fn` once per encountered element;
- repeating a traversal invokes it again.

Do not precompute the next mapped value inside `iterate`.

Do not cache callback results.

If `fn` fails, propagate the ordinary callback error.

## 8. Lazy `filter`

`FilterIterator` retains:

```text
source: Iterator
predicate: callable
```

Its cursor is an upstream live cursor.

Conceptual `iterate(previous)`:

```text
candidate = source.iterate(previous)

while candidate != None:
    value = source.iteratorValue(candidate)

    if predicate(value):
        return candidate

    candidate = source.iterate(candidate)

return None
```

`iteratorValue(cursor)` delegates directly to the source.

Properties:

- stage construction invokes the predicate zero times;
- predicate calls happen only while a consumer asks for the next accepted cursor;
- retained values preserve source encounter order;
- a rejected item is never surfaced as a stage cursor;
- a non-Bool predicate result follows the ordinary Bool/control-flow failure path.

Do not inspect callback arity.

Do not rename the lazy operation to `where`.

The canonical same-vocabulary rule is:

```text
collection.filter -> eager
collection.iter.filter -> lazy
```

## 9. Lazy `skip`

E.1 carries forward the useful U-SEQ limiter only behind `.iter`:

```phalcom
source.iter.skip(n)
```

`n` must be a non-negative Int.

Until the numeric tower lands, use the same compatibility rule as Spec C for an integral, finite, non-negative `Number`. Do not invent a second numeric-index convention.

Validation occurs when `skip(n)` constructs the stage, not when traversal later begins.

### 9.1 Cursor design

The stage must skip exactly the first `n` source elements **per traversal**.

Do not store "already skipped" on the stage object.

A simple cursor design can use a tagged/product cursor:

```text
(sourceCursor, started)
```

or can special-case the first `iterate(None)` call and thereafter delegate source cursors, provided `None` still unambiguously means "new traversal / exhausted".

Recommended algorithm:

```text
if previous == None:
    cursor = source.iterate(None)
    remaining = n

    while cursor != None and remaining > 0:
        cursor = source.iterate(cursor)
        remaining -= 1

    return cursor

return source.iterate(previous)
```

This works because every returned live cursor after the first call is an upstream cursor.

`iteratorValue` delegates to source.

`skip(0)` must not consume an extra source element.

## 10. Lazy `take`

Canonical form:

```phalcom
source.iter.take(n)
```

`n` uses the same non-negative Int validation rule as `skip`.

Validation occurs at stage construction.

### 10.1 Why count belongs in the cursor

A stateless stage needs to know both:

```text
upstream cursor
number already yielded
```

Recommended live cursor:

```phalcom
(upstreamCursor, yieldedCount)
```

a two-element Tuple.

The tuple never normalizes to Unit because its arity is 2.

Conceptual `iterate(previous)`:

```text
if n == 0:
    return None

if previous == None:
    upstream = source.iterate(None)
    if upstream == None:
        return None
    return (upstream, 1)

upstream = previous[0]
yielded  = previous[1]

if yielded >= n:
    return None

next = source.iterate(upstream)
if next == None:
    return None

return (next, yielded + 1)
```

`iteratorValue(cursor)`:

```text
source.iteratorValue(cursor[0])
```

### 10.2 Important zero-count behavior

`take(0)` must return exhaustion without calling:

```text
source.iterate(None)
source.iteratorValue(...)
mapping/filtering callbacks upstream
```

This is both a laziness law and the reason `take(0)` can safely bound an otherwise unbounded/effectful source.

## 11. Lazy `takeWhile`

Canonical form:

```phalcom
source.iter.takeWhile { value => predicate(value) }
```

The predicate is not invoked at construction.

Cursor can remain the upstream cursor.

Conceptual `iterate(previous)`:

```text
candidate = source.iterate(previous)

if candidate == None:
    return None

value = source.iteratorValue(candidate)

if predicate(value):
    return candidate

return None
```

Once a consumer observes the first false predicate result, that traversal is exhausted.

No mutable `_done` field is necessary: ordinary consumers stop calling after `None`. A fresh traversal begins again from `iterate(None)` and re-evaluates the predicate.

This repeatability is intentional.

## 12. Lazy `flatMap`

`FlatMapIterator` is the only stage in this phase whose cursor must retain nested traversal state.

For each outer source element:

```text
fn(outerValue) -> inner Iterable
```

The stage then yields the inner values in order before advancing the outer source.

### 12.1 Callback timing law

The flattening callback MUST be called exactly once for each outer element reached in a single traversal.

It MUST NOT be recomputed for every value of that same inner iterable.

Therefore a cursor that stores only:

```text
(outerCursor, innerCursor)
```

is insufficient unless the inner iterable can be reconstructed without re-running user code.

Recommended cursor:

```text
(outerCursor, innerIterable, innerCursor)
```

a three-element Tuple.

### 12.2 Initial/advance algorithm

Conceptually:

```text
function seekFromOuter(outerCursor):
    while outerCursor != None:
        outerValue = source.iteratorValue(outerCursor)
        inner = fn(outerValue)
        innerCursor = inner.iterate(None)

        if innerCursor != None:
            return (outerCursor, inner, innerCursor)

        outerCursor = source.iterate(outerCursor)

    return None
```

Initial step:

```text
outer = source.iterate(None)
return seekFromOuter(outer)
```

Subsequent step from `(outer, inner, innerCursor)`:

```text
nextInner = inner.iterate(innerCursor)

if nextInner != None:
    return (outer, inner, nextInner)

nextOuter = source.iterate(outer)
return seekFromOuter(nextOuter)
```

`iteratorValue(cursor)`:

```text
cursor[1].iteratorValue(cursor[2])
```

If the callback returns a non-iterable value, ordinary method lookup/type failure propagates when traversal reaches it.

No callback is invoked before traversal reaches the corresponding outer value.

## 13. Materializers and terminals are inherited eager consumers

Do not create special native materializer loops for iterator classes.

After D.1/D.3, the generic Iterable implementations can consume an Iterator because it speaks the same protocol.

Examples:

```phalcom
pipeline.toList
pipeline.toSet
pipeline.fold(initial: x, using: f)
pipeline.reduce(using: f)
pipeline.count(where: p)
```

These are terminal/eager operations.

E.3 decides whether a particular terminal use is statically invalid because the pipeline is provably unbounded.

Short-circuit queries such as:

```text
find(where:)
index(where:)
any(where:)
all(where:)
none(where:)
```

may stop before exhaustion and are treated separately by E.3.

## 14. `for` over an Iterator

No compiler change is required merely to iterate a pipeline.

Existing `for` lowering should continue to send:

```text
iterate
iteratorValue
```

to the runtime receiver.

This is important: `.iter` is a library-level semantic receiver boundary, not a second iteration protocol.

## 15. Error and side-effect propagation

Every lazy callback executes at consumption time.

Examples:

```phalcom
const p = xs.iter.map { x =>
    log(x)
    risky(x)
}
```

Construction of `p` performs neither operation.

A later:

```phalcom
p.toList
```

performs them in source encounter order.

If `risky(x)` fails:

- the failure propagates normally;
- no completed materialized result is returned;
- the pipeline object itself remains a reusable descriptor and is not marked "consumed";
- a later fresh traversal may execute the callback again from the beginning.

Do not wrap callback failure in `Result` unless the called terminal's independent API explicitly returns Result.

## 16. Source mutation remains deferred

`SourceIterator` and stages retain their source by reference.

Therefore mutations completed **before** a traversal naturally affect what that later traversal observes.

E.1 does not ratify what happens when the underlying collection is structurally mutated during an active traversal.

Do not add:

- snapshot copies;
- mutation version counters;
- fail-fast exceptions;
- hidden locking

solely in this phase.

Preserve whatever the underlying cursor protocol currently does and keep mutation-during-iteration tests out of normative E fixtures.

## 17. Migration from U-SEQ

D.1 removes the old direct-lazy surface. E.1 finishes the migration.

Historical:

```phalcom
xs.map(f)       // U-SEQ lazy MapView
xs.where(p)
xs.skip(3)
xs.take(10)
```

New:

```phalcom
xs.map(f)               // D eager List
xs.filter(p)            // D eager List

xs.iter.map(f)          // E lazy
xs.iter.filter(p)       // E lazy
xs.iter.skip(3)         // E lazy
xs.iter.take(10)        // E lazy
```

Do not restore:

```text
lazyMap
where
```

as aliases.

Repository audit scope should include:

```text
phalcom-core/core/core.ph
phalcom-core/tests/lang/sequence/
examples/
benchmarks/
docs/guide/
generated core selector tables
LSP/core symbol surfaces if generated from core.ph
```

## 18. Suggested source placement

Recommended:

1. keep `Iterable` near the kernel collection roots;
2. add `Iterable#iter`;
3. define `Iterator is Iterable` after `Iterable`;
4. define stage classes immediately after `Iterator` or after concrete kernel collection classes, consistently with current core ordering.

No stage needs Rust universe bootstrap unless current class-closure rules make ordinary core.ph class creation impossible for a referenced core class. Prefer ordinary `.ph` classes.

If current HEAD's closed-class compiler treats `core.ph` specially, use that existing mechanism; do not weaken class closure globally for E.

## 19. Tests

Add focused fixtures, adapting exact paths to the repository harness.

### 19.1 Receiver semantics

- `iterator_direct_map_still_eager.ph`
- `iterator_iter_map_is_lazy.ph`
- `iterator_iter_filter_is_lazy.ph`
- `iterator_iter_flatmap_is_lazy.ph`
- `iterator_iter_idempotent.ph`

Prove laziness with side-effect counters observed before and after materialization.

### 19.2 Ordering/composition

- `iterator_map_filter_pipeline_order.ph`
- `iterator_filter_map_pipeline_order.ph`
- `iterator_skip_basic.ph`
- `iterator_skip_zero.ph`
- `iterator_take_basic.ph`
- `iterator_take_zero_does_not_touch_source.ph`
- `iterator_takewhile_stops_before_next_value.ph`
- `iterator_flatmap_nested_order.ph`
- `iterator_flatmap_callback_once_per_outer.ph`

### 19.3 Repeatability

- `iterator_pipeline_repeatable.ph`
- `iterator_take_repeatable.ph`
- `iterator_takewhile_repeatable.ph`
- `iterator_flatmap_repeatable.ph`

Traverse the **same pipeline object** twice and pin identical value sequences while proving callbacks execute again.

### 19.4 Validation

Negative fixtures:

- `iterator_take_negative_raises.ph`
- `iterator_take_fractional_compat_number_raises.ph` until Int lands
- `iterator_skip_negative_raises.ph`
- `iterator_skip_non_number_raises.ph`

## 20. Completion checklist

E.1 is complete when:

- `.iter` exists on ordinary Iterable;
- iterator `.iter` is idempotent;
- Iterator overrides `map/filter/flatMap` lazily;
- `skip/take/takeWhile` are lazy iterator-only selectors;
- no direct collection `where/skip/take` remains;
- stage state is cursor-local, not mutable stage-instance traversal state;
- `flatMap` does not rerun its callback to advance inside an existing inner iterable;
- inherited D terminals work on pipeline objects;
- pipeline construction invokes no transform predicate/callback;
- repeatability fixtures pass;
- floor census remains unchanged.
