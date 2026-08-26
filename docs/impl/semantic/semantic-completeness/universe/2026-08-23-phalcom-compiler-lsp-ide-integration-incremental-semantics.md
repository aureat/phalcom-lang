# Phalcom Compiler, LSP, and IDE Integration — Incremental Formal Semantics Repository-Grounded Implementation Specification & Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to execute this plan task-by-task. Use `superpowers:test-driven-development` for each implementation slice, `superpowers:systematic-debugging` for failures, and `superpowers:verification-before-completion` before declaring any task finished.

**Goal:** Complete the compiler/LSP/VS Code semantic integration so Phalcom has one project/module authority, one canonical formal type-analysis authority, fast incremental workspace analysis, source-accurate diagnostics, formal-type-driven editor intelligence, module-aware completion/navigation, navigable builtin/core source, reliable analysis status and observability, and measurable performance guarantees.

**Architecture:** `phalcom-modules` remains the sole authority for project identity, logical modules, import resolution, package exposure, interfaces, exports, linking, and module dependency topology. `phalcom-semantic` remains the sole authority for formal type/kind semantics and evolves its already-present `db::SemanticDb` into the persistent incremental formal semantic database. `phalcom-lsp` remains a background-worker/query/presentation adapter: it owns no competing module resolver and no competing formal type system. Its existing advisory `ValueShape` engine remains optional editor evidence, never language truth. `phalcom-core` consumes analyzed snapshots and rejects invalid programs at the code-generation boundary. `tools/vsphalcom` presents status, logs, diagnostics, navigation and virtual builtin source without duplicating compiler logic.

**Tech stack:** Rust workspace; `phalcom-ast`; `phalcom-modules`; `phalcom-semantic`; `phalcom-core`; `phalcom-lsp`; `phalcom-native-*`; `phalcom-diagnostics`; `tower-lsp`; Tokio; immutable `Arc` publications; TypeScript; `vscode-languageclient`; VS Code Extension Host tests; existing `graphify` workflow.

**Repository baseline inspected:** `main` at `e37b38cf953f11de40bb868b164bb0f3bd383d91` (`fix(lsp): keep bundled core source access warning-free`), 2026-08-23.

**Active prerequisite:** user-supplied **Wave 4 — Formal Flow (F1–F5)** is being implemented concurrently. This plan treats Wave 4 as a dependency and does not duplicate or overwrite its formal CFG/predicate/join/widening/mutation/iteration work.

**Recommended repository path for this plan:**

```text
docs/superpowers/plans/2026-08-23-phalcom-compiler-lsp-ide-integration-incremental-semantics.md
```

---

# 1. Executive summary

Phalcom already has most of the difficult semantic pieces, but they are not yet assembled into one coherent compiler/IDE pipeline.

The current repository has:

- a real project/module system in `phalcom-modules`, including project manifests, dependency roots, relative imports, `std`/`universe`, package `expose` rules, interface extraction, linked exports, and module graphs;
- a canonical type system in `phalcom-semantic`, including `TypeStore`, `TypeId`, `KindId`, two-axis denotation, type evidence, relation solving, declaration surfaces, callable signatures, formal analysis product types, and whole-workspace semantic analysis;
- an embryonic incremental formal semantic database in `phalcom-semantic/src/db/` with revisions, typed query keys, reverse dependencies, scheduling, fingerprints, cancellation/budget concepts, and metrics;
- a mature asynchronous LSP worker with latest-wins edit coalescing, progressive scanning, immutable advisory publications, source-revision guards, body-local advisory invalidation, and query-path no-I/O goals;
- an LSP static bridge that already runs `phalcom_semantic::analyze_workspace` and publishes real type mismatch diagnostics;
- a VS Code extension with status UI, an output channel, language-client lifecycle, inlay hints, hover, navigation and semantic tokens.

The remaining problem is architectural duplication and incomplete bridges.

Today, the LSP has two semantic worlds:

```text
ADVISORY / EDITOR FLOW                    FORMAL / LANGUAGE TYPING
======================                    ========================

phalcom-lsp::SemanticEngine               phalcom-semantic
ValueShape / InferredValue                TypeId / TypeKnowledge
incremental callable worklist             whole-workspace rebuild
fast immutable snapshots                  fresh TypeStore each rebuild
completion/hover/inlay queries             diagnostics only in LSP
URI-oriented module graph                 canonical linked module program
```

The target is not to merge these two domains into one type algebra. The target is to make them cooperate under clear authority rules:

```text
                         SOURCE / WORKSPACE
                                │
                                ▼
                    phalcom-modules session
              project + modules + interfaces + links
                                │
               ┌────────────────┴────────────────┐
               │                                 │
               ▼                                 ▼
      phalcom-semantic DB                 advisory LSP engine
       formal language truth              runtime-shape evidence
               │                                 │
               └────────────────┬────────────────┘
                                ▼
                       immutable LSP snapshot
                                │
                   ┌────────────┼────────────┐
                   ▼            ▼            ▼
              diagnostics    completion    hover/inlays
                   │            │            │
                   └────────────┴────────────┘
                                ▼
                            VS Code
```

The most important implementation decisions are:

1. **Wave 4 is the formal flow substrate.** Do not build a second CFG or path-sensitive engine in the LSP.
2. **`phalcom-modules` is the only module-resolution authority.** Remove semantic dependence on URI/path guessing in the advisory LSP graph.
3. **The existing `phalcom-semantic::db::SemanticDb` becomes the incremental formal database.** Do not create a parallel incremental type database.
4. **A formal semantic workspace has a stable type-store identity across revisions.** Reusable formal products cannot safely cross fresh `TypeStore`s because `TypeId` is store-local.
5. **Cold startup must not deep-flow-analyze the entire core universe.** Load the core declaration/member/native surface first; analyze core bodies lazily or when source editing requires it.
6. **Formal editor presentation uses formal facts first.** Explicit annotations suppress inferred hints; formal `Known(T)` may produce type hints; advisory runtime shapes are visually and semantically distinct.
7. **Analysis errors never silently erase the last valid formal snapshot.** Failures are observable and last-known-good data remains queryable.
8. **Every analysis batch has a valid terminal status.** No edit-only batch may remain stuck in `Publishing`/“Updating”.
9. **Compiler analysis and compiler validity are separate.** `ProgramAnalyzer` returns an analyzed snapshot even when it contains semantic errors; `ProgramCompiler` rejects invalid snapshots before code generation.
10. **All user-facing diagnostics retain source ownership and use structured renderers.** A user file error may not accidentally claim `universe.core` ownership.

---

# 2. Concurrency gate: relationship to Wave 4

## 2.1 Wave 4 is an active dependency

The active Wave 4 implementation modifies at least:

```text
phalcom-semantic/src/checker/flow/graph.rs
phalcom-semantic/src/checker/flow/predicate.rs
phalcom-semantic/src/checker/flow/state.rs
phalcom-semantic/src/checker/flow/transfer.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/tests/spec04_5_flow_graph.rs
```

It establishes:

- formal CFG construction;
- path predicates;
- type refinement;
- conservative joins;
- loop widening;
- mutation/opaque-call invalidation;
- protocol-only iteration typing.

This plan MUST NOT independently implement those semantics.

## 2.2 Safe work before Wave 4 lands

The following tasks can be implemented while Wave 4 is still in flight because they do not need its final internal APIs:

- analysis status terminal-state repair;
- structured analysis logging and worker failure visibility;
- compiler analyzer/compiler responsibility repair;
- CLI rich diagnostic rendering;
- source-module ownership repair outside files actively modified by Wave 4;
- explicit-annotation inlay suppression;
- canonical module workspace/query APIs in `phalcom-modules`;
- import-path completion context classification;
- core source navigation/virtual source protocol;
- VS Code output/status/virtual-document plumbing;
- performance counters and harness improvements.

## 2.3 Work that waits for Wave 4 merge

Do not begin these implementation slices until Wave 4 has merged and its focused tests are green:

- formal `CallableAnalysis` population from flow products;
- formal expression/binding presentation indexes;
- formal incremental callable-body recomputation;
- formal-flow-based type hover/inlay results;
- any edits to Wave 4-owned `checker/flow/*` or protocol-only `Statement::For` behavior.

## 2.4 Post-Wave-4 re-grounding gate

Before Task 7 or later formal-flow tasks, run:

```bash
git rev-parse HEAD
git status --short
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test spec04_5_flow_graph
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
```

Then inspect the merged public APIs of:

```text
phalcom-semantic/src/checker/flow/
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/mod.rs
```

The implementation agent may adapt names/signatures to the landed Wave 4 API, but MUST preserve the architectural contracts in this plan.

---

# 3. Current repository diagnosis

## 3.1 Compiler analysis currently conflates analysis completion with program validity

`phalcom-core/src/modules/compile.rs` defines an excellent `AnalyzedProgram` product containing the linked program, formal semantic snapshot and parsed source units. However, `ProgramAnalyzer::analyze_entry_selection` currently checks `analysis.snapshot.has_errors()` and returns `ProgramCompileError::Semantic` before returning `AnalyzedProgram` in multiple paths.

This creates three problems:

1. `phalcom check` loses the source map it needs for rich semantic rendering.
2. callers cannot inspect a successful semantic analysis that concluded the source is invalid;
3. `ProgramCompiler::compile_analyzed` contains no independent semantic-error gate because it assumes analysis already rejected invalid programs.

The correct invariant is:

```text
analysis succeeded
    !=
program is valid
```

A type mismatch is a successful semantic conclusion, not an analyzer infrastructure failure.

## 3.2 Semantic diagnostics contain rich source data but source ownership is unsafe

`phalcom-semantic/src/diagnostic.rs` already has:

```rust
SemanticDiagnostic {
    code,
    severity,
    message,
    primary: SemanticSourceSpan { module, range },
    primary_range,
    labels,
    notes,
    helps,
    explanations,
}
```

and a proper snippet renderer.

The unsafe seam is the convenience constructor:

```rust
SemanticDiagnostic::error(...)
```

which defaults the diagnostic module to `ModuleId::core()`.

Current checker paths such as binding initializer mismatch and expression mismatch use this constructor, so user-source errors can be grouped under the correct source module while their internal `primary.module` incorrectly says core.

This is not merely cosmetic. It can corrupt:

- LSP related-information URIs;
- cross-module diagnostics;
- definition/explanation ownership;
- diagnostic grouping/refactoring;
- future fixes/code actions.

## 3.3 LSP parser diagnostics are already strong

The LSP uses the recovering `phalcom_ast::parse()` path and publishes all recovered syntax errors. Syntax and formal semantic diagnostics can coexist. Revision/generation/text guards prevent stale type errors from being applied to newer source.

This behavior must be preserved.

## 3.4 Formal diagnostics are integrated, but formal semantic facts are not

The LSP advisory snapshot contains:

```rust
pub static_snapshot: Option<Arc<phalcom_semantic::SemanticSnapshot>>
```

but `combined_diagnostics_for` is essentially the only formal consumer.

Inlay hints, hover and most completion inference consume advisory `ValueShape` / `InferredValue` instead.

This is why an advisory inlay may render a type-looking `CellNum` even if the canonical checker has not accepted the declaration.

## 3.5 Formal analysis product types exist but are not published

`phalcom-semantic/src/checker/analysis.rs` already defines:

```text
ExpressionAnalysis
BindingState
CallableAnalysis
CallableAnalysisStatus
```

including type knowledge, denotation, explanations, call resolution, dependencies and diagnostics.

Yet `SemanticSnapshot::new` currently initializes:

```rust
callable_analyses: Arc::new(HashMap::new())
```

so the LSP cannot query formal expression/binding facts even though the model exists.

Wave 4 provides the missing flow substrate that should populate these products.

## 3.6 The canonical formal workspace checker is non-incremental

`phalcom-semantic::analyze_workspace` currently:

- creates a fresh `TypeStore`;
- bootstraps the universe;
- predeclares every source declaration;
- constructs a fresh linked type resolver;
- rebuilds hierarchy and semantic graph;
- rebuilds dispatch/surfaces/signatures;
- checks every module body;
- freezes a completely new snapshot.

This happens whenever the LSP static bridge refreshes.

Because `TypeId` is store-local and snapshot identity explicitly includes `TypeStoreId`, formal products cannot simply be cached in the LSP and reused across unrelated fresh stores.

The incremental fix therefore belongs in `phalcom-semantic`, not only in `phalcom-lsp`.

## 3.7 The formal incremental database scaffolding already exists

`phalcom-semantic/src/db/` already contains:

```text
SemanticDb
SemanticRevision
QueryKey
ProductFingerprint
QueryState
DependencyRecorder
DependencyIndex
QueryScheduler
QueryMetrics
CancellationToken / QueryBudget
```

`QueryKey` already includes products such as:

```text
ParsedModule
UnlinkedInterface
LinkedInterface
DeclarationShell
SemanticComponent
DeclarationSurface
CallableBody
CallableEffects
CallableControl
CallableTermination
CallableContracts
VerificationConditions
ModuleDiagnostics
ModuleMetadata
```

and the dependency index already supports deterministic reverse invalidation closure.

This must become the formal incremental substrate. Do not create a competing database.

## 3.8 LSP advisory analysis is genuinely incremental

The advisory `phalcom-lsp::SemanticEngine` already classifies changes as:

```text
BodyOnly
ImportSurface
DeclarationSurface
FileAddedRemoved
CoreSurface
```

and body-only edits seed exact changed callables. Its callable solver propagates through dependents only when summaries/evidence change.

That architecture is worth preserving. It is not, however, a substitute for formal type checking.

## 3.9 Cold start performs deep core advisory analysis before workspace scanning

The current analysis worker enters `SelectingCore` and calls:

```rust
engine.update_core(FileRevision(1), &program)
```

before normal progressive workspace scanning proceeds.

The reported live trace of approximately 21–22 seconds, about 3,180 callables and 909 flow-analysis operations is therefore consistent with the implementation: the full core semantic surface is being treated as ordinary deep advisory source before the workspace is even indexed.

This is unnecessary for ordinary editing. Completion/hover need core declarations/member/native metadata immediately; they do not require every core body to be flow-solved at startup.

## 3.10 LSP formal workspace refresh rebuilds project/module infrastructure repeatedly

`analysis_service.rs::run_static_workspace_analysis` currently reconstructs:

- project roots;
- `ProjectUniverse`;
- loaded projects;
- source provider mappings;
- interfaces;
- `ModuleResolver`;
- import resolution closure;
- `ModuleLinker`;
- formal semantic workspace analysis.

Several failure branches use `continue`/`None`, and a failed refresh can cause `set_static_analysis(None)` to clear the previously valid static snapshot.

This is both a performance bottleneck and a failure-visibility bug.

## 3.11 The LSP has a second, weaker module-resolution model

`phalcom-lsp/src/semantic/module_graph.rs` contains both:

- a lightweight URI/path-based `ModuleGraph::update(...)`;
- a `SharedModuleResolver` adapter over canonical `phalcom_modules::ModuleResolver`.

Production advisory updates currently use the lightweight path.

This permits compiler/LSP disagreement over project roots, package exposure and cross-project imports. The canonical resolver must become the only authority; the advisory graph should ingest resolved edges rather than resolve them itself.

## 3.12 Import-path completion is absent

`phalcom-lsp/src/completion.rs::target_at_offset` recognizes a dot only as receiver/member syntax. It does not classify:

```phalcom
import geometry.|
import .|
import ..models.|
from geometry.point import (|)
```

as module/import completion contexts.

The LSP advertises `.` as a completion trigger, but the handler routes it through receiver/member semantics.

## 3.13 Module aliases are not first-class completion receivers

The advisory value domain has `ValueShape::Module`, but receiver completion expects class/instance alternatives. Whole-module imports should resolve to module-export completion, not class-member completion.

## 3.14 Export visibility is not authoritative in editor inference

Canonical compiler linking has `LinkedModuleInterface::exports`, while advisory name/class resolution can approximate by checking what declarations exist in an imported target module.

The IDE must never offer or resolve a name that the canonical linked interface says is private/unexported.

## 3.15 Import/module navigation lacks semantic occurrences

The LSP occurrence model has Binding, Class, Callable, Field, Member and Operator targets, but no first-class Module target and no preamble import-path occurrence indexing.

Consequently module path segments cannot participate cleanly in definition/references.

## 3.16 Core source is selected and parsed, but definition intentionally returns `None`

`Backend::member_definition_location` and `class_definition_location` explicitly reject `CORE_MODULE_URI`.

At the same time, core source selection already knows the physical URI when a configured/workspace source is available, and `build_core_surface` deliberately preserves source ranges for source-declared native members.

Core navigation is therefore blocked by presentation/identity mapping, not by lack of data.

## 3.17 Bundled combined-core ranges are not a safe navigation authority

The advisory bundled core parser combines statements parsed independently from many builtin modules. Per-module source ranges can overlap when combined. This combined view is useful for a semantic surface but must not be the source-of-truth for definition locations.

Builtin navigation must use module-specific canonical builtin source from `phalcom-modules::BuiltinProjectSourceProvider`, or a physical core file when available.

## 3.18 Inlay hints ignore explicit annotations

AST nodes already carry annotations for:

```text
LetBinding.annotation
FieldDef.annotation
ParameterDef.annotation
MethodDef.return_annotation
GetterDef.return_annotation
SetterDef.return_annotation
IndexMethodDef.return_annotation
```

Current inlay rendering does not suppress advisory hints when these annotations are present. This creates duplicated source such as:

```phalcom
dividedBy(_ other : CellNum : CellNum)
```

The first fix is policy correctness; formal integration comes afterward.

## 3.19 Analysis status can remain stuck in `Publishing`

Scan completion transitions to `Ready`, but edit-only batch publication transitions to `Publishing` and does not necessarily emit a subsequent terminal status.

The VS Code extension correctly renders `Publishing` as “Updating,” so the UI can remain stuck despite completed analysis.

## 3.20 Worker failures and stale batches are under-observable

The worker has `AnalysisEvent::Error` and `StaleBatchDiscarded`, and `StatusTracker` has `set_error`, but:

- many production failure paths do not emit an error event;
- backend fabricates an error status with session/sequence zero rather than preserving worker status identity;
- stale batch events are ignored;
- the Output channel receives little structured analysis information;
- timing logs are gated behind `PHALCOM_LSP_PERF=1`.

---

# 4. Normative architecture and decision register

## DEC-INTEG-001 — `phalcom-modules` is the only module-resolution authority

The LSP may retain an editor-oriented module graph for invalidation/query efficiency, but it must be populated from canonical module resolution results.

No production semantic path may infer logical import meaning from URI string manipulation when a project-aware canonical `ModuleId` is available.

## DEC-INTEG-002 — `phalcom-semantic` is the only formal type authority

`TypeId`, `KindId`, `TypeKnowledge`, assignability, formal flow, declaration typing and formal diagnostics live in `phalcom-semantic`.

The LSP must never translate `ValueShape` into a formal `TypeId` and treat the result as compiler truth.

## DEC-INTEG-003 — existing `phalcom-semantic::db::SemanticDb` becomes the persistent formal database

Extend the current database rather than introducing another query database.

The current revision, dependency, key, scheduler and metric concepts remain.

## DEC-INTEG-004 — formal workspace sessions preserve one type-store identity

Within one workspace semantic session:

```text
WorkspaceId        stable
TypeStoreId        stable
SemanticRevision   monotonic
SnapshotId         changes per revision
```

Existing `TypeId`s remain valid as long as the corresponding type form remains in the append-only store.

A formal incremental implementation that creates a new independent `TypeStore` per edit is incomplete.

## DEC-INTEG-005 — snapshots are immutable; mutable semantic state is worker/compiler-session owned

LSP request paths receive immutable snapshots only.

No request may acquire a mutable type store, linker, resolver or semantic database lock.

## DEC-INTEG-006 — analyzer success is distinct from semantic validity

`ProgramAnalyzer` returns `AnalyzedProgram` if discovery/parsing/linking/semantic execution completed, even when the snapshot contains ordinary semantic errors.

`ProgramCompiler::compile_analyzed` rejects snapshots whose diagnostics contain errors before producing bytecode/runtime artifacts.

## DEC-INTEG-007 — semantic diagnostic source ownership is explicit

Production diagnostics must be constructed with an explicit source module.

Remove or constrain constructors that silently default to `ModuleId::core()`.

## DEC-INTEG-008 — formal facts take priority in editor type presentation

Presentation order:

```text
explicit source annotation
    ↓ suppress inferred type hint
formal Known(T)
    ↓ render canonical type hint
formal Unknown / Dynamic / unavailable
    ↓ optional advisory runtime-shape hint
```

Formal and advisory hints may not use indistinguishable syntax.

Recommended presentation:

```text
formal:   : CellNum
advisory: ≈ CellNum
```

The advisory tooltip must explicitly say it is runtime-shape/editor evidence, not a Phalcom type judgment.

## DEC-INTEG-009 — core startup is surface-first

Normal startup ingests:

- core classes;
- inheritance/surface metadata;
- source declarations;
- native declarations;
- selectors/signatures where available;
- documentation/source identity.

It does not deep-flow solve every core callable before workspace scanning.

Core body flow analysis is triggered only when:

- the user opens/edits core source;
- a formal query actually requires source-body information not present in trusted declaration/native metadata;
- explicit workspace mode requests full deep analysis and the work is scheduled after interactive readiness.

## DEC-INTEG-010 — project roots are recognized before generic scanning

When an LSP workspace root contains `project.toml`, load that manifest once and derive semantic source roots from `ProjectUniverse` immediately.

Do not call `discover_owning_project` independently for every file in the steady state.

Generic filesystem scanning remains a fallback for non-project/multi-root folders.

## DEC-INTEG-011 — module completion is compiler-valid by construction

Hard invariant:

> Every module/import candidate offered by completion must be accepted by canonical module resolution under the same workspace snapshot.

Private/unexposed children must never appear.

## DEC-INTEG-012 — builtin/core definitions are navigable

Definition resolution returns:

1. a physical `file://` URI when canonical source exists on disk;
2. otherwise a read-only virtual `phalcom://...` URI backed by the canonical builtin source provider.

Do not use the collapsed combined-core source as the range authority for virtual module definitions.

## DEC-INTEG-013 — failed formal updates preserve last-known-good formal snapshots

A failed update may publish failure diagnostics/status/logs, but it must not replace a previously valid formal snapshot with `None`.

## DEC-INTEG-014 — status lifecycle is terminal and monotonic

Every non-stale batch must finish in one of:

```text
Ready
Error
Indexing / Analyzing for already-known continuing work
```

No completed batch remains indefinitely in `Publishing`.

Session/sequence identity is owned by `StatusTracker`; the backend never fabricates `session=0, sequence=0` statuses.

## DEC-INTEG-015 — errors and performance are observable without environment variables

`PHALCOM_LSP_PERF=1` remains useful for detailed local timing, but normal users/developers can enable structured analysis logging through extension/server configuration.

## DEC-INTEG-016 — performance correctness is enforced structurally in CI

Wall-clock benchmarks remain reference-machine measurements because shared CI timing is noisy.

CI enforces work-size invariants using counters/fingerprints:

- no whole core solve on normal startup;
- no project-universe/module-resolver reconstruction on body-only edits;
- no unrelated formal module rechecks;
- no query-path disk reads/canonicalizations;
- no publication of stale revisions;
- expected structural reuse occurs.

## DEC-INTEG-017 — distinguish the two existing `SemanticDb` types explicitly

There are two existing types with the same short name but different ownership:

```text
phalcom_semantic::db::SemanticDb
    compiler-owned formal incremental query database

phalcom_lsp::semantic::SemanticDb
    thread-safe immutable-publication handle for LSP requests
```

At LSP boundaries import the formal database as:

```rust
use phalcom_semantic::db::SemanticDb as FormalSemanticDb;
```

Do not merge these responsibilities. The LSP publication database remains a snapshot holder; `FormalSemanticDb` remains worker/session-owned mutable formal analysis state.

---

# 5. Target system architecture

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                               VS CODE                                       │
│                                                                             │
│  diagnostics · completion · hover · inlay · definition · references         │
│  analysis status · analysis log · virtual builtin documents                 │
└──────────────────────────────────┬──────────────────────────────────────────┘
                                   │ LSP
                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              phalcom-lsp                                    │
│                                                                             │
│  Backend / RequestContext                                                   │
│      │                                                                      │
│      └── pins ONE immutable PublishedWorkspaceSnapshot                      │
│              │                                                              │
│              ├── module_index: Arc<ModuleWorkspaceSnapshot>                 │
│              ├── formal: Arc<phalcom_semantic::SemanticSnapshot>            │
│              └── advisory: Arc<AdvisorySemanticSnapshot>                    │
│                                                                             │
│  AnalysisService worker — ONLY mutable LSP analysis owner                   │
│      │                                                                      │
│      ├── ModuleWorkspaceSession                                             │
│      ├── phalcom_semantic::SemanticWorkspaceSession / SemanticDb            │
│      └── advisory SemanticEngine                                            │
└──────────────────────────────────┬──────────────────────────────────────────┘
                                   │
              ┌────────────────────┴─────────────────────┐
              ▼                                          ▼
┌──────────────────────────────┐           ┌──────────────────────────────────┐
│       phalcom-modules        │           │         phalcom-semantic         │
│                              │           │                                  │
│ ProjectUniverse              │           │ SemanticDb                       │
│ source overlay/provider      │           │ stable TypeStore identity        │
│ ModuleResolver               │           │ declaration/type products        │
│ interfaces                   │           │ Wave-4 CFG/flow products          │
│ linked exports               │           │ callable analyses                │
│ exposure graph               │           │ diagnostics/explanations         │
│ reverse import dependencies  │           │ presentation indexes             │
└──────────────────────────────┘           └──────────────────────────────────┘
              │                                          │
              └────────────────────┬─────────────────────┘
                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              phalcom-core                                   │
│                                                                             │
│ ProgramAnalyzer -> AnalyzedProgram (may contain semantic errors)            │
│ ProgramCompiler -> rejects semantic errors -> compiled artifacts / VM       │
│ CLI check/run -> rich shared diagnostics                                    │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 5.1 File and ownership map

### `phalcom-modules` — canonical project/module authority

Create:

```text
phalcom-modules/src/workspace.rs
phalcom-modules/src/query.rs
phalcom-modules/tests/workspace_session.rs
phalcom-modules/tests/module_query.rs
phalcom-modules/tests/performance_structure.rs
```

Modify:

```text
phalcom-modules/src/lib.rs
phalcom-modules/src/resolver.rs
phalcom-modules/src/source.rs
phalcom-modules/src/interface.rs
phalcom-modules/src/linker.rs
```

### `phalcom-semantic` — canonical formal type/flow authority

Create:

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/presentation.rs
phalcom-semantic/tests/callable_analysis.rs
phalcom-semantic/tests/presentation.rs
phalcom-semantic/tests/semantic_db_incremental.rs
phalcom-semantic/tests/type_store_revisions.rs
phalcom-semantic/tests/incremental_workspace.rs
phalcom-semantic/tests/performance_structure.rs
```

Modify:

```text
phalcom-semantic/src/lib.rs
phalcom-semantic/src/workspace.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/src/invalidation.rs
phalcom-semantic/src/db/mod.rs
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/state.rs
phalcom-semantic/src/db/dependency.rs
phalcom-semantic/src/db/metrics.rs
phalcom-semantic/src/db/scheduler.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/declaration.rs
```

After the Wave 4 merge, modify `checker/flow/*` only where publication/incremental integration requires consuming the landed flow products; do not redesign F1–F5.

### `phalcom-core` — analysis/compilation boundary and CLI

Modify:

```text
phalcom-core/src/modules/compile.rs
phalcom-core/bin/phalcom/cli.rs
phalcom-core/tests/semantic_analysis.rs
phalcom-core/tests/integration.rs
```

### `phalcom-lsp` — worker, immutable query bridge and editor adapters

Create:

```text
phalcom-lsp/src/analysis_log.rs
phalcom-lsp/src/import_completion.rs
phalcom-lsp/src/virtual_source.rs
phalcom-lsp/tests/analysis_logging.rs
phalcom-lsp/tests/core_startup.rs
phalcom-lsp/tests/formal_type_presentation.rs
phalcom-lsp/tests/module_workspace_bridge.rs
phalcom-lsp/tests/module_completion.rs
phalcom-lsp/tests/module_navigation.rs
phalcom-lsp/tests/core_navigation.rs
phalcom-lsp/tests/formal_incremental.rs
phalcom-lsp/tests/project_startup.rs
phalcom-lsp/tests/module_diagnostics.rs
phalcom-lsp/tests/compiler_parity.rs
```

Modify:

```text
phalcom-lsp/src/analysis_service.rs
phalcom-lsp/src/analysis_status.rs
phalcom-lsp/src/backend.rs
phalcom-lsp/src/completion.rs
phalcom-lsp/src/diagnostics.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/inlay_hints.rs
phalcom-lsp/src/perf.rs
phalcom-lsp/src/request_context.rs
phalcom-lsp/src/workspace_scan.rs
phalcom-lsp/src/semantic/core_source.rs
phalcom-lsp/src/semantic/engine.rs
phalcom-lsp/src/semantic/invalidation.rs
phalcom-lsp/src/semantic/module_graph.rs
phalcom-lsp/src/semantic/occurrence.rs
phalcom-lsp/src/semantic/snapshot.rs
phalcom-lsp/tests/analysis_status.rs
phalcom-lsp/tests/performance.rs
phalcom-lsp/tests/stage6_inlay_hints.rs
phalcom-lsp/tests/stage7_static_diagnostics.rs
phalcom-lsp/tests/integration.rs
```

### `tools/vsphalcom` — VS Code presentation only

Create:

```text
tools/vsphalcom/src/test/suite/analysisLog.test.ts
```

Modify:

```text
tools/vsphalcom/src/analysisStatus.ts
tools/vsphalcom/src/extension.ts
tools/vsphalcom/package.json
tools/vsphalcom/src/test/suite/analysisStatus.test.ts
tools/vsphalcom/src/test/suite/lsp.e2e.test.ts
```

The extension does not parse Phalcom imports/types or run compiler logic.

---

# 6. Data ownership and lifecycle

## 6.1 `phalcom-modules`: persistent module workspace

Create:

```text
phalcom-modules/src/workspace.rs
```

with an owning mutable session and immutable publication.

Target public interfaces:

```rust
pub struct ModuleWorkspaceSession {
    universe: Arc<ProjectUniverse>,
    root_projects: Vec<ResolvedProjectId>,
    // worker/session-owned source overlay and caches
    // canonical module/interface/link state
}

pub struct ModuleWorkspaceSnapshot {
    pub universe: Arc<ProjectUniverse>,
    pub modules: Arc<BTreeMap<ModuleId, Arc<ParsedModuleUnit>>>,
    pub interfaces: Arc<BTreeMap<ModuleId, Arc<UnlinkedModuleInterface>>>,
    pub linked: Arc<LinkedProgram>,
    pub sources: Arc<BTreeMap<ModuleId, Arc<ParsedModuleUnit>>>,
    pub source_locations: Arc<BTreeMap<ModuleId, SourceLocation>>,
    pub reverse_imports: Arc<BTreeMap<ModuleId, Arc<[ModuleId]>>>,
}

pub struct ModuleWorkspaceUpdate {
    pub snapshot: Arc<ModuleWorkspaceSnapshot>,
    pub delta: ModuleWorkspaceDelta,
    pub diagnostics: Arc<[ModuleWorkspaceDiagnostic]>,
    pub stats: ModuleWorkspaceStats,
}
```

`ModuleWorkspaceDelta` must distinguish at least:

```rust
pub enum ModuleChangeKind {
    BodyOnly,
    Interface,
    Added,
    Removed,
}

pub struct ModuleWorkspaceDelta {
    pub changed: BTreeMap<ModuleId, ModuleChangeKind>,
    pub interface_dependents: BTreeSet<ModuleId>,
    pub runtime_dependents: BTreeSet<ModuleId>,
    pub relinked: BTreeSet<ModuleId>,
}
```

Refactor the two private `ModuleResolver` caches into a reusable `ModuleResolutionCache` owned by `ModuleWorkspaceSession`:

```rust
pub struct ModuleResolutionCache {
    parsed: HashMap<ModuleId, Result<Arc<ParsedModuleUnit>, ModuleLoadError>>,
    interfaces: HashMap<ModuleId, Result<UnlinkedModuleInterface, ModuleLoadError>>,
}
```

Add `ModuleResolver::with_cache(universe, source, &mut cache)` so short-lived resolver views can borrow the persistent universe/provider/cache without creating a self-referential session struct. Unchanged parsed/interface/link products retain `Arc` identity across updates.

## 6.2 Source overlay precedence

The module workspace must support unsaved editor source without teaching `phalcom-modules` about VS Code URLs.

Create a VM-free source overlay keyed by canonical module/source identity:

```rust
pub struct SourceOverlay {
    entries: BTreeMap<SourceId, Arc<str>>,
}
```

The worker maps editor URIs to canonical source IDs at ingestion time.

Provider precedence:

```text
open/unsaved overlay
    ↓
worker-cached disk source
    ↓
filesystem provider
    ↓
builtin provider
```

No LSP request handler performs this resolution.

## 6.3 `phalcom-semantic`: evolve the existing semantic DB

Do not add a second query database.

Extend:

```text
phalcom-semantic/src/db/mod.rs
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/state.rs
phalcom-semantic/src/db/dependency.rs
phalcom-semantic/src/db/metrics.rs
phalcom-semantic/src/db/scheduler.rs
```

and add:

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/presentation.rs
```

The owning workspace facade is:

```rust
pub struct SemanticWorkspaceSession {
    db: SemanticDb,
    // compiler/session-owned mutable indexes required to evaluate queries
}

pub struct SemanticWorkspaceUpdate {
    pub snapshot: Arc<SemanticSnapshot>,
    pub invalidated: Arc<[QueryKey]>,
    pub recomputed: Arc<[QueryKey]>,
    pub stats: SemanticUpdateStats,
}
```

The compatibility function remains:

```rust
pub fn analyze_workspace(input: SemanticWorkspaceInput) -> SemanticAnalysis
```

but internally becomes a one-shot wrapper over `SemanticWorkspaceSession` so existing callers/tests are not broken.

## 6.4 Formal query values must become typed products

The current `QueryValue { bytes: Arc<[u8]> }` cannot safely drive an in-process compiler type database without serialization/deserialization overhead and loss of Rust type safety.

Replace it with a typed product enum, or an equally type-safe internal product store. The recommended first implementation is an enum:

```rust
pub enum QueryValue {
    ParsedModule(Arc<ParsedModuleUnit>),
    UnlinkedInterface(Arc<UnlinkedModuleInterface>),
    LinkedInterface(Arc<LinkedModuleInterface>),
    DeclarationSurface(Arc<DeclarationSurface>),
    CallableAnalysis(Arc<CallableAnalysis>),
    ModuleDiagnostics(Arc<[SemanticDiagnostic]>),
    ModulePresentation(Arc<ModuleSemanticPresentation>),
    Metadata(Arc<phalcom_type_meta::SemanticMetadataBundle>),
}
```

Products that are global tables may remain session-owned and be assembled from per-query products at publication.

Do not persist raw `TypeId` data outside the process or across independent type-store identities.

## 6.5 Stable formal type-store identity

The formal workspace session owns the mutable type interner for its lifetime.

Required invariant:

```text
existing TypeId -> existing TypeData mapping never changes
```

New forms append/interner-insert; removed source declarations may become unreachable from the newest declaration tables but old published snapshots remain valid.

### Publication implementation stages

Implement in two stages, both tested:

**Stage A — correctness-first persistent identity**

- keep one session `TypeStoreId`;
- mutate a session-owned store;
- freeze an immutable snapshot copy preserving the same `TypeStoreId` and ID mapping;
- reuse formal products across revisions.

This may still copy store vectors on publication, but removes universe/type reconstruction and enables correct product reuse.

**Stage B — structural-sharing store freeze**

Refactor TypeStore storage to an append-only chunked arena with immutable shared full chunks and a session-owned mutable tail. Use one internal reusable arena abstraction for types/kinds and adapt the lambda/row/parameter arenas to the same freeze model. `TypeId` remains a sequential logical index; lookup computes chunk + offset without changing public IDs. Snapshot freeze clones only chunk `Arc`s and the current tail, never the entire historical arena. Requirements:

- old snapshots require no locks;
- new intern operations never mutate memory reachable by old snapshots;
- snapshot creation is proportional to newly appended chunks/metadata, not total type count;
- `TypeStoreId` remains stable for the session;
- `TypeId` numeric meaning remains stable.

Do not put an `RwLock<TypeStore>` into request-visible snapshots.

## 6.6 Snapshot publication

Extend `SemanticSnapshot::new` to accept real callable analyses instead of constructing an empty map.

Recommended signature addition:

```rust
pub fn new(
    ...,
    callable_analyses: Arc<HashMap<CallableId, Arc<CallableAnalysis>>>,
    presentation: Arc<SemanticPresentationIndex>,
    status: SnapshotStatus,
) -> Self
```

Do not force the LSP to reconstruct formal type sites from checker internals on every request.

---

# 7. Formal type presentation layer

Create:

```text
phalcom-semantic/src/presentation.rs
```

The presentation layer is semantic, not LSP-specific. It converts canonical formal products into deterministic human-readable forms and source-indexed sites.

## 7.1 Type formatting

Target APIs:

```rust
pub struct TypePresenter<'a> {
    store: &'a TypeStore,
    declarations: &'a DeclarationTypeTable,
}

impl<'a> TypePresenter<'a> {
    pub fn display_type(&self, ty: TypeId) -> String;
    pub fn display_knowledge(&self, knowledge: &TypeKnowledge) -> Option<String>;
    pub fn display_kind(&self, kind: KindId) -> String;
}
```

Formatting must cover all canonical forms currently supported:

```text
Nominal
ClassObject
Applied
Union
Tuple
Record
Callable
Parameter
Lambda
SelfType
Never
Unit
```

Do not use `Debug` strings in user-facing IDE output.

## 7.2 Source-indexed type sites

Add:

```rust
pub enum TypeSiteKind {
    Binding,
    Parameter,
    Field,
    Return,
    Expression,
}

pub struct FormalTypeSite {
    pub module: ModuleId,
    pub range: SourceRange,
    pub insertion_offset: Option<usize>,
    pub kind: TypeSiteKind,
    pub knowledge: TypeKnowledge,
    pub explicit_annotation: bool,
    pub explanation: Option<ExplanationId>,
}

pub struct ModuleSemanticPresentation {
    pub sites: Arc<[FormalTypeSite]>,
}

pub struct SemanticPresentationIndex {
    pub modules: BTreeMap<ModuleId, Arc<ModuleSemanticPresentation>>,
}
```

`insertion_offset` is optional because not every expression needs an inlay position.

The index is built during formal analysis publication and structurally shared for unchanged modules.

---

# 8. LSP published workspace snapshot

The LSP currently publishes one advisory `SemanticSnapshot` with an optional static snapshot. Evolve this without making request handlers query multiple mutable systems.

Recommended shape:

```rust
pub struct PublishedWorkspaceSnapshot {
    pub generation: SemanticGeneration,
    pub advisory: Arc<AdvisorySemanticSnapshot>,
    pub formal: Option<Arc<phalcom_semantic::SemanticSnapshot>>,
    pub modules: Option<Arc<phalcom_modules::ModuleWorkspaceSnapshot>>,
    pub documents: Arc<DocumentModuleMap>,
}
```

If replacing the existing public `SemanticSnapshot` type would create excessive churn, keep its name and add `modules` while retaining existing advisory fields. The invariant matters more than the name:

> every LSP request pins exactly one coherent publication that identifies which advisory, formal and module products belong together.

`RequestContext` remains the only normal entry point for open-document semantic requests.

---

# 9. Detailed implementation tasks

# Task 0 — Establish the post-Wave-4 integration baseline

**Depends on:** Wave 4 merge for the final completion of this task.

**Files:**

- inspect: `phalcom-semantic/src/checker/flow/*`
- inspect: `phalcom-semantic/src/checker/statement.rs`
- inspect: `phalcom-semantic/src/checker/analysis.rs`
- inspect: `phalcom-semantic/src/checker/mod.rs`

**Purpose:** prevent this integration wave from reintroducing old nominal iteration logic or creating a second flow engine.

- [ ] Run Wave 4 focused and full semantic tests.
- [ ] Confirm `Statement::For` no longer branches on nominal `List`/`Set` names.
- [ ] Confirm `resolve_iteration_element` or its landed equivalent is protocol-derived.
- [ ] Record the final flow graph/state/predicate transfer entry points used to construct `CallableAnalysis`.
- [ ] Confirm flow assignment invalidates binding facts and loop joins/widening have tests.
- [ ] Add a short integration-plan note to the implementation PR if any Wave 4 API name differs from this document.

**Verification:**

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test spec04_5_flow_graph
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
```

---

# Task 1 — Repair the LSP analysis status lifecycle

**Files:**

- modify: `phalcom-lsp/src/analysis_service.rs`
- modify: `phalcom-lsp/src/analysis_status.rs`
- modify: `phalcom-lsp/tests/analysis_status.rs`
- modify: `tools/vsphalcom/src/test/suite/analysisStatus.test.ts` only if notification shape changes

**Interfaces:**

- consumes current `StatusTracker`
- produces one terminal/continuing status after every batch

## Step 1.1 — Write the failing edit-only status test

Add an integration test that:

1. configures a small workspace;
2. waits for initial `Ready`;
3. opens a file;
4. performs an edit that triggers semantic work but no new scan;
5. consumes status events until the edit generation is published;
6. asserts the final status for that work is `Ready`, not `Publishing`.

Suggested test name:

```rust
edit_only_batch_returns_to_ready_after_publication
```

Run it and confirm current failure.

## Step 1.2 — Centralize batch completion

Add a worker-only helper conceptually equivalent to:

```rust
fn finish_status_after_batch(
    tracker: &mut StatusTracker,
    shared: &WorkerShared,
    scanner_active: bool,
    pending_newer_work: bool,
) -> AnalysisStatus
```

Rules:

```text
newer interactive work queued -> Analyzing
scanner still active           -> Indexing
idle                           -> Ready
```

Do not emit `Ready` if immediately-known work is already pending.

## Step 1.3 — Handle stale/cancelled work

When a batch is discarded because a newer epoch superseded it:

- emit `StaleBatchDiscarded` for logging/metrics;
- transition status to the phase of the newer work, or `Ready` if the worker is now idle;
- never leave the tracker in `Publishing`.

## Step 1.4 — Add state-machine tests

Add tests for:

- initial scan -> Ready;
- edit-only Analyzing -> Publishing -> Ready;
- edit during scan -> Publishing -> Indexing -> Ready;
- stale batch -> newer Analyzing, no stale Ready publication;
- error -> Error with preserved session/sequence.

**Verification:**

```bash
cargo test -p phalcom-lsp --test integration analysis_status -- --nocapture
```

---

# Task 2 — Add structured analysis logs and make failures observable

**Files:**

- modify: `phalcom-lsp/src/analysis_service.rs`
- modify: `phalcom-lsp/src/analysis_status.rs`
- modify: `phalcom-lsp/src/backend.rs`
- modify: `phalcom-lsp/src/perf.rs`
- create: `phalcom-lsp/src/analysis_log.rs`
- create: `phalcom-lsp/tests/analysis_logging.rs`
- modify: `phalcom-lsp/tests/integration.rs`
- modify: `tools/vsphalcom/src/extension.ts`
- modify: `tools/vsphalcom/package.json`
- create: `tools/vsphalcom/src/test/suite/analysisLog.test.ts`

## Step 2.1 — Define a structured notification

Create:

```rust
pub enum AnalysisLogLevel {
    Error,
    Info,
    Verbose,
}

pub struct AnalysisLogEvent {
    pub session: u64,
    pub sequence: u64,
    pub level: AnalysisLogLevel,
    pub phase: AnalysisPhase,
    pub event: String,
    pub epoch: Option<u64>,
    pub generation: Option<u64>,
    pub uri: Option<Url>,
    pub revision: Option<u64>,
    pub batch_size: Option<u32>,
    pub duration_ms: Option<u64>,
    pub message: Option<String>,
    pub counters: Option<CounterSnapshot>,
}
```

Custom notification:

```text
phalcom/analysisLog
```

Keep event names stable strings suitable for grep, e.g.:

```text
workspace.session.started
core.surface.loaded
scan.batch.completed
semantic.batch.started
semantic.batch.cancelled
formal.update.started
formal.update.published
formal.update.failed
module.resolve.failed
module.link.failed
snapshot.published
```

## Step 2.2 — Worker owns error status identity

Remove the backend behavior that fabricates:

```text
session = 0
sequence = 0
mode = Local
```

on `AnalysisEvent::Error`.

Preferred design:

- worker calls `status_tracker.set_error(message)`;
- worker emits `AnalysisEvent::Status(error_status)`;
- worker emits `AnalysisLogEvent` with full failure context;
- `AnalysisEvent::Error` is removed if it becomes redundant, or carries structured failure details only.

## Step 2.3 — Log previously silent formal/module failures

Every branch that currently uses `continue`/`None` for project load, source load, interface extraction, import resolution, linking or semantic refresh must produce either:

- a source diagnostic if attributable to source;
- an analysis error/log event if infrastructure-level;
- both when appropriate.

## Step 2.4 — Add extension configuration

Add:

```json
"phalcom.analysis.logLevel": {
  "type": "string",
  "enum": ["off", "summary", "verbose"],
  "default": "summary"
}
```

Pass it in initialization options.

The extension subscribes to `phalcom/analysisLog` and appends deterministic lines to the existing `Phalcom Language Server` output channel.

Example:

```text
[session=3 seq=42 phase=analyzing gen=18 epoch=91] formal.update.published modules_rechecked=1 modules_reused=14 duration_ms=27
```

Do not dump whole ASTs/source text at normal log levels.

## Step 2.5 — Tests

Test:

- sequence/session preserved;
- failures appear in output notification;
- stale batches are logged;
- summary mode suppresses verbose per-file noise;
- extension rejects stale status but still logs useful stale-batch information.

**Verification:**

```bash
cargo test -p phalcom-lsp --test integration analysis_logging -- --nocapture
cd tools/vsphalcom && npm test
```

---

# Task 3 — Separate compiler analysis from compiler validity

**Files:**

- modify: `phalcom-core/src/modules/compile.rs`
- modify: `phalcom-core/tests/semantic_analysis.rs`
- modify: `phalcom-core/tests/integration.rs` if module registration is needed

## Step 3.1 — Keep the existing RED analyzer test

The already-created test should express:

```rust
analyzer_preserves_semantic_errors_in_snapshot
```

with source:

```phalcom
const count: String = 1
```

Expected:

```text
ProgramAnalyzer => Ok(AnalyzedProgram)
analyzed.semantic.has_errors() == true
```

## Step 3.2 — Remove semantic rejection from analyzer paths

Remove `snapshot.has_errors()` early returns from:

- `EntrySelection::Inline`;
- standalone module analysis;
- project/package discovery analysis.

Do not remove parse/link/project infrastructure errors.

## Step 3.3 — Add snapshot-to-program-diagnostic conversion

Add:

```rust
impl ProgramSemanticDiagnostics {
    pub fn from_snapshot(snapshot: &phalcom_semantic::SemanticSnapshot) -> Self
}
```

Preserve all semantic diagnostics and modules.

## Step 3.4 — Add the compiler RED test

```rust
compiler_rejects_program_with_semantic_errors
```

After analyzer repair, this should initially fail because `compile_analyzed()` currently accepts the invalid snapshot.

## Step 3.5 — Gate code generation

At the start of:

```rust
ProgramCompiler::compile_analyzed
```

add:

```text
if analyzed.semantic.has_errors()
    => ProgramCompileError::Semantic(...)
```

No compiled module or semantic metadata artifact is created after this gate.

## Step 3.6 — Tests

Also test:

- valid analyzed program compiles;
- semantic warnings do not block compilation;
- semantic errors do;
- analyzer source map remains available for invalid programs.

**Verification:**

```bash
cargo test -p phalcom-core --test integration semantic_analysis -- --nocapture
```

---

# Task 4 — Make semantic diagnostics source-owned and render them richly everywhere

**Files:**

- modify: `phalcom-semantic/src/diagnostic.rs`
- audit/modify: `phalcom-semantic/src/checker/*.rs`
- audit/modify after Wave 4 merge: `phalcom-semantic/src/checker/flow/*.rs`
- modify: `phalcom-semantic/src/workspace.rs`
- modify: `phalcom-lsp/src/diagnostics.rs`
- modify: `phalcom-lsp/src/backend.rs`
- modify: `phalcom-core/bin/phalcom/cli.rs`
- create/modify tests in `phalcom-semantic/tests/identity_diagnostic_foundation.rs`
- modify: `phalcom-lsp/tests/stage7_static_diagnostics.rs`
- modify: `phalcom-core/tests/semantic_analysis.rs`

## Step 4.1 — Write source-ownership regression test

Given a synthetic/user module containing:

```phalcom
const count: String = 1
```

assert:

```text
diagnostic primary.module == source module
every same-file label span.module == source module
```

This test must fail against current `SemanticDiagnostic::error(...)` call sites.

## Step 4.2 — Remove implicit-core production constructors

Preferred API:

```rust
pub fn error_in(module: ModuleId, ...)
pub fn warning_in(module: ModuleId, ...)
pub fn info_in(module: ModuleId, ...)
pub fn hint_in(module: ModuleId, ...)
```

Either remove `error(...)` / `warning(...)` entirely or make them test-only/private. Core diagnostics must explicitly pass `ModuleId::core()`.

## Step 4.3 — Audit every semantic diagnostic creation

Run:

```bash
rg 'SemanticDiagnostic::(error|warning)\(' phalcom-semantic/src
```

Convert every production call to an explicitly owned module.

Checker diagnostics use `ctx.current_module.clone()` unless the diagnostic genuinely belongs elsewhere.

Workspace/link-derived diagnostics use the module that owns the source span.

## Step 4.4 — Cross-module LSP labels use their own URI

Current LSP adapter uses the primary document URI for every related label.

Introduce a source resolver abstraction:

```rust
pub trait DiagnosticSourceResolver {
    fn location_for_span(
        &self,
        span: &SemanticSourceSpan,
    ) -> Option<(Url, Arc<LineIndex>)>;
}
```

For each `DiagnosticLabel`, resolve `label.span.module` independently.

Notes/helps that are not source-specific may remain attached to the primary diagnostic.

## Step 4.5 — CLI uses semantic renderer

After Task 3, the `Ok(analyzed)` semantic-error branch in `cmd_check` becomes reachable.

Replace semantic use of generic `print_parse` with a semantic renderer that consumes:

- diagnostic code/severity;
- primary module source;
- primary/secondary labels;
- notes/helps.

At minimum, same-file labels use `SemanticDiagnostic::render`. Prefer a new multi-source renderer in `phalcom-diagnostics` for cross-file labels.

Update stale CLI help that says check is syntax-only.

Use the same rendering utility for semantic errors encountered by `run`.

## Step 4.6 — JSON diagnostic schema

Keep machine-readable output stable and richer:

```json
{
  "severity": "error",
  "code": "type.binding.initializer_mismatch",
  "message": "...",
  "module": "...",
  "range": { ... },
  "labels": [
    { "module": "...", "range": { ... }, "message": "declared type" }
  ],
  "notes": [],
  "helps": []
}
```

Use `serde::Serialize` structs rather than manual JSON string concatenation.

## Step 4.7 — Regression commands

```bash
cargo run -p phalcom-core --bin phalcom -- check --source 'const count: String = 1'
cargo run -p phalcom-core --bin phalcom -- check --format json --source 'const count: String = 1'
```

Expected text output includes source snippet and both `declared type` / `inferred type` labels.

---

# Task 5 — Suppress inlay hints at explicit annotations

**Files:**

- modify: `phalcom-lsp/src/inlay_hints.rs`
- modify: `phalcom-lsp/tests/stage6_inlay_hints.rs`
- modify: `phalcom-lsp/tests/inlay_hints.rs`

This task is deliberately independent of formal-type presentation. Fix policy correctness first.

## Step 5.1 — Add failing tests for every annotation site

Cover:

```phalcom
let x: Int = 1
const y: String = "x"

class Box {
  _value: Int = 1

  map(_ value: Int): String { ... }
  getter: String { ... }
  setter=(_ value: Int): Unit { ... }
  [_ index: Int]: String { ... }
}
```

Adapt syntax to the exact parser-supported forms.

Assert no duplicate advisory inlay is inserted at:

- annotated binding;
- annotated field;
- annotated parameter;
- annotated method/getter/setter/index return.

Also assert unannotated equivalents still receive hints.

## Step 5.2 — Read annotation presence from AST, not source text

Do not scan for `:` characters.

Use the existing AST fields and member AST references to determine whether a declaration is explicitly annotated.

For local scope bindings, build a declaration-range -> AST annotation map once per request/file, or enrich `BindingInfo` during scope construction with an `explicit_annotation` flag.

Preferred long-term fix: add annotation-presence metadata to source surfaces/scope bindings so inlay generation does not repeatedly walk the entire AST.

## Step 5.3 — Keep `suppressObvious` independent

`phalcom.inlayHints.suppressObvious` applies to unannotated obvious inference only. Explicit annotation suppression is unconditional while type inlays are enabled.

**Verification:**

```bash
cargo test -p phalcom-lsp --test integration stage6_inlay_hints -- --nocapture
cargo test -p phalcom-lsp --test integration inlay_hints -- --nocapture
```

---

# Task 6 — Make normal core startup surface-only

**Files:**

- modify: `phalcom-lsp/src/semantic/engine.rs` or its current owning module
- modify: `phalcom-lsp/src/semantic/core_source.rs`
- modify: `phalcom-lsp/src/analysis_service.rs`
- modify: `phalcom-lsp/src/semantic/invalidation.rs`
- modify: `phalcom-lsp/src/perf.rs`
- create: `phalcom-lsp/tests/core_startup.rs`
- modify: `phalcom-lsp/tests/integration.rs`
- modify: `phalcom-lsp/tests/performance.rs`

## Step 6.1 — Add analysis policy

Add a worker/engine concept:

```rust
pub enum SourceAnalysisDepth {
    SurfaceOnly,
    Deep,
}
```

Core update API:

```rust
pub fn update_core_with_depth(
    &mut self,
    revision: FileRevision,
    text: Arc<str>,
    program: &Program,
    depth: SourceAnalysisDepth,
) -> SemanticGeneration
```

`SurfaceOnly` must still build:

- core module source snapshot;
- classes/superclasses;
- members/fields;
- native metadata;
- occurrence/source indexes required for hover/navigation where ranges are trustworthy;
- completion surfaces.

It skips ordinary callable body flow solving.

## Step 6.2 — Default startup path uses `SurfaceOnly`

In `analysis_service.rs` core selection:

```text
select core
parse core
publish surface/native semantic core
start/continue workspace scan
```

Do not block initial workspace discovery on deep core flow analysis.

## Step 6.3 — Deep core behavior

When a physical core document is opened/edited, treat that specific source as deep-analyzed editor source.

Workspace analysis mode may schedule deep core analysis after the first `Ready` publication, but it must remain background/cancellable.

## Step 6.4 — Structural performance test

Add a counter such as:

```text
core_callables_deep_analyzed
```

Startup test asserts it is zero (or bounded only to explicit required bootstrap bodies) before initial Ready for an ordinary user workspace.

Also assert completion/hover for core members still works.

## Step 6.5 — Benchmark

Re-run the existing local/workspace convergence harness before and after. Record the reduction from the reported ~21–22 second core phase.

Do not make the exact developer-machine time a brittle CI assertion.

---

# Task 7 — Populate formal `CallableAnalysis` from Wave 4 flow

**Depends on:** Task 0 / Wave 4 merged.

**Files:**

- modify: `phalcom-semantic/src/checker/analysis.rs`
- modify: `phalcom-semantic/src/checker/declaration.rs`
- modify: `phalcom-semantic/src/checker/mod.rs`
- modify as required by landed API: `phalcom-semantic/src/checker/flow/*`
- modify: `phalcom-semantic/src/workspace.rs`
- modify: `phalcom-semantic/src/snapshot.rs`
- create: `phalcom-semantic/tests/callable_analysis.rs`

## Step 7.1 — Define the callable analysis boundary

Formal body checking must produce one coherent `CallableAnalysis` rather than only side-effecting `ctx.diagnostics`.

Target API:

```rust
pub fn analyze_callable_body(
    ...
) -> CallableAnalysis
```

or an equivalent method on a body analysis engine.

It consumes Wave 4 CFG/flow products and records:

- expression analyses by stable body-local expression ID;
- binding states;
- diagnostics;
- callable dependencies;
- analysis status;
- explanation IDs where available.

## Step 7.2 — Expression ID assignment

Assign deterministic `ExpressionId { owner: BodyId, local }` in source traversal order or from a dedicated body index.

Within one unchanged callable body fingerprint, expression identity should remain deterministic. Do not promise identity stability across arbitrary source edits; source ranges + revision guard remain authoritative for LSP use.

## Step 7.3 — Publish callable analyses

Change `SemanticSnapshot::new` so `callable_analyses` is supplied, not hard-coded empty.

Whole-workspace analysis collects analyses for all checked callables in the current one-shot implementation.

## Step 7.4 — Tests

For a method containing:

```phalcom
let x = 42
x
```

assert the published callable analysis contains:

- binding `x` with `Known(Int)`;
- literal expression `Known(Int)` with exact-syntax evidence;
- variable expression `Known(Int)`;
- no diagnostics.

Add branch-refinement test using one Wave 4 predicate and mutation invalidation test.

**Verification:**

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test callable_analysis
```

---

# Task 8 — Add the formal semantic presentation index

**Depends on:** Task 7.

**Files:**

- create: `phalcom-semantic/src/presentation.rs`
- modify: `phalcom-semantic/src/lib.rs`
- modify: `phalcom-semantic/src/snapshot.rs`
- modify: `phalcom-semantic/src/workspace.rs`
- create: `phalcom-semantic/tests/presentation.rs`

## Step 8.1 — Type presenter tests first

Test deterministic output for:

```text
Int
List<Int>
Map<String, Int>
Int | String
(Int, String)
#{ name: String }
(Int) -> String
Self
Never
()
```

Use canonical declaration names, not internal IDs/debug output.

## Step 8.2 — Formal site construction

Construct module presentation sites from:

- source declaration annotations;
- declaration semantic signatures;
- callable analyses;
- top-level binding analysis;
- field analysis.

Only sites with source ranges belonging to that module enter its presentation map.

## Step 8.3 — Annotation flag

Each declaration site records whether source already carried an explicit annotation. This allows LSP inlay code to become a simple policy query.

## Step 8.4 — Snapshot accessor

Add:

```rust
pub fn presentation_for(&self, module: &ModuleId) -> Option<&ModuleSemanticPresentation>
```

**Verification:**

```bash
cargo test -p phalcom-semantic --test presentation
```

---

# Task 9 — Make LSP type presentation formal-first

**Depends on:** Tasks 5, 8.

**Files:**

- modify: `phalcom-lsp/src/inlay_hints.rs`
- modify: `phalcom-lsp/src/backend.rs`
- modify: `phalcom-lsp/src/hover.rs`
- modify: `phalcom-lsp/src/request_context.rs`
- modify: `phalcom-lsp/tests/stage6_inlay_hints.rs`
- create: `phalcom-lsp/tests/formal_type_presentation.rs`
- modify: `phalcom-lsp/tests/integration.rs`

## Step 9.1 — Formal snapshot revision guard

Formal presentation is usable only when the static snapshot/source revision matches the pinned document, using the same strict coherence policy as formal diagnostics.

Never reinterpret formal source ranges against a newer document.

## Step 9.2 — Inlay priority

For a declaration:

```text
explicit annotation -> no inferred type hint
formal Known(T)      -> `: T` or ` -> T`
formal Dynamic       -> no formal type hint; optional advisory fallback
formal Unknown       -> optional advisory fallback
formal unavailable   -> optional advisory fallback
```

## Step 9.3 — Visually distinguish advisory fallback

Change advisory labels from formal-looking syntax to:

```text
≈ String
```

and no other marker in this implementation wave.

Tooltip:

```text
Advisory runtime-shape inference
Confidence: Exact/Flow/Heuristic
Not a formal Phalcom type judgment.
```

Formal tooltip:

```text
Formal Phalcom type: List<Int>
Evidence: Proven / ExactSyntax / Declared / TrustedNative
```

## Step 9.4 — Hover formal section

Where the cursor resolves to a binding/member/expression with formal analysis, add a clearly titled formal type section before advisory runtime shape.

Do not hide advisory information when it adds runtime-shape value; separate the axes.

## Step 9.5 — Integration tests

Test:

- `let x = 42` -> formal `: Int` after formal publication;
- `let x: Int = 42` -> no duplicate inlay;
- formal mismatch still displays diagnostic even if advisory shape agrees with source annotation;
- formal Unknown + strong advisory shape uses `≈` marker;
- stale formal snapshot does not produce stale hint.

---

# Task 10 — Implement a persistent project/module workspace session

**Files:**

- create: `phalcom-modules/src/workspace.rs`
- modify: `phalcom-modules/src/lib.rs`
- modify: `phalcom-modules/src/resolver.rs`
- modify: `phalcom-modules/src/source.rs`
- modify: `phalcom-modules/src/interface.rs` to add deterministic interface fingerprint construction
- modify: `phalcom-modules/src/linker.rs` only for incremental/public snapshot helpers
- create: `phalcom-modules/tests/workspace_session.rs`

## Step 10.1 — Project session construction test

Scratch project:

```text
app/
  project.toml
  src/package.ph
  src/main.ph
  src/models.ph
```

Construct one session from `project.toml` and assert:

- project loaded once;
- source root known;
- all canonical module IDs stable;
- root interface/link snapshot created.

## Step 10.2 — Source fingerprints

Define deterministic source and interface fingerprints. Avoid hashing `Debug` output.

Recommended:

```rust
pub struct SourceFingerprint(u64);
pub struct InterfaceFingerprint(u64);
```

The source fingerprint covers source bytes. The interface fingerprint covers semantic interface content:

- module kind;
- imports/re-exports/exposes;
- exported declaration surface needed by linking;
- metadata that affects semantics.

A method-body-only edit should change source fingerprint but not interface fingerprint.

## Step 10.3 — Reuse parsed/interface products

On source update:

```text
same source fingerprint      -> reuse parse + interface
body-only source change      -> parse new source, reuse link/interface dependents when interface fingerprint unchanged
interface change             -> invalidate/relink reverse interface dependents
added/removed module         -> repair affected imports only
```

## Step 10.4 — Open-source overlay

Add source replacement API suitable for the LSP worker:

```rust
pub enum WorkspaceSourceChange {
    Replace {
        module: ModuleId,
        text: Arc<str>,
    },
    Remove {
        module: ModuleId,
    },
}
```

An alternate source-id-based API is acceptable if it preserves canonical project identity.

## Step 10.5 — Immutable snapshot

Publish `Arc<ModuleWorkspaceSnapshot>` with structural sharing.

No request/query consumer gets mutable `ModuleResolver` access.

## Step 10.6 — Tests

Add tests for:

- body edit does not relink importer;
- export change relinks importer;
- add/remove provider repairs only matching importers;
- relative import identity stable;
- path dependency project loaded once;
- builtin imports resolve through the same snapshot.

**Verification:**

```bash
cargo test -p phalcom-modules --test workspace_session
cargo test -p phalcom-modules
```

---

# Task 11 — Add canonical module-completion queries

**Depends on:** Task 10.

**Files:**

- modify: `phalcom-modules/src/workspace.rs`
- optionally create: `phalcom-modules/src/query.rs`
- create: `phalcom-modules/tests/module_query.rs`

## Step 11.1 — Import root query

API:

```rust
pub struct ImportCandidate {
    pub name: Box<str>,
    pub target: ModuleId,
    pub kind: ImportCandidateKind,
}

pub enum ImportCandidateKind {
    ProjectRoot,
    DependencyRoot,
    BuiltinRoot,
    Package,
    Module,
    Export,
}

pub fn import_roots(&self, importer: &ModuleId) -> Vec<ImportCandidate>
```

Includes:

```text
self namespace
dependency aliases
std
universe
```

## Step 11.2 — Child path query

API concept:

```rust
pub fn import_children(
    &self,
    importer: &ModuleId,
    prefix: &ImportPathPrefix,
) -> Result<Vec<ImportCandidate>, ModuleResolutionError>
```

It must apply the same external exposure rules as exact resolution.

Do not enumerate arbitrary filesystem children and filter afterward as semantic authority.

## Step 11.3 — Export query

API:

```rust
pub fn public_exports(&self, module: &ModuleId) -> impl Iterator<Item = (&str, &LinkedExport)>
```

Completion and navigation use linked exports/re-export origin, not declaration scans.

## Step 11.4 — Golden exposure test

Reuse the existing geometry fixture concept:

```text
geometry
  point            exposed
  shapes           exposed
    circle         exposed
  private_tool     NOT exposed
```

Assert `geometry.` candidates are exactly the exposed public children required by the fixture and never contain `private_tool`.

---

# Task 12 — Add first-class import/module completion to the LSP

**Depends on:** Task 11.

**Files:**

- modify: `phalcom-lsp/src/completion.rs`
- create: `phalcom-lsp/src/import_completion.rs`
- modify: `phalcom-lsp/src/backend.rs`
- modify: `phalcom-lsp/src/request_context.rs`
- modify: `phalcom-lsp/src/semantic/snapshot.rs` or replacement published snapshot
- create: `phalcom-lsp/tests/module_completion.rs`
- modify: `phalcom-lsp/tests/integration.rs`

## Step 12.1 — Context classifier

Introduce:

```rust
pub enum CompletionContext {
    General,
    Member {
        receiver_range: SourceRange,
        partial: String,
    },
    ImportRoot {
        partial: String,
    },
    ImportPath {
        raw_prefix: String,
        partial: String,
    },
    SelectiveImport {
        raw_module_path: String,
        partial: String,
    },
    ReExport {
        raw_module_path: String,
        partial: String,
    },
    Expose {
        partial: String,
    },
    ModuleMember {
        module: phalcom_modules::ModuleId,
        partial: String,
    },
}
```

Member completion remains the current behavior.

## Step 12.2 — Incomplete syntax recovery

Import completion must work while the line is syntactically incomplete:

```phalcom
import geometry.
from geometry.point import (
```

Implement a bounded lexer-assisted completion-context scanner over the current preamble statement: tokenize from the start of the current dependency declaration through the cursor with `phalcom_ast::Lexer`, preserve dot/identifier/parenthesis/comma tokens even when the declaration is incomplete, and classify the context without requiring a clean full-document parse.

Do not reinterpret a dot inside an import path as object member access.

## Step 12.3 — Completion queries are snapshot-only

The handler maps the open document URI to canonical module identity and queries the immutable `ModuleWorkspaceSnapshot`.

No disk I/O, project discovery, or resolver mutation occurs in the request.

## Step 12.4 — Completion item semantics

Use appropriate LSP kinds:

```text
project/package/module -> MODULE
class/type export       -> CLASS
function/value export   -> VARIABLE/FUNCTION as available
```

Include detail such as:

```text
module · geometry.shapes
export · Point from geometry.point
```

## Step 12.5 — Trigger characters

Keep `.` and add trigger characters that materially improve selective import lists, such as `(` and `,`, after verifying VS Code behavior. Ordinary identifier typing still relies on normal completion requests.

## Step 12.6 — Tests

Cover:

```phalcom
import |
import geo|
import geometry.|
import geometry.shapes.|
import .|
import ..|
import ..models.|
from geometry.point import (|)
from geometry.point import (Po|)
```

Assert private/unexposed paths do not appear.

---

# Task 13 — Make module imports first-class semantic targets and navigation sites

**Depends on:** Tasks 10–12.

**Files:**

- modify: `phalcom-lsp/src/semantic/occurrence.rs`
- modify: `phalcom-lsp/src/backend.rs`
- modify: `phalcom-lsp/src/semantic/snapshot.rs`
- create: `phalcom-lsp/tests/module_navigation.rs`
- modify: `phalcom-lsp/tests/integration.rs`

## Step 13.1 — Add module target

Add an occurrence target that holds canonical module identity through an LSP-safe bridge:

```rust
SemanticTarget::Module(phalcom_modules::ModuleId)
```

If the advisory occurrence layer cannot directly use canonical IDs without undesirable type churn, use a stable wrapper that maps losslessly to canonical identity. Do not store only a guessed URI string.

## Step 13.2 — Index preamble occurrences

Walk `program.preamble.dependencies` and create occurrences for:

- import root/path segments;
- module alias declarations;
- selective imported names;
- re-exported names;
- `expose` child names.

Path segments should resolve to the closest canonical module/package represented by that prefix.

## Step 13.3 — Definition behavior

Examples:

```phalcom
import geometry.shapes.circle
       ^^^^^^^^ -> geometry project/package source
                ^^^^^^ -> shapes package source
                       ^^^^^^ -> circle module source
```

Selective import item definition follows the linked export origin, including re-exports.

## Step 13.4 — References

Whole-module alias references should resolve consistently. Cross-workspace references to a module may initially mean syntactic import/reference occurrences, not arbitrary runtime reflection.

## Step 13.5 — Tests

Add real JSON-RPC tests for definition into:

- sibling module;
- package module;
- path dependency;
- re-export origin;
- builtin module virtual/physical source after Task 14.

---

# Task 14 — Implement physical and virtual builtin/core source navigation

**Files:**

- modify: `phalcom-lsp/src/semantic/core_source.rs`
- modify: `phalcom-lsp/src/backend.rs`
- create: `phalcom-lsp/src/virtual_source.rs`
- create: `phalcom-lsp/tests/core_navigation.rs`
- modify: `phalcom-lsp/tests/integration.rs`
- modify: `tools/vsphalcom/src/extension.ts`
- modify: `tools/vsphalcom/src/test/suite/lsp.e2e.test.ts`

## Step 14.1 — Source identity abstraction

Represent definition sources as:

```rust
pub enum DefinitionSource {
    Physical(Url),
    Virtual(Url),
}
```

Map canonical builtin module IDs to their individual provider sources.

## Step 14.2 — Physical core definition

When selected core source is physical and ranges correspond to that physical module/source, return its `file://` location.

Do not blindly map every collapsed `CORE_MODULE_URI` range to one package file if the declaration originated in another builtin module.

## Step 14.3 — Virtual URI scheme

For bundled source, use deterministic URIs such as:

```text
phalcom://universe/core/package.ph
phalcom://universe/collections/list.ph
phalcom://std/...
```

Use this exact URI mapping from canonical `BuiltinProject + ModulePath + ModuleKind`:

```text
root package:      phalcom://universe/package.ph
module a.b:        phalcom://universe/a/b.ph
package a.b:       phalcom://universe/a/b/package.ph
std root package:  phalcom://std/package.ph
std module a.b:    phalcom://std/a/b.ph
```

Construct these URIs from canonical components; never parse a display string back into module identity.

## Step 14.4 — Custom source request

Add:

```text
phalcom/sourceText
```

Request:

```json
{ "uri": "phalcom://universe/..." }
```

Response:

```json
{
  "languageId": "phalcom",
  "text": "...",
  "readOnly": true
}
```

The server serves source from the builtin provider; no external network/disk dependency is required.

## Step 14.5 — VS Code content provider

Register:

```ts
workspace.registerTextDocumentContentProvider("phalcom", provider)
```

Provider calls the custom LSP request and returns canonical source text.

## Step 14.6 — Core class/member definition

Remove the unconditional `CORE_MODULE_URI => None` behavior from class/member definition routing.

For source-declared native members, navigate to the Phalcom declaration. Native Rust implementation navigation may be added later through `textDocument/implementation`; it is not required to make definition correct.

## Step 14.7 — Core Phaldoc

Once source resolution works, allow `member_phaldoc` and class Phaldoc harvesting against physical/virtual core source rather than returning `None` solely because owner is core.

## Step 14.8 — Tests

Test:

- `Int` class definition opens core source;
- a source-declared core method opens its declaration;
- virtual builtin document text matches provider source;
- source range lands on exact declaration token;
- physical sysroot source wins when configured.

---

# Task 15 — Replace LSP static rebuild orchestration with the persistent module workspace

**Depends on:** Task 10.

**Files:**

- modify: `phalcom-lsp/src/analysis_service.rs`
- modify: `phalcom-lsp/src/semantic/snapshot.rs`
- modify: `phalcom-lsp/src/semantic/engine.rs`
- modify: `phalcom-lsp/src/semantic/module_graph.rs`
- remove after migration: LSP-specific `StaticWorkspaceIdentity`
- remove after migration: LSP-specific `StaticSourceProvider`
- remove after migration: `run_static_workspace_analysis`
- remove after migration: `refresh_static_workspace_analysis`
- create: `phalcom-lsp/tests/module_workspace_bridge.rs`

## Step 15.1 — Worker owns one `ModuleWorkspaceSession`

Initialize/reconfigure it when workspace/project configuration changes.

Do not rebuild `ProjectUniverse` per semantic edit.

## Step 15.2 — Project-aware initialization

For each workspace root:

```text
root/project.toml exists
    -> load persistent root project immediately
    -> obtain root + dependency source roots
else
    -> generic progressive scan fallback
```

A multi-root workspace may own multiple module sessions internally or one session capable of several root projects; canonical project IDs must remain collision-free.

## Step 15.3 — Feed scan/open changes to the module session

Open document source wins over disk.

Watched-file changes update the same module session.

The module session returns a delta used by both advisory invalidation and formal invalidation.

## Step 15.4 — Advisory graph ingests canonical edges

Stop using URI/path `ModuleGraph::update(...)` as semantic resolution authority.

Construct advisory `ImportEdge`s directly from `ModuleWorkspaceSnapshot` resolved bindings/links. Keep `update_with_shared_resolver` only as a compatibility/test helper until its callers are migrated, then delete the production URI-heuristic resolution path. Remove the production need for `import_candidates()` path guessing.

## Step 15.5 — Failure behavior

A module update failure:

- emits structured logs/status;
- emits source diagnostics where possible;
- retains the last valid module/formal snapshot;
- does not silently `continue` and pretend the import vanished.

## Step 15.6 — Tests

Assert:

- one project universe/session construction on startup;
- body edit does not recreate project session;
- relative import resolves identically to compiler;
- package exposure honored in LSP graph;
- failed import remains visible as an unresolved edge/diagnostic.

---

# Task 16 — Turn `phalcom-semantic::SemanticDb` into the active formal query database

**Depends on:** Tasks 7, 10; Wave 4 merged.

**Files:**

- modify: `phalcom-semantic/src/db/mod.rs`
- modify: `phalcom-semantic/src/db/key.rs`
- modify: `phalcom-semantic/src/db/state.rs`
- modify: `phalcom-semantic/src/db/dependency.rs`
- modify: `phalcom-semantic/src/db/metrics.rs`
- modify: `phalcom-semantic/src/db/scheduler.rs`
- modify: `phalcom-semantic/src/invalidation.rs`
- create: `phalcom-semantic/src/session.rs`
- modify: `phalcom-semantic/src/lib.rs`
- create: `phalcom-semantic/tests/semantic_db_incremental.rs`

## Step 16.1 — Replace byte-erased query values

Use the typed product model from Section 6.4.

Write tests that a `CallableBody` query can be published/retrieved without serialization.

## Step 16.2 — Add query fingerprints

Define product fingerprints from semantic inputs/products, not arbitrary memory addresses.

At minimum:

```text
ParsedModule          source fingerprint
UnlinkedInterface     interface fingerprint
LinkedInterface       linked export/binding fingerprint
DeclarationSurface    declaration/signature fingerprint
CallableBody          callable body + relevant declaration environment fingerprint
ModuleDiagnostics     consumed semantic product fingerprints
ModulePresentation    callable/signature analysis fingerprints
```

## Step 16.3 — Record dependencies dynamically

Examples:

```text
CallableBody(A.m)
    depends on DeclarationSurface(A)
    depends on called CallableBody(B.n) or callable signature product
    depends on imported LinkedInterface(module)
    depends on hierarchy/declaration components consumed by relations

ModuleDiagnostics(M)
    depends on all formal products that emitted diagnostics for M
```

Use the existing `DependencyRecorder`/`DependencyIndex`.

## Step 16.4 — Seed invalidation from module delta

Mapping examples:

```text
body-only callable edit
    -> QueryKey::CallableBody(changed callable)

interface/export/import edit
    -> LinkedInterface(module)
    -> reverse dependent query closure

superclass/generic signature edit
    -> DeclarationSurface(declaration)
    -> hierarchy/affected body dependents

module removal
    -> parsed/interface/declaration/body keys for module
    -> reverse closure
```

## Step 16.5 — Do not erase ready products before knowing they are invalid

Use fingerprints and reverse dependencies so unaffected query states remain `Ready` across revisions.

`begin_revision()` does not imply global cache clear.

## Step 16.6 — Metrics

Extend `QueryMetrics`/semantic update stats with:

```text
queries_reused
queries_recomputed
queries_invalidated
callable_bodies_reused
callable_bodies_rechecked
module_diagnostics_reused
type_nodes_added
```

## Step 16.7 — Tests

Test structural work, not only output:

- body edit invalidates exactly its callable seed before dependency propagation;
- unrelated callable remains same `Arc` product;
- interface edit invalidates importer-dependent products;
- equivalent body edit whose product fingerprint is unchanged stops propagation;
- cancelled revision cannot publish into newer revision.

---

# Task 17 — Persist the type store across formal revisions

**Depends on:** Task 16.

**Files:**

- modify: `phalcom-semantic/src/types/store.rs`
- modify: `phalcom-semantic/src/types/type_lambda.rs`
- modify: row/type-parameter arenas as required
- modify: `phalcom-semantic/src/db/mod.rs`
- modify: `phalcom-semantic/src/snapshot.rs`
- create: `phalcom-semantic/tests/type_store_revisions.rs`

## Step 17.1 — Stable identity test

Across two revisions in one `SemanticWorkspaceSession`, assert:

```text
snapshot1.store.id == snapshot2.store.id
Int TypeId is identical
unchanged declaration form TypeId is identical
newly interned type IDs append without remapping older IDs
```

## Step 17.2 — Old-snapshot immutability test

Publish snapshot 1, perform revision 2 intern/update, then assert every type/kind visible through snapshot 1 remains unchanged and no synchronization is required to read it.

## Step 17.3 — Correctness-first freeze

Implement the Stage A freeze described earlier if structural storage refactoring is too large for one commit.

Measure its cost explicitly.

## Step 17.4 — Structural sharing

Refactor arenas so unchanged type nodes are shared.

Add a metric/assertion that a body-only edit which interns no new types does not clone/materialize the entire type universe.

## Step 17.5 — Store-local safety

Keep `SnapshotTypeRef { store, id }` checks where cross-snapshot APIs could accidentally combine store identities.

Independent compiler sessions may use different TypeStoreIds; only a single workspace session promises stable store identity.

---

# Task 18 — Incrementalize formal declaration, hierarchy, dispatch and body products

**Depends on:** Tasks 16–17.

**Files:**

- modify: `phalcom-semantic/src/workspace.rs`
- modify: `phalcom-semantic/src/session.rs`
- modify: `phalcom-semantic/src/declarations.rs`
- modify: `phalcom-semantic/src/dispatch.rs`
- modify: `phalcom-semantic/src/signature.rs`
- modify: `phalcom-semantic/src/resolver.rs`
- modify: `phalcom-semantic/src/snapshot.rs`
- create: `phalcom-semantic/tests/incremental_workspace.rs`

## Step 18.1 — Refactor whole-workspace phases into reusable queries/products

Map current phases to incremental products:

```text
A Universe Bootstrap
    -> session bootstrap once

B Predeclare source declarations
    -> per-module/declaration shell products

C LinkedTypeResolver
    -> workspace/session resolver over current linked snapshot

D Semantic graph
    -> incremental semantic component edges

E Shell realization
    -> affected declaration components

F Hierarchy
    -> per-declaration hierarchy edges + affected closure

G Declaration surfaces/signatures
    -> per-declaration surface products

H Body checking
    -> per-callable/top-level analysis queries

I/J Snapshot freeze
    -> assemble immutable reused products
```

## Step 18.2 — Universe bootstrap once

Native universe declarations, native surfaces and stable core relations are constructed once per semantic workspace session unless the configured core semantic surface itself changes.

A normal user body edit must not rerun universe bootstrap.

## Step 18.3 — Declaration surface invalidation

A method body change with identical declaration signature does not rebuild class declaration surface or hierarchy.

A method signature/field/generic/superclass change invalidates the owning declaration surface and actual semantic dependents.

## Step 18.4 — Formal callable invalidation

Use Wave 4 `CallableAnalysis.dependencies` plus DB dependency edges.

Recheck:

```text
changed callable
    -> callers/consumers only if its externally observed formal product changes
```

Avoid module-wide invalidation for a private body whose resulting signature/effects/control facts remain equivalent.

## Step 18.5 — Top-level body product

Represent top-level executable statements as a deterministic synthetic `BodyId`/query product so they participate in the same invalidation machinery.

## Step 18.6 — Snapshot status

If some products are blocked/cancelled but a coherent partial snapshot can be published, use `SnapshotStatus::Partial { blocked_modules }` and explicit analysis diagnostics rather than pretending complete success.

Infrastructure failure retains last-known-good snapshot instead of publishing a corrupt partial product.

## Step 18.7 — Tests

Add counter/Arc identity tests for:

- one-method body edit;
- return type/signature edit;
- superclass edit;
- imported exported type change;
- unrelated module unchanged;
- module removal;
- cancellation/stale publication;
- Wave 4 flow result change propagating only to actual dependents.

---

# Task 19 — Integrate persistent formal semantics into the LSP worker

**Depends on:** Tasks 15–18.

**Files:**

- modify: `phalcom-lsp/src/analysis_service.rs`
- modify: `phalcom-lsp/src/semantic/snapshot.rs`
- modify: `phalcom-lsp/src/backend.rs`
- modify: `phalcom-lsp/src/perf.rs`
- create: `phalcom-lsp/tests/formal_incremental.rs`
- modify: `phalcom-lsp/tests/integration.rs`

## Step 19.1 — Worker owns one formal session

Worker state includes:

```text
ModuleWorkspaceSession
SemanticWorkspaceSession
advisory SemanticEngine
```

No request thread owns these mutable sessions.

## Step 19.2 — Replace `run_static_workspace_analysis`

On module workspace update:

```text
module delta
    ↓
formal_session.apply_update(module_snapshot, delta, cancellation)
    ↓
SemanticWorkspaceUpdate
    ↓
coherent publication
```

Delete old fresh-project/fresh-resolver/fresh-store rebuild orchestration after parity tests pass.

## Step 19.3 — Last-known-good formal snapshot

Pseudo-code:

```rust
match formal_session.apply_update(...) {
    Ok(update) => {
        last_formal = Some(update.snapshot.clone());
        publish_combined(...);
    }
    Err(error) => {
        emit_error_status_and_log(error);
        publish_advisory_with(last_formal.clone());
    }
}
```

Do not call `set_static_analysis(None)` merely because one update failed.

## Step 19.4 — Publication coherence stamp

The combined snapshot records:

```text
LSP generation
module workspace revision
formal semantic revision
advisory generation
source revision map
```

Consumers still require exact source compatibility for source-local ranges.

## Step 19.5 — Formal performance counters

Add to LSP-visible perf snapshots:

```text
formal_updates_started
formal_updates_published
formal_updates_failed
formal_queries_reused
formal_queries_recomputed
formal_modules_reused
formal_modules_rechecked
formal_callables_reused
formal_callables_rechecked
module_interfaces_reused
module_interfaces_rebuilt
module_links_reused
module_links_rebuilt
```

## Step 19.6 — Tests

A cross-module hierarchy edit must continue to clear the importer mismatch, but now assert work counters:

- provider formal surface rechecked;
- importer rechecked because hierarchy fact changed;
- unrelated module formal products reused.

A body-only literal edit in a private method should not reconstruct the project universe, module linker or unrelated formal modules.

---

# Task 20 — Project-aware startup and progressive readiness

**Depends on:** Tasks 6, 10, 15, 19 for the final version. Parts may land earlier.

**Files:**

- modify: `phalcom-lsp/src/analysis_service.rs`
- modify: `phalcom-lsp/src/workspace_scan.rs`
- modify: `phalcom-lsp/src/analysis_status.rs`
- modify: `phalcom-lsp/src/perf.rs`
- create: `phalcom-lsp/tests/project_startup.rs`
- modify: `phalcom-lsp/tests/performance.rs`

## Step 20.1 — Fast project manifest path

At workspace configuration:

```text
workspace root has project.toml
    -> load ModuleWorkspaceSession immediately on worker
    -> obtain source roots for root + dependencies
    -> schedule bounded scans of those roots
```

This avoids repeated upward project discovery per file.

## Step 20.2 — Preserve generic workspace behavior

If no manifest exists, continue using progressive recursive discovery.

Multi-root folders may contain a mixture of Phalcom Projects and loose modules.

## Step 20.3 — Readiness stages

Initial user experience target:

```text
Starting
SelectingCore (surface only)
Indexing
Ready-for-basic-editor-queries as soon as open doc + core/module surface are available
Analyzing background formal/import closure
Publishing
Ready
```

The existing status enum need not gain a separate “partial ready” phase unless UX testing proves it necessary. The key requirement is that interactive requests never block waiting for full background workspace analysis.

## Step 20.4 — Open document priority

Open-document parse/syntax diagnostics remain immediate.

Open document + transitive import closure formal analysis outranks deep analysis of unrelated closed files in `Local` mode.

## Step 20.5 — Startup tests

For a project-root workspace, assert:

- project manifest loaded once;
- source root/dependency roots known without per-file owning-project discovery;
- parser diagnostics available before full formal convergence;
- core deep callable count remains bounded/zero before first Ready;
- hover/completion remain responsive during a 2k-file progressive scan.

---

# Task 21 — Complete module/import/error diagnostics

**Depends on:** Tasks 10, 15.

**Files:**

- modify: `phalcom-semantic/src/diagnostic.rs` if additional codes are required
- modify: `phalcom-modules/src/workspace.rs`
- modify: `phalcom-lsp/src/analysis_service.rs`
- modify: `phalcom-lsp/src/diagnostics.rs`
- create: `phalcom-lsp/tests/module_diagnostics.rs`
- modify: `phalcom-lsp/tests/integration.rs`

Existing stable codes already include:

```text
project.load.failed
module.interface.failed
module.import.unresolved
module.link.failed
module.runtime_cycle
```

Use them instead of silently dropping failures.

## Step 21.1 — Unresolved import diagnostic

For:

```phalcom
import completely_nonexistent.foo
```

publish `module.import.unresolved` at the import path range.

Include help when canonical roots are known, e.g. available root names, but do not flood diagnostics with every module candidate.

## Step 21.2 — Exposure diagnostic

For an unexposed dependency child, preserve the canonical `ModulePathNotExposed` distinction and explain the relevant project/package exposure rule.

## Step 21.3 — Interface/link errors

Convert attributable interface/link failures into module-owned diagnostics.

Infrastructure failures still produce analysis Error/log events.

## Step 21.4 — Recovery

When the user creates/fixes the missing module/export, the diagnostic clears through normal module-session invalidation without a full workspace rebuild.

---

# Task 22 — Finish VS Code extension integration

**Depends on:** Tasks 1–2, 9, 14; module completion itself is standard LSP and requires no custom extension provider.

**Files:**

- modify: `tools/vsphalcom/src/analysisStatus.ts`
- modify: `tools/vsphalcom/src/extension.ts`
- modify: `tools/vsphalcom/package.json`
- modify: `tools/vsphalcom/src/test/suite/analysisStatus.test.ts`
- modify: `tools/vsphalcom/src/test/suite/lsp.e2e.test.ts`
- create: `tools/vsphalcom/src/test/suite/analysisLog.test.ts`

## Step 22.1 — Status UX

Preserve the current useful labels, but ensure the extension sees actual `Ready` after edits.

Tooltip adds:

```text
session
sequence
generation
module/formal revision if exposed
last update duration
files indexed/analyzed
```

Do not display “Ready” when the latest worker status is Error.

## Step 22.2 — Analysis Output

Subscribe to structured analysis logs and append them to the existing output channel.

The “Show Language Server Output” command remains the user entry point.

## Step 22.3 — Virtual builtin documents

Register the `phalcom` content provider and route virtual source requests through the active language client.

Read-only source should still receive Phalcom syntax highlighting because the returned URI/language document is Phalcom content.

## Step 22.4 — Configuration restart semantics

Settings that require semantic-session rebuild:

```text
phalcom.lsp.sysrootPath
phalcom.analysis.mode
phalcom.analysis.exclude
```

should trigger a workspace analysis reconfiguration through `didChangeConfiguration`; they should not necessarily restart the whole language-server process.

Binary path/enabled settings may still restart client lifecycle.

## Step 22.5 — E2E tests

Extension-host test:

- open a project fixture;
- observe Ready;
- edit source and observe Updating -> Ready;
- receive type mismatch squiggle;
- request import completion;
- go to builtin definition and open virtual/physical source;
- verify analysis log contains session/batch events.

---

# Task 23 — Compiler/LSP semantic parity tests

**Files:**

- create: `phalcom-core/tests/compiler_lsp_parity.rs` or integrate under existing test harness
- create: `phalcom-lsp/tests/compiler_parity.rs`
- modify: `phalcom-lsp/tests/integration.rs`

Use shared fixture projects and assert that compiler and LSP agree on:

- import resolution;
- exposed/private modules;
- exported names;
- nominal type resolution;
- superclass relationships;
- binding initializer mismatch code;
- return/field/argument mismatch codes;
- no diagnostic for a valid fixed program.

The comparison should use stable codes/module identities rather than exact ANSI/text rendering.

Add the hard invariant test:

> Every import-path completion candidate selected from a snapshot can be resolved by canonical `ModuleResolver` under that snapshot.

---

# Task 24 — Performance acceptance and regression gates

**Files:**

- modify: `phalcom-lsp/src/perf.rs`
- modify: `phalcom-lsp/tests/performance.rs`
- create: `phalcom-semantic/tests/performance_structure.rs`
- create: `phalcom-modules/tests/performance_structure.rs`
- modify: `tools/vsphalcom/src/test/suite/lsp.e2e.test.ts` to cover responsiveness-visible behavior

## 24.1 Structural CI gates

These are mandatory deterministic assertions.

### Cold project startup

Before first stable Ready in default Local mode:

```text
core deep flow solve: zero or explicitly bounded bootstrap exceptions
project universe constructions: one per project-session configuration
module resolver/session constructions: one
query-path disk reads: zero
query-path canonicalizations: zero
```

### Body-only edit

For a one-method body edit that changes no public declaration/interface:

```text
module relinks: 0
module interfaces rebuilt: owning module only if parse/interface fingerprint computation requires it; linked interface reused
universe bootstrap reruns: 0
unrelated formal modules rechecked: 0
formal changed callable seeded: 1
formal propagation: only actual dependency closure
```

### Import/export change

Rebuild only affected module interface/link products and reverse dependents.

### Rapid edits

Twenty edits within coalescing window:

- intermediate source updates are coalesced/discarded as designed;
- no stale semantic snapshot is published;
- final source revision wins;
- status returns Ready.

## 24.2 Wall-clock benchmark harness

Run in `--release` on a documented reference development machine:

```bash
cargo test -p phalcom-lsp --test integration \
  performance::perf_local_and_workspace_convergence \
  --release -- --ignored --nocapture

cargo test -p phalcom-lsp --test integration \
  performance::perf_hover_during_progressive_scan \
  --release -- --ignored --nocapture
```

Add a real Project fixture benchmark, not only loose files.

## 24.3 Initial performance objectives

These are engineering objectives, not brittle shared-CI assertions until reference measurements stabilize:

- small project initial interactive readiness: **< 1 second** in release build after process startup on the reference machine;
- small project fully formal Ready: **< 1.5 seconds** where project size permits;
- body-only formal recomputation worker time after debounce: **< 100 ms p50**, **< 250 ms p95** for ordinary local edits;
- hover/completion request execution against a published snapshot: **single-digit milliseconds p50**, with zero disk I/O;
- 2,048-file progressive scan must not materially block an already-open document hover/completion request.

If these are missed, use counters to identify work amplification before micro-optimizing data structures.

## 24.4 Measure the reported cold-start defect

Record before/after values for:

```text
core_select_analyze duration
core callables analyzed
flow passes
workspace scan start time
first Ready time
formal full Ready time
```

The ~21–22 second deep-core phase must disappear from normal startup.

---

# 10. Detailed invalidation rules

The implementation is not complete until these rules are encoded and tested.

## 10.1 Source body change

```text
source bytes changed
interface fingerprint unchanged
callable declaration fingerprint unchanged
specific callable body fingerprint changed
```

Invalidate:

```text
CallableBody(changed)
its dynamic formal dependency reverse closure IF output fingerprint changes
module diagnostics/presentation consuming changed analysis
```

Do not invalidate:

```text
project universe
module linked interface
unrelated declaration surfaces
unrelated callable bodies
whole core
```

## 10.2 Callable declaration/signature change

Invalidate:

```text
owner DeclarationSurface
callable body
call sites/signature consumers
module interface if exported surface changes
formal reverse dependencies
presentation/diagnostics
```

## 10.3 Superclass change

Invalidate:

```text
owner declaration surface
hierarchy relation product
subclass/hierarchy-dependent relations
callable dispatch consumers whose lookup may change
importers only if exposed semantic surface changes/they consume affected declaration
```

## 10.4 Import change

Invalidate:

```text
module interface/link binding layout
name/type resolver dependencies within importing module
runtime/semantic module graph edge
affected formal bodies
```

Do not invalidate unrelated modules outside reverse import/query dependency closure.

## 10.5 Export/re-export/expose change

Invalidate:

```text
linked interface of provider
reverse interface dependents
module completion index entries
module navigation origin maps
formal consumers of removed/changed exported declarations
```

## 10.6 File add/remove

Repair only importers whose retained logical path could match the provider, using canonical module workspace candidate indexes.

## 10.7 Core/native surface change

This is broad because core can affect many declarations. Treat it as a deliberate session-level semantic surface invalidation, but do not conflate ordinary user edits with core changes.

---

# 11. Failure model

## 11.1 Source errors are products, not infrastructure failures

Examples:

```text
parse syntax errors
unresolved import
private/unexposed module path
unresolved type annotation
type mismatch
return mismatch
```

These produce diagnostics while analysis may still publish a coherent partial/invalid semantic snapshot.

## 11.2 Infrastructure failures are status/log failures

Examples:

```text
unreadable project manifest due unexpected I/O
internal semantic invariant violation
linker internal failure not attributable to source rule
budget exhaustion
cancellation superseded by newer revision
```

Handling:

```text
cancelled/stale -> normal control event, not user-facing fatal error
budget exceeded -> explicit analysis diagnostic/status, retain coherent prior products
internal failure -> Error status + structured log, retain last-known-good snapshot
```

## 11.3 Never clear valid state because a refresh failed

This prohibition applies specifically to the current `static_snapshot: Option<_>` behavior. A failed formal update is not equivalent to “the workspace has no formal semantics.”

---

# 12. Diagnostic presentation contract

## 12.1 LSP

Syntax diagnostics:

```text
source = phalcom
```

Formal semantic diagnostics:

```text
source = phalcom-typecheck
code = stable DiagnosticCode string
```

Module/project semantic diagnostics may use the same formal source or a dedicated `phalcom-modules` source only if stable UX warrants it. Prefer one language diagnostic pipeline with stable codes.

Related information uses each label's real module URI.

## 12.2 CLI

Text output must show:

- severity/code;
- source file/module;
- primary span;
- same-file secondary labels;
- cross-file secondary locations;
- notes;
- helps.

## 12.3 VS Code

No extension-side parsing of diagnostic messages. VS Code consumes LSP structured diagnostics only.

---

# 13. Module intelligence contract

## 13.1 Import roots

At module `app.main`, completion after `import ` may include only canonical roots recognized for that project:

```text
app namespace
path/package dependency aliases
std
universe
```

## 13.2 Absolute module children

After:

```phalcom
import geometry.
```

only externally exposed children are offered for cross-project paths.

## 13.3 Relative module children

After:

```phalcom
import .
import ..
```

candidates are computed from the current module/package path using canonical relative-import semantics.

## 13.4 Selective import exports

After:

```phalcom
from geometry.point import (
```

suggest from the target `LinkedModuleInterface::exports`, including public re-exports, not all declarations in the source file.

## 13.5 Whole-module alias member completion

Given:

```phalcom
import geometry.point as point

point.
```

completion uses module exports.

It must not run ordinary class dispatch against a fake module class.

## 13.6 Navigation

Imported/re-exported values navigate to the declaration origin represented by linked export metadata. Module path components navigate to module/package source.

---

# 14. Formal/advisory editor contract

Phalcom intentionally has useful runtime-shape inference in addition to the formal type system. Preserve that capability without confusing the user.

Example:

```phalcom
let x = somethingDynamic()
```

Possible editor state:

```text
formal:   Unknown(DynamicMessageSend)
advisory: exact runtime shape CellNum from trusted/local flow evidence
```

The editor may show:

```text
x ≈ CellNum
```

but must not show:

```text
x: CellNum
```

unless formal type analysis established `CellNum`.

Likewise, a formal mismatch is never suppressed because advisory inference happens to look plausible.

---

# 15. Test matrix

## 15.1 Parser diagnostics

- multiple syntax errors in one file;
- syntax + formal semantic errors coexist;
- syntax-invalid current revision suppresses stale formal diagnostics;
- fixed syntax clears parser diagnostics.

## 15.2 Formal type diagnostics

- binding initializer;
- assignment;
- return;
- field;
- call argument;
- annotation/kind/application errors;
- Wave 4 refinement/mutation effects;
- dynamic/unknown does not falsely reject.

## 15.3 Modules

- project root import;
- dependency alias import;
- relative import;
- package exposure;
- selective import;
- re-export;
- invalid import diagnostic;
- newly-created module repairs import;
- module removed invalidates importer;
- completion parity with resolver.

## 15.4 Navigation

- same-file class/member;
- cross-file class/member;
- module path;
- imported symbol;
- re-export origin;
- builtin/core class;
- builtin/core method;
- virtual source document;
- configured physical sysroot precedence.

## 15.5 Inlay hints

- unannotated binding formal hint;
- annotated binding suppressed;
- annotated field suppressed;
- annotated parameter suppressed;
- annotated return suppressed;
- formal Known preferred;
- advisory fallback marked `≈`;
- stale formal facts not rendered.

## 15.6 Status/logging

- scan Ready;
- edit Ready;
- edit during scan;
- stale batch;
- formal update failure;
- module resolution failure;
- status session/sequence monotonic;
- extension ignores stale status.

## 15.7 Incrementality

- body-only single callable;
- callable signature change;
- superclass change;
- import/export change;
- unrelated module reuse;
- rapid edits/cancellation;
- stable TypeStore identity;
- old snapshot remains readable;
- last-known-good formal snapshot after failure.

---

# 16. Recommended implementation/commit order

Because the repository is under concurrent Wave 4 development, use this ordering.

## Track A — safe immediately

1. Task 1 — status lifecycle.
2. Task 2 — structured logs/failure visibility.
3. Task 3 — analyzer/compiler responsibility.
4. Task 4 — diagnostic source ownership/CLI rendering, avoiding Wave-4-active files until merge.
5. Task 5 — annotation inlay suppression.
6. Task 6 — core surface-only startup.
7. Tasks 10–12 — module workspace + module completion.
8. Task 14 — core navigation/virtual source.

## Track B — begins after Wave 4 merge

9. Task 0 final post-Wave-4 re-grounding.
10. Task 7 — callable analysis publication.
11. Task 8 — formal presentation.
12. Task 9 — formal-first editor presentation.
13. Tasks 16–18 — semantic DB + persistent store + formal incrementality.
14. Task 19 — LSP formal session integration.

## Track C — integration completion

15. Task 13 — module occurrences/navigation.
16. Task 15 — remove old LSP static rebuild orchestration.
17. Task 20 — project-aware startup finalization.
18. Task 21 — module diagnostics.
19. Task 22 — VS Code completion.
20. Task 23 — parity tests.
21. Task 24 — performance gates.

Do not delete old compatibility paths until replacement parity tests are green.

---

# 17. Verification commands

## 17.1 Semantic

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test spec04_5_flow_graph
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
```

## 17.2 Modules

```bash
cargo test -p phalcom-modules
```

## 17.3 Core/compiler

```bash
cargo test -p phalcom-core --test integration -- --nocapture
cargo test -p phalcom-core
```

## 17.4 LSP

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-lsp --test integration -- --test-threads=2
cargo test -p phalcom-lsp
```

## 17.5 Extension

```bash
cd tools/vsphalcom
npm test
npm run package
```

## 17.6 Manual compiler diagnostics

```bash
cargo run -p phalcom-core --bin phalcom -- check \
  --source 'const count: String = 1'

cargo run -p phalcom-core --bin phalcom -- check \
  --format json \
  --source 'const count: String = 1'
```

## 17.7 Performance

```bash
cargo test -p phalcom-lsp --test integration \
  performance::perf_local_and_workspace_convergence \
  --release -- --ignored --nocapture

cargo test -p phalcom-lsp --test integration \
  performance::perf_hover_during_progressive_scan \
  --release -- --ignored --nocapture
```

Manual server tracing:

```bash
PHALCOM_LSP_PERF=1 phalcom-lsp
```

## 17.8 Repository architecture graph

```bash
graphify update .
```

---

# 18. Completion criteria

This implementation program is complete only when all of the following are true.

## Compiler

- `ProgramAnalyzer` returns invalid-but-analyzed snapshots.
- `ProgramCompiler` rejects semantic errors before code generation.
- `phalcom check` displays rich source-aware formal diagnostics.
- JSON diagnostics are structured and contain ranges/labels.
- compiler and LSP use canonical module semantics.

## Formal semantic engine

- Wave 4 flow is the only formal flow implementation.
- `CallableAnalysis` is populated and published.
- formal type presentation is deterministic and source indexed.
- the existing semantic DB is actively used for reuse/invalidation.
- TypeStore identity is stable across workspace revisions.
- body-only edits do not rebuild the whole formal world.
- unaffected products are structurally reused.

## LSP

- parser and semantic diagnostics coexist and remain revision safe.
- module/import failures are visible.
- import path completion works for absolute, relative and selective imports.
- completion honors package exposure and linked exports.
- module aliases complete exports.
- module/import definition navigation works.
- core/builtin classes and source-declared methods are navigable.
- formal inlays/hover use canonical formal facts.
- advisory runtime shapes are visibly distinct.
- explicit annotations never receive duplicate inferred hints.
- edit-only analysis returns to Ready.
- failures preserve last-known-good formal state and are logged.
- request paths do zero disk I/O/canonicalization.

## VS Code extension

- status is monotonic and truthful.
- output channel receives structured analysis events.
- virtual builtin documents open correctly.
- standard LSP completion/navigation/diagnostics require no duplicated extension logic.

## Performance

- the reported 21–22 second deep-core startup phase is eliminated from normal startup.
- project/module infrastructure is persistent within a workspace session.
- formal body rechecking is dependency-bounded.
- structural performance assertions are enforced in tests.
- wall-clock harness demonstrates responsive startup/edit behavior on the reference machine.

---

# 19. Explicit non-goals

Do not expand this program into unrelated language features.

This plan does **not** require:

- a new runtime type object model;
- changing `.class` or the metaclass tower;
- type-based runtime dispatch;
- whole-program ahead-of-time specialization;
- persistent on-disk semantic caches;
- remote/distributed indexing;
- a Salsa dependency;
- incremental text parsing as a prerequisite;
- semantic token delta protocol;
- rewriting tower-lsp;
- replacing the advisory runtime-shape engine with formal types;
- navigating native-only Rust implementations through `Go to Implementation` (useful later, not required here);
- changing Wave 4's ratified protocol-only iteration decision.

---

# 20. Risks and mitigation

## Risk 1 — TypeStore structural sharing becomes too invasive

Mitigation: land correctness-first persistent store identity and formal query reuse first, measure snapshot-freeze copy cost, then complete structural-sharing storage behind unchanged semantic APIs.

Do not fall back to fresh TypeStore per revision.

## Risk 2 — Module workspace session duplicates linker logic

Mitigation: the session owns caches/orchestration only. Exact path semantics remain in `ModuleResolver`; exports/bindings remain in `ModuleLinker`/interfaces.

## Risk 3 — Formal/advisory facts disagree in UI

Mitigation: formal facts are labeled authoritative; advisory facts use a different visual marker and tooltip. Never automatically “upgrade” advisory evidence to formal type evidence.

## Risk 4 — Wave 4 API changes during implementation

Mitigation: enforce Task 0 re-grounding and restrict concurrent changes to non-flow files until Wave 4 lands.

## Risk 5 — Last-known-good formal snapshot hides a current failure

Mitigation: strict source revision guards prevent stale source-local formal facts from being shown as current. Status/logs expose the current failure. Last-known-good remains available only where coherent/global products are safe or until a fresh snapshot succeeds.

## Risk 6 — Virtual builtin ranges are wrong because combined core is synthetic

Mitigation: virtual definition locations use module-specific builtin source and module-specific parsed ranges from `BuiltinProjectSourceProvider`, never the combined advisory-core offsets.

## Risk 7 — Project-aware scan misses loose files

Mitigation: project source roots are the semantic source set for that Project. Additional workspace roots/non-project folders still use generic progressive discovery and workspace symbols can include indexed loose sources according to configured policy.

---

# 21. Architectural outcome

After completion, Phalcom should have the following end-to-end behavior.

A user opens VS Code at:

```text
my-app/
  project.toml
  src/
    package.ph
    main.ph
    models.ph
```

The server:

1. recognizes `project.toml` immediately;
2. loads one persistent `ProjectUniverse` / module workspace;
3. loads core declaration/native surfaces without deep-solving thousands of core bodies;
4. starts bounded source-root scanning;
5. parses the open document immediately and publishes syntax diagnostics;
6. links its import closure through canonical `phalcom-modules` semantics;
7. incrementally updates formal `phalcom-semantic::SemanticDb` products;
8. publishes a coherent immutable workspace snapshot;
9. returns Ready;
10. serves hover/completion/definition/inlays from memory-only pinned snapshots.

When the user types:

```phalcom
import geometry.
```

completion comes from canonical package exposure and can never suggest `private_tool` if the compiler would reject it.

When the user writes:

```phalcom
const count: String = 1
```

both CLI and VS Code report the same stable formal diagnostic code and source ranges.

When the user writes:

```phalcom
let count = 1
```

formal analysis can show:

```text
: Int
```

while an advisory-only runtime shape, when formal knowledge is unavailable, appears distinctly:

```text
≈ SomeRuntimeShape
```

When the user invokes Go to Definition on `Int` or a source-declared builtin method, the editor opens the real physical core source if available, otherwise a read-only virtual canonical builtin source document.

When the user changes one method body, Phalcom does not rebuild the universe, project graph, linker, type store and every module. It invalidates the changed callable/formal query, follows recorded semantic dependents only when the resulting product changes, atomically publishes the new revision, updates diagnostics/inlays that actually changed, and returns to Ready.

That is the required compiler/LSP/IDE integration boundary for Phalcom's type-system implementation to be considered complete and scalable.
