// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// Iterator take rejects a non-number count

[1, 2, 3].iter.take("invalid")
