// area: values
// spec: values-and-absence.md; U-CORE-4 (R-INV-4.1)
// status: PASS
// Adversarial: a high-precision repeating fraction and a negative fraction
// both render via Rust's native `f64` `Display` (through `Value::to_string`)
// with no rounding/truncation applied by Phalcom itself.

System.print((1 / 3).toString)
System.print((0 - (1 / 4)).toString)
