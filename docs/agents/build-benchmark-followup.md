# Build Benchmark Follow-up

Use this guide after running the build-parallelism benchmark scripts. The
scripts measure cold compilation and linking of the workspace test artifacts;
they do not measure test execution time.

## Run the benchmarks

Run benchmarks serially from repository root:

```bash
scripts/bench-build-jobs.sh
scripts/bench-rustc-threads.sh
```

Pass values to limit a run:

```bash
scripts/bench-build-jobs.sh 2 4 6 8
scripts/bench-rustc-threads.sh 2 4 6
```

The rustc-thread script normally fixes Cargo to one job. Set
`PHALCOM_CARGO_JOBS` only when testing a selected combination:

```bash
PHALCOM_CARGO_JOBS=4 scripts/bench-rustc-threads.sh 2 4 6
```

Do not run two benchmark scripts at the same time. Concurrent builds distort
CPU, memory, disk, and linker measurements.

## Locate results

Each invocation creates a timestamped directory below `target/`:

```text
target/bench-build-jobs-<timestamp>/jobs-<value>.<random>/timing.log
target/bench-build-jobs-<timestamp>/jobs-<value>.<random>/cargo-timings/cargo-timing.html

target/bench-rustc-threads-<timestamp>/threads-<value>.<random>/timing.log
target/bench-rustc-threads-<timestamp>/threads-<value>.<random>/cargo-timings/cargo-timing.html
```

The terminal log contains Cargo output plus `/usr/bin/time -p` measurements:

```text
real <seconds>
user <seconds>
sys  <seconds>
```

`real` is elapsed wall-clock time and primary developer-experience metric.
`user` plus `sys` indicates total CPU work. If wall time stops improving while
CPU time rises, the configuration is likely oversubscribed.

Open a Cargo report with:

```bash
open target/bench-build-jobs-<timestamp>/jobs-<value>.<random>/cargo-timings/cargo-timing.html
```

The report identifies slow crates, build scripts, and link stages. Use it to
explain a result; do not infer a setting from wall time alone when one stage
failed or was retried.

## Compare fairly

The scripts already create a fresh `CARGO_TARGET_DIR` for every value and
bypass `sccache`. This is intentional: changing only `-j` or `-Zthreads` must
not turn later runs into cache-hit measurements.

Before comparing runs:

1. Keep source, `Cargo.lock`, toolchain, compiler flags, power mode, and target
   architecture unchanged.
2. Fetch dependencies before timing if needed:

   ```bash
   cargo fetch --locked
   ```

3. Let every selected value finish. Record failures separately; never treat a
   failed or incomplete run as a fast result.
4. Repeat each value at least three times and compare medians. Run the same
   values in more than one order if thermal throttling or background load is
   suspected.
5. Keep cold-build results separate from warm incremental and warm `sccache`
   results.

Resetting sccache counters does not clear its artifacts:

```bash
sccache --zero-stats
# run a warm-cache experiment
sccache --show-stats
```

Use cold scripts for compiler-throughput comparisons. Use warm-cache runs to
measure the workflow developers actually experience after dependencies and
unchanged crates are cached.

## Select a configuration

Treat the two scripts as separate experiments:

- `bench-build-jobs.sh` varies Cargo job count while retaining workspace rustc
  flags.
- `bench-rustc-threads.sh` varies rustc's internal `-Zthreads` value with one
  Cargo job by default.

After finding a promising rustc value, benchmark combined settings explicitly.
Total compiler pressure is approximately:

```text
Cargo jobs × rustc threads
```

On this eight-CPU machine, do not assume the largest values are fastest. The
workspace configuration has previously documented hangs at higher rustc thread
counts; stability is part of the result.

Before changing `.cargo/config.toml`, require:

- the candidate wins by median wall time, not by one run;
- no build or linker failures;
- no unacceptable memory pressure or thermal slowdown;
- the Cargo timing report explains the improvement;
- a warm incremental build remains acceptable.

Then rerun the candidate with sccache enabled and run the relevant validation
command, for example:

```bash
cargo test --workspace --no-run --locked
cargo test --workspace
```

Do not mix this build-tuning evidence with unrelated source or test failures.
Record baseline, candidate, and unverified scope separately.

## Record results

Use a compact table for each experiment:

| Experiment | Cargo jobs | rustc threads | real | user | sys | Cache mode | Result |
| --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| cold baseline |  |  |  |  |  | bypassed |  |
| cold candidate |  |  |  |  |  | bypassed |  |
| warm candidate |  |  |  |  |  | sccache |  |

Include the benchmark timestamp, toolchain, target architecture, selected
configuration, failures, and the path to the Cargo timing report. A result is
ready to apply only when another run can reproduce it from the recorded setup.
