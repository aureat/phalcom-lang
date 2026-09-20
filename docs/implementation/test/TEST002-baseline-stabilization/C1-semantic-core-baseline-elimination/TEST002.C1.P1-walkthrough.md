---
id: TEST002.C1.P1
kind: walkthrough
status: COMPLETE
completion: PARTIAL
verification: FOCUSED_TESTED
---

# TEST002.C1.P1 — Walkthrough

## Outcome

P1 resolved or migrated BL-01 through BL-18. It stops partially complete because
BL-19 is now blocked before its intended symbolic-evidence trace by an
independent generic override-compatibility defect introduced by concurrent
repository movement. The accepted ruling creates P2 in the same checkpoint;
BL-19's oracle is unchanged.

## T7 implementation

The runtime conformance recipe now treats a selected callable and the
callee-compatible conformance environment it requires as one activation recipe.
`Selection` and `DerivedConditional` recipes carry indexed nested slot recipes.
The compiler projects sibling requirement selections into the compiled trait
default body's global requirement-slot layout; the VM materializes that exact
layout and passes it through the selected method frame. Derived defaults use
`DerivedConformanceEvidence::requirements()` for their default-body siblings,
while non-default derived witnesses retain their existing antecedent nested
evidence path.

## Override-validator containment

An unrelated override-certification change prevented canonical bootstrap by
applying its user-facing relation to existing canonical Universe/native support
contracts. P1 limits this validator at its entry boundary to declarations with
canonical Universe provider provenance. The distinction is identity/provenance
based: a synthetic fixture that merely uses a Universe-looking module ID remains
validated.

## P2 prerequisite

The synthetic monad fixture still produces four generic `IncompatibleOverride`
diagnostics before BL-19 can analyze `Monad -> Applicative -> Functor`. A
one-time migration to an actual synthetic module identity did not change the
result and was reverted. Generic override compatibility is therefore the
prerequisite; P1 does not redesign that relation.

## Verification executed

| Scope | Result |
| --- | --- |
| `cargo check -p phalcom-core` | PASS after T7 recipe representation/materialization changes |
| `bounded_generic_function_dispatches_exact_witness_and_default` | PASS (BL-18) |
| `cv46_runtime_conditional_default_executes` | PASS (BL-17) |
| `cv47_nested_runtime_evidence_invokes_nested_behavior` | PASS |
| `cv48_generic_assumed_nested_evidence` | PASS |
| override incompatible and widening probes | PASS; ordinary validator surface remains active |
| `traverse_body_resolves_symbolic_inherited_capabilities` | BLOCKED before planned trace by four generic override diagnostics; recorded for P2 |

Earlier P1 evidence remains recorded in `CHECKPOINT.md` for T0 through T6.

## Deliberately deferred

- G4 cannot complete until P2 resolves BL-19.
- G5 broad semantic/core certification is deferred until every named baseline
  residual is resolved or migrated.
- Workspace/release verification remains outside the checkpoint scope.

## Consultations and amendments

- T7 derived-default environment: adopted callee-layout-compatible nested
  environment construction from the selected conformance requirement map.
- T8: adopted P2 split. The override compatibility prerequisite is a distinct
  semantic design surface and must be investigated before reattempting the
  original symbolic-evidence correction.
