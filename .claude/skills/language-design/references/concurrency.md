# Concurrency & Asynchrony

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** designing/critiquing fibers, coroutines, async, scheduling, or parallelism features.

## Contents
- Execution unit
- Coroutine symmetry & generators
- Async/await coloring vs transparent green threads
- Scheduling discipline
- Shared state vs isolation
- Structured concurrency & cancellation
- Memory model & data-race guarantees

## Execution unit
| Option | Langs | Consequence |
|---|---|---|
| OS threads 1:1 | Java(pre-Loom), Rust `std::thread`, C++ | Kernel-scheduled, preemptive; ~MB stacks, costly context switch |
| Green/M:N threads | Erlang/BEAM, Go, Loom, Smalltalk, Haskell | Runtime-scheduled over N cores; cheap, millions live |
| Stackful fibers/coroutines | Lua, Ruby Fiber, Wren fibers | Own heap stack; suspend anywhere; GC must scan each stack |
| Stackless coro / state machine | Rust `async`, Kotlin, JS, Python | Suspend only at marked points; compiled to a resumable struct |

**Syntax.** Go `go f()` · Java `Thread.ofVirtual().start(r)` · Erlang `spawn(F)` · Lua `coroutine.create(f)` · Rust `async fn`+`.await`
**Impl.** 1:1 = kernel TCB; green = runtime scheduler over worker pool; stackful = per-fiber stack switch (recipes.md#coroutine-switch); stackless = compiler lowers to a resumable enum.

**Hazard — stackful-fiber ⊗ moving/native-stack GC (CROWN JEWEL).** Each fiber carries a full native stack the collector must scan for roots; a moving/compacting GC must rewrite pointers inside every suspended stack. Stackless coros dodge it (state lives in a heap struct the GC already knows). → overlay

## Coroutine symmetry & generators
| Option | Langs | Consequence |
|---|---|---|
| Asymmetric yield/resume | Lua, Python gen, Ruby Fiber | Coro yields only to its resumer; strict caller/callee tree |
| Symmetric transfer | Lua `transfer`, Modula-2 | Coro hands control to any peer; no implied return path |
| Generators (yield values) | Python, JS `function*`, C# | Restartable producer; sugar over asymmetric coro |
| Channels-as-primitive | Go, CSP | Concurrency via communication, not shared coro handles |

**Syntax.** Lua `coroutine.yield(v)`/`resume(co)` · Python `yield x` / `def gen()` · JS `function*`+`yield` · Ruby `Fiber.new{ Fiber.yield }` · Go `ch <- v` / `<-ch`
**Impl.** Asymmetric = single saved resumer link; symmetric = explicit target on transfer; generators = state-machine or thin stackful coro (recipes.md#coroutine-switch).

**Hazard — non-local-return/exception ⊗ fiber boundary (CROWN JEWEL).** An exception or `^`-style non-local return unwinding out of a coroutine has no home frame across the yield boundary once the resumer moved on. Decide: propagate to resumer, kill the fiber, or error. See recipes.md#non-local-return and recipes.md#coroutine-switch. → overlay

## Async/await coloring vs transparent green threads
| Option | Langs | Consequence |
|---|---|---|
| Explicit async/await | Rust, JS, Python, C#, Kotlin | Sync/async split infects every signature up the call chain |
| Transparent green threads | Go, BEAM, Loom, Haskell | Blocking calls just park the fiber; no colored functions |
| Colorless via effects | Koka, OCaml 5 handlers | Async is an effect row, inferred not spelled everywhere |
| Callback/CPS | Node(old), C | Manual continuation passing; inversion of control, no coloring but unreadable |

**Syntax.** JS `await f()` / `async function` · Kotlin `suspend fun` / `launch{}` · Rust `f().await` / `async fn` · Go `go f()` (no marker) · Loom plain blocking call
**Impl.** await = suspend point in a compiler-built state machine polled by an executor; green threads park the fiber at the blocking syscall, no signature change.

**Hazard — function coloring ⊗ higher-order/callback APIs (CROWN JEWEL).** A `map`/`filter`/`Iterator` taking a callback can't accept an `async` fn without a parallel async-colored variant; the split metastasizes through every combinator (`AsyncIterator`, `try_stream`). Transparent threads avoid it entirely. → overlay

## Scheduling discipline
| Option | Langs | Consequence |
|---|---|---|
| Cooperative (yield points) | Lua, JS event loop, Python asyncio | Zero preemption cost; one runaway task freezes all |
| Preemptive (timer/reduction) | BEAM (reductions), Go(async preempt), OS | Fairness guaranteed; needs safepoints + interruptible loops |
| Run-to-completion | JS microtask, actors | Handler runs atomically to end; no mid-task interleave, simple invariants |
| Work-stealing M:N | Go, Rust Tokio, Java FJ | Load-balanced across cores; task must not pin a worker thread |

**Syntax.** asyncio `await asyncio.sleep(0)` · JS `await Promise.resolve()` (yield) · Go implicit at calls/`runtime.Gosched()` · BEAM implicit (no yield keyword)
**Impl.** Cooperative = only await/yield points re-enter the loop; preemptive = reduction counter (BEAM) or async-safepoint signal (Go) forces a switch mid-task.

**Hazard — cooperative scheduling ⊗ one blocking call (CROWN JEWEL).** A single synchronous syscall, CPU-bound loop, or FFI call with no yield point starves every other task on that scheduler thread. BEAM's reduction-counting preemption and Go's async-preemption exist precisely to bound this. → overlay

## Shared state vs isolation
| Option | Langs | Consequence |
|---|---|---|
| Shared memory + locks/atomics | Java, C++, Rust, Go | Max sharing; deadlock, races, lock-ordering burden |
| Actors / message passing | Erlang, Akka, Smalltalk | No shared mutable state; copy on send; mailbox backpressure |
| Isolates / Ractors | Ruby Ractor, Dart, JS Worker | Heaps disjoint; only immutable/shareable objects cross |
| STM | Haskell, Clojure | Composable optimistic transactions; retry storms, no I/O inside |
| GIL (serialize the interpreter) | CPython, MRI Ruby | Trivially memory-safe; no true CPU parallelism |

**Syntax.** Erlang `Pid ! Msg` / `receive` · Go `ch <- v` · Ruby `Ractor.new` / `r.send` · Clojure `dosync (alter r ...)` · Rust `Arc<Mutex<T>>` + `.lock()`
**Impl.** Actors/isolates = disjoint heaps + copy-on-send mailbox; STM = optimistic log + validate-and-retry commit; locks = OS mutex/atomics over one shared heap.

**Hazard — GIL ⊗ "add real threads later".** Ecosystem C-extensions assume the GIL's implicit mutual exclusion; removing it (nogil, Ractor) breaks every extension relying on non-atomic global state. Isolation retrofits are near-impossible once mutable sharing leaked into the API. → overlay

## Structured concurrency & cancellation
| Option | Langs | Consequence |
|---|---|---|
| Nursery/scope owns children | Trio, Kotlin `coroutineScope`, Java StructuredTaskScope | Tasks can't outlive scope; errors/cancel propagate to siblings |
| Cancellation token/context | Go `context`, .NET | Cooperative cancel threaded explicitly; forget-to-check leaks |
| Fire-and-forget | raw `go`, `spawn`, Promise | Orphan tasks; leaks, lost exceptions, no join point |
| Linked/supervised processes | Erlang links + supervisors | Crash propagates or restarts per supervision policy |

**Syntax.** Kotlin `coroutineScope { launch{} }` · Trio `async with open_nursery() as n:` · Java `try(var s=StructuredTaskScope...)` · Go `ctx,cancel:=context.WithCancel()`
**Impl.** Scope = a join barrier owning child handles; exit awaits/cancels all children. Token = a shared cancelled-flag polled at yield points; supervisor = linked-exit signal triggers restart policy.

**Hazard — cancellation ⊗ cooperative-only.** Cancellation that only fires at yield points can't interrupt a tight CPU loop or blocking FFI; the token is checked never. Structured scopes bound *lifetime* but not *latency* of cancel. → overlay

## Memory model & data-race guarantees
| Option | Langs | Consequence |
|---|---|---|
| Compile-time `Send`/`Sync` markers | Rust | Data races are type errors; closures/channels enforce ownership |
| Happens-before + `volatile`/atomics | Java, C++, Go | Races are UB/tearing; correctness is programmer discipline |
| No sharing → no model needed | Erlang, isolates | Per-process heaps make the memory model trivial |
| Sequential (single-threaded core) | JS, asyncio, GIL langs | One mutator; interleaving only at await points, no torn reads |

**Impl.** `Send`/`Sync` = compiler-checked auto-traits gating cross-thread moves/shares; JMM/C++ = happens-before edges from atomics+fences the codegen must honor; share-nothing = no model, isolation makes races unrepresentable.

**Hazard — Send/Sync ⊗ green-thread migration.** A fiber moved between worker threads (work-stealing) makes thread-local/`!Send` state observably wrong; the marker system must forbid capturing non-`Send` data across an `await`/spawn or you reintroduce races the type system claimed to remove. → overlay
