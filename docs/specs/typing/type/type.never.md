# Phalcom Type Algebra Registry — `type.never`

**Status:** Normative foundational specification  
**Registry family:** `type.never`  
**Identifier range used here:** `T0150–T0156`

## Purpose

This file defines the semantic laws of `Never`, Phalcom's bottom proper type.

`Never` denotes the empty type: it has no inhabitants and is a subtype of every proper type. These properties are semantic, not merely normalization conventions. Union, intersection, control-flow, function-return, and pattern-space consequences may depend on these laws, but their operator-specific rules belong to their respective registry families.

## Domain and notation

Unless stated otherwise:

- `A` denotes a well-formed proper type of kind `Type`.
- `v : A` means value `v` inhabits type `A`.
- `inhabitants(A)` denotes the semantic inhabitant set/cardinality abstraction where meaningful.
- `empty(A)` means the type has no possible inhabitants.
- `==` means semantic type identity.
- `<:` means semantic subtype.

## Laws

```phalcom
@id(T0150)
@name(type.never.kind)
NeverKind {
    kind(Never) == Type
}

@id(T0151)
@name(type.never.bottom)
NeverIsBottom<A> {
    Never <: A
}

@id(T0152)
@name(type.never.empty)
NeverIsEmpty {
    empty(Never)
}

@id(T0153)
@name(type.never.no-inhabitants)
NeverHasNoInhabitants {
    inhabitants(Never) == 0
}

@id(T0154)
@name(type.never.no-value)
NoValueHasNeverType<v> {
    not (v : Never)
}

@id(T0155)
@name(type.never.unique-bottom)
NeverIsUniqueBottom<A> {
    if A <: Never {
        A == Never
    }
}

@id(T0156)
@name(type.never.subtype-characterization)
NeverSubtypeCharacterization<A> {
    A == Never iff A <: Never
}
```

## T0150 — `NeverKind`

`Never` is a proper type and therefore has kind `Type`.

**Kind:** Semantic law  
**Depends on:** T0050

## T0151 — `NeverIsBottom`

`Never` is a subtype of every proper type.

This is the defining order-theoretic property of bottom. No nominal, structural, generic, exact, or user-defined proper type may sit below `Never` as a distinct semantic type.

**Kind:** Semantic law  
**Depends on:** T0060, T0100–T0103

## T0152 — `NeverIsEmpty`

`Never` denotes the empty type.

This law states semantic emptiness independently from any particular representation of inhabitant sets.

**Kind:** Semantic law  
**Depends on:** T0150

## T0153 — `NeverHasNoInhabitants`

The cardinality of `Never`, where represented, is zero.

This is an explicit audit form of T0152 useful for finite-domain reasoning, exact types, pattern exhaustiveness, and runtime reflection.

**Kind:** Derived semantic law  
**Depends on:** T0152

## T0154 — `NoValueHasNeverType`

No value may inhabit `Never`.

Expressions may have static type `Never` when they cannot complete normally, but no successfully produced runtime value has type `Never`.

**Kind:** Semantic law  
**Depends on:** T0152, T0153

## T0155 — `NeverIsUniqueBottom`

Any proper type that is itself a subtype of `Never` is semantically identical to `Never`.

This follows from T0151 and subtype antisymmetry.

**Kind:** Derived semantic law  
**Depends on:** T0103, T0151

## T0156 — `NeverSubtypeCharacterization`

For a proper type `A`, being a subtype of `Never` is equivalent to being semantically identical to `Never`.

**Kind:** Derived semantic law  
**Depends on:** T0102, T0155

## Explicitly deferred laws

The following belong elsewhere:

- `A | Never == A` — `type.union`
- `A & Never == Never` — `type.intersection`
- functions returning `Never` — `type.function`
- non-completing expression/control-flow behavior — control-flow semantics
- exhaustiveness reducing remaining space to `Never` — `type.pattern`
- normalization removal of `Never` from unions — `type.normalization`
