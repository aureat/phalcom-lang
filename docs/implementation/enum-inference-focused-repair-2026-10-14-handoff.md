# Enum inference audit-follow-up handoff

Date: 2026-10-14. Branch: `feat/integration-universe`. Base: `9b283a03`.

## Current state and completion boundary

The audit's primary repeated-specialization expression defect, shared discovery,
isolated candidate checking, symbolic union annotation loss and private
construction-access defects are **IMPLEMENTED / FOCUSED_TESTED**. Source changes
and this record remain uncommitted/unpushed. Full enum-inference requirements
remain **PARTIAL**. Do not label semantic or executable requirements COMPLETE.

The repair record owns the consultation decisions, exact test evidence and
historical context: [repair record](enum-inference-focused-repair-2026-10-14.md).

## Stable takeover interfaces

- `checker/contextual_variant.rs`: `ContextualOwner` retains declaration,
  canonical owner type or symbolic shell, branch-specific `ExpectedType`, context
  identity and exact `VariantId` via `ContextualVariantCandidate`. `discover`
  records every eligible enum owner dependency, matches cases within those
  owners and removes only canonically GADT-refuted cases.
- `CheckingContext::run_candidate_probe`: complete context copy; inference type
  and row coordinators deep-fork with row weak references rewired; flow, rigids,
  diagnostics and publication products are private. Semantic/query dependencies
  remain tracked and callable dependencies are merged. Budgets/cancellation are
  shared. Do not substitute `run_flow_probe` or shallow `Rc` cloning.
- `InferenceCoordinator::fork`: preserves query-local identities while detaching
  both solver stores. Candidate consumers return viability, never publish probe
  products. Unique winners execute again live through associated application.
- `expression::synthesize_contextual_variant`: applies candidate-specific
  expectations before counting viable interpretations. Unknown/blocked/status
  failures are not negative proof. Distinct singleton specializations remain
  ambiguous; diagnostic displays specialization types.
- `annotation_elaboration`: unions containing holes recurse within the same
  frame/context and preserve proper-kind checking for canonical members.
- `associated::authorize_variant_construction`: existing declaration access
  policy for associated acquisition/invocation, including references and
  singleton values. Construction permission does not redefine pattern or
  payload visibility.
- Pattern singleton and constructor discovery share the service; pattern local
  proof, field joining, coverage and exact repeated-owner opening remain intact.

## Verified evidence

- Contextual enum filter: 22 passed, including rejected-owner incremental edit
  parity with cold analysis.
- Associated lookup: 25 passed.
- Pattern matching: 163 passed, 16 pre-existing ignored.
- Type/row solver isolation regression: 1 passed.
- Linker: 19 passed.
- Scoped diff whitespace check passed.

All Cargo gates used `RUSTFLAGS='' RUSTC_WRAPPER=''` serially. See the repair
record for timing/order limits. No release certification was requested or run.

## Executable blocker

The named core discriminator fails before enum fixture execution:
`canonical Universe runtime bootstrap ... universe:callable.function ...
finalized executable link has no runtime read for Error`.

This reproduces the prior blocker. Preserve strict runtime read and cycle/DAG
validation. Diagnose canonical semantic import/read ownership at its owning
module/Universe layer in a separate bounded repair; do not insert late reads,
qualify fixture syntax to hide defects or bypass validation.

## Residuals and next action

Full closure still requires explicit acceptance for overload candidate-local
expectations, general local-rigid/projection viability, generic mutable and
projection-flow retention, comprehensive incremental invalidation and linked
execution. The canonical mutable regression and one incremental edit do not
certify those broader requirements. Required `let` variant destructuring and
Universe de-qualification remain out of scope.

Preserve concurrent edits. In particular, `packages/universe/src/object/object.ph`
changed externally during this task and is not task-owned. The user authorized
restoring task-owned edits after an external revert; this does not authorize
reverting other work. Stage only this work unit if delivery is requested.
