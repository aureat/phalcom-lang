---
id: UNIV002.C2.P8
category: UNIV
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - UNIV002.C2.P7
follows: UNIV002.C2.P7
supersedes: null
---

# UNIV002.C2.P8 — Buffer.capacity documentation

## 0. Executor contract

Make one Category A documentation-only change to a stable Priority 1
collections source module. Preserve all declarations, signatures, executable
expressions, module ownership, and native/runtime contracts. Do not infer
semantic or runtime verification from source review.

## 1. Goal

Document what `Buffer.capacity` returns from the live shadow source.

## 2. Checkpoint acceptance objective

This bounded source-quality slice advances `UNIV002.C2`; it does not close the
checkpoint or change buffer behavior.

## 3. Repository grounding

Prepared against:

```text
repository: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections
branch: feat/telos-collections
revision: 5b19385a3ff349461e11e6db15dd41fa61c11db0
```

`packages/universe/src/collections/buffer.ph` is unchanged from the P7 source
inventory. `Buffer` is described as owned bounded mutable storage, its
constructor accepts capacity, and `capacity` returns `rawCapacity`; `push`
describes failure when capacity is exhausted. No edit to the active
`collections/iteration/**` ownership is involved.

Re-read the live source and check target-path ownership before editing. Adapt
mechanical details locally; treat contract or ownership drift as a stop
condition.

## 4. Required reads before implementation

- Repository `AGENTS.md` and `docs/workflow/implementation-record-lifecycle-convention.md`.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/CHECKPOINT.md`.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/UNIV002.C2.P7-handoff.md`.
- `packages/universe/docs/modernization/GUIDELINES.md` and `INVENTORY.md`.
- `packages/universe/src/collections/buffer.ph`.

## 5. Normative authority

No buffer-specific normative specification is identified by this plan. The
comment must describe only the live source contract: `capacity` returns the
buffer's `rawCapacity`, which is the maximum number of elements the bounded
buffer can hold. Do not claim growth, shrinkage, or reservation behavior.

## 6. Takeover state

P1–P7 are complete as recorded in the checkpoint. The inventory is reconciled
through P7 source commit `58e2a31ec8d6823cb10d68e07def0d53cb62f94f`. The shadow
package remains non-production; parser/interface checks do not establish
method-body typing, canonical identity, or runtime behavior.

## 7. Architecture and ownership

The comment belongs immediately above the `capacity` getter in
`packages/universe/src/collections/buffer.ph`. The existing getter remains the
sole public projection of `rawCapacity`. No module path, native mapping,
representation, constructor, runtime behavior, or canonical identity changes
are authorized.

## 8. Ownership boundaries

### Owns

- One concise documentation comment for `Buffer.capacity`.
- P8 checkpoint/validation evidence and inventory hash refresh.

### Does not own

- Buffer capacity allocation, mutation, native implementation, or API changes.
- `collections/iteration/**`, Option/Result/Unit identity, reflection, or
  active MODL001 P4 linker work.
- Production Universe, provider/bootstrap, compiler, semantic, or runtime
  changes.

## 9. Global invariants

- **INV-01:** The comment states that capacity is the maximum element count the
  buffer can hold, consistent with the live getter and bounded-buffer contract.
- **INV-02:** No declaration, signature, executable line, module ownership, or
  native/runtime contract changes.
- **INV-03:** The comment makes no claim about dynamic resizing or reservation.
- **INV-04:** Shadow Universe remains non-production; source review and
  whitespace hygiene do not imply semantic or runtime verification.
- **INV-05:** Concurrent MODL001 P4 files remain untouched and unstaged.

## 10. Non-goals

- No source modernization beyond this comment.
- No constructor/API/representation or native implementation change.
- No behavior tests, broad suite, or inventory heuristic re-audit.
- No production source or compiler/semantic/runtime changes.

## 11. Expected impact map

- `packages/universe/src/collections/buffer.ph` — one getter comment.
- `packages/universe/docs/modernization/INVENTORY.md` — changed source hash
  and reconciliation revision.
- `packages/universe/docs/modernization/VALIDATION-LEDGER.md` — P8 evidence.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/`
  — checkpoint, walkthrough, and handoff.

## 12. Implementer decision authority

### FIXED

- Describe capacity as the maximum number of elements the buffer can hold.
- Preserve every source declaration and executable line.
- Do not imply dynamic resizing or add a second source of truth.

### MECHANICALLY FLEXIBLE

- Exact wording and line wrapping, provided INV-01 through INV-04 hold.

### VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| `capacity` still returns `rawCapacity` | Live `buffer.ph` | Stop and revise plan before editing |
| The buffer remains bounded and `push` reports exhaustion | Live type and method docs | Stop; do not infer a changed API contract |
| No concurrent edit owns the target path | `git status --short` | Coordinate before writing |

## 13. STOP / CONSULT triggers

Stop if accurate documentation requires deciding resize semantics, changing
the native API/representation, or editing another source of truth. Preserve all
concurrent MODL001 P4 work.

## 14. Debugging budget

This is a comment-only slice. Review the exact source diff and run scoped
`git diff --check`; no tests are required. If executable source changes, stop
and re-scope rather than expanding this plan.

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant | Evidence |
|---|---|---|---|
| CV-01 | Source-only scope | INV-01/02/03 | Exact diff review shows one comment and unchanged getter |
| CV-02 | Whitespace hygiene | INV-02 | Scoped `git diff --check` |
| CV-03 | Shadow semantic/runtime behavior | INV-04 | Not applicable to a comment; no claim is made |

## 16. Verification execution budget

Do not run Cargo tests for this comments-only change. Recompute the changed
source hash and refresh the inventory overlay after implementation.

## 17. Baseline / unrelated failure policy

The P5 production-Universe Boolean corpus attempt failed before fixture
execution during canonical import resolution and remains assigned to active
MODL001 P4 context. Do not rerun or diagnose it here. Concurrent P4 worktree
changes are outside P8 ownership.

## T1 — Document the capacity getter

Add one concise comment above `Buffer.capacity`, stating that it returns the
maximum number of elements the buffer can hold. Do not touch executable source.

## G1 — Scope and whitespace review

Confirm only the planned comment changed and run scoped `git diff --check`.

## G2 — Inventory reconciliation

Refresh the source hash/revision overlay for the 174 existing paths, preserve
all unchanged baseline hashes, and independently compare the path set and
hashes. Do not reinterpret textual inventory heuristics as a new audit.

## Outcome

Added one comment above `Buffer.capacity`: “Returns the maximum number of
elements this buffer can hold.” Review confirmed `capacity` returns
`rawCapacity`, while the type describes bounded storage and `push` documents
capacity exhaustion. The getter body, signature, and module path are unchanged.
Scoped `git diff --check` passed; no tests were run for this comments-only
change. An independent inventory comparison found 174 paths with no additions
or removals and 12 source hashes changed from MOD1 (the eleven reconciled
through P7 plus `collections/buffer.ph`). The P8 source commit is
`e24f37945b1dd6349932013cb6ebf89f33cf979c`; the current buffer source SHA-256
is `afe71fb114a044b19fae506f36c12f98d2887b7169132cdd17e36c7c960ccf92`.
Shadow semantic status remains `UNVERIFIED`; shadow runtime parity remains
`BLOCKED`.
