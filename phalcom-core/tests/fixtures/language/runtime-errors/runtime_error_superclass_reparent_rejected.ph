// area: runtime-errors
// spec: object-model.md §1.5/§5; ADR-0011; ADR-0041 (DEC-U13a=A, sealed hierarchy)
// status: NEGATIVE
// U13: a class's `superclass` is sealed at class creation — a runtime
// `superclass=` reparent is rejected with a clean, catchable error, never a
// panic, so ADR-0011's fixed instance slot layout and `ClassId`-keyed
// dispatch never shift under a live instance. Method reopening (adding a
// method to an existing class) is unaffected by the seal — see
// `classes/class_method_reopen_after_definition.ph`.
class A {
  @constructor
  new() { }
}
class B is A {
  @constructor
  new() { }
}
B.superclass = Object
