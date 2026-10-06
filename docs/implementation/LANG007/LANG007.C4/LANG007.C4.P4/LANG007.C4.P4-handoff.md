---
id: LANG007.C4.P4
kind: implementation-handoff
title: "LANG007.C4.P4 — Interim continuation handoff"
status: INTERIM
completion: PARTIAL
verification: UNVERIFIED
recorded_against_revision: 548c7a3629832b50c2d5bfd35529abc759de322e
recorded: 2026-10-06
---

# LANG007.C4.P4 — Interim handoff

> P4 is active, not complete: `IN_PROGRESS / PARTIAL / UNVERIFIED`. This
> handoff captures the present continuation point only; it is not a completion
> record and does not replace the C4 checkpoint.

## Revision and working-tree anchor

- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Recorded revision: `548c7a3629832b50c2d5bfd35529abc759de322e`
- Recorded branch: `fix/LANG007-a1-match-equality-authority`
- At handoff, the P4 implementation and checkpoint commits are pushed and the
  branch is synchronized with its upstream. Confirm the live branch/status
  before making delivery decisions.
- Preserve the unrelated C4.1 work in
  `phalcom-semantic/tests/semantic/adts/matching/exhaustiveness.rs` and
  `docs/audit/LANG007.C4.1/`. It is outside P4 and was explicitly left
  untouched; do not stage, modify, or include it in P4 commits.

## Read first

1. `docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md` — shared
   C4 status, invariants, accepted consultations INC-1–INC-5, and current next
   action.
2. `docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4-luna-patch-grade-implementation-plan.md`
   — especially amended T7.0D/T7.0E and their acceptance criteria.
3. `docs/workflow/implementation-record-lifecycle-convention.md` — lifecycle
   rules.
4. The P4 requirements analysis and this interim walkthrough.
5. The live owner/demand code listed below and the focused tests named by the
   plan. Follow the plan's normative-spec references where behavior is in
   question.

## Established state to inherit

- T1–T6 are complete and focused-tested according to the checkpoint.
- Field and callable signature owners project computed Record keys before
  canonical signature publication. Static-constant declared-type formation
  also projects keys and owns its exact selected dependencies.
- The session has an exact type-member prerequisite path: a checked expression
  can return `Awaiting` exact demands; the session advances prerequisites
  through canonical owners, refreshes resolver/context state, and retries.
  Production checks and focused integration suites passed as recorded in the
  checkpoint, but retry/no-leak behavior is not yet adequately exercised.
- The projection-aware static-definition query helper and deferred-definition
  resolver hook are groundwork only. Actual session flow currently supplies
  no deferred computed-key alias owners; there is no completed alias
  `StaticDefinition` worklist or exact static-definition demand routing.
- Consultations INC-1–INC-5 are accepted architecture clarifications, not
  evidence that the requested behaviors are implemented or verified.

Relevant implementation locations:

- `phalcom-semantic/src/static_constant_demand.rs`
- `phalcom-semantic/src/checker/context.rs`
- `phalcom-semantic/src/db/query.rs`
- `phalcom-semantic/src/session.rs`
- focused integration tests under `phalcom-semantic/tests/semantic/`

## Fixed constraints — do not redesign

- One canonical `FieldKey` model and one existing row/relationship authority;
  no second row solver, key encoding, static evaluator, or pattern-key
  resolver.
- P2's existing static evaluator owns key-expression evaluation.
- Each semantic owner remains the sole publisher of its product and exact
  dependency set. Service prerequisites through existing owners; do not build
  or publish their products in the demand driver.
- A suspended owner resumes only with current-revision prerequisite products
  and a fresh checker context. Awaiting attempts publish neither diagnostics
  nor semantic products. Do not use stale/LKG products, provisional rows, or
  phase-wide reordering.
- Alias deferral is limited to TypeForms containing computed Record keys.
  Such an alias stays absent from `TypeAliasTable`, `StaticDefinitionTable`,
  and the resolver until its canonical `StaticDefinition` owner publishes.
  Route exact static-definition demands to that owner; use the transient
  cross-owner stack for cycles and preserve the established syntax-alias and
  static-formation diagnostics.
- Preserve the boundary between heterogeneous Record keys and Symbol-only
  Tuple/Pack/call labels.
- T7.1 is gated on a passing T7.0E. Do not begin pattern work early.

## Next implementation objective

Continue amended T7.0D, with the immediate missing slice being computed-key
alias ownership:

1. Identify/defer only aliases whose TypeForms contain computed Record keys,
   without prematurely publishing an alias/static-definition/resolver value.
2. Route an exact `QueryKey::StaticDefinition` demand to its canonical owner;
   service only ready prerequisites through existing owner paths and rebuild
   current checker context before resuming the waiting consumer.
3. Preserve active owner/query frames and deterministic cycle unwinding,
   including paths involving computed-key constants. Fail closed on unexplained
   no-progress; do not create a second publisher or use LKG values.
4. Add discriminating tests for cold alias projection, exact selected
   dependencies, changed-key invalidation/republication, suspended-demand
   retry with no partial diagnostics/products, syntax cycles, mixed cycles,
   and imported constants/aliases as required by T7.0E.
5. Run the complete T7.0E cold/incremental acceptance matrix and update the
   checkpoint only when the evidence supports the result. T7.1 remains blocked
   until that gate passes.

After T7.0E, proceed in plan order through T7.1–T7.5, T8, and T9. Do not
promote this interim handoff or P4 status based on focused slice checks.

## Verification anchor

Recorded passing evidence includes `cargo check -p phalcom-semantic`, focused
`foundations::static_constants` and `foundations::type_members` suites, and
`integration::static_definition_bodies` as detailed in the checkpoint and
walkthrough. An isolated type-member resolver unit probe remains unverified
because the library test target was blocked by untouched record-row inference
test compilation failures. No T7.0E, T7.1–T7.5, T8, or T9 gate is claimed.

Start by checking live status and the scoped diff, then continue from the
checkpoint's amended T7.0D contract. Do not re-audit the completed T1–T6 slices
without a concrete regression or plan requirement; do not treat historical
focused passes as current T7.0E evidence.
