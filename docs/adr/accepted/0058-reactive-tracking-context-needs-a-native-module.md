# 58. Reactive tracking-context and effect scheduler need a native module, not class-side `.ph` state

- Status: Accepted
- Date: 2026-07-13
- Related: [`docs/spec/v0.2/concurrency.md`](../../spec/v0.2/concurrency.md) §2
  (`System.schedule(_)`/`System.nextScheduled`/`System.runScheduled` — the
  precedent this ADR reuses), [`docs/spec/v0.2/drafts/reactivity.md`](../../spec/v0.2/drafts/reactivity.md)
  (the `Reactive` runtime this unblocks, R-2), [ADR-0054](0054-two-speed-ratification-annotation-decorator-tiers.md)
  (Install/Dispatch/Runtime tier, `@observable`'s eventual host)

## Context

`reactivity.md`'s `Reactive` class sketch (`## The runtime`) needs ambient,
process-wide mutable state: the currently-tracking computation (`_current`),
a batching flag (`_batching`), and a pending-effects set (`_pending`),
written via `static boot { _current = None; ... }` and read/written by every
`Signal`/`Computed`/`Effect` operation.

**No `.ph`-reachable class-side/module mutable state exists today**
(`concurrency.md:234`, discovered while landing `System.schedule` for
`Future` — the identical problem: `Future`'s ready-queue needed a global,
and no `.ph` construct could hold one). `reactivity.md`'s own R-2 asks
whether the tracking context lives in "class-side (metaclass) state" or "a
well-known global singleton" — both of its own options are actually the same
unbuilt thing wearing two names; **neither is buildable in pure `.ph` right
now**. This is not a style choice among two working options; it is a hard
prerequisite gap, discovered the same way and for the same underlying reason
as `@featureFlag`'s `Flags` registry (`decorators-dispatch-observability.md`
D-3, resolved the same day) hit it.

## Decision

**Reactive's tracking context and effect scheduler get a native module**,
reusing exactly the pattern `System.schedule`/`nextScheduled`/`runScheduled`
already established for `Future`'s ready-queue (`concurrency.md` §2,
`VM::ready_queue`): a small set of native `System`-style class-side methods
backed by real Rust state in the `VM`/`Universe`, not a `.ph` class trying to
fake module-level mutability with a workaround.

Concretely, a native counterpart to each `Reactive` operation in the sketch:

| `.ph` surface (unchanged) | Native backing (new) |
|---|---|
| `Reactive.current` | reads a `VM`-owned `Option<ObjRef>` (the current tracking computation) |
| `Reactive.trackedBy(computation, run)` | native swap-run-restore over that field (mirrors `Fiber`'s `store_live_into`/`load_live_from` swap pattern — save, set, run, restore, even across a raise) |
| `Reactive.untracked(run)` | same swap pattern with `None` |
| `Reactive.schedule(effect)` / `.batch` / `.flush` | a `VM`-owned pending-effects set + batching flag, native `add`/`drain` |

This is **not** "build general class-side static-var support in the VM" —
that is a much larger, open-ended object-model feature (arbitrary mutable
slots on any `Class`/`Metaclass`) justified here by exactly one caller. A
narrow, precedented native module is the smaller, already-proven move:
`System.schedule` shipped this exact shape for `Future` without opening
general class-side mutability, and nothing about `Reactive`'s needs is wider
than `Future`'s were.

**Scope note — this ADR authorizes the module, not its implementation.**
Landing it is a new forge unit (tentatively `U-REACTIVE-NATIVE`), a
prerequisite for `docs/forge/PLAN-DECORATORS.md`'s `R-REACTIVITY` unit,
itself a prerequisite for `D-OBSERVABLE`. Building it is out of scope for
this ADR; only the "native module, not class-side `.ph` state, not general
static-var support" design call is ratified here.

## Consequences

- **Positive.** `Reactive` gets buildable ambient state without inventing a
  second, more general VM feature (class-side mutable slots) whose scope and
  hazards (thread/fiber-safety, GC rooting for a `Class`-owned field, metaclass
  tower interaction) are unexplored and unjustified by this one caller.
- **Positive.** Directly reuses `Fiber`'s existing save/set/run/restore swap
  idiom for `trackedBy`/`untracked` (`primitive/fiber.rs`'s
  `store_live_into`/`load_live_from` shape) rather than inventing a new one.
- **Negative.** `Reactive`'s ambient state is process-wide (one `VM`), not
  per-class — fine for a singleton scheduler (matches `System.schedule`'s own
  scope), but means a future per-fiber or per-sandbox `Reactive` isolation
  story (if ever needed) is a bigger redesign, not an incremental extension.
  Not a v0.2 concern; flagged for whoever eventually asks for it.

## What this precludes

Building `Reactive` as pure `.ph` class-side state is precluded — it cannot
exist without this ADR's native module regardless. General class-side
mutable-slot support for arbitrary user classes is **not** precluded by this
decision; it is simply not what this ADR builds, and remains available as a
future, separately-justified object-model feature if a caller other than
`Reactive` ever needs it.
