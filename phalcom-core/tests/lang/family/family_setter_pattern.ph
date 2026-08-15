// area: family
// spec: docs/spec/callables/family.md §§1–3
// status: PASS
// The hashless setter pattern `name=...` accepts setter shape and remains a
// live Family over the receiver.

class Box {
  @constructor
  new() { _value = 1 }
  value { _value }
  value=(put next) { _value = next }
}

const box = Box.new()
const setter = box::value=...
setter.set(12)
System.print(box.value)
