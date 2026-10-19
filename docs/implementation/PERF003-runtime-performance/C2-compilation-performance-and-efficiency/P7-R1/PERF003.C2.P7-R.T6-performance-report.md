# PERF003.C2.P7 — R.T6 Performance & Verification Report

## 1. Executive Summary

- **Gate**: R.G6 (Harness integrity, measurement boundary correctness, evidence certification)
- **Status**: PASS
- **Target Branch**: `perf/universe-characterization`
- **Starting Baseline Commit**: `fb49f61c`
- **Artifact Identifier**: `universe.telosc` (15,785,886 bytes)
- **Digest Algorithm**: BLAKE3-256 (`85ffcea0d504f88e20fc329a17752e0e5f9d5062a2b82ead377ddcf69546f5dc`)
- **Cumulative Load Allocation**: Exact 366,789,583 bytes (signed delta: 0)

---

## 2. Environment & Test Platform

- **Platform**: Darwin 25.1.0 (macOS, Apple Silicon ARM64)
- **Toolchain**: `rustc 1.90.0-nightly (2026-07-10)`
- **Build Profile**: `--release` (opt-level 3, LTO enabled, codegen-units 1, panic abort)
- **Binary**: `target/release/examples/perf_audit_p7`
- **Sample Counts**: 20 fresh OS processes per cold scenario/workload; 10 repetitions per warm scenario; 10 fresh processes per runtime scenario.

---

## 3. Artifact Metrics & Integrity

- **Artifact Container**: 15,785,886 bytes
- **Section Breakdown**:
  - `MODULE_WORLD`: 156,313 bytes
  - `SEMANTIC_WORLD`: 12,799,554 bytes
  - `QUERY_IMAGE`: 1,339,575 bytes
  - `METADATA`: 1,489,890 bytes
- **BLAKE3 Digests**:
  - Artifact: `85ffcea0d504f88e20fc329a17752e0e5f9d5062a2b82ead377ddcf69546f5dc`
  - Source: `6fa261187963d22e1e1adf4ee2d92f78584ff8d61c1d187c583f2c98d5c47f18`
  - Interface: `23dd1128a8fc283e5626ddbf066d33c403cb5b2f08c937ce6c7ebe2303039d83`
  - Semantic: `96cf3f84911995effd6fa41952517962037d6fcf8420ec3346a60342b9c7bd2b`
  - Query: `51e4e98cfcfd3e8d30672fa238a50a02f82e2446d0266d4c19cebc9c22bcd979`
  - Metadata: `7ffa54db5d017c0ce883caab2eef16240f18976c9105229ff50e7997a32e7099`
- **Determinism**: Same-image serialization produces byte-for-byte identical output.
- **Source-Free Completeness**: Validated (`loaded.formal_image.sources().is_empty()` and `snapshot.is_source_free_artifact_complete()` both true).

---

## 4. Cold Compilation Benchmark (Fresh OS Processes, N=20 each)

| Workload | Mode | Samples | Total Median (ms) | Init Median (ms) | Compile Median (ms) | Peak RSS Median (KB) | Speedup (Total) |
|---|---|---:|---:|---:|---:|---:|---:|
| `print42` | source | 20 | 11,337.20 | 11,329.35 | 7.82 | 800,048 | 1.0× |
| `print42` | artifact | 20 | 335.91 | 304.12 | 7.11 | 319,856 | **33.8×** |
| `option_string` | source | 20 | 10,385.13 | 10,377.17 | 7.50 | 800,736 | 1.0× |
| `option_string` | artifact | 20 | 334.15 | 303.02 | 6.78 | 320,400 | **31.1×** |
| `synthetic_pkg` | source | 20 | 10,229.24 | 10,220.46 | 8.45 | 800,576 | 1.0× |
| `synthetic_pkg` | artifact | 20 | 321.43 | 290.80 | 7.33 | 320,752 | **31.8×** |
| `slotmap` | source | 20 | 10,272.01 | 9,955.84 | 312.17 | 812,888 | 1.0× |
| `slotmap` | artifact | 20 | 635.85 | 289.75 | 321.39 | 342,984 | **16.2×** |

### Key Observations:
1. **Cold Initialization Speedup**: Loading precompiled `universe.telosc` takes ~290–305 ms, compared to ~10.0–11.3 s for source-bootstrap cold parsing and analyzing 44 modules (**~34× faster cold start**).
2. **User Compilation Parity**: Compiler execution on loaded formal image is within ~1–3% of source compiler execution (e.g. `slotmap` compilation 321.39 ms vs 312.17 ms).
3. **Memory Footprint**: Fresh-process peak RSS dropped from ~800–812 MB to ~320–343 MB (**~58–60% reduction in peak resident memory**).

---

## 5. Warm Compilation Benchmark (Same Process, N=10 repetitions)

| Scenario | Mode | Repetitions | Min (ms) | Median (ms) | Mean (ms) | P95 (ms) | Max (ms) | StdDev (ms) | Peak RSS (KB) |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Warm loop | source | 10 | 6.43 | 6.77 | 6.91 | 8.27 | 8.27 | 0.54 | 803,168 |
| Warm loop | artifact | 10 | 6.27 | 6.68 | 6.85 | 7.79 | 7.79 | 0.55 | 1,058,288 |

Warm compilation throughput is virtually identical: artifact median 6.68 ms vs source median 6.77 ms (artifact is 1.3% faster).

---

## 6. Real Package Parity Evaluation (`packages/slotmap`)

Evaluated with `./target/release/examples/perf_audit_p7 slotmap-eval`:
- `packages/slotmap` (4 modules):
  - Artifact compile: 344.08 ms, 4 modules
  - Source compile: 319.93 ms, 4 modules
  - Structural parity report: 0 differences (topology, interfaces, exports, lowering, bindings match).
- `packages/slotmap/tests/runtime` (8 modules):
  - Artifact compile: 421.04 ms, 8 modules
  - Source compile: 444.55 ms, 8 modules
  - Structural parity report: 0 differences.

---

## 7. Memory & Allocation Invariant Verification (R.G4)

- **Ledger Requested**: 366,789,583 bytes
- **Independently Observed**: 366,789,583 bytes
- **Signed Delta**: 0
- **Cumulative Limit N - 1**: Rejected fail-closed with `TeloscArtifactError::BudgetExceeded { limit_bytes: 366789582 }`.
- **Exact Limit N**: Accepted.
- **Rollback & Retry**: Child transaction rollbacks validated without leakage.

---

## 8. Runtime Execution Defect Classification (Class D)

During runtime VM execution (`run-runtime-suite`), both source and artifact paths encounter the pre-existing Class D runtime bootstrap defect:
```
canonical Universe runtime bootstrap failed: internal error (this is a Phalcom bug, please report): failed to compile Universe module universe:reflection.structural: runtime intrinsic declaration 'ProductKind' has no canonical facade binding.
```
- Source compiler initialization: ~10,030 ms
- Artifact compiler initialization: ~317 ms
- Execution is appropriately isolated and documented in `artifact_backed_slotmap_runtime_tests_execute`.

---

## 9. Gate R.G6 Acceptance Decision

All criteria for Gate R.G6 are satisfied:
1. CSV rows structurally valid and RFC 4180 escaped.
2. Raw row counts reconcile with child executions (160 cold samples + 20 warm samples + 20 runtime samples).
3. No failed-compile workload reported as successful.
4. Correct BLAKE3-256 digest terminology throughout.
5. Measurement boundaries separated (file read, container validation, section decode, relocation, publication, compilation, peak RSS).
6. Statistical summaries computed from raw samples.
7. Acceptance decision: **PASS**.
