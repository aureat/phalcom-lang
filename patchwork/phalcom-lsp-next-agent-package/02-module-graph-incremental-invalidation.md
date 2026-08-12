# Implementation Spec 2 — Module-Qualified Cross-File Semantics and Incremental Invalidation

**Repository:** `aureat/phalcom-lang`  
**Baseline commit:** `8f41ee4a7029f0617930cb01348454a111d072fb`  
**Prerequisite:** Spec 1 is green.  
**Primary package:** `phalcom-lsp`.

## 1. Goal

Replace the baseline “queue exists but whole workspace is recomputed” behavior with real dependency-scoped recomputation, while making module visibility match the Phalcom language model exactly.

This unit must preserve coherent semantic generations and live provider edits, including provider creation/removal and `didClose` disk restoration.

## 2. Language/module rules that are not negotiable

From `docs/spec/current/modules.md`:

- every `.ph` file is a module;
- imports are resolved relative to the importing file;
- `.ph` is appended only when the path has no extension;
- canonicalized paths identify modules;
- `import "./x" as X` binds the whole module only in the importing scope;
- imported names do **not** merge into the importer’s globals;
- core classes are the only automatically visible cross-module class surface.

Therefore a class that happens to be unique in the workspace is **not** visible without an import.

## 3. Read only these files first

1. `docs/spec/current/modules.md`, sections 1–7.
2. `phalcom-lsp/src/semantic/module_graph.rs`
   - `ImportEdge`
   - `ModuleGraph::{update,remove,dependents_of,refresh_resolutions,dependent_closure}`
   - `resolve_import`
3. `phalcom-lsp/src/semantic/invalidation.rs`
   - `InvalidationQueue`
4. `phalcom-lsp/src/semantic/mod.rs`
   - `FileSemanticSnapshot`
   - `SemanticState`
   - `SemanticDb::{update_file,remove_file}`
   - `rebuild_state`
   - `resolve_named_class`
5. `phalcom-lsp/src/backend.rs`
   - `scan_workspace`
   - `refresh_closed_file`
   - `remove_indexed_file`
   - `did_change_watched_files`
   - `did_close`
6. `phalcom-lsp/tests/workspace_semantics.rs` and the supplied regression patch.

Only if implementing the shared path helper in §6.2, run this targeted command and read only the matching import-resolution implementation files:

```sh
rg -n "import_module|resolve.*import|canonicalize.*import|with_extension\(\"ph\"\)" \
  phalcom-core/src phalcom-common/src
```

Do not scan examples, core source, or VS Code code.

## 4. Baseline defects

### 4.1 `InvalidationQueue` is drained and ignored

Baseline `semantic/mod.rs::rebuild_state` starts with:

```rust
let _affected = queue.drain().collect::<std::collections::BTreeSet<_>>();
let inputs = state.files.values() /* every file */;
```

The entire workspace is then re-solved and all local/field facts are recomputed. The queue currently has no semantic effect.

This violates the design/testing requirement that a leaf edit not force a whole-workspace semantic rebuild.

### 4.2 Startup repeatedly rebuilds the workspace

`Backend::scan_workspace` calls `SemanticDb::update_file` once per discovered `.ph` file. Because each `update_file` currently rebuilds every file accumulated so far, startup performs repeated global solving.

Add a batch publication path.

### 4.3 `resolve_named_class` leaks globally unique unimported classes

The baseline ends `resolve_named_class` with a workspace-wide fallback that accepts a bare name when only one matching class exists globally.

Delete that semantic behavior. It contradicts module scope.

Allowed bare class resolution is only:

1. local module;
2. core module.

Imported classes require the explicit module binding path already supported by the `binding.class` lookup.

### 4.4 Import-resolution changes returned by `refresh_resolutions` are ignored

`ModuleGraph::refresh_resolutions()` returns importers whose target changed. `SemanticDb::update_file` currently calls it without consuming the return value.

The global rebuild masks this bug. An incremental implementation will be stale unless those importers are queued.

### 4.5 Provider removal can lose reverse dependents before they are captured

On removal, the graph is mutated/refreshed before `dependent_closure(&removed_module)` is used. Once an import edge becomes unresolved, it no longer points to the removed target, so old dependents may disappear from the reverse query.

Capture old dependents before graph mutation.

### 4.6 Import resolution always calls `.with_extension("ph")`

The current helper replaces an existing extension. The language rule is “append `.ph` if there is no extension.” Match the compiler/runtime rule, do not create an LSP-only dialect.

### 4.7 Aggregated parameter facts are not source-owned

True incremental invalidation needs to remove stale call-site contributions from only the changed caller module. A single aggregated map cannot tell which caller contributed which fact.

Store per-module parameter contributions.

## 5. Required state model

Edit `semantic/mod.rs`.

Add source-owned call-site contributions:

```rust
struct SemanticState {
    generation: SemanticGeneration,
    files: BTreeMap<ModuleId, FileSemanticSnapshot>,
    classes: BTreeMap<ClassId, ClassSurface>,
    summaries: BTreeMap<CallableId, CallableSummary>,
    field_facts: BTreeMap<(ClassId, String), InferredValue>,
    parameter_facts: BTreeMap<(CallableId, String), InferredValue>,

    // NEW: facts contributed by call sites physically contained in each module.
    parameter_contributions: BTreeMap<ModuleId, ParameterFacts>,

    callable_dependents: BTreeMap<CallableId, BTreeSet<CallableId>>,
    graph: ModuleGraph,
}
```

The aggregate `parameter_facts` remains useful for queries, but it must be rebuilt by joining `parameter_contributions`, never by overwriting.

### 5.1 Contribution semantics

When module `M` changes:

1. recompute only `parameter_contributions[M]`;
2. replace the old contribution wholesale;
3. rebuild the aggregate for affected target callables by joining all source-module contributions;
4. if a target parameter changes, enqueue that callable and its reverse callable dependents.

When module `M` is removed:

1. remove `parameter_contributions[M]`;
2. rejoin any parameter targets to which `M` used to contribute;
3. enqueue targets whose aggregate changed.

This gives stale-fact removal without rescanning unrelated callers.

## 6. Module graph redesign

### 6.1 Resolve against known semantic modules, not only filesystem existence

A live LSP can hold an open `file://` document before the filesystem watcher settles. Resolution should use the semantic database’s available module set as authority, while path construction follows language rules.

Change module-graph resolution to separate:

- **candidate module identity** derived from importer + import string;
- whether that candidate is currently present in semantic state.

Recommended helper:

```rust
fn import_candidate(module: &ModuleId, import: &str) -> Option<ModuleId> {
    let uri = Url::parse(module.as_str()).ok()?;
    let source = uri.to_file_path().ok()?;
    let mut candidate = source.parent()?.join(import);
    if candidate.extension().is_none() {
        candidate.set_extension("ph");
    }
    let normalized = normalize_path(candidate);
    Url::from_file_path(normalized).ok().map(|uri| ModuleId::from_uri(&uri))
}
```

Then make graph update/refresh accept the available-module set:

```rust
pub fn update(
    &mut self,
    module: ModuleId,
    program: &Program,
    available: &BTreeSet<ModuleId>,
)

pub fn refresh_resolutions(
    &mut self,
    available: &BTreeSet<ModuleId>,
) -> Vec<ModuleId>
```

Each edge keeps `path`; its `target` is:

```rust
let candidate = import_candidate(module, &edge.path);
edge.target = candidate.filter(|id| available.contains(id));
```

If an existing compiler/common helper already implements the exact same path rule and is VM-free, extract/reuse it instead of duplicating the snippet above.

### 6.2 Shared path helper decision

Preferred final architecture:

```text
phalcom-common (VM-free import path normalization)
        ↑                         ↑
  phalcom-core/compiler      phalcom-lsp
```

Do not make `phalcom-lsp` depend on VM/runtime crates.

If moving the compiler helper would require broad runtime migration, keep a small LSP helper for this patch but add a TODO naming the exact common extraction follow-up. Correct behavior is mandatory; broad runtime churn is not.

## 7. Strict semantic class resolution

Replace the tail of `resolve_named_class` with exactly this policy:

```rust
fn resolve_named_class(
    classes: &BTreeMap<ClassId, ClassSurface>,
    graph: &ModuleGraph,
    module: &ModuleId,
    name: &str,
) -> Option<ClassId> {
    if let Some((binding, class_name)) = name.split_once('.') {
        let imported = graph
            .imports(module)
            .iter()
            .find(|edge| edge.binding == binding)
            .and_then(|edge| edge.target.as_ref())?;
        let class = ClassId::new(imported.clone(), class_name);
        return classes.contains_key(&class).then_some(class);
    }

    let local = ClassId::new(module.clone(), name);
    if classes.contains_key(&local) {
        return Some(local);
    }

    let core = ClassId::new(ModuleId::new(CORE_MODULE_URI), name);
    classes.contains_key(&core).then_some(core)
}
```

Delete the “globally unique class” fallback.

Do not compensate by querying `WorkspaceIndex`.

## 8. Invalidation frontier

### 8.1 Update/create

In `SemanticDb::update_file`:

1. snapshot old graph/import information needed for reverse invalidation;
2. insert/replace the new file snapshot and class surface;
3. update the module’s graph edges;
4. refresh graph resolutions against current `state.files.keys()`;
5. seed an `InvalidationQueue` with:
   - changed module;
   - all importers returned by `refresh_resolutions`;
   - transitive dependents of changed module after the update;
   - modules owning reverse callable dependents of callables changed/removed in this module;
6. expand the frontier until no new module is added;
7. recompute only the frontier.

### 8.2 Remove

Before removing graph/file state:

```rust
let old_dependents = state.graph.dependent_closure(&module);
let old_callables = state
    .summaries
    .keys()
    .filter(|id| id.owner.module == module)
    .cloned()
    .collect::<Vec<_>>();
```

Also capture modules owning reverse callable dependents of `old_callables`.

Then remove the module, refresh import resolutions, and queue the union of:

- `old_dependents`;
- changed importers from `refresh_resolutions`;
- old callable-dependent modules;
- any new dependent closure induced by resolution changes.

This ordering is required. Do not ask the graph for old dependents after erasing the edges that establish them.

## 9. Replace global `rebuild_state` with affected recomputation

Rename the baseline function to make the new contract explicit:

```rust
fn rebuild_affected_state(
    state: &mut SemanticState,
    generation: SemanticGeneration,
    affected: BTreeSet<ModuleId>,
)
```

### 9.1 What may be reused from unaffected modules

For a module not in `affected`, preserve:

- `FileSemanticSnapshot.local_facts`;
- its class/member surface;
- source-owned field facts;
- source-owned parameter contributions;
- callable summaries unless one is in the reverse callable frontier.

Do not re-run AST inference for it.

### 9.2 What must be recomputed for affected modules

For each affected module:

- parameter contribution from that module’s call sites;
- callable summaries owned by affected classes;
- local binding facts if any referenced summary/field input changed;
- field facts owned by classes in that module;
- dependency metadata.

Then use Spec 1’s pure solver over the affected callables plus any required boundary summaries from unaffected modules.

### 9.3 Boundary rule

An unaffected callable summary is a read-only boundary value for the affected solver.

If an affected summary changes, use `callable_dependents` to enqueue its reverse dependents. If that reaches a new module, add that module to the affected frontier and continue.

This is the real work-list behavior required by the design.

## 10. Batch workspace scan

Add a batch API so startup does not repeatedly solve prefixes of the workspace.

Recommended API in `SemanticDb`:

```rust
pub fn update_files_batch(
    &self,
    files: Vec<(Url, FileRevision, Program)>,
) -> SemanticGeneration
```

Implementation:

1. acquire one write lock;
2. install all source snapshots/surfaces;
3. build/refresh graph once;
4. solve all newly added modules as one frontier;
5. publish one generation.

Change `Backend::scan_workspace` to:

- keep updating the legacy `WorkspaceIndex` per file as today;
- collect semantic inputs into a vector;
- call `semantic.update_files_batch(...)` once after the filesystem walk.

Do not call `update_file` N times for an N-file initial scan.

## 11. Superclass scope

`surface.rs::build_module_surface` currently constructs superclass IDs in the same module. Do not invent a new qualified superclass grammar in this unit.

Required behavior now:

- local superclass works;
- core superclass fallback works through semantic lookup where already supported;
- cross-module inheritance is added only if the current parser grammar explicitly supports naming an imported class in the `is` clause.

If grammar does not support it, add a focused TODO and test the supported scope only.

## 12. Tests

### 12.1 Apply supplied regression tests

The patch contains:

- `unimported_workspace_class_is_not_semantic_authority_for_hover`.

It must become green after the global class fallback is removed and Spec 3’s conservative hover behavior is applied.

### 12.2 Add SemanticDb tests

Add test-only instrumentation rather than timing assertions.

Under `#[cfg(test)]`, expose a compact rebuild trace:

```rust
#[cfg(test)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RebuildTrace {
    pub modules_recomputed: BTreeSet<ModuleId>,
    pub callables_recomputed: BTreeSet<CallableId>,
}
```

Store the last trace in test builds only.

Add tests:

1. `leaf_edit_does_not_recompute_unrelated_module`
2. `provider_edit_recomputes_transitive_consumers`
3. `provider_creation_repairs_previously_unresolved_import`
4. `provider_removal_invalidates_existing_importer`
5. `caller_edit_removes_stale_parameter_contribution`
6. `unimported_unique_workspace_class_does_not_resolve`
7. `same_named_imported_classes_remain_module_qualified`
8. `cyclic_import_graph_terminates_without_panic`
9. `import_with_existing_ph_extension_is_not_rewritten`

### 12.3 RPC lifecycle tests

In `workspace_semantics.rs`, add/retain:

- provider `didChange` changes consumer completion without restart;
- watched provider creation repairs an unresolved import;
- watched provider deletion removes stale consumer facts;
- open unsaved provider content beats disk;
- `didClose` restores disk-backed semantics.

## 13. Implementation sequence

1. Remove global unique-class fallback; make affected tests fail/pass deterministically.
2. Correct import candidate/path semantics.
3. Make graph resolution depend on available semantic modules and consume `refresh_resolutions` results.
4. Add source-owned `parameter_contributions`.
5. Capture old dependents before removal.
6. Implement affected-frontier recomputation.
7. Add test-only rebuild trace and exact invalidation tests.
8. Add `update_files_batch`; change `scan_workspace` to use it.
9. Run focused tests before full suite.

## 14. Commands

```sh
cargo test -p phalcom-lsp semantic::module_graph
cargo test -p phalcom-lsp semantic::
cargo test -p phalcom-lsp --test integration workspace_semantics
cargo test -p phalcom-lsp --test integration semantic_completion
cargo test -p phalcom-lsp --test integration semantic_consistency
cargo test -p phalcom-lsp
cargo clippy -p phalcom-lsp --all-targets -- -D warnings
```

## 15. Completion criteria

This unit is complete only when:

- the invalidation queue determines actual recomputation work;
- unrelated modules are demonstrably not re-inferred on a leaf edit;
- cross-file parameter contributions can be removed/rejoined by source module;
- provider creation/removal repairs/invalidate importers;
- unimported workspace classes never become semantic authority;
- import path handling matches the language specification;
- initial workspace scan solves once in batch rather than once per discovered file;
- all semantic publications remain generation-coherent.
