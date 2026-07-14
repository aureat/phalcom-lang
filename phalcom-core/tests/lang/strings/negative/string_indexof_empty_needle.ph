// area: strings
// spec: core-classes.md §String; docs/forge/units/U-STRING/plan.md §2.3
// status: NEGATIVE
// indexOf(_) guards against an empty needle (every offset would "match").

"abc".indexOf("")
