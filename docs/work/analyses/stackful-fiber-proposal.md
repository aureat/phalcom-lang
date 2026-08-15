# Phalcom concurrency: proposed architecture

The best design for Phalcom is not “add `async`/`await` and a Future library.”

It should be a coherent concurrency model built around four distinct concepts:

> **Fiber = execution**
> **Future = eventual result**
> **Scope = lifetime/ownership**
> **Scheduler = execution policy**

Cancellation is a fifth concept, deliberately separate from all four.

My recommended model is:

- stackful, extremely cheap `Fiber`s;
- ordinary blocking-looking Phalcom code, with no mandatory `async`/`await` coloring;
- structured concurrency by default;
- no orphaned work;
- `Future<T>` as a passive, read-only eventual result rather than the unit of execution;
- cooperative, sticky, hierarchical cancellation;
- M:N scheduling over OS threads;
- transparent fiber-aware I/O;
- scheduler preemption for fairness, without asynchronous cancellation exceptions;
- true multi-core parallelism permitted by the language semantics;
- memory safety even in racy programs, with a DRF-SC-style memory model;
- explicit synchronization and strong concurrency diagnostics;
- Erlang-like supervision available for genuinely long-lived work;
- Tokio-level rigor around cancellation safety, but hidden behind a much simpler interface;
- deterministic scheduling, virtual time, tracing, and schedule fuzzing as first-class testing facilities.

The closest conceptual synthesis is:

**Trio structured concurrency + Java Loom ergonomics + Go/BEAM scheduling + Ruby fiber-aware blocking + C# cancellation capabilities + Tokio cancellation rigor + Erlang supervision.**

But Phalcom should copy none of them literally.

---

# I. The three plausible architectures

There are really three families Phalcom could choose.

| Model | Example lineage | Advantages | Problems for Phalcom |
|---|---|---|---|
| Stackless async/Futures | Rust, Swift, Python asyncio, JS, C# | Efficient, explicit suspension | async coloring; APIs bifurcate; Futures become control-flow machinery |
| Cheap stackful fibers | Loom, Go, Ruby fibers | ordinary call stacks; synchronous-looking APIs; natural OOP | runtime machinery is harder |
| Actor/isolate model | Erlang, Elixir, Dart | excellent isolation and fault tolerance | changes Phalcom's object-sharing semantics substantially |

I recommend **stackful fibers with structured concurrency**, while making actors/supervisors implementable above them.

Rust demonstrates the cleanest version of the stackless alternative: an async computation is itself a `Future` that gets repeatedly polled and woken, and async code suspends around `.await`. This is excellent for Rust's ownership-oriented zero-cost model, but it means the Future is both an eventual computation and a scheduling protocol.

Phalcom doesn't need that constraint.

Java's virtual threads demonstrate that ordinary blocking code can scale when the execution context is cheap and parking releases the carrier thread.  Ruby similarly allows ordinary operations such as reads, writes, sleeps, mutex acquisition and joins to route through a fiber scheduler.

That architecture fits Phalcom unusually well.

---

# II. Core concurrency doctrine

I would make the following principles normative.

## 1. Every Phalcom computation executes in a Fiber

There should not be two worlds:

```text
normal code
async code
```

The main program already executes in a root Fiber.

Every method call therefore has a fiber execution context available.

A Fiber may:

- execute;
- be preempted;
- voluntarily yield;
- park waiting for something;
- migrate between scheduler workers;
- complete;
- fail;
- receive cancellation.

This immediately removes the fundamental reason for an `async` keyword.

---

## 2. Blocking a Fiber must not normally block an OS thread

This:

```phalcom
data = socket.read
```

should conceptually mean:

1. try the operation;
2. if immediately ready, continue;
3. otherwise register interest with the runtime;
4. park the current Fiber;
5. run another Fiber;
6. resume this Fiber when the operation becomes ready.

The source code remains sequential.

The scheduler sees concurrency.

---

## 3. Concurrency creation is always explicit

Transparent suspension is acceptable.

Transparent concurrency creation is not.

Nothing should spontaneously create a Fiber because a method returned a Future.

You get concurrency only through an operation such as:

```phalcom
scope.spawn: || {
    loadUser
}
```

Trio's “no implicit concurrency” principle is excellent here: child tasks only exist when explicitly started, and nurseries ensure that they remain associated with an owner.

This distinction is crucial:

> Methods may suspend implicitly.
> Methods must never become concurrent implicitly.

---

## 4. Structured concurrency is the default law

A newly created Fiber belongs to a scope.

The scope cannot successfully close while its children remain alive.

A parent cancellation propagates downward.

A child cannot cancel its parent merely because the child was cancelled.

A child failure follows the scope's failure policy.

This creates a tree analogous to the ordinary call stack. Java's structured-concurrency work explicitly frames the task hierarchy as the concurrent counterpart of a call stack, which is exactly the mental model Phalcom should adopt.

Kotlin similarly defines parent/child coroutine lifetimes so parents wait for children and cancellation propagates downward.

---

# III. The object model

I would standardize these concepts.

```text
Fiber<T>                 running computation
Future<T>                read-only eventual outcome
Promise<T>               low-level completion capability
FiberScope               structured owner of fibers

Cancellation             operations on current cancellation context
CancellationToken        read-only cancellation observation
CancellationSource       authority to request cancellation
CancelReason             immutable cancellation cause

Scheduler                 execution domain
Clock                     monotonic scheduling clock
Wait                      generalized waiting/select machinery

Channel<T>                message synchronization
Mutex                     mutual exclusion
RwLock                    reader/writer exclusion
Semaphore                 permits/backpressure
Atomic<T>                 explicit atomic mutable location
```

Internally:

```text
Waker
WaitRegistration
Runnable
Worker
Reactor
TimerRegistration
BlockingOperation
```

Those should not be normal application-level abstractions.

And I would deliberately **not create `Task<T>`**.

A Task type would duplicate Fiber:

```text
Fiber = executing task
Future = its result
Scope = its owner
```

That's already a complete decomposition.

---

# IV. Fiber

A `Fiber<T>` is a logical execution context whose body eventually produces `T`.

It is not an OS thread.

It is not a Future.

It is not a coroutine object that arbitrary callers manually `resume`.

That last point matters.

Ruby's Fiber exposes `resume`, `yield`, and `transfer`, which is useful for implementing coroutine machinery.  Phalcom's application-level Fibers should be more constrained:

> Only the scheduler resumes parked Fibers.

Manual symmetric control transfer should, if ever supported, live under a lower-level `Continuation` abstraction.

## Recommended Fiber state machine

```text
Created
   ↓
Runnable
   ↓
Running ──────────────┐
   ↓                  │
Parked ──wake─────────┘
   ↓
Cancelling
   ↓
Completed(T)
Failed(Error)
Cancelled(CancelReason)
```

`Created` can be extremely transient.

Once spawned, the Fiber becomes runnable.

No ordering should be promised regarding whether the child runs before or after `spawn` returns.

## Recommended surface

Conceptually:

```text
Fiber.current
Fiber.scoped: block

fiber.id
fiber.name
fiber.state
fiber.parent
fiber.future

fiber.join
fiber.value

fiber.cancel
fiber.cancel reason:

Fiber.yield
```

I would define:

```text
fiber.join -> Outcome<T>
```

and:

```text
fiber.value -> T
```

where `value` rethrows a producer failure.

`join` is lifecycle-oriented.

`value` is result-oriented.

---

# V. Future

Phalcom should be extremely precise about what Future means.

> A `Future<T>` is an immutable observation capability for an eventual `Outcome<T>`.

It does not inherently execute anything.

It does not own anything.

It does not carry cancellation authority.

It does not have its own scheduler.

## State

Mathematically:

```text
Future<T>
    = Pending
    | Resolved(Outcome<T>)
```

with:

```text
Outcome<T>
    = Success<T>
    | Failure<Error>
    | Cancelled<CancelReason>
```

Completion is:

- exactly once;
- immutable;
- thread-safe;
- observable by any number of waiters.

## Recommended interface

```text
future.ready?
future.peek
future.wait
future.value
```

Semantics:

```text
peek  -> Option<Outcome<T>>
wait  -> Outcome<T>          # may suspend
value -> T                   # may suspend; unwraps/rethrows
```

Multiple calls are valid.

Multiple Fibers may wait simultaneously.

## Future is hot, but does not start anything

This distinction is worth making explicit.

Creating a Future does not schedule computation.

Spawning a Fiber schedules computation.

Its result happens to be observable through a Future.

That avoids Rust's deliberately lazy Future semantics and JavaScript-style Promise execution semantics becoming part of Phalcom's language model.

Python's own asyncio documentation says Future objects are primarily a low-level interoperability mechanism and recommends not exposing them as the main user-facing API.

Phalcom can expose Future because eventual values are useful, but it should not make developers program *in Futures*.

Normal code should look like normal code.

---

# VI. Promise

`Promise<T>` should exist primarily for runtime/library/FFI integration.

```text
promise = Promise<T>.new

promise.future

promise.resolve: value
promise.fail: error
promise.completeCancelled: reason
```

Only one terminal transition succeeds.

A Promise is therefore a **write capability**.

A Future is the corresponding **read capability**.

This is a valuable authority separation.

Application code should rarely construct Promises.

---

# VII. The most important rule: Future cannot cancel its producer

Do not implement:

```phalcom
future.cancel
```

That API conflates two unrelated authorities:

```text
"I can observe your result"
```

and:

```text
"I can terminate your work"
```

Consider:

```text
database query
       ↓
    Future
    ↙    ↘
request A  request B
```

If A times out, A should stop waiting.

It should not necessarily destroy the computation B is using.

Cancellation authority belongs to:

- a Fiber handle;
- a FiberScope;
- a CancellationSource.

Not to Future.

This also produces a clean race/select model.

---

# VIII. FiberScope: the centerpiece

I'd make structured spawning look approximately like this:

```phalcom
Fiber.scoped: |scope| {
    user = scope.spawn: || {
        users.load: id
    }

    posts = scope.spawn: || {
        posts.forUser: id
    }

    render user.value posts.value
}
```

The exact selector spelling can change.

The semantics matter far more.

## Scope invariant

When execution leaves `Fiber.scoped:`:

```text
every child is terminal
```

No exceptions.

A lexical scope therefore owns the lifetime of concurrent work started inside it.

Trio nurseries provide essentially this guarantee: the nursery does not close until its tasks have terminated, preventing orphan tasks.

This should be one of Phalcom's strongest concurrency invariants.

---

# IX. Default failure policy

The default scope should be fail-fast.

Suppose:

```text
parent
 ├── A
 ├── B
 └── C
```

If B fails unexpectedly:

```text
B → Failed(error)

scope:
    requests cancellation of A
    requests cancellation of C

waits for A/B/C termination

then propagates B's failure
```

But if A or C also fail while shutting down, those errors must not disappear.

Therefore:

```text
1 failure  → propagate original error
N failures → ConcurrentError / ErrorGroup
```

Sibling cancellations caused by the original failure are not themselves errors.

Cleanup failures are.

This is substantially better than “first exception wins and everything else disappears.”

---

# X. Parent termination semantics

There are three important cases.

### Parent returns normally

Children continue until completion.

The scope waits.

Then parent scope returns.

### Parent fails

All children receive cancellation.

Scope waits for them to terminate.

Original parent failure remains primary.

Additional child failures are attached/aggregated.

### Parent is cancelled

All children receive cancellation.

Scope waits for termination.

Cancellation then continues propagating outward.

Thus:

> Scope exit is a synchronization barrier.

---

# XI. Cancellation should be its own control-flow domain

This is probably the most consequential semantic decision after structured concurrency.

Do not model cancellation as an ordinary `Error`.

Python has had to take special care here: `asyncio.TaskGroup` itself uses cancellation internally and its documentation warns that swallowing `CancelledError` can cause structured-concurrency components to misbehave.

Swift and Kotlin also use cooperative cancellation: tasks have to reach points where cancellation is observed.

Phalcom can design this more cleanly from the beginning.

I propose an internal hierarchy like:

```text
ControlSignal
    CancellationSignal
```

separate from:

```text
Error
```

Consequences:

```phalcom
catch Error
```

does not accidentally swallow cancellation.

Cleanup still runs.

Explicit cancellation handling remains possible.

---

# XII. Cancellation is sticky

Once cancellation is requested for a Fiber:

```text
cancelRequested = true
```

That state remains true.

If code explicitly catches the cancellation signal and then reaches another cancellation point, cancellation is delivered again.

You don't casually “consume” cancellation.

This prevents:

```text
catch everything
ignore
continue forever
```

from silently defeating scope shutdown.

There should be no general:

```text
uncancel()
```

operation.

---

# XIII. Cancellation is cooperative—not asynchronous destruction

This distinction is critical.

Scheduler preemption:

```text
may occur at VM safepoints
```

Cancellation delivery:

```text
occurs only at cancellation points
```

These are completely different mechanisms.

Do **not** inject cancellation exceptions at arbitrary VM safepoints.

That produces the same fundamental problem as asynchronous thread termination: code might be interrupted halfway through restoring an invariant.

Instead:

```text
I/O wait
Future.wait
Channel.send
Channel.receive
Mutex.lock
Semaphore.acquire
Clock.sleep
Fiber.yield

Cancellation.check
```

are cancellation points.

CPU-bound algorithms that want prompt cancellation use:

```phalcom
Cancellation.check
```

at algorithmically safe boundaries.

That makes cancellation predictable.

---

# XIV. Preemption still exists

A Fiber that performs CPU work must not monopolize a worker forever.

Therefore the VM scheduler uses preemptive time slicing.

Not OS-signal-based destruction.

Instead, the VM counts execution budget:

```text
bytecode instructions
calls
backedges
allocations
or equivalent JIT safepoints
```

When the Fiber exhausts its quantum:

```text
Running → Runnable
```

and another Fiber gets an opportunity.

BEAM demonstrates the value of making lightweight execution contexts cheap and scheduling them over a fixed set of scheduler threads; current Erlang runtimes generally use multiple scheduler threads corresponding to available cores.

Phalcom should take that idea much further into its OOP runtime.

Important:

> Preemption changes which Fiber executes.
> It does not throw anything into the preempted Fiber.

---

# XV. Cancellation hierarchy

Each Fiber carries an effective cancellation context.

Conceptually:

```text
root
  ↓
request
  ↓
operation
  ↓
child fibers
```

A child automatically inherits its parent's cancellation.

A parent cancellation propagates recursively downward.

A child cannot cancel its parent.

Go's Context design captures the same crucial directionality: children observe parent cancellation, while a recipient of cancellation state doesn't get authority to cancel the parent.

But Phalcom should avoid requiring `context` arguments everywhere.

---

# XVI. No Context plumbing

Do not make Phalcom APIs look like:

```phalcom
database.query context: context sql: sql
http.get context: context url: url
service.run context: context
```

The current Fiber already provides the dynamic operation context.

Therefore:

```phalcom
http.get: url
```

automatically respects current cancellation and deadline state.

Explicit tokens remain useful for special ownership boundaries.

This gives Phalcom the useful semantics of Go/C# cancellation without infecting every selector signature.

---

# XVII. CancellationToken and CancellationSource

For explicit cross-boundary cancellation:

```text
CancellationSource
       │
       ├── cancel
       │
       └── token ───────→ observers
```

Recommended interface:

```text
source = CancellationSource.new

source.token
source.cancel
source.cancel reason:

token.cancelled?
token.reason
token.wait
token.check
```

Only `CancellationSource` owns cancellation authority.

`CancellationToken` is observational.

C# uses this source/token distinction and cooperative cancellation extensively.

This is a good capability model.

---

# XVIII. Deadlines and timeouts are cancellation scopes

No separate timeout machinery.

Conceptually:

```phalcom
Cancellation.withTimeout: 2.seconds do: || {
    remote.fetch
}
```

is:

```text
create nested cancellation scope
deadline = now + 2 seconds

execute block

if deadline expires:
    request cancellation
    wait for cancellation-safe unwinding
    report Timeout
```

Nested deadlines should compose as:

```text
effectiveDeadline =
    min(parentDeadline, requestedDeadline)
```

A child cannot silently extend the parent's deadline.

Timers use monotonic time.

Never wall-clock time.

---

# XIX. Shielding

Sometimes cleanup itself has to suspend:

```text
cancelled
→ flush buffer
→ close protocol
→ release remote lease
```

But ordinary cancellation points would immediately cancel that cleanup.

Therefore:

```phalcom
Cancellation.shield: || {
    resource.close
}
```

temporarily masks delivery of outer cancellation.

The cancellation request remains pending.

When the shield ends, cancellation becomes observable again.

Shielding should be explicit and preferably deadline-bounded.

No permanent cancellation suppression.

---

# XX. Hard kill: deliberately absent

Do not provide:

```text
Fiber.killImmediately
Fiber.terminate
Fiber.abortUnsafe
```

for normal application code.

There is no safe general way to asynchronously destroy computation sharing mutable process state.

A Fiber ignoring cancellation can delay scope shutdown indefinitely.

That is an honest semantic fact.

Java's structured-concurrency documentation makes essentially the same observation: a child blocked in something that does not respond to cancellation can prevent scope closure.

Phalcom should diagnose that rather than pretend it can safely solve it.

If truly untrusted computation requires forcible termination, use an isolate/process boundary.

---

# XXI. Transparent suspension without invisible semantics

There is one objection to removing `async`/`await`:

> How does the programmer know a method may suspend?

Phalcom has a better option.

Make suspension a reflective/compiler effect.

For example, conceptually:

```text
@suspends
Socket.read

@suspends
Future.wait

@suspends
Mutex.lock
```

The effect propagates through inference.

A method calling a suspending method is itself inferred as potentially suspending.

But application source does not require:

```text
async foo
await bar
```

The LSP can show:

```text
↕ may suspend
```

at calls.

Reflection can expose:

```text
method.suspends?
```

The checker can prohibit suspension in places marked:

```text
@nosuspend
```

This gets almost all of async/await's semantic visibility without infecting syntax or return types.

Ruby's Fiber scheduler proves transparent suspension is feasible, while also documenting that scheduler hooks introduce nondeterministic context-switch points.  Phalcom should make that potential suspension visible through semantic tooling rather than keywords.

This fits extremely well with Phalcom's eventual type/effect machinery.

---

# XXII. Scheduler architecture

Now the runtime underneath it.

The default scheduler should be:

```text
M Fibers
    ↓
N worker threads
    ↓
hardware cores
```

with approximately:

```text
N ≈ available CPU parallelism
```

This is an M:N scheduler.

Each worker gets:

```text
local runnable deque
current Fiber
scheduler bookkeeping
event/reactor integration
```

There is also:

```text
global injection queue
timer subsystem
I/O reactor
blocking-call executor
```

---

# XXIII. Work stealing

A good default topology:

```text
                 global wake queue
                       │
        ┌──────────────┼──────────────┐
        ↓              ↓              ↓
     Worker 0        Worker 1       Worker 2
        │              │              │
     local Q         local Q        local Q
        ↖──────── work stealing ─────↗
```

Normal spawn/wake prefers locality.

Idle workers steal work.

External threads inject runnable Fibers into a shared queue.

The scheduler should not specify exact run ordering as language semantics.

Programs relying on:

```text
"I spawned A first, therefore A will run first"
```

are incorrect.

---

# XXIV. Scheduling fairness

No strong FIFO guarantee.

No deterministic production scheduling guarantee.

But there should be a liveness contract approximately equivalent to:

> A runnable Fiber of ordinary priority should not remain indefinitely unscheduled under finite load.

Preemption prevents CPU-bound Fibers from monopolizing a worker.

Work stealing prevents one worker from being overloaded while another is idle.

---

# XXV. Don't expose numeric thread priorities in v1

Avoid:

```text
priority = 1..100
```

They create:

- priority inversion;
- starvation;
- platform inconsistencies;
- scheduler-policy dependencies.

If later necessary, expose semantic hints:

```text
interactive
default
background
```

without exact ordering guarantees.

But I would omit even those initially.

---

# XXVI. Scheduler and reactor should be separate concepts internally

This distinction will pay off enormously.

```text
Scheduler
    decides WHO runs

Reactor
    decides WHAT became ready
```

The reactor handles:

```text
socket readiness
I/O completion
timers
process waits
signals
```

The scheduler handles:

```text
runnable Fiber queues
work stealing
worker parking
preemption
affinity
```

Do not create one giant EventLoop god object.

---

# XXVII. Public scheduling interface versus runtime SPI

Phalcom should have two layers.

## Application-facing Scheduler

Something roughly like:

```text
Scheduler.current
Scheduler.default

scheduler.statistics
scheduler.blocking: block
scheduler.run: block
```

Potentially:

```text
Scheduler.serial
Scheduler.testing
```

later.

Applications should almost always create concurrency through `FiberScope`, not through direct Scheduler calls.

## Runtime scheduler SPI

A lower-level extension interface can expose operations conceptually like:

```text
enqueue runnable
nextRunnable worker:
wakeWorker:
parkWorker until:
```

But an important rule:

> Custom schedulers control policy, not Fiber correctness.

The VM—not the scheduler plugin—owns:

- Fiber state transitions;
- cancellation;
- waiter registration;
- lost-wakeup prevention;
- Future completion;
- scope invariants.

This is where I'd improve significantly on a Ruby-style scheduler-hook model.

Custom scheduling policy should not be capable of violating Phalcom concurrency semantics.

---

# XXVIII. Wait/Waker machinery

Internally, all suspension should converge onto one primitive.

Conceptually:

```text
current Fiber
     │
     ├── register wait
     ↓
WaitRegistration
     │
     ├── Waiting
     ├── Woken
     └── Cancelled
```

A `Waker` contains enough information to transition the Fiber back to Runnable.

The critical race is:

```text
event completes
```

at exactly the same moment that:

```text
Fiber parks
```

The state transition must be atomic so a wakeup can never be lost.

A typical structure:

```text
REGISTERED
   ├── event → COMMITTED
   └── cancel → CANCELLED
```

exactly one wins.

That machinery becomes the foundation for:

- Future waiting;
- channels;
- mutexes;
- semaphores;
- timers;
- sockets;
- process waits;
- cancellation tokens;
- `select`.

---

# XXIX. `Wait` / select should be native

Concurrent programs inevitably need:

```text
wait for A or B or timeout or cancellation
```

Do not force users to spawn helper Fibers.

Provide a generalized selection facility.

Something conceptually like:

```phalcom
Wait.first: [
    inbox.receiveCase,
    shutdown.cancelCase,
    Clock.after: 5.seconds
]
```

The exact surface can be refined later.

The important semantic property is:

> Losing cases are deregistered, not destructively cancelled.

Tokio exposes exactly how difficult cancellation safety becomes when `select!` drops in-progress Futures; its documentation has to classify individual operations according to whether cancellation loses state or data.

Phalcom can avoid a large portion of that complexity by designing native waiting operations around atomic registration/commit.

---

# XXX. Cancellation-safe operation semantics

Every suspending primitive needs a commit point.

For example:

```text
channel.send(message)
```

must behave as either:

```text
cancel before commit
→ message was not sent
```

or:

```text
commit wins
→ message was sent
→ operation completes normally
```

Never:

```text
maybe sent, but caller cannot know
```

where avoidable.

Similarly:

```text
Mutex.lock
Semaphore.acquire
Channel.receive
```

should have well-defined commit boundaries.

This ought to become a core standard-library design requirement:

> Every suspending method documents its cancellation semantics.

---

# XXXI. Networking and I/O

All native Phalcom I/O should be Fiber-aware from the start.

Platform implementation can use:

```text
Linux       epoll / io_uring where useful
BSD/macOS   kqueue
Windows     IOCP
```

without exposing those mechanisms.

The runtime API remains:

```text
operation ready?
    yes → execute
    no  → park Fiber
```

A Fiber should not care whether the operating system uses readiness or completion-based I/O.

---

# XXXII. Blocking operations

Some operations cannot be made genuinely asynchronous:

- legacy file APIs;
- DNS implementations;
- C libraries;
- Rust libraries with blocking interfaces;
- device APIs;
- foreign runtimes.

These go to a separate blocking executor.

```text
Fiber
  │
  ├── submit blocking operation
  │
  └── park
          ↓
     blocking worker
          ↓
       completion
          ↓
       wake Fiber
```

Never run arbitrary blocking foreign calls on scheduler workers.

Otherwise ten blocking C calls can freeze the entire Fiber runtime.

---

# XXXIII. Native/FFI annotations

Given Phalcom's future Rust interop ambitions, I would design this now.

Native operations should fall into explicit classes.

```text
@native
```

Short, nonblocking native call.

```text
@blocking
```

May block an OS thread; runtime automatically executes it through blocking executor when called by a Fiber.

And eventually something like:

```text
@threadAffine
```

Requires a particular OS thread/execution domain.

Potentially:

```text
@suspends
```

Native implementation participates directly in Phalcom parking/waking.

This eliminates guesswork for both runtime and tooling.

---

# XXXIV. Fiber implementation machinery

Semantically the Fiber is stackful.

That does **not** mean Phalcom should allocate a giant native stack for every Fiber.

Assuming a VM architecture, I would represent suspended stacks using heap-resident VM frames/stack segments.

Conceptually:

```text
FiberControlBlock
    id
    state
    scope
    scheduler
    cancellationContext
    currentWait
    result
    fiberLocals
    stack
    diagnostics
```

Stack:

```text
StackSegment
    ↓
StackSegment
    ↓
StackSegment
```

or a heap frame chain.

This allows:

- cheap creation;
- lazy stack growth;
- precise GC scanning;
- Fiber migration between OS threads;
- preserved full stack traces;
- suspension in arbitrarily deep ordinary methods.

The implementation is stackful **semantically**, even if no contiguous native stack is preserved.

---

# XXXV. Fiber migration

A runnable Fiber may resume on another worker.

Therefore:

```text
Fiber.current
```

identity is stable,

but:

```text
OS thread identity
```

is not.

Never make ordinary Phalcom semantics depend upon carrier-thread-local storage.

This matters enormously.

Java's virtual-thread work specifically calls out the interaction between lightweight threads and traditional thread-local practices.

Phalcom can avoid inheriting that legacy.

---

# XXXVI. Fiber-local state

Provide fiber-scoped dynamic values rather than thread-local values.

Conceptually:

```text
FiberLocal<T>
```

A child Fiber inherits the parent's context snapshot.

Mutation becomes child-local unless the stored object itself is mutable/shared.

Excellent uses include:

- tracing span;
- request ID;
- logging context;
- locale;
- authorization principal;
- transaction metadata.

Cancellation should **not** be smuggled through this generic mechanism.

It has dedicated semantics.

---

# XXXVII. Memory model

Cheap concurrency without a memory model becomes a disaster.

Phalcom should specify from the beginning that Fibers may genuinely execute in parallel.

Do not rely on a Python-style or Ruby-style global interpreter lock as a semantic guarantee.

An implementation could temporarily serialize execution during early development, but valid Phalcom programs must not rely on that.

## Recommended contract

For shared ordinary heap state:

1. object-reference reads/writes never tear;
2. runtime memory safety is preserved even in the presence of data races;
3. synchronization establishes happens-before relationships;
4. data-race-free programs receive sequentially consistent behavior;
5. unsynchronized races are program correctness defects and may produce nondeterministic values, but never C-style arbitrary undefined behavior.

Approximately:

> **DRF ⇒ SC**

That gives the compiler plenty of freedom without turning a dynamic language bug into memory corruption.

---

# XXXVIII. Synchronization operations establish ordering

At minimum:

```text
Fiber.spawn
```

publishes preceding parent actions to the child.

```text
Fiber.join / Future completion
```

publishes child actions to the observing Fiber.

```text
Mutex.unlock → subsequent Mutex.lock
```

establishes ordering.

```text
Channel.send → matching Channel.receive
```

establishes ordering.

Atomics establish their specified memory ordering.

For ordinary application code, `Atomic<T>` should probably be sequentially consistent by default.

Advanced acquire/release/relaxed operations can come later under a deliberately lower-level API.

---

# XXXIX. Mutex

A Phalcom Mutex should be:

- Fiber-aware;
- non-reentrant by default;
- owned by Fiber identity rather than OS thread;
- cancellation-safe while waiting;
- fair enough to avoid starvation, without promising exact FIFO.

I would **not poison Mutexes** when code fails while holding them.

Failure does not necessarily imply corrupted protected state, and poisoning pushes policy into the wrong abstraction.

Use contracts/invariants for that instead.

Also:

> Suspending while holding a Mutex should be legal but diagnostically suspicious.

The LSP/checker should flag:

```text
lock held
   ↓
potentially suspending call
```

unless explicitly acknowledged.

---

# XL. Channels

Channels are worth making first-class.

Default:

```text
Channel<T>.new
```

should probably create a **rendezvous channel**, capacity zero.

Buffered channels require explicit capacity:

```text
Channel<T>.buffered: 64
```

Unbounded:

```text
Channel<T>.unbounded
```

must be explicit.

Cheap producers plus an unbounded queue are one of the easiest ways to transform good concurrency into an out-of-memory failure.

Methods:

```text
channel.send: value
channel.receive

channel.trySend: value
channel.tryReceive

channel.close
channel.closed?
```

Send/receive should be cancellation-safe around an atomic handoff/queue commit point.

---

# XLI. Backpressure is not scheduling

This deserves a hard rule.

Do not limit concurrency by creating tiny worker pools.

Cheap Fibers should remain cheap Fibers.

Resource capacity should instead use:

```text
Semaphore
bounded Channel
connection pool
rate limiter
```

So:

```text
100,000 requests
```

can exist as Fibers while perhaps:

```text
100 database operations
```

have permits.

This separates:

```text
how much work exists
```

from:

```text
how much of a scarce resource may execute
```

which is conceptually cleaner.

---

# XLII. Long-lived concurrency: supervision

Structured concurrency is perfect for:

```text
do these three things as part of this operation
```

It is not sufficient for:

```text
run this server worker for the application's lifetime
restart it if it crashes
```

That is where Phalcom should borrow from Erlang.

Erlang processes are cheap, support links/monitors, and OTP builds supervision hierarchies on top of those lifecycle signals.

I'd therefore distinguish:

```text
FiberScope
```

for lexical concurrent decomposition,

from:

```text
Supervisor
```

for long-lived services.

But Supervisor belongs in the standard concurrency library, not necessarily the VM primitive layer.

---

# XLIII. No casual detached Fibers

Do not provide an easy:

```phalcom
Fiber.spawnDetached
```

This is how goroutine/task leaks are manufactured.

Tokio's `JoinHandle`, for example, detaches its underlying task if the handle is dropped; that task can continue running with its result lost.

Phalcom should deliberately choose the opposite default.

If you want long-lived independent work:

```text
Supervisor.spawn
```

The Supervisor becomes the owner.

There is always an owner.

That should be the invariant.

---

# XLIV. Unobserved failures

A Fiber failure must never silently disappear.

For scoped Fibers:

```text
scope owns failure propagation
```

For supervised Fibers:

```text
supervisor policy owns failure handling
```

For root Fibers:

```text
runtime owns failure reporting
```

No:

```text
Future rejected but nobody looked
```

with eventual vague warning.

The runtime knows the Fiber tree. Use it.

---

# XLV. Runtime root

The whole application should itself be one structured tree:

```text
Runtime
└── root Fiber
    ├── application supervisor
    │   ├── HTTP service
    │   └── telemetry service
    │
    └── main operation
```

Normal runtime shutdown:

1. request root cancellation;
2. propagate;
3. wait for Fibers;
4. run cleanup;
5. report stuck Fibers if shutdown deadline expires;
6. perform process-level termination only as policy.

This is enormously easier to debug than “event loop stopped and some tasks vanished.”

---

# XLVI. Observability should exploit the Fiber tree

Phalcom's debugger should be able to show:

```text
Fiber #1 "main"          RUNNING
├─ Fiber #12 "request"  PARKED on socket fd=42
│  ├─ Fiber #13 "user"  PARKED on Future #91
│  └─ Fiber #14 "posts" PARKED on db pool semaphore
└─ Fiber #18 "logger"   PARKED on Channel #7
```

For every Fiber:

```text
id
name
state
parent
children
scope
creation stack
current stack
park reason
park duration
deadline
cancellation state
scheduler worker
CPU time
last scheduling event
```

Structured concurrency makes this possible precisely because concurrent work retains its causal hierarchy. Java's structured-concurrency design explicitly cites improved observability as a major benefit of representing task relationships.

Phalcom should take full advantage.

---

# XLVII. Spawn backtraces

A normal stack trace answers:

```text
where did this Fiber fail?
```

A concurrency debugger must additionally answer:

```text
who created this Fiber?
```

Keep a lightweight spawn-site trace.

Then failures can show:

```text
Fiber #527 failed here:
    DatabaseConnection.read
    UserRepository.find
    ...

Fiber #527 was spawned here:
    RequestHandler.loadData
    ...
```

That is worth modest runtime overhead, perhaps configurable in optimized production mode.

---

# XLVIII. Deterministic testing Scheduler

This should be a marquee Phalcom feature.

Provide a special scheduler:

```text
Scheduler.testing
```

with:

- one logical worker;
- deterministic runnable ordering;
- virtual monotonic clock;
- deterministic timers;
- controllable I/O fakes;
- schedule trace.

Then:

```phalcom
Scheduler.testing run: || {
    testConcurrentOperation
}
```

can execute instantly without real sleeps.

A five-minute timeout test should consume effectively zero wall-clock time:

```text
virtual clock jumps forward
```

when nothing else is runnable.

---

# XLIX. Schedule fuzzing

Then go further.

Provide:

```text
Scheduler.fuzz seed: 91231
```

which deliberately varies legal scheduling choices.

Test output includes:

```text
scheduler seed: 91231
```

A failure can be exactly reproduced.

Eventually:

```text
phalcom test --schedule-fuzz 1000
```

could exercise thousands of schedules.

That would make Phalcom concurrency substantially easier to test than mainstream thread-based languages.

---

# L. Deadlock diagnostics

Because the runtime understands parking relationships, it can build a wait graph:

```text
Fiber A → Mutex X
Mutex X → Fiber B
Fiber B → Future Y
Future Y → Fiber A
```

and identify:

```text
cycle detected
```

Similarly:

```text
all Fibers parked
no timers
no external I/O registrations
root incomplete
```

is strongly suggestive of deadlock.

The runtime can report this structurally instead of appearing to freeze.

---

# LI. Data-race tooling

Phalcom's future checker/LSP/type system can become exceptionally useful here.

Eventually it should understand concepts such as:

```text
immutable
fiber-local
shared
synchronized
atomic
```

and reason about captures:

```phalcom
scope.spawn: || {
    mutableObject.foo = 1
}
```

Potential diagnostic:

```text
mutableObject is captured by concurrently executing Fibers
without synchronization
```

This can begin as heuristic analysis and become much stronger once Phalcom's optional types and effects mature.

Crucially, the concurrency model should not depend on such annotations for memory safety.

They improve correctness.

They aren't the thing preventing a segfault.

---

# LII. What Phalcom should learn from each ecosystem

Here is the distilled comparison.

| Ecosystem | Adopt | Do not inherit |
|---|---|---|
| Go | cheap user tasks, M:N scheduling, channels, cancellation trees | cancellation/context plumbing everywhere; easy orphan goroutines |
| Java Loom | blocking-looking lightweight concurrency; full stacks; tooling | legacy Thread semantics and thread-local dependence |
| Kotlin | structured parent-child lifecycle | `GlobalScope`-style escape hatches; context becoming a grab-bag |
| Swift | structured task groups; cooperative cancellation | requiring async coloring if fibers remove the need |
| Rust | precise wake/park protocol and strong cancellation reasoning | making `Future` itself the computation/runtime protocol |
| Tokio | excellent cancellation-safety distinctions | drop-driven cancellation/detachment semantics exposed to normal users |
| Trio | nurseries, cancellation scopes, no orphan tasks | very little—this is probably Phalcom's strongest semantic influence |
| Python asyncio | TaskGroup lessons and accumulated cancellation experience | Future-centric/event-loop-centric public programming model |
| Ruby | transparent Fiber-aware blocking; scheduler interception | arbitrary low-level Fiber resume/transfer as normal concurrency |
| Erlang/BEAM | lightweight scheduling, preemption, supervision, failure hierarchy | mandatory actor isolation for every object interaction |
| C# | cancellation source/token capability split | passing CancellationToken through every Phalcom API |

Go's cancellation model explicitly carries deadlines and downward cancellation; Kotlin and Trio make scope ownership central; Swift cancellation is cooperative; Tokio documents the hazards of destructive Future cancellation; Ruby shows transparent Fiber scheduling; Erlang demonstrates cheap concurrency plus supervision.

---

# LIII. A representative request

Imagine:

```phalcom
handle: request = {
    Cancellation.withTimeout: 2.seconds do: || {
        Fiber.scoped: |scope| {
            user = scope.spawn named: "user" do: || {
                users.find: request.userId
            }

            permissions = scope.spawn named: "permissions" do: || {
                auth.permissionsFor: request.userId
            }

            render
                user: user.value
                permissions: permissions.value
        }
    }
}
```

There are no:

```text
async
await
Promise.all
event loops
callbacks
context parameters
executor parameters
```

Yet the two queries execute concurrently.

If the client cancellation reaches `handle:`:

```text
request Fiber cancelled
    ↓
scope cancelled
    ↓
user Fiber cancelled
permissions Fiber cancelled
```

If `users.find:` is waiting on a socket:

```text
socket waiter deregistered
Fiber awakened for cancellation
ensure/cleanup executes
Fiber terminates
```

If permissions fails:

```text
permissions fails
→ scope requests user cancellation
→ waits for user cleanup
→ propagates permissions error
```

That is the concurrency story users should be able to explain from memory.

---

# LIV. CPU concurrency

The same system handles CPU work.

```phalcom
Fiber.scoped: |scope| {
    chunks.map: |chunk| {
        scope.spawn: || {
            crunch: chunk
        }
    }
}
```

Those Fibers may execute truly in parallel on worker threads.

No special `parallel` Future runtime is required.

For extremely fine-grained data parallelism, Phalcom may eventually add higher-level parallel collection operators, but they should compile down to scheduler primitives rather than introduce another concurrency universe.

---

# LV. The subtle difference between scheduling and cancellation

I would explicitly put this into the language specification:

```text
Scheduling preemption is transparent.

Cancellation is observable control flow.
```

Consequently:

```text
scheduler may stop you between instructions
```

but when resumed it is as though nothing happened.

Whereas:

```text
cancellation may only become observable
at a cancellation point.
```

This is what lets Phalcom simultaneously have:

- fair CPU scheduling;
- shared mutable state;
- safe resource cleanup;
- predictable cancellation.

Do not merge those mechanisms.

---

# LVI. The subtle difference between waiting cancellation and producer cancellation

Another normative example:

```phalcom
Cancellation.withTimeout: 1.second do: || {
    externalFuture.value
}
```

If the timeout expires:

```text
current Fiber stops waiting
```

but:

```text
externalFuture's producer continues
```

unless that producer belongs to the cancelled Fiber's subtree.

This follows directly from ownership.

Future doesn't own work.

Scope does.

This one rule eliminates a surprising number of concurrency ambiguities.

---

# LVII. Result propagation and target cancellation

Suppose:

```text
Fiber A waits for Fiber B
Fiber B is independently cancelled
```

B's cancellation should **not cancel A**.

Instead B's result is:

```text
Cancelled(reason)
```

So:

```phalcom
b.join
```

returns that Outcome.

If:

```phalcom
b.value
```

is used, Phalcom can raise an ordinary `CancelledResult`-style error representing failure to obtain B's value.

That is distinct from a `CancellationSignal` delivered because A itself is being cancelled.

This distinction should be rigorous:

```text
my cancellation        = control flow
someone else's cancel  = data/outcome
```

Excellent concurrency APIs depend on this distinction.

---

# LVIII. No callback microtask universe

Future completion should wake waiting Fibers.

It should not create a JavaScript-style special microtask execution tier.

If low-level APIs support:

```text
future.onComplete
```

callbacks should be scheduled normally, never executed synchronously inside `Promise.resolve`.

Otherwise Future completion creates reentrancy:

```text
resolve()
  → arbitrary application callback
  → resolve something else
  → arbitrary callback
```

That becomes extremely difficult to reason about.

Better:

```text
resolve
→ mark completed
→ enqueue waiters
→ return
```

Very clean.

---

# LIX. Proposed ownership graph

This is essentially the entire model:

```text
                        Runtime
                           │
                     root FiberScope
                           │
             ┌─────────────┴────────────┐
             │                          │
          Fiber A                    Fiber B
             │                          │
          Future<A>                  Future<B>
             │
       child FiberScope
         ┌───┴────┐
         │        │
      Fiber C   Fiber D
```

Cancellation follows:

```text
downward ownership edges
```

Results may be observed:

```text
through Future references in any direction
```

But observing a Future does not grant control of its producer.

That separation is the key architectural insight.

---

# LX. Recommended native surface

If we aggressively minimize the primitive layer, I would ratify roughly this.

### Execution

```text
Fiber
FiberScope
```

### Eventual values

```text
Future<T>
Promise<T>
Outcome<T>
```

### Cancellation

```text
Cancellation
CancellationToken
CancellationSource
CancelReason
```

### Scheduling/time

```text
Scheduler
Clock
Wait
```

### Fundamental synchronization

```text
Mutex
Channel<T>
Semaphore
Atomic<T>
```

Everything else can initially be implemented above them:

```text
RwLock
Barrier
Latch
Once
Supervisor
worker pools
actor/mailbox
rate limiter
parallel map
stream combinators
pipelines
```

Move things native only when performance warrants it.

---

# LXI. Things I would explicitly reject

These should become design non-goals.

**No mandatory `async`/`await`.**

**No Future-driven execution model.**

**No `future.cancel`.**

**No dropping a handle as cancellation.**

**No dropping a handle as detachment.**

**No easy global spawn.**

**No user-space hard Fiber kill.**

**No automatic cancellation injection at arbitrary VM safepoints.**

**No global interpreter lock as part of language semantics.**

**No unlimited implicit blocking-worker creation.**

**No Context object threaded through every API.**

**No execution ordering promises between sibling Fibers.**

**No special Promise/microtask scheduler.**

**No giant customizable EventLoop object responsible for every concurrency semantic.**

**No silent unobserved Fiber failures.**

Those exclusions are almost as important as the features.

---

# LXII. Development sequence

I would implement the model in layers.

### Phase 1 — Fiber VM machinery

Get right:

```text
Fiber control block
heap stack/frame suspension
run / park / wake
preemption
worker queues
work stealing
Fiber.current
FiberLocal
```

No elaborate API yet.

### Phase 2 — structured lifecycle

Implement:

```text
FiberScope
spawn
join
failure propagation
scope closure
Outcome
```

This should precede fancy Future combinators.

### Phase 3 — cancellation

Implement the cancellation tree:

```text
CancellationSignal
CancellationToken
CancellationSource
deadlines
shielding
cancellation-safe parking
```

Then aggressively torture race conditions.

### Phase 4 — Future/Promise

Build Future on the same waiter machinery:

```text
single-assignment completion
multiple waiters
cross-worker wakeups
Outcome
```

### Phase 5 — timers and I/O reactor

Add:

```text
Clock
sleep
network I/O
process waits
signals
```

Every operation uses the shared WaitRegistration machinery.

### Phase 6 — synchronization

Add:

```text
Mutex
Channel
Semaphore
Wait/select
Atomic
```

with specified cancellation commit semantics.

### Phase 7 — blocking/FFI integration

Add:

```text
@blocking
native asynchronous completion
affinity
Rust integration
```

### Phase 8 — observability

Do not leave this until years later:

```text
Fiber tree
park reasons
spawn traces
scheduler traces
deadlock graphs
```

### Phase 9 — deterministic concurrency testing

Then:

```text
virtual time
deterministic scheduler
seeded scheduler fuzzing
record/replay
```

### Phase 10 — advanced libraries

Finally build:

```text
Supervisor
actors
parallel collections
pipelines
streams
rate limiting
resource pools
```

on the primitive model.

If those abstractions cannot be implemented cleanly, that's evidence the primitive layer needs fixing.

---

# LXIII. The specification-level laws

I would actually write these into the eventual Phalcom concurrency specification as laws.

**Ownership law**

> Every live Fiber has an owning scope or supervisor.

**Scope law**

> A FiberScope cannot complete while any owned Fiber remains non-terminal.

**Observation law**

> Possession of a Future grants observation of a result, not authority over its producer.

**Cancellation direction law**

> Structured cancellation flows from an owner to descendants, never implicitly upward.

**Cancellation persistence law**

> Cancellation requests are sticky until the Fiber terminates.

**Cancellation-point law**

> Cancellation becomes observable only at defined cancellation points.

**Preemption law**

> Scheduler preemption is semantically transparent.

**Completion law**

> Future completion occurs exactly once and is immutable.

**Wait law**

> Cancelling a wait does not implicitly cancel the waited-upon producer.

**Commit law**

> Cancellation-safe synchronization operations have an atomic boundary distinguishing “operation occurred” from “operation did not occur.”

**Failure law**

> Concurrent failures are never silently discarded.

**Blocking law**

> Core suspending operations park Fibers rather than scheduler workers.

**Parallelism law**

> Distinct runnable Fibers may execute simultaneously on distinct OS threads.

**Memory-safety law**

> Data races never compromise runtime memory safety.

**DRF law**

> Data-race-free programs observe sequentially consistent synchronization semantics.

**Scheduling law**

> Runnable ordering is not observable language semantics except where explicitly established through synchronization.

These laws give everything else a foundation.

---

# LXIV. Overall recommendation

The center of Phalcom concurrency should not be Future.

And it should not even really be Fiber.

The conceptual center should be:

> **the structured lifetime tree.**

Fibers populate that tree.

Schedulers execute them.

Cancellation flows down it.

Failures propagate through it.

Futures expose values produced by it.

Debuggers visualize it.

Deadlines attach to subtrees.

Supervisors own the intentionally non-lexical parts of it.

The type checker can reason about it.

The LSP can show it.

The deterministic scheduler can replay it.

That gives Phalcom something much more coherent than merely borrowing another language's concurrency syntax.

And the lack of `async`/`await` isn't simply syntactic minimalism. Stackful Fibers let Phalcom recover the ordinary abstraction law that async runtimes historically broke:

```text
If method A calls method B,
A does not need to change its entire calling convention
merely because B sometimes has to wait.
```

At the same time, the `suspends` effect restores the semantic information developers and tooling need.

I would therefore ratify the foundational architecture as:

```text
                        STRUCTURED OWNERSHIP
                               │
              ┌────────────────┼────────────────┐
              ↓                ↓                ↓
            Fiber           Future        Cancellation
          execution          result           lifetime
              │                │                │
              └────────────────┼────────────────┘
                               ↓
                            Scheduler
                               │
              ┌────────────────┼────────────────┐
              ↓                ↓                ↓
           Workers          Reactor      Blocking executor
              │                │
              └────────────────┘
                      M:N Fiber runtime
```

The next design layer I would attack after this is the set of genuinely difficult semantic decisions underneath it: **exact `FiberScope` failure algebra, `Outcome`/exception interaction, the cancellation control-signal hierarchy, `Wait/select` transactional semantics, Channel close semantics, synchronization/memory ordering, and the scheduler/runtime SPI.** Those are where we can turn this architecture into a rigorous Phalcom concurrency specification rather than just a strong architecture sketch.
