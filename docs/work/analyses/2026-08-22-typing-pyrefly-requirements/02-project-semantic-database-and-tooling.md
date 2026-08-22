# Specification 02: Project Semantic Database, Typed Interfaces, and Shared Tooling Snapshot

**Status:** Draft implementation specification<br>
**Depends on:** Specification 01’s persistent type identities, type-constructor descriptors, relation contracts, and annotation-lowering interface<br>
**Enables:** project/package checking, import-aware typing, incremental reuse, CLI correctness, LSP static diagnostics, and a compiler/LSP single type source of truth<br>
**Primary owners:** `phalcom-modules`, `phalcom-semantic`, `phalcom-core`, `phalcom-lsp`<br>
**Non-goal:** parallelizing all semantic queries or replacing current LSP `ValueShape` analysis

## 1. Problem statement

Phalcom has two partial semantic worlds which must converge without erasing their distinct roles.

The compiler already discovers packages/modules, creates unlinked/linked interfaces, and separates semantic from runtime dependency graphs. Project, package, and owned-module entry selections use that discovery/linking path; inline and standalone selections also run local semantic type checking ([`compile_entry_selection`](../../../../phalcom-core/src/modules/compile.rs#L125-L450)). The linked interfaces only carry declaration/import/export linkage ([`interface.rs`](../../../../phalcom-modules/src/interface.rs)); they do not expose resolved type parameter descriptors, superclass/protocol headers, generic member surfaces, or diagnostic ownership. Therefore a body checker cannot resolve imported types or members from a stable header environment.

The present `run_semantic_typecheck` creates a new store, a local parent map, a string resolver, and a checker for one parsed program ([`compile.rs`](../../../../phalcom-core/src/modules/compile.rs#L513-L552)). It uses `ModuleId::core()` for the checked source. Its facts cannot be reused by a project compiler or editor, cannot be invalidated by a changed interface, and cannot represent a module cycle honestly.

The LSP has a semantic-diagnostic adapter but publishes only syntax errors ([adapter](../../../../phalcom-lsp/src/diagnostics.rs#L35-L74), [publication](../../../../phalcom-lsp/src/backend.rs#L298-L315)). Its `ValueShape` system should stay because it already powers editor completion/hover/inlays and implements bounded/cancellable advisory analysis. It is explicitly not Phalcom’s static type system ([`ValueShape`](../../../../phalcom-lsp/src/semantic/facts.rs)); publishing it as static typing would create the prohibited editor-only checker.

This specification builds one revisioned semantic database that turns linked module structure into typed interface and body products, then publishes immutable snapshots for compiler, CLI, LSP, and later proof consumers.

## 2. Architectural constraints

1. **One formal type checker.** `phalcom-semantic` owns static type facts. Neither CLI nor LSP may recreate relation, generic, or flow rules in an adapter.
2. **Headers before bodies.** Types named across modules, superclass/protocol declarations, and member signatures must be available from typed interface headers even if bodies form recursive semantic SCCs.
3. **Runtime and semantic cycle policy stay distinct.** `SemanticGraph` already models type/superclass/protocol/constraint/ADT relationships independently from runtime edges ([`graph.rs`](../../../../phalcom-modules/src/graph.rs)). Type-header SCCs may be legal where runtime initialization cycles are not.
4. **No stale or partial publication.** A snapshot is an immutable coherent product for one source/interface revision set. A cancelled or cyclic query never overwrites a prior coherent answer with partial data.
5. **Incremental by recorded dependencies, not filename guesses.** Every semantic read records a stable dependency key. Changes invalidate reverse dependents of the semantic fact which changed.
6. **Safe concurrency first.** Start with a project worker that owns mutable query state and produces `Arc` immutable snapshots. Do not introduce shared mutable solver internals, `unsafe` answer slots, or global compilation locks as the default design.
7. **Diagnostics are products, not logs.** Stable diagnostics have origin, code, range, evidence chain, snapshot revision, and replacement semantics per document/version.

## 3. Required semantic products and identities

### 3.1 Stable identity model

Persist identities at the layer which owns them:

| Identity | Owner and required fields | Lifetime |
|---|---|---|
| `WorkspaceId` / `PackageId` | Canonical manifest/root identity. | Workspace discovery revision. |
| `ModuleId` | Package + normalized logical module path; never `ModuleId::core()` except actual core. | Persistent across editing/moves per module policy. |
| `SourceId` / `DocumentId` | Physical source path/URI and language/source kind. | Source catalog. |
| `SourceRevision` | Monotonic document text/version plus content hash. | One snapshot input. |
| `DeclarationId` | Module, declaration kind, stable source identity/ordinal or durable symbol identity. | Interface revision. |
| `CallableId` / `FieldId` | Owning declaration, selector/name, instance/class side, signature slot. | Interface revision. |
| `TypeParameterId` | Owning declaration + zero-based binder index. | Declaration lifetime. |
| `QueryKey` | Semantic product kind plus all relevant IDs/side/type arguments. | Database operation. |
| `InterfaceRevision` | Structural typed-header fingerprint, not full body text. | Incremental invalidation. |
| `SemanticRevision` | Snapshot generation plus dependencies. | Published snapshot. |

`BindingId`, `InferVarId`, control-flow node IDs, and temporary constraint IDs are query/session-local. They must carry an owner/snapshot token or remain private to worker state. They are not valid LSP/CLI cache keys and must not be serialized in interface artifacts.

### 3.2 Typed interfaces

Extend, do not replace, `UnlinkedModuleInterface` and `LinkedModuleInterface` with a semantic header layer:

```rust
struct TypedModuleInterface {
    module: ModuleId,
    source_revision: SourceRevision,
    interface_revision: InterfaceRevision,
    imports: Arc<[ResolvedImport]>,
    exports: Arc<[ExportedDeclaration]>,
    declarations: Arc<[TypedDeclarationHeader]>,
    types: Arc<TypeInterfaceTable>,
    diagnostics: Arc<[SemanticDiagnostic]>,
    completeness: InterfaceCompleteness,
}

enum InterfaceCompleteness {
    Complete,
    HeaderCycle { scc: SemanticSccId },
    Blocked { reason: InterfaceBlocker },
    Invalid { error_count: u32 },
}
```

`TypedDeclarationHeader` contains the declaration’s stable ID/name/visibility/range; class or protocol header; type parameter descriptors; resolved superclass/protocol references; declared field, getter, setter, callable, and constructor surfaces; and source/evidence ranges. It may contain an invalid/blocked header without inventing a nominal parent. A client can then report an unresolved imported annotation precisely while still typechecking unrelated modules.

Interfaces must export only facts whose meaning does not depend on an unchecked body. Inferred public signatures need explicit policy: either public declarations require annotations initially, or their inferred signature is a query result whose interface revision includes body dependencies. Do not publish a guessed local inference result as an interface contract merely because it is convenient for a cycle.

### 3.3 Immutable snapshot

The skeletal semantic snapshot currently stores sources/type store/surfaces/dispatch and a generation ([`snapshot.rs`](../../../../phalcom-semantic/src/snapshot.rs)). Replace it incrementally with a product whose access APIs make coherence visible:

```rust
struct SemanticTypeSnapshot {
    workspace: WorkspaceRevision,
    type_store: Arc<TypeStore>,
    modules: Arc<ModuleSemanticTable>,
    typed_interfaces: Arc<TypedInterfaceTable>,
    body_summaries: Arc<BodySummaryTable>,
    dispatch: Arc<DispatchIndex>,
    diagnostics: Arc<DiagnosticIndex>,
    dependencies: Arc<DependencyIndex>,
    metadata: SnapshotMetadata,
}

struct SnapshotMetadata {
    semantic_revision: SemanticRevision,
    source_revisions: Arc<SourceRevisionMap>,
    completed_queries: QueryCompletionSet,
    timings: AnalysisTimings,
}
```

It is valid for a snapshot to hold `Blocked`, `Invalid`, or `Unknown` facts. It is invalid for it to combine an interface from one revision with a body summary which assumed a different interface revision, or to expose a mutable `TypeStore` which can add terms after publication. `SemanticSnapshot` methods should return typed result envelopes, not raw map lookups which erase completion state.

## 4. Pipeline and query model

### 4.1 Staged pipeline

The production pipeline shall be staged as follows:

```text
1. Source catalog + revisions
2. Parse/recover each source
3. Build unlinked interfaces                 (existing modules seam)
4. Resolve imports / create linked interfaces (existing modules seam)
5. Collect declaration header shells
6. Resolve type headers and publish typed interface SCC products
7. Build bindings and control-flow summaries
8. Demand body type, dispatch, conformance, and diagnostic queries
9. Solve SCC-local constraints / summarize callables
10. Publish immutable SemanticTypeSnapshot
```

Each stage declares inputs, outputs, revision key, diagnostic owner, and legal blockers. Parsing can finish with recoverable errors. Header collection can establish stable declaration names before all headers resolve. Header resolution processes semantic SCCs with placeholders only for declared recursive identities, not for arbitrary missing imports. Body checking consumes the final typed header product for its SCC and imported dependencies. Publish only after every demanded query has a non-pending terminal status.

### 4.2 Query lifecycle

Use a small explicit state machine at first:

```rust
enum QueryState<T> {
    Vacant,
    Computing { owner: WorkerEpoch },
    Ready { value: Arc<T>, revision: SemanticRevision },
    Blocked { reason: QueryBlocker, revision: SemanticRevision },
    Cancelled { revision: SemanticRevision },
    Poisoned { diagnostic: InternalDiagnostic, revision: SemanticRevision },
}
```

The worker maintains a stack of active query keys. Re-entering a key detects a cycle and resolves it according to the query’s declared policy: header SCC batch, recursive relation blocker, callable summary SCC, or illegal cycle diagnostic. It never waits on itself. Cancellation is checked at phase boundaries and bounded loops. Cancellation means “discard this attempt”; it does not turn into `Unknown`, `Dynamic`, or a success result.

Initial implementation may use a single project-worker `RefCell`/owned map because it is easy to make state transitions correct and to test them deterministically. Immutable `Arc` snapshots cross into compiler/LSP consumers. Fine-grained parallel query execution is deferred until per-query dependency keys, duplicate-work behavior, locks, and cancellation are measured.

### 4.3 Dependencies and invalidation

Expand the existing declaration-fingerprint invalidation shell ([`invalidation.rs`](../../../../phalcom-semantic/src/invalidation.rs)) into dependency records such as:

```text
SourceText(SourceId, SourceRevision)
ParsedSyntax(SourceId, SyntaxHash)
ImportResolution(ModuleId, ImportName)
TypedHeader(DeclarationId, InterfaceRevision)
MemberSurface(CallableId | FieldId, InterfaceRevision)
TypeRelation(CanonicalTypeId, CanonicalTypeId, RelationMode)
NativeSurface(NativeLibraryId, AbiRevision)
Config(TypecheckMode | FeatureFlag)
```

Every query reads dependencies through an instrumented context that records the key and revision. On edit, compute changed keys, invalidate reverse dependencies, then re-evaluate demand roots. A method-body edit should normally retain unchanged typed headers and dependents which only read those headers. A superclass signature, generic variance, or exported member change must invalidate all dependent type/dispatch/conformance queries. The design must explain why every invalidation edge exists; a broad “clear whole workspace” fallback is allowed only for recovery/first migration and must be surfaced in metrics.

## 5. Compiler and CLI contract

### 5.1 Entry selection

Replace the current divergent paths with one `TypecheckRequest`:

```rust
enum TypecheckTarget {
    Inline { virtual_module: ModuleId, source: Arc<str> },
    File { path: CanonicalPath },
    Module { module: ModuleId },
    Package { package: PackageId },
    Project { root: CanonicalPath },
}

struct TypecheckRequest {
    target: TypecheckTarget,
    mode: TypecheckMode,
    diagnostic_format: DiagnosticFormat,
    cancellation: CancellationToken,
}
```

Every target is resolved through the same source/module catalog. Inline source receives an explicit generated virtual module identity and no implicit access to project imports unless a project context is supplied. A file belonging to a project becomes its actual module, while a standalone file becomes a deterministic standalone module identity. Passing a directory either selects a documented `Project`/`Package` target or produces a targeted usage diagnostic; it must not fall through to a raw file-read failure.

`phalcom check` help must stop claiming syntax-only behavior. If syntax-only parsing remains valuable, expose it as `phalcom parse`, a clearly named `--syntax-only` flag, or a separate compiler operation; do not overload type-check command wording.

### 5.2 Modes and diagnostics

`--types=strict` is design-only at baseline. Do not ship the spelling as a promise until its acceptance matrix is decided. A compatible staged design is:

| Mode | Intended policy | Status |
|---|---|---|
| `off` | Parse/link without static type diagnostics. | Proposed compatibility mode. |
| `check` | Report static contradictions proved by authoritative facts; preserve dynamic/unknown boundaries according to evidence policy. | First production target. |
| `strict` | Elevate selected missing/blocked/dynamic-boundary obligations decided by spec. | Experimental until matrix and corpus exist. |

Diagnostics require stable codes, source range, module identity, severity, labels, evidence notes, and a snapshot/source revision. JSON output is a stable schema containing diagnostic source (`parser`, `phalcom-typecheck`, future `phalcom-proof`), `code`, `uri/module`, ranges, related locations, and an optional machine-readable blocker. Output order is stable by URI/range/code. CLI exit status must be documented: successful analysis with warnings differs from type errors; internal/cancelled analysis differs from user program errors.

## 6. LSP integration contract

### 6.1 Ownership and publication

The LSP backend submits changed document/workspace inputs to the same semantic project worker and receives a `SemanticTypeSnapshot` or a coherent per-document projection. It does not run a second checker on the request thread. A projection is usable only when its input document version/source hash matches the version for which it is published.

For each document version, publication does the following atomically from the backend’s viewpoint:

1. Build parser diagnostics from that exact syntax revision.
2. Obtain type diagnostics from the compatible formal snapshot projection.
3. Convert semantic diagnostics through the existing adapter, retaining source `phalcom-typecheck`.
4. Merge/deduplicate by stable diagnostic identity, sort deterministically, and publish a complete replacement set for the URI/version.
5. Drop stale worker results; never append a late type error to a newer syntax result.

The existing `ValueShape` analysis continues to publish completion/hover/inlay data according to its own snapshot/version policy. It may consume formal declarations/types when that improves precision, but it may not manufacture static mismatch diagnostics, change formal subtyping, or become a fallback static checker. This satisfies the typing plan’s “no separate editor-only type checker” requirement while preserving valuable advisory behavior.

### 6.2 Query features after diagnostics

Once diagnostics are coherent, hover and go-to-definition may show a formal type only with evidence/source revision and a clear rendering of `Unknown`, `Dynamic`, and blocked states. Avoid representing “no type result yet” as `Dynamic`. Inlay hints may combine a formal type fact and runtime-shape hint only if their labels identify their provenance; otherwise the UI falsely implies one system proved both.

Manual LSP acceptance must include rebuilding the server, configuring the extension’s server path, invoking **Phalcom: Restart Language Server**, checking the output panel, opening a multi-module fixture, editing an exported signature, and confirming stale diagnostics disappear after a fast follow-up edit. The current syntax diagnostic integration test is necessary but insufficient.

## 7. Implementation sequence

### Phase 1 — semantic identity and typed header data

- Define persistent IDs/revisions and source catalog adapters; eliminate `ModuleId::core()` from non-core typecheck calls.
- Add typed header shell data beside current interfaces. Keep it internal until diagnostics stabilize.
- Implement header name/import/type resolution using Specification 01 lowering APIs, including invalid and blocked header states.
- Define semantic graph SCC policy for headers, superclasses, protocols, and constraints; retain current runtime cycle validation independently.

**Exit criterion:** a two-module project can resolve an imported annotated class/member through a typed header, and a changed method body does not alter the header revision.

### Phase 2 — project database and invalidation

- Build the single-worker query/context system, snapshot builder, typed dependency keys, and reverse invalidation tables.
- Move standalone and inline checking through `TypecheckRequest`; project/package/module requests demand relevant roots.
- Add tracing/metrics: queried/reused/invalidated products, SCC count/size, cancellation count, whole-workspace fallback count, and phase timings.

**Exit criterion:** an edit to one non-exported body causes bounded rechecking, while an exported signature edit invalidates its known dependent fixture; snapshots never mix revisions.

### Phase 3 — CLI and LSP convergence

- Implement documented targets/formats/exit semantics; repair help and directory behavior.
- Publish formal diagnostic projections in LSP, version-gated and merged with syntax diagnostics.
- Preserve `ValueShape` as a separate input/output product. Add an explicit test proving LSP type diagnostics came from a compiler snapshot by comparing codes/ranges/revision metadata.

**Exit criterion:** `phalcom check` and the editor report the same static mismatch for a checked module graph, and both clear it after the same edit.

### Phase 4 — controlled performance evolution

- Establish project corpora and latency/memory budgets before concurrency changes.
- Parallelize only isolated parse/header/body SCC jobs with immutable inputs and explicit cancellation. Do not share one mutable solver across tasks.
- Profile cache hit rate, lock wait, duplicate work, cancellation waste, snapshot size, and tail latency. Keep a serial deterministic mode for debugging/regressions.

**Exit criterion:** a measured workload improves without changed diagnostics or nondeterministic snapshot results; serial and parallel fixtures agree byte-for-byte after normalization of timing fields.

## 8. Test matrix and acceptance evidence

| Area | Required evidence |
|---|---|
| Interface headers | Imports/exports, aliases when introduced, generic headers, missing import, cyclic type headers, invalid superclass/protocol header. |
| Module graph | Legal semantic SCC versus rejected runtime initialization cycle, project/package/module/file/inline target identities. |
| Invalidation | Private body edit, exported signature edit, native ABI revision, config change, deleted module, rename/move policy. |
| Snapshots | No cross-revision mixing, cancelled query does not publish, stale result discarded, stable deterministic diagnostics. |
| CLI | Help text, directory project mode, JSON schema, exit codes, single file/inline/project/package behavior. |
| LSP | Parser + type merge, type mismatch appears/clears, rapid edits, import edit propagation, server restart/manual path, no `ValueShape` static diagnostic. |
| Performance | Cold/warm workspace benchmark, edit latency distribution, memory/snapshot size, cancellation and invalidation counters. |

Tests must include a deliberately poisoned project: one module with an invalid annotation, an independent valid module, and a dependent module. This verifies that typed interfaces record blocked/invalid facts without collapsing all project analysis into unknown results or inventing dynamic success.

## 9. Pyrefly transfer: direct, adapted, rejected

**Take directly:** modular semantic products, compact IDs/tables, staged immutable snapshot publication, precise query dependency recording, explicit cycle/cancellation state, SCC-aware batching, and observable incremental metrics. The transfer’s implementation breakdown gives a compatible phased database path ([implementation breakdown](../pyrefly-transfer/implementation-breakdown.md)); its later phased specification emphasizes no partial answer publication and measured parallelism ([phased specification](../pyrefly-transfer/12-phased-implementation-specification.md)).

**Adapt:** model interfaces on `phalcom-modules` rather than Python import/module rules. Key callable/member queries by Phalcom `CallableId`/`FieldId`, selector, and class/instance side. Preserve semantic versus runtime graph split. Reuse the LSP engine’s cancellation/worklist experience but make compiler snapshots the authority for formal diagnostics.

**Reject:** a Python-style global import fallback, heuristic `Any` for failed imports, an editor-only type DB, shared mutable unsafe answer cells, locking an entire workspace while a type query runs, and unmeasured “blazing fast” claims. Correct coherent snapshots precede aggressive parallelism.

## 10. What this must not preclude

- HKT descriptors/kind checking from Specification 01 without making source type constructors runtime objects.
- Separate environments for classes, protocols, type aliases, ADTs, native modules, class-side surfaces, and reflected type values.
- A demand-driven proof database that reuses typed snapshots but has separate proof-result/cache keys.
- Remote/virtual source providers and REPL modules with explicit virtual identities.
- Multiple user-facing type modes only after each has stable documented semantics and test corpus.

## 11. Risks and design decisions

The primary risk is prematurely serializing or publishing body-inferred signatures as type interfaces. It creates invalidation cycles and turns incomplete inference into an API. Start with declared public headers, then explicitly design any inference-backed export policy.

The second risk is conflating LSP revision with compiler revision. A document’s LSP version, a workspace source hash, an interface revision, and a semantic snapshot generation solve different problems and all need representation. The third is broad cache invalidation hidden behind a “snapshot” name; the metrics must reveal it. The final risk is letting `ValueShape` seep into static diagnostics because it is convenient. Its rich runtime facts are valuable, but substituting them for type evidence violates the target architecture.
