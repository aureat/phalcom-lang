// area: classes
// spec: classes.md; object-model.md
// status: PASS
class Person {
  @constructor
  new(name, age) {
    _name = name
    _age = age
  }

  name { _name }
}

System.print(Person.new(name: "Ada", age: 7).name)
