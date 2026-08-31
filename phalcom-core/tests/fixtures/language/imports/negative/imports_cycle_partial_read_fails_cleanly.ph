// area: imports/negative
// spec: U15 plan §4/§6
// status: NEGATIVE
// The documented cyclic-import partial-init hazard: `lib/cycle_bad_b.ph`
// re-enters `lib/cycle_bad_a.ph` mid-load and reads a name (`late`) that
// unit has not defined yet (it appears *after* the cyclic import statement).
// This must fail cleanly — an ordinary `doesNotUnderstand` miss on the
// still-partial `Module` — never hang or panic.

import "../lib/cycle_bad_a" as A
