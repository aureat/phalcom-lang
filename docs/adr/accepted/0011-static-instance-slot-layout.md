# 11. Instances use a static per-class slot layout

- Status: Accepted
- Date: 2026-07-11
- Related: `docs/spec/current/classes.md` §2; `docs/spec/current/object-model.md` §2; [ADR-0009](0009-handle-arena-heap.md); [ADR-0010](0010-tagged-value-enum.md)

## Context

Fields in Phalcom are `_`-prefixed and **implicitly declared by assignment**: the
compiler collects the fields assigned anywhere in a class body and the spec fixes
the layout at class-definition time ([Classes §2](../../spec/current/classes.md)). The spec
also pins three properties that a field model must honor:

- **Read-before-write is a compile error** — reading a field never assigned in any
  method is rejected at compile time, catching the `_naem = name` typo class.
- **A declared-but-unassigned field reads `None`** ([Values & Absence](../../spec/current/values-and-absence.md)).
- **Fields are private and not inheritance-visible** — a subclass writing `_name`
  gets its own slot, so offsets stay stable and the fragile-base-class problem is
  eliminated.

The current tree stores fields in a per-instance `IndexMap<Symbol, Value>` — a
dynamic map keyed by symbol. That contradicts "layout fixed at class-definition
time," pays a hash probe per field access, and makes read-before-write undetectable.

## Decision

An instance carries a **fixed slot vector**, not a map:

- `InstanceObject { class, slots: Box<[Value]> }`, indexed by a **compile-time slot
  offset** drawn from a per-class field table computed once when the class is
  defined.
- Field reads/writes compile to `GetField(slot)` / `SetField(slot)` — a direct
  index into the `Box<[Value]>`, no symbol lookup.
- Because fields are private and non-inherited ([Classes §2](../../spec/current/classes.md)),
  a subclass's fields occupy fresh slots and never renumber the parent's — offsets
  are permanently stable.
- An unassigned slot reads `None`; the private `Nil` sentinel ([ADR-0010](0010-tagged-value-enum.md))
  backs "not yet stored" internally and is surfaced as `None`, never leaked.
- The whole-class field collection is what makes **read-before-write a compile
  error**: a field read that appears in no assignment set is rejected.

## Consequences

- Field access is an array index, not a hash probe — the common operation on every
  instance gets materially cheaper and cache-friendlier.
- The typo class (`_naem`) becomes a compile-time error rather than a silent
  dynamic-map entry that reads absent forever.
- Stable offsets are what let a call-site/field cache and a future inline cache
  ([ADR-0012](0012-selector-signature-encoding-and-dispatch.md)) assume a fixed
  shape per class.
- Slot layout is intentionally **not** inheritance-visible; cross-hierarchy field
  access must go through accessors, matching the spec and keeping offsets static
  even under a future runtime `superclass=` ([open question Q4](../../spec/current/open-questions.md)).
- The dynamic `IndexMap<Symbol, Value>` per instance is removed.

## Alternatives considered

- **Dynamic `IndexMap<Symbol, Value>` per instance** (the current model). Flexible
  and simple, but a hash probe per access, no read-before-write detection, and it
  contradicts the spec's "layout fixed at class-definition time." Rejected.
- **Inheritance-visible shared slots** (fields laid out contiguously across the
  superclass chain). Saves accessor indirection, but reintroduces the
  fragile-base-class problem — adding a field to a base class renumbers subclass
  slots — which the private/non-inherited rule exists to avoid. Rejected.
