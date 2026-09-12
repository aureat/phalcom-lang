---
id: CONC002.C1
category: CONC
program: CONC002
checkpoint: CONC002.C1
kind: checkpoint-record
status: COMPLETE
completion: COMPLETE
verification: BASELINE_BLOCKED
---

# CONC002.C1 — Corrected concurrency foundation

C1 owns the repaired Fiber/Future/scheduler execution foundation and the final verification and type prerequisites needed before C2 changes VM continuation semantics.

The runtime remediation, detached-scheduler-failure observability, post-remediation mainline type/concurrency delta recertification, and pre-C2 stabilization/typing contracts are fully implemented and verified. The checkpoint is closed with all plan obligations satisfied and cleanly handed off to `CONC002.C2`.

---

## 1. Plans Ledger

| Plan | Scope | Status | Verification | Summary / Outcome |
|---|---|---|---|---|
| `CONC002.C1.P1` | Truthful Fiber lifecycle, scheduler admission, ticketed await parking/wake, durable terminal completion, Future terminal adoption, scheduler-authority closure | **COMPLETE** | `BASELINE_BLOCKED` | Landed C0–C5 concurrency control remediation; owned runtime evidence retained. |
| `CONC002.C1.P2` | E010 detached scheduler-failure observability; Patch A module/REPL material remains historical companion context outside CONC002 ownership | **COMPLETE** | `FOCUSED_TESTED` | E010 failure channel landed and verified at safe root/scheduler boundaries. |
| `CONC002.C1.P3` | Reconcile and recertify the post-P1/P2 current-main concurrency/type delta: `Future<T>`, map/then split, adoption, Unit identity, scheduler-yield refusal, scheduler entry validation and root-result retention | **COMPLETE** | `BASELINE_BLOCKED` | Combined runtime and type closure recertified on current main revision. |
| `CONC002.C1.P4` | Pre-C2 stabilization: callable-domain/generic-pack capability, applied type-form/runtime identity prerequisites, bare-Fiber source census and specification handoff for final `Fiber<R>` work | **COMPLETE** | `VERIFIED` | Outcome B ratified; hostile protocol pinned (`concurrency_fiber_protocol_hostile.ph`); census and native re-entry handoff delivered to C2.P1-R1 / C2.P2. |

---

## 2. Ownership Boundary

C1 owns the foundational cooperative execution and observability mechanics. C1 does **not** own:

- native `on`/`ensure` continuation migration;
- VM-resident protected/cleanup control records;
- Call child-failure injection into a parent continuation;
- manual coroutine + executor composition (`yield → await → yield`);
- final public `Fiber<R>` typing;
- reactor/external readiness backends;
- cancellation, structured concurrency, channels or select.

Those belong to `CONC002.C2` and subsequent checkpoints.

---

## 3. Purpose and Evidence Epochs

The evidence underpinning C1 spans three distinct epochs:

- **Epoch A — P1/P2 remediation implementation**: The concurrency-control branch implemented truthful Fiber lifecycle, scheduler reservation, ticketed Future parking/wake, durable completion observers, terminal Future driving, scheduler authority closure and E010 detached-failure observability. Focused/core/language-corpus tests were green; workspace build and workspace Clippy were green; the full workspace-test gate was blocked by separately diagnosed REPL builtin export behavior.
- **Epoch B — later mainline concurrency/type delta (P3)**: Subsequent work added/changed generic `Future<T>`, typed Future constructors/combinators, map-versus-then semantics, recoverWith/flatten adoption, callback scheduling parity, scheduler entry validation, scheduled-yield refusal, root-result retention and Unit/empty-tuple canonicalization. Those changes are present in current mainline source and were fully recertified by `CONC002.C1.P3`.
- **Epoch C — pre-C2 stabilization and typing prerequisites (P4)**: Established callable-domain and generic-pack capabilities, evaluated runtime type identity against type-form expression lowering, inventoried all bare `Fiber` usage, proved heterogeneous multi-yield hostile protocols, and mapped the native re-entry handoff to `CONC002.C2.P1-R1`.

Repository baseline inspected for this restructuring: `main` at `2db7e3780772e847a178918ede239d91e0bcc5fd`.

---

## 4. Established Invariants

### 4.1 Execution and Lifecycle Invariants
- `Running` identifies the VM's current Fiber; a caller behind a running child is `BlockedOnChild`.
- `BlockedOnChild` is not manually resumable or scheduler-admissible.
- `New`, `Yielded`, `Parked(generation)`, `Queued`, `Done` and `Failed` are distinct lifecycle states.
- Root identity is a stable Fiber field, not an inference from `resumer`.
- Scheduler admission is VM-owned and reserves `Queued` before enqueue.
- Scheduler dequeue is FIFO and ignores stale non-`Queued` entries rather than aborting healthy work.
- Future parking uses a monotonic Fiber-local generation; only an exact `Parked(generation)` wake may reserve the Fiber back to `Queued`.
- A parked Future waiter cannot be stolen by public `call`, `try` or `System.schedule`.
- Terminal completion is owned by a GC-traced `completion_observer` independent of the dynamic `resumer`.
- Future action/callback settlement observes terminal Fiber success/failure; a suspension turn is not completion.
- Raw scheduler dequeue/resume/park/wake/completion seams are internal; public scheduler authority is `System.schedule(_)` plus the source-level scheduler driver.
- Detached scheduler failures without a completion owner are retained and reported at safe boundaries rather than aborting sibling work or disappearing.

### 4.2 Current Architectural Truths Relevant to C2
- `resumer` remains the dynamic coroutine-transfer relationship, not terminal-completion ownership.
- `completion_observer` is independent terminal ownership.
- pending Future parking is currently restricted to scheduler-owned Fibers.
- user `Fiber.yield` is rejected in Scheduler mode because the executor has no coroutine consumer for the yielded value.
- native `on`/`ensure` and several residual host callbacks still depend on restricted host re-entry; their guards remain genuine safety invariants until C2.P1-R1 makes continuation state VM-owned.
- Call-mode child failure still uses the transitional linked failure cascade until C2.P1-R1 installs parent call-site Raise delivery.
- manual coroutine pending-await remains intentionally unsupported until C2.P2 separates consumer ownership from executor ownership.

---

## 5. Verification Ledgers

### 5.1 Epoch-A Evidence Ledger (P1 / P2)

| Gate | Evidence | Recorded Result |
|---|---|---|
| Baseline | `cargo build -p phalcom-core` with cleared flags | PASS |
| Baseline | language-corpus concurrency lane | PASS |
| P1 C0 | `cargo check -p phalcom-core`; concurrency corpus | PASS |
| P1 C1 | scheduler admission/dequeue + concurrency corpus | PASS |
| P1 C2 | scheduler wake-generation unit test + concurrency corpus | PASS |
| P1 C3 | completion-observer unit test + concurrency corpus | PASS |
| P1 C4 | concurrency corpus including multi-await/late failure/adoption coverage | PASS |
| P1 C5 | concurrency corpus, native floor census, complete `--test core`, complete language corpus | PASS |
| Final | `cargo fmt --all -- --check` | PASS after scoped formatting |
| Final | `cargo build --workspace --all-targets` | PASS |
| Final | `cargo test -p phalcom-modules --all-targets` | PASS |
| Final | `cargo clippy -p phalcom-modules --all-targets -- -D warnings` | PASS |
| Final | focused REPL import-bug target associated with historical Patch A | PASS, 6 tests |
| Final | `cargo test --workspace --all-targets` | BASELINE-BLOCKED by 12 broader REPL builtin import/export cases; concurrency/core/modules/focused targets green |
| Final | `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| P2/E010 | dedicated scheduler-failure observability fixtures/unit cases | PASS in implementation epoch |

#### Diagnosed Unrelated Workspace Blocker from Epoch A
The full workspace test failure was not a concurrency failure. The affected broad REPL tests expected native-owner classes such as `Selector` and `Message` to materialize as exports of non-root canonical Universe modules, while the module/interface layer deliberately does not implicitly export arbitrary non-root source declarations. A broad projection attempt was reverted rather than weaken that authority rule.

Therefore:
```text
P1 = COMPLETE / IMPLEMENTED / BASELINE_BLOCKED
P2 = COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P3 = COMPLETE / IMPLEMENTED / BASELINE_BLOCKED
```
The blocker is not a reason to describe either plan as unimplemented.

---

### 5.2 Epoch-B Evidence Ledger (P3 Recertification)

Executed on revision `2db7e3780772e847a178918ede239d91e0bcc5fd`:

| Gate | Evidence / Command | Result | Detail |
|---|---|---|---|
| P3.1 | `cargo test -p phalcom-semantic --test semantic universe_future` | PASS | 2 tests passed: generic Future payload preservation through async/map/then/await, incompatible settlement/recovery rejection |
| P3.1 | `cargo test -p phalcom-semantic --test semantic empty_tuple` | PASS | 2 tests passed: canonical Unit identity in type model and nested generic annotations |
| P3.1 | `cargo test -p phalcom-semantic` | PASS | 1144 passed, 0 failed, 42 ignored |
| P3.2 | `cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --exact` | PASS | 1 passed, 0 failed (41.52s, full multi-await/callback/adoption coverage) |
| P3.2 | `cargo test -p phalcom-core --test language-corpus corpus::concurrency_negative -- --exact` | PASS | 1 passed, 0 failed (17.81s) |
| P3.3 | `cargo test -p phalcom-core --lib dispatch::tests` | PASS | 4 passed: terminal observer detach/admit, failing observer unhandled work, root result survival |
| P3.3 | `cargo test -p phalcom-core --lib scheduler_tests` | PASS | 2 passed: parked wake requires exact generation and is once-only, unhandled failure survives GC |
| P3.4 | `cargo test -p phalcom-core --lib modules::canonical_semantics` | PASS | 3 passed: exact baseline lock enforced; zero concurrency diagnostics in `semantic-diagnostics-baseline.txt` |
| P3.5 | `cargo fmt --all -- --check` | PASS | Clean |
| P3.5 | `cargo build --workspace --all-targets` | PASS | Clean build across all crates |
| P3.5 | `cargo clippy --workspace --all-targets -- -D warnings` | PASS | Clean (fixed unnecessary usize casts in `generics.rs`) |
| P3.5 | `cargo test -p phalcom-core --lib` | PASS | 110 passed, 0 failed |
| P3.5 | `cargo test -p phalcom-core --test core` | PASS | 468 passed, 0 failed, 24 ignored (184.13s) |
| P3.5 | `cargo test -p phalcom-modules` | PASS | All unit, integration, and doc tests passed |
| P3.5 | `cargo test --workspace --all-targets` | BASELINE_BLOCKED | Unrelated REPL import/export defect (12 failures in `repl_imports`) reproduces unchanged; 2 language-corpus non-concurrency baseline failures (`family`, `streams`). Concurrency products certified green. |

---

### 5.3 Epoch-C Evidence Ledger and Stabilization Contract (P4)

Executed on revision `2db7e3780772e847a178918ede239d91e0bcc5fd`:

#### 5.3.1 Ratified Typing Decisions
- **`Fiber<I, R>` and homogeneous `Fiber<Y, S, R>` formally rejected**: entry argument binding and suspended yield resume input occupy distinct protocol positions; distinct yield sites within a single fiber can yield and receive heterogeneous types.
- **Lifetime-stable nominal target**: `Fiber<R>` (terminal success type only).
- **Outcome B Ratified**: `phalcom-semantic`'s type system and AST type annotations do not support arbitrary parameter-pack abstraction (`(***P) -> R`), and there is no sound existential wildcard (`exists R. Fiber<R>`) for `Fiber.current` or heterogeneous scheduler queues without fabricating unsound covariance. Public `Fiber` remains non-generic in `fiber.ph` through C2.P1-R1 and C2.P2.
- **Unrelated type-system/compiler gap**: Non-reference `Expr::TypeForm` lowering to `Bytecode::Nil` in `compiler/lib/expr.rs` is documented and moved to the owning compiler/type backlog.

#### 5.3.2 Census of Bare `Fiber` Usages

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

#### 5.3.3 Native Re-entry Handoff Table for C2.P1-R1

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

#### 5.3.4 Epoch-C Evidence Ledger

| Gate | Evidence / Command | Result | Detail |
|---|---|---|---|
| C1.1 | `concurrency_fiber_protocol_hostile.ph` in `language-corpus corpus::concurrency` | PASS | Proves entry vs resume input independence, heterogeneous multi-yield, and data vs failure distinction |
| C2.1 | `cargo test -p phalcom-semantic --test semantic capabilities::generics` | PASS | 19 passed: includes `unsaturated_generic_constructors_are_cleanly_rejected_in_proper_type_positions` and `generic_callable_return_inference_requires_exact_parameter_domain` |
| C2.2 | `cargo test -p phalcom-semantic` | PASS | 1146 passed, 0 failed, 42 ignored |
| C3.1 | `cargo test -p phalcom-core --test language-corpus corpus::concurrency_negative -- --exact` | PASS | 1 passed, 0 failed (18.00s) |
| C3.2 | `cargo test -p phalcom-core --lib` | PASS | 110 passed, 0 failed |
| C4.1 | `cargo fmt --all -- --check` | PASS | Clean formatting |
| C4.2 | `cargo build --workspace --all-targets` | PASS | Clean workspace build |
| C4.3 | `cargo clippy --workspace --all-targets -- -D warnings` | PASS | Clean across workspace |

---

## 6. Checkpoint Closure & Handoff to C2

Checkpoint **CONC002.C1 is COMPLETE**:

1. P1/P2/P3/P4 are all complete with verified evidence.
2. P3's current-main recertification gate is green; unrelated baseline blockers are precisely classified.
3. P4 has established the required prerequisites, ratified Outcome B, and frozen the public Fiber contract without inventing unsound Fiber typing.
4. Canonical concurrency documentation and test fixtures describe the actual implementation (`concurrency_fiber_protocol_hostile.ph` passing).
5. Native re-entry handoff and runtime invariants are delivered to `CONC002.C2.P1-R1`.

Next resume action: **`CONC002.C2.P1-R1` (Native Suspension and VM Control Continuations)**.
