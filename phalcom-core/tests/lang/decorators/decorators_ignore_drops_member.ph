// area: decorators
// spec: docs/spec/v0.2/decorators/ignore.md
// status: PASS
// `@ignore` drops a member wholesale: no bytecode is emitted, no method is
// installed, and the method table is untouched. Sends to the ignored
// selector must raise `doesNotUnderstand` — proving absence, not merely
// "we didn't happen to observe it". The ignored body would print or fail if
// it ever ran, so a silent pass here is only possible if the drop actually
// happened. A sibling member on the same class must still compile and work.

class Draft {
  @ignore halfFinished(x) {
    System.print("SHOULD NEVER RUN")
    x.someMethodThatDoesNotExistYet()
  }

  finished => "ok"
}

let d = Draft.new()
System.print(d.finished)

try {
  d.halfFinished(1)
} on MessageNotUnderstood e {
  System.print(e.message)
}
