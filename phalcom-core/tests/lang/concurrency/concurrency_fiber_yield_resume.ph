// area: concurrency
// spec: concurrency.md; ADR-0030
// status: PASS
// C-FIB-1: a Fiber yields successive counter values across resumes.

let counter = Fiber.new {
  var n = 0
  while (true) {
    Fiber.yield(n)
    n = n + 1
  }
}
System.print(counter.call())
System.print(counter.call())
System.print(counter.call())
