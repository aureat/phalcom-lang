// area: dispatch
// spec: method-lookup.md §2; messages-and-selectors.md §5; ADR-0012
// status: PASS
// A Proxy overriding `doesNotUnderstand(_:)` reflectively forwards every
// missed send to a wrapped target via `perform(_:_:)` — the canonical
// proxy/forwarding mechanism. Both a getter (`greet`) and a two-arg method
// (`add`) round-trip through the miss path.

class Target { greet { "hello" }
  add(_ x, _ y) {
    return x + y
  }
}
class Proxy {
  @constructor
  new(_ t) {
    _target = t
  }
  doesNotUnderstand(_ msg) {
    return _target.perform(msg.selector, ***msg.args)
  }
}
const p = Proxy.new(Target.new())
System.print(p.greet)
System.print(p.add(2, 3))
