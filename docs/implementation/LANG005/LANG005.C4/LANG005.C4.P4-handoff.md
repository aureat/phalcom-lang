---
id: LANG005.C4.P4-HANDOFF
category: LANG
program: LANG005
checkpoint: LANG005.C4
plan: LANG005.C4.P4
kind: implementation-handoff
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
next_checkpoint: LANG005.C4.P4
date: 2026-09-15
---

# LANG005.C4.P4 Handoff — C5/C6 Extension Boundary

## Stable C4 seams

The following products are stable and must be extended rather than replaced:

```text
ImplId source/provenance identity
ConformanceIndex and exact ConformanceHeadMatch
ConformanceWitnessPlan and exact ConformanceEvidence
TraitRequirementId keyed requirement selections
TraitDispatchContribution / TraitDispatchIndex
TraitDispatchSelection / TraitDispatchSite / retained ambiguity candidates
TraitDispatchTerminal proof-state transport
semantic-to-core trait invocation projection
executable conformance plan and runtime conformance environment
detached trait defaults and conformance-local witnesses
editor/LSP projection through canonical semantic products

The ambiguity diagnostic is also now a stable seam: `DiagnosticCode::TraitDispatchAmbiguous`
is emitted from the canonical unresolved-application path, while
`ExpressionAnalysis::trait_dispatch_candidates` remains the complete
candidate product and `trait_dispatch` remains unset until a unique selection
exists. Candidate notes and source labels use exact conformance identities;
they do not re-solve dispatch in diagnostics.
```

The semantic authority remains in `phalcom-semantic`. Compiler and VM code
consume proven selections and executable plans; they must not scan traits,
impl declarations, conformance records, or class dictionaries to prove or
select conformance. Trait defaults and conformance-local witnesses remain
detached from target declaration surfaces and runtime class dictionaries.

## Extension ownership

C5 owns:

```text
associated type declarations
associated bindings in conformances
projection normalization
associated-type-driven Iterable forms
```

C6 owns:

```text
generic `T: Trait` constraints
trait-bound proof machinery
conditional conformance on trait evidence
nested evidence environments required by those constraints
```

C5 and C6 must extend `ConformanceEvidence` and its exact identity inputs;
they must not replace C4 conformance identity, target/trait matching, or
runtime authority.

## Verification and baseline boundary

Focused C4 implementation evidence is green at the takeover branch. Broad
certification is `BASELINE_BLOCKED`, not an implementation failure. Known
blockers include the existing Universe generic call-entry failure, the
existing incremental A7 cold/incremental presentation mismatch, two existing
Universe Bool capability failures, existing Iterable/outgoing-pack generic
call-entry failures, pre-existing formatting drift, and pre-existing AST
Clippy violations. Re-run these against the live tree before any release claim;
do not weaken assertions or attribute them to C5/C6 without a clean comparison.

The user-supplied plan file has a P4 filename but historical P3 internal id;
preserve it as the authoritative input. The checkpoint is the single durable
`BASELINE_BLOCKED`; T5's interaction matrix and the remaining certification
gates are still open. Do not mark C4 release-complete until the baseline gates
are resolved or explicitly accepted by project policy.
