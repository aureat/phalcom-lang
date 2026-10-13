---
id: UNIV002.C2.P20
category: UNIV
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on:
  - UNIV002.C2.P19
follows: UNIV002.C2.P19
supersedes: null
---

# UNIV002.C2.P20 — Iso.inverse documentation

## 0. Executor contract

Make one Category A documentation-only change immediately above `Iso.inverse`
in the stable Functional shadow module. Preserve the declaration, signature,
executable expression, module ownership, generic identities, and runtime
contract. The comment describes the direct callback exchange only; it does not
claim that supplied callbacks satisfy inverse laws. Source review establishes
scope and whitespace hygiene, not linked semantics or runtime behavior.

## 1. Goal

Document that `Iso.inverse` constructs an `Iso` with its forward and backward
callbacks exchanged.

## 2. Checkpoint acceptance objective

This is one of several Category A plans advancing `UNIV002.C2`; it does not
close the checkpoint.

## 3. Repository grounding

Prepared against:

```text
repository: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections
branch: feat/telos-collections
revision: 5c7f8a1e2675cb486f860f3d22877ef782c06b55
predecessor: UNIV002.C2.P19
```

`packages/universe/src/functional/optics.ph` has SHA-256
`67aed3cff74133bd826bbcebe7e1a2711f8e62963e9ab7f5f03a7a0ddcba70bc` at plan
start. `Iso.inverse` constructs `Iso<T, S>` with `self.backward` as `forward`
and `self.forward` as `backward`. There is no comment on this method. The type
documentation already states that round-trip laws are callback obligations.
Re-read the live method before editing. Adapt mechanical drift locally; treat
contract or ownership drift as a stop condition.

## 4. Required reads before implementation

- Repository `AGENTS.md` and `docs/workflow/implementation-record-lifecycle-convention.md`.
- This plan, `CHECKPOINT.md`, and the P19 walkthrough and handoff.
- `packages/universe/docs/modernization/GUIDELINES.md` and `INVENTORY.md`.
- `packages/universe/src/functional/optics.ph`.
- Live `git status --short` and the scoped source diff.

## 5. Normative authority

No additional normative rule is needed to document this direct method body.
Describe the swapped callbacks only. Do not claim callback round-trip laws;
the existing `Iso` type comment correctly identifies those laws as callback
obligations.

## 6. Takeover state

P1–P19 are complete as recorded in the checkpoint. The source inventory is
reconciled through P19 source commit `5c7f8a1e2675cb486f860f3d22877ef782c06b55`:
174 `.ph` paths, no additions or removals, and seventeen changed hashes
against MOD1. The shadow package remains non-production; source review does
not establish linked semantic identity or runtime behavior.

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| Isomorphism callback pair | `Iso<S,T>`, `functional/optics.ph` | Shadow Functional source | Forward maps `S` to `T`; backward maps `T` to `S`. |
| Inverse construction | `Iso.inverse`, same module | Shadow Functional source | Returned fields exchange the existing forward and backward callbacks. |

## 7. Architecture

Place one comment immediately above `inverse` in
`packages/universe/src/functional/optics.ph`. Suggested wording:

```telos
/// Exchanges the forward and backward callbacks.
```

No code or method contract changes. Do not edit the Product constraints,
reflection imports, adjacent optics, or type-level round-trip statement.

## 8. Ownership boundaries

### Owns

- One comment above `Iso.inverse`.
- P20 checkpoint/validation evidence and inventory reconciliation.

### Does not own

- Iso behavior, callback laws, API, generic identity, representation, or
  module ownership.
- Product/reflection semantics, other optics, identity or linker work,
  production Universe, compiler, semantic, or runtime changes.
- Concurrent MODL001 P4 files, fixtures, plans, or lifecycle records.

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| Callback exchange | `Iso.inverse` body | Its documentation | A separately maintained inverse implementation. |
| Callback round-trip laws | Callbacks supplied by clients, as noted by `Iso` docs | Call sites | A comment claiming the laws are proven by this method. |

## 9. Global invariants

- **INV-01:** The comment states only that the forward and backward callbacks
  are exchanged.
- **INV-02:** No declaration, signature, executable line, import, module owner,
  or generic identity changes.
- **INV-03:** No round-trip law or callback property is promised.
- **INV-04:** The adjacent Product/reflection constraints remain untouched.
- **INV-05:** Shadow Universe remains non-production; no semantic/runtime claim
  is made from comments or source hashes.
- **INV-06:** Concurrent MODL001 P4 changes remain untouched and unstaged.

## 10. Non-goals

- No executable source modernization, API/type/identity change, or new test.
- No production cutover or provider/bootstrap/native/runtime change.
- No reflection/Product semantics, Option/Result/Unit identity, or data migration.
- No inventory heuristic rescan or broad test run.

## 11. Expected impact map

- `packages/universe/src/functional/optics.ph` — exactly one method comment.
- `packages/universe/docs/modernization/INVENTORY.md` — add its current hash
  to the changed-file overlay and refresh the source revision.
- `packages/universe/docs/modernization/VALIDATION-LEDGER.md` — P20 evidence.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/`
  — checkpoint, plan metadata, walkthrough, and handoff.

No tests are expected or required. Unexpected executable source changes
require stopping and re-scoping rather than expanding this plan.

## 12. Implementer decision authority

### FIXED

- Preserve the callback exchange and avoid claiming callback laws.
- Preserve declaration, executable expression, owner, identity, and behavior.
- Do not edit the Product/reflection constraints or adjacent methods.

### MECHANICALLY FLEXIBLE

- Exact comment wording and wrapping, provided INV-01 through INV-06 hold.

### VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| `inverse` still exchanges `self.backward` and `self.forward` | Live method body | Stop and revise if contract drift is material. |
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
| CV-01 | Source-only scope | INV-01/02/03/04 | Exact diff review confirms one comment and unchanged method. |
| CV-02 | Whitespace hygiene | INV-02 | Scoped `git diff --check`. |
| CV-03 | Shadow semantic/runtime behavior | INV-05 | Not applicable to comments; no claim is made. |

## 16. Verification execution budget

Do not run tests for this comments-only change. Recompute the changed source
hash and refresh the inventory overlay. Independently compare the complete
live `.ph` path set and hashes with MOD1 before closing P20.

## 17. Baseline / unrelated failure policy

The prior P5 production-Universe Boolean corpus attempt failed before fixture
execution during canonical import resolution and remains assigned to active
MODL001 P4 context. Do not rerun or diagnose it here. Concurrent P4 changes
remain outside P20 ownership.

## T1 — Document callback exchange

Add the single planned comment above `inverse`. Do not touch executable source.

## G1 — Scope and whitespace review

Confirm only the planned comment changed and run scoped `git diff --check`.

## G2 — Inventory reconciliation

Add/update the `functional/optics.ph` hash in the changed-file overlay and
refresh the source revision for the 174 existing paths. Preserve unchanged
baseline hashes and independently compare the path set and hashes. Do not
reinterpret textual inventory heuristics as a new audit.

## 18. Final focused acceptance

| Coverage ID | Acceptance evidence |
|---|---|
| CV-01 | Exact diff contains only the planned comment. |
| CV-02 | Scoped `git diff --check` passes. |
| CV-03 | No semantic or runtime claim is made. |

## 19. Checkpoint bookkeeping and completion artifacts

At plan start, maintain the checkpoint ledger and active-plan pointer. At
completion, update plan status, inventory/validation ledger, and checkpoint
state. Create P20 walkthrough and handoff with exact source revision, change,
verification, deferred limitations, and the next safe bounded action.
