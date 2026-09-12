// area: concurrency
// spec: CONC002.C2.P1-R1
// status: PASS
// Proves that Block.whileTrue condition and body can suspend and resume freely.

let f = Fiber.new || {
  let count = 0
  let cond = || {
    Fiber.yield("checking " + count.toString)
    count < 2
  }
  let body = || {
    Fiber.yield("body " + count.toString)
    count = count + 1
  }
  cond.whileTrue(body)
}

System.print(f.call())
System.print(f.call())
System.print(f.call())
System.print(f.call())
System.print(f.call())
