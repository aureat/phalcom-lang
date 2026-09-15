# Traits and Conformance

A `trait` declaration introduces a nominal reusable behavioral contract. A trait owns requirements, defaults, and associated type declarations, but it does not own or inject instance representation.

```phalcom
trait Printable {
  toString -> String
}
```

A target satisfies a trait through an explicit conformance declaration:

```phalcom
impl Printable for User {
  ...
}
```

The conformance records how the trait's requirements are satisfied for that target.

```text
trait declaration
    reusable behavioral contract

trait requirement
    trait-owned obligation

trait default
    trait-owned fallback implementation

witness
    concrete behavior satisfying a requirement

conformance
    target + exact trait reference + evidence
```

> **Core invariant**
>
> A trait states what behavior a subject must provide. A conformance states why one particular subject satisfies that contract. Neither mechanism changes the subject's semantic instance representation.

---

# Trait Identity

Every trait declaration introduces distinct nominal trait identity.

```phalcom
trait Printable {
  toString -> String
}

trait Displayable {
  toString -> String
}
```

`Printable` and `Displayable` remain different traits even though their current requirements have the same selector and type.

A target may conform to one, both, or neither.

Structural coincidence of requirements does not imply trait equivalence or conformance.

> **Nominality invariant**
>
> Trait identity derives from its declaration, not from the structural shape of its requirements.

---

# Trait Declarations

A trait declaration may contain:

- method requirements;
- getter requirements;
- setter requirements;
- index requirements;
- associated type declarations;
- default implementations for behavioral requirements.

A trait declaration cannot contain instance fields, data components, enum variants, or other instance representation.

A trait does not define a superclass relationship.

A trait declaration also does not, by itself, define a first-class runtime existential or trait-object value type.

> **Representation invariant**
>
> Traits describe capabilities and type relationships. They never supply instance storage to conforming targets.

---

# Trait Requirements

A behavioral trait member declared without an implementation body is a **trait requirement**.

```phalcom
trait Printable {
  toString -> String
}
```

`Printable.toString` is a semantic requirement owned by `Printable`.

A requirement is not itself a concrete target callable.

A conformance must satisfy the requirement with a compatible witness or an applicable trait default.

## Stable requirement identity

Requirement identity is independent of witness identity.

```text
Printable.toString
    trait requirement

User.toString
    concrete callable
```

If `User.toString` satisfies the requirement, the conformance contains a witness relation connecting the two semantic entities.

The requirement does not become an alias for the witness.

> **Requirement identity invariant**
>
> A trait requirement remains the same trait-owned obligation across every conformance and every witness that may satisfy it.

---

# Trait Defaults

A behavioral trait member declared with a body provides a **trait default**.

```phalcom
trait Describable {
  describe -> String {
    self.toString
  }
}
```

The default is behavior owned by the trait.

It is type-checked in the abstract trait context, using abstract `Self`, the trait's generic parameters, and abstract associated types.

A conformance may select that default when no concrete witness supersedes it.

Selecting the default does not copy the declaration into the target's inherent surface and does not change the default's semantic owner.

```text
trait requirement/default identity
    trait-owned

selection of default
    conformance-owned evidence
```

> **Default identity invariant**
>
> Trait defaults remain trait-owned behavior even when selected for a particular target.

---

# Abstract `Self`

Within a trait, `Self` denotes the conforming subject type.

```phalcom
trait Cloneable {
  clone -> Self
}
```

For a conformance of `User` to `Cloneable`, the requirement specializes as though `Self` denotes `User`.

`Self` may appear in parameter types, result types, generic constraints, associated-type relationships, and default bodies according to the ordinary type-system rules.

`Self` does not denote the trait declaration itself.

> **`Self` invariant**
>
> Trait `Self` is abstract over the conforming subject and is specialized through conformance evidence.

---

# Property Requirements

A getter-shaped requirement describes readable property behavior.

It does not require a physical field.

A compatible witness may be:

- an inherent getter;
- a data component;
- another property-like semantic member that satisfies the required selector and type.

For example:

```phalcom
data User(
  name: String,
)
```

may satisfy a trait requirement for a readable `name` property when the requirement type is compatible.

## Mutable property requirements

A mutable property requirement requires both readable and writable capability according to the ordinary property/member model.

An immutable data component can satisfy a compatible readable requirement but cannot, by itself, satisfy a mutable property requirement.

A getter witness cannot substitute for a setter requirement, and an index getter cannot satisfy an index setter requirement.

> **Storage-independence invariant**
>
> Trait property requirements describe observable access capabilities, never physical storage.

## Property elaboration and direct-field `via`

A property-shaped trait requirement is behavioral syntax. A readable property
contributes an ordinary getter requirement; a mutable property contributes both
an ordinary getter and an ordinary setter requirement. The resulting
`TraitRequirementId`s remain behavior-owned and participate in the normal
witness and selector rules.

The initial `via` form supplies a class or inherent-implementation witness for
those ordinary accessors by naming one directly declared field:

```phalcom
class Counter {
  mut _count: Int
  mut count via _count
}
```

The field remains the sole authority for the value type and mutability. A
setter delegation is valid only when the named field is mutable. The named
field must belong directly to the declaration being checked; inherited fields,
property chains, subscripts, calls, arbitrary places, and delegate objects are
not `via` targets. `via` creates no storage and no runtime delegation protocol;
its accessors are ordinary target behavior and are subject to ordinary
selector-conflict rules.

The initial form is available in class bodies and inherent implementations.
Conformance-local `via` is rejected because a conformance body does not acquire
the target class's private-field authority. A target may instead expose
ordinary accessors that a conformance witnesses through the normal conformance
rules.

---

# Selector Identity

Trait behavioral requirements use the ordinary Phalcom selector model.

Selector identity is determined by the language-defined combination of:

```text
base name
positional arity
labels
variadic / pack shape
member category
dispatch side
```

Parameter and result types participate in compatibility checking but do not independently create selector overloading.

A trait cannot declare two incompatible requirements with the same selector merely by changing parameter or return types.

The same selector rules apply when matching a requirement to a witness.

---

# Associated Types

A trait may declare an **associated type** whose concrete binding depends on conformance.

```phalcom
trait Iterator {
  type Item
}
```

The associated type declaration is owned by the trait and has stable identity across all conformances.

A conformance binds that declaration:

```phalcom
impl<T> Iterator for List<T> {
  type Item = T
  ...
}
```

Conceptually:

```text
Iterator.Item
    associated type declaration

Iterator for List<T>
    Item = T
```

An associated type is not an ordinary generic argument of the trait.

Its concrete meaning is obtained through a conformance.

> **Associated-type invariant**
>
> Associated types are trait-owned declarations whose concrete bindings are conformance-dependent projections.

In the initial declaration/binding surface, an associated declaration is a
plain trait-owned type requirement with no default, generic associated-type
binder, or additional trait-bound syntax:

```phalcom
trait Iterator {
  type Item
}

impl<T> Iterator for List<T> {
  type Item = T
}
```

The binding name resolves against the exact trait declaration, not by a global
name lookup. Its right-hand side is checked in the conformance's generic
scope. Duplicate, unknown, invalid, or missing bindings make the conformance
incomplete; valid bindings are retained as conformance evidence. Associated
type declarations do not become callable requirements and do not add storage
or runtime lookup behavior.

## Associated type constraints

An associated type declaration may carry constraints where permitted by the general type system.

A conformance binding must satisfy those constraints under the conformance's generic substitution.

## Projection

Associated type projection uses the language's associated-type projection rules. Within a trait or conformance, `Self::Item` denotes the `Item` projection associated with the relevant `Self` and trait context when unambiguous.

Projection formation and normalization are later semantic consumers of the
canonical declaration, binding, and evidence products described above. They do
not authorize a second name-based binding resolver or runtime associated-type
lookup. Projection normalization, ambiguity, cycles, and future generic
associated types are specified by the associated-type/type-system
specifications rather than by this chapter.

---

# Conformance

A conformance declaration has the form:

```phalcom
impl TraitReference for Target {
  ...
}
```

A conformance is a semantic relationship among:

```text
exact trait reference
target
implementation provenance
target/trait generic substitutions
requirement witnesses
selected defaults
associated type bindings
nested conformance evidence
```

It is not merely a boolean annotation and not an inherent member fragment.

> **Conformance invariant**
>
> A conformance identifies one particular proof that a target satisfies one exact trait reference.

## Conformance targets

A conformance target must be a declaration-owned nominal target or an exact enum case whose semantics admit conformance. This includes data object types, class types, enum roots, and exact enum cases.

Anonymous structural Tuple, Record, function, and other structural type expressions are not direct conformance targets under this specification. Conformance is nominal and explicit.

For an exact enum case, the enum specification determines the case type environment while this specification determines trait requirement satisfaction.

---

# Explicit Conformance

Conformance is explicit.

Possessing structurally matching members does not, by itself, make a target conform to a trait.

```phalcom
class User {
  toString -> String { ... }
}
```

is not enough to establish `Printable` conformance merely because `Printable` requires `toString`.

The program must contain an applicable conformance declaration unless another specification explicitly defines derived or synthesized conformance.

Existing behavior may then be reused as witness evidence.

> **Explicitness invariant**
>
> Structural member coincidence may satisfy a witness obligation inside an explicit conformance, but it does not create the conformance itself.

---

# Conformance Ownership

A conformance may be declared only in a module that owns at least one side of the relationship:

```text
conformance declaration module
    == trait declaration module
        or
    == target declaration module
```

For an exact enum case, target ownership is inherited from the enum declaration's module.

This trait-or-target ownership rule prevents unrelated third-party modules from introducing globally competing conformances between two declarations they do not own.

Re-exporting a trait or target does not transfer ownership for this purpose.

> **Conformance-ownership invariant**
>
> A module may define a conformance only when it owns the trait or the target.

---

# Trait References

A conformance is established to an exact **trait reference**, not merely to a bare trait declaration name.

For a generic trait:

```phalcom
trait Converter<Target> {
  convert -> Target
}
```

`Converter<Int>` and `Converter<String>` are distinct trait references.

Generic arguments participate in trait-reference identity according to the ordinary exact-type rules.

A target may conform independently to distinct applications of the same generic trait when the conformances do not overlap as exact trait references and their resulting member use remains otherwise coherent.

Thus these are distinct conformance keys:

```text
User + Converter<Int>
User + Converter<String>
```

They are not considered duplicate conformances merely because the generic trait declaration is the same.

---

# Conformance Completeness

A conformance is valid only if every applicable requirement is satisfied and every required associated type has a valid binding.

A behavioral requirement may be satisfied by:

- one compatible existing inherent member;
- a compatible witness body declared in the conformance;
- the trait's applicable default implementation.

A required associated type must have a binding supplied or otherwise determined by rules explicitly defined for that trait.

If no valid satisfaction source exists, the conformance is incomplete and invalid.

> **Completeness guarantee**
>
> Declaring conformance commits the target to the trait's complete applicable contract.

---

# Witness Selection

A **witness** is concrete semantic behavior selected to satisfy one trait requirement.

Witness selection considers:

- selector compatibility;
- member/dispatch-role compatibility;
- type compatibility after `Self` and generic substitution;
- associated type substitutions;
- visibility/accessibility;
- any other callable compatibility conditions of the general type system.

## Automatic reuse of inherent members

If the target's effective inherent behavior contains exactly one compatible member for a requirement, that member is selected automatically as the witness unless the conformance body supplies an explicit witness body for that requirement.

Example:

```phalcom
class User {
  toString -> String {
    ...
  }
}

impl Printable for User {
}
```

may be complete because the existing `User.toString` uniquely satisfies `Printable.toString`.

Becoming a witness does not change the callable's inherent identity.

## Witness bodies in the conformance

A conformance may declare a concrete witness body using the requirement's ordinary member shape:

```phalcom
impl Printable for User {
  toString -> String {
    ...
  }
}
```

Such a declaration is a conformance witness for `Printable.toString`, not a new independent inherent overload of `User.toString`.

If a conflicting inherent selector already exists and cannot be the same compatible witness, the conformance is invalid rather than silently shadowing the inherent member.

## One callable, multiple requirements

One concrete inherent callable may witness multiple trait requirements when it is independently compatible with each.

```text
User.toString
    witnesses Printable.toString
    witnesses Debuggable.toString
```

The callable remains one callable. Each trait requirement retains distinct identity and a distinct witness relation.

> **Witness identity invariant**
>
> Witness selection relates existing or conformance-defined behavior to a requirement without collapsing requirement identity into callable identity.

---

# Witness Visibility

A witness must be accessible everywhere the trait requirement promises that capability to be accessible.

Compatibility is therefore based on semantic access coverage rather than literal equality of visibility tokens.

A witness whose access set is narrower than the required contract cannot satisfy that requirement.

The general visibility specification defines the access sets of traits, requirements, targets, conformances, and concrete members.

---

# Default Selection

For one requirement in one conformance, behavior is selected in the following order:

```text
explicit/compatible concrete witness
    otherwise
trait default
    otherwise
conformance incomplete
```

An existing compatible inherent member is a concrete witness and therefore takes precedence over the trait default.

A conformance-defined witness body likewise supersedes the default for that conformance.

The default remains available to other conformances that do not provide a concrete witness.

---

# Trait Behavior and Ordinary Member Syntax

Trait behavior selected by an applicable conformance participates in ordinary instance dispatch without becoming an inherent declaration.

For a receiver whose static type and conformance evidence establish `Printable`, an applicable `toString` requirement may be called using ordinary instance syntax:

```phalcom
object.toString
```

and referenced through the ordinary instance callable-reference syntax where callable shape requires it:

```phalcom
&object.method(_)
```

The member remains trait-evidenced behavior when no inherent declaration owns it.

This distinction matters to reflection and ambiguity analysis.

```text
available through ordinary instance dispatch
    ≠
inherent declaration owned by target
```

> **Surface-separation invariant**
>
> Conformance can make trait behavior dispatchable through a target without copying the trait member into the target's inherent surface.

---

# Multiple Traits and Selector Ambiguity

A target may conform to multiple traits that contain requirements with the same selector.

The requirements remain semantically distinct.

## Shared concrete witness

If one inherent member is compatible with both requirements, it may witness both and ordinary dispatch is unambiguous because both relations select the same concrete callable.

## Competing defaults

If multiple applicable traits supply defaults for the same ordinary selector and no single concrete inherent witness resolves the selector, ordinary unqualified dispatch is ambiguous.

The language does not choose one default by trait declaration order, conformance order, import order, or source order.

A program that needs unqualified ordinary dispatch must provide a concrete witness that resolves the selector for the target.

Future trait-qualified dispatch syntax may provide an additional way to select a particular trait member, but no such syntax is implied by this specification.

## Incompatible requirements

Requirements with the same selector but incompatible types remain separate obligations. A single witness can satisfy both only when it is compatible with each independently.

> **Default-ambiguity invariant**
>
> Trait defaults never acquire implicit precedence over one another.

---

# Generic Conformance

A conformance declaration may introduce its own generic parameters:

```phalcom
impl<T> Printable for List<T> {
  ...
}
```

The generic parameters belong to the conformance declaration and map into the target, trait reference, associated bindings, witness signatures, and constraints.

Parameter spelling does not determine generic identity.

The implementation-declaration specification defines implementation-local generic scope.

---

# Conditional Conformance

A conformance may be conditional on generic constraints.

Conceptually:

```text
List<T> conforms to Printable
    if
T conforms to Printable
```

The source-level constraint clause uses Phalcom's ordinary generic constraint language.

A conditional conformance is applicable to an exact target only when its constraints are proven.

```text
conformance declaration
    +
target / trait substitutions
    +
constraint evidence
    ↓
applicable conformance
```

The generic declaration does not become unconditionally conforming merely because a conditional conformance declaration exists.

> **Conditional-conformance invariant**
>
> Conformance applies only where the semantic environment proves its declared conditions.

---

# Conformance Evidence

Successful conformance resolution yields structured semantic evidence.

Conceptually, evidence includes:

```text
selected conformance
exact target substitution
exact trait-reference substitution
associated type bindings
requirement witnesses
selected defaults
nested conformance evidence
```

This is stronger than a boolean statement that a type “conforms.”

Associated-type projection, generic dispatch, nested conditional conformance, reflection, and diagnostics may all depend on which evidence established the result.

## Nested evidence

A conditional conformance may depend on another conformance.

For example, evidence that `List<User>` conforms to `Printable` may contain evidence that `User` conforms to `Printable`.

Conformance evidence may therefore be recursive in structure, subject to the general type system's termination and cycle rules.

> **Evidence invariant**
>
> Conformance is justified by a specific proof structure, not merely by a membership bit.

---

# Coherence

For one exact target and one exact trait reference, at most one applicable conformance may exist.

Two conformance declarations overlap when there exists an exact target and exact trait reference for which both are applicable.

Textually different generic heads may overlap semantically.

Example:

```text
impl<T> Trait for Box<T> where T <: Number
impl<T> Trait for Box<T> where T == Int
```

both apply to `Box<Int>` when `Int <: Number`.

Such overlapping conformances are invalid.

The language has no trait specialization or “most specific conformance” rule.

Distinct applications of a generic trait are distinct trait references and therefore do not overlap merely because they arise from the same trait declaration:

```text
Target + Trait<Int>
Target + Trait<String>
```

Other member-ambiguity rules still apply to their behavior.

> **Coherence invariant**
>
> An exact target and exact trait reference have at most one applicable conformance.

---

# Trait Constraints in Generic Code

A generic constraint requiring trait conformance establishes a proof obligation.

Conceptually:

```text
T conforms to Printable
```

requires the type checker to resolve applicable conformance evidence for `T` in the relevant environment.

Successful resolution makes the conformance's witnesses, defaults, associated bindings, and nested evidence available to the generic typing and lowering process.

The generic-constraint specification defines the source syntax and solver integration for trait constraints.

---

# Interaction with Data Objects

A data object type may conform to traits without changing its representation.

Data components may serve as witnesses for compatible readable property requirements.

Because data components are immutable, a component alone cannot satisfy a mutable property requirement requiring write capability.

Trait conformance cannot add hidden state to a data object.

---

# Interaction with Enums

An enum root or exact enum case may conform to a trait where the target and ownership rules permit it.

Trait conformance remains separate from the enum's own closed-case behavior contract.

A concrete callable may, where independently compatible, serve both as:

```text
an enum root-requirement witness
and
a trait-requirement witness
```

The two requirement identities and witness relations remain separate.

Conformance cannot add variants, change exhaustiveness, alter exact-case refinement, or substitute for an enum's closed-case completeness rules.

---

# Interaction with Classes

Class inheritance and trait conformance are independent.

A class may inherit behavior according to class semantics and separately conform to traits.

An inherited member that is part of the target's effective accessible behavior may serve as a trait witness when it satisfies the ordinary witness compatibility rules.

An override selected by the class's normal dispatch semantics is the effective member considered for witness compatibility at the conformance target.

Conformance does not create a superclass relationship.

---

# Interaction with Inherent `impl`

An inherent implementation:

```phalcom
impl User {
  ...
}
```

contributes target-owned behavior.

A conformance implementation:

```phalcom
impl Printable for User {
  ...
}
```

establishes trait evidence.

An inherent member may be reused automatically as a witness.

A conformance-defined witness does not silently create a competing inherent selector.

The implementation-declaration specification governs generic scope, provenance, and common `impl` syntax; this specification governs conformance semantics.

---

# Reflection

Where reflection exposes traits and conformances, it preserves the distinctions among:

```text
trait declaration
exact trait reference
trait requirement
trait default
associated type declaration
conformance
associated type binding
witness callable
default selection
implementation provenance
```

Reflection can report that a concrete callable witnesses one or more requirements without replacing those requirement identities.

A selected trait default remains distinguishable from the conformance that selected it.

Reflection descriptors report semantic facts and do not create conformance.

> **Reflection invariant**
>
> Requirement, default, witness, conformance, associated binding, and reflection-wrapper identities remain distinct.

---

# Dispatch and Runtime Freedom

Trait-dependent calls dispatch through witnesses selected by applicable conformance evidence.

A conforming implementation may realize this using:

- direct static lowering;
- witness tables;
- conformance tables;
- specialized generic code;
- cached conformance evidence;
- other equivalent mechanisms.

Runtime reflection or class-table scanning is not the semantic authority for determining whether a conformance exists.

The static semantic program establishes the conformance relation.

Equivalent programs cannot change conformance or witness selection according to module traversal order, import order, or cache state.

---

# Traits Are Not Classes

Traits do not:

```text
own instance fields
define class layout
establish superclass relationships
inject representation into conforming targets
```

A class may conform to a trait, but conformance and inheritance remain different semantic relationships.

---

# Traits Are Not Enum Root Contracts

An enum root requirement quantifies over the closed exact cases of one enum.

A trait requirement belongs to a reusable contract satisfied by explicitly conforming targets.

```text
enum root requirement
    closed-world, enum-owned obligation

trait requirement
    reusable, trait-owned obligation
```

The language does not synthesize hidden traits for enum root requirements, and trait conformance does not replace enum closed-case completeness.

---

# Traits Are Not Existential Value Types

A trait declaration does not automatically introduce a first-class runtime value type.

This specification does not define trait objects, existential trait packaging, `dyn Trait`, `any Trait`, or an equivalent feature.

Such a facility requires a separate specification describing representation, dispatch, associated-type erasure, and variance semantics.

> **Existential-separation invariant**
>
> Trait-as-contract does not imply trait-as-runtime-value-type.

---

# Trait Inheritance

This specification does not define trait inheritance, supertraits, or trait composition as a separate language mechanism.

A trait may participate in ordinary generic constraints involving other traits where the generic constraint system permits it, but such a constraint does not itself create an inheritance hierarchy between trait declarations.

A future trait-composition specification may introduce stronger relationships explicitly.

---

# Metatype and Class-Side Conformance

This specification does not define syntax for conforming a metatype/class-side target to a trait.

In particular, runtime/reflection expressions such as `.class` are not implicitly repurposed as conformance-target syntax.

Any future class-side conformance model requires a separate specification of the semantic target and source syntax.

---

# Invalid Programs

## Incomplete conformance

A conformance lacking a witness/default for a required behavior or lacking a required associated type binding is invalid.

## Incompatible witness

A member that fails selector, dispatch-role, type, generic, `Self`, associated-type, or visibility compatibility cannot satisfy the requirement.

## Mutable-property mismatch

A read-only property cannot satisfy a mutable property requirement without compatible write behavior.

## Unauthorized conformance

A conformance declared in a module that owns neither the trait nor the target is invalid.

## Overlapping conformance

Two conformances that can both apply to the same exact target and exact trait reference are invalid.

## Missing associated binding

A required associated type without a valid binding makes the conformance incomplete.

## Invalid associated binding

A binding that violates associated-type constraints is invalid.

## Conflicting requirement declarations

A trait cannot contain incompatible behavioral declarations with the same selector unless another callable-family rule explicitly distinguishes them.

## Ambiguous trait dispatch

An unqualified instance dispatch that would select among multiple competing trait defaults without a single resolving concrete witness is invalid.

## Invalid trait-as-value use

A trait cannot be used as a first-class existential value type under this specification.

---

# Diagnostics

Diagnostics distinguish, where practical, among:

- unresolved trait reference;
- invalid trait member;
- duplicate requirement selector;
- incomplete conformance;
- missing witness;
- incompatible witness;
- witness visibility too narrow;
- mutable-property requirement mismatch;
- missing associated type binding;
- invalid associated type binding;
- overlapping conformance;
- unauthorized conformance;
- ambiguous trait-default dispatch;
- invalid generic conformance;
- unsatisfied conditional conformance;
- cyclic conformance dependency under the general solver rules;
- invalid existential/trait-object use.

Diagnostic codes and exact wording are implementation-defined, but a contract failure must not be reduced to an unrelated generic “method not found” diagnostic when the trait/conformance relation gives the failure a more precise meaning.

---

# Non-Normative Implementation Model

This section is non-normative.

A compiler may represent trait semantics conceptually as:

```text
TraitSurface {
    requirements
    defaults
    associated types
}

TraitConformance {
    exact trait reference
    target
    substitutions
    associated bindings
    witnesses
    selected defaults
}
```

Conformance resolution may produce evidence such as:

```text
ConformanceEvidence {
    selected conformance
    substitutions
    associated bindings
    witnesses
    selected defaults
    nested evidence
}
```

The exact representation is implementation-specific.

The important separation is:

```text
trait declaration identity
    ≠
requirement/default identity
    ≠
conformance identity
    ≠
witness callable identity
    ≠
reflection descriptor identity
```

---

# Reserved Facilities

The following are not defined by this specification:

- trait inheritance or supertrait syntax;
- trait specialization;
- overlapping conformance resolution by specificity;
- first-class trait objects or existential trait values;
- metatype/class-side conformance syntax;
- generic associated types;
- trait-qualified callable-reference syntax beyond the ordinary concrete instance reference model.

These facilities may be introduced by later specifications without changing the foundational distinction between traits, conformances, and witnesses.

---

# Governing Guarantees

> **Behavioral contract**
>
> A trait defines reusable behavioral and type requirements, not instance representation.

> **Nominal trait identity**
>
> Structurally identical trait declarations remain distinct contracts.

> **Stable requirement identity**
>
> Requirements remain distinct from the concrete callables that witness them.

> **Abstract `Self`**
>
> `Self` denotes the conforming subject and specializes through conformance.

> **Storage independence**
>
> Property requirements describe capabilities and never inject fields or components.

> **Explicit conformance**
>
> Matching members do not create conformance without an applicable conformance declaration or another explicitly specified derivation mechanism.

> **Trait-or-target ownership**
>
> A conformance may be declared only by the module that owns the trait or the target.

> **Witness reuse**
>
> A unique compatible inherent member is automatically reusable as a witness without losing its inherent identity.

> **Default preservation**
>
> Trait defaults remain trait-owned behavior selected through conformance evidence.

> **Concrete-witness precedence**
>
> A compatible concrete witness supersedes a trait default for that conformance.

> **Conformance completeness**
>
> Every applicable behavioral and associated-type obligation must be satisfied.

> **Visibility coverage**
>
> A witness must be accessible everywhere the requirement promises the capability.

> **Associated-type dependence**
>
> Associated type bindings belong to conformances rather than becoming ordinary target aliases by implication.

> **Structured evidence**
>
> Conformance resolution preserves the selected conformance, substitutions, witnesses, defaults, associated bindings, and nested evidence required to justify the result.

> **Conditional applicability**
>
> Conditional conformances exist only where their constraints are proven.

> **Coherence**
>
> One exact target and one exact trait reference have at most one applicable conformance.

> **Generic trait-reference identity**
>
> Distinct exact generic applications of one trait are distinct trait references and may have independent conformances when otherwise coherent.

> **No default precedence by order**
>
> Competing trait defaults are never selected by declaration, conformance, import, or source order.

> **No inheritance conflation**
>
> Trait conformance does not establish class inheritance.

> **No enum-contract conflation**
>
> Trait requirements remain distinct from closed enum root requirements.

> **No existential implication**
>
> A trait declaration does not automatically define a first-class runtime trait value.

> **Reflection fidelity**
>
> Reflection preserves the distinction among trait, requirement, default, conformance, witness, associated type, associated binding, and implementation provenance.

The governing rule is:

> **A trait defines a nominal reusable contract; a conformance supplies the semantic evidence that one target satisfies one exact trait reference. Requirements, defaults, witnesses, associated bindings, and conformance evidence retain distinct identities even when they participate in one dispatch path.**
