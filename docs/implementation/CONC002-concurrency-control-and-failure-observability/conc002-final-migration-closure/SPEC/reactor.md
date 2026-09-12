# Specification — The Reactor (completion machinery for Future-shaped IO)

> **Status:** Normative machinery contract.
>
> Governing accepted decisions:
> - PDR-0003 — worker/VM-thread discipline;
> - PDR-0004 — Future-shaped blocking operations and reactor ownership.
>
> **Implementation owner:** CONC002.C3.
> - Phase 1: `CONC002.C3.P1-reactor-core-workers-timers-and-executor-liveness.md`
> - Poller phase: `CONC002.C3.P2-poller-and-external-readiness.md`
>
> PDR-0016 remains Proposed, therefore the poller phase is not yet dispatchable.
>
> **Implementation status:** unbuilt at this spec revision.
>
> The pre-C2 proposal that required guest/source-visible
> `System.nextCompletion_` and `System.parkForCompletion_(_)` pump seams is
> withdrawn. The post-C2 VM executor owns reactor ingress and idle waiting.

## 1. Role

The reactor is the external-progress source below every Future-shaped operation
whose completion depends on the operating system.

Mechanisms:

| Source | Mechanism |
|---|---|
| pollable descriptors | one single-VM-thread poller |
| timers | monotonic reactor timer heap |
| blocking host work / regular files | bounded worker pool |

User code sees Futures, not worker threads, tokens, poller events, or reactor registrations.

## 2. Thread discipline

The following are absolute laws:

1. Workers receive and return owned plain data only.
2. No worker may see `Value`, `ObjRef`, VM state, the Phalcom heap, Phalcom closures, or runtime objects.
3. Worker completions cross to the VM over a plain-data channel.
4. Only the VM thread creates Phalcom values, settles Futures, mutates the ready queue, or runs guest code.
5. The ready queue and heap remain single-threaded; no atomics are added merely for the reactor.

The job/completion types should make violations structurally difficult.

## 3. Registration identity and GC lifetime

Every external operation owns a generation-tagged reactor registration.

Conceptually:

```text
ReactorToken = (slot, generation)
```

This token is distinct from:

```text
Fiber identity
Fiber park generation
scheduler admission identity
C2 control/frame identity
future C5 cancellation generation
```

The reactor registry roots the Phalcom completion target while the registration is live.

For Future-shaped operations the ordinary object graph may be:

```text
reactor registration
 -> Future/completion target
   -> readiness registrations
     -> parked Fiber
```

A live registration is released only by:

```text
completion
explicit invalidation/deregistration
shutdown
```

Slot reuse changes generation. A completion carrying an old generation is ignored.

## 4. Completion lifecycle

```text
submit/register
      ↓
external progress
      ↓
plain completion/readiness
      ↓
VM ingress validates token
      ↓
VM-owned completion delivery
      ↓
ordinary Future settlement
      ↓
matching Future waiter wake
      ↓
Parked(generation) -> Queued
      ↓
later executor turn
```

### Submit

A VM/native host operation:

- creates or receives a pending Future/completion target;
- allocates a fresh registration token;
- copies only plain data into worker/poller/timer state;
- records the target in the VM-thread registry;
- returns without blocking.

### External completion

Workers return token plus plain data.

Poller readiness and timers are observed on the VM thread.

Nothing off-thread settles a Future.

### Ingress

Cross-thread completion transport is drained at a VM executor/dispatch safepoint in a bounded batch.

Ingress validates generations and drops stale events before guest settlement work is activated.

### Settlement

Completion reaches Future through the VM's ordinary visible activation/dispatch path.

The reactor does not depend on Future's private state representation.

C4 may therefore change Future internals without changing C3.

### Wake

Future settlement may authorize an exact parked Fiber generation to become `Queued`.

Wake never executes that Fiber inline.

## 5. Executor liveness

The post-C2 VM executor owns the progress loop.

The runtime may finish only when:

```text
no runnable guest
AND no pending reactor registration
AND no undrained reactor event
```

If no guest is runnable but reactor progress is possible, the executor waits for external progress or the nearest timer deadline.

If no guest is runnable and no reactor progress source exists, C2's ordinary exit/no-progress rule applies.

## 6. Fairness

Fairness remains an implementation policy until separately ratified.

Default:

- never preempt the current guest;
- ingest a bounded batch;
- woken Fibers join the back of the ready queue;
- events beyond the batch boundary wait for a later round.

No language-level priority guarantee is implied.

## 7. Timers — `System.sleep(_)`

Target surface:

```phalcom
System.sleep(_ milliseconds: Int) -> Future<Unit>
```

The older unparameterized-Future / `Ok(None)` wording is superseded by generic Future and canonical Unit.

Rules:

- non-negative integral milliseconds;
- negative/out-of-range values raise;
- monotonic clock;
- settlement occurs no earlier than the deadline;
- `sleep(0)` completes on a future executor turn, never inline;
- no timer thread;
- equal-deadline ordering is deterministic within the implementation.

A `.ph` wrapper may create `Future<Unit>`, register it through one internal System/native timer seam, and return it.

The registration primitive does not synchronously settle the Future.

## 8. Worker pool

Regular-file and other genuinely blocking host operations use a bounded worker pool.

Requirements:

- bounded worker count;
- owned plain-data jobs/completions;
- one completion transport to VM;
- orderly shutdown;
- pool size is an implementation constant in this phase, not a public knob.

The reactor must prove worker liveness with a synthetic internal job before user filesystem APIs depend on it.

## 9. Poller phase

PDR-0004 requires a real poller for pollable descriptors.

Backend choice is not accepted yet.

PDR-0016 currently proposes:

```text
mio
confined to the reactor module
try syscall first
register only on WouldBlock
generation remains reactor-owned
```

C3.P2 may implement that backend only if PDR-0016 is accepted unchanged or a replacement ruling is ratified.

Whatever backend is chosen:

- no second executor/runtime;
- readiness feeds the same registration/executor path;
- stale readiness is harmless;
- backend types remain confined to the reactor seam.

## 10. Deregistration and future cancellation

C3 ships the mechanism required to invalidate registrations:

```text
invalidate generation
remove poller interest if any
release target root
ignore later stale completion
```

This is resource correctness, not user cancellation semantics.

Public Future/Task cancellation belongs to C5 and must reuse this substrate.

## 11. Shutdown

On VM/process shutdown:

1. stop reactor intake;
2. drain completion ingress once;
3. invalidate/remove live registrations;
4. stop/join workers according to runtime policy;
5. release reactor roots/state;
6. run resource drain/leak reporting.

Do not synthesize arbitrary IO failures merely to settle pending Futures during process shutdown.

## 12. Laws

1. no Phalcom heap value crosses a worker-thread boundary;
2. VM-thread-only settlement;
3. stale external generations are harmless;
4. settlement remains at-most-once;
5. registrations root their completion targets;
6. timers are monotonic and lower-bounded;
7. wake authorizes later execution, never inline guest execution;
8. exit uses the three-way conjunction in §5;
9. C3 is independent of Future private storage;
10. cancellation semantics are not defined here.

## 13. Conformance

| Check | Requirement |
|---|---|
| sleep-only liveness | `System.sleep(...).await` progresses with no ready work |
| exit exactness | pending registration prevents early exit |
| worker round trip | worker completion settles on VM thread |
| settle once | duplicate completion cannot settle twice |
| stale token | old generation is ignored |
| timer ordering | deterministic equal/deadline behavior |
| zero timer | never inline |
| plain-data boundary | cross-thread types contain no VM/heap handles |
| GC parked target | live registration retains completion graph |
| root release | completion/deregistration stops reactor retention |
| wake separation | completion queues work but does not execute it |

Poller-specific conformance belongs to C3.P2.

## 14. Open questions

| Question | Status |
|---|---|
| fairness policy | default only, not language-ratified |
| worker-pool size | bounded internal constant; measure |
| poller backend | PDR-0016 Proposed |
| public cancellation | C5 |
| exact internal timer selector spelling | C3.P1 implementation detail/native-census decision |

## 15. Out of scope

- filesystem/network/process user selector design;
- TLS;
- structured cancellation semantics;
- channels/select;
- isolates;
- streaming/backpressure;
- parallel executor.
