---
id: UNIV002.C2.P4
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# UNIV002.C2.P4 walkthrough — Utf8Error getter docs

## Outcome

Added documentation to `Utf8Error.validUpTo` and `Utf8Error.errorLength` in
`packages/universe/src/text/errors.ph`.

## Review and evidence

- The wording matches `docs/specs/text.md` §6: `validUpTo` is the byte count
  of the valid prefix; `errorLength` is the invalid-sequence length when
  known, otherwise `None`.
- Getter signatures and bodies, constructors, and `message` behavior are
  unchanged.
- `git diff --check` passed. No tests were added or run for this
  comments-only change.

## Deviations and residuals

None. No behavior or runtime claim is made.

## Follow-up

Continue C2 with a high-confidence behavior-preserving syntax modernization in
an unaffected stable module.
