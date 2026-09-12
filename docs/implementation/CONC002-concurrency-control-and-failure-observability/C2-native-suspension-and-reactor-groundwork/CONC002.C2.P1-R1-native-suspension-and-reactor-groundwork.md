---
id: CONC002.C2.P1-R1
program: CONC002
checkpoint: CONC002.C2
revision_of: CONC002.C2.P1
kind: expanded-implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
prepared: 2026-09-12
baseline_head: 01b58ec04faa63414ececf46dab10e4ebd4bba71
---

# CONC002.C2.P1-R1 — Native suspension and reactor groundwork

## Checkpoint map

The program is CONC002; the owning repository checkpoint is CONC002.C2. C0–C8 below are **semantic execution gates within this revision**, not new program-level checkpoint directories. Execute in order. A task is an editing boundary; its containing checkpoint is the evidence boundary.

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| C0 — Reproducible baseline and scope | 1–3 | Known good callbacks, known native refusal, and current semantics are distinguished on the actual starting tree | Existing concurrency/errors controls; registered red reproduction; source inventory and ownership record | All implementation assertions → their owning gate; broad suites → C8 |
| C1 — Owned control and truthful activation outcomes | 4–8 | Control state has one live/parked owner; forwarding never mistakes pending work or a Fiber switch for a returned value | Internal outcome/ownership/GC tests; existing reflection, family and invocation checks; one compilation checkpoint | Full unwind semantics → C2–C4; full corpus → C8 |
| C2 — Unified transfers and safe host escape | 9–13 | Normal completion, Raise and non-local return traverse explicit boundaries, including retained host floors | Router/host bridge tests; existing block, error and depth behavior; negative transfer-tag checks | Public on/ensure suspension → C3; child propagation → C4 |
| C3 — Suspendable protection and cleanup | 14–17 | On/ensure and Error.raise run arbitrary callbacks with VM-owned continuations and exact cleanup precedence | New protection/cleanup cases; existing errors; GC saved outcomes; explicit and desugared sends | Fiber child failure injection → C4; fallback migration → C5 |
| C4 — Child failure re-enters the parent's continuation | 18–20 | Call-mode failure raises at the parent call site; parent cleanup finishes before terminal publication | Nested child failure, parent catch/ensure, suspended parent cleanup, Try/Scheduler isolation, E010 controls | Broad scheduler certification → C8 |
| C5 — Suspension-transparent native control fallbacks | 21–24 | Bool, Option, whileTrue and reversed ordering use ordinary activation; bypassed callbacks stay unexecuted | Explicit/deopt cases, forwarding routes, bounded recursion, legacy guard replacement | Full corpus and delivery → C8 |
| C6 — Lifetime and bootstrap integration | 25–27 | Live/parked outcomes, captured state, privilege and traces remain correct and are released at completion | Forced GC in actual parked cleanup, nested traces/access tests, native/full Universe contracts | Feature combinations and broad compatibility → C8 |
| C7 — Reusable readiness boundary | 28–30 | Exact wait episodes become runnable once, without running code at wake; reactor gaps are precisely located | Existing/extended wake tests, fake source only if useful, root idle-contract assertions | Real external registrations/polling → excluded future reactor work |
| C8 — Migration closure and delivery | 31–34 | No migrated path retains host re-entry; every residual boundary is guarded; all required evidence is accounted for | Deletion inventory, performance evidence, complete core/corpus/CLI and named workspace gates | None without explicit release-blocker or scope disposition |

## 1. Authority, revision boundary and baseline

This is a standalone expanded execution plan following [the repository planning prompt](../../../agents/prompt--create-plan.md). It preserves [P1](CONC002.C2.P1-native-suspension-and-reactor-groundwork.md) unchanged. P1's SHA-256 at revision preparation is `c12cdf49783bea3ebcef4b130126238339dd9e13b8488c01f6dc5eeca47ce14f`.

Repository: `/Users/altunhasanli/dev/phalcom/phalcom`; branch `main`; HEAD `01b58ec04faa63414ececf46dab10e4ebd4bba71`. Recent relevant commits: `01b58ec0` setter invocation repair, `9639f27d` selector activation workspace record, `420cf292` selector/Family activation. **This inspection includes uncommitted work.** It is not a clean-main behavioral certification. No Cargo build/test/benchmark was executed while preparing R1.

Relevant modified files include `phalcom-core/src/vm/{mod,dispatch}.rs`, `phalcom-core/src/heap/fiber.rs`, `phalcom-core/src/primitive/fiber.rs`, Universe `phalcom-core/src/concurrency/fiber.ph`, the concurrency specification and fixtures. Concurrent semantic/Future generic work also exists. Capture the actual diff before implementation; this list is a point-in-time inventory, not exclusive ownership. Do not reset, stash, broadly format, overwrite, or stage other agents' changes.

[Implementation hierarchy](../../README.md) and [specification authority](../../../spec/README.md) govern placement and semantics. Effective chapters currently remain in `docs/spec/current/`: `concurrency.md`, `error-handling.md`, `blocks.md`, `functions.md`, `control-flow.md`, `memory-management.md`; follow their linked decisions when needed. A runtime restriction documented there must be reconciled after the implementation is proven. A design or previous plan does not silently override a normative rule.

[CONC002.C1.P3](../C1-concurrency-control-and-failure-observability/CONC002.C1.P3-concurrency-architecture-and-continuation-semantics.md) and the newer [C1.P4](../C1-concurrency-control-and-failure-observability/CONC002.C1.P4-remaining-fiber-and-native-activation-work.md) overlap. R1 owns **P4 §2's native continuation and parent-failure injection slice**. It does not implement P4's Fiber generics, generator-await consumer protocol or cancellation. If §2 lands first, consume and verify its implementation instead of building a second control stack. C3 standard-library work is separate.

No source-language `async` coloring, universal suspendable-native ABI, reactor backend, timers/sockets/filesystem/process APIs, threads, Task/TaskGroup, cancellation, channels/select, or general Future redesign is part of this patch. Manual coroutine awaiting a pending Future remains restricted until a durable consumer relation exists; use scheduler-owned Fibers for pending-await tests. Scheduled Fibers still cannot emit user yields without a consumer. **Suspension transparency removes the native obstruction; it does not change those ownership rules.**

### R1 corrections and expansions relative to P1

1. Name all shape outcome inference sites, including `dispatch_selected_method_as`, `dispatch_shape_at_as`, and the rest branch of `activate_captured_method_as`. Adding an enum variant alone is insufficient.
2. Make child-failure injection a required gate: the current eager cascade would bypass newly installed parent controls.
3. Cover a host boundary from **before target activation**, not only while `run_until` is running. Immediate native dispatch can switch or enter more native code.
4. Distinguish a Fiber switch, bytecode entry, control entry and ordinary return. Do not infer any of them from a value or frame-count comparison.
5. Make host non-local-return escape an explicit, rooted, non-catchable internal transfer protocol, with Map/Set lock release and no resumed native algorithm work.
6. Correct corpus scheduling: Cargo filters select Rust test names, not fixture filename prefixes. The harness scans one directory non-recursively. Add named native-control sublanes and registrations.
7. Identify obsolete negative/depth fixtures and replace their test vehicle while preserving the protected invariant.
8. Preserve actual guard conditions: `fiber_resume` requires depth zero; yield/preparePark/park compare depth to the Fiber's `floor_depth`. Do not mechanically replace them with P1 prose.

## 2. Architecture and source evidence

All paths below are repository-relative. Symbols, not line numbers, are patch anchors. New APIs are marked STRUCTURAL. EXACT means an inspected source edit or registration snippet; neither classification claims a test has run.

### 2.1 Source-of-truth map

| Fact | Authority | Derived consumers | Forbidden replacement authority |
|---|---|---|---|
| Current continuation owner | `VM.current` and live buffers versus parked `FiberObject` buffers | dispatch, upvalues, tracing, scheduler | current frame count or an unrelated VM-global map |
| Lexical return destination | `FrameToken { frame_index, generation }` | ReturnNonLocal and unwind router | frame index alone; closure identity; a Rust stack address |
| Native return window and dispatch authority | `ArgumentView`, `InvocationLayout`, caller `(Option<ClassId>, bool)` | Function/Family/Method/perform and control activation | rebuilt positional-only binding or ambient later native privilege |
| Callable entry result | explicit `CallOutcome` at the dispatch owner | forwarding gateways and control reducer | `frames.len()` comparisons or stack-top sentinels |
| Normal/exceptional/non-local transfer | new Fiber-owned control transfer product | catch, ensure, Fiber completion | Error-as-data, None, or a shrunk-frame heuristic |
| Wait permission | exact `Parked(generation)` plus live owned Fiber handle | Future waiter and scheduler admission | public Suspended state or guessed ObjRef validity |
| Ready admission | `wake_parked_fiber` and `enqueue_unowned_fiber` | queue and resumeScheduled | directly running code in a completion callback |
| Native/source contract | native descriptor association plus authored Universe declaration | bootstrap, runtime dispatch, reflection | a second synthetic declaration or broad baseline suppression |

`ObjRef` is a heap handle, `FrameToken` is activation identity, a proposed `ControlId` identifies a control operation, and a park generation identifies a waiting episode. These identities must remain distinct. A frame token is not a GC root. A class handle captured for dispatch authority **is** a root. A park generation does not keep its Fiber alive. A Fiber resume destination identifies the operand window or exact waiting ControlId to receive a switch-back result; resume_slot alone cannot express the latter.

### 2.2 Inspected control paths

| Evidence anchor | Current path / consequence |
|---|---|
| `phalcom-core/src/method/object.rs::CallOutcome`, `PrimitiveFn` | Shape primitives return `Returned(Value)` or `EnteredFrame`; legacy leaves return `PhResult<Value>`. Retain the cheap legacy function pointer and inline argument buffer. |
| `phalcom-core/src/vm/send.rs::activate_function` | Activates Block, Closure, BoundMethod, Family, AssociatedFamily, BoundMethodFamily. Use this rather than block_call's narrower callable resolution. |
| `send.rs::call_method_legacy`, `call_method_with_selector_as` | Push native access context, invoke primitive, pop context and reconcile stack. `switch_pending` is consumed here; the result must be propagated truthfully to outer forwarding gateways. |
| `send.rs::dispatch_selected_method_as`, `dispatch_shape_at_as`, `activate_captured_method_as` | Reconstruct outcomes using frame depth and stack top. These are migration sites, not reliable authorities for new control work. |
| `primitive/block.rs::block_call` | Pushes arguments and bytecode frame, then increments native depth and recursively runs to a saved frame floor. Cannot retain Rust result/loop/protection state across a switch. |
| `block_on` | Validates Class; saves stack/frame depths; executes body recursively; captures traceback and normalizes non-Raise errors; unwinds; dispatches overridable `is(_)`; executes selected handler recursively. |
| `block_ensure` | Saves `PhResult<Value>` in Rust and temporarily roots success/Raise payload; cleanup may override with error or non-local return detected by frame shrink. |
| `vm/dispatch.rs::run_until_inner` | Returns early with `?` from many opcode paths; tests drained frames before safepoint/fetch; hoists `Rc<Callable>` but reloads frame/IP. Router interception must cover all error exits without losing the hoist's safety. |
| `Bytecode::{Return,ReturnNonLocal}`, `unwind_to`, `close_upvalues_from` | ReturnNonLocal destroys all crossed frames eagerly. Ensure must run before crossing destruction boundaries, with upvalues closed against correct stack segments. |
| `dispatch.rs::run_until`, failure arm | Marks current and Call-mode ancestors failed without executing ancestor bytecode; clears their parked state. New parent on/ensure requires resuming parent with a pending Raise instead. |
| `dispatch.rs::switch_to_fiber_and_deliver` | Restores target buffers and pushes a value at resume_slot. An exception delivery sibling must restore without inventing a return value. |
| `primitive/fiber.rs::store_live_into/load_live_from` | Moves frames, stack, open_upvalues, checking. New control stack and pending transfers must move with these. |
| `vm/gc.rs::collect_roots` | Intentionally exhaustive VM destructure; new fields must be classified. Preserve this compile-time reminder. |
| `heap/trace.rs::trace_object`, Object::Fiber arm | Roots parked stack/frames/resumer/observer/result/entry/checking. Extend to parked control and pending transfer roots. |
| `error.rs::{PhError,RuntimeError}` | Besides Raise, `SelectorPatternMismatchContext` retains Family and receiver Values. Module-initialization failures can recursively retain PhError. Trace all retained error payloads, not just Raise. |
| `vm/mod.rs::wake_parked_fiber` | Exact status comparison, Queued transition, queue insertion; no execution. Existing tests cover mismatch/current/duplicate tickets. |
| `primitive/fiber.rs::{fiber_prepare_park,fiber_park}` | Preflight before waiter registration; checked generation increment; only scheduler ownership can park. Preserve admission order. |
| `core/universe/src/concurrency/fiber.ph::Future.await` and root drain in `run_until` | Empty queue with unresolved root await is an error today. No external-registration/poll abstraction exists. Document future integration without adding a no-op reactor. |

### 2.3 Exhaustive direct re-entry disposition for the inspected scope

Production direct `block_call` sites: block loop condition/body, on body/handler, ensure body/cleanup; Bool `and`, `or`, `ifTrue`, `ifFalse`, paired conditional; Option.match. The use in `universe/mod.rs::kernel_option_match_override_reroutes_every_combinator` is test-only. Direct synchronous VM-driving APIs are `block_call`, `send_dynamic`, `invoke_method_object`; `run` is the owning top-level driver.

| Caller(s) | Taxonomy | Patch disposition and reason |
|---|---|---|
| Function.call/callWith; ordinary Iterable/List.each and for callbacks | Already VM-visible | Preserve; positive control. Do not rewrite collection traversal. |
| Block.on/ensure/whileTrue | Native control | Migrate via control records; no recursive host continuation remains. |
| Bool lazy/conditional primitives; Option.match | Tail callback or small post-call transform | Tail activation where possible; WrapSome only for one-armed Bool callback result. |
| Error.raise → `message` | User send followed by Raise | Migrate message-then-raise; raising must not hide an avoidable user-message callback restriction. |
| `Bytecode::ValidateOrdering { reverse: true }` → `reverse` | User send followed by validation | VM-visible selector activation plus validator; preserve both class checks. |
| `primitive/mod.rs::{send_hash,send_eq}` → `map.rs::locate_key` and `set.rs::locate` | Re-entrant native algorithm with probe/lock state | Keep synchronous guard; ensure internal escape releases locks before leaving. No general resumable collection mutation in this plan. |
| `value/render.rs::to_display_string` → `system.rs::system_class_print`, `list.rs::list_to_string` | Re-entrant formatting / buffered output | Keep guarded; do not promise arbitrary toString callbacks suspend. |
| `primitive/typing.rs` send to constructor before `alloc_variant(...TypingKnown...)` | Native reflective orchestration | Keep guarded and enumerate as residual limitation. No typing expression overhaul here. |
| `send.rs::try_module_export_send_dynamic` → `send_dynamic` | Host adapter | Preserve internal visibility/export rules and guard; remove nested duplicate drive only when equivalent routing is verified. |
| Direct host `send_dynamic`/`invoke_method_object` | Synchronous embedder contract | Return a completed value/failure; no suspendable host ABI. Wrap activation as well as recursive drive in the guard. |
| Representation/arithmetic/allocation with no user dispatch | Leaf native | Keep simple; no per-call heap continuation, hash map, atomics or runtime trait. |

Search direct calls again at C0 and C8. Expand helper fanout only through these known seams. A new user-code-driving native caller must receive an explicit migration or guarded-residual disposition; absence from this table is not permission to disable its guard.

## 3. Design contract for the patch

### 3.1 Control representation (STRUCTURAL)

Create `phalcom-core/src/vm/control.rs`; use a closed enum and contiguous owned storage. Do not enlarge every bytecode CallFrame with optional cleanup payloads. Empty control storage must allocate nothing. New VM/Fiber fields are crate-visible implementation state, not source-language fields.

```rust
// Structural API sketch; new types, not paste-ready repository definitions.
struct ControlStack {
    records: Vec<ControlActivation>,
    pending: Option<RoutedTransfer>,
    next_id: u64,
}
struct ControlActivation {
    id: ControlId,
    owner: Option<FrameToken>, // None only for an explicitly owned host/root floor
    stack_base: usize,
    callback_floor: usize,
    caller_authority: (Option<ClassId>, bool),
    source_range: SourceRange,
    destination: ControlDestination,
    phase: ControlPhase,
}
enum ControlDestination {
    OperandWindow { owner: FrameToken, slot: usize },
    ParentControl(ControlId),
    HostBoundary(HostBoundaryId),
}
enum Transfer {
    Returned(Value),
    Raise(PhError),
    NonLocalReturn { target: FrameToken, value: Value },
}
struct RoutedTransfer {
    destination: TransferDestination,
    outcome: Transfer,
}
```

Use existing FrameToken/Value/ClassId/SourceRange. Give ControlId a checked monotonic generation within its owning Fiber; never reuse a Vec index as identity. No raw pointer/reference may outlive an activation call. HostBoundary identity names an actual live synchronous invocation, distinct from Fiber or frame identity.

Records contain their callback/class/error/cleanup/receiver values in their phases. When taking a phase out with `mem::replace`, put every live payload into another traced slot **before** entering user execution or a safepoint. Moving a Rust enum does not root heap handles; cloning an ObjRef does not retain it.

`ControlDestination` defines completion ownership even when there is no bytecode child frame. Each operation completes into exactly one destination. Frame-floor matching alone is insufficient when nested native controls occupy the same bytecode depth. Normal child-return routing uses a stable callback/control identity plus its validated boundary; depth is a bound, not the whole identity.

### 3.2 Explicit activation outcomes (STRUCTURAL)

Extend the existing enum narrowly:

```rust
enum CallOutcome {
    Returned(Value),
    EnteredFrame,
    EnteredControl,
    SwitchedFiber,
}
```

`SwitchedFiber` represents an already-performed VM switch, not asynchronous native suspension. Keep `switch_pending` as the legacy boundary signal, consume it exactly once at that boundary and propagate an explicit outcome afterward. `EnteredControl` means runnable VM-owned control work exists without requiring another bytecode frame. Neither value may be inferred from stack top, Error, None, or equal frame depths.

The activation owner reconciles its receiver window on Returned; gateways forward the resulting disposition without re-reading a different Fiber's stack. Audit nested Function/Family/perform/captured-method gateways, including primitive-bound Fiber operations. Do not merely add wildcard arms that treat new dispositions as a returned value.

Shape gateways should validate/admit control state, then return; the VM loop drives arbitrary callbacks. An immediate callback result is stored in traced pending state and reduced iteratively. The reducer never uses `block_call`/`send_dynamic`/`run_until`. Long immediate-only chains service a bounded safepoint between coherent actions and count against a bounded control-depth policy; they must neither starve GC nor grow the Rust stack.

### 3.2.1 Successful switch-back into a control callback (STRUCTURAL)

Replace FiberObject's scalar-only resume destination with an explicit internal tag (retain the slot as part of each case):

```rust
enum FiberResumeDestination {
    Operand { slot: usize },
    Control { slot: usize, control: ControlId },
}
```

A control reducer may directly invoke a bound native Fiber.call or Fiber.yield. It receives SwitchedFiber, but no callback bytecode frame exists to produce a later Return. Before switching away, record Control{id, slot} on the Fiber whose current invocation is waiting. Successful delivery routes Returned(value) to that exact control phase before bytecode fetch. Ordinary bytecode Fiber calls/yields retain Operand delivery.

Carry the direct invocation's completion destination explicitly through the control activation helper and forwarding gateways; never infer it from the presence of any control on the Fiber. A practical implementation is a scoped `activating_control` marker in ControlStack, installed by the reducer and consumed when a direct native switch records its destination. Clear/suspend that marker when entering bytecode so an ordinary Fiber.yield *inside* the callback delivers to its operand window rather than prematurely completing the enclosing control. Forward it through tail gateways; nested control admission establishes a ParentControl destination. On all outcomes clean up the marker on the original owning Fiber, not whichever Fiber is current after the switch.

Update every current resume_slot consumer: constructors in heap/fiber.rs; fiber_park and fiber_yield registration; fiber_resume's caller registration and already-started callee delivery; switch_to_fiber_and_deliver and root-drive comments in vm/dispatch.rs. Both scheduler wake/resume and manual resume consume the destination exactly once. Failure injection uses the same waiting destination with Raise. No result is pushed as operand data and also delivered to a control.

### 3.3 Transfer router and destruction order

The driver must process pending control/transfer work **before** the old `frames.len() <= base_frames` completion check and before fetching another instruction. `Rc<Callable>` can remain hoisted, but reacquire live frame/IP after every control step/switch. Never persist an IP snapshot over a Fiber switch.

Route:

1. Ordinary Return: identify child destination before pop; close its open upvalues before truncating its operand window; deliver once to control or caller.
2. Any opcode/activation failure: capture needed traceback while frames exist, install a rooted Raise transfer, then look for the innermost eligible control within the current driver floor. Catch records match only their protected body phase.
3. Non-local return: validate `(frame_index,generation)` in this Fiber first. Traverse crossed control records inside-out. Pause at each ensure before destroying frames/slots it needs. A target still inside a region does not exit that region. On does not catch return.
4. Before cleanup activation, unwind only the abandoned callback portion; preserve live owner/home slots used by captures. Later truncate each remaining segment with close-before-truncate. The saved non-local return value is an explicit rooted outcome, not a stack artifact.
5. Suspendable cleanup remains in its cleanup phase; normal cleanup restores pending outcome, divergent cleanup replaces it. Resume by the same destination identity, not a guessed length delta.
6. An exhausted local transfer reaches a retained host floor before any outer control runs. It returns through the native call chain, releasing native housekeeping, then the owning outer VM driver routes it. See §3.5.
7. A raised error reaches the Fiber terminal path only after all local controls complete. Call-mode child failure is then injected into the restored parent's call-site continuation; it does not pre-terminalize the parent.

Suspension itself produces no Transfer arm. Await/yield preserve the current record phase and pending outcome in Fiber-owned buffers.

### 3.4 Protected execution and ensure transition table

| Phase | Completion | Next action |
|---|---|---|
| ProtectBody | ordinary value | remove record; return value |
| ProtectBody | Raise | capture/normalize as current on does; unwind body segment; schedule `error.is(class)` under saved caller authority |
| ProtectBody | non-local return | remove protection; continue targeted unwind |
| ProtectMatch | Bool true | schedule handler(error) |
| ProtectMatch | any ordinary result other than Bool true | propagate original normalized failure, preserving current matching semantics |
| ProtectMatch or ProtectHandler | Raise/non-local return | remove own protection; propagate outward; never catch itself |
| ProtectHandler | ordinary result | complete with handler result |
| EnsureBody | value / Raise / non-local return | save transfer; switch phase first; schedule cleanup |
| EnsureCleanup | ordinary result | discard cleanup result; restore saved transfer |
| EnsureCleanup | Raise or non-local return | discard saved transfer; propagate replacement |
| Any callback phase | yield/park/resume | keep same phase/destination and all retained values |

On's class validation precedes body invocation. Preserve native-error reification, first-match behavior, dynamic `is(_)`, original Error identity, rendered/help and traceback behavior. Do not replace dynamic matching with a superclass shortcut. Matching/handler errors cannot be caught by their own on region.

Cleanup precedence is explicit: success+success → body value; success+E2 → E2; E1+success → E1; E1+E2 → E2. Body non-local return plus successful cleanup → original target/value; cleanup non-local return overrides any pending outcome. Ordinary returned Error/None remains data.

### 3.5 Retained synchronous host floors

Add an owned host-boundary descriptor stack under VM **host execution** state, separate from Fiber continuations. Each descriptor records original Fiber, frame/control floors, original receiver window and unique boundary id. It exists only while a synchronous host adapter is live. Native depth and descriptors must balance on all Result paths. No global temporary-root depth may be used as the parked lifetime of a control record.

`send_dynamic` and `invoke_method_object` currently activate before incrementing native depth. Move preflight/depth/host-boundary installation ahead of any target activation or module-export forwarding. Use one cleanup epilogue after an immediately invoked closure returning Result: capture result, decrement depth/pop descriptor, then return result. Avoid `?` between increment and decrement in the outer body; avoid an unsafe RAII guard retaining `&mut VM` across VM calls.

A private VM transfer escape travels through `PhResult` when a non-local return must unwind retained Rust frames. Prefer a small dedicated `PhError` variant carrying an opaque host-escape identity; actual transfer stays in rooted VM-owned storage. This is not a language error. Explicitly intercept it before normalization, traceback display, Future rejection, scheduler failure reporting or module-failure publication. Preserve the error transport's size test by keeping the escape compact.

The escape must cross **all** nested host descriptors whose floors are outside the transfer target. Each adapter validates boundary/Fiber identity, releases native housekeeping and forwards it. Only a VM driver that owns the next eligible outer boundary consumes it. No host post-callback algorithm may run, although cleanup housekeeping such as Map/Set `exit_reentrant_send` and temp-root restoration must run. Do not replace those epilogues with early `?`.

An embedder root call has no language owner frame: normal return/failure may leave its host API; a non-local return can only target a live frame in the same execution. An escape reaching a public API with no matching owner is an internal invariant failure, never a stringified or catchable substitute return.

### 3.6 Child failure delivery

Introduce `switch_to_fiber_and_raise` or a shared restore helper plus typed delivery. Restore parent's frames/stack/upvalues/checking/control state, truncate the child-call receiver window at the parent's tagged resume destination slot, queue a rooted Raise to its operand-call-site or waiting ControlId destination and set parent Running. Do **not** push a dummy None, mark parent Failed, or execute its next bytecode before handling the exception.

Terminalize the failed child exactly once after its own cleanup. Preserve its durable result/error observer and Try/Scheduler isolation. Call then routes through parent on/ensure; a parent may catch and return, await during cleanup, or fail later. Only its own terminal point can enqueue its completion observer. Preserve logical FiberBoundary trace segments without duplicates as the failure crosses parents.

### 3.7 GC, privilege, bounds and diagnostics

Root all phase values, pending transfers, callback/owner class handles and residual host-escape payloads in both running and parked storage. Use `Value::gc_obj_ref` to include Option-wrapped handles. FrameToken/ControlId/indices/SourceRange/symbols are not heap roots. Trace PhError's actual heap-bearing variants, including nested module failures if retained; never assume every Err is Raise.

`VM::collect_roots` must remain an exhaustive destructure. Heap-only `Universe::verify_invariants(&Heap)` cannot verify the VM's running control buffer; add an internal VM control-state invariant checker and call it from tests. Check parked buffers through the same logical validator with the correct stack view.

Retain existing caller visibility and internal dispatch rules. Store the entry caller authority from ArgumentView; no ambient `native_method_contexts` entry should remain after a migrated gateway returns. Control bookkeeping may use runtime access internally, but user callbacks must receive exactly the authority ordinary invocation grants. Preserve foreign receiver layout checks.

Use a checked bounded control-stack depth, initially tied to `MAX_CALL_DEPTH` as a **separate resource ceiling**; record the new `what: "control depth"` if used. Do not silently halve existing ordinary call capacity by counting every control as an extra bytecode frame. Exhaustion must be catchable and unwind through already installed cleanup after freeing abandoned callback capacity. Control-only dispatch must have GC safepoints at coherent roots.

Capture traceback before destructive unwind; internal records do not invent user stack frames. Retain source call-site information for diagnostics. Distinguish `native_selector/native_class` host diagnostics from a logically retained control invocation; do not leave stale native rendering metadata after a successful entered-control/frame outcome.

## 4. Execution conventions

**Confidence:** EXACT denotes inspected signatures/edits or registration snippets. STRUCTURAL denotes new design APIs whose mechanics must be reconciled with source. INVESTIGATE-BEFORE-EDIT is limited to a named conflict with actual drift, not a license to repeat repository research.

**State file:** create `native-suspension-r1-implementation-state.md` beside this plan when execution starts. Each checkpoint is NOT_STARTED, IN_PROGRESS, COMPLETE or INCIDENT in that ledger; keep repository frontmatter lifecycle vocabulary unchanged. Required evidence failure means INCIDENT, not “mostly complete.” Stop dependent work until the incident is classified and resolved.

**Drift:** before each checkpoint check its files, symbol responsibilities, changes from prior gates and new callers. Adapt signatures/imports mechanically. If a second control authority has landed, a required guard differs, or public semantics conflict, record the exact delta and reconcile before editing. No full repo scan/index rebuild.

**Tests:** commands below are execution instructions, not claimed results. Run Cargo serially with pinned toolchain and `RUSTFLAGS='' RUSTC_WRAPPER=''`. New tests use exact names below and are registered in existing targets. `-- --list` confirms selection once per new test registration; zero selected tests is an incident. Mechanical tasks get no independent behavioral suite. Rerun passed suites only when subsequent changes affect their invariant.

**Out-of-scope defaults for every task:** parser grammar, semantic generic/type-store fixes, compiler type-form lowering, LSP, new source Future/Fiber APIs, reactor/threads. Secondary inspection is permitted only when an enumerated integration test points there. If a native descriptor macro needs a new ABI category instead of existing `abi = shape`, stop and explain why.

## Checkpoint C0 — Reproducible baseline and bounded ownership

Tasks: 1–3.

**Why:** an unchanged P1, a dirty C1 dependency and known-good ordinary callbacks must be distinguished before migrating runtime control. This gate establishes evidence preconditions, not a new implementation behavior.

**Entry:** current repository accessible; preserve all prior work. **Source of truth:** live source, current diff and existing fixtures; no prior test claim is imported as current evidence.

**Working set — primary:** P1, C1.P3/P4, `primitive/{block,boolean,option,error,fiber}.rs`, `vm/{send,dispatch,mod}.rs`, corpus/support files. **Secondary:** specific error/concurrency specification paragraphs. **Out of scope:** implementation edits outside new tests/state.

**Contract:** baseline cases identify native refusal independently from ownership restrictions; direct and transitive re-entry caller dispositions are recorded.

**Risks/hostile cases:** invalid source syntax masquerading as runtime failure; manual-await refusal mistaken for native refusal; scheduled-yield refusal mistaken for the bug; a filename filter selecting zero Rust tests.

### Task 1 — Freeze baseline and reconcile concurrent ownership

**Purpose:** make the later patch attributable. **Risk:** semantic LOW; fanout documentation/local. **Owner:** state file; `git` revision/diff facts. **Dependencies:** none. **Inspect:** P1 hash, C1.P3/P4, relevant working-tree diff. **Do not inspect:** unrelated semantic changes beyond recording their paths.

**Current → target:** mixed uncommitted work → recorded dependency boundary and one owner for C1.P4 §2/C2 implementation.

**Edit operations (EXACT):**
1. Record `git rev-parse HEAD`, `git branch --show-current`, `git status --short`, scoped `git diff --stat` and relevant diff identities.
2. Record P1 hash; do not modify P1. Identify whether C1.P4 §2 has implementation beyond the inspected baseline.
3. Create the state file using §8's template; record the root, pinned toolchain and which concurrent changes are prerequisites.
4. If implementing in another checkout, verify the required uncommitted dependency is included by an authorized mechanism. Do not assume HEAD contains it.

**Must not:** commit someone else's work to manufacture a clean baseline. **Testing classification:** no standalone test; C0 evidence. **State update:** exact starting identity, excluded dirty paths, next task.

### Task 2 — Finalize re-entry and transfer consumer inventory

**Purpose:** enumerate every caller requiring migration/escape forwarding. **Risk:** semantic MEDIUM; fanout multi-file read-only. **Owner:** state inventory; production callers under §2.3. **Dependencies:** Task 1. **Inspect:** `run_until`, `send_dynamic`, `invoke_method_object`, `block_call`, `PhError` normalization sites and Map/Set lock epilogues. **Do not inspect:** unrelated runtime algorithms.

**Edit operations (EXACT search; STRUCTURAL disposition):**
1. Run `rg -n 'block_call\(|run_until\(|send_dynamic\(|invoke_method_object\(' phalcom-core/src`.
2. Expand `send_hash`, `send_eq`, `to_display_string` callers; distinguish cfg(test) sites.
3. Record each retained host callback's native post-work, roots, guard, error/escape propagation and cleanup obligation.
4. Search `CallOutcome::`, `frames.len() > before`, and `switch_pending` in `vm/send.rs`; record each forwarding consumer listed in §2.2.
5. Record PhError conversion/rendering consumers in `vm/dispatch.rs`, `diagnostics/traceback.rs`, `modules/initialize.rs`, `vm/api.rs`, `interpret.rs` as escape-leak boundaries.

**Must not:** call a dynamic callback “leaf” because the current receiver happens to be primitive. **Testing:** C0 source evidence only. **State:** completed inventory plus exact newly found caller, if any.

### Task 3 — Register isolated test lanes and capture baseline failures

**Purpose:** create selectable semantic evidence. **Risk:** semantic LOW; fanout tests multi-file. **Owner:** `phalcom-core/tests/core/language/corpus.rs`, `phalcom-core/tests/fixtures/language/concurrency/native/{protection,fallbacks,child,negative}/`, new internal tests as later gates require. **Dependencies:** Task 2. **Inspect:** `phalcom-core/tests/support/mod.rs::{collect_cases,check_pass,check_negative_at_phase}` and existing each/error fixtures. **Do not inspect:** harness compiler internals unless a fixture cannot compile.

**EXACT registration pattern** in `phalcom-core/tests/core/language/corpus.rs` (repeat for the four named sublanes):

```rust
#[test]
fn concurrency_native_protection() {
    support::check_pass("concurrency/native/protection");
}
#[test]
fn concurrency_native_fallbacks() {
    support::check_pass("concurrency/native/fallbacks");
}
#[test]
fn concurrency_native_child() {
    support::check_pass("concurrency/native/child");
}
#[test]
fn concurrency_native_negative() {
    support::check_negative_at_phase(
        "concurrency/native/negative",
        support::ExpectedFailurePhase::Runtime,
    );
}
```

Add registrations only together with at least one active case in that directory. The harness is non-recursive, so these registrations do not duplicate `concurrency` coverage. `.expected` records exact stdout for PASS; NEGATIVE uses current runtime diagnostic conventions. Use `// area`, `// spec`, `// status` as existing fixtures do.

Create baseline probes for protected yield and scheduler-owned ensure pending await; initially run as expected-red evidence and record the native-frame diagnostic without accepting it as final output. Existing controls: `each_generator_yields.ph`, `concurrency_future_pending_continuations_suspend_safely.ph`, `errors_ensure_all_exits.ph`, `errors_ensure_cleanup_supersedes.ph`, block non-local-return fixtures. A handoff between checkpoints must explicitly state any expected-red case still outstanding; do not use broad ignores to hide it.

**Must not:** claim `cargo test ... concurrency_native_foo` selects an unregistered fixture. **Testing:** C0 is the deliberate baseline/red exception to implementation checkpoints. **State:** exact failing/passing cases and intended final outcomes.

**Required evidence:** list new names; run `cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --exact` and `... corpus::errors -- --exact` with cleared flags, then the new protection lane as expected RED. Existing controls must pass or be classified; new RED must be native-frame refusal, not parse/type/ownership failure.

**Do not run yet:** workspace/core-wide suites → C8; additional future gates not implemented.

**Escalate immediately if:** C1 implementation contradicts ownership assumptions; prior controls fail; native refusal cannot be reproduced with legal ownership.

**Completion:** [ ] Tasks 1–3 recorded; [ ] baseline controls pass or diagnosed blocker; [ ] expected-red path verified; [ ] inventory and deferred tests recorded; [ ] no unresolved baseline incident. **Suggested commit group if authorized:** baseline regressions and evidence, clearly marked expected-red if kept before repair; preferably integrate red fixtures with the repairing checkpoint.

## Checkpoint C1 — Owned control and truthful activation outcomes

Tasks: 4–8.

**Why:** storage plumbing and outcome propagation jointly establish representable work; either alone is unsafe to consume. **Entry:** C0 COMPLETE. **Source of truth:** VM/Fiber-owned ControlStack and explicit CallOutcome.

**Working set — primary:** `phalcom-core/src/vm/{control,mod,bootstrap,gc,send,dispatch}.rs`, `phalcom-core/src/heap/{fiber,trace}.rs`, `phalcom-core/src/method/object.rs`. **Secondary:** shape primitive consumers in `primitive/{object,method,family,block}.rs` located by rg. **Out of scope:** public on/ensure migration and full unwind semantics.

**Contract:** a pending immediate control operation and a switched Fiber cannot be mistaken for an ordinary return; retained control values are traceable under the unique owner.

**Risks/hostile cases:** two controls at equal frame depth; captured immediate native method; switch into same closure at different IP; root/native bootstrap without source types; authority ClassId held only in a record.

### Task 4 — Define control identity, phases and destinations

**Purpose:** represent semantic continuations independently of Rust frames. **Risk:** semantic HIGH; fanout local new module. **Owner:** create `phalcom-core/src/vm/control.rs`; add `mod control` in `phalcom-core/src/vm/mod.rs`. **Dependencies:** C0 inventory. **Inspect:** `FrameToken`, `ArgumentView`, error payloads. **Do not inspect:** parser/semantic layers.

**Current → target:** host-local body outcome/phase → §3.1 owned types and closed phase enum.

**Edit operations (STRUCTURAL):**
1. Add ControlStack/ControlActivation/ControlId/Transfer/destination types with Debug and only justified Clone/Copy derives; do not derive Copy for owned error/Vec payloads.
2. Define phases ProtectBody/Match/Handler, EnsureBody/Cleanup, WhileCondition/Body, WrapSome, RaiseAfterMessage, ValidateReversedOrdering with their exact callable/value fields.
3. Add checked id allocation and bounded admission; a failed admission leaves stack/queue/records unchanged.
4. Add read-only invariant and root-visitation helpers; tokens and indices validate against the supplied live/parked stack view.
5. Keep dispatch operations out of constructors; no constructor calls user code.

**Must not:** store Rust closures/borrows or use frame depth as operation identity. **Testing:** no standalone suite; C1 internal evidence. **State:** actual type definitions and bound policy.

### Task 5 — Move and initialize the control owner with Fibers

**Purpose:** make control state parkable. **Risk:** semantic HIGH; fanout multi-file. **Owner:** `VM`, `vm/bootstrap.rs::new_kernel_with_output`, `FiberObject::{new_entry,new_entry_with_buffers,root}`, `store_live_into/load_live_from`. **Dependencies:** Task 4. **Inspect:** all Fiber constructors via `rg 'pub fn|pub\(crate\) fn' src/heap/fiber.rs`; optional fiber-pool branches. **Do not inspect:** unrelated allocator implementations.

**Edit operations (STRUCTURAL):**
1. Add `controls: ControlStack` to VM and FiberObject; initialize every constructor with empty allocation-free storage.
2. Move controls with `mem::take` in store/load, in the same direction as frames/stack/checking. Replace scalar resume_slot with the tagged FiberResumeDestination from §3.2.1 in every constructor/read/write; preserve the physical slot for both arms. This is internal execution state, not a public Fiber field.
3. Assert running Fiber's parked control buffer is empty and target buffers are not accidentally appended/merged.
4. Preserve global frame generation and host native depth. Do not transplant host boundary descriptors to parked Fibers.
5. Ensure success/failure/pool cleanup eventually clears control/pending fields; final terminal behavior is completed in C4/C6.

**Must not:** clear a still-active parent's controls on child switch. **Testing:** C1 ownership unit cases; no full suite per constructor. **State:** constructor and transfer fanout reconciled.

### Task 6 — Trace controls and retained errors through both root paths

**Purpose:** prevent collected saved outcomes or authority handles. **Risk:** semantic HIGH; fanout multi-file. **Owner:** `vm/gc.rs::collect_roots`, `heap/trace.rs` Fiber arm, `vm/control.rs` root visitor, `error.rs` retained-error visitor if shared. **Dependencies:** Tasks 4–5. **Inspect:** `Value::gc_obj_ref`, RuntimeError::Raise/SelectorPatternMismatch, ModuleFailure causes. **Do not inspect:** collector algorithm unless a visitor is insufficient.

**Edit operations (STRUCTURAL):**
1. Add controls to exhaustive VM destructure as roots; visit live records and pending transfer.
2. Visit parked controls from Object::Fiber using the same value/handle visitor.
3. Enumerate all phase fields; trace caller authority ClassId, error/class/receiver/callback/saved outcome and parent-owned pending payloads.
4. Enumerate all heap-bearing error variants; recursive ModuleInitialization failure causes must not hide a Value. Keep purely owned Strings/Symbols/tokens out of root set.
5. Add synthetic running/parked only-root and after-clear reclamation assertions with real heap handles. Include an Option-wrapped saved object and mismatch-error context.

**Must not:** keep records rooted forever to make survival tests pass. **Testing:** C1 internal GC assertions; real suspended cleanup GC is C6. **State:** root-field ledger including non-roots.

### Task 7 — Propagate activation outcomes instead of reconstructing them

**Purpose:** stop pending work from being consumed as stack data. **Risk:** semantic HIGH; fanout multi-file shared dispatch. **Owner:** `method/object.rs::CallOutcome`; `vm/send.rs::{call_method_legacy,call_method_with_selector_as,dispatch_selected_method_as,dispatch_shape_at_as,activate_captured_method_as}`; rest activation helpers in `vm/dispatch.rs`. **Dependencies:** Task 4 types. **Inspect:** every CallOutcome consumer and `switch_pending` epilogue. **Do not inspect:** compiler optimization passes.

**Edit operations (STRUCTURAL, inspected signature fanout):**
1. Add EnteredControl and SwitchedFiber variants with precise docs from §3.2.
2. Change owning `call_method_legacy` and `call_method_with_selector_as` returns from `PhResult<()>` to `PhResult<CallOutcome>`. Closure branch returns EnteredFrame. Legacy result branch reconciles return or consumes switch_pending and returns SwitchedFiber.
3. Shape branch forwards every disposition; only Returned reconciles a completed receiver window. Clear successful native diagnostic context on entered outcomes as well as Returned; retain actual error metadata until captured.
4. Make `dispatch_selected_method_as` return the owner's outcome directly. Remove its frame-count and stack-top reconstruction.
5. Make `dispatch_shape_at_as` retain direct/rest/DNU outcome; migrate DNU/rest forwarding helpers as necessary to propagate it. Apply same fix to `activate_captured_method_as`'s non-shape rest branch.
6. Change `call_rest_method_as` and `activate_rest_method_as` to propagate the owner disposition. Keep statement-oriented wrappers such as `call_method`, `call_method_with_selector`, `activate_rest_method` returning unit where useful via explicit `.map(|_| ())`; they must not reconstruct completion for forwarding consumers.
7. Update exact/Family/subscript/perform/callWith/captured-method callers located by compiler errors and rg. Preserve InvocationLayout including setter lane and caller authority unchanged.
8. Verify nested gateway success reconciliation is idempotent at the same original receiver slot; never touch a different Fiber's stack after SwitchedFiber.

**Must not:** add `_ => Returned(stack.last())` or infer EnteredControl by comparing Vec lengths. **Testing:** C1 outcome/gateway cases; one strategic `cargo check -p phalcom-core` after fanout compiles, proving type-level migration only. **State:** callers changed and negative inference search.

### Task 8 — Add deferred control admission and scheduler-step integration

**Purpose:** ensure arbitrary callbacks execute only after native gateways return. **Risk:** semantic HIGH; fanout control/dispatch. **Owner:** new control admission/drive helpers; `run_until_inner` prefetch boundary. **Dependencies:** Tasks 4–7. **Inspect:** frame-drain check, safepoint, hoisted callable. **Do not inspect:** VM instruction encoding or parser grammar.

**Edit operations (STRUCTURAL):**
1. Add admission helpers that root operation args, retain original destination/authority and schedule its first phase without executing callback bytecode.
2. Add one iterative drive action: activate callback via `activate_function` or selector gateway; consume immediate Returned into rooted pending state, return to bytecode for EnteredFrame, retain parent/child ControlId for EnteredControl, and restart from `VM.current` for SwitchedFiber. Supply an explicit scoped direct-control completion marker as §3.2.1 specifies; native switch registration consumes it into the source Fiber's Control resume destination. Clear it on bytecode entry and every other completed admission path, using the source Fiber identity after a switch. A later successful switch-back queues Returned to that ControlId; no callback frame Return is required.
3. Drain ready control steps before frame exhaustion/fetch. Define a coherent-root safepoint between actions and limit admission/control depth.
4. Add internal synthetic operations/tests for immediate/frame/control/switch dispositions and same-depth nested controls. Public gateways remain unmigrated until C3/C5.

**Must not:** recursively drive control to completion inside a primitive or assume every callback creates a frame. **Testing:** C1 combined gate. **State:** drive-loop entry order and immediate-result convention.

**Required evidence:** internal `vm::control::tests` selection under `--lib`; `--test core execution_family_runtime`, `execution_send_arity`, and `reflection_conformance` only for the affected gateways; `cargo check -p phalcom-core` once after Task 7. Verify each filter from source/list. These establish representation and dispatch, not public suspension.

**Do not run yet:** full core/corpus/workspace → C8; protection lane remains recorded expected-red → C3.

**Escalate immediately if:** preserving outcome requires a new public native ABI, ownership needs references into live Vecs, or descriptor-only bootstrap cannot install existing shape ABI.

**Completion:** [ ] Tasks 4–8 integrated; [ ] outcome/owner/GC hostile cases pass; [ ] no old frame-inference forwarding authority remains in migrated helpers; [ ] state records compile evidence and deferred semantics; [ ] no INCIDENT. **Commit grouping:** one control-ownership/outcome-foundation unit, optionally mechanical fanout plus internal evidence as two coherent commits.

## Checkpoint C2 — Unified transfers and safe host escape

Tasks: 9–13.

**Why:** normal return hooks, exceptional routing and native-floor escape must integrate before any public cleanup can rely on them. **Entry:** C1 COMPLETE. **Source of truth:** rooted Transfer/destination and explicit host boundary; existing FrameToken for lexical return.

**Working set — primary:** `phalcom-core/src/vm/{control,dispatch,send}.rs`, `phalcom-core/src/error.rs`; retained epilogues in `primitive/{map,set,mod,block}.rs`. **Secondary:** `diagnostics/traceback.rs`, `modules/initialize.rs`, `vm/api.rs`, `interpret.rs` only to reject transfer leaks. **Out of scope:** child failure cascade and public source APIs.

**Contract:** transfers cross VM controls and retained native floors exactly once; no code runs beyond an abandoned callback except required host housekeeping; target frames survive until relevant cleanup can run.

**Risks/hostile cases:** user return through hash/equality; error at a zero-IP frame; depth exhaustion during handler admission; callbacks returning normally at the same depth; nested host descriptors; a private escape mistaken for a catchable Error.

### Task 9 — Intercept ordinary callback completion and every error exit

**Purpose:** make the reducer the continuation owner. **Risk:** semantic HIGH; fanout dispatch/control. **Owner:** `run_until_inner`, Return opcode, new `route_transfer/drive_control` helpers. **Dependencies:** C1. **Inspect:** entry exhaustion and all `return Err`/`?` exits in dispatch; caller result convention. **Do not inspect:** unrelated opcode semantics.

**Current → target:** a Result exits the interpreter and Rust on/ensure catches it → a Result becomes a rooted transfer offered to VM controls before leaving the appropriate execution floor.

**Edit operations (STRUCTURAL):**
1. Introduce an outer iterative drive layer around the existing instruction loop or extract one execution segment that returns a typed stop. Preserve one hoisted Callable per uninterrupted bytecode segment; do not clone it per instruction.
2. Ensure every runtime error leaving an instruction segment reaches the router, including rest dispatch, access errors and immediate native errors. Do not individually patch only throw/raise.
3. On Return, capture its destination before popping; close upvalues and truncate the callee window; publish its return to the exact control/operand destination.
4. On failure, retain error and capture relevant frame records before unwind, then route within current host floor. A root/host driver returns only when it owns the exhausted destination.
5. Drain pending controls before the no-frames test; otherwise an immediate final cleanup could be dropped as a finished program.

**Must not:** execute the caller's next instruction before pending Raise delivery, or push one result in both the opcode and reducer. **Testing:** C2 combined router tests. **State:** all opcode exits share the route, plus performance-preserving loop boundary.

### Task 10 — Replace eager non-local return with an unwind cursor

**Purpose:** run crossed cleanup before destroying its required state. **Risk:** semantic HIGH; fanout dispatch/control/frame consumers. **Owner:** `Bytecode::ReturnNonLocal`, `unwind_to`, `close_upvalues_from`, new transfer cursor. **Dependencies:** Task 9. **Inspect:** `FrameToken`, `Upvalue::{Open,Closed}`, existing block return fixtures. **Do not inspect:** source parser/lowering; bytecode already carries the home token.

**Edit operations (STRUCTURAL):**
1. Retain the existing pre-mutation home token validity test; record target and popped return Value in rooted pending transfer.
2. Replace bulk frames/stack truncation with a boundary walk using control identity and target index/generation. An inner return targeting a frame below the current callback but still inside an outer ensure does not fire that outer cleanup yet.
3. Before activating cleanup, close/remove only abandoned child-frame upvalues. Keep owner/home locals available while they remain semantically live.
4. After all crossed controls and host floors are handled, close remaining upvalues, remove target home frame and publish its return into the home caller window once.
5. Distinguish missing/dead/cross-Fiber token from a valid target at index zero; preserve DeadFrameError as a real language failure.
6. Route cleanup return replacement by the same mechanism. Do not infer it from post-cleanup frame length.

**Must not:** perform the old truncate first and attempt cleanup from dangling slots. **Testing:** C2 synthetic transfer records plus existing `corpus::blocks`, `corpus::control_flow`; full ensure cases C3. **State:** destroyed versus retained segments and target validation contract.

### Task 11 — Guard synchronous host activation and define its escape envelope

**Purpose:** retain safety around complete host callbacks, including immediate native targets. **Risk:** semantic HIGH; fanout send/error/control/mod. **Owner:** `send_dynamic`, `invoke_method_object`, temporary `block_call`, `PhError`, host-boundary state/root classification. **Dependencies:** Tasks 9–10. **Inspect:** `try_module_export_send_dynamic`, depth checks and error transport size test. **Do not inspect:** all native primitives beyond known callback fanout.

**Edit operations (STRUCTURAL):**
1. Add a bounded HostBoundary stack and opaque ids; record entry Fiber and frame/control floors before invoking a synchronous target.
2. Move native re-entry preflight and increment before stack activation/module-export forwarding. On preflight failure, no callback, pushed frame or queue mutation has occurred.
3. Put target activation and any recursive drive inside an immediately invoked Result-returning closure, then balance depth/descriptor in one outer epilogue on both Ok and Err.
4. Respect explicit CallOutcome: return an immediate completed result without running a drained unrelated frame stack; drive bytecode/control only until this host boundary completes; a switched outcome beneath the guard is an invariant failure.
5. Add an opaque internal escape variant in PhError, carrying only identity; retain its actual Transfer in a traced slot. Annotate its internal purpose and ensure size remains below the existing `runtime_error_transport_stays_below_large_result_threshold` cap.
6. Make nested host driver escape handling validate identity and forward until the outer eligible owner is live. Do not normalize, print or settle it.
7. Preserve public host method signatures `PhResult<Value>` and existing leaf primitive signatures.

**Must not:** decrement native depth only on success; activate a native Fiber call before installing the guard; use a Value sentinel for host escape. **Testing:** C2 host admission and nested escape tests. **State:** final guarded scope and envelope ownership.

### Task 12 — Preserve native housekeeping on escape; fence public errors

**Purpose:** make escaping Rust calls semantically inert but resource-correct. **Risk:** semantic HIGH; fanout retained host consumers and error presentation. **Owner:** `map.rs::locate_key`, `set.rs::locate`, `primitive/mod.rs::{send_hash,send_eq}`, `list.rs::list_to_string`, `system.rs::system_class_print`, `typing.rs::typing_context_construct`, `block.rs` while retained. **Dependencies:** Task 11. **Inspect:** exact result/epilogue order around sends; PhError wrappers. **Do not inspect:** collection layout or new async algorithms.

**Edit operations (STRUCTURAL with exact epilogue anchor):**
1. Preserve `let bucket_result = send_hash(...); exit_reentrant_send(); let bucket = bucket_result?;` and the equality analogue. Escape forwards after unlocking, without selecting buckets/mutating/inserting.
2. Preserve temporary-root truncation on both error and escape. For any legacy block_on retained before C3, special-case escape before its catch-all native-error normalization; legacy ensure retained before C3 must not consume an escaped non-local return as a catchable error or run outer cleanup under a live host floor.
3. Arrange the legacy on/ensure compatibility window explicitly: use the VM transfer router for crossed cleanup or migrate the minimal adapter concurrently; do not leave a checkpoint whose legacy ensure runs twice. If C2 cannot preserve existing semantics independently, integrate C2 and C3 as one evidence gate while retaining task order and record the merge; never claim C2 COMPLETE on a broken compatibility state.
4. Audit `to_display_string`, print/list formatting and TypingKnown constructor wrap to stop post-work after escape. Existing output emitted before the transfer need not be rolled back; nothing after the callback may be emitted.
5. Add guards at VM public error boundaries so an unmatched internal escape reports an internal invariant failure and cannot become a module initializer failure record, E010 event or surface Error. Valid escapes should never reach these fences.

**Must not:** run public cleanup inside an unsafe native callback just because “cleanup always runs”; it belongs after crossing the host floor. **Testing:** C2 retained hash/eq/format non-local-return cases plus ordinary error behavior. **State:** all escape consumers and resource epilogues.

### Task 13 — Prove transfer routing and negative escape boundaries

**Purpose:** establish C2 as a usable semantic boundary. **Risk:** semantic MEDIUM; fanout tests only. **Owner:** internal `vm/control.rs` tests and create `phalcom-core/tests/core/execution/native_control.rs`; register `execution_native_control` in `phalcom-core/tests/core/mod.rs`. **Dependencies:** Tasks 9–12. **Inspect:** core VM helpers and existing depth tests. **Do not inspect:** harness unrelated features.

**Edit operations (EXACT registration / STRUCTURAL assertions):**
1. Register `#[path = "execution/native_control.rs"] mod execution_native_control;` next to depth tests.
2. Add tests named `normal_and_nonlocal_transfer_destinations`, `host_floor_escape_preserves_outer_cleanup`, `immediate_host_switch_is_guarded_before_mutation`, `host_escape_releases_collection_lock`, and `internal_escape_never_becomes_surface_error`.
3. Test a return target inside versus outside a synthetic nested control, normal None/Error data, zero-IP failed child and same-depth immediate control.
4. Retain existing dead-frame tests and add a non-local return whose saved object is GC-reachable only through pending transfer while host escape is in flight.
5. Check balanced native depth/context/boundary/temp roots after both successful and failed calls using crate-local tests where internals are needed; do not expose production debug API solely for an integration assertion.

**Testing:** all run at C2 boundary; no per-fixture repeated full suite. **State:** result/destination matrix and any C2+C3 gate merge.

**Required evidence:** `--lib vm::control::tests`; `--test core execution_native_control`; `--test core execution_depth_limits`; `--test language-corpus corpus::blocks -- --exact`, `corpus::control_flow`, `corpus::errors`. If legacy on/ensure cannot consume the new transfer safely before C3, explicitly merge C2/C3 gate and run these with C3; this is the only pre-authorized gate merge.

**Negative gate:** source search shows host escape is intercepted before error normalization, and no ReturnNonLocal bulk truncation bypasses a crossed control. Every direct synchronous adapter balances scope ahead of activation.

**Do not run yet:** workspace and broad corpus → C8; child failure semantic assertions → C4.

**Escalate immediately if:** internal escape must be exposed as a public source error; module boundaries need semantic changes; cleanup requires restoring already destroyed slots; no bounded compatibility transition exists.

**Completion:** [ ] Tasks 9–13 integrated; [ ] destination/host/negative tests pass; [ ] preserved legacy semantics or explicit C2+C3 merged gate; [ ] state updated; [ ] no INCIDENT. **Commit grouping:** transfer router plus synchronous host adapters and their tests; avoid a commit that introduces an uncaught transfer marker.

## Checkpoint C3 — Suspendable protection and cleanup

Tasks: 14–17.

**Why:** protected body/matching/handler, raised-message callback and cleanup phases are one user-visible error-control contract. **Entry:** C2 COMPLETE, or its explicitly recorded merged C2+C3 integration. **Source of truth:** VM control phases and existing error-handling rules.

**Working set — primary:** `primitive/{block,error}.rs`, `vm/control.rs`, native protection fixture lane. **Secondary:** `core/universe/src/callable/closure.ph`, `errors/error.ph`, native descriptor contracts; no signature changes intended. **Out of scope:** parent cascade and fallback callbacks.

**Contract:** on/ensure may yield in a manual coroutine or await under scheduler ownership, preserving result/error identity and cleanup precedence.

**Risks/hostile cases:** catch matching suspends; handler recursively throws; cleanup runs twice; Error-as-data is caught; cleanup replaces a pending return; initial class validation mutates stack; depth error cannot allocate a handler frame.

### Task 14 — Convert on admission and dynamic matching to VM control

**Purpose:** remove body/match/handler host recursion. **Risk:** semantic HIGH; fanout block/control/native contract. **Owner:** `primitive/block.rs::block_on`, Protect phases in control module. **Dependencies:** C2 router/outcome product. **Inspect:** current Class validation, `capture_frames`, synthetic base Error and `is(_)` send. **Do not inspect:** parser try desugaring unless explicit syntax differs in tests.

**Edit operations (EXACT signature direction; STRUCTURAL state construction):**
1. Change the primitive attribute to include `abi = shape`; retain owner Closure and selector `on(_,_)`.
2. Change `fn block_on(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value>` to `fn block_on(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome>`.
3. Read class and handler with positional access through ArgumentView; validate Class before installing/body execution. Admit ProtectBody with body receiver, catch class, handler, destination, source and saved caller authority.
4. On real Raise, preserve current normalization and traceback capture; unwind failed body before match so depth capacity is available. Keep internal host escape out of this branch.
5. Schedule `is(_)` with exact encoded selector via ordinary dispatch. On true, schedule handler with one Error argument. Remove/disable own catch eligibility before matching/handler execution.
6. Remove recursive body/handler block_call and send_dynamic from block_on; preserve public result semantics and dynamic match overrides.

**Must not:** replace matching with nominal class comparison or catch handler/match failure locally. **Testing:** C3 lane and existing errors. **State:** field/selector identities and removed re-entry sites.

### Task 15 — Make Error.raise message evaluation VM-visible

**Purpose:** let an overridden message callback suspend without hiding another native continuation. **Risk:** semantic HIGH; fanout error/control. **Owner:** `primitive/error.rs::error_raise`, RaiseAfterMessage state. **Dependencies:** Task 14 error path and C2 router. **Inspect:** current `message` send and `to_string(&VM)` conversion, error subclass fixture syntax. **Do not inspect:** all rendering operations; generic print/list remain residual guards.

**Edit operations (STRUCTURAL):**
1. Change error_raise to existing shape ABI, retaining `Error`, `raise()` and source declaration's Never contract.
2. Root original error receiver and capture call destination before scheduling ordinary `message` dispatch with saved caller authority.
3. After ordinary message completion, preserve current non-dispatch `Value::to_string(&VM)` conversion, construct RuntimeError::Raise with original Error and rendered snapshot, and route it as Raise.
4. If message evaluation raises or non-locally returns, propagate that transfer rather than constructing the original Raise afterward. If it parks, keep receiver/state rooted.
5. Update stale native re-entry comment; do not change `to_display_string` into an implicit suspendable API.

**Must not:** return Error as data from raise or force cleanup failure into the original error identity. **Testing:** C3 message override, await/yield and Error identity tests. **State:** exact failed-message precedence.

### Task 16 — Replace ensure with saved transfer phases

**Purpose:** implement exactly-once cleanup on all exits. **Risk:** semantic HIGH; fanout block/control/unwind. **Owner:** `block_ensure`, EnsureBody/Cleanup, existing saved outcome GC tests. **Dependencies:** C2 transfer router; Tasks 14–15 for nested error cases. **Inspect:** current cleanup-supersedes and ReturnNonLocal behavior; existing `errors_ensure_all_exits.ph`. **Do not inspect:** cancellation or resource library design.

**Edit operations (STRUCTURAL):**
1. Convert ensure to shape signature like block_on; retain selector `ensure(_)` and Closure owner.
2. Admit body and cleanup, with explicit original destination. A callback may be any callable accepted by `activate_function`; validate at the correct invocation point to preserve error timing.
3. On body transfer, save complete outcome before changing phase; mark EnsureCleanup before entering cleanup.
4. Run cleanup as ordinary callback. On normal completion discard its result and restore saved transfer; on Raise/non-local return drop saved transfer and continue replacement.
5. Remove temp-root-depth local, direct block_call and frame-shrink outcome classification. Replace with traced phase/pending state; balance any temporary staging roots before gateway returns.
6. Preserve non-local-return target liveness until cleanup has had its chance to run. Nested ensures follow inside-out router ordering.
7. Ensure errors during cleanup activation itself obey cleanup-supersedes; no second cleanup retry is permitted.

**Must not:** publish Fiber terminal result or observer while cleanup is parked. **Testing:** C3 full precedence and live-capture cases. **State:** exact saved transfer lifetime and phase transition evidence.

### Task 17 — Prove protection/cleanup contract and replace obsolete guard vehicles

**Purpose:** establish public suspension semantics without losing negative safety coverage. **Risk:** semantic MEDIUM; fanout source fixtures plus existing GC tests. **Owner:** new `concurrency/native/protection/` and negative lane; `concurrency_fiber_restricted_yield_guard.ph`; `negative/fiber_resume_gate_call_native_frame.ph`; `phalcom-core/tests/core/memory/gc.rs`. **Dependencies:** Tasks 14–16. **Inspect:** current fixtures/expected text and actual source syntax. **Do not inspect:** new harness abstractions.

**Edit operations (STRUCTURAL cases; expected observations fixed):**
1. Add `on_yield_resume_and_failure.ph`: body/handler repeated yield via manual Fiber, delivered resume values, failure after resume, exact handler identity.
2. Add `on_pending_await.ph`: scheduled action waits, root pumps until parked, asserts Future not ready, settles gate, pumps and verifies returned/caught value. Include rejected await and nested on.
3. Add `ensure_precedence.ph`: all table rows/columns including non-local return; use distinct error objects and trace log for exactly-once, inside-out order.
4. Add `ensure_body_and_cleanup_await.ph`: independently pending body/cleanup gates; assert action still pending while cleanup parks and captured local mutations persist.
5. Add `match_handler_message_suspend.ph`: dynamic `is`, handler, Error.message callbacks each suspend in appropriate owner mode; failures propagate outward.
6. Rewrite original native guard test vehicle to a real retained host callback (e.g. Map key hash), preserving its intended guard assertion. Move the now-legal on-handler Fiber.call example to PASS coverage; do not leave it under a runtime-negative registration.
7. Extend existing ensure GC tests only where needed now; parked/reclamation strength is C6. Assert actual forced collection later rather than treating invariant verification alone as a GC proof.

**Testing:** C3 boundary. **State:** expected-red baseline cases now PASS, old negative vehicle replaced and exact expectations explained.

**Required evidence:** `--test language-corpus corpus::concurrency_native_protection -- --exact`, `corpus::concurrency_native_negative`, `corpus::errors`, `corpus::concurrency`, `corpus::concurrency_negative`; `--test core memory_gc::ensure_` filter; C2 evidence if gates merged. Error/None-as-data and all cleanup table cases are mandatory.

**Negative gate:** `rg -n 'block_call\(|send_dynamic\(' phalcom-core/src/primitive/block.rs phalcom-core/src/primitive/error.rs` finds no on/ensure/raise invocation sites; whileTrue and temporary helper definition may remain until C5/C8.

**Do not run yet:** workspace and full corpus → C8; parent Call failure cases → C4.

**Escalate immediately if:** on/ensure dispatch needs new syntax, source signature/descriptor identity diverges, cleanup cannot run at the depth bound, or revised negative tests only pass because the guard was weakened.

**Completion:** [ ] Tasks 14–17 implemented; [ ] public/hostile/GC regressions pass; [ ] old error-control host invocation removed; [ ] state and negative fixtures reconciled; [ ] no INCIDENT. **Commit group:** protected and cleanup continuation migration, raise message path, matching tests/expected outputs.

## Checkpoint C4 — Child failure re-enters the parent's continuation

Tasks: 18–20.

**Why:** representing parent cleanup is insufficient while Fiber failure discards ancestors before those records can execute. **Entry:** C3 COMPLETE. **Source of truth:** child terminal state and parent pending Raise in its restored continuation.

**Working set — primary:** `vm/dispatch.rs::{run_until,switch_to_fiber_and_deliver,enqueue_completion_observer}`, `vm/control.rs`, `primitive/fiber.rs` restore boundary; child fixture lane. **Secondary:** `vm/walk.rs`, traceback source and E010 record helpers. **Out of scope:** new generator consumer/await protocol and cancellation.

**Contract:** failed Call child raises at parent call site; Try/Scheduler keep isolation; parent terminal publication waits for its own cleanup.

**Risks/hostile cases:** eager ancestor failure survives as fallback; Error data marked failed; parent cleanup parks and gets a duplicate observer; child trace frames duplicated; an ancestor becomes independently schedulable while a descendant still runs.

### Task 18 — Separate restoring a Fiber from value/error delivery

**Purpose:** deliver an exception without a fake success value. **Risk:** semantic HIGH; fanout dispatch/control. **Owner:** `switch_to_fiber_and_deliver`, new typed delivery/raise helper. **Dependencies:** C3 controls. **Inspect:** current target BlockedOnChild assertion, resume_slot and load_live_from. **Do not inspect:** public Fiber API redesign.

**Edit operations (STRUCTURAL):**
1. Extract common target restore/status/trace setup, preserving its exact ownership assertions.
2. Dispatch successful delivery by FiberResumeDestination: Operand truncates at its slot and pushes one real Value; Control truncates its staging window and queues Returned(value) to its validated waiting ControlId without pushing a result operand. Consume/reset the tag once. Reuse this delivery path from fiber_resume's already-started callee branch as well as switch_to_fiber_and_deliver, so manual and scheduler resumes agree.
3. Add exception delivery truncating the abandoned call receiver window but pushing no value; install rooted pending Raise targeted at the operand call site or exact waiting ControlId from the tagged destination. Share tag consumption with success delivery.
4. Restore control buffers before routing; set current and Running coherently; preserve original parent frame IP for traceback and normal next instruction after a catch.
5. Restart dispatch control processing before fetching any bytecode in parent.

**Must not:** pass None to the existing value helper and then guess that it represented failure. **Testing:** C4 delivery tests. **State:** delivery enum/helper and destination ownership.

### Task 19 — Replace the eager Call-mode ancestor failure cascade

**Purpose:** let each parent process its own on/ensure. **Risk:** semantic HIGH; fanout terminal dispatch. **Owner:** `run_until` Err arm, `mark_fiber_failed`, observer enqueue and E010 boundary. **Dependencies:** Task 18. **Inspect:** current `failed = resumer` loop, error normalization/capture, close_fiber_upvalues_from, `had_completion_owner`. **Do not inspect:** Future generic source implementation.

**Edit operations (STRUCTURAL):**
1. On an unhandled local Raise, terminalize only current Fiber after local controls empty. Capture/normalize error once for that terminal event before clearing live state.
2. Preserve error/result fields, close upvalues, clear terminal buffers, capture resume mode/observer ownership and detach observer once.
3. For Call, replace `failed = resumer` cascade with restore-and-inject Raise to parent. Do not clear parent's parked frames/control state or mark it Failed.
4. For Try/Scheduler, preserve Error value delivery and unowned Scheduler failure reporting; no parent exception injection.
5. Add logical FiberBoundary and parent call-site trace information once per actual crossing; preserve an existing Raise's error identity and help. Avoid accidental duplicate synthetic base Error creation in capture/normalization.
6. A parent that fails later comes back through the same one-Fiber terminal path. Remove obsolete eager ancestor clearing code only after tests cover multi-parent propagation.

**Must not:** treat scheduler waiter registration as a completion owner or double-report an observed failure. **Testing:** C4 child/cascade/E010 tests. **State:** terminal ordering and removed cascade branch.

### Task 20 — Prove parent protection, cleanup and failure isolation

**Purpose:** validate cross-Fiber continuation semantics. **Risk:** semantic MEDIUM; fanout test lane and internal dispatch tests. **Owner:** `concurrency/native/child/`, existing `vm/dispatch.rs` observer tests. **Dependencies:** Tasks 18–19. **Inspect:** existing call/try chains and current unowned failure fixtures. **Do not inspect:** new source Task APIs.

**Edit operations (STRUCTURAL tests):**
1. `child_failure_parent_catches.ph`: child throws after yield, parent catches at child.call, parent continues; assert child Failed and parent succeeds.
2. `child_failure_parent_cleanup_await.ph`: scheduler-owned parent calls failing child, enters ensure and awaits gate; assert parent Future not settled until cleanup completes; outer parent catch observes failure afterward.
3. `child_failure_nested_parents.ph`: multiple Call ancestors, inner nonmatching on and nested ensure, outer matching handler, exact cleanup order and one error identity.
4. `child_try_scheduler_isolation.ph`: same raised Error under Call/Try/Scheduler; only Call injects Raise. Return an Error normally in companion case; state remains success.
5. Add internal observer-once and terminal-buffer-clear assertions; preserve E010 owned/unowned distinction.
6. Add ancestor admission rejection during active child and parked cleanup; no queue entry is produced for BlockedOnChild.

**Testing:** C4 gate; no workspace. **State:** delivery mode matrix and observer counts.

**Required evidence:** `corpus::concurrency_native_child -- --exact`; `corpus::concurrency -- --exact` for existing E010/Future ownership fixtures; relevant `--lib` observer tests found in `vm/dispatch.rs`; `--test core observability_fiber_trace` and `observability_traceback` because logical crossing changed.

**Negative gate:** no Call-mode loop eagerly marks a parked ancestor failed before routing its pending Raise. The remaining terminal clearing applies only to a genuinely terminal Fiber.

**Do not run yet:** complete core/workspace → C8; actual parked GC adversary → C6.

**Escalate immediately if:** parent cleanup needs a generator consumer protocol, error injection changes Try/Scheduler public behavior, or observer ownership becomes tied to current resumer.

**Completion:** [ ] Tasks 18–20; [ ] cross-Fiber hostile tests pass; [ ] eager ancestor bypass removed; [ ] exact trace/observer evidence recorded; [ ] no INCIDENT. **Commit group:** Call-mode exception injection and cross-Fiber error-control evidence.

## Checkpoint C5 — Suspension-transparent native control fallbacks

Tasks: 21–24.

**Why:** ordinary inlined syntax can pass while real selector sends still re-enter Rust. This gate establishes the callback protocol behind explicit/deoptimized sends. **Entry:** C4 COMPLETE. **Source of truth:** canonical Function activation, strict Bool/Option semantics and existing selector layouts.

**Working set — primary:** `primitive/{boolean,option,block}.rs`, `vm/control.rs`, `vm/dispatch.rs::ValidateOrdering`, fallback fixture lane, depth tests. **Secondary:** native metadata and compiler sacred-deopt tests in `universe/mod.rs`; preserve compiler behavior. **Out of scope:** collection traversal redesign and general native arithmetic changes.

**Contract:** only chosen branches run; post-callback transformations execute after actual completion; condition/body suspension preserves loop state and returns None on termination.

**Risks/hostile cases:** inlining masks old path; one-armed Bool loses Some wrapper; nested None flattened; loop's false condition still runs body; reflected ordering bypasses result validation; tail callback immediately switches Fiber.

### Task 21 — Tail-forward Bool and Option branches through shape activation

**Purpose:** remove host calls that need no native continuation. **Risk:** semantic MEDIUM; fanout primitives/control tests. **Owner:** `boolean.rs::{bool_and,bool_or,bool_if_true_if_false}`, `option.rs::option_match`. **Dependencies:** C1 canonical outcomes, C3/C4 integrated router. **Inspect:** exact selector annotations, positional/labeled argument access and chosen payload. **Do not inspect:** semantic Bool/type relation code.

**Edit operations (STRUCTURAL with exact owner names):**
1. Convert signatures to shape ABI, retaining selector metadata/intrinsic ids and return contracts. Do not remove BoolAnd/BoolOr intrinsic association.
2. Short-circuit and/or return the same immediate Bool value as before when bypassing callback. Paired conditional chooses exactly one callable.
3. Construct chosen callable's ordinary invocation window at the original receiver index, then forward through `activate_function`; preserve saved original caller authority. No new ControlActivation is required for pure tail forwarding.
4. Option.match reads structural labels `some` and `none` using the existing ArgumentView mapping; Some branch receives one payload, None branch receives zero. Preserve nested Option payload depth.
5. Remove `block_call` imports if no remaining function in that module uses them. Update test-only override implementation to a shape primitive if it is intended to remain suspension-transparent.

**Must not:** unpack labels as arbitrary physical positions without matching descriptor order; invoke the unselected branch for validation; manufacture a continuation allocation on every short circuit. **Testing:** C5 explicit/deopt/Family cases. **State:** pure-tail migrated sites and metadata parity.

### Task 22 — Add WrapSome and whileTrue control phases

**Purpose:** retain actual post-callback work without a live Rust loop. **Risk:** semantic HIGH; fanout boolean/block/control. **Owner:** `bool_if_true`, `bool_if_false`, `block_while_true`, WrapSome/While phases. **Dependencies:** Task 21 and control router. **Inspect:** existing `wrap_some`, strict Bool error and `none_value`. **Do not inspect:** rewrite of source Iterable/for implementation.

**Edit operations (STRUCTURAL):**
1. Convert one-arm Bool primitives to shape; skipped branch returns None immediately. Taken branch admits WrapSome with actual callback outcome destination.
2. On ordinary callback completion call existing `wrap_some`; errors/non-local returns propagate. Some(None) and Some(Error) are valid returned data, not unwrapped sentinels. Wrap overflow remains a real runtime failure.
3. Convert whileTrue to a condition phase and body phase; retain both callable values in control record.
4. On condition completion require `as_bool()`. False completes with `vm.none_value()`. True schedules body; body ordinary value is discarded and next condition scheduled.
5. Any condition/body Raise/non-local return routes outward. Suspension keeps its current phase. Immediate-only loops must pass coherent safepoints; no recursive reducer or Rust loop invoking block_call.
6. Preserve fallback selector dispatch and sacred override invalidation; no compiler inliner rewrite is needed.

**Must not:** use truthiness; continue a loop after callback non-local return; wrap skipped branch in Some(None). **Testing:** C5 semantic matrix and bounded nesting. **State:** phase machine and return/wrapper rules.

### Task 23 — Make ordering reversal use ordinary dispatch and validation

**Purpose:** remove user reverse dispatch beneath an opcode's live Rust continuation. **Risk:** semantic MEDIUM; fanout control/dispatch. **Owner:** `Bytecode::ValidateOrdering { reverse }` in `vm/dispatch.rs`, ValidateReversedOrdering state. **Dependencies:** outcome/router. **Inspect:** current pre/post class validation, canonical `semantic_roots.ordering_class`, source range and stack result destination. **Do not inspect:** redesign bilateral operator lookup or Ordering types.

**Edit operations (STRUCTURAL):**
1. Preserve initial Ordering/subclass validation. Keep reverse=false path synchronous and allocation-free beyond current behavior.
2. For reverse=true, root popped ordering receiver in a control state before yielding to ordinary selector activation of `reverse`.
3. After ordinary return, validate result against the same canonical Ordering root and subclass relation, then push exactly once at the saved operand destination.
4. Propagate callback error/non-local return and suspension through router; no direct `send_dynamic` in the opcode arm remains.
5. Preserve caller access/source range and invalid-result diagnostic text.

**Must not:** weaken second validation to accept arbitrary Value or replace the Ordering root with a string-name check. **Testing:** C5 reversed compare, invalid result and suspension cases. **State:** opcode no longer owns post-send state in Rust.

### Task 24 — Cover real fallback dispatch and move depth guards to real residual paths

**Purpose:** defeat inlining and preserve native guard/depth evidence. **Risk:** semantic MEDIUM; fanout tests and test-only native registrations. **Owner:** `concurrency/native/fallbacks/`, `phalcom-core/tests/core/execution/depth_limits.rs`, `phalcom-core/src/universe/mod.rs` deopt/injected native tests. **Dependencies:** Tasks 21–23. **Inspect:** existing `kernel_option_match_override_reroutes_every_combinator`, sacred override tests, `native_reentry_is_bounded`, `traceback_survives_a_frame_that_executed_nothing`. **Do not inspect:** global benchmark infrastructure yet.

**Edit operations (STRUCTURAL):**
1. Add explicit selectors and callable references/perform invocations that bypass simple compiler inlining; verify chosen branch count, and/or short-circuit result, Some wrapper, Some(None), Error-as-data.
2. Exercise whileTrue condition and body independently yielding/awaiting, then returning/failing; strict Bool invalid result must still fail. Include a false-first loop and a callback with captured mutable counter.
3. Exercise bound native callable/immediate control and Family gateway routes, including Fiber switching; they must propagate EnteredControl/SwitchedFiber correctly. Use a bound native Fiber.call or Fiber.yield directly as an ensure/on/whileTrue callback and verify successful switch-back advances the exact phase and cleanup runs once. Also yield inside a bytecode callback under ensure to prove the direct-activation marker is cleared and does not prematurely complete ensure.
4. Reuse existing sacred deopt/override tests without changing their expected override results; new tests verify native fallback itself can suspend.
5. `native_reentry_is_bounded` currently uses recursive native whileTrue; after migration change that test's vehicle to a genuinely retained hash/toString host chain. Keep assertion for MAX_NATIVE_REENTRY on that new native chain.
6. Add a separate assertion that recursive whileTrue no longer grows native depth and eventually meets ordinary/control depth policy. Update the zero-IP traceback test's vehicle likewise; do not just change every expected limit to whichever trips.
7. Keep Option override test either shape-aware or explicitly marked as a deliberate negative native injection; no obsolete block_call import may survive accidentally.

**Testing:** C5 combined gate. **State:** old depth test vehicle and new reason, deopt coverage, migrated direct-call inventory.

**Required evidence:** `corpus::concurrency_native_fallbacks -- --exact`; existing `corpus::booleans`, `corpus::absence`, `corpus::option`, `corpus::control_flow`; `--test core execution_depth_limits`, `execution_family_runtime`; specific `--lib` sacred deopt/Option override test names discovered by source/list. Include reversed ordering corpus label found in its current registration before running, rather than inventing a filename filter.

**Negative gate:** no production `block_call(` uses in boolean/option or block on/ensure/whileTrue; no `send_dynamic` in ValidateOrdering or Error.raise. Remaining helper definition/intentional tests are resolved at C8.

**Do not run yet:** full package/workspace → C8; cross-Fiber GC/trace matrix → C6.

**Escalate immediately if:** preserving fallback semantics requires new source syntax, duplicate overload resolution, changing Option physical representation or blanket native ABI migration.

**Completion:** [ ] Tasks 21–24; [ ] explicit/deopt/hostile tests pass; [ ] migrated recursive callers absent; [ ] state records retained native test vehicle; [ ] no INCIDENT. **Commit grouping:** tail branch migration; post-callback loop/wrapper/ordering migration with their fixtures, in one checkpoint evidence boundary.

## Checkpoint C6 — Lifetime and bootstrap integration

Tasks: 25–27.

**Why:** synthetic storage tests do not prove actual suspended cleanup preserves values, traces, authority or terminal release. **Entry:** C5 COMPLETE. **Source of truth:** live/parked control owner, traced Values and existing native/source identity.

**Working set — primary:** `vm/{control,gc,dispatch,walk}.rs`, `heap/trace.rs`, `phalcom-core/tests/core/{memory/gc,execution/native_control,observability/traceback}.rs`, native contract tests. **Secondary:** native install descriptors and Universe callable/scalar/option/error declarations. **Out of scope:** GC algorithm redesign, unrelated source baseline regeneration, new semantic products.

**Contract:** records survive actual suspension/collection and stop retaining objects at completion; bootstrap, runtime and reflection agree on the migrated primitive identity and invocation shape.

**Risks/hostile cases:** duplicate root in test hides missing control root; module global keeps saved value alive; error context carries non-Raise handles; cleanup changes self/captures; private access becomes allowed after native context popped; stale native diagnostic frame appears after resume.

### Task 25 — Force collection while real control continuations are parked

**Purpose:** prove lifetime across executor activity, not just enum tracing. **Risk:** semantic HIGH; fanout tests/owner fixes only. **Owner:** extend `phalcom-core/tests/core/memory/gc.rs`, internal control trace tests. **Dependencies:** C5. **Inspect:** existing ensure GC tests, forced GC API safety, Future waiter rooting. **Do not inspect:** heap collector internals absent contrary evidence.

**Edit operations (STRUCTURAL tests):**
1. Add `ensure_saved_value_survives_parked_collection`: create body result in local scope, park cleanup on a gate, collect from another safe execution boundary, then resume and observe exact saved object.
2. Add saved Raise and saved non-local-return value counterparts; ensure no unrelated global/root retains the tested object. Use internal tests for opaque handle-only reachability where source cannot eliminate incidental roots.
3. Add retained callable/receiver/self/class-authority and captured mutable local cases; escape a closure from failed body and verify closed upvalue after cleanup/return.
4. Add collection during ProtectMatch and nested host escape, including SelectorPatternMismatch's Family/receiver payload if stored raw.
5. Assert the collection actually ran (collection/swept counter or explicit safe `collect_garbage`), not merely that allocation exceeded a guessed threshold.
6. Assert saved outcome/callback handles can be reclaimed after terminal record/waiter references are cleared. Strong references retained intentionally by returned values are excluded from reclamation assertions and documented.

**Must not:** call forced GC mid-native while fresh values are only Rust locals; keep every old result in temp_roots. **Testing:** C6 GC gate. **State:** surviving sole roots, actual collection evidence and release observations.

### Task 26 — Validate authority, traces and terminal cleanup

**Purpose:** ensure control migration does not create privilege/diagnostic/retention drift. **Risk:** semantic HIGH; fanout control/trace tests. **Owner:** `execution_native_control`, `observability_traceback`, `vm/control.rs` invariant checks, terminal clearing paths. **Dependencies:** Task 25. **Inspect:** `ArgumentView::caller_authority`, native method contexts, `foreign_receiver_guard`, StackWalk/capture_frames/capture_parked_frames. **Do not inspect:** source access checker unless runtime tests reveal a cross-layer mismatch.

**Edit operations (STRUCTURAL):**
1. Assert both running and parked control invariants at suspend/resume/complete points, including unique ids/destinations and valid owner tokens.
2. Test public/private/internal callback dispatch under on/ensure before and after a switch; authority must match ordinary invocation. Include a reflected bound method with incompatible foreign receiver layout.
3. Verify same closure in two Fibers at different IPs continues correctly; no hoisted IP/native context is restored from another Fiber.
4. Assert logical traceback for body, match, handler, cleanup, parent child-call failure and cleanup superseding failure. Internal phases contribute no fake user frames; native_selector/class are not stale.
5. Verify Done/Failed Fibers have no active controls/pending transfers/parked execution buffers, observers detached once, and no abandoned checking/open-upvalue state.
6. Preserve terminal result/error intentionally stored for observation; do not clear legitimate result to pass a retention test.

**Must not:** weaken access checks or rewrite all trace snapshots without explaining exact logical frame changes. **Testing:** C6 execution/observability gate. **State:** privilege and trace invariants, final release fields.

### Task 27 — Check native descriptor and source-bootstrap consistency

**Purpose:** preserve one callable declaration and ABI association across bootstrap tiers. **Risk:** semantic MEDIUM; fanout native contracts/tests. **Owner:** `phalcom-core/tests/core/native/contracts.rs`, object-model/reflection tests; primitive attributes; `phalcom-native-macros/src/lib.rs` read-only unless proven necessary. **Dependencies:** all primitive migrations. **Inspect:** macro `abi_check/entry_tokens`, core native install paths, `callable/closure.ph`, `scalar/bool.ph`, `option/option.ph`, `errors/error.ph`. **Do not inspect:** generic semantic changes unrelated to ABI metadata.

**Edit operations (EXACT descriptor rule; STRUCTURAL tests):**
1. Reuse `#[primitive(..., abi = shape)]` and its existing PrimitiveShapeFn compile-time check. Do not introduce a third metadata ABI to represent VM control.
2. Verify owner/selector/visibility/arity/labels/intrinsic ids and source return types remain identical to baseline; only implementation entry ABI changes.
3. Use native_vm/VM::new_native for tests that require primitives without source methods, and universe_vm/VM::new for authored Function/Future/try integration. Do not ask kernel_vm to provide source protocols.
4. Check reflection reports native metadata with unchanged selectors, no duplicated overload and unchanged source declaration association.
5. If Universe source contract changes are unexpectedly necessary, stop and classify why; do not regenerate `semantic-diagnostics-baseline.txt` to suppress introduced errors.

**Must not:** mutate semantic type inference or perform cold/incremental tests unrelated to a runtime-only change. If declaration/query products actually change, reopen scope and add one owner-layer cold/incremental equivalence test then. **Testing:** C6 native/reflection gate. **State:** descriptor parity and exact bootstrap tier used.

**Required evidence:** `--test core memory_gc`, `execution_native_control`, `observability_traceback`, `native_surface_contracts`, `object_model_invariants`, `reflection_conformance`; exact trace internal cases as needed. Run each family once at C6 after all lifetime changes. Avoid re-running C1 synthetic checks unless owner implementation changed.

**Negative gate:** no VM-global temp-root lifetime for a parked control; no terminal control record retains its prior phase; no native descriptor change masquerades as a source declaration replacement.

**Do not run yet:** broad feature/build/workspace → C8.

**Escalate immediately if:** a test requires a new permanent global GC root, a cleanup captures slots already destroyed, private privilege leaks or source/native identity must change.

**Completion:** [ ] Tasks 25–27; [ ] actual suspended GC and release evidence pass; [ ] authority/trace/native parity pass; [ ] state records root ledger and bootstrap tier; [ ] no INCIDENT. **Commit grouping:** integration regressions and narrowly required lifetime/trace fixes together.

## Checkpoint C7 — Reusable readiness boundary

Tasks: 28–30.

**Why:** reactor readiness is a validation/admission boundary, not another execution model. **Entry:** C6 COMPLETE. **Source of truth:** current parking generation and VM queue reservation.

**Working set — primary:** `vm/mod.rs::{wake_parked_fiber,enqueue_unowned_fiber,pop_next_queued}`, `primitive/fiber.rs::{fiber_prepare_park,fiber_park}`, `primitive/system.rs::system_wake`, Future.await and root drain; state document. **Secondary:** GC ownership tests if a new retained registration is introduced. **Out of scope:** actual external registrations, poller, timers, threads, cancellation/queue revocation.

**Contract:** valid current ticket transitions Parked→Queued once; wake does not execute; root queue exhaustion is documented as runnable quiescence only in a future external-source architecture.

**Risks/hostile cases:** duplicate wake advances code twice; stale generation wakes a new await; generation overflow; invalid/freed handle dereference; BlockedOnChild admitted; empty queue reported as global completion with imagined external work.

### Task 28 — Preserve exact parking admission and episode identity

**Purpose:** keep the new continuation compatible with existing wait ownership. **Risk:** semantic MEDIUM; fanout fiber/control tests. **Owner:** preparePark/park, FiberStatus, park_generation. **Dependencies:** C6. **Inspect:** current scheduler-mode restriction, zero/depth-floor guards and checked_add. **Do not inspect:** manual coroutine pending-await redesign.

**Edit operations (STRUCTURAL; prefer no source change):**
1. Verify native-frame refusal remains before waiter registration; a migrated operation no longer adds native depth, but residual calls still do.
2. Preserve checked_add generation exhaustion and stale ticket rejection at park commit; do not replace with wrapping arithmetic.
3. Verify parked controls are part of same Fiber, with original completion ownership independent of resumer and queue.
4. Reuse/extend existing tests for preparation failure leaving no registration, valid current generation and prohibited public resume/schedule.

**Must not:** allow arbitrary yield/await solely because native depth is now zero; scheduler/coroutine ownership remains separate. **Testing:** C7 combined gate. **State:** preserved, changed and intentionally unsupported park behavior.

### Task 29 — Verify source-independent wake admission without inline execution

**Purpose:** establish the external-readiness seam using the existing implementation. **Risk:** semantic MEDIUM; fanout VM/system tests. **Owner:** wake_parked_fiber and existing vm/mod.rs tests. **Dependencies:** Task 28. **Inspect:** exact `(Fiber, generation)` waiter tuple in Future source, ready_queue roots. **Do not inspect:** OS APIs or cross-thread queues.

**Edit operations (STRUCTURAL; production change only if needed):**
1. Keep wake_parked_fiber as lower-level bool-returning validation/admission; remove Future-exclusive wording where it describes no actual Future dependency.
2. Extend its existing generation 6/7 test to old episode→new episode, terminal, BlockedOnChild and duplicate wake; assert queue length/state and no callback side effects before pumping.
3. Preserve live-handle precondition: Future waiter tuple retains Fiber. A fake source test must own/root the Fiber handle rather than call wake on arbitrary stale memory.
4. If the seam is refined, add one fake completion registration test: register/park, submit valid wake, observe zero execution, pump one item, observe continuation result. Otherwise existing VM unit coverage is sufficient; do not manufacture an abstraction for a test.
5. Document future ownership: registration owns a traced handle until unregistration/completion, or validates a generational external id before resolving it. Late parking tickets are rejected after object identity is established.

**Must not:** call resumeScheduled or run_until from system_wake; add a second source-specific queue for future I/O. **Testing:** C7 wake tests. **State:** exact callable seam and handle preconditions.

### Task 30 — Record executor quiescence and future native continuation contracts

**Purpose:** identify precise future attachment points without implementing a reactor. **Risk:** semantic LOW; fanout docs/local comments. **Owner:** state record and scoped comments beside Future.await/root drain/wake; no required new API. **Dependencies:** Task 29. **Inspect:** current empty ready queue paths, failure-report cursor, root result retention. **Do not inspect:** redesign full root lifecycle.

**Edit operations (STRUCTURAL documentation):**
1. Record that pending root await with no runnable work currently raises; root top-level return drains ready work and finishes. Preserve this observable behavior in this patch.
2. Name future extension at those exact branches: executor distinguishes Runnable, IdleWithExternalRegistrations and Complete only once a real registration owner exists; if idle with registrations, poll/inject then resume the same VM loop.
3. Describe I/O leaf→register→return Future/completion→ordinary await→valid wake→queue→later execution. No native Rust invocation spans I/O latency.
4. Describe optional genuinely resumable native continuation as owned GC-traced activation attached to control completion, with explicit resource release; it is future work and leaves existing leaf ABI cheap.
5. State single-executor ownership assumption. Cross-thread completion later submits an owned message to an executor-side validation boundary; it cannot directly mutate VM/Fiber fields.

**Must not:** add a useless idle enum, registration count always zero, reactor dependency or periodic polling loop. **Testing:** no standalone documentation test; C7 existing root behavior cases. **State:** implemented/preserved seam versus future work table.

**Required evidence:** existing and extended `--lib` wake tests in vm/mod.rs (actual names from list); targeted root-await/concurrency fixtures already registered under `corpus::concurrency`; native-control preparePark refusal test if newly added. Every new wait mechanism must prove valid/duplicate/stale/terminal/no-inline-execution; no new mechanism means reuse current evidence.

**Negative gate:** no recursive execution in wake; no second Future-only scheduling authority; no added atomics/locks or reactor dependencies.

**Do not run yet:** real I/O tests excluded; final broad compatibility → C8.

**Escalate immediately if:** source-independent readiness requires cancellation or queue-generation redesign. Document it separately; current cancellation-free queue ownership is the scope.

**Completion:** [ ] Tasks 28–30; [ ] ticket/queue/ownership hostile evidence passes; [ ] extension points explicit; [ ] state separates new/preserved/future; [ ] no INCIDENT. **Commit grouping:** small wake contract refinements/tests and reactor attachment documentation; no forced production refactor.

## Checkpoint C8 — Migration closure and delivery

Tasks: 31–34.

**Why:** replacement behavior is insufficient if old authority can still run, residual guards disappear or deferred gates are forgotten. **Entry:** C7 COMPLETE. **Source of truth:** final source inventory and completed evidence ledger.

**Working set — primary:** all scoped diff, state record, `docs/spec/current/concurrency.md`, affected stale comments/tests. **Secondary:** clean predecessor checkout for an independently reproducible baseline failure. **Out of scope:** unrelated formatting/lint/type fixes without an identified narrow owner.

**Contract:** every migrated control path is free of host recursion; every retained host path remains safely guarded; focused and release evidence are accurately distinguished.

**Risks/hostile cases:** forgotten test-only block_call keeps dead production API alive; negative fixture asserts obsolete on refusal; workspace failure attributed to baseline without reproducing; incomplete tests marked green; performance cost moved to every leaf.

### Task 31 — Remove obsolete re-entry and reconcile restriction documentation

**Purpose:** prevent silent use of the old mechanism. **Risk:** semantic MEDIUM; fanout scoped code/docs. **Owner:** block.rs helper/imports, test injections, native restriction comments, concurrency spec paragraph and checkpoint state. **Dependencies:** C0–C7 evidence. **Inspect:** final rg inventory and scoped diff. **Do not inspect:** unrelated documentation.

**Edit operations (EXACT deletion rule; STRUCTURAL adaptation):**
1. Search production `block_call(`. After migrations, remove `block_call` if no justified production caller exists; update test-only native override with shape activation. Preserve resolve_callable only if another remaining function needs it.
2. Keep send_dynamic/invoke_method_object for enumerated synchronous host callers; their guard is mandatory. Remove obsolete comments claiming ordinary each/on/ensure are native restrictions.
3. Search for Rust-local saved ensure outcome, frame-length shrink classification, ValidateOrdering send_dynamic and `failed = resumer` eager cascade; required expected results are in §7.
4. Update concurrency native-restriction paragraph to name retained hash/equality/printing/list rendering/typing host callbacks and host APIs. Do not declare all native code suspension-transparent.
5. Preserve P1; add as-built evidence only to R1's state file. Reconcile concurrent spec edits by paragraph ownership, never replacing the file wholesale.

**Must not:** remove the guard class or MAX_NATIVE_REENTRY because migrated paths no longer use it. **Testing:** deletion evidence at C8; negative guard suite before broad tests. **State:** final migrated/residual inventory and scope limitations.

### Task 32 — Check hot-path cost and boundedness

**Purpose:** verify the architecture does not tax every leaf. **Risk:** semantic LOW; fanout existing benchmarks/tests. **Owner:** control allocation policy, dispatch hot path, available runtime benchmark harness. **Dependencies:** Task 31 final architecture. **Inspect:** `phalcom-core/Cargo.toml` bench/feature definitions and narrowly relevant existing bench files. **Do not inspect:** redesign benchmark infrastructure.

**Edit operations (STRUCTURAL evidence task):**
1. Record size/layout of new control record and empty ControlStack; empty construction should not heap-allocate. Keep allocation only at actual control admission/growth or existing argument pack creation.
2. Compare existing leaf-native/ordinary-call benchmark on a clean predecessor and implementation using identical configuration; separately measure a control-heavy call/ensure loop and Fiber switch/park/wake workload where harness exists.
3. If no suitable harness exists, use a small reproducible local measurement artifact in checkpoint evidence; do not add a large benchmark framework. Record run count, workload, configuration and raw comparative results; no invented percentage claim.
4. Inspect for new per-leaf allocation/hash lookup/Rc churn/trait dispatch/atomics. A small enum outcome and empty-stack branch are expected; document measured/structural limits separately.
5. Verify deep nested controls meet bounded resource behavior and immediate control loops service GC. Performance correction may not remove guards or weaken semantic tests.

**Must not:** optimize away rooted pending state to recover a benchmark. **Testing:** no duplicated broad suite; focused changed-boundary tests only if measurement prompts code changes. **State:** actual performance evidence and limitations.

### Task 33 — Run final compatibility gates and classify failures narrowly

**Purpose:** complete deferred delivery evidence. **Risk:** semantic LOW; fanout validation only unless diagnosed repair. **Owner:** commands in §6–§8; scoped test failures. **Dependencies:** Task 32. **Inspect:** `.cargo/config.toml`, pinned toolchain, CI command flags and deferred ledger. **Do not inspect:** unrelated packages before a failing path points there.

**Edit operations (EXACT validation schedule):**
1. Run all new native-control corpus lanes together via `corpus::concurrency_native` filter; verify nonzero registrations. Run residual negative cases.
2. Run full core `--lib`, `--test core`, `--test language-corpus`, `--test cli-smoke` once each, serially. Diagnose before expanding if a failure occurs.
3. Run named workspace format/build/test/Clippy gates from §8. Preserve pinned nightly and config; do not add flags weakening diagnostics.
4. If the optional fiber-pool feature exists on the current manifest, run one targeted core/native-control execution under that feature because constructors/terminal clear paths changed. Record vm-trace feature coverage where trace paths changed; choose exact features from the manifest rather than guessing a command.
5. Classify any failure using §8 incident protocol. Verify suspected baseline on an isolated predecessor if needed and without disturbing shared work. Resume only within the recorded repair boundary.
6. Run scoped whitespace/link/metadata checks. No runtime gate is required for subsequent prose-only corrections.

**Must not:** record timeout/cancel/zero-test as PASS, or silently excuse an unrelated failure without evidence. **State:** exact commands, selected counts, results, baselines and release blockers.

### Task 34 — Close state, checkpoint evidence and delivery report

**Purpose:** produce an actionable final handoff. **Risk:** semantic LOW; fanout documentation. **Owner:** R1 state file, C2 checkpoint and program lifecycle metadata only if justified. **Dependencies:** Task 33. **Inspect:** evidence/deferred ledger and final diff. **Do not inspect:** whole repository again.

**Edit operations (EXACT record requirements):**
1. Fill checkpoint summary table with established contract/evidence/status; leave no checkpoint COMPLETE without all required assertions.
2. Reconcile every deferred gate: passed, explicitly removed with rationale, or known release blocker. No unexplained pending gate may vanish.
3. Record final types/APIs, removed host state, protected/cleanup semantics, parent delivery, root/park/wake limitations and exact residual boundaries.
4. Report unchanged P1 hash. If execution commits/pushes are separately authorized, stage only coherent scoped groups; verify final scoped diff and status. Do not infer authorization from this plan.
5. Update lifecycle metadata to reflect actual implementation/evidence; focused-green plus blocked workspace is not RELEASE_COMPLETE.
6. Give next action: reactor work remains excluded; independent C1.P4 Fiber/generator/cancellation work remains separately owned.

**Must not:** call clean/pushed tree behavioral certification. **Testing:** document and evidence consistency only. **State:** no unresolved INCIDENT for COMPLETE; all blockers explicit.

**Required evidence:** complete final gates in §8, negative/deletion searches in §7, performance note with stated limits, scoped diff review.

**Do not run yet:** nothing deferred without explicit exclusion/blocker. No actual reactor tests or speculative thread-safety tests.

**Escalate immediately if:** broad gates reveal a change outside planned runtime/native ownership; diagnose and classify before touching that subsystem.

**Completion:** [ ] Tasks 31–34; [ ] focused and broad required gates complete; [ ] deletion/guard evidence complete; [ ] all deferred entries disposed; [ ] no INCIDENT; [ ] final state/report ready. **Commit grouping:** cleanup/docs; final narrow compatibility fixes with their tests if needed; no blanket staging.

## 5. Concrete regression recipes and risk coverage

### 5.1 Seed fixture: independently yielding body and cleanup

Create `phalcom-core/tests/fixtures/language/concurrency/native/protection/ensure_body_cleanup_yield.ph`. This source follows the current closure/Fiber syntax, but is a proposed regression and has not been compiled during planning. Keep source/expected together.

```phalcom
// area: concurrency
// spec: concurrency.md; error-handling.md §4
// status: PASS
const f = Fiber.new || {
  || {
    Fiber.yield("body")
    "result"
  }.ensure || {
    Fiber.yield("cleanup")
  }
}
System.print(f.call())
System.print(f.call())
System.print(f.call())
```

Expected stdout:

```text
body
cleanup
result
```

This defeats “body can suspend but cleanup cannot” and “cleanup result replaces a normal body result.” Extend a sibling case with resume arguments and mutable capture; do not add a separate suite invocation per edit.

### 5.2 Seed fixture: body and cleanup have different pending waits

Create `.../native/protection/ensure_two_pending_gates.ph` using the current Future constructors and scheduler pump. Proposed source, not planning-time runtime evidence:

```phalcom
// area: concurrency
// spec: concurrency.md; error-handling.md §4
// status: PASS
const bodyGate = Future.new()
const cleanupGate = Future.new()
const result = Future.async || {
  || {
    bodyGate.await
    "result"
  }.ensure || {
    System.print("cleanup-start")
    cleanupGate.await
    System.print("cleanup-end")
  }
}
System.runScheduled()
System.print(result.isReady)
bodyGate.settleValue(1)
System.runScheduled()
System.print(result.isReady)
cleanupGate.settleValue(2)
System.runScheduled()
System.print(result.await)
```

Expected stdout:

```text
false
cleanup-start
false
cleanup-end
result
```

If concurrent Future typing changes require a contextual type annotation, adapt this fixture using current adjacent typed fixtures; do not change runtime ownership or source Future APIs. The regression is pending state **during cleanup**, not a requirement that Future.async execute synchronously or register a particular internal number of scheduler entries.

### 5.3 Regression matrix tied to the checkpoints

Use exact identity assertions in Rust where output equality could hide wrong objects. Source event logs establish control order, not GC ownership by themselves.

| Risk / plausible wrong implementation | Required case | Checkpoint / owner |
|---|---|---|
| No frame means native callback completed | Immediate primitive-bound method admits on/ensure/control without new frame; outer gateway preserves pending disposition | C1/C5, internal outcome + reflection/Family route |
| A native Fiber switch looks like a return | Family/captured gateway reaches Fiber.call/yield with equal frame counts; original stack is untouched after switch | C1/C5 |
| Successful switch-back strands a control phase | Direct bound native Fiber operation used as control callback has no bytecode Return; tagged ControlId delivery advances exactly once, while an ordinary inner bytecode yield remains operand delivery | C1/C4/C5 |
| Frame depth identifies controls | Two nested control operations at same bytecode depth complete into different stable destinations | C1/C2 |
| Entered-control args held only in Rust | Force GC before first callback starts with original receiver/class/callable retained only by the new record | C1/C6 |
| Only throw opcode is routed | Type/arity/access/depth error from an immediate primitive under on/ensure | C2/C3 |
| Non-local return skips cleanup | Body return crosses multiple ensures; logs show inside-out once; home caller receives original value | C2/C3 |
| Return wrongly exits an outer region | Return target lies inside an outer ensure but crosses an inner ensure; outer cleanup runs only on its later actual exit | C2/C3 |
| Cleanup errors lose original identity | Body E1/cleanup success returns same E1; cleanup E2 returns same E2; ordinary returned Error remains data | C3 |
| On catches its own handler | Handler throws matching Error and is caught only by outer region | C3 |
| Dynamic matching is accidentally bypassed | User `is(_)` override logs/suspends or fails; result governs matching as before | C3 |
| Catch remains installed twice after await | Body awaits two gates then throws; exactly one correct handler invocation | C3 |
| Error.message adds hidden re-entry | User message callback yields/awaits; Raise retains same receiver afterward | C3 |
| Native escape becomes a language failure | Non-local return through retained hash/toString crosses host floors and is neither caught by on(Error) nor rejected as a Future failure | C2 |
| Escape skips required Rust cleanup | Hash/eq escape releases reentrant lock; subsequent independent Map/Set operation succeeds | C2 |
| Guard starts too late | Synchronous host API targeting immediate Fiber resume fails before current/frames/queue mutate | C2 |
| Child failure eagerly kills parent | Parent catches child.call error and returns normally; parent ensure may park first | C4 |
| Observer runs before cleanup | Observed parent/child remains pending until cleanup finishes, settles exactly once afterward | C3/C4 |
| Try/Scheduler inherit Call behavior | Try yields captured Error as data; Scheduler isolates unrelated work and follows existing E010 ownership | C4 |
| One-armed Bool returns callback directly | Taken branch is Some(v), including Some(None); skipped branch is None | C5 |
| Loop follows fast inline route only | Explicit closure selector and forced deopt both exercise migrated native fallback; false condition never runs body | C5 |
| Reverse validation disappeared | Original invalid Ordering and invalid reverse result still fail; valid suspendable reverse preserves operand destination | C5 |
| Missing root masked by module globals | Sole-root parked saved result/error/non-local value survives forced collection and is later reclaimable | C6 |
| Upvalue read/write uses wrong Fiber | Two Fibers with same closure and distinct captured values; cleanup mutates correct parent capture across park | C6 |
| Ambient native privilege leaks | Access-denied/private and foreign receiver guard remain denied before/after suspend | C6 |
| Stale native trace marker remains | Failure after resumed control shows logical source/call/cleanup frames, no unrelated primitive marker | C6 |
| ABI change creates a second native/source identity | Native-only registration and full Universe reflection agree on original selector/owner/shape | C6 |
| Wake runs code | Side-effect count zero after valid enqueue and one after pump; duplicate/stale wake never advances it | C7 |
| Episode id treated as heap lifetime | Fake source roots Fiber; old wait token rejected after new park; no dereference of fabricated/freed handle | C7 |
| Every parked ancestor becomes runnable | BlockedOnChild public resume/schedule/wake rejected; only actual Parked leaf consumes ticket | C4/C7 |
| Guard removed globally | Residual hash/eq/render callbacks still reject switches and leave later healthy execution intact | C8 |

For every migrated callback operation require the relevant combination of ordinary value/None/Error-as-data/Raise, yield once/repeatedly, settled/pending/sequential await, and live capture. Share a data-driven **internal** test harness where it proves the same state transition; keep distinct source fixtures where selector lowering/bootstrap paths differ. Do not create a Cartesian explosion of tests whose only distinction is a label.

## 6. Verification command catalog and scheduling

All shell commands run from repository root; the pinned toolchain is `nightly-2026-07-10`. Prefix every Cargo behavior/build command with the cleared flags shown here. Keep `.cargo/config.toml`'s stack setting and job configuration unless diagnosing a measured infrastructure problem. Do not run concurrent Cargo builds.

### Discover exact selectable tests

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib -- --list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core -- --list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus -- --list
```

Use these after new registrations and record actual names/counts once. They prove selection exists, not behavior. `corpus::concurrency` is a Rust test covering its directory; a fixture stem is not a Cargo test name.

### Focused command families

Commands may run at multiple gates only when that gate changes the corresponding semantic boundary, as specified above. A later gate reuses prior recorded evidence if no relevant implementation changed.

```sh
# C1/C2: private state, reducer and host bridge assertions.
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib vm::control::tests
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core execution_native_control
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core execution_send_arity
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core execution_family_runtime
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core execution_depth_limits

# C3, C4, C5 respectively: separately registered source integration lanes.
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency_native_protection -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency_native_child -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency_native_fallbacks -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency_native_negative -- --exact

# Existing owner-level compatibility, scheduled only where its owner changes.
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::blocks -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::errors -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::control_flow -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::booleans -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::absence -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::option -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency_negative -- --exact

# C6: actual lifetime, diagnostics and source/native contracts.
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core memory_gc
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core observability_traceback
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core observability_fiber_trace
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core native_surface_contracts
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core object_model_invariants
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core reflection_conformance
```

**What they prove:** control tests prove explicit ownership/routing; source native lanes prove production compiler→descriptor→VM behavior; existing blocks/errors defeat unwind regressions; invocation/Family tests protect shared forwarding; GC asserts live roots/release; trace tests protect source/error presentation; native contracts protect bootstrap/source association. `cargo check` proves signatures and exhaustive consumers compile only. None proves full workspace correctness.

C7 uses `vm::tests::parked_wake_requires_the_exact_generation_and_is_once_only`; C4 observer evidence includes `terminal_observer_is_detached_and_admitted_once`, `failing_completion_observer_becomes_unhandled_scheduler_work`, and `root_result_survives_scheduler_drain_and_gc` in `vm/dispatch.rs`. Confirm their full module prefixes once in the listing. Add new internal tests beside them, not a public test-only wake API. Do not rerun the full library after every wake assertion.

## 7. Final deletion and retained-boundary gates

At C8 run these focused searches and classify matches, including cfg(test) scopes. Source search proves absence of a named mechanism, not absence of every possible indirect callback; pair it with the C0 transitive inventory and runtime evidence.

```sh
rg -n 'block_call\(' phalcom-core/src
rg -n 'send_dynamic\(|invoke_method_object\(|run_until\(' phalcom-core/src
rg -n 'frames\.len\(\) > before|frames_before_cleanup|failed = resumer' phalcom-core/src/vm phalcom-core/src/primitive
rg -n 'CannotYieldAcrossNativeFrame|native_reentry_depth|floor_depth' phalcom-core/src/primitive/fiber.rs phalcom-core/src/vm
rg -n 'each.*native|on.*non-suspend|ensure.*non-suspend|native frame' phalcom-core/src/primitive/block.rs phalcom-core/tests/core/language/corpus.rs docs/spec/current/concurrency.md
```

Expected dispositions:

- `block_call(`: zero production uses and remove unused definition; intentional synchronous test injection should use retained host API or a deliberate local adapter. No dead production helper just for one old test.
- `send_dynamic`: only guarded Map/Set hash/eq, display rendering, typing construction, host/module adapter and intentional tests. Zero migrated on/ensure/Bool/Option/whileTrue/Error.raise/ValidateOrdering callers.
- `invoke_method_object`: retained guarded host entry and intentional tests; source reflective methods use activation gateways.
- `run_until`: owning driver and explicit guarded host adapters only; no control reducer or migrated primitive recursively drives it.
- `frames.len() > before`: zero outcome reconstruction in migrated forwarding gateways. A genuinely unrelated frame-bound validation must be annotated in inventory, not deleted by text pattern.
- `frames_before_cleanup` / eager `failed = resumer`: removed as semantic transfer authority / ancestor failure cascade.
- Guard class/depth checks: remain on actual unsafe host boundaries; messages name those boundaries accurately. Their presence is a required positive safety check, not a failed migration.
- Old comments or negative fixtures declaring ordinary each/on/ensure non-suspendable: updated where behavior changed. P1 is preserved historical planning input and is exempt from rewriting.

No blind search-and-replace. Every retained match gets path/symbol/reason in state. Do not remove independent diagnostics or source syntax solely because a broad pattern matched.

## 8. Incidents, state and final delivery

### 8.1 Incident protocol

If a required test fails unexpectedly, stop dependent implementation expansion. Record:

1. Exact command, test name/count and relevant assertion/diagnostic.
2. Direct path from fixture or unit to callback activation, transfer/destination and failing owner.
3. A nearby passing comparator (e.g. ordinary call succeeds, on path fails; direct selector succeeds, Family forwarding fails; active trace passes, parked trace fails).
4. Classification: PRODUCT, FIXTURE, DEPENDENCY/PUBLICATION, BACKEND/HARNESS, BASELINE, or PLAN DRIFT.
5. Narrow allowed repair files/symbols and the evidence needed to close the incident.
6. Rejected broad shortcuts: weakening guard/assertions, converting Error/None to a control sentinel, bypassing authority, restoring recursive fallback, parser/semantic redesign, blanket baseline update.

A suspected baseline needs a credible predecessor comparison; record if not yet reproduced. A canceled or timed-out gate is INCOMPLETE. Later dependent gates cannot claim the missing invariant. Mechanical API drift may be adapted; semantic changes must be documented and reconciled with requirements, not silently substituted.

### 8.2 State-file template

Create only when implementation begins; planning does not pre-fill implementation success.

```md
# CONC002.C2.P1-R1 implementation state

Baseline: <HEAD, branch, relevant dirty dependency identity>
Current revision: <HEAD/diff identity>
P1 preservation hash: <hash>

## Established invariants
- <id, claim, checkpoint, source owner>

## Decisions
- <decision, reason, code anchor, rejected alternative>

## Checkpoint evidence
| Gate | Contract | Exact command / test count | Result | Proves | Revision |
|---|---|---|---|---|---|

## Migration and residual host inventory
| Caller | Retained host state | Guard / activation | Final disposition | Evidence |
|---|---|---|---|---|

## Roots and lifetime
| Field / payload | Live owner | Parked owner | Trace path | Release point |
|---|---|---|---|---|

## Deferred gates
- <command or assertion> → <gate>; <why deferred>

## Active incident
None / <reproducer, classification, allowed repair, next evidence>

## Next resume action
<exact checkpoint/task and source anchor>
```

Record facts/evidence/decisions only, not raw scratch reasoning. End each checkpoint with a short supervisor report: COMPLETE or INCIDENT; established contract; key symbols; passing evidence; hostile cases; negative searches; deferred gate destinations; unexpected findings; next task.

### 8.3 Checkpoint evidence summary at planning time

| Checkpoint | Semantic contract | Required evidence destination | Status |
|---|---|---|---|
| C0 | Reproducible baseline and owned scope | Existing controls + verified red + inventory | NOT_STARTED |
| C1 | Owned control and explicit activation | Internal roots/outcomes + gateway consistency | NOT_STARTED |
| C2 | VM transfers and host escape | Router/host/unwind evidence; may integrate with C3 as specified | NOT_STARTED |
| C3 | Protected/cleanup suspension | Public phases, precedence and negative guard replacement | NOT_STARTED |
| C4 | Parent exception injection | Call/Try/Scheduler and observer/trace evidence | NOT_STARTED |
| C5 | Native fallback transparency | Explicit/deopt/branch/wrapper/depth evidence | NOT_STARTED |
| C6 | Lifetime/bootstrap correctness | Real parked GC, authority, traces and descriptor parity | NOT_STARTED |
| C7 | Readiness identity and deferred execution | Ticket/wake/queue tests and precise reactor gap record | NOT_STARTED |
| C8 | Migration/delivery closure | Negative searches, performance note and broad gates | NOT_STARTED |

No row may become COMPLETE without evidence on the actual implementation revision. Plan review and source inspection do not change these statuses.

### 8.4 Final broad gates

Run core compatibility first, then workspace delivery, serially:

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

- Core library/integration/corpus prove compatibility across shared dispatch, bootstrap, error/GC behavior and existing source programs.
- CLI smoke proves process-facing error/exit behavior; no internal host escape may appear in output.
- Format is a check; do not broadly rewrite unrelated source.
- Workspace build proves all target consumers compile; workspace tests prove broad integration compatibility; Clippy proves the configured lint gate. These complement focused semantics, not replace them.
- Whitespace and final document-link/metadata checks prove reviewability only.

The current manifest confirms `fiber-pool` and `vm-trace`. Run targeted feature compatibility once at C8 (these prove constructor/terminal-buffer and instrumented-dispatch integration, not new semantic contracts):

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --features fiber-pool --test core execution_native_control
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --features vm-trace --test core observability_traceback
```

Performance has an existing Criterion target `phalcom-core/benches/vm_bench.rs`, with `bare_send`, `arith_send`, `fiber_spawn` and `rest_fallback_send` benchmarks. At Task 32 run the same command/configuration against predecessor and implementation and retain Criterion results:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo bench -p phalcom-core --bench vm_bench
```

The harness includes bootstrap in timings and checks program results; report that measurement limit. Do not enable vm-trace for performance comparisons. Add only the missing control-heavy/park-wake workload or document its structural cost separately; the existing four benchmarks do not directly measure cleanup latency.

### 8.5 Deferred evidence and release-complete criteria

No deferred assertion/gate remains without a successful execution, explicit removal from scope with rationale, or a recorded release blocker. All checkpoints must be COMPLETE, all hostile and deletion gates satisfied, all required broad gates completed, and no active INCIDENT before marking RELEASE_COMPLETE. Focused-tested plus unrelated verified baseline failure is BASELINE_BLOCKED, not release-complete.

Suggested commit groups, only when authorized: C1 foundation; C2/C3 integrated transfer and public error-control migration; C4 parent exception delivery; C5 native fallback migration; C6 lifetime/native integration tests and fixes; C7 small readiness/docs; C8 closure and evidence. Scope commits by behavior and owned files, not extension. P1 remains unchanged.

### 8.6 Final report requirements and exclusions

Report: verified old behavior; root cause and removed Rust-local continuation; native taxonomy; final control/outcome/host-escape design; on/ensure/return precedence; parent failure delivery; GC/upvalues/authority/trace ownership; exact migrated and guarded paths; wake token and handle lifetime; wake versus execution; root quiescence limitation; future I/O and optional native-continuation attachment; hot-path costs; commands/results and remaining limitations.

Implemented/preserved reactor groundwork must be distinguished from future design. Expected remaining restrictions are concrete: synchronous host APIs, Map/Set user hash/equality during native lookup, print/List user display callbacks, Typing constructor callback, manual-coroutine pending await, scheduled user yield without a consumer, and root no-runnable/pending-Future behavior without external registrations. Recheck the inventory if implementation changes this list.

Excluded work remains: reactor backend/polling; sockets/timers/files/process APIs; OS or cross-thread readiness injection; new public native async protocol; generator consumer relation; cancellation and queue revocation; Task/TaskGroup; channels/select; generic Fiber/Future typing and type-form lowering; parser/LSP redesign. No leftover task is disguised as a reactor requirement.

Final state must retain established invariants, decisions, code anchors, completed evidence, residual limitations, no forgotten deferred gates and an exact next action. Preserve the distinction between implementation complete, focused-tested, baseline-blocked and release-complete.

## Planning review and verification boundary

R1 is an expanded, source-grounded plan only. No runtime implementation, test execution or performance certification has occurred during this writing task. A bounded independent document review checked outcome forwarding, host-floor escape, C2/C3 compatibility, parent failure injection, GC and corpus selection. Its successful-switch-back finding is incorporated in §3.2.1, Tasks 5/8/18/24 and the regression matrix. Document links, task/checkpoint structure and P1 preservation are checked separately; implementing-agent evidence belongs in the state record.
