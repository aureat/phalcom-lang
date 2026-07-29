// area: classes
// spec: classes.md; object-model.md
// status: PASS

class Person {
  name => _name
  @constructor
  new(n) {
    _name = n
  }
  name=(value) {
    _name = value
  }
}
const p = Person.new("Ada")
p.name = "Bob"
System.print(p.name)
