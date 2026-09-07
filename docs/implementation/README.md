# Implementation records

This directory is the canonical home for Phalcom implementation programs,
checkpoints, plans, technical specifications, and lifecycle metadata.

## Hierarchy

```text
category/
  PROGRAM001-program-name/
    PROGRAM.md
    STATUS.md
    C1-checkpoint-name/
      CHECKPOINT.md
      STATUS.md
      P1-plan-name.md
      P2-follow-up-plan.md
      topic-spec.md
```

Identifiers use four-letter category codes and dot-qualified records:

```text
Program:    TEST001
Checkpoint: TEST001.C1
Plan:       TEST001.C1.P1
```

The category and program express ownership. The checkpoint expresses a bounded
acceptance objective. Multiple plans may belong to one checkpoint when the
original plan is incomplete or unexpected corrective work is discovered.

## Metadata

Every program, checkpoint, and plan records lifecycle metadata:

```yaml
status: PROPOSED | IN_PROGRESS | BLOCKED | DEFERRED | COMPLETE | SUPERSEDED | ABANDONED
completion: NOT_STARTED | PARTIAL | IMPLEMENTED
verification: UNVERIFIED | FOCUSED_TESTED | BASELINE_BLOCKED | RELEASE_COMPLETE
```

Incomplete and deferred work remains under its owning program and checkpoint;
status is not encoded by a separate canonical folder.

## Authority boundaries

- `docs/spec/` defines normative language behavior.
- Decision records explain why significant choices were made.
- `docs/implementation/` records how implementation work is planned and
  specified. Numbered implementation work is always a
  `P<n>-specific-name.md` plan directly under its checkpoint. Genuine technical
  or formal companions use a topic-specific `*-spec.md` name in the same
  checkpoint.
- `docs/design/` contains unresolved proposals and research.
- `docs/archive/` contains superseded or historical records.

The organization preserves substantive plan and specification content, not
legacy filenames, version labels, staged-part labels, session history, or
migration-only copies. Incomplete and deferred work is represented by the
metadata above.
