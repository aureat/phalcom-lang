// area: messages/selectors
// spec: selectors.md
// status: PENDING

class Point {
  x => _x
  static new(x) {
    let p = self.new();
    p.init(x);
    return p;
  }
  init(x) {
    _x = x;
  }
  move(to) {
    return Point.new(_x + to);
  }
}
let p = Point.new(1)
let f = p::move
System.print(f(4).x)
