class Person {
  @constructor
  make(_ name) {
    _name = name
  }
  name => _name
}
System.print(Person.make("Ada").name)
