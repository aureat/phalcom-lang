# Phalcom Benchmarks

> Sources: Phalcom repository, 2026-09-07
> Raw: [benchmarks documentation and schemas](../raw/benchmarks/2026-09-07-benchmarks.md)
> Updated: 2026-09-07

## Overview

The `benchmarks/` tree is a staged measurement and correctness corpus for Phalcom. It contains design-only annotation examples, self-verifying mathematics programs, VM microbenchmarks and baselines, cross-language Wren ports, and durable JSON result/schema files. The repository documentation treats these areas as evidence and promotion inputs, not as a single Cargo test suite.

## Corpus families

The annotation showcase is deliberately non-runnable: it presents the intended Phaldoc and annotation surface while documenting why the current lexer and documentation generator cannot execute it. It should remain separate from the math runner.

The math corpus consists of Phalcom programs that print `true` for mathematical identities rather than comparing only hardcoded decimal output. Its README separates the arithmetic/control-flow tier, an intermediate object-model rational tier, and a later tier that depends on mutable-collection and standard-library surfaces. The bundled runner supports the whole corpus, strict mode, or named files; later-tier pending results do not fail the default run.

The Wren suite contains direct Phalcom ports for throughput comparison. Its documentation emphasizes output comparison against the Wren originals and records porting gaps such as range syntax, index syntax, `System.clock`, and `System.gc()`. These ports are useful both as performance workloads and as executable surface-compatibility probes.

## Measurement and result storage

The VM area contains whole-process workloads, micro-programs, scripts, and a reproducible Tier 0 baseline. The baseline documentation measures lifetime wall-clock and peak resident memory for the process, and separately describes Criterion microbenchmarks and sampling-based attribution. It explicitly distinguishes measured data from oral or unmeasured comparison figures.

The results store keeps named baselines mapped to run IDs, committed `BenchmarkRun` history records, and a schema document. Schema v1 requires run identity, timestamp, Git/build/host metadata, layouts, command, suite, resource quality, cases, and summary; resource quality distinguishes `full` from `wall_only`.

## Promotion discipline

The math README describes promotion as a per-file process: run the program, confirm every output is `true`, move it into the language test area with the standard header, capture a golden stdout snapshot, and mark or remove the staging row. This keeps a benchmark's mathematical correctness evidence distinct from its eventual regression-test ownership.

Benchmark documentation also carries important scope boundaries: some files are drafts or design specimens, result numbers may be machine-specific, and comparison workloads must preserve output correctness before their timing is interpreted.

## See Also

- [Phalcom Native Surface](../native-surface/phalcom-native-surface.md)
- [Phalcom Native Surface Generator](../native-surface-gen/phalcom-native-surface-gen.md)
