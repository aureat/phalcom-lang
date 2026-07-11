// area: concurrency
// spec: concurrency.md
// status: PENDING

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
