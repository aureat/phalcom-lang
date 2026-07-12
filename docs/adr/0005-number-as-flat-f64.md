# 5. Keep a single flat `Number` type backed by `f64`

- Status: Superseded in part by [ADR-0024](0024-numeric-surface-split-int-float-and-division.md)
- Date: 2026-07-11
- Related: `docs/object-model.md`; `phalcom-core/src/value.rs`, `primitive/number.rs`

> **Superseded in part (2026-07-12) by [ADR-0024](0024-numeric-surface-split-int-float-and-division.md).**
> The `f64` representation decided here survives as the representation of the new
> `Float` type. ADR-0024 splits the *surface* into abstract `Number` over exact,
> unbounded `Int` (auto-promoting bignum) and `Float`, so this ADR's "single
> surface `Number` class / integers are really floats" clauses no longer hold.

## Context

Phalcom currently models all numbers with one `Number` class backed by a
`f64` (`Value::Number(f64)`). The alternative is a numeric tower — distinct
`Integer`/`Float` (and possibly arbitrary-precision) classes — as some languages
provide.

## Decision

**Recommendation (pending approval):** keep the single flat `Number` type backed
by `f64` for now.

- One `Number` class, one `Value::Number(f64)` variant.
- Defer any `Integer`/`Float` split or bignum support until there is a concrete
  need (e.g. exact integer semantics, bitwise ops, or big integers).

## Consequences

- Simplest possible numeric model: one class, one variant, one set of
  primitives — keeps the bootstrap and dispatch small.
- Inherits `f64` limitations: no exact large integers, `0.1 + 0.2` rounding,
  and integer-looking values are really floats. Acceptable for the current
  stage; revisit if the language targets domains needing exact arithmetic.
- Reversible: introducing a numeric tower later is a new ADR that supersedes this
  one. Keeping it flat now avoids premature complexity in the object model.

## Alternatives considered

- **Numeric tower (`Integer` + `Float`).** More precise semantics and better
  integer ergonomics, but adds classes, coercion rules, and dispatch complexity
  before the language needs them.
