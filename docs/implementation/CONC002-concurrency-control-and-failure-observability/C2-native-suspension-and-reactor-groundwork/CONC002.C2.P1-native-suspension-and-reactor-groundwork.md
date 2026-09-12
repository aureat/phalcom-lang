---
id: CONC002.C2.P1
program: CONC002
checkpoint: CONC002.C2
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# Native suspension and reactor groundwork implementation plan

> For the implementing agent: execute the checkboxes in dependency order, using `superpowers:executing-plans` if useful. This document authorizes no execution by itself. Preserve concurrent work; record evidence at each gate. No delegation is required.

**Goal:** make native language-control operations suspension-transparent by representing their continuations in the VM, while retaining guards for genuinely synchronous host callbacks and preserving a source-independent wake boundary.

**Architecture:** retain ordinary `activate_function` and the synchronous leaf ABI. Introduce a small, Fiber-owned control stack and an iterative outcome router for protected execution, cleanup, and native control fallbacks. Reuse current parking generations and scheduler admission; do not build an I/O backend.

**Technology:** Rust, pinned `nightly-2026-07-10`, Phalcom Universe source, existing core and language-corpus test targets.

## Scope, baseline, and authority

Prepared 2026-09-12 from local `main`, HEAD `01b58ec04faa63414ececf46dab10e4ebd4bba71`. Source findings below describe the **working tree**, which is dirty, not a clean-main test certification. No runtime tests were run for this planning task. Do not interpret source inspection as reproduced behavioral evidence.

The attachment supplied for this plan is “Phalcom — Architecturally Repair Native Suspension and Establish Reactor Groundwork,” sections 1–32. The immediate user request is planning only; its embedded implementation instructions define the later implementation's acceptance criteria. This plan captures those requirements without requiring the ephemeral attachment to execute it.

Read [documentation authority](../../README.md), [specification authority](../../../spec/README.md), and the effective chapters under `docs/spec/current/`: `concurrency.md`, `error-handling.md`, `blocks.md`, `functions.md`, and `memory-management.md`. Follow their ADR links for disputed semantics. These paths remain authoritative under the migration charter; this plan is not a replacement specification.

Related work: [C1 architecture/continuation plan](../C1-concurrency-control-and-failure-observability/CONC002.C1.P3-concurrency-architecture-and-continuation-semantics.md), P1 control ownership and P2 failure observability in that directory. C2 consumes their resulting ownership model. It does not repeat their Future combinator, adoption, typing, or E010 redesign.

Observed concurrent edits include `vm/{mod,dispatch}.rs`, `heap/fiber.rs`, `primitive/fiber.rs`, Universe `concurrency/fiber.ph`, `docs/spec/current/concurrency.md`, concurrency fixtures, and unrelated semantic type-store/tests. The C1.P3 plan is untracked. Before implementation, establish whether these changes have landed or must be incorporated; a worktree from HEAD alone does not contain them. Never stash, overwrite, blanket-stage, or commit them as part of this plan without authority.

## Findings that change the implementation

All source anchors are repository-relative and symbol names are durable; line numbers are discovery aids against the inspected tree.

| Evidence | Finding and implementation consequence |
|---|---|
| `src/method/object.rs:16`, `src/vm/send.rs:548`, `src/primitive/block.rs:140` under `phalcom-core/` | `CallOutcome::{Returned, EnteredFrame}` and `activate_function` already support VM-visible closure, block, bound method, Family, AssociatedFamily and BoundMethodFamily activation. Reuse this gateway and caller authority; do not copy the narrower `resolve_callable` path. |
| `phalcom-core/src/primitive/block.rs:203–262` | `block_call` owns argument binding, receiver window, saved frame floor and the Rust `run_until` return. Its counter spans recursive dispatch. The comment blaming `.each` is stale; ordinary `call(***)` uses the shape gateway. |
| `phalcom-core/src/primitive/block.rs:342–411` | `block_on` keeps stack/frame lengths, catch class, handler, normalized failure and traceback in host locals. It invokes overridable `is(_)` through `send_dynamic`, so migrating just the body leaves a second unsafe callback. |
| `phalcom-core/src/primitive/block.rs:434–473` | `block_ensure` retains body outcome in Rust and temporarily roots its value/error during cleanup. Cleanup failure or non-local return supersedes the body outcome. Frame-length shrinkage currently detects cleanup return. |
| `phalcom-core/src/vm/dispatch.rs:2136–2205` | `ReturnNonLocal` eagerly closes upvalues and truncates all frames through the home. A new ensure stack cannot be added after this operation: it must intercept the transfer **before destruction**. Ordinary return also needs a completion boundary hook. |
| `phalcom-core/src/vm/dispatch.rs:663` | `run_until` routes failures to the Fiber floor, with a separate nested-host floor. There is no general VM-resident on/ensure handler chain in the inspected path. Do not assume an existing handler registration API will do this work. |
| `phalcom-core/src/primitive/fiber.rs::store_live_into/load_live_from` | Live frames, stack, open upvalues and invariant `checking` state are moved between VM and Fiber. New control state must join that exact ownership transfer. Frame generations remain VM-global. |
| `phalcom-core/src/heap/trace.rs:180`, `src/vm/gc.rs` | Parked and running roots have separate traversal sites. Both must trace the new control records, including saved outcome/error and callable references. VM-global temporary roots cannot own parked continuation lifetime. |
| `phalcom-core/src/vm/mod.rs:555` | `wake_parked_fiber(fiber, generation)` already accepts only exact `Parked(generation)`, reserves `Queued`, and enqueues without running. Reuse it; do not create a competing token system. It assumes a valid live Fiber handle. |
| `phalcom-core/core/universe/src/concurrency/fiber.ph:220`, `src/vm/dispatch.rs:681` | Root await drives queued work; empty queue while pending is currently an error. Root completion drains runnable work. Neither is a reactor idle/poll contract. Preserve today's behavior and name the later integration point explicitly. |

### Native re-entry inventory and disposition

Search performed: `block_call`, `run_until`, `send_dynamic`, `invoke_method_object` throughout `phalcom-core/src`. Expand transitive helper callers during S0; this is a direct-call inventory, not proof that every reachable native dispatch site was audited.

| Site | Classification | Required disposition |
|---|---|---|
| Ordinary `Function.call`, `callWith`, reflective shape activation | Already VM-visible | Preserve; use as positive controls. Verify exact reflection gateways before changing anything. |
| `block_on`, `block_ensure` | Legacy native control | Migrate fully, including catch matching and cleanup. |
| `primitive/block.rs::block_while_true` | Legacy native loop | Migrate both condition and body; preserve strict Bool and None completion, plus sacred fallback semantics. |
| `primitive/boolean.rs::{bool_and,bool_or,bool_if_true_if_false}` | Callback tail branches | Shape activation with direct forwarding; retain short circuit values. |
| `primitive/boolean.rs::{bool_if_true,bool_if_false}` | Callback plus result transformation | VM-visible post-return `Some(result)` action; skipped branch remains None. |
| `primitive/option.rs::option_match` | Callback branch eliminator | Shape activation of chosen branch, preserving labels and Some payload arity. |
| `primitive/error.rs::error_raise` | User `message` send followed by Raise | Migrate to a VM-visible message-then-raise action so protected/cleanup failure paths do not hide another re-entry. Preserve returned Error versus raised Error distinction. |
| `vm/send.rs::{send_dynamic,invoke_method_object,try_module_export_send_dynamic}` | Synchronous host API/adapter | Keep guarded for remaining callers; source-facing paths must use shape activation. Do not advertise these host APIs as suspendable. |
| `primitive/mod.rs::{send_hash,send_eq}`, used by Map/Set lookup | Native algorithm with live bucket/probe state | Retain guard and document precise residual restriction; do not redesign collection mutation here. Enumerate all transitive Map/Set callers and add a guard regression. |
| `value/render.rs::to_display_string` | Native formatting with post-send String conversion | Retain guarded host rendering boundary unless needed by a migrated source-facing operation. Production callers found: `primitive/system.rs:20` (printing) and `primitive/list.rs:193` (element rendering); both retain native formatting state. Error.raise uses `to_string(&VM)` after `message`, which is a distinct non-dispatch conversion. |
| `primitive/typing.rs:2776` | Dynamic constructor then TypingKnown wrapping | Retain explicitly guarded inspection boundary for this bounded repair; verify whether public semantics require wider suspension in a follow-up. |
| `vm/dispatch.rs::ValidateOrdering`, reverse branch | User `reverse` dispatch then result validation | Convert to ordinary dispatch plus VM post-return validation; avoid a hidden native boundary in ordinary operator execution. |
| `universe/mod.rs:584` and later `vm/send.rs` test module calls | Test-only native injection/host calls | Do not count as production usage. Update injected callback implementation or deliberately retain it as a negative guard fixture. |
| Arithmetic, representation access, allocation primitives without user sends | Leaf synchronous | No continuation allocation, queue operation, hash map, atomics or ABI-wide suspension protocol. |

No genuinely asynchronous native operation was identified by this focused inspection. Residual guarded boundaries are permitted by the attachment; they must be reported by exact operation, not hidden behind a claim that all natives can now suspend. Expanding Map/Set or host rendering into resumable algorithms requires its own bounded plan.

## Chosen control representation and routing contract

Create `phalcom-core/src/vm/control.rs` for the owning types, transition logic and internal tests. Keep bytecode frames as they are; add a lazily allocated `Vec<ControlActivation>` to the live VM and to `FiberObject`. Control records store only explicit owned data and stable indices/tokens, never references into `VM`, Rust closures, or heap borrows.

Suggested concrete payload (adapt names to source conventions, preserve the contract):

```rust
struct ControlActivation {
    owner: FrameToken,       // caller identity; validate generation, not index alone
    stack_base: usize,       // result destination for this native invocation
    child_floor: usize,     // callback frame boundary
    caller_access: CallerAuthority, // use existing authority representation
    state: ControlState,
}
enum PendingTransfer {
    Value(Value),
    Raise(PhError),
    NonLocalReturn { target: FrameToken, value: Value },
}
// ControlState is a closed enum for ProtectBody, ProtectMatch, ProtectHandler,
// EnsureBody, EnsureCleanup(saved transfer), WhileCondition, WhileBody,
// WrapSome, RaiseAfterMessage, and ValidateReversedOrdering.
```

This is implementation-oriented pseudocode, not a promised compilable patch: `CallerAuthority` names the existing pair carried by `ArgumentView`, and some states need callable/class fields. `PendingTransfer::Value` is ordinary callback completion; a returned Error or None stays in that arm. Suspension is represented by Fiber execution metadata and is **never** a transfer/error/value sentinel. An abort that currently becomes a raised failure stays in that failure channel.

Install the record and root its retained values before allocating or activating a callback. Retain original receiver and callable arguments in the record or stack until ownership has transferred. Use receiver-index stack windows and the existing `InvocationLayout`/`ArgumentView` construction; capture caller privilege at entry rather than reconstructing it from a later handler frame.

The reducer is iterative. A shape native gateway installs a control operation and returns to the dispatch loop. Reuse `EnteredFrame` when a bytecode callback was entered; for work that is pending without a new frame (e.g. immediate native callback result), add a narrowly scoped `CallOutcome::EnteredControl` disposition if needed. Handle that disposition at **every** `CallOutcome` consumer; it is not a universal asynchronous native ABI. Never mislabel an immediate result as an entered bytecode frame, recurse into the reducer indefinitely, or call `run_until` from a migrated gateway.

The loop processes pending control work before fetching the next bytecode. Callback activation can return immediately, enter a frame, or fail; all three enter the same reducer path. No callback result is pushed twice, and no stale caller IP is reused after a switch. Keep the empty-control-stack path a cheap check; only control operations allocate/grow its storage.

### Outcome ordering

1. Normal callback Return delivers its value to the nearest record waiting at that callback floor; ordinary calls without a matching record retain current behavior.
2. Raised errors are offered to the nearest eligible control record **before** Fiber terminalization. A record for a protected body may catch; its own matching/handler phases must not catch their own failures.
3. Non-local return validates the live home token first. Walk crossed control boundaries innermost first, run ensures while relevant captured slots still exist, then resume the targeted unwind. An on record does not catch this transfer. A return targeting a frame inside a protected region does not spuriously exit that region.
4. Close open upvalues before truncating each actually abandoned stack segment. Do not close an ancestor's live slots before cleanup can read/write captures; do not leave abandoned callback frames installed while cleanup executes.
5. Preserve a still-pending transfer in rooted Fiber-owned storage while cleanup runs or parks. Cleanup normal completion discards its result and resumes the pending transfer. Cleanup Raise or non-local return replaces it. Nested cleanup executes inside-out, exactly once per crossed region.
6. When a retained synchronous host invocation is driving `run_until(base_frames)`, VM routing must respect that host floor: handle only control records belonging inside it. A Raise or NonLocalReturn leaving that floor must first return through the guarded host caller before outer VM controls consume it. Keep the transfer rooted during this handoff. For a non-local return, use an explicit internal escape tag through retained synchronous adapters (for example a private PhError control-escape variant plus VM-owned pending transfer); never a sentinel Value. Such a tag bypasses native error normalization and language catch matching. Audit every retained adapter and transitive caller so it forwards the escape without continuing bucket mutation, formatting or other native post-work. Only the owning outer VM driver consumes it after host depth drops to its boundary. Do not execute outer cleanup, consume an outer handler, or resume unrelated frames under a still-live host continuation.
7. Only after no eligible control remains may an outcome reach the existing Fiber completion/resumer/observer path. Do not publish Done/Failed or settle a Future while cleanup is still pending.

### Protected matching and cleanup semantics

Preserve class validation, first matching handler, native-error normalization to a base Error, traceback capture before unwinding, existing help/rendered payloads, and the overridable `error.is(class)` send. `ProtectMatch` retains normalized failure and invokes `is(_)` through VM dispatch; only Bool true matches, as in the current implementation. A failing match or handler propagates outward; it is not retried against its own on record. A suspended match/handler keeps the same region identity.

| Body outcome | Cleanup normal | Cleanup raises E2 | Cleanup non-local return R2 |
|---|---|---|---|
| ordinary value V | return V | raise E2 | perform R2 |
| raises E1 | raise original E1 | raise E2 | perform R2 |
| non-local return R1 | perform R1 | raise E2 | perform R2 |

This preserves documented cleanup-supersedes behavior, including return, not just failure precedence. Protect and cleanup may each suspend repeatedly. No new source-language coloring or public Fiber states are needed.

## Implementation checkpoints

Each gate is a reviewable work unit. Do not mark a gate passed without recorded completed test output. Commit a gate only if the implementation session authorizes commits. Persist findings in `native-suspension-implementation-state.md` beside this plan, with HEAD, dirty boundary, exact commands, selected-test counts and unresolved observations.

### S0 — Freeze evidence and acceptance fixtures

**Files:** existing C1 state/plan; `phalcom-core/tests/fixtures/language/concurrency/`; `phalcom-core/tests/core/execution/depth_limits.rs`; test registration files located through the existing test tree.

- [ ] Record HEAD, branch, relevant diff and C1.P3 completion status; read any more specific AGENTS guidance before edits.
- [ ] Re-run direct and transitive re-entry searches from the inventory, excluding `#[cfg(test)]` sections when counting production. Record caller, retained host state, arbitrary dispatch reachability, guard and chosen disposition for every site.
- [ ] Discover exact corpus registration rules and named tests with `-- --list`. Locate existing error/non-local-return/sacred-deopt cases; do not invent unsupported source syntax.
- [ ] Add red fixtures under the `concurrency_native_` prefix for on body yield/pending await and ensure body/cleanup suspension. Use known-correct Fiber/Future patterns from current fixtures, explicit event logs and bounded scheduler steps; do not use timing sleeps.
- [ ] Reproduce successful each yield and pending-await controls on the same baseline. Run the four cleanup failure combinations and non-local-return precedence before implementation.
- [ ] Capture current native-frame refusal at the actual native control boundary. If a probe fails to compile, repair its syntax before calling it a reproduced runtime defect.

**Exit:** source taxonomy complete for this scope; red cases fail for native-frame refusal; positive controls and precedence are understood. Prior source claims are now independently confirmed or explicitly corrected.

### S1 — Fiber-owned control storage and GC ownership

**Create:** `phalcom-core/src/vm/control.rs`.
**Modify:** `phalcom-core/src/vm/mod.rs`, `phalcom-core/src/heap/fiber.rs`, `phalcom-core/src/primitive/fiber.rs`, `phalcom-core/src/vm/gc.rs`, `phalcom-core/src/heap/trace.rs`.

- [ ] Define closed state/transfer types and shared value/handle tracing visitor; include callback, self, catch class, error, saved return, and phase-specific values. Audit every `PhError` payload that can contain heap handles.
- [ ] Initialize empty storage in all VM/Fiber constructors. Move it with frames/stack/checking in both switch directions. Do not move the native re-entry counter into Fibers: it still describes the live host stack.
- [ ] Trace running records through VM roots and parked records through Fiber tracing. Queue membership and Future waiter ownership must continue to root the Fiber itself.
- [ ] Make record removal/terminal teardown release saved references; extend invariant verification to validate record owner tokens, stack boundaries, phase and single live owner.
- [ ] Add internal state-switch/trace tests plus GC tests that root only through a live/parked record. Check both survival and reclamation after completion.

**Exit:** control state survives switch and collection without duplicates, stale handles or retained terminal payloads. No migrated public behavior yet; existing tests remain unchanged.

### S2 — Iterative return, error and non-local-transfer router

**Modify:** `phalcom-core/src/vm/{control,dispatch,send}.rs`, `phalcom-core/src/method/object.rs`, and the owning error type in `phalcom-core/src/error/` or `error.rs` if an internal host escape tag is needed (locate its definition first); inspect `phalcom-core/src/frame.rs` without expanding every frame unless necessary.

- [ ] Add callback activation and immediate-result routing helpers using `activate_function` and existing selector activation. Handle allocation/arity/access failures before partial state escapes; unwind partial installation on failure.
- [ ] Handle any new CallOutcome disposition exhaustively and drain pending control actions before bytecode fetch. Synchronous callbacks must not require fake bytecode frames.
- [ ] Intercept Return, failures from dispatch, and ReturnNonLocal before destructive unwind. Retain existing native-floor semantics and handle errors originating in immediate primitives as well as bytecode.
- [ ] Implement inside-out transfer walking and exact result destination restoration. Route exhausted controls into existing Fiber terminalization, not a duplicate terminal path.
- [ ] Add internal transition tests for immediate/frame/error outcomes, nested host floor, stale home token, return target within versus outside a region, and bounded stack growth under deep nesting. Include a non-local return originating under a retained synchronous host callback with its target above that host floor; assert native post-work is skipped, the pending transfer survives GC, and outer ensure runs only after host unwinding. Repeat with outer cleanup attempting suspension.

**Exit:** explicit transfers can traverse nested records with close-before-truncate behavior, one result delivery, and no recursive VM drive. No scope-wide switch-guard relaxation.

### S3 — Protected execution and raise migration

**Modify:** `phalcom-core/src/primitive/{block,error}.rs`, `phalcom-core/src/vm/control.rs`; update native registration metadata only where required by shape ABI.

- [ ] Convert block_on to shape admission that validates catch Class and installs ProtectBody; its Rust function ends before body bytecode executes.
- [ ] Implement ProtectMatch and ProtectHandler, preserving normalized error identity, traceback/help and access authority. Match and handler callbacks both use ordinary activation and may park.
- [ ] Convert Error.raise's message send into RaiseAfterMessage. The error receiver remains rooted while its message override executes; a failure/return from that override follows existing transfer precedence.
- [ ] Verify direct `.on(...)` and desugared try/catch reach the same operation. Do not introduce parser or semantic reconstruction of handlers.
- [ ] Add immediate value/None/Error-as-data, nested on, unmatched class, pre/post-suspension Raise, rejected await, suspended handler and suspended match/message override tests.

**Exit:** catch identity and outcome are stable across repeated suspension, and failures in match/handler propagate outward. Inspect traceback assertions rather than rewriting snapshots blindly.

### S4 — Ensure with suspendable cleanup and non-local return

**Modify:** `phalcom-core/src/primitive/block.rs`, `phalcom-core/src/vm/control.rs`, `phalcom-core/tests/core/memory/gc.rs`; add source fixtures.

- [ ] Replace recursive body/cleanup and temporary-root lifetime with EnsureBody/EnsureCleanup plus explicit saved transfer.
- [ ] Start cleanup on normal body completion, error and non-local transfer; mark the cleanup phase before activation so re-entry/failure cannot run it twice.
- [ ] Apply the full precedence table, preserving original error object and traceback after successful cleanup. A failed cleanup supersedes without later restoring the discarded body transfer.
- [ ] Test body yield then return/error, cleanup yield then return/error, both await pending Futures, sequential awaits, nested ensures, and inner cleanup superseding an outer pending transfer.
- [ ] Test non-local return from body and cleanup, dead/cross-Fiber home token, self/captured mutable local changes during cleanup, and ancestor re-entry refusal while a cleanup descendant is active.
- [ ] Extend the existing `ensure_outcome_survives_collecting_cleanup` and raised-error GC coverage to collection by another Fiber while cleanup is parked. Confirm control records and saved values become collectible after terminal completion.

**Exit:** all table cells plus suspension combinations pass; cleanup executes once; observers settle only after cleanup; no frame-length heuristic remains as the definition of transfer kind.

### S5 — Remaining language-control fallbacks

**Modify:** `phalcom-core/src/primitive/{boolean,option,block}.rs`, `phalcom-core/src/vm/{control,dispatch}.rs`; affected shape registration/invariant tests.

- [ ] Migrate Bool and/or/paired conditional and Option chosen branch as tail activation. Preserve selected branch only, argument count/labels, access and short-circuit return values.
- [ ] Migrate one-arm Bool callbacks with explicit WrapSome completion. `Some(None)` and `Some(Error-as-data)` remain values, not control.
- [ ] Migrate whileTrue to condition/body phases; validate strict Bool after every condition resume, discard body value, return None on false. Do not require source wrappers that recursively fall back to the same primitive.
- [ ] Migrate reversed ordering validation through ordinary dispatch and a post-return class check; preserve validation both before and after reverse.
- [ ] Exercise explicit selectors and deliberately forced sacred deopt/override routes. Passing only compiler-inlined conditionals/loops does not cover these primitives.
- [ ] Preserve normal callable shapes, rest/labeled lanes, reflected bound methods and Family activation by reusing canonical gateways. Update test-only injected natives deliberately.

**Exit:** each migrated callback path passes value, Raise, yield and pending-await cases; bypassed branches do not run. Leaf primitives keep their existing synchronous path.

### S6 — Preserve readiness identity and specify reactor attachment

**Modify only if required:** `phalcom-core/src/vm/mod.rs`, `phalcom-core/src/primitive/{system,fiber}.rs`, Universe `concurrency/fiber.ph`; tests beside existing wake tests.

- [ ] Retain `Parked(generation) -> Queued` as the single validated readiness transition. Remove Future-only wording where it describes a source-independent mechanism; avoid renaming stable APIs merely for aesthetics.
- [ ] Test valid ticket accepted once, duplicate/stale/new-episode mismatch rejected, terminal not revived, ancestor BlockedOnChild not queued, and wake does not execute user code. Reuse existing generation tests and add missing assertions instead of duplicating them.
- [ ] Preserve the checked generation increment already present in `primitive/fiber.rs:439–442`; test exhaustion rejects rather than wrapping into an earlier episode.
- [ ] Document that current Future waiter tuples strongly own their Fiber; wake accepts a live ObjRef. A future external registration must own a traced Fiber handle until completion/unregistration or validate a generational handle before dereference. Parking generation alone does not make a freed ObjRef safe.
- [ ] Add a fake internal completion-source test only if it exercises a new/refined seam: submit wake, observe no execution, pump one runnable item, observe completion. No timers, OS polling, threads or sleeps.
- [ ] Record exact root extension: at pending root await's empty-queue branch and post-root runnable drain, a future executor must distinguish runnable exhaustion from externally pending registrations before choosing poll/idle/completed. Keep today's no-external-source behavior; do not invent a global-completion claim or add a useless idle enum now.

**Exit:** Future and future external events have one conceptual wake boundary. Existing generations/queues are sufficient; report preserved groundwork distinctly from new code.

### S7 — Residual safety, performance and release handoff

**Modify:** scoped comments and `docs/spec/current/concurrency.md` restriction paragraph after behavioral gates; new checkpoint state record.

- [ ] Re-run the complete direct/transitive re-entry inventory. Remove block_call only if production and intentional test callers no longer need it; keep explicit synchronous APIs and guards where live host algorithms remain.
- [ ] Add negative tests for residual Map/Set hash/equality and host callback suspension, including attempted resume and pending await where applicable. Verify refusal occurs before damaging stack/queue/wait registration state and later healthy work still runs.
- [ ] Update outdated `.each`, `.on` and `.ensure` guard examples to exact retained boundaries. Reconcile concurrency restriction text with delivered scope without overwriting concurrent edits. Do not imply every native callback is now suspendable.
- [ ] Compare allocation/profile behavior for leaf arithmetic/native calls, ordinary calls, control activation, switch, park and wake. Empty Vec construction allocates nothing; controls may grow per-Fiber storage. Record actual overhead, not unmeasured numeric promises. No atomics, dynamic maps, trait dispatch or per-leaf continuation objects.
- [ ] Run validation ladder below; investigate failures against a clean predecessor only when necessary. Record canceled/time-limited runs as incomplete; never weaken assertions or expected outputs to hide failures.
- [ ] Publish state record with migrated/residual inventory, architectural rationale, GC/upvalue ownership, semantics table, readiness/quiescence/host-continuation extension points, actual tests, and remaining limitations.

**Exit:** migrated operations survive suspension/resumption/return/failure with live state; residual guard regressions pass; readiness remains deferred execution. Mark release-complete only after named release gates complete successfully.

## Regression coverage matrix

Create source cases under `phalcom-core/tests/fixtures/language/concurrency/concurrency_native_*.ph` with explicit `.expected` files through the current corpus conventions. Group cases only when failure attribution stays clear.

| Coverage | Required observable assertion |
|---|---|
| immediate, each migrated path | ordinary value, None and Error as data retain identity; raised Error is failure |
| yield, each migrated path | one/multiple yields go to correct resumer, supplied resume value restored, later return/failure correct |
| await, each migrated path | settled/pending/sequential waits, fulfillment/rejection, no spurious native-frame refusal |
| captured state | receiver/self, local, nested closure and mutable capture retain values across switching and collection |
| protection | same handler after resume, nested first match, rejected await caught, handler/match can suspend, non-local return bypasses catch |
| ensure | complete precedence table, body and cleanup independently suspending, nested inside-out ordering, exactly-once cleanup |
| call-chain ownership | public resume/schedule of active ancestor rejected; only actual external waiter receives wake |
| working control | Iterable/List.each yield and pending await remain green |
| GC | saved result/error/callable survives parked collection; abandoned values and records released |
| residual host boundary | genuine live Rust continuation still rejects switches, no registration or stack corruption |
| readiness | exact current episode once; stale/duplicate/terminal/ancestor rejected; enqueue does not execute |
| bounds | deep control nesting bounded by intentional VM resource policy; no growing native re-entry stack |

Use source tests for semantics and internal VM/GC tests for state/identity; do not assert formal semantic facts in ad hoc runtime helpers. Only add semantic-layer tests if source typing genuinely changes, which is not intended here.

## Commands and evidence protocol

Read `phalcom-core/tests/README.md`, `.cargo/config.toml`, and `.github/workflows/ci.yml`. Keep pinned toolchain and repository stack settings. Run Cargo commands serially. First list names; a zero-test filtered success is not evidence.

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus -- --list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core -- --list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus concurrency_native
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus concurrency
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core memory
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core execution
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test cli-smoke
```

Adjust filters to names actually listed, and explicitly include existing error-handling/non-local-return, sacred-deopt, traceback and GC cases. The broad corpus gate catches public native registration/bootstrap drift.

At the final coherent checkpoint, the requested release verification is:

```sh
cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

Check formatting without broadly rewriting unrelated files. Focused green, baseline-blocked and release-complete are different statuses. Evidence should include exact command, exit status, nonzero selected tests, baseline/working-tree identity and any independent failure reproduction.

## Future extension contracts and exclusions

A future native I/O leaf may register an owned completion and return Future/value immediately; ordinary await parks its VM continuation. Completion validates the current ticket, queues, returns; the executor later resumes it. No Rust frame spans the I/O latency. Future is an eventual-value API, not the scheduler's required readiness type.

If a genuinely resumable native operation later becomes necessary, its state belongs in an explicitly GC-traced owned activation integrated with the control router's completion path, with a defined native disposition, resource teardown and terminal behavior. It cannot be an opaque pointer to a host stack frame. Such a variant need not change the legacy leaf function-pointer ABI. This plan introduces no native OS suspension protocol.

Single-threaded assumptions: park-registration/check/queue transitions execute under exclusive VM access; plain counters and VecDeque suffice. Cross-thread readiness would need a separately owned injection boundary and executor-side validation, not mutable VM access from a completion thread. That future concern adds no current atomics or locks.

Excluded: reactor backend, polling implementation, sockets/timers/filesystem/process API, parallel VM, Task/TaskGroup, cancellation, channels/select, task-local state, public async syntax and universal suspendable-native ABI. Broader confirmed defects go into the checkpoint findings record with precise reproducers and an owner, not an opportunistic redesign.

## Final implementing-agent report checklist

- [ ] Distinguish verified previous behavior from source-only claims and dirty-tree dependencies.
- [ ] Explain removed host continuation state and chosen VM representation.
- [ ] List exact migrated paths and residual guarded operations, including transitive dynamic sends.
- [ ] Describe on matching/error behavior, ensure saved transfers, cleanup precedence and non-local-return routing.
- [ ] Record GC tracing, upvalue close ordering and terminal cleanup/observer timing.
- [ ] State exact parking identity, stale-event validation, wake/execution separation and quiescence limitation.
- [ ] Explain Future-based native I/O attachment and optional future owned native continuation location.
- [ ] Report measured hot-path consequences, every executed gate and remaining limitations without claiming an implemented reactor.

## Planning review

A focused independent document review checked continuation ownership, non-local return, GC, residual host floors and reactor scope. Its host-floor non-local-return clarification is incorporated into outcome ordering and S2. This is document review only, not implementation or runtime verification.
