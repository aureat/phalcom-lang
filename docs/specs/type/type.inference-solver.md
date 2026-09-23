# Phalcom Type Algebra Registry — `type.inference-solver`

**Status:** Normative implementation-facing specification  
**Registry families:** `type.inference`, `type.solver`  
**Identifier range used here:** `T0370–T0402`

## Purpose

This file defines the correctness invariants of type inference and semantic relation solving.

The solver must distinguish truth, falsity, and lack of proof where the relation is not closed or currently decidable. Inference variables represent unknown information to be refined by constraints; they are not semantic aliases for `Any`, `Never`, or arbitrary concrete types.

Soundness is mandatory. Completeness may be fragment-specific and must be declared explicitly.

## Domain and notation

Conceptually:

```text
Proven
Disproven
Unknown
```

are solver proof states.

- `solve(P)` returns a proof state for proposition `P`.
- `α`, `β` denote inference variables.
- `constrain(α, C)` adds a constraint.
- `solution(α)` denotes a solved type where one exists.
- `consistent(S)` means a constraint set has at least one solution.

## Laws

```phalcom
@id(T0370)
@name(type.solver.truth.sound)
SolverProvenIsSound<P> {
    if solve(P) == Proven {
        P
    }
}

@id(T0371)
@name(type.solver.false.sound)
SolverDisprovenIsSound<P> {
    if solve(P) == Disproven {
        not P
    }
}

@id(T0372)
@name(type.solver.unknown.not-false)
SolverUnknownIsNotDisproof<P> {
    if solve(P) == Unknown {
        solve(P) != Disproven
    }
}

@id(T0373)
@name(type.solver.unknown.not-true)
SolverUnknownIsNotProof<P> {
    if solve(P) == Unknown {
        solve(P) != Proven
    }
}

@id(T0374)
@name(type.solver.no-proof-not-disproof)
NoProofIsNotDisproof<P> {
    if not canProve(P) and not closedDecidable(P) {
        not infer(not P)
    }
}

@id(T0375)
@name(type.solver.no-disproof-not-proof)
NoDisproofIsNotProof<P> {
    if not canDisprove(P) and not closedDecidable(P) {
        not infer(P)
    }
}

@id(T0376)
@name(type.inference.variable.not-any)
InferenceVariableIsNotAny<α> {
    unresolved(α) doesNotImply α == Any
}

@id(T0377)
@name(type.inference.variable.not-never)
InferenceVariableIsNotNever<α> {
    unresolved(α) doesNotImply α == Never
}

@id(T0378)
@name(type.inference.refinement.kind-preserving)
InferenceRefinementPreservesKind<α, T> {
    if constrain(α, T)
        and solution(α) == S
    {
        kind(S) == kind(α)
    }
}

@id(T0379)
@name(type.inference.refinement.constraint-preserving)
InferenceSolutionSatisfiesConstraints<α> {
    if solution(α) == S {
        satisfiesAll(S, constraints(α))
    }
}

@id(T0380)
@name(type.inference.refinement.monotone-knowledge)
InferenceKnowledgeMonotone<α, C1, C2> {
    if constraints(C1) subsetOf constraints(C2) {
        knowledge(C2) >= knowledge(C1)
    }
}

@id(T0381)
@name(type.inference.refinement.no-fabrication)
InferenceDoesNotFabricate<α, T> {
    if noConstraintSupports(α, T) {
        not proven(solution(α) == T)
    }
}

@id(T0382)
@name(type.inference.occurs-check)
InferenceOccursCheck<α, T> {
    if contains(T, α) and not explicitlyRecursive(T) {
        reject(solution(α) == T)
    }
}

@id(T0383)
@name(type.inference.identity-substitution)
InferenceIdentitySubstitution<A, B, C> {
    if A == B {
        inferWith(A, C) == inferWith(B, C)
    }
}

@id(T0384)
@name(type.solver.identity.congruence)
SolverIdentityCongruence<A1, A2, B1, B2> {
    if A1 == A2 and B1 == B2 {
        solve(A1 <: B1) ==
        solve(A2 <: B2)
    }
}

@id(T0385)
@name(type.solver.normalization.consistency)
SolverNormalizationConsistency<A, B> {
    solve(A <: B) ==
    solve(normalize(A) <: normalize(B))
}

@id(T0386)
@name(type.solver.reflexive)
SolverProvesReflexiveSubtype<A> {
    solve(A <: A) == Proven
}

@id(T0387)
@name(type.solver.identity-implies-mutual)
SolverIdentityImpliesMutualSubtype<A, B> {
    if A == B {
        solve(A <: B) == Proven
        solve(B <: A) == Proven
    }
}

@id(T0388)
@name(type.solver.transitive)
SolverSubtypeTransitive<A, B, C> {
    if solve(A <: B) == Proven
        and solve(B <: C) == Proven
    {
        solve(A <: C) == Proven
    }
}

@id(T0389)
@name(type.solver.antisymmetric)
SolverMutualSubtypeIdentity<A, B> {
    if solve(A <: B) == Proven
        and solve(B <: A) == Proven
    {
        A == B
    }
}

@id(T0390)
@name(type.solver.never.bottom)
SolverProvesNeverBottom<A> {
    solve(Never <: A) == Proven
}

@id(T0391)
@name(type.solver.any.top)
SolverProvesAnyTop<A> {
    solve(A <: Any) == Proven
}

@id(T0392)
@name(type.solver.union.lub)
SolverUnionLeastUpperBound<A, B, U> {
    if solve(A <: U) == Proven
        and solve(B <: U) == Proven
    {
        solve(A | B <: U) == Proven
    }
}

@id(T0393)
@name(type.solver.intersection.glb)
SolverIntersectionGreatestLowerBound<A, B, L> {
    if solve(L <: A) == Proven
        and solve(L <: B) == Proven
    {
        solve(L <: A & B) == Proven
    }
}

@id(T0394)
@name(type.solver.projection.conservative)
SolverProjectionConservative<E> {
    if unresolved(E) {
        solverDoesNotInventReduction(E)
    }
}

@id(T0395)
@name(type.solver.cache-independent)
SolverCacheIndependent<P> {
    solveWithColdCache(P) ==
    solveWithWarmCache(P)
}

@id(T0396)
@name(type.solver.order-independent)
SolverOrderIndependent<A, B> {
    solve(A | B == B | A) == Proven
    solve(A & B == B & A) == Proven
}

@id(T0397)
@name(type.solver.complete.closed-fragment)
SolverCompleteForDeclaredClosedFragment<P> {
    if closedDecidable(P) {
        solve(P) != Unknown
    }
}

@id(T0398)
@name(type.solver.termination.declared-fragment)
SolverTerminatesForDeclaredFragment<P> {
    if supportedSolverFragment(P) {
        terminates(solve(P))
    }
}

@id(T0399)
@name(type.solver.result.deterministic)
SolverDeterministic<P> {
    solve(P) == solve(P)
}

@id(T0400)
@name(type.inference.constraint-order-independent)
InferenceConstraintOrderIndependent<α, ***Cs> {
    solveConstraints(α, ***Cs) ==
    solveConstraints(α, permutation(***Cs))
}

@id(T0401)
@name(type.inference.equivalent-constraints.same-solution)
EquivalentConstraintSetsSameSolution<α, C1, C2> {
    if equivalentConstraints(C1, C2) {
        solveConstraints(α, C1) ==
        solveConstraints(α, C2)
    }
}

@id(T0402)
@name(type.solver.semantic-authority)
SolverMustFollowRegistry<P> {
    if registryEntails(P) {
        solve(P) != Disproven
    }
}
```

## Law notes

### T0370 / T0371 — Soundness

A positive or negative solver conclusion must correspond to semantic truth.

Soundness is non-negotiable.

**Kind:** Fundamental solver invariants  
**Depends on:** the semantic registry

### T0372–T0375 — Unknown proof state

`Unknown` is a first-class proof state where a proposition cannot currently be established or refuted.

Absence of proof is not proof of falsity, and absence of disproof is not proof of truth, except in fragments explicitly declared closed and decidable.

**Kind:** Fundamental solver invariants  
**Depends on:** none

### T0376 / T0377 — Inference variables are epistemic

An unresolved inference variable is not `Any`, not `Never`, and not any other semantic type merely because information is missing.

**Kind:** Inference-safety invariants  
**Depends on:** T0030–T0045

### T0378 / T0379 — Valid solutions

Inference solutions preserve the variable's kind and satisfy every accumulated constraint.

**Kind:** Inference correctness laws  
**Depends on:** T0010–T0018

### T0380 — `InferenceKnowledgeMonotone`

Adding constraints may narrow possible solutions or reveal inconsistency; it must not arbitrarily increase the solution set.

**Kind:** Inference monotonicity invariant

### T0381 — `InferenceDoesNotFabricate`

Inference must be justified by constraints, defaults explicitly authorized by the language, or semantic laws.

**Kind:** Solver-safety invariant

### T0382 — `InferenceOccursCheck`

Non-recursive inference must reject infinite self-solutions such as `α = List<α>` unless recursive types explicitly authorize them.

**Kind:** Inference well-formedness invariant

### T0383 / T0384 — Semantic congruence

Inference and subtype solving must be invariant under semantic identity substitution.

**Kind:** Cross-subsystem coherence laws  
**Depends on:** T0001–T0003

### T0385 — `SolverNormalizationConsistency`

Normalized and unnormalized operands must yield the same proof state.

**Kind:** Cross-subsystem coherence law  
**Depends on:** T0360

### T0386–T0393 — Registry law enforcement

The solver must directly respect foundational subtype, top/bottom, union, and intersection laws.

**Kind:** Solver conformance laws  
**Depends on:** T0020–T0155

### T0394 — `SolverProjectionConservative`

Unresolved projections must remain symbolic rather than guessed.

**Kind:** Solver-safety invariant  
**Depends on:** T0318–T0320, T0334

### T0395 / T0399 — Cache independence and determinism

Caching, traversal order, or incremental history cannot alter semantic proof results.

**Kind:** Implementation coherence invariants

### T0396 — `SolverOrderIndependent`

The solver must prove commutativity of union and intersection regardless of source order.

**Kind:** Algebra conformance law  
**Depends on:** T0101, T0121

### T0397 — `SolverCompleteForDeclaredClosedFragment`

If a fragment is explicitly declared closed and decidable, the solver must not return `Unknown` for propositions within that fragment.

Completeness is therefore scoped, not globally assumed.

**Kind:** Fragment-completeness invariant

### T0398 — `SolverTerminatesForDeclaredFragment`

Supported solver fragments must terminate.

**Kind:** Operational invariant

### T0400 / T0401 — Constraint-order independence

Constraint solving must depend on semantic constraint content, not collection or processing order.

**Kind:** Inference coherence laws

### T0402 — `SolverMustFollowRegistry`

The registry is the semantic authority. A solver may be incomplete, but it must never contradict an entailed registry law.

**Kind:** Global conformance invariant

## Explicitly deferred laws

- overload ranking
- diagnostics priority
- defaulting rules
- numeric literal default inference
- higher-rank inference
- existential inference
- recursive constraint solving
- performance budgets
- incremental solver invalidation details
