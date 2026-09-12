---
id: CONC002.C2
category: CONC
program: CONC002
checkpoint: CONC002.C2
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: VERIFIED
---

# CONC002.C2 — VM suspension, control continuations, and coroutine semantics

C2 owns VM control continuation state, coroutine-consumer/executor semantics, and the source-agnostic readiness handoff.

`CONC002.C2.P1-R1` (Native suspension and VM control continuations) is fully implemented and verified. The runtime control stack, unified transfers, shape ABI migrations, parent call-site exception injection, and genuine native re-entry boundary guards are active and certified. Next active work proceeds on `CONC002.C2.P2` (Coroutine consumer / executor separation and Fiber protocol completion).

---

## 1. Plans Ledger

| Plan | Scope | Status | Verification | Summary / Outcome |
|---|---|---|---|---|
| `CONC002.C2.P1-R1` | Native suspension, VM-owned protected/cleanup control, transfer routing, Call failure injection, native-control migration | **COMPLETE** | `VERIFIED` | VM-owned `ControlStack` implemented; `on`, `ensure`, `whileTrue`, `Bool`, and `Option` migrated to shape ABI; Call-mode parent exception injection verified; genuine host re-entry guards preserved. |
| `CONC002.C2.P2` | Coroutine consumer / executor separation, VM-owned queued driving, manual pending await, yield-await-yield, nested GC/failure verification, conditional `Fiber<R>` | **IMPLEMENTED** | `FOCUSED VERIFIED / WORKSPACE BASELINE BLOCKED` | Separates manual coroutine consumption from executor driving; unblocks `yield -> await -> yield`; lib, concurrency, and stream suites verified. The workspace gate reaches an independently reproduced REPL reflection-export baseline. |

---

## 2. Ownership Boundary

C2 owns:
- VM-resident protected/cleanup control records (`ControlStack`, `ControlActivation`);
- suspension-transparent language control combinators (`Block.on`, `Block.ensure`, `whileTrue`, `Bool`, `Option.match`);
- Call-mode child failure exception injection at the parent call site with preserved error identity;
- manual coroutine consumer vs executor driving separation (`coroutine_consumer` vs `executor_wait`);
- source-agnostic readiness validation (`Parked(generation) -> Queued`);
- final `Fiber<R>` surface typing resolution.

C2 does **not** own:
- reactor registrations, timers, worker pool, or poller backends — `CONC002.C3`;
- Future library composition/state cleanup — `CONC002.C4`;
- cancellation, `Task`/`TaskScope`, structured concurrency — `CONC002.C5`;
- channels and select — `CONC002.C6`.

Sequence:
```text
C1.P4 -> C2.P1-R1 -> C2.P2 -> C3 / C4
```

---

## 3. Established Invariants & Architectural Rules

1. **VM-Owned Control Continuations**: Language-level control combinators (`Block.on`, `Block.ensure`, `Block.whileTrue`, `Bool` conditional/lazy branches, `Option.match`) are represented as VM/Fiber-owned data structures (`ControlStack`), eliminating host re-entry frames.
2. **Protected Genuine Rust Frames**: The native-depth guard (`native_reentry_depth`) remains strictly active and verified for genuinely synchronous host algorithms (`Map`/`Set` hash/equality probing, string rendering, reflective typing).
3. **Explicit Activation Outcomes**: Method invocations and control activations return an explicit `CallOutcome` (`Returned(Value)`, `EnteredFrame`, `EnteredControl`, `SwitchedFiber`), avoiding frame-count heuristics.
4. **Unified Transfer Protocol**: `step_control_transfer` routes `Transfer::Returned`, `Transfer::Raise`, and `Transfer::NonLocalReturn` iteratively inside-out before frame destruction.
5. **Parent Call-Site Exception Injection**: In Call mode, child fiber failure does not eagerly fail ancestors; it injects a `Raise` at the exact parent call site with preserved error object identity.
6. **Executor Readiness Handoff**: Readiness notifications validate `Parked(generation)` and transition to `Queued` once without executing user guest code.

---

## 4. Migration & Residual Host Inventory

| Caller / Location | Classification | Live Rust State / Hazard | C2 Status |
|---|---|---|---|
| `primitive/block.rs:block_on` | VM Control Target | `on` matching and handler block invocation | **Migrated** to `ControlPhase::OnBody`, `ControlPhase::OnMatch`, `ControlPhase::OnHandler` |
| `primitive/block.rs:block_ensure` | VM Control Target | `ensure` cleanup block invocation | **Migrated** to `ControlPhase::EnsureBody`, `ControlPhase::EnsureCleanup` with saved transfer |
| `primitive/block.rs:block_while_true` | VM Control Target | loop condition & body blocks | **Migrated** to `ControlPhase::WhileCondition`, `ControlPhase::WhileBody` |
| `primitive/boolean.rs:ifTrue/ifFalse` | VM Control Target | conditional branch evaluation | **Migrated** to direct shape tail-forwarding / `ControlPhase::BoolBranch` |
| `primitive/option.rs:Option.match` | VM Control Target | pattern match block evaluation | **Migrated** to direct shape tail-forwarding (`option_match_shape`) |
| `primitive/error.rs:Error.raise` | VM Control Target | dynamic `message` send | **Migrated** to VM-visible send with rooted receiver |
| `vm/dispatch.rs:Ordering.reverse` | VM Control Target | reverse comparison send | **Migrated** to VM-visible control step |
| `primitive/map.rs` / `set.rs` | Residual Host Algorithm | hash & equals probing loops | **Retained** `check_native_reentry` guard |
| `value/render.rs` | Residual Host Algorithm | recursive buffer formatting | **Retained** `check_native_reentry` guard |
| `primitive/typing.rs` | Residual Host Algorithm | reflective descriptor construction | **Retained** `check_native_reentry` guard |

---

## 5. Root & Lifetime Ledger

- `ControlStack` is owned by `FiberObject` (when parked) and `VM` (when executing).
- Live and parked GC traversal exhaustively visits all `ControlActivation` records in `ControlStack`:
  - `ControlPhase::OnMatch { error, handler }` (error Value and handler closure Value rooted);
  - `ControlPhase::EnsureCleanup { saved_transfer }` (saved Value or Raise error rooted);
  - `ControlStack.pending: Option<RoutedTransfer>` (transfer Value or Raise error rooted).
- Stack truncation and unwinding close open upvalues before destroying referenced frame slots.

---

## 6. Verification Ledgers

### 6.1 C2.P1-R1 Evidence Ledger

All verification commands executed serially on `main` with cleared compiler flags (`RUSTFLAGS='' RUSTC_WRAPPER=''`):

| Gate | Scope | Command / Target | Result | Duration / Details |
|---|---|---|---|---|
| **C0** | Baseline & scope | `git status`, direct searches, baseline concurrency tests | **PASS** | Baseline clean; inventory classified; negative re-entry established |
| **C1** | Control ownership & types | `cargo check -p phalcom-core`, unit tests | **PASS** | `ControlStack`, `ControlActivation`, `ControlDestination`, `Transfer` |
| **C2** | Unified transfers | `cargo test -p phalcom-core --lib vm::control::tests` | **PASS** | Lifecycle, lookup, and exhaustive root tracing unit tests green |
| **C3** | Protected/cleanup controls | `cargo test -p phalcom-core --test language-corpus corpus::concurrency` | **PASS** | `concurrency_control_on_suspend.ph`, `concurrency_control_ensure_suspend.ph` |
| **C4** | Parent call injection | `cargo test -p phalcom-core --test language-corpus corpus::concurrency` | **PASS** | `concurrency_control_call_failure_parent_catch.ph` (error identity preserved) |
| **C5** | Native fallback migration | `cargo test -p phalcom-core --test language-corpus corpus::concurrency` | **PASS** | `concurrency_control_while_true_suspend.ph`, Bool lazy branches |
| **C6** | Diagnostics & depth | `cargo test -p phalcom-core --test core execution_depth_limits` | **PASS** | 5 passed (retained native probe `Set.add(Probe.new())` verified) |
| **C7** | Readiness handoff | `cargo test -p phalcom-core --lib scheduler_tests` | **PASS** | Monotonic park wake generation verified |
| **C8** | Full suite & clippy | `cargo clippy --workspace --all-targets -- -D warnings` | **PASS** | 0 warnings |
| **C8** | Core lib tests | `cargo test -p phalcom-core --lib` | **PASS** | 112 passed, 0 failed |
| **C8** | Core integration tests | `cargo test -p phalcom-core --test core` | **PASS** | 468 passed, 0 failed, 24 ignored |
| **C8** | Negative concurrency | `cargo test -p phalcom-core --test language-corpus corpus::concurrency_negative` | **PASS** | 13/13 tests green |
| **C8** | Language corpus | `cargo test -p phalcom-core --test language-corpus` | **PASS** | 60 passed, 0 unexpected failures |
| **C8** | Code formatting | `cargo fmt --all -- --check` | **PASS** | Clean across workspace |

---

## 7. Next Actions and Handoff

- **Active Plan**: `CONC002.C2.P2` (*Coroutine consumer / executor separation and Fiber protocol completion*) is implemented and focused-verified.
- **Handoff Document**: [`CONC002.C2.P2-handoff.md`](CONC002.C2.P2-handoff.md).
- **Release Gate**: Workspace certification remains blocked by the clean-`HEAD` `phalcom-repl --test repl_imports` reflection-export failure; it is outside C2's changed paths.
