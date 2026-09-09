# Concurrency Control Implementation State

## Repository state

- branch: `codex/concurrency-control-remediation`
- implementation commits: `74a247b8` (linked runtime exports), `44f9364e` (E010 observability), and `c1b9d907` (E010 documentation)
- implementation baseline through: `c8beaea0` (the exploratory broad REPL visibility change was reverted; this evidence amendment follows)
- current exact HEAD is represented by the pushed branch history; this record avoids duplicating a hash that becomes stale when documentation is amended

## Established invariants

- CONC-STATE-01: `Running` identifies the VM's current Fiber; a caller parked behind a child is `BlockedOnChild`.
- CONC-STATE-02: `BlockedOnChild` is not manually resumable.
- CONC-STATE-03: `New` and explicit `Yielded` are distinct lifecycle states.
- CONC-ROOT-01: root identity is stored on the Fiber and does not depend on `resumer`.

## Decisions

- D-01 through D-11 remain as ratified in [`CONC002.C1.P1-concurrency-control-remediation.md`](CONC002.C1.P1-concurrency-control-remediation.md).
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
| C5 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core object_model_invariants::floor_census_matches_installed_bindings` | PASS, 1 test | native floor census matches the 229 installed public/internal bindings |
| C5 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core` | PASS, 456 passed, 29 ignored | complete core integration target, including scheduler/fiber object-model invariants |
| C5 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus` | PASS, 61 passed, 4 ignored | complete language corpus, including concurrency and negative fixtures |
| Final | `cargo fmt --all -- --check` | PASS after formatting the three existing drift files | workspace formatting hygiene |
| Final | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets` | PASS, existing warnings only | all workspace targets compile with the patch |
| Final | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --all-targets` | PASS, all module targets | module-session/source changes remain behaviorally green |
| Final | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy -p phalcom-modules --all-targets -- -D warnings` | PASS | all five requested `phalcom-modules` findings are fixed |
| Final | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-repl --test repl_import_bugs -- --nocapture` | PASS, 6 tests | focused Patch A REPL import/export regressions are green |
| Final | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets` | BASELINE-BLOCKED: 12 failures in the broader `phalcom-repl/tests/repl_imports.rs`; all core, language-corpus, modules, and focused REPL targets are green | workspace-wide behavioral baseline; failures are unrelated to the concurrency targets and focused Patch A gate |
| Final | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings` | PASS after resolving the semantic, core, and LSP findings; no warnings remain under `-D warnings` | workspace lint-clean delivery |

## ASTRA audit

- E007 is fixed: terminal observers settle `Future.async` only after terminal completion; the multi-await regression covers pending turns and the final value.
- E008 is fixed: VM-owned admission rejects duplicate scheduling; the regression covers one execution and continued healthy work.
- E010 is implemented: unowned scheduler-mode terminal failures are retained in a VM-owned unhandled-failure queue, rooted through GC, reported by the root pump and `System.runScheduled`, and exposed to root `Future.await` quiescence diagnostics.
- The focused E010 regressions cover GC retention and failure-observer reporting; the floor census and documentation were updated for the three internal reporting selectors.
- Audit limitation: ASTRA reviewed the implementation state but did not independently rerun the final gates; the verification ledger above is the fresh implementation-run record.

## REPL baseline diagnosis

The focused Patch A target is green: 6/6 tests pass, including direct-path package metadata imports, selective imports, missing-export failures, module property access, and Universe-root package exports.

The broader integration target remains baseline-blocked: 17/29 pass and 12 fail:

- `repl_bs_01_property_access_on_module_is_non_none` and `repl_ec_02_imported_module_property_access`: `selector.Selector` reaches module `doesNotUnderstand`.
- `repl_bs_02_builtin_classes_are_executable_class_objects`, `repl_cp_02_selectively_imported_class_usable_in_later_cell`, `repl_cp_03_import_declaration_and_use_persist`, `repl_ec_04_multiple_distinct_imports_do_not_collide`, `repl_se_04_failed_import_does_not_poison_session`, `repl_si_01_selective_import_selector_class_is_non_none`, `repl_si_03_multiple_selective_imports_bind`, `repl_si_04_selective_import_persists_across_cells`, and `repl_si_05_selective_import_is_immutable`: selective import resolution reports that `universe:reflection.selector` does not export `Selector`.
- `repl_si_02_selective_import_alias_rebinds`: the analogous `universe:reflection.message` module does not export `Message`.

The source/interface layer deliberately does not implicitly export non-root source declarations: `phalcom-modules/tests/builtin_catalog.rs::bcat_09_non_root_source_declarations_are_not_implicitly_exported` is green. The failed broad cases instead expect native-owner classes such as `Selector` and `Message` to appear as exports of their non-root canonical modules. The attempted broad projection was reverted because it violated that authoritative module-layer rule. This remains a separate builtin runtime/interface contract issue; no broad REPL expectation or source export was changed in this task.

## Negative/deletion gates

- `rg 'FiberStatus::Suspended' phalcom-core/src` → no matches.
- `rg 'resumer\.is_none\(\)' phalcom-core/src/primitive/fiber.rs` → no matches.
- `rg 'ready_queue\.(push_back|pop_front)' phalcom-core/src` → matches only VM-owned admission/dequeue helpers.
- `rg '_waiters.*Fiber|System\.schedule\(rawAwaiter\)' phalcom-core/core/universe/src/concurrency/fiber.ph` → no raw Fiber waiter path; await stores ticketed tuples.
- `completion_observer` is traced from `Object::Fiber` and is taken before observer scheduling; no Rust closure or Future-specific heap pointer is stored.
- `rg 'const res = fib\.try|const driver = Fiber\.new' phalcom-core/core/universe/src/concurrency/fiber.ph` → no one-turn Future driver remains.
- `rg 'nextScheduled' --glob '!target/**'` → only internal `_$nextScheduled` implementation/spec references and historical audit/context references remain; no public `System.nextScheduled` declaration or installed public surface record remains.
- Workspace Clippy is green across all packages and targets after resolving the semantic, core, and LSP diagnostics; package tests for the affected semantic, core, and LSP crates are also green.
- Workspace tests remain baseline-blocked by the twelve diagnosed broad REPL builtin import/export failures above; formatting, build, workspace Clippy, affected-package tests, the focused Patch A gate, and all concurrency-focused gates are green.
- C5 documentation closure: complete for the reviewed stale rustdoc, floor-census count/surface, ADR-0030 opcode wording, and E010 mechanism description.

## Deferred gates

- scheduler admission/resume → C1 complete
- ticketed Future parking/wake → C2 complete
- durable completion observers → C3 complete
- terminal Future adoption, settlement, and suspension-safe continuations → C4 complete
- ownership closure, specs, and permanent regressions → C5 complete
- format → PASS; affected-package tests/Clippy → PASS; focused Patch A tests → PASS; workspace test → baseline-blocked by 12 diagnosed broad REPL failures; workspace Clippy → PASS; workspace build → PASS

## Unexpected findings

- C5 removed the public `System.nextScheduled` compatibility getter after all Future consumers migrated; only internal scheduler dequeue/resume and exact wake/park/observer seams remain.
- Internal observer selectors are reserved to the core/runtime implementation; end-to-end language fixtures exercise them through the Future consumer.

## Active incident

Final workspace certification remains blocked only by the diagnosed broad REPL builtin export failures. Workspace Clippy and the requested five `phalcom-modules` findings are fixed. E007, E008, and E010 are implemented and covered by focused regressions; any remaining E010 work is general policy/reporting expansion beyond this plan.

## Next resume action

Implementation and reviewed documentation closure complete; retain the final-gate classifications above. Resolve the separate native-owner export contract if full workspace test certification is required.
