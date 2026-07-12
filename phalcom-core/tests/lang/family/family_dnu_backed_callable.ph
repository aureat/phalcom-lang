// area: family
// spec: selectors.md §3 error table ("Empty family, but class defines
//   doesNotUnderstand" — not an error); ADR-0047
// status: PASS
// A class overriding `doesNotUnderstand(_:)` makes every family callable
// even when no base name matches — the reference-time check is
// `empty && no DNU hook`, so `p::typo` is NOT an error here.

class Proxy {
  doesNotUnderstand(msg) {
    return "caught " + msg.name
  }
}
let p = Proxy.new()
let f = p::typo
System.print(f())
