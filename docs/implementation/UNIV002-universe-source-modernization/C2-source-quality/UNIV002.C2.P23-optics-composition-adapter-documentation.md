---
id: UNIV002.C2.P23
category: UNIV
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on:
  - UNIV002.C2.P22
follows: UNIV002.C2.P22
supersedes: null
---

# UNIV002.C2.P23 — Optional/Prism composition and optics adapter documentation

## 0. Executor contract

Make six Category A documentation lines in the stable Functional shadow
module: one above `Iso.asLens`, two above `Optional.compose`, two above
`Prism.compose`, and one above `Prism.asOptional`. Preserve all declarations,
signatures, executable expressions, module ownership, generic identities, and
runtime contracts.
Describe only callback selection and order visible in the method bodies.
Source review establishes scope and whitespace hygiene, not linked semantics
or runtime behavior.

## 1. Goal

Document the callback paths used by the Iso and Prism adapters and the read,
update, preview, and review order used by Optional and Prism composition.

## 2. Checkpoint acceptance objective

This is one of several Category A plans advancing `UNIV002.C2`; it does not
close the checkpoint.

## 3. Repository grounding

Prepared against:

```text
repository: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections
branch: feat/telos-collections
revision: 60b2e1463db0d4fd29981f0ec178b46d426edf7a
predecessor: UNIV002.C2.P22
```

`packages/universe/src/functional/optics.ph` has SHA-256
`acdf2870f9f759d7804124af688e8c3e4cbe7f9e6702816d166d6b245527fb87` at plan
start. `Iso.asLens` supplies `self.forward` as the Lens getter and calls
`self.restore` in its setter callback. `Optional.compose` reads through
`self.get` and then `other.get` for a present outer focus; its update callback
uses `other.set` inside `self.modify`. `Prism.compose` previews with `self`
then `other`, and reviews with `other` then `self`. `Prism.asOptional` uses
`self.select` for reads and calls `self.review` from its replacement callback.
Re-read the live methods before editing. Adapt mechanical drift locally; treat
contract or ownership drift as a stop condition.

## 4. Required reads before implementation

- Repository `AGENTS.md` and `docs/workflow/implementation-record-lifecycle-convention.md`.
- This plan, `CHECKPOINT.md`, and the P22 walkthrough and handoff.
- `packages/universe/docs/modernization/GUIDELINES.md` and `INVENTORY.md`.
- `packages/universe/src/functional/optics.ph`.
- Live `git status --short` and the scoped source diff.

## 5. Normative authority

No additional normative rule is needed to document these direct method
bodies. Do not claim callback laws, callback effects, or round-trip behavior.

## 6. Takeover state

P1–P22 are complete as recorded in the checkpoint. The source inventory is
reconciled through P22 source commit `60b2e1463db0d4fd29981f0ec178b46d426edf7a`:
174 `.ph` paths, no additions or removals, and eighteen changed hashes against
MOD1. The shadow package remains non-production; source review does not
establish linked semantic identity or runtime behavior.

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| Iso adapter | `Iso.asLens`, `functional/optics.ph` | Shadow Functional source | Lens getter uses `forward`; setter callback uses `restore`. |
| Optional composition | `Optional.compose`, same module | Shadow Functional source | Reads `self` then `other`; for a present outer focus, update callback routes through `other.set`. |
| Prism composition | `Prism.compose`, same module | Shadow Functional source | Preview order is `self` then `other`; review order is `other` then `self`. |
| Prism adapter | `Prism.asOptional`, same module | Shadow Functional source | Optional reads use `select`; replacement callback calls `review`. |

## 7. Architecture

Place documentation immediately above the named methods. Suggested wording:

```telos
/// Uses the forward callback for Lens reads and the backward callback for writes.
/// Reads through this optional, then `other` for a present focus.
/// Routes updates through `other.set` inside a present outer focus.
/// Previews through this prism, then `other`.
/// Reviews by applying `other`, then this prism.
/// Uses this prism's selector for optional reads and its review callback for replacement.
```

The four methods require six lines: one above `Iso.asLens`, two above
`Optional.compose`, two above `Prism.compose`, and one above
`Prism.asOptional`. Equivalent wording is acceptable only if it preserves the
described callback paths and order without promising callback effects. Do not
edit executable expressions, signatures, imports, or adjacent methods.

## 8. Ownership boundaries

### Owns

- The six documentation lines above the four named methods.
- P23 checkpoint/validation evidence and inventory hash refresh.

### Does not own

- Iso, Optional, or Prism behavior, API, generic identity, representation,
  callback laws, or module ownership.
- Product/reflection semantics, other optics, identity or linker work,
  production Universe, compiler, semantic, or runtime changes.
- Concurrent MODL001 P4 files, fixtures, plans, or lifecycle records.

## 9. Global invariants

- **INV-01:** `Iso.asLens` documentation states only the existing forward and
  backward callback use.
- **INV-02:** `Optional.compose` documentation states the nested read order
  and routes updates through `other.set` only inside a present outer focus.
- **INV-03:** `Prism.compose` documentation states preview order `self` then
  `other`, and review order `other` then `self`.
- **INV-04:** `Prism.asOptional` documentation states only the selector and
  review callbacks used by the adapter.
- **INV-05:** No declaration, signature, executable line, import, module
  owner, or generic identity changes.
- **INV-06:** No callback-law, effect, count, purity, exception, or timing
  promise is added.
- **INV-07:** Shadow Universe remains non-production; no semantic/runtime
  claim is made from comments or source hashes.
- **INV-08:** Concurrent MODL001 P4 changes remain untouched and unstaged.

## 10. Non-goals

- No executable source modernization, API/type/identity change, or new test.
- No Product/reflection behavior, production cutover, provider/bootstrap/native,
  compiler, semantic, or runtime change.
- No Option/Result/Unit identity or data migration.
- No inventory heuristic rescan or broad test run.

## 11. Expected impact map

- `packages/universe/src/functional/optics.ph` — exactly six comment lines.
- `packages/universe/docs/modernization/INVENTORY.md` — refresh the existing
  optics hash and source revision.
- `packages/universe/docs/modernization/VALIDATION-LEDGER.md` — P23 evidence.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/`
  — checkpoint, plan metadata, walkthrough, and handoff.

No tests are expected or required. Unexpected executable source changes
require stopping and re-scoping rather than expanding this plan.

## 12. Implementer decision authority

### FIXED

- Preserve callback paths and order in INV-01 through INV-04.
- Preserve declarations, bodies, ownership, identity, and runtime behavior.
- Do not promise callback effects or callback laws.

### MECHANICALLY FLEXIBLE

- Exact wording and line wrapping, provided INV-01 through INV-08 hold.

### VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| The four methods still match the described callback paths | Live method bodies | Stop and revise if contract drift is material. |
| No concurrent edit owns the target path | `git status --short` and scoped diff | Coordinate before writing. |

## 13. STOP / CONSULT triggers

Stop if accurate documentation would require changing an API, identity,
representation, callback semantics, Product/reflection constraints, or a
second source of truth. Follow the repository escalation protocol for any
architectural conflict.

## 14. Debugging budget

Comments-only scope: review the exact diff and run scoped `git diff --check`.
Do not run Cargo tests. If executable code changes, stop and re-scope.

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant | Evidence |
|---|---|---|---|
| CV-01 | Source-only scope | INV-01 through INV-06 | Exact diff review confirms only six comments and unchanged methods. |
| CV-02 | Whitespace hygiene | INV-05 | Scoped `git diff --check`. |
| CV-03 | Shadow semantic/runtime behavior | INV-07 | Not applicable to comments; no claim is made. |

## 16. Verification execution budget

Do not run tests for this comments-only change. Recompute the changed source
hash and refresh the inventory overlay. Independently compare the complete
live `.ph` path set and hashes with MOD1 before closing P23.

## 17. Baseline / unrelated failure policy

The prior P5 production-Universe Boolean corpus attempt failed before fixture
execution during canonical import resolution and remains assigned to active
MODL001 P4 context. Do not rerun or diagnose it here. Concurrent P4 changes
remain outside P23 ownership.

## T1 — Document adapters and compositions

Add the planned comments above `Iso.asLens`, `Optional.compose`,
`Prism.compose`, and `Prism.asOptional`. Do not touch executable source.

## G1 — Scope and whitespace review

Confirm only the six planned comment lines changed and run scoped
`git diff --check`.

## G2 — Inventory reconciliation

Refresh the `functional/optics.ph` hash and source-revision overlay for the 174
existing paths. Preserve unchanged baseline hashes and independently compare
the path set and hashes. Do not reinterpret textual inventory heuristics as a
new audit.

## 18. Final focused acceptance

| Coverage ID | Acceptance evidence |
|---|---|
| CV-01 | Exact diff contains only the six planned comment lines. |
| CV-02 | Scoped `git diff --check` passes. |
| CV-03 | No semantic or runtime claim is made. |

## 19. Checkpoint bookkeeping and completion artifacts

At completion, update plan status, inventory/validation ledger, and checkpoint
state. Create P23 walkthrough and handoff with exact source revision, change,
verification, deferred limitations, and the next safe bounded action.
