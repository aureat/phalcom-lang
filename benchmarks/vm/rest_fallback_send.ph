// rest_fallback_send.ph — rest-family fallback micro-benchmark (Tier 0, U-BENCH).
//
// Each concrete multi-argument selector misses exact lookup and exercises
// F.3 rest-family fallback before dNU. This keeps fallback cost measurable
// instead of confusing an unexecuted path with a performance wash.
//
// It prices current uncached fallback: each call still pays hierarchy walks.
//
// Loaded by phalcom-core/benches/vm_bench.rs via `include_str!`; also runnable
// standalone: `phalcom benchmarks/vm/rest_fallback_send.ph` (prints `6000000`).
class V {
  sum(*args) { return args.size }
}
const v = V.new()
let i = 0
let acc = 0
while (i < 2000000) {
  acc = acc + v.sum(1, 2, 3)
  i = i + 1
}
System.print(acc)
