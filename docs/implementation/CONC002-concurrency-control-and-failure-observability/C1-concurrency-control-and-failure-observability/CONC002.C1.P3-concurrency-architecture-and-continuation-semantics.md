# CONC002.C1.P3 — Concurrency architecture and continuation semantics

Assessment date: 2026-09-12.

**Final decision amendment:** the user's subsequent generic-typing requirement
supersedes the initial decision to retain an unparameterized Future and shared
implicit flattening. Future<T> is now invariant; map preserves U, then adopts
Future<U>, catch recovers T, recoverWith adopts Future<T>, and flatten removes
one nested layer. This distinction is necessary for accurate nested result types.
The initial assessment below records the predecessor and rationale; this amendment
and the implementation checkpoint describe the final working tree.

Inspected baseline: `01b58ec0` on `main`.
Scope: architecture assessment followed by a bounded implementation checkpoint.
This is an implementation/design record, not a replacement specification.

## A. Current runtime model

`phalcom-core/src/primitive/fiber.rs::fiber_resume` validates admission and entry
arity before swapping stacks. The caller becomes `BlockedOnChild`; the callee
becomes `Running`. `store_live_into`/`load_live_from` transfer operand stacks,
frames, open upvalues and invariant-checking state. Frame generations remain
VM-global, preventing cross-fiber non-local returns from finding a false home.

`vm/dispatch.rs::run_until` distinguishes entry exhaustion from failure at the
fiber floor. It records `Done`/`Failed`, detaches a completion observer, and
delivers control through the dynamic resumer. Failure in `Call` mode cascades
through linked parents; `Try` and `Scheduler` capture it. Open upvalues close
before abandoned stacks are cleared.

`concurrency/fiber.ph::Future.runToTerminal` already binds a durable observer
before admission. `await` uses exact `(Fiber, generation)` registrations;
`VM::wake_parked_fiber` consumes only the matching parking episode. The root
drives the queue instead of parking. Queue admission reserves `Queued` and
`pop_next_queued` skips stale states. Detached failures now have a reporting
queue: the historical E010 observation is no longer an accurate description.

GC traces parked stacks, resumers and completion observers in `heap/trace.rs`;
the VM roots ready work and retained scheduler failures. Plain Function calls
use the flat invocation gateway. `block_on`, `block_ensure`, and the native
`whileTrue` fallback still call `block_call` recursively on the Rust stack.

## B. Confirmed architectural defects

1. **Readiness changes callback failure and scheduling semantics.** Settled
   `then`/`map`/`catch` call user code inline; pending versions use an observed
   Fiber. A settled callback can throw from registration or fail to park when
   invoked from a manual coroutine. An equivalent pending callback rejects its
   derived Future. This is deliberately tested legacy behavior, but a poor
   semantic contract; replace it.
2. **Scheduler yield has no consumer.** `fiber_yield` accepts Scheduler mode,
   delivers data to the queue driver, and leaves the action yielded with its
   completion observer still pending. No owner guarantees another admission.
   Reject this protocol mismatch before the transfer; do not lose data or
   silently leave an async result pending.
3. **Native callback suspension remains incomplete.** `on` and `ensure` keep
   pending control state in Rust locals. Removing the depth guard would lose
   those continuations. Ordinary collection callbacks already use flat calls;
   the existing specification's blanket claim that `each` cannot yield is stale.
4. **Failure documentation overstates ordinary call semantics.** The actual
   linked failure cascade discards parent activations; it does not inject a
   catchable exception at the call expression. Native catch boundaries currently
   reject the transfer before this question can be exercised normally.
5. **Manual generator await is intentionally rejected**, not silently routed
   correctly. Supporting it requires preserving both consumer and executor
   ownership, beyond the existing scheduler-only park contract.

The historical premature async settlement, unowned wake, duplicate admission,
root-identity and active-ancestor-state defects have corresponding mechanisms
and regressions on this baseline. They are not reasons to rewrite those parts.

## C. Proposed fundamental concurrency model

Retain Fiber as an execution stack and Future as a one-shot outcome cell.
An executor-driven Fiber with a completion observer supplies the internal task
role; do not add public `Task` merely to rename that pair.

The dynamic resumer owns the current coroutine transfer, the completion observer
owns terminal settlement, and the current wait registration owns wake authority.
These are distinct relations even where their storage fits in one Fiber.
User yield belongs to a coroutine consumer. Parking belongs to an executor.
Neither a returned `None` nor an `Error` object determines lifecycle.

All matching Future callbacks run as scheduler work, regardless of source
readiness. Registration returns a derived Future before callback execution.
Thrown callback errors reject it; returned Error objects fulfill it. Preserve
the existing one-layer Future adoption behavior for `then`, `map`, and `catch`.

## D. State machine

| From | Operation/authority | To |
|---|---|---|
| New or Yielded | manual call/try | Running; caller BlockedOnChild |
| New or Yielded | scheduler admission | Queued |
| Queued | private dequeue/resume | Running; driver BlockedOnChild |
| Running, manual mode | user yield | Yielded; consumer Running |
| Running, scheduler mode | validated Future registration | Parked(g) |
| Parked(g) | matching wake g | Queued |
| Running | normal entry return | Done |
| Running | uncaught failure | Failed |

Wrong-authority operations fail locally without state mutation. Terminal states
cannot resume. A stale wake returns false. No public operation re-enters an
active ancestor. A scheduled user yield becomes a diagnosable failure rather
than an unconsumed nonterminal outcome.

## E. Completion / wake architecture

Keep the GC-traced single observer and monotonic checked park generation.
Centralize all continuation invocation in `runCallbackToFuture` and
`runToTerminal`. Adoption must subscribe directly to a returned Future's state;
implementing internal forwarding through public `then`/`catch` after making
those asynchronous would recursively create unnecessary derived Futures.
Reject direct self-adoption rather than leave an impossible dependency pending.

Registration and park commit are contiguous in this single-threaded runtime:
the interval must not execute arbitrary callbacks. Future I/O wakes should enter
through executor-owned registrations, not expose a raw Fiber resume API.

## F. Scheduler model

Keep FIFO admission with per-Fiber reservation; no global identity set is needed.
Only admitted work can be privately resumed. Stale entries do not abort drains.
No preemptive fairness is promised. Root exhaustion drains current ready work
and reports unobserved failures; an unresolved Future alone does not keep the
host alive. Root await reports an empty-queue deadlock in the current system,
which has no external readiness source.

The raw-handle queue is sufficient while admission is unique and entries cannot
be revoked/replaced. Cancellation or external admissions must introduce an
activation generation so a stale queued entry cannot claim a newer activation.

## G. Failure model

Target decision: `call` should deliver failure as an exception at its call site,
letting parent cleanup and catch run. Retire linked terminal cascade when VM
handler activations land, in one checkpoint with traceback and upvalue tests.
Until then document the actual cascade honestly; do not partially simulate it.
`try` captures terminal failure as data; `error`/`isDone` distinguish it from a
successfully returned Error. A future explicit stop API can improve this dynamic
surface without inventing a misleading generic return type now.

`abort` is an ordinary raise operation, not cancellation or uncatchable kill.
It has no normal return, so its declared result should be `Never`. Cancellation
must instead be a cooperative request with cleanup before terminal publication.

## H. Native suspension model

Choose split architecture (C), with an explicit resumable ABI (B) only for native
operations whose state cannot sensibly be expressed as ordinary bytecode.
Fully transparent arbitrary Rust stack capture (A) is rejected: it requires a
different host-stack architecture and obscures GC/lifetime ownership.

| Category | Current suspension | Migration |
|---|---|---|
| Leaf primitives | return directly; no callback continuation | retain synchronous ABI |
| Ordinary Function gateway / source collection callbacks | flat VM activations | retain |
| on / ensure | recursive `block_call`; forbidden across boundary | VM handler/cleanup activations |
| Native loop fallback | recursive condition/body callbacks | VM loop activation or source method |
| Other reflection/resource callback helpers | not exhaustively audited here | classify at each owning gateway |

Proposed `NativeActivation` variants carry callable handles, phase, saved stack
and frame depths, and a pending normal/raise/non-local-return outcome. They live
in the Fiber's owned activation storage and participate in tracing. `ensure`
retains the pending outcome while cleanup runs, including across parking; cleanup
failure/return supersedes it. Handlers unwind and close upvalues before running
matching code. Tracebacks include logical activation sites, not Rust recursion.
Cancellation enters the same unwind machinery, never bypasses cleanup, and is
not translated to a normal Error value. The remaining synchronous host/FFI
regions retain explicit suspension diagnostics until migrated.

The [Lua continuation API](https://www.lua.org/manual/5.4/manual.html#4.5) provides
evidence for saving continuation state explicitly when a native call can yield.
This motivates the representation, not adoption of Lua's surface API.

## I. Generator / await composition

Target: support yield-await-yield in one manually consumed execution. A generator
needs a persistent consumer relation; when it parks, its consumer waits for the
next *user* stop, while the executor can resume the generator after readiness.
The next yield must wake/deliver to that consumer, never the queue driver.
Implement this after handler activations and explicit activation ownership.
For this checkpoint retain rejection of manual pending await and reject direct
scheduled yield. This is an explicit implementation gap, not the final language
limitation. Pumping the scheduler recursively inside every generator is rejected:
it cannot express arbitrary suspended consumer chains cleanly.

## J. Type-system implications

The shipping declaration is `class Future`, not `Future<T>`. Its payload methods
are inferred/dynamic; Fiber call/try are explicitly Dynamic. There is no justified
claim of precise generic await typing on this baseline. Keep that honest surface
for this checkpoint; correct `abort -> Never` immediately. Unit is a normal
terminal value, and Never describes absence of a normal return, not cancellation.

Target `Future<T>` after separating a writable resolver from the read capability:
public `settleValue` makes a simple covariant Future unsound. Start invariant if
the mutable API remains. Callback adoption needs a precise one-layer result rule
for both plain and Future results; do not duplicate this in compiler consumers.
Defer Fiber yield/return generics until an explicit user stop protocol distinguishes
them. No source async coloring, throws parameter, or Send/Sync marker is warranted
by the present single-thread runtime. This is a declaration-level inspection,
not a claim of a complete inference-system audit.

## K. Structured concurrency and cancellation readiness

A later scope owns child registrations and waits for terminal observations;
parent-child lifetime ownership must not reuse the resumer. Cancellation competes
with completion for the current wait registration and schedules cleanup once.
The [Python TaskGroup contract](https://docs.python.org/3/library/asyncio-task.html#task-groups)
illustrates why scope exit, child completion and cancellation cleanup must agree.
No task-group implementation belongs in this checkpoint.

The [Rust Waker contract](https://doc.rust-lang.org/std/task/struct.Waker.html)
separates readiness notification from execution and allows coalescing. Retain
that distinction without importing polling syntax or atomic overhead. Future
parallel execution must transfer wake requests into the owner executor and
define object-sharing rules before exposing cross-thread execution.

## L. Runtime representation and cost

Current small enums, ObjRef observer and i64 generation already represent the
important concepts without hash lookups on wake/resume. Public surface remains
Fiber/Future. Keep runtime-only activation/handler metadata private.

Uniform callback execution adds an action Fiber and terminal observer scheduling
to the formerly inline settled path. This is a deliberate predictable-semantics
cost; later specialized internal forwarding may avoid allocations without running
user code at registration. Await of a settled Future remains an immediate read.
Wake and admission remain O(1); drains are O(number of registrations/entries).
No performance improvement is claimed without measurement.

## M. Incremental implementation sequence

1. This checkpoint: uniform callback execution/adoption, explicit self-adoption
   failure, scheduled-yield refusal, Never declaration, canonical rule updates,
   adversarial regressions. Preserve the already-working ownership model.
2. VM-owned handler/cleanup activations, including non-local-return and GC state;
   migrate `on`/`ensure`, then deliver child failure at the call site.
3. Separate consumer and executor activation ownership; add generator await
   routing and explicit stop observation. Preserve active-chain exclusion.
4. Typed Future/resolver capability design in canonical semantics; then I/O wait
   registrations, cancellation and structured scopes as independently verified
   features. No parallel executor until heap-sharing rules are explicit.

## N. Verification plan

Run the existing concurrency corpus as baseline and after changes. Add readiness
parity probes for then/map/catch with immediate and late failures, multiple parks,
None/Error/Unit data, nested pending/rejected adoption, direct self-adoption,
non-inline execution and scheduler-yield isolation. Retain existing wake/stale
queue/upvalue/GC/root/manual coroutine tests. Run VM ownership unit tests for
changed Fiber transitions and the relevant compiler/type contract check.

Native activation, generic Future, cancellation and generator-await acceptance
tests remain future checkpoint gates. Passing this checkpoint is not release
certification or completion of the entire architecture programme.

## Implementation checkpoint

Implemented in this working tree:

- Uniform observed callback execution for settled and pending Future sources.
  then/recoverWith await the returned Future inside the observed action, so late
  adoption errors reject the derived Future; self-adoption is explicitly rejected.
- Scheduler admission checks zero-argument entry compatibility before ownership
  changes; malformed entries cannot strand themselves or stop a later drain.
- Scheduled user-yield refusal; manual None/Error yields remain data.
- Root host result retained in the GC-traced root Fiber while scheduled work
  drains. A regression first returned the last task's value (`discarded`), then
  passed with the preserved root result, including a GC inside scheduled work.
- Explicit concurrency API parameter/return signatures, including callable
  contracts, Error inputs, generic Future<T> results and `abort -> Never`.
  Async/map/then/await preserve formal payload types, including nested Futures
  and Unit; the source-backed semantic regression asserts these identities.
  Raw Fiber transfer remains Dynamic pending the explicit stop protocol. Fluent settlement now declares Future, accurately reflecting
  its result without claiming the checker's unsupported receiver-dependent Self
  proof; the general Self checker defect is not fixed here.
- Empty tuple formation canonicalizes to Unit in the semantic TypeStore. This
  covers both value typing and tuple type annotations, including nested generic
  arguments. All four concurrency diagnostics disappear from the exact Universe
  bootstrap baseline; no new diagnostics were accepted into that baseline.

The preceding architecture sections describe both the inspected predecessor and
future checkpoints. Native on/ensure migration, catch-at-call failure delivery,
generator/await composition and Fiber generics are still open. They are not represented as implemented by this checkpoint.

Final validation: the complete positive and negative concurrency corpus passed
with final generic Future source and observed-action adoption (2 corpus groups,
44.03s); this includes production canonical Universe bootstrap and the
Result<(), Error>/Ok(()) fixture. Rust formatting and scoped whitespace checks
passed. The broader semantic suite passed before generic source extension
(1141 passed, 42 existing ignores). Root-result-with-GC regression passed.
Follow-up implementation is specified in
[CONC002.C1.P4](CONC002.C1.P4-remaining-fiber-and-native-activation-work.md).

## O. Decisions made

| Question | Decision | Rationale | Future consequence |
|---|---|---|---|
| Replace Fiber/Future? | Retain; no public Task yet | existing ownership pieces work | scopes can own internal tasks |
| Completion owner? | Single durable terminal observer | survives dynamic transfers | cancellation publishes once |
| Wake authority? | exact wait generation | rejects stale episodes | executor ingress for I/O |
| Settled callbacks? | always schedule matching callbacks | readiness cannot change failure semantics | stable optimization contract |
| Adoption? | then/recoverWith/flatten adopt; map preserves data | precise nested Future types | no implicit runtime flattening |
| Scheduler yield? | reject user yield without consumer | prevents lost values and orphaned result | add separate task checkpoint API if needed |
| Call failure? | target catch-at-call after handler migration | cleanup compositionality | linked cascade is transitional |
| Abort? | raise; Never; not cancellation | matches actual control flow | separate cooperative cancellation |
| Native suspension? | VM activations plus opt-in resumable native ABI | owned state survives parking | no mandatory async syntax |
| Generator await? | support in target, implementation gap explicit | preserve consumer routing | distinct consumer/executor links |
| Generic Future? | invariant Future<T>, implemented | writable settlement consumes T | split read capability before covariance |
| Parallelism? | no new locks/atomics now | no parallel executor | owner-executor handoff required |
