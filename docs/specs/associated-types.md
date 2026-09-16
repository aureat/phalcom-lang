# Associated Types

Associated types are trait-owned type requirements whose concrete meaning is supplied by a conformance.

They allow a trait to describe type relationships that depend on the conforming subject without turning those relationships into ordinary generic arguments of the trait.

```phalcom
trait Iterator {
  type Item

  next -> Option<Self::Item>
}
```

A conformance supplies the concrete binding for the associated declaration:

```phalcom
impl<T> Iterator for SequenceIterator<T> {
  type Item = T

  next -> Option<Self::Item> {
    ...
  }
}
```

For an exact subject such as `SequenceIterator<Int>`, the `Iterator::Item` associated type normalizes to `Int`.

```text
Iterator::Item
    trait-owned associated declaration

Iterator for SequenceIterator<T>
    source conformance binding: Item = T

Iterator for SequenceIterator<Int>
    exact associated value: Item = Int
```

> **Core invariant**
>
> An associated type declaration is a stable part of a trait contract. A conformance supplies evidence for that declaration. Projection refers to the trait-owned declaration; it is never identified merely by a written name.

---

# 1. Scope

This specification defines:

- associated type declarations in traits;
- associated type bindings in conformance implementations;
- associated requirement identity;
- conformance completeness for associated bindings;
- contextual `Self::Item` projection;
- symbolic associated projections;
- source-conformance and exact-conformance normalization;
- projections nested inside ordinary type structure;
- dependencies among associated bindings;
- recursive associated bindings;
- specialization under generic conformances;
- interaction with trait requirements, defaults, witnesses, and exact enum-case conformances;
- source identity and navigation expectations;
- the semantic boundary between associated-type resolution and compiler/runtime execution.

This specification does **not** define:

- generic trait-conformance constraint syntax;
- associated-type equality constraint syntax;
- shorthand constraint syntax on trait references;
- generic evidence introduced by `where` clauses;
- associated type defaults;
- generic associated types;
- higher-kinded associated declarations;
- explicit qualified projection syntax outside the contextual form defined here;
- trait objects or existential trait types;
- reflection descriptors for associated declarations;
- a runtime associated-type lookup protocol.

Those facilities require separate specifications.

---

# 2. Terminology

## 2.1 Associated type declaration

A declaration owned by a trait:

```phalcom
trait Container {
  type Item
}
```

`Container::Item` is an associated type declaration.

It is a type requirement, not a callable requirement and not instance storage.

## 2.2 Associated type requirement

The semantic obligation introduced by an associated type declaration.

Every conformance to the owning trait must provide a valid binding unless another specification explicitly defines a different satisfaction rule.

In the language surface standardized here, there are no associated type defaults.

## 2.3 Associated type binding

A conformance member assigning a concrete type expression to one associated declaration:

```phalcom
impl<T> Container for Box<T> {
  type Item = T
}
```

The binding satisfies `Container::Item` for that conformance.

## 2.4 Projection

A type expression referring to an associated declaration through a subject and trait context.

The contextual source form standardized by this specification is:

```phalcom
Self::Item
```

A projection may remain symbolic or normalize to another type depending on semantic context.

## 2.5 Source conformance

The conformance declaration as written, before selecting one exact substitution for all of its generic parameters.

For example:

```phalcom
impl<T> Container for Box<T> {
  type Item = T
}
```

is one source conformance.

## 2.6 Exact conformance

A conformance relationship after the target, trait reference, and relevant generic environment have been made exact.

The source conformance above may contribute exact conformances such as:

```text
Container for Box<Int>
Container for Box<String>
```

while remaining one source implementation declaration.

## 2.7 Normalization

The semantic operation that attempts to replace associated projections by the type values established by the relevant conformance evidence.

Normalization is not ordinary name lookup and is not a runtime operation.

---

# 3. Associated Declarations

A trait may declare associated types with `type` members.

```phalcom
trait Reader {
  type Item
}
```

A trait may declare more than one associated type:

```phalcom
trait Iterable {
  type Item
  type Iterator
}
```

Each associated declaration is an independent trait-owned requirement.

Associated declarations do not create fields, slots, callable selectors, or runtime members.

```text
type Item
    associated type requirement

item -> T
    getter requirement

item(_)
    method requirement
```

These are different semantic member categories even when their written names resemble one another.

> **Category-separation invariant**
>
> Associated type requirements and behavioral requirements occupy separate semantic lanes. The presence, number, or ordering of associated declarations does not suppress, replace, or reinterpret behavioral requirements.

---

# 4. Proper-Type Requirement

The associated declaration form standardized by this specification denotes a proper type.

Conceptually, its kind is `Type`.

```phalcom
trait Source {
  type Item
}
```

A valid binding must therefore denote a proper type under the conformance's type environment.

Examples include:

```phalcom
type Item = Int
type Item = T
type Item = List<T>
type Item = (key: K, value: V)
type Item = Option<Self::Element>
```

This specification does not define an associated declaration whose value is itself a type constructor, a kind-polymorphic entity, or a generic associated family.

---

# 5. Declaration Identity

Associated type identity is nominal and trait-owned.

```phalcom
trait Collection {
  type Item
}

trait Stream {
  type Item
}
```

`Collection::Item` and `Stream::Item` are distinct associated declarations.

They remain distinct even though they have the same written name, the same kind, or happen to bind to the same concrete type for some target.

```text
Collection::Item ≠ Stream::Item
```

The spelling `Item` resolves the declaration inside an already-established trait context. The spelling is not global identity.

Source location is provenance, not identity.

> **Identity invariant**
>
> Associated type identity derives from the owning trait declaration and the associated declaration within that trait. Neither the written name alone nor a source range is sufficient identity.

---

# 6. Duplicate Associated Declarations

A trait cannot declare the same associated type requirement more than once under the same associated name.

```phalcom
trait Invalid {
  type Item
  type Item
}
```

This is invalid.

Two different traits may independently declare the same associated name.

---

# 7. Source Order of Trait Members

Trait member source order does not determine whether an associated projection is valid.

These contracts are semantically equivalent:

```phalcom
trait A {
  type Item
  value -> Self::Item
}
```

and:

```phalcom
trait B {
  value -> Self::Item
  type Item
}
```

Trait analysis must establish the complete associated declaration set before forming behavioral signatures that may refer to those declarations.

> **Trait-order invariant**
>
> Reordering an associated declaration relative to behavior that refers to it does not change the contract, except for ordinary source-location consequences.

---

# 8. Associated Bindings

A conformance binds associated declarations using `type Name = Type`.

```phalcom
trait Container {
  type Item
}

class Box<T> {}

impl<T> Container for Box<T> {
  type Item = T
}
```

The binding belongs to that conformance.

It does not create an inherent nested type on `Box`, create a global alias named `Item`, mutate the trait declaration, become a runtime property, or add storage.

The same trait may have different bindings in different conformances.

```phalcom
class IntBox {}
class StringBox {}

impl Container for IntBox {
  type Item = Int
}

impl Container for StringBox {
  type Item = String
}
```

The declaration `Container::Item` is the same requirement in both conformances; the bound values differ.

---

# 9. Binding Name Resolution

The left-hand side of an associated binding resolves only against the associated declarations of the exact trait being conformed to.

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

The binding above satisfies `Collection::Item`.

It does not consult a global registry of associated names and cannot accidentally bind `Stream::Item`.

An unknown binding name is invalid:

```phalcom
impl Collection for Buffer {
  type Element = Byte
}
```

because `Element` is not an associated declaration of `Collection`.

> **Binding-authority invariant**
>
> Associated binding resolution is owned by the referenced trait surface. There is no global associated-name resolver.

---

# 10. Binding Generic Scope

The right-hand side of an associated binding is formed in the ordinary generic scope of the conformance declaration.

```phalcom
trait Container {
  type Item
}

class Box<T> {}

impl<T> Container for Box<T> {
  type Item = T
}
```

For one source conformance, different exact targets produce different exact associated values:

```text
Box<Int>       → Container::Item = Int
Box<String>    → Container::Item = String
Box<User>      → Container::Item = User
```

These exact values do not create multiple source implementation identities.

> **Specialization invariant**
>
> One generic source conformance may produce different exact associated values under different exact substitutions while remaining one source implementation declaration.

---

# 11. Heterogeneous Conformance Bodies

A conformance body may contain associated bindings and behavioral witnesses together.

```phalcom
trait IterableLike {
  type Item
  type Iterator

  iterator -> Self::Iterator
}

class Sequence<T> {}
class SequenceIterator<T> {}

impl<T> IterableLike for Sequence<T> {
  type Item = T
  type Iterator = SequenceIterator<T>

  iterator -> Self::Iterator {
    SequenceIterator.new(...)
  }
}
```

The body contains three distinct obligations:

```text
associated binding: Item
associated binding: Iterator
behavioral witness: iterator
```

They coexist.

Associated bindings do not absorb the getter witness into an associated namespace. The getter witness remains ordinary conformance-owned behavioral evidence for the trait's getter requirement.

Likewise, the getter witness does not satisfy either associated requirement merely because its result type mentions one of them.

> **Heterogeneous-body invariant**
>
> Conformance member category is semantic. Associated bindings and behavioral witnesses may be interleaved in source without changing their categories or preventing either category from participating in conformance construction.

---

# 12. Source Order of Conformance Bindings

Associated binding semantics are source-order independent.

```phalcom
trait CollectionShape {
  type Element
  type Sequence
}

impl CollectionShape for Buffer {
  type Sequence = List<Self::Element>
  type Element = Byte
}
```

is equivalent to:

```phalcom
impl CollectionShape for Buffer {
  type Element = Byte
  type Sequence = List<Self::Element>
}
```

provided the same declarations and bindings are otherwise present.

The semantic model is conceptually staged:

```text
1. resolve all binding left-hand sides to trait-owned requirements
2. establish the supplied binding set
3. form binding right-hand sides
4. normalize sibling projection dependencies
5. detect recursive dependency cycles
6. publish normalized source binding templates or failures
```

An implementation may organize these phases differently if it preserves the same observable semantics.

---

# 13. Binding Completeness

A conformance is complete only if all required associated declarations have valid bindings and all behavioral obligations are satisfied.

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

because `Right` is missing.

Associated failures and behavioral failures contribute to one conformance validity decision. They do not establish independent partial conformances.

> **Unified-completeness invariant**
>
> A target does not conform unless the complete applicable trait contract is satisfied. Behavioral success cannot erase a fixed associated-binding failure, and valid associated bindings cannot erase a behavioral failure.

---

# 14. Binding Failure Categories

At minimum, conformance analysis distinguishes the following associated-binding failures.

## 14.1 Missing binding

A required associated declaration has no supplied binding.

## 14.2 Duplicate binding

The same associated requirement is bound more than once in one conformance.

## 14.3 Unknown binding

The conformance names an associated type the trait does not declare.

## 14.4 Invalid binding

The right-hand side fails ordinary type formation, kind requirements, scope rules, or associated normalization requirements.

An unresolved invalid type cannot become a valid associated binding merely because no behavior currently uses it.

---

# 15. Fixed Failure Preservation

A source conformance may contain behavioral choices whose applicability is finalized under an exact substitution.

Deferred behavioral work does not make fixed associated failures disappear.

```text
source conformance
    associated failure: Missing Item
    behavioral selection: deferred

exact specialization
    behavioral selection succeeds

result
    conformance remains invalid because Item is still missing
```

> **Failure-preservation invariant**
>
> Exact specialization may refine genuinely substitution-dependent uncertainty. It cannot convert a conformance with a fixed source associated-binding failure into proven conformance.

---

# 16. Projection Syntax

The contextual associated projection form standardized by this specification is:

```phalcom
Self::Item
```

It is a type expression.

```phalcom
trait Cell {
  type Item

  value -> Self::Item
  replace(_ value: Self::Item) -> Self::Item
}
```

`Self::Item` may appear wherever an ordinary proper type is accepted, subject to the contextual rules below.

This specification does not define arbitrary generic-subject projection syntax or explicit trait-qualified projection syntax.

---

# 17. Projection Formation Context

`Self::Item` is legal only when semantic analysis already has an owning trait context that identifies what `Item` means.

The canonical contexts are:

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
impl<T> Collection for Box<T> {
  type Item = T

  first -> Self::Item {
    ...
  }
}
```

`Self::Item` denotes the same trait-owned requirement, projected from the conformance target under its source generic environment.

A bare class or unrelated inherent implementation does not gain an associated projection context merely because some conformance might exist elsewhere.

> **Context invariant**
>
> Projection formation uses lexical semantic trait/conformance context. It does not search all traits in the program for an associated type with the same name.

---

# 18. Same-Name Associated Declarations Across Traits

Because formation is contextual, same-name associated declarations do not create ambiguity inside their owning trait bodies.

```phalcom
trait A {
  type Item
  value -> Self::Item
}

trait B {
  type Item
  value -> Self::Item
}
```

Inside `A`, `Self::Item` means `A::Item`.

Inside `B`, `Self::Item` means `B::Item`.

Likewise, inside an `A` conformance, `Self::Item` refers to `A::Item`.

Ambiguity in associated normalization is therefore a conformance/evidence ambiguity, not a global collision among associated names.

---

# 19. Projection Identity

A projection is not identified by:

```text
subject + written associated name
```

A canonical projection contains, conceptually:

```text
subject type
relevant trait reference / trait context
associated declaration identity
```

Two projections written with the same spelling may therefore be different because they refer to different trait-owned declarations.

Source ranges and spelling are diagnostic provenance only.

> **Projection-identity invariant**
>
> Canonical projection identity includes the subject, relevant trait application/context, and trait-owned associated declaration identity.

---

# 20. Formation Is Not Normalization

Projection **formation** creates the semantic type denoted by source syntax.

Projection **normalization** attempts to resolve that projection through associated binding evidence.

These are separate operations.

Inside an abstract trait:

```phalcom
trait Iterator {
  type Item
  next -> Option<Self::Item>
}
```

forming `Self::Item` is valid even though no concrete binding exists yet.

The resulting projection remains symbolic.

It must not be replaced with `Dynamic`, an unknown type, or an error merely because the trait is abstract.

> **Symbolic-validity invariant**
>
> A well-formed abstract associated projection is a valid symbolic type, not failed type inference.

---

# 21. Associated Projections in Ordinary Type Structure

An associated projection may appear recursively inside ordinary type forms.

Examples:

```phalcom
Option<Self::Item>
List<Self::Item>
(Self::Item, Self::Item)
(key: Self::Key, value: Self::Value)
(Self::Item) -> Result<Self::Item>
```

The projection does not require a parallel type system.

It participates in the ordinary canonical type graph.

Ordinary structural type operations may traverse it, substitute its constituents, preserve it in rebuilt compound types, and compare identical symbolic projections where appropriate.

Structural operations do **not** establish conformance evidence merely because they encounter a projection.

---

# 22. Preservation of Structural Detail

Projection normalization preserves unrelated structural information in the containing type.

```phalcom
trait EntrySource {
  type Key
  type Value

  entry -> (key: Self::Key, value: Self::Value)
}
```

If an exact conformance binds:

```text
Key = String
Value = User
```

the normalized result is:

```phalcom
(key: String, value: User)
```

not:

```phalcom
(String, User)
```

and not any structurally weakened form.

Associated normalization changes projected components. It does not erase tuple labels, callable parameter lanes, record field names, or other ordinary type structure.

---

# 23. Structural Substitution

Ordinary generic substitution may rewrite the constituents of a projection without solving it.

Suppose a canonical projection depends on a generic subject:

```text
subject: Box<T>
associated declaration: Container::Item
```

Substituting `T = Int` may structurally produce the corresponding projection over `Box<Int>`.

Whether that projection normalizes to `Int` is a separate conformance/evidence question.

> **Substitution boundary**
>
> Structural substitution rewrites type structure. Associated normalization proves associated meaning.

This separation prevents ordinary type substitution utilities from becoming hidden conformance solvers.

---

# 24. Normalization Contexts

Associated projection normalization has three semantically distinct contexts:

1. abstract trait normalization;
2. source-conformance normalization;
3. exact-conformance normalization.

They share projection identity and recursive type traversal but use different evidence authority.

---

# 25. Abstract Trait Normalization

Inside an abstract trait contract, an associated projection normally remains symbolic.

```phalcom
trait Iterator {
  type Item

  next -> Option<Self::Item>
}
```

There is no concrete target conformance from which to obtain `Item`.

The signature remains meaningful as:

```text
Option<the Iterator-associated Item of abstract Self>
```

Trait defaults may likewise use the symbolic type:

```phalcom
trait IdentityCell {
  type Item

  identity(_ value: Self::Item) -> Self::Item {
    value
  }
}
```

The default is type-checked against the abstract contract.

No concrete associated value is required merely to validate that `identity` returns the value it receives.

---

# 26. Capabilities of Abstract Associated Values

A symbolic associated type does not automatically possess arbitrary behavior.

```phalcom
trait Example {
  type Item

  render(_ value: Self::Item) -> String {
    value.toString
  }
}
```

This is valid only if the ordinary type/evidence rules independently establish that the operation performed on `Self::Item` is available.

The existence of an associated declaration does not imply inheritance from a particular class or conformance to arbitrary traits.

Constraints that establish such capabilities are specified separately.

---

# 27. Source-Conformance Normalization

While checking a source conformance, associated projections use that conformance's own staged binding plan.

```phalcom
trait Collection {
  type Item

  first -> Self::Item
}

impl<T> Collection for Box<T> {
  type Item = T

  first -> Self::Item {
    ...
  }
}
```

`Self::Item` in the conformance signature or body may normalize to the source binding template `T`.

This normalization must not require the final exact conformance evidence currently being constructed.

Otherwise conformance construction would depend on itself.

Conceptually:

```text
source conformance
    ↓
resolve associated binding templates
    ↓
normalize source projections
    ↓
check source witnesses/signatures
    ↓
construct exact evidence later
```

> **Source-normalization invariant**
>
> Source-conformance projection normalization consumes the source conformance's staged associated binding authority, not final evidence that depends on successful source checking.

---

# 28. Missing Sibling Bindings During Source Construction

During staged source construction, a projection referring to a sibling associated requirement may temporarily remain symbolic while the complete binding set is being established.

```phalcom
trait Shape {
  type Element
  type Sequence
}

impl Shape for X {
  type Sequence = List<Self::Element>
}
```

The source normalizer must not manufacture a concrete value for `Self::Element`.

The conformance completeness pass reports the missing `Element` binding.

This distinction prevents construction order from turning a missing binding into an unrelated projection error.

---

# 29. Exact-Conformance Normalization

When the subject and trait relationship are exact, normalization consumes exact conformance evidence.

```phalcom
trait Container {
  type Item
}

class Box<T> {}

impl<T> Container for Box<T> {
  type Item = T
}
```

For exact `Box<Int>`:

```text
Container::Item projected from Box<Int>
    → Int
```

For exact `Box<String>`:

```text
Container::Item projected from Box<String>
    → String
```

These forms describe semantic meaning, not additional source syntax.

Exact normalization is evidence-driven. It does not rescan source declarations or independently rediscover a conformance by associated name.

> **Exact-evidence invariant**
>
> Exact projection normalization consumes the canonical exact conformance evidence for the subject and trait reference.

---

# 30. Exact Associated Values in Conformance Evidence

Published exact conformance evidence contains the specialized associated values needed by exact consumers.

If the source binding is:

```phalcom
type Item = T
```

then exact evidence for `Box<Int>` records the semantic equivalent of:

```text
Item = Int
```

and exact evidence for `Box<String>` records:

```text
Item = String
```

Consumers should not need to reinterpret the original source binding syntax.

Exact evidence must not be marked proven if an associated projection that should have been reducible remains unresolved because of an associated binding failure.

---

# 31. Binding Dependencies

Associated bindings may depend on sibling associated projections.

```phalcom
trait Shape {
  type Element
  type Sequence
}

impl Shape for X {
  type Element = Byte
  type Sequence = List<Self::Element>
}
```

This dependency is valid and normalizes to:

```text
Element = Byte
Sequence = List<Byte>
```

Dependencies are determined by associated declaration identity after binding left-hand sides have been resolved.

They are not keyed by source spelling alone.

---

# 32. Cyclic Associated Bindings

Recursive associated binding cycles are invalid.

Direct cycle:

```phalcom
trait Cyclic {
  type A
}

impl Cyclic for X {
  type A = Self::A
}
```

Indirect cycle:

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

These do not normalize by recursion until stack overflow.

They produce a recursive/cyclic associated-type failure.

A diagnostic should identify enough of the cycle to explain that the associated values cannot be established.

> **Cycle invariant**
>
> Associated normalization is cycle-safe and bounded. Recursive associated definitions are semantic failures, not `Dynamic` fallbacks.

---

# 33. Recursive Projections Inside Compound Types

A cycle remains a cycle when hidden inside ordinary type structure.

```phalcom
trait Recursive {
  type Item
}

impl Recursive for X {
  type Item = List<Self::Item>
}
```

Normalizing `Item` would require itself indefinitely.

The enclosing `List` does not make the recursion valid.

Cycle detection applies to canonical associated projection dependencies, including dependencies reached through nested applied, tuple, record, callable, union, or other ordinary type structure.

---

# 34. Normalization Outcomes

Normalization must preserve materially different outcomes rather than collapsing all non-success states into one permissive type.

## 34.1 Normalized

A projection resolves to an ordinary type.

```text
Self::Item → Int
```

## 34.2 Symbolic

A well-formed projection remains abstract because the context intentionally has no exact binding.

This is the normal result inside an abstract trait.

## 34.3 Incomplete

The relevant conformance is not complete, for example because a required associated binding is missing or invalid.

## 34.4 Ambiguous

More than one incompatible conformance/evidence candidate remains applicable where the language requires a unique one.

## 34.5 Recursive

Normalization encounters an associated dependency cycle.

## 34.6 Unknown

The analyzer lacks sufficient semantic information to prove a concrete result for a reason that is not the same as deliberate dynamic typing.

## 34.7 Blocked

Normalization cannot proceed because another unresolved semantic obligation blocks it.

## 34.8 Dynamic boundary

Ordinary language semantics have deliberately reached `Dynamic`.

This is distinct from failure to determine a static type.

Implementations may also maintain analysis-control outcomes such as cancellation, budget exhaustion, or internal failure. Those states must not be reported or cached as if they were semantic proof of `Dynamic`, successful normalization, or conformance refutation.

> **Terminal-state invariant**
>
> Uncertainty, ambiguity, recursion, incomplete conformance, and deliberate dynamic typing are different facts and must remain distinguishable.

---

# 35. Bounded Normalization

Associated normalization must be bounded.

Implementations must prevent unbounded recursive projection expansion, recursion through mutually dependent bindings, uncontrolled repeated proof search, and cancellation or budget exhaustion being reinterpreted as semantic proof.

The precise resource-budget mechanism is implementation-defined.

Its semantic requirement is conservative honesty: interrupted analysis must not manufacture proof.

---

# 36. Exact Enum-Case Subjects

Associated conformances may target exact enum cases where the trait/conformance specifications permit exact-case conformance.

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

The associated conformance belongs to the exact `Ok` case target.

Normalization must not silently widen the subject to the root `Result<Int>` type.

A sibling exact case does not acquire the associated binding merely because it belongs to the same enum root.

> **Exact-case invariant**
>
> Exact-case identity survives associated conformance selection and projection normalization.

---

# 37. Trait Generic Arguments

Associated types are distinct from ordinary generic parameters of a trait.

```phalcom
trait Converter<Target> {
  type Error
  convert -> Target
}
```

`Target` is an ordinary trait generic parameter.

`Error` is an associated declaration.

For distinct trait references:

```text
Converter<Int>
Converter<String>
```

the trait generic argument participates in exact trait-reference identity.

The `Error` associated declaration remains owned by the trait declaration, while its concrete binding is supplied by each applicable conformance.

Projection identity therefore includes the relevant exact trait application/context, not merely the bare trait declaration.

---

# 38. Associated Types Are Not Hidden Trait Arguments

The language does not reinterpret:

```phalcom
trait Iterator {
  type Item
}
```

as if it had been declared with a hidden ordinary generic argument.

The roles are different:

```text
trait generic argument
    selected as part of the trait reference

associated type
    supplied as evidence by a conformance
```

A future constraint system may allow generic code to state requirements about associated values. That does not transform associated declarations into ordinary generic parameters.

---

# 39. Requirement Signature Specialization

Trait behavioral requirements may use associated projections.

```phalcom
trait Cell {
  type Item

  read -> Self::Item
  write(_ value: Self::Item) -> ()
}
```

When checking an exact conformance, the requirement contract is specialized using:

1. the conformance subject for `Self`;
2. ordinary trait and implementation generic substitutions;
3. normalized associated bindings.

If:

```phalcom
impl Cell for IntCell {
  type Item = Int
  ...
}
```

then the exact requirement view is equivalent to:

```text
read -> Int
write(_ value: Int) -> ()
```

for witness compatibility and trait-evidenced use.

The abstract trait declaration itself remains unchanged.

> **Contract-specialization invariant**
>
> Exact specialization creates an exact requirement view; it does not mutate the trait's abstract signature.

---

# 40. Witness Compatibility

A behavioral witness must be compatible with the associated-type-specialized requirement.

```phalcom
trait Cell {
  type Item
  read -> Self::Item
}

class IntCell {
  read -> Int { 42 }
}

impl Cell for IntCell {
  type Item = Int
}
```

The inherent getter may satisfy the requirement because the specialized return type is `Int`.

This conformance is invalid:

```phalcom
class BadCell {
  read -> String { "wrong" }
}

impl Cell for BadCell {
  type Item = Int
}
```

because the associated binding specializes the requirement to `read -> Int`, which is incompatible with the available `String` witness.

Associated normalization therefore occurs before or as part of the ordinary witness compatibility relation.

It does not replace that relation with special associated-type-only rules.

---

# 41. Explicit Conformance Witnesses

A conformance may provide a witness whose own signature uses the contextual associated projection.

```phalcom
trait Cell {
  type Item
  read -> Self::Item
}

class IntCell {}

impl Cell for IntCell {
  type Item = Int

  read -> Self::Item {
    42
  }
}
```

Within the source conformance, `Self::Item` normalizes through the source binding plan.

The witness is behavioral evidence.

The `type Item = Int` member is associated evidence.

Both participate in the one complete conformance.

---

# 42. Trait Defaults

A trait default may use associated projections symbolically.

```phalcom
trait IdentityCell {
  type Item

  identity(_ value: Self::Item) -> Self::Item {
    value
  }
}
```

The default is valid in the abstract trait context because the body only relies on the identity of the parameter and result type.

For an exact conformance with:

```text
Item = String
```

the selected default's exact callable contract specializes to:

```text
identity(_ value: String) -> String
```

Selecting the default does not create a new inherent member on the target and does not change ownership of the default.

Associated specialization and trait-default ownership are independent concerns.

---

# 43. Source Conformance and Default Selection

Source conformance checking may use the source associated binding plan to establish compatibility between a trait default or explicit witness and the source target.

It must not require a monomorphic exact target merely because a requirement mentions an associated projection.

```phalcom
impl<T> IdentityCell for Box<T> {
  type Item = T
}
```

may select the abstract default for valid exact substitutions if the ordinary conformance rules establish that selection.

The source plan carries `Item = T`; exact evidence later specializes that template for exact `Box` applications.

---

# 44. Type Relation and Symbolic Projections

Identical symbolic projections may compare as the same type when their canonical projection identity is the same.

Distinct unresolved projections must not be rejected merely because they occupy different canonical type nodes without a completed semantic comparison.

In particular, the type relation must not treat every unresolved associated projection as an unrelated nominal type.

Where evidence is sufficient, reducible projections should be normalized before a definitive subtype, equality, or compatibility judgment that depends on their concrete values.

Where evidence is insufficient, the relation must preserve the appropriate blocked, symbolic, or uncertain state instead of fabricating a mismatch.

---

# 45. Dynamic Is Not Projection Failure

`Dynamic` is an ordinary language type and boundary.

A projection may legitimately normalize to `Dynamic` if the conformance binds that associated type to `Dynamic`.

That case is different from:

- missing binding;
- unknown associated declaration;
- ambiguous conformance;
- recursive binding;
- cancelled analysis;
- insufficient evidence.

Therefore normalization failure does not implicitly yield `Dynamic`.

> **Dynamic distinction**
>
> `Dynamic` is a type result only when language semantics establish it, not a catch-all recovery value for failed associated proof.

---

# 46. Conformance Ambiguity

Associated normalization depends on conformance identity.

If the applicable conformance is ambiguous under ordinary coherence rules, the projection is ambiguous as well.

The language must not resolve such ambiguity by source order, whichever conformance was analyzed first, associated binding name, or whichever associated value appears most convenient.

Associated types do not introduce a separate coherence policy.

They consume the conformance coherence policy defined by the trait specification.

---

# 47. Projection and Conformance Nonexistence

An exact associated value exists only through an applicable conformance that supplies the owning associated requirement.

If a concrete type simply does not conform to the relevant trait, associated normalization cannot invent an associated value.

Consumers should preserve the distinction between:

```text
subject does not conform to trait
```

and:

```text
subject conforms, but associated binding is incomplete
```

because these are different semantic failures.

---

# 48. Diagnostics and Source Provenance

Associated-type diagnostics should preserve the source ownership of the relevant failure.

Typical primary locations are:

```text
duplicate binding
    duplicate binding source

unknown binding
    unknown binding source

invalid RHS
    invalid type expression

missing binding
    conformance declaration / conformance head

invalid projection name
    projection source

recursive bindings
    one or more binding sources participating in the cycle
```

A missing binding has no binding source by definition, so a diagnostic must not fabricate one.

Cross-module diagnostics must use ranges belonging to the source module they reference.

This specification does not fix exact diagnostic wording.

---

# 49. Canonical Source Identity

The associated declaration is the canonical source identity for declaration, binding, and projection occurrences.

Conceptually:

```text
trait declaration:
    type Item
         └──── canonical associated declaration

conformance:
    type Item = Int
         └──── binding of the same associated declaration

trait/default/conformance signature:
    Self::Item
          └──── projection of the same associated declaration
```

The binding is not a second declaration of a new associated type.

Source tooling should therefore be able to treat these occurrences as references to one trait-owned semantic declaration.

---

# 50. Definition Navigation

For:

```phalcom
trait Collection {
  type Item

  first -> Self::Item
}
```

definition navigation from the `Item` token in `Self::Item` should resolve to:

```phalcom
type Item
```

For:

```phalcom
impl Collection for X {
  type Item = Int
}
```

definition navigation from the binding name should likewise resolve to the trait declaration, not to an unrelated binding with the same spelling.

Tooling consumes canonical semantic associated identity.

It does not perform a separate text search for `Item`.

---

# 51. References

Where a reference engine exposes associated-type references, declaration, binding, and projection occurrences belonging to the same associated requirement should share the canonical declaration target.

References from different traits with the same associated name remain separate.

```text
Collection::Item references
    do not include
Stream::Item references
```

merely because both are spelled `Item`.

---

# 52. Incremental Replacement and Removal

Incremental analysis must be owner-complete for associated products.

Editing:

```phalcom
type Item = Int
```

to:

```phalcom
type Item = String
```

must invalidate exact associated values and dependent specialized requirement views that rely on the old binding.

Removing the binding must remove those products or replace them with the correct incomplete state.

Re-adding a binding must reconstruct the same semantics as cold analysis.

> **Incremental-equivalence invariant**
>
> After the same source state is reached, cold analysis and incremental analysis agree on associated declarations, bindings, projections, exact evidence, diagnostics, and dependent contracts.

---

# 53. Associated Declaration Changes

Adding, removing, or renaming an associated declaration changes the owning trait contract.

Dependent products include, as applicable:

- conformance completeness;
- binding left-hand-side resolution;
- projection formation;
- exact associated evidence;
- specialized requirement contracts;
- diagnostics;
- source navigation.

Incremental analysis must remove products owned by declarations that no longer exist.

It must not retain stale bindings merely because the same source position now contains a different name.

---

# 54. Reordering and Stable Semantic Meaning

Reordering associated declarations may change implementation-private ordinal identities if an implementation deliberately derives declaration identity from stable owner-relative declaration order.

This specification does not require impossible persistence of such implementation IDs across arbitrary source rewrites.

It requires:

- correct ownership;
- no name-only identity;
- no stale products;
- cold/incremental semantic equivalence for the resulting source.

User-visible semantics depend on the declaration in the current source, not on preserving an obsolete implementation token after a declaration has moved or changed.

---

# 55. Compiler Boundary

The compiler consumes semantic associated-type decisions.

It does not independently resolve:

```text
which trait owns Item?
which conformance applies?
what does Item bind to?
is this conformance complete?
```

When executable lowering requires an exact type, semantic analysis must provide either an already-normalized ordinary type or a stable semantic/lowering product whose interpretation does not require re-solving conformance at runtime.

The compiler may structurally transport generic type recipes, substitutions, or exact type references.

It must not become a second associated-type solver.

---

# 56. Runtime Boundary

Associated types have no independent runtime lookup protocol under this specification.

The VM does not execute an operation conceptually equivalent to:

```text
lookupAssociatedType(receiver, trait, "Item")
```

during ordinary execution.

Instead:

```text
source projection
    ↓ semantic formation
canonical associated projection
    ↓ semantic normalization / specialization
ordinary exact or generic type information
    ↓ lowering/runtime metadata
runtime execution
```

The runtime may instantiate already-formed generic type recipes under a runtime type environment.

Instantiation is not associated-type proof.

> **Runtime-authority invariant**
>
> Runtime execution consumes associated-type specialization; it does not determine associated-type meaning.

---

# 57. Trait Dispatch and Associated Types

Associated type bindings may specialize behavioral contracts used by trait-evidenced dispatch.

```phalcom
trait Cell {
  type Item
  read -> Self::Item
}
```

with:

```phalcom
impl Cell for IntCell {
  type Item = Int
  read -> Int { 42 }
}
```

produces an exact trait requirement contract equivalent to `read -> Int`.

The trait dispatch mechanism selects the behavioral witness through ordinary conformance evidence.

Associated bindings are inputs to that evidence.

They do not turn trait behavior into inherent behavior.

A runtime failure to find an inherent method must not be used as a fallback associated-type resolution mechanism.

---

# 58. Bound and Detached Callables

When a trait-evidenced callable is bound or detached, any exact type environment needed to preserve its associated-specialized contract must survive with the callable according to ordinary callable lifetime rules.

For example, if a callable contract involving `Self::Item` specializes to:

```text
(Int) -> Int
```

binding the callable must not later lose that `Int` specialization because the original expression or temporary evidence object is gone.

Garbage collection or callable detachment does not authorize re-solving associated types from scratch.

---

# 59. Metadata and Serialization Boundary

This specification does not mandate one stable metadata encoding for symbolic associated projections.

It mandates the semantic boundary:

- if symbolic projection-bearing contracts cross a durable semantic metadata boundary, that metadata preserves canonical associated identity sufficiently to reconstruct the same semantic projection;
- if executable/runtime metadata is produced only after normalization, it may carry the resulting ordinary types instead;
- no metadata format reduces associated projection identity to a string name alone;
- deserialization does not require runtime conformance search to recover semantics that should have been established statically.

The VM/internals specifications define concrete metadata products.

---

# 60. Reflection Boundary

This specification establishes semantic associated declaration identity.

It does not define a public reflection object for associated types.

A future reflection specification may expose associated declarations, owning traits, exact associated bindings, or projection descriptors.

Such reflection must consume the canonical semantic model rather than inventing a second identity scheme.

---

# 61. No Associated Type Defaults

The declaration:

```phalcom
trait T {
  type Item
}
```

requires each valid conformance to establish `Item`.

This specification does not define an associated type default inside a trait.

If defaults are introduced later, their interaction with completeness, source plans, ambiguity, and specialization requires an explicit specification amendment.

---

# 62. No Generic Associated Types

This specification does not define associated declarations with their own generic parameter lists.

Every associated declaration defined here denotes one proper type per exact conformance.

---

# 63. No General Generic-Subject Projection Surface

The source projection form standardized here is contextual `Self::Item`.

This specification does not define arbitrary generic-subject projection in generic code.

Generic code that projects associated types from evidence assumptions requires the separate generic evidence/constraint specification.

Implementations must not add a hidden assumption rule merely because a generic projection would be convenient for a particular feature.

---

# 64. No Explicit Qualified Projection Syntax

This specification does not define an explicit trait-qualified projection source form.

Canonical semantics still retain the relevant trait context internally.

If a later language revision adds explicit qualification, it must map to the same trait-owned associated identity rather than create a parallel projection model.

---

# 65. No Implicit Structural Associated Types

A class does not acquire an associated type merely because it has:

- a nested declaration with the same name;
- a getter returning a type;
- a generic parameter with the same name;
- a matching method;
- a field;
- another trait conformance with an associated declaration spelled the same way.

Associated types exist only as declarations in trait contracts and values established by conformances to those traits.

---

# 66. No Runtime Associated Namespace

The language does not define a runtime namespace in which ordinary member dispatch recovers conformance bindings.

Associated projections are static type semantics.

Any future reflection API is separate from ordinary member dispatch.

---

# 67. Canonical Examples

## 67.1 Simple generic binding

```phalcom
trait Container {
  type Item
}

class Box<T> {}

impl<T> Container for Box<T> {
  type Item = T
}
```

Semantic consequences:

```text
Box<Int>:
    Container::Item = Int

Box<String>:
    Container::Item = String
```

One source conformance supplies both exact specializations.

---

## 67.2 Associated projection in a requirement

```phalcom
trait Cell {
  type Item

  read -> Self::Item
  write(_ value: Self::Item) -> ()
}

class IntCell {
  read -> Int { 42 }
  write(_ value: Int) -> () { ... }
}

impl Cell for IntCell {
  type Item = Int
}
```

The exact `Cell` contract for `IntCell` is:

```text
read -> Int
write(_ value: Int) -> ()
```

The inherent methods may satisfy the behavioral requirements through ordinary witness compatibility.

---

## 67.3 Trait default

```phalcom
trait Identity {
  type Item

  identity(_ value: Self::Item) -> Self::Item {
    value
  }
}

class Box<T> {}

impl<T> Identity for Box<T> {
  type Item = T
}
```

The abstract default is checked once against symbolic `Self::Item`.

Exact conformance evidence specializes its callable contract for each exact `Box<T>` application.

---

## 67.4 Sibling dependency

```phalcom
trait SequenceShape {
  type Element
  type Sequence
}

impl SequenceShape for BytesView {
  type Sequence = List<Self::Element>
  type Element = Int
}
```

The binding order is irrelevant.

The normalized source bindings are semantically:

```text
Element = Int
Sequence = List<Int>
```

---

## 67.5 Labeled tuple associated value

```phalcom
trait EntrySource {
  type Key
  type Value
  type Entry

  current -> Self::Entry
}

impl<K, V> EntrySource for TableCursor<K, V> {
  type Key = K
  type Value = V
  type Entry = (key: Self::Key, value: Self::Value)

  current -> Self::Entry {
    ...
  }
}
```

For exact `TableCursor<String, User>`:

```text
Key   = String
Value = User
Entry = (key: String, value: User)
```

The labels remain part of the ordinary tuple structure.

---

## 67.6 Exact enum-case binding

```phalcom
trait PayloadType {
  type Value
}

enum Result<T> {
  Ok(_ value: T)
  Error(_ message: String)
}

impl PayloadType for Result<Int>::Ok(_) {
  type Value = Int
}
```

The binding belongs to the exact `Ok` case conformance.

It does not imply the same binding for `Result<Int>::Error(_)` or the root `Result<Int>` unless those targets have their own conformances.

---

## 67.7 Invalid missing binding

```phalcom
trait PairLike {
  type Left
  type Right
}

class PairBox {}

impl PairLike for PairBox {
  type Left = Int
}
```

The conformance is incomplete because `Right` is missing.

---

## 67.8 Invalid witness after associated specialization

```phalcom
trait Cell {
  type Item
  read -> Self::Item
}

class BadCell {
  read -> String { "wrong" }
}

impl Cell for BadCell {
  type Item = Int
}
```

The exact requirement is `read -> Int`.

The available witness is incompatible.

---

## 67.9 Invalid recursive binding

```phalcom
trait Recursive {
  type A
  type B
}

class X {}

impl Recursive for X {
  type A = Self::B
  type B = Self::A
}
```

The conformance is invalid because the associated binding graph is recursive.

---

# 68. Semantic Invariants Summary

The associated type model is governed by the following invariants.

```text
AT-01
Associated declarations are trait-owned nominal requirements.

AT-02
Associated identity is not a global string name or source range.

AT-03
Associated requirements and behavioral requirements remain separate categories.

AT-04
A conformance binding resolves only against the exact trait surface it implements.

AT-05
One generic source conformance may specialize to different exact associated values.

AT-06
Associated and behavioral failures participate in one conformance completeness decision.

AT-07
Fixed associated failures cannot be erased by later behavioral specialization.

AT-08
The standardized projection syntax in this surface is contextual Self::Item.

AT-09
Projection identity includes subject, trait context/application, and associated declaration identity.

AT-10
Projection formation and projection normalization are distinct operations.

AT-11
Abstract trait projections are valid symbolic types.

AT-12
Source-conformance normalization consumes the staged source binding authority.

AT-13
Source-conformance normalization does not request final evidence under construction.

AT-14
Exact normalization consumes canonical exact conformance evidence.

AT-15
Associated binding source order does not determine semantics.

AT-16
Sibling associated bindings may depend on one another through projections.

AT-17
Recursive associated bindings are detected as cycles rather than expanded indefinitely.

AT-18
Normalization recursively preserves ordinary type structure, including tuple labels.

AT-19
Uncertainty, ambiguity, recursion, incomplete conformance, and Dynamic remain distinct outcomes.

AT-20
Exact enum-case identity survives associated conformance and projection normalization.

AT-21
Witness compatibility uses associated-specialized requirement contracts.

AT-22
Exact specialization does not mutate the abstract trait declaration.

AT-23
Source tooling uses canonical associated declaration identity for declarations, bindings, and projections.

AT-24
Cold and incremental analysis converge on the same associated semantics.

AT-25
Compiler and runtime consume semantic associated decisions; they do not solve associated types independently.

AT-26
No runtime associated-type lookup protocol is introduced by this model.

AT-27
Associated type defaults, generic associated types, generic evidence assumptions, and explicit qualified projection syntax require separate specifications.
```

---

# 69. Relationship to Trait Conformance

Associated types are part of conformance evidence.

Conceptually:

```text
conformance evidence
    exact target
    exact trait reference
    behavioral witnesses/default selections
    associated type bindings
    generic environment
    completeness state
```

This document specifies the associated-type portion of that evidence.

The trait and implementation-declaration specifications remain authoritative for:

- explicit conformance declarations;
- conformance ownership;
- coherence;
- behavioral requirement identity;
- witness selection;
- trait defaults;
- trait-evidenced member dispatch;
- implementation declaration authorization.

Associated normalization consumes those products; it does not redefine them.

---

# 70. Relationship to Future Generic Evidence

The associated model defined here is sufficient to answer questions about declared and exact conformances such as:

```text
For this exact conformance of Box<Int> to Container,
what is Container::Item?
```

A separate generic evidence system is required to answer questions of the form:

```text
Given an abstract generic type for which conformance evidence is assumed,
what associated value may generic code project and what relationships
may it require of that value?
```

This document intentionally does not define the source syntax for those assumptions or relationships.

When that system is introduced, it must consume the same:

- trait identity;
- associated declaration identity;
- projection identity;
- conformance evidence;
- normalization authority.

It must not establish a second associated-type model.

---

# 71. Conformance to This Specification

An implementation conforms to this associated-type specification only if it preserves all of the following architectural boundaries:

```text
trait declaration
    owns associated requirement identity

source conformance
    owns binding templates and source binding failures

exact conformance evidence
    owns exact associated values

projection normalization
    owns recursive associated reduction and cycle detection

ordinary type operations
    preserve/rewrite structure without proving conformance

compiler/runtime/tooling
    consume canonical semantic products
```

A language implementation is non-conforming if it instead relies on any of these as semantic authority:

```text
global associated-name lookup
source-range identity
runtime associated lookup
LSP-only associated resolution
compiler-only associated proof
behavioral witness success overriding associated binding failure
Dynamic as universal recovery for unresolved projections
```

These boundaries ensure that the same source program has one associated-type meaning across type checking, conformance checking, dispatch lowering, incremental analysis, tooling, and execution.
