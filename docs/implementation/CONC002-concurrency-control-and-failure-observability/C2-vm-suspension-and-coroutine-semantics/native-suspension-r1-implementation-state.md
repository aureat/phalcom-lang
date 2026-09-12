# CONC002.C2.P1-R1 Implementation State

- **Plan**: `CONC002.C2.P1-R1` (Native suspension and VM control continuations)
- **Baseline Revision**: `ecc080664348ea1b3272f3da78350fed0afdefad` (clean `main`)
- **Status**: IN_PROGRESS
- **Active Incident**: None
- **Next Task**: Gate C1 (Control ownership & activation outcomes)

---

## 1. Established Invariants & Architectural Rules

1. **VM-Owned Control Continuations**: Language-level control continuations (`on`, `ensure`, `whileTrue`, `Bool`/`Option`/`Ordering` fallbacks) are represented in VM/Fiber-owned data structures (`ControlStack`), not on the live Rust stack.
2. **Protected Rust Frames**: The native-depth guard (`native_reentry_depth`) remains strictly active for genuinely synchronous host algorithms (`Map`/`Set` hash/equality, string rendering, reflective typing).
3. **Explicit Activation Outcomes**: Function/method calls and control activations return an explicit `CallOutcome` (`Returned(Value)`, `EnteredFrame`, `EnteredControl`, `SwitchedFiber`), avoiding frame-count heuristics.
4. **Unified Transfer Protocol**: Routing for `Returned`, `Raise`, and `NonLocalReturn` processes crossed cleanup handlers inside-out before frame destruction.
5. **Parent Call-Site Exception Injection**: In Call mode, a child fiber failure does not eagerly fail ancestors; it injects a `Raise` at the exact parent call site.
6. **Executor Readiness Handoff**: Readiness notifications validate `Parked(generation)` and transition to `Queued` once without executing user guest code.

---

## 2. Migration & Residual Host Inventory

| Caller / Location | Classification | Live Rust State / Hazard | C2.P1-R1 Treatment |
|---|---|---|---|
| `primitive/block.rs:block_on` | VM Control Target | `on` matching and handler block invocation | Migrate to `ControlPhase::OnBody`, `ControlPhase::OnMatch`, `ControlPhase::OnHandler` |
| `primitive/block.rs:block_ensure` | VM Control Target | `ensure` cleanup block invocation | Migrate to `ControlPhase::EnsureBody`, `ControlPhase::EnsureCleanup` with saved transfer |
| `primitive/block.rs:block_while_true` | VM Control Target | loop condition & body blocks | Migrate to `ControlPhase::WhileCondition`, `ControlPhase::WhileBody` |
| `primitive/boolean.rs:ifTrue/ifFalse` | VM Control Target | conditional branch evaluation | Migrate to direct shape tail-forwarding / `ControlPhase::BoolBranch` |
| `primitive/option.rs:Option.match` | VM Control Target | pattern match block evaluation | Migrate to direct shape tail-forwarding |
| `primitive/error.rs:Error.raise` | VM Control Target | dynamic `message` send | Make VM-visible with rooted receiver |
| `vm/dispatch.rs:Ordering.reverse` | VM Control Target | reverse comparison send | Migrate to VM-visible control step |
| `primitive/map.rs` / `set.rs` | Residual Host Algorithm | hash & equals probing loops | Retain `check_native_reentry` guard |
| `value/render.rs` | Residual Host Algorithm | recursive buffer formatting | Retain `check_native_reentry` guard |
| `primitive/typing.rs` | Residual Host Algorithm | reflective descriptor construction | Retain `check_native_reentry` guard |

---

## 3. Root & Lifetime Ledger

- `ControlStack` is owned by `FiberObject` (when parked) and `VM` (when executing).
- Live and parked GC traversal exhaustively visits all `ControlActivation` records in `ControlStack`, including:
  - `ControlPhase::OnMatch { error, handler }` (error Value and handler closure Value rooted)
  - `ControlPhase::EnsureCleanup { saved_transfer }` (saved Value or Raise error rooted)
  - `ControlStack.pending: Option<RoutedTransfer>` (transfer Value or Raise error rooted)
- Stack truncation and unwinding close open upvalues before destroying referenced frame slots.

---

## 4. Gate Progression Ledger

| Gate | Status | Command / Evidence | Result | Detail |
|---|---|---|---|---|
| **C0** | **COMPLETE** | `git status`, direct searches, baseline concurrency tests | PASS | Baseline clean (`ecc08066`), inventory classified, baseline tests green |
| **C1** | IN_PROGRESS | `cargo check -p phalcom-core`, unit tests | PENDING | Core control representations (`ControlStack`, `CallOutcome`, etc.) |
| **C2** | PENDING | transfer router, non-local unwind, host escape tests | PENDING | Unified transfers & host boundary escape |
| **C3** | PENDING | `on`/`ensure` suspendable fixtures | PENDING | VM-owned `on`/`ensure` phases |
| **C4** | PENDING | child failure -> parent Raise injection tests | PENDING | Parent call-site exception injection |
| **C5** | PENDING | `Bool`, `Option`, `whileTrue`, `Ordering` fallback tests | PENDING | Native fallback migration |
| **C6** | PENDING | GC tracing & traceback integration tests | PENDING | Lifetime & diagnostics validation |
| **C7** | PENDING | exact wait generation & queue admission tests | PENDING | Executor readiness handoff |
| **C8** | PENDING | full test suite, clippy, fmt, negative searches | PENDING | Final verification & closure |
