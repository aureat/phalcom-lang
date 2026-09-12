---
id: CONC002.C4.P1
program: CONC002
checkpoint: CONC002.C4
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
prepared: 2026-09-12
requires:
  - CONC002.C2.P2 COMPLETE
---

# CONC002.C4.P1 — Future state, readiness registrations, and CompletionSource

## 0. Mission

Turn the already-typed `Future<T>` API into a precise, maintainable library substrate without changing C2 runtime ownership.

Current main already has:

- invariant `Future<T>`;
- `async`, `map`, `then`, `catch`, `recoverWith`, `flatten`;
- scheduled callback execution;
- ticketed Fiber waiting;
- durable terminal observers;
- Unit canonicalization.

P1 does not redo those achievements.

It owns the remaining library representation debt:

```text
string/Dynamic Future state
mixed waiter storage
no detachable readiness subscription
no producer capability wrapper
Backoff validation/calculation gaps
```

## 1. Non-goals

- reactor/timer implementation — C3;
- final Fiber typing — C2.P2;
- cancellation/TaskScope — C5;
- Channel/select — C6;
- Tracer/OffBehavior redesign;
- Future covariance while public mutation consumes `T`;
- general unobserved-Future-rejection subsystem unless required by P2 composition.

## 2. Target contracts

### Future state

Preferred:

```text
FutureState<T> =
  Pending
  Fulfilled(T)
  Rejected(Error)
```

Acceptable fallback if current private ADT field support is unsuitable:

```text
Option<Result<T, Error>>
```

Requirements:

- no `None` as placeholder T;
- no string state comparisons;
- no Dynamic payload state internally if typed representation is supported;
- one terminal outcome installed once.

### Settlement

One internal transition:

```phalcom
_$trySettle(_ outcome: Result<T, Error>) -> Bool
```

Public compatibility:

```phalcom
settleValue(_ value: T) -> Future<T>
settleError(_ error: Error) -> Future<T>
```

delegate to the same transition.

### Inspection

```phalcom
isReady -> Bool
value -> Option<T>
outcome -> Option<Result<T, Error>>
```

`value` is Some only for fulfillment.

Returned `Error` payload remains fulfillment data.

## 3. Checkpoints

| Checkpoint | Boundary |
|---|---|
| F0 | rebase on C2/C3-facing Future contract |
| F1 | typed state + single settlement transition |
| F2 | explicit readiness registration model |
| F3 | detachable subscriptions and retention bounds |
| F4 | callback/combinator integration |
| F5 | CompletionSource |
| F6 | pure Backoff policy |
| F7 | packaging/type/GC verification |

## 4. F0 — Rebase

Inspect final:

- `Future<T>` source after C2;
- C2 park/wake selectors;
- C3.P1 settlement target contract if C3 work has begun;
- semantic generic tests;
- package export/bootstrap arrangement.

Do not couple C4 to C3's registry internals.

The stable C3-facing contract is ordinary Future settlement/readiness behavior, not `_state` layout.

## 5. F1 — Typed state and settle-once

Implement one internal state.

Settlement sequence:

1. check Pending;
2. install complete terminal outcome;
3. detach/swap current registration collection;
4. deliver detached registrations;
5. loser settlement returns false/no-op.

No user callback or suspension may occur between steps 1 and 2.

Hostile tests:

- value/value conflict;
- value/error conflict;
- error/value conflict;
- error/error conflict;
- Unit payload;
- None payload;
- Error payload;
- nested Future payload;
- forced GC after settlement before observation.

## 6. F2 — Explicit registration variants

Replace tuple-vs-closure runtime guessing.

Conceptually:

```text
ReadinessRegistration =
    ParkedFiber {
        id,
        fiber,
        parkGeneration,
        state
    }
  | Callback {
        id,
        callback,
        state
    }
```

Lifecycle:

```text
Registered
Queued/Delivered
Detached
```

Exact source representation may use enum/record classes or another typed private structure.

Requirements:

- identity per registration;
- stale park generation harmless;
- active registrations traced normally;
- no registration delivered twice;
- settlement swaps active collection before walking it;
- reentrant terminal registration cannot be lost.

## 7. F3 — Detachable subscription

Provide package-internal API conceptually:

```text
subscribeReady(callback) -> DetachHandle
```

and/or a typed internal registration object.

Detachment:

- idempotent;
- releases callback/source captures when safe;
- before settlement prevents future queue/admission;
- after queue admission causes queued wrapper to no-op and release captures;
- does not scan all registrations per completion;
- repeated attach/detach does not retain unbounded tombstones.

Use indexed records/tombstones plus amortized compaction or an equally bounded structure.

For parked Fiber waiters, preserve the C2 exact registration/park ordering:

```text
prepare park
register exact Fiber + park generation
commit park
```

If park commit fails, detach the registration.

## 8. F4 — Existing combinator integration

Preserve:

```text
map       = value transform; nested Future is data
then      = explicit one-layer adoption
catch     = recovery value
recoverWith = explicit recovery adoption
flatten   = explicit then-based adoption
```

Matching user callbacks run under scheduler/executor ownership whether the source is already ready or becomes ready later.

Internal readiness fast path may schedule work immediately, but may not invoke arbitrary user callback inline.

Derived Future completion remains owned by a terminally observed action.

Keep direct self-adoption rejection.

Do not claim indirect dependency-cycle detection unless separately implemented.

## 9. F5 — CompletionSource<T>

Add:

```phalcom
class CompletionSource<T> {
  future -> Future<T>
  tryResolve(_ value: T) -> Bool
  tryReject(_ error: Error) -> Bool
}
```

Requirements:

- creates exactly one Future;
- repeated `future` reads return same identity;
- no duplicate state;
- try methods delegate to Future's one settlement transition;
- wrong payload rejected statically;
- conflicting producers have one winner.

Public Future settlers remain compatibility API for now. Therefore CompletionSource is a producer-ownership convention, not yet an enforced capability boundary.

If a future breaking change removes public settlers, variance can be reconsidered then—not in P1.

## 10. F6 — Backoff pure policy

Current Backoff remains in concurrency source but timed waiting is C4.P2.

P1 fixes pure policy only.

Validate:

```text
none
fixed(ms >= 0)
exponential(base >= 0, max >= base)
attempt >= 0
raw kind validity
```

Define explicit attempt convention and use it consistently.

Recommended pure calculation:

```text
retryIndex = max(attempt - firstRetryIndex, 0)
delay = min(max, base * 2^retryIndex)
```

Use overflow-safe saturation; once cap reached, stop multiplying.

No O(attempt) loop after saturation.

No host sleeping.

## 11. Tracer and OffBehavior scope correction

Do not redesign Tracer/OffBehavior here.

Their comments identify decorator/feature-flag consumers. Move future semantic improvements to the corresponding language/decorator plans.

If source-file packaging is split, move definitions mechanically with exports/tests, but keep behavior unchanged.

## 12. Type/GC tests

Semantic:

- exact Future<T> identities;
- invariance;
- outcome types;
- CompletionSource inference;
- wrong producer payloads.

Runtime:

- early/late registration;
- detach before settlement;
- detach after queue admission;
- self-detach;
- reentrant registration;
- stale parked waiter;
- many waiters;
- bounded retention after repeated detach;
- callback parks;
- callback failure;
- GC across parked wait and callbacks.

## 13. Verification

```bash
cargo test -p phalcom-semantic <future_type_tests>
cargo test -p phalcom-semantic
cargo test -p phalcom-core --test language-corpus corpus::concurrency
cargo test -p phalcom-core --lib <future_registration_tests>
cargo test -p phalcom-core --test core memory_gc
cargo check -p phalcom-core
cargo fmt --all -- --check
```

Run broader core/language/workspace gates after focused semantics pass.

## 14. Completion criteria

P1 complete only when:

- Future state has one typed terminal representation;
- all settlement routes share one transition;
- readiness registrations are explicit;
- detach is bounded/idempotent;
- existing combinators use the new readiness substrate without timing regressions;
- CompletionSource exists;
- Backoff pure policy is validated;
- Tracer/OffBehavior are no longer treated as C4 semantic work;
- no C3/C5/C6 mechanism was introduced.
