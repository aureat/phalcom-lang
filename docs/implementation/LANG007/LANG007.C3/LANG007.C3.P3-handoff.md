---
id: LANG007.C3.P3
kind: terminal-checkpoint-handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# LANG007.C3.P3 terminal handoff

P3 and checkpoint C3 are complete at `FOCUSED_TESTED`. The shared authority is
[`LANG007.C3-CHECKPOINT.md`](LANG007.C3-CHECKPOINT.md); read it before making
changes to this substrate. P3 began at completed-P2 revision `518f66d1`; the
implementation/evidence revision is `db6c60ab`. The lifecycle records are
being committed separately and do not alter the implementation revision.

## Frozen architecture

- Intrinsic source declarations bind to existing semantic and runtime owners;
  they add no second type, row, or runtime registry.
- Unit reuses the preinstalled runtime class identity. Tuple projections remain
  backed by `CompleteTypePack` / `TupleFromPacks`; Record remains backed by
  `RecordRowId`. No TypeId/RecordRow identity boundary was weakened.
- Intrinsic type members in inherent impls publish through the existing
  `TypeMemberDeclarationId` and `AssociatedSurface` path.
- TypeOnly declarations and terms remain outside runtime value lowering.
- Runtime ADTs use canonical `data`; `enum` syntax and repository-owned active
  source migrated. Metadata schema remains v8; P3 required no schema bump.
- Source index and LSP consume semantic declaration/reference identities for
  navigation, rename, and editor features.

The accepted amendments INC-C3-12, INC-C3-13, and INC-C3-14 are recorded in the
checkpoint and their incident records. The generic-owner payload correction
aligns checked catalog signatures with the existing Type-or-static-domain
builder contract from T0869/T0870. Its cold/incremental owner-kind regression
passes.

## Verification and deferred evidence

Focused evidence includes the all-target workspace build; semantic ADTs (277
passed, 25 ignored); incremental type members (24 passed); associated payload
kind and cold-parity regressions; metadata/import identity and invalidation;
proper-Type/static-domain and TypeOnly runtime boundaries; Unit runtime-class
reuse; type-meta (14 passed); source index/LSP/rename checks; VS Code LSP e2e
(11 passed); and unchanged bootstrap counts (106 discovered, 37 root-reachable,
101 executed) from revision `518f66d1`.

Deferred findings remain classified in the checkpoint: six stale AST snapshots
and one selector diagnostic, one impl witness-plan assertion, and two core
runtime invocation-layout failures reproduce on clean predecessor `9b4ca55d`;
formatter drift is inherited from the completed-P2 baseline. The full semantic
test-binary run is incomplete. These findings are outside C3 acceptance and do
not support `RELEASE_COMPLETE`; workspace-wide test and clippy certification
were not run.

## Next action

No successor LANG007 checkpoint or plan is accepted. Preserve the frozen C3
interfaces and wait for the next scoped objective. Do not invent C4/P4. Reopen
C3 only if an accepted corrective scope identifies a concrete acceptance gap;
record performance follow-ups separately if later accepted.
