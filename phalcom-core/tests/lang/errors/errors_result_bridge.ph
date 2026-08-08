// area: errors
// spec: error-handling.md §5; result.md §2-§3
// status: PASS
// Graduated from `pending/` (U-CORE-6 §4 reserved this name): rewritten onto
// the ratified `attempt()`/`Result` surface. `Number.parse` does not exist
// (no such primitive was ever ratified), so this exercises the same
// throw -> value -> combinator pipeline (error-handling.md §5's worked
// example) over an ordinary `Error` throw instead.
class ParseFailure is Error {
  @constructor
  new(_ msg) { super.new(msg) }
}
const parsed = { throw ParseFailure.new("bad input") }.attempt()
System.print(parsed.map { n => n * 2 }.unwrapOr(0))
