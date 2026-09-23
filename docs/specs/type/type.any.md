# Phalcom Type Algebra Registry — `type.any`

**Status:** Normative foundational specification  
**Registry family:** `type.any`  
**Identifier range used here:** `T0040–T0045`

## Purpose

This file defines the semantic laws of `Any`, Phalcom's top proper type.

`Any` is the supertype of every proper type. It must not be confused with unknown solver state, an unresolved inference variable, dynamic typing, or lack of evidence. Those are epistemic or implementation concepts; `Any` is an ordinary semantic type with a precise place in the subtype order.

## Domain and notation

Unless stated otherwise:

- `A` denotes a well-formed proper type of kind `Type`.
- `==` means semantic type identity.
- `<:` means semantic subtype.
- `Unknown` is used only conceptually to denote lack of solver knowledge and is not defined as a proper type here.

## Laws

```phalcom
@id(T0040)
@name(type.any.kind)
AnyKind {
    kind(Any) == Type
}

@id(T0041)
@name(type.any.top)
AnyIsTop<A> {
    A <: Any
}

@id(T0042)
@name(type.any.unique-top)
AnyIsUniqueTop<A> {
    if Any <: A {
        A == Any
    }
}

@id(T0043)
@name(type.any.supertype-characterization)
AnySubtypeCharacterization<A> {
    A == Any iff Any <: A
}

@id(T0044)
@name(type.any.not-unknown)
AnyIsNotUnknown {
    Any != Unknown
}

@id(T0045)
@name(type.any.semantic-not-knowledge-state)
AnyIsSemanticType {
    kind(Any) == Type
    not knowledgeState(Any)
}
```

## T0040 — `AnyKind`

`Any` is a proper type and therefore has kind `Type`.

**Kind:** Semantic law  
**Depends on:** T0010

## T0041 — `AnyIsTop`

Every proper type is a subtype of `Any`.

This is the defining order-theoretic property of top.

**Kind:** Semantic law  
**Depends on:** T0018, T0020–T0023

## T0042 — `AnyIsUniqueTop`

If `Any` itself is a subtype of some proper type `A`, then `A` is semantically identical to `Any`.

This follows from T0041 and subtype antisymmetry.

**Kind:** Derived semantic law  
**Depends on:** T0023, T0041

## T0043 — `AnySubtypeCharacterization`

For a proper type `A`, having `Any <: A` is equivalent to `A` being semantically identical to `Any`.

**Kind:** Derived semantic law  
**Depends on:** T0022, T0042

## T0044 — `AnyIsNotUnknown`

`Any` must never be used to represent lack of information.

This rule is intentionally cross-cutting: a solver may be unable to determine a type or relation, but that state must not silently widen to `Any`.

**Kind:** Solver-safety invariant  
**Depends on:** T0040

## T0045 — `AnyIsSemanticType`

`Any` participates in the ordinary proper-type universe. It is not a proof state, inference sentinel, wildcard, or "untyped" marker.

**Kind:** Semantic/representation boundary invariant  
**Depends on:** T0040, T0044

## Explicitly deferred laws

The following belong elsewhere:

- `A | Any == Any` — `type.union`
- `A & Any == A` — `type.intersection`
- parameter variance involving `Any` — `type.function`
- dynamic or unchecked typing, if introduced — its own semantic family
- inference fallback behavior — `type.inference` / `type.solver`
