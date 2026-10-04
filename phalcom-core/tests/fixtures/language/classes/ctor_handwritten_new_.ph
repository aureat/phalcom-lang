class Person {
  mut _name
  @constructor
  make(_ name) {
    _name = name
  }
  name { _name }
}
System.print(Person.make("Ada").name)
