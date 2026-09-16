---
id: LANG005.C5.P2
category: LANG
program: LANG005
checkpoint: LANG005.C5
kind: implementation-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
plan: LANG005.C5.P2
prepared: 2026-09-16
repository: aureat/phalcom-lang
starting_revision: 3a2dcbd49fa7548337b52e9562cab4f8582741e4
---

# LANG005.C5.P2 Walkthrough

## 1. Final state

P2 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`. The implementation remains in
the shared working tree after the pushed P1 baseline; no unrelated files were
reset or cleaned.

## 2. Objective and outcome

P2 adds contextual `Self::Item` as a canonical semantic projection, preserves
symbolic meaning in abstract trait checking, normalizes through source binding
plans and exact conformance evidence, integrates projections into signatures,
defaults, and witness compatibility, and publishes owner-complete incremental
and source-index products. It does not add generic trait-bound assumptions,
runtime projection solving, or broader projection syntax.

## 3. Architecture implemented

- `TypeData::AssociatedProjection` carries the canonical subject, relevant
  `TraitRef`, and `AssociatedTypeRequirementId`.
- `Self::Item` is formed only in an owning trait/conformance context and uses
  trait-owned requirement identity rather than a global name or source range.
- `types::normalize_type` is the single recursive type/projection walk and
  cycle detector for abstract, source-conformance, and exact modes.
- T5 retains staged binding scheduling, mutation, diagnostics, and provenance,
  but delegates recursive normalization to T6's central authority. A missing
  source sibling remains symbolic until completeness emits `Missing`; source
  mode never requests final evidence.
- Exact evidence materializes normalized associated values and normalized
  requirement views before compatibility and dispatch consumers use them.
- The semantic session builds a canonical projection occurrence table consumed
  by the source index; editor navigation targets
  `SemanticTargetId::AssociatedType(AssociatedTypeRequirementId)`.

## 4. Important implementation areas

- `phalcom-ast`: projection syntax node, parser classification, and range tests.
- `phalcom-semantic/src/types`: projection carrier, structural plumbing, and
  `projection::normalize_type`.
- `phalcom-semantic/src/traits.rs`, `impls.rs`, and `session.rs`: contextual
  formation, source/exact normalization, evidence publication, dependencies,
  and occurrence identity.
- `phalcom-semantic/src/source_index`: canonical projection occurrence
  attachment through existing source-index machinery.
- semantic capability, query, incremental, and editor integration tests.
- `docs/specs/objects/traits.md`, C5 checkpoint/guidance, and these P2 records.

## 5. Durable interfaces and invariants

- Associated projection identity is subject + trait application/context +
  trait-owned requirement ID; names and ranges are not identity.
- Structural substitution/materialization rewrites projection constituents but
  does not solve projections.
- Abstract/source/exact terminal states remain distinct; uncertainty is not
  collapsed to `Dynamic`.
- Exact case identity is retained through projection normalization.
- Source normalization consumes the staged conformance plan, not the evidence
  product under construction.
- No second completeness authority, editor resolver, runtime solver, or broad
  invalidation path was introduced.

## 6. Deviations and adaptations

The live source-index architecture did not turn compiler-owned `Occurrence`
sites into queryable occurrences by itself. T8 therefore added a semantic
session occurrence-resolution table and attached its canonical ranges through
the existing occurrence builder. This is a semantic identity bridge, not a
source-index trait/name resolver. The adviser-required T5 migration removed
the independent source recursive normalizer and made the central normalizer
the sole authority.

## 7. Consultation

The associated-projection adviser task `01a0a645-6654-7b20-99dc-973a16164e05`
resolved the T5/T6 architecture: T5 must invoke the T6 central normalizer,
with one recursive walk and one cycle detector; missing siblings remain
symbolic in building/source mode; and source mode does not request final
conformance evidence. The decision was adopted without a plan amendment and
recorded in the C5 consultation ledger.

## 8. Tests added

- canonical projection formation, normalization, terminal-state, nested, and
  exact-evidence coverage in `capabilities::traits`;
- exact projection/signature/default/witness integration regressions;
- incremental RHS replacement/removal/readdition with normalized evidence and
  cold parity;
- projection-bearing method add/remove lifecycle;
- associated declaration/binding rename and canonical source navigation;
- source-index declaration, binding, and `Self::Item` target convergence.

## 9. Verification actually executed

- affected-crate check for `phalcom-ast`, `phalcom-semantic`, `phalcom-core`,
  and `phalcom-lsp`: PASS;
- `phalcom-ast --test trait_syntax`: 8/8 PASS;
- `phalcom-ast --test integration impl_syntax`: 16/16 PASS;
- semantic `impls::queries`: 55/55 PASS;
- semantic `incremental::associated_types`: 5/5 PASS;
- semantic `integration::editor`: 11/11 PASS;
- semantic `projection_normalization`: 6/6 PASS;
- semantic `capabilities::traits`: 36/38, with the two known baseline failures
  below;
- `git diff --check`: PASS.

## 10. Deferred verification

Workspace all-targets tests, workspace Clippy, release certification, the full
Iterable migration/corpus, full LSP packaging, and unrelated baseline suites
remain deferred to P3/C5 certification. The repository-wide formatter check
remains noisy because of pre-existing formatting drift outside this plan.

## 11. Residual failures and classification

The two failed trait capability tests are the unchanged Universe `Bool`
declaration-dependency/capability failures recorded as `C5-BL-07`. They
reproduced with the same cause and are classification D (baseline/unrelated),
not P2 failures. No A/B P2 failure remains.

## 12. Follow-up

P3 should consume the frozen projection carrier, normalizer, exact evidence,
incremental, and source-target interfaces for Iterable migration, cross-stack
integration, and broader certification. It must preserve the C5/C6 boundary
around generic trait assumptions.
