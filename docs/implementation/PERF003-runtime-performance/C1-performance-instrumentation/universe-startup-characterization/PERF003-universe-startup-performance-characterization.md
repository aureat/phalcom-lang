# PERF003 Universe Startup Performance Characterization

**Status: `BASELINE_BLOCKED` — partial independent controls only**
**Benchmark baseline:** `b00d75b3ac9f50d37cf9abfddfde00917e099031`
**Branch:** `perf/universe-characterization`
**Date:** 2026-10-12

## Executive conclusion

The requested end-to-end Telos characterization could not be completed on the pinned baseline. Two separate source paths fail before user execution:

1. A tiny source fixture (`System.print("hello")`) fails program compilation with `legacy metadata cannot represent a Record remainder outside the Symbol key domain`.
2. `VM::new()` panics while obtaining the canonical Universe compiler product: `intrinsic runtime data ProductKind is not a canonical Tuple/Record/Pack source declaration`.

The first error prevents the corpus and CLI measurements from reaching VM construction. The second prevents VM-tier, full-VM, and same-process repeated-execution measurements. These are baseline blockers, not results showing that any Universe artifact optimization would help. No artifact conclusions or Amdahl ceilings are supported by this run.

The measurements that did complete are limited to the two VM controls and independent Python process startup. `VM::new_kernel()` took 13.871 ms and `VM::new_native()` took 76.058 ms in one process. Python 3.14.6 ran the tiny `print("hello")` file in a median 26.592 ms over five fresh processes; `python3 -S` took 18.999 ms. The failed Telos CLI probe lasted 35.804 s and used 678,805,504 bytes maximum RSS, but this is not a valid startup comparison because it never produced a successful Telos program result.

The first selected fixture in each of two corpus lanes failed. Across those two partial samples, observed phase time summed to 3.322 s: semantic update accounted for 86.26%, semantic session initialization 12.45%, projection 1.26%, and parse/link/I/O the remainder. Each process ran only one fixture, and both failures occurred before VM construction; there is no warm comparison. Treat these as evidence about the failing paths only.

## Repository state and environment

- Repository worktree: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization`
- Baseline SHA: `b00d75b3ac9f50d37cf9abfddfde00917e099031`
- Branch: `perf/universe-characterization`
- Parent verified the worktree clean at creation; HEAD remained pinned to the same SHA throughout measurement. Investigation artifacts are now uncommitted in this worktree.
- The active dirty `main` worktree was not used as an input and was not modified.
- Rust: `rustc 1.99.0-nightly (af3d95584 2026-07-09)`, Cargo `1.99.0-nightly (59800466c 2026-07-07)`; repository toolchain file requests nightly-2026-07-10.
- Host: macOS 26.5, Darwin 25.5.0, arm64.
- Rust profile: `dev` / unoptimized debug. The corpus executable and CLI were built before their timed invocations. Cargo build/link time is reported separately in the command log.
- Feature: `test-timings` for corpus timing and the characterization example; `PHALCOM_TEST_TIMINGS=1` for corpus timing output.

The checked-in PERF003 checkpoint records a separate successful `corpus::classes` timing run at an earlier repository state. That historical result does not establish behavior at this exact pinned SHA. This run must be retained as a distinct baseline until the revision difference is evaluated.

## Method and phase boundaries

The existing `FixtureTiming` path was used for the corpus attempt. Source inspection confirms the timing output covers compile I/O, parse, discovery/link, semantic session initialization, semantic update, and user projection. The VM timing fields are populated only after the compiler path reaches VM construction. Here the fixture failed before that point, so those fields are blank in the parsed CSV as unobserved.

The test support's `user_execution` field surrounds `run_compiled`; it is not a pure bytecode-execution timer. `run_compiled` includes runtime materialization, user bytecode compilation, and initialization. The added harness has a `decompose` mode intended to split those operations, but `VM::new()` fails before that mode can run.

The harness and scripts are under this investigation directory. No production Rust source, semantic code, VM bootstrap code, or shared instrumentation implementation was modified. The example is feature-gated and has a fallback `main` without `test-timings`.

## Experiment results

### A/B — corpus timing and first versus warm fixtures

`corpus::classes` and `corpus::booleans` each selected one real test and failed on the first fixture with the Record-remainder metadata-export error. Both produced a timing row before the test failed. The classes fixture recorded 1.734 s in observed phases; the booleans fixture recorded 1.588 s. No later fixture ran, so there is no warm sample, complete lane timing, cold/warm ratio, or representative corpus result.

Aggregated evidence from the first fixture in each process:

| Phase | Sum across two failed first fixtures | Share of observed phase sum |
|---|---:|---:|
| semantic update | 2,865.806 ms | 86.26% |
| semantic session initialization | 413.504 ms | 12.45% |
| user projection | 41.888 ms | 1.26% |
| discovery + link | 1.025 ms | 0.03% |
| parse | 0.158 ms | <0.01% |
| I/O | 0.099 ms | <0.01% |
| VM and user execution phases | Not observed | Not observed |
| **Observed phase sum** | **3,322.480 ms** | **100%** |

There are two samples from separate processes, and both fixtures failed. The whole test processes took 1.84 s (classes) and 1.68 s (booleans), including harness and failure handling. The measured phase sums are not full process times. Each lane has one cold sample and no warm sample; the aggregated sample count is too small to support P95.

The parser rejects malformed `[TIMING]` lines and records the fixture failure. The CSV leaves post-projection phase columns blank rather than misreporting zero-initialized timing fields as measured zero cost.

### C — VM tiers

| Constructor | Observed time | Samples | Result |
|---|---:|---:|---|
| `VM::new_kernel()` | 13.871 ms | 1 | Completed |
| `VM::new_native()` | 76.058 ms | 1 | Completed |
| `VM::new()` | Not measured | 0 completed | Panicked during canonical Universe source indexing |

The approximate native-over-kernel delta is **62.187 ms** for these single sequential samples. There is no warm full-VM median or p90, no Universe compile/initializer timing, and no full-VM counter snapshot. The harness stopped at `VM::new()` before emitting its counter summary.

### D/E/F — process startup, tiny-program attribution, compile versus execute

Five fresh Python processes successfully ran `tiny.py`; five fresh `python3 -S` processes also succeeded. These timings wrap subprocess launch and `/usr/bin/time`, so they are process-level measurements, not Python language throughput.

| Program path | Count | Median wall | Range | Median maximum RSS |
|---|---:|---:|---:|---:|
| `python3 tiny.py` | 5 | 26.592 ms | 25.164–26.955 ms | 15,269,888 bytes |
| `python3 -S tiny.py` | 5 | 18.999 ms | 18.547–19.672 ms | 12,681,216 bytes |
| Telos `phalcom tiny.ph` | 1 failed probe | 35,803.979 ms | n/a | 678,805,504 bytes |

The Telos probe exited 65 with the metadata-export error. It never completed compilation or execution; comparing its elapsed time directly to Python would conflate a failed Telos compiler/bootstrap path with a successful Python script. No successful Telos cold-process run was available. D1 stopped after the one long, reproducible blocker probe; D2 and the compile-only / compile-plus-run decomposition are unavailable because full VM construction fails.

### G/H — counters and executor reuse

No meaningful VM bootstrap counters were emitted: corpus tests stopped before VM construction, and the tier harness panicked before its summary. The existing counters did not produce evidence for cold product builds, Universe compilation, initializer execution, or session reuse in this run. The 1,000-execution precompiled-callable measurement was not reached because its harness mode constructs a full VM first.

### Cargo/build overhead

The CLI build took 1m34s in the dev profile. Building the feature-enabled corpus test executable took about 41s. Feature-disabled and feature-enabled all-target checks both completed successfully after correcting the example's no-feature fallback entry point. These are Rust build/check costs and are excluded from the Telos process measurements.

## Theoretical ceilings and hypotheses

Amdahl-style maximum speedups are **not computable** for the requested artifact candidates: successful Telos end-to-end total time and the corresponding Universe phase times are unavailable. The measured 3.322 s sum across two failed pre-VM paths and cannot stand in for a complete tiny-program runtime. No “worth architectural work?” judgment is warranted from these data.

| Hypothesis | Verdict | Evidence |
|---|---|---|
| H1 — cold canonical Universe construction dominates first use | INCONCLUSIVE | Product construction failed; no successful first-use completion or warm comparison. |
| H2 — fresh full VMs repeatedly pay substantial Universe bootstrap | INCONCLUSIVE | `VM::new()` failed before its first sample. |
| H3 — actual tiny user execution is not dominant | INCONCLUSIVE | No Telos user program reached execution. |
| H4 — process reconstruction is much more expensive than warm reuse | INCONCLUSIVE | No successful Telos process or retained-session path to compare. |
| H5 — runtime bytecode artifact alone has a bounded payoff | INCONCLUSIVE | Universe compile time and successful total runtime are unavailable. |
| H6 — persisted cold canonical products have high potential | INCONCLUSIVE | No successful canonical-product build or full end-to-end baseline. |

## Stop condition and next action

This characterization is blocked by the exact committed baseline, so no runtime architecture work should be planned from its incomplete timings. The source-index failure and metadata-export failure are correctness-path blockers. Do not work around either in the benchmark harness or change the canonical source-index path as part of this task.

The user has identified ongoing, uncommitted type-system corrections in the active `main` worktree. Those corrections were intentionally excluded. Since the observed metadata-export error and semantic-update path may be affected by those corrections, rerun the same benchmark matrix only after the relevant fixes land on a committed revision; record that new SHA separately and retain this report as the pinned-baseline result.

## Deliverables

- This report: `PERF003-universe-startup-performance-characterization.md`
- Fixture timing rows: `PERF003-universe-startup-measurements.csv`
- Process-level samples: `process-startup-measurements.csv`
- Harness and parsers: `../../../../../phalcom-core/examples/perf_universe_characterization.rs`, `analyze_performance_timings.py`, `measure_process_startup.py`, `analyze_process_startup.py`
- Raw command output: `raw/`
- Exact reproduction commands and outcomes: `raw/commands-and-results.md`
- Handoff: `PERF003-universe-startup-characterization-handoff.md`
