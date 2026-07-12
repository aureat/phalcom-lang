// area: values
// spec: values-and-absence.md §3.2; U-CORE-4 (R-INV-4.3)
// status: PASS
// `Some#toString` — the message path, via `Option#toString`'s `match`;
// constructed with the already-supported `Some.new(_)` send (no U-LEX).

System.print(Some.new(42).toString)
