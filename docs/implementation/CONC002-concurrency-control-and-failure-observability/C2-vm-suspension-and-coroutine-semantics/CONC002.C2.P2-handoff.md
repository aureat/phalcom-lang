# CONC002.C2.P2 Implementation Handoff & Execution Guide

## 1. Context and Current State

- **Current Revision**: `main` at `4e1c2fc7` (clean working tree).
- **Prerequisites Completed**:
  - `CONC002.C1.P1` through `CONC002.C1.P4` complete.
  - `CONC002.C2.P1-R1` (**Native Suspension and VM Control Continuations**) complete and verified across all test gates.
- **Active Plan**: `CONC002.C2.P2-coroutine-consumer-executor-separation-and-fiber-protocol.md`.

---

## 2. P1-R1 Accomplishments Inherited by P2

1. **VM-Owned Control Continuations**:
   - `ControlStack` and `ControlActivation` in `phalcom-core/src/vm/control.rs` are owned by `VM` (live) and `FiberObject` (parked), fully GC-traced.
   - `Block.on`, `Block.ensure`, `Block.whileTrue`, `Bool` lazy operators, and `Option.match` are converted to shape ABI and are suspension-transparent.
2. **Unified Transfers**:
   - `step_control_transfer` routes `Transfer::Returned`, `Transfer::Raise`, and `Transfer::NonLocalReturn` iteratively.
3. **Child Failure Injection**:
   - Call-mode child fiber failure injects `Transfer::Raise` at the exact parent call site with preserved error object identity.
4. **Retained Native Boundaries**:
   - Genuine native host boundaries (`Map`/`Set` hash/equals probing, reflective typing, string formatting) enforce `check_native_reentry` guards.

---

## 3. P2 Objectives and Core Mission

P2 resolves the remaining execution-ownership and coroutine protocol separation:
> **A Fiber must be able to retain its manual coroutine consumer while an executor independently parks, wakes, and resumes it.**

### Key Architectural Invariants to Realize in P2:
1. **Coroutine Consumer vs Executor Separation**:
   - Separate the dynamic caller/resumer (`coroutine_consumer`) from scheduler-owned execution driving (`executor`).
   - Support `yield -> pending await -> resume by executor -> yield` to the original manual consumer.
2. **Preserved Manual Turn Across Park**:
   - When a manually consumed child parks on a pending `Future`, the parent turn does not abort or treat parking as a user return value; instead, the turn is suspended awaiting the child's executor completion.
3. **No Stale Queue / Adoption Violations**:
   - Scheduler admission must not adopt an arbitrary yielded coroutine continuation with an invented operand.
   - User `Fiber.yield` without a coroutine consumer remains strictly rejected (`concurrency_scheduled_yield_has_no_consumer.ph`).
4. **Fiber Type and Surface Protocol**:
   - Resolve `Fiber<R>` surface typing and the `Fiber.abort` arbitrary-value vs `Error` declaration contract.

---

## 4. Immediate Starting Steps for P2

1. **Verify Baseline State**:
   ```sh
   git status --short
   RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --exact
   RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency_negative -- --exact
   ```
2. **Review Plan**:
   Read `CONC002.C2.P2-coroutine-consumer-executor-separation-and-fiber-protocol.md` Sections 2–6.
3. **Task Breakdown**:
   - Phase 1: Fiber object model refactoring (`coroutine_consumer` vs `executor_wait`).
   - Phase 2: VM dispatch & scheduler pump integration for coroutine/executor interleave.
   - Phase 3: Language corpus test suite for `yield-await-yield` and manual multi-turn coroutine async execution.
   - Phase 4: Full workspace verification and baseline gate closure.
