---
id: LANG007.C4.P3-interim-handoff
kind: handoff
plan: LANG007.C4.P3
checkpoint: LANG007.C4
created: 2026-10-05
for: next implementer (taking over LANG007.C4.P3 through closure)
---

# LANG007.C4.P3 — Interim Implementation Handoff

## 1. Executive Summary & Active Status

- **Plan**: `docs/implementation/LANG007/LANG007.C4/LANG007.C4.P3/LANG007.C4.P3-luna-patch-grade-implementation-plan.md`
- **Checkpoint**: `docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md`
- **State**: `IN_PROGRESS` (Completion: `PARTIAL`, Verification: `UNVERIFIED`)
- **Current Head**: Commit `b9f72cfe` on `main`, synchronized with `origin/main`. The worktree has uncommitted lifecycle-record updates and one test file with a T7 root-versus-slice equality reproducer.

### Current continuation update — 2026-10-05

This update supersedes the old T4–T7 task status and next-action text below;
the active numbered plan and C4 checkpoint remain authoritative.

- The semantic T1 source-formation continuation, T4 construction, T5 rest
  convergence, and T6 spreading work are committed and pushed. The source-name
  decision preserves C3 `TypePack`, `LabeledTypePack`, and `CompleteTypePack`;
  proper Pack aliases are `PositionalPack<T>`, `LabeledPack<T>`, and
  `CompletePack<T>`.
- Full-Universe rest-capture integration passes. T7 source-view testing passes
  for empty/Entry variants, labels/order, and repeated O(1) root-backed slices.
- A second source equality test fails at `root == slice` for a root and an
  equivalent visible slice. Its test-only reproducer is uncommitted in
  `phalcom-core/tests/core/collections/outgoing_packs_completion.rs`. The cause
  was not diagnosed; later hash/order assertions in that test are unverified.
- Next: diagnose the root/slice source equality path; finish T7 equality/hash,
  rendering, and no-Iterator boundary evidence; complete T8.1–T8.6 and serial
  G1–G4; then perform T9 lifecycle closure with the P3 walkthrough and final
  handoff. P3 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`.

This handoff captures established architecture, current evidence, and the remaining tasks required to bring **LANG007.C4.P3** to full completion and lifecycle closure.

---

## 2. Completed Milestones (Do Not Redesign or Rediscover)

### T0: Baseline and Takeover (COMPLETE)
- Working tree clean, toolchain verified.

### T1: Canonical Pack Type, Typed Remainder Syntax, Metadata & Bridge (COMPLETE)
- **Commit**: `81db5030` (`feat(semantic): add proper Pack type, typed remainder syntax, and stable metadata (LANG007.C4.P3 T1)`)
- **Key Artifacts**:
  - AST remainder syntax: `GenericArgumentSyntax::Remainder(TypePackRemainderSyntax)` with `ExpansionMode`.
  - Proper Pack type in semantic model: `PackType`, `PackTypeElement`, `TypeData::Pack(PackType)`, `TypeData::PackFromPacks { positionals, labeled }`.
  - TypeStore canonical formation: `TypeStore::pack(elements, pos_remainder, lab_remainder)` and `TypeStore::pack_from_packs(positionals, labeled)`.
  - Core declaration IDs: `CoreDeclarationIds::default().pack` bound to Universe `Pack`.
  - Semantic metadata format: `TypeNode::Pack`, `TypeNode::OpenPack`, `TypeNode::PackFromPacks` with round-trip encoding/decoding.
  - Verified with 7 focused tests in `phalcom-semantic/tests/semantic/foundations/pack_types.rs`.

### T2: StaticPack Storage, Slicing, Equality & Variadic Static Parameters (COMPLETE)
- **Commit**: `0cbd52f5` (`feat(semantic): add StaticPack storage, static construction, and variadic static parameters (LANG007.C4.P3 T2)`)
- **Key Artifacts**:
  - `StaticPackBackingId`, `StaticPackId`, `StaticPackBacking`, `StaticPackData`, `StaticValueData::Pack(StaticPackId)`.
  - O(1) slicing over immutable backing; interning `(backing, ranges)` in `StaticValueStore`.
  - Logical equality, hashing, and fingerprinting ignoring internal slice/backing offsets.
  - Value-domain variadic parameter modes: `GenericParameterMode` attached directly to `StaticParameter` in `phalcom-semantic/src/static_definition.rs`.
  - Checked variadic capture in `phalcom-semantic/src/checker/static_body.rs`.
  - Verified with 9 focused tests in `phalcom-semantic/tests/semantic/foundations/static_values_pack.rs` and `static_definitions.rs`.

### T3: Runtime Pack Representation, Root/Slice Object, EmptyPack, GC & Floor Primitives (COMPLETE)
- **Commit**: `ff986f4c` (`feat(core): add runtime Pack root/slice object, EmptyPack immediate, product shape, and GC tracing (LANG007.C4.P3 T3)`)
- **Key Artifacts**:
  - Immediate `ValueTag::EmptyPack`: `Value::empty_pack()`, `Value::is_empty_pack()`, mapped to canonical `pack_class`.
  - Product model: `AnonymousProductKind::Pack`, `ProductShape::Pack(PackProductShape)`.
  - Heap representation in `phalcom-core/src/heap/pack.rs`:
    - `PackStorage::Root { descriptor, storage }`
    - `PackStorage::Slice { root: ObjRef }` (slice-of-slice flattens to root directly).
    - `Object::Pack(PackObject)`.
  - O(1) non-allocating accessor `PackView`: `vm.pack_view(obj)` and `vm.pack_view_value(val)`.
  - Runtime construction primitive: `crate::product::finish_pack(vm, positionals, labeled)`.
  - GC trace: Roots trace `ProductStorage` value slots; slices trace the root `ObjRef`.
  - Verified with 5/5 tests in `phalcom-core/tests/core/collections/pack.rs` and 3/3 GC tests in `phalcom-core/tests/core/memory/pack_gc.rs`.

### T4.1: Scalable Duplicate Label Indexing in Builder (COMPLETE)
- **Commit**: `720c3445` (`feat(core): implement scalable label membership indexing in ArgumentPackBuilderObject (LANG007.C4.P3 T4.1)`)
- **Key Artifacts**:
  - In `phalcom-core/src/heap/pack_builder.rs`:
    - `const LABEL_INDEX_THRESHOLD: usize = 8;`
    - Added `label_index: Option<HashSet<Symbol>>` to `ArgumentPackBuilderObject`.
    - Small counts (< 8) use linear scan; at threshold or above, lazily builds `HashSet<Symbol>` for O(1) membership checks.
    - `take_parts()` clears `label_index = None` and resets builder state.
    - Verified with unit tests covering threshold crossover and duplicate rejection.

---

## 3. Original Task Breakdown (Superseded Where It Conflicts With the Current Update)

Use the ordered current next steps in the continuation update and the active
plan. The detailed T4–T6 instructions below are historical and already
implemented; they are retained as architectural context only.

The following original breakdown predates later implementation. Use it as design
reference only; the active plan contains current task status and exact closure
gates.

### Phase 1: Complete T4 (Bytecode Lowering & First-Class Pack Construction)

#### T4.2: Opcodes in Bytecode
1. In `phalcom-core/src/bytecode.rs`:
   - Add opcode `FinishPack` (stack effect: `[..., builder] -> [..., pack]`).
   - Add opcode `BuildStaticPack { spec: u16 }` (stack effect: `[..., v0, ..., vn] -> [..., pack]`, source order).
   - In `BYTECODE_NAMES`, append `"FinishPack"` and `"BuildStaticPack"`.
   - Update `Bytecode::VARIANTS` count (currently 112 -> 114) and update `Bytecode::index()` match arms.
   - Update `Bytecode::instruction_length()` if instruction has arguments.

#### T4.3: Product & VM Dispatch Implementation
1. In `phalcom-core/src/modules/semantic_lowering.rs`:
   - Extend `AnonymousProductConstructionKind` with variant:
     ```rust
     Pack {
         positional_len: u32,
         labels: Box<[Box<str>]>,
     }
     ```
2. In `phalcom-core/src/product/mod.rs`:
   - Add `finish_pack_from_spec(vm, spec: &AnonymousProductConstructionLoweringSpec, source_values: &[Value]) -> Result<Value, ProductBuildError>`:
     - Splits `source_values` according to `positional_len` and `labels`.
     - Allocates descriptor and constructs Pack root (or returns `Value::empty_pack()` if empty).
3. In `phalcom-core/src/vm/dispatch.rs`:
   - Handle `Bytecode::FinishPack`:
     - Pop `builder_val` (must be `Object::ArgumentPackBuilder`).
     - Check `!builder.has_pending()`.
     - Extract `(pos, labels, vals) = builder.take_parts()`.
     - Call `crate::product::finish_pack(self, pos, labeled_pairs)`.
     - Push resulting Pack value.
   - Handle `Bytecode::BuildStaticPack { spec }`:
     - Lookup spec from `callable.chunk.executable_semantics.anonymous_product_spec(spec)`.
     - Pop `spec.layout.components.len()` values from stack in reverse order.
     - Call `crate::product::finish_pack_from_spec(self, spec, &values)`.
     - Push resulting Pack value.

#### T4.4: Semantic Lowering Projection & Compiler Generation
1. In `phalcom-core/src/modules/semantic_lowering.rs`:
   - In `collect_anonymous_product_expr` / `collect_anonymous_product_statements`:
     - Inspect `Expr::UnqualifiedCall(call)` where `call.name == "Pack"`.
     - Confirm via `snapshot`: does this call resolve to canonical `CoreDeclarationIds::default().pack`?
     - If all items in `call.args` are static (positional or static label, NO spread), collect as `AnonymousProductSource::Pack`.
   - In `project_anonymous_products`:
     - For `AnonymousProductSource::Pack`, construct `AnonymousProductConstructionLoweringSpec` with `AnonymousProductConstructionKind::Pack`.
2. In `phalcom-core/src/compiler/lib/expr.rs`:
   - In `compile_expr`:
     - When compiling `Expr::UnqualifiedCall` for `Pack(...)`:
       - If lowering attached an `AnonymousProductConstructionLoweringSpec` with `Kind::Pack`:
         - Emit component expressions in order.
         - Emit `Bytecode::BuildStaticPack { spec: spec_idx }`.
       - If dynamic/contains spread:
         - Reserve scratch local for builder: `let builder_slot = self.reserve_pack_scratch("$pack_builder", call.range)?;`
         - Emit `Bytecode::NewArgumentPack`.
         - Emit `Bytecode::SetLocal(builder_slot)`.
         - Emit `Bytecode::Pop`.
         - Compile dynamic items via `self.compile_dynamic_pack_items(builder_slot, call.args)?`.
         - Emit `Bytecode::GetLocal(builder_slot)`.
         - Emit `Bytecode::FinishPack`.
         - Release scratch local.
3. In `phalcom-core/tests/core/collections/outgoing_packs_completion.rs`:
   - Add required T4 tests:
     - Fixed ordinary call -> emits `Invoke`, no builder / `FinishPack` / `BuildStaticPack`.
     - Fixed `Pack(...)` -> emits `BuildStaticPack`, no builder.
     - Spread `Pack(1, *x)` -> emits `NewArgumentPack ... FinishPack`.
     - Spread Tuple `(1, *x)` -> emits `NewArgumentPack ... FinishTuplePack` (not `FinishPack`).
     - Multi-spread `Pack(*a, *b)` -> emits single `NewArgumentPack ... FinishPack`.

---

### Phase 2: Implement T5 (Rest Capture Convergence)

#### Objective
Ensure all rest parameter binding (methods, blocks, fibers) uniformly captures into immutable Pack instead of Tuple or List.

1. **Central capture helper in VM**:
   - In `phalcom-core/src/vm/mod.rs` (or `dispatch.rs` / `call.rs`):
     - `fn capture_rest_pack(&mut self, positionals: &[Value], labeled: &[(Symbol, Value)]) -> Result<Value, VMError>`:
       - Uses `crate::product::finish_pack(self, positionals.to_vec(), labeled.to_vec())`.
2. **Method rest binding**:
   - Locate `call_rest_method_as_with_environments` in `phalcom-core/src/vm/` (or wherever `RestMode` is handled).
   - Update:
     - `Positional`: pack extra positional arguments into `Value` via `capture_rest_pack`.
     - `Labeled`: pack extra labeled arguments into `Value` via `capture_rest_pack`.
     - `Split`: pack extra positional into first rest slot, extra labeled into second rest slot.
     - `Complete`: pack extra positional and extra labeled into single complete Pack.
3. **Block & Fiber rest**:
   - Update block invocation rest capture: replace `finish_tuple` with `capture_rest_pack`.
   - Audit `Fiber` argument handling: ensure language-level excess arguments bind as Pack.
4. **Census of legacy rest paths**:
   - Search for `finish_tuple` and `alloc_list` in call activation / parameter binding. Ensure all language rest captures use Pack.
5. **Tests**:
   - Add/run focused tests verifying method `*`, `**`, `***` rest parameter capture, block rest capture, and forwarding.

---

### Phase 3: Implement T6 (Spreading, Forwarding, Tuple Interop, Fused Multi-spread)

#### Objective
Make Pack the first-class structural transport for `*`, `**`, `***` without requiring `Iterator`.

1. **Positional expansion (`*pack`)**:
   - In `phalcom-core/src/vm/dispatch.rs` (`Bytecode::PackTryExpandTuplePositionals` and list literal equivalent):
     - Inspect operand:
       - If `Value::is_empty_pack()` -> advance immediately (0 elements).
       - If `Object::Pack(pack)` -> retrieve `PackView`. Append its visible positional slice directly to builder/list.
       - If `Object::Tuple` -> preserve existing fast path.
       - Fallback to generic `Iterator` only for non-structural operands.
2. **Labeled & complete expansion (`**pack`, `***pack`)**:
   - In `PackExpandLabels` / `PackExpandComplete`:
     - If `Object::Pack`: iterate visible labeled / complete lanes directly and append to destination `ArgumentPackBuilder`.
3. **Explicit Pack ↔ Tuple conversions**:
   - Ensure `Pack(*(1, 2))` and `(*pack)` evaluate cleanly and produce the respective distinct types.
4. **Fused multi-spread**:
   - Verify `foo(*a, *b, *c)` and `Pack(*a, *b, *c)` share a single `NewArgumentPack` accumulator and do not perform repeated intermediate copying.
5. **Tests**:
   - Extend `outgoing_packs_completion.rs` with T6 test cases (Pack positional/labeled/complete spread, rest Pack forwarding, duplicate detection during spread).

---

### Phase 4: Implement T7 (Source Surface & Views: collections/pack.ph)

1. In `phalcom-core/core/universe/src/collections/pack.ph`:
   - Declare canonical surface types and view types:
     - `PositionalPackView`: `Empty` and `Entry(value, rest)`
     - `LabeledPackView`: `Empty` and `Entry(label, value, rest)`
   - Wire native floor methods on `Pack`:
     - Length getters: `positionalLength()`, `labeledLength()`
     - Accessors: `positionalAt(index)`, `labeledAt(index)`
     - Slice methods: `positionalRest()`, `labeledRest()` returning O(1) Pack slices.
   - Implement language `==` and `hash` on `Pack` traversing visible lane values.
2. Verify with unit tests in `phalcom-core/tests/core/collections/pack.rs`.

---

### Phase 5: Implement T8 & T9 (Incrementality, Verification Gates & Lifecycle Closure)

1. **Audit exhaustive enums**:
   - Verify all matches on `TypeData`, `StaticValueData`, `Object`, `ValueTag`, `ProductShape` have proper `Pack` arms.
2. **Run Verification Gates**:
   - **G1**: Semantic type/static Pack vertical:
     ```sh
     RUSTFLAGS='' cargo test -p phalcom-ast pack_type_remainder -- --nocapture
     RUSTFLAGS='' cargo test -p phalcom-type-meta pack -- --nocapture
     RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::pack_types -- --nocapture
     RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::static_values_pack -- --nocapture
     ```
   - **G2**: Runtime Pack/rest/spread vertical:
     ```sh
     RUSTFLAGS='' cargo test -p phalcom-core --test core collections::pack -- --nocapture
     RUSTFLAGS='' cargo test -p phalcom-core --test core memory::pack -- --nocapture
     RUSTFLAGS='' cargo test -p phalcom-core --test core core_outgoing_packs_completion -- --nocapture
     ```
   - **G3**: Fixed-call/performance check (disasm inspection confirming fixed calls emit zero Pack bytecodes).
   - **G4**: Compatibility check:
     ```sh
     RUSTFLAGS='' cargo check -p phalcom-ast -p phalcom-type-meta -p phalcom-semantic -p phalcom-core
     ```
3. **Close P3 Lifecycle**:
   - Update `LANG007.C4-CHECKPOINT.md`: mark P3 as `COMPLETE`, `IMPLEMENTED`, `FOCUSED_TESTED`.
   - Update `LANG007.C4.P3-luna-patch-grade-implementation-plan.md` status to `COMPLETE`.
   - Create `LANG007.C4.P3-walkthrough.md`.
   - Create final `LANG007.C4.P3-handoff.md`.

---

## 4. Key Architectural Constraints & Rules

1. **No surface Pack before dynamic send**: Dynamic method calls (`InvokePack`, `SuperSendPack`) directly consume `ArgumentPackBuilderObject`. A surface `Pack` value must **never** be created solely to execute a dynamic call.
2. **Fixed calls are untouched**: Fixed calls with statically known parameters must never touch `ArgumentPackBuilderObject`, `FinishPack`, or `BuildStaticPack`. They use direct `Invoke`.
3. **No string matching on `"Pack"` for lowering**: Verify declaration identity via `snapshot` and `CoreDeclarationIds::default().pack` or `IntrinsicRuntimeOwner::Pack`.
4. **BUILD Mode discipline**: Do not run broad test suites (such as `language-corpus` or whole workspace tests) during implementation steps. Use targeted, discriminating test filters.
