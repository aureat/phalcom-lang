# Phalcom Part 3 Persistent Workspace and LSP Cutover Implementation Plan

> For agentic workers: REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking. Do not collapse migration, deletion, UX, native-contract repair, and final acceptance into one commit.

Goal: complete Part 3 of the single-world semantic takeover so one persistent compiler module/session lifecycle publishes one immutable semantic snapshot consumed by every semantic LSP feature, with practical IDE presentation and formally correct native fixed returns.

Architecture: phalcom-modules owns project, source, module, overlay, linking, and module-query identity. phalcom-semantic owns one persistent SemanticWorkspaceSession, formal and advisory products, invalidation, source indexes, presentation views, and atomic immutable SemanticSnapshot publication. phalcom-lsp retains protocol adaptation, document buffering, syntax recovery, scheduling, rendering, and notifications; it performs no semantic analysis or semantic identity construction.

Tech Stack: Rust 2024 workspace; phalcom-modules; phalcom-semantic; phalcom-core native metadata/runtime; phalcom-lsp; tower-lsp; cargo test; cargo check; cargo fmt; graphify.

Spec: docs/impl/semantic/semantic-correctness/part-3/phalcom_semantic_correctness_single_world_takeover_part3_persistent_workspace_lsp_cutover_professional_ide_spec.md

## Global Constraints

- Parts 1 and 2 are hard prerequisites. Do not weaken their formal/advisory, identity, invalidation, or snapshot contracts to make Part 3 pass.
- Part 3 is complete only when every one of the 90 gates in section 88 is evidenced.
- phalcom-modules remains the sole authority for project identity, source identity, module resolution, linking, imports, exposure, and source overlays.
- phalcom-semantic remains the sole formal/advisory semantic owner and retains one TypeStore across ordinary workspace revisions.
- Every semantic LSP request pins exactly one Arc<phalcom_semantic::SemanticSnapshot> and uses it for the entire request.
- Formal TypeKnowledge and advisory ValueShape remain separate; advisory facts cannot reject code, participate in proof, or replace established formal knowledge.
- Ordinary IDE labels use canonical Phalcom type spelling. Production phalcom-lsp/src contains no advisory pseudo-type glyph, Confidence taxonomy, or Observed type/return boilerplate.
- Semantic errors publish current semantic products. Cancellation, budget exhaustion, and stale candidates retain the last-known-good publication and never publish a partial candidate.
- Compiler/module crates do not depend on tower_lsp::Url or other protocol DTOs.
- Request handlers perform indexed immutable reads and rendering only. They do not load files, resolve modules from the filesystem, run formal analysis, run advisory solving, rebuild surfaces, or scan the workspace.
- Syntax recovery may remain in LSP only when it identifies incomplete source shape and delegates meaning to compiler products.
- Preserve unrelated dirty, staged, deleted, and untracked work. Never reset, clean, broad-stage, or overwrite parallel-owned changes.
- Run graphify query before architecture navigation and graphify update . after code changes; inspect graphify-out status immediately after update.

## Live baseline and reconciliation

The checkout already contains a partial Part 3 slice. The first implementation action must re-check current source and tests rather than recreate existing work.

Current evidence:

- phalcom-modules/src/session.rs already defines WorkspaceModuleSession, WorkspaceSourceMutation, WorkspaceSourceState, overlay precedence, source removal, and linked products.
- phalcom-semantic/src/session.rs already retains WorkspaceModuleSession, SemanticDb, TypeStore, last snapshot, and last-known-good snapshot.
- phalcom-semantic/src/snapshot.rs already publishes source, formal, advisory, module, diagnostic, and occurrence products.
- phalcom-lsp/src/presentation.rs and initial professional_semantic_presentation tests already exist, but the renderer still exposes generic analyzer prose rather than the final contextual wording policy.
- phalcom-core/src/primitive/system.rs and docs/spec/current/system.md currently show System.print returning Unit; native-contract tests and generic formal propagation still require live verification.
- phalcom-lsp/src/analysis_service.rs still contains StaticWorkspaceIdentity, run_static_workspace_analysis, and engine.set_static_analysis.
- phalcom-lsp/src/backend.rs still owns WorkspaceIndex and composes nested static_snapshot state.
- phalcom-lsp/src/semantic/ still contains the duplicate semantic implementation.

No current item is marked complete in the checklist solely because a symbol exists. Each gate needs focused evidence against the live checkout and the predecessor release gates.

## Ownership map

Create or extend:

- phalcom-modules/src/session.rs: persistent source/project/module lifecycle and module publication.
- phalcom-semantic/src/session.rs: lifecycle-to-semantic update API, TypeStore reuse, publication effects, cancellation, and last-known-good handling.
- phalcom-semantic/src/snapshot.rs and phalcom-semantic/src/presentation.rs: one coherent immutable world and bounded read-only presentation views.
- phalcom-core/src/primitive/system.rs and docs/spec/current/system.md: canonical native contracts.
- phalcom-lsp/src/analysis_service.rs: worker scheduling around one compiler session and one published snapshot.
- phalcom-lsp/src/request_context.rs: one pinned snapshot plus source-revision match state.
- phalcom-lsp/src/presentation.rs: pure protocol wording and label rendering.
- phalcom-lsp/src/hover.rs, inlay_hints.rs, signature_help.rs, completion.rs, diagnostics.rs, import_completion.rs, semantic_tokens.rs: protocol consumers of compiler products.

Delete after replacement tests pass:

- phalcom-lsp/src/semantic/* duplicate semantic authority.
- phalcom-lsp/src/index.rs WorkspaceIndex semantic authority.
- static workspace reconstruction and nested static_snapshot bridges.

## Execution order and review gates

Each task below ends with a focused test gate. Commit each cohesive task separately. A task may use the existing partial implementation as its starting point, but it must not mark a release gate complete until its listed negative audit and predecessor dependencies pass.

### Task 0: Re-ground predecessor contracts and current worktree

Files:

- Read: the Part 1 spec, Part 1 corrections/amendments, Part 2 spec, Part 2 checklist, and this Part 3 spec.
- Inspect: graphify query for SemanticWorkspaceSession, WorkspaceModuleSession, SemanticSnapshot, StaticWorkspaceIdentity, WorkspaceIndex, and native return registration.
- Inspect: current git status, git diff --check, current test registrations, and existing partial Part 3 files.

Interfaces:

- Consumes: current repository and predecessor documents.
- Produces: an evidence table in the checklist that separates landed, partial, unverified, blocked by predecessor, and pending.

Steps:

- [ ] Confirm Part 1 and Part 2 release-gate state from live checklist/test evidence; do not infer completion from commit messages.
- [ ] Run the narrow baseline tests for phalcom-modules, phalcom-semantic, phalcom-core native contracts, and registered phalcom-lsp presentation/single-world tests.
- [ ] Record unrelated worktree changes and avoid overlapping ownership boundaries.
- [ ] Stop implementation at any predecessor gate that is genuinely missing; report it as a dependency rather than recreating its semantic authority in LSP.

Focused gate:

    cargo test -p phalcom-modules
    cargo test -p phalcom-semantic --test semantic
    cargo test -p phalcom-core --test native_contracts
    cargo test -p phalcom-lsp --test professional_semantic_presentation
    cargo test -p phalcom-lsp --test single_world_cutover

### Task 1: Lock professional presentation and native contract regressions

Files:

- Modify: phalcom-lsp/tests/professional_semantic_presentation.rs
- Modify: phalcom-lsp/src/presentation.rs
- Modify: phalcom-lsp/src/hover.rs, phalcom-lsp/src/inlay_hints.rs, phalcom-lsp/src/signature_help.rs
- Modify: phalcom-core/tests/native_contracts.rs
- Modify: docs/spec/current/system.md only when its live wording differs from the ratified Unit contract

Interfaces:

- Consumes: compiler formal/advisory presentation views and native surface metadata.
- Produces: failing-to-green user-visible assertions for plain labels, contextual evidence, and System.print/System.gc contract coherence.

Steps:

- [ ] Add table-driven presentation cases for established, assumed, advisory-only, narrowed, generic-specialized, unknown, dynamic, and formal/advisory disagreement views.
- [ ] Assert ordinary binding labels use colon syntax, return labels use arrow syntax, signature help has no glyph, and completion detail has no per-item epistemic decoration.
- [x] Assert hover primary lines use canonical type spelling and only show evidence lines for materially useful context.
- [x] Assert no production renderer emits the forbidden glyph, Confidence:, Observed type:, Observed return:, or repetitive inferred-by-Phalcom prose.
- [x] Assert native metadata and runtime behavior agree for System.print returning Unit and System.gc returning None.

Focused gate:

    cargo test -p phalcom-lsp --test professional_semantic_presentation -- --nocapture
    cargo test -p phalcom-core --test native_contracts -- --nocapture
    rg -n '≈|Observed type:|Observed return:|Confidence:|Inferred runtime value:' phalcom-lsp/src

### Task 2: Finish generic trusted fixed-return proving

Files:

- Modify: phalcom-semantic/src/types/native.rs
- Modify: phalcom-semantic/src/checker/call.rs
- Modify: phalcom-semantic/src/checker/body.rs
- Create or extend: phalcom-semantic/tests/trusted_return_contracts.rs
- Verify: phalcom-core/src/primitive/system.rs and docs/spec/current/system.md

Interfaces:

- Consumes: native surface registration, CallableSemanticSignature, TypeKnowledge, call expression analysis, normal return summary.
- Produces: Established formal return knowledge before advisory fallback for every canonical fixed-return contract.

Steps:

- [ ] Add the failing System.print tail-return fixture and table-driven fixed-return fixtures for Unit, ordinary nominal, Never, receiver, argument, declared source, constructor Self, and generic-specialized returns.
- [x] Trace native metadata through register_native_surfaces into callable signatures and call expression facts.
- [x] Implement generic formal-first precedence in the shared call-result path; do not special-case print(_).
- [x] Propagate established call facts through normal_return_summary and caller return presentation.
- [ ] Preserve advisory disagreement as non-authoritative evidence and assert it cannot replace formal Unit.

Focused gate:

    cargo test -p phalcom-semantic --test trusted_return_contracts -- --nocapture
    cargo test -p phalcom-core --test native_contracts -- --nocapture

### Task 3: Complete persistent WorkspaceModuleSession lifecycle

Files:

- Extend: phalcom-modules/src/session.rs
- Modify: phalcom-modules/src/lib.rs and phalcom-modules/src/source.rs only for session/provider integration
- Create or extend: phalcom-modules/tests/workspace_session.rs

Interfaces:

- Consumes: ProjectUniverse, discover_owning_project, FilesystemSourceProvider, OverlaySourceProvider, SourceOverlay, ModuleResolver, InterfaceBuilder, ModuleLinker, ModuleId, SourceId, and SourceLocation.
- Produces: WorkspaceModuleSession::set_workspace_roots, apply, set_overlay, remove_overlay, refresh_disk, remove_source, and one WorkspaceModuleUpdate.

Steps:

- [x] Test project-backed body edits retain ProjectSourceIdentity, ResolvedProjectId, ModuleId, and the same TypeStore-independent source identity.
- [x] Test standalone open/edit/close/reopen/delete identity rules and prove unrelated standalone files never reuse the deleted synthetic identity.
- [x] Test open overlay precedence, watched disk updates while open, close-to-disk restoration, and no fake URI module.
- [x] Test source removal clears reverse mappings, linked products, and module query products.
- [ ] Test explicit project-root/config changes rebuild the project graph while ordinary body edits do not.
- [ ] Add bounded batch mutation behavior so multiple overlays link once per workspace update.

Focused gate:

    cargo test -p phalcom-modules --test workspace_session -- --nocapture

### Task 4: Publish one semantic workspace atom with effects

Files:

- Modify: phalcom-semantic/src/session.rs
- Modify: phalcom-semantic/src/snapshot.rs
- Modify: phalcom-semantic/src/db/fingerprint.rs and phalcom-semantic/src/db/key.rs when lifecycle fingerprints require new query keys
- Create or extend: phalcom-semantic/tests/semantic_single_world.rs and phalcom-semantic/tests/type_store_revisions.rs

Interfaces:

- Consumes: WorkspaceModuleUpdate, SemanticWorkspaceInput, QueryBudget, CancellationToken, existing query/fingerprint machinery, and the retained TypeStore.
- Produces:

    pub struct SemanticPublicationEffects {
        pub diagnostics_changed: BTreeSet<ModuleId>,
        pub source_index_changed: BTreeSet<ModuleId>,
        pub formal_changed: BTreeSet<ModuleId>,
        pub advisory_changed: BTreeSet<ModuleId>,
        pub declaration_index_changed: bool,
        pub module_graph_changed: bool,
    }

    pub struct SemanticWorkspacePublication {
        pub snapshot: Arc<SemanticSnapshot>,
        pub invalidated: Arc<[QueryKey]>,
        pub recomputed: Arc<[QueryKey]>,
        pub stats: SemanticUpdateStats,
        pub effects: SemanticPublicationEffects,
    }

    Preserve SemanticWorkspaceUpdate as a type alias or compatibility name only while all production callers move to SemanticWorkspacePublication.

Steps:

- [x] Derive effects from compiler fingerprints and publish all source, module, formal, advisory, diagnostic, and presentation products under one snapshot generation.
- [x] Prove ordinary body edits retain the session and TypeStore identity while recomputing only the explicit dependency closure.
- [ ] Prove source removal invalidates reverse target and importer products.
- [ ] Make candidate publication atomic: semantic errors publish current products; cancellation, budget failure, and stale candidate discard retain last-known-good.
- [x] Extend SemanticUpdateStats with project graph, relink, source-index, advisory-source, and advisory-callable observability.
- [ ] Add source-revision coherence checks required by request-time exact/stale/unmapped policy.

Focused gate:

    cargo test -p phalcom-semantic --test semantic_single_world -- --nocapture
    cargo test -p phalcom-semantic --test type_store_revisions -- --nocapture

### Task 5: Replace LSP static reconstruction with the compiler session

Files:

- Modify: phalcom-lsp/src/analysis_service.rs
- Modify: phalcom-lsp/src/backend.rs only for worker/publication handle wiring
- Modify: phalcom-lsp/src/workspace_scan.rs only for protocol-to-source event conversion

Interfaces:

- Consumes: LSP document/source events, WorkspaceSourceMutation, SemanticWorkspaceSession, SemanticWorkspacePublication, debounce, latest-wins epoch, cancellation, and status/log notifications.
- Produces: AnalysisWorkerState with one SemanticWorkspaceSession and one published Arc<SemanticSnapshot>.

Steps:

- [ ] Convert protocol URI/document events at the LSP boundary into canonical SourceLocation/SourceId mutations.
- [ ] Route open/change/close/delete/project-root updates through the persistent compiler session without rebuilding source catalogs into semantic inputs on each request.
- [x] Delete production run_static_workspace_analysis, StaticWorkspaceIdentity, StaticWorkspacePublication, refresh_static_analysis, and engine.set_static_analysis.
- [ ] Keep worker scheduling, debounce, coalescing, cancellation, status, logs, open-buffer precedence, and source transport caches.
- [x] Add a worker test that applies two edits and observes one persistent compiler session, one retained TypeStore, and stable project/module identity.

Focused gate:

    cargo test -p phalcom-lsp --test single_world_cutover -- --nocapture
    rg -n 'run_static_workspace_analysis|StaticWorkspaceIdentity|StaticWorkspacePublication|static_snapshot|formal_static_snapshot|set_static_analysis' phalcom-lsp/src

### Task 6: Pin one compiler snapshot per request and cut diagnostics

Files:

- Modify: phalcom-lsp/src/request_context.rs
- Modify: phalcom-lsp/src/backend.rs
- Modify: phalcom-lsp/src/diagnostics.rs
- Extend: phalcom-lsp/tests/single_world_cutover.rs

Interfaces:

- Consumes: DocumentSnapshot, published Arc<phalcom_semantic::SemanticSnapshot>, canonical module/source registry, and compiler diagnostics.
- Produces:

    pub enum SourceMatch {
        Exact,
        Stale,
        Unmapped,
    }

    pub struct RequestContext {
        pub document: DocumentSnapshot,
        pub semantic: Arc<phalcom_semantic::SemanticSnapshot>,
        pub module: Option<phalcom_modules::ModuleId>,
        pub source_match: SourceMatch,
    }

Steps:

- [x] Pin document and semantic snapshots once at handler entry and use the same Arc for all target, site, diagnostic, and source lookups.
- [x] Compare source revision/fingerprint and classify Exact, Stale, or Unmapped.
- [x] Render current syntax diagnostics always; append compiler semantic diagnostics only for exact source revision.
- [x] Suppress stale semantic ranges against a changed open buffer while retaining closed-file compiler diagnostics.
- [ ] Test an in-flight request holding an older Arc while a new publication occurs; the response must remain coherent against the old snapshot.

Focused gate:

    cargo test -p phalcom-lsp --test single_world_cutover -- --nocapture

### Task 7: Add compiler-owned presentation views and pure LSP renderer

Files:

- Extend: phalcom-semantic/src/presentation.rs and phalcom-semantic/src/snapshot.rs
- Extend: phalcom-lsp/src/presentation.rs
- Modify: phalcom-lsp/src/hover.rs
- Extend: phalcom-lsp/tests/professional_semantic_presentation.rs

Interfaces:

- Consumes: SemanticSiteView, formal/advisory lanes, source-site provenance, canonical type spelling, Phaldoc input, and semantic diagnostics.
- Produces:

    pub struct SemanticTypePresentation {
        pub formal: Option<FormalFactView>,
        pub advisory: Option<AdvisoryFactView>,
    }

    pub struct EvidenceSummary {
        pub kind: EvidenceSummaryKind,
        pub source: Option<SemanticSourceSpan>,
        pub related_target: Option<SemanticTarget>,
        pub description: Option<Box<str>>,
    }

    Pure LSP functions:

    pub fn render_hover(context: HoverContext<'_>) -> Option<MarkupContent>;
    pub fn render_inlay_type(view: &SemanticSiteView<'_>, kind: InlaySiteKind) -> Option<RenderedInlay>;
    pub fn render_signature(view: &CallablePresentationView<'_>) -> RenderedSignature;

Steps:

- [ ] Select primary display type formal Known first, advisory shape second, explicit Dynamic third, and no concrete label otherwise without changing compiler precedence.
- [ ] Map evidence causes to contextual prose: native signature, constructor, declaration, flow narrowing, generic specialization, call sites, pattern, unknown blocker, and explicit dynamic.
- [ ] Keep Phaldoc immediately after the primary signature and omit evidence lines that merely repeat the primary fact.
- [ ] Keep renderer free of mutable semantic state, dispatch, name resolution, inference, target manufacture, and source parsing for meaning.
- [ ] Assert formal/advisory disagreement displays formal primary type and retains advisory context only where useful.

Focused gate:

    cargo test -p phalcom-lsp --test professional_semantic_presentation -- --nocapture
    cargo test -p phalcom-semantic --test semantic -- --nocapture

### Task 8: Cut hover, inlay hints, and signature help

Files:

- Modify: phalcom-lsp/src/hover.rs
- Modify: phalcom-lsp/src/inlay_hints.rs
- Modify: phalcom-lsp/src/signature_help.rs
- Extend: phalcom-lsp/tests/stage4_hover.rs, phalcom-lsp/tests/stage6_inlay_hints.rs, phalcom-lsp/tests/signature_help.rs

Interfaces:

- Consumes: RequestContext, compiler source-site/occurrence/target queries, CallableSemanticSignature, SemanticTypePresentation, and syntax-only CallSite recovery.
- Produces: hover/inlay/signature responses with ordinary labels and contextual evidence.

Steps:

- [ ] Replace WorkspaceIndex, LSP InferredValue, formal analysis scans, and selector-string bridges with pinned compiler source-site views.
- [ ] Preserve keyword hover and Phaldoc rendering as protocol presentation inputs.
- [ ] Enumerate compiler binding/field/parameter/return sites; suppress hints for explicit annotations, unknown facts, obvious literals, and stable-policy heuristic evidence.
- [ ] Keep incomplete-call recovery syntax-only and render canonical callable signatures without filling missing parameters with fabricated Unknown types.
- [ ] Add System.print tail-return hover and return-inlay assertions with Unit and no Option.

Focused gate:

    cargo test -p phalcom-lsp --test stage4_hover -- --nocapture
    cargo test -p phalcom-lsp --test stage6_inlay_hints -- --nocapture
    cargo test -p phalcom-lsp --test signature_help -- --nocapture

### Task 9: Cut completion to canonical compiler surfaces

Files:

- Modify: phalcom-lsp/src/completion.rs
- Modify: phalcom-lsp/src/backend.rs
- Extend: phalcom-lsp/tests/semantic_completion.rs and registered completion fixtures

Interfaces:

- Consumes: syntax receiver/range recovery, pinned SemanticSnapshot source sites, formal receiver facts, advisory receiver shapes, canonical surfaces, dispatch, visibility, and module query products.
- Produces: ordinary completion items without semantic reconstruction or per-item epistemic decoration.

Steps:

- [ ] Delete shallow receiver class inference, constructor/method-return scanning, field constructor tracking, argument propagation, source ModuleSurface reconstruction, and WorkspaceIndex fallback.
- [ ] Resolve exact receiver site through compiler products; use advisory receiver shape only when formal has no concrete receiver and retain its advisory status internally.
- [ ] Preserve self, super, class-object, union, native/core, incomplete dangling-dot, and current-buffer recovery behavior.
- [ ] Return bounded lexical/global completions for stale or unmapped source rather than invoking another analyzer.
- [ ] Assert no completion label/detail contains the advisory glyph, inferred, heuristic, or advisory boilerplate.

Focused gate:

    cargo test -p phalcom-lsp --test semantic_completion -- --nocapture
    cargo test -p phalcom-lsp --test stage3_completion -- --nocapture

### Task 10: Cut definitions, references, workspace symbols, and delete WorkspaceIndex

Files:

- Modify: phalcom-lsp/src/backend.rs
- Delete after replacement tests: phalcom-lsp/src/index.rs
- Extend: phalcom-lsp/tests/stage2_index.rs, phalcom-lsp/tests/module_navigation.rs, phalcom-lsp/tests/single_world_cutover.rs

Interfaces:

- Consumes: SemanticSnapshot source index, exact/advisory target attachments, reverse target index, declaration index, canonical SourceLocation, and snapshot-local identity guards.
- Produces: definition, references, and workspace-symbol results from immutable compiler products.

Steps:

- [ ] Implement definition as source-site exact target first, advisory target second, then canonical declaration location.
- [ ] Implement references through SemanticTarget -> SourceSiteId reverse index; never rescan workspace occurrences.
- [ ] Implement workspace symbols through a sorted compiler declaration product with a fingerprint.
- [ ] Remove every semantic WorkspaceIndex consumer and its mutable class/member/selector authority.
- [ ] Delete index.rs only after all focused tests pass and no legitimate non-semantic text feature depends on it; rename any retained text-only cache to its narrow role.

Focused gate:

    cargo test -p phalcom-lsp --test stage2_index -- --nocapture
    cargo test -p phalcom-lsp --test module_navigation -- --nocapture
    rg -n 'WorkspaceIndex|definition_info|symbols_matching|class_members|is_same_or_subclass' phalcom-lsp/src

### Task 11: Cut module completion and core/native source navigation

Files:

- Modify: phalcom-lsp/src/import_completion.rs
- Modify: phalcom-lsp/src/backend.rs
- Extend: phalcom-lsp/tests/module_navigation.rs and core-startup tests

Interfaces:

- Consumes: SemanticSnapshot::module_queries, ModuleQueryFacade, canonical module declaration/source products, CallableId/DeclarationId provenance, and virtual source provider.
- Produces: import/module completion and core/native locations without filesystem semantic discovery.

Steps:

- [ ] Recover only the partially typed import path in LSP.
- [x] Ask ModuleQueryFacade for roots, children, exports, and canonical module candidates.
- [ ] Resolve native/core declaration locations from compiler provenance and use phalcom:// virtual content only as protocol adaptation.
- [ ] Assert request handlers perform no filesystem module resolution or raw URI ModuleId construction.

Focused gate:

    cargo test -p phalcom-lsp --test module_navigation -- --nocapture
    rg -n 'ModuleId::new\\(.*uri.*to_string|filesystem|read_to_string|canonicalize' phalcom-lsp/src/{backend.rs,import_completion.rs,hover.rs,completion.rs}

### Task 12: Cut semantic-token role refinement

Files:

- Modify: phalcom-lsp/src/semantic_tokens.rs
- Extend: phalcom-lsp/tests/semantic_tokens_current_syntax.rs and stage5 semantic-token tests

Interfaces:

- Consumes: lexer tokens, exact pinned compiler occurrence roles, current source match, and semantic-token wire encoding.
- Produces: lexical fallback plus compiler-owned semantic role refinement.

Steps:

- [x] Keep lexer classification for syntax without semantic identity.
- [x] Map compiler occurrences to variable, parameter, method, property, class, selector, and operator roles when source match is Exact.
- [x] Omit semantic refinement for Stale and Unmapped snapshots rather than using stale ranges or reparsing AST meaning.
- [ ] Assert semantic-token refresh notifications are driven by compiler publication effects/fingerprints, not every generation.

Focused gate:

    cargo test -p phalcom-lsp --test semantic_tokens_current_syntax -- --nocapture
    cargo test -p phalcom-lsp --test stage5_semantic_tokens -- --nocapture

### Task 13: Delete duplicate LSP semantic implementation

Files:

- Delete after parity: phalcom-lsp/src/semantic/analyzer.rs
- Delete after parity: phalcom-lsp/src/semantic/callable.rs
- Delete after parity: phalcom-lsp/src/semantic/dispatch.rs
- Delete after parity: phalcom-lsp/src/semantic/engine.rs
- Delete after parity: phalcom-lsp/src/semantic/facts.rs
- Delete after parity: phalcom-lsp/src/semantic/flow.rs
- Delete after parity: phalcom-lsp/src/semantic/ids.rs
- Delete after parity: phalcom-lsp/src/semantic/infer.rs
- Delete after parity: phalcom-lsp/src/semantic/invalidation.rs
- Delete after parity: phalcom-lsp/src/semantic/module_graph.rs
- Delete after parity: phalcom-lsp/src/semantic/occurrence.rs
- Delete after parity: phalcom-lsp/src/semantic/query.rs
- Delete after parity: phalcom-lsp/src/semantic/scope.rs
- Delete after parity: phalcom-lsp/src/semantic/snapshot.rs
- Delete after parity: phalcom-lsp/src/semantic/source.rs
- Delete after parity: phalcom-lsp/src/semantic/surface.rs
- Modify: phalcom-lsp/src/semantic/mod.rs and phalcom-lsp/src/lib.rs only to remove duplicate exports or retain a pure protocol re-export.

Interfaces:

- Consumes: all replacement compiler/module imports and passing consumer tests.
- Produces: no mutable or computed semantic authority under phalcom-lsp/src.

Steps:

- [x] Run consumer tests and forbidden-symbol scans before deletion; capture the remaining references. Remaining references are the duplicate `SemanticEngine`, `WorkspaceIndex`, and `phalcom-lsp/src/semantic/*` adapter surface.
- [ ] Remove duplicate semantic definitions and directly import compiler/module identities, snapshots, views, and query facades.
- [ ] Remove compatibility shims that still compute or cache formal/advisory semantics under a different name.
- [ ] Keep only legitimate LSP concerns listed in spec section 63.
- [x] Run graphify update . and inspect status immediately. Graph rebuilt to 85,301 nodes / 121,904 edges; no graphify files appeared in worktree status.

Focused gate:

    rg -n 'struct SemanticEngine|struct SemanticDb|struct ValueShape|struct ScopeGraph|enum SemanticTarget|struct CallableId|struct ClassId|struct FieldId' phalcom-lsp/src
    rg -n 'run_static_workspace_analysis|StaticWorkspaceIdentity|StaticWorkspacePublication|static_snapshot|formal_binding_presentation_at|formal_expression_presentation_at|WorkspaceIndex' phalcom-lsp/src

### Task 14: Add lifecycle, parity, UX, and performance acceptance suites

Files:

- Extend: phalcom-modules/tests/workspace_session.rs
- Create or extend: phalcom-semantic/tests/semantic_single_world.rs
- Create or extend: phalcom-semantic/tests/type_store_revisions.rs
- Extend: phalcom-lsp/tests/single_world_cutover.rs
- Extend: phalcom-lsp/tests/performance.rs
- Extend: examples/ide-golden/ fixtures and registered LSP integration tests
- Modify: phalcom-semantic/Cargo.toml and phalcom-lsp/Cargo.toml test registrations when autotests=false requires explicit entries

Interfaces:

- Consumes: compiler publication effects, structural counters, exact/stale/unmapped request policy, module/source identity rules, and user-visible presentation.
- Produces: cold-vs-incremental parity, lifecycle, cancellation/latest-wins, concurrent snapshot, structural performance, native end-to-end, and 16-case IDE golden evidence.

Steps:

- [ ] Compare a cold final-source session against an incremental edit sequence for identities, signatures, formal facts, diagnostics, targets, advisory shapes, completion, and presentation.
- [ ] Test open/change/close/reopen, delete, rename/move, project config change, and overlay/disk precedence.
- [ ] Test cancellation/latest-wins and semantic-error publication separately from infrastructure failure.
- [x] Assert body-only edits do not rebuild ProjectUniverse, relink unrelated modules, or recompute unrelated callables.
- [ ] Add the System.print Greeter fixture and table-driven similar fixed-return regressions.
- [ ] Add the practical IDE golden matrix from spec section 58.

Focused gate:

    cargo test -p phalcom-modules --test workspace_session -- --nocapture
    cargo test -p phalcom-semantic --test semantic_single_world -- --nocapture
    cargo test -p phalcom-semantic --test type_store_revisions -- --nocapture
    cargo test -p phalcom-lsp --test single_world_cutover -- --nocapture
    cargo test -p phalcom-lsp --test performance -- --nocapture

### Task 15: Remove obsolete bridge docs/tests and record final architecture

Files:

- Modify: phalcom-lsp/src/backend.rs, phalcom-lsp/src/analysis_service.rs, and feature-module comments.
- Modify: architecture documentation that still describes WorkspaceIndex, LSP SemanticEngine, static_snapshot, or dual-world publication as current authority.
- Port valuable behavioral coverage from deleted bridge tests to canonical compiler/snapshot tests before removal.

Interfaces:

- Consumes: passing replacement tests and final ownership map.
- Produces: repository documentation matching the final architecture in spec section 90 and no obsolete dual-world claims.

Steps:

- [ ] Remove only tests whose purpose is validating the deleted bridge; preserve behavior tests by porting them.
- [ ] Update comments to name exactly one owner for project/module identity, formal semantics, advisory semantics, source targets, invalidation, and snapshot publication.
- [x] Run section 85 UX, section 86 single-world, and section 87 native-contract audits. UX and native audits are clean; single-world audit records the remaining duplicate authority explicitly.
- [x] Run graphify update . and verify changed graph files are limited to expected updates. No graphify files appeared in worktree status.

Focused gate:

    cargo fmt --check
    cargo check --workspace
    cargo test -p phalcom-modules
    cargo test -p phalcom-semantic
    cargo test -p phalcom-core
    cargo test -p phalcom-lsp
    cargo clippy --workspace

## Required audit commands

Run after Task 13 and again before release:

    rg -n '≈|Observed type:|Observed return:|Confidence:|Inferred runtime value:' phalcom-lsp/src phalcom-lsp/tests
    rg -n 'These .* inferred by Phalcom|inferred by Phalcom' phalcom-lsp/src
    rg -n 'run_static_workspace_analysis|StaticWorkspaceIdentity|StaticWorkspacePublication|static_snapshot|formal_static_snapshot|formal_binding_presentation_at|formal_expression_presentation_at' phalcom-lsp/src
    rg -n 'struct SemanticEngine|struct SemanticDb|struct ValueShape|struct ScopeGraph|enum SemanticTarget|struct CallableId|struct ClassId|struct FieldId' phalcom-lsp/src/semantic phalcom-lsp/src
    rg -n 'ModuleId::new\\(.*uri.*to_string|build_module_surface|WorkspaceIndex' phalcom-lsp/src
    rg -n 'System.*print|print\\(_\\)' phalcom-core/src/primitive/system.rs docs/spec/current/system.md

Expected production results:

- no forbidden UX strings under phalcom-lsp/src;
- no duplicate semantic authority or static bridge symbols under phalcom-lsp/src;
- canonical System.print return metadata, runtime result, and docs all say Unit;
- System.gc remains canonical None;
- every residual match is either a negative test assertion or a reviewed protocol-only concern.

## Commit boundaries

Use one reviewable commit per cohesive slice:

1. test(lsp): lock practical semantic presentation contract
2. fix(core): make System.print canonical Unit and reconcile System.gc metadata
3. test(semantic): prove trusted fixed returns before advisory fallback
4. modules: complete persistent workspace module session
5. semantic: publish lifecycle effects from one session
6. lsp: route worker mutations through compiler session
7. lsp: publish compiler snapshot directly
8. lsp: cut diagnostics to compiler snapshot
9. lsp: centralize practical semantic presentation
10. lsp: cut hover to source-site views
11. lsp: cut inlay and signature help
12. lsp: cut completion to compiler surfaces
13. lsp: cut definition/references/workspace symbols
14. lsp: cut module/source navigation and semantic tokens
15. lsp: delete duplicate semantic authority and WorkspaceIndex
16. test: add lifecycle/parity/performance acceptance gates
17. docs: record final single-world architecture

Each commit must stage only its named cohesive work unit. Do not commit unrelated existing worktree changes.

## Completion rule

Do not call this implementation complete after a focused green slice. Completion requires all Part 1 and Part 2 predecessor gates plus all 90 Part 3 gates in the companion checklist, broad workspace gates, and manually reviewed audits.
