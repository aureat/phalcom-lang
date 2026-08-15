# Fibers, Concurrency, and Scheduling Semantics

Cooperative fibers simplify interleaving, but they do not eliminate concurrency semantics. Programs can observe when execution may switch, what blocks scheduler, how cancellation works, and how failures propagate.

## 1. Execution unit

```text
Fiber = {
    id,
    machine/continuation,
    status,
    fiberLocals,
    result/outcome
}
```

Statuses might include:

```text
New | Runnable | Running | Suspended(reason) | Completed(outcome)
```

## 2. Scheduler state

```text
Scheduler = current + runnableQueue + waitSets + completed
```

Specify:

- when running fiber can stop;
- how next runnable chosen;
- where yielding fiber re-enters queue;
- fairness/starvation guarantees;
- behavior with no runnable fibers;
- how timers/IO wake fibers.

## 3. Atomicity between yield points

In current single-thread cooperative implementation, ordinary bytecode may execute without interleaving until scheduling boundary. This can justify temporary shared-state facts within a non-yielding region.

But native callbacks, implicit safepoints, or future parallelism can alter boundary. Language should specify scheduling points rather than relying on current VM accident.

## 4. Yield

A yield:

1. preserves current continuation/frame state;
2. marks fiber runnable/suspended per API;
3. chooses another runnable fiber;
4. may later resume original fiber with defined resume value.

Suspension is not completion. Suspended home frames remain live.

## 5. Await/Future

If await suspends until readiness:

```text
ready   -> produce result now
pending -> register waiter + suspend current fiber
complete -> enqueue waiter(s)
```

Define how failure is represented: rethrow, `Result`, or another protocol.

## 6. Blocking native calls

Synchronous OS operation on sole VM thread may block all fibers:

```text
blocking call: scheduler cannot run
awaitable operation: current fiber suspends, scheduler continues
```

Standard library and FFI should classify this explicitly.

## 7. Cancellation

Define:

- who can cancel whom;
- when cancellation observed;
- dedicated outcome/error;
- cleanup execution;
- catchability;
- child propagation;
- native interruption behavior.

Without this, resource safety and prover assumptions are unstable.

## 8. Shared mutable state

Even one OS thread permits inter-fiber mutation across yield:

```phalcom
let old = shared.value
await event
use(shared.value)
```

Second value need not equal first if other fibers can mutate `shared`.

## 9. Fiber-local versus shared state

Facts about fiber-local immutable/local state can survive suspension more readily. Separate:

```text
fiber-local state
shared heap state
module/global state
external world
```

in effect/proof models.

## 10. Non-local return

A block home frame belongs to specific fiber. Define cross-fiber invocation and non-local-return rule. Never unwind another fiber's physical stack accidentally.

## 11. Uncaught exception

Decide whether it:

- terminates current fiber only;
- is stored in Future/result;
- propagates to joiner/parent;
- cancels children;
- terminates runtime.

Structured concurrency may later refine parent-child semantics.

## 12. Fairness

Fairness is a liveness property if user programs rely on eventual progress.

Possible policies:

```text
no guarantee
weak fairness for continuously runnable fibers
round-robin best effort
```

Native blocking can defeat scheduler fairness, so promises must match runtime.

## 13. Deadlock

Even cooperative fibers can deadlock through waits:

```text
f1 waits for future produced by f2
f2 waits for future produced by f1
```

Define whether runtime detects cycles, simply remains with no runnable fibers, or raises an error. This is observable behavior.

## 14. Channels/select/backpressure

If later introduced, semantics must cover:

- send/receive blocking/suspension;
- buffer capacity;
- closed channel behavior;
- selection fairness;
- cancellation of pending operations;
- ordering guarantees.

Do not bolt these onto fiber API without scheduler model.

## 15. Future parallelism

Parallel execution requires formal memory model:

- data-race semantics;
- atomics;
- visibility/order;
- FFI thread-safety contracts;
- heap/GC synchronization.

Avoid APIs whose documented atomicity depends accidentally on single-thread runtime if parallelism is anticipated.

## 16. Scheduler traces

Useful events:

```text
yield(f)
suspend(f, reason)
wake(f)
resume(f)
complete(f, outcome)
cancel(f)
```

Tests can assert allowed trace properties without fixing every internal choice.

## 17. Static effects

Track at least:

```text
mayYield
mayBlockThread
maySpawnFiber
mayCancel
mayAccessSharedState
```

`mayYield` is interference boundary for shared-state refinements.

## 18. Competency checks

1. Why can cooperative scheduling invalidate field fact across `await`?
2. Difference between blocking and suspending?
3. Is suspended home frame dead?
4. Which guarantees need memory model only with parallelism?
5. Why should fairness promises be explicit?
6. How can cooperative fibers deadlock without parallel threads?
