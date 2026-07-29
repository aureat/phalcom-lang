// area: collections
// spec: docs/deferred/error-handling-followups.md §1 (G0 reentrancy lock, RULED 2026-07-20)
// status: NEGATIVE
// A key's `==` cannot structurally mutate the Map it is being compared for —
// locate() locks the collection for the duration of each reentrant hash/==
// send, so a same-map remove() called from inside `==` must raise a
// catchable Error at the mutation call site, never corrupt the map's slot
// index and never abort the process (the pre-fix panic mode: a stale slot
// from locate() indexed after the map shrank underneath it).
//
// A reentrancy-guard flag (`_triggered`) keeps the malicious `==` from
// recursing into itself when `remove` re-locates the same bucket.

let m = Map.new()

class BadKey {
  @constructor
  new() {
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
m.at(b, put: 2)
System.print("unreachable")
