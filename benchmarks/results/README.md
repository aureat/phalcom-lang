# Benchmark Results Store

This directory contains durable performance history and baseline indexes for the Phalcom VM.

- `baselines.json` maps named baselines (e.g. `main`) to specific run IDs in `history/`.
- `history/` stores committed `BenchmarkRun` JSON records.
- `schema-v1.json` documents the Schema v1 layout for `BenchmarkRun` and `ComparisonRun`.
