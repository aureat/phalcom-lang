# Phalcom Type Algebra Registry — `type.identity`

**Status:** Normative foundational specification  
**Registry family:** `type.identity`  
**Identifier range used here:** `T0001–T0003`

## Purpose

This file defines the foundational semantic identity relation used by the Phalcom type system.

`==` in these laws means **semantic type identity/equivalence**, not source-text equality, AST equality, allocation identity, canonical-node pointer identity, or reflective object identity. Two type expressions may therefore have different syntax or representation while denoting the same semantic type.

The laws in this file are intentionally minimal. Identity congruence through type constructors, preservation of kinds, mutual-subtyping equivalence, normalization consistency, and reflective identity are specified by the semantic families that introduce those concepts.

## Domain and notation

Unless a law says otherwise:

- `A`, `B`, and `C` denote well-formed type-level entities for which `==` is defined.
- Both operands of `==` must be kind-compatible as specified by `type.kind`.
- A law body states a semantic truth that every implementation path must preserve.
- Lower `TXXXX` identifiers denote more foundational laws.
- These identifiers are global across the entire Type Algebra Registry.

## Laws

```phalcom
@id(T0001)
@name(type.identity.reflexive)
TypeIdentityReflexive<A> {
    A == A
}

@id(T0002)
@name(type.identity.symmetric)
TypeIdentitySymmetric<A, B> {
    if A == B {
        B == A
    }
}

@id(T0003)
@name(type.identity.transitive)
TypeIdentityTransitive<A, B, C> {
    if A == B and B == C {
        A == C
    }
}
```

## T0001 — `TypeIdentityReflexive`

Every well-formed type-level entity is semantically identical to itself.

This law is more foundational than any canonicalization or interning strategy. Failure of pointer identity, delayed projection resolution, syntactic reconstruction, or independently-created equivalent nodes must not make a type unequal to itself.

**Kind:** Semantic law  
**Depends on:** none

## T0002 — `TypeIdentitySymmetric`

Semantic identity has no direction. If `A` and `B` denote the same type, reversing the operands must preserve the result.

An implementation must not allow asymmetric behavior to leak from solver caches, projection direction, alias expansion direction, canonicalization order, or inference state.

**Kind:** Semantic law  
**Depends on:** T0001

## T0003 — `TypeIdentityTransitive`

Semantic identity must compose. If `A` is identical to `B`, and `B` is identical to `C`, then `A` and `C` are identical.

This is especially important when equivalence is established through different mechanisms—for example alias expansion, normalization, associated-type resolution, exact-type reduction, or canonical interning. Those mechanisms must not form mutually inconsistent equivalence classes.

**Kind:** Semantic law  
**Depends on:** T0001, T0002

## Required implementation invariants

An implementation of semantic type identity must form equivalence classes satisfying T0001–T0003 regardless of which subsystem discovers the relation.

The following distinctions are deliberate:

```text
semantic identity    A == B
syntactic identity   syntax(A) == syntax(B)
representation       canonical(A) is canonical(B)
reflection identity  reflect(A) == reflect(B)
```

Only the first relation is defined here. Later registry families may require the other relations to agree with semantic identity under explicitly stated conditions, but they are not definitions of semantic identity.

## Explicitly deferred laws

The following laws do **not** belong in this file:

- identity preserving kind — `type.kind`
- equality implying mutual subtyping — `type.subtype`
- mutual subtyping implying identity — `type.subtype`
- constructor congruence such as `A == B => F<A> == F<B>` — `type.generic` / `type.constructor`
- normalization preserving identity — `type.normalization`
- canonical representation sharing — `type.normalization`
- runtime reflective identity consistency — `type.reflection`

Keeping these laws outside `type.identity` prevents the foundational equivalence relation from depending on machinery introduced later in the registry.
