// area: values
// spec: values-and-absence.md §3.2; U-CORE-4 (R-INV-4.3)
// status: PASS
// `Some#toString` — the message path, via `Option#toString`'s `match`;
// constructed with canonical `Some(_)` class-side call syntax.

System.print(Some(42).toString)
