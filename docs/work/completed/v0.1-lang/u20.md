# U20 — Fibers & Futures (first-cut, speculative)

> **Unit ID:** U20 (claimed 2026-07-12). U12=numeric split, U13=class-hierarchy,
> U14=destructuring, U15=modules/imports, U16=method-references, U17=Option-bootstrap
> were taken by concurrent planning agents; U20 is the first free ID after them.
> **Scope:** cooperative, single-threaded concurrency — `Fiber` (the sole primitive)
> + `Future` (library layer). **Spec:** [`docs/spec/concurrency.md`](../spec/concurrency.md).
> **Forward-compat audit:** [`docs/spec/current/core/forward-compat.md`](../spec/core/forward-compat.md)
> §7 (code-grounded — this plan is its companion; **read §7 first**).
>
> **Honesty flag — this plan is materially more speculative than the U1–U11
> spine.** *Zero* implementation exists (no `FiberObject`, no `Yield` opcode, no
> scheduler — confirmed against `bytecode.rs`/`vm.rs`). More importantly the **core
> execution-model decision is unmade** (forward-compat §7.2 / **BD-U20-EXEC**
> below): the entire shape of the central sub-units (U20.2/U20.3) branches on
> it. Sub-units are therefore **sequenced and scoped conditionally**, and each
> states how concrete vs. contingent it is. Do **not** dispatch U20.2+ until
> BD-U20-EXEC is ruled.

---

## 0. Prerequisites & gates

### Hard prerequisites (a fiber cannot be built without these)

| # | Prerequisite | Status | Why it is hard |
|---|---|---|---|
| P1 | **Execution-model decision** (BD-U20-EXEC) | **OPEN — blocks U20.2+** | Determines whether `Fiber.yield` is a top-loop pointer-swap (Option A), a full trampoline (Option B), or a native stack-switch (Option C). Governs the VM restructure. See forward-compat §7.2. |
| P2 | **First-class blocks / one `ClosureObject`** | landed (U4, ADR-0006/0013) | Both `Fiber.new` and `Future.async` take a `Function`; the shared closure repr means they don't care block-vs-method. |
| P3 | **Frame-token non-local return** | landed (U10, ADR-0013) | Fiber-local `return`/`DeadFrameError` falls out (forward-compat §7.1 D4) — *provided* `next_frame_generation` stays VM-global. |
| P4 | **Handle/arena heap + `Object` variant pattern** | landed (U1, ADR-0009/0010) | `FiberObject` is an `Object::Fiber` arena variant; the `List` variant is the template. |

### Soft prerequisite (gates the *failure* half only, not core suspend/resume)

| # | Prerequisite | Status | Gates |
|---|---|---|---|
| P5 | **Error mechanism / unified unwind** (ADR-0008) | pending (U-CORE-6) | `Fiber.abort(_)`, `try`/`error`, `failed` status, and `Future` *reject* are payloads of the one unwind. The *success* path (`new`/`call`/`yield`/`done`) can land **without** it; the failure path (U20.4) **depends** on it. |

### Genuinely-open design work — belongs in `open-questions.md`, NOT this plan

These are **not** unit sub-tasks; they are undecided design and must be resolved
(ADR or open-question ruling) before the sub-unit that needs them:

- **BD-U20-EXEC** — the execution model (Option A/B/C, §7.2). *New; not yet in
  `open-questions.md`.* **This plan proposes adding it there and drafting an ADR.**
- Structured concurrency / cancellation scopes — already OPEN (concurrency.md §3,
  overlay §OPEN).
- `Future` `select`/`race` combinators — already OPEN.
- Scheduler **fairness** guarantees & the external-completion source (timers/IO)
  shape on `System` — already OPEN; U20.5 stubs a minimal FIFO ready-queue only.
- **Per-fiber stack sizing / growth**, and (post-GC) scanning suspended fibers'
  stacks as roots (forward-compat §7.1 D1 CROWN JEWEL). Deferred until a GC exists.

---

## Sub-unit U20.0 — Execution-model ADR (design spike, no code)

- **Goal.** Resolve **BD-U20-EXEC** and record it as an ADR. Produce a written
  decision on Option A (restricted top-loop switch) vs B (full trampoline) vs C
  (stackful). This is the gate for everything else.
- **Depends on.** Nothing (pure design). Consumes forward-compat §7.2.
- **Write-set.** `docs/adr/00NN-fiber-execution-model.md` (new;
  number — 0024+ (ADR-0023 = floor amendment, ratified de03f26)), `docs/spec/open-questions.md` (add the
  concurrency execution-model entry), `docs/spec/concurrency.md` (correct §1's
  "return to the dispatch loop" sketch and the stale `Value::Fiber(PhRef<…>)`
  wording per forward-compat §7.4). **No source.**
- **Design decision.** **BLOCKED-ON-DECISION (BD-U20-EXEC).** Options and the
  audit's recommendation are in forward-compat §7.2. Summary:
  - **A (recommended by audit):** switch integrates with the **top-level**
    `run_until`; yield across a re-entrant native primitive raises
    `CannotYieldAcrossNativeFrame`. Smallest correct step; does not foreclose B;
    keeps ADR-0009 GC-readiness; the spec's canonical `while`-generator works.
  - **B:** de-recurse all callback primitives; yield anywhere. Big rewrite.
  - **C:** per-fiber native stack; yield anywhere but collides with the
    stackful-GC crown jewel. Do **not** pick without re-opening ADR-0009's GC path.
  - *The architect does not pick — the user / ADR ratifies.*
- **Risk.** Choosing C silently narrows the future GC design; choosing B blows the
  unit size up ~3×. Picking A but wording the ADR so B is foreclosed would be the
  subtle error — the ADR must state A's restriction is a *removable guard*.
- **Test strategy.** None (doc unit). Acceptance = ADR `Accepted`, open-question
  filed, concurrency.md corrected.
- **Forward-looking note.** The ADR must explicitly keep Option B reachable and
  must not commit the GC design (leave ADR-0009's "collector added later" intact).

---

## Sub-unit U20.1 — `FiberObject` substrate + kernel classes (no suspension yet)

- **Goal.** Introduce the fiber *value* and its two kernel classes, wired into the
  tower and bootstrap, with **status/reflection only** — `Fiber.new { }` allocates
  a `suspended` fiber; `isDone`/`error` read state; **no resume/yield yet.**
- **Depends on.** U20.0 (to know the field set the exec model needs; e.g. Option
  A vs C differ on whether a native-stack handle is stored). Landed P2/P4.
- **Write-set (source).**
  - `phalcom-core/src/fiber.rs` (**new**) — `FiberObject { entry: ObjRef, stack:
    Vec<Value>, frames: Vec<CallFrame>, status: FiberStatus, resumer:
    Option<ObjRef>, result: Value }`; `enum FiberStatus { Suspended, Running,
    Done, Failed }`.
  - `phalcom-core/src/heap.rs` — add `Object::Fiber(FiberObject)` variant +
    `fiber()`/`as_fiber()` accessors (mirror the `List` accessors at L294/L314).
  - `phalcom-core/src/value.rs` — **no new arm** (forward-compat §7.1 D2);
    add a `class`-resolution row so `Value::Obj`→`Object::Fiber` reports the
    `Fiber` class (mirror how `List` resolves its class).
  - `phalcom-core/src/universe.rs` — `fiber_class`/`future_class` on
    `CoreClasses`; register in `install_core`.
  - `phalcom-core/src/primitive/fiber.rs` (**new**) — floor primitives:
    `Fiber.new(_)` (class-side; wrap a `Function`), `isDone`, `error`. **No**
    `call`/`yield` yet.
  - `phalcom-core/core/core.ph` — `Fiber` + `Future` class skeletons under
    `Object` (`Future` is a plain instance class, forward-compat §7.1 D6).
- **Write-set (tests/fixtures).** `phalcom-core/tests/fixtures/…/fiber_new.ph`,
  invariant additions in `phalcom-core/tests/invariants.rs`.
- **Design decision.** `Fiber` and `Future` are ordinary heap classes (D6). Fiber
  is `Object::Fiber` + `Value::Obj` (D2). The `stack`/`frames` **live inside** the
  arena object (D1) so a future GC scans them uniformly. `FiberStatus` is a plain
  Rust enum in `fiber.rs`.
- **Risk.** `FiberObject` owning `Vec<CallFrame>` keeps `Object` non-`Copy`
  (already true — `Object` holds `Vec`s), fine; `CallFrame` is `Copy`, so
  `Vec<CallFrame>` clones cheaply — **do not** accidentally deep-copy a fiber's
  frames on any hot path. Bootstrap ordering: `Fiber`/`Future` must be installed
  after `Function`/`Block` (they reference them) and pass `verify_invariants()`.
- **Test strategy.** Golden: `Fiber.new { 1 }` allocates, `.isDone == false`,
  `.class == Fiber`. Invariant harness: `Fiber`/`Future` satisfy the parallel-rule
  (ADR-0002) and appear in the tower census. No resume behavior asserted yet.
- **Forward-looking note.** Reserve `resumer`/`result` fields now even though
  unused, so U20.3 doesn't reshape the struct. Do **not** store a native-stack
  handle unless U20.0 picked Option C. Keep the field set a superset that Option
  A and B both use.

---

## Sub-unit U20.2 — VM execution-model change (the big one; shape set by U20.0)

- **Goal.** Make the interpreter able to **switch which fiber it runs** without a
  data copy — relocate "current stack/frames" behind a `current: ObjRef` into the
  running `FiberObject`, per the U20.0 decision.
- **Depends on.** U20.0 (**hard** — this unit's whole shape is its output),
  U20.1 (needs `FiberObject`). **Serialized on the critical path; cannot
  parallelize.**
- **Write-set (source).** `phalcom-core/src/vm.rs` (the `VM` struct fields L47–54,
  `run_until` L739, `call_method` primitive arm L390–430), `phalcom-core/src/fiber.rs`.
  **This unit rewrites the loop's stack/frame access — it conflicts with any other
  unit that touches `vm.rs`; must run alone.**
- **Design decision (contingent on U20.0).**
  - **If Option A:** `VM.stack`/`VM.frames` become *accessors* onto
    `heap.fiber(current).stack/frames` (or the `current` fiber is swapped in/out
    of the existing fields at switch points). Add a `ControlFlow`-style signal so
    a fiber-switch primitive tells `run_until` "keep looping, the active fiber
    changed" — **distinct from the frame-count-shrink non-local-return signal**
    (forward-compat §7.1 D5, invariant 3). `run_until`'s floor check (L741) must
    become "the *root* fiber drained," not "frames ≤ base."
  - **If Option B:** additionally convert `block_call`/`perform`/`each`-family
    from `run_until` re-entry to frame-push + top-loop continuation.
  - **If Option C:** integrate the stack-switch crate; `current` swap saves/
    restores a native context. (Audit advises against — GC crown jewel.)
- **Risk. (Highest-risk unit in the plan.)** The `frames_before` heuristic in
  `call_method` (L401–428) currently reads *any* frame-count change as a non-local
  return; a fiber switch changes `frames.len()` too. Conflating them corrupts the
  stack. The borrow model is fragile here: relocating `stack`/`frames` behind a
  heap deref means the hot loop borrows `self.heap` while iterating — a potential
  double-borrow against primitives that also take `&mut self`. Prototype the borrow
  shape *before* committing.
- **Test strategy.** All existing goldens **must stay byte-identical** (the root
  fiber is just "the main program" — no behavior change until U20.3 exercises a
  switch). Add a Rust-level unit test that manually constructs two fibers and swaps
  `current`, asserting O(1) (no `Vec` clone) and that each fiber's `stack_offset`
  windows stay valid. Fuzz: deep pure-Phalcom recursion inside a fiber must not
  break the relocation.
- **Forward-looking note.** Keep the switch primitive's signal a typed enum, not a
  bool/length delta, so U20.4 (error unwind) and a future preemption safepoint
  can add causes without reworking the loop. Preserve VM-global
  `next_frame_generation` (D4 invariant 1).

---

## Sub-unit U20.3 — `call` / `yield` / `current` (core suspend & resume)

- **Goal.** The actual coroutine: `fiber.call`/`call(_)`, `Fiber.yield(_)`
  (class-side), `Fiber.current`, entry return → `done`. Bidirectional value
  transfer across the boundary.
- **Depends on.** U20.2 (**hard** — needs the switch mechanism).
- **Write-set (source).** `phalcom-core/src/primitive/fiber.rs`,
  `phalcom-core/src/fiber.rs`; possibly a `Bytecode::Yield` in
  `phalcom-core/src/bytecode.rs` **only if** U20.0 requires an opcode (Option A
  can likely do it as a pure primitive that returns the switch signal — prefer
  that, no bytecode change). `phalcom-core/core/core.ph` (`Fiber` method surface).
- **Design decision.** `call`/`yield` are primitives that (a) set statuses, (b)
  move the transferred value across the boundary (into the target's `result`/
  resume slot), (c) repoint `current`, (d) return the switch signal to the
  (top-level, Option A) loop (concurrency.md §1 pt 3). `yield(_)` is **class-side**
  (acts on `Fiber.current`). `resumer` forms a dynamic chain (push on `call`, pop
  on `yield`/return). **Under Option A**, `call`/`yield` guard: if the current
  fiber has any re-entrant native primitive between its entry and here, raise
  `CannotYieldAcrossNativeFrame` (detect via a per-fiber "native-depth" counter
  incremented around each `run_until` re-entry).
- **Risk.** The value-transfer slot must never leak the private `Nil` sentinel
  (Invariant 4) — first-resume with no arg and value-less `yield` surface as
  `None`, exactly like `run_until`'s drain (L745). Resumer-chain corruption
  (double-resume of a `running` fiber, resuming a `done` fiber) must be explicit
  errors, not UB. The native-depth guard (Option A) must count re-entries precisely
  or it will spuriously reject valid yields.
- **Test strategy.** Golden: the concurrency.md §1 counter (`while`-generator)
  prints `0,1,2`. Round-trip value transfer (`call(x)` becomes the value of the
  suspended `yield`). `done` after entry returns; re-`call` of a `done` fiber
  errors. **Negative:** `Fiber.new { list.each { Fiber.yield(x) } }.call` raises
  `CannotYieldAcrossNativeFrame` under Option A (this negative test *documents the
  restriction* and would flip to positive if Option B ever lands). Non-local
  `return` across a fiber boundary raises `DeadFrameError` (D4).
- **Forward-looking note.** Keep `yield`/`call` value transfer symmetric so
  `Future.await` (U20.5) can layer as "add to waiters, then `yield` to scheduler"
  with no new mechanism. Don't bake the scheduler in here — `Fiber` stays
  scheduler-free (concurrency.md §1).

---

## Sub-unit U20.4 — failure path: `abort` / `try` / `error` / `failed`

- **Goal.** Fiber failure as a captured `Error`: entry raises → `failed` status +
  `Error` in result slot; `try`/`try(_)` capture instead of propagate;
  `Fiber.abort(_)` raises out of the current fiber to its resumer; `error` returns
  the captured `Error` as `Option`.
- **Depends on.** U20.3 (core switch) **and P5 = U-CORE-6 Error mechanism**
  (ADR-0008) — **hard.** Cannot land before the unified unwind exists.
- **Write-set (source).** `phalcom-core/src/primitive/fiber.rs`,
  `phalcom-core/src/fiber.rs`, and the U-CORE-6 unwind site (shared — **sequence
  after U-CORE-6**, do not co-edit).
- **Design decision.** Fiber failure is a **payload of the one unwind** (ADR-0008;
  forward-compat §7.1 D7): the error unwinds the failed fiber's **own** frames to
  its floor, stores the `Error`, sets `failed`, resumes the resumer as if `try`
  caught it (or re-raises under `call`). The unwind must stop at the *fiber* floor,
  not the process floor.
- **Risk.** If U-CORE-6's unwind is written process-global (unwinds `self.frames`
  to 0), fiber failure will tear down the host. The forward-compat §2/§7 constraint
  ("do not special-case `throw` as host termination"; unwind to a *fiber* floor)
  must have been honored by U-CORE-6 — **verify that first.** `ensure`/`finally`
  must fire on `abort` too (ADR-0008 subtle rule).
- **Test strategy.** Golden: a fiber whose entry raises → `.isDone`, `.error`
  is `Some(err)`; `try` yields `None`/the error value; `call` re-raises.
  `abort(_)` unwinds to resumer. An `ensure` block in a fiber body fires on `abort`.
- **Forward-looking note.** Keep `error` returning `Option<Error>` (not a bare
  error/`nil`) so it composes with the U-CORE Option layer. Reserve nothing that
  blocks `Result` bridges (ADR-0008).

---

## Sub-unit U20.5 — `Future` library layer + minimal scheduler

- **Goal.** `Future` state machine over `Fiber`: `value(_)`/`error(_)`
  constructors, `async(_)`, `await`, `then`/`map`/`catch`, `isReady`/`value`, and
  a **minimal FIFO scheduler** (ready-queue) so `await` suspends to the scheduler.
- **Depends on.** U20.3 (await = "add `current` to waiters, then `Fiber.yield` to
  scheduler"); U20.4 for the *reject* path only.
- **Write-set (source).** **Almost entirely `phalcom-core/core/core.ph`** (Future
  is a plain `InstanceObject`, no new `Value` arm — forward-compat §7.1 D6 / spec
  §2). One `System` hook for external completions:
  `phalcom-core/src/primitive/system.rs` + `docs/spec/system.md`. A tiny native
  ready-queue may live on the `VM` or a `SchedulerObject` in `fiber.rs`.
- **Design decision.** `Future` owns **no new VM mechanism beyond `Fiber` + a
  queue** (concurrency.md §2). The top-level program runs inside the scheduler's
  root fiber so top-level `await` is legal. Settle-once: further completions
  ignored. `then`/`map`/`catch` and `await` bottom out in the same waiter list.
- **Risk.** Scheduler **fairness** and the external-completion (timer/IO) source
  are OPEN (concurrency.md §3) — U20.5 must ship a *deliberately minimal* FIFO
  with a documented "fairness unspecified" note, not an ad-hoc design a later
  scheduler ADR must unwind. `await` at top level requires the root fiber to *be*
  the scheduler's — verify that composes with U20.2's root-fiber definition.
- **Test strategy.** Golden: `Future.value(3).await == 3`; `Future.async {
  compute }.await`; `then` chains; `catch` recovers a rejected future; a two-fiber
  ping-pong through the scheduler. `isReady`/`value` never block.
- **Forward-looking note.** Do not implement `select`/`race` or cancellation
  (OPEN). Keep the waiter list and settle-once semantics so those combinators layer
  later. Keep the `System` completion hook narrow (an ADR-0019 floor-amendment if
  it needs a new primitive).

---

## Wave schedule

Foundational and central units are **serialized on the critical path**; only the
tail parallelizes, and even then narrowly (most sub-units touch `vm.rs`/`fiber.rs`
and cannot overlap). Write-sets in parentheses.

| Wave | Sub-units | Rationale / gating |
|---|---|---|
| **W0 (gate)** | **U20.0** (docs/ADR only) | Pure decision. **BD-U20-EXEC must be ruled by the user before W2.** Can run concurrently with unrelated non-U20 work. |
| **W1** | **U20.1** (`fiber.rs`+`heap.rs`+`value.rs`+`universe.rs`+`primitive/fiber.rs`+`core.ph`) | Substrate. Needs U20.0's field-set answer. Serial (introduces the `Object::Fiber` variant everything else builds on). |
| **W2 (critical path, alone)** | **U20.2** (`vm.rs`+`fiber.rs`) | The execution-model change. **Rewrites the dispatch loop — must run alone; conflicts with any `vm.rs` edit.** Gated on U20.0 *ruled* + U20.1 landed. |
| **W3** | **U20.3** (`primitive/fiber.rs`+`fiber.rs`+`core.ph`[+`bytecode.rs`]) | Core suspend/resume. Serial after U20.2 (shares `fiber.rs`/loop). |
| **W4** | **U20.4** (`primitive/fiber.rs`+`fiber.rs`+U-CORE-6 unwind site) | Failure path. **Gated on U-CORE-6 (Error mechanism) landing.** Sequence after both U20.3 and U-CORE-6. |
| **W5** | **U20.5** (`core.ph`+`primitive/system.rs`+`system.md`) | Future + scheduler. Mostly `.ph`; the only sub-unit that could **partially parallelize** with an unrelated non-`vm.rs`/non-`fiber.rs` unit, since its write-set is nearly disjoint. Needs U20.3 (+U20.4 for reject). |

**Critical path:** U20.0 (decision) → U20.1 → **U20.2** → U20.3 → U20.4
(waits on U-CORE-6) → U20.5. There is essentially **no intra-unit parallelism** —
every core sub-unit touches `vm.rs` or `fiber.rs`. U20 parallelizes only *against
other units* (non-concurrency work), never within itself.

---

## BLOCKED-ON-DECISION register (must reach the user before implementation)

- **BD-U20-EXEC (blocks U20.2 and everything downstream).** The fiber
  execution model: **A** restricted top-loop switch (audit recommendation) / **B**
  full callback-primitive trampoline / **C** stackful per-fiber native stack.
  Trade-offs and the recommendation are in forward-compat §7.2. The architect does
  **not** pick; this needs the user or a ratified ADR (U20.0). Until ruled,
  U20.1 (the substrate) can proceed but U20.2+ cannot be scheduled.

## New-ADR backlog

- **ADR-00NN (U20.0): Fiber execution model.** Records BD-U20-EXEC's
  resolution; must keep Option B reachable and must not commit the GC design.
- **Open-question addition:** the execution-model choice is **not** currently in
  `open-questions.md`; add it (companion to the already-listed structured-
  concurrency / `select`-`race` / scheduler-fairness open items).
