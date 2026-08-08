class Person {
  // Named Constructor 1: Anonymous
  @class
  anonymous() {
    const instance = self.new();
    instance.init("Anonymous");
    return instance;
  }

  // Named Constructor 2: With name
  @class
  new(_ name) {
    const instance = self.new();
    instance.init(name);
    return instance;
  }

  // Named Constructor 3: With name and age
  @class
  new(_ name, _ age) {
    const instance = self.new();
    instance.init(name, age);
    return instance;
  }

  init(_ name, _ age) {
    _name = name;
    _age = age;
  }

  init(_ name) {
    _name = name;
  }

  name {
    return _name;
  }

  name=(put value) {
    _name = value;
  }

  age {
    return _age;
  }

  age=(put value) {
    _age = value;
  }

  ==(_ other) {
    return self.name == other.name and self.age == other.age;
  }
}

// Using default constructor
const person0 = Person.new();
System.print(person0);

// Using constructor with no arguments
const person1 = Person.anonymous();
System.print(person1.name); // Anonymous
System.print(person1.age); // nil

// Using constructor with name
const person2 = Person.new("Alice");
System.print(person2.name); // Alice
System.print(person2.age); // nil

// Using constructor with name and age
const person3 = Person.new("Bob", 30);
System.print(person3.name); // Bob
System.print(person3.age); // 30

// Using getters and setters
person3.age = 31;
System.print(person3.age); // 31

