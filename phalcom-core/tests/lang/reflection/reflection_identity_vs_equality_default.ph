// area: reflection
// spec: object-model.md §4; ADR-0015; Object#hash (object.rs)
// status: PASS
// A user class that overrides neither `==` nor `hash` falls back to
// `Object`'s default: `==` is identity (a fresh instance is never `==` to
// another, structurally-identical-looking, fresh instance), a variable
// bound to the SAME instance IS `==` to itself, and `hash` agrees with `==`
// (R-INV-1.3): equal receivers (same object) hash equal.

class Point {
  construct new(x) { _x = x }
}
const a = Point.new(1)
const b = Point.new(1)
const c = a
System.print(a == b)
System.print(a == c)
System.print(a == a)
System.print(a.hash == c.hash)
