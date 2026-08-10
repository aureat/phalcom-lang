// area: decorators
// spec: docs/spec/v0.2/decorators/native.md
// status: PASS
// `@native` drops its member just like `@ignore` (native.md's drop is a
// provisional borrow of ignore.md's mechanism). `frobnicate()` names no real
// native binding, so the observable behaviour for a user class is that the
// selector simply does not resolve — `doesNotUnderstand`. A sibling member
// still compiles and works.

class Gadget {
  @native frobnicate() {
    System.print("SHOULD NEVER RUN")
  }

  ok { "fine" }
}

const g = Gadget.new()
System.print(g.ok)

try {
  g.frobnicate()
} on MessageNotUnderstood e {
  System.print(e.message)
}
