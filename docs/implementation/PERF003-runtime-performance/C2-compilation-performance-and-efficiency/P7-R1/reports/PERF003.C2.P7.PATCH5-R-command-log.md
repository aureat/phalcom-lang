# PERF003.C2.P7.PATCH5-R baseline command log

All commands ran from
`/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization`
on revision `9cbba391a4482052dec476b327619c957a028b19`, unless noted.

## Repository and environment

| Command | Result |
|---|---|
| `git fetch origin` | Exit 0. |
| `git rev-parse HEAD` | Exit 0; `9cbba391a4482052dec476b327619c957a028b19`. |
| `git rev-parse origin/perf/universe-characterization` | Exit 0; same revision. |
| `git status --short --branch` | Exit 0; branch matched, only the supplied plan was untracked before checkpoint updates. |
| `rustc --version` | Exit 0; `rustc 1.99.0-nightly (af3d95584 2026-07-09)`. |
| `cargo --version` | Exit 0; `cargo 1.99.0-nightly (59800466c 2026-07-07)`. |
| `uname -s -r -m` | Exit 0; `Darwin 25.5.0 arm64`. |

## R.T0.2 build and test baseline

| Command | Result |
|---|---|
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --lib -p phalcom-type-meta -p phalcom-modules -p phalcom-semantic -p phalcom-core` | Exit 0. |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --no-run` | Exit 101. `phalcom-core/examples/perf_workloads.rs:132` fails with E0601 (`main` function not found). |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta --test allocation_helpers` | Exit 0; 1 passed. |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test topology --test query` | Exit 0; query 5/5 and topology 7/7 passed. |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test artifact-allocation -- --nocapture` | Exit 0; 1 passed. |
| `STRICT_ALLOCATION_GATE=1 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact_allocation -- --nocapture --test-threads=1` | Exit 101 before test selection: Cargo reported the registered target name is `artifact-allocation`. |
| `STRICT_ALLOCATION_GATE=1 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1` | Exit 101; 1 test failed. Ledger `323324452`, independent allocator `528785777`, delta `205461325`. |

## R.T0.3 artifact and compiler behavior

| Command | Result |
|---|---|
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-core --example perf_audit_p7 -- generate --out target/telos/universe/universe.telosc --json target/perf003-p7-audit/ARTIFACT.json` | Exit 0. Generated 15,775,219 bytes; same-process double-build determinism check passed; loaded image reported source-free complete. |
| `target/debug/examples/perf_audit_p7 cold-single --mode source --workload print42 --sample-id 0 --artifact target/telos/universe/universe.telosc` | Exit 0; `SUCCESS`; one debug sample. |
| `target/debug/examples/perf_audit_p7 cold-single --mode artifact --workload print42 --sample-id 0 --artifact target/telos/universe/universe.telosc` | Exit 0; `SUCCESS`; one debug sample. |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-core --bin phalcom -- packages/slotmap` | Program exit 65; parser reports `Expected ")"` at `packages/slotmap/src/main.ph:27:12`. |
| `target/debug/examples/perf_audit_p7 slotmap-eval --artifact target/telos/universe/universe.telosc` | Exit 0 but not a successful compilation test. Root project returns the same missing-`package.ph` error in both modes because the harness passes `EntrySelection::Package`; runtime project returns the same non-canonical `basic_test.ph` error. The command then prints `100% parity`. |
| `target/debug/examples/perf_audit_p7 source-denial-check --artifact target/telos/universe/universe.telosc` | Exit 0; empty sources, complete snapshot, and synthetic package compile asserted. No filesystem denial. |
| `target/debug/examples/perf_audit_p7 allocation-check --artifact target/telos/universe/universe.telosc` | Exit 0; ledger-derived `N = 323324452`, `N-1` rejected, `N` accepted. The check matches any error at `N-1`; it is not an independent allocator proof. |
| `target/debug/examples/perf_audit_p7 runtime-single --mode artifact --artifact target/telos/universe/universe.telosc --sample-id 0` | Exit 0 with status `BASELINE_BOOTSTRAP_ERROR`; VM reports missing canonical facade binding for `ProductKind`. P8 boundary. |

The test target spelling correction follows Cargo's registered target name. No
assertion or limit was changed during baseline capture. The generated artifact,
JSON, and benchmark packages are under ignored `target/` paths.

## R.T1 focused progress after baseline

These commands ran from the same worktree after implementation edits on the
uncommitted PATCH5-R worktree based on `9cbba391a4482052dec476b327619c957a028b19`.
They are implementation evidence, not a replacement for the R.T0 baseline.

| Command | Result |
|---|---|
| `RUSTFLAGS='' cargo check -p phalcom-semantic --lib` | Exit 0. Semantic library check passed after budget-aware import changes. |
| `PHALCOM_UNIVERSE_TELOSC_FOR_TEST=/tmp/perf003-patch5r-universe.telosc RUSTFLAGS='' cargo test -p phalcom-core --test artifact-allocation full_universe_telosc_load_allocation_equality_and_n_minus_one -- --nocapture` | Exit 101. Type graph import equality was `765031/765031`; semantic import observed `266069278` bytes while the ledger charged `263600723`, delta `2468555`. Exact semantic equality failed. Output: `/tmp/perf003-patch5r-import-check9.log`. |
| `git diff --check` | Exit 0 after lifecycle-record updates. |

R.T1 remains partial and R.G1 fails the exact semantic import equality condition.
The sampled allocator backtraces were removed: their stack-frame heuristic could
not distinguish precharged from uncharged requests and is not retained as
evidence. The cached artifact path hooks remain in the focused test.
