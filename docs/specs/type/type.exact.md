# Phalcom Type Algebra Registry — `type.exact`

**Status:** Normative foundational specification  
**Registry family:** `type.exact`  
**Identifier range used here:** `T0060–T0071`

## Purpose

This file defines exact-value types: singleton proper types containing one specific value.

Conceptually, `exact(v)` is the type whose only inhabitant is `v`. Surface syntax may later allow certain values to stand directly in type position; this specification deliberately separates that syntactic question from the semantic exact-type model.

Exact types provide the basis for literal types, finite closed domains, Boolean decomposition, type-level value arguments, and precise pattern-space reasoning.

## Domain and notation

- `exact(v)` denotes the exact singleton type for value `v`.
- `base(v)` denotes a proper type that `v` ordinarily inhabits.
- `inhabitants(A)` denotes semantic cardinality where meaningful.
- `v : A` means `v` inhabits `A`.
- `a == b` between values means ordinary semantic value equality.
- Type identity also uses `==`; context distinguishes the two.

## Laws

```phalcom
@id(T0060)
@name(type.exact.kind)
ExactTypeKind<v> {
    kind(exact(v)) == Type
}

@id(T0061)
@name(type.exact.value.inhabits)
ExactValueInhabitsExactType<v> {
    v : exact(v)
}

@id(T0062)
@name(type.exact.singleton)
ExactTypeHasOneInhabitant<v> {
    inhabitants(exact(v)) == 1
}

@id(T0063)
@name(type.exact.only-inhabitant)
ExactTypeHasOnlyExactValue<v, x> {
    if x : exact(v) {
        x == v
    }
}

@id(T0064)
@name(type.exact.base.subtype)
ExactTypeSubtypesBase<v, A> {
    if v : A {
        exact(v) <: A
    }
}

@id(T0065)
@name(type.exact.equal-values.identity)
EqualValuesHaveIdenticalExactTypes<a, b> {
    if a == b {
        exact(a) == exact(b)
    }
}

@id(T0066)
@name(type.exact.identity.implies-value-equality)
IdenticalExactTypesHaveEqualValues<a, b> {
    if exact(a) == exact(b) {
        a == b
    }
}

@id(T0067)
@name(type.exact.injective)
ExactTypeInjective<a, b> {
    exact(a) == exact(b) iff a == b
}

@id(T0068)
@name(type.exact.distinct.disjoint)
DistinctExactTypesAreDisjoint<a, b> {
    if a != b {
        disjoint(exact(a), exact(b))
    }
}

@id(T0069)
@name(type.exact.distinct.not-identical)
DistinctValuesHaveDistinctExactTypes<a, b> {
    if a != b {
        exact(a) != exact(b)
    }
}

@id(T0070)
@name(type.exact.never.nonempty)
ExactTypeIsNotNever<v> {
    exact(v) != Never
}

@id(T0071)
@name(type.exact.any.bounded)
ExactTypeIsBounded<v> {
    Never <: exact(v)
    exact(v) <: Any
}
```

## T0060 — `ExactTypeKind`

Every exact-value type is a proper type.

**Kind:** Semantic law  
**Depends on:** T0010

## T0061 — `ExactValueInhabitsExactType`

The defining value inhabits its exact type.

**Kind:** Semantic law  
**Depends on:** T0060

## T0062 — `ExactTypeHasOneInhabitant`

Every exact type has cardinality one.

**Kind:** Semantic law  
**Depends on:** T0061

## T0063 — `ExactTypeHasOnlyExactValue`

If some value inhabits `exact(v)`, that value is semantically equal to `v`.

T0061 and T0063 characterize exact types extensionally.

**Kind:** Semantic law  
**Depends on:** T0061, T0062

## T0064 — `ExactTypeSubtypesBase`

If `v` inhabits a proper type `A`, then the exact type of `v` is a subtype of `A`.

Examples include integer literal exact types being subtypes of `Int`, exact Boolean cases being subtypes of `Bool`, and `exact(())` being a subtype of `Unit`.

**Kind:** Semantic law  
**Depends on:** T0020–T0023, T0061

## T0065 — `EqualValuesHaveIdenticalExactTypes`

Semantically equal values generate the same exact semantic type.

This prevents exact-type identity from depending on source spelling, object reconstruction, or representation details when value equality says the values are the same.

**Kind:** Semantic law  
**Depends on:** T0001–T0003

## T0066 — `IdenticalExactTypesHaveEqualValues`

If two exact types are semantically identical, their defining values are semantically equal.

**Kind:** Semantic law  
**Depends on:** T0062, T0063

## T0067 — `ExactTypeInjective`

`exact` is injective with respect to semantic value equality.

**Kind:** Derived semantic law  
**Depends on:** T0065, T0066

## T0068 — `DistinctExactTypesAreDisjoint`

Distinct exact values produce singleton types with no common inhabitant.

`disjoint(A, B)` is a semantic relation here; the later intersection family may establish its equivalence with `A & B == Never`.

**Kind:** Semantic law  
**Depends on:** T0062, T0063, T0067

## T0069 — `DistinctValuesHaveDistinctExactTypes`

Distinct values cannot generate the same exact type.

**Kind:** Derived semantic law  
**Depends on:** T0067

## T0070 — `ExactTypeIsNotNever`

Every exact type has an inhabitant and therefore cannot be `Never`.

**Kind:** Derived semantic law  
**Depends on:** T0032–T0034, T0061

## T0071 — `ExactTypeIsBounded`

Every exact type lies between bottom and top in the ordinary proper-type subtype order.

**Kind:** Derived semantic law  
**Depends on:** T0031, T0041

## Required implementation invariants

Exact-value typing must not depend on whether the value arose from:

- a literal,
- constant folding,
- type-level computation,
- a closed `type data` constructor,
- reflection,
- or canonicalization.

If the semantic value is the same, T0065–T0067 require the exact type to be the same.

## Explicitly deferred laws

- which runtime values are admissible as exact type arguments
- literal surface syntax in type position
- exact integer/string/symbol-specific rules
- intersection of distinct exact types reducing to `Never`
- union of exact cases reconstructing a finite parent type
- exact-type reflection and serialization
