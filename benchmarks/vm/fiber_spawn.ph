// fiber_spawn.ph — fiber spawn/yield micro-benchmark (Tier 0, U-BENCH).
//
// Layers fiber-object allocation and the fiber switch on top of the same
// per-send dispatch + allocation cost as arith_send.ph. ADR-0051 /
// performance.md §2 already establish the switch itself is O(1)
// (`mem::take` of three containers, fiber.rs:29-51) and is not the
// target — this benchmark exists to confirm that finding numerically and
// to give Tier 2+ a fiber-path regression tripwire, since Skynet
// (benchmarks/concurrency/skynet.ph) is dominated by the sends around
// millions of fiber switches, not the switches themselves.
//
// Loaded by phalcom-core/benches/vm_bench.rs via `include_str!`; also
// runnable standalone: `phalcom benchmarks/vm/fiber_spawn.ph` (prints `0`).
//
// `acc` holds the value the last fiber yielded back through `call()` and `i`
// the loop count; the bench reads both back after the run, so a build that
// spawns fibers without running them cannot post a number.
var i = 0
var acc = 0
while (i < 20000) {
  var f = Fiber.new { Fiber.yield(0) }
  acc = f.call()
  i = i + 1
}
System.print(acc)
