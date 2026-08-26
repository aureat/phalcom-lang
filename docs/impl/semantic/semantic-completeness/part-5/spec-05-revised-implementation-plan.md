# Spec 05 (Revised) Implementation Plan — Advanced Type Semantics, Effects, Totality, Contracts, and Proofs

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the advanced semantic domains ratified by `05-advanced-kinds-constraints-effects-and-proofs-REVISED.md` without creating parallel type, flow, dispatch, contract, or diagnostic authorities.

**Architecture:** Build Spec 05 strictly on the compiler-owned semantic substrate established by Specs 01–04.5. Record rows become a canonical row-backed structural type domain; effects, exits, termination, contracts, and proofs remain independent callable products; verification consumes the 04.5 flow graph and expression/call analysis rather than reinterpreting raw AST; native declarations flow through the canonical 03.5 native-surface pipeline; advanced results are separate `SemanticDb` products with explicit budgets, invalidation, and publication rules.

**Tech Stack:** Rust 2024 workspace, `phalcom-semantic`, `phalcom-native-meta`, `phalcom-native-decl`, `phalcom-native-macros`, `phalcom-native-surface`, `phalcom-type-meta`, `phalcom-core`, `phalcom-lsp`, compiler-owned `SemanticDb`, deterministic `BTreeMap`/stable-ID products, no mandatory external solver dependency in the initial VC implementation.

**Spec:** `docs/work/analyses/typing/05-advanced-kinds-constraints-effects-and-proofs-REVISED.md`

**Repository baseline for this revision:** current `main` observed on 2026-08-23. The latest repository head observed during planning was `1b7230c1f9df11097114621a7b26182ba88f5012`; the immediately relevant semantic-foundation implementation is commit `36304deccbcdbb59f1fbf34249d52c4056a1f53b` (`feat(semantic): add analysis and diagnostics foundations`). Re-ground before implementation if `main` advances.

## Global Constraints

- Static type/effect/proof metadata must not participate in selector identity, ordinary method lookup, runtime class identity, instance layout, allocation semantics, or inline-cache keys.
- The six callable semantic products are independent: normal return type, effect summary, exit summary, termination knowledge, contract set, proof evidence/status.
- `Never` is a normal-return fact only. It does not imply divergence or termination.
- Empty effects do not imply termination and do not imply absence of raises.
- `@total` means termination proven under the selected semantic model. It does not mean pure and does not forbid raises, mutation, I/O, or scheduling.
- Runtime contract guards remain executable runtime behavior according to existing compile-mode rules. Guard execution is not static proof evidence.
- `Unknown`, `Blocked`, `Cancelled`, `BudgetExceeded`, `InternalFailure`, and `Disproven` are distinct states. No failure-to-prove path may silently become success.
- Row solver variables are session-local and may never enter canonical `TypeStore`, snapshots, metadata, runtime reflection, or durable proof artifacts.
- Proof backend choice remains gated. Initial implementation must not add a mandatory SMT runtime dependency.
- Backend SAT/UNSAT output is not automatically trusted proof. Trust must be explicit and evidence/policy-backed.
- Current 04.5 semantic products are authoritative inputs. Spec 05 must not build a second flow graph, second call resolver, second expression type checker, or LSP-only formal analysis path.
- Native effect/raise/termination facts must originate from the canonical native declaration pipeline, not from a hand-maintained semantic side table.
- Missing native termination metadata defaults to `Unknown`. No inference from purity, accessor-ness, `ReturnFlowSpec::Never`, or absence of visible loops is permitted.
- New advanced queries must be budgeted/cancellable and may publish only coherent complete/explicitly partial products according to `SemanticDb` publication rules.
- No implementation step may claim build/test/REPL success unless that command was actually run in the implementing checkout.

---

# 1. Revision Summary and Ratified Decisions

This plan replaces the earlier Spec-05 implementation plan. It incorporates the review decisions and current repository state.

## 1.1 Ratified decision: one canonical record-row representation

Use a canonical row arena. All structural records, including closed records, are represented through a canonical `RecordRowId`.

Target shape:

```rust
pub struct RecordTypeData {
    pub row: RecordRowId,
}

pub enum TypeData {
    // ... existing variants ...
    Record(RecordTypeData),
}

pub struct RecordRowData {
    pub fields: Box<[RecordRowField]>,
    pub tail: RecordRowTail,
}

pub enum RecordRowTail {
    Closed,
    Parameter(TypeParameterId),
}
```

Do **not** keep closed records as `TypeData::Record(Box<[RecordTypeField]>)` while introducing a second representation for open rows. There must be one semantic representation.

A compact storage optimization for tiny closed records may be introduced later behind the same semantic interface; it must not create a second equality domain.

## 1.2 Ratified decision: native termination is explicit and conservative

Add:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TerminationSpec {
    Unknown,
    Terminates,
    MayDiverge,
}
```

Every existing primitive initially maps to `Unknown` unless explicitly audited. Pure math/accessor primitives may be annotated `Terminates` only through explicit declaration changes backed by review/tests. `ReturnFlowSpec::Never` cannot be used as termination evidence.

## 1.3 Ratified decision: backend-free proof semantics first

Split proof work into three layers:

```text
5A  canonical proof procedure + deterministic VC generation + normalization
5B  tiny deterministic baseline reasoner/model validator
5C  backend protocol + trust policy + durable proof artifacts/cache
```

The VC generator must exist and be testable without external solvers. Heavy solver integration remains gated.

## 1.4 Ratified decision: consume 04.5 analysis products

Current `main` already contains important 04.5 foundations:

- `phalcom-semantic/src/checker/analysis.rs`
  - `ExpressionAnalysis`
  - `BindingState`
  - `CallableAnalysis`
- `phalcom-semantic/src/checker/flow/`
  - `graph.rs`
  - `predicate.rs`
  - `state.rs`
  - `transfer.rs`
- `phalcom-semantic/src/checker/inference.rs`
  - solver-local `InferenceSession`
  - `InferVarId != TypeId`
- shared `phalcom-diagnostics` extraction has already begun/landed.

Spec 05 analysis must consume these products. Do not recreate them.

## 1.5 Ratified decision: `@total` is a source-level static requirement

`@total` must be added to source attribute plumbing. It does not weave a runtime guard.

At minimum:

```text
phalcom-ast          BuiltinAttr::Total
parser/legality      accept @total only on supported callable declarations
phalcom-semantic     TerminationRequirement::Total
termination query    requires TerminationKnowledge::Proven
metadata/reflection  optional projected fact per Specs 02/03
```

## 1.6 Ratified decision: canonical contract identity is separate from proof admissibility

A runtime-valid contract may be statically unsupported. Therefore do not store a boolean `purity_verified` in canonical `ContractDecl`.

Use separate semantic products:

```rust
pub struct ContractDecl {
    pub id: ContractId,
    pub expression: ContractExprId,
    pub source: SourceOrigin,
    pub runtime_policy: RuntimeContractPolicy,
}

pub enum ContractAdmissibility {
    Admissible(ContractModelFacts),
    Unsupported(ContractUnsupportedReason),
    Blocked(ContractBlockedReason),
    Invalid(ContractDiagnosticSet),
}
```

Static proof eligibility requires more than empty effects and proven termination: deterministic semantics, a supported logical model, no unmodeled dynamic/reflective/FFI boundary, and safe state-read semantics are also required.

---

# 2. Current Repository State to Preserve

This section is implementation context for executors. Re-check every observation before editing if `main` changes.

## 2.1 Kind layer

`phalcom-semantic/src/types/kind.rs` already contains:

```rust
pub enum KindData {
    Type,
    RecordRow,
    Arrow { parameters: Box<[KindId]>, result: KindId },
}
```

`KindId::RECORD_ROW` therefore already exists as part of the semantic kind universe. Do not introduce a second row-kind enum.

## 2.2 Type store

`phalcom-semantic/src/types/store.rs` currently stores records directly:

```rust
TypeData::Record(Box<[RecordTypeField]>)
```

and `TypeStore::record(...)` interns that form. This is the main Phase-1 migration target.

The same store currently allows generic parameters to be materialized as `TypeData::Parameter(TypeParameterId)` using the parameter's kind. That is insufficient for the row/type-domain separation required by Spec 05. A `RecordRow` parameter must not be publishable as an ordinary type term simply because it has a `TypeParameterId`.

## 2.3 04.5 analysis substrate

`phalcom-semantic/src/checker/analysis.rs` currently publishes expression and callable analysis models. `BindingState` already distinguishes persistent declared constraints from current flow knowledge.

`phalcom-semantic/src/checker/flow/graph.rs` contains the formal callable `FlowGraph` with nodes, edges, branch predicates, entries, joins, loop headers, and exits.

`phalcom-semantic/src/checker/inference.rs` already introduces solver-local `InferenceSession` and explicitly documents the law that inference variables are never canonical `TypeStore` nodes.

Spec 05 must treat these as dependencies, not alternatives.

## 2.4 Native metadata

`phalcom-native-meta/src/primitive.rs` currently defines:

```rust
RaisesSpec
NativeEffect
EffectSpec
ReturnFlowSpec
PrimitiveSurfaceSpec
```

but does not yet contain `TerminationSpec`.

The native declaration pipeline includes at least:

```text
phalcom-native-decl/src/parse.rs
phalcom-native-decl/src/normalized.rs
phalcom-native-decl/src/validate.rs
phalcom-native-macros/src/lib.rs
phalcom-native-meta/src/primitive.rs
phalcom-native-surface/
phalcom-semantic native-surface import
```

Termination metadata must traverse all of these layers.

## 2.5 Source attributes

`phalcom-ast/src/ast.rs` currently recognizes built-in attributes such as `requires`, `ensures`, and `invariant`, but has no `BuiltinAttr::Total` at the reviewed baseline.

## 2.6 Query graph

`phalcom-semantic/src/db/key.rs` currently contains coarse keys including:

```rust
CallableBody(CallableId)
ModuleDiagnostics(ModuleId)
ModuleMetadata(ModuleId)
```

but no dedicated advanced-analysis product keys. Spec 05 must extend the query graph rather than storing all advanced facts as eager fields of `CallableBody`.

---

# 3. Target Semantic Dependency Graph

The target dependency graph is:

```text
04.5 callable analysis
    ├── ExpressionAnalysis
    ├── CallResolution
    ├── FlowGraph
    ├── FlowState / FlowPredicate
    └── callable dependencies
            │
            ├──────────────► effects
            │                   │
            │                   └── interprocedural effect SCC
            │
            ├──────────────► control facts
            │                   ├── raises
            │                   ├── may return normally
            │                   ├── suspension/process-exit candidates
            │                   └── cycle candidates
            │                          │
            │                          ▼
            │                     termination
            │                          │
            │                          ▼
            │                  finalized exit summary
            │
            └──────────────► canonical contracts
                                │
                                ├── admissibility
                                │
                                └── proof procedure lowering
                                         │
                                         ▼
                                        VCs
                                         │
                              ┌──────────┴───────────┐
                              ▼                      ▼
                       baseline reasoner        external backend
                              │                      │
                              └──────────┬───────────┘
                                         ▼
                                   ProofResult
                                         │
                                         ▼
                                 trust/artifact layer
```

Critical non-cycle rule:

```text
ControlFacts -> Termination -> ExitSummary
```

Do not define `ExitSummary::divergence = ProvenAbsent` before termination has established absence of divergence, otherwise termination and exit analysis acquire a circular dependency.

---

# 4. File/Module Map

The following file structure is the target. Existing modules should be extended where they already own the concept; new files below are proposals unless already present at execution time.

## `phalcom-semantic`

```text
src/types/
    row.rs                  canonical record-row data/IDs
    row_solver.rs           solver-local row terms/lacks/unification
    store.rs                row arena and TypeData::Record migration
    annotation.rs           open/closed record lowering
    relation.rs             structural row relations

src/effects/
    mod.rs
    atom.rs                 EffectAtom/EffectSet
    knowledge.rs            EffectKnowledge + opaque reasons
    infer.rs                intra/interprocedural effect analysis

src/control/
    mod.rs
    facts.rs                pre-termination control facts
    infer.rs                callable control extraction
    exit.rs                 finalized ExitSummary

src/termination/
    mod.rs                  TerminationKnowledge/public API
    analyze.rs              graph/SCC termination analysis over 04.5 FlowGraph
    ranking.rs              ranking argument recognition
    evidence.rs             proof/counterevidence/block reasons

src/contracts/
    mod.rs
    ir.rs                   canonical contract expressions/declarations
    lower.rs                source -> canonical contract IR
    admissibility.rs        proof-model admissibility

src/proof/
    mod.rs
    procedure.rs            CFG/SSA-like proof procedure IR
    logic.rs                backend-neutral scalar logic terms
    vc.rs                   deterministic VC generation
    normalize.rs            canonical normalization/order
    fingerprint.rs          VC/assumption/dependency fingerprints
    baseline.rs             tiny deterministic reasoner/model validation
    result.rs               ProofResult + reason taxonomies
    policy.rs               proof policy keys; no external process yet

src/db/
    key.rs                  advanced query keys
    ...                     query storage/dependency publication integration

src/snapshot.rs             read-only projections of advanced products
src/diagnostic.rs           structured advanced diagnostic codes/details
src/export.rs               metadata export hooks
src/metadata/               advanced metadata adaptation where appropriate
```

## Native pipeline

```text
phalcom-native-meta/src/primitive.rs
phalcom-native-decl/src/parse.rs
phalcom-native-decl/src/normalized.rs
phalcom-native-decl/src/validate.rs
phalcom-native-macros/src/lib.rs
phalcom-native-surface/      generated surface transport/serialization as currently structured
```

## Source/runtime/tooling

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs                 if builtin-attribute parsing is centralized there
phalcom-core/src/compiler/attributes.rs  runtime contract bridge; preserve existing weave
phalcom-type-meta/                        advanced metadata extension/profile carriage
phalcom-core/src/primitive/typing.rs      reflection projection only when Spec-03 capability gates allow
phalcom-lsp/src/hover.rs
phalcom-lsp/src/diagnostics.rs            or current diagnostic adapter path
```

---

# 5. Phase 0 — Rebase and Freeze the 04.5 Interfaces Spec 05 Will Consume

**Goal:** prevent Spec 05 from coding against stale 04.5 assumptions or duplicating infrastructure that already landed.

**Files:**
- Inspect: `phalcom-semantic/src/checker/analysis.rs`
- Inspect: `phalcom-semantic/src/checker/call.rs`
- Inspect: `phalcom-semantic/src/checker/flow/{graph,predicate,state,transfer}.rs`
- Inspect: `phalcom-semantic/src/checker/inference.rs`
- Inspect: `phalcom-semantic/src/explain/`
- Inspect: `phalcom-semantic/src/db/key.rs`
- Test: existing 04.5 semantic tests

**Interfaces consumed by later phases:**

```rust
ExpressionAnalysis
CallableAnalysis
FlowGraph
FlowNodeId
FlowEdgeId
FlowPredicate / PredicateId
CallResolution / CallResolutionId
CallableId
ExplanationId
```

- [ ] Record the exact current fields/signatures of the above types in the implementation branch's plan notes.
- [ ] Verify that Spec 05 can map every call expression to a semantic callee/call outcome without redoing selector resolution from AST.
- [ ] Verify that `FlowGraph` exposes enough branch/join/loop structure for effect, control, termination, and proof lowering. If it lacks an explicit semantic event needed later, extend the existing graph rather than introducing a second CFG.
- [ ] Verify that cancellation/budget facilities used by `SemanticDb` queries are reusable for advanced analyses.
- [ ] Run focused 04.5 tests before any Spec-05 change and record the baseline result.

**Gate P0:** No Spec-05 module begins implementation until there is one agreed set of 04.5 analysis interfaces to consume.

---

# 6. Phase 1 — Canonical Record Rows

## Task 1.1 — Introduce row identity and canonical row arena

**Files:**
- Create: `phalcom-semantic/src/types/row.rs`
- Modify: `phalcom-semantic/src/types/id.rs`
- Modify: `phalcom-semantic/src/types/mod.rs`
- Modify: `phalcom-semantic/src/types/store.rs`
- Test: `phalcom-semantic/tests/record_rows.rs`

**Produces:**

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RecordRowId(u32);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RecordRowField {
    pub name: Box<str>,
    pub ty: TypeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RecordRowTail {
    Closed,
    Parameter(TypeParameterId),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RecordRowData {
    pub fields: Box<[RecordRowField]>,
    pub tail: RecordRowTail,
}
```

**Canonicalization laws:**

1. fields sorted by canonical field name order;
2. duplicate labels rejected before interning;
3. every field type is a proper `Type`-kind form;
4. open tail parameter has kind `RecordRow`;
5. equal canonical data interns to equal `RecordRowId` within one `TypeStore`;
6. row IDs are store-relative and never transferable unchecked between stores.

**Test-first cases:**

```rust
#[test]
fn row_field_order_is_canonical() {
    // #{b: Int, a: String} and #{a: String, b: Int} intern to same RecordRowId.
}

#[test]
fn duplicate_row_field_is_rejected() {
    // #{a: Int, a: String} cannot be interned.
}

#[test]
fn row_tail_requires_record_row_kind_parameter() {
    // A Type-kind T cannot be installed as RecordRowTail::Parameter(T).
}
```

## Task 1.2 — Make every record type row-backed

**Files:**
- Modify: `phalcom-semantic/src/types/store.rs`
- Modify exhaustive `TypeData::Record` matches found by repository search
- Test: existing type-store/record tests + `phalcom-semantic/tests/record_rows.rs`

Replace:

```rust
TypeData::Record(Box<[RecordTypeField]>)
```

with:

```rust
pub struct RecordTypeData {
    pub row: RecordRowId,
}

TypeData::Record(RecordTypeData)
```

`TypeStore::record(fields)` remains as a convenience only if it canonicalizes a `RecordRowData { tail: Closed }` first.

Add an explicit open-row constructor:

```rust
pub fn record_with_row(&mut self, row: RecordRowId) -> TypeId
```

or equivalent checked API.

**Deletion gate:** no production code directly constructs a record's field vector inside `TypeData` after this task.

## Task 1.3 — Separate row parameters from type terms

**Files:**
- Modify: `phalcom-semantic/src/types/store.rs`
- Modify: `phalcom-semantic/src/types/parameter.rs`
- Modify: annotation/generic signature lowering paths that materialize parameters
- Test: `phalcom-semantic/tests/record_rows.rs`

Current generic parameter materialization must not permit a `RecordRow` parameter to become an ordinary `TypeData::Parameter` and then flow through ordinary `TypeId` APIs as though it were a value type.

Introduce kind-specific checked materialization. One acceptable API shape is:

```rust
pub enum GenericParameterForm {
    Type(TypeId),
    Row(RecordRowTail),
}

pub fn parameter_form(&mut self, id: TypeParameterId) -> Result<TypeId, ParameterFormError>
pub fn record_row_parameter(&self, id: TypeParameterId) -> Result<RecordRowTail, ParameterFormError>
```

`parameter_form` returns an error for `RecordRow` kind parameters.

**Hard test:** a `RecordRow` binder must be structurally impossible to publish as a normal `TypeId` parameter node.

## Task 1.4 — Implement solver-local row terms

**Files:**
- Create: `phalcom-semantic/src/types/row_solver.rs`
- Test: `phalcom-semantic/tests/row_solver.rs`

**Produces:**

```rust
pub struct RecordRowVarId(u32);

pub enum RecordRowTerm {
    Canonical(RecordRowId),
    Var(RecordRowVarId),
    Extend {
        fields: Box<[RecordRowFieldTerm]>,
        tail: Box<RecordRowTerm>,
    },
}

pub struct RecordRowLacks {
    pub row: RecordRowVarId,
    pub label: Box<str>,
}

pub enum RecordRowSolveResult {
    Solved(RecordRowSolution),
    Rejected(RecordRowFailure),
    Blocked(RecordRowBlockedReason),
    Cancelled,
    BudgetExceeded(RowBudgetReport),
    InternalFailure(AnalysisIncidentId),
}
```

The row solver owns temporary row variables exactly as `InferenceSession` owns ordinary inference variables: they never enter the canonical store.

Implement:

- deterministic field subtraction;
- row unification;
- lacks-constraint propagation;
- row occurs check;
- kind checks;
- cancellation checks;
- deterministic step/pair budgets;
- explicit blocked reasons for unsupported/capability-dependent relation cases.

**No fallback:** budget exhaustion or unsupported relation never materializes `Dynamic`, `Unknown`, or a closed row silently.

## Task 1.5 — Structural record relations and access capabilities

**Files:**
- Modify: `phalcom-semantic/src/types/relation.rs`
- Add focused helper in `types/row.rs` or `types/row_relation.rs` if relation.rs becomes unwieldy
- Test: `phalcom-semantic/tests/record_rows.rs`

Initial policy:

```text
ReadOnly
    width allowed
    corresponding fields covariant

ReadWrite
    field types invariant
    exact field set in version 1 unless a later aliasing model explicitly permits width

WriteOnly
    conservative exact field set + equivalent field types in version 1

Capability unavailable / aliasing model insufficient
    Blocked, never permissive success
```

This gives implementation semantics to the spec's access-capability distinction without claiming a more permissive alias model than the runtime currently guarantees.

## Task 1.6 — Source lowering and metadata compatibility

**Files:**
- Modify: `phalcom-semantic/src/types/annotation.rs`
- Modify: metadata exporter under `phalcom-semantic/src/metadata/` or `export.rs` as current ownership dictates
- Modify: `phalcom-type-meta` only for open-row carriage required by revised Spec 02
- Test: annotation lowering + metadata round-trip tests

Rules:

- closed record source forms lower to `RecordRowTail::Closed`;
- open record forms lower to `RecordRowTail::Parameter(...)` after kind validation;
- closed canonical rows may continue exporting to the pre-existing closed-record metadata wire representation for compatibility;
- open rows use the advanced/open-row metadata shape only where required;
- row solver variables are rejected by metadata publication.

**Phase-1 exit gate:** every canonical record type is row-backed; no solver row variable is publishable; existing closed-record behavior remains semantically equivalent.

---

# 7. Phase 2 — Control Facts and Effect Summaries

## Task 2.1 — Compact effect representation

**Files:**
- Create: `phalcom-semantic/src/effects/{mod,atom,knowledge}.rs`
- Test: `phalcom-semantic/tests/effects.rs`

Use a compact value representation first. With six fixed atoms, an interned heap set is unnecessary.

```rust
#[repr(u8)]
pub enum EffectAtom {
    Mutation,
    Io,
    Scheduling,
    Reflection,
    Nondeterminism,
    Blocking,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct EffectSet(u16);

impl EffectSet {
    pub const EMPTY: Self = Self(0);
    pub fn contains(self, atom: EffectAtom) -> bool;
    pub fn insert(&mut self, atom: EffectAtom);
    pub fn union(self, other: Self) -> Self;
    pub fn is_subset_of(self, other: Self) -> bool;
}
```

Semantic knowledge:

```rust
pub enum EffectKnowledge {
    Known(EffectSet),
    Opaque(EffectOpaqueReason),
    Invalid(EffectDiagnosticSet),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(AnalysisIncidentId),
}
```

Do not represent missing metadata as `Known(EMPTY)`.

## Task 2.2 — Introduce pre-termination control facts

**Files:**
- Create: `phalcom-semantic/src/control/{mod,facts}.rs`
- Test: `phalcom-semantic/tests/control_summary.rs`

Create a lower-level control product that does not require termination to be known:

```rust
pub struct ControlFacts {
    pub may_return_normally: bool,
    pub raises: RaiseKnowledge,
    pub may_exit_process: bool,
    pub may_suspend: bool,
    pub cycle_candidates: Box<[ControlCycleId]>,
}
```

`ControlFacts` is extracted from the 04.5 `FlowGraph` and analyzed calls. It must not itself claim `divergence = ProvenAbsent`.

## Task 2.3 — Intraprocedural effect/control extraction from semantic analysis

**Files:**
- Create: `phalcom-semantic/src/effects/infer.rs`
- Create: `phalcom-semantic/src/control/infer.rs`
- Consume: `checker/analysis.rs`, `checker/flow/*`, call-resolution products
- Test: `phalcom-semantic/tests/effects.rs`, `control_summary.rs`

Do not walk raw AST to rediscover semantic calls if `ExpressionAnalysis`/`CallResolution` already identifies the call.

Direct syntax-owned effects may inspect semantic expression kinds where needed:

- local assignment alone is not necessarily externally observable `Mutation`; classify mutation according to the ratified effect definition and place/escape semantics;
- field/collection mutation records `Mutation` where externally visible state may change;
- calls join callee effect knowledge;
- native calls adapt canonical `EffectSpec`;
- dynamic dispatch/reflection/DNU/foreign boundaries produce `Opaque` reasons unless the target set is statically closed enough to join known callees.

Control extraction records:

- explicit `return`/normal tail paths;
- raises/throws and known callee raises;
- process-exit primitives when metadata identifies them;
- suspension/yield/future semantics when identified by formal call/native metadata;
- graph cycles as candidates, not automatic divergence.

## Task 2.4 — Interprocedural SCC inference

**Files:**
- Extend: `phalcom-semantic/src/effects/infer.rs`
- Extend: `phalcom-semantic/src/control/infer.rs`
- Reuse: `SemanticDb` dependency/scheduler primitives
- Test: recursive-call fixtures

Use deterministic SCC fixed points over callable dependencies.

Effect lattice:

```text
Known(S1) join Known(S2) = Known(S1 ∪ S2)
Known(_) join Opaque(r)  = Opaque(join reason/provenance)
Opaque join Opaque       = Opaque
Invalid propagates as invalid/cause-aware
Cancelled/BudgetExceeded never publish as success
```

Raise/control joins similarly preserve opacity rather than inventing an empty set.

**Phase-2 exit gate:** source/native callable effect/control products are queryable and deterministic, recursive SCCs converge or return explicit bounded outcomes, and no termination conclusion is yet smuggled into control facts.

---

# 8. Phase 3 — Native Termination Metadata and `@total` Source Plumbing

## Task 3.1 — Add `TerminationSpec` end-to-end through native declarations

**Files:**
- Modify: `phalcom-native-meta/src/primitive.rs`
- Modify: `phalcom-native-decl/src/parse.rs`
- Modify: `phalcom-native-decl/src/normalized.rs`
- Modify: `phalcom-native-decl/src/validate.rs`
- Modify: `phalcom-native-macros/src/lib.rs`
- Modify generated native-surface transport as required by its current files
- Modify semantic native import adapter
- Test: native-declaration parser/normalization/surface tests

Add:

```rust
pub enum TerminationSpec {
    Unknown,
    Terminates,
    MayDiverge,
}
```

and:

```rust
pub struct PrimitiveSurfaceSpec {
    // existing fields
    pub termination: TerminationSpec,
}
```

Native attribute syntax should use one explicit spelling, for example the repository's existing key/value style if available:

```text
termination = "terminates"
termination = "may_diverge"
```

Do not invent a second annotation syntax if the native declaration parser already has a canonical key grammar; follow its established convention.

Migration policy:

```text
omitted legacy termination => Unknown
explicit terminates        => Terminates
explicit may-diverge       => MayDiverge
```

No automatic promotion from `Pure`, `Known([])`, accessor selector shape, `ReturnFlowSpec::Never`, or `raises` metadata.

## Task 3.2 — Add `@total` to source attributes

**Files:**
- Modify: `phalcom-ast/src/ast.rs`
- Modify parser/attribute legality path that calls `BuiltinAttr::parse`
- Modify semantic declaration extraction
- Test: parser/attribute legality + semantic tests

Add:

```rust
BuiltinAttr::Total
```


with name `"total"` in `BuiltinAttr::name` and `BuiltinAttr::parse`.

Legality:

- allowed on methods/getters/setters/indexers/callables supported by the semantic callable model;
- rejected on fields/type aliases/non-callable declarations unless a future spec explicitly assigns meaning;
- duplicate `@total` is rejected or deduplicated according to existing builtin-attribute duplicate policy; do not create a new inconsistency.

Semantic representation:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminationRequirement {
    Unspecified,
    Total,
}
```

Attach this to the semantic callable declaration/surface side table, not runtime dispatch identity.

`@total` produces **no runtime guard weave**.

## Task 3.3 — Preserve native/source convergence

**Files:**
- Modify canonical native-surface semantic adapter
- Modify source callable semantic surface publication
- Test: source/native callable advanced fact parity

Both source and native callables must expose termination declarations through one compiler-owned query interface even though their evidence origins differ.

For example:

```rust
pub enum TerminationDeclaration {
    SourceRequirement(TerminationRequirement),
    NativeSpec(TerminationSpec),
    None,
}
```

This is provenance, not a second termination semantics.

**Phase-3 exit gate:** `@total` parses and reaches semantic callable identity; native termination metadata traverses the entire generated surface; all omitted legacy native entries are `Unknown`.

---

# 9. Phase 4 — Termination Analysis and Final Exit Summaries

## Task 4.1 — Define termination knowledge without collapsing bounded failures

**Files:**
- Create: `phalcom-semantic/src/termination/{mod,evidence}.rs`
- Test: `phalcom-semantic/tests/termination.rs`

Target result:

```rust
pub enum TerminationKnowledge {
    Proven(TerminationEvidence),
    Refuted(TerminationCounterevidence),
    Blocked(TerminationBlockedReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(AnalysisIncidentId),
}
```

Do not encode cancellation/budget/internal failure as `Blocked` merely to shorten the enum. Existing semantic relation/query architecture already preserves these distinctions.

Evidence categories may include:

```rust
pub enum TerminationEvidence {
    AcyclicControlFlow,
    AllCalleesTerminate { callees: Box<[CallableId]> },
    RankingFunction(RankingEvidence),
    StructuralRecursion(StructuralRecursionEvidence),
    NativeDeclared,
}
```

Counterevidence is reserved for sound evidence of non-totality, such as a statically reachable unconditional cycle with no exit under the supported model or an explicitly trusted native `MayDiverge` declaration when that declaration semantically guarantees existence of a diverging admitted execution.

If `MayDiverge` is only a conservative possibility marker in the chosen native metadata semantics, map it to `Blocked(NativeMayDiverge)` rather than `Refuted`. Choose and document one meaning before annotating primitives.

## Task 4.2 — Analyze the existing 04.5 `FlowGraph`; do not create `termination/cfg.rs`

**Files:**
- Create: `phalcom-semantic/src/termination/analyze.rs`
- Consume: `phalcom-semantic/src/checker/flow/graph.rs`
- Test: `phalcom-semantic/tests/termination.rs`

The old plan proposed a second `termination/cfg.rs`. Delete that proposal.

Implement graph algorithms over `FlowGraph`:

- reachable-node calculation;
- SCC/cycle detection;
- identification of loop headers/back edges using existing graph structure;
- callee termination joins from call analysis;
- branch-sensitive exclusion of unreachable cycles where 04.5 has direct reachability proof;
- budget/cancellation checks in graph traversal.

Acyclic reachable control flow with only proven-terminating callees may yield `Proven(AcyclicControlFlow/AllCalleesTerminate)`.

## Task 4.3 — Ranking arguments

**Files:**
- Create: `phalcom-semantic/src/termination/ranking.rs`
- Test: `phalcom-semantic/tests/termination.rs`

Initial recognizers are deliberately narrow and sound:

1. integer induction variable known non-negative at loop entry;
2. update known to decrease by a strictly positive constant;
3. loop guard establishes exit when measure reaches a lower bound;
4. no mutation/call invalidates the ranking-place fact in the loop body;
5. recursive structural call demonstrably uses a strict substructure under a supported structural model.

Failure to recognize a measure returns `Blocked(UnsupportedTerminationPattern)`; it is not evidence of divergence.

## Task 4.4 — `@total` validation

**Files:**
- Extend: termination query/declaration validator
- Modify: `phalcom-semantic/src/diagnostic.rs`
- Test: `phalcom-semantic/tests/termination.rs`

Rules:

```text
@total + Proven        -> accepted
@total + Refuted       -> error totality.refuted
@total + Blocked       -> error totality.unproven with blocked reason
@total + BudgetExceeded-> explicit totality.analysis_budget_exceeded
@total + Cancelled     -> query cancelled; do not publish a false compile error as stable semantic truth
```

A total callable may raise, mutate, perform I/O, or return `Never`; those dimensions are diagnosed independently.

## Task 4.5 — Finalize exit summaries after termination

**Files:**
- Create: `phalcom-semantic/src/control/exit.rs`
- Test: `phalcom-semantic/tests/control_summary.rs`

Target:

```rust
pub struct ExitSummary {
    pub may_return_normally: bool,
    pub raises: RaiseKnowledge,
    pub divergence: DivergenceKnowledge,
    pub may_exit_process: bool,
    pub may_suspend: bool,
}

pub enum DivergenceKnowledge {
    ProvenAbsent,
    Possible,
    Opaque(DivergenceOpaqueReason),
}
```

Finalize divergence from `ControlFacts + TerminationKnowledge`:

```text
TerminationKnowledge::Proven
    -> DivergenceKnowledge::ProvenAbsent

sound nontermination counterevidence
    -> DivergenceKnowledge::Possible

blocked/opaque/cancelled/budgeted analysis
    -> Opaque or unpublished query state according to the enclosing query result
```

**Phase-4 exit gate:** no `Never`/effect inference is used as termination evidence; `@total` is enforced; exit summaries cannot claim `ProvenAbsent` without termination evidence.

---

# 10. Phase 5 — Canonical Contract Semantics

## Task 5.1 — Stable contract identity

**Files:**
- Create: `phalcom-semantic/src/contracts/{mod,ir}.rs`
- Modify callable/declaration identity helpers as needed
- Test: `phalcom-semantic/tests/contracts.rs`

Target:

```rust
pub struct ContractId {
    pub owner: ContractOwner,
    pub kind: ContractKind,
    pub index: u16,
}

pub enum ContractOwner {
    Callable(CallableId),
    Declaration(DeclarationId),
}

pub enum ContractKind {
    Requires,
    Ensures,
    Invariant,
}
```

IDs are deterministic from semantic owner + source declaration order. They are not object pointers and do not depend on runtime weaving order.

## Task 5.2 — Canonical contract expression IR

**Files:**
- Extend: `phalcom-semantic/src/contracts/ir.rs`
- Test: `phalcom-semantic/tests/contracts.rs`

Use normalized semantic references rather than raw AST names:

```rust
pub enum ContractExpr {
    Bool(bool),
    Int(i128),
    Local(BindingId),
    Parameter { callable: CallableId, index: u16 },
    ResultValue,
    Old(ContractExprId),
    Not(ContractExprId),
    And(Box<[ContractExprId]>),
    Or(Box<[ContractExprId]>),
    Equal(ContractExprId, ContractExprId),
    NotEqual(ContractExprId, ContractExprId),
    Less(ContractExprId, ContractExprId),
    LessEqual(ContractExprId, ContractExprId),
    Greater(ContractExprId, ContractExprId),
    GreaterEqual(ContractExprId, ContractExprId),
    StableProjection(StableProjectionId),
    Unsupported(ContractUnsupportedSyntax),
}
```

Do not initially model arbitrary mutable member reads as ordinary scalar expressions. General heap reasoning requires a heap/alias/frame model and must return explicit unsupported/unknown status until ratified.

`old(expr)` is legal only where the selected contract kind has a pre-state meaning and `expr` is capturable/modelable.

## Task 5.3 — Lower source contracts without replacing runtime weaving

**Files:**
- Create: `phalcom-semantic/src/contracts/lower.rs`
- Modify: `phalcom-core/src/compiler/attributes.rs` only to attach/preserve stable semantic identity metadata where architecture permits
- Test: semantic contract lowering + existing runtime contract tests

Preserve current runtime guard behavior and compile-mode matrix.

The static semantic path consumes the source contracts independently and maps them to `ContractDecl`.

Do not make the runtime compiler depend on proof success before weaving a guard.

## Task 5.4 — Separate contract admissibility from contract identity

**Files:**
- Create: `phalcom-semantic/src/contracts/admissibility.rs`
- Consume: effect, termination, control, call-analysis products
- Test: `phalcom-semantic/tests/contracts.rs`

Target:

```rust
pub enum ContractAdmissibility {
    Admissible(ContractModelFacts),
    Unsupported(ContractUnsupportedReason),
    Blocked(ContractBlockedReason),
    Invalid(ContractDiagnosticSet),
}
```

A contract is proof-admissible only if all semantic dependencies needed by the selected logic are modeled:

- known allowed/empty effect profile required by the initial proof model;
- proven termination for predicate evaluation;
- deterministic semantics;
- no unsupported dynamic dispatch, reflection, DNU, FFI, scheduler behavior, or opaque native result in the predicate;
- every state read has a supported stability/heap interpretation;
- all scalar operators have defined proof semantics.

A contract that fails admissibility remains a valid runtime contract unless existing runtime contract rules independently reject it.

**Phase-5 exit gate:** runtime contract behavior is unchanged; every contract has stable semantic identity; proof eligibility is explicit and not encoded as a boolean on `ContractDecl`.

---

# 11. Phase 6 — Backend-Neutral Proof Procedure and VC Generation

## Task 6.1 — Proof procedure IR over semantic flow

**Files:**
- Create: `phalcom-semantic/src/proof/{mod,procedure}.rs`
- Consume: 04.5 `FlowGraph`, expression/call analysis, canonical contracts
- Test: `phalcom-semantic/tests/proof_procedure.rs`

Do not lower raw AST directly to formulas. Introduce a program-semantic proof procedure:

```rust
pub struct ProofProcedure {
    pub owner: CallableId,
    pub blocks: Box<[ProofBlock]>,
    pub entry: ProofBlockId,
}

pub struct ProofBlock {
    pub id: ProofBlockId,
    pub assumptions: Box<[LogicExprId]>,
    pub statements: Box<[ProofStatement]>,
    pub terminator: ProofTerminator,
}

pub enum ProofTerminator {
    Goto(ProofBlockId),
    Branch {
        condition: LogicExprId,
        if_true: ProofBlockId,
        if_false: ProofBlockId,
    },
    Return(Option<LogicExprId>),
    Raise(Option<TypeId>),
    Diverge,
    Unsupported(ProofUnsupportedReason),
}
```

Use versioned locals/SSA-like names or an equivalent deterministic environment so branch joins are explicit rather than relying on mutable AST traversal state.

## Task 6.2 — Logic IR

**Files:**
- Create: `phalcom-semantic/src/proof/logic.rs`
- Test: logic normalization unit tests

Initial logic fragment:

```rust
pub enum LogicExpr {
    Bool(bool),
    Int(i128),
    Symbol(LogicSymbolId),
    Not(LogicExprId),
    And(Box<[LogicExprId]>),
    Or(Box<[LogicExprId]>),
    Equal(LogicExprId, LogicExprId),
    NotEqual(LogicExprId, LogicExprId),
    Add(LogicExprId, LogicExprId),
    Subtract(LogicExprId, LogicExprId),
    MultiplyByConstant { value: LogicExprId, constant: i128 },
    Less(LogicExprId, LogicExprId),
    LessEqual(LogicExprId, LogicExprId),
    Greater(LogicExprId, LogicExprId),
    GreaterEqual(LogicExprId, LogicExprId),
    Ite {
        condition: LogicExprId,
        if_true: LogicExprId,
        if_false: LogicExprId,
    },
}
```

Do not add nonlinear arithmetic, general heap arrays, quantifiers, higher-order predicates, or arbitrary user calls merely because a backend could theoretically support them.

## Task 6.3 — Deterministic verification-condition generation

**Files:**
- Create: `phalcom-semantic/src/proof/vc.rs`
- Test: `phalcom-semantic/tests/contracts_and_proofs.rs`

Obligation kinds:

```rust
pub enum ProofObligationKind {
    CallPrecondition,
    CallablePostcondition,
    InvariantPreservation,
    Assertion,
    TerminationMeasure,
}
```

Generate at least:

- call-site precondition obligations using specialized actual/formal substitution;
- postcondition obligations on each normal return path only;
- invariant obligations only at ratified invariant checkpoints;
- `old(...)` references bound to pre-state symbols;
- explicit unsupported terminators/facts when the model cannot lower an operation.

Partial correctness rule: exceptional exit and divergence do not satisfy normal-return postconditions; they are modeled separately rather than converted into `false`/`true` shortcuts.

## Task 6.4 — Canonical normalization and VC fingerprinting

**Files:**
- Create: `phalcom-semantic/src/proof/normalize.rs`
- Create: `phalcom-semantic/src/proof/fingerprint.rs`
- Test: deterministic fingerprint tests

Normalization rules must be deterministic and versioned. Examples:

- sort commutative `And`/`Or` operands by canonical structural key;
- flatten nested same operators;
- canonicalize integer literal representation;
- normalize local/proof symbol numbering by stable procedure traversal, not allocation addresses;
- preserve semantically meaningful order for non-commutative operations;
- include semantic-model version in fingerprint domain separation.

Create a **VC fingerprint**, not yet a durable proof artifact:

```rust
pub struct VerificationConditionFingerprint {
    pub digest: [u8; 32],
    pub semantic_model_version: SemanticModelVersion,
}
```

The digest covers normalized obligation + assumptions + proof-relevant dependency signatures/contracts/facts.

**Phase-6 exit gate:** deterministic VCs can be generated, rendered structurally, fingerprinted, cancelled/budgeted, and tested without any external solver.

---

# 12. Phase 7 — Tiny Deterministic Baseline Reasoner

This phase is optional for initial architecture completion but useful for end-to-end proving of the supported scalar fragment. It must stay intentionally small.

**Files:**
- Create: `phalcom-semantic/src/proof/baseline.rs`
- Create/extend: `phalcom-semantic/src/proof/result.rs`
- Test: `phalcom-semantic/tests/baseline_proof.rs`

Result model:

```rust
pub enum ProofResult {
    Proven(ProofEvidence),
    Disproven(Counterexample),
    Unknown(ProofUnknownReason),
    Blocked(ProofBlockedReason),
    Cancelled,
    BudgetExceeded(ProofBudgetReport),
    InternalFailure(AnalysisIncidentId),
}
```

Initial baseline capabilities may include:

- boolean constant propagation;
- direct assumption lookup;
- propositional simplification;
- equality/reflexivity;
- interval-style reasoning for simple integer comparisons;
- simple affine implications such as `x > 10` proving `x > 0`;
- validation/replay of candidate scalar counterexamples.

Do not label arbitrary baseline success as `KernelChecked` unless the baseline engine is explicitly defined as a trusted proof kernel with a versioned derivation checker. Until then, use a trust classification that accurately reflects its authority or keep trust outside `ProofResult` until Phase 8.

Counterexample rule:

```text
candidate model found
    + model replay validates assumptions and negated goal
        => Disproven
    otherwise
        => Unknown(UnvalidatedCounterexample)
```

**Phase-7 exit gate:** the baseline reasoner is sound for its declared fragment and returns `Unknown` for everything outside it.

---

# 13. Phase 8 — Proof Backend, Trust, and Durable Artifact Platform

This phase is deliberately gated. Before implementation, perform a dependency/threat-model review and reconsider a dedicated `phalcom-proof` crate to keep external process/solver SDK/cache dependencies out of the core semantic crate.

## Task 8.1 — Proof policy and backend protocol

**Files:**
- Prefer new crate if approved: `phalcom-proof/`
- Otherwise isolate under a dedicated module with no default external dependency
- Modify semantic adapter only through a narrow trait/interface

Suggested interface:

```rust
pub trait ProofBackend {
    fn identity(&self) -> ProofBackendIdentity;
    fn prove(&self, request: &ProofRequest, budget: &ProofBudget, cancellation: &CancellationToken) -> BackendProofResult;
}
```

No backend receives VM objects or raw AST.

## Task 8.2 — Explicit proof trust

Define:

```rust
pub enum ProofTrust {
    KernelChecked {
        kernel: ProofKernelIdentity,
    },
    TrustedBackend {
        backend: ProofBackendIdentity,
        policy: ProofPolicyKey,
    },
}
```

Do not add `Assumed` as a proof trust tier. Assumptions are inputs to obligations and should remain visible as assumptions, not proofs.

## Task 8.3 — Durable proof artifacts

Only now introduce:

```rust
pub struct ProofArtifact {
    pub vc_fingerprint: VerificationConditionFingerprint,
    pub assumptions_fingerprint: AssumptionFingerprint,
    pub dependencies_fingerprint: DependencyFingerprint,
    pub backend: ProofBackendIdentity,
    pub policy: ProofPolicyKey,
    pub trust: ProofTrust,
    pub evidence: DurableProofEvidence,
}
```

The cache key must cover every proof-relevant semantic world component, including:

- normalized VC;
- assumptions/contracts;
- callable signatures used;
- callee contracts used;
- effect/termination/control facts relied on;
- native surface semantic revision;
- semantic-model version;
- arithmetic/heap-model version;
- backend identity/version/options;
- proof policy;
- proof-kernel version where applicable;
- dependency package/module fingerprints.

Approximate/stale cache matches never preserve `Proven`.

**Phase-8 exit gate:** proof trust is auditable and durable evidence cannot be replayed into a semantically different world.

---

# 14. Phase 9 — `SemanticDb` Advanced Query Products

## Task 9.1 — Add separate advanced query keys

**Files:**
- Modify: `phalcom-semantic/src/db/key.rs`
- Modify query storage/scheduler modules under `phalcom-semantic/src/db/`
- Test: `phalcom-semantic/tests/db_advanced.rs`

Add conceptually:

```rust
pub enum QueryKey {
    // existing keys...
    CallableEffects(CallableId),
    CallableControl(CallableId),
    CallableTermination(CallableId),
    CallableContracts(CallableId),
    VerificationConditions(CallableId),
    ProofResult(VerificationConditionId, ProofPolicyKey),
}
```

If the database's type-erasure/storage design requires a different key payload, retain the same semantic granularity.

Why separate keys:

- body typing should not eagerly prove contracts;
- effect analysis can invalidate independently of proof backend policy;
- proof results depend on policy/backend/model versions not relevant to ordinary typing;
- LSP hover can request effects without requesting proof;
- cancellation/budgets differ by query class.

## Task 9.2 — Dependency recording and invalidation

Define exact dependencies:

```text
CallableEffects
    depends on CallableBody + callee surfaces/effect products + native metadata

CallableControl
    depends on CallableBody + callee exit/native raise/flow products

CallableTermination
    depends on CallableControl + FlowGraph + callee termination products + native termination

CallableContracts
    depends on source declaration + semantic expression/call facts used to lower contracts

VerificationConditions
    depends on contracts + FlowGraph + expression/call facts + selected effect/control/termination facts

ProofResult
    depends on VC fingerprint + proof policy/backend/model version
```

Add cold-vs-incremental differential tests for every product.

## Task 9.3 — Publication states

Do not publish cancelled/budget/internal failures as successful facts. Reuse explicit query states from the existing DB architecture.

Where a product itself semantically supports `Opaque`/`Blocked`, distinguish that from query infrastructure cancellation/budget failure.

**Phase-9 exit gate:** advanced analyses are independently queryable, dependency-tracked, invalidated, cancellable, and do not make ordinary callable-body analysis invoke a proof backend.

---

# 15. Phase 10 — Snapshot, Metadata, Reflection, Diagnostics, CLI/LSP

## Task 10.1 — Snapshot accessors are projections, not storage authority

**Files:**
- Modify: `phalcom-semantic/src/snapshot.rs`
- Test: snapshot API tests

Expose read-only queries such as:

```rust
callable_effects(CallableId) -> EffectKnowledge
callable_control(CallableId) -> ControlFacts
callable_exits(CallableId) -> ExitSummary
callable_termination(CallableId) -> TerminationKnowledge
callable_contracts(CallableId) -> Arc<[ContractDecl]>
verification_conditions(CallableId) -> VerificationConditionSet
proof_result(VerificationConditionId, ProofPolicyKey) -> ProofResult
```

Do not make `callable_proofs(CallableId) -> &[ProofResult]` the fundamental API because proof result identity is obligation- and policy-specific.

## Task 10.2 — Advanced metadata carriage

**Files:**
- Modify: `phalcom-type-meta` according to Spec-02 extension/profile rules
- Modify: `phalcom-semantic/src/export.rs` and metadata modules
- Test: metadata profile/round-trip tests

Metadata should carry durable advanced facts only when the selected profile/capability requires them.

At minimum define explicit optional sections for:

- effect summary;
- exit summary;
- termination declaration/status where publishable;
- canonical contract declarations;
- proof summary/trust/artifact references only where policy permits.

Do not serialize solver-local row/type variables, active proof-backend process state, cancellation tokens, raw semantic arenas, or VM objects.

## Task 10.3 — Runtime reflection projection

**Files:**
- Modify: `phalcom-core/src/primitive/typing.rs` only according to Spec-03 capability APIs
- Modify runtime typing registry/metadata adapters where required
- Test: reflection capability + no-allocation-unless-requested tests

Advanced reflection remains lazy/capability-gated.

Do not add effect/termination/proof fields to `MethodObject` layout merely for reflection. Resolve through snapshot/metadata side tables.

No reflected proof result may become runtime dispatch authority.

## Task 10.4 — Structured diagnostics

**Files:**
- Modify: `phalcom-semantic/src/diagnostic.rs`
- Reuse: explanation graph / `phalcom-diagnostics`
- Test: structural + rendered diagnostic tests

Add stable codes in families:

```text
row.*
effect.*
totality.*
contract.*
proof.*
```

Required distinctions include:

```text
totality.refuted
totality.unproven
totality.analysis_budget_exceeded
contract.effectful_predicate
contract.nonterminating_predicate
contract.unsupported_semantics
proof.disproved
proof.unknown
proof.blocked
proof.stale_artifact
proof.budget_exceeded
```

Diagnostics should cite structured explanation/proof provenance rather than embedding backend logs as primary messages.

## Task 10.5 — LSP presentation must be demand-driven

**Files:**
- Modify: `phalcom-lsp/src/hover.rs`
- Modify current LSP diagnostic/code-action adapters
- Test: LSP integration tests

Hover may display already available:

- effect summary;
- exits/raises;
- totality status;
- contracts;
- proof summaries.

But a routine hover/type query must not silently launch expensive proof work. If an explicit editor action requests proof, route through a cancellable proof query with an appropriate budget.

Suggested UX split:

```text
normal hover
    shows cached/published advanced facts only

"Verify contracts" action
    runs explicit VC/proof queries

"Explain proof" action
    renders derivation/counterexample/trust details
```

**Phase-10 exit gate:** compiler, CLI, LSP, metadata, and reflection observe the same advanced semantic products without altering runtime dispatch or forcing eager proof allocation.

---

# 16. Phase 11 — Performance, Adversarial Testing, and Runtime-Invariance Gates

## 11.1 Required benchmarks

Create benchmark/metrics coverage for:

- cold callable effect/control analysis;
- warm cached advanced query;
- body-only edit invalidating one callable;
- signature/contract edit invalidating reverse dependents;
- recursive SCC effect/control analysis;
- deep open-row unification;
- row occurs-check adversarial inputs;
- loop/SCC termination analysis;
- VC count and generation time for branch-heavy methods;
- baseline prover budget exhaustion;
- proof cache hit/miss if Phase 8 is enabled;
- LSP hover without proof request;
- explicit proof request cancellation;
- TypeStore/row-arena growth across repeated no-op analyses.

## 11.2 Runtime invariance tests

Prove by tests/structural assertions that:

- selector keys contain no effects/exits/termination/contracts/proof data;
- ordinary runtime dispatch performs no proof/effect lookup;
- `MethodObject` layout is unchanged by advanced static facts unless an independently ratified runtime ABI change says otherwise;
- a program's runtime dispatch behavior is unchanged by adding/removing purely static `@total` metadata;
- runtime contract weave behavior remains governed by existing compile mode, not static proof result;
- proof availability does not remove runtime guards unless a separate optimization specification explicitly ratifies such elimination.

## 11.3 Adversarial soundness tests

Include:

- opaque reflection inside a contract;
- DNU/open-world call from a supposedly pure function;
- mutable field read in a proof predicate across an unknown mutating call;
- recursive call graph with mixed known/unknown native termination;
- solver cancellation during SCC traversal;
- stale proof artifact after callee contract change;
- stale proof artifact after native semantic metadata change;
- same VC under different semantic-model version;
- candidate counterexample that fails model replay;
- row tail unification cycle;
- wrong-kind row parameter attempted through generic type APIs.

---

# 17. Migration and Deletion Ledger

No legacy path is deleted merely because a new module exists. Use these completion gates.

| Legacy / transitional item | Delete only when |
|---|---|
| direct `TypeData::Record(Box<[...fields...]>)` | every record constructor/relation/export path uses `RecordRowId`; closed-record parity tests pass |
| any row variable represented as `TypeId` | all row solving uses `RecordRowVarId`; snapshot/metadata negative tests prove non-publication |
| raw-AST effect scanner introduced during experimentation | semantic call/expression analysis provides all formal call facts and parity tests pass |
| proposed second termination CFG | never create; use 04.5 `FlowGraph` |
| runtime-only contract identity | canonical `ContractId` can be mapped to runtime guard/source contract without changing guard semantics |
| syntax-only purity as static proof admissibility | semantic effects/termination/model checks fully own static admissibility; existing runtime heuristic may remain for runtime-specific legality if still needed |
| eager advanced fields embedded in callable-body query | dedicated advanced `SemanticDb` queries are production consumers |
| proof result keyed only by callable | obligation + policy identity is available and all consumers migrate |
| experimental proof cache with incomplete fingerprint | full proof-relevant semantic fingerprint is implemented; stale-world tests pass |
| duplicated LSP advanced reasoning | LSP consumes compiler-owned facts and parity tests show no formal feature regression |

---

# 18. Acceptance Matrix

Spec 05 implementation is not complete merely because modules compile. The following behaviors must hold.

## Record rows

```text
#{a: Int, b: String}
#{b: String, a: Int}
```

canonicalize to the same closed row.

Open row inference supports the ratified form, e.g. a row parameter can preserve additional fields without becoming an ordinary `TypeId` metavariable.

A row occurs check rejects cyclic substitutions.

Wrong-kind row tails are rejected.

Read-only width/covariance works; mutable/write capability cases follow the conservative explicit relation policy.

## Effects/exits

A known pure literal-only callable gets empty effects.

A mutation source/native call propagates `Mutation`.

I/O propagates `Io`.

Dynamic/reflection boundaries remain opaque rather than empty.

Raise information remains independent from effects.

`Never` remains independent from divergence.

## Termination

Straight-line callable with proven-terminating callees -> `Proven`.

Known infinite loop/counterevidence -> `Refuted` only when soundly established.

Unrecognized recursive pattern -> `Blocked`, not `Refuted`.

Recognized decreasing measure -> `Proven`.

A total method may raise/mutate/I/O and remain total.

`@total` rejects unproven/refuted termination with distinct diagnostics.

Missing native termination -> `Unknown`/blocked totality, never implicit `Terminates`.

## Contracts

Source contract has stable `ContractId` independent of runtime weave mode.

Runtime Debug/Release/Unchecked contract behavior remains unchanged.

An executable runtime contract can be statically `Unsupported` without being deleted/rejected as a runtime contract.

`old(...)` captures the defined pre-state only.

General mutable heap reads do not silently become scalar proof terms.

## VCs/proofs

VC generation is deterministic under stable semantic input.

Call preconditions are specialized with actual arguments.

Postconditions are generated on normal returns only.

Exceptional/diverging paths remain distinct.

Unsupported operations yield explicit unknown/blocked results.

Baseline prover proves only its declared fragment.

Counterexamples require model replay before `Disproven`.

Backend result is not automatically `KernelChecked`.

Proof artifact changes/invalidation follow every proof-relevant dependency.

## Query/tooling

Ordinary body typing does not run proof queries.

LSP ordinary hover does not launch heavy proving.

Advanced query cancellation does not publish false success.

Cold and incremental advanced facts are structurally equivalent.

Compiler/LSP/reflection observe the same semantic status where capability/profile allows observation.

---

# 19. Verification Commands

Run focused commands after each task/phase rather than waiting for the full workspace.

Representative sequence; executors must adjust exact test target names to files that actually exist after implementation:

```bash
cargo test -p phalcom-semantic --test record_rows
cargo test -p phalcom-semantic --test row_solver
cargo test -p phalcom-semantic --test effects
cargo test -p phalcom-semantic --test control_summary
cargo test -p phalcom-semantic --test termination
cargo test -p phalcom-semantic --test contracts
cargo test -p phalcom-semantic --test proof_procedure
cargo test -p phalcom-semantic --test contracts_and_proofs
cargo test -p phalcom-semantic --test db_advanced
```

Native pipeline after termination metadata changes:

```bash
cargo test -p phalcom-native-meta
cargo test -p phalcom-native-decl
cargo test -p phalcom-native-macros
cargo test -p phalcom-native-surface
```

Runtime/tooling integration:

```bash
cargo test -p phalcom-core
cargo test -p phalcom-lsp
```

Repository hygiene/final gate:

```bash
cargo fmt --all -- --check
git diff --check
cargo test --workspace
```

If the repository requires additional generated-artifact/graph validation such as `graphify update .`, run the repository-prescribed command after code changes and verify the working tree contains only expected generated changes.

Report verification in four buckets:

```text
PASSING
BASELINE/UNRELATED FAILURE
DEFERRED BY EXPLICIT GATE
NOT RUN / UNVERIFIED
```

Never convert “not run” into “passing.”

---

# 20. Suggested Commit / Review Groups

Keep review surfaces cohesive. A recommended sequence is:

```text
A  canonical row arena + row-backed closed records
B  row solver + relations + source/metadata open rows
C  control facts + effect representation/intraprocedural inference
D  interprocedural effect/control SCCs
E  native TerminationSpec pipeline + @total syntax plumbing
F  termination analysis + finalized exit summaries
G  canonical contract IR + runtime bridge + admissibility
H  proof procedure + logic IR + deterministic VC normalization
I  baseline scalar reasoner/model replay
J  SemanticDb advanced query integration
K  metadata/reflection/diagnostic/LSP projections
L  optional proof backend/trust/artifact platform
M  performance/adversarial/incremental hardening
```

Groups H/J may be ordered so that VC queries are database-backed immediately if that better matches current `SemanticDb` implementation, but do not make proof result computation eager from `CallableBody`.

---

# 21. Explicitly Deferred / Gated Features

The following are not accidental omissions and must not be smuggled into this implementation plan:

- mandatory external SMT solver vendor/runtime;
- general nonlinear arithmetic;
- quantified theorem proving;
- dependent types or `Type :: Type`;
- public effect rows/handlers;
- arbitrary heap separation logic;
- proof terms as ordinary Phalcom runtime values;
- automatic runtime contract guard elimination based on static proof;
- implicit native termination inference from function shape;
- finite-set generic constraints/default generic arguments/associated types unless separately ratified;
- generalized mutable-record width subtyping before alias/capability semantics justify it;
- kind polymorphism beyond the explicitly gated Spec-05 extension points.

A gated feature receives `Unsupported`, `Blocked`, or `Unknown` in the relevant semantic product when encountered; it does not receive an unsound approximation.

---

# 22. Completion Definition

The revised Spec-05 implementation is complete when all of the following are true:

1. canonical records are uniformly row-backed and open-row solving is solver-local, bounded, kind-safe, and non-publishable;
2. source and native callables expose compiler-owned effects/control facts with explicit opacity;
3. termination is independently analyzed and `@total` is a real source-level static requirement;
4. exit summaries use termination evidence rather than `Never` or effect heuristics to classify divergence absence;
5. contracts have stable semantic identity while runtime guard behavior remains unchanged;
6. static contract admissibility is a separate model-aware product;
7. deterministic proof procedures and VCs are generated from the 04.5 semantic flow model rather than raw AST reinterpretation;
8. proof results distinguish proven/disproven/unknown/blocked/cancelled/budget/internal outcomes and counterexamples are validated;
9. proof trust is explicit if/when backend integration is enabled;
10. durable proof artifacts, if enabled, are keyed by the complete proof-relevant semantic world and cannot survive stale dependencies as `Proven`;
11. effects, control, termination, contracts, VCs, and proof results are independent `SemanticDb` products with correct invalidation/cancellation/budget behavior;
12. compiler, LSP, metadata, and runtime reflection consume those products through shared semantic authority without changing runtime dispatch/object layout;
13. focused, incremental, adversarial, performance, and workspace verification gates pass or any non-passing baseline is explicitly documented rather than hidden.

At that point Spec 05 has added advanced semantic reasoning to Phalcom without turning typing/proofs into a second runtime and without weakening the language's message-oriented execution semantics.
