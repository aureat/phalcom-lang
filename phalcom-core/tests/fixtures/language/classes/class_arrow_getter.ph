// area: classes
// spec: classes.md; object-model.md
// status: PASS

class Circle {
  @constructor
  new(_ r) {
    _r = r
  }
  radius { _r }
}
System.print(Circle.new(9).radius)
