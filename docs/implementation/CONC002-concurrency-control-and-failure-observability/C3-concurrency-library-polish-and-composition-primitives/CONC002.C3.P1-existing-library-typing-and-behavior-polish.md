---
id: CONC002.C3.P1
program: CONC002
checkpoint: CONC002.C3
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# Existing concurrency library: typing and behavior polish

Polish the entire existing concurrency package: Fiber, Future, scheduler helpers,
Tracer, OffBehavior and Backoff. Establish truthful types, explicit internal state,
consistent callback behavior, and production Universe bootstrap. Added completion,
timer, task and channel APIs belong to [P2](CONC002.C3.P2-completion-composition-time-and-task-primitives.md).
This plan is bounded library work, not full concurrency-library implementation.

## Source baseline and authority

Inspected 2026-09-12 in the shared dirty checkout. The existing package comprises
`phalcom-core/core/universe/src/concurrency/{fiber.ph,package.ph}`. Its source
contains invariant Future<T> and typed async/map/then/catch/recoverWith/flatten,
but Fiber remains erased, Future uses string state and mixed waiter storage,
OffBehavior carries a Dynamic payload, and timed Backoff raises as unimplemented.
Tracer's sink exists independently of its unimplemented decorator consumers.

Preserve the current pending/fulfilled/rejected distinction, first-settlement
wins, scheduler-owned callback execution, ticketed parking and terminal observer.
The prior task reported focused tests passing; no runtime checks are being run
for this documentation revision. Reproduce affected evidence when implementing.

[Specification authority](../../../spec/README.md) and the
[current concurrency chapter](../../../spec/current/concurrency.md) govern public
semantics. [C1.P3](../C1-concurrency-control-and-failure-observability/CONC002.C1.P3-concurrency-architecture-and-continuation-semantics.md)
records the existing repair. [C1.P4](../C1-concurrency-control-and-failure-observability/CONC002.C1.P4-remaining-fiber-and-native-activation-work.md)
owns generic Fiber/compiler prerequisites; [C2.P1](../C2-native-suspension-and-reactor-groundwork/CONC002.C2.P1-native-suspension-and-reactor-groundwork.md)
owns native continuation migration. Consume their results rather than independently
implementing incompatible runtime models. A dependency marked incomplete remains
an explicit gate, not a reason to erase types or weaken a baseline.

## Execution order and owning files

| Unit | Implement | Owners / prerequisite |
|---|---|---|
| E0 | Freeze signatures, compatibility and examples | Universe concurrency source; source-backed generic tests |
| E1 | Canonical Future state and single settlement path | Future constructors, value, await, settleValue/settleError |
| E2 | Explicit, detachable completion registrations | Future drain/whenReady/await; existing Fiber wake capability |
| E3 | Callback and scheduler behavior consistency | Future runToTerminal/combinators; System.runScheduled |
| E4 | Finish Fiber and helper typing | C1.P4 generic Fiber gate; Tracer and OffBehavior source |
| E5 | Backoff validation, calculation and timer integration | Pure policy first; P2.N3 timer only for actual waiting |
| E6 | Package exports, examples and bootstrap closure | E0–E5; E4/E5 blocked portions recorded until complete |

E1–E3 unblock P2.N0–N1 without waiting for E4's generic Fiber or E5's timer.
P2.N3 depends only on the early E2 registration interface, so the Backoff timer
integration does not create a dependency cycle. P1 is fully complete only after
its typed Fiber and timed Backoff integration gates are complete.

## E0 — Lock down the surface before changing representation

- [ ] Record each exported selector's side, parameter/result type and behavior in
  a source-adjacent API table. Include existing aliases and native declarations;
  use the actual package import rather than assuming all definitions remain in
  one file. Keep inferred pending/error Future constructors context-sensitive.
- [ ] Extend `phalcom-semantic/tests/semantic/capabilities/generics.rs` beside
  `universe_future_preserves_payload_types_through_async_map_then_and_await`.
  Assert canonical TypeIds for payload, nested Future and Unit, not printed names.
  Add rejected incompatible setters/callbacks and invariant Future assignment.
- [ ] Preserve executable cases for Error-as-data, None-as-data and `Result<(),
  Error>` versus `Result<Unit, Error>`. The alias normalizes in semantics; do not
  add consumer-side tuple/Unit conversions or a second runtime unit object.
- [ ] Mark the residual Dynamic boundary of each public selector explicitly.
  Fix source signatures and native metadata together. Do not type arbitrary
  Function as `() -> T` without a callable-shape proof.

## E1 — One typed state and one settlement transition

Recommended internal model, expressed as design notation:

```text
FutureState<T> = Pending | Fulfilled(T) | Rejected(Error)
Future<T> { state: FutureState<T>, registrations: registration collection }
trySettle(outcome: Result<T, Error>) -> Bool
```

- [ ] Use a private tagged declaration supported by the existing ADT machinery.
  If its field typing exposes a compiler defect, isolate that defect in the
  owning semantic layer; do not use None as an uninitialized T or Dynamic casts.
  A private `Option<Result<T, Error>>` is an acceptable equivalent representation
  if it gives the same single-state invariant without unsupported declarations.
- [ ] Constructors initialize Pending and delegate to the same transition.
  `trySettle` checks Pending, installs the complete outcome, detaches the current
  registration collection, then delivers it. No user callback or suspension may
  occur between the check and state installation. A rejected transition is a no-op.
- [ ] Keep public `settleValue(T)/settleError(Error) -> Future<T>` returning self
  after either win or loss. Add internal Bool-returning settlement for P2's
  CompletionSource; do not implement a separate check-then-set policy there.
- [ ] `isReady` reads the state. `value -> Option<T>` returns Some only for success.
  Add `outcome -> Option<Result<T, Error>>` as typed inspection of the existing
  state: None pending, Some(Ok(T)) fulfilled, Some(Err(Error)) rejected. Do not
  flatten nested Option or interpret a fulfilled Error as rejection.
- [ ] `await -> T` loops on state after every authorized wake; it returns the
  fulfilled payload or raises the stored rejection. Preserve root/manual/scheduled
  authority checks and current root empty-queue diagnostics until P2.N3 lands.

Acceptance: all four duplicate-settlement pairings preserve the first outcome;
value/outcome agree for Unit, None and Error payloads; malformed public access
cannot fabricate fulfilled T; forced GC between settlement and observation is safe.

## E2 — Replace tuple-versus-closure waiter guessing

Use two internal registration variants: parked `(FiberHandle, parkGeneration)`
and ready callback `(() -> Unit)`. FiberHandle here is a design name for the
existing erased internal handle, not a new public type. Give each record an
identity and lifecycle `Registered -> Queued -> Delivered` or `Detached`.
A wake-only record may transition directly Registered -> Delivered. Settlement
reserves callback queue admission before publishing Queued. Detach is idempotent;
a queued callback checks Detached before invoking any retained user work.

- [ ] Implement registration, detach and drain as package-internal operations.
  Evolve existing public `drain` into a compatibility wrapper if its public status
  is already relied on: on Pending it must not remove waits or wake fibers; on a
  terminal Future it drains at most once. Hide the new mutating helper.
- [ ] Swap out the registration collection before walking it. Newly registered
  work observes the terminal state and cannot be lost when drain returns. Do not
  traverse a live collection while clearing it afterward.
- [ ] Return a detach capability from internal readiness subscription. After
  detachment clear callback and source captures. Prefer indexed records plus
  tombstones and amortized compaction, or an existing suitable queue; do not add
  a full scan per completion or leave captures in a never-settled Future forever.
- [ ] Check park permission before installing a waiter; obtain the existing
  generation and commit registration/park without calling user code in between.
  If park fails, detach the just-created record before propagating the error.
  A stale registration may neither consume nor replace a later park generation.
- [ ] Trace active captures through ordinary source objects. Only extend
  `heap/trace.rs` and `vm/gc.rs` if native record storage is actually introduced;
  source-level objects do not require a redundant VM-global root table.
- [ ] Isolate each delivered registration so one internal admission failure cannot
  strand the rest of the detached batch. Retain failure-reporting responsibility
  and settle the affected derived result when one exists. Inspect the actual native
  completion observer gateway before adding a callback there: observers must not
  hide new suspendable work under a retained native host frame.

Acceptance: early/late registration, detach before settlement, detach after queue
admission, self-detach, reentrant registration, pending drain misuse, stale wake,
multiple live waiters and bounded retained records after repeated detachment.

## E3 — Preserve typed composition across every scheduling path

| Existing method | Success path | Failure path |
|---|---|---|
| async<U>(() -> U) | terminal success settles U | terminal raise rejects |
| map<U>((T) -> U) | execute callback once; keep U as data | propagate source rejection; callback raise rejects |
| then<U>((T) -> Future<U>) | execute callback and await adopted Future inside observed action | source/callback/adopted rejection rejects |
| catch((Error) -> T) | successful source passes through | callback recovers T or rejects if it raises |
| recoverWith((Error) -> Future<T>) | successful source passes through | execute and await recovery inside observed action |
| flatten<U>(Future<Future<U>>) | use explicit then adoption | propagate either rejection |

- [ ] Reuse a typed action runner with its terminal observer attached before
  scheduler admission. Do not complete derived Futures after the first park.
  Keep callback invocation and adopted await inside that observed action; otherwise
  an invalid Dynamic callback result can fail in an unowned observer and leave the
  derived Future pending forever.
- [ ] Both ready and pending sources schedule matching user callbacks. The internal
  fast path may propagate an already-known outcome, but must never call user code
  inline. Make registration order deterministic without promising completion order
  for callbacks that themselves suspend.
- [ ] Keep direct self-adoption rejection for then/recoverWith. Indirect adoption
  cycles need dependency identity to diagnose reliably: record the limitation and
  a reproducer; do not claim the equality guard detects every cycle or add a whole
  graph subsystem to this polish unit.
- [ ] Tighten scheduler drain unwrapping so it does not introduce None as a fake
  Fiber. Use flow-proven extraction supported by canonical semantics. Retain FIFO,
  admission-before-execution validation, duplicate scheduling rejection, isolated
  sibling failures and root-result retention. Keep raw dequeue/wake internal.
- [ ] Fix stale comments about already-migrated native boundaries only after C2
  verifies that gateway. Do not remove native re-entry guards as a library cleanup.

Acceptance: all six methods on pending/ready sources, callback parks twice, late
failure, self-adoption, invalid Dynamic adoption, independent sibling callbacks,
no callback on a skipped branch, owned failure reported once, and GC across await.

## E4 — Finish truthful Fiber and helper types

Fiber integration consumes C1.P4's implementation: `new<R>(() -> R)` produces
`Fiber<Unit,R>`, and a proven one-argument entry can use `withInput<I,R>`.
Require a safe terminal `result -> Option<R>`; retain Dynamic for the mixed
manual yield/return result until a yield protocol is modeled. Crucially, entry
input and later yield-resume input are different contracts: do not claim the
entry's I constrains every subsequent call unless the yield site establishes it.
Track that proof boundary in C1.P4 before enabling a typed resume overload.

- [ ] Verify generic source/native identities, constructor inference, erased
  `current` and heterogeneous scheduler handles together. Preserve rest-parameter
  entry support or document a separately specified pack-entry API. An unsaturated
  bare generic type is not a valid erasure strategy.
- [ ] Type Tracer's existing enter/exit/threw arguments against its duck-typed sink
  protocol. Keep arbitrary printable values broad where intentional; do not narrow
  them merely to eliminate Dynamic. Document the elapsed unit and that the default
  sink presently ignores elapsed; preserve its output unless a contract change is
  explicitly included. Exercise sink calls directly, without implementing @traced.
- [ ] Represent OffBehavior internals as a validated tagged policy: Raise,
  Fallback(Symbol), Skip(payload). Preserve kind/payload compatibility, including
  Some(None) for skipping with None. Validate the currently public raw constructor
  rather than permitting malformed kind/payload pairs to fail later in dispatch.
  Keep a legacy Dynamic accessor if heterogeneous payloads require it; add typed
  case access only when the tag proves it. Generic skip typing requires matching
  consumer signatures, not an isolated generic decoration on the helper class.
- [ ] Keep OffBehavior.applyTo and decorator interception out of scope. Fix helper
  source references to the actual decorator design paths and describe shipped
  helpers separately from unbuilt consumers.

Acceptance: generic terminal type preserved, yielded Error versus failed Fiber,
entry arity and resume proof boundaries, all helper factories and invalid raw
constructors, Unit-returning Tracer methods, and existing decorator-independent use.

## E5 — Complete Backoff policy and its actual waiting behavior

- [ ] Validate fixed milliseconds >= 0; exponential base >= 0 and cap >= base;
  reject invalid raw kinds and negative attempt indices. Define the attempt index
  against the decorator design before publishing: recommended convention is 1 for
  the initial attempt (no delay), 2 for the first retry (base delay). Adapt callers
  and examples together if the existing consumer contract specifies another index.
- [ ] Separate pure delay calculation from waiting. For retry index k >= 0,
  exponential delay is min(cap, base * 2^k); stop doubling once capped and check
  before multiplication. Handle base/cap zero and very large k without an O(k)
  loop after saturation. Fixed and none remain constant-time.
- [ ] Until P2.N3's real timer exists, timed waitBefore must retain its honest
  unsupported error; pure validation/calculation can ship independently. After
  the gate, use `System.sleep(ms).await` and preserve the existing
  `waitBefore(Int) -> Option<Never>` result None. Unit-return migration would be a
  separate compatibility decision, not part of the `()` type alias change.
- [ ] Use the fake timer seam from P2 to verify requested delays, overflow caps,
  initial-attempt behavior and suspension. Do not insert host thread sleep or
  silently change an asynchronous timer into a synchronous System method.

## E6 — Packaging, regression placement and completion

- [ ] Retain existing package imports and exposed names. Split future/helper files
  only when it improves ownership of changed code, then update package exports,
  bootstrap ordering and the semantic test's current single-file include together.
  Prefer a shared source loader if several tests need the split package.
- [ ] Place formal type assertions in semantic capabilities/foundations, source
  behavior in `phalcom-core/tests/fixtures/language/concurrency/` with paired
  expected files, and private wake/GC transitions in the existing runtime test owner.
  Use full Universe bootstrap for source behavior. Do not create an extra Cargo
  integration target for each new fixture.
- [ ] Run the changed semantic regression and selected concurrency corpus first,
  serially; confirm nonzero test counts. Then run affected suites and canonical
  Universe bootstrap once at completion. Do not add diagnostic-baseline allowances.
  The timed/fiber gates remain open until exercised; early Future polish is a
  partial delivery, not proof that P1 is complete.
- [ ] Publish an API example for typed async/adoption, Unit, terminal Fiber access
  and timed Backoff, with explicit limitations. Review scoped whitespace and
  links. Do not run workspace/release gates unless requested or justified by an
  actual cross-package change; never claim unrun gates passed.
