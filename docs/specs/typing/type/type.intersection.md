# Phalcom Type Algebra Registry — `type.intersection`

**Status:** Normative specification  
**Registry family:** `type.intersection`  
**Identifier range used here:** `T0500–T0515`

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
@id(T0500)
@name(type.intersection.kind)
IntersectionKind<A, B> {
    kind(A & B) == Type
}

@id(T0501)
@name(type.intersection.commutative)
IntersectionCommutative<A, B> {
    A & B == B & A
}

@id(T0502)
@name(type.intersection.associative)
IntersectionAssociative<A, B, C> {
    (A & B) & C == A & (B & C)
}

@id(T0503)
@name(type.intersection.idempotent)
IntersectionIdempotent<A> {
    A & A == A
}

@id(T0504)
@name(type.intersection.any.identity)
IntersectionAnyIdentity<A> {
    A & Any == A
}

@id(T0505)
@name(type.intersection.never.absorbing)
IntersectionNeverAbsorbing<A> {
    A & Never == Never
}

@id(T0506)
@name(type.intersection.subtype.left)
IntersectionSubtypeLeft<A, B> {
    A & B <: A
}

@id(T0507)
@name(type.intersection.subtype.right)
IntersectionSubtypeRight<A, B> {
    A & B <: B
}

@id(T0508)
@name(type.intersection.greatest-lower-bound)
IntersectionGreatestLowerBound<A, B, L> {
    if L <: A and L <: B {
        L <: A & B
    }
}

@id(T0509)
@name(type.intersection.subtype-absorption)
IntersectionSubtypeAbsorption<A, B> {
    if A <: B {
        A & B == A
    }
}

@id(T0510)
@name(type.intersection.identity-substitution)
IntersectionIdentitySubstitution<A, B, C> {
    if A == B {
        A & C == B & C
    }
}

@id(T0511)
@name(type.intersection.monotone.left)
IntersectionLeftMonotone<A, B, C> {
    if A <: B {
        A & C <: B & C
    }
}

@id(T0512)
@name(type.intersection.monotone.right)
IntersectionRightMonotone<A, B, C> {
    if A <: B {
        C & A <: C & B
    }
}

@id(T0513)
@name(type.intersection.monotone)
IntersectionMonotone<A, B, C, D> {
    if A <: B and C <: D {
        A & C <: B & D
    }
}

@id(T0514)
@name(type.intersection.membership)
IntersectionMembership<v, A, B> {
    v : A & B iff v : A and v : B
}

@id(T0515)
@name(type.intersection.disjoint-never)
DisjointIntersectionIsNever<A, B> {
    disjoint(A, B) iff A & B == Never
}
```

## Law notes

### T0500 — `IntersectionKind`

The intersection of two proper types is itself a proper type.

**Kind:** Semantic law  
**Depends on:** T0060

### T0501 — `IntersectionCommutative`

Intersection is insensitive to operand order.

**Kind:** Semantic algebra law  
**Depends on:** T0001–T0003

### T0502 — `IntersectionAssociative`

Grouping does not affect semantic intersection identity.

**Kind:** Semantic algebra law  
**Depends on:** T0501

### T0503 — `IntersectionIdempotent`

Intersecting a type with itself yields the same type.

**Kind:** Semantic algebra law  
**Depends on:** T0102–T0104

### T0504 — `IntersectionAnyIdentity`

`Any` is the identity element of intersection.

**Kind:** Semantic algebra law  
**Depends on:** T0201–T0203

### T0505 — `IntersectionNeverAbsorbing`

`Never` is the absorbing element of intersection.

**Kind:** Semantic algebra law  
**Depends on:** T0151–T0156

### T0506 / T0507 — Result subtyping

The intersection is a subtype of each operand.

**Kind:** Semantic order laws  
**Depends on:** T0100–T0103

### T0508 — `IntersectionGreatestLowerBound`

`A & B` is the greatest lower bound of `A` and `B`.

Together with T0506 and T0507, this is the defining order-theoretic characterization of intersection.

**Kind:** Semantic order law  
**Depends on:** T0506, T0507, T0101

### T0509 — `IntersectionSubtypeAbsorption`

If `A <: B`, the intersection collapses to the narrower type `A`.

**Kind:** Derived semantic law  
**Depends on:** T0506–T0508, T0103

### T0510 — `IntersectionIdentitySubstitution`

Replacing an operand with a semantically identical type preserves the intersection.

**Kind:** Derived coherence law  
**Depends on:** T0001–T0003, T0501–T0503

### T0511–T0513 — Monotonicity

Intersection is monotone in each operand and jointly monotone.

**Kind:** Derived order laws  
**Depends on:** T0506–T0508, T0101

### T0514 — `IntersectionMembership`

A value inhabits `A & B` exactly when it inhabits both operands.

**Kind:** Semantic membership law  
**Depends on:** T0506–T0508

### T0515 — `DisjointIntersectionIsNever`

Two proper types are semantically disjoint exactly when their intersection is `Never`.

This connects the independently useful `disjoint` relation to ordinary type algebra.

**Kind:** Cross-family semantic law  
**Depends on:** T0152–T0156, T0358–T0360, T0505, T0514

## Required implementation invariants

Semantic intersection must remain coherent across nominal, structural, exact, generic, projected, and inferred types.

The compiler is not required to eagerly prove every empty intersection. Failure to prove `A & B == Never` is not permission to assert that the intersection is inhabited; proof-state behavior belongs to `type.solver`.

## Explicitly deferred laws

- union/intersection absorption — `type.algebra`
- distributivity — `type.algebra`
- eager simplification and canonical operand ordering — `type.normalization`
- nominal/interface composition details — nominal and structural type families
- pattern reachability using empty intersections — `type.pattern`
