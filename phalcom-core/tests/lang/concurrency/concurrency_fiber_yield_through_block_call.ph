// area: concurrency
// spec: bytes.md §3.1 law 8; concurrency.md; ADR-0030 §4
// status: PASS
// The flat-entry tripwire (U-BYTES follow-on): an ordinary `f.call(...)` sent
// from bytecode on a Block/Closure receiver enters the closure frame in the
// SAME dispatch loop — no recursive `run_until`, no native frame — so
// `Fiber.yield` inside it suspends legitimately. This is what keeps
// `each`/`map`/`filter` (all `.ph` over `Function#call`) yield-transparent:
// Lua's "attempt to yield across a C-call boundary" wall, removed for the
// bytecode→call path. If this fixture ever reds with
// CannotYieldAcrossNativeFrame, someone re-routed block calls through the
// re-entrant native path — see vm/send.rs `call_method`'s flat-entry fork.

const f = Fiber.new || {
  const inner = || { Fiber.yield(41) }
  inner.call()
  99
}
System.print(f.call())
System.print(f.call())

// The combinator form, over a native container (bytes.md §9 "yield
// mid-iteration"): each octet is yielded out of the fiber and the fiber
// resumes correctly to completion.
const b = Bytes.fromList([10, 20])
const g = Fiber.new || {
  b.each |x| { Fiber.yield(x) }
  "done"
}
System.print(g.call())
System.print(g.call())
System.print(g.call())
