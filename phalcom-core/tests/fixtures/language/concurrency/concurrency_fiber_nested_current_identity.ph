// area: concurrency
// spec: concurrency.md; ADR-0030 §3
// status: PASS
// C-FIB adversarial: a fiber (`outer`) creates and drives a *second*, inner
// fiber. `Fiber.current` inside `inner` answers `inner`, never `outer`
// (identity is per-currently-running-fiber, ADR-0030 §3), and `inner`'s
// `Fiber.yield` hands control back to *its* resumer — `outer`, since `outer`
// is the one that called `inner.call()` — not to `outer`'s own resumer (the
// top-level script). So the top level only ever observes `outer`'s single
// `call()` return, never sees `inner`'s intermediate yield.
let outer = None
let inner = None
outer = Fiber.new || {
  System.print(Fiber.current == outer)
  inner = Fiber.new || {
    System.print(Fiber.current == inner)
    System.print(Fiber.current == outer)
    Fiber.yield("inner-yield")
    "inner-done"
  }
  System.print(inner.call())
  System.print(Fiber.current == outer)
  "outer-done"
}
System.print(outer.call())
