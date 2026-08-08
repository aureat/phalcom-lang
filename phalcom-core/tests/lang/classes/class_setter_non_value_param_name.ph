// area: classes
// spec: classes.md; selectors.md
// status: PASS

class Person name => _name
  @constructor
  new(_ n) {
    _name = n
  }
  name=(put v) {
    _name = v
  }
}
const p = Person.new("Ada")
p.name = "Bob"
System.print(p.name)
