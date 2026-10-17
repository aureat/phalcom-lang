# Reproducible Audit & Benchmark Commands

All commands are executed from the worktree root:
`/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization`

## 1. Environment & Revision Verification

```sh
# Verify branch and tree cleanliness
git status
git log -1 --format="%H %s"

# Verify pinned toolchain
cargo --version
rustc --version
```

## 2. Compile Release Benchmark Harness

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --release -p phalcom-core --example perf_audit_p7
```

## 3. Generate Persistent Universe Artifact (`universe.telosc`)

```sh
# Generates universe.telosc, verifies byte-for-byte determinism, writes ARTIFACT.json
./target/release/examples/perf_audit_p7 generate \
  --out target/telos/universe/universe.telosc \
  --json target/perf003-p7-audit/ARTIFACT.json
```

## 4. Fresh-Process Cold Compilation Suite (80 Fresh Processes)

```sh
# Runs 10 fresh OS process samples for each of 4 workloads under source and artifact
./target/release/examples/perf_audit_p7 run-cold-suite \
  --artifact target/telos/universe/universe.telosc \
  --samples 10 \
  --csv target/perf003-p7-audit/RAW-COLD.csv
```

## 5. Warm In-Memory Compilation Benchmark (20 Repetitions)

```sh
# Measures 20 repetitions (first, unchanged, modified) with resident compiler image
./target/release/examples/perf_audit_p7 run-warm-suite \
  --artifact target/telos/universe/universe.telosc \
  --repetitions 20 \
  --csv target/perf003-p7-audit/RAW-WARM.csv
```

## 6. End-to-End Runtime & VM Bootstrap Suite (20 Fresh Processes)

```sh
# Compiles minimal package and attempts VM bootstrap and execution
./target/release/examples/perf_audit_p7 run-runtime-suite \
  --artifact target/telos/universe/universe.telosc \
  --samples 10 \
  --csv target/perf003-p7-audit/RAW-RUNTIME.csv
```

## 7. Real Package Evaluation (`packages/slotmap`)

```sh
# Tests slotmap and runtime tests under source vs artifact compilation
./target/release/examples/perf_audit_p7 slotmap-eval \
  --artifact target/telos/universe/universe.telosc
```

## 8. Exact Allocation Budget & N-1 Rejection Check

```sh
# Performs binary search for exact minimum transaction budget and checks N-1 fail-closed
./target/release/examples/perf_audit_p7 allocation-check \
  --artifact target/telos/universe/universe.telosc
```

## 9. Source Denial Verification

```sh
# Verifies loaded compiler image has 0 sources and complete authority
./target/release/examples/perf_audit_p7 source-denial-check \
  --artifact target/telos/universe/universe.telosc
```

## 10. Workspace Integration Test Lanes

```sh
# 1. Compiler artifact codec & container validation (59 tests)
RUSTFLAGS='' cargo test -p phalcom-compiler-artifact

# 2. Semantic artifact export/import & proof relocation (43 tests)
RUSTFLAGS='' cargo test -p phalcom-semantic --lib artifact::

# 3. Canonical artifact vertical loader & closure suite (11 tests)
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_canonical_artifact::
```
