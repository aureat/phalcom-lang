# Phalcom Collections Core Semantics Specification

**Status:** Ratified language design specification  
**Scope:** Core collection access and mutation semantics shared across built-in collections; strict and safe lookup; subscript getter/setter behavior; subscript assignment expression value; negative indexing; insertion positions; slicing; List slice assignment; eager collection transformations; standard transformation/query vocabulary; `fold`/`reduce`; mutation result conventions; sorting; grouping; partitioning; Map entry/value conventions; conversion rules already ratified.  
**Out of scope:** Full collection protocol hierarchy, iterator object model, mutation-during-iteration rules, complete Set semantics, generic variance, complete Range/Progression runtime semantics, Bytes details, collection printing, and unresolved cross-family equality rules.

---

## 1. Purpose

This specification defines the core semantic behavior expected of Phalcom's built-in collection operations.

The design emphasizes:

- selector-based dispatch;
- strict operations where syntax implies strictness;
- explicit safe lookup through `Option`;
- `Result` only for recoverable failures that carry useful error detail;
- consistent negative indexing for finite indexed sequences;
- eager behavior for concrete collections;
- lazy behavior only through explicit iterator pipelines;
- non-fluent mutation results;
- deterministic, role-revealing API naming.

---

## 2. Normative Terminology

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

### 2.1 Finite indexed sequence

A **finite indexed sequence** is a finite collection whose elements are addressable by integer position.

Examples include:

```text
List
Tuple
Bytes
```

subject to each type's detailed specification.

### 2.2 Strict lookup

A **strict lookup** returns a value or raises the corresponding language error when the requested element/key does not exist.

### 2.3 Safe lookup

A **safe lookup** returns `Option<T>` rather than raising for ordinary absence.

### 2.4 Mutation command

A **mutation command** is a mutating operation whose successful result carries no payload.

Such operations return `Unit`.

### 2.5 Eager concrete transformation

An **eager concrete transformation** executes immediately against a concrete collection and returns a concrete result collection.

### 2.6 Lazy iterator transformation

A **lazy iterator transformation** is applied to an iterator/pipeline and produces a lazy iterator-stage value.

---

## 3. Native Result-Type Conventions

Collection APIs use Phalcom's native semantic result types.

The general rule is:

```text
successful command with no payload
    → Unit

ordinary absence
    → Option<T>

recoverable failure carrying useful detail
    → Result<T, E>

strict language operation
    → value or raises

predicate
    → Bool

index/count
    → Int

comparison result
    → Ordering
```

`None` MUST NOT be used as a generic successful-command result.

---

## 4. Unit

`Unit` is Phalcom's zero-product type.

Its sole value is:

```phalcom
()
```

Collection mutation commands such as:

```phalcom
list.append(value)
map.clear
```

return `Unit` when successful and no other result payload is required.

The zero-product normalization rules are specified separately.

---

## 5. Subscript Selector Families

Subscript access is modeled through ordinary selector families.

Getter family:

```phalcom
[](*args, **kwargs)
```

Setter family:

```phalcom
[]=(*args, put, **kwargs)
```

`[]` and `[]=` are selector families.

The exact internal parser/lowering representation is implementation-defined, but semantic dispatch MUST behave as specified here.

---

## 6. Subscript Assignment Lowering

A source-level assignment:

```phalcom
obj[index] = value
```

conceptually lowers to:

```phalcom
obj.[]=(index, put: value)
```

The label `put` is an ordinary labeled parameter.

It is not globally reserved.

Therefore getter syntax may legally use:

```phalcom
obj[put: address]
```

but:

```phalcom
obj[put: address] = value
```

is invalid because lowering would produce duplicate `put` labels.

Duplicate-label rules are defined by the argument-pack specification.

---

## 7. Subscript Assignment Expression Value

Subscript assignment evaluates to the original right-hand-side value.

For:

```phalcom
const result = obj[index] = value
```

on success:

```text
result
    = original RHS value/object
```

The setter method's own return value is discarded for purposes of the assignment expression.

This rule is independent of the setter implementation's internal return convention.

---

## 8. Subscript Assignment Evaluation Order

The evaluation order is:

```text
1. evaluate receiver
2. evaluate subscript arguments in lexical source order
3. evaluate RHS
4. construct setter argument pack
5. dispatch []=
6. discard setter return for assignment-expression purposes
7. evaluate assignment expression to original RHS
```

Any failure during steps 1-5 aborts the expression.

The assignment expression produces no successful RHS result if dispatch or execution fails.

---

## 9. Strict Sequence Lookup

For finite indexed sequences:

```phalcom
sequence[index]
```

returns the element or raises `IndexError`.

Examples:

```phalcom
list[index]
tuple[index]
bytes[index]
```

The exact returned element type depends on the collection family.

Strict subscript syntax MUST NOT silently return `None` or `Option`.

---

## 10. Strict Map Lookup

For Map:

```phalcom
map[key]
```

returns the associated value or raises `KeyError`.

This strictness is part of subscript syntax.

Safe absence handling uses `get`.

---

## 11. Safe Lookup

Safe lookup uses:

```phalcom
collection.get(key)
```

and returns:

```text
Option<T>
```

for the relevant value type.

For an indexed sequence, the argument is an index.

For a Map, the argument is a key.

Ordinary absence produces:

```text
None
```

Presence produces:

```text
Some(value)
```

---

## 12. Stored `None` Versus Absence

A stored value equal to `None` remains distinguishable from absence.

Example:

```phalcom
map[key] = None

map.get(key)
// Some(None)
```

Therefore safe lookup MUST wrap present values regardless of whether the stored value is `None`.

---

## 13. Eager Default Lookup

An eager fallback may be supplied through strict lookup syntax:

```phalcom
collection[key, default: fallback]
```

The `fallback` expression follows ordinary argument evaluation and is therefore evaluated eagerly before dispatch.

This form is suitable only when eager fallback evaluation is intended.

---

## 14. Lazy Lookup Fallback

Lazy fallback uses a trailing closure:

```phalcom
collection.get(key) orElse: |missingKey| {
    computeFallback(missingKey)
}
```

This is a single selector shape in Phalcom's selector-based dispatch model.

It is not defined as an `Option` chain requiring intermediate user-visible `Option` dispatch.

The fallback closure receives the originally supplied key/index.

---

## 15. Mutable Map Lookup-and-Insert

Mutable Map supports a lazy missing-value insertion form:

```phalcom
map.get(key) orPut: |missingKey| {
    createValue(missingKey)
}
```

If the key is present, the existing value is returned.

If absent, the closure is evaluated and its result is inserted according to normal Map insertion semantics.

The callback receives the originally supplied key.

The exact return type is the Map value type.

---

## 16. Index Types

Finite indexed sequence element access accepts integer indices.

Valid:

```phalcom
list[0]
list[-1]
```

Invalid without explicit conversion:

```phalcom
list[1.0]
```

Phalcom MUST NOT implicitly coerce floating-point values or arbitrary objects into sequence indices.

A non-integer index is a type error.

An integer outside the valid element domain raises or produces `IndexError` according to the calling API.

---

## 17. Negative Index Normalization

All finite indexed sequences support negative indexing.

For sequence length `n` and supplied integer `i`:

```text
if i >= 0:
    normalized = i
else:
    normalized = n + i
```

A normalized element index is valid if and only if:

```text
0 <= normalized < n
```

Examples for length 5:

```text
-1 → 4
-2 → 3
-5 → 0
-6 → invalid
```

---

## 18. Negative Index Scope

Negative-index normalization applies consistently to sequence operations whose argument denotes an element coordinate.

Examples include:

```phalcom
sequence[index]
sequence[index] = value
sequence.get(index)
sequence.remove(at: index)
```

Equivalent operations MUST NOT use conflicting negative-index conventions.

---

## 19. Original Versus Normalized Index

When an API supplies the failing/missing index to a user callback, the callback receives the original supplied index rather than the normalized internal index.

Example:

```phalcom
sequence.get(-100) orElse: |missingIndex| {
    ...
}
```

receives:

```text
-100
```

rather than an internal normalized value.

Returned/discovered indices from collection APIs are canonical nonnegative indices unless explicitly documented otherwise.

---

## 20. Maps Do Not Normalize Negative Integer Keys

Map keys are keys, not sequence coordinates.

Therefore:

```phalcom
map[-1]
```

looks up the integer key `-1`.

It MUST NOT reinterpret `-1` relative to Map size.

---

## 21. Insertion Positions

Sequence insertion uses insertion positions rather than existing-element coordinates.

For sequence size `n`, a normalized insertion position is valid if and only if:

```text
0 <= position <= n
```

The position equal to `n` is valid and represents insertion at the end.

---

## 22. Negative Insertion Positions

Negative insertion positions normalize relative to sequence size using the same end-relative principle.

The final normalized insertion position MUST satisfy:

```text
0 <= position <= n
```

Out-of-range insertion positions are not silently clamped.

---

## 23. Insertion Failure Result

Recoverable insertion APIs use:

```phalcom
list.insert(value, at: index)
// Result<Unit, IndexError>
```

Successful insertion returns:

```text
Ok(())
```

conceptually.

An invalid insertion position produces `IndexError` inside `Result`.

The exact `Result` surface constructors are specified elsewhere.

---

## 24. Range Syntax for Slicing

The ratified range syntax includes:

```phalcom
a..b
a..=b
a..
..b
..=b
..
```

For slicing, the primary contiguous slice form uses Range semantics.

General Range syntax and runtime behavior are specified separately.

---

## 25. Read Slicing

Read slicing returns a copy, not a view.

The result preserves the collection family:

```text
List slice
    → List

Tuple slice
    → Tuple

Bytes slice
    → Bytes
```

A zero-copy/view API, if introduced later, is separate and explicit.

---

## 26. Slice Bound Normalization

Slice bounds are more forgiving than strict element indices.

Slice processing supports:

- negative end-relative bounds;
- omitted lower bound;
- omitted upper bound;
- clamping to valid slicing boundaries.

This behavior intentionally differs from strict single-element indexing.

For a sequence of length `n`, slicing operates over boundaries in the interval:

```text
0 ... n
```

after normalization/clamping.

---

## 27. Negative Slice Bounds

Negative slice bounds normalize relative to sequence length.

A bound below the beginning or beyond the end is clamped to the valid slicing interval.

The exact treatment of inclusive upper bounds follows the Range specification.

---

## 28. List Slice Assignment

Contiguous List slice assignment may change List length.

Example:

```phalcom
list[2..4] = [a]
```

shrinks the List if the removed slice contains more elements than the replacement.

Example:

```phalcom
list[2..4] = [a, b, c, d]
```

grows the List.

Replacement length does not need to equal removed length.

---

## 29. Strict List Slice Assignment

Source-level slice assignment syntax is strict.

If the slice assignment is malformed or not applicable, it raises the appropriate language error rather than returning `Result`.

On success, the assignment expression evaluates to the original RHS according to the general subscript-assignment rule.

---

## 30. Recoverable Slice Replacement

The method-form equivalent uses:

```phalcom
list.replace(range, with: replacements)
// Result<Unit, SliceError>
```

This API is suitable when callers want explicit recoverable error handling.

The exact structure of `SliceError` is deferred.

---

## 31. Concrete Collection Transformations Are Eager

Operations invoked directly on concrete collections execute eagerly.

Example:

```phalcom
list.map |value| {
    transform(value)
}
```

returns a concrete result collection immediately.

Concrete collection transformations MUST NOT secretly return lazy pipelines under the same selector.

---

## 32. Lazy Pipelines Are Explicit

Lazy behavior begins through an explicit iterator/pipeline receiver:

```phalcom
list.iter
    .map |value| {
        transform(value)
    }
    .filter |value| {
        predicate(value)
    }
    .toList
```

The receiver determines eager versus lazy semantics.

Phalcom does not need parallel names such as:

```text
map
lazyMap
```

for the same conceptual transformation.

---

## 33. Standard Transformation Vocabulary

The ratified transformation names are:

```phalcom
collection.map |value| {
    ...
}
```

```phalcom
collection.filter |value| {
    ...
}
```

```phalcom
collection.flatMap |value| {
    ...
}
```

```phalcom
collection.each |value| {
    ...
}
```

These names are preferred over historical aliases such as `collect` or language-specific legacy terminology.

---

## 34. `each`

`each` performs an effect-oriented traversal.

Conceptually:

```text
Iterable<T>.each(
    |T| -> Unit
) -> Unit
```

The callback is expected to type-check as returning `Unit`.

The collection API MUST NOT silently interpret a value-returning callback as a mapping operation.

---

## 35. Indexed Callback Variants

Callback arity is not inspected to select behavior.

Indexed variants use distinct labeled selectors.

Examples:

```phalcom
collection.map indexed: |index, value| {
    ...
}
```

```phalcom
collection.each indexed: |index, value| {
    ...
}
```

Phalcom MUST NOT implement special behavior solely because a closure happens to accept two parameters.

---

## 36. Query Vocabulary

Predicate-qualified query operations use `where:`.

Ratified forms include:

```phalcom
collection.find where: |value| {
    ...
}
```

```phalcom
collection.index where: |value| {
    ...
}
```

```phalcom
collection.any where: |value| {
    ...
}
```

```phalcom
collection.all where: |value| {
    ...
}
```

```phalcom
collection.none where: |value| {
    ...
}
```

```phalcom
collection.count where: |value| {
    ...
}
```

---

## 37. Quantifier Empty-Input Identities

For an empty input:

```text
any
    → false

all
    → true

none
    → true
```

These identities are normative.

---

## 38. Query Short-Circuiting

The following operations short-circuit when their result is determined:

```text
find
any
all
none
```

`count where:` evaluates the predicate for all encountered elements unless the source itself fails.

The precise lazy iterator interaction is specified separately.

---

## 39. `fold`

Explicit-initial accumulation uses:

```phalcom
collection.fold(initial: accumulator) using: |accumulator, value| {
    ...
}
```

Conceptually:

```text
Iterable<T>.fold(
    initial: A,
    using: |A, T| -> A
) -> A
```

For empty input:

```text
fold result
    = initial accumulator
```

The callback is not invoked.

---

## 40. `reduce`

Reduction without an explicit initializer uses:

```phalcom
collection.reduce using: |left, right| {
    ...
}
```

Conceptually:

```text
Iterable<T>.reduce(
    using: |T, T| -> T
) -> Option<T>
```

For empty input:

```text
None
```

For one-element input:

```text
Some(element)
```

and the callback is not invoked.

For two or more elements, reduction proceeds according to collection encounter order unless a specific collection says otherwise.

---

## 41. Mutation API Philosophy

Mutating collection operations do not return the receiver merely to support fluent chaining.

The result should communicate the semantic outcome:

```text
successful command
    → Unit

removed/replaced payload
    → Option<T> or Result<T,E>

count of affected values
    → Int
```

This avoids conflating mutation with builder-style chaining.

---

## 42. List Mutation Results

The following List mutation signatures are ratified:

```text
append(value)
    → Unit

prepend(value)
    → Unit

clear
    → Unit

insert(value, at:)
    → Result<Unit, IndexError>

remove(value)
    → Option<T>

remove(at:)
    → Result<T, IndexError>

popFirst
    → Option<T>

popLast
    → Option<T>

removeAll where:
    → Int

replace(range, with:)
    → Result<Unit, SliceError>

move(from:, to:)
    → Result<Unit, IndexError>

swap(first:, second:)
    → Result<Unit, IndexError>
```

---

## 43. List Removal by Value

```phalcom
list.remove(value)
// Option<T>
```

returns:

```text
Some(removedValue)
```

if a matching element is removed, otherwise:

```text
None
```

The exact equality rule used to match values follows the language's ordinary equality semantics.

Whether removal targets the first matching element is expected for sequence semantics but MUST be confirmed in the final List specification if not otherwise stated.

---

## 44. List Removal by Index

```phalcom
list.remove(at: index)
// Result<T, IndexError>
```

returns the removed element on success.

Negative index normalization applies.

An invalid normalized index yields `IndexError`.

---

## 45. Pop Operations

```phalcom
list.popFirst
list.popLast
```

return:

```text
Option<T>
```

For an empty List:

```text
None
```

For a nonempty List:

```text
Some(removedElement)
```

These operations do not raise merely because the List is empty.

---

## 46. `removeAll where:`

```phalcom
list.removeAll where: |value| {
    predicate(value)
}
```

returns:

```text
Int
```

equal to the number of removed elements.

The relative order of retained elements MUST remain unchanged.

---

## 47. `move` and `swap`

Ratified forms:

```phalcom
list.move(from: source, to: destination)
// Result<Unit, IndexError>
```

```phalcom
list.swap(first: a, second: b)
// Result<Unit, IndexError>
```

Both use normal negative-index normalization.

Invalid indices should use `IndexError` rather than operation-specific error classes.

Implementations SHOULD attach structured metadata sufficient to identify:

- which argument failed;
- original supplied index;
- normalized index if meaningful;
- collection size.

Exact error payload representation is deferred.

---

## 48. `first` and `last`

The ratified convenience accessors are safe:

```phalcom
collection.first
// Option<T>
```

```phalcom
collection.last
// Option<T>
```

for collection families where first/last are meaningful.

Strict sequence access remains available through:

```phalcom
sequence[0]
sequence[-1]
```

Therefore an empty collection is an ordinary absence case for `first`/`last`, not an exceptional condition.

---

## 49. List Equality

List equality is ordered structural equality.

Examples:

```phalcom
[1, 2, 3] == [1, 2, 3]
// true
```

```phalcom
[1, 2, 3] == [3, 2, 1]
// false
```

List equality is family-sensitive under the ratified current direction.

A List does not compare equal to a Tuple merely because their element values line up.

---

## 50. List Hashability

List is mutable and therefore unhashable.

A List MUST NOT be accepted as a hash key under default semantics.

---

## 51. Sorting Vocabulary

Non-mutating sorting uses:

```phalcom
collection.sorted
```

for default ordering where available.

Key extraction uses:

```phalcom
collection.sorted on: |value| {
    key
}
```

Direct comparison uses:

```phalcom
collection.sorted using: |left, right| {
    ...
}
```

The direct comparator returns native `Ordering`.

---

## 52. Ordering Type

Comparator callbacks use:

```text
Ordering
```

rather than arbitrary negative/zero/positive integers.

Conceptually:

```phalcom
enum Ordering {
    less
    equal
    greater
}
```

Exact enum case naming/casing may follow global enum-style decisions.

Comparator semantics MUST use the native Ordering result.

---

## 53. In-Place List Sorting

List provides mutating sorting forms:

```phalcom
list.sort
```

```phalcom
list.sort on: |value| {
    key
}
```

```phalcom
list.sort using: |left, right| {
    ...
}
```

Successful in-place sorting returns:

```text
Unit
```

The sort operation does not return the List receiver for chaining.

---

## 54. `sorted` Versus `sort`

The distinction is:

```text
sorted
    → returns a new sorted collection

sort
    → mutates List in place and returns Unit
```

Collection-family preservation for `sorted` follows the relevant collection specification.

---

## 55. Grouping

Grouping uses:

```phalcom
collection.group by: |value| {
    key
}
```

and returns:

```text
Map<K, List<T>>
```

Each group contains source values in source encounter order.

The resulting Map preserves first-seen key order because default Map preserves insertion encounter order.

---

## 56. Grouping Duplicate Keys

Grouping deliberately accumulates multiple source values under the same group key.

This is not treated as a duplicate-key error.

The first occurrence creates the group; subsequent equal keys append to the existing group List.

---

## 57. Partitioning

Partitioning uses:

```phalcom
values.partition where: |value| {
    predicate(value)
}
```

and returns a two-element Tuple.

Canonical order:

```text
first
    → values for which predicate is true

second
    → values for which predicate is false
```

Example:

```phalcom
const (accepted, rejected) =
    values.partition where: |value| {
        predicate(value)
    }
```

Each side preserves source encounter order.

---

## 58. Map Entry Type

Map entries use:

```text
Entry<K, V>
```

as an immutable semantic entry type.

Each entry exposes:

```phalcom
entry.key
entry.value
```

Entry destructuring is supported by the collection/destructuring model:

```phalcom
for (key, value) in map.entries {
    ...
}
```

The complete destructuring rules are specified separately.

---

## 59. Map Views

Map exposes:

```phalcom
map.keys
map.values
map.entries
```

as lightweight view objects rather than eagerly copied Lists.

These views preserve Map encounter order.

The exact live/snapshot/fail-fast mutation behavior remains deferred.

---

## 60. Basic Conversion Vocabulary

Ratified conversion names include:

```phalcom
iterable.toList
iterable.toSet
```

Map conversion from entries uses:

```phalcom
entries.toMap
```

with duplicate rejection.

---

## 61. Safe `toMap`

Conceptually:

```text
Iterable<Entry<K,V>>.toMap
    → Result<Map<K,V>, DuplicateKeyError<K>>
```

If duplicate equivalent keys are encountered, conversion fails rather than silently choosing one association.

No implicit first-wins or last-wins rule exists.

---

## 62. `toMap merging:`

Explicit conflict resolution uses:

```phalcom
entries.toMap merging: |existing, incoming| {
    ...
}
```

The merge callback resolves duplicate-key value conflicts explicitly.

When explicit conflict resolution is supplied, the conversion does not fail merely because the same key is encountered again.

The exact callback invocation order follows source encounter order.

---

## 63. Association Construction With Multiple Closures

A ratified direction for multi-closure association construction is:

```phalcom
users.associate
    key: |user| {
        user.id
    }
    value: |user| {
        ...
    }
```

Default duplicate-key behavior should reject collisions rather than silently overwrite.

Conceptually:

```text
Result<Map<K,V>, DuplicateKeyError<K>>
```

The exact final API is subject to the broader collection-conversion specification, but the duplicate-safety principle is ratified.

---

## 64. Collection API Naming Style

Collection APIs use familiar modern verbs.

Preferred vocabulary includes:

```text
map
filter
flatMap
each
find
reduce
fold
group
sorted
insert
remove
replace
```

Phalcom does not adopt historical Smalltalk names solely for authenticity.

---

## 65. Label Use in API Design

Labels are used when they communicate non-obvious semantic roles.

Examples:

```phalcom
list.insert(value, at: index)
list.replace(range, with: replacements)
list.move(from: source, to: destination)
```

Labels SHOULD NOT be added decoratively where the role is already obvious:

```phalcom
list.append(value)
list.contains(value)
```

Ordinary labeled arguments remain inside parentheses.

Trailing syntax is reserved for closure arguments according to the closure specification.

---

## 66. Closure Placement

Collection callbacks occur last in selector shape where practical.

Examples:

```phalcom
collection.map |value| {
    ...
}
```

```phalcom
collection.find where: |value| {
    ...
}
```

```phalcom
collection.fold(initial: zero) using: |acc, value| {
    ...
}
```

This supports consistent trailing-closure syntax.

---

## 67. Callback Failure Propagation

Collection APIs do not automatically wrap callback failures in `Result`.

If a callback fails according to the ordinary language exception/error model, that failure propagates normally.

For example:

```phalcom
collection.map |value| {
    mayFail(value)
}
```

does not become a `Result` merely because `mayFail` can fail.

`Result` is used where the collection operation itself models a recoverable semantic failure.

---

## 68. Selector-Based Callback Variants

Because Phalcom dispatch is selector-based, semantic variants must be represented by selector differences rather than runtime closure inspection.

Valid distinction:

```phalcom
collection.map |value| {
    ...
}
```

versus:

```phalcom
collection.map indexed: |index, value| {
    ...
}
```

Invalid design:

```text
choose behavior by closure arity
```

The runtime MUST NOT inspect callback arity to decide which collection operation the programmer intended.

---

## 69. Collection Encounter Order

Whenever an operation depends on iteration/encounter order, it uses the source collection's defined encounter order.

Examples include:

- eager `map`;
- eager `filter`;
- `fold`;
- `reduce`;
- grouping member order;
- partition output order;
- non-keyed `first`/`last`.

Collection families with unspecified encounter order, if introduced later, naturally produce correspondingly unspecified operation order unless an operation imposes an order explicitly.

---

## 70. Family Preservation

Where already ratified:

```text
List slice
    → List

Tuple slice
    → Tuple

Bytes slice
    → Bytes
```

Concrete eager transformations SHOULD preserve the most natural concrete family where the operation's semantics allow it.

The complete return-family matrix is deferred to per-collection specifications.

---

## 71. Error Families

The following error families are ratified by use:

```text
IndexError
KeyError
SliceError
DuplicateKeyError<K>
```

Exact inheritance, fields, rendering, and diagnostic formatting are specified elsewhere.

Implementations SHOULD carry structured metadata rather than relying only on rendered strings.

---

## 72. Deferred Issues

The following are intentionally deferred:

1. complete Set and `ImmutableSet` APIs;
2. Set encounter-order semantics and equality/hash rules;
3. Map equal-but-not-identical key replacement behavior;
4. mutation during iteration and view semantics;
5. complete iterator protocol and lazy-pipeline object model;
6. Range and Progression runtime semantics beyond slice use;
7. Bytes element and mutation semantics;
8. full collection capability/protocol hierarchy;
9. cross-family equality beyond ratified List-family sensitivity;
10. generic variance and specialization rules;
11. exact return family for every eager transformation;
12. destructuring rules;
13. printing/debug representation;
14. complete conversion matrix;
15. exact error object schemas;
16. sort stability unless explicitly settled elsewhere;
17. precise semantics of `remove(value)` if multiple equal values exist, pending final List completion;
18. exact `first`/`last` applicability across every collection family.

Future specifications MAY refine these areas but MUST preserve the ratified semantics here unless explicitly superseded.

---

## 73. Conformance Summary

A conforming Phalcom implementation MUST preserve these core laws:

```text
strict subscript lookup
    → value or raises

safe lookup
    → Option<T>

stored None
    → distinguishable from absence

subscript assignment
    → dispatches []=
    → evaluates to original RHS

finite indexed sequences
    → support negative indices

negative index normalization
    i < 0 → n + i

Map integer keys
    → never reinterpreted as negative indices

insertion position
    → valid in 0..size inclusive
    → no clamping

read slicing
    → copy
    → family-preserving

List slice assignment
    → may change length

concrete transformations
    → eager

iterator transformations
    → lazy only through explicit iterator receiver

map/filter/flatMap/each
    → canonical transformation vocabulary

find/index/any/all/none/count
    → qualified with where:

fold(initial:, using:)
    → explicit initial accumulator

reduce using:
    → Option<T>
    → empty => None
    → singleton => Some(element)

mutation commands with no payload
    → Unit

List
    mutable
    ordered structural equality
    unhashable

first/last
    → Option<T>

sorted
    → new collection

sort
    → in-place List mutation
    → Unit

comparator
    → Ordering

group by:
    → Map<K,List<T>>
    → first-seen group-key encounter order

partition where:
    → (accepted, rejected)

Map entries
    → Entry<K,V>

Map keys/values/entries
    → lightweight ordered views

toMap
    → duplicate-safe Result

toMap merging:
    → explicit conflict resolution
```
