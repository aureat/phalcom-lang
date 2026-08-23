// area: decorators
// spec: docs/spec/current/decorators/ignore.md
// status: PASS
// `@ignore` drops a `Getter` member. The observable behaviour is that the
// selector does not resolve.

class Gizmo {
  @ignore shown { "SHOULD NEVER RUN" }
}

try {
  Gizmo.new().shown
} on MessageNotUnderstood e {
  System.print(e.message)
}
