// area: absence
// spec: values-and-absence.md §3.3; ADR-0007 (Option as abstract with
//   Some/None); invariant-requirements.md R-INV-2.4
// status: PASS
// U-CORE-2: R-INV-2.4. Every `Option` combinator (`isSome`/`isNone`/
// `ifNone`/`orElse`) is derived purely over the `match(some:none:)`
// eliminator (core.ph) — no combinator peeks at a variant tag. Reopening
// `Option` to override `match` so it ALWAYS drives the `none:` arm must
// therefore flip every combinator's answer, proving the routing property.
// The reopen uses LABELED params (`some:`, `none:`) so it installs onto the
// same selector as the floor `option_match` primitive and overrides it
// totally — including the fixture's own trailing explicit `.match(...)`
// call below, which also routes through the override (so the last line is
// `-1`, the override's `none:` answer, not the real `match`'s `9`).

// Baseline: real match -- Some drives the some: arm, None drives none:
System.print(Some.new(1).isSome)
System.print(None.isNone)
// Override match on Option to ALWAYS take the none: arm.
class Option { match(some:, none:) { return none.call() } }
// Every combinator now reflects the override (proving they route through
// match, not a variant tag): a Some reports itself absent.
System.print(Some.new(1).isSome)
System.print(Some.new(1).isNone)
System.print(Some.new(1).orElse { Some.new(9) }.match(some: { v => v }, none: { -1 }))
