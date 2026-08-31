# Phalcom Algebraic Data Types and Generalized Algebraic Data Types

**Status:** Ratified language-design baseline  
**Scope:** First-class nominal algebraic data types (`enum`), variant constructors, exact case types, pattern refinement, exhaustiveness, and generalized algebraic data types (GADTs)  
**Audience:** Phalcom compiler, semantic analyzer, parser, diagnostics, LSP, runtime, standard-library, and tooling implementers  
**Normative intent:** This document specifies language semantics. Runtime layout optimizations are permitted but, unless explicitly stated otherwise, are not semantic guarantees.

---

## 1. Summary

Phalcom introduces first-class algebraic data types through `enum`.

An `enum` is a **nominal, closed sum type**. Each `@variant` declaration contributes one case to that sum. Every variant has a constructor shape described using Phalcom's ordinary positional/labeled parameter syntax, and every payload-bearing variant induces a specialized callable **variant constructor**.

Example:

```phalcom
enum Option<T> {
    @variant Some(const _ value: T) {
        unwrap -> T {
            value
        }
    }

    @variant None

    isSome -> Bool {
        match self {
            Some(_) => true
            None => false
        }
    }

    @class
    fromNullable(_ value: T?) -> Option<T> {
        ...
    }
}
```

This declaration introduces:

- the nominal type family `Option<T>`;
- the exact case type `Option.Some<T>`;
- the exact case type `Option.None<T>`;
- the payload-bearing variant constructor `Option.Some`;
- the nullary singleton case value `Option.None`;
- enum-wide instance behavior such as `isSome`;
- exact-case behavior such as `Option.Some<T>.unwrap`;
- class-side behavior such as `Option.fromNullable`.

Phalcom remains governed by its uniform semantic object model:

> **Everything is an object semantically; objecthood does not require a uniform physical object representation.**

An enum value is therefore an object in the language semantics even if the compiler represents it as a discriminant, an immediate value, a tagged payload, a niche-encoded pointer, or another optimized representation.

GADTs extend the same model. A variant may explicitly state a more specific result type:

```phalcom
enum Expr<T> {
    @variant Int(const _ value: Int) -> Expr<Int>
    @variant Bool(const _ value: Bool) -> Expr<Bool>

    @variant Add(
        const _ left: Expr<Int>,
        const _ right: Expr<Int>
    ) -> Expr<Int>

    @variant Equal<U>(
        const _ left: Expr<U>,
        const _ right: Expr<U>
    ) -> Expr<Bool>
}
```

Matching an `Expr<T>` against `Int` proves `T = Int`; matching against `Bool` proves `T = Bool`; matching against `Equal<U>` introduces the constructor's local type variable `U` and proves that the scrutinee result index is `Bool`.

ADTs and GADTs are therefore integrated directly into Phalcom's proof-oriented semantic model.

---

# 2. Goals

This design has the following goals.

## 2.1 Native algebraic modeling

Phalcom must support closed nominal sums directly instead of encoding them indirectly through sealed class hierarchies, structural unions, or compiler conventions.

## 2.2 Preserve Phalcom's object semantics

Enum values, variant values, variant constructors, and enum type objects must participate naturally in Phalcom's object/message/type system.

ADTs must not form a separate "functional language island".

## 2.3 Preserve Phalcom selector regularity

Variant constructors use the same positional/labeled invocation vocabulary as ordinary Phalcom callables.

Selector shape is part of variant identity.

## 2.4 Support exact-case behavior

Variants may define methods that are available only when the receiver has been proven to inhabit that exact case.

This makes flow refinement observable through ordinary member lookup and IDE completion.

## 2.5 Support exhaustive proof

A closed enum gives the semantic analyzer a finite case space. Pattern matching can therefore prove exhaustiveness and unreachability.

## 2.6 Support GADTs without a parallel declaration model

A GADT is an enum whose constructors may return more specific indexed members of the enclosing type family.

No separate `gadt` declaration keyword is required.

## 2.7 Leave runtime representation optimizable

Nominality and object semantics must not force heap allocation, object headers, runtime class pointers, or virtual dispatch.

---

# 3. Non-goals

The first design does **not** require the following features.

## 3.1 Extensions

Out-of-line:

```phalcom
extension Option<T> {
    ...
}
```

and:

```phalcom
extension Option.Some<T> {
    ...
}
```

are deferred.

## 3.2 Enum reopening through `class`

The following is not part of the current design:

```phalcom
enum Option<T> {
    ...
}

class Option<T> {
    ...
}
```

Reopening nominal types introduces coherence and module-ordering questions and is deferred.

## 3.3 Open/extensible variants

The ordinary `enum` is closed.

Third-party modules do not add variants to an existing enum.

## 3.4 Implicit promotion of case methods

If every current variant happens to define the same selector, Phalcom does not infer an enum-wide method automatically.

## 3.5 Mandatory advanced layout optimization

Niche optimization, pointer tagging, payload indirection, scalar replacement, unboxing, and similar optimizations are permitted but not required for semantic correctness.

---

# 4. Terminology

## 4.1 Algebraic data type

An algebraic data type is a type assembled from sums and products.

A Phalcom enum is a **nominal closed sum of variant payload products**.

## 4.2 Product

A product combines multiple values simultaneously.

Examples include:

```phalcom
(Int, String)
```

and a record such as:

```phalcom
#{name: String, age: Int}
```

A variant's constructor parameter list also describes a product.

## 4.3 Sum

A sum represents one of a finite set of alternatives.

An enum is a nominal closed sum.

## 4.4 Variant declaration

A `VariantDecl` is the semantic declaration introduced by `@variant`.

It is not a method declaration.

## 4.5 Variant constructor

A `VariantConstructor` is the specialized callable induced by a payload-bearing variant declaration.

It injects a payload product into a nominal sum.

Conceptually:

```text
VariantConstructor : PayloadProduct -> ExactCase
```

where:

```text
ExactCase <: EnclosingEnum
```

## 4.6 Exact case type

Every enum case induces a semantic exact-case/refinement type.

For:

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}
```

the semantic model contains concepts corresponding to:

```text
Option.Some<T>
Option.None<T>
```

with:

```text
Option.Some<T> <: Option<T>
Option.None<T> <: Option<T>
```

These are not required to be ordinary runtime subclasses.

## 4.7 Discriminant

A discriminant is the semantic identity of a variant case within an enum.

Physical storage of a discriminant is implementation-defined.

## 4.8 GADT

A generalized algebraic data type is an indexed nominal sum whose constructors may return different refinements of the enclosing type family.

---

# 5. Type-system taxonomy

The following taxonomy is normative conceptually.

| Construct | Structural / Nominal | Algebraic form |
|---|---|---|
| Tuple | Structural | Positional product |
| Record | Structural | Labeled product |
| `@data class` | Nominal | Product |
| `A | B` | Structural/type-level | Union |
| `enum` | Nominal | Closed sum of products |
| GADT | Nominal | Indexed closed sum of products |

Traits/typeclasses are orthogonal. They describe behavior/capabilities rather than data shape.

---

# 6. Core syntax

The canonical syntax is:

```phalcom
enum Name<GenericParameters> {
    @variant VariantA(...)
    @variant VariantB(...)

    method(...) -> ReturnType {
        ...
    }

    @class
    factory(...) -> Name<...> {
        ...
    }
}
```

A variant may have an exact-case body:

```phalcom
enum Option<T> {
    @variant Some(const _ value: T) {
        unwrap -> T {
            value
        }
    }

    @variant None
}
```

A variant may explicitly state a GADT result type:

```phalcom
enum Expr<T> {
    @variant Int(const _ value: Int) -> Expr<Int>
}
```

---

# 7. Grammar model

The concrete parser grammar may differ internally, but the semantic grammar should distinguish the following declaration forms.

```text
EnumDecl :=
    "enum" TypeName GenericParams? EnumBody

EnumBody :=
    "{" EnumMember* "}"

EnumMember :=
      VariantDecl
    | InstanceMethodDecl
    | ClassMethodDecl
    | OtherPermittedAssociatedDecl

VariantDecl :=
    AttributeListIncludingVariant
    VariantHead
    VariantBody?

VariantHead :=
      Identifier
    | Identifier GenericParams? "(" VariantParamList? ")" VariantResultType?

VariantResultType :=
    "->" Type

VariantBody :=
    "{" VariantMember* "}"

VariantMember :=
    InstanceMethodDecl
```

`@variant` is semantically significant and removes ambiguity between callable-shaped declarations and methods.

The parser must not infer variant status from capitalization.

---

# 8. `@variant` semantics

`@variant` declares a variant case.

Example:

```phalcom
@variant Some(_ value: T)
```

must not be represented merely as a method with an annotation.

Instead the semantic layer should produce a distinct declaration kind, conceptually:

```rust
enum AssociatedDecl {
    Method(MethodDecl),
    Variant(VariantDecl),
    // ...
}
```

A variant declaration should record at least:

```text
VariantDecl
    owner enum
    source identity
    selector identity
    generic parameters
    constructor parameters
    payload fields
    result type
    exact case type
    constructor visibility
    pattern/case visibility
    case-specific methods
```

The precise Rust structure is implementation-specific.

---

# 9. Variant identity

Variant identity is selector-shaped.

For:

```phalcom
enum Value<T> {
    @variant Some(_ value: T)

    @variant Some(
        _ first: T,
        _ second: T
    )

    @variant Some(
        _ value: T,
        at index: Int
    )
}
```

the cases are distinct.

Conceptually:

```text
Value.Some(_)
Value.Some(_,_)
Value.Some(_,at)
```

The identity rule is:

```text
VariantId =
    (
        EnclosingEnumIdentity,
        ConstructorSelectorIdentity
    )
```

The following do **not** participate in variant identity:

- internal parameter names;
- parameter types;
- field mutability;
- implementation body;
- source ordering.

Therefore this is invalid:

```phalcom
enum Invalid {
    @variant Value(_ value: Int)
    @variant Value(_ value: String)
}
```

Both declarations have the same selector identity.

Likewise:

```phalcom
enum Invalid {
    @variant Value(_ value: Int)
    @variant Value(const _ value: Int)
}
```

is a duplicate variant.

---

# 10. Variant parameter products

Variant constructor parameters use Phalcom's ordinary positional/labeled parameter syntax.

Example:

```phalcom
@variant Move(
    _ item: Item,
    from source: Position,
    const to destination: Position
)
```

This declaration contains two related shapes.

## 10.1 Constructor invocation shape

```text
(Item, from: Position, to: Position)
```

Usage:

```phalcom
Move(
    item,
    from: oldPosition,
    to: newPosition
)
```

The selector is:

```text
Move(_,from,to)
```

## 10.2 Stored payload shape

The stored fields are named using internal names:

```text
item: Item
source: Position
destination: Position
```

Thus the compiler maintains a mapping:

```text
positional argument 0 -> field item
label `from`          -> field source
label `to`            -> field destination
```

External labels describe invocation.

Internal names describe storage and local/member access.

---

# 11. Parameter forms

The following forms are supported conceptually.

## 11.1 Positional

```phalcom
_ value: T
```

Meaning:

```text
external label: none
internal field: value
type: T
```

## 11.2 Labeled shorthand

```phalcom
index: Int
```

Meaning:

```text
external label: index
internal field: index
type: Int
```

Construction:

```phalcom
Entry("key", index: 3)
```

## 11.3 Distinct external and internal names

```phalcom
at index: Int
```

Meaning:

```text
external label: at
internal field: index
type: Int
```

Construction:

```phalcom
Entry("key", at: 3)
```

Inside exact-case behavior, the stored member is:

```phalcom
index
```

not `at`.

---

# 12. Positional/labeled ordering

Variant constructor shapes should obey the same positional/labeled ordering rules as other Phalcom callable/product shapes.

Positional parameters precede labeled parameters.

Valid:

```phalcom
@variant Entry(
    _ key: String,
    _ value: Object,
    index: Int,
    source: String
)
```

Invalid:

```phalcom
@variant Entry(
    _ key: String,
    index: Int,
    _ value: Object
)
```

This rule preserves one regular product-shape model across:

- tuples;
- methods;
- constructors;
- variant constructors.

---

# 13. Mutability and `const`

Current Phalcom semantics remain unchanged.

`const` means that the corresponding binding/storage location is immutable.

For now, `const` is **not** redefined to mean "must be compile-time evaluable".

The compiler may nevertheless evaluate any expression at compile time when it can prove doing so is valid.

## 13.1 Variant fields

A variant field may be marked `const`.

```phalcom
@variant Point(
    _ x: Int,
    const _ y: Int,
    label: String,
    const named name: String
)
```

Conceptually:

```text
x     mutable positional field
y     immutable positional field
label mutable labeled field
name  immutable labeled field
```

Field mutability is independent from constructor selector identity.

## 13.2 Discriminant immutability

A value's case identity is not mutated in place.

A `Some(...)` object does not become `None` by overwriting its discriminant.

Changing from one case to another means assigning/replacing the entire enum value in a mutable location.

---

# 14. Nullary variants

A nullary variant denotes a value, not a mandatory zero-argument function call.

Canonical:

```phalcom
Option.None
CompanyKind.LLC
```

Not canonical:

```phalcom
Option.None()
CompanyKind.LLC()
```

Conceptually a nullary variant corresponds to an injection from unit:

```text
() -> Option<T>
```

but source syntax exposes the resulting singleton case value directly.

Generic nullary cases may be statically polymorphic.

For example:

```phalcom
const a: Option<Int> = Option.None
const b: Option<String> = Option.None
```

may use the same runtime singleton representation when implementation constraints allow it.

---

# 15. Variant constructors

A payload-bearing variant induces a specialized callable constructor.

For:

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}
```

`Option.Some` denotes a variant constructor callable.

Its type is conceptually:

```text
<T>(T) -> Option.Some<T>
```

with:

```text
Option.Some<T> <: Option<T>
```

At interfaces that do not expose exact-case types, it may be presented as:

```text
<T>(T) -> Option<T>
```

Variant constructors should be first-class where ordinary callable values are accepted.

Example:

```phalcom
values.map(Option.Some)
```

This is a design requirement unless a concrete implementation limitation temporarily prevents it.

---

# 16. Variant constructors are not methods

Although variant constructors are callable, they are not ordinary methods.

A useful semantic callable taxonomy is:

```text
Callable
    Function
    Closure
    Method
    Constructor
        ClassConstructor
        DataConstructor
        VariantConstructor
```

This taxonomy is conceptual. The implementation may organize its types differently.

The important semantic distinction is that a `VariantConstructor`:

- has a fixed owning enum;
- has a fixed exact case result;
- has a canonical payload mapping;
- contributes to exhaustiveness;
- has pattern identity;
- may establish GADT evidence;
- does not participate in ordinary override chains;
- does not execute arbitrary user-defined constructor code.

---

# 17. Canonical structural construction

Variant construction is a canonical algebraic injection.

For:

```phalcom
@variant Some(_ value: T)
```

construction performs the language-defined equivalent of:

```text
payload value -> exact Some case
```

Arbitrary user code is not part of this operation.

This invariant is important for:

- sound pattern inversion;
- GADT evidence;
- compile-time construction;
- serialization/layout analysis;
- optimizer reasoning;
- smart-constructor invariants.

Custom behavior belongs in ordinary methods.

Example:

```phalcom
enum EmailAddress {
    @variant Valid(const _ value: String)

    @class
    parse(_ input: String) -> Result<EmailAddress, ParseError> {
        if isValidEmail(input) {
            Ok(Valid(input))
        } else {
            Err(ParseError(...))
        }
    }
}
```

`Valid` performs structural construction.

`parse` performs validation.

---

# 18. Private variant constructors

Phalcom supports the semantic capability to make variant construction less visible than the case itself.

The model must distinguish at least:

```text
case/pattern visibility
constructor-call visibility
payload visibility
```

These must not be collapsed into one boolean if doing so prevents:

> publicly matchable case, privately callable constructor.

Example surface syntax may initially use an existing visibility annotation:

```phalcom
enum EmailAddress {
    @private @variant Valid(const _ value: String)

    @class
    parse(...) -> Result<EmailAddress, ParseError> {
        ...
    }
}
```

However the exact annotation semantics should be explicit in the implementation: "private constructor" and "fully private case" are not equivalent concepts.

A fully hidden case affects public exhaustiveness and API abstraction and should be treated as a stronger feature.

---

# 19. Enum-wide instance methods

Ordinary methods declared directly in the enum body belong to the enclosing enum type.

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None

    isSome -> Bool {
        match self {
            Some(_) => true
            None => false
        }
    }
}
```

Conceptually:

```text
Option<T>.isSome : Option<T> -> Bool
```

Every exact case receives enum-wide behavior because:

```text
Option.Some<T> <: Option<T>
Option.None<T> <: Option<T>
```

Thus both are valid:

```phalcom
Option.Some(10).isSome
Option.None.isSome
```

This does not imply virtual inheritance dispatch.

---

# 20. Class-side enum methods

`@class` retains its ordinary Phalcom meaning.

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None

    @class
    fromNullable(_ value: T?) -> Option<T> {
        ...
    }
}
```

Usage:

```phalcom
Option.fromNullable(value)
```

Enums therefore have an associated type/class-side object just as other nominal types do.

The fact that enum values may have compact algebraic representation does not remove class-side behavior.

---

# 21. Variant-specific methods

A variant declaration may contain methods specific to the exact case.

```phalcom
enum Option<T> {
    @variant Some(const _ value: T) {
        unwrap -> T {
            value
        }
    }

    @variant None
}
```

Conceptually:

```text
Option.Some<T>.unwrap : Option.Some<T> -> T
```

The method does not belong to `Option<T>` generally.

Therefore:

```phalcom
const value: Option<Int> = ...
value.unwrap
```

is rejected unless the semantic analyzer has established that `value` is `Option.Some<Int>`.

After refinement:

```phalcom
if let Some(_) = value {
    value.unwrap
}
```

the call is valid.

## 21.1 Field lookup inside variant methods

Internal field names are available as members/bindings according to normal Phalcom receiver/member rules.

For:

```phalcom
@variant Move(
    _ item: Item,
    from source: Position,
    to destination: Position
) {
    distance -> Float {
        source.distance(to: destination)
    }
}
```

the variant body sees:

```text
item
source
destination
```

External labels `from` and `to` are constructor-interface names, not payload-member names.

## 21.2 No implicit enum-wide lifting

If every case defines:

```phalcom
area -> Float
```

Phalcom does not automatically make `area` callable on the enclosing enum type.

An explicit future closed-dispatch/abstract-contract feature may provide this behavior, but it is deferred.

---

# 22. Exact case types

Every variant induces an exact nominal case/refinement type.

Given:

```phalcom
enum Result<T, E> {
    @variant Ok(_ value: T)
    @variant Err(_ error: E)
}
```

the semantic system has concepts equivalent to:

```text
Result.Ok<T, E>
Result.Err<T, E>
```

with subtype/refinement relations:

```text
Result.Ok<T, E> <: Result<T, E>
Result.Err<T, E> <: Result<T, E>
```

Exact case types serve several purposes:

- member lookup for variant-specific methods;
- flow-sensitive refinement;
- diagnostics;
- hover and completion;
- pattern evidence;
- GADT equality propagation;
- unreachable-case reasoning.

Exact case types are not required to be:

- user-subclassable classes;
- heap objects;
- independent runtime class descriptors;
- ordinary inheritance nodes.

They are semantic nominal refinements.

---

# 23. Pattern matching

Pattern matching is a language-level elimination construct.

It is not ordinary method dispatch.

For:

```phalcom
match option {
    Some(value) => ...
    None => ...
}
```

the analyzer:

1. determines the scrutinee type;
2. resolves each pattern constructor against the scrutinee's possible cases;
3. tests case compatibility;
4. introduces payload bindings;
5. refines the scrutinee to the exact case type in the branch;
6. introduces GADT equalities or existentials when applicable;
7. tracks covered portions of the scrutinee type space;
8. proves exhaustiveness or reports uncovered cases;
9. diagnoses unreachable patterns.

---

# 24. Pattern constructor resolution

The canonical identity of a case is qualified:

```text
Option.Some
Option.None
Expr.Int
```

Inside contexts where the expected/scrutinee enum type is known, the language may permit contextual short names:

```phalcom
match option {
    Some(value) => ...
    None => ...
}
```

The resolver should not globally search every enum merely because `Some` or `None` is written unqualified.

A reasonable semantic rule is:

- in pattern position, use the scrutinee's known enum family to resolve short variant names;
- in construction expressions, use explicit qualification unless an expected enum type or explicit import/opening makes short resolution unambiguous.

The exact name-resolution surface may reuse Phalcom's existing namespace/import mechanisms.

---

# 25. Pattern shape

Pattern syntax should mirror the constructor's invocation shape rather than the internal storage names.

For:

```phalcom
@variant Move(
    _ item: Item,
    from source: Position,
    to destination: Position
)
```

construction is:

```phalcom
Move(item, from: a, to: b)
```

and a corresponding pattern is conceptually:

```phalcom
Move(item, from: source, to: destination)
```

The pattern introduces local bindings from the extracted payload.

Internal payload field names remain relevant for member access on an exact-case receiver.

---

# 26. Exhaustiveness

An enum's variant set is closed.

Therefore a match over an enum can be proven exhaustive from the declared case space.

Example:

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}

match value {
    Some(x) => ...
    None => ...
}
```

covers the complete enum.

A missing case should produce an exhaustiveness diagnostic.

An impossible/redundant case should produce an unreachable-pattern diagnostic.

The exhaustiveness engine should be general rather than enum-specific.

It should eventually operate over finite/known spaces such as:

- enum cases;
- `Bool`;
- finite unions;
- nested algebraic patterns;
- GADT-refined case spaces;
- exact-case types.

---

# 27. Unions versus enums

Phalcom's union type operator remains distinct from enum.

Example:

```phalcom
@data
class Person {
    const name: String
    const age: Int
}

@data
class Company {
    const name: String
    const kind: CompanyKind
}

type Entity = Person | Company
```

`Entity` may be exhaustively matchable if it denotes exactly those alternatives.

However:

```phalcom
type Entity = Person | Company
```

is not semantically equivalent to:

```phalcom
enum Entity {
    @variant Person(...)
    @variant Company(...)
}
```

The union's alternatives are independently meaningful nominal types.

The enum's alternatives are cases owned by one nominal sum.

Rule of thumb:

> Use a union when the alternatives exist independently. Use an enum when the alternatives primarily exist as cases of one concept.

---

# 28. Algebraic model

For each variant constructor `Cᵢ` of enum `D`, define its payload product `Pᵢ`.

Then:

```text
Cᵢ : Pᵢ -> D
```

or more precisely:

```text
Cᵢ : Pᵢ -> D.Cᵢ
D.Cᵢ <: D
```

The enum is conceptually:

```text
D ≅ Σᵢ (Tag<Cᵢ> × Pᵢ)
```

where `Σ` is a finite tagged sum.

Example:

```phalcom
enum Example<T, U> {
    @variant Some(_ x: T)
    @variant Some(_ x: T, _ y: U)
    @variant None
}
```

has:

```text
P₀ = (T,)
P₁ = (T, U)
P₂ = ()
```

and conceptually:

```text
Example<T,U>
≈
    Tag<Some(_)>   × (T,)
  + Tag<Some(_,_)> × (T,U)
  + Tag<None>      × ()
```

This is a semantic/algebraic model, not a mandatory ABI.

---

# 29. GADT syntax

A variant may explicitly state a result type.

```phalcom
enum Expr<T> {
    @variant Int(const _ value: Int) -> Expr<Int>
    @variant Bool(const _ value: Bool) -> Expr<Bool>

    @variant Add(
        const _ left: Expr<Int>,
        const _ right: Expr<Int>
    ) -> Expr<Int>

    @variant Equal<U>(
        const _ left: Expr<U>,
        const _ right: Expr<U>
    ) -> Expr<Bool>

    @variant If<U>(
        const condition: Expr<Bool>,
        const then thenExpr: Expr<U>,
        const else elseExpr: Expr<U>
    ) -> Expr<U>
}
```

No separate GADT declaration keyword is needed.

Ordinary ADTs are the special case where every constructor's result is the enclosing enum applied to its ordinary generic parameters.

---

# 30. GADT constructor typing

The semantic representation of every variant should be constructor-centric.

Conceptually:

```text
VariantConstructorSignature {
    generic_parameters
    parameter_product
    result_type
}
```

Examples:

```text
Int :
    Int -> Expr<Int>

Bool :
    Bool -> Expr<Bool>

Add :
    (Expr<Int>, Expr<Int>) -> Expr<Int>

Equal :
    <U>(Expr<U>, Expr<U>) -> Expr<Bool>

If :
    <U>(
        condition: Expr<Bool>,
        then: Expr<U>,
        else: Expr<U>
    ) -> Expr<U>
```

This representation should be used even for ordinary ADTs.

That avoids implementing GADTs as an unrelated feature layered on top of a field-only enum representation.

---

# 31. GADT branch refinement

Suppose:

```phalcom
evaluate<T>(_ expression: Expr<T>) -> T {
    match expression {
        Int(value) => value
        Bool(value) => value

        Add(left, right) =>
            evaluate(left) + evaluate(right)

        Equal(left, right) =>
            evaluate(left) == evaluate(right)

        If(condition, then: thenExpr, else: elseExpr) =>
            if evaluate(condition) {
                evaluate(thenExpr)
            } else {
                evaluate(elseExpr)
            }
    }
}
```

The analyzer should establish branch-specific evidence.

## 31.1 `Int`

Constructor result:

```text
Expr<Int>
```

Scrutinee:

```text
Expr<T>
```

Matching proves:

```text
T = Int
```

and:

```text
expression : Expr.Int
value : Int
```

The branch expression `value` therefore has type `T` under the established equality.

## 31.2 `Bool`

Matching proves:

```text
T = Bool
```

## 31.3 `Add`

Matching proves:

```text
T = Int
left : Expr<Int>
right : Expr<Int>
```

## 31.4 `Equal<U>`

Matching introduces a fresh constructor-local existential/skolem corresponding to `U`:

```text
∃U.
    left : Expr<U>
    right : Expr<U>
```

and proves:

```text
T = Bool
```

The two operands share the same hidden `U`.

## 31.5 `If<U>`

Matching introduces:

```text
U
```

with:

```text
condition : Expr<Bool>
thenExpr : Expr<U>
elseExpr : Expr<U>
```

and proves:

```text
T = U
```

The branch therefore returns `U`, which is equal to required result `T`.

---

# 32. Universal versus existential constructor variables

GADT implementations must distinguish construction-time and match-time interpretations of constructor generics.

For:

```phalcom
@variant Equal<U>(
    _ left: Expr<U>,
    _ right: Expr<U>
) -> Expr<Bool>
```

construction is universally quantified:

```text
Equal : ∀U. (Expr<U>, Expr<U>) -> Expr<Bool>
```

When matching an already-constructed `Equal`, the hidden `U` is discovered existentially:

```text
∃U. Equal(Expr<U>, Expr<U>)
```

This distinction is essential for soundness.

A match branch must not allow an existential constructor-local type to escape into a context where it would be treated as an arbitrary universally chosen type.

---

# 33. Constructor result restrictions

A variant's explicit GADT result must belong to the enclosing enum type family.

For:

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
}
```

the result is valid.

A declaration such as:

```phalcom
enum Expr<T> {
    @variant Invalid(_ value: Int) -> String
}
```

must be rejected because it does not construct the owning nominal sum family.

More complex indexed results may be permitted when they are valid instances of the enclosing family.

The exact normalization/equality rules should follow Phalcom's general type equivalence/proof system.

---

# 34. Exhaustiveness for GADTs

GADT exhaustiveness is indexed by the scrutinee type.

A constructor whose result cannot unify with the scrutinee index is impossible and should not be required.

Example:

```phalcom
const expr: Expr<Int> = ...
```

A `Bool` constructor whose result is `Expr<Bool>` is not a possible inhabitant of `Expr<Int>`.

Therefore:

```phalcom
match expr {
    Int(value) => ...
    Add(left, right) => ...
    // other constructors whose result can be Expr<Int>
}
```

should be checked against the refined constructor space, not blindly against every declaration in `Expr`.

This requires the exhaustiveness engine to query constructor result compatibility.

---

# 35. Pattern evidence is formal proof evidence

Phalcom's semantic checker treats successfully established constructor facts as formal evidence.

If the checker can prove from the scrutinee and constructor result that:

```text
T = Int
```

that equality is authoritative within the branch.

Developer annotations that contradict established evidence must be rejected.

This follows Phalcom's broader semantic principle:

> When the checker can prove a fact, contradictory developer claims are errors. Developer annotations are useful as evidence when the checker cannot otherwise prove a fact, subject to consistency checking.

GADT pattern evidence therefore integrates with the same proof/evidence infrastructure as ordinary control-flow refinement.

---

# 36. Everything is an object semantically

Enums do not weaken Phalcom's universal object model.

Every enum runtime value is a semantic object.

Every variant value is a semantic object.

Every payload-bearing variant constructor is a semantic callable object.

Every nullary variant is a semantic object value.

The enum itself has an associated type/class-side object.

An enum value participates in the same language-level facilities as other values, where supported by its type:

- type membership;
- member lookup;
- message sending;
- trait satisfaction;
- equality/hash protocols;
- reflection;
- erasure to general object types;
- diagnostics and tooling.

The normative principle is:

> **Everything is an object in Phalcom semantically; object semantics do not prescribe physical object representation.**

---

# 37. Objecthood does not imply reference identity

Semantic objecthood must remain distinct from observable reference identity.

A value such as:

```phalcom
Option.Some(10)
```

does not automatically require:

- a unique heap address;
- an object header;
- pointer identity;
- shared aliasing semantics.

Two equal enum values may be represented as copied value bits without a meaningful heap-identity question.

Ordinary reference-oriented classes may still have observable identity according to Phalcom's broader object model.

This distinction is important for efficient algebraic values.

---

# 38. Runtime type objects

An enum has an associated runtime/type object analogous to other nominal Phalcom types.

Conceptually:

```text
Option
    type/class-side object

Option<T>
    nominal instance type family

Option.Some
    associated variant constructor callable

Option.None
    associated nullary case value
```

The precise generic reflection model is governed by Phalcom's broader type-object design.

ADTs should not introduce a parallel runtime metatype system.

---

# 39. Physical representation

Physical representation is implementation-defined unless constrained by an explicit ABI/layout contract.

The canonical conceptual representation is:

```text
discriminant + payload
```

but the compiler may choose any semantics-preserving representation.

Permitted strategies include:

- discriminant-only representation for nullary enums;
- inline tag plus union payload;
- elimination of redundant tags for one-case types;
- niche optimization;
- pointer tagging;
- partial payload indirection;
- recursive indirection;
- stack/register scalarization;
- heap allocation when required;
- dynamic boxing at erasure boundaries.

None of these is required by the language specification unless explicitly standardized later.

---

# 40. No mandatory per-value object header

An enum value does not inherently carry:

- a class pointer;
- a vtable pointer;
- reflection metadata;
- a GC object header.

For example:

```phalcom
enum CompanyKind {
    @variant LLC
    @variant Ltd
    @variant Inc
}
```

may be physically represented by a small integer.

It remains a semantic object capable of ordinary Phalcom behavior.

Similarly:

```phalcom
Option.Some(10).isSome
```

does not require a method table inside the value.

The compiler may statically lower the method invocation to ordinary code operating on the compact representation.

---

# 41. GADT indices are normally erased

A GADT result index is static proof information unless runtime semantics require it.

For:

```phalcom
@variant Int(_ value: Int) -> Expr<Int>
```

the runtime value need not additionally store:

```text
T = Int
```

The constructor identity already determines the static result relation.

Matching the case allows the semantic analyzer to recover the equality.

This is a zero-cost typing principle:

> Stronger static GADT evidence should not automatically introduce runtime metadata.

---

# 42. Runtime evidence when required

Runtime type or trait evidence may be required after type erasure.

Example:

```phalcom
enum Printable {
    @variant Value<T>(_ value: T)
    where T: Show
}
```

If `Printable` existentially hides `T` but branch code must invoke `Show` behavior on the payload, the runtime representation may need to store an implicit `Show<T>` witness.

Conceptually:

```text
Value<T>(
    value: T,
    hidden showWitness: Show<T>
) -> Printable
```

This is not general enum overhead.

It is evidence required by existential semantics.

General rule:

> Runtime evidence is materialized when execution after erasure requires it.

---

# 43. [IGNORE]


---

# 44. Recursive enums

Recursive ADTs are supported semantically.

Example:

```phalcom
enum Tree<T> {
    @variant Empty

    @variant Node(
        const _ value: T,
        const _ left: Tree<T>,
        const _ right: Tree<T>
    )
}
```

The source type need not expose explicit boxing syntax merely to break recursive physical size.

The compiler/runtime may introduce the required indirection.

The exact allocation policy is an implementation concern.

This permits a high-level semantic type to remain algebraic while choosing an efficient recursive representation.

---

# 45. [IGNORE]

---

# 46. Reflection

Reflection should expose semantic declarations rather than accidental physical representation.

Possible enum reflection information includes:

```text
EnumInfo
    nominal identity
    generic parameters
    closed case list
    methods
    class-side methods
```

Possible variant reflection information includes:

```text
VariantInfo
    owning enum
    selector identity
    external labels
    internal field names
    field types
    field mutability
    generic parameters
    declared result type
    visibility
    exact case type
    case methods
```

Reflection should not promise physical tag numbers or payload offsets unless explicit layout reflection is separately requested and standardized.

---

# 47. Diagnostics

The semantic layer should produce specialized diagnostics for ADT/GADT errors rather than collapsing them into generic type mismatch messages.

Important diagnostic categories include:

## 47.1 Duplicate selector-shaped variant

```phalcom
@variant Value(_ x: Int)
@variant Value(_ x: String)
```

Suggested diagnostic:

```text
duplicate variant selector `Value(_)`
variant identity is determined by selector shape, not payload type
```

## 47.2 Invalid GADT result family

```phalcom
@variant Broken(_ value: Int) -> String
```

Diagnostic should explain that a variant of `Expr` must construct an `Expr<...>` result.

## 47.3 Missing match cases

Show concrete uncovered cases after index refinement.

## 47.4 Unreachable GADT case

When a constructor result is incompatible with the scrutinee index, report the equality conflict.

## 47.5 Variant-specific method unavailable

For:

```phalcom
value: Option<Int>
value.unwrap
```

diagnostic should explain:

```text
`unwrap` is defined on `Option.Some<Int>`, not on `Option<Int>`
the receiver has not been proven to be the `Some(_)` case
```

and may suggest pattern refinement.

## 47.6 Private constructor invocation

Explain that the case may be matchable but construction is restricted.

## 47.7 Existential escape

When a constructor-local hidden type escapes its branch, diagnostics should identify the existential source.

---

# 48. Explanation/evidence integration

Phalcom's semantic explanation graph should be capable of recording ADT/GADT reasoning.

Useful evidence nodes include:

```text
ResolvedVariant
MatchedVariant
ExactCaseRefinement
ConstructorResultInstantiation
GadtEquality
IntroducedExistential
ExhaustivenessCoverage
ImpossibleConstructor
PayloadProjection
```

Example reasoning:

```text
expression : Expr<T>
matched constructor `Int`
constructor result = Expr<Int>
scrutinee result = Expr<T>
therefore T = Int
value : Int
branch result Int satisfies required T
```

This evidence should be available to diagnostics, hover explanations, and developer-facing traces without requiring those presentation layers to reimplement semantic reasoning.

---

# 49. LSP behavior

The LSP should consume semantic facts from the canonical semantic layer.

## 49.1 Hover

Hover over a variant declaration may show:

```text
variant Expr.Int(_ value: Int) -> Expr<Int>
constructor: (Int) -> Expr.Int
```

Exact formatting is presentation policy.

## 49.2 Completion

Given:

```phalcom
value: Option<Int>
```

completion should show enum-wide members.

Inside:

```phalcom
if let Some(_) = value {
    value.
}
```

completion should additionally show `Option.Some<Int>` exact-case methods.

## 49.3 Go to definition

`Some` in:

- construction;
- patterns;
- variant references;

should resolve to the same `VariantDecl` identity.

A reference to a case-specific method resolves to the method declaration inside the variant body.

## 49.4 Rename

Renaming a variant must rename constructor and pattern references according to selector identity without conflating same-base-name variants of different selector shapes.

## 49.5 Semantic highlighting

Variant declarations and variant constructor uses may receive their own semantic token category if desirable, but highlighting policy is not normative here.

---

# 50. Name-resolution model

Variant declarations live in the namespace of the owning enum.

Canonical qualified identities:

```text
Option.Some
Option.None
Expr.Int
```

Associated lookup should not be modeled as ordinary dynamic class-side method dispatch.

A variant constructor is an associated declaration that is callable.

This allows:

```phalcom
Option.Some
```

to be a first-class callable without claiming that `Some` is an ordinary `@class` method.

---

51. [IGNORE]

---

# 52. Interaction with `@data`

`@data class` and `enum` share underlying product concepts.

Example:

```phalcom
@data
class Person {
    const name: String
    const age: Int
}
```

is a nominal product.

A variant:

```phalcom
enum Entity {
    @variant Person(
        const name: String,
        const age: Int
    )
}
```

contains a payload product owned by a nominal sum.

They should share compiler machinery where appropriate for:

- stored fields;
- destructuring;
- generated equality/hash;
- product shape;
- field metadata;
- constant evaluation;
- layout.

However, a variant exact-case type should not automatically become an independently extensible ordinary class.

---

# 53. Compile-time constant evaluation

Current `const` syntax means immutable binding/storage.

Compile-time constantness remains an inferred property.

Enum construction is particularly suitable for constant evaluation because variant constructors are canonical structural injections.

For example:

```phalcom
const x = Option.Some(10)
```

may be constant-evaluated when:

- `10` is constant-evaluable;
- the selected representation can be materialized at compile time;
- no runtime-only semantics intervene.

Likewise nullary variants are natural constant values.

This does not redefine `const`; it merely allows the optimizer/evaluator to exploit the expression.

---

# 54. Equality and hashing

The exact standard protocols are outside this specification, but algebraic equality should naturally be definable as:

```text
same variant identity
AND
equal corresponding payload fields
```

when the required payload equality evidence exists.

Different selector-shaped variants with the same base name are different cases.

For example:

```phalcom
Some(1)
```

and:

```phalcom
Some(1, 2)
```

cannot be equal as enum values because their variant identities differ.

Derived hashing should similarly include variant identity and payload hashes.

Physical discriminant numbers must not become the semantic hash contract.

---

# 55. Serialization

Serialization behavior is not automatically derived by this spec.

If serialization is derived later, serialized variant identity should be based on stable semantic names/explicit schema metadata rather than compiler-chosen physical tag values.

Selector-shaped overloads require an unambiguous serialized case identifier.

---

# 56. Control-flow invalidation and mutable payloads

Because current Phalcom permits mutable fields, exact-case and payload facts must obey the existing mutation/effect invalidation model.

However, changing an individual mutable payload field does not change the variant identity.

Therefore a proven exact case can remain valid across payload-field mutation unless some operation can replace the entire containing value or invalidate the receiver binding through aliasing/effects.

The semantic analyzer must distinguish:

```text
case identity stability
payload value stability
binding stability
```

This distinction is especially important for proof reuse.

---

# 57. Case identity and assignment

Given a mutable binding:

```phalcom
let state: State = State.Waiting
```

assignment may replace the entire value:

```phalcom
state = State.Running(...)
```

Any proof that `state` is `State.Waiting` must then be invalidated.

This is ordinary flow-sensitive assignment invalidation.

It is not modeled as an in-place mutation of the enum's discriminant.

---

# 58. Public API evolution

Enums are closed, so adding a public variant may invalidate downstream exhaustive matches.

This is an intentional semantic consequence of closed sums.

Library versioning/documentation should therefore treat adding a publicly visible case as a potentially source-breaking change unless the language later introduces non-exhaustive enum annotations.

Private/hidden cases require separate public exhaustiveness rules and are deferred beyond constructor privacy.

---

# 59. Implementation architecture recommendations

The following are strong implementation recommendations, not surface syntax guarantees.

## 59.1 Keep semantic authority in the semantic layer

`phalcom-semantic` should own:

- enum declaration resolution;
- variant identities;
- constructor signatures;
- exact case types;
- GADT equality evidence;
- exhaustiveness;
- pattern refinement;
- member lookup after refinement.

Parser, LSP, diagnostics, and runtime layers should consume these facts rather than reconstructing them.

## 59.2 Reuse callable parameter shape

Variant constructors should reuse existing parameter/selector machinery where correct.

Do not duplicate parsing and selector computation.

## 59.3 Do not reuse method dispatch semantics

A variant constructor must not be inserted into ordinary method-family/override dispatch merely because it is callable-shaped.

## 59.4 Constructor-first representation

Represent variants semantically by constructor signatures rather than only by field lists.

This is necessary for GADTs.

## 59.5 Exact-case types should be first-class semantic nodes

Avoid representing exact-case facts as unstructured side flags.

They participate in ordinary member lookup and subtype/refinement reasoning.

## 59.6 Exhaustiveness should consume type-space descriptions

Do not hard-code a separate enum-only checker if a general finite-space coverage engine can be built incrementally.

---

# 60. Suggested semantic structures

The exact codebase design should be adapted to existing Phalcom types, but conceptually the semantic system needs equivalents of:

```rust
struct EnumDecl {
    id: NominalTypeId,
    generics: GenericParams,
    variants: Vec<VariantId>,
    instance_methods: MethodSet,
    class_methods: MethodSet,
}

struct VariantDecl {
    id: VariantId,
    owner: NominalTypeId,
    generics: GenericParams,
    selector: SelectorId,
    parameters: Vec<VariantParameter>,
    result_type: Type,
    exact_case_type: Type,
    constructor_visibility: Visibility,
    case_visibility: Visibility,
    methods: MethodSet,
}

struct VariantParameter {
    external_label: Option<Name>,
    internal_name: Name,
    ty: Type,
    mutability: FieldMutability,
}

struct VariantConstructor {
    variant: VariantId,
    callable_signature: CallableSignature,
}
```

This is illustrative, not a mandate to create these exact Rust structures.

---

# 61. Proof rules

The following high-level rules are normative.

## 61.1 Construction

If:

```text
C : P -> D.C
```

and an argument product `p` checks against `P`, then:

```text
C(p) : D.C
```

and by exact-case subtyping:

```text
C(p) : D
```

## 61.2 Match refinement

If:

```text
x : D
```

and branch pattern resolves to `D.C`, then inside the successful branch:

```text
x : D.C
```

plus payload bindings and constructor-result equalities.

## 61.3 GADT result equality

If scrutinee has type:

```text
D<S>
```

and constructor has result:

```text
D<R>
```

then the branch is possible only if `D<S>` and `D<R>` can be made consistent under Phalcom's type equality/unification/proof rules.

Any resulting equalities become branch evidence.

## 61.4 Impossible case

If constructor result and scrutinee type are inconsistent, the case is impossible.

An explicitly written impossible case is unreachable.

An impossible case does not count as a required uncovered case for exhaustiveness.

---

# 62. Examples

## 62.1 Option

```phalcom
enum Option<T> {
    @variant Some(const _ value: T) {
        unwrap -> T {
            value
        }
    }

    @variant None

    isSome -> Bool {
        match self {
            Some(_) => true
            None => false
        }
    }
}
```

## 62.2 Result

```phalcom
enum Result<T, E> {
    @variant Ok(const _ value: T) {
        unwrap -> T {
            value
        }
    }

    @variant Err(const _ error: E) {
        unwrapError -> E {
            error
        }
    }

    isOk -> Bool {
        match self {
            Ok(_) => true
            Err(_) => false
        }
    }
}
```

## 62.3 Selector-shaped variants

```phalcom
enum Lookup<T> {
    @variant Found(const _ value: T)

    @variant Found(
        const _ value: T,
        at index: Int
    )

    @variant Found(
        const _ value: T,
        in collection: Collection<T>
    )

    @variant Missing
}
```

Distinct identities:

```text
Found(_)
Found(_,at)
Found(_,in)
Missing
```

## 62.4 Exact-case method

```phalcom
const result: Result<Int, Error> = ...

match result {
    Ok(_) => {
        result.unwrap
    }

    Err(_) => {
        result.unwrapError
    }
}
```

## 62.5 GADT

```phalcom
enum Expr<T> {
    @variant Int(const _ value: Int) -> Expr<Int>
    @variant Bool(const _ value: Bool) -> Expr<Bool>

    @variant Add(
        const _ left: Expr<Int>,
        const _ right: Expr<Int>
    ) -> Expr<Int>

    @variant Equal<U>(
        const _ left: Expr<U>,
        const _ right: Expr<U>
    ) -> Expr<Bool>

    @variant If<U>(
        const condition: Expr<Bool>,
        const then thenExpr: Expr<U>,
        const else elseExpr: Expr<U>
    ) -> Expr<U>
}
```

Evaluator:

```phalcom
evaluate<T>(_ expression: Expr<T>) -> T {
    match expression {
        Int(value) => value
        Bool(value) => value

        Add(left, right) =>
            evaluate(left) + evaluate(right)

        Equal(left, right) =>
            evaluate(left) == evaluate(right)

        If(condition, then: thenExpr, else: elseExpr) =>
            if evaluate(condition) {
                evaluate(thenExpr)
            } else {
                evaluate(elseExpr)
            }
    }
}
```

---

# 63. Parser requirements

The parser must:

1. recognize `enum` as a nominal type declaration;
2. permit attributes on enum members;
3. parse `@variant` members as `VariantDecl`, not methods;
4. parse positional/labeled variant parameters using shared callable parameter grammar where possible;
5. parse optional explicit variant result types;
6. parse optional variant bodies containing exact-case methods;
7. parse ordinary enum instance methods;
8. parse `@class` enum methods;
9. preserve complete source ranges for enum/variant/parameter/result/body nodes;
10. avoid capitalization-based variant inference.

---

# 64. Resolver requirements

The resolver must:

1. create nominal enum identity;
2. create selector-shaped variant identities;
3. reject duplicate selector identities;
4. resolve parameter types;
5. resolve GADT result types;
6. verify result belongs to owning enum family;
7. create exact case type identities;
8. publish associated variant names;
9. distinguish associated variant lookup from ordinary class-method lookup;
10. resolve contextual short variant names in patterns where appropriate.

---

# 65. Type-checker requirements

The checker must:

1. type-check variant constructor invocation;
2. infer/instantiate constructor generics;
3. produce exact-case constructor result facts;
4. widen exact cases to enclosing enum when required;
5. type-check enum-wide methods with `self : Enum<...>`;
6. type-check case methods with `self : ExactCase<...>`;
7. make payload members available in case methods;
8. refine scrutinees after successful pattern matches;
9. introduce GADT equalities;
10. introduce constructor-local existentials safely;
11. prevent existential escape;
12. prove match exhaustiveness;
13. detect unreachable patterns;
14. invalidate refinements according to assignment/effect rules.

---

# 66. Runtime/compiler requirements

The runtime/compiler must preserve semantic distinctions but is free to choose physical layout.

Minimum viable implementation may use a straightforward representation such as:

```text
tag + payload storage
```

with ordinary boxing where the existing runtime requires it.

A later optimizer may improve representation without changing language semantics.

The first implementation should prioritize:

1. semantic correctness;
2. stable variant identities;
3. correct construction;
4. correct pattern matching;
5. correct method behavior;
6. GADT proof soundness;
7. exhaustiveness;
8. only then advanced representation optimization.

---

# 67. Deferred representation work

The following optimizations are explicitly deferred/optional:

- niche optimization;
- null-pointer Option encoding;
- pointer tagging;
- single-case tag elimination;
- payload size heuristics;
- partially boxed large variants;
- stack allocation;
- escape analysis;
- scalar replacement;
- specialized monomorphized generic layouts;
- compressed discriminants;
- ABI-specific layout derivation.

The language specification permits them.

Implementation plans should schedule them separately from semantic correctness.

---

# 68. Deferred language features

The following remain deferred.

## 68.1 `extension`

Out-of-line behavior attachment.

## 68.2 Separate `class` reopening

Using:

```phalcom
class ExistingEnum<T> {
    ...
}
```

to attach behavior to an existing enum.

## 68.3 Out-of-line variants

For example:

```phalcom
enum Option<T>

@variant Option.Some<T>(_ value: T)
```

## 68.4 Open/extensible enums

Third-party variant contribution.

## 68.5 Common per-variant method contracts

A future feature may allow:

```phalcom
enum Shape {
    area -> Float

    @variant Circle(...) {
        area -> Float { ... }
    }

    @variant Rectangle(...) {
        area -> Float { ... }
    }
}
```

with closed tag dispatch.

This is not part of the initial model.

## 68.6 Full private-case semantics

Constructor privacy is required conceptually.

Completely hidden cases and public exhaustiveness abstraction require additional specification.

## 68.7 Stable layout annotations

`@repr(C)`, transparent representation, explicit discriminants, and similar ABI features require a separate layout/FFI specification.

---

# 69. Decision register

The following decisions are ratified.

1. `enum` is a first-class nominal closed sum declaration.
2. Variants are explicitly introduced with `@variant`.
3. `@variant` introduces a distinct semantic declaration kind.
4. A variant is not a method.
5. Payload-bearing variants induce specialized callable variant constructors.
6. Nullary variants are values rather than requiring `()`.
7. Variant identity is `(owning enum, selector identity)`.
8. Parameter types do not participate in variant identity.
9. Field mutability does not participate in variant identity.
10. Variant parameters support positional and labeled forms.
11. External labels control invocation and selector identity.
12. Internal names control stored/member names.
13. Labeled shorthand uses one name as both external and internal.
14. Positional fields precede labeled fields.
15. Current Phalcom `const` semantics remain immutable-binding/storage semantics.
16. `const` may annotate variant payload fields.
17. The compiler may constant-evaluate enum construction when possible.
18. Variant case identity/discriminant does not mutate in place.
19. Variant constructors perform canonical structural construction.
20. Arbitrary construction logic belongs in smart/factory methods.
21. Private variant constructor capability is required.
22. Constructor visibility and case/pattern visibility are semantically separable.
23. Enums support ordinary instance methods.
24. Enums support `@class` methods.
25. Variants support exact-case methods in the variant body.
26. Exact-case methods are available only after sufficient case proof/refinement.
27. Exact case types exist semantically.
28. Exact case subtype relations do not imply runtime class inheritance.
29. No `extension` feature is included initially.
30. No enum reopening through a separate `class` declaration is included initially.
31. No out-of-line variant declarations are included initially.
32. No implicit lifting of same-named case methods to the enum is performed.
33. Pattern matching is a language-level elimination/proof construct.
34. Exhaustiveness is a general semantic capability.
35. Structural unions remain distinct from nominal enums.
36. A GADT uses ordinary `enum` plus explicit constructor result types.
37. Constructor signatures are fundamental semantic data.
38. GADT matching introduces type equalities.
39. Constructor-local generics may become existential in pattern branches.
40. Impossible GADT cases are excluded from required exhaustiveness coverage.
41. Everything remains an object semantically.
42. Semantic objecthood does not require object-shaped physical representation.
43. Objecthood does not imply observable reference identity.
44. Enums retain associated runtime/type objects.
45. Variant constructors are callable objects semantically.
46. Enum runtime representation is implementation-defined by default.
47. Advanced layout optimizations are permitted but not required.
48. GADT indices are normally erased.
49. Runtime type/trait evidence is materialized only when runtime semantics require it.
50. Dynamic boxing is an erasure-boundary concern, not inherent enum representation.

---

# 70. Recommended implementation order

A practical implementation should proceed in the following order.

## Phase 1 — Syntax and declaration model

- add/solidify `enum`;
- parse `@variant`;
- parse variant parameters;
- parse nullary variants;
- parse optional variant body;
- parse explicit variant result type;
- preserve ordinary enum and `@class` methods.

## Phase 2 — Semantic identity

- publish enum nominal type;
- publish selector-shaped `VariantId`;
- detect duplicate variants;
- publish exact case types;
- publish variant constructor callables;
- publish nullary case values.

## Phase 3 — Construction and ordinary ADTs

- type-check variant calls;
- infer generic arguments;
- type payloads;
- produce exact-case result facts;
- support widening to enum type;
- support member access on enum/case values.

## Phase 4 — Pattern matching

- resolve case patterns;
- bind payloads;
- refine scrutinee exact case;
- exhaustiveness for ordinary enums;
- unreachable-case diagnostics.

## Phase 5 — Variant-specific methods

- exact-case receiver environment;
- payload field/member lookup;
- refined completion/hover;
- diagnostic for unavailable case methods.

## Phase 6 — GADT constructor results

- resolve explicit result type;
- validate owning family;
- instantiate constructor result;
- unify with expected/scrutinee indexed type.

## Phase 7 — GADT pattern evidence

- branch equality evidence;
- existential constructor parameters;
- existential-escape checks;
- indexed exhaustiveness.

## Phase 8 — Diagnostics and explanation graph

- structured constructor evidence;
- refinement traces;
- specialized exhaustiveness diagnostics;
- GADT equality explanations.

## Phase 9 — Runtime representation cleanup

- establish a simple correct representation first;
- benchmark;
- implement high-value optimizations separately.

---

# 71. Acceptance examples

The following examples should eventually pass semantic tests.

## 71.1 Basic construction

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}

const a: Option<Int> = Option.Some(10)
const b: Option<Int> = Option.None
```

## 71.2 Overloaded selector-shaped cases

```phalcom
enum E {
    @variant V(_ x: Int)
    @variant V(_ x: Int, _ y: Int)
    @variant V(_ x: Int, at y: Int)
}
```

All three are distinct.

## 71.3 Duplicate selector rejection

```phalcom
enum E {
    @variant V(_ x: Int)
    @variant V(_ x: String)
}
```

Must fail.

## 71.4 Exact case method refinement

```phalcom
enum Option<T> {
    @variant Some(_ value: T) {
        unwrap -> T { value }
    }

    @variant None
}

const value: Option<Int> = ...

if let Some(_) = value {
    const x: Int = value.unwrap
}
```

Must pass.

Outside the refinement, `value.unwrap` must fail.

## 71.5 External/internal label mapping

```phalcom
enum Change {
    @variant Move(
        _ value: Int,
        from source: Int,
        to destination: Int
    ) {
        delta -> Int {
            destination - source
        }
    }
}

const x = Change.Move(3, from: 5, to: 8)
```

Must bind fields `value`, `source`, and `destination`.

## 71.6 GADT return proof

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}

evaluate<T>(_ expr: Expr<T>) -> T {
    match expr {
        Int(value) => value
        Bool(value) => value
    }
}
```

Must type-check from branch equalities.

## 71.7 Impossible indexed branch

For `Expr<Int>`, a `Bool` constructor is impossible and should not be required for exhaustiveness if no other `Expr<Int>` constructors remain uncovered.

---

# 72. Final semantic statement

Phalcom ADTs are **nominal algebraic objects**.

An `enum` declares a closed nominal sum. Each `@variant` declares an associated exact case whose selector-shaped constructor injects a tuple-shaped payload product into that sum. Variants may define exact-case behavior; the enum may define behavior available across the entire sum and class-side behavior on its associated type object. Pattern matching eliminates the sum, refines receivers to exact case types, and contributes formal proof evidence to the semantic analyzer.

GADTs use the same model, with constructor result types refining the enclosing generic family. Pattern matching on a GADT introduces the equalities and existential evidence implied by the selected constructor.

The runtime representation remains deliberately decoupled from these semantics.

> **Everything is an object semantically; nothing in the ADT/GADT model requires every object to have an object-shaped physical representation.**
