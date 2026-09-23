# Phalcom Type Algebra Registry — `type.algebra`

**Status:** Normative specification  
**Registry family:** `type.algebra`  
**Identifier range used here:** `T0140–T0155`

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
@id(T0140)
@name(type.algebra.lattice.bounded)
TypeAlgebraBounded<A> {
    Never <: A
    A <: Any
}

@id(T0141)
@name(type.algebra.absorption.union)
UnionAbsorption<A, B> {
    A | (A & B) == A
}

@id(T0142)
@name(type.algebra.absorption.intersection)
IntersectionAbsorption<A, B> {
    A & (A | B) == A
}

@id(T0143)
@name(type.algebra.order.union-characterization)
SubtypeByUnion<A, B> {
    A <: B iff A | B == B
}

@id(T0144)
@name(type.algebra.order.intersection-characterization)
SubtypeByIntersection<A, B> {
    A <: B iff A & B == A
}

@id(T0145)
@name(type.algebra.order.dual-characterization)
SubtypeAlgebraCharacterization<A, B> {
    A <: B iff A | B == B and A & B == A
}

@id(T0146)
@name(type.algebra.identity.join-meet)
IdentityByJoinAndMeet<A, B> {
    A == B iff A | B == A and A & B == A
}

@id(T0147)
@name(type.algebra.distributive.intersection-over-union)
IntersectionDistributesOverUnion<A, B, C> {
    A & (B | C) == (A & B) | (A & C)
}

@id(T0148)
@name(type.algebra.distributive.union-over-intersection)
UnionDistributesOverIntersection<A, B, C> {
    A | (B & C) == (A | B) & (A | C)
}

@id(T0149)
@name(type.algebra.duality.never-any)
NeverAnyDuality {
    Never | Any == Any
    Never & Any == Never
}

@id(T0150)
@name(type.algebra.exact.distinct-intersection)
DistinctExactIntersection<a, b> {
    if a != b {
        exact(a) & exact(b) == Never
    }
}

@id(T0151)
@name(type.algebra.boolean.partition)
BoolPartition {
    Bool == exact(true) | exact(false)
    exact(true) & exact(false) == Never
}

@id(T0152)
@name(type.algebra.unit.singleton)
UnitSingletonAlgebra {
    Unit == exact(())
    Never <: Unit
    Unit <: Any
}

@id(T0153)
@name(type.algebra.order.identity-from-mutual)
IdentityFromAlgebraicOrder<A, B> {
    if A | B == B and A & B == A
        and B | A == A and B & A == B
    {
        A == B
    }
}

@id(T0154)
@name(type.algebra.disjoint.order)
DisjointSubtypePropagation<A, B, C> {
    if A <: B and disjoint(B, C) {
        disjoint(A, C)
    }
}

@id(T0155)
@name(type.algebra.semantic-normalization-boundary)
SemanticLawDoesNotRequireExpansion<A, B, C> {
    A & (B | C) == (A & B) | (A & C)

    // does not require:
    // normalize(A & (B | C))
    // to structurally expand into
    // normalize((A & B) | (A & C))
}
```

## Law notes

### T0140 — `TypeAlgebraBounded`

The proper-type subtype order is bounded by `Never` and `Any`.

**Kind:** Derived semantic law  
**Depends on:** T0031, T0041

### T0141 — `UnionAbsorption`

Union absorbs an intersection that already includes the union operand.

**Kind:** Derived semantic algebra law  
**Depends on:** T0106–T0109, T0126–T0129

### T0142 — `IntersectionAbsorption`

Intersection absorbs a union that already includes the intersection operand.

**Kind:** Derived semantic algebra law  
**Depends on:** T0106–T0109, T0126–T0129

### T0143 — `SubtypeByUnion`

`A <: B` exactly when joining `A` with `B` yields `B`.

This gives union an order-characterization role and is a high-value consistency law for the subtype solver.

**Kind:** Derived semantic law  
**Depends on:** T0106–T0109, T0023

### T0144 — `SubtypeByIntersection`

`A <: B` exactly when meeting `A` with `B` yields `A`.

**Kind:** Derived semantic law  
**Depends on:** T0126–T0129, T0023

### T0145 — `SubtypeAlgebraCharacterization`

Subtype order can be characterized simultaneously through both join and meet.

The two equalities are intentionally redundant as a registry invariant: disagreement between them reveals a broken operator implementation or subtype solver.

**Kind:** Derived cross-check law  
**Depends on:** T0143, T0144

### T0146 — `IdentityByJoinAndMeet`

Two types are identical exactly when their join and meet both collapse to one of the operands.

**Kind:** Derived semantic law  
**Depends on:** T0024, T0143, T0144

### T0147 — `IntersectionDistributesOverUnion`

Intersection distributes over union at the semantic level.

The compiler is not required to eagerly rewrite the left side into the right side. Semantic equivalence does not imply a mandatory expansion strategy.

**Kind:** Semantic algebra law  
**Depends on:** T0100–T0114, T0120–T0134

### T0148 — `UnionDistributesOverIntersection`

Union distributes over intersection at the semantic level.

As with T0147, this is not an instruction to eagerly expand normalized types.

**Kind:** Semantic algebra law  
**Depends on:** T0100–T0114, T0120–T0134, T0147

### T0149 — `NeverAnyDuality`

Bottom and top behave dually under join and meet.

**Kind:** Derived semantic law  
**Depends on:** T0104, T0105, T0124, T0125

### T0150 — `DistinctExactIntersection`

Distinct exact-value types intersect to `Never`.

**Kind:** Cross-family derived law  
**Depends on:** T0067–T0069, T0135

### T0151 — `BoolPartition`

`Bool` is a closed partition into two disjoint exact cases.

**Kind:** Cross-family derived law  
**Depends on:** T0080–T0090, T0115, T0150

### T0152 — `UnitSingletonAlgebra`

`Unit` is the exact singleton type of `()` and lies within the bounded proper-type order.

**Kind:** Cross-family derived law  
**Depends on:** T0050–T0057, T0060–T0071, T0140

### T0153 — `IdentityFromAlgebraicOrder`

If each type absorbs the other in both algebraic order directions, the types are semantically identical.

This intentionally exercises union, intersection, subtype order, and semantic identity together.

**Kind:** Derived coherence law  
**Depends on:** T0143–T0146

### T0154 — `DisjointSubtypePropagation`

If `A` is narrower than `B` and `B` is disjoint from `C`, then `A` is also disjoint from `C`.

This is important for exact-type reasoning, pattern reachability, and finite-domain elimination.

**Kind:** Derived semantic law  
**Depends on:** T0021, T0135

### T0155 — `SemanticLawDoesNotRequireExpansion`

Semantic distributivity must not be confused with canonical representation.

The compiler may preserve `A & (B | C)` as a compact semantic form while still proving it equivalent to `(A & B) | (A & C)`. Representation policy belongs to `type.normalization`.

This boundary is normative because blindly treating every valid algebraic equivalence as an eager rewrite rule can make normalization exponential.

**Kind:** Semantic/representation boundary invariant  
**Depends on:** T0147, T0148

## Algebraic structure

With the laws defined through this point, the proper-type universe forms a bounded distributive lattice modulo semantic identity:

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
