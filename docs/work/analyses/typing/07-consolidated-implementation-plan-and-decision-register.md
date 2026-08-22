# Phalcom Typing Platform: Consolidated Implementation Plan and Decision Register

**Date:** 2026-08-22
**Status:** Ratified dependency-ordered plan; no task or suggested commit is claimed complete
**Authority:** execution plan for [specifications 01–06](README.md#6-specification-dependency-graph) after the completed two-axis semantic-tower milestone
**Depends on:** [01 — Implementation Architecture](01-implementation-architecture.md), [02 — Runtime Reification and Metadata](02-runtime-reification-and-metadata.md), [03 — Reflection API and Capabilities](03-reflection-api-and-capabilities.md), [04 — User-Facing Type Syntax and Lowering](04-user-facing-type-syntax-and-lowering.md), [05 — Advanced Kinds, Constraints, Effects, and Proofs](05-advanced-kinds-constraints-effects-and-proofs.md), and [06 — Language Comparisons and Design Rationale](06-language-comparisons-and-design-rationale.md)
**Owners:** `phalcom-modules`, `phalcom-semantic`, `phalcom-type-syntax`, compiler/runtime in `phalcom-core`, `phalcom-lsp`, future proof component, language design reviewers
**Non-goals:** implementation in this documentation change, runtime class/metaclass redesign, selector changes, test execution, staging, or commits

## 1. Program objective and completion contract

Implement one compiler-owned, incremental, bounded typing platform spanning source syntax, semantic analysis, metadata, reflection, effects, contracts, and proofs while preserving Phalcom runtime semantics.

Program is complete only when:

- source syntax lowers through one formal semantic pipeline;
- proper types, constructors, kinds, rows, effects, and proof states remain separate;
- project/interface/import/link failures publish structured states and diagnostics;
- recursive relations and analyses terminate under explicit cycle/budget policy;
- compiler, CLI, REPL, and LSP consume one immutable formal snapshot;
- advisory LSP `ValueShape` remains advisory;
- durable metadata contains no raw semantic/store IDs or solver variables;
- nominal reification preserves existing class identity;
- synthetic descriptors are immutable and reclaimable;
- proof artifacts preserve result, trust, dependencies, and cache validity;
- cold and incremental analysis are structurally equivalent;
- baseline two-axis and runtime object-model gates remain green.

No focused test alone establishes program acceptance. Every phase report must separate passed scope, unrelated baseline failures, deferred gates, and unverified work.

## 2. Current baseline and protected invariants

### 2.1 Observed baseline

**Observed current implementation.** Live source provides:

- `TypeStore`, kind-checked applications, owner/index type parameters, declaration surfaces, semantic checking, and `SemanticSnapshot` in `phalcom-semantic`;
- parse-once module units, interfaces, linker, reference/semantic/runtime graphs, project universe, and stable project/module identities in `phalcom-modules`;
- analyzed-program compilation and runtime contract weaving in `phalcom-core`;
- an advisory incremental semantic engine plus a wrapper over formal static snapshots in `phalcom-lsp`;
- native symbolic type/effect/raise/return-flow metadata in `phalcom-type-syntax` and `phalcom-native-meta`.

**Observed test coverage.** Supplied Task 13 evidence reports the completed two-axis milestone passed 1,103 workspace tests with six skipped plus doctests. This plan treats that report as baseline evidence; it was not rerun for this documentation task.

### 2.2 Invariants at every phase

Every task and review must enforce:

1. runtime semantics unchanged unless a separately ratified runtime-boundary feature says otherwise;
2. selector identity stays type-independent;
3. class/metaclass hierarchy and nominal identity stay unchanged;
4. compiler and LSP share formal facts;
5. advisory `ValueShape` stays advisory;
6. no raw semantic/store ID enters an artifact;
7. no inference/kind/row/effect/proof variable escapes its solver;
8. no missing/invalid/unresolved/unknown/`Dynamic`/budget/cancellation/internal state becomes success;
9. no analysis loop can diverge;
10. no proof result overstates trust;
11. current two-axis tests remain baseline gates;
12. incremental and cold analysis produce structurally equivalent public results.

Violation blocks the task even when focused tests pass.

## 3. Dependency graph and phase order

```text
completed two-axis tower
        |
        v
Phase A: invariant and failure-state repairs
        |
        +--------------------+
        v                    v
Phase B: source grammar   Phase C: compiler SemanticDb
        |                    |
        +---------+----------+
                  v
Phase D: generics, kinds, bounds, Self, aliases, rows
                  |
                  v
Phase E: constants, flow, effects, exits, totality
                  |
                  v
Phase F: metadata DAG and native convergence
                  |
                  v
Phase G: runtime reification and reflection
                  |
                  v
Phase H: compiler/CLI/REPL/LSP convergence and old-path deletion
                  |
                  v
Phase I: canonical contracts, VCs, proving, artifacts
                  |
                  v
Phase J: stabilization, fuzzing, performance, rollout
```

Phase B parser core and Phase C DB substrate may proceed in parallel after Phase A if file ownership is isolated. Phase D needs both. Phase I contract IR can begin after Phase E and semantic snapshots stabilize, but persistent artifact carriage depends on Phase F.

## 4. Task-card contract

Every task below contains objective, normative source, likely files, dependencies, products, implementation order, tests first, verification command, migration, deletion criterion, risks, must-not-preclude check, reviewer checklist, and suggested cohesive commit. Exact line numbers may drift; named symbols and modules are authoritative resume points.

Commands are planned verification commands. This documentation task does not run them.

## 5. Phase A — Harden completed tower invariants and failures

### A1 — Enforce proper-type construction in release builds

**Objective.** Make it impossible for a constructor-kinded `TypeId`, foreign-store ID, or solver variable to enter value knowledge through unchecked public APIs.

**Normative sections.** [01 §3.1–3.2](01-implementation-architecture.md#31-semantic-domains), [01 §4.2](01-implementation-architecture.md#42-type-store-lifecycle-and-invariant-enforcement), [05 §3.3](05-advanced-kinds-constraints-effects-and-proofs.md#33-publishability-invariant).

**Files/modules.** `phalcom-semantic/src/types/id.rs`, `store.rs`, `evidence.rs`, `checker/typed_expr.rs` (`TypeKnowledge::known`, `TypedExpression::known`), snapshot/export boundaries, focused type/store/checker tests.

**Inputs/dependencies.** Completed two-axis kind store; no later phase.

**Products/APIs.** `ProperTypeId`; `TypeStore::expect_proper_type`; checked `TypeKnowledge::known`; `PublishabilityError` seed.

**Ordered implementation.** (1) write misuse tests in debug/release semantics; (2) add checked newtype/store ownership validation; (3) migrate constructors and checker call sites; (4) validate snapshot/export boundaries; (5) restrict raw constructor visibility; (6) audit unsafe/raw `TypeId` entry points.

**Tests first.** Foreign store, arrow-kind form, partial application, infer variable, valid `Never`/`Unit`/nominal, and store-generation mismatch.

**Verify.** `cargo test -p phalcom-semantic proper_type && cargo test -p phalcom-semantic --release proper_type`

**Migration.** Temporary deprecated checked adapter may accept `TypeId` and return `Result<ProperTypeId, _>`; no silent conversion.

**Deletion criterion.** Formal value-knowledge constructors cannot accept raw `TypeId`; debug assertions are not sole enforcement.

**Risks.** Broad call-site churn; accidental treatment of explicit `Dynamic` as ordinary ID.

**Must not preclude.** Constructor-kinded positions, prenex kinds, explicit dynamic knowledge, metadata export.

**Reviewer checklist.** Release enforcement; store ownership; no solver escape; no runtime change; negative tests prove old hole.

**Suggested commit.** `refactor(semantic): enforce proper type boundaries`

### A2 — Replace boolean/coarse relations with explicit outcomes and budgets

**Objective.** Give equivalence, subtype, assignability, and consistency separate, terminating result APIs.

**Normative sections.** [01 §3.2, §3.4, §4.5, §5.3](01-implementation-architecture.md#32-knowledge-and-relation-laws), [05 §7](05-advanced-kinds-constraints-effects-and-proofs.md#7-constraint-ir-and-solver), [06 §12.11](06-language-comparisons-and-design-rationale.md#1211-boolean-relations-versus-explicit-terminal-outcomes).

**Files/modules.** `phalcom-semantic/src/types/relation.rs`, equality/application/substitution callers, checker diagnostics, exports in `phalcom-semantic/src/lib.rs`, relation tests.

**Inputs/dependencies.** A1 proper operands.

**Products/APIs.** `RelationKind`, `RelationOutcome`, `RelationContext`, `RelationBudget`, query-local pair state and reason paths.

**Ordered implementation.** (1) characterize current laws; (2) add outcome API beside booleans; (3) add pair-state cycle guard and budget; (4) migrate checker/application callers by relation kind; (5) prohibit `Unknown` as success; (6) remove public boolean exports.

**Tests first.** Callable variance, record width/depth, unions, class-object versus nominal, dynamic consistency versus subtype, recursive pair, budget, cancellation hook.

**Verify.** `cargo test -p phalcom-semantic relation`

**Migration.** Boolean compatibility wrapper exists only internally and must map exclusively `ProvenYes` to `true`; diagnostic callers use full outcome immediately.

**Deletion criterion.** `is_subtype`/coarse assignability no longer public or used by formal compiler paths.

**Risks.** Callers may preserve previous permissive behavior accidentally; cache keys may omit policy.

**Must not preclude.** Rows, F-bounds, intersections, proof relations, open-world dispatch.

**Reviewer checklist.** Relation distinctions; deterministic cause path; recursion termination; budget keyed; no dynamic/budget success.

**Suggested commit.** `refactor(semantic): publish bounded relation outcomes`

### A3 — Add source-owned diagnostics and partial module states

**Objective.** Preserve project, load, interface, import, link, and runtime-cycle failures as structured module/source-owned facts.

**Normative sections.** [01 §4.6, §5.2, §6, Unit A2/A4](01-implementation-architecture.md#46-diagnostics-contract), [06 §9](06-language-comparisons-and-design-rationale.md#9-python-and-pyrefly-architecture-without-python-semantics).

**Files/modules.** `phalcom-semantic/src/diagnostic.rs` (`DiagnosticLabel`, `SemanticDiagnostic`), `workspace.rs`, snapshot/module-state additions; `phalcom-modules/src/error.rs`, `interface.rs`, `linker.rs`, `graph.rs`; `phalcom-lsp/src/analysis_service.rs` (`run_static_workspace_analysis`).

**Inputs/dependencies.** Existing `ModuleId`, `SourceId`, source ranges, module error enums.

**Products/APIs.** `DiagnosticOwner`, `ModuleSemanticState::{Available, Partial, Failed}`, structured `WorkspaceAnalysisOutcome`; runtime-cycle diagnostic retaining cycle path.

**Ordered implementation.** (1) tests for every skipped/fallback path; (2) add source/module owner to labels; (3) convert module errors into partial states; (4) remove sorted-order runtime-cycle fallback; (5) publish unaffected module facts with failed states; (6) adapt LSP rendering.

**Tests first.** Missing project, load failure, interface failure, unresolved import, link failure, semantic cycle accepted, runtime cycle rejected, multi-file labels.

**Verify.** `cargo test -p phalcom-modules && cargo test -p phalcom-semantic workspace_failure && cargo test -p phalcom-lsp --test integration static_workspace_failure`

**Migration.** Existing all-or-nothing `SemanticAnalysis` may wrap new outcome during transition; sorted runtime-cycle fallback is removed in same task, not deprecated.

**Deletion criterion.** No `continue`/skip path drops a formal project/module error; no runtime cycle is compiled/analyzed using sorted order.

**Risks.** Diagnostic duplication across module/semantic layers; partial snapshots used without status check.

**Must not preclude.** Incremental recovery, multiple projects, LSP partial results, exact invalidation.

**Reviewer checklist.** Ownership on every label; cycle path stable; unaffected modules usable; compiler cannot consume failed module as complete.

**Suggested commit.** `fix(semantic): preserve module failures and reject runtime cycles`

### A4 — Establish stable project/module/snapshot/store identity boundaries

**Objective.** Name persistent, generation-local, store-local, and solver-local identity lifetimes before query caching.

**Normative sections.** [01 §4.1, §4.3–4.4](01-implementation-architecture.md#41-stable-store-and-solver-identities), [02 §4.2](02-runtime-reification-and-metadata.md#42-header-identities-and-feature-flags), [05 §3](05-advanced-kinds-constraints-effects-and-proofs.md#3-domain-and-identity-model).

**Files/modules.** `phalcom-modules/src/identity.rs`, `stabilization.rs`, project universe; `phalcom-semantic/src/identity.rs`, `snapshot.rs`, type IDs/store; LSP `semantic/ids.rs`, `semantic/query.rs`, snapshot stamps.

**Inputs/dependencies.** A3 module states; existing `ProjectIdentity`, `ModuleId`, `ResolverGeneration`.

**Products/APIs.** `SemanticGeneration`, `SnapshotStamp`, `TypeStoreId`, stable project/module/source keys, explicit conversions for LSP document identities.

**Ordered implementation.** (1) identity matrix tests; (2) add owner/stamp fields; (3) thread stamps through snapshots/results; (4) reject cross-store/generation reuse; (5) map LSP-local IDs to formal IDs; (6) document serialization exclusions.

**Tests first.** Same logical module across revisions, project relocation policy, cross-store ID rejection, stale document/snapshot request, deterministic stable key.

**Verify.** `cargo test -p phalcom-modules identity && cargo test -p phalcom-semantic identity && cargo test -p phalcom-lsp --test integration stale_snapshot`

**Migration.** Retain local adapter maps; never transmute/equate same-shaped ID newtypes.

**Deletion criterion.** Formal cache/result key cannot be created without project/module and revision/store ownership as applicable.

**Risks.** Over-stable identities surviving semantic replacement; under-stable identities destroying reuse.

**Must not preclude.** Package artifact reproducible identity, multi-workspace service, metadata fingerprints.

**Reviewer checklist.** Lifetime table; stale rejection; no path-as-logical-ID shortcut; raw IDs excluded from artifacts.

**Suggested commit.** `refactor(semantic): define stamped identity boundaries`

## 6. Phase B — Full source type grammar and lowering

### B1 — Parse core annotation forms with recovery

**Objective.** Make existing AST application/union/tuple/callable forms source-spellable and add explicit unit/dynamic/never/self/invalid nodes.

**Normative sections.** [04 §3–§4, §8](04-user-facing-type-syntax-and-lowering.md#3-lexical-and-contextual-boundary).

**Files/modules.** `phalcom-ast/src/ast.rs` (`TypeAnnotationExpr`), `parser.rs` (`parse_type_annotation` and helpers), lexer token fission if needed, `phalcom-ast/tests/integration.rs` and parser fixtures.

**Inputs/dependencies.** A1/A2 semantic target shapes; current contextual annotation entry points.

**Products/APIs.** Precedence parser, source ranges, recovery nodes, nested `>` handling, core source grammar.

**Ordered implementation.** (1) snapshots for grammar/precedence/errors; (2) parse atoms/postfix; (3) tuples/callable domains; (4) right-associative arrows/unions; (5) recovery/synchronization; (6) nested token fission; (7) update source fixtures.

**Tests first.** Full [04 §13 S1](04-user-facing-type-syntax-and-lowering.md#unit-s1--parser-parity-for-existing-ast-forms) matrix plus malformed composites.

**Verify.** `cargo test -p phalcom-ast --test integration type_annotation`

**Migration.** Existing qualified references parse identically. No generic/row syntax enabled in this task.

**Deletion criterion.** `parse_type_annotation` no longer always returns `Reference`; hand-built-only core AST cases have equivalent source tests.

**Risks.** Callable/group/tuple ambiguity; `>>` interaction; recovery consuming outer expressions.

**Must not preclude.** Record rows, intersections, aliases, formatter round trips, native lowering convergence.

**Reviewer checklist.** Precedence table matches AST; all ranges exact; recovery has progress; no runtime expression evaluation.

**Suggested commit.** `feat(parser): parse complete core type annotations`

### B2 — Introduce explicit annotation and native knowledge results

**Objective.** Stop representing invalid/missing/opaque native types as ordinary unknown type forms.

**Normative sections.** [04 §2.2–2.3, §7.2, §9](04-user-facing-type-syntax-and-lowering.md#22-semantic-lowering-already-covers-core-forms), [05 §3.3](05-advanced-kinds-constraints-effects-and-proofs.md#33-publishability-invariant).

**Files/modules.** `phalcom-semantic/src/types/annotation.rs`, `evidence.rs`, diagnostics and tests; `phalcom-type-syntax/src/lib.rs`; `phalcom-native-meta` adapters/spec types.

**Inputs/dependencies.** A1 proper types; B1 AST atoms/recovery.

**Products/APIs.** `ResolvedAnnotation`, `AnnotationStatus`, `MetadataTypeKnowledge::{Known, Opaque, Missing, Invalid}`; explicit dependencies/provenance.

**Ordered implementation.** (1) negative status tests; (2) replace string-matched atoms; (3) distinguish parse/resolution/kind/application failure; (4) adapt native parser output without deleting compatibility input; (5) validate publishability; (6) update diagnostics.

**Tests first.** Missing, invalid, unresolved, legacy native `Unknown`, explicit `Dynamic`, budget/cancel, unsupported native form.

**Verify.** `cargo test -p phalcom-semantic --test type_annotations && cargo test -p phalcom-type-syntax && cargo test -p phalcom-native-meta`

**Migration.** Native `Unknown` input may decode under old schema but immediately becomes `Opaque(reason)`; it is never interned.

**Deletion criterion.** Invalid application/kind paths do not return `UnknownReason::UnannotatedDeclaration`; native `TypeExpr::Unknown` absent from canonical output.

**Risks.** Breaking native catalog parsing; callers ignoring new status.

**Must not preclude.** Schema-version adapters, explicit `Any`, proof unknown states.

**Reviewer checklist.** Exhaustive matching; no fallback success; provenance retained; source/native front ends remain separate.

**Suggested commit.** `refactor(typing): separate annotation and native knowledge states`

## 7. Phase C — Compiler-owned semantic DB and partial outcomes

### C1 — Introduce compiler-owned staged SemanticDb

**Objective.** Replace fresh whole-workspace formal analysis with stamped, query-owned staged products while preserving current answers.

**Normative sections.** [01 §4.3–§5.2, Unit A3](01-implementation-architecture.md#43-semantic-database), [06 §9 and §12.12](06-language-comparisons-and-design-rationale.md#9-python-and-pyrefly-architecture-without-python-semantics).

**Files/modules.** New `phalcom-semantic/src/db/` or `query/`; `workspace.rs` (`analyze_workspace` adapter); `snapshot.rs`; parsed/interface/link integration with `phalcom-modules`; semantic tests/bench instrumentation.

**Inputs/dependencies.** A3 structured states, A4 identities, A2 outcomes.

**Products/APIs.** `SemanticDb`, typed `QueryKey`, `QueryCell` safe state, `SemanticDb::apply`, `SemanticDb::snapshot`, staged parse/interface/shell/body products, cancellation token.

**Ordered implementation.** (1) golden old/new differential harness; (2) introduce DB with cold-only staged queries; (3) stamp dependencies/results; (4) add SCC-local batch publication; (5) publish immutable snapshot; (6) make `analyze_workspace` a compatibility one-shot DB call; (7) add metrics.

**Tests first.** Cold equality, partial modules, same-thread recursion, cancellation before publish, stale candidate rejection, deterministic SCC order.

**Verify.** `cargo test -p phalcom-semantic workspace && cargo test -p phalcom-semantic snapshot`

**Migration.** Existing one-shot API remains until all consumers adopt DB; it cannot own a divergent analysis path.

**Deletion criterion.** Formal analysis implementation lives behind DB queries; one-shot function contains no separate checker/linker orchestration.

**Risks.** Cache-validity bugs; diagnostics published from losing/stale computation; memory growth.

**Must not preclude.** Safe parallelism, proof queries, partial workspaces, compaction/eviction.

**Reviewer checklist.** Query state machine; dependency/stamp in key; atomic snapshot publication; no partial candidate leak; differential equality.

**Suggested commit.** `feat(semantic): add staged compiler-owned semantic database`

### C2 — Move reverse invalidation and workspace ownership into `phalcom-semantic`

**Objective.** Reuse formal semantic dependency keys and eliminate LSP as owner of formal reverse invalidation.

**Normative sections.** [01 §5.1, Unit A3/A6](01-implementation-architecture.md#51-revision-update-and-invalidation), [Pyrefly invalidation transfer](../pyrefly-transfer/06-dependency-graph-and-incremental-invalidation.md#phalcom-dependency-products).

**Files/modules.** Semantic DB dependency/change modules; workspace/source integration; `phalcom-lsp/src/semantic/invalidation.rs`, `engine.rs`, `semantic/mod.rs` adapters.

**Inputs/dependencies.** C1 query keys; A4 identities; current LSP change classification.

**Products/APIs.** `DependencyKey`, `ChangeKind`, forward/reverse edge stores, `ChangeSet`, affected closure, reuse metrics.

**Ordered implementation.** (1) differential edit corpus; (2) define existence/type/metadata/class/callable keys; (3) record dependencies during queries; (4) build reverse graph; (5) compare old/new surfaces; (6) invalidate exact closure with wildcard fallback; (7) adapt LSP changes to `SemanticDb::apply`.

**Tests first.** Body edit, export type/existence/metadata changes, alias/contract/native changes, reverse edge add/remove, cyclic modules, clean/incremental equality.

**Verify.** `cargo test -p phalcom-semantic invalidation && cargo test -p phalcom-lsp --test integration incremental_static`

**Migration.** LSP advisory `ValueShape` invalidation remains for advisory facts. Formal reverse graph moves only after differential coverage.

**Deletion criterion.** No LSP module owns reverse dependencies for formal static types/diagnostics/linking.

**Risks.** Under-invalidation; fallback invalidating everything; duplicated dependency graphs.

**Must not preclude.** Expression-level keys if measurement requires, proof artifact dependencies, multiple clients.

**Reviewer checklist.** Reverse-edge symmetry; every lookup records key; wildcard explicit; metrics; clean equality.

**Suggested commit.** `feat(semantic): own formal dependency invalidation`

### C3 — Complete partial outcomes, cancellation, budgets, and publication

**Objective.** Apply one terminal-state protocol to module, type, relation, and query results.

**Normative sections.** [01 §3.4, §4.5–4.6, §5.5](01-implementation-architecture.md#34-termination-cycles-and-budgets), [03 §4](03-reflection-api-and-capabilities.md#4-result-objects-honest-terminal-states).

**Files/modules.** Semantic DB cells/snapshots/diagnostics; relation and workspace results; LSP analysis status/static snapshot adapters.

**Inputs/dependencies.** A2/A3, C1/C2.

**Products/APIs.** common terminal envelope or domain-specific exhaustive statuses; budget classes; cancellation propagation; committed-generation diagnostics/traces.

**Ordered implementation.** (1) state-transition tests; (2) add terminal reasons to each query; (3) thread cancellation/budgets; (4) hold diagnostics/traces until winning publication; (5) expose module/query status in snapshots; (6) adapt UI status.

**Tests first.** Every terminal state, retry policy, stale/cancelled nonpublication, partial module queries, trace-off semantic equality.

**Verify.** `cargo test -p phalcom-semantic query_state && cargo test -p phalcom-lsp --test analysis_status`

**Migration.** Coarse uncertain values decode into explicit legacy reason; no reverse mapping for formal consumers.

**Deletion criterion.** No formal query silently omits failure or publishes stale/cancelled diagnostics.

**Risks.** Generic result abstraction erases domain detail; callers retry internal errors indefinitely.

**Must not preclude.** Proof-specific results/trust, streaming observability, safe parallel cells.

**Reviewer checklist.** Exhaustive state chart; cancellation points; metrics; retry rules; losing work has no side effects.

**Suggested commit.** `feat(semantic): publish reasoned query terminal states`

## 8. Phase D — Generics, variance, bounds, `Self`, aliases, rows, and HKTs

### D1 — Add generic declaration syntax and prenex kind machinery

**Objective.** Parse/lower class, method, and alias binders with stable type/kind identities and checked partial application.

**Normative sections.** [04 §5](04-user-facing-type-syntax-and-lowering.md#5-generic-declarations), [05 §4](05-advanced-kinds-constraints-effects-and-proofs.md#4-kind-system).

**Files/modules.** AST/parser class/method/type-alias declarations; semantic `types/kind.rs`, `parameter.rs`, new `kind_solver.rs`; declarations/interfaces/metadata-independent export tests.

**Inputs/dependencies.** B1/B2, C1, A1.

**Products/APIs.** generic syntax nodes; `KindScheme`; `KindParameterId`/`KindVarId`; kind unifier/generalization; explicit proper/constructor position checks.

**Ordered implementation.** (1) syntax/kind failure tests; (2) AST binders; (3) semantic stable IDs; (4) solver-local unification; (5) prenex generalization/instantiation; (6) declaration/interface publication; (7) reject escaping variables.

**Tests first.** Owner/index, same-name binders, arrow kinds, partial applications, occurs check, unsolved escape, nested source parsing.

**Verify.** `cargo test -p phalcom-ast --test integration generic_type && cargo test -p phalcom-semantic kind`

**Migration.** Native bootstrap generic signatures coexist until Phase F; source generic support no longer hand-built.

**Deletion criterion.** Source declarations can own generic signatures; no stable interface contains solver `KindVarId`.

**Risks.** Parser delimiter conflicts; accidental higher-rank generalization; constructor in value position.

**Must not preclude.** Protocol/ADT binders, higher-kinded libraries, metadata schemes.

**Reviewer checklist.** Prenex only; stable versus local IDs; release publishability; no `Type :: Type`.

**Suggested commit.** `feat(typing): add generic binders and prenex kinds`

### D2 — Implement variance, bounds, `Self`, and transparent aliases

**Objective.** Complete generic signature semantics and reusable alias/source identity.

**Normative sections.** [04 §5–§7](04-user-facing-type-syntax-and-lowering.md#5-generic-declarations), [05 §5 and §8](05-advanced-kinds-constraints-effects-and-proofs.md#5-type-parameters-variance-and-bounds).

**Files/modules.** AST/parser `+`/`-`, `where`, `type`; semantic `parameter.rs`, declarations, new `variance.rs`, `constraint.rs`, `alias.rs`; `Self` lowering/substitution; interfaces/export fingerprints.

**Inputs/dependencies.** D1, A2 relation outcomes, C2 invalidation.

**Products/APIs.** `Variance`, `TypeBound`, occurrence validator, exact-set/F-bound constraints, `SelfTypeTerm`, transparent alias identity/expansion.

**Ordered implementation.** (1) semantic tests before parser enablement; (2) variance composition; (3) bound normalization/solving; (4) owner/side `Self`; (5) alias cycle guard/fingerprint; (6) parse/lower syntax; (7) incremental dependency tests.

**Tests first.** Positive/negative/invariant paths, callable/nested constructors, finite set, F-bound budget, inherited `Self`, class side, alias equivalence/substitution/cycle/change.

**Verify.** `cargo test -p phalcom-semantic variance && cargo test -p phalcom-semantic alias && cargo test -p phalcom-ast --test integration generic_constraint`

**Migration.** Applied generic relations remain invariant until a declaration's variance validates. Recursive aliases remain rejected.

**Deletion criterion.** No parameter identity by name; no hard-coded invariant relation for validated applications; no alias expansion without cycle/dependency context.

**Risks.** Variance through mutable members; F-bound nontermination; alias diagnostic explosion.

**Must not preclude.** Protocol coherence, opaque/newtypes, guarded recursive ADTs, intersections.

**Reviewer checklist.** Occurrence paths; kind before bound; `Self` side distinction; alias provenance; budgets.

**Suggested commit.** `feat(typing): validate generic signatures and aliases`

### D3 — Add record-specific open rows

**Objective.** Extend closed structural records to explicit `RecordRow` tails and ratified `#{ fields, | R }` syntax.

**Normative sections.** [04 §4.4 and §9.4](04-user-facing-type-syntax-and-lowering.md#44-record-row-syntax), [05 §6](05-advanced-kinds-constraints-effects-and-proofs.md#6-record-rows).

**Files/modules.** AST/parser record type nodes; semantic `kind.rs`, `store.rs`, `relation.rs`, new `row.rs`/`row_solver.rs`; annotation lowering; interfaces and tests.

**Inputs/dependencies.** D1 row kind/binders, D2 substitution, A2 relations.

**Products/APIs.** `RecordRowId`, `RecordTail`, `RecordRowVarId`, canonical row/type construction, row solver, capability-aware record relations.

**Ordered implementation.** (1) canonical/solver tests; (2) newtyped domain IDs; (3) row equations/occurs checks; (4) relation integration; (5) syntax and lowering; (6) fingerprints; (7) incremental tests.

**Tests first.** Sorting, duplicates, closed/open, tail substitution, final-field union plus comma tail, width/depth/mutation capability, budget/cancel.

**Verify.** `cargo test -p phalcom-semantic record_row && cargo test -p phalcom-ast --test integration record_type`

**Migration.** Existing closed `TypeData::Record` converts to `RecordTail::Closed` with identical relations under current read-only policy.

**Deletion criterion.** No sentinel field/fake type parameter encodes openness; row variables cannot appear in canonical store/artifacts.

**Risks.** Mutation unsoundness; row/type domain mixing; ambiguous syntax regression.

**Must not preclude.** Separate variant/effect rows and typed shared row utility.

**Reviewer checklist.** Domain wrappers; comma grammar; deterministic fields; capability in relation context; no class-layout reflection.

**Suggested commit.** `feat(typing): add record-specific open rows`

## 9. Phase E — Constants, flow, effects, exits, and totality

### E1 — Retain exact constant and formal flow facts

**Objective.** Preserve exact literal/value knowledge without singleton types and move formal flow dependencies into compiler-owned summaries.

**Normative sections.** [01 §3.3](01-implementation-architecture.md#33-numeric-and-constant-facts), [05 §7 and §9](05-advanced-kinds-constraints-effects-and-proofs.md#7-constraint-ir-and-solver), [06 §12.6](06-language-comparisons-and-design-rationale.md#126-singleton-literal-types-versus-constant-facts).

**Files/modules.** semantic evidence/checker expression/flow/typed expression modules; callable summaries; LSP adapters where formal facts replace duplicate ones.

**Inputs/dependencies.** C1 DB, C2 dependencies, A1 proper types.

**Products/APIs.** `ConstantFact`, binding/program-point flow versions, joins/phi facts, formal callable value-flow summary.

**Ordered implementation.** (1) literal/branch tests; (2) add constant fact separate from type; (3) retain through typed expressions; (4) build binding-keyed flow versions; (5) summary dependencies; (6) expose immutable facts; (7) adapt editor displays.

**Tests first.** `1`/`1.0`, branch joins, predicate refinement, return receiver/argument, invalidation after body edit.

**Verify.** `cargo test -p phalcom-semantic constant_fact && cargo test -p phalcom-semantic flow`

**Migration.** Existing nominal literal types remain unchanged; LSP advisory facts may enrich but not override formal flow.

**Deletion criterion.** No formal consumer invents singleton `TypeId` for ordinary literals; duplicated formal return-flow logic removed from LSP after parity.

**Risks.** State explosion; conflating compile-time constant with runtime immutability.

**Must not preclude.** Explicit refinement types, proof terms over constants, effect/termination summaries.

**Reviewer checklist.** Nominal type unchanged; provenance; join bounded; formal/advisory boundary.

**Suggested commit.** `feat(semantic): retain constants and formal flow facts`

### E2 — Infer effects, exits, and explicit totality

**Objective.** Publish separate effect, raise/exit, return-flow, and termination products.

**Normative sections.** [05 §9–§10](05-advanced-kinds-constraints-effects-and-proofs.md#9-effect-and-exit-model), [06 §2 and §11](06-language-comparisons-and-design-rationale.md#2-haskell-and-ml-kinds-constructors-inference-and-correctness).

**Files/modules.** semantic callable/flow/query modules; new `effects.rs`, `termination.rs`; native metadata adapters; snapshots/diagnostics; compiler call summaries.

**Inputs/dependencies.** E1 flow/call graph, C3 terminal states, native effect/raise/return-flow specs.

**Products/APIs.** `EffectAtom`, `EffectKnowledge`, `ExitSummary`, `TerminationRequirement`, `TerminationKnowledge`, SCC summary queries.

**Ordered implementation.** (1) native and source summary tests; (2) canonical effect sets; (3) body/call joins; (4) exit separation; (5) SCC fixed point; (6) explicit totality checker; (7) metadata-ready summary; (8) diagnostics/metrics.

**Tests first.** Pure/unknown/known, mutation/I/O/reflection/FFI/DNU, raise versus diverge, recursive SCCs, total requirement, `Never` distinctions, budgets/cancel.

**Verify.** `cargo test -p phalcom-semantic effect && cargo test -p phalcom-semantic termination && cargo test -p phalcom-native-meta`

**Migration.** Native specs adapt into formal summaries; current syntactic purity predicate remains runtime-contract eligibility floor until canonical contract IR.

**Deletion criterion.** No formal purity/totality result is derived only from `is_pure_expr`, `Never`, or missing metadata.

**Risks.** Open-world calls falsely pure; fixed-point widening; totality false positives.

**Must not preclude.** Effect rows/handlers, termination measures, proof VCs.

**Reviewer checklist.** Four-axis separation; native authority validation; open-world fallback; SCC monotonicity; no default totality.

**Suggested commit.** `feat(semantic): publish effects exits and totality`

## 10. Phase F — Versioned metadata and native convergence

### F1 — Create and validate indexed semantic metadata DAG

**Objective.** Export deterministic, versioned, depth-bounded, VM-independent semantic metadata.

**Normative sections.** [02 §4 and §6.1–6.2](02-runtime-reification-and-metadata.md#4-vm-independent-metadata-schema), [05 §13.3–13.4](05-advanced-kinds-constraints-effects-and-proofs.md#133-persistent-artifact).

**Files/modules.** New common metadata crate per [02 §4.1](02-runtime-reification-and-metadata.md#41-crate-placement-and-dependency-rule); semantic exporter; compiled artifact structs in `phalcom-core/src/modules/artifact.rs`; schema/property/fuzz tests.

**Inputs/dependencies.** A4 identities, D/E canonical summaries, C3 statuses.

**Products/APIs.** schema header/features; indexed kind/type/parameter/alias/declaration/effect/contract/proof nodes; validator; deterministic encoder/decoder; metadata profiles.

**Ordered implementation.** (1) ADR/schema fixtures; (2) ID-free data model; (3) validator limits; (4) semantic export with fingerprints; (5) deterministic encode/decode; (6) artifact carriage; (7) hostile input tests.

**Tests first.** Round trips, byte determinism, node order permutation, raw-ID rejection, depth/width/size limits, unknown feature/version, solver escape.

**Verify.** `cargo test -p phalcom-semantic-metadata && cargo test -p phalcom-semantic metadata_export && cargo test -p phalcom-core --test modules_compile metadata`

**Migration.** `CompiledTypeRef` remains an in-memory adapter until all compiled/native consumers use DAG; it is not serialized as permanent wire format.

**Deletion criterion.** Compiled/native reflection path no longer requires recursive `CompiledTypeRef`; no raw semantic ID crosses artifact boundary.

**Risks.** Premature permanent wire format; decoder allocation attacks; nondeterministic graph numbering.

**Must not preclude.** Permanent format decision, package reproducibility, proof certificates, metadata profiles.

**Reviewer checklist.** Crate dependencies acyclic; hostile limits; deterministic fixtures; feature negotiation; ID hygiene.

**Suggested commit.** `feat(metadata): add versioned semantic DAG`

### F2 — Converge native and source semantics below separate grammars

**Objective.** Make native metadata authoritative, explicit about opacity, and semantically equivalent to source forms where overlapping.

**Normative sections.** [02 §7.1](02-runtime-reification-and-metadata.md#71-authoritative-native-schema), [04 §2.3 and Unit S6](04-user-facing-type-syntax-and-lowering.md#23-native-metadata-has-a-separate-parser), [06 §12.10](06-language-comparisons-and-design-rationale.md#1210-duplicate-complete-grammars-versus-shared-normalized-lowering).

**Files/modules.** `phalcom-type-syntax/src/lib.rs`; `phalcom-native-meta` specs/catalog; semantic native surface loader; metadata DAG adapters; core native descriptors.

**Inputs/dependencies.** B2 knowledge results, D/E semantic forms, F1 schema.

**Products/APIs.** shared normalized semantic term/metadata builder; explicit native opaque/missing/invalid states; variance/kind/bounds/effect/raise/return metadata.

**Ordered implementation.** (1) source/native equivalence corpus; (2) adapter term vocabulary; (3) schema-version legacy `Unknown` conversion; (4) add generic/effect fields; (5) validate native catalog at build/load; (6) route semantic surfaces through metadata; (7) remove duplicate interpretations.

**Tests first.** Equivalent forms, `Never`, `Self`, applications/callables, kind/variance/bounds, unknown/opaque, catalog mismatch, schema compatibility.

**Verify.** `cargo test -p phalcom-type-syntax && cargo test -p phalcom-native-meta && cargo test -p phalcom-semantic native_surface`

**Migration.** Separate parsers remain. Old native schemas decode through explicit version adapter and re-emit current schema only.

**Deletion criterion.** No native `Unknown` type fallback; no second semantic normalization/arity/kind implementation in native loader.

**Risks.** Native bootstrap cycles; old catalog breakage; source recovery abstractions leaking into metadata.

**Must not preclude.** Generated native catalogs, third-party native modules, richer source syntax.

**Reviewer checklist.** Authority direction; legacy adapter bounded; no runtime evaluation; equivalent structural fingerprints.

**Suggested commit.** `refactor(native): converge semantic type metadata`

## 11. Phase G — Runtime reification and reflection API

### G1 — Add loader-owned registry and immutable descriptors

**Objective.** Reify metadata without changing nominal class identity or retaining unbounded synthetic forms.

**Normative sections.** [02 §3, §5–§6](02-runtime-reification-and-metadata.md#3-semantic-and-runtime-contract), [06 §6 and §10](06-language-comparisons-and-design-rationale.md#6-swift-metatypes-and-reflection-stratification).

**Files/modules.** `phalcom-core/src/modules/` new type registry; value/object descriptor representation; loader/materializer; GC tracing/weak handles; current `reflection_cache.rs` only where shared lifecycle patterns apply.

**Inputs/dependencies.** F1 validated metadata, F2 native surfaces.

**Products/APIs.** `RuntimeTypeRegistry`, `TypingContextArena`, `TypeDescriptor`/kind descriptor objects, weak canonical map, nominal class lookup.

**Ordered implementation.** (1) object-model/GC tests; (2) immutable representation; (3) nominal reification returns class; (4) synthetic canonicalization; (5) bounded context/world validity; (6) weak cleanup/unload; (7) metrics/security limits.

**Tests first.** `Int` identity, union/tuple/callable equivalence, VM-local identity, GC reclamation, unload, stale world, depth/width attack.

**Verify.** `cargo test -p phalcom-core --test invariants && cargo test -p phalcom-core type_reification`

**Migration.** Existing module/project reflection cache remains separate; no strong tracing of every synthetic descriptor.

**Deletion criterion.** No nominal wrapper object; no strong immortal synthetic cache; registry rejects unvalidated metadata.

**Risks.** GC cycles; stale descriptors; accidental class tower changes; identity/equivalence confusion.

**Must not preclude.** Multiple metadata profiles, VM teardown, explicit generic construction, proof descriptors.

**Reviewer checklist.** Object invariants; weak ownership; nominal `===`; structural equivalence; capability/world checks.

**Suggested commit.** `feat(runtime): reify semantic type metadata`

### G2 — Expose capability-checked TypingContext and result objects

**Objective.** Provide honest runtime/static reflection APIs without ambient generic dispatch.

**Normative sections.** [03 §3–§9 and §11](03-reflection-api-and-capabilities.md#3-object-model-of-reflected-semantics), [06 §7 and §12.3](06-language-comparisons-and-design-rationale.md#7-smalltalk-objects-messages-dnu-and-live-reflection).

**Files/modules.** core universe/class definitions; native primitives for typing module; runtime registry facade; semantic query facade; module exports; LSP adapters for source TypeUse.

**Inputs/dependencies.** G1 registry, C1 snapshots, F1 source occurrence metadata.

**Products/APIs.** `TypingResult`, `TypeRelationResult`, `ProofResult` shells; `TypingContext`; TypeForm role methods; TypeUse; checked form construction/relations/member lookup; explicit reflective construction.

**Ordered implementation.** (1) API contract tests; (2) result classes; (3) context acquire/pin/budget/capabilities; (4) observe nominal/kind/synthetic forms; (5) source TypeUse; (6) relations/members; (7) explicit construction; (8) open-world/security handling.

**Tests first.** Known/opaque/missing/invalid/cancel/budget; class-object versus nominal; private metadata; `perform`/DNU/FFI; no ambient context; world invalidation.

**Verify.** `cargo test -p phalcom-core typing_reflection && cargo test -p phalcom-core --test invariants`

**Migration.** New APIs are additive. No `Type.currentApplication` compatibility shim and no applied descriptor forwarding.

**Deletion criterion.** None for ambient APIs because they must never be introduced; temporary direct registry test hooks removed after primitives exist.

**Risks.** Capability bypass; descriptors treated as semantic authority; construction altering dispatch.

**Must not preclude.** Proof results, richer TypeUse, safe tooling queries, context-specific metadata retention.

**Reviewer checklist.** Explicit context everywhere; resource limits; no selector/class changes; terminal states exhaustive; runtime/static boundary clear.

**Suggested commit.** `feat(reflection): expose explicit typing context`

## 12. Phase H — Compiler, CLI, REPL, and LSP convergence

### H1 — Route all formal consumers through published SemanticDb snapshots

**Objective.** Make one formal snapshot authoritative across compiler, check command, REPL, and LSP.

**Normative sections.** [01 Unit A5](01-implementation-architecture.md#unit-a5--compiler-cli-repl-and-lsp-consumers), [03 §11.3–11.4](03-reflection-api-and-capabilities.md#113-semantic-query-facade), [06 §12.12](06-language-comparisons-and-design-rationale.md#1212-editor-owned-checker-versus-compiler-owned-semantic-db).

**Files/modules.** `phalcom-core/src/modules/compile.rs`, compiler entry points, CLI/check/REPL entry points; `phalcom-lsp/src/analysis_service.rs`, semantic static snapshot adapter; workspace/project service ownership.

**Inputs/dependencies.** C1–C3, E summaries, F metadata as needed.

**Products/APIs.** long-lived workspace semantic service; `AnalyzedProgram` from stamped snapshot; consumer request APIs; structured partial/stale status.

**Ordered implementation.** (1) cross-consumer golden tests; (2) compiler DB service; (3) compiler/check adoption; (4) REPL cell update/query; (5) LSP static adapter; (6) stale document checks; (7) manual server validation instructions/metrics.

**Tests first.** Same diagnostics/export structure across compiler/check/LSP, REPL update invalidation, failed module status, stale LSP request, cancellation.

**Verify.** `cargo test -p phalcom-core --test modules_compile && cargo test -p phalcom-repl && cargo test -p phalcom-lsp --test integration static_semantic`

**Migration.** Consumers switch one at a time behind differential assertions. `ValueShape` continues for advisory hover/completion refinements.

**Deletion criterion.** No consumer rebuilds/links/checks a separate formal program; all formal diagnostics name snapshot stamp.

**Risks.** service lifetime leaks; REPL revision semantics; LSP latency regression.

**Must not preclude.** multiple projects, background analysis status, headless compiler, snapshot compaction.

**Reviewer checklist.** Same formal facts; advisory separation; server path/restart/manual validation documented; stale rejection.

**Suggested commit.** `refactor(frontends): consume compiler semantic snapshots`

### H2 — Delete LSP-owned formal linking/checking and duplicate IDs

**Objective.** Complete authority transfer only after parity, preserving advisory UX analysis.

**Normative sections.** [01 §8 migration](01-implementation-architecture.md#8-migration-compatibility-and-must-not-preclude), [06 §9](06-language-comparisons-and-design-rationale.md#9-python-and-pyrefly-architecture-without-python-semantics).

**Files/modules.** `phalcom-lsp/src/analysis_service.rs`, `semantic/engine.rs`, `semantic/invalidation.rs`, `semantic/ids.rs`, snapshot/query wrappers; tests and docs/config if server behavior changes.

**Inputs/dependencies.** H1 parity and performance evidence; C2 formal invalidation.

**Products/APIs.** thin mappings from documents/requests to formal IDs/snapshots; explicit advisory subsystem ownership.

**Ordered implementation.** (1) inventory each LSP fact/consumer; (2) classify formal versus advisory; (3) assert formal parity; (4) delete formal linker/checker/reverse graph; (5) simplify IDs; (6) keep advisory flow/UX; (7) measure and manually validate extension host.

**Tests first.** Navigation/hover/tokens/diagnostics/inlay behavior, body/interface edit reuse, unloaded project errors, no duplicate diagnostics.

**Verify.** `cargo test -p phalcom-lsp --test integration && cargo test -p phalcom-lsp --test analysis_status`

**Migration.** Delete only paths proven replaced. Feature flag may compare old/new during H1 but is removed here.

**Deletion criterion.** No LSP formal `SemanticDb`, formal module linker, or formal reverse dependency graph remains; `ValueShape` clearly labeled advisory.

**Risks.** deleting UX-only inference misclassified as formal duplicate; extension-host status regressions.

**Must not preclude.** richer advisory analysis, remote workspace service, progressive status.

**Reviewer checklist.** Ownership inventory; no formal duplication; manual extension validation; latency/memory comparison.

**Suggested commit.** `refactor(lsp): remove duplicate formal analysis`

## 13. Phase I — Contracts, verification conditions, proving, artifacts

### I1 — Introduce canonical semantic contract IR

**Objective.** Give runtime weaving and static proof lowering one contract identity and typed semantic source.

**Normative sections.** [05 §11](05-advanced-kinds-constraints-effects-and-proofs.md#11-contract-model), [02 §4.6](02-runtime-reification-and-metadata.md#46-effects-termination-contracts-and-proofs).

**Files/modules.** AST attributes/contracts; new semantic contract IR/lowering; `phalcom-core/src/compiler/attributes.rs`; compiler metadata and method object adapters; focused core/semantic tests.

**Inputs/dependencies.** E1 typed flow, E2 effects/totality, C1 snapshots.

**Products/APIs.** `CallableContract`, `ContractPredicate`, `ContractId`, admissibility result, runtime-guard lowering adapter.

**Ordered implementation.** (1) current weaving golden tests; (2) typed contract lowering; (3) purity/termination admissibility; (4) runtime adapter; (5) `old` parity; (6) compile-mode/proof-status separation; (7) metadata export.

**Tests first.** Requires/ensures/invariant, `old`, impure predicate, release/unchecked modes, inherited contracts, source IDs.

**Verify.** `cargo test -p phalcom-semantic contract && cargo test -p phalcom-core contract`

**Migration.** Preserve emitted runtime guard behavior bytecode/semantically while semantic IR becomes source. Duplicate direct AST interpretation remains only until differential parity.

**Deletion criterion.** Runtime compiler does not independently reinterpret contract meaning from raw AST; stripped guard never changes proof result.

**Risks.** Runtime behavior drift; logical subset confused with executable subset; capture mismatch.

**Must not preclude.** heap models, total-correctness VCs, contract reflection, multiple backends.

**Reviewer checklist.** Differential guard behavior; one identity; phase/capture correctness; no false proof.

**Suggested commit.** `refactor(contracts): add canonical semantic contract IR`

### I2 — Add proof IR, VC generation, and result-rich prover facade

**Objective.** Generate deterministic verification conditions and return honest proof results before choosing aggressive proof features.

**Normative sections.** [05 §12–§13.2](05-advanced-kinds-constraints-effects-and-proofs.md#12-proof-ir-and-verification-conditions), [06 §11](06-language-comparisons-and-design-rationale.md#11-proof-oriented-languages-evidence-without-dependent-apis).

**Files/modules.** New proof crate/module only after ADR; semantic snapshot/query facade; proof IR, weakest-precondition, logic normalization, backend trait; CLI opt-in adapter; tests.

**Inputs/dependencies.** I1 contracts, E2 effects/exits/totality, A2 relations, C3 budgets.

**Products/APIs.** `ProofProcedure`, `LogicExpr`, `VerificationCondition`, `ProofPolicy`, `ProofResult`, backend protocol, source mapping.

**Ordered implementation.** (1) ADR and logic subset; (2) proof IR golden tests; (3) CFG lowering; (4) WP/VC generation; (5) deterministic normalization/fingerprint; (6) mock backend result facade; (7) source-mapped counterexamples; (8) opt-in CLI.

**Tests first.** Straight line, branches, loops/invariants, calls, normal/raise/diverge, unsupported operations, deterministic fingerprints, unknown/cancel/budget/internal.

**Verify.** `cargo test -p phalcom-prover`

**Migration.** Runtime contracts remain authority for execution. Proving is opt-in and cannot change code generation initially.

**Deletion criterion.** No ad hoc AST-to-solver path; backend cannot directly label semantic declarations proven without VC/provenance.

**Risks.** unsound lowering/model; path explosion; backend protocol ambiguity.

**Must not preclude.** certificates/kernel, multiple backends, richer theories, partial versus total proofs.

**Reviewer checklist.** Logic/heap assumptions explicit; exits distinct; fingerprint stable; unsupported becomes unknown; no `Prop`/proof terms.

**Suggested commit.** `feat(prover): generate typed verification conditions`

### I3 — Persist trust-aware proof artifacts and integrate reflection

**Objective.** Cache, validate, carry, and reflect proof evidence without overstating authority.

**Normative sections.** [05 §13–§15](05-advanced-kinds-constraints-effects-and-proofs.md#13-proof-results-trust-and-artifacts), [02 §4.6 and Unit B6](02-runtime-reification-and-metadata.md#46-effects-termination-contracts-and-proofs), [03 §4.3 and Unit C6](03-reflection-api-and-capabilities.md#43-proofresult).

**Files/modules.** proof artifact schema/cache/backend/kernel adapter; common metadata DAG; semantic proof queries/dependencies; core reflection result/primitives; CLI/LSP display.

**Inputs/dependencies.** I2 VCs/results, F1 metadata, G2 result objects, C2 invalidation.

**Products/APIs.** `ProofArtifact`, `ProofTrust`, certificate checker interface, cache store, artifact metadata profile, reflected proof result.

**Ordered implementation.** (1) backend/trust threat model; (2) artifact schema/key tests; (3) cache read/write validation; (4) certificate or trusted-backend adapter; (5) dependency invalidation; (6) metadata carriage; (7) reflection/UI trust labels; (8) corruption/hostile tests.

**Tests first.** Every key component change, trust-tier display, corrupt certificate/model, backend version, kernel mismatch, stale interface/native/model, cache location failure, GC/metadata profile.

**Verify.** `cargo test -p phalcom-prover artifact && cargo test -p phalcom-core proof_reflection && cargo test -p phalcom-lsp --test integration proof_status`

**Migration.** Ephemeral proof results remain valid session results but never masquerade as reusable artifacts. Cache is opt-in until location/reproducibility policy ratified.

**Deletion criterion.** No cached verdict lacks full fingerprint/trust/provenance; no UI collapses trust tiers.

**Risks.** stale proof reuse; artifact attack surface; cache poisoning; trusted backend mislabeled kernel checked.

**Must not preclude.** multiple backends, offline certificate checking, package-carried proofs, cache relocation.

**Reviewer checklist.** Full key matrix; hostile decoding; explicit assumption authority; trust visible end to end.

**Suggested commit.** `feat(prover): persist trust-aware proof artifacts`

## 14. Phase J — Stabilization, determinism, fuzzing, performance, rollout

### J1 — Build invariant, differential, fuzz, and performance gates

**Objective.** Prove platform termination, determinism, robustness, and acceptable incremental/cold cost before default rollout.

**Normative sections.** [01 §9 and Unit A6](01-implementation-architecture.md#9-verification-and-acceptance), [02 §11](02-runtime-reification-and-metadata.md#11-verification-and-acceptance), [03 §13](03-reflection-api-and-capabilities.md#13-verification-and-acceptance), [04 §14](04-user-facing-type-syntax-and-lowering.md#14-acceptance-matrix), [05 §18](05-advanced-kinds-constraints-effects-and-proofs.md#18-verification-matrix).

**Files/modules.** `scripts/verify.sh`/`verify_invariants` if current ownership agrees; semantic/core/LSP integration tests; fuzz targets/dictionaries; benchmark harnesses; syntax torture/golden `.ph` corpus; rollout docs.

**Inputs/dependencies.** All phases; current baseline suite.

**Products/APIs.** one verification entry point; invariant corpus; incremental differential harness; parser/metadata/artifact fuzz targets; performance dashboards/budgets; rollout flags and deletion schedule.

**Ordered implementation.** (1) enumerate gates/baseline; (2) add invariants/property laws; (3) cold/incremental differential corpus; (4) fuzz syntax/metadata/relations/rows/artifacts; (5) Miri/sanitizer lanes where applicable; (6) benchmark cold/edit/query/memory; (7) staged rollout; (8) delete flags/transitional adapters after evidence.

**Tests first.** Every migration invariant in §2.2, hostile/cyclic inputs, deterministic fresh stores, cancellation races, GC reclamation, stale proof/cache.

**Verify.** Focused commands above, then `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, doctests, project invariant script, fuzz/sanitizer lanes, and manual VS Code extension-host validation.

**Migration.** Roll out DB/metadata/reflection/prover features separately. Keep fallback only with a named deletion milestone and differential telemetry.

**Deletion criterion.** All transitional feature flags/adapters/duplicate formal paths named by prior tasks removed; performance and correctness gates accepted.

**Risks.** expensive gates hiding focused failures; flaky timing; host-specific toolchain failures; baseline/unrelated failures misreported as task regressions.

**Must not preclude.** future semantic domains, backend swaps, safer cache implementations, additional editor clients.

**Reviewer checklist.** focused then broad evidence; baseline separation; deterministic fixtures; no ignored/unregistered test mistaken for coverage; manual LSP steps recorded.

**Suggested commit.** Multiple cohesive commits by gate: invariant corpus, fuzz targets, benchmarks, rollout/deletion. Never one undifferentiated stabilization commit.

## 15. Phase acceptance gates

| Phase | Acceptance gate |
|---|---|
| A | release proper-type enforcement; explicit relation outcomes; every module failure preserved; runtime cycles rejected; stamped identities tested |
| B | source parser covers core AST with recovery; invalid/missing/native opaque states separated |
| C | cold DB equals current baseline; exact invalidation differential tests; cancelled/stale work never publishes |
| D | generic source signatures publish no solver variables; variance/bounds/`Self`/aliases/rows pass relation and incremental tests |
| E | constants remain facts; effects/exits/return/termination separate; open-world calls never default pure/total |
| F | deterministic bounded metadata; no raw IDs; native/source semantic equivalence; legacy unknown explicit |
| G | nominal identity and object-model invariants; weak synthetic GC; explicit capability/context APIs |
| H | all formal consumers share snapshot; LSP duplicate formal checker/linker/invalidation deleted; manual editor validation |
| I | one contract IR; deterministic VCs; honest result/trust; cache invalidation and hostile artifact tests |
| J | broad/fuzz/performance/manual gates accepted; transition paths removed; rollout report separates evidence classes |

## 16. Decision register

| ID | Decision | Status | Owning specification | Phase | Dependencies | Supersedes | Open follow-up |
|---|---|---|---|---|---|---|---|
| `DEC-TWO-AXIS` | Separate `value.class`, `value : T`, `T :: K`, and explicit reification | Ratified; foundation implemented | 01 §3; 03 §3 | A onward | completed tower | any class/type collapse | none |
| `DEC-TYPEFORM` | `Type` is atomic kind; `TypeForm` is semantic/reflective role, not superclass | Ratified | 03 §3.3; 06 §12.1 | G | two-axis | `Type`-as-protocol proposals | protocol declaration/coherence |
| `DEC-NO-CLASS-WRAPPER` | Nominal forms reify as existing class objects | Ratified | 02 §3.2; 06 §12.2 | G | metadata registry | universal class wrappers | none |
| `DEC-EXPLICIT-REFLECTION` | Explicit immutable `TypingContext`; no ambient current application/forwarding | Ratified | 03 §5; 06 §12.3 | G | metadata, capabilities | `Type.currentApplication` | context acquisition policy details |
| `DEC-VARIANCE-SIGNS` | Declaration-site `+T`, `-T`, `T` | Ratified | 04 §5; 05 §5 | D | generics/kinds | `out`/`in` | protocol variance interactions |
| `DEC-KIND-POLY` | Prenex kind schemes; stable `KindParameterId`, local `KindVarId` | Ratified semantics; public syntax gated | 05 §4 | D | identity, SemanticDb | `Type :: Type`, dependent/universe design | public kind-parameter syntax |
| `DEC-RECORD-ROWS` | Record-specific `RecordRow`, explicit tail, source `#{ fields, | R }` | Ratified | 04 §4.4; 05 §6 | D | kinds, relation outcomes | universal row/fake sentinel designs | capability policy for mutable structural records |
| `DEC-NUMERIC-LITERALS` | Spelling fixes `Int`/`Float`; exact value is `ConstantFact`; no hidden coercion | Ratified | 01 §3.3; 06 §12.6 | E | proper types/flow | fake `Int <: Float`, default singleton types | explicit refinement-type design |
| `DEC-TOTALITY` | Partial default; explicit totality requires termination evidence | Ratified semantics; syntax gated | 05 §10 | E/I | flow/effects | total-by-default, `Never` as divergence | source totality/measure syntax |
| `DEC-PROOF-ARTIFACTS` | Persistent fingerprinted evidence with explicit trust tiers | Ratified architecture; backend gated | 05 §13 | I | metadata, VCs | ephemeral verdict as proof | backend/kernel/trust configuration |
| `DEC-METADATA-DAG` | Versioned indexed bounded metadata DAG; no raw IDs | Ratified architecture | 02 §4 | F | stable identities | recursive persisted `CompiledTypeRef` | permanent wire format/cache location |
| `DEC-REFLECTION-IDENTITY` | Nominal identity existing class; synthetic identity VM-local; equivalence semantic | Ratified | 02 §3; 03 §3 | G | registry | global wrapper identity | none |
| `DEC-COMPILER-SEMANTIC-DB` | `phalcom-semantic` owns formal DB/snapshots/invalidation | Ratified architecture | 01 §4.3; 06 §12.12 | C/H | stamped identities | LSP-owned formal checker | multi-project service lifetime |
| `DEC-RELATION-OUTCOMES` | Separate bounded relation APIs with explicit terminal outcomes | Ratified | 01 §4.5; 05 §7 | A | proper types | boolean/coarse compatibility | intersection/overload relation policy |
| `DEC-NATIVE-SURFACE-AUTHORITY` | Versioned validated native metadata is authoritative surface input | Ratified architecture | 02 §7.1; 05 §9 | F | metadata DAG | ad hoc native fallbacks | third-party native compatibility policy |
| `DEC-SOURCE-NATIVE-GRAMMAR-CONVERGENCE` | Separate front ends lower to shared semantic vocabulary | Ratified | 04 §2.3/§9; 06 §12.10 | B/F | annotation statuses | duplicate semantic grammars or one recovery parser | formatter/source-printer ownership |
| `DEC-ANNOTATION-STATES` | Missing, unresolved, invalid, opaque, dynamic, cancel, budget, internal stay distinct | Ratified | 04 §7.2/§9 | B/C | proper types | `Unknown` type fallback | `Any` admission/semantics |
| `DEC-EFFECT-AXES` | Return type, effects, exits, and termination are separate | Ratified architecture; syntax gated | 05 §9–§10 | E | formal flow | `Never`/effect/termination collapse | effect syntax and handler semantics |
| `DEC-CONTRACT-IR` | One semantic contract identity feeds runtime guards and proof IR | Ratified architecture | 05 §11 | I | effects/flow | duplicate raw-AST interpretation | logical subset/heap model |
| `DEC-WEAK-DESCRIPTORS` | Synthetic descriptors use bounded context-owned weak canonicalization | Ratified | 02 §5; 06 §12.5 | G | registry/GC | strong global descriptor cache | retention tuning |

## 17. Open ratification gates

Open decisions are explicit work blockers at their boundary:

| Gate | Required before | Evidence/owner required |
|---|---|---|
| Public kind-polymorphism syntax | exposing kind binders | language grammar review plus kind-scheme round trips |
| `Any` admission and semantics | accepting `Any` source/metadata | lattice, relation, diagnostic, dynamic-interaction laws |
| Intersection and overload policy | parsing `&` or overload sets | normalization, coherence, dispatch/open-world specification |
| Protocol declaration/coherence | protocol source syntax or conformance lookup | object-model/open-world/coherence decision |
| Recursive alias and ADT guardedness | accepting recursive aliases/ADTs | guardedness, positivity, metadata, relation termination |
| Effect syntax and handler semantics | source effect declarations/handlers | operational semantics, row domain, inference policy |
| Source totality syntax/measures | accepting `total`/measure declarations | termination analysis and assumption policy |
| Exact proof backend/trust configuration | non-mock proving/default rollout | threat model, versioning, certificate/trust review |
| Permanent metadata wire format/cache location | compatibility guarantee or shared cache | schema evolution and migration ADR |
| Package artifact reproducible identity | cross-machine artifact/proof reuse | package/build identity specification |
| Opaque/newtype aliases | syntax/runtime identity | representation, construction, reflection, compatibility decision |
| Mutable structural-record capability | write-compatible row subtyping | mutation capability and variance proof |

Record-row source spelling is no longer open: [04 §4.4](04-user-facing-type-syntax-and-lowering.md#44-record-row-syntax) ratifies `#{ fields, | R }` with mandatory comma before tail.

## 18. Migration and deletion ledger

| Transitional path | Introduced/retained in | Delete when |
|---|---|---|
| raw `TypeId` checked adapter | A1 | all formal value knowledge uses `ProperTypeId` |
| boolean relation wrapper | A2 | compiler/checker callers use outcomes |
| one-shot `analyze_workspace` wrapper | C1 | consumers own/use long-lived DB or explicit one-shot DB call only |
| LSP formal reverse graph/link/check path | C2/H1 | H2 parity and performance gates pass |
| native `Unknown` schema adapter | B2/F2 | oldest supported schema expires under version policy |
| native-bootstrapped generic signature duplicate | D1/F2 | native surfaces lower exclusively through shared metadata |
| recursive `CompiledTypeRef` adapter | F1 | artifacts/loaders consume metadata DAG |
| direct registry test hooks | G1/G2 | capability-checked runtime primitives cover API |
| runtime compiler raw-AST contract interpretation | I1 | semantic contract differential tests pass |
| old/new checker comparison flags | C/H | H2 plus J acceptance |
| ephemeral proof session cache | I2 | persistent cache chosen, or explicitly retained as non-artifact cache |

Deletion is required work, not optional cleanup. Each owning task records code search proving the path absent.

## 19. Verification and evidence reporting

### 19.1 Focused order

For each task:

1. write failing focused tests;
2. implement smallest semantic seam;
3. run focused crate/target filters;
4. run adjacent integration targets named by task;
5. run formatting/diff checks;
6. request independent review;
7. run broader gates only at phase boundary unless risk requires earlier execution.

`autotests = false` applies in crates such as `phalcom-ast`, `phalcom-core`, and `phalcom-lsp`; use registered targets (`--test integration`, `--test invariants`, and named crate targets). A file under `tests/` is not evidence unless Cargo registers/runs it.

### 19.2 Phase report format

Every report states:

- implemented and passing;
- baseline/unrelated failures;
- deferred by named gate;
- not run/unverified;
- migration paths retained and deletion owner;
- performance measurements and environment;
- manual LSP/runtime reflection validation where applicable.

No task says “accepted” from snapshots, compilation, or focused tests alone.

## 20. Risks across program

| Risk | Control |
|---|---|
| semantic/kernel rewrite destabilizes accepted tower | Phase A adapters, differential tests, small commit boundaries |
| parser enables unresolved semantics | Phase B/D gates; semantic validators first for advanced forms |
| under-invalidation creates stale truth | dependency-key tests and clean/incremental differential corpus |
| query recursion/deadlock | same-thread cycle states, SCC-local work, safe cells, budgets |
| parallel optimization creates memory unsafety | safe first implementation; profiling and separate review before atomics/unsafe |
| metadata becomes accidental permanent ABI | schema/version gate and explicit permanent-format decision |
| reflection alters runtime identity | object-model invariants and direct nominal reification |
| LSP loses useful advisory behavior | formal/advisory inventory before deletion and manual extension tests |
| proof model unsound or trust hidden | logic/threat ADR, result algebra, trust tiers, certificate validation |
| open decisions hidden in enums | decision register and named gates; implementation stops at boundary |

## 21. What program must not preclude

Program must leave coherent extension seams for:

- separately ratified `Any`, intersections, overloads, protocols, opaque aliases, recursive ADTs;
- record, variant, and effect rows sharing typed implementation utilities while retaining domains;
- richer flow/refinement facts without turning all constants into singleton types;
- effect handlers and resource/ownership analyses;
- higher-kinded libraries without dependent kinds;
- multiple proof backends, offline certificate checks, and richer logics;
- metadata profile evolution and reproducible package-carried artifacts;
- safe parallel query publication and measured cache eviction;
- multiple IDE clients consuming same snapshots;
- internal specialization preserving observable runtime identity.

It need not preserve an escape route for ambient type context, type-dependent selectors, runtime class cloning, global strong descriptor caches, permissive unknown success, or untrusted proof claims.

## 22. Take directly / Adapt / Reject

### Take directly

- completed two-axis identities and kind-checked application;
- existing module/project identity, graphs, interface shells, and parsed-unit seams;
- compiler analyzed-program gate;
- native effect/raise/return-flow vocabulary as input;
- Pyrefly staged queries, dependency recording, SCC publication, cancellation, snapshots, metrics, and differential testing discipline.

### Adapt

- current relations into reasoned bounded results;
- LSP invalidation into compiler-owned formal dependencies while retaining advisory analysis;
- current contract weaving into one semantic contract IR;
- native symbolic syntax into explicit normalized metadata knowledge;
- current reflection caching patterns into bounded weak synthetic descriptor ownership;
- proof-system architecture into persistent evidence without dependent language features.

### Reject

- production reliance on debug-only type invariants;
- dropped project/interface/import/link failures;
- runtime-cycle sorted fallback;
- fresh whole-workspace analysis as permanent architecture;
- LSP-owned formal truth;
- raw IDs or solver variables in artifacts;
- native `Unknown` as canonical type;
- runtime class wrappers/specializations or ambient generic forwarding;
- boolean/coarse relation success under uncertainty;
- `Never` as termination proof;
- runtime guards or backend verdicts mislabeled as trusted proof.
