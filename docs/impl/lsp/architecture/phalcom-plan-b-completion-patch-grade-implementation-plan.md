# Phalcom LSP Module Architecture — Plan B Completion & Certification
## Patch-Grade Implementation Plan

**Status:** Ready for implementation  
**Scope:** Complete Plan B without redesigning the already-landed architecture  
**Repository:** `aureat/phalcom-lang`  
**Audited baseline:** `94b9f14361333dfda06cd8abd792f672d12db06a` — `Repair ADT and Plan B semantic indexing tests`  
**Preceding Plan B landing:** `018fbe2e9bb6578d2852cf77647eac1432635a6b` — `Implement Plan B semantic indexing and LSP integration`

---

# 1. Purpose

This plan closes the remaining gaps in the Phalcom LSP Module Architecture Plan B implementation.

The repository already contains the central Plan B architecture:

- persistent source-index roots;
- module-owned source shards;
- module-owned reference contributions;
- persistent reverse reference index;
- distinct lexical and semantic reference domains;
- imported-binding provenance;
- persistent formal projection roots;
- incremental workspace-symbol index;
- compiler-owned editor queries;
- LSP location mapping over immutable semantic snapshots;
- batched source-shard construction with all formal callable attachments;
- presentation-only contribution retention;
- exact import-prefix occurrence projection;
- substantial enum/type-reference coverage;
- direct editor-query cutover for definitions, references, rename preparation, and workspace symbols.

This completion patch must **not** replace that architecture.

Instead, it must repair the remaining places where the implementation does not yet satisfy Plan B's own invariants, complexity guarantees, lifecycle semantics, or acceptance requirements.

The unresolved issues are:

1. PB-12 scan counters exist but are not genuinely instrumented.
2. Source-index publication still performs workspace-sized scans for reuse/retirement accounting.
3. Plan-A's retained importer → `ImportSiteId` index is not threaded across the semantic boundary, so source indexing filters all import products.
4. Formal projection rebuilds one module by scanning all callable analyses.
5. Plan B's required LSP latency/work counters and dedicated LSP acceptance suite are missing.
6. `OccurrenceIndex` still retains a duplicate target reverse index after the `ReferenceIndex` cutover.
7. source attachment incidents are global append-only state instead of module-owned replaceable contributions.
8. reference added/removed counters use length deltas rather than actual set differences.
9. workspace symbols cannot distinguish classes, enums, and type aliases at the declaration-source layer.
10. class index-accessor generic/`where` type-reference coverage is incomplete.
11. several Plan B acceptance tests are too weak or do not exercise the actual production path.
12. residual broad publication loops must be classified before Plan B can make its final no-hidden-workspace-tax claim.

---

# 2. Completion Definition

Plan B is complete only when all of the following are true.

## 2.1 Correctness

- Definitions, lexical references, and semantic references remain distinct.
- Imported aliases preserve both local lexical identity and upstream semantic identity.
- Import path segments use exact `ResolvedImportPrefix` identities.
- Deleted or replaced source contributions disappear from all derived indexes.
- Source attachment incidents obey module contribution lifetime.
- Workspace symbols preserve declaration kind correctly.
- Cold and incrementally updated snapshots produce equivalent editor-visible semantic products.
- Old immutable snapshots remain fully queryable after later edits, retargeting, renames, and deletions.

## 2.2 Incrementality

For an ordinary incremental publication:

- source work is proportional to exact rebuilt/retired modules;
- import-product work is proportional to imports owned by rebuilt modules;
- formal projection work is proportional to callables owned by rebuilt modules;
- reference work is proportional to semantic targets touched by changed module contributions;
- workspace-symbol replacement is proportional to symbols in changed modules;
- no source, reference, or formal whole-workspace traversal occurs merely to discover the delta;
- no instrumentation counter itself introduces a whole-workspace scan.

## 2.3 LSP latency

Definition/reference/workspace-symbol/diagnostic adaptation:

- consumes compiler-owned indexed products;
- does not scan source shards to rediscover semantic candidates;
- builds line indexes only for modules that actually need location conversion;
- performs no workspace-wide diagnostic source scan;
- exposes deterministic work counters that prove those properties.

## 2.4 Acceptance evidence

PB-1 through PB-12 must pass with tests that exercise the **production architecture**, not compatibility fallbacks or zero-valued unused counters.

---

# 3. Architectural Invariants

## PB-INV-1 — No editor re-resolution

The source/editor layer publishes identities already established by compiler/module products. It must not reconstruct import meaning or semantic identity from source spelling when a canonical compiler product exists.

## PB-INV-2 — One source contribution per module

A module owns one replaceable `ModuleSourceIndex`. A rebuild computes the final module shard first and publishes it once.

## PB-INV-3 — Immutable snapshots

Published semantic snapshots and all indexed products remain immutable. Incremental updates create new persistent roots and retain unaffected `Arc`s.

## PB-INV-4 — Snapshot-local source identities

`SourceSiteId` remains valid only within the semantic snapshot/source contribution that owns it.

## PB-INV-5 — Presentation movement does not change meaning

Whitespace/comment/range-only edits may rebuild a source shard while retaining semantic reference and workspace-symbol contributions when their semantic contents are equal.

## PB-INV-6 — Reference domains remain distinct

Maintain separate definitions, lexical references, and semantic references.

## PB-INV-7 — Imported aliases preserve dual identity

For `from provider import Foo as Bar`, `Bar` uses are lexical references to the local binding and semantic references to upstream `Foo`.

## PB-INV-8 — Exact target only

Hints must never become canonical `SemanticTargetId`s.

## PB-INV-9 — No total workspace traversal on ordinary publication

A normal incremental update must not scan every source module, import product, target reference set, formal callable analysis, source shard, or diagnostic source merely to discover work already known by Plan A/Plan B.

## PB-INV-10 — High fanout is bounded by actual semantic impact

A provider body-only edit must not rebuild consumer source shards or mutate their reference contributions when source-facing semantic identity is unchanged.

## PB-INV-11 — Contribution lifetime defines derived-product lifetime

Replacing or retiring a module source contribution must replace/retire references, workspace symbols, source attachment incidents, formal source projection, and source-local presentation metadata.

## PB-INV-12 — LSP remains an adapter

The LSP layer translates compiler-owned semantic products into protocol types. It must not recreate compiler semantic search/index behavior.

---

# 4. Non-Goals

This patch must **not**:

- redesign Plan A module stabilization;
- replace `im::OrdMap`;
- change `SourceSiteId` identity semantics;
- redesign the semantic query database;
- redesign type checking or advisory analysis;
- add speculative LSP semantic caches outside immutable snapshots;
- introduce string/name fallback as a correctness repair;
- remove useful standalone test helpers merely because production must use exact products;
- optimize unrelated semantic phases unless a broad loop directly affects the Plan B completion claim.

Broad loops outside Plan B should be classified and documented rather than opportunistically rewritten.

---

# 5. Implementation Sequence

Implement in this order:

1. **PB-C0 — Make Plan B performance evidence real**
2. **PB-C1 — Remove residual source-index workspace scans**
3. **PB-C2 — Thread exact importer → `ImportSiteId` products**
4. **PB-C3 — Make formal projection module-local**
5. **PB-C4 — Repair contribution lifetime and reverse-index accounting**
6. **PB-C5 — Close source-index functional coverage gaps**
7. **PB-C6 — Complete LSP latency instrumentation and acceptance**
8. **PB-C7 — Strengthen PB-1…PB-12 acceptance tests**
9. **PB-C8 — Final broad-loop audit, verification, and completion ledger**

Do not reorder PB-C0 after the optimizations. The tests must first become capable of detecting the existing violations.

---

# 6. PB-C0 — Real Plan B Work/Scan Instrumentation

## Goal

Convert Plan B's scan counters from inert fields into trustworthy deterministic measurements.

## Primary files

```text
phalcom-semantic/src/source_index/mod.rs
phalcom-semantic/src/source_index/reference.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/presentation.rs
phalcom-semantic/tests/semantic/incremental/plan_b_indexing.rs
```

## Required changes

### C0.1 Define exact counter semantics

Document:

```rust
pub source_workspace_scan_units: usize;
pub reference_workspace_scan_units: usize;
pub formal_workspace_scan_units: usize;
```

Recommended meaning:

- `source_workspace_scan_units`: retained source/import units inspected by broad source-publication discovery.
- `reference_workspace_scan_units`: global reverse-reference units inspected by broad traversal instead of exact touched targets.
- `formal_workspace_scan_units`: callable/formal products inspected solely to discover products owned by a rebuilt module.

Exact worklists do **not** increment them.

### C0.2 Instrument current broad source scans before removing them

Instrument:

- reuse counting by enumerating all source-index modules;
- retirement detection by enumerating previous source-index modules;
- full import-product filtering used to discover imports belonging to changed modules;
- any other full retained source pass used only to discover delta work.

### C0.3 Instrument formal all-analysis discovery

The current `build_module_projection` filtering over `analyses.values()` must register formal scan work before PB-C3 removes it.

### C0.4 Reference instrumentation

`ReferenceIndex::replace_module_contribution` should normally report zero broad reference scans because it operates on exact touched targets. Any global rebuild/helper must increment `reference_workspace_scan_units`.

### C0.5 Add instrumentation unit coverage

Add a test proving a deliberately broad path or instrumentation helper produces a nonzero counter. PB-12 must not merely assert default-zero fields.

## Acceptance

- every scan field has explicit semantics;
- instrumentation can demonstrably produce nonzero counts;
- known broad paths become observable before optimization;
- counters do not themselves perform broad scans.

---

# 7. PB-C1 — Remove Residual Source-Index Workspace Scans

## Goal

Make incremental source publication operate exclusively from exact rebuild/retirement worklists.

## Primary files

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/source_index/mod.rs
phalcom-semantic/tests/semantic/incremental/plan_b_indexing.rs
```

## Required changes

### C1.1 Introduce explicit source-index delta

```rust
#[derive(Clone, Debug, Default)]
struct SourceIndexDelta {
    rebuild: BTreeSet<ModuleId>,
    retired: BTreeSet<ModuleId>,
    presentation_rebuild: BTreeSet<ModuleId>,
}
```

Use equivalent naming if needed.

### C1.2 Retire from exact removal set

Replace discovery by `previous.module_ids()` with:

```rust
for module in &delta.retired {
    index.retire_module_shard(module);
}
```

Use Plan-A/semantic exact `removed_modules`.

### C1.3 Do not scan to compute reuse metrics

Do not calculate `source_modules_reused` via `module_ids().filter(...)`.

Derive it arithmetically from persistent cardinality and exact rebuilt/retired work, or redefine it so it can be maintained without total traversal.

### C1.4 Avoid total `current_index_modules` construction for retirement discovery

If presentation-source membership needs a set, keep that separate from ordinary retirement detection.

### C1.5 Preserve cold-build behavior

Cold publication may enumerate all source modules because the entire workspace is the worklist.

Document:

```text
cold build: O(workspace) expected
incremental build: O(exact delta + exact affected work)
```

## Tests

Add/strengthen:

- PB-1 presentation-only edit;
- PB-9a high-fanout body-only edit;
- one-module deletion in large workspace.

Assert real `source_workspace_scan_units == 0`.

---

# 8. PB-C2 — Exact Plan-A Import-Site Worklists

## Goal

Consume Plan A's retained importer-owned import-site products instead of scanning all import products.

## Primary files

```text
phalcom-modules/src/session.rs
phalcom-semantic/src/workspace.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/tests/semantic/incremental/plan_b_indexing.rs
phalcom-semantic/tests/semantic/integration/imported_resolution.rs
phalcom-semantic/tests/semantic/integration/source_index.rs
```

## Required changes

### C2.1 Publish importer → site ownership across the semantic boundary

Thread the existing Plan-A `sites_by_importer` product into `SemanticWorkspaceInput`.

Preferred shape:

```rust
pub import_sites_by_module:
    Arc<BTreeMap<ModuleId, BTreeSet<ImportSiteId>>>,
```

Add a builder/helper.

Prefer retaining/sharing the Plan-A map rather than cloning the whole ownership map each publication.

### C2.2 Build changed import context by exact lookup

Replace:

```rust
import_products
    .iter()
    .filter(|(site, _)| rebuild_modules.contains(&site.importer))
```

with exact per-module site lookup:

```rust
for module in rebuild_modules {
    for site in import_sites_by_module.get(module).into_iter().flatten() {
        if let Some(product) = import_products.get(site) {
            context.import_products.insert(site.clone(), product.clone());
        }
    }
}
```

### C2.3 Separate production exact-product mode from compatibility fallback

`SourceIndexContext` may retain legacy path maps for isolated low-level tests, but production source publication must fail closed when an expected canonical `ImportResolutionProduct` is missing.

Do not silently re-resolve by source path in production.

### C2.4 Preserve `ResolvedImportPrefix`

Keep exact path-segment mapping:

```text
a     → Module(a)
b     → Module(a.b)
c     → Module(a.b.c)
```

### C2.5 Audit every dependency form

Verify exact identity for:

- module import;
- selective import;
- re-export;
- expose.

## Tests

Add at least one fixture built through real `WorkspaceModuleSession` resolution, not only a manually assembled `LinkedProgram`.

Assert actual `ImportSiteId`/`ImportResolutionProduct` consumption and imported alias dual identity.

---

# 9. PB-C3 — Module-Local Formal Projection

## Goal

Remove the O(all callables) scan performed while rebuilding one module's formal projection.

## Primary files

```text
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/session.rs
phalcom-semantic/tests/semantic/incremental/plan_b_indexing.rs
```

## Required changes

### C3.1 Change `build_module_projection`

Preferred API:

```rust
pub fn build_module_projection<'a>(
    module: &ModuleId,
    callable_ids: impl IntoIterator<Item = &'a CallableId>,
    analyses: &HashMap<CallableId, Arc<CallableAnalysis>>,
    source_index: Option<&SourceSemanticIndex>,
) -> ModuleFormalProjection
```

### C3.2 Exact lookup only

```rust
for callable in callable_ids {
    let Some(analysis) = analyses.get(callable) else {
        continue;
    };
    debug_assert_eq!(analysis.callable.module(), module);
    // project facts
}
```

Do not filter `analyses.values()`.

### C3.3 Reuse existing per-module callable worklists

The session already derives callable IDs from semantic structure shards. Reuse that exact product.

### C3.4 Fix cold construction complexity

Cold build should be approximately:

```text
O(modules + total callables)
```

not:

```text
O(modules × total callables)
```

### C3.5 Do not scan all modules for formal reuse statistics

Derive metrics from known cardinality and exact rebuild/retirement work.

## Tests

Large workspace, one module edit:

```text
formal_modules_rebuilt == 1
formal_workspace_scan_units == 0
unrelated ModuleFormalProjection Arcs retained
```

---

# 10. PB-C4 — Contribution Lifetime, Reverse-Index Cleanup, Accurate Accounting

## Primary files

```text
phalcom-semantic/src/source_index/mod.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/source_index/reference.rs
phalcom-semantic/src/source_index/symbol.rs
phalcom-semantic/tests/semantic/integration/source_index.rs
phalcom-semantic/tests/semantic/incremental/plan_b_indexing.rs
```

## C4.1 Remove duplicate occurrence reverse target map

Remove after caller migration:

```rust
target_occurrences:
    BTreeMap<SemanticTargetId, Arc<[SourceSiteId]>>
```

and `OccurrenceIndex::occurrences_for_target`.

Workspace reverse lookup remains owned by `ReferenceIndex`.

## C4.2 Make attachment incidents module-owned

Replace flat:

```rust
pub incidents: Arc<[SourceAttachmentError]>
```

with:

```rust
incidents_by_module:
    im::OrdMap<ModuleId, Arc<[SourceAttachmentError]>>
```

Add replace/retire APIs.

Semantics:

```text
rebuilt module → replace its incident contribution
successful rebuild → old incidents disappear
retired module → incidents disappear
old snapshot → keeps historical incidents
```

## C4.3 Count true reference site deltas

Replace length-only accounting with a sorted two-way difference.

For each definitions/lexical/semantic slice, calculate actual:

```text
old \ new
new \ old
```

Test equal-cardinality replacement:

```text
[A] → [B]
removed == 1
added == 1
```

## C4.4 Preserve contribution `Arc`s when contents are equal

Lock in:

```text
equal reference contribution → retain old Arc, no target update
equal symbol contribution → retain old Arc, no symbol update
```

with focused presentation-only tests.

---

# 11. PB-C5 — Functional Source-Index Coverage Closure

## Primary files

```text
phalcom-semantic/src/source_index/scope.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/mod.rs
phalcom-semantic/src/source_index/symbol.rs
phalcom-semantic/tests/semantic/integration/source_index.rs
```

## C5.1 Preserve declaration kind

Add:

```rust
pub enum SourceDeclarationKind {
    Class,
    Enum,
    TypeAlias,
}
```

Store it in `DeclarationSourceInfo`.

Map into `EditorSymbolKind` rather than emitting every declaration as `Class`.

Test class/enum/type alias/variant workspace symbol kinds.

## C5.2 Complete generic class-index type-reference traversal

For `ClassMember::Index(index)`:

1. clone class bound;
2. add `index.generic_parameters`;
3. visit parameter annotations under the index-local bound;
4. visit setter value annotation;
5. visit return annotation;
6. visit `index.where_clause`.

Mirror enum index behavior.

Test index-local generic shadowing and nominal references in bounds/application/return types.

## C5.3 Lock enum/type-bearing closure

Add a table-driven coverage fixture for:

- enum `where`;
- variant `where`;
- payload annotation;
- GADT result annotation;
- enum-root behavior;
- variant-owned behavior;
- getter;
- setter;
- generic index getter/setter.

---

# 12. PB-C6 — Complete LSP Latency Instrumentation and Acceptance

## Primary files

```text
phalcom-lsp/src/perf.rs
phalcom-lsp/src/backend.rs
phalcom-lsp/src/diagnostics.rs
phalcom-lsp/tests/plan_b_indexing.rs   # new
phalcom-lsp/tests/imported_binding_resolution.rs
phalcom-lsp/tests/module_navigation.rs
phalcom-lsp/tests/performance.rs
```

## Required counters

Add:

```rust
pub reference_source_modules_converted: AtomicU64,
pub reference_line_indexes_built: AtomicU64,
pub reference_duplicate_filter_steps: AtomicU64,
pub workspace_symbol_source_shard_scans: AtomicU64,
pub diagnostic_workspace_source_scans: AtomicU64,
```

Wire into reset/snapshot/serde/unit tests.

## Semantics

### `reference_source_modules_converted`

Distinct modules whose sites are converted for one reference/definition request.

### `reference_line_indexes_built`

Increment only when `SnapshotLocationMapper` builds a new `LineIndex` for a module.

### `reference_duplicate_filter_steps`

Count LSP-side duplicate elimination. Target is zero when compiler reference slices are already deduplicated.

### `workspace_symbol_source_shard_scans`

Count forbidden source-shard discovery for workspace symbols. Target: zero.

### `diagnostic_workspace_source_scans`

Count forbidden full-source scans during diagnostic adaptation. Target: zero.

## Mapper requirements

Use one `SnapshotLocationMapper` per request. No per-site mapper recreation.

## Workspace symbols

Retain compiler query:

```rust
compiler.editor().workspace_symbols(...)
```

No source-shard fallback.

## Diagnostics

Retain current related-module/current-document mapping. Never enumerate all sources merely to prepare diagnostic conversion.

## New LSP acceptance suite

Create `phalcom-lsp/tests/plan_b_indexing.rs` with:

1. imported-alias cross-file definition;
2. lexical alias references;
3. semantic upstream references;
4. import-prefix navigation;
5. workspace symbol kinds;
6. many references in few source modules;
7. diagnostics with one related module in a large workspace;
8. zero symbol shard scans;
9. zero diagnostic workspace scans;
10. bounded line-index construction.

---

# 13. PB-C7 — Strengthen PB-1 Through PB-12

## PB-1 — Presentation-only movement

Keep current fingerprint/Arc/reference assertions. Add workspace-symbol contribution retention, formal range movement, and real zero-scan counters.

## PB-2 — Local references

Add shadowing/redeclaration/source-order coverage and role distinctions where applicable.

## PB-3 — Imported binding

Exercise real Plan-A import products rather than only manual linked fixtures.

## PB-4 — Alias dual relation

Assert local lexical rename set independently from upstream semantic reference set.

## PB-5 — Missing target appears

Exercise topology/import-product re-resolution and fail-closed unresolved state.

## PB-6 — Target disappears

Assert definition/reference/symbol/incident retirement and old-snapshot preservation.

## PB-7 — Delete/re-add contribution lifetime

Delete and re-add the same module identity. Assert no duplicate refs, symbols, incidents, or formal facts.

## PB-8 — Workspace symbols

Test class/enum/type-alias/variant plus both trigram (`>=3`) and short-query (`1–2`) paths. Short queries may scan symbol entries but not source shards.

## PB-9a — High-fanout provider body-only edit

Use **500+ consumers** plus a separate entry module.

Assert:

```text
source_modules_rebuilt == 1
all consumer shard Arcs retained
reference_targets_touched == 0
Foo TargetReferenceSet Arc retained
source_workspace_scan_units == 0
reference_workspace_scan_units == 0
formal_workspace_scan_units == 0
```

## PB-9b — One-consumer reference edit

In the same fixture, edit one consumer.

Assert only that source shard rebuilds and actual reference added/removed counters reflect the precise delta.

## PB-10 — Cold/incremental parity

Replace count-only checks with full deterministic projections.

Suggested helper:

```rust
#[derive(Debug, Eq, PartialEq)]
struct EditorSnapshotProjection {
    source_sites: ...,
    occurrences: ...,
    targets: ...,
    import_origins: ...,
    definitions: ...,
    lexical_references: ...,
    semantic_references: ...,
    formal_facts: ...,
    workspace_symbols: ...,
}
```

Compare values, not `Arc` identity.

Use mutation sequence including:

1. initial build;
2. presentation movement;
3. target appearance;
4. alias/reference edit;
5. rename;
6. deletion;
7. re-add.

Also compare `target_at`, `definition_locations`, reference queries, and workspace symbols.

## PB-11 — Old snapshot immutability

Pin snapshots across:

```text
range movement
import retarget
rename
deletion
```

Assert historical ranges, definitions, lexical refs, semantic refs, formal facts, and workspace symbols.

## PB-12 — Real zero-scan gate

Use 500+ modules and multiple callables/symbols.

Exercise:

- provider body-only edit;
- one-consumer reference edit;
- presentation-only movement;
- one-module deletion.

Assert semantic zero-scan counters and LSP zero/bounded counters.

PB-12 must fail if a broad discovery loop is reintroduced.

---

# 14. PB-C8 — Final Broad-Loop Audit and Completion Ledger

Before completion, inspect:

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/source_index/*
phalcom-semantic/src/presentation.rs
phalcom-lsp/src/backend.rs
phalcom-lsp/src/diagnostics.rs
phalcom-lsp/src/perf.rs
```

Classify remaining broad operations such as:

- resolved-import map cloning/reconstruction;
- all-source loops for `ModuleQueryProducts`;
- `source_modules` / `display_path_modules` construction;
- internal incident collection over all callable analyses;
- cold-only persistent-root construction.

Record a table:

| Operation | Complexity | Ordinary edit? | Ownership | Action |
|---|---:|---|---|---|
| exact source rebuild worklist | O(Δsource) | Yes | Plan B | retain |
| exact touched reference targets | O(Δtargets + sites) | Yes | Plan B | retain |
| cold source-index build | O(workspace) | No | Plan B | allowed |
| module-query product rebuild | O(workspace) | classify | Plan A/general semantic | classify |
| internal incident collection | O(callables) | classify | general semantic | classify |

The final claim must distinguish Plan B's source/reference/formal/LSP behavior from remaining costs owned elsewhere.

---

# 15. Suggested Commit Decomposition

```text
test(plan-b): make source/formal scan counters observable
perf(semantic): remove source-index workspace retirement and reuse scans
perf(semantic): consume exact importer-owned ImportSiteId worklists
perf(semantic): build formal projections from exact module callable ids
refactor(semantic): retire duplicate occurrence reverse target index
fix(semantic): make source attachment incidents module-owned
fix(semantic): count reference site deltas exactly
fix(semantic): preserve declaration kinds in workspace symbols
fix(semantic): cover generic class index type references
perf(lsp): add Plan B request work counters
test(lsp): add Plan B indexing and latency acceptance suite
test(plan-b): strengthen PB-1 through PB-12 certification
docs(plan-b): record final completion evidence and residual broad-loop classification
```

---

# 16. File-Level Checklist

## `phalcom-semantic/src/source_index/mod.rs`

- [ ] document scan-counter semantics;
- [ ] module-own attachment incidents;
- [ ] replace/retire incident operations;
- [ ] preserve equal reference/symbol contribution Arcs;
- [ ] remove duplicate reverse-index compatibility surface.

## `phalcom-semantic/src/source_index/reference.rs`

- [ ] keep exact touched-target replacement;
- [ ] true sorted set-delta counters;
- [ ] equal-cardinality replacement test;
- [ ] no full reference-index traversal.

## `phalcom-semantic/src/source_index/occurrence.rs`

- [ ] remove `target_occurrences`;
- [ ] retain exact `site -> target`;
- [ ] retain prefix projection;
- [ ] production missing import product fails closed.

## `phalcom-semantic/src/source_index/scope.rs`

- [ ] add `SourceDeclarationKind`;
- [ ] store kind in `DeclarationSourceInfo`.

## `phalcom-semantic/src/source_index/builder.rs`

- [ ] populate declaration kind;
- [ ] class index generic/`where` coverage;
- [ ] retain enum coverage;
- [ ] isolate compatibility import fallback.

## `phalcom-semantic/src/workspace.rs`

- [ ] add importer→site product;
- [ ] builder/helper;
- [ ] update constructors/tests.

## `phalcom-semantic/src/session.rs`

- [ ] exact `SourceIndexDelta`;
- [ ] exact retirement set;
- [ ] no reuse scan;
- [ ] exact importer-owned site lookup;
- [ ] exact module callable IDs for formal projection;
- [ ] no formal reuse scan;
- [ ] real work counters.

## `phalcom-semantic/src/presentation.rs`

- [ ] exact callable-ID projection;
- [ ] cold path not O(modules × callables).

## `phalcom-semantic/src/source_index/symbol.rs`

- [ ] declaration-kind mapping.

## `phalcom-lsp/src/perf.rs`

- [ ] five Plan-B counters;
- [ ] reset/snapshot/serde;
- [ ] unit tests.

## `phalcom-lsp/src/backend.rs`

- [ ] request-local conversion instrumentation;
- [ ] one mapper per request;
- [ ] zero symbol-shard scan regression counter.

## diagnostic path

- [ ] zero full-source diagnostic scan instrumentation;
- [ ] related-module/current-document mapping retained.

## semantic Plan-B tests

- [ ] real scan evidence;
- [ ] 500+ fanout;
- [ ] production import products;
- [ ] exact reference delta;
- [ ] full cold/incremental parity;
- [ ] expanded old-snapshot test;
- [ ] delete/re-add lifecycle.

## LSP Plan-B tests

- [ ] create `phalcom-lsp/tests/plan_b_indexing.rs`;
- [ ] navigation/reference/alias/prefix;
- [ ] symbol kinds;
- [ ] reference conversion work;
- [ ] diagnostic locality;
- [ ] zero source-shard scans.

---

# 17. Verification Strategy

## Focused semantic

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic plan_b_indexing
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic source_index
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic imported_resolution
```

## Full semantic

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic
```

## Focused LSP

```sh
RUSTFLAGS='' cargo test -p phalcom-lsp --test plan_b_indexing
RUSTFLAGS='' cargo test -p phalcom-lsp --test imported_binding_resolution
RUSTFLAGS='' cargo test -p phalcom-lsp --test module_navigation
RUSTFLAGS='' cargo test -p phalcom-lsp --test performance
```

## Full LSP

```sh
RUSTFLAGS='' cargo test -p phalcom-lsp
```

## Core regressions

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core
RUSTFLAGS='' cargo test -p phalcom-core --test cli-smoke
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus
```

## Workspace

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets
```

---

# 18. Performance Acceptance Table

| Scenario | Source rebuilds | Source scans | Ref targets touched | Ref scans | Formal rebuilds | Formal scans | LSP shard scans |
|---|---:|---:|---:|---:|---:|---:|---:|
| presentation-only one module | 1 | 0 | 0 | 0 | exact/0 | 0 | 0 |
| provider body-only, 500 consumers | 1 | 0 | 0 | 0 | provider only | 0 | 0 |
| one consumer adds Foo ref | 1 | 0 | exact Foo targets | 0 | consumer only | 0 | 0 |
| delete one module | 0 rebuild / 1 retire | 0 | retired contribution targets | 0 | 1 retire | 0 | 0 |
| workspace symbol query | 0 | N/A | N/A | N/A | 0 | N/A | 0 |
| diagnostics with one related module | 0 | N/A | N/A | N/A | 0 | N/A | diagnostic scans = 0 |

Wall-clock measurements may supplement these checks, but deterministic work counters are the primary contract.

---

# 19. Final Completion Gates

## Architecture

- [ ] persistent `SourceSemanticIndex`;
- [ ] `ReferenceIndex` is the workspace target reverse index;
- [ ] module contributions replace/retire cleanly;
- [ ] imported alias dual identity preserved;
- [ ] persistent module-local formal projection;
- [ ] incremental workspace symbols;
- [ ] LSP consumes compiler products.

## Correctness

- [ ] exact production import products;
- [ ] exact prefix navigation;
- [ ] correct class/enum/type-alias symbol kinds;
- [ ] complete current type-bearing source coverage;
- [ ] stale incidents retire;
- [ ] stale refs/symbols/formal facts retire;
- [ ] cold/incremental parity by value;
- [ ] old snapshots preserve historical values.

## Complexity

- [ ] `source_workspace_scan_units == 0`;
- [ ] `reference_workspace_scan_units == 0`;
- [ ] `formal_workspace_scan_units == 0`;
- [ ] `workspace_symbol_source_shard_scans == 0`;
- [ ] `diagnostic_workspace_source_scans == 0`;
- [ ] no all-import-product filter on ordinary source publication;
- [ ] no all-callable filter for one module's formal projection;
- [ ] no all-source-module retirement/reuse discovery scan.

## Acceptance

- [ ] PB-1 through PB-12 pass with strengthened production-path fixtures;
- [ ] PB-9 uses 500+ consumers;
- [ ] PB-10 compares full values;
- [ ] PB-11 covers range movement/import retarget/rename/deletion;
- [ ] PB-12 uses real semantic + LSP instrumentation;
- [ ] semantic suite passes;
- [ ] LSP suite passes;
- [ ] core regression suites pass;
- [ ] workspace/all-targets pass.

---

# 20. Completion Deliverable

Update the Plan B implementation-state document with:

1. final commit SHA;
2. checkpoint status;
3. exact architecture changes;
4. tests added/strengthened;
5. deterministic performance table;
6. pointer-retention evidence for unchanged products;
7. cold/incremental parity evidence;
8. old-snapshot evidence;
9. residual broad-loop classification;
10. verification command results.

The final claim should remain narrow and defensible:

> **Plan B's source-index, reverse-reference, formal-projection, workspace-symbol, and LSP adaptation paths are compiler-owned, contribution-based, immutable, and delta-sized on ordinary incremental publications.**

Do not claim the entire semantic pipeline is O(delta) unless the adjacent broad-loop audit separately proves that stronger statement.

---

# 21. Target Final Dataflow

```text
Plan A exact module/import/topology delta
        │
        ▼
exact Plan B module worklists
        │
        ├── source shard rebuild ── one final shard per changed module
        │
        ├── reference delta ─────── exact targets touched by changed contribution
        │
        ├── formal projection ───── exact callable IDs owned by changed module
        │
        ├── workspace symbols ───── exact module symbol contribution
        │
        └── incident contribution ─ exact module lifetime
        │
        ▼
immutable SemanticSnapshot
        │
        ▼
EditorSemanticQuery
        │
        ▼
thin LSP conversion + bounded location mapping
```

There should be no intermediate stage that asks:

```text
"Which workspace entries belong to the module I already know changed?"
```

by scanning the whole workspace.

That is the central completion criterion for Plan B.
