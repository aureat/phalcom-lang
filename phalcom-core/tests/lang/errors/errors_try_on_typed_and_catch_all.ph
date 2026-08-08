// area: errors
// spec: error-handling.md §2; ADR-0031
// status: PASS
// `try`/`on T e {}`/`catch e {}` desugar to `.on(T){e=>}`/`.on(Error){e=>}`
// (ADR-0031 §3). A typed `on` catches its class (and subclasses); an
// untyped `catch` is the catch-all, since `Error` is the raisable root.

class ParseError is Error {
  @constructor
  new(_ msg) { super.new(msg) }
}
class RangeErrorLike is Error {
  @constructor
  new(_ msg) { super.new(msg) }
}

try {
  throw ParseError.new("bad token")
} on ParseError e {
  System.print("parse: " + e.message)
} on RangeErrorLike e {
  System.print("range: " + e.message)
}

try {
  throw RangeErrorLike.new("out of bounds")
} on ParseError e {
  System.print("parse: " + e.message)
} catch e {
  System.print("other: " + e.message)
}
