// area: family
// spec: docs/spec/callables/family.md §1–2
// status: PASS

class Box {
  value { 11 }
  value() { 12 }
}
const b = Box.new()
const getter = (Box >> #value).bind(b)
const nullary = (Box >> #value()).bind(b)
System.print(getter())
System.print(nullary())
