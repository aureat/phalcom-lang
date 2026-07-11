// area: classes
// spec: classes.md; selectors.md
// status: PENDING

class Person {
  name => _name
  static new(n) {
    let p = self.new();
    p.setup(n);
    return p;
  }
  setup(n) {
    _name = n;
  }
  name=(v) {
    _name = v;
  }
}
let p = Person.new("Ada")
p.name = "Bob"
System.print(p.name)
