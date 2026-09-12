---
id: CONC002.C4.P2
program: CONC002
checkpoint: CONC002.C4
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
prepared: 2026-09-12
requires:
  - CONC002.C4.P1 COMPLETE
partial_dependencies:
  - Future.timeout and timed Backoff require CONC002.C3.P1 COMPLETE
---

# CONC002.C4.P2 — Future composition, timeout, and Backoff

## 0. Mission

Build higher-level concurrency library operations on C4.P1's explicit readiness-registration substrate.

P2 owns:

```text
Future.all
Future.allSettled
Future.race
Future.timeout
Backoff timed waiting
```

It owns no reactor mechanism, TaskScope/cancellation, Channel, or select.

## 1. Dependency split

Aggregation can land after C4.P1.

Time-dependent work requires C3.P1:

```text
Future.timeout
Backoff.waitBefore timed behavior
```

Do not block aggregation on the poller PDR/C3.P2.

## 2. Future.all

```phalcom
Future.all<T>(_ inputs: List<Future<T>>) -> Future<List<T>>
```

Requirements:

- snapshot input list;
- register every input eagerly;
- preserve input order;
- empty input -> fulfilled empty List;
- first observed rejection settles output;
- no sequential-await implementation;
- repeated same Future at multiple indices remains multiple logical entries.

Coordinator owns:

```text
output
remaining count
per-index completion bit
Option<T> slots
detach handles
failure-observation responsibilities
```

O(n) state and O(1) work per completion, aside from final ordered assembly.

## 3. Future.allSettled

```phalcom
Future.allSettled<T>(
  _ inputs: List<Future<T>>
) -> Future<List<Result<T, Error>>>
```

Requirements:

- wait for every input;
- preserve input order;
- rejection becomes `Err(error)` data;
- empty -> fulfilled empty list;
- duplicate input identities still occupy separate indices;
- no output rejection merely because an input rejected.

## 4. Future.race

```phalcom
Future.race<T>(_ inputs: List<Future<T>>) -> Future<T>
```

Requirements:

- first observed terminal outcome wins;
- rejection wins equally with fulfillment;
- empty input -> immediately rejected output with clear argument error;
- ready inputs inspected deterministically in input order;
- pending completions use executor observation order;
- no claim about wall-clock simultaneity.

## 5. Initialization race law

Already-ready sources may notify during subscription setup.

Therefore:

1. allocate complete coordinator record first;
2. allocate slots/bitsets/handle capacity;
3. then subscribe inputs;
4. callbacks may only observe fully initialized coordinator state.

Do not append a detach handle after a callback could already have won and tried to detach all losers without that slot existing.

## 6. Loser retention / failure observation

Early `all` rejection and `race` victory should release unnecessary value-retaining captures.

But detaching a loser must not silently lose an otherwise unowned failure.

Before implementation, inspect actual Future rejection-observation policy.

If no general unobserved-Future-rejection reporter exists:

- retain minimal failure-observation registration for losers;
- release value payload/coordinator captures not required for error observation;
- document that scheduler E010 does not automatically cover Future rejection.

Do not build a global dependency graph.

## 7. Future.timeout

Requires C3.P1 `System.sleep`.

```phalcom
future.timeout(_ milliseconds: Int) -> Future<T>
```

Semantics:

- wrapper only; source is not cancelled;
- invalid duration rejected according to System.sleep validation;
- source and timer race through one C4 registration arbiter;
- winning source result/rejection settles output;
- winning timer rejects output with a dedicated timeout error according to the chosen error taxonomy;
- losing subscription detaches;
- loser failure observation responsibility remains owned.

Tie rule:

```text
if source is already terminal when timeout is called:
    source wins without waiting for a timer turn
else if duration == 0:
    timer may win on the next executor turn
```

Ratify exact error class/name before implementation; do not invent a generic Error string if a timeout error type already exists/has a spec owner.

## 8. Backoff timed integration

C4.P1 supplies validated pure delay calculation.

P2 implements actual waiting:

```text
delay = backoff.delayFor(attempt)
if delay == 0:
    return ()
System.sleep(delay).await
()
```

Preferred result:

```phalcom
waitBefore(_ attempt: Int) -> Unit
```

Current source returns `Option<Never>` as a workaround for the no-delay `None` path. If that API is already compatibility-sensitive, make the migration explicit and update all consumers/examples together.

Do not:

- host-block;
- busy-wait;
- turn System.sleep into synchronous Unit;
- retain the old “unimplemented timer” Error after C3 exists.

## 9. Aggregation hostile matrix

- empty all/allSettled/race;
- one input;
- all already fulfilled;
- already-settled rejection;
- pending first input + later rejection;
- out-of-order fulfillment;
- duplicates;
- multiple simultaneous terminal notifications;
- callback reentrancy during subscription;
- GC while coordinator pending;
- loser failure after race winner;
- all early rejection followed by later loser failures;
- payload `None`;
- payload `Error`;
- payload `Future<U>` as data.

## 10. Timeout hostile matrix

- source ready before call;
- source settles before positive deadline;
- deadline wins;
- zero timeout on pending source;
- source rejects before deadline;
- source rejects after timeout won;
- timer registration becomes stale/detached;
- GC during timeout race;
- repeated timeout wrappers around one source;
- nested timeout;
- root and non-root await.

Use fake C3 clock/event injection, not wall-clock sleep assertions.

## 11. Backoff hostile matrix

- none;
- fixed 0;
- fixed positive;
- exponential base 0;
- exponential normal growth;
- cap saturation;
- huge attempt index without O(k) loop;
- overflow boundary;
- invalid constructor/raw kind;
- negative attempt;
- timing integration requests exact calculated delay through fake timer seam.

## 12. Non-goals

- Task/TaskScope;
- cancellation;
- queue-admission revocation;
- Channel;
- select;
- mapConcurrent;
- reactor token APIs;
- poller;
- Fiber typing.

## 13. Verification

Focused:

```bash
cargo test -p phalcom-semantic <future_aggregation_type_tests>
cargo test -p phalcom-core --test language-corpus <future_aggregation_lane>
cargo test -p phalcom-core --lib <future_registration_coordinator_tests>
```

With C3.P1:

```bash
cargo test -p phalcom-core --test language-corpus <future_timeout_lane>
cargo test -p phalcom-core --test language-corpus <backoff_time_lane>
```

Then:

```bash
cargo test -p phalcom-semantic
cargo test -p phalcom-core --test language-corpus corpus::concurrency
cargo test -p phalcom-core --test core
cargo check --workspace --all-targets
cargo fmt --all -- --check
```

## 14. Completion criteria

P2 complete when:

- all/allSettled/race are eager, typed, order-correct and retention-bounded;
- loser failure responsibility is explicit;
- timeout composes with C3 timers without cancelling source;
- Backoff waits using Future-shaped sleep;
- time tests are deterministic;
- no C5/C6 policy leaked into C4;
- old C3 N0/N1/N3 library requirements are fully migrated.
