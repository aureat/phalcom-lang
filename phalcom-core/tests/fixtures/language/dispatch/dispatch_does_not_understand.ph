// area: dispatch
// spec: method-lookup.md
// status: PASS

class Proxy {
  doesNotUnderstand(_ msg) {
    return "missing: " + msg.name;
  }
}
System.print(Proxy.new().frobnicate())
