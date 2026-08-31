// area: decorators
// spec: docs/spec/current/decorators/ignore.md
// status: PASS
// `@ignore` drops its member. A sibling member still compiles and works.

class Gadget {
  @ignore frobnicate() {
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
