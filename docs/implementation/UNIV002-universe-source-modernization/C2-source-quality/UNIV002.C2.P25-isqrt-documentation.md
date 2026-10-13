---
id: UNIV002.C2.P25
category: UNIV
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on:
  - UNIV002.C2.P24
follows: UNIV002.C2.P24
supersedes: null
---

# UNIV002.C2.P25 — Integer square-root documentation

## 0. Executor contract

Add one Category A documentation line immediately above `IntegerMath.isqrt`'s
`@class` annotation. Preserve the algorithm, declaration, signature, imports,
integer owner, and behavior. This documents the accepted contract and live
invalid-input path; it does not verify the shadow implementation.

## 1. Goal

Document the floor-square-root result for nonnegative inputs and the
`ArgumentError` raised for negative inputs.

## 2. Checkpoint acceptance objective

This is one of several Category A plans advancing `UNIV002.C2`; it does not
close the checkpoint.

## 3. Repository grounding

Prepared against:

```text
repository: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections
branch: feat/telos-collections
revision: 0e6c058a1dc5f90849f91adf4a16ad3f53eea6ad
predecessor: UNIV002.C2.P24
source: packages/universe/src/math/integer.ph
source SHA-256 at plan start: 6571fb9a16b20a35a30a4c9d0af1ab427d7d1b1dfdbd2f751f3b3ba519b9ab90
```

The live method raises `ArgumentError` for negative input and uses integer
iteration to produce the floor root otherwise. The source and normative spec
are not concurrently modified. Re-read both before editing. Adapt only
mechanical drift; treat semantic or ownership drift as a stop condition.

## 4. Required reads before implementation

- Repository `AGENTS.md` and `docs/workflow/implementation-record-lifecycle-convention.md`.
- This plan, C2 `CHECKPOINT.md`, and the P24 walkthrough and handoff.
- `docs/implementation/STDL001-numeric-library/C9-math-foundation/CHECKPOINT.md`.
- `docs/specs/numeric-math.md`, “Integer algorithms”.
- `packages/universe/src/math/integer.ph` and its scoped diff.
- Live `git status --short`.

## 5. Normative authority

`docs/specs/numeric-math.md` is normative for `universe.math`. Its integer
algorithm table specifies `isqrt(n)` as the floor of the nonnegative square
root and makes negative input invalid. The live method raises `ArgumentError`
for negative input. The comment must not add a resource, performance,
complexity, or rounding promise.

## 6. Takeover state

`STDL001.C9.P1` keeps exact integer algorithms in `universe.math`. Phalcom
source compilation and runtime behavior remain unverified; P25 does not advance
those gates.

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| Exact integer math | `IntegerMath` in `math/integer.ph` | Shadow `universe.math` | Exact Int algorithms retain their current owner. |
| Integer square root | `IntegerMath.isqrt` | Same module | Floor root for nonnegative input; negative input is invalid. |

## 7. Architecture

Add exactly this line immediately before `@class` for `isqrt`:

```telos
/// Returns the floor of the square root; negative inputs raise `ArgumentError`.
```

The existing method remains the sole source of executable behavior. The
accepted numeric spec remains the authority for the public result contract.

## 8. Ownership boundaries

### Owns

- One comment immediately above `IntegerMath.isqrt`'s `@class` annotation.
- P25 lifecycle evidence and `math/integer.ph` inventory hash refresh.

### Does not own

- The integer square-root algorithm, numeric API, integer identity, or runtime
  behavior.
- C9 acceptance, math/numerics ownership, Option/Result/Unit identity,
  iteration, reflection, compiler, semantic, production Universe, or VM work.
- Concurrent MODL001 P4 or UNIV002 C3 crypto changes.

## 9. Global invariants

- **INV-01:** The comment states only the normative result and live negative
  input error path.
- **INV-02:** No executable line, declaration, signature, import, owner, API,
  identity, or resource policy changes.
- **INV-03:** No performance, allocation, complexity, or runtime guarantee is
  added.
- **INV-04:** `universe.math` remains the owner of exact Int algorithms.
- **INV-05:** Shadow Universe remains non-production; no semantic/runtime
  claim follows from the comment.
- **INV-06:** Concurrent MODL001 P4 and C3 crypto changes remain untouched and
  unstaged.

## 10. Non-goals

- No algorithm rewrite, executable source modernization, API change, or test.
- No edit to `docs/specs/numeric-math.md` or C9 status.
- No broad inventory rescan or source-quality heuristic audit.

## 11. Expected impact map

- `packages/universe/src/math/integer.ph` — exactly one comment.
- `packages/universe/docs/modernization/INVENTORY.md` — current hash and
  source revision overlay.
- `packages/universe/docs/modernization/VALIDATION-LEDGER.md` — P25 evidence.
- UNIV002 C2 checkpoint, plan metadata, walkthrough, and handoff.

No tests are expected or required. Any executable source change requires
stopping and rescoping rather than expanding P25.

## 12. Implementer decision authority

### FIXED

- Preserve INV-01 through INV-06 and the exact floor-root result.
- Keep the edit comment-only; make no complexity or runtime claim.

### MECHANICALLY FLEXIBLE

- None; the specified comment is short and directly grounded.

### VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| Source error path and result match the spec | Live `IntegerMath.isqrt` body | Stop and report the discrepancy; do not rewrite behavior. |
| Target source and spec have no competing edits | Scoped status and diff | Coordinate before editing. |

## 13. STOP / CONSULT triggers

Stop if the source contradicts the accepted contract, or if accurate
documentation requires changing integer semantics, identity, representation,
or ownership. Follow the repository escalation protocol for any architectural
or specification conflict.

## 14. Debugging budget

This comments-only task needs exact source review and scoped `git diff --check`.
Do not run tests. If executable code changes, stop and rescope.

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant | Evidence |
|---|---|---|---|
| CV-01 | Comment-only scope | INV-01 through INV-04 | Exact diff contains one comment and preserves the method. |
| CV-02 | Whitespace hygiene | INV-02 | Scoped `git diff --check`. |
| CV-03 | Inventory reconciliation | INV-05 | All 174 paths/hashes independently compared with MOD1 and overlay. |
| CV-04 | Semantic/runtime status | INV-05 | No behavior claim; no tests run. |

## 16. Verification execution budget

Do not run tests for this documentation-only change. Refresh the current hash
for `math/integer.ph`, then independently compare the complete live `.ph`
path/hash set with the MOD1 baseline. Preserve baseline heuristic columns.

## 17. Baseline / unrelated failure policy

The prior P5 production-Universe Boolean corpus attempt failed before fixture
execution during canonical import resolution and remains assigned to active
MODL001 P4 context. Do not rerun or diagnose it here. Preserve active P4 and
C3 crypto changes.

## T1 — Document integer square root

Add the specified comment before `@class` for `IntegerMath.isqrt`. Change no
executable source.

## G1 — Scope and whitespace review

Confirm the exact source diff contains only the one comment and run scoped
`git diff --check`.

## G2 — Inventory reconciliation

Refresh the integer source hash and source revision. Verify all 174 source
paths and hashes against MOD1; confirm the overlay exactly equals the live
changed-hash map.

## 18. Final focused acceptance

| Coverage ID | Acceptance evidence |
|---|---|
| CV-01 | Exact diff contains only the `isqrt` comment. |
| CV-02 | Scoped `git diff --check` passes. |
| CV-03 | All 174 paths/hashes reconcile; overlay records the current hash. |
| CV-04 | No semantic or runtime claim is made. |

## 19. Checkpoint bookkeeping and completion artifacts

At completion, update P25 status, inventory/validation ledger, and checkpoint
state. Create the P25 walkthrough and handoff with the exact source revision,
actual comment diff, evidence, semantic/runtime limits, and the next safe
bounded action.
