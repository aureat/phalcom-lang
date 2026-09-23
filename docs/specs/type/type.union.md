# Phalcom Type Algebra Registry — `type.union`

**Status:** Normative specification  
**Registry family:** `type.union`  
**Identifier range used here:** `T0100–T0115`

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
@id(T0100)
@name(type.union.kind)
UnionKind<A, B> {
    kind(A | B) == Type
}

@id(T0101)
@name(type.union.commutative)
UnionCommutative<A, B> {
    A | B == B | A
}

@id(T0102)
@name(type.union.associative)
UnionAssociative<A, B, C> {
    (A | B) | C == A | (B | C)
}

@id(T0103)
@name(type.union.idempotent)
UnionIdempotent<A> {
    A | A == A
}

@id(T0104)
@name(type.union.never.identity)
UnionNeverIdentity<A> {
    A | Never == A
}

@id(T0105)
@name(type.union.any.absorbing)
UnionAnyAbsorbing<A> {
    A | Any == Any
}

@id(T0106)
@name(type.union.left-subtype)
UnionLeftSubtype<A, B> {
    A <: A | B
}

@id(T0107)
@name(type.union.right-subtype)
UnionRightSubtype<A, B> {
    B <: A | B
}

@id(T0108)
@name(type.union.least-upper-bound)
UnionLeastUpperBound<A, B, U> {
    if A <: U and B <: U {
        A | B <: U
    }
}

@id(T0109)
@name(type.union.subtype-absorption)
UnionSubtypeAbsorption<A, B> {
    if A <: B {
        A | B == B
    }
}

@id(T0110)
@name(type.union.identity-substitution)
UnionIdentitySubstitution<A, B, C> {
    if A == B {
        A | C == B | C
    }
}

@id(T0111)
@name(type.union.monotone.left)
UnionLeftMonotone<A, B, C> {
    if A <: B {
        A | C <: B | C
    }
}

@id(T0112)
@name(type.union.monotone.right)
UnionRightMonotone<A, B, C> {
    if A <: B {
        C | A <: C | B
    }
}

@id(T0113)
@name(type.union.monotone)
UnionMonotone<A, B, C, D> {
    if A <: B and C <: D {
        A | C <: B | D
    }
}

@id(T0114)
@name(type.union.membership)
UnionMembership<v, A, B> {
    v : A | B iff v : A or v : B
}

@id(T0115)
@name(type.union.boolean-decomposition)
BoolUnionDecomposition {
    Bool == exact(true) | exact(false)
}
```

## Law notes

### T0100 — `UnionKind`

The union of two proper types is itself a proper type.

**Kind:** Semantic law  
**Depends on:** T0018

### T0101 — `UnionCommutative`

Union is insensitive to operand order.

**Kind:** Semantic algebra law  
**Depends on:** T0001–T0003

### T0102 — `UnionAssociative`

Grouping does not affect semantic union identity. This permits flat semantic treatment of nested unions without requiring source representation to be flat.

**Kind:** Semantic algebra law  
**Depends on:** T0101

### T0103 — `UnionIdempotent`

Repeating the same alternative does not create a new type.

**Kind:** Semantic algebra law  
**Depends on:** T0022–T0024

### T0104 — `UnionNeverIdentity`

`Never` is the identity element of union.

**Kind:** Semantic algebra law  
**Depends on:** T0031–T0036

### T0105 — `UnionAnyAbsorbing`

`Any` is the absorbing element of union.

**Kind:** Semantic algebra law  
**Depends on:** T0041–T0043

### T0106 / T0107 — Operand subtyping

Each operand is a subtype of the union.

**Kind:** Semantic order laws  
**Depends on:** T0020–T0023

### T0108 — `UnionLeastUpperBound`

`A | B` is the least upper bound of `A` and `B`.

Together with T0106 and T0107, this is the defining order-theoretic characterization of union.

**Kind:** Semantic order law  
**Depends on:** T0106, T0107, T0021

### T0109 — `UnionSubtypeAbsorption`

If one operand is already a subtype of the other, the narrower operand contributes no additional type space.

**Kind:** Derived semantic law  
**Depends on:** T0106–T0108, T0023

### T0110 — `UnionIdentitySubstitution`

Replacing a union operand with a semantically identical type preserves the union.

**Kind:** Derived coherence law  
**Depends on:** T0001–T0003, T0101–T0103

### T0111–T0113 — Monotonicity

Union is monotone in each operand and jointly monotone.

**Kind:** Derived order laws  
**Depends on:** T0106–T0108, T0021

### T0114 — `UnionMembership`

A value inhabits `A | B` exactly when it inhabits at least one operand.

**Kind:** Semantic membership law  
**Depends on:** T0106–T0108

### T0115 — `BoolUnionDecomposition`

The closed Boolean domain is exactly the union of the two exact Boolean cases.

**Kind:** Cross-family semantic law  
**Depends on:** T0060–T0069, T0080–T0090, T0106–T0114

## Required implementation invariants

An implementation must preserve these laws regardless of source operand order, nested versus flat AST representation, normalization timing, alias expansion, exact-type reduction, projection resolution, canonical interning, or solver cache state.

A union representation may retain source order for diagnostics, but semantic identity must obey T0101–T0103.

## Explicitly deferred laws

- intersection/union absorption — `type.algebra`
- distributivity — `type.algebra`
- canonical flattening, sorting, duplicate removal — `type.normalization`
- pattern-space subtraction — `type.pattern`
- variance-specific union behavior — the corresponding structural type family
