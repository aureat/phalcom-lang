// area: concurrency
// spec: concurrency.md; ADR-0030 §4; iteration.md; bytes.md §3.1 law 8
// status: PASS
// Formerly `each_generator_raises.ph`, asserting that `Fiber.yield` through
// `List#each`'s block call raises CannotYieldAcrossNativeFrame. The flat-entry
// fork (U-BYTES follow-on, vm/send.rs) removed that native frame: an ordinary
// `f.call(...)` from bytecode enters the closure in the same dispatch loop,
// so `each` is now yield-transparent and the generator pattern works — the
// Lua/Python "cannot yield across native boundary" wall, gone for this path.

const f = Fiber.new || {
  [1, 2, 3].each |x| { Fiber.yield(x) }
  "end"
}
System.print(f.call())
System.print(f.call())
System.print(f.call())
System.print(f.call())
