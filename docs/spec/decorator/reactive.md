# Reactive decorators — `@observable`, `@computed` (and the `@effect` question)

- Status: **`@observable` design-ratified with zero open questions of its own
  (decorators-observable.md); implementation transitively blocked.** The
  blocker is not the decorator — it is the Signal/Computed/Effect runtime
  (reactivity.md, still Proposed, R-1…R-5 open) and its tracking context,
  which ADR-0058 (Accepted) rules must be a **native `System`-style module**
  (`Reactive.current`/`trackedBy`/`untracked`/`schedule`/`batch`/`flush`)
  because no `.ph`-reachable module-mutable state exists. Build order is
  therefore fixed: U-REACTIVE-NATIVE → Layout slots → these decorators.
- `@computed`'s tier correction (Install→Layout, ADR-0052) is settled; what it
  still lacks is a dedicated spec file of its own — this file is that spec's
  placeholder and records its committed constraints.

## `@observable` — verified shape (deltas from the draft: none)

Two-phase, and the phases are the point:

1. **Compile/generate** (`runtime: false`): derives a tracked getter
   (`Reactive.current.ifSome { c => c.dependOn(self) }` prologue) and a
   notifying setter with an equality bail (`_value != next` — no-op writes
   must not propagate). Ordinary generated members, same machinery as
   `@get`/`@set`.
2. **Layout** (builtin): reboxes the field's slot to hold a `Signal`. This is
   per-receiver state ⇒ ADR-0052 confinement ⇒ builtin, lowercase.

Shallow by default (tracks the reference; `_items.add(x)` does not fire) —
R-3's default, correct for v0.2; deep reactivity is a rejected default, not a
rejected feature. There is exactly **one** `@observable` in the language; the
persistence draft's dirty-tracking is a *consumer* (a `Signal`-backed column
is dirty iff its signal fired since load), never a second decorator.

Composition constraints already ruled and worth pinning where an implementer
will look:

- `@data` on a class with `@observable` fields must compare through the
  *getter* (the value), never the raw slot (a `Signal` box) — otherwise
  derived `==` breaks the moment a field reboxes.
- `@construct`/`@constructor` seed the `Signal`'s initial value — the derive
  writes through the same path the setter uses, minus notification (nothing
  can observe during construction; the freeze/A-5 argument in miniature).
- `@Traced` on a method reading observables composes as two independent
  seams (Runtime around-send vs read-tracking) — no ordering interaction,
  per D-3's chaining rule.

## `@computed` — committed constraints, spec owed

Layout tier; per-receiver `Computed` in a reserved slot (the ADR-0052 Fix 2
case study — the receiver-keyed side-table version was the leak that produced
the rule). Auto-tracking (MobX/Solid model): reads inside the computed body
register dependencies via `Reactive.current`. Its dedicated spec must decide,
against reactivity.md's R-1/R-4: recompute policy (lazy-on-read vs eager on
invalidation) and glitch discipline (batched, per R-4's scheduling). Neither
is decidable at the decorator layer — recorded as inherited, not new.

## `@effect` — evaluated, deferred with a reason

An `@effect`-decorated method ("re-run when dependencies change") is the
natural third member, and this tree deliberately does **not** propose it for
v0.2. An effect is not per-receiver derived state — it is a *subscription
with a lifetime*, and reactivity.md's R-5 (ownership/disposal tree) is
exactly the unresolved question that decides whether effects leak. A
decorator that creates undisposable subscriptions at class-definition time
would bake R-5's worst answer into the surface. Sequence: R-5 resolves →
`Effect` exists as a value with explicit disposal → *then* evaluate whether a
method-position decorator adds anything over `Effect.new { … }`. (Precedent
with consequence: Svelte 5 and Solid both keep effects as scoped runtime
constructs, not annotations, precisely because disposal is lexical there;
Phalcom has no such scope to lean on yet.)

## Performance posture

Tracked reads are the hot path of any reactive program. The generated getter
adds one `Reactive.current` probe (native module read — cheap, but not free)
per read of an observable field. Two commitments inherited from the committed
positions: the probe must be a plain send lowered against the native module
(no new opcode — ADR-0019: speed is never sufficient), and any fast-path
special-casing later must ride the IC/JIT track above the floor, guarded like
every other speculative optimization. Measure before optimizing (ADR-0051);
no number exists yet because no runtime exists yet.

## What this precludes

- A second `@observable` or an Install-tier variant — both explicitly
  foreclosed by the observable spec; restated because the persistence draft
  is where the temptation recurs.
- Deep reactivity by default.
- `@effect` before R-5. A subscription without a disposal story is a leak
  with a decorator name.
