# LANG007.C3.P2 — Advanced Agent Handoff

Continue P2 from the current worktree. The numbered plan and checkpoint remain
authoritative; this document is an operational handoff, not a competing status record
or a claim that P2 is complete. The 2026-09-23 ratified static-case/kind-form slice is
implemented and focused-tested. Continue with the still-open T4 and T8 gates, then T9.

## Mission and scope

- Complete the remaining P2 semantic work, with T4 heterogeneous row inference and
  T8 metadata/import and cold/incremental equivalence as the next major gates. Reconcile
  remaining T7 consumers, then finish through T9 only after T1–T8 acceptance.
- Preserve the accepted authority split: checked type-member signatures are
  declaration products keyed by stable owner-plus-selector identity; `AssociatedSurface`
  alone admits names, detects collisions, and selects members. The scheduler orders
  semantic products; it must never become another lookup authority.
- Do not redesign the already-ratified T6 source placement: class-body associated
  `type data`, and runtime-data associated `type data` in an unguarded inherent `impl`
  in the owner's module.
- Preserve the six staged `phalcom-core/core/future/*` renames and the untracked
  `docs/implementation/STDL002-standard-library-buildout/C2-time-complete/STDL002.C2-post-closure-implementation-audit-2026-09-23.md`.
  Do not modify them; preserve every other existing staged/unstaged/untracked path
  unless its relationship to P2 is established first.

## Current state

- Program/checkpoint/plan: `LANG007` / `LANG007.C3` / `LANG007.C3.P2`.
- Status: `IN_PROGRESS / PARTIAL / FOCUSED_TESTED` for the implemented slice; P2 is not complete.
- State anchor: branch `main`, HEAD
  `5d8d66f91b978ac60e29eca9aac2569af9c7a071` (working tree changes only). The current
  P2 diff is not committed.
- Fresh focused evidence on 2026-09-23: semantic crate check; parser
  `parse_associated_static_domain_kind_as_type_form`; generic associated-domain kind
  resolution; singleton versus zero-arity constructor signature; recursive static-domain
  payload; alias/case-signature and associated-generic-owner SCCs; nominal catalog rename
  parity; class-owned signature edit with sibling reuse and cold parity; inherent-impl
  signature edit and cold parity; and the TypeOnly runtime-negative boundary all pass.
  See the C3 checkpoint for the exact evidence list. This proves the ratified slice only.
- Current P2 edits for this slice also touch the P2 plan, `INC-C3-10.md`, AST kind
  parsing, static-case query products, algebraic builders, resolver/snapshot, and focused
  parser/semantic tests. Earlier P2 edits remain in their recorded paths.
  Preserve the six staged `phalcom-core/core/future/*` renames and the untracked STDL002
  audit listed above. Check fresh status before editing; do not stage broadly.

### Remaining work, ordered by risk

| Priority | Plan area | Current gap | Next-agent disposition |
|---|---|---|---|
| 1 | T7 declaration dependency graph | The Alias/Header/Member/StaticCaseSignature graph, stable case catalog, and checked per-case signatures are implemented. Static-case edit, sibling-reuse, cycle, recursive-domain, and cold-parity evidence is fresh. Full graph edges and SCCs are still reconstructed across the full graph. | Continue remaining all-annotation and static type-member convergence checks; keep `AssociatedSurface` as the sole lookup authority and track full-graph reconstruction as a performance follow-up. |
| 2 | T4 heterogeneous inference | The query-owned coordinator now preserves `Term` and `RecordRow` children, routes row shape solving separately, feeds field equalities into ordinary inference, and retains unresolved ancestor rows symbolically. The semantic library compiles; behavior is unverified. | Add/run the focused T4 acceptance matrix when verification is authorized. Cover mixed `Applied` and `StaticData`, nested ancestor-row resolution, inferred shared fields, mismatch, and unresolved root rows. |
| 3 | T8 persistence/import closure | The schema-v8 typed catalog, canonical-product export path, and catalog/Applied validators compile; provider composition, import reconstruction, and cold/incremental parity remain unverified. | Close exact metadata/import dependencies and parity cases after checking current gates; do not redo nominal target selection. |
| 4 | T7/T4/T8 remaining feature gates | Associated generic-owner kind resolution and zero-arity constructor formation now have focused coverage. Heterogeneous owner arguments including row forms, complete static type-member convergence, and cross-module payload argument/import parity remain open. | Reconcile each against current implementation before taking it; keep one owner for `::` selection and exact dependencies. |
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

Do not reopen these decisions. The unified graph, shell-built `AssociatedSurface`, and
old-graph reverse-invalidation path are now implemented. Focused behavioral evidence is
still required, and full-graph edge reconstruction remains a performance concern.

## Highest-risk open issue — T7 phase/dependency graph

The current implementation in `phalcom-semantic/src/session.rs` now has `Alias`,
`Header`, and `Member` nodes. It builds `AssociatedSurface` from stable declaration
shells before checked signatures, and signature fingerprints no longer participate in
the surface product fingerprint. Selected projections depend on both the owner surface
and the exact checked member product.

Before product removal, changed declaration fingerprints seed the retained graph's
previous reverse edges. Affected aliases, headers, and member products are rebuilt; the
new graph/reverse map and source fingerprints are retained after analysis. Behavior-only
inherent-impl changes refresh old and new owner surfaces. Contextual nominal binder
headers are resolved after all declaration shells exist, which addresses the recorded
`Tensor` alias application phase gap.

The source fingerprint refresh is scoped to structurally recomputed modules. Graph edge
collection and SCC classification still traverse the full declaration graph on each
analysis, so the audit's incremental-performance concern remains open. Correctness is
also not accepted until focused evidence demonstrates dependent recomputation, sibling
reuse, mixed-SCC reclassification, behavior-only collision refresh, and cold parity.
The current diff compiles; focused tests were not run against it.

Current verification update (2026-09-23): the existing `incremental::type_members`
lane passes 12/12 after test-fixture updates and a separate canonical Universe
`fiber.ph` parser repair. A new source alias regression for
`type V: Type = Tensor<Int, Shape::Scalar>` fails in cold analysis before graph
invalidation can be tested. `resolve_type_form` treats its zero-payload
`ExactEnumCase` as a contextual trait-associated projection, producing
`AnnotationUnresolved` instead of the canonical static term. Nominal case
products are currently built after aliases, so the correction requires a
publication/scheduling decision before adding annotation selection. The
focused evidence and decision request are recorded in `INC-C3-10.md`.
After that decision, run the header-edit, unrelated-sibling, and
cold/incremental assertions.

## Other difficult P2 gates

### T4 — heterogeneous row-argument inference

`InferenceTerm::Applied.arguments` and `StaticData.arguments` now store
`InferenceStaticArgument::{Term, RecordRow}`. `InferenceCoordinator` owns the ordinary
`InferenceSession` and a separate `RecordRowSolver<InferenceTerm>`; row identities remain
distinct from `InferVarId`. The coordinator sends same-label field pairs back to ordinary
inference and shares its budget and cancellation state. `GenericApplicationSession` is a
per-call view over that coordinator. Nested frames inherit row-parameter mappings, and
child results that depend on unresolved ancestor rows remain symbolic in the parent
context. The library compile check passes; behavioral acceptance remains unverified.

The original loss point was `InferenceSession::type_id_to_inference_with_rows`: row
arguments caused an opaque canonical fallback that also hid neighboring ordinary
arguments. The live conversion now retains heterogeneous children. Next, verify mixed
term/row unification, payload inference, cross-domain occurs rejection, and child-to-parent
resolution. Keep unresolved root rows underconstrained; never default them to empty.

Required T4 evidence includes static-domain generic application inference, nested
constructor unification, wrong-domain refutation, underconstraint preservation,
heterogeneous `RecordRow` application behavior, same-label field inference, row-tail
extension, and nested-frame resolution. Do not infer acceptance from compilation or the
already-passing non-row tests.

### T8 — metadata, imports, indexing, invalidation

Schema v8 now adds a typed static-domain declaration/case catalog and validates
associated owner argument kinds, static constructor membership/payload kinds,
heterogeneous row argument fields/tails, and global and scoped `Applied`
argument/result kinds. Snapshot-backed export consumes current
nominal data/enum products and associated declaration products, rather than treating
TypeStore intern history as current declaration membership. Schemas v1-v7 retain their
existing validation floors. The affected type-meta, semantic, and core libraries compile;
no tests were run against this diff. Provider catalog composition, cross-module import
reconstruction, exact invalidation, and cold/incremental equivalence remain open. Follow
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

- Historical focused result before the graph-invalidation correction:

  ```sh
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::type_members -- --nocapture
  ```

  Result: 12 selected, all passed.
- The current correction has only compile evidence so far. A later focused test
  attempt was blocked before selecting tests by the unrelated canonical Universe
  parser failure in `core/universe/src/concurrency/fiber.ph`. Do not treat that as
  evidence for the current diff.
- Also passed earlier in this P2 work (verify current tree before relying on these as
  fresh evidence): focused `adts::associated::lookup`, static-domain inference,
  generic application, generic core, imported nominal static-case identity, and
  type-data case cold parity. These are partial slice results, not a full T4/T8 gate.
- The recorded `Tensor` alias failure was traced to missing nominal shells while
  resolving named binder domains. The implementation now publishes shells before
  contextual headers; parser, ordinary-kind, and associated generic-owner kind
  resolution have focused evidence.
- Not run: broad crate/workspace tests, P2 T9 stabilization, full metadata/import and
  cold/incremental matrices. Run one focused discriminator at a time; broaden only at
  the plan's explicit gate.

## Immediate resume sequence

1. Reconcile fresh status and scoped diff with any protected user-owned paths. Keep P2
   `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`; these results close only the ratified
   static-case/kind-form slice.
2. Continue remaining all-annotation and static type-member convergence checks,
   including dependent versus sibling reuse and behavior-only collision refresh where
   not already covered by the checkpoint's historic evidence.
3. Track full-graph dependency-edge reconstruction and SCC traversal as the remaining
   T7 performance issue. Do not replace semantic dependency tracking with global
   invalidation.
4. Verify T4's mixed term/row inference using the accepted query-context coordinator and
   separate ordinary and row identities. The ownership decision is recorded under
   INC-C3-8; do not reopen it for mechanical follow-through.
5. Close T8 provider composition/import reconstruction, exact provider dependencies,
   and cold/incremental parity. The typed schema-v8 slice is recorded under INC-C3-9 but
   is not a full T8 gate. Then reconcile static-domain convergence, heterogeneous
   associated-owner, payload-constructor, and T9 gates.
6. Update the checkpoint at coherent gates. Create the normal P2 walkthrough and
   completion handoff only when P2 is actually complete.

## Recommended first work session

1. Reconcile current status with protected user-owned paths and inspect the T7-scoped
   diff.
2. Establish focused evidence for T4's coordinator and T7's retained dependency graph,
   then close T8 metadata/import and cold/incremental gates.
3. Enter T9 only after reconciling every remaining checkpoint item against actual
   implementation and acceptance evidence. Create P2 completion records only on
   actual plan completion.

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
