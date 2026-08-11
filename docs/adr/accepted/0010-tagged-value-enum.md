# 10. `Value` is a tagged `enum` with a private `Nil` sentinel

- Status: Accepted (numeric arm amended by [ADR-0024](0024-numeric-surface-split-int-float-and-division.md))
- Date: 2026-07-11
- Related: `docs/spec/current/object-model.md` §3; `docs/spec/current/values-and-absence.md` §2; [ADR-0005](../retired/0005-number-as-flat-f64.md); [ADR-0009](0009-handle-arena-heap.md); [ADR-0024](0024-numeric-surface-split-int-float-and-division.md)

> **Numeric arm amended (2026-07-12) by [ADR-0024](0024-numeric-surface-split-int-float-and-division.md).**
> The single `Number(f64)` arm below is replaced by two arms — `Int(i64)` (exact,
> auto-promoting to a heap `LargeInt` on overflow) and `Float(f64)`. The rest of the
> enum (`Bool`/`Obj`/`Symbol`/private `Nil`) is unchanged.

> **Amended 2026-08-11 by PDR-0033.** The tagged `Value` API now admits immediate
> bounded Option state (`None`, `Some1` … `Some7`). This is a correctness-first
> landing substrate, not the permanent bit-level design; final encoding and
> NaN-boxing remain deferred.

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

## Decision

`Value` is a **tagged Rust `enum`** with these arms, including the bounded
immediate Option variants admitted by PDR-0033:

- `Int(i64)` / `Float(f64)` — the two numeric arms ([ADR-0024](0024-numeric-surface-split-int-float-and-division.md), amending ADR-0005). `Int` is the small-integer fast path; large integers box to a heap `LargeInt`. *(Originally one `Number(f64)` arm; split by ADR-0024.)*
- `Bool(bool)` — one `Bool` class; `True`/`False` are a later dispatch refinement,
  not a `Value` arm ([ADR-0004](0004-boolean-as-abstract-bool-with-true-false.md)).
- `Obj(ObjRef)` — every heap object (instances, strings, blocks, classes, …) by
  handle into the `Heap` ([ADR-0009](0009-handle-arena-heap.md)).
- `None` and `Some1`…`Some7` — immediate bounded Option state. A `SomeN` payload
  is non-recursive and may itself contain an immediate scalar or one heap handle;
  the collector uses `Value::gc_obj_ref()` to see that edge.
- `Symbol(...)` — interned identifiers/selectors.
- `Nil` — a **private** sentinel for uninitialized slots. It is not surface-visible:
  it has no class row ([Object Model §3](../../spec/current/object-model.md)), the compiler
  never emits a literal for it, and it must never leak into a `Some` or reach user
  code (Invariant 4).

Clarity and safety come first: the enum is the API every other subsystem programs
against.

## Consequences

- One uniform value carried in registers and on the stack; `x.class` is total for
  every arm except the private `Nil`, which is never observed by user code.
- Playing well with the handle heap: `Value` is `Copy`, so it stays cheap to move
  and never owns heap memory directly; immediate `SomeN` can still carry a traced
  `ObjRef` payload.
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
