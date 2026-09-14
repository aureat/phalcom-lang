# Phalcom Language Specification — Enumerated Types and Variants

## Overview

An `enum` declaration introduces a **nominal closed sum type**. The declaration defines the complete set of variants that constitute the sum. Each variant has its own semantic identity and may carry a payload whose components form the variant's case-specific product state.

```phalcom
enum Expression {
  Literal(_ value: Int)
  Add(_ left: Expression, _ right: Expression)
}
```

`Expression` is the enum root type. `Expression::Literal(_)` and `Expression::Add(_, _)` identify distinct variants and induce distinct exact-case semantics.

An enum declaration defines **which cases exist**. It does not define behavioral members inside the enum body. Behavior is declared separately through implementation declarations.

```phalcom
impl Expression {
  evaluate -> Int

  precedence -> Int {
    100
  }
}

impl Expression::Literal(_) {
  evaluate -> Int {
    value
  }
}

impl Expression::Add(_, _) {
  evaluate -> Int {
    left.evaluate + right.evaluate
  }

  precedence -> Int {
    10
  }
}
```

The enum root may declare closed-case requirements and defaults through an inherent `impl`. Exact-case implementations may satisfy those requirements, override defaults, or introduce behavior that exists only for that exact case.

> **Core invariant**
>
> An enum is a nominal closed sum. Its declaration owns the closed set of variants; its implementation declarations own behavior. Variant identity, exact-case type identity, behavior identity, and physical representation are distinct concepts.

---

# Semantic Category

## Enum declarations

An **enum declaration** introduces a nominal enum type and its complete set of variants.

```phalcom
enum Result<T, E> {
  Ok(_ value: T)
  Error(_ error: E)
}
```

The declaration establishes:

- the nominal identity of `Result`;
- the enum's generic parameters;
- the closed set of variants;
- the payload shape of each variant;
- any variant-local generic parameters;
- any variant-specific result specialization permitted by the type system.

The declaration body contains variant declarations only.

Behavioral methods, getters, setters, index behavior, root requirements, and root defaults are not declared inside the enum body. They are expressed through `impl`.

> **Variants-only invariant**
>
> The enum body defines the closed sum. It does not serve as a mixed representation-and-behavior body.

## Enum root type

The type introduced by the enum declaration is the **enum root type**.

For:

```phalcom
enum Option<T> {
  Some(_ value: T)
  None
}
```

`Option<T>` is the enum root type.

The root type denotes values that may belong to any variant allowed by the exact applied enum type.

A root-typed value is not assumed to belong to a particular variant unless semantic analysis, construction, or control-flow refinement proves an exact case.

## Variants

A **variant** is one alternative of an enum's closed sum.

For:

```phalcom
enum Result<T, E> {
  Ok(_ value: T)
  Error(_ error: E)
}
```

`Ok` and `Error` are distinct variants.

A variant has stable semantic identity independent of:

- the shape of its payload;
- its runtime tag encoding;
- its physical payload layout;
- the callable object used to construct it;
- any reflected descriptor used to inspect it.

Two variants remain distinct even when they carry payloads of identical type and shape.

```phalcom
enum Token {
  Identifier(_ text: String)
  Keyword(_ text: String)
}
```

`Token::Identifier(_)` and `Token::Keyword(_)` are different variants despite carrying the same payload type.

> **Variant identity invariant**
>
> Variant identity is semantic. Equal payload layout does not imply equal variant identity.

## Exact-case types

A variant induces an **exact-case type**: a type-state in which the receiver is known to belong to one specific variant of the enum.

Conceptually, the distinction is:

```text
Expression
    enum root type

Expression::Literal(_)
    exact case

Expression::Add(_, _)
    exact case
```

Exact-case types participate in:

- variant payload access;
- case-specific behavior lookup;
- pattern-matching refinement;
- GADT result specialization;
- variant-local generic reasoning;
- case-specific conformance or behavior where separately permitted.

An exact-case type is not a separate nominal declaration unrelated to the enum. It is the exact variant-specific type state of the enum value.

> **Exact-case guarantee**
>
> When the type system proves an exact case, case-specific type information and behavior become available without promoting those members to the enum root.

## Applicable exact cases

For a particular exact applied enum root type, an **applicable exact case** is a variant case whose result specialization and declared constraints admit an instantiation of that root type. Ordinary non-GADT variants are applicable whenever they belong to that exact applied enum. A GADT-style variant whose result equations make the case impossible for that root application is not applicable to that application.

Closed-case behavior completeness is quantified over applicable exact cases. This prevents an impossible GADT case from creating a witness obligation while preserving the closed-world requirement for every inhabitable case.

---

# Closed Sum Semantics

## The variant set is closed

The variants declared in an enum declaration form the complete variant set of that enum.

```phalcom
enum Direction {
  North
  South
  East
  West
}
```

No `impl`, import, extension module, trait conformance, runtime reflection operation, or later declaration may add another variant to `Direction`.

The set of variants is therefore available to static reasoning such as exhaustiveness analysis.

> **Closed-world invariant**
>
> The variant set belongs exclusively to the enum declaration and cannot be extended by behavior declarations.

## Variant order and identity

Source declaration order may be observable where another Phalcom specification explicitly assigns significance to it, such as reflection presentation or derived ordering policy.

Variant semantic identity does not depend on a particular runtime discriminant value.

An implementation may assign physical tags, reorder internal tables, or use specialized encodings as long as all language-defined observations remain unchanged.

The source declaration must therefore not be interpreted as defining a stable ABI discriminant unless another specification explicitly establishes such a guarantee.

---

# Variant Payloads

## Product-shaped payloads

A variant may carry a payload.

```phalcom
enum Expression {
  Literal(_ value: Int)
  Add(_ left: Expression, _ right: Expression)
}
```

The payload is product-shaped semantic state belonging to that exact case.

`Expression::Literal(_)` carries one component:

```text
value
```

`Expression::Add(_, _)` carries:

```text
left
right
```

Payload components are part of the exact case's semantic state.

They are not ordinary hidden mutable fields injected into the enum root.

## Payload components

A payload component has semantic identity within its variant.

Its name, position, type, and argument shape are determined by the variant declaration.

Payload components may be made available to exact-case behavior and pattern bindings according to the ordinary rules of the language.

An implementation may use shared product-storage machinery for enum payloads, data objects, Tuple, or Record without collapsing their semantic categories.

> **Payload invariant**
>
> Variant payload state belongs to the exact case. Shared physical product representation does not turn the variant into `data`, Tuple, or Record.

## Logical payload order

The variant declaration determines logical payload component order.

Physical storage order is not part of the semantic contract unless another specification explicitly makes it observable.

Construction and pattern matching operate on logical component identity and argument shape rather than physical offsets.

---

# Variant Construction and Associated Lookup

## Associated construction

A payload-bearing variant is constructed through associated lookup on the enum declaration.

```phalcom
Option::Some(42)
```

```phalcom
Result::Error(error)
```

The associated constructor belongs semantically to the variant.

A variant constructor is not an ordinary class-side method merely because it is callable.

Its identity remains variant/associated-constructor identity.

## Callable references

Construction and callable reference are distinct operations.

Construction:

```phalcom
Option::Some(42)
```

Callable reference:

```phalcom
&Option::Some(_)
```

or, for a compatible variadic shape:

```phalcom
&Option::Some(...)
```

The `&` form denotes a reference to the associated callable.

The non-`&` form invokes construction.

> **Syntax invariant**
>
> `Option::Some(_)` in an exact-case `impl` target is not a callable reference. `&Option::Some(_)` is a callable reference.

## Constructor identity and variant identity

A constructor provides a way to create a variant value. It does not replace the semantic identity of the variant.

Conceptually:

```text
variant identity
    ≠
constructor callable identity
```

Reflection may expose both where appropriate.

## Constructor evaluation order

Argument expressions used to construct a variant follow ordinary Phalcom call-expression evaluation order.

Physical payload layout must not reorder observable source evaluation.

For example, if labeled arguments permit:

```phalcom
Pairish::Case(
  second: side2(),
  first: side1(),
)
```

then `side2()` executes before `side1()` when that is the source evaluation order, regardless of payload storage order.

---

# Nullary and Singleton Variants

A variant may carry no payload.

```phalcom
enum Option<T> {
  Some(_ value: T)
  None
}
```

A nullary variant denotes the single value of that exact variant for the corresponding exact applied enum type.

A nullary variant is obtained through getter-shaped associated lookup. For example, `Option::None` evaluates to the singleton value of the `None` case for the inferred or explicitly supplied exact `Option<T>` application. It is not invoked as a nullary ordinary method.

The semantic value is determined by:

- the exact applied enum type; and
- the variant identity.

The implementation is not required to allocate an ordinary heap object for each occurrence of a nullary variant.

> **Singleton-case guarantee**
>
> A nullary variant denotes one semantic case value per exact applied enum type and variant identity; this does not imply a unique heap allocation.

The exact spelling and associated-reference behavior of nullary variant constructors follow the language's associated-member specification.

---

# Generic Enums

Enums participate in the ordinary Phalcom generic type system.

```phalcom
enum Result<T, E> {
  Ok(_ value: T)
  Error(_ error: E)
}
```

Different exact applications are distinct exact semantic types.

```text
Result<Int, String>
Result<String, Int>
```

are not interchangeable merely because individual runtime values may use similar payload layouts.

All semantically relevant generic arguments contribute to exact applied type identity according to the general type system.

## Generic argument preservation

A runtime value whose exact applied enum type is semantically known must preserve that exact type across materialization and runtime observation boundaries.

A generic argument cannot be silently replaced by `Dynamic`, erased to the bare declaration, or reconstructed from payload runtime classes when semantic analysis already determined the exact application.

> **Reification invariant**
>
> Exact enum type information must be preserved when established by language semantics; runtime payload inspection must not invent a more precise static type than was proven.

---

# Variant-Local Generics

A variant may introduce generic parameters local to that variant where permitted by the Phalcom type system.

Variant-local generics participate in the exact-case type environment.

They may affect:

- payload component types;
- exact-case refinement;
- GADT result specialization;
- root-requirement specialization;
- exact-case behavior checking.

Variant-local generic parameters are scoped according to the variant declaration and are not automatically parameters of unrelated variants.

> **Locality invariant**
>
> A variant-local generic parameter contributes to that variant's exact-case semantics without becoming a new parameter of the enum root as a whole.

The exact declaration syntax and generic constraint grammar follow the general generic-type specification.

---

# GADT and Result Specialization

Phalcom enums may express variant-specific result specialization where supported by the type system.

Such enums require exact-case reasoning that is stronger than ordinary tagged-union checking.

Pattern matching or exact construction may refine:

- the active variant;
- generic substitutions;
- result-type equations;
- variant-local generic bindings;
- other case-specific type facts.

These refinements form the **case type environment**.

Exact-case behavior must be checked under that environment.

> **GADT specialization guarantee**
>
> Case-specific behavior and requirement witnessing are validated after applying the exact case's type refinements, not by comparing unspecialized root signatures mechanically.

The specification of the general type solver, equality constraints, and generic inference remains authoritative for how such refinements are represented and proven.

---

# Pattern Matching

## Variant patterns

Pattern matching decomposes enum values by semantic variant identity.

A variant pattern selects a specific variant and may bind payload components.

Conceptually:

```phalcom
match expression {
  Expression::Literal(value) => ...
  Expression::Add(left, right) => ...
}
```

The exact pattern syntax is governed by the pattern-matching specification.

The semantic rule is that successful matching against a variant pattern refines the matched value to the corresponding exact case within the branch.

## Case refinement

Inside a branch that proves an exact case, the type system may expose:

- the variant payload;
- exact-case-only members;
- GADT refinements;
- variant-local generic facts.

For example, after proving an `Expression::Literal(_)` case, behavior declared only for that case may become statically available.

## Exhaustiveness

Because an enum's variant set is closed, the compiler may determine whether a match covers every possible case.

Exhaustiveness operates over semantic variants and their type refinements, not over runtime discriminant integers.

For GADTs, exhaustiveness may consider case impossibility established by type constraints.

The exact exhaustiveness algorithm is not part of the source-language contract, but it must preserve the language's closed-sum and type-refinement semantics.

## Unreachable cases

Where type refinement proves that a variant cannot occur for a particular scrutinee type, the corresponding pattern may be diagnosed as unreachable according to the general pattern-matching rules.

Moving behavior out of enum declarations does not change pattern semantics.

> **Separation invariant**
>
> Pattern matching decomposes the closed sum. `impl` declarations contribute behavior. Neither mechanism defines the other.

---

# Behavior Outside the Enum Declaration

Enum behavior is declared using implementation declarations.

The enum declaration remains variants-only.

For the root:

```phalcom
impl Expression {
  evaluate -> Int

  precedence -> Int {
    100
  }
}
```

For an exact case:

```phalcom
impl Expression::Literal(_) {
  evaluate -> Int {
    value
  }
}
```

```phalcom
impl Expression::Add(_, _) {
  evaluate -> Int {
    left.evaluate + right.evaluate
  }
}
```

The general rules governing implementation ownership, member selectors, visibility, conflicts, and generic applicability are defined by the implementation-declaration specification.

This enum specification defines the enum-specific semantic interpretation of root and exact-case behavior.

---

# Receiver and Payload Scope in Enum Implementations

An `impl` targeting the enum root is checked with `Self` denoting the enum-root target under the implementation's generic substitution. A root implementation body has only the behavior and state available through the enum root; it cannot assume a particular variant payload without refinement.

An `impl` targeting an exact case is checked with `Self` denoting that exact-case target. The exact case's payload components participate in ordinary implicit-receiver member lookup inside the implementation body. Thus, for:

```phalcom
impl Expression::Literal(_) {
  evaluate -> Int { value }
}
```

`value` denotes the `Literal` case's payload component on `self`; it is not a new lexical variable introduced by the `impl` header. The same component remains accessible through the ordinary explicit receiver form where that form is permitted by Phalcom member syntax.

Variant-local generic parameters and GADT refinements that belong to the exact case are in the implementation's case type environment.

> **Receiver-context invariant**
>
> Root implementations are checked against root semantics; exact-case implementations are checked against exact-case semantics. Payload access never leaks from an exact case onto the root.

---

# Enum Root Behavior

## Root member declarations

An inherent implementation targeting the enum root declares behavior on the root type.

```phalcom
impl Expression {
  evaluate -> Int
  precedence -> Int { 100 }
}
```

A root member is explicitly part of the enum root's behavior surface.

A root member may be invoked on a receiver typed as the enum root, subject to ordinary visibility and member-resolution rules.

Exact-case behavior that corresponds to that root member participates in closed-case dispatch.

## Root requirements

A root behavior declaration without a body is a **closed enum root requirement**.

```phalcom
impl Expression {
  evaluate -> Int
}
```

The requirement states that every applicable exact case of `Expression` must provide a compatible implementation.

Because the enum's variants are closed, this requirement is checked over the closed set of cases.

A root requirement is not a trait requirement and does not create a reusable conformance relationship.

> **Closed-contract invariant**
>
> An enum root requirement quantifies over the closed cases of one enum. It is not a hidden or synthesized trait.

## Root defaults

A root behavior declaration with a body is a **root default implementation**.

```phalcom
impl Expression {
  precedence -> Int {
    100
  }
}
```

The default supplies behavior for exact cases that do not provide a more specific exact-case implementation of the same root member.

A root default remains semantically associated with the enum root contract.

It is not copied into each variant as an unrelated callable declaration.

## Root behavior identity

A root requirement or root default has semantic identity distinct from each exact-case callable that may satisfy or override it.

Conceptually:

```text
Expression.evaluate
    root requirement identity

Expression::Literal.evaluate
    exact-case callable identity

Expression::Add.evaluate
    exact-case callable identity
```

The exact-case callables may witness the root requirement, but they are not the same semantic entity.

---

# Exact-Case Implementation Declarations

An implementation may target a specific variant shape.

```phalcom
impl Expression::Literal(_) {
  ...
}
```

```phalcom
impl Expression::Add(_, _) {
  ...
}
```

The target denotes exact-case semantics.

It does not invoke the constructor and does not form a callable reference.

The implementation may declare behavior that:

- satisfies a root requirement;
- overrides a root default;
- introduces case-only behavior.

## Requirement witnesses

If the enum root declares:

```phalcom
impl Expression {
  evaluate -> Int
}
```

then:

```phalcom
impl Expression::Literal(_) {
  evaluate -> Int {
    value
  }
}
```

provides an exact-case implementation that may serve as the witness for the `Literal` case.

Every applicable exact case must have a valid witness for every applicable root requirement.

A missing witness makes the enum's behavioral contract incomplete and is a static error.

## Default overrides

If the root declares a default:

```phalcom
impl Expression {
  precedence -> Int {
    100
  }
}
```

an exact case may provide its own implementation:

```phalcom
impl Expression::Add(_, _) {
  precedence -> Int {
    10
  }
}
```

The exact-case member overrides the root default for that exact case.

Other exact cases continue to use the root default unless they provide their own override.

## Case-only behavior

An exact-case implementation may declare behavior absent from the enum root.

```phalcom
impl Expression::Literal(_) {
  literalValue {
    value
  }
}
```

`literalValue` is available only when the receiver is statically known to satisfy the applicable exact case.

It is not a member of the enum root.

Even if every currently declared variant independently declares a member with the same selector, that member does not become a root member unless it is explicitly declared on the root.

> **Root-surface invariant**
>
> Root behavior exists only by explicit root declaration. Repetition across cases never structurally promotes case-only behavior to the enum root.

---

# Closed-Case Requirement Satisfaction

## Completeness

For every root requirement, each applicable exact case must provide a compatible witness.

Given:

```phalcom
enum Expression {
  Literal(_ value: Int)
  Add(_ left: Expression, _ right: Expression)
}

impl Expression {
  evaluate -> Int
}
```

the behavioral definition is incomplete unless both exact cases have valid witnesses:

```phalcom
impl Expression::Literal(_) {
  evaluate -> Int { value }
}

impl Expression::Add(_, _) {
  evaluate -> Int { left.evaluate + right.evaluate }
}
```

A root default removes the need for each exact case to provide a separate implementation unless another rule requires an override.

## Compatibility is semantic, not textual

A case implementation does not satisfy a root requirement merely because the member names look alike.

Compatibility is determined according to:

- selector identity;
- dispatch side;
- parameter and result compatibility;
- visibility/accessibility;
- exact-case specialization;
- generic substitutions;
- variant-local generics;
- GADT result specialization;
- any other requirements of the general callable type system.

The selector rules remain those of ordinary Phalcom member identity.

## Specialize before checking

For generic or GADT enums, the root requirement must be specialized to the exact case before compatibility is judged.

Conceptually:

```text
root requirement
    +
exact-case type environment
    +
variant-local generic substitutions
    +
result specialization
    ↓
specialized requirement
    compared with
exact-case implementation
```

Comparing an unspecialized raw root signature directly against the case implementation is not semantically sufficient.

> **Witness-specialization invariant**
>
> Root-to-case compatibility is checked in the exact case's specialized type environment.

---

# Dispatch Across the Enum Root

A root-declared member establishes behavior that may be used through the enum root.

When invoked on a root-typed receiver, the selected behavior depends on the receiver's exact variant according to the enum closed-contract rules:

- use the exact-case implementation where one exists for the root member;
- otherwise use the root default where one exists;
- a root requirement without a witness is an invalid program and therefore cannot produce a valid runtime dispatch case.

The language semantics are equivalent to closed-case dispatch over the variant set.

The implementation is not required to perform a source-level scan of exact-case `impl` declarations at runtime.

It may use dispatch tables, compiled case switches, specialized entry points, or other equivalent mechanisms.

> **Dispatch guarantee**
>
> Runtime dispatch strategy may vary, but it must preserve the statically established root/default/exact-case behavior relation.

---

# Case-Only Member Availability

A case-only member is available only when static typing establishes the required exact case or another separately specified refinement proves applicability.

For example:

```phalcom
impl Expression::Literal(_) {
  literalValue { value }
}
```

permits:

```text
literal.literalValue
```

when `literal` is statically known to be `Expression::Literal(_)`.

The same member is not available on an arbitrary `Expression`.

Pattern matching is one way to obtain such refinement.

Case-only behavior is not structural duck typing over the closed set of variants.

> **Availability guarantee**
>
> Exact-case-only behavior follows exact-case typing. It is never inferred onto the root from coincidental member repetition.

---

# Member Identity and Selector Rules

Enum behavior uses the ordinary Phalcom selector model.

Selector identity is determined by callable shape, including the language-defined combination of:

- base name;
- positional arity;
- labels;
- variadic or pack shape where applicable;
- getter/setter/index/method category;
- dispatch side.

Parameter types and return types do not independently create selector overloading.

Therefore, two incompatible concrete declarations with the same selector are conflicts rather than last-definition-wins overloads unless a separately specified dispatch mechanism applies.

## Variant-associated constructor conflicts

Variant constructors retain associated-callable identity and do not become ordinary class-side methods.

An implementation declaration cannot silently hide or replace a variant constructor by declaring a conflicting associated member with the same selector.

Such collisions are invalid according to the general implementation/member conflict rules.

> **Associated-member invariant**
>
> Variant constructors remain variants/associated callables even when ordinary behavior shares nearby namespace syntax.

---

# Enum Contracts Are Not Traits

Closed enum contracts and traits solve different problems.

An enum root requirement:

```phalcom
impl Expression {
  evaluate -> Int
}
```

means:

```text
every applicable exact case of this closed enum
must provide evaluate
```

A trait requirement means:

```text
any type that explicitly conforms to this reusable trait
must satisfy the requirement
```

These are not interchangeable.

An enum contract:

- is owned by one enum;
- ranges over that enum's closed variant set;
- participates in exact-case dispatch and GADT specialization;
- does not create reusable conformance evidence for unrelated types.

A trait:

- defines a reusable behavioral contract;
- may be conformed to by separately declared types;
- has separate conformance identity and witness evidence.

> **Semantic separation invariant**
>
> A closed enum requirement must not be modeled as an automatically generated trait, and trait conformance must not be used as a substitute for closed-case completeness.

---

# Generic and GADT Behavior Contracts

## Specialized root requirements

If the enum root is generic, a root requirement may itself contain references to root generic parameters.

Each exact case is checked under the exact applied enum type and the case-specific substitutions established by its variant.

## Variant-local generic witnesses

A witness declared in an exact-case `impl` may use variant-local generic parameters that are valid for that case.

Compatibility is judged after those parameters and case equations are incorporated into the case type environment.

## Result-specialized cases

For a GADT-style variant that specializes the enum's result type, the exact-case implementation is checked against the root requirement as specialized for that result.

The language must not reject a sound witness merely because the unspecialized root signature appears different before the case equations are applied.

Conversely, case specialization must not be used to accept an implementation that is incompatible after the actual exact-case substitutions are applied.

---

# Inherent Implementation Ownership

Enum root and exact-case `impl` declarations follow the general ownership/coherence rules of inherent implementation declarations.

They do not create open enums or runtime extension semantics.

Loading another module must not mutate the canonical enum behavior surface through import order.

An exact-case `impl` may contribute behavior to the exact case, but it cannot:

- introduce a new variant;
- change the variant payload shape;
- change the enum's generic parameter list;
- alter another variant;
- mutate the enum's closed-set identity.

> **Ownership invariant**
>
> `impl` may extend behavior within the language's authorized ownership boundary; it can never extend the enum's variant set or representation definition.

---

# Representation Independence

Enum semantics do not prescribe one physical representation.

An implementation may use:

- tagged values;
- boxed payloads;
- shared product storage;
- immediate encodings;
- specialized singleton encodings;
- optimized special cases;
- scalar replacement where semantically valid;
- other equivalent representations.

The physical representation of a variant does not define its semantic identity.

The following concepts remain distinct:

```text
enum declaration identity
variant identity
exact-case type identity
runtime variant identity
physical discriminant
payload layout
reflection descriptor identity
```

An implementation may map several semantic cases to similar layouts or optimize a particular enum such as `Option` using a specialized representation.

Such choices must preserve the enum's source-level and reflective semantics.

> **Representation-freedom guarantee**
>
> Enum meaning is defined by nominal type, variant identity, payload values, and established behavioral contracts—not by a particular runtime tag or payload layout.

---

# Runtime Discriminants

A physical case discriminant is an implementation mechanism.

Unless another specification explicitly defines an ABI or serialization mapping, programs cannot rely on:

- a variant's numeric runtime tag;
- declaration-order-derived tag values;
- payload address layout;
- box identity;
- internal object category.

Pattern matching, reflection, and dispatch operate on semantic variant identity even when the implementation lowers them to physical tags.

---

# Specialized Representations

A conforming implementation may use a representation specialized for a particular enum where semantics are preserved.

For example, an option-like enum may use a native or compact representation rather than the general enum-payload representation.

Specialized representation does not create a second language category.

It must preserve:

- nominal enum identity;
- exact applied type identity;
- variant identity;
- pattern behavior;
- exact-case refinement;
- root/default/case dispatch;
- reflection guarantees;
- generic reification.

> **Optimization invariant**
>
> Representation specialization may eliminate general enum machinery. It may not eliminate enum semantics.

---

# Exact Type Reification and `Dynamic`

An enum value crossing a runtime observation boundary must preserve the exact semantic type information established by the type system.

For generic enums, this includes exact generic arguments.

A value of exact semantic type:

```text
Result<Int, String>
```

must not become merely:

```text
Result
```

or be widened to an invented:

```text
Result<Dynamic, Dynamic>
```

solely because it crosses into `Dynamic`.

Likewise, the active semantic variant must remain truthfully observable where reflection or dynamic operations expose it.

Runtime classes or payload values must not be used to reverse-engineer static generic arguments when semantic inference already established the relevant exact type.

> **Dynamic-boundary guarantee**
>
> `Dynamic` may limit static knowledge at a use site; it does not erase or fabricate the enum value's actual semantic type or variant identity.

---

# Reflection

Where Phalcom reflection exposes enum semantics, it must preserve the distinctions defined by this specification.

Reflection must be capable, through the broader reflection model, of distinguishing:

- the enum declaration;
- an exact applied enum type;
- a variant declaration;
- an exact-case type or equivalent case descriptor;
- variant payload components;
- root requirements;
- root defaults;
- exact-case implementations;
- witness relationships between root requirements and exact-case implementations;
- inherent implementation provenance.

A root requirement and its concrete witness are distinct reflected semantic entities.

Conceptually:

```text
Expression.evaluate
    root requirement

Expression::Literal.evaluate
    concrete callable

witness relation
    links them
```

Likewise, a root default retains its own provenance even where a particular exact case selects it.

Reflection descriptors report semantic facts; they do not become the authority that creates those facts.

The caching, lifetime, or allocation identity of reflection wrappers must not alter enum semantics.

> **Reflection invariant**
>
> Reflection must preserve declaration, variant, requirement, witness, and implementation-provenance distinctions rather than flattening them into one callable list.

---

# Exactness, Equality, and Hashing

This specification does not redefine the language-wide semantics of `===`, `==`, hashing, or derived behavior for enum values unless those operations are defined by the general value or enum relation specifications.

However, any such operation must obey representation independence.

A conforming operation must not distinguish otherwise semantically equivalent enum values merely because:

- different physical discriminants encode the same semantic variant across execution modes;
- one payload was boxed and another flattened;
- a specialized representation was used;
- immutable backing storage was shared;
- a value was rematerialized after optimization.

Where exactness observes enum case identity, it must observe semantic variant identity rather than raw runtime tag identity.

Where payload values participate, their comparison follows the applicable language-level relation rather than physical storage identity.

> **Representation-observation prohibition**
>
> Physical enum encoding is never itself a source-level equality or identity criterion unless another specification explicitly says otherwise.

---

# Exhaustiveness and Behavior Completeness Are Different Analyses

Two distinct forms of completeness exist for enums.

**Pattern exhaustiveness** asks whether a pattern match accounts for every possible semantic case.

**Behavior completeness** asks whether every applicable exact case satisfies every root behavior requirement.

The two analyses may use the same closed variant set but answer different questions.

A match may be exhaustive even if a root requirement is missing a case witness; the program is still invalid because the behavioral contract is incomplete.

Likewise, a complete set of witnesses does not make an unrelated pattern match exhaustive.

> **Analysis-separation invariant**
>
> Closed-sum exhaustiveness and closed-behavior completeness share enum case knowledge but remain distinct semantic judgments.

---

# Invalid Programs

The following categories are invalid under the enum semantics defined here.

## Behavior inside the enum body

Under the variants-only model, ordinary behavioral declarations do not belong inside the enum declaration body.

Legacy syntax, if temporarily supported for migration, is not part of the normative language model described by this specification.

## Adding variants through `impl`

An implementation declaration cannot add, remove, replace, or reorder the semantic variant set.

## Missing root requirement witness

If an applicable root requirement lacks a compatible witness for any applicable exact case and no root default supplies the behavior, the enum definition is behaviorally incomplete.

## Incompatible witness

An exact-case member with a matching selector does not satisfy a root requirement if it is incompatible after exact-case specialization, type substitution, visibility checking, or other callable-compatibility rules.

## Conflicting concrete definitions

Two incompatible concrete members with the same selector for the same effective target are invalid unless another language feature explicitly defines the conflict-resolution mechanism.

There is no implicit last-definition-wins rule.

## Accidental root promotion

Case-only behavior cannot be invoked on the root merely because all or many current variants independently provide similarly shaped members.

## Invalid exact-case target

An `impl` target purporting to identify an exact case must resolve to a valid variant shape of the enum under the implementation-declaration rules.

## Unauthorized inherent implementation

An enum root or exact-case implementation declared outside the language's authorized ownership boundary is invalid.

## Variant/associated-member collision

An implementation member that conflicts with a variant constructor or other associated member in a namespace where the language defines them as colliding is invalid.

## Invalid generic or GADT specialization

A case implementation that assumes type equations or generic substitutions not actually established by the exact case is invalid.

---

# Diagnostics

Diagnostics must distinguish materially different semantic failures.

Useful diagnostic categories include:

- behavior illegally declared inside a variants-only enum body;
- unresolved or malformed variant declaration;
- invalid variant constructor arguments;
- invalid exact-case `impl` target;
- missing exact-case witness for a root requirement;
- incompatible exact-case witness after specialization;
- duplicate or conflicting root member;
- duplicate or conflicting exact-case member;
- case-only member used through an insufficiently refined receiver;
- unauthorized inherent `impl`;
- associated-member/variant-constructor collision;
- unreachable enum pattern;
- non-exhaustive match.

The exact diagnostic codes and wording are implementation concerns, but diagnostics must preserve the semantic distinction between these failures.

A missing witness must not be reported merely as a generic “method not found” error when doing so would erase the stronger closed-world contract established by the root requirement.

---

# Non-Normative Implementation Model

This section is non-normative.

A compiler may represent enum semantics using products such as:

```text
root requirements
root defaults
case implementations
```

and retain explicit witness relationships between root contracts and case callables.

The implementation may assign stable internal identities corresponding conceptually to:

```text
EnumRequirementId
VariantId
RuntimeVariantId
CaseDiscriminant
CallableId
ImplId
ProductLayoutId
```

The exact names are implementation-specific.

The important architecture is separation of concerns:

```text
semantic enum/variant identity
        |
        +--> exact-case typing and GADT refinement
        |
        +--> root requirement/default model
        |
        +--> exact-case behavior and witness relation
        |
        +--> reflection products
        |
        v
compiler lowering
        |
        v
runtime dispatch and representation
```

The runtime may use a compact case tag and product payload, a specialized native option representation, direct compiled branching, or another model.

No implementation representation may become the language's semantic authority.

## Behavior-product migration

An implementation migrating from older behavior-bearing enum syntax may internally normalize both old and new source forms into the same semantic enum-behavior product during a compatibility period.

That is a migration technique, not a second semantic model.

The final normative source model remains:

```text
enum declaration
    variants only

enum-root impl
    requirements/defaults/root behavior

exact-case impl
    witnesses/overrides/case-only behavior
```

---

# Relationship to Data Objects

Both data objects and enum payloads may be product-shaped, but they represent different semantic categories.

```text
data
    one nominal transparent product

enum
    one nominal closed sum of variants
```

A `data` declaration introduces one nominal product type directly.

An enum declaration introduces a nominal sum whose variants each have distinct semantic identity and may contain product payloads.

Shared product representation does not make a data object a one-case enum or make an enum payload an independent data object.

---

# Relationship to Classes

An enum is not an open class hierarchy.

Variants are not subclasses that can be extended by later declarations.

The variant set is closed by the enum declaration.

Class inheritance and enum case refinement are separate mechanisms.

An implementation may reuse object dispatch machinery for enum behavior without turning variants into subclasses or the enum into an ordinary class hierarchy.

---

# Relationship to Traits

Enums and exact cases may participate in trait conformance where permitted by the trait specification.

Trait conformance remains separate from the enum's own closed-case behavioral contract.

A root enum requirement does not create trait conformance evidence.

A trait requirement does not automatically become a root enum requirement merely because the conforming target happens to be an enum.

The same concrete callable may potentially participate in both systems where independently valid, but the requirement identities and witness relationships remain separate.

---

# Relationship to Implementation Declarations

The implementation-declaration specification governs common mechanics such as:

- implementation syntax;
- target resolution;
- ownership authorization;
- generic parameters owned by the impl;
- conditional applicability;
- member selectors;
- member conflicts;
- visibility;
- callable provenance.

This enum specification adds the enum-specific interpretation of two targets:

```phalcom
impl Expression {
  ...
}
```

and:

```phalcom
impl Expression::Literal(_) {
  ...
}
```

The first may establish enum-root behavior, including requirements and defaults.

The second contributes exact-case behavior.

The distinction between root requirement, root default, exact-case witness, override, and case-only member belongs to enum semantics.

---

# Reserved and Separately Specified Questions

The following matters are intentionally not settled by this document unless another Phalcom specification already defines them.

## Derived enum behavior

Automatic derivation or synthesis of:

```text
==
hash
toString
ordering
serialization
copy/update helpers
```

is specified separately.

The representation-independence requirements of this document still apply to any such behavior.

## Stable ABI and discriminants

No stable numeric discriminant or binary layout is implied by an ordinary enum declaration.

Any FFI or stable-representation feature requires a separate explicit language contract.

## Trait conformance syntax details

Traits, witness selection, coherence, associated types, and conditional conformance are governed by the trait specification.

## Constrained inherent impl overlap

General rules for overlap or specialization among constrained inherent `impl` declarations belong to the implementation-declaration specification.

This enum specification assumes only that the applicable implementation set has been determined according to those rules.

## Pattern surface syntax

This document specifies variant-pattern semantics but does not independently redesign the language's pattern syntax.

## Class-side/metatype behavior

Any future class-side trait or metatype feature is outside the enum semantic model described here.

---

# Governing Guarantees

Every conforming implementation of Phalcom must preserve the following guarantees for enums.

> **Nominal closed sum**
>
> An enum declaration introduces one nominal sum with a closed, declaration-owned set of variants.

> **Variants-only declaration**
>
> The enum body defines cases and their payloads; behavioral members live in implementation declarations.

> **Stable variant identity**
>
> Each variant has semantic identity independent of payload shape, runtime tag, or physical layout.

> **Exact-case typing**
>
> A proven variant refines the receiver to exact-case semantics, including payload, case-specific behavior, and applicable type refinements.

> **Closed root contracts**
>
> A root member without a body is a requirement over the enum's applicable exact cases; a root member with a body supplies a root default.

> **Witness completeness**
>
> Every applicable exact case must satisfy every applicable root requirement unless a valid root default provides that behavior.

> **Specialization-aware witnessing**
>
> Requirement compatibility is checked after exact-case specialization, variant-local generic substitution, case type refinement, and GADT result specialization.

> **Explicit root behavior**
>
> Behavior belongs to the enum root only when explicitly declared there. Repetition across cases cannot implicitly promote a member to the root.

> **Case-only behavior**
>
> An exact-case member is available only when the receiver is sufficiently refined to that case or another rule proves its applicability.

> **Contract/trait separation**
>
> Closed enum requirements are not traits and do not create reusable conformance relationships.

> **Constructor/reference separation**
>
> `Option::Some(...)` constructs a variant; `&Option::Some(...)` denotes an associated callable reference; an exact-case `impl` target denotes neither invocation nor callable reference.

> **Behavior/representation separation**
>
> `impl` may contribute enum behavior but cannot add variants or modify payload representation semantics.

> **Representation freedom**
>
> Runtime tags, payload layouts, specialized encodings, boxing, flattening, and scalar replacement are implementation choices so long as semantic enum behavior is preserved.

> **Truthful reification**
>
> Exact applied enum type and semantic variant identity survive runtime observation boundaries where those facts are semantically established.

> **Reflection fidelity**
>
> Reflection must preserve distinctions among enum declaration, variant, exact case, root requirement/default, exact-case callable, witness relation, and implementation provenance.

> **Exhaustiveness/behavior separation**
>
> Pattern exhaustiveness and behavioral requirement completeness are distinct static judgments even though both reason over the same closed variant set.

The governing rule is:

> **An enum is defined by its nominal root type, its closed set of semantic variants, the payload state of each exact case, and its explicitly declared closed-case behavior contracts. Physical representation and implementation strategy may vary freely, but they may never change those semantics.**
