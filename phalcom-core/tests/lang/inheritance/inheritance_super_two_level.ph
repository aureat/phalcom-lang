// area: inheritance
// spec: method-lookup.md §1.14; ADR-0040
// status: PASS
// U-INH §3.4: a two-level chain `C extends B extends A` resolves `super`
// relative to each method's OWN defining class, so C#tag → B#tag → A#tag.

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
System.print(C.new().tag)
