// Imported by negative/imports_cycle_partial_read_fails_cleanly.ph — not a
// standalone test driver. Half of a mutual-import cycle where the *other*
// half (cycle_bad_b.ph) reads a name of this module before this module's own
// top level reaches the statement that defines it — the documented
// partial-init hazard (U15 plan §4) must fail cleanly, not hang or panic.
import "./cycle_bad_b" as B
let late = 10
