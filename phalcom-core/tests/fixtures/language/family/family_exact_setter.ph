// area: family
// spec: docs/spec/callables/family.md §1–2
// status: PASS

class Box {
  mut _value
  @constructor
  new() { _value = 1 }
  value { _value }
  value=(_ x) { _value = x }
}
let b = Box.new()
let setter = (Box >> #value=(_)).bind(b)
setter(8)
System.print(b.value)
