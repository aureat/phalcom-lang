# Step 1 — Semantic Query Validity Verification

## Baseline

- Repository: `aureat/phalcom-lang`
- Uploaded source baseline: `/mnt/data/phalcom-main`
- Inspected GitHub `main`: `3ff22158a60db3a323a4a64e8b6ab3957de02408`
- Isolated edited tree: `/mnt/data/phalcom-step1`

The uploaded archive has no `.git` directory, so the implementation was isolated by copying the uploaded baseline rather than creating a Git worktree.

## Scope implemented

1. Separate query computation revision from current-validation revision.
2. Reject transitive reuse through a dependency that is merely old-`Ready` but not validated in the current semantic revision.
3. Advance validation on cache reuse without rewriting the original computation revision.
4. Require dependency products to be current-validated before recording an edge.
5. Fail staged queries closed when a required dependency edge cannot be recorded.
6. Stop silently discarding staged-query publication errors.
7. Record constructor body dependency against the class-side constructor signature actually consumed by body checking.
8. Repair the retained-snapshot `TypeStore` regression so it reads a `TypeId` that existed in revision 1.

## Files changed

- `phalcom-semantic/src/checker/body.rs`
- `phalcom-semantic/src/db/mod.rs`
- `phalcom-semantic/src/db/query.rs`
- `phalcom-semantic/src/db/state.rs`
- `phalcom-semantic/tests/callable_dependency_invalidation.rs`
- `phalcom-semantic/tests/db.rs`
- `phalcom-semantic/tests/semantic_db_incremental.rs`
- `phalcom-semantic/tests/type_store_revisions.rs`

No fingerprint redesign, module-lifecycle inversion, LSP changes, or fine-grained invalidation rewrite is included in this patch.

## New/changed behavioral regressions

- `test_generic_reuse_validation_matrix`
  - stale middle query now prevents root reuse;
  - a semantically stable middle product allows root reuse after current-revision validation;
  - reuse advances `validated_revision` while preserving the original computation `revision`.
- `record_dependency_rejects_ready_but_unvalidated_dependency`
  - old `Ready` is insufficient for dependency recording;
  - revalidation makes it recordable without recomputation.
- `callable_body_query_fails_closed_when_consumed_signature_product_is_missing`
  - a body consuming a published surface signature cannot publish `CallableBody` if the matching `CallableSignature` DB product is absent.
- `case_g_constructor_body_depends_on_class_side_constructor_signature`
  - constructor body query depends on `CallableSignature(..., DispatchSide::Class)`, not its synthetic instance-side body identity.
- `retained_old_snapshot_preserves_type_denotation_after_later_revisions`
  - captures a revision-1 declaration form from `snapshot1` and reads the exact denotation from that retained snapshot after later revisions.

## Verification actually executed in this environment

### Patch round trip

The generated patch was dry-run against a fresh copy of the untouched uploaded baseline, then applied. Every touched file was byte-compared with the isolated working tree.

Result:

```text
PATCH_ROUNDTRIP_OK
GIT_APPLY_CHECK=0
```

### Static invariant checks

Result:

```text
PASS - Ready tracks validated_revision
PASS - reuse requires dependency current validation
PASS - reuse can advance validation without recompute
PASS - record_dependency rejects old Ready
PASS - query layer has no ignored dependency recording
PASS - query layer has no ignored publication
PASS - query layer uses validating reuse
PASS - constructor body records consumed signature id
PASS - constructor regression checks class-side signature
PASS - transitive stale regression rejects root
PASS - TypeStore regression reads old snapshot denotation
```

### Diff whitespace check

`git diff --no-index --check` produced no whitespace diagnostics for any of the eight touched files.

## Rust execution status

The sandbox does not contain a Rust toolchain. Fresh execution attempt:

```text
$ cargo test -p phalcom-semantic --test semantic_db_incremental --test db --test callable_dependency_invalidation --test type_store_revisions
bash: cargo: command not found
CARGO_EXIT=127
```

The repository pins:

```toml
[toolchain]
channel = "nightly-2026-07-10"
```

Network/package provisioning was unavailable in the sandbox, so this patch is **not compile/test verified here**. Static verification and patch-application verification are complete; Rust verification must be run in an environment with the pinned toolchain.

## Required verification after applying

Run from the repository root:

```bash
cargo fmt --check
cargo test -p phalcom-semantic --test semantic_db_incremental
cargo test -p phalcom-semantic --test db
cargo test -p phalcom-semantic --test callable_dependency_invalidation
cargo test -p phalcom-semantic --test type_store_revisions
cargo test -p phalcom-semantic
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

If any focused test fails, do not proceed to Step 2 until the failure is investigated against the Step 1 invariants above.
