// area: classes
// spec: classes.md; object-model.md
// status: PASS

class Circle {
  static new(r) {
    let c = self.new();
    c.init(r);
    return c;
  }
  init(r) {
    _r = r;
  }
  radius => _r
}
System.print(Circle.new(9).radius)
