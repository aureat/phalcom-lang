# Phalcom Type Algebra Registry — `type.reflection-pattern`

**Status:** Normative specification  
**Registry families:** `type.reflection`, `type.pattern`  
**Identifier range used here:** `T0410–T0444`

## Purpose

This file defines coherence laws between static type semantics, public reified reflection, and pattern-space reasoning.

Reflection must expose the same semantic type relations as the compiler whenever sufficient type information is reified. Pattern matching must consume semantic type space using the same identity, subtype, union, intersection, exact-type, and closed-domain laws defined elsewhere in the registry.

Neither reflection nor pattern matching may become an independent second type system.

## Domain and notation

- `reflect(T)` denotes the public reflective representation of type `T`.
- `reified(T)` means sufficient runtime/static metadata exists to answer the relation exactly.
- `runtimeSubtype(A, B)` denotes reflective subtype testing.
- `space(P)` denotes the semantic type/value space matched by pattern `P`.
- `remaining(T, ***Ps)` denotes unmatched space after patterns.
- `covers(P, T)` means pattern `P` covers type space `T`.
- `reachable(P, R)` means pattern `P` intersects remaining space `R`.

## Reflection laws

```phalcom
@id(T0410)
@name(type.reflection.identity)
ReflectionIdentity<T> {
    reflect(T) == reflect(T)
}

@id(T0411)
@name(type.reflection.semantic-identity)
EquivalentTypesReflectEquivalently<A, B> {
    if A == B {
        reflect(A) == reflect(B)
    }
}

@id(T0412)
@name(type.reflection.kind)
ReflectionPreservesKind<T> {
    reflectedKind(reflect(T)) == kind(T)
}

@id(T0413)
@name(type.reflection.subtype.static-consistency)
ReflectionSubtypeStaticConsistency<A, B> {
    if reified(A) and reified(B) {
        runtimeSubtype(reflect(A), reflect(B)) ==
        staticSubtype(A, B)
    }
}

@id(T0414)
@name(type.reflection.identity.static-consistency)
ReflectionIdentityStaticConsistency<A, B> {
    if reified(A) and reified(B) {
        runtimeTypeEqual(reflect(A), reflect(B)) ==
        (A == B)
    }
}

@id(T0415)
@name(type.reflection.union)
ReflectionUnionConsistency<A, B> {
    reflect(A | B) ==
    reflectUnion(reflect(A), reflect(B))
}

@id(T0416)
@name(type.reflection.intersection)
ReflectionIntersectionConsistency<A, B> {
    reflect(A & B) ==
    reflectIntersection(reflect(A), reflect(B))
}

@id(T0417)
@name(type.reflection.never)
ReflectionNeverConsistency {
    reflect(Never).hasInhabitants == false
}

@id(T0418)
@name(type.reflection.any)
ReflectionAnyConsistency<A> {
    if reified(A) {
        runtimeSubtype(reflect(A), reflect(Any)) == true
    }
}

@id(T0419)
@name(type.reflection.exact)
ReflectionExactConsistency<v> {
    reflectedCardinality(reflect(exact(v))) == 1
}

@id(T0420)
@name(type.reflection.projection)
ReflectionProjectionConsistency<T, R> {
    if resolve(T::Item) == R {
        reflect(T::Item) == reflect(R)
    }
}

@id(T0421)
@name(type.reflection.normalization)
ReflectionNormalizationConsistency<T> {
    reflect(T) == reflect(normalize(T))
}

@id(T0422)
@name(type.reflection.incomplete-information.conservative)
ReflectionIncompleteInformationConservative<A, B> {
    if not reified(A) or not reified(B) {
        runtimeReflectionMustNotInventDefinitiveRelation(A, B)
    }
}
```

## Pattern-space laws

```phalcom
@id(T0430)
@name(type.pattern.space.subtype)
PatternSpaceSubtype<P, T> {
    if patternValidFor(P, T) {
        space(P) <: T
    }
}

@id(T0431)
@name(type.pattern.reachable)
PatternReachable<P, R> {
    reachable(P, R) iff space(P) & R != Never
}

@id(T0432)
@name(type.pattern.unreachable)
PatternUnreachable<P, R> {
    not reachable(P, R) iff space(P) & R == Never
}

@id(T0433)
@name(type.pattern.consume)
PatternConsumesSpace<P, R> {
    remainingAfter(R, P) ==
    subtract(R, space(P))
}

@id(T0434)
@name(type.pattern.exhaustive)
PatternSetExhaustive<T, ***Ps> {
    exhaustive(T, ***Ps) iff
    remaining(T, ***Ps) == Never
}

@id(T0435)
@name(type.pattern.exact)
ExactPatternSpace<v> {
    space(pattern(v)) == exact(v)
}

@id(T0436)
@name(type.pattern.wildcard)
WildcardPatternSpace<T> {
    space(wildcardFor(T)) == T
}

@id(T0437)
@name(type.pattern.union)
UnionPatternSpace<P1, P2> {
    space(P1 or P2) ==
    space(P1) | space(P2)
}

@id(T0438)
@name(type.pattern.type-test)
TypeTestPatternSpace<A, T> {
    space(typePattern(A) within T) ==
    A & T
}

@id(T0439)
@name(type.pattern.boolean.exhaustive)
BooleanPatternsExhaustive {
    remaining(Bool, true, false) == Never
}

@id(T0440)
@name(type.pattern.closed-data.exhaustive)
ClosedDataPatternsExhaustive<D> {
    if closed(D) and patternsCoverAll(variants(D)) {
        remaining(D, variants(D)) == Never
    }
}

@id(T0441)
@name(type.pattern.variant.disjoint)
DistinctVariantPatternsDisjoint<D, V1, V2> {
    if V1 != V2 {
        space(pattern(D::V1)) &
        space(pattern(D::V2)) == Never
    }
}

@id(T0442)
@name(type.pattern.order.remaining-space)
PatternOrderUsesRemainingSpace<T, P1, P2> {
    reachability(P2) isEvaluatedAgainst
    remainingAfter(T, P1)
}

@id(T0443)
@name(type.pattern.guard.not-total-by-default)
GuardedPatternNotTotal<P, G> {
    if not staticallyTrue(G) {
        guarded(P, G) doesNotConsumeAll space(P)
    }
}

@id(T0444)
@name(type.pattern.solver.consistency)
PatternSolverTypeSolverConsistency<A, B> {
    patternDisjoint(A, B) iff
    solve(A & B == Never) == Proven
}
```

## Reflection law notes

### T0410 / T0411 — Reflective identity

Public reflection must preserve semantic identity.

Distinct syntax or normalization paths must not create semantically distinct reflective type objects for the same type, even if implementation handles differ.

**Kind:** Reflection coherence laws  
**Depends on:** T0001–T0003

### T0412 — `ReflectionPreservesKind`

Reflection must expose the same kind-level classification as static semantics.

**Kind:** Cross-subsystem coherence law  
**Depends on:** T0010–T0018

### T0413 — `ReflectionSubtypeStaticConsistency`

Where both operands are sufficiently reified, runtime reflective subtype testing and static subtype semantics must agree.

**Kind:** Cross-subsystem coherence law  
**Depends on:** T0020–T0026

### T0414 — `ReflectionIdentityStaticConsistency`

Reified runtime type equality must agree with semantic static identity.

**Kind:** Cross-subsystem coherence law  
**Depends on:** T0001–T0003

### T0415 / T0416 — Union and intersection reflection

Reflective construction of union/intersection must obey the same semantic algebra as source-level types.

**Kind:** Reflection algebra laws  
**Depends on:** T0100–T0155

### T0417–T0419 — Foundational reflected domains

Reflection of `Never`, `Any`, and exact types must preserve their defining semantic properties.

**Kind:** Cross-family coherence laws  
**Depends on:** T0030–T0071

### T0420 — `ReflectionProjectionConsistency`

Resolved projections reflect as their resolved semantic result.

**Kind:** Cross-family coherence law  
**Depends on:** T0315

### T0421 — `ReflectionNormalizationConsistency`

Normalization must be invisible to semantic reflection.

**Kind:** Cross-family coherence law  
**Depends on:** T0340–T0365

### T0422 — `ReflectionIncompleteInformationConservative`

If reflection lacks enough information to decide a relation, it must expose uncertainty or unsupportedness rather than fabricate a definitive semantic answer.

**Kind:** Reflection-safety invariant  
**Depends on:** T0372–T0375

## Pattern law notes

### T0430 — `PatternSpaceSubtype`

A valid pattern cannot match values outside the scrutinee domain for which it is checked.

**Kind:** Pattern semantic law  
**Depends on:** T0020–T0026

### T0431 / T0432 — Reachability

A pattern is reachable exactly when its space has non-empty intersection with the remaining scrutinee space.

`Never` is therefore the canonical representation of impossible pattern space.

**Kind:** Pattern algebra laws  
**Depends on:** T0120–T0135

### T0433 — `PatternConsumesSpace`

A successful unguarded pattern removes its matched semantic space from the remainder.

The `subtract` operation is pattern-space machinery and need not imply a globally exposed first-class type-difference operator.

**Kind:** Pattern semantic law

### T0434 — `PatternSetExhaustive`

A match is exhaustive exactly when no semantic space remains.

**Kind:** Pattern exhaustiveness law  
**Depends on:** T0032–T0036, T0433

### T0435 — `ExactPatternSpace`

A literal/exact pattern matches precisely the corresponding exact-value type.

**Kind:** Cross-family law  
**Depends on:** T0060–T0071

### T0436 — `WildcardPatternSpace`

A wildcard covers the entire current scrutinee domain.

**Kind:** Pattern semantic law

### T0437 — `UnionPatternSpace`

An alternative/or-pattern covers the union of its branch spaces.

**Kind:** Cross-family pattern law  
**Depends on:** T0100–T0115

### T0438 — `TypeTestPatternSpace`

A type-test pattern covers the intersection between the tested type and current scrutinee domain.

**Kind:** Cross-family pattern law  
**Depends on:** T0120–T0135

### T0439 — `BooleanPatternsExhaustive`

Matching both Boolean exact values exhausts `Bool`.

**Kind:** Derived finite-domain law  
**Depends on:** T0151, T0434, T0435

### T0440 / T0441 — Closed data domains

Closed `data` and `type data` domains support exact exhaustiveness and variant-disjointness reasoning.

**Kind:** Cross-family pattern laws  
**Depends on:** T0280–T0304

### T0442 — `PatternOrderUsesRemainingSpace`

Reachability of a later arm is evaluated after earlier unguarded coverage has been removed.

**Kind:** Pattern control-flow invariant

### T0443 — `GuardedPatternNotTotal`

A guarded pattern does not consume its full structural pattern space unless the guard is statically proven true.

This avoids false exhaustiveness.

**Kind:** Pattern soundness invariant

### T0444 — `PatternSolverTypeSolverConsistency`

Pattern-space disjointness and type intersection emptiness must agree when the type solver proves the relation.

**Kind:** Cross-subsystem coherence law  
**Depends on:** T0135, T0370–T0402

## Required implementation invariants

Reflection and pattern matching are consumers of the same semantic type algebra. They must not maintain incompatible duplicate notions of:

- identity,
- subtyping,
- exact types,
- union,
- intersection,
- finite closed domains,
- projections,
- or `Never`.

A runtime reflective API may expose less information than the compiler possesses, but it must not expose contradictory information.

Pattern analysis may use specialized space representations for performance, but those representations must be semantically equivalent to the registry laws.

## Explicitly deferred laws

- pattern bindings and variable type refinement
- destructuring of object interfaces/state
- range-pattern algebra
- guard theorem proving
- nested GADT pattern refinement
- runtime `is` operator syntax/behavior
- `.type` versus `.class` reflection surface semantics
- metaclass reflection
