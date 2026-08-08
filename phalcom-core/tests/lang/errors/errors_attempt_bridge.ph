// area: errors
// spec: error-handling.md §5; result.md §3
// status: PASS
// `Block#attempt()` — the throw -> value bridge: success is `Ok(v)`; a caught
// `throw` becomes `Err(e)`. Composes with `Result`'s combinators
// (error-handling.md §5's worked example).

class AttErr is Error {
  @constructor
  new(_ msg) { super.new(msg) }
}

System.print(|| { 21 * 2 }.attempt().toString)
System.print(|| { throw AttErr.new("x") }.attempt().toString)
System.print(|| { 21 * 2 }.attempt().map |n| { n + 1 }.unwrapOr(0))
System.print(|| { throw AttErr.new("x") }.attempt().map |n| { n + 1 }.unwrapOr(0))
