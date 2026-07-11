// area: classes
// spec: classes.md; object-model.md; messages-and-selectors.md
// status: PASS

class Person {
  static new(name) {
    let p = self.new();
    p.init(name);
    return p;
  }
  static new(name, age) {
    let p = self.new();
    p.init(name, age);
    return p;
  }
  init(name) {
    _name = name;
  }
  init(name, age) {
    _name = name;
    _age = age;
  }
  name => _name
  age => _age
}
let p1 = Person.new("Ada")
let p2 = Person.new("Grace", 36)
System.print(p1.name)
System.print(p1.age)
System.print(p2.name)
System.print(p2.age)
