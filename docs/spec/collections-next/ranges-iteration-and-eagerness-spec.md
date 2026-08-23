# Phalcom Ranges, Iteration, and Eagerness Specification

**Status:** Ratified language design specification
**Scope:** Range syntax and precedence, inclusive/exclusive bounds, one-sided ranges, Range versus Progression, slice-bound use, eager concrete collection operations, lazy iterator pipelines, eager exhaustors, source boundedness classification, compile-time rejection of provably unbounded eager consumption, and boundedness propagation principles already ratified.
**Out of scope:** Full Range/Progression runtime object model, complete iterator protocol, mutation during iteration, exact lazy pipeline implementation, all terminal iterator operations, Range equality/hashability, descending/reversed Range semantics, and generic capability hierarchy.

---

## 1. Purpose

This specification defines the core semantic relationship among:

1. `Range` as a bound/inclusion structure;
2. `Progression` as stepped iteration derived from a Range;
3. eager operations that must consume a source completely;
4. lazy iterator pipelines that defer consumption;
5. static boundedness information used to diagnose impossible eager exhaustion.

The design intentionally separates:

```text
bounds
from
step/iteration behavior
```

and:

```text
concrete collection transformations
from
lazy iterator transformations
```

---

## 2. Normative Terminology

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

### 2.1 Range

A **Range** denotes lower and/or upper bounds together with bound-inclusion semantics.

### 2.2 Progression

A **Progression** denotes stepped iteration over a Range-like domain.

### 2.3 Finite Range

A **finite Range** is statically or dynamically bounded on both ends in a way that yields a finite number of iterated elements for the relevant element domain.

### 2.4 Unbounded Range

An **unbounded Range** lacks a terminating bound in the iteration direction.

Example:

```phalcom
0..
```

### 2.5 Eager operation

An **eager operation** computes its result immediately.

### 2.6 Lazy iterator operation

A **lazy iterator operation** creates or transforms an iterator pipeline without consuming all source elements immediately.

### 2.7 Eager exhaustor

An **eager exhaustor** must consume its source until exhaustion before it can complete successfully.

Examples include:

```phalcom
iterator.toList
foo(*iterator)
```

### 2.8 Statically bounded source

A **statically bounded source** is provably finite for the operation under analysis.

### 2.9 Provably unbounded source

A **provably unbounded source** is statically known not to terminate by exhaustion.

### 2.10 Unknown-boundedness source

An **unknown-boundedness source** is one for which finite termination cannot be proven or disproven statically.

---

## 3. Range Syntax

Phalcom supports the following Range forms:

```phalcom
a..b
a..=b
a..
..b
..=b
..
```

The syntax intentionally distinguishes exclusive and inclusive upper bounds.

---

## 4. Half-Open Range

The form:

```phalcom
a..b
```

denotes a lower-inclusive, upper-exclusive Range:

```text
a <= x < b
```

For ordinary ascending integer iteration:

```phalcom
1..4
```

corresponds to:

```text
1, 2, 3
```

subject to the final Range runtime semantics.

---

## 5. Closed Upper-Bound Range

The form:

```phalcom
a..=b
```

denotes a lower-inclusive, upper-inclusive Range:

```text
a <= x <= b
```

For ordinary ascending integer iteration:

```phalcom
1..=4
```

corresponds to:

```text
1, 2, 3, 4
```

subject to the final Range runtime semantics.

---

## 6. One-Sided Ranges

Phalcom supports one-sided Range syntax.

### 6.1 Lower-bounded unbounded-above

```phalcom
a..
```

has a lower bound and no upper bound.

### 6.2 Upper-exclusive one-sided

```phalcom
..b
```

has no lower bound and excludes `b`.

### 6.3 Upper-inclusive one-sided

```phalcom
..=b
```

has no lower bound and includes `b`.

### 6.4 Fully unbounded

```phalcom
..
```

has neither lower nor upper bound.

The exact domains in which each one-sided form can be iterated are deferred to the Range runtime specification.

---

## 7. `...` Is Not a Range Operator

Phalcom does not use:

```phalcom
...
```

for Range syntax.

The token sequence remains available for other language facilities such as spread/rest syntax.

This is a deliberate syntax reservation.

---

## 8. Range Operators Are Non-Associative

Range operators MUST NOT associate.

Therefore:

```phalcom
1..2..3
```

is invalid.

Likewise, combinations such as:

```phalcom
1..=2..3
```

are invalid without explicit grouping and a semantically valid surrounding operation.

This prevents accidental interpretation of a second Range operator as a step.

---

## 9. Range Precedence

Range operators bind:

```text
less tightly than arithmetic
more tightly than assignment
```

Thus arithmetic expressions may naturally form Range endpoints.

Conceptually:

```phalcom
a + 1 .. b * 2
```

groups as:

```phalcom
(a + 1) .. (b * 2)
```

while:

```phalcom
x = a..b
```

groups as assignment of the Range value.

Exact parser precedence numbers are implementation detail.

---

## 10. Range and Progression Are Distinct Concepts

Range and Progression are not the same abstraction.

A Range expresses:

```text
bounds
+
bound inclusion
```

A Progression expresses:

```text
stepped traversal
```

Canonical example:

```phalcom
1..10
// Range
```

```phalcom
(1..10).by(2)
// Progression
```

Phalcom does not use:

```phalcom
1..10..2
```

for stepping.

---

## 11. Why Step Is Not Part of Range Syntax

The separation is semantic rather than cosmetic.

A Range is useful independently of iteration for concepts such as:

- containment;
- slicing;
- interval-like bounds;
- boundary validation.

A Progression adds traversal step semantics.

This avoids overloading Range punctuation with both interval structure and iteration stride.

---

## 12. Integer Range Iteration

Finite integer Ranges may participate naturally in iteration.

For ordinary positive-step iteration, a half-open Range:

```phalcom
a..b
```

iterates values beginning at `a` and stopping before `b`.

An inclusive Range:

```phalcom
a..=b
```

includes `b` when reachable.

The precise semantics for reversed bounds, negative stepping, and non-integer element domains are deferred.

---

## 13. Progression Construction

The canonical progression constructor is:

```phalcom
range.by(step)
```

Example:

```phalcom
(0..10).by(2)
```

conceptually iterates:

```text
0
2
4
6
8
```

The exact method result type, step validation, zero-step error, sign mismatch behavior, and descending behavior are deferred.

---

## 14. Range Use in Slicing

Range values serve as slice descriptors for finite indexed sequences.

Examples:

```phalcom
list[2..5]
list[2..=5]
list[2..]
list[..5]
list[..=5]
list[..]
```

The sequence slicing specification interprets Range bounds as slice boundaries.

Slice normalization is intentionally more forgiving than strict element indexing.

---

## 15. Slice Bound Normalization

For a sequence of length `n`, slice bounds normalize into valid boundary coordinates.

The slicing interval uses boundary positions rather than element-only positions.

Conceptually, valid normalized slice boundaries lie in:

```text
0 <= boundary <= n
```

Negative bounds are interpreted end-relative.

Out-of-range slice bounds are clamped.

This differs intentionally from strict element access, where an out-of-range index is an error.

---

## 16. Negative Slice Bounds

A negative slice bound is normalized relative to sequence length.

For length `n`:

```text
negativeBound
    → n + negativeBound
```

followed by clamping into the valid slice-boundary interval.

Example for length 10:

```text
-1 → 9
-3 → 7
```

subject to inclusive/exclusive upper-bound interpretation.

---

## 17. Slice Results Are Copies

Read slicing returns a new value rather than a zero-copy view.

Ratified family preservation includes:

```text
List slice
    → List

Tuple slice
    → Tuple

Bytes slice
    → Bytes
```

A future explicit view API may provide zero-copy behavior.

Range syntax itself does not imply view semantics.

---

## 18. Concrete Collection Operations Are Eager

Operations invoked directly on concrete collections execute eagerly.

Example:

```phalcom
const names =
    users.map |user| {
        user.name
    }
```

The mapping operation runs immediately and produces a concrete result collection.

Concrete collection operations MUST NOT silently return lazy iterator stages under their ordinary selector names.

---

## 19. Lazy Behavior Requires an Iterator Receiver

Lazy transformation begins explicitly through an iterator/pipeline receiver.

Example:

```phalcom
const names =
    users.iter
        .map |user| {
            user.name
        }
        .filter |name| {
            !name.empty?
        }
```

The result remains lazy until consumed by a terminal/eager operation.

This explicit receiver distinction replaces duplicate names such as:

```text
map
lazyMap
```

---

## 20. Receiver Determines Eagerness

The same transformation vocabulary may be used on concrete and lazy receivers.

Example:

```phalcom
list.map |x| {
    x * 2
}
```

is eager because `list` is concrete.

Example:

```phalcom
list.iter.map |x| {
    x * 2
}
```

is lazy because the receiver is an iterator pipeline.

The semantic mode follows the receiver, not a naming suffix.

---

## 21. Eager Exhaustors

Some operations cannot complete until the source is exhausted.

Examples include:

```phalcom
iterator.toList
iterator.toSet
foo(*iterator)
```

and any similar complete materialization.

Such operations are eager exhaustors.

An eager exhaustor MUST observe source exhaustion before successful completion.

---

## 22. Positional Expansion Is an Eager Exhaustor

For:

```phalcom
foo(*source)
```

where `source` is expanded through general iteration, Phalcom must determine the final positional arity before selector derivation.

The conceptual process is:

```text
evaluate source
    ↓
consume elements
    ↓
append positional arguments
    ↓
observe source exhaustion
    ↓
determine final positional arity
    ↓
derive selector
    ↓
lookup method
    ↓
invoke method
```

The target method is not called incrementally.

---

## 23. `toList` Is an Eager Exhaustor

For:

```phalcom
source.toList
```

the operation repeatedly consumes source elements and appends them to the result List until source exhaustion.

Only then can the completed List be returned.

If source exhaustion never occurs, the operation does not successfully complete.

---

## 24. No Implicit Truncation

Phalcom MUST NOT impose an arbitrary hidden element limit on eager exhaustors.

An infinite or nonterminating source is not automatically truncated.

Users explicitly introduce a finite bound when required.

Example:

```phalcom
source.take(100).toList
```

---

## 25. Boundedness Classification

For purposes of eager-exhaustor diagnostics, static analysis distinguishes:

```text
statically bounded
provably unbounded
unknown-boundedness
```

These classifications may be represented as compiler metadata rather than public user-visible types.

---

## 26. Statically Bounded Sources

A source is statically bounded when the compiler can prove a finite upper bound on the number of values consumed.

Examples include finite literal collections and finite bounded Ranges.

Conceptually:

```phalcom
[1, 2, 3]
```

is bounded.

```phalcom
0..10
```

is bounded under ordinary finite integer iteration.

A bounded source may be consumed by an eager exhaustor.

---

## 27. Provably Unbounded Sources

A source is provably unbounded when the compiler can establish that exhaustion cannot occur under the current pipeline semantics.

Canonical example:

```phalcom
0..
```

when interpreted as an unbounded ascending integer source.

An eager exhaustor applied to such a source is invalid.

---

## 28. Compile-Time Rejection of Provably Unbounded Exhaustion

Phalcom MUST diagnose a statically provable unbounded eager exhaustion before program execution.

Examples that MUST fail statically:

```phalcom
foo(*(0..))
```

```phalcom
(0..).toList
```

```phalcom
[
    *(0..),
]
```

The diagnostic is semantic, not a runtime resource-limit error.

---

## 29. Unknown-Boundedness Sources

If the compiler cannot prove whether a source terminates, eager exhaustion remains legal.

Example:

```phalcom
someIterator.toList
```

or:

```phalcom
foo(*someIterator)
```

If the source never exhausts at runtime, the operation never successfully completes unless:

- interrupted;
- failed by the source;
- terminated by another explicit runtime mechanism.

The language does not reject unknown-boundedness merely because nontermination is possible.

---

## 30. Boundedness Through `take`

A finite `take(n)` introduces a finite upper bound when `n` is statically or semantically finite and valid.

Example:

```phalcom
(0..)
    .take(10)
    .toList
```

is valid.

Likewise:

```phalcom
foo(*((0..).take(3)))
```

is valid.

The resulting source is statically bounded by the take count.

---

## 31. Boundedness Through `map`

A pure `map` transformation does not change source cardinality.

Therefore:

```phalcom
(0..)
    .map |x| {
        x * 2
    }
```

remains provably unbounded.

Applying an eager exhaustor remains invalid:

```phalcom
(0..)
    .map |x| {
        x * 2
    }
    .toList
```

MUST be diagnosed statically when the compiler tracks the unboundedness.

---

## 32. Boundedness Through `filter`

Filtering does not by itself prove a finite output from an unbounded source.

Example:

```phalcom
(0..)
    .filter |x| {
        x.even?
    }
```

remains unbounded in the ordinary case.

More generally, arbitrary predicate semantics may make cardinality reasoning undecidable.

The compiler SHOULD use only sound boundedness facts.

It MUST NOT claim boundedness merely because a filter might eventually reject all later elements.

---

## 33. `takeWhile` and Unknown Boundedness

A predicate-dependent limiter such as:

```phalcom
source.takeWhile |x| {
    predicate(x)
}
```

may terminate but cannot generally be proven to do so.

When the source is otherwise unbounded, the resulting boundedness is typically unknown unless the compiler can prove termination.

Therefore:

```phalcom
(0..)
    .takeWhile |x| {
        externalCondition(x)
    }
    .toList
```

is legal under unknown-boundedness semantics.

It may still fail to terminate dynamically.

---

## 34. Short-Circuit Terminal Operations

Not every operation on an unbounded source is an eager exhaustor.

Operations that can terminate after observing a prefix MAY be meaningful on unbounded sources.

Examples may include:

```text
find
any
all
none
first
```

depending on predicate/data behavior.

Such operations are not rejected merely because the source is unbounded.

However, some may still fail to terminate dynamically if their stopping condition is never met.

The precise iterator-terminal specification is deferred.

---

## 35. `reduce` and `fold` Over Unbounded Sources

Ordinary full-source `reduce` and `fold` are eager exhaustors when used as terminal operations over a lazy source.

Therefore they cannot successfully finish on a provably unbounded source unless their semantics are later explicitly extended with short-circuiting forms.

Conceptually, the compiler should reject:

```phalcom
(0..).iter.fold(initial: 0) using: |acc, x| {
    acc + x
}
```

when the source is statically known unbounded and the operation requires full exhaustion.

Exact receiver/method syntax depends on the final iterator API.

---

## 36. Materialization Over Unbounded Progressions

A Progression derived from an unbounded Range may remain provably unbounded.

Example:

```phalcom
(0..).by(2)
```

is unbounded.

Therefore:

```phalcom
(0..).by(2).toList
```

MUST fail statically once the progression's unboundedness is known.

A finite bound introduced later may make it valid.

---

## 37. Selector Derivation and Infinite Expansion

The selector-based dispatch model makes infinite positional expansion especially important.

For:

```phalcom
foo(*source)
```

Phalcom cannot derive the call selector until positional arity is known.

Therefore an unbounded expansion is not merely "a call that keeps passing arguments."

The call itself cannot be dispatched.

This is why:

```phalcom
foo(*(0..))
```

is semantically impossible to complete and is statically rejected when recognized.

---

## 38. Failure During Eager Consumption

If a source fails before exhaustion during an eager exhaustor:

- materialization fails;
- argument-pack construction fails;
- selector derivation does not occur for a call expansion;
- no partial successful result is returned.

Example:

```phalcom
foo(*source)
```

where `source` raises after producing three values does not call `foo` with three arguments.

The failure propagates.

---

## 39. Resource Exhaustion Is Not Language-Level Termination

For unknown-boundedness sources, a nonterminating eager exhaustor may eventually encounter resource exhaustion.

Such failure is not the semantic meaning of the operation.

The semantic meaning remains:

```text
consume until source exhaustion
```

The runtime MAY fail due to memory or system limits, but the specification does not define a collection-specific result based on such exhaustion.

---

## 40. Lazy Pipelines Do Not Materialize by Themselves

Creating a lazy pipeline MUST NOT eagerly consume an unbounded source merely to build pipeline structure.

For example:

```phalcom
const p =
    (0..).iter
        .map |x| {
            x * 2
        }
```

must be constructible without iterating all integers.

Consumption begins only when the pipeline is advanced or passed to an eager/terminal consumer.

---

## 41. Side Effects and Laziness

A callback inside a lazy pipeline executes when its corresponding source element is consumed, not when the pipeline stage is declared.

Example:

```phalcom
const p =
    values.iter.map |x| {
        log(x)
        x * 2
    }
```

constructing `p` does not itself imply that all `log(x)` calls occur immediately.

The complete timing guarantees are deferred to the iterator specification.

---

## 42. Concrete Eager Transformations and Side Effects

By contrast:

```phalcom
values.map |x| {
    log(x)
    x * 2
}
```

executes eagerly over the concrete collection.

Callback invocation follows source encounter order unless otherwise specified by the collection.

---

## 43. Range as Data Versus Iteration

Constructing a Range value is not itself iteration.

Example:

```phalcom
const r = 0..
```

is legal.

The fact that `r` is unbounded does not constitute an error.

The error arises only when a context demands complete eager exhaustion, such as:

```phalcom
r.toList
```

or:

```phalcom
foo(*r)
```

---

## 44. Fully Unbounded Range as Data

The fully unbounded Range:

```phalcom
..
```

may serve as a bound descriptor, especially in slicing or APIs that interpret omitted bounds.

Its existence does not imply that it is directly iterable in every context.

Whether `..` itself implements iteration is deferred.

---

## 45. Slice Use Does Not Require Exhaustion

Using an unbounded Range as a slice descriptor is not equivalent to iterating the Range.

Example:

```phalcom
list[2..]
```

means:

```text
slice from boundary 2 through the sequence end
```

It does not mean "iterate the unbounded Range 2.. and materialize it."

This distinction MUST be preserved.

---

## 46. Range Object Identity Versus Slice Interpretation

The same Range syntax may produce a Range value that is interpreted differently by consuming APIs.

For slicing, a sequence interprets Range bounds relative to sequence size.

For iteration, a Range/Progression follows its iteration semantics.

The Range object remains a bound structure; consumers determine the relevant operation.

---

## 47. Range Bound Evaluation Order

Range endpoint expressions are evaluated according to ordinary lexical expression evaluation.

For:

```phalcom
lower() .. upper()
```

`lower()` is evaluated before `upper()`.

For one-sided ranges only the present endpoint expression is evaluated.

Exact error propagation follows ordinary expression semantics.

---

## 48. Inclusive Upper Bounds in Slicing

The form:

```phalcom
sequence[a..=b]
```

includes the element at upper index `b` when that bound refers to a valid element after normalization.

The slicing implementation may internally convert an inclusive element endpoint to an exclusive boundary endpoint.

Such lowering is implementation detail.

Clamping and negative-bound normalization still apply.

---

## 49. Empty Slice Results

A slice whose normalized bounds select no elements produces the corresponding empty collection-family value.

Examples may include:

```phalcom
list[3..3]
```

producing an empty List.

For Tuple, an empty resulting slice is a zero-product and therefore normalizes to `Unit` under the separate product normalization specification.

This consequence follows from the ratified zero-product model.

---

## 50. Range and Progression Type Distinction

Type reflection MUST be able to distinguish a Range from a Progression when both exist as runtime values.

Conceptually:

```phalcom
typeOf(1..10)
// Range-like type
```

```phalcom
typeOf((1..10).by(2))
// Progression-like type
```

The exact generic type parameters and names are deferred.

---

## 51. No Hidden Step in Range Equality or Syntax

Because step is not part of Range, two Range values are not distinguished by some hidden iteration step field.

Any stepping semantics belong to Progression.

The final Range equality/hash specification must respect this conceptual division.

---

## 52. Static Diagnostic Quality

When rejecting a provably unbounded eager exhaustor, diagnostics SHOULD identify:

- the eager operation;
- the source known to be unbounded;
- the reason complete exhaustion is required;
- a likely explicit bounding operation where useful.

Example diagnostic concept:

```text
cannot expand provably unbounded source into call arguments:
final positional arity is required before selector dispatch
```

The exact wording is implementation-defined.

---

## 53. Compiler Boundedness Metadata

A compiler MAY represent boundedness with internal metadata conceptually similar to:

```text
Boundedness =
    Bounded(optionalKnownUpperCount)
    | Unbounded
    | Unknown
```

This is not required as a public type.

The implementation MAY track more precision, such as exact cardinality.

The semantic obligations are only that statically provable unbounded eager exhaustion is rejected and unknown cases are not rejected unsoundly.

---

## 54. Boundedness Propagation Principles

Known iterator transformations SHOULD propagate boundedness information soundly.

Ratified principles include:

```text
map(Bounded)
    → Bounded

map(Unbounded)
    → Unbounded

take(finite n, any source)
    → Bounded by n

ordinary filter(Bounded)
    → Bounded

ordinary filter(Unbounded)
    → not automatically Bounded

takeWhile(...)
    → generally Unknown unless provable
```

The complete transfer function for every iterator combinator is deferred.

---

## 55. Cardinality Versus Boundedness

Boundedness does not require knowing the exact result size.

A filtered finite collection may have unknown exact cardinality but is still bounded.

Example:

```phalcom
[1, 2, 3, 4]
    .iter
    .filter |x| {
        predicate(x)
    }
```

has at most four output elements even if the exact number is unknown.

Compilers MAY use cardinality upper bounds where convenient.

---

## 56. Infinite Source Safety Is Not a General Type Error

Unbounded values are legitimate values.

Phalcom MUST NOT reject:

```phalcom
const naturals = 0..
```

simply because the Range is unbounded.

Only operations whose semantics require complete exhaustion are statically incompatible with a provably unbounded source.

This preserves useful lazy infinite computations.

---

## 57. Iterator Reuse and Restartability

Whether a lazy iterator/pipeline is one-shot or restartable is not specified here.

Boundedness classification is independent of restartability.

A finite one-shot iterator is still bounded.

An infinite restartable sequence is still unbounded.

---

## 58. Interaction With Collection Expansion

The argument-pack expansion specification defines:

```phalcom
*source
```

as positional/element projection.

When the source is consumed through iteration, all eager-exhaustor and boundedness rules in this specification apply.

For built-in Tuple, `*Tuple` is a direct positional-lane projection and does not rely on full Tuple iteration.

---

## 59. Interaction With Set and Map Construction

Element expansion:

```phalcom
{
    *source,
}
```

into Set is an eager materialization of the expanded source into the Set literal.

Association expansion:

```phalcom
{
    **mapping,
}
```

is bounded according to the association source's own finite mapping semantics.

A future infinite association stream would be subject to the same eager-exhaustor principles if such expansion is generalized.

---

## 60. Interaction With Product Construction

Tuple construction with `*` from a general Iterable is eager because the final Tuple must contain all contributed positional values.

Therefore:

```phalcom
(
    *(0..),
)
```

MUST be rejected statically when `0..` is recognized as provably unbounded.

Likewise, any Tuple construction that requires full consumption of an unbounded source is invalid.

---

## 61. Optimization Freedom

Implementations MAY optimize finite Range iteration, slicing, and bounded eager consumption.

Examples include:

- preallocating List capacity when exact Range cardinality is known;
- deriving final positional arity without materializing an intermediate iterator when cardinality is statically known;
- specialized integer Range loops;
- constant-folding finite Range lengths.

Optimizations MUST preserve evaluation order, failure behavior, and selector semantics.

---

## 62. No Required Intermediate Range Allocation

Range syntax need not allocate a heap object if the compiler can represent it directly in IR or lower it into a consuming operation.

This is implementation detail.

Reflection and first-class Range usage must still preserve the language's observable Range semantics.

---

## 63. Deferred Range Decisions

The following Range/Progression issues remain unresolved:

1. Range equality;
2. Range hashing;
3. exact runtime representation;
4. reversed bounded Range semantics;
5. whether `5..1` is empty, invalid, or otherwise meaningful;
6. descending iteration;
7. negative Progression steps;
8. zero-step behavior;
9. step-sign mismatch;
10. Range membership semantics;
11. Range indexing;
12. Range slicing;
13. non-integer Range domains;
14. Progression equality/hashability;
15. exact type signatures.

These MUST be decided in a later Range runtime specification.

---

## 64. Deferred Iterator Decisions

The following iterator matters remain unresolved:

1. iterator object protocol;
2. `next` return convention;
3. exhausted-state representation;
4. restartability;
5. one-shot iterators;
6. mutation during iteration;
7. pipeline fusion;
8. caching;
9. complete terminal operation set;
10. infinite-pipeline short-circuit details;
11. exact boundedness propagation for all combinators;
12. async iteration, if any;
13. user-defined Iterable capability design.

Future specifications MAY refine these areas but MUST preserve the ratified eager/lazy and boundedness rules here unless explicitly superseded.

---

## 65. Conformance Summary

A conforming Phalcom implementation MUST satisfy:

```text
a..b
    → lower-inclusive, upper-exclusive Range

a..=b
    → lower-inclusive, upper-inclusive Range

a..
..b
..=b
..
    → supported one-sided/unbounded Range forms

...
    → not a Range operator

Range operators
    → non-associative
    → lower precedence than arithmetic
    → higher precedence than assignment

Range
    → bounds/inclusion concept

Progression
    → stepped iteration concept

(range).by(step)
    → Progression direction

read slicing
    → uses Range bounds
    → supports negative normalization
    → clamps boundaries
    → returns family-preserving copy

concrete collection transformations
    → eager

iterator transformations
    → lazy through explicit iterator receiver

eager exhaustor
    → must observe source exhaustion before success

foo(*iterable)
    → eager exhaustor when positional projection uses iteration
    → final arity required before selector dispatch

toList
    → eager exhaustor

provably unbounded source + eager exhaustor
    → compile-time error

unknown-boundedness source + eager exhaustor
    → legal
    → may fail to terminate dynamically

take(finite n)
    → introduces finite bound

map over unbounded source
    → remains unbounded

filter
    → does not unsafely prove boundedness

constructing an unbounded Range
    → legal

using an unbounded Range as a slice descriptor
    → does not imply iterating it
```
