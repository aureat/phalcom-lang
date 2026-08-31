// area: strings
// spec: core-classes.md §String; docs/forge/units/U-STRING/plan.md §2.3
// status: NEGATIVE
// split(_) guards against an empty delimiter (would loop forever / produce
// a delimiter-length-zero infinite match) — ArgumentError, not a hang.

"a,b".split("")
