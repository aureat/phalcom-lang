# AUD-RUNTIME-S2 — Uninvestigated leads and architectural insights

This is a continuation index of sites noticed during the audit but not investigated to a conclusion. It preserves concrete reasons to look, the shortest useful next check, and reasons a suspicion might be disproved. **These are not additional confirmed bugs.** The original index contained no new reproductions. The continuation checkpoint below records subsequent investigation; no performance measurements have been run. Selected source anchors were refreshed while writing it; prefer the named functions if line numbers drift.

Confirmed findings remain in AUD-RUNTIME-001 through 004. Smaller established observations remain in [005](AUD-RUNTIME-005-grouped-contract-and-performance-findings.md). Leads below extend those records without changing their confidence or severity.

All source paths are relative to the repository root.

## Correctness leads to examine first

### L01 — Strong versus weak ownership in runtime side tables

**Pointers:** `phalcom-core/src/vm/gc.rs::collect_roots` (excludes `native_method_contexts`, `typing_registry`, `prelude_bindings`); `typing/registry.rs::RuntimeTypingRegistry`; `chunk.rs::AssociatedTargetCache`; `heap/trace.rs::trace_object`.

**Observed:** Some excluded VM structures contain handles or lead to structures that do. Associated caches contain receiver Values and method handles. The proven closure authority omission in issue 003 makes these neighboring ownership decisions worth checking.

**Unanswered:** Does another root always retain each referent? Are cache entries intentionally weak, and do readers reject stale generations before use? Does a world-version check cover reclamation as well as method changes?

**Next check:** For each table, write down its owning object, lifetime, invalidation event and consuming lookup. Attempt collection with only the table retaining the referent where that is a supported state. Do not simply make all caches strong: some weak caches deliberately allow reclamation.

### L02 — Native diagnostic and authority state across nested calls

**Pointers:** `vm/send.rs::call_method_legacy`, `call_method_with_selector_as`; `vm/mod.rs::NativeMethodContext`; `vm/gc.rs::collect_roots`.

**Observed:** Native authority uses a push/pop context stack, while `native_selector` and `native_class` are single fields that are overwritten and cleared on successful calls. The legacy activation also manipulates `switch_pending` and interprets frame-count changes after native return.

**Unanswered:** If a native call makes a nested successful send and then fails, does the diagnostic identify the outer primitive correctly? Can a caught error or allowed switch leave stale diagnostic/context state? Is every authority owner alive during re-entry?

**Next check:** One outer primitive → nested successful send → outer error; then nested error caught by outer code → later successful send. Inspect reported selector/class, context depth and stack result. This is a normal error-lifecycle question; do not assume a requirement to recover a VM after arbitrary Rust panic.

### L03 — Rest-parameter shape acceptance checks count but may not check fixed labels

**Pointers:** `parameters.rs::ParameterShape::accepts` at line 61, `binding_plan` at line 75; `method/mod.rs::RestLayout`; `vm/dispatch.rs::lookup_rest_method`, `call_rest_method_as`; `primitive/block.rs` gateways.

**Observed:** For labeled/split/complete rest, `ParameterShape::accepts` checks `args.labels.len() >= fixed_labels.len()`. For non-rest shapes it compares actual labels. `binding_plan` currently refuses labeled shapes altogether. Rest dispatch also has a separate acceptance representation.

**Unanswered:** Is the weaker count check reachable for declarations with fixed labels, or is this API currently confined to positional closures? Does another layer establish the fixed-label prefix before this check?

**Next check:** Trace actual callers before generating a test. Compare missing, reordered and substituted fixed labels with the same label count. If callers exclude such shapes, document the restriction rather than filing incorrect argument binding as a proven defect.

### L04 — Multiple arity and capture metadata representations

**Pointers:** `callable.rs::Callable` (`arity`, `parameter_shape`, `num_upvalues`, `upvalues`, `max_slots`); `method/mod.rs::Signature`; `vm/send.rs::invoke_method_object`; `primitive/block.rs::block_arity`, callable activation.

**Observed:** Signature arity, Callable arity and accepted parameter shape coexist. Reflective method invocation and ordinary sends use different entry checks. Capture count duplicates descriptor length. `max_slots` tracks compiler locals, although its comment describes maximum stack slots.

**Unanswered:** Which representation is authoritative for fixed, labeled and rest calls? Is `max_slots` only local storage metadata or ever consumed as a bound including temporaries? Can method installation or reflection construct disagreement?

**Next check:** Trace producers and consumers for one exact method, labeled method, rest method and closure. Add consistency assertions only where equality is actually required; counts with different meanings must remain distinct.

### L05 — Raw execution reuse after an error

**Pointers:** `interpret.rs::run_in_module`; `vm/api.rs::run_cell`, `unwind_cell`; `vm/dispatch.rs::unwind_to`, `close_upvalues_from`.

**Observed:** Raw execution clears frames/stack; the REPL path closes upvalues before truncating. `close_upvalues_from` assumes every open index still addresses live stack storage.

**Unanswered:** Can a supported embedding caller run another module after a failed raw run while retaining an escaped capture? Does a higher-level caller always clean up first?

**Next check:** Trace production callers, then test failure after capture creation → second raw run → invoke escaped closure. Keep diagnostic preservation on error separate from cleanup before reuse. This extends the source-inspected concern in 005; no failing sequence was executed yet.

### L06 — Scratch-slot movement and open captures

**Pointers:** `compiler/lib/expr.rs::reserve_pack_scratch`, `release_pack_scratch_from`, `emit_release_scratch_range`; `compiler/lib/loops.rs`; `vm/dispatch.rs` handlers for `ReserveScratchLocal`, `ReleaseScratchLocal`, and open-upvalue index relocation.

**Observed:** Pack/iteration lowering creates temporary locals around expressions that can call arbitrary user code. Runtime scratch handlers can shift storage and update open-upvalue indices. Release lowering has debug-only bookkeeping assertions.

**Unanswered:** Are all live aliases updated when temporary slots move? Are nested expansion, early return and fiber suspension handled consistently? Are compiler-generated locals included in the operand-limit checks from issue 001?

**Next check:** One captured local spanning a nested pack expansion, with a call or suspension inside the expansion; verify capture value before and after scratch release. Compare live and parked fiber ownership. The presence of relocation logic is evidence of deliberate handling, not evidence it is wrong.

### L07 — Map/Set re-entrant hash and equality bookkeeping

**Pointers:** `primitive/map.rs::locate_key` at line 75; `primitive/set.rs`; `primitive/mod.rs::send_hash`, `send_eq`; `heap/map.rs::MapObject`.

**Observed:** Lookup calls language hash/equality, uses a re-entrancy counter around sends, snapshots candidate indices, and re-resolves objects after calls. Numeric keys use SameValueZero. Structural mutation is guarded, while updating an existing entry's value is deliberately treated differently.

**Unanswered:** Do nested read-only lookups and caught errors restore the counter at every level? Can permitted updates invalidate any assumptions held by the outer lookup? Are all references kept live through re-entry?

**Next check:** Colliding keys whose equality performs a nested read, a permitted replacement and an attempted structural mutation; repeat with a raised error. Verify order, slot stability, lock release and GC liveness. Preserve the existing discipline of never holding a heap borrow across a language send.

### L08 — Out-of-range field reads silently become None

**Pointers:** `vm/dispatch.rs` `GetField` and `SetField`; `vm/send.rs::guard_foreign_layout_access`; `heap/instance.rs::InstanceObject`.

**Observed:** Reads use `slots.get(...).unwrap_or(Value::nil())` and surface None; writes return a bounds error. Foreign-method activations have an optional layout guard. ADT payload writes are rejected.

**Unanswered:** Is a missing slot deliberately equivalent to uninitialized storage, or does this hide a compiler/layout mismatch? Are all transplanted method and nested closure paths stamped with the guard?

**Next check:** Trace guard propagation for reflection, super, associated dispatch and nested blocks. Test incompatible field layouts through supported method invocation APIs. Establish the semantic rule before changing the read fallback.

## Identity, mutation and construction leads

### L09 — Method mutation and cache/base-name invalidation have several producers

**Pointers:** `heap/class.rs::add_method`, `set_superclass`; `vm/dispatch.rs::install_method_binding`; `vm/adt.rs`; `primitive/mod.rs` registration macros; `vm/api.rs::finalize_class_base_names`; `universe::note_method_installed` call sites.

**Observed:** The ordinary installation path increments `world_version`; lower-level dictionary writes do not intrinsically do so. Class `base_names` is a flattened index. Sacred inlining has a separate override guard mechanism.

**Unanswered:** Can any live mutation update the method dictionary without updating ordinary cache validity, base-name visibility or sacred guards? Are the direct ADT writes restricted to unpublished classes?

**Next check:** Inventory only mutation sites, classify bootstrap versus observable runtime mutation, and compare their required notifications. Warm a relevant lookup before each supported mutation. No stale production cache result has been demonstrated.

### L10 — Runtime typing identity after class replacement

**Pointers:** `typing/registry.rs::register_nominal_binding`, `resolve_nominal`, `declaration_for_nominal`; `vm/api.rs::create_class`; `compiler/lib/class_decl.rs` REPL replacement handling.

**Observed:** The registry stores declaration→class and class→declaration mappings separately. Runtime class creation registers a binding where a stable declaration identity is available. REPL replacement creates fresh class handles.

**Unanswered:** Which declaration identity describes a replacement, and what should reverse lookup report for an old surviving class? Can forward and reverse tables retain contradictory or stale associations? Which entries are intended to survive GC?

**Next check:** Keep old/new class instances alive through replacement, query both identity directions, then collect obsolete objects. Resolve intended historical-versus-current semantics before changing table retention.

### L11 — Partially initialized object escape and constructor failure

**Pointers:** `compiler/attributes.rs::lower_constructors`; `primitive/class.rs::class_new_`; constructor return handling in `compiler/lib/mod.rs`; hidden initializer sends.

**Observed:** Factory code allocates an instance, invokes a hidden initializer and returns the instance. Storage initially contains private Nil, surfaced as None on reads. No transactional rollback was established. An initializer can potentially publish self through another object or global before failing.

**Unanswered:** Is such escape permitted by the language? Are const-field and invariant guarantees intended to hold before completion or only for successfully constructed objects? What happens when superclass initialization fails?

**Next check:** Read the effective constructor/invariant rule, then test escaped self after failure and explicit returns inside initialization. Do not infer transactional construction as a requirement merely from the absence of rollback.

### L12 — Unwired classes and mutable hierarchy assumptions

**Pointers:** `heap/class.rs::ClassObject::bare`, `lookup_method_in_hierarchy`, `is_strict_subclass`; `vm/api.rs::create_single_class`, `create_class`; compiler superclass-cycle checks; `primitive/class.rs::class_set_superclass`.

**Observed:** Bare classes start with an unset metaclass handle. Bootstrap intentionally allocates then patches. Hierarchy walks assume eventual termination. Public Rust setters can construct states that source paths may prohibit.

**Unanswered:** Can incompletely wired classes reach dispatch or a safepoint through supported APIs? Does every supported hierarchy mutation preserve acyclicity? Is the Rust helper explicitly a trusted construction-only boundary?

**Next check:** Trace publication points rather than attacking arbitrary public fields. Determine whether current source APIs permit hierarchy mutation at all. A manually constructed cycle is not proof of a source-reachable defect.

### L13 — Invalid runtime variant IDs degrade to Object behavior

**Pointers:** `value/mod.rs::Value::class`; `adt.rs::RuntimeVariantId`, registry insertion/lookup; `heap/adt.rs::AdtCaseObject`; `vm/adt.rs`.

**Observed:** For an unregistered singleton/case variant ID, class lookup falls back to the Object class. Runtime variant IDs are publicly constructible Rust wrappers and are VM-local.

**Unanswered:** Is the fallback a deliberate defensive policy or a way to conceal corrupt identity? Can normal loading/redefinition produce a missing descriptor, or only trusted Rust callers? Are payload shape and registry identity checked at every construction boundary?

**Next check:** Trace normal ADT registration and executable construction before proposing stricter errors. Preserve the distinction between semantic VariantId, runtime identity and physical discriminant.

## Lower-priority contract and performance insights

### L14 — Rust Eq versus language numeric equality

**Pointers:** `value/repr.rs::PartialEq`, `impl Eq for Value`, `Hash`; `value/mod.rs::value_eq`, `same_value_zero`; `heap/map.rs`.

**Observed:** NaN is unequal to itself in Rust Value comparison despite the Eq marker. Language Map/Set uses a different protocol; signed zero hashing is normalized.

**Next check:** Find actual Rust HashMap/HashSet or deduplication consumers keyed by Value and determine their expected NaN behavior. The trait mismatch is observed; a language-map failure is not. Avoid “fixing” language numeric equality to satisfy an unrelated Rust container.

### L15 — Transformation control-flow inventory can drift

**Pointers:** `chunk.rs::branch_targets`, `fuse_superinstructions`; `compiler/lib/jumps.rs`; `compiler/lib/expr.rs` bilateral lowering.

**Observed:** Branch discovery omits newer `TryInvokeExact` and `JumpIfUnsupported` offsets. Fusion currently leaves the original Invoke in place and skips it only on the fused execution path.

**Next check:** Compare branch forms across patching, validation and transformations. Test entry at both instructions of a fused pair. **Do not promote the missing inventory entries alone to a corruption finding:** the retained original Invoke is a material counterargument. Shared opcode metadata becomes more valuable if future fusion compacts code.

### L16 — Module-export sends bypass parts of ordinary dispatch

**Pointers:** `vm/send.rs::try_module_export_send`, `try_module_export_send_dynamic`; `vm/dispatch.rs::invoke_at`.

**Observed:** Module-export dispatch is checked separately from the ordinary class-method path, including around cache handling. Earlier tests distinguish zero-argument method invocation from getter access.

**Next check:** Compare static and dynamic sends for exported values, callable exports, absent exports and same-named module methods. Verify getter/method distinction, arity and visibility. A special path is not inherently wrong; parity is the question.

### L17 — Bilateral operator sends lack the ordinary inline cache

**Pointers:** `compiler/lib/expr.rs` bilateral lowering; `vm/dispatch.rs` `BilateralPreferReflected`, `TryInvokeExact`, `ValidateOrdering`.

**Observed:** Numeric/operator execution can perform subclass/preference checks and hierarchy lookup outside the ordinary Invoke cache. This means a benchmark of normal method sends does not directly describe arithmetic cost.

**Next check:** Measure monomorphic arithmetic and reflected/subtype cases separately. Any cache must preserve reflected priority, Unsupported fallback and method invalidation. No worthwhile speedup has yet been measured.

### L18 — Capture closing, object-category checks and cold metadata costs

**Pointers:** `vm/dispatch.rs::close_upvalues_from`, `Closure`, `GetField`, `run_until_inner`; `frame.rs::CallFrame`; `heap/closure.rs::ClosureObject`; `chunk.rs::Chunk`.

**Observed:** Closing captures collects an index vector before BTreeMap removal. Closure creation clones capture descriptors and allocates closure/block representations. Field access can attempt several typed arena lookups. Frames include source/authority/token metadata and are copied in the loop. Cache arrays occupy all instruction sites.

**Next check:** Profile these separately in capture-heavy, field-heavy and arithmetic workloads. Count allocations rather than inferring them from empty Vec construction. Avoid removing required roots/tokens or changing the proven callable-hoisting guard for speculative locality gains. Rest metadata and per-instruction cache footprint are already documented in 005.

## Suggested follow-up order

1. L01–L06: ownership, native lifecycle, binding consistency and capture/stack integrity.
2. L07–L13: re-entrant collections, representation guards, mutation, replacement and construction semantics.
3. L14–L18: container contracts, transformation completeness and measured performance.

Start each lead with its named producer/consumer pair. Close it as **confirmed issue**, **disproved**, **intentional contract**, or **still unresolved**, recording the shortest supporting evidence. Promote only established, consequential problems to numbered issue documents. Keep fixes outside the audit until implementation is requested.

## Continuation checkpoint — 2026-09-07

Examined HEAD `94b9f14361333dfda06cd8abd792f672d12db06a` with unrelated dirty semantic/LSP/documentation work preserved. The reviewed `vm/send.rs`, `parameters.rs`, `method/mod.rs`, `primitive/block.rs` and traceback renderer have no diff from the original audited HEAD. This is a bounded continuation, not revalidation of all original findings.

- **L02 — confirmed diagnostic issue; other lifecycle dimensions unresolved.** Four legacy-native sequences executed with the shipping JSON renderer. Nested success erases the outer failure frame; caught inner failure misattributes a subsequent outer failure. Recorded in [005 §6](AUD-RUNTIME-005-grouped-contract-and-performance-findings.md#6-nested-native-calls-lose-or-misattribute-diagnostic-context). Ordinary authority push/pop is balanced by source inspection in both ABIs; no authority behavior, GC, fiber-switch or complete stack-cleanup probe was executed.
- **L03 — disproved for current compiler-produced wrong-label binding; intentional positional closure restriction.** `ParameterShape::closure` always sets empty fixed labels and only positional rest. Production constructors are `compiler/lib/mod.rs` and `vm/adt.rs`. `primitive/block.rs::block_call` and `primitive/fiber.rs` pass positional argument shapes to `accepts`; the shape-aware `vm/send.rs::activate_closure_call` rejects labeled arguments before binding. Labeled method rest uses `RestLayout::accepts` (`method/mod.rs:73`), which requires actual labels to start with fixed labels; ordinary rest lookup and captured-method activation call that predicate. The count-only branch remains a latent public metadata/API weakness if labeled ParameterShape construction is introduced, not a demonstrated current source binding defect. Source inspection only; no new binding test run.

Next bounded work: L01 ownership or L05 raw-run reuse. L02's shape-aware `EnteredFrame`, fiber-switch and authority-liveness questions remain open. All other lead statuses are unchanged. Do not rerun the preserved native lifecycle probe without drift or an expanded contract to test.
