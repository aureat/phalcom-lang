---
id: ARTF001.C1.P1
kind: baseline-failures
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINED
---

# ARTF001.C1.P1 — Baseline Failure Ledger

## Baseline Verification Context

- Working directory: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact`
- Branch: `feat/artifact`
- Base revision: `39334961e61cc3b9e251d03bcd348bceb50abbc6` (`origin/main`)
- Toolchain: `rustc 1.99.0-nightly (af3d95584 2026-07-09)`
- User instruction: "Ignore test baseline." Broad workspace test execution and comprehensive failure resolution were bypassed per direct user instruction. Focused discriminator checks executed prior to this instruction are recorded below.

## Gate Results

| Command | Result | Notes |
|---|---|---|
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo fmt --all -- --check` | **PASS** | 0 formatting diffs across workspace |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets` | **PASS** | Clean build across all crates and targets |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules` | **FAIL** (7 pass, 2 fail) | Failures in `builtin_catalog.rs` |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib` | **FAIL** (191 pass, 1 fail) | Failure in `context.rs` tracking resolver test |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic` | **TIMEOUT / FAIL** | Monolithic suite (2259 tests); 3 failures observed before timeout: `export_invalidation`, `existentials`, `integration_matrix` |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib` | **PASS** (211 pass, 0 fail) | Completed in 144.39s |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace` | **SKIPPED** | Skipped per direct user instruction ("Ignore test baseline.") |

## Failure Classification Ledger

| ID | Command / Test | Observed Failure | Reproduces on Clean Main? | Owner | Classification | Blocks P2? | Recommended Action |
|---|---|---|---|---|---|---|---|
| `BF-01` | `cargo test -p phalcom-modules --test builtin_catalog bcat_02_reflection_exposed_children_in_nodes` | `assertion left == right failed: expected 22 expose declarations in reflection/package.ph; left: 23, right: 22` | Yes (`39334961`) | `phalcom-modules` | `BASELINE_CURRENT_MAIN` / `STALE_FIXTURE` | No | Update test fixture expectation from 22 to 23 reflecting added reflection module expose. |
| `BF-02` | `cargo test -p phalcom-modules --test builtin_catalog bcat_08_root_alias_does_not_create_synthetic_declaration` | `UnresolvedImport { module: collections.list, path: "universe.collections.iterator_laws" }` | Yes (`39334961`) | `phalcom-modules` | `BASELINE_CURRENT_MAIN` / `STALE_FIXTURE` | No | Pre-existing import resolution drift in legacy list module; non-blocking for artifact container. |
| `BF-03` | `cargo test -p phalcom-semantic --lib checker::context::tests::tracking_resolver_demands_the_exact_unready_type_member_owner` | Panic at `checker/context.rs:4498:13: an unready product must not be fetched from the stale resolver view` | Yes (`39334961`) | `phalcom-semantic` | `BASELINE_CURRENT_MAIN` | No | Semantic resolver assertion in tracking resolver unit test; does not affect container codec. |
| `BF-04` | `cargo test -p phalcom-semantic --test semantic export_invalidation::test_previously_missing_public_name_appears_invalidates_consumer` | Panic at `export_invalidation.rs:64:5: consumer body must recompute when previously missing name appears, stats: ... callable_bodies_recomputed: 1` | Yes (`39334961`) | `phalcom-semantic` | `BASELINE_CURRENT_MAIN` | No | Invalidation stat expectation mismatch on incremental update; does not affect cold artifact loading. |
| `BF-05` | `cargo test -p phalcom-semantic --test semantic semantic::adts::existentials::structural_wrapper_cannot_hide_local_existential_escape` | `assertion left == right failed: unexpected type.existential.escape diagnostics; left: 0, right: 1` | Yes (`39334961`) | `phalcom-semantic` | `BASELINE_CURRENT_MAIN` | No | Diagnostic assertion fixture drift. |
| `BF-06` | `cargo test -p phalcom-semantic --test semantic semantic::advanced::integration_matrix::test_matrix_1_record_row_subtyping_and_unification` | Panic at `integration_matrix.rs:61:5: Record width subtyping must succeed for read-only fields` | Yes (`39334961`) | `phalcom-semantic` | `BASELINE_CURRENT_MAIN` | No | Record row width subtyping edge test on advanced semantic integration matrix. |

## Conclusion

None of the pre-existing failures in `BF-01` through `BF-06` touch or block the upcoming P2 tasks (creation of `phalcom-compiler-artifact` container codec and `phalcom-type-meta` allocation budgeting). All pre-existing failures are recorded as baseline drift on `origin/main` and are excluded from consolidation scope.
