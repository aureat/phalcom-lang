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

- Run the final contextual expression tests, especially
  `contextual_enum_if_and_while_patterns_preserve_annotations_and_assignments`
  and `contextual_enum_impossible_gadt_case_reports_conflicting_specialization`;
  those two new tests have not executed.
- Complete linked execution only within the next agent's authorized scope.
- Review repeated generic-owner specializations in union patterns: owner
  grouping uses DeclarationId, and the selected subject currently uses the
  first matching specialization. This is a source-observed concern, not a
  tested failure. Do not claim arbitrary-specialization behavior is resolved.
- Speculative overload isolation, complete visibility/GADT/local-proof candidate
  selection, generic mutable/projection-flow preservation, and incremental
  invalidation certification are not complete.
- No broad workspace/release gates, Universe de-qualification, required
  declaration-pattern semantics, commits, or pushes.

See [the continuation handoff](enum-inference-focused-repair-2026-10-14-handoff.md)
for exact file ownership and the bounded next action.
