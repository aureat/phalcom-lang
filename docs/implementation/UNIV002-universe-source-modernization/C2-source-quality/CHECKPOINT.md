---
id: UNIV002.C2
category: UNIV
program: UNIV002
checkpoint: UNIV002.C2
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
---

# UNIV002.C2 — source modernization by stable modules

## Objective and boundary

Apply the MOD2–MOD4 source-quality standards in bounded slices selected from
the accepted shadow inventory. This checkpoint owns source-only changes under
`packages/universe/src/**` and their validation records. It does not own
module-boundary changes, public API changes, class-to-data conversion, native
integration, or production cutover.

## Plan ledger

| Plan | Scope | Status | Completion | Verification | Outcome |
|---|---|---|---|---|---|
| `UNIV002.C2.P1` | Validation module docs and compact `ensure` formatting | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Category A only; focused source review and `git diff --check` passed; no behavior tests required or run. |
| `UNIV002.C2.P2` | Either API docs and compact match formatting | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Category A only; exact parser/unlinked-interface test passed and scoped diff review confirmed unchanged arms. |
| `UNIV002.C2.P3` | StringIndex comparison documentation | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Comments only; source/spec review and `git diff --check` passed; no tests required or run. |
| `UNIV002.C2.P4` | Utf8Error offset documentation | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Comments only; wording matches `docs/specs/text.md` §6 and `git diff --check` passed; no tests required or run. |

## Established invariants

- `UNIV002.C1` completed the 174-file inventory and five-function pilot.
- Shadow source is still non-production. Do not change provider, bootstrap,
  native registration, compiler, or VM ownership.
- P2 has parser/unlinked-interface evidence only. Its exact-site semantic
  status remains unverified; runtime parity remains blocked.
- Category A documentation/formatting changes may proceed without changing
  behavior. A proposed Category B/C change requires its own exact evidence and
  plan boundary.
- Keep `collections/iteration/**`, `reflection/**`, and `concurrency/fiber.ph`
  outside this plan due active ownership or architecture holds.
- The user directed that Option/Result/Unit identity follow the shadow layout
  and that the normative Universe spec be amended. This identity/path work is
  separate from C2 source quality; no such edits are included here. Carry the
  architect's guidance into a dedicated spec-backed plan before changing those
  modules or identities.

## Starting state

- Branch: `feat/telos-collections`.
- HEAD: `8d168c0ec9ed0dc6024195bbe98c5875aef44841`.
- Concurrent P4 edits in `phalcom-semantic/src/session.rs`,
  `phalcom-semantic/src/workspace.rs`, and
  `phalcom-semantic/tests/semantic/integration/resolver.rs` are unrelated and
  must not be staged or changed. Preserve the untracked
  `docs/implementation/MODL001-module-architecture/C2-module-project-model/MODL001.C2.P4-handoff.md`
  as part of that work.

## Verification ledger

P1's scoped source review confirmed only comments and whitespace changed in the
five planned functions. `git diff --check` passed. No test suite was run because
the plan changed no AST or observable operation. This does not establish
semantic linking or runtime behavior for the shadow package.

P2's exact shadow parser/unlinked-interface test selected one test and passed.
Independent review confirmed that the five documented methods and three
expanded matches retain the original signatures and arms. `git diff --check`
passed. This does not establish semantic linking or runtime behavior.

P3's five comments match the existing byte-offset operations and
`docs/specs/text.md` §2. Review and `git diff --check` passed. No tests were run
for the comments-only change; no semantic or runtime claim is made.

P4's getter comments match the valid-prefix and optional-error-length rules in
`docs/specs/text.md` §6. Getter signatures and bodies are unchanged;
`git diff --check` passed. No tests were run for the comments-only change.

## Deferred candidate

`TextCursor.next` has a candidate rewrite from `isSome`/`unwrap` to
`if let Option::Some`, with a clear local single-evaluation equivalence proof.
Do not apply it while the Option identity, normative-spec, and shadow-linking
work is active: the new explicit pattern would exercise that unresolved seam.
The source remains unchanged. Revisit after the architect-guided Option
identity work is complete, or choose a Category B slice independent of those
identities.

## Active plan and next action

- Active plan: none.
- Latest complete plan: `UNIV002.C2.P4`; latest complete checkpoint: `UNIV002.C1`.
- Next: select a high-confidence behavior-preserving syntax modernization
  independent of Option/Result/Unit identity and the active P4 linker work.
  Verify source contract and the smallest useful test before editing; escalate
  semantic or ownership uncertainty.
