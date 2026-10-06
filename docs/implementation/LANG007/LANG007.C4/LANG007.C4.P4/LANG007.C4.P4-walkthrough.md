---
id: LANG007.C4.P4
kind: implementation-walkthrough
title: "LANG007.C4.P4 — Generalized FieldKey and Unordered-Product Calculus"
status: INTERIM
completion: PARTIAL
verification: UNVERIFIED
recorded_against_revision: 548c7a3629832b50c2d5bfd35529abc759de322e
recorded: 2026-10-06
---

# LANG007.C4.P4 — Interim walkthrough

> This is an interim lifecycle record created while the plan is still active.
> It is not plan-completion evidence and does not change the plan or checkpoint
> status: P4 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`.

## Objective and current outcome

P4's objective is to carry the canonical heterogeneous Record-key model and
unordered-product behavior through the language stack without changing the
existing row solver, static evaluator, or ordered Tuple/Pack/call-label model.
The plan's T1–T6 slices are recorded complete and focused-tested in the C4
checkpoint. Work has advanced into T7.0, which projects computed static keys
into types and signatures before their canonical owners publish.

The current slice establishes projection at field and callable signature
owners, static-constant declared types, and an exact session dispatch path for
some checked type-member prerequisites. It does **not** establish the complete
type-owner integration or T7.0 acceptance. Computed-key type-alias owner
scheduling remains unimplemented; T7.0E has not passed, so T7.1 has not begun.

## Work and architecture established

- The semantic layer owns `FieldKey` and projected Record-key facts. Static key
  expressions continue to use the existing P2 evaluator; P4 adds no evaluator.
- Field and callable signature formation project selected computed keys before
  canonical signature publication. Downstream declaration surfaces consume
  those canonical signatures and retain exact owner dependencies.
- Static-constant declared-type formation discovers selected constant reads,
  obtains computed-key projections, and records exact dependencies in the
  owning `StaticConstant` product. Direct mixed constant/type-formation cycles
  are covered in the focused static-constant slice.
- A session-owned demand path now carries exact not-current type-member
  prerequisites out of checked-expression analysis. The session advances them
  through the existing canonical member owner path, refreshes its resolver and
  checker context, and retries the suspended projection. Awaiting attempts do
  not publish their diagnostics or products.
- `Awaiting(exact demands)` is distinct from a source-invalid checked
  expression. The checked-expression boundary and demand dispatcher are
  implemented, but their end-to-end resume/no-leak behavior remains to be
  proven by a focused integration test and T7.0E.
- INC-1 through INC-5 clarified the T7.0D scheduler contract. In particular,
  only computed-key aliases are deferred; they must remain absent from alias,
  static-definition, and resolver products until their canonical
  `StaticDefinition` owner can publish. That alias-owner routing is not yet
  implemented.

The durable architecture, frozen invariants, and all accepted consultation
decisions remain recorded in the C4 checkpoint and active plan. This walkthrough
does not supersede them.

## Files and implementation areas

The implementation at the recorded revision includes changes in the semantic
static-constant demand driver, checker context/readiness handling, query
dependency publication support, and session owner integration. Focus areas for
the continuation are:

- `phalcom-semantic/src/static_constant_demand.rs`
- `phalcom-semantic/src/checker/context.rs`
- `phalcom-semantic/src/db/query.rs`
- `phalcom-semantic/src/session.rs`

The projection-aware static-definition query helper exists, but the current
session does not mark computed-key aliases deferred or route exact
`StaticDefinition` demands through a completed alias-owner worklist. Helper
availability is not end-to-end alias support.

## Focused evidence

The checkpoint records the following completed focused checks for the relevant
slices:

- `foundations::static_constants`: 18/18 for field/callable owner projection
  coverage, exact dependency propagation, incremental republication, and
  cold/incremental parity at that slice.
- `foundations::type_members`: 11/11 for the type-member integration slice.
- `integration::static_definition_bodies`: 5/5 for the earlier static-constant
  owner slice.
- `cargo check -p phalcom-semantic`: passed for the owner-dispatch production
  changes.
- A `--lib` probe for the exact deferred-static-definition resolver was run in
  an earlier slice; later checkpoint evidence records the exact type-member
  resolver probe as unverified because unrelated untouched record-row
  inference test code prevented that library test target from compiling.

For the current committed state, `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check
-p phalcom-semantic` and the filtered exact deferred-static-definition library
test passed; `git diff --check` passed. These checks do not prove alias demand
routing, fresh-context retry, no-partial-publication, transitive mixed-cycle
unwinding, or T7.0E. No compilation or test command was run for this
documentation-only record creation.

## Deferred verification and remaining scope

Not established by this interim record:

- the full T7.0 cold/incremental acceptance matrix (T7.0E);
- completed deferred computed-key alias scheduling, exact `StaticDefinition`
  demand routing, and owner publication;
- end-to-end fresh-context retry and no-diagnostic/no-product leakage when an
  owner suspends;
- imported computed-key forcing and transitive mixed-cycle diagnostics;
- T7.1–T7.5 pattern semantics/runtime work, T8 metadata/incrementality/perf
  closure, and T9 stabilization/certification.

The plan's broader metadata compatibility, exhaustive migration audit,
large-Record performance evidence, and release/stabilization gates have not
been claimed here. A prior repository-wide `cargo fmt --all -- --check`
reported formatting differences outside this scoped P4 work; do not apply a
repository-wide formatting rewrite as part of the continuation.

## Current classification and next action

P4 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`; the checkpoint is the shared
status authority. Continue with amended T7.0D: implement deferred computed-key
alias owner scheduling, exact demand dispatch through the canonical
`StaticDefinition` publisher, prerequisite servicing, and active-stack cycle
unwinding. Add focused tests for suspension, exact dependencies, retry without
partial publication, and aliases. Then run and review every T7.0E cold and
incremental case. Do not start T7.1 unless T7.0E passes.
