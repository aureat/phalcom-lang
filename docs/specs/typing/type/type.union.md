# Phalcom Type Algebra Registry — `type.union`

**Status:** Normative specification  
**Registry family:** `type.union`  
**Identifier range used here:** `T0450–T0465`

## Purpose

This file defines the semantic laws of the type-union operator `|`.

For proper types `A` and `B`, `A | B` denotes their least upper bound in the Phalcom subtype order: every inhabitant of either operand inhabits the union, and the union is a subtype of every proper type that is a supertype of both operands.

These laws are semantic. They do not prescribe a particular AST shape, storage order, normalization algorithm, or interning strategy.

## Domain and notation

Unless stated otherwise:

- `A`, `B`, `C`, and `U` denote well-formed proper types of kind `Type`.
- `==` means semantic type identity.
- `<:` means semantic subtype.
- `|` means type union.
- `v : A` means value `v` inhabits type `A`.

## Laws

```phalcom
@id(T0450)
@name(type.union.kind)
UnionKind<A, B> {
    kind(A | B) == Type
}

@id(T0451)
@name(type.union.commutative)
UnionCommutative<A, B> {
    A | B == B | A
}

@id(T0452)
@name(type.union.associative)
UnionAssociative<A, B, C> {
    (A | B) | C == A | (B | C)
}

@id(T0453)
@name(type.union.idempotent)
UnionIdempotent<A> {
    A | A == A
}

@id(T0454)
@name(type.union.never.identity)
UnionNeverIdentity<A> {
    A | Never == A
}

@id(T0455)
@name(type.union.any.absorbing)
UnionAnyAbsorbing<A> {
    A | Any == Any
}

@id(T0456)
@name(type.union.left-subtype)
UnionLeftSubtype<A, B> {
    A <: A | B
}

@id(T0457)
@name(type.union.right-subtype)
UnionRightSubtype<A, B> {
    B <: A | B
}

@id(T0458)
@name(type.union.least-upper-bound)
UnionLeastUpperBound<A, B, U> {
    if A <: U and B <: U {
        A | B <: U
    }
}

@id(T0459)
@name(type.union.subtype-absorption)
UnionSubtypeAbsorption<A, B> {
    if A <: B {
        A | B == B
    }
}

@id(T0460)
@name(type.union.identity-substitution)
UnionIdentitySubstitution<A, B, C> {
    if A == B {
        A | C == B | C
    }
}

@id(T0461)
@name(type.union.monotone.left)
UnionLeftMonotone<A, B, C> {
    if A <: B {
        A | C <: B | C
    }
}

@id(T0462)
@name(type.union.monotone.right)
UnionRightMonotone<A, B, C> {
    if A <: B {
        C | A <: C | B
    }
}

@id(T0463)
@name(type.union.monotone)
UnionMonotone<A, B, C, D> {
    if A <: B and C <: D {
        A | C <: B | D
    }
}

@id(T0464)
@name(type.union.membership)
UnionMembership<v, A, B> {
    v : A | B iff v : A or v : B
}

@id(T0465)
@name(type.union.boolean-decomposition)
BoolUnionDecomposition {
    Bool == exact(true) | exact(false)
}
```

## Law notes

### T0450 — `UnionKind`

The union of two proper types is itself a proper type.

**Kind:** Semantic law  
**Depends on:** T0060

### T0451 — `UnionCommutative`

Union is insensitive to operand order.

**Kind:** Semantic algebra law  
**Depends on:** T0001–T0003

### T0452 — `UnionAssociative`

Grouping does not affect semantic union identity. This permits flat semantic treatment of nested unions without requiring source representation to be flat.

**Kind:** Semantic algebra law  
**Depends on:** T0451

### T0453 — `UnionIdempotent`

Repeating the same alternative does not create a new type.

**Kind:** Semantic algebra law  
**Depends on:** T0102–T0104

### T0454 — `UnionNeverIdentity`

`Never` is the identity element of union.

**Kind:** Semantic algebra law  
**Depends on:** T0151–T0156

### T0455 — `UnionAnyAbsorbing`

`Any` is the absorbing element of union.

**Kind:** Semantic algebra law  
**Depends on:** T0201–T0203

### T0456 / T0457 — Operand subtyping

Each operand is a subtype of the union.

**Kind:** Semantic order laws  
**Depends on:** T0100–T0103

### T0458 — `UnionLeastUpperBound`

`A | B` is the least upper bound of `A` and `B`.

Together with T0456 and T0457, this is the defining order-theoretic characterization of union.

**Kind:** Semantic order law  
**Depends on:** T0456, T0457, T0101

### T0459 — `UnionSubtypeAbsorption`

If one operand is already a subtype of the other, the narrower operand contributes no additional type space.

**Kind:** Derived semantic law  
**Depends on:** T0456–T0458, T0103

### T0460 — `UnionIdentitySubstitution`

Replacing a union operand with a semantically identical type preserves the union.

**Kind:** Derived coherence law  
**Depends on:** T0001–T0003, T0451–T0453

### T0461–T0463 — Monotonicity

Union is monotone in each operand and jointly monotone.

**Kind:** Derived order laws  
**Depends on:** T0456–T0458, T0101

### T0464 — `UnionMembership`

A value inhabits `A | B` exactly when it inhabits at least one operand.

**Kind:** Semantic membership law  
**Depends on:** T0456–T0458

### T0465 — `BoolUnionDecomposition`

The closed Boolean domain is exactly the union of the two exact Boolean cases.

**Kind:** Cross-family semantic law  
**Depends on:** T0351–T0360, T0400–T0411, T0456–T0464

## Required implementation invariants

An implementation must preserve these laws regardless of source operand order, nested versus flat AST representation, normalization timing, alias expansion, exact-type reduction, projection resolution, canonical interning, or solver cache state.

A union representation may retain source order for diagnostics, but semantic identity must obey T0451–T0453.

## Explicitly deferred laws

- intersection/union absorption — `type.algebra`
- distributivity — `type.algebra`
- canonical flattening, sorting, duplicate removal — `type.normalization`
- pattern-space subtraction — `type.pattern`
- variance-specific union behavior — the corresponding structural type family
