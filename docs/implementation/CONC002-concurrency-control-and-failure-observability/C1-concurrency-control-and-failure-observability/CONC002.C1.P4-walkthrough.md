# CONC002.C1.P4 Walkthrough — Pre-C2 Stabilization and Fiber Typing Prerequisites

This document records the implementation, verification evidence, and checkpoint closure for **CONC002.C1.P4**.

---

## 1. Executive Summary

- **Checkpoint**: `CONC002.C1` (Corrected Concurrency Foundation)
- **Plan**: `CONC002.C1.P4` (Pre-C2 Stabilization and Fiber Typing Prerequisites)
- **Baseline Revision**: `main` at `2db7e3780772e847a178918ede239d91e0bcc5fd`
- **Result Status**: **COMPLETE / IMPLEMENTED / VERIFIED**
- **Outcome**: **Outcome B Ratified** (Explicit Blocker on Type System).
- **Handoff Target**: `CONC002.C2.P1-R1` (Native Suspension and VM Control Continuations) and `CONC002.C2.P2` (Coroutine Consumer/Executor Separation and Fiber Protocol).

---

## 2. Key Accomplishments & Technical Decisions

### 2.1 Checkpoint C1 — Manual Fiber Protocol Pinned
- **Adversarial Hostile Fixture**: Created `phalcom-core/tests/fixtures/language/concurrency/concurrency_fiber_protocol_hostile.ph` and verified against `.expected`.
- **Protocol Findings**:
  1. **Entry vs Resume Input Independence**: Initial entry arguments passed to `f.call(arg)` participate in entry `Function` parameter binding, whereas subsequent `f.call(arg)` values are delivered as the evaluation result of suspended `Fiber.yield(...)`. These are distinct protocol positions with independent types.
  2. **Heterogeneous Multi-Yield Sites**: A single fiber can yield and receive distinct types across sequential yield points (e.g., yielding `1`, receiving `"one"`, then yielding `"two"`, receiving `true`, and completing with `42`).
  3. **Data vs Terminal Failure Distinction**: Yielding or returning an `Error` object as normal data produces a completed or suspended fiber with `error.isNone`, whereas raising or aborting transitions to `FiberStatus::Failed` with `error.isSome`.
- **Formally Rejected**: `Fiber<I, R>` and homogeneous `Fiber<Y, S, R>`.

### 2.2 Checkpoint C2 — Fiber Generic Feasibility & Outcome B
- **Nominal Target**: Ratified terminal-only `Fiber<R>` as the sole lifetime-stable type target.
- **Type System Evaluation**:
  - `phalcom-semantic`'s type system and AST annotations currently lack higher-rank parameter-pack abstraction (`(***P) -> R`). The solver in `inference.rs` strictly requires exact parameter count matching.
  - Wildcard / existential types (`exists R. Fiber<R>`) do not exist for `Fiber.current` and scheduler queues without fabricating unsound covariance (`Fiber<T> <: Fiber<Dynamic>`).
- **Outcome B Ratification**: Public `Fiber` remains non-generic in `fiber.ph` through C2.P1-R1 and C2.P2. Parameter-pack polymorphism is tracked as a dedicated type-system item.
- **Proper-Type / Kind Safety**: Added tests in `phalcom-semantic/tests/semantic/capabilities/generics.rs` (`unsaturated_generic_constructors_are_cleanly_rejected_in_proper_type_positions` and `generic_callable_return_inference_requires_exact_parameter_domain`). Verified that unsaturated generic constructors in proper-type positions emit clean `KindExpectedType` and `AnnotationUnsaturatedConstructor` diagnostics without panicking.

### 2.3 Checkpoint C3 — Pre-C2 Native-Boundary Handoff
- Audited all production native re-entry calls (`block_call`, `send_dynamic`, `activate_function`, `invoke_method_object`):
  - **VM-Visible Activations**: `activate_function`, `block_call` (suspension-safe).
  - **C2.P1-R1 Migration Targets**: `block_on` (`Block#on(_)`, error capture), `block_ensure` (`Block#ensure(_)`, cleanup hook), `Option#match`, `Bool` lazy callbacks, `Error#raise -> message`.
  - **Retained Guarded Host Algorithms**: `Map`/`Set` table probing/hashing, `render`/`toString`, reflective typing/descriptors.
- Verified that `CannotYieldAcrossNativeFrame` guards and ticketed park/wake remain non-mutating on refusal.

### 2.4 Checkpoint C4 — Contract Freeze and Documentation
- Frozen public `Fiber` surface in `fiber.ph` as non-generic.
- Documented that non-reference `Expr::TypeForm` lowering to `Bytecode::Nil` in `compiler/lib/expr.rs` is an existing compiler gap assigned to the compiler/type backlog.
- Updated all authoritative spec, contract, and state files.

---

## 3. Authoritative State & Census Artifacts

### 3.1 Census of Bare `Fiber` Usages

| Location | Form | Category | Final Treatment |
|---|---|---|---|
| `fiber.ph:8` | `System.schedule(_ fiber: Object) -> Fiber` | Erased runtime handle | Retain non-generic / existential handle |
| `fiber.ph:10` | `System._$nextScheduled -> Option<Fiber>` | Internal scheduler handle | Retain erased handle |
| `fiber.ph:18` | `System._$wake(_ fiber: Fiber, _ generation: Int) -> Bool` | Internal ticketed wake | Retain erased handle |
| `fiber.ph:78` | `class Fiber is Object` | Canonical class declaration | Target `Fiber<R>` in C2.P2 when type system allows |
| `fiber.ph:79` | `Fiber.new(_ body: Function) -> Fiber` | Constructor | `new<P, R>(_ body: (***P) -> R) -> Fiber<R>` in C2.P2 |
| `fiber.ph:95` | `_$onComplete(_ observer: () -> Unit) -> Fiber` | Internal terminal observer | Retain erased handle |
| `fiber.ph:103` | `Fiber.current -> Fiber` | Existential runtime handle | Requires wildcard / existential typing |
| `fiber.ph:270` | `Future.runToTerminal<U>(...) -> Fiber` | Internal execution driver | `Fiber<U>` candidate |
| `fiber.rs` | `Object::Fiber(FiberObject)` | Runtime heap object | Runtime identity distinct from type application |
| `vm/` | `VM.current`, ready queue | Runtime scheduling queue | Heterogeneous runtime handles |

### 3.2 Native Re-entry Handoff Table for C2.P1-R1

| Path / Caller | Rust State Across Execution | Classification | C2 Disposition |
|---|---|---|---|
| `activate_function` / `block_call` | Stack frames / view | VM-visible activation | Suspension-safe language execution |
| `block_on` (`Block#on(_)`) | Native frame / error catch | Host-held language continuation | C2.P1-R1 migration target to VM control records |
| `block_ensure` (`Block#ensure(_)`) | Native frame / cleanup hook | Host-held language continuation | C2.P1-R1 migration target to VM control records |
| `Option#match` / `Bool#ifTrue/ifFalse` | Native dispatch callback | Control-flow block call | C2.P1-R1 migration target |
| `Error#raise` -> `message` send | Native error formatting | Host dynamic send | C2.P1-R1 migration target |
| `Map`/`Set` hashing / equality | Hash table probing loop | Synchronous host algorithm | Retained guarded host algorithm |
| `render` / `toString` | Recursive formatting buffer | Synchronous host algorithm | Retained guarded host algorithm |
| Reflective typing / descriptor | Type construction state | Synchronous host algorithm | Retained guarded host algorithm |

---

## 4. Verification Evidence Ledger

All verification commands executed serially with cleared compiler flags (`RUSTFLAGS='' RUSTC_WRAPPER=''`):

| Suite | Command | Result | Duration / Details |
|---|---|---|---|
| Language Corpus Concurrency | `cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --exact` | **PASS** | 1 passed, 0 failed (41.57s) — verified `concurrency_fiber_protocol_hostile.ph` |
| Concurrency Negative | `cargo test -p phalcom-core --test language-corpus corpus::concurrency_negative -- --exact` | **PASS** | 1 passed, 0 failed (18.00s) |
| Semantic Capabilities | `cargo test -p phalcom-semantic --test semantic capabilities::generics` | **PASS** | 19 passed, 0 failed — verified kind/proper-type rejection and callable inference |
| Full Semantic Suite | `cargo test -p phalcom-semantic` | **PASS** | 1,146 passed, 0 failed, 42 ignored (175.48s) |
| Core Unit Tests | `cargo test -p phalcom-core --lib` | **PASS** | 110 passed, 0 failed (13.71s) |
| Format Check | `cargo fmt --all -- --check` | **PASS** | Clean |
| Workspace Build | `cargo build --workspace --all-targets` | **PASS** | Clean build |
| Workspace Lint | `cargo clippy --workspace --all-targets -- -D warnings` | **PASS** | Clean across all crates |

---

## 5. Checkpoint C1 Closure and Next Steps

- Checkpoint **CONC002.C1 is COMPLETE**.
- All prerequisite type-system, runtime-identity, and native-reentry classifications are explicit.
- Next resume action: **`CONC002.C2.P1-R1` (Native Suspension and VM Control Continuations)**.
