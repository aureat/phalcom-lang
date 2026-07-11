// area: classes
// spec: classes.md; values-and-absence.md; ADR-0011
// status: PASS

class Person {
  construct new(name:) { _name = name }
  name => _name
  age => _age
  age=(v) { _age = v }
}
let p = Person.new(name: "Ada")
System.print(p.age)
