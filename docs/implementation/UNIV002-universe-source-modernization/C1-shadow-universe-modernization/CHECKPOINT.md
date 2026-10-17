---
id: UNIV002.C1
category: UNIV
program: UNIV002
checkpoint: UNIV002.C1
kind: checkpoint-record
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# UNIV002.C1 — shadow Universe modernization baseline and pilot

## Objective and boundary

Create a source-wide shadow-Universe inventory, document the supported modern
Telos syntax and modernization rules, and complete a small, behavior-preserving
pilot without changing module ownership. This checkpoint owns MOD1 and its first
bounded MOD2–MOD4 pilot. It does not own broad domain migration, native/runtime
representation changes, or production cutover.

## Plan ledger

| Plan | Scope | Status | Completion | Verification | Outcome |
|---|---|---|---|---|---|
| `UNIV002.C1.P1` | Source inventory, guidelines, and language-feature matrix (MOD1) | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | 174 live source paths covered exactly once; all recorded hashes match. Parser/interface history is clearly distinguished from current semantic/runtime evidence. |
| `UNIV002.C1.P2` | First source-modernization pilot selected from the inventory | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Five functions modernized; source-level equivalence reviewed; exact shadow parser/unlinked-interface test passed. Semantic linking remains unverified and runtime parity blocked. |

## Established boundaries

- The target is the shadow source tree `packages/universe/src/**`.
- The production Universe remains `phalcom-core/core/universe/**`.
- No production provider/bootstrap/native/runtime cutover is authorized.
- Source moves and semantic/API/representation changes are outside the pilot.
- Textual occurrence counts are search leads, not proof that a rewrite is
  eligible; ambiguous matches must be marked for manual/type-aware review.
- Native class-to-data conversion is a separate checkpoint and cannot be called
  complete until host construction and runtime semantics are validated.
- The current shadow validator parses sources and builds unlinked interfaces;
  it does not typecheck bodies, prove exact-site match exhaustiveness, or run
  shadow code. P2 permits a bounded source-level pilot while preserving
  semantic-unverified/runtime-blocked status.

## Starting state

- Branch: `feat/telos-collections`.
- Revision: `18aaf8b568a0b45dd450befc35932bb7a1fdd442`.
- Worktree has concurrent uncommitted P4 semantic-linking edits in
  `phalcom-semantic/src/session.rs`, `workspace.rs`, and
  `tests/semantic/integration/resolver.rs`; this checkpoint must preserve them
  and avoid those files.
- No tests or compiler checks have been run for UNIV002.C1 at checkpoint start.

## Consultation / amendment ledger

- **P2 verification amendment:** A bounded validator investigation found no
  current path that semantically links or executes the complete shadow source
  graph. P2 now accepts source-level causal review and optional
  parse/unlinked-interface validation; behavior/API invariants remain fixed,
  while semantic verification is unavailable and runtime parity is blocked.

## Active plan and next action

- Active: none. Latest complete: `UNIV002.C1.P2`.
- Next: begin `UNIV002.C2` for the next source-modernization slice, selected by
  stable module ownership and the inventory. Keep behavior-changing syntax
  work separate from data representation and broader module moves.
- Do not modify the active P4 semantic-linking files.
