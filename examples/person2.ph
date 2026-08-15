class Person {
  @constructor
  new() {
    _name = None
  }

  name { _name }
}

const person = Person.new()
System.print(person.name)
