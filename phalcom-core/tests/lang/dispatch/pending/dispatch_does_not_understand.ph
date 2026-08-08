// area: dispatch
// spec: method-lookup.md
// status: PENDING

class Proxy {
  doesNotUnderstand(_ msg) {
    return "missing: " + msg.name;
  }
}
System.print(Proxy.new().frobnicate())
