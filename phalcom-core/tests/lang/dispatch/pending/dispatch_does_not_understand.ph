// area: dispatch
// spec: method-lookup.md
// status: PENDING

class Proxy {
  doesNotUnderstand(msg) {
    return "missing: " + msg.name;
  }
}
System.print(Proxy.new().frobnicate())
