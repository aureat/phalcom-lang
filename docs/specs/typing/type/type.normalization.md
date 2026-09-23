# Phalcom Type Algebra Registry — `type.normalization`

**Status:** Normative implementation-facing specification  
**Registry family:** `type.normalization`  
**Identifier range used here:** `T0950–T0975`

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
@id(T0950)
@name(type.normalization.identity-preserving)
NormalizationPreservesIdentity<T> {
    normalize(T) == T
}

@id(T0951)
@name(type.normalization.idempotent)
NormalizationIdempotent<T> {
    normalize(normalize(T)) == normalize(T)
}

@id(T0952)
@name(type.normalization.deterministic)
NormalizationDeterministic<T> {
    normalize(T) == normalize(T)
}

@id(T0953)
@name(type.normalization.identity-substitution)
NormalizationIdentitySubstitution<A, B> {
    if A == B {
        normalize(A) == normalize(B)
    }
}

@id(T0954)
@name(type.normalization.canonical)
NormalizationProducesCanonical<T> {
    if canonicalizable(T) {
        canonical(normalize(T))
    }
}

@id(T0955)
@name(type.normalization.canonical.identity)
CanonicalEquivalentTypesShareForm<A, B> {
    if canonicalizable(A)
        and canonicalizable(B)
        and A == B
    {
        normalize(A) structurallyEquals normalize(B)
    }
}

@id(T0956)
@name(type.normalization.union.flatten)
NormalizeUnionFlatten<A, B, C> {
    normalize((A | B) | C) ==
    normalize(A | B | C)
}

@id(T0957)
@name(type.normalization.union.unique)
NormalizeUnionUnique<***Ts> {
    noDuplicates(members(normalize(union<***Ts>)))
}

@id(T0958)
@name(type.normalization.union.never-elimination)
NormalizeUnionNever<A> {
    normalize(A | Never) == normalize(A)
}

@id(T0959)
@name(type.normalization.union.any-collapse)
NormalizeUnionAny<A> {
    normalize(A | Any) == normalize(Any)
}

@id(T0960)
@name(type.normalization.union.subtype-redundancy)
NormalizeUnionSubtypeRedundancy<A, B> {
    if A <: B {
        normalize(A | B) == normalize(B)
    }
}

@id(T0961)
@name(type.normalization.intersection.flatten)
NormalizeIntersectionFlatten<A, B, C> {
    normalize((A & B) & C) ==
    normalize(A & B & C)
}

@id(T0962)
@name(type.normalization.intersection.unique)
NormalizeIntersectionUnique<***Ts> {
    noDuplicates(members(normalize(intersection<***Ts>)))
}

@id(T0963)
@name(type.normalization.intersection.any-elimination)
NormalizeIntersectionAny<A> {
    normalize(A & Any) == normalize(A)
}

@id(T0964)
@name(type.normalization.intersection.never-collapse)
NormalizeIntersectionNever<A> {
    normalize(A & Never) == normalize(Never)
}

@id(T0965)
@name(type.normalization.intersection.supertype-redundancy)
NormalizeIntersectionSupertypeRedundancy<A, B> {
    if A <: B {
        normalize(A & B) == normalize(A)
    }
}

@id(T0966)
@name(type.normalization.order.deterministic)
NormalizedMemberOrderingDeterministic<T> {
    memberOrder(normalize(T)) == canonicalMemberOrder(T)
}

@id(T0967)
@name(type.normalization.projection.reduce-when-known)
NormalizeResolvedProjection<E, R> {
    if resolve(E) == R {
        normalize(E) == normalize(R)
    }
}

@id(T0968)
@name(type.normalization.projection.preserve-when-unknown)
NormalizeUnresolvedProjectionConservatively<E> {
    if unresolved(E) {
        normalize(E) doesNotInventConcreteResult
    }
}

@id(T0969)
@name(type.normalization.distributivity.not-mandatory)
NormalizationNeedNotDistribute<A, B, C> {
    if distributiveTypeFragment(A, B, C) {
        A & (B | C) == (A & B) | (A & C)
        notRequiresStructuralExpansion(A & (B | C))
    }
}
```

```phalcom
@id(T0970)
@name(type.normalization.subtype-semantic-consistency)
NormalizationSubtypeSemanticConsistency<A, B> {
    A <: B iff normalize(A) <: normalize(B)
}

@id(T0971)
@name(type.normalization.disjoint-consistency)
NormalizationDisjointConsistency<A, B> {
    disjoint(A, B) ==
    disjoint(normalize(A), normalize(B))
}

@id(T0972)
@name(type.normalization.kind-preserving)
NormalizationPreservesKind<T> {
    kind(normalize(T)) == kind(T)
}

@id(T0973)
@name(type.normalization.terminating)
NormalizationTerminates<T> {
    if supportedNormalizationFragment(T) {
        terminates(normalize(T))
    }
}

@id(T0974)
@name(type.normalization.cache-independent)
NormalizationCacheIndependent<T> {
    normalizeWithColdCache(T) ==
    normalizeWithWarmCache(T)
}

@id(T0975)
@name(type.normalization.source-order-independent)
NormalizationSourceOrderIndependent<A, B> {
    normalize(A | B) == normalize(B | A)
    normalize(A & B) == normalize(B & A)
}
```

## Law notes

### T0950 — `NormalizationPreservesIdentity`

Normalization may change representation but never semantic meaning.

**Kind:** Fundamental normalization invariant  
**Depends on:** T0001–T0003

### T0951 — `NormalizationIdempotent`

Normalizing an already normalized type produces no semantic change.

A canonical implementation should also avoid unnecessary representation churn.

**Kind:** Representation invariant  
**Depends on:** T0950

### T0952 / T0953 — Determinism and substitution

Normalization must not depend on evaluation order, cache state, or syntactic accidents once semantic identity is fixed.

**Kind:** Coherence invariants  
**Depends on:** T0001–T0003

### T0954 / T0955 — Canonical form

Where canonicalization is declared complete, normalization produces one stable structural form for each semantic equivalence class.

This completeness claim is intentionally conditional on `canonicalizable`.

**Kind:** Representation invariants  
**Depends on:** T0950–T0953

### T0956–T0960 — Union normalization

Normalized unions flatten associative nesting, remove duplicates, eliminate `Never`, collapse on `Any`, and remove subtype-redundant members.

**Kind:** Derived normalization laws  
**Depends on:** T0450–T0465

### T0961–T0965 — Intersection normalization

Normalized intersections flatten associative nesting, remove duplicates, eliminate `Any`, collapse on `Never`, and remove redundant supertypes.

**Kind:** Derived normalization laws  
**Depends on:** T0500–T0515

### T0966 — `NormalizedMemberOrderingDeterministic`

Canonical member order must be deterministic.

The semantic algebra is unordered, but deterministic representation is essential for stable hashing, caching, diagnostics, serialization, and incremental builds.

**Kind:** Representation invariant  
**Depends on:** T0451, T0501

### T0967 / T0968 — Projection normalization

Known projections reduce consistently. Unknown projections remain symbolic and conservative.

**Kind:** Cross-family invariants  
**Depends on:** T0905–T0910

### T0969 — `NormalizationNeedNotDistribute`

Semantic distributivity does not require eager distributive expansion.

This protects the normalizer from exponential growth.

**Kind:** Semantic/representation boundary invariant  
**Depends on:** T0557, T0558, T0565

### T0970 — `NormalizationSubtypeSemanticConsistency`

Normalization preserves the semantic subtype relation. This law intentionally speaks about semantic truth, not exact solver proof-state behavior. The canonical solver entrypoint may impose the stronger proof-state invariant in `type.inference-solver`.

**Kind:** Semantic normalization invariant  
**Depends on:** T0950, T0100–T0106

### T0971 — `NormalizationDisjointConsistency`

Normalization must preserve semantic disjointness.

**Kind:** Cross-subsystem coherence invariant  
**Depends on:** T0515, T0950

### T0972 — `NormalizationPreservesKind`

Normalization may not move a type-level entity into a different kind.

**Kind:** Kinding invariant  
**Depends on:** T0050–T0060, T0950

### T0973 — `NormalizationTerminates`

Every fragment advertised as supported by canonical normalization must terminate.

Recursive or intentionally symbolic fragments may use bounded reduction instead, but must say so explicitly.

**Kind:** Operational invariant  
**Depends on:** none beyond fragment definition

### T0974 — `NormalizationCacheIndependent`

Caching may improve performance but cannot alter normalized semantic results.

**Kind:** Implementation coherence invariant  
**Depends on:** T0952

### T0975 — `NormalizationSourceOrderIndependent`

Commutative operator normalization cannot depend on source operand order.

**Kind:** Representation coherence invariant  
**Depends on:** T0451, T0501, T0966

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
