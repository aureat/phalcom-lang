// area: concurrency
// spec: concurrency.md; ADR-0030
// status: PASS

// C-FUT-1: value/error await
const f1 = Future.value("ok")
System.print(f1.await)

const err = Error.new("rejected")
const f2 = Future.error(err)
try {
  f2.await
} catch e {
  System.print("caught: " + e.message)
}

// C-FUT-2: async/await suspending
const f3 = Future.async {
  System.print("async running")
  "async result"
}
System.print(f3.await)

// C-FUT-4: pending then/map/catch settlement
const f4 = Future.new()
const f5 = f4.then { v =>
  System.print("then run: " + v)
  "then result"
}
const f6 = f4.map { v =>
  System.print("map run: " + v)
  "map result"
}
const f7 = f4.catch { e =>
  System.print("catch run: " + e.message)
  "catch result"
}

f4.settleValue("settled")
System.print(f5.await)
System.print(f6.await)
System.print(f7.await) // should pass through fulfilled value without running catch

const f8 = Future.new()
const f9 = f8.catch { e =>
  System.print("catch run error: " + e.message)
  "catch recovered"
}
f8.settleError(Error.new("failed"))
System.print(f9.await)

// C-FUT-7: await under native frame raises CannotYieldAcrossNativeFrame
const f10 = Future.new()
const helper = Fiber.new {
  try {
    // some body
  } ensure {
    f10.await
  }
}
helper.try()
System.print("caught yield across native frame: " + helper.error.unwrapOr(None).message)
