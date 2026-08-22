# Phalcom Type System Manifest

## Foundational Type Architecture Specification

**Status:** Design Manifest v1.0

## 1. Purpose and Scope

This document defines the foundational philosophy and semantic
architecture of the Phalcom type system. It is a language-design
document, not an implementation reference. Its purpose is to establish
the principles that all future type features must preserve.

Phalcom is designed as a dynamically executing language with a
statically reasoning semantic layer. Runtime execution remains dynamic,
reflective, and object-oriented, while the compiler, IDE, and tooling
maintain a rich semantic understanding of programs.

Types are persistent semantic information. They are not erased
annotations, compiler-only artifacts, or a separate language layered
above Phalcom.

The type system exists simultaneously for:

-   developer communication of intent;
-   compiler consistency checking;
-   IDE assistance;
-   runtime reflection;
-   contract validation;
-   future optimization.

------------------------------------------------------------------------

# 2. Unified Semantic Model

Phalcom does not contain separate value and type languages.

Values, objects, classes, protocols, and types exist within one semantic
universe.

A class declaration creates a runtime object that can also serve as a
type expression:

``` phalcom
class User {
}

let user: User
```

The meaning of `User` depends on context:

-   as a runtime object, it receives messages;
-   as a type expression, it describes admissible values;
-   as reflection metadata, it exposes semantic information.

The compiler may use optimized representations internally, but the
semantic model remains unified.

------------------------------------------------------------------------

# 3. Fundamental Principles

## 3.1 Types are persistent semantic information

A type describes possible values, supported behavior, and semantic
identity.

Type information is preserved for:

-   static analysis;
-   LSP presentation;
-   reflection;
-   documentation;
-   optimization.

## 3.2 Annotations express developer intent

A declaration:

``` phalcom
age: Int
```

is a claim:

> This entity is intended to satisfy Int.

Annotations do not blindly override compiler reasoning. They participate
as evidence.

## 3.3 Inference provides independent evidence

The compiler gathers evidence from:

-   literals;
-   expressions;
-   control flow;
-   contracts;
-   fields;
-   method signatures;
-   native metadata;
-   runtime observations.

Inference and annotations are compared, not merged by authority.

------------------------------------------------------------------------

# 4. Evidence-Based Type Checking

The type system reasons through evidence.

Example:

``` phalcom
const count: Int = 10
```

Evidence:

    Developer:
        count should be Int

    Compiler:
        literal 10 is Int

Result:

    Proven

Example:

``` phalcom
const value: String = 10
```

Evidence:

    Developer:
        value should be String

    Compiler:
        initializer is Int

Result:

    Refuted

The compiler does not replace developer intent. It proves that the
collected evidence is inconsistent.

------------------------------------------------------------------------

# 5. Knowledge States

Phalcom separates semantic types from knowledge about types.

## Unknown

Unknown means:

    A type exists, but analysis cannot currently determine it.

Unknown is not a normal value type.

It represents incomplete knowledge.

## Dynamic

Dynamic means:

    The developer intentionally disables static reasoning.

Example:

``` phalcom
plugin: Dynamic
```

Dynamic is an explicit escape mechanism.

## Any

Any is the universal value type.

Mathematically:

    Any = Universe

Every value belongs to Any.

Any does not disable analysis.

------------------------------------------------------------------------

# 6. Fundamental Types

## Never

Never is the bottom type.

    Never = empty set

Therefore:

    Never <: T

for every type `T`.

A function returning Never never produces a value.

Example:

``` phalcom
panic("failure") -> Never
```

## Unit

Unit is the singleton type.

Mathematically:

    |Unit| = 1

The unique value may be represented by:

``` phalcom
()
```

and:

``` phalcom
#{}
```

Unit represents existence of one meaningless value, not absence.

------------------------------------------------------------------------

# 7. Type Categories

The foundational type universe contains:

    Type

    ├── Never
    ├── Any
    ├── Unit
    ├── Primitive Types
    ├── Nominal Types
    │   ├── Class Types
    │   └── Protocol Types
    ├── Product Types
    │   ├── Tuple
    │   └── Record
    ├── Sum Types
    │   └── Union
    ├── Function Types
    ├── Applied Types
    └── Type Parameters

------------------------------------------------------------------------

# 8. Classes, Protocols, and Types

Classes are nominal types.

``` phalcom
class User {}
```

creates a runtime class object and a type identity.

Classes are not converted into artificial wrappers such as:

    ClassType(User)

Synthetic types are created for expressions such as:

    List<Int>
    Int | String
    Int -> String

Protocols describe behavioral contracts.

Example:

``` phalcom
protocol Drawable {
    draw()
}
```

A type conforms when it satisfies the required behavior.

Protocols can appear as:

``` phalcom
value: Drawable
```

meaning an existential value, or:

``` phalcom
<T: Drawable>
```

meaning a generic constraint.

------------------------------------------------------------------------

# 9. Type Relations

Phalcom distinguishes:

## Identity

The same declared semantic entity.

## Equivalence

Two expressions denote the same type.

Example:

    () ≡ #{}

## Subtyping

Every value of A satisfies B.

Example:

    Int <: Number

## Consistency

A value may satisfy a type under gradual typing rules.

These relations must remain separate.

------------------------------------------------------------------------

# 10. Nominal and Structural Typing

Phalcom uses a hybrid model.

Classes are nominal:

    Money != Integer

even if their structures match.

Records are structural:

    {name: String}

describes shape rather than declaration identity.

Protocols use structural conformance because they describe capabilities.

------------------------------------------------------------------------

# 11. Algebraic Types

Phalcom supports mathematical composition of types.

Products:

    (Int, String)

represent combined information.

Records:

    {name: String, age: Int}

represent labeled products.

Unions:

    Int | String

are real semantic types.

Examples:

    Option<T> = T | None

    Result<T,E> = Success<T> | Failure<E>

------------------------------------------------------------------------

# 12. Function Types

Function types are first-class semantic types.

Example:

    Int -> String

is a type-producing operation equivalent to:

``` phalcom
Int.->(String)
```

The selector:

    ->(_)

is intrinsic and cannot be overridden.

Function variance follows standard rules:

-   parameters are contravariant;
-   return types are covariant.

------------------------------------------------------------------------

# 13. Generic Types and Kinds

Generic declarations create type constructors.

Examples:

    List : Type -> Type

    Map : Type -> Type -> Type

Application:

    List<Int>

is semantically:

``` phalcom
List.<>(Int)
```

Generic application is an intrinsic operation.

Types themselves have kinds:

    Int : Type

    List : Type -> Type

Higher-kinded types are part of the foundational architecture.

------------------------------------------------------------------------

# 14. Variance

Variance is declaration-site.

Examples:

``` phalcom
class Producer<+T>
class Consumer<-T>
class Box<T>
```

Meaning:

    +T  covariance
    -T  contravariance
    T   invariance

Variance participates in generic subtyping.

------------------------------------------------------------------------

# 15. Runtime Checking Philosophy

Normal execution:

    dynamic execution
    no mandatory type checks

Checked execution:

    runtime validates unresolved obligations

Runtime checks are used for:

-   external boundaries;
-   Dynamic values;
-   reflection;
-   uncertain assumptions.

The runtime provides additional evidence; it is not the primary
authority.

------------------------------------------------------------------------

# 16. Type Expressions as Semantic Operations

Phalcom avoids creating a separate type language.

Type expressions are semantic operations.

Examples:

    List<Int>

is conceptually:

``` phalcom
List.<>(Int)
```

and:

    Int -> String

is:

``` phalcom
Int.->(String)
```

The compiler may optimize these operations, but their meaning remains
part of the unified semantic model.

------------------------------------------------------------------------

# 17. Design Goals

The Phalcom type system prioritizes:

1.  Mathematical coherence.
2.  Reflective visibility.
3.  Gradual adoption.
4.  Strong semantic tooling.
5.  Explicit uncertainty.
6.  Evidence-based validation.
7.  Optimization opportunities.
8.  Compatibility with dynamic execution.

The type system should make programs easier to understand and reason
about without forcing the runtime into a rigid static model.

------------------------------------------------------------------------

# Final Principle

The Phalcom type system is:

> A reflective, gradual, evidence-based type system built over a dynamic
> object runtime. Types are persistent semantic entities. Developers
> express intent through annotations. The compiler gathers evidence and
> proves consistency. Runtime validation supplies additional evidence
> when static reasoning is insufficient. Values, objects, and types
> remain part of one unified semantic universe.
