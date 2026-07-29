// area: errors
// spec: error-handling.md §4; ADR-0008 §4.2
// status: PASS
// cleanup-supersedes: a `throw` inside an `ensure` cleanup block replaces the
// pending unwind — the cleanup's own outcome wins, not the original one.

class FirstErr is Error {
  @constructor
  new(msg) { super.new(msg) }
}
class SecondErr is Error {
  @constructor
  new(msg) { super.new(msg) }
}

const r = { { throw FirstErr.new("first") }.ensure { throw SecondErr.new("second") } }.on(Error) { e => e.message }
System.print(r)
