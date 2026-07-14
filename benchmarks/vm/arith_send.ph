// arith_send.ph — allocation-bound micro-benchmark (Tier 0, U-BENCH).
//
// Isolates per-send allocation cost (performance.md §2 cost class 2): `1 +
// 2` compiles to an ordinary `Invoke` of the Number `+` primitive
// (compiler/lib.rs:2190 — there is no bytecode-level arithmetic fast path
// yet), and the `Primitive` arm of `call_method` builds a heap
// `Vec<Value>` for the argument on every call (vm.rs:626). Same send count
// as bare_send.ph, so the wall-clock delta between the two attributes the
// allocation tax independent of dispatch.
//
// Loaded by phalcom-core/benches/vm_bench.rs via `include_str!`; also
// runnable standalone: `phalcom benchmarks/vm/arith_send.ph` (prints `3`).
//
// `acc` holds `1 + 2` and `i` the loop count; the bench reads both back after
// the run and fails on a wrong answer, so a build whose arithmetic fast path
// is wrong cannot post a number.
var i = 0
var acc = 0
while (i < 200000) {
  acc = 1 + 2
  i = i + 1
}
System.print(acc)
