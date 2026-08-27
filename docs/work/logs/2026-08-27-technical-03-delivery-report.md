# Technical 03 Delivery Report

Date: 2026-08-27

Scope: Generic inference proof integrity, following:

- `docs/impl/semantic/semantic-correctness/part-4/phalcom-semantic-correctness-technical-03-implementation-plan.md`
- `docs/impl/semantic/semantic-correctness/part-4/phalcom-semantic-correctness-technical-03-generic-inference-proof-integrity-spec.md`

## Outcome

Technical 03 is implemented and integrated into the semantic checker. Generic
inference now retains required premises and proof state through solving,
publication, fallback, and diagnostics. The delivery also preserves the
current checkout's unrelated WIP changes because the requested outcome was a
clean tree.

## Integrated walkthrough

The end-to-end path is:

1. Generic application binds supplied arguments once through the canonical
   static argument binder. Binding failures publish canonical shape diagnostics
   and analyze supplied expressions once in source order.
2. Each argument is analyzed once against its inference term. The checker
   records a required premise before filtering by formal `Known` evidence, so
   an unresolved premise cannot disappear merely because no concrete type was
   available to the solver.
3. Evidence is converted into proof state: `Established`, `Assumed`,
   `Unknown(reason)`, or `Dynamic(reason)`. Alias and compound-term joins meet
   proof state with deterministic reason preservation.
4. Known arguments add subtype constraints. Unknown arguments block formal
   proof. Dynamic arguments create a dynamic boundary without fabricating a
   formal `TypeId`. Unresolved variable-to-variable subtype relations remain
   directed edges rather than being unified.
5. The inference solver checks cancellation and budget before work, propagates
   directed edges using the real subtype relation, preserves bound origins,
   and returns structured internal failures for missing variable metadata.
6. Expected-result constraints are added only after the first inference pass
   reaches a non-terminal outcome. Expected context cannot seed proof for an
   unresolved required premise.
7. Publication uses proof state and outcome together. Established and assumed
   results can materialize; unknown and dynamic dependent results retain their
   conservative evidence; conflicts, blocked inference, cancellation, budget
   exhaustion, and internal failures do not publish partial specialization.
   Fixed return annotations remain available as conservative fallback.

## Main implementation surfaces

- `phalcom-semantic/src/checker/inference.rs`: proof state, required premises,
  directed subtype propagation, control checks, origin retention, and internal
  failure outcome.
- `phalcom-semantic/src/checker/call.rs`: one-pass generic argument analysis,
  proof-aware publication, fixed-return fallback, and failing-argument range
  selection.
- `phalcom-semantic/src/checker/context.rs`: solver control delegation and
  call-status capture.
- `phalcom-semantic/src/types/evidence.rs`: shared deterministic reason joins.
- `phalcom-semantic/tests/semantic/foundations/inference.rs`: 17 focused
  inference tests.
- `phalcom-semantic/tests/semantic/foundations/generic_inference_proof_integrity.rs`:
  8 generic proof-integrity regression tests.

## Verification

| Check | Result |
| --- | --- |
| Focused inference foundation | 17 passed |
| Generic proof-integrity regressions | 8 passed |
| Generic capability suite | 12 passed |
| Canonical call application | 31 passed |
| Semantic correctness regressions | 11 passed |
| Bidirectional calls | 4 passed |
| Semantic library tests | 34 passed |
| Workspace all-target check | Passed |
| `git diff --check` | Passed |
| Full semantic integration suite | 453 passed, 17 ignored, 1 known failure |

## Remaining gaps and issues

1. `semantic::integration::imported_resolution::imported_binding_use_resolves_to_exported_declaration_not_local_import_site`
   still fails because imported use identity is local `Binding` rather than
   the exported declaration identity. This predates Technical 03 and is not
   in the changed implementation surface.
2. Repository-wide formatting remains blocked by unrelated formatting in
   `phalcom-modules/src/linker.rs` and `phalcom-modules/src/session.rs`.
3. Workspace strict Clippy remains blocked by existing generated native-surface
   `clippy::deref_addrof` findings. Semantic-only strict Clippy also retains
   existing lint debt outside the Technical 03 surfaces.
4. The capability ledger still contains intentional `GATED` and
   `RED-CAPABILITY` scenarios, including branch refinement and multi-revision
   invalidation. Focused green Technical 03 tests do not constitute release
   completion for the broader semantic-correctness program.
5. Technical 04 remains the next implementation slice.

## Next steps

1. Repair compiler-owned imported declaration identity and rerun the focused
   imported-resolution test plus the full semantic suite.
2. Resolve the red capability scenarios without weakening assertions:
   branch refinement and incremental publication/reuse.
3. Clear formatter and generated/native Clippy baselines, then rerun strict
   workspace gates.
4. Execute Technical 04 against this delivered Technical 03 baseline.

## Delivery scope note

The clean-tree request caused the commit to include pre-existing WIP edits in
the complex-analysis plan/log, compiler tail-binding behavior, capability
ledger/tests, workspace integration tests, and incremental dependency tests.
Those edits are called out here so the delivery history does not imply that
all of them were authored by Technical 03.
