// area: runtime-errors
// spec: error-handling.md §1, §4; ADR-0008
// status: NEGATIVE
// An uncaught `throw` renders the error's `message` via the trace and exits
// non-zero — the runtime half of the unified unwind (ADR-0008 §4.3).
class MyError is Error {
  @constructor
  new(_ msg) { super.new(msg) }
}
throw MyError.new("boom")
