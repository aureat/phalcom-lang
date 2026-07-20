# Concurrency correctness audit — 2026-07-20

Full-stack verification of fibers / scheduling / futures / async-await at HEAD
(`501967b` + working tree). Method: three read-only single-lens auditors (switch
machinery, Future/await layer, scheduler surface) + main-thread empirical
reproduction of every claimed defect under `target/debug/phalcom`. Nothing below
is recorded on an auditor's word alone; each defect has an observed repro and a
control (`docs/errors/` method).

## Verdict

The **fiber machinery is sound** — switch state save/restore, resume guards,
cross-fiber non-local-return trapping, GC rooting of parked fibers and the ready
queue all check out. Every confirmed defect lives in the **seam between the two
execution models the runtime composes**: the Wren-style coroutine layer (dynamic
resumer chains, `try()` returns on first yield) and the added task layer (ready
queue, no fixed resumer). Six defects filed/updated in `docs/errors/`:

| ID | One line | Severity |
|----|----------|----------|
| [E002](../errors/E002-fiber-floor-upvalue-crash.md) (re-confirmed) | Fiber failure drops open upvalues → deterministic VM panic (`dispatch.rs:1094`) | blocker |
| [E007](../errors/E007-async-await-missettle.md) (new) | `Future.async { … await … }` settles `Some(None)`; real result delivered to the pump and discarded | blocker |
| [E008](../errors/E008-double-schedule-kills-run.md) (new) | Double-`System.schedule` of one fiber kills the run via the resume-refusal channel | major |
| [E009](../errors/E009-return-in-fiber-entry-deadframe.md) (new) | Explicit `return` in a fiber entry always `DeadFrameError`s | major |
| [E003](../errors/E003-schedule-pump-arity.md) (re-confirmed, sharpened) | Arity-1 scheduled entry detonates at drain, misattributed to `try`, drops the rest of the queue | major |
| [E010](../errors/E010-pump-swallows-task-errors.md) (new) | Pumps swallow captured task errors; `await`'s quiescence diagnostic masks the cause | major |

## Root-cause families (three, not six)

1. **Completion ≠ first resume-return.** `Fiber#try`/`call` return on the callee's
   first yield; the Future layer repeatedly treats that as "the callee finished"
   (E007), and `resumer` reassignment on every resume (`fiber.rs:326`) means a
   parked waiter's eventual completion is delivered to whichever fiber pumped it,
   not to the party that cares. The reactor/completion-machinery spec is aimed at
   exactly this; E007 is its natural first acceptance test.
2. **Two error channels, one `PhResult`.** A *resume refusal* (finished fiber,
   arity mismatch) and an *entry failure* ride the same `Err`, but only the second
   is covered by capture-not-propagate. E008 and E003 are both this; the fix
   boundary is `System.schedule(_)` (validate where the user's line is on the
   stack), plus making the pumps total against refusals.
3. **Failure paths skip invariants the success paths honor.** The fiber-floor
   `Err` arm discards stacks without the documented close-upvalues-first step
   (E002); the pumps discard captured errors without any observability hook
   (E010).

## Design tensions confirmed (committed design, not defects — but real costs)

- **State-dependent coloring.** `try { fut.await }` succeeds if `fut` is already
  settled and raises `CannotYieldAcrossNativeFrame` if pending (verified live).
  Any suspension under a native block re-entry (`try`/`catch`/`ensure`/`.each`)
  is illegal (ADR-0030 Option A), while `for` is compiler-inlined and
  yield-transparent (verified: `Fiber.yield` inside `for` works; inside `.each`
  refuses). The surface syntax gives no hint which constructs are which — the
  language *looks* colorless but has an invisible, dynamic color. `catch` can
  never guard a resume; `Fiber#try` is the only capture route. The deferred
  ADR-0033 CallBlock trampoline is the acknowledged real cure.

  > **Superseded same day, in part, by `5ba6101` (flat-entry block calls),**
  > which landed from a parallel session while this audit was being written.
  > Re-verified after it: `Fiber.yield` inside `.each`/`map`/`filter` now
  > **works**; `try { fiber.call() }` and `try { pending.await }` still raise
  > (`.on(_)`/`.ensure(_)` remain native re-entry). The tension survives,
  > narrowed to the error-handling surface. Details:
  > [edge matrix](2026-07-20-concurrency-edge-matrix.md).
- **`ensure` is silently skipped for abandoned fibers.** No-finalizer heap policy
  means a suspended fiber that becomes garbage never runs pending `ensure`s.
  Consistent with the heap's stance; documented nowhere in `concurrency.md`.
  (The failing fiber's own `ensure`s *do* run during unwind — verified.)
- **No external cancellation.** `Fiber::abort(_)` acts on the *current* fiber
  only (receiver ignored, root refused); no API kills a suspended/queued fiber.
  Corollary checked: an aborted fiber can never strand `Future` waiters, because
  a parked waiter can't be the aborter. Structured concurrency / cancellation
  remains the open register item.

## Verified sound (positive results, checked at HEAD)

- Switch moves `stack`/`frames`/`open_upvalues`/`checking` per-fiber as one
  atomic unit (`store_live_into`/`load_live_from`); no cross-fiber bleed.
- Resume guards: dead/running/self-resume are typed `NotAllowed` errors; no
  corruption path found.
- First-resume arity validated *before* any state mutation (regression golden
  exists for the resumer-corruption bug this once was).
- Cross-fiber `return` → `DeadFrameError` by generation check, by construction
  (the same mechanism E009 asks to *scope*, not remove).
- GC: `ready_queue` and parked fiber state are roots (forge F6 fix present);
  `Upvalue::Open` resolution is fiber-aware on both read and write.
- Future layer: settle-once holds; E004 fix (`Fiber#isRoot` predicate + bare
  yield) intact; root-await deadlock produces a diagnostic, not a spin; FIFO
  `VecDeque` queue; corpse-waiter skip in `drain()`; multi-waiter FIFO wake;
  mid-drain scheduling appends FIFO to the same drain.
- Spec self-contradiction **resolved empirically**: pending-receiver
  `then`/`map`/`catch` continuations fire during the drain (queued), *not*
  synchronously at settle — `concurrency.md:197-200` should be corrected to
  match `:228-260`. Settled-receiver continuations fire synchronously (matches
  spec; Zalgo asymmetry is real and documented).
- No auto-flattening: `await`/`Future.value` return a Future-valued payload
  as-is (JS-divergent, consistent with the pure-library-layer stance).
- E005/E006 checked for concurrency relevance: none.

## Test-coverage debt

43 concurrency fixtures + negatives are real coverage, but every new defect sat
in an untested *composition*: fail-then-call-escaped-block (E002 shape),
async-wrapping-await (E007), schedule-twice (E008), explicit-return-in-entry
(E009), task-failure-then-await-quiescence (E010). Each E-entry names the owed
fixture; land them with the fixes, negative-lane where behavior is kept.
