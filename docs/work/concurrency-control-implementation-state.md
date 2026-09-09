# Concurrency Control Implementation State

## Repository state

- branch: `codex/concurrency-control-remediation`
- HEAD: `268da129` plus the C1 working changes below
- relevant local changes preserved: baseline audit, plan, and unrelated files are in the pushed parent commits

## Established invariants

- CONC-STATE-01: `Running` identifies the VM's current Fiber; a caller parked behind a child is `BlockedOnChild`.
- CONC-STATE-02: `BlockedOnChild` is not manually resumable.
- CONC-STATE-03: `New` and explicit `Yielded` are distinct lifecycle states.
- CONC-ROOT-01: root identity is stored on the Fiber and does not depend on `resumer`.

## Decisions

- D-01 through D-11 remain as ratified in `docs/implementation/INBOX/phalcom-concurrency-control-patch-grade-implementation-plan.md`.
- C0 uses `FiberStatus::{New, Running, BlockedOnChild, Yielded, Parked, Queued, Done, Failed}`; `Parked` and `Queued` are reserved for later checkpoints.
- Manual `call`/`try` accepts only `New` and `Yielded` in C0.
- Scheduler admission is VM-owned: `New`/`Yielded` transition atomically to `Queued`; duplicate, parked, active, blocked, and terminal fibers are rejected.
- Scheduler dequeue is FIFO and skips stale non-`Queued` entries without queue scanning or duplicate admission checks.
- Scheduler resume has an explicit internal mode and is used by both the root-drive pump and `System.runScheduled`; public `Fiber#try` remains a manual path.

## Evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|
| Baseline | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo build -p phalcom-core` | PASS | clean starting runtime build |
| Baseline | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture` | PASS, 2 tests | pre-change concurrency corpus |
| C0 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core` | PASS | exhaustive lifecycle caller migration compiles |
| C0 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture` | PASS, 2 tests | manual coroutine compatibility, stable root identity, and blocked-parent rejection |
| C1 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core` | PASS | scheduler admission/dequeue and explicit resume-mode fanout compile |
| C1 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture` | PASS, 2 tests | FIFO, nested scheduling, scheduled-failure isolation, duplicate admission, queued manual-resume rejection, and existing concurrency regressions |

## Negative/deletion gates

- `rg 'FiberStatus::Suspended' phalcom-core/src` → no matches.
- `rg 'resumer\.is_none\(\)' phalcom-core/src/primitive/fiber.rs` → no matches.
- `rg 'ready_queue\.(push_back|pop_front)' phalcom-core/src` → matches only VM-owned admission/dequeue helpers.

## Deferred gates

- scheduler admission/resume → C1 complete
- ticketed Future parking/wake → C2
- durable completion observers → C3
- terminal Future settlement and continuations → C4
- ownership closure, specs, and broad gates → C5/final gate

## Unexpected findings

- `System.nextScheduled` remains a compatibility getter that releases its queue reservation; production scheduler pumps use the internal dequeue and scheduler-resume seams. Public raw-authority removal is deferred to C5 after Future consumers migrate.

## Active incident

None.

## Next resume action

Begin C2, Task 9 — add ticketed Fiber parking and the internal Future wake seam.
