// area: family
// spec: selectors.md §3 (Pinned form, bound to a class object); ADR-0047
// status: PASS
// `Type::#square()` pins a zero-arg static selector on the class object
// itself — the same bound-`::` grammar as `obj::#sel(...)`, just with a
// class as the receiver (mirrors `family_type_bound_static.ph`'s Open case).

class Point {
  static square() { return "Point.square" }
}
let f = Point::#square()
System.print(f())
