# Benchmark Environment Specifications

- **Date of Execution**: 2026-10-17
- **Host System**: Apple MacBook Pro (14-inch)
- **CPU**: Apple M1 Pro (8 physical cores: 6 performance + 2 efficiency)
- **Architecture**: `aarch64-apple-darwin`
- **RAM**: 16 GB unified memory (17,179,869,184 bytes)
- **OS**: macOS 26.5 (Darwin Kernel Version 25.5.0, Build 25F71)

## Toolchain & Compiler
- **Toolchain Channel**: `nightly-2026-07-10` (pinned in `rust-toolchain.toml`)
- **rustc**: `rustc 1.99.0-nightly (af3d95584 2026-07-09)`
- **cargo**: `cargo 1.99.0-nightly (59800466c 2026-07-07)`
- **Flags**: `RUSTFLAGS=""`, `RUSTC_WRAPPER=""`

## Git & Repository
- **Repository**: `aureat/phalcom-lang`
- **Branch**: `perf/universe-characterization`
- **Verified Commit SHA**: `e45030869cfcd5fb00c1e7d221cdaebce8984b27`
- **Upstream Sync**: Synchronized with `origin/perf/universe-characterization`
- **Worktree**: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization`
- **Tree Status**: Clean (`git status --short` empty)

## Build Profile Configuration
- **Profile**: `release` (`--release`)
- **Optimization Level**: `opt-level = 3`
- **Link-Time Optimization**: `lto = true`
- **Codegen Units**: `codegen-units = 1`
- **Panic Strategy**: `panic = "abort"`

## Measurement Methodology
- **Clock**: Rust standard monotonic timer (`std::time::Instant`)
- **Fresh-Process Isolation**: Subprocess spawning via `std::process::Command` ensuring no resident memory, cache leaks, or singleton reuse between samples.
- **Memory Measurement**: Kernel resource accounting (`libc::getrusage(RUSAGE_SELF, ...)` on macOS, reading `ru_maxrss` in bytes).
- **Sampling Discipline**: 10 distinct fresh-process executions per scenario/workload; raw per-sample data preserved in accompanying CSVs.
