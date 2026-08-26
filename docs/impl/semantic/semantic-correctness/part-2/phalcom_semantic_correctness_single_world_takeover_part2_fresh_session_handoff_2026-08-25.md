# Fresh-Session Handoff — Phalcom Semantic Correctness Part 2

Continue this task from handoff below. Original/current spec is attached and remains authoritative:
`/Users/altunhasanli/dev/phalcom/phalcom/docs/impl/semantic/phalcom_semantic_correctness_single_world_takeover_part2_canonical_identity_projection_advisory_takeover_spec.md`

## Mission and scope

- Deliver: implement Part 2’s canonical source identity/index, machine-readable formal projection, compiler-owned advisory domain, advisory flow/solver, coherent snapshot publication, and eventual LSP authority demotion.
- In scope: Tasks 1–12 and §62 release gates in the attached Part 2 specification, continuing from current partial implementation.
- Out of scope / do not disturb: Part 3 persistent workspace/LSP lifecycle cutover; unrelated dirty work in `phalcom-core`, `phalcom-lsp`, `phalcom-modules`, examples, typing docs, patchwork, and existing user-owned specifications.
- Authority and constraints: the attached Part 2 spec; [Part 2 checklist](/Users/altunhasanli/dev/phalcom/phalcom/docs/impl/semantic/phalcom_semantic_correctness_single_world_takeover_part2_implementation_checklist.md:1); preserve canonical formal ownership and keep advisory facts unable to upgrade formal `Unknown`, `Dynamic`, invalid, or non-ready states. Do not commit, reset, clean, switch branch, or broaden ownership without explicit request.

## Current state

- Status: implementation active; ready for the next narrow Task 9 slice.
- Complete and focused-verified:
  - Tasks 1–2: snapshot-scoped source-site identities, canonical targets, compiler-owned lexical scopes, source-order resolution, imports/classes, parameters, destructuring, loop bindings, mutability, and redeclaration tracking.
  - Task 3 primitives: bounded interval lookup, nested occurrence selection, unresolved hints, reverse exact-target index, and large-index coverage. AST-wide occurrence collection/publication remains incomplete.
  - Tasks 4–5 partial: exact formal `(CallableId, BindingId)` / `(CallableId, ExpressionId)` attachment, indexed `FormalSemanticProjection`, source-index publication in `SemanticSnapshot`, and formal fact queries. Session incident publication, full call-resolution table publication, richer formal status/cause projection, and workspace reference wiring remain open.
  - Task 6: compiler-owned advisory `ValueShape`, confidence, canonical provenance, bounded deterministic joins, canonical selector-family identity.
  - Task 7 partial: compiler-owned literal/collection/local/field/formal-call expression analysis and one-pass statement flow product over `SourceScopeIndex`.
  - Task 8 partial: canonical parameter slots, contribution source replacement/removal, changed-slot deltas, explicit advisory product outcomes, and deterministic callable-summary/fact fingerprints.
- Deliberately unfinished: Tasks 7–8 integration with real formal callable products/dispatch/query dependencies; Task 9 advisory workspace publication/reuse; Tasks 10–12 LSP adapter/takeover/incrementality audits. Part 2 release gates remain open.
- Working tree: branch `main`. Part 2-owned edits include `phalcom-semantic/src/identity.rs`, `src/lib.rs`, `src/presentation.rs`, `src/session.rs`, `src/snapshot.rs`, new `src/source_index/`, new `src/advisory/`, new advisory/source tests, and the Part 2 checklist. Existing unrelated modified/untracked files are present and must remain untouched.
- Relevant workflow context: use targeted repository reads; preserve concurrent work; use `apply_patch` for edits; run focused verification at each slice; run `graphify update .` after code changes.

## Evidence and decisions

- Canonical formal/source snapshot products exist: `SemanticSnapshot` publishes `formal_projection` and `source_index`; `SemanticWorkspaceSession` builds source scope/index products and attaches formal callable products. Evidence: `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/snapshot.rs` symbols `SemanticSnapshot::formal_projection`, `source_index`, `formal_expression`, `formal_binding`, `formal_fact_at`; `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/session.rs` helper `build_source_semantic_index`.
- Formal projection is keyed, not rendered-string authority: `FormalFactRef`, `FormalFactSite`, `FormalSemanticProjection::get`, and `fact_at`. Evidence: `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/presentation.rs`.
- Exact source attachment rejects missing/ambiguous identity rather than choosing an arbitrary range. Evidence: `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/source_index/mod.rs`, `CallableSourceAttachment::from_analysis`, `SourceSemanticIndex::attach_formal_analysis`.
- Advisory and formal channels remain structurally separate. Advisory uses `ValueShape`, `AdvisoryFact`, `AdvisoryConfidence`, and `AdvisoryOrigin`; no advisory API constructs `TypeKnowledge` or emits formal diagnostics. Evidence: `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/advisory/`.
- Advisory literal identity must come from supplied canonical builtin declarations. Missing builtin identity returns `ValueShape::Unknown`; no class identity is fabricated. Evidence: `AdvisoryBuiltins`, `analyze_expr`, and test `analyzer_does_not_fabricate_missing_builtin_identity`.
- Parameter contributions are source-indexed and recompute only touched slots. Evidence: `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/advisory/parameters.rs`, `AdvisoryParameterContributions::replace_source`, `remove_source`.
- The current advisory flow is deliberately a compiler-owned foundation, not yet a complete interprocedural solver or snapshot product. Do not mark Tasks 7–9 complete based on these unit products alone.
- Full capability baseline is not green: prior handoff verification observed `12/40` capability tests passing and `28/40` failing, primarily stale `var`/bare-brace fixtures plus documented capability gaps. This baseline was not rerun after the latest advisory-only slice; do not call it a current full pass.

## Code and artifact map

- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/identity.rs` — `SourceOwner`, `SourceSiteLocalId`, `SourceSiteId`, `SourceSiteRef`, `SemanticTargetId`; cross-revision canonical identities versus snapshot-local source sites.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/source_index/` — compiler-owned `SourceScopeIndex`, builder, interval index, occurrences, formal attachments, and `SourceSemanticIndex`; AST occurrence collection remains the main Task 3 gap.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/presentation.rs` — `FormalFactRef`, `FormalFactSite`, `FormalSemanticProjection`; preserve keyed checker products as formal truth.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/snapshot.rs` — immutable source/formal publication and keyed formal queries; next work should add advisory publication without mixing generations.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/session.rs` — current source index build path; currently uses default `SourceIndexContext` and ignores attachment errors, so linked import context and incident publication need deliberate repair.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/advisory/shape.rs` — canonical bounded runtime-shape domain and method-family shape.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/advisory/fact.rs` — `AdvisoryFact`, confidence, bounded provenance, deterministic fact fingerprint.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/advisory/analyzer.rs` — `AdvisoryBuiltins`, `AdvisoryExpressionContext`, `analyze_expr`; consumes source scopes, formal resolved-call callback, and injected canonical dispatch adapter.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/advisory/flow.rs` — `AdvisoryFlowContext`, `AdvisoryFlowProduct`, `analyze_statements`; currently one-pass and not yet a published callable product.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/advisory/parameters.rs` — contribution replacement/removal and delta model.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/advisory/summary.rs` — explicit advisory status, minimal effects placeholder, deterministic `AdvisoryCallableSummary`.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/tests/source_semantic_index.rs` — 11 source identity/scope/occurrence/formal attachment/projection tests.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/tests/advisory_domain.rs` — 6 domain/contribution/summary tests.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/tests/advisory_analyzer.rs` — 5 analyzer/flow tests.
- `/Users/altunhasanli/dev/phalcom/phalcom/docs/impl/semantic/phalcom_semantic_correctness_single_world_takeover_part2_implementation_checklist.md` — authoritative live task status; update checkboxes only after focused evidence.

## Validation

- Passed: `cargo test -p phalcom-semantic --test advisory_analyzer --test advisory_domain --test source_semantic_index` — 5 + 6 + 11 tests passed.
- Passed: `RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test workspace --test product_stability_invalidation --test callable_dependency_invalidation --test presentation` — 8 + 7 + 9 + 4 tests passed.
- Passed: `cargo check -p phalcom-semantic`.
- Passed: `cargo fmt --all -- --check`.
- Passed: `git diff --check -- phalcom-semantic docs/impl/semantic/phalcom_semantic_correctness_single_world_takeover_part2_implementation_checklist.md`.
- Passed: `graphify update .` — graph refreshed; expected warning remains for zero-node app config files and HTML visualization is skipped because graph exceeds visualization limit.
- Failed/baseline: full semantic capability target previously reported `12/40` passed and `28/40` failed; classification: pre-existing handoff baseline plus known fixture/capability gaps; not rerun after latest advisory-only changes.
- Not run: Tasks 9–12 snapshot/LSP/workspace final gates — run only after advisory workspace publication and LSP bridge work land.

## Resume plan

1. Read attached Part 2 spec, this handoff, and the checklist. Start only with `phalcom-semantic/src/advisory/flow.rs`, `parameters.rs`, `summary.rs`, `snapshot.rs`, `session.rs`, and spec §§20–22.
2. Implement `AdvisoryWorkspace`/module shard products from spec §21, reusing `SourceSiteId`, `FieldId`, `AdvisoryParameterSlot`, `AdvisoryCallableSummary`, and exact formal attachments. Success: one immutable advisory product can answer expression, binding, field, parameter, callable, and target queries without LSP types.
3. Add snapshot publication under the existing `SemanticSnapshot` identity. Build advisory products from the same source/formal input revision; advisory failure must be non-fatal to valid formal snapshots. Add focused snapshot coherence and missing-vs-`Unknown` tests.
4. Replace default/ignored source-index context and attachment errors in `build_source_semantic_index` with explicit linked module context and internal incident handling. Preserve deterministic product fingerprints and `Arc` reuse boundaries.
5. Add only then: compiler DB query/dependency integration and SCC/worklist convergence tests (Task 8), followed by Task 10 LSP canonical-module adapters and Task 11 authority demotion. Finish with Task 12 reuse/invalidation/forbidden-pattern/final gates.

## Do not re-explore

- Do not reread old conversation or scan the repository broadly; this file and the attached spec contain the current state.
- Do not revisit the decision that formal `TypeKnowledge` and advisory `ValueShape` are separate semantic channels; the existing APIs/tests establish it.
- Do not port the LSP identity/scope/dispatch graph wholesale. Extend compiler-owned source/formal products and use LSP only as an adapter until Part 3.
- Do not mark the Part 2 release gate complete from focused tests. The checklist explicitly records Tasks 3–5 and 7–8 as partial and Tasks 9–12 as open.
- Before changing snapshot/session publication, preserve same-revision coherence among formal projection, source index, advisory products, and `SnapshotId`; test failure/non-fatal behavior before wiring LSP.
