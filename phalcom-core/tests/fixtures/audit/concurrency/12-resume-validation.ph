let root = Fiber.current
let fresh = Fiber.new |x| { x }
let wrong = Fiber.new || { fresh.try() }
wrong.try()
System.print(wrong.error.isSome)
System.print(fresh.isDone)
System.print(fresh.call(42))
let doneCall = Fiber.new || { fresh.call() }
let doneTry = Fiber.new || { fresh.try() }
doneCall.try()
doneTry.try()
System.print(doneCall.error.isSome)
System.print(doneTry.error.isSome)
let failed = Fiber.new || { Error.new("failure").raise() }
failed.try()
let failedCall = Fiber.new || { failed.call() }
let failedTry = Fiber.new || { failed.try() }
failedCall.try()
failedTry.try()
System.print(failedCall.error.isSome)
System.print(failedTry.error.isSome)
let recursive = Fiber.new || { Fiber.current.try() }
recursive.try()
System.print(recursive.error.unwrapOr(None).message)
let ancestor = Fiber.new || { root.try() }
ancestor.try()
System.print(ancestor.error.unwrapOr(None).message)
let q = Fiber.new || { System.schedule(Fiber.current); 1 }
q.call()
System.print(System.nextScheduled.unwrapOr(None) == q)
System.print(q.isDone)
