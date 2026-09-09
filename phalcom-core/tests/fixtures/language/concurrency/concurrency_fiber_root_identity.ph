// area: concurrency
// spec: concurrency.md; ADR-0030 §1
// status: PASS

// Rootness is a stable identity fact, not a consequence of the current
// dynamic resumer link.
const child = Fiber.new || { 1 }
System.print(child.isRoot)
System.print(Fiber.current.isRoot)
