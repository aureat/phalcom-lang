# AUD-RUNTIME-005 — Grouped contract risks and performance opportunities

These smaller findings share a document, not necessarily a root cause. They do not justify a runtime rewrite. Costs below are source-derived operation counts or measured structure sizes, not benchmark speedup claims.

## 1. Executable chunks remain a trusted, mutable internal representation

- Severity: Medium
- Category: Bytecode, Runtime Safety, Testing
- Confidence: Confirmed
- Priority: Before accepting external/cached bytecode or adding more rewriting passes
- Runtime impact: Invalid chunks can panic instead of producing a controlled VM error.

`Chunk` exposes `code`, `constants`, spans and parallel caches publicly (`phalcom-core/src/chunk.rs:150`). `add_instruction` maintains their parallel lengths, but direct mutation bypasses that invariant. `run_until_inner` directly indexes `code[ip]` (`vm/dispatch.rs:1155`); `invoke_at` assumes the receiver window, selector constant type and cache slot are valid (`:987`). Upvalue operations similarly trust indices. The executed probe replaced a compiled closure's chunk with an empty chunk and caught an index panic.

This demonstrates lack of validation for malformed internal input, not a source-language exploit or memory unsafety. Generational arena checks and Rust bounds checks prevent unchecked memory access in these paths. Source narrowing in [001](AUD-RUNTIME-001-unchecked-operand-narrowing.md) is more urgent because it violates these assumptions through compilation itself.

**Direction:** distinguish mutable construction from validated executable chunks. Validate control-flow destinations, operand kinds/ranges, stack requirements and joins, closure descriptors and side-table lengths before execution. Keep cheap release checks where host APIs remain public. A shared opcode contract should serve validation and transformations; adding a second independently maintained stack-effect table would repeat the problem.

**Tests:** empty/truncated chunks; bad constants and selector types; negative/outside branch destinations; inconsistent cache lengths; wrong receiver/argument windows; invalid capture indices. Property tests should generate valid CFGs and compare transformed/untransformed execution. No full verifier or all-opcode proof was found or established here.

## 2. Rest and native sends repeat known metadata work

- Severity: Medium
- Category: Performance, Dispatch, Memory
- Confidence: Confirmed operations; payoff needs measurement
- Priority: Opportunistic after correctness fixes
- Runtime impact: Avoidable parsing, string allocation, interning and argument copying on recurring calls.

`vm/dispatch.rs:1013` decodes selector text even on a cached rest-method hit, collects label strings/symbols and interns labels again. `call_rest_method_as` (`:194`) clones the rest layout and copies the incoming argument slice into a vector before constructing the retained rest tuple. The tuple itself is semantically required when exposed; the intermediate copy is a separate optimization question.

`vm/send.rs`, legacy and shape-aware primitive activation, clones the holder's class-name string and re-interns it on every invocation to populate native diagnostic fields. Legacy arguments up to eight use a stack array; larger argument slices allocate a vector. Shape-aware labeled calls can decode and allocate labels too. Do not claim an argument-vector allocation on every ordinary send.

**Direction:** immutable call-site shape descriptors with interned label IDs; cache diagnostic class symbols on stable method/class metadata; profile rest binding before replacing temporary copies with window moves. Preserve caller authority and label ordering.

**Tests/measurement:** exact versus rest, positional versus labeled, 0/1/8/9/255 arguments; allocation counts and elapsed time in default builds; GC stress for any stack-window rewrite. No speedup was measured in this checkpoint.

## 3. Per-instruction metadata is wider than the instruction stream

- Severity: Medium
- Category: Memory, Performance, Architecture
- Confidence: Confirmed sizes on the audited 64-bit host
- Priority: Defer implementation until workload memory measurements
- Runtime impact: Larger chunks and potential cache pressure, especially for code with few sends/globals.

Measured: `Value=16` bytes, alignment 8; `Bytecode=8`; `CallFrame=120`; each `Cell<Option<InlineCache>>=24`; each global cache slot also 24. `Chunk::add_instruction` allocates both cache entries for every opcode, although most sites use neither. Thus code plus cache arrays cost **56 bytes per instruction before spans**, constants, capacities and semantic side tables. SourceRange is stored separately per instruction; its size was not measured by the retained probe.

**Direction:** consider sparse or opcode-specific cache storage with a compact site index after measuring real chunk memory. Keep source spans behind `span_at`. Do not remove frame fields merely because the frame is 120 bytes: tokens, caller spans and authority guards support real behavior. Profile copy cost and hot/cold field separation first.

## 4. Public fresh-run reset bypasses close-before-truncate discipline

- Severity: Medium
- Category: VM, Runtime Safety, Maintainability
- Confidence: High confidence by source inspection; failing source sequence not executed
- Priority: Clarify before broadening embedding APIs
- Runtime impact: Reusing a VM after a failed raw run can leave open upvalues referring to discarded/reused stack slots.

`interpret.rs:147`, `run_in_module`, clears frames and stack directly. `vm/api.rs:282`, `unwind_cell`, deliberately calls `unwind_to`, which closes upvalues first. `run_cell` uses the disciplined path after reporting errors. `run_until` intentionally preserves error frames for diagnostics; that is not itself a defect.

**Direction:** either enforce a fresh-state precondition on raw execution, or route reset through close-before-truncate cleanup. Preserve traceback capture before teardown. Test failed raw run → retained closure → next run, alongside the existing REPL behavior.

## 5. Observations to retain without promoting to confirmed bugs

- **GC side tables:** `vm/gc.rs:88,113` excludes native contexts and typing registry. `typing/registry.rs` contains class handles. Associated target caches also retain receiver/method handles. Establish intended weak/strong ownership before filing another GC issue; registry roots may currently mask expiry.
- **Cache mutation:** ordinary installation increments `world_version`; public `ClassObject::add_method` and direct dictionary writes do not intrinsically enforce it. Bootstrap/new-variant installation may safely occur before observation. A live mutation gateway should own invalidation if more mutation is exposed. No stale production inline-cache result was reproduced.
- **Rust equality:** `Value` implements `Eq` while float NaN remains unequal to itself. This violates the Rust trait's reflexivity promise, but language maps use their own language hash/equality and SameValueZero path. Do not conflate it with a proven language-map bug. Scope actual Rust keyed consumers before changing numeric semantics.
- **Fusion branch inventory:** `Chunk::branch_targets` omits newer `TryInvokeExact` and `JumpIfUnsupported` offsets. However fusion leaves the original `Invoke` instruction executable at its original address. The omission alone does **not** prove wrong execution. Extend shared CFG metadata/tests if changing the transform; do not report a speculative corruption bug.
- **Construction:** factories allocate, call a hidden initializer and return the allocated instance. No rollback/transactional initialization was found. An initializer may publish `self` before failing; establish the language's required rule before calling this a violation.
- **Identity lifetime:** runtime IDs and Symbols are VM-local, not transferable identities between VMs. Generational handles are not globally unique across heaps. Reification should not silently reinterpret them as global identities.

## 6. Nested native calls lose or misattribute diagnostic context

- Severity: Medium; diagnostic correctness
- Confidence: Confirmed by a bounded legacy-native probe and the shipping JSON traceback renderer at HEAD `94b9f14361333dfda06cd8abd792f672d12db06a`.
- Scope: L02 diagnostic lifecycle. No authority escalation or source-only reproducer is claimed.

`vm/send.rs::call_method_legacy` sets singleton `native_selector`/`native_class`, calls native code, pops authority context, and clears diagnostic fields only on success. A nested call overwrites those fields without saving/restoring its parent's identity. `diagnostics/traceback.rs::render_json_traceback` and `maybe_native_frame_line` consume that pair directly.

Executed with fresh kernel VMs and zero-argument diagnostic primitives:

| Sequence | Observed result |
| --- | --- |
| Direct native failure (control) | JSON includes `Int.auditControl()` |
| Outer → nested success → outer failure | JSON has no native frame (`frames: []`) |
| Outer catches nested failure → outer failure | JSON attributes failure to `Int.auditFail()`, the caught inner call |
| Outer catches nested failure → nested success → outer success | Returns `Ok(7)`; diagnostic pair is cleared |

Evidence: [exact probe](evidence/native_lifecycle_probe.rs), [completed output](evidence/native-lifecycle-output.txt). The probe catches an ordinary `PhResult::Err`, not a Rust panic; it installs native methods on the kernel Int class. It does not prove ordinary-source reachability or complete stack/capture cleanup after a caught native error.

**Underlying cause:** active native identity and retained failure identity share one mutable pair with different lifetimes. Saving/restoring active state must also preserve the actual uncaught failure origin; simply restoring the parent on every error could misattribute an inner error that propagates unchanged.

**Direction:** distinguish scoped active diagnostic context from the error's retained origin. Cover nested success, caught/replaced errors and unchanged propagated errors in both native ABIs. Preserve flat `EnteredFrame` behavior and diagnostic capture before unwind.

**Counterevidence and limits:** both legacy and shape-aware activation push/pop `native_method_contexts` around the native function before ordinary error propagation. This supports balanced authority restoration by source inspection, not an executed access-control test. Shape-aware `Returned` has the same singleton-clearing structure; `EnteredFrame` retains the pair. Shape-aware transitions, fiber switches, GC retention of authority owners and complete post-error stack state remain unresolved.

## Recommended order and verification

First fix [001](AUD-RUNTIME-001-unchecked-operand-narrowing.md) and [002](AUD-RUNTIME-002-super-dispatch-loses-lexical-identity.md), then [003](AUD-RUNTIME-003-gc-misses-authority-edges.md) and [004](AUD-RUNTIME-004-native-entry-validation.md). Establish executable validation and API cleanup next. Measure metadata overhead and call-shape waste before selecting an optimization. Preserve the current handle arena, direct slots, cache guards and flat gateways.
