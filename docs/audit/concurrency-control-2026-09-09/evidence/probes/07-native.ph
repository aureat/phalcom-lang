const f = Fiber.new || { for x in [1, 2] { Fiber.yield(x) }; "for done" }
System.print(f.call())
System.print(f.call())
System.print(f.call())
const g = Fiber.new || { [1, 2].each |x| { Fiber.yield(x) }; "each done" }
System.print(g.try())
System.print(g.isDone)
const pending = Future.new()
const h = Fiber.new || { [1].each |x| { pending.await }; "await done" }
System.print(h.try())
System.print(h.isDone)
pending.settleValue(7)
System.runScheduled()
System.print("host survived")
const bad = Future.new()
const guarded = Fiber.new || {
  try { bad.await } catch e { System.print(e.class.name); Fiber.yield(88) }
}
System.print(guarded.try())
System.print(guarded.isDone)
bad.settleValue(1)
System.runScheduled()
System.print("guard host survived")
