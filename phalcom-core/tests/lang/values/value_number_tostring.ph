// area: values
// spec: values-and-absence.md; U-CORE-4 (R-INV-4.1)
// status: PASS
// `Number#toString` (native, ADR-0019 amendment) renders the f64 value as a
// decimal string — no `.ph` number->string path exists.

System.print(42.toString)
