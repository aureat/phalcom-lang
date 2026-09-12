---
id: CONC002.C2.P1-R1
program: CONC002
checkpoint: CONC002.C2
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
prepared: 2026-09-12
prepared_against: 2db7e3780772e847a178918ede239d91e0bcc5fd
---

# CONC002.C2.P1-R1 — Native suspension and VM control continuations

## Goal

Make language-level native control operations suspension-transparent by representing their semantic continuation in VM/Fiber-owned state, while preserving explicit guards for residual synchronous host algorithms whose continuation still lives on the Rust stack.

This is the sole C2 native-control implementation plan. It does **not** preserve an active predecessor P1 and it does **not** implement a reactor. Its final readiness gate defines only the executor handoff that C3 will consume.

## Required C1 input

Before editing runtime code, consume the completed/current outputs of C1 P3/P4:

- truthful eight-state Fiber lifecycle and exact park generation;
- scheduler-owned `Queued` reservation and stale-entry tolerance;
- durable terminal completion observer and E010 detached-failure ownership;
- generic `Future<T>`/callback/adoption behavior as recertified by P3;
- explicit status of callable-domain/existential prerequisites from P4.

P1-R1 must not redesign Future combinators or force final Fiber generics.

## Checkpoint map

| Gate | Tasks | Semantic boundary | Required evidence |
|---|---:|---|---|
| C0 — Reproducible baseline and scope | 1–3 | Known legal suspension, known native refusal and residual host callbacks are distinguished on the actual tree | controls + red native-control reproducer + inventory |
| C1 — Owned control and truthful activation outcomes | 4–8 | native semantic continuation has one live/parked owner; pending work/switch/return are explicit outcomes | internal outcome/GC/gateway tests |
| C2 — Unified transfers and safe host escape | 9–13 | value, Raise and non-local return cross VM controls and retained host floors exactly once | router/host/escape tests |
| C3 — Suspendable protection and cleanup | 14–17 | `on`, `ensure`, Error.raise and cleanup may suspend with exact precedence | public source fixtures + GC controls |
| C4 — Child failure re-enters parent continuation | 18–20 | Call child failure becomes Raise at parent call site; parent controls run before terminal publication | nested child/catch/ensure/observer/trace tests |
| C5 — Suspension-transparent native control fallbacks | 21–24 | Bool, Option, whileTrue and reversed ordering no longer rely on recursive host continuations | explicit/deopt/Family/fallback tests |
| C6 — Lifetime/bootstrap integration | 25–27 | parked control state, errors, captures, privilege and traces survive/release correctly | real parked GC + trace + native contract tests |
| C7 — Executor readiness handoff and C3 compatibility | 28–30 | exact wait episodes become runnable once; wake never runs guest code; future C3 extension point is precise | wake/queue/root-current-behavior tests |
| C8 — Migration closure and delivery | 31–34 | migrated paths have no legacy re-entry; residual host callbacks remain guarded; broad evidence is classified | searches + perf note + broad gates |

## Architectural invariants

### A. Do not make arbitrary Rust frames suspendable

The current native-yield error protects a real representation invariant. Never “fix” suspension by weakening/removing the native-depth/Fiber-switch guard while an unrepresented host continuation is live.

Preferred split:

1. leaf synchronous primitives remain cheap Rust calls;
2. language/library operations that invoke arbitrary user code move to ordinary VM-visible activations/control records;
3. residual native algorithms with live Rust state remain explicitly guarded;
4. a future resumable-native ABI is reserved for genuinely native asynchronous state that cannot be represented as ordinary VM activation—not for ordinary higher-order library control.

### B. Wake is deferred admission, never recursive execution

A completion/wake may validate and reserve a Fiber and enqueue it. It may not recursively execute arbitrary continuations inside Future settlement or another wake callback.

### C. Value, transfer and suspension are distinct

`None`, `Error`, Unit and arbitrary Values are data. VM control routing uses explicit states/outcomes. Suspension is never encoded as a Value or exception sentinel.

### D. Active ancestors remain non-runnable

A caller whose descendant owns the active continuation is blocked structurally. If the leaf parks, the leaf owns the external wait; ancestors do not independently become queue entries.

## Source-of-truth map

| Fact | Authority |
|---|---|
| executing Fiber | `VM.current` + live VM buffers |
| parked Fiber continuation | `FiberObject` buffers |
| lexical non-local-return home | generation-tagged `FrameToken` |
| control-operation identity | new generation-tagged `ControlId` owned by Fiber |
| callback activation outcome | explicit `CallOutcome` propagated by activation owner |
| normal/raise/non-local transfer | explicit rooted transfer product |
| Future/external wait authority | exact `Parked(generation)` episode |
| ready admission | VM queue reservation (`Queued`) |
| terminal observer | durable Fiber completion observer |

These identities must remain distinct. A frame index is not a frame identity; a park generation is not heap lifetime; a control Vec index is not a ControlId; a coroutine `resumer` is not completion ownership.

## Core representation

Create a Fiber-owned control stack with a closed set of control phases and rooted pending transfer state. Keep it out of every ordinary bytecode CallFrame so the empty/hot path remains cheap.

Conceptual structure:

```rust
struct ControlStack {
    records: Vec<ControlActivation>,
    pending: Option<RoutedTransfer>,
    next_id: u64,
}

struct ControlActivation {
    id: ControlId,
    owner: Option<FrameToken>,
    stack_base: usize,
    callback_floor: usize,
    caller_authority: CallerAuthority,
    source_range: SourceRange,
    destination: ControlDestination,
    phase: ControlPhase,
}

enum Transfer {
    Returned(Value),
    Raise(PhError),
    NonLocalReturn { target: FrameToken, value: Value },
}
```

Exact Rust names may adapt to current source, but the ownership/destination contract may not be weakened.

Activation outcomes must distinguish at least:

```rust
enum CallOutcome {
    Returned(Value),
    EnteredFrame,
    EnteredControl,
    SwitchedFiber,
}
```

No consumer may reconstruct these states from frame-count changes or stack-top values.

A control callback that directly invokes a native Fiber operation may switch without creating a callback bytecode frame. Therefore Fiber resume destination must distinguish ordinary operand delivery from completion of an exact waiting ControlId. Successful and exceptional switch-back both consume that destination exactly once.

## Transfer routing

The dispatcher processes pending controls/transfers before fetching another bytecode instruction or declaring a frame-floor completion.

Required routing order:

1. ordinary callback return closes/truncates the completed child window and delivers once to its explicit destination;
2. any failure leaving an instruction/activation segment becomes a rooted Raise offered to the innermost eligible control before Fiber terminalization;
3. non-local return validates its generation-tagged home first and walks crossed ensures inside-out **before** destroying frames/slots they need;
4. cleanup retains the pending prior outcome while it runs or parks; normal cleanup restores it, cleanup Raise/return replaces it;
5. a transfer leaving a retained synchronous host floor escapes through a private internal host-boundary protocol until Rust housekeeping is released; it is never normalized as a language Error;
6. only after local controls finish may a transfer reach Fiber terminal/resumer/completion handling.

## Protected execution semantics

### `on`

Migrate protected body, dynamic `error.is(class)` matching and handler invocation to VM-owned phases. Preserve first-match behavior, actual dynamic `is(_)` dispatch, error identity/help/rendered/traceback behavior and the rule that matching/handler failures do not get caught by the same region.

### `ensure`

Required precedence:

| Body outcome | Cleanup normal | Cleanup raises E2 | Cleanup returns non-locally R2 |
|---|---|---|---|
| value V | V | E2 | R2 |
| raises E1 | E1 | E2 | R2 |
| non-local return R1 | R1 | E2 | R2 |

Cleanup may suspend repeatedly. No terminal Fiber/Future observation happens until cleanup completes.

### Error.raise

If obtaining/rendering a user-defined message can dispatch user code, make that path VM-visible before constructing the Raise. The original Error receiver must remain rooted and retain identity.

## Child failure semantics

After control records exist, replace the transitional eager Call ancestor failure cascade.

For a failed child:

```text
child local controls finish
    ↓
child terminalizes exactly once
    ↓
Call mode: restore parent and inject Raise at exact call destination
Try mode: restore parent and deliver captured Error as data
Scheduler mode: preserve scheduler isolation/E010 ownership
```

The parent may catch, run/suspend cleanup, continue, or later fail. A parent is not marked Failed merely because its child failed.

## Native fallback migration

Migrate the ordinary user-callback control surfaces whose Rust frames retain callback/postwork state:

- `Block.on` / `Block.ensure` / native `whileTrue` fallback;
- Bool lazy/conditional callback primitives;
- Option.match fallback;
- Error.raise user-message callback path;
- reversed Ordering validation callback path.

Retain guarded synchronous host boundaries where this plan intentionally does not make the algorithm resumable, including current Map/Set hash/equality probing, display/printing/list rendering, bounded typing reflective orchestration and explicit synchronous embedder APIs unless source inspection at C0 shows they changed.

Every retained callback must be listed with its live Rust postwork and guard in the implementation-state inventory.

## GC, upvalues, access and tracing

Control state moves with the same live/parked Fiber ownership transfer as frames, operand stack, open upvalues and invariant-checking state.

Both running and parked root traversal must trace every retained Value/callable/class/error payload. Tokens, indices, symbols and ranges are not roots. Preserve the exhaustive VM root destructure so a new field requires explicit classification.

Close open upvalues before truncating each actually abandoned stack segment. Do not close ancestor slots before cleanup that captures them runs.

Capture callback caller authority at admission; do not reconstruct privilege from a later ambient native frame. Traces show logical Phalcom frames/control sites and Fiber boundaries, not stale primitive/native markers.

## C7 readiness handoff — exact C2/C3 boundary

C2 preserves and verifies this mechanism:

```text
readiness owner holds a live Fiber reference + exact wait generation
    ↓
readiness signal arrives
    ↓
validate Fiber is still Parked(the same generation)
    ↓
set Queued and enqueue exactly once
    ↓
return; do not run Fiber here
```

C7 may generalize comments/naming away from “Future-only” if the implementation is source-independent. It does not add pollers, timer wheels, external registration tables, sockets/files/process backends, idle loops, cross-thread mutation or new public async-native APIs.

Document the current root behavior truthfully: unresolved root await with no runnable work is an error because there is no external readiness source yet; root program completion drains runnable work and then completes.

## Tasks

### C0 — baseline and scope

1. Capture actual HEAD/branch/status/diff and C1 prerequisite revisions; create the P1-R1 implementation-state file.
2. Re-run direct/transitive searches for `block_call`, `run_until`, `send_dynamic`, `invoke_method_object`, activation-outcome reconstruction and host error normalization; classify every production callback caller.
3. Register/select isolated native-control test lanes and prove one legal ordinary callback suspension control plus one genuine native-control refusal on the actual baseline.

### C1 — control ownership/outcomes

4. Add ControlId, control phases, transfer and destination types; bounded checked identity allocation; invariant/root visitors.
5. Add control state to VM/Fiber live/parked ownership and replace scalar-only resume destination with operand-versus-control destination.
6. Trace running and parked controls, pending transfer and every heap-bearing error/control payload; prove release as well as survival.
7. Propagate `Returned / EnteredFrame / EnteredControl / SwitchedFiber` through every owning/forwarding gateway; delete frame-count/stack-top outcome reconstruction in migrated paths.
8. Drive admitted control iteratively from the VM loop; immediate results are rooted pending state, bytecode entry returns to normal dispatch, direct Fiber switch records exact control completion destination.

### C2 — unified transfers/host escape

9. Route all callback return and error exits through the control reducer before frame-floor completion.
10. Replace eager bulk `ReturnNonLocal` destruction with a target-aware unwind cursor that pauses for crossed cleanup.
11. Install synchronous host-boundary descriptors before target activation and define a compact private internal host-escape transport.
12. Ensure Map/Set locks, temp roots, formatting/typing postwork and other retained host housekeeping release correctly before forwarding escape; prevent internal escape from becoming user Error/Future rejection/E010/module failure.
13. Add focused router/host/escape tests and negative leak checks.

### C3 — suspendable protection/cleanup

14. Convert `on` body/match/handler to VM control phases and ordinary activation.
15. Make Error.raise user-message evaluation VM-visible with original error rooted.
16. Convert `ensure` to saved-transfer phases; cleanup may park; cleanup divergence supersedes prior outcome exactly once.
17. Add public source regressions for repeated yield, pending await, matching/handler suspension, full cleanup precedence and revised residual-native guard fixtures.

### C4 — parent exception injection

18. Split Fiber restore from typed value/exception delivery; route result/raise to operand or exact waiting ControlId.
19. Replace eager Call ancestor terminal cascade with one-child terminalization plus parent Raise injection; retain Try/Scheduler semantics and E010 distinction.
20. Prove nested parent catch/ensure, suspended parent cleanup, observer timing, Error-as-data and ancestor non-admission.

### C5 — native callback fallbacks

21. Tail-forward Bool/Option chosen callbacks through the ordinary shape activation gateway.
22. Add VM control phases for one-arm Bool wrapping and `whileTrue` condition/body postwork.
23. Make reversed Ordering callback/validation VM-visible.
24. Force real fallback/deopt/Family/captured-method/native-Fiber callback paths in tests; move depth-guard tests to actual residual host callbacks.

### C6 — lifetime/bootstrap

25. Force GC while actual body/cleanup/match/transfer state is parked; prove sole-root survival and later reclamation.
26. Verify caller authority, same-closure/different-Fiber state, logical tracebacks and terminal control cleanup.
27. Verify native descriptor/source/reflection identity stays one canonical callable surface; use existing shape ABI rather than inventing a second native/source declaration system.

### C7 — executor readiness handoff

28. Preserve exact parking episode identity and pre-registration safety checks; keep manual/scheduler ownership restrictions unchanged.
29. Extend exact wake tests across old/new episodes, duplicate/stale/terminal/blocked states and assert zero user-code execution at wake time.
30. Document C3 extension points at current root-await/root-drain boundaries and optional future native continuation ownership without adding reactor implementation.

### C8 — closure/delivery

31. Remove obsolete production re-entry helpers/usages and update native restriction documentation to name only real residual guarded boundaries.
32. Measure/check hot-path structural cost: empty control stack cheap, no new allocation/hash/atomic/trait overhead on leaf natives; benchmark predecessor versus implementation where existing harness permits.
33. Run full focused/core/corpus/CLI then workspace format/build/test/Clippy gates serially; classify baseline failures rather than broad-fixing them.
34. Close the state record with exact migrated/residual inventory, root/lifetime ledger, commands/results, release blockers and C2.P2/C3 handoff.

## Mandatory hostile regression matrix

At minimum cover:

- immediate native callback result that creates no frame;
- native Fiber switch that creates no callback frame;
- two control records at equal bytecode depth;
- control arguments rooted only by the control record across GC;
- runtime Type/Arity/Access failure under `on`/`ensure`, not just explicit Raise;
- non-local return across nested ensure and a target that remains inside an outer region;
- handler failure not caught by its own region;
- dynamic `is(_)` matching that suspends/fails;
- cleanup that awaits and later overrides an earlier Raise/return;
- Call child failure caught by parent after parent cleanup;
- Try/Scheduler isolation unchanged;
- Bool Some(None)/Error data and unselected-branch nonexecution;
- whileTrue condition/body suspension and strict Bool failure;
- reversed Ordering result validation after suspension;
- sole-root parked state across forced GC;
- caller authority/private/foreign-receiver checks after resume;
- stale/duplicate wake never runs guest code;
- residual Map/Set/render host callbacks still reject unsafe Fiber switching without corrupting later execution.

## Verification command families

Discover exact names first:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib -- --list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core -- --list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus -- --list
```

Focused owners include `vm::control::tests`, native-control execution/depth/Family/reflection tests, new `concurrency_native_{protection,child,fallbacks,negative}` lanes, existing blocks/errors/control-flow/booleans/absence/option/concurrency lanes, GC and traceback/native-contract targets.

Final gate:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test cli-smoke
cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

If `fiber-pool`/`vm-trace` remain available, run one targeted control/trace compatibility gate under each because Fiber parked buffers and dispatch tracing change.

## Deletion/negative searches at C8

```sh
rg -n 'block_call\(' phalcom-core/src
rg -n 'send_dynamic\(|invoke_method_object\(|run_until\(' phalcom-core/src
rg -n 'frames\.len\(\) > before|frames_before_cleanup|failed = resumer' phalcom-core/src/vm phalcom-core/src/primitive
rg -n 'CannotYieldAcrossNativeFrame|native_reentry_depth|floor_depth' phalcom-core/src/primitive/fiber.rs phalcom-core/src/vm
```

Expected outcome: no migrated control path retains host recursion/outcome inference/eager ancestor cascade; residual host boundaries still have their safety guards and are listed explicitly.

## Implementation-state template

Create `native-suspension-r1-implementation-state.md` when execution starts, recording:

- baseline/current revision and relevant dirty dependencies;
- established invariants/decisions;
- checkpoint evidence with exact command/test count/result;
- migration/residual host inventory;
- root/lifetime table;
- deferred gates;
- active incident or `None`;
- exact next task.

Do not pre-mark any C0–C8 gate complete from planning/source inspection.

## Explicit exclusions

This plan does not implement:

- reactor backend or external poller;
- timers/sockets/files/process async APIs;
- manual coroutine pending-await or yield-await-yield routing;
- final `Fiber<R>` typing;
- cancellation/queue revocation;
- Task/TaskGroup;
- channels/select;
- threads/parallel heap sharing;
- source `async` coloring;
- a universal resumable-native ABI.

Those exclusions are ownership boundaries, not permission to leave an unsafe native guard removed.
