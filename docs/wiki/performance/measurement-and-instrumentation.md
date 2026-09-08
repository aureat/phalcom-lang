# Measurement and instrumentation

> Sources: performance specification; PERF003
> Raw: [performance specification and program snapshot](../raw/performance/2026-09-08-performance-sources.md)
> Updated: 2026-09-08

Measure before optimizing. A valid performance change has a reproducible in-repo benchmark, a profile attributing cost to a named mechanism, and a recorded before/after number. Behavior must remain byte-identical unless the change is separately specified.

## Protocol

Record baseline identity, build/toolchain/host, workload, warm-up assumptions, wall-clock and memory boundaries, and quality of the result. PERF003 is proposed; current numbers in the benchmark archive remain evidence with their own scope.
