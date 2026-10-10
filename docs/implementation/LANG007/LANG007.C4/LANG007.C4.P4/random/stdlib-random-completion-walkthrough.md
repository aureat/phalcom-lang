# `universe.random` implementation walkthrough at delivery stop

Status: IN_PROGRESS. Completion: PARTIAL. Verification: FOCUSED_TESTED.
Branch: `main`. Starting HEAD: `800474d8`; stable owner prerequisite: `fbf057ea`.
User requested delivery after the focused negative/resource gate. No subagents
were used. The delivery commit contains this record and the implementation;
retrieve its revision with `git log --oneline --grep='random'`.

## Delivered architecture

`universe.random` is registered as a package with private child modules for
`Generator`, `Xoshiro256PlusPlus`, `Distribution`, `UniformInt`, `Bernoulli`,
and `SecureRandom`. The public facade exports the typed deterministic API and
secure capability. Deterministic behavior is ordinary source code. Xoshiro
state is explicit mutable class state; `nextU64` is the only inherent witness
operation and the trait supplies defaults for booleans, floats, arbitrary-size
unbiased integers, bytes, choice, and Fisher–Yates shuffle.

The stream uses standard SplitMix64 increment-before-mix seeding and standard
xoshiro256++ transition. The exact choices are: high bit for bool; top 53 bits
over 2^53 for float; earlier words as high chunks with excess high bits masked;
big-endian byte extraction; no reservoirs; descending shuffle; zero consumption
for bound one and Bernoulli endpoints.

`UniformInt` and `Bernoulli` are validated immutable descriptors with associated
`Output` types. `SecureRandom` is separate and does not implement `Generator`.
Its only native operation fills an existing `Bytes` buffer using `getrandom`
0.3.3; source code owns allocation and raises `EntropyError` on host failure.
The native implementation releases its mutable heap borrow before allocating an
error String. Effects are mutation + nondeterminism + blocking.

## General prerequisite repairs

Before random completion, stable callable metadata was upgraded to preserve
Declaration/Variant/Conformance owners with schema v11 and v10 legacy decode.
Method-generic trait evidence now translates requirement binders to selected
witness binders and carries the existing call-entry recipe through selected
trait lowering and VM activation. Trait slots are local to their owning surface.
Qualified module class exports use canonical type-name resolution. These are
general semantic/runtime seams; no random-specific opcode or dispatch branch was
added.

## Acceptance evidence

- Original syntax probe and deterministic random corpus: PASS, 2 selected.
- Core random facade/evidence, secure runtime, negative typing/topology and
  numeric-policy tests: PASS, 4 selected (final gate).
- Core trait suite: PASS, 18 selected.
- Core secure native unit tests: PASS, 2 selected.
- Semantic source Universe constants/trait-default publication: PASS.
- Semantic selected-witness binder/proof and cold/incremental parity regressions:
  PASS.
- `cargo check -p phalcom-modules`, `-p phalcom-native-meta`,
  `-p phalcom-native-surface`, and `-p phalcom-core`: PASS.
- Native surface generator `--check`: PASS, 400 declarations.
- `git diff --check`: PASS.

The full semantic trait lane remains 41 PASS / 4 baseline failures. The four
existing failures are recorded in the incident and were reproduced with the
random/recipe changes isolated; they are outside random acceptance.

Workspace release certification, broad core tests, performance benchmarking,
dedicated effect/native-source identity assertions, remaining byte resource
cases, and cross-platform execution were not run. The focused injected native
failure test executes the public `bytes` API and catches source `EntropyError`;
the native zero/nonempty success tests also pass. Full workspace formatting
reports baseline drift; only scoped Rust files were formatted.

## Last completed step and remaining acceptance

The final core random lane initially failed because its error-text expectations
used the wrong case and hidden-import wording. The assertions were corrected to
the actual diagnostics (`GenericConstraintUnsatisfied`, `ModuleExposureRejected`)
without changing rejection behavior. The final run passed 4/4, including the
70-bit candidate assembly and oversized-bound numeric failure.

Next: record the required throughput/allocation/GC measurements; complete the
metadata/effect assertions and byte-resource tests; run directly affected provider,
native verification, numeric and full corpus gates; decide/run broad core gate;
then promote plan completion only from completed evidence. The source slice is
implemented, but the plan remains PARTIAL at the user-directed stop.

The retained corpus session separately exposed an existing update failure:
`FieldSignature(Xoshiro256PlusPlus._s0)` was Ready but not validated for the new
revision, and last-known-good fallback returned stale fixture source. The syntax
probe now has its own lane. This general dependency-validation issue remains
recorded, not repaired by the fixture isolation.

## Completion continuation (2026-10-08)

The follow-on focused work added public security/replay guidance, a forced-GC
deterministic sequence check, a secure byte-count probe, canonical native
source/effect assertions, provider registration coverage, and manual source
benchmark programs. The focused random corpus passed 2/2, numeric-policy
sampling passed 1/1, native-owner metadata passed 1/1, byte-count rejection
passed 1/1, provider registration passed 1/1, and native-surface `--check`
passed with 400 declarations. Scoped formatting and diff hygiene passed.

The secure byte-count probe rejects counts outside the host index range. It
does not prove that very large representable counts are safe: the owning
`Bytes.new` constructor uses infallible vector allocation without a resource
limit. This is recorded as an R21 owner-layer premise failure. R11's
deterministic mutation-effect assertion cannot be obtained from the current
general source effect pipeline, which neither derives effects from writes nor
publishes the available interprocedural result. The user amended R11 to defer
that presentation until the general capability exists; no random-specific
effect metadata was added. R21 still needs a user decision before changing the
general byte resource contract.

The release benchmark executable builds but fails during canonical Universe
bootstrap before loading the benchmark source, with the existing associated
family invocation lowering error. A debug fixture run succeeds, but does not
count as performance evidence. The full corpus attempt was interrupted after
unrelated categories failed, so it remains incomplete. No timings, allocation
counts, broad corpus result, or cross-platform result are claimed. Random plan
status remains IN_PROGRESS / PARTIAL / FOCUSED_TESTED.

## Deferred by plan

Jump/longJump, stream splitting, parallel reproducibility, complex distributions,
weighted/reservoir sampling, state serialization, secure token/UUID helpers,
fixed-width optimization, and mutation-capability integration remain deferred.

## Architect blocker repair continuation (2026-10-08)

The user renewed general fix authority and the architect role, inline on main
without subagents. The plan and owning VM specification now define a general
Bytes constructor resource contract: configurable 64 MiB default, separate
from numeric limits, checked before fallible zero-filled backing reservation.
Focused direct and public secure-caller checks pass without giant allocations.
R11's source-effect deferral remains unchanged.

Release-only associated-invocation failure was caused by expression-owner stack
mutation inside a debug assertion. The pop now executes in every profile; the
original syntax fixture passes in the optimized CLI. A nested-owner regression
was added. The optimized build took 3m24s and the syntax process completed in
13.17s, with 619,413,504 bytes peak RSS.

Retained source replacement reproduced the stale snapshot. A fallible semantic
publication seam now protects executable consumers; exact session repairs retain
member prerequisites, effective definitions, constructor signature identity, and
witness body/access facts. The initial retained/cold-parity regression passes.
The affected incremental suite completed 208/36/4; all 33 smaller failures
reproduced on the predecessor, while three large reuse cases are still being
compared. Expanded fixture and focused runtime gates remain pending.

The first million-word benchmark was interrupted after 225.51 seconds without
completion. Sampling found 859/871 main-thread samples in idle OS polling.
The general reactor can skip OS polling with no active source while continuing
to drain worker completion ingress. Active behavior and executor waits retain
their contracts. Final reactor checks and renewed release measurements follow.

### Completed incremental gate failure inventory

Command: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::`.
The completed result is 208 PASS / 36 FAIL / 4 ignored, 190.68 seconds.
The predecessor comparison temporarily replaced only this task's session source
with HEAD, ran the 33 smaller exact failures, and restored the task source
byte-for-byte in a finally block: all 33 reproduced. Three large cases run
against a copied predecessor test executable; their result is recorded below.
No assertion or fixture was weakened. Exact failure identities:

- `semantic::incremental::adts::adt_incr_10_unrelated_method_edit_reuses_match_analysis`
- `semantic::incremental::associated_types::cross_module_associated_binding_replacement_recomputes_consumer_and_matches_cold`
- `semantic::incremental::callable_dependencies::case_a_unchanged_caller_changed_callee_signature_recomputes`
- `semantic::incremental::callable_dependencies::case_b_callee_body_change_signature_unchanged_reuses_caller`
- `semantic::incremental::callable_dependencies::case_c_field_annotation_change_recomputes_reader`
- `semantic::incremental::callable_dependencies::case_d_superclass_change_recomputes_inherited_call`
- `semantic::incremental::callable_dependencies::case_f_unrelated_edit_reuses_unaffected_callables`
- `semantic::incremental::callable_dependencies::case_j_generic_supertype_template_edit_invalidates_specialized_caller_and_matches_cold_rebuild`
- `semantic::incremental::callable_dependencies::dependency_edit_remove_readd_recomputes_affected_summary_deterministically`
- `semantic::incremental::callable_dependencies::direct_result_cycle_diagnostics_track_incremental_edges_and_match_cold_analysis`
- `semantic::incremental::callable_dependencies::inferred_return_refresh_preserves_stable_products_and_counts_final_disposition_once`
- `semantic::incremental::derived_conformance::trait_contract_edit_invalidates_derived_evidence`
- `semantic::incremental::generic_contracts::constructor_generic_bound_edit_invalidates_dependent_call_and_matches_cold`
- `semantic::incremental::product_stability::binding_flow_edit_rechecks_only_the_affected_callable`
- `semantic::incremental::product_stability::body_only_edit_stops_at_stable_semantic_products`
- `semantic::incremental::product_stability::diagnostic_aggregate_matches_cold_after_repair_and_unrelated_edit`
- `semantic::incremental::product_stability::field_mutability_edit_invalidates_field_and_write_products`
- `semantic::incremental::product_stability::inferred_cross_module_result_updates_dependents_and_matches_cold_analysis`
- `semantic::incremental::product_stability::pa8_recomputed_stable_intermediate_allows_downstream_reuse`
- `semantic::incremental::product_stability::previously_absent_name_addition_recomputes_exact_consumer`
- `semantic::incremental::product_stability::range_only_body_edit_reuses_semantic_callers`
- `semantic::incremental::product_stability::signature_edit_recomputes_exact_callers_and_reuses_unrelated_bodies`
- `semantic::incremental::product_stability::stable_intermediate_callable_product_stops_later_propagation`
- `semantic::incremental::product_stability::superclass_edit_recomputes_hierarchy_consumers_without_touching_unrelated_bodies`
- `semantic::incremental::product_stability::unrelated_edit_retains_unaffected_module_diagnostics`
- `semantic::incremental::recomputation_metrics::a6_cross_module_alias_relowers_when_provider_shell_changes`
- `semantic::incremental::recomputation_metrics::a6_high_fanout_unrelated_declaration_and_hierarchy_retain_existing_products`
- `semantic::incremental::recomputation_metrics::a6_high_fanout_unused_provider_body_edit_has_no_external_semantic_recompute`
- `semantic::incremental::recomputation_metrics::pa9_large_hierarchy_edit_isolates_unrelated_formal_products`
- `semantic::incremental::recomputation_metrics::unused_public_export_does_not_recompute_unrelated_consumer_body`
- `semantic::incremental::record_rows::row_body_only_edit_preserves_signature_semantics`
- `semantic::incremental::record_rows::row_prefix_edit_invalidates_dependent_call`
- `semantic::incremental::type_members::associated_static_case_alias_uses_domain_catalog`
- `semantic::incremental::type_members::static_case_rename_invalidates_alias_and_matches_cold_analysis`
- `semantic::incremental::type_store_revisions::publication_effects_distinguish_initial_graph_build_from_body_edit`
- `semantic::incremental::type_store_revisions::stable_type_store_id_and_snapshots_across_revisions`

## Architect delivery checkpoint (2026-10-08)

Status remains IN_PROGRESS / PARTIAL / FOCUSED_TESTED. R11 source-effect deferral
stands. R21 is resolved through a general configurable 64 MiB Bytes.new limit
and fallible backing reservation, independent of numeric budgets. Owner and
public secure-caller regressions pass. Release expression-owner stack repair
passed optimized syntax startup. Reactor idle-poll repair passed all 16 reactor
tests. Final core-lib selection: 20 PASS / 1 FAIL / 147 filtered, 28.74 seconds.
Retained publication repair remains PARTIAL: expanded actual-fixture replacement
fails at `phalcom-semantic/src/session.rs:6169`, where an accepted inherent
callable lacks a canonical signature. Preserve the failing regression and the
invariant. The earlier simple four-step retained sequence passed.

Completed incremental lane: 208 PASS / 36 FAIL / 4 ignored. All 36 failures
reproduce on predecessor behavior: 33 smaller exact tests plus three large reuse
cases (0 PASS / 3 FAIL, 205.16 seconds). These are Category D baseline failures.
Exact identities are retained in the walkthrough. No assertions were weakened.

Remaining tasks: finish canonical retained inherent-signature publication, rerun
exact regression and affected acceptance gates, rebuild final release CLI,
complete R20 throughput/allocation/GC evidence, full corpus classification, broad
core and cross-platform gates. The million-word benchmark was interrupted at
225.51 seconds before reactor repair; no workload throughput is claimed.

The user requested lifecycle updates, commit/push of everything, and an inline
handoff at this checkpoint. Existing COLL006, slotmap, and math-document
relocations are included under that instruction without additional behavioral
verification. Delivery does not promote completion or verification status.

Delivery hygiene: scoped Rust formatting and `git diff --check` PASS. The nested
expression-owner unit regression PASS 1/1. No broad test PASS is claimed.
