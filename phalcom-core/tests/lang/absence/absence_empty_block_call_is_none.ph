// area: absence
// spec: values-and-absence.md; control-flow.md; ADR-0007; U6-plan.md §4
// status: PASS
// U6: invoking an empty block via `call()` returns the block's absent result,
// which surfaces to the `None` singleton rather than the block object left in
// slot 0. Without the fix this printed `<closure>`.

System.print(|| { }.call())
