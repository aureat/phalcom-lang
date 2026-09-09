---
id: LANG004.C2.P1
category: LANG
program: LANG004
checkpoint: LANG004.C2
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on: [LANG004.C1.P1]
follows: LANG004.C1.P1
supersedes: null
deferred_reason: Awaiting the associated-reference migration in LANG004.C1.
---

# LANG004.C2.P1 — logical paths and module-associated lookup

## Phalcom Logical `::` Paths and Module/Package Associated Lookup — Patch-Grade Implementation Specification

**Repository:** `aureat/phalcom-lang`  
**Prepared against remote branch:** `main`  
**Baseline HEAD:** `288da3f5da322dba60d0be9dfa4b52f5f1505d2f` (`chore(repo): fix formatting drift`, 2026-09-09)  
**Repository visibility:** GitHub remote state was inspected. Local uncommitted working-tree state was not observable. Before implementation, compare the local checkout to this revision and follow the repository-drift procedure below if primary symbols or ownership boundaries have changed.

**Prerequisite:** the companion `&` callable-reference / associated-lookup migration must have established that `::` means associated lookup only and that ordinary bound method references are introduced with `&` over the normal `.` message surface.

---

# 0. Purpose and scope

This specification implements two ordered changes.

First, migrate logical source paths used by module declarations from dotted component traversal to `::` component traversal while preserving leading relative-dot anchors:

```phalcom
from app::support::graph::algorithms import (DFS, BFS)
from .errors::result import Result
from ..concurrency import Future
```

Second, make the same associated namespace relationship available as expression syntax outside import/export declarations:

```phalcom
universe::concurrency::fibers::Fiber.new {
    Fiber.yield(42)
}
```

The intended semantic boundary is:

```text
.     ordinary message/member surface of a runtime object
::    associated namespace lookup
```

For a runtime module/package descriptor:

```phalcom
const concurrency = universe::concurrency

concurrency.name       // ordinary reflective message/member surface
concurrency::fibers    // associated namespace lookup
```

The implementation must preserve the repository's current module architecture: `phalcom-modules` remains the sole authority that interprets logical paths and canonical module identities; `phalcom-semantic` consumes canonical module products; the LSP schedules work and consumes compiler-owned semantic/module snapshots rather than implementing an independent resolver.

## 0.1 Ratified logical path surface

Absolute logical path:

```phalcom
app::support::graph::algorithms
```

Relative logical paths:

```phalcom
.errors::result
..concurrency
..support::logging
```

The leading dot run is a relative-path anchor. It is not member access and is not an expression-level `.` chain.

Path components after the anchor are separated exclusively by `::`.

## 0.2 Statement forms in scope

Whole-module import:

```phalcom
import app::support::graph
import .errors::result
```

Selective import:

```phalcom
from app::support::graph::algorithms import (DFS, BFS)
from .errors::result import Result
```

Direct re-export:

```phalcom
export DFS, BFS from app::support::graph::algorithms
```

Local body export remains a local-binding export and has no logical path to migrate:

```phalcom
export DFS
```

`expose` remains the existing direct-child package declaration:

```phalcom
expose .concurrency
```

The current AST/interface model stores exposed children as one `ModuleComponent` per declaration. This program does **not** introduce nested `expose .a::b`. Exposure across multiple package boundaries remains represented by an `expose .child` declaration at each relevant package boundary.

## 0.3 Expression-level associated paths

The following is valid expression syntax after this program:

```phalcom
universe::concurrency::fibers::Fiber
```

Every intermediate result is a normal value:

```phalcom
universe
universe::concurrency
universe::concurrency::fibers
universe::concurrency::fibers::Fiber
```

A typical chain may therefore move through different runtime values while retaining one surface operator:

```text
universe                            Package descriptor
universe::concurrency               Package descriptor
universe::concurrency::fibers       Module descriptor
universe::concurrency::fibers::Fiber class object
```

The concrete runtime descriptor class already exists: `Package` inherits `Module` in the Universe source surface. Do not introduce a parallel package-descriptor hierarchy.

## 0.4 Class-side methods remain message sends

Expression-level module traversal stops when the desired runtime object has been obtained. Subsequent class-side calls remain ordinary message sends:

```phalcom
universe::concurrency::fibers::Fiber.new {
    Fiber.yield(42)
}
```

Do not rewrite this as `Fiber::new` or `Fiber::yield`.

## 0.5 Static and dynamic associated lookup

A source-known chain may be resolved and lowered directly:

```phalcom
universe::concurrency::fibers::Fiber
```

A descriptor stored in a value remains capable of runtime associated lookup:

```phalcom
const concurrency = universe::concurrency
const fibers = concurrency::fibers
```

The static optimization and runtime operation must obey the same export/exposure rules and return equivalent values.

This program does not add general flow-sensitive constant propagation merely to recover descriptor identity through arbitrary expressions. Static dependency extraction is required for syntactically direct canonical paths and for receiver facts that the existing compiler analysis already proves to denote a canonical module/package descriptor. Other descriptor-valued expressions may use runtime lookup without acquiring invented static target identity.

## 0.6 Core non-goals

Do not:

- redesign package discovery, `ModuleId`, `ModulePath`, or project identity;
- create an LSP-specific logical-path resolver;
- create a second module graph for expression paths;
- turn `::` into ordinary `Module.doesNotUnderstand` / message dispatch;
- expose an unexposed child merely because the topology knows it exists;
- expose a private/unexported module binding through runtime `::`;
- make class-side methods associated functions;
- add nested `expose` paths;
- add new flow-sensitive analysis just to optimize aliased descriptor values;
- preserve old dotted logical paths as a second permanent source syntax unless a temporary diagnostic-only migration branch is explicitly required.

---

# 1. Repository state and current architecture

## 1.1 Repository revision

Planning was performed against remote `main` at:

```text
288da3f5da322dba60d0be9dfa4b52f5f1505d2f
```

Recent commits in the requested area show that the module/incremental architecture is active and should be treated as current rather than replaced. Relevant recent work includes stable import-site resolution, indexed topology, transactional module-session updates, affected-component linking, canonical cross-module targets, exact module-fact freshness, and LSP publication of canonical module diagnostics.

The implementation agent must run a small drift check before each checkpoint. Do not assume an older implementation document is more authoritative than current production symbols.

## 1.2 AST and path representation

Primary file:

- `phalcom-ast/src/ast.rs`
  - `ImportPath`
  - `ImportRoot`
  - `PathSegment`
  - `DependencyDecl`
  - `ImportDecl`
  - `ModuleImportDecl`
  - `SelectiveImportDecl`
  - `ReExportDecl`
  - `ExportDecl`
  - `ExposeDecl`

Current structural path representation is already suitable:

```text
ImportPath
  root:
    Absolute(first segment)
    or Relative { dots }
  segments: remaining path components
  range
```

The important observation is that punctuation is not the path's semantic identity. The AST already separates root mode, relative depth, components, and ranges. The syntax migration should therefore change parsing/rendering, not invent a new path-identity model.

`ExposeDecl` is intentionally different: it owns one direct child `PathSegment`, and `UnlinkedModuleInterface.exposed_children` stores a `BTreeSet<ModuleComponent>`.

## 1.3 Parser ownership

Primary file:

- `phalcom-ast/src/parser.rs`
  - `parse_import_path`
  - whole-module import parsing
  - selective import parsing
  - `parse_reexport_decl`
  - `parse_expose_decl`

`parse_import_path` currently documents/accepts dotted component examples such as `geometry.point`, `.point`, and `..units`. It must be migrated so that:

```text
absolute root/component boundary: ::
subsequent component boundaries: ::
leading relative anchor: . / .. / ... unchanged
```

Expression-level `::` parsing is owned by the associated-lookup grammar introduced/cleaned up by the companion implementation. Import-path parsing remains a dedicated declaration grammar even though it uses the same visual separator.

## 1.4 Module interface ownership

Primary file:

- `phalcom-modules/src/interface.rs`
  - `ImportSurface`
  - `UnlinkedExportTarget`
  - `LinkedExportTarget`
  - `UnlinkedModuleInterface`
  - `LinkedModuleInterface`
  - `InterfaceBuilder::build`

Current important products:

```text
UnlinkedExportTarget::ReExport { path: ImportPath, remote }
LinkedExportTarget::Binding(SymbolId)
LinkedExportTarget::Module(ModuleId)
UnlinkedModuleInterface.exposed_children: BTreeSet<ModuleComponent>
```

`InterfaceBuilder` already owns namespace collision checks, import surface extraction, direct re-export surface construction, local export validation, and `expose` validation. Keep those responsibilities here.

## 1.5 Canonical path resolution ownership

Primary file:

- `phalcom-modules/src/resolver.rs`
  - `ModuleResolver`
  - `resolve_import`
  - `resolve_import_product`
  - `resolve_import_product_for_site`
  - `resolve_import_with_trace`
  - `ImportResolutionProduct`
  - `ResolvedImportPrefix`

The resolver is the authority for converting an authored `ImportPath` plus importing `ModuleId` into canonical project/module identity while enforcing root, relative, package, and exposure rules.

The current product retains both canonical identity and presentation/provenance:

```text
ImportResolutionProduct
  site
  written_path
  prefixes
  target
  dependencies
  fingerprint
```

`written_path` and `ResolvedImportPrefix.prefix` currently reflect dotted source spelling. These are expected to change to `::` rendering. Canonical `ModuleId`/`ModulePath` identity must not change merely because source punctuation changed.

## 1.6 Fingerprint ownership

Primary file:

- `phalcom-modules/src/fingerprint.rs`
  - `hash_import_path`
  - `hash_unlinked_interface`
  - `interface_fingerprint`
  - `unlinked_interface_input_fingerprint`
  - `linked_interface_fingerprint`

`hash_import_path` already hashes:

- absolute vs relative root;
- absolute root component or relative dot depth;
- ordered segment names;
- ranges only for the source/provenance-sensitive form.

It does **not** hash the literal `.` punctuation between components. Preserve that property. Two structurally identical logical paths must remain the same semantic path even though the canonical source spelling changes from `a.b.c` to `a::b::c`.

## 1.7 Persistent module workspace ownership

Primary file:

- `phalcom-modules/src/session.rs`
  - `WorkspaceModuleSession`
  - `WorkspaceModuleUpdate`
  - `WorkspaceModuleStats`
  - `WorkspaceSourceState`
  - transactional/staged source and product maps

The persistent session publishes canonical module products including:

```text
linked
sources
interfaces
import_products: Arc<BTreeMap<ImportSiteId, Arc<ImportResolutionProduct>>>
topology
reverse_importers
sites_by_importer
reverse_site_importers
module_graph_changed
```

This is the module-side incremental source of truth. Expression-level static module paths must integrate with this architecture rather than being resolved later in the LSP or reconstructed from strings.

## 1.8 Module topology/query ownership

Primary files:

- `phalcom-modules/src/topology.rs`
  - `ModuleTopology`
  - `module_children`
- `phalcom-modules/src/query.rs`
  - `ModuleQueryFacade`
  - `module_children`
  - `import_children`
  - `external_import_children`
  - `public_exports`
  - `resolve_relative_prefix`
  - `reverse_importers`

Topology is the canonical source for known module-child relationships. Importability/exposure filtering is a separate policy from raw topology enumeration. Preserve this split, especially for `expose` completion: an unexposed direct child must still be discoverable as something the package may expose.

## 1.9 Semantic workspace ownership

Primary files:

- `phalcom-semantic/src/workspace.rs`
  - `SemanticWorkspaceInput`
  - `analyze_workspace`
- `phalcom-semantic/src/session.rs`
  - `SemanticWorkspaceSession`
  - `SemanticWorkspacePublication`
  - `SemanticUpdateStats`
- `phalcom-semantic/src/snapshot.rs`
  - immutable published module/semantic products
- `phalcom-semantic/src/db/query.rs`
  - dependency/freshness query evaluation

Production semantic publication consumes exact canonical module products:

```text
import_products
module_graph_changed
import_sites_by_module
topology
reverse_imports
```

Compatibility-only source-index inputs may still contain path-string fallbacks, but production must not be changed to rely on them.

## 1.10 Compiler-owned source/editor identity

Primary files:

- `phalcom-semantic/src/identity.rs`
  - `SemanticTargetId::Module(ModuleId)`
  - `SemanticTargetId::ModuleBinding(SymbolId)`
- `phalcom-semantic/src/source_index/occurrence.rs`
  - import dependency occurrences
  - `path_occurrences`
  - `resolve_import_path`
  - expression visitor
- `phalcom-semantic/src/source_index/builder.rs`
- `phalcom-semantic/src/editor.rs`

The source index already records each import-path prefix against canonical module targets when `ImportResolutionProduct.prefixes` is available. Extend that identity-based approach to expression-level associated module paths. Do not infer editor targets from rendered path text.

## 1.11 LSP ownership

Primary files:

- `phalcom-lsp/src/analysis_service.rs`
  - `CompilerWorkspaceState`
  - worker scheduling/publication
- `phalcom-lsp/src/import_completion.rs`
  - `ImportContext`
  - `detect_import_context`
  - `import_completions`
- request/definition/completion consumers in `phalcom-lsp/src/backend.rs` and related modules

Current `CompilerWorkspaceState` contains one `phalcom_semantic::SemanticWorkspaceSession`. The LSP is a scheduler/adapter around compiler-owned state. This ownership is non-negotiable for the migration.

`detect_import_context` is currently punctuation-sensitive and splits paths on `.`. It must move to the new syntax, while completion resolution continues to use `snapshot.module_queries()`.

## 1.12 REPL completion ownership

Primary file:

- `phalcom-repl/src/completer.rs`
  - import-path context detection
  - `suggest_import_paths`

The REPL must emit/recognize the same canonical source spelling as the parser and LSP.

## 1.13 Runtime module/package descriptors

Primary files:

- `phalcom-core/src/heap/module.rs`
  - `ModuleObject`
  - `RuntimeExportRef`
- `phalcom-core/src/modules/registry.rs`
  - runtime materialized-module identity records
- `phalcom-core/src/modules/materialize.rs`
  - `LinkedExportTarget` to runtime export materialization
- `phalcom-core/src/modules/builtin_materialize.rs`
  - canonical Universe package/module materialization
- `phalcom-core/src/vm/send.rs`
  - current module-export *message send* path
- `phalcom-core/core/universe/src/reflection/module.ph`
- `phalcom-core/core/universe/src/reflection/package-object.ph`

`ModuleObject` already contains:

```text
id: ModuleId
kind: ModuleKind
package/root_package handles
exports: HashMap<Symbol, RuntimeExportRef>
```

`RuntimeExportRef` already distinguishes:

```text
Binding(BindingRef)
Module(ObjRef)
```

The Universe reflection surface already defines `Package is Module`. Preserve this runtime object model.

The VM's existing `try_module_export_send` path belongs to ordinary message dispatch and must not become the implementation definition of `::`. `module.name` and `module::name` must remain distinguishable.

---

# 2. Authoritative sources of truth

| Concern | Source of truth | Derived consumers | Forbidden competing authority |
|---|---|---|---|
| Authored logical path structure | `phalcom_ast::ast::ImportPath` | interface extraction, resolver, source index | raw source-string splitting in semantic/LSP |
| Canonical module identity | `phalcom_modules::ModuleId` / `ModulePath` | linker, semantic targets, runtime registry | rendered logical path string |
| Import resolution | `phalcom_modules::ModuleResolver` + `ImportResolutionProduct` | workspace session, semantic session, source index, LSP | LSP-only resolver |
| Package/module child topology | `ModuleTopology` / `ModuleQueryFacade` | completion, static path resolution support | filesystem probing in editor code |
| Public module namespace | `LinkedModuleInterface.exports` and package exposure products | semantic associated lookup, runtime materialization | runtime raw globals table alone |
| Export target identity | `LinkedExportTarget::{Binding, Module}` | semantic target projection, runtime `RuntimeExportRef` | public-name string heuristics |
| Incremental module state | `WorkspaceModuleSession` publications | semantic session | semantic full-rescan of paths |
| Semantic/editor target | `SemanticTargetId::{Module, ModuleBinding, Declaration, ...}` | definitions/references/LSP | textual `a::b` reconstruction |
| Runtime module identity | module registry + `ModuleObject.id` | runtime associated lookup | new descriptor map keyed only by names |
| Runtime public export | `ModuleObject.exports: RuntimeExportRef` populated from linked products | runtime associated lookup | `name_to_slot` bypassing export policy |

---

# 3. Semantic invariants

## I-01 — `::` is the logical/associated separator

Inside an absolute or relative logical path, ordered components are separated by `::`.

```phalcom
app::support::graph
.errors::result
..support::logging
```

No interior `.` remains a logical component separator.

## I-02 — relative dot depth is unchanged

The relative anchor remains a count of leading dots. The path:

```phalcom
..support::logging
```

must preserve the same relative depth and canonical target that old `..support.logging` represented.

## I-03 — punctuation does not redefine canonical identity

The migration from old source spelling to new source spelling must not create a second `ModuleId`, source identity, linked binding identity, or runtime descriptor for the same structural logical module.

## I-04 — only `phalcom-modules` resolves logical module paths

Parser, semantic analyzer, LSP, REPL, and runtime consumers may carry or query canonical products. They must not independently decide what `app::support::graph` means.

## I-05 — `expose` stays a direct-child declaration

```phalcom
expose .concurrency
```

continues to add exactly one `ModuleComponent` to the package exposure surface. No nested exposure syntax is introduced by this plan.

## I-06 — production semantic publication uses identity-keyed products

Production source indexing and semantic analysis must continue to consume `ImportResolutionProduct` by `ImportSiteId`. Updating `ImportPath::to_string()` is not a reason to restore path-string semantic authority.

## I-07 — LSP consumes compiler-owned module products

Completion, definition, references, and diagnostics must observe the same canonical module identities as compiler analysis. The LSP may parse the current line enough to determine completion context, but it may not resolve logical paths itself.

## I-08 — module/package descriptors are normal runtime values

A module or package obtained by `::` can be stored, passed, reflected upon, and used as a later associated-lookup receiver.

```phalcom
const concurrency = universe::concurrency
System.print(concurrency.name)
const fibers = concurrency::fibers
```

## I-09 — `.` and `::` remain observably distinct on descriptors

```phalcom
concurrency.name
```

uses the ordinary `Module`/`Package` reflective message surface.

```phalcom
concurrency::fibers
```

uses the descriptor's associated namespace.

Implementing `::` by forwarding an ordinary message named `fibers` is incorrect.

## I-10 — package child lookup obeys exposure

A package may have a child in topology that is not exposed. `package::hiddenChild` must not become a bypass around exposure policy.

## I-11 — module lookup obeys linked public exports

`module::Name` resolves only through the module's associated/public namespace, represented canonically by linked exports (and package child exposure where applicable). A private global merely present in `ModuleObject.name_to_slot` is not sufficient.

## I-12 — static expression paths contribute dependency edges

A direct statically resolvable expression path such as:

```phalcom
universe::concurrency::fibers::Fiber
```

must participate in module dependency/topology/invalidation products even when no `import` declaration introduces the dependency.

Imports become local-binding conveniences; they are not the only possible source of a compile-time module dependency.

## I-13 — dynamic descriptor lookup does not fabricate static identity

For:

```phalcom
const descriptor = chooseDescriptor()
descriptor::Thing
```

runtime associated lookup may be legal even when static analysis cannot prove one canonical module target. Do not invent a canonical `ModuleId` merely to make navigation appear complete.

## I-14 — direct static paths and runtime descriptor lookup agree

When the receiver is canonically known, statically collapsed lookup and runtime descriptor lookup must resolve the same public associated value or the same failure.

## I-15 — class-side behavior remains ordinary send

After a path yields `Fiber`, `Fiber.new` and `Fiber.yield` remain message sends. Module qualification must not alter the class/metaclass object model.

---

# 4. Tempting wrong fixes

Do **not** implement any of the following shortcuts.

### Wrong fix A — replace dots only in pretty-printing

Changing `ImportPath::Display` without changing parser/LSP/REPL context detection leaves multiple source dialects and mismatched diagnostics.

### Wrong fix B — split strings on `::` in the LSP

`phalcom-lsp/src/import_completion.rs` may identify syntactic segments for completion context, but canonical resolution must remain through `snapshot.module_queries()` / compiler products. Do not add a second root/exposure resolver.

### Wrong fix C — use `ImportPath::to_string()` as canonical identity

The repository already has `ModuleId`, `ModulePath`, `ImportSiteId`, and canonical resolution products. Rendered path text is provenance/presentation, not identity.

### Wrong fix D — synthesize fake `ImportSiteId`s for expression paths

`ImportSiteId` currently identifies authored import surfaces and participates in exact incremental indexes. Do not overload it for a different source-site category merely because it already has resolution-product infrastructure. Generalize the shared resolver below the site wrapper or introduce a distinct expression/logical-path product with explicit identity.

### Wrong fix E — resolve expression paths only in semantic analysis

If `universe::concurrency::fibers::Fiber` is discovered only after module linking, the module graph can omit a real dependency. Static module-path dependency extraction belongs in the module/workspace layer or in a shared compiler product published before graph/invalidation decisions.

### Wrong fix F — resolve expression paths by filesystem probing

Expression-level associated paths must obey the same project identity, package exposure, Universe root, overlay, and canonical source rules as imports. Reuse `ModuleResolver`/topology products.

### Wrong fix G — treat `ModuleObject` globals as the associated namespace

`module::privateGlobal` must not become visible merely because `name_to_slot` contains it. Use linked/exported namespace products.

### Wrong fix H — implement `module::name` as `module.name`

The current VM has special module-export message-send handling. Reusing it as the definition of `::` would collapse reflection/message and associated lookup back together. Add/use a dedicated associated module lookup route.

### Wrong fix I — add `PackageObject`

The current source/runtime model already has `Package is Module`, and `ModuleObject` carries `ModuleKind`, package/root-package relationships, and exports. Extend the existing runtime registry/materialization model.

### Wrong fix J — make `expose .a::b` work incidentally

`expose` currently records one direct child. If the parser accidentally accepts multiple components, reject them until a separate design changes exposure semantics deliberately.

### Wrong fix K — preserve dotted paths forever as a silent alias

Permanent dual syntax increases formatter/LSP/parser ambiguity. If transitional diagnostics are retained, they must not become a second canonical syntax and must have a deletion gate.

### Wrong fix L — relink the entire workspace on every path edit

The current module architecture was built around stable import sites, component-bounded linking, exact topology deltas, and incremental publication. Preserve that precision.

---

# 5. Implementation program and checkpoint map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| C0 | 1–3 | Logical declaration paths parse/render canonically with `::`; relative anchors and direct-child expose remain intact | AST/parser focused tests, syntax negative cases, `cargo check -p phalcom-ast` | module/linker behavior, LSP, runtime expression paths |
| C1 | 4–7 | `phalcom-modules` resolves the new spelling to unchanged canonical identities and linked surfaces | resolver/interface/linker/fingerprint tests including hostile exposure/relative cases | cold/incremental semantic parity, editor behavior |
| C2 | 8–11 | Persistent module + semantic workspace publications preserve precise identity-keyed incrementality under new path syntax | module-session and semantic incremental tests; cold/incremental canonical-target parity | LSP presentation, expression-level `::` |
| C3 | 12–15 | LSP, REPL, source indexes, fixtures, current docs emit/understand only the new logical path spelling without independent resolution | LSP completion/source-target tests, REPL tests, negative searches | runtime module/package associated lookup |
| C4 | 16–20 | Direct expression-level module/package `::` paths resolve canonically, publish dependency products, and produce exact semantic targets where statically known | semantic/module integration tests; dependency/invalidation hostile cases | runtime execution and final editor integration |
| C5 | 21–24 | Runtime module/package descriptors execute associated lookup distinctly from message send, and static lowering agrees with runtime lookup | core/runtime tests, direct vs descriptor-valued parity, exposure/private hostile cases | broad workspace gates |
| C6 | 25–27 | Repository-wide source migration is complete and compiler/LSP/runtime behavior is consistent | focused integration suites, negative searches, final broad gates | none |

---

# 6. Checkpoint C0 — Canonical logical path grammar

Tasks:
- Task 1 — Change `ImportPath` rendering to `::` components.
- Task 2 — Parse absolute/relative import/re-export paths with `::`.
- Task 3 — Preserve direct-child `expose` and add syntax migration diagnostics/tests.

## Why this is a checkpoint

Every downstream module product consumes `ImportPath`. The AST must establish one stable structural representation before resolver and incremental-product evidence is meaningful. The checkpoint is complete when source spelling changes without changing the semantic path structure.

## Entry conditions

- Baseline parser tests pass before edits.
- `ImportPath` still represents root mode + ordered components.
- Companion associated-lookup parser work has established expression-level `::` without changing declaration-path ownership.

## Working set

Primary:

- `phalcom-ast/src/ast.rs`
- `phalcom-ast/src/parser.rs`
- `phalcom-ast/src/error.rs`
- `phalcom-ast/tests/parser.rs`
- any dedicated module-syntax AST tests already present

Secondary — inspect only if evidence requires it:

- lexer token adjacency/`ColonColon` handling
- formatter or syntax-pretty-printer crate if one exists in current HEAD

Out of scope:

- `phalcom-modules` resolution
- semantic analysis
- runtime modules
- LSP completion

## Semantic contract established by C0

- `app::support::graph` parses as an absolute `ImportPath` with components `[app, support, graph]`.
- `.errors::result` preserves relative depth `1` and ordered components `[errors, result]`.
- `..concurrency` preserves relative depth `2` and component `[concurrency]`.
- `ImportPath::to_string()` / `Display` emits the canonical new spelling.
- `expose .child` remains a direct-child declaration.
- old dotted multi-component source paths no longer silently parse as canonical logical paths.

## Semantic risks

- treating `.` after a relative anchor as a component separator;
- consuming the first `::` as expression associated syntax inside declaration parsing;
- changing relative-dot count;
- accidentally accepting nested expose;
- ranges pointing at separators rather than logical components, degrading source indexing.

## Hostile cases

- `.errors::result` must not be parsed as `.` member syntax.
- `..support::logging` must not become relative depth `1` plus a component beginning with `.`.
- `app.support::graph` must not partially succeed as a mixed path dialect.
- `expose .a::b` must be rejected under current direct-child semantics.
- whitespace behavior around `::` must follow the existing token grammar consistently; do not invent a path-only whitespace exception.

## Required evidence

1. Focused `phalcom-ast` parser tests for absolute and relative paths — prove structural AST equality and ranges.
2. Focused re-export tests — prove path grammar is shared.
3. Expose tests — prove direct-child behavior remains unchanged.
4. Negative old/mixed spelling tests — prove one canonical grammar.
5. `cargo check -p phalcom-ast` — proves AST/parser caller exhaustiveness compiles.

## Do not run yet

- `cargo test --workspace` — no cross-crate semantic evidence exists yet.
- LSP suites — they still intentionally use old context parsing until C3.

## Escalate immediately if

- `ImportPath` structural fields must change merely to represent `::`;
- parsing import paths requires reusing expression AST nodes;
- an existing supported `expose` form is more than one direct component in current HEAD.

## Checkpoint completion

- [ ] all C0 tasks implemented
- [ ] parser/AST focused tests pass
- [ ] relative hostile cases pass
- [ ] direct-child expose rule proven
- [ ] mixed/old canonical syntax negative cases pass
- [ ] implementation state updated
- [ ] no active incident remains

### Task 1 — Change `ImportPath` rendering to `::` components

Purpose:
Make the structural `ImportPath` render the new canonical spelling without changing canonical path identity.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file

Owned files and symbols:

- `phalcom-ast/src/ast.rs` — `ImportPath`, `ImportRoot`, `Display for ImportPath` or equivalent rendering implementation.
- tests asserting path `to_string()` / display output.

Inspect before editing:

- every `impl Display`/`to_string` helper for `ImportPath`;
- call sites that use rendered paths as diagnostics/provenance;
- `phalcom-modules/src/resolver.rs` prefix rendering, but do not edit it until C1.

Do not inspect unless evidence forces expansion:

- runtime VM
- semantic type checker

Dependencies:
- none beyond current AST.

Source of truth:
- `ImportRoot` + ordered `PathSegment`s.

Implementation boundary:

Changes:

1. OPEN `phalcom-ast/src/ast.rs`.
2. FIND `ImportPath` and its display/string rendering.
3. RETAIN leading relative dots exactly.
4. CHANGE only component separation after the root/anchor to `::`.
5. UPDATE comments/examples next to the type.
6. SEARCH consumers that compare exact `ImportPath::to_string()` output; record them for C1–C3 rather than silently changing their semantics now.

Must not:

- change `ModulePath` or `ModuleId`;
- encode punctuation into the path's structural identity;
- add compatibility fields storing both dotted and `::` forms.

Current implementation:
- path semantics are structural but display uses `.` component separators.

Target implementation:
- same AST values render `::` between logical components.

Code instructions:

STRUCTURAL:

```text
Absolute([app, support, graph])
    -> "app::support::graph"

Relative(dots=1, [errors, result])
    -> ".errors::result"

Relative(dots=2, [concurrency])
    -> "..concurrency"
```

Testing classification:
- Focused AST/parser assertions belong in C0 evidence.

Checkpoint state update:
Record the exact render helper changed and any production consumers of the old rendered string.

### Task 2 — Parse declaration logical paths with `::`

Purpose:
Move import/selective-import/direct-re-export path parsing to the new separator while retaining the existing `ImportPath` product.

Risk:
- Semantic: HIGH
- Implementation fanout: local parser with broad downstream syntax fanout

Owned files and symbols:

- `phalcom-ast/src/parser.rs` — `parse_import_path`, import parsing, `parse_reexport_decl`.
- `phalcom-ast/src/error.rs` — migration-specific structured syntax diagnostics if needed.

Inspect before editing:

- `parse_import_path`;
- parser token utilities for `Token::ColonColon`;
- current relative-dot parsing;
- parser tests for import/re-export.

Source of truth:
- `ImportPath` structural AST.

Implementation boundary:

Changes:

1. Preserve leading `.` run parsing and `u16`/existing relative-depth representation.
2. After the absolute first segment or relative anchor + first segment, consume additional components only after `Token::ColonColon`.
3. Preserve each `PathSegment.range` on identifiers, not on separator punctuation.
4. Ensure selective import's `import` keyword terminates the path correctly.
5. Ensure direct re-export's `from` path uses the same helper.
6. Reject mixed dotted/`::` multi-component paths with a focused syntax error rather than producing a misleading later module error.
7. Keep declaration-path parser separate from expression parsing.

Must not:

- parse `from app::x import Y` by first creating an `Expr::AssociatedLookup` chain;
- treat relative anchor dots as ordinary `Token::Dot` member accesses;
- change import alias/item semantics.

Code instructions:

STRUCTURAL:

```text
parse_import_path:
  parse absolute-first-segment OR leading-relative-dot-run
  parse first component where required
  while current token is ColonColon:
      consume ColonColon
      require identifier component
      append PathSegment
  return ImportPath
```

Use the repository's actual parser helpers/token APIs rather than copying this pseudocode literally.

Testing classification:
- Focused parser regression required at C0.

### Task 3 — Preserve direct-child `expose` and establish migration diagnostics

Purpose:
Ensure exposure semantics do not drift while surrounding path syntax changes.

Risk:
- Semantic: MEDIUM
- Implementation fanout: local

Owned files and symbols:

- `phalcom-ast/src/parser.rs` — `parse_expose_decl`.
- `phalcom-ast/src/ast.rs` — `ExposeDecl` comments only unless repository evidence requires a mechanical update.
- `phalcom-ast/tests/parser.rs` — expose tests.

Source of truth:
- `ExposeDecl.child: PathSegment` and module-layer `exposed_children: BTreeSet<ModuleComponent>`.

Changes:

- Keep `expose .child` parsing.
- Reject trailing `::child` after the direct child.
- Update comments/docs that inaccurately describe expose as an arbitrary path.
- Add old/mixed logical-path diagnostics only where a stable structured parser error materially improves migration; do not keep a hidden fallback accepting the old syntax.

Must not:
- generalize `ExposeDecl` to `ImportPath` in this program.

Testing classification:
- C0 focused test.

Suggested commit grouping for C0:

```text
C0.1 syntax(ast): render logical module paths with associated separators
C0.2 syntax(parser): parse import and re-export paths with ::
C0.3 test(syntax): lock relative anchors and direct-child expose
```

---

# 7. Checkpoint C1 — Canonical module resolution and linking remain authoritative

Tasks:
- Task 4 — Update `ModuleResolver` written-prefix production without changing canonical resolution.
- Task 5 — Preserve interface/re-export/exposure linking semantics.
- Task 6 — Preserve path and interface fingerprint identity.
- Task 7 — Add resolver/linker hostile cases and remove dotted-path assumptions.

## Why this is a checkpoint

C1 proves that new source spelling is merely a surface migration over the existing canonical module architecture. The checkpoint is not complete until relative roots, Universe roots, exposure, direct re-export, module-valued exports, and fingerprints all yield the same canonical identity semantics.

## Entry conditions

- C0 COMPLETE.
- `ImportPath` new spelling is canonical.
- current module resolver baseline is green before edits.

## Working set

Primary:

- `phalcom-modules/src/resolver.rs`
- `phalcom-modules/src/interface.rs`
- `phalcom-modules/src/linker.rs`
- `phalcom-modules/src/fingerprint.rs`
- module resolver/interface/linker tests

Secondary:

- `phalcom-modules/src/identity.rs`
- `phalcom-modules/src/query.rs`
- `phalcom-modules/src/topology.rs`

Out of scope:

- semantic incremental session edits
- LSP source parsing
- runtime expression associated lookup

## Semantic contract established by C1

- new and old structural equivalents resolve to the same canonical `ModuleId` conceptually; only new source spelling is accepted.
- `ImportResolutionProduct.written_path` and prefix strings use `::`.
- direct re-export preserves exact `LinkedExportTarget` identity.
- package exposure rules are unchanged.
- `interface_fingerprint` remains based on path structure rather than separator punctuation.
- Universe absolute root resolution remains canonical.

## Semantic risks

- updating `written_path` but leaving prefix strings dotted;
- accidentally using rendered strings as resolver lookup keys;
- exposure traversal skipping a package boundary;
- re-export target losing `ModuleId` identity;
- source-input fingerprints being mistaken for semantic fingerprints.

## Hostile cases

- an existing-but-unexposed external child remains rejected;
- a same-named module in another project remains distinct;
- a relative path attempting to climb above project root still fails;
- `universe::...` reaches `ProjectIdentity::Universe`, not a user project named `universe`;
- legacy forbidden root aliases remain forbidden according to current resolver policy;
- re-export of a whole module remains `LinkedExportTarget::Module`, not a binding guessed from the leaf name.

## Required evidence

1. Resolver tests for absolute, relative, Universe, invalid-root and exposure behavior.
2. Interface/linker test for direct re-export and module-valued export identity.
3. Fingerprint test proving structural path hashing remains canonical.
4. Negative search for production string splitting on `.` in `phalcom-modules` logical-path resolution.
5. `cargo test -p phalcom-modules` at checkpoint completion if focused tests pass.

## Do not run yet

- semantic workspace full suite — C2 owns incrementality evidence.
- LSP suite — C3 owns presentation/context migration.

## Escalate immediately if

- canonical resolver depends on `ImportPath::to_string()` for identity rather than provenance;
- changing punctuation alters `ModuleId` formation;
- `LinkedExportTarget::Module` has regressed or disappeared in current HEAD.

## Checkpoint completion

- [ ] Tasks 4–7 complete
- [ ] canonical resolver tests pass
- [ ] exposure hostile tests pass
- [ ] whole-module export identity passes
- [ ] fingerprint evidence passes
- [ ] no parallel resolver introduced
- [ ] state updated
- [ ] no incident remains

### Task 4 — Update `ModuleResolver` written-prefix production

Purpose:
Keep `ModuleResolver` as the single interpreter of logical module paths while updating only authored/prefix rendering to `::`.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file / module-core

Owned files and symbols:

- `phalcom-modules/src/resolver.rs`
  - `ModuleResolver`
  - `resolve_import_with_trace`
  - `resolve_import_product*`
  - `ImportResolutionProduct`
  - `ResolvedImportPrefix`

Inspect before editing:

- every construction of `written_path`;
- prefix accumulation inside `resolve_import_with_trace`;
- dependency collection/fingerprint generation;
- absolute Universe/root handling;
- relative prefix arithmetic.

Source of truth:
- canonical `ModuleId`/`ModulePath` produced by resolver traversal.

Changes:

1. Let `ImportPath::to_string()` provide the new written path where already appropriate.
2. Replace manual prefix concatenation that inserts `.` between components with a shared/canonical renderer or `::` construction.
3. Do not change target module identity arithmetic.
4. Preserve each `ResolvedImportPrefix.module` exactly.
5. Preserve resolver dependency collection and exposure checks.
6. Search for maps keyed by written prefix strings; classify each as presentation/provenance vs semantic authority before changing.

Must not:
- add a new `resolve_colon_colon_path` beside `resolve_import_with_trace` with duplicate project/exposure rules.

Testing classification:
- checkpoint C1.

### Task 5 — Preserve interface, re-export, and exposure linking semantics

Purpose:
Ensure the source syntax migration does not alter module public-surface ownership.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:

- `phalcom-modules/src/interface.rs` — `InterfaceBuilder::build`, `ImportSurface`, `UnlinkedExportTarget`.
- `phalcom-modules/src/linker.rs` — import/re-export linking, `LinkedExportTarget` production.

Inspect before editing:

- direct re-export resolution path;
- whole-module import binding handling;
- package expose checks;
- tests for module-valued re-export.

Source of truth:
- `UnlinkedModuleInterface` + canonical import products -> linked interface.

Changes:

- Mechanical syntax fixture changes should be enough for most interface code because it consumes structural `ImportPath`.
- Update any diagnostics that print old dotted path examples.
- Verify `exposed_children` continues to hold one direct component.
- Verify module-valued re-export survives as `LinkedExportTarget::Module`.

Must not:
- flatten module-valued exports into leaf-name bindings;
- make a child public solely because the physical source exists.

Testing classification:
- C1 integration evidence.

### Task 6 — Preserve semantic fingerprints across separator migration

Purpose:
Keep semantic product identity based on path structure, while allowing source-input fingerprints to reflect changed source/ranges as intended.

Risk:
- Semantic: HIGH
- Implementation fanout: local but incrementality-sensitive

Owned file and symbols:

- `phalcom-modules/src/fingerprint.rs`
  - `hash_import_path`
  - interface input/product fingerprint functions

Source of truth:
- root kind/depth + ordered component names.

Changes:

- Do not add separator bytes to semantic path hashing.
- Update tests/comments to state canonical `::` spelling.
- Add a focused assertion that structural hash is driven by components rather than rendered separator text.
- Keep source/provenance range hashing unchanged.

Hostile test:
- two different component sequences must remain different even if a careless renderer could produce similar text; do not replace structural hashing with a string hash.

Testing classification:
- focused C1 regression.

### Task 7 — Resolver/linker migration and deletion gates

Purpose:
Remove old source-spelling assumptions from the module authority layer.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file mechanical

Edit operations:

1. SEARCH production `phalcom-modules` for examples/manual joins/splits of logical paths using `.`.
2. Classify filesystem extension/path uses separately; do not mechanically replace filesystem dots.
3. Update only logical-path source rendering/parsing assumptions.
4. Update resolver/interface/linker test fixtures to `::`.
5. Add hostile tests listed above.
6. Record intentionally retained historical documentation occurrences rather than editing archived implementation history indiscriminately.

Suggested commit grouping for C1:

```text
C1.1 refactor(modules): canonicalize logical path rendering through ::
C1.2 test(modules): preserve resolution, exposure, re-export and fingerprint identity
```

---

# 8. Checkpoint C2 — Persistent module and semantic incrementality preserve exact products

Tasks:
- Task 8 — Migrate `WorkspaceModuleSession` import-site products without widening invalidation.
- Task 9 — Keep `SemanticWorkspaceSession` dependent on canonical identity-keyed module products.
- Task 10 — Update compiler-owned import path occurrences and compatibility fallbacks.
- Task 11 — Prove cold/incremental equivalence and bounded work.

## Why this is a checkpoint

The repository's pyrefly-inspired architecture depends on persistent exact products rather than repeated whole-workspace reconstruction. Path spelling changes can easily reintroduce string-keyed invalidation or broad relinking. C2 proves the syntax migration is absorbed by the existing canonical incremental pipeline.

## Entry conditions

- C1 COMPLETE.
- `WorkspaceModuleSession` baseline incremental tests are known green at local HEAD before edits.

## Working set

Primary:

- `phalcom-modules/src/session.rs`
- `phalcom-modules/src/topology.rs`
- `phalcom-modules/src/query.rs`
- module-session incremental tests
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/workspace.rs`
- `phalcom-semantic/src/snapshot.rs`
- `phalcom-semantic/src/source_index/occurrence.rs`
- semantic incremental tests

Secondary:

- `phalcom-semantic/src/db/query.rs`
- `phalcom-semantic/src/source_index/builder.rs`

Out of scope:

- LSP line-prefix parser
- module descriptor runtime `::`

## Semantic contract established by C2

- `ImportSiteId` remains the identity for authored import resolution products.
- new path punctuation does not create duplicate canonical targets.
- a body-only edit with unchanged import/interface products does not force module-graph relinking.
- a genuine logical path target change invalidates the exact affected import/dependent frontier.
- production semantic source indexing consumes canonical import products.
- cold and incremental analysis agree on `ModuleId` / `SemanticTargetId` for imports after edits.

## Semantic risks

- comparing `written_path` strings to determine semantic reuse;
- rebuilding all import products because display spelling changed;
- semantic fallback map becoming production authority;
- source ranges moving after `.` -> `::` and causing target identity loss;
- LSP worker accidentally maintaining stale old-generation path products.

## Hostile cases

- edit an unrelated body in a module with a `::` import: import resolution product should be reused where current incremental contract says it is reusable;
- change one import target from `a::x` to `a::y`: only relevant topology/import/dependent products should change;
- add an unused export in a provider: an unrelated importer should not be semantically recomputed merely because path rendering changed;
- cold analysis and an edit-driven incremental analysis must attach the same canonical target to each path component;
- remove/re-add an overlay source: source identity and module identity remain canonical.

## Required evidence

1. Focused `WorkspaceModuleSession` import-product reuse/invalidation tests.
2. Existing affected-component/high-fanout tests, only where the changed code falls inside their semantic boundary.
3. Semantic incremental imported-binding/path occurrence regression.
4. Cold vs incremental comparison of `ModuleId`, import target and source-index target.
5. Module/semantic stats assertions proving no accidental full-workspace fallback.

## Do not run yet

- full LSP integration suite — C3.
- runtime core suite — C5.

## Escalate immediately if

- a passing incremental test requires reintroducing a path-string fallback into production;
- the module session lacks sufficient canonical product identity to determine affected sites;
- path syntax requires a second source scan outside `WorkspaceModuleSession`.

## Checkpoint completion

- [ ] tasks 8–11 complete
- [ ] module-session incremental evidence passes
- [ ] semantic cold/incremental parity passes
- [ ] high-fanout/bounded work evidence passes where relevant
- [ ] production path-string fallback remains disabled
- [ ] state updated
- [ ] no incident remains

### Task 8 — Preserve `WorkspaceModuleSession` exact import products

Purpose:
Keep stable import-site and component-bounded linking behavior under the new syntax.

Risk:
- Semantic: HIGH
- Implementation fanout: module-session core

Owned file and symbols:

- `phalcom-modules/src/session.rs`
  - `WorkspaceModuleSession`
  - import product roots/indexes
  - `WorkspaceModuleUpdate`
  - `WorkspaceModuleStats`

Inspect before editing:

- import product cache key/reuse comparisons;
- `sites_by_importer` and `reverse_site_importers` updates;
- graph-change detection;
- topology invalidation;
- tests added by recent stable-import-site/affected-component commits.

Source of truth:
- `ImportSiteId` + structural `ImportPath` + canonical resolver product.

Changes:

- Update only assumptions that compare/render authored path strings.
- Preserve copy-on-write map behavior and exact worklists.
- If any reuse fingerprint includes written path presentation, split provenance from canonical semantic dependency rather than making the new punctuation itself a semantic graph change.
- Keep source revision/range changes observable where required for editor products.

Must not:
- key canonical import reuse solely by `written_path`.

Testing classification:
- C2 checkpoint tests.

### Task 9 — Preserve semantic workspace canonical product consumption

Purpose:
Keep compiler semantics and LSP-visible snapshots downstream of the module session.

Risk:
- Semantic: HIGH
- Implementation fanout: cross-crate

Owned files and symbols:

- `phalcom-semantic/src/session.rs` — module update -> semantic input/publication.
- `phalcom-semantic/src/workspace.rs` — `SemanticWorkspaceInput`.
- `phalcom-semantic/src/snapshot.rs` — retained canonical module products.

Source of truth:
- `WorkspaceModuleUpdate` canonical products.

Changes:

- Verify no new path parsing is needed here.
- Update test fixture strings and any presentation-only maps using `ImportPath::to_string()`.
- Preserve `require_canonical_import_products: true` in production updates.
- Preserve topology/reverse-import roots by identity.

Must not:
- reconstruct imports by walking AST strings after `WorkspaceModuleSession` already resolved them.

Testing classification:
- C2 cross-layer parity.

### Task 10 — Update compiler-owned import path source occurrences

Purpose:
Keep go-to-definition/reference identity on each new-syntax path segment.

Risk:
- Semantic: MEDIUM
- Implementation fanout: local semantic source-index

Owned file and symbols:

- `phalcom-semantic/src/source_index/occurrence.rs`
  - `OccurrenceBuilder::dependencies`
  - `path_occurrences`
  - `resolve_import_path`

Changes:

- Because component ranges come from AST identifiers, preserve occurrence attachment to root and each segment.
- Continue to prefer `context.import_products[ImportSiteId]` and its resolved prefixes.
- Update compatibility-only `path.to_string()` fallbacks to new spelling.
- Do not enable compatibility fallback when `require_canonical_import_products` is true.

Hostile case:
- go-to-definition on `support` in `app::support::graph` targets the prefix module, not always the final `graph` target.

Testing classification:
- one source-index regression can prove all prefix targets.

### Task 11 — Cold/incremental path product equivalence

Purpose:
Prove the persistent pipeline produces the same canonical module facts after edits as a cold analysis.

Risk:
- Semantic: HIGH
- Implementation fanout: test integration

Test design:

Create/reuse a small package fixture with:

```text
app/package.ph
app/support/package.ph
app/support/graph.ph
consumer.ph
```

Consumer uses absolute and relative `::` imports. Capture at minimum:

- resolved `ModuleId`;
- import resolution prefix targets;
- `SemanticTargetId` for path occurrences;
- relevant interface/link fingerprint;
- recomputation stats.

Apply an overlay edit that changes a non-path body, then one that changes a path target. Compare incremental publication against a freshly built cold workspace for each state.

Testing classification:
- checkpoint-defining regression.

Suggested commit grouping for C2:

```text
C2.1 fix(modules): retain canonical import-site incrementality with :: paths
C2.2 test(semantic): prove cold/incremental path target parity
```

---

# 9. Checkpoint C3 — LSP, REPL, source presentation, fixtures, and current docs use one syntax

Tasks:
- Task 12 — Rewrite LSP import context detection for `::` and leading relative anchors.
- Task 13 — Keep LSP completion resolution snapshot-backed and update insertion/presentation.
- Task 14 — Update REPL import path completion.
- Task 15 — Migrate path fixtures/examples/current documentation and add deletion gates.

## Why this is a checkpoint

The compiler/module pipeline can be correct while editor tooling still emits stale dotted paths or parses them into incorrect contexts. C3 closes the user-facing declaration-path surface without changing semantic authority.

## Entry conditions

- C2 COMPLETE.
- canonical compiler snapshots expose required topology/public exports.

## Working set

Primary:

- `phalcom-lsp/src/import_completion.rs`
- LSP completion/context tests
- `phalcom-lsp/src/analysis_service.rs` only for integration verification, not path semantics
- `phalcom-repl/src/completer.rs`
- module-related examples/fixtures/current docs

Secondary:

- LSP server completion trigger configuration if current client behavior requires `:` trigger support
- source-index tests

Out of scope:

- module resolver implementation
- runtime descriptor associated lookup

## Semantic contract established by C3

- typing `import app::sup` is classified as import-child context.
- typing `from .errors::res import` preserves relative anchor and components.
- completion candidates come from `snapshot.module_queries()`.
- `expose .child` completion still enumerates raw direct children, including currently unexposed ones.
- REPL suggestions use `::`.
- current examples/fixtures no longer teach dotted logical component traversal.

## Semantic risks

- naive `split(':')` misparsing `::`;
- treating leading relative dots as empty components;
- using external-import exposure filtering for `expose` completion, which would hide the very child being exposed;
- LSP completion producing correct labels but inserting old punctuation;
- changing worker ownership by moving resolver logic into the LSP.

## Hostile cases

- `import app::` yields child completion with empty partial.
- `import .errors::` yields relative child completion.
- `from app::support::graph import D` yields exports of exactly that canonical module.
- `expose .hiddenChild` can offer an existing unexposed direct child.
- a similarly named child from another project is not offered through the wrong root.

## Required evidence

1. `phalcom-lsp` `detect_import_context` focused unit tests for new absolute/relative forms.
2. completion test using snapshot/module query facade, including exposure hostile case.
3. REPL completion focused tests.
4. source fixture compilation/parsing evidence after bulk migration.
5. negative search for current/source examples of dotted logical paths, with filesystem dots and archived historical docs excluded/justified.

## Do not run yet

- runtime VM tests for `module::name` — C5.

## Escalate immediately if

- LSP completion needs filesystem traversal because module query facade lacks data;
- client completion trigger characters require a product-level decision outside this spec rather than a mechanical `:` addition.

## Checkpoint completion

- [ ] tasks 12–15 complete
- [ ] LSP context/completion tests pass
- [ ] REPL completion tests pass
- [ ] fixtures parse/compile with canonical syntax
- [ ] current docs/examples negative search clean
- [ ] no LSP resolver introduced
- [ ] state updated
- [ ] no incident remains

### Task 12 — Rewrite LSP import context detection

Purpose:
Make the syntax adapter understand the parser's new source spelling without becoming a resolver.

Risk:
- Semantic: MEDIUM
- Implementation fanout: local

Owned file and symbols:

- `phalcom-lsp/src/import_completion.rs`
  - `ImportContext`
  - `detect_import_context`

Current implementation:
- manually splits path text on `.` and separately counts leading dots.

Target implementation:
- count leading relative dots first;
- tokenize remaining logical components on `::`;
- retain partial trailing component state;
- distinguish `import`, `from ... import`, and `expose` contexts.

Edit operations:

1. Replace repeated `split('.')` logic with one small syntax-only helper local to LSP completion, or consume a parser/path-prefix helper if one already exists and is suitable for incomplete text.
2. Do **not** call that helper a resolver and do not produce `ModuleId`s from it.
3. Ensure `::` trailing separator yields empty partial component.
4. Keep `expose` direct-child: reject/avoid multi-component exposure completion context.
5. Update comments/examples.

Testing classification:
- focused unit tests in C3.

### Task 13 — Preserve snapshot-backed LSP completion

Purpose:
Keep all candidate identity/exposure decisions compiler-owned.

Risk:
- Semantic: HIGH
- Implementation fanout: local adapter over shared snapshot

Owned file/symbol:

- `phalcom-lsp/src/import_completion.rs` — `import_completions`.

Source of truth:
- `snapshot.module_queries()`.

Changes:

- Continue `module_children` for self/relative package traversal.
- Continue `external_import_children` for externally exposed traversal.
- Continue `public_exports` for selective member completion.
- Update only path context/presentation assumptions.
- For `expose`, continue using raw `module_children`, not exposure-filtered children.

Must not:
- read the filesystem to resolve candidates;
- build `ModuleId` from display text when query facade already supplies canonical identities.

Testing classification:
- checkpoint integration test.

### Task 14 — Update REPL logical path completion

Purpose:
Keep REPL-authored import paths aligned with parser/LSP canonical syntax.

Risk:
- Semantic: LOW
- Implementation fanout: local

Owned file:
- `phalcom-repl/src/completer.rs` — import-path context/suggestion helpers.

Changes:
- update logical path separator parsing/rendering to `::`;
- preserve filesystem path handling elsewhere;
- add focused completion tests if existing coverage exists.

Testing classification:
- focused REPL test or crate check at C3.

### Task 15 — Migrate current fixtures/examples/documentation for logical paths

Purpose:
Prevent stale syntax from re-seeding old behavior after implementation.

Risk:
- Semantic: LOW for edits; HIGH migration completeness
- Implementation fanout: repository-wide source/docs

Search categories:

1. `.ph` sources under `phalcom-core/core`, `examples`, test fixtures.
2. parser/module/semantic/LSP test fixture strings.
3. current language docs/guide/wiki pages that describe active syntax.
4. ADR/implementation-history/archive documents — do not blindly rewrite historical evidence; identify intentional retained old syntax.

Required edits:

- absolute logical paths: `a.b.c` -> `a::b::c` only where the text is a Phalcom logical path, never filesystem/member access;
- relative paths: `.a.b` -> `.a::b`, `..a.b` -> `..a::b`;
- direct re-export paths similarly;
- `expose .child` remains unchanged.

Negative gates:

Use targeted searches, not a global replacement of every dot. Record every intentionally retained occurrence in historical docs.

Suggested commit grouping for C3:

```text
C3.1 feat(lsp): understand :: logical path contexts from compiler snapshots
C3.2 chore(repl): emit canonical :: import paths
C3.3 chore(sources): migrate current logical path fixtures and docs
```

---

# 10. Checkpoint C4 — Expression-level module/package associated lookup is canonical and dependency-aware

Tasks:
- Task 16 — Introduce/extend semantic denotation for canonical module/package descriptor identity.
- Task 17 — Generalize the module resolver below import-site wrappers for static expression paths.
- Task 18 — Publish static expression-path resolution/dependency products in `WorkspaceModuleSession`.
- Task 19 — Resolve `::` against module/package descriptor namespaces in semantic analysis.
- Task 20 — Attach canonical source/editor targets and prove incremental dependency behavior.

## Why this is a checkpoint

Making `universe::concurrency::fibers::Fiber` parse is insufficient. Static expression paths must resolve through the same canonical module authority, contribute dependencies before graph/invalidation decisions, and carry exact semantic targets. These products must integrate before runtime lowering can safely optimize them.

## Entry conditions

- C3 COMPLETE for declaration-path migration.
- Companion `::` implementation has removed ordinary receiver-bound method fallback from associated lookup.
- canonical module topology/linking products are stable.

## Working set

Primary:

- `phalcom-modules/src/resolver.rs`
- `phalcom-modules/src/session.rs`
- `phalcom-modules/src/interface.rs` / `linker.rs` only for public namespace queries
- `phalcom-semantic/src/checker/associated.rs`
- `phalcom-semantic/src/checker/expression.rs`
- `phalcom-semantic/src/types/denotation.rs`
- `phalcom-semantic/src/db/fingerprint.rs`
- `phalcom-semantic/src/source_index/occurrence.rs`
- `phalcom-semantic/src/identity.rs`
- semantic/module incremental tests

Secondary:

- `phalcom-modules/src/query.rs`
- `phalcom-semantic/src/session.rs`
- source structure/index builders that enumerate expression nodes

Out of scope:

- general constant propagation
- runtime `ModuleObject` lookup (C5)
- callable-reference typing

## Semantic contract established by C4

- `universe::concurrency::fibers::Fiber` resolves each known package/module prefix canonically.
- a module/package descriptor value can be the receiver of `::`.
- module/package associated lookup uses public linked exports and exposed child relationships, not ordinary method lookup.
- direct static paths contribute exact module dependency edges/products.
- statically known prefixes receive `SemanticTargetId::Module`; final exported bindings receive their existing canonical target kind.
- unresolved/dynamic descriptor receivers remain dynamic/unknown targets without fabricated module identity.
- path-related edits invalidate the same canonical dependency frontier in cold and incremental analysis.

## Semantic risks

- discovering dependency only after module graph publication;
- making all associated lookup require `SemanticDenotation::TypeForm` and therefore rejecting module descriptors;
- broadening `SemanticDenotation` without fingerprint/query updates;
- exposing raw topology children rather than exposure-filtered associated namespace;
- treating final export binding as a `ModuleId` when it is a `SymbolId`/declaration;
- using import-site IDs for expression sites and corrupting stable import worklists.

## Hostile cases

- direct absolute expression path with no import still creates a dependency on the target module.
- unexposed external package child is rejected.
- private module global is rejected even though runtime storage contains it.
- final exported module produces a module descriptor and can continue `::` traversal.
- final exported class produces class/type value and `.new` remains ordinary message send.
- `const x = chooseDescriptor(); x::Thing` does not claim one exact module target when analysis cannot prove it.
- editing the exported binding/child exposure invalidates the direct expression consumer incrementally.

## Required evidence

1. Ownership-layer module test for static expression-path resolution product and dependency edges.
2. Semantic test for module/package associated lookup exact target kind.
3. Hostile exposure/private-global tests.
4. Cold/incremental edit test comparing dependency target and semantic target.
5. Negative search proving semantic/LSP did not grow a duplicate project/root resolver.

## Do not run yet

- full runtime suite — C5.

## Escalate immediately if

- module-session source products cannot identify direct `Expr::AssociatedLookup` chains before link graph computation;
- resolving a static expression path requires semantic type inference for its root in all cases;
- runtime/compiler architecture requires loading a module not represented in canonical module dependency products.

## Checkpoint completion

- [ ] tasks 16–20 complete
- [ ] static expression path dependencies are canonical
- [ ] semantic associated lookup supports module/package descriptors
- [ ] exposure/private hostile cases pass
- [ ] cold/incremental parity passes
- [ ] exact source targets attach where provable
- [ ] no fake import sites / duplicate resolver introduced
- [ ] state updated
- [ ] no incident remains

### Task 16 — Represent canonical module/package descriptor denotation

Purpose:
Allow semantic associated lookup to know when a value denotes one canonical module/package descriptor without confusing its runtime class type with its namespace identity.

Risk:
- Semantic: HIGH
- Implementation fanout: semantic shared product

Owned files and symbols:

- `phalcom-semantic/src/types/denotation.rs` — `SemanticDenotation`.
- `phalcom-semantic/src/db/fingerprint.rs` — denotation fingerprinting.
- expression/name resolution sites that synthesize whole-module bindings or the canonical `universe` root.

Inspect before editing:

- all exhaustive matches on `SemanticDenotation`;
- how imported whole-module bindings are typed today;
- how `SemanticTargetId::Module` is attached in source index;
- whether a current semantic fact already carries exact `ModuleId` and can be reused instead of adding a new variant.

Source of truth:
- canonical `ModuleId`, not descriptor display name.

Implementation boundary:

INVESTIGATE-BEFORE-EDIT:

Determine whether the current semantic products already preserve exact module descriptor identity outside source occurrences. If they do, reuse that fact. If they do not, introduce the narrowest denotation product needed, conceptually:

```text
SemanticDenotation::ModuleDescriptor(ModuleId)
```

A package is identified by `ModuleId` plus the linked/source `ModuleKind::Package`; do not invent a parallel package identity.

If a new denotation variant is added:

- update semantic fact hashing/fingerprinting;
- update merge behavior so equal identities survive merges and differing identities drop to no exact denotation;
- update exhaustive consumers;
- preserve ordinary value type as the runtime `Module`/`Package` object type already used by the semantic system.

Must not:
- encode module identity in a `String` or `Symbol` alone.

Testing classification:
- no isolated test if C4 semantic lookup test observes the denotation end-to-end; add a focused unit only if hashing/merge behavior would otherwise be unproven.

### Task 17 — Generalize canonical path traversal below import-site wrappers

Purpose:
Reuse one root/relative/exposure traversal implementation for imports and static expression paths without pretending the latter are import declarations.

Risk:
- Semantic: HIGH
- Implementation fanout: module resolver core

Owned file and symbols:

- `phalcom-modules/src/resolver.rs`
  - `resolve_import_with_trace`
  - related internal path traversal helpers
  - `ImportResolutionProduct`

Source of truth:
- `ModuleResolver` canonical traversal.

Target architecture:

```text
shared canonical logical-path traversal
    inputs: importer/current module context + structural path/root + policy/site context
    output: canonical prefixes + target + dependencies / resolution failure

ImportResolutionProduct
    wraps shared traversal for ImportSiteId + authored import provenance

ExpressionPathResolutionProduct (name to be reconciled with repo conventions)
    wraps shared traversal for expression/source-site identity
```

The exact new type/name is STRUCTURAL, not paste-ready. Search current module product/site identity abstractions before introducing it.

Edit operations:

1. Extract the project-root, relative-prefix, child/exposure traversal from `resolve_import_with_trace` only where it can be shared without changing behavior.
2. Keep `resolve_import_product_for_site` as the import-specific product constructor.
3. Add a distinct product constructor for statically recognizable expression logical paths.
4. Carry canonical prefix `ModuleId`s and dependency set.
5. Do not expose filesystem source-provider internals to semantic/LSP callers.

Must not:
- duplicate traversal logic;
- synthesize `ImportSiteId` for expression sites;
- weaken exposure policy to make expression lookup easier.

Testing classification:
- C4 module ownership tests.

### Task 18 — Publish static expression-path products through `WorkspaceModuleSession`

Purpose:
Make direct expression-level module paths visible to topology/dependency/invalidation before semantic body analysis relies on them.

Risk:
- Semantic: HIGH
- Implementation fanout: module-session architecture

Owned files/symbols:

- `phalcom-modules/src/session.rs` — source update, interface/product extraction, graph publication.
- potentially a focused new module-layer source-product type/file if current product organization warrants it.
- `phalcom-ast` expression walking helpers only as a consumer; do not relocate syntax ownership.

Inspect before editing:

- how `WorkspaceModuleSession` discovers authored import sites from `UnlinkedModuleInterface`;
- where dependency edges are formed from import products;
- how `sites_by_importer` and reverse indexes are partitioned;
- current source parse products already stored in `WorkspaceSourceState`.

Source of truth:
- expression source site -> shared resolver product -> canonical `ModuleId` dependencies.

Static-path recognition scope:

Required:

1. direct chains rooted in canonical `universe`;
2. direct chains rooted in whole-module bindings whose exact `ModuleId` is already available from module/link products;
3. any receiver site for which an existing compiler-owned fact already carries exact module descriptor identity without new general dataflow.

Not required:

- arbitrary local-variable propagation such as recovering `const p = choose(...); p::x`.

Changes:

- introduce a stable source-site identity for static expression paths following existing module/semantic source-site conventions;
- cache/reuse resolution products by stable site + source/product fingerprint;
- incorporate their canonical target modules into dependency graph/reverse importer products;
- publish exact changes so unrelated modules are not invalidated;
- expose products to semantic publication through `WorkspaceModuleUpdate` or an equivalent canonical field.

Must not:
- merge expression sites into `import_products` while pretending they are imports;
- clone whole workspace maps on ordinary body edits if current COW products avoid it.

Testing classification:
- checkpoint-defining incremental test.

### Task 19 — Resolve `::` on module/package descriptors semantically

Purpose:
Extend associated lookup beyond declaration-backed type forms to namespace-bearing module/package descriptor values.

Risk:
- Semantic: HIGH
- Implementation fanout: semantic checker

Owned files and symbols:

- `phalcom-semantic/src/checker/associated.rs`
  - `resolve_associated_owner`
  - associated lookup resolution entry points
- `phalcom-semantic/src/checker/expression.rs`
  - `synthesize_associated_lookup`
  - `synthesize_associated_invoke` as applicable for final associated callable values

Source of truth:

For declaration/type receivers:
- existing associated member table/family products.

For module/package receivers:
- canonical module/package descriptor identity + linked public namespace / exposure products.

Target owner classification:

STRUCTURAL:

```text
AssociatedOwner
  Declaration(TypeForm / declaration-backed owner)
  ModuleDescriptor(ModuleId, ModuleKind)
  DynamicNamespaceBearingValue   // only if runtime legality is known but exact target is not
```

Do not adopt this enum name mechanically if an existing owner abstraction can be extended.

Module/package lookup behavior:

- package child name -> exposed child `ModuleId` -> module/package descriptor semantic fact;
- module public export -> `LinkedExportTarget::Binding` or `LinkedExportTarget::Module`;
- final binding target -> existing canonical declaration/module-binding target projection;
- missing/unexposed name -> associated lookup diagnostic, not ordinary method `doesNotUnderstand` at static resolution.

Must not:
- route class-side `.new` through this resolver;
- look at raw runtime globals;
- fall back from failed module associated lookup to a message send with the same name.

Testing classification:
- focused semantic C4 integration.

### Task 20 — Expression path source targets and incremental hostile cases

Purpose:
Give editor consumers exact canonical navigation when the compiler proves a static path, and prove dependency invalidation.

Risk:
- Semantic: HIGH
- Implementation fanout: semantic source index + incremental tests

Owned files:

- `phalcom-semantic/src/source_index/occurrence.rs`
- `phalcom-semantic/src/source_index/builder.rs`
- semantic source-index/incremental tests

Changes:

- visit associated lookup chain receiver/member ranges as source occurrences.
- attach each statically resolved module/package segment to `SemanticTargetId::Module`.
- attach final binding using existing target projection (`Declaration`, `ModuleBinding`, `Module`, etc.).
- if static descriptor identity is unavailable, retain a hint/reference occurrence without inventing target.

Cold/incremental hostile sequence:

1. cold analyze direct `universe::...` consumer;
2. edit provider export/body without changing public target identity — consumer recomputation stays bounded according to exact dependency facts;
3. edit exposure/export identity — exact consumer invalidates;
4. compare fresh cold state to incremental target IDs and diagnostics.

Suggested commit grouping for C4:

```text
C4.1 refactor(modules): share canonical path traversal below import products
C4.2 feat(modules): publish static expression-path dependency products
C4.3 feat(semantic): resolve module/package descriptor associated lookup
C4.4 test(incremental): prove static path target and invalidation parity
```

---

# 11. Checkpoint C5 — Runtime module/package associated lookup and lowering

Tasks:
- Task 21 — Project semantic module/package associated resolution into lowering specs.
- Task 22 — Implement dedicated runtime associated namespace lookup on `ModuleObject`/registry products.
- Task 23 — Preserve package/module descriptor materialization and exported module values.
- Task 24 — Prove static-vs-runtime parity and message-send separation.

## Why this is a checkpoint

C4 establishes canonical compile-time identity, but descriptor values must remain first-class. C5 makes `const p = universe::concurrency; p::fibers` execute with the same namespace policy while keeping `p.name` an ordinary message send.

## Entry conditions

- C4 COMPLETE.
- semantic lowering can distinguish module/package associated resolution from declaration-associated variant/member resolution.
- runtime materialization tests for modules/packages are green at baseline.

## Working set

Primary:

- `phalcom-core/src/modules/semantic_lowering.rs`
- compiler associated-expression lowering files, currently including `phalcom-core/src/compiler/lib/associated.rs`
- `phalcom-core/src/heap/module.rs`
- `phalcom-core/src/modules/registry.rs`
- `phalcom-core/src/modules/materialize.rs`
- `phalcom-core/src/modules/builtin_materialize.rs`
- `phalcom-core/src/vm/associated.rs` if it remains the associated runtime owner after companion Spec 1
- `phalcom-core/src/vm/send.rs` for separation tests, not as the new `::` authority
- core module/runtime tests

Secondary:

- `phalcom-core/src/heap/trace.rs`
- reflection cache/primitive files if a new reference must be traced/exposed

Out of scope:

- changing `Package`/`Module` class inheritance
- changing ordinary `Module` reflection methods
- general lazy package loader redesign

## Semantic contract established by C5

- statically resolved module/package associated lookup may lower directly to linked identity/value access.
- descriptor-valued runtime lookup uses a dedicated associated namespace operation.
- both paths enforce the same export/exposure namespace.
- a `RuntimeExportRef::Module` returns the same descriptor object registered for that canonical `ModuleId`.
- binding exports load the canonical live binding value.
- `.name` remains normal message/member behavior and does not collide with `::name` semantics.

## Semantic risks

- a new bytecode duplicates `GetLinked`/module registry semantics unnecessarily;
- runtime lookup accesses private global slot;
- package child descriptors are materialized under duplicate objects;
- GC misses module references returned/cached by associated lookup;
- static lowering bypasses initialization order;
- existing `try_module_export_send` leaks into `::` implementation and collapses the two surfaces.

## Hostile cases

- module has reflective method/getter `name` and an exported associated binding also named `name`: `module.name` and `module::name` must take their respective surfaces.
- package physically has `hidden` child but does not expose it: runtime `package::hidden` fails.
- module has private global `secret` but does not export it: runtime `module::secret` fails.
- exported child module is returned by object identity, not rematerialized.
- direct static path and descriptor-stored path return identical class/module values.
- class object returned at path end still receives `.new` through ordinary dispatch.

## Required evidence

1. lowering test/bytecode assertion for statically known module export vs runtime descriptor lookup.
2. core runtime test for `universe::concurrency::fibers::Fiber.new { ... }` or the smallest existing equivalent module fixture.
3. runtime descriptor-valued chain test.
4. `.name` vs `::name` separation test.
5. unexposed/private hostile tests.
6. module object identity assertion for re-exported child.

## Do not run yet

- broad workspace gates until C6 migration is complete.

## Escalate immediately if

- runtime associated lookup requires a second module registry;
- a module can be referenced through `::` before canonical materialization/initialization semantics can produce its descriptor;
- export/exposure policy is unavailable at runtime and would need to be guessed from source names.

## Checkpoint completion

- [ ] tasks 21–24 complete
- [ ] direct and descriptor-valued runtime lookup pass
- [ ] export/exposure hostile cases pass
- [ ] message/associated surface separation passes
- [ ] module object identity preserved
- [ ] state updated
- [ ] no incident remains

### Task 21 — Add module/package associated lowering specs

Purpose:
Carry semantic resolution to the compiler without forcing runtime re-resolution when identity is already known.

Risk:
- Semantic: HIGH
- Implementation fanout: semantic/core bridge

Owned files/symbols:

- `phalcom-core/src/modules/semantic_lowering.rs`
  - `AssociatedLoweringSpec`
  - `project_associated_resolution`
  - `ModuleLoweringSemantics`
- `phalcom-core/src/compiler/lib/associated.rs` or its post-Spec-1 replacement.

Inspect before editing:

- post-Spec-1 associated lowering variants;
- existing `GetLinked`/linked binding lowering APIs;
- module runtime materialization assumptions.

Source of truth:
- semantic resolved target (`ModuleId` / linked binding identity).

Target lowering categories, conceptually:

```text
LoadAssociatedModule(ModuleId)
LoadAssociatedExport(SymbolId / linked export target)
DynamicModuleAssociatedLookup(name)
```

Names are STRUCTURAL. Reuse existing `GetLinked`/constant/module opcodes when they already express the exact operation safely.

Must not:
- introduce a new opcode merely for syntax if an existing linked-load operation has identical semantics and initialization guarantees.

Testing classification:
- C5 lowering/runtime evidence.

### Task 22 — Implement dedicated runtime associated namespace lookup

Purpose:
Execute `descriptor::name` without routing through ordinary method dispatch.

Risk:
- Semantic: HIGH
- Implementation fanout: runtime/VM

Owned files:

- likely `phalcom-core/src/vm/associated.rs` after companion migration;
- `phalcom-core/src/heap/module.rs` for namespace accessor helper if ownership fits;
- `phalcom-core/src/modules/registry.rs` for canonical module descriptor retrieval.

Inspect before editing:

- `ModuleObject::export`;
- module registry lookup by `ModuleId`;
- package child export/materialization representation;
- `RuntimeExportRef` handling;
- `try_module_export_send` only to understand what must remain separate.

Source of truth:
- `ModuleObject.exports` populated from canonical linked interface/exposure products plus module registry identity.

Target behavior:

```text
runtime_associated_lookup(module_obj, name):
  verify receiver is namespace-bearing module/package descriptor
  lookup name in public associated namespace
  RuntimeExportRef::Binding -> read canonical binding
  RuntimeExportRef::Module  -> return registered module object value
  miss -> associated lookup failure
```

If package direct children are materialized into `exports` by existing `builtin_materialize`/general materialization, reuse that table. If general packages maintain child associations elsewhere, reconcile the one authoritative representation before editing; do not add a second child-name map without evidence.

Must not:
- call `try_module_export_send` as if `::` were a getter send;
- call `doesNotUnderstand` for a static associated namespace miss unless the language's associated-lookup runtime contract explicitly owns such behavior after Spec 1.

Testing classification:
- focused core runtime tests.

### Task 23 — Preserve canonical descriptor materialization

Purpose:
Guarantee associated lookup returns existing module/package descriptor objects and linked export values.

Risk:
- Semantic: HIGH
- Implementation fanout: module runtime materialization

Owned files/symbols:

- `phalcom-core/src/modules/materialize.rs`
- `phalcom-core/src/modules/builtin_materialize.rs`
- `phalcom-core/src/modules/registry.rs`
- `phalcom-core/src/heap/module.rs` — `RuntimeExportRef`

Changes:

- Verify `LinkedExportTarget::Module` becomes `RuntimeExportRef::Module` pointing at the registry-owned object.
- Verify package direct exposed children are represented in the runtime associated namespace.
- Preserve package/root-package links and `ModuleKind`.
- Do not allocate a fresh descriptor on each `::` lookup.
- Ensure dependency initialization/materialization occurs according to existing linked program order.

Testing classification:
- core module materialization tests.

### Task 24 — Runtime parity and surface-separation verification

Purpose:
Defeat the easiest incorrect implementations.

Risk:
- Semantic: HIGH
- Implementation fanout: integration tests

Required test fixture behaviors:

1. **Static chain:** direct `universe::...::Fiber` returns expected class object.
2. **Stored descriptor:** assign intermediate package/module descriptor then continue with `::`.
3. **Reflection separation:** `.name` reflects descriptor while `::name` resolves exported/associated name when such a fixture is constructed.
4. **Private rejection:** raw global but no export cannot be reached with `::`.
5. **Exposure rejection:** physical child but not exposed cannot be reached externally.
6. **Object identity:** direct path, re-export path, and descriptor lookup all return the same module object for one `ModuleId`.
7. **Class-side send:** final class `.new` uses ordinary method dispatch and remains replaceable/overridable according to existing class-side semantics.

Suggested commit grouping for C5:

```text
C5.1 feat(core): lower canonical module/package associated lookup
C5.2 feat(vm): execute descriptor associated namespace lookup separately from sends
C5.3 test(core): enforce exposure, privacy, identity and . / :: separation
```

---

# 12. Checkpoint C6 — Repository migration and delivery closure

Tasks:
- Task 25 — Migrate remaining current source/tests/examples/spec references.
- Task 26 — Run cross-consumer consistency and deletion gates.
- Task 27 — Run final broad delivery gates and finalize implementation state.

## Why this is a checkpoint

The change is incomplete while old syntax remains in active fixtures or while compiler/module/LSP/runtime consumers can disagree. C6 is the delivery boundary, not a place to introduce new semantics.

## Entry conditions

- C0–C5 COMPLETE.
- no active incident.

## Working set

Primary:
- current `.ph` source tree
- active tests/fixtures/examples
- current language docs and guide
- compiler/LSP/core integration tests

Historical implementation/archive docs:
- inspect only for misleading “current” references; retain historical snapshots where appropriate and record them as justified negative-search hits.

## Semantic contract established by C6

- one canonical logical path source syntax exists.
- compiler/module/semantic/LSP/REPL agree on canonical module identities.
- expression `::` and declaration logical paths share namespace meaning without sharing inappropriate AST/site identities.
- module/package descriptor `.`, `::`, and final class-side `.` behavior remain distinct.
- old path/reference authorities cannot silently execute.

## Required evidence

1. cross-layer module fixture through parser -> module session -> semantic snapshot -> LSP definition/completion.
2. runtime fixture for direct qualified expression path.
3. negative searches for old active dotted logical paths.
4. negative searches for LSP/semantic duplicate path resolver logic.
5. final format/check/test/clippy gates.

## Checkpoint completion

- [ ] Tasks 25–27 complete
- [ ] all active source migrated
- [ ] cross-consumer identity tests pass
- [ ] negative gates clean/justified
- [ ] all deferred evidence executed or explicitly dispositioned
- [ ] final broad gates pass
- [ ] state file has no incident

### Task 25 — Final active-source migration

Purpose:
Remove stale logical path source spellings left outside C3's first-pass working set.

Risk:
- Semantic: LOW per edit; migration completeness HIGH
- Implementation fanout: repository-wide

Edit operations:

1. Search `.ph`, Rust fixture strings, test golden files, active docs, examples.
2. Classify each dot-containing occurrence as:
   - ordinary member/message syntax — leave unchanged;
   - filesystem filename/extension — leave unchanged;
   - decimal/version/text — leave unchanged;
   - logical module path — migrate to `::`;
   - historical archived syntax — retain with justification.
3. Re-run parser/module tests after batch migration rather than after each file.

Must not:
- global search/replace `.` -> `::`.

### Task 26 — Cross-consumer consistency and deletion gates

Purpose:
Prove one canonical fact reaches every consumer.

Risk:
- Semantic: HIGH
- Implementation fanout: cross-crate tests/search

Cross-consumer invariant:

```text
authored path segment
  -> ImportPath/static expression path product
  -> canonical ModuleId / linked export target
  -> SemanticTargetId
  -> source occurrence target
  -> LSP definition/completion identity
  -> runtime ModuleObject.id when executed
```

Where the final target is a declaration/binding rather than module, the corresponding canonical declaration/module-binding target must be used instead of forcing `ModuleId` equality.

Negative/deletion searches must include:

- old active dotted logical-path examples in current source/test/docs;
- `split('.')` or manual dotted joining in `phalcom-lsp/src/import_completion.rs` logical-path handling;
- new path-resolution helpers in LSP/semantic that duplicate `ModuleResolver` root/exposure policy;
- any compatibility fallback that accepts old dotted path source syntax in production;
- any runtime `::` implementation that simply calls `try_module_export_send`.

Expected retained hits must be documented, especially historical ADR/implementation snapshots and ordinary filesystem/member operations.

### Task 27 — Final delivery gates

Purpose:
Prove repository-wide compatibility after all migrations.

Risk:
- Semantic: LOW new logic; broad integration gate
- Implementation fanout: workspace

Run smallest-first before broad commands. Exact commands should be reconciled with repository toolchain configuration at implementation time.

Recommended final gates:

```bash
cargo +stable fmt --all -- --check
cargo +stable check --workspace --all-targets
cargo +stable test --workspace --all-targets
cargo +stable clippy --workspace --all-targets -- -D warnings
```

If the repository's established CI command differs, use the canonical repository command and record it.

Each command proves:

- `fmt` — delivery formatting only;
- workspace `check` — exhaustive caller/API migration compiles;
- workspace tests — broad cross-crate regression compatibility;
- clippy — no new lint debt at delivery threshold.

They do not replace the focused semantic evidence from C0–C5.

Suggested commit grouping for C6:

```text
C6.1 chore(syntax): finish active logical path source migration
C6.2 test(integration): lock compiler/LSP/runtime qualified path identity
C6.3 docs(spec): publish canonical :: module path surface
```

---

# 13. Testing matrix by semantic risk

| Risk | Ownership-layer evidence | Cross-layer evidence | Hostile case |
|---|---|---|---|
| Relative path depth | AST/parser + resolver | semantic target | climb-above-root rejected |
| Exposure | module resolver/linker | runtime + LSP completion | existing unexposed child rejected |
| Re-export identity | linker | semantic/runtime | whole-module target stays module identity |
| Incremental import reuse | module session | semantic publication | body-only edit does not relink broad graph |
| Static expression dependency | module session | semantic incremental | no-import direct path still invalidates on provider change |
| Module descriptor identity | runtime registry/materializer | expression execution | same `ModuleId` never rematerialized as second descriptor |
| `. / ::` separation | runtime/semantic | end-to-end program | reflective member and associated export with same name remain distinct |
| Editor navigation | semantic source index | LSP | each intermediate prefix targets its own module |

---

# 14. Repository-drift procedure

Before each checkpoint:

1. verify every primary file still exists;
2. search exact primary symbols named in the checkpoint;
3. inspect commits since `288da3f5da322dba60d0be9dfa4b52f5f1505d2f` touching the working set;
4. inspect effects of completed earlier checkpoints;
5. search for new consumers when an enum/API is changed.

The implementing agent may adapt mechanics to repository drift. It may not silently change these semantic decisions:

- `::` is associated lookup;
- logical path components use `::`;
- relative dots remain anchors;
- `expose` remains direct-child in this program;
- `phalcom-modules` owns logical path resolution;
- module/package descriptors are first-class runtime values;
- `.`, `::`, and class-side message sends stay distinct;
- static expression paths contribute canonical dependencies.

If current production architecture contradicts one of those requirements, mark the checkpoint `INCIDENT` and escalate with concrete code evidence.

---

# 15. Failure procedure

If required evidence fails, stop outward patching.

Record:

1. exact command;
2. failing test/check and key output;
3. direct path from fixture to failure;
4. one nearby passing comparator;
5. failure classification;
6. narrow allowed repair boundary.

Classify as:

```text
PRODUCT
FIXTURE
DEPENDENCY/PUBLICATION
BACKEND/HARNESS
BASELINE
PLAN DRIFT
```

Examples:

```text
explicit import path resolves; direct expression path dependency missing
    -> likely PRODUCT/DEPENDENCY in module-session path product

cold semantic target correct; incremental stale
    -> DEPENDENCY/PUBLICATION

compiler snapshot correct; LSP completion wrong
    -> adapter/presentation boundary, not resolver

static runtime path works; stored Module descriptor::child exposes private global
    -> runtime associated namespace PRODUCT defect
```

Rejected broad repairs:

- do not add LSP resolver fallback;
- do not weaken exposure checks;
- do not make all module globals public;
- do not force full-workspace rebuild to hide a missing dependency edge;
- do not synthesize fake canonical target for dynamic descriptor values;
- do not restore old dotted syntax because one fixture has not been migrated.

A checkpoint with failed required evidence is `INCIDENT`, not partially complete.

---

# 16. Implementation state-file requirements

Maintain a concise state file for continuous implementing agents. Suggested location should follow the repository's existing implementation-state convention for the program executing this spec.

After every checkpoint record:

```md
## Established invariants
- I-...

## Decisions
- D-...

## Changed anchors
- `path` — `symbol`

## Evidence ledger
| Checkpoint | Command/test | Result | Proves |
|---|---|---|---|

## Negative gates
- search -> result / justified retained hits

## Deferred gates
- command -> destination checkpoint

## Active incident
None.

## Next resume action
Begin C<N> Task <N>.
```

Do not record raw scratchpad reasoning. Record facts, decisions, code anchors, and evidence.

---

# 17. Checkpoint completion report template

At each boundary the implementer should be able to report:

```text
Checkpoint C<N> COMPLETE

Established:
    <one dominant semantic contract>

Changed:
    <path> — <symbols/responsibility>

Evidence:
    <test/command> — PASS — proves ...

Hostile cases:
    <case> — PASS

Negative gates:
    <search> — expected result

Deferred:
    <gate> -> C<M>/Final

Unexpected findings:
    none / concise finding

Next:
    C<N+1> — ...
```

---

# 18. Final checkpoint evidence summary

The implementing agent must fill this table with actual results. Do not pre-mark a checkpoint complete.

| Checkpoint | Semantic contract | Required evidence | Status |
|---|---|---|---|
| C0 | canonical declaration path grammar | parser/AST + negative syntax | NOT RUN |
| C1 | canonical module resolution/linking identity | resolver/linker/fingerprint | NOT RUN |
| C2 | persistent module/semantic incrementality | module-session + cold/incremental parity | NOT RUN |
| C3 | LSP/REPL/current-source surface parity | completion/REPL/migration gates | NOT RUN |
| C4 | canonical dependency-aware expression paths | module + semantic incremental evidence | NOT RUN |
| C5 | runtime descriptor associated lookup | core runtime parity/hostile cases | NOT RUN |
| C6 | repository delivery closure | cross-consumer + broad gates | NOT RUN |

---

# 19. Final negative/deletion gates

At delivery, require explicit evidence for each item.

## 19.1 Old logical component separator

Search active Phalcom source/test fixtures/current docs for dotted logical paths. Expected result: zero active canonical-syntax hits. Ordinary member access, filesystem paths, decimal text, and intentionally historical docs are not failures.

## 19.2 Duplicate path resolvers

Search `phalcom-lsp` and `phalcom-semantic` for root/project/exposure resolution logic introduced by this patch. Expected result: no independent implementation duplicating `phalcom-modules::ModuleResolver`/module query products.

## 19.3 Production string-keyed semantic resolution

Search for production lookups using `ImportPath::to_string()` or raw rendered expression path as canonical identity. Expected result: presentation/compatibility-only uses are justified; canonical production resolution is identity-keyed.

## 19.4 Fake expression import sites

Search for expression paths inserted into `import_products` under synthetic `ImportSiteId`s. Expected result: zero unless the architecture was deliberately generalized and the site type renamed/redefined consistently across all consumers after explicit escalation.

## 19.5 Message-send fallback for module `::`

Search new runtime associated lookup for calls into `try_module_export_send` / ordinary `invoke` as the namespace lookup definition. Expected result: zero semantic delegation that collapses `.` and `::`.

## 19.6 Nested expose syntax

Search active parser tests/current docs for `expose .a::b`. Expected result: zero positive cases.

---

# 20. Deferred-evidence audit

Before release completion:

```text
No deferred test/check may remain without one of:
- executed successfully;
- explicitly removed from scope with justification;
- recorded as a known release blocker.
```

Do not report final completion while a C0–C6 required gate is merely deferred.

---

# 21. Known scope exclusions

This implementation does not include:

- a redesign of callable family typing;
- new class associated/static function declarations;
- nested `expose` paths;
- a new package descriptor object hierarchy;
- general compile-time evaluation of arbitrary descriptor-valued locals;
- a filesystem/package resolver in the LSP;
- replacement of current project/package/module identity types;
- broad runtime module loading redesign unrelated to executing canonical associated lookup;
- formatting-language changes beyond rendering the new canonical logical source spelling;
- unrelated module architecture cleanup discovered during implementation.

If one of these becomes necessary to satisfy a required invariant, stop and escalate rather than silently expanding scope.

---

# 22. Release-complete criteria

The program is complete only when all of the following are true:

- [ ] C0 through C6 are `COMPLETE`;
- [ ] all focused semantic evidence passes;
- [ ] all hostile cases pass;
- [ ] absolute logical path components use `::`;
- [ ] relative logical paths preserve leading dot depth and use `::` after the anchor;
- [ ] `expose .child` remains direct-child only;
- [ ] `phalcom-modules` remains the single canonical path-resolution authority;
- [ ] persistent module products and semantic snapshots retain exact cold/incremental identity parity;
- [ ] LSP and REPL emit/understand the canonical syntax without independent resolution;
- [ ] direct expression-level module/package paths contribute canonical dependencies;
- [ ] module/package descriptor values support runtime `::` lookup;
- [ ] runtime `.` and `::` surfaces are observably distinct;
- [ ] class-side calls after path traversal remain ordinary `.` message sends;
- [ ] unexposed/private module names cannot be reached through `::`;
- [ ] module-valued exports preserve canonical descriptor object identity;
- [ ] old active syntax/mechanism negative gates are clean or every retained occurrence is justified;
- [ ] all deferred evidence is dispositioned;
- [ ] final format/check/test/clippy gates pass;
- [ ] state file contains no unresolved incident.

---

# 23. Resume order for a continuous implementing agent

Execute strictly in this order unless a checkpoint explicitly records independent work:

```text
C0  canonical declaration path grammar
 ↓
C1  canonical module resolver/linker identity
 ↓
C2  persistent module + semantic incrementality
 ↓
C3  LSP / REPL / active-source migration
 ↓
C4  expression-level static module/package associated lookup + dependencies
 ↓
C5  runtime descriptor associated lookup + lowering
 ↓
C6  delivery closure
```

Do not begin C4 by adding runtime behavior first. The dependency/source-of-truth layer must exist before runtime sugar can make a statically invisible dependency appear to work.

Do not begin C3 by teaching the LSP a new resolver. C1/C2 compiler-owned products are the authority it must consume.

Do not collapse C4 and C5 into one “make `universe::...` work” patch. Static canonical dependency identity and runtime descriptor execution are separate semantic risks and require separate evidence boundaries.
