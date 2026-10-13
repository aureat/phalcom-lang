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
| `UNIV002.C2.P5` | Gregorian leap-year predicate expression | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Exact shadow parser/interface test passed; source-level divisibility review passed. Semantic linking remains unverified and shadow runtime remains blocked. A separate Boolean corpus attempt failed before fixture execution during canonical import resolution; classified unrelated to P5 and left with concurrent MODL001 P4 work. |
| `UNIV002.C2.P6` | ZoneTransition type documentation | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Added one type-level comment describing the transition timestamp and offsets before/after. No executable code changed; `git diff --check` passed. |
| `UNIV002.C2.P7` | UtcOffset.seconds documentation | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Added two method documentation lines before `@class`; source body/signature unchanged. Scoped diff check passed; independent inventory reconciliation retained 174 paths and records 11 changed source hashes. Semantic status remains unverified; shadow runtime remains blocked. |

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

## Checkpoint source and concurrent-work state

- Branch: `feat/telos-collections`.
- P7 started from `146be31eb76c2c9649fa30f627a2819d02a1041e`; its source-only
  commit is `58e2a31ec8d6823cb10d68e07def0d53cb62f94f`.
- Concurrent P4 edits in `phalcom-semantic/src/resolver.rs`,
  `phalcom-semantic/src/session.rs`, `phalcom-semantic/src/workspace.rs`, and
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

P5 replaces `Gregorian.isLeapYear`'s two early returns with the equivalent
Gregorian predicate. The exact shadow parser/unlinked-interface check selected
one test and passed; a source-level review covers year zero and negative years
using exact integer remainder semantics. This does not typecheck or execute the
shadow method, so semantic linking is `UNVERIFIED` and shadow runtime parity is
`BLOCKED`. A supplemental Boolean corpus attempt selected one test but failed
before fixture execution while resolving the canonical import
`universe.numbers.capabilities` as private at `phalcom-semantic/src/session.rs:114:41`.
That failure is unrelated to the P5 shadow source and is left with concurrent
MODL001 P4 work; it was not rerun. The initial `--exact` variant selected zero
tests and is not evidence.

P6 added one type-level documentation comment to `ZoneTransition`, describing
its stored timestamp and before/after offsets. No declaration or executable
line changed. `git diff --check` passed; no tests were run for this
comments-only slice. The comment deliberately does not promise `shift`,
`isGap`, or `isOverlap`, which appear in an implementation design sketch but
not in the live source.

P7 added two method documentation lines before the `@class` annotation for
`UtcOffset.seconds`, matching the existing declaration-comment convention.
They describe whole-second input, the inclusive `-93_599..93_599` bounds, and
the live `ArgumentError` path. The method body, signature, and module ownership
are unchanged. Scoped `git diff --check` passed; no tests were run. An
independent inventory comparison found 174 unchanged source paths and 11
changed hashes relative to MOD1, with `time/offset.ph` added to the P6 overlay.
Semantic linking remains `UNVERIFIED` and shadow runtime remains `BLOCKED`.

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
- Latest complete plan: `UNIV002.C2.P7`; latest complete checkpoint: `UNIV002.C1`.
- The source inventory is reconciled through P7 source commit
  `58e2a31ec8d6823cb10d68e07def0d53cb62f94f`: 174 paths, no additions or
  removals, and 11 changed hashes with the rest matching MOD1. Textual
  inventory heuristics remain baseline-only.
- P6 is complete as a docs-only change to `ZoneTransition`. Keep its wording
  grounded in the live source; `docs/implementation/time.md` §29 proposes
  additional derived members that are not implemented in this type.
- P7 is a Category A comment-only edit in `time/offset.ph`; the live method
  enforces the inclusive `-93_599..93_599` range and raises `ArgumentError`
  outside it. The range is not asserted to be a normative language rule.
- P7 completed as a documentation-only slice. Select the next stable source
  modernization independently of the held identity, iteration ownership,
  reflection, and active linker work; give any Category B/C change its own
  plan and exact validation boundary.
