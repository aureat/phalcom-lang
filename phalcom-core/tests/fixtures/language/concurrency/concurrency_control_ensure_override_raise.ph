// area: concurrency
// spec: CONC002.C2.P1-R1
// status: PASS
// Proves that when ensure cleanup raises, it supersedes the prior raise.

let mut action = || {
  let mut body = || { throw Error.new("body failure") }
  body.ensure || {
    throw Error.new("cleanup override failure")
  }
}

let mut result = action.on(Error) |e| {
  "caught: " + e.message
}

System.print(result)
