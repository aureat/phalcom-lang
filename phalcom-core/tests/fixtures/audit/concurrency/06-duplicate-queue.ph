const pending = Future.new()
const f = Fiber.new || { System.print(pending.await); System.print("body once"); 42 }
f.call()
System.schedule(f)
pending.settleValue("settled")
System.schedule(|| { System.print("healthy later task") })
System.runScheduled()
System.print("after drain")
