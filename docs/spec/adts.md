# Phalcom Language Specification
## Algebraic Data Types (ADTs)

**Status:** Normative language specification — initial ADT entry  
**Applies to:** Phalcom v1 ADT model  
**Scope:** Source-level and static semantics  
**Non-scope:** Compiler architecture, bytecode layout, GC representation, optimization strategy

---

## 1. Purpose

Phalcom algebraic data types (ADTs) define **closed nominal sum types**.

An ADT is declared with `enum`. Its alternatives are declared explicitly with `@variant`.

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}
```

`Option<T>` is the nominal enum type. `Some` and `None` are its variants.

An enum is closed: the complete set of variants belongs to the enum declaration and cannot be extended elsewhere.

This document specifies the language meaning of ADTs. Physical representation is deliberately separate from these semantics.

---

## 2. Enum declarations

The basic form is:

```phalcom
enum Name<GenericParameters...> {
    members...
}
```

Enums may use the ordinary generic declaration machinery.

```phalcom
enum Result<T, E> {
    @variant Ok(_ value: T)
    @variant Error(_ error: E)
}
```

An enum does not declare a source-level superclass. Its relationship to its variants is intrinsic to ADT semantics.

Enum bodies may contain:

- variant declarations;
- enum-root behavior;
- signature-only enum requirements.

Variants must be introduced explicitly with `@variant`. A bare identifier in an enum body is not implicitly a variant.

---

## 3. Variants

A variant is a first-class semantic declaration belonging to exactly one enum.

Phalcom distinguishes three important variant forms.

### 3.1 Singleton variant

A bare variant has no constructor.

```phalcom
@variant None
```

It declares one canonical case value.

The associated getter:

```phalcom
Option::None
```

obtains that singleton value.

A singleton is not a zero-argument constructor.

### 3.2 Zero-argument constructor variant

Parentheses explicitly declare a constructor, even when there are no parameters.

```phalcom
@variant Empty()
```

The expression:

```phalcom
Container::Empty()
```

performs construction.

This form is semantically distinct from:

```phalcom
@variant Empty
```

An implementation must not collapse an explicit zero-argument constructor into the singleton form merely because both carry no payload.

### 3.3 Payload constructor variant

A variant may carry immutable payload data.

```phalcom
@variant Some(_ value: T)
```

Variant payload parameters reuse ordinary Phalcom parameter syntax, including positional and externally labeled parameters.

```phalcom
@variant Error(
    _ error: E,
    reason message: String
)
```

The constructor's callable shape follows its declared parameter shape.

Variant rest parameters are not part of the initial ADT model.

---

## 4. Variant families and selector identity

Variants with the same base name may coexist when their exact selector shapes differ.

```phalcom
enum Example {
    @variant None
    @variant None()
    @variant None(_ value: Int)
}
```

These are three different exact members of one associated `None` family:

```text
#None
#None()
#None(_)
```

Consequently:

```phalcom
Example::None
Example::None()
Example::None(42)
```

have different meanings:

- `Example::None` obtains the singleton;
- `Example::None()` invokes the zero-argument constructor;
- `Example::None(42)` invokes the matching payload constructor.

Getter-shaped and zero-argument callable selectors are never interchangeable.

---

## 5. Associated access to variants

Enum variants are declaration-owned associated members exposed through `::`.
Associated lookup has precedence over ordinary receiver-bound `::` behavioral family
resolution at a reserved associated base. Outside an associated base, `::` retains its
ordinary receiver-bound deferred-dispatch semantics.

If a declaration exposes an associated base (such as `Some`), no ordinary behavior declared
in that same declaration may use base `Some`, regardless of exact selector shape or dispatch side.
For example, declaring `@variant Some(_ value: T)` and `@class Some(_ left: T, _ right: T)` in
the same enum is forbidden by entire-base reservation.

```phalcom
Option::None
Option::Some(42)
```

`.` remains ordinary message sending.

There is no rule that interprets a dot send as variant lookup merely because the receiver is an enum or type.

There is also no fallback from failed `::` lookup to ordinary message dispatch or `doesNotUnderstand` once an associated base is reserved.

### 5.1 Exact getter lookup

```phalcom
Option::None
```

means exact getter-shaped associated lookup for `#None`.

It does not mean "the `None` family."

### 5.2 Direct constructor invocation

```phalcom
Option::Some(42)
```

performs associated-family selection using the incoming call shape and invokes the selected constructor.

A variant constructor is a constructor identity, not an ordinary method identity.

### 5.3 Exact constructor reference

An exact constructor member may be reified by specifying its residual selector shape.

```phalcom
const some = Option::Some::(_)
```

The resulting value is callable, but its semantic executable identity remains a variant constructor rather than a method.

### 5.4 Whole family

The complete associated family is written:

```phalcom
Option::Some::*
```

`::*` is the only whole-family form. `Option::Some` is not an abbreviation for it.

Whole-family values are first-class capability values. Their detailed structural typing is specified separately.

---

## 6. Exact cases and enum types

Each variant denotes an **exact case** of its enum.

For example:

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}
```

construction of:

```phalcom
Option::Some(42)
```

has a type more precise than merely `Option<Int>`: the checker knows that the value is specifically the `Some` case of `Option<Int>`.

Conceptually:

```text
ExactCase(Option::Some(_), Option<Int>)
```

The exact case is a subtype of the corresponding enum specialization:

```text
ExactCase(V, E) <: E
```

Therefore an exact `Some` value is valid wherever its containing `Option<Int>` is expected.

Exact-case identity is part of the semantic type system. Phalcom v1 does not require public source syntax for writing exact-case types explicitly.

The checker should preserve exact-case precision while it remains useful. It must not globally widen every exact case to its enum root immediately after construction.

---

## 7. Generics

Enums use Phalcom's ordinary generic type system.

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}
```

For:

```phalcom
const x = Option::Some(42)
```

the constructor argument may establish the specialization `Option<Int>`.

The exact result is the `Some` case within that specialization.

Generic specialization is a static property. It does not require a distinct runtime class or runtime enum definition for every concrete generic argument.

---

## 8. GADTs

A variant may explicitly specialize the result of its enclosing generic enum.

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}
```

The result annotation must be an application of the same enclosing enum.

The `Int` declaration establishes the case fact:

```text
T = Int
```

and the `Bool` declaration establishes:

```text
T = Bool
```

These equalities belong to the variant declaration. They are available while checking case-local behavior and may be introduced into the corresponding branch when the exact case is proven during elimination.

The equality proof is not part of runtime value identity and does not need to be stored on each constructed value.

A GADT case still has an exact-case type. For example:

```text
ExactCase(Expr::Int(_), Expr<Int>)
```

GADT result specialization refines the enum result; it does not create a different kind of runtime value.

---

## 9. Enum-root behavior

An enum may define ordinary shared instance behavior.

```phalcom
enum Shape {
    describe -> String {
        "shape"
    }

    @variant Circle(_ radius: Float)
}
```

Bodyful enum-root behavior is inherited by every exact case unless that case provides a compatible override.

This is ordinary `.` behavior of enum values. It is distinct from the enum's `::` associated variant surface.

---

## 10. Closed-enum requirements

A signature-only behavior declaration at the enum root is a requirement that every concrete variant must satisfy.

```phalcom
enum Shape {
    area -> Float

    @variant Circle(_ radius: Float) {
        area -> Float {
            ...
        }
    }

    @variant Rectangle(_ width: Float, _ height: Float) {
        area -> Float {
            ...
        }
    }
}
```

Here:

```phalcom
area -> Float
```

is not an abstract method intended for unknown future subclasses.

`Shape` is closed. The declaration is therefore a **closed-enum requirement** over the finite set of `Shape` variants.

Every variant must satisfy the requirement, including:

- singleton variants;
- zero-argument constructor variants;
- variants whose construction is not publicly accessible.

A case may also add behavior that is not declared on the enum root.

Case-local behavior is instance-side behavior. Case-local static/class-side declarations are not part of the initial ADT model.

---

## 11. Payload semantics

Constructor payload parameters define case data.

```phalcom
enum Result<T, E> {
    @variant Ok(_ value: T)
    @variant Error(_ error: E)
}
```

Payload data is immutable case state.

Within case-local behavior, payload fields are receiver-owned data of that exact case.

The semantic model distinguishes:

- variant identity;
- constructor parameter identity;
- payload field identity.

They must not be collapsed into one ordinary method or field declaration merely because an implementation happens to store them similarly.

Public source syntax for arbitrary exact-case payload projection is specified separately. This document does not invent such syntax.

---

## 12. Variant visibility

Variant visibility has independent semantic dimensions:

```text
name / match visibility
construction visibility
payload visibility
```

These are not one visibility bit.

The initial `@private` variant rule restricts construction while preserving the case as part of the enum.

```phalcom
@private
@variant Hidden(_ value: Int)
```

means, by default:

```text
name / match    public
construction    private
payload         public
```

Therefore a private constructor remains:

- a real case of the enum;
- visible to closed-enum completeness rules;
- matchable where its name is visible;
- subject to enum-root requirements.

Privacy of construction must never make a variant disappear from exhaustiveness analysis.

Additional payload-visibility syntax is not defined here.

---

## 13. Elimination and matching

ADTs are intended to be eliminated by pattern matching.

Matching an enum is not method dispatch. It is elimination of a closed sum using variant identity.

The matcher must therefore reason about exact variants, not invoke variant methods to determine which case a value represents.

For GADTs, successful identification of an exact variant may introduce the equality evidence declared by that variant into the corresponding branch.

The concrete `match` grammar, pattern forms, exhaustiveness algorithm, redundancy checks, and branch-refinement rules are specified in the dedicated matching specification.

---

## 14. Equality, identity, and construction

A singleton declaration:

```phalcom
@variant None
```

denotes one canonical singleton case value.

An explicit constructor declaration:

```phalcom
@variant None()
```

denotes a construction operation.

These meanings remain distinct even if an implementation can represent both very compactly.

Compiler optimizations may remove allocations, scalar-replace values, or use immediate representations only when observable language behavior is preserved. Optimization cannot change a constructor into a singleton or merge semantically distinct variant identities.

---

## 15. Representation is not semantics

Phalcom does not define ADTs as inherently boxed objects.

The language semantics require:

- enum identity;
- exact variant identity;
- payload values;
- ordinary behavior;
- associated constructor/singleton behavior;
- correct elimination and type refinement.

They do not require a particular physical layout.

An implementation may use, where valid:

- immediate tags;
- tagged unions;
- heap case objects;
- stack allocation;
- scalar replacement;
- niche encoding;
- specialized native layouts.

Crossing an `Object` or `Dynamic` erasure boundary may require materializing a more general runtime representation. Such materialization is a representation cost of the boundary; it does not imply that every ADT value is inherently boxed.

Static semantic identity, runtime metadata identity, physical representation, and allocation strategy are separate concerns.

---

## 16. Closedness and extension

An enum's variants are fixed by its declaration.

Code outside the declaration cannot add another variant to the enum.

This closed-world property enables:

- exhaustive matching;
- precise exact-case unions;
- enum-wide requirements;
- GADT refinement over a known case set.

Ordinary behavior inheritance of enum values does not make the set of variants open.

---

## 17. Required semantic distinctions

A conforming Phalcom implementation must preserve all of the following distinctions:

```text
enum type
!= exact case type

variant identity
!= variant constructor identity

singleton variant
!= zero-argument constructor variant

constructor
!= ordinary method

associated lookup (`::`)
!= message send (`.`)

whole associated family (`::*`)
!= exact getter lookup

GADT equality evidence
!= runtime case identity

language semantics
!= physical representation
```

Implementations may optimize representation aggressively, but they may not erase these distinctions where doing so changes program meaning, typing, reflection, source tooling, visibility, or elimination.

---

## 18. Deferred to dedicated specifications

This introductory ADT specification deliberately leaves the following to focused language specifications:

- the full `match` grammar and pattern language;
- exhaustiveness and redundancy diagnostics;
- branch-local refinement and GADT proof rules;
- the full structural type of first-class associated families;
- exact-case source type syntax;
- additional payload-access visibility syntax;
- reflection APIs;
- serialization/ABI guarantees;
- implementation-specific runtime layouts.

Those documents may refine the corresponding areas, but must preserve the semantic invariants stated here.

---

## 19. Minimal conformance examples

### Generic ADT

```phalcom
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}

const a = Option::Some(42)
const b = Option::None
```

`a` is known to be the exact `Some` case of `Option<Int>`.  
`b` is the canonical `None` singleton in the relevant `Option` specialization.

### Distinct getter and constructor selectors

```phalcom
enum Example {
    @variant State
    @variant State()
    @variant State(_ value: Int)
}

const singleton = Example::State
const empty = Example::State()
const payload = Example::State(1)
```

The three declarations and the three expressions are semantically distinct.

### Shared behavior and closed requirement

```phalcom
enum Shape {
    area -> Float

    describe -> String {
        "shape"
    }

    @variant Circle(_ radius: Float) {
        area -> Float {
            3.14159 * radius * radius
        }
    }

    @variant Rectangle(_ width: Float, _ height: Float) {
        area -> Float {
            width * height
        }
    }
}
```

Every case receives `describe` by default. Every case must satisfy `area -> Float`.

### GADT

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}
```

The result annotation records exact case-specific equalities that the checker can use when the case is proven.

---

## 20. Normative summary

Phalcom ADTs are closed nominal sums with first-class exact variant identity.

Variants are explicit `@variant` declarations and may be canonical singletons or constructor-shaped cases. Parentheses are semantically meaningful: a bare singleton and a zero-argument constructor are different declarations and different operations.

Variant construction and lookup live on the associated `::` surface, which takes precedence over receiver-bound behavioral `::` resolution at reserved associated bases. Ordinary `.` sends remain behavioral dispatch. Exact cases retain more precise static types than their enum roots, generic ADTs use ordinary specialization, and GADT result annotations establish case-specific type equalities.

Enum-root bodyful behavior is shared/default instance behavior. Enum-root signature-only behavior is a closed-enum requirement that every variant must satisfy.

ADTs are not defined by boxing or any other physical representation. Runtime layout is an implementation concern so long as the observable semantics and static distinctions of this specification are preserved.

---

## 21. Native Core ADTs and Reflection (Part 06)

### 21.1 Core sum types

Core library sum types (`Option<T>`, `Result<T, E>`, `Ordering`) are canonical `@native enum` declarations:

```phalcom
@native
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}

@native
enum Result<T, E> {
    @variant Ok(_ value: T)
    @variant Error(_ error: E)
}

@native
enum Ordering {
    @variant Less
    @variant Equal
    @variant Greater
}
```

The `@native` attribute indicates implementation provenance and runtime representation strategy; it does not grant alternate enum semantics. Core ADTs publish the same canonical declaration, type, match, and reflection products as user-declared enums.

`Bool` remains a primitive finite domain and is not converted into an enum.

### 21.2 ExactCase canonicalization and reflection

Every exact variant instantiation produces a canonical `ExactCase` type in the type store. Specialized exact cases (such as `ExactCase(Some, Option<Int>)`) are canonical semantic types.

Reflection operates through dedicated semantic metaobjects (`EnumReflection`, `VariantReflection`, `VariantFamilyReflection`, `VariantFieldReflection`, `ExactCaseTypeReflection`). Associated lookup (`::`) remains purely for value construction and member denotation, not reflection.

`.class` on an ADT instance returns its runtime case behavior class, which is reflectable but distinct from the static `ExactCase` type and `VariantId` declaration identity.
