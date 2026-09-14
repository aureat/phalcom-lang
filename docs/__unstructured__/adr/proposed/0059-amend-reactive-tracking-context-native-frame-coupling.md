# 59. Amend ADR-0058 + ADR-0033: the reactive tracking context is bound to the native-frame switch guard

- Status: Proposed (needs user ratification)
- Date: 2026-07-14
- Amends: [ADR-0058](../accepted/0058-reactive-tracking-context-needs-a-native-module.md)
  (adds a soundness invariant its Consequences understate);
  [ADR-0033](../retired/0033-amend-fiber-execution-trampolined-block-callsite.md)
  (adds `Reactive.current` to §Decision 4's sequencing constraint)
- Related: [ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md) §4–§5
  (restricted switch model — the guard this ADR leans on);
  [`docs/spec/current/drafts/reactivity.md`](../../spec/current/drafts/reactivity.md)
  (`Reactive.trackedBy`/`untracked`, the `Computed` recompute path);
  [`docs/design-notes/reactivity-coherence-and-hazards.md`](../design-notes/reactivity-coherence-and-hazards.md)
  (finding 1, where this was surfaced)

## Context

[ADR-0058](../accepted/0058-reactive-tracking-context-needs-a-native-module.md) makes
`Reactive.current` a single **`VM`-owned** `Option<ObjRef>`, with
`trackedBy`/`untracked` as native save/set/run/restore. `Computed#recompute`
(`reactivity.md`) collects dependencies by dynamic extent: every reactive read
occurring *anywhere* during `Reactive.trackedBy(self) { _compute.call() }`
registers into `self`.

A process-wide field holding "who is currently reading" is sound only if **no
other fiber can run while it is set**. Nothing in ADR-0058 says so. Its
Consequences do raise per-fiber state, but frame it as a future
**isolation/sandboxing** nice-to-have — "not a v0.2 concern" — rather than as a
correctness precondition. That framing is what a future reader will act on, and
it is wrong: if a tracked computation could suspend, another fiber's reactive
reads would register into the **suspended** computation's dependency set. Wrong
edges, missed invalidations, spurious reruns — silent, with no diagnostic.

**It cannot happen today, and the reason is worth naming.** The VM tracks
`native_reentry_depth`, and *both* fiber-switch directions are guarded against a
native re-entrant `run_until` on the Rust stack:

- `fiber_yield` raises `CannotYieldAcrossNativeFrame` when the depth has grown
  past the fiber's recorded `floor_depth`
  ([`primitive/fiber.rs:82`](../../phalcom-core/src/primitive/fiber.rs));
- `fiber_call`/`fiber_try` → `fiber_resume` raise the *same surface class* whenever
  `native_reentry_depth != 0`
  ([`primitive/fiber.rs:96`](../../phalcom-core/src/primitive/fiber.rs)).

The second is **wider than ADR-0030 §4 requires** — §4's restriction table
forecloses only *yielding* under a native frame — and the implementation says so
explicitly, justifying the over-restriction by nested-`run_until` `base_frames`
corruption. `Reactive.trackedBy` is native (ADR-0058) and reaches its block through
the re-entrant `block_call`, so `native_reentry_depth != 0` for the whole tracked
extent, and *every* switch out of it already raises.

So the tracking context is protected — **by accident**. Two ways that protection
disappears without anyone noticing:

1. **`trackedBy` is moved onto the trampolined path.** [ADR-0033](../retired/0033-amend-fiber-execution-trampolined-block-callsite.md)
   (Deferred, not Rejected — "revisit as the general lift") makes bytecode block
   call-sites push a `CallFrame` instead of recursing, so they add no native frame.
   Its §Decision 2 retains re-entrant `block_call` for native callers, which keeps
   `trackedBy` safe — but only as long as nobody reads a blocked `Fiber.yield`
   inside a `Computed` as a bug and "fixes" it by trampolining `trackedBy`.
   ADR-0033 §Decision 4 already carries a sequencing constraint naming what it must
   not land before (ADR-0030 §5's typed switch signal); `Reactive.current` is not on
   that list.
2. **The resume over-restriction is narrowed.** `fiber.rs:96`'s comment justifies
   itself solely by `base_frames`. Someone who solves *that* problem — or who
   aligns the guard down to ADR-0030 §4's letter — removes reactivity's protection
   as a side effect, from a file that never mentions reactivity.

## Decision

**Name the invariant, and pin the two mechanisms that enforce it.**

### 1. Invariant — tracking-context integrity

> `Reactive.current` must not be observable across a fiber switch. While a tracked
> computation is running (`Reactive.current` is `Some`), no other fiber may run.

This is a **soundness** property of ADR-0058's single-`VM`-field design, not an
isolation preference. It supersedes the "not a v0.2 concern" framing in ADR-0058's
second Negative consequence.

### 2. `trackedBy`/`untracked` invoke their block through the re-entrant `block_call`

`Reactive.trackedBy` and `Reactive.untracked` MUST reach `run.call()` via the
re-entrant native path ([ADR-0033](../retired/0033-amend-fiber-execution-trampolined-block-callsite.md)
§Decision 2's retained `block_call`), never the trampolined `CallBlock` call-site.
The resulting `native_reentry_depth != 0` is what enforces §1. This is what a
straightforward implementation of ADR-0058 does anyway — this ADR records that it
is **load-bearing rather than incidental**, so it is not optimized away.

### 3. `cannot_resume_across_native_frame`'s over-restriction gains a second justification

[`primitive/fiber.rs:96`](../../phalcom-core/src/primitive/fiber.rs)'s wider-than-spec
resume guard is now load-bearing for two independent reasons: nested-`run_until`
`base_frames` integrity (its original rationale) **and** §1. Narrowing it to
ADR-0030 §4's letter requires revisiting this ADR. The rustdoc there should cite
this ADR alongside D-FIB-1.

### 4. Sequencing constraint added to ADR-0033 §Decision 4

If ADR-0033 is ever revisited, or if the full Option-B lift
([ADR-0030](../accepted/0030-fibers-and-futures-cooperative-concurrency.md) §Alternatives)
removes native frames from the block-call path generally, then **in the same unit**
the tracking context must move from a `VM` field to per-fiber state on
`FiberObject`, saved/restored at switch alongside `store_live_into`/`load_live_from`
([`primitive/fiber.rs:29`/`:49`](../../phalcom-core/src/primitive/fiber.rs)).
Not a follow-up; a precondition of that lift.

### 5. A tracked computation cannot suspend — designed, not deficient

A `Computed` or `Effect` body that yields, awaits, or resumes a fiber raises
`CannotYieldAcrossNativeFrame`. This is **correct behaviour** and must be
documented as such in `reactivity.md`, not filed as a restriction to lift.

Precedent: no signals implementation has async computeds — Solid's `createMemo`,
Vue's `computed` getter, Preact Signals' `computed`, and React's `useMemo` are all
synchronous, and asynchronously-derived state is universally a *separate* primitive
(resource/suspense), never a computed. The reason is the same one as here:
dependency collection by dynamic extent cannot survive suspension, because "the
reads that happened during this evaluation" stops being a well-defined set.
Phalcom gets that restriction enforced by the VM rather than by documentation.

## Consequences

- **Positive.** ADR-0058's single-field design stays as-is — no per-fiber
  machinery, no `FiberObject` growth, no cost — and is now *known* sound rather
  than accidentally sound.
- **Positive.** The blocked-suspension behaviour is reframed from a wart into a
  designed guarantee with precedent, so it is not "fixed" into a bug.
- **Positive.** Two edits (ADR-0033 §Decision 4's list, `fiber.rs:96`'s rustdoc)
  make the coupling discoverable from the places someone would actually be standing
  when they break it — neither of which mentions reactivity today.
- **Negative.** A future async-derived-state feature (`AsyncComputed`/resource)
  cannot reuse `Computed`'s tracking path; it needs its own design, and per §4 the
  per-fiber migration is its likely prerequisite. Named now rather than discovered
  then.
- **Negative.** Adds a cross-subsystem constraint between reactivity and the fiber
  execution model that did not previously exist on paper. It existed in the code
  either way; this makes the bill visible.

## What this precludes

Precludes implementing `trackedBy`/`untracked` on the trampolined `CallBlock` path,
and precludes narrowing `fiber.rs`'s resume guard, without superseding this ADR.

Does **not** preclude the ADR-0033 lift, Option B, or async derived state — each
stays reachable, and §4 states the price (per-fiber tracking context, in the same
unit).
