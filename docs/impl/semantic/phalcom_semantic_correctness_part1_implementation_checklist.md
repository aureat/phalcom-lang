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

- [~] Replace `EvidenceAuthority` with `EvidenceStatus::{Established, Assumed}` and `EvidenceOrigin`.
- [x] Keep `Known`, `Unknown`, and `Dynamic` distinct; no formal advisory status in the new API.
- [x] Add controlled `established`, `assumed`, accessors, and `map_type` preserving status/origin/provenance/range.
- [ ] Remove unrestricted mixed-authority construction and migrate every production caller deliberately; classify old `Declared` uses as contract, assumption, or established declaration semantics.
- [x] Keep new formal constructors out of advisory/LSP evidence paths.
- [ ] Verify `rg 'EvidenceAuthority' phalcom-semantic/src` has no production hits; compatibility migration remains open.
- [x] Add knowledge-to-contract relation API and remove authority-gated refutation from the new formal relation path; legacy authority callers remain to migrate.

## Task 3 — Introduce binding contracts and one current-state owner

**Create:** `phalcom-semantic/src/checker/binding.rs` and causal helper if needed.

**Modify:** `checker/analysis.rs`, `checker/context.rs`, `checker/mod.rs`, `checker/flow/state.rs`.

- [x] Add `BindingContract`, `BindingContractOrigin`, `BindingConsistency`, `AssumptionBasis`, and pure reconciliation; `BindingState`/`FlowState` now retain contract and consistency fields.
- [x] Add `CausalInvalidity::{Clean, One, Multiple}` and explicit non-clean `SuppressionCause`; test algebra and conversion.
- [x] Evolve `BindingState` to hold contract, current knowledge, denotation, consistency, invalidity, mutability, version, and explanation.
- [~] Remove `LocalEnv` as a current-fact owner; scopes map names to `BindingId`; `FlowState` owns current facts. LocalEnv storage removed; analysis-index mirroring remains for publication compatibility.
- [ ] Replace `bind_local()` with explicit seed/declaration and source-specific binding helpers.
- [ ] Make same-scope insertion preserve first identity and return a structured redeclaration result.

## Task 4 — Repair initializer, binding-kind, and assignment transfer

**Modify:** `checker/statement.rs`, `checker/expression.rs`, `checker/context.rs`, `checker/diagnostic.rs`.

- [x] Preserve initializer knowledge under explicit contracts; derive `InferredInitializer` only from usable known initializer evidence.
- [x] Add `UnknownReason::NoTypeEvidence` and assumption-eligibility classifier; coverage gaps, unresolved names, blocked inference, invalid dependencies, and syntax errors remain ineligible.
- [ ] Derive mutability from `BindingKind`; diagnose `const` without initializer, bare `let` without initializer honestly, same-scope redeclaration, and immutable writes.
- [ ] Reconcile writes against persistent contract, not previous current fact.
- [ ] Preserve actual RHS knowledge after mutable type-refuted writes; do not mutate immutable binding state on illegal writes.
- [ ] Emit one owning mismatch cause/diagnostic per relation site; keep causal invalidity separate.

## Task 5 — Make expression status explicit and causal

**Modify:** `checker/typed_expr.rs`, `checker/analysis.rs`, `checker/expression.rs`, `checker/context.rs`, diagnostics, causal suppression tests.

- [~] Add explicit suppression status and causal fields to typed/published expression results; causes now use a monotonic allocator, but legacy diagnostic-range ownership remains.
- [ ] Delete diagnostic-range scanning and expression-ID/range-derived causes.
- [~] Allocate monotonic diagnostic causes explicitly and attach roots at owning judgments; allocator landed, root attachment remains range-derived.
- [ ] Propagate `CausalInvalidity` by value dependency, without contagious parent `Invalid` status.
- [ ] Preserve independently known result knowledge for invalid calls/annotations; use `Suppressed(SuppressionCause)` only when a required premise disappeared.

## Task 6 — Implement epistemically monotone flow joins and widening

**Modify:** `checker/flow/state.rs`, flow transfer/expression synthesis.

- [x] Add one deterministic `join_type_knowledge` operation: reachable `Unknown` wins over known; otherwise `Dynamic` wins; known unions preserve `Established` only when all inputs are established.
- [~] Intersect binding membership; divergent consistency is fail-closed, while contract/mutability diagnostics remain open.
- [ ] Join denotation conservatively and causal invalidity independently.
- [ ] Recompute consistency through pure reconciliation; never emit a new join diagnostic.
- [ ] Remove declaration-as-current loop widening; fail closed on nonconvergence.
- [ ] Add direct flow tests for established/assumed/unknown/dynamic, denotation disagreement, and loop behavior.

## Task 7 — Make expected types contextual, not evidentiary

**Modify:** `checker/expected.rs`, statement/body/call/expression consumers.

- [ ] Add `ExpectationOrigin` and origin-carrying proper/inference expectations.
- [ ] Delete `ExpectedType::from_knowledge` and all fake expected `TypeKnowledge` creation.
- [ ] Route checks through knowledge-against-type relation API; preserve actual knowledge while changing only status/diagnostics.
- [ ] Mark contextual block parameters assumed/contextual, not syntax-established.
- [ ] Test expected context proving a relation without overwriting actual literal/unknown knowledge.

## Task 8 — Standardize exact dispatch results and call shape matching

**Modify:** `checker/call.rs`, `checker/expression.rs`, `checker/context.rs`, `dispatch.rs` only as needed.

- [ ] Retain full resolved callable identity and dependencies through checker call analysis.
- [ ] Add explicit fixed-return promotion and `CallCheckResult`; do not clone contract evidence into current facts.
- [ ] Preserve exact established constructor/call/getter/operator/index result knowledge despite independent argument invalidity.
- [ ] Add deterministic linear/indexed argument-to-parameter matching before inference.
- [ ] Fail closed for unsupported dynamic labels/expansion packs; unmatched arguments never add constraints.

## Task 9 — Harden generic inference (correction spec overrides WIP Task 9)

**Modify:** `checker/inference.rs`, `checker/call.rs`, expected-result metadata as needed.

- [x] Use real `TypeParameterData.kind`; enforce `TypeStore::kind_of` compatibility.
- [x] Replace boolean `bind`/`unify_terms`/`subtype_terms` failure APIs with structured `Result` failures.
- [x] Preserve actual constraint origin, variable, bounds, and structural failure; eliminate fabricated `InferVarId(0)`, `Never`, `Unit`, or syntax-error placeholders from solver conflicts.
- [x] Convert unresolved `Self` to structured `UnresolvedSelf` failure; receiver specialization integration remains open.
- [ ] Feed declared generic constraints with real origins and real call/argument `ExpressionId`s.
- [x] Track bounded monotone `InferenceSupport::{Established, Assumed}` at solver variables/representatives.
- [x] Classify solved generic result from return-influencing variables only; expected context selects valid instantiation but is not value support.
- [~] On conflict/blocked/underconstrained/cancelled/budget, generic call code now returns explicit unknown reasons instead of cloning the unspecialized signature return; fixed-return independence and support-aware promotion now exist, but full call-site coverage remains open.
- [~] Add all amendment regression tests, including assumed generic return, fixed-return independence, and conflict payload evidence.

## Task 10 — Audit Unknown, sentinels, and existing synthesis

**Modify:** checker expression/statement/call/inference paths and focused tests.

- [ ] Classify every remaining `Unit`/`Never`/`Object` fallback as genuine language semantics, type-theoretic construction, or illegal sentinel.
- [ ] Replace illegal composite component fallbacks with honest unknown/blocked results.
- [ ] Remove arbitrary first-generic-argument iteration inference.
- [ ] Fix if-let and other branch synthesis to use shared epistemic joins.
- [ ] Ensure every `UncheckedExpression` producer is fail-closed and cannot receive a contract-backed assumption.

## Task 11 — Preserve explanations and semantic product fingerprints

**Modify:** `explain/node.rs`, `explain/arena.rs`, `db/fingerprint.rs`, presentation compatibility, tests.

- [ ] Explanations copy actual evidence status/origin and preserve contract relations/real callable identity.
- [~] Fingerprints include type, status, origin, contract origin/type, consistency, mutability, and epistemic flow state. Binding contract/consistency/causal fields now participate; explanation and some flow fields remain.
- [x] Fingerprints ignore range-only provenance and raw `DiagnosticCauseId` allocation; hash causal/status shape instead.
- [ ] Preserve Step 5.5 dependency ownership for callable signatures, declaration surfaces, and generic constraints.
- [ ] Add status-change, cause-renumbering, range-only, reuse, and invalidation regressions.

## Task 12 — Final epistemic audit and Part 1 release gate

- [ ] Run all WIP §30 and correction §13 searches; manually classify every surviving hit.
- [~] Run focused semantic tests and `phalcom-lsp` compatibility tests; semantic passes, LSP has one documented baseline failure.
- [~] Run `cargo fmt --check` and focused clippy; formatting passes, focused clippy remains open.
- [ ] Verify Part 1 completion gate items 1–31 plus correction additions 32–42.
- [x] Record remaining `CURRENT`, `PARTIAL`, `UNVERIFIED`, `DEFERRED`, and unrelated baseline scope here before handoff.
- [ ] Do not begin Part 2 until all release-gate items pass.

## Verification log

- `CURRENT`: formal implementation still has compatibility `EvidenceAuthority` callers, bare expectations, range-derived invalidity ownership, and partial generic call provenance at live HEAD.
- `IMPLEMENTED THIS RUN`: Added status/origin constructors and mapping, `NoTypeEvidence` eligibility, knowledge-to-contract relation with real operands, binding contracts and FlowState-backed current lookup, fail-closed epistemic joins/widening, causal invalidity/suppression algebra with monotonic causes, structured generic solver failures/kind checks, per-variable generic support, fixed-return independence, and contract/status/causal fingerprinting. Focused and full semantic suites pass.
- `PARTIAL`: Task 2 compatibility projection and production migration remain; binding redeclaration/mutability diagnostics, expected-origin model, explicit causal ownership, real generic expression IDs, and complete explanation transfer remain open.
- `UNVERIFIED`: final Part 1 release gate, cold/incremental equivalence after this slice, and full LSP compatibility.
- `BASELINE`: `phalcom-lsp` constructor-factory hover test fails at baseline `a3f932e0` because top-level binding `x` is absent from callable-local formal products; recorded as deferred LSP boundary, not caused by this run.
- `VERIFIED`: `RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic` passed all unit/integration/doc tests; `cargo check -p phalcom-lsp` and `cargo fmt --all -- --check` passed; LSP integration recorded 51 passed, 2 ignored, 1 same baseline failure.
- `DEFERRED`: Part 2/3 takeover and later completeness listed above.
