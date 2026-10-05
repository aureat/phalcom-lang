class Person {
  @constructor new() {}
  greet() { "hello" }
  name { "Ada" }
  rename(_ value) { value }
}

let person = Person.new()
person./*@completion*/greet()
