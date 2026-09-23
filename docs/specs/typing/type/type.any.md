# Phalcom Type Algebra Registry — `type.any`

**Status:** Normative foundational specification  
**Registry family:** `type.any`  
**Identifier range used here:** `T0200–T0205`

## Purpose

This file defines the semantic laws of `Any`, Phalcom's top proper type.

`Any` is the supertype of every proper type. It must not be confused with unknown solver state, an unresolved inference variable, dynamic typing, or lack of evidence. Those are epistemic or implementation concepts; `Any` is an ordinary semantic type with a precise place in the subtype order.

## Domain and notation

Unless stated otherwise:

- `A` denotes a well-formed proper type of kind `Type`.
- `==` means semantic type identity.
- `<:` means semantic subtype.
- Solver proof states such as `Unknown` are not type operands and are therefore outside the domain of `==` in this file.

## Laws

```phalcom
@id(T0200)
@name(type.any.kind)
AnyKind {
    kind(Any) == Type
}

@id(T0201)
@name(type.any.top)
AnyIsTop<A> {
    A <: Any
}

@id(T0202)
@name(type.any.unique-top)
AnyIsUniqueTop<A> {
    if Any <: A {
        A == Any
    }
}

@id(T0203)
@name(type.any.supertype-characterization)
AnySubtypeCharacterization<A> {
    A == Any iff Any <: A
}

@id(T0204)
@name(type.any.semantic-type)
AnyIsSemanticType {
    semanticType(Any)
    kind(Any) == Type
}

@id(T0205)
@name(type.any.not-knowledge-state)
AnyIsNotKnowledgeState {
    not knowledgeState(Any)
}
```

## T0200 — `AnyKind`

`Any` is a proper type and therefore has kind `Type`.

**Kind:** Semantic law  
**Depends on:** T0050

## T0201 — `AnyIsTop`

Every proper type is a subtype of `Any`.

This is the defining order-theoretic property of top.

**Kind:** Semantic law  
**Depends on:** T0060, T0100–T0103

## T0202 — `AnyIsUniqueTop`

If `Any` itself is a subtype of some proper type `A`, then `A` is semantically identical to `Any`.

This follows from T0201 and subtype antisymmetry.

**Kind:** Derived semantic law  
**Depends on:** T0103, T0201

## T0203 — `AnySubtypeCharacterization`

For a proper type `A`, having `Any <: A` is equivalent to `A` being semantically identical to `Any`.

**Kind:** Derived semantic law  
**Depends on:** T0102, T0202

## T0204 — `AnyIsSemanticType`

`Any` participates in the ordinary proper-type universe.

**Kind:** Semantic law  
**Depends on:** T0200

## T0205 — `AnyIsNotKnowledgeState`

`Any` is not a proof state, inference sentinel, wildcard, or representation of missing knowledge. Solver states live in a different semantic category and therefore are not compared to `Any` with type identity.

**Kind:** Semantic/solver boundary invariant  
**Depends on:** T0204

## Explicitly deferred laws

The following belong elsewhere:

- `A | Any == Any` — `type.union`
- `A & Any == A` — `type.intersection`
- parameter variance involving `Any` — `type.function`
- dynamic or unchecked typing, if introduced — its own semantic family
- inference fallback behavior — `type.inference` / `type.solver`
