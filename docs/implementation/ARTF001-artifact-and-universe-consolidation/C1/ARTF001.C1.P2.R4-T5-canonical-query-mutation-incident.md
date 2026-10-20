# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `ARTF001`
- Checkpoint: `ARTF001.C1`
- Plan: `ARTF001.C1.P2.R4`
- Task/Gate: `R4.T5.5` local source overlay; regression also exercises the `T5.3` initial session contract
- Repository/worktree: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact`
- Branch: `feat/artifact-universe-consolidation`
- Starting revision: `dbec7ec05cee04bf7b5d61941c5bb12e4a7068cd`
- Current HEAD at capture: `dbec7ec05cee04bf7b5d61941c5bb12e4a7068cd` (working tree changes are uncommitted)

## 2. Escalation trigger

Mandatory trigger 7 fired: the local session update crosses the semantic-authority boundary and attempts to mutate a query owned by the immutable canonical image. Trigger 11 also applies because plausible corrections change different architectural boundaries: current query-root fingerprint parity or session bootstrap ownership.

## 3. Required invariant

Imported canonical query roots remain immutable. Local workspace updates may publish local query state and depend on canonical roots, but may not duplicate, overwrite, retire, or invalidate canonical roots into local state. This is required by R4.T5.3–T5.5 and R4.T4.7.

## 4. Expected architecture

`SemanticWorkspaceSession::from_image` uses the loaded `CanonicalUniverseFormalImage` as its ordinary session base. Its initial and last-known-good snapshots share the image snapshot. Local source updates use normal session/query publication, while canonical query products remain immutable leaves in `SemanticDb`.

## 5. Observed behavior

After the initial snapshot identity assertions pass, the first local update panics while publishing a declaration-shell query. The immutable-query assertion is reached through `SemanticDb::set_state`; no local update result is returned.

The assertion does not report the `QueryKey` or distinguish which `query_declaration_shell` failure path called `query_failure`. Therefore the exact rejected declaration and immediate failure reason remain unknown.

## 6. Minimal reproduction

```sh
RUST_BACKTRACE=1 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic ordinary_image_session_keeps_universe_external_across_replacement -- --nocapture
```

Observed result:

```text
thread panic at phalcom-semantic/src/db/mod.rs:454:
cannot mutate an immutable canonical query

SemanticDb::set_state
  -> db::query::query_failure
  -> db::query::query_declaration_shell
  -> SemanticWorkspaceSession::update_candidate_with_budget_and_cancel_and_delta
  -> SemanticWorkspaceSession::update_with_budget_and_cancel_and_delta
  -> SemanticWorkspaceSession::update_with_delta
  -> SemanticWorkspaceSession::update_module_workspace
  -> SemanticWorkspaceSession::apply_module_mutations
  -> ordinary_image_session_keeps_universe_external_across_replacement
```

Expected: each local source update succeeds, local queries publish in the session, and canonical roots remain immutable and shared.

## 7. Relevant code path

```text
SemanticWorkspaceSession::update_candidate_with_budget_and_cancel_and_delta
  -> publishes/revalidates base declaration shells with query_declaration_shell
  -> query_declaration_shell reaches query_failure for a canonical QueryKey
  -> query_failure calls SemanticDb::set_state
  -> set_state rejects mutation because canonical_entry(key) exists
```

`query_declaration_shell` first calls `validate_reuse`. If that returns false, its subsequent failure paths can call `query_failure`; `set_state` correctly asserts that a canonical key cannot be mutated. The backtrace does not establish whether reuse failed because of an input-fingerprint mismatch, alias dependency state, or another query failure.

## 8. Scoped implementation diff

- Core unit-test fixtures were migrated to the current materialization, registry, and semantic-lowering APIs so the existing failpoint test can compile.
- The semantic integration target's stale fixture representations and constructors were migrated until the target compiled.
- The canonical-session test now checks the current `from_image` snapshot identity contract.
- No production code was changed for this incident.

## 9. Evidence

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --test semantic --message-format=short` completed successfully after fixture migration.
- The core failpoint test `loaded_artifact_prepublication_failures_are_retryable` passed all four boundaries: 1 passed, 0 failed.
- The session regression fails during its first local update at the `SemanticDb::set_state` canonical immutability assertion.
- `SemanticDb::validate_reuse` returns false when the canonical input fingerprint does not match; canonical dependency lookup is also a separate possible failure path.
- `query_declaration_shell` calls `query_failure` on alias-dependency and publication failures. `query_failure` attempts to store failed query state, which is forbidden for a canonical key.
- The session update path iterates `base_declarations` and invokes `query_declaration_shell`, so canonical base-shell reuse is part of the ordinary update path.

## 10. Implementer's causal hypothesis

The image-backed update asks the ordinary declaration-shell query path to process a base declaration whose `QueryKey` is already present in the canonical image. The canonical entry is not accepted as reusable, or its declaration-shell failure path is reached for another reason. The failure handler then tries to record local failed state for the immutable key and panics. It is not yet established whether the primary defect is imported query metadata parity or session bootstrap reprocessing.

## 11. Alternative hypotheses

1. The imported declaration-shell input fingerprint or dependency closure differs from the current `query_declaration_shell` contract, so `validate_reuse` misses a valid image product.
2. `from_image` sessions should skip bootstrap publication for image-owned declaration shells and consume the loaded snapshot/database products directly.
3. A canonical dependency is absent or not visible as current during alias-shell publication; the error path is valid, but it must return a non-mutating failure for immutable keys.

These alternatives imply different changes to importer validation, session ownership, and query failure handling.

## 12. Attempts already made

- Migrated the semantic test target's stale API fixtures; compiler diagnostics fell from 451 to 0.
- Updated the test's obsolete expectation that `from_image` starts without snapshots; the current plan and constructor require both initial snapshots to share the image snapshot.
- Re-ran the focused session regression. It advanced past those assertions and reproduced the immutable-query panic. A backtrace confirmed the query/session path above.
- No production correction was attempted.

## 13. Why local authority is insufficient

Choosing between query-root fingerprint/authority reconciliation and changing which layer owns bootstrap publication affects semantic authority and the ordinary session contract. A fallback that writes failed local state for a canonical key would violate the immutable-root invariant. The current evidence does not distinguish the primary cause, so a local workaround would risk masking a T2/T4 query-image defect or introducing a second publication path.

## 14. Decision requested

Which boundary should own the correction: should R4 reconcile imported declaration-shell fingerprints/dependencies so canonical reuse succeeds, or should `from_image` avoid re-running bootstrap publication for canonical base shells? Please also specify the fail-closed behavior when an immutable canonical key is not reusable, without mutating the image or recording duplicate local state.
