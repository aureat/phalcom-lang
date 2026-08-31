// area: values
// spec: values-and-absence.md §3.2; U-CORE-4 (R-INV-4.1)
// status: PASS
// The native print path (`Value::to_string`) renders `Some` the same way the
// message does — the pairing `Some#toString` message == print requires.

System.print(Some(42))
