# Contextual enum inference focused repair

Date: 2026-10-14. Branch: `feat/integration-universe`.
Original starting HEAD: `52838aa5`. Earlier repairs were delivered in
`c3119df5` and `9b283a03`; historical results below describe their tested
working-tree states. Audit follow-up starts at `9b283a03`.

Current completion: **PARTIAL** for full enum-inference requirements.
Canonical expression candidate selection is implemented and focused-tested; linked execution
remains **BLOCKED / UNVERIFIED**. Broad validation has not been run.

## Historical stop and role transition

The user requested: finish diagnosing the small linker reproducer, record the
result and a clear handoff, then stop; future work is delegated to another
agent, with this agent acting as adviser. No further implementation is
continuing. The second linked runtime test was terminated with exit 143 at this
stop; it is incomplete, not a pass or a newly established failure.

Completion: **PARTIAL**. Verification: **FOCUSED_TESTED** for the named
semantic and linker surfaces; linked execution remains **UNVERIFIED**.

## Implemented work

- Bare singleton names and constructor calls fall back to independently
  established expected enum owners after ordinary lookup. Canonical and
  symbolic applied expectations identify owners. Constructor application keeps
  the original expected type and uses existing generic inference; it does not
  publish solver-local variables as canonical types.
- Associated elaboration is reused, preserving declaration-backed targets.
  Missing members, wrong selector shape, ambiguity and GADT conflicts receive
  dedicated diagnostics. No global variant-name search or capitalization rule.
- Unqualified constructor patterns enumerate union owners and open selected
  cases against the matching member specialization. Canonical GADT proof
  excludes refuted owners in expressions and patterns; blocked evidence is not
  treated as refutation.
- The compiler's bare-variable branch consumes an attached associated lookup
  product through the same emission helper used by qualified lookup. This
  integration compiled but has not completed executable regression testing.
- Both Universe interval copies use direct case declarations instead of retired
  `@variant` syntax. A stale workspace-publication test now requires successful
  publication before accessing its snapshot; assertions were preserved.

## Small linker diagnosis and result

The first full-Universe runtime attempt failed before the enum fixture ran:
`runtime read from universe:<root> to universe:errors.argument is not covered by the runtime graph`.

A three-module regression independently reproduced this in
`phalcom-modules/tests/linker.rs::conservative_reexport_reads_are_covered_by_canonical_owner_edges`:
leaf exports Thing; bridge imports and exports Thing; entry imports Thing from
bridge. Canonical read identity points to leaf, while conservative graph
collection previously recorded only entry -> bridge. `from_conservative_link`
correctly rejected the uncovered entry -> leaf read.

Correction: in the existing import canonicalization pass,
`LinkContext::collect_imports_and_graphs` retains provider edges and adds the
canonical owner edge for selective imports and reexports whose finalized target
belongs to another module. Read identity, finalizer validation, strict DAG checks
and initialization-cycle rejection remain intact. No late read insertion or
Universe exception was added.

Evidence:

- Exact reproducer before correction: **1 failed**, `UncoveredRuntimeRead` for
  entry -> leaf.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test linker -- --nocapture`:
  **19 passed**, including the reproducer and existing runtime-cycle rejection.
- `git diff --check`: passed at handoff.

## Other completed verification

- Semantic compile checks passed before later refinements. Later semantic/core
  test builds compiled those refinements; do not treat that as execution proof.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic associated::lookup -- --nocapture`:
  **13 passed** at that stage, including four added contextual expression tests.
  This predates the later GADT filtering and two final added tests.
- The subsequent `contextual_enum` run: **7 passed, 1 failed**. The failure was
  a redundant wildcard in the new GADT pattern fixture, because Restricted<Int>
  had no reachable cases and Plain had only Take. Plain now also has Other,
  making the wildcard meaningful. The later complete matching run verifies
  this corrected fixture.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic adts::matching -- --nocapture`:
  **161 passed, 16 pre-existing ignored**, zero failures. Includes union payload
  specialization, union ambiguity and GADT union owner filtering.
- First runtime command:
  `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core contextual_enum_construction_and_union_payload_patterns_execute -- --nocapture`:
  failed in Universe startup with the linker coverage defect above, before
  testing the enum fixture.
- Second invocation after the bounded linker and compiler-consumption edits:
  compilation completed, runtime was still running, then canceled at the
  user's stop request. **Incomplete**.
## Deliberately unfinished

- Speculative overload isolation, complete visibility/GADT/local-proof candidate
  selection, generic mutable/projection-flow preservation, and incremental
  invalidation certification are not complete.
- No broad workspace/release gates, Universe de-qualification or required
  declaration-pattern semantics were completed. The earlier source work was
  subsequently committed in `c3119df5` and `9b283a03`.

## Continuation results (2026-10-14, later session)

Historical continuation started at `c3119df5`; its repeated-owner pattern
repair was subsequently committed as `9b283a03`.

### Final contextual expression discriminators

`RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic contextual_enum -- --nocapture`:
**12 passed, 0 failed** — includes the two previously unexecuted tests
(`contextual_enum_if_and_while_patterns_preserve_annotations_and_assignments`,
`contextual_enum_impossible_gadt_case_reports_conflicting_specialization`)
and the corrected GADT fixture. `associated::lookup` full filter: **16 passed**.

### Repeated-owner union specialization repair

The source-observed concern was confirmed by regression and repaired.

Regressions added
(`phalcom-semantic/tests/semantic/adts/matching/resolution.rs`):

- `contextual_enum_repeated_owner_union_pattern_preserves_each_specialization`:
  `Restricted<Int> | Restricted<Bool>` with `Take` requiring
  `Restricted<Bool>`. Before the fix, `.find` selected `Restricted<Int>`,
  refuted `Take`, and emitted a false GADT contradiction. After: resolved,
  payload `Bool`.
- `contextual_enum_repeated_owner_union_pattern_joins_payload_specializations`:
  `Boxed<Int> | Boxed<String>` with `Take`. Before: payload bound to `Int`
  only (arbitrary first member). After: payload joins to `Int | String`.

Correction
(`phalcom-semantic/src/checker/pattern.rs::resolve_variant_pattern`):

- the constructor path now opens the selected case against every
  `(owner, specialization)` pair of the owning declaration, keeping each
  reachable instantiation as its own `ResolvedVariantCandidate` (parity with
  `try_resolve_contextual_singleton`), instead of committing to the first
  matching specialization;
- `contextual_owner_subject` no longer pairs an arbitrary local union member
  with a canonical owner type: a repeated owner in a local union pairs the
  exact canonical member when present, otherwise falls back to canonical
  evidence only;
- ambiguity across distinct declarations remains: more than one owner
  candidate still requires qualification.

Evidence:

- Repeated-owner pair before fix: **2 failed** (false GADT contradiction;
  unjoined payload). After fix: **2 passed**.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic adts::matching -- --nocapture`:
  **163 passed, 16 pre-existing ignored, 0 failures**.

### Linked executable proof remains blocked by a baseline bootstrap failure

`RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core contextual_enum_construction_and_union_payload_patterns_execute -- --nocapture`
fails before the fixture executes: canonical Universe bootstrap fails while
compiling `universe:callable.function` with
`finalized executable link has no runtime read for `Error``.

Classification: **D — baseline, not caused by the enum-inference patch**.

- The failing source (`packages/universe/src/callable/function.ph:22`,
  `return || { Result::Ok(self.call()) }.on(Error) |e| { Result::Error(e) }`)
  is unchanged since before the task: identical at `52838aa5`, `41181e0b`
  and the committed enum work.
- A clean-tree run of the same selector at `41181e0b` selects 0 tests (the
  fixture only exists in the enum patch), so it cannot produce a
  pass/fail at baseline; classification rests on the source and failure-site
  evidence instead.
- The failure is a bare `Error` reference reaching
  `emit_global_reference` (`phalcom-core/src/compiler/lib/mod.rs:318`) with
  no import binding and no canonical runtime read in the finalized link — a
  Universe/module-graph concern, not the contextual enum resolver or the
  bounded linker edge addition (the three-module reexport reproducer and the
  full linker suite pass: **19/19**).
- No attempt was made to repair this within the enum scope; it requires its
  own bounded module/universe investigation. Do not weaken runtime DAG/read
  validation to make it pass.

### Session incident and recovery

A crate-wide `cargo fmt -p phalcom-semantic` briefly reformatted ~120 clean
files outside task scope (concurrently with the user committing `c3119df5`).
All formatting-only churn was reverted via `git restore .`; the two real edit
files were re-applied from saved copies. Final working tree diff is exactly
`phalcom-semantic/src/checker/pattern.rs` and
`phalcom-semantic/tests/semantic/adts/matching/resolution.rs`.
Post-recovery: `contextual_enum` filter re-run, **12 passed**.

### Updated completion state

Corrected completion: contextual enum core **SUBSTANTIALLY IMPLEMENTED**;
repeated-owner pattern correctness **IMPLEMENTED**; full semantic requirements
**PARTIAL**. Historical focused semantic/linker evidence is retained above.
Linked execution is **BLOCKED / UNVERIFIED** by the named Universe bootstrap
failure. These results do not constitute current-HEAD or release certification.

See [the continuation handoff](enum-inference-focused-repair-2026-10-14-handoff.md)
for exact file ownership and the bounded next action.

## Audit follow-up: candidate isolation clarification

Starting revision: `9b283a03`, clean `feat/integration-universe`. Full
requirements remain PARTIAL; this audit follow-up is the active work unit.

INC-ENUM-001 (Level 1 clarification): flow probes share inference coordinators
and propagate rigids. Adviser ruling: implementation bug only; resume with
a complete-context candidate probe. Fork type and row solvers together, rewire
row weak references, retain specialization-specific expectations and local IDs,
discard flow/diagnostics/publication/rigids and replay the unique winner. Retain
dependency reads from all candidates and share budgets/cancellation. Only proven
payload/constraint/access contradictions refute; blocked/unknown alternatives
prevent unique selection. Distinct singleton specializations remain ambiguous.

INC-ENUM-002 — symbolic union annotations: hole detection recognized unions,
but annotation elaboration handled only applications and silently returned an
empty error for `Boxed<_> | Text`. Snapshot evidence: initializer unresolved,
binding had no contract, payload projected as unspecialized `T`. Adviser approved
recursively elaborating union members using the existing context/frame/hole list
and canonical proper-type checking for hole-free members. No pattern or flow
workaround; no representation redesign. Regression oracle remains unchanged.

INC-ENUM-003 — construction access: variant analysis publishes
`visibility.construct`, but associated variant specialization/invocation did
not enforce it. Adviser approved using the existing declaration access relation
at these canonical acquisition paths, including constructor references and
singletons. Pattern and payload visibility remain independent.

The symbolic-union fixture's wildcard became unreachable after correction;
adviser approved removing that arm while strengthening the payload-Int and
declaration-backed constructor assertions. The original generic-conflict
oracle was preserved.

## Audit-follow-up delivery and verification

The corrective slice is **IMPLEMENTED / FOCUSED_TESTED** on
`feat/integration-universe`, based on `9b283a03`. Changes are not yet committed
or pushed. Full enum-inference requirements remain **PARTIAL**, not COMPLETE.

Implemented:

- Shared contextual owner/case discovery retains canonical specializations,
  symbolic branch expectations, context IDs, and exact declaration-backed cases.
  Pattern singleton/constructor paths consume the same discovery service and
  retain their existing local field-opening/coverage authority.
- Expression candidates undergo isolated specialization-specific application
  before ambiguity. A unique survivor is checked live; losing diagnostics,
  flow, rigid and publication state are discarded. Dependency reads survive.
  Repeated-owner ambiguity displays specialization types.
- Symbolic union annotations preserve their inference context and hole list;
  invalid nonproper union members retain canonical annotation diagnostics.
- Variant construction visibility is enforced through existing associated
  specialization/invocation paths, including singleton and constructor-reference
  acquisition. Owner-context access remains allowed; patterns remain independent.

Completed working-tree checks (serial Cargo validation):

| Gate | Result |
| --- | --- |
| `phalcom-semantic --test semantic contextual_enum` | **22 passed** |
| `phalcom-semantic --test semantic associated::lookup` | **25 passed** |
| `phalcom-semantic --test semantic adts::matching` | **163 passed, 16 existing ignored** |
| `phalcom-semantic --lib candidate_fork_detaches_inherited_type_and_row_constraints` | **1 passed** |
| `phalcom-modules --test linker` | **19 passed** |
| `git diff --check` | **passed** |

The matching suite ran after the shared-discovery migration, before the later
union annotation and expression construction-access corrections. The final
contextual suite includes pattern, symbolic, visibility and incremental cases.
The fork test predates only formatting and the independent annotation/access
corrections. Associated lookup ran after all behavioral corrections.

The core executable discriminator compiled and ran, but failed before fixture
execution in canonical Universe bootstrap: `universe:callable.function` has
no finalized executable runtime read for `Error` (91.51 seconds). This reproduces
the previously recorded blocker. No runtime DAG/read invariant was weakened and
no module-system repair was attempted. Execution remains **BLOCKED / UNVERIFIED**.

Deliberately deferred: broad workspace/release gates, generalized overload
candidate expectation contracts, complete local-rigid/projection viability,
generic mutable/projection-flow companion work and comprehensive incremental
certification. One rejected-owner edit is now certified against cold analysis;
that does not establish all incremental contracts. No required `let` variant
destructuring or Universe de-qualification was introduced.

An external process reverted several task edits during execution. At the user's
explicit instruction, those task-owned edits were restored and validation resumed.
An unrelated concurrent `packages/universe/src/object/object.ph` edit was left
untouched and is outside this work unit.

Next action: review/deliver this bounded correction; full semantic closure
requires bounded companion acceptance plans for the residual surfaces above,
and executable closure requires the separate Universe runtime-read repair.

Final independent source review: no actionable important correctness findings.
Review covered solver/row detachment, dependency retention, winner replay,
shared discovery, symbolic union handling and construction access. It did not
run additional tests and does not certify the broader residual requirements.
