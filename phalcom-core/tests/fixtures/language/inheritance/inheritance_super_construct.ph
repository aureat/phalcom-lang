// area: inheritance
// spec: object-model.md §5.1; ADR-0011 (slots); ADR-0040
// status: PASS
// U-INH §3.5: `super.construct(…)` runs the parent initializer on the SAME
// instance, so inherited slots are filled before the subclass's own.

class Animal {
  @constructor
  new(_ name) { _name = name }
  name { _name }
}
class Dog is Animal {
  @constructor
  new(_ name, _ breed) {
    super.new(name)
    _breed = breed
  }
  breed { _breed }
}
const d = Dog.new("Rex", "Collie")
System.print(d.name)
System.print(d.breed)
