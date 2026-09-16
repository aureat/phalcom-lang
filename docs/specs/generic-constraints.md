# Generic Constraints and Trait Bounds

**Status:** Normative  
**Language area:** Type system, generics, trait conformance  
**Implementation checkpoint:** LANG005.C6  
**Specification role:** Authoritative surface-language and semantic specification for generic constraints and trait bounds

This document specifies Phalcom's generic constraint language: the `where` clause, kind constraints, trait-conformance requirements, associated-type specialization, associated projections, type-equivalence constraints, subtype constraints, generic evidence environments, and the rules by which these constraints participate in generic inference and instantiation.

The document defines language behavior. Concrete solver algorithms, query scheduling, cache organization, incremental implementation, worklist strategy, union-find representation, and other implementation mechanisms are outside its normative scope except where observable semantics require a specific distinction.

## Semantic model

A generic declaration introduces abstract type-level parameters. A `where` clause states propositions that must hold about those parameters or about type-level expressions derived from them.

The central syntactic invariant is:

> Generic parameter lists introduce generic parameters. `where` clauses constrain them.

The central semantic invariant is:

> A `where` clause denotes a conjunction of proof-bearing propositions. The generic body is checked under those propositions, and every concrete instantiation must discharge the corresponding obligations.

Phalcom's initial generic constraint language contains four principal relation families:

```phalcom
F: (Type) -> Type
T |= Iterator<Item = User>
Lower <: T <: Upper
A::Item == B::Item
```

They are semantically distinct:

```text
:     kind classification
|=    trait conformance / evidence requirement
<:    subtype relation
==    canonical semantic type equivalence
```

Supporting syntax includes:

```text
::    associated selection / projection
+     conjunction of same-subject trait requirements
=     named associated-type specialization inside a constrained trait application
```

These relations must not be conflated. In particular:

- trait conformance is not subtyping;
- trait conformance is not kinding;
- type equivalence is not assignment;
- type equivalence is not defined as mutual assignability;
- `+` in a trait bound does not construct an intersection type or synthetic trait;
- `Dynamic` is not proof of any static bound.

All constraints in one `where` clause are conjunctive. Their source order has no semantic significance.

## Generic parameter introduction

A generic parameter list binds generic parameters:

```phalcom
<T>
```

```phalcom
<T, U>
```

```phalcom
<F, G, A, B, E>
```

The parameter list itself does not contain generic bounds or relational constraints.

The canonical form is therefore:

```phalcom
print<T>(_ value: T)
    where T |= Printable
{
    ...
}
```

not:

```phalcom
print<T: Printable>(_ value: T) {
    ...
}
```

Likewise, higher-kind constraints belong in `where` rather than in the generic parameter list:

```phalcom
map<F, A, B>(
    _ value: F<A>,
    _ transform: (A) -> B,
) -> F<B>
    where
        F: (Type) -> Type,
        F |= Functor,
{
    ...
}
```

A `where` clause never introduces an implicit type parameter.

Therefore:

```phalcom
foo<T>()
    where T == U
{
    ...
}
```

is invalid unless `U` already resolves as a type-level name in the enclosing scope.

This separation is normative:

```text
<T, U, F>
    binds generic parameters

where ...
    constrains already-bound or otherwise-resolvable type-level entities
```

## Where-clause placement

A `where` clause is part of a declaration header. Its grammar is independent of physical line breaks.

A short clause may appear on the same line:

```phalcom
print<T>(_ value: T) where T |= Printable {
    ...
}
```

The same declaration may place the clause on the following line:

```phalcom
print<T>(_ value: T)
    where T |= Printable
{
    ...
}
```

A longer clause may be expanded:

```phalcom
print<T>(_ value: T)
    where
        T |= Printable + Hashable,
        T <: Entity,
{
    ...
}
```

These forms are semantically identical.

The language grammar must not depend on whether `where` appears on the same physical line as the declaration head.

The formatter should generally:

- retain genuinely short, simple clauses inline when doing so preserves readability;
- move nontrivial `where` clauses below the principal declaration signature;
- place multiple substantial constraints on separate lines;
- use continuation indentation for long same-subject trait conjunctions.

Formatting is not semantic.

## Declarations that admit constraints

A `where` clause may appear on generic declaration forms whose semantics admit generic constraints.

The normative surface includes at least:

- functions;
- methods;
- generic nominal declarations;
- traits;
- `impl` / conformance declarations;
- generic type aliases;
- GADT enum variant constructors;
- other declaration forms that bind generic parameters and explicitly adopt the generic constraint grammar.

Examples:

```phalcom
print<T>(_ value: T)
    where T |= Printable
{
    ...
}
```

```phalcom
class Box<T>
    where T |= Serializable
{
    ...
}
```

```phalcom
impl<T> Printable for Box<T>
    where T |= Printable
{
    ...
}
```

A generic alias may carry constraints:

```phalcom
type Items<I> = List<<I as Iterator>::Item>
    where I |= Iterator
```

A GADT constructor may carry constructor-specific constraints:

```phalcom
enum Expr<T> {
    IntLiteral(Int)
        where T == Int

    Wrapped<U>(Expr<U>)
        where T == Option<U>
}
```

The exact surrounding grammar of each declaration kind remains owned by that declaration's specification. The meaning of its `where` clause is governed by this document.

## Constraint grammar

The following grammar is illustrative normative grammar. Concrete parser notation may differ while preserving the same accepted language.

```text
GenericParameterList
    := "<" GenericParameter ("," GenericParameter)* [","] ">"

GenericParameter
    := Identifier

WhereClause
    := "where" Constraint ("," Constraint)* [","]

Constraint
    := KindConstraint
     | ConformanceConstraint
     | EquivalenceConstraint
     | SubtypeConstraint

KindConstraint
    := GenericParameter ":" Kind

ConformanceConstraint
    := TypeLevelExpression "|=" TraitBoundList

TraitBoundList
    := TraitBound ("+" TraitBound)*

TraitBound
    := TraitReference
     | TraitReference "<" TraitBoundArgumentList ">"

TraitBoundArgumentList
    := TraitBoundArgument ("," TraitBoundArgument)* [","]

TraitBoundArgument
    := TypeExpression
     | AssociatedSpecialization

AssociatedSpecialization
    := Identifier "=" TypeExpression

EquivalenceConstraint
    := TypeLevelExpression "==" TypeLevelExpression

SubtypeConstraint
    := ProperTypeExpression ("<:" ProperTypeExpression)+
```

The exact productions for `Kind`, `TypeExpression`, `TypeLevelExpression`, `ProperTypeExpression`, and `TraitReference` are defined by their owning specifications.

The grammar must preserve the semantic distinction between ordinary trait generic arguments and associated-type specializations.

## Kinds and kind constraints

A kind classifies a type-level entity in the same broad sense that a type classifies a value-level entity.

At the value level:

```phalcom
value: Int
```

means that `value` has type `Int`.

At the type level:

```phalcom
F: (Type) -> Type
```

means that `F` has kind `(Type) -> Type`.

The use of `:` is therefore a classification relation across adjacent levels:

```text
value : type
type-level entity : kind
```

Kind classification is semantically distinct from trait conformance.

These two constraints state different facts:

```phalcom
F: (Type) -> Type
F |= Functor
```

The first determines the category of type-level entity that `F` ranges over. The second requires conformance evidence for `Functor`.

### Default kind

An ordinary generic parameter defaults to kind `Type` when no explicit or inferred higher kind applies.

Thus:

```phalcom
identity<T>(_ value: T) -> T {
    value
}
```

does not require:

```phalcom
where T: Type
```

Conceptually:

```text
T : Type
```

is implicit.

### Explicit higher kinds

Explicit higher-kind constraints are recommended when the public contract depends on a type constructor rather than a proper type.

Preferred forms use callable-like kind syntax:

```phalcom
F: (Type) -> Type
G: (Type, Type) -> Type
H: (Type, Type, Type) -> Type
```

These correspond naturally to Phalcom's n-ary generic application syntax:

```phalcom
F<A>
G<A, B>
H<A, B, C>
```

The unparenthesized unary shorthand is permitted:

```phalcom
F: Type -> Type
```

and is semantically equivalent to:

```phalcom
F: (Type) -> Type
```

An accepted unparenthesized multi-argument form such as:

```phalcom
F: Type -> Type -> Type
```

must canonicalize to the corresponding n-ary callable-like kind:

```phalcom
F: (Type, Type) -> Type
```

for purposes of kind identity, diagnostics, reflection, and semantic comparison.

The source spelling does not introduce currying or partial type-constructor application unless a separate language feature explicitly does so.

A formatter or linter may prefer the canonical callable-like spelling:

```phalcom
(Type, Type) -> Type
```

over:

```phalcom
Type -> Type -> Type
```

### Kind inference

Kinds may be inferred from type-forming use and from trait subject requirements.

For example:

```phalcom
map<F, A, B>(
    _ value: F<A>,
    _ transform: (A) -> B,
) -> F<B>
    where F |= Functor
{
    ...
}
```

may infer that `F` must accept one proper type and produce one proper type when both:

- `F<A>` is well-formed only under that kind; and
- `Functor` requires a subject of that kind.

Explicit higher kinds remain recommended when they are part of the API contract:

```phalcom
where
    F: (Type) -> Type,
    F |= Functor,
```

Kind inference and explicit kind constraints contribute to one coherent kind system.

Incompatible kind information is always a hard error.

For example, if one use requires:

```text
F : (Type) -> Type
```

while another requires:

```text
F : (Type, Type) -> Type
```

the declaration is ill-kinded.

The compiler must not widen the conflict to an unknown or dynamic kind.

### Domain of explicit kind constraints

An explicit kind constraint in `where` directly constrains a generic parameter introduced by the enclosing declaration.

This is valid:

```phalcom
where F: (Type) -> Type
```

This is not the intended explicit constraint form:

```phalcom
where F<A>: Type
```

The kind of a compound type expression is checked through ordinary type formation rather than by restating it as a source-level `where` proposition.

## Trait subject kinds

Every trait application has a subject kind: the kind of entity that may conform to that trait.

Most ordinary traits accept a subject of kind:

```text
Type
```

Higher-kinded traits may accept subjects such as:

```text
(Type) -> Type
```

Accordingly:

```phalcom
F |= Functor
```

is well-kinded only if the kind of `F` is compatible with the subject kind required by `Functor`.

Kind compatibility is checked before conformance proof is meaningful.

A trait applicable only to proper types cannot accidentally apply to a type constructor merely because member names happen to match.

The exact declaration syntax by which a trait establishes its subject kind is owned by the trait and kind specifications.

## Trait conformance constraints

Trait conformance uses `|=`.

```phalcom
where T |= Printable
```

is read:

> `T` conforms to `Printable`.

Semantically, the generic environment must provide evidence for the proposition:

```text
Conforms(T, Printable)
```

The operator `|=` in a `where` clause is a built-in semantic relation.

It is:

- not compound assignment;
- not bitwise assignment;
- not overloadable;
- not structural method matching;
- not subtyping;
- not a type annotation.

A valid conformance requirement must resolve to canonical trait evidence according to Phalcom's conformance and coherence model.

Method-shape coincidence alone is not proof.

## General conformance subjects

The subject of `|=` may be any well-formed type-level expression whose kind is compatible with the subject kind of the referenced trait application.

Examples include:

```phalcom
T |= Printable
```

```phalcom
List<T> |= Printable
```

```phalcom
F<T> |= Serializable
```

```phalcom
I::Item |= Hashable
```

and, for a higher-kinded trait:

```phalcom
F |= Functor
```

where `F` itself has an appropriate higher kind.

The semantic primitive is therefore general:

```text
Conforms(subject, trait_application)
```

It is not restricted to a special representation such as:

```text
BoundOnGenericParameter(parameter, trait)
```

Before proof search, the conformance subject is kind-checked and normalized according to all semantic normalization available in the current environment.

Normalization must preserve distinctions relevant to conformance identity, including exact-case identity and canonical applied trait identity.

## Conformance evidence

A conformance constraint introduces abstract evidence into a generic body's semantic environment.

Given:

```phalcom
print<T>(_ value: T)
    where T |= Printable
{
    ...
}
```

the body is checked under the assumption:

```text
T |= Printable
```

Operations justified by `Printable` may therefore be type-checked against `T`.

At a concrete instantiation, the caller must discharge the same proposition with concrete conformance evidence.

Generic assumed evidence and exact concrete evidence are distinct sources of knowledge but participate in one semantic conformance model.

## Trait conjunction

Multiple traits on the same subject may be conjoined with `+`:

```phalcom
where T |= Printable + Hashable + Serializable
```

This is semantic sugar for:

```phalcom
where
    T |= Printable,
    T |= Hashable,
    T |= Serializable,
```

Each trait contributes an independent conformance obligation.

No synthetic combined trait is created.

Therefore:

```phalcom
T |= A + B + C
```

normalizes conceptually to:

```text
Conforms(T, A)
Conforms(T, B)
Conforms(T, C)
```

and not:

```text
Conforms(T, A+B+C)
```

The `+` operator in this position:

- does not construct an intersection type;
- does not construct a trait object;
- does not create a reflection identity;
- does not affect coherence as though a new trait existed;
- is local to same-subject conformance-bound syntax.

Operand order is semantically irrelevant:

```phalcom
T |= A + B
```

and:

```phalcom
T |= B + A
```

state the same set of obligations.

Likewise:

```phalcom
T |= A + B
```

and:

```phalcom
where
    T |= A,
    T |= B,
```

normalize to the same semantic obligations.

### Conjunction precedence

Associated specialization binds inside the immediately enclosing trait bound before trait conjunction is expanded.

Thus:

```phalcom
T |= Iterator<Item = A> + Printable
```

means:

```text
T |= Iterator<Item = A>
AND
T |= Printable
```

A comma terminates the same-subject conjunction:

```phalcom
where
    T |= A + B,
    U |= C,
```

means:

```text
T |= A
T |= B
U |= C
```

## Generic trait applications

A generic trait is referenced with ordinary positional generic arguments.

Given:

```phalcom
trait Converts<From, To> {
    type Error

    convert(_ value: From) -> Result<To, Self::Error>
}
```

the following constraint refers to one canonical trait application:

```phalcom
C |= Converts<String, Int>
```

Trait application identity includes its ordinary generic arguments.

Therefore:

```text
Converts<String, Int>
```

and:

```text
Converts<Bytes, Int>
```

are different trait applications even though they share the same trait declaration.

A type may be required to satisfy more than one applied form where coherence rules permit:

```phalcom
T |= Trait<A> + Trait<B>
```

This denotes two independent conformance obligations.

Phalcom does not have labeled ordinary generic arguments.

Accordingly:

```phalcom
Converter<From = A, To = B>
```

does not name ordinary generic parameters by label.

Within a constrained trait application, `Name = Type` has one exclusive meaning: associated-type specialization.

## Associated-type specialization

A conformance bound may constrain associated types directly.

```phalcom
where I |= Iterator<Item = User>
```

requires both:

```text
I |= Iterator
```

and:

```text
<I as Iterator>::Item == User
```

The source form is intentionally compact and trait-anchored.

Multiple associated types may be specialized:

```phalcom
where
    I |= Iterator<
        Cursor = Int,
        Item = User,
    >
```

Each associated name is resolved against the canonical associated-type requirements of the immediately enclosing trait application.

Names are not looked up globally.

### Generic arguments and associated specialization together

A trait bound may contain both ordinary positional generic arguments and named associated specializations:

```phalcom
C |= Converts<A, B, Error = E>
```

Given:

```phalcom
trait Converts<From, To> {
    type Error
}
```

the semantic partition is:

```text
A, B
    ordinary trait generic arguments

Error = E
    associated-type specialization
```

The associated specialization does not become part of the trait's ordinary generic argument list.

It constrains an associated projection of the already-identified trait application.

Once an associated specialization appears inside the angle-bracket list, subsequent entries must also be associated specializations.

Thus:

```phalcom
Converts<A, B, Error = E>
```

is valid.

A form such as:

```phalcom
Converts<A, Error = E, B>
```

is invalid.

### Single `=` in associated specialization

Single `=` inside:

```phalcom
Iterator<Item = User>
```

is not the general type-equivalence operator.

It is structured named-specialization syntax:

```text
associated requirement Item
must resolve to
User
```

Its semantic effect is an equivalence obligation, but its surface role is analogous to specifying the required value of a named associated member.

General type equivalence uses `==`.

### Duplicate associated specialization

The same associated requirement may appear at most once in one constrained trait application.

This is invalid:

```phalcom
Iterator<Item = A, Item = B>
```

This is also invalid:

```phalcom
Iterator<Item = A, Item = A>
```

Both are duplicate structured specializations and must produce a diagnostic.

The compiler must not reinterpret:

```phalcom
Item = A, Item = B
```

as an implicit request to prove:

```text
A == B
```

If an additional equality is intended, it must be stated as a general `where` constraint.

### Narrow specialization language

Associated specialization intentionally supports exact specialization only:

```phalcom
Iterator<Item = User>
```

These forms are not part of the associated-specialization mini-language:

```phalcom
Iterator<Item <: Entity>
Iterator<Item == Other::Item>
```

General relations remain ordinary `where` constraints:

```phalcom
where
    I |= Iterator,
    I::Item <: Entity,
```

or:

```phalcom
where
    I |= Iterator,
    I::Item == Other::Item,
```

This prevents `Trait<...>` from embedding a second copy of the full constraint language.

## Associated projections

Associated types may be projected from a subject when the relevant conformance context identifies the associated requirement.

Phalcom supports contextual projections:

```phalcom
Self::Item
```

```phalcom
T::Item
```

and explicit qualified projections:

```phalcom
<T as Iterator>::Item
```

```phalcom
<T as Converter<A, B>>::Error
```

### Contextual projection

A contextual projection such as:

```phalcom
T::Item
```

is valid only when the available semantic environment identifies exactly one applicable associated requirement named `Item` for the subject.

For example:

```phalcom
where T |= Iterator
```

may make:

```phalcom
T::Item
```

unambiguous if `Iterator` is the only applicable conformance exposing `Item`.

Source order must never choose among multiple candidates.

### Ambiguous contextual projection

Suppose:

```phalcom
trait Iterator {
    type Item
}

trait Collection {
    type Item
}
```

and:

```phalcom
where T |= Iterator + Collection
```

Then:

```phalcom
T::Item
```

is ambiguous because two distinct canonical associated requirements are available:

```text
Iterator::Item
Collection::Item
```

The compiler must diagnose the ambiguity.

### Qualified projection

A qualified projection names the trait application explicitly:

```phalcom
<T as Iterator>::Item
```

This identifies the projection by:

```text
subject
trait application
canonical associated requirement
```

It is therefore unambiguous even when several traits expose the same associated name.

For generic traits:

```phalcom
<T as Converter<A, B>>::Error
```

the ordinary trait arguments are part of the qualifying trait application and therefore part of projection identity.

### Qualified projection and conformance obligations

A qualified projection is meaningful only if the subject conforms to the qualifying trait application.

Accordingly, forming:

```phalcom
<T as Iterator>::Item
```

creates or requires the well-formedness obligation:

```text
T |= Iterator
```

The programmer need not redundantly state the same conformance solely to make the projection well-formed.

Thus this is a complete relation:

```phalcom
where
    <T as Iterator>::Item
        == <T as Collection>::Item
```

It entails the well-formedness obligations necessary to form both projections:

```text
T |= Iterator
T |= Collection
```

and then constrains the resulting associated types to be equivalent.

Explicit qualification is the canonical escape hatch for otherwise ambiguous associated projections.

### Same-named associated requirements

Inline specialization remains naturally unambiguous:

```phalcom
where
    T |= Iterator<Item = A>
       + Collection<Item = B>
```

Each `Item` belongs to its immediately enclosing trait bound.

If both associated types must be the same, either of the following is valid:

```phalcom
where
    T |= Iterator<Item = X>
       + Collection<Item = X>
```

or:

```phalcom
where
    <T as Iterator>::Item
        == <T as Collection>::Item
```

The first expresses equality by sharing a type parameter. The second expresses the projection relationship directly.

## Type equivalence constraints

General type equivalence uses `==`.

```phalcom
where A == B
```

is a type-level proposition, not a runtime equality operation and not an assignment.

The normative meaning is:

> `A == B` holds when `A` and `B`, after all normalization justified by the current semantic environment, denote the same canonical semantic type.

Relevant normalization may include:

- generic substitution;
- transparent type aliases;
- canonical union normalization;
- associated projection normalization;
- generic assumed conformance evidence;
- exact conformance evidence;
- canonical tuple, record, callable, and other structural type normalization where applicable;
- any other normative canonical type reduction defined by the type system.

The relation is symmetric.

```phalcom
A == B
```

and:

```phalcom
B == A
```

state the same proposition.

### Mathematical canonicalization

Phalcom's type equivalence follows canonical semantic meaning rather than source spelling wherever the type system defines mathematically equivalent forms.

For canonical unions:

```phalcom
A | B == B | A
```

is true.

The same principle applies to other canonical forms whose owning type specification defines equivalent source representations.

The compiler may choose efficient canonical representations, interning, hash-consing, equivalence classes, or normalization strategies, but those implementation choices must preserve the specified semantic equivalence.

### Equivalence is not mutual assignability

`==` is not defined as:

```text
Assignable(A, B) AND Assignable(B, A)
```

unless the owning type theory independently proves those relations coincide for a specific pair.

Phalcom preserves distinct semantic relations for:

- canonical equivalence;
- subtyping;
- assignability;
- consistency;
- conformance;
- dynamic-boundary behavior.

The generic constraint language invokes the canonical equivalence relation specifically.

## Equality as a solver-producing constraint

Type-equivalence constraints participate in generic inference.

They are not merely post-inference assertions.

Consider:

```phalcom
first<I, T>(_ iterator: I) -> T
    where I |= Iterator<Item = T>
{
    ...
}
```

At generic-body checking time, `I` and `T` are rigid generic parameters. The compiler records the relation:

```text
<I as Iterator>::Item == T
```

without choosing a concrete type for `T`.

At a concrete instantiation, generic parameters may correspond to inference metavariables.

If the caller determines:

```text
I = UserIterator
```

and exact conformance evidence establishes:

```text
<UserIterator as Iterator>::Item == User
```

then the generic equality becomes:

```text
User == T
```

and may solve:

```text
T = User
```

Likewise:

```phalcom
process<T, U>(_ value: T)
    where T == List<U>
{
    ...
}
```

called with:

```text
T = List<Int>
```

may structurally constrain:

```text
U = Int
```

Equivalence constraints may propagate information in either direction when the combined constraint system uniquely determines a solution.

### No arbitrary model selection

Solver-producing does not mean that the compiler may invent arbitrary satisfying substitutions.

Given:

```phalcom
make<T, U>() -> T
    where T == List<U>
{
    ...
}
```

and no arguments, expected type, or other constraints determining either variable, infinitely many substitutions satisfy the relation.

The compiler must report underdetermined generic inference rather than arbitrarily choose:

```text
U = Int
T = List<Int>
```

The normative rule is:

> Equivalence constraints propagate information and may solve uniquely constrained inference variables. They do not select arbitrary substitutions from an underdetermined family of solutions.

## Subtype constraints

The subtype relation uses `<:`.

```phalcom
where A <: B
```

means:

> `A` is a subtype of `B`.

The left operand is the subtype. The right operand is the supertype.

The generic constraint language uses Phalcom's ordinary canonical subtype relation. It does not define a second subtype algorithm.

Accordingly, ordinary subtype semantics apply, including whatever the owning type specification establishes for:

- nominal inheritance;
- unions;
- exact enum cases;
- variance;
- `Never`;
- top types, if any;
- generic applications;
- associated projections;
- other proper-type forms.

### General subtype operands

Either side of `<:` may be any well-formed proper type expression of kind `Type`.

Examples:

```phalcom
List<A> <: Sequence<B>
```

```phalcom
I::Item <: Entity
```

```phalcom
F<A> <: G<B>
```

```phalcom
A | B <: C
```

Both operands must have kind `Type`.

Higher-kinded constructor subtyping is not introduced by this specification.

Thus:

```phalcom
where
    F: (Type) -> Type,
    G: (Type) -> Type,
    F <: G,
```

is invalid unless a future specification explicitly defines subtyping between higher-kinded entities.

## Lower and upper bounds

An upper bound is written:

```phalcom
where T <: Upper
```

A lower bound is written:

```phalcom
where Lower <: T
```

Both are applications of the same binary subtype relation.

Phalcom therefore supports simultaneous lower and upper bounds:

```phalcom
where Lower <: T <: Upper
```

This is syntactic composition of adjacent subtype propositions:

```phalcom
where
    Lower <: T,
    T <: Upper,
```

It does not construct a range type.

Longer chains are permitted by the same rule:

```phalcom
A <: B <: C <: D
```

and elaborate pairwise:

```text
A <: B
B <: C
C <: D
```

### Subtype constraints and inference

Subtype constraints restrict candidate substitutions but do not arbitrarily select a type from a valid interval.

Consider:

```phalcom
make<T>() -> T
    where Dog <: T <: Animal
{
    ...
}
```

Several types may satisfy the interval.

The compiler must not choose `Dog`, `Animal`, or another valid type merely because it satisfies the bounds.

An independently supplied expected type may determine the candidate:

```text
expected result = Mammal
```

after which the bounds verify:

```text
Dog <: Mammal <: Animal
```

The normative rule is:

> Subtype constraints narrow and validate inference. They do not invent an arbitrary point in an underdetermined subtype interval.

## Interaction between equivalence and subtyping

Canonical equivalence propagates through subtype constraints.

Given:

```phalcom
where
    A == B,
    A <: C,
```

the constraint set also entails:

```text
B <: C
```

Likewise:

```phalcom
where
    A == B,
    Lower <: A <: Upper,
```

constrains `B` to the same subtype interval.

This is not a special source-level rewrite rule.

It follows because `A` and `B` denote the same semantic type under `==`, so subsequent semantic relations operate over equivalent normalized representatives.

The implementation may realize this through canonicalization, unification, equivalence classes, substitution, or another sound mechanism.

## Constraint normalization

Surface syntax elaborates into a smaller canonical obligation language.

Consider:

```phalcom
where
    F: (Type) -> Type,

    I |= Iterator<Item = A>
       + Resettable
       + ShowDebug,

    Lower <: A <: Upper,
```

The semantic obligations are conceptually:

```text
Kind(
    subject = F,
    kind = (Type) -> Type
)

Conforms(
    subject = I,
    trait = Iterator
)

AssociatedEquivalent(
    subject = I,
    trait = Iterator,
    requirement = Iterator::Item,
    expected = A
)

Conforms(
    subject = I,
    trait = Resettable
)

Conforms(
    subject = I,
    trait = ShowDebug
)

Subtype(Lower, A)
Subtype(A, Upper)
```

The exact internal representation is implementation-defined.

The normative requirement is:

> Proof and inference semantics operate over canonical obligations, not over syntax-specific special cases.

Thus:

```phalcom
T |= A + B
```

and:

```phalcom
where
    T |= A,
    T |= B,
```

must normalize to the same conformance obligations.

Likewise:

```phalcom
Lower <: T <: Upper
```

and:

```phalcom
where
    Lower <: T,
    T <: Upper,
```

must normalize identically.

And:

```phalcom
I |= Iterator<Item = User>
```

contains the same associated-equivalence proposition as:

```phalcom
where
    I |= Iterator,
    I::Item == User,
```

when the unqualified projection is uniquely resolvable.

The inline form is preferable because its associated requirement identity is explicitly anchored to `Iterator`.

## Generic evidence environments

A generic body is checked under an abstract constraint environment derived from its normalized `where` obligations.

For:

```phalcom
consume<I>(_ input: I)
    where I |= Iterator<Item = User>
{
    ...
}
```

the body receives abstract knowledge equivalent to:

```text
I |= Iterator
<I as Iterator>::Item == User
```

This generic evidence may justify:

- trait member availability;
- trait-evidenced dispatch;
- associated projection formation;
- associated projection normalization;
- equality propagation;
- downstream subtype checks;
- nested generic obligations.

Generic assumed evidence is not exact concrete evidence, but both participate in a unified proof model.

## Associated projection through generic evidence

Given:

```phalcom
where I |= Iterator<Item = User>
```

the body may normalize:

```phalcom
I::Item
```

to:

```phalcom
User
```

when the projection is contextually unambiguous.

Likewise, a qualified projection:

```phalcom
<I as Iterator>::Item
```

normalizes through the exact trait application named in the qualification.

Generic projection normalization extends the associated-type machinery established by the associated-type specification. It does not create a second projection model.

## Trait-member availability

Generic code may use trait behavior when the generic environment provides the required evidence.

Given:

```phalcom
consume<I>(_ input: I)
    where I |= Iterator
{
    ...
}
```

members justified by `Iterator` are available on `I` through the conformance evidence.

Generic code does not gain trait methods merely because an unrelated type happens to contain selectors with matching shapes.

Trait behavior in a constrained generic body remains evidence-based.

## Constraint satisfaction at instantiation

At a concrete generic use, the compiler must discharge the declaration's normalized obligations under the concrete or inferred substitutions.

The semantic process must account for:

- known generic arguments;
- argument-derived inference;
- expected-type inference;
- kind constraints;
- canonical type normalization;
- generic and exact conformance evidence;
- associated-type bindings;
- type equivalence;
- subtype relations;
- nested conditional-conformance obligations.

The implementation may choose any sound solving order or fixed-point strategy that preserves the specified result.

Source order of constraints does not determine solving order.

### Associated-type-driven inference

Associated evidence may solve generic inference variables.

For:

```phalcom
foo<I, T>(_ input: I)
    where I |= Iterator<Item = T>
{
    ...
}
```

if `I` is known and exact `Iterator` evidence establishes:

```text
<I as Iterator>::Item == User
```

then `T` may be inferred as `User`.

### No reverse existential search

Knowing only:

```text
T = User
```

does not authorize the compiler to search the program for some arbitrary type `I` satisfying:

```text
I |= Iterator<Item = User>
```

Generic constraint solving constrains declared inference variables. It does not perform open-world existential type synthesis unless a separate language feature explicitly defines such search.

This asymmetry is normative.

## Constraint satisfiability

A generic declaration is invalid when its declared constraint set is statically provably inconsistent.

Examples include contradictions such as:

```phalcom
where
    T == Int,
    T == String,
```

when `Int` and `String` are canonically non-equivalent.

A kind contradiction is likewise invalid:

```phalcom
where
    F: (Type) -> Type,
    F == Int,
```

if `Int` has kind `Type`.

A subtype interval may also be statically contradictory when the subtype system can prove that no type can satisfy it.

The compiler is not required to decide arbitrary undecidable propositions at declaration time.

A constraint set may remain conditionally satisfiable until concrete substitutions are known.

The important distinction is:

- provable contradiction invalidates the generic declaration;
- unresolved conditional obligations remain obligations to be discharged at instantiation.

## Duplicate constraints

Exact duplicate top-level logical constraints are semantically redundant.

For example:

```phalcom
where
    T |= Printable,
    T |= Printable,
```

has the same logical meaning as:

```phalcom
where T |= Printable
```

The implementation may diagnose such duplication as a lint or style issue, but the duplicate does not create a second semantic conformance identity.

This differs from duplicate associated specialization within one trait application, which is a structural diagnostic error.

Repeated applications of the same generic trait with different generic arguments are not duplicates:

```phalcom
T |= Trait<A> + Trait<B>
```

denotes two distinct trait-application obligations.

## Conditional conformance

A conformance declaration may itself have a `where` clause.

```phalcom
impl<T> Printable for List<T>
    where T |= Printable
{
    ...
}
```

This defines a conditional conformance:

> `List<T>` conforms to `Printable` whenever `T` conforms to `Printable`.

An exact proof request for:

```text
List<User> |= Printable
```

therefore generates the nested obligation:

```text
User |= Printable
```

The outer conformance is proven only if the nested obligation is proven.

Conditional conformance may depend on associated-type specialization:

```phalcom
impl<I> Serializable for Stream<I>
    where I |= Iterator<Item = Byte>
{
    ...
}
```

It may also depend on equality and subtype constraints when those constraints are valid for the declaration:

```phalcom
impl<T> Trait for Wrapper<T>
    where Lower <: T <: Upper
{
    ...
}
```

Conditional conformance proof uses the same canonical obligation language as ordinary generic declarations.

## Recursive obligations

Conditional conformances and nested generic evidence may produce recursive proof graphs.

A conforming proof system must detect recursive obligation cycles rather than recurse indefinitely.

Conceptually:

```text
A |= Trait
    requires B |= Trait

B |= Trait
    requires A |= Trait
```

is a recursive proof cycle.

The language specification requires that recursion be represented as a distinct proof state until and unless another rule soundly resolves it.

The implementation must not silently convert recursion to:

- `Dynamic`;
- success;
- ordinary missing-conformance failure.

The exact fixed-point or cycle-resolution algorithm belongs to implementation/internal specifications.

## Proof outcomes

Generic proof machinery must preserve semantically distinct outcomes.

At minimum, the implementation model must be capable of distinguishing the equivalents of:

```text
proven
refuted
ambiguous
blocked
recursive
dynamic boundary
cancelled
budget exhausted
internal failure
```

Exact enum names are not normative.

The semantic distinctions are.

In particular:

- ambiguity is not missing evidence;
- blocked analysis is not refutation;
- recursion is not automatically failure or success;
- cancellation is not a semantic result;
- budget exhaustion is not proof of absence;
- a dynamic boundary is not proof of a static proposition.

## Dynamic boundaries

`Dynamic` does not discharge static generic constraints.

The central rule is:

> Dynamic knowledge represents absence or deferral of static proof. It cannot itself constitute proof of conformance, equivalence, or subtyping.

Therefore the compiler must not satisfy:

```phalcom
Dynamic |= Iterator
```

merely because the subject is dynamic.

It must not satisfy:

```phalcom
Dynamic == T
```

merely through gradual consistency.

It must not satisfy:

```phalcom
Dynamic <: T
```

or:

```phalcom
T <: Dynamic
```

merely because a dynamic boundary would permit a runtime operation.

If the static relation independently holds for a larger type expression containing a dynamic component without requiring `Dynamic` itself to satisfy a proof obligation, that larger proof is evaluated according to the ordinary static rules.

For example, an unconditional conformance of a generic outer type may still be provable for an instantiation containing `Dynamic` if no nested proof requires `Dynamic` to satisfy a trait or subtype relation.

Conversely, a conditional conformance such as:

```text
Wrapper<T> |= Printable
    if T |= Printable
```

cannot be proven for `T = Dynamic` by treating `Dynamic` as automatically printable.

A dynamic-boundary outcome remains distinct from static proof.

Unconstrained generic code may still accept a dynamic value where no static bound must be discharged:

```phalcom
identity<T>(_ value: T) -> T {
    value
}
```

Any future mechanism that explicitly checks a dynamic value at runtime and produces runtime conformance evidence is a separate language feature and does not alter the static meaning of ordinary C6 bounds.

## Generic aliases

Type aliases may be generic and may carry `where` clauses.

Example:

```phalcom
type Items<I> = List<<I as Iterator>::Item>
    where I |= Iterator
```

The alias's constraints are part of its generic contract.

Using the alias requires the constraints to be satisfied under the concrete alias arguments.

An alias `where` clause is not a textual comment and is not discarded after expansion.

Aliases participate in ordinary:

- kind checking;
- conformance obligations;
- associated projection normalization;
- equality solving;
- subtype obligations.

Alias expansion and generic constraint solving must preserve canonical type equivalence.

## GADT variant constructors

GADT enum variant constructors may carry constructor-specific `where` clauses.

For example:

```phalcom
enum Expr<T> {
    IntLiteral(Int)
        where T == Int

    StringLiteral(String)
        where T == String
}
```

Construction of a variant requires its constructor constraints to be satisfiable.

Pattern matching on a successfully recognized GADT constructor may introduce the constructor's proven equalities and evidence into the corresponding branch environment according to the GADT/pattern-matching specification.

Thus a variant-specific equality such as:

```phalcom
where T == Int
```

may refine `T` within the matching branch.

A variant constructor may also carry conformance or associated-type constraints where semantically meaningful:

```phalcom
Case<U>(U)
    where
        U |= Printable,
        <U as SomeTrait>::Item == T,
```

provided all referenced projections are well-formed.

This document specifies the meaning of those constraints. The details of branch refinement and exhaustiveness remain owned by the GADT and pattern specifications.

## Lexical composition of constraint environments

Constraint environments compose lexically.

For example:

```phalcom
class Box<T>
    where T |= Printable
{
    method<U>(_ value: U)
        where U |= Hashable
    {
        ...
    }
}
```

The method body is checked under both:

```text
T |= Printable
U |= Hashable
```

The inner declaration adds constraints to the outer environment.

It does not erase outer assumptions.

If an inner constraint set is provably contradictory with the inherited environment, the inner declaration is invalid according to ordinary constraint-satisfiability rules.

## `Self` in constraints

`Self` may appear in a `where` clause wherever the enclosing declaration already gives `Self` a semantic meaning.

For example:

```phalcom
where Self |= SomeTrait
```

or:

```phalcom
where
    <Self as Iterator>::Item == T
```

may be valid inside an appropriate trait or conformance context.

`where` does not itself introduce `Self`.

If the enclosing declaration has no `Self` binding, use of `Self` is invalid.

The semantics of `Self` remain those defined by the enclosing declaration model.

## Supertraits

This specification does not introduce supertraits.

If a future supertrait feature establishes transitive conformance evidence, that specification must define how such evidence contributes to:

- contextual associated projection lookup;
- trait-member availability;
- generic proof closure;
- ambiguity detection.

Until then, contextual projection lookup considers evidence actually established by the current language's conformance mechanisms.

No implicit supertrait behavior is assumed by this document.

## Source identity and canonical semantic identity

Generic constraints are resolved to canonical semantic identities.

For:

```phalcom
T |= Iterator<Item = A>
```

the compiler resolves:

```text
Iterator
    -> canonical trait declaration / application

Item
    -> canonical associated-type requirement owned by Iterator
```

The implementation must not represent associated specialization solely by textual name.

Conceptually:

```text
AssociatedEquivalent {
    subject: T,
    trait: Iterator,
    requirement: Iterator::Item,
    expected: A,
}
```

is the semantic relation.

Two unrelated traits may each declare `Item` without collision.

Likewise:

```phalcom
<T as Iterator>::Item
```

is identified by:

```text
subject
canonical trait application
canonical associated requirement
```

rather than by source spelling alone.

This identity model is required for:

- coherence;
- exact evidence;
- generic evidence;
- projection normalization;
- diagnostics;
- source indexing;
- incremental dependency tracking;
- reflection.

## Source-order independence

Constraint semantics are source-order independent.

These are equivalent:

```phalcom
where
    F: (Type) -> Type,
    F |= Functor,
    T |= Printable,
```

and:

```phalcom
where
    T |= Printable,
    F |= Functor,
    F: (Type) -> Type,
```

A compiler may process kinds, formation, equivalence, conformance, and subtype obligations in an implementation-dependent order.

That order must not make the source order semantically significant.

Likewise, associated declarations and generic evidence must not acquire meaning from incidental ordering within the `where` clause.

## Diagnostics

The compiler should preserve distinct diagnostic categories for semantically distinct failures.

Important categories include:

- unknown kind;
- incompatible kind;
- conflicting inferred kinds;
- unknown trait;
- invalid trait application;
- trait subject kind mismatch;
- missing conformance;
- ambiguous conformance;
- unknown associated specialization name;
- duplicate associated specialization;
- ambiguous contextual projection;
- invalid qualified projection;
- missing conformance required by a qualified projection;
- associated-type mismatch;
- unsatisfied type equivalence;
- subtype constraint failure;
- unsatisfiable constraint set;
- underdetermined generic inference;
- recursive obligation;
- dynamic-boundary failure;
- blocked proof;
- budget exhaustion;
- cancellation.

Exact diagnostic codes and wording are implementation-defined unless specified elsewhere.

### Likely-syntax diagnostics

When the source strongly suggests a mistaken relation, diagnostics should explain the correct relation.

For example:

```phalcom
where T: Iterator
```

is invalid when `Iterator` is a trait rather than a kind.

A diagnostic should explain that `:` in `where` denotes kind classification and that trait conformance uses:

```phalcom
T |= Iterator
```

Likewise:

```phalcom
where T::Item = User
```

is invalid as a standalone equivalence constraint.

The correct form is:

```phalcom
where T::Item == User
```

Single `=` remains valid only inside named associated specialization:

```phalcom
where T |= Iterator<Item = User>
```

Inside associated specialization, the converse mistake should also be diagnosed:

```phalcom
I |= Iterator<Item == User>
```

should suggest:

```phalcom
I |= Iterator<Item = User>
```

## Formatting

Formatting is non-semantic.

The formatter should prefer readable structure while retaining short clauses inline where appropriate.

A concise clause may remain:

```phalcom
print<T>(_ value: T) where T |= Printable {
    ...
}
```

A substantial clause should normally be formatted below the signature:

```phalcom
traverse<F, G, A, B>(
    _ value: F<A>,
    _ transform: (A) -> G<B>,
) -> G<F<B>>
    where
        F: (Type) -> Type,
        G: (Type) -> Type,
        F |= Traversable<Item = A> + Functor,
        G |= Applicative,
{
    ...
}
```

A long conjunction may use continuation alignment:

```phalcom
where
    T |= Printable
       + Hashable
       + Serializable
       + ShowDebug,
```

A long associated specialization may use multiline angle-bracket formatting:

```phalcom
where
    I |= Iterator<
        Cursor = Int,
        Item = User,
    >,
```

The formatter may canonicalize accepted kind spelling toward callable-shaped forms such as:

```phalcom
(Type, Type) -> Type
```

## Integrated example: higher-kinded traversal

```phalcom
traverse<F, G, A, B>(
    _ value: F<A>,
    _ transform: (A) -> G<B>,
) -> G<F<B>>
    where
        F: (Type) -> Type,
        G: (Type) -> Type,

        F |= Traversable<Item = A>
           + Functor
           + ShowDebug,

        G |= Applicative
           + Functor,
{
    ...
}
```

This establishes:

```text
F : (Type) -> Type
G : (Type) -> Type

F |= Traversable
<F as Traversable>::Item == A
F |= Functor
F |= ShowDebug

G |= Applicative
G |= Functor
```

The generic parameter list introduces only:

```text
F, G, A, B
```

The `where` clause establishes every nontrivial fact about them.

## Integrated example: explicit associated projection equality

Suppose:

```phalcom
trait Iterator {
    type Item
}

trait Collection {
    type Item
}
```

A generic declaration may directly state that the two associated types coincide:

```phalcom
transfer<T>(_ value: T)
    where
        <T as Iterator>::Item
            == <T as Collection>::Item
{
    ...
}
```

The qualified projections create the well-formedness requirements:

```text
T |= Iterator
T |= Collection
```

and then assert:

```text
<T as Iterator>::Item
    ==
<T as Collection>::Item
```

No ambiguity exists because each projection explicitly identifies its owning trait application.

## Integrated example: subtype interval and solver-producing equality

```phalcom
process<T, U>(_ value: T) -> U
    where
        Entity <: T <: DomainObject,
        U == Result<T, Error>,
{
    ...
}
```

The declaration establishes:

```text
Entity <: T
T <: DomainObject
U == Result<T, Error>
```

If a call determines `T`, the equality may solve `U`.

If a call determines `U = Result<User, Error>`, canonical structural equivalence may constrain `T = User`, after which the subtype interval verifies:

```text
Entity <: User <: DomainObject
```

If neither side is otherwise constrained, the compiler must not invent a satisfying `T` and `U`.

## Integrated example: conditional conformance

```phalcom
impl<I> Serializable for Stream<I>
    where
        I |= Iterator<Item = Byte>
           + Resettable,
{
    ...
}
```

Proof of:

```text
Stream<UserIterator> |= Serializable
```

generates obligations equivalent to:

```text
UserIterator |= Iterator
<UserIterator as Iterator>::Item == Byte
UserIterator |= Resettable
```

The outer conformance is valid only if all nested obligations are proven.

## Integrated example: generic alias

```phalcom
type Collected<I> = List<<I as Iterator>::Item>
    where I |= Iterator
```

Use of:

```phalcom
Collected<UserIterator>
```

requires:

```text
UserIterator |= Iterator
```

and the alias expands semantically to:

```text
List<<UserIterator as Iterator>::Item>
```

with ordinary associated projection normalization.

## Integrated example: GADT constructor constraints

```phalcom
enum Expr<T> {
    IntLiteral(Int)
        where T == Int

    StringLiteral(String)
        where T == String

    Mapped<F, U>(F<U>)
        where
            F: (Type) -> Type,
            F |= Functor,
            T == F<U>,
}
```

Each constructor carries its own propositions.

Construction must satisfy them.

A successful pattern match may refine the branch's generic environment using the constructor-specific evidence according to the GADT semantics.

## Invalid examples

Trait conformance does not use `:`:

```phalcom
where T: Printable
```

when `Printable` is a trait.

Use:

```phalcom
where T |= Printable
```

A standalone equality does not use single `=`:

```phalcom
where T::Item = User
```

Use:

```phalcom
where T::Item == User
```

Associated specialization does use single `=`:

```phalcom
where T |= Iterator<Item = User>
```

Duplicate associated specialization is invalid:

```phalcom
where T |= Iterator<Item = A, Item = B>
```

A `where` clause does not bind undeclared parameters:

```phalcom
foo<T>()
    where T == U
```

is invalid if `U` is not already in scope.

Subtype constraints do not apply to higher-kinded constructors merely because both sides have the same higher kind:

```phalcom
where
    F: (Type) -> Type,
    G: (Type) -> Type,
    F <: G,
```

is invalid under this specification.

A dynamic boundary cannot satisfy a static conformance requirement merely by being dynamic:

```phalcom
where Dynamic |= Iterator
```

does not establish static `Iterator` evidence.

## Deferred features

This specification intentionally does not define:

- supertraits;
- existential trait types;
- trait objects;
- general intersection types;
- a general type-level meaning for `A + B`;
- higher-kinded subtyping;
- user-defined constraint operators;
- runtime dynamic conformance checks;
- automatic existential search for a type satisfying a bound;
- partial application of type constructors unless independently specified;
- any separate labeled generic-argument system.

Future features may integrate with the generic constraint language, but they must preserve the distinctions established here unless this specification is explicitly amended.

## Relationship to neighboring specifications

This document owns:

```text
where-clause syntax
constraint categories
surface elaboration into canonical obligations
generic conformance assumptions
associated specialization inside trait bounds
qualified projection use inside constraints
constraint composition
generic-body proof assumptions
instantiation-time obligation semantics
```

The kind specification owns:

```text
kind representation
kind formation
kind compatibility
canonical kind identity
```

The subtype specification owns:

```text
the semantic meaning of A <: B
variance
nominal subtype rules
union subtype rules
bottom/top behavior
```

The type-equivalence specification owns:

```text
canonical semantic equivalence used by ==
normalization laws
union equivalence
alias transparency
structural canonicalization
```

The trait/conformance specification owns:

```text
trait identity
applied trait identity
coherence
conformance evidence
witness resolution
conditional conformance candidate selection
```

The associated-type specification owns:

```text
associated-type declarations
associated requirement identity
associated bindings
projection identity
projection normalization
```

The generic-inference and implementation-internals specifications own:

```text
solver data structures
worklist algorithms
union-find or equivalent implementation
query caching
incremental invalidation
cycle-detection algorithm
budget management
cancellation machinery
```

Those implementations must nevertheless preserve the observable semantics required by this document.

## Normative summary

The generic parameter list introduces variables:

```phalcom
<T, U, F>
```

Nontrivial facts belong in `where`.

Kinds use `:`:

```phalcom
F: (Type) -> Type
```

Trait conformance uses `|=`:

```phalcom
T |= Printable
```

Same-subject trait conjunction uses `+`:

```phalcom
T |= Printable + Hashable
```

Associated specialization uses single `=` inside the constrained trait application:

```phalcom
I |= Iterator<Item = User>
```

General type equivalence uses `==`:

```phalcom
I::Item == O::Input
```

Subtyping uses `<:`:

```phalcom
Lower <: T <: Upper
```

Contextual associated projection uses:

```phalcom
T::Item
```

when the owning associated requirement is uniquely determined.

Explicit qualified projection uses:

```phalcom
<T as Iterator>::Item
```

when trait identity must be stated directly.

General semantic equivalence is canonical and normalization-aware:

```phalcom
A | B == B | A
```

is true.

Equivalence constraints participate in inference but never choose arbitrary solutions from an underdetermined solution family.

Subtype constraints restrict inference but never arbitrarily choose a point in a subtype interval.

Any well-kinded type-level expression may be a conformance subject when its kind is compatible with the referenced trait.

Any well-formed proper type expression may participate in `<:`.

`Dynamic` never acts as automatic proof of a static generic bound.

Generic aliases and GADT variant constructors may carry `where` clauses.

Constraint environments compose lexically.

`where` never introduces implicit generic parameters.

Surface syntax elaborates into canonical semantic obligations, and all proof machinery operates over those obligations rather than over syntax-specific special cases.
