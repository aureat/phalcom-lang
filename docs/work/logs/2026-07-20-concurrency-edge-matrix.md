# Concurrency edge-case matrix — 2026-07-20

Empirical probe sweep at HEAD, second lens of the concurrency audit (first:
[correctness](2026-07-20-concurrency-correctness-audit.md)). Two probe agents
(Future/scheduler semantics; GC ⊗ fibers) plus main-thread re-verification.
Every fact below was observed under `target/debug/phalcom`, not argued.

**Mid-audit ground shift:** `5ba6101` (flat-entry for block calls) landed from a
parallel session *while this audit ran*. All facts below were re-verified at or
re-stated against that commit. Headline change: `Fiber.yield` inside
`each`/`map`/`filter` **now works** — the combinator wall is gone. Still native
(re-entrant, switch-refusing): `.on(_)` handlers, `.ensure(_)`, `invokeOn`,
`BoundMethod#call`, `@invariant` wrapping — so `try { fiber.call() }` and
`try { pendingFuture.await }` still raise `CannotYieldAcrossNativeFrame`, and the
state-dependent-coloring tension in the correctness report survives, narrowed to
the error-handling surface.

## Verified behavioral facts (Future/scheduler)

| # | Fact |
|---|---|
| 1 | Multi-waiter settle wakes all waiters in FIFO registration order. |
| 2 | A fiber may `.await` the same settled future repeatedly; each read returns the value, no corruption. |
| 3 | `await` on a pre-rejected future re-raises inside the awaiting fiber; locally catchable. |
| 4 | **Pending**-receiver `then`/`map`/`catch` continuations run during the drain (queued), **not** synchronously at settle. Resolves the `concurrency.md:197-200` vs `:228-260` self-contradiction in favor of the queued reading — spec §197-200 needs the correction. |
| 5 | Rejection propagates through a `then` chain: the derived future settles rejected and is independently catchable. |
| 6 | **Settled**-receiver `then`/`map`/`catch` fire synchronously, including nested cascades (Zalgo asymmetry is real: same combinator, two firing disciplines, chosen by settlement state — matches spec, worth a DX warning box). |
| 7 | Mid-drain `System.schedule` appends FIFO to the live drain; scheduled-by-scheduled runs in the same `runScheduled()` call. |
| 8 | `call(v)`/`try(v)` interchangeable per resume; first binds the entry param, later ones deliver at the yield site. |
| 9 | `Fiber.yield()` bare ≡ `Fiber.yield(None)`. |
| 10 | Root-await's pump drains through nested scheduling chains correctly. |
| 11 | No auto-flattening: awaiting a Future-valued future returns the inner **Future object** (JS-divergent, by absence of any flattening code — worth one spec sentence to make it a decision rather than an accident). |

## Verified sound (GC ⊗ fibers)

- `Upvalue::Open { fiber, slot }` traces its owning fiber (`heap/trace.rs:154`);
  `Object::Fiber` traces parked `stack`/`frames`/`open_upvalues`/`resumer`/
  `result`/`entry`/`checking` (`trace.rs:162-183`). A parked fiber reachable
  *only* through an escaped upvalue cell survives GC with correct values
  (probed: 3× `System.gc`, owning local out of scope, read returned `12345`).
- Cross-fiber upvalue writes are coherent both directions (root writes a parked
  fiber's captured `let` through the open cell; fiber sees it on resume).
- Settle → `System.gc` → drain: waiter resumes with the correct value.
- 100-deep nested fiber `call()` chain with innermost raise: clean cascade to a
  single traceback, no panic, no overflow.
- `Done` fiber's `result` held live across GC.

## Edge gaps still open (no fixture pins them)

- Double-schedule (E008), explicit-`return` entry (E009), async⊗await (E007),
  fail-then-call-escaped-block (E002) — repro'd, unfixtured; owed with fixes.
- The `CannotYieldAcrossNativeFrame` message text still says
  "(e.g. inside .each { })" (`primitive/fiber.rs:83,97`) — stale as of
  `5ba6101`, since `.each` is now precisely the place this *cannot* happen.
  The example should name a still-true site (`.on(_)` handler / `ensure`).
- Combinator yield-transparency now *differs* from `.on`/`ensure` opacity —
  neither spec §6's restriction table nor ADR-0030 §4 reflects the flat-entry
  split yet; ADR-0033's Deferred status is also half-overtaken (its trampoline
  goal shipped for the bytecode-call path under a different mechanism).
