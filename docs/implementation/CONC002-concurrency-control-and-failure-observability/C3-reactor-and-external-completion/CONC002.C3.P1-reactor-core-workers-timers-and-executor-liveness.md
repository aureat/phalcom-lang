---
id: CONC002.C3.P1
program: CONC002
checkpoint: CONC002.C3
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
prepared: 2026-09-12
requires:
  - CONC002.C2.P1-R1 COMPLETE
  - CONC002.C2.P2 COMPLETE
---

# CONC002.C3.P1 — Reactor core, workers, timers, and executor liveness

## 0. Mission

Implement the accepted phase-1 reactor on top of the **post-C2 VM-owned executor**:

```text
registration registry
+ bounded worker pool
+ plain-data completion channel
+ safepoint ingress
+ monotonic timers
+ executor idle wait
+ System.sleep(Int) -> Future<Unit>
+ shutdown/leak integration
```

Do not resurrect the pre-C2 design in which guest Phalcom code manually pops completions and recursively drives scheduled Fibers.

The repository and final C2 implementation are authoritative. Rebase every structural anchor before editing.

## 1. Binding laws

1. No worker thread may touch `Value`, `ObjRef`, `Heap`, `VM`, Phalcom closures, or class/runtime objects.
2. Worker jobs and completions are owned plain data only.
3. Reactor registrations are generation-tagged and stale completions are harmless.
4. Pending registration ownership is a GC root for its Phalcom completion target.
5. Only the VM thread converts plain completion data into Phalcom values or settles a Phalcom completion object.
6. Settlement/wake never recursively executes the resumed Fiber.
7. Timers use a monotonic clock and no timer thread.
8. Ready-queue emptiness is not program completion when external progress is pending.
9. Reactor identity is separate from Fiber park/admission/control identities.
10. Public cancellation semantics are not implemented here.

## 2. Post-C2 integration requirement

C2.P2 owns executor control. C3 must first discover the exact final seam for:

```text
executor has no runnable guest
executor can ingest external completions
executor can wait for progress
executor can resume newly queued work
```

Preferred direction:

```text
VM executor
  -> drain bounded reactor ingress
  -> deliver completions through VM-visible settlement activation
  -> run queued Fibers
  -> if no runnable work:
       if reactor progress possible: block until completion/timer
       else: apply C2 no-progress/exit rule
```

Rejected architecture:

```text
guest System.runScheduled
  -> System.nextCompletion_
  -> settle Future in ad-hoc source pump
  -> System.parkForCompletion_
```

unless the post-C2 implementation proves those are genuinely VM-internal wrappers rather than guest ownership.

## 3. Checkpoint map

| Checkpoint | Boundary |
|---|---|
| R0 | rebase on post-C2 executor and pin integration seam |
| R1 | token registry and GC-root lifecycle |
| R2 | worker pool and plain-data completion transport |
| R3 | safepoint ingress and VM-thread completion materialization |
| R4 | monotonic timers and `System.sleep` |
| R5 | executor liveness/idle wait |
| R6 | shutdown/leak lifecycle |
| R7 | specification/floor/verification closure |

## 4. R0 — Post-C2 rebase

Inspect:

- final executor owner/drive type from C2.P2;
- `VM::ready_queue` / queue admission;
- wake seam;
- safepoint service;
- GC root enumeration;
- System native registration surface;
- canonical `Future<T>` settlement API.

Record exact answers:

```text
How does executor enter idle?
How is a queued Fiber selected?
What VM-visible mechanism can call Future.settleValue/settleError?
How does root/no-progress behavior differ before reactor progress exists?
Where may reactor roots be visited?
```

PLAN DRIFT if C2 already introduced an external-event interface.

## 5. R1 — Reactor token registry

STRUCTURAL model:

```rust
struct ReactorToken {
    slot: u32,
    generation: u32,
}

struct ReactorRegistration {
    generation: u32,
    state: RegistrationState,
    target: ObjRef,
    source: ReactorSource,
}
```

Exact representation may differ.

Requirements:

- O(1) slot lookup;
- generation increments on slot reuse/invalidation;
- stale completion does not observe a later registration;
- target is traced while registration live;
- target root is released on completion/invalidation/shutdown;
- registration cannot be completed twice;
- reactor token is never Fiber park generation.

Tests:

- allocate/reuse slot;
- stale token after reuse;
- double complete;
- forced GC retains target;
- completion/invalidation permits later collection.

## 6. R2 — Worker pool

Add one reactor-owned worker pool.

Requirements:

- lazy bounded worker creation;
- configurable only as internal constant for this phase;
- jobs contain owned plain data;
- completions contain token + owned plain outcome;
- workers have no VM reference;
- one MPSC back to VM;
- orderly stop intake and worker shutdown;
- no Phalcom heap handle crosses the channel.

Compile-time posture:

```rust
fn assert_send_static<T: Send + 'static>() {}
```

for all cross-thread job/completion payloads.

The type definitions themselves must structurally omit `Value`/`ObjRef`.

Use internal synthetic test jobs before filesystem APIs exist.

## 7. R3 — Safepoint ingress

At the existing dispatch safepoint or the post-C2 equivalent:

- non-blockingly drain a bounded batch of worker completions;
- validate token/generation;
- discard stale entries;
- move valid completions into VM-owned pending reactor events;
- do not execute guest code inside the channel-drain mutation phase;
- do not allocate Phalcom heap objects off-thread.

Completion delivery then occurs through the executor's normal VM-visible activation/settlement route.

Do not inspect Future private `_state`/`_value`/`_waiters`.

C4 may later change Future representation without requiring reactor rewrite.

## 8. R4 — Timers and `System.sleep`

Canonical API:

```phalcom
System.sleep(_ milliseconds: Int) -> Future<Unit>
```

Rules:

- non-negative integral milliseconds;
- negative/fractional/out-of-range input raises;
- monotonic clock;
- settles no earlier than deadline;
- `0` completes on a future executor turn, never inline;
- no timer thread.

Internal timer structure:

```text
min-heap(deadline, insertionSequence, token)
+
live token registry
```

Heap tombstones are validated against live token generation.

`.ph` wrapper:

```text
create pending Future<Unit>
register timer/future with one internal System/native seam
return Future
```

Native does not call Future settlement recursively.

If canonical parser/surface still uses `Number` rather than `Int` for milliseconds, resolve against `system.md` and numeric conventions during R0; do not silently widen.

## 9. R5 — Executor liveness

The accepted liveness law becomes executor logic.

Program/executor may report idle/complete only when:

```text
no runnable guest
AND no pending reactor registration
AND no undrained reactor event
```

When no guest is runnable but reactor progress is possible:

- block in reactor wait;
- bound wait by earliest timer deadline;
- worker completion wakes wait;
- on wake, ingest events and resume ordinary executor loop.

Phase-1 wait can use worker-channel timeout plus timer deadline. Do not add a poller dependency in P1.

Interaction with C2 no-progress:

```text
C2 no external progress source
  -> catchable no-progress logic

C3 pending reactor registration
  -> wait rather than inject no-progress
```

A registration is progress capability, not a proof that a particular Future will complete forever. Diagnostics/leak rules remain necessary.

## 10. R6 — Shutdown and leak lifecycle

On VM/process shutdown:

1. stop reactor intake;
2. drain cross-thread completion ingress once;
3. invalidate/drop live registrations;
4. stop worker pool;
5. join workers within the repository's shutdown policy;
6. release roots;
7. run resource/leak reporting.

Do not synthesize IO rejection merely to settle abandoned Futures during shutdown.

A registration that cannot complete and remains live must participate in leak reporting.

Public cancellation remains C5.

## 11. Required hostile tests

- liveness FIRST: only work is `System.sleep(...).await`;
- program does not exit with timer outstanding;
- program exits with no runnable/registration/completion;
- zero sleep is not inline;
- equal timer deadlines deterministic;
- stale timer heap entry ignored;
- worker round trip;
- worker completion while executor idle wakes progress;
- stale worker completion after token reuse ignored;
- double completion settles once;
- registered target survives forced GC;
- released registration no longer roots target;
- completion queues waiter but does not execute it inline;
- root result remains authoritative;
- timer target with no source reference still completes;
- shutdown with no work;
- shutdown with live registration reports/reclaims according to leak rules;
- plain-data compile-time boundary.

Use fake/injectable time/event sources. Do not make wall-clock timing assertions the correctness oracle.

## 12. Performance requirements

- no global heap scan to find registrations;
- no O(n) ready-queue scan on completion;
- bounded safepoint ingress batch;
- O(log n) timers;
- O(1) registration lookup/invalidation;
- no atomics/locks in VM heap/ready queue;
- worker synchronization confined to worker queues/wake primitive;
- no new runtime class per timer/registration.

## 13. Forbidden fixes

- no guest `.ph` completion pump as executor authority;
- no native Future-private-state mutation;
- no `Value` on worker threads;
- no immediate waiter execution from completion;
- no Fiber park generation reused as reactor token;
- no timer thread;
- no cancellation API;
- no socket/file public APIs;
- no `mio` in P1;
- no always-settled fake sleep Future.

## 14. Verification

Smallest-first:

```bash
cargo test -p phalcom-core --lib <reactor_registry_tests>
cargo test -p phalcom-core --lib <reactor_worker_tests>
cargo test -p phalcom-core --lib <reactor_timer_tests>
cargo test -p phalcom-core --test language-corpus <sleep_liveness_fixture>
cargo test -p phalcom-core --test core memory_gc
cargo test -p phalcom-core --test core native_surface_contracts
cargo test -p phalcom-core --test language-corpus corpus::concurrency
cargo test -p phalcom-core --lib
cargo test -p phalcom-core --test core
cargo test -p phalcom-core --test language-corpus
cargo fmt --all -- --check
cargo check --workspace --all-targets
```

Run broader workspace tests/clippy according to repository delivery policy after focused contracts pass.

## 15. Completion handoff

P1 completion report must state:

- registration identity/slot/generation model;
- GC roots and release points;
- worker plain-data types;
- safepoint ingress boundary;
- how VM-visible Future settlement is invoked;
- timer representation;
- exact executor idle/exit condition;
- shutdown behavior;
- the one internal System/native timer seam;
- what P2 must extend for poller-backed readiness.
