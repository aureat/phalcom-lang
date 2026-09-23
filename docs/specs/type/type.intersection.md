# Phalcom Type Algebra Registry — `type.intersection`

**Status:** Normative specification  
**Registry family:** `type.intersection`  
**Identifier range used here:** `T0120–T0135`

## Purpose

This file defines the semantic laws of the type-intersection operator `&`.

For proper types `A` and `B`, `A & B` denotes their greatest lower bound in the Phalcom subtype order: every inhabitant of the intersection inhabits both operands, and every proper type that is a subtype of both operands is also a subtype of the intersection.

These laws define semantic intersection. They do not require eager normalization, distributive expansion, or a particular internal representation.

## Domain and notation

Unless stated otherwise:

- `A`, `B`, `C`, and `L` denote proper types.
- `==` means semantic type identity.
- `<:` means semantic subtype.
- `&` means type intersection.
- `v : A` means value `v` inhabits `A`.
- `disjoint(A, B)` means the two types have no common inhabitant.

## Laws

```phalcom
@id(T0120)
@name(type.intersection.kind)
IntersectionKind<A, B> {
    kind(A & B) == Type
}

@id(T0121)
@name(type.intersection.commutative)
IntersectionCommutative<A, B> {
    A & B == B & A
}

@id(T0122)
@name(type.intersection.associative)
IntersectionAssociative<A, B, C> {
    (A & B) & C == A & (B & C)
}

@id(T0123)
@name(type.intersection.idempotent)
IntersectionIdempotent<A> {
    A & A == A
}

@id(T0124)
@name(type.intersection.any.identity)
IntersectionAnyIdentity<A> {
    A & Any == A
}

@id(T0125)
@name(type.intersection.never.absorbing)
IntersectionNeverAbsorbing<A> {
    A & Never == Never
}

@id(T0126)
@name(type.intersection.subtype.left)
IntersectionSubtypeLeft<A, B> {
    A & B <: A
}

@id(T0127)
@name(type.intersection.subtype.right)
IntersectionSubtypeRight<A, B> {
    A & B <: B
}

@id(T0128)
@name(type.intersection.greatest-lower-bound)
IntersectionGreatestLowerBound<A, B, L> {
    if L <: A and L <: B {
        L <: A & B
    }
}

@id(T0129)
@name(type.intersection.subtype-absorption)
IntersectionSubtypeAbsorption<A, B> {
    if A <: B {
        A & B == A
    }
}

@id(T0130)
@name(type.intersection.identity-substitution)
IntersectionIdentitySubstitution<A, B, C> {
    if A == B {
        A & C == B & C
    }
}

@id(T0131)
@name(type.intersection.monotone.left)
IntersectionLeftMonotone<A, B, C> {
    if A <: B {
        A & C <: B & C
    }
}

@id(T0132)
@name(type.intersection.monotone.right)
IntersectionRightMonotone<A, B, C> {
    if A <: B {
        C & A <: C & B
    }
}

@id(T0133)
@name(type.intersection.monotone)
IntersectionMonotone<A, B, C, D> {
    if A <: B and C <: D {
        A & C <: B & D
    }
}

@id(T0134)
@name(type.intersection.membership)
IntersectionMembership<v, A, B> {
    v : A & B iff v : A and v : B
}

@id(T0135)
@name(type.intersection.disjoint-never)
DisjointIntersectionIsNever<A, B> {
    disjoint(A, B) iff A & B == Never
}
```

## Law notes

### T0120 — `IntersectionKind`

The intersection of two proper types is itself a proper type.

**Kind:** Semantic law  
**Depends on:** T0018

### T0121 — `IntersectionCommutative`

Intersection is insensitive to operand order.

**Kind:** Semantic algebra law  
**Depends on:** T0001–T0003

### T0122 — `IntersectionAssociative`

Grouping does not affect semantic intersection identity.

**Kind:** Semantic algebra law  
**Depends on:** T0121

### T0123 — `IntersectionIdempotent`

Intersecting a type with itself yields the same type.

**Kind:** Semantic algebra law  
**Depends on:** T0022–T0024

### T0124 — `IntersectionAnyIdentity`

`Any` is the identity element of intersection.

**Kind:** Semantic algebra law  
**Depends on:** T0041–T0043

### T0125 — `IntersectionNeverAbsorbing`

`Never` is the absorbing element of intersection.

**Kind:** Semantic algebra law  
**Depends on:** T0031–T0036

### T0126 / T0127 — Result subtyping

The intersection is a subtype of each operand.

**Kind:** Semantic order laws  
**Depends on:** T0020–T0023

### T0128 — `IntersectionGreatestLowerBound`

`A & B` is the greatest lower bound of `A` and `B`.

Together with T0126 and T0127, this is the defining order-theoretic characterization of intersection.

**Kind:** Semantic order law  
**Depends on:** T0126, T0127, T0021

### T0129 — `IntersectionSubtypeAbsorption`

If `A <: B`, the intersection collapses to the narrower type `A`.

**Kind:** Derived semantic law  
**Depends on:** T0126–T0128, T0023

### T0130 — `IntersectionIdentitySubstitution`

Replacing an operand with a semantically identical type preserves the intersection.

**Kind:** Derived coherence law  
**Depends on:** T0001–T0003, T0121–T0123

### T0131–T0133 — Monotonicity

Intersection is monotone in each operand and jointly monotone.

**Kind:** Derived order laws  
**Depends on:** T0126–T0128, T0021

### T0134 — `IntersectionMembership`

A value inhabits `A & B` exactly when it inhabits both operands.

**Kind:** Semantic membership law  
**Depends on:** T0126–T0128

### T0135 — `DisjointIntersectionIsNever`

Two proper types are semantically disjoint exactly when their intersection is `Never`.

This connects the independently useful `disjoint` relation to ordinary type algebra.

**Kind:** Cross-family semantic law  
**Depends on:** T0032–T0036, T0067–T0069, T0125, T0134

## Required implementation invariants

Semantic intersection must remain coherent across nominal, structural, exact, generic, projected, and inferred types.

The compiler is not required to eagerly prove every empty intersection. Failure to prove `A & B == Never` is not permission to assert that the intersection is inhabited; proof-state behavior belongs to `type.solver`.

## Explicitly deferred laws

- union/intersection absorption — `type.algebra`
- distributivity — `type.algebra`
- eager simplification and canonical operand ordering — `type.normalization`
- nominal/interface composition details — nominal and structural type families
- pattern reachability using empty intersections — `type.pattern`
