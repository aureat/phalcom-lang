// area: inheritance
// spec: method-lookup.md §1.14; ADR-0040
// status: PASS
// Adversarial extension of `super_two_level`: a 4-level chain
// `D extends C extends B extends A`, each level appending its own letter via
// `super`. Confirms `super` keeps resolving relative to each method's OWN
// defining class arbitrarily deep, not just for 2-3 levels.

class A {
  @constructor
  new() { }
  tag => "A"
}
class B is A {
  @constructor
  new() { }
  tag => super.tag + "B"
}
class C is B {
  @constructor
  new() { }
  tag => super.tag + "C"
}
class D is C {
  @constructor
  new() { }
  tag => super.tag + "D"
}
System.print(D.new().tag)
