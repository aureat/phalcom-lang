---
id: CONC002.C1.P1
category: CONC
program: CONC002
checkpoint: CONC002.C1
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
depends_on: []
follows: null
supersedes: null
deferred_reason: null
---

# CONC002.C1.P1 — concurrency control remediation

## Phalcom Concurrency Control Remediation
## Repository-Grounded, Checkpoint-Driven, Patch-Grade Implementation Plan

**Program:** Fiber / Scheduler / Future control-transfer correctness  
**Repository:** `aureat/phalcom-lang`  
**Prepared against remote branch:** `main`  
**Prepared against exact HEAD:** `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`  
**HEAD subject:** `fix(parser): preserve infix operator ranges across newlines`  
**Repository toolchain:** `nightly-2026-07-10` via `rust-toolchain.toml`  
**Planning date:** 2026-09-09

> **Repository-state limitation:** this plan was prepared against the remote GitHub repository. The local working tree was not visible during planning. Before implementation, the implementing agent must run `git status --short`, `git rev-parse --abbrev-ref HEAD`, and `git rev-parse HEAD`, preserve unrelated work, and apply the drift protocol below.

---

# 1. Implementation Program

The implementation program repairs one focused semantic boundary:

> **A Phalcom Fiber must have one truthful execution state, one unambiguous authority for each nonterminal resume, and one durable terminal-completion path that is independent of the Fiber's dynamic coroutine `resumer`.**

The program deliberately preserves:

- the single-threaded cooperative runtime;
- the current heap-resident Fiber representation;
- O(1) stack/frame ownership transfer;
- public `Fiber.new`, `call`, `try`, and `yield` manual-coroutine semantics;
- `call`-mode linked terminal failure and `try`-mode containment for this patch;
- uncolored direct-style `Future#await`;
- `Future<T>` as a library-level eventual-result abstraction rather than a second execution engine;
- current close-before-discard upvalue behavior;
- the native re-entry safety restriction.

The program corrects three shared root defects rather than patching each observed symptom separately:

1. **No durable terminal-completion owner.** `resumer` is dynamic and is overwritten when the scheduler resumes a Fiber; `Future.async` and pending continuations therefore cannot rely on one `try()` turn to represent terminal completion.
2. **No exclusive park/wake authority.** `Future#await` currently registers a raw Fiber and performs ordinary `Fiber.yield(None)`, so any caller can resume the Fiber and stale Future waiters can later advance an unrelated suspension.
3. **The Fiber lifecycle state is too coarse and currently false.** `Suspended` conflates new/yielded/await-parked execution, while a caller parked behind a running child remains marked `Running`.

This is **not** a general concurrency redesign.

---

# 2. Checkpoint Map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| **C0 — Truthful Fiber lifecycle and root identity** | 1–4 | `FiberStatus` truthfully distinguishes new, running, child-blocked, yielded, parked, queued, and terminal states; root identity no longer depends on `resumer` | focused Fiber lifecycle tests; existing manual coroutine/failure corpus; compile fanout | Future/await tests, full `phalcom-core`, workspace |
| **C1 — Scheduler owns ready admission and scheduler resumes** | 5–8 | queue admission reserves a Fiber exactly once; scheduler resume is distinct from public `try`; duplicate/stale queue entries cannot kill unrelated work | FIFO regression; duplicate-schedule hostile case; scheduled-failure isolation comparator; focused `language-corpus` concurrency subset | Future parking, async completion, full crate |
| **C2 — `await` is an authorized park, not a coroutine yield** | 9–12 | non-root await parks with a unique ticket; only the matching Future wake can move `Parked → Queued`; manual resume/schedule cannot steal the park | wrong-authority hostile case; stale-ticket hostile case; native-boundary no-stale-waiter case; existing await suspension fixture | `Future.async`, continuation callbacks |
| **C3 — Terminal completion is durable and independent of `resumer`** | 13–16 | a scheduler-managed Fiber can bind one completion observer that survives any number of park/wake turns and fires once on `Done`/`Failed`, including call-mode cascades | terminal observer success/failure tests; multi-turn survival probe; call-cascade observer hostile case; GC retention test | `Future.async` rewrite |
| **C4 — Futures adopt terminal computation outcomes** | 17–20 | `Future.async` and pending `then`/`map`/`catch` settle only from terminal action/callback outcomes; no driver-turn result is mistaken for completion | E007 regression; multi-await; late failure; suspending continuation matrix; immediate path comparators | broad `phalcom-core`, docs closure |
| **C5 — Ownership closure, compatibility cleanup, and specification alignment** | 21–24 | raw scheduler internals no longer act as competing authority; audit regressions are permanent; canonical docs describe the repaired model | negative searches; complete concurrency corpus; `phalcom-core` tests; docs consistency checks | workspace/release gates |
| **Final Gate** | — | delivery compatibility and repository hygiene | format, workspace build/test/clippy, deferred-evidence audit | none |

Implementation order is mandatory. In particular:

```text
truthful state
    ↓
scheduler reservation
    ↓
ticketed park/wake
    ↓
durable completion observer
    ↓
Future.async / continuations
    ↓
API/docs closure
```

Do **not** start by editing `Future.async`.

---

# 3. Repository Grounding

## 3.1 Relevant repository organization

The requested work is almost entirely owned by `phalcom-core/`.

| Location | Responsibility in this program |
|---|---|
| `phalcom-core/src/heap/fiber.rs` | authoritative Fiber lifecycle and parked execution metadata |
| `phalcom-core/src/primitive/fiber.rs` | public/manual Fiber resume/yield API plus new internal park/completion primitives |
| `phalcom-core/src/primitive/system.rs` | scheduler admission and internal wake/dequeue primitives |
| `phalcom-core/src/vm/mod.rs` | VM-owned `ready_queue`, current Fiber, scheduler bookkeeping |
| `phalcom-core/src/vm/dispatch.rs` | Fiber-floor success/failure, resumer restoration, root-drive scheduler execution |
| `phalcom-core/src/heap/trace.rs` | GC traversal of Fiber-held object references |
| `phalcom-core/src/vm/gc.rs` | ready-queue rooting if queue representation changes |
| `phalcom-core/core/universe/src/concurrency/fiber.ph` | public `System`/`Fiber` declarations and pure-Phalcom `Future` implementation |
| `phalcom-core/tests/fixtures/language/concurrency/` | production-faithful end-to-end concurrency fixtures |
| `phalcom-core/tests/core/` and source-local `#[cfg(test)]` modules | lower-level VM/Fiber state invariants |
| `docs/spec/current/concurrency.md` | canonical concurrency semantics |
| `docs/spec/current/system.md` | canonical `System` scheduler surface |
| `docs/spec/current/core/floor-census.md` | native floor inventory |
| `docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md` | accepted architectural rationale that must be amended where its Future-forward-compat assumption is no longer true |
| `docs/work/errors/E007-async-await-missettle.md` | existing blocker for premature `Future.async` settlement |
| `docs/work/errors/E008-double-schedule-kills-run.md` | existing high-severity duplicate queue failure |
| `docs/work/errors/E010-scheduler-pump-swallows-task-errors.md` | adjacent open issue; compatibility constraint only, not part of this program |

### Explicitly out of repository scope

Do not investigate or modify unless a checkpoint escalation proves a direct dependency:

- `phalcom-ast/`;
- `phalcom-semantic/`;
- `phalcom-modules/`;
- `phalcom-lsp/`;
- parser grammar;
- type inference;
- generic typing;
- reactor implementation;
- timers and I/O;
- cancellation;
- structured concurrency;
- multithreading;
- actor models.

A need to cross one of those boundaries is an **escalation event**, not permission to expand.

---

## 3.2 Current implementation anchors

### `phalcom-core/src/heap/fiber.rs`

Current authoritative types:

```rust
FiberStatus {
    Suspended,
    Running,
    Done,
    Failed,
}

FiberResumeMode {
    Call,
    Try,
}
```

Current `FiberObject` carries:

```text
stack
frames
open_upvalues
status
resumer
result
entry
started
resume_slot
floor_depth
resume_mode
checking
seq
spawn_file
spawn_line
```

Important current mismatch:

- the documentation says `Running` is the `VM::current` Fiber whose stacks live in the VM;
- `fiber_resume` parks the caller's stacks but does not move the caller out of `Running`.

Important disproven assumption:

- the file documents `resumer + result` as deliberately general enough for Future/await;
- the audit shows scheduler resumption necessarily overwrites `resumer`, so it cannot also be durable completion ownership.

### `phalcom-core/src/primitive/fiber.rs`

Primary paths:

```text
store_live_into
load_live_from
new_fiber_ref
fiber_is_root
fiber_abort
fiber_call
fiber_try
fiber_resume
fiber_yield
```

Current `fiber_resume`:

1. checks native re-entry;
2. accepts only `Suspended`;
3. validates first-resume arity;
4. parks the current Fiber's stacks;
5. overwrites `callee.resumer`;
6. records `resume_mode`;
7. restores/creates callee execution state;
8. sets callee `Running`;
9. changes `VM::current`.

Current `fiber_yield`:

1. infers rootness from `resumer.is_none()`;
2. validates native re-entry;
3. records `resume_slot`;
4. marks the current Fiber `Suspended`;
5. parks its stacks;
6. restores its resumer.

### `phalcom-core/src/primitive/system.rs`

Current `system_schedule`:

- if the argument is already a Fiber, enqueue that exact Fiber;
- otherwise create a fresh Fiber;
- unconditionally `ready_queue.push_back(fiber_ref)`.

There is no state reservation, duplicate check, park authority, or admission validation.

Current `system_next_scheduled` exposes a raw queue pop.

### `phalcom-core/src/vm/dispatch.rs`

`VM::run_until(0)` owns:

- non-root successful Fiber terminalization;
- non-root failure capture and Call/Try cascade;
- root-drive ready-queue pumping.

`switch_to_fiber_and_deliver` owns restoration of a parked resumer and currently marks it `Running`.

This makes it a central state-transition boundary.

### `phalcom-core/core/universe/src/concurrency/fiber.ph`

Current Future behavior:

```text
Future#await
    pending root      → pump System.nextScheduled / Fiber.try
    pending non-root  → push raw Fiber waiter + Fiber.yield(None)

Future.async
    driver Fiber
        → action Fiber
        → action.try() exactly once
        → non-error return treated as terminal success

pending then/map/catch
    continuation closure
        → callback Fiber
        → callback.try() exactly once
        → non-error return flattened as terminal result
```

Those one-turn assumptions are the direct implementation mechanism behind AUD-001 and AUD-002.

---

# 4. Source-of-Truth Model

The implementation must preserve these authority boundaries.

| Concern | Source of truth | Derived consumers | Forbidden competing authority |
|---|---|---|---|
| Currently executing Fiber | `VM::current` | `FiberStatus::Running`, VM live stack/frame mirrors | multiple Fibers marked `Running` |
| Fiber lifecycle eligibility | `FiberObject.status` | `call`, `try`, scheduler admission, wake | guessing from `started`, `resumer`, queue membership alone |
| Immediate coroutine recipient | `FiberObject.resumer` + `resume_mode` | `yield`, terminal return delivery, Call/Try failure routing | using `resumer` as logical async-task owner |
| Root identity | stable Fiber identity field/constructor fact | `isRoot`, root yield/abort restrictions | `resumer.is_none()` |
| Await wake authority | `Parked(wait_generation)` on the Fiber plus matching waiter generation | internal Future wake | raw Fiber waiter |
| Scheduler ownership | `Queued` Fiber state + `VM::ready_queue` | scheduler pump | duplicate raw queue entries / O(n) queue membership scans |
| Terminal Fiber outcome | `Done`/`Failed` + `FiberObject.result` | completion observer, public `error` | one resume-turn return value |
| Async completion ownership | `FiberObject.completion_observer` (or repository-equivalent single durable binding) | `Future.async`, pending Future continuations | dynamic `resumer` |
| Future settlement | pure `.ph` Future state machine | `await`, `then`, `map`, `catch` | VM directly knowing Future internals |

---

# 5. Ratified Semantic Decisions for This Program

These decisions are the semantic design of the patch. Mechanics may adapt to repository drift; these decisions may not be silently changed.

## D-01 — Preserve manual coroutine semantics

`Fiber.call`, `Fiber.try`, and `Fiber.yield` remain the public manual-coroutine protocol.

A yielded Fiber may be manually resumed. A Fiber parked on an unresolved Future may not.

## D-02 — `Running` means exactly current

After the patch:

```text
FiberStatus::Running ⇔ VM.current == that Fiber
```

An active caller whose child currently executes is **not** `Suspended`; it is a distinct non-resumable `BlockedOnChild` state or a repository-equivalent state carrying the same invariant.

## D-03 — Distinguish nonterminal reasons

The runtime must distinguish at least:

```text
New
Running
BlockedOnChild
Yielded
Parked(wait_generation)
Queued
Done
Failed
```

The exact Rust enum spelling can change if repository mechanics require it, but those semantic categories must not collapse back together.

Keep `started` initially if doing so reduces patch fanout. It may remain an entry-frame bookkeeping bit even after `New` is explicit.

## D-04 — `System.schedule` is scheduler admission, not Future wake

For compatibility, `System.schedule(existingFiber)` may continue to accept a Fiber only when it is an **unowned runnable Fiber** (`New`, and optionally `Yielded` if existing compatibility tests require it).

It must reject:

```text
BlockedOnChild
Parked
Queued
Running
Done
Failed
```

A Future wake uses an internal ticket-aware wake operation.

Do not scan the queue to deduplicate.

## D-05 — Scheduler resume is an internal resume mode

Add an internal scheduler resume path, preferably represented as:

```rust
FiberResumeMode::Scheduler
```

or an equivalent explicit discriminator.

Do not continue to model scheduler execution as ordinary public `try()` merely because both isolate an uncaught child failure from unrelated scheduled work.

## D-06 — Await parking is ticketed

Every successful non-root async park obtains a monotonically increasing per-Fiber generation.

The Future registration captures:

```text
Fiber identity + exact generation
```

Only a matching wake may perform:

```text
Parked(generation) → Queued
```

A stale wake is harmless. A duplicate wake is harmless.

Use a fixed-size integer compatible with Phalcom `Int`; handle overflow explicitly rather than silently recycling a still-live ticket.

## D-07 — Await under a manual Fiber boundary is rejected for this patch

The minimal coherent policy is:

```text
root Fiber
    await via root scheduler pump

scheduler-resumed Fiber
    await via Parked(ticket)

manually call/try-resumed Fiber
    await rejected before waiter registration
```

This preserves uncolored suspension through ordinary Phalcom method calls inside a scheduler-owned Fiber.

It deliberately does **not** implement transparent parking of an entire manual Fiber-call chain or async generators. That is separate architecture.

## D-08 — Terminal completion observer is durable and generic

A Fiber may carry one internal completion observer that:

- is independent of `resumer`;
- survives any number of park/wake cycles;
- fires exactly once when the Fiber becomes `Done` or `Failed`;
- also fires if a Call-mode failure cascade terminally fails the Fiber;
- is cleared/moved once terminal notification is scheduled;
- is GC-traced while attached.

The VM must not know Future settlement semantics.

## D-09 — Completion observers run later, not inline at the Fiber floor

Terminalization must:

1. commit `Done`/`Failed` and `result`;
2. detach the observer exactly once;
3. enqueue the observer as fresh scheduler work;
4. continue normal resumer restoration/failure cascade.

Do not execute arbitrary Phalcom observer bytecode inside the terminalization branch.

## D-10 — Preserve Call/Try failure semantics in this program

For this patch:

```text
Call-mode child uncaught failure
    → linked terminal failure through Call-mode resumers

Try-mode boundary
    → error delivered as a value
```

Do not redesign this into normal handler lookup at the `call()` expression.

Update comments that claim stronger equivalence than the runtime actually provides.

## D-11 — `E010` remains outside this program

`docs/work/errors/E010-scheduler-pump-swallows-task-errors.md` is a separate error-observation policy problem.

The new scheduler resume path must not accidentally make E010 worse or silently declare it fixed.

---

# 6. Required Invariants

| ID | Invariant |
|---|---|
| **CONC-STATE-01** | Exactly one Fiber is `Running`, and it is `VM.current`. |
| **CONC-STATE-02** | A caller parked behind a running child is explicitly non-resumable. |
| **CONC-STATE-03** | Coroutine yield and Future park are different runtime states. |
| **CONC-ROOT-01** | Root identity is stable and independent of `resumer`. |
| **CONC-PARK-01** | Every async park owns a unique wait generation. |
| **CONC-PARK-02** | `call`/`try` reject an async-parked Fiber before mutating caller or callee state. |
| **CONC-PARK-03** | A queued Fiber cannot be manually resumed or admitted to the queue again. |
| **CONC-WAKE-01** | Only a matching wait generation can move `Parked → Queued`. |
| **CONC-WAKE-02** | A stale wake has no execution effect. |
| **CONC-WAKE-03** | Duplicate wakes cannot create duplicate live queue entries. |
| **CONC-SCHED-01** | Scheduling new/unowned work and waking Future-parked work are separate operations. |
| **CONC-SCHED-02** | Invalid scheduling is rejected at admission rather than deferred until pump execution. |
| **CONC-SCHED-03** | A defensive stale queue entry cannot prevent unrelated healthy work from running. |
| **CONC-SCHED-04** | All scheduler drivers use the same queue-state and scheduler-resume semantics. |
| **CONC-AWAIT-01** | Non-root await parks; it does not implement parking as ordinary `Fiber.yield(None)`. |
| **CONC-AWAIT-02** | `await` rechecks Future readiness after a wake. |
| **CONC-AWAIT-03** | A failed park attempt cannot leave an actionable waiter authority behind. |
| **CONC-COMP-01** | Scheduler-managed computation has a durable terminal observer independent of `resumer`. |
| **CONC-COMP-02** | The observer fires exactly once, only on `Done`/`Failed`. |
| **CONC-COMP-03** | The observer survives arbitrary park/wake cycles. |
| **CONC-COMP-04** | Call-mode failure cascade notifies observers of every Fiber it terminally fails. |
| **CONC-FUT-01** | `Future.async` remains pending across every nonterminal suspension. |
| **CONC-FUT-02** | `Future.async` adopts the action's actual terminal value/error. |
| **CONC-FUT-03** | Pending `then`/`map`/`catch` use the same terminal-computation machinery. |
| **CONC-FUT-04** | Existing settle-once behavior remains intact. |
| **CONC-GC-01** | New Fiber-held observer references remain live across GC. |
| **CONC-GC-02** | Consumed observer/park metadata does not retain terminal work unnecessarily. |
| **CONC-PERF-01** | Fiber switching and ready-queue operations remain O(1). |
| **CONC-PERF-02** | No locks/atomics are introduced into the single-thread scheduler. |
| **CONC-ERR-01** | Existing Error object identity survives Call/Try terminal propagation. |
| **CONC-UPVALUE-01** | Existing close-before-discard behavior remains intact. |
| **CONC-NATIVE-01** | Native suspension guards reject before a valid wait authority can be consumed. |

---

# 7. Tempting Wrong Fixes — Explicitly Forbidden

Do not implement any of these as the primary repair:

1. **Only add `if fib.isDone` to `Future.async`.**  
   This stops premature settlement but strands the result Future forever after the action parks.

2. **Only make `Future#await` loop until ready.**  
   Rechecking a condition does not establish who is authorized to wake the Fiber.

3. **Only skip `isDone` waiters in `Future#drain`.**  
   A stale waiter can target a still-live Fiber now parked on a different Future.

4. **Only skip `Done`/`Failed` Fibers during queue pumping.**  
   This does not prevent duplicate admission or stale wake authority.

5. **Deduplicate `ready_queue` with an O(n) scan.**  
   Queue membership is not the source of truth; Fiber scheduler state is.

6. **Mark a child caller `Suspended`.**  
   This makes an active ancestor manually resumable while its child still owns control.

7. **Keep using `resumer` as async completion ownership.**  
   Scheduler resume intentionally overwrites the dynamic resumer.

8. **Keep raw Fiber waiters and add an `isReady` check.**  
   The raw Fiber waiter still has authority over the wrong suspension generation.

9. **Run terminal observers inline in `run_until` terminalization.**  
   This reintroduces re-entrant execution exactly at a sensitive control-transfer seam.

10. **Wrap async callbacks in existing native-driven exception combinators to recover state.**  
    Do not reintroduce the native-frame suspension problem.

11. **Introduce public `Task`, `TaskGroup`, cancellation, reactors, or function coloring.**  
    None is required to repair the audited correctness defects.

---

# 8. Checkpoint C0 — Truthful Fiber Lifecycle and Root Identity

Tasks:
- Task 1 — Expand `FiberStatus` into truthful execution states.
- Task 2 — Make manual `call`/`try` transitions block the caller explicitly.
- Task 3 — Make `yield` restore truthful caller/child state.
- Task 4 — Make root identity stable and migrate root checks.

## Why this is a checkpoint

The scheduler and Future fixes must be able to authorize behavior from Fiber state. They cannot safely build on the current lie where:

- multiple Fibers may be `Running`;
- `Suspended` means both unstarted and yielded;
- rootness is inferred from a transient `resumer`.

C0 establishes the state language used by every later checkpoint.

## Entry conditions

- remote reference commit remains semantically equivalent to `b84da68f...`;
- `FiberObject` remains the owner of parked stacks/frames;
- `VM::current` remains the unique execution owner.

## Working set

Primary:

- `phalcom-core/src/heap/fiber.rs` — `FiberStatus`, `FiberResumeMode`, `FiberObject`, constructors.
- `phalcom-core/src/primitive/fiber.rs` — `fiber_resume`, `fiber_yield`, root/error reflection.
- `phalcom-core/src/vm/dispatch.rs` — `switch_to_fiber_and_deliver`, terminal success/failure.
- `phalcom-core/core/universe/src/concurrency/fiber.ph` — native Fiber declarations only if root/internal state surface changes.

Secondary — inspect only if evidence requires it:

- `phalcom-core/src/vm/bootstrap.rs` — root constructor call.
- `phalcom-core/src/heap/trace.rs` — only if C0 adds an object-valued field; planned C0 fields are scalar.

Out of scope:

- Future implementation changes;
- scheduler queue behavior;
- docs beyond comments adjacent to changed state.

## Semantic contract established by C0

- `Running` iff `VM.current`.
- Active parent → `BlockedOnChild` before child becomes current.
- Unstarted Fiber and Fiber suspended at explicit `yield` are distinguishable.
- `call`/`try` accept only manual-runnable states.
- root identity is constructor-stable.

## Semantic risks

- accidentally making an active ancestor resumable;
- changing manual yield/resume value delivery;
- corrupting `resume_slot`;
- changing first-resume arity validation ordering;
- breaking Call/Try failure cascade;
- changing root-drive assumptions before scheduler work lands.

## Hostile cases

- A resumes B; B attempts to resume A — must reject A as active/blocked.
- Fresh Fiber `isRoot` — must be false.
- Root `Fiber.yield` and `Fiber.abort` — still rejected.
- `Fiber.yield(None)` versus terminal `return None` — state distinguishes them.

## Required evidence

1. `RUSTFLAGS='' cargo check -p phalcom-core`  
   **Proves:** exhaustive Rust match/caller migration compiles after expanding `FiberStatus`/`FiberResumeMode`.  
   **Does not prove:** lifecycle transitions are semantically correct.

2. Run exact existing manual Fiber fixtures from `phalcom-core/tests/fixtures/language/concurrency/`, including the lifecycle, nested current/resumer, Call/Try failure, and upvalue-close cases. Use `-- --list` first to confirm filters.  
   **Proves:** the old manual coroutine contract survives the state rewrite.

3. Add/extend one focused runtime test for stable root identity and one hostile nested-resume test.  
   **Proves:** rootness and active-ancestor non-resumability are not accidental side effects.

## Do not run yet

- full `language-corpus`;
- full `phalcom-core`;
- workspace tests;
- Future async regressions.

They provide no additional evidence until scheduler/park states exist.

## Escalate immediately if

- first-resume validation can still fail after the caller has been irreversibly parked;
- any code outside `phalcom-core` switches on `FiberStatus`;
- making caller state truthful requires changing stack layout rather than control metadata;
- an existing fixture depends on treating an active caller as `Running`.

## Checkpoint completion

- [ ] Tasks 1–4 implemented.
- [ ] required manual Fiber evidence passes.
- [ ] hostile nested-resume case passes.
- [ ] fresh Fiber rootness regression passes.
- [ ] no `FiberStatus::Suspended` production semantics remain ambiguous.
- [ ] state file updated.
- [ ] no active incident.

### Suggested commit group

```text
refactor(concurrency): make fiber lifecycle states truthful
fix(concurrency): make root fiber identity explicit
test(concurrency): lock manual fiber state invariants
```

---

## Task 1 — Expand `FiberStatus` into truthful execution states

Purpose:  
Give the VM enough state to distinguish manual coroutine suspension, async parking, scheduler reservation, and active caller blocking.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

Owned files and symbols:

- `phalcom-core/src/heap/fiber.rs` — `FiberStatus`, `FiberObject::new_entry`, `new_entry_with_buffers`, `root`.
- production matches discovered by `rg 'FiberStatus::' phalcom-core/src phalcom-core/tests`.

Inspect before editing:

- all current `FiberStatus` matches;
- `FiberObject.started`;
- `VM::switch_to_fiber_and_deliver`;
- terminalization branches in `VM::run_until`.

Do not inspect unless evidence forces expansion:

- semantic/type crates;
- parser/compiler lowering.

Dependencies:
- none.

Source of truth:
- `FiberObject.status`.

Implementation boundary:

### STRUCTURAL — required state shape

Use one state representation with at least these meanings:

```rust
pub enum FiberStatus {
    New,
    Running,
    BlockedOnChild,
    Yielded,
    Parked(i64), // exact integer type may adapt to Value::Int constraints
    Queued,
    Done,
    Failed,
}
```

Requirements:

- keep the enum `Copy`/cheap;
- keep `started` during this program unless removing it is demonstrably simpler and all first-resume paths are exhaustively migrated;
- add a monotonic park-generation field in C2, not prematurely if it complicates C0;
- do not encode root identity as another status.

Current implementation:

```text
Suspended = new OR yielded OR await-parked
Running   = current OR active parked caller
```

Target implementation:

```text
New              unstarted, manually/scheduler-admissible
Running          exactly VM.current
BlockedOnChild   stacks parked because a child currently owns execution
Yielded          explicit coroutine yield, manually resumable
Parked(g)        Future-owned async wait
Queued           scheduler owns next resume
Done/Failed      terminal
```

Edit operations:

1. OPEN `phalcom-core/src/heap/fiber.rs`.
2. FIND `pub enum FiberStatus`.
3. REPLACE `Suspended` with explicit `New` and `Yielded`; add `BlockedOnChild`, `Parked(...)`, `Queued`.
4. UPDATE constructors:
   - `new_entry` → `New`;
   - `new_entry_with_buffers` → `New`;
   - `root` → `Running`.
5. SEARCH all production `FiberStatus::` matches.
6. For C0, migrate only cases whose intended state is already known:
   - finished checks → `Done | Failed`;
   - current execution → `Running`;
   - manual resume eligibility → `New | Yielded`.
7. Leave scheduler/park-specific matches as compile failures or explicit TODO anchors only until C1/C2; do not map them back to generic manual resume.
8. Update rustdoc so each state documents who may resume it.
9. CLEAN obsolete “Suspended = created or yield” comments.

Testing classification:
- no standalone behavioral test for the enum edit; validated by C0.

Optional compile checkpoint:

```bash
RUSTFLAGS='' cargo check -p phalcom-core
```

Run after the exhaustive match migration in Tasks 1–3, not after the enum declaration alone.

Checkpoint state update:
- record final enum spelling and every permitted resume state.

---

## Task 2 — Make manual `call`/`try` block the caller explicitly

Purpose:  
Make the caller state truthful while preserving its parked stacks and preventing re-entry.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **local/multi-file**

Owned files and symbols:

- `phalcom-core/src/primitive/fiber.rs` — `fiber_resume`.
- `phalcom-core/src/vm/dispatch.rs` — restoration helper.

Inspect before editing:

- validation that occurs before `store_live_into`;
- first-entry `push_frame` failure behavior;
- `resume_slot` calculation.

Dependencies:
- Task 1 state variants.

Source of truth:
- `VM::current` for who runs;
- caller `FiberObject.status` for who is blocked.

Changes:

- manual `call`/`try` accept `New | Yielded`;
- reject `BlockedOnChild`, `Parked`, `Queued`, `Running`, terminal states before caller mutation;
- immediately before parking the current Fiber, transition caller `Running → BlockedOnChild`;
- preserve current `resumer` assignment and Call/Try `resume_mode`.

Must not:

- mark caller `Yielded`;
- mark caller `Parked`;
- weaken first-resume validation ordering;
- make `BlockedOnChild` manually resumable.

Current implementation:

```text
caller remains Running
store caller buffers
callee becomes Running/current
```

Target:

```text
caller Running → BlockedOnChild
store caller buffers
callee New|Yielded → Running/current
```

Edit operations:

1. OPEN `phalcom-core/src/primitive/fiber.rs`.
2. FIND `fn fiber_resume`.
3. REPLACE the `Suspended` eligibility match with explicit manual-resume eligibility.
4. KEEP arity/entry validation before mutating caller state.
5. Immediately before `store_live_into(vm, resumer_ref)`, assert current caller state is `Running`, then set it to `BlockedOnChild`.
6. Preserve `resume_slot`.
7. Set callee `Running` only after its live stack/frame state is ready.
8. Add `debug_assert_eq!(vm.current, resumer_ref)` before the switch where useful.
9. SEARCH for any other direct transition into `Running`; route restoration through the same semantic rule.

### INVESTIGATE-BEFORE-EDIT — one bounded question

Inspect whether `vm.push_frame(frame)?` can return a recoverable error *after* caller state has been parked. If yes, preserve the repository's “validate before mutation” principle by either:

- performing the remaining fallible validation before parking; or
- adding a narrow rollback for the caller/callee state.

Do **not** broaden into general frame-depth redesign.

Testing classification:
- focused hostile nested-resume regression required at C0.

---

## Task 3 — Make `yield` restore truthful caller/child state

Purpose:  
Represent explicit coroutine suspension as `Yielded` and restore only an active blocked resumer.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **local**

Owned files and symbols:

- `phalcom-core/src/primitive/fiber.rs` — `fiber_yield`.
- `phalcom-core/src/vm/dispatch.rs` — `switch_to_fiber_and_deliver`.

Source of truth:
- current Fiber state + resumer state.

Target transition:

```text
child Running → Yielded
resumer BlockedOnChild → Running
VM.current child → resumer
```

Changes:

1. In `fiber_yield`, set current status to `Yielded`, not generic suspended.
2. Before switching, require the resumer to be `BlockedOnChild` (root pump/restoration exceptions must be explained by later scheduler mode, not silently accepted).
3. In `switch_to_fiber_and_deliver`, assert target is a state that can legitimately be restored from child execution; for C0 this is primarily `BlockedOnChild`.
4. Set target `Running` only after loading its parked buffers.
5. Preserve the yielded value / resume-slot protocol exactly.

Must not:

- interpret a yielded value as a terminal outcome;
- change yielded `None` semantics;
- change `Fiber#isDone`.

Testing classification:
- existing call/yield/send fixtures at C0.

---

## Task 4 — Make root identity stable

Purpose:  
Make rootness an identity fact rather than a property of the current resumer chain.

Risk:
- Semantic: **MEDIUM**
- Implementation fanout: **local**

Owned files and symbols:

- `phalcom-core/src/heap/fiber.rs` — add stable root field or repository-equivalent.
- `phalcom-core/src/primitive/fiber.rs` — `fiber_is_root`, `fiber_yield`, `fiber_abort`.

Source of truth:
- explicit root constructor fact.

### EXACT semantic instruction

Add a scalar root identity field unless repository evidence reveals an existing stable VM root handle that is cleaner:

```rust
pub is_root: bool
```

Set:

```text
FiberObject::root()      → true
new_entry(...)           → false
new_entry_with_buffers   → false
```

Then migrate all root behavior checks from:

```rust
fiber.resumer.is_none()
```

to stable root identity.

Must not:

- infer root from `entry.is_none()` in one call site and a bool in another;
- leave `isRoot` using the old resumer heuristic.

Hostile regression:

```phalcom
const f = Fiber.new || { 1 }
Assert.falsehood(f.isRoot)
Assert.truth(Fiber.current.isRoot)
```

Testing classification:
- focused C0 regression.

---

# 9. Checkpoint C1 — Scheduler Owns Ready Admission and Scheduler Resumes

Tasks:
- Task 5 — Centralize ready-queue state transitions in VM-owned helpers.
- Task 6 — Make `System.schedule` state-aware and duplicate-safe.
- Task 7 — Add explicit scheduler resume mode/path.
- Task 8 — Route every scheduler pump through the same queue/resume contract.

## Why this is a checkpoint

C1 establishes the second ownership layer:

```text
unowned runnable Fiber
    → scheduler admission
    → Queued
    → scheduler resume
    → Running
```

Until this exists, C2 cannot safely define Future wake as `Parked → Queued`.

## Entry conditions

- C0 COMPLETE.
- `Running` means only current.
- `BlockedOnChild` exists.
- manual call/try eligibility is explicit.

## Working set

Primary:

- `phalcom-core/src/vm/mod.rs` — `ready_queue`.
- `phalcom-core/src/primitive/system.rs` — `system_schedule`, queue pop.
- `phalcom-core/src/primitive/fiber.rs` — internal scheduler resume.
- `phalcom-core/src/vm/dispatch.rs` — root-drive pump.
- `phalcom-core/core/universe/src/concurrency/fiber.ph` — `System.runScheduled` and internal native declarations.

Secondary:

- `phalcom-core/src/vm/gc.rs` — verify queue rooting remains correct.
- `docs/work/errors/E008-double-schedule-kills-run.md` — reproducer.

Out of scope:

- Future waiter changes;
- completion observers;
- E010 error-observation policy.

## Semantic contract established by C1

- Queue insertion reserves the Fiber by moving it to `Queued`.
- A Fiber cannot be admitted twice.
- A queued Fiber cannot be manually call/try resumed.
- Scheduler execution is semantically distinguishable from public `try`.
- Invalid stale queue state is skipped/contained, never allowed to kill unrelated ready work.
- FIFO ordering among valid ready Fibers is preserved.

## Semantic risks

- breaking existing `System.schedule(Fiber.new(...))` usage;
- changing FIFO order;
- making scheduled exceptions propagate differently;
- forgetting GC roots;
- leaving root-drive and `.ph` pump with different resume semantics.

## Hostile cases

- same Fiber scheduled twice, with a healthy task behind it;
- queued Fiber manually `call()`ed before pump;
- stale terminal Fiber artificially present in queue (defensive unit test);
- scheduled task schedules another task during its turn.

## Required evidence

1. Existing `concurrency_scheduler_fifo` fixture.  
   **Proves:** queue reservation did not change FIFO semantics.

2. E008-derived hostile regression.  
   **Proves:** duplicate admission is rejected/no-op at admission and cannot poison later healthy work.

3. Existing scheduled raising-fiber fixture.  
   **Proves:** scheduler resume still isolates one task's uncaught terminal failure from unrelated scheduled work.

4. Focused `cargo check -p phalcom-core`.  
   **Proves:** scheduler-mode match fanout is complete.

## Do not run yet

- E007/Future async tests;
- full workspace.

## Escalate immediately if

- preserving `System.schedule` compatibility requires permitting `Parked` Fibers;
- queue admission needs O(n) membership checks;
- scheduler error isolation can only be maintained by retaining public `try()` as the semantic owner;
- queue representation must change away from `VecDeque<ObjRef>` merely to carry a wake ticket.

## Checkpoint completion

- [ ] Tasks 5–8 complete.
- [ ] FIFO passes.
- [ ] duplicate hostile case passes.
- [ ] queued manual resume is rejected.
- [ ] scheduled-failure isolation comparator passes.
- [ ] queue remains O(1) and single-threaded.
- [ ] state file updated.
- [ ] no active incident.

### Suggested commit group

```text
refactor(concurrency): centralize ready-queue ownership transitions
fix(concurrency): reserve fibers on scheduler admission
refactor(concurrency): give scheduler resume an explicit mode
test(concurrency): prevent duplicate queue poisoning
```

---

## Task 5 — Centralize ready-queue state transitions

Purpose:  
Make queue state and Fiber state change together under VM-owned operations.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

Owned files and symbols:

- `phalcom-core/src/vm/mod.rs` — `ready_queue`.
- preferred implementation location: a small `vm` scheduler helper module if that reduces duplication; otherwise methods adjacent to current queue consumers.

Source of truth:
- `FiberObject.status` plus `VM::ready_queue`.

### STRUCTURAL API shape

Introduce VM-owned helpers equivalent to:

```rust
fn enqueue_unowned_fiber(&mut self, fiber: ObjRef) -> PhResult<()>;
fn wake_parked_fiber(&mut self, fiber: ObjRef, generation: i64) -> PhResult<bool>; // body completed in C2
fn pop_next_queued(&mut self) -> Option<ObjRef>;
```

For C1, `enqueue_unowned_fiber` must:

```text
New/Yielded → Queued + push_back
Queued      → reject duplicate
other state → reject
```

`pop_next_queued` must:

- pop FIFO;
- verify the Fiber is still `Queued`;
- defensively skip stale non-queued entries and continue;
- never run queue scanning for dedupe.

If `ready_queue` remains `VecDeque<ObjRef>`, `vm/gc.rs` should require no structural rooting change. Verify that assumption explicitly.

Edit operations:

1. FIND every direct `ready_queue.push_back` / `pop_front`.
2. Introduce one VM-owned admission/pop authority.
3. Migrate `system_schedule` and `run_until` root pump away from raw queue mutations.
4. Leave Future wake implementation stubbed/absent until C2.
5. Negative-search direct queue mutation after C1; intentional initialization/GC traversal is allowed, semantic enqueue/dequeue outside helpers is not.

Testing classification:
- no standalone test; C1 integration evidence.

---

## Task 6 — Make `System.schedule` state-aware

Purpose:  
Prevent scheduler admission from doubling as unrestricted Fiber wake.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **local + Universe declaration/docs later**

Owned files and symbols:

- `phalcom-core/src/primitive/system.rs` — `system_schedule`.

Current implementation:

```text
existing Fiber → enqueue as-is
other callable → create Fiber → enqueue
```

Target:

```text
existing Fiber:
    only unowned runnable states accepted
    reserve as Queued atomically

other callable:
    new_fiber_ref
    New → Queued atomically
```

Compatibility rule:

- preserve `System.schedule(worker)` where `worker = Fiber.new(...)` and has not run;
- permit `Yielded` only if search of existing production/tests demonstrates intentional compatibility value;
- never permit `Parked` as a public wake path.

Edit operations:

1. OPEN `system_schedule`.
2. Keep current callable validation/new-Fiber creation.
3. Replace direct `push_back` with VM admission helper.
4. Ensure duplicate scheduling errors before a second queue entry exists.
5. Add error text that distinguishes:
   - already queued;
   - parked on Future;
   - active/running;
   - terminal.
6. SEARCH current `System.schedule(` uses; verify each existing Fiber argument is `New` or intentionally `Yielded`.

Hostile case:

```phalcom
const f = Fiber.new || { ... }
System.schedule(f)
{ System.schedule(f) }.attempt() // or repository-native catch surface
System.schedule(healthy)
System.runScheduled()
```

The second admission must not poison `healthy`.

Testing classification:
- focused hostile regression at C1.

---

## Task 7 — Add explicit scheduler resume mode/path

Purpose:  
Stop pretending scheduler execution is public `Fiber#try`.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

Owned files and symbols:

- `phalcom-core/src/heap/fiber.rs` — `FiberResumeMode`.
- `phalcom-core/src/primitive/fiber.rs` — shared resume engine and new internal entry.
- `phalcom-core/core/universe/src/concurrency/fiber.ph` — internal native anchor if `.ph` pump needs it.

Source of truth:
- `resume_mode` for how a turn was initiated.

### STRUCTURAL

Extend:

```rust
pub enum FiberResumeMode {
    Call,
    Try,
    Scheduler,
}
```

Refactor shared resume code so:

```text
Call/Try:
    receiver state New|Yielded
    caller becomes BlockedOnChild

Scheduler:
    receiver state Queued
    scheduler/root driver becomes BlockedOnChild
```

Scheduler failure semantics for this program:

- isolate the scheduled Fiber terminal failure from unrelated ready work, like current `try` pump behavior;
- do not use this checkpoint to define how users observe unhandled scheduled errors (E010 stays open).

Add an internal primitive only if needed for the `.ph` pump, e.g. a source-internal selector such as:

```phalcom
@internal @native _$resumeScheduled() -> Dynamic
```

Do not expose a new public Fiber API.

Testing classification:
- existing scheduled failure comparator + C1.

---

## Task 8 — Route scheduler pumps through one contract

Purpose:  
Ensure root-drive, explicit `System.runScheduled`, and later root-await pumping cannot drift.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

Owned files and symbols:

- `VM::run_until` root-drive branch.
- `System.runScheduled`.
- queue pop/native helper.

Target logical loop:

```text
pop valid Queued Fiber
    ↓
scheduler-resume it
    ↓
return to pump on yield/park/terminal completion
    ↓
repeat
```

Changes:

1. Remove direct root-pump use of public `fiber_try`.
2. Change `.ph` `System.runScheduled` to use internal dequeue + scheduler-resume seams.
3. Preserve drain-to-exhaustion behavior including tasks enqueued by running tasks.
4. Preserve root frame/stack placeholder invariants; do not duplicate placeholder arithmetic in multiple new sites.
5. Do not yet modify `Future#await` root branch; C2 will migrate it after parking exists.

Migration note on `System.nextScheduled`:

- Do **not** finalize public API removal in C1.
- Introduce internal scheduler seams first.
- C5 will decide/remove public raw-pop authority once all production consumers have migrated.

Testing classification:
- C1 checkpoint evidence.

---

# 10. Checkpoint C2 — `await` Is an Authorized Park, Not a Coroutine Yield

Tasks:
- Task 9 — Add Fiber park generation and prepare/park primitives.
- Task 10 — Add internal ticket-aware Future wake.
- Task 11 — Rewrite Future waiter registration and `await`.
- Task 12 — Migrate root-await pump and native-boundary failure behavior.

## Why this is a checkpoint

This is the ownership correction for AUD-003.

After C2:

```text
await unresolved Future
    ↓
Parked(generation)

matching Future settles
    ↓
wake(fiber, generation)
    ↓
Queued

scheduler
    ↓
Running
```

No public Fiber operation may bypass that sequence.

## Entry conditions

- C1 COMPLETE.
- scheduler admission/resume owns `Queued`.
- public manual resume rejects `Queued`.

## Working set

Primary:

- `phalcom-core/src/heap/fiber.rs` — park generation field/state.
- `phalcom-core/src/primitive/fiber.rs` — internal park preparation/commit.
- `phalcom-core/src/primitive/system.rs` — internal wake.
- `phalcom-core/core/universe/src/concurrency/fiber.ph` — `Future#await`, waiter/drain representation, root pump.
- concurrency fixtures.

Secondary:

- native-frame guard helpers already in `primitive/fiber.rs`.

Out of scope:

- `Future.async`;
- terminal observer;
- async generator support.

## Semantic contract established by C2

- unresolved non-root await never appears to its caller as a normal yielded `None`;
- parked Fiber is not manually resumable/schedulable;
- matching Future owns the wake ticket;
- stale/duplicate wake is harmless;
- native-frame refusal happens before actionable wake authority can later advance the Fiber;
- await under manual `call`/`try` fails cleanly under D-07.

## Semantic risks

- waiter registration / park partial state;
- ticket overflow;
- wrong Future waking later park;
- root await losing progress;
- accidental native re-entry through helper closures;
- public `Fiber.yield` behavior changing.

## Hostile cases

1. Park on A; attempt `fiber.call`, `fiber.try`, `System.schedule(fiber)` before A settles.
2. After rejected wrong-authority operations, settle A; Fiber resumes exactly once.
3. Park on A, later park on B; stale A generation must not advance B.
4. Await invoked below a native re-entrant callback must not leave an actionable registration.
5. Await inside a manually resumed Fiber must fail before registration.

## Required evidence

1. Existing `concurrency_future_await_suspends.ph` or nearest current fixture.  
   **Proves:** ordinary scheduler-owned await still suspends and later resumes.

2. New wrong-authority hostile fixture.  
   **Proves:** public resume/schedule cannot steal a Future park.

3. New stale-ticket hostile fixture.  
   **Proves:** a previous Future cannot advance a later suspension.

4. Native-boundary fixture derived from existing `CannotYieldAcrossNativeFrame` coverage.  
   **Proves:** refusal leaves no actionable stale wake.

5. Manual-Fiber await rejection fixture.  
   **Proves:** D-07 is explicit rather than silently losing generator ownership.

## Do not run yet

- Future.async multi-await;
- continuation callback suspension;
- workspace.

## Escalate immediately if

- a park ticket must be stored globally rather than per Fiber;
- wake requires queue scanning;
- `_preparePark` requires invoking a Phalcom callback from native Rust;
- correct await requires retaining `Fiber.yield(None)` as the actual park event;
- manual call-chain parking appears necessary to keep existing supported behavior.

## Checkpoint completion

- [ ] Tasks 9–12 complete.
- [ ] ordinary await fixture passes.
- [ ] wrong-authority case passes.
- [ ] stale-ticket case passes.
- [ ] native refusal leaves no actionable waiter.
- [ ] manual-Fiber await policy is tested.
- [ ] no raw Fiber Future waiter remains in production Future await logic.
- [ ] state file updated.
- [ ] no active incident.

### Suggested commit group

```text
feat(concurrency): add ticketed fiber park state
fix(concurrency): make future wake authority generation-bound
refactor(concurrency): implement await as scheduler parking
test(concurrency): reject stale and wrong-authority wakes
```

---

## Task 9 — Add Fiber park generation and prepare/park primitives

Purpose:  
Create a suspension operation whose semantics are “wait for an authorized scheduler wake,” not “yield to coroutine caller.”

Risk:
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

Owned files and symbols:

- `FiberObject`;
- internal primitives in `primitive/fiber.rs`;
- internal source anchors in `fiber.ph`.

Source of truth:
- `FiberStatus::Parked(generation)`.

### STRUCTURAL API shape

Add a per-Fiber monotonic generation field, e.g.:

```rust
pub park_generation: i64
```

or the nearest integer representation that can round-trip through Phalcom `Int` without truncation.

Add internal-only operations equivalent to:

```phalcom
@internal @native _$preparePark() -> Int
@internal @native _$park(_ generation: Int) -> Dynamic
```

`_$preparePark` must validate **before Future registration**:

- receiver is `Fiber.current`;
- receiver state is `Running`;
- receiver is not root;
- receiver's current resume mode is `Scheduler`;
- native re-entry depth permits a switch.

Then allocate the next generation.

`_$park(generation)` must:

- accept only the current prepared/current generation;
- record its own resume slot;
- set `Parked(generation)`;
- park live buffers;
- restore the scheduler resumer;
- be guaranteed under valid internal use not to fail for the already-validated native-depth reason.

If the implementation does not retain an explicit “prepared” marker, stale registration remains safe only because wake validates exact `Parked(generation)`. Document this reasoning in rustdoc.

Must not:

- reuse `Fiber.yield`;
- make `Parked` accepted by `fiber_call`/`fiber_try`;
- mutate waiter lists in Rust.

Testing classification:
- C2.

---

## Task 10 — Add ticket-aware internal wake

Purpose:  
Make Future settlement the only authority that can requeue its exact parked turn.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **local/multi-file**

Owned files and symbols:

- `primitive/system.rs`;
- VM ready admission helper from C1;
- internal System source anchor.

### STRUCTURAL API

Add an internal primitive equivalent to:

```phalcom
@class
@internal
@native
_$wake(_ fiber: Fiber, _ generation: Int) -> Bool
```

Semantics:

```text
Fiber == Parked(generation):
    status = Queued
    push_back once
    return true

Fiber == Queued:
    duplicate/stale wake
    return false

Fiber == Parked(other_generation):
    stale wake
    return false

any other state:
    stale/not-waiting
    return false
```

The no-op cases are intentional for stale waiter cleanup; they are not ordinary programmer-facing errors.

Keep transition + enqueue atomic within one VM-thread operation.

Testing classification:
- C2 hostile cases.

---

## Task 11 — Rewrite Future waiter registration and `await`

Purpose:  
Remove raw-Fiber waiter authority and ordinary coroutine yield from Future waiting.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **local `.ph` library**

Owned file:

- `phalcom-core/core/universe/src/concurrency/fiber.ph` — `Future` fields, `drain`, `await`.

Inspect before editing:

- List private push/iteration surface;
- current continuation waiter representation;
- exact native/internal visibility available in Universe source.

Source of truth:
- ticketed Fiber park state.

### STRUCTURAL waiter representation

Do not store a raw Fiber as an await waiter.

Use the smallest repository-native representation that owns both Fiber and generation without exposing a new public language type. Preferred options in order:

1. an existing private/product representation already used in Universe internals;
2. a dedicated internal-only value if the repository already supports such source-private helpers;
3. separate awaiter/continuation collections if that is simpler than preserving the heterogeneous `_waiters` list.

Do not create a public `FutureWaiter` class merely for this patch unless source visibility machinery proves it can remain internal.

Target `await` shape:

```phalcom
await {
  if not self.isReady {
    if Fiber.current.isRoot {
      // root pump, Task 12
    } else {
      const current = Fiber.current
      const generation = current._$preparePark()
      self._registerAwaiter(current, generation)
      current._$park(generation)
    }
  }

  // Wake is authority to run again, not proof the condition is true.
  if not self.isReady {
    // repeat park path / loop in repository-supported direct style
  }

  if rejected { raise }
  return value
}
```

The exact loop syntax must use existing Phalcom control flow and must not wrap `_park` inside a native-driven block that violates the re-entry guard.

`drain` must:

- wake awaiters via `System._$wake(fiber,generation)`;
- continue scheduling continuation closures as work;
- clear consumed waiter storage.

Must not:
- `System.schedule(rawAwaiterFiber)`;
- `Fiber.yield(None)` for await.

Testing classification:
- C2.

---

## Task 12 — Migrate root-await pump and native refusal

Purpose:  
Make root await use the C1 scheduler resume path and ensure non-root registration never gains stale authority after a native-frame refusal.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **local/multi-file**

Owned symbols:

- `Future#await` root branch;
- internal scheduler dequeue/resume seam;
- park prepare guard.

Changes:

1. Replace root `nextScheduled → f.try()` with C1 scheduler-mode resume.
2. Preserve quiescence behavior:
   - while target Future pending;
   - if no valid queued work remains;
   - raise the existing “scheduler empty / nothing can settle it” error.
3. Ensure a forbidden native suspension fails in `_$preparePark` before registration.
4. If registration can occur before final commit for mechanical reasons, prove stale registration cannot act because exact state/generation validation fails.
5. Add comments explaining why wake != readiness and why root uses pump rather than park.

Testing classification:
- C2.

---

# 11. Checkpoint C3 — Durable Terminal Completion Independent of `resumer`

Tasks:
- Task 13 — Add a single GC-traced completion observer to `FiberObject`.
- Task 14 — Add internal observer binding and terminal-result access.
- Task 15 — Centralize terminal success/failure notification.
- Task 16 — Cover call-mode cascades and GC lifecycle.

## Why this is a checkpoint

This is the architectural repair that makes `Future.async` possible.

C2 can suspend/resume correctly, but without C3 no stable object observes the Fiber's final terminal value after scheduler resumption changes `resumer`.

C3 establishes:

```text
dynamic resumer
    = immediate control recipient

completion observer
    = durable terminal recipient
```

They are intentionally distinct.

## Entry conditions

- C2 COMPLETE.
- scheduler wake can resume a parked Fiber any number of times.
- terminal state remains `Done`/`Failed` + `result`.

## Working set

Primary:

- `heap/fiber.rs`;
- `heap/trace.rs`;
- `primitive/fiber.rs`;
- `vm/dispatch.rs`;
- VM scheduler helper from C1.

Secondary:

- `vm/gc.rs` only if observer enqueue changes queue representation.

Out of scope:

- Future rewrite until C4;
- Task abstraction;
- cancellation.

## Semantic contract established by C3

- observer can be bound before execution;
- at most one observer is attached;
- yield/park do not notify it;
- terminal success/failure notify it exactly once;
- scheduler/resumer changes cannot disconnect it;
- Call failure cascade notifies every observer-bearing Fiber it terminally marks failed;
- observer is retained while attached and released after handoff.

## Semantic risks

- double notification;
- missing notification on cascade;
- observer GC collection while action parked;
- observer retaining terminal Fiber forever;
- running observer inline and re-entering dispatch;
- observer scheduling recursively corrupting root pump.

## Hostile cases

- action parks three times before returning;
- action parks then fails;
- Fiber with observer is failed indirectly by `child.call()`;
- force GC while observed Fiber is parked, then settle dependency.

## Required evidence

1. Focused internal/fixture observer-success test.  
   **Proves:** terminal value can be observed after scheduler resume.

2. Terminal failure test.  
   **Proves:** failure path triggers observer once.

3. Multi-park observer test.  
   **Proves:** resumer churn does not detach completion ownership.

4. Call-cascade hostile test.  
   **Proves:** observer on an intermediate Fiber terminally failed by Call cascade fires.

5. GC stress/forced-GC fixture.  
   **Proves:** attached observer and captured result Future survive while action is parked.

## Do not run yet

- full Future continuation matrix;
- workspace.

## Escalate immediately if

- the VM must know the `Future` class or call `settleValue` directly;
- observer execution must occur inline to obtain the terminal result;
- adding one observer requires a second execution stack;
- Call cascade cannot identify terminalized Fibers individually.

## Checkpoint completion

- [ ] Tasks 13–16 complete.
- [ ] observer success/failure evidence passes.
- [ ] multi-park observer survives.
- [ ] call-cascade observer passes.
- [ ] GC test passes.
- [ ] no observer callback executes inline at the Fiber floor.
- [ ] state file updated.
- [ ] no active incident.

### Suggested commit group

```text
feat(concurrency): add durable fiber completion observers
refactor(concurrency): centralize terminal observer handoff
fix(concurrency): notify observers through call-mode failure cascade
test(concurrency): preserve completion ownership across park and gc
```

---

## Task 13 — Add GC-traced completion observer field

Purpose:  
Persist logical terminal ownership independently of dynamic `resumer`.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

Owned files:

- `phalcom-core/src/heap/fiber.rs`;
- `phalcom-core/src/heap/trace.rs`.

Source of truth:
- one optional Fiber-owned observer handle.

### STRUCTURAL

Add:

```rust
pub completion_observer: Option<ObjRef>
```

or the exact existing callable handle type required by the repository.

Initialize to `None` for:

- root;
- new entry;
- pooled entry.

In `heap/trace.rs`, trace the observer when present.

Do not:

- store a raw Rust closure;
- store a direct `Future`-specific pointer;
- keep the observer attached after terminal handoff.

Testing classification:
- no standalone test; C3 GC and lifecycle evidence.

---

## Task 14 — Add observer binding and terminal-result access

Purpose:  
Allow pure-Phalcom Future code to bind terminal behavior without making the VM Future-aware.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

Owned files/symbols:

- `primitive/fiber.rs` — new internal primitives.
- `fiber.ph` — internal native anchors.

### STRUCTURAL internal surface

Add internal selectors equivalent to:

```phalcom
@internal @native _$onComplete(_ observer: Dynamic) -> Fiber
@internal @native _$terminalValue -> Dynamic
```

Binding requirements:

- validate observer as Block/Closure using the same callable validation strategy as `Fiber.new`;
- require no observer is already bound;
- require Fiber has not terminally completed;
- prefer binding while Fiber is `New` before scheduling;
- return receiver for internal chaining if consistent with native API style.

Terminal value getter:

- valid only after `Done` or `Failed`;
- returns `FiberObject.result`;
- Future observer should use `fiber.error` to distinguish terminal failure from successful return of an `Error` object.

Must not:
- infer failure from result's class/type.

Testing classification:
- C3/C4.

---

## Task 15 — Centralize terminal success/failure notification

Purpose:  
Guarantee exactly-once observer handoff in every terminal path.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

Owned file:

- `phalcom-core/src/vm/dispatch.rs`.

Inspect before editing:

- non-root success branch in `run_until`;
- failure loop and upvalue-close ordering;
- ready-queue helper from C1;
- `switch_to_fiber_and_deliver`.

Source of truth:
- terminal state transition.

### STRUCTURAL helper

Extract narrow helpers equivalent to:

```rust
fn mark_fiber_done(&mut self, fiber: ObjRef, value: Value);
fn mark_fiber_failed(&mut self, fiber: ObjRef, error: Value);
fn enqueue_completion_observer(&mut self, fiber: ObjRef);
```

The exact factoring may differ, but terminal notification must be centralized enough that a new terminal path cannot forget it.

Success ordering:

```text
close/finish normal activation
set Done
store result
take observer
enqueue observer as fresh scheduler work
restore resumer
```

Failure ordering:

```text
capture error
close open upvalues BEFORE discard
for each Fiber terminally failed:
    set Failed
    store same error value
    take/enqueue observer
    clear parked terminal retention
    continue Call cascade or stop at Try/root
```

Observer enqueue:

- use the C1 new-work scheduler admission path;
- create fresh observer Fiber;
- never execute observer bytecode inline.

Testing classification:
- C3.

---

## Task 16 — Cover cascade and GC lifecycle

Purpose:  
Defeat two easy incomplete implementations: success-only observer notification and non-traced observer storage.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **tests + possible GC fix**

Owned tests:

- nearest focused core/VM test location;
- `phalcom-core/tests/fixtures/language/concurrency/` for full-Universe behavior;
- existing GC stress infrastructure if it can force `System.gc`.

Required hostile shapes:

### Call cascade

```text
scheduler-owned outer with completion observer
    → inner.call()
        → uncaught raise
outer becomes Failed via cascade
observer must run exactly once
```

### GC

```text
bind observer capturing result Future
schedule action
action parks
drop other observer references
System.gc
settle dependency
observer still settles result
```

Testing classification:
- focused regression required at C3.

---

# 12. Checkpoint C4 — Futures Adopt Terminal Computation Outcomes

Tasks:
- Task 17 — Add one private Future helper that runs a callable to terminal completion.
- Task 18 — Rewrite `Future.async` without the one-turn driver assumption.
- Task 19 — Rewrite pending `then`/`map`/`catch` through the same terminal adapter.
- Task 20 — Add the adversarial async/continuation regression set.

## Why this is a checkpoint

C4 removes the duplicated library code that caused AUD-001 and AUD-002.

The Fiber/scheduler substrate is already correct by C3. Future becomes a consumer of terminal completion rather than an improvised Fiber driver.

## Entry conditions

- C3 COMPLETE.
- observer can read terminal outcome.
- action can park/wake repeatedly without losing observer.

## Working set

Primary:

- `phalcom-core/core/universe/src/concurrency/fiber.ph`;
- concurrency language fixtures;
- E007 documentation for comparator behavior.

Secondary:

- no Rust runtime changes expected unless C4 exposes a missing internal surface.

Out of scope:

- redesigning Future public API;
- cancellation;
- new task abstraction.

## Semantic contract established by C4

- `Future.async(action)` settles only when `action` reaches `Done`/`Failed`.
- Await suspension never settles the outer Future.
- Multiple awaits are transparent.
- Late failure rejects.
- pending continuation callback suspension remains pending.
- continuation result Future adoption/flattening occurs only after the callback's terminal success.
- successful callback returning `Error` is still success unless raised.

## Semantic risks

- accidental double settlement between observer and returned nested Future;
- continuation immediate and pending paths diverging;
- flattening temporary turn values;
- observer callback itself suspending unexpectedly;
- changing settle-once behavior.

## Hostile cases

- three sequential awaits then terminal value;
- await then raise;
- callback awaits then returns `None` intentionally;
- callback awaits then returns `Error` as data;
- callback awaits then returns another Future;
- callback awaits then raises;
- already-settled receiver path compared with pending receiver path.

## Required evidence

1. E007 exact regression: outer Future remains pending after first parked turn and eventually gets real terminal value.
2. Multi-await action.
3. Late failure.
4. Pending `then`, `map`, `catch` each with a suspending callback on the applicable branch.
5. Nested Future/flatten continuation case.
6. Existing `concurrency_future_slice_b` and immediate `Future.async` fixtures as compatibility comparators.

## Do not run yet

- workspace;
- LSP/type crates.

## Escalate immediately if

- Future helper needs direct VM access;
- observer cannot settle a Future from ordinary scheduled `.ph` code;
- fixing pending continuations requires changing immediate settled continuation semantics;
- a callback result must be inspected before terminalization.

## Checkpoint completion

- [ ] Tasks 17–20 complete.
- [ ] E007 regression passes.
- [ ] multi-await passes.
- [ ] late failure rejects.
- [ ] suspending then/map/catch matrix passes.
- [ ] nested Future flatten case passes.
- [ ] existing immediate Future cases pass.
- [ ] one-turn driver pattern removed from Future production code.
- [ ] state file updated.
- [ ] no active incident.

### Suggested commit group

```text
refactor(concurrency): run future actions through terminal completion binding
fix(concurrency): settle Future.async only on terminal action outcome
fix(concurrency): make pending future continuations suspension-safe
test(concurrency): cover multi-await and suspending continuations
```

---

## Task 17 — Add one private terminal-computation helper in Future

Purpose:  
Make one `.ph` abstraction responsible for “run callable in a Fiber and react to terminal outcome.”

Risk:
- Semantic: **HIGH**
- Implementation fanout: **local**

Owned symbol:

- new private/class-private helper near `Future.async`, `flatten`, continuations.

Source of truth:
- Fiber terminal observer.

### STRUCTURAL shape

Implement a helper equivalent to:

```phalcom
@private
@class
runToTerminal(_ action, _ onSuccess, _ onError) {
  const fiber = Fiber.new(action)

  fiber._$onComplete(|| {
    if fiber.error.isSome {
      onError.call(fiber.error.unwrapOr(None))
    } else {
      onSuccess.call(fiber._$terminalValue)
    }
  })

  System.schedule(fiber)
  fiber
}
```

Do not paste this blindly if private method/callback syntax differs; preserve current repository-native closure calling conventions and avoid native-frame suspension traps.

The helper's callbacks should be simple settlement/adoption actions. The action Fiber itself is the computation.

Must not:
- call `fiber.try()` once and inspect its return;
- use `fiber.isDone` as a polling completion mechanism;
- expose the action Fiber publicly through Future API.

Testing classification:
- C4.

---

## Task 18 — Rewrite `Future.async`

Purpose:  
Remove the driver Fiber and settle from terminal action outcome.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **local**

Current:

```text
driver Fiber
    creates action Fiber
    action.try() once
    settles result from turn return
```

Target:

```text
result Future
action Fiber
completion binding action → result Future
schedule action
return result Future
```

Success:

```text
Done(value) → result.settleValue(value)
```

Failure:

```text
Failed(error) → result.settleError(error)
```

Nonterminal states:

```text
Yielded/Parked/Queued/Running → no settlement
```

Preserve current `Future.async` behavior with respect to a terminal return that happens to be another Future; do not add flattening unless current specification explicitly requires it.

Negative search after Task 18:

```bash
rg 'const res = fib\.try\(\)' phalcom-core/core/universe/src/concurrency/fiber.ph
```

Expected after C4: no `Future.async` or continuation production hit.

Testing classification:
- E007 and multi-await at C4.

---

## Task 19 — Rewrite pending `then` / `map` / `catch`

Purpose:  
Eliminate three copies of the same one-turn Fiber bug.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **local**

Owned symbols:

- `Future#then`;
- `Future#map`;
- `Future#catch`;
- `Future.flatten`.

Target pending callback flow:

```text
source Future settles
    ↓
schedule continuation
    ↓
terminal-run callback Fiber
    ↓
callback terminal success
    ↓
Future.flatten(actual terminal callback result)
    ↓
adopt flattened Future into derived Future

callback terminal failure
    ↓
derived Future rejected
```

Preserve branch semantics:

- `then` / `map`: source rejection passes through as rejection.
- `catch`: source fulfillment passes through as fulfillment.
- already-settled receiver behavior should remain semantically equivalent.

Refactor duplication:

- use one private helper for adopting a terminal callback result into `f_next`;
- do not maintain three separate `Fiber.new → try → error/isReady` mini-drivers.

Testing classification:
- C4 continuation matrix.

---

## Task 20 — Add adversarial Future regressions

Purpose:  
Make the repair resistant to the exact easy-but-wrong patches identified by the audit.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **tests**

Preferred ownership layer:

- `phalcom-core/tests/fixtures/language/concurrency/`.

Reuse/extend existing fixtures where a single fixture can prove a coherent law. Do not create a dozen one-assertion files merely to mirror tasks.

Minimum semantic fixtures:

### A. `Future.async` multi-suspension

Suggested name:

```text
concurrency_future_async_multi_await_terminal_completion.ph
```

Proves:

- result not ready after each individual dependency turn;
- terminal result is correct only after final dependency;
- no `Some(None)` missettlement.

### B. Late failure

```text
concurrency_future_async_late_failure.ph
```

Proves:
- parking is nonterminal;
- post-wake uncaught raise rejects outer Future.

### C. Suspending continuations

One fixture may cover:

```text
then
map
catch
callback returns nested Future
callback raises after await
```

### D. Value hostility

Ensure at least one successful terminal callback returns:

```text
None
Error.new("data")
```

without being mistaken for suspension/failure.

Verification:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus concurrency_future
```

First run `-- --list` and confirm the filter selects the intended fixtures.

---

# 13. Checkpoint C5 — Ownership Closure, Compatibility Cleanup, and Specification Alignment

Tasks:
- Task 21 — Remove/de-internalize raw scheduler authority that competes with C1/C2.
- Task 22 — Convert audit probes/open errors into permanent regression evidence.
- Task 23 — Update canonical specs/ADR/floor census to the repaired semantics.
- Task 24 — Run the full `phalcom-core` delivery gate and negative searches.

## Why this is a checkpoint

A migration is incomplete if both the new authority and old shortcuts remain usable.

C5 proves:

```text
replacement works
AND
obsolete competing authority cannot silently execute
```

It also aligns documentation only after behavior is proven.

## Entry conditions

- C4 COMPLETE.
- no active semantic incident.

## Working set

Primary:

- changed `phalcom-core` runtime/library files;
- `docs/spec/current/concurrency.md`;
- `docs/spec/current/system.md`;
- `docs/spec/current/core/floor-census.md`;
- ADR-0030;
- E007/E008;
- concurrency fixtures.

Secondary:

- docs mentioning `System.nextScheduled` only where public-surface migration requires factual correction.

Out of scope:

- E010 resolution;
- reactor design;
- unrelated app documentation prose unless it becomes factually false due an intentional public API removal.

## Semantic contract established by C5

- Future no longer parks with public `Fiber.yield`.
- Future no longer wakes by public `System.schedule(rawFiber)`.
- scheduler execution no longer depends on public `Fiber.try`.
- terminal completion no longer depends on dynamic `resumer`.
- canonical spec describes explicit park/wake/completion ownership.
- audit defects have permanent regressions.

## Required evidence

1. Negative searches listed below.
2. Focused complete concurrency corpus.
3. `RUSTFLAGS='' cargo test -p phalcom-core --test core`.
4. `RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus`.
5. `RUSTFLAGS='' cargo test -p phalcom-core` if repository baseline is green and duration is acceptable.

## Do not run yet

- workspace build/test/clippy until Final Gate.

## Escalate immediately if

- an old public scheduler seam has real production consumers that cannot migrate without a new public API decision;
- canonical spec contradicts D-07 or D-10;
- a reactor draft depends on raw queue-pop semantics as a normative commitment rather than analogy.

## Checkpoint completion

- [ ] Tasks 21–24 complete.
- [ ] negative searches satisfy expected results.
- [ ] concurrency corpus passes.
- [ ] core integration target passes.
- [ ] language corpus passes.
- [ ] relevant docs are aligned.
- [ ] E007/E008 status updated only with passing evidence.
- [ ] E010 explicitly remains open if still unfixed.
- [ ] state file updated.
- [ ] no active incident.

### Suggested commit group

```text
refactor(concurrency): close raw scheduler ownership seams
test(concurrency): promote concurrency audit probes to regressions
docs(concurrency): specify park wake and terminal ownership
```

---

## Task 21 — Close raw scheduler authority

Purpose:  
Ensure users/library code cannot bypass queue reservation or resume a Future-owned park.

Risk:
- Semantic: **HIGH**
- Implementation fanout: **multi-file + docs**

Owned symbols:

- `System.nextScheduled`;
- internal dequeue/scheduler-resume selectors introduced in C1;
- `System.schedule`.

Decision:

- internal scheduler pumps must no longer consume a public raw queue-pop seam;
- if `System.nextScheduled` has no supported external semantic requirement beyond scheduler implementation, migrate it to an `@internal` selector or remove it from the public source surface;
- if compatibility must temporarily retain it, document exactly what ownership transition occurs when it pops a Fiber and prove it cannot expose a `Parked`/`Queued` Fiber that public `call/try` can steal.

Preferred outcome:

```text
public:
    System.schedule
    System.runScheduled

internal:
    dequeue next ready
    scheduler resume
    wake parked Fiber
```

Before removing/renaming, run:

```bash
rg 'nextScheduled' --glob '!target/**'
```

Classify every production/spec/document occurrence. Do not blindly edit historical analysis files.

Testing classification:
- C5 migration gate.

---

## Task 22 — Promote audit probes and open errors to permanent regressions

Purpose:  
Turn the audit into ownership-law coverage rather than temporary reproducer scripts.

Risk:
- Semantic: **MEDIUM**
- Implementation fanout: **tests/docs**

Minimum mapping:

| Audit defect | Permanent evidence |
|---|---|
| AUD-001 / E007 | multi-await `Future.async` terminal settlement |
| AUD-002 | suspending pending continuation callbacks |
| AUD-003 | wrong-authority + stale-ticket wait cases |
| AUD-004 / E008 | duplicate scheduler admission cannot poison queue |
| AUD-005 | truthful running/blocked state invariant |
| AUD-006 | fresh Fiber is not root |

Do not add tests that merely inspect private enum spelling. Test executable invariants.

Update E007/E008 status to FIXED only after the owning regression passes at C5.

Do not mark E010 fixed unless separately reproduced and resolved.

---

## Task 23 — Update specs and ADR

Purpose:  
Remove architectural claims disproved by the audit and document the corrected ownership model.

Risk:
- Semantic: **MEDIUM**
- Implementation fanout: **docs**

Primary documents:

- `docs/spec/current/concurrency.md`;
- `docs/spec/current/system.md`;
- `docs/spec/current/core/floor-census.md`;
- `docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md`.

Required documentation changes:

1. Fiber states distinguish:
   - manual yield;
   - child-blocked;
   - Future park;
   - queue reservation.

2. `resumer` is explicitly:
   - dynamic immediate control-transfer recipient;
   - **not** durable async completion ownership.

3. Future await:
   - root pumps scheduler;
   - scheduler-owned Fiber parks with wake authority;
   - manual Fiber call-chain await limitation D-07 is explicit.

4. Scheduler:
   - queue admission reserves work;
   - public scheduling is not Future wake;
   - internal wake validates exact park generation.

5. Completion:
   - terminal observer is independent of `resumer`;
   - `Future.async` settles on terminal outcome only.

6. Failure:
   - describe current Call cascade accurately;
   - do not say intermediate handlers execute if they do not.

7. Floor census:
   - add/remove internal native seams exactly as implemented.

ADR handling:

- amend ADR-0030 rather than pretending its old “`resumer + result` are general enough for Future” forward-compat claim remains true;
- preserve its accepted decisions on heap-resident Fibers, restricted native re-entry, O(1) switching, and single-thread cooperative execution.

Testing classification:
- no behavioral test for prose; validated by source/spec negative searches and C5 behavior.

---

## Task 24 — C5 broad package and deletion gates

Purpose:  
Prove the entire `phalcom-core` package remains coherent after ownership migration.

Risk:
- Semantic: **LOW implementation / HIGH evidence**
- Implementation fanout: **verification only unless incident**

Run serially, smallest first:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus concurrency
RUSTFLAGS='' cargo test -p phalcom-core --test core
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus
RUSTFLAGS='' cargo test -p phalcom-core
```

Confirm filters select tests.

Negative searches:

```bash
rg 'Fiber\.yield\(None\)' phalcom-core/core/universe/src/concurrency/fiber.ph
```

Expected:
- zero hits in `Future#await`.

```bash
rg 'const res = fib\.try\(\)' phalcom-core/core/universe/src/concurrency/fiber.ph
```

Expected:
- zero production hits in `Future.async`, `then`, `map`, `catch`.

```bash
rg 'ready_queue\.(push_back|pop_front)' phalcom-core/src
```

Expected:
- only the centralized scheduler-owner implementation; no independent semantic mutations in `primitive/system.rs` and `vm/dispatch.rs`.

```bash
rg 'resumer\.is_none\(\)' phalcom-core/src/primitive/fiber.rs
```

Expected:
- zero root-identity checks.

```bash
rg 'System\.schedule\(.*Fiber|System\.schedule\(w\)' phalcom-core/core/universe/src/concurrency/fiber.ph
```

Interpret manually; expected:
- no Future await wake path scheduling a raw waiter Fiber.

```bash
rg 'FiberStatus::Suspended' phalcom-core/src
```

Expected:
- zero production hits if the variant is removed.

Do not weaken any failing fixture. Use the failure protocol below.

---

# 14. Patch-Level State Transition Specification

The implementing agent should keep this table beside the code while editing.

| From | Operation | Required authority | To | Immediate recipient |
|---|---|---|---|---|
| `New` | `call` | manual caller | `Running` | callee |
| `New` | `try` | manual caller | `Running` | callee |
| `New` | `System.schedule` | scheduler admission | `Queued` | none yet |
| `Yielded` | `call`/`try` | manual caller | `Running` | callee |
| `Yielded` | `System.schedule` | explicit compatibility, only if ratified | `Queued` | none yet |
| `Queued` | internal scheduler resume | scheduler | `Running` | callee |
| caller `Running` | resumes child | coroutine/scheduler | `BlockedOnChild` | child |
| `Running` | `Fiber.yield(v)` | coroutine | `Yielded` | dynamic resumer |
| scheduler `Running` | `Future.await` unresolved | park generation | `Parked(g)` | scheduler resumer |
| `Parked(g)` | matching wake(g) | Future waiter | `Queued` | none yet |
| `Parked(g2)` | stale wake(g1) | stale | unchanged | none |
| `Queued` | duplicate wake/schedule | none valid | unchanged/reject | none |
| `Running` | normal return | VM terminalization | `Done` | resumer + completion observer scheduled |
| `Running` | uncaught failure under `Try`/Scheduler | VM terminalization | `Failed` | resumer + completion observer scheduled |
| `Running`/`BlockedOnChild` | Call cascade | VM terminalization | `Failed` | next resumer or Try boundary + observer |
| `Done`/`Failed` | any resume | none | terminal unchanged | error/reject |

Global laws:

```text
count(status == Running) == 1
VM.current.status == Running
BlockedOnChild is never accepted by call/try/schedule/wake
Parked is resumed only by matching wake → scheduler
Queued is resumed only by scheduler
Done/Failed never resume
```

---

# 15. Testing Architecture

Use the repository's existing split rather than duplicating semantic assertions.

## Ownership-layer tests

Prefer lower-level Rust/unit/core tests for:

- constructor/root state;
- queue admission state transitions;
- park ticket matching;
- stale queue defensive skip;
- completion-observer field tracing/cleanup where direct heap inspection is valuable.

If integration-test privacy prevents direct state inspection, place narrowly scoped `#[cfg(test)]` tests beside the owning Rust module instead of adding a public test-only Fiber status API.

Do **not** add public reflection solely for testing private scheduler state.

## Production-faithful language corpus

Use `phalcom-core/tests/fixtures/language/concurrency/` for:

- manual call/yield compatibility;
- root await;
- wrong-authority observable behavior;
- Future async;
- continuations;
- failure propagation;
- native-boundary behavior.

The runner auto-discovers fixture files; do not invent a parallel harness.

## Smallest-first command policy

Before using a filter:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus -- --list
```

Then:

```text
exact regression
    ↓
concurrency-filtered language corpus
    ↓
core integration target
    ↓
full language corpus
    ↓
phalcom-core package
    ↓
workspace Final Gate
```

---

# 16. Failure / Incident Protocol

If required checkpoint evidence fails unexpectedly, mark the checkpoint:

```text
C<N> — INCIDENT
```

Do not build later checkpoints on it.

Record:

## Exact reproduction

```text
command:
test:
key output:
```

## Direct path

Example:

```text
fixture
→ Future.await
→ Fiber._$preparePark
→ waiter registration
→ Fiber._$park
→ System._$wake
→ ready_queue
→ scheduler resume
→ failing assertion
```

## Passing comparator

Find a nearby behavior that still works:

```text
manual yield works, ticketed park fails
```

or:

```text
single await works, second-generation stale wake fails
```

## Classification

Choose:

```text
PRODUCT
FIXTURE
DEPENDENCY/PUBLICATION
BACKEND/HARNESS
BASELINE
PLAN DRIFT
```

## Narrow repair boundary

State exact files/symbols allowed to change.

## Rejected broad fixes

For this program, always reject without new evidence:

- parser changes;
- type-system changes;
- Task/TaskGroup addition;
- global queue scans;
- restoring raw Fiber waiter/schedule shortcuts;
- turning native guard failures into ignored success;
- weakening assertions.

Resume implementation only after this incident note exists.

---

# 17. Repository Drift Protocol

Before each checkpoint:

```bash
git status --short
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
```

Then:

1. verify primary files still exist;
2. verify the named primary symbols retain the same responsibility;
3. inspect changes produced by prior checkpoints;
4. run targeted `rg` for new consumers if a signature/state enum changed;
5. adapt mechanics only.

If current HEAD materially differs from `b84da68f...` in the concurrency files:

- inspect the relevant diff/commits;
- update exact code anchors;
- do not silently alter D-01 through D-11.

If the semantic design is contradicted, stop with `PLAN DRIFT`.

---

# 18. Implementation State File Protocol

Create or reuse one concise state document for this program. If no repository convention has already been established by the implementer, use:

```text
        docs/implementation/CONC002-concurrency-control-and-failure-observability/C1-concurrency-control-and-failure-observability/concurrency-control-implementation-state.md
```

After each checkpoint record:

```md
# Concurrency Control Implementation State

## Repository state
- branch:
- HEAD:
- relevant local changes preserved:

## Established invariants
- I-01: ...
- I-02: ...

## Decisions
- D-01: ...
- ...

## Evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|

## Negative/deletion gates
- search:
- expected:
- result:

## Deferred gates
- command → destination checkpoint/final gate

## Unexpected findings
- ...

## Active incident
None.

## Next resume action
Begin C<N> Task <M>.
```

Do not store chain-of-thought or verbose diaries.

At each completion the supervisor report should be:

```text
Checkpoint C<N> COMPLETE

Established:
    <semantic contract>

Changed:
    <file> — <responsibility>

Evidence:
    <command> — PASS

Hostile cases:
    <case> — PASS

Negative gates:
    <search> — expected result

Deferred:
    <gate> → <destination>

Unexpected findings:
    none / concise fact

Next:
    C<N+1> — ...
```

---

# 19. Final Checkpoint Evidence Summary Template

Populate during execution; do not pre-mark as passed.

| Checkpoint | Semantic contract | Evidence | Status |
|---|---|---|---|
| C0 | truthful Fiber lifecycle + stable root identity | focused Fiber/manual coroutine tests | PENDING |
| C1 | scheduler reservation + scheduler resume ownership | FIFO + duplicate/stale queue hostile cases | PENDING |
| C2 | ticketed await park/wake ownership | wrong-authority + stale-ticket + native refusal | PENDING |
| C3 | durable terminal observer | success/failure/multi-park/cascade/GC | PENDING |
| C4 | terminal Future settlement | E007 + multi-await + suspending continuations | PENDING |
| C5 | old authority removed + specs aligned | negative searches + full `phalcom-core` | PENDING |

---

# 20. Final Broad Gates

Follow repository guidance and run serially.

```bash
cargo fmt --all -- --check
```

**Proves:** Rust formatting consistency.  
**Does not prove:** concurrency semantics.

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets
```

**Proves:** all workspace targets compile with the migrated runtime API/state model.

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets
```

**Proves:** cross-workspace compatibility and no broad regression detectable by existing tests.

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
```

**Proves:** lint-clean delivery under repository policy.

Do not claim these broad gates prove the semantic laws; those are established by checkpoint evidence.

If a broad gate fails outside `phalcom-core`, classify whether it is:

```text
real cross-crate regression
or
pre-existing baseline
```

before editing outside the planned working set.

---

# 21. Final Negative / Deletion Gates

At delivery, rerun and record:

```bash
rg 'FiberStatus::Suspended' phalcom-core/src
rg 'Fiber\.yield\(None\)' phalcom-core/core/universe/src/concurrency/fiber.ph
rg 'const res = fib\.try\(\)' phalcom-core/core/universe/src/concurrency/fiber.ph
rg 'ready_queue\.(push_back|pop_front)' phalcom-core/src
rg 'resumer\.is_none\(\)' phalcom-core/src/primitive/fiber.rs
rg 'nextScheduled' phalcom-core/core/universe/src/concurrency/fiber.ph docs/spec/current
```

Expected:

- no old generic `Suspended` production state;
- no `Future#await` implemented by `Fiber.yield(None)`;
- no one-turn Fiber driver in Future async/continuations;
- raw queue mutation centralized;
- no root identity inferred from `resumer`;
- every remaining `nextScheduled` occurrence is intentional, internalized, or specifically justified.

Also search new internal selectors and ensure they are not exposed without `@internal`.

---

# 22. Deferred-Evidence Audit

Before release-complete status, every deferred gate must be one of:

1. executed successfully;
2. explicitly removed from scope with a concrete reason;
3. recorded as a known release blocker.

There must be no “we did not get to it” test entry.

---

# 23. Suggested Staged Commit Groups

Do not force one commit per task. Suggested coherent groups:

## C0

```text
refactor(concurrency): make fiber lifecycle states truthful
fix(concurrency): make root identity stable
test(concurrency): lock manual coroutine lifecycle
```

## C1

```text
refactor(concurrency): centralize scheduler ownership
fix(concurrency): reserve fibers on queue admission
test(concurrency): prevent duplicate queue poisoning
```

## C2

```text
feat(concurrency): add ticketed async fiber parking
fix(concurrency): bind future wakes to exact park generation
test(concurrency): reject stale and wrong-authority resumes
```

## C3

```text
feat(concurrency): add durable terminal completion observers
fix(concurrency): notify completion through failure cascades
test(concurrency): retain completion observers across gc and parking
```

## C4

```text
refactor(concurrency): centralize terminal future computation
fix(concurrency): make async and continuations suspension-safe
test(concurrency): cover multi-await and late terminal outcomes
```

## C5

```text
refactor(concurrency): close obsolete scheduler seams
docs(concurrency): specify park wake and completion ownership
test(concurrency): promote audit probes to permanent regressions
```

---

# 24. Known Scope Exclusions

Do not silently include:

- `Task` or `TaskGroup`;
- structured concurrency;
- cancellation;
- target-Fiber kill;
- reactor or I/O implementation;
- timers;
- multithreading;
- parallelism;
- preemption;
- scheduler fairness policy;
- public scheduler-yield API;
- async generator support / transparent manual Fiber-chain parking;
- function coloring or an async function type;
- changing `Fiber.abort` semantics;
- redesigning Call-mode exception semantics;
- solving E010 scheduler error observation;
- Future state representation optimization (e.g. string → enum) unless mechanically necessary;
- performance work beyond preserving O(1) control operations and avoiding new redundant Fiber allocation where practical.

If one is required to satisfy a checkpoint invariant, stop and escalate before implementing it.

---

# 25. Release-Complete Criteria

The concurrency-control remediation is complete only when:

- [ ] C0 through C5 are all `COMPLETE`.
- [ ] `Running` uniquely identifies `VM.current`.
- [ ] active parent Fibers are explicitly blocked and non-resumable.
- [ ] root identity is stable.
- [ ] scheduler admission prevents duplicate ownership.
- [ ] `Future#await` no longer uses ordinary `Fiber.yield(None)` as parking.
- [ ] every Future park has exact wake authority.
- [ ] stale and duplicate wakes are harmless.
- [ ] parked/queued Fibers cannot be stolen by public `call`, `try`, or `schedule`.
- [ ] scheduler resume is explicit and not semantically inferred from public `try`.
- [ ] completion ownership is independent of `resumer`.
- [ ] completion observers are traced, exactly-once, and cleared after handoff.
- [ ] completion observers fire on both direct and Call-cascade terminal failure.
- [ ] `Future.async` survives multiple awaits and settles with terminal outcome.
- [ ] pending `then`/`map`/`catch` callbacks may await without premature derived-Future settlement.
- [ ] existing manual Fiber and failure/upvalue behavior remains passing.
- [ ] E007 and E008 have passing permanent regressions before being marked fixed.
- [ ] E010 is not falsely claimed fixed.
- [ ] all required obsolete mechanisms pass negative-search gates.
- [ ] canonical concurrency/System/floor documents match implementation.
- [ ] all deferred gates are resolved.
- [ ] final format/build/test/clippy gates pass or a clearly classified pre-existing baseline blocker is recorded.
- [ ] implementation state contains no unresolved `INCIDENT`.

---

# 26. First Resume Action for the Implementing Agent

Before writing code:

```bash
git status --short
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
rg 'FiberStatus::' phalcom-core/src phalcom-core/tests
rg 'ready_queue\.(push_back|pop_front)' phalcom-core/src
rg 'System\.schedule|nextScheduled|Fiber\.yield' \
  phalcom-core/core/universe/src/concurrency \
  phalcom-core/tests/fixtures/language/concurrency
```

Then inspect only:

```text
phalcom-core/src/heap/fiber.rs
phalcom-core/src/primitive/fiber.rs
phalcom-core/src/vm/dispatch.rs
```

Confirm C0's current anchors still match this plan.

If they do, begin:

> **Checkpoint C0, Task 1 — Expand `FiberStatus` into truthful execution states.**

Do not open Future implementation for editing until C0 and C1 are complete.
