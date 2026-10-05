// area: classes
// spec: classes.md; object-model.md
// status: PASS

class Person {
  mut _name
 name { _name }
  @constructor
  new(_ n) {
    _name = n
  }
  name=(_ value) {
    _name = value
  }
}
let p = Person.new("Ada")
p.name = "Bob"
System.print(p.name)
