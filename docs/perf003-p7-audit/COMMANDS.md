# PERF003.C2.P7 commands and evidence map

## Measurement revision

All cold, warm, runtime, SlotMap, allocation, and source-denial measurements were generated with the release audit binary built from measured implementation revision `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2` using the environment in [ENVIRONMENT.md](ENVIRONMENT.md). The later `c1524361b9606dc537671997fe5069f2aa66035b` commit adds regression tests only; it does not alter the benchmark harness or measured implementation.

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --release -p phalcom-core --example perf_audit_p7
./target/release/examples/perf_audit_p7 generate --out target/telos/universe/universe.telosc --json target/perf003-p7-audit/ARTIFACT.json
./target/release/examples/perf_audit_p7 run-cold-suite --artifact target/telos/universe/universe.telosc --samples 10 --csv target/perf003-p7-audit/RAW-COLD.csv
./target/release/examples/perf_audit_p7 run-warm-suite --artifact target/telos/universe/universe.telosc --repetitions 20 --csv target/perf003-p7-audit/RAW-WARM.csv
./target/release/examples/perf_audit_p7 run-runtime-suite --artifact target/telos/universe/universe.telosc --samples 10 --csv target/perf003-p7-audit/RAW-RUNTIME.csv
./target/release/examples/perf_audit_p7 slotmap-eval --artifact target/telos/universe/universe.telosc --csv target/perf003-p7-audit/RAW-SLOTMAP.csv
./target/release/examples/perf_audit_p7 allocation-check --artifact target/telos/universe/universe.telosc
./target/release/examples/perf_audit_p7 source-denial-check --artifact target/telos/universe/universe.telosc
```

The warm command starts one child for source mode and one for artifact mode. Each child performs one untimed warmup and 20 timed repetitions. See `logs/warm-suite.txt` for the two PIDs and summaries.

## Focused R.G7 regression commands

The listed focused suites passed. `RUSTFLAGS='' RUSTC_WRAPPER=''` was used for Cargo validation commands unless stated otherwise.

```sh
cargo check --lib -p phalcom-type-meta -p phalcom-modules -p phalcom-semantic -p phalcom-core
cargo test -p phalcom-compiler-artifact
cargo test -p phalcom-modules --test topology --test query
cargo test -p phalcom-semantic --test artifact-allocation -- --nocapture
cargo test -p phalcom-semantic --lib artifact::
cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1
cargo test -p phalcom-core --test core modules_canonical_artifact::
cargo test -p phalcom-semantic --test session-indexer-match -- --nocapture
cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime
cargo test -p phalcom-core --lib loaded_artifact_prepublication_failures_are_retryable -- --nocapture
cargo test -p phalcom-semantic --lib canonical_image_ -- --nocapture
cargo test -p phalcom-semantic --lib query_persistence_classification_is_fail_closed_for_nonstructural_products -- --nocapture
```

Test-only R.G7 additions, tested at `c1524361b9606dc537671997fe5069f2aa66035b`:

```sh
cargo test -p phalcom-compiler-artifact
cargo test -p phalcom-modules --lib module_world_rejects_invalid_and_duplicate_module_import_identities -- --nocapture
cargo test -p phalcom-semantic --lib type_graph_import_rejects_duplicate_relocated_type_identity -- --nocapture
```

`cargo fmt --all -- --check` and `git diff --check` passed after the test additions.

## Required broad commands and classification

```sh
cargo test --workspace --no-run
cargo test --workspace
```

Both commands fail at the same 412 stale semantic fixture/API errors. See `logs/workspace-no-run.txt` and `logs/workspace-test.txt`. The predecessor comparison at `08e2b2f5` is recorded in the PERF003.C2 checkpoint; classification is inherited baseline drift.

## Saved outputs

- Benchmark summaries: `logs/cold-suite.txt`, `logs/warm-suite.txt`, `logs/runtime-suite.txt`.
- SlotMap parity: `logs/slotmap-eval.txt`, `RAW-SLOTMAP.csv`.
- Allocation and source denial: `logs/allocation-check.txt`, `logs/source-denial-check.txt`.
- Test commands and results: [RAW-TESTING.csv](RAW-TESTING.csv), with detailed outputs under `logs/`.
- Artifact identity: [ARTIFACT.json](ARTIFACT.json).
