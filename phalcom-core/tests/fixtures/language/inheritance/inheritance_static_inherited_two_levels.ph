// area: inheritance
// spec: object-model.md §5 rule 4; ADR-0002 (parallel metaclass)
// status: PASS
// Adversarial extension of `static_inherited`: the static member lives on the
// GRANDPARENT (`Grand`), and the middle class (`Animal`) defines no static
// members at all. `Dog.kingdom` must walk the parallel-metaclass superclass
// chain transitively (Dog class -> Animal class -> Grand class), not just one
// hop, to find it.

class Grand {
  @constructor
  new() { }
  @class
  kingdom { "Animalia" }
}
class Animal is Grand {
  @constructor
  new() { }
}
class Dog is Animal {
  @constructor
  new() { }
}
System.print(Dog.kingdom)
