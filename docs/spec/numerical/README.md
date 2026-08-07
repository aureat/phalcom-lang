# Phalcom Numeric Specification

This directory defines Phalcom's complete numeric language contract for `Number`, `Int`, and `Float`, together with the required runtime architecture, conformance suite, and migration boundary.

The documents are authoritative as a set. Public semantics do not depend on host integer width, allocator layout, C-library behavior, locale, or the internal algorithms of a particular dependency.

## Document map

| File | Purpose |
|---|---|
| [`numeric-tower.md`](numeric-tower.md) | Public classes, arithmetic, comparison, conversion, numeric keys, hashing, and tower closure. |
| [`float-protocol.md`](float-protocol.md) | Binary64 interpretation, signed zero, NaN, narrowing, remainder, power, and total ordering. |
| [`numeric-literals.md`](numeric-literals.md) | Source grammar, token boundaries, classification, oversized constants, limits, and diagnostics. |
| [`text-and-errors.md`](text-and-errors.md) | Text constructors, correctly rounded parsing, canonical rendering, and structured numeric errors. |
| [`bitwise.md`](bitwise.md) | Infinite-two's-complement Int operations, syntax, huge-count behavior, and primitive floor. |
| [`implementation.md`](implementation.md) | Runtime representation, semantic kernels, constants, hashing, limits, closed-kernel intrinsics, and landing architecture. |
| [`conformance.md`](conformance.md) | Reference models, properties, edge corpora, differential tests, and release gates. |
| [`migration.md`](migration.md) | Breaking changes, source and native migration, bytecode incompatibility, and atomic pre-1.0 release policy. |

## Normative hierarchy

1. `numeric-tower.md`, `float-protocol.md`, `numeric-literals.md`, `text-and-errors.md`, and `bitwise.md` define public language behavior.
2. `conformance.md` defines what every implementation must prove.
3. `implementation.md` defines required architecture while keeping private representations unobservable.
4. `migration.md` defines the compatibility boundary of the numeric update.

The words **must**, **must not**, **shall**, and **shall not** are normative. **Should** records a strong implementation default. **May** grants permission.

## Closed invariants

```text
1.class == Int
1.0.class == Float
Int.isA(Number)
Float.isA(Number)
Number has no instances
Int is exact and arbitrary precision
Float is IEEE-754 binary64
6 / 2 has class Float
~/ returns exact Int
Int and equal finite Float compare by exact mathematical value
Map and Set merge equivalent numeric keys
all NaNs form one numeric-key equivalence class
Float indices and counts are rejected where Int is required
Number, Int, and Float are closed kernel classes after bootstrap
exact built-in numeric operations may bypass message dispatch through equivalent intrinsics
```

## Scope boundary

The concrete built-in tower contains only `Int` and `Float`; user-defined classes cannot subclass `Number`. This specification introduces no Decimal, Rational, Complex, fixed-width integer, unsigned integer, SIMD numeric value, implicit user-defined coercion, reverse numeric operator, public `LargeInt`, persistent hash value, or public numeric-extension hook.
