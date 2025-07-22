class Person {
  static new(name, age) {
    let instance = self.new();
    instance.init(name, age);
    return instance;
  }

  init(name, age) {
    _name = name;
    _age = age;
  }

  name {
    return _name;
  }

  name=(value) {
    _name = value;
  }

  age {
    _age;
  }

  age=(value) {
    _age = value;
  }

  ==(other) {
    return self.name == other.name and self.age == other.age;
  }
}

let person3 = Person.new("Bob", 30);
// person3.age = 31;
// System.print(person3.name); // Bob
System.print(person3.age); // 30
//
// person3.age = 31;
// System.print(person3.age); // 31