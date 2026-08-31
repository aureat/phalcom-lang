// area: collections
// spec: docs/deferred/error-handling-followups.md §1 (G0 reentrancy lock, RULED 2026-07-20)
// status: NEGATIVE
// The `Set` twin of `map_key_hash_mutates_same_map_rejected.ph`: a `Set`
// binding reuses `Map`'s locate()/MapObject reentrancy lock. Pre-fix this was
// the *silent* corruption mode, not the panic mode: `add(_)`'s "already
// present" arm is a no-op, so the stale slot `==`'s side effect leaves behind
// was never dereferenced — `a` silently vanished (the malicious `remove`
// went through), `b` was reported "added" without ever landing, and the set
// ended up empty with exit 0, no diagnostic. Now the reentrant `remove` call
// itself is rejected, raising a catchable Error before either corruption can
// happen.

let s = Set.new()

class SK {
  @constructor
  new() {
    _triggered = false
  }
  hash { 0 }
  ==(_ other) {
    if (not _triggered) {
      _triggered = true
      s.remove(other)
    }
    return true
  }
}

const a = SK.new()
const b = SK.new()
s.add(a)
s.add(b)
System.print("unreachable")
