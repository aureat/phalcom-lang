// area: concurrency
// spec: CONC002.C2.P1-R1 Gate C4
// status: PASS
// Proves parent call-site exception injection when child fiber fails under Call mode.

let mut child = Fiber.new || {
  Fiber.abort(Error.new("child exploded"))
}

let mut parent = Fiber.new || {
  let mut cleaned = false
  let mut action = || {
    let mut sub = || { child.call() }
    sub.ensure || {
      cleaned = true
    }
  }
  let mut result = action.on(Error) |e| {
    "parent caught: " + e.message + " (cleaned=" + cleaned.toString + ")"
  }
  result
}

System.print(parent.call())
