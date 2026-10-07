# Whole-Lane Rest Parameter Inference & Parameter Contracts Walkthrough

**ID:** LANG007.C4.P4.R1-TEL-1-whole-lane  
**Kind:** walkthrough-record  
**Status:** COMPLETE  
**Completion:** IMPLEMENTED  
**Verification:** FOCUSED_TESTED  
**Recorded:** 2026-10-07  

---

## Audit remediation — 2026-10-07

The post-implementation audit remediation ran inline on local `main` without a
branch or worktree. Structured generic/concrete lanes now use topology plus
pointwise relations; unavailable lane evidence is never converted to `Unit`;
invalid declaration recovery remains explicit; and callable compatibility
consumers specialize and compare `ParameterContract`.

Fresh focused evidence: `cargo check -p phalcom-semantic` passed,
`capabilities::rest_contracts` passed 20 tests, and `adts::requirements` passed
3 tests. The broader audit obligations remain open, so this walkthrough does
not promote R1/P4 to final completion.

## 1. Summary of Changes

Implemented whole-lane rest parameter annotations (`*args: *P`, `**args: **P`, `***args: ***P`) and canonical parameter contracts in Telos per `packs-correction.md`. This enables exact generic rest-call inference for `GoalQuery.of` while preserving `*args: T` as an element contract and eliminating TypePack/Pack conflation.

### Core Architecture & Implementation Points:
1. **AST Layer (`phalcom-ast`):**
   - Introduced `ParameterAnnotationSyntax` (`Type` vs `RestLane(TypePackExpansionSyntax)`) on `ParameterDef`.
   - Updated contextual parsing in `parser.rs` after `:` in parameter lists to parse `*P`, `**P`, `***P` into `ParameterAnnotationSyntax::RestLane`. General type annotation positions continue to reject bare lane expansions.
   - Preserved `type_annotation()` and `rest_lane_annotation()` accessor helpers on `ParameterDef`.
2. **Semantic Signatures (`phalcom-semantic`):**
   - Introduced canonical `ParameterContract` (`Fixed` vs `Rest { mode, contract: Unconstrained | Element | Lane }`) and `RestValueContract`.
   - Synchronized `CallableParameterSemantic` and `CallableParameter` with authoritative contracts.
   - Canonical `derive_body_type(&mut TypeStore)` derives Pack types (`Pack<*: T>`, `PackFromPacks(P, [])`) for local method-body bindings.
3. **Declaration Lowering & Projection:**
   - Lowered parameter contracts in `declaration_signature.rs:parameter_fact` reusing `store.project_positionals`, `store.project_labeled`, `store.project_complete`.
   - Diagnosed bare TypePack names in element position (`*args: P`) with `DiagnosticCode::KindExpectedType`.
   - Preserved external parameter labels on lowered semantics.
4. **Local Parameter Binding:**
   - In `checker/context.rs:bind_canonical_callable_parameter`, bound parameters using derived body Pack types rather than raw parameter types.
5. **Inference & Solver:**
   - Added `ConstraintOrigin::RestLane { call: ExpressionId, parameter_index: u16 }` mapped to `InferenceConstraintRole::ValueSelection`.
   - Whole type-pack variable unification unifies empty and non-empty lanes via `InferenceRelation::Equivalent` without introducing a TypePack subtype lattice.
6. **Call Checking (`checker/call.rs`):**
   - Preserved data constructor shape handling.
   - Evaluated all arguments in strict source order exactly once.
   - Assembled captured lane terms (`TypePack([])`, `TypePack([args..])`, `LabeledTypePack`, `CompleteTypePack`) and constrained generic variables via `InferenceRelation::Equivalent`.
   - Recorded required premises for empty and non-empty lane captures.
   - In `apply_non_generic_callable`, verified concrete lane contracts (`*(Number, String)`) directly with topology and pointwise subtyping.
   - Extended `callable_signature_mentions_parameters` and `specialize_dispatch_signature` to properly inspect and specialize parameter contracts across inherent impls.
   - In `types/specialization.rs:specialize_receiver_to_owner`, included `TypeData::ClassObject` so class-side receivers provide template arguments for inherent impl method specialization.
7. **Fingerprint & Override:**
   - In `db/fingerprint.rs`, hashed `ParameterContract` in callable signature fingerprints.
   - In `checker/declaration.rs`, enforced `ParameterContract` compatibility in override checks (rejecting Element vs Lane mismatches).

---

## 2. Verification Evidence

### Targeted Suite Verification:
- **Parser Test Suite:**
  `RUSTFLAGS='' cargo test -p phalcom-ast --test integration rest_lane_annotations` -> **PASS** (1 passed, 0 failed).
- **Primary Acceptance Test:**
  `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic generic_pack_query_infers_empty_and_heterogeneous_goal_lanes` -> **PASS** (1 passed, 0 failed).
  - Verified `empty` infers `TypePack[]` established by generic inference.
  - Verified `mixed` infers `TypePack[Int, String]` established by generic inference.
- **Rest Contracts Regression Suite:**
  `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::rest_contracts` -> **PASS** (17 passed, 0 failed).
  - All 15 mandatory anti-regressions and acceptance tests passing:
    1. `rest_plain_annotation_remains_element_contract`
    2. `rest_typepack_name_is_not_implicitly_expanded`
    3. `rest_typepack_element_contract_rejects_non_type_kind`
    4. `rest_explicit_positional_lane_infers_whole_typepack`
    5. `rest_empty_lane_infers_empty_typepack`
    6. `rest_capture_local_is_proper_pack_not_element_type`
    7. `rest_lane_capture_local_is_pack_from_inferred_lane`
    8. `rest_pack_of_expansion_remains_proper_pack_element_contract`
    9. `rest_concrete_lane_uses_pointwise_value_conformance`
    10. `rest_whole_pack_inference_does_not_create_typepack_subtyping`
    11. `rest_lane_inference_does_not_partition_pack_concat`
    12. `rest_argument_analysis_preserves_source_order`
    13. `rest_complete_capture_preserves_both_lanes`
    14. `rest_labeled_lane_preserves_labels_and_order`
    15. `rest_contract_change_invalidates_dependent_calls`
    16. `rest_concrete_lane_rejects_arity_mismatch`
    17. `override_checking_rejects_element_vs_lane_mismatch`
- **Solver Unit Tests:**
  `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic whole_type_pack_variable_unifies_with_empty_and_concrete_lanes` -> **PASS**.
- **Foundations Annotations Test:**
  `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic rest_parameter_lane_projection_and_kind_enforcement` -> **PASS**.
- **Pre-existing Failures Classified D:**
  `Future` and `unsaturated_generic_constructors` tests remain classified as baseline D defects outside this task.

---

## 3. Preserved Invariants

1. Ongoing work in `packages/parse/` and `phalcom-core/tests/core/mod.rs` untouched.
2. Evaluated all arguments in strict source order.
3. No global expansion in type positions.
4. Single authority parameter contracts preserved across semantic, dispatch, and specialization boundaries.
