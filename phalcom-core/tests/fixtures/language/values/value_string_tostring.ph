// area: values
// spec: values-and-absence.md; U-CORE-4 (R-INV-4.1)
// status: PASS
// `String#toString` is `.ph`-derived (`{ self }`, `core.ph`'s `String` reopen):
// a string's display *is* itself, no representation read needed.

System.print("hi".toString)
