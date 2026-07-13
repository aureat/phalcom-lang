// area: string
// spec: core/core-classes.md; values.md (U-CORE-4)
// status: PASS
// Wren precedent: test/core/string/to_string.wren (minus the 8-bit-clean
// `\0` cases — no escape sequence for it, see string_concatenation.ph).
// U-CORE-4: `String#toString` is `self` (a string's display *is* itself,
// R-INV-4.1) — no copy, no re-render, so identity-by-content holds trivially.
System.print("".toString == "")
System.print("blah".toString == "blah")
System.print("blah".toString)
