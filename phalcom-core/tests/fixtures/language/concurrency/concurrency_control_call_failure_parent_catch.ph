// area: concurrency
// spec: CONC002.C2.P1-R1 Gate C4
// status: PASS
// Proves parent call-site exception injection when child fiber fails under Call mode.

let child = Fiber.new || {
  Fiber.abort("child exploded")
}

let parent = Fiber.new || {
  let cleaned = false
  let action = || {
    let sub = || { child.call() }
    sub.ensure || {
      cleaned = true
    }
  }
  let result = action.on(Error) |e| {
    "parent caught: " + e.message + " (cleaned=" + cleaned.toString + ")"
  }
  result
}

System.print(parent.call())
