// area: family
// spec: docs/spec/callables/family.md §3
// status: PASS

class Box {
  value() { 12 }
  value(_ x) { x + 1 }
}
const b = Box.new()
const family = (Box >> #value(...)).bind(b)
System.print(family())
System.print(family(4))
