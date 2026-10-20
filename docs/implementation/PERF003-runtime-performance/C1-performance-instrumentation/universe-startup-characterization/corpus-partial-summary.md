Baseline SHAs: `b00d75b3ac9f50d37cf9abfddfde00917e099031`
Branch: `perf/universe-characterization`
Raw timing logs: `docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/universe-startup-characterization/raw/corpus-classes.raw.txt`, `docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/universe-startup-characterization/raw/corpus-booleans.raw.txt`
Parsed fixture records: 2
Matching fixture failures: 2
Failed samples are partial. Blank phase fields mean the phase was not observed; they are excluded from totals.

| Rank | Phase | Count | Sum (ms) | Share of observed phase sum | Mean (ms) | Median (ms) | P90 (ms) | P95 (ms) | Min (ms) | Max (ms) |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | semantic_update | 2 | 2865.806 | 86.26% | 1432.903 | 1432.903 | 1498.466 | N/A (<20 samples) | 1367.340 | 1498.466 |
| 2 | semantic_session_init | 2 | 413.504 | 12.45% | 206.752 | 206.752 | 212.953 | N/A (<20 samples) | 200.551 | 212.953 |
| 3 | compile_projection | 2 | 41.888 | 1.26% | 20.944 | 20.944 | 21.938 | N/A (<20 samples) | 19.950 | 21.938 |
| 4 | compile_link | 2 | 1.025 | 0.03% | 0.513 | 0.513 | 0.719 | N/A (<20 samples) | 0.306 | 0.719 |
| 5 | compile_parse | 2 | 0.158 | 0.00% | 0.079 | 0.079 | 0.112 | N/A (<20 samples) | 0.047 | 0.112 |
| 6 | compile_io | 2 | 0.099 | 0.00% | 0.050 | 0.050 | 0.067 | N/A (<20 samples) | 0.032 | 0.067 |
| 7 | canonical_universe_product | 0 | N/A | N/A | N/A | N/A | N/A | N/A | N/A | N/A |
| 8 | vm_kernel_native | 0 | N/A | N/A | N/A | N/A | N/A | N/A | N/A | N/A |
| 9 | vm_universe_compile | 0 | N/A | N/A | N/A | N/A | N/A | N/A | N/A | N/A |
| 10 | vm_universe_execute | 0 | N/A | N/A | N/A | N/A | N/A | N/A | N/A | N/A |
| 11 | user_execution | 0 | N/A | N/A | N/A | N/A | N/A | N/A | N/A | N/A |
| — | measured_phase_total_ms | 2 | 3322.480 | 100.00% | 1661.240 | 1661.240 | 1734.219 | N/A (<20 samples) | 1588.261 | 1734.219 |

## First fixture versus warm fixtures by process

| Process | Phase | First (ms) | Warm median (ms) | First / warm median |
|---|---|---:|---:|---:|
| corpus-booleans.raw | compile_io | 0.067 | N/A (no warm sample) | N/A |
| corpus-booleans.raw | compile_parse | 0.047 | N/A (no warm sample) | N/A |
| corpus-booleans.raw | compile_link | 0.306 | N/A (no warm sample) | N/A |
| corpus-booleans.raw | semantic_session_init | 200.551 | N/A (no warm sample) | N/A |
| corpus-booleans.raw | semantic_update | 1367.340 | N/A (no warm sample) | N/A |
| corpus-booleans.raw | compile_projection | 19.950 | N/A (no warm sample) | N/A |
| corpus-booleans.raw | canonical_universe_product | N/A | N/A | N/A |
| corpus-booleans.raw | vm_kernel_native | N/A | N/A | N/A |
| corpus-booleans.raw | vm_universe_compile | N/A | N/A | N/A |
| corpus-booleans.raw | vm_universe_execute | N/A | N/A | N/A |
| corpus-booleans.raw | user_execution | N/A | N/A | N/A |
| corpus-classes.raw | compile_io | 0.032 | N/A (no warm sample) | N/A |
| corpus-classes.raw | compile_parse | 0.112 | N/A (no warm sample) | N/A |
| corpus-classes.raw | compile_link | 0.719 | N/A (no warm sample) | N/A |
| corpus-classes.raw | semantic_session_init | 212.953 | N/A (no warm sample) | N/A |
| corpus-classes.raw | semantic_update | 1498.466 | N/A (no warm sample) | N/A |
| corpus-classes.raw | compile_projection | 21.938 | N/A (no warm sample) | N/A |
| corpus-classes.raw | canonical_universe_product | N/A | N/A | N/A |
| corpus-classes.raw | vm_kernel_native | N/A | N/A | N/A |
| corpus-classes.raw | vm_universe_compile | N/A | N/A | N/A |
| corpus-classes.raw | vm_universe_execute | N/A | N/A | N/A |
| corpus-classes.raw | user_execution | N/A | N/A | N/A |
