# 07 — Concurrency and Coroutines

Suspending a computation and resuming it elsewhere. The through-line: *whose stack is it,
and what is allowed to be on that stack when you leave.*

Questions first. Answers below. Do not scroll.

---

## Questions

### Q1 — Where the value comes from

```
let a = yield(11)
```

`yield` sends `11` out to whoever is driving this coroutine, and the coroutine parks.
Later someone resumes it with `100`, and `a` becomes `100`.

1. In a stack-based bytecode VM, what physical operation delivers `100` into `a`, and
   which *other*, entirely ordinary VM operation is it identical to?
2. Why does that identity mean the suspended code cannot detect that it was suspended?
3. The first resume of a coroutine is different: its argument binds to the entry
   function's parameter rather than to a `yield` expression's result. Explain that
   asymmetry without saying "because it hasn't started yet" — what is *physically*
   absent on the first resume?

### Q2 — Stackful versus stackless

Lua coroutines, Go goroutines, and Ruby fibers are **stackful**. C# `async`, Rust
`async fn`, and JS `async function` are **stackless**.

1. State the operational difference in one sentence about *where the suspended state
   lives*.
2. A stackless design cannot suspend from inside a helper function unless that helper is
   itself marked `async`. Why is that a *forced* consequence rather than a design choice
   someone could have avoided?
3. Give one thing stackless buys that stackful cannot match, and be specific about the
   resource.

### Q3 — Function colouring

"What Color Is Your Function" complains that `async` infects every caller. Meanwhile
Go, Erlang, and Java 21's virtual threads have no `async` keyword at all.

1. What did Go and Java pay to get uncoloured suspension? Name the concrete runtime
   capability required.
2. Rust chose colouring deliberately. What was the constraint that made stackful
   suspension unacceptable for Rust specifically?
3. Is `throws` in Java a colour in the same sense? Argue both ways, then commit.

### Q4 — The C-stack boundary

Lua raises `attempt to yield across a C-call boundary`. Many VMs with coroutines have some
form of this error. (Python generators also cannot suspend from inside a `list.sort`
comparator, but for an unrelated reason — `yield` is lexically confined to the generator's
own frame, so the callback's implementation language is irrelevant. That is the stackless
limitation of Q6, not a boundary check.)

1. Why does a native frame between the coroutine's entry and the `yield` site make
   suspension unsound? Be specific about what breaks — it is not "we can't save C
   locals" alone.
2. A VM that trampolines *all* calls, including calls from native code back into the
   interpreter, does not have this restriction. What does it pay for that?
3. You ship the restricted version first and want the lift to be purely additive later.
   What must you get right in v1 so that v2 breaks no program?

### Q5 — Symmetric and asymmetric

Two coroutine transfer disciplines:

- **Asymmetric** — `resume(c)` and `yield(v)`. `yield` always returns to whoever resumed you.
- **Symmetric** — `transfer(c, v)`. You name your successor; there is no implicit caller.

1. Which one is strictly more expressive, and what does the other one gain by being weaker?
2. In an asymmetric design, is the "resumer" link a static parent or a dynamic caller
   chain? Construct a program where the answer is observable.
3. Show how to build a symmetric transfer on top of an asymmetric primitive, and name
   the cost you just paid per switch.

### Q6 — Generators are not coroutines

Python's `yield` (pre-`yield from`) suspends only the generator's own frame. Lua's
`coroutine.yield` suspends the whole coroutine stack, however deep.

1. What is the concrete limitation of the Python-style design, and what is the workaround
   called?
2. Why is `yield from` / `await` delegation *not* merely sugar — what does a naive
   sugar-only desugaring cost asymptotically?
3. If generators are the weak form, why did Python, JS, and C# all ship the weak form
   first?

### Q7 — Async/await as a state machine

A stackless `async fn` compiles to a state machine.

1. Sketch what the compiler does to local variables that are live across an `await`,
   versus those that are not.
2. Why does this make the size of a future a *compile-time* property, and why do people
   complain about "huge futures"?
3. Rust's async has a self-reference problem that C#'s does not. Name it, and name the
   type-system machinery introduced to contain it.

### Q8 — Cooperative scheduling has no scheduler

A "ready queue" of runnable coroutines, drained in FIFO order, is often called a
scheduler.

1. Argue that a ready queue alone is *not* a scheduler. What is the missing capability
   that would make it one?
2. In a purely cooperative single-threaded runtime, what class of bug replaces the data
   race — and why is it in some ways worse to debug?
3. A cooperative runtime wants to add a "this task ran too long" watchdog. Why is that
   substantially harder than in a preemptive runtime, and what is the standard trick?

### Q9 — Parked stacks and the collector

A suspended coroutine holds a value stack and a call-frame stack full of live references.

1. Why is it a hard requirement that those stacks live in GC-visible memory rather than
   in native/C memory?
2. A coroutine object is suspended and *unreachable* from any root. What must happen, and
   what makes it genuinely hard? Name two languages' answers.
3. A stackful coroutine's stack grows. Segmented stacks and copying/relocating stacks are
   the two answers. Give the failure mode each one introduced in practice.

### Q10 — Structured concurrency

`go f()` returns immediately and the goroutine outlives the caller's scope. Structured
concurrency (Trio nurseries, Kotlin `coroutineScope`, Java's `StructuredTaskScope`) makes
a scope wait for all children.

1. What class of bug does structured concurrency eliminate *by construction*, and what is
   the analogy to a much older language feature?
2. What does it cost — name a legitimate pattern that becomes awkward or impossible.
3. Cancellation: why does structured concurrency make cancellation *tractable* where
   unstructured spawning makes it nearly impossible to get right?

### Q11 — Cancellation is not an error

Killing a running task mid-flight.

1. Why is "just raise an exception in the target task" insufficient as a cancellation
   mechanism? Give a concrete way it goes wrong.
2. Go chose cooperative cancellation via an explicit `context.Context`, not a `kill()`.
   Erlang chose the opposite — arbitrary `exit/2` on any process. Why can Erlang do this
   safely when Go cannot?
3. Where must cancellation checks be placed, and what is the name of that concept in a
   GC's vocabulary?

### Q12 — Actors and shared nothing

Erlang/BEAM processes: no shared mutable state, copy-on-send messages, per-process heaps.

1. Per-process heaps make sends O(size of message). Why is that a *good* trade for BEAM's
   actual goal, and what is that goal (it is not throughput)?
2. What does a shared-nothing design buy the *garbage collector* specifically?
3. Erlang's "let it crash" depends on a language property most languages lack. What is it?

### Q13 — Reentrancy of the interpreter loop

A native primitive (`map`, `sort`, `each`) calls back into user code. Two implementations:
recursively re-enter the interpreter loop on the native stack, or push a frame and return
to the single flat loop.

1. Name three distinct language features that become harder or impossible under the
   recursive-reentry design.
2. Why do so many real VMs ship the recursive design anyway? Give the honest engineering
   reason.
3. Under the recursive design, a nested loop invocation typically runs "until the frame
   stack drains back to a recorded depth." Explain precisely why a coroutine switch
   underneath such a call corrupts it — this is the mechanism behind Q4.

### Q14 — Continuations, one-shot and multi-shot

Scheme's `call/cc` captures a continuation you can invoke many times. Most coroutine
systems capture one you can invoke once.

1. What does multi-shot require of the captured stack that one-shot does not?
2. Coroutines, generators, exceptions, and early `return` are all expressible via
   `call/cc`. Why did essentially no production language adopt it as *the* primitive?
3. Delimited continuations (`shift`/`reset`, or effect handlers in OCaml 5 / Koka) are the
   modern answer. What is the delimiter buying you, in implementation terms?

### Q15 — Detecting the end

A generator's `next()` returns a value. So does its final `return`.

1. Why can the *value* alone never distinguish "here is another element" from "I'm done",
   in a language with no distinguished bottom value?
2. Enumerate the standard answers used by real languages, and say which one costs an
   allocation per step.
3. A cursor protocol (`hasNext`/`advance`, or a checked end-sentinel) can be zero-alloc
   per step. What does it give up compared to a generator?

---

## Answers

### A1 — Where the value comes from

**1.** The resume records the stack index of the `yield` send's receiver slot — call it
`resume_slot` — then delivers by `stack.truncate(slot); stack.push(value)`. That is
*byte-for-byte the ordinary method-return sequence*: in a stack VM, a call pushes
receiver + args, and returning collapses that window and leaves the result where the
receiver was. Suspension is a return whose value arrives late and from somewhere else.

**2.** Because the resumed code's next instruction is the one immediately after the
`Invoke`, and the stack beneath it is in exactly the shape a normal return leaves. There
is no observable difference to expose. From inside the coroutine, `yield(11)` is a call
that took a long time and returned `100`. This is precisely what makes coroutines
composable: the suspending call site is syntactically and operationally an ordinary call.

**3.** On the first resume there is **no open send window** — the coroutine has never
executed a `yield`, so no `resume_slot` was ever recorded and no expression is waiting for
a value. There is nothing to fill. So the argument cannot be "delivered"; it has to be
*bound*, which means pushing a fresh frame with the argument in the parameter window. Two
different code paths, and the state bit selecting between them (`started`) is real state,
not bookkeeping.

**Trap.** Saying the resume "passes the value into the coroutine like an argument." It
does not; on every resume after the first, it fills a hole in a half-evaluated expression.
Getting this wrong means you cannot explain why the first resume arity-checks and later
resumes silently ignore extra arguments.

### A2 — Stackful versus stackless

**1.** Stackful: the suspended state is a real, separately allocated call stack, and
suspension is a stack switch. Stackless: the suspended state is a heap-allocated object
(a state machine) holding just the live locals of *one* function, and suspension is a
return.

**2.** Because stackless suspension is *implemented as returning*. To suspend, the frame
must vanish from the machine stack and its live state must have been moved into a heap
object — and only the compiler for that function can perform that transformation, because
only it knows which locals are live across the suspend point and how to re-enter at the
right resume label. A helper compiled as an ordinary function has no such machinery, and
its caller has no way to add it. Hence the marker on every frame in the suspension path:
it is not a courtesy annotation, it is a *codegen request*.

**3.** Memory proportional to live state, not to stack depth, and no stack-growth policy
at all. A stackful coroutine must reserve or grow a stack; a stackless future can be 48
bytes. That is why stackless dominates in embedded, in Rust's no-alloc contexts, and
anywhere you want millions of in-flight I/O operations with a known memory ceiling. Also:
a stackless future is a value — movable, storable, composable in a struct — which a stack
is not.

**Trap.** Framing this as "stackless is just an optimization of stackful." They are not the
same feature at different costs — they have *different expressive power*, and the
difference is exactly (2). Anyone who says stackless is the cheap version has not hit the
wall where a suspension point needs to live inside a callback they do not control.

### A3 — Function colouring

**1.** They pay for a **runtime that owns the stack**: growable/relocatable per-task
stacks, a scheduler, and — critically — the ability to find and rewrite every pointer into
a stack when it moves. Go's stacks are copied on growth, which requires a precise GC and a
compiler that emits stack maps. Java's virtual threads required the JVM to be able to
mount/unmount a continuation onto a carrier thread, which is a JVM-level capability, not a
library. No colour, but the runtime is now heavy and non-negotiable.

**2.** Rust targets environments with no runtime at all — kernels, firmware, wasm, FFI
across the C ABI. A stackful design demands an allocator and a stack-management policy at
the language level, which would have made `async` unavailable exactly where Rust wanted to
compete. Colouring is the price of `async` being a *library-schedulable, allocation-optional
value*. Related: Rust cannot relocate stacks the way Go does, because raw pointers into
stack memory are legal and untracked.

**3.** For: `throws` propagates up the call chain, forces annotation of intermediate
functions, and splits the library ecosystem into two dialects — structurally identical to
`async`. Against: an exception is a *result* of a call, not a *reshaping* of it; a caller
can discharge `throws` locally with a `try/catch`, whereas `await` cannot be discharged
without either blocking or becoming async yourself. **Commit:** it is a colour, but a
discharge-able one, and dischargeability is the whole difference. The `async` complaint is
really "this colour has no local exit."

**Trap.** Treating colouring as a language-design mistake someone could simply have avoided.
It is a *consequence of the runtime you are willing to require*. If the answer does not
mention what Go's and Java's runtimes must be able to do — find and rewrite every pointer
into a stack — it is an aesthetic complaint, not an engineering one.

### A4 — The C-stack boundary

**1.** Two things break, and only naming both is a complete answer. (a) The native frame's
state lives on the machine stack, which the coroutine does not own and cannot park — so
resuming would have to reconstruct a C frame, which is not possible portably. (b) More
subtly and more fatally in practice: the native frame is typically *mid-way through
driving the interpreter loop re-entrantly*, holding invariants about the current
coroutine's frame stack — a recorded base depth to drain back to, a borrowed pointer, an
iteration cursor. A switch swaps that frame stack out from under it. When control
eventually returns to the native frame, its invariants refer to a stack that is no longer
mounted. (b) is what makes the restriction necessary even in a VM where you *could* save
the C frame.

**2.** It pays: every native combinator has to be rewritten in a resumable style (an
explicit state object, not a Rust/C loop with a local index), the interpreter loop grows
cases for "native frame in progress", and the fast path for the common non-suspending call
gets slower because it now goes through the trampoline. You trade a whole-VM tax for a
capability used by a minority of code.

**3.** Three things. (a) The restriction must be a **catchable, named surface error**, not
a panic or UB — so that programs can probe and adapt. (b) The switch must be signalled to
the interpreter with a **typed signal**, not inferred from a heuristic like "the frame
count changed" (a non-local return trips that heuristic too, and the two need opposite
handling). (c) You must not let any *observable semantics* depend on the restriction —
e.g. don't let people rely on "yield inside `each` throws" as control flow. Then lifting
the restriction only turns errors into successes, which no correct program depends on.

### A5 — Symmetric and asymmetric

**1.** Symmetric is more expressive: it is a raw `goto` between stacks and can express any
transfer graph, including a cycle with no distinguished driver. Asymmetric gains
**structure** — the resumer link forms a stack discipline, so return, error propagation,
and "who gets the value when this thing finishes" all have unambiguous answers. Same trade
as `goto` vs. structured control flow, one level up.

**2.** Dynamic caller chain. Observable: create coroutine `f`; resume it from the main
routine so it yields once; then create `g`, resume `g`, and have `g` resume `f`. `f`'s
second yield goes to `g`, not to main — the link was rewritten at resume time. If it were
a static parent, it would go to main. This also means the resumer link is *mutable state
on the coroutine object*, written by the resumer on every resume, which is exactly why
resuming an already-running coroutine must be an error rather than a re-entry.

**3.** `transfer(target, v)` becomes: yield a request record naming `target` back to a
central trampoline (the "driver"), which then resumes `target`. Cost: **two switches per
logical transfer** instead of one, plus the driver's dispatch. Every symmetric transfer
round-trips through the top of the resumer chain.

### A6 — Generators are not coroutines

**1.** A Python-style generator can only suspend from its *own* frame, so you cannot
factor a generator's body into helper functions that yield. The workaround is **delegation**
— `yield from` in Python, `yield*` in JS — which makes the outer generator explicitly
re-yield the inner one's values.

**2.** Naive sugar (a loop in the outer generator that pulls from the inner and re-yields)
makes each value traverse every level of the delegation chain, so a chain of depth *n*
costs O(n) per element, O(n²) for a linear recursive generator over *n* items — the classic
quadratic recursive-tree-traversal blowup. But do **not** claim `yield from` fixes the
asymptotics: PEP 380 lists chain short-circuiting only as an optional optimisation, and
CPython declined it — a depth-*n* delegation still costs O(n) per element, measurably about
1.3× faster than a hand-written re-yield loop and no better. What `yield from` genuinely adds
over sugar is *semantics*: it forwards `send`, `throw`, and `close` to the innermost
generator and propagates the subgenerator's return value, all of which a re-yield loop gets
silently wrong.

**3.** Because the weak form requires no stack switching. A generator is a frame turned
into a heap object — the compiler already knows that frame's layout — so it needs no
runtime stack management, no GC changes, and no interaction with native frames. It is the
form you can ship inside an existing VM without touching the VM's core. Every one of those
languages later paid for it (`yield from`, `await`, virtual threads, `Task`), which is the
recurring lesson: the cheap version of suspension is the one that composes worst.

### A7 — Async/await as a state machine

**1.** Locals live across an `await` are lifted into fields of the generated state object;
locals dead at every suspend point stay as ordinary stack slots in the resume function. The
body is split at suspend points into labelled resume states, and the whole function becomes
a `poll`/`MoveNext` switch on the current state. Control flow across a suspend point
(loops, `try` blocks) has to be reified into that state machine, which is why `await`
inside `finally` or inside a `lock` is either restricted or expensive.

**2.** Because the set of live-across-await locals is known statically, the state object's
layout — and hence its size — is fixed at compile time. It is the max over all states, not
the current state, so a future is as big as its fattest suspension point. Nesting composes
by *embedding*: an outer async fn awaiting an inner one contains the inner future inline,
so sizes add down the call tree, and one large buffer deep in a chain inflates everything
above it. Hence `Box::pin` as the manual size-cut, and "async fn size" being a real
production concern.

**3.** **Self-referential futures**: a local holding a reference to another local — e.g. a
slice into a buffer — is legal across an await, so the state object contains a pointer into
itself. Move the object and the pointer dangles. C# doesn't care because references are GC
handles that survive relocation. Rust's answer is `Pin`: once a value has been *pinned* —
by `Box::pin`, `pin!`, or an unsafe `Pin::new_unchecked`, not by the first poll — it may
never be moved again, which is why `Future::poll` takes `self: Pin<&mut Self>` rather than
`&mut self`. `Unpin` is the auto-trait that opts *out*: a type with no interior
self-references implements it automatically and can still be moved out of a `Pin`, which is
what stops the scheme infecting ordinary types. A generated `async` state machine is `!Unpin`
exactly when it holds a reference into itself.

**Trap.** Saying `Pin` exists "for safety" without naming *what* moves and *why anything
points into it*. `Pin` is not a general safety wrapper; it exists because the state machine
transformation legitimately produces a self-referential struct, and the language had no
other way to say "this address is now load-bearing."

### A8 — Cooperative scheduling has no scheduler

**1.** A queue is a data structure; a scheduler makes *policy*. Missing: the ability to
choose which runnable task goes next by some criterion — priority, fairness, deadline,
starvation avoidance — and to preempt or requeue. FIFO drain has no policy; it has an
order. Calling it a scheduler misleads people into thinking priority or fairness questions
have already been answered, when nothing in the system can even express them.

**2.** **Starvation / failure to yield.** One task that loops without a suspension point
freezes the entire runtime, and there is no timer to break in. It is worse to debug than a
data race in one specific way: a data race has a *culprit region* you can find by
inspecting shared state, whereas a starvation bug's culprit is often code that is
individually correct and merely long-running — the bug is in the *composition* and is
invisible in any single stack trace. The observable symptom (everything hangs) is also
maximally uninformative.

**3.** Hard because interrupting a task requires suspending it at a point where its state
is coherent, and in a cooperative runtime the only such points are the suspension points
the task chose. The standard trick is compiler- or interpreter-inserted **yield points** —
a check on loop back-edges and function entries, exactly analogous to GC safepoints. Go is
the instructive counter-example, and getting it right is worth points: Go only ever had
checks at **function prologues**, so a loop containing no function calls — not merely a
non-allocating one — was uninterruptible before 1.14. When Go measured explicit
loop-back-edge preemption, even the cheapest scheme cost roughly 8% geomean, so it abandoned
that route for **signal-based asynchronous preemption**, paying instead for register and
stack maps at nearly every instruction.

### A9 — Parked stacks and the collector

**1.** Because the values on those stacks are the only references keeping objects alive.
If a parked stack lives in native memory the collector cannot see it, so either you
conservatively scan native memory (imprecise, blocks moving collection, risks retaining
garbage) or you free live objects. Keeping parked stacks *inside* the collector's own
object graph is what makes a coroutine an ordinary traceable object and keeps a precise,
potentially moving collector on the table.

**2.** A suspended coroutine that nothing references is garbage, but it holds a stack with
live frames — including any `finally`/`ensure`/`defer` blocks and open resources. Do you
run them? Running arbitrary user code during collection is the finalizer problem in its
worst form: it can resurrect objects, block, or itself suspend. **Python** runs
`GeneratorExit` by throwing into the generator at collection time, and has an explicit
documented mess around generators whose `finally` blocks yield. **Go** never collects a
blocked goroutine's deferred work — a goroutine blocked forever on a channel is simply
leaked, by design, and the runtime only reports it via deadlock detection when *all*
goroutines are blocked. Both answers are unsatisfying, which is the point.

**3.** **Segmented stacks**: the "hot split" problem — a function call that repeatedly
crosses a segment boundary inside a loop pays segment allocation on every iteration, with a
cliff-shaped performance profile. Go shipped and then removed segmented stacks for exactly
this. **Copying stacks**: every pointer *into* the stack must be found and rewritten,
which requires precise stack maps and forbids unmanaged interior pointers — this is why Go
can do it and why C-interop from Go has to switch to a system stack.

**Trap.** Answering (2) with "the GC collects it like any other object." That is the
question, not the answer — the difficulty is that collecting it may require *running user
code* (cleanup handlers) at collection time, which is the finalizer problem with a call
stack attached. An answer that does not reach the cleanup-handler conflict has not
understood why anyone asks.

### A10 — Structured concurrency

**1.** It eliminates **orphaned/leaked tasks and lost errors** by construction: a task
cannot outlive the scope that created it, so an error in a child has a defined place to
propagate to (its parent's scope) and cannot vanish into a background task nobody awaits.
The analogy is the arrival of structured control flow — `go`/`spawn` is `goto` for
concurrency; a nursery is the `while` loop. Same argument, same shape of benefit
(composable local reasoning: what happens inside a scope, ends inside a scope).

**2.** Genuinely long-lived background work whose lifetime is *not* lexical — a connection
pool janitor, a metrics flusher, a listener that should outlive the request that started
it. You have to hoist those to an outer scope, which means threading a scope handle
through your API. That is a real ergonomic cost and the honest reason unstructured spawn
still exists everywhere.

**3.** Because cancellation needs a well-defined *set* to cancel and a well-defined place
to report the outcome. A scope is exactly that set: cancelling it means cancelling a known
finite tree of children and waiting for them to finish unwinding, with one join point that
collects errors. With unstructured spawning, there is no name for "everything this
computation started", so cancellation degenerates into per-task bookkeeping that is
correct only if nobody forgot.

### A11 — Cancellation is not an error

**1.** Because an exception can be caught — including accidentally, by a
`catch (Exception e)` or a broad `except:` in library code — which silently converts
cancellation into "kept running". It can also fire at a point where the task's invariants
are broken, or *inside a cleanup handler*, aborting the cleanup. Systems that use
exceptions for cancellation therefore need a distinguished, hard-to-catch exception type
(Python's `CancelledError` derives from `BaseException`, not `Exception`, precisely for
this) plus rules about re-raising.

**2.** Erlang processes share nothing **by default** — no shared heap, no shared mutable
terms — so killing one cannot tear the data structures other processes are reading. The
escape hatches prove the rule rather than refuting it: ETS tables *are* genuinely shared
mutable state, individual operations are atomic but multi-operation sequences are not, and a
kill between two related writes leaves exactly the inconsistency the model otherwise rules
out. Name ETS before the interviewer does. Go's goroutines share memory
freely: killing one mid-mutation could leave a map or a lock in a broken state visible to
everyone else, so Go can only offer *cooperative* cancellation where the target chooses
its own consistent stopping points. The lesson generalizes: **preemptive kill is only safe
in a shared-nothing world.**

**3.** At points where state is coherent and the runtime knows the full live set —
suspension points, loop back-edges, allocation sites, calls. The GC vocabulary word is
**safepoint**, and it is the same concept with the same metadata requirements: you can
only stop where you can describe the machine's state.

**Trap.** Calling cooperative cancellation a weakness of Go's design. It is forced by the
memory model, and (2) is the proof: the language that *can* kill arbitrarily is the one whose
processes share nothing by default. Anyone proposing `killGoroutine()` is proposing
shared-memory corruption with extra steps — and note the symmetry, since Erlang's own shared
mutable escape hatch, ETS, is precisely where a kill *can* leave torn state.

### A12 — Actors and shared nothing

**1.** The goal is **soft real-time responsiveness and fault isolation**, not throughput.
Per-process heaps mean each process is collected independently, so a collection pauses one
process for microseconds instead of pausing the world — which is what lets a telecom switch
hold latency bounds with millions of processes. Paying O(n) per message to preserve that is
a good trade because messages are mostly small and the alternative (shared heap) buys
throughput at the cost of the one property the system exists for.

**2.** Per-process heaps are **independently collectible and never require a write barrier
for cross-process references** in the common case, because a message is copied and no pointer
crosses a heap boundary. That kills the two hardest parts of concurrent GC at once:
inter-region reference tracking and stop-the-world coordination. Know the documented
exception before it is produced against you: binaries over 64 bytes live in a shared off-heap
area, and a message passes a refcounted handle rather than a copy — which is exactly why that
one case needs explicit refcount bookkeeping and is the classic source of BEAM binary leaks.
It also means a dead
process's heap is freed wholesale with no tracing at all, which is why process death is
cheap enough to be a control-flow mechanism.

**3.** **No shared mutable state**, hence no possibility that a crashed process left global
invariants broken. "Let it crash" is only a sane default when a crash's blast radius is
provably bounded to one heap. Bolting supervision trees onto a shared-memory language gives
you supervised restart of processes that may have corrupted the very state they are
restarting into.

### A13 — Reentrancy of the interpreter loop

**1.** (a) **Coroutine suspension** across the native frame — Q4. (b) **Non-local return /
unwinding** through it: a `return` from inside a block passed to `map` has to unwind past a
native frame, which means the native code must be prepared for its callee to not return
normally, in every primitive. (c) **Deep recursion in user code becomes native stack
overflow** — the user's recursion depth is now bounded by the C stack, and you get a
segfault rather than a catchable language-level error. Also credible: debugger stepping and
stack introspection, because half the language stack is native frames.

**2.** Because it is *dramatically* simpler and faster to write. A native `map` is a Rust
loop that calls "run this block, give me the value" and gets a value back — you keep your
locals in registers, you keep the code readable, and the fast path costs one call. The
resumable version requires reifying every combinator's loop state into an explicit object
and threading it through the interpreter. Nearly every VM starts here, including CPython
and Lua, and pays later.

**3.** The nested invocation runs "until `frames.len()` drops back to *base*", where *base*
was recorded as the frame count **of the currently mounted coroutine** at entry. A switch
replaces the entire frame vector with a different coroutine's. Now the drain condition is
being evaluated against a stack it was never computed for: if the new coroutine happens to
have fewer frames, the nested loop believes its callee already returned and drains — popping
frames belonging to a different coroutine and returning a bogus value; if more, it runs the
wrong code until something else breaks. The condition is not merely stale, it is a
comparison between two unrelated coordinate systems. That is why the guard is "no switch
while native re-entry depth ≠ 0" rather than a subtler per-frame check.

**Trap.** Describing the drain condition as "stale" or "off by a few frames." Stale implies
a value that was right and aged out, repairable by recomputation. This is a number from one
coordinate system being compared against another — there is no correct value to recompute,
because the quantity it names does not exist on the stack now mounted. Getting this
distinction wrong is what produces the tempting-and-wrong fix of "just recompute the base
after the switch."

### A14 — Continuations, one-shot and multi-shot

**1.** Multi-shot requires the captured stack to be **immutable or copied**, because a
second invocation must see the original state, not the state left by the first. That means
either copying the whole stack at capture (expensive, and wrong if there are pointers into
it) or heap-allocating every frame so that reuse is safe. One-shot can just *move* the
stack, which is a pointer swap.

**2.** Cost and interaction. `call/cc` makes every frame potentially re-entrant, which
forecloses stack allocation of frames, complicates FFI to any C ABI, and interacts badly
with resource cleanup — `dynamic-wind` exists because "leaving a scope" is no longer a
single event. It is also *undelimited*: capturing the entire rest of the program means the
capture cost is unbounded and it composes poorly with a host runtime that owns the outer
stack. You get one enormously powerful primitive with a global tax on the implementation.

**3.** The delimiter bounds the capture: `shift` captures only up to the nearest enclosing
`reset`, so the cost is proportional to a known slice of the stack, not the whole program.
That makes capture cheap, makes composition local, and — decisively — makes the captured
thing a *function-shaped value* you can type. Effect handlers are this idea with the
handler selected by the effect's *constructor* and the resumption reified as a first-class
continuation value — which is why OCaml 5 can implement stackful concurrency as a *library*
on top of one runtime primitive. Be precise about OCaml here: its effects are deliberately
**unchecked**, nothing appears in function types, and an unhandled effect is a runtime
failure. Koka is the system that tracks effects in the type system.

### A15 — Detecting the end

**1.** Because the generator's value domain and the protocol's signal domain overlap: any
sentinel you pick (`nil`, `-1`, `None`) is a value a generator might legitimately produce,
so a consumer cannot distinguish "yielded `None`" from "finished". A distinguished bottom
value only helps if it is *unforgeable* by user code — i.e. genuinely outside the value
domain — which most dynamic languages don't have.

**2.** (a) **Exception on exhaustion** — Python's `StopIteration`; correct but expensive if
exceptions are expensive, and it leaks (a `StopIteration` escaping a generator body used to
silently truncate the caller; PEP 479 fixed it by converting to `RuntimeError`). (b) **A
result record** — JS's `{value, done}`, which **allocates an object per step** unless the
engine escape-analyses it away. (c) **A separate status from the value** — Java's
`hasNext()`/`next()`, state-dependent and forcing the iterator to compute one element ahead;
C#'s `MoveNext()`/`Current` is the sharper form of this family, folding advance and status
into one `bool`-returning call with the value read separately, and allocating nothing. (d) **A private, unforgeable end sentinel** — zero alloc, but
requires the sentinel to be inaccessible from user code.

**3.** A cursor gives up **suspension of arbitrary computation**. A generator can be paused
in the middle of a deep recursive traversal with its whole call stack intact; a cursor must
carry an explicit, materialized representation of "where I am" — which for a tree means an
explicit worklist you maintain by hand. Generators buy you the traversal's control stack for
free; cursors make you write it. The zero-allocation, no-VM-support price is exactly that
manual reification.

**Trap.** Saying `hasNext`/`next` is "the same thing without the sugar". It is not: for any
traversal whose state is not a simple index, the cursor version requires you to invert the
control flow by hand, and that inversion is the entire difficulty of writing iterators in
languages without coroutines.
