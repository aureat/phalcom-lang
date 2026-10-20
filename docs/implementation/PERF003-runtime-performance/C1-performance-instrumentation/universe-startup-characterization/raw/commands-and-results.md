# Reproduction commands and outcomes

All commands run with working directory:

```text
/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization
```

Benchmark source revision for every command:

```text
BASELINE_HEAD=b00d75b3ac9f50d37cf9abfddfde00917e099031
branch=perf/universe-characterization
profile=dev (unoptimized debug)
```

The corpus executable, CLI, and example were built before their timed direct-binary invocations. Cargo build time is not included in those process measurements. Corpus runs set `PHALCOM_TEST_TIMINGS=1`; the startup script changes nothing else between standard Python and `-S` runs.

## Build and compile checks

| Command | Result | Raw output |
|---|---|---|
| `cargo build -p phalcom-core --bin phalcom` | PASS; dev build took 1m34s | `build-cli.txt` |
| `cargo build -p phalcom-core --example perf_universe_characterization --features test-timings` | PASS after correction; incremental dev build | `build-characterization-harness-final.txt` |
| `cargo check -p phalcom-core --all-targets` | PASS after adding non-feature fallback `main` | `check-core-all-targets-default-retry.txt` |
| `cargo check -p phalcom-core --all-targets --features test-timings` | PASS | `check-core-all-targets-test-timings.txt` |
| `cargo test -p phalcom-core --test language-corpus --features test-timings --no-run` | PASS; builds the timed corpus binary | `build-language-corpus-test-timings.txt` |

The first no-feature all-target check failed because the newly added example had no `main` when its feature was absent. The fallback was added; the retry passed. That initial output is retained as `check-core-all-targets-default.txt`.

## Corpus probes

`target/debug/deps/language_corpus-729c87ba84f02663` is the already-built feature-enabled executable.

```sh
PHALCOM_TEST_TIMINGS=1 /usr/bin/time -l \
  target/debug/deps/language_corpus-729c87ba84f02663 \
  corpus::classes --exact --nocapture
```

Result: selected one test, failed at its first fixture `classes/class_arity_overload_construction.ph` with `program compile failure: io error: metadata export error: legacy metadata cannot represent a Record remainder outside the Symbol key domain`. Process wall 1.84 s. Raw output: `corpus-classes.raw.txt`.

```sh
PHALCOM_TEST_TIMINGS=1 /usr/bin/time -l \
  target/debug/deps/language_corpus-729c87ba84f02663 \
  corpus::booleans --exact --nocapture
```

Result: selected one test, failed at its first fixture `booleans/bool_and_or_return_operand.ph` with the same metadata-export error. Process wall 1.68 s. Raw output: `corpus-booleans.raw.txt`.

## VM tier probe

```sh
/usr/bin/time -l target/debug/examples/perf_universe_characterization tiers 20
```

The kernel and native constructors completed at 13.871 ms and 76.058 ms. The first `VM::new()` panicked before any full-VM sample: `canonical Universe source index failed: intrinsic runtime data ProductKind is not a canonical Tuple/Record/Pack source declaration`. Raw output: `vm-tiers.raw.txt`.

No warm full VM sample was produced; `20` is the requested maximum loop count, not the observed sample count.

## Process startup controls

```sh
python3 docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/universe-startup-characterization/measure_process_startup.py \
  --cwd /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization \
  --output docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/universe-startup-characterization/raw/process-startup.raw.txt \
  --repetitions 5
```

Result: five fresh Python processes and five fresh `python3 -S` processes succeeded. The one Telos CLI blocker probe exited 65 after 35.804 s with the metadata-export error. Raw output: `process-startup.raw.txt`.

## Parsing

```sh
python3 docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/universe-startup-characterization/analyze_performance_timings.py \
  --input docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/universe-startup-characterization/raw/corpus-classes.raw.txt \
  --input docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/universe-startup-characterization/raw/corpus-booleans.raw.txt \
  --csv docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/universe-startup-characterization/PERF003-universe-startup-measurements.csv \
  --summary docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/universe-startup-characterization/corpus-partial-summary.md
```

Result: parsed two timing lines, one from each separate process; malformed timing lines: 0. VM/user phases are absent because both fixtures failed before VM construction. Raw log headers identify the pinned baseline.

```sh
python3 docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/universe-startup-characterization/analyze_process_startup.py \
  --input docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/universe-startup-characterization/raw/process-startup.raw.txt \
  --csv docs/implementation/PERF003-runtime-performance/C1-performance-instrumentation/universe-startup-characterization/process-startup-measurements.csv
```

Result: parsed 11 process samples; malformed/unparsed output: 0.

## Deliberately not run

- No three-run Telos cold-process set: the first such probe failed after 35.804 s before user execution, so further identical failures would not add phase evidence.
- No warm full-VM loop, `decompose` mode, or 1,000-call executor sample: all require a successful full VM and failed at the ProductKind source-index blocker.
- No broad suite, profiler, release build, or artifact experiment: those cannot resolve the pre-execution correctness blockers and are outside this bounded characterization.
