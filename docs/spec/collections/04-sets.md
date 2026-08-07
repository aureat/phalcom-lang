# Set Specification

## 1. Status

The set design was not deeply ratified in the preceding discussion. This document supplies a deliberately conservative normative candidate so the collection family can be evaluated together.

Rules in this document are **PROVISIONAL** unless explicitly marked otherwise.

## 2. Definition

A Set is a finite collection of unique values with membership semantics independent of insertion position.

```text
Set<T> = finite mathematical set of values satisfying T
```

A Set has:

- no positional lane;
- no labeled lane;
- no argument-pack interpretation;
- no duplicate elements under Set equality.

## 3. Construction

Until literal syntax is ratified, the normative construction form is:

```phalcom
const values = Set.new(1, 2, 3)
```

Duplicates collapse:

```phalcom
Set.new(1, 1, 2) == Set.new(1, 2)
```

Potential literal syntaxes remain **OPEN** because `{}` is naturally associated with Records and `#` already participates in Symbols and Selectors.

Candidate syntaxes for review:

```phalcom
set(1, 2, 3)
Set{1, 2, 3}
#{1, 2, 3}
```

No candidate is ratified by this suite.

## 4. Type syntax

The canonical Set Type is generic application:

```phalcom
Set<Int>
Set<String>
```

A Set Type is not tuple-shaped and is never contextually interpreted as a pack schema.

## 5. Equality and hashing

Set equality is extensional:

```text
A = B iff every member of A is in B and every member of B is in A
```

Iteration order MUST NOT participate in equality or hashing.

Every Set element MUST be hashable under the language's ordinary hash/equality contract.

## 6. Iteration order

**PROVISIONAL:** Core Set iteration order is unspecified and MUST NOT be relied upon for semantic correctness.

An implementation MAY preserve insertion order as a quality-of-implementation property, but this is not observable language semantics unless later ratified.

A separate `OrderedSet<T>` may provide ordering guarantees.

## 7. Mutability

Recommended split:

```phalcom
Set<T>        mutable unique collection
FrozenSet<T>  immutable hashable unique collection
```

This naming and split remain **OPEN**.

## 8. Operations

Minimum protocol:

```phalcom
values.contains(value)
values.add(value)
values.remove(value)
values.union(other)
values.intersection(other)
values.difference(other)
values.isSubsetOf(other)
values.isSupersetOf(other)
values.size
```

For immutable Sets, mutating operations return new Sets or are unavailable.

## 9. Spread

**RATIFIED BY GENERAL SPREAD SCOPE:** Value spread syntax is call-only and therefore unavailable in a future Set literal itself.

A constructor call may still use ordinary call expansion:

```phalcom
Set.new(*positionals)
```

Here `*` expands the positional lane into the call to `Set.new`; it is not a Set-specific spread operation. Its legality depends on `Set.new`'s callable domain.

No special Set-literal spread operator is defined. Explicit collection composition uses operations such as `union`.

## 10. Pack conversion

A Set MUST NOT be directly used with `*`, `**`, or `***` expansion because it has no stable positional or labeled lane.

```phalcom
target(*values)   // error: Set has no positional lane
target(**values)  // error: Set has no labeled lane
target(***values) // error: Set is not a pack
```

Explicit conversion is required:

```phalcom
target(*values.toTuple)
```

Because Set iteration order is unspecified, such conversion SHOULD require the caller to accept or establish an order.

## 11. Satisfaction

```phalcom
Set<Int>.satisfiedBy(Set.new(1, 2, 3))
// true
```

Satisfaction checks every member and handles cycles according to the general reflective satisfaction algorithm.

## 12. Open set questions

1. Literal syntax.
2. Mutability model.
3. Iteration-order guarantee.
4. Hashability of mutable Sets.
5. Variance of `Set<T>`.
6. Whether `Set.new(*positionals)` should be a preferred constructor pattern.
7. Whether an immutable Set should be named `FrozenSet`, `ImmutableSet`, or represented through `const` construction.
