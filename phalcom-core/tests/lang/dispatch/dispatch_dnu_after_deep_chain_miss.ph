// area: dispatch
// spec: method-lookup.md §2; ADR-0012
// status: PASS
// `doesNotUnderstand(_:)` is defined only on the top ancestor `A`. A message
// the chain DOES understand (`greet`, defined on the leaf `C`) dispatches
// normally without ever reaching DNU. Only a message the FULL 3-level chain
// misses falls through to `A`'s DNU handler — proving DNU itself is found by
// ordinary inherited lookup, and is only invoked after every level of the
// chain has failed to match.

class A {
  @constructor
  new() { }
  doesNotUnderstand(msg) {
    System.print("DNU: " + msg.name)
    return None
  }
}
class B is A {
  @constructor
  new() { }
}
class C is B {
  @constructor
  new() { }
  greet => "hi"
}
const c = C.new()
System.print(c.greet)
c.mystery()
