// area: strings
// spec: core-classes.md §String; docs/forge/units/U-STRING/plan.md §2.3
// status: NEGATIVE
// trimStart(_)'s charset argument must be a String — a Number would break
// the codePointAt/leadByteLen_ charset scan silently otherwise.

"  x".trimStart(5)
