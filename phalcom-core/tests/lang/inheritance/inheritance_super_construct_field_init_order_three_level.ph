// area: inheritance
// spec: ADR-0011 (fixed slot layout); object-model.md §5.1; ADR-0040
// status: PASS
// U-INH §3.5 extended to 3 levels: `super.new()` chains A -> B -> C, each
// level allocating its own field into its own fresh slot. All three fields
// survive independently on the final `C` instance, confirming field-init
// ordering holds transitively, not just for a single super-construct hop.

class A {
  construct new() { _a = "A" }
  a => _a
}
class B extends A {
  construct new() {
    super.new()
    _b = "B"
  }
  b => _b
}
class C extends B {
  construct new() {
    super.new()
    _c = "C"
  }
  c => _c
}
let x = C.new()
System.print(x.a)
System.print(x.b)
System.print(x.c)
