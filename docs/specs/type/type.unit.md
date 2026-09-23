# Phalcom Type Algebra Registry — `type.unit`

**Status:** Normative foundational specification  
**Registry family:** `type.unit`  
**Identifier range used here:** `T0050–T0057`

## Purpose

This file defines `Unit` as Phalcom's canonical one-inhabitant proper type.

The sole inhabitant is `()`. `Unit` is therefore neither bottom nor top. It is a finite singleton type and forms the canonical bridge between ordinary proper types and exact/singleton-value typing.

## Domain and notation

- `()` denotes the unit value.
- `v : A` means value `v` inhabits type `A`.
- `inhabitants(A)` denotes semantic cardinality where meaningful.
- `exact(v)` denotes the exact singleton type of value `v`; the exact-type family defines this operation.
- `==` means semantic type identity.

## Laws

```phalcom
@id(T0050)
@name(type.unit.kind)
UnitKind {
    kind(Unit) == Type
}

@id(T0051)
@name(type.unit.value.inhabits)
UnitValueInhabitsUnit {
    () : Unit
}

@id(T0052)
@name(type.unit.singleton)
UnitHasOneInhabitant {
    inhabitants(Unit) == 1
}

@id(T0053)
@name(type.unit.only-inhabitant)
UnitHasOnlyUnitValue<v> {
    if v : Unit {
        v == ()
    }
}

@id(T0054)
@name(type.unit.not-never)
UnitIsNotNever {
    Unit != Never
}

@id(T0055)
@name(type.unit.not-any)
UnitIsNotAny {
    Unit != Any
}

@id(T0056)
@name(type.unit.bounded)
UnitBounds {
    Never <: Unit
    Unit <: Any
}

@id(T0057)
@name(type.unit.exact)
UnitIsExactUnitValue {
    Unit == exact(())
}
```

## T0050 — `UnitKind`

`Unit` is a proper type.

**Kind:** Semantic law  
**Depends on:** T0010

## T0051 — `UnitValueInhabitsUnit`

The value `()` inhabits `Unit`.

**Kind:** Semantic law  
**Depends on:** T0050

## T0052 — `UnitHasOneInhabitant`

`Unit` has exactly one semantic inhabitant.

**Kind:** Semantic law  
**Depends on:** T0051

## T0053 — `UnitHasOnlyUnitValue`

Every inhabitant of `Unit` is the unit value `()`.

Together with T0051, this fully characterizes `Unit` extensionally as `{()}`.

**Kind:** Semantic law  
**Depends on:** T0051, T0052

## T0054 — `UnitIsNotNever`

`Unit` is not bottom because it has an inhabitant whereas `Never` has none.

**Kind:** Derived semantic law  
**Depends on:** T0032–T0034, T0051

## T0055 — `UnitIsNotAny`

`Unit` is not the universal top type.

This law is explicit because collapsing one-inhabitant `Unit` with `Any` would destroy the intended subtype lattice and cardinality semantics.

**Kind:** Semantic law  
**Depends on:** T0041, T0052

## T0056 — `UnitBounds`

`Unit` occupies an ordinary point in the proper-type order: above `Never` and below `Any`.

**Kind:** Derived semantic law  
**Depends on:** T0031, T0041

## T0057 — `UnitIsExactUnitValue`

Once exact-value types are admitted, `Unit` is semantically identical to the exact type of `()`.

This is an intentional unification law: Phalcom does not need two distinct singleton types for the unit value.

**Kind:** Cross-family semantic law  
**Depends on:** T0051–T0053 and the `type.exact` family

## Explicitly deferred laws

- syntax/metatype behavior of `().type`
- exact-type constructor representation
- union/intersection behavior with `Unit`
- function-return conventions for methods that conceptually return no meaningful value
