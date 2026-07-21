class Person {
  init(name) { _name = name }
  @class
  make(name) {
    let instance = self.new_()
    instance.init(name)
    return instance
  }
  name => _name
}
System.print(Person.make("Ada").name)
