---
id: CONC002.C2.P2
program: CONC002
checkpoint: CONC002.C2
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
prepared: 2026-09-12
prepared_against_remote_head: 2db7e3780772e847a178918ede239d91e0bcc5fd
requires:
  - CONC002.C1.P3 COMPLETE
  - CONC002.C1.P4 COMPLETE
  - CONC002.C2.P1-R1 COMPLETE
---

# CONC002.C2.P2 — Coroutine consumer / executor separation and Fiber protocol completion

## 0. Mission, authority, and preparation boundary

This plan completes C2 after `CONC002.C2.P1-R1`.

P1-R1 makes native/control continuations VM-owned, makes `on` / `ensure` suspension-safe, routes non-local transfers explicitly, and replaces the current eager Call-linked failure cascade with failure delivery at the parent call site. P2 builds on that substrate and fixes the remaining execution-ownership defect:

> a Fiber must be able to retain its **manual coroutine consumer** while an **executor** independently parks, wakes, and resumes it.

That distinction is required for a manually consumed Fiber to perform:

```text
yield → pending await → resume by executor → yield
```

without delivering the second yield to the scheduler, losing the original caller, or treating parking as a user-visible value.

The repository is authoritative. This plan was prepared against remote `main` at:

```text
2db7e3780772e847a178918ede239d91e0bcc5fd
```

Remote inspection does not expose the developer's local working tree. Execution MUST begin by rebasing this plan onto the actual P1-R1 implementation revision and by recording local `HEAD`, branch, worktree status, and relevant scoped diff. Do not assume that the structural APIs proposed by P1-R1 landed under exactly the names used in its plan.

This plan follows `docs/agents/prompt--create-plan.md`: tasks are patch boundaries; checkpoints are semantic evidence boundaries. Tests run when they prove an integrated invariant rather than after every small edit.

### Primary completion claim

After P2:

1. manual coroutine consumption and executor driving are separate runtime authorities;
2. an active manual `call` / `try` turn survives executor parking;
3. a manually consumed Fiber may await a pending Future and later yield/return/fail to its original consumer;
4. scheduler execution no longer overwrites coroutine-consumer identity;
5. executor park is not delivered as `None` or any other guest value;
6. scheduler-owned work with no consumer still cannot user-yield;
7. public scheduler admission no longer implicitly adopts a manually yielded continuation with an invented resume value;
8. nested active ancestors remain non-runnable while a parked descendant waits;
9. quiescence without external progress is reported through a catchable execution path rather than stranding a parked manual chain;
10. the runtime is ready for C3 external-readiness integration without giving a reactor a new resume protocol;
11. the final simple Fiber generic target is either implemented soundly as `Fiber<R>` or explicitly left non-generic with a verified type-system blocker; `Fiber<I,R>` is not introduced.

### Explicitly out of scope

- reactor registrations, timers, poller backends, sockets/files/process readiness — C3;
- Future library cleanup/composition beyond edits required for `await` — C4;
- cancellation, Task/TaskScope, structured child lifetime, queue-revocation policy — C5;
- channels and select — C6;
- source-level `async` coloring;
- parallel execution, shared-memory threads, atomics/locks for future concurrency;
- session types, linear types, affine typestate, or a general typed-coroutine protocol;
- general callable-pack/type-system implementation work not already available after C1.P4;
- universal resumable-native ABI.

---

# 1. Verified pre-P1 repository facts that constrain the post-P1 design

The execution agent MUST re-check these facts after P1-R1 because P1 changes dispatch/control internals. They are included here because they explain the required semantic separation.

## 1.1 Current Fiber lifecycle

At preparation time `FiberStatus` is:

```rust
New
Running
BlockedOnChild
Yielded
Parked(i64)
Queued
Done
Failed
```

`Running` is the guest Fiber whose live stacks are mirrored by `VM::{frames, stack, open_upvalues, checking}`. `BlockedOnChild`, `Yielded`, `Parked(g)`, and `Queued` are intentionally distinct.

P2 preserves those distinctions. It may introduce an additional executor-specific blocked state if the P1-R1 implementation requires a guest Fiber to wait for a synchronous scheduler-drain operation, but it MUST NOT collapse executor wait into `BlockedOnChild`, `Yielded`, or `Parked`.

## 1.2 Current `resumer` / `resume_mode` coupling

At preparation time the callee stores:

```rust
resumer: Option<ObjRef>
resume_mode: FiberResumeMode::{Call, Try, Scheduler}
```

and `fiber_resume` sets both on every resume.

This combines three logically independent questions:

```text
who receives a user-visible yield / return?
how is terminal failure delivered to that consumer?
who happened to drive this execution turn?
```

P2 MUST separate them.

The target model MUST make this statement true:

> The executor may resume a parked Fiber without becoming that Fiber's coroutine consumer and without changing Call/Try failure policy for an already-active manual call turn.

## 1.3 Current park restriction proves the conflation

At preparation time:

```text
Fiber._$preparePark
Fiber._$park
```

require:

```rust
fiber.resume_mode == FiberResumeMode::Scheduler
```

and `_$park` requires the scheduler `resumer` itself to be `BlockedOnChild`.

This is why pending `await` is intentionally rejected for a manually consumed Fiber.

P2 replaces this authority test with an executor-aware parking rule. It MUST NOT simply remove the check while leaving the old resumer-based driver intact.

## 1.4 Current user-yield restriction is correct only for consumer-less execution

At preparation time `Fiber.yield` rejects every `Scheduler`-mode Fiber because scheduler mode implies there is no coroutine consumer.

After P2, that inference is no longer valid:

```text
Fiber may have an active manual consumer
AND
be resumed after a Future wake by the executor.
```

The correct rule becomes:

```text
user yield requires an active coroutine consumer
```

not:

```text
user yield requires "not scheduler mode"
```

A fresh detached scheduled Fiber still has no consumer and therefore still MUST reject user yield.

## 1.5 Current scheduler admission has an accidental `Yielded` capability

At preparation time `VM::enqueue_unowned_fiber` accepts:

```text
New | Yielded
```

even though the ready-queue documentation says queued work has never been resumed.

Scheduling a yielded Fiber also gives the suspended `yield` expression an implicit no-argument/`None` resume value and then leaves any later user yield without a consumer.

No repository evidence found during preparation establishes this as an intentional public semantic.

P2 MUST make this behavior explicit. The default decision of this plan is:

> Public `System.schedule(_)` admits **fresh `New` Fibers** (or wraps a fresh zero-argument Function). A manually `Yielded` Fiber is not silently adopted by the scheduler. Future wake remains the separate `Parked(g) -> Queued` path.

If current P1-R1 evidence establishes a ratified use of `System.schedule(yieldedFiber)`, stop and record PLAN DRIFT before changing this rule.

## 1.6 Current scheduler pump depends on a fake coroutine relationship

At preparation time:

- `VM::run` root draining;
- `.ph` `System.runScheduled`;
- `Fiber._$resumeScheduled()`;
- `Fiber._$park()`;

use ordinary Fiber switch mechanics so scheduled work returns to a guest Fiber "driver".

That works only because scheduler driving is represented as a special resumer mode.

P2 MUST replace that representation. A scheduler/executor return boundary is not a user coroutine consumer.

## 1.7 Fiber entry and post-yield resume types are different contracts

On first resume, `fiber_resume` validates and binds the entry callable's actual parameter shape.

On a later resume, the first supplied value is written into the suspended `Fiber.yield` result slot.

Therefore:

```text
entry argument type(s)
!= necessarily
resume-after-yield value type
```

A Fiber may also have multiple yield sites with different yielded/resume value types.

`Fiber<I,R>` is rejected.

---

# 2. Target ownership model

P2 MUST establish these independent authorities.

| Authority | Meaning | Lifetime |
|---|---|---|
| execution owner | which guest Fiber currently owns VM live buffers, or that the executor currently owns the machine between guest turns | one execution turn |
| coroutine consumer | which Fiber is waiting for the next **user stop** from this Fiber, plus Call/Try delivery policy | one active manual `call` / `try` turn; survives executor parks |
| scheduler admission | whether one execution episode is reserved in `Queued` | from admission/wake until executor resumes it |
| wait registration | exact external/Future parking episode `Parked(g)` | from park commit until matching wake/invalidation |
| completion observer | durable terminal observer independent of manual consumer | until Done/Failed handoff |
| P1 control continuation | protected/cleanup/transfer state owned by the Fiber | according to control activation lifetime |

The following identities MUST NOT be reused as each other:

```text
Fiber ObjRef
park generation
P1 ControlId
P1 frame token
future C3 reactor registration token
future C5 cancellation/admission generation
```

---

# 3. Target manual-consumer law

A manual call turn begins when Fiber A executes:

```phalcom
B.call(...)
```

or:

```phalcom
B.try(...)
```

and ends only when B produces a **user-visible stop** for A:

```text
B yields
B returns
B fails and the failure is delivered according to Call/Try
```

Executor parking is NOT a user-visible stop.

Therefore, during:

```text
A calls B
B awaits pending Future
```

the valid state is conceptually:

```text
A = BlockedOnChild
B = Parked(g)
B.consumer = (A, Call|Try)
executor owns scheduling
```

When `g` wakes:

```text
B: Parked(g) -> Queued -> Running
consumer remains (A, Call|Try)
```

If B next yields:

```text
B -> Yielded
A -> Running
B.consumer is consumed/cleared
yielded value is delivered to A
```

If B parks again before yielding:

```text
B -> Parked(g2)
A remains BlockedOnChild
consumer remains active
```

If B returns:

```text
B -> Done
A -> Running
consumer is consumed/cleared
terminal result is delivered to A
```

If B fails:

- `Call` consumer: P1-R1 injects `Raise` at A's call-site continuation;
- `Try` consumer: captured `Error` is delivered as data;
- no consumer: completion observer/E010 scheduler policy applies.

---

# 4. Target executor-stop law

P2 MUST stop representing a scheduler/executor return as an ordinary Fiber result delivered through a coroutine resumer.

Post-P1 implementation MAY name the internal type differently, but the execution driver must distinguish at least:

```text
UserYield(value)
Parked(wait identity)
Returned(value)
Failed(error)
```

or an equivalent decomposition between Fiber stop and routed transfer.

Rules:

```text
UserYield
  -> requires a coroutine consumer
  -> deliver to that consumer
  -> not an executor/scheduler result

Parked
  -> preserve coroutine consumer
  -> give control to executor
  -> no guest value delivered

Returned
  -> terminalize exactly once
  -> consumer if active, otherwise completion/detached scheduler ownership

Failed
  -> P1 routed failure semantics
  -> consumer Call/Try if active, otherwise observer/E010
```

`None`, `Unit`, `Error`, or another ordinary `Value` MUST NOT encode any of these stop classes.

---

# 5. Checkpoint map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| C0 — Post-P1 rebase and red proof | 1–3 | actual P1-R1 runtime is mapped; current manual-await prohibition and scheduler-driver coupling are reproduced without stale assumptions | exact HEAD/diff; registered expected-red composition fixture; source ownership inventory | implementation assertions -> C1–C5 |
| C1 — Consumer identity independent of driver | 4–7 | Call/Try consumer persists across non-user stops; detached scheduled work has no consumer; user yield consults consumer, not executor mode | internal Fiber state tests + manual coroutine corpus | pending await -> C3 |
| C2 — VM-owned executor driving | 8–12 | queued execution runs without becoming a coroutine child of the scheduler; park returns to executor; public scheduler drain has its own continuation | scheduler unit tests, root/pump tests, admission hostile cases | manual pending await -> C3 |
| C3 — Manual pending await and yield-await-yield | 13–17 | manual consumer chain survives one or many pending waits and resumes at the original user stop destination | dedicated source corpus + quiescence/error cases | generic Fiber -> C5 |
| C4 — Nested ownership, failure, GC, traces | 18–21 | nested consumer chains, P1 cleanup/failure routing, completion observers, roots/upvalues and traces remain correct across executor parks | forced-GC integration, Call/Try failure matrix, trace tests | C3 reactor integration later |
| C5 — Fiber typing decision and implementation gate | 22–26 | `Fiber<I,R>` remains rejected; `Fiber<R>` lands only if callable-domain/existential/erasure prerequisites are actually satisfied | semantic probes/full suite; canonical Universe/bootstrap identity checks | richer typed coroutine/session protocol excluded |
| C6 — Specification closure and delivery | 27–30 | current spec/source/comments reflect consumer/executor model; old scheduler-resumer authority is absent; all evidence is accounted for | negative searches, core/corpus/semantic/broad gates | reactor/C5/C6 future programs |

---

# 6. Checkpoint C0 — Post-P1 rebase and red proof

Tasks:
- Task 1 — capture the actual P1-R1 implementation boundary;
- Task 2 — map the final post-P1 Fiber/executor control paths;
- Task 3 — register the composition fixture and prove the remaining defect.

Why:
P1-R1 is allowed to change `CallOutcome`, control stacks, Fiber resume destinations, host escape, failure delivery, GC roots, and dispatch structure. P2 must consume the implementation that actually landed rather than patching the pre-P1 source described in this plan's preparation evidence.

Entry:
- C1.P3 COMPLETE;
- C1.P4 COMPLETE;
- C2.P1-R1 COMPLETE with its state/evidence ledger;
- no active P1 incident.

Primary inspect set:
- `phalcom-core/src/heap/fiber.rs`;
- `phalcom-core/src/primitive/fiber.rs`;
- `phalcom-core/src/vm/{mod,dispatch,control,send}.rs` as they exist post-P1;
- `phalcom-core/src/primitive/system.rs`;
- `phalcom-core/core/universe/src/concurrency/fiber.ph`;
- `docs/spec/current/concurrency.md`;
- P1-R1 implementation-state file;
- concurrency language corpus registrations and fixtures.

### Task 1 — Freeze actual execution baseline

EXACT:
```bash
git rev-parse HEAD
git branch --show-current
git status --short
git log -12 --oneline --decorate
```

Record:
- P1-R1 implementation commit(s);
- any uncommitted changes overlapping P2;
- names and locations of final P1 stop/control/delivery types;
- whether `resumer`, `resume_mode`, `switch_pending`, `resume_slot`, and `_$resumeScheduled` still exist unchanged.

Do not mechanically implement the structural names in this document when P1 already replaced them with a stronger equivalent.

### Task 2 — Post-P1 ownership inventory

Search:

```bash
rg -n 'resumer|resume_mode|Scheduler|BlockedOnChild|Parked|Queued|resumeScheduled|runScheduled|nextScheduled|wake_parked_fiber|switch_to_fiber' phalcom-core/src phalcom-core/core/universe/src/concurrency/fiber.ph
```

Build a table:

| Symbol/path | current authority | P2 target |
|---|---|---|
| manual `call` / `try` | ? | establish consumer |
| user `yield` | ? | consume consumer |
| park | ? | preserve consumer, return to executor |
| queued resume | ? | executor resume, preserve consumer |
| normal terminal return | ? | consumer or detached terminal owner |
| failure | ? | P1 Call/Try/observer/E010 routing |
| root scheduler drain | ? | VM-owned executor drive |
| `System.runScheduled` | ? | executor-drive continuation |
| root Future.await | ? | consume executor-drive result |
| wake | ? | unchanged enqueue-only authority |

PLAN DRIFT if P1 has already implemented durable consumer/executor separation; in that case reduce P2 to missing composition/type work instead of adding duplicate authority.

### Task 3 — Registered expected-red composition fixture

Add a dedicated selectable lane or fixture according to the post-P1 harness layout.

Required seed behavior:

```phalcom
const gate = Future.new()

const f = Fiber.new(|| {
  const resume = Fiber.yield("first")
  Assert.equal(resume, "resume-1")

  const awaited = gate.await
  Assert.equal(awaited, 41)

  const resume2 = Fiber.yield("second")
  Assert.equal(resume2, "resume-2")

  return 99
})

Assert.equal(f.call(), "first")

// Resume the manual turn. It should park at await rather than complete.
System.schedule(|| {
  gate.settleValue(41)
})

Assert.equal(f.call("resume-1"), "second")
Assert.equal(f.call("resume-2"), 99)
```

Adapt orchestration to the real post-P1 scheduler pump. The semantic requirement is fixed:

```text
the call turn beginning with call("resume-1")
does not return because of the executor park;
it returns only when the same Fiber later produces "second".
```

At C0 the case is expected to fail under the current manual-pending-await restriction. Confirm the failure is the ownership restriction, not native-frame refusal, parse/type failure, or unrelated scheduler breakage.

C0 complete when:
- exact post-P1 owners recorded;
- one legal expected-red reproduces the remaining defect;
- no P1 regression is mistaken for P2 scope.

---

# 7. Checkpoint C1 — Consumer identity independent of driver

Tasks:
- Task 4 — replace dynamic resumer-mode conflation with explicit consumer state;
- Task 5 — migrate manual call/try admission;
- Task 6 — route user yield and terminal stops through the consumer;
- Task 7 — trace and validate consumer state.

## 7.1 Required representation

STRUCTURAL target:

```rust
struct FiberConsumer {
    fiber: ObjRef,
    mode: FiberConsumerMode,
}

enum FiberConsumerMode {
    Call,
    Try,
}
```

Use repository naming/types appropriate after P1.

A Fiber stores:

```text
consumer: Option<FiberConsumer>
```

for the currently active manual call turn.

The executor is NOT represented by a `FiberConsumerMode::Scheduler`.

If P1's final transfer destination already embeds Call/Try ownership in another exact type, extend/reuse that authoritative abstraction instead of introducing a competing struct. The semantic law is what matters.

### Task 4 — Separate consumer from execution driver

Current -> target:

```text
resumer + {Call,Try,Scheduler}
```

becomes conceptually:

```text
consumer? + {Call,Try}
executor drive = separate VM authority
```

Edit owners:
- `heap/fiber.rs`;
- P1 control/delivery types if they carry resume target;
- `heap/trace.rs`;
- VM invariant checks.

Requirements:
1. consumer is GC-traced;
2. consumer survives `Parked(g) -> Queued -> Running`;
3. consumer is not overwritten by executor resume;
4. consumer is cleared exactly when a user stop is delivered to it or the relationship is otherwise terminally invalidated;
5. completion observer remains independent;
6. root never acquires a consumer merely because the executor drives it.

Do not keep a second `resumer` field with overlapping meaning unless it is temporary inside one checkpoint and deleted before C1 completes.

### Task 5 — Manual call/try owns consumer creation

Manual `call`/`try` may resume only a manual-resumable Fiber (`New` / `Yielded` after the P2 scheduler-admission correction).

Before mutation:
- validate state;
- validate first-entry arity/domain exactly as current code does;
- validate native/P1 control restrictions;
- ensure there is no already-active consumer.

Then:
1. park caller as `BlockedOnChild`;
2. store caller live state;
3. install `(caller, Call|Try)` as callee consumer;
4. start/restore callee;
5. do not install any scheduler/executor identity into consumer.

A failed preflight MUST leave caller and callee unchanged.

### Task 6 — User yield and terminal delivery use consumer presence

`Fiber.yield` target law:

```text
consumer Some(c) -> legal user yield to c
consumer None    -> NotAllowed("fiber has no coroutine consumer")
```

Do not branch on whether the current turn was executor-resumed.

On yield:
- save current Fiber continuation;
- set `Yielded`;
- TAKE/CLEAR active consumer;
- restore consumer Fiber;
- deliver yielded value into its P1 resume destination;
- caller becomes Running.

On terminal return:
- mark callee Done once;
- completion observer semantics remain exactly P1/C1;
- if consumer exists, take it and deliver result;
- if consumer absent, hand terminal state to executor/completion ownership; no fake guest return.

On terminal failure:
- use P1-R1's final parent exception delivery;
- `Call`: inject Raise at consumer call site;
- `Try`: deliver captured Error;
- consumer absent: observer/E010 policy.

### Task 7 — Consumer root/lifetime/invariant tests

Add internal tests for:
- consumer survives an artificial Parked -> Queued -> Running transition;
- yield clears consumer exactly once;
- return clears consumer exactly once;
- failure under Call and Try clears/delivers correctly;
- completion observer may coexist with a manual consumer without becoming that consumer;
- consumer chain roots blocked ancestors;
- invalid manual resume of Blocked/Parked/Queued remains local and non-mutating.

Negative search at C1 end:

```bash
rg -n 'FiberResumeMode::Scheduler|resume_mode.*Scheduler' phalcom-core/src
```

Expected: zero semantic scheduler-ownership uses. If a legacy name remains only during migration, record and remove it before C2 completes.

C1 evidence:
- focused `--lib` Fiber state/invariant tests;
- current manual coroutine corpus;
- P1 Call/Try failure controls.

---

# 8. Checkpoint C2 — VM-owned executor driving

Tasks:
- Task 8 — introduce truthful executor/guest ownership and stop handling;
- Task 9 — make park return to executor, not consumer;
- Task 10 — resume queued work without installing a consumer;
- Task 11 — migrate root and `System.runScheduled` pumps;
- Task 12 — narrow public scheduler admission to fresh work.

This is the highest-risk P2 checkpoint.

## 8.1 Executor ownership invariant

The VM needs a truthful representation for:

```text
guest Fiber is executing
vs
executor owns control between guest turns
```

A Fiber may be Parked while its consumer is BlockedOnChild; in that state no guest Fiber is necessarily able to act as a scheduler driver.

Do NOT solve this by temporarily marking the blocked consumer Running.

Preferred architecture:

```text
ExecutionOwner::Guest(ObjRef)
ExecutionOwner::Executor
```

or an equivalent representation established by P1's driver.

If the simplest post-P1 implementation is to make `VM.current` optional during executor ownership, that is acceptable but high-fanout. If P1 already has an explicit top-level typed stop/driver, extend it instead.

Rejected shortcut:
- leave `VM.current` pointing at a Parked/Blocked Fiber and continue calling guest-sensitive helpers as if it were Running without an explicit executor mode.

Required invariant:

```text
Guest(f):
  f.status == Running
  VM live guest buffers belong to f

Executor:
  no guest bytecode/native primitive is executing
  guest-sensitive current-Fiber access is invalid
  parked/blocked Fiber buffers remain in their Fiber objects
```

### Task 8 — Add executor stop/drive boundary

Consume P1's actual typed dispatch/control result.

The driver MUST be able to observe:

```text
guest user-yielded
guest parked
guest terminal success
guest terminal failure
```

without decoding a `Value`.

The executor handles `Parked` and consumer-less terminal stops. User yield is routed to the consumer before the executor makes another scheduling decision.

Do not add a second Future-specific driver.

### Task 9 — Park without switching to the coroutine consumer

Change `_$preparePark` / `_$park` or their post-P1 equivalents:

Preflight:
- current guest Fiber;
- non-root under current C2 policy;
- Running;
- native/control switch-safe;
- checked park generation;
- executor capable of receiving a park stop.

Remove:
- requirement that the Fiber was "resumed in Scheduler mode";
- requirement that a scheduler resumer Fiber is `BlockedOnChild`.

Commit:
1. preserve active consumer;
2. record exact `Parked(g)`;
3. move live guest state into Fiber;
4. publish a Parked stop to executor;
5. do NOT write `None` into any consumer call site;
6. do NOT consume the consumer.

Waiter registration ordering remains:

```text
prepare/validate generation
register exact (Fiber,g)
commit park
```

A refusal before park MUST leave no actionable waiter.

### Task 10 — Resume queued Fiber as executor work

Introduce/rework one VM-owned resume seam:

```text
resume_queued(fiber)
```

Semantics:
- requires `Queued`;
- restores Fiber buffers;
- preserves existing consumer unchanged;
- starts fresh entry only when `started == false`;
- for woken parked Fiber, restores exact parked continuation without inventing a user resume argument;
- sets guest Running/current ownership;
- does not create a `BlockedOnChild` scheduler Fiber;
- does not set Call/Try consumer policy.

This reveals an important difference:

```text
manual resume of Yielded
  -> supplies next Fiber.yield result value

executor resume of Parked
  -> supplies no user value; it continues after executor park/await
```

These MUST be separate code paths even if they share buffer-transfer helpers.

### Task 11 — Replace guest-driven scheduler pumping

At preparation time both root draining and `.ph` `System.runScheduled` depend on `_$nextScheduled` + `_$resumeScheduled()` guest switching.

P2 must replace that execution ownership.

Preferred surface:

```text
VM executor:
  dequeue valid Queued
  resume_queued
  execute until next stop
  repeat as required
```

Public `System.runScheduled()` remains synchronous from Phalcom's perspective but its continuation is not the coroutine consumer of each scheduled Fiber.

Use P1's VM-owned control continuation mechanism for the caller waiting on a scheduler drain, or another explicit executor-drive continuation. If a new Fiber status is needed for a guest waiting for an executor drain, use a truthful status such as `BlockedOnExecutor`; do not reuse `BlockedOnChild`.

The executor-drive continuation must:
- remember the calling guest continuation;
- drain current ready work according to the existing synchronous API;
- tolerate tasks parking;
- resume the caller when the requested drain boundary is reached;
- never receive user-yield data from scheduled tasks;
- preserve E010 reporting at the same safe boundary.

Root `VM::run` should use the same queue-driving core without needing a guest driver Fiber.

Root Future.await may keep a root-special loop in C2, but it must consume the new executor-drive seam rather than raw guest scheduler resumption.

### Task 12 — Make public `System.schedule` fresh-work admission

Change public admission:

```text
System.schedule(existing Fiber):
  FiberStatus::New -> Queued
  FiberStatus::Yielded -> reject
```

Functions are still wrapped as fresh Fibers, subject to current zero-entry-argument scheduler validation.

Internal Future wake remains:

```text
Parked(g) --matching wake--> Queued
```

and therefore is unaffected.

Reason:
A yielded manual coroutine has no active call turn. Silently scheduling it would invent a resume value for the suspended `yield` expression and would convert a manual protocol into detached execution without an explicit ownership transfer.

Add regression:

```text
f yields
System.schedule(f) -> NotAllowed
f.call(explicitValue) -> still resumes normally
```

If repository evidence after P1 shows an intentional scheduler-adoption API, stop and ratify its explicit resume-value/consumer semantics instead of silently retaining the current behavior.

C2 evidence:
- FIFO and duplicate admission;
- malformed fresh-entry arity;
- queued manual-resume rejection;
- wake does not run code inline;
- detached scheduled user yield still rejects;
- yielded public schedule rejects without corrupting later manual resume;
- root result remains preserved across host executor drain.

---

# 9. Checkpoint C3 — Manual pending await and yield-await-yield

Tasks:
- Task 13 — allow pending await for a manually consumed non-root Fiber;
- Task 14 — prove consumer survival through one/many waits;
- Task 15 — implement no-progress/quiescence failure for the active manual chain;
- Task 16 — verify root-await compatibility;
- Task 17 — retire obsolete restriction fixtures/comments.

## 9.1 Future.await change

Current Future source uses the same ticketed wait representation. Preserve it.

For non-root Fiber:

```text
while Future pending:
  generation = preparePark()
  add (current, generation)
  park(generation)
  on executor resume, recheck Future state
```

This algorithm does not need to know whether the Fiber has a consumer.

The runtime park path now owns that distinction.

Do not:
- turn park into `Fiber.yield(None)`;
- deliver any synthetic resume value to the suspended await;
- let `System.schedule` manually steal a parked Fiber;
- change `Future<T>` value/error/adoption semantics.

### Task 13 — Remove manual-owner rejection

Delete the C1 rule that pending await requires scheduler resume mode.

Preserve:
- root special handling for now;
- native/P1 control safety;
- exact generation;
- stale wake no-op;
- readiness recheck after every wake.

Add a positive fixture where the same active manual `call` turn crosses a pending await.

### Task 14 — Full user-stop composition matrix

Required cases:

1. `yield -> await pending -> yield -> return`;
2. await pending before first user yield;
3. two sequential pending awaits between yields;
4. already-ready await between yields;
5. rejected Future after a prior yield;
6. returned `Error` and `None` after await as data;
7. `try` consumer with failure after await;
8. nested `A.call(B)` / `B.call(C)` with C parking;
9. awaited callback performs P1 suspendable `ensure` cleanup before yielding to consumer;
10. Future wake occurs before the executor next selects the Fiber — exact queued ownership remains once-only.

For every case assert:
- scheduler never receives the user's yielded payload;
- original consumer gets the next user stop;
- active ancestors remain BlockedOnChild;
- only actual Parked leaf is wakeable/runnable.

## 9.2 Executor quiescence without reactor

C3 will later add external registrations. C2 has no such progress source.

If the executor has:

```text
no runnable work
and
the root program is blocked through a manual consumer chain whose leaf is Parked
and
no C3 external registration facility exists
```

the program must not spin or strand the host.

The failure should be catchable at the waiting computation, allowing P1 `ensure`/`on` cleanup.

### Task 15 — Quiescence error injection

Track/classify the parked leaf that blocks the root's active manual consumer chain.

A parked Fiber is root-blocking when following its consumer links reaches the root Fiber.

When the executor reaches no-runnable quiescence and such a leaf exists:

1. create the same class/category of no-progress error used by current root pending await, or centralize the diagnostic;
2. invalidate that exact parked episode so its retained Future waiter becomes stale;
3. restore/resume the parked Fiber under executor control with a pending `Raise` at the await continuation, using P1's typed failure delivery;
4. let ordinary P1 cleanup/handler machinery run;
5. if caught, execution may continue and the old waiter remains harmless;
6. if uncaught, normal Call/Try/root failure semantics apply.

Do NOT:
- return an Error directly from the consumer's `call` without resuming the parked child;
- leave the child `Parked(g)` after reporting the error;
- remove arbitrary waiters by scanning Future private state;
- add a reactor registration count in C2.

Hostile case:
after quiescence error is caught, settle the old Future later; stale waiter wake MUST return false and MUST NOT revive the Fiber.

If P1's final executor architecture offers a different compositional no-progress injection seam, reuse it.

### Task 16 — Root-await compatibility

Root remains a special non-parked waiter in C2 unless the post-P1/P2 executor model can unify it without introducing C3 reactor semantics.

Requirements:
- ready root Future returns immediately;
- pending root Future drives ready work;
- queue exhaustion while still pending remains a catchable no-progress error;
- E010 failures created during the drive are included according to current failure-cursor semantics;
- no guest scheduler resumer is required.

Do not broaden P2 solely to park the root if that would require redesigning every `VM.current` consumer. C3 may later integrate root idle with external registrations.

### Task 17 — Replace old negative expectation

Update canonical spec and fixtures that say manual pending await is intentionally rejected.

Retain negative coverage for:
- manually calling/trying a Fiber while it is `Parked(g)`;
- public scheduling of a parked Fiber;
- stale/duplicate wake;
- scheduled user yield without a consumer;
- residual native boundary restrictions not removed by P1.

C3 complete only when the original expected-red yield-await-yield case is PASS.

---

# 10. Checkpoint C4 — Nested ownership, failure, GC, and traces

Tasks:
- Task 18 — prove nested consumer chains across executor parks;
- Task 19 — verify P1 failure/cleanup semantics after executor resume;
- Task 20 — trace/root every live ownership edge;
- Task 21 — verify completion observers and detached failure ownership.

### Task 18 — Nested active-chain exclusion

Required structure:

```text
root calls A
A calls B
B calls C
C awaits pending Future
```

Expected during wait:

```text
root BlockedOnChild
A    BlockedOnChild
B    BlockedOnChild
C    Parked(g)
```

Matching wake:

```text
C -> Queued
```

No ancestor is queued.

After C returns:
- B resumes;
- then ordinary program logic determines whether B returns/yields/calls again;
- no scheduler shortcut jumps directly to A/root.

Add wrong-authority assertions for every ancestor.

### Task 19 — Failure and cleanup after executor parks

Because P1-R1 owns call-site exception injection, P2 must verify it survives executor-driven resumption.

Cases:
- C parks, resumes, fails; B `on` catches at `C.call`;
- B has `ensure` whose cleanup itself awaits;
- `Call` consumer receives Raise, `Try` consumer receives Error data;
- terminal observer on C fires exactly once after C cleanup;
- B can catch and then user-yield to A;
- ordinary returned Error from C is not failure.

No linked eager cascade may reappear.

### Task 20 — GC and upvalue ownership

Consumer links are real heap roots.

Extend:
- `Object::Fiber` tracing;
- running/parked control root visitors;
- VM root enumeration where P1 makes execution owner explicit.

Forced-GC case:

```text
root -> A -> B -> C
C parked
P1 ensure/control records hold values
A/B/C each hold unique captured objects
GC occurs while unrelated scheduler work runs
Future wakes C
C/B/A resume in order
captured/upvalue values remain correct
```

Also prove release:
after the call turn completes and consumer links clear, otherwise-unreachable Fibers/objects are collectible.

### Task 21 — Observers and E010

Matrix:

| consumer | completion observer | terminal failure owner |
|---|---|---|
| Call | absent | Raise into consumer |
| Try | absent | Error delivered to consumer |
| none | present | completion observer/Future |
| none | absent, scheduler-owned | E010 unhandled scheduler failure |
| Call/Try | present | manual consumer delivery AND observer terminal notification, each exactly once |

The last row is legal internal state unless P1 explicitly forbids attaching an observer to a manually consumed Fiber. If forbidden, make that invariant explicit and test local refusal.

Tracing:
executor wake/resume events MUST NOT rewrite the logical coroutine parent/consumer chain to say "scheduler called this Fiber". FiberBoundary traceback records should reflect user/coroutine crossings established by P1.

C4 evidence:
- memory GC;
- observability traceback/fiber trace;
- concurrency native child/protection lanes from P1;
- dedicated P2 manual-await lane.

---

# 11. Checkpoint C5 — Fiber typing decision and implementation gate

Tasks:
- Task 22 — re-inventory post-P2 Fiber type consumers;
- Task 23 — test callable-domain return inference without narrowing entry shapes;
- Task 24 — resolve `Fiber.current` and `System.schedule` erasure/existential surface;
- Task 25 — implement `Fiber<R>` only if all gates pass;
- Task 26 — otherwise record a verified type-system blocker without delaying runtime completion.

## 11.1 Ratified typing requirements

Reject:

```text
Fiber<I,R>
```

because first-entry input and post-yield resume input are different state-dependent operations.

Reject as a claim of precise arbitrary coroutine typing:

```text
Fiber<Y,S,R>
```

because one Fiber may have multiple yield sites with different `(Y,S)` pairs and order matters.

Do not introduce typestate `Fiber<State>` without an alias/linearity system capable of updating every alias.

Preferred simple nominal type:

```text
Fiber<R>
```

where:

```text
R = successful terminal entry-function result only
```

This does NOT change:

```text
call(...) -> Dynamic
try(...)  -> Dynamic
Fiber.yield(...) -> Dynamic
```

because manual calls may observe yielded data or terminal data, and `try` may also observe terminal Error-as-control data.

A future richer typed coroutine/session protocol is outside CONC002.C2.

### Task 22 — Re-run bare/generic Fiber inventory after runtime edits

Search canonical source/native/semantic products for:

```bash
rg -n '\bFiber\b|Fiber<' phalcom-core/core/universe/src phalcom-semantic phalcom-native-meta phalcom-native-surface-gen phalcom-native-macros docs/spec/current
```

Classify:
- `Fiber.new` constructor inference;
- `Fiber.current`;
- `System.schedule`;
- internal wake/park selectors;
- `Future.runToTerminal`;
- `Option<Fiber>` / waiter tuples if source still exposes them;
- reflection/type identity;
- internal native erased handles.

Do not replace every bare occurrence with `Fiber<Dynamic>`.

### Task 23 — Callable-domain inference gate

Canonical callable-domain design says:

```phalcom
(***P,) -> R
```

is the generic complete-domain forwarding form.

But the current collections gap-analysis still lists generic pack retention/substitution questions.

Therefore test, do not assume.

Required semantic probes:

1. can source syntax form a method generic over `P: Tuple` and `R` whose parameter is `(***P,) -> R`?
2. can inference infer `P` and `R` from:
   - `() -> Int`;
   - `(Int,) -> String`;
   - multiple positional args;
   - labeled args;
   - rest/complete argument-pack entry;
3. can a generic native class constructor use this without forcing explicit type application?
4. does applying `Fiber<R>` retain one runtime Fiber class identity rather than allocate a runtime class per R?

Do NOT substitute:

```phalcom
(...) -> R
```

for existential "some domain returning R".

By specification `(...) -> R` means a callable accepting ANY well-formed pack; it would narrow ordinary exact-domain entry Functions rather than erase their domain.

### Task 24 — Current/executor/schedule type surface gate

`Fiber.current` conceptually returns:

```text
exists R. Fiber<R>
```

A heterogeneous scheduler contains Fibers of many `R`.

`Fiber<Dynamic>` is not automatically a sound existential/wildcard.

Determine which already-existing Phalcom type mechanism can represent:
- arbitrary current Fiber;
- internal heterogeneous Fiber handles;
- `System.schedule(_)`.

`System.schedule` needs special attention because today it accepts either a Function or an existing Fiber through `Object`.

Candidate final contracts may include:
- a truthful erased/bare native Fiber capability already supported by the type system;
- an existing wildcard/existential applied type;
- retaining a Dynamic/Object boundary for `System.schedule` while preserving `Fiber<R>` on constructed values, only if the resulting return type is truthful;
- narrowing schedule to existing Fiber only is an API change and requires explicit ratification, not an implementation convenience.

No fake covariance to `Fiber<Dynamic>`.

### Task 25 — Conditional `Fiber<R>` implementation

Implement only if ALL are proven:

- generic callable pack inference is available for existing Fiber entry shapes;
- no current entry shape is narrowed;
- `Fiber.current` has a truthful type;
- `System.schedule` has a truthful type;
- canonical source/native bootstrap accepts all applied/bare forms without unsaturated generic diagnostics;
- reflection identity remains canonical;
- full semantic suite passes;
- no new general type-system architecture is required.

Expected shape, only if supported:

```phalcom
@native
class Fiber<R> is Object {
  @class @native new<P: Tuple>(_ body: (***P,) -> R) -> Fiber<R>

  @native call() -> Dynamic
  @native call(_ value: Dynamic) -> Dynamic

  @native try() -> Dynamic
  @native try(_ value: Dynamic) -> Dynamic

  @class @native yield() -> Dynamic
  @class @native yield(_ value: Dynamic) -> Dynamic

  @native result -> Option<R>
  @native error -> Option<Error>
  ...
}
```

Exact class-side generic syntax MUST follow what semantic parser/inference actually supports; the sketch is structural.

Add `result -> Option<R>` only when generic `R` is real:
- Some only for Done;
- None for New/Running/Blocked/Yielded/Parked/Queued/Failed;
- `error` remains Some only for Failed.

Internal `_$terminalValue` may remain for the Future bridge if it avoids extra Option allocation.

### Task 26 — Allowed conservative outcome

If any gate above fails because Phalcom lacks a required generic-pack/existential/erasure capability:

- leave public `Fiber` non-generic;
- keep `call`/`try` Dynamic;
- do not add fake `Fiber<Dynamic>`;
- record exact blocker and owning type-system program;
- mark P2 runtime semantics COMPLETE independently.

This is not a P2 failure.

P2's semantic requirement is:

> Do not publish an unsound Fiber generic surface.

C5 evidence:
- focused semantic probes;
- canonical Universe semantic bootstrap;
- full `phalcom-semantic` suite if source Fiber declaration changes;
- reflection/native surface contracts if generic Fiber lands.

---

# 12. Checkpoint C6 — Specification closure and delivery

Tasks:
- Task 27 — update canonical concurrency specification;
- Task 28 — delete old scheduler-resumer authority and stale restrictions;
- Task 29 — run focused and broad compatibility gates;
- Task 30 — close C2 and hand executor readiness to C3.

### Task 27 — Canonical specification update

Update `docs/spec/current/concurrency.md`.

Required final statements:
- Fiber consumer is one active manual call turn, not scheduler ownership;
- executor parking preserves an active consumer;
- manual pending await is supported;
- yield-await-yield returns the next user yield to the original consumer;
- only the parked leaf is externally runnable;
- fresh detached scheduled work has no coroutine consumer and cannot user-yield;
- public `System.schedule` does not silently adopt a yielded manual Fiber unless implementation chose and specified an explicit adoption protocol;
- scheduler wake makes runnable and does not execute;
- root await remains special until C3 external readiness if that is still true;
- reactor/external registrations remain C3;
- cancellation remains C5;
- channels/select remain C6;
- Fiber generic surface reflects the C5 gate's actual result, not the desired target.

Update ADR wording only where current accepted behavior has genuinely changed; preserve historical decision context.

### Task 28 — Deletion / authority searches

Run and classify:

```bash
rg -n 'FiberResumeMode::Scheduler|resume_mode.*Scheduler' phalcom-core/src
rg -n 'resumer' phalcom-core/src/heap/fiber.rs phalcom-core/src/primitive/fiber.rs phalcom-core/src/vm
rg -n '_\$resumeScheduled|nextScheduled|runScheduled' phalcom-core/src phalcom-core/core/universe/src/concurrency/fiber.ph
rg -n 'park requires scheduler ownership|scheduler resumer' phalcom-core/src docs/spec/current/concurrency.md
rg -n 'manual.*await.*reject|manual.*pending.*await' phalcom-core docs/spec/current/concurrency.md
rg -n 'Fiber<I, *R>|Fiber<I,R>' docs/implementation/CONC002*
```

Expected:
- no scheduler-as-consumer semantic path;
- no park authority based on Scheduler resume mode;
- no stale spec claim that manual pending await is forbidden;
- no normative `Fiber<I,R>` target;
- any retained scheduler helper has executor-only semantics and does not create a consumer.

Also inspect `ready_queue` rustdoc: it must no longer contradict actual admission states.

### Task 29 — Final verification schedule

Run serially with repository-standard cleared flags.

Focused first:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib <p2_executor_and_consumer_tests> -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus <p2_manual_await_lane> -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency_negative -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core memory_gc
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core observability_traceback
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core observability_fiber_trace
```

If Fiber source typing changes:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic <fiber_generic_probe_tests> -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core native_surface_contracts
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core reflection_conformance
```

Then delivery:

```bash
cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
```

Run workspace tests at final delivery if appropriate:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets
```

If the known independent REPL export baseline still reproduces unchanged, classify it as BASELINE rather than expanding P2.

### Task 30 — C2 -> C3 handoff

C2 completion report MUST state exactly:

```text
- what makes a Fiber a coroutine consumer-owned computation;
- what makes it executor-runnable;
- what a park episode identity is;
- how executor wake differs from execution;
- how manual consumer chains survive executor parks;
- how no-progress is handled before a reactor exists;
- whether root can park or remains special;
- whether Fiber<R> landed or why it remains blocked;
- which VM method/control C3 must call when an external registration completes.
```

C3 MUST be able to integrate external readiness by replacing/augmenting the executor's "no runnable work" decision, not by inventing a second Fiber resume path.

---

# 13. Required hostile regression matrix

| Case | Required observation |
|---|---|
| manual `yield -> await pending -> yield -> return` | second yield reaches original manual consumer |
| manual pending await before first yield | active `call` remains pending until user stop/terminal |
| multiple waits between yields | consumer survives every park generation |
| already-ready await | no ownership mutation beyond ordinary expression continuation |
| nested root -> A -> B -> C, C parks | only C is wakeable/queued |
| manual call on Parked Fiber | rejected, no state mutation |
| manual try on Queued Fiber | rejected |
| public schedule on Parked Fiber | rejected |
| public schedule on Yielded Fiber | rejected under default P2 decision; later manual resume remains valid |
| stale waiter after new park generation | cannot advance new continuation |
| stale waiter after quiescence error | cannot revive failed/resumed Fiber |
| scheduled detached Fiber user-yields | rejected because no consumer |
| manually consumed Fiber executor-resumes then user-yields | legal; value goes to consumer |
| failure after await under Call | P1 Raise injection at caller call site |
| failure after await under Try | Error data delivered to caller |
| returned/yielded Error | remains success data |
| P1 ensure cleanup awaits after child failure | terminal publication waits for cleanup |
| completion observer + manual consumer | exact once semantics; no consumer replacement |
| GC while nested chain parked | consumer/controls/upvalues remain valid |
| GC after turn completes | cleared consumer no longer retains chain |
| root pending await no progress | same catchable deadlock semantics |
| executor wake only | no user code before explicit executor drive |
| malformed fresh scheduler entry | rejected before queue ownership |
| root result + detached drain | root program result remains authoritative |
| trace after wake | executor not shown as logical coroutine parent |
| first-entry rest/labeled shape | P2 runtime changes do not narrow Fiber entry |
| Fiber generic probe | entry domain and terminal result inferred only if canonical pack machinery supports it |

---

# 14. Performance and representation requirements

P2 is an ownership rewrite, not a reason to add heavy scheduling infrastructure.

Requirements:
- no HashMap lookup on every Fiber switch;
- no per-switch heap continuation allocation when a small inline/optional consumer record suffices;
- no atomics or locks;
- queue wake/admission remains O(1);
- consumer delivery remains O(1);
- nested active chain is represented by existing per-Fiber links, not global scans;
- quiescence root-chain classification may traverse the active consumer chain, whose depth is bounded by call depth; do not scan the whole heap;
- generic Fiber type arguments, if implemented, do not create separate runtime Fiber classes or runtime stack representations;
- no performance claim without measurement.

If P1's `ControlStack` can host synchronous `System.runScheduled` driver continuation without enlarging every call frame, prefer that to a separate heap scheduler object.

---

# 15. Tempting wrong fixes — forbidden

1. Do not change manual pending await to `Fiber.yield(None)`.
2. Do not resume the manual consumer merely because the child parked.
3. Do not overwrite the active consumer with the scheduler/root driver after wake.
4. Do not keep `FiberResumeMode::Scheduler` as the authority for user-yield legality.
5. Do not treat scheduler driver as a `Call` or `Try` coroutine consumer.
6. Do not make every blocked ancestor `Queued`.
7. Do not add a public raw Fiber wake/resume API.
8. Do not wake and immediately execute from Future settlement.
9. Do not turn quiescence into a spin loop.
10. Do not report quiescence by returning an Error from the parent's `call` while leaving the child parked.
11. Do not let `System.schedule` invent a `None` resume value for a yielded manual Fiber.
12. Do not implement `Fiber<I,R>`.
13. Do not implement `Fiber<Y,S,R>` and claim precise ordered coroutine typing.
14. Do not use `(...) -> R` as existential domain erasure for `Fiber.new`.
15. Do not assert `Fiber<R> <: Fiber<Dynamic>` unless the type system explicitly provides that relation.
16. Do not make C2 depend on reactor registration or cancellation machinery.
17. Do not solve scheduler driving with recursive host `run_until` beneath a non-representable Rust frame.
18. Do not regress P1 suspendable `on`/`ensure`, parent failure injection, GC roots, or host-floor safeguards.

---

# 16. Incident protocol

On required-evidence failure:

1. record exact command and selected test count;
2. record current Fiber status/consumer/wait/queue state;
3. identify whether failure is in consumer delivery, executor driving, wait authority, P1 transfer routing, fixture orchestration, semantic typing, or known baseline;
4. find one nearby passing comparator;
5. classify as PRODUCT, FIXTURE, DEPENDENCY/PUBLICATION, BACKEND/HARNESS, BASELINE, or PLAN DRIFT;
6. state narrow repair files/symbols before editing;
7. rerun the smallest proof that closes the incident.

A failure to genericize Fiber because pack/existential machinery is unavailable is not a runtime incident. Record it under the C5 type-gate decision and retain the non-generic public Fiber.

Do not:
- broaden to C3/C5/C6;
- weaken status assertions;
- use Dynamic to paper over a generic identity bug;
- reintroduce scheduler-as-resumer to make a test pass;
- mark a zero-selected Cargo filter PASS.

---

# 17. Implementation-state protocol

Create `coroutine-consumer-executor-implementation-state.md` beside the plan when execution begins.

Use:

```md
# CONC002.C2.P2 implementation state

Baseline:
P1-R1 implementation revision:
Current revision:

## Established invariants

## Consumer/executor model
| Relation | Owner | Creation | Retention | Consumption |

## Fiber state transitions
| From | Event/authority | To | Consumer effect | Executor effect |

## Scheduler/pump migration
| Old path | New owner | Removed authority | Evidence |

## Fiber typing gate
- terminal-only target:
- callable-pack inference:
- current/existential representation:
- schedule representation:
- implemented/deferred decision:

## Evidence ledger
| Checkpoint | Command | Result | Proves | Revision |

## Deferred gates

## Active incident
None.

## C3 readiness handoff

## Next resume action
```

Record claims/evidence/decisions, not hidden scratch reasoning.

---

# 18. Checkpoint completion summary

| Gate | Contract | Status at planning time |
|---|---|---|
| C0 | post-P1 implementation mapped and remaining defect proven | NOT_STARTED |
| C1 | consumer identity separated from executor driver | NOT_STARTED |
| C2 | VM executor drives queued/parked work without becoming consumer | NOT_STARTED |
| C3 | manual pending await + yield-await-yield works compositionally | NOT_STARTED |
| C4 | nested failure/GC/trace/observer semantics correct | NOT_STARTED |
| C5 | truthful Fiber type surface implemented or soundly deferred | NOT_STARTED |
| C6 | canonical docs/deletions/delivery gates complete | NOT_STARTED |

No checkpoint becomes COMPLETE without its required evidence.

---

# 19. Final release-complete criteria

P2 is complete only when:

- P1-R1 is complete and its invariants remain green;
- manual consumer and executor driver are independent runtime authorities;
- executor parking preserves the active manual consumer;
- wake/resume does not overwrite that consumer;
- `yield -> pending await -> yield` works and routes the second yield to the original consumer;
- nested blocked ancestors never become independently runnable;
- Call/Try failure after executor parking obeys P1 semantics;
- consumer, controls, upvalues, observer state and wait tickets survive forced GC;
- consumer links are released after the active manual turn completes;
- scheduler-owned work with no consumer still cannot user-yield;
- public scheduler admission no longer silently adopts `Yielded` Fibers unless a separately ratified explicit adoption protocol replaced this plan's default;
- no runnable work with a root-blocking manual wait produces a catchable no-progress error rather than spinning/stranding;
- stale waiters cannot revive a Fiber after quiescence error or a later park generation;
- root scheduler/result behavior remains correct;
- no scheduler-resumer authority remains disguised under old names;
- canonical concurrency spec reflects actual behavior;
- `Fiber<I,R>` is absent from normative plans/source;
- `Fiber<R>` either passes every type gate or remains explicitly deferred with a verified blocker;
- no C3 reactor, C5 cancellation, or C6 channel/select machinery was introduced;
- focused and required broad gates are complete or accurately classified;
- no active P2 incident remains.

The next runtime implementation owner after P2 is C3 reactor/external completion execution.
