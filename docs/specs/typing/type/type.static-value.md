# Phalcom Type Algebra Registry — `type.static-value`

**Status:** Normative foundational specification  
**Registry family:** `type.static-value`  
**Identifier range used here:** `T0250–T0259`

## Purpose

This file defines the static-value universe used by exact types, static generic arguments, type-level computation, and `type data`.

A runtime value does not automatically become a type-level value. `staticValue(v)` means the compiler can represent `v` with stable semantic identity during type checking. Static values retain the proper type they inhabit as their kind. This gives Phalcom a coherent bridge between value-level domains and type-level computation without using arbitrary runtime equality as type identity.

## Laws

```phalcom
@id(T0250)
@name(type.static-value.equivalence.reflexive)
StaticEquivalenceReflexive<v> {
    if staticValue(v) {
        staticEquivalent(v, v)
    }
}

@id(T0251)
@name(type.static-value.equivalence.symmetric)
StaticEquivalenceSymmetric<a, b> {
    if staticEquivalent(a, b) {
        staticEquivalent(b, a)
    }
}

@id(T0252)
@name(type.static-value.equivalence.transitive)
StaticEquivalenceTransitive<a, b, c> {
    if staticEquivalent(a, b) and staticEquivalent(b, c) {
        staticEquivalent(a, c)
    }
}

@id(T0253)
@name(type.static-value.kind)
StaticValueKind<v, A> {
    if staticValue(v) and v : A {
        kind(v) == A
    }
}

@id(T0254)
@name(type.static-value.kind-compatible-equivalence)
StaticEquivalenceKindCompatible<a, b> {
    if staticEquivalent(a, b) {
        kind(a) == kind(b)
    }
}

@id(T0255)
@name(type.static-value.types)
TypesAreStaticValues<A> {
    if kind(A) == Type {
        staticValue(A)
    }
}

@id(T0256)
@name(type.static-value.substitution)
StaticValueSubstitution<F, a, b> {
    if staticEquivalent(a, b)
        and wellFormed(F<a>)
        and wellFormed(F<b>)
    {
        F<a> == F<b>
    }
}


@id(T0257)
@name(type.static-value.type-identity)
StaticTypeEquivalenceMatchesTypeIdentity<A, B> {
    if kind(A) == Type and kind(B) == Type {
        staticEquivalent(A, B) iff A == B
    }
}

@id(T0258)
@name(type.static-value.runtime-equality.not-authority)
RuntimeEqualityDoesNotDefineStaticIdentity<a, b> {
    runtimeEqual(a, b) doesNotImply staticEquivalent(a, b)
}

@id(T0259)
@name(type.static-value.no-implicit-promotion)
RuntimeValueNotImplicitlyStatic<v> {
    runtimeValue(v) doesNotImply staticValue(v)
}
```

## Notes

### Static equivalence

`staticEquivalent` is the identity relation used for compile-time values. It is an equivalence relation and is kind-compatible. It is deliberately distinct from an overloadable runtime `==` operation.

### Static kinding

If a static value `v` inhabits proper type `A`, then `kind(v) == A`. Thus an exact integer literal can have kind `Int`, `true` can have kind `Bool`, and a `type data` case can have its user-defined static domain as its kind.

### Types as static values

Proper types are themselves static values of kind `Type`. This is the ordinary type-level universe and does not imply that every runtime value is statically reifiable.

### Type identity bridge

Proper types are static values of kind `Type`, and their static equivalence is exactly semantic type identity. This lets generic substitution use one static-value equivalence relation across type and non-type static arguments without weakening type identity.

## Explicitly deferred laws

- serialization/reification requirements for static values
- which runtime constants are promoted automatically
- floating-point static identity details
- compile-time object identity
- singleton-kind surface syntax
