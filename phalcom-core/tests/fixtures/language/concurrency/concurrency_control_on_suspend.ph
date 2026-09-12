// area: concurrency
// spec: CONC002.C2.P1-R1
// status: PASS
// Proves that Block.on handler is suspension-transparent: it can yield and resume.

let f = Fiber.new || {
  let h = || { throw Error.new("boom") }
  h.on(Error) |e| {
    Fiber.yield("caught " + e.message)
    "handler final"
  }
}

System.print(f.call())
System.print(f.call())
