// area: strings
// spec: core-classes.md §String; docs/forge/units/U-STRING/plan.md §2.3
// status: NEGATIVE
// replace(_, _) guards against an empty `from` (would loop forever
// re-matching the same zero-width position).

"abc".replace("", "x")
