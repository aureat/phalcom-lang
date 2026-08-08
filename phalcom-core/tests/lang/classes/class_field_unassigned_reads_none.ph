// area: classes
// spec: classes.md; values-and-absence.md; ADR-0011
// status: PASS

class Person {
  @constructor
  new(name) { _name = name }
  name => _name
  age => _age
  age=(put v) { _age = v }
}
const p = Person.new(name: "Ada")
System.print(p.age)
