class Person {
  @class
  new(name, age) {
    const instance = new()
    instance.init(name, age)
    instance
  }

  init(_ name, _ age) {
    _name = name
    _age = age
  }

  name { _name }
  name=(put value) { _name = value }

  age { _age }
  age=(put value) { _age = value }

  ==(_ other) {
    self.name == other.name and self.age == other.age;
  }

  toString { "Person(name: \(name), age: \(age))" }
}

const person = Person.new(name: "Bob", age: 30);
System.print(person)
