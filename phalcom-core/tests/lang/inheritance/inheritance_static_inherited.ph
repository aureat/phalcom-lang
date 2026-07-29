// area: inheritance
// spec: object-model.md §5 rule 4; ADR-0002 (parallel metaclass)
// status: PASS
// U-INH §3.3: a subclass's metaclass superclass is wired to the superclass's
// metaclass, so a `static` member defined on the superclass is reachable on
// the subclass (the parallel-metaclass rule that makes static inheritance work).

class Animal {
  @constructor
  new() { }
  static kingdom => "Animalia"
}
class Dog is Animal {
  @constructor
  new() { }
}
System.print(Dog.kingdom)
