# Handoff: continue implementation-documentation consolidation

This is a continuation prompt for the next agent. Continue the documentation
organization task from the current checkout; do not restart the migration from
the historical source directories.

## Mission and scope

Finish consolidating Phalcom implementation records under
`docs/implementation/` using the established hierarchy:

```text
category/
  FOURL001-specific-program-name/
    PROGRAM.md
    STATUS.md
    C1-specific-checkpoint-name/
      CHECKPOINT.md
      P1-specific-plan-name.md
      topic-spec.md
```

The identifiers are dot-qualified:

```text
Program:    TEST001
Checkpoint: TEST001.C1
Plan:       TEST001.C1.P1
```

Continue with the remaining implementation-documentation audit. In scope:

- identify broad or semantically mixed programs that still need to be split;
- organize their material into logical programs and checkpoints;
- give every canonical file a specific, descriptive name;
- make every general plan/spec heading agree with its canonical identifier;
- preserve substantive plan and formal/technical specification content;
- retain incomplete, partial, blocked, deferred, and unverified state in
  metadata rather than encoding it in directory names;
- repair canonical local links and update the implementation index when paths
  change.

Out of scope:

- changing Rust source, fixtures, tests, or language semantics;
- restoring history-only documents or old filename aliases;
- commits, pushes, resets, cleans, or broad staging;
- changing `.obsidian/workspace.json`, which is unrelated user work;
- broad formatting of pre-existing staged documentation.

## Current state

The canonical contract is already written in
`docs/implementation/README.md`. The implementation tree currently uses:

- four-letter category codes;
- stable three-digit program numbers;
- `C<number>-specific-name` checkpoint directories;
- `P<number>-specific-name.md` implementation plans directly under a
  checkpoint;
- topic-specific `*-spec.md` companions directly under a checkpoint;
- `PROGRAM.md`, `STATUS.md`, and `CHECKPOINT.md` metadata records;
- lifecycle values `PROPOSED`, `IN_PROGRESS`, `BLOCKED`, `DEFERRED`,
  `COMPLETE`, `SUPERSEDED`, and `ABANDONED`;
- completion values `NOT_STARTED`, `PARTIAL`, and `IMPLEMENTED`;
- verification values `UNVERIFIED`, `FOCUSED_TESTED`, `BASELINE_BLOCKED`, and
  `RELEASE_COMPLETE`.

The following focused reorganization is complete in the physical worktree:

- `COLL001` through `COLL005`: product model, map model, indexed/range
  behavior, collection traversal, and argument expansion;
- `CONC001`: fiber execution, scheduler/reactor, reflection, and deferred
  Future work;
- `CLIT001`: REPL foundation, session execution, intelligence, and operations;
- `NATV001`: bytes, filesystem/path, resources, streams, path runtime,
  resource runtime, standard-file bytes, and standard-file streams;
- `STDL001`: numeric contract/model, integer literals/arithmetic, Float
  protocol/text, and numeric errors/verification;
- `TYPE004`: type formation, expression semantics, interface/kind tower, and
  semantic-tower integration.

`DOCS001` owns the migration itself:

`docs/implementation/docs/DOCS001-implementation-documentation-organization/`

Its current active checkpoint is `DOCS001.C4`. Its next planned validation
checkpoint is `DOCS001.C5`, for links, identifiers, metadata, and retirement of
legacy paths.

Important worktree condition: `git status --short` contains many staged-added,
worktree-deleted (`AD`) entries for former migration paths. Those entries are
leftover staged work from the earlier migration and do not represent live
files. Inspect the physical tree with `find`/`rg --files`; do not reset, clean,
restore, or stage broadly to make the index look tidy.

## Evidence and decisions

- The user’s governing rule is: preserve actual plan/spec contents, not old
  filenames, version labels, session history, or migration-only copies.
- A regression or remaining-fix plan stays in the same checkpoint as the
  acceptance objective it serves. It does not become a separate history-based
  program or a `part-2` directory.
- A checkpoint is a bounded acceptance objective. Multiple plans may belong to
  one checkpoint when the original plan is incomplete or new corrective work
  appears.
- Broad names such as “semantic completeness” or “type system” are not
  sufficient when the material can be divided by ownership and acceptance
  objective. Split only when the contents support a real boundary; do not
  invent empty programs or duplicate material.
- Conceptual labels inside preserved plan bodies may remain when they are part
  of the technical content. Canonical paths and general headings must follow
  the new organization.
- Deferred work is represented by metadata. For example, the Future-library
  checkpoint under `CONC001` is `DEFERRED` with an explicit prerequisite
  reason; do not rename it to imply completion.
- No Rust/source changes were made for this documentation migration, so no
  Cargo tests were run.

## Code and artifact map

Read these first:

- [implementation contract](../implementation/README.md)
- [implementation index](../implementation/INDEX.md)
- [DOCS001 program](../implementation/docs/DOCS001-implementation-documentation-organization/PROGRAM.md)
- [DOCS001 status](../implementation/docs/DOCS001-implementation-documentation-organization/STATUS.md)

The recently reorganized families are under:

- `docs/implementation/coll/`
- `docs/implementation/conc/`
- `docs/implementation/clit/`
- `docs/implementation/natv/`
- `docs/implementation/stdl/`
- `docs/implementation/type/`

The broader remaining audit is primarily across the other program directories
listed in `docs/implementation/INDEX.md`, especially programs whose names or
checkpoint names still describe an entire subsystem instead of a bounded
technical objective. Start from actual files, not staged deletion records.

One known item to verify during the index audit: the `DOCS001` row in
`docs/implementation/INDEX.md` currently links to
`docs/DOCS001-implementation-documentation-organization/PROGRAM.md`, while the
physical program is under `docs/implementation/docs/`. Confirm the correct
relative link before changing it, and fix it if it is genuinely broken.

## Validation

The last focused validation over the affected category roots reported:

```text
programs=15 checkpoints=51 plans=69 specs=51 metadata_errors=0
forbidden_basenames=
canonical_local_link_errors=0
```

That validation checked path-derived identifiers and headings, required
program/checkpoint metadata, plan/spec placement, and absence of noncanonical
nested checkpoint directories. Re-run an equivalent validator after each
family migration rather than relying on visual inspection.

The scoped checks for `docs/implementation/README.md` and
`docs/implementation/INDEX.md` had no whitespace errors. A repository-wide
staged diff check still reports trailing whitespace in older staged migrated
documents; treat that as pre-existing migration noise and do not broad-format
those files as part of this task.

## Resume plan

1. Read this handoff, `docs/implementation/README.md`,
   `docs/implementation/INDEX.md`, and the current `DOCS001` records.
2. Run a concise `git status --short` and inspect the relevant physical files
   with `rg --files docs/implementation`; preserve unrelated staged and
   untracked work.
3. Inventory every actual program, checkpoint, plan, and spec against the
   path/metadata contract. Flag generic names, mixed program scope, nested
   `part-*`/version/session directories, and headings that do not identify the
   canonical record.
4. Process one remaining category or program family at a time. First decide
   whether a broad program should be split; then move/rename only the
   substantive records, update identifiers/headings/metadata, and repair local
   links.
5. Update `docs/implementation/INDEX.md` only for confirmed canonical paths,
   program descriptions, or status changes.
6. Re-run the path-derived metadata/heading validator, the canonical local-link
   check, and scoped whitespace checks. Report documentation validation only;
   Cargo tests are unnecessary unless source files are touched.
7. Update `DOCS001` status to reflect the actual active checkpoint and any
   remaining audit work. Do not mark the migration complete while legacy-path,
   link, identifier, or metadata evidence remains open.

## Do not re-explore

- Do not redesign the four-letter/program/checkpoint/plan convention.
- Do not redo the completed COLL, CONC, CLIT, NATV, STDL, or TYPE004 splits
  unless a concrete invariant violation is found.
- Do not reconstruct the removed historical directory layout.
- Do not treat staged `AD` records as evidence that the physical migration is
  incomplete.
- Do not use generic filenames such as `plan.md`, `spec.md`, `part-1`, `v0.1`,
  `sc1`, or `continuation` inside `docs/implementation`.
- Do not claim implementation completion from documentation metadata alone;
  preserve the distinctions between planned, implemented, focused-tested,
  baseline-blocked, and release-complete.
