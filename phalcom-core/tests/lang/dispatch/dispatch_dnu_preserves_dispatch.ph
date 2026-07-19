// area: dispatch
// spec: method-lookup.md §1-2; ADR-0012
// status: PASS
// A warmed call site (`s.say`) that is interleaved with a polymorphic miss on
// a different receiver (a Proxy routing through `doesNotUnderstand(_:)`) still
// dispatches correctly on subsequent hits — the dNU slow path must not corrupt
// ordinary dispatch (IC-non-corruption guard; dispatch is a stateless chain
// walk today, so this guards against a future regression).

class Speaker {
  say => "hi"
}
class Proxy2 {
  doesNotUnderstand(msg) {
    return "proxied"
  }
}
const s = Speaker.new()
const p = Proxy2.new()
System.print(s.say)
System.print(p.whatever)
System.print(s.say)
System.print(s.say)
