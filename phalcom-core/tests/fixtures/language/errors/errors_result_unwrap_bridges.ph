// area: errors
// spec: result.md §3; error-handling.md §5
// status: PASS
// `unwrap` re-throws a caught `Err`; `ok()`/`okOr(_)` round-trip `Result` and
// `Option` (result.md §5 "absence <-> error").

class UErr is Error {
  @constructor
  new(_ msg) { super.new(msg) }
}

System.print(Ok.new(7).unwrap)
const caught = || { Err.new(UErr.new("nope")).unwrap }.on(Error) |e| { e.message }
System.print(caught)

System.print(Ok.new(9).ok().toString)
System.print(Err.new("e").ok().toString)
System.print(Some(9).okOr("missing").toString)
System.print(None.okOr("missing").toString)
