// area: inheritance
// spec: method-lookup.md §1.14; ADR-0040
// status: PASS
// U-INH §3.4: `super` from C (defining class C) starts above C. A selector
// only A defines is found by walking B (which does not define it) up to A —
// `super` does not require the immediate parent to implement it.

class A {
  construct new() { }
  origin => "from A"
}
class B extends A {
  construct new() { }
}
class C extends B {
  construct new() { }
  origin => super.origin + " via C"
}
System.print(C.new().origin)
