let err = Error.new("same")
let mut leak = || { 0 }
let inner = Fiber.new || { err.raise() }
let middle = Fiber.new || {
  let mut x = 77
  leak = || { x }
  inner.call()
  System.print("BAD-after-handler")
}
let outer = Fiber.new || { middle.call(); System.print("BAD-after-middle") }
System.print(outer.try() == err)
System.print(inner.error.unwrapOr(None) == err)
System.print(middle.error.unwrapOr(None) == err)
System.print(outer.error.unwrapOr(None) == err)
System.print(leak.call())
let b = Fiber.new || { err.raise() }
let a = Fiber.new || { System.print(b.try() == err); "A survived" }
System.print(a.try())
let d = Fiber.new || { err.raise() }
let c = Fiber.new || { d.call(); "BAD" }
let top = Fiber.new || { System.print(c.try() == err); "top survived" }
System.print(top.try())
let aborter = Fiber.new || { try { Fiber.abort(err) } catch e { System.print("abort caught"); 99 } }
System.print(aborter.try())
System.print(aborter.error.isNone)
let untouched = Fiber.new || { System.print("CHILD ENTERED"); err.raise() }
try { untouched.call() } catch e { System.print(e.message) } ensure { System.print("guard cleanup") }
System.print(untouched.isDone)
System.print(untouched.try() == err)
