// area: classes
// spec: classes.md; values-and-absence.md; ADR-0011
// status: PASS

class Person {
  mut _name
  mut _age
  @constructor
  new(name) { _name = name }
  name { _name }
  age { _age }
  age=(_ v) { _age = v }
}
let p = Person.new(name: "Ada")
System.print(p.age)
