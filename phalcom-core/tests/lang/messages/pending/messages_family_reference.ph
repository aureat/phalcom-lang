// area: messages/selectors
// spec: selectors.md
// status: PENDING

class Point {
  x => _x
  @class
  new(_ x) {
    const p = self.new();
    p.init(x);
    return p;
  }
  init(_ x) {
    _x = x;
  }
  move(_ to) {
    return Point.new(_x + to);
  }
}
const p = Point.new(1)
const f = p::move
System.print(f(4).x)
