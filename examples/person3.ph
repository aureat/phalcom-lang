class Person {
  @class
  new(_ name, _ age) {
    const instance = self.new(); // super.new()
    instance.init(name, age);
    return instance;
  }

  @constructor
  new(name, age) {
    _name = name
    _age = age
  }

  name {
    return _name;
  }

  name=(put value) {
    _name = value;
  }

  age {
    _age;
  }

  age=(put value) {
    _age = value;
  }

  ==(_ other) {
    return self.name == other.name and self.age == other.age;
  }
}

const person3 = Person.new("Bob", 30);
// person3.age = 31;
// System.print(person3.name); // Bob
System.print(person3.age); // 30
//
// person3.age = 31;
// System.print(person3.age); // 31
