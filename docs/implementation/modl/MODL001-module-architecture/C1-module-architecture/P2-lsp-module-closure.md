---
id: MODL001.C1.P2
category: MODL
program: MODL001
checkpoint: MODL001.C1
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on: []
follows: null
supersedes: null
deferred_reason: null
---

# MODL001.C1.P2 — lsp module closure
## IDE Indexing, References, Rename Readiness, Retention, and Latency
## Execution scope

**Prepared:** 2026-09-06  
**Repository:** `aureat/phalcom-lang`  
**Plan B implementation baseline:** `a2b86fb4ce35657780623ad7c532d8b1d1178839`  
**Baseline commit:** `perf(plan-a): complete semantic incrementality patch slice`  
**Predecessor:** Plan A — Incrementality, Dependency Precision, and Semantic Propagation  
**Purpose:** Complete the compiler-owned editor/indexing architecture so source semantics, definitions, references, formal presentation, and workspace-symbol queries are incrementally maintained, structurally shared, exact, snapshot-safe, and asymptotically independent of total workspace size for ordinary edits.

---

# 1. Executive Objective

Plan A established the semantic/module side of the architecture:

```text
source mutation
    ↓
transactional module delta
    ↓
exact import / topology / linking products
    ↓
exact semantic dependency invalidation
    ↓
changed/revalidated semantic products
    ↓
retained semantic workspace products
    ↓
immutable SemanticSnapshot
```

Plan B must complete the editor-facing half:

```text
Plan-A semantic/module delta
    ↓
changed source/editor modules only
    ↓
module-owned source-index contributions
    ↓
exact contribution replace / retire
    ↓
delta-maintained definition/reference indexes
    ↓
delta-maintained formal presentation shards
    ↓
delta-maintained workspace-symbol index
    ↓
structurally shared immutable editor products
    ↓
SemanticSnapshot
    ↓
EditorSemanticQuery
    ↓
thin LSP protocol conversion
```

The central performance invariant is:

```text
ordinary single-file edit work
    ∝ changed source/editor contribution
      + Plan-A semantic products that actually changed
      + exact reverse-target sets touched
      + bounded persistent-map updates
```

It must **not** remain:

```text
precise Plan-A semantic recomputation
    +
clone all source shards
    +
rebuild all target occurrences
    +
walk every callable to rebuild formal projection
    +
scan all source shards for workspace-symbol queries
```

Plan B is therefore **not** a second semantic architecture. It is the incremental editor/index publication layer over the compiler-owned products Plan A already computes.

---

# 2. Repository-Grounded Baseline

The plan is grounded at pushed `main`:

```text
a2b86fb4ce35657780623ad7c532d8b1d1178839
perf(plan-a): complete semantic incrementality patch slice
```

This is one commit ahead of the earlier Plan-B requirements-analysis baseline `6410ebcb3a0d1df5b86dc34d00ec820c88e96658`.

The final Plan-A patch touched the semantic/module incrementality path, including:

```text
phalcom-modules/src/session.rs
phalcom-semantic/src/db/mod.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/tests/semantic/incremental/a7_performance.rs
```

It did **not** remove the Plan-B bottlenecks in:

```text
phalcom-semantic/src/source_index/
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/editor.rs
phalcom-lsp/src/backend.rs
```

## 2.1 Plan-A mechanisms Plan B must consume, not reproduce

Plan B must preserve and consume:

- canonical `ModuleId`, `SourceId`, `SourceLocation`;
- transactional `WorkspaceModuleSession`;
- exact module/source identity lifecycle;
- stable `ImportSiteId`;
- retained `ImportResolutionProduct`;
- retained `ResolvedImportPrefix`;
- exact topology and reverse import-site indexes;
- component-scoped linking;
- exact semantic `QueryKey` dependency graph;
- `ProductFingerprint` stability barriers;
- canonical `DeclarationId`, `CallableId`, `FieldId`, variant identities;
- retained module semantic structure shards;
- exact callable recomputation/reuse dispositions;
- immutable `SemanticSnapshot`;
- `SemanticPublicationEffects`;
- current source semantic/presentation fingerprint split;
- current imported-binding origin separation;
- snapshot-local `SourceSiteId`.

Plan B must not add another resolver, another semantic identity model, or another invalidation graph.

## 2.2 Current Plan-B-specific defects

At this baseline:

1. `SourceSemanticIndex` still owns a workspace-wide:
   ```rust
   BTreeMap<ModuleId, Arc<ModuleSourceIndex>>
   ```
   and `build_source_semantic_index` starts from:
   ```rust
   previous.modules.clone()
   ```
   which is O(total modules).

2. `SourceSemanticIndex::rebuild_target_occurrences()` still walks all modules and all occurrences.

3. `OccurrenceIndex` also stores a per-module reverse target map, duplicating reverse-index ownership.

4. `SourceIndexContext` still bridges import meaning through broad maps keyed by forms such as:
   ```text
   (ModuleId, String)
   ```
   instead of consuming exact Plan-A import-site products for the changed importer.

5. compound import path occurrences collapse multiple written segments onto the final module target instead of Plan-A prefix provenance.

6. `FormalSemanticProjection::from_callable_analyses_with_source_index(...)` walks/sorts every callable analysis and rebuilds every module interval index.

7. `EditorSemanticQuery::reference_sites` still starts from a global target occurrence list and classifies definition/reference status at query time.

8. imported aliases preserve:
   ```text
   lexical target = local Binding(SourceSiteId)
   semantic origin = remote canonical target
   ```
   but reverse indexes do not make both reference relations explicit.

9. `compiler_workspace_symbols` scans every source module/site on every workspace-symbol request.

10. LSP reference/location conversion performs avoidable repeated sorting/dedup and may reconstruct line indexes repeatedly.

11. `resolve_type_reference_targets` currently has an explicit `Statement::Enum(_)` skip in the inspected walker; enum type-bearing syntax needs a coverage closure.

12. bounded Universe presentation products are mixed into normal source-index construction instead of being retained as static editor infrastructure.

---

# 3. Ownership Boundary

| Area | Owner | Plan B rule |
|---|---|---|
| module identity | Plan A | consume |
| source/provider identity | Plan A | consume |
| import resolution algorithm | Plan A | do not re-resolve |
| `ImportSiteId` / import prefixes | Plan A | project exactly |
| component linking | Plan A | untouched |
| semantic dependency graph | Plan A | untouched |
| semantic recomputation | Plan A | consume exact effects |
| source-site construction | Plan B | own |
| source occurrence completeness | Plan B | own |
| definition index | Plan B | own |
| lexical reference index | Plan B | own |
| semantic/upstream reference index | Plan B | own |
| source contribution retirement | Plan B | own |
| formal projection retention | Plan B | own |
| workspace-symbol index | Plan B | own |
| immutable editor publication | cross-boundary | preserve snapshot immutability |
| editor query facade | Plan B | compiler-owned |
| URI/UTF-16 conversion | LSP | protocol only |
| rename endpoint | optional follow-up | index must be rename-ready |
| semantic-token lexical pass | unrelated | preserve current model |
| completion semantic resolution | unrelated | preserve compiler ownership |
| hover semantic resolution | unrelated | preserve compiler ownership |

---

# 4. Non-Negotiable Plan-B Invariants

## PB-INV-1 — No editor re-resolution

The source/index layer projects compiler products. It does not rediscover semantic meaning.

Forbidden:

```text
source text → filesystem/module search → export walk → guessed semantic target
```

Required:

```text
Plan-A canonical product → source occurrence/provenance
```

## PB-INV-2 — One source module owns one replaceable editor contribution

Every non-static source module must have a contribution that can be inserted, retained, replaced, or retired without scanning unrelated modules.

## PB-INV-3 — Published snapshots are immutable

Publishing B must not mutate source/reference/formal/symbol data observable through pinned snapshot A.

## PB-INV-4 — `SourceSiteId` remains snapshot-local

Do not create a global persistent local-binding identity to simplify retention.

Cross-revision correctness comes from contribution retirement/replacement.

## PB-INV-5 — Range movement is presentation-only when meaning is unchanged

If only trivia/ranges move:

```text
semantic fingerprint: unchanged
presentation fingerprint: changed
```

Then semantic target membership must not churn, while current ranges must update.

## PB-INV-6 — Definitions, lexical references, and semantic references are distinct relations

At minimum preserve:

```text
target → definitions
target → lexical references
target → semantic/upstream references
```

## PB-INV-7 — Imported aliases preserve dual identity

For:

```phalcom
from shapes import Circle as Local
Local.new()
```

retain:

```text
lexical target of Local use
    = local import Binding(SourceSiteId)

semantic origin
    = Declaration(shapes::Circle)
```

Do not collapse either fact into the other.

## PB-INV-8 — Exact target only

Only compiler-proven single targets enter exact reference sets. Completion/dynamic candidate sets are not references.

## PB-INV-9 — Ordinary publication performs no total source/reference/formal traversal

Cold construction may be O(workspace). Ordinary edits may not clone/scan/rebuild total editor state.

## PB-INV-10 — High-fanout work is proportional to exact affected fanout

Provider body-only change must not touch consumer reference shards. A genuine exported identity change may legitimately affect the exact fanout.

## PB-INV-11 — Delete/re-add correctness is contribution-lifetime based

Same-name re-add may reproduce an equal semantic ID. Correctness depends on full retirement of old source-owned entries.

## PB-INV-12 — LSP remains an adapter

No LSP-side import/reference resolver or workspace semantic scan may become a second authority.

---

# 5. Target Architecture

```text
WorkspaceModuleUpdate / Plan-A semantic effects
                 │
                 ▼
      SourceIndexPublicationDelta
                 │
       ┌─────────┴─────────┐
       ▼                   ▼
ModuleSourceIndex     ModuleFormalProjection
       │                   │
       ├── source sites    ├── formal facts
       ├── occurrences     └── interval index
       ├── exact targets
       ├── reference contribution
       └── symbol contribution
       │
       ▼
exact replace / retire
       │
       ├── persistent definition index
       ├── persistent lexical-reference index
       ├── persistent semantic-reference index
       └── persistent workspace-symbol postings
       │
       ▼
structurally shared immutable roots
       │
       ▼
SemanticSnapshot
       │
       ▼
EditorSemanticQuery
       │
       ▼
LSP conversion
```

---

# 6. Persistent Collection Decision

Plan B needs genuine structural sharing at root-map level. `Arc` leaves inside a cloned `BTreeMap` are not sufficient.

## Decision

Add the `im` crate as a workspace dependency and use `im::OrdMap` / `im::OrdSet` behind compiler-owned index types.

Suggested dependency:

```toml
im = "15.1"
```

Files:

```text
Cargo.toml
phalcom-semantic/Cargo.toml
```

Requirements:

- persistent root clone does not copy all entries;
- insert/remove copies only touched tree paths;
- deterministic ordered iteration;
- old snapshots retain old roots;
- no `im::*` type leaks into broad public API.

Do **not** substitute:

```rust
Arc<BTreeMap<K, V>>
```

plus full `BTreeMap::clone()` per edit.

If dependency policy rejects `im`, implement an equivalent persistent ordered-map abstraction before continuing; do not regress to full-map copy-on-write.

---

# 7. Revised Core Data Structures

## 7.1 `ModuleReferenceContribution`

Add:

```text
phalcom-semantic/src/source_index/reference.rs
```

Conceptual shape:

```rust
#[derive(Clone, Debug, Default)]
pub struct ModuleReferenceContribution {
    pub definitions:
        BTreeMap<SemanticTargetId, Arc<[SourceSiteId]>>,
    pub lexical_references:
        BTreeMap<SemanticTargetId, Arc<[SourceSiteId]>>,
    pub semantic_references:
        BTreeMap<SemanticTargetId, Arc<[SourceSiteId]>>,
}
```

It is owned by exactly one source module and built after final AST/formal target attachment.

Rules:

- declaration site → definition;
- local use → lexical + semantic reference to local binding;
- exact declaration/callable/field/variant use → lexical + semantic same target;
- import binding declaration → definition of local binding;
- imported alias use → lexical local binding + semantic remote origin;
- written remote import item → semantic/direct remote reference;
- exact module path segment → module reference;
- unresolved hint/dynamic candidate → no exact reverse entry.

## 7.2 `TargetReferenceSet`

```rust
#[derive(Clone, Debug, Default)]
pub struct TargetReferenceSet {
    pub definitions: Arc<[SourceSiteId]>,
    pub lexical_references: Arc<[SourceSiteId]>,
    pub semantic_references: Arc<[SourceSiteId]>,
}
```

Each slice must be sorted, duplicate-free, immutable.

## 7.3 `ReferenceIndex`

```rust
#[derive(Clone, Debug, Default)]
pub struct ReferenceIndex {
    by_target:
        im::OrdMap<
            SemanticTargetId,
            Arc<TargetReferenceSet>,
        >,
}
```

Public read API:

```rust
pub fn definitions(
    &self,
    target: &SemanticTargetId,
) -> &[SourceSiteId];

pub fn lexical_references(
    &self,
    target: &SemanticTargetId,
) -> &[SourceSiteId];

pub fn semantic_references(
    &self,
    target: &SemanticTargetId,
) -> &[SourceSiteId];
```

Internal update API:

```rust
pub(crate) fn replace_module_contribution(
    &self,
    old: Option<&ModuleReferenceContribution>,
    new: Option<&ModuleReferenceContribution>,
    stats: &mut SourceIndexUpdateStats,
) -> Self;
```

Only targets in `keys(old) ∪ keys(new)` may be touched.

## 7.4 `ModuleSourceIndex`

Revise:

```rust
pub struct ModuleSourceIndex {
    pub structure: Arc<SourceScopeIndex>,
    pub occurrences: Arc<OccurrenceIndex>,
    pub expression_sites: Arc<[SourceSite]>,
    ...
    pub attachments:
        BTreeMap<CallableId, Arc<CallableSourceAttachment>>,

    reference_contribution:
        Arc<ModuleReferenceContribution>,

    workspace_symbols:
        Arc<[WorkspaceSymbolEntry]>,
}
```

The contribution fingerprint must exclude source ranges.

## 7.5 `SourceSemanticIndex`

Target shape:

```rust
pub struct SourceSemanticIndex {
    modules:
        im::OrdMap<
            ModuleId,
            Arc<ModuleSourceIndex>,
        >,

    references:
        Arc<ReferenceIndex>,

    workspace_symbols:
        Arc<WorkspaceSymbolIndex>,

    incidents_by_module:
        im::OrdMap<
            ModuleId,
            Arc<[SourceAttachmentError]>,
        >,
}
```

Make representation private and provide read-only iteration/accessors.

Remove production dependence on:

```rust
SourceSemanticIndex::rebuild_target_occurrences()
```

## 7.6 Workspace symbol types

Add:

```text
phalcom-semantic/src/source_index/symbol.rs
```

Suggested protocol-neutral types:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorkspaceSymbolId {
    pub target: SemanticTargetId,
    pub site: SourceSiteId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EditorSymbolKind {
    Module,
    Class,
    Enum,
    TypeAlias,
    Callable,
    Field,
    Variant,
    VariantField,
    Binding,
}

#[derive(Clone, Debug)]
pub struct WorkspaceSymbolEntry {
    pub id: WorkspaceSymbolId,
    pub name: Box<str>,
    pub normalized_name: Box<str>,
    pub target: SemanticTargetId,
    pub declaration_site: SourceSiteId,
    pub kind: EditorSymbolKind,
}
```

No URI or LSP `SymbolKind` belongs here.

## 7.7 `WorkspaceSymbolIndex`

Required capabilities:

```rust
pub fn replace_module(
    old: &[WorkspaceSymbolEntry],
    new: &[WorkspaceSymbolEntry],
) -> Self;

pub fn search(
    query: &str,
    limit: usize,
) -> Vec<&WorkspaceSymbolEntry>;
```

Preserve current case-insensitive substring semantics.

Recommended implementation:

- persistent entry map by `WorkspaceSymbolId`;
- normalized-name postings;
- trigram postings for queries length >= 3;
- verify final `contains()` semantics;
- short queries may inspect indexed symbol-name entries but never source shards.

## 7.8 `ModuleFormalProjection`

Refactor `phalcom-semantic/src/presentation.rs`:

```rust
#[derive(Clone, Debug, Default)]
pub struct ModuleFormalProjection {
    by_fact: BTreeMap<FormalFactRef, FormalFactSite>,
    sites: Arc<[FormalFactSite]>,
    intervals: RangeIndex<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct FormalSemanticProjection {
    modules:
        im::OrdMap<
            ModuleId,
            Arc<ModuleFormalProjection>,
        >,
}
```

Add:

```rust
impl FormalFactRef {
    pub fn module(&self) -> &ModuleId;
}
```

## 7.9 `SourceIndexUpdateStats`

Add deterministic publication metrics:

```rust
pub struct SourceIndexUpdateStats {
    pub source_modules_rebuilt: usize,
    pub source_modules_reused: usize,
    pub source_modules_retired: usize,
    pub presentation_only_modules: usize,

    pub reference_contributions_replaced: usize,
    pub reference_targets_touched: usize,
    pub reference_sites_added: usize,
    pub reference_sites_removed: usize,

    pub formal_modules_rebuilt: usize,
    pub formal_modules_reused: usize,
    pub formal_modules_retired: usize,

    pub workspace_symbol_contributions_replaced: usize,
    pub workspace_symbol_entries_added: usize,
    pub workspace_symbol_entries_removed: usize,

    pub source_workspace_scan_units: usize,
    pub reference_workspace_scan_units: usize,
    pub formal_workspace_scan_units: usize,
}
```

The three scan counters must be zero on the ordinary delta path.

---

# 8. Import-Site Projection Contract

## 8.1 Publish importer→site index across the Plan-A boundary

`WorkspaceModuleSession` already retains `sites_by_importer`.

Plan B may expose this retained product without changing resolution semantics.

Preferred addition to `WorkspaceModuleUpdate`:

```rust
pub import_sites_by_module:
    Arc<BTreeMap<ModuleId, BTreeSet<ImportSiteId>>>,
```

Thread it through:

```text
phalcom-semantic/src/workspace.rs
SemanticWorkspaceInput
```

Do not reconstruct the importer index in semantic code by scanning all `import_products`.

## 8.2 Revised module-local `SourceIndexContext`

The final source-index context should be built per changed module.

Conceptually:

```rust
pub struct SourceIndexContext {
    pub module: ModuleId,

    pub import_products:
        BTreeMap<
            ImportSiteLocalId,
            Arc<ImportResolutionProduct>,
        >,

    pub exported_targets:
        BTreeMap<
            (ModuleId, Box<str>),
            SemanticTargetId,
        >,

    pub callable_targets:
        BTreeMap<
            (DeclarationId, Selector),
            CallableId,
        >,

    pub type_reference_targets:
        BTreeMap<
            SourceRange,
            DeclarationId,
        >,
}
```

Exact representation may differ, but context construction for module M must be proportional to M's authored imports and exact target dependencies, not the entire workspace.

## 8.3 Exact prefix occurrences

For:

```phalcom
import a.b.c
```

with Plan-A prefix products:

```text
a     → Module(a)
a.b   → Module(a.b)
a.b.c → Module(a.b.c)
```

source occurrences must attach:

```text
"a" → Module(a)
"b" → Module(a.b)
"c" → Module(a.b.c)
```

Do not attach all segments to the final module.

---

# 9. Reference Semantics Contract

## Definitions

Definition sites introduce the target:

```text
class/enum/type alias name
callable
field
variant/variant field
local binding
top-level binding
local import binding
```

An import alias declaration defines the local import binding, not the remote class/module.

## Lexical references

Lexical references answer:

> Which source sites resolve directly to this target in local name resolution?

For imported alias `Local`, downstream `Local` uses target the local binding.

## Semantic references

Semantic references answer:

> Which source sites semantically refer to this canonical upstream entity, possibly through an import binding?

The same imported alias use can therefore also contribute to the upstream declaration's semantic reference set.

## Rename readiness

Plan B must retain enough data to distinguish:

```text
rename local alias
rename upstream declaration
```

Do not ship a naive rename endpoint that uses only semantic references.

Mandatory retained information:

```text
source site
lexical target
semantic/upstream target
occurrence role
import origin
```

The protocol endpoint itself is optional.

---

# 10. Implementation Sequence

Execute in this order:

```text
B0  Baseline lock and no-regression harness
B1  Persistent source-index roots
B2  Module-owned reference contributions
B3  Delta source/reference replacement and retirement
B4  Exact import-site/prefix projection
B5  Occurrence completeness
B6  Incremental formal projection
B7  Incremental workspace-symbol index
B8  Editor query cutover
B9  LSP adapter/latency cutover
B10 Lifecycle, old-snapshot, parity, and performance closure
```

Do not add LSP-side caches around the old global rebuild path. Compiler publication must become incremental first.


---

# 11. Checkpoint B0 — Lock the Plan-A Baseline

**Risk:** LOW  
**Type:** verification / instrumentation

## Goal

Establish the exact implementation baseline and ensure Plan B does not reopen Plan-A behavior.

Expected baseline:

```text
a2b86fb4ce35657780623ad7c532d8b1d1178839
```

## Files

Read/verify:

```text
phalcom-modules/src/session.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/source_index/mod.rs
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/editor.rs
phalcom-lsp/src/backend.rs

phalcom-semantic/tests/semantic/integration/source_index.rs
phalcom-semantic/tests/semantic/integration/imported_resolution.rs
phalcom-semantic/tests/semantic/incremental/a7_performance.rs
phalcom-lsp/tests/imported_binding_resolution.rs
phalcom-lsp/tests/module_navigation.rs
```

## Tasks

- [ ] Record:
  ```bash
  git status --short
  git rev-parse HEAD
  git log -5 --oneline
  ```
- [ ] If local HEAD differs from the recorded baseline, inspect and reconcile; do not reset user work.
- [ ] Run current source-index/editor/imported-resolution tests.
- [ ] Run Plan-A performance/parity gates.
- [ ] Run imported-binding and module-navigation LSP tests.
- [ ] Add:
  ```text
  phalcom-semantic/tests/semantic/incremental/plan_b_indexing.rs
  ```
- [ ] Register it in:
  ```text
  phalcom-semantic/tests/semantic/incremental/mod.rs
  ```

## Verification

```bash
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic source_index
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic imported_resolution
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic a7_performance

RUSTFLAGS='' cargo test -p phalcom-lsp --test imported_binding_resolution
RUSTFLAGS='' cargo test -p phalcom-lsp --test module_navigation
```

## Completion criterion

Baseline behavior and Plan-A acceptance remain unchanged; Plan-B tests can be added without modifying production semantics.

Suggested commit:

```text
test(plan-b): establish editor indexing baseline
```

---

# 12. Checkpoint B1 — Persistent Source-Index Roots

**Risk:** MEDIUM  
**Primary objective:** eliminate O(workspace) root-map cloning.

## Files

```text
Cargo.toml
phalcom-semantic/Cargo.toml
phalcom-semantic/src/source_index/mod.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/tests/semantic/integration/source_index.rs
phalcom-semantic/tests/semantic/incremental/plan_b_indexing.rs
```

## Tasks

- [ ] Add workspace `im` dependency.
- [ ] Convert `SourceSemanticIndex.modules` from public `BTreeMap` to private persistent ordered map.
- [ ] Add read APIs:
  ```rust
  pub fn module(
      &self,
      module: &ModuleId,
  ) -> Option<&ModuleSourceIndex>;

  pub fn modules(
      &self,
  ) -> impl Iterator<Item = (&ModuleId, &Arc<ModuleSourceIndex>)>;
  ```
- [ ] Migrate production callers away from direct `.modules`.
- [ ] Replace:
  ```rust
  previous.modules.clone()
  ```
  with persistent-root reuse plus exact changed inserts/removals.
- [ ] Preserve cold source-index constructors but make them produce the new representation.
- [ ] Add `SourceIndexUpdateStats`.
- [ ] Instrument full source-map traversal as `source_workspace_scan_units`.
- [ ] Keep all source-index publication immutable.

## Mandatory tests

### PB-8a — unchanged source shard pointer retention

Create 3+ disconnected modules, edit one, assert:

```rust
Arc::ptr_eq(old_b, new_b)
Arc::ptr_eq(old_c, new_c)
```

### PB-12a — no root workspace scan

Large fixture; one ordinary source body edit:

```text
source_workspace_scan_units == 0
```

### Cold representation parity

Cold and incrementally published source indexes must be semantically equal.

## Removal gate

Production incremental code must no longer clone the complete previous source-module map.

Suggested commit:

```text
perf(source-index): add persistent module roots
```

---

# 13. Checkpoint B2 — Module-Owned Reference Contributions

**Risk:** HIGH  
**Primary objective:** make reference replacement/retirement exact.

## Files

```text
phalcom-semantic/src/source_index/mod.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/source_index/scope.rs
phalcom-semantic/src/source_index/reference.rs
phalcom-semantic/src/editor.rs
phalcom-semantic/tests/semantic/integration/source_index.rs
phalcom-semantic/tests/semantic/integration/imported_resolution.rs
```

## Tasks

- [ ] Add `ModuleReferenceContribution`.
- [ ] Add `TargetReferenceSet`.
- [ ] Add `ReferenceIndex`.
- [ ] Build a module contribution only after AST occurrence and formal exact-target attachment are complete.
- [ ] Classify definition vs reference once at publication time.
- [ ] Resolve imported-binding semantic origin without rewriting its lexical target.
- [ ] Store contribution on `ModuleSourceIndex`.
- [ ] Add contribution semantic fingerprinting that excludes ranges.
- [ ] Do not add dynamic/ambiguous candidate sets to exact indexes.

## Required semantic cases

### Local bindings

Test:

```phalcom
method(x) {
  let y = x
  let f = |x| x + y
  let (a, b) = pair
  x + y + a + b
}
```

Prove:

- parameter references;
- closure shadowing;
- closure capture;
- destructuring;
- independent same-name bindings;
- write roles where syntax supports writes.

### Declaration/callable/field/variant targets

Test:

- class use;
- class-side/static use;
- method call;
- getter;
- setter/write;
- field read;
- field write;
- variant constructor;
- variant field.

### Imported alias

For:

```phalcom
from provider import Item as Local
Local.make()
```

assert:

```text
definitions[Binding(local)]
    contains alias declaration

lexical_references[Binding(local)]
    contains Local use

semantic_references[Declaration(provider::Item)]
    contains Local use

definitions[Declaration(provider::Item)]
    does not contain local alias declaration
```

## Completion criterion

Each source module can describe exactly what it contributes to reverse target relations without inspecting unrelated source modules.

Suggested commit:

```text
feat(source-index): add module-owned reference contributions
```

---

# 14. Checkpoint B3 — Delta Reference Replacement and Retirement

**Risk:** HIGH  
**Primary objective:** eliminate global `rebuild_target_occurrences()`.

## Files

```text
phalcom-semantic/src/source_index/reference.rs
phalcom-semantic/src/source_index/mod.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/editor.rs
phalcom-semantic/tests/semantic/incremental/plan_b_indexing.rs
```

## Replacement algorithm

For changed module M:

```text
old contribution targets
    ∪
new contribution targets
        ↓
exact touched target set
        ↓
for each touched target:
    remove old M-owned sites
    add new M-owned sites
    keep sorted unique order
    replace persistent target entry
```

For deletion:

```text
old contribution
new = None
```

For addition:

```text
old = None
new contribution
```

## Presentation-only fast path

If:

```text
old.fingerprints().semantic
==
new.fingerprints().semantic
```

then:

```text
old reference contribution Arc is retained
reference_targets_touched == 0
```

even though current source ranges may differ.

## Tasks

- [ ] Implement deterministic sorted site merge/difference helpers.
- [ ] Implement exact module contribution replacement.
- [ ] Wire changed/removed source modules from `SemanticWorkspaceSession`.
- [ ] Stop calling `SourceSemanticIndex::rebuild_target_occurrences()` in incremental publication.
- [ ] Remove global `target_occurrences` storage after editor cutover; a temporary compatibility accessor may delegate to `ReferenceIndex`.
- [ ] Remove `OccurrenceIndex` reverse target storage once no production caller needs it.
- [ ] Add stats:
  - touched targets;
  - sites added;
  - sites removed;
  - reference workspace scans.

## Tests

### PB-1 — presentation-only movement

Insert comments/whitespace before declarations/uses.

Assert:

- new ranges correct;
- semantic source fingerprint stable;
- presentation fingerprint changed;
- reverse target membership unchanged;
- `reference_targets_touched == 0`;
- unrelated module shards retained.

### PB-6 — target disappearance

Remove declaration/source.

Assert:

- old definition removed;
- old references removed/reclassified;
- no stale site survives.

### PB-7 — delete and same-name re-add

Delete and re-add same canonical declaration name.

Assert old contribution was fully retired even if the new `DeclarationId` compares equal.

### PB-8b — reverse-index retention

Edit unrelated module and assert unaffected `TargetReferenceSet` `Arc`s are pointer-identical.

### PB-9a — high fanout provider body-only edit

500+ consumers reference one provider declaration. Change only provider method body.

Assert:

```text
reference_targets_touched == 0
```

for unaffected exported identity relationships.

## Completion criterion

No ordinary update performs a global reverse occurrence rebuild.

Suggested commit:

```text
perf(source-index): delta-maintain reverse target indexes
```

---

# 15. Checkpoint B4 — Exact Import-Site and Prefix Projection

**Risk:** HIGH  
**Primary objective:** consume Plan-A import products without path-string semantic reconstruction.

## Files

```text
phalcom-modules/src/session.rs
phalcom-semantic/src/workspace.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/tests/semantic/integration/source_index.rs
phalcom-semantic/tests/semantic/integration/imported_resolution.rs
phalcom-semantic/tests/semantic/incremental/plan_b_indexing.rs
phalcom-lsp/tests/module_navigation.rs
phalcom-lsp/tests/imported_binding_resolution.rs
```

## Tasks

- [ ] Publish retained importer→import-site index from module session.
- [ ] Thread it through `SemanticWorkspaceInput`.
- [ ] Build module-local `SourceIndexContext` from exact importer sites.
- [ ] Match authored AST imports to canonical `ImportSiteId` by existing interface import-site ordering.
- [ ] Use `ImportResolutionProduct.target`.
- [ ] Use `ImportResolutionProduct.prefixes`.
- [ ] Remove source-index dependence on global `(ModuleId, String)` resolution scans.
- [ ] Attach path segments to exact prefix modules.
- [ ] Keep unresolved imports local without fabricating remote targets.
- [ ] Preserve existing imported definition behavior.

## Tests

### PB-3 — imported binding reference indexing

Existing definition fixtures remain green.

Add:

- imported declaration has local binding identity;
- downstream use has lexical local relation;
- downstream use has semantic remote relation;
- unrelated import edit does not touch the target set.

### PB-4 — alias dual relation

Explicitly test `Remote as Local`.

### PB-5 — missing target appears

Start unresolved:

```phalcom
from missing import Thing
Thing
```

Then add canonical target.

Assert:

- Plan A re-resolves exact import site;
- only affected importer/editor contribution changes;
- semantic remote relation appears;
- unrelated source/reference shards retained.

### Prefix precision

For compound path, every segment navigates/references its exact resolved prefix module.

### Re-export

Verify remote exported name and module-prefix source occurrences retain canonical provenance.

## Completion criterion

Source indexing no longer reconstructs import meaning from path strings when exact Plan-A products exist.

Suggested commit:

```text
feat(source-index): project exact import-site prefix provenance
```

---

# 16. Checkpoint B5 — Occurrence Coverage Closure

**Risk:** MEDIUM/HIGH  
**Primary objective:** make current language source semantics complete enough for definition/reference tooling.

## Files

```text
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/source_index/scope.rs
phalcom-semantic/tests/semantic/integration/source_index.rs
phalcom-semantic/tests/semantic/integration/imported_resolution.rs
```

## 16.1 Type-reference coverage

Refactor `TypeReferenceTargetCollector` into explicit declaration-form helpers.

Add helpers conceptually:

```rust
fn class_definition(...)
fn enum_definition(...)
fn enum_member(...)
fn enum_behavior_member(...)
fn callable_annotations(...)
fn variant_annotations(...)
```

### Mandatory enum coverage

`Statement::Enum(_)` must not be silently skipped.

Cover all type-bearing enum syntax present in the current AST:

- enum generic bounds;
- enum where clauses;
- variant field annotations;
- variant-local generic annotations;
- enum behavior parameter annotations;
- enum behavior return annotations;
- getter/setter/index annotations represented by enum behavior members.

Generic binders remain binders, not nominal type references.

### Qualified type references

For:

```text
pkg.models.User
```

the final type token targets `User`.

Module prefixes should use module/import occurrence provenance, not fabricated declaration targets.

## 16.2 Member/value references

Audit AST occurrence traversal for:

- variable reads/writes;
- assignment targets;
- field reads/writes;
- getter/setter calls;
- index get/set;
- operator dispatch;
- constructor/variant use;
- class-side methods;
- closure bodies;
- nested blocks;
- `for` bindings;
- destructuring;
- match/pattern bindings.

Formal analysis remains authoritative where syntax alone cannot prove an exact member target.

## 16.3 Attachment merge

When `CallableSourceAttachment.exact_targets` supplies a stronger exact target:

- update that module's effective target relation;
- rebuild only that module's contribution;
- avoid duplicate sites for the same source token.

## Test matrix

| Syntax | Expected target | Role |
|---|---|---|
| local read | Binding | Read |
| local write | Binding | Write |
| parameter | Binding | Read |
| field read | Field | Read |
| field write | Field | Write |
| method call | Callable | Call |
| class name | Declaration | Reference |
| type annotation | Declaration | Reference |
| variant constructor | Variant/VariantFamily | Call/Reference |
| import segment | Module | Reference |
| import item | canonical remote target | Reference |
| re-export item | canonical remote target | Reference |

## Completion criterion

No known semantic source surface remains absent because the source walker skipped an AST form.

Suggested commit:

```text
fix(source-index): close occurrence coverage gaps
```

---

# 17. Checkpoint B6 — Incremental Formal Projection

**Risk:** HIGH  
**Primary objective:** remove the all-callable presentation rebuild.

## Files

```text
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/editor.rs
phalcom-semantic/tests/semantic/integration/presentation.rs
phalcom-semantic/tests/semantic/incremental/plan_b_indexing.rs
```

## Tasks

- [ ] Introduce `ModuleFormalProjection`.
- [ ] Convert `FormalSemanticProjection` to persistent module roots.
- [ ] Add `FormalFactRef::module()`.
- [ ] Enumerate current callable IDs from the changed module's source/semantic shard.
- [ ] Exact-lookup those callables in retained `callable_analyses`.
- [ ] Rebase current ranges from current source attachments.
- [ ] Do not filter all callable analyses to discover one module's callables.
- [ ] Add exact replace/retire API:
  ```rust
  FormalSemanticProjection::replace_modules(...)
  ```
- [ ] Rebuild a formal module shard if:
  - callable semantic products in that module changed;
  - or source presentation in that module changed.
- [ ] Retire removed module formal shards directly.
- [ ] Make snapshot setters attach already-built products rather than rebuilding formal projection as hidden side effects.
- [ ] Retain Universe/static presentation shard separately from ordinary project updates.

## Critical snapshot change

Current conceptual behavior:

```text
with_callable_analyses
    → rebuild formal projection

with_source_index
    → rebuild formal projection again
```

End state:

```text
session builds exact retained formal projection
    ↓
snapshot attaches it
```

No hidden workspace-wide derived recomputation.

## Tests

### PB-1 formal movement

Range-only source change:

- callable analysis `Arc` retained;
- changed module formal shard rebuilt;
- formal range updated;
- unrelated formal shards pointer-retained.

### PB-8c

Assert unrelated `ModuleFormalProjection` `Arc::ptr_eq`.

### PB-12b

Ordinary edit:

```text
formal_workspace_scan_units == 0
```

### PB-11 formal old snapshot

Pin A, publish B after range movement. A retains old formal ranges; B has new ranges.

## Completion criterion

No ordinary publication walks every callable analysis to reconstruct formal presentation.

Suggested commit:

```text
perf(presentation): retain module formal projection shards
```


---

# 18. Checkpoint B7 — Incremental Workspace-Symbol Index

**Risk:** MEDIUM  
**Primary objective:** remove request-time traversal of all source shards.

## Files

```text
phalcom-semantic/src/source_index/symbol.rs
phalcom-semantic/src/source_index/mod.rs
phalcom-semantic/src/editor.rs
phalcom-semantic/src/session.rs
phalcom-lsp/src/backend.rs
phalcom-lsp/src/perf.rs
phalcom-semantic/tests/semantic/incremental/plan_b_indexing.rs
phalcom-lsp/tests/plan_b_indexing.rs
phalcom-lsp/Cargo.toml
```

## Tasks

- [ ] Add protocol-neutral workspace-symbol types.
- [ ] Build one symbol contribution per `ModuleSourceIndex`.
- [ ] Store `SourceSiteId` and semantic target, not URI/LSP location.
- [ ] Build persistent `WorkspaceSymbolIndex`.
- [ ] Replace one changed module's contribution exactly.
- [ ] Remove one deleted module's contribution exactly.
- [ ] Preserve current case-insensitive substring behavior.
- [ ] Use indexed candidate narrowing for nontrivial queries.
- [ ] Add:
  ```rust
  EditorSemanticQuery::workspace_symbols(
      query,
      limit,
  )
  ```
- [ ] Cut LSP workspace-symbol handler over to compiler-owned index.
- [ ] Delete production:
  ```text
  for module in compiler.source_index().modules...
  ```
- [ ] Add LSP counter:
  ```text
  workspace_symbol_source_shard_scans
  ```

## Tests

- source addition → symbol appears;
- declaration rename → old result disappears, new result appears;
- deletion → symbol disappears;
- delete/re-add → exactly one current symbol;
- alias/public surface entries preserve current intended naming behavior;
- unrelated source edit retains unaffected symbol contributions;
- case-insensitive substring query parity;
- large workspace query performs zero source-shard scans.

## Completion criterion

Workspace symbols are an incrementally maintained compiler product.

Suggested commit:

```text
perf(editor): add incremental workspace symbol index
```

---

# 19. Checkpoint B8 — Editor Query Cutover

**Risk:** MEDIUM  
**Primary objective:** make definitions/references direct indexed lookups.

## Files

```text
phalcom-semantic/src/editor.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/source_index/reference.rs
phalcom-semantic/tests/semantic/integration/editor.rs
phalcom-semantic/tests/semantic/integration/imported_resolution.rs
phalcom-semantic/tests/semantic/incremental/plan_b_indexing.rs
```

## Reference domain API

Add:

```rust
pub enum ReferenceDomain {
    Lexical,
    Semantic,
}
```

Revise/extend:

```rust
pub fn definition_sites(
    &self,
    target: &SemanticTargetId,
) -> &[SourceSiteId];

pub fn reference_sites(
    &self,
    target: &SemanticTargetId,
    domain: ReferenceDomain,
) -> &[SourceSiteId];
```

A borrowed iterator or lightweight view is acceptable if it avoids allocation.

## Definition behavior

For ordinary canonical targets:

```text
target
→ ReferenceIndex.definitions
```

For local import binding go-to-definition:

```text
Binding(import)
→ ImportBindingOrigin
→ remote canonical target
→ remote definition
```

Preserve current imported-definition behavior.

## Default reference behavior

Recommended compatibility behavior:

```text
Binding target
    → Lexical domain

Declaration/Callable/Field/Variant/Module target
    → Semantic domain
```

This ensures local alias references stay local while global canonical references include imported semantic uses.

If the existing one-argument `reference_sites` method remains temporarily, make it a compatibility wrapper around this rule and cover it explicitly.

## Rename-readiness API

Expose a protocol-neutral view:

```rust
pub struct RenameTargetView<'a> {
    pub lexical_target: &'a SemanticTargetId,
    pub semantic_target: Option<&'a SemanticTargetId>,
    pub definitions: &'a [SourceSiteId],
    pub lexical_references: &'a [SourceSiteId],
    pub semantic_references: &'a [SourceSiteId],
}
```

Exact type/name may vary.

This is infrastructure only; no mandatory LSP rename endpoint in Plan B.

## Remove query-time classification

Delete/retire logic equivalent to:

```text
all sites for target
→ resolve each site
→ inspect SourceSiteKind
→ decide definition/reference
```

Classification belongs in source contribution publication.

## Tests

- local binding definition/reference;
- imported alias lexical references;
- imported alias semantic references;
- upstream definition excludes local alias declaration;
- canonical remote semantic references include downstream imported use;
- deterministic order;
- no duplicates;
- zero workspace scans.

## Completion criterion

Definition/reference query complexity is target lookup plus result count.

Suggested commit:

```text
refactor(editor): cut definitions and references to exact indexes
```

---

# 20. Checkpoint B9 — LSP Adapter and Latency Cutover

**Risk:** MEDIUM  
**Primary objective:** prevent protocol conversion from reintroducing avoidable result-scale costs.

## Files

```text
phalcom-lsp/src/backend.rs
phalcom-lsp/src/line_index.rs
phalcom-lsp/src/perf.rs
phalcom-lsp/tests/imported_binding_resolution.rs
phalcom-lsp/tests/module_navigation.rs
phalcom-lsp/tests/plan_b_indexing.rs
```

## 20.1 Definition/reference location mapper

Introduce a request-local helper conceptually:

```rust
struct SnapshotLocationMapper<'a> {
    snapshot: &'a SemanticSnapshot,
    line_indexes: BTreeMap<ModuleId, LineIndex>,
    uris: BTreeMap<ModuleId, Url>,
}
```

Use the exact pinned snapshot.

Flow:

```text
SourceSiteId results
    ↓
group/lookup by ModuleId
    ↓
URI once per module
    ↓
LineIndex once per module
    ↓
range conversion
```

Requirements:

- do not build `LineIndex` per reference;
- do not resolve filesystem paths per reference;
- do not consult newer snapshot/current mutable source for an old pinned request;
- preserve Universe virtual-source mapping.

## 20.2 Result dedup

Compiler reference sets must already be sorted/deduplicated.

If `includeDeclaration` merges definitions with references, merge ordered sets rather than:

```rust
locations.iter().any(...)
```

for every new result.

No O(R²) duplicate filtering.

## 20.3 Workspace symbols

LSP calls:

```text
compiler.editor().workspace_symbols(...)
```

Then only maps:

```text
EditorSymbolKind → LSP SymbolKind
SourceSiteId → Location
```

No semantic search in `backend.rs`.

## 20.4 Diagnostic/publication audit

While touching request/publication latency, audit the current diagnostics path.

`combined_diagnostics_for` currently risks building source-location metadata across the workspace for a single document.

Refactor any such broad source traversal to exact diagnostic modules/current document only.

Use existing `SemanticPublicationEffects.diagnostics_changed` rather than republishing semantic diagnostics for every open document whenever any generation changes.

This is Plan-B-owned editor publication work, not semantic analysis.

Add deterministic LSP counters:

```text
reference_source_modules_converted
reference_line_indexes_built
reference_duplicate_filter_steps
workspace_symbol_source_shard_scans
diagnostic_workspace_source_scans
```

Expected:

```text
reference_duplicate_filter_steps == 0
workspace_symbol_source_shard_scans == 0
diagnostic_workspace_source_scans == 0
reference_line_indexes_built <= distinct result modules
```

## 20.5 Keep unrelated LSP architecture intact

Do not redesign:

- semantic-token lexical pass;
- completion syntax recovery;
- hover formatting;
- signature-help semantics;
- inlay-hint policy.

Only ensure they continue using compiler-owned semantic products and do not gain workspace scans.

## Tests

Large reference fixture:

```text
1000 reference sites
10 result modules
```

Assert:

```text
reference_line_indexes_built <= 10
reference_duplicate_filter_steps == 0
```

Also assert:

- imported definition/reference parity;
- workspace symbol parity;
- old snapshot location conversion remains old-snapshot correct;
- query disk reads/canonicalization remain zero where existing perf counters cover them.

## Completion criterion

Protocol conversion is O(result count + distinct result modules) and no LSP semantic fallback exists.

Suggested commit:

```text
perf(lsp): remove editor request scans and redundant conversion
```

---

# 21. Checkpoint B10 — Lifecycle, Parity, Performance, and Release Closure

**Risk:** HIGH  
**Type:** acceptance / correction only

B10 certifies B1–B9. Do not defer major architecture to this checkpoint.

## PB-1 — Presentation-only movement

Mutation:

- comments/whitespace move declarations/uses;
- semantic meaning unchanged.

Prove:

```text
semantic source fingerprint unchanged
presentation fingerprint changed

current source ranges updated

unrelated ModuleSourceIndex Arc retained
unrelated TargetReferenceSet Arc retained
changed module formal shard rebuilt
unrelated formal shard Arc retained

reference_targets_touched == 0
source_workspace_scan_units == 0
reference_workspace_scan_units == 0
formal_workspace_scan_units == 0
```

## PB-2 — Local references

Cover:

- method parameters;
- locals;
- closure parameters;
- closure captures;
- destructuring;
- loop/pattern bindings;
- shadowing;
- same-scope redeclaration;
- reads/writes.

Prove exact lexical target sets before and after range-moving edits.

## PB-3 — Imported binding

Keep existing cross-module definition tests green.

Add index proof:

```text
local import declaration → local Binding definition
downstream use → local lexical ref
downstream use → upstream semantic ref
go-to-definition → upstream definition
```

## PB-4 — Alias dual relation

For:

```phalcom
from a import Foo as Bar
Bar.make()
Bar.new()
```

prove:

```text
Bar declaration is local Binding definition
Bar uses are local lexical references
Bar uses are semantic references to Foo
Foo upstream definition excludes Bar declaration
```

## PB-5 — Missing target appears

Initial unresolved import, then add target.

Prove:

- Plan A resolution change triggers affected source projection;
- semantic relation appears;
- unrelated source/reference/formal shards remain retained;
- no full editor scan.

## PB-6 — Target disappears

Remove provider declaration/module.

Prove:

- stale external definition gone;
- stale semantic references gone;
- local unresolved import identity remains where syntax still declares it;
- workspace symbol removed;
- no stale formal/source site.

## PB-7 — Rename and delete/re-add

Sequence:

```text
Foo
→ Bar
→ delete
→ Foo again
```

Prove exact old contribution retirement after every mutation.

Do not use semantic-ID novelty as proof.

## PB-8 — Structural sharing

Assert pointer identity for unaffected:

- `ModuleSourceIndex`;
- module reference contribution;
- `TargetReferenceSet`;
- `ModuleFormalProjection`;
- workspace-symbol module contribution where observable.

## PB-9 — High fanout

### PB-9a provider body-only edit

Fixture:

```text
provider target Foo
500+ or practical 10k consumers reference Foo
```

Change provider body without public semantic identity change.

Expected:

```text
consumer source shards rebuilt == 0
reference target sets touched == 0
```

subject to exact Plan-A semantic effects.

### PB-9b one consumer reference edit

Change one consumer's use.

Expected:

```text
one consumer source contribution replaced
exact small target set touched
zero global source/reference scan
```

### PB-9c exported identity change

Change/remove exported identity.

Exact affected fanout may be large.

Require unrelated target sets and source shards remain untouched.

## PB-10 — Cold/incremental editor parity

Use a deterministic mutation sequence modeled after PA-10.

For each state compare incremental session vs fresh cold session:

```text
source structure
source semantic/presentation fingerprints
source sites/ranges
occurrence kinds/roles
exact lexical targets
import origins
definitions
lexical references
semantic references
formal fact sites/status
workspace-symbol entries/search
target_at
definition_locations
reference query outputs
selected LSP definition/reference/workspace-symbol results
```

Compare values/meaning, not `Arc` identity.

## PB-11 — Old snapshot immutability

Pin snapshot A.

Apply:

- range movement;
- import retarget;
- rename;
- deletion.

Publish B/C.

Query A again.

Require A returns its old:

```text
ranges
definitions
references
formal facts
workspace symbols
```

and newer snapshots return new state.

## PB-12 — Zero prohibited workspace scans

Use a large deterministic fixture.

After cold build, reset counters and perform one ordinary one-source edit.

Require:

```text
source_workspace_scan_units == 0
reference_workspace_scan_units == 0
formal_workspace_scan_units == 0

workspace_symbol_source_shard_scans == 0
diagnostic_workspace_source_scans == 0
```

Also record:

```text
source_modules_rebuilt
source_modules_reused
reference_targets_touched
reference_sites_added
reference_sites_removed
formal_modules_rebuilt
workspace_symbol_contributions_replaced
```

Assertions must be work-count based, not timing based.

Suggested commit:

```text
test(plan-b): certify retention parity and no-scan gates
```

---

# 22. Performance Complexity Contract

Let:

```text
N    = workspace modules
O    = all source occurrences
C    = all callables
S    = all workspace symbols

ΔM   = changed/removed source modules
ΔO   = occurrences in changed source contributions
T    = exact semantic targets touched by contribution changes
R(t) = reference count for target t
ΔF   = formal modules requiring rebuild
```

## Cold build

Allowed:

```text
O(N + O + C + S)
```

## Ordinary edit publication

Target:

```text
O(
    Plan-A exact semantic work
    + ΔO
    + persistent-map path updates
    + Σ R(t) for t ∈ T
    + formal facts in ΔF
)
```

Unacceptable as unconditional additive terms:

```text
O(N)
O(O)
O(C)
O(S)
```

## Definition query

Target:

```text
O(log targets + result_count)
```

or persistent-map equivalent.

## Reference query

Target:

```text
O(log targets + result_count)
```

plus protocol range conversion.

## Workspace symbol query

Must not traverse source shards.

Query cost may depend on candidate/result breadth, but not on reconstructing semantic source state.

---

# 23. Migration and Deletion Map

Plan B is incomplete if new indexes are added while broad old paths remain active.

## Remove/retire production use of

```text
SourceSemanticIndex::rebuild_target_occurrences

SourceSemanticIndex.target_occurrences
    after ReferenceIndex cutover

OccurrenceIndex.target_occurrences
    after no caller needs duplicate reverse lookup

previous.modules.clone()
    in incremental source publication

FormalSemanticProjection::from_callable_analyses_with_source_index
    as ordinary-update whole-workspace builder

EditorSemanticQuery::sites_for_target
    query-time definition classification

backend::compiler_workspace_symbols
    source-shard traversal

global/path-string source-index import resolution
    where exact ImportResolutionProduct exists
```

Cold-only helpers may remain if explicitly named as cold constructors and never called on ordinary edit publication.

## Make representation private

Prefer private:

```text
SourceSemanticIndex.modules
ReferenceIndex storage
FormalSemanticProjection.modules
WorkspaceSymbolIndex storage
```

Provide narrow read/query APIs.

## Snapshot assembly

Remove hidden global derived rebuilds from chained setters.

Preferred end state:

```text
session computes retained source index
session computes retained formal projection
session computes retained symbol/reference roots
session assembles immutable snapshot
```

---

# 24. Plan-A Preservation Requirements

Every checkpoint must preserve:

- `SemanticDb::record_dependency` current-revision rules;
- exact `QueryKey` dependencies;
- `ProductFingerprint` stability barriers;
- module transaction atomicity;
- import resolution invalidation;
- topology/component linking;
- declaration/callable/field identity;
- source/provider canonicalization.

Plan B must consume existing changed/removed/effected sets instead of inventing a second invalidation engine.

If extra module→semantic metadata is required, expose a retained Plan-A index across the boundary rather than reconstructing it by workspace scan.

---

# 25. File-by-File Patch Map

## Root

### `Cargo.toml`

- add persistent ordered collection dependency.

## `phalcom-modules`

### `phalcom-modules/src/session.rs`

Plan-B-only boundary work:

- expose retained importer→import-site index in `WorkspaceModuleUpdate` or equivalent.

Do not change resolution/linking algorithms.

## `phalcom-semantic`

### `phalcom-semantic/Cargo.toml`

- consume persistent-map dependency.

### `phalcom-semantic/src/workspace.rs`

- carry exact importer→site product into semantic input.

### `phalcom-semantic/src/source_index/mod.rs`

Major changes:

- persistent source-module root;
- module reference/symbol contributions;
- exact incremental replace/retire;
- stats;
- private representation;
- remove global reverse rebuild.

### `phalcom-semantic/src/source_index/reference.rs` — new

Own:

- `ModuleReferenceContribution`;
- `TargetReferenceSet`;
- `ReferenceIndex`;
- deterministic replacement algorithms.

### `phalcom-semantic/src/source_index/symbol.rs` — new

Own:

- protocol-neutral symbol types;
- module symbol contributions;
- persistent symbol search index.

### `phalcom-semantic/src/source_index/builder.rs`

- module-local context;
- exact import-site consumption;
- enum/type-reference coverage;
- no broad workspace context construction.

### `phalcom-semantic/src/source_index/occurrence.rs`

- exact import prefix targets;
- occurrence coverage;
- no candidate-set guessing;
- remove duplicate reverse index storage after cutover.

### `phalcom-semantic/src/source_index/scope.rs`

- richer helper accessors/provenance needed by contribution construction.

### `phalcom-semantic/src/presentation.rs`

- `ModuleFormalProjection`;
- persistent formal roots;
- exact replace/retire.

### `phalcom-semantic/src/session.rs`

Central Plan-B orchestration:

- derive exact source rebuild roots;
- rebuild changed module shards;
- retire removed shards;
- replace reference contribution;
- replace formal shard;
- replace workspace-symbol contribution;
- retain unaffected roots;
- publish exact stats;
- assemble snapshot without hidden global rebuild.

### `phalcom-semantic/src/snapshot.rs`

- attach already-built source/formal products;
- preserve immutable query API;
- remove hidden all-callable projection rebuild.

### `phalcom-semantic/src/editor.rs`

- direct definition/reference lookup;
- lexical/semantic reference domain;
- workspace-symbol query;
- rename-ready dual-target view.

## `phalcom-lsp`

### `phalcom-lsp/src/backend.rs`

- workspace symbol cutover;
- grouped source-location conversion;
- exact compiler queries only.

### `phalcom-lsp/src/perf.rs`

- Plan-B request work counters.

### `phalcom-lsp/Cargo.toml`

- register Plan-B integration test target.

---

# 26. Test File Map

## Extend semantic integration

```text
phalcom-semantic/tests/semantic/integration/source_index.rs
    occurrence coverage
    enum type references
    field writes
    import prefixes
    module contribution semantics

phalcom-semantic/tests/semantic/integration/imported_resolution.rs
    alias lexical vs semantic relations

phalcom-semantic/tests/semantic/integration/editor.rs
    indexed query behavior

phalcom-semantic/tests/semantic/integration/presentation.rs
    formal range rebasing
```

## New incremental suite

```text
phalcom-semantic/tests/semantic/incremental/plan_b_indexing.rs
```

Own:

```text
PB-1
PB-5
PB-6
PB-7
PB-8
PB-9
PB-10
PB-11
PB-12
```

## Existing LSP compatibility gates

```text
phalcom-lsp/tests/imported_binding_resolution.rs
phalcom-lsp/tests/module_navigation.rs
```

## New LSP Plan-B suite

```text
phalcom-lsp/tests/plan_b_indexing.rs
```

Own:

- indexed references;
- include-declaration behavior;
- workspace-symbol parity;
- location conversion work counts;
- no semantic source-shard scans.

---

# 27. Checkpoint Commit Strategy

Recommended commits:

```text
B0  test(plan-b): establish editor indexing baseline
B1  perf(source-index): add persistent module roots
B2  feat(source-index): add module-owned reference contributions
B3  perf(source-index): delta-maintain reverse target indexes
B4  feat(source-index): project exact import-site prefix provenance
B5  fix(source-index): close occurrence coverage gaps
B6  perf(presentation): retain module formal projection shards
B7  perf(editor): add incremental workspace symbol index
B8  refactor(editor): cut definitions and references to exact indexes
B9  perf(lsp): remove editor request scans and redundant conversion
B10 test(plan-b): certify retention parity and no-scan gates
```

Keep checkpoints independently reviewable unless repository workflow explicitly requires squashing.

---

# 28. Verification Commands

## Focused semantic

```bash
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic source_index
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic imported_resolution
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic editor
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic presentation
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic plan_b_indexing
```

If `plan_b_indexing` is a module inside the `semantic` integration target, use exact test filters rather than a separate test target.

## Module boundary

```bash
RUSTFLAGS='' cargo test -p phalcom-modules
```

Especially after B4.

## LSP focused

```bash
RUSTFLAGS='' cargo test -p phalcom-lsp --test imported_binding_resolution
RUSTFLAGS='' cargo test -p phalcom-lsp --test module_navigation
RUSTFLAGS='' cargo test -p phalcom-lsp --test plan_b_indexing
```

## Full release candidates

```bash
RUSTFLAGS='' cargo test -p phalcom-modules
RUSTFLAGS='' cargo test -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-lsp
```

## Compile

```bash
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo check -p phalcom-lsp
```

## Plan-A regression gates

Run the exact PA-9/PA-10/product-stability tests retained by the current repository after every checkpoint that touches `session.rs`, `snapshot.rs`, or semantic publication.

---

# 29. Final Grep/Audit Gates

Before declaring Plan B complete:

```bash
rg 'rebuild_target_occurrences' phalcom-semantic/src
rg 'previous\.modules\.clone\(\)' phalcom-semantic/src
rg 'source_index.*modules.*values\(\)' phalcom-lsp/src
rg 'from_callable_analyses_with_source_index' phalcom-semantic/src
```

Expected production status:

- no ordinary incremental caller of `rebuild_target_occurrences`;
- no source-index root full clone;
- no workspace-symbol source-shard scan;
- no ordinary all-callable formal rebuild.

Also audit broad loops:

```bash
rg 'for .* in .*modules|for .* in .*sources|values\(\)|retain\(' \
  phalcom-semantic/src/source_index \
  phalcom-semantic/src/presentation.rs \
  phalcom-lsp/src/backend.rs
```

Every broad traversal must be classified as one of:

```text
cold-only
explicit broad user query/result-proportional
bounded static Universe data
test-only
```

or removed.

---

# 30. Release Evidence Ledger

At completion record:

```text
Plan-B baseline commit
final Plan-B commit
git status
exact test commands
exact pass/fail/ignored counts
PB-1 ... PB-12 result matrix
large fixture sizes
pointer-retention evidence
performance counters
known unrelated failures
```

For PB-12 record at minimum:

```text
workspace modules
changed modules
source_modules_rebuilt
source_modules_reused
source_modules_retired
reference_targets_touched
reference_sites_added
reference_sites_removed
formal_modules_rebuilt
formal_modules_reused
workspace_symbol_contributions_replaced
source_workspace_scan_units
reference_workspace_scan_units
formal_workspace_scan_units
workspace_symbol_source_shard_scans
diagnostic_workspace_source_scans
```

“Incremental test passed” is not sufficient evidence.

---

# 31. Mandatory vs Optional Scope

## Mandatory

- persistent source-index root;
- module-owned source/reference contribution;
- exact definitions;
- lexical references;
- semantic/upstream references;
- delta replace/retire;
- exact import-site/prefix projection;
- occurrence coverage closure;
- enum type-reference closure;
- incremental formal projection;
- incremental workspace-symbol index;
- editor query cutover;
- LSP definition/reference/workspace-symbol cutover;
- old-snapshot immutability;
- cold/incremental parity;
- no-scan performance gates.

## Optional after architecture completion

- LSP `textDocument/rename`;
- advanced rename UX;
- fuzzy workspace-symbol ranking beyond current behavior;
- semantic-token delta protocol;
- completion ranking changes;
- hover formatting changes;
- unrelated type-system/runtime changes.

Even if rename is deferred, the compiler indexes must already distinguish local alias references from upstream semantic references.

---

# 32. Review Anti-Patterns

Reject patches that do any of the following.

## Incremental leaves, cloned root

```rust
let mut modules = previous.modules.clone();
```

## One global reverse rebuild after batching

```rust
for module in modules.values() {
    for occurrence in module.occurrences.all() {
        ...
    }
}
```

Batching once fixes a quadratic bug but still violates ordinary-edit Plan-B asymptotics.

## LSP cache over compiler scan

Caching workspace-symbol results in LSP does not solve compiler ownership/invalidation.

## Alias normalized away

Making imported alias uses target only the upstream declaration breaks lexical identity and rename semantics.

## Everything is one “reference” relation

Definition, lexical reference, semantic alias/reference, and import provenance are distinct.

## Range becomes semantic identity

Whitespace movement must not churn target relationships.

## Global stable `SourceSiteId`

Do not make snapshot-local source identity permanent.

## All-callable formal rebuild

Presentation rebasing must be module-delta maintained.

## Workspace symbol = declaration scan on request

Workspace symbols are a maintained index product.

## Candidate set becomes exact target

Completion/dynamic candidates never enter exact reference indexes.

---

# 33. Expected Final Behavior

## Ordinary range/body edit

```text
Plan A:
    exact semantic/module delta

Plan B source:
    rebuild edited module shard only
    retain unrelated source shards

Plan B references:
    touch only changed target relationships
    often zero work for presentation-only movement

Plan B formal:
    rebuild changed source/formal module only

Plan B symbols:
    replace changed module contribution only

snapshot:
    publish new persistent roots
    old snapshot remains valid
```

## Definition request

```text
cursor
→ occurrence interval lookup
→ exact target
→ definition index lookup
→ source location conversion
```

## Reference request

```text
cursor
→ exact lexical/semantic target
→ selected reference-domain index
→ source location conversion
```

## Imported alias

```text
Local use
    ├── lexical target → local Binding
    └── semantic target → upstream declaration

definition:
    follows import origin upstream

local rename/reference:
    lexical Binding relation

global semantic find references:
    upstream semantic relation
```

## Workspace symbol request

```text
query
→ persistent symbol postings
→ candidates/results
→ SourceSiteId locations
```

No source-shard discovery scan.

---

# 34. Definition of Done

Plan B is complete only when:

- [ ] implementation baseline is recorded and reconciled;
- [ ] no ordinary source-index publication clones the whole module map;
- [ ] no ordinary update globally rebuilds target occurrences;
- [ ] each source module owns an exactly replaceable/retirable reference contribution;
- [ ] definition, lexical-reference, and semantic-reference relations are distinct;
- [ ] imported aliases preserve local and upstream identity;
- [ ] import path segments use Plan-A prefix provenance;
- [ ] enum/type-reference occurrence coverage is complete for current syntax;
- [ ] no ordinary update rebuilds formal presentation from every callable;
- [ ] workspace symbols are incrementally indexed;
- [ ] editor definition/reference queries use direct indexes;
- [ ] LSP workspace symbols do not scan source shards;
- [ ] LSP reference conversion is linear in results/distinct result modules;
- [ ] delete/re-add leaves no stale navigation/reference/symbol state;
- [ ] presentation-only movement updates ranges without semantic target churn;
- [ ] unchanged source/reference/formal shards demonstrate structural sharing;
- [ ] PB-1 through PB-12 pass;
- [ ] PB-12 proves zero prohibited workspace scans;
- [ ] cold/incremental editor meaning is identical;
- [ ] old snapshots remain immutable/queryable;
- [ ] full module/semantic/LSP gates are recorded at one exact final revision;
- [ ] Plan-A performance/parity gates remain green.

The completed architecture should then have the intended two-part shape:

```text
Plan A
    exact module + semantic incrementality

Plan B
    exact editor + index publication incrementality
```

At that point an ordinary Phalcom edit no longer carries a hidden workspace-sized source/reference/presentation tax.
