# Benchmark corpus

> Sources: benchmark documentation and result schemas
> Raw: [benchmark documentation and schemas](../raw/benchmarks/2026-09-07-benchmarks.md)
> Updated: 2026-09-08

The benchmark corpus separates design-only annotation examples, self-verifying mathematics, VM microbenchmarks/baselines, Wren ports, and durable JSON result history. Correctness promotion and performance measurement are related but separate operations.

## Evidence

A benchmark result records workload identity, command, suite, host/build metadata, resource quality, cases, and summary. Whole-process wall-clock/peak-memory measurements are distinct from Criterion microbenchmarks and sampling attribution. Machine-specific or unmeasured comparisons are not universal claims.

[Native surface](../native/canonical-surface.md) and [runtime](../runtime/overview.md) are workload dependencies, not benchmark owners.
