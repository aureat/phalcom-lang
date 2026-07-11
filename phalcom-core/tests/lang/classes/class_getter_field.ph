// area: classes
// spec: classes.md; object-model.md
// status: PASS

class Box {
  static new(v) {
    let b = self.new();
    b.init(v);
    return b;
  }
  init(v) {
    _v = v;
  }
  value {
    return _v;
  }
}
System.print(Box.new(42).value)
