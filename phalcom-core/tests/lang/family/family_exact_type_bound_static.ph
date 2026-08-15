// area: family
// spec: docs/spec/callables/family.md §1 and §2
// status: PASS
// A class expression can bind an exact nullary class-side Family. Exact `::`
// selector specs are written directly, without the first-class-symbol `#`.

class Point {
  @class
  square() { return "Point.square" }
}
const f = Point::square()
System.print(f())
