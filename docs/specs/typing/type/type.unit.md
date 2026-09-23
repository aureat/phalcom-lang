# Phalcom Type Algebra Registry — `type.unit`

**Status:** Normative foundational specification  
**Registry family:** `type.unit`  
**Identifier range used here:** `T0300–T0307`

## Purpose

This file defines `Unit` as Phalcom's canonical one-inhabitant proper type.

The sole inhabitant is `()`. `Unit` is therefore neither bottom nor top. It is a finite singleton type and forms the canonical bridge between ordinary proper types and exact/singleton-value typing.

## Domain and notation

- `()` denotes the unit value.
- `v : A` means value `v` inhabits type `A`.
- `inhabitants(A)` denotes semantic cardinality where meaningful.
- `==` means semantic type identity.

## Laws

```phalcom
@id(T0300)
@name(type.unit.kind)
UnitKind {
    kind(Unit) == Type
}

@id(T0301)
@name(type.unit.value.inhabits)
UnitValueInhabitsUnit {
    () : Unit
}

@id(T0302)
@name(type.unit.singleton)
UnitHasOneInhabitant {
    inhabitants(Unit) == 1
}

@id(T0303)
@name(type.unit.only-inhabitant)
UnitHasOnlyUnitValue<v> {
    if v : Unit {
        v == ()
    }
}

@id(T0304)
@name(type.unit.not-never)
UnitIsNotNever {
    Unit != Never
}

@id(T0305)
@name(type.unit.not-any)
UnitIsNotAny {
    Unit != Any
}

@id(T0306)
@name(type.unit.bounded)
UnitBounds {
    Never <: Unit
    Unit <: Any
}

@id(T0307)
@name(type.unit.value.static)
UnitValueIsStatic {
    staticValue(())
}
```

## T0300 — `UnitKind`

`Unit` is a proper type.

**Kind:** Semantic law  
**Depends on:** T0050

## T0301 — `UnitValueInhabitsUnit`

The value `()` inhabits `Unit`.

**Kind:** Semantic law  
**Depends on:** T0300

## T0302 — `UnitHasOneInhabitant`

`Unit` has exactly one semantic inhabitant.

**Kind:** Semantic law  
**Depends on:** T0301

## T0303 — `UnitHasOnlyUnitValue`

Every inhabitant of `Unit` is the unit value `()`.

Together with T0301, this fully characterizes `Unit` extensionally as `{()}`.

**Kind:** Semantic law  
**Depends on:** T0301, T0302

## T0304 — `UnitIsNotNever`

`Unit` is not bottom because it has an inhabitant whereas `Never` has none.

**Kind:** Derived semantic law  
**Depends on:** T0152–T0154, T0301

## T0305 — `UnitIsNotAny`

`Unit` is not the universal top type.

This law is explicit because collapsing one-inhabitant `Unit` with `Any` would destroy the intended subtype lattice and cardinality semantics.

**Kind:** Semantic law  
**Depends on:** T0201, T0302

## T0306 — `UnitBounds`

`Unit` occupies an ordinary point in the proper-type order: above `Never` and below `Any`.

**Kind:** Derived semantic law  
**Depends on:** T0151, T0201

## T0307 — `UnitValueIsStatic`

The unit value has stable compile-time identity and is admitted to the static-value universe. The later exact-type family may therefore form `exact(())`.

**Kind:** Cross-family semantic law  
**Depends on:** T0301 and `type.static-value`

## Explicitly deferred laws

- syntax/metatype behavior of `().type`
- exact-type constructor representation
- union/intersection behavior with `Unit`
- function-return conventions for methods that conceptually return no meaningful value
