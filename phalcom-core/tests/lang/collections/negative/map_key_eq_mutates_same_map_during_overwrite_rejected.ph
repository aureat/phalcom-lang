// area: collections
// spec: docs/deferred/error-handling-followups.md §1 (G0 reentrancy lock, RULED 2026-07-20)
// status: NEGATIVE
// The corruption-mode shape from the followups doc's trigger sketch: two
// same-bucket keys already present, then overwriting the first (`at(k1, put:
// 99)`) locates it by scanning candidates and sends `==`, whose side effect
// removes a *different* live entry (`k2`) from the same map mid-scan. Before
// the reentrancy lock this silently swap-removed a neighboring slot and left
// the map's value in the wrong place with no error at all (exit 0). Now it
// must raise a catchable Error at the mutation site instead.

let m = Map.new()

class K {
  @constructor
  new() {
    _triggered = false
  }
  hash { 0 }
  ==(other) {
    if (not _triggered) {
      _triggered = true
      m.remove(k2)
    }
    return true
  }
}

const k1 = K.new()
const k2 = K.new()
m.at(k1, put: 1)
m.at(k2, put: 2)
m.at(k1, put: 99)
System.print("unreachable")
