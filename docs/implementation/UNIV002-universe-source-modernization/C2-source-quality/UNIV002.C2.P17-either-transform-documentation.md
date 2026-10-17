---
id: UNIV002.C2.P17
category: UNIV
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - UNIV002.C2.P16
follows: UNIV002.C2.P16
supersedes: null
---

# UNIV002.C2.P17 — Either transformation documentation

## 0. Executor contract

Make three Category A documentation-only changes in the stable Functional
shadow module, limited to the related `Either.mapRight`, `Either.bimap`, and
`Either.swap` methods. Preserve every declaration, signature, executable
expression, module owner, generic identity, and runtime contract. Source review
establishes scope and whitespace hygiene only; it does not establish linked
semantics or runtime behavior.

## 1. Goal

Document which payload each transformation method changes and how `swap`
changes the selected variant while retaining the payload.

## 2. Checkpoint acceptance objective

This bounded source-quality slice advances `UNIV002.C2`; it does not change
Either behavior or close the checkpoint.

## 3. Repository grounding

Prepared against:

```text
repository: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections
branch: feat/telos-collections
revision: 09ba7159b094aa0e6137e148d92012a8529d93d2
```

At plan start, `packages/universe/src/functional/either.ph` has SHA-256
`338c9cc2a4220717c8f70f7b3116cc0eff343c21afbee7f1a81be4400f7ce716` and is
unmodified. `mapRight` applies its callback only to a Right payload and
preserves Left; `bimap` applies the corresponding callback to the selected
variant; `swap` changes Left to Right or Right to Left while forwarding the
same payload. The proposed comments are:

```telos
/// Transforms a `Right` payload and leaves a `Left` payload unchanged.
/// Applies `left` to a `Left` payload or `right` to a `Right` payload.
/// Reverses the variant while preserving its payload.
```

The comments describe the existing branches and do not change the closed-sum
representation or rely on Option/Result/Unit identity.

Re-read the live targets before editing and confirm they remain unmodified.
Treat source-contract or ownership drift as a stop condition.

## 4. Required reads before implementation

- Repository `AGENTS.md` and `docs/workflow/implementation-record-lifecycle-convention.md`.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/CHECKPOINT.md`.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/UNIV002.C2.P16-handoff.md`.
- `packages/universe/docs/modernization/GUIDELINES.md` and `INVENTORY.md`.
- `packages/universe/src/functional/either.ph`.

## 5. Normative authority

No Either-specific normative specification is identified for these comments.
Describe only the direct source contracts: `mapRight` transforms Right and
preserves Left; `bimap` applies the matching function to the selected
payload; `swap` reverses the variant and retains its payload. Do not call Left
an error or add callback-count, purity, exception, or timing guarantees.

## 6. Takeover state

P1–P16 are complete as recorded in the checkpoint. The source inventory is
reconciled through P16 source commit
`09ba7159b094aa0e6137e148d92012a8529d93d2`: 174 `.ph` paths, no additions or
removals, sixteen changed hashes. `functional/either.ph` already differs from
MOD1 and has the P15 `mapLeft` documentation; P17 refreshes its existing
overlay row rather than adding a changed path. The shadow package remains
non-production; source review does not establish linked semantic identity or
runtime behavior.

## 7. Architecture and ownership

The comments belong immediately above `mapRight`, `bimap`, and `swap` in
`packages/universe/src/functional/either.ph`. The existing signatures,
branches, payload forwarding, imports, module path, representation, and
runtime contracts remain unchanged.

## 8. Ownership boundaries

### Owns

- One concise documentation comment each for `Either.mapRight`,
  `Either.bimap`, and `Either.swap`.
- P17 checkpoint/validation evidence and the existing inventory hash refresh.

### Does not own

- Either behavior, API, generic identity, or representation.
- Option/Result/Unit identity; `collections/iteration/**`; reflection; or active
  MODL001 P4 linker work.
- Production Universe, providers/bootstrap, compiler, semantic, or runtime
  changes.
- Any concurrent MODL001 P4 file, fixture, plan, or lifecycle record.

## 9. Global invariants

- **INV-01:** Each comment describes only its corresponding existing branch
  mapping and payload preservation.
- **INV-02:** No declaration, signature, executable line, import, or module
  ownership changes.
- **INV-03:** The comments do not characterize Left as an error or add callback,
  purity, exception, or timing guarantees.
- **INV-04:** Shadow Universe remains non-production; source review and
  whitespace hygiene do not imply semantic or runtime verification.
- **INV-05:** Concurrent MODL001 P4 files remain untouched and unstaged.

## 10. Non-goals

- No source modernization beyond the three method comments.
- No behavior, API, type, identity, or representation changes.
- No tests, broad suite, or inventory heuristic re-audit.
- No production source or compiler/semantic/runtime changes.

## 11. Expected impact map

- `packages/universe/src/functional/either.ph` — three method comments.
- `packages/universe/docs/modernization/INVENTORY.md` — existing changed-file
  hash and source revision refresh.
- `packages/universe/docs/modernization/VALIDATION-LEDGER.md` — P17 evidence.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/`
  — checkpoint, walkthrough, and handoff.

## 12. Implementer decision authority

### FIXED

- Preserve the selected branch, corresponding callback/payload, and variant
  conversion described by each method.
- Preserve every declaration and executable expression.
- Do not call Left an error or add guarantees beyond the source.

### MECHANICALLY FLEXIBLE

- Exact comment wording and line wrapping, provided INV-01 through INV-04
  hold.

### VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| All three methods still match the stated variant branches and forward payloads | Live `either.ph` | Stop and revise the plan before editing |
| No concurrent edit owns the target path | `git status --short` | Coordinate before writing |

## 13. STOP / CONSULT triggers

Stop if accurate documentation requires changing Either's API, identity,
representation, or a second source of truth.

## 14. Debugging budget

This is a comments-only slice. Review the exact source diff and run scoped
`git diff --check`; no tests are required. If executable source changes, stop
and re-scope rather than expanding this plan.

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant | Evidence |
|---|---|---|---|
| CV-01 | Source-only scope | INV-01/02/03 | Exact diff review shows three comments and unchanged methods |
| CV-02 | Whitespace hygiene | INV-02 | Scoped `git diff --check` |
| CV-03 | Shadow semantic/runtime behavior | INV-04 | Not applicable to comments; no claim is made |

## 16. Verification execution budget

Do not run Cargo tests for this comments-only change. Recompute the changed
source hash and refresh the inventory overlay after implementation. Independently
compare the complete live `.ph` path set and hashes with MOD1 before closing
P17.

## 17. Baseline / unrelated failure policy

The P5 production-Universe Boolean corpus attempt failed before fixture
execution during canonical import resolution and remains assigned to active
MODL001 P4 context. Do not rerun or diagnose it here. Concurrent P4 worktree
changes are outside P17 ownership.

## T1 — Document Either transformations

Add the three specified comments above `mapRight`, `bimap`, and `swap`. Do
not touch executable source.

## G1 — Scope and whitespace review

Confirm only the three planned comments changed and run scoped
`git diff --check`.

## G2 — Inventory reconciliation

Refresh the `functional/either.ph` hash and source-revision overlay for the
174 existing paths, preserve unchanged baseline hashes, and independently
compare the path set and hashes. Do not reinterpret textual inventory
heuristics as a new audit.
