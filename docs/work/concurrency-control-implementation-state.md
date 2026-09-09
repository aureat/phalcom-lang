# Concurrency Control Implementation State

## Repository state

- branch: `codex/concurrency-control-remediation`
- HEAD: `6b5ca843` plus the C0 working changes below
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

## Evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|
| Baseline | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo build -p phalcom-core` | PASS | clean starting runtime build |
| Baseline | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture` | PASS, 2 tests | pre-change concurrency corpus |
| C0 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core` | PASS | exhaustive lifecycle caller migration compiles |
| C0 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture` | PASS, 2 tests | manual coroutine compatibility, stable root identity, and blocked-parent rejection |

## Negative/deletion gates

- `rg 'FiberStatus::Suspended' phalcom-core/src` → no matches.
- `rg 'resumer\.is_none\(\)' phalcom-core/src/primitive/fiber.rs` → no matches.

## Deferred gates

- scheduler admission/resume → C1
- ticketed Future parking/wake → C2
- durable completion observers → C3
- terminal Future settlement and continuations → C4
- ownership closure, specs, and broad gates → C5/final gate

## Unexpected findings

- None.

## Active incident

None.

## Next resume action

Begin C1, Task 5 — centralize ready-queue state transitions in VM-owned helpers.
