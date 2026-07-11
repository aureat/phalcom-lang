// area: classes
// spec: classes.md; object-model.md
// status: PASS

class Person {
  name => _name
  construct new(n) {
    _name = n
  }
  name=(value) {
    _name = value
  }
}
let p = Person.new("Ada")
p.name = "Bob"
System.print(p.name)
