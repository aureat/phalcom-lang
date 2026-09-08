# Raw source snapshot: collections

> Captured: 2026-09-08
> Source area: collection specifications and collection-program boundaries
> This immutable snapshot preserves the source excerpts used by the compiled concept articles. Program metadata is lifecycle evidence, not proof that proposed work is implemented.

--- docs/spec/collections-next/collections-core-semantics-spec.md ---
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

--- docs/spec/collections-next/product-normalization-and-unit-spec.md ---
# Phalcom Product Normalization and Unit Specification

**Status:** Ratified language design specification
**Scope:** `Unit`, the zero-arity product, normalization of empty Tuple and closed empty Record forms, compile-time and runtime canonicalization, interaction with variadic capture/expansion, runtime representation, and distinctions from empty collection canonicalization.
**Out of scope:** Full Tuple and Record semantics, row-polymorphism syntax, generic specialization internals beyond required normalization behavior, complete collection literal rules, and nominal empty-class semantics except where contrasted with structural products.

---

## 1. Purpose

Phalcom follows the mathematical and type-theoretic interpretation of `Unit` as the canonical zero-arity product.

Tuple and Record are both structural product families:

```text
Tuple
    ordered positional coordinates
    +
    ordered labeled coordinates

Record
    unordered Symbol-labeled coordinates
```

For positive arity, these product families are semantically distinct.

At arity zero, there are no coordinates on which positionality, labels, or order can differ. The two empty structural product constructions therefore normalize to the same canonical type and value:

```phalcom
()
#{}
```

Both denote:

```text
Unit
```

This normalization is definitional. It is not an implicit conversion between distinct runtime values.

---

## 2. Normative Terminology

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

### 2.1 Product

A **product** is a structural value formed from zero or more component coordinates.

Tuple and Record are Phalcom's two structural product families.

### 2.2 Zero-arity product

A **zero-arity product** is a product with no component coordinates.

It has exactly one possible value.

### 2.3 Unit

`Unit` is Phalcom's canonical zero-arity product type.

Its unique value is written:

```phalcom
()
```

### 2.4 Product normalization

**Product normalization** is the semantic canonicalization rule by which any closed zero-coordinate structural product is represented as `Unit`.

### 2.5 Definitional equality

Two type/value constructions are **definitionally equal** when semantic elaboration canonicalizes them to the same type/value rather than inserting a conversion between them.

### 2.6 Closed Record

A **closed Record** has an exact, complete field set.

A closed Record with zero fields normalizes to `Unit`.

### 2.7 Open Record row

An **open Record row** denotes a Record shape that may contain additional fields beyond those explicitly known.

A zero-explicit-field open row does not imply zero actual fields and therefore does not normalize to `Unit`.

---

## 3. Unit as the Zero-Product Type

`Unit` is not merely a conventional "no return value" marker.


--- docs/spec/collections-next/ranges-iteration-and-eagerness-spec.md ---
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


--- docs/spec/collections-next/argument-packs-and-expansion-spec.md ---
# Phalcom Argument Packs and Expansion Specification

**Status:** Ratified language design specification
**Scope:** Argument-pack construction, selector derivation context, positional and labeled lanes, expansion operators, variadic capture, forwarding, and eager expansion constraints.
**Out of scope:** Full Tuple semantics, Record and Map type specifications, Symbol grammar in full, iterator protocol details, and generic capability hierarchy except where required to define expansion behavior.

---

## 1. Purpose

This specification defines how Phalcom represents arguments before dispatch and how argument-like structure is expanded, captured, forwarded, and composed.

Phalcom dispatch is selector-based rather than type-based. Consequently, argument-pack construction is a semantic phase that occurs before method lookup. In particular, the ordered sequence of labeled arguments participates in selector identity.

The design separates:

1. source evaluation order;
2. argument-lane construction;
3. selector derivation;
4. method lookup;
5. parameter binding.

These concepts MUST NOT be conflated by an implementation.

---

## 2. Normative Terminology

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

### 2.1 Positional lane

The **positional lane** is the ordered sequence of unlabeled argument values in an argument pack or Tuple-like argument structure.

### 2.2 Labeled lane

The **labeled lane** is the ordered sequence of `(Symbol, Value)` labeled arguments in an argument pack or Tuple-like argument structure.

Labels are Symbols. No implicit String-to-Symbol conversion occurs.

### 2.3 Argument pack

An **argument pack** contains two logically distinct lanes:

```text
ArgumentPack {
    positional: [Value, ...]
    labeled:    [(Symbol, Value), ...]
}
```

The positional lane precedes the labeled lane semantically.

### 2.4 Lane projection

A **lane projection** extracts contribution suitable for one or both argument lanes from a source value.

The three expansion operators select projections:

```text
*source
    positional/element projection

**source
    labeled/association projection

***source
    complete two-lane argument projection
```

### 2.5 Encounter order

**Encounter order** is the stable traversal order exposed by an ordered source for expansion.

Encounter order is distinct from equality semantics. A Record or Map may have order-insensitive equality while still preserving a stable encounter order used by `**`.

### 2.6 Eager exhaustor

An **eager exhaustor** is an operation that must consume a source until exhaustion before it can complete successfully.

Examples include positional expansion from a general Iterable and materialization operations such as `toList`.

### 2.7 Boundedness classifications

A source may be classified statically as:

- **statically bounded** — known to terminate after a finite number of elements;
- **provably unbounded** — known not to terminate by exhaustion;
- **unknown-boundedness** — termination cannot be proven either way.

---

## 3. Dispatch Context

Phalcom dispatch is purely selector-based.

--- docs/implementation/COLL001-product-model/PROGRAM.md ---
---
id: COLL001
category: COLL
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# COLL001 — collection product model

This program owns the syntax, runtime representation, construction, and
interoperability of Unit, Tuple, and Record product values.

--- docs/implementation/COLL002-map-model/PROGRAM.md ---
---
id: COLL002
category: COLL
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# COLL002 — map model

This program owns mutable Map semantics, ordered map views, Record conversion,
and map-literal construction.

--- docs/implementation/COLL003-indexed-ranges/PROGRAM.md ---
---
id: COLL003
category: COLL
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# COLL003 — indexed ranges

This program owns strict and safe indexed access, range representation, and
range-based slicing and replacement.

--- docs/implementation/COLL004-collection-traversal/PROGRAM.md ---
---
id: COLL004
category: COLL
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# COLL004 — collection traversal

This program owns eager collection operations, list mutation helpers, lazy
iterator pipelines, range iteration, and boundedness diagnostics.

--- docs/implementation/COLL005-argument-expansion/PROGRAM.md ---
---
id: COLL005
category: COLL
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# COLL005 — argument expansion

This program owns positional and labeled expansion, argument-pack assembly,
rest capture, and expansion completion fixes.
