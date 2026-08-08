// area: control flow
// spec: control-flow.md §1; values-and-absence.md §3.3; ADR-0007; ADR-0018
// status: PASS
// U-CORE-2: `if (c) { A } else { B }` desugars to `c.ifTrue || { A }.ifNone || { B }`
// (control-flow.md §1). This only composes because `ifTrue` now returns a
// well-formed `Option` (`Some(A)` when taken, `None` when not) instead of the
// pre-U-CORE-2 half-Option (a raw value on the taken arm). `ifNone` (defined
// over `match`, core.ph) runs its block only on `None`, so exactly one of the
// two prints fires per line, matching `if`/`else` exactly.

true.ifTrue || { System.print("yes") }.ifNone || { System.print("no") }
false.ifTrue || { System.print("yes") }.ifNone || { System.print("no") }
