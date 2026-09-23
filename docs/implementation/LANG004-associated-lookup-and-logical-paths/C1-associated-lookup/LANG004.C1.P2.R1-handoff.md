---
id: LANG004.C1.P2.R1
category: LANG
program: LANG004
checkpoint: LANG004.C1
kind: implementation-handoff
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
based_on_revision: 2ea4f3a1
---

# LANG004.C1.P2.R1 — Handoff

## Current state

The setter-value lane correction is implemented. The core invariants now enforced at
dispatch are:

- selector structural slot count agrees with the invocation layout;
- setter presence agrees with getter/method/subscript-get/setter/subscript-set kind;
- the layout covers exactly the topmost receiver and physical argument window;
- captured Setter methods reject an ordinary positional RHS;
- labeled structural values remain before the trailing setter RHS in physical stack
  order and are read through their own lanes.

`Family.set(Tuple, value)` and exact/pattern Family forwarding preserve positional,
labeled, and RHS values independently. BoundMethodFamily uses the same lane layout.
Static and dynamic-pack subscript setter sends pass the same layout to a shape-aware
primitive.

## Verification inherited

See [`LANG004.C1.P2.R1-walkthrough.md`](LANG004.C1.P2.R1-walkthrough.md) for exact
commands and results. Core crate check and the narrow lane regressions pass. The
full `execution_family_runtime` filter is blocked by the pre-existing Universe
`Tuple.[]` category conflict. Workspace all-target check and repository-wide formatting
are also blocked by unrelated baselines; do not repair those under R1.

## Next action

No further R1 code change is currently indicated. When the independent baseline issues
are corrected, rerun the R1 family runtime filter and the planned broader C5 gates, then
update this verification state. Keep assignment result semantics and selector identity
unchanged. Preserve unrelated untracked plans in the worktree.
