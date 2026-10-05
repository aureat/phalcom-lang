// area: family
// spec: docs/spec/callables/family.md §2 and docs/spec/current/classes.md
// status: PASS

class Animal {
  speak() { return "..." }
}
class Dog is Animal {
}
let d = Dog.new()
let f = (Dog >> #speak()).bind(d)
System.print(f())
