# PERF003 in-memory Universe image compile characterization

**Date:** 2026-10-14
**Branch:** `perf/universe-characterization`
**Measured commit:** `5196d00213356a125bc7d4c872d6b9ad05df5e81`
**Profile:** dev, pinned nightly toolchain, `test-timings` + detailed observability

## Result

The current process-shared in-memory canonical Universe image removes the
dominant cold-session cost for compile-only testing. A fresh process spent a
median **41.305 s** initializing the semantic session; a process with the image
already resident initialized a new session in **0.067 ms**. The first warm
fixture compile then completed in **23.556 ms**, versus **29.857 ms** after cold
initialization. Including session initialization, the measured compile path fell
from approximately **41.335 s to 23.622 ms** (**1,750× faster; 99.943% lower**).

The warmup itself cost **42.152 s** in this run and is intentionally excluded
from the warm operation. Therefore the cache is valuable for a retained process
or for two or more compilations amortized over one process; it is not a
one-shot cold-process speedup by itself.

This is a compile-only measurement. VM bootstrap was excluded because the
existing full-VM path still fails independently at `ProductKind` canonical
facade binding. The result measures the semantic in-memory image used by
`ProgramCompilationSession`, not a persisted artifact or full runtime startup.

## Method

- **Cold process:** three separate executable processes, no pre-existing image;
  each created `ProgramCompilationSession::standalone()` and compiled the same
  tiny fixture once.
- **Warm process:** one process first loaded the canonical image, then created a
  new compilation session and compiled the fixture once for the comparable
  warm-first sample, followed by 19 retained-session fixture replacements.
- The fixture alternated `1 + 2` and `2 + 3` after the first sample so the
  retained-session rows measure ordinary source replacement rather than a
  repeated identical-input shortcut.
- The reported phase sum uses the existing instrumentation fields. It excludes
  small wrapper/recorder overhead, so it is not expected to equal the outer
  wall-clock sample byte-for-byte.

## Compile phase shares

Shares are of the instrumented phase sum, not of process wall time.

| Phase | Cold process<br>median | Cold share | Warm process<br>first fixture | Warm share | Warm retained session<br>median samples 2–20 | Warm share |
|---|---:|---:|---:|---:|---:|---:|
| Semantic session initialization | 41,303.507 ms | 99.9306% | 0.053 ms | 0.2258% | 0.000 ms | 0.0000% |
| Parse | 0.032 ms | 0.0001% | 0.040 ms | 0.1676% | 0.026 ms | 0.0636% |
| Discovery + link | 3.027 ms | 0.0073% | 0.338 ms | 1.4324% | 0.189 ms | 0.4626% |
| Semantic update | 21.215 ms | 0.0513% | 22.672 ms | 96.2116% | 40.328 ms | 98.7052% |
| User/compiler projection | 4.415 ms | 0.0107% | 0.462 ms | 1.9625% | 0.314 ms | 0.7685% |
| **Instrumented phase sum** | **41,332.197 ms** | **100%** | **23.564 ms** | **100%** | **40.857 ms** | **100%** |

The cache changes the dominant phase from semantic session initialization to the
fixture's own semantic update. Once the session is retained, semantic update is
the remaining bottleneck at roughly **98.7%** of measured compile-phase time.

## Cold image-build composition

The detailed cold-process report recorded one image build and no cache hit. The
**41,300.537 ms** image-build span was composed as follows:

| Image-build phase | Time | Share of image-build span |
|---|---:|---:|
| Canonical semantic update | 40,847.909 ms | 98.90% |
| Canonical semantic baseline | 292.228 ms | 0.71% |
| Query freeze | 68.842 ms | 0.17% |
| Snapshot validation | 0.110 ms | <0.01% |
| Other image-build overhead | 91.429 ms | 0.22% |

Within the cold canonical semantic update, the largest directly attributed
children were callable signatures (18.781 s, 45.98%), declaration surfaces
(6.307 s, 15.44%), initial callable bodies (4.895 s, 11.98%), field signatures
(2.965 s, 7.26%), source indexing (2.844 s, 6.96%), and conformances (2.167 s,
5.31%).

## Cache evidence and variance

| Scenario | Samples | Session init | Fixture compile | Cache evidence |
|---|---:|---:|---:|---|
| Cold process | 3 | 40.990–42.915 s; median 41.305 s | 22.019–31.107 ms; median 29.857 ms | 1 build, 0 hits per process |
| Warm process, first fixture | 1 | 0.067 ms | 23.556 ms | 0 build, 1 hit |
| Warm retained session | 19 | 0 ms | 38.545–43.695 ms; median 40.878 ms | no additional image build |

The warm process's complete external wall time was 42.98 s because it includes
the excluded 42.152 s image warmup and all 20 fixture compiles. That number is
not a contradiction: it is the cost of creating the warm process, while the
reported warm compile operation assumes the process and its image are already
resident.

## Reproduction

The compile-only harness is
[`perf_universe_compile_cache.rs`](../../../../../../phalcom-core/examples/perf_universe_compile_cache.rs).
Build it with:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build -p phalcom-core \
  --example perf_universe_compile_cache --features test-timings
```

Then run a cold sample in three fresh processes:

```text
/usr/bin/time -l target/debug/examples/perf_universe_compile_cache cold
```

And run the warm retained-process sample:

```text
/usr/bin/time -l target/debug/examples/perf_universe_compile_cache warm 20
```

The summarized rows are in
[`PERF003-in-memory-image-compile-measurements.csv`](PERF003-in-memory-image-compile-measurements.csv).

## Boundary and residuals

The requested branch worktree had unrelated uncommitted P7 artifact edits that
currently fail compilation in `phalcom-semantic/src/artifact/export.rs` due a
type-inference error. To avoid modifying or masking those edits, the benchmark
ran from an isolated clean checkout of the measured commit with the existing
one-line `TypeRelocation` import correction applied. No production source in
the requested worktree was changed by the measurement.

The prior full-VM characterization remains blocked by the independent
`ProductKind` canonical-facade bootstrap failure; this report makes no claim
about VM initialization or user execution time.
