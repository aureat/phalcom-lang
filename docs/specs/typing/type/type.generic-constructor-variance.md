# Phalcom Type Algebra Registry — `type.generic-constructor-variance`

**Status:** Normative specification  
**Registry families:** `type.generic`, `type.constructor`, `type.variance`  
**Identifier range used here:** `T0800–T0824`

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
@id(T0800)
@name(type.constructor.application.identity)
ConstructorApplicationIdentity<F, A> {
    F<A> == F<A>
}

@id(T0801)
@name(type.constructor.identity-substitution)
ConstructorIdentitySubstitution<F, A, B> {
    if staticEquivalent(A, B) {
        F<A> == F<B>
    }
}

@id(T0802)
@name(type.constructor.nominal-distinct.identity)
DistinctNominalConstructorsStayDistinct<F, G, A> {
    if nominalConstructor(F)
        and nominalConstructor(G)
        and F != G
    {
        F<A> != G<A>
    }
}

@id(T0803)
@name(type.constructor.kind.application)
ConstructorApplicationKind<F, A, K1, K2> {
    if kind(F) == K1 -> K2 and kind(A) == K1 {
        kind(F<A>) == K2
    }
}

@id(T0804)
@name(type.variance.must-be-declared)
GenericSubtypingRequiresDeclaredVariance<F, A, B> {
    if A <: B and variance(F, 0) != Covariant {
        notEntailsFrom(A <: B, F<A> <: F<B>)
    }
}

@id(T0805)
@name(type.variance.covariant)
CovariantConstructor<F, A, B> {
    if variance(F, 0) == Covariant and A <: B {
        F<A> <: F<B>
    }
}

@id(T0806)
@name(type.variance.contravariant)
ContravariantConstructor<F, A, B> {
    if variance(F, 0) == Contravariant and A <: B {
        F<B> <: F<A>
    }
}

@id(T0807)
@name(type.variance.invariant.no-induced-subtype)
InvariantParameterDoesNotInduceSubtype<F, A, B> {
    if variance(F, 0) == Invariant and A != B {
        notEntailsFrom(A <: B, F<A> <: F<B>)
        notEntailsFrom(A <: B, F<B> <: F<A>)
    }
}

@id(T0808)
@name(type.variance.covariant.identity)
CovariantIdentityPreserved<F, A, B> {
    if variance(F, 0) == Covariant and A == B {
        F<A> == F<B>
    }
}

@id(T0809)
@name(type.variance.contravariant.identity)
ContravariantIdentityPreserved<F, A, B> {
    if variance(F, 0) == Contravariant and A == B {
        F<A> == F<B>
    }
}

@id(T0810)
@name(type.variance.invariant.identity)
InvariantIdentityPreserved<F, A, B> {
    if variance(F, 0) == Invariant and A == B {
        F<A> == F<B>
    }
}

@id(T0811)
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

@id(T0812)
@name(type.constructor.injective.declared)
DeclaredInjectiveConstructor<F, A, B> {
    if injective(F) and F<A> == F<B> {
        A == B
    }
}

@id(T0813)
@name(type.constructor.noninjective.not-assumed)
ConstructorInjectivityNotImplicit<F, A, B> {
    if not injective(F) and F<A> == F<B> {
        // does not imply:
        // A == B
    }
}

@id(T0814)
@name(type.constructor.arity.required)
ConstructorArityRequired<F, ***As> {
    if not acceptsArity(F, arity(***As)) {
        illFormed(F<***As>)
    }
}

@id(T0815)
@name(type.constructor.nominal-parameter-order.significant)
NominalConstructorParameterOrderSignificant<F, A, B> {
    if nominalConstructor(F) and not staticEquivalent(A, B) {
        F<A, B> != F<B, A>
    }
}

@id(T0816)
@name(type.generic.never.covariant)
CovariantNeverLowerBound<F, A> {
    if variance(F, 0) == Covariant {
        F<Never> <: F<A>
    }
}

@id(T0817)
@name(type.generic.any.covariant)
CovariantAnyUpperBound<F, A> {
    if variance(F, 0) == Covariant {
        F<A> <: F<Any>
    }
}

@id(T0818)
@name(type.generic.never.contravariant)
ContravariantNeverUpperBound<F, A> {
    if variance(F, 0) == Contravariant {
        F<A> <: F<Never>
    }
}

@id(T0819)
@name(type.generic.any.contravariant)
ContravariantAnyLowerBound<F, A> {
    if variance(F, 0) == Contravariant {
        F<Any> <: F<A>
    }
}

@id(T0820)
@name(type.generic.invariant.nominal-never-any-distinct)
InvariantNominalNeverAnyDistinct<F> {
    if nominalConstructor(F) and variance(F, 0) == Invariant {
        F<Never> != F<Any>
    }
}

@id(T0821)
@name(type.constructor.partial-application.identity)
PartialConstructorIdentity<F, A> {
    F<A> == F<A>
}

@id(T0822)
@name(type.constructor.partial-application.kind)
PartialConstructorKind<F, A, K1, K2> {
    if kind(F) == Type -> K1 -> K2 and kind(A) == Type {
        kind(F<A>) == K1 -> K2
    }
}

@id(T0823)
@name(type.constructor.variance.composition)
VarianceComposition<F, G> {
    variance(F<G<_>>, 0) ==
        composeVariance(variance(F, 0), variance(G, 0))
}

@id(T0824)
@name(type.constructor.variance.identity-consistency)
VarianceNeverBreaksIdentity<F, A, B> {
    if staticEquivalent(A, B) {
        F<A> == F<B>
    }
}
```

## Law notes

### T0800–T0803 — Constructor application

Type constructor application participates in ordinary semantic identity and kinding. Distinct-constructor identity is only asserted for nominal constructors; arbitrary type functions may reduce to the same result.

**Kind:** Semantic laws  
**Depends on:** T0050–T0057, T0001–T0003

### T0804 — `GenericSubtypingRequiresDeclaredVariance`

This is a critical safety invariant.

`A <: B` alone is insufficient to conclude `F<A> <: F<B>`.

**Kind:** Solver-safety invariant  
**Depends on:** T0100–T0103

### T0805 — `CovariantConstructor`

Covariant parameters preserve subtype direction.

**Kind:** Semantic variance law  
**Depends on:** T0804

### T0806 — `ContravariantConstructor`

Contravariant parameters reverse subtype direction.

**Kind:** Semantic variance law  
**Depends on:** T0804

### T0807 — `InvariantParameterDoesNotInduceSubtype`

Invariant parameters do not induce either covariant or contravariant subtyping from argument subtyping. Separate declared subtype relations may still exist for other reasons.

**Kind:** Semantic variance law  
**Depends on:** T0104, T0804

### T0808–T0810 — Identity preservation

Variance never changes the basic rule that semantically identical arguments produce identical applications.

**Kind:** Derived coherence laws  
**Depends on:** T0801

### T0811 — `MultiParameterVariance`

Each generic parameter contributes according to its own declared variance.

**Kind:** Semantic variance law  
**Depends on:** T0805–T0807

### T0812 / T0813 — Constructor injectivity

Injectivity is explicit metadata/semantics, not a universal property of arbitrary type constructors or reducible type functions.

**Kind:** Semantic constructor laws  
**Depends on:** T0001–T0003

### T0814 / T0815 — Constructor shape identity

Constructor arity is a well-formedness requirement. For nominal constructors, ordered static arguments participate in nominal identity, so swapping distinct arguments changes the specialization.

**Kind:** Semantic identity laws  
**Depends on:** T0001–T0003

### T0816–T0819 — Boundary consequences

`Never` and `Any` induce predictable extrema through covariant and contravariant parameters.

**Kind:** Derived semantic laws  
**Depends on:** T0151, T0201, T0805, T0806

### T0820 — `InvariantNominalNeverAnyDistinct`

For nominal constructors, invariant applications over semantically distinct arguments remain distinct by nominal argument identity. Invariance alone would not justify this law for arbitrary reducible type functions.

**Kind:** Derived semantic law  
**Depends on:** `type.nominal`, `type.static-value`, T0156, T0203

### T0821 / T0822 — Partial application

Partially applied constructors retain semantic identity and the remaining constructor kind.

**Kind:** Semantic constructor laws  
**Depends on:** T0056

### T0823 — `VarianceComposition`

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
**Depends on:** T0805–T0807

### T0824 — `VarianceNeverBreaksIdentity`

Variance affects subtype direction, never semantic identity substitution.

**Kind:** Cross-cutting coherence law  
**Depends on:** T0801

## Explicitly deferred laws

- declaration syntax for variance
- inferred variance
- higher-kinded variance
- associated type variance
- existential/wildcard variance
- runtime reification of generic arguments
