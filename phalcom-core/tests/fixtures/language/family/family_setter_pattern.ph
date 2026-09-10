// area: family
// spec: docs/spec/callables/family.md §§1–3
// status: PASS

class Box {
  @constructor
  new() { _value = 1 }
  value { _value }
  value=(_ next) { _value = next }
}

const box = Box.new()
const setter = (Box >> #value=(_)).bind(box)
setter(12)
System.print(box.value)
