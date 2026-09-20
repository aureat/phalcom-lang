---
id: TEST002.C2.P1
category: TEST
program: TEST002
checkpoint: TEST002.C2
kind: implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on:
  - TEST002.C1.P2
---

# TEST002.C2.P1 - Runtime correctness remediation

This canonical execution record follows the fixed architecture in the task
contract `docs/implementation/TEST002/TEST002.C2/TEST002.C2.P1.md`. The legacy
plan is intent; `C2-runtime-correctness-remediation/CHECKPOINT.md` is the
durable state authority.

## Execution order

Work one retained audit cluster at a time. For each cluster: reproduce the
retained probe, trace the full call/data path, record observed cause and
predicted effect, add a focused regression and observe the expected failure,
make the smallest canonical-owner repair, run the owning focused tests, then
update the checkpoint before proceeding.

## Fixed architecture

- Checked producer-side bounded operands.
- Stable lexical `super` identity distinct from access/layout authority.
- GC tracing of actual strong runtime edges.
- Public native-call validation before stack-window mutation.
- Separate native activation and interpreter re-entry budgets.
- Error-associated native provenance.
- Exact numeric-domain implementations using the existing Float decoder.
- Resource-v2 hidden-slot/exact-handle model.
- Short-write-safe `BufferedWriter` drain/flush/gate behavior.

## Active cluster

`AUD-RUNTIME-006` was the first active cluster and is now implemented and
focused-tested. Its owners are
`phalcom-core/src/primitive/number.rs`,
`phalcom-core/src/primitive/float.rs`, and the focused numeric tests. The
accepted authority is `docs/spec/library/numbers/numeric-tower.md` and
`docs/spec/library/numbers/float-protocol.md`.

The next cluster must be selected only after committing this numeric slice.

## Required evidence

Focused commands are serial and narrow: the exact retained/new numeric probe,
then directly affected numeric tests, then the named G4 gate only after the
coherent numeric slice is complete. Broad workspace tests and Clippy are
deferred unless explicitly required by a later certification gate.
