---
id: UNIV002.C2.P19
category: UNIV
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - UNIV002.C2.P18
follows: UNIV002.C2.P18
supersedes: null
---

# UNIV002.C2.P19 — Arrow mapping documentation

## 0. Executor contract

Make two Category A documentation-only changes in the stable Functional
shadow module. Preserve all declarations, signatures, executable expressions,
module ownership, generic identities, and runtime contracts. Comments may
describe only the input/output mapping visible in the current method bodies.
Source review establishes scope and whitespace hygiene; it does not establish
linked semantics or runtime behavior.

## 1. Goal

Document the input/output transforms of `Arrow.dimap` and the component
transformed by `Arrow.second`.

## 2. Checkpoint acceptance objective

This is one of several Category A plans advancing `UNIV002.C2`; it does not
close the checkpoint.

## 3. Repository grounding

Prepared against:

```text
repository: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections
branch: feat/telos-collections
revision: 2423d7dbdd845d853dc5696c77e4fa2971924550
predecessor: UNIV002.C2.P18
```

`packages/universe/src/functional/arrow.ph` has SHA-256
`18f48d8445d28bc4ced75f63c37fc6658de93a02170c07548c061340c5cdd4b7` at plan
start. `dimap` applies `before` to its input, calls this arrow, then applies
`after` to its output. `second` applies this arrow to `pair[1]` and passes
`pair[0]` through as the first output component. Neither method has a
documentation comment. Re-read both live bodies before editing. Adapt
mechanical drift locally; treat contract or ownership drift as a stop
condition.

## 4. Required reads before implementation

- Repository `AGENTS.md` and `docs/workflow/implementation-record-lifecycle-convention.md`.
- This plan, `CHECKPOINT.md`, and the P18 walkthrough and handoff.
- `packages/universe/docs/modernization/GUIDELINES.md` and `INVENTORY.md`.
- `packages/universe/src/functional/arrow.ph`.
- Live `git status --short` and the scoped source diff.

## 5. Normative authority

No Arrow-specific normative rule is needed to document these direct method
bodies. Describe only the observed mappings. Do not assert profunctor/arrow
laws, callback count, purity, exception behavior, or eager/lazy timing.

## 6. Takeover state

P1–P18 are complete as recorded in the checkpoint. The source inventory is
reconciled through P18 source commit `2423d7dbdd845d853dc5696c77e4fa2971924550`:
174 `.ph` paths, no additions or removals, and seventeen changed hashes
against MOD1. The shadow package remains non-production; source review does
not establish linked semantic identity or runtime behavior.

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| Input/output adaptation | `Arrow.dimap`, `functional/arrow.ph` | Shadow Functional source | `before` runs before this arrow; `after` runs on its output. |
| Second-component mapping | `Arrow.second`, `functional/arrow.ph` | Shadow Functional source | This arrow transforms the second component while the first is passed through. |

## 7. Architecture

Comments belong immediately above the two methods in
`packages/universe/src/functional/arrow.ph`. Suggested wording:

```telos
/// Applies `before`, then this arrow, then `after`.
/// Applies this arrow to the second tuple component and preserves the first.
```

Equivalent wording is acceptable only if it preserves those direct mappings.
No code or method contract changes.

## 8. Ownership boundaries

### Owns

- One documentation comment for `Arrow.dimap` and one for `Arrow.second`.
- P19 checkpoint/validation evidence and the existing inventory hash refresh.

### Does not own

- Arrow behavior, API, generic identity, representation, or module ownership.
- Other Functional declarations; iteration, identity, reflection, or linker
  work; production Universe; compiler, semantic, or runtime changes.
- Concurrent MODL001 P4 files, fixtures, plans, or lifecycle records.

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| `dimap` callback order | `Arrow.dimap` body | Its documentation | A comment promising callback laws or different ordering. |
| `second` component mapping | `Arrow.second` body | Its documentation | A second implementation or altered tuple representation. |

## 9. Global invariants

- **INV-01:** `dimap` documentation states `before`, this arrow, then `after`.
- **INV-02:** `second` documentation states that the second component is
  transformed and the first is preserved.
- **INV-03:** No declaration, signature, executable line, import, module owner,
  or generic identity changes.
- **INV-04:** No algebraic-law, callback-count, purity, exception, or timing
  promise is added.
- **INV-05:** Shadow Universe remains non-production; no semantic/runtime claim
  is made from comments or source hashes.
- **INV-06:** Concurrent MODL001 P4 changes remain untouched and unstaged.

## 10. Non-goals

- No executable source modernization, API/type/identity change, or new test.
- No production cutover or provider/bootstrap/native/runtime change.
- No iteration, reflection, Option/Result/Unit identity, or data migration.
- No inventory heuristic rescan or broad test run.

## 11. Expected impact map

- `packages/universe/src/functional/arrow.ph` — exactly two method comments.
- `packages/universe/docs/modernization/INVENTORY.md` — refresh existing
  `functional/arrow.ph` hash and source revision.
- `packages/universe/docs/modernization/VALIDATION-LEDGER.md` — P19 evidence.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/`
  — checkpoint, plan metadata, walkthrough, and handoff.

No tests are expected or required. Unexpected executable source changes require
stopping and re-scoping rather than expanding this plan.

## 12. Implementer decision authority

### FIXED

- Preserve the exact input/output mappings in INV-01 and INV-02.
- Preserve declarations, bodies, ownership, identity, and runtime behavior.
- Do not promise callback properties or unimplemented algebraic laws.

### MECHANICALLY FLEXIBLE

- Exact comment wording and wrapping, provided INV-01 through INV-06 hold.

### VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| Both methods still match the planned mapping | Live method bodies | Stop and revise the plan if contract drift is material. |
| No concurrent edit owns the target path | `git status --short` and scoped diff | Coordinate before writing. |

## 13. STOP / CONSULT triggers

Stop if accurate documentation would require changing Arrow's API,
representation, identity, execution order, or a second source of truth. Follow
the repository escalation protocol for any architectural conflict.

## 14. Debugging budget

Comments-only scope: review the exact diff and run scoped `git diff --check`.
Do not run Cargo tests. If executable code changes, stop and re-scope.

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant | Evidence |
|---|---|---|---|
| CV-01 | Source-only scope | INV-01/02/03/04 | Exact diff review confirms two comments and unchanged methods. |
| CV-02 | Whitespace hygiene | INV-03 | Scoped `git diff --check`. |
| CV-03 | Shadow semantic/runtime behavior | INV-05 | Not applicable to comments; no claim is made. |

## 16. Verification execution budget

Do not run tests for this comments-only change. Recompute the changed source
hash and refresh the inventory overlay. Independently compare the complete
live `.ph` path set and hashes with MOD1 before closing P19.

## 17. Baseline / unrelated failure policy

The prior P5 production-Universe Boolean corpus attempt failed before fixture
execution during canonical import resolution and remains assigned to active
MODL001 P4 context. Do not rerun or diagnose it here. Concurrent P4 changes
remain outside P19 ownership.

## T1 — Document Arrow mappings

Add one comment immediately above `dimap` and one immediately above `second`.
Do not touch executable source.

## G1 — Scope and whitespace review

Confirm only the two planned comments changed and run scoped
`git diff --check`.

## G2 — Inventory reconciliation

Refresh the `functional/arrow.ph` hash and source-revision overlay for the 174
existing paths. Preserve unchanged baseline hashes and independently compare
the path set and hashes. Do not reinterpret textual inventory heuristics as a
new audit.

## 18. Final focused acceptance

| Coverage ID | Acceptance evidence |
|---|---|
| CV-01 | Exact diff contains only the two planned comments. |
| CV-02 | Scoped `git diff --check` passes. |
| CV-03 | No semantic or runtime claim is made. |

## 19. Checkpoint bookkeeping and completion artifacts

At plan start, maintain the checkpoint ledger and active-plan pointer. At
completion, update plan status, inventory/validation ledger, and checkpoint
state. Create P19 walkthrough and handoff with exact source revision, change,
verification, deferred limitations, and the next safe bounded action.
