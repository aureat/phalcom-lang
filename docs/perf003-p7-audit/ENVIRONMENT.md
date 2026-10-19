# PERF003.C2.P7 measurement environment

- Measurement date: 2026-10-19.
- Branch: `perf/universe-characterization`.
- Measured implementation revision: `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2`.
- R.G7 test additions: `c1524361b9606dc537671997fe5069f2aa66035b`.
- Rust: `rustc 1.99.0-nightly (af3d95584 2026-07-09)`.
- Cargo: `cargo 1.99.0-nightly (59800466c 2026-07-07)`.
- OS and architecture: Darwin 25.5.0, arm64.
- CPU: Apple M1 Pro.
- Build: `--release`, opt-level 3, LTO enabled, codegen-units 1, panic abort.
- Build flags were cleared with `RUSTFLAGS='' RUSTC_WRAPPER=''`.
- Benchmark executable: `target/release/examples/perf_audit_p7`.
- Artifact: 15,936,496 bytes, BLAKE3-256 `b91ccc4e6011c2789c6e20861e6d6d98908074bb45461d54b5bd8885ca9665bf`.

The full artifact section digests and generation measurements are in [ARTIFACT.json](ARTIFACT.json). Benchmark invocations are in [COMMANDS.md](COMMANDS.md).
