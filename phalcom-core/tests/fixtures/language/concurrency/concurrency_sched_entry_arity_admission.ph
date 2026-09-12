// area: concurrency
// spec: concurrency.md §2
const needsValue = Fiber.new |value| { value }
System.schedule(|| { System.print("healthy") })
try {
  System.schedule(needsValue)
} catch e {
  System.print("rejected at admission")
}
// Failed admission leaves the Fiber new and usable by the correct caller.
System.print(needsValue.call(9))
System.runScheduled()
