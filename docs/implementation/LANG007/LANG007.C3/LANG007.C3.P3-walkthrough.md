---
id: LANG007.C3.P3
kind: implementation-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
starting_revision: 518f66d19941eb24e379e60f64aa89df66643a8a
implementation_revision: db6c60ab
---

# LANG007.C3.P3 walkthrough

## Outcome

P3 migrated C3's intrinsic structural surface, canonical Universe and corpus
source, parser/tooling, and specification ownership to the unified `data` /
`type data` model. Intrinsic facades bind existing semantic and runtime owners;
they do not create parallel registries. C3 is `COMPLETE / IMPLEMENTED /
FOCUSED_TESTED`. The implementation revision is `db6c60ab`; this walkthrough
and the terminal handoff are lifecycle-only follow-up records.

## Architecture and migration

- Unit binds to and reuses the preinstalled runtime class identity.
- Tuple exposes intrinsic members through inherent impls and delegates its
  structural entries to `CompleteTypePack` / `TupleFromPacks`.
- Record remains row-owned through `RecordRowId`; it is not coerced into a
  `TypeId` application.
- Intrinsic type members retain canonical
  `TypeMemberDeclarationId` identity and publish through the existing
  `AssociatedSurface` collision and lookup authority.
- TypeOnly declarations remain static semantic products and are rejected as
  runtime values. No VM static solver or TypeOnly runtime class was added.
- Repository-owned active enum source migrated to `data`; legacy enum syntax
  was retired after migration. The parser-backed census found no positive enum
  declarations after migration.
- Source index/LSP navigation, rename, and editor products consume semantic
  identities. VS Code grammar and specification ownership were updated to the
  final declaration model.
- Metadata schema remains v8; P3 changed no persisted schema shape.

The accepted T4 amendments are INC-C3-12 (establish and test existing-owner
binding/reuse before source migration), INC-C3-13 (support intrinsic type
members in inherent impls via the existing associated-surface pipeline), and
INC-C3-14 (preserve Tuple pack and Record row applications while projecting
the existing runtime class form). These decisions are captured in the C3
checkpoint and incident records.

The final certification correction fixed checked associated static-case
signatures for proper `Type` payloads. The catalog-backed path now follows the
existing associated builder and T0869/T0870, which allow constructor payload
kind `K`. A regression asserts the published payload kind, and an incremental
owner-kind edit from `Type` to `Shape` rebuilds the checked signature and
matches cold analysis.

## Verification performed

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets` — pass.
- Semantic ADT module — 277 passed, 25 ignored.
- Semantic incremental type-member module — 24 passed, including the new
  associated payload owner-kind cold-parity regression.
- Focused associated static-domain payload signature, generic-header cycle
  dependency, metadata identity/owner environment, imported provider
  invalidation and specialization, type-data edit parity, TypeOnly runtime
  rejection, ProperType/static-domain boundary, and named static-kind binder
  checks — pass.
- Unit intrinsic facade reuses the existing runtime class — pass.
- `cargo test -p phalcom-type-meta` — 14 passed.
- Static-domain source-index and LSP navigation/symbol/rename checks — pass.
- `PATH=target/debug:$PATH npm run test:lsp:e2e` — 11 passed.
- Universe bootstrap counts match starting revision `518f66d1`: 106 discovered,
  37 root-reachable, 101 executed.
- `git diff --check` — pass. `cargo fmt --all -- --check` reports inherited
  workspace formatting drift; broad formatting changes were not retained.

## Deferred findings and limits

The following failures reproduce on clean predecessor `9b4ca55d` and are
recorded as baseline findings outside C3 acceptance:

- AST: six stale associated-type-data snapshots and one selector-diagnostic
  assertion.
- Semantic impl queries: one data-component witness-plan assertion.
- Core algebraic data: two runtime constructor-reference invocation-layout
  failures (`1/2 structural slots for Method(0)`).

Workspace formatting drift is inherited from the completed-P2 baseline. The
full semantic test-binary run was stopped after high-fanout incremental stress
tests exceeded the useful gate window; it is incomplete, not a pass. Workspace
test and clippy release gates were not run. These limits are recorded in the
checkpoint and do not imply `RELEASE_COMPLETE`.

## Lifecycle and next action

T1–T10 and G1–G11 are closed with focused evidence or an explicit
clean-predecessor disposition. P3 and C3 are `COMPLETE / IMPLEMENTED /
FOCUSED_TESTED`; no acceptance-overlapping failure remains unresolved. The
checkpoint is the durable ledger for the three accepted T4 amendments, gate
results, and deferred baselines.

No successor LANG007 checkpoint or plan is currently accepted. Wait for the
next scoped LANG007 objective; do not infer C4/P4 from this closure. The
implementation is not release-certified.
