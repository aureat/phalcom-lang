// area: family
// spec: docs/spec/callables/family.md §§1–3
// status: PASS

class Box {
  mut _value
  @constructor
  new() { _value = 1 }
  value { _value }
  value=(_ next) { _value = next }
}

let box = Box.new()
let setter = (Box >> #value=(_)).bind(box)
setter(12)
System.print(box.value)
