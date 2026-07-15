// area: decorators
// spec: docs/spec/v0.2/decorators/native.md
// status: PASS
// `@native` on a `Getter` specifically — `toString` is a
// `SignatureKind::Getter` (native.md's motivating case, ADR-0022's CB-1
// amendment), so `Getter` is load-bearing among `@native`'s legal targets,
// not an afterthought. No native binding exists for `Gizmo#shown`, so the
// observable behaviour is that the selector does not resolve.

class Gizmo {
  @native shown => "SHOULD NEVER RUN"
}

try {
  Gizmo.new().shown
} on MessageNotUnderstood e {
  System.print(e.message)
}
