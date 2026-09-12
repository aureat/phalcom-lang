// area: concurrency
// spec: concurrency.md §2; CONC002.C2.P2
// status: PASS
// System.schedule rejects a Fiber that is already in Yielded status.

const f = Fiber.new || {
  Fiber.yield("yielded")
}

f.call()

try {
  System.schedule(f)
} catch e {
  System.print("caught: " + e.message)
}
