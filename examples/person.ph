class Person {
  // Named Constructor 1: Anonymous
  static anonymous() {
    let instance = self.new();
    instance.init("Anonymous", -1);
    return instance;
  }

  // Named Constructor 2: With name
  static new(name) {
    let instance = self.new();
    instance.init(name);
    return instance;
  }

  // Named Constructor 3: With name and age
  static new(name, age) {
    let instance = self.new();
    instance.init(name, age);
    return instance;
  }

  init(name, age) {
    _name = name;
    _age = age;
  }

  init(name) {
    _name = name;
  }

  name {
    return _name;
  }

  name=(value) {
    _name = value;
  }

  age {
    return _age;
  }

  age=(value) {
    _age = value;
  }

  ==(other) {
    return self.name == other.name and self.age == other.age;
  }
}

// Using default constructor
let person0 = Person.new();
System.print(person0);

// Using constructor with no arguments
let person1 = Person.anonymous();
System.print(person1.name); // nil
System.print(person1.age); // nil

// Using constructor with name
let person2 = Person.new("Alice");
System.print(person2.name); // Alice
System.print(person2.age); // nil

// Using constructor with name and age
let person3 = Person.new("Bob", 30);
System.print(person3.name); // Bob
System.print(person3.age); // 30

// Using getters and setters
person3.age = 31;
System.print(person3.age); // 31

