class Person {
  // Named Constructor 1: Anonymous
  @class
  anonymous() {
    const instance = self.new();
    instance.init("Anonymous", -1);
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

  // Getter for name
  name {
    return _name;
  }

  // Setter for name
  name=(put value) {
    _name = value;
  }

  // Getter for age
  age {
    return _age;
  }

  // Setter for age
  age=(put value) {
    _age = value;
  }

  // Example of operator overloading: equality check
  ==(_ other) {
    return self.name == other.name and self.age == other.age;
  }
}

const person0 = Person.new();
System.print(person0.name);