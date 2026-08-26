class Person {
  @constructor new() {}
  greet() { "hello" }
  name { "Ada" }
  rename(_ value) { value }
}

const person = Person.new()
person./*@completion*/greet()
