---
id: UNIV002.C2.P21
category: UNIV
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - UNIV002.C2.P20
follows: UNIV002.C2.P20
supersedes: null
---

# UNIV002.C2.P21 — Iso composition and modification documentation

## 0. Executor contract

Make three Category A documentation lines above `Iso.compose` and
`Iso.modify` in the stable Functional shadow module. Preserve all declarations,
signatures, executable expressions, module ownership, generic identities, and
runtime contracts. Describe only the callback and focus flow visible in the
method bodies. Source review establishes scope and whitespace hygiene, not
linked semantics or runtime behavior.

## 1. Goal

Document the forward/reverse order used by `Iso.compose` and the
convert-transform-restore sequence used by `Iso.modify`.

## 2. Checkpoint acceptance objective

This is one of several Category A plans advancing `UNIV002.C2`; it does not
close the checkpoint.

## 3. Repository grounding

Prepared against:

```text
repository: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections
branch: feat/telos-collections
revision: a66c51f84f50dc2476a36836cdd057e7849cb492
predecessor: UNIV002.C2.P20
```

`packages/universe/src/functional/optics.ph` has SHA-256
`57f4a4b83a914259e7d4f77beb32fa25792b5f3d211eee7b7e19753db8b7e44e` at plan
start. `compose` applies `self.convert` before `other.convert` in the forward
callback and `other.restore` before `self.restore` in the backward callback.
`modify` converts the source, applies `f` to the focus, then restores a source
value. Neither method has a documentation comment. Re-read the live methods
before editing. Adapt mechanical drift locally; treat contract or ownership
drift as a stop condition.

## 4. Required reads before implementation

- Repository `AGENTS.md` and `docs/workflow/implementation-record-lifecycle-convention.md`.
- This plan, `CHECKPOINT.md`, and the P20 walkthrough and handoff.
- `packages/universe/docs/modernization/GUIDELINES.md` and `INVENTORY.md`.
- `packages/universe/src/functional/optics.ph`.
- Live `git status --short` and the scoped source diff.

## 5. Normative authority

No additional normative rule is needed to document these direct method bodies.
The existing `Iso` type documentation states that callback round-trip laws are
obligations. Do not claim those laws are proven by composition or modification.

## 6. Takeover state

P1–P20 are complete as recorded in the checkpoint. The source inventory is
reconciled through P20 source commit `a66c51f84f50dc2476a36836cdd057e7849cb492`:
174 `.ph` paths, no additions or removals, and eighteen changed hashes against
MOD1. The shadow package remains non-production; source review does not
establish linked semantic identity or runtime behavior.

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| Iso composition | `Iso.compose`, `functional/optics.ph` | Shadow Functional source | Forward order is `self` then `other`; restore order is `other` then `self`. |
| Focus modification | `Iso.modify`, same module | Shadow Functional source | Convert, apply `f` to the focus, then restore a source value. |

## 7. Architecture

Place two comment lines immediately above `compose` and one above `modify`.
Suggested wording:

```telos
/// Applies this isomorphism, then `other`, in the forward direction.
/// Restores by applying `other` first, then this isomorphism.
/// Converts to the focus, applies `f`, then restores a source value.
```

Equivalent wording is acceptable only if it preserves these exact sequences.
Do not edit callback composition, Product bounds, reflection imports, or
adjacent methods.

## 8. Ownership boundaries

### Owns

- Two documentation lines above `Iso.compose` and one above `Iso.modify`.
- P21 checkpoint/validation evidence and inventory hash refresh.

### Does not own

- Iso behavior, API, generic identity, representation, callback laws, or
  module ownership.
- Product/reflection semantics, other optics, identity or linker work,
  production Universe, compiler, semantic, or runtime changes.
- Concurrent MODL001 P4 files, fixtures, plans, or lifecycle records.

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| Composition call order | `Iso.compose` body | Its documentation | A comment promising callback laws or a different order. |
| Focus update sequence | `Iso.modify` body | Its documentation | A second update algorithm or a guarantee about supplied callbacks. |

## 9. Global invariants

- **INV-01:** `compose` documentation states forward `self` then `other`, and
  backward `other` then `self`.
- **INV-02:** `modify` documentation states conversion, `f` on the focus, then
  restoration to a source value.
- **INV-03:** No declaration, signature, executable line, import, module owner,
  or generic identity changes.
- **INV-04:** No callback-law, count, purity, exception, or timing promise is
  added.
- **INV-05:** Product/reflection constraints and imports remain untouched.
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
- `packages/universe/docs/modernization/VALIDATION-LEDGER.md` — P21 evidence.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/`
  — checkpoint, plan metadata, walkthrough, and handoff.

No tests are expected or required. Unexpected executable source changes
require stopping and re-scoping rather than expanding this plan.

## 12. Implementer decision authority

### FIXED

- Preserve all sequences in INV-01 and INV-02.
- Preserve declaration, body, ownership, identity, and runtime behavior.
- Do not promise callback properties or round-trip laws.
- Do not edit Product/reflection imports or constraints.

### MECHANICALLY FLEXIBLE

- Exact wording and line wrapping, provided INV-01 through INV-07 hold.

### VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| `compose` and `modify` still match the described call sequences | Live method bodies | Stop and revise if contract drift is material. |
| No concurrent edit owns the target path | `git status --short` and scoped diff | Coordinate before writing. |

## 13. STOP / CONSULT triggers

Stop if accurate documentation would require changing Iso's API, identity,
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
| CV-02 | Whitespace hygiene | INV-03 | Scoped `git diff --check`. |
| CV-03 | Shadow semantic/runtime behavior | INV-06 | Not applicable to comments; no claim is made. |

## 16. Verification execution budget

Do not run tests for this comments-only change. Recompute the changed source
hash and refresh the inventory overlay. Independently compare the complete
live `.ph` path set and hashes with MOD1 before closing P21.

## 17. Baseline / unrelated failure policy

The prior P5 production-Universe Boolean corpus attempt failed before fixture
execution during canonical import resolution and remains assigned to active
MODL001 P4 context. Do not rerun or diagnose it here. Concurrent P4 changes
remain outside P21 ownership.

## T1 — Document Iso composition and modification

Add the two planned comments above `compose` and the one planned comment above
`modify`. Do not touch executable source.

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

At plan start, maintain the checkpoint ledger and active-plan pointer. At
completion, update plan status, inventory/validation ledger, and checkpoint
state. Create P21 walkthrough and handoff with exact source revision, change,
verification, deferred limitations, and the next safe bounded action.
