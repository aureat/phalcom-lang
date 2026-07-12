// area: values
// spec: values-and-absence.md; U-CORE-4 (R-INV-4.1)
// status: PASS
// Adversarial: a negative integer, zero, and a large integer all render
// through the same native `toString` renderer (`Value::to_string`) as the
// print path — no special-casing for sign or magnitude.

System.print((0 - 7).toString)
System.print((0).toString)
System.print((123456789012).toString)
