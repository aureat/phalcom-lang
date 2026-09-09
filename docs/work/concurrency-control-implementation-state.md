# Concurrency Control Implementation State

## Repository state

- branch: `codex/concurrency-control-remediation`
- HEAD: `e8628a46` plus the final-gate ledger changes below
- relevant local changes preserved: baseline audit, plan, and unrelated files are in the pushed parent commits

## Established invariants

- CONC-STATE-01: `Running` identifies the VM's current Fiber; a caller parked behind a child is `BlockedOnChild`.
- CONC-STATE-02: `BlockedOnChild` is not manually resumable.
- CONC-STATE-03: `New` and explicit `Yielded` are distinct lifecycle states.
- CONC-ROOT-01: root identity is stored on the Fiber and does not depend on `resumer`.

## Decisions

- D-01 through D-11 remain as ratified in `docs/implementation/INBOX/phalcom-concurrency-control-patch-grade-implementation-plan.md`.
- C0 uses `FiberStatus::{New, Running, BlockedOnChild, Yielded, Parked, Queued, Done, Failed}`; C1 owns `Queued` and C2 owns generation-tagged `Parked`.
- Manual `call`/`try` accepts only `New` and `Yielded` in C0.
- Scheduler admission is VM-owned: `New`/`Yielded` transition atomically to `Queued`; duplicate, parked, active, blocked, and terminal fibers are rejected.
- Scheduler dequeue is FIFO and skips stale non-`Queued` entries without queue scanning or duplicate admission checks.
- Scheduler resume has an explicit internal mode and is used by both the root-drive pump and `System.runScheduled`; public `Fiber#try` remains a manual path.
- Future parking uses a Fiber-local monotonic generation and internal prepare/park operations; `Parked(generation)` cannot be manually resumed or publicly scheduled.
- Future waiter tuples carry `(Fiber, generation)` and settle through exact ticket-aware wake; stale and duplicate wakes are no-ops.
- Root await pumps through the C1 internal scheduler dequeue/resume path; non-root await rejects manual ownership and refuses native re-entry before registration.
- Fiber completion ownership is a single GC-traced observer handle, detached exactly once at `Done`/`Failed` and admitted as fresh scheduler work.
- Terminal result access is independent of `resumer`; call-mode failure cascades notify every terminalized observer-bearing Fiber.
- `Future.async` and pending `then`/`map`/`catch` use terminal observers; parked action/callback turns do not settle derived Futures.
- Callback terminal values are flattened/adopted only after terminal success, preserving `None` and `Error` as ordinary successful data.
- Public scheduler authority is limited to `System.schedule(_)` and `.ph` `System.runScheduled`; raw dequeue, scheduler resume, exact wake, park, and completion-observer operations are internal.
- `System.nextScheduled` is removed from the public native surface; queued work cannot be stolen into a public manual `Fiber#try` path.

## Evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|
| Baseline | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo build -p phalcom-core` | PASS | clean starting runtime build |
| Baseline | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture` | PASS, 2 tests | pre-change concurrency corpus |
| C0 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core` | PASS | exhaustive lifecycle caller migration compiles |
| C0 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture` | PASS, 2 tests | manual coroutine compatibility, stable root identity, and blocked-parent rejection |
| C1 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core` | PASS | scheduler admission/dequeue and explicit resume-mode fanout compile |
| C1 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture` | PASS, 2 tests | FIFO, nested scheduling, scheduled-failure isolation, duplicate admission, queued manual-resume rejection, and existing concurrency regressions |
| C2 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib scheduler_tests -- --nocapture` | PASS, 1 test | exact park generation and once-only wake authority |
| C2 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture` | PASS, 2 tests | ordinary await, repeated park generations, wrong-authority rejection, manual-await policy, native-boundary compatibility, and existing concurrency regressions |
| C3 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core` | PASS | observer field, traced edge, internal binding/result seams, and centralized terminal handoff compile |
| C3 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib terminal_observer_is_detached_and_admitted_once -- --nocapture` | PASS, 1 test | observer detaches once and becomes a queued fresh Fiber |
| C3 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture` | PASS, 2 tests | C0–C2 behavior remains green with observer infrastructure installed |
| C4 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture` | PASS, 2 tests | multi-await async completion, late failure, call cascade, forced-GC observer retention, suspending continuation matrix, nested Future adoption, and existing compatibility fixtures |
| C5 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture` | PASS, 2 tests | public raw-pop removal, scheduler ownership closure, terminal settlement, exact wake, and all concurrency regressions |
| C5 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core object_model_invariants::floor_census_matches_installed_bindings` | PASS, 1 test | native floor census matches the 226 installed public/internal bindings |
| C5 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core` | PASS, 456 passed, 29 ignored | complete core integration target, including scheduler/fiber object-model invariants |
| C5 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus` | PASS, 61 passed, 4 ignored | complete language corpus, including concurrency and negative fixtures |
| Final | `cargo fmt --all -- --check` | BASELINE-BLOCKED: only `phalcom-modules/tests/workspace_session.rs` remains unformatted; concurrency files were formatted in `e8628a46` | repository formatting hygiene outside the patch scope |
| Final | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets` | PASS, existing warnings only | all workspace targets compile with the patch |
| Final | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets` | BASELINE-BLOCKED: 3 failures in `phalcom-repl/tests/repl_import_bugs.rs`; core target and language corpus remain green | workspace-wide behavioral baseline; no failure was in the concurrency targets |
| Final | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings` | BASELINE-BLOCKED: 5 findings in `phalcom-modules` (`unnecessary_map_or` x2, `too_many_arguments`, `type_complexity`, `collapsible_if`) | workspace lint baseline outside the patch scope |

## Negative/deletion gates

- `rg 'FiberStatus::Suspended' phalcom-core/src` → no matches.
- `rg 'resumer\.is_none\(\)' phalcom-core/src/primitive/fiber.rs` → no matches.
- `rg 'ready_queue\.(push_back|pop_front)' phalcom-core/src` → matches only VM-owned admission/dequeue helpers.
- `rg '_waiters.*Fiber|System\.schedule\(rawAwaiter\)' phalcom-core/core/universe/src/concurrency/fiber.ph` → no raw Fiber waiter path; await stores ticketed tuples.
- `completion_observer` is traced from `Object::Fiber` and is taken before observer scheduling; no Rust closure or Future-specific heap pointer is stored.
- `rg 'const res = fib\.try|const driver = Fiber\.new' phalcom-core/core/universe/src/concurrency/fiber.ph` → no one-turn Future driver remains.
- `rg 'nextScheduled' --glob '!target/**'` → only internal `_$nextScheduled` implementation/spec references and historical audit/context references remain; no public `System.nextScheduled` declaration or installed public surface record remains.
- `cargo fmt --all -- --check`, workspace tests, and workspace clippy are not release-green because of the unrelated baseline findings recorded in the final evidence ledger; no concurrency-focused gate is blocked.

## Deferred gates

- scheduler admission/resume → C1 complete
- ticketed Future parking/wake → C2 complete
- durable completion observers → C3 complete
- terminal Future adoption, settlement, and suspension-safe continuations → C4 complete
- ownership closure, specs, and permanent regressions → C5 complete
- format, workspace build/test/clippy → final gate pending

## Unexpected findings

- C5 removed the public `System.nextScheduled` compatibility getter after all Future consumers migrated; only internal scheduler dequeue/resume and exact wake/park/observer seams remain.
- Internal observer selectors are reserved to the core/runtime implementation; end-to-end language fixtures exercise them through the Future consumer.

## Active incident

Final workspace certification remains blocked by unrelated formatting, REPL import, and `phalcom-modules` clippy findings. E010 remains an intentional open concurrency-policy issue; it was not silently closed by C5.

## Next resume action

Run the final format, workspace build, workspace test, and workspace clippy gates; classify any clean-baseline blockers separately from the concurrency evidence above.
