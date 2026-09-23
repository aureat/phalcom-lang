# Phalcom Type Algebra Registry — `type.boolean`

**Status:** Normative foundational specification  
**Registry family:** `type.boolean`  
**Identifier range used here:** `T0400–T0412`

## Purpose

This file defines `Bool` as Phalcom's canonical closed two-value proper type.

Its only inhabitants are `true` and `false`. The exact types of those values form the complete finite decomposition of `Bool`.

This specification uses the semantic exact-type notation `exact(true)` and `exact(false)`. If Phalcom later permits `true` and `false` directly in type position, that syntax must denote these exact types rather than introduce a second Boolean-case type system.

## Domain and notation

- `true` and `false` are the two Boolean values.
- `exact(v)` is defined by `type.exact`.
- `inhabitants(A)` denotes semantic cardinality where meaningful.
- `closed(A)` means all inhabitants/cases of `A` are statically known.
- `cases(A)` denotes the finite exact-type case set of a closed type.

## Laws

```phalcom
@id(T0400)
@name(type.boolean.kind)
BoolKind {
    kind(Bool) == Type
}

@id(T0401)
@name(type.boolean.true.inhabits)
TrueInhabitsBool {
    true : Bool
}

@id(T0402)
@name(type.boolean.false.inhabits)
FalseInhabitsBool {
    false : Bool
}

@id(T0403)
@name(type.boolean.values.distinct)
BooleanValuesDistinct {
    not staticEquivalent(true, false)
}

@id(T0404)
@name(type.boolean.values.static)
BooleanValuesStatic {
    staticValue(true)
    staticValue(false)
}

@id(T0405)
@name(type.boolean.cardinality)
BoolHasTwoInhabitants {
    inhabitants(Bool) == 2
}

@id(T0406)
@name(type.boolean.closed)
BoolIsClosed {
    closed(Bool)
}

@id(T0407)
@name(type.boolean.only-values)
BoolHasOnlyBooleanValues<v> {
    if v : Bool {
        v == true or v == false
    }
}

@id(T0408)
@name(type.boolean.true.exact-subtype)
ExactTrueSubtypeBool {
    exact(true) <: Bool
}

@id(T0409)
@name(type.boolean.false.exact-subtype)
ExactFalseSubtypeBool {
    exact(false) <: Bool
}

@id(T0410)
@name(type.boolean.exact-cases.distinct)
BooleanExactCasesDistinct {
    exact(true) != exact(false)
}

@id(T0411)
@name(type.boolean.case-set)
BoolCaseSet {
    cases(Bool) == { exact(true), exact(false) }
}

@id(T0412)
@name(type.boolean.bounded)
BoolBounds {
    Never <: Bool
    Bool <: Any
}
```

## T0400 — `BoolKind`

`Bool` is a proper type.

**Kind:** Semantic law  
**Depends on:** T0050

## T0401 — `TrueInhabitsBool`

`true` is a Boolean value.

**Kind:** Semantic law  
**Depends on:** T0400

## T0402 — `FalseInhabitsBool`

`false` is a Boolean value.

**Kind:** Semantic law  
**Depends on:** T0400

## T0403 — `BooleanValuesDistinct`

The two Boolean values are semantically distinct.

**Kind:** Semantic law  
**Depends on:** none beyond value equality

## T0404 — `BooleanValuesStatic`

Both Boolean values have stable static identity and may be used in exact types and static type-level computations.

**Kind:** Cross-family semantic law  
**Depends on:** `type.static-value`

## T0405 — `BoolHasTwoInhabitants`

`Bool` has cardinality two.

**Kind:** Semantic law  
**Depends on:** T0401–T0403 and the closed-domain characterization in this file

## T0406 — `BoolIsClosed`

The Boolean domain is closed. The compiler may rely on the fact that no third Boolean inhabitant can appear later through extension, subclassing, generic specialization, or runtime registration.

**Kind:** Semantic law  
**Depends on:** T0400

## T0407 — `BoolHasOnlyBooleanValues`

Every value inhabiting `Bool` is either `true` or `false`.

Together with T0401–T0403, this establishes the complete value-level Boolean domain.

**Kind:** Semantic law  
**Depends on:** T0401–T0403, T0406

## T0408 — `ExactTrueSubtypeBool`

The exact singleton type of `true` is a subtype of `Bool`.

**Kind:** Derived semantic law  
**Depends on:** T0355, T0401

## T0409 — `ExactFalseSubtypeBool`

The exact singleton type of `false` is a subtype of `Bool`.

**Kind:** Derived semantic law  
**Depends on:** T0355, T0402

## T0410 — `BooleanExactCasesDistinct`

The two Boolean exact types are distinct because their defining values are distinct.

**Kind:** Derived semantic law  
**Depends on:** T0358, T0403

## T0411 — `BoolCaseSet`

The exact cases of `Bool` are precisely `exact(true)` and `exact(false)`.

This is the canonical finite-domain statement. A later union specification may derive:

```phalcom
Bool == exact(true) | exact(false)
```

but that equation belongs to `type.union` because it introduces union semantics.

**Kind:** Semantic finite-domain law  
**Depends on:** T0351–T0360, T0406–T0410

## T0412 — `BoolBounds`

`Bool` is an ordinary proper type bounded by `Never` and `Any`.

**Kind:** Derived semantic law  
**Depends on:** T0151, T0201

## Explicitly deferred laws

- `Bool == exact(true) | exact(false)` — `type.union`
- `exact(true) & exact(false) == Never` — `type.intersection`
- Boolean pattern exhaustiveness — `type.pattern`
- direct use of `true` / `false` as type syntax
- Boolean operators and value-level truth tables
