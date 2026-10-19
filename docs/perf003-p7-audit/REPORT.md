# PERF003.C2.P7 independent certification report

## Decision

**R.G6: PASS. R.G7: PASS. P7 is accepted and P8 takeover is unblocked.**

P7 implementation remains `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`. This reconciliation accepts its independent certification on the evidence below. The broader workspace commands still stop at 412 inherited semantic-fixture API errors; those failures are recorded and do not invalidate the focused P7 gates.

## Provenance

| Item | Value |
|---|---|
| Branch | `perf/universe-characterization` |
| Measured implementation revision | `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2` |
| R.G7 regression-test revision | `c1524361b9606dc537671997fe5069f2aa66035b` |
| Evidence publication revision | `99768ecfed6b5d0f33ac9b2a6239a5af07aeec96` |
| Toolchain | `rustc 1.99.0-nightly (af3d95584 2026-07-09)`, `cargo 1.99.0-nightly (59800466c 2026-07-07)` |
| Platform | Darwin 25.5.0 arm64, Apple M1 Pro |
| Profile | Release: opt-level 3, LTO enabled, one codegen unit, panic abort |

The benchmark harness correction was committed as `317992000e358bc54ef0ba2446df3ffdc0be0abd`. The final measured implementation revision is `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2`. The certification test additions are test-only and are committed at `c1524361b9606dc537671997fe5069f2aa66035b`.

## Artifact identity

The release audit binary generated `target/telos/universe/universe.telosc` at the measured implementation revision.

| Property | Value |
|---|---:|
| Container bytes | 15,936,496 |
| BLAKE3-256 | `b91ccc4e6011c2789c6e20861e6d6d98908074bb45461d54b5bd8885ca9665bf` |
| MODULE_WORLD | 156,313 bytes |
| SEMANTIC_WORLD | 12,945,704 bytes |
| QUERY_IMAGE | 1,338,303 bytes |
| METADATA | 1,495,622 bytes |
| Header, directory, and container overhead | 554 bytes |
| Exact whole-load allocation budget N | 373,069,634 bytes |
| Source-free complete snapshot | true |
| Same-image serialization determinism | true |

The determinism result applies to repeated serialization of the same image. This report makes no cross-process full-build determinism claim. Section digests and source, interface, semantic, query, and metadata digests are in [ARTIFACT.json](ARTIFACT.json).

## R.G6 — measurement evidence integrity

**PASS.** Cold and warm rows are machine-readable, sample counts reconcile with process identities, and all reported timing statistics below were recomputed from the committed CSVs. Percentiles use nearest rank, `ceil(0.95 × n)`; standard deviation is population standard deviation.

### Cold compilation

Each workload and mode has 10 fresh child processes. All 80 rows are `SUCCESS`, with 80 distinct process IDs. Total is end-to-end process duration. RSS is the process high-water mark reported in KB.

| Workload | Mode | n / success | Total min / mean / median / p95 / max / SD (ms) | Init median (ms) | Compile median (ms) | Peak RSS median / max (KB) |
|---|---|---:|---|---:|---:|---:|
| `print42` | source | 10 / 10 | 9249.350 / 9524.459 / 9535.315 / 10169.390 / 10169.390 / 268.387 | 9528.081 | 6.928 | 809208 / 816960 |
| `print42` | artifact | 10 / 10 | 310.304 / 316.511 / 314.915 / 331.475 / 331.475 / 6.065 | 287.226 | 6.134 | 324736 / 326320 |
| `option_string` | source | 10 / 10 | 9157.014 / 9409.440 / 9273.362 / 10076.836 / 10076.836 / 303.983 | 9266.172 | 6.864 | 809240 / 813088 |
| `option_string` | artifact | 10 / 10 | 307.176 / 310.165 / 309.579 / 313.476 / 313.476 / 2.231 | 282.064 | 6.290 | 325880 / 327568 |
| `synthetic_pkg` | source | 10 / 10 | 9114.823 / 9183.458 / 9162.568 / 9381.982 / 9381.982 / 77.665 | 9154.067 | 7.422 | 809368 / 811440 |
| `synthetic_pkg` | artifact | 10 / 10 | 308.204 / 312.154 / 310.480 / 323.807 / 323.807 / 4.355 | 282.537 | 6.678 | 326032 / 327424 |
| `slotmap` | source | 10 / 10 | 9399.423 / 9444.685 / 9447.540 / 9492.364 / 9492.364 / 28.846 | 9150.249 | 297.468 | 822416 / 824768 |
| `slotmap` | artifact | 10 / 10 | 613.116 / 623.296 / 624.601 / 634.836 / 634.836 / 5.590 | 286.001 | 313.245 | 349912 / 356496 |

Using total-duration medians, artifact-backed cold compilation is 30.28× faster for `print42`, 29.95× for `option_string`, 29.51× for `synthetic_pkg`, and 15.13× for `slotmap`. These are cold end-to-end process comparisons; they do not describe warm compiler execution or VM runtime.

### Warm compilation

Source and artifact modes ran in separate child processes, PIDs `14176` and `14200`. Each process performed one untimed warmup followed by 20 timed compilations of the same workload shape. The timed inputs alternated two string-literal variants: 10 `modified_input` and 9 `unchanged_input` rows after the first compilation in each mode. All 40 rows succeeded. RSS is reported as each isolated process's peak RSS.

| Mode | Rows / success | Min / mean / median / p95 / max / SD (ms) | Peak RSS (bytes) |
|---|---:|---|---:|
| source | 20 / 20 | 4.990 / 5.315 / 5.151 / 5.808 / 6.025 / 0.320 | 828,112,896 |
| artifact | 20 / 20 | 4.905 / 5.162 / 5.081 / 5.552 / 5.589 / 0.223 | 334,823,424 |

The reported RSS is a process high-water mark. It does not establish retained memory or leak behavior.

### Runtime attempts

There are 10 fresh processes per mode and 20 distinct PIDs. All rows are recorded as `RUNTIME_ERROR`; none is counted as a successful runtime execution. The error-detail field is empty in the current harness, so the root runtime error is not identified by this bundle. These failed attempts are not used for a runtime speedup claim.

| Mode | Rows / runtime errors | Total min / mean / median / p95 / max / SD (ms) | Peak RSS max (KB) |
|---|---:|---|---:|
| source | 10 / 10 | 9242.122 / 9316.319 / 9310.778 / 9417.226 / 9417.226 / 51.473 | 828400 |
| artifact | 10 / 10 | 9539.059 / 9617.238 / 9640.322 / 9685.476 / 9685.476 / 52.388 | 1,037,088 |

`VM bootstrap` and the runtime attempt are separate columns in the CSV. The observed status is `RUNTIME_ERROR`; this report does not infer an underlying cause from the empty detail field.

### Package parity and allocation boundary

The source-backed and artifact-backed `packages/slotmap` comparison completed with 4 modules each; the runtime package comparison completed with 8 modules each. Both structural comparisons passed. The whole-load allocation discriminator measured N as 373,069,634 bytes: N succeeded, N−1 returned the typed budget error, and retry at N succeeded. The source-denial check observed zero retained sources, a complete snapshot, and successful user-package compilation. Logs and the supplementary SlotMap CSV are under this directory.

### R.G6 checklist

- CSV structure, sample/process counts, workload labels, and outcomes reconcile.
- No cold compile failure is counted as success; the 20 runtime errors remain failures.
- The artifact digest is labeled BLAKE3-256.
- Statistics are recomputed from raw rows using the stated conventions.
- Warm source and artifact scenarios are isolated by process.
- Evidence names the measured implementation revision and environment.
- Leak and retained-memory claims remain unverified.

## R.G7 — regression and corruption matrix

**PASS.** The [R.G6/R.G7 certification matrix](R.G6-R.G7-CERTIFICATION.md) lists the required cases, exact tests, tested revisions, results, and saved logs. Newly added discriminators cover omitted required sections, whole-container digest mismatch, malformed or duplicate module/import identities, duplicate relocated type identity, and duplicate static pack labels.

The mandatory focused suites passed. The two requested workspace commands fail before test execution because the monolithic semantic integration target has 412 stale API fixture errors (`BTreeMap` fixtures where current APIs require `AnalysisMap` and `ModuleAuthorityMap`). The same failure signature was recorded at clean predecessor `08e2b2f5`; it is inherited baseline drift, not a P7 regression. The failed broad logs are retained under `logs/`.

## Limitations and deferred failures

- All 20 runtime-suite attempts are `RUNTIME_ERROR`; their output field is blank. Runtime execution performance and the precise error cause are not certified.
- `ru_maxrss` is peak RSS, not retained memory. No leak test was run.
- `cargo test --workspace --no-run` and `cargo test --workspace` remain blocked by the inherited 412 fixture compile errors. No unrelated fixture migration was performed.

## Reproduction

Exact benchmark and regression commands are recorded in [COMMANDS.md](COMMANDS.md). Raw source rows are [RAW-COLD.csv](RAW-COLD.csv), [RAW-WARM.csv](RAW-WARM.csv), and [RAW-RUNTIME.csv](RAW-RUNTIME.csv). Test results and log paths are in [RAW-TESTING.csv](RAW-TESTING.csv).
