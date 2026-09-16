# LANG005.C6.P1 Implementation Guidance: Tasks T5–T9

This document provides concise, actionable, and symbol-accurate implementation guidance for the remaining slices of **LANG005.C6.P1** (Tasks T5 through T9), grounded in the live codebase state following T1–T4.

---

## 1. Grounding & Baseline State (T1–T4 Review)

The foundational contracts established in T1–T4 are live in `phalcom-semantic`:
- **T1**: `Token::Conforms` (`|=`), `GenericConstraintSyntax::Conforms`, and `GenericConstraint::Conforms { subject: TypeTerm, trait_ref: TraitRef }` with canonical sorting and deduplication.
- **T2**: `generic_evidence.rs` defines `GenericConformanceKey`, `AssumedConformanceEvidence`, `GenericConformanceEvidence { Exact, Assumed }`, `GenericConformanceProof` (with complete terminal variants), and `query_generic_conformance_proof(...)`.
- **T3**: `GenericEvidenceSlotKey`, deterministic `GenericEvidenceLayout`, and `GenericEvidenceEnvironment` with lexical composition of owner + callable bounds.
- **T4**: `TraitDispatchSite::Assumed(AssumedTraitDispatchSelection)` in `trait_dispatch.rs` enabling lexical assumed trait dispatch on generic receiver types.

---

## 2. Task-by-Task Implementation Plan (T5 to End)

---

### Task T5: Discharge Generic Call Conformance Obligations & Evidence Binding

#### Objective
During callable application (`checker/call.rs`), after type inference solves type arguments:
1. Instantiate callee `GenericConstraint::Conforms` obligations under candidate substitutions.
2. Query `query_generic_conformance_proof(...)` using the caller's `GenericEvidenceEnvironment`.
3. Distinguish **Exact** evidence (`ConformanceEvidence`) from **Forwarded** evidence (`AssumedConformanceEvidence`).
4. Attach `Box<[GenericCallConformanceBinding]>` to `CallSpecialization`.
5. Emit rich diagnostics (`GenericConstraintUnsatisfied`) if proof fails, ensuring failures are isolated and do not contaminate other calls.

#### Key Types & Modifications
- In `phalcom-semantic/src/checker/analysis.rs`:
  ```rust
  #[derive(Clone, Debug, Eq, PartialEq)]
  pub struct GenericCallConformanceBinding {
      pub callee_conformance: GenericConformanceKey,
      pub source: GenericCallConformanceSource,
  }

  #[derive(Clone, Debug, Eq, PartialEq)]
  pub enum GenericCallConformanceSource {
      Exact { evidence: std::sync::Arc<crate::impls::ConformanceEvidence> },
      Forwarded { caller_conformance: GenericConformanceKey },
  }

  #[derive(Clone, Debug, Eq, PartialEq)]
  pub struct CallSpecialization {
      pub callable: CallableId,
      pub bindings: Box<[(TypeParameterId, TypeId)]>,
      pub conformances: Box<[GenericCallConformanceBinding]>,
  }
  ```
- In `phalcom-semantic/src/checker/call.rs`:
  - In `apply_generic_callable_in_context`:
    - After `InferenceOutcome::Solved(solution)` / `projected_solution`, collect all `GenericConstraint::Conforms` from `domains.signatures()`.
    - For each `Conforms { subject, trait_ref }`:
      - Substitute `subject` and `trait_ref` using solved `TypeParameterId -> TypeId` map.
      - Call `query_generic_conformance_proof(&instantiated_subject, &instantiated_trait_ref, Some(&ctx.generic_evidence), ...)`.
      - If `Proven(GenericConformanceEvidence::Exact(ev))`: record `GenericCallConformanceSource::Exact { evidence: ev }`.
      - If `Proven(GenericConformanceEvidence::Assumed(assumed))`: record `GenericCallConformanceSource::Forwarded { caller_conformance: assumed.key }`.
      - If non-proven (`Missing`, `Unknown`, `Blocked`, `Dynamic`, etc.): emit diagnostic `DiagnosticCode::GenericConstraintUnsatisfied` with structured explanation and record `AnalysisStatus::Invalid(cause)`.
  - Update `publish_call_specialization` to accept the computed `conformances` slice.

#### Focused Validation
```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::conforming_concrete_argument_discharges_direct_bound
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::bounded_generic_caller_forwards_evidence_to_bounded_callee
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::dynamic_does_not_discharge_static_conformance_bound
cargo test -p phalcom-semantic --test semantic capabilities::constraints
```

---

### Task T6: Lower Generic Evidence to Runtime Call Entries & Conformance Environments

#### Objective
Lower semantic evidence bindings in `phalcom-core` into executable runtime recipes without repeating semantic proof or trait searches in the compiler or VM.

#### Core Contracts & Lowering Flow
1. **Recipe Projection**:
   - `CallSpecialization` lowering translates each `GenericCallConformanceBinding` against callee's `GenericEvidenceLayout`:
     - `Exact`: Extract executable method `ObjRef` (witness or selected default) for each requirement in callee layout.
     - `Forwarded`: Map callee slot index $N \leftarrow$ caller slot index $M$ from caller's `GenericEvidenceLayout`.
2. **Runtime Conformance Environment Construction**:
   - Assemble `RuntimeConformanceEnvironment { slots: Box<[Option<ObjRef>]> }`.
   - Intern into `RuntimeConformanceEnvironmentRegistry::intern(...)`.
3. **Assumed Body Dispatch Execution**:
   - `TraitDispatchSite::Assumed(sel)` lowers to `InvokeTraitRequirement { slot, arity }` where `slot` is obtained deterministically from `GenericEvidenceEnvironment.layout.slot_for(&GenericEvidenceSlotKey { conformance: sel.conformance, requirement: sel.requirement })`.
4. **Activation & Frame**:
   - Generic activation receives both `RuntimeTypeEnvironmentId` and `RuntimeConformanceEnvironmentId`.

#### Files to Touch
- `phalcom-core/src/typing/conformance_environment.rs`
- `phalcom-core/src/typing/environment.rs`
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/modules/compile.rs`
- `phalcom-core/src/vm/dispatch.rs` / `vm/send.rs`
- New test: `phalcom-core/tests/core/language/generic_conformance.rs` (registered in `tests/core/language/mod.rs`)

#### Focused Validation
```sh
cargo test -p phalcom-core --test core language::generic_conformance
```

---

### Task T7: Metadata Round-Trip, Reflection, and Cross-Module Parity

#### Objective
Ensure `GenericConstraint::Conforms` is serializable, importable, verifiable, and inspectable through stable metadata and runtime reflection.

#### Key Types & Implementations
- In `phalcom-type-meta`:
  - `StableTraitApplicationRef { declaration: StableDeclarationRef, arguments: Box<[TypeNodeId]> }`
  - `GenericConstraintRef::Conforms { subject: TypeNodeId, trait_application: StableTraitApplicationRef }`
- In `phalcom-semantic/src/metadata/export.rs` & `types/native.rs`:
  - Export `GenericConstraint::Conforms` into `GenericConstraintRef::Conforms`.
  - Importer verifies arity and valid declaration target.
- In `phalcom-core/src/typing/inspect.rs`:
  - Add reflection support for `Conforms` constraints exposing `subject` and `trait_ref`.

#### Focused Validation
```sh
cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance::stable_metadata_preserves_direct_conformance_constraint
cargo test -p phalcom-core --test core language::generic_conformance::cross_module_bounded_generic_call_executes_selected_evidence
```

---

### Task T8: Incremental Invalidation, Source-Index, LSP & Presentation

#### Objective
1. **Change Invalidation**: Bound changes invalidate only affected signatures, bodies, call sites, and exported fingerprints (`db/fingerprint.rs`).
2. **Reordering No-Op**: `where T |= A + B` vs `where T |= B + A` results in identical semantic fingerprints and slot layouts.
3. **Source Index**: Map trait reference in `where T |= Printable` to canonical `DeclarationId` for LSP Go-To-Definition (`source_index/builder.rs`).
4. **Associated Hover Fix**: Format `Cell.Item` as `Cell::Item` in LSP hover (`phalcom-lsp`).

#### Focused Validation
```sh
cargo test -p phalcom-semantic --test semantic incremental::fingerprints
cargo test -p phalcom-semantic --test semantic incremental::recomputation_metrics
cargo test -p phalcom-lsp
```

---

### Task T9: Stabilization, Baselines Classification, Walkthrough & Handoff

#### Objective
1. Run Gate G6 verification:
   ```sh
   cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance
   cargo test -p phalcom-semantic --test semantic capabilities::constraints
   cargo test -p phalcom-core --test core language::generic_conformance
   ```
2. Verify all 36 coverage obligations (`CV-01` through `CV-36`).
3. Classify baseline test results.
4. Update `LANG005.C6-CHECKPOINT.md` to `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
5. Create `LANG005.C6.P1-walkthrough.md` and `LANG005.C6.P1-handoff.md`.

---

## 3. Fixed Architectural Invariants (Must Not Deviate)
1. **No VM/Runtime Trait Search**: Runtime carries only compiler-selected `ObjRef` slots.
2. **Separate Type & Conformance Environments**: Keep `RuntimeTypeEnvironment` and `RuntimeConformanceEnvironment` distinct.
3. **Slot Identity**: Identified by `(GenericConformanceKey, TraitRequirementId)`, not requirement alone.
4. **Dynamic Never Proves Bound**: Dynamic types stay dynamic boundaries, failing static conformance checks.
5. **Deterministic Slot Order**: Evidence layouts are sorted canonically by `GenericEvidenceSlotKey`.
