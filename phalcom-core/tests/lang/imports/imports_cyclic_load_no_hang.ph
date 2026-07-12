// area: imports
// spec: U15 plan §4/§5/§6
// status: PASS
// A mutual import cycle (`lib/cycle_a.ph` imports `lib/cycle_b.ph`, which
// imports `lib/cycle_a.ph` back) must load both units and terminate — the
// canonical-path registry breaks the cycle by returning the same
// (still-loading) `Module` on re-entry rather than recursing forever or
// double-compiling. A name defined *before* the cyclic import statement in
// its own file (`late`, after `cycle_a` finishes running its own top level)
// resolves normally once the whole cycle has settled.

import "./lib/cycle_a" as A
System.print(A.late)
