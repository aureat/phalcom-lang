// area: concurrency
// spec: concurrency.md §1
const result = Future.async || { Fiber.yield(Error.new("data")); 42 }
System.schedule(|| { System.print("healthy") })
System.runScheduled()
System.print(result.isReady)
try { result.await } catch e { System.print(e.message) }
// A manual coroutine still delivers Error as data without failing.
const generator = Fiber.new || { Fiber.yield(Error.new("element")); None }
System.print(generator.call().message)
System.print(generator.error.isNone)
System.print(generator.call())
System.print(generator.isDone)
