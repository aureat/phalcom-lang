# Phalcom Fibers, Futures, Scheduling, and Failure Semantics — Repository Audit

I inspected the current concurrency specification, accepted architectural decisions, `FiberObject`, native fiber primitives, VM scheduling state, `System` scheduling primitives, the pure-Phalcom `Future` implementation, error-handling semantics, reactor/cancellation designs, implementation plans, historical failure investigations, and current regression status in `aureat/phalcom-lang`.

The most important conclusion is that **Phalcom already has considerably more concurrency machinery than our previous discussion assumed**. Its Fiber is not a placeholder. It is an implemented, independently suspendable VM execution stack with resume/yield semantics, dynamic resumer chains, failure containment, scheduler integration through Futures, tracing support, and explicit parked state.

But it is also important not to overstate what exists:

> **Today's Phalcom has a low-level coroutine runtime plus a small cooperative scheduler. It does not yet have a complete concurrency runtime.**

In particular, it has no structured ownership tree, no scheduler-level park/wake abstraction, no Fiber cancellation, no cancellation propagation, no supervisor model, no implemented reactor, no fairness/preemption guarantees, no general wait/select machinery, and no parallel execution of Phalcom objects.

There is also one major architectural finding that changes part of my previous recommendation:

> **M:N shared-memory multi-core Fibers are incompatible with Phalcom's currently accepted architecture.**

PDR-0003 deliberately commits each VM to one thread and one heap owner, with parallelism—if Phalcom gets it—going through isolates rather than shared-memory threads. That is not a minor scheduler choice; it protects the current heap, GC, `ObjRef`, `Value`, and primitive ABI.

So I would now divide the future design into two layers:

1. **Near/mid-term Phalcom:** extremely capable single-threaded structured Fiber concurrency.
2. **Parallel Phalcom:** multiple isolated VMs running those schedulers concurrently, rather than multiple worker threads concurrently touching one object graph.

That actually fits Phalcom's existing architecture extremely well.

---

# 1. Executive classification

| Question | Current Phalcom |
|---|---|
| Is `Fiber` implemented? | **Yes** |
| Does every Fiber preserve an independent call stack? | **Yes, a VM call stack** |
| Is it “stackful”? | **VM-stackful/direct-style: yes. Native-stackful: no.** |
| Lua/Ruby coroutine-like? | **Very much so; ADR explicitly calls it Lua-5.1-style** |
| Resume operation? | **`fiber.call(...)` / `fiber.try(...)`** |
| Yield operation? | **`Fiber.yield(...)`** |
| Bidirectional resume/yield values? | **Yes** |
| Ruby-style `transfer`? | **No** |
| Can arbitrary Phalcom calls be suspended? | Mostly |
| Can suspension cross native Rust callbacks? | **No** — `CannotYieldAcrossNativeFrame` |
| Is the root itself a Fiber? | **Yes** |
| Is raw `Fiber` scheduler-owned? | **No** |
| Is there a scheduler? | **Yes, minimal FIFO ready queue** |
| Is `Future.async` implemented? | **Yes** |
| Is `Future.await` implemented? | **Yes** |
| Does `await` suspend a non-root Fiber? | **Yes** |
| Can root `await` suspend normally? | **No; it pumps the scheduler itself** |
| Is concurrency structured? | **No** |
| Do Fibers have lifecycle parents? | **No** |
| Do Fibers have dynamic resumers? | **Yes** |
| Is failure contained per Fiber? | **Yes** |
| Does child failure automatically cancel sibling work? | **No** |
| Fiber cancellation? | **Not designed/implemented yet** |
| Future cancellation? | **Proposed, not ratified/landed** |
| Reactor/event poller? | **Specified, not implemented** |
| I/O-aware scheduler? | **Planned, not landed** |
| Preemption? | **No** |
| Multiple Phalcom Fibers in parallel? | **No, intentionally** |
| Shared-memory threads? | **Explicitly rejected** |
| Planned parallelism model | **Isolates, if implemented** |

The current concurrency specification itself is explicit: concurrency is **cooperative and single-threaded**, and a Fiber runs until it yields, awaits, returns, or raises. Structured concurrency/cancellation scopes and scheduler fairness remain open.

---

# 2. What the current `Fiber` actually is

The runtime representation is quite good.

`phalcom-core/src/heap/fiber.rs` defines:

```rust
pub enum FiberStatus {
    Suspended,
    Running,
    Done,
    Failed,
}

pub enum FiberResumeMode {
    Call,
    Try,
}

pub struct FiberObject {
    pub stack: Vec<Value>,
    pub frames: Vec<CallFrame>,
    pub open_upvalues: BTreeMap<usize, ObjRef>,

    pub status: FiberStatus,
    pub resumer: Option<ObjRef>,
    pub result: Value,

    pub entry: Option<ObjRef>,
    pub started: bool,
    pub resume_slot: usize,
    pub floor_depth: usize,
    pub resume_mode: FiberResumeMode,

    pub checking: HashSet<ObjRef>,

    pub seq: u32,
    pub spawn_file: Option<Symbol>,
    pub spawn_line: u32,
}
```

That is a real execution context, not a “generator state” object. Each Fiber has its own operand stack, call-frame stack, open upvalues, execution status, result, resumer, and entry closure.

While a Fiber is running, those three main execution structures are temporarily represented by:

```rust
VM::stack
VM::frames
VM::open_upvalues
```

and `VM::current` identifies which Fiber owns them. When it suspends, the vectors move back into its `FiberObject`.

The transfer functions are essentially:

```rust
store_live_into(vm, old_fiber)
load_live_from(vm, new_fiber)
vm.current = new_fiber
```

using `mem::take`; execution stacks are not copied frame-by-frame.

That is a strong foundation for future concurrency work.

---

# 3. Is it stackful?

The exact answer needs a distinction because the repository itself uses “stackful” in two senses.

## At the Phalcom VM level: yes

Suppose:

```phalcom
a() {
    b()
}

b() {
    c()
}

c() {
    Fiber.yield(42)
}
```

Conceptually the suspended Fiber preserves:

```text
a
└── b
    └── c
        └── Fiber.yield
```

The caller does not have to manually rewrite `a`, `b`, and `c` into continuation objects.

The Fiber owns an actual `Vec<CallFrame>` representing its suspended Phalcom call stack. So from the language programmer's perspective, this is a **stackful/direct-style coroutine**.

That is the important semantic property.

## At the native-machine-stack level: deliberately no

ADR-0030 calls the current implementation “Lua-5.1 style” and explicitly rejected the alternative of allocating/capturing real native stacks. The current Fiber does **not** save arbitrary Rust frames that happen to be active underneath the VM.

That causes the current restriction:

```phalcom
Fiber.new {
    for (x in list) {
        Fiber.yield(x)
    }
}
```

works because `for` lowers to Phalcom VM control flow.

But:

```phalcom
Fiber.new {
    list.each |x| {
        Fiber.yield(x)
    }
}
```

can raise:

```text
CannotYieldAcrossNativeFrame
```

because the native implementation of `each` has called the user block through a re-entrant Rust call into the VM. Suspending would require preserving that Rust call frame too. The runtime deliberately refuses instead of corrupting execution.

So the precise classification should be:

> **Phalcom Fibers are VM-stackful, native-stackless coroutines.**

Or:

> **They preserve arbitrary Phalcom call stacks, but not arbitrary native implementation stacks.**

That is probably the most accurate terminology.

---

# 4. It is indeed very Lua-coroutine-like

ADR-0030 explicitly characterizes the design as a restricted Lua-5.1-style model.

The public mechanics are:

```phalcom
let f = Fiber.new {
    let x = Fiber.yield("first")
    Fiber.yield(x)
}

f.call()          // "first"
f.call("second")  // "second"
```

This has the classic coroutine handshake:

```text
resumer                        fiber

f.call()
   ───────────────────────────→ starts

                   Fiber.yield("first")
   ←───────────────────────────

f.call("second")
   ───────────────────────────→ yield expression returns "second"

                   Fiber.yield("second")
   ←───────────────────────────
```

The value passed into the second `call` becomes the return value of the suspended `Fiber.yield`.

That is substantially more coroutine-like than Go goroutines, Java virtual threads, or Erlang processes.

---

# 5. `call` is Phalcom's `resume`

There is no selector literally named `resume`.

Instead:

```phalcom
fiber.call()
fiber.call(value)
```

are resume.

And:

```phalcom
fiber.try()
fiber.try(value)
```

are the same resume operation under a different failure-delivery policy.

Internally both dispatch to:

```rust
fiber_resume(..., FiberResumeMode::Call)
fiber_resume(..., FiberResumeMode::Try)
```

`fiber_resume` validates Fiber state, parks the current Fiber, installs the current Fiber as the callee's `resumer`, restores or initializes the callee's execution stack, changes it to `Running`, changes `VM::current`, and sets the typed `switch_pending` flag.

This is a particularly important implementation detail:

```rust
callee.resumer = Some(resumer_ref);
```

The relationship is **dynamic**.

It is not:

```text
fiber.parent
```

It is:

```text
fiber.currentResumer
```

That distinction becomes central when we discuss structured concurrency.

---

# 6. There is no `transfer`

Ruby distinguishes `resume/yield` from certain forms of Fiber-to-Fiber transfer.

Phalcom doesn't expose that.

The only normal control graph is:

```text
A calls B
    ↓
B.resumer = A

B yields
    ↓
return to A
```

If B resumes C:

```text
A
└─resumes→ B
             └─resumes→ C
```

then:

```text
C.yield → B
B.yield → A
```

There is no operation equivalent to:

```phalcom
someUnrelatedFiber.transfer(...)
```

that abandons the current resumer relation and directly hands execution elsewhere.

I would keep it that way for concurrency.

A general `transfer` is useful for exotic coroutine/control-flow abstractions, but it works against structured lifetime reasoning. If Phalcom eventually exposes a distinct low-level `Coroutine`, `transfer` can be reconsidered there.

---

# 7. This reveals an important naming problem

The thing currently called `Fiber` is conceptually doing two jobs.

It is first a manually controlled coroutine:

```text
new
call
yield
try
```

But it is also the execution substrate used by scheduled concurrent work:

```text
Future.async
System.schedule
Future.await
```

Those are related, but not identical concepts.

I would describe the current stack as:

```text
              manual control
                   │
                   ▼
              Fiber coroutine
             /               \
      generator use       scheduler use
                               │
                               ▼
                            Future
```

The future concurrency architecture we discussed wants:

```text
Coroutine                    Fiber
manual control          scheduler-owned execution
resume/yield            spawn/park/wake/join/cancel
     │                         │
     └──────── common VM suspension machinery ───────┘
```

I think that distinction has become more—not less—compelling after inspecting the implementation.

You do not necessarily have to rename `Fiber` tomorrow. But conceptually, **today's `Fiber.call/Fiber.yield` protocol is a coroutine protocol**, whereas the future scheduler needs a Fiber execution/lifecycle protocol.

---

# 8. Current Fiber state machine

Today:

```text
               Fiber.new
                   │
                   ▼
              Suspended
                   │
          call / try
                   │
                   ▼
                Running
              /    |    \
             /     |     \
         yield   return   uncaught error
           │       │           │
           ▼       ▼           ▼
      Suspended   Done        Failed
           │
        call/try
           │
           └──────────────→ Running
```

This is perfectly adequate for coroutines.

For scheduler-managed execution, it is missing meaningful states:

```text
Created
Runnable
Running
Parked
Cancelling
Completed
Failed
Cancelled
```

In particular, today's `Suspended` conflates:

- newly created;
- generator-yielded;
- scheduler-waiting;
- potentially externally parked.

Those states have substantially different lifecycle meanings.

---

# 9. Current scheduler: real, but very small

`VM` contains:

```rust
pub(crate) ready_queue: VecDeque<ObjRef>
```

It is explicitly a FIFO queue of scheduled Fibers.

And `System.schedule` does essentially:

```rust
let fiber = existing_fiber_or_wrap_function(args[0]);
vm.ready_queue.push_back(fiber);
return fiber;
```

while:

```phalcom
System.nextScheduled
```

pops the next one.

The library-side scheduler drain is:

```phalcom
@class
runScheduled() {
    let next = System.nextScheduled

    while (next.isSome) {
        let f = next.unwrapOr(None)
        f.try()
        next = System.nextScheduled
    }
}
```

The deliberate `try()` is significant: one scheduled Fiber's uncaught error is captured instead of terminating scheduler processing for unrelated scheduled work.

So Phalcom definitely already has a scheduler.

But this is still much closer to:

> **“a ready queue and pump”**

than to:

> **“a scheduler as the universal owner of executable Fiber state.”**

---

# 10. The unusual root-drive architecture

An earlier scheduler design contemplated running main under a scheduler root Fiber.

That was later superseded by the currently implemented **root-drive pump**.

The normal root program runs directly.

After root execution, the VM drains scheduled work; during execution, library code can explicitly call `System.runScheduled`.

This produced a visible semantic asymmetry in `Future.await`.

A normal non-root Fiber can do:

```phalcom
_waiters._$push(Fiber.current)
Fiber.yield(None)
```

because it has a `resumer`.

But the root Fiber has no resumer.

So root `await` instead does:

```phalcom
while (not self.isReady) {
    const next = System.nextScheduled

    if (next.isNone) {
        return Error.new(
            "await: the future is still pending and the scheduler is empty; nothing can settle it"
        ).raise()
    }

    const f = next.unwrapOr(None)
    f.try()
}
```

That is in the actual `Future.await` implementation.

This is one of the strongest signals that the current machinery needs to evolve.

---

# 11. `yield` and `park` are currently conflated

This deserves emphasis.

Coroutine yield means:

> “Return control to the Fiber that resumed me.”

Scheduler parking means:

> “I cannot currently make progress. Record what I am waiting for and let the scheduler run any appropriate runnable Fiber.”

Those are not the same operation.

Current `Future.await` implements scheduler waiting with coroutine yielding:

```phalcom
_waiters._$push(Fiber.current)
Fiber.yield(None)
```

So the path is approximately:

```text
worker fiber
    │
    │ await Future
    ▼
register as waiter
    │
    ▼
Fiber.yield
    │
    ▼
its dynamic resumer
    │
    ▼
scheduler pump
```

A proper scheduler-oriented primitive would instead be:

```text
worker fiber
    │
    │ await Future
    ▼
WaitRegistration
    │
    ▼
park(current)
    │
    ▼
Scheduler
    │
    ├── choose some runnable fiber
    │
    └── later wake(current)
```

Then the root Fiber is not special.

The root can park just like everyone else.

This is the biggest *mechanical* gap in the current scheduler.

---

# 12. Current `Future` is entirely implemented in Phalcom

This part is elegant and worth preserving.

`Future` is not a new VM object variant. Its state is ordinary Phalcom state:

```phalcom
_state
_value
_waiters
```

with states:

```text
pending
fulfilled
rejected
```

Construction:

```phalcom
class Future {
    @constructor
    new() {
        _state = "pending"
        _value = None
        _waiters = List.new()
    }
}
```

Settlement is settle-once:

```phalcom
settleValue(_ v) {
    if (self.isReady) {
        return self
    }

    _state = "fulfilled"
    _value = v
    self.drain()
    return self
}
```

and similarly for rejection.

That decomposition is good:

```text
VM primitive:
    Fiber

small native seam:
    ready queue

Phalcom:
    Future
```

I would retain that philosophy.

---

# 13. But current `Future.async` owns execution creation

Current code is roughly:

```phalcom
@class
async(_ action) {
    const f = Future.new()

    const driver = Fiber.new || {
        const fib = Fiber.new(action)
        const res = fib.try()

        if (fib.error.isSome) {
            f.settleError(fib.error.unwrapOr(None))
        } else {
            f.settleValue(res)
        }
    }

    System.schedule(driver)
    return f
}
```

So `Future.async`:

1. allocates Future;
2. creates a scheduler driver Fiber;
3. driver creates another Fiber for the user's action;
4. calls the user Fiber via `try`;
5. translates its terminal status into Future settlement.

Graphically:

```text
Future.async(action)
        │
        ├──── Future F
        │
        └──── Driver Fiber
                 │
                 └── Action Fiber
                         │
                         ├── success → F.fulfilled
                         └── failure → F.rejected
```

This is functional, but it is stronger machinery than necessary.

In the model I proposed:

```text
scope.spawn(action)
        │
        ├──── Fiber
        └──── Future/result view
```

one execution object owns the body.

The Future is merely the read capability for its eventual outcome.

---

# 14. Future's current waiter model

`_waiters` currently contains heterogeneous things:

```text
Fiber
or
Closure
```

A Fiber comes from:

```phalcom
future.await
```

A closure comes from:

```phalcom
future.then(...)
future.map(...)
future.catch(...)
```

When Future settles:

```phalcom
drain() {
    _waiters.each |w| {
        const dead = w.isA(Fiber) and w.isDone

        if (not dead) {
            System.schedule(w)
        }
    }

    _waiters = List.new()
}
```

`System.schedule` happens to accept either an existing Fiber or a function it wraps as a Fiber.

This is clever for a minimal implementation.

It is not the abstraction I would retain as concurrency grows.

Eventually there should be something closer to:

```text
WaitRegistration
    fiber
    waitable
    state
    wake reason
    cancellation registration
```

with a scheduler-owned wake path.

That machinery would be reused by:

- Future;
- Channel;
- Mutex;
- Semaphore;
- timers;
- sockets;
- process waits;
- cancellation;
- select/race.

---

# 15. There is a timing/reentrancy inconsistency in `Future.then`

This is a real semantic rough edge.

If Future has already settled, the current implementation can execute the continuation immediately:

```phalcom
if (self.isReady) {
    if (_state == "fulfilled") {
        return Future.flatten(f.call(_value))
    }
}
```

But if it is pending when `then` is registered, the continuation is put into `_waiters` and later scheduled.

So:

```phalcom
f.then(handler)
```

can mean either:

```text
handler executes inline before `then` returns
```

or:

```text
handler executes on a later scheduler turn
```

depending solely on whether settlement won a timing race.

That is exactly the sort of reentrancy nondeterminism a concurrency API should eliminate.

I would normalize this.

Either:

- continuations are always scheduled; or
- Phalcom de-emphasizes Future combinators and uses direct Fiber waiting as the normal style.

I prefer the latter plus **always-scheduled** continuation callbacks for consistency.

---

# 16. Is the current concurrency scheduler-aware?

The nuanced answer is:

> **Future is scheduler-aware. Raw Fiber is not scheduler-owned.**

A raw Fiber:

```phalcom
const f = Fiber.new || { ... }
f.call()
```

requires no scheduler at all.

The scheduler is layered over Fibers using:

```phalcom
System.schedule(...)
System.nextScheduled
System.runScheduled()
```

Future then builds asynchronous behavior over that queue.

So:

```text
Fiber
   │
   │ manually resumable
   │
   ├───────────────┐
   │               │
   ▼               ▼
manual coroutine   scheduler queue
                       │
                       ▼
                     Future
```

This is architecturally coherent for the current small system.

The future model should move toward:

```text
              Scheduler
                  │
        ┌─────────┴─────────┐
        │                   │
      Fiber              Reactor
        │                   │
        └──── park/wake ─────┘
        │
      Future
   observation only
```

---

# 17. Is current concurrency structured?

No.

And this is not an ambiguous judgment: the current specification explicitly leaves structured concurrency/cancellation scopes open.

The current `FiberObject` contains:

```rust
resumer: Option<ObjRef>
```

but no:

```text
parent
owner scope
children
cancellation context
supervisor
```

It does record spawn metadata:

```text
seq
spawn_file
spawn_line
```

and the spawn path computes a `parent_seq` for tracing. But that is observability, not ownership.

This distinction is essential.

## Current relationship

```text
A resumes B
```

causes:

```text
B.resumer = A
```

That means:

> “If B yields or returns, give control back to A.”

It does **not** mean:

> “A owns B's lifetime.”

A Fiber can have different resumers across different resumptions.

A structured parent cannot.

So `resumer` cannot simply be renamed `parent`.

---

# 18. What structured concurrency is currently missing

Consider:

```phalcom
const a = Future.async || { operationA() }
const b = Future.async || { operationB() }

a.await
```

Nothing currently says:

- who owns `b`;
- whether the enclosing operation may return before `b`;
- whether failure of `a` cancels `b`;
- whether failure of `b` must be observed;
- whether cancellation of the outer operation propagates;
- where simultaneous failures are aggregated.

Those are exactly the questions structured concurrency answers.

The target should instead impose:

```phalcom
Fiber.scoped |scope| {
    const a = scope.spawn || { operationA() }
    const b = scope.spawn || { operationB() }

    use(a.value, b.value)
}
```

with the invariant:

> The `Fiber.scoped` call cannot successfully return while `a` or `b` is still alive.

That property does not exist today.

---

# 19. Current error handling inside an individual Fiber is already strong

This is one area where I would change relatively little.

Phalcom has two failure channels:

```text
exceptional failure → Error / throw
expected local outcome → Result / Option
```

`throw` is terminating stack unwinding.

`on`, `catch`, and `ensure` are built over the block protocol.

The spec explicitly says `ensure` participates in all unwind paths, including throw, non-local return, and Fiber abort.

Inside a Fiber:

```phalcom
const f = Fiber.new || {
    try {
        risky()
    } catch e {
        recover(e)
    } ensure {
        cleanup()
    }
}
```

behaves as normal lexical exception handling.

That is the correct abstraction.

Fiber boundaries should not change ordinary error semantics inside a Fiber.

---

# 20. What happens when an Error escapes a non-root Fiber?

A Fiber has a hard execution boundary: its entry activation.

An uncaught Error reaching that boundary changes the Fiber to:

```text
Failed
```

and stores the captured Error in:

```rust
FiberObject::result
```

The current runtime distinguishes how that Fiber had been resumed.

### `fiber.call()`

Means:

> If the Fiber fails uncaught, propagate the failure into me.

### `fiber.try()`

Means:

> If the Fiber fails uncaught, give me the failure as a value.

That distinction is represented explicitly by:

```rust
pub enum FiberResumeMode {
    Call,
    Try,
}
```

on the callee.

This is an unusually nice low-level coroutine API.

---

# 21. `call` failure propagation

Conceptually:

```phalcom
let child = Fiber.new || {
    throw Error.new("boom")
}

child.call()
System.print("never")
```

behaves as though the failure escaped through the `call`.

If there is a chain:

```text
root
  └─calls→ A
            └─calls→ B
                       └─throws boom
```

and every edge is `call`, the failure propagates along that dynamic resumer chain.

The current concurrency spec states that the entry error reaches the Fiber floor, the Fiber becomes `failed`, and under `call` the error is re-raised through the cascade.

This is a **coroutine-call-stack policy**.

It is not structured child-failure propagation.

That distinction matters.

---

# 22. `try` creates a failure firewall

With:

```phalcom
const child = Fiber.new || {
    throw Error.new("boom")
}

const result = child.try()
```

the error becomes the delivered result.

The calling Fiber continues.

That is represented by `FiberResumeMode::Try`.

The Fiber itself remains terminal:

```text
child.status = Failed
child.error   = Some(error)
```

This is a sensible low-level primitive.

---

# 23. Current error containment is therefore dynamic

Suppose:

```text
Root
  │
  │ try
  ▼
A
  │
  │ call
  ▼
B
  │
  │ call
  ▼
C
   throws
```

Then conceptually:

```text
C fails
↓
B fails through call
↓
A sees failure through its try boundary
↓
Root survives
```

That gives low-level coroutine users precise control.

But notice what determines failure ownership:

```text
which resumer happened to use call/try?
```

rather than:

```text
which structured scope owns this computation?
```

That is appropriate for `Coroutine`.

It is not sufficient for a full concurrency system.

---

# 24. Root Fiber error semantics

Every VM starts with a root Fiber.

The root is created as:

```text
Running
started = true
entry = None
resumer = None
seq = 1
```



The lack of a resumer has three current consequences.

### Root cannot `yield`

`Fiber.yield` checks:

```rust
let Some(resumer) = fiber.resumer else {
    return Err("cannot yield the root fiber")
}
```



### Root cannot `Fiber.abort`

The primitive currently returns:

```text
cannot abort the root fiber
```

because there is no resumer into which its fiber-floor failure can be delivered.

### Unhandled root Error terminates the run

There is no higher Fiber boundary.

That is basically right: an unhandled root failure should eventually become process failure.

But in a structured runtime it should first drive structured shutdown.

---

# 25. How root failure should eventually work

I would evolve:

```text
root throws
→ runtime ends
```

into:

```text
root throws E
        │
        ▼
record root failure
        │
        ▼
cancel root scope
        │
        ▼
request cancellation of remaining descendants
        │
        ▼
wait for cleanup / ensure
        │
        ▼
aggregate any shutdown failures
        │
        ▼
render root failure + related failures
        │
        ▼
process exits unsuccessfully
```

The root Fiber should be owned by a root runtime scope.

That gives the top of the tree exactly the same ownership semantics as every lower structured scope.

---

# 26. A historical Fiber failure bug has already taught an important lesson

There was a serious bug in Fiber-floor failure teardown: escaping closures could retain `Upvalue::Open` references into a Fiber whose stack had been discarded.

That was not hypothetical; it could crash the VM.

But it is **fixed** as of commit `a265684` on July 20, 2026. The fix closes both the originating Fiber's live upvalues and intermediate call-mode resumers' parked upvalues before their stacks are destroyed. Regression tests cover both cases.

Current `dispatch.rs` now has both:

```rust
close_upvalues_from(...)
```

for the current live Fiber and:

```rust
close_fiber_upvalues_from(fiber_ref, ...)
```

for parked Fiber state.

This matters architecturally.

It demonstrates that **terminal Fiber transitions are resource/unwind operations**, not merely:

```text
status = Failed
stack.clear()
```

Any future cancellation mechanism must run through similarly rigorous unwind machinery.

That strongly supports the earlier recommendation:

> Cancellation must not be arbitrary Fiber destruction.

---

# 27. `Fiber.abort` deserves reconsideration

Current:

```phalcom
Fiber.abort(value)
```

is implemented as a fiber-floor raise.

The primitive itself accepts a generic `Value`, whereas normal `throw` is specified as only accepting `Error` subclasses.

That gives Phalcom two subtly different failure surfaces:

```text
throw Error
Fiber.abort(any Value)
```

I don't think this should become part of the future scheduler model.

For low-level coroutine compatibility, `abort` may be acceptable.

For concurrency, we need a cleaner partition:

```text
throw Error
    ordinary exceptional failure

Cancellation
    structured cooperative control

return
    normal Fiber completion
```

No concurrency subsystem should use `Fiber.abort` as cancellation.

---

# 28. `Future.async` failure handling today

Current `Future.async` uses:

```phalcom
const fib = Fiber.new(action)
const res = fib.try()

if (fib.error.isSome) {
    f.settleError(fib.error.unwrapOr(None))
} else {
    f.settleValue(res)
}
```

Thus:

```text
action throws Error
        │
        ▼
action Fiber = Failed
        │
        ▼
driver's `try` captures it
        │
        ▼
Future = rejected(error)
```



Then `Future.await` does:

```phalcom
if (_state == "rejected") {
    return _value.raise()
}
```

So:

```text
producer Fiber error
        ↓
Future rejection
        ↓
await
        ↓
Error raised in consumer Fiber
```

This gives a fairly intuitive surface.

But it loses a distinction I would preserve in the future:

```text
the producer failed
```

versus:

```text
the current waiter is being cancelled
```

These should not be represented as the same kind of control transfer.

---

# 29. My recommended terminal model

Rather than overloading status/result/error fields, define the semantic result of every execution Fiber as:

```text
Outcome<T>
    = Success(T)
    | Failure(Error)
    | Cancelled(CancelReason)
```

Then:

```phalcom
fiber.join
```

would return:

```text
Outcome<T>
```

whereas:

```phalcom
fiber.value
```

would effectively be:

```text
Success(v)   → v
Failure(e)   → throw e
Cancelled(r) → report target cancellation
```

Similarly:

```phalcom
future.wait
future.value
future.peek
```

can expose different levels of convenience.

This cleanly separates:

```text
Fiber terminal state
```

from:

```text
what the observer wants to do with it.
```

Today `call`/`try` combine those concerns because they are coroutine-control messages.

---

# 30. This suggests keeping `call/try` on the coroutine side

A scheduler-managed Fiber normally should not be “resumed” by random application code.

This:

```phalcom
fiber.call()
```

means:

> “I decide that you run now, and I become your resumer.”

That is perfect coroutine semantics.

It is wrong scheduler semantics.

For concurrent Fibers, application code should instead do:

```text
scope.spawn
fiber.join
fiber.value
fiber.cancel
```

The scheduler alone decides when that Fiber resumes.

So the long-term semantic split should be:

| Coroutine concept | Concurrent Fiber concept |
|---|---|
| `call` / resume | scheduler dispatch |
| `yield(value)` | scheduler park/yield |
| dynamic `resumer` | fixed owning `FiberScope` |
| `try` | `join -> Outcome` |
| `abort` | ordinary throw / structured cancellation |
| yielded value | not applicable |
| manually selected next coroutine | scheduler selects next runnable Fiber |

They can share 90% of VM stack machinery without being the same public abstraction.

---

# 31. Current Future cancellation: proposed, not current behavior

There is a cancellation design in the repository, but it is still **Proposed**.

PDR-0017 proposes:

```phalcom
Future#cancel
Future#isCancelled
```

with cancellation meaning **renunciation**:

- settle the Future rejected immediately;
- release reactor registration;
- best-effort suppress work that hasn't started;
- never interrupt work that already started;
- `Future.async` producer continues running;
- late settlement gets dropped;
- cancellation is shallow;
- no Fiber cancellation;
- no structured propagation.

The accompanying cancellation spec explicitly leaves structured propagation and Fiber cancellation as open questions.

I would **not ratify this surface unchanged**.

The reactor mechanics behind it are good.

The authority model is not.

---

# 32. Why `Future#cancel` is the wrong long-term authority

Imagine:

```text
producer Fiber
      │
      ▼
   Future F
    /      \
   /        \
Alice      Bob
```

Both Alice and Bob possess `F`.

If:

```phalcom
F.cancel()
```

settles the shared Future cancelled, Alice can cancel Bob's observation too.

Yet possession of:

```text
the right to observe a result
```

does not logically imply:

```text
the right to cancel the underlying operation.
```

This problem becomes more severe once structured concurrency exists.

My previous recommendation stands strongly here:

```text
Future<T>
    result observation capability

CancellationToken
    cancellation observation capability

CancellationSource / FiberScope
    cancellation authority
```

The excellent existing reactor design around generation-tagged registrations and stale-completion suppression can remain exactly the implementation substrate. Only the public authority model needs changing.

---

# 33. “Stop waiting” and “stop producing” need distinct operations

The current proposed `Future#cancel` calls itself renunciation, but settles the shared Future.

A cleaner model distinguishes:

### Cancel my wait

```text
stop this Fiber waiting for F
```

No effect on F.

### Cancel this operation

```text
request cancellation through the operation's CancellationSource/scope
```

May affect producer and descendants.

### Cancel this structured scope

```text
request cancellation of its entire child tree
```

### Producer eventually completes anyway

Possible for operations that cannot actually be interrupted.

That decomposition maps much better onto the reactor realities the current PDR already discovered.

---

# 34. The planned reactor is architecturally strong

Although it is not yet implemented, `docs/spec/current/stdlib/reactor.md` contains several good decisions.

It divides external completion into:

```text
pollable:
    sockets
    pipes
    TTY
    timers
    signals
        ↓
    event poller

inherently blocking:
    filesystem operations
        ↓
    bounded worker pool
```

Workers are forbidden from carrying:

```text
Value
ObjRef
Heap
VM
```

They exchange owned plain data only.

Completion returns to the VM thread, and only that thread can settle a Future or modify the ready queue.

This follows directly from PDR-0003's single-owner VM architecture.

I strongly agree with that.

---

# 35. The reactor specification already contains the future scheduler's central liveness law

The spec says that when:

```text
ready queue empty
but pending I/O exists
```

the runtime must **not exit**.

It should block in the reactor until something completes.

The lifecycle is already expressed as:

```text
submit
   ↓
park
   ↓
complete
   ↓
drain at VM safepoint
   ↓
settle
   ↓
ready
```



That is exactly the right spine.

What is missing is making `park` and `ready` true scheduler concepts instead of emulating parking via `Fiber.yield`.

---

# 36. Reactor implementation status

The reactor is not currently landed.

A repository search for the runtime seams such as `reactor.rs`, `System.sleep`, completion parking, and completion draining returned the reactor specification and pending implementation specification rather than runtime source.

So we should distinguish:

```text
Fiber                    IMPLEMENTED
ready queue              IMPLEMENTED
Future                   IMPLEMENTED
Future.await             IMPLEMENTED
Future.async             IMPLEMENTED

reactor                  SPECIFIED
timers                   SPECIFIED
async file workers       SPECIFIED
socket polling           SPECIFIED
registration cancel      SPECIFIED / proposed
```

---

# 37. Current parallelism decision changes our earlier architecture

This is the largest divergence from my earlier proposal.

I previously recommended approximately:

```text
M Fibers
  ↓
N worker OS threads
  ↓
shared Phalcom object graph
```

That does **not** fit current Phalcom.

PDR-0003 explicitly says:

- one VM owns one Heap;
- `ObjRef` is meaningful only relative to that VM;
- GC root enumeration is per VM;
- `SlotMap`/heap ownership is not `Sync`;
- there is no `Thread`;
- no `Mutex`;
- no `Atomic`;
- no shared-object `spawn`;
- runtime worker threads may not touch Phalcom Values or objects;
- future CPU parallelism should use isolates.

Changing that would require reworking:

```text
Heap ownership
GC
Value / ObjRef assumptions
native primitives taking &mut VM
object mutation semantics
safepoints
upvalues
class mutation
inline caches
memory ordering
synchronization
FFI assumptions
```

So M:N is not “Phase 2 scheduler work.”

It is effectively:

> **a different VM architecture.**

I would not do it.

---

# 38. Better target: M:N at the isolate level, not Fiber level

Phalcom can still obtain multi-core parallelism later:

```text
                         Process
                            │
          ┌─────────────────┼─────────────────┐
          │                 │                 │
        Isolate A         Isolate B         Isolate C
        VM / Heap         VM / Heap         VM / Heap
          │                 │                 │
       Scheduler         Scheduler         Scheduler
       /  |  \           /  |  \           /  |  \
     Fibers...          Fibers...          Fibers...
```

Each isolate can execute on its own OS thread/core.

Inside an isolate:

```text
objects shared freely
Fibers extremely cheap
single-threaded semantics
no object locks
```

Between isolates:

```text
copy / transfer / serialization / immutable payload
```

That has a lot of attractive properties for Phalcom.

It preserves the simple object model while still allowing CPU parallelism.

---

# 39. Cooperative scheduling remains a separate question from parallelism

Even on one OS thread, Phalcom could eventually add **preemptive Fiber time slicing**.

Today:

```text
Fiber runs until:
    yield
    await
    return
    raise
```

A CPU loop:

```phalcom
while (true) {
    crunch()
}
```

can starve every other Fiber indefinitely. The accepted concurrency design makes that explicit.

A future scheduler could introduce safepoint preemption:

```text
bytecode budget exhausted
        ↓
current Running → Runnable
        ↓
push at back of ready queue
        ↓
run another Fiber
```

while still remaining:

```text
one OS thread
one VM
no races
```

That is a much smaller architectural decision than shared-memory parallelism.

However, it changes one useful existing semantic invariant: execution no longer switches only at explicit suspension points.

I would defer it until there is evidence CPU starvation is a practical problem.

---

# 40. Full comparison: current model vs proposed target

| Area | Current Phalcom | Recommended mature model | Gap |
|---|---|---|---|
| Execution representation | Independent VM stacks | Same | Very small |
| Native stack capture | No | Prefer no | None |
| Coroutine resume | `call` / `try` | Keep on low-level Coroutine | Naming/role |
| Coroutine yield | `Fiber.yield` | Keep on low-level Coroutine | Naming/role |
| Direct transfer | No | Probably still no | None |
| Execution ownership | Dynamic `resumer` | Fixed `FiberScope` owner | **Major** |
| Scheduler | FIFO `VecDeque` | Runtime scheduler object/policy | Medium |
| Parking | `Fiber.yield` to resumer | `Scheduler.park(current, wait)` | **Major** |
| Waking | reschedule waiter Fiber | atomic WaitRegistration/Waker | Major |
| Root | Cannot yield; special await pump | Ordinary scheduler-managed Fiber | Major |
| Spawn | `System.schedule`, `Future.async` | `scope.spawn` | Major |
| Future | Active async facade | Passive result capability | Medium |
| Future wait | `await` | direct `wait/value` suspension | Small/medium |
| Future continuations | inline if settled, scheduled if pending | consistent scheduling | Medium |
| Failure inside Fiber | terminating `throw`/catch/ensure | Same | None |
| Fiber terminal failure | `Failed + Error` | `Outcome.Failure` | Small |
| Cross-Fiber failure | `call` cascade / `try` capture | scope failure policy | Major |
| Multiple failures | No structured aggregation | ErrorGroup/ConcurrentError | Major |
| Cancellation | proposed Future renunciation | scope/token cooperative cancellation | **Major** |
| Cancellation hierarchy | None | parent → descendants | Major |
| Deadlines | Not implemented | nested cancellation deadlines | Major |
| Reactor | specified | implement it | Major implementation gap |
| I/O worker pool | specified | same model | Implementation gap |
| Channels | None | first-class eventually | Medium |
| Mutex | Intentionally unnecessary single-threaded | still usually unnecessary | Low |
| Select/race | Open | generalized Wait/select | Major |
| Backpressure | None native | Channel/Semaphore/resource pools | Medium |
| Structured concurrency | None | foundational | **Largest semantic gap** |
| Supervision | None | optional long-lived layer | Later |
| Scheduler fairness | Open | documented weak fairness | Medium |
| Preemption | None | optional later | Deliberate |
| Shared-memory parallelism | Explicitly rejected | don't add | None |
| CPU parallelism | None | isolates | Future feature |
| Virtual-time scheduler | None | testing scheduler | Medium |
| Deadlock/wait diagnostics | limited tracing | wait graph | Later |
| Fiber observability | IDs/spawn location/tracing already present | extend tree/wait state | Good foundation |

---

# 41. The strongest parts of the current design

I would preserve these essentially intact.

## The per-Fiber VM stack representation

This is excellent.

No need to move to native stack switching.

## `VM::current`

Exactly the right fundamental identity.

## Typed `switch_pending`

A dedicated Fiber-switch signal is much safer than inferring control transfer from stack-length changes.

## Per-Fiber open upvalues

Absolutely required and already thought through carefully.

## Root itself being a Fiber

Correct abstraction; only its scheduler relationship needs improvement.

## Fiber-local error unwinding

Correct.

## Failure capture at Fiber boundaries

Also correct as a low-level mechanism.

## Pure-Phalcom Future

A good object-model choice.

## Tiny native scheduler seam

Good architectural discipline.

## Generation-tagged reactor registrations

Strong future cancellation substrate.

## Single-thread ownership of Phalcom heap

Given the existing runtime, this is a very rational choice.

---

# 42. The biggest design debts

I'd rank them this way.

### 1. No structured ownership

This is the largest semantic hole.

### 2. Scheduler parking is implemented through coroutine yielding

This is the largest mechanical hole.

### 3. Root scheduling is exceptional

Mostly a consequence of #2.

### 4. `Future.async` owns both execution creation and result representation

This becomes awkward once scopes exist.

### 5. Cancellation authority is unresolved

And the currently proposed Future-level solution should be reconsidered before ratification.

### 6. No generalized waiter model

Future's heterogeneous waiter list cannot scale cleanly to channels, timers, locks, I/O and select.

### 7. Reactor not yet landed

So the scheduler can only make progress from already queued in-process work.

### 8. Coroutine and concurrent-Fiber semantics currently share one public class

Manageable today, but increasingly problematic.

### 9. Continuation timing depends on Future state

Potential reentrancy hazard.

### 10. Scheduler fairness/liveness beyond the future reactor design remains weakly specified

The reactor document improves this considerably, but it is not yet the actual general scheduler contract.

---

# 43. The architecture I would now recommend for Phalcom

Taking the actual repository into account, I would revise the target to this:

```text
                           Runtime
                              │
                         RootScope
                              │
                    ┌─────────┴─────────┐
                    │                   │
                  Fiber               Fiber
                    │                   │
                  Future              Future
               result view         result view
                    │                   │
                    └─────────┬─────────┘
                              │
                           Scheduler
                              │
                ┌─────────────┼──────────────┐
                │             │              │
           Ready Queue      Reactor      Timer Queue
                │             │              │
                └─────────────┴──────────────┘
                              │
                         one VM thread
                              │
                         one object heap
```

And then outside the VM:

```text
                  multiple isolates
             ┌────────┼────────┐
             ▼        ▼        ▼
           VM A      VM B      VM C
```

That is a much better fit than putting multiple OS workers underneath one VM.

---

# 44. Introduce a true scheduler park/wake protocol

This should be the next important VM-level concurrency abstraction.

Something internally equivalent to:

```rust
park_current(wait_registration)
wake(fiber, reason)
yield_current()
```

The first operation should:

```text
Running
   ↓
Parked
```

without consulting `resumer`.

`wake`:

```text
Parked
   ↓
Runnable
   ↓
ready_queue
```

Then `Future.await` becomes conceptually:

```phalcom
await {
    if (not self.isReady) {
        self.registerWaiter(Fiber.current)
        Scheduler.parkCurrent()
    }

    return self.unwrap()
}
```

No:

```phalcom
Fiber.current.isRoot
```

No root-only scheduler pump.

No dependence on who resumed this Fiber.

---

# 45. Put the scheduler outside the dynamic resumer chain

Current topology:

```text
scheduler pump
     │
     └─ resumes worker
            │
            └─ worker Fiber.yield
                    │
                    └─ returns to scheduler's caller
```

Target:

```text
                Scheduler
              /     |      \
             /      |       \
         Fiber A  Fiber B  Fiber C
             │
          park/wake
             │
           Waiter
```

The scheduler should not need to masquerade as the resumer of every concurrent Fiber.

`resumer` remains useful for Coroutines.

Scheduler ownership is a separate relationship.

---

# 46. Add `FiberScope`

This is the central semantic addition.

Conceptually:

```phalcom
Fiber.scoped |scope| {
    const user = scope.spawn || {
        users.find(id)
    }

    const posts = scope.spawn || {
        posts.forUser(id)
    }

    render(user.value, posts.value)
}
```

The scope owns:

```text
children
failure policy
cancellation context
deadline
```

and enforces:

```text
scope cannot exit while a child is alive
```

The ownership graph becomes:

```text
RootScope
├── RequestScope
│   ├── User Fiber
│   └── Posts Fiber
└── Service Supervisor
```

That is very different from:

```text
resumer chain
```

Both relationships may exist simultaneously.

---

# 47. Keep resumer and owner as separate runtime concepts

For example:

```text
Fiber A owns B through Scope S

but Scheduler happens to run:
C → B → park → D
```

Control-flow history and ownership are different.

A future Fiber might therefore have internally:

```text
owner_scope: ScopeRef
```

and, only if it also participates in manual coroutine control:

```text
resumer: Option<FiberRef>
```

Never derive one from the other.

---

# 48. Recommended structured failure semantics

Default `FiberScope`:

```text
scope
├── A
├── B
└── C
```

B throws:

```text
B → Failure(E)
        │
        ▼
scope records E
        │
        ├── cancellation request → A
        └── cancellation request → C
        │
        ▼
wait for A and C cleanup
        │
        ▼
propagate E
```

If A also fails while shutting down:

```text
ConcurrentError([
    B's original failure,
    A's cleanup/failure
])
```

Cancellation of siblings because B failed is not itself another failure.

This is the major semantic upgrade over current dynamic `call` cascades.

---

# 49. Keep `Fiber#try`, but treat it as low-level coroutine behavior

It remains useful.

For a manual coroutine:

```phalcom
fiber.try()
```

means:

> resume and contain its failure.

For structured concurrency:

```phalcom
fiber.join
```

should be the analogous operation.

That gives a clean split:

```text
Coroutine:
    call
    try
    yield

Concurrent Fiber:
    join
    value
```

---

# 50. Cancellation target after repository inspection

I would now make cancellation completely scope-oriented.

Something like:

```phalcom
Cancellation.withTimeout(2.seconds) || {
    Fiber.scoped |scope| {
        ...
    }
}
```

or:

```phalcom
scope.cancel()
```

with an explicit capability for cross-boundary cancellation:

```text
CancellationSource
CancellationToken
```

The existing reactor generation tokens remain implementation machinery, not the primary public cancellation abstraction.

Cancellation should be:

```text
sticky
cooperative
hierarchical
delivered at cancellation points
```

and should run ordinary `ensure` cleanup.

No arbitrary stack destruction.

The E002 history is a good reminder of why teardown cannot be hand-waved.

---

# 51. Cancellation should not be `Error`

Phalcom's normal handler:

```phalcom
catch e {
    ...
}
```

should not accidentally defeat shutdown.

I would make internal cancellation propagation a control signal parallel to the existing unwind machinery:

```text
Unwind
├── Return
├── Raise(Error)
└── Cancel(CancelReason)
```

Then:

```text
ensure
```

runs for all three.

But:

```text
catch Error
```

matches only `Raise`.

That gives very clean semantics.

When observing *another Fiber's* cancelled outcome, however, cancellation becomes data:

```text
Outcome.Cancelled(reason)
```

The distinction is:

> **My cancellation is control flow. Another Fiber's cancellation is an outcome I observe.**

---

# 52. How the root should fit structured cancellation

Eventually:

```text
Runtime
  │
  └── RootScope
       │
       └── Root Fiber
```

An external shutdown request:

```text
cancel RootScope
```

causes:

```text
all application fibers receive cooperative cancellation
```

An uncaught root error:

```text
root Failure(E)
```

causes the same structured shutdown, except final process status is failure.

This eliminates several current one-off root semantics.

---

# 53. Future after structured concurrency

I would simplify Future considerably.

Today:

```phalcom
Future.async || { ... }
```

does both:

```text
spawn execution
create result object
```

Future should eventually become:

> **a passive observation capability.**

Execution comes from:

```phalcom
scope.spawn
```

For example:

```phalcom
const worker = scope.spawn || {
    compute()
}

const future = worker.future
```

or `spawn` itself could return a handle with Future-like result methods.

Future then needs approximately:

```text
isReady
peek
wait
value
```

and perhaps carefully defined continuation combinators.

It does not need to be the central concurrency API.

---

# 54. You may not even need a user-visible `Promise`

The previous architecture proposed `Promise<T>` as a low-level write capability.

Looking at Phalcom's current design, I would now keep that decision open.

Because Future is deliberately pure `.ph`, reactor-native operations can receive the Future object and have VM-side completion machinery eventually invoke settlement on the VM thread.

If the implementation can retain capability integrity without exposing public settlement methods, a public Promise may be unnecessary.

The conceptual read/write split is still useful:

```text
result observer
producer completion authority
```

but Phalcom need not expose both as general user types.

---

# 55. Future callback behavior should be normalized before the model grows

I would change:

```text
settled Future.then → callback inline
pending Future.then → callback scheduled later
```

to:

```text
Future.then → continuation always scheduled
```

or eventually de-emphasize callbacks altogether.

That removes an entire class of timing-sensitive reentrancy.

Do this before external I/O makes Future settlement timing genuinely nondeterministic.

---

# 56. Generalized `Wait` should replace heterogeneous waiters

Long term:

```text
Future.await
Channel.receive
Channel.send
Timer.sleep
Socket.read
Semaphore.acquire
CancellationToken.wait
```

should all compile conceptually to the same runtime operation:

```text
register waiter
park current Fiber
wake it exactly once
```

Then `select` is no longer a strange Future combinator.

It becomes:

```text
register N candidate waits
atomically commit one
withdraw the others
wake Fiber
```

This is substantially more robust.

---

# 57. Channels fit the current single-threaded architecture extremely well

One correction from my previous model: because Phalcom does not have shared-memory parallel Fibers inside one VM, it does not need Mutexes/Atomics as normal concurrency primitives.

Channels, however, remain extremely valuable:

```phalcom
const ch = Channel.new()

Fiber.scoped |scope| {
    scope.spawn || {
        ch.send(produce())
    }

    consume(ch.receive())
}
```

They provide:

- coordination;
- queues;
- pipelines;
- backpressure;
- actor mailboxes;
- select cases.

A zero-capacity rendezvous channel is still a compelling default.

---

# 58. Mutexes are probably unnecessary inside a Phalcom VM

This is an important change from my previous broad proposal.

With exactly one Fiber executing Phalcom bytecode at a time, ordinary code between suspension points is not concurrently executed.

There are no parallel writers.

So a traditional shared-memory mutex solves very little.

What Phalcom may eventually need instead are semantic synchronization objects such as:

```text
Semaphore
Channel
Once
Barrier
Condition/event
```

to coordinate Fiber progress.

If isolates do not share object graphs, Mutex remains unnecessary there too.

That is a substantial simplification.

---

# 59. The current single-thread model gives Phalcom a very useful atomicity property

Today:

```phalcom
balance = balance - amount
ledger.add(entry)
```

cannot be interleaved by another Fiber unless something inside those operations suspends.

This is extremely powerful for reasoning.

If suspension effects eventually become visible to the LSP/checker—as we discussed earlier—the tooling could effectively tell the programmer:

```text
this region cannot suspend
⇒ it is atomic with respect to other Fibers in this isolate
```

That is a beautiful language-level consequence of Phalcom's chosen concurrency architecture.

It may be worth preserving rather than chasing shared-memory parallelism.

---

# 60. Suspension effects become even more valuable in this actual model

Current Phalcom has an implicit semantic divide:

```text
non-suspending method
```

versus:

```text
method that may eventually invoke yield/await
```

Once the restricted native-frame problem is removed and scheduler parking becomes general, I would expose this through semantic tooling:

```text
@suspends
```

as inferred metadata rather than mandatory surface syntax.

Then LSP can explain:

```text
This send may suspend the current Fiber.
```

That is arguably more important in a cooperative single-thread model than in an M:N runtime because suspension points are the places where interleaving becomes observable.

---

# 61. The current native-frame restriction should eventually be removed

I would still pursue ADR-0033's general direction.

Not by adding native stacks.

Instead, continue de-recursing native callbacks so Phalcom code is driven through VM frames rather than nested Rust `run_until` calls.

Eventually:

```phalcom
list.each |x| {
    future.await
}
```

should be legal.

So should:

```phalcom
try {
    future.await
} ensure {
    cleanup()
}
```

without Fiber-switch restrictions leaking through implementation details.

The current guard is correct as a safety boundary, but it should not become permanent language philosophy.

---

# 62. Recommended migration path

## Phase 1 — Clarify the two abstractions

Specify explicitly:

```text
Coroutine control
    call / try / yield

Concurrent Fiber lifecycle
    spawn / park / wake / join
```

Decide whether they remain one class for compatibility or eventually split into `Coroutine` and `Fiber`.

Do not add `transfer`.

---

## Phase 2 — Introduce scheduler-level park/wake

Extend Fiber states.

Add scheduler-owned waiting.

Remove the semantic dependency between:

```text
await
```

and:

```text
Fiber.yield to resumer
```

This is the mechanical prerequisite for everything after it.

---

## Phase 3 — Make root scheduler-owned

The root remains a Fiber, but the scheduler becomes its execution host.

Then:

```phalcom
future.await
```

has exactly one implementation regardless of which Fiber invokes it.

Delete root-specific Future pumping.

---

## Phase 4 — Add `FiberScope`

Add:

```phalcom
Fiber.scoped
scope.spawn
```

with mandatory child completion on scope exit.

Establish fixed ownership separate from dynamic execution control.

---

## Phase 5 — Add terminal `Outcome`

Unify:

```text
success
failure
cancellation
```

as Fiber outcomes.

Keep ordinary `throw` semantics unchanged inside Fibers.

---

## Phase 6 — Add structured cancellation

Add:

```text
Cancel
CancellationSource
CancellationToken
deadline
timeout
shield
```

or the Phalcom-equivalent names.

Do not ratify `Future#cancel` as producer authority.

Reuse the reactor's already-designed registration-generation machinery.

---

## Phase 7 — Land the reactor

Implement the already-specified:

```text
poller
timer source
filesystem worker pool
completion channel
registration root set
safepoint completion drain
```

The new scheduler park/wake machinery should be what it targets.

---

## Phase 8 — Generalize waiting

Implement:

```text
WaitRegistration
select/race
Channel
Semaphore
```

with atomic cancellation/commit semantics.

---

## Phase 9 — Normalize Future

Move execution creation toward:

```phalcom
scope.spawn
```

rather than:

```phalcom
Future.async
```

Future becomes predominantly an eventual-result protocol.

Make continuation timing consistent.

---

## Phase 10 — Testing and observability

Phalcom already records useful Fiber sequence IDs and spawn locations.

Extend this into:

```text
Fiber tree
owner scope
park reason
wait object
deadline
cancellation state
creation stack
```

Then add:

```text
deterministic scheduler
virtual monotonic clock
schedule fuzzing
wait-cycle diagnostics
```

---

## Phase 11 — Isolates for real CPU parallelism

Only after the single-VM concurrency model is excellent.

Each isolate gets:

```text
VM
Heap
Scheduler
Reactor
RootScope
```

and can execute on another OS thread.

This preserves all the accepted heap/GC assumptions.

---

# 63. Proposed final conceptual architecture

I would now formulate Phalcom concurrency as five layers.

```text
┌──────────────────────────────────────────────────────────────┐
│  Structured concurrency                                     │
│  FiberScope · Supervisor · cancellation · deadlines         │
├──────────────────────────────────────────────────────────────┤
│  Concurrent execution                                       │
│  Fiber · Outcome · Future                                   │
├──────────────────────────────────────────────────────────────┤
│  Coordination                                               │
│  Wait · Channel · Semaphore · select                        │
├──────────────────────────────────────────────────────────────┤
│  Scheduling                                                 │
│  ready queue · park/wake · timer/reactor completion         │
├──────────────────────────────────────────────────────────────┤
│  VM continuation machinery                                  │
│  stack · frames · upvalues · current Fiber · switching      │
└──────────────────────────────────────────────────────────────┘
```

And beside—not inside—the concurrent Fiber surface:

```text
Coroutine
    resume/call
    try
    yield

implemented using the same VM continuation machinery.
```

For multi-core:

```text
Isolate
└── entire stack above
```

rather than putting multiple OS threads inside one stack.

---

# 64. The current model is closer to our desired model than it first appears

The foundation does **not** need to be thrown away.

In fact, several difficult things are already solved:

```text
✓ independent Fiber VM stacks
✓ stack preservation
✓ open-upvalue ownership
✓ current Fiber identity
✓ O(1)-style state switching
✓ root Fiber
✓ cross-Fiber return safety
✓ terminal Fiber states
✓ error containment
✓ call-vs-try propagation
✓ Future state machine
✓ Future waiter resumption
✓ FIFO ready queue
✓ spawn-site observability
✓ scheduler isolation of scheduled failures
✓ GC awareness of parked Fibers
✓ explicit native-frame suspension safety guard
✓ fixed Fiber-floor upvalue teardown
```

The missing pieces are mostly **ownership and scheduling semantics**, rather than basic coroutine machinery.

That is very good news for the future design.

---

# 65. But current `Fiber` is more Coroutine than Fiber

If I had to summarize the repository in one sentence, it would be:

> **Phalcom has already implemented a Lua-style VM coroutine and then built a small Future scheduler over it; the next architectural step is to turn that substrate into scheduler-owned structured Fibers rather than extending the coroutine protocol itself into a concurrency framework.**

That means I would resist adding things like:

```text
Fiber.transfer
more resume variants
more magical Future combinators
Future cancellation propagation
global spawn
```

Those deepen the current coroutine-centric direction.

Instead add:

```text
FiberScope
scheduler park/wake
Outcome
structured cancellation
WaitRegistration
reactor
Channel
Supervisor
```

---

# 66. One repository hygiene issue

`docs/guide/concurrency.md` is stale: it still says that Fiber and Future do not run yet, whereas the current spec and actual `heap/fiber.rs`, `primitive/fiber.rs`, `core.ph`, and scheduler primitives clearly show that both Fiber and Future slices have landed.

That should be corrected because concurrency semantics are now substantial enough that an outdated guide will actively mislead contributors.

---

# 67. Bottom line

The answer to your original questions is therefore:

**Is current Phalcom Fiber stackful?**

Yes in the important language sense: it owns and preserves an arbitrary Phalcom VM call stack. No in the native-stack-switching sense: it cannot preserve arbitrary Rust frames. The repository deliberately chose this restricted Lua-like architecture.

**Is it coroutine-like?**

Very much so. `call`/`try` are resume operations; `Fiber.yield` is bidirectional yield; `resumer` is dynamically assigned. It is closer to Lua/Ruby coroutine mechanics than to a Go goroutine or Loom thread.

**Does it have `transfer`?**

No, and I would not add it to the concurrent Fiber abstraction.

**Does it already schedule Fibers?**

Yes. There is a FIFO VM `ready_queue`, `System.schedule`, `System.nextScheduled`, `System.runScheduled`, a VM root-drive pump, and Future settlement reschedules waiting Fibers.

**Is it scheduler-aware?**

Future is. Raw Fiber is fundamentally manually resumed. `await` currently bridges the two by calling `Fiber.yield`, which is the seam I would replace with scheduler-level parking.

**Is it structured?**

No. There is no scope ownership, child tree, structured cancellation, scope join barrier, sibling failure policy, or failure aggregation. `resumer` is control-flow state, not ownership. The current specification explicitly leaves structured concurrency open.

**How do errors work today?**

Within a Fiber, ordinary Phalcom terminating error semantics apply, including `catch`/`on` and `ensure`. If the error reaches the Fiber floor, the Fiber becomes `Failed` and captures the error. A `call`-resumed failure propagates along the resumer chain; a `try`-resumed failure becomes a value. Future drivers translate Fiber failure into Future rejection; `await` re-raises that rejection in the waiting Fiber. Root uncaught failure exits the execution because root has no resumer.

**How should errors work eventually?**

Keep ordinary per-Fiber exception unwinding exactly as it is, but move concurrency failure propagation away from dynamic `call/try` chains and into structured scope ownership. Every concurrent Fiber should terminate with an `Outcome`; scopes should define fail-fast/supervision policy; cancellation should be a distinct cooperative unwind signal; the root scope should perform orderly shutdown before an unhandled root failure terminates the process.

**Is current Future the right long-term centerpiece?**

No. It is a useful result abstraction and the implementation is nicely lightweight, but `Future.async` currently creates execution and Future itself is drifting toward cancellation authority. The mature model should make `scope.spawn` create execution while Future remains predominantly an observation capability.

**Should we pursue the M:N shared-memory scheduler I previously proposed?**

Not under the current Phalcom architecture. The accepted single-owner heap design makes that a VM rewrite rather than a scheduler upgrade. Structured single-threaded Fibers plus parallel isolates is the more coherent path.

The most consequential next design exercise is therefore not “design more Fiber methods.” It is to define the exact boundary between **Coroutine and concurrent Fiber**, and then specify **`FiberScope`, scheduler `park/wake`, `Outcome`, cancellation unwinding, and scope failure algebra** together. Those five decisions determine almost everything else—and the existing VM is already surprisingly well-positioned to support them.