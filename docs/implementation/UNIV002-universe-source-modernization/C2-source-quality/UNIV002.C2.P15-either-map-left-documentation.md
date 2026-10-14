---
id: UNIV002.C2.P15
category: UNIV
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - UNIV002.C2.P14
follows: UNIV002.C2.P14
supersedes: null
---

# UNIV002.C2.P15 — Either.mapLeft documentation

## 0. Executor contract

Make one Category A documentation-only change in the stable Functional shadow
module. Preserve every declaration, signature, executable expression, module
owner, generic identity, and runtime contract. Source review establishes scope
and whitespace hygiene only; it does not establish linked semantics or runtime
behavior.

## 1. Goal

Document that `Either.mapLeft` transforms a `Left` payload and preserves a
`Right` payload unchanged.

## 2. Checkpoint acceptance objective

This bounded source-quality slice advances `UNIV002.C2`; it does not change
Either behavior or close the checkpoint.

## 3. Repository grounding

Prepared against:

```text
repository: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections
branch: feat/telos-collections
revision: bb2d55a2d2571121f6b710dea88acf8a6f930703
```

At plan start, `packages/universe/src/functional/either.ph` has SHA-256
`4ea073b9ec44ab2f26ac6381d377266284867060c5007a031dc58bb878368824` and is
unmodified. `mapLeft` matches the `Either::Left` variant, applies
`f.call(value)` only there, and returns an unchanged `Either::Right(value)` for
the other variant. The proposed comment is:

```telos
/// Transforms a `Left` payload and leaves a `Right` payload unchanged.
```

This source-only comment does not use Option, Result, or Unit identities and
does not change the `Either` type's representation or canonical identity.

Re-read the live target before editing and confirm it remains unmodified.
Treat source-contract or ownership drift as a stop condition.

## 4. Required reads before implementation

- Repository `AGENTS.md` and `docs/workflow/implementation-record-lifecycle-convention.md`.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/CHECKPOINT.md`.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/UNIV002.C2.P14-handoff.md`.
- `packages/universe/docs/modernization/GUIDELINES.md` and `INVENTORY.md`.
- `packages/universe/src/functional/either.ph`.

## 5. Normative authority

No Either-specific normative specification is identified for this comment.
Describe only the direct source contract: `mapLeft` maps the Left payload and
passes a Right payload unchanged. Do not imply that Left represents an error,
or add callback-count, purity, exception, or evaluation-timing guarantees.

## 6. Takeover state

P1–P14 are complete as recorded in the checkpoint. The source inventory is
reconciled through P14 source commit
`cb6ceb0ebe53cdded4e57effddc4c91665dde403`: 174 `.ph` paths, no additions or
removals, sixteen changed hashes. `functional/either.ph` already differs from
MOD1; P15 refreshes its existing overlay row rather than adding a changed path.
The shadow package remains non-production; source review does not establish
linked semantic identity or runtime behavior.

## 7. Architecture and ownership

The comment belongs immediately above `mapLeft<A>(_ f: (L) -> A) -> Either<A, R>`
in `packages/universe/src/functional/either.ph`. The method continues to map
only a Left payload and to pass a Right payload through unchanged. No module
path, imports, source/native mapping, representation, signature, or behavior
change is authorized.

## 8. Ownership boundaries

### Owns

- One concise documentation comment for `Either.mapLeft`.
- P15 checkpoint/validation evidence and the existing inventory hash refresh.

### Does not own

- Either behavior, API, generic identity, or representation.
- Option/Result/Unit identity; `collections/iteration/**`; reflection; or active
  MODL001 P4 linker work.
- Production Universe, providers/bootstrap, compiler, semantic, or runtime
  changes.
- Any concurrent MODL001 P4 file, fixture, plan, or lifecycle record.

## 9. Global invariants

- **INV-01:** The comment states that `mapLeft` maps a Left payload and retains
  a Right payload unchanged.
- **INV-02:** No declaration, signature, executable line, import, or module
  ownership changes.
- **INV-03:** The comment does not call Left an error and adds no callback,
  purity, exception, or timing guarantee.
- **INV-04:** Shadow Universe remains non-production; source review and
  whitespace hygiene do not imply semantic or runtime verification.
- **INV-05:** Concurrent MODL001 P4 files remain untouched and unstaged.

## 10. Non-goals

- No source modernization beyond this comment.
- No behavior, API, type, identity, or representation changes.
- No tests, broad suite, or inventory heuristic re-audit.
- No production source or compiler/semantic/runtime changes.

## 11. Expected impact map

- `packages/universe/src/functional/either.ph` — one method comment.
- `packages/universe/docs/modernization/INVENTORY.md` — existing changed-file
  hash and source revision refresh.
- `packages/universe/docs/modernization/VALIDATION-LEDGER.md` — P15 evidence.
- `docs/implementation/UNIV002-universe-source-modernization/C2-source-quality/`
  — checkpoint, walkthrough, and handoff.

## 12. Implementer decision authority

### FIXED

- Describe the Left payload transformation and unchanged Right payload only.
- Preserve every declaration and executable expression.
- Do not characterize Left as an error or make behavior claims beyond source.

### MECHANICALLY FLEXIBLE

- Exact comment wording and line wrapping, provided INV-01 through INV-04
  hold.

### VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| `mapLeft` still maps Left and passes Right through unchanged | Live `either.ph` | Stop and revise the plan before editing |
| No concurrent edit owns the target path | `git status --short` | Coordinate before writing |

## 13. STOP / CONSULT triggers

Stop if accurate documentation requires changing Either's API, identity,
representation, or a second source of truth.

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
source hash and refresh the inventory overlay after implementation. Independently
compare the complete live `.ph` path set and hashes with MOD1 before closing
P15.

## 17. Baseline / unrelated failure policy

The P5 production-Universe Boolean corpus attempt failed before fixture
execution during canonical import resolution and remains assigned to active
MODL001 P4 context. Do not rerun or diagnose it here. Concurrent P4 worktree
changes are outside P15 ownership.

## T1 — Document Either.mapLeft

Add one concise comment above `Either.mapLeft`, stating that it transforms a
Left payload and leaves a Right payload unchanged. Do not touch executable
source.

## G1 — Scope and whitespace review

Confirm only the planned comment changed and run scoped `git diff --check`.

## G2 — Inventory reconciliation

Refresh the `functional/either.ph` hash and source-revision overlay for the
174 existing paths, preserve unchanged baseline hashes, and independently
compare the path set and hashes. Do not reinterpret textual inventory
heuristics as a new audit.
