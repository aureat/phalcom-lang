// area: values
// spec: values-and-absence.md; U-CORE-4 (R-INV-4.1)
// status: PASS
// A non-integer f64 renders through the same native `toString` renderer as
// the print path (`Value::to_string`) — no special-casing for the fraction.

System.print((3 / 2).toString)
