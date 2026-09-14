# 10. `Value` is a 16-byte explicit tagged representation with a private `Nil` sentinel

- Status: Accepted (amended by [ADR-0024](0024-numeric-surface-split-int-float-and-division.md) and Spec 2: 16-byte Value Representation)
- Date: 2026-07-11
- Related: `docs/spec/current/object-model.md` §3; `docs/spec/current/values-and-absence.md` §2; [ADR-0005](../retired/0005-number-as-flat-f64.md); [ADR-0009](0009-handle-arena-heap.md); [ADR-0024](0024-numeric-surface-split-int-float-and-division.md)

> **Amended 2026-08-16 by Spec 2 (16-Byte Value Representation).**
> `Value` is implemented as an explicit 16-byte struct (`payload: u64, meta: u64`)
> with 8 base tags (Nil, Unit, Bool, Int, Float, Symbol, Obj, None) and an inline
> 32-bit Option nesting depth in `meta` bits 8..=39. This eliminates enum discriminant
> padding, guarantees 16-byte `Copy` layout across all platforms, supports arbitrary Option
> nesting depth up to `u32::MAX`, and paves the way for NaN-boxing.

> **Numeric arm amended (2026-07-12) by [ADR-0024](0024-numeric-surface-split-int-float-and-division.md).**
> The single `Number(f64)` arm below is replaced by two arms — `Int(i64)` (exact,
> auto-promoting to a heap `LargeInt` on overflow) and `Float(f64)`. The rest of the
> tags (`Bool`/`Obj`/`Symbol`/private `Nil`) is unchanged.

## Context

Every surface value maps onto a class ([Object Model §3](../../spec/current/object-model.md)),
but the VM needs one in-register representation for all of them. [ADR-0005](../retired/0005-number-as-flat-f64.md)
settled the numeric arm (`Number` = flat `f64`) but not the whole value type. Two
constraints frame the rest:

- **`nil` is private (Invariant 4).** The VM keeps a `nil` for uninitialized slots
  and internal sentinels, but it has no surface class, no literal, and cannot be
  produced by user code ([Values & Absence §2](../../spec/current/values-and-absence.md)).
  Absence is `Option`.
- Heap objects are now handles, not pointers ([ADR-0009](0009-handle-arena-heap.md)),
  so the value type must carry an `ObjRef`, not an `Rc`.

## Decision (as amended by Spec 2, 2026-08-16)

`Value` is a **16-byte explicit tagged struct** (`payload: u64, meta: u64`) with
8 base tags encoded in `meta` bits 0..=7:

| Tag | Name    | Payload |
|-----|---------|---------|
| 0   | Nil     | zero (private uninitialized sentinel) |
| 1   | Unit    | zero |
| 2   | Bool    | `0` or `1` |
| 3   | Int     | `i64` bit pattern |
| 4   | Float   | `f64` bit pattern |
| 5   | Symbol  | `u32` interned id |
| 6   | Obj     | opaque `ObjRef` token |
| 7   | None    | zero |

Option nesting depth is encoded in `meta` bits 8..=39 (a `u32` field).
Bits 40..=63 are reserved and must be zero.

The bounded `Some1`…`Some7` variants from PDR-0033 are **retired**. A `Some(x)`
at any depth is represented by setting the depth field to the nesting count and
retaining `x`'s tag and payload. This supports arbitrary nesting depth up to
`u32::MAX` in two 64-bit words with zero heap allocation.

## Consequences

- One uniform value carried in registers and on the stack; `x.class` is total for
  every tag except the private `Nil`, which is never observed by user code.
- Playing well with the handle heap: `Value` is `Copy`, so it stays cheap to move
  and never owns heap memory directly; an `Option` wrapping an object value still
  carries a traced `ObjRef` payload inspectable via `Value::gc_obj_ref()`.
- The private `Nil` gives the VM a zero-cost "not yet assigned" marker while the
  surface language keeps its no-`nil` invariant; enforcing that it never escapes is
  a standing obligation of the Option work (values-and-absence).
- **NaN-boxing is deferred.** Packing `Value` into a single NaN-tagged `f64` word is
  a later optimization *behind this same enum API* — it is a deferred consequence,
  not part of the accepted decision. Correctness and legibility of the tagged enum
  win now; the boxed representation can replace the layout without changing callers.

## Alternatives considered

- **A surface `nil` value** (JavaScript/Wren style). Rejected: it reintroduces the
  null coercion the object model exists to remove ([Values & Absence §2](../../spec/current/values-and-absence.md),
  Invariant 4); absence is `Option` ([ADR-0007](0007-option-as-abstract-with-some-none.md)).
- **NaN-boxing from the start.** Fewer bytes per value and better cache behavior,
  but it obscures the representation, complicates debugging, and buys speed before
  the VM is correct. Reserved as a deferred optimization behind the enum API.
- **`Rc`-carrying object arm.** Superseded by the handle heap ([ADR-0009](0009-handle-arena-heap.md));
  `Value` carries an `ObjRef`, not an owning pointer.
