// area: family
// spec: docs/spec/callables/family.md §1 and §3
// status: PASS
// Family construction never probes the receiver. A missing exact getter can
// still be called through a receiver-side doesNotUnderstand hook.

class Proxy {
  doesNotUnderstand(_ msg) {
    return "caught " + msg.name
  }
}
const p = Proxy.new()
const f = p::typo
System.print(f.get())
