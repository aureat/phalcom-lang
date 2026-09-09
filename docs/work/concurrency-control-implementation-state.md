# Concurrency Control Implementation State

## Repository state

- branch: `codex/concurrency-control-remediation`
- runtime implementation commit: `914d38bb`
- baseline diagnosis commit: `0a8658ac`
- current exact HEAD is represented by the pushed branch history; this record avoids duplicating a hash that becomes stale when documentation is amended

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
| Final | `cargo fmt --all -- --check` | PASS after formatting the three existing drift files | workspace formatting hygiene |
| Final | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets` | PASS, existing warnings only | all workspace targets compile with the patch |
| Final | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --all-targets` | PASS, all module targets | module-session/source changes remain behaviorally green |
| Final | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy -p phalcom-modules --all-targets -- -D warnings` | PASS | all five requested `phalcom-modules` findings are fixed |
| Final | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-repl --test repl_import_bugs -- --nocapture` | BASELINE-BLOCKED: 3 failures, 3 passes; see diagnosis below | reproduced the unrelated REPL import baseline independently |
| Final | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets` | BASELINE-BLOCKED: the same 3 failures in `phalcom-repl/tests/repl_import_bugs.rs`; core target and language corpus remain green | workspace-wide behavioral baseline; no failure was in the concurrency targets |
| Final | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings` | BASELINE-BLOCKED after the module fix: 36 findings in `phalcom-semantic`; the requested five `phalcom-modules` findings no longer appear | workspace lint baseline beyond the requested module scope |

## ASTRA audit

- E007 is fixed: terminal observers settle `Future.async` only after terminal completion; the multi-await regression covers pending turns and the final value.
- E008 is fixed: VM-owned admission rejects duplicate scheduling; the regression covers one execution and continued healthy work.
- E010 is correctly open: scheduled failure isolation does not provide general reporting for failed fire-and-forget tasks.
- E010 documentation now describes scheduler-mode resume and captured-error loss; its OPEN status is unchanged.
- Audit limitation: ASTRA reviewed this state and cited evidence but did not independently rerun the recorded commands; the implementation run above is the fresh verification record.

## REPL baseline diagnosis

The focused target reproduces three failures and three passes:

- `selective_import_selector_class_is_non_none`: `from universe.errors.unsupported import unsupported` is rejected because the runtime `universe:errors.unsupported` module has no installed `unsupported` export.
- `module_import_selector_property_access_is_non_none`: `import universe.errors.unsupported` succeeds, but `unsupported.unsupported` reaches module `doesNotUnderstand` for the same missing runtime export.
- `universe_root_exports_package_info`: `from universe import PackageInfo` is rejected because the pre-materialized Universe root has no `PackageInfo` export.

The source/interface layer contains the intended declarations: `errors/unsupported.ph` exports `unsupported`, `reflection/package-info.ph` declares `PackageInfo`, and the module-interface tests confirm the root interface exposes native Universe bindings. The runtime path diverges: `phalcom-core/src/modules/builtin_materialize.rs` installs native bindings in their canonical owner modules and direct child packages, but does not project source-level export tables into the eagerly materialized builtin modules; `phalcom-core/src/modules/context.rs` then returns early for those already-registered modules. The failures are therefore a builtin runtime materialization/export-surface mismatch, unrelated to concurrency and unrelated to the `phalcom-modules` lint cleanup. No REPL expectation or source export was changed in this task.

## Negative/deletion gates

- `rg 'FiberStatus::Suspended' phalcom-core/src` → no matches.
- `rg 'resumer\.is_none\(\)' phalcom-core/src/primitive/fiber.rs` → no matches.
- `rg 'ready_queue\.(push_back|pop_front)' phalcom-core/src` → matches only VM-owned admission/dequeue helpers.
- `rg '_waiters.*Fiber|System\.schedule\(rawAwaiter\)' phalcom-core/core/universe/src/concurrency/fiber.ph` → no raw Fiber waiter path; await stores ticketed tuples.
- `completion_observer` is traced from `Object::Fiber` and is taken before observer scheduling; no Rust closure or Future-specific heap pointer is stored.
- `rg 'const res = fib\.try|const driver = Fiber\.new' phalcom-core/core/universe/src/concurrency/fiber.ph` → no one-turn Future driver remains.
- `rg 'nextScheduled' --glob '!target/**'` → only internal `_$nextScheduled` implementation/spec references and historical audit/context references remain; no public `System.nextScheduled` declaration or installed public surface record remains.
- The five requested `phalcom-modules` Clippy findings are gone; package Clippy/tests are green. Workspace Clippy now reaches 36 pre-existing `phalcom-semantic` findings, which remain outside this task.
- Workspace tests remain baseline-blocked by the three diagnosed builtin runtime import/export failures above; formatting, build, module tests/Clippy, and all concurrency-focused gates are green.
- C5 documentation closure: complete for the reviewed stale rustdoc, floor-census count/surface, ADR-0030 opcode wording, and E010 mechanism description.

## Deferred gates

- scheduler admission/resume → C1 complete
- ticketed Future parking/wake → C2 complete
- durable completion observers → C3 complete
- terminal Future adoption, settlement, and suspension-safe continuations → C4 complete
- ownership closure, specs, and permanent regressions → C5 complete
- format → PASS; module tests/Clippy → PASS; workspace test → baseline-blocked by diagnosed REPL failures; workspace Clippy → baseline-blocked by 36 `phalcom-semantic` findings; workspace build → PASS

## Unexpected findings

- C5 removed the public `System.nextScheduled` compatibility getter after all Future consumers migrated; only internal scheduler dequeue/resume and exact wake/park/observer seams remain.
- Internal observer selectors are reserved to the core/runtime implementation; end-to-end language fixtures exercise them through the Future consumer.

## Active incident

Final workspace certification remains blocked by the diagnosed REPL builtin export failures and 36 unrelated `phalcom-semantic` Clippy findings. The requested five `phalcom-modules` findings are fixed. E010 remains an intentional open concurrency-policy issue; it was not silently closed by C5.

## Next resume action

Implementation and reviewed documentation closure complete; retain the final-gate classifications above. Resolve the builtin runtime export projection and the remaining `phalcom-semantic` Clippy baseline separately if full workspace certification is required.
