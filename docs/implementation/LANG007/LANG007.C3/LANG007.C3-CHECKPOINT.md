---
id: LANG007.C3
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: checkpoint-record
title: "Type Data and Unified Algebraic Static Domains"
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
requires:
  - LANG007.C1
  - LANG007.C2
prepared_against_revision: 62a6082ab463387c812d558e5561508267ecc63c
prepared: 2026-09-22
active_plan: LANG007.C3.P4.R1
latest_completed_plan: LANG007.C3.P3.R1
next_plan: none accepted beyond active plan
---

# LANG007.C3 — Checkpoint Record

## Type Data and Unified Algebraic Static Domains

## 0. Checkpoint state

```yaml
program: LANG007
checkpoint: LANG007.C3
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
active_plan: LANG007.C3.P4.R1
latest_completed_plan: LANG007.C3.P3.R1
next_plan: none accepted beyond active plan
next_action: >
  T12.7 is IMPLEMENTED / FOCUSED_TESTED across commits `a66b8906`,
  `a9e584b7`, and `8727d5df`; its source-index, navigation, completion, and
  syntax-token selectors each passed 1/1. T12.8 normative spec convergence is
  IMPLEMENTED / documentation-only; no Cargo tests were run. T13's focused
  matrix is supervisor-reviewed; its broader certification remains incomplete.
  INC-C3-20 ruling is adopted with Outcome B: unannotated trait-default return
  results are not a canonical contract consumable by trait dispatch; ordinary
  class callable inferred-return publication exists. `Iterator.take` is
  separately blocked by `RecursiveFixpoint`; T13 may not add trait-default
  return inference/publication.
  The actual-expression gate uses a canonically known ListIterator receiver
  (`receiver.toList`) plus an ineligible Int negative control; both selectors
  passed 1/1. `ordinary_trait_lowering_preserves_exact_and_derived_dispatch_evidence`
  passed 1/1 and compares the full Exact/Derived semantic selections to compiled
  `TraitInvocationSpec` products. `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p
  phalcom-semantic` and `env RUSTFLAGS= RUSTC_WRAPPER= cargo check -p
  phalcom-core` passed with the existing dead-code warning. The natural
  `.iter.take(10).toList`
  discriminator remains ignored as BLOCKED by INC-C3-20; it is not acceptance
  evidence. G5 collection aggregates remain unpassed/uncertified and G6 remains
  incomplete. The §13 focused matrix is satisfied and supervisor-reviewed.
  T13 stopped at the architect ruling §15 boundary: INC-C3-20 still prohibits
  adding trait-default inference to T13 and does not authorize broad G5/G6
  certification. Separately, INC-C3-21 is now adopted and P4.R1 is active.
  T1's bounded read-only census is complete and supervisor-reviewed. T2/G1 is
  IMPLEMENTED / FOCUSED_TESTED: body-bearing trait defaults now publish and
  refresh through the canonical `CallableSignatureTable`; trait-default call
  consumers transiently project its result through already-selected Exact or
  Derived evidence and record the canonical signature dependency. The
  `TraitSurface` remains a contract projection; no parallel result state or
  proof search was added. `trait_default_inferred_return_is_canonical_and_independent_of_caller_expectation`
  and the existing
  `branch_derived_tail_type_is_published_to_unannotated_callable_signature`
  selector each passed 1/1. T3/G2 is now IMPLEMENTED / FOCUSED_TESTED under
  INC-C3-22. The focused lifecycle test verifies canonical inferred/explicit
  projections, compatible and incompatible witnesses, stable-identity plan
  replacement, exact signature/surface/plan dependencies, same-result
  fingerprint stability, unaffected cross-module plan reuse, client
  revalidation, cold parity, no conformance-to-inference feedback, and current
  versus stale diagnostic publication; it passed 1/1. The
  original CV-04 discriminator, canonical-result/caller-independence control,
  symbolic Self/generic/associated substitution, return-mismatch control, and
  diagnostic-repair control each passed 1/1. T4/CV-05–CV-07 is now
  IMPLEMENTED / FOCUSED_TESTED: same-result inferred and explicit-result body
  edits preserve interface fingerprints; changed inferred results update the
  canonical fingerprint and dependent consumer; cross-module metadata retains
  CompilerInferred authority through encode/decode/validation; cold and
  incremental products agree. The ordinary disposition selector observed one
  body recomputed and two same-result bodies reused, correcting a stale counter
  expectation without changing production stats. Exact selectors and 1/1
  results are recorded under INC-C3-21 below. T5/CV-08 is IMPLEMENTED /
  FOCUSED_TESTED under adopted INC-C3-24/25: direct-result SCC classification
  uses only canonical normal-return root-call identities; a bounded one-shot
  barrier retracts inferred-only results of unseeded SCCs, invalidates only
  their `CallableBody` roots, and reanalyzes each affected member once while
  withholding its result from the generic refresh loop. Recomputed SCC roots
  feed existing changed-callable propagation to dependent consumers; this is
  not a result fixpoint or new authority. The A→B→C test verifies cycle and
  observer Known(Int) → Unknown → Known(Int), one deterministic diagnostic
  only in B, cold parity for B/C, and reuse/stability of unrelated `h() { 3 }`.
  Direct/mutual unseeded, explicit-seed, non-result false-positive, CV-09
  Known/Ready and genuine-blocker, and canonical `Iterator.take`
  body/publication selectors pass 1/1; exact commands are in INC-C3-25 §15.
  Invalid-`n` runtime behavior remains BASELINE_BLOCKED / unverified because
  the prior gate stops before fixture execution at 20
  `ImplConformanceIncomplete` plus 3 `type.generic.underconstrained` Universe
  bootstrap diagnostics (INC-C3-23 §6/§9); this is not a CV-08 lifecycle
  failure. No corpus aggregate was run. T5/G4 is focused-tested; T6 is not
  started. G5 collection aggregates remain unpassed/uncertified, G6 incomplete,
  and the natural chain blocked/ignored. P4/C3 remains IN_PROGRESS / PARTIAL /
  UNVERIFIED. The only added corpus gate is the direct `corpus::collections`
  selector after R1's chain gate passes. Priority 3 remains unchanged.
  T13.1 census found and corrected the effective iterator-spec claim: generic
  Tuple iteration yields `Tuple<***Es>::Element`; only runtime-shaped
  `rawFromList`, `rawSlice`, and TupleZip APIs remain Dynamic. G1 AST/parser/
  semantic data/row selectors passed: `data_syntax` 28, AST `parser` 182,
  semantic `data` 124, and `record_row_polymorphism` 18. The semantic `generics`
  selector previously selected 59 (54 pass, 5 fail); that aggregate predates
  the current fixture corrections and has not been rerun. The four generic
  cases are now classified as fixture interference with the established
  mutable-binding opaque-call policy, not production generic inference
  regressions. Their exact selectors each pass 1/1 after changing result
  bindings to `const`: `generic_callable_return_inference_requires_exact_parameter_domain`,
  `generic_calls_in_one_body_do_not_share_type_variables`,
  `generic_identity_solves_parameter_from_argument_and_specializes_return`,
  and `generic_conflict_is_local_to_call_and_sibling_remains_established`.
  Direct call-expression inference remains asserted; immutable bindings keep
  the tests from observing current-value invalidation/restoration after later
  opaque calls. The detailed historical evidence and classification are in
  `04-semantic-regression-classification.md`. No generic inference or flow
  production behavior changed.
  `universe_future_preserves_payload_types_through_async_map_then_and_await`
  failed with the recorded clean-predecessor `IncompatibleOverride` for `new()`
  against `Class`; retain it as BASELINE_BLOCKED. G2 semantic `declarations`
  (85) and `type_annotations` (58) passed. The first core `data` run selected
  123 (102 pass, 19 ignored, 2 failed); both failures traced to post-c386
  compiler handling that required a generic signature for matching monomorphic
  conditional impls with empty constraints and empty proof provenance. Both
  exact selectors pass 1/1:
  `specialized_data_impl_executes_only_for_the_applicable_receiver` and
  `enum_case_methods_dispatch_through_nested_payloads`. At c386 both were
  blocked earlier by the predecessor's Tuple::Element Iterable conformance
  error, so that comparison does not classify the current failures as a
  same-stage baseline. Commit `184a3608` preserves selected/source ImplId and
  domain checks, accepts only the empty-constraint/no-signature case, and still
  requires a signature for nonempty constraints. The complete post-fix core
  `data` gate passed: 123 selected, 104 passed, 19 ignored. Core `native` passed
  32/32. G3 focused selectors passed 1/1 each: core
  `static_labeled_sources_spread_into_record_and_collections`,
  `record_pattern_remainder_binds_exact_unmatched_record_fields`,
  `map_literal_expansion_preserves_values_and_rejects_duplicate_keys`,
  `empty_tuple_and_record_keep_distinct_runtime_classes_and_equality`,
  `tuple_iterator_preserves_specialized_element_type_through_both_protocols`,
  and `runtime_shaped_tuple_slice_and_zip_stay_behind_dynamic_boundary`;
  semantic `source_visible_complete_pack_lane_projections_keep_canonical_member_declarations`
  and `record_fields_projection_preserves_exact_row_identity`. G4 has begun:
  `cargo test -p phalcom-type-meta` passed 16/16 schema tests, and semantic
  `integration::metadata` passed 28/28. The corrected exact incremental
  discriminator
  `p3_mode_only_pack_binder_edit_invalidates_dependents_to_cold_parity` now
  passes 1/1 after correcting only the final fixture argument from scalar
  `C<*Int>` to one-item pack `C<*(Int,)>`; the initial `C<Int>` negative
  `Single(TypePack)` control and cold/incremental assertions are unchanged.
  The Priority 3 ledger now classifies this as a resolved fixture syntax
  migration, not a production semantic regression. The full semantic
  `incremental` filter passed: 249 selected, 245 passed, 0 failed, 4 ignored.
  The T12.7 LSP focused gates passed 1/1 each:
  `module_navigation::goto_definition_on_structural_type_members_uses_canonical_owner_source`,
  `module_navigation::goto_definition_tracks_product_binders_and_record_remainders`,
  `semantic_completion::generic_kind_completion_uses_source_kind_names_not_runtime_values`,
  `semantic_completion::collection_literal_value_positions_use_runtime_completion_namespace`,
  and `semantic_tokens_current_syntax::record_map_and_set_literals_follow_their_distinct_source_syntax`.
  G4 focused acceptance is complete. G5 migrated-source selectors
  `corpus::system` and `corpus::indexing` passed 1/1 each; source census
  `reflection_census::canonical_source_census_matches_descriptors_and_relations`
  and Universe bootstrap measurement
  `modules_universe::boot_01_bootstrap_measurement_separates_catalog_closure_and_execution`
  passed 1/1 each. The required collection aggregates are not green: exact
  `corpus::collections` fails first at unchanged
  `boundedness_take_bounds_unbounded.ph` because `TakeIterator` lacks
  `toList`; exact `corpus::collections_literals_negative` fails first at
  unchanged `map_key_eq_mutates_same_map_during_overwrite_rejected.ph` with
  `type.generic.underconstrained`, before its expected mutation diagnostic.
  Both are D / outside P4 causality based on their unchanged pre-migration
  fixtures, but remain encountered, not baseline-certified. Focused G5
  migration/source evidence is recorded; do not claim a green aggregate
  collection gate.
  G6 has started: `cargo check --workspace --all-targets` passed with one
  existing dead-code warning. The first `cargo test --workspace` attempt
  stopped in `phalcom-ast --test integration` (283 selected, 279 passed,
  3 failed, 1 ignored) on three stale test oracles: the associated `*`
  operator chain was still labeled obsolete, `CompleteTypePack` expected the
  old `StaticDomain` AST form, and the Set lexer snapshot expected a colon for
  comma-separated `#{...}`. No production parser/lexer change was needed.
  Exact focused selectors
  `family_selector_syntax::legacy_family_and_shape_errors_are_diagnosed`,
  `family_selector_syntax::associated_operator_and_subscript_parse_correctly`,
  `impl_syntax::intrinsic_type_member_declaration_parses_in_inherent_impl`,
  and `lexer::selector_spec_components` now pass 1/1 each after updating the
  tests/snapshot. The one authorized `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test
  --workspace` retry reached `phalcom-core --lib` and reported 157 run, 155
  passed, 2 failed; the command session disappeared before a final Cargo exit
  summary, so the result is incomplete/unknown after that target and is not a
  workspace-wide total. The two failures were independently confirmed with
  exact selectors: `universe::tests::kernel_option_match_override_reroutes_every_combinator`
  failed 1/1 with the Some-through-none-arm assertion; this is the existing
  internal generic call-entry failure already recorded in LANG005.C4. The
  exact `vm::send::tests::captured_setter_rejects_an_ordinary_positional_rhs_lane`
  selector also failed 1/1. Its earlier LANG004 R1 evidence passed before
  c386; c386 subsequently changed captured-method shape validation, while no
  P4/T13 commit edits either the test or `vm/send.rs`. Classify this as D / out
  of P4 causality, but not a verified baseline because it is a post-c386
  discrepancy. The authorized workspace-test retry's command session was lost
  after the core library target; no final Cargo exit summary or later workspace
  target results were captured, so this is incomplete/unknown after core, not
  a workspace total. Do not repeat the workspace test. The single planned
  `RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D
  warnings` gate exited 101. It reported `match_like_matches_macro` for the
  pre-P4 boolean `starts_expression` match in `phalcom-ast/src/parser.rs`
  (P4 only added token alternatives), plus pre-existing type-meta lints:
  `result_large_err`, `manual_range_patterns`, and `op_ref` in
  `phalcom-type-meta`; no clippy-only fixes were made. The single
  `cargo fmt --all -- --check` gate exited 1 with broad formatting drift
  (5,263 output lines) across the workspace. Its diffs include P4-authored
  hunks in `phalcom-ast/src/parser.rs:8187-8189`,
  `phalcom-core/src/compiler/lib/patterns.rs:597-602`,
  `phalcom-core/src/compiler/lib/scope.rs:469-475`,
  `phalcom-core/src/modules/compile.rs:486-490`,
  `phalcom-core/src/modules/semantic_lowering.rs:2336-2340`, and
  `phalcom-core/tests/core/language/data_e2e.rs:623-642`, along with broad
  unrelated/pre-existing drift. P4-authored blocks in six files received only
  rustfmt-equivalent formatting edits; targeted
  `rustfmt --edition 2024 --check` on those six Rust files now passes. The
  workspace-wide format gate remains unpassed; no broad formatting was
  applied. Global `git diff --check` exited 2 solely for six trailing-
  whitespace lines in the preserved user-edited
  `docs/__unstructured__/Untitled 1.md`; that file was not edited. G6 is
  incomplete and P4 remains PARTIAL / UNVERIFIED.
  T12.1 confirmed the existing metadata tag
  `RecordRow` remains stable while `RecordFields` is the source spelling. The
  schema-v9 addition remains the scoped heterogeneous row node, not a rename.
  T11.6 is focused-tested, while its aggregate
  negative-fixture selector is encountered/unclassified due to the earlier
  Map-key mutation fixture's generic underconstraint. T11.3 found no executable legacy Record
  literals using `#{...}` in first-party `.ph` sources or the named AST,
  semantic, core, and LSP source/test surfaces; no migration edit was needed.
  Bare-brace Record, Set-opener, and legacy Record-as-Set diagnostic controls
  passed 1/1 each. Record-spread Map rejection and labeled-spread core controls
  also passed 1/1 each. T6.4 is IMPLEMENTED / FOCUSED_TESTED:
  Both inline and standalone Universe source selection include `option/unit`,
  so canonical entry, standalone fixture, and traceback compilation consume
  the source-backed Unit alias without a name fallback or runtime class
  mapping. The exact specialized
  iterator/bootstrap selector, standalone labeled-spread traceback selector,
  and T11.2 runtime-shaped Tuple slice/zip boundary selector each
  passed 1/1 under `nightly-2026-07-10`. T11.1 declarations are satisfied and
  T11.2 is IMPLEMENTED / FOCUSED_TESTED. T11.4 is IMPLEMENTED /
  FOCUSED_TESTED: Map-intent bare-brace literals use explicit `Map` heads;
  focused Map selectors and direct migrated fixtures pass. `corpus::collections`
  remains blocked before the migrated fixtures by `TakeIterator.toList`; this
  is encountered, not a certified baseline. T11.5 is IMPLEMENTED /
  FOCUSED_TESTED: simple static Set population uses `#{...}` while empty,
  mutating, dynamically populated, and rejection-focused Set.new cases remain.
  The Set parser selector and `corpus::system` passed 1/1; the earlier system
  underconstraint at `Set.new()` was resolved by the literal. Keep
  TypeStore::union, loop
  joining/whileTrue, and lexical-body semantics unchanged. T11.6 is
  IMPLEMENTED / FOCUSED_TESTED: three Map-specific negative spread fixtures
  now use explicit `Map` heads; `**1` keeps its semantic diagnostic, and
  `*source`/`***source` have the current parser's `Expected one of ";",
  newline` diagnostic. The aggregate negative-fixture selector selected 1 but
  stopped at an earlier unchanged `map_key_eq_mutates_same_map_during_overwrite_rejected.ph`
  underconstraint, before these cases. Direct CLI checks confirmed the
  diagnostics for the three edited fixtures; the aggregate failure is
  encountered, not baseline-certified. T12.1 is IMPLEMENTED / FOCUSED_TESTED
  with no source change: schema-v8 already round-trips `KindNode::RecordRow`
  and its row-tail forms; schema v9 remains justified by the separately added
  `ScopedTypeNode::RecordRow` for scoped heterogeneous arguments. The exact
  `schema_v9_scoped_record_rows_round_trip_all_tail_forms_and_v8_remains_compatible`
  selector passed 1/1, including v8 round-trip compatibility and rejection of
  the v9-only scoped node under schema v8. No schema bump or wire-tag rename
  was made for terminology. T12.2 is IMPLEMENTED / FOCUSED_TESTED: metadata
  persists semantic kind and expansion mode separately across parameter
  records and scoped lambdas. The required `(LabeledPack, LabeledTypePack)`,
  `(LabeledPack, RecordRow)`, and `(Single, RecordRow)` pairs validate and
  survive JSON decode and validated VM metadata loading; changing only mode
  for RecordRow changes both generic interface and scoped-lambda structural
  fingerprints. Semantic selectors
  `parameter_mode_and_kind_metadata_round_trip_independently` and
  `scoped_record_row_lambda_keeps_kind_and_mode_separate_through_fingerprinting`
  passed 1/1 each; core selector
  `metadata_loader_preserves_parameter_mode_and_kind_independently` passed
  1/1. Existing compatibility validation remains: it admits these intended
  pairs without deriving either field from the other. T12.3 is IMPLEMENTED /
  FOCUSED_TESTED without a metadata schema change: a cross-module consumer
  imports public provider declarations for a fixed-prefix `RecordFields`
  product and a `CompleteTypePack` product, constructs both, accesses the
  generated Record fields, and executes successfully. Exact core selector
  `cross_module_import_reconstructs_generic_data_product_shapes_from_source_declarations`
  passed 1/1. This import path analyzes the provider's authored source into
  canonical `DataInfo`/shape templates; the semantic metadata bundle has no
  static typechecking importer (the core loader validates and retains runtime
  metadata), so no downstream consumer currently requires a serialized
  product-shape template. T12.4 is IMPLEMENTED / FOCUSED_TESTED: source
  intrinsic projection resolution selects the canonical associated member
  product and publishes its `source_declaration`; RuntimePublic export carries
  that identity unchanged. Internal synthetic projections and legacy imported
  metadata may remain source-less, with omitted legacy fields accepted by the
  serde-default compatibility path. Semantic selector
  `record_fields_projection_preserves_exact_row_identity` now verifies the
  source-backed `Record::Fields` export and validation; type-meta selector
  `older_projection_metadata_without_source_identity_remains_compatible`
  verifies a schema-v8 projection with the field omitted decodes and validates.
  T12.5 is IMPLEMENTED / FOCUSED_TESTED with no production change: exact
  `PreludeTypeMap` Unit and source `collections/record::EmptyRecord` identities
  round-trip as `TypeNode::Unit` and closed-empty `TypeNode::Record`, not
  nominal declarations. Semantic selector
  `unit_and_empty_record_alias_metadata_round_trip_as_structural_zero_forms`
  passed 1/1 through RuntimePublic export, validation, JSON decode, and exact
  alias-identity checks. T12.6 is IMPLEMENTED / FOCUSED_TESTED. Its five
  cold/incremental differential tests pass against the final diff, including
  the Unit/EmptyRecord consumer row. The 2026-09-29 architect ruling keeps
  production `EmptyRecord` non-exported and forbids linker/dependency changes.
  A test-support-only source-text override at `UniverseSourceProvider` adds
  `export EmptyRecord` only to the controlled canonical-source fixture while
  preserving module, source, and location identities; parsed/interface
  products derive from that same text through the ordinary pipeline. The
  process-wide `ModuleId`-keyed builtin caches are bypassed for override
  providers so they cannot return products from production source text. The
  alias differential verifies the changed declaration-shell fingerprint,
  existing forward/reverse dependency edges, signature recomputation, and
  incremental/cold signature parity. Changed alias shells seed the existing
  dependency invalidation graph; no P4-specific invalidation registry was
  added. The initial failure was a fixture public-surface boundary, not a
  dependency-graph defect. No production export, linker, or dependency-closure
  change was made; no plan amendment is required. T12.8 is complete; T13 is
  next. P4 remains
  IN_PROGRESS / PARTIAL / UNVERIFIED; broad certification is open.
  T6.6's supplemental family-classifier discriminators pass focused testing;
  its canonical `corpus::errors` selector remains BASELINE_BLOCKED at the
  clean-predecessor fixture error, not a PASS. Keep TypeStore::union, loop
  joining/whileTrue, and lexical-body semantics unchanged. T6.7–T6.8 and
  T7.1–T7.11 plus T8.1–T8.10 are implemented and focused-tested. The adopted
  2026-09-28 T8.5 ruling permits statically projectable Record rows and labeled
  Tuple lanes, rejects Map-to-Record with a direct diagnostic, and preserves
  shared runtime lane machinery. T8.5 is IMPLEMENTED / FOCUSED_TESTED; continue
  with T8.6–T8.10 under the active plan. T8.6–T8.10 are IMPLEMENTED /
  FOCUSED_TESTED. T8.7 declaration-pattern binding preserves known field
  knowledge and residual RecordRow identity/tail; generated runtime remainder
  extraction uses the existing one-dispatch compiler-internal authority for
  `rawSize`/`rawLabelAt`/`rawValueAt`, without widening VM authorization.
  T8.8 parser evidence distinguishes explicit `Map { ... }` from bare Record
  patterns. T8.9 confirms braced source-level `@!` metadata literals remain
  `MetadataLiteral::Record`. T8.10 adds targeted old Set/record-tail parser
  messages and canonical-Map/structural-Record binding guidance without
  reinterpreting valid Set syntax. T9.1–T9.6 are now IMPLEMENTED /
  FOCUSED_TESTED: the distinct EmptyRecord immediate is retained, value-level
  Tuple/Record views expose Unit/EmptyRecord as zero-length without allocation,
  and `.class`, equality, hash stability, lane observers, Unit rendering, and
  empty iteration are covered. Focused product/value tests and the source-level
  runtime discriminator passed; `Value::class` is the immediate runtime type
  observation authority. T6.4's catalog/source-boundary changes are
  IMPLEMENTED / FOCUSED_TESTED: the live intrinsic
  application/runtime-owner enums and runtime class catalog have no Unit case
  or Unit-to-ClassId mapping; source `Unit` remains the PreludeTypeMap-owned
  alias, and empty Tuple reduction remains `store.unit()`. Canonical bootstrap
  now includes the `option/unit` source module.
  The two bare-reference `Unit` shortcuts were removed; explicit `()` syntax
  remains structural. Existing selectors `unit_is_not_a_separate_intrinsic_semantic_declaration`,
  `prelude_map_contains_only_explicit_source_backed_type_names`,
  `unqualified_unit_type_reference_uses_exact_prelude_alias_declaration`,
  `zero_intrinsic_applications_select_complete_pack_and_record_row_domains`,
  and `generated_empty_tuple_return_resolves_to_canonical_unit_type`, together
  with T9's empty-product runtime selector, provide isolated focused evidence.
  The canonical core iterator selector initially exposed that the inline
  Universe source-selection filter omitted `option/unit`; adding that source
  module fixed bootstrap without restoring either removed Unit fallback. The
  iterator selector now passes. T10 adopts the
  2026-09-28 ruling: remove Unit from the runtime-support key/catalog, while
  preserving source visibility through canonical `option/unit::Unit` in the
  existing `PreludeTypeMap`. Native Unit returns use structural empty-tuple
  `()` metadata, resolving through the semantic zero-product type. Focused
  semantic prelude/declaration/editor, native signature, runtime class lookup,
  and core-check evidence is recorded in §20. T10 is IMPLEMENTED /
  FOCUSED_TESTED; the isolated unrelated floor-census discrepancy does not
  show a Unit binding or class-count regression.
  Keep `corpus::bytes` unclear/nonblocking. Priority 3 categorical classification
  is already recorded; causal isolation, remediation/migration decisions, and
  residual disposition remain. INC-C3-20 ruling is adopted: Outcome B applies
  to unannotated trait-default returns, whose results are not a canonical
  contract consumable by trait dispatch; ordinary class callable inferred
  returns exist, and `Iterator.take` is separately blocked by
  `RecursiveFixpoint`. T13 does not add trait-default return inference/
  publication; further work requires separate semantic design. The natural
  `.iter.take(10).toList` expression still has
  `Unknown(UnannotatedDeclaration)` after `Iterable.iter` and remains blocked.
  T13's amended known-receiver gate passes its positive and negative source-level
  selectors. T13/P4/C3 remains IN_PROGRESS / PARTIAL / UNVERIFIED; Priority 3
  is unchanged. No runtime fixture, `corpus::collections`, workspace test, or
  broad certification gate was run for this amendment. The 2026-09-30
  completion-audit follow-up separately corrected iterator adapter source
  contracts and `Iterable.toMap`; its semantic fixture passes, but canonical
  Universe/core verification is blocked before the test body by bootstrap
  conformance diagnostics confirmed at parent `01e4db77`; mark that runtime
  gate BASELINE_BLOCKED, not pass. Do not promote P4/C3 or treat the
  source/runtime changes as end-to-end verified.
blocker: INC-C3-20 natural iterator-chain integration remains blocked under Outcome B; no T13 return-inference expansion is authorized.
```

This checkpoint record is the single durable state authority for C3 once implementation begins.

The checkpoint must be changed to `IN_PROGRESS` when P1 starts, not merely because planning documents exist.

---

## 1. Objective

Establish one coherent algebraic declaration and static-domain system in which:

```text
data
    defines runtime closed algebraic domains
    including products, sums, ADTs, and GADTs

type data
    defines type-system-only closed algebraic/static domains

associated type data
    participates in the declaration-owned :: namespace

intrinsic data / intrinsic type data
    expose compiler-owned structural domains through ordinary language semantics
```

while preserving:

- exact-case/GADT behavior;
- C1/C2 pack and type-member identity;
- generic binder-mode semantics;
- row separation;
- metadata stability;
- incremental correctness;
- runtime representation freedom;
- the no-runtime-static-solver boundary.

---

## 2. Bounded acceptance statement

C3 is complete only when this statement is true:

> Runtime products and closed sums use one canonical `data` algebraic declaration model; `type data` provides nominal canonical static domains and terms; generic/static application, inference, substitution, aliases, associated lookup, metadata, imports, and incremental analysis preserve user static-domain identity; compiler intrinsic structural domains are exposed through that same model without duplicating C1/C2 terms; tooling and canonical source have migrated from `enum`; and no VM static-term execution authority has been introduced.

---

## 3. Ownership boundary

### 3.1 C3 owns

- unified runtime algebraic declaration syntax/semantic identity;
- `enum` source migration/removal;
- product/sum shape classification;
- runtime case syntax under `data`;
- `type data`;
- nominal static-domain identity;
- canonical type-data static terms;
- compile-time static Int term integration needed by C3;
- heterogeneous generic/static argument application;
- domain-directed non-row inference over user static domains;
- associated `type data`;
- associated static-domain lookup through `::`;
- stable metadata for C3 domains/terms/applications;
- cross-module/incremental identity;
- intrinsic structural domain source surface;
- Universe/corpus migration;
- C3 LSP/source-index closure;
- specification migration;
- checkpoint certification.

### 3.2 C3 does not own

- arbitrary compile-time execution;
- a macro system;
- a theorem prover;
- user-defined primitive kinds;
- user-defined static equality hooks;
- arbitrary type-level pattern functions;
- trait requirements that bind whole `type data` domains;
- GAT expansion beyond existing accepted semantics;
- a general trait static-domain generic redesign unless separately amended;
- automatic lifting of runtime values into static terms;
- new pack decomposition/backtracking;
- runtime ArgumentPack ABI redesign;
- runtime type-data values;
- VM static-domain solving;
- forced unification of Tuple/Record/data runtime storage;
- release-wide unrelated cleanup.

---

## 4. Plans ledger

| Plan | Scope | Status | Completion | Verification | Outcome |
|---|---|---|---|---|---|
| `LANG007.C3.P1` | Unified algebraic `data`; enum semantic migration; top-level `type data` foundation | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | completed; walkthrough and handoff present |
| `LANG007.C3.P2` | General static domains; heterogeneous application/inference; associated `type data`; metadata/import/incremental | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | T1–T9 accepted; G-FINAL completed with documented clean-predecessor and formatter baselines; see P2 walkthrough/handoff |
| `LANG007.C3.P3` | Intrinsic structural surface; Universe/corpus/tooling migration; certification and closure | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | T1-T10 accepted; generic-owner payload behavior fixed; all required C3 coverage has direct evidence or a clean-predecessor baseline disposition; see P3 walkthrough/handoff |
| `LANG007.C3.P3.R1` | Corrective closure: explicit intrinsic semantic application and runtime/native owner projections | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | user-adopted projections implemented and focused gates passed; see P3.R1 handoff |
| `LANG007.C3.P4` | Intrinsic structural data, structural kinds, product-shape expansion, and Tuple/Record semantic completion | IN_PROGRESS | PARTIAL | UNVERIFIED | active plan |
| `LANG007.C3.P4.R1` | Canonical inferred result contracts for body-bearing callables, including trait defaults | IN_PROGRESS | PARTIAL | UNVERIFIED | T1–T5/G4 focused work recorded; T6 remains unopened; does not certify T13/G5/G6 |

P3.R1 was a bounded corrective closure required by the explicit ruling supplied
after P3 closure. P4 completes the intrinsic structural-data model and structural literal convergence.

### 4.1 P2 architecture amendment — static cases and static-domain kinds

On 2026-09-23 the user ratified the architecture recorded in `INC-C3-10.md`.
P2 therefore adds per-case checked constructor signatures to the existing
Alias/Header/Member graph, while retaining early structural case catalogs as
the declaration-identity authority. Singleton getters require no signature;
constructor-shaped cases, including `Unit()`, require a checked signature.
Cycles containing Member or StaticCaseSignature are rejected as one
`TypeSignatureCycle`; existing Alias-only and Header/Alias-only behavior stays
unchanged. Static-domain kind syntax carries an ordinary `TypeAnnotation` and
resolves through the existing type-form path so applied associated owners retain
their heterogeneous arguments. Details and implementation order are in
`INC-C3-10.md` and the P2 plan amendment.

---

## 5. Predecessor state

### 5.1 LANG007.C1

C1 established the type-level domain/member substrate C3 relies on.

C3 must preserve:

- canonical `TypeId`/`KindId`;
- `ProperTypeId`;
- parameterized type-member identity;
- structural Tuple/Record operations;
- RecordRow separation;
- compiler-owned closed normalization.

### 5.2 LANG007.C2

C2 is:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

Inherited C2 facts:

```text
*   positional
**  labeled
*** complete

TypePack
LabeledTypePack
CompleteTypePack
TypePackConcat
LabeledTypePackConcat
TupleFromPacks
CallableFromPacks

GenericParameterMode
GenericBinderHeader
checked substitution/materialization
whole-pack InferVarId(KindId) inference
metadata schema v4 scoped packs
cold/incremental parity
no runtime pack solver
```

Do not redesign these.

---

## 6. Repository baseline

Planning baseline:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   62a6082ab463387c812d558e5561508267ecc63c
```

Relevant post-C2 observation:

- C2 checkpoint records are complete.
- Future-facing `PROGRAM.md` / `STATUS.md` contain some stale C2 next-action wording.
- P1 start must rebaseline active program state to C3.

This planning document does not rewrite historical C2 records.

---

## 7. Inherited established invariants

These are already established before C3 begins.

### 7.1 Canonical type/static substrate

```text
C2-I01 — TypeId is a store-relative canonical type-level term handle.
C2-I02 — ProperTypeId proves the Type domain.
C2-I03 — KindId classifies canonical TypeId terms.
C2-I04 — RecordRow remains a separate row-domain representation/solver.
C2-I05 — LabeledTypePack is not RecordRow.
C2-I06 — CompleteTypePack is not Tuple.
```

### 7.2 Packs

```text
C2-I07 — * means positional lane.
C2-I08 — ** means labeled lane.
C2-I09 — *** means complete lane.
C2-I10 — ... is not pack syntax.
C2-I11 — GenericParameterMode is independent from KindId.
C2-I12 — Single(TypePack) != PositionalPack(TypePack).
C2-I13 — whole-pack inference uses ordinary InferVarId(KindId).
C2-I14 — no pack partition/decomposition solver exists.
C2-I15 — C2 materialization re-enters canonical TypeStore constructors.
```

### 7.3 Persistence/runtime

```text
C2-I16 — stable metadata is structural, not source-string identity.
C2-I17 — query-local inference IDs do not enter durable products.
C2-I18 — cold and incremental pack semantics converge.
C2-I19 — no runtime pack object/opcode/solver exists.
```

---

## 8. Required C3 acceptance invariants

These are requirements, not yet implementation facts.

They become "established" in this checkpoint only after the owning plan implements and verifies them.

```text
C3-I01 — `data` is the sole canonical runtime algebraic declaration category.
C3-I02 — `type data` is semantic phase TypeOnly with no runtime inhabitants.
C3-I03 — phase, shape, implementation provenance, and ownership are orthogonal.
C3-I04 — Product/Sum shape, not keyword category, triggers algebraic semantics.
C3-I05 — runtime sum exact-case and GADT semantics are preserved.
C3-I06 — one algebraic semantic authority owns product/case structure.
C3-I07 — TypeId remains canonical carrier for non-row kinded static terms.
C3-I08 — ProperTypeId remains the proper-Type proof boundary.
C3-I09 — user static domains use nominal StaticDomain identity, not per-domain Rust enum cases.
C3-I10 — StaticArgument is the heterogeneous application envelope.
C3-I11 — RecordRow remains row-owned under heterogeneous applications.
C3-I12 — GenericParameterMode stays orthogonal to static domain.
C3-I13 — non-row static inference reuses InferVarId classified by canonical domain.
C3-I14 — type-data unification is closed structural first-order unification.
C3-I15 — runtime equality/dispatch never defines static identity.
C3-I16 — valid unresolved static terms remain symbolic.
C3-I17 — associated type-data is declaration-owned, not trait-evidence-owned.
C3-I18 — one associated namespace authority owns collision/lookup semantics.
C3-I19 — associated domain specialization preserves owner environment.
C3-I20 — legacy enum compatibility never creates distinct semantic identity.
C3-I21 — TupleEntries is the public surface over C2 CompleteTypePack.
C3-I22 — intrinsic declarations do not replace existing C1/C2 canonical term owners.
C3-I23 — metadata stores stable declaration/domain/static-argument structure.
C3-I24 — incremental and cold results converge for C3 edits.
C3-I25 — type-data inhabitants require no VM value/object/opcode/solver.
C3-I26 — runtime algebraic storage remains representation-independent.
C3-I27 — LSP/source tools consume semantic products, not rendered-name reconstruction.
C3-I28 — current authoritative specs are coherent before checkpoint closure.
```

---

## 9. Stable takeover map

| Concept | Live owner | C3 rule |
|---|---|---|
| canonical type-level term | `TypeStore` / `TypeId` | extend; do not duplicate |
| proper type proof | `ProperTypeId` | preserve strictly |
| kind/domain classifier | `KindId` / `KindData` | extend with nominal static-domain form |
| row term | `RecordRowId` / row subsystem | preserve separate |
| generic binder mode | `GenericParameterMode` | preserve orthogonal |
| generic early contract | `GenericBinderHeader` | extend domain support |
| generic signature | `GenericSignature` | extend domain support |
| non-row inference var | `InferVarId(KindId)` | reuse |
| runtime product | `DataInfo` / `DataSemanticTable` | derive from unified algebraic authority |
| runtime sum | `EnumInfo` / `VariantInfo` | derive/reuse under unified algebraic authority |
| exact case | `VariantId` + `TypeData::ExactCase` | preserve identity; neutral terminology may wrap |
| GADT environment | `CaseTypeEnvironment` | preserve |
| associated lookup | `AssociatedSurface` / checker associated resolver | extend |
| type-member projection | type-member subsystem | integrate with common static domains |
| packs | C1/C2 TypeStore constructors | preserve |
| complete Tuple entries | `CompleteTypePack` | expose as `TupleEntries` |
| metadata | `phalcom-type-meta`, schema v4 baseline | version explicitly for C3 |
| Universe identity | canonical Universe modules | migrate source surface; keep identity |
| runtime data/sum lowering | `phalcom-core` specialized lowering | preserve representation freedom |

---

## 10. Architecture decisions adopted for planning

### A. Canonical term carrier

Use:

```text
TypeId + KindId
```

for all non-row canonical static terms, including new const/type-data terms.

Do not introduce a second general static-term arena.

### B. User static domain classifier

Use one dynamic nominal classifier equivalent to:

```rust
KindData::StaticDomain(StaticDomainId)
```

Do not add one compiler enum variant per source declaration.

### C. Heterogeneous application

Use a canonical envelope equivalent to:

```rust
StaticArgument::Term(TypeId)
StaticArgument::RecordRow(RecordRowId)
```

and migrate `Applied`, scoped application, substitution, metadata, and formatting.

### D. Inference

Reuse:

```text
InferVarId(kind)
```

for non-row user static domains.

Keep row inference separate.

### E. Algebraic declarations

Use one semantic declaration product/facade with:

```text
phase
shape
generic contract
product/case structure
```

DataInfo/EnumInfo may remain compatibility projections.

### F. Associated domains

Extend `AssociatedSurface`.

Do not create an independent static-domain namespace.

### G. Trait boundary

Whole associated `type data` requirements are out of scope.

User static-domain trait generics are explicitly unsupported until a plan proves safe conformance/coherence migration.

### H. Runtime boundary

Only Runtime algebraic data lower to runtime inhabitants.

TypeOnly terms never do.

---

## 11. P1 acceptance contract

`LANG007.C3.P1` must establish:

```text
source:
    canonical data Product/Sum
    type data Product/Sum
    legacy enum -> same algebraic AST during compatibility

semantics:
    one algebraic declaration authority
    preserved DataInfo/EnumInfo derived behavior
    top-level nominal StaticDomainId
    canonical type-data terms

soundness:
    ProperTypeId boundary
    static payload validity
    no runtime type-data inhabitants

runtime:
    exact-case/GADT/match/impl behavior preserved

persistence:
    first stable C3 algebraic/static-domain metadata
```

P1 must not complete heterogeneous generic application or associated type data.

---

## 12. P2 acceptance contract

`LANG007.C3.P2` must establish:

```text
StaticArgument
heterogeneous Applied
generic/static binder domains
type-data/const static arguments
domain-directed inference
checked substitution/materialization
static aliases/result domains
associated type data
unified :: lookup
metadata/import round trip
cold/incremental parity
source-index identity
explicit trait boundary
```

Representative source must work:

```phalcom
type data Shape {
    Scalar
    Vector(_ size: Int)
}

data Tensor<Element: Type, S: Shape>()

type V: Type = Tensor<Float, Shape::Vector(3)>

class Parser {
    type data State {
        Start
        Done
    }

    type Initial: State = State::Start
}
```

---

## 13. P3 acceptance contract

`LANG007.C3.P3` must establish and certify:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries
intrinsic type data RecordRow

intrinsic data Unit()
intrinsic data Tuple<***Es>
intrinsic data Record<R: RecordRow>
```

and:

```text
TupleEntries == CompleteTypePack semantic domain

*T   == T::Positionals
**T  == T::Labeled
***T == T::Entries
```

where valid.

P3 also owns:

- Option/Result/Ordering migration to `data`;
- final enum compatibility removal/finalization;
- LSP tooling;
- corpus/examples/active docs migration;
- authoritative spec closure;
- metadata/interface fingerprint certification;
- cold/incremental parity;
- performance/resource evidence;
- runtime-negative audit;
- C3 checkpoint closure.

---

## 14. Verification ledger

C3 was closed as COMPLETE / IMPLEMENTED / FOCUSED_TESTED after P3 T10, then
reopened for the bounded P4 correction adopted from the user's 2026-09-25
ruling. P1/P2/P3 remain complete; current C3 closure depends on P4 separating
the semantic application behavior from the runtime/native owner explicitly.

| Gate | Scope | Result | Meaning |
|---|---|---|---|
| predecessor C2.P3 G1–G5 | C2 substrate | PASS | C3 starts from verified packs/inference/metadata/runtime boundary |
| P1 AST syntax filters | phalcom-ast integration data_syntax, enum_syntax | PASS | 25 + 12 parser/compatibility tests passed |
| P1 semantic static-domain foundation | phalcom-semantic integration foundations::kinds static filters | PASS | canonical domains/ConstInt/StaticData, proper-type rejection, and source type-data kind publication passed |
| P1 semantic crate check | cargo check -p phalcom-semantic | PASS | all new TypeData consumers compile |
| P1 core compile boundary | cargo check -p phalcom-core | PASS | compiler/runtime consumers compile with static forms present |
| P1 runtime algebraic preservation | semantic `adts::declarations`, `constructors`, `variants`, `exact_cases`, `vertical_gadt` | PASS | 16 + 13 + 12 (9 active, 3 gated) + 5 active (1 gated) + 1 tests passed; legacy enum/data semantic identity remains equivalent |
| P1 metadata/static-term export | semantic `integration::metadata` and `foundations::kinds` | PASS | 21 metadata tests and 11 kind tests passed; schema-v5 static-domain, ConstInt, StaticData export/round-trip validation and runtime-negative coverage are covered |
| P1 metadata compatibility | phalcom-type-meta schema_compat | PASS | 9 tests passed; v1 fixture and v2–v4 compatibility remain valid, v5 feature/kind floors reject malformed static nodes |
| P1 ADT/static incremental parity | semantic `incremental::adts`, `incremental::fingerprints`, `incremental::type_store_revisions` | PASS | 13 active + 3 gated, 33, and 20 tests passed; runtime ADT and TypeOnly case-edit cold parity are green |
| P1 runtime representation | phalcom-core `core language::algebraic_data` | PASS | 44 passed, 19 explicitly ignored/gated; native_adt_runtime is not a live Cargo test target |
| P1 library unit-test binary | cargo test -p phalcom-semantic static_data | BASELINE_BLOCKED | pre-existing DeclarationTypeInfo test initializer omits generic_binder_header; outside the focused integration/G5 matrix |
| P4 T12.6 incremental fingerprints | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::t12_6 -- --nocapture` | PASS | 5 selected, 5 passed after final formatting; covers pack kind/mode, product expansion, intrinsic member result kinds, Record literal edits, and Unit/EmptyRecord alias consumer parity |
| P4 T12.6 module session integration | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test workspace_session` | PASS | 26 passed; default provider construction remains covered without the test-support feature |

Do not copy predecessor PASS status into C3 verification.

---

## 15. Deferred / baseline issue ledger

### BASELINE-001

```text
Observed:
    semantic::capabilities::traits::
    trait_default_replays_external_nominal_callable_dependencies

Classification:
    inherited baseline / unrelated unless C3 proves causal interaction

Why nonblocking at C3 start:
    C2 explicitly deferred it; C3 architecture does not require trait-default repair.

When to revisit:
    only if a C3 focused trait-boundary test is blocked by the same cause.
```

### BASELINE-002

```text
Observed:
    semantic::capabilities::traits::
    trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities

Classification:
    inherited baseline / unrelated unless C3 proves causal interaction

Why nonblocking at C3 start:
    C2 explicitly deferred it.

When to revisit:
    only if a required C3 trait/conformance compatibility gate depends on it.
```

### BASELINE-003

```text
Observed:
    phalcom-semantic library test compilation fails in
    checker/context.rs:3362 because a pre-existing DeclarationTypeInfo test
    initializer omits generic_binder_header.

Classification:
    unrelated baseline compile failure; the C3 integration test binary does
    not depend on this cfg(test) initializer and remains usable.

Why nonblocking at this stage:
    focused C3 integration tests and both affected crate checks pass.

When to revisit:
    before claiming a broader semantic crate unit-test gate; it does not block
    the focused P1 G5 matrix.
```

### P1-RESOLVED-001 — stable algebraic phase/shape declaration records

```text
Observed:
    schema v5 now persists stable static-domain kinds, ConstInt/StaticData
    nodes, and declaration-owned algebraic phase/shape records. Runtime
    Product versus Sum facts are projected from the canonical algebraic tables.

Classification:
    resolved P1 implementation and metadata gate.

Evidence:
    `metadata_publishes_stable_algebraic_phase_and_shape` and
    `metadata_keeps_legacy_enum_and_equivalent_data_identity_neutral` pass;
    the exporter records Runtime/TypeOnly and Product/Sum structurally and the
    interface fingerprint includes those records.
```

### P1-RESOLVED-002 — type-data runtime-negative and incremental parity evidence

```text
Observed:
    TypeStore proper-type rejection, TypeOnly compiler no-op behavior, and
    source-level runtime-use rejection are implemented. TypeOnly case edits are
    covered by a cold/incremental parity regression.

Classification:
    resolved P1 implementation and verification gate.

Evidence:
    `type_data_case_cannot_be_used_as_a_runtime_value` and
    `type_data_case_edit_matches_cold_static_domain_analysis` pass. The latter
    proves product-fingerprint invalidation, stable nominal domain kind, static
    constructor-registry convergence, and cold/incremental semantic equality.
```

### P1-RESOLVED-003 — final lifecycle/specification closure

```text
Observed:
    The effective data, enum, and type-level-domain specifications now describe
    the shipped unified/static foundation, and the P1 walkthrough/handoff are
    present.

Classification:
    resolved P1 lifecycle/specification closure.

Evidence:
    G5 focused suites are green; the plan is COMPLETE / IMPLEMENTED /
    FOCUSED_TESTED. The live P1→P2 handoff records the remaining generalized
    application, associated lookup, and P3 migration scope.
```

---

## 16. Known planning risks

| Risk | Severity | Owner plan |
|---|---|---|
| AST data/enum convergence loses GADT/case precision | critical | P1 |
| duplicate DataInfo/EnumInfo authority survives | high | P1 |
| type-data term accidentally considered ProperType | critical | P1/P2 |
| static-domain classifier becomes per-declaration compiler enum | high | P1 |
| heterogeneous `Applied` breaks row architecture | critical | P2 |
| trait/conformance assumes TypeId args are proper types | high | P2 |
| associated domain becomes second namespace | critical | P2 |
| associated specialization uses source strings | critical | P2 |
| metadata silently changes schema semantics | high | P1/P2 |
| TupleEntries duplicates CompleteTypePack | critical | P3 |
| intrinsic Tuple forces general data storage | high | P3 |
| enum keyword migration churns identity | high | P1/P3 |
| static syntax accidentally authorizes runtime expression execution | critical | P1/P2 |
| C3 grows into general compile-time evaluation | critical | all |

---

## 17. Consultation / amendment ledger

```text
INC-C3-1
Plan/task: LANG007.C3.P2 / T5.C — type-member user-domain parameters/results
Trigger: T5 lacked source-declaration validation/publication; selecting its
  owner/index affects T7 namespace lookup.
Decision: declaration semantic products own checked signatures keyed by stable
  owner-plus-canonical-selector identity; AssociatedSurface owns name admission,
  collisions, and lookup selection and refers to that identity. DeclarationTypeTable
  remains nominal type-form metadata. No separately queried type-member registry.
Architecture impact: authoritative split recorded in the amended T5 contract.
Plan amendment: Level 1 clarification recorded in P2 T5 on 2026-09-22; T5 publishes
  declaration products, T7 routes selection through AssociatedSurface.
Verification required: user-domain parameter/result formation, wrong-domain rejection,
  projection identity, same-owner collision through AssociatedSurface, and incremental
  invalidation of changed signatures without invalidating unrelated members.
Status: ADOPTED; implementation resumed. T5 publication is now implemented and focused
  tested; source-member application/projection selection still belongs to T7.

INC-C3-2
Plan/task: LANG007.C3.P2 / T6.B — runtime-data-owned associated type data
Trigger: T6 explicitly requires STOP AND CONSULT if the effective specification does
  not define unambiguous runtime-data owner syntax.
Observed: docs/specs/type-members.md defines Inherent members on nominal declarations,
  trait-owned Associated members, and DomainNative members; docs/specs/objects/data-objects.md
  does not define nested associated type-data declarations or their source placement.
  No accepted runtime-data-owned `type data` syntax/ownership rule was found.
Accepted decision (user ratified 2026-09-23): classes may declare associated `type data` in the
  class body; a runtime `data` owner's representation remains closed, and its associated
  `type data` is declared in an unguarded inherent `impl` in the owner's module, not in
  a conformance impl. Both forms have the nominal owner as semantic owner; impl location
  is provenance only. Use stable owner-plus-name declaration identity, specialized by
  the canonical owner generic environment. Reject duplicate associated domain names
  across inherent impl blocks and let AssociatedSurface decide collisions with other
  associated categories. Guarded and partial-target impl declarations are excluded.
  Associated type data has no runtime inhabitants or invocation target; its static cases
  belong to the associated domain.
Architecture impact: accepted rule preserves the closed runtime-data representation,
  stable owner-plus-name declaration identity, and AssociatedSurface name authority.
  The impl fragment supplies provenance only; the owner generic environment supplies
  canonical specialization.
Plan amendment: applied to T6.B. The owning normative specifications define the domain,
  ownership/identity, representation boundary, and inherent-impl placement with
  cross-references rather than competing definitions.
Verification required: parse and publish class-body and inherent-impl
  forms; reject declarations in runtime-data representation bodies and conformance impls;
  reject duplicate names across inherent impls; verify stable declaration identity over
  two generic owner specializations, distinct canonical concrete domains, and no runtime
  value production.
Normative amendments: docs/specs/type-level-domains.md, docs/specs/type-members.md,
  docs/specs/objects/data-objects.md, and docs/specs/objects/impl.md.
Status: ADOPTED; T6 resumed and now has focused parser, declaration-publication,
  collision, generic-specialization, and no-runtime-invocation evidence recorded
  below.

INC-C3-3
Plan/task: LANG007.C3.P2 / T7 — source type-member projection selection
Trigger: mandatory trigger 14 (a second source of canonical resolution would be
  introduced by a local workaround); trigger 7 if the type-formation and surface
  query boundaries imply materially different integration designs.
Observed: source `TypeMemberDef` signatures are published as identity-keyed
  declaration products and `AssociatedSurface` stores their admitted identities.
  `resolve_type_member_projection` in `types/annotation.rs` receives only
  `&dyn TypeResolver`, resolves intrinsic operations locally, and otherwise builds
  a symbolic projection; it receives neither the selected surface entry nor a
  checked source declaration product. `CheckingContext` tracks associated-surface
  dependencies, but the type-formation resolver has no corresponding selected-member
  product/dependency seam.
Expected architecture: T7 must select by owner/base through AssociatedSurface,
  retrieve the checked signature by stable `TypeMemberDeclarationId`, and preserve
  that identity in the canonical projection. No parallel name lookup is allowed.
Scoped work already present: T6 publishes associated static-domain products and
  T7 resolves associated static domains and singleton cases as static semantic
  results; heterogeneous owner applications preserve `RecordRow` in the canonical
  `StaticArgument` envelope. Focused tests pass for those slices.
Implementer hypothesis: type formation needs an explicit session/query-provided
  associated-member resolution context that records dependencies and returns the
  selected declaration identity/product, while leaving AssociatedSurface as the
  sole selection authority.
Alternative: route type-member projection formation through a new canonical
  semantic query at the declaration/snapshot boundary rather than extending the
  resolver context. These differ in query ownership and invalidation wiring.
Decision: user confirmed the session/query-provided seam: AssociatedSurface
  selects the member; checked declaration products are retrieved by stable
  TypeMemberDeclarationId; consumed surface and signature products participate
  in invalidation dependencies. No independent name registry or precedence path.
Plan amendment: required and applied in T7 before implementation resumes.
Verification required: source type-member selection through surface, wrong-domain
  rejection, projection identity across equivalent spellings, collision through the
  surface, and changed-signature invalidation with an unrelated sibling reusable.
Status: RESOLVED; implementation now selects source members only through the
confirmed surface/query seam. Focused wrong-domain and selected-identity tests pass.
Source projection identity is now focused-tested in field, alias, and generic-header
annotations. The user-directed phase reorganization remains incomplete: even a
same-module alias application of a source generic data declaration fails because
nominal generic signatures are published later. INC-C3-6 records the ratified
alias/member cycle rule and the new decision needed before extending the dependency
graph to generic headers. Preserve AssociatedSurface as the only selector and do not
publish provisional or parallel lookup surfaces. Independent T4/T8 work may continue.

```

### INC-C3-4

Incident report: [INC-C3-4.md](INC-C3-4.md)

Plan/task: LANG007.C3.P2 / T8 — nominal static-case source-index identity
Trigger: mandatory trigger 11 (two plausible semantic authorities have different
  invalidation consequences) and trigger 14 (do not reconstruct canonical targets
  from source spelling).
Observed: the original defect was that nominal `Shape::Scalar` received
  `TypeOnlyRuntimeValue` before case selection, leaving no source target. The accepted
  fix now converges nominal and associated domains at canonical static-case selection.
Expected architecture: T8 targets derive from canonical semantic selection and stable
  domain/case identity, not token spelling or a second case-name registry.
Decision (advisor response adopted): resolve the nominal owner to
  `StaticDomainDeclarationId`, select its case through canonical static-domain
  semantics, publish the existing `StaticTerm` resolution with stable `StaticCaseId`,
  then retain `TypeOnlyRuntimeValue` for illegal runtime use. `CallableSourceAttachment`
  consumes this product and must not search declarations or source spelling. The
  semantic static-domain/member-selection dependency owns case-edit invalidation.
Plan amendment: T8's existing target and invalidation requirements are clarified by
  INC-C3-4; no second lookup authority or new resolution variant is introduced.
Verification: nominal target/runtime-diagnostic, missing-case, and rename/cold-parity
  tests plus semantic crate check passed as recorded in INC-C3-4. On the current HEAD,
  the same focused nominal-case set selected 3 and passed in a clean worktree; the
  shared dirty checkout currently fails earlier while parsing the unrelated local
  `fiber.ph` edit. T8 remains open for cross-module metadata/catalog validation and
  broader persistence/import parity.
Status: RESOLVED — nominal static-case selection is implemented and focused-tested.

Focused evidence: semantic crate check; `adts::declarations` (26 passed);
`adts::associated` (22 passed, 5 gated); `associated::invocation` (1 passed);
`incremental::type_members` (8 passed); `nominal_static_case_rename` (1 passed);
`integration::metadata` (23 passed); `incremental::adts` (13 passed, 3 gated);
core crate check; and `phalcom-type-meta` (9 passed). The semantic library-unit
filter is baseline-blocked by the previously recorded missing `generic_binder_header`
test initializer. Workspace formatter check remains baseline-failing on untouched
files; no broad formatting was applied.

### INC-C3-5 — T7 all-annotation phase ordering

Incident report: [INC-C3-5.md](INC-C3-5.md)

Plan/task: LANG007.C3.P2 / T7 — all declaration annotations
Observed: aliases lower before generic headers; callable/field signatures are
  materialized before data/enum and `AssociatedSurface` publication; the resolver
  receives the associated type-member context only after that publication.
Classification: known implementation gap from the accepted T7 amendment, not a
  newly reproduced semantic failure or a new architecture decision.
Decision: continue with the user-directed phase reorganization using one canonical
  surface and the INC-C3-3 identity/dependency seam. Preserve alias/generic-header
  dependency contracts. Consult only if live dependencies require changing those
  contracts or introducing another authority.
Verification: not run for the incident report; add annotation-phase, wrong-domain,
  canonical identity, and incremental-dependency tests with the implementation.
Status: OPEN — implementation and focused verification remain required.

### INC-C3-6 — T7 alias/member signature dependency ordering

Incident report: [INC-C3-6.md](INC-C3-6.md)

Plan/task: LANG007.C3.P2 / T7 — all declaration annotations
Trigger: mandatory trigger 7 (phase ordering may change the declaration-signature
  dependency contract); trigger 16 for cycle-policy changes.
Observed: the field-annotation regression initially selected a symbolic projection
  without its source declaration identity; the implementation now fixes this for
  field, alias, and generic-header cases. Alias lowering precedes members, while checked
  member signatures resolve domains through the alias-populated resolver. A same-module
  generic `Tensor` application in an alias still fails as unresolved because nominal
  generic signatures are built after alias lowering. The current graph models aliases
  and source members, not generic headers.
Decision: user accepted on 2026-09-23 that cross-kind declaration-signature cycles
  are rejected with a deterministic diagnostic; ordinary symbolic projection
  normalization remains valid. `docs/specs/type-members.md` and the T7 plan are
  amended; `TypeSignatureCycle` is implemented for alias/member SCCs. User has now
  accepted including nominal/trait generic headers in the unified graph: reject an
  SCC containing a source type-member signature under the same deterministic rule,
  while preserving alias-only and header-only cycle behavior.
Verification: `incremental::type_members` selects 12 and passes, including field,
  alias, generic-header identity, collision, wrong-domain, incremental-surface, and
  mixed-cycle tests. The same-module `Tensor` alias probe fails and is recorded in
  INC-C3-6; it is not a passing gate.
Status: PARTIALLY RESOLVED — cycle policy and focused implementation are in place;
  all-annotation scheduling, complete AssociatedSurface publication, and exact
  invalidation remain open. Generic-header participation and mixed-SCC policy are
  decided; implementation and focused cycle/application tests remain. See
  [advanced-agent handoff](LANG007.C3.P2-advanced-agent-handoff.md).

### INC-C3-7 — T7 graph invalidation and shell edit triggers

Incident report: [INC-C3-7.md](INC-C3-7.md)

Plan/task: LANG007.C3.P2 / T7 — incremental signature dependencies and shell
publication.
Trigger: mandatory trigger 15 (nonlocal incremental invalidation is uncertain) and
trigger 16 (the live retained-product lifecycle does not satisfy the accepted plan).
Finding: the Alias/Header/Member graph originally scheduled and classified SCCs but
was not retained or traversed for reverse invalidation. Existing alias/header maps did
not cover cross-kind member edges, and retained member products could bypass the
scheduler. The shell builder also failed to seed affected owners for behavior-only
inherent impl edits, leaving collision facts stale.
Decision: use the same stable-identity graph for scheduling, SCC handling, and reverse
invalidation. Traverse old reverse edges from changed declaration nodes before dropping
retained products; rebuild the exact consumer closure, refresh edges for recomputed
products, and apply SCC withholding before publication. Include old and new resolved
owners for every changed namespace-affecting inherent impl. Keep `AssociatedSurface`
as the sole selection/collision authority. No plan amendment is required; these
requirements are already in T7's exact invalidation contract. The current working diff
now persists the graph/reverse edges, seeds old-graph closure from source-level
declaration fingerprints before dropping products, and retains old/current inherent
impl targets for surface rebuilding.
Verification: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib`
and `git diff --check` pass. Focused semantic tests were not run against this diff; a
prior test attempt was blocked before the selected case by the canonical Universe
parser failure documented in the incident report.
Status: IMPLEMENTED / UNVERIFIED — compile checked; exact dependent reuse,
behavior-only collision refresh, and cold/incremental parity remain unverified.

Current focused update (2026-09-23): the semantic test target compiles after
updating T4/T7 test fixtures, and `incremental::type_members` selects and passes
12 existing tests. A new `Tensor<Int, Shape::Scalar>` alias/header regression
fails on its initial cold analysis: zero-payload `ExactEnumCase` formation
forwards `Shape::Scalar` to contextual trait-associated projection resolution
and emits `AnnotationUnresolved` for missing conformance evidence. The accepted
static term must instead be selected through canonical static-domain/case
authority, with exact dependency recording. The retained-graph edit and cold
parity portions of this regression remain unverified. The canonical Universe
`fiber.ph` parser defect was repaired separately; it had masked these tests.

### INC-C3-8 — T4 mixed inference solver ownership and frame scope

Incident report: [INC-C3-8.md](INC-C3-8.md)

Plan/task: LANG007.C3.P2 / T4 — heterogeneous applied inference with row arguments.
Trigger: mandatory triggers 3 and 5 (solver ownership and lifetime); T4 STOP/CONSULT.
Finding: the original `type_id_to_inference_with_rows` path made an entire `Applied` or
`StaticData` term opaque when one child was a `RecordRow`. Ordinary constraints lived in
a retained query-context `InferenceSession`, but row variables lived in a call-local
`GenericApplicationSession`.
Decision: preserve separate `InferVarId` and `RecordRowVarId` identities. Amend T4 so a
query-context-owned coordinator owns both existing solvers, routes mixed equations, and
uses explicit frame-scoped row roots. Row variables persist to query-context teardown;
unresolved cross-frame equations remain for the parent, and child results cannot
materialize unresolved outer rows. No row identity enters canonical products. The T4
plan is amended with this contract and required acceptance cases before implementation.
The row solver is generalized over field payloads: it solves labels/tails and emits
same-label payload pairs to ordinary inference using `InferenceTerm` payloads without
fake `TypeId`s. `InferenceCoordinator` owns the ordinary session and a separate row
solver, alternates both to a fixed point, materializes resolved field terms, and checks
cross-domain occurs through row fields and substitutions. Nested views inherit active row
parameter mappings; a child can retain a same-context symbolic result dependent on an
unresolved ancestor row instead of materializing a canonical generic instantiation.
Verification: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib` passed.
No T4 tests were added or run; mixed-argument behavior and nested resolution remain
unverified. Existing T7 working diff remains `IMPLEMENTED / UNVERIFIED` as recorded under
INC-C3-7.
Status: IMPLEMENTED / UNVERIFIED — implementation present; focused acceptance remains open.

### INC-C3-9 — T8 typed catalog and metadata validation boundary

Incident report: [INC-C3-9.md](INC-C3-9.md)

Plan/task: LANG007.C3.P2 / T8 — stable metadata catalog and validation.
Trigger: metadata could validate that a static constructor's owner matched its domain,
but the serialized artifact had no declaration/case catalog for membership, payload
kind, or associated owner-environment checks.
Decision: schema v8 adds a typed catalog keyed by stable nominal/associated declaration
identity. Each entry records ordered owner parameter kind/mode pairs and sorted cases
with ordered payload kinds. Associated owner specialization remains in
`StaticDomainRef::Associated`. Schemas v1-v7 keep their prior validation behavior; the
new catalog defaults empty when decoding older bundles, and older readers reject v8.
Snapshot-backed export consumes canonical current data/enum and associated declaration
products. It must not infer current declaration membership by enumerating TypeStore
interning history. v8 validation checks catalog membership, owner arity/kinds, case
membership, payload arity/kinds, and global/scoped Applied argument and result kinds.
The bounded implementation is present and `cargo check` passes for type-meta,
semantic, and core libraries. No tests were run. Cross-module catalog composition,
import reconstruction, provider invalidation, and cold/incremental parity remain open.
Status: IMPLEMENTED / UNVERIFIED — schema and exporter slice compiles; T8 remains partial.

Resolution update (2026-09-24): schema-v8 acceptance is now focused-tested. The
compatibility suite passes 14/14. The metadata integration module passes its new
round-trip/scoped cases; its one unrelated alpha-equivalence failure reproduces at the
clean `9c150747` predecessor. Cross-module provider/cold parity, source-index identity,
incremental ADT refresh, and TypeStore-history exclusion gates also pass. `INC-C3-9`
is RESOLVED / ACCEPTED with the baseline failure retained below as nonblocking evidence.

### INC-C3-10 — static cases in declaration annotations

Incident report: [INC-C3-10.md](INC-C3-10.md)

Plan/task: LANG007.C3.P2 / T5/T7 — static aliases and all-annotation availability.
Observed: the new `Tensor<Int, Shape::Scalar>` alias regression fails in cold
analysis before its incremental assertions. Zero-payload `ExactEnumCase`
formation routes to contextual trait-associated projection, while checked
nominal static-case products are built after aliases. The T5 static-alias
example and T7 all-annotation contract therefore remain unimplemented.
Decision: accepted architectural verdict in the attached INC-C3-10 consultation.
Publish a declaration-owned structural case catalog before signature checking;
its stable IDs use the existing structural `Selector`, which distinguishes a
singleton getter from an explicit-empty constructor. Checked payload signatures
are separate products in the Alias/Header/Member/StaticCaseSignature graph.
Only checked-signature SCCs containing a Member or StaticCaseSignature are
rejected with one deterministic `TypeSignatureCycle`; catalog/header recursion
is legal. Annotation and expression selection consume the same catalog; the
store only materializes selected terms. Catalog and selected signature reads
are exact dependencies, and associated namespace publication depends on the
domain header rather than payload signatures.
Plan amendment: apply this split in T7 before treating static alias, case
selection, or incremental gates as accepted. Preserve catalog identity when a
payload signature is invalid, and preserve cold/incremental parity.
Status: ARCHITECTURE RESOLVED / IMPLEMENTATION IN PROGRESS / UNVERIFIED.

### INC-C3-12 — runtime intrinsic facade binding prerequisite

Incident report: [INC-C3-12.md](INC-C3-12.md)

Plan/task: LANG007.C3.P3 / T4/G4 — Unit/Tuple/Record intrinsic runtime facade migration.
Trigger: mandatory triggers 1, 2, 7, and 12; T4's prerequisite and P3's ordered
  migration rule require a runtime facade binding that is not established by T2/G2.
Observed: the declaration table registers TypeOnly intrinsic kinds and existing
  Tuple/Record class-owner descriptors, but no runtime intrinsic Data declaration
  for Unit/Tuple/Record. Runtime product `Statement::Data` goes through
  `compile_data`, which emits `Bytecode::Data` / `FinalizeData`; native source
  presentations are indexed from Class and sum declarations. Existing structural
  TypeStore and runtime class authorities are present.
Decision: user ratified on 2026-09-24 that T4 first establishes and tests the
  declaration-identity binding and existing-owner compiler/runtime reuse path,
  then migrates source presentation. Keep within T4/G4 as a Level 1 sequencing
  clarification; no corrective plan.
Architecture impact: do not migrate Unit/Tuple/Record source or add speculative
  compiler/runtime handling until the binding ownership and reuse path are ratified.
Plan amendment: Level 1 clarification recorded in INC-C3-12.md and the active
  P3 plan. T4/G4 may resume.
Verification required: prove facade declarations resolve to the existing canonical
  Unit/Tuple/Record semantic/runtime identities, do not emit generic data lowering,
  retain native descriptor/source targets, and preserve exact runtime census.
Status: ADOPTED — T4/G4 implementation resumed.

### INC-C3-13 — intrinsic type members in inherent impls

Incident report: [INC-C3-13.md](INC-C3-13.md)

Plan/task: LANG007.C3.P3 / T4/G4 — migrate Tuple/Record source members while
preserving intrinsic type-member identity.
Trigger: mandatory triggers 1, 2, 8, 12, and 18; the planned source shape puts
intrinsic type members in inherent impls, but `ImplMember` cannot represent
type-member declarations and semantic publication currently owns them through
class member syntax.
Observed: behavior/native method anchors can move through existing impl support;
type members cannot. Extending that syntax changes associated-surface publication
and cannot safely be treated as a mechanical migration.
Decision: user chose to extend `ImplMember` and the semantic associated-surface/
type-member pipeline for intrinsic type members on inherent impls, retaining
owner-plus-selector identity and one `AssociatedSurface` authority.
Architecture impact: T4 includes the bounded AST/parser and semantic/source-index
extension before Unit/Tuple/Record source migration.
Plan amendment: T4/G4 amended in INC-C3-13.md; no corrective plan.
Verification: semantic owner-binding test, compiler existing-Unit-class reuse
test, native Tuple facade/impl-anchor test, intrinsic type-member parser
regression, canonical Tuple associated-surface publication, and source-index
target regression each passed.
Status: ADOPTED — T4/G4 implementation resumed.

### INC-C3-14 — intrinsic structural application and runtime class projection

Incident report: [INC-C3-14.md](INC-C3-14.md)

Plan/task: LANG007.C3.P3 / T4/G4 — Tuple/Record intrinsic data migration.
Trigger: mandatory triggers 1, 3, 5, 7, 11, and 12; the P3 endpoint needs
structural generic applications while native method import needs the existing
runtime class type, and Record's argument remains row-owned.
Observed: Unit intrinsic migration and inherent-impl type-member publication
pass focused checks. Replacing canonical Tuple with `intrinsic data
Tuple<***Es>` fails semantic bootstrap because native import cannot find a
proper `Tuple` owner form after runtime-owner registration. Separately, the
current type-lambda beta path accepts `TypeId` arguments and cannot apply a
RecordRow argument to produce `TypeData::Record(RecordRowId)`.
Decision-time status (superseded by the adopted user ruling): the recommended
path was one intrinsic-form application route backed by the existing
runtime-owner binding: Tuple maps to `TupleFromPacks`, Record maps through the
existing row-owned `RecordRowId` path, and native signature import continues
to use the existing runtime class form.
Plan amendment: P4 records the adopted application/runtime projection
contract. The Tuple data experiment was reverted before adoption; no
TypeId/RecordRow boundary was weakened.
Decision-time verification: parser and semantic inherent-impl member tests
passed; Unit canonical bootstrap passed; the earlier Tuple migration attempt
failed as recorded in INC-C3-14.

Final resolution: the user adopted the recommended application/runtime
projection contract and confirmed the inherent-impl type-member syntax path.
Tuple application uses the existing CompleteTypePack/TupleFromPacks route,
Record application retains RecordRowId, native imports use the existing
runtime-class projection, and Unit remains store.unit(). P4 owns the
corrective semantic/runtime projection work and its focused regressions; see
the accepted decision in [INC-C3-14.md](INC-C3-14.md) and the active P4 ledger.

### P4 structural behavior-owner refinement — user-ratified 2026-09-25

Source: the user's latest P4 continuation guidance, which supersedes the earlier
attached review's restriction on anonymous structural Tuple/Record targets.

Decision: all canonical structural Tuple types are behaviorally owned by the
canonical Universe `Tuple` declaration; all canonical structural Record types
are behaviorally owned by the canonical Universe `Record` declaration. The
semantic target remains structural. Direct structural shapes and qualified
family applications converge to one target type before indexing,
applicability, overlap, or collision checks. Only the corresponding Universe
owner module may author these impls. Generic family heads use the existing
covering/specialized machinery, and RecordFields remains row-owned under
`RecordRowId` / `GenericInstantiation`.

Architecture impact: centralize structural-family owner classification; do not
add source-provenance owner state, structural target variants, or a parallel
member registry. Structural targets authored from foreign modules report an
ownership/coherence error.

Plan amendment: P4 T6.8 records implementation and coverage obligations.
`docs/specs/objects/impl.md` now states the normative target and authorization
rules. Verification required: direct and family-spelled Tuple/Record targets,
covering and specialized domains, equivalent-spelling canonicalization and
overlap, owner-module positives, foreign-module negatives, and row-aware
materialization. Status: ADOPTED / IMPLEMENTATION PENDING / UNVERIFIED.

### P4 generic pack application syntax and binder defaults — user-confirmed 2026-09-25

P4 generic pack contributions require explicit `*`, `**`, or `***` expansion,
or expansion of an already-bound pack variable. Direct mixed contributions
such as `Foo<Int, String, x: Bool>` and `Record<name: String>` remain outside
P4. Fixed ordinary arguments may precede the variadic tail.

When omitted, binder mode fixes the default kind without contextual inference:
`T: Type`, `*Ts: TypePack`, `**Fs: RecordFields`, and
`***Es: CompleteTypePack`. Explicit compatible kind annotations remain
permitted. Expansion mode and semantic kind remain separate concepts.

The P4 plan §5.4 and normative type-level-domains specification record this
boundary. Verification required: parser and semantic rejection of direct pack
contributions; explicit structural and bound-pack expansion; fixed-prefix
application; omitted binder defaults; and explicit compatible annotations.
Status: ADOPTED / IMPLEMENTATION IN PROGRESS / UNVERIFIED.

### INC-C3-15 — T5.2 runtime heterogeneous applied-type identity — 2026-09-27

Plan/task: LANG007.C3.P4 / T5.2 — exact runtime specialization descriptors for
  nominal data with heterogeneous static arguments.
Trigger: mandatory triggers 1, 3, and 11. `RuntimeTypeRecipe::Template` can
  receive distinct callable-owned type bindings, but runtime applied-type
  instantiation currently rejects `StaticArgumentRef::RecordRow`; closing the
  recipe erases distinct exact row specializations.
Observed: semantic and metadata representations already distinguish
  `Term(TypeId)` from `RecordRow(RecordRowId)`. `RuntimeOverlayTypeNode::Applied`
  currently narrows this to `RuntimeTypeRef` arguments, and
  `instantiate_type_ref` returns `None` for row arguments. Generic callable
  construction therefore fails once the existing type-parameter detector
  correctly marks a row-nested parameter as a template.
Decision: extend the existing runtime overlay's `Applied` argument envelope
  heterogeneously as `Term(RuntimeTypeRef) | RecordRow(RuntimeRecordRow)`.
  Keep rows structurally distinct from proper `RuntimeTypeRef` and
  `RuntimeOverlayTypeNode::Record`. Instantiate ordinary type parameters inside
  closed row field types from the existing `RuntimeTypeEnvironment`; preserve
  row identity through the existing overlay interner so exact rows with
  `payload: Int` and `payload: String` remain distinct. Repeated identical
  specializations must intern identically.
  Do not broaden runtime environments to row-valued bindings, add row solving,
  introduce a second type/descriptor registry, or create specialization-owned
  behavior classes. Unresolved row tails fail exact instantiation rather than
  being erased or closed.
Architecture impact: this faithfully projects the established heterogeneous
  static-argument model into the existing runtime type interner. The runtime
  descriptor key remains `(declaration owner, exact runtime type)`, with one
  behavior class per declaration and exact descriptor/layout state per
  specialization.
Plan amendment: T5.2 may resume with closed row substitution only; runtime
  row-parameter bindings and VM row solving are explicitly not added by this
  amendment.
Verification required: row preservation and type substitution; distinct
  Int/String exact runtime identities despite equal physical layout; repeated
  specialization interning; row-vs-proper-Record distinction; open-tail
  rejection; unchanged type-only Applied controls; and the existing T5 class,
  descriptor, layout, field-access, and generic-callable regressions.
Implementation evidence: the runtime `Applied` overlay now retains
  `Term(RuntimeTypeRef)` and `RecordRow(RuntimeRecordRow)` separately; closed
  row fields substitute through the existing type environment and interner,
  while unresolved tails fail exact instantiation. Legacy type-only argument
  projection does not flatten row arguments. Low-level row substitution,
  interning, separation, tail rejection, equal-layout exact descriptor, and
  ordinary Applied reflection controls passed, as did focused MyTuple,
  MyRecord, generic-callable, Envelope fixed+expanded, and fixed-only source
  controls plus `cargo check -p phalcom-core` (all focused selectors selected
  one test and passed). No broad suite was run.
Status: ADOPTED; this T5.2 runtime representation slice is IMPLEMENTED /
  FOCUSED_TESTED. T5 and P4 remain incomplete; this evidence does not certify
  the remaining T5 acceptance or release gates.
Supplemental discriminator: the focused source test
  `p4_empty_layout_row_specializations_keep_exact_identity_through_generic_callable`
  fails at `PhantomRow()` with the existing `generic data type could not be
  instantiated` error. A single temporary trace, removed immediately after
  capture, observed a `Template(Base { pool: 2, node: 1 })` recipe, a live
  `Factory.make<T>` environment binding T to the runtime Int type, and one
  `RecordRow` argument with no fields and an unresolved declaration-owned
  `PhantomRow.Fs` tail (parameter index 0). Exact instantiation correctly
  rejects that open tail; the T binding does not select a row for it. The
  passing MyRecord generic-callable path has source evidence for its closed
  `payload: T` row, and closed-row substitution passes at the low-level runtime
  seam. This is a semantic expected-result/constructor-publication gap, not a
  reason to relax the open-tail rejection rule. The discriminator is retained
  as a required semantic acceptance test; see the superseding INC-C3-16 ruling.

### INC-C3-16 — Phantom generic contextual construction and constructor surface — 2026-09-27

Plan/task: LANG007.C3.P4 / T4–T5 semantic/runtime boundary.
Trigger: the focused empty-layout `PhantomRow` constructor discriminator reaches
  runtime with an unresolved declaration-owned RecordFields tail even though
  its generic callable has expected result `PhantomRow<{ payload: T }>`; the
  accepted language already permits phantom generic data, nullary data
  constructors, and generic inference from surrounding result context.
Observed: `PhantomRow()` is applied through the existing primary data
  constructor/generic application path; this nullary declaration has no
  product expansion, so no RecordFields constructor adapter is attached. The
  expected-result relation solves the nested `Applied` RecordRow equation to
  `{ payload: T }`, but row proof marking handled only a top-level
  `InferenceTerm::Record`. Publication therefore returned
  `Unknown(NoTypeEvidence)`, and lowering retained the open constructor
  template. After semantic publication was repaired, the core discriminator
  reached execution but exposed a second defect: `LoadDataSingleton` cached a
  `RuntimeTypeRecipe::Template` descriptor across generic invocations, so the
  Int, String, and repeated Int values all used one descriptor. Nullary data
  is represented by `Value::DataSingleton`; the test must read its descriptor
  directly rather than expecting a heap `DataObject`. The initial adapter
  hypothesis was disproved by the focused semantic trace.
Decision: the prior provisional deferral in INC-C3-15 is superseded. Semantic
  expected-result specialization of otherwise-underconstrained declaration-
  owned data constructor parameters is in scope. `PhantomRow()` under expected
  `PhantomRow<{ payload: T }>` must publish that exact symbolic closed-row
  specialization before lowering. Context-only selection is `Assumed`; it
  must not override conflicting source evidence or downgrade stronger
  independent evidence. Without context, phantom parameters remain
  underconstrained; nullary construction does not default a phantom row to
  `{}`. This is the ordinary generic application rule, not a RecordFields-only
  special case.
Architecture impact: no new inference engine, canonical identity, constructor
  identity, runtime row-valued environment, runtime row solver, second registry,
  or specialization-owned behavior class. Runtime exact instantiation must
  continue rejecting unresolved row tails; semantic analysis must resolve
  context-selected rows before lowering.
Plan amendment: the semantic expected-result/primary-constructor gap is an
  in-scope T4/T5 completion discovered while validating T5. Existing T4 row
  calculus and proof bridge remain accepted and are not reopened.
Required verification: ordinary `Phantom<T>()` context selection and
  no-context underconstraint; PhantomRow context selection with `Assumed`
  evidence; conflicting source/context evidence reports the ordinary inference
  conflict; the empty-layout generic-callable core discriminator proves exact
  descriptor separation/reuse through `DataSingleton`; unresolved runtime row
  tails remain rejected. Then resume P4's numbered execution order.
Focused evidence: pinned-toolchain semantic filter `phantom_data_constructor`
  selected 4 and passed 4; the conflict, existing source-established-row,
  and existing expected-result-row selectors each selected/passed 1. Core
  `p4_empty_layout_row_specializations_keep_exact_identity_through_generic_callable`
  selected/passed 1 after nullary template recipes stopped sharing a per-site
  descriptor cache. The low-level open-tail rejection, existing nullary
  singleton identity, and zero-allocation/specialization-sensitive singleton
  selectors each selected/passed 1. These are focused results, not P4/T5
  acceptance or release certification.
Status: ADOPTED — semantic constructor contextual specialization is authorized
and implemented in this focused slice; T5 and P4 remain incomplete.

### INC-C3-17 — scoped RecordFields application arguments — 2026-09-27

Plan/task: LANG007.C3.P4 / T6.2 — canonical and scoped intrinsic application
  use the same family contract while preserving heterogeneous row identity.
Trigger: required T6.2 scoped/canonical convergence cannot represent a
  resolver-owned `RecordRowId` as a scoped generic application argument.
Observed: canonical `RecordFamily<**Fs>` preserves
  `StaticArgument::RecordRow`, while scoped lowering has no row-valued free
  carrier: `ScopedTypeData::OpenRecord` is a proper structural Record type and
  `scoped_kind` classifies it as `Type`. The resolver-owned row itself cannot
  be converted to that shape without changing identity.
Decision: add a first-class scoped RecordRow carrier, distinct from
  `ScopedTypeData::OpenRecord`, `TypeId`, and proper `Record`. Preserve the
  canonical row parameter/field/tail identity when lowering and when a scoped
  application reduces. Thread it only through the existing scoped arena's
  kinding, interning/equality, and substitution/application consumers; do not
  add row-solving authority, encode a row as `TypeId`, or erase a row into a
  structural Record. Callable-owned free rows remain the accepted source
  context; this does not enable RecordFields type-lambda binders.
Architecture impact: extends the existing alpha-normalized scoped type
  calculus with the row argument category already required by canonical
  `StaticArgument::RecordRow`; canonical identity/solver/runtime ownership is
  unchanged.
Plan amendment: T6.2 implementation must carry scoped RecordRows as
  heterogeneous application arguments and beta-reduce them through
  `TypeStore::apply_static_form_with_arena`; unresolved/bound row tails must
  remain explicit and never be closed or converted to proper Record. The T8
  metadata preflight is amended to add a schema-v9 scoped `RecordRow` node;
  schemas v1-v8 remain readable and v8 must reject this v9-only node. Export
  fields as Type-kinded references and preserve Closed, Bound, stable
  FreeParameter, and RecordRow-kinded Projection tails.
Verification required: scoped callable-owned free row under an enclosing
  type lambda; closed row specialization; row-vs-proper-Record distinction;
  alpha-equivalence and substitution; ordinary Type-only scoped applications;
  canonical RecordRow controls; scoped intrinsic Tuple/Record zero and symbolic
  forms. Existing restriction against declaring RecordFields type-lambda
  binders remains unchanged.
Focused evidence: `record_fields_applications_require_expansion_and_preserve_row_arguments`,
  `scoped_record_row_type_fields_substitute_and_alpha_normalize`,
  `scoped_type_lambda_record_fields_binders_remain_unsupported`,
  `scoped_bound_row_tail_remains_domain_specific`,
  `scoped_zero_tuple_and_record_applications_select_their_declared_domains`,
  `scoped_complete_pack_expansion_projects_a_tuple_with_bound_elements`,
  `type_lambda_applications_preserve_canonical_identity`,
  `zero_user_defined_variadic_applications_retain_the_declared_argument_domains`,
  and `intrinsic_family_keeps_its_static_contract_separate_from_runtime_form`
  each selected/passed 1. `schema_v9_scoped_record_rows_round_trip_all_tail_forms_and_v8_remains_compatible`
  selected/passed 1; the complete `schema_compat` target selected 15 and passed
  15. `metadata_round_trip_preserves_heterogeneous_application_and_scoped_static_domain_bound`
  selected/passed 1. The exporter integration assertion verifies an Applied
  scoped row retains its RecordRow kind, field name/bound field identity, and
  closed tail through validate/JSON round-trip.
Status: ADOPTED; scoped row carrier and schema-v9 projection are focused-tested,
  while T6.2/T8 implementation remains in progress pending review and remaining
  acceptance coverage. Overall P4 remains IN_PROGRESS / PARTIAL / UNVERIFIED.

### T13 amendment — guarded conformance evidence for ordinary trait dispatch — 2026-09-29

Plan/task: LANG007.C3.P4 / T13 — `TakeIterator.toList` fails after bootstrap
  reaches the existing blanket `impl<I> Iterable for I where I |= Iterator`.
Trigger: ordinary trait dispatch discovers explicit `ConformanceIndex`
  contributions, while valid guarded/derived conformances are resolved by the
  canonical generic conformance proof engine but are not considered as
  selector candidates.
Adopted decision: guarded/derived conformances are valid evidence sources for
  ordinary trait dispatch. Discover candidate members by selector, then ask the
  one canonical conformance resolver for applicability and proof. A successful
  Exact, Forwarded, or Derived proof may publish the existing
  `TraitDispatchSite::Evidenced` product and must retain rule identity,
  substitutions, nested proof provenance, and dependencies.
Constraints: add no proof solver or dispatch-local proof authority. Preserve
  ambiguity/coherence behavior, stable identities, the existing compiler/VM
  trait-default path, language behavior, ABI, and Priority 3 classifications.
  Overall T13/P4/C3 status remains IN_PROGRESS / PARTIAL / UNVERIFIED.
Verification required: exact and derived trait defaults, concrete and symbolic
  receivers, unsatisfied guards, unrelated same-selector traits, nested proof
  provenance, ambiguity/exact-derived overlap, inherent precedence, actual
  `receiver.toList` evidence for a canonically known Iterator receiver with an
  ineligible known-receiver control, and focused invalidation if supported by
  the existing dependency graph. The natural `.iter.take(10).toList` chain is
  separately blocked under INC-C3-20 and is not an acceptance gate.

### Adopted INC-C3-20 ruling — Outcome B / known-receiver gate — 2026-09-29

The bounded callable-result census established Outcome B for unannotated
trait-default bodies: their return results are not a canonical published
return-signature contract consumable by trait dispatch. Ordinary class callable
inferred-return publication exists and is outside this ruling; `Iterator.take`
is additionally blocked by `RecursiveFixpoint`. T13 does not add
trait-default return inference/publication; that is a separate semantic design
item. T13's actual-expression gate is amended to use
  a known receiver and a negative ineligible receiver. The canonical-Universe
  ListIterator source test proves `Iterable::toList`, guarded Derived provenance,
  nested Exact Iterator evidence, and rule/nested-proof dependencies; the Int
  control receives no Derived `Iterable::toList` evidence. The new core selector
  `ordinary_trait_lowering_preserves_exact_and_derived_dispatch_evidence` passed
  1/1 and verifies full Exact/Derived selection equality between semantic sites
  and compiled lowering specs. Explicit `cargo check -p phalcom-semantic` and
  `cargo check -p phalcom-core` passed (one existing dead-code warning each).
  The natural chain remains blocked/ignored under INC-C3-20. Priority 3 is
  unchanged; P4/C3 remains IN_PROGRESS / PARTIAL / UNVERIFIED. The §13 focused
  matrix is satisfied and supervisor-reviewed; stop at architect ruling §15.
  G5 collection aggregates remain unpassed/uncertified and G6 incomplete. No
  further broad, corpus, or workspace rerun, and no trait-default return
  inference, is authorized under this ruling. Crossing this boundary requires
  new user or architect authorization. The separately adopted INC-C3-21 / P4.R1
  direction does not reopen this stop; Priority 3 remains unchanged.

### INC-C3-20 — unannotated trait-default return blocks actual guarded dispatch — 2026-09-29

Status: RULING ADOPTED; Outcome B established, with natural integration still
BLOCKED. See
[`INC-C3-20.md`](INC-C3-20.md) for the required incident packet. The actual
inline-method selector historically failed 1/1: `Iterable::iter` is selected exactly, but
its unannotated default return remains `Unknown(UnannotatedDeclaration)`, so
the `.take(10).toList` receiver products remain unknown and no
`Iterable::toList` evidence is published. Exact default, concrete guarded,
symbolic guarded, coherence, and core compile selectors pass. No runtime
fixture or aggregate collection test was run for the ruling amendment. The
adopted disposition forbids return-signature inference in T13 and replaces the
acceptance expression with a known-receiver source gate. Priority 3 and overall
P4/C3 verification remain unchanged.

### INC-C3-21 — canonical result contract for unannotated trait defaults — 2026-09-29

Status: ADOPTED. The architect authorized canonical inferred result contracts
for body-bearing callables, including trait defaults, through the existing
callable signature authority. This is scoped in companion plan
`LANG007.C3.P4.R1`. T1 completed read-only: ordinary signatures enter
`CallableSignatureTable` before body analysis, while trait-default signatures
were initially omitted from that authority. T2/G1 is now IMPLEMENTED /
FOCUSED_TESTED through the existing canonical signature table. Trait defaults
are seeded from their source surface contract, then the normal result refresh
publishes body summaries into that table; evidenced trait-default consumers
transiently specialize the table result using the already-selected proof and
record `CallableSignature` dependencies. No independently mutable
`TraitSurfaceMember` result copy or dispatch-local proof search was added.
  `trait_default_inferred_return_is_canonical_and_independent_of_caller_expectation`
and `branch_derived_tail_type_is_published_to_unannotated_callable_signature`
each selected and passed 1/1. T3/G2 is now IMPLEMENTED / FOCUSED_TESTED
under INC-C3-22; its focused incremental matrix passed 1/1. The ruling
preserves explicit annotations, caller independence, symbolic generic/Self
substitution, existing metadata/signature representation, and the
no-general-recursive-fixpoint boundary. T4/CV-05–CV-07 is now IMPLEMENTED /
FOCUSED_TESTED. With the pinned toolchain and cleared local Rust flags, each
exact command selected and passed 1/1:

```text
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic semantic::incremental::callable_dependencies::inferred_return_refresh_preserves_stable_products_and_counts_final_disposition_once -- --exact
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic semantic::incremental::product_stability::inferred_cross_module_result_updates_dependents_and_matches_cold_analysis -- --exact
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic semantic::integration::metadata::cross_module_inferred_callable_result_round_trips_as_compiler_inferred_metadata -- --exact
```

These cover same-result inferred fingerprints/body reuse, changed-result
invalidation and cold parity, explicit-result body-only stability, and a
two-module Universe workspace metadata round-trip carrying
`PublishedTypeAuthority::CompilerInferred`. The former ordinary counter
expectation was stale: the observed disposition is one body recomputed and two
reused; semantic results and stable signature products remain Int. T5/CV-08's
explicit-result direct and mutual recursion controls pass 1/1 each; existing
unseeded controls remain epistemically Unknown but publish no diagnostic. The
CV-09 probe shows `Iterator.take`'s generic constructor result is known and
specialized, while the call status is blocked because constructor parameter
`n` has no declared type. This is not evidence of a semantic/query cycle. The
two policy questions are captured in [`INC-C3-23.md`](INC-C3-23.md); no
production change is authorized before its ruling. T5/G4 remains partial, T6
is not started, and this does not amend T13 or certify G5/G6. See
[`INC-C3-21.md`](INC-C3-21.md) and the
[`P4.R1 plan`](LANG007.C3.P4.R1-canonical-inferred-result-contracts.md).
P4/C3 remains IN_PROGRESS / PARTIAL / UNVERIFIED.

### INC-C3-22 — T3/CV-04 inferred trait-default conformance contract — 2026-09-29

Status: RESOLVED — adopted architecture implemented and G2 focused-tested. The
canonical `CallableSignatureTable` is the sole callable-result authority;
`ConformanceWitnessPlan` is a derived, replaceable projection. After canonical
result refresh, affected plans must be recomputed under their existing stable
identities and authoritatively replaced. Consumers must depend on the plan and
be invalidated/reanalyzed through existing dependency machinery before the
final snapshot is accepted. Prefer final compatibility/dispatch consumers
after the result-stabilization barrier; early consumers are allowed only when
their products and diagnostics are retractable. This is ordinary derived
product invalidation, not a second result owner or a general inference
fixpoint. A genuine result/conformance dependency cycle remains governed by
INC-C3-21's explicit-contract rule. The amended G2 matrix includes incompatible
and compatible witnesses, explicit-result parity, stable replacement,
already-analyzed-client and stale-dispatch retirement, exact incremental
invalidation, same-result reuse, cold parity, and cycle-safety controls.
`inferred_trait_result_replaces_conformance_plans_and_rechecks_clients`
passes 1/1 and covers the focused matrix. The canonical inferred-result,
symbolic substitution, return-mismatch, original CV-04, and diagnostic-repair
controls each pass 1/1. T4/CV-05–CV-07, Priority 3, and P4/C3 certification
remain separate and incomplete.
See [`INC-C3-22.md`](INC-C3-22.md).

### INC-C3-23 — T5 cycle diagnostics and `Iterator.take` generic-argument block — 2026-09-30

Status: ARCHITECT RULING ADOPTED; implementation remains partial. Unseeded
callable result SCCs must receive one deterministic diagnostic while results
remain unknown and no fixpoint is added. Explicitly seeded direct and mutual
cycles publish canonical `Int` results and their focused tests pass. The
CV-09 ruling removes the artificial blocker from generic applications with
untyped formals, preserving genuine argument blockers and existing fail-closed
publication. Commit `f10f3d49` is IMPLEMENTED / FOCUSED_TESTED: both new
generic-call semantic selectors and the canonical-Universe
`Iterator.take` publication selector passed 1/1. The latter verifies
`TakeIterator.new(self, n)` is Known/Ready and matches the inferred signature.
The invalid-`n` runtime selector is blocked before fixture execution by the
known Universe iterator-selector conflict (20 `ImplConformanceIncomplete`, 3
`type.generic.underconstrained`); runtime behavior remains unverified. CV-08
result-edge ownership is under INC-C3-24; T5/G4 remains partial and T6 is not
started. See
[`INC-C3-23.md`](INC-C3-23.md) and [`INC-C3-24.md`](INC-C3-24.md).

### INC-C3-24 — T5 result-dependency edge ownership — 2026-09-30

The architect's INC-C3-23 ruling authorizes deterministic diagnostics for
unseeded recursive result SCCs. Live products do not currently link each
normal-return value to a selected callable: `CallableAnalysis.dependencies` is
an aggregate body dependency set, while `NormalReturnFact` has no
result provenance. AST/range joins identify direct returned-call roots only;
they cannot soundly trace nested expressions or locals through branches and
flow. Using aggregate calls or recursively collected syntax calls could
diagnose recursion that does not contribute to the result. CV-08 source edits
are paused pending a ruling on whether T5 is limited to direct returned-call
cycles or expands the body product with stable result dependencies propagated
through expression/flow/binding values and included in product fingerprints.
CV-09 proceeds independently.
See [`INC-C3-24.md`](INC-C3-24.md).

**Disposition:** Option 1 is adopted. CV-08 uses only the canonical selected
`CallableId` on a direct normal-return root call (tail or explicit return),
fingerprints that edge, and classifies unresolved omitted-result SCCs after
ordinary publication without a result fixpoint. Explicit declared results are
seeds. No local/nested/branch provenance is inferred from syntax or ranges.
This does not decide incremental stale-inferred-result retraction; that
separate question was subsequently resolved by adopted INC-C3-25; see the
current status immediately below.

### INC-C3-25 — T5 incremental direct-result cycle retraction — 2026-09-30

**Disposition: adopted and implemented; T5/G4 focused-tested.** The
incremental A→B→C discriminator now agrees with cold analysis. The one-shot
barrier examines current direct-result SCCs despite retained inferred values,
preserves explicit declared-result seeds, clears only inferred-only SCC
results, invalidates only member `CallableBody` roots, and runs one withheld
member reanalysis. Members cannot re-enter the generic inferred-result refresh
loop in that pass. Their recomputed dispositions seed existing
changed-callable propagation so an external observer is revalidated. The
source-owned DB signature product can remain Unknown while the session's
canonical table carries a prior inferred result; therefore body-root
invalidation is the bounded lifecycle signal, not mutation of signature
publication or a fabricated source-signature fingerprint change.

The focused test proves A→B→C Known(Int)→Unknown→Known(Int) for SCC members
and the dependent observer, one deterministic diagnostic in B and none in C,
cold parity at B/C, and stable/reused unrelated `h() { 3 }`. The semantic
direct, mutual, explicit-seed, and non-result cycle controls; both CV-09 generic
argument controls; canonical `Iterator.take` Known/Ready publication; and
`cargo check -p phalcom-semantic` passed their exact focused selectors. The
invalid-`n` runtime behavior remains BASELINE_BLOCKED / unverified: the prior
`corpus::iterator_negative` and direct fixture attempts stop before runtime at
20 `ImplConformanceIncomplete` and 3 `type.generic.underconstrained` Universe
bootstrap diagnostics (INC-C3-23 §6/§9). No corpus aggregate/workspace suite
was run. This blocker does not invalidate the focused lifecycle acceptance.
T6 remains unopened, G5/G6 are not certified, and P4/C3 remains
IN_PROGRESS / PARTIAL / UNVERIFIED; Priority 3 is unchanged.

See [`INC-C3-25.md`](INC-C3-25.md).

Use entries:

```text
INC-C3-<n>
Plan/task:
Trigger:
Observed:
Decision:
Architecture impact:
Plan amendment:
Verification required:
Status:
```

Material changes to:

- canonical term carrier;
- ProperTypeId boundary;
- row ownership;
- generic application representation;
- exact-case identity;
- associated namespace ownership;
- runtime/static execution boundary;

require a principal-architect amendment, not local implementer improvisation.

---

## 18. Mandatory checkpoint STOP conditions

Implementation must stop and consult if:

1. TypeId cannot represent new non-row static terms without weakening ProperTypeId.
2. RecordRow must be collapsed into TypeId to implement heterogeneous applications.
3. Data/Enum convergence breaks exact-case/GADT identity.
4. associated type-data requires trait evidence ownership.
5. one associated namespace cannot cover the required categories without duplicate lookup authority.
6. metadata requires source strings for identity.
7. CompleteTypePack cannot faithfully implement TupleEntries.
8. the VM must execute type-data constructors for semantic correctness.
9. enum/data compatibility produces different declaration or case identity.
10. C3 requires arbitrary static functions/evaluation to satisfy its accepted examples.
11. trait static-domain support becomes mandatory and requires coherence architecture changes.
12. any C2 pack/row invariant must be weakened.

---

## 19. Checkpoint verification strategy

### P1 evidence

Focus:

```text
parser/AST
algebraic semantic declaration
runtime ADT/GADT preservation
type-data canonical identity
metadata foundation
runtime-negative type-data boundary
```

### P2 evidence

Focus:

```text
generic/static application
inference
substitution
associated lookup
metadata/imports
incremental
source index
```

### P2 in-progress implementation evidence

The following bounded P2 slices are implemented in the live working tree:

```text
T1
    P1 prerequisite rechecked with `cargo check -p phalcom-semantic`.
    P2 activated from the P1-complete working tree at HEAD 60a27f79...;
    no commit or push was authorized.

T2
    named static-domain kind syntax (`S: Shape`) parses and resolves through
    declaration-backed `KindId::StaticDomain` classification.

T3
    `TypeData::Applied` uses heterogeneous `StaticArgument` entries;
    `TypeStore::apply_static_form` performs checked kinding, flat canonical
    application, and type-lambda compatibility reduction; type-only callers
    delegate through `apply_type_form`; substitution preserves the envelope
    and reports application failures instead of failing open.

T4 partial
    inference now lifts non-row `StaticData` constructor terms into a
    solver-local structural form, recursively tracks their variables/support,
    unifies only matching domain/constructor identities, infers a non-row
    static-domain generic application argument, and materializes
    through `TypeStore::static_data`. The `InferVarId(KindId)` identity and
    existing row solver remain unchanged. Six focused integration tests cover
    ConstInt payload solving, domain/case mismatch refutation, rejection of a
    proper-Type candidate, nested structural solving, preservation of an
    underconstrained static-domain variable, and inference through a static-domain
    generic application. `foundations::static_domain_inference`
    (5), `foundations::generic_application` (11), and `foundations::generics_core`
    (8) pass. The original library unit-test filter remains behind BASELINE-003.
    Heterogeneous row arguments in the general inference-term application
    representation remain unimplemented; alias/type-member convergence remains part
    of G3.

T5 partial
    type aliases accept an optional declaration-backed result kind such as
    `type DefaultShape: Shape = ...`; the alias lowering path validates the
    body kind and preserves the historical Type-valued default. Source static
    constructor paths still await T7 unified `::` lookup. Integer type-member
    arguments now canonicalize to `TypeData::ConstInt` at projection formation;
    normalization and intrinsic tuple indexing consume that canonical term.
    Focused type-member tests pass. Source declarations now publish checked
    signatures under stable owner-plus-canonical-selector identities. Snapshot
    products own signatures; AssociatedSurface entries reference identities, own
    collision/admission decisions, and depend on signature fingerprints. A source
    signature edit recomputes the entry while an unchanged sibling product is reused.
    A signature-only argument validator is available for a previously selected
    declaration product and rejects wrong-domain arguments; T7 owns routing source
    applications through AssociatedSurface and preserving selected-declaration
    projection identity across equivalent lookup spellings.

T6 focused-tested
    associated `type data` parses in class bodies and unguarded inherent
    implementations of runtime data owners. Stable declaration identity is
    owner-plus-name; owner generic arguments form canonical specialized domains.
    Checked declaration products are retained separately from AssociatedSurface,
    which admits the category and handles collisions. Constructor payload kinds
    are checked against the owner generic environment. Conformance and guarded/
    partial-target impl declarations are rejected; duplicate names across impl
    fragments are rejected. No runtime invocation target is created. Structural
    updates always republish a product-data owner's base constructor surface even
    when no associated static domain is contributed.

T7 partial
    Nested associated `::` paths parse, and kind atoms now wrap ordinary
    `TypeAnnotation` forms so `Container<Int>::State` resolves through the existing
    type-form path and `AssociatedSurface`. Static case catalogs publish stable
    identities for every case before payload checking; per-case constructor
    signatures (including zero-arity constructors) are query products in the
    Alias/Header/Member/StaticCaseSignature graph. Builders consume catalog and
    checked-signature products, and one shared intern/materialization path enforces
    singleton versus constructor requirements. The graph classifies any SCC with a
    Member or StaticCaseSignature as one `TypeSignatureCycle`, while recursive
    domain payloads remain legal. TypeOnly aggregate metadata now accepts static
    domain forms without running them through the proper-runtime-type checker.
    Class-body and inherent-impl associated signatures have per-case source
    fingerprints; an old duplicate impl builder path was redirected to the same
    catalog/signature products.

    Focused evidence on 2026-09-23: parser associated-domain kind-form test (1),
    associated generic-owner kind resolution (1), singleton versus `Empty()`
    signature/alias (1), recursive static-domain payload (1), alias/case-signature
    SCC (1), associated generic-owner SCC (1), nominal catalog rename and cold parity
    (1), class-owned signature edit with sibling reuse and cold parity (1),
    inherent-impl signature edit and cold parity (1), and runtime-negative TypeOnly
    value use (1) all pass; `cargo check -p phalcom-semantic` passes. These results
    close the ratified static-case/kind-form implementation slice, not all P2 gates.
    Heterogeneous row inference (T4), metadata/import reconstruction and provider
    parity (T8), remaining static type-member convergence, and final T9 stabilization
    remain open. Declaration fingerprint discovery is module-scoped, but dependency
    graph reconstruction and SCC traversal still scan the full graph; retain this as
    a performance follow-up rather than claiming exact-worklist completion.

T8 partial
    schema v8 adds a stable declaration/case catalog, validates associated owner
    argument kinds and static constructor membership/payload kinds, validates
    heterogeneous row argument fields/tails, and checks kinded arguments and result
    kinds for global/scoped Applied nodes. Snapshot-backed export
    consumes current nominal data/enum products and associated declaration products;
    the metadata contract does not enumerate retained TypeStore history. Schemas v1-v7
    retain their existing validation floors. The affected libraries compile, but no
    v8 behavior tests were run against this diff. Cross-module catalog composition and
    import reconstruction, provider edit invalidation, and cold/incremental parity
    remain open. The source index retains the prior focused nominal-case identity
    evidence; it does not close the broader T8 contract.

### Corrective closeout activation

`LANG007.C3.P2.R1-closeout-corrective-patch-grade-implementation-plan.md` is the
active execution companion for closing the remaining P2 acceptance gaps. Its
preflight started at `9c150747262275b34e52cb7b6642d7d3ceb4ad4d` with a clean
worktree. The two commits since the plan's `2ea4f3a1` repository anchor contain
the unrelated LANG004 setter-lane delivery and this corrective plan; neither
changes P2 semantic implementation. Live inspection confirms the three T4 gaps
remain: heterogeneous generic-parameter reachability, stable row-lacks traversal,
and transitive/frame-aware row-root completion.

### Corrective closeout execution progress

T2 implementation is applied. `type_contains_any_parameter` now traverses
heterogeneous `StaticArgument`s, record-row tails, associated static-domain owner
arguments, and scoped type-lambda bodies. Stable row-lacks collection follows the
same heterogeneous argument and owner paths. No `RecordRowVarId` is emitted into
stable facts. The direct RecordRow generic syntax attempted for a non-callable
declaration is rejected by the live checker (`RecordRow generic binders are
supported only on callable declarations`); no grammar was inferred or added. Tail
reachability is therefore covered by semantic construction beside the private
helper, rather than a fabricated source form.

T3 implementation is applied at the coordinator boundary. Requested roots are
normalized transitively before completion classification: local and solver-internal
residual tails remain underconstrained, while an unresolved ancestor-owned tail is
retained as a symbolic solution. The row solver remains frame-agnostic, roots remain
call-scoped, and ordinary/row variable identities remain separate. The focused
coordinator matrix passes in a disposable validation worktree: local and transitive
underconstraint, child-to-parent symbolic retention and later resolution, both
cross-domain occurs insertion orders, heterogeneous `Applied` and `StaticData`
children, inferred row-extension payload materialization, and semantic same-label
conflict reporting. The existing nested generic-closure row integration test also
passes.

The library unit-test target in the primary checkout remains blocked by stale,
unrelated test-only initializers (`DeclarationTypeInfo.generic_binder_header` and
legacy `InferenceStaticArgument` construction). For the isolated coordinator/helper
test run, only those test-harness compatibility sites were adjusted in a disposable
worktree; active implementation sources and new tests were copied unchanged. The
temporary worktrees were removed afterward.

The call-parameter helper matrix passes (4 tests), the stable row-lacks helper matrix
passes (3 tests), and the frame/coordinator matrix passes (9 tests). The focused
foundation gates pass: static-domain inference (6), row inference (6), generic
application (11), and generic core controls (8). `cargo check -p phalcom-semantic
--lib` passes. The focused record-row integration group selects 11 tests: 8 pass,
and these three
failures reproduce unchanged at the clean `9c150747` predecessor, so they are
classified as baseline: `repeated_row_parameter_conflict_is_rejected`,
`row_conflict_message_does_not_expose_solver_variables`, and
`return_record_prefix_contributes_lacks_constraint_to_row_inference`. T2/T3 are
implemented and focused-tested with the stated unit-harness limitation. CL-01's
row-tail-only case exercises declaration-generic selection with a constructed
canonical callable signature because the corresponding non-callable source binder
is rejected by the live language. T4's corrective matrix is accepted with those
three baseline failures explicitly classified. T5/T7 acceptance is now focused
tested: `incremental::type_members` selects 23 and passes, `adts::associated`
selects 27 (22 pass, 5 fixture-gated ignored), `associated::invocation` selects
1 and passes, and `foundations::kinds` selects 18 and passes. Evidence includes
callable parameter/return type-member projection, A/B/C associated-surface
collision refresh with cold parity, and a canonical mixed Term/RecordRow/constant
associated-owner specialization. T7 is accepted.
```

### Corrective closeout continuation — T8 accepted; T9 consultation open (2026-09-24)

T1–T8 are accepted for the P2 pre-T9 gate. This is implementation acceptance with
focused evidence, not release certification. T4's three record-row integration
failures remain classified as clean-predecessor baseline failures. T8 schema/import
coverage is complete: `phalcom-type-meta` passes 14/14; schema-v8 static alias,
associated-term, heterogeneous row application, scoped-bound, and stale-TypeStore
history regressions pass; provider payload edit/rename and associated owner
specialization tests pass; `integration::imported_resolution` passes 14/14,
`integration::source_index` 22/22, `incremental::adts` 13/13 selected non-gated tests,
and `incremental::type_store_revisions` 22/22.

The trait boundary rejects user-defined static-domain generic binders with a targeted
diagnostic, withholds the invalid header and conformance contribution, and retires a
previously published valid header after an incremental invalid edit. Both new focused
tests pass. `capabilities::traits` selects 39 tests: 37 pass and two dependency
assertions fail identically at clean predecessor `9c150747` because non-generic trait
defaults acquire a Universe `Bool` declaration-shell dependency. These are baseline
failures, not trait-boundary regressions.

T9 has started its audits. Runtime isolation found no P2 runtime static evaluator or
new VM static-domain execution contract. The P2 code does not weaken ProperType
admission; general heterogeneous consumers preserve `Term`/`RecordRow`, while
projection normalization and rigid/runtime-only consumers explicitly require
Type-valued arguments or return symbolic/reject. The different-origin supertype
projection path remains type-only and conservatively does not project heterogeneous
owner arguments; no required P2 acceptance case needs row inheritance through that
path. The resource audit found `self.lambda_arena.clone()` in
`TypeStore::apply_static_form`, which conflicts with canonical P2 `INV-P2-22` and
corrective `INV-24` / `CL-35`.

`INC-C3-11` is IMPLEMENTED / FOCUSED_TESTED. The architect's ruling is implemented:
one active `TypeLambdaArena` is threaded through recursive reduction and free-capture
specialization, and public TypeStore boundaries restore it on ordinary returns. Both
whole-arena clone sites are removed. No transaction, journal, or rollback behavior was
added.

Focused evidence acquired during this P2 continuation:

```text
AST parser named static-domain binder test                         PASS (1)
semantic foundations/kinds                                         PASS (15)
AST parser static-alias result-domain test                          PASS (1)
semantic foundations/generics_core                                 PASS (8)
semantic foundations/type_annotations                               PASS (45)
semantic adts                                                      PASS (281; 28 ignored/gated)
semantic metadata integration                                     PASS (21)
type-meta schema compatibility                                     PASS (9)
semantic source type-member surface selection/domain validation    PASS (1)
semantic associated static-domain/case source-index targets         PASS (1)
semantic imported nominal static-case provider identity             PASS (1)
semantic integration/metadata                                      PASS (23)
semantic incremental/type_members                                  PASS (7)
semantic foundations/static_domain_inference                       PASS (6)
semantic incremental/adts                                          PASS (13; 3 ignored/gated)
semantic incremental type-data case edit cold parity               PASS (1)
semantic incremental associated static-domain edit/cold parity     PASS (1)
core crate check                                                   PASS
core language/algebraic_data                                      PASS (44; 19 ignored/gated)
core and semantic crate checks                                     PASS
semantic library static-data inference unit filter                 BASELINE_BLOCKED (BASELINE-003)
semantic foundations/type_members                                 PASS (7)
semantic integration/type_member_closure                           PASS (9)
semantic incremental/type_members                                  PASS (7)
AST parser associated type-data sites and nested path                PASS (2)
semantic associated static-domain publication/duplicates            PASS (focused filters)
semantic nested associated static domain/term lookup                 PASS (focused)
semantic generic-owner associated-domain specialization              PASS (focused)
core associated lowering boundary                                    PASS (cargo check only)
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic      PASS (post-amendment)
repo-wide cargo fmt --all -- --check                               FAIL (D; pre-existing formatting drift outside this slice)
semantic incremental/type_members                                  PASS (12; continuation)
semantic associated lookup                                         PASS (1; continuation)
semantic imported static-case provider identity                    PASS (1; continuation)
semantic type-data case cold parity                                PASS (1; continuation)
semantic mixed alias/member cycle                                  PASS (1; continuation)
semantic source-member alias projection                            PASS (1; continuation)
semantic source-member generic-header projection                   PASS (1; continuation)
semantic nominal_static_case at HEAD                               PASS (3; clean worktree)
type-meta schema_compat                                             PASS (9; clean worktree)
semantic integration/metadata                                       PASS (23; clean worktree)
semantic incremental/adts                                          PASS (13; 3 ignored; clean worktree)
semantic imported nominal static-case resolution                  PASS (1; clean worktree)
```

Final T9 / G-FINAL evidence acquired 2026-09-24:

```text
semantic foundations/type_annotations                               PASS (49; includes four INC-C3-11 regressions)
semantic foundations/record_row_inference                           PASS (6)
semantic foundations/static_domain_inference                        PASS (6)
semantic foundations/generic_application                            PASS (11)
semantic foundations/generics_core                                  PASS (8)
semantic foundations/substitution                                   PASS (7)
semantic foundations/kinds                                          PASS (18)
semantic adts/associated                                            PASS (22; 5 ignored/gated)
semantic incremental/type_members                                   PASS (23)
semantic integration/imported_resolution                           PASS (14)
semantic integration/source_index                                   PASS (22)
semantic incremental/adts                                           PASS (13; 3 ignored/gated)
type-meta schema_compat                                              PASS (14)
core crate check                                                    PASS
semantic integration/metadata                                       BASELINE_BLOCKED (24 pass, 1 clean-predecessor failure)
semantic integration/record_row_polymorphism                        BASELINE_BLOCKED (8 pass, 3 clean-predecessor failures)
workspace formatter                                                 BASELINE_BLOCKED (formatter drift already present in the four touched Rust files at HEAD; additional workspace drift)
```

The focused failures and formatter differences are clean-predecessor baselines and are
not attributed to the arena correction. The source audit finds no whole `TypeLambdaArena`
clone in TypeStore application, substitution, or beta-reduction paths. Runtime isolation,
ProperType, and Applied-consumer audits remain as previously recorded. P2 is
`COMPLETE / IMPLEMENTED / FOCUSED_TESTED`; at the time of this historical P2
snapshot, C3 remained partial because P3 had not yet run. The final checkpoint
state is recorded in §21 below.

The type-member publication slice is committed and pushed as
`f2a6e3d8 lang007: publish checked type-member declarations`. The repository-wide
formatter check reports differences in untouched AST/core/checker files; no formatting
changes from that check were retained.

The paragraph above is a historical continuation snapshot and is superseded by the final
T9/G-FINAL state recorded here. The library unit-test filter remains blocked by
BASELINE-003. Full-graph edge reconstruction and SCC traversal remain a performance
follow-up, not a P2 correctness blocker.

### P3 evidence

Focus:

```text
Universe/intrinsic structural domains
core algebraic data
TupleEntries/pack equivalence
LSP
corpus migration
spec consistency
cold/incremental certification
runtime-negative audit
```

Workspace/release-wide certification is not implied by `FOCUSED_TESTED`.

---

## P3 takeover and migration manifest

T1/G1 completed at starting revision:

```text
repository: aureat/phalcom-lang
branch: main
revision: 518f66d19941eb24e379e60f64aa89df66643a8a
subject: fix(semantic): thread active type lambda arena
worktree: one unrelated untracked directory, phalcom-core/tests/integration/; preserved
metadata schema: 8
P1: COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P2: COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

The P2 hard predecessor contract is verified against the completed-P2 handoff,
checkpoint evidence, and live source symbols. Stable owners include `TypeStore`
and `TypeId`; `ProperTypeId`; `KindId` / `KindData`; `StaticDomainId`;
`StaticArgument::{Term, RecordRow}`; `RecordRowId` and the separate row solver;
`TypeStore::complete_type_pack` / `project_complete`;
`TypeData::CompleteTypePack`; `AssociatedSurface`; semantic editor
`target_at`/definition/reference products; metadata schema v8; the active
`TypeLambdaArena` move-out/restore boundary; and
`UniverseBootstrapMeasurement::{discovered_units, root_reachable_units,
executed_units}`. No missing predecessor authority was found.

Exact source/tooling anchors for follow-on tasks:

| Surface | Live anchor | T1 finding |
|---|---|---|
| Tuple and pack presentation | `phalcom-core/core/universe/src/collections/tuple.ph` | `native class Tuple`; exposes Positionals/Labeled/At/Map; no Entries |
| Record presentation | `phalcom-core/core/universe/src/collections/record.ph` | `native class Record` |
| Unit presentation | `phalcom-core/core/universe/src/option/unit.ph` | `native class Unit` |
| closed sums | `phalcom-core/core/universe/src/option/option.ph`, `errors/result.ph`, `object/ordering.ph` | `Option`, `Result`, `Ordering` remain `native enum` |
| other Universe sums | `time/calendar/date.ph`, `month.ph`, `weekday.ph`, `datetime.ph` | DateOverflow, Month, Weekday, LocalTimeDisambiguation, LocalTimeResolution |
| source index | `phalcom-semantic/src/source_index/scope.rs`, `symbol.rs`, `builder.rs`, `mod.rs` | SourceDeclarationKind and EditorSymbolKind still include Enum; builder has `visit_enum` |
| intrinsic operation dispatch | `phalcom-semantic/src/types/type_member.rs` | `lookup_intrinsic_type_operation` still dispatches by owner string |
| LSP navigation | `phalcom-lsp/src/backend.rs`; tests in `module_navigation.rs`, `workspace_indexing.rs` | definition/references substrate exists; no rename/prepareRename implementation or capability |
| editor syntax | `tools/vsphalcom/syntaxes/phalcom.tmLanguage.json`; tests in `src/test/suite/lsp.e2e.test.ts` | grammar retains enum/ADT declaration wording; no rename lane identified |
| bootstrap baseline | `phalcom-core/src/vm/mod.rs`; `tests/core/modules/universe.rs` | counters and measurement test exist; retain before/after counts at T4 certification |

Parser-backed census: a temporary external helper used `phalcom_ast::parse`
and `AlgebraicDef::legacy_enum_keyword_range` over repository `.ph` files,
excluding hidden directories and build output. It found these current declarations:

| Category | Parser-confirmed declarations / source | Treatment |
|---|---|---|
| A — canonical Universe | 8 declarations in `option/option.ph` (Option), `errors/result.ph` (Result), `object/ordering.ph` (Ordering), `time/calendar/date.ph` (DateOverflow), `month.ph` (Month), `weekday.ph` (Weekday), and `datetime.ph` (LocalTimeDisambiguation, LocalTimeResolution) | migrate in T5 |
| B — first-party examples and parser project sources | `examples/phalcom-parser/src/lex/token.ph` (Keyword, TokenKind); `syntax/ast.ph` (14 parsed sums); `syntax/diagnostic.ph` (DiagnosticKind); `examples/either.ph` has declaration-shaped source but current parser reports errors before yielding it | migrate active owned source; account for parser limitations explicitly |
| C — active positive test sources | `tests/core/typing_integration/sources/either.ph` (Either), `expression.ph` (Expression) | migrate with identity-preserving source changes |
| D — editor golden | `examples/ide-golden/phalcom-enum-highlighting-showcase.ph`; parser recovered Direction and reports errors on competing legacy forms; it contains seven showcase declaration spellings (four Direction variants plus Option, Result, Expr) | migrate/replace as editor migration fixture in T5/T6 |
| E — benchmarks/sample packages | no parser-confirmed declarations found | none at T1 |
| F — active authoritative specs/docs | prose and examples remain to be inventoried during T8; excluded from `.ph` AST census | update in T8 |
| G — embedded/generated/self-hosted source | `examples/phalcom_parser.ph` identifies itself as a self-hosted parser design draft; `examples/phalcom-parser/README.md` labels its package a design/implementation draft; no workspace, package, or test entrypoint references either draft | T7 disposition: excluded from the active source census as non-executable design drafts; retained as historical material, not a current lexical authority |
| H — intentional negative fixtures | `tests/fixtures/language/errors/annotation_variant_global_reference.ph` (Ordering), `annotation_variant_visitor_exhaustive.ph` (Shape) | retain as rejection/diagnostic coverage and update expectations at T7 |

The census also found parser limitations: `examples/either.ph` reports
`EnumBehaviorUnsupported`; `examples/phalcom-parser/src/lex/token.ph` reports
recovery errors after two parsed declarations; the IDE showcase reports errors
after its first recovered declaration; `examples/phalcom_parser.ph` and the
self-hosted lexer do not parse cleanly with the current parser. These are
path-classified source migration candidates, not evidence that the files are
historical or negative. T5/T7 must re-run the census and prove no positive
source route remains before parser retirement. The P3 plan’s §7.12 A–F census
categories are extended here with G (embedded/generated/self-hosted) and H
(intentional negative fixtures), resolving the supplied audit’s A–H wording.

Inherited P2 baseline ledger carried forward without new reproduction:

- one metadata alpha-equivalence failure;
- three record-row-polymorphism failures;
- two trait-suite failures;
- formatter drift already present at completed-P2 HEAD;
- full declaration signature graph/SCC traversal retained as a performance follow-up.

G1 result: PASS. The completed-P2 revision, required lifecycle records,
predecessor architecture, metadata schema, source identity substrate, and
bootstrap measurement hook were present.

T2/G2 result: PASS. Canonical Universe source now authors bodiless
`intrinsic type data` anchors for `TypePack`, `LabeledTypePack`, `TupleEntries`,
and `RecordRow` in the existing tuple/record modules. `DeclarationTypeTable`
holds declaration-identity bindings to `TYPE_PACK`, `LABELED_TYPE_PACK`,
`COMPLETE_TYPE_PACK`, and `RECORD_ROW`; the facades have no TypeId form and do
not allocate nominal `StaticDomainId`s. Session declaration/shell, nominal
hierarchy, static-case catalog, and fingerprint passes exclude registered
facades. Source indexing targets each authored declaration. The parser accepts
the no-shape form only for intrinsic data declarations. Core bootstrap-root
selection and runtime class census exclude TypeOnly facades.

T2/G2 focused evidence: AST `data_syntax` (27 tests); semantic intrinsic
facade authority, TupleEntries kind resolution, and source declaration target
tests; core TypeOnly-root unit test; and the core
`boot_01_bootstrap_measurement_separates_catalog_closure_and_execution` test.
That VM gate first exposed a session predeclaration path that rebuilt a
registered facade as a nominal type; after excluding intrinsic facades from
the nominal passes, the gate passed. The P2 starting-revision control passed
the same gate, confirming the first failure was introduced by P3 and was
corrected within T2.

T3/G3 result: PASS. `Tuple::Entries` is an intrinsic member whose public result
kind is `COMPLETE_TYPE_PACK` and whose normalization delegates to
`TypeStore::project_complete`; metadata still uses `CompleteTypePack`. Intrinsic
member selection consumes canonical `IntrinsicTypeOwner` descriptors populated
by declaration identity. `TypeMemberDeclarationId` is now a source-index target
for member definitions. Concrete and symbolic Entries projections pass, as do
the existing At/Map and symbolic projection cases. Focused evidence:
semantic `type_member_closure` (10 tests), the source-index type-member target
test, semantic crate check, and the canonical Universe bootstrap measurement
test with the new Tuple member compiled.

---

## 20. Active plan and next action

Current:

```text
active plan:
    LANG007.C3.P4.R1

latest completed plan:
    LANG007.C3.P3.R1

next task:
    The active plan is LANG007.C3.P4.R1 under adopted INC-C3-21. T2/G1 is
    IMPLEMENTED / FOCUSED_TESTED: canonical signature publication and
    consumer projection pass the paired trait-default/ordinary parity test
    and the existing ordinary publication control, each 1/1. T3/G2 is now
    IMPLEMENTED / FOCUSED_TESTED under INC-C3-22. T4/CV-05–CV-07 is now
    IMPLEMENTED / FOCUSED_TESTED by three exact semantic selectors, each 1/1:
    `semantic::incremental::callable_dependencies::inferred_return_refresh_preserves_stable_products_and_counts_final_disposition_once`,
    `semantic::incremental::product_stability::inferred_cross_module_result_updates_dependents_and_matches_cold_analysis`,
    and `semantic::integration::metadata::cross_module_inferred_callable_result_round_trips_as_compiler_inferred_metadata`.
    T5/CV-08 is IMPLEMENTED / FOCUSED_TESTED under the adopted INC-C3-24/25
    rulings. The incremental A→B→C cold-parity test passes 1/1: SCC members
    and an external observer transition Known(Int) → Unknown → Known(Int),
    canonical signature-product fingerprints change on both edges of the
    transition, exactly one deterministic diagnostic appears in B and retires
    in C, and unrelated `h() { 3 }` stays stable/reusable. The bounded
    one-shot barrier invalidates only SCC-member body-query roots, reanalyzes
    those members once with results withheld, and uses existing changed-callable
    propagation for dependent consumers; no result fixpoint or new authority.
    The INC-C3-23 ruling is adopted. CV-09
    is IMPLEMENTED / FOCUSED_TESTED at `f10f3d49`: the untyped-formal Known/Ready
    and genuine-blocker controls, plus canonical Iterator.take publication,
    passed 1/1 each. Invalid-`n` runtime behavior remains BASELINE_BLOCKED /
    unverified because the prior gate stops before fixture execution with 20
    `ImplConformanceIncomplete` and 3 `type.generic.underconstrained` Universe
    bootstrap diagnostics; it does not block T5/G4 lifecycle acceptance. T5/G4
    is FOCUSED_TESTED and T6 is not started. This is
    separate from T13 and does not
    change T13's §15 stop, certify G5/G6,
    or authorize workspace/release gates.
    G5 collection aggregates remain unpassed/uncertified, G6 is incomplete, the
    natural chain is blocked/ignored, and P4/C3 remains IN_PROGRESS / PARTIAL /
    UNVERIFIED. Priority 3 is unchanged.
    T7.9–T7.11 and T8.1–T8.10 are IMPLEMENTED / FOCUSED_TESTED. T8.5 preserves
    Record/open-row and labeled-Tuple Record spreads, rejects Map-to-Record for
    both bindings and literals with the same semantic diagnostic, retains
    Map-to-Map and Record/Tuple-to-Map behavior, and leaves shared runtime lane
    machinery unchanged. T8.7's parser, semantic row-subtraction, and runtime
    exact-unmatched-fields selectors pass; generated raw Record observer sends
    use the established single-dispatch compiler-internal authority. T8.8's
    explicit-Map-vs-bare-Record pattern parser discriminator passes. T8.9's
    `braced_module_metadata_literals_remain_records` parser selector passes
    1/1. T8.10's old braced-record Set message, `| R` to `**R` message, and
    canonical Map-expected/Record-actual binding guidance selectors pass 1/1
    each; the existing Set literal control remains green. T9.1–T9.6 are
    IMPLEMENTED / FOCUSED_TESTED: value-level `TupleView`/`RecordView` adapters
    represent Unit/EmptyRecord as canonical zero-length views without heap
    allocation; Tuple and Record observers consume those views, while the
    existing ObjRef view entry points remain available. Empty spec/dynamic
    construction paths, Value equality/hash/same-bits, source `==`/`===`,
    size/positionals/labeled/toString/iteration, and `.class` checks are covered.
    Focused product and Value selectors plus the language E2E selector each
    passed 1/1; `Value::class` is the established immediate runtime type
    observation authority. T9.1–T9.6 were committed and pushed as `3507cdff`.
    T10 adopts the ruling to remove `UniverseKey::Unit` and its
    `UNIVERSE_BINDINGS` row entirely. The runtime catalog stays homogeneous:
    every retained key resolves to a real ClassId. Unit remains visible as the
    exact source alias `option/unit::Unit` through the existing source catalog
    and `PreludeTypeMap`; no alias registry, synthetic class, or Unit-to-Tuple
    runtime resolver mapping is added. Tuple/Record retain their direct runtime
    owners. Native primitive return contracts formerly written as Unit now use
    the structural empty tuple `()` TypeExprSpec; a semantic resolver test
    proves this resolves to `store.unit()`, while runtime primitives continue
    returning `Value::unit()`. T10.2/3 source ownership was already correct:
    Unit source contains only the alias and Tuple owns its generic behavior;
    the existing runtime class census excludes Unit. Focused tests pass for
    Unit's absence as an intrinsic declaration, exact source-backed prelude and
    editor visibility, Tuple/Record owner registration, native structural
    return resolution and metadata, and builtin runtime class lookup. The
    native-surface generator check and `cargo check -p phalcom-core` pass. The
    focused object-model floor-census test
    (`floor_census_matches_installed_bindings`) reports only the unchanged
    `System class#monotonicNanoseconds` entry as listed but not installed, with
    no extra installed bindings. This is `D — unrelated baseline`: both
    `primitive/time.rs` and the census row are unchanged, and the Unit-specific
    catalog/class lookup test passes. T10 is IMPLEMENTED / FOCUSED_TESTED;
    after review continue T11–T12, Priority 3 causal/residual disposition, and
    T13. The
    INC-C3-19 producer correction and focused lexical-body acceptance evidence
    are recorded below. Keep TypeStore::union, loop flow, and lexical-body
    semantics unchanged. T6.4 is now IMPLEMENTED / FOCUSED_TESTED after T9/T10:
    bare Unit type references use the exact prelude source alias rather than a
    name fallback, while structural `()` remains canonical Unit. T12.6 is
    IMPLEMENTED / FOCUSED_TESTED: all five final cold/incremental differential
    selectors pass, including Unit -> EmptyRecord consumer recomputation via
    the ruling-approved test-only canonical-source override. Canonical source
    and module identities remain unchanged; no production export, linker, or
    dependency-closure change was made. Changed aliases seed existing
    declaration-shell dependency roots, with no P4-specific invalidation
    registry. The 26-test `phalcom-modules --test workspace_session` suite also
    passes. T12.7 (LSP/editor) is IMPLEMENTED / FOCUSED_TESTED: commits
    `a66b8906`, `a9e584b7`, and `8727d5df` cover source-index products,
    canonical navigation/hover/references/rename, kind/value completion
    separation, and Record/Map/Set token classification. Focused selectors
    (each selected 1 / passed 1): `phalcom-semantic --test semantic`
    `intrinsic_type_member_source_site_targets_canonical_owner_and_selector`,
    `structural_projection_occurrences_navigate_to_owner_qualified_source_members`,
    `product_expansion_parameters_and_record_remainders_have_source_targets`;
    `phalcom-lsp --test module_navigation`
    `goto_definition_on_structural_type_members_uses_canonical_owner_source`,
    `goto_definition_tracks_product_binders_and_record_remainders`;
    `phalcom-lsp --test semantic_completion`
    `generic_kind_completion_uses_source_kind_names_not_runtime_values`,
    `collection_literal_value_positions_use_runtime_completion_namespace`;
    `phalcom-lsp --test semantic_tokens_current_syntax`
    `record_map_and_set_literals_follow_their_distinct_source_syntax`.
    T12.8 normative spec convergence is IMPLEMENTED / documentation-only;
    no Cargo tests were run. T13 is next. P4 remains IN_PROGRESS
    / PARTIAL / UNVERIFIED; broad certification is open.
    P4 remains IN_PROGRESS / PARTIAL / UNVERIFIED; broad certification is open.

next action:
    R2 Steps 5–11 are implemented and focused-tested; the T4/T5 amendment is
    accepted. T5 is IMPLEMENTED / FOCUSED_TESTED. Step 8 commit `1e82f1c3`
    preserves source-shaped constructor references and adds labeled invocation
    only to generated all-labeled constructor thunks through the existing
    ParameterShape/closure activation path; ordinary source closures remain
    positional-only. Step 9 is IMPLEMENTED / FOCUSED_TESTED pending review:
    `label_placeholder_sugar_normalizes_across_shared_selector_surfaces`
    passes (1/1), proving legacy bare labels and `label: _` normalize to the
    same selectors/patterns for bound references, associated `Construct`
    references, exact `#foo(...)`, and rest-pattern hash/bound references.
    Colon-placeholder ranges include the full `label: _` / `tail: _` text
    where retained by the AST. Canonical encoding remains `foo(label)` and
    `foo(_, ..., tail)`; no selector encoding or parser implementation change
    was needed beyond the shared Step 8 parser path. Step 8's parser, semantic,
    and core constructor-reference selectors were rerun and passed 1/1 each.
    Step 10 collision coverage is IMPLEMENTED / FOCUSED_TESTED in
    `data_constructor_call_collision_preserves_explicit_and_runtime_call_paths`
    (1/1): direct inferred/specialized and associated construction, plus both
    primary-constructor reference forms, yield the primary value with id 42;
    explicit `Tagged.call`/`Tagged<Int>.call`, both explicit behavior references,
    and invocation of the runtime class-object value `factory = Tagged<Int>`
    yield 142 through the user-authored ordinary class-side `call`. A separate
    constructor-reference variable continues to produce id 42.

    Step 11 T4/T5 focused matrix was run serially with the pinned
    `nightly-2026-07-10` toolchain (`RUSTFLAGS='' RUSTC_WRAPPER=''`):
    semantic selectors below use
    `cargo test -p phalcom-semantic --test semantic <selector>`, core source
    selectors use `cargo test -p phalcom-core --test core <selector>`, core
    unit selectors use `cargo test -p phalcom-core --lib <selector>`, and the
    AST selector uses `cargo test -p phalcom-ast --test integration <selector>`.

    - Semantic `phantom_data_constructor_`: 4 selected / 4 passed; exact
      `phantom_data_constructor_uses_compatible_expected_result_as_assumed_evidence`:
      1/1; `data_constructor_source_and_expected_result_conflict_is_not_overridden`:
      1/1.
    - Semantic `record_row_constructor_`: 7/7; compatible source-established
      row under expected context: 1/1; `complete_pack_data_constructor_`:
      5/5; `data_record_materialization_`: 2/2; nominal MyTuple and MyRecord
      identity selectors: 1/1 each; expected-result row selection, return-
      irrelevant dynamic row, and exact empty RecordRow proof: 1/1 each.
    - Core runtime row substitution and open-tail rejection unit selectors:
      1/1 each; equal-layout distinct exact descriptors: 1/1. Core `p4_`
      selected 10: 9 passed, 1 failed. The relevant T4/T5 cases all passed:
      registry/class identity, fixed+expanded Envelope, MyTuple, MyRecord,
      generic-callable MyRecord, and empty-layout PhantomRow. The remaining
      test was the separately deferred
      `language::generic_conformance::p4_specialized_rest_dispatch_preserves_conformance_environment`,
      failing at `item.print` with `no active conformance target`.
    - Core constructor-reference alias and fixed-only record controls: 1/1
      each; nullary phantom runtime specialization: 1/1; AST
      `label_placeholder_sugar_normalizes_across_shared_selector_surfaces`:
      1/1.

    The rest-dispatch result is not a certified clean-predecessor baseline or
    evidence of runtime environment erasure. The prior audit record
    [`03-nested-derived-conformance-runtime-layout.md`](LANG007.C3.P4-audit-remediation/03-nested-derived-conformance-runtime-layout.md)
    reports this probe's call as `Unknown(DynamicMessageSend)` with no callable
    or call specialization, so it never established an environment recipe.
    `phalcom-core/tests/core/language/generic_conformance.rs` is unchanged
    from `origin/main` (last changed at `ae5409b9`). This remains a separate
    deferred semantic-dispatch failure; no rest adapter change was made.
    No broad T4/T5 suite, workspace gate, or release certification was run.
    T4 remains IMPLEMENTED / FOCUSED_TESTED and T5 is IMPLEMENTED /
    FOCUSED_TESTED under the R2 amendment. Original P4 T6.1–T6.3 have focused
    semantic and metadata evidence, pending supervisor review. T6.5's explicit
    replace/refresh split and stale-projection regressions were already
    implemented in `5d18729d` and were reverified in this continuation; the
    production callsite census uses explicit replace/refresh operations, while
    `insert` remains a compatibility alias to authoritative replacement.
    Pinned selectors `runtime_owner_registration_preserves_intrinsic_semantic_application`
    and `static_domain_projection_survives_shell_refresh_and_authoritative_replacement_retires_it`
    each selected/passed 1, covering runtime/semantic owner and static-domain
    projection retirement on authoritative replacement.
    T6.4 is IMPLEMENTED / FOCUSED_TESTED after T9/T10: the bare-name Unit
    fallback was removed from both type-formation paths, and unqualified Unit
    resolves through the exact `option/unit::Unit` prelude declaration and
    alias form. Inline and standalone Universe selection include
    `option/unit`, fixing canonical bootstrap and standalone fixture/traceback
    compilation while keeping Unit absent from intrinsic application,
    runtime-owner, and runtime-class mappings. Explicit `()` and zero-pack
    Tuple still resolve to `store.unit()`. The canonical specialized iterator
    selector and runtime-shaped Tuple slice/zip boundary selector each passed
    1/1 under the pinned toolchain. T11.1 is satisfied and T11.2 is
    IMPLEMENTED / FOCUSED_TESTED. T11.3 is IMPLEMENTED / FOCUSED_TESTED with
    no source migration required: the first-party `.ph` census found only a
    Symbol-rendering comment, and the Rust-source census found valid Set
    syntax/parser controls plus the intentional old-Record migration
    diagnostic, but no executable legacy Record literal. Its focused bare-
    brace Record, Set literal, migration-diagnostic, Map-spread rejection, and
    labeled-spread controls each passed 1/1. T11.4 migrates first-party Map
    literal heads explicitly without changing Record syntax. The semantic
    `map_literal_specialization_uses_entry_and_context_evidence_without_defaulting_empty_maps`
    and core `language::maps::map_literal_expansion_preserves_values_and_rejects_duplicate_keys`
    selectors passed 1/1; `corpus::indexing` and the labeled-spread traceback
    selector passed 1/1 after standalone source selection gained `option/unit`.
    Direct execution of migrated symbol-key, evaluation-order, and key-hash
    Map fixtures produced their expected outputs; static duplicate-key and
    non-Symbol spread fixtures retained their targeted diagnostics. The
    `corpus::collections` group fails
    before reaching migrated Map fixtures at `boundedness_take_bounds_unbounded.ph`
    because `TakeIterator` lacks `toList`. `corpus::system` reaches the
    collection-rendering fixture; its initial `Set.new()` underconstraint was
    resolved by T11.5 below. This collections aggregate failure is encountered,
    not a certified baseline or Map failure. T11.4 is IMPLEMENTED /
    FOCUSED_TESTED. T11.5 is IMPLEMENTED / FOCUSED_TESTED: the parser's
    empty/populated Set literal selector and `corpus::system` passed 1/1 after
    converting its simple `1, 2` population to `#{1, 2}`. Other simple static
    population sites in list/nested/iteration fixtures were converted too, but
    their direct checks stop earlier at unchanged `Map.new()` generic
    underconstraint, before reaching the Set expressions; these are recorded
    as encountered and unclassified, not as Set failures or certified
    baselines. Empty, mutation/dedup, custom-hash, membership, rejection,
    concurrency, and dynamic accumulation cases retain `Set.new()`. T11.6 is
    IMPLEMENTED / FOCUSED_TESTED: three Map-specific negative spread fixtures
    now use explicit `Map` heads; `**1` keeps its semantic diagnostic, and
    `*source`/`***source` have the current parser's `Expected one of ";",
    newline` diagnostic. The aggregate negative-fixture selector selected 1
    but stopped at an earlier unchanged
    `map_key_eq_mutates_same_map_during_overwrite_rejected.ph` underconstraint,
    before these cases. Direct CLI checks confirmed the diagnostics for the
    three edited fixtures; the aggregate failure is encountered, not
    baseline-certified. T12.1 and T12.2 are IMPLEMENTED / FOCUSED_TESTED;
    T12.3–T12.6 are IMPLEMENTED / FOCUSED_TESTED. T12.4's
    RuntimePublic `Record::Fields` source-identity export/validation selector
    and schema-v8 omitted-source compatibility selector each passed 1/1.
    T12.5's Unit/EmptyRecord alias metadata round-trip selector passed 1/1.
    T12.6's final `incremental::t12_6` filter passed 5/5 after formatting, and
    `phalcom-modules --test workspace_session` passed 26/26. The ruling-approved
    test-only source override preserves production `EmptyRecord` visibility
    while exercising ordinary import, dependency, fingerprint, and
    cold/incremental analysis. T12.7 LSP/editor acceptance is IMPLEMENTED /
    FOCUSED_TESTED as recorded above. T12.8 normative spec convergence is
    complete; T13 may proceed.
    INC-C3-18's source rewrite is
    superseded. The adopted INC-C3-19 producer correction and its focused
    acceptance evidence are recorded below. The exact predecessor exposure
    delta is not needed to establish the rule. No union, loop-flow,
    `whileTrue`, or lexical-body change was made. T6.7 is IMPLEMENTED /
    FOCUSED_TESTED: `compile_intrinsic_runtime_data` selects the preinstalled
    Tuple or Record class from `facade.owner`; source identity locates/binds
    the facade but does not derive the runtime owner. The semantic
    `runtime_structural_declarations_bind_canonical_platform_owners` test and
    core Tuple reuse, Record reuse, and absent-facade fail-closed selectors
    each passed 1/1. T6.8 is IMPLEMENTED / FOCUSED_TESTED with no production
    changes: canonical Tuple/Record owners, owner-module rejection, covering
    and specialized domains, equivalent target identity, RecordFields row
    binding, and ordinary member-collision policy are covered. The seven
    focused semantic selectors each passed 1/1: Tuple structural owner and
    equivalent spellings; Record structural owner/equivalent spellings plus
    row identity; foreign Tuple/Record rejection; RecordRow and
    CompleteTypePack covering materialization; equivalent structural targets;
    and direct/spread Tuple/Record collision handling. T7.1 is IMPLEMENTED /
    FOCUSED_TESTED with no source change. The semantic
    `intrinsic_structural_source_facades_bind_existing_authorities_without_nominal_forms`
    test verifies the four source anchors map to existing kind/owner authority,
    with no nominal form or nominal static domain. Semantic
    `complete_type_pack_kind_resolution_uses_its_canonical_facade_declaration`
    verifies binder resolution, and core
    `boot_01_bootstrap_measurement_separates_catalog_closure_and_execution`
    confirms these anchors do not create runtime classes. All three selectors
    passed 1/1. T7.2 is IMPLEMENTED / FOCUSED_TESTED with no edits. The
    Universe source declares the pack-domain operations listed in the plan;
    the evaluator catalog is mapped by owner/selector, while projection
    resolution still requires the published associated source product and a
    matching intrinsic operation. No new concat API is required by the
    effective spec. Focused selectors
    `intrinsic_type_member_source_products_match_the_evaluator_catalog`,
    `canonical_tuple_intrinsic_impl_member_publishes_through_associated_surface`,
    and `intrinsic_type_member_source_site_targets_canonical_owner_and_selector`
    each passed 1/1. T7.3 is IMPLEMENTED / FOCUSED_TESTED. The projection
    normalizer now routes CompleteTypePack Positionals/Labeled through the
    canonical lane results and shares positional/labeled map helpers with
    TypePackMap/LabeledTypePackMap; concrete outputs normalize and symbolic
    outputs retain the exact source-backed residual. Selectors
    `complete_type_pack_projections_and_map_preserve_both_lanes`,
    `test_tuple_and_record_map_projection`, and
    `source_visible_complete_pack_lane_projections_keep_canonical_member_declarations`
    each passed 1/1. T7.4 is IMPLEMENTED / FOCUSED_TESTED with no production
    change: concrete `Tuple::Element` unions positional and labeled payloads,
    empty Tuple yields `Never`, and an unenumerable `TupleFromPacks` retains
    the source-backed projection. Selectors
    `tuple_element_unions_payloads_and_preserves_unsolved_pack_projection`
    and `tuple_entries_is_complete_pack_projection_for_concrete_and_symbolic_tuples`
    each passed 1/1. T7.5 is IMPLEMENTED / FOCUSED_TESTED with no production
    change: `Record::Value` unions all payloads in a closed row, returns
    `Never` for an empty Record, and keeps open-row results symbolic rather
    than omitting the unknown tail. Selector
    `record_value_unions_closed_fields_and_preserves_open_row_projection`
    passed 1/1. T7.6's actual-Universe source/evaluator catalog check is now
    implemented and focused-tested (new selector 1/1; existing catalog
    selector 1/1); this is test-only and source-visible evaluators are the full
    current catalog. T7.7 is IMPLEMENTED / FOCUSED_TESTED without edits:
    Tuple behavior is under `impl<***Es: CompleteTypePack> Tuple<***Es>` and
    Record behavior under `impl<**Fs: RecordFields> Record<**Fs>`; member
    signatures consume their specialized Element/Value projections. Six
    focused Tuple/Record construction, generic-callable, specialization-slot,
    iterator, and get selectors passed 1/1. T6.6's supplemental family tests
    are focused-tested. T7.8 is IMPLEMENTED / FOCUSED_TESTED: Tuple raw element
    access returns `Tuple<***Es>::Element`, Record raw value access returns
    `Record<**Fs>::Value`; public Option behavior is preserved by checked
    wrappers and invalid direct internal reads fail closed. The exact
    specialized type/behavior, sequence, and Record-get selectors each passed
    1/1. T7.9 is IMPLEMENTED / FOCUSED_TESTED: the new
    `tuple_lane_observers_preserve_exact_structural_types_and_values`
    regression verifies exact Tuple lane observer result types and values;
    the new selector and `dynamic_tuple_preserves_lanes_and_normalizes_empty_expansion`
    each passed 1/1. No production change was needed because source signatures
    and VM primitives already preserve the structural Tuple lanes. T7.10 is
    IMPLEMENTED / FOCUSED_TESTED: the iterator regression now requires exact
    `Tuple<***(Int, String)>::Element` typing for both `next` and the `for`
    protocol item; `tuple_iterator_preserves_specialized_element_type_through_both_protocols`
    passed 1/1 on the pinned toolchain. T7.11 is IMPLEMENTED / FOCUSED_TESTED:
    runtime-shaped construction, slicing, and TupleZip items stay Dynamic,
    while fixed-shape iteration remains typed. The audit found TupleZip calling public optional
    `at(_)` as if it returned an element; it now uses bounds-checked `tuple[i]`
    under its `i < size` loop. The new
    `runtime_shaped_tuple_slice_and_zip_stay_behind_dynamic_boundary` selector
    passed 1/1 on supervisor rerun. T8.1 is IMPLEMENTED / FOCUSED_TESTED: the
    existing parser directly maps bare braces to Record literal/type/pattern
    forms while preserving explicit Map and nominal construction; the new
    `bare_braces_are_records_across_expression_type_and_pattern_positions`
    AST selector passed 1/1 on supervisor rerun. T8.2 is IMPLEMENTED /
    FOCUSED_TESTED: `#{` already tokenized/routed to Set; corrected a stale
    parser comment and added lexer/parser controls for empty/populated Set,
    bare Record distinction, and non-enabled Set spread. The focused selector
    passed 1/1 on supervisor rerun, with compiler spread rejection unchanged.
    T8.3 is IMPLEMENTED / FOCUSED_TESTED: `MapLiteralExpr` preserves its
    explicit source head; semantic checking resolves canonical Universe Map
    identity and rejects local value/declaration shadows. The AST source-head
    and semantic canonical-head selectors passed 1/1 each on supervisor rerun.
    T8.4 is IMPLEMENTED / FOCUSED_TESTED: a semantic discriminator covers
    explicit specialization, entry/context inference, Map expansion, and
    empty-map underconstraint without Dynamic defaults; a core runtime test
    preserves successful expansion and duplicate-key rejection. Both focused
    selectors passed 1/1 on supervisor rerun. No production change was needed.
    Broader `collections` selectors failed before Map cases in unrelated
    iterator/negative-fixture paths; they are not T8.4 acceptance evidence and
    are not reclassified here. T8.5 resumes under the adopted ruling below.
    INC-C3-17 records the scoped RecordRow carrier and schema-v9
    amendment. Overall checkpoint P4 remains
    IN_PROGRESS / PARTIAL / UNVERIFIED.

residual evidence:
    Priority 3's 40-way categorical classification is already recorded in
    `LANG007.C3.P4-audit-remediation/04-semantic-regression-classification.md`
    (34 current regressions, 4 true clean-predecessor baselines, 2 intentional
    P4 migrations). One former row was reclassified after its corrected
    explicit-pack fixture passed; the prior full pinned aggregate predates that
    correction and the full incremental filter is pending. Remaining Priority 3 work is causal isolation, remediation
    or migration decisions, and final residual disposition; classification
    alone does not complete P4 T13, whose T1–T12 preconditions remain unmet. The
    last full pinned semantic run was on clean main `bb3d6cd2` (1711 selected,
    1632 passed, 37 failed, 42 ignored, exit 101), before the current T4
    commits. No current broad semantic rerun has been performed. Priority 2's
    nine named runtime selectors remain individually focused-tested; no broad
    core, workspace, or release gate has passed. The plan's focused
    `record_row` filter selected 56 tests: 55 passed and
    `map_expansion_does_not_fabricate_record_row` failed its expected-Unknown
    assertion. This is the already-classified Priority 3 current regression
    (older PASS evidence and failure at c386 are recorded in the per-selector
    ledger), not a new T4 failure or a clean-predecessor baseline; it remains
    unresolved outside this T4 slice. P4 remains
    `IN_PROGRESS / PARTIAL / UNVERIFIED`.

2026-09-28 formatting evidence: a targeted `rustfmt --check` covered all eight
modified Rust source/test files. It reported formatting suggestions only in
pre-existing unchanged sections of `phalcom-semantic/tests/semantic/impls/targets.rs`;
the newly modified lines were clean. A workspace-wide `cargo fmt --all -- --check`
was attempted and reported formatting diffs across numerous unchanged files
in other crates; no broad formatting rewrite was applied. This is not a
workspace or release certification result.
```

P3's final T9/T10 certification section below records the previously completed
P3 state. P3.R1 is the completed corrective closure for intrinsic
semantic-application/runtime-owner separation. P4 is the active structural-data
completion plan.

## 21. P3 certification and closure checkpoint — 2026-09-25

P3 starting revision: `518f66d1` (completed P2). T9 continuation comparison
revision: `9b4ca55d` (P3 T4/T5 delivery). Final implementation revision:
`db6c60ab`; the remaining closeout commit updates lifecycle records only.

Completed gates and evidence:

| Gate / evidence | Outcome |
|---|---|
| T1/G1 through T7/G7 | PASS; see implementation/amendment entries above and focused source/AST tests |
| T8/G8 authoritative spec ownership and syntax contradiction search | PASS; closed-sum `data`, TypeOnly/intrinsic facades, TupleEntries, associated `type data`, and Universe ownership documented; no current spec calls `enum` canonical |
| `cargo fmt --all -- --check` | BASELINE_BLOCKED; reports pre-existing workspace formatting drift, already documented at P2 handoff. No repository-wide formatting rewrite retained |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets` | PASS |
| `cargo test -p phalcom-type-meta` | PASS, 14 schema tests |
| AST crate | Current tree: 53 unit + 261 integration pass; 6 stale `associated_type_data: []` snapshots and 1 legacy selector-diagnostic assertion fail; 1 ignored. Clean predecessor `9b4ca55d` reproduces the same 7 failures (53 unit + 270 integration pass; 1 ignored), confirming they predate this continuation |
| Semantic ADT module | 277 pass, 25 ignored; the inherited generic associated static-domain payload failure was fixed by accepting proper `Type` payload kinds in the checked catalog-signature path, consistent with the existing associated builder and T0869/T0870 |
| Semantic impl query module | 54 pass, 1 data-component witness-plan assertion failure; the exact test also fails at clean predecessor `9b4ca55d`, outside the P3 static-domain payload change |
| Core algebraic-data suite | 42 pass, 2 clean-predecessor runtime constructor-reference failures, 19 ignored; both exact failures reproduce at `9b4ca55d`. The nested fixture's second `enum` was migrated to `data` and now passes |
| Source-index static-domain test incl. inherent-impl associated case | PASS |
| LSP workspace indexing/rename/navigation test | PASS |
| Metadata round-trip: static alias and associated term identity | PASS; `snapshot_metadata_round_trip_preserves_static_alias_and_associated_term_identity` |
| Associated static-domain metadata owner environment | PASS; `metadata_exports_associated_static_domain_identity_and_owner_environment` |
| Imported static-domain provider edit invalidation and cold parity | PASS; `imported_static_domain_provider_edits_recompute_exact_consumer_and_match_cold` |
| Imported associated-domain provider owner specialization | PASS; `imported_associated_static_domain_retains_provider_owner_specialization` |
| Type-data case edit incremental/cold parity | PASS; `type_data_case_edit_matches_cold_static_domain_analysis` |
| Associated static-case signature owner-kind edit and cold parity | PASS; `associated_static_case_payload_tracks_generic_owner_kind_edit_with_cold_parity`; associated type-member incremental module is 24/24 |
| TypeOnly case runtime-value rejection | PASS; `type_data_case_cannot_be_used_as_a_runtime_value` |
| ProperType/static-domain boundary | PASS; `static_domains_and_const_ints_are_distinct_from_proper_types` |
| Generic static-domain binder | PASS; `source_generic_binder_resolves_named_static_domain_kind` |
| Associated generic-header signature dependency | PASS; `associated_generic_static_case_signature_dependencies_join_the_cycle_graph` |
| Unit facade/runtime-class identity | PASS; `intrinsic_unit_facade_reuses_the_preinstalled_runtime_class` |
| Universe bootstrap/resource counters | PASS; starting revision `518f66d1` and current tree both report 106 discovered, 37 root-reachable, 101 executed; no measured increase |
| VS Code LSP e2e | PASS; `PATH=target/debug:$PATH npm run test:lsp:e2e`, 11 passing; initial run without the binary directory on PATH logged `phalcom-lsp ENOENT`, resolved by supplying the built binary path |
| Full semantic test binary | INCOMPLETE; stopped after repository high-fanout stress tests exceeded the useful gate window; multiple unrelated baseline failures appeared before stop |

The generic associated static-domain payload failure was reproduced on clean
predecessor `9b4ca55d` and resolved in this continuation: the catalog-signature
path had rejected proper `Type` payloads even though the existing associated
builder accepted them and T0869/T0870 permit a constructor input kind `K`. The
existing regression now asserts the constructor publishes with parameter kind
`Type`; all 302 selected semantic ADT tests pass, with 25 intentionally ignored.
The associated owner-kind incremental regression also passes: changing the
owner parameter kind from `Type` to `Shape` rebuilds the checked signature and
matches cold analysis; the type-member incremental module passes 24/24.
Runtime constructor-reference failures report `invocation layout has 1/2 structural
slots for Method(0)` and reproduce on the same clean predecessor, so they are
recorded as inherited VM baselines. The stale AST snapshots and selector
diagnostic, plus the impl witness failure, were not introduced by this plan.

The focused metadata round-trip, imported-provider invalidation, cold/incremental
parity, runtime-negative, bootstrap-counter, and VS Code e2e obligations have
direct evidence. The bootstrap counters are unchanged from P3's starting
revision. The broad semantic run remains incomplete; the final ledger records
that limitation and the clean-predecessor dispositions. No C3 acceptance
blocker remains unresolved.

Source-level performance review found no new algorithm in the certification
correction:
static constructors remain hash-consed through `TypeStore`'s constructor map;
associated selection remains owner/base-indexed by `AssociatedSurface`;
complete-pack projections still delegate to the existing `CompleteTypePack`
projection; metadata still traverses canonical type/domain graph nodes; and
signature invalidation remains dependency-directed, with the associated
generic-header dependency regression passing. Bootstrap counters are identical
at the P3 starting revision and current tree (106/37/101).

### P3 completion audit follow-up — 2026-09-25

The final checklist audit found stale unchecked boxes in the P3 hard-predecessor,
T1, T4–T8, and completion truth-table sections. They are now checked against
the gate and coverage evidence above, accepted amendments INC-C3-12 through
INC-C3-14, and the P3 walkthrough/handoff. This is lifecycle record repair;
it does not change the P3 verification classification.

The T5 source-census category G disposition is explicit: the self-hosted parser
files are design/implementation drafts, have no workspace/package/test
entrypoint, and remain historical material rather than active lexical
authority. The editor showcase's header now describes its current closed-sum
`data` examples without presenting `enum` as current syntax.

The T8 audit corrected normative `objects/enums.md` wording that still called
the source owner an “enum declaration,” despite its compatibility header and
examples already establishing `data` as canonical. The document now assigns
closed-sum cases and behavior to sum-shaped `data` declarations and explains
that remaining enum terminology refers only to runtime/semantic concepts. The
Universe ownership text likewise uses closed-sum/case terminology. A targeted
spec scan found no remaining claims that enum declarations, enum syntax, or
enum keywords are current canonical Phalcom source.
```

Historical P1-start checklist (retained for audit context; no longer an active
instruction):

1. verify local revision/worktree;
2. re-read live C3 target paths;
3. update `PROGRAM.md` / `STATUS.md` active lifecycle state;
4. mark checkpoint `IN_PROGRESS / PARTIAL / UNVERIFIED`;
5. mark P1 active;
6. do not begin P2 representation work opportunistically.

---

## 22. C3 completion truth table

C3 may be marked `COMPLETE` only when:

```text
[x] P1 complete
[x] P2 complete
[x] P3 complete
[x] P3.R1 corrective closure complete
[x] unified algebraic source/semantic model implemented
[x] type-data static domains implemented
[x] heterogeneous static application implemented
[x] associated type data implemented
[x] intrinsic structural domains exposed
[x] enum canonical source migrated/finalized
[x] exact-case/GADT behavior preserved
[x] metadata/import/incremental identity certified
[x] LSP/source-index baseline certified
[x] authoritative specifications coherent
[x] runtime-negative type-data boundary certified
[x] deferred failures classified
[x] checkpoint state truthful
[x] successor action explicit
```

## 23. P3.R1 corrective closure — 2026-09-25

P3.R1 resolved the binding-projection defect discovered at the P3
T4 endpoint. One canonical `DeclarationTypeInfo` now retains semantic
application behavior and optional runtime/native ownership as sibling fields.
Semantic annotation formation selects Unit/Tuple/Record behavior from the
semantic discriminator; native import and runtime facade lowering use the
runtime owner. Ordinary declaration-shell refresh preserves already-registered
projections, and the declaration-shell fingerprint includes both.

The fixed authorities remain intact: Unit is `store.unit()` semantically and
reuses its preinstalled VM class; Tuple consumes `CompleteTypePack` and
canonicalizes through `TupleFromPacks`; Record consumes `RecordRowId` and
produces the existing `TypeData::Record`; native Tuple surfaces resolve to the
existing Tuple declaration/class; `AssociatedSurface` and type-member identity
are unchanged. No extra runtime/semantic owner registry or RecordRow-to-TypeId
conversion was introduced.

P3.R1 focused evidence:

- semantic declarations: 8 passed;
- semantic type annotations, including ordinary type-lambda applications and
  Tuple/Record without a runtime projection: 51 passed;
- semantic native Tuple runtime-owner import: 1 passed;
- semantic declaration-shell fingerprints: 34 passed;
- semantic associated/type-member closure, including `TupleEntries` over
  concrete and symbolic Tuple: 10 passed;
- semantic incremental type-member suite: 24 passed;
- core Unit intrinsic facade reuses the preinstalled class: 1 passed.

P3.R1 handoff: [LANG007.C3.P3.R1-handoff.md](LANG007.C3.P3.R1-handoff.md).

## 24. P4 active execution — 2026-09-25

P4 reopens C3 to complete intrinsic structural data, structural kinds,
product-shape expansion, and Tuple/Record semantic completion.

Active plan: [LANG007.C3.P4-intrinsic-structural-data-semantic-completion-plan.md](LANG007.C3.P4-intrinsic-structural-data-semantic-completion-plan.md).
Current status: `IN_PROGRESS / PARTIAL / UNVERIFIED`.
Audit-remediation baseline: `main` at `86dc7c26` (`test(lang007-c3-p4): expose runtime evidence and specialization gaps`). Conditional evidence remediation is recorded in commits `6e2197e6` and `ccde8670`; the next implementer should continue from current `main` after consolidation.
The user-ratified Tuple/Record structural behavior-owner refinement is recorded
in §17. Its source implementation and the latest pack-syntax clarification are
part of the active tree. Audit-remediation implementation evidence and focused
results are recorded below; P4 remains open for complete-pack canonicalization,
remaining regression classification, the outstanding plan tasks, and final gates.

### P4 constructor-shape consultation — 2026-09-26

The implementer found that `DataInfo` publishes shape/constructor templates,
but the generic call path still binds fixed callable parameters only and
rejects expanded argument shapes. The architect confirmed that P4 should add a
narrow data-constructor shape adapter to the shared callable application path.
The adapter maps source arguments to fixed formals or pack/row contributions,
preserves one-pass source order, and submits constraints to the existing
inference coordinator and `GenericApplicationSession`. `CompleteTypePack`
contributions remain inference terms; `RecordFields` remains a row equation
and `StaticArgument::RecordRow`. No fake components, second solver, or general
runtime-expansion support in `bind_static_arguments` is introduced.

The P4 plan's T4.4–T4.6 contract was amended with this seam and the single-pass
requirement. The data-constructor adapter is present in the shared call path;
its required focused coverage and shape-materialization behavior remain to be
verified. The attached architecture audit additionally identifies a blocking
TupleIterator typing regression, which now takes precedence as the next
bounded correction. T5 exact layouts and shared behavior-class registration
remain downstream. Generic application syntax remains explicit-expansion-only
as specified in P4 §5.4 and the latest architect note.

### Runtime signature correction — 2026-09-26

The attached audit's independent runtime-signature items 2–5 are implemented
and focused-tested. `SignatureKind` retains structural selector-slot count;
`Signature` now requires explicit positional arity, and `InvocationLayout`
continues to carry the setter value as a separate lane. Native installation
passes the descriptor's complete signature to both value and shape primitive
constructors. Exact invocation validation checks slot count and positional /
labeled decomposition; rest candidate lookup and activation share validation
against the full `RestLayout` and reject setter lanes.

Focused evidence:

- `method::tests::` — 12 passed, including exact, setter, and all four rest
  capture modes.
- `native::install::tests::installed_native_methods_preserve_descriptor_signatures`
  — passed; compares every registered installed descriptor and synthetic
  positional-only, labeled-only, mixed, subscript-get, subscript-set, setter,
  and complete-rest cases across both primitive ABIs.
- `compiler_declaration_dispatch::reflective_send_uses_rest_family_fallback`
  — passed.

### P4 Tuple::Element inference ruling — user-ratified 2026-09-26

The user supplied an architectural ruling for the blocking specialization-aware
TupleIterator regression. `Tuple<***Es>::Element` is a source-owned
domain-native type-member projection. Unqualified lookup must route it through
the published owner surface and canonical type-member evaluator; it does not
require trait evidence. Lookup must select among applicable inherent/domain
and trait-associated candidates as one operation, reporting ambiguity rather
than choosing by syntax spelling or source order.

The exact failing projection IDs were inspected in the same analyzed snapshot:
`TypeId(907)` and `TypeId(1023)` are both `Tuple::Element` projections with the
same `source_declaration` (`Universe::collections.tuple::Tuple::Element`), no
member arguments, and result kind `Type`. Their symbolic subjects differ:
`TypeId(906)` retains `TupleFromPacks(Es::Positionals, Es::Labeled)`, while
`TypeId(1022)` retains the lanes as projections from
`CompleteTypePack<Es>`. `TypeId(1019)` and `TypeId(1024)` are the corresponding
`Option` applications; `TypeId(1026)` is the downstream trait-associated
`Iterable::Iterator::Item` projection. The mismatch is therefore not caused
by source-member identity divergence.

The corrective owner is query-local inference: `TypeData::TypeMemberProjection`
must retain its symbolic subject, member arguments, source declaration, and
result kind through inference substitution and materialization. Projection
results are not injective; equality of `Tuple<X>::Element` and
`Tuple<Y>::Element` must never infer `X = Y`. Generic binder correspondence
must come from the surrounding applied-type constraints. Materialized
projections reuse `TypeStore::type_member_projection` and the existing
projection normalizer. CompleteTypePack inference remains unchanged unless a
projection-free constructor probe still fails after this correction.

### P4 Tuple::Element inference and iteration path resolution — verified 2026-09-26

The specialization-preserving Tuple iterator path:

```text
Tuple<***Es>
    → TupleIterator<***Es>
    → Iterator::Item
    → Tuple<***Es>::Element
```

has been completely implemented and verified without `Dynamic`, conformance weakening, or Tuple-specific solver bypasses.

Key architectural and semantic changes:
1. **Canonical Complete Pack Decomposition:** Enforced `Complete(Positionals(P), Labeled(P)) ≡ P` roundtripping in `TypeStore::canonical_complete_pack_subject` for intrinsic projections.
2. **Unified Domain Head Conformance Matching:** Unified conformance domain head matching in `match_conformance_head_with_mode`, supporting structural/pack heads across concrete syntax (`Tuple`, `Unit`) and abstract pack representations (`TupleFromPacks`, `CompleteTypePack`).
3. **Sound Non-Injective Projections:** Enforced non-injective equality rules for `TypeMemberProjection` and `AssociatedProjection` in inference unification, eliminating invalid reverse subject/argument unifications.
4. **Pattern Destructuring Unwrapping:** Corrected tuple/list element destructuring in `patterns.rs` to unwrap `Option` (`unwrapOr(nil)`).

Focused Verification Evidence:
- `phalcom-semantic/tests/semantic/capabilities/generic_associated_constraints.rs`: All 14 tests pass (covering non-injective member projections, canonical pack congruence, structural domain matching, and deferred symbolic projection normalization).
- `phalcom-core/tests/core/language/iteration.rs`: All 15 tests pass (including `tuple_iterator_preserves_specialized_element_type_through_both_protocols`, `list_for_uses_fresh_stateful_canonical_iterator`, `canonical_set_iterator_is_fresh_sticky_and_ordered`, and `canonical_bytes_and_string_views_iterate_their_stateful_surfaces`).
- `phalcom-semantic --lib`: All 119 tests pass.

### P4 audit remediation — conditional call evidence capability — 2026-09-26

The audit exposed that conditional selection identity did not carry the
selected inherent implementation's executable conformance evidence. The
architect ratified a semantic-owned capability that combines conditional
selection, exact impl-owned bindings, and proof provenance for each conformance
constraint. `phalcom-semantic` publishes that product to both direct calls and
bound-family members; core projects it through the existing runtime recipe
machinery. The VM materializes bound-family recipes at capture time and only
activates the recipe for `SelectedFallback`. A strict-subclass
`OrdinaryOverride` receives empty type/conformance environments.

Commits on the local remediation branch:

- `6e2197e6` — semantic conditional selected-call capability and proof-source
  fingerprint/dependency tracking.
- `ccde8670` — core recipe projection, direct and captured-family activation,
  and override evidence isolation.

Focused evidence:

- semantic capability publication: Exact, Forwarded, Derived-with-nested-
  evidence, and same-trait/different-subject tests passed (one each);
- core `p4_conditional` integration filter: 3 tests passed, covering direct
  selected fallback, captured-family fallback, and invocation after the
  creator activation exits;
- direct and captured-family strict-subclass override unit tests: 1 each
  passed, both asserting the override frame has an empty conformance
  environment;
- `git diff --check` passed before the local runtime commit.

Established invariant: neither direct nor reified conditional dispatch reads
the caller frame's ambient conformance environment. Exact, forwarded, and
derived proof sources are projected into the selected callable's own slot
layout; reified forwarding slots are materialized while the creator frame is
active. Runtime does not reconstruct evidence from `ImplId`.

The solver-soundness, conditional runtime evidence, and complete-pack
source-backed canonicalization findings are addressed; remaining regression
classification, T4/T5 and later P4 obligations, and certification remain open.

### P4 audit remediation — complete-pack source identity — 2026-09-26

Completed the audited law:

```text
Complete(Positionals(P), Labeled(P)) ≡ P
```

`TypeStore` now recognizes the canonical intrinsic lane operations for
source-less internal projections and for projections carrying the canonical
`CompleteTypePack::Positionals` / `CompleteTypePack::Labeled`
`TypeMemberDeclarationId`s. Recognition retains and validates source identity,
checks each lane's result kind, requires zero arguments and the same
`CompleteTypePack`-kind subject, and does not treat selector spelling alone as
intrinsic authority. Source-visible projections keep their canonical member
declaration IDs. Different subjects and unrelated user-owned members do not
collapse.

Focused evidence from [Actions run 36264589395](https://github.com/aureat/phalcom-lang/actions/runs/36264589395):

- `complete_pack_reassembles_only_its_own_intrinsic_lanes`: 1 passed, 118
  filtered;
- `source_visible_complete_pack_lane_projections_keep_canonical_member_declarations`:
  1 passed, 1710 filtered.

The source-visible selector deliberately feeds the published product IDs to
`TypeStore`, proving source-declaration identity is retained through
normalization. It does not establish that the annotation resolver handles a
kind-only `TypeData::Parameter` source fixture; an earlier attempt at that
fixture was superseded by this narrower discriminator. No broad suite or
release gate was run for this item.

The next audit item is runtime-regression investigation/classification against
the named predecessor revisions, followed by the remaining semantic-failure
classification. P4 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`; the overall
audit and release gates remain open.

### P4 audit remediation — nested derived conformance runtime layout — 2026-09-27

The CV50 multi-witness regression was caused by core recipe projection assigning
every nested conditional-impl proof to runtime slot `0`. The compiler now
reconstructs the selected source impl's canonical `GenericEvidenceLayout`,
aligns the already-published nested proofs with their source constraints
(including Equivalent/Subtype placeholders), and places each executable trait
requirement at its canonical slot. `DataComponent` requirements remain omitted
from method recipes, consistent with the existing runtime representation.

Commits `4f0edc63` and `250c4ad4` contain the focused correction and compatibility
follow-up. The exact CV50 selector passed independently: 1 selected, 1 passed,
681 filtered. No broader core suite or release gate was run.

The adjacent rest-method probe is not runtime-environment evidence: its semantic
expression product is `Unknown(DynamicMessageSend)` with no callable or call
specialization, so lowering emits ordinary `Invoke` and supplies no recipe to
the VM. It remains a separate semantic-dispatch question.

The CV51 and repeated-identical-environment probes initially lost
`CallSpecialization` after earlier opaque calls because their inputs were
mutable module-level `let` bindings. With only those inputs changed to `const`,
the exact tests pass and retain their original runtime assertions. The bounded
generic witness/default probe needed the same fixture correction. No flow/effect
policy or assertion was changed. Commits `8b680710` and `3edca835` record these
fixture-intent corrections; each affected exact selector passed 1/1.

Priority 2's nine audit-named selectors have now each passed individually; this
does not certify a broad core suite or release gate. CV44's `c1`/`c2` inputs
were changed to `const` in `c0fd12b2`. Direct-field delegation changed only
`counter` from `let` to `const` in `293c8766`: the index had an eligible
contribution, but opaque sends had made the receiver
`Unknown(NoTypeEvidence)` before resolver entry, so this was not a missing
default-method installation. The multi-parameter, same-trait/distinct-subject,
and owner/method-local probes passed with only the affected inputs stabilized
as `const` in `84051c01`, `72565cb7`, and `ae5409b9`; assertions were unchanged.
The owner/method test is not ignored in the live tree: ignored mode selected
zero, while the normal exact selector selected and passed one. Full per-test
classification is in the [P4 audit remediation record](LANG007.C3.P4-audit-remediation/03-nested-derived-conformance-runtime-layout.md).

The selected-fallback branches of `InvokeConditional` and conditional-family
dispatch carry their explicit environments; ordinary overrides remain
evidence-neutral. Rest and module-export source probes produced no callable,
call specialization, or runtime recipe, so no runtime environment drop was
proven and no speculative adapter change was made. Priority 3's 40-way
classification now has a later full pinned semantic result recorded above:
37 failures, 3 passes, 42 ignored. The first Phase B T4 materializer gate
rejects concrete fixed/expanded Record duplicate labels, while the symbolic
row-lacks control remains green. Continue the remaining T4 shape-specialization
acceptance. P4 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`; overall audit and
release gates remain open.

### P4 Phase B T4 — concrete Record duplicate materialization — 2026-09-27

`materialize_data_product_shape` now detects duplicate logical labels while
appending both fixed Record components and materialized `RecordFields` row
fields. It returns `DataProductShapeMaterializationError::DuplicateRecordField`
with the conflicting label instead of allowing an ambiguous concrete product
layout. The change remains at the canonical TypeStore-backed materialization
owner: it adds no component identity, changes no row identity, and does not
move symbolic lacks checking out of the existing inference solver.

Focused evidence with `nightly-2026-07-10`:

- `data_record_materialization_rejects_duplicate_fixed_and_expanded_labels` —
  1 selected, 1 passed; also asserts a non-conflicting concrete row preserves
  the ordered fixed and expanded fields.
- `return_record_prefix_contributes_lacks_constraint_to_row_inference` —
  1 selected, 1 passed unchanged as the symbolic row-lacks control.
- `cargo fmt -p phalcom-semantic -- --check` remains blocked by pre-existing
  formatting drift elsewhere in the crate; no broad formatting was applied.

This closes only the concrete materializer duplicate check. CV-14's constructor
inference and broader T4 acceptance, T5, P4 completion, and release gates remain
open.

### P4 Phase B T4 — constructor row-lacks call-path discriminator — 2026-09-27

`compute_shape_aware_data_argument_mapping` converts a materialization error to
`None`, after which associated argument mapping may use the fixed-constructor
fallback. This mapping is downstream of `apply_resolved_callable`. In the live
constructor adapter, labels matching fixed Record components are bound to the
fixed parameters; only unmatched labels contribute to the inferred row. When
the row expansion is complete, `constrain_record_row_expansion` adds lacks for
each fixed field before solving the residual row equation. Repeated source
labels are independently rejected by parsing.

The new source-level discriminator gives the constructor an expected type
`Envelope<**{ id: Int }>` while constructing its fixed `id` field. The exact
selector `data_record_constructor_rejects_expected_row_repeating_fixed_label`
passes (1 selected, 1 passed, pinned `nightly-2026-07-10`): this incompatible
fixed/expanded shape receives a semantic `GenericInferenceConflict` diagnostic
at the constructor call. The diagnostic is not `RecordRowLacksViolation`, so
this evidence establishes rejection but does not claim that this path exercises
the dedicated lacks-diagnostic mapping. The existing
`return_record_prefix_contributes_lacks_constraint_to_row_inference` remains a
separate 1/1 control for the dedicated row-lacks route. The direct materializer
test remains the precise DuplicateRecordField oracle.

This resolves the `.ok()?` fallback concern for the tested accepted-language
collision without changing solver ownership or constructor mapping behavior.
It does not close the remaining CV-14/T4 shape-specialization acceptance, T5,
P4 completion, or release gates.

### P4 Phase B T4 — CV-10..CV-15 focused coverage inventory — 2026-09-27

The next T4 acceptance slice adds direct CV-10 evidence. The exact pinned
selector `user_complete_pack_data_type_keeps_nominal_identity_distinct_from_tuple`
passes (1 selected, 1 passed, `nightly-2026-07-10`). It applies a user
`MyTuple` root to a `CompleteTypePack`, verifies the result is `Applied` with
that exact static argument, constructs structural Tuple from the same lanes,
and proves the two identities differ.

| Criterion | Focused evidence found | Remaining direct evidence |
| --- | --- | --- |
| CV-10 user Tuple nominal identity | `user_complete_pack_data_type_keeps_nominal_identity_distinct_from_tuple` — 1/1 | Covered for canonical semantic type identity. |
| CV-11 user Record nominal identity | `user_record_row_data_type_keeps_nominal_identity_distinct_from_record` — 1/1 | Covered for canonical semantic type identity. |
| CV-12 expanded constructor inference | `complete_pack_data_constructor_infers_nonzero_pack_without_expected_result` — 1/1; covers one, heterogeneous many, and a labeled lane without an expected result. `complete_pack_data_constructor_infers_exact_empty_pack_without_expected_result` — 1/1; exact empty CompleteTypePack is Established from validated constructor shape. | Covered for zero/one/many positional plus labeled contributions. |
| CV-13 fixed prefix + pack | `complete_pack_data_constructor_preserves_fixed_prefix_order_without_expected_result` — 1/1; verifies nominal result, inferred pack lanes, and materialized fixed-prefix-then-pack type order without an expected result type. | Covered for one fixed positional prefix plus two expanded lanes. |
| CV-14 fixed Record + row | `record_row_constructor_infers_fixed_and_residual_fields_without_expected_result` — 1/1; duplicate rejection, source conflict, and symbolic-lacks controls; `data_record_materialization_preserves_symbolic_open_row_shape` — 1/1 retains exact open `RecordRowId`, known field, fixed prefix, and `Symbolic` state. | Covered within accepted surface. Source-level open-row forwarding would require record spread, explicitly deferred by `docs/specs/typing/type/type.record-row.md`; no new syntax is introduced. |
| CV-15 heterogeneous nominal RecordRow argument | Known-value construction retains exact `StaticArgument::RecordRow`; Assumed, mixed Established/Assumed, exact-empty, Unknown, and Dynamic constructor proof tests pass; `record_row_constructor_result_matches_cold_analysis` — 1/1. | Covered; Unknown/Dynamic remain epistemic-status assertions where no canonical known TypeId is published. |
| Cold result stability | `complete_pack_data_constructor_empty_result_matches_cold_analysis` — 1/1 and `record_row_constructor_result_matches_cold_analysis` — 1/1 compare nominal result, exact lanes/row fields, and proof status after incremental update against cold analysis. | Covered for inferred CompleteTypePack and heterogeneous RecordRow products. |

The CV-11 selector `user_record_row_data_type_keeps_nominal_identity_distinct_from_record`
passes (1 selected, 1 passed, pinned `nightly-2026-07-10`). It verifies that
user `MyRecord` stays `Applied` with the exact heterogeneous `RecordRow`, while
a structural Record formed from the same fields reuses that row identity and
remains a distinct type. The CV-10..CV-15 inventory and focused checks below
are accepted as T4 `IMPLEMENTED / FOCUSED_TESTED`. T5 is next and has not
started. P4 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`, and audit/release
gates remain open.

CV-12 now has direct value-driven zero/one/many evidence. The exact empty case
passes `complete_pack_data_constructor_infers_exact_empty_pack_without_expected_result`
(1/1); the validated CompleteTypePack constructor shape selects canonical
empty positional and labeled lanes and supplies Established
`ConstructorSemantics` evidence, not a solver default. The one/many selector
`complete_pack_data_constructor_infers_nonzero_pack_without_expected_result`
passes (1/1) for `PackBox(1)`, `PackBox(1, "value")`, and
`PackBox(1, labeled: true)`, asserting exact positional and labeled lanes.
Nonzero source arguments retain their actual `TypeKnowledge` as required
premises with `DataExpansionArgument` provenance; synthetic pack equivalence
remains shape binding, not evidence. The contextual control
`projection_free_complete_pack_data_constructor_infers_pack` also passes
(1/1). Cold/incremental result stability is covered separately below.

CV-13 now has a direct semantic acceptance test:
`complete_pack_data_constructor_preserves_fixed_prefix_order_without_expected_result`
passes (1 selected, 1 passed, pinned `nightly-2026-07-10`). With no expected
result type, `Prefixed(7, "tail", true)` publishes a nominal `Prefixed`
application whose CompleteTypePack contains only the `String`/`Bool` expansion
lanes. `materialize_data_product_shape` returns a concrete shape ordered as
`Int`, `String`, `Bool`; the fixed prefix retains its declared component ID and
the expanded lanes do not invent component IDs. No production change was
needed. The constructor application loop in
`checker/call.rs::apply_generic_callable_in_context` walks the source arguments
in order and analyzes each residual expression once; the one/many regression
confirms the resulting lane order without a second constructor inference path.

CV-14/CV-15 now also have a direct known-argument source discriminator:
`record_row_constructor_infers_fixed_and_residual_fields_without_expected_result`
passes (1 selected, 1 passed, pinned `nightly-2026-07-10`). Without an expected
result type, `Envelope { id: 1, name: "Ada", age: 42 }` publishes nominal
`Envelope` with the exact closed `StaticArgument::RecordRow` containing
`age: Int` and `name: String`. Canonical materialization yields `id`, `age`,
`name` in logical fixed-prefix-then-sorted-row order; only `id` retains a
declaration component ID. Assumed, mixed Established/Assumed, exact-empty,
Unknown, and Dynamic source-row evidence are covered by focused constructor
tests; the query-local proof matrix preserves only return-relevant strength.
Cold/incremental parity for the inferred heterogeneous row is also tested.

### T4 query-local RecordRow proof propagation — first focused gate — 2026-09-27

Architect ruling: proof strength belongs to the active `GenericApplicationSession`
and is keyed by `RecordRowVarId`; it must not enter canonical `RecordRowId` or
`RecordRowField`, and `RecordRowSolver` remains epistemic-agnostic. Source
`RecordField` knowledge is captured before conversion to inference terms, row
dependencies are followed during return publication, and ordinary type proof
is met only with row proof reachable from the returned term. Fixed/lacks/where
constraints do not establish value evidence. A context-selected row is
Assumed, except that compatible context must not downgrade independent
Established source evidence. A solved exact closed-empty row is Established;
Unknown and Dynamic retain the weakest status.

The first focused slice implements the query-local proof bridge and passes
`record_row_constructor_preserves_assumed_generic_field_evidence` (1/1),
`expected_result_can_select_row` (1/1, now asserts Assumed), and
`source_established_row_stays_established_under_compatible_expected_result`
(1/1). The existing CV-14/CV-15 known-value constructor control remains
available; no evidence was added to canonical row identity. This is partial
T4 evidence only; P4 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`.

The second focused slice completes the current proof-strength/dependency
discriminators: four `row_proof_matrix_tests` pass (Established, Assumed,
mixed/multiple contributions, Unknown, Dynamic, exact empty row, restricted
unproven row, nested dependency, and return-irrelevant weak row); six focused
`record_row_constructor_*` tests pass (known, assumed, mixed, exact empty,
Unknown, and Dynamic source fields). The assumed and unknown constructor cases
now also carry `where T <: Number`, while the fixed `id` field imposes the
existing row-lacks restriction; their exact selectors preserve Assumed and
Unknown respectively. The two context-selection tests pass individually,
preserving Assumed for context-only selection and Established when compatible
context accompanies independent source evidence. The separate
`return_irrelevant_dynamic_row_does_not_weaken_exact_int_result` test passes
1/1. Focused commands selected 4 unit tests and 6 constructor tests; each of
the three integration discriminators selected and passed 1 test. Together
with the CV-10..CV-15 evidence above, these results support T4
`IMPLEMENTED / FOCUSED_TESTED`; they do not assert a broad suite gate or close
P4. T5 has since started and reached the runtime representation blocker below.
P4 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`.

### P4 T5 — runtime specialization representation gate — 2026-09-27

T5 is active but not accepted. The shared declaration behavior class,
exact descriptor/layout registry path, and specialization-aware expanded-field
projection have focused evidence: the registry separation test, MyTuple and
MyRecord specialization tests, the fixed-plus-expanded Envelope field test,
the fixed-only control, and the semantic assumed-evidence projection test each
passed individually. No broad plan or release gate was run.

The generic-callable discriminator reaches the planned T5.2 stop condition.
Its callable environment carries distinct `T=Int` and `T=String` bindings, but
exact result specialization fails because
`phalcom-core/src/typing/environment.rs::instantiate_type_ref` declines applied
types containing `StaticArgumentRef::RecordRow`, and
`phalcom-core/src/typing/overlay.rs::RuntimeOverlayTypeNode::Applied` stores only
type arguments. Keeping the type recipe closed erases the distinction between
the two row-specialized results; marking it as a template exposes the unsupported
row reification and fails with `generic data type could not be instantiated`.

This was a runtime type representation/canonical identity decision, not a local
lowering adjustment. INC-C3-15 below adopts heterogeneous runtime `Applied`
arguments with closed-row type substitution and exact overlay interning. The
implementation may now resume from clean local main `0fe6e270`. Do not use
erasure, per-specialization classes, another registry, broaden runtime
environments to row-valued bindings, or weaken exact specialization acceptance
to bypass the gate.

P4 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`; T5 is incomplete, and no
checkpoint or release acceptance is claimed.

### INC-C3-18 — T6.6 generic exact-case contextual result boundary — 2026-09-27

Normative rule: `docs/specs/objects/enums.md` § nullary variants states that
`Option::None` evaluates for the inferred or explicitly supplied exact
`Option<T>` application. A generic expected result may therefore specialize
an otherwise-underconstrained unqualified exact case only when inference proves
the same canonical applied enum owner. An explicit conflicting owner and a
no-context lookup remain unchanged.

Observed: the canonical bootstrap first failed in
`universe:errors.result::Result.transpose` at its nested `Option::None`. The
declared result `Option<Result<U, E>>` reaches the body-tail match and the
`synthesize_match_expr` branch expectation. The failing source used a braced
match arm; parser `parse_match_arm_branch` represents those braces as an
`Expr::Block` closure, whose callable return expectation is independent of the
enclosing non-callable return type. Its tail expression consequently receives
`ExpectedType::None`. The direct-match semantic discriminator receives the
expected owner and publishes the exact case. Impl owner/callable parameters
and the `where T == Option<U>` domain constraint are available in body scope;
they do not block construction of the declared result type.

Historical decision (superseded by INC-C3-19): the `Result::Ok(option)` arm
was rewritten as a direct nested `match` because the parser represented its
braces as a Closure. Preserve the exact-case rule: contextual specialization
is accepted only after the materialized `ExactCase.enum_type` equals the exact
expected applied owner; other residual generic values still use the ordinary
underconstraint guard.

Evidence retained historically: the canonical bootstrap advanced past
`universe:errors.result` but failed in `universe:option.option` with
`AssociatedLookupNotLoweredYet`; this was not a passing bootstrap gate. The
focused semantic generic-context, no-context, conflicting-owner, and
associated lookup controls each selected/passed 1 test. No codegen fallback
was restored. The source rewrite and no-parser-change conclusion are
superseded: match-arm/if braces are lexical control-flow bodies, not Closure
construction.

### INC-C3-19 — lexical control-flow bodies versus Closure values — 2026-09-27

Decision: represent lexical control-flow bodies distinctly from first-class
Closure expressions. Braced match arms and source `if` bodies are lexical
bodies: semantic analysis executes them inline in a nested scope, retains the
enclosing return/control-flow context and flow, and propagates the expected
result type. Explicit, parameterized, and trailing Closure syntax remains a
Closure with independent capture, expectation, and construction behavior.
Compiler lowering consumes the distinct AST forms; match arms already lower
their branch blocks inline. Do not restore codegen-time `UniverseKey` name
recognition or extend this work into a Closure runtime overhaul. Source `while`
may follow the same lexical model only if the shared representation directly
requires it for P4 acceptance; consult root before broadening.

INC-C3-18's `Result::Ok(option) => match ...` source rewrite was a workaround
for conflating braced lexical bodies with Closure construction and is
superseded. Restore the natural braced match arm in
`phalcom-core/core/universe/src/errors/result.ph` when the shared semantics
work is ready. The same rule must fix `Option.filter`'s braced `if` body
without special-casing either Universe source.

Acceptance requires focused evidence for: match-arm versus Closure AST shape;
dedicated `if` versus explicit/trailing Closure parsing; expected-result
propagation through match and `if`; Closure expectation isolation; flow joins
without Closure construction; inline match lowering without Closure
construction; and canonical Universe bootstrap through both Result and
Option. T6.6 remains open until these gates pass; P4 remains
`IN_PROGRESS / PARTIAL / UNVERIFIED`.

Attribution/verification update — 2026-09-28: with `RUSTFLAGS=''` and
`RUSTC_WRAPPER=''`, the exact selectors were run on detached clean predecessor
`fd88ad76` using `cargo test -p phalcom-core --test language-corpus
corpus::errors -- --exact` and the corresponding `corpus::option` command.
`corpus::option` passed 1/1. `corpus::errors` selected 1 and failed during
fixture execution in `errors/contracts_invariant_cross_receiver.ph` because
`B` could not call `invariantEnter()` owned by `<unbound>`; this is materially
different from the current implementation's pre-fixture union panic and is
not evidence that the current union failure is a baseline.

At the pre-remediation implementation snapshot committed as `27d224a5` on
`main` and pushed to `origin/main`, both selectors panicked before fixture
execution at `TypeStore::union`'s unchanged `is_proper_type` assertion. A
temporary trace (removed after capture) identified `BindingId(0)` / `list` at
`universe:collections.bytes::Bytes.fromList`, declaration range `[5665,5669)`:
both incoming facts were `Known(TypeId(190), Established, Flow)`, with
`TypeData::Nominal { declaration: universe:collections.list::List }` and kind
`(Type) -> Type` (`kind_of != KindId::TYPE`), so this is not a proper Type.
`bytes.ph` has `if list is not List` at lines 213–215 and the loop at lines
218–221. Predicate extraction inverts the false polarity to the positive
`IsInstance(list, List)` observation; `positive_type_refinement` assigns the
target as `Established(..., Flow)` for an unknown binding without checking
that the target is a proper Type. The new `Expr::If` path runs
`analyze_condition_split` and retains `split.when_false` as the normal
post-if flow; the later `whileTrue` loop-header join consumes that fact.

The confirmed defect is that the trusted type-test refinement producer could
admit a higher-kinded declaration-family form into ordinary value
`TypeKnowledge`. The lexical `If` path is an exposure/retention delta: it
allows the later join to observe the invalid fact, but the full predecessor
masking behavior is not isolated. The predicate/refinement machinery and
`analyze_branch_pair` predate this change; the predecessor's parser sent source
`if` through `ifTrue` and the same branch-pair helper also installed
`split.when_false`. Do not restore predecessor information loss as a repair.
The `TypeStore::union` assertion and proper-Type boundary remain intact.

Supplemental ruling — 2026-09-28 (adopted): the defect is at the static
refinement producer boundary, not in `TypeStore::union`, loop joining, or
lexical `if`. Keep three projections distinct: declaration identity; the
runtime class-object/value projection accepted by the classifier method; and
the type-family projection (for example `List : Type -> Type`, whose applied
`List<Int>` is a proper `Type`). A runtime classifier test remains a valid
Boolean even when it cannot produce a representable static refinement.

`FlowPredicate` is the existing static-refinement IR, not a general runtime
proposition. Emit a predicate only when the canonical runtime classifier has a
supported interpretation and the runtime observation entails the proposed
proper static value type. Properness is necessary but not sufficient for
future applied-generic classifiers: runtime ancestry alone does not prove
generic arguments. For now, fail closed for bare higher-kinded families and
unsupported/applied generic forms; preserve prior `TypeKnowledge` and do not
invent an erased specialization, existential, or placeholder type. Future
family-aware filtering may refine already-known proper alternatives or use a
dedicated runtime-class fact domain, but that work is outside P4. Keep a
defensive proper-type check at publication.

The shipped `Object#is` implementation is subclass-inclusive and
`Object#is!` is exact-class equality (`core/universe/src/object/object.ph`);
the normative reflection-pattern document defers runtime `is` semantics.
Therefore do not equate `is!` with ordinary subtype exclusion: its positive
branch may derive weaker membership when sound, but its false branch emits no
`IsNotInstance` predicate without an exact-class type representation. This
records current runtime implementation semantics, not a new normative rule.

The bounded implementation leaves runtime Bool evaluation unchanged, rejects
unsupported classifier-to-static projections at the producer, and preserves
the strict union invariant and lexical/loop behavior. Existing INC-C3-19
evidence covers proper nominal positive/negative `is`, bare-family predicate
omission, unsupported applied-generic targets, exact-class `is!`, inline
lowering without Closure construction, natural braced `Result.transpose`,
direct `Bytes.fromList`, and Option filter/canonical Option execution. The
supplemental matrix is now focused-tested: source-backed bare `List` and
`is not List` expressions yield Bool knowledge without error diagnostics;
both polarities omit unsupported predicates and preserve known `List<Int>`;
branch/loop joins retain the applied fact; and false exact-class `is!`
preserves a subclass fact. The source expressions' analysis status is
`Blocked(UnknownType(NoTypeEvidence))` because canonical `Object.is` leaves
its classifier parameter unannotated. This is missing static argument evidence,
not an invalid Bool result or published refinement; no signature/reification
change is authorized. Focused test results are listed below.

The focused acceptance evidence is now recorded: the parser/semantic/branch
controls listed above, plus inline lowering without Closure construction,
natural braced `Result.transpose`, direct `Bytes.fromList`, and Option
filter/canonical Option execution. `corpus::errors` now reaches fixture
execution and has the same `invariantEnter` unbound-owner error as `fd88ad76`;
classify that selector `BASELINE_BLOCKED`, not PASS. `corpus::option` passes
1/1, and the focused `Result.transpose` test passes 1/1.

The separate `corpus::bytes` selector selected 1 and failed during compilation
of `bytes_equality_and_keys.ph` with `type.generic.underconstrained`. That
fixture does not use a type-test predicate, and the direct
`canonical_bytes_and_string_views_iterate_their_stateful_surfaces` test
invoking `Bytes.fromList` passes 1/1. Classify the broader selector as
unclear/nonblocking for INC-C3-19; do not change unrelated generic inference.

Implementation/verification update — 2026-09-28: the producer now admits only
the resolved declaration's direct nominal form after a proper-type check, and
the transfer boundary independently rejects non-proper `IsInstance` /
`IsNotInstance` targets before changing knowledge or publishing a flow fact.
The added producer discriminator, existing AST predicate-extraction control,
added direct transfer guard, and existing pure transfer matrix each selected
and passed 1/1. The exact `corpus::option` selector passed 1/1. The exact
`corpus::errors` selector now reaches fixture execution and fails at
`contracts_invariant_cross_receiver/invariantEnter` with an unbound-owner
runtime error, matching the exact failure observed on clean predecessor
`fd88ad76`; it no longer panics at `TypeStore::union`. Treat that selector as
`BASELINE_BLOCKED`, not PASS. Focused regression selectors:

- `semantic::foundations::flow_graph::type_test_refinements_require_supported_proper_targets_and_respect_exactness` — 1/1.
- `semantic::capabilities::flow_branches::bare_generic_type_tests_preserve_applied_facts_through_branch_and_loop_joins` — 1/1.
- `test_predicate_extraction_from_ast` — 1/1.
- `non_proper_type_test_targets_do_not_change_or_publish_flow_facts` — 1/1.
- `test_predicate_transfer_pure_matrix` — 1/1.
- `language::compiler_lowering_scenarios::lexical_if_and_match_branches_lower_inline_without_closure_construction` — 1/1.
- `language::algebraic_data_scenarios::result_transpose_executes_natural_braced_lexical_match_arm` — 1/1.
- `language::iteration::canonical_bytes_and_string_views_iterate_their_stateful_surfaces` — 1/1.
- `object_model_invariants::isa_is_reflexive_and_superclass_closed` — 1/1 (runtime classifier Bool/subclass behavior).

Existing checkpoint-recorded AST, flow/expected-result, and Closure-isolation
controls remain reused evidence; no duplicate rerun was needed. The exact
`corpus::bytes` failure described above is recorded as an unclear residual and
is not treated as a failure of the focused `Bytes.fromList` test.

Supplemental architect guidance — 2026-09-28: keep the runtime classifier
proposition separate from the static refinement derived from it. Preserve all
three projections—declaration identity, runtime class-object/value projection,
and type-family projection. Properness is necessary but not the complete future
soundness rule: runtime observation must entail the proposed proper static
type, including applied generic arguments. Do not migrate
`FlowPredicate::target` to `ProperTypeId` unless the live IR is shown to mean
only static refinement and the migration is genuinely mechanical; for P4,
retain the bounded producer guard and defensive transfer guard. Failure to
derive refinement means only “no formal flow refinement,” never an invalid
runtime expression or condition. The direct discriminators for valid-Bool
`is`/`is not List`, preservation of known `List<Int>` in both polarities,
branch/loop join safety, and exact-class `is!` false behavior are now
focused-tested. Existing proper nominal, unsupported applied-generic, Bytes,
Option, Result, no-Closure, and clean-predecessor evidence remains recorded;
do not weaken the union invariant or repair loop/lexical code.

The 2026-09-28 amendment resolves the architectural disposition under
INC-C3-19, and the producer repair plus supplemental discriminators have
focused evidence. The canonical `corpus::errors` failure remains
BASELINE_BLOCKED, not PASS; this does not promote overall P4 verification.
T7.9–T7.11 are now IMPLEMENTED / FOCUSED_TESTED: lane observers retain exact
structural products, specialized iteration retains `Tuple::Element`, and
runtime-shaped slicing/zip remain behind bounded Dynamic APIs. T7.11 also
corrects TupleZip's use of the newly honest optional `at(_)` observer. T8.1–
T8.4 are IMPLEMENTED / FOCUSED_TESTED; resume T8.5 under the adopted ruling
below, then continue with T8.6–T12.

T8.3 preserves the explicit Map source head and its range in `MapLiteralExpr`.
Semantic checking resolves that head through the existing type resolver and
requires the canonical Universe `Map` declaration identity; a local value or
declaration shadow emits a deterministic diagnostic and yields unknown
knowledge rather than silently constructing the builtin. The AST identity and
semantic canonical-head selectors passed 1/1 each on supervisor rerun. No
string-only identity registry or compiler-side special case was added.

T8.4 is IMPLEMENTED / FOCUSED_TESTED with no production change required. The
semantic selector covers explicit specialization, entry/context inference,
Map expansion, and empty-map underconstraint; the direct core selector proves
successful expansion preserves entries and duplicate expanded keys are still
rejected. Both passed 1/1 on supervisor rerun. Broader collections selectors
failed before the Map cases in unrelated iterator/negative-fixture paths and
are not treated as T8.4 evidence or reclassified here.

T8.5 architectural ruling adopted 2026-09-28: Record spread is a
shape-preserving operation and requires static projection to `RecordFields`.
Record rows and labeled Tuple lanes are valid sources; a mixed Tuple contributes
only its labeled lane, through a checked projection that does not equate
`LabeledTypePack` with `RecordFields`. Symbolic/open Record rows remain valid
when represented faithfully. Map-to-Record is invalid even for a literal and
must produce a direct diagnostic because `Map<K,V>` does not encode its exact
key set. Admission is type/shape-based, never expression-provenance-based.
Map-to-Map remains unchanged, and Record/Tuple-to-Map remains valid through the
existing entry-inference path. Preserve the broader runtime labeled-lane
machinery; do not fabricate rows, use Dynamic, or introduce another solver.
This supersedes the prior T8.5 hold. T8.5 is IMPLEMENTED / FOCUSED_TESTED.
Semantic selectors passed: `record_spread` selected 3/3; the exact mixed
labeled-Tuple projection selector selected/passed 1/1. The core direct-runtime
selector `static_labeled_sources_spread_into_record_and_collections` passed
1/1, covering Tuple/Record-to-Record and Tuple/Record/Map-to-Map; the migrated
`labeled_spread_record_map.ph` fixture was run directly with the Phalcom CLI
and its output matched the updated expected sidecar. The existing Map expansion
selector passed 1/1, retaining Map-to-Map success and duplicate-key rejection.
Map bindings and Map literals both produce the same `RecordSpreadSourceUnsupported`
diagnostic; symbolic/open Record row identity, closed specialization,
destination duplicate rejection, and unproven row-lacks remain covered.
The aggregate `collections` selector fails before reaching this fixture when
`boundedness_take_bounds_unbounded.ph` hits the inherited
`TakeIterator.toList` runtime error; `collections_literals_negative` fails in
the unrelated `map_key_eq_mutates_same_map_during_overwrite_rejected.ph`
fixture with `type.generic.underconstrained`. These are not T8.5 failures or
acceptance evidence and are not classified as baselines here. Continue with
T8.6.
Priority 3
categories are already recorded; causal isolation, remediation or migration
decisions, and final residual disposition remain. Run T13 only after its
prerequisites. The interim
[P4 walkthrough](LANG007.C3.P4-walkthrough.md) records the current snapshot
and remaining work. It is not the T13 final walkthrough or a P4 completion
claim.

## 25. Completion-audit follow-up — iterator contracts and `toMap` — 2026-09-30

The four generic inference cases named by the 2026-09-29 completion audit were
reclassified as fixture interference with the established mutable-binding
opaque-call policy, not production generic-inference regressions. Their result
bindings are now immutable so the tests isolate call-expression inference;
the four exact selectors each passed 1/1 and the scoped test-only change was
committed as `01e4db77`. Direct call-expression and sibling-call assertions
remain in place.

The Iterator adapter declarations in `iterator_laws.ph` now state precise
associated Item products for FlatMap, Flatten, Zip, and Indexed, with the
required iterator/iterable/equality bounds on their implementations,
constructors, and public default methods. `ChainIterator` now requires both
operands to be Iterators and their Item types to be equal. The extra `it.next`
read in `Iterable.toMap` was removed, and a four-distinct-entry core regression
was added. These source/runtime corrections are implemented but not accepted
as canonical-Universe behavior yet.

The semantic selector
`capabilities::iteration::iterator_adapter_fixture_preserves_item_projections_and_chain_equality`
passed 1/1. It uses an isolated semantic Fixture and asserts symbolic canonical
associated-projection shapes for FlatMap/Flatten/Zip/Indexed, plus rejection of
a heterogeneous chain at the `left.chain(right)` call. The observed rejection
code is `GenericInferenceUnderconstrained`; it proves call-site rejection in
this fixture, not ideal diagnostic specificity and not the canonical Universe
declarations.

The direct core selector
`iterable_to_map_preserves_all_distinct_entries` selected 1 test but failed
before inline source-body execution during Universe bootstrap. The same exact
selector, with the identical regression test added in a detached snapshot of
parent `01e4db77`, also selected 1 and failed at `compile_entry_selection`
before VM setup/body execution. Both runs reported the same 20
`ImplConformanceIncomplete` diagnostics for the `Iterator` `iterator`
requirement's inherent-selector conflict: 14 in `collections.iterator_laws`,
1 in List, 4 in Map, and 1 in Set. The failure class, phase, owners,
requirement, and counts match; only `iterator_laws` source offsets and numeric
TypeIds differ. This core gate is BASELINE_BLOCKED for end-to-end verification,
not a pass. Consequently neither the canonical adapter source contracts nor
the four-entry `toMap` runtime behavior have end-to-end evidence from this
slice. No Universe conformance rule, `TypeStore::union` behavior, or fallback
typing was changed. C3/P4 remains IN_PROGRESS / PARTIAL / UNVERIFIED; the
audit's structural-subtyping/spec conflict and broader gates remain open.
