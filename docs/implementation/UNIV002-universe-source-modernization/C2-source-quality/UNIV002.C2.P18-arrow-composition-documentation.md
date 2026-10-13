---
id: UNIV002.C2.P18
category: UNIV
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - UNIV002.C2.P17
follows: UNIV002.C2.P17
supersedes: null
---

# UNIV002.C2.P18 — Arrow composition documentation

## 0. Executor contract

Make two Category A documentation-only changes in the stable Functional
shadow module. Preserve all declarations, signatures, executable expressions,
module ownership, generic identities, and runtime contracts. The comments may
describe only the application order visible in the current method bodies.
Source review establishes scope and whitespace hygiene; it does not establish
linked semantics or runtime behavior.

## 1. Goal

Document that `Arrow.then` applies this arrow before the next arrow and
`Arrow.compose` applies the previous arrow before this arrow.

## 2. Checkpoint acceptance objective

This bounded source-quality slice advances `UNIV002.C2`, which applies
MOD2–MOD4 source-quality standards to stable shadow modules. It is one of
several Category A plans and does not close the checkpoint.

## 3. Repository grounding

Prepared against:

```text
repository: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections
branch: feat/telos-collections
revision: 9d03621c86ae1abab4a7f5db170953e1ead3e92d
predecessor: UNIV002.C2.P17
```

`packages/universe/src/functional/arrow.ph` has SHA-256
`48bfe2b7a807658d86d88de3dac7fb2007f447c627287493da2261d3dfa8b427` at plan
start. Neither `then` nor `compose` currently has a documentation comment.
`then` constructs a pipeline that calls `self` before `next`; `compose`
delegates to `previous.then(self)`. Re-read the live paths before editing.
Adapt mechanical drift locally; treat contract or ownership drift as a stop
condition.

## 4. Required reads before implementation

- Repository `AGENTS.md` and `docs/workflow/implementation-record-lifecycle-convention.md`.
- This plan, `CHECKPOINT.md`, and the P17 walkthrough and handoff.
- `packages/universe/docs/modernization/GUIDELINES.md` and `INVENTORY.md`.
- `packages/universe/src/functional/arrow.ph`.
- Live `git status --short` and the scoped source diff.

## 5. Normative authority

No Arrow-specific normative rule is needed to document these direct method
bodies. Describe only the observed order. Do not assert mathematical laws,
callback count, purity, exception behavior, eager/lazy timing, or other
guarantees beyond the source.

## 6. Takeover state

P1–P17 are complete as recorded in the checkpoint. The source inventory is
reconciled through P17 source commit `9d03621c86ae1abab4a7f5db170953e1ead3e92d`:
174 `.ph` paths, no additions or removals, and sixteen changed hashes against
MOD1. The shadow package remains non-production; source review does not
establish linked semantic identity or runtime behavior.

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| Function composition | `Arrow.then`, `functional/arrow.ph` | Shadow Functional source | `self` is called before `next` in the constructed pipeline. |
| Reverse composition syntax | `Arrow.compose`, `functional/arrow.ph` | Shadow Functional source | It delegates to `previous.then(self)`. |

## 7. Architecture

Comments belong immediately above the two methods in
`packages/universe/src/functional/arrow.ph`. Suggested wording:

```telos
/// Applies this arrow before `next`.
/// Applies `previous` before this arrow.
```

Equivalent wording is acceptable only if it preserves those exact roles and
order. The comments do not change pipeline construction or callback behavior.

## 8. Ownership boundaries

### Owns

- One source documentation comment for `Arrow.then` and one for
  `Arrow.compose`.
- P18 checkpoint/validation evidence and the existing inventory hash refresh.

### Does not own

- Arrow behavior, API, generic identity, representation, or module ownership.
- Other Functional declarations; iteration, identity, reflection, or linker
  work; production Universe; compiler, semantic, or runtime changes.
- Concurrent MODL001 P4 files, fixtures, plans, or lifecycle records.

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| `then` application order | `Arrow.then` body | Its documentation | A comment describing a different order or inferred law. |
| `compose` application order | `Arrow.compose` body and delegation | Its documentation | A second implementation or identity rule. |

## 9. Global invariants

- **INV-01:** `then` documentation states `self` before `next`.
- **INV-02:** `compose` documentation states `previous` before `self`.
- **INV-03:** No declaration, signature, executable line, import, module owner,
  or generic identity changes.
- **INV-04:** No callback-count, purity, exception, timing, or algebraic-law
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
- `packages/universe/docs/modernization/VALIDATION-LEDGER.md` — P18 evidence.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/`
  — checkpoint, plan metadata, walkthrough, and handoff.

No tests are expected or required. Unexpected executable source changes require
stopping and re-scoping rather than expanding this plan.

## 12. Implementer decision authority

### FIXED

- Preserve the exact application order in INV-01 and INV-02.
- Preserve declarations, bodies, ownership, identity, and all runtime behavior.
- Do not promise callback properties or unimplemented algebraic laws.

### MECHANICALLY FLEXIBLE

- Exact comment wording and wrapping, provided INV-01 through INV-06 hold.

### VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| Both methods still have the planned composition order | Live method bodies | Stop and revise the plan if contract drift is material. |
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
live `.ph` path set and hashes with MOD1 before closing P18.

## 17. Baseline / unrelated failure policy

The prior P5 production-Universe Boolean corpus attempt failed before fixture
execution during canonical import resolution and remains assigned to active
MODL001 P4 context. Do not rerun or diagnose it here. Concurrent P4 changes
remain outside P18 ownership.

## T1 — Document composition order

Add one comment immediately above `then` and one immediately above `compose`.
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
state. Create P18 walkthrough and handoff with exact source revision, change,
verification, deferred limitations, and the next safe bounded action.
