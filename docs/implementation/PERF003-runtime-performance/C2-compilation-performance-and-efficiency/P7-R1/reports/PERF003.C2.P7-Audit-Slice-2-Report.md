# PERF003.C2.P7 — Audit Slice 2 Report

**Audit slice:** 2  
**Date:** 2026-10-15  
**Branch:** `perf/universe-characterization`  
**Audited remote HEAD:** `2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a` (`feat(perf): extend artifact loading foundations`)  
**Primary scope:** `CallableAnalysis` persistence, relocation, compiler-reader parity, deterministic owner-local identity, allocation accounting, incremental fingerprints, `StaticApplication` ownership, and the tests required to remove the callable-analysis fail-closed guards.

---

## 1. Executive conclusion

Slice 2 confirms that `CallableAnalysis` is currently the dominant semantic blocker to P7 closure, and that the remaining work is substantially broader than filling in the existing `CallableAnalysisRecordV1`.

The current wire record is a **partial skeleton**. It preserves callable identity, body source owner/range, top-level analysis status, `exits.unreachable`, the control-flow graph, constructor-super selection, an expression count, expression/binding denotations, the static-argument subset of call specialization, formal binding source attachments, and binding storage origins. The live semantic owner contains substantially more compiler authority, and current code generation consumes many of those omitted products directly.

The accepted P7 owner-graph amendment is correct: complete callable-body authority must be persisted and relocated; it cannot be reconstructed from execution summaries, signatures, names, or source ranges. The existing writer/import guards therefore remain necessary.

This audit additionally found two deeper closure risks that are **not merely missing artifact transport**:

1. `callable_body_product_fingerprint` omits compiler-relevant semantic products, including at minimum `binding_write` and the entire `family_applications` index. Its callable-reference and associated-resolution hashing is also materially incomplete. These gaps can make incremental query identity insensitive to a change in executable meaning.
2. The current `CallableAnalysis` storage model relies heavily on `BTreeMap`, `HashMap`, and `BTreeSet`, while `ExplanationArena` owns a `BTreeMap`. Those native container-node allocations cannot satisfy the exact shared artifact allocation ledger that already forced `FlowGraph` to adopt a frozen `BudgetedSortedMap` backing. Full callable import therefore needs an explicit immutable/budget-aware owner representation seam before `UnaccountedBoundary` can be removed.

**P7 closure should not remove either callable-analysis guard until these issues are resolved.**

---

## 2. Scope and repository evidence

This slice inspected the following live implementation seams at the audited HEAD:

- `phalcom-semantic/src/checker/analysis.rs`
  - `ExpressionAnalysis`
  - `BindingState`
  - `CallSpecialization`
  - `CallableBindingOrigin`
  - `CallableAnalysis`
  - `CallableAnalysisTable`
- `phalcom-semantic/src/checker/context.rs`
  - expression/binding ID allocation
  - `finalize_with_normal_returns`
  - checked-static-application retention
- `phalcom-semantic/src/checker/associated.rs`
  - associated/callable-reference/family-application products
- `phalcom-semantic/src/static_definition.rs`
  - `StaticBodyId`
  - `StaticApplicationId`
  - `CheckedStaticApplication`
  - `CheckedStaticBody`
- `phalcom-semantic/src/checker/static_body.rs`
- `phalcom-semantic/src/artifact/roots.rs`
- `phalcom-semantic/src/artifact/export.rs`
- `phalcom-semantic/src/artifact/import.rs`
- `phalcom-semantic/src/artifact/operational.rs`
- `phalcom-compiler-artifact/src/semantic/world.rs`
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/modules/compile.rs`
- `phalcom-semantic/src/db/fingerprint.rs`
- `phalcom-semantic/src/db/query.rs`
- focused callable-analysis artifact/root/import tests
- current P7 detailed implementation plan and its adopted callable-analysis amendments

No GitHub combined-status checks or pull-request workflow runs are associated with the audited HEAD.

---

## 3. Severity summary

| ID | Severity | Finding |
|---|---|---|
| S2-01 | **CRITICAL** | `CallableAnalysisRecordV1` represents only a small subset of the live owner and cannot reconstruct executable compiler semantics. |
| S2-02 | **CRITICAL** | Type-root registration is still incomplete for several type-bearing callable-analysis families; completing DTOs without extending root closure will produce relocation failures or missing graph nodes. |
| S2-03 | **CRITICAL** | There is no complete callable-analysis importer/materializer. Import is intentionally rejected for every nonempty callable-analysis payload. |
| S2-04 | **HIGH** | `callable_body_product_fingerprint` omits `binding_write`, so assignment-target changes can be invisible to the semantic product fingerprint. |
| S2-05 | **HIGH** | `callable_body_product_fingerprint` omits the entire `family_applications` index even though core lowering consumes it. |
| S2-06 | **HIGH** | Callable-reference fingerprinting is incomplete: `BoundFamily` omits `family_type` and selector spec; the `Associated` branch hashes only a variant tag. |
| S2-07 | **HIGH** | Associated-resolution fingerprinting covers only `DataExpansionField`; most associated selections consumed by lowering are absent from the callable product fingerprint. |
| S2-08 | **HIGH** | Full callable import cannot satisfy exact allocation accounting using the current concrete `BTreeMap`/`HashMap`/`BTreeSet` owner storage. |
| S2-09 | **HIGH** | `CallableAnalysisTable` exposes its `HashMap` through `Deref`, making a frozen/budget-aware imported backing much harder to introduce without first tightening its read API. |
| S2-10 | **HIGH** | `ExpressionAnalysisIndex`, `BindingAnalysisIndex`, resolution indexes, and `ExplanationArena` bake native map-node allocation into the owner. |
| S2-11 | **HIGH** | `import_callable_call_specializations` does not use a child transaction; a later error can leave the caller's allocation ledger debited. |
| S2-12 | **HIGH** | `SemanticWorldV2::validate` does not bound `expression_count` by `max_semantic_entries_per_family`, creating a latent large-count hazard for the future importer. |
| S2-13 | **HIGH** | The source-independent lowering path can silently treat a missing explicit formal attachment as an implicit receiver when broad `source_index` is absent. |
| S2-14 | **HIGH** | Current wire has no expression rows or binding-state rows, so it cannot reconstruct ranges, type knowledge, status, call targets, read/write attachment, mutability, initialization, versions, or causal state. |
| S2-15 | **HIGH** | Generic call specialization transport preserves static arguments but drops all conformance bindings; export rejects them and import reconstructs `conformances: Box::default()`. |
| S2-16 | **HIGH** | `StaticApplication` ownership is now corrected at the producer, but the canonical row is owned by `CheckedStaticBody`, not `CallableAnalysis`; duplicating those rows would create the wrong authority. |
| S2-17 | **HIGH** | Once the broad guard is removed, canonical Universe publication needs an explicit early gate requiring every persisted callable analysis to be `Complete`. |
| S2-18 | **MEDIUM-HIGH** | Deterministic owner-local ordinal assignment is structurally reasonable but not yet proven against allocation-history/body-ID perturbation. |
| S2-19 | **HIGH** | Current callable-analysis tests are synthetic/helper-level. There is no complete source→wire→fresh relocation→fingerprint parity→compiler projection test. |
| S2-20 | **MEDIUM** | Full owner transport will materially increase artifact size and load memory; dense representation is appropriate, but section size and peak allocation need explicit measurement. |

---

# 4. Live owner vs. current wire

## 4.1 `CallableAnalysis` live field inventory

At `phalcom-semantic/src/checker/analysis.rs`, the live owner is effectively:

```rust
pub struct CallableAnalysis {
    pub callable: CallableId,
    pub body_range: SourceRange,
    pub body_source_owner: Option<SourceOwner>,
    pub expressions: ExpressionAnalysisIndex,
    pub bindings: BindingAnalysisIndex,
    pub binding_origins: CallableBindingOriginIndex,
    pub for_iteration_lanes: Arc<ForIterationLaneResolutionIndex>,
    pub positional_expansions: Arc<PositionalExpansionIterationResolutionIndex>,
    pub known_invalid_positional_expansions: Arc<KnownInvalidPositionalExpansionIndex>,
    pub sequence_literals: Arc<SequenceLiteralResolutionIndex>,
    pub associated_resolutions: Arc<AssociatedResolutionIndex>,
    pub callable_reference_resolutions: Arc<CallableReferenceResolutionIndex>,
    pub family_applications: Arc<FamilyApplicationResolutionIndex>,
    pub match_resolutions: Arc<MatchResolutionIndex>,
    pub pattern_site_resolutions: Arc<PatternSiteResolutionIndex>,
    pub flow_graph: Arc<FlowGraph>,
    pub entry_flow: FlowStateSummary,
    pub exits: BodyExitFacts,
    pub return_validation: ReturnContractValidation,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
    pub internal_incidents: Arc<[InternalSemanticIncident]>,
    pub explanations: Arc<ExplanationArena>,
    pub return_explanation: Option<ExplanationId>,
    pub dependencies: Arc<[CallableId]>,
    pub semantic_dependencies: Arc<[SemanticDependency]>,
    pub constructor_super_initializer: Option<ConstructorSuperCallResolution>,
    pub dependency_fingerprint: ProductFingerprint,
    pub status: CallableAnalysisStatus,
}
```

The accepted P7 owner-graph amendment classifies nearly all of this as required compiler artifact authority, with `dependency_fingerprint` derived after relocation and nonempty internal incidents rejected for canonical publication.

## 4.2 Current wire shape

At `phalcom-compiler-artifact/src/semantic/world.rs`:

```rust
record!(CallableAnalysisRecordV1 {
    callable: StableCallableRef,
    body_source_owner: Option<CallableAnalysisSourceOwnerV1>,
    body_range: SpanV1,
    status: CallableAnalysisStatusV1,
    exits_unreachable: bool,
    flow_graph: CallableFlowGraphV1,
    constructor_super_initializer: Option<ConstructorSuperInitializerV1>,
    expression_count: u32,
    expression_denotations: Vec<CallableExpressionDenotationV1>,
    binding_denotations: Vec<CallableBindingDenotationV1>,
    call_specializations: Vec<CallableCallSpecializationV1>,
    formal_bindings: Vec<CallableBindingSourceAttachmentV1>,
    binding_origins: Vec<CallableBindingOriginEntryV1>,
});
```

This is not close to complete parity.

---

# 5. Field-by-field callable-analysis coverage matrix

Legend:

- **W**: represented in current wire.
- **P**: partially represented.
- **—**: absent.

| Owner field | Wire | Root status | Current import | Compiler/query impact |
|---|---:|---|---|---|
| `callable` | W | n/a | owner helper exists | Fundamental owner identity |
| `body_range` | W | n/a | not assembled into a full owner | Used for body/source attachment classification |
| `body_source_owner` | W | n/a | no full owner import | Required for `<main>`/source namespace |
| `expressions` | **P** | **Partial** | no full rows | **Critical**: lowering consumes range, binding read/write, safety, specialization, dispatch/contextual facts |
| `bindings` | **P** | **Partial** | no rows | **Critical**: lowering binding identity and attachments |
| `binding_origins` | W | n/a | no full owner assembly | **Critical**: body-local vs module storage |
| `for_iteration_lanes` | — | roots added | none | Compiler lowering consumes |
| `positional_expansions` | — | roots added | none | Compiler lowering consumes |
| `known_invalid_positional_expansions` | — | roots added | none | Compiler lowering consumes |
| `sequence_literals` | — | roots added | none | Compiler lowering consumes |
| `associated_resolutions` | — | roots added | none | Compiler lowering consumes |
| `callable_reference_resolutions` | — | roots added | none | Compiler lowering consumes; nested capabilities matter |
| `family_applications` | — | roots added | none | Compiler lowering consumes |
| `match_resolutions` | — | **No root traversal** | none | Compiler lowering consumes exact pattern/binding plans |
| `pattern_site_resolutions` | — | **No root traversal** | none | Compiler lowering consumes |
| `flow_graph` | W | no type roots needed | dedicated frozen importer exists | Query/fingerprint/control-flow parity |
| `entry_flow` | — | roots added | none | Fingerprint/semantic parity |
| `exits` | **P** (`unreachable` only) | return/throw roots added | none | Fingerprint/return semantics |
| `return_validation` | — | n/a | none | Included in product fingerprint |
| `diagnostics` | — | incomplete census | none | Accepted owner amendment requires preserved judgments |
| `internal_incidents` | omitted-by-policy | n/a | canonical image rejects | Correct fail-closed policy |
| `explanations` | — | **No root traversal** | none | Diagnostics/causal parity |
| `return_explanation` | — | **No root traversal** | none | Explanation closure |
| `dependencies` | — | n/a | none | Query dependency reconstruction |
| `semantic_dependencies` | — | n/a | none | Query invalidation reconstruction |
| `constructor_super_initializer` | W | n/a | not attached to full owner | Compiler lowering consumes |
| `dependency_fingerprint` | Derived | n/a | not recomputed because owner import absent | Must be recomputed after relocation |
| `status` | W | n/a | no full owner | Canonical publication requires `Complete` |

The current field manifest is therefore an **intended contract**, not evidence that the wire has implemented that contract.

---

# 6. `ExpressionAnalysis` is the real executable subgraph

The live `ExpressionAnalysis` contains:

```rust
pub struct ExpressionAnalysis {
    pub id: ExpressionId,
    pub range: SourceRange,
    pub knowledge: TypeKnowledge,
    pub callable: Option<CallableId>,
    pub call_specialization: Option<CallSpecialization>,
    pub conditional_dispatch: Option<ConditionalDispatchSelection>,
    pub conditional_capability: Option<ConditionalSelectedCallCapability>,
    pub trait_dispatch: Option<TraitDispatchSite>,
    pub trait_dispatch_candidates: Option<Box<[TraitEvidencedMemberCandidate]>>,
    pub contextual_realization: Option<Arc<ContextualRealization>>,
    pub denotation: Option<SemanticDenotation>,
    pub status: AnalysisStatus,
    pub causal_invalidity: CausalInvalidity,
    pub explanation: Option<ExplanationId>,
    pub call: Option<CallResolutionId>,
    pub binding_read_safety: Option<BindingReadSafety>,
    pub binding_read: Option<BindingId>,
    pub binding_write: Option<BindingId>,
}
```

Current wire transports only the *count* of expressions, optional denotation, and the static-argument subset of optional call specialization. It does **not** transport an expression row carrying the range, knowledge, selected callable, status, causal state, binding edges, conditional/trait dispatch, contextual realization, explanation, or binding safety.

## Finding S2-01 — CRITICAL: `expression_count` is not an expression owner graph

The future artifact needs one explicit record per expression ordinal. Sparse side tables can remain useful only after the expression row itself exists.

Recommended conceptual shape:

```text
CallableExpressionV2
  ordinal
  range
  knowledge
  callable?
  call_specialization?
  conditional_dispatch?
  conditional_capability?
  trait_dispatch?
  trait_dispatch_candidates[]
  contextual_realization?
  denotation?
  status
  causal_invalidity
  explanation_ordinal?
  binding_read_safety?
  binding_read_ordinal?
  binding_write_ordinal?
```

`CallResolutionId` should remain fail-closed while it has no canonical artifact owner, matching the accepted P7 amendment.

---

# 7. Root-closure audit

`register_callable_analysis_roots` currently registers roots for:

- expression type knowledge;
- supported expression denotations;
- static call arguments;
- binding contracts/current knowledge/denotations;
- entry-flow binding/field knowledge;
- normal-return and throw-flow knowledge;
- known-invalid positional expansion source types;
- sequence result/element/extent types;
- iteration and positional-expansion products;
- associated resolutions;
- callable-reference resolutions;
- family applications.

That is meaningful progress.

However there are no live root traversal branches for several still-required fields, including:

- `match_resolutions`;
- `pattern_site_resolutions`;
- expression `conditional_dispatch`;
- expression `conditional_capability`;
- expression `trait_dispatch`;
- `trait_dispatch_candidates`;
- `contextual_realization`;
- diagnostics/explanations where evidence payloads own type-bearing products;
- the complete generic-conformance payload of call specializations.

The existing root tests cover only the already-added families and explicitly prove that `StaticApplication` remains fail-closed.

## Finding S2-02 — CRITICAL: DTO completion cannot precede root completion

Before extending the callable record, perform a **field-exhaustive root census**. Every portable field that carries `TypeId`, `KindId`, `RecordRowId`, `TypeParameterId`, static-domain IDs, generic conformance evidence, callable signatures, or type-bearing resolution payloads must register roots before `TypeExportContext::close_graph`.

No later row projection should be permitted to discover a new type root.

---

# 8. Full importer does not exist

`ensure_complete_callable_analysis_payload` currently rejects every nonempty list:

```rust
fn ensure_complete_callable_analysis_payload(
    analyses: &[CallableAnalysisRecordV1]
) -> Result<(), ArtifactError> {
    if analyses.is_empty() {
        return Ok(());
    }
    Err(referential(
        "callable analysis body facts are not completely represented; refusing semantic image import",
    ))
}
```

`validate_and_import_semantic_world_inner` invokes this before owner materialization.

The importer contains isolated helpers for frozen flow graphs, supported denotations, typed call static arguments, callable signatures, and other semantic families. It does **not** reconstruct complete `CallableAnalysis` values.

## Finding S2-03 — CRITICAL: no importer may be enabled through default-filled rows

The future importer must build each callable owner transactionally, validate all internal references, recompute its product fingerprint, and only then publish it into the reconstructed table.

Filling absent fields with `Default`, empty maps, `Unknown`, or synthesized resolutions would publish false compiler authority and violate the accepted owner-graph ruling.

---

# 9. Incremental fingerprint correctness audit

This slice found serious gaps in:

`phalcom-semantic/src/db/fingerprint.rs::callable_body_product_fingerprint`.

This function is especially important for P7 because the accepted plan requires the importer to relocate the complete owner and recompute the callable product fingerprint before rebuilding query products.

## 9.1 `binding_write` is missing

The expression fingerprint hashes:

```rust
expression.binding_read_safety.hash(&mut hasher);
expression.binding_read.hash(&mut hasher);
```

but does not hash:

```rust
expression.binding_write
```

Core lowering consumes `binding_write` when projecting exact assignment targets.

### Finding S2-04 — HIGH

Two callable analyses that differ only in assignment target can currently produce the same callable body product fingerprint.

Required discriminator:

```text
same callable
same other semantic products
binding_write = A  vs  binding_write = B
→ product fingerprints must differ
```

This should be fixed independently of artifact transport.

## 9.2 `family_applications` is entirely absent

There is no `family_applications` reference in `db/fingerprint.rs`, yet core lowering consumes `analysis.family_applications` to freeze first-class family invocation semantics.

### Finding S2-05 — HIGH

A family application selection can change without changing `callable_body_product_fingerprint`, unless an unrelated field changes coincidentally.

Hash every relevant field of both static and dynamic family application selections.

## 9.3 Callable-reference fingerprinting is materially incomplete

For:

```rust
CallableReferenceResolutionKind::BoundFamily {
    family_type,
    spec,
    members,
}
```

current fingerprinting hashes member details but not `family_type` or the selector `spec`.

For:

```rust
CallableReferenceResolutionKind::Associated(Box<AssociatedResolution>)
```

it hashes only a variant tag rather than the nested selection.

### Finding S2-06 — HIGH

The callable product fingerprint is not an exhaustive semantic identity for callable-reference lowering.

## 9.4 Associated-resolution hashing covers only one special case

The function explicitly hashes only `DataExpansionField` entries from `associated_resolutions`.

The compiler consumes many other associated outcomes: exact values, exact callables, static invokes, family selections, dynamic invokes, etc.

### Finding S2-07 — HIGH

Introduce one canonical `hash_associated_resolution` helper that covers every `AssociatedResolutionKind` and all owner/family/target/type fields. Reuse it for nested callable-reference associated selections.

## 9.5 Existing tests do not detect these omissions

Focused fingerprint tests cover source/frozen flow graph equivalence, return validation variants, Dynamic-vs-concrete static call arguments, and flow field validity. They do not cover the missing executable families above.

---

# 10. Generic call specialization transport remains incomplete

`CallSpecialization` owns:

```rust
pub struct CallSpecialization {
    pub callable: CallableId,
    pub bindings: Box<[(TypeParameterId, StaticArgument)]>,
    pub conformances: Box<[GenericCallConformanceBinding]>,
}
```

Current wire stores only the selected callable and static-argument bindings.

Export explicitly rejects nonempty `specialization.conformances`. Import reconstructs:

```rust
CallSpecialization {
    callable,
    bindings: ...,
    conformances: Box::default(),
}
```

This is safe only because export refuses source rows that need conformance evidence. It is not complete.

## Finding S2-15 — HIGH

Generic conformance provenance is compiler authority and is consumed by `compile.rs` when constructing runtime conformance slots/environments.

Portable representation must retain every `GenericCallConformanceSource` variant:

- Exact;
- Forwarded;
- Derived;
- CompilerDerived;

with all stable owners, fingerprints, antecedent/provenance data, and type roots required by each variant.

---

# 11. Allocation accounting: the next architectural blocker

The flow-graph slice already hit the core problem: native `BTreeMap` node allocations cannot be exactly charged through the artifact transaction. The adopted solution was dual backing: mutable source maps vs immutable imported `BudgetedSortedMap` storage.

The same problem now exists across the rest of `CallableAnalysis`.

## 11.1 Concrete map-backed fields

Examples:

```rust
pub type ExpressionAnalysisIndex = BTreeMap<ExpressionId, ExpressionAnalysis>;
pub type BindingAnalysisIndex = BTreeMap<BindingId, BindingState>;
```

Many resolution indexes are also `BTreeMap`s.

`ExplanationArena` owns:

```rust
nodes: BTreeMap<ExplanationId, ExplanationNode>
```

`CallableAnalysisTable` owns:

```rust
by_id: HashMap<CallableId, Arc<CallableAnalysis>>,
by_module: HashMap<ModuleId, BTreeSet<CallableId>>,
```

## Finding S2-08 — HIGH

A complete importer cannot remove `UnaccountedBoundary` while constructing these owners through ordinary native map insertion and still claim exact shared-ledger accounting.

### Recommended direction

Follow the successful `FlowGraph` precedent:

- preserve mutable source construction where mutation is needed;
- add immutable imported/frozen backing based on dense sorted arrays / `BudgetedSortedMap`;
- expose one read API across both forms;
- prevent hidden thaw/rebuild;
- make clone of frozen storage share allocation;
- construct module indexes with budgeted sorted structures.

Do not pre-charge guessed `BTreeMap` node sizes. That would not be allocator-exact and is not portable across standard-library/platform implementations.

## 11.2 `CallableAnalysisTable::Deref<HashMap>` is representation debt

The table currently implements:

```rust
impl Deref for CallableAnalysisTable {
    type Target = HashMap<CallableId, Arc<CallableAnalysis>>;
}
```

### Finding S2-09 — HIGH

Before introducing a frozen imported table, inventory all implicit `HashMap` consumers and migrate them to explicit read methods:

- `get`;
- `contains_key`;
- `iter`;
- `keys`;
- `values`;
- `iter_module`;
- `len`;
- `is_empty`.

Otherwise a dual backing will require either emulating `HashMap` or retaining an unbudgeted shadow map.

## 11.3 Owner-local indexes have the same issue

Changing only the outer table is insufficient. Expressions, bindings, match/pattern indexes, associated indexes, and explanation nodes still allocate native trees.

### Finding S2-10 — HIGH

Treat callable owner storage as a coordinated frozen-representation problem, not as a collection of independent decoder helpers.

---

# 12. Transactional allocation bug in partial call-specialization import

`import_callable_flow_graph_with_allocation_budget` correctly uses a child transaction and commits only after success.

`import_callable_denotations` does the same.

`import_callable_call_specializations`, however, allocates directly from the caller's budget. A later invalid record, duplicate relocated parameter, or relocation error can therefore return `Err` after earlier allocation requests have already been charged.

## Finding S2-11 — HIGH

Wrap the whole helper in a child transaction and commit only on success.

Add tests for:

- exact-fit success;
- N-1 failure;
- malformed-later-row rollback;
- duplicate relocated parameter rollback.

The current test proves successful relocation only; it does not prove rollback.

---

# 13. Wire validation gap: `expression_count` is unbounded

`SemanticWorldV2::validate_with_allocation_budget` bounds flow nodes/edges, formal bindings, origins, denotation side tables, call-specialization count, and per-call binding count. It validates expression ordinals against `expression_count`.

It does **not** validate:

```text
expression_count <= limits.max_semantic_entries_per_family
```

Today the full importer is blocked, so this cannot yet trigger a giant owner allocation. Once real expression rows/materialization exist, it becomes an input-amplification hazard.

## Finding S2-12 — HIGH

Add the bound now, before extending the record.

Future dense owner families need explicit per-owner limits for expressions, bindings, explanations, diagnostic causes, diagnostics, dependencies, semantic dependencies, match/pattern products, iteration/sequence products, candidate lists, and generic evidence.

---

# 14. Source-independent compiler projection remains incomplete

The accepted P7 amendment says broad `source_index` remains omitted; compiler-required binding/site/target attachments must instead come from validated callable owner facts and reconstructed formal projection.

Current lowering still has source-backed assumptions.

## 14.1 Missing formal attachment can be misclassified as implicit receiver

For body-local formal bindings, lowering effectively does:

```rust
let Some(range) = snapshot.formal_projection
    .binding_attachment_range(callable, binding.binding)
else {
    if binding.parameter.as_ref().is_some_and(|parameter|
        !snapshot.source_index.has_parameter_source_site(&source_callable, parameter)
    ) {
        // implicit receiver
        continue;
    }
    return Err(MissingFormalBindingAttachment { ... });
};
```

An artifact-loaded P7 snapshot intentionally has an empty broad `source_index`.

Therefore a missing explicit parameter attachment can satisfy the negative source-index condition and be skipped as if it were an implicit receiver.

## Finding S2-13 — HIGH

The imported owner needs an explicit attachment classification. Lowering must not use absence from an omitted source index as semantic evidence.

Required invariant:

```text
BodyLocal + source-declared parameter:
    formal attachment REQUIRED

Implicit receiver / generated body-local binding:
    owner record explicitly marks attachment absence as valid

ModuleValue:
    no callable-local declaration attachment
```

## 14.2 Other source AST reads remain outside this slice

`semantic_lowering.rs` still reads `snapshot.sources` for accepted-definition details such as generated inherent delegation and trait-default source-member lookup.

That means completing `CallableAnalysis` is necessary but not sufficient for P7 T12. These accepted-definition/source dependencies should be a primary Slice 3 target.

---

# 15. Binding rows are absent, not merely under-specified

`BindingState` contains:

```text
binding
name
parameter
range
contract
current
denotation
consistency
causal_invalidity
mutable
initialization
version
explanation
```

Current wire stores binding origins and optional denotations, but no actual `BindingState` row.

Many of these fields contribute to the live callable product fingerprint and flow/compiler behavior.

## Finding S2-14 — HIGH

Add a dense `CallableBindingV2` row keyed by owner-local ordinal. `binding_origins` should either be folded into that row or validated 1:1 against it.

Do not retain a parallel count-only authority.

---

# 16. `StaticApplication`: producer is corrected, authority remains checked-body-owned

Earlier P7 planning recorded an owner bug. Live code has corrected the producer.

`CheckingContext::record_checked_static_application` now obtains:

```rust
let body = self.current_static_body_owner.clone()?;
let id = StaticApplicationId { body, ordinal };
```

and `analyze_static_body_inner` sets:

```rust
context.current_static_body_owner =
    Some(StaticBodyId(definition.id.clone()));
```

This is the correct owner relation.

However ordinary `CallableAnalysis::finalize_with_normal_returns` does **not** retain `checked_static_applications`.

`CheckedStaticBody` does:

```rust
pub static_applications: Arc<[CheckedStaticApplication]>
```

and each row owns the actual `StaticBodyId`, expression, source range, target static definition, exact signature, and checked arguments.

## Finding S2-16 — HIGH

Do not make callable artifact completeness “work” by copying checked-static-application rows into `CallableAnalysis`.

Correct model:

```text
CallableAnalysis denotation
    StaticApplication { static_body_owner, ordinal }
            ↓
CheckedStaticBody artifact owner
    checked_static_applications[ordinal]
```

The checked-body transport must be complete before `SemanticDenotation::StaticApplication` is accepted.

Current export/root fail-closed behavior is correct.

---

# 17. Canonical publication status policy

The artifact DTO represents several analysis statuses, which is useful for schema completeness:

- Complete;
- Partial;
- Blocked;
- Cancelled;
- BudgetExceeded.

But the Universe compiler artifact is intended to be reusable canonical compiler authority.

## Finding S2-17 — HIGH

When the broad callable guard is removed, add an early canonical publication gate:

```text
for every required callable analysis:
    status == Complete
    internal_incidents empty
    complete owner graph present
```

Do not wait for the final imported snapshot completeness predicate to discover a noncanonical source row.

---

# 18. Deterministic local identity audit

`CallableAnalysisOrdinalMap` assigns local ordinals from sorted local IDs for expressions, bindings, flow nodes/edges, explanations, and diagnostic causes.

### Positive findings

- native IDs are not emitted directly;
- references are intended to be callable-qualified owner-local ordinals;
- expression IDs are constrained to one body owner;
- binding IDs are qualified by callable owner;
- flow closure is validated before projection;
- explanations and diagnostic causes already have preparatory closure validation;
- wire callable owners are sorted by stable callable identity.

Expression and binding IDs are allocated monotonically by `CheckingContext`.

## Finding S2-18 — MEDIUM-HIGH

Determinism is **plausible but not proven**.

Required discriminator:

1. build the same callable semantics under perturbed unrelated arena/allocation history;
2. make native body/type/store IDs differ where possible;
3. export both;
4. assert identical callable wire bytes;
5. import into differently numbered fresh stores;
6. assert equivalent compiler projection and semantic fingerprint meaning.

Do not infer determinism from BTree ordering alone.

---

# 19. Relocation and fingerprint stability require direct proof

The accepted P7 plan requires recomputing `callable_body_product_fingerprint` after relocation.

The current fingerprint hashes many process-local IDs directly through their `Hash` implementations. Fresh-store type numbering may differ across processes.

This is not yet proven to be a defect because the intended query-fingerprint reconstruction contract may compare relocated-process values rather than raw source-process values. However it is a **high-priority verification risk**.

Required test:

```text
source callable analysis
  exported under store numbering A
  imported under deliberately different numbering B
  recomputed callable fingerprint/query evidence
```

Verify exactly which fingerprints are required to be stable across processes and which are process-local recomputations derived from stable wire authority.

If a portable query fingerprint must remain equal, its algorithm must be based on canonical semantic identity rather than incidental native indices.

---

# 20. Testing audit

## 20.1 What current tests genuinely prove

### Artifact codec

`phalcom-compiler-artifact/tests/semantic_world_callable_analysis_test.rs` proves selected synthetic behavior including:

- status-tag round trip;
- current binding-origin codec;
- constructor-super range retention;
- flow graph node/edge/entry/exit codec;
- malformed graph/body range rejection;
- selected typed-call-argument validation.

### Semantic root tests

`artifact/roots.rs` has focused tests for supported root families and explicitly verifies `StaticApplication` remains blocked.

### Import helper tests

Current tests cover isolated flow-graph, denotation, static-call-argument, and other helper behavior, including exact-fit rollback for some families.

## 20.2 What is not tested

There is no test that constructs one real complete `CallableAnalysis` and proves:

```text
live source owner
→ full root registration
→ full wire record
→ encode/decode
→ fresh-store relocation
→ complete CallableAnalysis
→ owner graph closure
→ formal projection
→ callable product fingerprint parity/defined reconstruction
→ core lowering parity
```

There is also no current discriminator for:

- `binding_write` fingerprint sensitivity;
- family-application fingerprint sensitivity;
- associated-resolution fingerprint sensitivity;
- callable-reference family type/spec sensitivity;
- full generic conformance call specialization;
- match/pattern wire parity;
- contextual-realization wire parity;
- diagnostics + explanation DAG sharing;
- two owners with overlapping local numeric IDs;
- formal-binding completeness with empty broad `source_index`;
- allocator-exact complete callable owner materialization;
- imported `CallableAnalysisTable` module indexing;
- source-denied compilation consuming imported callable attachments.

The current end-to-end core artifact test still fails before artifact construction at the callable-analysis export guard, so it provides no downstream proof.

---

# 21. Required callable-analysis wire architecture

A viable next schema should be owner-centric rather than count/side-table-centric.

Suggested conceptual form:

```text
CallableAnalysisRecordV2
  stable callable owner
  body source owner
  body range
  status

  expressions[]
  bindings[]
  flow graph

  for iteration lanes[]
  positional expansions[]
  invalid positional expansions[]
  sequence literals[]

  associated resolutions[]
  callable reference resolutions[]
  family applications[]
  match resolutions[]
  pattern site resolutions[]

  entry flow
  normal exits[]
  throw exits[]
  unreachable

  return validation

  diagnostic causes[]
  diagnostics[]
  explanations[]
  return explanation?

  callable dependencies[]
  semantic dependencies[]

  constructor super initializer?
```

Every nested handle should be an **owner-local ordinal qualified by the stable callable owner**.

Do not encode native:

- `BindingId`;
- `ExpressionId`;
- `ExplanationId`;
- `DiagnosticCauseId`;
- `FlowNodeId`;
- `FlowEdgeId`;
- `TypeStoreId`;
- native arena ordinals.

---

# 22. Recommended implementation sequence

## Phase A — repair semantic fingerprint completeness first

Before expanding artifact transport:

1. Hash `binding_write`.
2. Add full `family_applications` hashing.
3. Add exhaustive `AssociatedResolution` hashing.
4. Add exhaustive `CallableReferenceResolution` hashing.
5. Verify conditional/trait/contextual fields are complete.
6. Add one discriminator test per executable family.
7. Define and test relocation-stable fingerprint expectations.

This prevents P7 from persisting an existing incremental-cache defect.

## Phase B — finish the root census

Add root traversal for remaining type-bearing products:

- match/pattern;
- conditional dispatch/capability;
- trait dispatch/candidates;
- contextual realization;
- generic conformance call bindings;
- diagnostic/explanation evidence where type-bearing;
- checked-body reference products needed by `StaticApplication`.

Add an exhaustive regression so a new type-bearing callable field cannot silently bypass root registration.

## Phase C — settle frozen callable-owner storage

Before writing the complete importer:

1. inventory all concrete map/set fields;
2. inventory `CallableAnalysisTable::Deref` users;
3. migrate consumers to explicit read APIs;
4. adopt frozen budgeted backing for imported owner-local maps/indexes;
5. make imported outer table/module indexes budget-aware;
6. make explanation storage support frozen import;
7. preserve mutable source construction semantics.

This is the allocation prerequisite for removing semantic `UnaccountedBoundary` for this family.

## Phase D — implement dense owner rows

Recommended dependency order:

1. binding rows + origin + formal attachment;
2. expression rows;
3. flow state / exits / return validation;
4. generic call specialization evidence;
5. associated/reference/family products;
6. match/pattern products;
7. iteration/sequence products;
8. conditional/trait/contextual products;
9. diagnostic causes + diagnostics + explanation DAG;
10. dependencies;
11. constructor-super relation.

Validate owner-local closure before final owner allocation wherever possible.

## Phase E — complete checked-body / `StaticApplication` authority

Complete `CheckedStaticBody` transport and checked application rows.

Then allow `SemanticDenotation::StaticApplication` only when `(body owner, ordinal)` resolves to a validated imported checked row.

Do not duplicate checked rows into callable analysis.

## Phase F — full transactional import

For each callable:

```text
child transaction
  validate row limits
  allocate frozen owner structures
  relocate every type/static/stable reference
  validate cross-family closure
  rebuild formal binding attachments
  reconstruct CallableAnalysis
  recompute semantic product fingerprint
commit child
```

Any failure must leave the parent allocation transaction unchanged.

## Phase G — compiler-reader parity

Compare source-built and imported projections for at least:

- binding identities;
- read/write attachments;
- read safety;
- associated lowerings;
- callable references;
- family applications;
- match/pattern plans;
- conditional invocation specs;
- trait invocation specs;
- iteration specs;
- positional expansions;
- sequence literal plans;
- contextual realizations;
- constructor-super initializers;
- runtime call environment/conformance recipes.

No dependency syntax lookup may be required.

## Phase H — remove guards only after discriminators pass

Only then remove:

- the callable-analysis branch of `validate_semantic_world_export_coverage`;
- `ensure_complete_callable_analysis_payload`.

The global semantic-allocation `UnaccountedBoundary` may still remain until other semantic owner families are budget-closed.

---

# 23. Required test matrix before callable-analysis guard removal

| Gate | Required proof |
|---|---|
| Full field parity | Every `CallableAnalysis` owner field survives modulo relocation |
| Expression parity | Range, knowledge, status, target, read/write, safety, dispatch, denotation |
| Binding parity | Contract/current/consistency/causal/mutability/init/version/explanation/origin |
| Generic calls | Static args + every conformance-source variant |
| Owner isolation | Two callables with same numeric binding/expression/explanation IDs do not alias |
| Match/pattern | Binding identities and executable pattern plan survive |
| Dispatch | Conditional + trait selections/candidates survive without lookup |
| Contextual realization | Exact algebra operation/capability survives |
| Explanations | DAG sharing and parent/evidence closure survive |
| Diagnostics | Root-cause/explanation references survive |
| Static application | Checked-body-qualified row resolution; overlapping ordinals under two bodies |
| Fingerprint | Defined source/import semantic product parity under fresh numbering |
| Determinism | Same semantics under perturbed native allocation → identical wire |
| Allocation | Exact fit + N-1 rollback for whole callable owner |
| Formal reader | Imported formal projection works with broad `source_index` absent |
| Compiler projection | Source/import lowering output equivalent |
| Source denied | Fresh-process compilation succeeds with Universe source inaccessible |

---

# 24. Performance and memory requirements

Completing this owner graph will move substantially more compiler state into `SEMANTIC_WORLD`.

Measure:

- total callable-analysis bytes;
- bytes per callable;
- expression rows per callable;
- binding rows per callable;
- flow nodes/edges;
- diagnostic/explanation bytes;
- generic-evidence bytes;
- peak load transaction bytes;
- validation scratch bytes;
- final frozen retained memory;
- repeated signature/evidence payload rate.

Recommended rules:

1. Prefer dense owner-local vectors on wire.
2. Do not persist derived indexes separately when O(N) rebuild is sufficient.
3. Avoid duplicating signatures already owned canonically unless a nested product genuinely owns its own signature snapshot.
4. Preserve checked selections rather than rerunning dispatch.
5. Build frozen lookup indexes once.
6. Do not materialize BTree/Hash shadow copies merely to preserve legacy APIs.
7. Add section-size telemetry so future language features cannot silently explode `universe.telosc`.

---

# 25. Implementer warnings

- **Do not remove the callable export guard because the existing record “encodes enough to test”.** It does not.
- **Do not remove the import guard and fill missing fields with defaults.** That would publish false compiler authority.
- **Do not make `StaticApplication` point directly to its target declaration.** Its identity is `(checked static body owner, ordinal)`.
- **Do not reconstruct dispatch/family/trait selections from names.** The checked selection is the authority.
- **Do not use source range as owner identity.** Ranges are attachment coordinates only.
- **Do not depend on broad `source_index` in an artifact-loaded snapshot.** P7 intentionally omits it.
- **Do not reconstruct imported owner maps through ordinary `BTreeMap`/`HashMap` and then claim exact load-ledger closure.** That recreates the allocation problem already solved for `FlowGraph`.
- **Do not certify incremental parity until `callable_body_product_fingerprint` covers every executable semantic family.** The current function demonstrably does not.

---

# 26. Revised P7 critical path after Slice 2

```text
1. Repair callable product fingerprint completeness
        ↓
2. Complete CallableAnalysis type-root census
        ↓
3. Establish budgeted/frozen callable owner storage
        ↓
4. Implement complete callable owner wire V2
        ↓
5. Implement complete transactional callable import
        ↓
6. Complete CheckedStaticBody / StaticApplication reference authority
        ↓
7. Rebuild source-independent formal/compiler reader projection
        ↓
8. Prove source/import callable semantic + lowering parity
        ↓
9. Remove callable export/import fail-closed guards
        ↓
10. Continue remaining semantic-owner allocation closure
        ↓
11. Build universe.telosc end-to-end
        ↓
12. Fresh-process, source-denied package compilation
        ↓
13. parse / similarity / slotmap standalone-package acceptance
        ↓
14. determinism + P5/P6 + performance certification
```

---

# 27. What Slice 2 changes from Slice 1

Slice 1 identified callable analysis as the principal semantic blocker.

Slice 2 refines that conclusion in three important ways:

1. **The blocker is not just schema completeness.** Current compiler code directly consumes the omitted owner fields.
2. **There is an incremental correctness defect in the live source model.** `callable_body_product_fingerprint` is not exhaustive for compiler meaning.
3. **There is an allocation-architecture prerequisite.** Exact-budget import cannot be completed using the current concrete native map/set storage without a frozen representation seam similar to `FlowGraph`.

Accordingly, the implementer should **not** jump directly into adding dozens of DTO fields. The correct first work unit is fingerprint repair + root census + storage-seam design, followed by the owner DTO/import in dependency order.

---

# 28. Proposed Slice 3

The next audit slice should leave `CallableAnalysis` and follow the reconstructed snapshot outward into **compiler-reader/source-denial closure**:

- `EffectiveCallableDefinition` / accepted definition transport;
- static definitions/constants/case catalogs/module static bindings;
- trait-default/inherent source dependence in `semantic_lowering`;
- reconstructed formal projection and P6 module-local indexes;
- query-image owner completeness and dependency evidence;
- exact fresh-process standalone-package compile path.

The core question for Slice 3 should be:

> After `CallableAnalysis` is fully portable, what remaining source-backed/compiler-owner dependencies would still prevent a loaded `.telosc` image from compiling a real standalone package without Universe source?

That is the next highest-value audit boundary.

---

## Final Slice 2 verdict

**P7 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`.**

The current callable-analysis fail-closed guards are necessary. The implementation behind them is not ready to publish.

The immediate engineering priority is:

> **Repair callable semantic fingerprint completeness, finish the type-root census, and establish a budgeted frozen storage model for the complete callable owner graph before implementing/removing the final wire/import guards.**

This sequence minimizes rework and prevents two dangerous failure modes: a source-free image that silently loses compiler authority, and an incremental artifact/cache path whose fingerprints fail to change when executable semantics change.
