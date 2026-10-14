# Contextual enum inference focused repair

Date: 2026-10-14. Branch: `feat/integration-universe`.
Starting HEAD: `52838aa5`; HEAD at handoff: `41181e0b` (advanced concurrently).
All task edits remain uncommitted. No commit or push performed by this agent.

## Stop and role transition

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
- No broad workspace/release gates, Universe de-qualification, required
  declaration-pattern semantics, commits, or pushes.

## Continuation results (2026-10-14, later session)

Starting state: predecessor work had been committed by the user as
`c3119df5`; this session's edits remain unstaged on top of it.

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

Completion: semantic surface **COMPLETE**, linked executable **BLOCKED
(baseline Universe bootstrap failure, classified D)**. Verification:
**FOCUSED_TESTED** (semantic + linker). Linked execution remains
**UNVERIFIED**, now with a named baseline blocker for the next authorized
work unit.

See [the continuation handoff](enum-inference-focused-repair-2026-10-14-handoff.md)
for exact file ownership and the bounded next action.
