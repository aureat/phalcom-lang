// area: functions
// spec: functions.md §3; selectors.md §1 R1-R2, §2; ADR-0012
// status: PASS
// U-LEX-HASH: a selector symbol literal canonicalizes through the same
// `encode_selector` routine a method definition uses, so `#move(_,to,duration)`
// interns to the *same* Symbol as the selector `move(p, to:, duration:)`
// registers — `methodFor` hits with no reflection-side re-encoding needed.

class Mover {
  move(_ p, to, duration) {
    return p + " to " + to + " in " + duration.toString
  }
}
const m = Mover.new()
const method = m.methodFor(#move(_,to,duration))
System.print(method.selector.toString)
System.print(method.invokeOn(m, List.new().add("A").add("B").add(3)))
