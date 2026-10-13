---
id: UNIV002.C2.P5
category: UNIV
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - UNIV002.C2.P4
follows: UNIV002.C2.P4
supersedes: null
---

# UNIV002.C2.P5 — Gregorian leap-year expression

## 0. Executor contract

Make one behavior-preserving source modernization in the shadow Universe. Do
not expand scope if semantic or identity assumptions fail. Keep all behavioral
claims bounded by the available shadow validation substrate.

## 1. Goal

Express `Gregorian.isLeapYear` with one equivalent Gregorian divisibility
expression while preserving its signature and observable result.

## 2. Checkpoint acceptance objective

This is one of several bounded source-quality plans under `UNIV002.C2`; it does
not close the checkpoint or establish linked semantics/runtime parity.

## 3. Repository grounding

Prepared against:

```text
repository: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections
branch: feat/telos-collections
revision: 7e41d1a5fd1db4678ea2c1d8347bbe4c5c1c5706
relevant predecessor: UNIV002.C2.P4
```

The shadow parser/unlinked-interface test loads all `.ph` modules under
`packages/universe/src`. It does not typecheck bodies, link canonical
identities, or execute the shadow package. Re-read named live files before
editing; adapt only mechanical drift.

## 4. Required reads before implementation

- `AGENTS.md`
- `docs/workflow/implementation-record-lifecycle-convention.md`
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/CHECKPOINT.md`
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/UNIV002.C2.P4-walkthrough.md`
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/UNIV002.C2.P4-handoff.md`
- `packages/universe/docs/modernization/GUIDELINES.md`
- `packages/universe/src/time/calendar/month.ph`
- `docs/implementation/time.md` §15
- `docs/specs/README.md` and `docs/specs/numeric-types.md`
- `phalcom-modules/tests/universe_shadow_sources.rs`

## 5. Normative authority

- `docs/specs/numeric-types.md` owns exact `Int` remainder semantics.
- `docs/implementation/time.md` §15 records the implementation's expected
  Gregorian leap-year rule and canonical examples: 2000 and 2024 leap; 1900
  and 2025 common. It is implementation guidance, not a calendar normative
  specification.
- No calendar-specific normative specification was found for this helper.
  This plan preserves the existing source algorithm and does not claim a new
  calendar contract.
- Current production-Universe fixtures are not evidence for the changed
  shadow source.

## 6. Takeover state

`UNIV002.C2.P1`–`P4` completed Category A documentation/formatting slices.
Shadow Universe remains non-production. Option/Result/Unit identity work is
separate and must follow the architect-guided spec-backed plan; this source
slice has no ADT, module ownership, or identity dependency.

## 7. Architecture and ownership

The change stays inside the existing `Gregorian.isLeapYear` source method in
`packages/universe/src/time/calendar/month.ph`. There is no provider,
bootstrap, native registration, compiler, VM, semantic-authority, or runtime
representation change.

## 8. Ownership boundaries

### Owns

- One expression body inside `Gregorian.isLeapYear`.
- This plan's walkthrough, handoff, checkpoint row, and validation-ledger entry.

### Does not own

- Production Universe or runtime calendar implementations.
- Public API, module path, type identity, or method signature changes.
- Tests or changes to the shadow loader/compiler architecture.
- Concurrent MODL001 P4 files.

## 9. Global invariants

- **INV-01:** Preserve the exact Gregorian predicate: divisible by 4 and either
  not divisible by 100 or divisible by 400.
- **INV-02:** Preserve results for year zero, negative years, and positive
  years. Divisibility is characterized by zero remainder, independent of the
  sign convention for nonzero remainders.
- **INV-03:** Preserve the method signature, visibility, owner, and surrounding
  calendar source.
- **INV-04:** Keep `packages/universe/src/**` non-production and do not change
  canonical Universe identity or source ownership.
- **INV-05:** Report parser/interface, semantic, and executable evidence as
  separate states. Parser acceptance is not semantic/runtime proof.

## 10. Non-goals

- No broad calendar cleanup or formatting.
- No API or specification change.
- No new tests against production sources presented as shadow behavior proof.
- No attempt to enable a shadow runtime/provider.
- No edits to Option, Result, Unit, or concurrent MODL001 P4 scope.

## 11. Expected impact map

- `packages/universe/src/time/calendar/month.ph` — one expression rewrite.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/` — lifecycle records.
- `packages/universe/docs/modernization/VALIDATION-LEDGER.md` and `STATUS.md` — evidence and next action.

Unexpected touches to production, compiler, semantic, runtime, or identity
owners require stopping and escalating.

## 12. Implementer decision authority

### FIXED

- The predicate in INV-01 and the public/module boundaries in INV-03/04.
- The replacement expression is
  `(year % 4) == 0 and ((year % 100) != 0 or (year % 400) == 0)`.
- Do not claim semantic or runtime verification from source inspection or the
  parser/interface test.

### MECHANICALLY FLEXIBLE

- Local whitespace required by the existing formatter.
- Exact wording in lifecycle evidence, provided evidence limits stay explicit.

### VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| `%` is exact for `Int` and zero remainder characterizes divisibility | `docs/specs/numeric-types.md` | Stop and report conflict |
| The shadow parser accepts the expression | Exact shadow source validator | Repair only mechanical syntax; otherwise stop |
| `and`/`or` preserve the intended Boolean predicate | Live language contract/implementation plus source-level proof | Stop if semantics conflict |

## 13. STOP / CONSULT triggers

Stop if the change needs a different identity, type, public contract, runtime
owner, special case, or architectural workaround; if a live spec conflicts;
or if parser/semantic evidence contradicts the proposed expression. Do not
weaken INV-01 to make validation pass.

## 14. Debugging budget

No behavioral failure is expected. Allow at most three mechanical syntax or
formatting correction cycles, each tied to a concrete parser/formatter result.
Any nontrivial semantic contradiction stops this slice.

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant | Evidence |
|---|---|---|---|
| CV-01 | Shadow syntax and unlinked interface | INV-03/04/05 | Exact existing `universe_shadow_sources` test |
| CV-02 | Boolean `and`/`or` short-circuit behavior | INV-01 | Existing corpus command attempted; current run fails during canonical import resolution before fixture execution, so it supplies no runtime evidence |
| CV-03 | Arithmetic equivalence, including zero and negative integers | INV-01/02 | Source-level proof against exact `Int` remainder semantics; not executable shadow evidence |
| CV-04 | Shadow behavior/runtime | INV-01/02/05 | Blocked: no complete shadow semantic execution path |

## 16. Verification execution budget

During BUILD, run only:

```sh
RUSTFLAGS='' cargo test -p phalcom-modules --test universe_shadow_sources universe_shadow_sources_parse_and_have_valid_unlinked_interfaces -- --exact
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus booleans
git diff --check
```

The shadow validator must select one test and pass. The Boolean corpus filter
must select one test; if the current shared worktree fails before fixture
execution, classify the failure relative to P5, preserve the concurrent owner,
and do not rerun without a relevant change. That corpus exercises production
sources and cannot certify `Gregorian.isLeapYear` in the shadow package. Do not
run the production `time` corpus, workspace tests, or unrelated suites. Record
semantic and runtime limitations explicitly.

## 17. Baseline / unrelated failure policy

Classify any failure by causal relationship. A failure in the active shadow
source test is active work. Unrelated `phalcom-semantic` edits/failures belong
to concurrent MODL001 P4 work and must remain untouched.

## T1 — Rewrite the leap-year predicate

Replace only the three-line early-return body in `Gregorian.isLeapYear` with
the fixed expression in §12. Preserve all other source.

## G1 — Focused source validation

Run the exact test and whitespace check in §16. A passing parser/interface
test means syntax/interface acceptance only; it does not prove body typing,
canonical identity, semantics, or execution.

## G2 — Equivalence and evidence review

Review INV-01/02 against the old branch sequence and exact `Int` remainder
contract. Mark runtime parity blocked and semantic linking unverified because
the shadow package cannot be executed/linked on this path.

## 20. Final focused acceptance

- Only the planned expression changes in the source file.
- The exact shadow source validator passes with one selected test.
- `git diff --check` passes.
- The source-level equivalence reasoning and its limitations are recorded.
- The runtime gate remains `BLOCKED`; no production test is substituted.
- Any unrelated test failure is classified and carried forward without
  changing its owner or weakening P5's acceptance criteria.

## 21. Performance/resource evidence

No performance claim is made. The expression has no allocation or state
change. Its conditional arithmetic follows the same divisibility sequence
when Boolean operators short-circuit; the exact runtime behavior remains
unmeasured in the shadow package.

## 22. Checkpoint bookkeeping

Update `CHECKPOINT.md`, `packages/universe/docs/modernization/VALIDATION-LEDGER.md`,
and `STATUS.md` with actual evidence. Mark this plan complete only when its
focused source validation and source-level review are recorded. Do not promote
semantic or runtime status.

## 23. Walkthrough deliverable

Create `UNIV002.C2.P5-walkthrough.md` recording the actual diff, source-level
proof, exact checks and results, and unavailable gates.

## 24. Handoff deliverable

Create `UNIV002.C2.P5-handoff.md` with the live worktree/branch/revision,
committed/pushed status when known, established boundaries, verification
limits, and the next bounded C2 slice. Preserve the Option/Result/Unit identity
decision and concurrent MODL001 P4 work as inherited context.

## 25. Completion truth table

| Condition | Plan status |
|---|---|
| Planned rewrite and focused parser/interface check complete; source-level proof recorded | COMPLETE / IMPLEMENTED / FOCUSED_TESTED |
| Exact focused test cannot pass or source-level equivalence is unresolved | IN_PROGRESS or BLOCKED, with cause recorded |
| Shadow semantic/runtime path unavailable | Does not invalidate source completion; semantic remains UNVERIFIED and runtime remains BLOCKED |

## 26. Plan self-review

- Scope is one source method and its evidence.
- The formula is grounded in the existing algorithm and exact integer remainder
  semantics; no architecture or identity choice changes.
- The plan distinguishes parser evidence from semantic and executable proof.
- Concurrent P4 work is excluded.
