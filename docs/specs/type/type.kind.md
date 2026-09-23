# Phalcom Type Algebra Registry — `type.kind`

**Status:** Normative foundational specification  
**Registry family:** `type.kind`  
**Identifier range used here:** `T0010–T0018`

## Purpose

This file defines the foundational well-kindedness laws of the Phalcom type system.

Kinds determine which type-level entities may appear where, which arguments may be supplied to type constructors, and which semantic relations are well formed. The subtype solver must never be used as a fallback mechanism for deciding an ill-kinded expression.

This specification deliberately does not define Phalcom's eventual universe/metatype tower—for example, the kind or metatype of `Type` itself. It defines only the laws required to reason coherently about proper types and type constructors.

## Domain and notation

The conceptual notation used here is:

```phalcom
kind(X)          // the kind of X
F<A>             // type-level application
K1 -> K2         // constructor kind
***K -> R        // variadic constructor kind
wellFormed(E)    // E is semantically well formed
illFormed(E)     // E must be rejected before semantic solving
```

`A: K` in prose means that `A` has kind `K`. Proper types have kind `Type`.

## Laws

```phalcom
@id(T0010)
@name(type.kind.unique)
KindUnique<X, K1, K2> {
    if kind(X) == K1 and kind(X) == K2 {
        K1 == K2
    }
}

@id(T0011)
@name(type.kind.identity-preserving)
IdentityPreservesKind<A, B> {
    if A == B {
        kind(A) == kind(B)
    }
}

@id(T0012)
@name(type.kind.application.result)
TypeApplicationResultKind<F, A, K1, K2> {
    if kind(F) == K1 -> K2 and kind(A) == K1 {
        kind(F<A>) == K2
    }
}

@id(T0013)
@name(type.kind.application.argument-required)
TypeApplicationRequiresArgumentKind<F, A, K1, K2> {
    if kind(F) == K1 -> K2 and kind(A) != K1 {
        illFormed(F<A>)
    }
}

@id(T0014)
@name(type.kind.application.partial)
PartialTypeApplicationKind<F, A, K1, K2, K3> {
    if kind(F) == K1 -> K2 -> K3 and kind(A) == K1 {
        kind(F<A>) == K2 -> K3
    }
}

@id(T0015)
@name(type.kind.application.full)
FullTypeApplicationKind<F, A, B, K1, K2, K3> {
    if kind(F) == K1 -> K2 -> K3
        and kind(A) == K1
        and kind(B) == K2
    {
        kind(F<A, B>) == K3
    }
}

@id(T0016)
@name(type.kind.application.variadic)
VariadicTypeApplicationKind<F, ***As, K, R> {
    if kind(F) == ***K -> R and kinds(***As) == ***K {
        kind(F<***As>) == R
    }
}

@id(T0017)
@name(type.kind.identity.compatible-operands)
TypeIdentityRequiresCompatibleKinds<A, B> {
    if wellFormed(A == B) {
        kind(A) == kind(B)
    }
}

@id(T0018)
@name(type.kind.subtype.proper-type-operands)
SubtypeRequiresProperTypeOperands<A, B> {
    if wellFormed(A <: B) {
        kind(A) == Type
        kind(B) == Type
    }
}
```

## T0010 — `KindUnique`

Every well-formed type-level entity has one semantic kind in a given typing environment.

Different syntax may denote the same kind, and kind expressions may themselves normalize, but an entity may not simultaneously have two semantically distinct kinds.

**Kind:** Semantic law  
**Depends on:** T0001–T0003

## T0011 — `IdentityPreservesKind`

Semantically identical type-level entities must have identical kinds.

This prevents semantic identity from collapsing entities that inhabit different levels of the type language. If two expressions have different kinds, they cannot be semantically identical under the ordinary `==` relation.

**Kind:** Cross-family semantic law  
**Depends on:** T0001–T0003, T0010

## T0012 — `TypeApplicationResultKind`

Applying a constructor of kind `K1 -> K2` to an argument of kind `K1` produces a result of kind `K2`.

This is the fundamental kinding rule for type-level application.

**Kind:** Semantic law  
**Depends on:** T0010

## T0013 — `TypeApplicationRequiresArgumentKind`

Type-level application is ill formed when the supplied argument does not inhabit the constructor's required input kind.

The compiler must reject the application as a kind error. It must not manufacture an unresolved type, ask the subtype solver to decide the mismatch, or silently reinterpret the constructor.

**Kind:** Well-formedness invariant  
**Depends on:** T0010, T0012

## T0014 — `PartialTypeApplicationKind`

A multi-argument constructor may be partially applied when its kind permits partial application. The result retains the unapplied portion of the constructor kind.

The conceptual arrow notation is right-associative:

```text
K1 -> K2 -> K3
==
K1 -> (K2 -> K3)
```

**Kind:** Semantic law  
**Depends on:** T0012

## T0015 — `FullTypeApplicationKind`

Supplying all required arguments to a multi-argument constructor produces the constructor's result kind.

`F<A, B>` is conceptual notation for full application; an implementation may internally represent it as nested application if Phalcom's type-constructor model is curried.

**Kind:** Semantic law  
**Depends on:** T0012, T0014

## T0016 — `VariadicTypeApplicationKind`

A variadic type constructor accepts a pack whose element kinds match the constructor's variadic input kind and produces the declared result kind.

This law is intended to cover constructors such as a variadic `Tuple` without prescribing the final surface syntax of kind packs.

**Kind:** Semantic law  
**Depends on:** T0012

## T0017 — `TypeIdentityRequiresCompatibleKinds`

The semantic identity relation is only well formed between kind-compatible operands.

This does not require an implementation to evaluate both kinds before every equality check. It requires the observable semantics to reject or prevent cross-kind equality expressions rather than treating them as ordinary unequal proper types.

**Kind:** Well-formedness invariant  
**Depends on:** T0001–T0003, T0010, T0011

## T0018 — `SubtypeRequiresProperTypeOperands`

The ordinary `<:` relation is defined over proper types. Both operands must therefore have kind `Type`.

Higher-kinded constructor variance or constructor-level relations, if Phalcom later supports them, must be specified explicitly rather than being smuggled through ordinary proper-type subtyping.

**Kind:** Well-formedness invariant  
**Depends on:** T0010

## Explicitly deferred laws

This file does not yet define:

- the kind or metatype of `Type` itself
- reflective `T::Type` / `.type` semantics
- user-defined kinds introduced through `type data`
- constructor variance
- generic constructor injectivity
- associated-type projection kinds
- type-function reduction
- higher-kinded subtyping, if Phalcom ever introduces it

Those belong to later semantic families and must preserve T0010–T0018.
