---
id: TEST002.C1.P1
kind: handoff
status: COMPLETE
completion: PARTIAL
verification: FOCUSED_TESTED
---

# TEST002.C1.P1 — Handoff

## State anchor

P1 ends with uncommitted source and record changes for T7. The active checkpoint
record is `CHECKPOINT.md`; P2 is
`TEST002.C1.P2-generic-override-compatibility-and-symbolic-inheritance.md`.

## Completed prerequisites

- BL-01 through BL-16 were resolved or migrated in prior P1 slices.
- BL-17 and BL-18 now pass.
- Exact and derived trait defaults execute nested abstract requirements through
  environments indexed for the callee default body's compiled slot layout.
- The override validator excludes only canonical Universe provider sources,
  identified by canonical source URI plus module identity. It remains active for
  synthetic source fixtures.

## Stable interfaces

- `RuntimeConformanceSlotRecipe::{Selection, DerivedConditional}` carries
  indexed nested recipes.
- `RuntimeConformanceSlot.conformance_environment` is the selected method's
  activation environment, not ambient caller evidence.
- `GenericEvidenceLayout` and semantic requirement selection remain the
  authority for executable requirement targets; the VM materializes recipes only.

## Do not redesign

- No runtime trait proof, method-table search, source-order slot mapping, slot
  compaction, or `Printable`/`Monad` name cases.
- Do not broaden canonical Universe exclusion based on module spelling alone.
- Do not weaken BL-19 or make it Dynamic/Blocked to satisfy its fixture.

## P2 starting diagnosis

`traverse_body_resolves_symbolic_inherited_capabilities` fails before its target
call because four generic methods (`map`, `pure`, `map2`, `flatMap`) are reported
as incompatible overrides. The fixture's temporary migration to a real
synthetic module identity retained all four diagnostics, proving that provider
provenance is not causal. P2 first specifies the general generic override
compatibility relation, then resumes the original T8 trace for the first
`Known(symbolic, Assumed) -> Unknown(NoTypeEvidence)` transition.

## Recommended first commands

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  traverse_body_resolves_symbolic_inherited_capabilities -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  audit_probes::override_certification -- --nocapture
```

## Deferred verification

Do not run G5 until P2 resolves BL-19. P1's focused T7 regression evidence is
recorded in the walkthrough and checkpoint.
