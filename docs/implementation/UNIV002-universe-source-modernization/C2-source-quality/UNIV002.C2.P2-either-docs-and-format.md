---
id: UNIV002.C2.P2
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: 8d168c0ec9ed0dc6024195bbe98c5875aef44841
---

# UNIV002.C2.P2 — Either API documentation and match formatting

## Objective

Apply a bounded Category A source-quality edit to the stable shadow module
`packages/universe/src/functional/either.ph`. Document five public operations
and expand three compact match expressions while preserving their behavior.

## Fixed scope

- Add behavior-accurate docs to `Either.fold`, `isLeft`, `isRight`, `left`,
  and `right`.
- Expand the compact matches in `isLeft`, `left`, and `right` to the
  established multiline style.
- Do not modify `mapLeft`, `mapRight`, `bimap`, `swap`, `merge`, module docs,
  declarations, or any other file under `packages/universe/src/**`.

## Invariants

- Preserve all public signatures, types, module ownership, and match scrutinees.
- `fold` invokes exactly the callback selected by the current variant.
- `isLeft` reports the current `Left`/`Right` variant; `isRight` remains the
  inverse of `isLeft`.
- `left` yields `Some(value)` only for `Left`; `right` yields `Some(value)` only
  for `Right`; preserve the opposite-side `None` branches.
- No Category B/C behavior modernization or data/runtime work is included.

## Tasks

### T1 — confirm live scope

Confirm the exact methods and verify the source remains outside active module
moves, collection iteration ownership, reflection/runtime work, and P4 semantic
linking. Adapt mechanically if the source changed since the inventory snapshot;
stop if an ownership or API conflict exists.

### T2 — document and format

Add concise docs and expand only the three listed matches. Do not change their
arms, payloads, evaluation order, or callbacks.

### T3 — review and validate

Review the exact source diff. Run
`RUSTFLAGS='' cargo test -p phalcom-modules --test universe_shadow_sources universe_shadow_sources_parse_and_have_valid_unlinked_interfaces -- --exact`
and `git diff --check`. The parser/interface test does not typecheck method
bodies, establish linked shadow identities, or execute shadow code. Update the
C2 checkpoint and create the plan walkthrough and handoff.

## Acceptance gate G1

- Only the five named methods in `functional/either.ph` change.
- Docs match the live behavior and each match retains its exact prior arms.
- Public interfaces and module ownership are unchanged.
- The focused shadow parser/unlinked-interface test passes with one selected
  test, and `git diff --check` passes.
- No semantic-link or runtime verification is claimed.

## Outcome

Completed as scoped. The exact parser/unlinked-interface test selected one
test and passed; `git diff --check` passed. No semantic-link or runtime claim
is made.
