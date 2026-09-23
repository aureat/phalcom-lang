# Phalcom Type Algebra Registry — `type.projection-type-function`

**Status:** Normative specification  
**Registry families:** `type.projection`, `type.associated`, `type.type-function`  
**Identifier range used here:** `T0310–T0334`

## Purpose

This file defines semantic laws for associated-type projection and type-level function application.

A projection such as `T::Item` and a type function application such as `F<A>` are type-level computations. Their results must be deterministic, kind-correct, substitutive under semantic identity, and conservative when unresolved.

The compiler may delay reduction, preserve symbolic projections, or normalize them eagerly when sufficient evidence exists. Those implementation choices must not change semantic results.

## Domain and notation

Unless stated otherwise:

- `T::Item` denotes an associated-type projection.
- `T::At<I>` denotes a parameterized associated-type projection.
- `F<A>` denotes type-level function application.
- `resolve(E)` denotes semantic reduction when sufficient evidence exists.
- `resolved(E)` means the semantic result is known.
- `unresolved(E)` means reduction cannot currently be completed.
- `K`, `K1`, and `K2` denote kinds.
- `==` means semantic type identity.
- `<:` means semantic subtype.

## Laws

```phalcom
@id(T0310)
@name(type.projection.deterministic)
ProjectionDeterministic<T> {
    T::Item == T::Item
}

@id(T0311)
@name(type.projection.receiver.identity-substitution)
ProjectionReceiverIdentitySubstitution<A, B> {
    if A == B {
        A::Item == B::Item
    }
}

@id(T0312)
@name(type.projection.argument.identity-substitution)
ProjectionArgumentIdentitySubstitution<T, A, B> {
    if A == B {
        T::At<A> == T::At<B>
    }
}

@id(T0313)
@name(type.projection.kind)
ProjectionKind<T, K> {
    if declaredProjectionKind(T::Item) == K {
        kind(T::Item) == K
    }
}

@id(T0314)
@name(type.projection.parameterized.kind)
ParameterizedProjectionKind<T, A, K1, K2> {
    if declaredProjectionKind(T::At) == K1 -> K2
        and kind(A) == K1
    {
        kind(T::At<A>) == K2
    }
}

@id(T0315)
@name(type.projection.reduction)
ProjectionReduction<T, R> {
    if resolve(T::Item) == R {
        T::Item == R
    }
}

@id(T0316)
@name(type.projection.parameterized.reduction)
ParameterizedProjectionReduction<T, A, R> {
    if resolve(T::At<A>) == R {
        T::At<A> == R
    }
}

@id(T0317)
@name(type.projection.reduction.identity-stable)
ProjectionReductionIdentityStable<T, R1, R2> {
    if resolve(T::Item) == R1
        and resolve(T::Item) == R2
    {
        R1 == R2
    }
}

@id(T0318)
@name(type.projection.unresolved.no-invention)
UnresolvedProjectionInventsNoResult<T, R> {
    if unresolved(T::Item) {
        not proven(T::Item == R)
    }
}

@id(T0319)
@name(type.projection.unresolved.not-any)
UnresolvedProjectionIsNotAny<T> {
    if unresolved(T::Item) {
        not proven(T::Item == Any)
    }
}

@id(T0320)
@name(type.projection.unresolved.not-never)
UnresolvedProjectionIsNotNever<T> {
    if unresolved(T::Item) {
        not proven(T::Item == Never)
    }
}

@id(T0321)
@name(type.projection.constraint-preservation)
ProjectionConstraintPreservation<T, R, C> {
    if declaredConstraint(T::Item, C)
        and resolve(T::Item) == R
    {
        satisfies(R, C)
    }
}

@id(T0322)
@name(type.projection.normalization-congruence)
ProjectionNormalizationCongruence<T> {
    normalize(T::Item) == normalize(resolveIfPossible(T::Item))
}

@id(T0323)
@name(type.type-function.kind)
TypeFunctionApplicationKind<F, A, K1, K2> {
    if kind(F) == K1 -> K2
        and kind(A) == K1
    {
        kind(F<A>) == K2
    }
}

@id(T0324)
@name(type.type-function.deterministic)
TypeFunctionDeterministic<F, A> {
    F<A> == F<A>
}

@id(T0325)
@name(type.type-function.argument.identity-substitution)
TypeFunctionArgumentIdentitySubstitution<F, A, B> {
    if A == B {
        F<A> == F<B>
    }
}

@id(T0326)
@name(type.type-function.function.identity-substitution)
TypeFunctionIdentitySubstitution<F, G, A> {
    if F == G {
        F<A> == G<A>
    }
}

@id(T0327)
@name(type.type-function.reduction)
TypeFunctionReduction<F, A, R> {
    if resolve(F<A>) == R {
        F<A> == R
    }
}

@id(T0328)
@name(type.type-function.reduction.identity-stable)
TypeFunctionReductionIdentityStable<F, A, R1, R2> {
    if resolve(F<A>) == R1
        and resolve(F<A>) == R2
    {
        R1 == R2
    }
}

@id(T0329)
@name(type.type-function.result.constraint)
TypeFunctionResultConstraint<F, A, R, C> {
    if declaredResultConstraint(F, C)
        and resolve(F<A>) == R
    {
        satisfies(R, C)
    }
}

@id(T0330)
@name(type.type-function.unresolved.no-invention)
UnresolvedTypeFunctionInventsNoResult<F, A, R> {
    if unresolved(F<A>) {
        not proven(F<A> == R)
    }
}

@id(T0331)
@name(type.type-function.pure)
TypeFunctionPure<F, A> {
    evaluateType(F<A>) == evaluateType(F<A>)
}

@id(T0332)
@name(type.type-function.no-runtime-dependence)
TypeFunctionNoRuntimeDependence<F, A> {
    typeResult(F<A>) doesNotDependOn runtimeExecutionState
}

@id(T0333)
@name(type.projection.type-function.consistency)
ProjectionTypeFunctionConsistency<T, F, A> {
    if T::Item == F<A> {
        normalize(T::Item) == normalize(F<A>)
    }
}

@id(T0334)
@name(type.projection.solver.conservative)
ProjectionSolverConservative<E, R> {
    if unresolved(E) {
        solverMustNotAssume(E == R)
    }
}
```

## Law notes

### T0310 — `ProjectionDeterministic`

The same associated projection in the same semantic environment denotes the same type.

**Kind:** Semantic law  
**Depends on:** T0001–T0003

### T0311 / T0312 — Identity substitution

Semantically identical receivers and arguments produce semantically identical projections.

**Kind:** Semantic congruence laws  
**Depends on:** T0001–T0003

### T0313 / T0314 — Projection kinding

Associated projections must produce values of their declared result kinds.

Ill-kinded arguments must be rejected before ordinary subtype solving.

**Kind:** Kinding laws  
**Depends on:** T0010–T0018

### T0315 / T0316 — Projection reduction

When a projection resolves to `R`, the symbolic projection and `R` are semantically identical.

**Kind:** Semantic reduction laws  
**Depends on:** T0001–T0003

### T0317 — `ProjectionReductionIdentityStable`

A deterministic projection cannot resolve to two semantically different results.

**Kind:** Coherence invariant  
**Depends on:** T0310, T0315

### T0318–T0320 — Unresolved projections

Failure to resolve a projection is absence of proof. It is not permission to manufacture `Any`, `Never`, or any arbitrary concrete result.

**Kind:** Solver-safety invariants  
**Depends on:** T0030–T0045

### T0321 — `ProjectionConstraintPreservation`

Associated-type declarations may constrain their result. Every successful reduction must satisfy that declared contract.

**Kind:** Semantic contract law  
**Depends on:** T0315

### T0322 — `ProjectionNormalizationCongruence`

Normalization may preserve a symbolic projection or reduce it, but both routes must converge semantically when reduction is available.

**Kind:** Cross-family coherence law  
**Depends on:** `type.normalization`

### T0323–T0328 — Type-level functions

Type functions obey ordinary application kinding, semantic identity substitution, determinism, and stable reduction.

**Kind:** Semantic type-computation laws  
**Depends on:** T0010–T0018, T0001–T0003

### T0329 — `TypeFunctionResultConstraint`

A declared type-function result contract remains authoritative after reduction.

**Kind:** Semantic contract law  
**Depends on:** T0327

### T0330 — `UnresolvedTypeFunctionInventsNoResult`

Unresolved type computation does not justify a fabricated result.

**Kind:** Solver-safety invariant  
**Depends on:** T0318

### T0331 / T0332 — Purity and static dependence

Type-level evaluation is semantically pure and cannot depend on mutable runtime execution state.

Compilation caches, incremental state, or evaluation order must not change the resulting type.

**Kind:** Semantic determinism invariants  
**Depends on:** T0324

### T0333 — `ProjectionTypeFunctionConsistency`

If a projection is defined in terms of a type function, both paths must normalize to the same semantic type.

**Kind:** Cross-subsystem coherence law  
**Depends on:** T0315, T0327

### T0334 — `ProjectionSolverConservative`

The solver must remain conservative around unresolved type computation.

**Kind:** Solver-safety invariant  
**Depends on:** T0318–T0320, T0330

## Required implementation invariants

The implementation may represent projections as:

- symbolic projection nodes,
- normalized concrete types,
- cached reduction results,
- solver obligations,
- or public reflective projection handles.

Those representations must remain semantically coherent.

Reduction caches must be invalidated when their semantic dependencies change, but cache mechanics must never define type identity.

## Explicitly deferred laws

- recursive type-function termination policy
- higher-order type functions
- effectful compile-time execution, if ever introduced
- associated-type specialization precedence
- trait evidence selection
- ambiguity diagnostics
- projection cycles and recursive equations
