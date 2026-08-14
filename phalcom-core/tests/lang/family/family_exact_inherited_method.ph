// area: family
// spec: docs/spec/callables/family.md §2 and docs/spec/current/classes.md
// status: PASS
// An exact Family can target a method inherited by the bound receiver.

class Animal {
  speak() { return "..." }
}
class Dog is Animal {
}
const d = Dog.new()
const f = d::speak()
System.print(f())
