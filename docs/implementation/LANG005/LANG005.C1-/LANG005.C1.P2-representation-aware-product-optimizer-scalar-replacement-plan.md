# LANG005.C1.P2 — Representation-Aware Product Optimizer, Scalar Replacement, and Allocation Sinking Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to execute this document checkpoint-by-checkpoint. Every checklist item is part of the implementation contract unless the drift protocol explicitly replaces only a mechanical name/path.

**Goal:** Build the first representation-aware product optimizer on top of LANG005.C1.P1 so non-escaping `data` values and provably non-observable exact enum payloads can remain virtual, live as scalar component slots, project directly without aggregate allocation, and materialize through P1's canonical `ProductLayout` / `ProductStorage` boundary only when execution actually requires a runtime `Value`.

**Architecture:** Keep Phalcom's existing direct AST-to-stack-bytecode compiler. Do **not** introduce a general SSA/IR rewrite in this checkpoint. Add one backend-only intraprocedural product planning layer in `phalcom-core`: it consumes P1's already-resolved semantic lowering attachments, walks lexical AST use structure only to prove escape/capture/use shape, and selects a conservative emission strategy. Optimized product locals occupy ordinary `Value` frame slots, one per scalar leaf, so the VM stack/call ABI remains unchanged. `data` may be rematerialized because PDR-0035 forbids observable allocation identity; general enum cases may be virtualized only when the compiler proves no whole-case/identity-observing use, because PDR-0035 does not change enum allocation identity. P1 bytecode remains the canonical fallback and is always available through an optimizer-disabled test seam.

**Tech stack:** Rust 2024 workspace; `phalcom-core` AST-to-bytecode compiler; P1 data/product lowering specs; existing `ReserveScratchLocal` / `ReleaseScratchLocal` stack machinery; `ModuleLoweringSemantics`; `Chunk` and `ExecutableSemanticPool`; existing ADT match lowering; Criterion VM benchmark harness.

**Governing decisions and prerequisite:** PDR-0035 and LANG005.C1.P1. P2 does not add new language semantics. P1 must be COMPLETE before P2 implementation begins.

---

# 1. Repository grounding

This plan was prepared against remote `main`:

```text
repository: aureat/phalcom-lang
branch inspected: main
remote HEAD: b38adeb1ebf72550d09e8a8494ce67dbd610d82f
HEAD message: docs: add time plan and language pattern matching support fixtures
inspection date: 2026-09-12
```

At that revision, first-class LANG005 `data` / P1 product-layout support is not present on `main`; searching the tree for LANG005 data implementation returned no implementation result. Therefore this P2 document is intentionally **P1-dependent**.

Before touching code, the implementer must open:

```text
docs/implementation/LANG005-data-impl-traits/
  C1-optimized-immutable-products/
    implementation-state.md
```

and verify that P1 is marked COMPLETE. The P1 state file, not this pre-P1 repository snapshot, is authoritative for the **final mechanical names** of:

```text
DataInfo
DataComponentId
DataConstructorId
DataConstructionLoweringSpec
DataComponentLoweringSpec
ProductLayoutSpec
ProductLayoutId
ProductStorage
RuntimeDataDescriptorId
ConstructData
GetDataComponent
the general-enum ProductStorage lowering fields
```

If P1 shipped the same semantics under mechanically different Rust names, substitute those final names throughout P2 and record the mapping in the state file. This is a drift adaptation, not permission to redesign the P1 contract.

P2 relies on P1's final invariant:

> A data/enum product may remain purely logical in the compiler and may be materialized through one representation-independent `ProductLayout` / `ProductStorage` boundary only when execution requires a runtime value.

## 1.1 Current compiler facts verified before this plan

The current tree establishes the following architectural constraints:

- `phalcom-core/src/compiler/lib/mod.rs::Compiler` lowers AST directly to stack bytecode; it owns `functions: Vec<FunctionState>` and an optional `ModuleLoweringSemantics`.
- `FunctionState` owns `Chunk`, `locals`, `num_locals`, `max_slots`, and upvalue metadata. Local vector position currently doubles as the runtime frame-slot number.
- `Chunk::fuse_superinstructions()` is the existing post-emission optimizer. It performs safe local peephole fusion without relayout; there is no general SSA/CFG optimizer.
- `ModuleLoweringSemantics` is the formal semantic-to-codegen projection boundary. It already carries resolved enum/associated/match facts and must remain the source of resolved data/variant identity after P1.
- `CallableAnalysis` contains expression types, bindings, and a semantic control-flow graph, but that graph is statement/control-level rather than an expression-level SSA def-use graph.
- `ReserveScratchLocal` and `ReleaseScratchLocal` already provide runtime-supported insertion/removal of compiler scratch slots, including relocation bookkeeping for open upvalues.
- `compiler/lib/expr.rs::reserve_pack_scratch` and `release_pack_scratch_from` are the nearest safe compiler precedent for temporary contiguous slot regions.
- `docs/audit/runtime-2026-09-07/AUD-RUNTIME-S2-uninvestigated-leads-and-insights.md` L06 explicitly flags scratch movement plus open captures as an area that requires hostile testing; the relocation mechanism is deliberate, not automatically suspect.
- `docs/work/deferred/destructuring-scratch-slots.md` documents why arbitrary local-slot reclamation is dangerous: local indices and upvalue arithmetic rely on stable slot numbering.
- `docs/work/deferred/match-lowering-optimizations.md` explicitly requires backend match optimizations to consume resolved semantic candidates and forbids compiler-side variant lookup, exhaustiveness solving, or GADT solving.
- `phalcom-core/benches/vm_bench.rs` and `benchmarks/vm/run.sh` provide the existing Criterion performance harness.

## 1.2 Why P2 is not a general IR project

P2 needs escape/use proofs for a narrow immutable product class. Replacing Phalcom's direct emitter with a general SSA IR would simultaneously rewrite:

- lexical locals/upvalues;
- sacred-call guarded inlining;
- block/non-local-return lowering;
- pack scratch slots;
- pattern staging;
- source-span/IP tables;
- inline/global caches;
- bytecode fusion.

That is disproportionate to C1.P2 and would erase the diagnostic value of P1's canonical bytecode fallback.

The chosen architecture is therefore:

```text
Semantic snapshot
      |
      v
P1 ModuleLoweringSemantics
  - exact data constructor target
  - exact component identity
  - exact variant target
  - exact match candidates
  - P1 materialization recipes/layout
      |
      v
P2 product planning prepass
  - lexical use/escape classification only
  - no type inference
  - no member/variant resolution
      |
      +----------------------------+
      |                            |
      v                            v
canonical P1 lowering         virtual-product lowering
(materialized Value)          (component Value slots)
      |                            |
      +-------------+--------------+
                    |
                    v
               normal Chunk
                    |
                    v
       existing fuse_superinstructions()
                    |
                    v
                    VM
```

P2 is therefore an emission optimization, not a second semantic pipeline.

---

# 2. Non-negotiable optimizer laws

1. **P1 is canonical fallback.** With product optimization disabled, compilation and execution use P1's ordinary materialized data/enum path unchanged.
2. **No new semantic lookup.** P2 may inspect AST lexical structure, but constructor identity, component identity, exact variant identity, argument-to-component mapping, and match candidates come only from P1 `ModuleLoweringSemantics`.
3. **No type-system policy in the backend.** P2 never infers a generic type, solves a GADT, decides exhaustiveness, selects a family member, or compares declaration names to guess identity.
4. **Evaluate exactly once.** Every source constructor argument executes exactly once.
5. **Preserve source evaluation order.** Arguments execute in the same source order as canonical P1 lowering, including labeled arguments.
6. **Do not skip unused arguments.** Projection of one component does not permit dropping evaluation of other constructor arguments; they may throw, allocate, mutate, suspend, or otherwise have effects.
7. **Data rematerialization is legal.** A virtual data value may be reconstructed at a whole-value boundary because PDR-0035 removes observable backing allocation identity.
8. **Enum rematerialization is not assumed legal.** A general enum case may be virtualized only when no whole-case value can be observed. If an enum value must escape, be compared as a whole, be sent a method, be returned, be captured, or be bound whole by a pattern, compile the candidate through canonical P1 materialization.
9. **Native `Option` is not a P2 target.** It already has a specialized immediate representation. Do not route it through the general virtual-enum path merely for architectural symmetry.
10. **Only local immutable bindings are virtualized in P2.** Globals, fields, parameters, mutable/reassigned locals, destructuring bindings, and captured locals use canonical materialization.
11. **No interprocedural ABI change.** Parameters and return values remain ordinary `Value`s. P2 may sink materialization to a call/return boundary but never changes method/block signatures or frame entry ABI.
12. **Virtual leaves are ordinary `Value` frame slots.** P2 scalar replacement means aggregate-to-component scalar replacement in the bytecode VM; it does not introduce an untagged integer/float register file.
13. **P1 physical layout stays authoritative when materialized.** P2 does not copy or reinterpret `ProductLayout` rules.
14. **No anonymous tuple/record takeover.** C1.P3 owns anonymous product representation convergence.
15. **No hidden user-visible identity.** Optimization choice is not reflectable and does not alter `.class`, type reification, equality/hash, or presentation after a materialization boundary.
16. **No diagnostic drift from skipped semantic work.** Optimized and disabled compilation must report the same source diagnostics. P2 is allowed to change executable bytecode, allocation count, and performance only.
17. **No optimizer-induced semantic `Dynamic`.** Incomplete proof means baseline materialization, not a weakened type fact.
18. **No speculative runtime guard is required.** P2 acts only on compile-time proven product identities. If proof is absent, it falls back.
19. **No new semantic DB query solely for optimization.** The plan is derived transiently from AST + already-published lowering facts unless repository drift proves a durable compiler query is already the canonical architecture.
20. **Existing sacred-call guards remain untouched.** Product optimization must compose with fast/fallback copies rather than broadening or bypassing sacred override guards.

---

# 3. Materialization boundary matrix

The following is the P2 policy floor.

| Use of a virtual value | `data` | general exact enum case |
|---|---|---|
| direct declared component projection | stay virtual | not a source-level general operation; match payload projection may stay virtual |
| nested declared data projection | stay virtual recursively | enum payload itself stays a scalar `Value` unless separately proven by enum rules |
| local immutable alias | P2 fallback initially | fallback |
| call/send argument | materialize at use | candidate ineligible; canonical materialized binding |
| receiver of ordinary method/getter | materialize at use | candidate ineligible |
| return / non-local return | materialize at use | candidate ineligible |
| assignment to global/field/index/collection | materialize at use | candidate ineligible |
| closure capture | candidate ineligible | candidate ineligible |
| mutable/reassigned local | candidate ineligible | candidate ineligible |
| `.class` / reflection / type descriptor query | materialize at use | candidate ineligible |
| `===`, `==`, `hash`, comparison/send | materialize at use in P2 | candidate ineligible |
| exact resolved pattern match, payload only | materialize in P2 unless future data patterns require otherwise | may stay virtual |
| root pattern whole-value binding | materialize/fallback | candidate ineligible |
| dynamic pack expansion / unknown opaque consumer | materialize/fallback | candidate ineligible |
| `NativeOption` | existing representation | existing representation |

For `data`, P2 initially allows at most **one** opaque whole-value use of a virtual local. Zero whole uses is full scalar replacement. One whole use is allocation sinking/rematerialization at that site. More than one whole use falls back to canonical P1 materialization; cached lazy materialization is deliberately out of scope.

---

# 4. Core optimizer data model

The following compiler-only model is the target. Mechanical naming may follow existing module conventions, but its information boundaries are required.

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProductOptimizationMode {
    Disabled,
    Enabled,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ProductBindingSite {
    pub range: SourceRange,
    pub name: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VirtualProductKind {
    Data {
        constructor_site: LoweringSite,
    },
    Variant {
        variant: VariantId,
        constructor_site: LoweringSite,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProductUseKind {
    DataProjection {
        range: SourceRange,
        component_path: Box<[u32]>,
    },
    ExactVariantMatch {
        range: SourceRange,
    },
    WholeValue {
        range: SourceRange,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProductUseSummary {
    pub projections: usize,
    pub exact_variant_matches: usize,
    pub whole_values: usize,
    pub captured: bool,
    pub reassigned: bool,
    pub opaque_expansion: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProductOptimizationDecision {
    Virtualize(VirtualBindingPlan),
    Materialize(MaterializationReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MaterializationReason {
    OptimizationDisabled,
    NoResolvedConstruction,
    NotSimpleLocalBinding,
    MutableBinding,
    CapturedBinding,
    ReassignedBinding,
    ModuleOrGlobalBinding,
    DynamicOrUnprovenConstruction,
    ExpandedArguments,
    NullaryAlreadyImmediate,
    NoProfitableVirtualUse,
    MultipleWholeValueUses,
    EnumWholeValueObserved,
    NativeOptionAlreadySpecialized,
    LeafBudgetExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProductFunctionPlan {
    pub bindings: BTreeMap<ProductBindingSite, ProductOptimizationDecision>,
    pub ephemeral_projections: BTreeMap<LoweringSite, VirtualShapePlan>,
}
```

## 4.1 Virtual shape

Only nested `data` products flatten recursively in P2. Enum payload values are scalar leaves even if their physical payload is itself product-shaped, because repeated reconstruction of a general enum object is not assumed identity-neutral.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VirtualShapePlan {
    pub kind: VirtualProductKind,
    pub components: Box<[VirtualComponentPlan]>,
    pub leaf_count: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VirtualComponentPlan {
    Scalar {
        logical_component: u32,
        leaf_offset: u16,
    },
    NestedData {
        logical_component: u32,
        leaf_offset: u16,
        shape: Box<VirtualShapePlan>,
    },
}
```

A positive-arity data product has at least one leaf. Nullary data is already immediate under P1 and is not virtualized into zero frame slots.

## 4.2 Active code-generation state

```rust
#[derive(Clone, Debug)]
pub(super) struct ActiveVirtualProduct {
    pub head_slot: u16,
    pub leaf_count: u16,
    pub shape: VirtualShapePlan,
    pub materialization_spec: u16,
}
```

`materialization_spec` is the current chunk's P1 executable semantic-pool index for the original constructor recipe. It is retained even when the corresponding `ConstructData` opcode is not emitted at initialization, so a later whole-value use can reconstruct the exact data type with P1 semantics.

`FunctionState` owns:

```rust
pub(super) active_virtual_products: BTreeMap<u16, ActiveVirtualProduct>,
```

keyed by the source binding's head slot.

## 4.3 Slot invariant

For a virtual local with `N` leaves:

```text
slot H       = source-visible local name, logical leaf 0
slot H + 1   = hidden virtual leaf 1
slot H + k   = hidden virtual leaf k
slot H+N-1   = hidden virtual leaf N-1
```

The source binding is still represented in the compiler's ordinary lexical local sequence, so later locals and upvalues keep the existing slot-numbering model. The additional leaves are ordinary compiler-only locals with reserved names. A candidate is accepted only after use analysis proves no nested block captures the source binding.

A source read must never emit `GetLocal(H)` as a whole-product load merely because `H` is a normal local index; the expression emitter first checks `active_virtual_products`. Direct component projections select the corresponding leaf. Whole-data uses rematerialize.

---

# 5. Profitability floor

P2 is correctness-first but must not replace one small allocation with pathological frame growth.

Initial constant:

```rust
pub(crate) const MAX_VIRTUAL_PRODUCT_LEAVES: usize = 8;
```

This is a conservative P2 engineering threshold, not language semantics.

A local data candidate is accepted only when:

```text
immutable simple local
AND positive-arity exact data construction
AND no capture
AND no reassignment
AND no dynamic/expanded constructor argument pack
AND recursively flattened leaf_count <= 8
AND at least one direct/nested projection
AND whole_value_use_count <= 1
```

An exact general enum candidate is accepted only when:

```text
immutable simple local
AND positive-payload exact general variant construction
AND no capture/reassignment
AND no dynamic/expanded constructor argument pack
AND payload leaf_count <= 8
AND all uses are exact resolved matches/payload projections
AND zero whole-value uses
AND representation != NativeOption
```

C5 measures the threshold. Change `8` only if the measurement demonstrates a better threshold and record the before/after evidence in the implementation state. Do not silently tune it while fixing correctness failures.

---

# 6. Sources of truth

| Concern | Authority | P2 consumer | Forbidden competing authority |
|---|---|---|---|
| data declaration/component identity | P1 semantic products / `DataComponentId` | product planner/emitter | component string name |
| exact data construction | P1 `DataConstructionLoweringSpec` | virtual shape/materialization | parsing call text |
| argument → logical component mapping | P1 construction spec | constructor-to-slots emitter | source argument index alone |
| exact variant identity | existing `VariantId` lowering | virtual enum candidate | selector text / discriminant guess |
| resolved match candidates | `MatchLoweringSpec` / `ExecutablePattern` | virtual exact-variant match emitter | backend pattern resolution |
| local lexical use | AST lexical traversal | capture/escape/use proof | semantic type inference |
| active virtual storage | `FunctionState.active_virtual_products` | expression emitter | heap `ProductStorage` |
| materialized representation | P1 `ProductLayout` / `ProductStorage` | `ConstructData` / enum construct | virtual leaf layout |
| runtime method semantics | existing dispatch | materialized use | optimizer shortcut |
| optimization enable/disable | compiler option | tests + production default | source code / runtime reflection |

---

# 7. Tempting wrong implementations — reject them

- Do not create a second type inference pass in `phalcom-core`.
- Do not search a declaration/component by source name if P1 lowering did not resolve it.
- Do not build a whole-program escape analyzer.
- Do not add a new generic SSA IR solely for product optimization.
- Do not rewrite parameters/returns to multiple values.
- Do not change the VM stack from `Value` to a tagged union of native scalar/register forms.
- Do not store an unboxed `i64` directly in a frame slot that the GC expects to be a `Value`.
- Do not remove evaluation of constructor arguments that are not projected.
- Do not reorder labeled arguments into declaration order **before evaluating them**.
- Do not rematerialize enum cases merely because their payload bits are identical.
- Do not virtualize a captured local and then capture only its first component.
- Do not use a fake placeholder `Value` as a virtual product and let ordinary `GetLocal` read it.
- Do not make optimizer eligibility depend on `ProductLayoutId`; semantic exact construction/use evidence is the proof.
- Do not flatten nested general enum objects inside data.
- Do not optimize `NativeOption` through the general enum path.
- Do not turn a failed proof into `Dynamic`.
- Do not persist product optimization plans in the semantic DB without architectural evidence that they are a semantic query product.
- Do not add optimizer-visible reflection or debug APIs to the language surface.
- Do not change `Chunk::fuse_superinstructions()` into a relayout/CFG optimizer as a side task.
- Do not close unrelated deferred match jump-table/decision-DAG work in this plan.
- Do not claim allocation sinking is allocation elimination when a whole-value use still materializes once.
- Do not use timing benchmarks as the only proof. Deterministic bytecode/allocation assertions remain the hard gate.

---

# 8. Drift protocol

Before each checkpoint:

- [ ] verify checkout `HEAD`;
- [ ] verify P1 state remains COMPLETE;
- [ ] read the previous checkpoint's state entries;
- [ ] confirm Primary working-set symbols still own the responsibilities described here;
- [ ] if P1 final names differ, update the state-file mechanical-name map;
- [ ] search for new exhaustive callers before editing public enums;
- [ ] re-run the previous checkpoint's narrow regression set;
- [ ] adapt mechanics only; do not alter the optimizer laws above.

Escalate rather than silently redesign if repository drift invalidates one of these assumptions:

- P1 no longer has one canonical materialization opcode/recipe;
- product component identity is no longer available in lowering;
- local index no longer equals runtime slot;
- `ReserveScratchLocal` semantics change;
- general enum identity is explicitly re-ruled as value/identity-free;
- the compiler acquires a new canonical typed IR that supersedes direct AST emission before P2 begins.

---

# 9. Checkpoint map

| Checkpoint | Tasks | Contract proved | Hard evidence |
|---|---:|---|---|
| C0 | 1–3 | optimizer has a disabled canonical path and can classify product uses without changing bytecode | planner unit tests; enabled-with-no-transform == disabled disassembly/execution |
| C1 | 4–7 | virtual products can occupy safe contiguous `Value` slots and can project/rematerialize without violating stack/capture/suspension invariants | compiler slot tests; scratch relocation/capture controls; VM suspension/GC hostile test |
| C2 | 8–11 | positive-arity data constructor allocation is eliminated for projection-only local/ephemeral values and sunk for one opaque use | bytecode absence + heap allocation counts + differential side-effect/error tests |
| C3 | 12–14 | nested data products flatten recursively within a bounded cost model and fall back conservatively | nested allocation tests, nested projection, bailout matrix |
| C4 | 15–18 | exact general enum payloads can stay virtual only across resolved match/payload uses without changing case/GADT semantics | ADT/GADT/match differential tests; no `ConstructVariant` for eligible case; identity-hostile fallbacks |
| C5 | 19–22 | optimizer composes with compiler transformations/incrementality and provides measured benefit with complete delivery evidence | inliner/fusion tests; Criterion product benches; full affected/workspace gates; negative searches |

---

# Checkpoint C0 — P1 takeover, optimizer mode, and proof-plan construction

Tasks:
- Task 1 — Reconcile P1 final interfaces and establish canonical baseline fixtures.
- Task 2 — Add an optimizer enable/disable seam with zero semantic effect.
- Task 3 — Implement lexical product-use planning and conservative eligibility classification.

Why this checkpoint exists:

Before emitting one optimized instruction, P2 needs a reproducible canonical control and a proof object explaining why each candidate is or is not virtualized. This prevents later compiler changes from becoming impossible to differential-test.

Entry conditions:
- P1 state file says COMPLETE.
- all P1 focused data/product/ADT tests pass.
- current checkout is recorded in the implementation state.

Primary working set:
- `phalcom-core/src/compiler/lib/mod.rs`
- `phalcom-core/src/compiler/lib/state.rs`
- new `phalcom-core/src/compiler/lib/product_opt.rs`
- P1 final `phalcom-core/src/modules/semantic_lowering.rs`
- compiler unit tests / new `phalcom-core/src/compiler/lib/product_opt/tests.rs`
- `docs/implementation/LANG005-data-impl-traits/C1-optimized-immutable-products/implementation-state.md`

Secondary:
- `phalcom-ast/src/ast.rs` — inspect expression/binding forms only.
- `phalcom-semantic/src/checker/analysis.rs` — inspect only if P1 already exposes a reusable binding fact; P2 must not create new type analysis here.

Out of scope:
- virtual frame slots;
- allocation elimination;
- enum match changes;
- runtime changes.

Required evidence:
- planner classifies simple immutable data candidates, captures, mutation, opaque uses, expansions, globals, and nested blocks correctly;
- production compilation defaults to `Enabled`;
- an internal test path can compile with `Disabled`;
- before transformations are activated, Enabled and Disabled produce byte-for-byte equivalent opcode sequences for representative P1 data/enum fixtures.

Do not proceed to C1 if:
- the disabled path cannot reproduce canonical P1 bytecode;
- P1 lowering does not expose exact construction/component identities;
- use analysis relies on type guessing.

Suggested commit grouping:
- `refactor(compiler): add product optimization mode and planning seam`
- `test(compiler): classify product escapes without changing bytecode`

## Task 1 — Reconcile P1 final interfaces and establish canonical baseline fixtures

**Purpose:** Convert P1's completed state into an explicit P2 interface map and freeze pre-optimization behavior.

**Risk:**
- semantic: HIGH if skipped;
- implementation: LOW.

**Owned files/symbols:**
- LANG005 C1 implementation state.
- P1 data/product compiler/runtime fixtures.
- no production behavior change.

**Inspect before editing:**
- P1 state `Established invariants`, `Decisions`, `Evidence ledger`.
- final data construction/component lowering specs.
- final chunk executable semantic-pool representation.
- final general enum product-storage lowering.
- final bytecode names for data construct/project.

**Dependencies:** P1 COMPLETE.

**Source of truth:** P1 state + committed code, not the pre-P1 names printed in this plan.

**Implementation boundary:**

Add a P2 takeover section to the state file:

```markdown
## C1.P2 takeover interface map

| P2 concept | Final P1 symbol/path |
|---|---|
| data construction spec | Task 1 records the exact committed P1 symbol |
| data component projection spec | Task 1 records the exact committed P1 symbol |
| data materialization bytecode | Task 1 records the exact committed P1 symbol |
| data projection bytecode | Task 1 records the exact committed P1 symbol |
| product layout recipe | Task 1 records the exact committed P1 symbol |
| exact runtime data descriptor identity | Task 1 records the exact committed P1 symbol |
| general enum product payload spec | Task 1 records the exact committed P1 symbol |
```

The literal right-hand cells are filled with the actual P1 symbols during implementation. This is state recording, not a design choice.

**Edit operations:**
1. [ ] Verify every P1 C0–C5 checkpoint is COMPLETE.
2. [ ] Run P1 focused data/product/ADT suites and record PASS before P2 edits.
3. [ ] Fill the takeover interface map with exact committed symbols.
4. [ ] Create canonical fixture sources covering:
   - `Point<Int>` local projection;
   - labeled data constructor with side-effecting arguments;
   - one opaque data use;
   - captured data local;
   - exact general enum local matched for payload;
   - same enum local used by `===` or method send;
   - NativeOption.
5. [ ] Save pre-P2 disassembly for those fixtures under the existing compiler test harness, not as prose snapshots in docs.
6. [ ] Record current repository revision in the state file.

**Testing classification:** baseline evidence only; no optimizer code yet.

## Task 2 — Add `ProductOptimizationMode` and canonical disabled compilation

**Purpose:** Guarantee every optimization can be compared against P1 execution.

**Risk:**
- semantic: LOW;
- implementation fanout: local compiler constructor paths.

**Owned files/symbols:**
- new `compiler/lib/product_opt.rs::ProductOptimizationMode`.
- `Compiler` field.
- `Compiler::new_with_bindings`.
- narrow test constructor/helper.

**Inspect before editing:**
- all `Compiler` constructors.
- REPL/file compilation entry points.
- bootstrap compiler entry.
- any tests constructing `Compiler` directly.

**Dependencies:** Task 1.

**Source of truth:** production always uses Enabled; only internal/test entry can request Disabled.

**Target interface:**

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProductOptimizationMode {
    Disabled,
    Enabled,
}
```

`Compiler` gains:

```rust
product_optimization_mode: ProductOptimizationMode,
```

Existing production constructor behavior:

`Compiler::new_with_bindings` keeps its existing argument list and internally selects
`ProductOptimizationMode::Enabled`; the test-only/internal companion constructor accepts the
same existing arguments plus an explicit mode.

Add one crate-internal constructor/helper for tests that takes the mode explicitly. Do not expose a language/runtime setting.

**Edit operations:**
1. [ ] Add `mod product_opt`.
2. [ ] Add the mode enum.
3. [ ] Add the compiler field.
4. [ ] Keep every production call site behaviorally unchanged and default Enabled.
5. [ ] Add one internal test helper allowing Disabled.
6. [ ] Verify no runtime object, CLI flag, environment variable, or reflection surface exposes the mode.
7. [ ] Compile representative P1 fixtures with both modes before any transformation is activated and compare chunks.

**Testing classification:** focused compiler unit tests at C0.

## Task 3 — Implement `ProductFunctionPlan` and lexical use classification

**Purpose:** Prove product eligibility before code emission without adding semantic resolution.

**Risk:**
- semantic: HIGH;
- implementation fanout: AST visitor + lowering lookups.

**Owned files/symbols:**
- `compiler/lib/product_opt.rs`.
- P1 `ModuleLoweringSemantics` lookup helpers only if ergonomic accessors are missing.

**Inspect before editing:**
- AST `Statement::Let`/binding representation.
- `Expr::Var`, `GetProperty`, calls, assignment, blocks, match, return.
- existing source-index/boundedness AST visitors for exhaustive traversal conventions.
- P1 lowering-site kinds/accessors for data construction/projection.
- existing `MatchLoweringSpec`.

**Dependencies:** Task 2.

**Source of truth:**
- lexical ownership/use comes from AST;
- semantic meaning of a site comes from P1 lowering.

**Planner rules:**

The planner tracks lexical scopes by source binding, including same-name shadowing. It analyzes an initializer **before** inserting the newly declared name into the scope, matching ordinary lexical self-reference rules.

Only a simple local name binding can become a `ProductBindingSite`.

For each reference to a candidate:
- direct/nested P1-resolved data component chain → `DataProjection`;
- exact enum used as a match scrutinee with no root whole-value binding → `ExactVariantMatch`;
- occurrence inside a nested runtime block → `captured = true`;
- assignment target → `reassigned = true`;
- any other occurrence → `WholeValue`.

Expanded/dynamic constructor argument forms set `opaque_expansion = true`.

**Edit operations:**
1. [ ] Define the plan/use/decision/reason types from §4.
2. [ ] Implement a lexical environment stack mapping visible names to internal candidate IDs.
3. [ ] Walk statements in source order and handle scope shadowing.
4. [ ] Recognize candidate initializers only through exact P1 lowering attachments.
5. [ ] Recognize data projection chains only through P1 component attachments.
6. [ ] Mark a reference from a nested runtime block as capture; do not attempt multi-component upvalue lowering.
7. [ ] Inspect resolved match patterns to reject root whole-value binding for enum candidates.
8. [ ] Reject dynamic pack/argument expansion.
9. [ ] Apply the C1 profitability floor and produce explicit `MaterializationReason`.
10. [ ] Add planner unit tests for every reason variant and same-name shadowing.
11. [ ] Add a test proving the planner does not consult a declaration/component source name to establish identity.
12. [ ] With transformations still inactive, verify no emitted chunk changes.

**Testing classification:** required C0 proof.

Checkpoint C0 completion:
- [ ] takeover map recorded;
- [ ] canonical P1 baseline green;
- [ ] Disabled path works;
- [ ] use planner covers all relevant AST forms conservatively;
- [ ] Enabled currently emits canonical P1 bytecode;
- [ ] state updated;
- [ ] no active incident.

---

# Checkpoint C1 — Virtual slot substrate, projection, and rematerialization primitives

Tasks:
- Task 4 — Add virtual product state to `FunctionState`.
- Task 5 — Reserve contiguous product leaf slots safely.
- Task 6 — Compile exact constructors into leaf slots and emit direct projections/rematerialization.
- Task 7 — Prove scratch/upvalue/GC/fiber safety before enabling user-visible transformations.

Why this checkpoint exists:

The optimization is only sound if virtual aggregate lifetime obeys the existing frame/upvalue model. C1 builds the substrate while transformations remain gated behind planner decisions/tests.

Entry conditions:
- C0 COMPLETE.

Primary working set:
- `compiler/lib/state.rs`
- `compiler/lib/scope.rs`
- `compiler/lib/expr.rs`
- `compiler/lib/product_opt.rs`
- P1 data compiler module
- `bytecode.rs` only for documenting/using existing scratch operations; no new opcode is expected
- `vm/dispatch.rs` tests around scratch relocation, not semantic changes

Out of scope:
- enabling data local optimization broadly;
- enum match optimization;
- new VM register representation.

Required evidence:
- leaf slots remain ordinary valid `Value`s and GC roots;
- virtual binding region follows normal local slot numbering;
- nested blocks containing unrelated captures survive reserve/release operations;
- fiber suspension with virtual/scratch values retains reachable heap children;
- rematerialization uses P1 construction recipe exactly.

## Task 4 — Add virtual-product compiler state

**Purpose:** Let expression lowering distinguish a logical product binding from an ordinary one-value local.

**Risk:**
- semantic: HIGH;
- implementation fanout: local/scope bookkeeping.

**Owned files/symbols:**
- `compiler/lib/state.rs::FunctionState`.
- `compiler/lib/product_opt.rs::ActiveVirtualProduct`.
- scope teardown.

**Inspect before editing:**
- `FunctionState::new`.
- `end_scope`.
- local declaration/lookup.
- function/block compiler state push/pop.
- sacred inliner `compile_inline_block_body`.

**Dependencies:** C0.

**Source of truth:** planner decision activates virtual state; ordinary local table remains runtime-slot authority.

**Target state:**

```rust
pub(super) struct ActiveVirtualProduct {
    pub head_slot: u16,
    pub leaf_count: u16,
    pub shape: VirtualShapePlan,
    pub materialization_spec: u16,
}

pub(crate) struct FunctionState {
    // retain every currently committed FunctionState field unchanged
    pub(super) active_virtual_products: BTreeMap<u16, ActiveVirtualProduct>,
}
```

Add compiler helpers:

```rust
fn active_virtual_product(&self, head_slot: usize) -> Option<&ActiveVirtualProduct>;
fn remove_virtual_products_outside_live_locals(&mut self);
```

At scope end, prune any virtual binding whose entire slot range is no longer represented by `num_locals`.

**Must not:**
- change `Local` index semantics;
- alter ordinary upvalue descriptors;
- attach runtime product metadata to `CallFrame`.

**Edit operations:**
1. [ ] Add virtual state to `FunctionState`.
2. [ ] Initialize it empty for every function/block.
3. [ ] Add lookup/prune helpers.
4. [ ] Call prune after lexical local metadata is removed.
5. [ ] Ensure inline block compilation in the same `FunctionState` receives its own planner scope, not a globally range-keyed mutable plan.
6. [ ] Add assertions that every active virtual product range is within live `num_locals`.
7. [ ] Add scope-shadowing tests.

**Testing classification:** compiler invariant tests at C1.

## Task 5 — Reserve persistent and ephemeral virtual leaf regions

**Purpose:** Reuse existing safe scratch/local mechanics instead of inventing stack insertion.

**Risk:**
- semantic: HIGH because slot movement intersects captures;
- implementation fanout: compiler-local.

**Owned files/symbols:**
- `compiler/lib/product_opt.rs`.
- `compiler/lib/scope.rs`.
- existing `reserve_pack_scratch` / `release_pack_scratch_from`.

**Inspect before editing:**
- `compiler/lib/expr.rs::reserve_pack_scratch`, `release_pack_scratch_from`, and `emit_release_scratch_range`.
- `compiler/lib/scope.rs::add_local`, local resolution, and lexical scope teardown.
- VM `ReserveScratchLocal` / `ReleaseScratchLocal` handlers and their open-upvalue relocation.
- loop-local reservation helpers so persistent virtual regions follow existing frame-slot conventions.

**Dependencies:** Task 4.

**Source of truth:** ordinary local slot numbering.

**Persistent local rule:**

For an accepted source binding with `N > 0` leaves:
1. reserve the first slot under the actual source symbol;
2. reserve `N-1` compiler-hidden slots immediately after it;
3. keep all `N` slots live for the source binding's lexical lifetime;
4. record `head_slot` in active virtual map.

Use fresh internal names for non-head leaves, e.g. `$product#<counter>`.

**Ephemeral expression rule:**

For constructor→projection:
1. reserve `N` topmost hidden scratch slots;
2. evaluate/store all source arguments;
3. load/materialize the required projection result;
4. release the `N` scratch slots with the existing release mechanism while preserving the result window.

Do not use the old un-reclaimed destructuring scratch discipline for ephemeral products.

**Edit operations:**
1. [ ] Add `reserve_virtual_binding_slots(source_symbol, leaf_count, range)`.
2. [ ] Add `reserve_ephemeral_virtual_slots(leaf_count, range)`.
3. [ ] Reuse `ReserveScratchLocal` for runtime slot insertion.
4. [ ] For persistent slots, retain local metadata until normal lexical teardown.
5. [ ] For ephemeral slots, call existing release helper in reverse order.
6. [ ] Add checked `usize/u16` conversion and compiler error on slot overflow.
7. [ ] Assert contiguous suffix requirements before ephemeral release.
8. [ ] Test nested ordinary captured locals around an ephemeral virtual region.
9. [ ] Test repeated loop execution does not grow the stack window.

**Testing classification:** high-risk C1 slot tests.

## Task 6 — Compile virtual constructors, projections, and data rematerialization

**Purpose:** Implement reusable codegen primitives before enabling planner-selected transformations.

**Risk:**
- semantic: HIGH;
- implementation fanout: data expression paths.

**Owned files/symbols:**
- `compiler/lib/product_opt.rs`.
- P1 `compiler/lib/data_decl.rs` or final data compiler module.
- `compiler/lib/expr.rs`.

**Inspect before editing:**
- P1 canonical data construction emitter.
- P1 construction spec's argument-to-component mapping.
- P1 component projection emitter.
- pack-item compile order.

**Dependencies:** Task 5.

**Source of truth:** P1 construction/projection lowering specs.

**Required helpers:**

```rust
fn compile_virtual_data_constructor_into(
    &mut self,
    expr: &Expr,
    shape: &VirtualShapePlan,
    head_slot: u16,
    range: SourceRange,
) -> Result<(), CompilerError>;

fn emit_virtual_projection(
    &mut self,
    value: &ActiveVirtualProduct,
    component_path: &[u32],
    range: SourceRange,
) -> Result<(), CompilerError>;

fn emit_virtual_data_materialization(
    &mut self,
    value: &ActiveVirtualProduct,
    range: SourceRange,
) -> Result<(), CompilerError>;
```

The exact receiver types may follow P1 code, but these responsibilities must remain separate.

**Constructor evaluation law:**

Reserve slots first, then walk source arguments in canonical **source evaluation order**. For each argument:
- find its logical component using P1's mapping;
- compile the argument exactly once;
- store into the corresponding leaf/subshape;
- pop the temporary expression result if `SetLocal` leaves it on the stack.

A labeled call does not get evaluated in declaration order merely because storage is logical-component ordered.

**Materialization law:**

To reuse the original P1 construction recipe, emit arguments in the ordering expected by that recipe. If the recipe represents source parameter order, load the appropriate logical component for each recipe argument. A nested virtual data component is recursively materialized at this point.

**Edit operations:**
1. [ ] Add checked logical-component → leaf range resolution.
2. [ ] Implement scalar argument store.
3. [ ] Implement recursively nested-data store hook but keep recursive shape activation disabled until C3.
4. [ ] Implement direct leaf projection.
5. [ ] Implement nested path projection API.
6. [ ] Implement P1 recipe-driven data materialization.
7. [ ] Add unit tests with reversed labeled source order proving effects execute in source order while storage follows logical component identity.
8. [ ] Add a throwing argument test proving all preceding/following evaluation semantics match canonical lowering as applicable.
9. [ ] Add bytecode stack-height assertions at statement/expression boundary.

**Testing classification:** helper behavior at C1; optimization activation later.

## Task 7 — Prove capture, scratch relocation, GC, and suspension safety

**Purpose:** Close the repository-specific risk around extra compiler slots before using them as an optimization.

**Risk:**
- semantic: HIGH;
- implementation fanout: tests, only narrow fixes if evidence fails.

**Owned files/tests:**
- compiler product optimizer tests.
- VM scratch tests near `ReserveScratchLocal` / `ReleaseScratchLocal`.
- core language data runtime fixtures from P1.

**Inspect before editing:**
- runtime scratch handlers in `phalcom-core/src/vm/dispatch.rs`.
- root enumeration / parked-Fiber stack tracing.
- current open-upvalue relocation tests.
- P1 DataObject/enum GC tests so the virtual-slot cases compare against canonical materialization.

**Dependencies:** Tasks 4–6.

**Source of truth:** existing VM scratch/upvalue relocation semantics.

**Required hostile scenarios:**
- ordinary captured local below an ephemeral virtual slot region;
- ordinary captured local above an earlier scratch region after release;
- nested ephemeral product regions;
- a constructor argument that invokes arbitrary user code while slots are reserved;
- fiber suspension/yield between virtual local construction and component read;
- GC while the only reference to a child object is a virtual component slot;
- early throw during argument construction;
- non-local return from a nested block during argument evaluation, if legal in the fixture;
- scope shadowing after a virtual binding ends.

**Edit operations:**
1. [ ] Add compiler/VM tests for every hostile scenario.
2. [ ] Verify scratch release relocates open upvalues correctly in the scenarios already supported by the runtime.
3. [ ] Verify persistent component slots require no special GC trace path because frame stack values are already roots.
4. [ ] Verify parked fibers retain component values through existing Fiber stack tracing.
5. [ ] If L06 audit behavior fails, classify/fix the existing scratch mechanism separately and record it as a dependency incident before continuing.
6. [ ] Do not weaken product optimization proof rules to hide a scratch bug.

**Testing classification:** C1 hard gate.

Checkpoint C1 completion:
- [ ] state model implemented;
- [ ] persistent/ephemeral slot regions correct;
- [ ] projection/materialization helpers correct;
- [ ] hostile scratch/capture/suspension/GC tests green;
- [ ] no source optimization enabled beyond test-only helper exercise;
- [ ] state updated;
- [ ] no active incident.

---

# Checkpoint C2 — Positive-arity data scalar replacement and allocation sinking

Tasks:
- Task 8 — Eliminate ephemeral constructor→projection allocation.
- Task 9 — Scalar-replace eligible immutable local data bindings.
- Task 10 — Sink one required whole-value materialization to its actual use.
- Task 11 — Differentially prove evaluation, errors, control flow, and allocation behavior.

Why this checkpoint exists:

C2 is the first user-visible performance change. It proves PDR-0035's representation freedom is usable in the actual stack VM without changing the method ABI.

Entry conditions:
- C1 COMPLETE.

Primary working set:
- `compiler/lib/product_opt.rs`
- `compiler/lib/expr.rs`
- `compiler/lib/mod.rs` statement binding path
- P1 data compiler module
- data integration tests / disassembly tests

Semantic contract:
- zero aggregate allocation for projection-only virtual data;
- at most one allocation at an actual opaque whole-value use;
- no change in source side effects, diagnostics, or result.

## Task 8 — Optimize ephemeral `data-construction -> component-path`

**Purpose:** Remove the simplest allocation without changing local binding machinery.

**Risk:**
- semantic: MEDIUM;
- implementation fanout: expression compiler.

**Owned symbols:**
- GetProperty/data projection expression path.
- product optimizer ephemeral plan.

**Inspect before editing:**
- P1 component-projection branch in `compiler/lib/expr.rs`.
- P1 data construction emission helper.
- `release_pack_scratch_from` result-window behavior.
- expression stack-effect conventions for `GetProperty`.

**Dependencies:** C1.

**Source of truth:** P1 construction and component lowering at exact source ranges.

**Eligible examples:**

```phalcom
Point(x: side1(), y: side2()).x
Response(makeBody(), status: makeStatus(), headers: makeHeaders()).status
Outer(inner: Point(x: 1, y: 2)).inner // C2 materializes nested result; C3 removes nested allocation
```

All constructor arguments still execute.

**Implementation:**
- intercept P1-resolved component projection before compiling the receiver normally;
- if receiver is an eligible exact data construction and leaf budget passes, reserve ephemeral slots, compile constructor into slots, emit projection, release slots;
- do not emit P1 `ConstructData` for the eliminated receiver.

**Edit operations:**
1. [ ] Add planner entry for exact constructor→projection.
2. [ ] Intercept only P1-resolved data component projection.
3. [ ] Compile/store every constructor argument.
4. [ ] Load requested component.
5. [ ] Release scratch region preserving result.
6. [ ] Assert emitted chunk contains no receiver `ConstructData`.
7. [ ] Add side-effect-order test where unselected components mutate a log.
8. [ ] Add throwing unselected-component test proving it is still evaluated.
9. [ ] Add optimizer-disabled differential execution test.

**Testing classification:** focused C2 compiler/runtime regression; must include Disabled/Enabled differential execution.

**Hard evidence:**
- disassembly/opcode assertion: no aggregate construct for the receiver;
- heap live/allocation counter: zero DataObject allocation attributable to expression;
- same output/error order as Disabled.

## Task 9 — Scalar-replace eligible immutable local data bindings

**Purpose:** Keep a positive-arity data local as component slots across multiple projections.

**Risk:**
- semantic: HIGH;
- implementation fanout: statement binding + variable/property reads.

**Eligible shape:**

```phalcom
const p = Point(x: makeX(), y: makeY())
consumeScalar(p.x)
consumeScalar(p.y)
```

Use the language's actual immutable-binding spelling/kind if it differs at implementation time; eligibility is semantic immutability, not the token text.

**Owned files/symbols:**
- simple local binding compilation.
- bare variable/property compile paths.
- active virtual map.

**Inspect before editing:**
- existing `Statement::Let`/simple-name binding compiler branch.
- local declaration mutability checks.
- variable/bare-name compiler path.
- lexical scope cleanup and `max_slots` accounting.
- P1 `ConstructData` and `GetDataComponent` emission sites.

**Dependencies:** Task 8.

**Source of truth:** `ProductFunctionPlan` acceptance.

**Implementation boundary:**
- only simple local-name pattern;
- not module/global scope;
- no capture/reassignment;
- positive arity;
- leaf count within limit;
- at least one projection;
- whole use count <= 1.

**Edit operations:**
1. [ ] Before generic simple-local binding lowering, query current product plan.
2. [ ] For `Virtualize`, reserve persistent leaf slots with source name on head.
3. [ ] Register P1 materialization spec in executable semantic pool even though construct opcode is omitted.
4. [ ] Compile initializer into those leaves.
5. [ ] Insert `ActiveVirtualProduct`.
6. [ ] Intercept data component reads whose receiver resolves to the virtual head slot.
7. [ ] Emit direct `GetLocal` for scalar component leaf.
8. [ ] Prevent generic whole-variable emission from blindly reading only the head component.
9. [ ] Let ordinary scope teardown prune active state and reuse local slots normally.
10. [ ] Add disassembly tests: projections contain `GetLocal`, no `ConstructData`, no `GetDataComponent`.
11. [ ] Add repeated-loop fixture proving no per-iteration aggregate allocation and no stack growth.
12. [ ] Add shadowing fixture with an inner ordinary `p`.

**Testing classification:** C2 core optimization tests.

## Task 10 — Sink a single required data materialization

**Purpose:** Preserve optimization when a data local is mostly projected but crosses one opaque boundary.

**Risk:**
- semantic: HIGH;
- implementation fanout: variable expression emission.

**Example:**

```phalcom
const p = Point(x: 1, y: 2)
System.print(p.x)
sink(p) // only here does the aggregate need a runtime Value
```

**Owned files/symbols:**
- `compiler/lib/product_opt.rs` whole-use planner metadata.
- `compiler/lib/expr.rs` bare local/value emission.
- P1 data materialization helper/executable semantic-pool registration.

**Inspect before editing:**
- every current bare-local emission path (`GetLocal`, fused `InvokeLocal`, return, packs).
- call/send argument compilation order.
- return/non-local-return expression compilation.
- P1 data `===`/`==`/hash behavior to ensure rematerialization is representation-independent.

**Dependencies:** Task 9.

**Source of truth:** planner records exactly one `WholeValue` range.

**Implementation:**
- when ordinary variable compilation resolves the virtual head at the planned whole-use range, call `emit_virtual_data_materialization`;
- do not store/cache the materialized object back into the virtual binding;
- after the use, components remain authoritative;
- because PDR-0035 removes allocation identity, rematerialization does not change data semantics.

One whole-value use is allowed. More than one falls back for P2.

**Opaque boundaries include:**
- send receiver/argument;
- return;
- global/field/index/list/map store;
- `.class`;
- reflection;
- equality/hash/comparison;
- pack expansion.

**Edit operations:**
1. [ ] Add whole-value use-site verification against the plan.
2. [ ] Emit recursive P1 materialization at that exact use.
3. [ ] Verify direct projections before/after the whole use still read components.
4. [ ] Add branch fixture where the only whole use is in one branch; prove allocation occurs only when branch executes.
5. [ ] Add `.class`, return, ordinary call argument, and equality differential tests.
6. [ ] Add >1 whole-use fixture proving planner selects canonical baseline.

**Testing classification:** C2.

## Task 11 — Differentially prove data optimization semantics

**Purpose:** Make Enabled vs Disabled execution the primary semantic oracle.

**Risk:**
- semantic: HIGH;
- implementation fanout: tests.

**Owned files/tests:**
- `compiler/lib/product_opt/tests.rs`.
- focused P1/P2 data fixtures under the current `phalcom-core/tests/core/language/data/` location.
- crate-internal compilation/execution test helper that selects `ProductOptimizationMode`.

**Inspect before editing:**
- current test utilities for compiling source to a `Chunk` and executing it.
- heap allocation/live-count instrumentation introduced by P1.
- diagnostics comparison helpers for runtime and compile errors.

**Source of truth:** Disabled P1 execution is the semantic oracle; PDR-0035 permits backing-allocation differences only.

**Dependencies:** Tasks 8–10.

**Required differential matrix:**

```text
constructor argument evaluation order
unselected argument side effects
throw in first/middle/last component
loop construction + projections
branch-only whole materialization
component containing mutable heap object
component containing closure
component containing Fiber/Future where legal
.class
===
==
hash
return
ordinary method argument
labeled constructor in source order different from declaration order
generic Point<Int>
phantom generic Id<User>
```

For each suitable fixture:
1. compile/run with Disabled;
2. compile/run with Enabled;
3. compare result/output/error kind/source span;
4. separately assert the expected allocation/opcode difference.

Do not compare heap handles; PDR-0035 intentionally permits them to differ.

**Edit operations:**
1. [ ] Add a crate-internal differential harness.
2. [ ] Add the full matrix.
3. [ ] Assert no changed source diagnostic set.
4. [ ] Assert projection-only fixtures allocate zero DataObjects.
5. [ ] Assert one-whole-use fixture allocates at most once and only on executed path.
6. [ ] Re-run P1 data suite with production Enabled mode.

**Testing classification:** C2 hard differential gate; a semantic mismatch is an INCIDENT, not a benchmark issue.

Checkpoint C2 completion:
- [ ] ephemeral allocation elimination works;
- [ ] local scalar replacement works;
- [ ] one-use allocation sinking works;
- [ ] differential matrix green;
- [ ] P1 data suite green;
- [ ] state updated;
- [ ] no active incident.

Suggested commit grouping:
- `feat(compiler): scalar-replace non-escaping data products`
- `feat(compiler): sink data materialization to opaque uses`
- `test(data): prove virtual and materialized execution equivalent`

---

# Checkpoint C3 — Recursive nested data virtualization and bounded profitability

Tasks:
- Task 12 — Build recursive data-only virtual shapes.
- Task 13 — Optimize nested projection chains and recursive rematerialization.
- Task 14 — Enforce cost/bailout policy and prove conservative fallback.

Why this checkpoint exists:

Single-level scalar replacement establishes correctness. C3 obtains the major product-layout win: nested immutable value products can disappear as aggregates as long as they remain data values. The cost model prevents stack-slot explosion.

Entry conditions:
- C2 COMPLETE.

Semantic contract:
- nested `data` may flatten recursively because every layer is identity-free;
- nested enum/class/anonymous values remain scalar `Value` leaves in P2;
- materialization reconstructs exact nested data values only where required.

## Task 12 — Build recursive `VirtualShapePlan` for nested data

**Purpose:** Represent nested data constructor arguments as leaf ranges rather than boxed intermediate data.

**Risk:**
- semantic: HIGH;
- implementation fanout: planner.

**Owned files/symbols:**
- `product_opt.rs::VirtualShapePlan`.
- candidate shape builder.

**Inspect before editing:**
- final P1 data construction lowering shape for nested constructor expressions.
- P1 exact applied-type materialization recipe.
- C2 virtual constructor argument mapping.
- current recursion/depth limits so shape construction cannot create unbounded compiler recursion.

**Dependencies:** C2.

**Source of truth:** nested exact P1 data construction attachments.

**Example:**

```phalcom
data Point(x: Int, y: Int)
data Rect(topLeft: Point, bottomRight: Point)

const r = Rect(
  topLeft: Point(x: 1, y: 2),
  bottomRight: Point(x: 3, y: 4),
)
```

Target leaves:

```text
r[0] = topLeft.x
r[1] = topLeft.y
r[2] = bottomRight.x
r[3] = bottomRight.y
```

A component expression is recursively flattened only when it is itself an exact positive-arity `data` construction accepted by the same rules.

**Never recursively flatten in P2:**
- general enum constructor;
- class instance;
- structural tuple/record;
- dynamic/unproven data value;
- an existing data variable/alias rather than a nested exact constructor expression.

**Edit operations:**
1. [ ] Implement recursive shape construction.
2. [ ] Accumulate checked `leaf_count`.
3. [ ] Stop before exceeding `MAX_VIRTUAL_PRODUCT_LEAVES`.
4. [ ] Preserve source argument evaluation tree/order.
5. [ ] Treat nullary data child as a scalar immediate expression when needed.
6. [ ] Treat enum/class/tuple/record child as scalar.
7. [ ] Add shape unit tests for 2-level/3-level nesting and mixed scalar/object leaves.
8. [ ] Add phantom generic nested data shape test proving semantic materialization recipe retains exact type.

**Testing classification:** C3 planner tests.

## Task 13 — Optimize nested projections and recursive materialization

**Purpose:** Eliminate both outer and nested data allocations and reconstruct only the requested logical value.

**Risk:**
- semantic: HIGH;
- implementation fanout: expression/materialization helpers.

**Owned files/symbols:**
- `compiler/lib/product_opt.rs` path-to-subshape resolution and recursive materialization.
- `compiler/lib/expr.rs` nested GetProperty interception.
- P1 data construction emitter.

**Inspect before editing:**
- AST shape for chained `GetProperty`.
- P1 lowering attachment ranges on each edge of a property chain.
- C2 materialization helper argument ordering.
- heap allocation counter tests for nested P1 DataObjects.

**Dependencies:** Task 12.

**Examples:**

```phalcom
r.topLeft.x        // no Rect, no Point allocation
r.topLeft          // materialize Point only, not Rect
sink(r)            // recursively materialize both Points then Rect
```

**Source of truth:** P1 component path attachments + recursive virtual shape.

**Edit operations:**
1. [ ] Resolve a component path into nested shape/leaf range.
2. [ ] If final projection is scalar, emit leaf load.
3. [ ] If final projection denotes a nested virtual data product as a whole, materialize only that subshape.
4. [ ] For whole outer use, recursively reconstruct child data in the argument order required by each original P1 recipe.
5. [ ] Keep nested construction side effects in original source order.
6. [ ] Add disassembly/allocation assertions for the three examples above.
7. [ ] Add Disabled/Enabled differential tests with nested throwing/side-effecting constructors.
8. [ ] Add GC fixture where nested leaf is the sole reference to an object.

**Testing classification:** C3.

## Task 14 — Enforce leaf-budget and bailout policy

**Purpose:** Make optimizer conservatism explicit and measurable.

**Risk:**
- semantic: MEDIUM;
- performance: HIGH if omitted.

**Owned symbols:**
- `MAX_VIRTUAL_PRODUCT_LEAVES`.
- eligibility classifier.

**Inspect before editing:**
- all eligibility call sites created in C2/C3.
- `max_slots` accounting in `FunctionState`.
- benchmark harness slot/frame-sensitive workloads noted in repository perf logs.

**Source of truth:** one centralized pure eligibility function over planner facts + recursive leaf count; emitters do not independently loosen policy.

**Dependencies:** Tasks 12–13.

**Required bailout matrix:**

| Case | P2 result |
|---|---|
| positive data, 2 leaves, projections only | virtual |
| nested data, total 8 leaves | virtual |
| nested data, total 9 leaves | canonical P1 |
| nullary data | canonical P1 immediate |
| mutable/reassigned local | canonical P1 |
| captured local | canonical P1 |
| module/global binding | canonical P1 |
| destructuring binding | canonical P1 |
| dynamic/unproven constructor | canonical P1 |
| expansion/computed pack constructor | canonical P1 |
| zero projections, one opaque use | canonical P1 |
| >1 opaque whole use | canonical P1 |
| optimizer Disabled | canonical P1 |

**Edit operations:**
1. [ ] Centralize eligibility in one pure function.
2. [ ] Avoid duplicated “should optimize” checks in emitter paths.
3. [ ] Add one unit test per matrix row.
4. [ ] Add integration test proving canonical materialized bytecodes remain present for every bailout.
5. [ ] Add `#[must_use]` or equivalent discipline so a rejected decision cannot be ignored and optimized anyway.
6. [ ] Record current threshold in state.

**Testing classification:** C3 bailout-contract tests plus canonical-bytecode comparison for every rejected row.

Checkpoint C3 completion:
- [ ] nested data scalar replacement works;
- [ ] selective nested materialization works;
- [ ] cost limit enforced;
- [ ] bailout matrix green;
- [ ] state updated;
- [ ] no active incident.

Suggested commit grouping:
- `feat(compiler): flatten nested virtual data products`
- `test(compiler): pin product optimizer bailout policy`

---

# Checkpoint C4 — Exact general-enum virtualization across resolved matches

Tasks:
- Task 15 — Prove exact enum eligibility without assuming value identity.
- Task 16 — Store eligible enum payloads as virtual component slots.
- Task 17 — Compile resolved match patterns directly against a virtual exact variant.
- Task 18 — Prove GADT/or-pattern/identity/NativeOption preservation.

Why this checkpoint exists:

P1 gives general enums the same physical product substrate, but PDR-0035 did **not** make enum allocations identity-free. C4 therefore uses a stricter proof: the case object may disappear only when no execution can observe the whole case value.

Entry conditions:
- C3 COMPLETE.
- existing P1 ADT/GADT suites green.

Primary working set:
- `compiler/lib/product_opt.rs`
- `compiler/lib/mod.rs` simple binding path
- `compiler/lib/match_expr.rs`
- `compiler/lib/patterns.rs`
- P1 enum lowering spec
- core algebraic-data tests
- semantic ADT/GADT tests as regression only

Out of scope:
- variant jump tables;
- broad decision DAG;
- candidate-set optimization;
- changing enum equality/identity;
- NativeOption reimplementation.

## Task 15 — Add strict exact-enum eligibility proof

**Purpose:** Select enum cases only when allocation identity cannot be observed.

**Risk:**
- semantic: VERY HIGH;
- implementation fanout: planner.

**Owned files/symbols:**
- `compiler/lib/product_opt.rs` enum candidate/use classifier.
- P1/TYPE001 enum construction lowering accessors.
- existing `ExecutablePattern`/match lowering specs.

**Inspect before editing:**
- final `RuntimeAdtRepresentation` enum after P1.
- final P1 general variant construction spec and payload field mapping.
- `ExecutablePattern::{Binding,Variant,Or,Wildcard}` semantics.
- source-level match scrutinee/body AST traversal for references to the original binding.

**Dependencies:** C3.

**Source of truth:**
- exact P1/TYPE001 variant constructor lowering;
- resolved match patterns.

**Eligible example:**

```phalcom
const r = Result::Ok(expensive())
match r {
  Result::Ok(x) => use(x)
  Result::Error(e) => useError(e)
}
```

provided `r` has no other whole-value use.

**Ineligible examples:**

```phalcom
const r = Result::Ok(1)
sink(r)

const r = Result::Ok(1)
r === r

const r = Result::Ok(1)
r.someCaseMethod

const r = Result::Ok(1)
const f = { r }

match r {
  whole => sink(whole)
}
```

**Rules:**
- exact known general variant only;
- positive payload only;
- no NativeOption;
- no capture/reassignment;
- zero `WholeValue`;
- every use is a resolved match scrutinee;
- every relevant match pattern must avoid root whole-value binding;
- payload fields remain ordinary `Value` leaves; do not recursively reconstruct enum objects.

**Edit operations:**
1. [ ] Extend planner candidate kind for exact variant.
2. [ ] Inspect P1 enum representation and reject NativeOption.
3. [ ] Classify match scrutinee uses.
4. [ ] Walk `ExecutablePattern` root shape to detect whole-scrutinee binding.
5. [ ] Treat ordinary sends/equality/return/alias as WholeValue.
6. [ ] Add one unit test for every eligible/ineligible example.
7. [ ] Add a proof test that two variants with identical payload layout remain distinct candidates by `VariantId`.

**Testing classification:** C4 proof gate.

## Task 16 — Compile eligible enum constructor into virtual payload slots

**Purpose:** Remove `ConstructVariant` allocation when the exact case object is never observed.

**Risk:**
- semantic: HIGH;
- implementation fanout: local binding path.

**Owned files/symbols:**
- simple local-binding virtual branch from C2.
- `compiler/lib/product_opt.rs::VirtualProductKind::Variant`.
- P1 enum construction emitter/spec.

**Inspect before editing:**
- canonical `ConstructVariant` stack contract.
- P1 ProductStorage field ordering.
- variant-local generic constructor argument mapping.
- current singleton/nullary variant fast path.

**Dependencies:** Task 15.

**Source of truth:** P1 enum constructor lowering field order/mapping.

**Implementation:**
- reserve one ordinary `Value` leaf per logical payload field;
- source binding head slot is payload leaf 0;
- evaluate all constructor arguments in source order;
- store them by resolved logical field mapping;
- register `ActiveVirtualProduct` whose `VirtualProductKind::Variant` stores the exact `variant` and constructor site;
- do not retain a materialization recipe because accepted enum candidates are forbidden from whole-value use.

If a future code path requests whole materialization for an active virtual enum, treat it as an internal optimizer-plan invariant failure; do not silently reconstruct a new enum object.

**Edit operations:**
1. [ ] Add variant virtual shape with scalar payload fields only.
2. [ ] Compile source args into payload slots.
3. [ ] Skip canonical `ConstructVariant`.
4. [ ] Add assertion/error for unexpected whole enum read.
5. [ ] Add heap allocation test proving zero `AdtCase`/general case object for eligible fixture.
6. [ ] Add side-effect/error-order differential tests.

**Testing classification:** C4.

## Task 17 — Compile match against a virtual exact variant

**Purpose:** Reuse semantic match resolution while replacing runtime `IsVariant`/`GetVariantPayload` work with compile-time-known case selection and direct leaf loads.

**Risk:**
- semantic: VERY HIGH;
- implementation fanout: match/pattern lowering.

**Owned files/symbols:**
- `compiler/lib/match_expr.rs`.
- `compiler/lib/patterns.rs`.
- `ExecutablePattern` consumer paths.

**Inspect before editing:**
- `compiler/lib/match_expr.rs` scrutinee staging and arm loop.
- `compiler/lib/patterns.rs` binding staging/rollback and `GetVariantPayload` emission.
- `ExecutableVariantCandidate` and field projection ordering.
- or-pattern failure jump patching.
- current match invariant-failure path.

**Dependencies:** Task 16.

**Source of truth:** existing `MatchLoweringSpec`.

**Required behavior:**

For each arm, in source order:
- if the resolved pattern's exact candidate set cannot contain the virtual `VariantId`, emit the normal arm-failure jump/skip without an `IsVariant` runtime test;
- if it can contain the exact variant, stage each requested payload field directly from the appropriate virtual component slot;
- continue compiling child payload patterns with existing pattern code;
- preserve arm order and all existing pattern binding staging/rollback behavior;
- never re-solve candidate membership by selector/name.

Root wildcard can match without reading the case object.
Root binding makes the candidate ineligible in Task 15.
Or-patterns use their already-resolved alternatives/candidates.

**Edit operations:**
1. [ ] Add a `VirtualMatchScrutinee` adapter carrying exact `VariantId` plus payload slots.
2. [ ] Add a match compiler entry path when scrutinee is an active virtual enum local.
3. [ ] Reuse existing arm ordering/jump scaffolding.
4. [ ] Replace exact `IsVariant` tests with resolved membership check over `ExecutablePattern`.
5. [ ] Replace outer payload `GetVariantPayload` with direct `GetLocal`.
6. [ ] Leave nested child pattern matching unchanged once a payload `Value` is staged.
7. [ ] Preserve binding rollback on failed child patterns/or alternatives.
8. [ ] Add disassembly test proving eligible outer match has no `ConstructVariant`, no outer `IsVariant`, and no outer `GetVariantPayload`.
9. [ ] Add overlapping/or-pattern fixture proving first-match semantics remain unchanged.

**Testing classification:** C4 high-risk.

## Task 18 — Prove enum/GADT/identity/NativeOption preservation

**Purpose:** Demonstrate that C4 is only an allocation/projection optimization.

**Risk:**
- semantic: VERY HIGH;
- implementation fanout: tests.

**Owned files/tests:**
- core algebraic-data execution/behavior/GC/match tests.
- semantic ADT/GADT tests as unchanged oracle.
- product optimizer differential fixtures.

**Inspect before editing:**
- existing GADT exact-case matching fixtures.
- variant-local generic constructor fixtures.
- NativeOption execution/reification tests.
- current enum `===`/ordinary object identity behavior so bailout cases test the actual contract.

**Source of truth:** pre-existing enum semantic products and Disabled P1 execution; P2 may only change allocation/projection for already-proven non-observable cases.

**Dependencies:** Tasks 15–17.

**Required controls:**
- same exact variant matched through GADT-refined arm;
- variant-local generics;
- or-pattern with same payload binding;
- root wildcard;
- root whole-value binding → optimizer must bail out;
- arm body referencing original scrutinee → bail out;
- method send on scrutinee → bail out;
- `===`/`==`/hash of scrutinee → bail out;
- captured scrutinee → bail out;
- two variants with same payload layout → correct exact arm;
- payload containing heap object survives GC;
- payload constructor can suspend/yield;
- NativeOption `Some`/`None` stays on existing special path.

**Edit operations:**
1. [ ] Add Enabled/Disabled differential tests for every control.
2. [ ] Assert bailout fixtures still emit canonical `ConstructVariant`.
3. [ ] Assert eligible fixtures do not allocate general case objects.
4. [ ] Run full core algebraic-data suite.
5. [ ] Run semantic ADT/GADT suite unchanged.
6. [ ] Verify no semantic enum table/code was changed merely to satisfy optimizer tests.
7. [ ] Verify NativeOption branch remains explicit and untouched.

**Testing classification:** C4 release-blocking ADT/GADT differential gate.

Checkpoint C4 completion:
- [ ] strict enum proof works;
- [ ] eligible exact case allocation removed;
- [ ] match direct payload path works;
- [ ] identity-observing cases fall back;
- [ ] ADT/GADT/NativeOption suites green;
- [ ] state updated;
- [ ] no active incident.

Suggested commit grouping:
- `feat(compiler): virtualize non-observable exact enum payloads`
- `feat(match): project virtual exact variants without case allocation`
- `test(adt): prove scalar replacement preserves gadt and case semantics`

---

# Checkpoint C5 — Compiler integration, qualification, performance evidence, and delivery

Tasks:
- Task 19 — Compose product planning with sacred inlining, chunk finalization, and superinstruction fusion.
- Task 20 — Verify incremental/compiler publication boundaries and optimizer invisibility.
- Task 21 — Add product-allocation Criterion microbenchmarks and deterministic qualification.
- Task 22 — Complete state/docs/negative gates and full delivery verification.

Why this checkpoint exists:

The optimizer is not complete because local fixtures pass. It must coexist with duplicated sacred-inliner code, REPL/file compilation, chunk side tables, incremental lowering, and the repository's existing performance methodology.

Entry conditions:
- C4 COMPLETE.

## Task 19 — Compose with inliner and chunk finalization

**Purpose:** Ensure optimizer planning/emission remains a pre-bytecode specialization and does not corrupt later transforms.

**Risk:**
- semantic: HIGH;
- implementation fanout: compiler/inliner/chunk tests.

**Owned files/symbols:**
- `compiler/inliner.rs`.
- `compiler/lib/mod.rs`.
- `compiler/lib/product_opt.rs`.
- `chunk.rs::fuse_superinstructions`.

**Inspect before editing:**
- `compile_inline_block_body`.
- sacred fallback compilation.
- function finalization order.
- `fuse_superinstructions` call site.
- branch target inventory.

**Dependencies:** C4.

**Source of truth:** existing compiler body boundaries and chunk-finalization order; product optimization is complete before the existing peephole fusion begins.

**Rules:**
- product planning occurs for each actual codegen body/scope; a source block compiled once inline and once as fallback receives independent ephemeral planner state;
- no active virtual binding state leaks between the two copies;
- product optimization completes during canonical emission;
- `fuse_superinstructions()` remains the final local peephole pass and sees ordinary GetLocal/Invoke patterns;
- do not compact or relayout bytecode.

**Edit operations:**
1. [ ] Identify every body compilation entry and ensure a corresponding product-plan scope exists.
2. [ ] Add planner scope around `compile_inline_block_body`.
3. [ ] Ensure fallback closure gets a fresh FunctionState/plan.
4. [ ] Verify no range-key collision shares mutable state across duplicated fast/fallback source ranges.
5. [ ] Keep chunk fusion after product emission.
6. [ ] Add nested sacred-if fixture containing a virtual data binding in an inline arm.
7. [ ] Force sacred deopt fallback and compare Enabled/Disabled results.
8. [ ] Add branch target/fusion regression if new emitted GetLocal sequences expose an existing unsafe fusion edge.

**Testing classification:** C5 compiler integration.

## Task 20 — Verify incremental/publication boundaries and optimizer invisibility

**Purpose:** Prove P2 did not create a second semantic/incremental authority.

**Risk:**
- semantic: MEDIUM;
- implementation fanout: tests and state.

**Owned files/tests:**
- product optimizer compile/edit/REPL tests.
- `ModuleLoweringSemantics` consumers.
- existing semantic incremental harness and P1 LSP/source-index tests.

**Inspect before editing:**
- module/cell compiler lifecycle.
- semantic session invalidation and module-lowering rebuild path.
- REPL compiler state reset between cells.
- any artifact cache keyed by chunk/lowering fingerprint.

**Dependencies:** Task 19.

**Source of truth:** AST + P1 `ModuleLoweringSemantics`; plan is transient compiler state.

**Required assertions:**
- no new any `QueryKey` whose purpose is product optimization;
- no optimizer decision stored in `SemanticSnapshot`;
- source/LSP results identical with optimization enabled/disabled;
- editing a component type changes P1 lowering and naturally changes the derived plan;
- editing an unrelated method body does not mutate the semantic data declaration product;
- REPL compilation produces correct fresh planner state per cell.

**Edit operations:**
1. [ ] Add compile-after-edit regression using existing incremental session harness.
2. [ ] Add REPL two-cell regression with same local names.
3. [ ] Run source-index/LSP data tests from P1 unchanged.
4. [ ] Search semantic crates for new optimizer policy symbols; expected zero.
5. [ ] Verify compiled chunk contains no reflectable optimizer metadata.
6. [ ] Record “optimizer plan is transient backend state” as a C1 invariant.

**Testing classification:** C5.

## Task 21 — Add deterministic and measured performance qualification

**Purpose:** Demonstrate P2 reduces the costs it was created to remove without hiding regressions behind timing noise.

**Risk:**
- correctness: LOW;
- performance interpretation: HIGH.

**Owned files/symbols:**
- `phalcom-core/benches/vm_bench.rs`.
- focused allocation/disassembly tests.
- performance evidence section in implementation state.

**Inspect before editing:**
- `phalcom-core/benches/vm_bench.rs` benchmark setup/teardown.
- `benchmarks/vm/run.sh`.
- repository performance baseline instructions/design note.
- P1 allocation counters and C2/C4 deterministic fixtures.

**Source of truth:** deterministic allocation/opcode assertions establish mechanism; named Criterion baselines quantify performance but do not override correctness.

**Dependencies:** Tasks 19–20.

**Hard deterministic matrix:**

| Fixture | Disabled | Enabled required |
|---|---|---|
| `Point(x: 1, y: 2).x` | 1 materialized DataObject | 0 |
| local `Point`; two projections | 1 DataObject | 0 |
| local Point; one opaque use | 1 eager DataObject | 1 sunk DataObject at use |
| branch-only opaque use not taken | 1 eager DataObject | 0 |
| nested `Rect<Point,Point>` projections | outer + nested allocations | 0 |
| 9-leaf data | canonical P1 | canonical P1 |
| captured data | canonical P1 | canonical P1 |
| eligible general enum + match | 1 case object | 0 |
| enum + identity use | 1 case object | 1 canonical case object |
| NativeOption | existing P1 behavior | identical existing behavior |

Add Criterion cases in `vm_bench.rs` for:
- tight loop constructing two-Int data and reading one/two components;
- nested four-leaf data projection;
- exact general enum construct+match;
- one canonical bailout control.

Benchmark both Enabled and Disabled with identical source/semantic setup where harness ergonomics allow it.

**Performance methodology:**
- use explicit Criterion baseline names or the repository's documented reproducible method;
- do not interpret the rolling default baseline as a stable historical comparison;
- report compiler-time as well as run-time change for optimizer-heavy fixtures;
- the optimization is acceptable only if planner cost does not create a material bootstrap/compiler regression.

**Edit operations:**
1. [ ] Add deterministic allocation/opcode qualification tests.
2. [ ] Add Criterion product cases.
3. [ ] Record Disabled baseline using a named stable baseline.
4. [ ] Record Enabled comparison.
5. [ ] Run `benchmarks/vm/run.sh` or the current equivalent.
6. [ ] Record runtime median/range and compiler/bootstrap effect.
7. [ ] If the eight-leaf threshold is changed, record measurements for both thresholds and update the constant deliberately.
8. [ ] Never weaken hard allocation assertions because a timing result looks favorable.

**Testing classification:** C5 performance + deterministic gate.

## Task 22 — Final documentation, state, negative gates, and delivery suite

**Purpose:** Close P2 with reproducible evidence and no hidden scope creep.

**Risk:**
- semantic: MEDIUM.

**Owned files/symbols:**
- LANG005 C1 implementation-state document.
- optimizer docs/comments.
- all P2 focused test/benchmark files.
- repository formatting/check/test/clippy gates.

**Inspect before editing:**
- current CI command set.
- P1 final negative/deletion gates.
- any new files/symbols introduced during P2 that require documentation or exhaustive searches.

**Source of truth:** actual executed evidence recorded in the state file; no checkpoint status is inferred from code inspection alone.

**Dependencies:** all prior tasks.

**Edit operations:**
1. [ ] Update LANG005 C1 implementation state with C1.P2 checkpoint statuses.
2. [ ] Record final optimizer API/type names and threshold.
3. [ ] Record all Enabled/Disabled differential evidence.
4. [ ] Record allocation/disassembly matrix.
5. [ ] Record benchmark method/results.
6. [ ] Update PDR/spec implementation notes only where they describe shipped representation optimization; do not amend PDR-0035 semantics.
7. [ ] Run all final negative searches.
8. [ ] Run affected crate suites, then workspace gates.
9. [ ] Perform deferred-evidence audit.
10. [ ] Mark P2 COMPLETE only with no active incident.
11. [ ] Set next resume action to C1.P3.

**Testing classification:** final P2 delivery gate; every command/evidence item is recorded before COMPLETE.

Checkpoint C5 completion:
- [ ] inliner/fusion composition green;
- [ ] no semantic/incremental optimizer authority created;
- [ ] deterministic allocation matrix green;
- [ ] benchmark evidence recorded;
- [ ] broad gates green;
- [ ] negative gates green;
- [ ] no deferred P2 evidence;
- [ ] no active incident;
- [ ] P2 COMPLETE.

---

# 10. Implementation details by compiler path

This section is binding guidance for the tasks above.

## 10.1 Simple local binding interception

The virtual path is deliberately narrow.

Pseudo-flow:

```rust
match statement {
    Statement::Let(binding)
        if product_opt.enabled()
        && binding_is_simple_local_name(&binding)
        && let Some(plan) = product_opt.plan_for_binding(&binding) =>
    {
        match plan {
            ProductOptimizationDecision::Virtualize(plan) => {
                self.compile_virtual_product_binding(binding, plan)?;
            }
            ProductOptimizationDecision::Materialize(_) => {
                self.compile_binding_canonical(binding)?;
            }
        }
    }

    Statement::Let(binding) => self.compile_binding_canonical(binding)?,

    // all other currently committed statement arms remain on their existing paths
}
```

Do not duplicate the entire normal binding compiler. Extract the smallest existing canonical helper if the current statement arm is monolithic.

A virtual binding must end at the same compiler/runtime stack height as the normal statement at the statement boundary.

## 10.2 Variable read interception

When the ordinary bare-name resolver returns a local slot:

```rust
if let Some(virtual_value) = self.active_virtual_product(slot) {
    return self.emit_virtual_whole_value_use(virtual_value, expr.range());
}
```

This whole-value fallback applies only to `data`.

Data projection compilation must intercept earlier so:

```phalcom
p.x
```

does not compile `p` whole first.

Enum planner guarantees no ordinary whole read exists. Reaching this branch for a virtual enum is an optimizer invariant failure.

## 10.3 Component-path interception

Recognize a nested property chain structurally but validate every edge against P1 lowering:

```text
Var(p)
  ↓ P1-resolved component `topLeft`
GetProperty
  ↓ P1-resolved component `x`
GetProperty
```

P2 may then ask the active virtual shape for `[topLeft, x]`.

It may **not** infer that `"topLeft"`/`"x"` are data components solely from source text.

## 10.4 Constructor storage order

Example:

```phalcom
data Pair(first: Int, second: Int)

Pair(second: side2(), first: side1())
```

If Phalcom evaluates arguments in source order, P2 must execute:

```text
side2()
store logical second leaf
side1()
store logical first leaf
```

not declaration order.

Materialization later may load `first`, then `second` if P1's constructor recipe expects declaration-order arguments. Evaluation order and storage/materialization order are separate.

## 10.5 Why leaf slots remain `Value`

P1's materialized `ProductStorage` may represent proven `Int` as one raw 64-bit payload word. P2 does not copy that encoding into frame slots.

The running VM stack is a GC root array of `Value`. Putting a raw `i64` into it without a `Value` tag would break:

- GC root tracing;
- stack inspection;
- scratch relocation;
- call ABI;
- exception/fiber parking.

P2's win is:

```text
remove aggregate allocation
remove aggregate projection
delay materialization
flatten nested identity-free data
```

not:

```text
replace VM stack Value with native machine registers
```

A future typed/register backend may use the same virtual-product proofs to do deeper scalar unboxing.

## 10.6 Sacred inline block copies

A source literal block can be:
- compiled as a real closure;
- compiled inline by sacred-call optimization;
- compiled again as the deopt fallback copy.

Do not cache one mutable `ProductFunctionPlan` keyed only by source range globally. Plans are immutable proof descriptions; active slot state is per codegen body/copy.

## 10.7 Product plan failure is not a user diagnostic

Optimizer proof failure means:

```text
ProductOptimizationDecision::Materialize(reason)
```

and canonical P1 codegen.

It must not produce a language error.

Only an impossible contradiction after a `Virtualize` decision—such as an unexpected enum whole-value use—may be reported as an internal compiler invariant failure.

---

# 11. Required tests by semantic risk

## 11.1 Evaluation order

Use visible logging/side effects:

```phalcom
var log = []

fn mark(_ n) {
  log.append(n)
  n
}

const p = Pair(second: mark(2), first: mark(1))
System.print(p.first)
System.print(log)
```

Enabled and Disabled must have identical log/order.

## 11.2 Error timing

Test throws in:
- first source argument;
- middle source argument;
- unprojected source argument after projected value;
- nested data argument.

The optimizer must not return a projection before a later source argument that canonical execution would evaluate.

## 11.3 Capture fallback

```phalcom
const p = Point(x: 1, y: 2)
const f = { p }
f()
```

must remain canonical P1 materialization.

Also test nested block only reading `p.x`; P2 still falls back in this checkpoint. Do not add multi-component closure capture.

## 11.4 Mutable/reassignment fallback

Any source form permitting reassignment of the binding must remain materialized even if all current reads are projections.

## 11.5 Alias fallback

```phalcom
const p = Point(x: 1, y: 2)
const q = p
q.x
```

remains canonical in P2. Virtual alias propagation is a later optimization; do not accidentally treat `q` as the same slot range.

## 11.6 Identity-sensitive enum controls

Any enum whole-value observation must force canonical materialization. The test suite must specifically prove the optimizer does not reconstruct a fresh case object at each use.

## 11.7 GC and fibers

A child heap object reachable only from a virtual data/enum component slot must survive collection in:
- running fiber;
- parked/suspended fiber if the current concurrency runtime supports the fixture.

---

# 12. Failure protocol

A failing checkpoint becomes `INCIDENT`.

Before broadening the patch:

1. record exact command/test and observed failure;
2. identify the direct path:
   ```text
   P1 lowering fact
       -> P2 planner decision
       -> virtual slot emission
       -> VM behavior
   ```
3. compare with Disabled canonical execution;
4. classify:
   - `PROOF` — planner accepted an unsafe candidate;
   - `EMISSION` — accepted proof correct, bytecode/slot emission wrong;
   - `P1 CONTRACT` — canonical materialization/lowering itself violates P1;
   - `SCRATCH/STACK` — existing reserve/release/upvalue behavior fails;
   - `MATCH BACKEND` — virtual-match adapter diverges from existing pattern staging;
   - `BASELINE`;
   - `PLAN DRIFT`;
5. state the narrow symbol family allowed to change;
6. fix the narrowest owner.

Default response to a proof ambiguity is **more conservative materialization**, not new type inference.

Later checkpoints may not rely on an unresolved incident.

---

# 13. Implementation state protocol

Continue the P1 state file rather than creating a disconnected optimizer diary.

Required P2 section:

```markdown
# LANG005.C1 implementation state

## C1.P2 repository revision
- P2 base:
- current:

## P1 takeover interface map
| P2 concept | final P1 symbol/path |
|---|---|

## P2 checkpoint status
- C0 — COMPLETE | INCIDENT | NOT STARTED
- C1 — COMPLETE | INCIDENT | NOT STARTED
- C2 — COMPLETE | INCIDENT | NOT STARTED
- C3 — COMPLETE | INCIDENT | NOT STARTED
- C4 — COMPLETE | INCIDENT | NOT STARTED
- C5 — COMPLETE | INCIDENT | NOT STARTED

## P2 established invariants
- I-P2-01: Disabled mode is canonical P1 lowering.
- I-P2-02: Virtual leaves are ordinary `Value` frame slots and are not a second runtime representation.

## P2 decisions
- D-P2-01: Record each implementation-time mechanical adaptation that differs from this plan and the evidence that preserves the same optimizer law.

## Evidence ledger
| Checkpoint | Command | Result | Proves |
|---|---|---|---|

## Differential matrix
| Fixture | Disabled | Enabled | Allocation/opcode delta | Result |
|---|---|---|---|---|

## Benchmark evidence
- named baseline:
- command:
- runtime result:
- compiler/bootstrap result:

## Negative gates
- command -> result -> intentional exceptions

## Deferred to C1.P3 or later
- item -> owner

## Active incident
None.

## Next resume action
Begin LANG005.C1.P3.
```

Store facts/evidence, not chain-of-thought.

---

# 14. Final negative gates

Run after C5.

No optimizer policy in semantic query authority:

```bash
rg 'ProductOptimization|VirtualProduct|ScalarReplacement' phalcom-semantic/src
```

Expected: zero production hits unless P1 already established a semantic product with coincidentally similar naming. Any hit must be explained.

No new product optimizer runtime object:

```bash
rg 'VirtualProduct|ProductOptimizationMode' phalcom-core/src/heap phalcom-core/src/vm
```

Expected: zero production runtime representation hits. Tests/comments are reviewed separately.

No compiler-side data/variant name guessing:

```bash
rg '"Point"|"Some"|"Ok"|"Error"' phalcom-core/src/compiler/lib/product_opt.rs
```

Expected: zero identity special cases.

No anonymous product takeover:

```bash
rg 'TupleLiteral|RecordLiteral|BuildTuple|BuildRecord' \
  phalcom-core/src/compiler/lib/product_opt.rs
```

Expected: zero optimizer ownership of anonymous tuples/records in P2.

No interprocedural ABI bytecodes:

```bash
rg 'VirtualParam|VirtualReturn|MultiValueReturn|UnboxedParam' \
  phalcom-core/src phalcom-semantic/src
```

Expected: zero P2 additions.

No NativeOption generalization:

Inspect all `NativeOption` representation branches touched by P2. Expected behavior remains P1/TYPE001 special handling.

No whole enum rematerializer:

```bash
rg 'materialize.*variant|rematerialize.*variant|emit_virtual.*variant.*material' \
  phalcom-core/src/compiler
```

Expected: no P2 helper that reconstructs a general enum case from virtual payload after the original allocation was elided.

No optimizer control in language surface:

```bash
rg 'productOptimization|scalarReplacement|virtualProduct' \
  phalcom-core/core phalcom-ast phalcom-lsp
```

Expected: zero surface configuration.

---

# 15. Focused test commands

Exact test filters may adapt to final P1 module names, but create stable `product_opt` prefixes so these commands work.

```bash
cargo +stable test -p phalcom-core product_opt
```

Proves:
- planner and compiler optimizer unit/integration tests.

```bash
cargo +stable test -p phalcom-core data_
```

Proves:
- P1/P2 data runtime regressions.

```bash
cargo +stable test -p phalcom-core algebraic_data
```

Proves:
- enum/ADT runtime/match preservation.

```bash
cargo +stable test -p phalcom-semantic adts
```

Proves:
- P2 did not disturb semantic ADT/GADT authority.

Run P1's focused semantic data filter as recorded in its state file.

For benchmarks:

```bash
cargo +stable bench -p phalcom-core --features benchmarks --bench vm_bench
```

and the repository harness:

```bash
benchmarks/vm/run.sh
```

Use the repository's current stable-baseline procedure when the harness/documentation has evolved.

---

# 16. Final broad delivery gates

After all focused C5 evidence is green:

```bash
cargo +stable fmt --all -- --check
```

```bash
cargo +stable check --workspace --all-targets
```

```bash
cargo +stable test -p phalcom-ast
cargo +stable test -p phalcom-modules
cargo +stable test -p phalcom-semantic
cargo +stable test -p phalcom-core
cargo +stable test -p phalcom-lsp
```

```bash
cargo +stable test --workspace --all-targets
```

```bash
cargo +stable clippy --workspace --all-targets -- -D warnings
```

If current CI at implementation time runs additional feature matrices, native metadata validation, or platform-specific tests relevant to modified files, run those exact current commands and record them in the state file.

---

# 17. Staged commit groups

Recommended sequence:

1. `refactor(compiler): add product optimization planning and disabled control`
2. `refactor(compiler): add virtual product slot substrate`
3. `feat(compiler): eliminate ephemeral data product allocations`
4. `feat(compiler): scalar-replace local immutable data`
5. `feat(compiler): sink data materialization to opaque boundary`
6. `feat(compiler): flatten nested virtual data products`
7. `feat(compiler): virtualize non-observable exact enum cases`
8. `feat(match): read virtual exact variant payloads directly`
9. `test(lang005): differential-test product optimizer semantics`
10. `perf(lang005): benchmark product scalar replacement`
11. `docs(lang005): record C1.P2 optimization evidence`

Do not combine the basic data scalar-replacement commit with the enum match optimization. The separate boundary is valuable when diagnosing identity/GADT regressions.

---

# 18. Known scope exclusions

C1.P2 does **not** implement:

- anonymous tuple/record convergence — C1.P3;
- exhaustive Dynamic box/unbox/re-narrow qualification — C1.P3;
- virtual alias propagation;
- multi-whole-use lazy cached data materialization;
- captured virtual products / multi-component upvalue ABI;
- mutable virtual product locals;
- global/static/field virtual products;
- interprocedural escape analysis;
- multi-value function/block ABI;
- unboxed native machine-register frame representation;
- untagged Int/Float VM locals;
- JIT;
- whole-program specialization;
- generic method monomorphization;
- inline `List<Data>`/array-of-struct or struct-of-array storage;
- nested general-enum flattening;
- enum allocation-identity re-ruling;
- NativeOption redesign;
- general match jump tables;
- match decision DAG/shared-prefix optimizer except the exact-virtual-scrutinee shortcut required here;
- broad payload-extraction CSE for already-materialized enums;
- inherent `impl` devirtualization;
- trait witness devirtualization;
- FFI layout;
- legacy `@data` removal.

These exclusions are deliberate. P2 establishes a reusable proof/emission substrate without turning LANG005.C1 into a general optimizer rewrite.

---

# 19. Deferred-evidence audit

Before marking P2 complete, every evidence item named by C0–C5 must be one of:

```text
PASS with command/result recorded
or
explicitly reclassified outside P2 scope with written architectural reason
or
active release-blocking INCIDENT
```

The following are scope exclusions, not missing P2 evidence:
- P3 anonymous product support;
- P3 Dynamic/reification stress;
- captured/mutable/interprocedural virtualization;
- JIT/native-register unboxing.

No P2 checkpoint can be COMPLETE while one of its own differential/allocation/GC/match gates is merely deferred.

---

# 20. Release-complete criteria

LANG005.C1.P2 is COMPLETE only when all of the following are true:

- P1 remains COMPLETE and its canonical materialization path is intact.
- `ProductOptimizationMode::Disabled` reproduces P1 behavior.
- optimizer planning is backend-only and consumes resolved P1 semantics.
- simple positive-arity data constructor→projection can execute with zero DataObject allocation.
- eligible immutable local data can remain component `Value` slots across multiple projections.
- one required whole-data use is materialized exactly at that use rather than eagerly.
- constructor arguments still evaluate exactly once in original source order.
- unselected components are still evaluated.
- captured, mutable, global, destructuring, expanded, unproven, over-budget, and multi-whole-use candidates fall back.
- nested identity-free data constructions can flatten recursively within the leaf budget.
- projecting a nested data value can materialize only that nested subvalue when required.
- whole outer materialization reconstructs exact nested data semantics through P1 recipes.
- eligible general exact enum cases can avoid case-object allocation only when all uses are resolved match/payload uses.
- an eligible virtual exact enum match preserves source arm order and uses existing semantic candidates.
- enum identity-observing/opaque uses force canonical allocation.
- GADT, exact-case, variant-local-generic, behavior, GC, and NativeOption suites remain green.
- virtual component slots survive GC and fiber suspension through existing stack-root semantics.
- sacred inline fast/fallback copies do not share mutable optimizer state.
- chunk superinstruction fusion remains correct after optimized emission.
- no new optimizer semantic query authority exists.
- source/LSP/reflection output is independent of optimizer choice.
- deterministic allocation/opcode matrix passes.
- Criterion evidence shows the intended runtime benefit without unacceptable compiler/bootstrap regression.
- all final negative gates pass.
- all affected/workspace format/check/test/clippy gates pass.
- implementation state contains complete C0–C5 evidence and no active incident.
- next resume action is LANG005.C1.P3.

At that point C1.P3 may rely on a new invariant:

> **The compiler has a proven, representation-independent virtual-product mechanism: a product may live as scalar `Value` leaves, projections may read those leaves directly, nested identity-free data may flatten, and materialization can be inserted at an explicit boundary without changing source semantics.**
