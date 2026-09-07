# Part 06 — Semantic Authority Audit

Audit date: 2026-08-29. Scope: production Rust under `phalcom-semantic/src`.

## Decision

Every formal `Established` construction has a compiler-owned proof source. Source annotations and contextual contracts are `Assumed`; relation success validates a fact and never relabels it. Test-only construction sites inside `#[cfg(test)]` modules are excluded from this production inventory.

## Approved proof-construction inventory

| Sites | Authority category | Constraint / regression anchor |
| --- | --- | --- |
| `types/evidence.rs:98,175-178,242-245` | Authority-preserving composition | `EvidenceStatus::meet` selects weakest premise; `required_composition_*` tests. |
| `checker/typed_expr.rs:75` | Constructor helper only | Caller must provide an approved origin; audit rows below classify all production callers. |
| `checker/expression.rs:190,198,206,214,678-680,821` | Canonical literal/container syntax | Uses `CoreDeclarationIds` plus registered `core_type`; `authority_boundaries::user_object_name_is_not_universal_supertype`. |
| `checker/expression.rs:238-284,1371,1434-1439` | Canonical declaration semantics | Resolved `DeclarationId` / declaration table, never spelling-based builtin lookup. |
| `checker/expression.rs:376,390,400,465,475,479,521,540,593,628,1249,1284,1864` | Canonical control / assignment / closure semantics | Unit, Never, or callable shape is intrinsic to the executed construct; control-region and loop suites. |
| `checker/expression.rs:663` | Deliberately broad Range result | Canonical registered `Object` only; missing core form is Unknown. Precise range semantics remains outside Part 06. |
| `checker/body.rs:231,261`; `checker/control.rs:87,94,223,289,314,319`; `checker/declaration.rs:250`; `checker/statement.rs:159` | Executable-region and callable completion semantics | Reachable empty/tail completion yields Unit; all-normal-absent join yields Never. `control_regions` regressions. |
| `checker/declaration_signature.rs:111,274,293`; `declaration_type.rs:97-102`; `types/native.rs:166,190,203,322,324` | Constructor, declaration, and trusted native semantics | Native surface import and constructor conformance tests. Source annotation branch in `declaration_type.rs:96` stays Assumed. |
| `checker/call.rs:359,403,457,613-668,678,717,955,991,1106,1170-1209,1377` | Exact dispatch, structural builtin, and established generic inference | Exact-return authority meets receiver/premise/return authority; structural targets are compiler-owned; `authority`, `canonical_call_application`, and generic-support tests. |
| `checker/context.rs:756,1761,1764,1869` | Trusted predicate / certified return / canonical control declaration | Flow predicate authority gate and callable return certification prevent relation-only upgrading. |
| `checker/flow/transfer.rs:66,86,101,122,139,160,169` | Trusted runtime/type-test observation | `PredicateAuthority::AuthoritativeObservation` required where an assumption becomes established; `predicate_transfer` suite. |
| `checker/field_lifecycle.rs:146` | Validated field lifecycle | Requires validated lifecycle plus clean causal path; field lifecycle suite. |
| `signature.rs:121` | Validated callable publication | Only source declaration with `Satisfied(Established)` is republished as Established. |
| `checker/expected.rs:163,167-168` | Context propagation | `ExplicitCheck` may preserve established authority; source declaration/return/assignment contexts are Assumed. `authority_boundaries::contextual_empty_*`. |
| `checker/analysis.rs:255` | Empty-exit bottom semantics | No normal exit denotes Never; invalid normal facts are quarantined. |
| `checker/flow/state.rs`, `checker/loop_analysis.rs`, `checker/composition.rs`, `checker/call.rs:1433`, `checker/context.rs:2000`, `db/fingerprint.rs:1576`, `diagnostic_presentation.rs:342,494`, `types/relation.rs:663,676`, `types/evidence.rs:452-516`, `signature.rs:293-313` | Test or presentation-only | No production formal fact is created from these occurrences. |

## Relation-success consumer audit

| Area | Use of `Proven` / subtype | Non-strengthening result |
| --- | --- | --- |
| `checker/binding.rs`, `checker/context.rs`, `checker/statement.rs` | Contract compatibility | Retains `actual.clone()`; only consistency becomes Validated or Assumed. |
| `checker/field_lifecycle.rs` | Field-write compatibility | Validity follows actual evidence status; a relation cannot establish an assumed write. |
| `checker/flow/transfer.rs` | Trusted predicate transfer and contradiction | Established refinements require authoritative predicate observation; retained assumed facts remain Assumed. |
| `checker/call.rs`, `checker/inference.rs` | Dispatch applicability / generic constraints | Result authority is a meet of target, receiver, premise, and generic support. |
| `checker/expression.rs`, `checker/control.rs`, `checker/loop_analysis.rs` | Expression checking and flow joins | Relations decide compatibility/reachability; result synthesis preserves child authority or returns Unknown. |
| `types/relation.rs` | Relation solver | Returns `RelationOutcome` only; does not construct new evidence. |

Direct regression: `authority_boundaries::proven_relation_does_not_upgrade_assumed_actual`.

## Canonical identity and TypeStore boundaries

- `core_surface/identity.rs` owns exact `ModuleId::core()` declaration identities.
- `checker/context.rs::core_type` reads only a registered canonical declaration form; it does not fall back to source name resolution or synthesize a missing core form.
- `checker/flow/predicate.rs` trusts type tests, equality, and ordering only when their callable owner equals one of those canonical identities; `checker/call.rs` applies the same identity check to transitional List/Map indexer targets.
- `types/relation.rs` materializes generic supertypes with the active mutable `TypeStore`; clone materialization is absent from the relation/checker paths.
- Regressions: `authority_boundaries::{user_object_name_is_not_universal_supertype,user_function_name_is_not_callable_supertype,generic_supertype_specialization_materializes_in_live_store}`.

## Intentional `Unknown(UncheckedExpression)` boundaries

| Site family | Classification |
| --- | --- |
| `expression.rs` membership / is-membership | No ratified compiler-owned membership operation. Fail closed; no Bool proof. |
| `expression.rs` `Matches` / `Understands` chain links | No Part-06 operation contract; link makes full chain Unknown. |
| `expression.rs` ellipsis and malformed structural contribution recovery | Unsupported/recovery syntax boundary. |
| `call.rs` unavailable iteration argument | Missing protocol evidence, not Dynamic or Established. |

The expression audit also reviewed every child-analysis followed by a known result. Intrinsic syntax and canonical control/composition sites are listed above. Dispatch-dependent comparison chains now route through resolved binary applications; membership is intentionally Unknown.

## Verification anchors

- `authority_boundaries` covers canonical shadowing, live-store specialization, relation non-strengthening, comparison links, membership failure, and contextual empty evidence.
- Incremental fingerprint/type-store suites cover status-sensitive products, retained snapshots, one-session TypeStore identity, and declaration-template invalidation.
- Presentation/advisory suites verify formal Unknown and authority status are projected without advisory repair.
