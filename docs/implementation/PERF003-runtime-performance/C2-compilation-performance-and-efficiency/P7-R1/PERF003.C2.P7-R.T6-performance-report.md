# PERF003.C2.P7 R.T6 performance report — reconciled

## Certification status

- R.G6: **PASS**, independently recertified on the evidence in `docs/perf003-p7-audit/`.
- Measured implementation revision: `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2`.
- Evidence publication revision: `99768ecfed6b5d0f33ac9b2a6239a5af07aeec96`.
- Evidence includes 80 cold rows, 40 isolated warm rows, 20 classified runtime-error rows, artifact metadata, SlotMap parity, and allocation-boundary logs.
- Test-only R.G7 additions were committed at `c1524361b9606dc537671997fe5069f2aa66035b`.

## Current artifact

`universe.telosc` is 15,936,496 bytes with BLAKE3-256 digest `b91ccc4e6011c2789c6e20861e6d6d98908074bb45461d54b5bd8885ca9665bf`. Whole-load allocation N is 373,069,634 bytes; independent allocator requests match exactly. N−1 fails with the typed budget error, and exact-N retry succeeds.

## Cold and warm results

Cold runs used 10 fresh child processes per workload and mode. All 80 compiles succeeded. Artifact-backed total-duration medians were 310–315 ms for `print42`, `option_string`, and `synthetic_pkg`; the `slotmap` runtime package median was 625 ms. Source-backed medians were 9,163–9,535 ms. Detailed values and recomputable statistics are in `docs/perf003-p7-audit/REPORT.md` and `RAW-COLD.csv`.

Warm source and artifact modes used separate PIDs, one untimed warmup each, and 20 successful timed repetitions each. The timed workload alternated two equivalent string-literal inputs; the CSV records 10 `modified_input` and 9 `unchanged_input` rows per mode after the initial compilation. Median compile times were 5.151 ms and 5.081 ms. Peak RSS is a high-water mark; it is not a retained-memory or leak measurement.

The runtime suite recorded 20 `RUNTIME_ERROR` rows. Their output field is blank, so this report makes no runtime execution or speedup claim.

## Environment and reproduction

The measurements used Rust `1.99.0-nightly`, Darwin 25.5.0 arm64 on Apple M1 Pro, and the release profile (opt-level 3, LTO, one codegen unit, panic abort). Exact commands and complete statistics are in `docs/perf003-p7-audit/COMMANDS.md` and `REPORT.md`.
