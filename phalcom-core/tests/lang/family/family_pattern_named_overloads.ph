// area: family
// spec: docs/spec/callables/family.md §3
// status: PASS
// A named structural pattern routes incoming nullary and unary shapes to the
// corresponding current overload.

class Box {
  value() { 12 }
  value(_ x) { x + 1 }
}
const b = Box.new()
const family = b::value(...)
System.print(family())
System.print(family(4))
