const err = Error.new("x")
const f = Fiber.new |first| {
  System.print(first)
  const v = Fiber.yield(None)
  System.print(v)
  Fiber.yield(err)
  err
}
System.print(f.isRoot)
const a = Fiber.new || { System.print(f.call("first")); System.print(f.isDone) }
const b = Fiber.new || { System.print(f.try("second") == err); System.print(f.isDone) }
const c = Fiber.new || { System.print(f.call() == err); System.print(f.isDone); System.print(f.error.isNone) }
a.call()
b.call()
c.call()
const n = Fiber.new || { Fiber.yield(None); None }
System.print(n.try())
System.print(n.isDone)
System.print(n.try())
System.print(n.isDone)
const failing = Fiber.new || { Fiber.yield(1); err.raise() }
const firstCaller = Fiber.new || { failing.call() }
System.print(firstCaller.call())
const lastCaller = Fiber.new || { System.print(failing.try() == err); "last survived" }
System.print(lastCaller.call())
System.print(firstCaller.error.isNone)
