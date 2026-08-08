// area: classes
// spec: classes.md; selectors.md; object-model.md
// status: PASS

// Two `new` constructors are distinguished by SELECTOR, not arity hacks:
// `new(name:city:)` and `new(name:)` are two distinct Initializer selectors.
class Person {
  @constructor
  new(name, city) { _name = name; _city = city }
  @constructor
  new(name) { _name = name; _city = "Unknown" }
  city => _city
}
System.print(Person.new(name: "Ada", city: "London").city)
System.print(Person.new(name: "Bob").city)
