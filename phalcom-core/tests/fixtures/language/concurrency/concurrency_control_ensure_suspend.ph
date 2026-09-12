// area: concurrency
// spec: CONC002.C2.P1-R1
// status: PASS
// Proves that Block.ensure cleanup is suspension-transparent: it can yield and restores saved outcome.

let f = Fiber.new || {
  let body = || { 42 }
  body.ensure || {
    Fiber.yield("cleanup step 1")
    Fiber.yield("cleanup step 2")
    "cleanup discarded"
  }
}

System.print(f.call())
System.print(f.call())
System.print(f.call())
