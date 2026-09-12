---
id: CONC002.C3.P2
program: CONC002
checkpoint: CONC002.C3
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# Additional completion, composition, time and task primitives

Add the specific APIs below on top of [P1's existing-library polish](CONC002.C3.P1-existing-library-typing-and-behavior-polish.md).
This is a bounded feature set, not the full concurrency standard library. Source
paths and baseline are in P1; this plan is proposed and runtime-unverified. API
notation describes intended contracts, not already-supported parser syntax.

## Dependencies and work order

| Unit | Deliverable | Required gate | Primary implementation owner |
|---|---|---|---|
| N0 | CompletionSource<T> | P1.E1–E2 | Universe completion source, no extra runtime state |
| N1 | all/allSettled/race | N0; P1.E3 | Future registration and aggregate records |
| N2 | checkpoint and revocable admissions | C1.P4 admission identity; C2 safe suspension contexts | primitive/fiber.rs, heap/fiber.rs, vm admission/dispatch |
| N3 | Future-shaped sleep and timeout | P1.E2, N2; C2 readiness attachment; actual timer driver | reactor registration, VM pump, Universe System/time |
| N4 | TaskScope, Task, cancellation | N2; C2 cleanup; public cancellation design reconciliation | VM control router and Universe task records |
| N5 | Bounded channels | N2, N4 wait cancellation | Universe endpoints and shared channel state |
| N6 | mapConcurrent and retry | N4; N3/P1.E5 for timed retry | Universe worker scope and policy loop |
| N7 | Atomic select | N5; N3 for timed branches | Shared operation arbitration |

N0–N1 can ship before N2–N7. N3 and N4 can proceed independently after their
own gates. C2 provides groundwork, not an implemented reactor. Reuse an existing
reactor unit if it is being implemented elsewhere; do not duplicate it in C3.
The feature owner owns integration evidence even when another checkpoint supplies
the underlying runtime. P1.E5 owns Backoff's final integration with N3.

Follow [specification authority](../../../spec/README.md),
[concurrency](../../../spec/current/concurrency.md), and the accepted
[reactor contract](../../../spec/current/stdlib/reactor.md). The old single-plan
L4 proposal conflicted with the ruled Future-shaped timer and pending-registration
exit law; N3 corrects both. Task cancellation remains a proposed new contract and
must be reconciled with the separate Future-renunciation proposal before shipping.
No plan status implies that a design has been ratified or a gate has passed.

## N0 — CompletionSource over the polished Future

Add `CompletionSource<T>` with `future -> Future<T>`,
`tryResolve(T) -> Bool`, and `tryReject(Error) -> Bool`. Allocate one Future once;
every `future` read returns that identity. A try operation returns true only for
the winning pending-to-terminal transition. A producer owns the source and gives
consumers the Future. Initially preserve existing public `settleValue/settleError`
for compatibility and document that the authority separation is conventional
until their eventual removal. Do not silently change their duplicate-settlement
behavior or claim the wrapper already enforces exclusive producer authority.

Consume P1.E1's typed outcome inspection and internal trySettle operation.
Future stays invariant while public mutation exists. Do not duplicate state in
CompletionSource or introduce a second settlement path.

Implementation: store the one created Future in a private field; resolve/reject
construct the appropriate terminal outcome and return trySettle's Bool. Contextual
construction must infer T without relying on executable generic type forms that
C1.P4 still records as incomplete. Test that two sources cannot accidentally share
storage and that repeated future reads preserve identity.

Acceptance: conflicting producers have one winner; terminal snapshot is stable;
Future<Unit>, Future<Option<Int>>, Future<Error> and nested Future payloads retain
exact types. Wrong producer payloads and recovery callback types are rejected.

## N1 — Add eager, typed Future aggregation

| API | Completion contract |
|---|---|
| `Future.all<T>(List<Future<T>>) -> Future<List<T>>` | Register all inputs without waiting between registrations. Preserve input order; reject on the first rejection observed by the coordinator. Empty input fulfills with an empty list. |
| `Future.allSettled<T>(List<Future<T>>) -> Future<List<Result<T, Error>>>` | Wait for every input, preserve input order, and store failures as results. Empty input fulfills with an empty list. |
| `Future.race<T>(List<Future<T>>) -> Future<T>` | First observed terminal outcome wins, including rejection. Empty input returns an immediately rejected Future with a clear argument error. |

Snapshot the input collection when called. Give repeated references distinct
input indices for all/allSettled. Already-settled inputs are inspected in input
order; later outcomes use scheduler-observed completion order. There is no promise
of wall-clock simultaneity ordering. Capture every input before any coordinator
callback can run.

Do not implement all as sequential await: a later failure would be hidden behind
an earlier never-settling input. Aggregate settlement never cancels producers.
Detach unnecessary value-retaining registrations after all rejects or race wins,
but retain or transfer failure-reporting responsibility for losers explicitly.
Specify the observation policy with the existing failure-reporting owner before
shipping; detachment must not silently discard an otherwise unobserved failure.

Acceptance: out-of-order completions, duplicates, multiple failures, empty input,
already-settled ties, pending first input plus later rejection, no inline user
callbacks, and retained loser failure reporting. Include exact inferred types.

### Coordinator implementation

Allocate an aggregate record before subscriptions: output, input snapshot,
remaining count, per-index completion bits, typed result slots and detach handles.
Slots are Option<T> (or Option<Result<T,Error>> for allSettled), never placeholder
T values. Each input callback claims its index once before decrementing remaining.
On all success, construct the ordered result only after every slot is filled.
On early failure/race victory, claim the output once and release unused payload
slots. Guard initialization so an immediate terminal notification cannot inspect
an incomplete handle array. Register every input even when an early ready input
already determines the result, to establish the documented failure responsibility.

Do not build subscriptions as user map calls that allocate hidden derived Futures.
Use P1's internal registration API. Before shipping, choose a concrete loser error
sink compatible with C1's owned/detached failure reporting; retain only the minimal
record needed for error observation after dropping value captures. If current
Future failures have no unobserved-rejection reporter, record and implement that
bounded policy explicitly instead of claiming scheduler error reporting covers it.
Aggregation uses O(n) state and O(n) notifications, not an O(n) scan per completion.

## N2 — Cooperative checkpoint and revocable wait ownership

Add `System.checkpoint() -> Unit` to let scheduler-owned code voluntarily yield
its execution turn without producing a coroutine value. It reserves exactly one
new runnable admission, suspends to the scheduler, and resumes once. Keep
`Fiber.yield(value)` for manual coroutine consumers. Define root checkpoint as
driving one available runnable turn; with none it returns. Reject use from a
manual coroutine until durable consumer routing supports scheduler suspension.
Do not emulate this with allocation of throwaway Futures.

Before cancellation or select, make queued admissions revocable by activation
generation, not only object identity. A stale queue entry cannot consume a newer
admission. Keep wait registration identity distinct from Fiber identity: a wait
can have multiple competing registrations but only one winning resume. Perform
registration/park commitment without a suspension gap; recheck readiness before
parking where completion can arrive during attachment.

Acceptance: FIFO fairness between cooperative siblings, no duplicate resume,
checkpoint produces Unit, rejected manual use preserves its resumer, and stale
queue/wait events cannot revive terminal or newly admitted executions.

### Transition implementation

Keep three identities explicit: Fiber, queue admission generation, and wait
registration generation. At checkpoint validate authority first, save continuation
state, reserve one new queue generation, append at the tail and switch through the
existing Fiber transfer gateway. At dequeue, reject stale generations without
altering current ownership. Do not enqueue while the Fiber still advertises a
separately resumable Running state. Reuse existing status variants when they can
express this atomic transition; add a variant only with transition tests.

For root checkpoint, drive one admitted turn and restore the root's retained
result/current identity. This is cooperative fairness, not preemption: a sibling
that does not suspend can still run indefinitely. No arbitrary source callback
runs while queue ownership is half-mutated. Store an explicit wait-set outcome
when a wake races cancellation so the resumed code does not infer cause from None.

## N3 — Time, deadlines and honest root waiting

Implement the ruled `System.sleep(milliseconds) -> Future` shape from
[System](../../../spec/current/system.md) and the
[reactor contract](../../../spec/current/stdlib/reactor.md), not the earlier draft's
synchronous `sleep(Duration) -> Unit`. Accept a Number representing finite integral
milliseconds, reject negative/fractional/out-of-range input, and use a monotonic
checked deadline internally. Zero creates a pending Future that settles on the
next pump. The existing contract names None as success data: type it as
`Future<Option<Never>>` under the canonical Option model unless a separately
ratified migration changes that payload. A rejected Future is the failure channel;
do not wrap another Result inside its payload. Backoff explicitly awaits sleep.

Add `Future<T>.timeout(milliseconds) -> Future<T>` with the same units and
validation. On zero, an already-settled source wins; otherwise return a rejected
wrapper. Positive deadlines settle no earlier than the deadline. Timeout never
implicitly cancels the source. One arbiter detaches the losing timer/source
registration and preserves the loser's failure-reporting responsibility.

Implementation sequence:

1. Add a VM-owned timer table indexed by generation-tagged registration tokens,
   plus a min-heap ordered by `(deadline, insertionSequence)`. Heap entries contain
   token/scalars; the live table owns traced Future handles. On removal increment
   the generation and release roots; heap tombstones cannot settle a reused slot.
2. Source sleep validates and allocates the pending Future, then a native
   registration seam records it. Natives never call source settlement recursively.
   Safepoint event draining supplies terminal records to the source settlement
   gateway. Use the established reactor floor contracts; do not invent a second
   incompatible registration table beside an implementation already in progress.
3. Add nonblocking due-event draining while runnable work exists, and a blocking
   idle wait bounded by the earliest deadline when runnable work is empty. Worker
   completions, if present, wake the wait through plain-data events; no heap Value
   crosses the thread boundary. Use C2's readiness attachment and ticketed wake.
4. Adopt the reactor's ruled exit conjunction: no runnable Fiber, no pending
   registration, and no undrained completion. A live timer keeps the program alive.
   The current runnable-only root drain is the pre-reactor implementation boundary,
   not the target rule. Test a timer scheduled without an explicit root await.
5. Root await with no ready work and no external progress source still reports a
   deadlock. Pending registrations supply progress capability, not a general proof
   that any particular Future will settle; expose their ownership in diagnostics
   instead of claiming complete deadlock detection. Follow existing leak/shutdown
   reporting without synthesizing IO failures for abandoned pending Futures.

Use an injectable clock and event driver for deterministic tests. P1.E5 owns fixed/exponential Backoff integration; supply its clock seam and
Future-shaped timer, then verify that cross-plan integration. No wall-clock timing assertions in unit
tests. Cover timer detachment, equal deadlines, source/timeout ties, GC while idle,
clock overflow, external event after cancellation and shutdown cleanup.

## N4 — Structured tasks before broad cancellation APIs

### Cancellation design gate

The existing [Future cancellation document](../../../spec/current/stdlib/cancellation.md)
is explicitly proposed pending PDR-0017 ratification. It describes renouncing a
Future, rejecting with catchable CancelledError and releasing its registration;
it does not stop its producer. Task cancellation below instead requests execution
unwind and joins cleanup. Keep distinct operations and ownership. Reconcile these
proposals and the accepted error/control rules before implementing public task
cancellation; do not silently promote either proposal through this plan. Internal
wait detachment remains usable without ratifying a public Future.cancel API.


Introduce `TaskScope.run<R>((TaskScope) -> R) -> R` and
`scope.spawn<T>(() -> T) -> Task<T>`. Task is a lifecycle owner with a typed join
result, not another name for Future. `Task<T>.join() -> T` waits through cleanup;
`cancel() -> Unit` requests cancellation idempotently. Keep Future a reusable
completion value: cancelling one waiter does not cancel every consumer's work.

Scope exit waits for every child, including cleanup. On body failure or first
child failure, request sibling cancellation, then join all children before
propagating. Body failure is primary when present; otherwise the first observed
non-cancellation child failure is primary. Retain later failures and cleanup
failures in deterministic spawn order as suppressed diagnostics. Never replace
the primary failure with routine sibling cancellation. On normal body completion,
join outstanding children normally. Nested scopes own their own child sets.

Cancellation is a control outcome delivered at checkpoints and blocking waits,
with late success versus cancellation decided once. Revoke active waits before
resuming for unwind. Run ensures exactly once; shield cleanup from repeat
cancellation while allowing cleanup itself to suspend. Publish terminal task
state only after cleanup. Proposed cancellation remains distinct from Error data
and ordinary catchable failure; reconcile this public rule in the specification
before implementation. Do not implement cancellation as `Fiber.abort` or direct
terminal-state assignment.

Acceptance: child failure during body execution, body failure with parked
children, simultaneous failures, nested scope exit, child cancelled before first
run, cancellation while parked and while cleanup parks, failure in cleanup,
normal late completion, escaped handles and no child surviving scope exit.

Generic Fiber remains owned by C1.P4: start with proven entry input and terminal
result types; do not type the mixed yield/return resume result as terminal R.
Heterogeneous scheduler queues require an erased capability or existential
boundary, not fabricated `Fiber<Dynamic, Dynamic>` subtyping.

### Owned task and scope records

Task record: typed terminal outcome, execution handle, scope identity, spawn index,
cancellation-request bit, join registrations and lifecycle New/Running/Completing/
Done. The runtime outcome can include Cancelled separately from success/failure;
Future projection must follow the ratified public cancellation rule. Completing
means cleanup is still active and join cannot yet return. Scope record: Open/
Closing/Closed, ordered child records, live count and primary/suppressed failures.
Reject spawn once Closing, including through an escaped scope handle.

Implement scope body as owned work so a child's early failure can request a
checkpoint-delivered interruption of the still-running body. Every terminal child
observer updates its record exactly once, cancels siblings when required and wakes
scope join only after live count reaches zero. Keep task records reachable through
scope closure; release completed execution stacks/captures while retaining typed
outcomes required by escaped task handles. Already-joined child failures do not
create duplicate unhandled reports; the scope still owns its chosen fail-fast policy.

Cancellation delivery goes through C2's transfer router, never direct stack
truncation. Define repeated join as reading the stable terminal outcome; a caller
cancelled while joining detaches only its join wait. Document that a noncooperative
child or indefinitely parked shielded cleanup can prevent scope exit: deadlines
request cancellation and cannot guarantee forcible termination in this model.

## N5 — Bounded typed channels with backpressure

Add a factory producing `Sender<T>` and `Receiver<T>` endpoints for a capacity.
`send(T) -> Unit` waits for capacity; `receive() -> Option<T>` waits for an item
or returns None after closure and buffer drain. Negative capacity is invalid;
zero is rendezvous. Sending an Option value preserves its nesting, so a sent None
is distinguishable from channel closure. Use explicit close, never GC finalization
as the semantic end-of-stream event.

Close is idempotent: disallow new sends, fail waiting senders, preserve already
accepted buffered items, and wake receivers to drain or observe closure. Define
an accepted send as buffer insertion or a committed rendezvous, not registration.
Cancellation removes an uncommitted operation without losing a value; cancellation
after commit does not roll back an item already transferred. FIFO applies among
live waiters on each endpoint. Closed-send errors are raised outcomes, not values
invented in T. No lock or thread-safe claim is needed for this VM model.

Acceptance: capacity 0/1/N, FIFO backpressure, close with both kinds of waiter,
buffer drain, multiple receivers, cancellation before/after commit, nested Option
payloads, GC while blocked, and repeated close. Measure retained wait records
under repeated cancellation to expose leaks.

### Channel algorithm and retention

Use one shared ChannelState<T> containing capacity, bounded ring buffer, closed
flag and FIFO queues of send/receive operation records. Sender/Receiver hold that
state; record identity and status distinguish Waiting/Committed/Cancelled. Pending
send records retain their T until commit/cancel. Pending receives retain only
result slots and wake capabilities. Never execute user callbacks while matching.

Send: reject if closed; honor older active senders; match the oldest eligible
receiver, otherwise insert if space exists, otherwise enqueue and park. Receive:
first drain the oldest buffered item; refill the freed slot from the oldest live
sender before admitting newcomers; with an empty buffer rendezvous with a sender,
otherwise observe closed or enqueue and park. Skip detached tombstones and compact
amortized. At rendezvous mark both sides Committed and transfer T before wake.
Wake both sides exactly once through their tickets.

Close detaches/fails uncommitted senders, then drains buffered items into waiting
receivers and completes the remainder with None. Clear abandoned send payloads
immediately. Cancellation after commit must not cause automatic resend: expose the
chosen success-versus-cancellation ordering in the operation contract and tests.
Do not infer last-sender closure from nondeterministic GC.

## N6 — Bounded parallel composition and explicit retry

Add `mapConcurrent<T,U>(List<T>, limit: Int, (T) -> U) -> Future<List<U>>`
using an internal task scope. Validate limit > 0 and snapshot inputs. Maintain at
most limit running actions, preserve input order, stop admission on failure,
cancel/join active siblings, and settle only after cleanup. Do not spawn every
action and call that bounded concurrency. A returned Future is U-as-data;
asynchronous adoption needs an explicitly named variant or an explicit await.

Retry takes an action factory, positive maximum attempts, an explicit retryable
error predicate and Backoff. Invoke a fresh action per attempt; never re-await
the same failed Future. Stop on cancellation; preserve the final failure and
attempt count. Do not retry every error by default or hide timeout/cancellation
inside policy callbacks. Jitter may follow later with an injectable RNG.

Acceptance: measured peak in-flight actions never exceeds limit, failure stops
new admissions, result order remains stable, cleanup finishes before settlement,
attempt boundaries and capped delay are deterministic, and cancellation prevents
the next attempt.

### Worker and retry implementation

Use min(limit, input length) worker tasks sharing a next-index counter that is
claimed without suspension. Each worker claims one index, runs its callback,
stores that result and repeats only while the scope is healthy. Callback failure
stops new claims before sibling cancellation; an empty list returns an empty
result without workers. Scope completion, not remaining-counter zero alone,
settles the returned Future. Retain only the snapshot, result slots and live task
records; do not allocate one parked Fiber per unstarted input.

Retry uses one observed action loop: invoke a fresh factory, catch only retryable
ordinary failures after cleanup, check attempt budget/cancellation, calculate
P1.E5's delay and await N3's timer, then invoke again. Predicate failure is its own
failure and is not retried. Preserve the original terminal error identity with
attempt metadata outside its payload when possible; no error-message-only cloning.
Tests use a counter and fake clock, and assert no action starts after cancellation.

## N7 — Select requires committing one operation

Build typed operation descriptors for channel receive/send and timer readiness.
Register alternatives without consuming a value; atomically choose one winning
operation in the cooperative scheduler, commit its transfer, and detach all
losers. Keep result identity tagged by branch; begin with homogeneous receives
if heterogeneous result typing is not ready. Rotate the preference among already
ready branches for fairness and make the policy testable.

Do not race Futures wrapping independent receives: losing branches can consume
and discard channel items. A correct select shares an arbitration record between
all alternatives and commits at most once. Test ready ties, close versus send,
cancellation during registration, self-selecting send/receive on one rendezvous
channel, losing timers, GC and repeated fairness. Defer the public API rather
than shipping lossy selection when reservation/commit is unavailable.

### Arbitration implementation

SelectState holds operation descriptors, a rotating start index, winner=None,
per-branch registration handles and the waiting Fiber's ticket. Probe ready
alternatives without mutation, then register alternatives under the same arbiter
without suspension, recheck, and park only if no branch committed. Both immediate
and delayed completion use the same winner transition. A branch can transfer an
item only after its arbiter wins. On a rendezvous involving two select operations,
commit only if both distinct arbiters can win; one select's own send and receive
must not satisfy each other. Do not invoke callbacks or wake either side between
validating and committing those two records on the VM thread.

Cancellation competes through the same arbiter, detaches every alternative, and
cannot revoke a committed transfer. Clear losing send payloads and timer roots.
Begin with tagged homogeneous receive results `(branchIndex, Option<T>)`; extend
heterogeneous selects only with a canonical sum type and inference tests. A timeout
branch has its own tag and cannot masquerade as closed-channel None.

## Packaging and delivery gates

Keep existing concurrency imports working. Extract `future.ph` and new
`completion.ph`, `combinators.ph`, `time.ph`, `task.ph`, `channel.ph` only when
their unit lands; expose through `concurrency/package.ph`. Avoid a cosmetic split
before bootstrap dependency order and cyclic imports are understood. Internal
registration/control capabilities must not become public exports.

For each unit, add semantic tests for exact public types and rejected misuse,
source-language tests for observable behavior with the full Universe, and focused
runtime tests for transitions/GC that source tests cannot isolate. Run the
smallest meaningful test first, then the affected suite once at the checkpoint;
run Cargo serially. Verify the canonical Universe bootstrap without adding
diagnostic-baseline allowances for the new API. Record exact selected test counts
and any incomplete gate. Workspace certification is a separate requested gate.

Each delivery includes a short usage example, normative contract updates where
required, changed source/fixture paths, and verified limitations. The next agent
should begin with N0 after P1.E1–E3's gates, then N1. Record later runtime
prerequisites individually; no repository-wide audit is needed to start.

Deferred: actors, parallel executors, Send/Sync traits, distributed work, async
syntax, task-local values, synchronization locks and full networking APIs. These
need separate ownership and evidence; none is a prerequisite for N0–N1.
