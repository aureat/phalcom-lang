// area: runtime-errors
// spec: method-lookup.md §1.14; U-CORE-6 (MessageNotUnderstood); U-INH §3.4
// status: NEGATIVE
// U-INH: a `super` send that walks off the top of the chain routes to the same
// doesNotUnderstand → MessageNotUnderstood path as an ordinary miss (not a panic).
class A {
  @constructor
  new() { }
}
class B is A {
  @constructor
  new() { }
  go { super.missingThing() }
}
System.print(B.new().go)
