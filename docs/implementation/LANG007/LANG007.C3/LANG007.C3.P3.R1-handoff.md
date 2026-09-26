---
id: LANG007.C3.P3.R1
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
prepared: 2026-09-25
---

# LANG007.C3.P3.R1 handoff

## State

P3.R1 completed the corrective closure and restored C3 at that time to
`COMPLETE / IMPLEMENTED / FOCUSED_TESTED`. P4 has since reopened C3 for the
accepted structural-data completion objective. Workspace/release certification
remains unclaimed.

## Stable model

`DeclarationTypeInfo::binding` is the canonical owner of two sibling
projections:

- `semantic_form`: ordinary application or intrinsic Unit/Tuple/Record
  application behavior;
- `runtime_owner`: optional existing VM/native Unit/Tuple/Record owner.

Semantic annotation formation consumes `semantic_form`. Native surface import,
runtime form selection, and runtime facade lowering consume `runtime_owner`.
Declaration-shell replacement preserves a registered projection when an
ordinary shell refresh carries the default binding. Fingerprints include the
complete binding.

Keep the established identities and boundaries: Tuple application flows
through `CompleteTypePack` to `TupleFromPacks`; Record receives `RecordRowId`
and produces `TypeData::Record`; Unit semantics remain `store.unit()` while
runtime lowering reuses the installed class. Do not add another owner registry
or route RecordRow through `TypeId`/`TypeLambdaArena`.

## Evidence

The final focused evidence is listed in
[the P3.R1 walkthrough](LANG007.C3.P3.R1-walkthrough.md) and checkpoint §23.
One initial core failure identified projection loss during
ordinary shell refresh; the preservation path and its regression are now
green. The initial owner-free annotation fixtures were updated to remove the
existing declaration before inserting the projection-isolated copy.

## Residual state

Previously classified AST/impl-query/runtime and formatter baselines, plus the
incomplete broad semantic test-binary run, remain deferred as recorded in the
C3 checkpoint. They did not block the P3.R1 focused acceptance objective.

Next action: see the active P4 action in `LANG007.C3-CHECKPOINT.md`.
