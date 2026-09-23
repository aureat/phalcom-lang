# Phalcom Type Algebra Registry — `type.normalization`

**Status:** Normative implementation-facing specification  
**Registry family:** `type.normalization`  
**Identifier range used here:** `T0340–T0365`

## Purpose

This file defines the semantic and representation invariants of type normalization.

Normalization maps semantically equivalent type expressions into stable canonical forms where the relevant fragment admits canonicalization. It must preserve meaning, be deterministic, terminate for supported fragments, and remain idempotent.

Normalization is not the definition of semantic identity. Some semantically equivalent types may remain represented differently when canonicalization is intentionally incomplete or would cause pathological expansion.

## Domain and notation

- `normalize(T)` denotes canonical normalization.
- `canonical(T)` means `T` is in the chosen canonical form for its fragment.
- `canonicalizable(T)` means the fragment has a complete canonicalization strategy.
- `members(U)` denotes flattened union/intersection members where meaningful.
- `is` denotes representation identity only where explicitly stated.

## Laws

```phalcom
@id(T0340)
@name(type.normalization.identity-preserving)
NormalizationPreservesIdentity<T> {
    normalize(T) == T
}

@id(T0341)
@name(type.normalization.idempotent)
NormalizationIdempotent<T> {
    normalize(normalize(T)) == normalize(T)
}

@id(T0342)
@name(type.normalization.deterministic)
NormalizationDeterministic<T> {
    normalize(T) == normalize(T)
}

@id(T0343)
@name(type.normalization.identity-substitution)
NormalizationIdentitySubstitution<A, B> {
    if A == B {
        normalize(A) == normalize(B)
    }
}

@id(T0344)
@name(type.normalization.canonical)
NormalizationProducesCanonical<T> {
    if canonicalizable(T) {
        canonical(normalize(T))
    }
}

@id(T0345)
@name(type.normalization.canonical.identity)
CanonicalEquivalentTypesShareForm<A, B> {
    if canonicalizable(A)
        and canonicalizable(B)
        and A == B
    {
        normalize(A) structurallyEquals normalize(B)
    }
}

@id(T0346)
@name(type.normalization.union.flatten)
NormalizeUnionFlatten<A, B, C> {
    normalize((A | B) | C) ==
    normalize(A | B | C)
}

@id(T0347)
@name(type.normalization.union.unique)
NormalizeUnionUnique<***Ts> {
    noDuplicates(members(normalize(union<***Ts>)))
}

@id(T0348)
@name(type.normalization.union.never-elimination)
NormalizeUnionNever<A> {
    normalize(A | Never) == normalize(A)
}

@id(T0349)
@name(type.normalization.union.any-collapse)
NormalizeUnionAny<A> {
    normalize(A | Any) == normalize(Any)
}

@id(T0350)
@name(type.normalization.union.subtype-redundancy)
NormalizeUnionSubtypeRedundancy<A, B> {
    if A <: B {
        normalize(A | B) == normalize(B)
    }
}

@id(T0351)
@name(type.normalization.intersection.flatten)
NormalizeIntersectionFlatten<A, B, C> {
    normalize((A & B) & C) ==
    normalize(A & B & C)
}

@id(T0352)
@name(type.normalization.intersection.unique)
NormalizeIntersectionUnique<***Ts> {
    noDuplicates(members(normalize(intersection<***Ts>)))
}

@id(T0353)
@name(type.normalization.intersection.any-elimination)
NormalizeIntersectionAny<A> {
    normalize(A & Any) == normalize(A)
}

@id(T0354)
@name(type.normalization.intersection.never-collapse)
NormalizeIntersectionNever<A> {
    normalize(A & Never) == normalize(Never)
}

@id(T0355)
@name(type.normalization.intersection.supertype-redundancy)
NormalizeIntersectionSupertypeRedundancy<A, B> {
    if A <: B {
        normalize(A & B) == normalize(A)
    }
}

@id(T0356)
@name(type.normalization.order.deterministic)
NormalizedMemberOrderingDeterministic<T> {
    memberOrder(normalize(T)) == canonicalMemberOrder(T)
}

@id(T0357)
@name(type.normalization.projection.reduce-when-known)
NormalizeResolvedProjection<E, R> {
    if resolve(E) == R {
        normalize(E) == normalize(R)
    }
}

@id(T0358)
@name(type.normalization.projection.preserve-when-unknown)
NormalizeUnresolvedProjectionConservatively<E> {
    if unresolved(E) {
        normalize(E) doesNotInventConcreteResult
    }
}

@id(T0359)
@name(type.normalization.distributivity.not-mandatory)
NormalizationNeedNotDistribute<A, B, C> {
    A & (B | C) == (A & B) | (A & C)

    // but canonical form is not required to eagerly expand
}
```

```phalcom
@id(T0360)
@name(type.normalization.subtype-consistency)
NormalizationSubtypeConsistency<A, B> {
    solve(A <: B) ==
    solve(normalize(A) <: normalize(B))
}

@id(T0361)
@name(type.normalization.disjoint-consistency)
NormalizationDisjointConsistency<A, B> {
    disjoint(A, B) ==
    disjoint(normalize(A), normalize(B))
}

@id(T0362)
@name(type.normalization.kind-preserving)
NormalizationPreservesKind<T> {
    kind(normalize(T)) == kind(T)
}

@id(T0363)
@name(type.normalization.terminating)
NormalizationTerminates<T> {
    if supportedNormalizationFragment(T) {
        terminates(normalize(T))
    }
}

@id(T0364)
@name(type.normalization.cache-independent)
NormalizationCacheIndependent<T> {
    normalizeWithColdCache(T) ==
    normalizeWithWarmCache(T)
}

@id(T0365)
@name(type.normalization.source-order-independent)
NormalizationSourceOrderIndependent<A, B> {
    normalize(A | B) == normalize(B | A)
    normalize(A & B) == normalize(B & A)
}
```

## Law notes

### T0340 — `NormalizationPreservesIdentity`

Normalization may change representation but never semantic meaning.

**Kind:** Fundamental normalization invariant  
**Depends on:** T0001–T0003

### T0341 — `NormalizationIdempotent`

Normalizing an already normalized type produces no semantic change.

A canonical implementation should also avoid unnecessary representation churn.

**Kind:** Representation invariant  
**Depends on:** T0340

### T0342 / T0343 — Determinism and substitution

Normalization must not depend on evaluation order, cache state, or syntactic accidents once semantic identity is fixed.

**Kind:** Coherence invariants  
**Depends on:** T0001–T0003

### T0344 / T0345 — Canonical form

Where canonicalization is declared complete, normalization produces one stable structural form for each semantic equivalence class.

This completeness claim is intentionally conditional on `canonicalizable`.

**Kind:** Representation invariants  
**Depends on:** T0340–T0343

### T0346–T0350 — Union normalization

Normalized unions flatten associative nesting, remove duplicates, eliminate `Never`, collapse on `Any`, and remove subtype-redundant members.

**Kind:** Derived normalization laws  
**Depends on:** T0100–T0115

### T0351–T0355 — Intersection normalization

Normalized intersections flatten associative nesting, remove duplicates, eliminate `Any`, collapse on `Never`, and remove redundant supertypes.

**Kind:** Derived normalization laws  
**Depends on:** T0120–T0135

### T0356 — `NormalizedMemberOrderingDeterministic`

Canonical member order must be deterministic.

The semantic algebra is unordered, but deterministic representation is essential for stable hashing, caching, diagnostics, serialization, and incremental builds.

**Kind:** Representation invariant  
**Depends on:** T0101, T0121

### T0357 / T0358 — Projection normalization

Known projections reduce consistently. Unknown projections remain symbolic and conservative.

**Kind:** Cross-family invariants  
**Depends on:** T0315–T0320

### T0359 — `NormalizationNeedNotDistribute`

Semantic distributivity does not require eager distributive expansion.

This protects the normalizer from exponential growth.

**Kind:** Semantic/representation boundary invariant  
**Depends on:** T0147, T0148, T0155

### T0360 — `NormalizationSubtypeConsistency`

Subtype solving must produce the same semantic result before and after normalization.

**Kind:** Cross-subsystem coherence invariant  
**Depends on:** T0340, T0020–T0026

### T0361 — `NormalizationDisjointConsistency`

Normalization must preserve semantic disjointness.

**Kind:** Cross-subsystem coherence invariant  
**Depends on:** T0135, T0340

### T0362 — `NormalizationPreservesKind`

Normalization may not move a type-level entity into a different kind.

**Kind:** Kinding invariant  
**Depends on:** T0010–T0018, T0340

### T0363 — `NormalizationTerminates`

Every fragment advertised as supported by canonical normalization must terminate.

Recursive or intentionally symbolic fragments may use bounded reduction instead, but must say so explicitly.

**Kind:** Operational invariant  
**Depends on:** none beyond fragment definition

### T0364 — `NormalizationCacheIndependent`

Caching may improve performance but cannot alter normalized semantic results.

**Kind:** Implementation coherence invariant  
**Depends on:** T0342

### T0365 — `NormalizationSourceOrderIndependent`

Commutative operator normalization cannot depend on source operand order.

**Kind:** Representation coherence invariant  
**Depends on:** T0101, T0121, T0356

## Canonicalization completeness boundary

The following implication is valid only for a fragment whose normalizer is declared complete:

```phalcom
if canonicalizable(A)
    and canonicalizable(B)
    and normalize(A) structurallyEquals normalize(B)
{
    A == B
}
```

The converse must not be globally assumed for unresolved projections, recursive types, incomplete type functions, existential types, or other symbolic fragments unless separately specified.

## Explicitly deferred laws

- hash-consing/pointer identity requirements
- recursive-type canonicalization
- alpha-equivalence for quantified types
- normalization budgets
- diagnostics-preserving source forms
- incremental invalidation mechanics
