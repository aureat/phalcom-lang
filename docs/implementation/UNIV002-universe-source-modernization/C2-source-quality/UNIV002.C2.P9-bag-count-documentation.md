---
id: UNIV002.C2.P9
category: UNIV
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - UNIV002.C2.P8
follows: UNIV002.C2.P8
supersedes: null
---

# UNIV002.C2.P9 — Bag.count documentation

## 0. Executor contract

Make one Category A documentation-only change in a stable Priority 1
collections source module. Preserve all declarations, signatures, executable
expressions, module ownership, and native/runtime contracts. Do not infer
semantic or runtime verification from source review.

## 1. Goal

Document the occurrence-count contract of `Bag.count` from the live shadow
source and its native implementation.

## 2. Checkpoint acceptance objective

This bounded source-quality slice advances `UNIV002.C2`; it does not change
multiset behavior or close the checkpoint.

## 3. Repository grounding

Prepared against:

```text
repository: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections
branch: feat/telos-collections
revision: 9b17bf7b7211df4eb582271a297fc3dfc6dc6004
```

`packages/universe/src/collections/bag.ph` is unchanged from the P8 source
inventory. `Bag.count(value)` returns `self.rawCount(value)`. The native
`bag_raw_count` looks up the value's stored multiplicity and returns zero when
the value is absent. `BagObject` stores the total occurrence count separately
from its number of distinct keys. No `collections/iteration/**` file is in
scope.

Re-read the live source and relevant native implementation before editing.
Check target-path ownership. Adapt mechanical details locally; treat contract
or ownership drift as a stop condition.

## 4. Required reads before implementation

- Repository `AGENTS.md` and `docs/workflow/implementation-record-lifecycle-convention.md`.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/CHECKPOINT.md`.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/UNIV002.C2.P8-handoff.md`.
- `packages/universe/docs/modernization/GUIDELINES.md` and `INVENTORY.md`.
- `packages/universe/src/collections/bag.ph`.
- `phalcom-core/src/primitive/bag.rs::bag_raw_count`.

## 5. Normative authority

No bag-specific normative specification is identified by this plan. The
comment must describe only the live public/native behavior: count returns the
number of occurrences of a value in the bag. Do not claim ordering, equality,
or mutation semantics.

## 6. Takeover state

P1–P8 are complete as recorded in the checkpoint. The inventory is reconciled
through P8 source commit `e24f37945b1dd6349932013cb6ebf89f33cf979c`. Shadow
Universe remains non-production; source review does not establish semantic
identity or runtime behavior for this package.

## 7. Architecture and ownership

The comment belongs immediately above `count(_ value: T) -> Int` in
`packages/universe/src/collections/bag.ph`. `Bag.count` remains the public
source wrapper and `bag_raw_count` remains the native implementation owner.
No module path, source/native mapping, representation, signature, or behavior
change is authorized.

## 8. Ownership boundaries

### Owns

- One concise documentation comment for `Bag.count`.
- P9 checkpoint/validation evidence and inventory hash refresh.

### Does not own

- Bag representation, hashing/equality, algebra, iteration, or native behavior.
- `collections/iteration/**`, Option/Result/Unit identity, reflection, or
  active MODL001 P4 linker work.
- Production Universe, provider/bootstrap, compiler, semantic, or runtime
  changes.

## 9. Global invariants

- **INV-01:** The comment says `count` returns the number of occurrences of
  the supplied value, matching source and native lookup behavior.
- **INV-02:** No declaration, signature, executable line, module ownership, or
  native/runtime contract changes.
- **INV-03:** The comment does not strengthen equality, ordering, mutation, or
  missing-value semantics beyond the observed occurrence count.
- **INV-04:** Shadow Universe remains non-production; source review and
  whitespace hygiene do not imply semantic or runtime verification.
- **INV-05:** Concurrent MODL001 P4 files remain untouched and unstaged.

## 10. Non-goals

- No source modernization beyond this comment.
- No collection behavior, API, type, or representation changes.
- No behavior tests, broad suite, or inventory heuristic re-audit.
- No production source or compiler/semantic/runtime changes.

## 11. Expected impact map

- `packages/universe/src/collections/bag.ph` — one method comment.
- `packages/universe/docs/modernization/INVENTORY.md` — changed source hash
  and reconciliation revision.
- `packages/universe/docs/modernization/VALIDATION-LEDGER.md` — P9 evidence.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/`
  — checkpoint, walkthrough, and handoff.

## 12. Implementer decision authority

### FIXED

- Describe only the occurrence count returned by `Bag.count`.
- Preserve every source declaration and executable line.
- Do not reimplement native counting or add a second source of truth.

### MECHANICALLY FLEXIBLE

- Exact wording and line wrapping, provided INV-01 through INV-04 hold.

### VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| `count` still delegates to `rawCount` | Live `bag.ph` | Stop and revise plan before editing |
| Native lookup returns a stored count or zero | `primitive/bag.rs::bag_raw_count` | Stop and report contract drift |
| No concurrent edit owns the target path | `git status --short` | Coordinate before writing |

## 13. STOP / CONSULT triggers

Stop if accurate documentation requires changing the native API,
representation, identity, or a second source of truth. Preserve concurrent
MODL001 P4 work.

## 14. Debugging budget

This is a comment-only slice. Review the exact source diff and run scoped
`git diff --check`; no tests are required. If executable source changes, stop
and re-scope rather than expanding this plan.

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant | Evidence |
|---|---|---|---|
| CV-01 | Source-only scope | INV-01/02/03 | Exact diff review shows one comment and unchanged method |
| CV-02 | Whitespace hygiene | INV-02 | Scoped `git diff --check` |
| CV-03 | Shadow semantic/runtime behavior | INV-04 | Not applicable to a comment; no claim is made |

## 16. Verification execution budget

Do not run Cargo tests for this comments-only change. Recompute the changed
source hash and refresh the inventory overlay after implementation.

## 17. Baseline / unrelated failure policy

The P5 production-Universe Boolean corpus attempt failed before fixture
execution during canonical import resolution and remains assigned to active
MODL001 P4 context. Do not rerun or diagnose it here. Concurrent P4 worktree
changes are outside P9 ownership.

## T1 — Document the occurrence count

Add one concise comment above `Bag.count`, stating that it returns the number
of occurrences of the supplied value. Do not touch executable source.

## G1 — Scope and whitespace review

Confirm only the planned comment changed and run scoped `git diff --check`.

## G2 — Inventory reconciliation

Refresh the source hash/revision overlay for the 174 existing paths, preserve
all unchanged baseline hashes, and independently compare the path set and
hashes. Do not reinterpret textual inventory heuristics as a new audit.
