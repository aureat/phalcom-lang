// area: family
// spec: docs/spec/callables/family.md §1–2
// status: PASS

class Box {
  value { 11 }
  value() { 12 }
}
let b = Box.new()
let getter = (Box >> #value).bind(b)
let nullary = (Box >> #value()).bind(b)
System.print(getter())
System.print(nullary())
