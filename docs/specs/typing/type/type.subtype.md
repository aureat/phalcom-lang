# Phalcom Type Algebra Registry — `type.subtype`

**Status:** Normative foundational specification  
**Registry family:** `type.subtype`  
**Identifier range used here:** `T0100–T0106`

## Purpose

This file defines the foundational semantic laws of Phalcom's proper-type subtype relation `<:`.

The relation is a partial order over semantic proper types: it is reflexive and transitive, and mutual subtyping collapses to semantic type identity. These laws define the relation itself; they do not yet define which specific types are above or below other types.

Accordingly, laws for `Never`, `Any`, unions, intersections, functions, tuples, records, generic variance, exact types, and user-defined static domains belong to their own registry families and must be consistent with the foundation here.

## Domain and notation

Unless stated otherwise:

- `A`, `B`, and `C` denote well-formed proper types of kind `Type`.
- `<:` means semantic subtype.
- `==` means semantic identity as defined by `type.identity`.
- A subtype expression that violates `type.kind` is ill formed, not `false`.
- Solver inability to prove a relation is not defined here as either truth or falsity; proof-state semantics belong to `type.solver`.

## Laws

```phalcom
@id(T0100)
@name(type.subtype.reflexive)
SubtypeReflexive<A> {
    A <: A
}

@id(T0101)
@name(type.subtype.transitive)
SubtypeTransitive<A, B, C> {
    if A <: B and B <: C {
        A <: C
    }
}

@id(T0102)
@name(type.subtype.identity.implies-mutual)
IdentityImpliesMutualSubtyping<A, B> {
    if A == B {
        A <: B
        B <: A
    }
}

@id(T0103)
@name(type.subtype.antisymmetric)
SubtypeAntisymmetric<A, B> {
    if A <: B and B <: A {
        A == B
    }
}

@id(T0104)
@name(type.subtype.identity.iff-mutual)
IdentityIffMutualSubtyping<A, B> {
    A == B iff A <: B and B <: A
}

@id(T0105)
@name(type.subtype.identity-substitution.left)
SubtypeLeftIdentitySubstitution<A, B, C> {
    if A == B and A <: C {
        B <: C
    }
}

@id(T0106)
@name(type.subtype.identity-substitution.right)
SubtypeRightIdentitySubstitution<A, B, C> {
    if A == B and C <: A {
        C <: B
    }
}
```

## T0100 — `SubtypeReflexive`

Every proper type is a subtype of itself.

This must hold even when the type is represented through aliases, projections, generic applications, exact types, or other forms that require semantic resolution.

**Kind:** Semantic law  
**Depends on:** T0001, T0060

## T0101 — `SubtypeTransitive`

Subtyping composes transitively.

If `A` may be used where `B` is required, and `B` may be used where `C` is required, then `A` may be used where `C` is required.

An implementation may use shortcuts, caches, nominal ancestry, structural decomposition, or normalized forms, but no internal path may violate the transitive closure of the semantic relation.

**Kind:** Semantic law  
**Depends on:** T0100

## T0102 — `IdentityImpliesMutualSubtyping`

Semantically identical types must subtype each other in both directions.

This connects the identity relation defined in `type.identity` to the subtype order without defining identity in terms of representation or syntax.

**Kind:** Cross-family semantic law  
**Depends on:** T0001–T0003, T0100

## T0103 — `SubtypeAntisymmetric`

Mutual subtyping of proper types implies semantic identity.

This law is a major semantic commitment: Phalcom's `==` relation identifies the equivalence classes induced by `<:`. Two proper types that subtype each other are not merely observationally substitutable; they denote the same semantic type.

**Kind:** Semantic law  
**Depends on:** T0001–T0003, T0100–T0102

## T0104 — `IdentityIffMutualSubtyping`

For proper types, semantic identity is equivalent to mutual subtyping.

This law is logically derived from T0102 and T0103 and is included as an explicit registry law because it is a high-value cross-check for solver, normalization, reflection, and canonicalization implementations.

It must not be used to bypass kind checking: the expression is only meaningful for well-formed proper-type operands.

**Kind:** Derived semantic law  
**Depends on:** T0102, T0103

## T0105 — `SubtypeLeftIdentitySubstitution`

Replacing the left operand of a subtype relation with a semantically identical type preserves the subtype result.

This is derived from identity-implied mutual subtyping plus transitivity, but is registered explicitly because implementations frequently route aliases, projections, normalized forms, and canonical types through different solver paths.

**Kind:** Derived coherence law  
**Depends on:** T0101, T0102

## T0106 — `SubtypeRightIdentitySubstitution`

Replacing the right operand of a subtype relation with a semantically identical type preserves the subtype result.

Together, T0105 and T0106 require subtype solving to be invariant under semantic identity of either operand.

**Kind:** Derived coherence law  
**Depends on:** T0101, T0102

## Structural consequence

T0100, T0101, and T0103 establish `<:` as a partial order over semantic proper types:

```text
reflexive      A <: A
transitive     A <: B and B <: C  =>  A <: C
antisymmetric  A <: B and B <: A  =>  A == B
```

The use of semantic identity in antisymmetry is deliberate. Different source expressions or internal representations may occupy the same point in this order.

## Explicitly deferred laws

The following do not belong in this foundational file:

- `Never <: A` — `type.never`
- `A <: Any` — `type.any`
- union least-upper-bound laws — `type.union`
- intersection greatest-lower-bound laws — `type.intersection`
- function parameter/result variance — `type.function`
- tuple and record subtyping — `type.tuple`, `type.record`, `type.row`
- generic variance — `type.generic`, `type.variance`
- exact-value subtype rules — `type.exact`
- subtype normalization consistency — `type.normalization` / `type.solver`
- inability-to-prove versus disproof — `type.solver`
- reflective runtime subtype consistency — `type.reflection`

Each later family must preserve T0100–T0106.
