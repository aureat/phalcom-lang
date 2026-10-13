---
id: UNIV002.C2.P22
category: UNIV
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on:
  - UNIV002.C2.P21
follows: UNIV002.C2.P21
supersedes: null
---

# UNIV002.C2.P22 — Optional replacement and Prism modification documentation

## 0. Executor contract

Make three Category A documentation lines above `Optional.set`,
`Optional.modify`, and `Prism.modify` in the stable Functional shadow module.
Preserve all declarations, signatures, executable expressions, module
ownership, generic identities, and runtime contracts. Describe only the
present/absent and selected/unselected branches visible in the method bodies.
Source review establishes scope and whitespace hygiene, not linked semantics
or runtime behavior.

## 1. Goal

Document when Optional and Prism methods pass values to their callbacks and
when they return the original source. Do not imply that caller-supplied
callbacks satisfy replacement, review, or round-trip laws.

## 2. Checkpoint acceptance objective

This is one of several Category A plans advancing `UNIV002.C2`; it does not
close the checkpoint.

## 3. Repository grounding

Prepared against:

```text
repository: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections
branch: feat/telos-collections
revision: cb385f1ecccacc38e4746e3b47d4ce5a943387e3
predecessor: UNIV002.C2.P21
```

`packages/universe/src/functional/optics.ph` has SHA-256
`e0f95221731b0a0827c109afbba1510f20ce252013ee8d825e757582923e26d8` at plan
start. `Optional.set` reads the optional focus and passes the source and
supplied value to its replacement callback for `Some`, returning the original
source for `None`. `Optional.modify` reads the focus, applies `f`, and passes
the result to the replacement callback for `Some`; it returns the original
source for `None`. `Prism.modify` previews the source, applies `f`, and passes
the result to the review callback for `Some`; it returns the original source
for `None`. Re-read the live methods before editing. Adapt mechanical drift
locally; treat contract or ownership drift as a stop condition.

## 4. Required reads before implementation

- Repository `AGENTS.md` and `docs/workflow/implementation-record-lifecycle-convention.md`.
- This plan, `CHECKPOINT.md`, and the P21 walkthrough and handoff.
- `packages/universe/docs/modernization/GUIDELINES.md` and `INVENTORY.md`.
- `packages/universe/src/functional/optics.ph`.
- Live `git status --short` and the scoped source diff.

## 5. Normative authority

No additional normative rule is needed to document these direct method
bodies. Do not claim callback laws, or properties of the callbacks supplied
to `Optional` or `Prism`.

## 6. Takeover state

P1–P21 are complete as recorded in the checkpoint. The source inventory is
reconciled through P21 source commit `cb385f1ecccacc38e4746e3b47d4ce5a943387e3`:
174 `.ph` paths, no additions or removals, and eighteen changed hashes against
MOD1. The shadow package remains non-production; source review does not
establish linked semantic identity or runtime behavior.

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| Optional replacement | `Optional.set`, `functional/optics.ph` | Shadow Functional source | Call the replacement callback for a present focus; return the input source for an absent focus. |
| Optional modification | `Optional.modify`, same module | Shadow Functional source | Transform a present focus and pass it to the replacement callback; return the input source when absent. |
| Prism modification | `Prism.modify`, same module | Shadow Functional source | Transform a selected payload and pass it to the review callback; return the input source when unselected. |

## 7. Architecture

Place one comment line immediately above each method. Suggested wording:

```telos
/// Applies the replacement callback when a focus is present; otherwise preserves the source.
/// Transforms and passes a present focus to the replacement callback; otherwise preserves the source.
/// Transforms and passes a selected payload to the review callback; otherwise preserves the source.
```

Equivalent wording is acceptable only if it describes these callback paths
without promising callback effects. Do not edit the match expressions,
callbacks, declarations, signatures, or adjacent methods.

## 8. Ownership boundaries

### Owns

- One documentation line above each of the three methods.
- P22 checkpoint/validation evidence and inventory hash refresh.

### Does not own

- Optional or Prism behavior, API, generic identity, representation, callback
  laws, or module ownership.
- Product/reflection semantics, other optics, identity or linker work,
  production Universe, compiler, semantic, or runtime changes.
- Concurrent MODL001 P4 files, fixtures, plans, or lifecycle records.

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| Present/absent replacement | `Optional.set` body | Its documentation | A comment claiming callback effects or invocation for an absent focus. |
| Present/absent modification | `Optional.modify` body | Its documentation | A second update algorithm or callback-effect/count guarantee. |
| Selected/unselected modification | `Prism.modify` body | Its documentation | A comment claiming review-callback effects or invocation for an unselected source. |

## 9. Global invariants

- **INV-01:** `Optional.set` documentation states that the replacement
  callback is used for a present focus and the source is returned when absent.
- **INV-02:** `Optional.modify` documentation states that `f` is applied to a
  present focus and the result is passed to the replacement callback; absent
  focus returns the source.
- **INV-03:** `Prism.modify` documentation states that `f` is applied to a
  selected payload and the result is passed to the review callback; an
  unselected source is returned unchanged.
- **INV-04:** No declaration, signature, executable line, import, module
  owner, or generic identity changes.
- **INV-05:** No callback-law, count, purity, exception, or timing promise is
  added.
- **INV-06:** Shadow Universe remains non-production; no semantic/runtime claim
  is made from comments or source hashes.
- **INV-07:** Concurrent MODL001 P4 changes remain untouched and unstaged.

## 10. Non-goals

- No executable source modernization, API/type/identity change, or new test.
- No Product/reflection behavior, production cutover, provider/bootstrap/native,
  compiler, semantic, or runtime change.
- No Option/Result/Unit identity or data migration.
- No inventory heuristic rescan or broad test run.

## 11. Expected impact map

- `packages/universe/src/functional/optics.ph` — exactly three comment lines.
- `packages/universe/docs/modernization/INVENTORY.md` — refresh the existing
  optics hash and source revision.
- `packages/universe/docs/modernization/VALIDATION-LEDGER.md` — P22 evidence.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/`
  — checkpoint, plan metadata, walkthrough, and handoff.

No tests are expected or required. Unexpected executable source changes
require stopping and re-scoping rather than expanding this plan.

## 12. Implementer decision authority

### FIXED

- Preserve callback and branch paths in INV-01 through INV-03 without
  implying properties of the supplied callbacks.
- Preserve declaration, body, ownership, identity, and runtime behavior.
- Do not promise callback properties or callback laws.

### MECHANICALLY FLEXIBLE

- Exact wording and line wrapping, provided INV-01 through INV-07 hold.

### VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| The three methods still match the described branches | Live method bodies | Stop and revise if contract drift is material. |
| No concurrent edit owns the target path | `git status --short` and scoped diff | Coordinate before writing. |

## 13. STOP / CONSULT triggers

Stop if accurate documentation would require changing either API, identity,
representation, callback semantics, Product/reflection constraints, or a
second source of truth. Follow the repository escalation protocol for any
architectural conflict.

## 14. Debugging budget

Comments-only scope: review the exact diff and run scoped `git diff --check`.
Do not run Cargo tests. If executable code changes, stop and re-scope.

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant | Evidence |
|---|---|---|---|
| CV-01 | Source-only scope | INV-01/02/03/04/05 | Exact diff review confirms three comments and unchanged methods. |
| CV-02 | Whitespace hygiene | INV-04 | Scoped `git diff --check`. |
| CV-03 | Shadow semantic/runtime behavior | INV-06 | Not applicable to comments; no claim is made. |

## 16. Verification execution budget

Do not run tests for this comments-only change. Recompute the changed source
hash and refresh the inventory overlay. Independently compare the complete
live `.ph` path set and hashes with MOD1 before closing P22.

## 17. Baseline / unrelated failure policy

The prior P5 production-Universe Boolean corpus attempt failed before fixture
execution during canonical import resolution and remains assigned to active
MODL001 P4 context. Do not rerun or diagnose it here. Concurrent P4 changes
remain outside P22 ownership.

## T1 — Document Optional replacement and focus modification

Add one comment above `Optional.set` and one above `Optional.modify`. Do not
touch executable source.

## T2 — Document Prism focus modification

Add one comment above `Prism.modify`. Do not touch executable source.

## G1 — Scope and whitespace review

Confirm only the three planned comment lines changed and run scoped
`git diff --check`.

## G2 — Inventory reconciliation

Refresh the `functional/optics.ph` hash and source-revision overlay for the 174
existing paths. Preserve unchanged baseline hashes and independently compare
the path set and hashes. Do not reinterpret textual inventory heuristics as a
new audit.

## 18. Final focused acceptance

| Coverage ID | Acceptance evidence |
|---|---|
| CV-01 | Exact diff contains only the three planned comment lines. |
| CV-02 | Scoped `git diff --check` passes. |
| CV-03 | No semantic or runtime claim is made. |

## 19. Checkpoint bookkeeping and completion artifacts

At completion, update plan status, inventory/validation ledger, and checkpoint
state. Create P22 walkthrough and handoff with exact source revision, change,
verification, deferred limitations, and the next safe bounded action.
