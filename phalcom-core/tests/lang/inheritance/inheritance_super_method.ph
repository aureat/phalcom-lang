// area: inheritance
// spec: method-lookup.md §1.14; ADR-0040 (SuperSend)
// status: PASS
// U-INH §3.4: `super.sel` runs the superclass's definition with `self` still
// bound to the subclass instance, so an override can extend inherited behaviour.

class Animal {
  construct new() { }
  speak => "generic"
}
class Dog extends Animal {
  construct new() { }
  speak => super.speak + " woof"
}
System.print(Dog.new().speak)
