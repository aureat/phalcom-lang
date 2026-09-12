// area: concurrency
// spec: concurrency.md; ADR-0030 §3
// status: PASS
// CONC002.C1.P4: manual Fiber protocol hostile cases.
// Proves:
// 1. Initial entry arguments and post-yield resume arguments are independent.
// 2. Distinct yield sites can yield and receive heterogeneous types.
// 3. Error/None/Unit yielded or returned as data are distinct from terminal failure.

// Case 1: Entry input differs from resume input
const f1 = Fiber.new |initial| {
  System.print("f1 entry: " + initial.toString)
  const response = Fiber.yield("ready")
  System.print("f1 resumed: " + response.toString)
  true
}
System.print("f1 call 1: " + f1.call(41).toString)
System.print("f1 call 2: " + f1.call("continue").toString)

// Case 2: Heterogeneous multi-yield
const f2 = Fiber.new || {
  const first = Fiber.yield(1)
  System.print("f2 yield 1 received: " + first.toString)
  const second = Fiber.yield("two")
  System.print("f2 yield 2 received: " + second.toString)
  42
}
System.print("f2 call 1: " + f2.call().toString)
System.print("f2 call 2: " + f2.call("one").toString)
System.print("f2 call 3: " + f2.call(true).toString)

// Case 3: Error, None, and Unit as data vs terminal failure
const fData = Fiber.new || {
  const yieldedErr = Fiber.yield(Error.new("yielded-data"))
  System.print("fData received after err: " + yieldedErr.toString)
  Error.new("returned-data")
}

const errVal1 = fData.call()
System.print("fData yielded: " + errVal1.message)
System.print("fData isDone after yield: " + fData.isDone.toString)
System.print("fData error after yield isNone: " + fData.error.isNone.toString)

const errVal2 = fData.call("resumed-with-string")
System.print("fData returned: " + errVal2.message)
System.print("fData isDone after return: " + fData.isDone.toString)
System.print("fData error after return isNone: " + fData.error.isNone.toString)

// Distinguish from genuine failure via abort
const fFail = Fiber.new || {
  Fiber.abort(Error.new("genuine-failure"))
}
const tryResult = fFail.try()
System.print("fFail try returned: " + tryResult.message)
System.print("fFail isDone: " + fFail.isDone.toString)
System.print("fFail error isSome: " + fFail.error.isSome.toString)
System.print("fFail error message: " + fFail.error.unwrapOr(Error.new("none")).message)
