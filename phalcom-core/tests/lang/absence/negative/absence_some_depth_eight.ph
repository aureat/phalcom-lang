// area: absence
// spec: values-and-absence.md §3.1; PDR-0033
// status: NEGATIVE
// Generic immediate Option nesting is bounded at seven layers. No fallback
// heap boxing is permitted at depth eight.

Some(Some(Some(Some(Some(Some(Some(Some(None))))))))
