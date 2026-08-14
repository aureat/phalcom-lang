// area: family/negative
// spec: docs/spec/callables/family.md §3
// status: NEGATIVE
// Pattern construction does not probe the receiver. A call with no matching
// route reaches ordinary target doesNotUnderstand.

class Box {}
const b = Box.new()
const family = b::missing(...)
System.print(family())
