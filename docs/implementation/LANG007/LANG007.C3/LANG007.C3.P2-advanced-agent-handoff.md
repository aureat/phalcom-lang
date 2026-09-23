# LANG007.C3.P2 — Advanced Agent Handoff

Continue P2 from the current worktree. The numbered plan and checkpoint remain
authoritative; this document is an operational handoff, not a competing status record
or a claim that P2 is complete. Start with the T7 dependency boundary below: it is the
principal unresolved architecture blocker and can invalidate otherwise-correct
all-annotation results. T4 and T8 are separate lanes where work can proceed without
depending on that decision.

## Mission and scope

- Complete the highest-risk remaining P2 semantic work, beginning with T7's
  declaration-annotation dependency ordering and canonical `AssociatedSurface`
  publication. Then close T4 heterogeneous row inference and T8 metadata/import and
  cold/incremental equivalence; finish through T9 only after T1–T8 acceptance.
- Preserve the accepted authority split: checked type-member signatures are
  declaration products keyed by stable owner-plus-selector identity; `AssociatedSurface`
  alone admits names, detects collisions, and selects members. The scheduler orders
  semantic products; it must never become another lookup authority.
- Do not redesign the already-ratified T6 source placement: class-body associated
  `type data`, and runtime-data associated `type data` in an unguarded inherent `impl`
  in the owner's module.
- Do not modify unrelated dirty work: `phalcom-core/core/universe/src/concurrency/fiber.ph`,
  untracked `docs/specs/type/`, or `docs/specs/type.zip`. Preserve every other existing
  staged/unstaged/untracked path unless its relationship to P2 is established first.

## Current state

- Program/checkpoint/plan: `LANG007` / `LANG007.C3` / `LANG007.C3.P2`.
- Status: `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`; P2 is not complete.
- State anchor: branch `main`, pre-delivery revision
  `db6529f215357702944fa0367351155f95ab7b9b`. Use current `git status` and `HEAD`
  when taking over; the implementation and records described here are being delivered
  in the commits following that base revision.
- Latest focused result: `incremental::type_members` selected 12 tests and passed.
  `git diff --check` passed. Neither is broad P2 or workspace certification.
- Current worktree includes P2 edits in the checkpoint, P2 plan, type-member spec,
  `phalcom-semantic/src/{diagnostic.rs,session.rs}`, and semantic tests, plus the new
  `INC-C3-6.md` and this handoff. Check fresh `git status --short` before editing; do
  not stage broadly.

### Remaining work, ordered by risk

| Priority | Plan area | Current gap | Next-agent disposition |
|---|---|---|---|
| 1 | T7 declaration dependency graph | Source projections work in selected annotation tests, but a nominal generic application in an alias still fails before the alias is formed; the graph models aliases and source members, not generic headers. | Resolve the open graph/cycle boundary in INC-C3-6, then implement one scheduler and one complete AssociatedSurface publication path. Do not add provisional lookup. |
| 2 | T4 heterogeneous inference | Structural non-row static terms have focused coverage; applied inference arguments remain type-only, while row bindings are a separate solver domain. | Trace the loss point and add a RecordRow-in-Applied discriminator. Preserve separate row identity; consult if solver identity/lifetime must change. |
| 3 | T8 persistence/import closure | Schema-v7 and nominal static-case source targets are implemented; associated catalog/reference validation, cross-module reconstruction, and cold/incremental parity remain. | Close exact metadata/import dependencies and parity cases after checking current gates; do not redo nominal target selection. |
| 4 | T7/T4/T8 remaining feature gates | Static type-member domain convergence, heterogeneous associated-owner arguments, payload static-constructor argument lookup, and complete reverse invalidation remain listed in the checkpoint. | Reconcile each against current implementation/tests before taking it; keep one owner for `::` selection and exact dependencies. |
| Final | T9 | Stabilization, runtime isolation, consumer audit, lifecycle updates, walkthrough/handoff. | Start only after T1–T8 acceptance evidence is complete. |

## Settled architecture and decisions

1. **T5/T7 signature ownership:** declaration semantic products own checked source
   signatures. `AssociatedSurface` entries refer to their stable identity and own
   namespace admission/collision/selection. No independently queried type-member
   registry, no fallback priority chain, and no lookup keyed by rendered owner names.
2. **T7 selection:** type formation consumes a session/query-provided surface selection,
   retrieves the signature by `TypeMemberDeclarationId`, validates canonical static
   domains, and preserves declaration identity in the projection. Surface and signature
   changes must invalidate consumers.
3. **All-annotation availability:** the user directed phase reorganization so source
   projections are available in declaration annotations, including aliases, generic
   headers, fields, and callable signatures.
4. **Cycle rule accepted 2026-09-23:** declaration-signature SCCs containing source
   type-member signatures are invalid, including when nominal/trait generic headers
   are also participants; withhold participant products and emit one deterministic
   `TypeSignatureCycle` per component. Alias-only and header-only cycle behavior remains
   unchanged. Ordinary stuck symbolic projection normalization remains valid. The
   normative rule is in `docs/specs/type-members.md`.
5. **T6:** associated `type data` on runtime `data` owners is declared in an unguarded
   inherent `impl`, not in the representation body or a conformance impl. Owner-plus-
   name is declaration identity; the impl is provenance only.

Do not reopen these decisions. The open work is implementing one dependency graph that
includes nominal/trait generic-header products, then validating all-annotation ordering,
category-complete `AssociatedSurface` publication, and exact invalidation.

## Highest-risk open issue — T7 phase/dependency graph

Evidence in the live tree:

- `phalcom-semantic/src/session.rs` collects source members and creates a dependency
  graph whose current node variants are only `Alias` and `Member`
  (`TypeSignatureDependencyNode`). It scans all semantic structure
  shards; check whether exact incremental dependency edges can replace broad rescans.
- The early session pass publishes ready source members and updates the associated
  surface. The early builder currently sees type-member
  candidates and class behavior, not the complete final set of associated categories.
  Do not solve this by adding a second/provisional selector surface. Compare the later
  associated-surface construction and make one coherent category-complete publication
  available to every consumer.
- Alias lowering uses `lower_scoped_type_alias_form`; source `data` generic signatures
  are built later in the session pipeline.
- A same-module fixture fails before the alias is published:

  ```phalcom
  type data Shape { Scalar }
  data Tensor<T: Type, S: Shape>()
  type V: Type = Tensor<Int, Shape::Scalar>
  ```

  Observed diagnostic: `AnnotationUnresolved: unresolved type Tensor`. An importing-
  module probe failed similarly. The temporary test was removed; re-add a focused
  regression before implementing a correction. This is phase availability, not proof
  of imported static-case identity.
- The order is observable in the live session: transparent aliases are lowered before
  nominal generic signatures are published. The graph currently has `Alias` and
  `Member` node kinds, but no nominal/trait generic-header node. The records do not
  yet implement generic-header participation. The accepted decision is to add those
  nodes while preserving alias-only and header-only cycle behavior; SCCs containing a
  source type-member signature use the deterministic rejection/withholding rule.
- The graph presently does not model nominal/trait generic headers. The user accepted
  including them in the unified graph; any SCC containing a source type-member
  signature follows the accepted deterministic rejection/withholding rule, while
  alias-only and header-only cycle behavior remains unchanged. Implement this decision
  and cover mixed alias/member/header cycles; do not request the same architecture
  decision again.
- Incremental acceptance is not closed: a changed source-member signature must
  invalidate only its dependent projections/surfaces; an unrelated sibling member
  should remain reusable. Cycle component edits must recheck the affected component
  and dependents. Cold and incremental results must agree.

Focused T7 tests currently in `phalcom-semantic/tests/semantic/incremental/type_members.rs`:

- `source_type_member_projection_is_available_in_field_annotations`
- `source_type_member_projection_is_available_in_alias_annotations`
- `source_type_member_projection_is_available_in_generic_header_annotations`
- `alias_and_type_member_signature_cycle_is_rejected_as_one_component`
- existing collision, wrong-domain, signature invalidation, and cold-parity cases

Current evidence proves these focused cases only; it does not prove category-complete
early publication or all-annotation generic application availability.

## Other difficult P2 gates

### T4 — heterogeneous row-argument inference

`InferenceTerm::Applied.arguments` in `phalcom-semantic/src/checker/inference.rs`
currently stores `Box<[InferenceTerm]>`; `RecordRow` is handled separately by
`GenericApplicationSession` and `RecordRowSolver` in `checker/row_inference.rs`.
Non-row static constructor inference now has focused semantic coverage, but arbitrary
heterogeneous applied arguments that include row terms remain unfinished. Preserve
`InferVarId(kind)` and existing row identity. If correctness requires merging the row
solver's identity/lifetime into ordinary inference, stop and consult; do not add a
second variable identity or backtracking/search.

Concrete seam to inspect first: `InferenceSession::type_id_to_inference_with_rows`
receives both type and row substitutions, but `TypeData::Applied` and `StaticData`
arguments are converted through `StaticArgument::as_type_id()`. A row argument makes
that conversion fail, and the branch falls back to an opaque canonical term. The row
substitution is currently used for record tails, not as an argument alternative inside
`InferenceTerm::Applied`. The plan already specifies an inference argument envelope
(`Term(InferenceTerm)` / `RecordRow(InferenceRecordTerm)`); verify the live
`StaticArgument` and row-term APIs before selecting the smallest implementation. This
pinpoints the gap but does not authorize merging solver identities.

Required T4 evidence includes static-domain generic application inference, nested
constructor unification, wrong-domain refutation, underconstraint preservation, and
heterogeneous `RecordRow` application behavior. The current plan's local stop/consult
conditions are in T4; do not infer acceptance from the already-passing non-row tests.

### T8 — metadata, imports, indexing, invalidation

The semantic metadata path already has schema-v7 support for stable heterogeneous
static arguments and associated identity. The imported nominal static-case source-index
slice has one focused test, but broader provider metadata/catalog validation, associated
domain reconstruction, and cold/incremental equivalence remain open. Follow T8's
stable declaration identity rules; never serialize store-local IDs or reconstruct
targets from spelling. Do not broaden to workspace scans where existing exact dependency
keys can express invalidation.

Do not reopen INC-C3-4: the advisor-directed nominal-case fix uses canonical static
domain selection, publishes the existing `AssociatedResolutionKind::StaticTerm` with
stable `StaticCaseId`, and keeps `TypeOnlyRuntimeValue` for illegal runtime use.
`CallableSourceAttachment` consumes that product; it does not resolve by spelling.
Focused nominal-case, metadata, ADT incremental, and imported-provider tests passed in
a clean verification worktree at the recorded HEAD. In the shared checkout those same
tests are masked by the unrelated dirty `fiber.ph` parser failure; preserve and classify
that edit as D unless investigation changes its relationship to P2.

## Evidence and validation

- Passed in the current continuation:

  ```sh
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::type_members -- --nocapture
  ```

  Result: 12 selected, all passed.
- Also passed earlier in this P2 work (verify current tree before relying on these as
  fresh evidence): focused `adts::associated::lookup`, static-domain inference,
  generic application, generic core, imported nominal static-case identity, and
  type-data case cold parity. These are partial slice results, not a full T4/T8 gate.
- Failed diagnostic probe (not retained as a test): the `Tensor` alias fixture above
  produced `AnnotationUnresolved: unresolved type Tensor`. Do not call this a baseline;
  it is a live semantic gap in the requested all-annotation behavior.
- Not run: broad crate/workspace tests, P2 T9 stabilization, full metadata/import and
  cold/incremental matrices. Run one focused discriminator at a time; broaden only at
  the plan's explicit gate.

## Immediate resume sequence

1. Re-read `docs/workflow/luna-implementer-prompt.md`,
   `docs/workflow/commit-and-push-discipline.md`,
   `docs/workflow/implementation-record-lifecycle-convention.md`, the active P2 plan,
   the C3 checkpoint, and `INC-C3-6.md`. Inspect fresh status/diff and preserve the
   unrelated paths listed above.
2. Trace the failed `Tensor` alias path through declaration shells, generic binder
   headers/signatures, alias lowering, type formation, and associated-surface
   publication. Produce a compact graph of actual dependencies for aliases, nominal
   and trait headers, and source type-member signatures. Add a focused failing test.
3. Resolve the graph/cycle boundary with the principal/user before changing accepted
   behavior. Then amend the P2 plan and checkpoint and implement a single scheduler
   plus category-complete `AssociatedSurface` publication. Do not publish a partial
   lookup authority to satisfy early consumers.
4. Add exact reverse-dependency tests: edit one member signature and prove its consumer
   invalidates while an unrelated member is reusable; edit a cycle participant and
   compare incremental analysis with cold analysis. Preserve valid symbolic projection
   normalization.
5. Continue T4 row-argument inference and T8 metadata/import/invalidation gates in the
   dependency order of the plan. If row and ordinary inference need identity unification,
   or T8 identity/invalidation needs a broader semantic authority change, stop for a
   decision rather than improvising.
6. Update the checkpoint only at coherent gates. Create the normal P2 walkthrough and
   completion handoff only when P2 is actually complete; this advanced-agent handoff
   does not change P2 lifecycle status.

## Recommended first work session

1. Reconcile current `git status --short` with the dirty paths listed above; inspect
   only the P2-scoped diff and preserve concurrent work.
2. Reproduce the `Tensor` alias failure, then trace generic-header publication, alias
   lowering, and `AssociatedSurface` selection. Implement the accepted unified graph
   extension, category-complete surface publication, and exact dependent invalidation.
3. In parallel with the bounded T7 implementation, take the T4 discriminator: test a generic
   application containing a row argument alongside a term argument, and establish
   whether inference preserves, solves, and materializes both categories. Follow T4's
   stop/consult boundary if shared solver identity is required.
4. Resume T7 after the decision; then finish T8 metadata/import and incremental parity
   gates. Update the checkpoint at coherent gates, not chronologically.
5. Enter T9 only after reconciling every remaining checkpoint item against actual
   implementation and tests. Create the normal P2 walkthrough and completion handoff
   only on actual plan completion.

## Do not re-explore / do not shortcut

- Do not reread the whole C3 history or scan the repository broadly. Start from the
  P2 plan, checkpoint, this handoff, and `INC-C3-6.md`; inspect only named source seams.
- Do not reopen the T5 declaration-product / `AssociatedSurface` ownership decision,
  T6 `type data` placement, or the ratified alias/member cycle rule.
- Do not add another type-member registry, string-based owner key, precedence chain,
  nominal `DeclarationTypeTable` surrogate, or runtime invocation target for static
  terms/domains.
- Do not claim P2 completion from the 12-test T7 suite, an empty diff check, or a clean
  Git tree. Keep `IMPLEMENTED`, `FOCUSED_TESTED`, and `RELEASE_COMPLETE` distinct.
- Do not reset/clean the shared tree, broadly format it, or stage unrelated work.
