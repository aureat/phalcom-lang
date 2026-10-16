---
id: PERF003.C2.P7.PATCH1
category: PERF
program: PERF003
checkpoint: PERF003.C2
parent_plan: PERF003.C2.P7
kind: corrective-implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on:
  - PERF003.C2.P7.PATCH0
follows: PERF003.C2.P7.PATCH0
supersedes: null
target_branch: perf/universe-characterization
prepared_against_revision: dbde1d8cacba40067223ca7230b79580bff9c164
audit_baseline_revision: 2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a
executor_profile: GPT-5.6 Luna High / Extra High
---

# PERF003.C2.P7.PATCH1 — Complete the Canonical `CallableAnalysis` Owner Graph

> **Execution role:** Luna-oriented patch-grade implementation plan.  
> **Parent acceptance objective:** `PERF003.C2.P7` — persisted semantic compiler image (`universe.telosc`).  
> **Patch objective:** make `CallableAnalysis` a complete, owner-authoritative, relocatable, immutable artifact product, with exact consumer parity and a budget-compatible import representation.  
> **Lifecycle note:** this is corrective work inside the existing `PERF003.C2` checkpoint and normalized `PERF003.C2.P7` plan. It is not a new checkpoint and does not authorize P8 takeover.

> **PATCH0 dependency status — 2026-10-15:** `PATCH0_callable_fingerprint` is
> **SATISFIED** by implementation commit
> `8284893769d121dc2fe33ed0c866a402ff48bd35` and its focused evidence. PATCH1
> remains PROPOSED / NOT_STARTED / UNVERIFIED; only the fingerprint-dependent
> implementation gate is unblocked. P7 remains partial/unverified and P8 remains
> blocked.

---

## 0. Executor contract

This plan is designed for a capable implementation model working under constrained architectural authority.

The implementer:

- **may** adapt mechanical details to the live repository;
- **must** preserve the architecture and invariants below;
- **must not** improvise material ownership, identity, allocation, artifact-versioning, or compiler-authority semantics;
- **must** follow the verification budget rather than running broad tests reflexively;
- **must** classify and defer unrelated failures;
- **must STOP AND CONSULT** when any escalation trigger fires;
- **must** keep the shared `PERF003.C2` checkpoint record current;
- **must** produce the required walkthrough and handoff;
- **must not** remove the global semantic allocation guards in this patch;
- **must not** weaken callable completeness merely to reach downstream artifact code.

Testing is evidence gathering, not a ritual. Run the smallest command that answers the current correctness question.

This plan assumes `PERF003.C2.P7.PATCH0` has established the executable artifact contract and corrected the callable-product fingerprint defects assigned to PATCH.0. If that prerequisite is not actually implemented in the live tree, do not silently absorb PATCH.0 into this patch. Follow §12.3 and §13.

---

# 1. Goal

Implement a complete owner-centric `CallableAnalysis` artifact contract from source semantic owner through stable wire, validation, fresh-store relocation, immutable native reconstruction, formal/compiler projection, and callable-specific guard removal, without fabricating semantic authority or violating the P7 exact-allocation architecture.

---

# 2. Checkpoint acceptance objective

`PERF003.C2` exists to eliminate the Universe-dominated compilation floor by making canonical Universe compiler state immutable, reusable, and cheap to attach to user overlays.

`PERF003.C2.P7` narrows that objective to a persisted compiler image. This patch owns one of P7's principal blockers: callable-body semantic authority.

A successful PATCH1 does **not** close P7. It advances P7 from:

```text
source CallableAnalysis
    ↓
partial roots
    ↓
partial sparse wire record
    ↓
full import deliberately rejected
```

to:

```text
source CallableAnalysis
    ↓
complete root closure
    ↓
one owner-centric portable record
    ↓
closed owner-local validation
    ↓
fresh type/static relocation
    ↓
budget-compatible frozen callable owner
    ↓
recomputed callable product fingerprint
    ↓
formal/compiler reader parity
```

PATCH1 is **corrective semantic closure plus allocation-compatible representation hardening**. It must leave these later responsibilities open:

- other semantic owner families — PATCH2;
- global semantic/native load allocation closure — PATCH3;
- query image closure — PATCH4;
- `.telosc` canonical byte hardening — PATCH5;
- full canonical-image assembly — PATCH6;
- P7 package/performance certification — PATCH7.

The callable-specific export/import coverage guards may be removed at the end of this plan **only** if every PATCH1 discriminator passes. The global `UnaccountedBoundary` guards remain until PATCH3.

---

# 3. Repository grounding

## 3.1 Prepared repository state

Prepared against:

```text
repository: aureat/phalcom-lang
branch: perf/universe-characterization
live branch head verified during planning:
    21a54705c33dae95ee9c77d86789b4911635e6b0
    "fix(perf): close callable analysis type roots"

six-audit pinned baseline:
    2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a
    "feat(perf): extend artifact loading foundations"
```

The live branch is one commit ahead of the audit baseline. That commit materially extends `register_callable_analysis_roots`, updates P7 lifecycle records, and does **not** add the missing complete callable wire/materialization path.

The live checkpoint at this planning baseline still states:

```text
PERF003.C2.P7
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
```

and records that:

- full callable-analysis wire/materialization remains missing;
- callable export/import remain fail-closed;
- `FlowGraph` already has dual source/frozen budgeted storage;
- callable-analysis root registration has been substantially extended;
- `CheckedStaticBody`, not `CallableAnalysis`, owns checked static-application rows;
- nonempty generic call conformance slots still fail closed;
- no complete artifact, source-denied package acceptance, determinism gate, or P5/P6 final acceptance is established.

## 3.2 Live facts verified during planning

At `21a54705...`:

1. `CallableAnalysis` remains the canonical semantic owner and still contains:
   - expressions;
   - bindings;
   - binding origins;
   - iteration/expansion/sequence products;
   - associated/callable-reference/family products;
   - match/pattern products;
   - flow graph;
   - entry/exit/return facts;
   - diagnostics/incidents/explanations;
   - callable and semantic dependencies;
   - constructor-super selection;
   - dependency/product fingerprint;
   - analysis status.

2. `CallableAnalysisTable` currently owns:
   ```rust
   HashMap<CallableId, Arc<CallableAnalysis>>
   HashMap<ModuleId, BTreeSet<CallableId>>
   ```
   and still implements `Deref<Target = HashMap<...>>`.

3. `ExpressionAnalysisIndex`, `BindingAnalysisIndex`, `CallableBindingOriginIndex`, many resolution indexes, `FlowStateSummary` maps, sequence expansion maps, and `ExplanationArena` remain native tree/map-backed structures.

4. `FlowGraph` is already the approved precedent:
   ```text
   Source(BTreeMap-backed mutable graph)
   Frozen(BudgetedSortedMap-backed immutable graph)
   ```
   with one allocation-free read interface.

5. `CallableAnalysisRecordV1` remains a sparse partial record containing:
   - callable/body owner/range/status;
   - `exits_unreachable`;
   - flow graph;
   - constructor-super target;
   - expression count;
   - sparse expression/binding denotations;
   - partial call specialization static arguments;
   - binding source ranges;
   - binding origins.

6. `export_callable_analysis_record` still rejects:
   ```text
   nonempty specialization.conformances
   ```
   and still exports formal attachments as range-only rows.

7. `FormalSemanticProjection` already has a source-free builder:
   ```rust
   from_callable_analyses_with_binding_ranges(...)
   ```
   but it materializes only ranges, while the source attachment model owns exact `SourceSiteId` identities.

8. Core lowering still consumes:
   - owner-qualified binding identity;
   - binding origins;
   - formal binding attachment range;
   - expression binding reads/writes;
   - match and other checked resolution products.

9. `register_callable_analysis_roots` has been broadened at the live head. Treat its current coverage as inherited work to verify against the final wire schema, not as a reason to skip a final field-exhaustive census.

## 3.3 Mandatory live-tree recheck

Before editing:

> Re-read every named live path below. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

Do not assume the branch remains at `21a54705...`.

---

# 4. Required reads before implementation

Read in this order. Do not broadly explore unrelated subsystems unless evidence requires it.

## 4.1 Repository/lifecycle

```text
AGENTS.md
docs/implementation/README.md
docs/implementation/PERF003-runtime-performance/
  C2-compilation-performance-and-efficiency/
    CHECKPOINT.md
```

Then the live P7 records:

```text
docs/implementation/PERF003-runtime-performance/
  C2-compilation-performance-and-efficiency/P7/
    PERF003.C2.P7-telosc-persisted-semantic-compiler-image-detailed-implementation-plan.md
    PERF003.C2.P7-remaining-execution-plan.md
    PERF003.C2.P7-walkthrough.md
    PERF003.C2.P7-handoff.md
```

Read the current audit reports under the live P7-R1/reports directory if they remain there.

## 4.2 This patch's planning authorities

Read the supplied/accepted:

```text
PERF003.C2.P7 Consolidated Patch Program — Part I:
  Architecture, Contracts, and Callable Authority

Luna-Oriented Patch-Grade Implementation Plan Schema

Phalcom Implementation Record & Work-Lifecycle Convention

Phalcom Principal Architect, Specifier & Implementation Planner Doctrine
```

## 4.3 Source paths

Read before coding:

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/checker/flow/graph.rs
phalcom-semantic/src/explain/arena.rs
phalcom-semantic/src/source_index/mod.rs
phalcom-semantic/src/presentation.rs

phalcom-semantic/src/artifact/manifest.rs
phalcom-semantic/src/artifact/roots.rs
phalcom-semantic/src/artifact/export.rs
phalcom-semantic/src/artifact/import.rs
phalcom-semantic/src/artifact/operational.rs

phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/query.rs

phalcom-compiler-artifact/src/semantic/world.rs
phalcom-compiler-artifact/src/limits.rs
phalcom-compiler-artifact/src/error.rs

phalcom-type-meta/src/allocation.rs
# plus the exact live file owning BudgetedSortedMap / charged sorted builders

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/compile.rs
```

## 4.4 Tests/conventions

Read the current test layout and existing focused tests:

```text
phalcom-compiler-artifact/tests/semantic_world_callable_analysis_test.rs

phalcom-semantic/src/artifact/roots.rs          # local root tests
phalcom-semantic/src/artifact/export.rs         # local export tests if present
phalcom-semantic/src/artifact/import.rs         # local import/allocation tests
phalcom-semantic/tests/semantic/integration/source_index.rs
phalcom-semantic/tests/explanation_derivation.rs

phalcom-core/tests/core/modules/canonical_artifact.rs
# plus the exact semantic-lowering focused tests discovered from the live tree
```

Do not invent a new testing hierarchy if the live repository already has an established nearby convention.

---

# 5. Normative authority

This patch changes artifact transport and internal storage; it does **not** change Telos language semantics.

Authority order for this patch:

1. **Explicit user-ratified consolidated P7 patch program**
   - callable body is full compiler semantic authority;
   - artifact transport is owner-authoritative;
   - no fabricated facts;
   - one owner-local ordinal authority;
   - imported canonical structures are immutable;
   - formal attachment semantics are explicit;
   - `CheckedStaticBody` remains the `StaticApplication` row owner;
   - `dependency_fingerprint` is derived after relocation;
   - guard removal is per owner contract.

2. **Current `PERF003.C2` checkpoint and adopted P7 amendments**
   - preserve fail-closed behavior;
   - preserve one cumulative allocation architecture;
   - preserve P5/P6 immutable parent and module-locality architecture;
   - preserve the approved frozen `FlowGraph` design.

3. **Live semantic owners**
   - `CallableAnalysis` and its nested products define the source semantic facts.
   - The artifact format may project them but may not redefine them.

4. **Live compiler consumers**
   - core lowering establishes which facts must survive as executable compiler authority.

5. **Audit reports**
   - defect evidence and required discriminators;
   - not independent language-spec authority.

6. **Future proposals**
   - P8 executable image and P9 freshness/incremental cache work are non-normative for this patch.

If the live code, current checkpoint, and accepted patch program materially conflict on ownership or required behavior, STOP AND CONSULT. Do not choose whichever is easiest to implement.

---

# 6. Takeover state

## 6.1 Stable interface map

| Concept | Current symbol/path | Owner | Invariant inherited by PATCH1 |
|---|---|---|---|
| Callable body authority | `CallableAnalysis` in `checker/analysis.rs` | semantic checker | Artifact must reproduce compiler-consumed meaning; no re-analysis |
| Expression authority | `ExpressionAnalysis` | `CallableAnalysis` | Read/write/dispatch/contextual facts belong to checked expression |
| Binding authority | `BindingState` + `CallableBindingOrigin` | `CallableAnalysis` | `BindingId` is body-local; storage may be callable-local or module-owned |
| Generic call authority | `CallSpecialization` | expression analysis | Dynamic is explicit; conformance source must not be dropped |
| Flow graph | `FlowGraph` | `CallableAnalysis` | Existing source/frozen dual backing is approved precedent |
| Flow state | `FlowStateSummary` / `BodyExitFacts` | `CallableAnalysis` | Entry/return/throw facts must survive; unreachable alone is insufficient |
| Explanation graph | `ExplanationArena` | callable analysis | DAG sharing and references are semantic facts |
| Portable local identity | `CallableAnalysisOrdinalMap` in artifact export | exporter projection | One mapping pass; nested refs are owner-local |
| Root closure | `register_callable_analysis_roots` | artifact type export | Roots close before row projection |
| Current wire | `CallableAnalysisRecordV1` | compiler artifact DTO | Partial only; not a completion contract |
| Flow import | `import_callable_flow_graph_with_allocation_budget` | semantic artifact import | Approved frozen import pattern |
| Partial call import | `import_callable_call_specializations` | semantic artifact import | Incomplete and must become child-transactional |
| Formal attachment source truth | `CallableSourceAttachment` / `SourceSiteId` | source-index builder | Source range is a coordinate, not canonical site identity |
| Formal compiler reader | `FormalSemanticProjection` | semantic projection | Artifact path must work with broad `source_index` absent |
| Compiler binding identity | `LoweringBindingIdentity` / lowering helpers | core lowering | Body-local and module-value storage must remain distinct |
| Callable table | `CallableAnalysisTable` | semantic snapshot | Module-bounded iteration must remain O(module callables) |

## 6.2 Already-implemented work to preserve

Do not redo these unless the live tree disproves them:

- typed `Term` / `RecordRow` / `DynamicType` static call arguments;
- direct type roots for the current supported callable families;
- frozen/budgeted `FlowGraph`;
- owner-local flow-node/edge reconstruction;
- typed allocation error propagation for already-corrected semantic helper paths;
- source-free external module authority;
- P5/P6 parent overlay and module-indexing direction;
- `CheckedStaticBody` ownership of checked static applications.

---

# 7. Architecture

## 7.1 End-to-end authority direction

The architecture is:

```text
native semantic owner
CallableAnalysis
    │
    ├── one owner-local ordinal projection
    │
    ├── complete type/static root registration
    │
    ▼
owner-centric callable artifact record
    │
    ├── bounded structural validation
    ├── wrong-owner / duplicate / closure checks
    │
    ▼
fresh type/static relocation
    │
    ▼
budget-compatible immutable callable owner storage
    │
    ├── exact formal binding attachment contribution
    ├── post-relocation semantic product fingerprint
    │
    ▼
CallableAnalysisTable in reconstructed snapshot
    │
    ▼
FormalSemanticProjection + core lowering
```

No arrow points back into source syntax or semantic inference.

## 7.2 Owner-centric wire architecture

The portable record must represent the callable as one closed owner graph, not as a count plus unrelated sparse side tables.

The target conceptual shape is:

```text
CallableAnalysisRecord
    owner: StableCallableRef
    body_source_owner
    body_range
    status

    bindings[]
    expressions[]

    flow_graph
    entry_flow
    normal_exits[]
    throw_exits[]
    unreachable
    return_validation

    for_iteration_lanes[]
    positional_expansions[]
    invalid_positional_expansions[]
    sequence_literals[]

    associated_resolutions[]
    callable_reference_resolutions[]
    family_applications[]
    match_resolutions[]
    pattern_site_resolutions[]

    diagnostic_causes[]
    diagnostics[]
    explanations[]
    return_explanation?

    callable_dependencies[]
    semantic_dependencies[]

    constructor_super_initializer?
```

The exact public DTO type name/version is **VERIFY-FIRST** because the live artifact format already has `SemanticWorldV2` containing `CallableAnalysisRecordV1`. Luna must inspect the repository's artifact schema/version policy before renaming or reinterpreting an existing serialized record.

**Fixed semantic requirement:** changing field layout may not silently reinterpret old bytes as the new contract. If the current P7 artifact format is intentionally unreleased and the accepted versioning policy permits replacing the record in place, follow that policy. If compatibility/versioning consequences are material or unclear, STOP AND CONSULT.

## 7.3 Portable identity

Within a callable record, all callable-local handles are represented as family-specific local ordinals qualified by the record's stable callable owner.

Logical identities:

```text
(callable, expression_ordinal)
(callable, binding_ordinal)
(callable, flow_node_ordinal)
(callable, flow_edge_ordinal)
(callable, explanation_ordinal)
(callable, diagnostic_cause_ordinal)
```

Never serialize as portable authority:

```text
ExpressionId
BindingId
FlowNodeId
FlowEdgeId
ExplanationId
DiagnosticCauseId
TypeStoreId
native map/arena offsets
```

Native local IDs are permitted only as the source-side keys used to construct the ordinal projection and the import-side fresh IDs produced during reconstruction.

## 7.4 One ordinal allocator

`CallableAnalysisOrdinalMap::new` remains the single source-side mapping authority.

Do not create:

```text
expression ordinal allocator in expression exporter
binding ordinal allocator in binding exporter
separate explanation ordinal allocator in diagnostics exporter
```

Every sub-exporter receives the one already-built map.

Import performs the inverse mapping once per callable.

## 7.5 Frozen imported storage

The approved model follows `FlowGraph`:

```text
source checker representation:
    mutable native maps as needed by analysis

imported artifact representation:
    immutable charged sorted backing

consumer:
    one read API
```

PATCH1 must eliminate representation leakage that forces imported data back through native `BTreeMap` / `HashMap` nodes solely to satisfy legacy readers.

### Required design

Introduce one reusable semantic read-map abstraction, or an equivalently coherent set of wrappers, for callable-owned ordered maps.

Conceptually:

```rust
enum CanonicalAnalysisMap<K, V> {
    Source(BTreeMap<K, V>),
    Frozen(BudgetedSortedMap<K, V>),
}
```

Required read API:

```text
get
contains_key
iter
keys
values
len
is_empty
```

Source-only mutation may be exposed through explicitly source/mutable methods.

Do not implement `Deref<Target = BTreeMap<...>>`.

Use the same design principle for:

- `ExpressionAnalysisIndex`;
- `BindingAnalysisIndex`;
- `CallableBindingOriginIndex` if it remains separate;
- associated/callable-reference/family indexes;
- iteration/expansion/sequence indexes;
- match/pattern indexes;
- explanation nodes;
- nested callable flow-state maps;
- sequence exact-expansion maps;
- any other map/set reachable from a reconstructed `CallableAnalysis`.

If a nested map remains ordinary `BTreeMap` in the final imported owner, PATCH1 cannot truthfully pass its whole-callable exact-allocation gate.

### Outer table

`CallableAnalysisTable` must stop exposing its `HashMap` through `Deref`.

Keep explicit lookup and iteration methods.

Imported table backing must be immutable and budget-compatible. Preserve:

```text
O(log N) or better callable lookup
O(callables in module) iter_module
stable iteration where artifact consumers require it
no unbudgeted shadow map
```

Whether the frozen table uses one sorted callable array plus module ranges or a separate charged module index is mechanically flexible **only if** module-bounded complexity and exact accounting are preserved.

## 7.6 Binding authority and formal attachment

PATCH1 must avoid three independent authorities for one binding.

The preferred portable form folds storage origin and formal-attachment classification into the binding row.

Conceptually:

```text
CallableBindingRecord
    ordinal
    name
    parameter?
    range
    contract?
    current
    denotation?
    consistency
    causal_invalidity
    mutable
    initialization
    version
    explanation?

    authority:
        BodyLocalSource {
            site_owner
            site_local
            declaration_range
        }

        BodyLocalGenerated {
            reason
        }

        ModuleValue {
            stable module
            symbol name
            import_binding?
        }
```

The exact enum/type names are flexible.

### Fixed semantics

- `BodyLocalSource` means an explicit source declaration/formal attachment exists.
- `BodyLocalGenerated` means absence of a source declaration is semantically valid and explicitly represented.
- `ModuleValue` means storage belongs to the canonical module symbol; it must not acquire a callable-local declaration attachment.
- A source range alone is not the canonical source-site identity.
- The portable source-site identity must preserve the source owner and local site identity needed to distinguish otherwise identical ranges/names.
- The native `FormalSemanticProjection` may still reduce that authority to an attachment range for its current lowering reader, but the artifact must not lose the exact site identity during transport.

### Export seam

The exporter must receive exact binding attachment authority, not merely:

```rust
BTreeMap<BindingId, SourceRange>
```

Change the source-to-artifact seam so it can project the actual `SourceSiteId`/equivalent owner-qualified site plus range.

Do not reconstruct site identity later by scanning names or ranges.

## 7.7 Synthetic `<main>`

The body-source namespace and binding-storage namespace are independent.

For synthetic top-level body analysis:

```text
body_source_owner = SourceOwner::Module(...)
```

may coexist with:

```text
BodyLocalSource binding
```

inside a nested top-level lambda or generated body.

Never:

- invent a normal callable signature for `<main>`;
- infer `<main>` identity from selector spelling;
- rebase module-owned source sites by callable count/order;
- merge two same-named bindings from distinct lambda bodies;
- treat a module-valued handle as a callable-local binding because it appears in the callable table.

## 7.8 Expression records

Every expression ordinal gets an owning row.

Minimum semantic dimensions:

```text
ordinal
range
TypeKnowledge

selected callable?
call specialization?

conditional dispatch?
conditional capability?

trait dispatch?
trait dispatch candidates?

contextual realization?
denotation?

analysis status
causal invalidity
explanation?

binding read safety?
binding read?
binding write?
```

`ExpressionAnalysis.call: Option<CallResolutionId>` is a special case.

The accepted patch program does **not** authorize inventing a portable `CallResolutionId` owner.

Policy:

```text
if absent:
    encode absence normally

if present and an already-accepted canonical artifact owner exists:
    reference that owner

if present and no canonical portable owner exists:
    fail closed with a precise coverage error
```

If a real canonical Universe callable requires nonempty `call` and no accepted artifact owner exists, this is a plan-premise failure. STOP AND CONSULT. Do not manufacture an ordinal and do not silently drop the field.

## 7.9 Generic call specialization

Preserve:

```text
selected callable
static argument bindings:
    Term
    RecordRow
    DynamicType

generic conformance bindings:
    Exact
    Forwarded
    Derived
    CompilerDerived
```

Each conformance source must retain the canonical stable owners, generic key, evidence/provenance, and fingerprints required by its native product.

Do not:

- map `DynamicType` to `None`;
- map Dynamic to `TypeId(0)`;
- clear the conformance array on import;
- rerun generic conformance solving during load.

## 7.10 Resolution families

Transport exact checked products for:

```text
for_iteration_lanes
positional_expansions
known_invalid_positional_expansions
sequence_literals
associated_resolutions
callable_reference_resolutions
family_applications
match_resolutions
pattern_site_resolutions
conditional dispatch
conditional capability
trait dispatch
trait dispatch candidates
contextual realizations
```

The compiler consumes restored selections.

No name-based or selector-based re-resolution on load.

## 7.11 Flow/return authority

Keep the existing frozen `FlowGraph`.

Add the missing owner facts:

```text
entry_flow
normal return facts
throw flow states
unreachable
return-contract validation
```

`BodyExitFacts` must not be reduced to `unreachable`.

Nested flow-state binding and field maps must use the budget-compatible imported representation selected in §7.5.

## 7.12 Explanations and diagnostics

The artifact must preserve explanation/diagnostic identity and sharing.

Validation must establish:

```text
each explanation ordinal unique
each explanation parent exists
each evidence reference resolves
each diagnostic cause exists
each expression/binding explanation resolves
return_explanation resolves
no wrong-owner reference
native DAG/cycle invariant preserved
shared node remains shared
```

Do not turn invalid/blocked facts into ready facts by deleting causes.

## 7.13 Dependencies

Persist both:

```text
dependencies: CallableId[]
semantic_dependencies: SemanticDependency[]
```

Stable dependency keys must be encoded through canonical stable owner codecs.

Do not encode `Debug` strings as owner identity.

## 7.14 Post-relocation fingerprint

`dependency_fingerprint` is not trusted wire authority.

After complete native reconstruction:

```rust
analysis.dependency_fingerprint =
    callable_body_product_fingerprint(&analysis);
```

or use the repository's equivalent immutable constructor pattern if fields cannot be assigned post-construction.

The final source/import comparison must use the PATCH0-defined fingerprint semantics.

## 7.15 Canonical publication status

A canonical Universe artifact may publish a required callable only when:

```text
status == Complete
internal_incidents.is_empty()
portable required owner fields are complete
all local/cross-owner references validate
```

The wire may retain other status variants for schema completeness or noncanonical future uses, but canonical Universe publication must fail early on non-Complete analyses.

## 7.16 `StaticApplication` boundary

`CheckedStaticBody` remains the sole checked static-application row owner.

PATCH1 must **not** copy checked static-application rows into `CallableAnalysis`.

If an expression denotation requires `SemanticDenotation::StaticApplication` and the checked-body artifact row/reference is not yet complete, keep that denotation fail-closed. The checked-body transport belongs to PATCH2 according to the consolidated patch program.

---

# 8. Ownership boundaries

## 8.1 Owns

PATCH1 owns:

- complete callable-analysis field disposition;
- callable owner wire shape;
- callable-local portable identity;
- frozen/budget-compatible callable owner storage;
- callable-table frozen storage;
- binding/source attachment authority carried by callable transport;
- complete expression and binding transport;
- call specialization conformance transport;
- callable resolution-family transport;
- entry/exit/return transport;
- explanation/diagnostic closure;
- callable and semantic dependency transport;
- constructor-super transport integration;
- per-callable transactional import;
- callable product fingerprint recomputation after relocation;
- formal projection contribution from imported callable authority;
- source-built/imported compiler-reader parity for callable facts;
- callable-specific coverage guard removal.

## 8.2 Does not own

PATCH1 does not own:

- accepted callable-definition provenance;
- trait header/surface fidelity;
- witness-plan repair;
- derived conformance primary rules;
- global trait dispatch restoration;
- full static definition/constant/case/module-binding restoration;
- checked `StaticApplication` row transport;
- global `TypeStore` allocation closure;
- global semantic validator allocation closure;
- global semantic importer `UnaccountedBoundary` removal;
- query-image evidence reconstruction;
- container canonical-byte fixes;
- package-level P7 closure;
- P8 executable Universe;
- P9 artifact freshness/incremental rebuild.

## 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate/reconstruction |
|---|---|---|---|
| Expression type/status/dispatch/read/write | `ExpressionAnalysis` | lowering, diagnostics, fingerprints | source AST/name lookup |
| Binding state | `BindingState` | lowering/formal projection/fingerprint | sparse denotation side table as substitute |
| Binding storage authority | callable binding authority/origin | core lowering | range/name inference |
| Formal declaration identity | `SourceSiteId`-equivalent attachment authority | formal projection | range-only canonical identity |
| Call specialization | expression `CallSpecialization` | compiler runtime recipe | redo inference |
| Generic conformance source | `GenericCallConformanceSource` | compile/runtime evidence | empty conformances |
| Iteration selection | callable iteration resolution | lowering | selector matching |
| Associated selection | associated resolution product | lowering | member re-lookup |
| Family selection | family application product | lowering | family re-resolution |
| Match/pattern plan | match/pattern resolution | lowering | re-run matcher semantics |
| Flow graph | `FlowGraph` | queries/fingerprint | rebuild from syntax |
| Flow boundary facts | `entry_flow` / `BodyExitFacts` | return semantics/fingerprint | `unreachable` alone |
| Explanation DAG | `ExplanationArena` | diagnostics/explanations | regenerated generic explanations |
| Callable dependencies | `CallableAnalysis` | query invalidation | source scanning |
| Callable product fingerprint | recomputed native restored owner | query system | arbitrary wire hash |

---

# 9. Global invariants

`INV-*` identifiers are durable and must be referenced by tasks/tests.

### INV-01 — Owner authority

Every compiler-consumed callable semantic fact is either transported losslessly or explicitly rejected. No default, guess, source lookup, or semantic re-solve may masquerade as restored authority.

### INV-02 — Owner-qualified local identity

Every callable-local reference is interpreted only inside its stable callable owner and semantic family. Equal numeric local IDs in different callables or different families never alias.

### INV-03 — One ordinal projection

One `CallableAnalysisOrdinalMap` projection determines all source-side local ordinals for one callable record.

### INV-04 — Root-before-projection

Every type/kind/domain/row/parameter/evidence root reachable from a portable callable row is registered before shared type graph closure. Row export may not discover a new root.

### INV-05 — No source fallback

Artifact import and imported compiler lowering must not require canonical Universe source syntax, source names, or broad `SourceSemanticIndex` lookup.

### INV-06 — Exact binding storage identity

`BodyLocal` and `ModuleValue` remain distinct. A module symbol is canonical storage authority for module values; a local binding is always qualified by its callable owner.

### INV-07 — Explicit attachment absence

A missing source attachment is semantically valid only when owner data explicitly classifies the binding as generated/implicit or module-valued. Missing explicit formal attachment is an error.

### INV-08 — Synthetic `<main>` separation

Module source ownership does not imply module storage for every body-local binding in synthetic top-level analysis.

### INV-09 — Dynamic remains explicit

`DynamicType` and dynamic evidence are never converted to missing arguments or arbitrary concrete type IDs.

### INV-10 — Checked selections survive

Conditional, trait, associated, family, iteration, sequence, match/pattern, and contextual selections are restored as checked semantic products; they are not re-solved.

### INV-11 — Flow parity

Imported flow graph, entry state, exit states, return validation, and abrupt/unreachable facts are semantically equivalent to source-built facts.

### INV-12 — Explanation closure

Explanation and diagnostic graphs preserve identity, sharing, parent/evidence closure, and invalidity state.

### INV-13 — Derived fingerprint

Callable product fingerprint is recomputed from the final relocated owner using the PATCH0-correct algorithm.

### INV-14 — Source/import read equivalence

Existing semantic and core readers receive equivalent results from source and frozen imported callable owners without materializing shadow native maps.

### INV-15 — Imported immutability

Imported callable owner indexes and tables cannot be mutated or silently thawed into native mutable maps.

### INV-16 — Exact callable-stage allocation

All allocator requests made by complete callable owner materialization are charged through its child transaction before request; failure leaves the parent ledger unchanged.

### INV-17 — Module-local complexity

`CallableAnalysisTable::iter_module` remains bounded by that module's callable set; PATCH1 must not regress P6 to global scans.

### INV-18 — Guard discipline

Callable-specific fail-closed guards remain until all PATCH1 guard-removal discriminators pass. Global allocation guards remain after PATCH1.

### INV-19 — `StaticApplication` owner boundary

Checked static-application rows remain owned only by `CheckedStaticBody`.

### INV-20 — Canonical publication status

A required canonical Universe callable may be exported only with `Complete` status and no internal incidents.

### INV-21 — Deterministic wire order

Equivalent callable semantics must emit canonically ordered rows independent of hash-map iteration and incidental allocation history.

### INV-22 — No hidden duplicate authority

Sparse compatibility side tables may exist only as codec compatibility artifacts if they are mechanically derived from one owning row and validated 1:1; they must not become independent semantic authorities.

---

# 10. Non-goals

Do not expand PATCH1 into:

- PATCH0 fingerprint-contract work except verifying its prerequisite;
- trait/witness/callable-definition repair;
- complete static owner transport;
- checked-static-application transport;
- query-image repair;
- full `TypeStore` transactional rewrite;
- global semantic validator accounting;
- container schema canonicality;
- artifact discovery/freshness;
- P8 VM image work;
- package fixture cleanup;
- broad compiler performance optimization unrelated to callable import;
- unrelated source-index/editor redesign.

Do not opportunistically migrate unrelated BTreeMap-based semantic owners merely because a reusable frozen map abstraction is introduced.

---

# 11. Expected impact map

## 11.1 Expected source areas

### Primary

- `phalcom-semantic/src/checker/analysis.rs`
  - callable/index storage abstractions;
  - explicit read APIs;
  - imported frozen construction seams.

- `phalcom-semantic/src/explain/arena.rs`
  - frozen imported explanation backing.

- `phalcom-semantic/src/artifact/export.rs`
  - complete owner projection;
  - one ordinal map;
  - exact attachment export;
  - canonical ordering.

- `phalcom-semantic/src/artifact/import.rs`
  - complete transactional callable materializer;
  - exact closure validation;
  - formal projection contribution;
  - fingerprint recomputation.

- `phalcom-semantic/src/artifact/roots.rs`
  - final schema reconciliation only;
  - preserve/extend live `21a547` root closure as necessary.

- `phalcom-compiler-artifact/src/semantic/world.rs`
  - owner-centric wire rows;
  - bounds;
  - structural validation.

### Secondary / likely

- `phalcom-semantic/src/checker/associated.rs`
- `phalcom-semantic/src/match_semantics/...`
- `phalcom-semantic/src/trait_dispatch.rs`
- `phalcom-semantic/src/generic_evidence.rs`
- `phalcom-semantic/src/presentation.rs`
- `phalcom-semantic/src/source_index/mod.rs`
- `phalcom-semantic/src/db/fingerprint.rs` — verify-only unless PATCH0 mechanical adaptation is needed
- `phalcom-core/src/modules/semantic_lowering.rs`
- live allocation helper module owning `BudgetedSortedMap`

## 11.2 Expected tests

- `phalcom-compiler-artifact/tests/semantic_world_callable_analysis_test.rs`
- callable root tests in `artifact/roots.rs`
- callable import/export allocation tests near `artifact/import.rs`
- source attachment regression tests
- explanation DAG tests
- focused core semantic-lowering parity tests
- one new coherent source→wire→fresh relocation→consumer parity harness, placed according to live test conventions.

## 11.3 Expected documentation/state

Update:

```text
PERF003.C2 CHECKPOINT.md
PERF003.C2.P7 walkthrough/state records as required by current convention
PERF003.C2.P7.PATCH1-walkthrough.md
PERF003.C2.P7.PATCH1-handoff.md
```

Do not create a competing `implementation-state.md`.

## 11.4 Unexpected-touch rule

Touch adjacent helpers/tests when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem if that entry changes ownership, representation, artifact compatibility, language semantics, global allocation policy, or the P7/P8/P9 boundary.

---

# 12. Implementer decision authority

## 12.1 FIXED

Luna must not change these decisions.

1. `CallableAnalysis` remains the semantic owner.
2. Artifact load does not redo semantic analysis.
3. One owner-local ordinal projection per callable.
4. Local ordinals are callable-qualified and family-specific.
5. Imported callable owners use immutable budget-compatible storage.
6. Do not rebuild imported shadow `BTreeMap`/`HashMap` structures merely for legacy APIs.
7. `FlowGraph` dual backing is precedent, not a temporary workaround to undo.
8. Binding storage/attachment authority must be explicit.
9. Source range is not canonical source-site identity.
10. Broad source index is absent in the imported P7 canonical image.
11. Dynamic remains an explicit static-argument/evidence state.
12. Generic conformance bindings are semantic authority.
13. Compiler lowering consumes checked products; it does not re-solve dispatch/match/iteration/family semantics.
14. `dependency_fingerprint` is derived after relocation.
15. `CheckedStaticBody` owns checked static applications.
16. Global semantic `UnaccountedBoundary` guards remain after PATCH1.
17. Callable export/import guards are removed only at the final PATCH1 gate.
18. Canonical Universe callable publication requires `Complete` status and no internal incidents.

## 12.2 MECHANICALLY FLEXIBLE

Luna may adapt:

- helper/type names;
- private module decomposition;
- whether one generic frozen/read map wrapper or a few specialized wrappers are cleaner;
- exact iterator adapter implementation;
- exact test file placement matching repository conventions;
- exact DTO names if artifact schema versioning dictates them;
- whether dense rows inline small nested records or reference owner-local side arrays, provided there is one semantic authority and closure is validated;
- helper signatures needed to satisfy Rust borrowing;
- internal builder decomposition;
- canonical sorting implementation;
- use of existing stable owner codecs instead of adding equivalent ones.

Mechanical flexibility does **not** permit a second semantic source of truth.

## 12.3 VERIFY-FIRST

Before substantive edits, verify:

| Assumption | Where | If false |
|---|---|---|
| PATCH0 callable fingerprint corrections are already implemented | `db/fingerprint.rs`, PATCH0 record/checkpoint | STOP AND CONSULT if PATCH1 would otherwise persist known-wrong fingerprint semantics |
| Current root census substantially matches `21a547` checkpoint claim | `artifact/roots.rs` + root tests | adapt missing roots mechanically if ownership is unchanged |
| `CallableAnalysisOrdinalMap` remains the only exporter ordinal authority | `artifact/export.rs` | consolidate if mechanical; consult if another accepted identity authority exists |
| `CallableId`/relevant local keys have stable ordering suitable for frozen maps | identity types | choose charged index shape mechanically if yes; consult if stable lookup requires new identity semantics |
| `SourceSiteId` has portable owner + local identity available at export seam | identity/source index | adapt codec if straightforward; consult if source-site identity is not stable enough for artifact use |
| `FormalSemanticProjection` still consumes ranges only | `presentation.rs` | preserve exact site in artifact and project range; consult if compiler now consumes richer source-site identity |
| `ExpressionAnalysis.call` remains without a canonical artifact owner | analysis/query/source code | if canonical Universe uses nonempty values, STOP AND CONSULT |
| Existing conformance evidence types expose enough stable owner data for lossless codec | generic/conformance code | consult before inventing new proof identity |
| `BudgetedSortedMap` is still the accepted charged immutable map primitive | live allocation code/checkpoint | adapt equivalent accepted primitive; consult if allocation policy changed |
| Public consumers rely on `CallableAnalysisTable::Deref` | code search | migrate mechanically; consult only if external API compatibility is intentionally promised |
| Root registration now covers checked static application rows separately | `roots.rs` | preserve owner boundary; do not move rows into callable analysis |
| Current semantic world schema versioning policy permits the required record evolution | artifact header/schema docs | STOP AND CONSULT if compatibility semantics are unclear/material |

---

## 12.4 Adopted architecture amendment — portable query-owner identity

**Decision ID:** `PERF003.C2.P7.PATCH1-ADR-QUERY-OWNER`
**Decision date:** 2026-10-16
**Decision baseline:** `0bbc678b9dc280de3c33ba9938c8bfc01d4d4fd0`
**Status:** approved for implementation; implementation and acceptance pending.

This amendment adopts the architect's supplied decision. It is normative for
PATCH1 and resolves the T4/T5 consultation hold recorded in the checkpoint.

### Fixed architecture and PATCH4 boundary

PATCH1 owns a closed, typed, deterministic, callable-local portable
representation of `QueryKey` identities embedded in callable semantic facts.
It preserves semantic identity only; it does not serialize query-cache state,
reconstruct a query result, execute a query, re-run semantic analysis, or use
source lookup. PATCH1 must not depend on PATCH4 completion or expand the shared
query-image wire. PATCH4 owns query-image persistence and may reuse the logical
identity rules established here without reinterpreting them.

Unsupported/nonportable identities use typed fail-closed rejection. This is a
codec disposition, not a redefinition of canonical-publication eligibility.
The publisher must separately validate semantic eligibility and complete
portable representability. A legitimate canonical callable state required by
T4/T5 cannot be made unsupported merely to simplify the codec.

### Publication domain and portable identities

The contract covers an exhaustively classified canonical-publication subset
of `QueryKey`, not automatically every internal query variant. Before fixing
the final wire enum, QO-0 must classify every live `QueryKey` variant as:

1. supported with complete portable identity and root closure;
2. ineligible for canonical callable publication, with governing
   invariant/evidence; or
3. unsupported and explicitly rejected with a typed error.

Every query owner reachable from an accepted callable must be supported.
No wildcard may silently accept future variants; a rejection wildcard is
permitted only when it always fails closed.

`GenericConformance` and `TraitDispatchBucket` must be represented losslessly.
The generic key preserves the complete `TypeTerm` and `TraitRef`, including
stable declaration identity and ordered type arguments. The dispatch bucket
preserves every field participating in `TraitDispatchBucketKey` equality and
hashing. Exact field layouts must come from live types. Names, ranges, partial
selectors, or re-resolved dispatch cannot substitute for identity.

Use stable portable type/declaration identities, not raw `TypeId`,
`InferVarId`, `TypeStoreId`, or other process-local handles. Reuse existing
canonical identity rules where complete. Portable encoding and fingerprinting
must share one logical identity contract.

### PUB-QO-01 — Closed query-owner identity

Every query-owner identity reachable from a publishable canonical callable
must be fully resolved to portable semantic identities. Unresolved
`TypeTerm::Infer(InferVarId)` is prohibited. Export preflight must detect and
reject it before portable record construction. An existing authoritative
finalized substitution may be projected only under established canonical
normalization rules; export must not perform new inference, query execution,
or semantic resolution.

### Root closure, validation, and relocation

Extend callable root registration through every occurrence of
`UnavailableSemanticOwner(QueryKey)`, including binding consistency and
expression status, and audit the rest of the callable graph for other such
paths. Root traversal and encoding must have matching variant coverage.
Register all type, trait, declaration, and identity roots before portable
references are finalized. Import may not repair missing roots.

Import must validate variant tags, admitted query kinds, graph references,
stable declarations, trait argument consistency, owner relationships,
canonical encoding, nesting/resource limits, and allocation-budget use before
publishing the reconstructed callable. It reconstructs the exact native key
after fresh-store relocation and never queries or re-solves semantics.

### FP-QO-01 — Relocation-invariant query-owner fingerprinting

Query-owner fingerprint contributions use the same normalized logical
identity as portable encoding, including canonical structural type and stable
declaration identities. Equivalent owners in independent stores must
contribute equal fingerprints despite different native handles; distinct
semantic owners must remain distinguishable. Raw relocated handles are not
cross-store semantic identity inputs. Preserve PATCH0 fingerprint coverage.
Derive `dependency_fingerprint` after relocation. Test cross-store fingerprint
equality and independently compare reconstructed semantic facts; recomputing
a fingerprint alone is not proof of parity. Do not add a serialized
fingerprint field unless the established schema contract requires one.

### T4/T5 and guard acceptance

T4 preserves every admitted `BindingConsistency` variant and complete payload.
T5 preserves every admitted `AnalysisStatus` variant and complete payload.
Required discriminators include generic-conformance and trait-dispatch blocked
owners, distinct owner keys, unsupported variants, unresolved inference
identities, malformed/missing references, fresh-store relocation, exact
reconstruction, and fingerprint parity. Exercise one logical blocked owner
appearing in both a binding and an expression. Existing canonical-publication
eligibility rules remain independent and unchanged.

Remove the callable completeness guard only after the complete PATCH1 owner
graph, T4/T5 admitted domains, validation, root closure, relocation,
fingerprinting, source-independent reader parity, focused negative tests, and
approved allocation architecture pass. Keep precise typed rejection for
unsupported cases after guard removal. Global `UnaccountedBoundary` guards
remain owned by PATCH3.

Keep the accepted semantic-world schema 2 contract; schema 1 remains rejected.
Do not reinterpret existing bytes or change the shared query-image schema for
this amendment. Unknown discriminants fail closed.

Preserve prior settled decisions: checked `StaticApplication` rows remain
solely owned by `CheckedStaticBody`, and the budgeted enum-requirement wrapper
remains unchanged.

### Required execution order

| Gate | Work | Acceptance |
|---|---|---|
| QO-0 | Exhaustive live `QueryKey` reachability and identity census | Every variant classified before final DTO design |
| QO-1 | Define supported callable-local query-owner variants | Complete portable identity contract |
| QO-2 | Enforce PUB-QO-01 | Unresolved inference identity is typed rejection |
| QO-3 | Register roots nested in blocked owner payloads | INV-04 closure |
| QO-4 | Encode and validate deterministically | Structural, closure, and negative tests |
| QO-5 | Relocate and reconstruct in a fresh store | Exact owner round-trip |
| QO-6 | Normalize query-owner fingerprint contribution | FP-QO-01 parity |
| QO-7 | Complete T4 binding-state cases | T4 acceptance |
| QO-8 | Complete T5 expression-status cases | T5 acceptance |
| QO-9 | Run integrated callable-owner gates | Decide guard-removal frontier |

### QO-0 live `QueryKey` census — 2026-10-16

The live `QueryKey` enum at the decision baseline has 45 variants. The
repository-wide search found four production construction sites for
`BlockReason::UnavailableSemanticOwner`: two in declaration-surface owner
validation, one in callable-signature projection, and one in field-signature
projection. `unavailable_computed_signature_owner` and
`declaration_signature_owner_diagnostics` construct only
`CallableSignature` or `FieldSignature`; the direct projection sites use their
own `CallableSignature` and `FieldSignature` keys. The incremental
`query_ownership` test independently confirms the field-signature case.

The portable query-owner contract must also implement the architect-required
`GenericConformance` and `TraitDispatchBucket` variants even though no current
`UnavailableSemanticOwner` construction site was found for those variants.
Their presence as `SemanticDependency` variants is a separate owner relation;
do not mistake that storage for a blocked query-owner payload.

| Disposition | `QueryKey` variants | Evidence/requirement |
|---|---|---|
| Existing portable identity; currently reachable | `CallableSignature`, `FieldSignature` | Existing stable callable/field owner encodings; live unavailable-owner producers above |
| New callable-local portable identity required | `GenericConformance`, `TraitDispatchBucket` | Explicit architect requirement; complete owner fields and roots must be encoded |
| No current callable blocked-owner producer; typed fail-closed if encountered | `ParsedModule`, `UnlinkedInterface`, `ResolvedImports`, `LinkedInterface`, `DeclarationShell`, `SemanticComponent`, `DeclarationSurface`, `TraitHeader`, `TraitSurface`, `ConformanceDispatch`, `CallableDefinition`, `HierarchyEdge`, `ConstructorInitialization`, `CallableBody`, `CallableEffects`, `CallableControl`, `CallableTermination`, `CallableContracts`, `VerificationConditions`, `SourceStructure`, `SourceFormalAttachment`, `AdvisoryCallable`, `AdvisoryModule`, `ModuleDiagnostics`, `ModuleMetadata`, `DataDeclaration`, `EnumDeclaration`, `EnumRequirements`, `TypeMemberDeclaration`, `StaticDomainCaseCatalog`, `StaticCaseSignature`, `AssociatedStaticDomainDeclaration`, `AssociatedSurface`, `ResolvedImport`, `LinkedName`, `PublicExport`, `StaticDefinition`, `StaticConstant`, `ModuleStaticBindingCatalog`, `ModuleStaticBindings`, `CheckedStaticBody` | Exhaustive remainder of the live enum; no production unavailable-owner constructor or propagation source reaches callable state in the current tree. If a producer is added, revisit this disposition before accepting publication. |

All 45 variants are classified above. The callable owner graph can contain
other `BlockReason` payload sites in addition to `AnalysisStatus` and
`BindingConsistency`: field validity inside flow summaries, blocked pattern
resolution/value-space/exhaustiveness facts, and internal-blocked explanation
steps. The encoder, root visitor, validator, importer, and fingerprint census
must cover every such path found in the final field-exhaustive walk. The direct
producer census currently limits `UnavailableSemanticOwner` payloads to the
two existing signature-owner variants, while the portable codec must also
support the two architect-required variants.

These are PATCH1 task-order amendments, not a new checkpoint. PATCH1 remains
`IN_PROGRESS / PARTIAL / UNVERIFIED`; this decision alone does not complete
T4/T5, remove guards, close P7, or unblock P8.

# 13. Global STOP / CONSULT triggers

A triggered consultation cannot be self-waived.

STOP AND CONSULT if:

1. `CallableAnalysis` can no longer serve as the restored compiler owner without duplicating semantic authority.
2. A native callable field consumed by core has no stable artifact representation and cannot be safely classified fail-closed.
3. Real canonical Universe analyses contain nonempty `ExpressionAnalysis.call` and no accepted canonical portable owner exists.
4. Exact formal site identity cannot be represented without importing the broad source index or parsed syntax.
5. Two plausible formal-attachment designs have materially different identity semantics.
6. A frozen imported representation would require changing observable semantic behavior of source-built analyses.
7. A required map-backed field cannot be migrated to charged immutable storage without a new global allocation policy.
8. Artifact schema versioning/compatibility must be materially changed beyond the existing accepted mechanism.
9. Generic conformance evidence cannot be losslessly represented using stable existing owners.
10. The implementer is tempted to store a checked static-application row in `CallableAnalysis`.
11. Compiler lowering can pass only by querying canonical Universe syntax/source.
12. A required parity test implies the accepted source semantics are wrong.
13. The same nontrivial semantic failure persists after one serious correction.
14. A workaround/special-case architecture starts emerging for Universe-only rows.
15. A new duplicate source of semantic truth would be introduced.
16. Exact whole-callable allocation can only be claimed through guessed native map-node sizes.
17. An invariant above must be weakened to make the tests pass.
18. PATCH1 scope expands into trait/witness/static/query/container architecture materially.
19. The live repository has diverged enough that the consolidated patch's owner model is no longer true.

### Consultation packet format

Collect cheaply:

```text
Trigger:
Live revision:
Task:
Observed:
Expected invariant:
Relevant owner types:
Relevant wire types:
Relevant consumer:
Minimal failing test:
Small source excerpts/diff:
Two plausible directions, if any:
Decision required:
```

Do not ask an adviser to rediscover the whole repository.

---

# 14. Debugging budget

## 14.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress.

One cycle:

```text
inspect
identify concrete mechanical cause
make one coherent correction
rerun smallest discriminating command
```

Examples:

- renamed helper;
- import visibility;
- trait bounds on generic frozen map wrapper;
- iterator lifetime;
- wrong test filter;
- borrow-checker cleanup.

## 14.2 Semantic failures

Before changing code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow **one** serious correction for the same semantic failure.

If it persists: STOP AND CONSULT.

Examples:

- source/import binding identity differs;
- imported generic conformance source changes;
- formal attachment resolves wrong site;
- owner-local ordinal aliases;
- fingerprint changes after relocation unexpectedly;
- core lowering differs despite field parity.

## 14.3 Architectural failures

Zero speculative architecture-fix attempts.

Consult immediately.

Never repeatedly rerun an unchanged failing test unless relevant code/data changed.

---

# 15. Testing surface analysis

The designed coverage is broad. Execution remains staged and selective.

| Coverage ID | Dimension | Invariants | Required case |
|---|---|---|---|
| CV-01 | canonical success | 01,03,04,10,13 | one rich complete callable round trip |
| CV-02 | owner isolation | 02 | two callables with same numeric local IDs |
| CV-03 | empty/zero boundary | 02,11,12 | valid body with zero optional resolution families |
| CV-04 | cardinality boundary | 16 | max-allowed and max+1 expression/binding/nested counts |
| CV-05 | duplicate negative | 02,12 | duplicate expression/binding/explanation ordinal |
| CV-06 | wrong-owner negative | 02 | cross-callable local reference rejected |
| CV-07 | binding authority | 06,07 | body-local source, generated, module value |
| CV-08 | synthetic main | 08 | module source owner + two local same-name bindings |
| CV-09 | expression parity | 01,10 | knowledge/status/read/write/safety/dispatch/denotation |
| CV-10 | Dynamic | 09 | Dynamic vs missing vs concrete term |
| CV-11 | generic calls | 09,10 | Exact/Forwarded/Derived/CompilerDerived |
| CV-12 | flow parity | 11 | entry, normal returns, throws, unreachable, validation |
| CV-13 | associated/reference/family | 10 | exact associated + family + callable-reference selections |
| CV-14 | match/pattern | 10 | executable pattern binding plan |
| CV-15 | iteration/sequence | 10 | for, positional expansion, invalid expansion, sequence |
| CV-16 | conditional/trait/context | 10 | conditional capability, trait selection/candidates, contextual realization |
| CV-17 | explanation DAG | 12 | shared parent, evidence, return explanation |
| CV-18 | diagnostic closure | 12 | cause references preserved and invalid status unchanged |
| CV-19 | dependency parity | 01,13 | callable + semantic dependencies survive relocation |
| CV-20 | constructor-super | 01 | exact range + selected initializer |
| CV-21 | status publication | 20 | Complete accepted; partial/blocked/incidents rejected |
| CV-22 | root closure | 04 | every final portable type-bearing family closes graph |
| CV-23 | relocation | 13 | source store A → fresh numbering B |
| CV-24 | determinism | 02,21 | equivalent owner under perturbed allocation emits equal wire |
| CV-25 | frozen read parity | 14,15 | source/frozen reads produce identical observable sequence |
| CV-26 | module locality | 17 | `iter_module` touches only module callable rows |
| CV-27 | allocation exactness | 16 | observed N succeeds, N−1 rejects/rolls back |
| CV-28 | formal reader | 05,06,07,08 | projection works with empty broad source index |
| CV-29 | lowering parity | 05,10,14 | source/import compiler projection equal |
| CV-30 | guard frontier | 18 | callable guard removed only after all above; global guard remains |
| CV-31 | unsupported call ID | 01 | nonportable `CallResolutionId` path fails precisely |
| CV-32 | static-app boundary | 19 | no callable-owned checked static application introduced |

---

# 16. Verification execution budget

## 16.1 Modes

### BUILD MODE

Goal: implement architecture quickly with sparse discriminating feedback.

### STABILIZE MODE

Goal: prove complete callable owner parity and allocation behavior.

### CERTIFY MODE

PATCH1 certification is **focused**, not workspace/release certification. Full P7 certification belongs to PATCH7.

## 16.2 Verification ladder

```text
T0 — exact reproducer/new regression
T1 — directly affected callable artifact tests
T2 — phalcom-semantic owning subsystem focused suite
T3 — semantic→core lowering integration
T4 — affected crate/package verification if evidence demands
T5 — workspace/release verification (deferred)
```

## 16.3 Mandatory during BUILD

Run only at coherent task boundaries:

- new focused DTO/validator test when the schema slice becomes coherent;
- focused semantic root test when final rows are known;
- focused source/frozen read parity test after storage conversion;
- focused import helper test after each coherent materializer family;
- `git diff --check` at meaningful commit/checkpoint boundaries.

Do **not** run core end-to-end artifact loading while global semantic guards intentionally prevent that path.

## 16.4 Mandatory named gates

Defined in §19.

## 16.5 Mandatory during STABILIZE

- complete callable owner source→wire→fresh relocation parity;
- whole-callable exact allocation;
- formal projection with broad `source_index` absent;
- source/import core lowering parity;
- callable-specific guard frontier test.

## 16.6 Only if evidence demands

- full `phalcom-semantic` library tests;
- full `phalcom-core` canonical artifact target;
- full `phalcom-compiler-artifact` test set.

Run these only if focused changes plausibly affect broad behavior or a required focused gate cannot classify a failure.

## 16.7 Explicitly deferred / DO NOT RUN during BUILD

Do not run reflexively:

```text
cargo test --workspace
cargo clippy --workspace
full language corpus
parse/similarity/slotmap package matrix
cross-process complete .telosc determinism
full P5/P6 regression matrix
P8 tests
P9 cache/freshness tests
```

Reason: PATCH1 intentionally leaves global semantic import allocation guards and other required owner-family guards in place. These broad tests cannot certify PATCH1 during intermediate states and would waste implementation time or fail at unrelated frontiers.

---

# 17. Baseline / unrelated failure policy

Classify every failure:

```text
A — definitely caused by PATCH1
B — probably caused by PATCH1
C — unclear
D — clearly unrelated/baseline
```

Policy:

- A/B: active responsibility.
- C: one bounded classification pass; if nonblocking and still unclear, record and defer.
- D: record and continue immediately.
- Do not weaken assertions to make broad gates green.
- Do not repair unrelated Universe bootstrap/package/consolidation failures.
- Do not consume premium adviser compute on D unless it blocks a mandatory PATCH1 gate.

Record C/D items in `PERF003.C2/CHECKPOINT.md` deferred/baseline ledger.

---

# 18. Tasks

# T0 — Take over the live branch and freeze the callable contract

## Purpose

Establish live repository truth, activate PATCH1 in the checkpoint, verify PATCH0 prerequisites, and produce the exact callable field/consumer/storage census before structural edits.

## Preconditions

- target branch is the intended P7 implementation branch;
- no destructive repository cleanup is authorized.

## Consumes

- current `CHECKPOINT.md`;
- current P7 handoff/walkthrough;
- consolidated PATCH1 architecture;
- current source owner definitions.

## Produces

1. live revision/state anchor;
2. updated checkpoint active-plan note for PATCH1;
3. a field matrix for every `CallableAnalysis` field:
   ```text
   native field
   artifact disposition
   current wire
   root status
   validator
   importer
   native storage
   allocation status
   compiler/query consumers
   parity test
   ```
4. exact list of `CallableAnalysisTable::Deref` consumers;
5. exact list of native map/set allocations reachable from callable owner materialization;
6. verification that PATCH0's fingerprint contract is complete.

## Files and symbols

**Read:**

- all paths in §4.

**Modify:**

- checkpoint record only;
- optionally the accepted artifact contract manifest if PATCH0 already defines a place for implementation-state bookkeeping.

**Tests:**

Use only the smallest existing callable tests to establish baseline.

## Required implementation shape

Do not edit production semantic architecture in this task.

Verify current branch head and dirty state:

```bash
git rev-parse HEAD
git status --short
git branch --show-current
```

Do not reset, clean, or overwrite uncommitted work.

Verify current focused baselines using exact test discovery. Suitable known examples at planning time include:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-compiler-artifact \
  --test semantic_world_callable_analysis_test -- --test-threads=1
```

and the current callable root filter documented by the live checkpoint. Confirm the selector actually runs tests.

Verify PATCH0 by inspecting `callable_body_product_fingerprint` and its discriminator tests. Required PATCH0 dimensions:

```text
binding_write
family_applications
full associated resolution
full callable-reference resolution
conditional/trait/contextual executable dimensions as accepted
```

If any remains known-wrong, PATCH1 cannot proceed to final parity/fingerprint acceptance.

## Forbidden approaches

- no production schema changes;
- no guard removal;
- no “while here” unrelated fixes;
- no broad suite.

## Tests to run now

T0 only:

- current callable artifact wire test;
- current callable root focused filter;
- PATCH0 fingerprint discriminator filter if present.

## Tests explicitly deferred

All new PATCH1 tests and all core end-to-end artifact tests.

## Acceptance

T0 is complete when the implementer can point to an exact current field/storage/consumer matrix and PATCH0 prerequisite status.

## Local STOP / CONSULT triggers

- PATCH0 is materially incomplete;
- live `CallableAnalysis` ownership differs from this plan;
- source attachment ownership changed materially;
- the live branch contains conflicting concurrent callable-schema work.

## Checkpoint update

Record:

- starting revision;
- PATCH1 active;
- prerequisite status;
- any inherited baseline failures.

---

# T1 — Introduce budget-compatible callable owner storage and remove representation leakage

## Purpose

Make the native `CallableAnalysis` owner capable of holding source-mutable or imported-frozen maps without forcing imported data through unaccounted native tree/hash nodes.

## Preconditions

T0 census complete.

## Consumes

- `BudgetedSortedMap`/charged sorted builder API;
- `FlowGraph` dual-backing precedent;
- list of current map consumers.

## Produces

- explicit read APIs for all migrated map-backed callable products;
- frozen construction seam(s);
- no `CallableAnalysisTable::Deref<HashMap>`;
- imported-frozen `CallableAnalysisTable` backing;
- frozen-capable `ExplanationArena`;
- source checker behavior unchanged.

## Files and symbols

**Read/modify:**

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/explain/arena.rs
phalcom-semantic/src/checker/associated.rs
match/pattern index owners discovered in T0
iteration/sequence index owners discovered in T0
FlowStateSummary-related types in checker/analysis.rs
phalcom-semantic/src/checker/flow/graph.rs  # precedent; avoid needless edits
allocation helper module owning BudgetedSortedMap
```

**Tests:**

Add focused source/frozen map/read parity tests near the owning modules.

## Required implementation shape

Create one coherent dual-backing read-map abstraction if the live type system permits it cleanly.

Conceptual API:

```rust
pub struct AnalysisMap<K, V> {
    storage: AnalysisMapStorage<K, V>,
}

enum AnalysisMapStorage<K, V> {
    Source(BTreeMap<K, V>),
    Frozen(BudgetedSortedMap<K, V>),
}

impl<K: Ord, V> AnalysisMap<K, V> {
    pub fn get(&self, key: &K) -> Option<&V>;
    pub fn contains_key(&self, key: &K) -> bool;
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&K, &V)>;
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &K>;
    pub fn values(&self) -> impl ExactSizeIterator<Item = &V>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

Source mutation must be explicit:

```text
insert_source
get_source_mut
retain_source
remove_source
```

or an equivalent private mutable interface.

Imported mutation must fail typed or be impossible through the API.

Do not emulate `Deref<BTreeMap>`.

### Nested maps

Apply the same imported-storage rule to all maps allocated as part of a restored callable, including:

- flow summaries;
- sequence exact expansions;
- explanation node table;
- any resolution-owned substitutions/environments that are truly owned by the callable product and not already immutable shared owners.

If a nested semantic product already owns an immutable `Arc` to canonical evidence constructed elsewhere, reuse that authority rather than copying it into a new map.

### `CallableAnalysisTable`

Refactor to explicit API first:

```text
get
contains_key
len
is_empty
keys
values
iter
iter_module
```

Delete `Deref<Target = HashMap<...>>`.

Preserve source mutation methods.

Add imported construction conceptually:

```rust
CallableAnalysisTable::from_budgeted(
    allocations,
    sorted_callables,
    module_index_or_ranges,
)
```

Exact method name flexible.

## Forbidden approaches

- pre-charging guessed BTree/Hash node sizes;
- keeping an unbudgeted shadow `HashMap`;
- cloning frozen data into mutable maps on first read;
- changing source semantics or iteration order unintentionally;
- global replacement of all semantic maps unrelated to `CallableAnalysis`.

## Test changes required

Covers CV-25, CV-26, CV-27 foundation.

Add:

1. source/frozen read parity;
2. frozen mutation rejection/impossibility;
3. source clone/mutation isolation;
4. module iteration parity;
5. exact allocation request test for each new frozen backing helper where feasible.

## Tests to run now

Only the new storage tests plus directly affected `CallableAnalysisTable`/flow tests.

Do not run artifact round-trip yet.

## Tests explicitly deferred

Full callable DTO/import, compiler lowering, broad semantic tests.

## Acceptance

- all source readers compile through explicit read APIs;
- no `CallableAnalysisTable::Deref<HashMap>` remains;
- imported storage can be created entirely from charged sorted buffers;
- no shadow native maps are required.

## Local STOP / CONSULT triggers

- a consumer requires mutation of imported canonical owners;
- one required index cannot be represented without materially changing semantic identity;
- module-bounded iteration cannot be preserved without a different identity/index architecture.

## Checkpoint update

Record the durable frozen callable-owner storage interface once stable.

---

# T2 — Stabilize one owner-local ordinal and validation model

## Purpose

Make every local cross-reference type-safe in meaning and structurally closed before the complete schema is added.

## Preconditions

T1 read APIs stable.

## Consumes

- current `CallableAnalysisOrdinalMap`;
- current flow-graph ordinal behavior;
- final field census from T0.

## Produces

- one extended ordinal map covering all local families required by final schema;
- one validator model for local ordinals;
- wrong-owner/duplicate tests;
- deterministic ordinal evidence.

## Files and symbols

```text
phalcom-semantic/src/artifact/export.rs
phalcom-compiler-artifact/src/semantic/world.rs
callable artifact tests
```

## Required implementation shape

Extend `CallableAnalysisOrdinalMap` only as required.

It must map:

```text
ExpressionId -> expression ordinal
BindingId -> binding ordinal
FlowNodeId -> flow node ordinal
FlowEdgeId -> flow edge ordinal
ExplanationId -> explanation ordinal
DiagnosticCauseId -> diagnostic-cause ordinal
```

and any other owner-local ID proven necessary by the final native field census.

Every local reference validation helper must receive the owning callable record/context. No global local-ordinal namespace.

Prefer semantically typed wrapper fields/newtypes if existing artifact codec conventions make them cheap. If the codec convention uses `u32`, keep dedicated validation functions and field names so expression/binding/cause ordinals cannot be accidentally interchanged.

### Determinism discriminator

Construct semantically equivalent callable products under perturbed internal allocation/history if the repository test harness permits it.

Require identical portable ordinal assignment.

If ordinal order differs because a native ID encodes incidental allocation history rather than canonical semantic order, STOP AND CONSULT. Do not silently sort by source range/name unless that identity rule is approved.

## Forbidden approaches

- one allocator per DTO family;
- raw native ID serialization;
- ownerless global ordinals;
- source-range ordering invented as canonical identity;
- hiding dangling refs by dropping rows.

## Test changes required

CV-02, CV-05, CV-06, CV-24.

## Tests to run now

Only new ordinal/closure tests.

## Acceptance

One mapping pass can project all current local handles and rejects:

```text
duplicate ordinal
dangling reference
wrong family
wrong callable owner
inconsistent flow adjacency
invalid explanation/cause link
```

before native publication.

## Local STOP / CONSULT triggers

Any determinism failure implying a new canonical ordering rule.

## Checkpoint update

Record owner-local identity invariant if the implementation refines it.

---

# T3 — Replace sparse callable wire with the owner-centric record and final root closure

## Purpose

Define the complete wire owner and make low-level codec validation exhaustive before materialization.

## Preconditions

T1/T2 stable; PATCH0 contract/fingerprint prerequisite verified.

## Consumes

- field matrix;
- one ordinal map;
- live root closure;
- existing stable owner codecs.

## Produces

- complete callable wire record;
- explicit expression/binding rows;
- bounded nested families;
- closed enum/discriminant handling;
- root closure synchronized to final schema;
- low-level encode/decode tests.

## Files and symbols

```text
phalcom-compiler-artifact/src/semantic/world.rs
phalcom-compiler-artifact/src/limits.rs
phalcom-semantic/src/artifact/roots.rs
phalcom-semantic/src/artifact/export.rs
phalcom-compiler-artifact/tests/semantic_world_callable_analysis_test.rs
```

## Required implementation shape

### Schema

Implement the conceptual owner record from §7.2.

Do not retain `expression_count` as the only expression authority.

If a count field remains for codec preflight, validate:

```text
count == expressions.len()
```

and do not use the count as a separate semantic source of truth.

### Canonical row ordering

Emit each map-backed family in stable owner-local key order.

Nested candidate/evidence arrays preserve their semantic order when order is meaningful.

Never emit a `HashMap` iteration order.

### Limits

Before reserving/decoding each dense family, enforce the PATCH0/live `ValidationLimits` bound appropriate to:

```text
expressions
bindings
diagnostic causes
diagnostics
explanations
dependencies
semantic dependencies
match/pattern rows
iteration/sequence rows
candidate lists
generic conformance bindings
normal returns
throw states
nested flow maps
```

Use checked accumulation for total nested row counts when amplification is possible.

### Root reconciliation

For every final portable field containing/transitively containing:

```text
TypeId
KindId
RecordRowId
TypeParameterId
StaticDomainId
callable signature types
generic evidence
type knowledge
type-bearing denotation/resolution
```

verify a corresponding root registration path exists in the live `register_callable_analysis_roots`.

The latest live commit already extends this substantially. Add only the roots the final schema still requires.

No projection helper may call `types.ty(...)`, `types.row(...)`, etc. for a root that was not registered before `close_graph`.

### Unsupported fields

For `ExpressionAnalysis.call` and `SemanticDenotation::StaticApplication`, keep precise fail-closed coverage unless their accepted external owner is already available.

## Forbidden approaches

- keep sparse side tables as substitute for expression/binding rows;
- discover new roots during projection;
- default unknown enum tags;
- silently omit unsupported nonempty native fields;
- version format by accident.

## Test changes required

CV-01 through CV-06, CV-21, CV-22, CV-31, CV-32 at codec/root level.

## Tests to run now

- compiler-artifact callable wire target;
- callable root focused filter.

Do not run semantic import or core yet.

## Acceptance

One rich synthetic wire record encodes/decodes with exact local closure, and malformed variants are rejected before native materialization.

## Local STOP / CONSULT triggers

- artifact version policy is unclear;
- a final type-bearing field has no stable root representation;
- real required native field has no permissible wire disposition.

## Checkpoint update

Record final callable wire schema shape as a durable interface.

---

# T4 — Implement complete binding rows and exact formal/storage attachment authority

## Purpose

Restore binding semantic state and source/storage identity without range/name reconstruction.

## Preconditions

T3 wire row support exists.

## Consumes

- `BindingState`;
- `CallableBindingOrigin`;
- exact source attachment from `CallableSourceAttachment`;
- local binding ordinal map.

## Produces

- complete portable binding row;
- exact owner-qualified source attachment;
- generated/implicit classification;
- module-value authority;
- source-free formal binding range projection.

## Files and symbols

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/source_index/mod.rs
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/artifact/export.rs
phalcom-semantic/src/artifact/import.rs
phalcom-compiler-artifact/src/semantic/world.rs
phalcom-semantic/tests/semantic/integration/source_index.rs
```

## Required implementation shape

Implement the folded binding authority model from §7.6.

### Preserve full `BindingState`

Transport:

```text
binding local ordinal
name
parameter identity?
range
contract?
current TypeKnowledge
denotation?
consistency
causal invalidity
mutable
initialization
version
explanation ordinal?
binding authority
```

Parameter identity must use a portable callable/parameter identity, not raw process-local ID if `CallableParameterId` is not itself portable.

### Source attachment export

Change the export caller so it supplies exact source site authority, not just declaration ranges.

A portable source-declared attachment must preserve at least:

```text
source owner
source-site local identity
declaration range
```

plus any live identity component required by `SourceSiteId`.

The imported formal projection may be built from the transported declaration range, but validation retains the exact site identity so same-range/name ambiguity cannot be reintroduced.

### `<main>` test

Build or reuse a source fixture with:

- synthetic module body;
- two top-level lambdas;
- both have a parameter named `text`;
- both can use the same native local numeric binding ID in their separate callable owners;
- module preamble/imports/declarations present.

Verify no rebasing/count shortcut aliases them.

### Module values

Validate stable module/symbol/import binding against loaded module authority at the seam available to PATCH1. If full linker-owner validation belongs to a later global import stage, keep the row structurally valid and add the exact later cross-owner validation hook. Do not turn it into a local declaration attachment.

## Forbidden approaches

- source name/range scanning during import;
- treating absent broad source index as “implicit receiver” evidence;
- serializing only range;
- storing origin separately without 1:1 validation if the binding row already owns it.

## Test changes required

CV-07, CV-08, CV-28.

## Tests to run now

Only binding/source attachment focused tests.

## Acceptance

Source and imported binding rows agree modulo relocated IDs, and source-free formal projection returns the exact required binding attachment ranges without consulting broad source index.

## Local STOP / CONSULT triggers

Exact portable `SourceSiteId` cannot be preserved under current identity model.

## Checkpoint update

Record the final binding authority/attachment invariant.

---

# T5 — Implement complete expression rows and generic call specialization evidence

## Purpose

Restore per-expression executable semantic authority and complete generic call environments.

## Preconditions

Bindings can be referenced by stable owner-local ordinals.

## Consumes

- `ExpressionAnalysis`;
- binding ordinals;
- existing denotation codec;
- `CallSpecialization`;
- generic conformance evidence types.

## Produces

- complete expression row;
- full specialization with conformance bindings;
- imported expression indexes using frozen storage;
- precise unsupported handling for `CallResolutionId`.

## Files and symbols

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/artifact/export.rs
phalcom-semantic/src/artifact/import.rs
phalcom-semantic/src/artifact/roots.rs
phalcom-compiler-artifact/src/semantic/world.rs
```

## Required implementation shape

Transport every field listed in §7.8 except fields explicitly fail-closed by approved policy.

### Status and invalidity

Preserve exact variants and payloads:

```text
Ready
Invalid(cause)
Suppressed(...)
Blocked(...)
DynamicBoundary(...)
Cancelled
BudgetExceeded(...)
InternalFailure(...)
```

Do not map them to the coarse callable status enum.

### Binding links

`binding_read` and `binding_write` use owner-local binding ordinals.

Validate referenced binding exists before native construction.

### Call specialization

Replace the current conformance rejection with complete portable evidence.

Each `GenericCallConformanceBinding` contains:

```text
callee_conformance key
source
```

Encode every source variant exactly.

Use existing stable conformance/impl/declaration/type owner codecs where available.

### `CallResolutionId`

Implement only an accepted canonical owner reference.

Otherwise:

```text
nonempty expression.call
    => precise coverage rejection
```

Add a discriminator that proves it cannot be silently dropped.

## Forbidden approaches

- `conformances: Box::default()` on import;
- assumed `TypeId(0)`;
- Dynamic-to-missing conversion;
- re-solving selected callable or trait evidence;
- copying raw `CallResolutionId`.

## Test changes required

CV-09, CV-10, CV-11, CV-16, CV-31.

Include:

```text
Box<Int>
Box<Dynamic>
nested List<T>/Box<T>
row argument
Exact conformance
Forwarded conformance
Derived conformance
CompilerDerived conformance
zero conformance slots for genuinely Dynamic-dependent call
```

## Tests to run now

Expression/specialization focused codec/import tests.

## Acceptance

Every supported expression field round-trips under fresh type numbering and no specialization evidence is dropped.

## Local STOP / CONSULT triggers

Any generic evidence source lacks a stable portable owner/provenance representation.

## Checkpoint update

Only if a new durable portable generic-evidence interface is established.

---

# T6 — Transport flow boundaries and all callable resolution families

## Purpose

Complete compiler-executable callable products beyond the basic expression/binding rows.

## Preconditions

T5 expression rows stable.

## Consumes

- existing frozen flow graph;
- final root closure;
- associated/reference/family/match/iteration product types.

## Produces

- entry/exit/return parity;
- all resolution-family wire rows;
- imported frozen resolution indexes.

## Files and symbols

Read/modify exact live owners discovered in T0, primarily:

```text
checker/analysis.rs
checker/associated.rs
match_semantics/*
trait_dispatch.rs
artifact/export.rs
artifact/import.rs
artifact/roots.rs
compiler-artifact semantic/world.rs
```

## Required implementation shape

### Flow state

Encode and restore:

```text
FlowStateSummary.bindings
FlowStateSummary.fields
fact_count

BodyExitFacts.normal_returns:
    knowledge
    flow
    status
    causal_invalidity
    direct_result_dependency

BodyExitFacts.throws
BodyExitFacts.unreachable
ReturnContractValidation
```

Imported flow maps use charged frozen backing.

### Resolution rows

Each map key must be represented through the correct expression/pattern/site local identity.

Preserve all target/signature/evidence/order/shape fields from the native product.

Do not design reduced “execution summaries” unless a direct consumer proof shows a native field is derived and PATCH0's artifact contract classifies it that way.

### Root closure

After the final row types are in place, run the exhaustive root test again. Treat this as the final root-closure gate for PATCH1.

## Forbidden approaches

- lowering-time re-resolution;
- dropping invalid resolution rows;
- storing only selected callable when native product contains evidence/shape/order needed by consumers;
- using source ranges as map identity.

## Test changes required

CV-12 through CV-16, CV-22.

## Tests to run now

Resolution-family focused tests plus root filter.

## Acceptance

A rich callable containing each supported family can be projected and imported with field parity, even though full callable assembly is not yet enabled.

## Local STOP / CONSULT triggers

A core consumer reads a field the planned portable row does not preserve.

## Checkpoint update

Record completion of callable executable-resolution transport.

---

# T7 — Transport diagnostics, explanation DAG, dependencies, constructor-super, and publication status

## Purpose

Complete causal/debug semantic authority and the callable's query dependency edge facts.

## Preconditions

Expression/binding local refs stable.

## Consumes

- `ExplanationArena`;
- semantic diagnostics;
- diagnostic causes;
- `SemanticDependency`;
- constructor-super resolution.

## Produces

- frozen explanation arena;
- diagnostic/cause closure;
- callable/semantic dependency rows;
- constructor-super row in complete owner;
- canonical Complete/internal-incident preflight.

## Files and symbols

```text
phalcom-semantic/src/explain/arena.rs
phalcom-semantic/src/explain/node.rs
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/artifact/export.rs
phalcom-semantic/src/artifact/import.rs
phalcom-compiler-artifact/src/semantic/world.rs
```

## Required implementation shape

### Explanation arena

Follow the same dual backing as other callable maps:

```text
Source(BTreeMap<ExplanationId, ExplanationNode>)
Frozen(BudgetedSortedMap<ExplanationId, ExplanationNode>)
```

or the accepted equivalent.

Restore `next_id` only for mutable source mode. Imported frozen mode does not need to support allocation of new explanation nodes.

### Causes

Use one diagnostic-cause ordinal map.

Validate all expression/binding/status references before publication.

### Diagnostics

Preserve semantic diagnostic payloads required by callable state and explanation closure.

If a diagnostic field is intentionally presentation-only and PATCH0 classifies it omitted, follow that explicit disposition. Do not decide omissions ad hoc here.

### Dependencies

Encode each `SemanticDependency` variant through stable owner records.

`LinkedName` / export names may carry strings as payload but must not become debug-formatted owner identity.

### Status

Canonical export preflight:

```rust
if !analysis.status.is_complete() { reject }
if !analysis.internal_incidents.is_empty() { reject }
```

Do not synthesize Complete on import.

## Forbidden approaches

- generic replacement failure messages;
- flatten explanation DAG into independent copies;
- omit dependency variants because query reconstruction is later work;
- convert internal incidents into diagnostics and publish anyway.

## Test changes required

CV-17 through CV-21.

## Tests to run now

Focused explanation/diagnostic/dependency tests.

## Acceptance

One callable's causal graph and dependencies survive with exact closure and status behavior.

## Local STOP / CONSULT triggers

A diagnostic/explanation payload owns a type/evidence family absent from the artifact root contract.

## Checkpoint update

Record durable explanation/dependency transport interface if new.

---

# T8 — Assemble the complete transactional callable importer and formal projection contribution

## Purpose

Turn all imported subproducts into one complete immutable `CallableAnalysis`, publish it only after validation/fingerprint recomputation, and prove whole-callable exact allocation behavior.

## Preconditions

T1–T7 complete.

## Consumes

- complete wire record;
- type/static relocation context;
- frozen map/table builders;
- formal attachment rows;
- PATCH0 fingerprint function.

## Produces

- `import_callable_analysis_with_allocation_budget` or equivalent;
- complete imported `CallableAnalysisTable`;
- formal projection attachment input;
- exact-fit/N−1 whole-callable proof;
- no callable partial publication on error.

## Files and symbols

```text
phalcom-semantic/src/artifact/import.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/db/fingerprint.rs
allocation helper code as needed
```

## Required implementation shape

For each callable:

```text
child transaction

1. validate record-level counts
2. validate unique local ordinals
3. validate all local reference closure
4. validate all stable cross-owner syntax/shape available at this stage
5. relocate type/kind/row/parameter/static refs
6. allocate reverse local-ID maps as charged/frozen structures
7. materialize bindings
8. materialize expressions
9. materialize flow graph + entry/exit/return facts
10. materialize resolution families
11. materialize explanation/cause/diagnostic graph
12. materialize callable/semantic dependencies
13. materialize constructor-super relation
14. build immutable CallableAnalysis
15. recompute callable_body_product_fingerprint
16. validate restored owner invariants
17. stage formal attachment contribution
18. commit child transaction
```

No insertion into the outer callable table before step 16 succeeds.

### Partial call-specialization helper

Convert any helper still debiting the parent transaction directly into a child transaction or absorb it into the per-callable child.

Malformed later rows must roll back earlier debit.

### Outer table

After all callable owners validate, build imported `CallableAnalysisTable` through charged frozen backing.

No unbudgeted `HashMap`/`BTreeSet` publication.

### Formal projection

Use imported exact binding attachment rows to create the source-free range map/projection required by current `FormalSemanticProjection`.

The broad `source_index` remains absent.

Do not treat missing attachment as implicit receiver unless binding authority explicitly says generated/implicit.

## Forbidden approaches

- constructing `CallableAnalysis::default()` and filling fields later if that permits temporarily invalid authority;
- inserting partial owners then removing them on error;
- parent-budget direct debit across a fallible multirow import;
- rebuilding source index;
- default empty resolution maps for nonempty wire data.

## Test changes required

CV-01, CV-23, CV-25–CV-28.

### Allocation discriminator

Use an independent allocator observation around the complete callable materializer.

For observed request total `N`:

```text
budget N:
    success
    committed debit == independently observed requested bytes

budget N - 1:
    typed allocation failure
    no unmatched allocator request
    parent ledger unchanged

fresh budget N:
    retry succeeds
```

Also test malformed-later-row rollback.

## Tests to run now

- complete callable importer focused test;
- whole-callable allocation focused test;
- formal projection source-free focused test.

## Acceptance

A complete callable owner can be imported under fresh type numbering without source access, with exact child-transaction accounting and post-relocation fingerprint.

## Local STOP / CONSULT triggers

- any native allocation in the callable owner cannot be charged exactly;
- formal projection still requires broad source index;
- fingerprint parity fails for semantically equivalent relocated owner after PATCH0.

## Checkpoint update

Record first complete imported callable owner and allocation evidence as a coherent verification event.

---

# T9 — Prove compiler-reader parity against source-built callable authority

## Purpose

Demonstrate that restored callable authority is not merely field-complete but behaviorally equivalent to what core lowering consumes.

## Preconditions

T8 complete.

## Consumes

- source-built callable analysis;
- imported callable analysis under fresh store;
- semantic/core lowering readers.

## Produces

- differential consumer parity harness;
- exact lowering-binding parity;
- no-source fallback proof at callable reader boundary.

## Files and symbols

```text
phalcom-core/src/modules/semantic_lowering.rs
relevant focused core tests
phalcom-semantic artifact parity test harness
```

## Required implementation shape

Create a differential test that obtains:

```text
A = source-built SemanticSnapshot callable owner
B = artifact-projected + decoded + fresh-relocated callable owner
```

Compare consumer-visible results rather than native numeric IDs.

At minimum compare:

```text
binding identities
binding read/write attachments
read safety
formal declaration attachments
associated lowerings
callable-reference lowerings
family applications
match/pattern plans
conditional invocation specs
trait invocation specs/candidates
iteration specs
positional expansion specs
sequence literal plans
contextual realizations
constructor-super initializer
runtime generic/conformance recipe inputs
flow/return behavior used by lowering
```

Run the imported side with broad canonical `source_index` absent.

A test may compare generated lowering specs/IR if that is the narrowest stable consumer surface.

Do not require full `.telosc` loader because PATCH3 global allocation guards intentionally remain.

Use the lowest internal artifact/import seam that still performs real wire encode/decode and fresh relocation.

## Forbidden approaches

- comparing only collection lengths;
- comparing only debug output;
- copying source analysis directly into imported test snapshot;
- enabling source fallback in test harness;
- bypassing the real import helper.

## Test changes required

CV-28, CV-29.

## Tests to run now

Only the new differential parity target plus directly affected lowering tests.

## Acceptance

Source and imported callable authority generate equivalent lowering-reader products with no broad source index.

## Local STOP / CONSULT triggers

Any lowering consumer needs canonical source AST to interpret a restored callable fact.

## Checkpoint update

Record G6 parity evidence once passed.

---

# T10 — Remove only the callable-specific fail-closed guards and finalize PATCH1

## Purpose

Promote the now-proven callable owner contract while preserving all independent global/other-family barriers.

## Preconditions

All mandatory PATCH1 gates pass.

## Consumes

- guard-removal matrix;
- verification ledger.

## Produces

- callable-analysis branch removed from export coverage rejection;
- `ensure_complete_callable_analysis_payload` removed/replaced by real complete owner validation/import;
- next independent guard remains visible;
- walkthrough and handoff;
- checkpoint truth updated.

## Files and symbols

```text
phalcom-semantic/src/artifact/export.rs
phalcom-semantic/src/artifact/import.rs
checkpoint + P7 records
```

## Required implementation shape

Remove only the guards whose safety contracts PATCH1 now implements.

Expected:

```text
validate_semantic_world_export_coverage:
    remove callable-analysis "not encoded" rejection
    retain every other unsupported owner-family rejection

ensure_complete_callable_analysis_payload:
    delete obsolete blanket rejection
    replace with/route through complete callable validator/materializer

global:
    keep require_semantic_validation_allocation_accounting()
    keep require_semantic_import_allocation_accounting()
```

Add/adjust a frontier test that proves the callable stage is now accepted and the full pipeline, if still blocked, fails at the **next expected independent** guard.

Do not treat that downstream failure as PATCH1 failure.

## Forbidden approaches

- remove all export coverage guards;
- weaken `has_source_free_artifact_contents`;
- mark P7 complete;
- claim full artifact load;
- remove global `UnaccountedBoundary`.

## Test changes required

CV-30.

## Tests to run now

Run the PATCH1 stabilization gates in §19 and final focused acceptance in §20.

## Acceptance

Callable persistence is independently complete and no longer blocked by its old blanket guards; remaining P7 blockers are truthful and separate.

## Local STOP / CONSULT triggers

A downstream guard can only be reached by weakening a PATCH1 invariant.

## Checkpoint update

At completion:

- mark PATCH1 COMPLETE / IMPLEMENTED;
- set verification to the strongest evidence actually achieved, normally `FOCUSED_TESTED`;
- P7 remains `IN_PROGRESS / PARTIAL / UNVERIFIED` unless broader independent plans have also completed;
- set next action to PATCH2/PATCH3 according to live program sequencing.

---

# 19. Verification gates

# G1 — Frozen callable storage gate

**Purpose:** prove INV-14 through INV-17 foundations.

Run the exact focused tests added for:

```text
source/frozen map read parity
CallableAnalysisTable source/frozen parity
module-bounded iteration
frozen immutability
```

Expected:

```text
PASS
no shadow native imported map
no observable source reader change
```

Do not broaden after PASS.

---

# G2 — Owner-local identity and codec gate

**Purpose:** prove INV-02, INV-03, INV-21.

Run:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-compiler-artifact \
  --test semantic_world_callable_analysis_test -- --test-threads=1
```

plus the exact new semantic ordinal determinism filter.

Expected:

```text
rich record round trip PASS
duplicate ordinals rejected
wrong-owner refs rejected
overlapping local numbers across owners remain distinct
unsupported tags rejected
```

---

# G3 — Final root closure gate

**Purpose:** prove INV-04 against the final schema.

Run the current focused callable-analysis root filter. Use exact discovery and ensure nonzero selection.

Expected:

```text
every final portable type-bearing family closes the shared graph
no row projection discovers a missing root
```

A PASS against only the pre-PATCH1 root families is insufficient.

---

# G4 — Binding/expression/generic evidence parity gate

**Purpose:** prove INV-06 through INV-10.

Run focused semantic artifact tests covering:

```text
binding rows
source attachment
synthetic <main>
expression read/write
Dynamic
all four generic conformance-source variants
```

Expected: source/import field parity modulo relocated IDs.

---

# G5 — Complete callable reconstruction and allocation gate

**Purpose:** prove INV-11 through INV-16.

Run the complete callable owner import test and whole-callable independent counting allocator test.

Expected:

```text
fresh-store import PASS
fingerprint recomputed
exact N PASS
N−1 typed failure
parent rollback
retry PASS
```

Do not use the global full semantic loader for this gate if PATCH3 still blocks it.

---

# G6 — Compiler-reader differential gate

**Purpose:** prove INV-05 and INV-14.

Run the focused source-vs-imported semantic lowering test.

Expected:

```text
same consumer-level binding/dispatch/match/iteration/family outputs
broad canonical source_index absent on imported side
no source lookup
```

---

# G7 — Callable guard-removal frontier

**Purpose:** prove INV-18 and prevent overclaim.

Run:

1. callable export/import focused tests;
2. the narrow canonical artifact test only if it reaches the intended coverage frontier without requiring unrelated repairs.

Expected:

```text
callable-specific coverage no longer rejects
next independent unsupported owner/global allocation guard remains fail-closed
typed/intended frontier asserted
```

A generic `is_err()` is not acceptable evidence.

---

# 20. Final focused acceptance

PATCH1 is complete only when every required coverage obligation is mapped to passing evidence or an explicitly approved non-applicable disposition.

Required final ledger:

| Coverage | Evidence required |
|---|---|
| CV-01 | complete rich callable source→wire→fresh import parity |
| CV-02 | overlapping local IDs across two callables |
| CV-05/06 | duplicate/wrong-owner rejection |
| CV-07/08 | explicit binding authority + synthetic `<main>` |
| CV-09 | complete expression fields |
| CV-10/11 | Dynamic + all generic conformance variants |
| CV-12 | entry/returns/throws/unreachable/return validation |
| CV-13–16 | all compiler resolution families |
| CV-17/18 | explanation/diagnostic closure |
| CV-19 | callable + semantic dependencies |
| CV-20 | constructor-super |
| CV-21 | canonical status/incidents |
| CV-22 | final root closure |
| CV-23 | fresh type-store relocation |
| CV-24 | deterministic local wire identity |
| CV-25/26 | frozen reader + module-locality parity |
| CV-27 | exact allocation N/N−1/rollback |
| CV-28 | formal projection without broad source index |
| CV-29 | core lowering parity |
| CV-30 | callable guards removed, global guards retained |
| CV-31 | unsupported `CallResolutionId` fails precisely if still unsupported |
| CV-32 | no callable-owned checked static application |

Do not automatically run workspace certification.

PATCH1's normal final lifecycle is:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

unless the exact evidence supports something stronger.

It is valid for `PERF003.C2.P7` itself to remain:

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
```

because PATCH2–PATCH7 remain.

---

# 21. Performance/resource evidence

PATCH1 has a real allocation and artifact-size concern, so lightweight telemetry is required.

Record for representative callable owners:

```text
wire bytes / callable
expression rows
binding rows
flow node/edge rows
resolution rows
generic conformance rows
diagnostic/explanation bytes
dependencies
retained frozen callable allocation
temporary validation/import allocation
```

For at least one rich callable and one canonical Universe sample, record:

```text
source native owner approximate retained shape
encoded callable bytes
import child transaction requested bytes
retained frozen bytes
```

Do not claim a speedup from PATCH1 alone.

Resource acceptance:

- no obvious accidental quadratic validation over owner-local families;
- no duplicate source+frozen imported map;
- lookup complexity remains appropriate;
- section-size growth is visible in the walkthrough.

If callable section size explodes because large signatures/evidence are duplicated instead of referenced through canonical owners, treat that as a design problem before guard removal.

---

# 22. Checkpoint bookkeeping

## 22.1 At plan start

Update the canonical `PERF003.C2/CHECKPOINT.md`:

- PATCH1 active under P7;
- starting revision;
- PATCH0 prerequisite status;
- inherited branch/baseline facts.

Do not create another checkpoint-state file.

## 22.2 During plan

Update only at meaningful durable events:

1. frozen callable owner storage API established;
2. final callable wire schema established;
3. exact binding/formal attachment identity established;
4. complete imported callable owner first passes;
5. source/import compiler parity gate passes;
6. architectural consultation/amendment;
7. significant deferred failure;
8. callable guards removed.

Do not log every test invocation.

## 22.3 At plan completion

Record:

- PATCH1 status/completion/verification;
- final invariants;
- gate evidence;
- deferred/baseline failures;
- consultations;
- residual P7 blockers;
- next action.

---

# 23. Walkthrough deliverable

Create:

```text
PERF003.C2.P7.PATCH1-walkthrough.md
```

Place it according to the live P7 implementation-record convention.

It must contain:

1. plan identity/final revision;
2. final result;
3. architecture implemented;
4. final wire shape;
5. frozen storage design;
6. binding/source-site design;
7. complete field disposition;
8. important source files changed;
9. deviations from this plan;
10. consultations and decisions;
11. tests added;
12. tests actually run;
13. tests deliberately deferred;
14. exact allocation evidence;
15. artifact-size/resource observations;
16. guard changes;
17. residual risks;
18. P7 state after PATCH1.

Never claim a test passed unless it actually ran on the recorded revision.

---

# 24. Handoff deliverable

Create:

```text
PERF003.C2.P7.PATCH1-handoff.md
```

Required content:

```text
current revision
working-tree state
completed PATCH1 prerequisites
stable callable wire interface
stable frozen callable storage interface
binding/formal attachment model
owner-local identity rules
remaining fail-closed fields
global guards still present
invariants next implementer must not redesign
deferred failures
must-read files
next patch objective
first recommended commands
explicit do-not-reexplore guidance
```

Explicitly state:

```text
CheckedStaticBody remains StaticApplication owner.
Do not duplicate checked rows into CallableAnalysis.
Do not remove global allocation UnaccountedBoundary.
Do not reconstruct canonical source index.
Do not reopen PATCH1 owner identity without a recorded incident.
```

---

# 25. Completion truth table

At finalization, distinguish these states:

| Claim | Meaning |
|---|---|
| source written | code exists |
| tests added | tests exist, not necessarily run |
| targeted tests passed | exact listed focused commands passed |
| callable owner implemented | complete source/wire/import/consumer architecture exists |
| callable guards removed | only PATCH1 guards removed after proof |
| semantic loader works | **not established by PATCH1** |
| full `.telosc` builds/loads | **not established by PATCH1** |
| P7 checkpoint accepted | **not established by PATCH1** |
| P8 takeover authorized | **not established by PATCH1** |
| release certified | **not established by PATCH1** |

Do not conflate them in the walkthrough/checkpoint.

---

# 26. Plan self-review

The architect has checked this plan against the supplied Luna schema, lifecycle convention, principal-architect doctrine, consolidated PATCH1 architecture, six audits, and live `perf/universe-characterization` state.

## Architecture

- [x] `CallableAnalysis` has one clear authority.
- [x] expression/binding/source identity is explicit.
- [x] checked static applications remain outside the callable owner.
- [x] no compiler consumer is instructed to reconstruct semantic authority.
- [x] imported immutability and exact allocation shape are explicit.
- [x] post-relocation fingerprint ownership is explicit.
- [x] guard removal boundary is explicit.

## Luna executability

- [x] fixed decisions are separated from mechanical flexibility.
- [x] live assumptions are verify-first.
- [x] STOP/CONSULT triggers are objective.
- [x] debugging loops are bounded.
- [x] each task has a coherent consumed/produced interface.
- [x] intermediate incomplete states are explicitly allowed.

## Testing

- [x] coverage is designed broadly.
- [x] execution is narrow.
- [x] every important semantic risk has a discriminator.
- [x] allocation is independently measured.
- [x] broad workspace tests are deferred.
- [x] downstream fail-closed guards are not mistaken for PATCH1 failures.

## Lifecycle

- [x] checkpoint updates are mandatory.
- [x] walkthrough required.
- [x] handoff required.
- [x] no competing implementation-state file is planned.

## Residual architectural risks intentionally surfaced

These are the highest-value possible consultation points:

1. portable formal `SourceSiteId` representation;
2. `ExpressionAnalysis.call` portable owner, if canonical bodies actually require it;
3. artifact record/schema versioning if current policy cannot evolve the callable record locally;
4. stable deterministic ordinal assignment if native local IDs prove allocation-history-sensitive;
5. any nested generic evidence/map structure that cannot satisfy exact charged frozen storage without changing owner semantics.

None should be solved by ad hoc fallback.

---

# Appendix A — Native-to-portable callable field checklist

Use this during implementation and copy the final disposition into the walkthrough.

| Native owner field | PATCH1 disposition | Required proof |
|---|---|---|
| `callable` | wire stable owner | exact stable round trip |
| `body_range` | wire | exact span |
| `body_source_owner` | wire | module/callable owner parity |
| `expressions` | dense wire rows | full field parity |
| `bindings` | dense wire rows | full field parity |
| `binding_origins` | fold into binding authority preferred | exact storage parity |
| `for_iteration_lanes` | wire | lowering parity |
| `positional_expansions` | wire | lowering parity |
| `known_invalid_positional_expansions` | wire | fail-closed lowering parity |
| `sequence_literals` | wire | construction parity |
| `associated_resolutions` | wire | target/evidence parity |
| `callable_reference_resolutions` | wire | callable/family parity |
| `family_applications` | wire | target/spec parity |
| `match_resolutions` | wire | executable plan parity |
| `pattern_site_resolutions` | wire | binding/site parity |
| `flow_graph` | existing wire + frozen import | source/frozen parity |
| `entry_flow` | wire | field/binding flow parity |
| `exits.normal_returns` | wire | knowledge/flow/status parity |
| `exits.throws` | wire | flow parity |
| `exits.unreachable` | wire | exact |
| `return_validation` | wire | exact |
| `diagnostics` | wire per PATCH0 contract | cause/explanation parity |
| `internal_incidents` | omitted-by-policy; canonical requires empty | publication rejection |
| `explanations` | wire DAG | sharing/closure |
| `return_explanation` | wire local ref | closure |
| `dependencies` | wire stable callables | exact |
| `semantic_dependencies` | wire stable owners | exact |
| `constructor_super_initializer` | wire | range/target parity |
| `dependency_fingerprint` | derived after relocation | recomputation |
| `status` | wire; canonical requires Complete | publication gate |

---

# Appendix B — Expression checklist

| `ExpressionAnalysis` field | Portable rule |
|---|---|
| `id` | owner-local expression ordinal |
| `range` | span |
| `knowledge` | relocated typed evidence |
| `callable` | stable callable |
| `call_specialization` | full static args + conformances |
| `conditional_dispatch` | exact checked product |
| `conditional_capability` | exact proof-bearing product |
| `trait_dispatch` | exact site selection |
| `trait_dispatch_candidates` | ordered candidate provenance |
| `contextual_realization` | exact checked operation/evidence |
| `denotation` | existing/extended denotation codec; unsupported owners fail closed |
| `status` | exact expression status payload |
| `causal_invalidity` | exact cause-local representation |
| `explanation` | explanation ordinal |
| `call` | accepted owner ref or precise fail-closed |
| `binding_read_safety` | exact |
| `binding_read` | binding ordinal |
| `binding_write` | binding ordinal |

---

# Appendix C — Binding checklist

| `BindingState` field | Portable rule |
|---|---|
| `binding` | binding ordinal |
| `name` | owned string, charged on import |
| `parameter` | portable callable parameter identity |
| `range` | body/local span |
| `contract` | exact type + origin + source relation required by semantics |
| `current` | exact `TypeKnowledge` |
| `denotation` | exact supported semantic denotation |
| `consistency` | exact |
| `causal_invalidity` | cause-local representation |
| `mutable` | exact |
| `initialization` | exact |
| `version` | exact |
| `explanation` | explanation ordinal |
| storage/source authority | folded explicit authority enum |

---

# Appendix D — Guard-removal checklist

Before editing either callable guard, all boxes must be true on the same recorded revision:

```text
[ ] complete field matrix has no unintended Missing/Partial required field
[ ] final root closure passes
[ ] full wire codec passes
[ ] two-owner local-ID isolation passes
[ ] bindings/formal attachment parity passes
[ ] expression parity passes
[ ] all generic conformance-source variants pass
[ ] all resolution-family parity passes
[ ] flow/return parity passes
[ ] explanation/diagnostic closure passes
[ ] dependency parity passes
[ ] constructor-super parity passes
[ ] fresh-store fingerprint recomputation passes
[ ] deterministic portable ordering passes
[ ] source/frozen read parity passes
[ ] module-bounded iteration passes
[ ] whole-callable exact N/N−1/rollback allocation passes
[ ] formal projection passes with broad source_index absent
[ ] source/import core lowering differential passes
[ ] canonical status/internal-incident publication gate passes
[ ] unsupported CallResolutionId behavior is explicit
[ ] no StaticApplication row was duplicated into CallableAnalysis
```

Then and only then:

```text
[ ] remove callable branch of validate_semantic_world_export_coverage
[ ] remove/replace ensure_complete_callable_analysis_payload blanket rejection
[ ] retain all other family guards
[ ] retain global semantic validation UnaccountedBoundary
[ ] retain global semantic import UnaccountedBoundary
[ ] assert next intended frontier
```

---

# Appendix E — Explicit forbidden shortcuts

The following are plan violations:

```text
Default::default() for missing compiler authority
TypeId(0) placeholder
empty conformance array for nonempty source conformance evidence
range/name lookup to recreate binding identity
absence from source_index => implicit receiver
native BindingId/ExpressionId written to artifact as global identity
one ordinal allocator per subfamily
new type root discovered during DTO projection
ordinary BTreeMap/HashMap imported shadow copies
guessed BTreeMap node allocation debit
source syntax lookup during imported lowering
rerun generic inference or trait dispatch on load
drop invalidity/diagnostic/explanation facts
adopt wire dependency_fingerprint without recomputation
copy CheckedStaticApplication into CallableAnalysis
remove global allocation guards
weaken snapshot completeness
make full-artifact test green by deleting guards
run workspace tests after every task
repair unrelated baseline failures
```

---

# Appendix F — Recommended final implementation sequence summary

```text
T0  live truth + PATCH0 prerequisite
 ↓
T1  frozen callable storage/read APIs
 ↓
T2  single local-ordinal + closure model
 ↓
T3  owner-centric wire + final roots
 ↓
T4  binding state + exact formal/storage authority
 ↓
T5  expressions + full generic specialization
 ↓
T6  flow boundaries + resolution families
 ↓
T7  diagnostics/explanations/dependencies/status
 ↓
T8  full transactional native importer + formal projection
 ↓
T9  compiler-reader differential parity
 ↓
T10 callable-only guard removal + lifecycle closure
```

The global semantic load remains fail-closed after T10 until PATCH3 proves exact allocation closure for the entire semantic import graph.
