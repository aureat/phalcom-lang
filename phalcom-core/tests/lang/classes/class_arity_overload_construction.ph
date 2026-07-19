// area: classes
// spec: classes.md; object-model.md; messages-and-selectors.md
// status: PASS

class Person {
  construct new(name) {
    _name = name
  }
  construct new(name, age) {
    _name = name
    _age = age
  }
  name => _name
  age => _age
}
const p1 = Person.new("Ada")
const p2 = Person.new("Grace", 36)
System.print(p1.name)
System.print(p1.age)
System.print(p2.name)
System.print(p2.age)
