# 17. Class-side stored static fields live on the metaclass instance

- Status: Accepted
- Date: 2026-07-11
- Related: [ADR-0011](0011-static-instance-slot-layout.md); [ADR-0002](0002-metaclass-tower-parallel-rule.md); [ADR-0007](0007-option-as-abstract-with-some-none.md); [ADR-0010](0010-tagged-value-enum.md); `docs/spec/v0.2/classes.md` §2–3; `docs/forge/U7-plan.md` §3 (DEC-D)

## Context

[ADR-0011](0011-static-instance-slot-layout.md) fixed the *instance* field model: a
per-class field table computed at class-definition time assigns each `_`-prefixed
field a stable slot in the instance's `Box<[Value]>`. That ADR's word "static" means
*compile-time-fixed instance layout* — it says nothing about mutable per-class state.

U7 (DEC-D) introduces a *second, colliding* meaning of "static": a `static`-keyword
class-side stored field, e.g.

```phalcom
class Counter {
    static _count = 0
    static bump => _count = _count + 1
}
```

`static _count` is **class state**, shared across all instances and living for the
lifetime of the class object — not a per-instance field. The spec (`classes.md` §3)
today shows only *computed* static getters; stored mutable class state had **no ADR
or spec coverage**, and ADR-0011's "static" naming collides with the `static`
keyword. This ADR resolves both: it defines where class-side stored fields live and
reconciles the two meanings of "static".

The design constraint is uniformity. Phalcom's tower ([ADR-0002](0002-metaclass-tower-parallel-rule.md))
already models **a class as an instance of its metaclass**. If a class is an object,
then class-side stored state is just *instance* state — one level up the tower. We
should not invent a second storage mechanism when ADR-0011's already applies.

## Decision

Apply the ADR-0011 slot mechanism **uniformly, one level up the tower**.

- A `ClassObject` gains its own slot vector, `static_slots: Box<[Value]>`, holding
  its class-side stored fields — exactly as an `InstanceObject` holds instance fields.
- These slots are indexed by a **per-*metaclass* field table**, computed at
  class-definition time from the `static`-marked `_`-prefixed assignments in the
  class body. Instance fields index by the class's `field_slots`; static fields index
  by the *metaclass's* field table. The two collection passes are parallel and
  independent.
- `static _count` reads/writes lower to `GetField`/`SetField` slot ops against the
  **class object's** `static_slots`, **not** against `self`'s instance slots. The
  receiver of a class-side field access is the class object.
- The whole-class field-collection rule and **read-before-write compile error**
  ([ADR-0011](0011-static-instance-slot-layout.md)) carry over verbatim, keyed to the
  `static` assignment set: a `static _x` read that appears in no `static`-assignment
  anywhere in the class is a compile error.
- An unassigned static slot reads `None`, backed by the private `Nil` sentinel
  ([ADR-0010](0010-tagged-value-enum.md)) and surfaced via the same absence helper
  ([ADR-0007](0007-option-as-abstract-with-some-none.md)) — the sentinel is never
  leaked. Static slots are **not** eagerly filled with a constructed `None` object
  (that would reintroduce the bootstrap-absence cycle ADR-0007 avoids).
- **Offset stability carries up the tower.** A subclass's metaclass appends its own
  static slots after its super-metaclass's; static offsets are as permanently stable
  as instance offsets, under the same private / non-inheritance-visible discipline —
  one level up. A subclass's static fields never renumber the superclass's.

### The two meanings of "static", reconciled

| Term | Meaning | Where it lives |
| --- | --- | --- |
| ADR-0011 "static … slot layout" | Instance layout is *fixed at compile time* | `InstanceObject.slots`, indexed by the class's `field_slots` |
| `static` keyword (this ADR) | *Class-side stored state*, shared per class | `ClassObject.static_slots`, indexed by the metaclass field table |

Both are "static per-class layout computed once at definition time"; they differ only
in *which* tower level's field table indexes them. That is the whole point: one
mechanism, two levels.

## Consequences

- **One mechanism, not two.** Class-side stored state reuses ADR-0011's slot vector,
  field-table collection, read-before-write check, and `None`-default — shifted up one
  tower level. No new storage primitive, no new absence path.
- Class-side field access is an array index into `static_slots`, not a hash probe —
  same performance profile as instance fields, and equally inline-cache-friendly for a
  future class-side field cache.
- Stable static offsets survive subclassing, so a class-side field cache can assume a
  fixed shape per metaclass, mirroring ADR-0011's instance guarantee.
- **Preclusion:** freezing the static layout at definition time forecloses adding a
  static field to a *live* class, exactly as ADR-0011 forecloses reshaping instance
  layout. Acceptable and deliberate; record in `DEFERRED.md` if runtime static-field
  addition is ever wanted.
- **Dispatch impact:** none. Static *methods/getters* already flow through
  `is_static` + metaclass dispatch ([ADR-0002](0002-metaclass-tower-parallel-rule.md));
  this ADR adds only *stored* slots on the class object, changing no selector encoding
  or method lookup.
- **Hazard — identity-dispatch ⊗ optional arity:** unchanged. Static stored fields add
  no selectors, so the default-args ⊗ selector-identity trap does not apply here.

## Alternatives considered

- **A separate `IndexMap<Symbol, Value>` for class state.** Simple, but reintroduces
  the per-access hash probe ADR-0011 removed, forfeits read-before-write detection, and
  splits class-side storage from instance storage into two divergent mechanisms.
  Rejected — the tower already gives us a class-as-instance model for free.
- **Class-side fields stored on the metaclass *type* rather than the class object.**
  Would make static state shared across *all* classes of that metaclass rather than
  per-class. Wrong semantics: `static _count` must be per-class state. Rejected; the
  storage belongs on the class object (the instance), indexed by the metaclass's table
  (the type), which is precisely the instance/class relationship one level down.
- **Treat `static` as sugar for a module-level global.** Breaks encapsulation and
  subclassing (no per-subclass copy, no offset stability), and detaches class state
  from the class's lifetime. Rejected.
