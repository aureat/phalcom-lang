// area: rest
// spec: F.3-rest-capture-and-rest-pattern-dispatch-amended.md §20
// status: PASS
// A miss with no rest entry falls through to `doesNotUnderstand(_)`.

class Proxy3 {
  doesNotUnderstand(_ msg) {
    return "proxied"
  }
}
const p = Proxy3.new()
System.print(p.bogus(1, 2, 3))
