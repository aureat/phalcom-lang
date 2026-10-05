let err = Error.new("x")
let f = Fiber.new |first| {
  System.print(first)
  let v = Fiber.yield(None)
  System.print(v)
  Fiber.yield(err)
  err
}
System.print(f.isRoot)
let a = Fiber.new || { System.print(f.call("first")); System.print(f.isDone) }
let b = Fiber.new || { System.print(f.try("second") == err); System.print(f.isDone) }
let c = Fiber.new || { System.print(f.call() == err); System.print(f.isDone); System.print(f.error.isNone) }
a.call()
b.call()
c.call()
let n = Fiber.new || { Fiber.yield(None); None }
System.print(n.try())
System.print(n.isDone)
System.print(n.try())
System.print(n.isDone)
let failing = Fiber.new || { Fiber.yield(1); err.raise() }
let firstCaller = Fiber.new || { failing.call() }
System.print(firstCaller.call())
let lastCaller = Fiber.new || { System.print(failing.try() == err); "last survived" }
System.print(lastCaller.call())
System.print(firstCaller.error.isNone)
