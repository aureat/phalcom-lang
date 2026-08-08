class Person {
  init(_ name) { _name = name }
  @class
  make(_ name) {
    let instance = self.new_()
    instance.init(name)
    return instance
  }
  name => _name
}
System.print(Person.make("Ada").name)
