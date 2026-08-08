// area: classes
// spec: classes.md; object-model.md
// status: PASS

class Person { name => _name
  @constructor
  new(_ n) {
    _name = n
  }
  name=(put value) {
    _name = value
  }
}
const p = Person.new("Ada")
p.name = "Bob"
System.print(p.name)
