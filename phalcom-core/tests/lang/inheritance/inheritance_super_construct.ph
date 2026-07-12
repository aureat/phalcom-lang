// area: inheritance
// spec: object-model.md §5.1; ADR-0011 (slots); ADR-0040
// status: PASS
// U-INH §3.5: `super.construct(…)` runs the parent initializer on the SAME
// instance, so inherited slots are filled before the subclass's own.

class Animal {
  construct new(name) { _name = name }
  name => _name
}
class Dog extends Animal {
  construct new(name, breed) {
    super.new(name)
    _breed = breed
  }
  breed => _breed
}
let d = Dog.new("Rex", "Collie")
System.print(d.name)
System.print(d.breed)
