# `@observable` — one reactive-field decorator, three sketches unified

- Status: **Accepted** (decorator design ratified 2026-07-13 under
  [ADR-0054](../../../adr/accepted/0054-two-speed-ratification-annotation-decorator-tiers.md);
  no open questions of its own). **Implementation remains blocked**, transitively,
  on [reactivity.md](reactivity.md)'s own Signal/Computed/Effect runtime, which is
  still `Status: Proposed` with its R-1–R-5 open questions unresolved — see
  `docs/forge/PLAN-DECORATORS.md` BLOCKED-ON-DECISION #1.
- Date: 2026-07-13
- Depends on:
  [attribute-classes.md](attribute-classes.md) (the `Attribute` root, `@On`,
  the generate/`finalizeLayout` hooks, the per-receiver ⇒ Layout rule) ·
  [reactivity.md](reactivity.md) (the `Signal`/`Computed`/`Effect` push-pull runtime
  — `@observable` is the thin ergonomic layer over it, per that doc's own §"ergonomic
  layer" intent) ·
  [decorators.md](decorators.md) (the tier axis; `@observable` is its named Layout
  example)
- Related:
  [ADR-0052](../../../adr/accepted/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)
  (per-receiver state is Layout — the rule that fixes `@observable`'s tier and
  corrects the earlier "Layout + Install" label) ·
  [decorators-stdlib.md](decorators-stdlib.md) (`@observable var _x` sketch —
  superseded here) ·
  [decorators-persistence.md](decorators-persistence.md) (`@observable` columns →
  dirty tracking — a **consumer** of this one decorator, not a second one) ·
  [annotations-data.md](../experimental/annotations-data.md) / annotations-construct.md
  (`@data`/`@construct` — composition rules below)

## Context

`@observable` appears in **three** places across the spec set, which the task
flagged as possibly one, two, or three distinct features. Resolving that is a
precondition for specifying it, so it is done first:

1. **[reactivity.md §"ergonomic layer"](reactivity.md)** — `@observable` makes a
   field `Signal`-backed: reads register a dependency, writes notify dependents. This
   is the canonical feature, defined as a thin layer over the `Signal`/`Computed`/
   `Effect` push-pull runtime that doc specifies in full.
2. **[decorators-stdlib.md](decorators-stdlib.md) `@observable var _x`** — reboxes the
   slot as a `Signal` (Layout) and generates a tracked getter + notifying setter. This
   is the **same feature**, rendered in the stdlib's (older, pre-A-1) surface. Not a
   second `@observable`.
3. **[decorators-persistence.md](decorators-persistence.md) "`@observable` columns →
   dirty tracking"** — an ORM `@column` that is *also* `@observable` gets dirty
   tracking "for free," because a `Signal`-backed slot records which columns changed
   since load. This is **not a third `@observable`**: it is the ORM *consuming* the
   same reactive `@observable` from (1). The persistence doc says so directly ("A
   column that is also @observable (Layout) is Signal-backed").

**Resolution: there is exactly one `@observable`** — the reactive `Signal`-backed
field. Sketch (2) is its stdlib rendering; use (3) is a downstream consumer. No
rename, no collision. This doc specifies the one decorator; [reactivity.md](reactivity.md)
remains the runtime; [decorators-persistence.md](decorators-persistence.md) remains a
consumer. The one **correction** folded in: the earlier "Layout + Install" tier label
(reactivity.md line 167, decorators-stdlib.md) is tightened to **Layout (builtin) with
a Compile/generate accessor derivation** — the setter is a *generated member*, not an
Install-tier `wrap`, and per-receiver `Signal` storage is Layout by
[ADR-0052](../../../adr/accepted/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md).

## Decision

### `@observable` — Layout (builtin) + generate-phase accessors

`@observable var _x` does two things, in two phases of the fixed pipeline:

- **generate (Compile):** derive a tracked getter and a notifying setter for the
  field — ordinary generated members, exactly like `@get`/`@set`, so `runtime: false`
  for this half (stripping it removes the two accessors, nothing else).
- **finalize (Layout):** rebox the field's slot so it holds a `Signal` rather than a
  bare value, and initialize it (`_x = Signal.new(value: <init>)`). Per-receiver `Signal`
  storage lives on the receiver's slot vector → **Layout, builtin**.

```phalcom
// BUILTIN — Layout + generate. Compiler-owned (tier: Layout is reserved, A-3).
@On(Field, tier: Layout)
class Observable extends Attribute {
  finalizeLayout(field) { field.reboxAsSignal }          // slot: _x = Signal.new(value: <init>)

  expand(field) {                                         // generate phase — tracked accessors
    return [
      Method.getter(field.baseName)   { self.slotAt(field.slot).value },        // x  => _x.value
      Method.setter(field.baseName) { v => self.slotAt(field.slot).value = v }  // x=(v) { _x.value = v }
    ]
  }
}
```

The generated `x => _x.value` is a **tracked read**: reading it inside a `Computed`
or `Effect` registers `_x` as a dependency of that computation
([reactivity.md](reactivity.md)'s `Signal>>value` does the
`Reactive.current.ifSome { c => c.dependOn(self) }`). The generated `x=(v)` is a
**notifying write**: assigning marks every dependent stale (the push half of the
push-pull graph), with the equality bail (`_value != next`) so no-op writes don't
propagate.

### How a read registers a dependency — the mechanics, end to end

`@observable` adds *no* new tracking mechanism; it routes through
[reactivity.md](reactivity.md)'s existing one:

1. An `Effect` (or `Computed`) runs its body inside `Reactive.trackedBy(self) { … }`,
   which sets `Reactive.current` to that computation for the duration.
2. Reading `account.balance` calls the generated getter `_balance.value`, which asks
   `Reactive.current.ifSome { c => c.dependOn(_balance) }` — so the running effect
   subscribes to `_balance`'s observer set. The dependency edge *is* that subscription
   ([reactivity.md](reactivity.md)'s "a signal's observer `Set` is the dependency
   edge").
3. Writing `account.balance = x` calls the generated setter `_balance.value = x`, which
   (on an actual change) marks every observer stale; effects are then scheduled, and
   pull-recompute on read yields the glitch-free diamond behavior reactivity.md
   specifies.

So `@observable` is *exactly* sugar for "store this field in a `Signal` and route its
accessors through `.value`" — nothing reactive lives in the decorator itself; it is a
Layout+generate shim onto the ratified runtime.

### `@computed` — the sibling (cross-reference, not re-specified)

`@computed` (a getter → a reactive `Computed`) is `@observable`'s read-derived sibling.
It was reclassified from Install to **Layout** by
[ADR-0052](../../../adr/accepted/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)
(the `Computed` cache is per-receiver, so it needs a reserved slot, not a receiver-keyed
side table). It is specified where ADR-0052 placed it; this doc only notes the pairing:
`@observable` for reactive *fields*, `@computed` for reactive *derived getters*, both
Layout/builtin, both thin layers over reactivity.md.

## Composition

- **`@observable` ⊗ `@data` ([annotations-data.md](../experimental/annotations-data.md)).**
  `@data` derives structural `==`/`hash`/`toString`/`with(...)` over the class's fields.
  An `@observable` field is `Signal`-backed, so `@data`'s generated `==` must compare
  the **values**, not the `Signal` boxes: the derivation reads through the generated
  tracked getter (`other.balance`, i.e. `_balance.value`), not the raw slot. This is a
  required interaction rule — `@data` over `@observable` fields compares
  `self.balance == other.balance` (unboxed), never `_balance == other._balance` (which
  would compare `Signal` identities and always differ). Because `@data` generates in
  the same generate phase and `@observable` reboxes at finalize (later), `@data`'s `==`
  emitted against the *getter* is correct regardless of the rebox. Reading an
  `@observable` field inside a derived `==` also, incidentally, registers a dependency
  if that `==` runs inside an effect — usually harmless, but note it (an effect that
  compares two `@data` values subscribes to both).
- **`@observable` ⊗ `@construct` ([annotations-construct.md](../experimental/annotations-construct.md)).**
  `@construct` binds declared fields in the generated `new(...)`. For an `@observable`
  field the binding assigns through the reboxed slot: `_balance = Signal.new(value: balance)`
  — the constructor parameter seeds the `Signal`'s initial value. `@construct`'s
  field-order-sensitive parameter list is unchanged; `@observable` only changes what the
  slot *holds*, not the constructor's shape.
- **`@observable` ⊗ persistence `@column`
  ([decorators-persistence.md](decorators-persistence.md)).** An `@column @observable`
  field is both persisted and reactive: the `Signal`'s fired-since-load state gives the
  ORM dirty tracking for free (`save` writes only changed columns) and an admin/live
  view can `subscribe`. No new mechanism — the persistence layer reads the same `Signal`
  the reactive layer writes. Whether dirty tracking is opt-in per column or implied by
  `@entity` is that doc's open question E-3, unaffected by this unification.
- **`@observable` ⊗ `@traced` ([decorators-dispatch-observability.md](decorators-dispatch-observability.md)).**
  A `@traced` (Runtime) object whose field is `@observable` composes two interception
  seams — the trace interceptor and the reactive read-tracking — on the same object.
  They do not conflict (trace wraps the *send*; tracking fires *inside* the getter), and
  Runtime hooks chain ([decorators.md D-3](decorators.md)); this is the concrete stake
  reactivity.md's tie-in raised, now resolved by the chaining rule.

## Hazards

- **Shallow by default — the classic reactive trap.** `@observable var _items` tracks
  the *reference*; `_items.add(x)` mutates in place and never fires the signal
  ([reactivity.md §Design calls](reactivity.md)). Ship shallow-by-default (predictable);
  deep reactivity is signals + a reactive-collection `Proxy`, opt-in. `@observable` does
  not silently deep-wrap.
- **Tier label correction is load-bearing.** Any doc still calling `@observable`
  "Layout + Install" is describing a superseded shape — the setter is a *generated*
  member (Compile/generate), not an Install `wrap`. An Install-tier `@observable` would
  imply a class-level attribute instance holding reactive state, which for per-receiver
  `Signal`s would be the exact ADR-0052 leak. Layout is not optional here.
- **Disposal / ownership leak.** A reactive graph leaks without an owner (effects hold
  sources, sources hold effects) — [reactivity.md](reactivity.md)'s open R-5 (a
  `Reactive.root { }` owner tree). `@observable` inherits that open question; it does not
  introduce or resolve it. An `@observable` field on a collected receiver is only
  reclaimed once its `Signal`'s observers are also unreachable — the same ownership story
  reactivity.md must settle.

## Test strategy

Golden `.ph` cases (positive stdout-exact):

- (1) Read inside an `Effect` registers a dependency; a later write reruns the effect
  once (the reactivity.md smoke test, driven through `@observable` sugar rather than raw
  `Signal`).
- (2) No-op write (`x = x`) does not rerun dependents (equality bail through the generated
  setter).
- (3) `@observable @data` — structural `==` over two instances compares unboxed values,
  not `Signal` identities (assert two structurally-equal `@observable`-field instances
  are `==`).
- (4) `@observable @construct` — the constructor seeds the `Signal`'s initial value;
  reading the field post-construct returns the seed.
- (5) Layout/leak golden — the `@observable` builtin stores its `Signal` in a reserved
  slot, never a receiver-keyed side table (the ADR-0052 snapshot assertion, extended to
  `@observable`).
- (6) Erasure — stripping the generate half removes exactly the two accessors; the Layout
  rebox is a structural strip (layout changes), matching decorators.md's Layout-strip rule.
- (7) `@column @observable` (with the persistence unit) — a changed field marks the column
  dirty; an unchanged field does not (dirty tracking rides the same `Signal`).

## What this precludes

- **A second or third `@observable`.** There is one — the reactive `Signal`-backed field.
  The persistence "dirty tracking" and stdlib "reboxed slot" are the same decorator seen
  from a consumer and from an older surface, not independent features. No future doc may
  introduce a differently-behaving `@observable` without renaming it.
- **An Install-tier `@observable`.** Foreclosed by ADR-0052: per-receiver `Signal` storage
  is Layout. The only Install-shaped piece (the accessors) is actually generate-phase
  codegen, not an Install `wrap`.
- **Deep reactivity by default.** `@observable` is shallow; deep tracking requires the
  opt-in reactive-collection proxy, not a silent behavior of the decorator.

## Open questions

`@observable` introduces no new open questions of its own; it **inherits**
[reactivity.md](reactivity.md)'s open set (R-1 three-color vs over-eager propagation, R-2
tracking-context home, R-3 shallow-vs-deep default, R-4 scheduling policy, R-5 ownership
tree) and [decorators-persistence.md](decorators-persistence.md)'s E-3 (per-column opt-in
vs `@entity`-implied dirty tracking). None of those block ratifying `@observable`'s
decorator surface — they are runtime-policy knobs on the layer beneath it, resolvable
independently.
