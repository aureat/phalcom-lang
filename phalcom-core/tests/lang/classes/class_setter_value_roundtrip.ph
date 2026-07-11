// area: classes
// spec: classes.md; object-model.md
// status: PASS

class Person {
  name => _name
  static new(n) {
    let p = self.new();
    p.init(n);
    return p;
  }
  init(n) {
    _name = n;
  }
  name=(value) {
    _name = value;
  }
}
let p = Person.new("Ada")
p.name = "Bob"
System.print(p.name)
