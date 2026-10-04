// area: bindings
// spec: values-and-absence.md; LANG007.C4-I-02
// status: PASS
// `None` is an ordinary value; an uninitialized binding does not read as None.

let mut x: Int | None = None
System.print(x)
