// area: decorators
// spec: docs/spec/v0.2/decorators/ignore.md
// status: PASS
// `@ignore` on a `Getter` specifically (`toString` is a `SignatureKind::
// Getter`, ADR-0022's CB-1 amendment, so `Getter` is a load-bearing legal
// target, not an afterthought). Sending the ignored getter's selector raises
// `doesNotUnderstand`.

class Widget {
  @ignore label => "SHOULD NEVER RUN"
}

try {
  Widget.new().label
} on MessageNotUnderstood e {
  System.print(e.message)
}
