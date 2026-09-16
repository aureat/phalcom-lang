---
id: LANG005.C6.P1
category: LANG
program: LANG005
checkpoint: LANG005.C6
plan: LANG005.C6.P1
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
prepared: 2026-09-16
repository: aureat/phalcom-lang
---

# LANG005.C6.P1 — Implementation Walkthrough

## Scope delivered

P1 establishes the direct generic-conformance vertical for source bounds of
the form `where Subject |= TraitApplication`. The implementation covers the
canonical semantic proposition, lexical assumed evidence, generic-body trait
dispatch, exact and forwarded generic-call discharge, compiler/runtime
transport, stable metadata, reflection, source indexing, fingerprints, and
tooling.

P1 does not implement generic associated projection/equality, recursive
conditional conformance proof, the general `Iterator => Iterable` theorem,
production iteration migration, or C6 closure certification. Those remain
owned by P2–P6.

## Implementation shape

- `GenericConstraint::Conforms` is a canonical semantic constraint. Generic
  assumptions remain abstract evidence and are not represented as fake
  `ImplId` evidence.
- Generic bodies resolve trait behavior through the lexical generic-evidence
  environment. Generic applications discharge direct obligations from exact
  C4/C5 evidence or compatible enclosing assumed evidence.
- Semantic specialization remains the authority for selected evidence. Core
  lowering projects the selected type and conformance recipes, and the VM
  only materializes already-selected requirement targets through the existing
  interned runtime conformance environment; it does not search for proofs.
- Trait-bound name tracking uses `SemanticDependency::TraitHeader` because
  traits do not own nominal declaration shells. Owner dischargers continue to
  use `DeclarationShell`.
- Stable metadata exports direct conformance constraints with stable trait
  declaration references and type-node arguments. Reflection exposes the
  relation, subject, trait declaration, and arguments without fabricating a
  right-hand type.
- Source indexing preserves the canonical trait declaration target. LSP uses
  canonical `::` associated-type hover spelling and classifies `|=` as an
  operator token.

## Focused verification

The following focused gates completed successfully:

- semantic generic-conformance: 11 passed;
- semantic generic constraints: 3 passed;
- semantic fingerprints: 33 passed;
- core generic-conformance: 4 passed, including the cross-module bounded
  generic call;
- core direct-conformance reflection regression: 1 passed;
- `phalcom-type-meta` tests: 5 passed;
- associated-type LSP navigation/hover regression: 1 passed;
- full `phalcom-lsp` package tests: passed.

The broad semantic recomputation target remains `BASELINE_BLOCKED`: 12 tests
passed and one inherited Universe cold/incremental presentation-parity test
failed due retained `surfaces`/`callable_signatures` differences. That failure
is outside the direct generic-conformance seam and was not changed or used to
weaken P1 acceptance.

`cargo fmt --all -- --check` was not promoted to a passing gate because the
shared worktree contains broad pre-existing formatting drift. The scoped
`git diff --check` hygiene check is clean after removing the one new trailing
blank line.

## Consultation record

The adviser decision was `PROCEED`: add the dedicated
`SemanticDependency::TraitHeader` lane and exact query-key/currentness
mappings, while retaining `DeclarationShell` for owner dischargers and
`TraitSurface` for behavioral/member use. A second consultation corrected
compiler recipe projection so callee generic evidence slots are materialized
from semantic specialization bindings before runtime transport. No plan
amendment was required.

## Takeover boundary

P1 is complete as `IMPLEMENTED / FOCUSED_TESTED`. The checkpoint remains
`IN_PROGRESS / PARTIAL` because later C6 plans are outstanding. Continue with
the authoritative P2 plan and reuse the semantic evidence environment,
`GenericConstraint::Conforms`, stable trait-application metadata, and the
semantic-to-core evidence recipe established here.
