# Iterators and Iterables

> **Status:** Draft normative specification  
> **Semantic ownership:** the `Iterable` and `Iterator` contracts, traversal state and exhaustion, `for` iteration semantics, indexed and parallel loop lanes, and the standard item types of core iterable families.  
> **Related specifications:** [Traits](objects/traits.md), [Associated Types](associated-types.md), [Dispatch](callables/dispatch.md), [Selectors](callables/selectors.md), and the pattern/control-flow specifications.

> **Static structural iteration boundary**
>
> This specification owns ordinary runtime `Iterable`/`Iterator` traversal.
> C4 static iteration over Pack/Product/UnorderedShape/Sum reflection is a separate
> semantic operation owned by [Static Evaluation](static-evaluation.md) and
> [Structural Products](structural-products.md). Static structural `for` does not
> send `iterator`/`next` and does not require Product values or Types to conform to
> `Iterable`.
>
Iteration in Phalcom is trait-based. An **iterable** is a value that can produce a traversal; an **iterator** is one stateful traversal.

```phalcom
trait Iterable {
  type Item
  type Iterator

  iterator -> Self::Iterator
}

trait Iterator {
  type Item

  next -> Option<Self::Item>

  iterator { self }
}

The relationship between these contracts is realized by the ordinary guarded
generic-conformance mechanism, not by a special compiler or VM branch:

```phalcom
impl<I> Iterable for I
    where I |= Iterator
{
  type Item = <I as Iterator>::Item
  type Iterator = I
  iterator { self }
}
```

This is the canonical source law. Its antecedent is `I |= Iterator`; its
associated bindings establish that the iterable item is the iterator item and
that the iterator type is the subject itself. The rule is subject to ordinary
trait ownership, evidence, coherence, cycle, and generic-contract rules.
The standard library may stage this declaration in a transition module while
legacy cursor classes remain in production; that staging does not change the
normative law or grant cursor inheritance the `Iterator` or `Iterable` trait
without conformance evidence.
```

Every iterator is iterable over itself. Not every iterable is an iterator.

## Semantic model

Iteration uses two distinct language mechanisms.

| Form | Semantic category | Meaning |
|---|---|---|
| `source.iterator` | Getter message send | obtain a traversal from an iterable |
| `iterator.next` | Getter message send | advance a traversal and obtain its next result |
| `Self::Item` | associated projection | the item type in the current trait or conformance context |
| `Self::Iterator` | associated projection | the iterator type in the current `Iterable` context |
| `Iterable::Item` | associated lookup | the `Item` declaration owned by `Iterable` |
| `Iterable::Iterator` | associated lookup | the `Iterator` declaration owned by `Iterable` |
| `Iterator::Item` | associated lookup | the `Item` declaration owned by `Iterator` |

`.` is message-send syntax. `source.iterator` and `iterator.next` are ordinary Getter sends and use ordinary behavioral dispatch.

`::` is associated lookup or projection syntax. Associated types are not messages and are not recovered through behavioral dispatch.

Getter shape does not imply purity, idempotence, memoization, or absence of state change. Accessing `source.iterator` may construct traversal state. Accessing `iterator.next` advances traversal state.

## `Iterable`

`Iterable` describes a reusable source of traversal.

```phalcom
trait Iterable {
  type Item
  type Iterator

  iterator -> Self::Iterator
}
```

`Iterable::Item` is the type of values produced when the subject is traversed.

`Iterable::Iterator` is the concrete iterator type produced by the subject.

The type bound to `Iterable::Iterator` must satisfy the `Iterator` contract, and the iterator's `Iterator::Item` binding must be the same type as the iterable's `Iterable::Item` binding. The generic evidence and constraint syntax by which such relationships are expressed is specified separately.

For an iterable that is not itself an iterator, each access to `iterator` produces a fresh traversal:

```phalcom
const a = source.iterator
const b = source.iterator
```

`a` and `b` have independent traversal state. Advancing either traversal does not advance the other.

Freshness is an observable state property, not an allocation requirement. An implementation may optimize iterator allocation provided the independent-traversal semantics are preserved.

A value does not become iterable merely by defining members named `iterator` or `next`. Iterability is established through `Iterable` conformance.

## `Iterator`

`Iterator` describes one stateful, single-pass traversal.

```phalcom
trait Iterator {
  type Item

  next -> Option<Self::Item>

  iterator { self }
}
```

`Iterator::Item` is the type yielded by the traversal.

Traversal position belongs to the iterator. The protocol exposes no separate cursor.

### `next`

`next` is a Getter message send whose successful result is `Option<Self::Item>`.

```text
Some(item)
    one item was produced

None
    the traversal is exhausted
```

Reading `next` advances the traversal when an item is produced.

`None` denotes exhaustion, not temporary unavailability and not failure. Once an iterator returns `None`, all later reads of `next` on that same iterator return `None`.

```text
Some(a)
Some(b)
None
None
None
```

Failure while evaluating `next` follows ordinary Phalcom error propagation. Failure is not encoded as `None`.

The base `Iterator` contract does not specify the iterator's state after a raised error.

## Iterators are iterable

Every type satisfying `Iterator` also satisfies `Iterable`. This relationship does not imply that every iterable is an iterator.

For an iterator subject, the corresponding `Iterable` contract is determined by the iterator itself:

| `Iterable` member | Iterator subject |
|---|---|
| `Iterable::Item` | the same type as `Iterator::Item` |
| `Iterable::Iterator` | the iterator's own subject type |
| `iterator` | returns `self` |

Therefore:

```phalcom
const same = iterator.iterator
```

refers to the same traversal. It does not restart, clone, copy, or reset it.

If:

```phalcom
iterator.next
```

advances the traversal, then a subsequent:

```phalcom
iterator.iterator.next
```

continues that same traversal.

The language-level relationship above is normative. The generic evidence mechanism that realizes it is specified separately.

## `for`

A `for` statement consumes one or more iterable lanes.

Conceptually, its grammar is:

```text
for lane ("," lane)* block

lane:
    pattern ["at" ordinalBinding] "in" iterableExpression
```

Examples:

```phalcom
for item in items {
  use(item)
}

for (key, value) in map {
  use(key, value)
}

for item at index in items {
  use(index, item)
}

for x in xs, y in ys {
  use(x, y)
}
```

Each lane's source must satisfy `Iterable`.

### Single-lane iteration

For:

```phalcom
for value in source {
  body
}
```

the source expression is evaluated exactly once and its `iterator` Getter is read exactly once before traversal begins.

The loop then repeatedly reads `next` from that iterator. `Some(item)` binds the loop pattern and executes the body once. `None` terminates the loop.

The observable semantics are equivalent to:

```phalcom
let iterator = source.iterator

loop {
  match iterator.next {
    Some(value) => {
      body
    },
    None => break,
  }
}
```

This is a semantic equivalence, not a required compiler transformation.

### Pattern binding

A `for` lane binds a pattern, not merely an identifier.

```phalcom
for (key, value) in map {
  ...
}
```

uses the same pattern system as other Phalcom binding and matching constructs.

`for` uses that pattern system in **irrefutable binding mode**. A yielded value that does not satisfy the lane pattern raises the ordinary pattern-mismatch failure; it is not silently skipped.

Consequently, a refutable-looking pattern in `for` is not a filtering construct:

```phalcom
for Some(value) in options {
  ...
}
```

does not mean "ignore `None`". A `None` item causes pattern mismatch according to ordinary irrefutable binding semantics.

### Ordinal binding

A lane may bind its zero-based encounter ordinal with `at`:

```phalcom
for item at index in items {
  ...
}
```

The ordinal is:

- zero-based;
- local to that lane;
- incremented once for each successfully formed iteration step;
- independent of the source's internal traversal state;
- not a collection index;
- not a `Map` key;
- not a `Range` value;
- not an iterator cursor.

For example:

```phalcom
for (key, value) at index in map {
  ...
}
```

binds `index` to `0`, `1`, `2`, ... in encounter order while `key` remains the Map entry's actual key.

The explicit value-level counterpart is:

```phalcom
for (index, item) in items.indexed {
  ...
}
```

`indexed` is therefore an iterable view that yields each source item paired with its zero-based encounter ordinal.

### Parallel lanes

Two or more comma-separated lanes advance in lockstep:

```phalcom
for x in xs, y in ys {
  body
}
```

All lane source expressions are evaluated exactly once, left-to-right, before traversal begins. One iterator is obtained from each lane.

Parallel lanes have **strict zip** semantics. For every attempted step, each lane is advanced once.

```text
all lanes produce Some(...)
    bind all lane patterns and execute the body

all lanes produce None
    terminate normally

some lanes produce Some(...) while others produce None
    raise ZipLengthMismatchError
```

Parallel iteration never silently truncates to the shortest source.

A mismatching strict-zip step produces no body iteration. Because mismatch detection must inspect all lanes for that attempted step, another stateful iterator may already have advanced when `ZipLengthMismatchError` is raised.

N lanes generalize the same rule.

Parallel loop syntax is semantically equivalent to zipped traversal:

```phalcom
for x in xs, y in ys {
  body
}
```

corresponds to:

```phalcom
for (x, y) in (xs, ys).zipped {
  body
}
```

`(xs, ys).zipped` is a first-class iterable view. Its yielded tuple preserves lane order and its length semantics are the same strict-zip semantics as comma-separated `for` lanes.

Nested loops are not parallel lanes:

```phalcom
for x in xs {
  for y in ys {
    ...
  }
}
```

The inner iterable is evaluated and traversed according to the ordinary nested-loop semantics for each execution of the outer body.

### Parallel ordinals

Each lane may bind its own encounter ordinal:

```phalcom
for user at userIndex in users,
    role at roleIndex in roles {
  ...
}
```

Each ordinal belongs to its lane. Under ordinary lockstep traversal they advance together, but they remain distinct source bindings.

### Loop control

`break` terminates the enclosing `for` immediately. No further `next` read is performed for that loop.

`continue` skips the remainder of the body and proceeds to the next iteration step.

For a single lane, the next step reads `next` once from that lane's iterator.

For parallel lanes, the next step performs the next strict lockstep advance across all lane iterators.

Errors raised while evaluating source expressions, obtaining iterators, reading `next`, binding patterns, or enforcing strict zip propagate normally.

## Pattern loops

`if let` and `while let` are pattern-driven control-flow forms, not part of the iteration protocol.

They compose naturally with iterators:

```phalcom
let iterator = source.iterator

while let Some(value) = iterator.next {
  use(value)
}
```

`while let` repeatedly evaluates its scrutinee expression. If the pattern matches, it binds the pattern and executes the body; the first mismatch terminates the loop normally.

Thus the example above advances the iterator until `next` returns `None`.

Likewise:

```phalcom
if let Some(value) = iterator.next {
  use(value)
}
```

performs one iterator step and conditionally enters the body.

Pattern syntax, binding scope, refutable matching, and the general semantics of `if let` and `while let` belong to the pattern and control-flow specifications.

## Standard item contracts

The standard iterable families use the following `Iterable::Item` bindings:

| Iterable family | `Iterable::Item` |
|---|---|
| `List<T>` | `T` |
| `Set<T>` | `T` |
| `Map<K, V>` | `(key: K, value: V)` |
| `Tuple<***Es: CompleteTypePack>` | `Tuple<***Es>::Element` |
| `Bytes` | `Int` |
| `Range` | `Number` |
| `String.chars` and `Substring.chars` | `Char` |
| `String.bytes`, `String.utf8`, and `Substring.utf8` | `Int` |

String and Substring are traversed through these explicit scalar and UTF-8 byte
views; the text contract for their encoding, indexing, and view behavior is
owned by [Text and String Model](text.md).

A `Map<K, V>` yields the complete entry as a labeled tuple:

```phalcom
(key: K, value: V)
```

Direct Map iteration therefore naturally supports:

```phalcom
for (key, value) in map {
  ...
}
```

When retained as one value, the entry preserves its labels and supports the ordinary tuple operations associated with that labeled shape.

Key-only and value-only traversal are separate collection views or operations; the Map's own item is the entry.

Tuple iteration preserves its specialized structural element type through the
canonical `Tuple<***Es>::Element` member. Runtime-shaped `TupleZip` views remain
behind their documented dynamic boundary because their lane count and result
Tuple shape are known only at runtime. This specification does not define an
alternative heterogeneous element-union inference rule.

## Protocol boundaries

The primitive iteration protocol deliberately remains small.

There is no public cursor. Traversal position is iterator-owned state.

There is no primitive `hasNext` requirement. Consumers inspect `next`.

There is no primitive reset requirement. A reusable iterable produces another fresh traversal through `iterator`.

`Iterator` does not imply clonability. Copying or cloning traversal state requires a separate capability where provided.

The base protocol does not define mutation-during-iteration behavior. Collection specifications may impose stronger rules.

Iterator adapters such as mapping, filtering, taking, skipping, flattening, indexing, or zipping may themselves be iterable and iterator-producing abstractions, but their complete APIs are standard-library concerns. They do not change the primitive `Iterable`/`Iterator` contract defined here.

The syntax and proof machinery for generic trait-conformance constraints and associated-type equality constraints are specified separately. This specification states only the semantic relationships required by iteration.
