const err = Error.new("same")
let leak = || { 0 }
const inner = Fiber.new || { err.raise() }
const middle = Fiber.new || {
  let x = 77
  leak = || { x }
  inner.call()
  System.print("BAD-after-handler")
}
const outer = Fiber.new || { middle.call(); System.print("BAD-after-middle") }
System.print(outer.try() == err)
System.print(inner.error.unwrapOr(None) == err)
System.print(middle.error.unwrapOr(None) == err)
System.print(outer.error.unwrapOr(None) == err)
System.print(leak.call())
const b = Fiber.new || { err.raise() }
const a = Fiber.new || { System.print(b.try() == err); "A survived" }
System.print(a.try())
const d = Fiber.new || { err.raise() }
const c = Fiber.new || { d.call(); "BAD" }
const top = Fiber.new || { System.print(c.try() == err); "top survived" }
System.print(top.try())
const aborter = Fiber.new || { try { Fiber.abort(err) } catch e { System.print("abort caught"); 99 } }
System.print(aborter.try())
System.print(aborter.error.isNone)
const untouched = Fiber.new || { System.print("CHILD ENTERED"); err.raise() }
try { untouched.call() } catch e { System.print(e.message) } ensure { System.print("guard cleanup") }
System.print(untouched.isDone)
System.print(untouched.try() == err)
