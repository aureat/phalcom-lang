// Imported by cycle_a.ph — not a standalone test driver. Closes the mutual
// import cycle: re-entering cycle_a.ph's still-mid-load canonical path
// returns the same (partially-built) Module instead of looping or
// recompiling (U15 plan §4/§5).
import "./cycle_a" as A
var marker = 2
