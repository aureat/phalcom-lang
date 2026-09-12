---
id: CONC002.C1.P3
category: CONC
program: CONC002
checkpoint: CONC002.C1
kind: verification-and-closure
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
depends_on: [CONC002.C1.P1, CONC002.C1.P2]
follows: CONC002.C1.P2
supersedes: null
deferred_reason: null
---

# CONC002.C1.P3 — Current-main concurrency architecture and type closure

## Goal

Close the gap between the fully implemented P1/P2 ownership repair and the later mainline Future/type changes. This plan is primarily a **recertification and specification-alignment checkpoint**, not a second runtime redesign.

P3 must establish, on one actual revision, that the current implementation simultaneously preserves:

1. the P1/P2 Fiber/scheduler/parking/completion invariants;
2. the current generic `Future<T>` type contract;
3. uniform callback scheduling and terminal observation;
4. map/then/recovery/flatten separation;
5. Unit/empty-tuple canonicalization required by concurrency source;
6. scheduler entry validation, scheduled-yield refusal and root-result retention;
7. the documented transitional limitations that C2 will deliberately remove.

## Non-goals

P3 does not implement VM-owned native control continuations, catch-at-call parent exception injection, manual coroutine pending-await, final `Fiber<R>` typing, reactor I/O, cancellation, structured concurrency, channels or select.

## 1. Baseline and evidence discipline

At execution start record:

```sh
git branch --show-current
git rev-parse HEAD
git status --short
```

Read the P1/P2 state record first. Historical P1/P2 passes are compatibility evidence, not proof of the later delta. Re-run only the focused lanes needed to certify the combined current revision, then broaden once at the final gate.

If the current tree differs semantically from the baseline recorded by this plan, update the source map before changing implementation. Do not normalize a failing type baseline merely to close P3.

## 2. Final Future semantic contract

`Future<T>` is invariant while public settlement consumes `T`.

Required source semantics:

```text
Future.value<T>(T)                 -> Future<T>
Future.error<T>(Error)             -> Future<T> with rejected state
Future.async<U>(() -> U)           -> Future<U>
Future<T>.await                     -> T
Future<T>.value                     -> Option<T>
Future<T>.map<U>((T) -> U)          -> Future<U>
Future<T>.then<U>((T) -> Future<U>) -> Future<U>
Future<T>.catch((Error) -> T)       -> Future<T>
Future<T>.recoverWith((Error) -> Future<T>) -> Future<T>
Future.flatten<U>(Future<Future<U>>) -> Future<U>
```

`map` preserves a returned Future as data when `U` itself is a Future type. `then` adopts exactly one returned Future layer. No runtime payload test decides mapping versus chaining. Direct self-adoption must reject rather than deadlock silently. Indirect dependency cycles are not required here.

A returned `Error`, `None`, `Unit` or nested Future may be ordinary successful data where the declared operation permits it. Lifecycle must never be inferred from those values.

## 3. Callback execution parity

Matching user callbacks for `then`, `map`, `catch` and `recoverWith` must execute as observed scheduler work whether the source Future was already settled or becomes settled later. Registration returns before invoking matching user code.

Required parity matrix:

| Source state | Callback outcome | Derived result |
|---|---|---|
| already fulfilled | value | fulfilled |
| later fulfilled | value | fulfilled identically |
| already fulfilled | Raise | rejected, not registration-time escape |
| later fulfilled | Raise | rejected identically |
| callback awaits repeatedly | eventual value | settle only at terminal return |
| callback returns Error as data | fulfilled with Error where type permits | no failure inference |

Unmatched already-settled branches may pass through without scheduling a callback because no user callable is invoked.

## 4. Scheduler and root closure

Verify the current scheduler contract together with the later changes:

- admission reserves a Fiber exactly once;
- zero-argument scheduler entry compatibility is checked before reservation;
- stale queue entries cannot abort later healthy work;
- an executor-owned Fiber cannot emit a user `yield` without a coroutine consumer;
- root program result survives any subsequent ready-queue drain and forced GC;
- detached scheduler failures retain the P2/E010 reporting behavior;
- completion-owned failures reach their Future and are not double-reported as detached work.

Do not broaden the scheduler contract to manual pending-await in P3.

## 5. Type-system closure

Verify the canonical semantic products rather than only executing source programs.

Required type assertions include:

```text
Future.value(42)                         : Future<Int>
Future.async(|| 42)                      : Future<Int>
Future.async(|| ())                      : Future<Unit>
Future.value(Future.value(42))           : Future<Future<Int>>
Future.value(1).map(|x| Future.value(x))  : Future<Future<Int>>
Future.value(1).then(|x| Future.value(x)) : Future<Int>
```

Also verify:

- `await` exposes the payload type;
- incompatible `settleValue` arguments fail;
- empty tuple value and empty tuple type annotation both normalize to Unit consistently, including nested generic positions;
- no unsaturated generic constructor is accepted as a proper value type merely to silence concurrency source diagnostics;
- no new concurrency diagnostics are added to a baseline suppression file.

## 6. Transitional limitations to document accurately

P3 closes documentation drift but does not remove these limitations:

1. `call` still uses the transitional linked terminal-failure cascade until C2.P1-R1 provides parent continuation injection.
2. native control helpers that retain Rust continuation state remain guarded until C2.P1-R1.
3. pending Future await is scheduler-owned; a manual coroutine cannot yet await and retain its consumer.
4. scheduled work cannot user-yield without a consumer.
5. root pending-await with an empty ready queue is an error because no external readiness source exists yet.

These are explicit next-checkpoint requirements, not accidental omissions.

## 7. Verification gates

Run serially with cleared wrapper/flags where the repository requires them.

### Gate P3.1 — focused semantic/type source

Run the exact source-backed semantic tests that assert Future constructor/combinator/await/nested/Unit types. Discover exact test names first if they have drifted.

### Gate P3.2 — concurrency positive/negative corpus

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency_negative -- --exact
```

Required cases must cover multi-await async, late failure, callback scheduling parity, nested Future mapping/adoption, direct self-adoption rejection, scheduler entry validation, scheduled-yield refusal, root-result retention and E010 compatibility.

### Gate P3.3 — runtime owner unit tests

Run the exact VM tests for:

- exact once-only park wake;
- completion observer detach/admission;
- detached completion-observer failure reporting;
- root result survival across scheduler drain/GC.

### Gate P3.4 — type/bootstrap diagnostics

Compile/bootstrap the canonical Universe and verify the concurrency source contributes no newly accepted baseline diagnostics. Assert Unit canonicalization through the semantic owner layer.

### Gate P3.5 — broad C1 closure

Run once after focused gates:

```sh
cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
```

Run the full workspace test only if it is part of the repository's current delivery gate. If the known REPL export blocker persists unchanged, classify it as the same baseline blocker rather than changing concurrency code.

## 8. Completion criteria

P3 becomes `COMPLETE / IMPLEMENTED` when the combined current-main behavior is freshly evidenced and canonical documentation matches it. Use `BASELINE_BLOCKED` rather than `RELEASE_COMPLETE` if an independently reproduced unrelated workspace baseline still blocks the global test gate.

P3 must leave C2 with a stable input contract, not with new implementation ambitions.

## 9. Recorded verification evidence

Evidence freshly executed on revision `2db7e3780772e847a178918ede239d91e0bcc5fd`:

| Gate | Scope | Command / Target | Result | Detail |
|---|---|---|---|---|
| **P3.1** | Generic `Future<T>` and `Unit` semantics | `cargo test -p phalcom-semantic --test semantic universe_future` | PASS | 2 passed (payload type preservation + incompatible settlement/recovery rejection) |
| **P3.1** | Canonical Unit identity | `cargo test -p phalcom-semantic --test semantic empty_tuple` | PASS | 2 passed (type model + nested generic annotations) |
| **P3.1** | Full semantic suite | `cargo test -p phalcom-semantic` | PASS | 1144 passed, 0 failed, 42 ignored (171.28s) |
| **P3.2** | Concurrency positive corpus | `cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --exact` | PASS | 1 passed, 0 failed (41.52s, full multi-await/callback/adoption coverage) |
| **P3.2** | Concurrency negative corpus | `cargo test -p phalcom-core --test language-corpus corpus::concurrency_negative -- --exact` | PASS | 1 passed, 0 failed (17.81s) |
| **P3.3** | Runtime completion observer | `cargo test -p phalcom-core --lib dispatch::tests` | PASS | 4 passed (terminal observer detach/admit, failing observer unhandled work, root result survival) |
| **P3.3** | Monotonic park wake generation | `cargo test -p phalcom-core --lib scheduler_tests` | PASS | 2 passed (parked wake requires exact generation and is once-only, unhandled failure survives GC) |
| **P3.4** | Canonical Universe diagnostics | `cargo test -p phalcom-core --lib modules::canonical_semantics` | PASS | 3 passed (baseline lock active; 0 concurrency diagnostics in `semantic-diagnostics-baseline.txt`) |
| **P3.5** | Core unit tests | `cargo test -p phalcom-core --lib` | PASS | 110 passed, 0 failed |
| **P3.5** | Core integration suite | `cargo test -p phalcom-core --test core` | PASS | 468 passed, 0 failed, 24 ignored (184.13s) |
| **P3.5** | Code formatting | `cargo fmt --all -- --check` | PASS | Clean |
| **P3.5** | Workspace build | `cargo build --workspace --all-targets` | PASS | Clean across all crates |
| **P3.5** | Workspace lints | `cargo clippy --workspace --all-targets -- -D warnings` | PASS | Clean (unnecessary cast in test probe corrected) |
| **P3.5** | Modules suite | `cargo test -p phalcom-modules` | PASS | All unit, integration, and doc tests green |
| **P3.5** | Workspace delivery audit | `cargo test --workspace --all-targets` | BASELINE_BLOCKED | REPL import/export defect (12 failures in `repl_imports`) reproduces unchanged; 2 language-corpus non-concurrency baseline failures (`family`, `streams`). Concurrency products certified green. |

