// area: errors
// spec: error-handling.md §4; ADR-0008 §4.1
// status: PASS
// `ensure` fires on all three exit paths: (a) normal completion, (b) a
// non-local `return` unwinding through the protected block, (c) a caught
// `throw`. Observable side-effect order pins each.

class EErr extends Error {
  construct new(msg) { super.new(msg) }
}

// (a) normal completion
{ System.print("normal-body") }.ensure { System.print("normal-cleanup") }

// (b) non-local return through the protected block
class M {
  run() {
    { return "early" }.ensure { System.print("return-cleanup") }
    return "unreached"
  }
}
System.print(M.new().run())

// (c) a caught throw (ensure runs, then the outer `on` catches)
try {
  { throw EErr.new("boom") }.ensure { System.print("throw-cleanup") }
} catch e {
  System.print("caught: " + e.message)
}
