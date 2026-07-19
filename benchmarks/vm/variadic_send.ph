// variadic_send.ph — variadic-dispatch micro-benchmark (Tier 0, U-BENCH).
//
// The ONLY program in this repo that reaches the variadic probe. `Invoke`'s
// miss order is IC -> exact-selector probe -> variadic probe -> dNU
// (vm/dispatch.rs, method-lookup.md §1), so a variadic call is the sole shape
// that executes the probe and `VM::variadic_selector_cache` behind it. Without
// this program that path is unmeasurable, and U-HOTPATH Change 2 was first
// recorded as "a wash" purely because nothing here executed it (perf-log 004,
// findings F12's sibling lesson: `wash` and `never ran` look identical to a
// harness that only reports time).
//
// It also prices what Change 2 did NOT fix: a variadic hit never refills the
// IC, so each call still pays two full hierarchy walks.
//
// Loaded by phalcom-core/benches/vm_bench.rs via `include_str!`; also runnable
// standalone: `phalcom benchmarks/vm/variadic_send.ph` (prints `6000000`).
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
