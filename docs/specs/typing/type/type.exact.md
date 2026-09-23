# Phalcom Type Algebra Registry — `type.exact`

**Status:** Normative foundational specification  
**Registry family:** `type.exact`  
**Identifier range used here:** `T0350–T0363`

## Purpose

This file defines exact-value types: singleton proper types containing one specific value.

Conceptually, `exact(v)` is the type whose only inhabitant is `v`. Surface syntax may later allow certain values to stand directly in type position; this specification deliberately separates that syntactic question from the semantic exact-type model.

Exact types provide the basis for literal types, finite closed domains, Boolean decomposition, type-level value arguments, and precise pattern-space reasoning.

## Domain and notation

- `exact(v)` denotes the exact singleton type for value `v`.
- `base(v)` denotes a proper type that `v` ordinarily inhabits.
- `inhabitants(A)` denotes semantic cardinality where meaningful.
- `v : A` means `v` inhabits `A`.
- `staticEquivalent(a, b)` is compile-time/static value identity from `type.static-value`.
- `valueEqual(a, b)` may describe ordinary value equality but does not define exact-type identity.

## Laws

```phalcom
@id(T0350)
@name(type.exact.admissible)
ExactTypeRequiresStaticValue<v> {
    wellFormed(exact(v)) iff staticValue(v)
}

@id(T0351)
@name(type.exact.kind)
ExactTypeKind<v> {
    if staticValue(v) {
        kind(exact(v)) == Type
    }
}

@id(T0352)
@name(type.exact.value.inhabits)
ExactValueInhabitsExactType<v> {
    v : exact(v)
}

@id(T0353)
@name(type.exact.singleton)
ExactTypeHasOneInhabitant<v> {
    inhabitants(exact(v)) == 1
}

@id(T0354)
@name(type.exact.only-inhabitant)
ExactTypeHasOnlyExactValue<v, x> {
    if x : exact(v) {
        staticEquivalent(x, v)
    }
}

@id(T0355)
@name(type.exact.base.subtype)
ExactTypeSubtypesBase<v, A> {
    if v : A {
        exact(v) <: A
    }
}

@id(T0356)
@name(type.exact.static-equivalence.identity)
EquivalentStaticValuesHaveIdenticalExactTypes<a, b> {
    if staticEquivalent(a, b) {
        exact(a) == exact(b)
    }
}

@id(T0357)
@name(type.exact.identity.implies-static-equivalence)
IdenticalExactTypesHaveEquivalentStaticValues<a, b> {
    if exact(a) == exact(b) {
        staticEquivalent(a, b)
    }
}

@id(T0358)
@name(type.exact.injective)
ExactTypeInjective<a, b> {
    exact(a) == exact(b) iff staticEquivalent(a, b)
}

@id(T0359)
@name(type.exact.distinct.disjoint)
DistinctExactTypesAreDisjoint<a, b> {
    if not staticEquivalent(a, b) {
        disjoint(exact(a), exact(b))
    }
}

@id(T0360)
@name(type.exact.static-distinct.not-identical)
DistinctStaticValuesHaveDistinctExactTypes<a, b> {
    if not staticEquivalent(a, b) {
        exact(a) != exact(b)
    }
}

@id(T0361)
@name(type.exact.never.nonempty)
ExactTypeIsNotNever<v> {
    exact(v) != Never
}

@id(T0362)
@name(type.exact.any.bounded)
ExactTypeIsBounded<v> {
    Never <: exact(v)
    exact(v) <: Any
}

@id(T0363)
@name(type.exact.unit.identity)
UnitExactIdentity {
    Unit == exact(())
}
```

## T0350 — `ExactTypeRequiresStaticValue`

`exact(v)` is well formed exactly for values admitted to the static-value universe. Arbitrary runtime objects do not automatically induce exact types.

**Kind:** Well-formedness law  
**Depends on:** `type.static-value`

## T0351 — `ExactTypeKind`

Every exact-value type is a proper type.

**Kind:** Semantic law  
**Depends on:** T0050

## T0352 — `ExactValueInhabitsExactType`

The defining value inhabits its exact type.

**Kind:** Semantic law  
**Depends on:** T0351

## T0353 — `ExactTypeHasOneInhabitant`

Every exact type has cardinality one.

**Kind:** Semantic law  
**Depends on:** T0352

## T0354 — `ExactTypeHasOnlyExactValue`

If some value inhabits `exact(v)`, that value is statically equivalent to `v`.

T0352 and T0354 characterize exact types extensionally.

**Kind:** Semantic law  
**Depends on:** T0352, T0353

## T0355 — `ExactTypeSubtypesBase`

If `v` inhabits a proper type `A`, then the exact type of `v` is a subtype of `A`.

Examples include integer literal exact types being subtypes of `Int`, exact Boolean cases being subtypes of `Bool`, and `exact(())` being a subtype of `Unit`.

**Kind:** Semantic law  
**Depends on:** T0100–T0103, T0352

## T0356 — `EquivalentStaticValuesHaveIdenticalExactTypes`

Statically equivalent values generate the same exact semantic type. Runtime-overloadable equality is not the authority for exact-type identity.

**Kind:** Semantic law  
**Depends on:** `type.static-value`

## T0357 — `IdenticalExactTypesHaveEquivalentStaticValues`

If two exact types are semantically identical, their defining static values are statically equivalent.

**Kind:** Semantic law  
**Depends on:** T0353, T0354

## T0358 — `ExactTypeInjective`

`exact` is injective with respect to static-value equivalence.

**Kind:** Derived semantic law  
**Depends on:** T0356, T0357

## T0359 — `DistinctExactTypesAreDisjoint`

Statically distinct exact values produce singleton types with no common inhabitant.

`disjoint(A, B)` is a semantic relation here; the later intersection family may establish its equivalence with `A & B == Never`.

**Kind:** Semantic law  
**Depends on:** T0353, T0354, T0358

## T0360 — `DistinctStaticValuesHaveDistinctExactTypes`

Statically distinct values cannot generate the same exact type.

**Kind:** Derived semantic law  
**Depends on:** T0358

## T0361 — `ExactTypeIsNotNever`

Every exact type has an inhabitant and therefore cannot be `Never`.

**Kind:** Derived semantic law  
**Depends on:** T0152–T0154, T0352

## T0362 — `ExactTypeIsBounded`

Every exact type lies between bottom and top in the ordinary proper-type subtype order.

**Kind:** Derived semantic law  
**Depends on:** T0151, T0201

## T0363 — `UnitExactIdentity`

Because `()` is a static value and `Unit` has exactly that one inhabitant, `Unit` and `exact(())` are the same semantic singleton type.

**Kind:** Cross-family derived law  
**Depends on:** `type.unit` and the preceding exact-type laws

## Required implementation invariants

Exact-value typing must not depend on whether the value arose from:

- a literal,
- constant folding,
- type-level computation,
- a closed `type data` constructor,
- reflection,
- or canonicalization.

If the static value is equivalent, the exact-type identity laws require the exact type to be the same.

## Explicitly deferred laws

- literal surface syntax in type position
- exact integer/string/symbol-specific rules
- intersection of distinct exact types reducing to `Never`
- union of exact cases reconstructing a finite parent type
- exact-type reflection and serialization
