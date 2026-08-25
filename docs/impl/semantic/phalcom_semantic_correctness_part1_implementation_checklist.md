# Phalcom Semantic Correctness Part 1 Implementation Checklist

> **For agentic workers:** Execute this checklist task-by-task. The correction spec overrides the WIP specification wherever they differ. Check an item only after its code and focused verification pass.

**Goal:** Implement Part 1's formal semantic epistemic foundation without allowing declarations, assumptions, invalidity, expected context, flow, generic inference, or advisory evidence to impersonate one another.

**Architecture:** Keep formal `TypeKnowledge` inside `phalcom-semantic`; separate status/origin from provenance; make `FlowState` the sole owner of current binding facts; preserve explicit relation/status/causal outcomes. Part 2/3 LSP and advisory takeover remains out of scope.

**Spec authority:**

- Primary WIP: `docs/impl/semantic/phalcom_semantic_correctness_single_world_takeover_part1_formal_epistemic_foundation_spec.md`
- Normative override: `docs/impl/semantic/phalcom_semantic_correctness_part1_corrections_and_amendments.md`
- Baseline observed: `a3f932e01118053265378e678b0dbaef2b9ceab8` (repository has moved beyond WIP baseline; live source wins for file shape)

## Status key

- `[x]` verified complete in current implementation and focused tests
- `[~]` partially implemented or verified only at a narrower seam
- `[ ]` not implemented
- `CURRENT` observed repository behavior; `NORMATIVE` required target; `DEFERRED` explicit Part 2/3 or later completeness scope

## Re-grounding and scope

- [x] `NORMATIVE` Read WIP and correction specs together; correction precedence recorded.
- [x] `CURRENT` Re-verified live `HEAD`, dirty-worktree ownership, semantic crate, tests, and graphify map.
- [x] `DEFERRED` Keep advisory `ValueShape` migration, compiler presentation indices, persistent project lifecycle, LSP semantic-engine deletion, and LSP consumer cutover out of Part 1.
- [x] `NORMATIVE` Preserve unrelated dirty/untracked work and run focused baseline tests before each broad slice.

## Task 1 — Freeze semantic regressions before representation changes

**Files:** focused semantic integration tests; add/extend `phalcom-semantic/tests/` without copying old `EvidenceAuthority` assertions as normative truth.

- [~] Add behavior fixtures for compatible/refuted annotations, genuine no-evidence assumptions, invalid-but-analyzable results, flow joins, expected context, calls, bindings, and fingerprints.
- [~] Add correction tests for real generic conflict payloads, kind mismatch, assumed/established generic support, fixed-return independence, terminal solver outcomes, `SuppressionCause`, and cause-number-insensitive fingerprints.
- [x] Run each new test and confirm failures are semantic, not parser/setup failures.

## Task 2 — Split formal evidence origin from epistemic status

**Files:** `phalcom-semantic/src/types/evidence.rs`, `types/mod.rs`, `lib.rs`, all formal construction sites, explanation compatibility.

- [x] Replace `EvidenceAuthority` with `EvidenceStatus::{Established, Assumed}` and `EvidenceOrigin`.
- [x] Keep `Known`, `Unknown`, and `Dynamic` distinct; no formal advisory status in the new API.
- [x] Add controlled `established`, `assumed`, accessors, and `map_type` preserving status/origin/provenance/range.
- [x] Remove unrestricted mixed-authority construction and migrate every production caller deliberately; classify old `Declared` uses as contract, assumption, or established declaration semantics.
- [x] Keep new formal constructors out of advisory/LSP evidence paths.
- [x] Verify `rg 'EvidenceAuthority' phalcom-semantic/src` has no production hits.
- [x] Add knowledge-to-contract relation API and remove authority-gated refutation from the formal relation path.

## Task 3 — Introduce binding contracts and one current-state owner

**Create:** `phalcom-semantic/src/checker/binding.rs` and causal helper if needed.

**Modify:** `checker/analysis.rs`, `checker/context.rs`, `checker/mod.rs`, `checker/flow/state.rs`.

- [x] Add `BindingContract`, `BindingContractOrigin`, `BindingConsistency`, `AssumptionBasis`, and pure reconciliation; `BindingState`/`FlowState` now retain contract and consistency fields.
- [x] Add `CausalInvalidity::{Clean, One, Multiple}` and explicit non-clean `SuppressionCause`; test algebra and conversion.
- [x] Evolve `BindingState` to hold contract, current knowledge, denotation, consistency, invalidity, mutability, version, and explanation.
- [x] Remove `LocalEnv` as a current-fact owner; scopes map names to `BindingId`; `FlowState` owns current facts. The published binding analysis index is derived from `FlowState`, not a second current-fact owner.
- [x] Replace `bind_local()` with explicit seed/declaration and source-specific binding helpers.
- [x] Make same-scope insertion preserve first identity and return a structured redeclaration result.

## Task 4 — Repair initializer, binding-kind, and assignment transfer

**Modify:** `checker/statement.rs`, `checker/expression.rs`, `checker/context.rs`, `checker/diagnostic.rs`.

- [x] Preserve initializer knowledge under explicit contracts; derive `InferredInitializer` only from usable known initializer evidence.
- [x] Add `UnknownReason::NoTypeEvidence` and assumption-eligibility classifier; coverage gaps, unresolved names, blocked inference, invalid dependencies, and syntax errors remain ineligible.
- [x] Derive mutability from `BindingKind`; diagnose `const` without initializer, bare `let` without initializer honestly, same-scope redeclaration, and immutable writes.
- [x] Reconcile writes against persistent contract, not previous current fact.
- [x] Preserve actual RHS knowledge after mutable type-refuted writes; do not mutate immutable binding state on illegal writes.
- [x] Emit one owning mismatch cause/diagnostic per relation site; keep causal invalidity separate. Binding/assignment, context annotations, calls, and branch dependencies attach and aggregate explicit roots by value.

## Task 5 — Make expression status explicit and causal

**Modify:** `checker/typed_expr.rs`, `checker/analysis.rs`, `checker/expression.rs`, `checker/context.rs`, diagnostics, causal suppression tests.

- [x] Add explicit suppression status and causal fields to typed/published expression results; causes use a monotonic allocator and owner frames.
- [x] Delete diagnostic-range scanning and expression-ID/range-derived causes.
- [x] Allocate monotonic diagnostic causes explicitly and attach roots at owning judgments; expression-owner frames retain ownership without range matching.
- [x] Propagate `CausalInvalidity` by value dependency, without contagious parent `Invalid` status. Calls, receivers, arguments, branch joins, and annotation diagnostic vectors are covered.
- [x] Preserve independently known result knowledge for invalid calls/annotations; use `Suppressed(SuppressionCause)` only when a required premise disappeared.

## Task 6 — Implement epistemically monotone flow joins and widening

**Modify:** `checker/flow/state.rs`, flow transfer/expression synthesis.

- [x] Add one deterministic `join_type_knowledge` operation: reachable `Unknown` wins over known; otherwise `Dynamic` wins; known unions preserve `Established` only when all inputs are established.
- [x] Intersect binding membership; divergent contracts and consistency are fail-closed without retaining first-branch metadata, while owning contract/mutability diagnostics remain at declaration/write sites.
- [x] Join denotation conservatively and causal invalidity independently.
- [x] Recompute consistency through pure reconciliation; never emit a new join diagnostic when hierarchy is available.
- [x] Remove declaration-as-current loop widening; fail closed on nonconvergence. New loop-only declarations are no longer inserted, and inference pass exhaustion returns `Blocked(RecursiveFixpoint)`.
- [x] Add direct flow tests for established/assumed/unknown/dynamic, denotation disagreement, and loop behavior. `spec04_5_flow_graph` covers all variants and deterministic denotation/cause joins.

## Task 7 — Make expected types contextual, not evidentiary

**Modify:** `checker/expected.rs`, statement/body/call/expression consumers.

- [x] Add `ExpectationOrigin` and origin-carrying proper/inference expectations.
- [x] Delete `ExpectedType::from_knowledge` and fake expected `TypeKnowledge` creation from expected-context construction.
- [x] Route checks through knowledge-against-type relation API; preserve actual knowledge while changing only status/diagnostics. Callable return/argument checks retain contract adapters because they are contract judgments, not expected evidence.
- [x] Mark contextual block parameters assumed/contextual, not syntax-established.
- [x] Test expected context proving a relation without overwriting actual literal/unknown knowledge.

## Task 8 — Standardize exact dispatch results and call shape matching

**Modify:** `checker/call.rs`, `checker/expression.rs`, `checker/context.rs`, `dispatch.rs` only as needed.

- [x] Retain full resolved callable identity and dependencies through checker call analysis. Expression products and `CallCheckResult` retain callable identity and dependency edges.
- [x] Add explicit fixed-return promotion and `CallCheckResult`; invalidity/cause and explanation parents remain separate from result knowledge.
- [x] Preserve exact established constructor/call/getter/operator/index result knowledge despite independent argument invalidity.
- [x] Add deterministic linear/indexed argument-to-parameter matching before inference.
- [x] Fail closed for unsupported dynamic labels/expansion packs; unmatched arguments add no constraints.

## Task 9 — Harden generic inference (correction spec overrides WIP Task 9)

**Modify:** `checker/inference.rs`, `checker/call.rs`, expected-result metadata as needed.

- [x] Use real `TypeParameterData.kind`; enforce `TypeStore::kind_of` compatibility.
- [x] Replace boolean `bind`/`unify_terms`/`subtype_terms` failure APIs with structured `Result` failures.
- [x] Preserve actual constraint origin, variable, bounds, and structural failure; eliminate fabricated `InferVarId(0)`, `Never`, `Unit`, or syntax-error placeholders from solver conflicts.
- [x] Convert unresolved `Self` to structured `UnresolvedSelf` failure; receiver specialization integration remains open.
- [x] Feed declared generic constraints with real origins and real call/argument `ExpressionId`s.
- [x] Track bounded monotone `InferenceSupport::{Established, Assumed}` at solver variables/representatives.
- [x] Classify solved generic result from return-influencing variables only; expected context selects valid instantiation but is not value support.
- [x] On conflict/blocked/underconstrained/cancelled/budget, generic call code now returns explicit unknown reasons instead of cloning the unspecialized signature return; fixed-return independence and support-aware promotion are covered for every terminal outcome.
- [~] Add all amendment regression tests, including assumed generic return, fixed-return independence, and conflict payload evidence. Solver-level and call-level regressions pass; the complete amendment matrix remains open.

## Task 10 — Audit Unknown, sentinels, and existing synthesis

**Modify:** checker expression/statement/call/inference paths and focused tests.

- [x] Classify every remaining `Unit`/`Never`/`Object` fallback as genuine language semantics, type-theoretic construction, or illegal sentinel; surviving hits are language/type rules or internal store bootstrap.
- [x] Replace illegal composite component fallbacks with honest unknown/blocked results in list/set/map/tuple/record and generic constructor synthesis.
- [x] Remove arbitrary first-generic-argument iteration inference from generic constructor argument completion; missing evidence now blocks.
- [x] Fix if-let and other branch synthesis to use shared epistemic joins.
- [x] Ensure every `UncheckedExpression` producer is fail-closed and cannot receive a contract-backed assumption; all surviving producers return unknown/ineligible evidence.

## Task 11 — Preserve explanations and semantic product fingerprints

**Modify:** `explain/node.rs`, `explain/arena.rs`, `db/fingerprint.rs`, presentation compatibility, tests.

- [x] Explanations copy actual evidence status/origin and preserve contract relations/real callable identity. Status/origin, resolved callable identity, child explanations, generic argument explanation parents, and explicit binding-contract nodes are published.
- [x] Fingerprints include type, status, origin, contract origin/type, consistency, mutability, causal shape, callable identity, and epistemic flow state.
- [x] Fingerprints ignore range-only provenance and raw `DiagnosticCauseId` allocation; hash causal/status shape instead.
- [x] Preserve Step 5.5 dependency ownership for callable signatures, declaration surfaces, and generic constraints. Callable/declaration dispatch dependencies and generic argument explanation parents are tracked.
- [x] Add status-change, cause-renumbering, range-only, reuse, and invalidation regressions.

## Task 12 — Final epistemic audit and Part 1 release gate

- [x] Run all WIP §30 and correction §13 searches; manually classify every surviving hit. Authority/fake-expected/bind-local/range-cause searches are empty; `.declared`, `UncheckedExpression`, `Unit`/`Never`, and internal `TypeId::DUMMY` hits are classified.
- [~] Run focused semantic tests and `phalcom-lsp` compatibility tests; semantic passes, LSP has one documented baseline failure.
- [~] Run `cargo fmt --check` and focused clippy; formatting passes, focused clippy reports 18 existing `-D warnings` findings in semantic architecture paths.
- [ ] Verify Part 1 completion gate items 1–31 plus correction additions 32–42.
- [x] Record remaining `CURRENT`, `PARTIAL`, `UNVERIFIED`, `DEFERRED`, and unrelated baseline scope here before handoff.
- [ ] Do not begin Part 2 until all release-gate items pass.

## Verification log

- `CURRENT`: formal implementation separates `EvidenceStatus` and `EvidenceOrigin`; binding current facts have one `FlowState` owner; causal annotation aggregation, generic terminal outcomes, and binding-contract explanation nodes are implemented and focused-verified.
- `IMPLEMENTED THIS RUN`: Completed formal evidence migration; explicit binding seeds/source helpers with first-identity redeclaration; BindingKind mutability, missing-initializer and immutable-write transfer; expression-owner diagnostic causes without range scanning; conservative flow denotation/cause joins with hierarchy reconciliation; origin-carrying expected context; real generic constraint/call/argument expression identities; unsupported dynamic pack fail-closed behavior; and composite/generic sentinel removal at audited producers. Prior foundations remain: `NoTypeEvidence` eligibility, knowledge-to-contract relation with real operands, fail-closed epistemic joins/widening, causal invalidity/suppression algebra with monotonic causes, structured generic solver failures/kind checks, per-variable generic support, fixed-return independence, and contract/status/causal fingerprinting.
- `PARTIAL`: behavior-fixture/correction-matrix completeness, focused clippy cleanup, and final release-gate closure; broad AST coverage remains intentionally fail-closed per Part 1 scope.
- `UNVERIFIED`: final Part 1 release gate and full LSP compatibility; final cold/incremental differential and product-stability/dependency checks now pass.
- `BASELINE`: `phalcom-lsp` constructor-factory hover test fails at baseline `a3f932e0` because top-level binding `x` is absent from callable-local formal products; recorded as deferred LSP boundary, not caused by this run.
- `VERIFIED`: final `RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic` passed all unit/integration/doc tests; focused semantic regressions pass; `cargo check -p phalcom-lsp`, `cargo fmt --all -- --check`, and scoped `git diff --check` pass; final registered LSP integration records 51 passed, 2 ignored, and 1 unchanged documented baseline failure.
- `DEFERRED`: Part 2/3 takeover and later completeness listed above.
- `VERIFIED THIS SLICE`: annotation diagnostics preserve all owning root causes; `FlowState` divergent-contract fail-closed regression passes; generic terminal outcome/fixed-return tests pass; binding-contract explanation and invalid-annotation composition tests pass; `cargo check -p phalcom-semantic` and focused semantic suites pass. Focused clippy remains red on 18 architecture warnings; final release gate remains open.
- `CLASSIFIED AUDIT`: zero hits for `EvidenceAuthority`, unrestricted `TypeKnowledge::known`, `ExpectedType::from_knowledge`, `bind_local`, checker `LocalEnv`, expression-range cause allocation, and user-facing `TypeId::DUMMY`; `.declared` is the analysis-index publication compatibility mirror; `UncheckedExpression` producers are fail-closed; surviving `Unit`/`Never` uses are language semantics, type-theoretic rules, or internal store bootstrap; callable return reads are contract reads or exact `promote_exact_return` promotion.
- `SCOPE GATE`: Part 2 compiler identity/projection/advisory work is active; Part 3 lifecycle cutover has not started. Part 1 release gate remains open because focused clippy, behavior/correction matrix, and documented LSP boundary baseline remain; cold/incremental differential, product-stability, and dependency tracking checks pass.
