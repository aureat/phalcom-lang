# Fresh-Session Handoff Prompt — Plan Phalcom Step 7 and Write the Implementation Specification

You are taking over an ongoing, deeply repository-grounded compiler/LSP/incremental-semantics architecture project for the programming language **Phalcom**.

Your task in this fresh session is:

> **Verify the implementation state after Step 6, then plan Step 7 and produce a detailed repository-grounded Step-7 implementation specification.**

Do **not** begin by drafting the specification.

First perform a fresh repository archaeology and verification pass against the **current `main` branch** of `aureat/phalcom-lang`.

The repository is the implementation source of truth. The attached prior specifications are architectural context and decision history, but their implementation-state observations may be stale. They are not permission to assume that APIs, file paths, tests, or defects still have the same shape.

The quality bar is extremely high. A shallow conceptual plan, a paraphrase of the old architectural spec, invented APIs, or a plan based only on filenames is unacceptable.

---

# 1. Deliverables

Produce one final downloadable markdown document:

```text
phalcom-step-7-compiler-owned-module-lifecycle-implementation-spec.md
```

The document must be an implementation specification, not an essay.

It must include:

1. verified current repository state;
2. verified Step-6 postconditions;
3. exact remaining Step-7 defects;
4. target ownership/lifecycle model;
5. exact files to modify/create/delete;
6. exact existing APIs to reuse;
7. exact new/changed APIs where justified;
8. query topology and dependency ownership;
9. lifecycle/generation/invalidation semantics;
10. source-overlay semantics;
11. canonical physical-path ↔ logical-module identity semantics;
12. project-universe persistence semantics;
13. snapshot/module-product publication semantics;
14. error/outcome behavior;
15. cold-wrapper compatibility behavior;
16. a TDD implementation sequence with reviewable slices;
17. concrete regression tests;
18. static deletion/audit gates;
19. performance/incrementality acceptance gates;
20. executable Cargo/fmt/clippy/workspace verification commands;
21. explicit non-goals separating Step 7 from Steps 8+;
22. a final acceptance checklist;
23. recommended commit boundaries.

Do **not** implement Step 7 in this session unless explicitly asked later.

---

# 2. Historical anchor — do not use as the fresh baseline

At the time this handoff prompt was written, GitHub `main` had advanced to:

```text
23e6ca126e96b11504a275fa3e777e18fe4d9ef5
test(semantic): cover incrementality hardening regressions
```

That commit is the final Step-5.5 regression commit in this historical sequence:

```text
04ec157  fix(semantic-db): enforce generic dependency reuse
b67aea9  fix(semantic): add declaration shell products
78dab25  fix(semantic): cache declaration surfaces by source contract
cddfedb  fix(semantic): evaluate callable signature prerequisites on demand
1e4337f  fix(semantic): fingerprint callable bodies by semantic facts
23e6ca1  test(semantic): cover incrementality hardening regressions
```

**This is not your expected Step-7 baseline.**

The user is expected to implement/commit Step 6 before starting the fresh Step-7 session.

Therefore your first job is to determine the actual current HEAD and identify the Step-6 commit(s).

Run or query equivalent information for:

```bash
git status --short
git rev-parse HEAD
git log -n 20 --oneline
git show --stat --oneline HEAD
```

If GitHub is the only repository access available, use the GitHub connector extensively.

Never say “Step 6 is complete” just because its specification is attached.

Prove it from source and tests.

---

# 3. Read the attached prior documents in this targeted order

The user will attach the prior implementation specifications and verification documents.

Do **not** read all several-thousand-line files linearly first.

Start with the following exact ranges. Expand only when archaeology shows that a skipped section is necessary.

## 3.1 `phalcom-step-6-db-projected-formal-read-model-implementation-spec.md`

Read:

```text
lines 1–305
```

Purpose:

- Step-6 goal;
- hard precondition;
- why query ownership was still insufficient;
- source-semantic authority rules;
- current-product requirement;
- structural-sharing invariants;
- Step-6 non-goals.

Then read:

```text
lines 337–549
```

Purpose:

- `current_product` / `current_products`;
- current-validity semantics;
- the `core.generic_super` hierarchy bug;
- required hierarchy correction.

Then read:

```text
lines 808–1457
```

Purpose:

- declaration/hierarchy/member materialization;
- snapshot projection metadata;
- coherent production snapshot constructor;
- the required session producer/consumer phases;
- declaration-removal semantics.

Then read:

```text
lines 2243–2622
```

Purpose:

- performance acceptance matrix;
- code-quality constraints;
- interaction with Step 5.5;
- generic inheritance;
- dispatch/body implications;
- exactly what Step 7 may assume;
- Step-7 issues deliberately deferred from Step 6.

Finally read:

```text
lines 2622–2696
```

Purpose:

- Step-6 acceptance checklist;
- recommended implementation commits.

Do not assume the implementation followed this design exactly. Verify.

---

## 3.2 `phalcom-step-5.5-semantic-incrementality-hardening-implementation-spec.md`

Read:

```text
lines 12–53
```

Purpose: Step-5.5 executive decision.

Read:

```text
lines 161–339
```

Purpose:

- `DeclarationSurface` previously stood in for declaration metadata;
- inactive `DeclarationShell`;
- callable-body prewarming;
- presentation-sensitive body fingerprints;
- query-specific DB reuse exception;
- persistent kind identity.

Read:

```text
lines 394–594
```

Purpose:

- generic reuse law;
- direct input vs result identity;
- declaration type identity as its own dependency product;
- target post-5.5 query topology.

Read:

```text
lines 596–948
```

Purpose:

- activated `DeclarationShell`;
- fingerprint/query/publication model;
- dependency vocabulary;
- type-resolution and generic-substitution dependency tracking.

Read:

```text
lines 1241–1594
```

Purpose:

- shell activation before surfaces;
- demand-driven callable signature prerequisites;
- semantic-only body product fingerprint;
- deletion of the `DeclarationSurface -> ParsedModule` DB exception.

Read:

```text
lines 2267–2402
```

Purpose:

- post-Step-5.5 architecture;
- exactly what Step 6 was allowed to assume.

---

## 3.3 `phalcom-steps-1-5.5-verification-2026-08-24.md`

Read:

```text
lines 304–490
```

Purpose:

- the remaining authority problem identified before Step 6;
- the hierarchy-template bug;
- Step-7 substrate that was already present early;
- sequencing decision: Step 6 before compiler-owned module lifecycle.

Note: the beginning of this verification report contains a now-stale statement that Step 5.5 was not yet visible remotely. It became visible later at `23e6ca...`. Do not repeat that stale observation.

---

## 3.4 `0005-product-stability-fine-grained-invalidation-verification.md`

Read:

```text
lines 370–386
```

Purpose:

- import/export propagation was still transitional;
- linked-interface dependencies remained coarse;
- declaration-surface reuse limitation that Step 5.5 later fixed.

---

## 3.5 `0004-db-owned-formal-semantic-queries-verification.md`

Read:

```text
lines 181–193
```

Purpose:

- deliberately deferred ownership findings after DB formal-query work.

---

## 3.6 `0003-semantic-read-dependency-capture-verification.md`

Read:

```text
lines 258–268
```

Purpose:

- transitional dependency/read limitations;
- what was intentionally left for later query ownership.

---

## 3.7 Original architectural completion specification

If attached:

```text
phalcom_compiler_lsp_incremental_semantics_architectural_completion_spec.md
```

Do **not** read it as implementation truth.

Locate and read the section:

```text
# 12. Task 7 — Compiler-owned project/module lifecycle
```

through immediately before:

```text
# 13. Task 8 — Delete LSP formal workspace reconstruction
```

Also read the top-level acceptance invariants and target ownership graph.

Treat Task 7 as the architectural starting hypothesis, then revise it against current code.

A large amount of Task-7 substrate was already implemented before the Step-7 lifecycle itself, so blindly restating the old Task-7 section will produce a bad plan.

---

# 4. Architectural sequence you are inheriting

The intended sequence is:

```text
Step 1
  query validity + fail-closed dependencies

Step 2
  semantic product fingerprint redesign

Step 3
  semantic-read dependency capture

Step 4
  DB ownership of hierarchy / declaration surface / callable signature

Step 5
  product-stability propagation + fine-grained invalidation

Step 5.5
  incrementality hardening:
    generic DB reuse law
    DeclarationShell
    pre-resolution surface reuse
    demand-driven signature prerequisites
    semantic-only body product identity

Step 6
  DB-projected formal read model:
    current validated product view
    immutable projection materialization
    source-state authority exclusively from DB products
    coherent body/snapshot read model
    structural sharing
    no fake generic superclass

Step 7
  compiler-owned project/module/source lifecycle

Step 8
  delete LSP formal workspace reconstruction

Step 9+
  canonical module diagnostics/completion/navigation and remaining editor identity work
```

Do not collapse Step 8 or later work into Step 7 simply because the final architecture needs it.

Step 7 must create the compiler-owned APIs and lifecycle that Step 8 can consume.

---

# 5. Phalcom architecture principles that matter here

These principles are non-negotiable unless current repository evidence proves the architecture has deliberately changed.

## 5.1 `phalcom-modules` owns module semantics

`phalcom-modules` is the sole implementation authority for:

- project identity;
- manifest loading/validation;
- project dependency graphs;
- import roots;
- module spelling;
- logical `ModuleId`;
- physical source location;
- package semantics;
- exposure rules;
- import resolution;
- unlinked interfaces;
- linked interfaces;
- exports;
- module linking.

Do not move these algorithms into `phalcom-semantic`.

`phalcom-semantic` owns lifecycle, revisioning, dependency tracking and publication of module products.

---

## 5.2 `SemanticWorkspaceSession` owns the formal workspace lifetime

After Step 7, the session—not the LSP—should own the persistent formal state required to interpret source edits:

```text
one SemanticDb
one mutable TypeStore
one project-universe generation
one canonical source-provider/overlay state
one sequence of semantic revisions
one publication/LKG boundary
```

The LSP later becomes a scheduler/client of this session.

---

## 5.3 “Persistent compiler-owned lifecycle” does not mean persistent resolver/linker objects

Be careful here.

Current `ModuleResolver<'u, P>` borrows a `ProjectUniverse` and `SourceProvider`.

Putting a borrowing resolver inside the same struct that owns those values creates a self-referential Rust design problem.

Do not design Step 7 around storing:

```rust
ModuleResolver<'self, ...>
```

inside `SemanticWorkspaceSession`.

Also, `ModuleLinker` is largely a computation object, not workspace identity.

The important persistence is:

```text
ProjectUniverse
source provider / overlays
resolver generation
DB products
semantic revision state
```

It is acceptable—and probably preferable—for the semantic session/query layer to instantiate an ephemeral resolver/linker against persistent canonical inputs when a module query actually needs them.

The forbidden construction in the final architecture is the **LSP independently rebuilding formal module infrastructure**, not the semantic owner constructing short-lived algorithm objects internally.

Verify exact current types before finalizing this conclusion.

---

## 5.4 Query products, not mutable helper caches, are semantic authority

After Step 6:

```text
DB products -> immutable projection -> checker/snapshot
```

Step 7 must preserve that.

Module resolver/linker private caches are implementation caches only.

They cannot become a second semantic product authority.

---

## 5.5 Every consumed semantic product needs a real dependency relation

If a module query semantically depends on:

```text
UnlinkedInterface(package)
ResolvedImports(importer)
SemanticComponent(entry)
LinkedInterface(module)
```

the DB graph must represent the consumed product.

Do not “record” a dependency on a DB product if the computation actually read an unrelated separately-built semantic value and merely assumes equivalence.

This point is especially important for package exposure resolution. See §10.5 below.

---

# 6. Mandatory fresh repository archaeology

Inspect current `main`, not only attached specs.

At minimum inspect these files completely enough to understand ownership and call flow.

## `phalcom-modules`

```text
phalcom-modules/src/lib.rs
phalcom-modules/src/identity.rs
phalcom-modules/src/stabilization.rs
phalcom-modules/src/project.rs
phalcom-modules/src/manifest.rs
phalcom-modules/src/source.rs
phalcom-modules/src/resolver.rs
phalcom-modules/src/interface.rs
phalcom-modules/src/linker.rs
phalcom-modules/src/query.rs
phalcom-modules/src/error.rs
```

Inspect related tests, especially:

```text
phalcom-modules/tests/
```

Search for:

```text
SourceOverlay
OverlaySourceProvider
FilesystemSourceProvider
clear_cache
generation
ResolverGeneration
ResolvedDocumentIdentity
ProjectUniverse
load_root
load_synthetic_root
discover_owning_project
ModuleResolver
resolve_import_with_trace
load_package_surface
ModuleLinker
ModuleQueryFacade
import_root_entries
module_children
external_import_children
resolve_relative_prefix
```

---

## `phalcom-semantic`

Inspect:

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/workspace.rs
phalcom-semantic/src/workspace_inputs.rs
phalcom-semantic/src/materialize.rs        # expected after Step 6; verify name
phalcom-semantic/src/module_product.rs
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/mod.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
```

Inspect all Step-6 tests and current module/session tests.

Search for:

```text
SemanticWorkspaceInput
SemanticWorkspaceSession
ProjectUniverse
OverlaySourceProvider
SourceChange
WorkspaceRootInput
SourceOverlayUpdate
WorkspaceSessionError
ResolvedImports
SemanticComponent
LinkedInterface
ModuleQueryProducts
InterfaceBuilder
ModuleResolver
ModuleLinker
last_known_good
last_published
```

---

## `phalcom-lsp`

Step 8, not Step 7, owns deletion of the LSP reconstruction path.

But Step 7 APIs must be designed for the real consumer.

Inspect:

```text
phalcom-lsp/src/analysis_service.rs
phalcom-lsp/src/import_completion.rs
phalcom-lsp/src/semantic/snapshot.rs
phalcom-lsp/src/semantic/module_graph.rs
phalcom-lsp/src/semantic/ids.rs
phalcom-lsp/src/semantic/occurrence.rs
```

Search for:

```text
StaticWorkspaceIdentity
refresh_static_workspace_analysis
run_static_workspace_analysis
ProjectUniverse::new
ModuleResolver::new
ModuleLinker::new
ResolvedDocumentIdentity
ResolverGeneration
```

Do not delete those in Step 7 unless Step-7 implementation genuinely requires a tiny compatibility adjustment.

The main LSP migration belongs to Step 8.

---

## `phalcom-core`

Inspect module/compiler consumers only enough to prevent API breakage:

```text
phalcom-core/src/modules/
```

Especially search for:

```text
ProjectUniverse
ResolverGeneration
ResolvedDocumentIdentity
ModuleResolver
ModuleLinker
FilesystemSourceProvider
```

The compiler/runtime module pipeline may rely on APIs you are about to modify.

Do not optimize exclusively for the LSP.

---

# 7. Verify Step 6 before planning Step 7

Do not merely inspect commit messages.

Prove these postconditions from current code/tests:

1. `SemanticDb` exposes a way to enumerate/read only products validated for the current revision.
2. source declaration tables in snapshots are projections of current `DeclarationShell` products;
3. direct hierarchy is projected from current `HierarchyEdge` products;
4. generic supertype templates do not fabricate `core.generic_super`;
5. source surfaces are projected from current `DeclarationSurface` products;
6. canonical callable signature tables are projected from current `CallableSignature` products;
7. body checking and snapshot publication consume the same projected `Arc`s;
8. body-only edits structurally reuse stable formal projections;
9. stale old Ready products from removed declarations cannot leak into a new snapshot;
10. the session no longer treats independently rebuilt source declaration/hierarchy/dispatch/signature tables as formal authorities.

If any fail, stop and report that Step 6 is incomplete.

The Step-7 specification must not route around an incomplete Step 6.

---

# 8. Known Step-7 substrate that already existed before Step 7

This is one of the most important handoff facts.

Do not waste Step 7 reimplementing these concepts without checking current code.

At `23e6ca...`, the repository already had:

## 8.1 Source overlays

In:

```text
phalcom-modules/src/source.rs
```

there were already:

```rust
SourceOverlay
OverlaySourceProvider<P>
```

with overlay-aware:

```text
locate
read
set_overlay
remove_overlay
clear_overlays
```

This substrate exists, but has correctness issues described later.

---

## 8.2 Traced import resolution

In:

```text
phalcom-modules/src/resolver.rs
```

there were already:

```rust
ImportResolutionTrace {
    target: SourceUnit,
    package_interfaces: BTreeSet<ModuleId>,
}
```

and:

```rust
resolve_import_with_trace(...)
```

External hierarchical exposure validation records package interfaces consulted.

The compatibility `resolve_import(...)` projects to the target.

Do not re-spec this as if absent.

---

## 8.3 Pure immutable module queries

In:

```text
phalcom-modules/src/query.rs
```

there were already:

```rust
ModuleQueryFacade
ImportRootQueryTarget
RelativeQueryError
```

and pure query methods including:

```text
import_root_entries
import_roots
module_children
external_import_children
resolve_relative_prefix
public_exports
resolved_import_target
definition_source
reverse_importers
```

This is important infrastructure for later completion/navigation.

Step 7 should make snapshots feed it canonical products.

Do not move completion policy into Step 7.

---

## 8.4 Snapshot module product bundle

In:

```text
phalcom-semantic/src/snapshot.rs
```

there was already:

```rust
ModuleQueryProducts {
    universe,
    unlinked,
    linked,
    resolved_imports,
    sources,
}
```

and:

```rust
SemanticSnapshot::module_queries()
```

Again: the problem is not absence of the data structure.

The problem is lifecycle/authority and how those maps are produced.

---

## 8.5 Workspace input model skeleton

In:

```text
phalcom-semantic/src/workspace_inputs.rs
```

there were already:

```rust
WorkspaceRootInput
SourceOverlayUpdate
SourceChange
WorkspaceSessionError
```

They were exported from `phalcom-semantic`.

At the historical baseline they were largely skeletal/unintegrated.

Do not create duplicate types with new names until you inspect how Step 6/current main changed them.

---

## 8.6 Resolver/document identity types

In:

```text
phalcom-modules/src/stabilization.rs
```

there were already:

```rust
ResolverGeneration(pub u64)

ResolvedDocumentIdentity {
    source: SourceId,
    module: ModuleId,
    generation: ResolverGeneration,
}
```

There is already a typed generation concept.

Do not introduce a second uncoordinated resolver-generation abstraction.

---

# 9. Core target for Step 7

The intended ownership shape is approximately:

```text
SemanticWorkspaceSession
  |
  |-- SemanticDb
  |-- TypeStore
  |
  |-- Arc<ProjectUniverse>
  |-- canonical overlay-capable SourceProvider
  |-- resolver/project generation
  |-- workspace roots / project-root state
  |-- document revisions / source overlay state
  |
  |-- last published immutable SemanticSnapshot
  |
  +--> invokes phalcom-modules algorithms
         InterfaceBuilder
         ModuleResolver
         ModuleLinker
  |
  +--> owns semantic module query scheduling
         ParsedModule
         UnlinkedInterface
         ResolvedImports
         SemanticComponent
         LinkedInterface
         downstream formal queries
```

Ordinary body/source edits should not rebuild the project universe.

Project/manfiest/root changes may advance a module-resolution generation and replace/share the project universe appropriately.

The immutable snapshot should retain the exact canonical module products for its generation.

---

# 10. Important defects and design traps already identified

These are not optional trivia. Investigate every one before writing the Step-7 spec.

Some may already have been fixed by the time of the fresh session. If so, record the fix and remove it from proposed work.

---

## 10.1 Overlay provider has a lock-order inversion

Historical implementation used two `RwLock`s:

```text
overlays_by_module
overlays_by_source
```

Mutation path acquired:

```text
module map -> source map
```

while `read()` acquired:

```text
source map -> module map
```

That is a classic lock-order inversion and can deadlock under concurrent reads/writes.

Preferred direction to investigate:

```rust
RwLock<OverlayState {
    by_module,
    by_source,
}>
```

so both indexes update atomically under one lock.

At minimum there must be one globally consistent lock order.

Do not hand-wave this as “unlikely in the LSP.”

---

## 10.2 Overlay replacement can leave a stale reverse source mapping

Historical `set_overlay` did:

```text
by_source.insert(new_source_id, module)
by_module.insert(module, new_overlay)
```

If the same module previously used a different `SourceId`, the old:

```text
old_source_id -> module
```

entry remained.

A later `read(old_source_id)` could then find the module and return the module's **new** overlay text.

That is semantically wrong.

Replacing/removing an overlay must update both indexes atomically.

Add a regression test.

---

## 10.3 `ProjectUniverse` historically was not `Clone`

The old Step-7 architecture proposed:

```rust
Arc<ProjectUniverse>
Arc::make_mut(...)
```

so immutable snapshots can retain an old universe while the session changes project topology.

At the historical baseline:

```rust
ProjectUniverse
```

derived `Debug`, not `Clone`.

Verify current code.

Also inspect:

```rust
SyntheticProjectIdAllocator
```

before mechanically deriving `Clone`.

The allocator's identity/freshness semantics matter.

Do not make cloneability corrupt future synthetic ID allocation.

---

## 10.4 There may be multiple “generation” notions

Historical code had:

```text
FilesystemSourceProvider.generation: AtomicU64
ResolverGeneration(pub u64)
Semantic revision
LSP generation/document revision
```

These are not interchangeable.

Step 7 must define exactly what each means.

Likely desired separation:

```text
SemanticRevision
  changes on every formal semantic commit

ResolverGeneration
  changes only when project/module resolution topology/input identity
  requires invalidating provider/resolver caches

Document revision
  changes per source buffer/update

LSP worker generation
  scheduling/staleness concept, not module identity
```

Do not collapse them into one counter merely for convenience.

But also do not leave two different resolver-generation counters that are supposed to mean the same thing.

---

## 10.5 Traced import resolution records a DB dependency on a product that may not be the exact value consumed

This deserves careful architecture review.

Historical `query_resolved_imports` did:

```text
resolver.resolve_import_with_trace(...)
for package_interface in trace.package_interfaces:
    record DB dependency on UnlinkedInterface(package_interface)
```

But historical `ModuleResolver::validate_external_path_with_trace` obtained package exposure by calling its own:

```text
load_package_surface
  -> load_interface
  -> load_parsed
  -> SourceProvider
  -> InterfaceBuilder
```

So the resolver may have computed exposure against a private resolver-built `UnlinkedModuleInterface`, then the semantic DB records an edge to a separately produced DB `UnlinkedInterface` with the same identity.

That is better than no dependency, but it is not literally:

```text
the semantic product read == the product represented by the dependency edge
```

It also duplicates parsing/interface construction.

Investigate whether current Step-7 design should:

1. provide resolver exposure/package-surface reads from canonical already-produced interfaces;
2. inject a small module-interface/package-surface provider trait into `phalcom-modules`;
3. preseed resolver caches from DB products;
4. or retain the current trace approach only if you can prove equivalence and lifecycle coherence.

Do **not** move exposure rules into `phalcom-semantic`.

The algorithm remains owned by `phalcom-modules`.

The aim is one semantic value, one dependency identity.

---

## 10.6 `LinkedInterface` projection dependencies may be missing/coarse

Historical `query_semantic_component`:

```text
depends on all UnlinkedInterface + ResolvedImports in the component
runs ModuleLinker
publishes SemanticComponent
then publishes LinkedInterface(M) projections
```

The historical linked-interface projection publication used no dependency edges.

That means a body may depend on:

```text
LinkedInterface(M)
```

while that projection itself may not expose its dependency on the semantic component or underlying module products.

Separately, the session historically called `query_linked_interface(...)` against an externally supplied linked interface.

This is transitional ownership.

Step 7 must audit the exact current DAG.

Possible correct shape:

```text
SemanticComponent(entry)
  -> all required UnlinkedInterface / ResolvedImports

LinkedInterface(M)
  -> SemanticComponent(entry)
```

or a more precise equivalent if the linker/query model supports it.

Do not introduce a cycle.

Do not leave `LinkedInterface` as an independently trusted externally supplied value on the normal incremental path.

---

## 10.7 Snapshot module maps were historically rebuilt ad hoc instead of projected from DB products

Historical session snapshot publication did all of the following:

### Unlinked map

Re-ran:

```rust
InterfaceBuilder::build(...)
```

over source units.

This duplicated the DB-owned `UnlinkedInterface` query.

### Linked map

Copied:

```text
input.linked.modules[*].interface
```

from the externally supplied linked program.

This bypassed compiler-owned semantic-component query ownership.

### Resolved import map

Reconstructed mappings from:

```text
linked_mod.bindings.imports
linked_mod.linked_reads
```

and binding names.

But the canonical `ResolvedImportsProduct` already stores:

```text
(importer, exact ImportPath::to_string()) -> target ModuleId
```

Those are not necessarily equivalent keys.

Step 7 should build immutable snapshot module products from current DB products, not re-run/reverse-engineer module semantics.

This is a major requirement.

---

## 10.8 `SemanticWorkspaceInput` is still a transitional cold/external shape

Historical:

```rust
SemanticWorkspaceInput {
    linked: Arc<LinkedProgram>,
    sources: ...,
    generation: ...
}
```

means an external caller can hand the session an already-linked program.

That is incompatible with the final normal incremental ownership model.

But many tests/compiler APIs may rely on it.

Do not simply delete it.

The old architecture intended:

> retain `update(SemanticWorkspaceInput)` only as a compatibility/cold wrapper implemented through the same session primitives.

You must investigate how feasible that is with current APIs.

Possible outcomes:

- compatibility wrapper seeds/feeds canonical session inputs;
- compatibility constructor is retained only for low-level tests;
- new primary API becomes source/workspace changes + `commit_revision`;
- old wrapper is clearly marked cold/compatibility and not used by production LSP.

Do not maintain two independent semantic algorithms.

---

## 10.9 `ResolvedDocumentIdentity` may not carry enough information for overlay creation

Historical type:

```rust
ResolvedDocumentIdentity {
    source: SourceId,
    module: ModuleId,
    generation: ResolverGeneration,
}
```

Historical `SourceOverlayUpdate` requires:

```text
ModuleId
ModuleKind
SourceLocation
revision
text
```

If `resolve_document_path()` returns only `ResolvedDocumentIdentity`, the session still needs:

```text
ModuleKind
display path / SourceLocation
```

to create/update the overlay.

Do not solve this by making the LSP reconstruct module identity or kind.

Investigate a coherent compiler API.

Possibilities include:

- returning a richer resolved source/document record;
- retaining `SourceUnit` internally while returning the public identity;
- adding an internal path-to-`SourceUnit` helper;
- extending the canonical identity type if that is truly its intended role.

Do not duplicate source-path interpretation outside `phalcom-modules`.

---

## 10.10 Physical path → logical module identity must be canonical and inverse-compatible

The old Step-7 design called for a canonical:

```rust
resolve_source_path(project, path) -> SourceUnit
```

in `phalcom-modules`.

Before specifying it, inspect the actual forward location implementation:

```text
FilesystemSourceProvider::locate
find_directory
find_final_candidates
ModuleComponent constructors
kebab/snake conversion rules
package.ph handling
nested project boundaries
source-root confinement
ambiguity behavior
```

The inverse must use the same canonical rules.

Do not invent a simplistic:

```text
strip .ph
split path
replace '-' with '_'
```

implementation.

Add round-trip tests:

```text
logical -> locate -> physical -> reverse resolve -> same ModuleId/kind
```

for:

- root `package.ph`;
- nested package;
- ordinary module;
- kebab/snake canonical spelling;
- nested project boundary rejection;
- outside-source-root rejection;
- ambiguous or noncanonical paths.

---

## 10.11 Unsaved/new editor files may challenge path canonicalization

The old proposed reverse-path algorithm began with:

```text
canonicalize(path)
```

That normally requires the file to exist.

An LSP may receive an open document for a new file that has not yet been written.

Do not automatically expand Step 7 to solve every untitled-buffer case, but explicitly investigate current LSP behavior and intended support.

If Step 7 intentionally supports only file-backed project documents at this layer, say so and defer untitled/new-file behavior explicitly.

Do not accidentally make it impossible to support later by baking `canonicalize(existing file)` into the only identity API without discussion.

---

## 10.12 Standalone package/module semantics must not be forgotten

`ProjectUniverse` already supports persistent projects and synthetic roots.

`source.rs` already had `EntryOwnership` variants such as:

```text
ProjectOwned
StandalonePackageOwned
StandaloneModule
Inline
```

Fresh archaeology must find the actual current entry classification path.

Step 7 must decide what session document/path APIs support:

- persistent project documents;
- standalone package trees;
- standalone `.ph` modules;
- inline/synthetic sources;
- builtin virtual modules.

Do not silently reduce the language to manifest-backed projects.

But do not invent a broad new entry model if existing module APIs already define it.

---

## 10.13 Project-universe removal/change semantics need explicit design

`ProjectUniverse::load_root` historically supported loading/deduplication but not obvious project removal.

If workspace roots/manifests change, simply mutating an old universe may retain stale projects.

This is where:

```text
Arc<ProjectUniverse>
```

plus a new universe generation may be preferable:

```text
ordinary source edit:
  keep same universe Arc

workspace root/manifest/dependency topology change:
  construct/derive new universe
  bump ResolverGeneration
  old snapshot retains old Arc
```

Do not force in-place mutation if the existing type is not designed for removal.

Specify exact replacement/reuse law.

---

## 10.14 Filesystem provider cache clearing must not happen on ordinary source edits

Historical `FilesystemSourceProvider::clear_cache()`:

```text
increments generation
clears locate cache
clears source-text cache
clears source-id reverse map
```

Ordinary editor text updates should normally hit the overlay and invalidate semantic source queries, not blow away project/source-resolution caches.

Define precisely what events call `clear_cache`.

Likely candidates:

- workspace root change;
- manifest/dependency topology change;
- filesystem topology event that affects logical module location.

Not:

- method body edit in an existing open file.

---

## 10.15 Snapshot universe sharing matters

If a published snapshot contains:

```rust
Arc<ProjectUniverse>
```

and the next resolver generation changes the universe, the old snapshot must remain self-consistent.

Do not mutate a universe behind old snapshot references.

Use immutable replacement/copy-on-write semantics or another design with equivalent snapshot safety.

---

# 11. Required Step-7 target behaviors to validate

The final specification should derive exact tests from current APIs, but at minimum cover these laws.

## 11.1 Ordinary body edit

Expected:

```text
same ProjectUniverse identity/Arc
same ResolverGeneration
no filesystem provider cache reset
overlay text changes
ParsedModule invalidated/recomputed
normal semantic query propagation applies
```

---

## 11.2 Open-buffer overlay beats disk

Given disk text A and overlay text B:

```text
canonical semantic analysis reads B
filesystem stays A
```

After overlay removal:

```text
canonical provider reads A
```

---

## 11.3 Overlay replacement cleans reverse index

Module overlay moves/rebinds from SourceId A to SourceId B.

Expected:

```text
read(A) does not return B's text
read(B) returns B's text
```

---

## 11.4 Concurrent overlay read/write cannot deadlock

Design testable lock semantics.

If deterministic concurrency testing is difficult, at minimum structure the provider so deadlock-by-lock-order is impossible by construction, preferably one state lock.

---

## 11.5 Exact import resolution is retained in snapshot products

For each successful source import:

```text
ResolvedImportsProduct exact path string -> target ModuleId
```

must match:

```text
snapshot.module_queries().resolved_import_target(...)
```

Do not reconstruct keys from local binding names.

---

## 11.6 Package `expose` edit invalidates external import resolution

If dependency package exposure changes:

```text
package UnlinkedInterface product changes
ResolvedImports(importer) recomputes
import may become valid/invalid
```

without changing unrelated projects/modules.

This is the reason import-resolution trace dependencies exist.

---

## 11.7 Relative/self import does not depend on external exposure

Relative/self imports should preserve current canonical resolver semantics.

Do not apply external hierarchical exposure filtering to internal project paths.

---

## 11.8 Linked interface comes from compiler-owned linking

Normal incremental source/session flow must not require the caller to supply the authoritative `LinkedProgram`.

`SemanticComponent`/linker computation must be owned by the semantic workspace lifecycle.

---

## 11.9 Module products are snapshot reads, not recomputation

After publication:

```rust
snapshot.module_queries()
```

must perform:

```text
no filesystem access
no source-provider mutation
no ProjectUniverse loading
no resolver construction
no linker construction
no SemanticDb revision
no query computation
```

Step 7 prepares this data.

Later completion/navigation consumes it.

---

## 11.10 Workspace topology change replaces the resolution generation

Change roots/manifest/dependency topology.

Expected:

```text
ResolverGeneration changes
universe version changes
old snapshot retains old universe
new module-resolution products correspond to new universe
```

---

## 11.11 Source-only edit does not replace the resolution generation

This is a hard performance/incrementality gate.

---

## 11.12 Failed/cancelled semantic commit must not publish a mixed module snapshot

Do not combine:

```text
new overlays/universe
old linked products
new formal body products
```

into one snapshot.

Publication remains atomic at the semantic snapshot boundary.

Clarify whether mutable session input state may advance while the last published snapshot remains old; if so, the next commit must recompute consistently from current inputs.

---

# 12. Tests and instrumentation you should inspect before inventing new helpers

Before writing proposed tests, search existing suites for reusable fixtures.

At minimum inspect:

```text
phalcom-modules/tests/
phalcom-semantic/tests/
phalcom-lsp/tests/
```

Search specifically for tests around:

```text
source provider
overlay
module resolution
external exposure
relative imports
project dependency aliases
module query facade
semantic workspace session
incremental invalidation
module completion/navigation
resolver generations
stale/cancelled publication
```

Prefer extending established fixtures over creating a parallel mock module system.

---

# 13. Think carefully about `ProjectUniverse` persistence

The desired property is not:

> never construct a new `ProjectUniverse`.

It is:

> do not reconstruct project/module identity on ordinary source edits, and never let the LSP own an independent formal universe.

A sound design may legitimately create a **new immutable universe version** when workspace topology changes.

That is preferable to mutating an old universe retained by snapshots.

The Step-7 spec must distinguish:

```text
source semantic revision
vs
project/module-resolution generation
```

and explain exactly when each advances.

---

# 14. Think carefully about resolver caches

Historical `ModuleResolver` has private:

```text
parsed_cache
interface_cache
```

After Steps 1–6, the semantic DB is supposed to own parsed/interface semantic products.

Do not blindly make `ModuleResolver` itself long-lived just to preserve those caches.

That can create:

```text
DB parsed/interface products
+
resolver parsed/interface cache
```

as duplicate work/authority.

The better Step-7 design may reduce resolver private cache importance by feeding canonical products/package surfaces, or instantiate it per query generation.

Investigate before deciding.

---

# 15. Think carefully about source ownership and physical files

`OverlaySourceProvider<P>` should be a **source text/location layer**, not a module-resolution algorithm.

It should answer:

```text
locate canonical ModulePath under known project
read canonical SourceId
```

while respecting overlays.

It should not decide:

```text
what an import root means
whether a package is exposed
how project dependencies resolve
```

Keep those in `ProjectUniverse` / `ModuleResolver`.

---

# 16. Think carefully about exact query DAG after Step 7

Do not assume the final graph is a linear pipeline.

Likely module side:

```text
ParsedModule(M)
    |
    v
UnlinkedInterface(M)
    |
    +-------------------------------+
    |                               |
    v                               |
ResolvedImports(M)                  |
    |  \                            |
    |   \ external package exposes  |
    |    -> UnlinkedInterface(Pkg)  |
    |                               |
    +---------------+---------------+
                    |
                    v
SemanticComponent(entry)
    depends on module interfaces/resolutions
                    |
                    v
LinkedInterface(M) projection
                    |
                    v
DeclarationShell / HierarchyEdge / DeclarationSurface / ...
```

But verify actual Step-6/current query implementation.

You may need different component granularity or projection edges.

The spec must explicitly state every module query key, direct input, semantic dependencies and product identity.

---

# 17. Step-7 non-goals

Unless repository reality makes a tiny prerequisite unavoidable, keep these out of Step 7:

## Step 8

Do not yet perform the full deletion/migration of:

```text
refresh_static_workspace_analysis
run_static_workspace_analysis
StaticWorkspaceIdentity formal ownership
LSP ProjectUniverse/ModuleResolver/ModuleLinker reconstruction
```

Step 7 creates the compiler-owned lifecycle APIs Step 8 will call.

You may make minimal compile-preserving API adaptations only.

---

## Later module editor work

Do not make Step 7 own the full final solution for:

- import completion UX;
- canonical unresolved-import diagnostics UX;
- goto-definition module origin behavior;
- module occurrence identity cleanup;
- formal-site navigation;
- virtual-source UI.

Step 7 must publish the canonical data those features need.

---

## No new module semantics

Do not change Phalcom's accepted module/exposure/import language semantics unless you discover a repository/spec contradiction that must be resolved first.

If so, flag it explicitly rather than burying a language-design change inside lifecycle refactoring.

---

# 18. Required archaeology questions the final spec must answer

Your investigation is incomplete until you can answer all of these with file/symbol evidence.

1. Who currently constructs `ProjectUniverse` in semantic/compiler/LSP paths?
2. How many independent universes can exist during one LSP analysis refresh?
3. Who currently owns `FilesystemSourceProvider`?
4. Is `OverlaySourceProvider` used in production anywhere?
5. Where do overlay updates originate?
6. What exactly increments `FilesystemSourceProvider::generation`?
7. Where is `ResolverGeneration` used, and is it currently coupled to provider generation?
8. How are persistent projects vs standalone packages/modules discovered today?
9. Is there already a canonical physical-path-to-ModuleId function?
10. What exact forward filename spelling law does `FilesystemSourceProvider::locate` enforce?
11. How are nested project boundaries enforced?
12. How does `ModuleResolver` obtain package exposure surfaces?
13. Can those surfaces come from current DB products without moving resolver semantics out of `phalcom-modules`?
14. Who currently calls `query_resolved_imports`?
15. Who currently calls `query_semantic_component`?
16. Does production `SemanticWorkspaceSession` use them, or does it still receive an external `LinkedProgram`?
17. What are the exact dependencies of current `LinkedInterface` products?
18. How are `ModuleQueryProducts.unlinked` currently populated?
19. How is `ModuleQueryProducts.linked` currently populated?
20. How is `ModuleQueryProducts.resolved_imports` currently populated?
21. Is the exact import syntax string preserved end-to-end?
22. Can stale old module products appear in a new snapshot after deletion/removal?
23. Does Step 6's `current_products()` solve that for module products too?
24. What compatibility callers depend on `SemanticWorkspaceInput`?
25. Can `update(SemanticWorkspaceInput)` be implemented through the new source/session lifecycle without a second algorithm?
26. What does the LSP need from Step-7 APIs so Step 8 can be a deletion/migration rather than another architecture redesign?
27. What compiler/core callers need compatibility?
28. What tests already encode resolver/project/module identity behavior?
29. What source edit currently causes project/module reconstruction?
30. What exact operation should distinguish source edit from topology edit after Step 7?

The final Step-7 specification should make these answers obvious.

---

# 19. Repository evidence standard

For every major current-state claim in the final specification, cite repository evidence in prose:

```text
file path
type/function/module
what the implementation currently does
why it is insufficient or reusable
```

Example quality:

> `phalcom-modules/src/resolver.rs::resolve_import_with_trace` already owns external exposure validation and returns `package_interfaces`; Step 7 must not reimplement exposure traversal in `phalcom-semantic`. The open question is how the resolver obtains the package surface so the semantic dependency edge corresponds to the exact product consumed.

Bad quality:

> “The resolver handles imports.”

Use exact names.

---

# 20. Specification style

Write dense technical prose with code where the exact API matters.

For each proposed API:

- explain why it belongs in that crate;
- name its owner;
- define inputs/outputs;
- define mutation/lifetime behavior;
- define error behavior;
- define revision/generation behavior;
- identify callers;
- identify tests.

Do not invent a large abstraction hierarchy merely for conceptual neatness.

Prefer the smallest architecture that establishes one authority.

---

# 21. TDD granularity

The Step-7 implementation spec should be executable as several medium-sized reviewable commits.

Likely slices—revise after archaeology:

```text
7.1  repair/complete overlay source provider correctness
7.2  canonical physical source path -> module identity
7.3  project universe / resolver-generation lifecycle in session
7.4  document/source change APIs
7.5  compiler-owned ResolvedImports + SemanticComponent scheduling
7.6  DB-derived ModuleQueryProducts snapshot projection
7.7  compatibility cold wrapper + structural/performance regressions
7.8  deletion/static audit and full gates
```

Do not preserve these boundaries if current code suggests better ones.

Each slice must have:

```text
red test
expected failure
minimal implementation
focused test
broader regression
commit boundary
```

---

# 22. Static audit gates the final spec should probably include

After Step 7, likely expected production checks include variants of:

```bash
rg "InterfaceBuilder::build" phalcom-semantic/src/session.rs
```

Expected: no snapshot ad-hoc rebuild of DB-owned unlinked interfaces.

```bash
rg "linked_mod\\.bindings|linked_reads" phalcom-semantic/src/session.rs
```

Expected: no reverse engineering of exact resolved-import map for snapshot publication.

```bash
rg "ProjectUniverse::new" phalcom-semantic/src/session.rs
```

Interpret carefully:

- session constructor may legitimately initialize one;
- ordinary commit/update path must not create one per edit.

```bash
rg "clear_cache" phalcom-semantic phalcom-lsp
```

Expected: cache generation changes only from topology-relevant events.

```bash
rg "ModuleResolver::new|ModuleLinker::new" phalcom-lsp/src
```

These are primarily a Step-8 deletion gate, so Step 7 may still show matches.

Do not incorrectly require zero LSP matches until Step 8.

Also audit:

```bash
rg "generic_super" phalcom-semantic
```

Expected after Step 6: no fake direct superclass.

And:

```bash
rg "SemanticWorkspaceInput" .
```

to enumerate all compatibility callers before changing it.

---

# 23. Performance expectations

Step 7 is not complete merely because behavior is correct.

It must preserve the incremental architecture.

The final specification should include tests/metrics proving:

## Source body edit

```text
ProjectUniverse not rebuilt
ResolverGeneration unchanged
FilesystemSourceProvider caches not globally cleared
unrelated module products reused
formal Step-6 projections reused where semantic products stable
```

## Import edit

```text
affected UnlinkedInterface/ResolvedImports/SemanticComponent closure changes
unrelated projects/modules retain stable products
```

## Package exposure edit

```text
only importers whose traced exposure path consumed that package surface recompute
```

at least to the precision supported by the current query design.

## Workspace topology edit

```text
new resolver generation
new/replaced universe version
old snapshot remains valid
```

---

# 24. Error and publication semantics

Do not conflate:

```text
language/module semantic error
```

with:

```text
infrastructure failure
```

The session should be able to publish a snapshot containing ordinary semantic/module diagnostics.

Cancellation, stale generation, budget exhaustion or internal consistency failure should not replace the last successfully published snapshot with a mixed/partial one.

Step 7 may not yet own final user-facing module diagnostic aggregation, but its lifecycle must retain enough structured error information for that later step.

Do not stringify canonical module errors too early unless the existing DB product deliberately owns rendered diagnostics.

---

# 25. Fresh-session reasoning discipline

Do not use prior assistant claims as repository facts.

The previous planning process discovered several stale assumptions only after deeper inspection.

Specifically:

- Step-7 primitives existed earlier than expected.
- The Step-5.5 commit was initially invisible, then appeared later.
- Snapshot module products existed but were populated through transitional duplicate logic.
- Resolver tracing existed, but exact product-consumption semantics remained questionable.
- `ResolvedDocumentIdentity` existed independently of the proposed Step-7 session API.
- module generations already had more than one representation.
- the Step-6 hierarchy review found a real semantic bug (`generic_super`) hidden inside a helper.

Expect similar surprises.

Search implementation before designing.

---

# 26. What I most wish I had known earlier

Carry these lessons directly into Step-7 planning.

## 26.1 “Type exists” is not “architecture complete”

A struct such as:

```text
ModuleQueryProducts
WorkspaceRootInput
SourceOverlay
ResolvedDocumentIdentity
```

can already exist while the ownership flow is still wrong.

Always trace who constructs it, who mutates it, who publishes it and whether production uses it.

---

## 26.2 “DB query exists” is not “DB owns the truth”

Earlier steps had DB `DeclarationSurface` and `HierarchyEdge` products while the session still rebuilt separate authoritative tables.

For Step 7, watch for the same pattern:

```text
ResolvedImports query exists
but snapshot reconstructs imports separately

SemanticComponent query exists
but session receives external LinkedProgram

UnlinkedInterface query exists
but snapshot reruns InterfaceBuilder
```

The goal is ownership, not API count.

---

## 26.3 Dependency tracing must correspond to what was actually read

A trace that tells you “package X was consulted” is useful.

But if the resolver consulted its own separately rebuilt interface while the DB edge names a different DB product, investigate the equivalence boundary.

The checker read audit taught us this lesson already.

Apply it to modules.

---

## 26.4 Rust ownership constraints matter to architecture

Do not design self-referential session state because it looks conceptually tidy.

A resolver borrowing universe/provider should probably remain an ephemeral computation unless its ownership model is deliberately redesigned.

Prefer architecture that is idiomatic in Rust, not a translation of an OO object graph.

---

## 26.5 Structural sharing should follow semantic lifetime boundaries

After Step 6, formal projections reuse whole Arcs when product stamps are unchanged.

Step 7 should use the same philosophy for:

```text
ProjectUniverse
module product maps
source provenance maps
```

Do not rebuild immutable views on every edit if their semantic inputs did not change.

But do not overengineer persistent data structures prematurely.

Whole-component Arc reuse is often enough.

---

## 26.6 Exact canonical identity beats heuristic reconstruction

Do not derive:

```text
ModuleId
resolved import target
package identity
```

from URI strings, binding names or path guessing when `phalcom-modules` already owns canonical identities.

This is foundational for Step 8+ LSP correctness.

---

# 27. Final output requirements

When archaeology is complete, write the Step-7 specification.

The document should clearly separate:

```text
A. verified current state
B. architecture already implemented and retained
C. remaining defects
D. target Step-7 design
E. exact implementation plan
F. tests/verification
G. explicit Step-8+ boundary
```

For every proposed deletion, show the replacement owner first.

For every proposed new abstraction, show why existing types cannot carry the responsibility.

For every generation counter, define its semantic meaning.

For every mutable session field, define when it changes.

For every snapshot field, define where its canonical data comes from.

For every module query product, define:

```text
direct input identity
semantic dependencies
product identity
failure behavior
snapshot projection
```

No shallow output.

No “add tests” placeholders.

No invented APIs without repository justification.

No module logic duplicated in semantic/LSP layers.

No Step-8 work smuggled into Step 7.

---

# 28. Suggested final Step-7 document title

Use:

```markdown
# Phalcom Incremental Semantics — Step 7: Compiler-Owned Project, Source, and Module Lifecycle
```

Suggested one-sentence goal:

> Move project-universe, source-overlay, canonical document identity, import-resolution and linking lifecycle under `SemanticWorkspaceSession`, so ordinary source edits update one persistent compiler-owned semantic workspace and published snapshots expose module products derived directly from the semantic DB rather than from externally rebuilt module state.

Revise that wording if repository archaeology proves a more precise boundary.

---

# 29. Before you finalize the spec

Run a self-review against this checklist.

- Did you verify current HEAD after Step 6?
- Did you prove Step 6 postconditions from source/tests?
- Did you inspect all existing Step-7 substrate before proposing replacements?
- Did you resolve or explicitly flag the overlay locking/reverse-index defects?
- Did you distinguish semantic revision, resolver generation and document revision?
- Did you avoid a self-referential persistent `ModuleResolver` design?
- Did you trace how package exposure surfaces are actually consumed?
- Did you audit `LinkedInterface` dependencies?
- Did you eliminate ad-hoc snapshot reconstruction from source/linker internals in the proposed target?
- Did you preserve exact import-path resolution keys?
- Did you define physical-path ↔ logical-module behavior from actual forward resolver rules?
- Did you cover project, standalone package/module and builtin identity boundaries appropriately?
- Did you preserve old snapshot consistency across topology changes?
- Did you define how the cold `SemanticWorkspaceInput` compatibility path converges on one algorithm?
- Did you keep LSP lifecycle deletion for Step 8?
- Did you provide exact tests and exact commands?
- Did you identify every file/symbol with repository evidence?

If any answer is “no,” continue archaeology before delivering.
