# Implementation Declarations

An `impl` declaration contributes behavior to an existing semantic target or establishes an explicit trait conformance for a target.

```phalcom
impl Point {
  magnitude { ... }
}
```

```phalcom
impl Printable for User {
  ...
}
```

These forms share the `impl` declaration mechanism but have different semantic effects.

```text
impl T { ... }
    inherent implementation

impl Trait for T { ... }
    conformance implementation
```

An **inherent implementation** contributes behavior semantically owned by its target. A **conformance implementation** establishes evidence that a target satisfies a trait contract and is governed further by the trait specification.

> **Core invariant**
>
> `impl` separates behavior from representation without separating behavior from its semantic owner.

---

# Semantic Model

## Implementation declarations

An implementation declaration has:

- a semantic target;
- an implementation kind;
- a source/provenance identity;
- an implementation-local generic environment, if any;
- an applicability condition;
- a body containing declarations permitted for that implementation kind.

An implementation declaration does not introduce a new nominal type.

It does not wrap its target, create a secondary extension namespace, or reopen a runtime class.

## Inherent implementations

An inherent implementation has the form:

```phalcom
impl Target {
  ...
}
```

or, for a generic target:

```phalcom
impl<A, B> Pair<A, B> {
  ...
}
```

An inherent implementation contributes target-owned behavior.

For:

```phalcom
impl User {
  name { ... }
}
```

`name` is an inherent member of `User`.

The `impl` declaration is the member's source provenance; it is not a separate behavioral owner.

```text
callable owner
    User

implementation provenance
    the particular impl declaration
```

> **Ownership invariant**
>
> A callable introduced through an inherent `impl` is owned by the target. The `impl` records where and under which generic conditions the callable was declared.

## Conformance implementations

A conformance implementation has the form:

```phalcom
impl Printable for User {
  ...
}
```

It establishes a conformance rather than contributing an ordinary inherent fragment.

The trait specification defines requirement satisfaction, witness selection, defaults, associated bindings, coherence, and conformance evidence.

> **Impl-kind invariant**
>
> Inherent implementation and trait conformance may share syntax and generic machinery without producing the same semantic product.

---

# Static Composition

An inherent implementation is a static declaration.

It does not mean:

```text
load module
    ↓
find runtime type
    ↓
mutate method table
```

Import order, module initialization order, cache state, source-file discovery order, and runtime loading order do not change the target's inherent behavior.

The semantic program determines the complete set of authorized inherent implementations.

A conforming implementation may precompute effective member surfaces, dispatch tables, or other runtime products, but such products realize the static declaration model rather than define it.

> **Static-composition guarantee**
>
> An inherent `impl` is never a runtime reopening or monkey-patching operation.

---

# Authorization

## Inherent ownership boundary

An inherent implementation may be declared only in the module that owns the target declaration.

Conceptually:

```text
impl declaration module
    ==
target declaration module
```

For an exact enum case, the owning module is the module that owns the enum declaration.

This rule applies regardless of where the target is imported or re-exported.

An unrelated dependency cannot add inherent behavior to a nominal type it does not own.

> **Authorization invariant**
>
> Inherent behavior may be split into `impl` declarations, but it remains controlled by the module that owns the nominal target.

A future language revision may deliberately broaden the ownership boundary. Such a revision changes the language's coherence model and is not implied by this specification.

## Authorization is not visibility

The right to declare an inherent implementation and the accessibility of members declared inside it are separate concerns.

An authorized implementation may contain members with restricted visibility according to the ordinary visibility rules.

---

# Legal Inherent Targets

An inherent implementation target must be a nominal semantic target owned by a declaration, or an exact enum case defined by such a declaration.

Legal target categories are:

```text
data object type
enum root
exact enum case
class type
covering generic application of an eligible nominal type
```

Examples:

```phalcom
impl Point {
  ...
}
```

```phalcom
impl<A, B> Pair<A, B> {
  ...
}
```

```phalcom
impl Expression {
  ...
}
```

```phalcom
impl Expression::Literal(_) {
  ...
}
```

Anonymous structural types such as Tuple and Record are not direct inherent `impl` targets.

Function types and other structural type expressions are likewise not direct inherent targets unless a separate specification explicitly introduces such a facility.

> **Nominal-target invariant**
>
> Inherent behavior is attached to declaration-owned nominal targets, not to arbitrary structural type expressions.

---

# Exact Enum Case Targets

An exact enum case may be an inherent implementation target:

```phalcom
impl Option::Some(_) {
  ...
}
```

The target is the exact case described by the variant shape.

It is not a variant-constructor invocation and not a callable reference.

```phalcom
Option::Some(42)      // construction
&Option::Some(_)      // callable reference

impl Option::Some(_) {
  ...                 // exact-case implementation target
}
```

The enum specification defines root requirements, root defaults, exact-case witnesses, overrides, case-only members, GADT specialization, and payload availability.

The present specification governs target resolution, ownership, generic scope, selector coherence, and implementation provenance.

---

# Representation Boundary

An inherent implementation contributes behavior only.

It cannot change the representation-defining declaration of its target.

It cannot introduce:

```text
class instance fields
data components
enum variants
enum payload components
superclass relationships
layout declarations
hidden semantic instance state
```

For data objects:

```text
data declaration
    owns the complete component state

impl
    contributes behavior only
```

For enums:

```text
enum declaration
    owns variants and their payload state

impl
    contributes behavior only
```

For classes, instance storage and superclass relationships remain governed by class declaration semantics.

> **Representation-preservation invariant**
>
> `impl` may change what behavior a target provides. It cannot change what semantic state or inheritance structure defines that target.

---

# Permitted Behavior

An inherent implementation may declare behavior categories supported by the ordinary Phalcom member system, including:

- methods;
- getters;
- setters;
- index getters and setters;
- other callable member categories explicitly defined by the member specification.

Associated/class-side behavior may be declared through `impl` only where the ordinary associated-member specification defines that member category for the target.

Primary data constructors and enum variant constructors retain their own semantic categories. An implementation declaration cannot reclassify or replace them with an ordinary associated member.

Metatype-target implementation syntax is not defined by this specification.

---

# Effective Inherent Behavior

A target's **effective inherent behavior** is the checked composition of:

```text
members declared directly by the target
    +
members contributed by applicable authorized inherent impls
```

Ordinary inherent member lookup observes this semantic composition rather than the source-file boundaries in which members were written.

A member declared directly and the same member declared through `impl` occupy the same target-owned selector namespace.

The source form does not create distinct “direct method” and “extension method” dispatch categories.

> **Surface invariant**
>
> Source fragmentation through `impl` does not fragment the target's inherent member surface.

---

# Selector Identity

Implementation members use the ordinary Phalcom selector model.

Selector identity is determined by the language-defined callable shape, including as applicable:

```text
base name
positional arity
labels
variadic / pack shape
member category
dispatch side
```

Parameter types and return types do not independently distinguish selectors.

Thus declarations such as:

```phalcom
impl Example {
  convert(_ value: Int) { ... }
  convert(_ value: String) { ... }
}
```

conflict when both declarations have the same Phalcom selector.

They are not type-overloaded methods merely because their parameter types differ.

Generic parameter names and constraints likewise do not create new selector identities. Constraints affect applicability, not the identity of the selector contributed when the implementation applies.

> **Selector invariant**
>
> `impl` preserves Phalcom's selector-shaped callable identity and does not introduce parameter-type-based overloading.

---

# Member Conflicts

For any exact target and selector, at most one applicable concrete inherent definition may exist.

Conflicting definitions are invalid.

There is no last-definition-wins, last-import-wins, or source-order-wins rule.

## Direct declaration and `impl`

A concrete member declared directly on a class and a concrete member contributed through an inherent `impl` conflict if they define the same selector for the same target and both are applicable.

## Data components

A data component is a semantic component rather than a synthesized getter, but its property selector is reserved by the data declaration.

```phalcom
data Point(x: Int)

impl Point {
  x { 42 }
}
```

is invalid.

An implementation cannot override, shadow, or reclassify a data component.

## Enum variants and constructors

Variant constructors retain associated-constructor identity.

An associated member introduced through an `impl` cannot silently hide or replace an enum variant constructor with a colliding selector.

## Conflict detection is semantic

Conflicts are determined after target resolution, generic substitution, and applicability analysis.

Textually different implementation heads can still conflict if there exists an exact target for which both contribute the same selector.

---

# Implementation-Local Generics

An implementation declaration may introduce its own generic parameters.

```phalcom
impl<A, B> Pair<A, B> {
  swapped -> Pair<B, A> {
    ...
  }
}
```

`A` and `B` belong to the implementation declaration's lexical generic scope.

They are not the original declaration's generic parameter identities reused by spelling.

The target application maps implementation-local parameters into the generic parameter space of the target declaration.

Therefore:

```phalcom
impl<X, Y> Pair<X, Y> {
  ...
}
```

has the same generic coverage as the corresponding form written with `A` and `B`, assuming the same target mapping and constraints.

> **Generic-scope invariant**
>
> Generic parameter identity is lexical and semantic, not name-based.

---

# Covering Generic Implementations

A **covering implementation** applies across the complete generic domain of the target declaration, subject only to constraints that are themselves universally satisfied for that declaration domain.

The usual form is:

```phalcom
impl<A, B> Pair<A, B> {
  ...
}
```

Coverage is semantic rather than textual.

For example:

```phalcom
impl<A, B> Pair<B, A> {
  ...
}
```

is also covering when every exact `Pair<X, Y>` corresponds to a substitution of the implementation-local parameters.

The compiler determines coverage through generic target substitution rather than declaration-header string equality.

A covering implementation contributes unconditional inherent behavior to every exact application covered by the target.

---

# Conditional Implementations

An implementation may refine a covering generic target by constraints.

```phalcom
impl<T> Point<T> where T <: Number {
  magnitude { ... }
}
```

Such an implementation is **conditional**.

Its members are available only for receiver types for which the implementation's constraints are proven.

Conceptually:

```text
receiver exact/static type evidence
    +
impl target substitution
    +
impl constraints
    ↓
implementation applicable / not applicable
```

For a receiver of type `Point<T>`, `magnitude` is available if the typing environment proves the constraints required by the implementation.

If no such proof exists, the member is not part of the receiver's available surface merely because a runtime value might happen to satisfy the condition.

> **Conditional-availability invariant**
>
> Conditional inherent behavior is selected from static type evidence, not runtime probing.

## Constraint language

Implementation constraints use Phalcom's general generic-constraint system.

The meaning of `<:`, `:>`, equality constraints, trait constraints, and other constraint forms is defined by the type-system specifications.

The implementation declaration defines only how such proven constraints affect applicability.

## Reserved target-pattern specialization

This specification does not define concrete or non-covering target-pattern heads such as:

```phalcom
impl Point<Int> {
  ...
}
```

or:

```phalcom
impl<T> Pair<T, T> {
  ...
}
```

Specialization by target-pattern matching requires a separate language rule.

Conditional inherent behavior in this specification is expressed by constraints over a covering nominal target head.

---

# Overlap and Coherence

Two conditional inherent implementations may have overlapping applicability domains.

Overlap is permitted when their contributed selectors are disjoint.

Example:

```phalcom
impl<T> Box<T> where T <: Number {
  magnitude { ... }
}

impl<T> Box<T> where T == Int {
  bitWidth { ... }
}
```

If both constraints hold for `Box<Int>`, both members may be available because they do not compete for the same selector.

Competing definitions are not permitted:

```phalcom
impl<T> Box<T> where T <: Number {
  describe { ... }
}

impl<T> Box<T> where T == Int {
  describe { ... }
}
```

If both implementations can apply to an exact target, the program is invalid.

The language does not choose the more specific-looking declaration.

There is no inherent-impl specialization order in this specification.

> **Coherence invariant**
>
> Overlapping applicability may compose disjoint behavior, but an exact target and selector can never have two competing applicable inherent definitions.

---

# `Self`

Inside an inherent implementation, `Self` denotes the semantic implementation target under the current implementation substitution.

```phalcom
impl<T> Point<T> {
  clone -> Self { ... }
}
```

Here, `Self` denotes `Point<T>` in the implementation environment.

For an exact enum case implementation:

```phalcom
impl Expression::Literal(_) {
  ...
}
```

`Self` denotes the exact `Expression::Literal(_)` case under the case's specialized type environment.

`Self` never denotes the implementation declaration itself.

> **`Self` invariant**
>
> `Self` is target-relative, not provenance-relative.

---

# Visibility

Members declared in an inherent implementation use the ordinary Phalcom visibility system.

The implementation declaration itself does not introduce an independently accessible namespace whose visibility controls all members as one unit.

Its body is a declaration context; each contributed member has the accessibility determined by the ordinary rules for that member and its source module.

The target's own accessibility and the accessibility of types mentioned by a member signature continue to constrain the resulting API under the general visibility rules.

Authorization to write the `impl` does not imply that every member declared inside it is publicly accessible.

---

# Member Families and Callable References

An inherent `impl` participates in the existing Phalcom member-family system.

It does not create a second overload model.

Method-family rank, selector families, callable references, and generic callable identity remain governed by the ordinary callable specifications.

```text
callable-family polymorphism
    ≠
implementation applicability
```

A constrained implementation can determine whether a selector is available for a target; it does not alter the identity rules of the callable family containing that selector.

Instance method references continue to use the ordinary reference syntax, for example:

```phalcom
&object.method(_)
```

Associated constructor references retain the associated-callable syntax defined for their declaration category.

---

# Interaction with Data Objects

A data declaration owns the complete semantic component state of its values.

An inherent implementation may add behavior:

```phalcom
data Point(
  x: Int,
  y: Int,
)

impl Point {
  magnitude { ... }
}
```

It may not add components or redefine component selectors.

The data-object specification defines construction, immutability, exactness, and representation semantics.

---

# Interaction with Enums

An implementation targeting an enum root contributes enum-root behavior:

```phalcom
impl Expression {
  evaluate -> Int
}
```

An implementation targeting an exact case contributes exact-case behavior:

```phalcom
impl Expression::Literal(_) {
  evaluate -> Int { value }
}
```

The enum specification defines the enum-specific meanings of:

- root requirements;
- root defaults;
- exact-case witnesses;
- exact-case overrides;
- case-only members;
- GADT-specialized compatibility.

The present specification continues to govern authorization, selectors, generic scope, applicability, conflicts, and provenance.

---

# Interaction with Classes

For a class target, an inherent implementation contributes ordinary target-owned behavior.

It does not reopen class representation.

Fields, superclass relationships, and other representation-defining class declarations cannot be introduced through `impl`.

Members written directly in the class body and members written in inherent implementations participate in the same selector namespace.

---

# Interaction with Traits

The form:

```phalcom
impl Trait for Target {
  ...
}
```

is a conformance implementation.

It does not by itself add the trait's requirements or defaults to the target's inherent member surface.

The conformance has semantic identity distinct from:

- the target declaration;
- the trait declaration;
- the `impl` provenance identity;
- the concrete witness callables.

The trait specification defines how the body supplies or selects witnesses and associated bindings.

---

# Declaration Identity and Provenance

Every implementation declaration has stable source/provenance identity.

That identity supports diagnostics, reflection, incremental analysis, source navigation, and tooling.

Implementation identity remains distinct from member identity.

```text
implementation identity
    source + target + generic environment + constraints

callable identity
    semantic owner + selector + dispatch role
```

A member can therefore remain the same kind of target-owned callable regardless of whether its implementation was written directly or in a separate `impl` fragment.

For conformance implementations, conformance identity is additionally distinct from both implementation provenance and individual witnesses.

---

# Reflection

Where reflection exposes implementation declarations, it preserves the distinction between target ownership and implementation provenance.

Reflection may expose:

- the target of an implementation;
- whether it is inherent or conforming;
- implementation-local generic parameters;
- declared constraints;
- members contributed by an inherent implementation;
- the implementation provenance of a target member;
- a conformance relationship established by a conformance implementation.

Reflecting an `impl` reports static semantic structure.

It does not install, remove, or mutate behavior.

> **Reflection invariant**
>
> Implementation provenance may be observable without becoming callable ownership or runtime mutation authority.

---

# Compilation and Runtime Freedom

The language does not prescribe how an implementation stores or dispatches inherent behavior.

A compiler may:

- merge inherent surfaces during semantic analysis;
- maintain indexed implementation contributions;
- specialize constrained lookup;
- build dispatch tables;
- lower calls directly;
- eliminate implementation metadata where it is not observably required.

All such strategies must preserve target ownership, selector coherence, static applicability, visibility, and reflection guarantees.

Equivalent programs cannot change behavior according to compilation order, import traversal order, or cache state.

---

# Invalid Programs

## Unauthorized inherent implementation

An inherent implementation declared outside the target's owning module is invalid.

## Invalid target

A target that is not an eligible nominal target or exact enum case is invalid.

## Structural target

An anonymous structural type cannot be used as an inherent target.

## Representation contribution

An implementation cannot declare fields, data components, enum variants, superclass relationships, or other semantic representation state.

## Duplicate selector

Two applicable concrete inherent definitions of the same selector for the same exact target are invalid.

## Component collision

An implementation member cannot collide with a data component's reserved selector.

## Associated-constructor collision

An implementation member cannot hide or replace a primary data constructor or enum variant constructor where the associated-member namespace would collide.

## Invalid generic mapping

Implementation-local generic parameters must form a valid target application under the ordinary generic rules.

## Invalid constraint

Implementation constraints must be well-formed in the implementation's generic environment.

## Ambiguous conditional applicability

If two conditional implementations can both supply the same selector for one exact target, the program is invalid.

## Reserved specialization form

A concrete or non-covering target-pattern implementation head is invalid unless another specification introduces that form.

## Invalid exact-case target

An exact-case target must resolve to a valid variant shape under the enum specification.

## Invalid conformance

A conformance implementation that violates the trait specification is invalid.

---

# Diagnostics

Diagnostics distinguish, where practical, among:

- unauthorized implementation;
- unresolved or illegal target;
- structural target use;
- illegal representation member;
- duplicate selector in one implementation;
- duplicate selector across implementation fragments;
- direct-member/impl conflict;
- data-component/member collision;
- enum-constructor/member collision;
- invalid generic implementation head;
- unsatisfied conditional applicability at a use site;
- ambiguous overlapping inherent definitions;
- invalid exact enum-case target;
- invalid trait conformance.

Diagnostic codes and wording are implementation-defined, but diagnostics must not erase the underlying semantic distinction between these failures.

---

# Non-Normative Implementation Model

This section is non-normative.

A compiler may model inherent behavior conceptually as:

```text
DeclaredSurface
    members written on the primary declaration

ImplContribution
    members written in one inherent impl

EffectiveSurface
    coherent target-owned composition
```

Conditional implementations may be indexed separately and selected from static type evidence.

An internal implementation identity may retain:

```text
source location
target identity
generic-owner identity
target substitution
constraints
```

while each callable remains identified by its semantic owner and selector.

The exact compiler products are not part of Phalcom source semantics.

---

# Reserved Facilities

The following facilities are not defined by this specification:

- inherent implementations directly targeting anonymous structural types;
- concrete target-pattern specialization such as `impl Point<Int>`;
- non-covering generic target patterns such as `impl<T> Pair<T, T>`;
- inherent-implementation specialization or “most specific impl” selection;
- metatype-target `impl` syntax;
- runtime open-class or monkey-patching behavior.

A future specification may introduce any of these deliberately. Their absence does not weaken the behavior-extension model defined here.

---

# Governing Guarantees

> **Target ownership**
>
> Behavior declared by an inherent `impl` belongs semantically to the target.

> **Provenance separation**
>
> The implementation declaration retains source identity without becoming the semantic owner of its members.

> **Static composition**
>
> Inherent implementations are static program declarations, never runtime reopening events.

> **Module ownership**
>
> Only the module that owns a nominal target may declare its inherent implementations.

> **Nominal targeting**
>
> Inherent implementations target nominal declarations or exact enum cases, not anonymous structural types.

> **Representation preservation**
>
> `impl` may add behavior but cannot add semantic instance representation or inheritance structure.

> **Selector coherence**
>
> Member identity remains selector-shaped; parameter and result types do not create implicit overloads.

> **No last-wins behavior**
>
> Source order, import order, and discovery order never resolve conflicting inherent definitions.

> **Generic scope fidelity**
>
> Implementation-local generic parameters have their own semantic identities and map explicitly into the target.

> **Conditional availability**
>
> Members from constrained implementations are available only when the static type environment proves applicability.

> **Disjoint overlap**
>
> Conditional implementations may overlap in target applicability only so far as they do not produce competing concrete definitions of the same selector.

> **No implicit specialization**
>
> More specific-looking constraints do not override broader implementations automatically.

> **Exact-case targeting**
>
> Enum variants may serve as exact-case implementation targets without becoming callable references or subclasses.

> **Conformance separation**
>
> `impl Trait for T` establishes trait conformance rather than merely adding inherent behavior.

> **Reflection fidelity**
>
> Reflection preserves the distinction among target identity, member identity, implementation provenance, and conformance identity.

The governing rule is:

> **An inherent implementation extends the behavior of an existing nominal target without changing what that target is, how its representation is defined, or who owns its behavior. Applicability and coherence are determined statically from target identity, generic substitution, constraints, and selectors.**
