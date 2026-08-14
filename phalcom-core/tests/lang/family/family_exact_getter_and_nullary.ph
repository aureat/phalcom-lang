// area: family
// spec: docs/spec/callables/family.md §1–2
// status: PASS
// Exact getter and exact nullary method references retain distinct selector
// shapes and use their matching Family gateways.

class Box {
  value { 11 }
  value() { 12 }
}
const b = Box.new()
const getter = b::value
const nullary = b::value()
System.print(getter.get())
System.print(nullary())
