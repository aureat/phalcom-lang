# Enum inference repair — delegated-agent handoff

Continue from this handoff. The attached requirements audit and user instructions
remain authoritative; do not rediscover the whole repository.

## Mission and stopping conditions

Review and finish the bounded enum inference patch on `feat/integration-universe`.
The previous agent is now the adviser. The user explicitly stopped its
implementation after diagnosing the small linker reproducer. You must follow
whatever bounded execution scope the user gives you next; this document alone
is not authorization for an open-ended module-system repair.

Preserve ordinary lookup precedence, declaration-backed identity, conservative
blocked evidence, and strict runtime dependency validation. Explicit type
annotations and user disambiguation are accepted. Do not add global variant
search, capitalization rules, Option/Result special cases, late runtime reads,
or weaker assertions. Preserve all concurrent/staged/untracked work.

## Current state

- Checkout: `/Users/altunhasanli/dev/phalcom/phalcom`.
- Branch: `feat/integration-universe`.
- Starting HEAD was `52838aa5`; current HEAD at handoff is `41181e0b`, advanced
  concurrently. This agent did not commit or push.
- Task changes are unstaged. The user-supplied requirements audit is an
  untracked input: `/Users/altunhasanli/dev/phalcom/phalcom/Telos-enum-inference-integration-universe-requirements-audit.md`.
- Completion **PARTIAL**; named semantic/linker checks **FOCUSED_TESTED**;
  linked execution **UNVERIFIED**. All task-owned test processes are stopped.

## Code and artifact map

All paths below are absolute.

- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/expression.rs`
  — `synthesize_contextual_variant`, bare `Expr::Var` fallback, final
  `synthesize_unqualified_call` fallback, and receiver-taking associated lookup /
  invocation helpers. Owner enumeration comes from pattern nominal owners;
  symbolic inference shells supply a declaration while original expectations
  remain in the ordinary constructor application pipeline. Dependencies are
  recorded for contextual enum owners, including failed lookups.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/associated.rs`
  — `contextual_variant_is_refuted` delegates to canonical GADT branch proof.
  Only Refuted excludes a candidate; blocked evidence remains conservative.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/checker/pattern.rs`
  — `resolve_variant_pattern` uses union owners, GADT refutation filtering and
  member-specific `contextual_owner_subject`. See repeated-owner limitation below.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/compiler/lib/associated.rs`
  — `compile_associated_lookup_at` extracts existing qualified lookup emission
  by source range; canonical lowering product remains the authority.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/compiler/lib/expr.rs`
  — bare `Expr::Var` consumes attached `AssociatedLookup` lowering via that helper.
  This compiled, but linked executable behavior is not verified.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/tests/semantic/associated/lookup.rs`
  — added `contextual_enum_*` expression regressions, including lowercase cases,
  assignments, returns, parameters, nesting, shadowing, symbolic holes,
  underconstraint, missing shapes, GADT filtering, and final if/while tests.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/tests/semantic/adts/matching/resolution.rs`
  — three added union constructor regressions, all passed in the matching suite.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/tests/semantic/adts/data_declarations.rs`
  — mechanical stale API repair: workspace update Result must succeed before
  accessing the snapshot. No assertion changed.
- `/Users/altunhasanli/dev/phalcom/phalcom/packages/universe/src/collections/interval.ph`
  and `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/core/universe/src/collections/interval.ph`
  — removed retired `@variant` markers on direct data cases. This was required
  for canonical Universe loading by semantic tests.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/tests/core/language/algebraic_data/execution.rs`
  — `contextual_enum_construction_and_union_payload_patterns_execute` checks
  singleton identity, payload construction/matching and union payload execution.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-modules/src/linker.rs`
  — bounded correction in `LinkContext::collect_imports_and_graphs`: after
  canonicalizing selective imports and reexports, also add the canonical target
  module edge if it differs from the provider. Existing provider edges remain.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-modules/tests/linker.rs`
  — `conservative_reexport_reads_are_covered_by_canonical_owner_edges`.
- `/Users/altunhasanli/dev/phalcom/phalcom/docs/implementation/enum-inference-focused-repair-2026-10-14.md`
  — bounded repair record, diagnosis and exact evidence.

## Settled linker diagnosis

The full-Universe test originally stopped at:

`runtime read from universe:<root> to universe:errors.argument is not covered by the runtime graph`

The independent three-module reproducer is conclusive: leaf exports Thing,
bridge imports and exports it, entry selectively imports it from bridge. Linked
read identity is canonical `leaf::Thing`, but conservative graph construction
had only the provider edge entry -> bridge. `ExecutableLinkedProgram::from_conservative_link`
correctly required coverage for entry -> leaf.

The fix adds canonical owner coverage in the existing canonicalization pass;
it does not alter read identity or bypass validation. Exact reproducer failed
before the fix with `UncoveredRuntimeRead`. The entire linker suite passed
**19/19** afterward, including cycle rejection tests. Do not reopen this as a
broad dependency-architecture investigation without new evidence.

## Validation evidence and limits

Commands run from `/Users/altunhasanli/dev/phalcom/phalcom`, serial Cargo builds:

1. `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic associated::lookup -- --nocapture`
   — **13 passed** at that stage. Predates GADT filtering and the two final tests.
2. `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic contextual_enum -- --nocapture`
   — later **7 passed, 1 failed** because the new GADT pattern fixture's wildcard
   was correctly redundant. The fixture now includes Plain::Other; the next
   matching-suite run verifies its correction. Earlier failures led to the
   symbolic-shell, diagnostics and GADT refutation corrections recorded above.
3. `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic adts::matching -- --nocapture`
   — **161 passed, 16 pre-existing ignored**, zero failures.
4. `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test linker conservative_reexport_reads_are_covered_by_canonical_owner_edges -- --nocapture`
   — **1 failed before the linker correction**, a conclusive reproducer.
5. `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test linker -- --nocapture`
   — **19 passed after correction**.
6. `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core contextual_enum_construction_and_union_payload_patterns_execute -- --nocapture`
   — first attempt failed before fixture execution with the linker error above.
   Second attempt compiled after linker/compiler-consumption changes, entered
   the test, and was **terminated at the user's stop request, exit 143**.
   No second runtime result exists; do not label it pass or persisting failure.
7. `git diff --check` — passed at handoff.

Not executed: the two final new expression tests
`contextual_enum_if_and_while_patterns_preserve_annotations_and_assignments` and
`contextual_enum_impossible_gadt_case_reports_conflicting_specialization`.
No workspace/release validation, broad formatting gate, or incremental enum
invalidation certification was run.

## Narrow resume path

1. Read this handoff and the user-supplied audit. Inspect current status and
   scoped diff because HEAD advanced concurrently. Do not reset or stage broadly.
2. Run the final smallest semantic discriminator:
   `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic contextual_enum -- --nocapture`.
   Confirm the final new tests select and execute. Fix only causal enum defects
   within the user's delegated scope and repository debugging/escalation budget.
3. Review the same-declaration union concern before claiming consistency:
   `resolve_variant_pattern` deduplicates owners by DeclarationId and then uses
   `.find` to select an owner subject. For `Left<Int> | Left<String>`, that can
   select a specialization arbitrarily. This is source-observed, not tested.
   Begin with a precise regression. Decide whether preserving multiple case
   instantiations or requiring explicit narrower context follows the accepted
   semantic contract; do not silently choose the first instance. The singleton
   path also deserves consistency review for repeated-owner local union terms.
4. If the user authorizes completing executable proof, rerun only the named
   core regression. If another module/Universe failure blocks it, classify it
   and report to the adviser; do not expand into an open-ended module-system
   repair or weaken runtime DAG/read validation.
5. Update the repair record with actual completed results and residuals. Commit
   or push only if requested.

## Other residuals

Speculative overload isolation, fully proof/visibility-aware multi-candidate
selection, projection/generic mutable-flow retention and imported/incremental
certification are not complete. No Universe de-qualification or new required
`let`-variant destructuring semantics was introduced. Explicit annotations and
qualification remain the accepted escape hatches.
