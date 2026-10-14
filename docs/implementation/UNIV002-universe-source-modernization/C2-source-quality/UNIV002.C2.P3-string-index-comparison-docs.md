---
id: UNIV002.C2.P3
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: 8d168c0ec9ed0dc6024195bbe98c5875aef44841
---

# UNIV002.C2.P3 — StringIndex comparison documentation

## Objective

Add concise, behavior-accurate docs to the five comparison operations in
`packages/universe/src/scalar/string_index.ph`.

## Fixed scope

- `StringIndex.compare`
- `StringIndex.<`
- `StringIndex.<=`
- `StringIndex.>`
- `StringIndex.>=`

Document comparisons as operating on the underlying byte offsets. Do not
document wrong-type guard behavior as a new contract. Do not change method
bodies, signatures, the equality/hash implementations, or any other file.

## Invariants

- Preserve every signature, expression, runtime guard, and error path.
- Comparison ordering remains the existing ordering of `_offset` values.
- Documentation must not imply that an index carries identity for a particular
  text value; `StringIndex` is an offset value as stated in `docs/specs/text.md`.
- No Category B/C modernization or runtime/data work is included.

## Tasks

### T1 — confirm source/spec contract

Review the live methods and `docs/specs/text.md` §2. Stop if the source or
normative contract changed such that the planned docs would overstate behavior.

### T2 — document

Add one concise doc comment to each of the five methods. Do not modify bodies or
other declarations.

### T3 — review and validate

Review the source diff against the live implementation and spec. Run
`git diff --check`. Do not run tests for this comments-only change. Update the
C2 checkpoint and create the plan walkthrough and handoff.

## Acceptance gate G1

- Only docs on the five listed methods change.
- Docs describe byte-offset comparison without changing or overstating the
  public contract.
- All bodies, signatures, guards, and error paths are unchanged.
- `git diff --check` passes; no tests are added or run.

## Outcome

Completed as scoped. The docs match the live byte-offset comparison bodies and
the `StringIndex` contract in `docs/specs/text.md` §2. `git diff --check`
passed; no tests were added or run for this comments-only change.
