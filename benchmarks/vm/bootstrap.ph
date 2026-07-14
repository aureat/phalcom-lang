// bootstrap.ph — bootstrap-cost tripwire (Tier 5, U-BENCH).
//
// Does nothing on purpose. Its whole-process time IS `VM::new` — which re-lexes,
// re-parses and re-compiles `core.ph` on every process start, including every
// golden test and every criterion iteration.
//
// This file exists because bootstrap silently regressed 5ms -> 180ms (35x) and
// passed every gate the harness had: run.sh only asks "did it run", the criterion
// benches amortize bootstrap inside a ~0.9s program, and the wren-suite table is
// single-run. The cause was not `core.ph` growing but the `ifTrue` inliner being
// exponential in nest depth (perf-log findings F13) — a 14-deep conditional in
// `String.codePointAt` costing ~200ms to compile by itself.
//
// Run: `time phalcom benchmarks/vm/bootstrap.ph`. Expect single-digit ms.
// If this is over ~50ms, something is compiling exponentially — profile the
// compiler, not the VM.
System.print(0)
