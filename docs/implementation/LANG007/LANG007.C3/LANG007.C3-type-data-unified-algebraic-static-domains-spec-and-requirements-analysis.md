---
id: LANG007.C3.specification-and-requirements-analysis
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: specification-and-requirements-analysis
status: PROPOSED
title: "Type Data and Unified Algebraic Static Domains"
prepared: 2026-09-22
---

# LANG007.C3 — Type Data and Unified Algebraic Static Domains

## Specification and Requirements Analysis

## 0. Status, authority, and intent

This document specifies the proposed language semantics for **LANG007.C3 — Type Data and Unified Algebraic Static Domains** and analyzes the implementation requirements necessary to realize them in Phalcom.

It incorporates the following accepted design direction:

1. `enum` is removed as a distinct source-language algebraic declaration keyword.
2. `data` is the single runtime algebraic declaration form and may describe products, closed sums, ADTs, and GADTs.
3. `type data` declares algebraic domains whose inhabitants exist **only in the type system**.
4. `type data` may be declared at module scope and as an associated declaration inside nominal owners such as classes and data declarations.
5. Associated type-data domains, associated type-level values, variant/case families, and existing type members use the same `::` associated namespace model.
6. Compiler-owned runtime algebraic domains may be declared with `intrinsic data`.
7. Compiler-owned type-system-only domains may be declared with `intrinsic type data`.
8. Existing `TypePack`, `LabeledTypePack`, `RecordRow`, complete-pack, type-member projection, variadic binder, ADT/GADT, and exact-case machinery must be generalized and reused rather than replaced by parallel systems.
9. `*`, `**`, and `***` retain their already-ratified meanings: positional, labeled, and complete lanes respectively. `...` is not pack syntax.

Until accepted and merged into the authoritative specification corpus, this document is a proposed specification. Once accepted, conflicting older statements that distinguish `data` and `enum` as separate semantic declaration categories must be migrated or superseded.

---

# Part I — Normative Language Specification

## 1. Purpose

Phalcom has one algebraic declaration language with two phases:

```text
data
    closed algebraic domains whose inhabitants are runtime values

type data
    closed algebraic domains whose inhabitants are type-system-only terms
```

Both forms support the same core algebraic shapes:

```text
product
    tuple-shaped
    record-shaped

sum
    singleton cases
    tuple-payload cases
    record-payload cases
    GADT-specialized cases
```

The declaration keyword determines **phase**. The declaration shape determines **algebraic structure**.

The model deliberately separates four independent axes:

```text
phase
    Runtime
    TypeOnly

algebraic shape
    Product
    Sum

implementation provenance
    Phalcom
    Native
    Intrinsic
    other implementation kinds defined elsewhere

ownership
    module declaration
    associated declaration owned by another nominal declaration
```

No one axis may be inferred from another.

---

## 2. Core terminology

### 2.1 Algebraic declaration

An **algebraic declaration** introduces one nominal closed domain and defines all representation-bearing alternatives of that domain.

A runtime algebraic declaration is introduced by `data`.

A type-system-only algebraic declaration is introduced by `type data`.

### 2.2 Product

A **product declaration** has exactly one representation shape. Its complete constituent state is its declared components.

Examples:

```phalcom
data Point(
    _ x: Int,
    _ y: Int
)

data Person {
    name: String
    age: Int
}
```

### 2.3 Sum

A **sum declaration** has a closed set of named cases. Each case may be a singleton or may carry product-shaped payload state.

Example:

```phalcom
data Option<T: Type> {
    None
    Some(_ value: T)
}
```

### 2.4 Case

A **case** is one alternative of a sum declaration. Case identity is nominal and owned by its enclosing algebraic declaration.

### 2.5 Static domain

A **static domain** is a domain whose terms may occur in type-system/static contexts and participate in canonical type identity.

Every `type data` declaration introduces a nominal static domain.

Compiler-defined domains such as `TypePack`, `LabeledTypePack`, `TupleEntries`, and `RecordRow` are static domains.

Existing compiler-supported constant domains such as compile-time `Int` remain static domains as separately specified.

### 2.6 Static term

A **static term** is a canonical type-system term classified by a static domain.

Examples include:

```text
Int
Option<String>
3
#name
Shape::Scalar
Shape::Vector(4)
[Int, String]
[name: String]
```

Static terms are not automatically runtime values.

---

## 3. Declaration taxonomy

The canonical declaration taxonomy is:

```text
class
    nominal object-oriented runtime abstraction

data
    closed runtime algebraic value domain

type data
    closed type-system-only algebraic domain

type
    type/static-term member, alias, or computation as specified by its context

trait
    reusable behavioral/type contract
```

`enum` is not a canonical source declaration category.

`kind` is not introduced as a user declaration category by this feature.

### 3.1 `data`

`data` defines runtime values and their exact nominal algebraic type.

```phalcom
data Point(_ x: Int, _ y: Int)

data Option<T: Type> {
    None
    Some(_ value: T)
}
```

Runtime `data` values:

- participate in the ordinary runtime object/value model;
- may cross `Dynamic` boundaries where otherwise valid;
- may have inherent behavior through authorized `impl` declarations;
- may conform to traits;
- preserve exact applied type identity where established;
- remain representation-independent.

### 3.2 `type data`

`type data` defines type-system-only terms.

```phalcom
type data Shape {
    Scalar
    Vector(_ size: Int)
    Matrix(_ rows: Int, _ columns: Int)
}
```

A `type data` declaration:

- introduces no runtime values;
- requires no heap representation;
- cannot be used to create a runtime local value;
- may be used as a static parameter domain;
- has canonical declaration and case identity;
- may participate in type identity, substitution, inference, metadata, and incremental compilation;
- is never normalized using runtime dispatch or user-defined runtime equality.

Example:

```phalcom
data Tensor<Element: Type, S: Shape>

type A = Tensor<Float, Shape::Vector(3)>
```

A runtime binding such as the following is invalid solely by virtue of `Shape` being type-only:

```phalcom
let shape: Shape = Shape::Vector(3)
```

### 3.3 `intrinsic data`

`intrinsic data` is a runtime algebraic declaration whose semantic formation or representation is compiler-owned.

Example conceptual Universe declarations:

```phalcom
intrinsic data Unit()
intrinsic data Tuple<***Es>
intrinsic data Record<R: RecordRow>
```

`intrinsic` does not make the declaration type-only. `data` still determines runtime phase.

### 3.4 `intrinsic type data`

`intrinsic type data` is a type-system-only algebraic/static domain whose formation, normalization, or canonical representation is compiler-owned.

Examples:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries
intrinsic type data RecordRow
```

`intrinsic` is implementation provenance; `type data` is phase and semantic category.

---

## 4. Algebraic declaration grammar and shape

The following grammar is conceptual; the parser may use different internal productions provided it preserves the specified distinctions.

```text
AlgebraicDecl
    ::= DataDecl
     | TypeDataDecl

DataDecl
    ::= Modifiers? "data" Name GenericParams? ProductTail Constraints?
     |  Modifiers? "data" Name GenericParams? SumBody Constraints?

TypeDataDecl
    ::= Modifiers? "type" "data" Name GenericParams? ProductTail Constraints?
     |  Modifiers? "type" "data" Name GenericParams? SumBody Constraints?

ProductTail
    ::= "(" ProductComponents? ")"
     |  RecordProductBody

RecordProductBody
    ::= "{" RecordComponents "}"

SumBody
    ::= "{" SumCases? "}"

SumCase
    ::= Name
     |  Name "(" CaseParameters? ")" GadtResult?
     |  Name "{" RecordCaseComponents? "}" GadtResult?
```

### 4.1 Tuple-shaped product

```phalcom
data Pair<A: Type, B: Type>(
    _ first: A,
    _ second: B
)
```

The declaration has one primary constructor:

```phalcom
Pair(1, "x")
```

The constructor shape follows ordinary Phalcom argument-lane rules.

### 4.2 Record-shaped product

```phalcom
data Person {
    name: String
    age: Int
}
```

Construction is record-shaped:

```phalcom
Person {
    name: "Ada",
    age: 37,
}
```

### 4.3 Singleton sum

```phalcom
data Weekday {
    Monday
    Tuesday
    Wednesday
}
```

Each bare case denotes one canonical singleton case value per exact applicable runtime specialization.

Associated access uses `::`:

```phalcom
Weekday::Monday
```

### 4.4 Singleton shorthand

A comma-separated body is shorthand for bare singleton cases:

```phalcom
data Weekday {
    Monday, Tuesday, Wednesday
}
```

is semantically identical to:

```phalcom
data Weekday {
    Monday
    Tuesday
    Wednesday
}
```

Comma separation does not introduce a distinct enum category.

### 4.5 Tuple-payload sum case

```phalcom
data Option<T: Type> {
    None
    Some(_ value: T)
}
```

Construction:

```phalcom
Option<Int>::Some(42)
Option<Int>::None
```

### 4.6 Record-payload sum case

Record-shaped payloads are permitted:

```phalcom
data Message {
    Request {
        method: Method
        path: String
    }

    Response {
        status: Int
        body: Bytes
    }

    Closed
}
```

Construction is associated and record-shaped:

```phalcom
Message::Request {
    method: .Get,
    path: "/users",
}
```

### 4.7 Zero-argument constructor case

A bare singleton and an explicit zero-argument constructor remain semantically distinct:

```phalcom
data Example {
    Singleton
    Constructed()
}
```

Their associated surfaces differ:

```phalcom
Example::Singleton
Example::Constructed()
```

An implementation must not collapse `Constructed()` into the singleton form merely because both carry no payload.

### 4.8 GADT case

A case may specialize the result of its enclosing generic sum:

```phalcom
data Expr<T: Type> {
    Int(_ value: Int) -> Expr<Int>
    Bool(_ value: Bool) -> Expr<Bool>

    Equal<U: Type>(
        _ left: Expr<U>,
        _ right: Expr<U>
    ) -> Expr<Bool>
}
```

The result annotation:

- must denote an application of the same enclosing algebraic declaration;
- establishes case-local type equalities;
- participates in exact-case applicability;
- is available to pattern/refinement semantics;
- does not add runtime evidence fields solely to represent the equality.

The same declaration form is permitted for `type data` where its parameters and payloads are valid static-domain terms.

### 4.9 Product/sum body discrimination

A braced algebraic declaration body must be unambiguously either a record product or a sum.

A body containing root component declarations of the form:

```phalcom
name: TypeExpression
```

is a record product.

A body containing case declarations is a sum.

Mixing product components and sum cases at the same declaration level is invalid:

```phalcom
data Invalid {
    value: Int
    None
}
```

The parser/checker must issue a dedicated mixed-algebraic-body diagnostic rather than choosing one interpretation by source order.

### 4.10 Empty algebraic forms

The two empty forms have intentionally different algebraic meanings.

```phalcom
data Token()
```

is the zero-component product and has exactly one semantic value per exact applied runtime type.

```phalcom
data Impossible {}
```

is an empty sum and has no inhabitants.

The same algebraic distinction applies to `type data`:

```phalcom
type data StaticUnit()
type data StaticNever {}
```

`StaticUnit()` has one canonical static inhabitant; `StaticNever` has none.

This preserves:

```text
empty product = 1
empty sum     = 0
```

---

## 5. Representation and behavior separation

Algebraic declaration bodies define representation-bearing algebra only. Ordinary behavior is declared through `impl`.

Valid:

```phalcom
data Shape {
    Circle(_ radius: Float)
    Rectangle(_ width: Float, _ height: Float)
}

impl Shape {
    area -> Float
}
```

Case-specific behavior remains exact-case behavior:

```phalcom
impl Shape::Circle(_) {
    area -> Float {
        3.14159 * radius * radius
    }
}
```

Ordinary runtime methods are not representation members and do not belong in the sum/product body.

### 5.1 Runtime data `impl`

Runtime `data` supports the same authorized inherent behavior and trait conformance model currently applicable to data objects and closed sums.

### 5.2 Type-data `impl`

A `type data` declaration may receive **type-level/static behavior** where the general type-member model permits it.

Example:

```phalcom
intrinsic type data TypePack

impl TypePack {
    intrinsic type Map<F: Type -> Type>: TypePack
}
```

A `type data` target does not thereby acquire runtime methods, runtime fields, runtime constructors, or VM dispatch.

Bodyful user-defined arbitrary compile-time methods are not introduced by this specification.

---

## 6. Nominal identity

Every algebraic declaration has declaration identity independent of its shape.

```phalcom
data Position(_ x: Int, _ y: Int)
data Velocity(_ x: Int, _ y: Int)
```

remain different runtime types.

Likewise:

```phalcom
type data Width(_ value: Int)
type data Height(_ value: Int)
```

remain different static domains even when structurally identical.

Structural equality of declarations does not merge nominal declaration identity.

---

## 7. Runtime product semantics

Runtime product data preserve the existing data-object invariants unless explicitly superseded here:

- nominal;
- transparent with respect to semantic components;
- shallowly immutable component bindings;
- allocation-identity-free at the language level;
- non-inheritable;
- representation-closed;
- behavior-extensible only through authorized behavior mechanisms;
- exact applied generic identity is semantically significant.

The primary constructor belongs to the declaration and is not implicitly an ordinary method named `new`.

Product component order is logical declaration order. It is distinct from source expression evaluation order and physical storage order.

---

## 8. Runtime sum semantics

A sum-shaped runtime `data` declaration is a nominal closed sum.

### 8.1 Closedness

The case set belongs exclusively to the declaration.

No `impl`, import, runtime action, reflection operation, or later declaration may add another case.

### 8.2 Case identity

Case identity is nominal and declaration-owned.

```phalcom
data Token {
    Identifier(_ text: String)
    Keyword(_ text: String)
}
```

`Token::Identifier(_)` and `Token::Keyword(_)` remain distinct despite identical payload shape.

### 8.3 Exact-case types

Runtime sum cases induce exact-case semantic types/refinements.

Conceptually:

```text
Token
    root exact-applied sum type

ExactCase(Token::Identifier(_), Token)
    exact case
```

Exact-case information may drive:

- payload availability;
- case-specific inherent behavior;
- GADT equality evidence;
- pattern refinement;
- exact-case conformance where separately valid;
- closed behavior requirements/defaults.

The existing exact-case semantic model is preserved under the new `data` surface.

### 8.4 Root behavior contracts

Closed root requirements/defaults previously owned by enum-root semantics become **closed sum-data contracts**.

A root requirement applies to every applicable exact case unless a root default or other accepted rule satisfies it.

Behavior completeness and match exhaustiveness remain separate analyses.

---

## 9. Type-data semantics

A `type data` declaration introduces a nominal static domain and a closed algebra of canonical static terms.

### 9.1 No runtime materialization

No `type data` term is a runtime program value merely because its syntax resembles runtime data construction.

The following are static terms:

```phalcom
Shape::Scalar
Shape::Vector(3)
```

They may occur in:

- generic/static argument positions;
- type-member arguments where their domain is expected;
- static/type aliases with the corresponding result domain;
- other canonical static expressions accepted by the type system.

They do not imply:

- runtime allocation;
- runtime class identity;
- object identity;
- VM method dispatch;
- a runtime `Shape` value.

### 9.2 Type-data product terms

A product-shaped `type data` declaration has one static constructor corresponding to its product shape.

```phalcom
type data Coordinate(
    _ x: Int,
    _ y: Int
)
```

A canonical term may be written:

```phalcom
Coordinate(3, 4)
```

only in a static/type-system context.

Record-shaped form is analogous:

```phalcom
type data Field {
    name: Symbol
    ty: Type
}
```

with static formation using the declaration's record construction syntax.

### 9.3 Type-data sum terms

A singleton case:

```phalcom
type data Endian {
    Little
    Big
}
```

forms canonical static terms:

```phalcom
Endian::Little
Endian::Big
```

Payload cases form canonical static applications:

```phalcom
type data Shape {
    Scalar
    Vector(_ size: Int)
}

Shape::Vector(4)
```

### 9.4 Static structural equality

Type-data definitional identity is compiler-controlled.

For a constructor term:

```text
C<A1, ..., An>
```

canonical identity includes:

```text
owning type-data declaration identity
case/constructor identity
canonical identities of all static arguments
applicable enclosing specialization/capture environment
```

Thus:

```text
Shape::Vector(3) ≡ Shape::Vector(3)
Shape::Vector(3) ≠ Shape::Vector(4)
Shape::Vector(3) ≠ OtherShape::Vector(3)
```

No user-defined `==`, `hash`, runtime method, object address, or runtime representation participates in this equality.

### 9.5 Static argument validity

Every payload of a `type data` term must itself be valid in the static language.

A declaration requiring a payload domain with no canonical static-term semantics is invalid.

Conceptually invalid:

```phalcom
type data Invalid {
    Open(_ file: File)
}
```

when `File` has no static-domain semantics.

This failure belongs to the declaration or payload contract, not to arbitrary later runtime use.

### 9.6 Type-data GADTs

A generic `type data` declaration may use case result specialization:

```phalcom
type data StaticExpr<T: Type> {
    Int(_ value: Int) -> StaticExpr<Int>
    Bool(_ value: Bool) -> StaticExpr<Bool>
}
```

Case equations participate in static identity, applicability, and inference.

This specification does not require arbitrary user-defined static pattern evaluation over such terms.

---

## 10. Static domains and generic parameters

### 10.1 General rule

A generic/static binder:

```phalcom
X: D
```

means:

> `X` ranges over canonical static terms belonging to static domain `D`.

Examples:

```phalcom
T: Type
N: Int
S: Shape
P: TypePack
L: LabeledTypePack
E: TupleEntries
R: RecordRow
```

The colon classifies the binder's static domain. It does not imply trait conformance.

### 10.2 User-defined static domains

Every `type data D` declaration makes `D` usable as a static parameter domain.

```phalcom
type data Endian {
    Little
    Big
}

data Integer<
    Bits: Int,
    Order: Endian
>
```

is valid.

### 10.3 Runtime `data` is not automatically a static domain

Declaring:

```phalcom
data Point(_ x: Int, _ y: Int)
```

does not make arbitrary `Point` runtime values valid static arguments.

`type data` is the explicit declaration mechanism for nominal algebraic domains intended to exist in the type system.

Compiler-provided primitive static domains such as compile-time `Int` are separately authorized language primitives.

### 10.4 Binder mode remains orthogonal

Variadic binder mode remains a separate property from the static domain:

```text
Single
PositionalPack
LabeledPack
CompletePack
```

Conceptually:

```phalcom
P: TypePack
```

is one ordinary static parameter whose domain is `TypePack`.

```phalcom
*Ts
```

is a positional-pack binder whose source binding mode is variadic and whose bound term belongs to `TypePack`.

The same domain does not imply the same binder mode.

---

## 11. Heterogeneous static generic arguments

Generic applications must be capable of carrying canonical static terms from the parameter's declared domain.

Examples:

```phalcom
Tensor<Float, Shape::Vector(3)>
Integer<32, Endian::Big>
```

A nominal generic application's arguments therefore cannot semantically be restricted to proper types.

A canonical generic argument is conceptually:

```text
StaticArgument
    TypeForm
    Const
    TypeDataTerm
    TypePack
    LabeledTypePack
    TupleEntries
    RecordRow
    other accepted static-domain term
```

An implementation may optimize these categories internally but must preserve one semantic application model.

---

## 12. Static term aliases and type members

### 12.1 `type` produces a static result

The general type-member model already permits results outside `Type`. This enhancement makes the surface model explicit:

```phalcom
type First: Type
type Positionals: TypePack
type Entries: TupleEntries
type Initial: State
```

A `type` declaration computes or denotes one static term in its declared result domain.

### 12.2 Explicit-domain definition

Where static aliases/definitions are permitted, the canonical explicit form is:

```phalcom
type Name: Domain = StaticExpression
```

Examples:

```phalcom
type DefaultShape: Shape = Shape::Scalar
type P: TypePack = T::Positionals
```

Existing unannotated type aliases continue to mean proper-type aliases where the existing grammar specifies that default.

### 12.3 Intrinsic type member

A declaration such as:

```phalcom
intrinsic type Positionals: TypePack
```

means:

- `Positionals` is a type-level/static member;
- its result belongs to `TypePack`;
- its implementation is supplied by canonical compiler normalization;
- it is not a runtime method or property.

---

## 13. Associated `type data`

### 13.1 Declaration inside an owner

A nominal owner may declare an associated type-data domain:

```phalcom
class Parser {
    type data State {
        Start
        InString
        Done
    }
}
```

The declaration identity is:

```text
Parser::State
```

Its cases are associated members of that domain:

```phalcom
Parser::State::Start
Parser::State::Done
```

### 13.2 Same associated namespace

Associated type-data declarations and associated static terms participate in the same associated namespace model used by type members and case families.

Example:

```phalcom
class C {
    type data State {
        Empty
        Ready
    }

    type Initial: State
}
```

Conceptually:

```text
C::State
    associated static-domain declaration

C::State::Empty
    static case term

C::Initial
    associated static term of domain C::State
```

They are not separate dot namespaces.

### 13.3 Specialized receiver lookup

Members declared on a generic owner are available through exact/specialized receiver types.

For a member declaration owned by `Tuple<***Es>`:

```text
declaration identity
    Tuple::First

projection
    (Int, String)::First
```

The projection resolves the declaration through the specialized receiver environment.

The tuple type does not own a duplicate `First`; it exposes the member declared by `Tuple`.

### 13.4 Associated domain specialization

An associated `type data` declaration inside a generic owner forms an associated domain family over the owner's specialization.

Conceptually:

```phalcom
class Container<T: Type> {
    type data State {
        Empty
        Present(_ value: T)
    }
}
```

yields concrete associated domains such as:

```text
Container<Int>::State
Container<String>::State
```

The declaration identity remains:

```text
Container::State
```

while the concrete associated-domain identity carries the applicable owner specialization/environment.

An unspecialized generic-owner path may identify the associated declaration/family but is not a concrete static domain where unresolved owner parameters are required.

### 13.5 Namespace conflicts

One associated base name may not ambiguously denote incompatible declaration categories for the same owner unless a separate selector rule explicitly permits their coexistence.

The checker must diagnose collisions among:

- associated type-data domains;
- associated type/static members;
- case/variant families;
- associated callable families;
- other declaration-owned associated bases.

Name resolution must never choose by source order.

### 13.6 Trait-associated domain requirements

This checkpoint does **not** require a trait requirement form in which a conformance supplies an entire `type data` domain:

```phalcom
trait StateMachine {
    type data State
}
```

Such associated-domain requirements are a coherent future extension but materially expand conformance binding, evidence, metadata, and coherence semantics. They require a separate accepted specification or explicit C3 amendment.

Ordinary trait-associated `type` requirements remain governed by the existing associated-type specification.

---

## 14. Unified `::` lookup

`::` is the associated/static lookup operator.

The following all use the same semantic lookup framework:

```phalcom
Option::Some
Option::None

Parser::State
Parser::State::Done

(Int, String)::Positionals
(Int, String)::At<0>
```

Lookup conceptually performs:

```text
resolve subject/owner
    ↓
resolve associated declaration identity
    ↓
establish owner specialization/evidence/domain context
    ↓
check member arguments
    ↓
produce static term, associated domain, case constructor, or other associated entity
    ↓
normalize when canonical information is sufficient
```

`::` is not runtime message dispatch.

`.` remains ordinary runtime member/message syntax.

---

## 15. Compiler-intrinsic structural static domains

The standard Universe should be able to expose compiler-owned structural domains using `intrinsic type data`.

The canonical public model is:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries
intrinsic type data RecordRow
```

### 15.1 `TypePack`

`TypePack` is an ordered sequence of proper type forms.

Conceptually:

```text
[Int, String] : TypePack
```

Order participates in identity.

### 15.2 `LabeledTypePack`

`LabeledTypePack` is an ordered sequence of unique label/type entries.

Conceptually:

```text
[name: String, age: Int] : LabeledTypePack
```

Order participates in identity.

`LabeledTypePack` is not `RecordRow`.

### 15.3 `RecordRow`

`RecordRow` is a structural row domain whose identity is key/type association, not labeled-pack sequence order. Open row semantics remain as separately specified.

### 15.4 `TupleEntries`

`TupleEntries` is the public static domain of a complete two-lane tuple entry shape:

```text
TupleEntries(
    positionals: TypePack,
    labeled: LabeledTypePack
)
```

It is not a proper Tuple type.

The existing internal canonical `CompleteTypePack` representation may realize this domain.

### 15.5 Static-domain operations

Compiler-owned static-domain operations may be declared using ordinary type-member syntax:

```phalcom
impl TypePack {
    intrinsic type Map<F: Type -> Type>: TypePack
}

impl LabeledTypePack {
    intrinsic type Map<F: Type -> Type>: LabeledTypePack
}

impl TupleEntries {
    intrinsic type Positionals: TypePack
    intrinsic type Labeled: LabeledTypePack
    intrinsic type Map<F: Type -> Type>: TupleEntries
}

impl RecordRow {
    intrinsic type Map<F: Type -> Type>: RecordRow
}
```

These are compiler normalization operations, not runtime methods.

---

## 16. Structural runtime domains

The Universe may expose compiler-owned runtime structural products as `intrinsic data`.

### 16.1 `Unit`

Conceptually:

```phalcom
intrinsic data Unit()
```

`Unit` is the zero-component runtime product.

The canonical zero Tuple product remains:

```text
() ≡ Unit
```

### 16.2 `Tuple`

Conceptually:

```phalcom
intrinsic data Tuple<***Es>
```

where `***Es` is the complete two-lane static shape associated with the exact Tuple specialization.

Its type-level surface may include:

```phalcom
impl Tuple<***Es> {
    intrinsic type Entries: TupleEntries
    intrinsic type Positionals: TypePack
    intrinsic type Labeled: LabeledTypePack
    intrinsic type At<I: Int>: Type
    intrinsic type Map<F: Type -> Type>: Type
}
```

For:

```phalcom
type T = (Int, String, name: Symbol)
```

the canonical identities are:

```text
T::Positionals : TypePack
T::Labeled     : LabeledTypePack
T::Entries     : TupleEntries

*T   ≡ T::Positionals
**T  ≡ T::Labeled
***T ≡ T::Entries
```

The existing C2 complete-pack and projection machinery remains the normalization authority.

### 16.3 `Record`

Conceptually:

```phalcom
intrinsic data Record<R: RecordRow>
```

with:

```phalcom
impl Record<R> {
    intrinsic type Row: RecordRow
    intrinsic type Map<F: Type -> Type>: Type
}
```

Record and labeled-Tuple shape remain distinct.

### 16.4 Representation freedom

Declaring Tuple/Record/Unit as `intrinsic data` does not require the VM to replace specialized tuple/record/immediate representations with general data-object storage.

Source semantics and runtime physical representation remain separate.

---

## 17. Runtime standard algebraic data

Ordinary standard-library closed sums should use `data` unless their semantic formation itself is compiler-intrinsic.

Examples:

```phalcom
data Option<T: Type> {
    None
    Some(_ value: T)
}

data Result<T: Type, E: Type> {
    Ok(_ value: T)
    Error(_ error: E)
}

data Ordering {
    Less
    Equal
    Greater
}
```

Representation specialization, niche encoding, immediate values, and native lowering do not by themselves require `intrinsic data`.

`intrinsic` indicates semantic/compiler ownership, not merely optimization.

---

## 18. Construction and case families

### 18.1 Product primary constructor

A product declaration has one primary constructor corresponding to its complete product shape.

It is not an ordinary method.

### 18.2 Sum case constructor

A payload case has a declaration-owned associated constructor family.

```phalcom
Option::Some(42)
```

Case constructors are not ordinary associated methods.

### 18.3 Singleton case

A bare case is an associated singleton value:

```phalcom
Option::None
```

### 18.4 Callable references

Existing callable-reference semantics for runtime constructors/case families remain applicable where already defined.

`type data` constructor terms are not automatically runtime callable values. First-class higher-order static constructor families require separate static-function semantics if not already expressible.

---

## 19. Visibility and access

### 19.1 Product constructor visibility

Existing product primary-constructor visibility rules continue to apply to runtime `data`.

Example:

```phalcom
data Date private(
    year: Int,
    month: Month,
    day: Int
)
```

Constructor visibility does not alter declaration identity or component semantics.

### 19.2 Runtime sum case visibility

Existing semantic distinctions for sum cases remain applicable:

```text
case name/match visibility
construction visibility
payload visibility
```

Removing `enum` does not collapse these into one bit.

### 19.3 Type-data visibility

`type data` declarations and cases use static declaration/member accessibility.

There is no runtime-construction capability to protect.

Where a type-data case is inaccessible, it cannot be named or formed outside its authorized static scope. Visibility does not change canonical identity.

Runtime-only constructor visibility modifiers are invalid on `type data` when they have no static meaning.

---

## 20. Pattern matching and elimination

### 20.1 Runtime sum matching

Pattern matching over sum-shaped runtime `data` uses the existing closed-sum semantics:

- exact case discrimination;
- payload binding;
- exhaustiveness;
- redundancy/unreachable-case analysis;
- GADT branch refinement.

The semantic trigger becomes “closed sum-shaped `data`,” not the obsolete `enum` keyword.

### 20.2 Runtime product destructuring

Product `data` continues to use product/destructuring patterns as separately specified.

### 20.3 Type-data matching

`type data` terms do not participate in runtime `match`.

Arbitrary user-defined type-level pattern matching/evaluation is not required by this checkpoint.

A future static-function facility may eliminate type-data terms using the same algebraic case structure, subject to explicit termination/normalization rules.

---

## 21. Inference and unification

### 21.1 Canonical constructor unification

Static inference may structurally unify type-data constructor terms.

Conceptually:

```text
Shape::Vector(N)
=
Shape::Vector(3)
```

may establish:

```text
N = 3
```

when `N` is an inference variable of the appropriate static domain.

Different constructors are disjoint:

```text
Shape::Scalar
≠
Shape::Vector(N)
```

### 21.2 No user-defined equality hooks

Inference and definitional equality must not invoke:

- `==`;
- `hash`;
- ordinary methods;
- runtime reflection;
- runtime object identity.

### 21.3 Symbolic validity

A well-formed static term containing unresolved parameters remains symbolic.

It must not be approximated as `Dynamic`, `Never`, or an unrelated type merely because it is not yet reducible.

### 21.4 Substitution

Substitution must recursively preserve and re-canonicalize:

- type-data terms;
- associated-domain specializations;
- generic applications with heterogeneous static arguments;
- packs/rows/complete tuple entries;
- existing symbolic type-member projections.

Substitution that makes a formerly symbolic term invalid must return an invalidity/error through the checked substitution path rather than retaining a malformed canonical term.

---

## 22. Canonical identity and interning

Every durable static term must have canonical semantic identity independent of one inference run.

The following identities must be declaration-based, not spelling-based:

```text
TypeDataDeclarationId
TypeDataCaseId
AssociatedTypeDataDeclarationId
specialized AssociatedTypeData identity
static constructor application
```

Canonical identity must survive:

- substitution;
- metadata export/import;
- incremental recomputation;
- cross-module use;
- cold versus incremental checking;
- aliasing.

Query-local inference IDs may not leak into durable metadata or interface fingerprints.

---

## 23. Metadata and module boundaries

A module interface that exposes any of the following must serialize stable semantic records rather than source strings:

- `type data` declarations;
- their generic parameters and static-domain parameter contracts;
- case declarations and payload domains;
- GADT result specialization;
- associated type-data declarations;
- static aliases with non-`Type` result domains;
- generic parameters whose domains are user-defined `type data`;
- generic applications containing type-data terms;
- `TupleEntries` and complete-pack terms;
- associated-domain specializations where required.

Schema validation must reject malformed combinations, including:

- runtime-only payload domains in type-data terms;
- unknown case IDs;
- constructor arity/label mismatches;
- result specialization outside the enclosing declaration;
- invalid associated owner environment;
- inconsistent generic parameter domain/mode combinations.

---

## 24. Incremental semantics

Incremental invalidation must account for semantic changes to:

- algebraic phase (`data` versus `type data`);
- product versus sum shape;
- component/case set;
- case payload shapes;
- case-local generics;
- GADT result specialization;
- associated type-data ownership;
- static-domain parameter contracts;
- type-data case use in public generic applications;
- intrinsic static-domain declarations;
- associated namespace collisions.

Cold and incremental final semantic state must converge.

Source-order changes that do not affect semantic identity must not produce accidental type identity changes unless source order is explicitly semantic for that declaration category.

---

## 25. Reflection and reification boundary

### 25.1 Runtime `data`

Runtime reflection may expose:

- algebraic declaration identity;
- exact applied runtime type;
- product versus sum shape;
- product components;
- sum cases;
- exact active case;
- case payload components;
- behavior/impl provenance as specified elsewhere.

### 25.2 `type data`

`type data` declarations are static semantic entities.

This feature does not require runtime reification of every type-data domain or term.

Compiler/LSP/static reflection may expose them.

A future runtime type-reflection system may choose to reify descriptors, but such descriptors are representations of static semantics, not runtime inhabitants of the type-data domain.

---

## 26. Diagnostics

At minimum, implementations should distinguish the following failures.

### Declaration-shape diagnostics

- mixed product components and sum cases in one braced body;
- duplicate product component;
- duplicate case identity/selector;
- malformed record case;
- invalid GADT result target;
- invalid empty-form interpretation where grammar is violated.

### Type-data diagnostics

- runtime use of a type-data domain or term;
- non-static payload domain in `type data`;
- invalid static constructor argument;
- constructor arity/label mismatch;
- inaccessible type-data declaration/case;
- invalid associated type-data owner specialization;
- unspecialized associated domain used where a concrete domain is required.

### Generic/static-domain diagnostics

- static argument domain mismatch;
- runtime `data` value supplied where a static-domain term is required;
- unsupported heterogeneous generic application;
- pack binder mode/domain mismatch;
- invalid static substitution after normalization.

### Namespace diagnostics

- associated base collision;
- ambiguous `::` lookup;
- runtime `.` used where associated/static lookup is required;
- associated/static lookup attempted through insufficient subject information.

### Migration diagnostics

After compatibility removal:

- `enum` declaration keyword is obsolete/invalid;
- legacy enum-only syntax is invalid where no compatibility mode is active.

---

## 27. Invalid programs

A program is invalid when, among other ordinary failures:

- a `data` braced body mixes root fields and cases;
- a `type data` declaration contains a payload that cannot form a canonical static term;
- a `type data` term is required to materialize as an ordinary runtime value;
- a static term uses a runtime-dependent expression;
- two cases conflict in the enclosing associated family;
- a GADT case result is not an allowed specialization of its enclosing declaration;
- an associated type-data domain is used without a required owner specialization;
- a generic static argument does not inhabit its parameter's declared static domain;
- user-defined runtime equality is treated as static definitional equality;
- an `impl` attempts to add product components or cases;
- an `impl` attempts to add runtime behavior to a type-only domain where the member category is not static/type-level;
- an external declaration attempts to extend a closed case set;
- an inaccessible case is named or formed;
- query-local inference identity is serialized as durable static identity.

---

## 28. Non-goals

LANG007.C3 does not, merely by introducing `type data`, require:

- arbitrary compile-time execution;
- Zig-style unrestricted `comptime`;
- a macro system;
- a theorem prover;
- user-defined primitive kinds;
- user-defined equality hooks in the type checker;
- runtime objects representing every type-data term;
- VM type-pack solving;
- arbitrary pack decomposition/backtracking beyond accepted C2 semantics;
- runtime ArgumentPack ABI redesign;
- stable numeric case discriminants or binary ABI;
- automatic promotion of ordinary runtime `data` values into static terms;
- trait requirements that bind whole associated `type data` domains unless separately accepted;
- first-class higher-order static constructor values;
- general user-defined static pattern evaluation;
- universe-polymorphism syntax exposed to ordinary users.

---

## 29. Canonical invariants

| ID | Requirement |
|---|---|
| **ALG-ONE-DATA** | `data` is the single canonical runtime algebraic declaration keyword. |
| **ALG-TYPE-DATA** | `type data` introduces type-system-only algebraic domains with no runtime inhabitants. |
| **ALG-SHAPE** | Product versus sum is determined by declaration shape, not by separate `data`/`enum` keywords. |
| **ALG-CLOSED** | Sum case sets are declaration-owned and closed. |
| **ALG-REP-BEHAVIOR** | Algebraic declarations own representation-bearing components/cases; `impl` contributes behavior only. |
| **ALG-NOMINAL** | Declaration identity remains nominal even when algebraic structure is equal. |
| **ALG-CASE-ID** | Case identity is declaration-owned and independent of physical tags/layout. |
| **ALG-EXACT-CASE** | Runtime sum cases preserve exact-case typing and GADT refinement. |
| **STATIC-CANONICAL** | Type-data terms have compiler-defined canonical structural identity. |
| **STATIC-NO-RUNTIME-EQ** | Runtime `==`, hashing, dispatch, and object identity never define static equality. |
| **STATIC-DOMAIN** | `X: D` classifies `X` in static domain `D`; user-defined `type data` domains may appear there. |
| **STATIC-EXPLICIT** | Ordinary runtime `data` does not automatically become a static-domain declaration. |
| **STATIC-SYMBOLIC** | Well-formed unresolved static terms remain symbolic rather than degrading to unrelated runtime/type approximations. |
| **ASSOC-ONE-NS** | Associated type-data domains and associated static members use the canonical `::` associated namespace. |
| **ASSOC-SPECIALIZE** | Specialized receiver lookup reuses declaration-owned member identity plus the receiver environment; it does not clone declarations per type. |
| **PACK-DOMAINS** | `TypePack`, `LabeledTypePack`, `TupleEntries`, and `RecordRow` remain distinct static domains. |
| **PACK-LANES** | `*`, `**`, and `***` retain positional, labeled, and complete meanings. |
| **PACK-TUPLE-DISTINCT** | `TupleEntries`/complete-pack is not a proper Tuple type. |
| **META-STABLE** | Static declaration/term identity survives metadata, imports, inference, and incremental recomputation. |
| **VM-BOUNDARY** | Type-data/static-domain normalization remains compiler semantic work; the VM does not become a general type solver. |

---

# Part II — Requirements Analysis

## 30. Current repository model

The live repository already contains several mature but currently separate systems that C3 must reconcile.

### 30.1 Current runtime product `data`

`docs/specs/objects/data-objects.md` currently defines `data` as a nominal transparent immutable runtime product and explicitly states that a data declaration is **not** a one-case enum.

The current implementation has dedicated data structures and lowering paths including:

```text
phalcom_ast::ast::DataDef
DataDeclarationProduct
DataInfo / DataShape
DataConstructorId
data declaration checker
data runtime/lowering registry
```

These are valuable product semantics but the “not an enum” language conflicts with C3 and must be superseded.

### 30.2 Current `enum`

`docs/specs/objects/enums.md` currently owns:

- closed nominal sums;
- variants;
- singleton versus zero-argument constructor distinction;
- payload cases;
- exact-case types;
- GADT result specialization;
- root behavior requirements/defaults;
- case behavior;
- match/exhaustiveness semantics;
- variant visibility;
- representation independence.

The implementation has:

```text
phalcom_ast::ast::EnumDef
VariantDecl / VariantId
EnumDeclarationProduct
exact-case semantics
GADT proof machinery
enum/sum exhaustiveness
enum lowering/runtime case machinery
```

C3 should preserve these semantic products while moving them under the unified `data` declaration model.

### 30.3 Current type-level domains

`docs/specs/type-level-domains.md` currently defines a closed kind/domain grammar centered on:

```text
Type
RecordRow
TypePack
LabeledTypePack
kind arrows
Const(Int)
```

Generic parameter classification is currently conceptually:

```text
Kind(κ) | Const(τ)
```

C3 requires nominal user-defined static domains and therefore cannot encode every `type data` declaration as another primitive `KindData` enum variant.

### 30.4 Current type members

`docs/specs/type-members.md` already establishes the crucial general rule that a type member's result may belong to any type-level domain, not only `Type`.

Existing examples include:

```phalcom
intrinsic type Positionals: TypePack
intrinsic type Labeled: LabeledTypePack
intrinsic type At<I: Int>: Type
intrinsic type Map<F: Type -> Type>: Type
```

C3 should extend this substrate to user-defined type-data domains rather than create a second static-member system.

### 30.5 Current associated types

`docs/specs/associated-types.md` currently defines associated types as trait-owned requirements supplied by conformance evidence.

That model must remain valid.

C3's **associated `type data` inside classes/data owners** is a different semantic category: an inherent/nested associated static-domain declaration, not a trait-associated binding requirement.

The specifications must use terminology carefully so “associated type data” does not accidentally weaken trait-owned associated type semantics.

### 30.6 Current `impl`

`docs/specs/objects/impl.md` currently permits inherent targets for data objects, enum roots, exact enum cases, classes, and covering generic applications.

It explicitly separates representation from behavior.

C3 should generalize:

```text
data object type
enum root
exact enum case
```

to:

```text
runtime data product
runtime data sum root
runtime data exact case
```

while adding a narrow static/type-level `impl` surface for type-data domains only where type-member declarations are valid.

### 30.7 Current C2 pack substrate

LANG007.C2 has already established or is completing:

```text
KindData::CompleteTypePack
KindId::COMPLETE_TYPE_PACK
TypeData::CompleteTypePack
TypePackConcat
LabeledTypePackConcat
TupleFromPacks
CallableFromPacks
GenericParameterMode::{
    Single,
    PositionalPack,
    LabeledPack,
    CompletePack
}
checked substitution/materialization
stable metadata nodes
```

C3 must reuse these.

The public `TupleEntries` domain should map to the canonical complete-pack substrate rather than reimplement it.

---

## 31. Required semantic architecture

The implementation should converge on one algebraic declaration model with orthogonal axes rather than proliferating declaration kinds.

Conceptually:

```text
AlgebraicDeclaration
    identity
    phase
    implementation provenance
    generic/static parameters
    algebraic shape
    visibility/access
    constraints
```

where:

```text
phase
    Runtime
    TypeOnly

shape
    Product(ProductShape)
    Sum(CaseSet)
```

The exact Rust names are not normative.

### 31.1 Do not collapse runtime and static term representations unnecessarily

Source-language unification does not require one physical representation for:

- runtime data objects;
- runtime exact cases;
- type-data constructor terms;
- TypePack;
- RecordRow;
- TupleEntries.

The correct unification is semantic identity and declaration architecture, not forced storage convergence.

### 31.2 Preserve optimized runtime paths

Existing optimized runtime categories such as:

```text
DataObject
ADT/variant case storage
TupleObject
RecordObject
immediate singleton forms
```

may remain.

They are lowering/representation choices beneath the unified language model.

### 31.3 Generalize declaration identity before lowering

Product/sum/runtime/type-only distinctions must be canonical semantic declaration facts before compiler lowering.

The VM must not infer algebraic semantics from runtime object shape.

---

## 32. Parser and AST requirements

### 32.1 Remove canonical `enum` declaration grammar

The parser must normalize all canonical closed sums through `data`.

A temporary compatibility parser may accept `enum` during migration, but it must lower immediately to the same algebraic declaration AST and may not survive as a parallel semantic category.

Final canonical source rejects `enum`.

### 32.2 Add `type data`

The lexer/parser must recognize the two-keyword declaration prefix:

```text
type data
```

as one declaration form rather than parsing it as an ordinary `type` alias followed by a stray `data`.

### 32.3 Unified algebraic AST

The current `DataDef` and `EnumDef` split should converge behind one canonical algebraic declaration AST or one common semantic input abstraction.

A suitable conceptual form is:

```rust
struct AlgebraicDef {
    name: String,
    phase: AlgebraicPhase,
    implementation: ImplementationKind,
    generic_parameters: Vec<GenericParameterSyntax>,
    shape: AlgebraicShape,
    constraints: ...,
    ...
}

enum AlgebraicShape {
    Product(ProductShape),
    Sum(Vec<CaseDecl>),
}
```

The exact type names are flexible.

The key requirement is that semantic consumers do not need to rediscover whether a declaration is runtime product, runtime sum, or type-only algebra from unrelated AST variants.

### 32.4 Record body discrimination

The parser must preserve enough syntax to distinguish:

```phalcom
data Person {
    name: String
}
```

from:

```phalcom
data State {
    Ready
}
```

without depending on semantic name lookup.

Mixed bodies must retain enough range information for precise diagnostics.

### 32.5 Associated `type data`

Class/data owner bodies must admit `type data` declarations in the static/associated declaration namespace.

AST identity must preserve:

- owning declaration;
- member name;
- generic parameters if any;
- algebraic shape;
- source range.

---

## 33. Semantic declaration product requirements

The semantic DB currently has separate data and enum query products.

C3 should introduce one authoritative algebraic declaration product or an equivalent facade that owns common facts:

```text
declaration identity
phase
generic/static parameter contract
product/sum shape
components/cases
case identities
GADT equations
constructor/case visibility
associated namespace publication
```

Specialized runtime products may still derive from this common semantic authority.

There must not be two independently computed sources of truth for the same case or component set.

---

## 34. Static-domain representation requirements

### 34.1 Nominal static domain identity

Each `type data` declaration needs a stable domain identity.

A domain identity must be usable by:

- binder checking;
- generic application;
- static constructor formation;
- substitution;
- inference;
- metadata;
- diagnostics;
- source navigation;
- associated lookup.

### 34.2 Static term carrier

The current system uses `TypeId` for canonical kinded type-level forms and separate const handling in some paths.

C3 requires a general semantic representation capable of carrying heterogeneous static arguments without weakening existing optimized `TypeId` semantics.

Two architectures are acceptable:

**Option A — generalized static-term carrier**

```text
StaticTermId
    wraps/references:
        TypeId
        ConstTerm
        TypeDataTerm
        Row term
        ...
```

**Option B — extend canonical type-level term storage**

Broaden the existing canonical store so a stable term ID can classify terms by an arbitrary `StaticDomainId`, while preserving `ProperTypeId` or equivalent proof wrappers for actual `Type` terms.

The architecture must satisfy:

```text
proper type
    !=
arbitrary static term
```

even if one arena or ID family stores both.

### 34.3 Preserve `ProperTypeId`

Any existing proof that a term has proper-type domain `Type` must remain meaningful.

Generalizing static terms must not allow a `Shape::Vector(3)` term to be passed accidentally to APIs that require a proper runtime type.

---

## 35. Generic application requirements

### 35.1 Heterogeneous argument storage

Nominal applied types, aliases, and other generic entities that accept user-defined static domains must store arguments by their declared static domain rather than assuming every argument is a proper type.

This is a deliberate expansion beyond C1's initially bounded heterogeneous type-member arguments.

### 35.2 Trait/conformance migration boundary

If trait generic arguments currently assume only proper types, C3 must either:

1. migrate them to the generalized static argument representation where source syntax permits static-domain parameters; or
2. explicitly reject user-defined static-domain parameters on traits until that consumer is migrated.

The implementation must not silently encode type-data terms as fake `TypeId::Type` values.

### 35.3 Binder contracts

Generic binder headers/signatures must publish static domain identity separately from variadic binder mode.

---

## 36. Inference requirements

Inference must become domain-directed.

An inference variable has:

```text
static domain
lifetime/scope
solution state
```

Pack variables should continue using the C2 kind/domain-aware inference path rather than introducing a duplicate pack-variable system.

Type-data constructor unification should be structural and first-order for the initial implementation.

No backtracking theorem prover is required.

---

## 37. Associated lookup requirements

The existing type-member projection model should be generalized so associated-domain declarations are discoverable through the same declaration-owned lookup infrastructure.

A projection/lookup product must preserve:

```text
subject/owner
member declaration identity
owner specialization
member arguments
applicable evidence where relevant
result category
```

Associated type-data declarations are not trait evidence projections unless they are actually trait-owned requirements under a future extension.

---

## 38. Exact-case and GADT requirements

The existing enum-specific names in implementation code may remain temporarily, but semantic ownership must migrate to sum-shaped `data`.

Required preservation:

- exact case identity;
- case applicability;
- GADT equality proof introduction;
- payload projections;
- exact-case impl targets;
- root default/witness behavior;
- conformance on exact cases where already supported;
- exhaustiveness and redundancy checking;
- variant constructor/family semantics.

No C3 design is acceptable if replacing `enum` causes exact-case precision to be widened away.

---

## 39. Runtime/compiler boundary

`type data` requires **zero runtime value machinery** for its inhabitants.

The runtime should not gain:

- `TypeDataObject`;
- VM opcodes to compare static constructor terms;
- dynamic lookup of type-data cases;
- runtime inference for static domains.

Compiler/export/reflection descriptors may exist where needed.

---

## 40. Metadata requirements

The stable metadata schema must represent:

```text
AlgebraicPhase
AlgebraicShape
TypeDataDeclaration
TypeDataCase
TypeDataTerm
AssociatedTypeDataDeclaration
StaticDomainRef
StaticArgument
```

or exact equivalents.

Metadata must retain declaration IDs rather than names as semantic identity.

Schema versioning must be explicit because C3 changes public interface identity.

Older metadata containing enums may be decoded into the unified runtime-sum algebraic form during a bounded compatibility window if required, but new metadata must not re-export a distinct canonical enum category.

---

## 41. Source indexing and LSP requirements

Source indexing must expose:

- algebraic declaration;
- product components;
- sum cases;
- type-data domains;
- type-data cases;
- associated type-data declarations;
- `::` references;
- generic static-domain references.

LSP support should provide:

- go-to-definition from `Shape::Vector`;
- hover classification distinguishing runtime `data` from `type data`;
- hover of static-term domain;
- completion for associated type-data cases;
- rename based on semantic identity;
- diagnostics for obsolete `enum`;
- navigation from `(Int, String)::Positionals` to canonical Tuple/domain member declaration where supported.

Tooling must not infer identity from rendered strings.

---

## 42. Specification migration requirements

The following authoritative areas require coordinated revision.

### `docs/specs/objects/data-objects.md`

Must be generalized from product-only data objects to unified runtime algebraic data, or split into a general algebraic-data owner plus product-specific subsections.

The statement that `data` is not a one-case enum must be removed/superseded.

### `docs/specs/objects/enums.md`

Its semantic content must migrate to sum-shaped `data` and exact cases.

The `enum` keyword becomes historical/compatibility syntax only.

### `docs/specs/type-level-domains.md`

Must generalize the fixed `Kind(κ) | Const(τ)` model to admit nominal user-defined static domains without turning each domain into a primitive `KindData` case.

### `docs/specs/type-members.md`

Must define `type` result domains over user-defined static domains and associated-domain lookup interaction.

### `docs/specs/associated-types.md`

Must explicitly preserve trait-associated type requirements as distinct from inherent associated `type data` declarations.

### `docs/specs/objects/impl.md`

Must rename enum-root/exact-enum-case target language to sum-data-root/exact-case semantics and define static type-member impl legality for type-data domains.

### Pattern/GADT specifications

Must trigger closed-sum analysis from algebraic shape, not an `enum` AST tag.

---

## 43. Compatibility and migration strategy

### 43.1 Source migration

Canonical migration:

```text
enum X { ... }
    →
data X { ... }
```

Existing bare variants, payload variants, GADT result annotations, exact-case impl targets, and patterns should otherwise retain their current meaning.

### 43.2 Compatibility window

A temporary parser compatibility mode may accept `enum`, provided:

- it emits a migration diagnostic;
- it produces the canonical unified algebraic AST;
- no semantic product records “enum” as a distinct source meaning;
- new formatting/docs/codegen use `data`.

### 43.3 Removal

After the migration window, `enum` is removed from canonical grammar and may cease to be a reserved declaration keyword if no other lexical rule requires it.

### 43.4 Internal names

Rust/internal names containing `Enum` or `Variant` need not be mechanically renamed in the first patch if they remain semantically correct and non-public.

However, persistent metadata, diagnostics, source-facing reflection, and new architectural APIs should converge on neutral algebraic/sum/case terminology.

---

## 44. Performance requirements

C3 must preserve approximately linear behavior in ordinary declaration and term size.

Requirements:

- case duplicate detection uses hash/set indexing, not quadratic scans where avoidable;
- static constructor term interning is hash-consed/canonical;
- associated lookup uses declaration/domain indices;
- substitution walks each static term structure proportionally to reachable size;
- no recursive pathological concat trees are introduced for packs;
- metadata round-trip does not stringify/parse static terms;
- exact-case exhaustiveness remains bounded by the existing accepted case-analysis strategy.

C3 does not require aggressive compile-time evaluation of user code, avoiding a new unbounded evaluator cost center.

---

## 45. Security/soundness requirements

The compiler must reject any route by which runtime-dependent state enters durable type identity.

Static type-data terms must not depend on:

- I/O;
- wall-clock time;
- randomness;
- mutable global state;
- object address/identity;
- resource handles;
- arbitrary closures/environment identity;
- runtime method dispatch.

If future static evaluation is added, it must have its own purity/termination/authority specification.

---

## 46. Test requirements

The implementation must include focused coverage for at least the following matrices.

### Syntax

- runtime tuple product;
- runtime record product;
- runtime singleton sum;
- runtime payload sum;
- runtime record-payload sum;
- runtime GADT;
- type-data versions of each supported shape;
- `intrinsic data`;
- `intrinsic type data`;
- associated `type data`;
- mixed-body rejection;
- empty product versus empty sum;
- obsolete `enum` migration.

### Identity

- structurally identical declarations remain nominally distinct;
- cases with identical payloads remain distinct;
- type-data terms canonicalize across repeated formation;
- different type-data constructors do not unify;
- associated domain specializations retain owner environment;
- declaration aliases do not invent new identities.

### Generic/static application

- `Tensor<Float, Shape::Vector(3)>`;
- static-domain mismatch;
- type-data term through alias;
- cross-module type-data argument;
- incremental edit of static argument;
- pack mode/domain orthogonality.

### Associated lookup

- `Parser::State`;
- `Parser::State::Done`;
- specialized generic owner associated domain;
- `(Int, String)::Positionals`;
- ambiguous/colliding associated base;
- unspecialized family used as concrete domain.

### ADT/GADT preservation

- exact-case construction;
- exact-case impl;
- exhaustive match;
- non-exhaustive match;
- GADT branch refinement;
- root behavior completeness;
- singleton vs zero-arg constructor.

### Metadata/incremental

- metadata round-trip;
- cold/imported equality;
- incremental case addition/removal;
- payload type edit;
- phase change;
- associated type-data edit;
- static-domain generic signature edit;
- no query-local IDs in metadata.

### Runtime boundary

- type-data term emits no runtime construction bytecode;
- runtime data sums/products preserve current runtime semantics;
- Tuple/Record continue using specialized runtime representations;
- no VM static-domain solver path.

---

## 47. Acceptance criteria

LANG007.C3 is accepted when all of the following are true.

1. `data` is the canonical source form for products, closed sums, ADTs, and GADTs.
2. `enum` has no distinct semantic declaration category.
3. `type data` is implemented as a type-system-only algebraic domain with canonical terms and no runtime inhabitants.
4. User-defined `type data` domains can classify generic/static parameters.
5. Generic applications can preserve heterogeneous static arguments without fake proper types.
6. Associated `type data` works in nominal owners and resolves through the canonical `::` namespace.
7. Existing type-member result-domain semantics work with user-defined static domains.
8. Exact-case/GADT/exhaustiveness semantics are preserved under sum-shaped `data`.
9. `TypePack`, `LabeledTypePack`, `TupleEntries`, and `RecordRow` can be exposed as `intrinsic type data` without duplicating C1/C2 canonical terms.
10. Tuple/Record/Unit can be exposed coherently as `intrinsic data` while keeping existing runtime representation freedom.
11. `*`, `**`, and `***` agree with the exposed static domains.
12. Metadata and incremental compilation preserve canonical static-domain identity cross-module.
13. LSP/source indexing recognizes the new declarations and associated lookups.
14. No VM type-data execution/solver architecture is introduced.
15. Authoritative specifications no longer contradict the unified algebraic model.
16. Focused and corpus-level regressions demonstrate no semantic loss in existing product, ADT/GADT, associated-type, pack, and Tuple/Record behavior.

---

# Part III — Checkpoint Decomposition

## 48. LANG007.C3 checkpoint objective

**Checkpoint:** `LANG007.C3`

**Title:** Type Data and Unified Algebraic Static Domains

**Objective:**

> Establish one coherent algebraic declaration system in which `data` defines runtime closed algebraic domains, `type data` defines type-system-only algebraic domains, associated static domains participate in the canonical `::` namespace, and compiler intrinsic structural domains are exposed through the same language model while preserving existing C1/C2 pack, type-member, ADT/GADT, metadata, and runtime boundaries.

C3 follows completion of `LANG007.C2.P3`.

It must not be folded into C2.P3.

---

## 49. LANG007.C3.P1 — Unified Algebraic Data Declarations and Type-Data Foundation

P1 should establish the declaration ontology and preserve runtime semantics.

### Required scope

- unified algebraic declaration AST/semantic product;
- `enum` → `data` canonical syntax migration;
- runtime product/sum shape classification;
- singleton/zero-arg/payload/record-payload cases;
- GADT case result specialization under `data`;
- `type data` syntax and declaration identity;
- canonical type-data case/constructor identity;
- type-only formation and runtime-use rejection;
- `intrinsic data` / `intrinsic type data` legality;
- preservation of exact-case, match, root-contract, and runtime lowering semantics;
- first metadata form for type-data declaration/case identity;
- authoritative spec migration for data/enums.

### P1 non-goals

- language-wide heterogeneous generic applications;
- full user static-domain inference;
- associated `type data`;
- Universe migration of TypePack/Tuple;
- general static evaluation.

### P1 gate

At P1 completion:

```text
data products work
data sums work
GADTs work under data
type data declarations and canonical terms exist
enum has no distinct semantic identity
runtime behavior is preserved
```

---

## 50. LANG007.C3.P2 — Generalized Static Domains, Associated Type Data, and Static-Term Application

P2 should integrate type data into generic/type-member semantics.

### Required scope

- nominal `StaticDomainId` or equivalent;
- heterogeneous canonical static-term representation;
- generic binder domains over user `type data`;
- generic applications with type-data terms;
- type-data constructor unification;
- substitution/materialization;
- static term aliases with explicit result domains where accepted;
- associated `type data` in classes/data owners;
- specialized associated-domain identity;
- `::` lookup integration;
- namespace collision diagnostics;
- metadata round-trip for static terms and associated domains;
- cross-module and incremental invalidation;
- LSP/indexing baseline.

### P2 gate

At P2 completion:

```phalcom
type data Shape {
    Scalar
    Vector(Int)
}

data Tensor<T: Type, S: Shape>

type V = Tensor<Float, Shape::Vector(3)>

class Parser {
    type data State {
        Start
        Done
    }

    type Initial: State
}
```

must have complete canonical semantics across source, checking, inference, metadata, imports, and incremental analysis.

---

## 51. LANG007.C3.P3 — Intrinsic Structural Domain Surface, Universe Migration, and Certification

P3 should expose the built-in structural model using the new language facilities and close C3.

### Required scope

Canonical Universe surface equivalent to:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries
intrinsic type data RecordRow

intrinsic data Unit()
intrinsic data Tuple<***Es>
intrinsic data Record<R: RecordRow>
```

plus type-level members:

```phalcom
impl Tuple<***Es> {
    intrinsic type Entries: TupleEntries
    intrinsic type Positionals: TypePack
    intrinsic type Labeled: LabeledTypePack
    intrinsic type At<I: Int>: Type
    intrinsic type Map<F: Type -> Type>: Type
}
```

P3 also owns:

- public `TupleEntries` mapping to canonical C2 complete-pack representation;
- `*` / `**` / `***` equivalence certification;
- migration of core closed sums (`Option`, `Result`, `Ordering`, etc.) to `data`;
- stale enum terminology cleanup in public specs/tooling;
- complete metadata/interface fingerprints;
- LSP hover/navigation/completion;
- corpus migration;
- performance checks;
- cold/incremental parity;
- C3 checkpoint closure and successor handoff.

---

# Part IV — Architectural Risks and Stop Conditions

## 52. Major risks

### 52.1 Treating `type data` as another primitive kind

This would make every user declaration require changes to a closed compiler enum and defeat the purpose of user-defined static domains.

**Required response:** introduce nominal static-domain identity.

### 52.2 Encoding static terms as fake proper types

This would make APIs expecting `Type` accidentally accept `Shape::Vector(3)` and destroy kind/domain soundness.

**Required response:** preserve a proof/category distinction between proper types and arbitrary static terms.

### 52.3 Reimplementing ADTs under `data`

The repository already has mature exact-case/GADT/exhaustiveness machinery.

**Required response:** migrate ownership and syntax; reuse semantics.

### 52.4 Replacing C2 complete packs

C2 already has canonical complete-pack identity and projection/substitution machinery.

**Required response:** `TupleEntries` is the public declaration surface over that substrate.

### 52.5 Confusing associated `type data` with trait-associated types

Trait-associated types are evidence-bound requirements. Nested/inherent associated type-data domains are declaration-owned.

**Required response:** preserve separate semantic categories under a shared projection/lookup infrastructure.

### 52.6 Introducing arbitrary compile-time execution accidentally

Type-data algebra only requires canonical terms, not a full evaluator.

**Required response:** keep C3 normalization structural and closed.

### 52.7 Forcing one runtime representation

Source unification does not require storage unification.

**Required response:** keep runtime lowering representation-specialized.

---

## 53. Mandatory STOP/CONSULT conditions during implementation planning/execution

Stop and reconcile architecture before proceeding if repository work reveals any of the following:

1. `TypeId`/`ProperTypeId` architecture cannot express a general static-term carrier without unsound coercion.
2. Generalizing nominal generic applications requires a breaking trait/conformance representation change larger than the accepted C3 scope.
3. `EnumDef`/`DataDef` convergence would erase exact-case identity or GADT evidence.
4. Associated `type data` requires redefining trait-associated-type evidence rather than sharing lookup infrastructure.
5. C2 `CompleteTypePack` cannot serve as canonical `TupleEntries` semantics without observable mismatch.
6. Metadata cannot represent static-domain declaration identity without source-string identity.
7. A proposed implementation introduces VM execution as the authority for static terms.
8. Runtime `data` and `type data` become distinguishable only by ad hoc parser flags rather than semantic declaration phase.
9. Existing accepted zero-product, Tuple, Record, pack, or associated-type invariants would need weakening.
10. Scope expands into arbitrary user-defined static functions, theorem proving, or unrestricted compile-time evaluation.
11. A public `enum` compatibility path would create distinct semantic identities from equivalent `data`.
12. The implementation requires duplicating case/member lookup authorities.

---

## 54. Final semantic summary

The complete model is:

```text
data
    one closed runtime algebraic domain
    ├── tuple product
    ├── record product
    └── closed sum
        ├── singleton case
        ├── zero-arg constructor case
        ├── tuple-payload case
        ├── record-payload case
        └── GADT-specialized case

type data
    the same algebraic declaration vocabulary,
    but inhabitants exist only as canonical type-system terms
```

The static language then has multiple domains:

```text
Type
Int              (compiler-supported static constants)
Symbol           (when supported as a static constant domain)
user type data   (Shape, Endian, Dimension, ...)
TypePack
LabeledTypePack
TupleEntries
RecordRow
```

and generic/static binders are uniformly domain-classified:

```phalcom
T: Type
N: Int
S: Shape
P: TypePack
E: TupleEntries
R: RecordRow
```

Associated lookup remains one system:

```phalcom
Option::Some
Parser::State
Parser::State::Done
(Int, String)::Positionals
(Int, String)::At<0>
```

Compiler-owned structural semantics become expressible in the same public language:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries
intrinsic type data RecordRow

intrinsic data Unit()
intrinsic data Tuple<***Es>
intrinsic data Record<R: RecordRow>
```

The implementation remains free to retain specialized arenas, kind IDs, exact-case records, tuple/record runtime objects, and pack normalizers. The unification is a **language-semantic and declaration-identity unification**, not a requirement to collapse every internal representation.

That boundary is the central architectural invariant of LANG007.C3.
