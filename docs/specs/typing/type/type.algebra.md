# Phalcom Type Algebra Registry — `type.algebra`

**Status:** Normative specification  
**Registry family:** `type.algebra`  
**Identifier range used here:** `T0550–T0565`

## Purpose

This file defines laws that relate multiple foundational type operators or characterize the global algebra induced by semantic identity, subtyping, union, intersection, `Never`, and `Any`.

`type.union` and `type.intersection` define the two operators independently as least upper bound and greatest lower bound. This file states the resulting lattice laws and cross-operator consequences.

The distinction between **semantic law** and **normalization strategy** is mandatory. A law may be semantically valid without requiring the compiler to eagerly rewrite one side into the other.

## Domain and notation

Unless stated otherwise:

- `A`, `B`, and `C` denote proper types.
- `|` is semantic union.
- `&` is semantic intersection.
- `==` is semantic type identity.
- `<:` is semantic subtype.

## Laws

```phalcom
@id(T0550)
@name(type.algebra.lattice.bounded)
TypeAlgebraBounded<A> {
    Never <: A
    A <: Any
}

@id(T0551)
@name(type.algebra.absorption.union)
UnionAbsorption<A, B> {
    A | (A & B) == A
}

@id(T0552)
@name(type.algebra.absorption.intersection)
IntersectionAbsorption<A, B> {
    A & (A | B) == A
}

@id(T0553)
@name(type.algebra.order.union-characterization)
SubtypeByUnion<A, B> {
    A <: B iff A | B == B
}

@id(T0554)
@name(type.algebra.order.intersection-characterization)
SubtypeByIntersection<A, B> {
    A <: B iff A & B == A
}

@id(T0555)
@name(type.algebra.order.dual-characterization)
SubtypeAlgebraCharacterization<A, B> {
    A <: B iff A | B == B and A & B == A
}

@id(T0556)
@name(type.algebra.identity.join-meet)
IdentityByJoinAndMeet<A, B> {
    A == B iff A | B == A and A & B == A
}

@id(T0557)
@name(type.algebra.distributive.intersection-over-union)
IntersectionDistributesOverUnion<A, B, C> {
    if distributiveTypeFragment(A, B, C) {
        A & (B | C) == (A & B) | (A & C)
    }
}

@id(T0558)
@name(type.algebra.distributive.union-over-intersection)
UnionDistributesOverIntersection<A, B, C> {
    if distributiveTypeFragment(A, B, C) {
        A | (B & C) == (A | B) & (A | C)
    }
}

@id(T0559)
@name(type.algebra.duality.never-any)
NeverAnyDuality {
    Never | Any == Any
    Never & Any == Never
}

@id(T0560)
@name(type.algebra.exact.distinct-intersection)
DistinctExactIntersection<a, b> {
    if not staticEquivalent(a, b) {
        exact(a) & exact(b) == Never
    }
}

@id(T0561)
@name(type.algebra.boolean.partition)
BoolPartition {
    Bool == exact(true) | exact(false)
    exact(true) & exact(false) == Never
}

@id(T0562)
@name(type.algebra.unit.singleton)
UnitSingletonAlgebra {
    Unit == exact(())
    Never <: Unit
    Unit <: Any
}

@id(T0563)
@name(type.algebra.order.identity-from-mutual)
IdentityFromAlgebraicOrder<A, B> {
    if A | B == B and A & B == A
        and B | A == A and B & A == B
    {
        A == B
    }
}

@id(T0564)
@name(type.algebra.disjoint.order)
DisjointSubtypePropagation<A, B, C> {
    if A <: B and disjoint(B, C) {
        disjoint(A, C)
    }
}

@id(T0565)
@name(type.algebra.semantic-normalization-boundary)
SemanticLawDoesNotRequireExpansion<A, B, C> {
    if distributiveTypeFragment(A, B, C) {
        A & (B | C) == (A & B) | (A & C)
        notRequiresStructuralExpansion(A & (B | C))
    }
}
```

## Law notes

### T0550 — `TypeAlgebraBounded`

The proper-type subtype order is bounded by `Never` and `Any`.

**Kind:** Derived semantic law  
**Depends on:** T0151, T0201

### T0551 — `UnionAbsorption`

Union absorbs an intersection that already includes the union operand.

**Kind:** Derived semantic algebra law  
**Depends on:** T0456–T0459, T0506–T0509

### T0552 — `IntersectionAbsorption`

Intersection absorbs a union that already includes the intersection operand.

**Kind:** Derived semantic algebra law  
**Depends on:** T0456–T0459, T0506–T0509

### T0553 — `SubtypeByUnion`

`A <: B` exactly when joining `A` with `B` yields `B`.

This gives union an order-characterization role and is a high-value consistency law for the subtype solver.

**Kind:** Derived semantic law  
**Depends on:** T0456–T0459, T0103

### T0554 — `SubtypeByIntersection`

`A <: B` exactly when meeting `A` with `B` yields `A`.

**Kind:** Derived semantic law  
**Depends on:** T0506–T0509, T0103

### T0555 — `SubtypeAlgebraCharacterization`

Subtype order can be characterized simultaneously through both join and meet.

The two equalities are intentionally redundant as a registry invariant: disagreement between them reveals a broken operator implementation or subtype solver.

**Kind:** Derived cross-check law  
**Depends on:** T0553, T0554

### T0556 — `IdentityByJoinAndMeet`

Two types are identical exactly when their join and meet both collapse to one of the operands.

**Kind:** Derived semantic law  
**Depends on:** T0104, T0553, T0554

### T0557 — `IntersectionDistributesOverUnion`

Within a declared distributive fragment, intersection distributes over union at the semantic level.

The compiler is not required to eagerly rewrite the left side into the right side. Semantic equivalence does not imply a mandatory expansion strategy.

**Kind:** Semantic algebra law  
**Depends on:** T0450–T0464, T0500–T0514

### T0558 — `UnionDistributesOverIntersection`

Within a declared distributive fragment, union distributes over intersection at the semantic level.

As with T0557, this is not an instruction to eagerly expand normalized types.

**Kind:** Semantic algebra law  
**Depends on:** T0450–T0464, T0500–T0514, T0557

### T0559 — `NeverAnyDuality`

Bottom and top behave dually under join and meet.

**Kind:** Derived semantic law  
**Depends on:** T0454, T0455, T0504, T0505

### T0560 — `DistinctExactIntersection`

Distinct exact-value types intersect to `Never`.

**Kind:** Cross-family derived law  
**Depends on:** T0358–T0360, T0515

### T0561 — `BoolPartition`

`Bool` is a closed partition into two disjoint exact cases.

**Kind:** Cross-family derived law  
**Depends on:** T0400–T0411, T0465, T0560

### T0562 — `UnitSingletonAlgebra`

`Unit` is the exact singleton type of `()` and lies within the bounded proper-type order.

**Kind:** Cross-family derived law  
**Depends on:** `type.unit`, `type.exact`, T0550

### T0563 — `IdentityFromAlgebraicOrder`

If each type absorbs the other in both algebraic order directions, the types are semantically identical.

This intentionally exercises union, intersection, subtype order, and semantic identity together.

**Kind:** Derived coherence law  
**Depends on:** T0553–T0556

### T0564 — `DisjointSubtypePropagation`

If `A` is narrower than `B` and `B` is disjoint from `C`, then `A` is also disjoint from `C`.

This is important for exact-type reasoning, pattern reachability, and finite-domain elimination.

**Kind:** Derived semantic law  
**Depends on:** T0101, T0515

### T0565 — `SemanticLawDoesNotRequireExpansion`

Semantic distributivity must not be confused with canonical representation.

Within a declared distributive fragment, the compiler may preserve `A & (B | C)` as a compact semantic form while still proving the distributive equivalence. Representation policy belongs to `type.normalization`.

This boundary is normative because blindly treating every valid algebraic equivalence as an eager rewrite rule can make normalization exponential.

**Kind:** Semantic/representation boundary invariant  
**Depends on:** T0557, T0558

## Algebraic structure

With the laws defined through this point, the proper-type universe forms a bounded lattice modulo semantic identity. Fragments declared `distributiveTypeFragment` additionally form distributive lattices:

```text
bottom          Never
top             Any
join            |
meet            &
order           <:
equivalence     ==
```

with:

```text
A <: B
iff
A | B == B
iff
A & B == A
```

The algebra is specified modulo semantic identity rather than source syntax or internal node identity.

## Non-goals and deferred extensions

This file does not introduce a general complement operator.

Accordingly, the following Boolean-algebra laws are **not** currently part of the registry:

```text
A | not A == Any
A & not A == Never
not Never == Any
not Any == Never
```

Closed finite domains may support subtraction or complement-like reasoning in `type.pattern` or a future `type.difference` family without committing the entire Phalcom type universe to first-class complements.

Also deferred:

- canonical distributive normal forms — `type.normalization`
- solver proof completeness for distributive equivalence — `type.solver`
- recursive-type algebra
- quantified/existential-type algebra
- associated-projection reduction across lattice operators
