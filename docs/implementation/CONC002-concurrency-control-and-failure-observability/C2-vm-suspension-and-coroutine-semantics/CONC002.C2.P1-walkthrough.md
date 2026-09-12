# CONC002.C2.P1-R1 Walkthrough — Native Suspension and VM Control Continuations

This document records the architectural changes, implementation details, defect resolutions, and verification evidence for **`CONC002.C2.P1-R1`** (*Native Suspension and VM Control Continuations*).

---

## 1. Executive Summary

- **Checkpoint**: `CONC002.C2` (*VM Suspension and Coroutine Semantics*)
- **Plan**: `CONC002.C2.P1-R1` (*Native Suspension and VM Control Continuations*)
- **Status**: **COMPLETE / IMPLEMENTED**
- **Commits**:
  - `108fe104` — `feat(concurrency): implement C2.P1-R1 VM control continuations and native suspension`
  - `4e1c2fc7` — `fix(concurrency): harden Call failure injection, OnMatch outcome, and shape dispatch`
- **Objective**: Replace native host re-entry frames (`block_call`, `run_until` recursion) with VM/Fiber-owned control continuations (`ControlStack`), making `Block.on`, `Block.ensure`, `Block.whileTrue`, `Bool` lazy operators, and `Option.match` suspension-transparent while enforcing strict restricted-yield guards at genuine native host boundaries.

---

## 2. Key Architecture & Implementation Changes

### 2.1 VM-Owned Control Stack (`phalcom-core/src/vm/control.rs`)
Created the core data structures for VM-managed control continuations:
- **`ControlId`**: Unique identifier for tracking control activation frames.
- **`ControlDestination`**: Distinguishes `StackOperand { target_index }`, `ControlActivation { id }`, and `FiberResult`.
- **`ControlPhase`**:
  - `OnBody`: Protected body evaluation for `Block.on`.
  - `OnMatch`: Dynamic `is(_)` exception class matching.
  - `OnHandler`: Handler execution on successful error match.
  - `EnsureBody`: Protected block evaluation for `Block.ensure`.
  - `EnsureCleanup`: Cleanup execution preserving original transfer outcomes.
  - `WhileCondition` / `WhileBody`: Looping condition and body evaluations for `whileTrue`.
  - `BoolBranch`: Branch evaluation for `Bool.ifTrue`, `Bool.ifFalse`, and `ifTrue:ifFalse:`.
  - `OptionBranch`: Arm evaluation for `Option.match`.
  - `OrderingReverse`: Validation and outcome inversion for reversed `Ordering`.
- **`Transfer`**: Models `Returned(Value)`, `Raise(PhError)`, and `NonLocalReturn { target, value }`.
- **`ControlStack`**: Stack of `ControlActivation` records owned by each Fiber, with full GC root tracing.

### 2.2 Fiber Ownership & GC Tracing
- **Fiber Integration** (`fiber.rs`): Added `control_stack: ControlStack` and `resume_destination: Option<ControlDestination>` to `FiberObject`.
- **Live/Parked Ownership** (`fiber.rs`): `store_live_into` and `load_live_from` swap `control_stack` alongside frames, operand stacks, and upvalues.
- **Root Tracing** (`gc.rs`, `trace.rs`): All active control records and pending transfers are fully traced as roots during garbage collection.

### 2.3 Migration to Shape ABI Primitives
- **`Block`** (`block.rs`): Migrated `whileTrue`, `on`, and `ensure` to shape ABI (`block_while_true_shape`, `block_on_shape`, `block_ensure_shape`).
- **`Bool`** (`boolean.rs`): Migrated `and`, `or`, `ifTrue`, `ifFalse`, `ifTrue(_:ifFalse:)` to shape ABI.
- **`Option`** (`option.rs`): Migrated `match` to `option_match_shape`.

### 2.4 Child Fiber Call-Mode Failure Injection & Object Identity
- When a child fiber finishes in `Failed` state under `ResumeMode::Call`:
  1. The parent fiber is restored without delivering a return operand via `switch_to_fiber_without_deliver`.
  2. `Transfer::Raise` is stepped through the parent's `control_stack` at the exact call destination.
  3. Reused the existing `error_value` when synthesizing `RuntimeError::Raise`, ensuring that `child.error` and parent `on(Error)` observe the *exact same object identity*.

---

## 3. Defect Resolutions & Hardening

1. **Call-Mode Child Abort Contract in Fixture**:
   - Fixed `concurrency_control_call_failure_parent_catch.ph` to use `Fiber.abort(Error.new("child exploded"))` so the error payload is an `Error` instance matching `action.on(Error)`.

2. **`ControlPhase::OnBody` CallOutcome Handling**:
   - Updated `control.rs` to inspect `CallOutcome` from `dispatch_selector_window_as`: immediately continues with `Transfer::Returned(v)` if synchronous, or returns `ControlStepOutcome::Continued` on frame entry.

3. **`Bool` Sacred Deoptimization & Keyword Extraction**:
   - Fixed `bool_if_true_if_false_shape` in `boolean.rs` to extract keyword arguments via `args.labeled_value(vm, 0).or_else(|| args.positional(vm, 1))` and check `physical_arity()`.
   - Used `branch_val.wrap_some()?` for `BoolBranchKind::IfTrueOption` / `IfFalseOption` in `control.rs`.

4. **Setter Function Invocation Shape**:
   - Updated `validate_captured_method_shape` in `send.rs` to allow setter methods invoked with 1 positional argument (`setter_as_positional`), resolving `family_exact_setter.ph`.

5. **Native Re-entrancy Depth Limits**:
   - Updated `execution_depth_limits.rs` to test retained synchronous native host frames (`Probe.hash` inside `Set.add`), acknowledging that `whileTrue` is no longer a native re-entry frame.

6. **Contract Invariant Suspension Tests**:
   - Relocated `contracts_invariant_fiber_yield.ph` to positive tests in `language/errors/`, confirming that `@invariant` guarded methods now yield and resume without triggering native frame restrictions.

---

## 4. Verification Evidence

All test gates were executed serially with clean compiler flags (`RUSTFLAGS='' RUSTC_WRAPPER=''`):

| Gate / Suite | Command | Result | Notes |
|---|---|---|---|
| **Code Formatting** | `cargo fmt --all -- --check` | **PASS** | Clean across workspace |
| **Workspace Lints** | `cargo clippy --workspace --all-targets -- -D warnings` | **PASS** | 0 warnings |
| **Core Unit Tests** | `cargo test -p phalcom-core --lib` | **PASS** | 112 passed, 0 failed |
| **Core Integration Tests** | `cargo test -p phalcom-core --test core` | **PASS** | 468 passed, 0 failed, 24 ignored |
| **Execution Depth Limits** | `cargo test -p phalcom-core --test core execution_depth_limits` | **PASS** | 5 passed, 0 failed |
| **Concurrency Positive Corpus** | `cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --exact` | **PASS** | All call, try, ensure, on, whileTrue suspension tests green |
| **Concurrency Negative Corpus** | `cargo test -p phalcom-core --test language-corpus corpus::concurrency_negative -- --exact` | **PASS** | 13/13 tests green, verifying native re-entry guards |
| **Family Corpus** | `cargo test -p phalcom-core --test language-corpus corpus::family -- --exact` | **PASS** | 2 passed (positive and negative) |
| **Runtime Errors Corpus** | `cargo test -p phalcom-core --test language-corpus corpus::runtime_errors -- --exact` | **PASS** | 1 passed, 0 failed |
| **Errors Corpus** | `cargo test -p phalcom-core --test language-corpus corpus::errors -- --exact` | **PASS** | 1 passed, 0 failed |
| **Full Language Corpus** | `cargo test -p phalcom-core --test language-corpus` | **60 PASS / 0 Unexpected Failures** | 1 pre-existing `streams` baseline failure |

---

## 5. Next Steps

- **Handoff Target**: `CONC002.C2.P2` (*Coroutine Consumer / Executor Separation and Fiber Protocol*).
- See `CONC002.C2.P2-handoff.md` for the detailed execution plan and prerequisites.
