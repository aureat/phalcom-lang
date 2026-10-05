// area: classes
// spec: classes.md; selectors.md
// status: PASS

class Person {
  mut _name
 name { _name }
  @constructor
  new(_ n) {
    _name = n
  }
  name=(_ v) {
    _name = v
  }
}
let p = Person.new("Ada")
p.name = "Bob"
System.print(p.name)
