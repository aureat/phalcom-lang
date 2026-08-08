// area: collections
// spec: docs/deferred/error-handling-followups.md §1 (G0 reentrancy lock, RULED 2026-07-20)
// status: PASS
// The G0 lock only rejects a key's hash/== *mutating the collection it is
// being compared for*; ordinary custom hash/== (reading fields, no side
// effects on the map/set) is unaffected — put/overwrite/remove/has all still
// work through the locked window, and a Set built over the same key class
// behaves identically.

class Pt {
  x => _x
  y => _y
  @constructor
  new(_ x, _ y) {
    _x = x
    _y = y
  }
  ==(_ other) {
    return _x == other.x and _y == other.y
  }
  hash {
    return _x * 31 + _y
  }
}

const m = Map.new()
const p1 = Pt.new(1, 2)
const p2 = Pt.new(1, 2)
const p3 = Pt.new(3, 4)
m.at(p1, put: "a")
System.print(m.size)
System.print(m[p2])
m.at(p2, put: "b")
System.print(m.size)
System.print(m[p1])
m.at(p3, put: "c")
System.print(m.size)
m.remove(p2)
System.print(m.size)
System.print(m.includes(p1))
System.print(m.includes(p3))

const s = Set.new()
s.add(Pt.new(5, 6))
s.add(Pt.new(5, 6))
System.print(s.size)
System.print(s.includes(Pt.new(5, 6)))
s.remove(Pt.new(5, 6))
System.print(s.size)
