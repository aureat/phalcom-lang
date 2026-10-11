# Generic Constraints

**Status:** Normative  
**Language area:** Type system, generics, trait conformance  
**Implementation checkpoint:** LANG005.C6  
**Specification role:** Authoritative specification of Phalcom generic constraints

A generic declaration introduces abstract type-level entities. A `where` clause states propositions that must hold about them.

> **Core rule:** generic code is checked under its declared constraints; every concrete specialization or invocation must satisfy the corresponding obligations.

This specification defines constraint syntax and semantics. Solver algorithms, evidence representation, bytecode/runtime transport, caching, and incremental mechanics are non-normative.

---

## 1. Language model

Phalcom has **no source-level free functions**. Generic executable behavior is method-based.

`where` clauses may occur on declaration forms that admit generic contracts, including:

- classes, data declarations, enums, and traits;
- inherent and conformance `impl` declarations;
- type aliases;
- methods and other generic behavior members;
- variant constructors where the ADT/GADT grammar permits constructor-specific constraints.

Example:

```phalcom
trait Printable {
  print -> String
}

class Renderer {
  render<T>(_ value: T) -> String
      where T |= Printable
  {
    value.print
  }
}
```

This specification does not introduce any free-function declaration form.

---

## 2. Relations

Phalcom generic constraints use four distinct relations:

| Syntax | Meaning |
|---|---|
| `:` | generic binder-domain classification; kind constraint in `where` |
| `|=` | trait conformance |
| `<:` | subtyping |
| `==` | canonical type equivalence |

Supporting syntax:

| Syntax | Meaning |
|---|---|
| `+` | conjunction of trait requirements on one subject |
| `=` | associated-type specialization inside a trait application |
| `::` | type-member lookup/projection; associated-type projection is one specialization |

They must not be conflated. In particular:

- conformance is neither subtyping nor structural method matching;
- kinding is not conformance;
- `==` is neither assignment nor ordinary runtime equality;
- `+` creates no intersection type or synthetic trait;
- `Dynamic` is not automatic proof of a static constraint.

Constraints in one `where` clause are conjunctive and source-order independent.

---

## 3. Generic parameters and `where`

Generic parameter lists bind parameters:

```phalcom
<T>
<T, U>
<F, A, B>
```

Where the declaration form permits parameter-domain annotations, a binder may
classify a parameter directly:

```phalcom
<F: Type -> Type, P: PositionalPack<Type>, I: Int>
```

`F` is a type-level callable-domain parameter. `P` is a static Pack of `Type` values.
`I` is a compile-time value parameter in the `Int` domain; `Int` is not treated as a
kind merely because the value is compile time. The complete domain model is owned by
[Type-Level Domains](type-level-domains.md).

Trait, subtype, and equivalence bounds belong in `where`:

```phalcom
class Box<T>
    where T |= Printable
{
  ...
}
```

not:

```phalcom
class Box<T: Printable> {
  ...
}
```

Explicit kind constraints may also be written in `where`:

```phalcom
where F: Type -> Type
```

If a declaration form also admits a kind annotation directly on a generic binder, both spellings contribute to the same parameter-kind contract and must agree.

`where` constrains already-bound or otherwise-resolvable names. It never
introduces an implicit generic parameter. Binder annotations establish a
parameter's domain; `where` remains the proposition/obligation language and
does not need a redundant `where I: Int` restatement for a const parameter.

Line breaks are non-semantic.

---

## 4. Grammar

The exact grammars of `TypeExpression`, `TypeLevelExpression`, `ProperTypeExpression`, `TraitReference`, and `Kind` are owned by their respective specifications.

```text
WhereClause
    := "where" Constraint ("," Constraint)* [","]

Constraint
    := KindConstraint
     | ConformanceConstraint
     | EquivalenceConstraint
     | SubtypeChain

KindConstraint
    := GenericParameter ":" Kind

ConformanceConstraint
    := TypeLevelExpression "|=" TraitRequirementList

TraitRequirementList
    := TraitRequirement ("+" TraitRequirement)*

TraitRequirement
    := TraitReference
     | TraitReference "<" TraitArguments ">"

TraitArguments
    := OrdinaryTraitArguments ["," AssociatedSpecializations]
     | AssociatedSpecializations

OrdinaryTraitArguments
    := TypeExpression ("," TypeExpression)*

AssociatedSpecializations
    := AssociatedSpecialization ("," AssociatedSpecialization)*

AssociatedSpecialization
    := Identifier "=" TypeLevelExpression

EquivalenceConstraint
    := TypeLevelExpression "==" TypeLevelExpression

SubtypeChain
    := ProperTypeExpression ("<:" ProperTypeExpression)+
```

Ordinary trait arguments are positional. Once an associated specialization appears, remaining arguments must also be associated specializations.

Ordinary trait arguments remain type expressions in the initial surface.
Heterogeneous non-`Type` arguments are required initially for type-member
applications, not for general trait references. An associated specialization's
right-hand side must inhabit the associated declaration's result domain; an
unannotated associated declaration therefore still requires a proper type.

---

## 5. Parameter-domain constraints

On a generic binder, `:` classifies a generic parameter by domain. A kind
expression names a kinded domain, and a proper type may name a compile-time
value domain. In a `where` clause, the initial constraint grammar retains only
kind constraints over an already-bound parameter:

```phalcom
where F: Type -> Type
```

The equivalent binder annotation is `F: Type -> Type`. A compile-time value
parameter is instead declared on its binder:

```phalcom
<I: Int>
```

means that `I` is a static value in the `Int` domain, not that `Int` becomes a kind. `Int` remains a proper
type of kind `Type`. This specification does not introduce `where I: Int` as
a redundant restatement. Neither form states trait conformance.

```phalcom
where
    F: Type -> Type,
    F |= Functor,
```

contains two independent propositions.

An ordinary generic parameter defaults to the proper `Type` domain unless an explicit
annotation or expansion prefix supplies another static parameter domain.

For static algorithms over ordered Packs, expansion binders capture the corresponding
Pack lane:

```text
*Ts      -> positional Pack capture
**Ls     -> labeled Pack capture
***Es    -> complete Pack capture
```

The resulting public Type-valued families are `PositionalPack<Type>`,
`LabeledPack<Type>`, and `CompletePack<Type>`; they are proper Pack specializations
rather than independent TypePack kinds. The canonical aliases and their element-type
parameterization are owned by [Type-Level Domains](type-level-domains.md).

Expansion punctuation is also used by structural owner grammars. A Record field/tail
form such as `Record<**Fs>` is interpreted by the Record/unordered-product grammar and
does not become an ordered LabeledPack merely because it uses `**`.

The parameter's checked semantic domain therefore comes from the declaration/form that
owns the binder, not from punctuation alone. Unordered row correlation and public
reflection use the UnorderedShape/tail model from
[Structural Products](structural-products.md). `RecordFields` remains the
context-specific kinded binder domain for Record field expansions; it is not the public
unordered-reflection ontology.

A `where` kind constraint directly constrains a generic parameter. Compound type expressions are kind-checked through ordinary type formation.

The type-level-domain specification owns domain syntax, canonical domain
identity, application, partial application, and restricted compile-time terms.

---

## 6. Conformance constraints

Trait conformance uses `|=`:

```phalcom
where T |= Printable
```

Semantically:

```text
Conforms(T, Printable)
```

Conformance is nominal and evidence-based. Matching selectors or method shapes do not prove it.

The subject may be any well-formed type-level expression whose kind is compatible with the applied trait:

```phalcom
T |= Printable
List<T> |= Serializable
I::Item |= Hashable
```

Higher-kinded subjects are permitted when the trait subject kind admits them:

```phalcom
where
    F: Type -> Type,
    F |= Functor,
```

A generic conformance constraint supplies **assumed evidence** while checking the generic body. At a concrete specialization or invocation, the instantiated obligation must be discharged by valid evidence.

Assumed generic evidence is not a fabricated concrete `impl`, witness, or exact conformance identity.

### Guarded derived-conformance rules

A guarded generic-target conformance rule has the form:

```phalcom
impl<I> Iterable for I
    where I |= Iterator
{
  type Item = <I as Iterator>::Item
  type Iterator = I
  iterator { self }
}
```

The rule's `where` clause is an ordinary generic signature. The trait's own
generic contract is entailed by conformance evidence: `Self` is instantiated
from the concrete conformance subject, and the contract's constraints are
proved in the same evidence environment. The rule therefore supplies no
reverse search and does not make a subject conform merely because it has
members with matching names.

Rule application succeeds only after every antecedent obligation and the
consequent trait contract is proven. The resulting evidence preserves the
selected rule, its canonical substitutions, associated bindings, and nested
antecedent evidence. Cyclic or ambiguous rule applications are not successful
proofs and do not publish partial evidence.

`Dynamic` does not satisfy `T |= Trait` merely because runtime dispatch is possible.

---

## 7. Same-subject conjunction

`+` is shorthand for multiple conformance obligations on the same subject:

```phalcom
where T |= Printable + Hashable
```

is equivalent to:

```phalcom
where
    T |= Printable,
    T |= Hashable,
```

No combined trait is created. Operand order is irrelevant.

Associated specialization binds inside its immediate trait requirement:

```phalcom
where T |= Iterator<Item = A> + Printable
```

means:

```text
T |= Iterator<Item = A>
T |= Printable
```

A comma ends the same-subject conjunction.

---

## 8. Applied traits and associated specialization

Applied trait identity includes the trait declaration and ordinary generic arguments.

```phalcom
trait Converts<From, To> {
  type Error
  convert(_ value: From) -> Result<To, Self::Error>
}
```

`Converts<String, Int>` and `Converts<Bytes, Int>` are distinct trait applications.

Ordinary trait arguments are positional. `Name = Value` inside a constrained
trait application means associated-type specialization, with `Value` checked
against the associated declaration's result domain:

```phalcom
where C |= Converts<A, B, Error = E>
```

Here `A` and `B` are ordinary arguments and `Error = E` constrains `Converts::Error`.

A bound:

```phalcom
where I |= Iterator<Item = User>
```

states:

```text
I |= Iterator
<I as Iterator>::Item == User
```

Single `=` is used only for this structured specialization syntax. General equivalence uses `==`.

The same associated requirement may appear at most once in one trait application:

```phalcom
Iterator<Item = A, Item = B>   // invalid
Iterator<Item = A, Item = A>   // invalid
```

Associated specialization expresses exact equivalence only. Other relations remain top-level constraints:

```phalcom
where
    I |= Iterator,
    I::Item <: Entity,
```

---

## 9. Type-member projections

Generic constraints may use contextual or qualified type-member projections.
Trait-associated projections are one kind of type member. Domain-native
projections such as `P::Map<F>` are governed by [Type Members](type-members.md)
and [Structural Products](structural-products.md); they do not require trait
conformance evidence merely because they use `::`.

### Contextual

```phalcom
T::Item
Self::Item
```

A contextual projection is valid only when the available semantic evidence identifies exactly one applicable associated requirement with that name for the subject.

If both `Iterator` and `Collection` provide distinct `Item` requirements:

```phalcom
where T |= Iterator + Collection
```

then `T::Item` is ambiguous. Source order must not choose one.

### Qualified

```phalcom
<T as Iterator>::Item
<T as Converts<A, B>>::Error
```

A qualified projection identifies its subject, applied trait, and trait-owned associated requirement explicitly.

It is well-formed only if the qualifying conformance can be established. Writing the projection creates that well-formedness obligation; it does not make the conformance true.

Thus:

```phalcom
where
    <T as Iterator>::Item == <T as Collection>::Item
```

requires both qualifying conformances and then equality of the projected types.

Projection normalization uses the applicable semantic evidence and the member's
declared result domain. Trait-associated declaration identity and conformance
normalization remain owned by [Associated Types](associated-types.md); general
projection identity and symbolic/stuck/invalid states remain owned by [Type
Members](type-members.md).

---

## 10. Type equivalence

General type equivalence uses `==`:

```phalcom
where A == B
```

`A == B` holds when, after semantic normalization justified by the current environment, both operands denote the same canonical type.

The relation is symmetric and distinct from assignment, assignability, and gradual consistency.

Equivalence constraints participate in generic inference. Associated evidence may therefore determine a generic parameter:

```phalcom
class IteratorTools {
  first<I, T>(_ iterator: I) -> T
      where I |= Iterator<Item = T>
  {
    ...
  }
}
```

If concrete evidence establishes `<I as Iterator>::Item == User`, the constraint may determine `T = User`.

Equivalence propagates information only when the combined constraints determine it. It does not choose arbitrary substitutions from an underdetermined solution set.

`Dynamic` is not automatically equivalent to an arbitrary static type.

---

## 11. Subtype constraints

Subtyping uses `<:`:

```phalcom
where A <: B
```

The left operand is the subtype and the right operand the supertype.

Chains are pairwise shorthand:

```phalcom
where Lower <: T <: Upper
```

means:

```phalcom
where
    Lower <: T,
    T <: Upper,
```

Operands must be proper types of kind `Type`. This specification does not introduce higher-kinded subtyping.

Subtype constraints use Phalcom's ordinary subtype relation. They may constrain or validate inference, but they do not choose an arbitrary type from an underdetermined interval.

---

## 12. Lexical evidence and methods

Constraint environments compose lexically.

```phalcom
class Box<T>
    where T |= Printable
{
  render<U>(_ value: U) -> String
      where U |= Printable
  {
    value.print
  }
}
```

The method body is checked under both the owner's and the method's applicable constraints.

Because Phalcom is method-based, generic constraints compose with receiver specialization.

For:

```phalcom
class Box<T>
    where T |= Printable
{
  describe(_ value: T) -> String {
    value.print
  }
}
```

specializing a receiver as `Box<User>` may establish:

```text
T = User
```

but that type substitution does not itself prove:

```text
User |= Printable
```

Receiver/type specialization and conformance evidence are distinct.

A constrained generic method may forward assumed evidence when invoking another constrained generic method:

```phalcom
class Renderer {
  inner<T>(_ value: T) -> String
      where T |= Printable
  {
    value.print
  }

  outer<T>(_ value: T) -> String
      where T |= Printable
  {
    self.inner(value)
  }
}
```

`outer` may satisfy `inner`'s corresponding obligation with its existing assumed evidence; it need not invent an exact conformance for abstract `T`.

---

## 13. Instantiation and inference

At a concrete generic use, every declared constraint is instantiated under the selected type substitutions and must be satisfied.

Information may come from:

- receiver specialization;
- explicit generic arguments;
- argument types;
- expected result types;
- enclosing generic assumptions;
- exact conformance evidence;
- associated-type bindings;
- equivalence and subtype constraints.

Solver order is not source semantics.

A direct bound such as:

```phalcom
where T |= Printable
```

does not authorize the compiler to choose an arbitrary printable type for otherwise-undetermined `T`.

Associated specialization and equivalence may constrain inference once their subject is known:

```phalcom
where I |= Iterator<Item = T>
```

may determine `T` from `I`'s associated evidence.

The reverse does not authorize open-world existential search. Knowing `T = User` does not permit searching for an arbitrary `I` satisfying `I |= Iterator<Item = User>`.

### Contextual inference for empty list literals

An empty list literal contributes its known outer constructor, `List`, even
though it contributes no element-value evidence. When a generic parameter is
constrained by a canonical `List<T>` conformance and its associated binding
determines `T`, inference may use that binding to specialize the literal. For
example, an empty list passed to a parameter `I` with
`I |= Iterable, <I as Iterable>::Item == Float` may be inferred as
`List<Float>` when the canonical `List<T> |= Iterable` conformance binds
`Iterable::Item` to `T`.

This rule does not search for an arbitrary type satisfying the conformance
bound. The literal fixes the candidate constructor as `List`; only its element
argument is inferred from the uniquely applicable canonical conformance and
associated equality. The resulting call must carry the ordinary canonical
conformance evidence for `List<Float>`. If the context does not uniquely
determine the element type, the literal remains underconstrained. A bare
`Iterable` bound, without an item equality or other element evidence, does not
authorize choosing an element type.

---

## 14. Conditional conformance

A conformance may itself carry generic constraints:

```phalcom
impl<T> Printable for List<T>
    where T |= Printable
{
  ...
}
```

`List<T> |= Printable` is valid only when the nested obligation `T |= Printable` is satisfied.

Conditional conformances may depend on any constraints valid for the `impl`, including associated specialization, equivalence, and subtype constraints.

Trait candidate selection, overlap, and coherence remain governed by the trait/conformance specification. This specification introduces no "most specific wins" rule.

A recursive obligation cycle is not proof by itself and must not be treated as automatic success or `Dynamic`.

---

## 15. Canonicalization, duplicates, and satisfiability

Surface syntax denotes canonical propositions.

These are semantically equivalent:

```phalcom
where T |= A + B
```

```phalcom
where
    T |= B,
    T |= A,
```

Likewise, subtype chains are equivalent to their pairwise expansion.

Exact duplicate top-level constraints are redundant:

```phalcom
where
    T |= Printable,
    T |= Printable,
```

They create one semantic obligation, not two.

This differs from duplicate associated specialization inside one trait application, which is invalid.

Distinct applied traits are not duplicates:

```phalcom
where T |= Trait<A> + Trait<B>
```

when the two trait applications are distinct.

A generic declaration is invalid when its constraint set is statically provably inconsistent. Obligations that depend on later substitutions may remain conditional until specialization.

Canonical meaning is independent of source ordering, `+` grouping, and duplicate top-level obligations.

---

## 16. Declaration-specific use

### Type aliases

A constrained alias retains its generic contract:

```phalcom
type Items<I> = List<<I as Iterator>::Item>
    where I |= Iterator
```

Using the alias requires its instantiated constraints to hold. Alias expansion does not discard them.

### Variant constructors and GADTs

Where the ADT/GADT grammar permits constructor-specific generic parameters or `where` clauses, those constraints belong to the constructor's contract.

Construction must satisfy them. A successful pattern match may expose proven constructor equalities or evidence to the matching branch according to the GADT/pattern specification.

### `Self`

`Self` may occur in a constraint only where the enclosing declaration already defines it:

```phalcom
where Self |= SomeTrait
```

```phalcom
where <Self as Iterator>::Item == T
```

`where` never introduces `Self`.

---

## 17. Diagnostics

Diagnostics must preserve important semantic distinctions, especially:

- kind mismatch vs conformance failure;
- missing vs ambiguous conformance;
- unknown vs ambiguous associated projection;
- duplicate associated specialization vs unsatisfied equality;
- underdetermined inference vs refuted constraint;
- dynamic boundary vs static proof.

Clear operator mistakes should identify the proper relation:

```phalcom
where T: Printable
```

is invalid when `Printable` is a trait; use `T |= Printable`.

```phalcom
where T::Item = User
```

is invalid as standalone equivalence; use `T::Item == User`.

```phalcom
where T |= Iterator<Item == User>
```

is invalid specialization syntax; use `Item = User`.

---

## 18. Non-features and ownership

This specification does not define:

- source-level free functions;
- supertraits;
- trait objects or existential trait types;
- general intersection types or a general type-level meaning for `+`;
- higher-kinded subtyping;
- user-defined constraint operators;
- open-world existential search for a type satisfying a bound;
- automatic runtime conformance checks for `Dynamic`;
- arbitrary user-defined type families or arbitrary compile-time execution;
- general nominal or trait const-generics beyond the initial type-member and
  structural-operation surface.

Neighboring specifications remain authoritative for:

- generic parameter binding, kinds, and type formation;
- type-level domains, packs, rows, compile-time value terms, and domain
  classification;
- general type-member declaration, lookup, application, and projection
  normalization;
- trait identity, conformance, witnesses, defaults, and coherence;
- trait-associated ownership, conformance bindings, and evidence-based
  normalization;
- subtyping and canonical type equivalence;
- ADT/GADT syntax and branch refinement;
- method selectors, receiver semantics, method references, and invocation.

Implementation specifications own solver architecture, evidence storage and transport, runtime environments, caching, incremental invalidation, and cycle-detection mechanics.

---

## 19. Normative summary

```text
F: Type -> Type
    parameter-domain classification as Kind(Type -> Type)

I: Int
    binder-domain classification as Const(Int), not kinding by Int

T |= Printable
    nominal trait-conformance obligation

T |= A + B
    two independent conformance obligations on T

I |= Iterator<Item = User>
    I |= Iterator
    AND
    <I as Iterator>::Item == User

T::Item
    contextual associated projection; requires a unique applicable requirement

<T as Iterator>::Item
    explicitly qualified associated projection

A == B
    canonical semantic type equivalence

A <: B
    subtype relation

Lower <: T <: Upper
    pairwise subtype constraints
```

The governing invariants are:

1. Generic executable behavior is method-based; generic constraints do not imply free functions.
2. `where` states proof-bearing propositions over already-bound type-level entities.
3. Conformance is nominal and evidence-based.
4. Assumed generic evidence and exact concrete evidence are distinct.
5. Receiver specialization supplies type substitution, not conformance proof.
6. Owner and method constraints compose lexically.
7. `+` creates independent same-subject obligations, not a synthetic trait.
8. Associated specialization is trait-anchored exact equivalence and uses single `=` only inside the trait application.
9. Contextual projection requires unique associated identity; qualification names that identity explicitly.
10. `==` and `<:` may constrain inference but never select arbitrary solutions.
11. Direct conformance bounds do not invent otherwise-undetermined type arguments.
12. Conditional conformance requires all nested obligations.
13. `Dynamic` is not automatic proof of a static relation.
14. Constraint meaning is canonical and source-order independent.
