# COLL006.C1.P8 — RangeMap Test and Compilation Timing Report

**Date:** 2026-10-13

**Audience:** Architect investigating long test and compilation times

**Branch:** `feat/telos-collections`

**RangeMap source revision measured:** `76847894` (`feat(COLL006.C1.P8): normalize persistent RangeMap mutations`)

**Timing instrumentation commit:** `6f4adf21` (`test(COLL006): report RangeMap fixture timings`)

## Executive summary

The selected 14 non-complexity RangeMap tests passed serially in **765.24s
(12m 45s)**. Of that time, **675.82s (88.3%)** was spent in the inline fixture
compile call. Ordinary fixtures of 345–7,494 source bytes each took about
44–47s to compile, despite running for only 7–344ms. VM creation totaled
85.92s, of which **81.27s** was the first, cold build of the process-cached
canonical Universe product. Subsequent VM creation took about 0.32–0.40s per
fixture. Inline fixture execution across the 14 tests totaled **3.31s**.

The separately timed complexity test passed and took **469.54s to execute**
its 4,850-byte fixture. That fixture constructs maps up to 32,768 entries in
Phalcom source. The randomized model test is the other compile outlier: its
generated 92,609-byte fixture took 82.80s to compile and 2.50s to execute.

The observed costs point to two separate contributors: repeated semantic
workspace compilation for every `run_inline` fixture, and the large
source-level insertion workload in the complexity test. The timer does not
decompose the compile call internally; a one-second live sample during
compilation showed work in semantic workspace update and formal semantic
projection construction/sorting, which is a lead for deeper profiling rather
than a complete profile.

## Measurement setup

- **Host:** Apple M1 Pro, Darwin 25.5.0, ARM64.
- **Toolchain:** `rustc 1.99.0-nightly (af3d95584 2026-07-09)`.
- **Cargo profile:** test profile, unoptimized/debug. No `--release` run was
  performed.
- **Environment:** `RUSTFLAGS=''`, `RUSTC_WRAPPER=''`, feature
  `test-timings`, and `PHALCOM_TEST_TIMINGS=1`.
- **Scheduling:** one test thread, so the cold first VM and warm later VMs are
  easy to distinguish. This intentionally differs from the earlier recorded
  15-test run, which used three test threads.

The completed command was:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' PHALCOM_TEST_TIMINGS=1 \
  cargo test -p phalcom-core --features test-timings --test core core_range_map \
  -- --nocapture --test-threads=1 --skip complexity
```

The test binary's `--list` output confirmed **14 selected tests**. The
complexity test was excluded from this completed run by name; its source test
was not removed or edited.

The timer was added to `phalcom-core/tests/support/vm.rs::run_inline`, behind
the `test-timings` feature. It measures wall time for `VM::new()`, the existing
bootstrap subphases, `ProgramCompiler::compile_entry_selection`, and
`vm.run_compiled`. The `vm_unaccounted` value is the `VM::new()` wall time minus
the sum of the four recorded bootstrap subphases.

## Aggregate results: 14 non-complexity tests

| Phase | Total | Interpretation |
|---|---:|---|
| `ProgramCompiler::compile_entry_selection` | **675.82s** | 88.3% of the 765.24s test-run time |
| `VM::new()` wall time | **85.92s** | Includes the cold canonical Universe product build |
| ├ canonical Universe product | **81.27s** | One-time process-local cold build; later calls were about 2–8µs |
| ├ kernel/native setup | **0.10s** | 7–11ms per VM |
| ├ per-VM Universe module compilation | **4.30s** | 0.29–0.37s per VM |
| ├ Universe initializer execution | **0.20s** | 13–17ms per VM |
| └ other VM setup (`vm_unaccounted`) | **0.05s** | 3–6ms per VM |
| `vm.run_compiled` fixture execution | **3.31s** | 7ms–2.50s per fixture |
| Test body/harness remainder | **about 0.19s** | Difference between summed measured phases and test result duration |
| **Test result duration** | **765.24s** | 14 passed, 0 failed |

The VM subphase totals sum to the VM wall time within timer rounding. The
non-canonical part of VM construction totaled about 4.65s across all 14 VMs;
the first VM's cold canonical product accounts for nearly all of its 81.58s
creation time.

## Per-test measurements

Times below are wall seconds. `VM::new()` includes canonical-product build
time; only the first test in this serial process paid the cold build.

| Test | Source bytes | `VM::new()` | Inline compile | Inline execution |
|---|---:|---:|---:|---:|
| `range_map_avl_rotations_and_structural_sharing` | 2,951 | 81.581 | 44.820 | 0.344 |
| `range_map_boundary_splices_and_overlay_matrix` | 7,494 | 0.318 | 45.081 | 0.177 |
| `range_map_comparison_callbacks_reenter_allocate_and_fail_atomically` | 1,635 | 0.370 | 44.639 | 0.065 |
| `range_map_construction_and_lookup` | 894 | 0.323 | 47.247 | 0.019 |
| `range_map_coverage_removal` | 1,041 | 0.366 | 46.691 | 0.019 |
| `range_map_equality_and_hash` | 722 | 0.324 | 45.560 | 0.015 |
| `range_map_gc_retains_live_versions_endpoints_values_and_iterators` | 1,846 | 0.397 | 46.820 | 0.038 |
| `range_map_iteration` | 345 | 0.326 | 46.044 | 0.007 |
| `range_map_normalization_and_coalescing` | 1,823 | 0.317 | 44.707 | 0.037 |
| `range_map_overlay_and_range_splicing` | 687 | 0.326 | 44.716 | 0.014 |
| `range_map_randomized_model_conformance_multiple_seeds` | 92,609 | 0.317 | 82.796 | 2.503 |
| `range_map_strict_insertion_and_overlap_rejection` | 1,102 | 0.318 | 47.342 | 0.018 |
| `range_map_unbounded_point_lookup` | 485 | 0.321 | 44.909 | 0.009 |
| `range_map_value_equality_callback_failure_is_atomic` | 1,977 | 0.316 | 44.446 | 0.042 |

For the 13 fixtures other than the generated randomized model, inline
compilation averaged **45.62s** (median **45.56s**). This is a large, fairly
stable cost even for fixtures under 1 KB. The randomized test generated its
92,609-byte source from three deterministic 350-operation sequences and took
about 38 seconds longer than that ordinary compile range.

## Separately measured complexity fixture

The complexity test completed and passed in the initial serial measurement
run, before the requested 14-test rerun. It was not part of the completed
14-test command above.

| Test | Source bytes | Warm `VM::new()` | Inline compile | Inline execution |
|---|---:|---:|---:|---:|
| `range_map_complexity_scales_with_tree_height_and_affected_region` | 4,850 | 0.321s | 48.122s | **469.539s** |

Its source loop inserts 32,768 `ComplexityPoint` values, retaining snapshots
at 32 and 1,024 entries, then exercises lookup, removal, and overlay at all
three sizes. The 469.54s measurement covers the complete `vm.run_compiled`
phase, not only the insertion loop. The source and live stack sample identify
the insertion workload as the likely dominant operation, but no timer was
placed inside that loop, so its exact share is unknown.

Adding the separately measured complexity compile and execution to the 14-test
run gives a **serial all-15 estimate of about 1,283s (21m 23s)**. This is an
estimate across separate runs, not a completed all-15 invocation. The earlier
checkpoint records a successful 15/15 run in **596.99s** with three test
threads. These results are not a controlled thread-count A/B: the current
measurement enabled timing instrumentation and ran serially. Concurrent tests
can overlap fixture compilation and the long complexity execution, which is a
plausible contributor to the lower historical wall time, but the size of that
effect has not been measured in a paired run.

## Code-path analysis

Each source-language RangeMap test calls `vm_support::run_inline` with a
distinct fixture. For each call the helper:

1. Creates a fresh full VM with `VM::new()`.
2. Calls `ProgramCompiler::compile_entry_selection(EntrySelection::Inline(...))`.
3. Runs the compiled entry with `vm.run_compiled`.

`ProgramCompiler::compile_entry_selection` analyzes the selection before
compilation. The inline analyzer parses the fixture, constructs a synthetic
project universe, loads the selected source-authored Universe conformance
modules, links their interfaces, and calls
`phalcom_semantic::analyze_workspace`. That path is repeated for every fixture
because the helper invokes the one-shot compile API separately; it does not
share a `ProgramCompilationSession` between tests.

The code references are:

- `phalcom-core/tests/support/vm.rs` — timings and `run_inline`.
- `phalcom-core/src/modules/compile.rs` — `ProgramCompiler::compile_entry_selection`
  and inline analysis/linking.
- `phalcom-core/tests/core/collections/range_map.rs` — the 1,050-operation
  randomized fixture and 32,768-entry complexity fixture.
- `phalcom-core/src/vm/bootstrap.rs` — existing per-VM bootstrap timers.

During the first measured fixture's compile, a one-second macOS `sample` showed
active frames in `analyze_workspace`, `SemanticWorkspaceSession::update`, and
`FormalSemanticProjection::build_module_projection`, including sorting
formal callable-site records. This supports semantic workspace/projection
work as a compile hotspot. One sample does not apportion the complete
44–83-second compile interval, and no call counts or allocation profile were
collected.

## Findings, confidence, and caveats

### Directly measured

- Fresh inline fixture compilation dominates the 14-test serial run.
- The canonical Universe product build is expensive but is cached within the
  test process; it explains only the first VM creation.
- Warm `VM::new()` and ordinary fixture execution are small compared with
  inline compilation.
- The complexity fixture's source-level VM execution is very expensive in the
  unoptimized test profile.
- The randomized fixture's large generated source increases compile time and
  modestly increases runtime.

### Strongly supported interpretation

Repeated inline fixture analysis is the primary cost of the ordinary tests.
This conclusion follows from the helper call boundaries and the measured
compile wall times. The sampled semantic workspace/projection frames provide a
specific investigation lead, not a complete causal profile.

### Not established

- The exact time split within inline compilation: parsing, Universe source
  loading, linking, semantic session initialization/update, projection
  construction, and bytecode compilation were not timed separately.
- Whether the same timing distribution holds in an optimized/release profile.
- The exact elapsed cost of the 32,768-insert loop apart from the remainder of
  the complexity fixture.
- A controlled serial-versus-three-thread comparison under identical build
  flags and instrumentation.
- Whether timing instrumentation itself materially changes wall time; there
  was no uninstrumented control run.

## Suggested architect follow-up

1. Add or reuse phase timers around the inline analysis path to split the
   44–83s `compile_entry_selection` interval into parse, Universe module load,
   link, semantic workspace/session, projection, and bytecode phases.
2. Profile formal projection construction with record counts, sort input
   sizes, allocations, and CPU samples over a full fixture compile.
3. Evaluate whether these tests can share compilation work without weakening
   fixture isolation, cold/incremental parity, or semantic identity
   guarantees. Treat session reuse as an architectural question, not an
   assumed local optimization.
4. Time the 32,768-entry insertion loop independently and decide whether the
   wall-clock-heavy complexity test belongs in the default lane or a dedicated
   slow/performance lane. Preserve its comparison-count and structural
   assertions if its scheduling changes.
5. If thread-count behavior matters, repeat the same 14-test command with
   serial and three-thread scheduling on the same commit, machine, toolchain,
   and timing feature.

No test source, acceptance condition, or P8/P9 lifecycle gate was changed by
this timing investigation.
