# CONC002.C3.P2 Walkthrough — Poller and External Readiness

This document records the architectural changes, implementation details, invariant proofs, and verification evidence for **`CONC002.C3.P2`** (*Poller and External Readiness*).

---

## 1. Executive Summary

- **Checkpoint**: `CONC002.C3` (*Reactor and External Completion Runtime*)
- **Plan**: `CONC002.C3.P2` (*Poller and External Readiness*)
- **Status**: **COMPLETE / IMPLEMENTED**
- **Objective**: Complete the reactor for OS descriptor readiness without introducing a second executor or exposing poller internals to guest code. Ratify PDR-0016, pin `mio` confined strictly to `phalcom-core/src/reactor/`, integrate cross-thread `mio::Waker` into `WorkerPool`, replace phase-1 channel timeout wait with unified descriptor/timer polling wait `poll(timeout = min(timer, cap))`, and enforce try-first / register-second edge-triggered readiness discipline.

---

## 2. Key Architecture & Implementation Changes

### 2.1 PDR-0016 Ratification & Confinement
- **Decision Ratified**: Updated `PDR-0016` status to **Accepted (ratified 2026-09-12)** in [`docs/pdr/0016-poller-backend-is-mio.md`](../../../../docs/pdr/0016-poller-backend-is-mio.md) and [`docs/pdr/STATUS.md`](../../../../docs/pdr/STATUS.md).
- **Workspace Dependency**: Added `mio = { version = "1.0", features = ["os-poll", "net"] }` to `[workspace.dependencies]` in root [`Cargo.toml`](../../../../Cargo.toml) and `mio = { workspace = true }` in [`phalcom-core/Cargo.toml`](../../../../phalcom-core/Cargo.toml).
- **Law of Confinement**: No `mio` type, import, or signature appears outside `phalcom-core/src/reactor/`. Validated at test time by `test_confinement_no_mio_outside_reactor`.

### 2.2 Poller Module (`phalcom-core/src/reactor/poller.rs`)
Implemented the single-VM-thread OS readiness poller:
- **`Poller`**: Wraps `mio::Poll`, `mio::Events` (capacity 128), and `Arc<mio::Waker>`.
- **`PollInterest`**: Strongly typed interest enum (`Readable`, `Writable`, `Both`) mapped to `mio::Interest`.
- **`PollReadiness`**: Struct holding boolean flags (`is_readable`, `is_writable`, `is_read_closed`, `is_write_closed`, `is_error`) and `.bits() -> u32` bitmask conversion.
- **Sentinel Token**: `WAKER_TOKEN = mio::Token(usize::MAX)` reserved for cross-thread wakers.
- **Registration APIs**: `register`, `reregister`, `deregister` for `mio::event::Source` and raw Unix file descriptors (`register_raw_fd`, `reregister_raw_fd`, `deregister_raw_fd`).

### 2.3 Cross-Thread Wake (`phalcom-core/src/reactor/worker.rs`)
- Updated `WorkerPool::new` to accept `Option<Arc<mio::Waker>>`.
- Background worker threads call `waker.wake()` immediately after sending `WorkerCompletion` through the MPSC channel.
- Unblocks the VM thread parked in `poll` instantaneously without waiting for timer deadlines or executor timeouts.

### 2.4 Generational Token Registry Integration (`phalcom-core/src/reactor/registry.rs`)
- Added `ReactorSource::Pollable` to `ReactorSource`.
- Added helper queries `generation(slot: u32) -> Option<u32>` and `get(slot: u32) -> Option<&ReactorRegistration>`.
- `ReactorToken(slot, generation)` continues to govern validation and GC root tracking for descriptor registrations.
- Stale or deregistered poller readiness events are discarded as harmless no-ops.

### 2.5 Unified Wait & Ingress Drain (`phalcom-core/src/reactor/mod.rs`)
- Replaced phase-1 channel timeout with unified wait in `Reactor::idle_wait`:
  ```rust
  let timeout = match self.timers.peek_deadline() {
      Some(deadline) => {
          let remaining = deadline - now;
          Some(remaining.min(executor_cap))
      }
      None => Some(executor_cap),
  };
  self.poller.poll(timeout);
  ```
- Drains:
  1. Expired monotonic timers.
  2. Descriptor readiness events validated against current generational registry records.
  3. Buffered and channel-incoming background worker completions.
- `poll_ingress_safepoint`: Non-blockingly polls both `poller.poll(Some(Duration::ZERO))` and channel MPSC into internal buffers.

### 2.6 Try-First Edge-Triggered Discipline
- Introduced `PollOpResult<T>` (`Ready(T)`, `WouldBlock`, `Err(String)`).
- Enforced try-first rule: syscalls attempt non-blocking execution at submission and on readiness before registering/re-arming interest, preventing edge-triggered hangs on pre-buffered kernel data.

---

## 3. Verification Evidence

All test suites and workspace verification gates executed serially with clean compiler flags (`RUSTFLAGS='' RUSTC_WRAPPER=''`):

| Gate / Suite | Command | Result | Notes |
|---|---|---|---|
| **Reactor Lib Tests** | `cargo test -p phalcom-core --lib reactor` | **PASS** | 14 passed in 0.03s (unified wait, waker, stale discard, try-first, rearm, GC roots, confinement) |
| **Full Core Lib** | `cargo test -p phalcom-core --lib` | **PASS** | 126 passed, 0 failed |
| **Core Suite** | `cargo test -p phalcom-core --test core` | **PASS** | 468 passed, 0 failed, 24 ignored |
| **Floor & Surface Census** | `cargo test -p phalcom-core --test core floor_census_matches_installed_bindings`, `canonical_surface_census_is_unique_and_actionable` | **PASS** | Floor bindings verified at 234, surface records generated at 333 |
| **Concurrency Language Corpus** | `cargo test -p phalcom-core --test language-corpus corpus::concurrency` | **PASS** | 88 passed, 0 failed |
| **Negative Language Corpus** | `cargo test -p phalcom-core --test language-corpus corpus::concurrency_negative` | **PASS** | 14 passed, 0 failed |
| **All Language Corpus** | `cargo test -p phalcom-core --test language-corpus` | **PASS** | 61 passed, 0 failed |
| **CLI Smoke Tests** | `cargo test -p phalcom-core --test cli-smoke` | **PASS** | 3 passed, 0 failed |
| **Code Formatting** | `cargo fmt --all -- --check` | **PASS** | Clean across workspace |
| **Workspace Clippy** | `cargo clippy --workspace --all-targets -- -D warnings` | **PASS** | 0 warnings |

---

## 4. Invariant Certifications

1. **Law of Confinement (PDR-0016)**: Certified that zero `mio` types or symbols appear outside `phalcom-core/src/reactor/`.
2. **Single VM Thread**: Certified that readiness observation and completion settlement occur exclusively on the main VM thread.
3. **Generational Safety**: Certified that stale descriptor readiness cannot settle recycled registration slots or unroot unrelated objects.
4. **Unified Wake**: Certified that timers, worker jobs, and descriptor readiness share a single blocking wait.
5. **Try-First Correctness**: Certified that pre-buffered socket/pipe data does not deadlock under edge-triggered readiness.
