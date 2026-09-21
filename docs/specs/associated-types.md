# Associated Types

**Status:** Normative  
**Language area:** Traits, conformances, type projection  
**Implementation checkpoint:** LANG005.C5, extended by LANG005.C6 generic evidence  
**Specification role:** Authoritative language specification for associated type declarations, bindings, projections, and normalization

Associated types are trait-owned type requirements whose concrete meaning is supplied by a conformance.

```phalcom
trait Iterator {
  type Item
  next -> Option<Self::Item>
}
```

A conformance supplies a binding for the associated declaration:

```phalcom
class SequenceIterator<T> {}

impl<T> Iterator for SequenceIterator<T> {
  type Item = T

  next -> Option<Self::Item> {
    ...
  }
}
```

For the exact conformance of `SequenceIterator<Int>` to `Iterator`, the associated value of `Iterator::Item` is `Int`.

The governing model is:

```text
trait declaration
    owns associated declaration

conformance
    supplies associated binding

projection
    refers to the trait-owned declaration through a subject and trait context

normalization
    uses valid conformance evidence to determine the associated value
```

> **Core invariant**
>
> An associated type is a trait-owned semantic declaration. A conformance supplies its value. A projection refers to that declaration through a subject and trait context; it is never identified by a written name alone.

---

## 1. Scope

This specification defines:

- associated type declarations in traits;
- associated type bindings in trait conformances;
- associated declaration identity;
- associated-binding completeness;
- contextual `Self::Item` projection;
- symbolic associated projections;
- projection identity;
- source-conformance and exact-conformance normalization;
- sibling associated-binding dependencies;
- recursive associated bindings;
- specialization of behavioral requirements through associated values;
- interaction with trait defaults and witnesses;
- exact enum-case subjects;
- the semantic boundary between associated types and generic constraints.

This specification does not define:

- the general generic constraint grammar;
- the `|=` conformance relation;
- generic-subject projection syntax such as `T::Item`;
- explicit trait-qualified projection syntax such as `<T as Iterator>::Item`;
- associated-type equality constraints;
- associated specialization inside generic trait bounds;
- conditional-conformance proof search;
- associated type defaults;
- the general type-member declaration/projection substrate, which is owned by
  [Type Members](type-members.md);
- trait objects or existential trait packaging;
- public reflection descriptors for associated declarations;
- runtime associated-type lookup.

Generic-subject projections, explicit qualification, associated equality, and associated specialization in generic bounds are governed by [Generic Constraints](generic-constraints.md). Trait and conformance semantics are governed by [Traits](objects/traits.md) and [Implementation Declarations](objects/impl.md).

---

## 2. Semantic categories

Associated types introduce three distinct semantic categories.

### 2.1 Associated declaration

A trait member of the form:

```phalcom
trait Container {
  type Item
}
```

declares one associated type requirement owned by `Container`.

`Container::Item` is not:

- an instance field;
- a getter;
- a method;
- an ordinary generic parameter;
- a nested nominal type;
- a runtime property.

It is a trait-owned type requirement.

### 2.2 Associated binding

A conformance member of the form:

```phalcom
impl<T> Container for Box<T> {
  type Item = T
}
```

binds `Container::Item` for that conformance.

The binding belongs to the conformance. It does not mutate the trait declaration and does not create an inherent member named `Item` on `Box<T>`.

### 2.3 Associated projection

A projection denotes the associated value of one associated declaration for one subject under one trait context.

The associated specialization of the contextual projection form is:

```phalcom
Self::Item
```

A projection is an ordinary type-level expression in its declaration's result
domain. It may remain symbolic or normalize depending on the evidence available
in its semantic context. General projection identity and states are owned by
[Type Members](type-members.md); this specification supplies the trait context
and conformance evidence for associated members.

> **Category-separation invariant**
>
> Associated declarations, associated bindings, and associated projections are distinct semantic products even when they refer to the same trait requirement.

---

## 3. Declaration identity

Associated declaration identity is nominal and trait-owned.

```phalcom
trait Collection {
  type Item
}

trait Stream {
  type Item
}
```

`Collection::Item` and `Stream::Item` are different declarations.

Their shared spelling does not merge them:

```text
Collection::Item ≠ Stream::Item
```

Canonical identity therefore includes the owning trait and the associated declaration within that trait.

It does not derive from:

- the name `Item` alone;
- a source range;
- declaration order alone;
- the type to which a particular conformance happens to bind the declaration.

Two traits may independently declare associated types with the same name. One trait may not declare the same associated name twice.

```phalcom
trait Invalid {
  type Item
  type Item
}
```

is invalid.

> **Identity invariant**
>
> Associated type identity is declaration identity, not name identity.

---

## 4. Associated declaration result domains

An associated declaration has a declared result domain under the general
type-member model. Existing unannotated declarations default to the proper
type domain `Type` and therefore denote one proper type per exact valid
conformance. The initial associated-type surface may declare an explicit
result domain but does not bind member-local parameters. Associated declarations
remain trait-owned requirements, not inherent or domain-native members.

```phalcom
trait Source {
  type Item
}
```

A binding for an associated declaration must form a value in its declared
result domain in the conformance's type environment. For the unannotated
`Item` form above, that remains a proper type.

Examples include:

```phalcom
type Item = Int
type Item = T
type Item = List<T>
type Item = (key: K, value: V)
type Item = Option<Self::Element>
```

The result-domain annotation uses the general rules in [Type Members](type-members.md).
Parameterized associated members remain outside the initial trait surface; this
category-specific restriction does not restrict parameterized inherent or
domain-native type members.

Ordinary trait generic parameters and associated types remain different mechanisms:

```phalcom
trait Converter<Target> {
  type Error
  convert -> Target
}
```

Here:

```text
Target
    ordinary trait generic argument
    selected as part of the trait reference

Error
    associated type
    supplied by conformance evidence
```

Associated types are not hidden trait generic parameters.

---

## 5. Trait member order

Trait member source order does not determine associated projection validity.

These declarations have the same contract:

```phalcom
trait A {
  type Item
  value -> Self::Item
}
```

```phalcom
trait B {
  value -> Self::Item
  type Item
}
```

The complete trait surface determines whether `Item` exists.

> **Trait-order invariant**
>
> Reordering an associated declaration relative to behavioral requirements that refer to it does not change the trait contract.

---

## 6. Associated bindings

A conformance binds an associated declaration with:

```phalcom
type Name = Type
```

For example:

```phalcom
trait Container {
  type Item
}

class IntBox {}
class StringBox {}

impl Container for IntBox {
  type Item = Int
}

impl Container for StringBox {
  type Item = String
}
```

Both conformances bind the same declaration, `Container::Item`, to different associated values.

### 6.1 Binding-name resolution

The left-hand side of a binding resolves only against associated declarations of the trait reference being conformed to.

```phalcom
trait Collection {
  type Item
}

trait Stream {
  type Item
}

class Buffer {}

impl Collection for Buffer {
  type Item = Byte
}
```

The binding satisfies `Collection::Item`. It does not search for another `Item` declaration elsewhere.

An unknown binding is invalid:

```phalcom
impl Collection for Buffer {
  type Element = Byte
}
```

when `Collection` declares no associated type named `Element`.

### 6.2 Binding scope

The right-hand side is formed in the ordinary lexical and generic environment of the conformance.

```phalcom
class Box<T> {}

impl<T> Container for Box<T> {
  type Item = T
}
```

The one source conformance yields specialized associated values under exact substitutions:

```text
Container for Box<Int>
    Item = Int

Container for Box<String>
    Item = String
```

This does not create multiple source conformance declarations.

> **Specialization invariant**
>
> One generic source conformance may produce different exact associated values while remaining one source conformance.

---

## 7. Conformance completeness

Associated requirements participate in the same conformance-validity decision as behavioral requirements.

```phalcom
trait PairLike {
  type Left
  type Right

  left -> Self::Left
  right -> Self::Right
}
```

This conformance is incomplete:

```phalcom
impl PairLike for PairBox {
  type Left = Int

  left -> Int { ... }
  right -> String { ... }
}
```

because `Right` has no binding.

At minimum, the following associated-binding failures are distinct:

- **missing binding** — a required associated declaration has no binding;
- **duplicate binding** — the same associated declaration is bound more than once in one conformance;
- **unknown binding** — a binding names no associated declaration of the conformed trait;
- **invalid binding** — the right-hand side is not valid in the associated declaration's result domain;
- **recursive binding** — the associated dependency graph contains a cycle.

A conformance is not valid unless all required associated bindings and all applicable behavioral obligations are valid.

> **Unified-completeness invariant**
>
> Associated and behavioral obligations form one conformance contract. Success in one lane cannot erase failure in the other.

---

## 8. Contextual projection with `Self`

This specification owns the contextual projection form:

```phalcom
Self::Item
```

It is valid only when the semantic context already establishes the owning trait reference.

The two primary contexts are:

1. the body of the owning trait;
2. a conformance implementation for that trait.

Inside a trait:

```phalcom
trait Collection {
  type Item
  first -> Self::Item
}
```

`Self::Item` denotes `Collection::Item` projected from abstract trait `Self`.

Inside a conformance:

```phalcom
class Box<T> {}

impl<T> Collection for Box<T> {
  type Item = T

  first -> Self::Item {
    ...
  }
}
```

`Self::Item` denotes the same trait-owned declaration projected from the conformance subject.

An unrelated inherent implementation does not acquire an associated projection context merely because the target conforms to a trait elsewhere.

> **Context invariant**
>
> Contextual projection resolves through an already-established trait or conformance context. It does not search globally for a same-named associated declaration.

---

## 9. Projection identity

A projection is not identified by:

```text
subject + associated name
```

The general projection identity from [Type Members](type-members.md), specialized
for an associated declaration, contains conceptually:

```text
subject type
trait reference / trait context
associated declaration identity
```

This matters when multiple trait applications or multiple traits expose the same associated name.

```phalcom
trait A {
  type Item
}

trait B {
  type Item
}
```

A projection of `A::Item` and a projection of `B::Item` from the same subject remain different projections unless language rules independently prove an equivalence between their resulting types.

For generic traits, the trait application is part of the context:

```phalcom
trait Converter<Target> {
  type Error
}
```

`Converter<Int>` and `Converter<String>` are different trait references. Projection identity therefore retains the relevant trait application, not only the bare `Converter` declaration.

> **Projection-identity invariant**
>
> Canonical associated projection identity includes subject, trait context, and associated declaration identity.

---

## 10. Formation and normalization are different operations

Projection formation constructs the semantic projection denoted by source.

Projection normalization determines whether that projection can be reduced using valid associated evidence.

Inside an abstract trait:

```phalcom
trait Iterator {
  type Item
  next -> Option<Self::Item>
}
```

`Self::Item` is well formed even though no concrete binding is available. The projection remains symbolic.

It must not become:

```text
Dynamic
unknown type
error type
```

merely because the trait is abstract.

> **Symbolic-validity invariant**
>
> A well-formed abstract associated projection is a valid symbolic type.

---

## 11. Associated projections are ordinary type structure

Associated projections with result domain `Type` participate in the ordinary
canonical type graph. Associated projections with another declared domain
participate in that domain's canonical structure.

They may appear inside applied, tuple, record, callable, union, or other type structure:

```phalcom
Option<Self::Item>
List<Self::Item>
(Self::Item, Self::Item)
(key: Self::Key, value: Self::Value)
(Self::Item) -> Result<Self::Item>
```

Ordinary type operations may:

- preserve projections while rebuilding surrounding type structure;
- substitute constituents of projection subjects and trait arguments;
- compare canonically identical symbolic projections;
- normalize projections when the required evidence is available.

Ordinary structural substitution does not itself prove conformance.

For example, substituting `T = Int` into a projection whose subject is `Box<T>` may produce the corresponding projection over `Box<Int>`. Whether that projection reduces to an ordinary type is a separate evidence question.

> **Substitution boundary**
>
> Substitution rewrites type structure. Associated normalization establishes associated meaning.

Normalization preserves unrelated type structure. If:

```phalcom
trait EntrySource {
  type Key
  type Value

  entry -> (key: Self::Key, value: Self::Value)
}
```

has exact bindings:

```text
Key = String
Value = User
```

then the normalized result is:

```phalcom
(key: String, value: User)
```

not an unlabeled tuple.

---

## 12. Normalization authority

Associated normalization has three principal semantic contexts.

### 12.1 Abstract trait context

In an abstract trait contract, associated projections remain symbolic unless another already-established proposition determines them.

```phalcom
trait IdentityCell {
  type Item

  identity(_ value: Self::Item) -> Self::Item {
    value
  }
}
```

The default can be type-checked symbolically. No concrete associated value is required merely to validate the identity operation.

A symbolic associated type has no capabilities beyond those established by the ordinary type and evidence environment.

### 12.2 Source-conformance context

While checking a source conformance, projections use that conformance's own binding plan.

```phalcom
trait Collection {
  type Item
  first -> Self::Item
}

class Box<T> {}

impl<T> Collection for Box<T> {
  type Item = T

  first -> Self::Item {
    ...
  }
}
```

Within this conformance, `Self::Item` may normalize to the source binding template `T`.

Source-conformance checking must not require already-published exact conformance evidence for the conformance currently being established.

Conceptually:

```text
resolve associated declarations
    ↓
establish source bindings
    ↓
normalize source projections
    ↓
check specialized behavioral contract
    ↓
publish exact evidence for valid exact applications
```

> **Source-normalization invariant**
>
> A conformance may use its own valid staged associated bindings while being checked; it does not depend circularly on final evidence that exists only after the conformance is accepted.

### 12.3 Exact-conformance context

When an exact subject and exact trait reference have valid exact conformance evidence, projection normalization consumes that evidence.

For:

```phalcom
impl<T> Container for Box<T> {
  type Item = T
}
```

exact normalization yields:

```text
Container for Box<Int>
    Item → Int

Container for Box<String>
    Item → String
```

Exact normalization does not rediscover conformance by scanning source declarations or matching associated names.

> **Exact-evidence invariant**
>
> Exact projection normalization consumes canonical evidence for the exact subject and trait reference.

---

## 13. Binding dependencies and recursion

Associated bindings may depend on sibling associated declarations.

```phalcom
trait Shape {
  type Element
  type Sequence
}

impl Shape for ByteShape {
  type Element = Byte
  type Sequence = List<Self::Element>
}
```

The resulting associated values are:

```text
Element = Byte
Sequence = List<Byte>
```

Binding semantics are source-order independent. Reversing those two binding declarations does not change their meaning.

Dependencies are tracked by associated declaration identity, not by spelling alone.

Recursive associated bindings are invalid.

Direct recursion:

```phalcom
trait Cyclic {
  type A
}

impl Cyclic for X {
  type A = Self::A
}
```

Indirect recursion:

```phalcom
trait Cyclic {
  type A
  type B
}

impl Cyclic for X {
  type A = Self::B
  type B = Self::A
}
```

Recursion remains invalid when hidden inside surrounding type structure:

```phalcom
trait Recursive {
  type Item
}

impl Recursive for X {
  type Item = List<Self::Item>
}
```

> **Cycle invariant**
>
> Associated values must have a finite semantic meaning. Cyclic associated definitions are invalid and never degrade to `Dynamic`.

---

## 14. Associated values specialize behavioral requirements

Trait behavioral requirements may mention associated projections.

```phalcom
trait Cell {
  type Item

  read -> Self::Item
  write(_ value: Self::Item) -> ()
}
```

For:

```phalcom
class IntCell {
  read -> Int { 42 }
  write(_ value: Int) -> () { () }
}

impl Cell for IntCell {
  type Item = Int
}
```

the exact behavioral requirement view is equivalent to:

```text
read -> Int
write(_ value: Int) -> ()
```

for witness compatibility and trait-evidenced use.

This specialization does not mutate the trait declaration. The abstract trait contract remains expressed in terms of `Self::Item`.

> **Contract-specialization invariant**
>
> Associated bindings specialize a conformance's requirement view; they do not rewrite the owning trait declaration.

### 14.1 Witness compatibility

A behavioral witness must be compatible with the associated-specialized requirement.

```phalcom
class BadCell {
  read -> String { "wrong" }
}

impl Cell for BadCell {
  type Item = Int
}
```

is invalid because `read -> String` does not satisfy the specialized requirement `read -> Int`.

### 14.2 Explicit conformance witnesses

A conformance-defined witness may itself use contextual projections:

```phalcom
class StoredCell {}

impl Cell for StoredCell {
  type Item = Int

  read -> Self::Item {
    42
  }

  write(_ value: Self::Item) -> () {
    ()
  }
}
```

The binding is associated evidence. The method/getter bodies are behavioral evidence. They remain distinct parts of one conformance.

### 14.3 Trait defaults

A trait default may be checked against symbolic associated projections:

```phalcom
trait IdentityCell {
  type Item

  identity(_ value: Self::Item) -> Self::Item {
    value
  }
}
```

For a conformance with `Item = String`, the selected default has the specialized contract:

```text
identity(_ value: String) -> String
```

Selecting a default does not convert it into an inherent member of the target.

---

## 15. Exact enum-case subjects

An exact enum case may be a conformance subject where the trait and enum specifications permit it.

```phalcom
enum Result<T> {
  Ok(_ value: T)
  Error(_ message: String)
}

trait PayloadType {
  type Value
}

impl PayloadType for Result<Int>::Ok(_) {
  type Value = Int
}
```

The binding belongs to the exact `Ok` case subject.

It does not widen to:

```text
Result<Int>
```

and does not apply to the sibling `Error` case.

> **Exact-case invariant**
>
> Exact-case subject identity survives conformance selection and associated normalization.

---

## 16. Relationship to generic constraints

This specification defines the associated-type semantic substrate. [Generic Constraints](generic-constraints.md) defines how generic code introduces evidence assumptions and how projections are written outside a lexical `Self` trait/conformance context.

That specification owns forms such as:

```phalcom
where T |= Iterator
```

```phalcom
T::Item
```

```phalcom
<T as Iterator>::Item
```

```phalcom
where T::Item == Element
```

and associated specialization inside a conformance requirement:

```phalcom
where T |= Iterator<Item = Element>
```

These forms do not introduce a second associated-type model.

They must resolve to the same semantic entities defined here:

```text
same trait-owned associated declaration
same projection identity
same conformance evidence model
same normalization semantics
```

A generic conformance assumption may justify a projection without an exact monomorphic source conformance being selected at generic-body checking time. The generic-constraint specification governs that assumed-evidence path.

This specification continues to govern what the associated declaration means and what value an exact conformance supplies.

> **Single-model invariant**
>
> Contextual `Self::Item`, generic `T::Item`, and qualified `<T as Trait>::Item` are different source contexts for one associated projection model, not separate type systems.

---

## 17. Type relations and `Dynamic`

A projection may participate in subtype, equivalence, assignability, and other ordinary type relations.

When a projection is reducible under the evidence available to that relation, the relation may normalize it before making a judgment that depends on its associated value.

Canonically identical symbolic projections may compare as the same symbolic type.

A projection that remains abstract must not be treated as an unrelated nominal type merely because it has not normalized.

`Dynamic` is not the recovery meaning of a failed projection.

A projection may normalize to `Dynamic` only when language semantics establish `Dynamic` as its associated value.

These facts are distinct:

```text
associated value is Dynamic
projection remains symbolic
conformance is absent
conformance is incomplete
conformance is ambiguous
associated binding is recursive
```

> **Dynamic distinction**
>
> Failure to establish an associated value does not imply `Dynamic`.

---

## 18. Static and runtime boundary

Associated type meaning is established by the static semantic system.

The runtime does not perform an ordinary execution-time operation equivalent to:

```text
lookupAssociatedType(receiver, trait, "Item")
```

to decide what an associated type means.

Executable lowering may transport already-established type or conformance evidence required by generic execution, but that transport does not redefine associated declarations or perform a second independent associated-type search.

Likewise:

- runtime class membership does not establish associated conformance;
- ordinary method lookup does not establish an associated binding;
- reflection does not create associated evidence;
- an associated name is never resolved by runtime string lookup as the semantic authority.

> **Static-authority invariant**
>
> Runtime execution may consume associated specialization and generic evidence, but associated type meaning is determined by the language's semantic conformance model.

---

## 19. Reflection boundary

This specification establishes the identities that any future reflection API must preserve:

```text
trait declaration
associated declaration
conformance
associated binding
projection
exact associated value
```

It does not prescribe the public reflection classes or APIs for exposing those identities.

A reflection system must not collapse an associated declaration into:

- its written name;
- a binding;
- the concrete type of one specialization;
- a behavioral member with the same spelling.

---

## 20. Deliberately unsupported forms

The following are not introduced solely by this trait/conformance specialization.

### 20.1 Associated type defaults

This is not defined:

```phalcom
trait T {
  type Item = Int
}
```

Every associated declaration defined here requires each valid conformance to establish its associated value.

### 20.2 Parameterized associated members

The general type-member model permits member-local parameters, but the initial
trait-associated surface remains unparameterized.

A form conceptually resembling:

```text
type Item<T: Type>: Type
```

This form requires a later amendment defining associated-family bindings. Such
parameters would belong to each projection application rather than becoming
hidden trait generic arguments. This restriction does not apply to
compiler/domain-native members such as structural `At<I>` and `Map<F>`.

### 20.3 Non-`Type` and higher-kinded result domains

An associated declaration may have an explicit result domain where its owning
grammar permits it. A result such as `TypePack` is not a proper type, and a
higher-kinded result is not implicitly saturated. The declaration's domain is
part of its identity and binding contract.

### 20.4 Structural associated types

A type does not acquire an associated value because it happens to expose:

- a same-named method;
- a same-named getter;
- a same-named field;
- a same-named nested declaration;
- a generic parameter with that spelling;
- a same-named associated type from an unrelated trait.

Associated values exist through trait declarations and conformance evidence.

### 20.5 Runtime associated namespaces

Ordinary member or associated lookup at runtime is not an associated-type resolution mechanism.

---

## 21. Invalid programs

A program is invalid when, among other ordinary trait/conformance failures:

- a trait declares the same associated name more than once;
- a conformance binds an associated name the trait does not declare;
- a conformance binds the same associated declaration more than once;
- a required associated declaration has no binding;
- an associated binding does not form a valid value in the declaration's
  result domain;
- associated bindings form a recursive dependency cycle;
- a behavioral witness is incompatible after associated specialization;
- an exact-case binding is incorrectly treated as a binding for the enum root or another exact case;
- generic projection syntax is used without the evidence required by the generic-constraint specification.

---

## 22. Canonical invariants

The following invariants summarize the associated-type model.

| Invariant | Requirement |
| --- | --- |
| **Trait ownership** | Associated declarations are owned by traits. |
| **Nominal identity** | Same spelling in different traits does not imply the same associated declaration. |
| **Conformance binding** | Concrete associated values are supplied by conformances. |
| **Category separation** | Declaration, binding, projection, and behavioral witness identities remain distinct. |
| **Declared result domain** | An associated declaration's result domain is part of its contract; unannotated declarations default to `Type`. |
| **Contextual `Self`** | `Self::Item` resolves through an established trait/conformance context, never global name search. |
| **Projection identity** | Subject, trait context, and associated declaration identity participate in canonical projection identity. |
| **Symbolic validity** | A well-formed abstract projection may remain symbolic without becoming `Dynamic` or an error. |
| **Evidence normalization** | Exact normalization consumes applicable conformance evidence. |
| **Source staging** | A conformance may use its own staged bindings without depending on final evidence for itself. |
| **Order independence** | Trait-member and binding source order do not determine associated meaning. |
| **Cycle rejection** | Recursive associated bindings are invalid. |
| **Structural preservation** | Normalization preserves unrelated surrounding type structure. |
| **Requirement specialization** | Associated values specialize behavioral contracts without mutating the abstract trait declaration. |
| **Exact-case preservation** | Exact enum-case subject identity survives associated normalization. |
| **Single projection model** | `Self::Item` and generic/qualified projection forms share one semantic model. |
| **Dynamic distinction** | Failure or uncertainty is not implicitly `Dynamic`. |
| **Static authority** | Runtime execution consumes associated meaning; it does not determine it. |

---

## 23. Related specifications

Associated types depend on, and deliberately delegate to, the following specifications:

- [Type Members](type-members.md) — general member parameters, result domains,
  projection identity, lookup, and symbolic/invalid normalization states;
- [Type-Level Domains](type-level-domains.md) — kinds, packs, rows, and
  compile-time parameter domains;
- [Generic Constraints](generic-constraints.md) — `where`, `|=`, generic evidence, generic-subject projections, qualified projections, associated equality, and associated specialization in trait bounds;
- [Traits](objects/traits.md) — trait requirements, conformance, witnesses, defaults, coherence, and evidence;
- [Implementation Declarations](objects/impl.md) — conformance declaration structure, generic scope, and implementation ownership;
- [Enumerated Types and Variants](objects/enums.md) — exact enum-case identity and exact-case conformance subjects;
- [Callables](callables/INDEX.md) — callable requirement and witness shape where associated values appear in behavioral signatures.

The semantic dependency is one-way:

```text
trait-associated declaration model
    ↓
conformance bindings
    ↓
canonical projections
    ↓
generic constraints / exact evidence establish normalization authority
    ↓
specialized ordinary type structure
```

No downstream subsystem should introduce a second associated-type identity or normalization model.
