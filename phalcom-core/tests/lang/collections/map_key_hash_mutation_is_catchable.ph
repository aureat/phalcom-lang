// area: collections
// spec: docs/deferred/error-handling-followups.md §1 (G0 reentrancy lock, RULED 2026-07-20)
// status: PASS
// The G0 reentrancy lock raises an ordinary RuntimeError (ConcurrentMutation),
// not a Raise — so, per the existing wrap-and-probe behavior
// (primitive/block.rs), `.on(Error) { }` catches it like any other built-in
// failure. The map is left exactly as it was before the aborted `at(_,put:)`
// call: the offending key's removal never went through.

let m = Map.new()

class BadKey {
  construct new() {
    _triggered = false
  }
  hash { 0 }
  ==(other) {
    if (not _triggered) {
      _triggered = true
      m.remove(other)
    }
    return true
  }
}

const a = BadKey.new()
const b = BadKey.new()
m.at(a, put: 1)
const caught = { m.at(b, put: 2) }.on(Error) { e => e.message }
System.print(caught)
System.print(m.size)
System.print(m.includes(a))
