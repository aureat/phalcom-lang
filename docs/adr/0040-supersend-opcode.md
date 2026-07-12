# 40. Add the `SuperSend` dispatch opcode for `super.sel(…)`

- Status: Proposed
- Date: 2026-07-12
- Related: [ADR-0012](0012-label-encoded-selectors.md) (label-encoded selectors
  and IC-ready message-send dispatch — `SuperSend` is a sibling send opcode to
  `Invoke`, keying on the same selector encoding); [ADR-0002](0002-metaclass-tower-parallel-rule.md)
  (parallel-metaclass rule — makes inherited `static`/`construct` reachable, the
  super-construct half of this opcode); [ADR-0011](0011-fixed-instance-slot-layout.md)
  (fixed slot layout — subclass fields stack, so super-construct fills inherited
  slots in place without aliasing); [ADR-0018](0018-sacred-selector-inliner-and-override-guard.md)
  (override-epoch guard — the IC-invalidation seam a future cached `SuperSend`
  must share); `docs/forge/units/U-INH/plan.md` (DEC-INH-B/D/F — the decisions
  this ADR records); `docs/spec/v0.2/method-lookup.md` §1.14 (`super` semantics);
  `docs/spec/v0.2/object-model.md` §5.1 (`extends` surface)

## Context

`super.sel(…)` starts method lookup at the **superclass of the method's
defining class**, with the original receiver (`self`) unchanged
(method-lookup.md §1.14). Ordinary sends compile to `Invoke(argc, selector)`
([ADR-0012](0012-label-encoded-selectors.md)), whose lookup start is the
*receiver's* class — the wrong start for `super`, and one that would recurse
forever if a method sent `super.sel` for the same `sel` it defines. Before
U-INH, `super` parsed but the compiler emitted `Nil` for it, so `super` was
silently broken.

`super` needs a start class that is **static per call site** (the defining
class is known at compile time) rather than derived from the receiver. That is
a different dispatch shape from `Invoke`, so it warrants its own opcode rather
than an overload of `Invoke`'s operands.

## Decision

Add a third message-send opcode:

```
SuperSend(argc: u8, selector: u16, defining_class_name: u16)
```

- **`defining_class_name`** is a constant-pool index to the **defining class's
  name symbol** — not its superclass, and not a class handle (DEC-INH-B). The
  class object does not exist at compile time, so the name is baked and resolved
  to a class at dispatch (the same global lookup `GetGlobal` performs). The VM
  computes `defining.superclass` **at dispatch time**, so the send stays correct
  under a future runtime `superclass=` mutation (U13).
- **Receiver** stays the original `self`, pushed by the compiler ahead of the
  args, so an overridden method runs its *super*'s definition against the same
  instance.
- **Walk**: start at `defining.superclass` on the instance side. For a
  super-**construct** — the selector decodes to a positional `Method` kind that
  misses the instance side — re-encode to the `Initializer` form and walk the
  superclass's **metaclass** chain, where constructors are installed
  ([ADR-0002](0002-metaclass-tower-parallel-rule.md) parallel rule). `NewInstance`
  is made idempotent on an existing instance so the parent initializer fills the
  inherited slots in place ([ADR-0011](0011-fixed-instance-slot-layout.md)), never
  aliasing the subclass's fresh slots.
- **Miss**: a walk that exhausts the chain routes to the **same**
  `doesNotUnderstand(_:)` → surface `MessageNotUnderstood` path as an ordinary
  `Invoke` miss (U-CORE-6) — never a panic.
- **Errors**: a bare `super`, or `super` outside any method (top level / free
  function), is a **compile error** — there is no defining class to anchor the
  walk, and `super` has no value on its own.

This is a **VM-opcode addition, not a frozen-floor primitive** — the `.ph` floor
census is unchanged ([ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) is not
amended). Net delta: **+1 opcode, 0 floor primitives.**

## Consequences

- `super` is real end-to-end: inherited-method extension, multi-level chains,
  and explicit super-construct chaining (DEC-INH-C).
- **Inline caching (DEC-INH-F):** `SuperSend` has a static per-call-site start
  class, so its cache key differs from a receiver-polymorphic `Invoke`. This
  first cut is **uncached** (correct, simple). A later cached `SuperSend` must be
  invalidated by the same `superclass=` (U15) / override-epoch bump
  ([ADR-0018](0018-sacred-selector-inliner-and-override-guard.md)) that
  invalidates `Invoke` — flagged to U15/U16, not silently diverged.
- No `Value` tag change and no selector-encoding change; the `Class`-bytecode
  stack contract is unchanged.

## Alternatives considered

- **Amend `Invoke` instead of a new opcode** — rejected: `super`'s start class
  is structurally unlike a receiver-derived start; overloading `Invoke`'s
  operands would blur two dispatch shapes and complicate a future IC.
- **Bake the superclass directly** (not the defining class) — rejected
  (DEC-INH-B): it would go stale under a runtime `superclass=` and does not match
  "superclass of the defining class" literally.
- **Dynamic-receiver-class super** — rejected: breaks the "defining class" rule
  for multi-level chains.
- **Implicit constructor auto-chaining** — rejected (DEC-INH-C): explicit
  `super.construct(…)` matches the Smalltalk/Wren precedent and keeps ADR-0011
  slot initialization predictable.
