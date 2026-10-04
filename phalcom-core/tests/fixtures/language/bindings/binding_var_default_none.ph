// area: bindings
// spec: values-and-absence.md; LANG007.C4-I-02
// status: PASS
// An explicitly initialized mutable optional binding reads as `None`.

let mut x: Int | None = None
System.print(x)
