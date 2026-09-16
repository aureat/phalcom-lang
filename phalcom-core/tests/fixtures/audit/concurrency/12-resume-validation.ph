const root = Fiber.current
const fresh = Fiber.new |x| { x }
const wrong = Fiber.new || { fresh.try() }
wrong.try()
System.print(wrong.error.isSome)
System.print(fresh.isDone)
System.print(fresh.call(42))
const doneCall = Fiber.new || { fresh.call() }
const doneTry = Fiber.new || { fresh.try() }
doneCall.try()
doneTry.try()
System.print(doneCall.error.isSome)
System.print(doneTry.error.isSome)
const failed = Fiber.new || { Error.new("failure").raise() }
failed.try()
const failedCall = Fiber.new || { failed.call() }
const failedTry = Fiber.new || { failed.try() }
failedCall.try()
failedTry.try()
System.print(failedCall.error.isSome)
System.print(failedTry.error.isSome)
const recursive = Fiber.new || { Fiber.current.try() }
recursive.try()
System.print(recursive.error.unwrapOr(None).message)
const ancestor = Fiber.new || { root.try() }
ancestor.try()
System.print(ancestor.error.unwrapOr(None).message)
const q = Fiber.new || { System.schedule(Fiber.current); 1 }
q.call()
System.print(System.nextScheduled.unwrapOr(None) == q)
System.print(q.isDone)
