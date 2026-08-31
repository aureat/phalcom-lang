// area: family
// spec: docs/spec/callables/family.md §1–2
// status: PASS

class Box {
  @constructor
  new() { _value = 1 }
  value { _value }
  value=(put x) { _value = x }
}
const b = Box.new()
const setter = (Box >> #value=(put)).bind(b)
setter(8)
System.print(b.value)
