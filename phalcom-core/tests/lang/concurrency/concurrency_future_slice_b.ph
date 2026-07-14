// area: concurrency
// spec: concurrency.md; ADR-0030
// status: PASS

// C-FUT-1: value/error await
let f1 = Future.value("ok")
System.print(f1.await)

let err = Error.new("rejected")
let f2 = Future.error(err)
try {
  f2.await
} catch e {
  System.print("caught: " + e.message)
}

// C-FUT-2: async/await suspending
let f3 = Future.async {
  System.print("async running")
  "async result"
}
System.print(f3.await)

// C-FUT-4: pending then/map/catch settlement
let f4 = Future.new()
let f5 = f4.then { v =>
  System.print("then run: " + v)
  "then result"
}
let f6 = f4.map { v =>
  System.print("map run: " + v)
  "map result"
}
let f7 = f4.catch { e =>
  System.print("catch run: " + e.message)
  "catch result"
}

f4.settleValue("settled")
System.print(f5.await)
System.print(f6.await)
System.print(f7.await) // should pass through fulfilled value without running catch

let f8 = Future.new()
let f9 = f8.catch { e =>
  System.print("catch run error: " + e.message)
  "catch recovered"
}
f8.settleError(Error.new("failed"))
System.print(f9.await)

// C-FUT-7: await under native frame raises CannotYieldAcrossNativeFrame
let f10 = Future.new()
let helper = Fiber.new {
  try {
    // some body
  } ensure {
    f10.await
  }
}
helper.try()
System.print("caught yield across native frame: " + helper.error.unwrapOr(None).message)
