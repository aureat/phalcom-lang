# PERF003 in-memory Universe image compile characterization (Release Build)

**Date:** 2026-10-14
**Branch:** `perf/universe-characterization`
**Measured commit:** `7159ea859a5a02ce6722b0c1e37a8f1fec554bc2` (built with `--release`)
**Profile:** release (optimized, `-Ctarget-cpu=native`), pinned nightly toolchain, `test-timings` + detailed observability

## Result

Building in `release` mode provides an across-the-board **~3.1× to 3.4× speedup** over the debug profile for universe image construction and session initialization.

- Cold process session initialization dropped from **41.305 s** (debug median) to **12.198 s** (release median), representing a **3.39× speedup** (70.5% lower).
- In-memory image construction fell from **41.301 s** to **12.197 s** (**3.39× faster**), with the dominant canonical semantic update improving from **40.848 s** to **12.060 s** (**3.39× faster**).
- Warm process session initialization completed in **0.052 ms** (vs 0.067 ms debug), and the first fixture compile completed in **7.612 ms** (vs 23.556 ms debug, **3.09× faster**).
- Including session initialization, cold vs warm first fixture compile in release dropped from **12.207 s → 7.620 ms** (**~1,600× faster**).
- Warm retained-session fixture compiles completed with a median of **15.644 ms** (vs 40.878 ms debug, **2.61× faster**).
- Warmup itself (excluded from warm fixture compile) dropped from **42.152 s** to **13.259 s** (**3.18× faster**).

## Debug vs Release Comparison Summary

| Metric | Debug (`dev`) | Release (`--release`) | Speedup | Time Reduction |
|---|---:|---:|---:|---:|
| **Cold Session Init (median)** | 41,304.700 ms | 12,197.584 ms | **3.39×** | **-70.5%** |
| **Cold Image Construction Span** | 41,300.537 ms | 12,197.311 ms | **3.39×** | **-70.5%** |
| — Canonical semantic update | 40,847.909 ms | 12,059.863 ms | **3.39×** | **-70.5%** |
| — Canonical semantic baseline | 292.228 ms | 72.028 ms | **4.06×** | **-75.4%** |
| — Query freeze | 68.842 ms | 20.361 ms | **3.38×** | **-70.4%** |
| — Snapshot validation | 0.110 ms | 0.036 ms | **3.06×** | **-67.3%** |
| **Cold Fixture Compile Total** | 29.857 ms | 9.322 ms | **3.20×** | **-68.8%** |
| **Warmup (Image Preload)** | 42,152.000 ms | 13,259.150 ms | **3.18×** | **-68.5%** |
| **Warm Session Init** | 0.067 ms | 0.052 ms | **1.29×** | **-22.4%** |
| **Warm First Fixture Compile** | 23.556 ms | 7.612 ms | **3.09×** | **-67.7%** |
| **Warm Retained Fixture Compile (median)** | 40.878 ms | 15.644 ms | **2.61×** | **-61.7%** |

## Method

- **Cold process:** three separate executable processes compiled under `--release`; each created `ProgramCompilationSession::standalone()` and compiled the tiny fixture once.
- **Warm process:** one release process first preloaded the canonical image, created a compilation session, compiled the fixture once for the warm-first sample, followed by 19 retained-session fixture replacements alternating `1 + 2` and `2 + 3`.
- Reported phase sums use the internal telemetry instrumentation.

## Compile Phase Shares (Release)

Shares are of the instrumented phase sum, not process wall time.

| Phase | Cold process<br>median | Cold share | Warm process<br>first fixture | Warm share | Warm retained session<br>median samples 2–20 | Warm share |
|---|---:|---:|---:|---:|---:|---:|
| Semantic session initialization | 12,197.465 ms | 99.9241% | 0.048 ms | 0.6299% | 0.000 ms | 0.0000% |
| Parse | 0.011 ms | 0.0001% | 0.038 ms | 0.4987% | 0.013 ms | 0.0831% |
| Discovery + link | 1.236 ms | 0.0101% | 1.061 ms | 13.9239% | 0.154 ms | 0.9848% |
| Semantic update | 6.817 ms | 0.0558% | 6.374 ms | 83.6483% | 15.437 ms | 98.7147% |
| User/compiler projection | 1.043 ms | 0.0085% | 0.099 ms | 1.2992% | 0.090 ms | 0.5755% |
| **Instrumented phase sum** | **12,206.736 ms** | **100%** | **7.620 ms** | **100%** | **15.638 ms** | **100%** |

## Cold Image-Build Composition (Release)

The release cold image-build span was **12,197.311 ms**, composed as follows:

| Image-build phase | Time (Release) | Share (Release) | Time (Debug) | Debug → Release Speedup |
|---|---:|---:|---:|---:|
| Canonical semantic update | 12,059.863 ms | 98.87% | 40,847.909 ms | **3.39×** |
| Canonical semantic baseline | 72.028 ms | 0.59% | 292.228 ms | **4.06×** |
| Query freeze | 20.361 ms | 0.17% | 68.842 ms | **3.38×** |
| Snapshot validation | 0.036 ms | <0.01% | 0.110 ms | **3.06×** |
| Other image-build overhead | 45.023 ms | 0.37% | 91.429 ms | **2.03×** |

### Canonical Semantic Update Children Breakdown

| Semantic Update Phase | Release Time | Release Share | Debug Time | Speedup |
|---|---:|---:|---:|---:|
| Callable signatures | 5,467.216 ms | 45.33% | 18,781.000 ms | **3.44×** |
| Declaration surfaces | 1,947.819 ms | 16.15% | 6,307.000 ms | **3.24×** |
| Initial callable bodies | 1,415.740 ms | 11.74% | 4,895.000 ms | **3.46×** |
| Field signatures | 866.101 ms | 7.18% | 2,965.000 ms | **3.42×** |
| Conformances | 745.435 ms | 6.18% | 2,167.000 ms | **2.91×** |
| Source indexing | 740.955 ms | 6.14% | 2,844.000 ms | **3.84×** |
| Formal projection | 250.727 ms | 2.08% | — | — |
| Final revalidation | 169.130 ms | 1.40% | — | — |

All major phases scaled consistently between **2.9× and 3.8× faster** when moving from `dev` to `release`.

## Cache Evidence and Variance (Release)

| Scenario | Samples | Session init | Fixture compile | Cache evidence |
|---|---:|---:|---:|---|
| Cold process | 3 | 11.694–13.726 s; median 12.198 s | 6.903–13.701 ms; median 9.322 ms | 1 build, 0 hits per process |
| Warm process, first fixture | 1 | 0.052 ms | 7.612 ms | 0 build, 1 hit |
| Warm retained session | 19 | 0 ms | 10.935–87.613 ms; median 15.644 ms | no additional image build |

## Memory Footprint

Measured via `/usr/bin/time -l` on macOS (arm64):
- Cold process peak RSS: **854 MB – 858 MB** (peak footprint ~845 MB).
- Warm retained process peak RSS: **810 MB** (peak footprint ~842 MB).

## Reproduction

Build the release binary:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --release -p phalcom-core \
  --example perf_universe_compile_cache --features test-timings
```

Run cold process sample:

```text
/usr/bin/time -l target/release/examples/perf_universe_compile_cache cold
```

Run warm retained-process sample:

```text
/usr/bin/time -l target/release/examples/perf_universe_compile_cache warm 20
```

Raw summary CSV is stored at:
[`PERF003-in-memory-image-compile-measurements-release.csv`](PERF003-in-memory-image-compile-measurements-release.csv).
