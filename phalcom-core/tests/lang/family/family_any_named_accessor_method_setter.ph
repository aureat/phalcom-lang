// area: family
// spec: docs/spec/callables/family.md §§1–3
// status: PASS
// `name...` is AnyNamed: it accepts the getter, setter, nullary method, and
// ordinary method selector kinds sharing the same base name.

class Box {
  @constructor
  new() { _value = 1 }
  value { _value }
  value() { _value + 10 }
  value(_ delta) { _value + delta }
  value=(put next) { _value = next }
}

const box = Box.new()
const family = box::value...
System.print(family.get())
System.print(family())
System.print(family(4))
family.set(9)
System.print(family.get())
