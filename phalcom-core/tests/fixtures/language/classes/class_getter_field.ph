// area: classes
// spec: classes.md; object-model.md
// status: PASS

class Box {
  mut _v
  @constructor
  new(_ v) {
    _v = v
  }
  value {
    return _v
  }
}
System.print(Box.new(42).value)
