// area: values
// spec: values-and-absence.md §3.2; U-CORE-4 (R-INV-4.3)
// status: PASS
// `None.toString` — the message path, via `Option#toString`'s `match`
// (distinct from the print-path fixture `absence/absence_none_prints`).

System.print(None.toString)
