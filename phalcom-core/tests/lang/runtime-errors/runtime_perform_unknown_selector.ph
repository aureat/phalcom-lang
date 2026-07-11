// area: errors
// spec: method-lookup.md §2; messages-and-selectors.md §5; ADR-0012
// status: NEGATIVE
// A `perform` of an unknown selector re-enters `doesNotUnderstand(_:)` exactly
// ONCE and surfaces `MessageNotUnderstood` — it must not loop. (If it looped,
// this case would hang rather than exit with the diagnostic.)

System.print(3.perform(Symbol.new("bogus"), List.new()))
