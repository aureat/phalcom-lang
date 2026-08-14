// area: family
// spec: docs/spec/callables/family.md §1–2
// status: PASS
// Exact setter Families use the setter lane through Family#set(_).

class Box {
  @constructor
  new() { _value = 1 }
  value { _value }
  value=(put x) { _value = x }
}
const b = Box.new()
const setter = b::value=(put)
setter.set(8)
System.print(b.value)
