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
| `UNIV002.C2.P8` | Buffer.capacity documentation | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Added one comment documenting the maximum element count; getter body/signature unchanged. Scoped diff check passed; inventory reconciliation retained 174 paths and records 12 changed source hashes. Semantic status remains unverified; shadow runtime remains blocked. |
| `UNIV002.C2.P9` | Bag.count documentation | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Added the occurrence-count comment; exact source review and whitespace check passed, and the 174-path inventory reconciled with 13 changed hashes. |
| `UNIV002.C2.P10` | Reader.run documentation | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Added one comment explaining that run invokes the computation with the supplied environment; exact source review and whitespace check passed, and the 174-path inventory reconciled with 14 changed hashes. |
| `UNIV002.C2.P11` | Reader.map documentation | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Added one comment describing the transformed value and preserved environment; exact source review and whitespace check passed, and the 174-path inventory reconciled with 14 changed hashes. |
| `UNIV002.C2.P12` | Writer.map documentation | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Added one comment documenting value transformation and log preservation; exact source review and whitespace check passed, and the 174-path inventory reconciled with 15 changed hashes. |
| `UNIV002.C2.P13` | State.run documentation | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Added two comment lines documenting the initial state input and StateStep outputs; exact source review and whitespace check passed, and the 174-path inventory reconciled with 16 changed hashes. |
| `UNIV002.C2.P14` | Writer.mapLog documentation | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Added one comment documenting log transformation and value preservation; exact source review and whitespace check passed, and the 174-path inventory reconciled with 16 changed hashes. |
| `UNIV002.C2.P15` | Either.mapLeft documentation | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Added one comment documenting Left transformation and Right preservation; exact source review and whitespace check passed, and the 174-path inventory reconciled with 16 changed hashes. |
| `UNIV002.C2.P16` | Chain.fold documentation | IN_PROGRESS | PARTIAL | UNVERIFIED | One source comment is planned; implementation and inventory reconciliation are pending. |

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
  commit is `58e2a31ec8d6823cb10d68e07def0d53cb62f94f`. P8's source-only
  commit is `e24f37945b1dd6349932013cb6ebf89f33cf979c`.
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

P8 added one documentation line immediately above `Buffer.capacity` in
`collections/buffer.ph`, describing the maximum element count the buffer can
hold. The getter body, signature, and module ownership are unchanged. Scoped
`git diff --check` passed; no tests were run. An independent inventory
comparison found 174 unchanged source paths and 12 changed hashes relative to
MOD1, with `collections/buffer.ph` added to the P7 overlay. Semantic linking
remains `UNVERIFIED` and shadow runtime remains `BLOCKED`.

P9 added one documentation line immediately above `Bag.count`, stating that
it returns the number of occurrences of the supplied value in the bag. The
source wrapper delegates to `rawCount`; the native lookup returns the stored
multiplicity or zero when absent. Only a comment changed. Exact diff review
and scoped `git diff --check` passed; no tests were run. Independent inventory
reconciliation confirmed 174 matching paths and 13 changed hashes against
MOD1, with `collections/bag.ph` recorded at
`8c291717d8eddd99d615f442bc7f4540a42e046ac3a8a494a96f6185eb271a47` through
source commit `a7fb21a9700b5ab038ff70021392f5be521d6a0b`. Semantic linking
remains `UNVERIFIED` and shadow runtime remains `BLOCKED`.

P10 added one documentation line immediately above `Reader.run`, stating that
it runs the computation with the supplied environment. The method delegates
to the stored computation with that environment; the comment adds no callback
count, purity, exception, or memoization promise. Only a comment changed.
Exact diff review and scoped `git diff --check` passed; no tests were run.
Independent inventory reconciliation confirmed 174 matching paths and 14
changed hashes against MOD1, with `functional/reader.ph` recorded at
`53e1ccb0137ab2c8ecb6f5508a856d21498b881c592d07afd78c0df8e9d74465` through
source commit `9cd38fab3fe054e493e7a224f9695c18607c0849`. Semantic linking
remains `UNVERIFIED` and shadow runtime remains `BLOCKED`.

P11 added one documentation line above `Reader.map`, stating that when run it
transforms the produced value and retains the same environment. The closure
runs the original Reader with the supplied environment, then applies `f` to
the produced value. The comment makes no eager-execution, callback-count,
purity, exception, or memoization promise. Only a comment changed. Exact diff
review and scoped `git diff --check` passed; no tests were run. Independent
inventory reconciliation confirmed 174 matching paths and 14 changed hashes
against MOD1; `functional/reader.ph` is recorded at
`51c0c95da302b4b590c3860bbde14e1bb77a6525293409c32bc36515d39493b6` through
source commit `42982dd6daa9c616a025236407c442e36872b29d`. Semantic linking
remains `UNVERIFIED` and shadow runtime remains `BLOCKED`.

P12 added one documentation line above `Writer.map`, stating that it applies
`f` to the value and preserves this Writer's log. The constructor applies the
callback to `self.value` and passes through `self.log` unchanged. Only a
comment changed. Exact diff review and scoped `git diff --check` passed; no
tests were run. Independent inventory reconciliation confirmed 174 matching
paths and 15 changed hashes against MOD1; `functional/writer.ph` is recorded
at `71d2bdf4af929aa0db0ab0de7e4a9e439760b37e203be82fce2a2952979a230e` through
source commit `de77efaf6b4019ccea426ff51ffa300d925af364`. Semantic linking
remains `UNVERIFIED` and shadow runtime remains `BLOCKED`.

P13 added two documentation lines above `State.run`, stating that it runs the
transition with the supplied initial state and that the returned `StateStep`
contains the produced value and successor state. The method body, signature,
and module ownership are unchanged. Exact diff review and scoped
`git diff --check` passed; no tests were run. Independent inventory
reconciliation confirmed 174 matching paths and 16 changed hashes against
MOD1; `functional/state.ph` is recorded at
`23759cc11a18253e08fc0061787edb3fad85fb8154e96d5a65a7e69572d199f9` through
source commit `44beb521c830c836a70f827ca7288c7345b3e2ba`. Semantic linking
remains `UNVERIFIED` and shadow runtime remains `BLOCKED`.

## Deferred candidate

`TextCursor.next` has a candidate rewrite from `isSome`/`unwrap` to
`if let Option::Some`, with a clear local single-evaluation equivalence proof.
Do not apply it while the Option identity, normative-spec, and shadow-linking
work is active: the new explicit pattern would exercise that unresolved seam.
The source remains unchanged. Revisit after the architect-guided Option
identity work is complete, or choose a Category B slice independent of those
identities.

## Active plan and next action

- Active plan: `UNIV002.C2.P16`, documenting `Chain.fold`. The plan authorizes
  one comment only and leaves `collections/iteration/**` untouched.
- Latest complete plan: `UNIV002.C2.P15`; latest complete checkpoint: `UNIV002.C1`.
- The source inventory is reconciled through P15 source commit
  `88a46e226ea2d899df654b85b8d526aa9b6da92c`: 174 paths, no additions or
  removals, and 16 changed hashes with the rest matching MOD1. Textual
  inventory heuristics remain baseline-only.
- P6 is complete as a docs-only change to `ZoneTransition`. Keep its wording
  grounded in the live source; `docs/implementation/time.md` §29 proposes
  additional derived members that are not implemented in this type.
- P7 is a Category A comment-only edit in `time/offset.ph`; the live method
  enforces the inclusive `-93_599..93_599` range and raises `ArgumentError`
  outside it. The range is not asserted to be a normative language rule.
- P7 completed as a documentation-only slice. P8 documented
  `Buffer.capacity`; P9 documented the `Bag.count` occurrence-count contract.
  Both are bounded Category A edits independent of the held identity,
  iteration ownership, reflection, and active linker work. Give any Category
  B/C change its own plan and exact validation boundary.
- P10 documented `Reader.run` in the functional package. The exact source
  comment does not alter code; preserve the held identity, iteration,
  reflection, and active linker boundaries.
- P11 completed a bounded Category A comment in `functional/reader.ph::Reader.map`.
  The source creates a deferred Reader that applies `f` to the produced value
  while retaining the same environment. No behavior or identity change was in
  scope.
- P12 completed a bounded Category A comment in `functional/writer.ph::Writer.map`.
  The method applies `f` to the stored value and carries the stored log into
  the result. No behavior or identity change was in scope.
- P13 completed a bounded Category A comment in `functional/state.ph::State.run`.
  The method invokes the transition with the supplied initial state and
  returns a `StateStep` containing the produced value and successor state.
- P14 completed a bounded Category A comment in
  `functional/writer.ph::Writer.mapLog`; the method transforms the log and
  carries the existing value into the result. Only a comment changed; no
  semantic or runtime claim is made.
- P15 completed a bounded Category A comment in
  `functional/either.ph::Either.mapLeft`; the method transforms a Left payload
  and passes a Right payload through unchanged. Only a comment changed; no
  semantic or runtime claim is made.
- P16 is active as a bounded Category A comment in
  `functional/chain.ph::Chain.fold`; the method starts from the supplied seed
  and processes linked elements from head to tail. Do not change executable
  source or the separate iteration subsystem.

P14 added one documentation line immediately above `Writer.mapLog`, stating
that it applies `f` to the log and preserves this Writer's value. The method
body and signature are unchanged. Exact diff review and scoped
`git diff --check` passed; no tests were run. Independent inventory
reconciliation confirmed the same 174 `.ph` paths and sixteen changed hashes
against MOD1; `functional/writer.ph` is recorded at SHA-256
`69566557fba46dff65545669b9106d4da00bd86f40f18ef0b9c31ae325518a0a` through
source commit `cb6ceb0ebe53cdded4e57effddc4c91665dde403`. Semantic linking
remains `UNVERIFIED` and shadow runtime remains `BLOCKED`.

P15 added one documentation line immediately above `Either.mapLeft`, stating
that it transforms a `Left` payload and leaves a `Right` payload unchanged.
The method body and signature are unchanged. Exact diff review and scoped
`git diff --check` passed; no tests were run. Independent inventory
reconciliation confirmed the same 174 `.ph` paths and sixteen changed hashes
against MOD1; `functional/either.ph` is recorded at SHA-256
`338c9cc2a4220717c8f70f7b3116cc0eff343c21afbee7f1a81be4400f7ce716` through
source commit `88a46e226ea2d899df654b85b8d526aa9b6da92c`. Semantic linking
remains `UNVERIFIED` and shadow runtime remains `BLOCKED`.
