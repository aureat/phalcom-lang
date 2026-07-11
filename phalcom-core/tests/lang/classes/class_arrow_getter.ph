// area: classes
// spec: classes.md; object-model.md
// status: PASS

class Circle {
  construct new(r) {
    _r = r
  }
  radius => _r
}
System.print(Circle.new(9).radius)
