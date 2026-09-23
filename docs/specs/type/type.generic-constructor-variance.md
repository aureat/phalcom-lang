# Phalcom Type Algebra Registry — `type.generic-constructor-variance`

**Status:** Normative specification  
**Registry families:** `type.generic`, `type.constructor`, `type.variance`  
**Identifier range used here:** `T0250–T0274`

## Purpose

This file defines generic type-constructor application, constructor identity, argument substitution, and variance.

Generic subtyping must be derived from constructor-declared variance. A subtype relation between arguments does not by itself imply a subtype relation between generic applications.

## Domain and notation

- `F`, `G` denote type constructors.
- `F<A>` denotes type application.
- `+T` conceptually denotes covariance.
- `-T` conceptually denotes contravariance.
- invariant parameters have neither marker conceptually.
- `variance(F, i)` denotes the declared variance of constructor parameter `i`.

## Laws

```phalcom
@id(T0250)
@name(type.constructor.application.identity)
ConstructorApplicationIdentity<F, A> {
    F<A> == F<A>
}

@id(T0251)
@name(type.constructor.identity-substitution)
ConstructorIdentitySubstitution<F, A, B> {
    if A == B {
        F<A> == F<B>
    }
}

@id(T0252)
@name(type.constructor.distinct.identity)
DistinctConstructorsStayDistinct<F, G, A> {
    if F != G {
        F<A> != G<A>
    }
}

@id(T0253)
@name(type.constructor.kind.application)
ConstructorApplicationKind<F, A, K> {
    if kind(F) == Type -> K and kind(A) == Type {
        kind(F<A>) == K
    }
}

@id(T0254)
@name(type.variance.must-be-declared)
GenericSubtypingRequiresDeclaredVariance<F, A, B> {
    if A <: B {
        // does not imply:
        // F<A> <: F<B>
    }
}

@id(T0255)
@name(type.variance.covariant)
CovariantConstructor<F, A, B> {
    if variance(F, 0) == Covariant and A <: B {
        F<A> <: F<B>
    }
}

@id(T0256)
@name(type.variance.contravariant)
ContravariantConstructor<F, A, B> {
    if variance(F, 0) == Contravariant and A <: B {
        F<B> <: F<A>
    }
}

@id(T0257)
@name(type.variance.invariant)
InvariantConstructor<F, A, B> {
    if variance(F, 0) == Invariant {
        F<A> <: F<B> iff A == B
    }
}

@id(T0258)
@name(type.variance.covariant.identity)
CovariantIdentityPreserved<F, A, B> {
    if variance(F, 0) == Covariant and A == B {
        F<A> == F<B>
    }
}

@id(T0259)
@name(type.variance.contravariant.identity)
ContravariantIdentityPreserved<F, A, B> {
    if variance(F, 0) == Contravariant and A == B {
        F<A> == F<B>
    }
}

@id(T0260)
@name(type.variance.invariant.identity)
InvariantIdentityPreserved<F, A, B> {
    if variance(F, 0) == Invariant and A == B {
        F<A> == F<B>
    }
}

@id(T0261)
@name(type.constructor.multi-parameter.variance)
MultiParameterVariance<F, A1, A2, B1, B2> {
    if variance(F, 0) == Covariant
        and variance(F, 1) == Contravariant
        and A1 <: B1
        and A2 <: B2
    {
        F<A1, B2> <: F<B1, A2>
    }
}

@id(T0262)
@name(type.constructor.injective.declared)
DeclaredInjectiveConstructor<F, A, B> {
    if injective(F) and F<A> == F<B> {
        A == B
    }
}

@id(T0263)
@name(type.constructor.noninjective.not-assumed)
ConstructorInjectivityNotImplicit<F, A, B> {
    if not injective(F) and F<A> == F<B> {
        // does not imply:
        // A == B
    }
}

@id(T0264)
@name(type.constructor.arity.significant)
ConstructorAritySignificant<F, A, B> {
    F<A> != F<A, B>
}

@id(T0265)
@name(type.constructor.parameter-order.significant)
ConstructorParameterOrderSignificant<F, A, B> {
    if A != B {
        F<A, B> != F<B, A>
    }
}

@id(T0266)
@name(type.generic.never.covariant)
CovariantNeverLowerBound<F, A> {
    if variance(F, 0) == Covariant {
        F<Never> <: F<A>
    }
}

@id(T0267)
@name(type.generic.any.covariant)
CovariantAnyUpperBound<F, A> {
    if variance(F, 0) == Covariant {
        F<A> <: F<Any>
    }
}

@id(T0268)
@name(type.generic.never.contravariant)
ContravariantNeverUpperBound<F, A> {
    if variance(F, 0) == Contravariant {
        F<A> <: F<Never>
    }
}

@id(T0269)
@name(type.generic.any.contravariant)
ContravariantAnyLowerBound<F, A> {
    if variance(F, 0) == Contravariant {
        F<Any> <: F<A>
    }
}

@id(T0270)
@name(type.generic.invariant.never-any-distinct)
InvariantNeverAnyDistinct<F> {
    if variance(F, 0) == Invariant {
        F<Never> != F<Any>
    }
}

@id(T0271)
@name(type.constructor.partial-application.identity)
PartialConstructorIdentity<F, A> {
    F<A> == F<A>
}

@id(T0272)
@name(type.constructor.partial-application.kind)
PartialConstructorKind<F, A, K1, K2> {
    if kind(F) == Type -> K1 -> K2 and kind(A) == Type {
        kind(F<A>) == K1 -> K2
    }
}

@id(T0273)
@name(type.constructor.variance.composition)
VarianceComposition<F, G> {
    variance(F<G<_>>, 0) ==
        composeVariance(variance(F, 0), variance(G, 0))
}

@id(T0274)
@name(type.constructor.variance.identity-consistency)
VarianceNeverBreaksIdentity<F, A, B> {
    if A == B {
        F<A> == F<B>
    }
}
```

## Law notes

### T0250–T0253 — Constructor application

Type constructor application participates in ordinary semantic identity and kinding.

**Kind:** Semantic laws  
**Depends on:** T0010–T0015, T0001–T0003

### T0254 — `GenericSubtypingRequiresDeclaredVariance`

This is a critical safety invariant.

`A <: B` alone is insufficient to conclude `F<A> <: F<B>`.

**Kind:** Solver-safety invariant  
**Depends on:** T0020–T0023

### T0255 — `CovariantConstructor`

Covariant parameters preserve subtype direction.

**Kind:** Semantic variance law  
**Depends on:** T0254

### T0256 — `ContravariantConstructor`

Contravariant parameters reverse subtype direction.

**Kind:** Semantic variance law  
**Depends on:** T0254

### T0257 — `InvariantConstructor`

Invariant parameters permit generic subtyping only when the arguments are semantically identical.

**Kind:** Semantic variance law  
**Depends on:** T0024, T0254

### T0258–T0260 — Identity preservation

Variance never changes the basic rule that semantically identical arguments produce identical applications.

**Kind:** Derived coherence laws  
**Depends on:** T0251

### T0261 — `MultiParameterVariance`

Each generic parameter contributes according to its own declared variance.

**Kind:** Semantic variance law  
**Depends on:** T0255–T0257

### T0262 / T0263 — Constructor injectivity

Injectivity is explicit metadata/semantics, not a universal property of generic constructors.

**Kind:** Semantic constructor laws  
**Depends on:** T0001–T0003

### T0264 / T0265 — Constructor shape identity

Application arity and parameter order are semantically significant.

**Kind:** Semantic identity laws  
**Depends on:** T0001–T0003

### T0266–T0269 — Boundary consequences

`Never` and `Any` induce predictable extrema through covariant and contravariant parameters.

**Kind:** Derived semantic laws  
**Depends on:** T0031, T0041, T0255, T0256

### T0270 — `InvariantNeverAnyDistinct`

Invariant applications over semantically distinct arguments remain distinct.

**Kind:** Derived semantic law  
**Depends on:** T0257, T0036, T0043

### T0271 / T0272 — Partial application

Partially applied constructors retain semantic identity and the remaining constructor kind.

**Kind:** Semantic constructor laws  
**Depends on:** T0014

### T0273 — `VarianceComposition`

Nested constructor variance composes.

Conceptually:

```text
covariant × covariant     = covariant
covariant × contravariant = contravariant
contravariant × covariant = contravariant
contravariant × contravariant = covariant
anything × invariant      = invariant
invariant × anything      = invariant
```

**Kind:** Derived variance law  
**Depends on:** T0255–T0257

### T0274 — `VarianceNeverBreaksIdentity`

Variance affects subtype direction, never semantic identity substitution.

**Kind:** Cross-cutting coherence law  
**Depends on:** T0251

## Explicitly deferred laws

- declaration syntax for variance
- inferred variance
- higher-kinded variance
- associated type variance
- existential/wildcard variance
- runtime reification of generic arguments
