---
id: LANG007.C3.P4
kind: implementation-walkthrough
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
as_of: 2026-09-29
---

# LANG007.C3.P4 Walkthrough — T13 Certification Outcome

> T13 focused stabilization and the single authorized certification attempt
> are recorded below. P4 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`: the
> broad gates are not all green, and this walkthrough does not claim C3
> closure or release certification.

## Current disposition

P4 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`. The final T13 records and
localized rustfmt-only corrections were committed on local `main` as
`40974862`; no push was performed by this continuation. The user-edited
`docs/.obsidian/workspace.json` and `docs/__unstructured__/Untitled 1.md` were
preserved and excluded from delivery. They remain the only modified paths.

The T13 certification result is incomplete: the one authorized full workspace
test retry lost its command session after reporting the `phalcom-core --lib`
target (157 run, 155 passed, 2 failed), with no final Cargo exit summary or
later target results captured. The repeated exact selectors confirm each
failure individually; their classifications and the remaining G6 command
results are in the T13 section below and the checkpoint. No workspace-wide
pass or release completion is claimed.

The lexical-block implementation remains unchanged. The architect's
2026-09-28 supplemental ruling under [INC-C3-19](LANG007.C3-CHECKPOINT.md)
resolves the disposition: the defect is at trusted static-refinement
production, not in `TypeStore::union`, loop flow, or lexical `if`. A runtime
classifier condition remains a valid Bool even when no sound proper static
target can be derived. The confirmed defect is that a higher-kinded
declaration-family form could enter ordinary value `TypeKnowledge`; the
lexical-If path exposes/retains that fact for a later join, while the exact
predecessor masking delta remains unisolated. Restoring predecessor
information loss is not a valid repair. Do not coerce or filter non-proper terms. The live
checkout at the start of this continuation is local `main` at `27d224a5`,
matching both `origin/main` and the live remote `main` reference. The supplied
architect note describes `8268c54e`, but that revision is not the current local
or remote `main` in this checkout; this record follows the verified live state.
The pre-existing unrelated
`docs/.obsidian/workspace.json` edit is preserved.

## Work and focused evidence recorded

The delivered P4 source changes include dedicated AST/parser forms for lexical
`if` and match-arm bodies, semantic branch-flow and expected-result handling,
and compiler lowering that treats those bodies inline rather than as Closure
values. Associated semantic/native/source-index changes and focused regression
tests are also present in the snapshot. These changes do not by themselves
establish final P4 acceptance.

Before delivery, the following focused checks were recorded as passing:

- AST/parser discriminator for lexical braced `if` and match arms versus
  explicit, parameterized, and trailing Closure syntax: 1/1.
- One-arm `if` Unit/effect behavior and branch-flow mutation controls: 1/1
  each.
- Generic Closure-context control and nested match/`if` expected-`Option::None`
  semantic control: 1/1 each.
- `cargo check -p phalcom-semantic` and `cargo check -p phalcom-core` with
  `RUSTFLAGS=''`: passed.

The canonical corpus comparison is **not** accepted:

| Snapshot | `corpus::option` | `corpus::errors` |
|---|---|---|
| Pre-remediation implementation at `27d224a5` | Panics before fixture execution at the unchanged `TypeStore::union` proper-type assertion | Panics before fixture execution at the same assertion |
| Detached clean predecessor `fd88ad76` | Passes, 1/1 | Reaches fixture execution and fails: `B` cannot call `invariantEnter()` owned by `<unbound>` |
| Current uncommitted producer correction | Passes, 1/1 | Reaches fixture execution and fails at the same `invariantEnter` unbound-owner runtime error as `fd88ad76` |

The prior union panic was a real producer defect, not a baseline; it is gone
after the correction. The remaining `corpus::errors` failure now matches the
predecessor's fixture failure, so that selector is `BASELINE_BLOCKED`, not a
pass. `corpus::option` passes, and the direct `Result.transpose()` E2E test
passes; these are focused evidence for the Option and Result paths,
respectively. Do not treat the enclosing `corpus::errors` selector as a pass.
The exact pinned commands were:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::option -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::errors -- --exact
```

The current-tree trace identified an unapplied higher-kinded `List` refinement
from `if list is not List` in `Bytes.fromList`, later consumed by a loop-header
flow join. `List` has kind `(Type) -> Type`, not `KindId::TYPE`; its flow fact is
not a proper type. The defect was at the producer: the trusted static-refinement
path allowed the family projection into ordinary value knowledge without
checking for a sound proper value type. That is now guarded both when the
predicate is formed and immediately before flow facts/knowledge are published.
`TypeStore::union` and loop behavior were not changed.

Focused semantic checks all passed 1/1: the new exactness/proper-target
producer test, existing AST extraction control, new improper-target transfer
guard test, and existing pure transfer matrix. No workspace or release suite
was run. The no-Closure compiler test passed 1/1 with an explicit Closure
control; the natural `Result.transpose()` execution test passed 1/1; and the
focused `Bytes.fromList` iteration test passed 1/1. `corpus::option` includes
`option_filter.ph` and passed 1/1. Together with the previously recorded parser,
branch-flow, expected-result, and Closure-isolation controls, these satisfy
the focused INC-C3-19 acceptance evidence. `corpus::errors` remains a
baseline-blocked enclosing selector, not a pass.

The separate `corpus::bytes` selector failed 1/1 during semantic compilation
of `bytes_equality_and_keys.ph` with `type.generic.underconstrained`. That
fixture contains no `is`/`is!` predicate; the direct `Bytes.fromList` test
passes. This is recorded as unclear/nonblocking for INC-C3-19, not as a
confirmed baseline and not as a reason to alter generic inference.

T6.7 is implemented and focused-tested. `compile_intrinsic_runtime_data`
selects the preinstalled runtime Tuple or Record class directly from
`facade.owner`; declaration identity locates and binds the semantic facade but
does not re-derive the runtime owner by module/name. The semantic
`runtime_structural_declarations_bind_canonical_platform_owners` test passed
1/1. Core selectors `intrinsic_tuple_facade_reuses_the_preinstalled_runtime_class`,
`intrinsic_record_facade_reuses_the_preinstalled_runtime_class`, and
`intrinsic_structural_data_without_facade_semantics_fails_closed` each passed
1/1. The absent-facade test confirms there is no name-based runtime-owner
guess.

T6.8 is implemented and focused-tested; no production change was needed. The
live semantic implementation already maps structural Tuple/TupleFromPacks and
Record targets to their canonical Universe behavior owners, preserves the
structural target head, enforces owner-module authorization, and keeps
RecordFields out of the proper-Type binding domain. The Record family test now
checks the actual structural `Record<**Fs>` covering substitution through
`GenericInstantiation`: it has a row binding, no type binding, and preserves
the mapped declaration parameter. A new collision discriminator submits
direct and spread spellings for both Tuple and Record with the same member and
observes the existing same-owner `ImplMemberConflict` diagnostic.

The seven focused semantic selectors each selected and passed 1/1:

- `tuple_structural_impls_use_canonical_owner_and_equivalent_spellings`
- `record_structural_impls_use_canonical_owner_and_equivalent_spellings`
- `foreign_modules_cannot_author_structural_tuple_or_record_impls`
- `row_parameter_covering_impl_materializes_through_record_row_identity`
- `product_parameter_covering_impl_materializes_through_complete_type_pack`
- `structural_impl_equivalent_spellings_produce_identical_targets`
- `equivalent_structural_impl_spellings_reach_the_same_member_collision_policy`

The T6.8 changes are test-only. No structural target variants, owner side
channel, parallel registry, or generalized open-row solver were added.

T7.1 is already implemented and focused-tested; no source change was needed.
`bootstrap_universe_declarations` maps source declarations for TypePack,
LabeledTypePack, CompleteTypePack, and RecordFields to the existing built-in
kinds. Focused semantic checks confirm the canonical facade owners and binder
resolution without nominal forms or nominal static domains; the core bootstrap
measurement confirms the anchors do not create runtime classes. Each selected
and passed 1/1:

- `intrinsic_structural_source_facades_bind_existing_authorities_without_nominal_forms`
- `complete_type_pack_kind_resolution_uses_its_canonical_facade_declaration`
- `boot_01_bootstrap_measurement_separates_catalog_closure_and_execution`

T7.2 is implemented and focused-tested with no edits. The Tuple, Record, and
pack-domain source declarations match the evaluator catalog, but evaluator
lookup is not a publication path: projection resolution still requires the
associated source product and matching intrinsic operation. The effective
spec defers concat/spread algebra, so no new general concat API is required.
The catalog selector uses synthesized checked products; a separate
source-backed Tuple::Entries publication test parses an intrinsic source
implementation, while the live Universe source declarations provide the
remaining authored surface. Three selectors each selected and passed 1/1:

- `intrinsic_type_member_source_products_match_the_evaluator_catalog`
- `canonical_tuple_intrinsic_impl_member_publishes_through_associated_surface`
- `intrinsic_type_member_source_site_targets_canonical_owner_and_selector`

T7.3 is implemented and focused-tested. The CompleteTypePack projection
normalizer uses shared positional and labeled pack-map helpers, so
`CompleteTypePack::Map` follows the same mapping authority as the individual
pack operations. Concrete Positionals/Labeled results normalize immediately;
Map preserves order, labels, and lane boundaries. Symbolic Positionals,
Labeled, and Map retain their exact source-backed residual projections. The
new discriminator and two existing source/map controls each selected and
passed 1/1:

- `complete_type_pack_projections_and_map_preserve_both_lanes`
- `test_tuple_and_record_map_projection`
- `source_visible_complete_pack_lane_projections_keep_canonical_member_declarations`

T7.4 is implemented and focused-tested with no production change. The
existing `Tuple::Element` evaluator unions every concrete positional and
labeled payload, yields `Never` for an empty Tuple, and keeps an unenumerable
`TupleFromPacks` result symbolic with its canonical source declaration. Both
focused selectors passed 1/1:

- `tuple_element_unions_payloads_and_preserves_unsolved_pack_projection`
- `tuple_entries_is_complete_pack_projection_for_concrete_and_symbolic_tuples`

T7.5 is implemented and focused-tested with no production change. The existing
`Record::Value` evaluator unions every payload in a closed row, yields `Never`
for an empty Record, and preserves the source-backed projection for an open
row instead of presenting a partial union as exact. The new
`record_value_unions_closed_fields_and_preserves_open_row_projection` test
selected and passed 1/1.

The producer correction, verification changes, and new tests remain
uncommitted on local `main`. No workspace build/test/clippy or release
certification was run. A workspace-wide `cargo fmt --all -- --check` attempt
reported formatting differences in numerous unchanged files across other
crates; a targeted `rustfmt --check` over all eight modified Rust files showed
only pre-existing formatting suggestions in unchanged sections of
`phalcom-semantic/tests/semantic/impls/targets.rs`. The newly modified lines
were clean, and no broad formatting rewrite was applied.

## T7.6 — Canonical source/evaluator catalog

Added `canonical_universe_intrinsic_type_members_match_evaluator_catalog` in
`phalcom-semantic/tests/semantic/incremental/type_members.rs`. It checks the
actual canonical Universe bootstrap products bidirectionally against
`IntrinsicTypeOperation::ALL`: unique source member per evaluator, canonical
owner/selector identity, parameter/result kinds, and no intrinsic source
product without an evaluator. The new selector and existing synthetic catalog
selector each passed 1/1. This was test-only. There is no separate
compiler-private operation catalog or source visibility field today; all 15
current evaluator variants are source-selectable by design.

## T7.7 — Family-specialized Tuple/Record behavior

No implementation change was needed. The live Universe source already places
Tuple instance behavior under `impl<***Es: CompleteTypePack> Tuple<***Es>` and
Record behavior under `impl<**Fs: RecordFields> Record<**Fs>`; typed member
signatures use the matching `Element` and `Value` projections. Six focused
nightly selectors passed 1/1:
`p4_shape_generic_tuple_specializations_share_behavior_and_keep_exact_descriptors`,
`p4_shape_generic_record_specializations_expose_generated_fields_without_fake_declarations`,
`p4_shape_generic_construction_through_generic_callable_keeps_behavior_class_identity`,
`p4_fixed_and_expanded_record_components_use_exact_specialization_slots`,
`tuple_iterator_preserves_specialized_element_type_through_both_protocols`,
and `record_get_returns_option_values`. The initial unqualified `--exact`
selector selected zero tests and is excluded as evidence.

## T7.8 — Exact element/value observers

Changed only the collection source declarations, their native primitives, and
one core language regression. `Tuple.rawAt` now returns
`Tuple<***Es>::Element`; `Record.rawValueAt` returns
`Record<**Fs>::Value`. Direct invalid internal reads report an internal
runtime error. Public Tuple `at`/`first`/`last` and indexed access preserve
their Option/throw behavior by checking or wrapping before the internal call;
negative indexing remains supported. `Record.get` wraps a present value and
still returns `None` for an absent key.

`tuple_and_record_raw_observers_keep_exact_types_and_public_options`,
`tuple_satisfies_sequence_contract`, and
`record_get_returns_option_values` each passed 1/1; the supervisor reran the
new regression on the pinned nightly. The new source case checks exact
`Option<Int>`/`Option<String>` projections, present payloads, empty-Tuple
first/last, negative Tuple index, out-of-range None, and missing Record-key
None. Scoped formatting and `git diff --check` passed.

## T7.9–T7.11 — Tuple lane, iterator, and runtime-shaped boundaries

T7.9 required no production change: the existing source signatures and VM
primitives already return structural Tuple products for `positionals` and
`labeled`. The new
`tuple_lane_observers_preserve_exact_structural_types_and_values` source test
checks exact `(Int, String)` and `(tag: Bool)` result types, values, labels,
and indexed access; it passed 1/1. The existing
`dynamic_tuple_preserves_lanes_and_normalizes_empty_expansion` selector also
passed 1/1.

T7.10 likewise required no production change: `TupleIterator<***Es>`, its
source Tuple, `next`, and both iterator protocol declarations already preserve
`Tuple<***Es>::Element`. The existing
`tuple_iterator_preserves_specialized_element_type_through_both_protocols`
regression now asserts that exact projection for `next` and the `for` item
rather than accepting a manually written `Int | String`; it passed 1/1 on the
pinned `nightly-2026-07-10` toolchain.

T7.11 confirmed that runtime-shaped Tuple construction (`rawFromList`), slicing
(`rawSlice`), and TupleZip items must remain behind their existing `Dynamic`
boundaries: runtime List contents determine constructed shape, runtime bounds
determine slice arity/labels, and runtime lane count determines each zipped
Tuple's shape. The source-visible generic iterator and ordinary element
observers remain typed through `Tuple<***Es>::Element`. The audit also found
that `TupleZipIterator.new` still called optional `at(_)` as if it returned a
raw element. It now uses bounds-checked `tuple[i]` under its `i < tuple.size`
loop, without changing zip typing. The new
`runtime_shaped_tuple_slice_and_zip_stay_behind_dynamic_boundary` regression
passed 1/1 on supervisor rerun. No existential or variadic zip arithmetic was
introduced. T7.11 is IMPLEMENTED / FOCUSED_TESTED.

## T8.1 — Bare braces select Record syntax

No parser production change was needed: the live parser already maps bare
braces directly to Record literal, type, and pattern forms, independently of
entry contents. The new
`bare_braces_are_records_across_expression_type_and_pattern_positions`
AST regression covers empty/populated Record expressions, a structural Record
type alias and pattern, explicit Map expression/pattern heads, and distinct
nominal `Person { ... }` construction. The pinned parser selector passed 1/1
on supervisor rerun. Match-arm block/expression grammar was not changed.

## T8.2 — `#{...}` is Set syntax

No lexer or parser routing change was needed: the lexer already emits
`SetLBrace` and the parser already routes that token to `Expr::SetLiteral`.
Only a stale `parse_set_literal` comment describing `Map` as the opener was
corrected. The lexer test distinguishes `#{}` from bare `{}`; the parser test
covers empty, singleton, and populated Set literals, bare braces remaining a
Record, and spread remaining represented as `SetLiteralEntry::Expansion`.
Set spread compilation remains rejected by the existing diagnostic path. Both
new selectors passed 1/1; the supervisor reran the parser discriminator.

## T8.3 — Explicit Map head identity

`MapLiteralExpr` now retains the parsed explicit head, including its source
range, instead of discarding it after using the syntax to select Map-entry
grammar. Semantic checking resolves that head through the existing type
resolver and requires identity with the canonical Universe `Map` declaration.
Same-spelled local value and class-declaration shadows each produce a
deterministic diagnostic and unknown expression knowledge; neither silently
constructs the builtin Map. This reuses canonical declaration identity rather
than adding a string-based registry or compiler special case. The AST source-
identity and semantic canonical-head selectors passed 1/1 each on supervisor
rerun. T8.3 is IMPLEMENTED / FOCUSED_TESTED.

## T8.4 — Preserve Map entry semantics

No production change was needed. The new semantic discriminator verifies
explicit `Map<K,V>` specialization, inference from computed and bare-symbol
keys, `**` Map expansion, context-guided inference, and an uncontextualized
empty Map remaining `Unknown(NoTypeEvidence)` rather than defaulting to
`Dynamic`. A focused core runtime test verifies that a successful expansion
preserves entries and that a literal key colliding with an expanded key is
rejected. Both selectors passed 1/1 on supervisor rerun. The broader
`collections` and `collections_literals_negative` selectors fail before
reaching Map cases in unrelated iterator/negative-fixture paths; those results
are not treated as T8.4 acceptance evidence or classified as baselines here.
T8.4 is IMPLEMENTED / FOCUSED_TESTED; T8.5 resumes under the adopted ruling
below.

## T8.5 — Adopted Record spread source-domain ruling

The architect's ruling supplied on 2026-09-28 resolves the hold. Record
`**` is shape-preserving: require the source to project statically to
`RecordFields`. Record rows and labeled Tuple lanes are valid; mixed Tuple
positional elements do not participate, and `LabeledTypePack` remains distinct
from `RecordFields`. Symbolic/open rows remain valid when represented
faithfully. Map-to-Record is invalid even for a Map literal and must produce a
direct semantic diagnostic because `Map<K,V>` does not encode its exact key
set. Admission must be type/shape-based, not expression-provenance-based.

Map-to-Map behavior remains unchanged; Record and labeled Tuple sources may
also contribute labeled entries to Map. Do not narrow the shared runtime
labeled-lane primitive, fabricate a row, use `Dynamic`, or introduce a second
row solver. The legacy PASS fixture
`phalcom-core/tests/fixtures/language/collections/labeled_spread_record_map.ph`
must be split/migrated: preserve positive labeled-Tuple-to-Record and
Map-to-Map coverage, and add direct Map-to-Record rejection coverage for both
a binding and literal. T8.5 is now IMPLEMENTED / FOCUSED_TESTED under this
boundary. `synthesize_record_literal` rejects canonical Map-origin sources
before Record-shape projection and emits the same direct diagnostic for a Map
binding or Map literal. Record rows (including symbolic tails) and checked
labeled-Tuple projections continue through the existing canonical
`project_record_shape` path; the runtime `snapshot_labeled_lane` helper is
unchanged.

Focused evidence: semantic `record_spread` selected/passed 3/3; the exact
`mixed_labeled_tuple_spread_projects_only_its_labeled_lane_to_record` selector
passed 1/1. Core `static_labeled_sources_spread_into_record_and_collections`
passed 1/1 for Record/Tuple-to-Record and Record/Tuple/Map-to-Map runtime
behavior. The migrated fixture
`labeled_spread_record_map.ph` was run directly through `./target/debug/phalcom`
and matched its updated expected output. The existing Map expansion test
passed 1/1, retaining Map-to-Map and duplicate-key behavior. The aggregate
`collections` lane fails before this fixture when
`boundedness_take_bounds_unbounded.ph` reaches the inherited
`TakeIterator.toList` runtime error; `collections_literals_negative` fails in
the unrelated `map_key_eq_mutates_same_map_during_overwrite_rejected.ph`
fixture with `type.generic.underconstrained`. These aggregate failures are not
claimed as T8.5 evidence or classified as baselines.

## INC-C3-19 — Supplemental classifier discriminators

The added semantic test
`semantic::capabilities::flow_branches::bare_generic_type_tests_preserve_applied_facts_through_branch_and_loop_joins`
checks canonical-Universe-backed `input is List` and `input is not List`
expressions produce Bool knowledge with no error diagnostics, and that
`List<Int>` knowledge remains intact through branch and loop joins. The
producer discriminator
`semantic::foundations::flow_graph::type_test_refinements_require_supported_proper_targets_and_respect_exactness`
checks both truth edges for both source spellings create no improper predicate
and preserve the existing `List<Int>` fact; it also verifies false exact-class
`is!` preserves a subclass fact. The transfer guard discriminator
`semantic::foundations::predicate_transfer::non_proper_type_test_targets_do_not_change_or_publish_flow_facts`
checks improper targets neither change knowledge nor publish facts. All three
selectors passed 1/1 on supervisor rerun.

The source expressions have `Known(Bool, Assumed)` knowledge but
`Blocked(UnknownType(NoTypeEvidence))` analysis status because canonical
`Object.is` intentionally leaves its classifier parameter unannotated. They
have clean causal invalidity and no error diagnostics; this is missing static
argument evidence, not an invalid Bool result or a new refinement. Establishing
`Ready` would require changing/reifying the `Object.is` classifier argument,
outside this ruling. The existing core
`isa_is_reflexive_and_superclass_closed` runtime Bool/subclass control passed
1/1 on supervisor rerun.

## T13 — Certification outcome and residual disposition

### Focused gates

The search-based T13.1 census corrected the effective iterator spec: generic
Tuple iteration yields `Tuple<***Es>::Element`; only `rawFromList`, `rawSlice`,
and TupleZip remain documented runtime-shaped Dynamic boundaries. The G1–G4
focused gates passed as recorded in the checkpoint. G5 focused system,
indexing, source census, and Universe-bootstrap selectors passed; the two
required collection aggregates remain encountered failures before migrated
fixtures (`TakeIterator.toList` and the map-mutation fixture's
`type.generic.underconstrained`) and are not baseline-certified.

### INC-C3-20 ruling amendment and focused matrix audit

Outcome B is scoped to unannotated trait-default return results: they are not
published as a canonical return-signature contract consumable by trait
dispatch. Ordinary class callable inferred-return publication exists;
`Iterator.take` is separately blocked by `RecursiveFixpoint`. T13 does not add
trait-default return inference/publication. The natural
`(0..).iter.take(10).toList` selector is retained as ignored/blocked, not
acceptance evidence. The ruling amends only the actual-expression gate to a
canonically known receiver; P4/C3 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`,
and Priority 3 is unchanged.

The ruling's focused matrix is audited against the incident, checkpoint, live
tests, and lowering source without rerunning selectors:

| Matrix item | Recorded evidence / disposition |
|---|---|
| Exact trait default | `trait_dispatch_query_selects_an_exact_trait_default` — 1/1 PASS; asserts Exact evidence and `TraitDefault`. |
| Concrete guarded Derived | `trait_dispatch_query_accepts_guarded_derived_evidence_without_broadening_ineligible_receivers` — 1/1 PASS; `GuardedRule` and nested Exact proof. The known source receiver is additionally covered by `known_list_iterator_source_receiver_gets_guarded_iterable_to_list_evidence` — 1/1 PASS. |
| Symbolic constrained Derived | `symbolic_assumed_iterator_dispatches_guarded_derived_iterable_default` — 1/1 PASS; `Caller<I> where I |= Iterator`, Derived proof with nested Assumed evidence. |
| Ineligible known receiver | `known_non_iterator_source_receiver_does_not_get_derived_iterable_dispatch` — 1/1 PASS; known Int receives no Derived `Iterable::toList`. The semantic concrete test also checks an unsatisfied receiver and an unrelated same-selector trait. |
| Coherence conflict | `trait_dispatch_does_not_hide_canonical_coherence_conflict_behind_another_selector_candidate` — 1/1 PASS; canonical exact-plus-derived Iterable conflict remains visible beside another same-selector trait. |
| Actual known-receiver expression | `known_list_iterator_source_receiver_gets_guarded_iterable_to_list_evidence` — 1/1 PASS; source `[1, 2].iterator` supplies a known ListIterator for `receiver.toList`. |
| Proof/dependency capture | Same known-receiver selector asserts canonical Iterable owner/selector, `TraitDispatchSite::Evidenced`, guarded rule/source identity, exact subject/ref, nested Exact Iterator proof, and guarded/nested conformance plus trait-surface dependencies. |
| Exact lowering projection without proof search | `ordinary_trait_lowering_preserves_exact_and_derived_dispatch_evidence` — 1/1 PASS; compiles the analyzed source and compares the complete Exact semantic selection to the entry module's `TraitInvocationSpec` at the same source range. `semantic_lowering.rs:1338` clones the already selected `Evidenced` product; `:1523` projects Exact iteration evidence. Existing `trait_evidenced_default_and_witness_execute_without_target_installation` also exercises an exact default at runtime. |
| Derived lowering projection without proof search | Same new selector — 1/1 PASS; compares the complete GuardedRule Derived selection, including nested Exact evidence, to the lowered spec at the `receiver.toList` range. Ordinary lowering clones the selected product at `semantic_lowering.rs:1338`; the Exact/Derived iteration paths at `:1523`/`:1552` likewise project supplied evidence and do not call a resolver. |
| `cargo check -p phalcom-semantic` | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic` — PASS; one existing `apply_type_form_with_arena` dead-code warning. |
| `cargo check -p phalcom-core` | `env RUSTFLAGS= RUSTC_WRAPPER= cargo check -p phalcom-core` — PASS; same existing dead-code warning. |

The §13 focused matrix is satisfied and supervisor-reviewed; stop at the
architect ruling §15 boundary. G5 collection aggregates remain
unpassed/uncertified, G6 remains incomplete, and the natural chain remains
blocked/ignored. P4/C3 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`; this focused
matrix does not mark T13 or P4 complete. No further broad, corpus, or workspace
rerun, and no trait-default return inference, is authorized under this ruling.
Crossing the boundary requires new user or architect authorization.

### Broad gates attempted

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets` passed
  with the recorded existing dead-code warning.
- The first `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace` attempt
  stopped at the AST integration target with three stale test-oracle failures;
  the focused AST oracle and snapshot corrections are committed. One authorized
  full retry progressed to `phalcom-core --lib` and reported 157 tests, 155
  passed, 2 failed. The process session was lost before a final Cargo exit
  summary; no later workspace target results or workspace-wide total are
  claimed. Exact serial selectors confirmed the two failures:
  `universe::tests::kernel_option_match_override_reroutes_every_combinator`
  (1 failed), already recorded as the LANG005.C4 internal generic call-entry
  failure; and
  `vm::send::tests::captured_setter_rejects_an_ordinary_positional_rhs_lane`
  (1 failed). The latter had an earlier LANG004 R1 PASS before c386; c386
  changed captured-method shape validation. No P4/T13 commit changes that test
  or `vm/send.rs`, so it is outside P4 causality but not a verified baseline.
- The single `RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace
  --all-targets -- -D warnings` attempt exited 101. It reported the
  `match_like_matches_macro` lint on the pre-P4 `starts_expression` boolean
  match (P4 only added token alternatives) and pre-P4 type-meta lints
  `result_large_err`, `manual_range_patterns`, and `op_ref`. No unrelated lint
  fixes were made.
- The single `cargo fmt --all -- --check` attempt exited 1 with extensive
  workspace formatting drift (5,263 output lines). Six identified P4-authored
  Rust hunk blocks received only localized rustfmt-equivalent edits. Targeted
  `rustfmt --edition 2024 --check` over
  `phalcom-ast/src/parser.rs`, `phalcom-core/src/compiler/lib/patterns.rs`,
  `phalcom-core/src/compiler/lib/scope.rs`,
  `phalcom-core/src/modules/compile.rs`,
  `phalcom-core/src/modules/semantic_lowering.rs`, and
  `phalcom-core/tests/core/language/data_e2e.rs` then passed. The workspace
  format gate remains unpassed; no broad formatting was applied.
- Global `git diff --check` exited 2 only on six trailing-whitespace lines in
  the preserved user edit `docs/__unstructured__/Untitled 1.md`; it was not
  changed. `docs/.obsidian/workspace.json` was also preserved.

### Final state

G6 is incomplete: the workspace-test retry has unknown scope after the core
library target, clippy failed, the workspace format gate failed, and the global
diff check is blocked by the preserved user note. No architectural decision is
currently requested by this evidence; no unrelated core fix or broad formatting
change was attempted. P4 and C3 remain `IN_PROGRESS / PARTIAL / UNVERIFIED`,
not `RELEASE_COMPLETE`. Do not repeat the workspace test or alter the two core
failure paths without a cause-backed instruction. The checkpoint remains the
authoritative status record; the current local changes were not pushed.
