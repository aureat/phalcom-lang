// area: absence
// spec: values-and-absence.md; control-flow.md; ADR-0007; ADR-0010; ADR-0018
// status: PASS
// U-CORE-2: `ifTrue`/`ifFalse` return a well-formed `Option`
// (`Some(A) ∪ None`), never a raw value. A *taken* `ifTrue` with an empty
// body still wraps its arm: the block's own absent result surfaces to the
// `None` singleton (Invariant 4), which is then `Some`-lifted because the
// branch was taken. Prints `Some(None)` (U-CORE-4's `Option#toString`), not
// `None` — the old half-Option behavior this fixture pinned before U-CORE-2.

System.print(true.ifTrue || { })
