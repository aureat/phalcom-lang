---
id: CONC002.C2
category: CONC
program: CONC002
checkpoint: CONC002.C2
kind: checkpoint-record
status: IN_PROGRESS
completion: IN_PROGRESS
verification: IN_PROGRESS
---

# CONC002.C2 — VM suspension, control continuations, and coroutine semantics

C2 makes language control continuations representable independently of live Rust host frames, then completes the execution-ownership model so a manually consumed Fiber can retain its coroutine consumer while executor-driven readiness parks and resumes it.

---

## 1. Plans Ledger

| Plan | Scope | Status | Verification | Summary / Outcome |
|---|---|---|---|---|
| `CONC002.C2.P1-R1` | VM-owned native control continuations, explicit activation outcomes/transfers, protected execution/cleanup, host escape, Call child-failure injection, suspension-transparent native control fallbacks, GC/trace/bootstrap integration and readiness handoff | **IN_PROGRESS** | `IN_PROGRESS` | Active implementation plan for VM-owned control continuations and suspension transparency. |
| `CONC002.C2.P2` | Coroutine-consumer/executor separation, manual pending-await and yield-await-yield composition, final `Fiber<R>` surface if C1.P4 type prerequisites allow it | **PLANNED** | `UNVERIFIED` | Next deliverable in C2. |

---

## 2. Ownership Boundary

C2 owns:
- VM-owned native control continuations (`on`, `ensure`, `whileTrue`, `Bool`/`Option`/`Ordering` fallbacks);
- VM-resident protected/cleanup control records and saved transfer routing;
- Call child-failure injection as Raise into the parent call site;
- Manual coroutine + executor composition (`yield → await → yield`);
- Executor readiness handoff mechanism (`Parked(gen)` -> `Queued` admission without executing user code).

C2 does **not** own:
- External reactor registrations, pollers, timers, sockets/files/process readiness backends (owned by C3);
- Cross-thread injection or idle-with-registrations executor policy (owned by C3);
- Structured concurrency, Task/TaskGroup, channels or select (owned by C4).

---

## 3. Implementation State and Invariants

### 3.1 Core Architecture & Representation
- Control continuations are owned by the `Fiber` / `VM` execution context rather than living as live Rust stack frames during arbitrary user code execution.
- Activation outcomes distinguish `Returned(Value)`, `EnteredFrame`, `EnteredControl`, `SwitchedFiber`.
- Transfer routing handles normal return, Raise, and non-local return with crossed cleanup execution inside-out before frame destruction.
- Host boundaries distinguish leaf synchronous primitives from retained host algorithms with explicit safety guards.

### 3.2 Gate Progression (CONC002.C2.P1-R1)

| Gate | Scope | Status | Notes |
|---|---|---|---|
| **C0** | Baseline, scope & inventory | **IN_PROGRESS** | Classify callback callers, baseline tests & reproducers |
| **C1** | Control ownership & activation outcomes | **PENDING** | ControlStack, ControlActivation, CallOutcome |
| **C2** | Unified transfers & safe host escape | **PENDING** | Transfer routing, non-local unwind cursor, host escape |
| **C3** | Suspendable protection & cleanup | **PENDING** | `on` / `ensure` / `Error.raise` VM-owned phases |
| **C4** | Child failure re-enters parent continuation | **PENDING** | Call child failure -> parent Raise injection |
| **C5** | Suspension-transparent native control fallbacks | **PENDING** | `Bool`, `Option`, `whileTrue`, `Ordering` |
| **C6** | Lifetime & bootstrap integration | **PENDING** | GC tracing, upvalues, logical traces |
| **C7** | Executor readiness handoff | **PENDING** | Exact wait generation validation, once-only queue admission |
| **C8** | Migration closure & delivery | **PENDING** | Deprecation cleanup, perf verification, full gates |

---

## 4. Verification Ledger

| Gate | Evidence / Command | Result | Detail |
|---|---|---|---|
| Baseline | `cargo test -p phalcom-core --test language-corpus corpus::concurrency` | PASS | Baseline from C1 |
| Baseline | `cargo test -p phalcom-core --lib` | PASS | Baseline from C1 |
