let future = Future.new()
let f = Fiber.new || {
  Fiber.yield(1)
  let x = future.await
  Fiber.yield(x)
  "terminal"
}
System.print(f.call())
System.print(f.call())
System.print(f.isDone)
future.settleValue(42)
System.runScheduled()
System.print("pump returned")
System.print(f.isDone)
System.print(f.call())
System.print(f.isDone)
let ordinary = Future.new()
let waiter = Fiber.new || { System.print(ordinary.await); "waiter terminal" }
System.print(waiter.try())
ordinary.settleValue("wake")
System.runScheduled()
System.print(waiter.isDone)
