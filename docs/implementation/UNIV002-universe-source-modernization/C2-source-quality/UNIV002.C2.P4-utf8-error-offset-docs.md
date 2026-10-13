---
id: UNIV002.C2.P4
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: 8d168c0ec9ed0dc6024195bbe98c5875aef44841
---

# UNIV002.C2.P4 — Utf8Error offset documentation

## Objective

Document the two public offset properties on `Utf8Error` in
`packages/universe/src/text/errors.ph`.

## Fixed scope

- `Utf8Error.validUpTo -> Int`
- `Utf8Error.errorLength -> Option<Int>`

Describe `validUpTo` as the byte count of the valid prefix before the first
invalid sequence. Describe `errorLength` as the length of that sequence when
known, otherwise `None`. Do not document exact error messages or conversion
behavior.

## Invariants

- Preserve getter signatures, bodies, constructors, and `message` behavior.
- Wording must match `docs/specs/text.md` §6.
- No behavior/API change, Category B/C modernization, or runtime/data work.

## Tasks

### T1 — confirm source/spec contract

Check the exact getters and normative text. Stop if the live contract conflicts
with the proposed wording.

### T2 — document

Add only one concise doc comment per getter.

### T3 — review and validate

Review the scoped diff against `docs/specs/text.md` §6 and run
`git diff --check`. Do not run tests for this comments-only change. Update the
C2 checkpoint and create the plan walkthrough and handoff.

## Acceptance gate G1

- Only docs on the two getters change.
- Wording matches the normative valid-prefix and optional-length semantics.
- Getter bodies, signatures, constructors, and error behavior are unchanged.
- `git diff --check` passes; no tests are added or run.

## Outcome

Completed as scoped. Both comments match `docs/specs/text.md` §6;
`git diff --check` passed. No tests were added or run for this comments-only
change.
