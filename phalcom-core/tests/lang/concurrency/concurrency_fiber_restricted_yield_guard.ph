// area: concurrency
// spec: concurrency.md; ADR-0030 §4
// status: PASS
// C-FIB-3: `Fiber.yield` under a GENUINE re-entrant native call frame raises
// `CannotYieldAcrossNativeFrame` instead of corrupting the fiber's suspended
// position. Post flat-entry (U-BYTES follow-on, bytes.md §3.1): an ordinary
// `f.call(...)` from bytecode no longer creates a native frame — that case is
// now LEGAL and asserted by `concurrency_fiber_yield_through_block_call.ph`.
// The guard's remaining territory is block invocation from inside a native
// primitive: here, an `.on(_)` error handler, which the unwind machinery
// invokes through the re-entrant `block_call` path.

const f = Fiber.new || {
  || { throw Error.new("boom") }.on(Error) |e| { Fiber.yield(1) }
}
const result = f.try()
System.print(result.class.name)
