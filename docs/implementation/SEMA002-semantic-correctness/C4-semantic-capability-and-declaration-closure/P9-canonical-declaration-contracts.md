# Phalcom Canonical Declaration Contracts and Evidence Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Option B so canonical partial declaration contracts own callable/field type declarations, body analysis consumes them without provenance loss, advisory analysis is seeded from formal parameter facts, and editor/LSP type hints use one compiler-owned presentation path.

**Architecture:** Add canonical parameter identity and declaration contract types, publish declaration contract sets as incremental DB products, project those products into dispatch, consume canonical callable contracts in formal body checking, attach parameter bindings by identity, refine advisory observations against formal baselines, and expose protocol-neutral type-hint products through `EditorSemanticQuery`. `phalcom-lsp` becomes a renderer of those products and removes AST-owned annotation suppression from the canonical path.

**Tech Stack:** Rust 2024 workspace; `phalcom-ast`; `phalcom-common`; `phalcom-modules`; `phalcom-semantic`; `phalcom-lsp`; existing semantic DB/query/fingerprint infrastructure; `tower-lsp` only in the LSP crate.

**Spec:** `docs/impl/semantic/semantic-correctness/part-3/phalcom_canonical_declaration_contracts_tech_spec.md`

**Grounded repository:** `aureat/phalcom-lang`  
**Grounded branch:** `main`  
**Grounded HEAD:** `9b30ec324d4361128f285154fe236e25746df750`  
**Grounded date:** 2026-08-28

## Global Constraints

- [ ] Treat `9b30ec324d4361128f285154fe236e25746df750` as the implementation baseline. If `main` advances, repeat Task 0's symbol/path inventory before applying edits.
- [ ] `phalcom-semantic` remains the only semantic authority. Do not add inference or declaration-contract reconstruction to `phalcom-lsp`.
- [ ] Do not merge advisory facts into formal `TypeKnowledge` or checker acceptance.
- [ ] Do not promote advisory field observations into formal field contracts in this plan.
- [ ] Partial callable signatures must publish canonical `CallableSemanticSignature` products.
- [ ] Dispatch surfaces are projections from canonical contracts after Task 3; do not add new production code that reconstructs canonical signatures from surfaces.
- [ ] Source annotation presence/ranges are compiler-owned source metadata after Task 4.
- [ ] Method, setter, and index parameters have exactly one ordinary type-hint owner: `CallableParameterId`.
- [ ] Every task follows TDD: add focused failing tests, demonstrate the expected failure, implement the task, run focused tests, then run the relevant crate-level suite.
- [ ] Do not leave compatibility aliases/adapters without an explicit deletion step in this plan.
- [ ] Preserve constructor class-side public identity versus instance-side body identity behavior.
- [ ] Preserve the formal doctrine: source annotations may supply assumptions when formal value evidence is unavailable; proven incompatible formal evidence refutes the annotation.
- [ ] Keep `tower_lsp`/LSP protocol types out of `phalcom-semantic`.

---

# 0. Verified Baseline and File Map

At the grounded HEAD:

- `phalcom-semantic/src/signature.rs` has complete-only `TypeTerm` parameter/return fields.
- `phalcom-semantic/src/db/query.rs::semantic_signature_from_surface` refuses partial signatures.
- `phalcom-semantic/src/checker/binding.rs::BindingContractOrigin` mixes source/provenance and declaration role.
- `phalcom-semantic/src/checker/body.rs` obtains body parameter types from `SurfaceDispatchResolver` and passes `body_range` to every parameter binding.
- `phalcom-semantic/src/source_index/scope.rs` stores positional `parameter_name_ranges` and booleans rather than parameter source records.
- `phalcom-semantic/src/session.rs` imports/uses `AdvisoryParameterSlot` and seeds callable advisory bindings from caller contributions or `Unknown`.
- `phalcom-lsp/src/inlay_hints.rs` owns `ExplicitAnnotationIndex` and recursively walks `Program` for hint suppression.

## Files to create

- `docs/impl/semantic/semantic-correctness/part-3/phalcom_canonical_declaration_contracts_tech_spec.md`
- `docs/impl/semantic/semantic-correctness/part-3/phalcom_canonical_declaration_contracts_implementation_plan.md`
- `phalcom-semantic/src/contract.rs`
- `phalcom-semantic/tests/semantic/integration/contracts.rs`

## Files expected to modify

Core model and publication:

- `phalcom-semantic/src/identity.rs`
- `phalcom-semantic/src/lib.rs`
- `phalcom-semantic/src/signature.rs`
- `phalcom-semantic/src/dispatch.rs`
- `phalcom-semantic/src/surface.rs`
- `phalcom-semantic/src/snapshot.rs`
- `phalcom-semantic/src/session.rs`

DB/query graph:

- `phalcom-semantic/src/db/key.rs`
- `phalcom-semantic/src/db/product.rs`
- `phalcom-semantic/src/db/fingerprint.rs`
- `phalcom-semantic/src/db/query.rs`
- `phalcom-semantic/src/db/mod.rs`

Formal checking:

- `phalcom-semantic/src/checker/analysis.rs`
- `phalcom-semantic/src/checker/binding.rs`
- `phalcom-semantic/src/checker/context.rs`
- `phalcom-semantic/src/checker/incident.rs`
- `phalcom-semantic/src/checker/body.rs`
- `phalcom-semantic/src/checker/declaration.rs`
- `phalcom-semantic/src/checker/flow/state.rs` if its stored contract type is named directly.

Source identity:

- `phalcom-semantic/src/source_index/scope.rs`
- `phalcom-semantic/src/source_index/builder.rs`
- `phalcom-semantic/src/source_index/mod.rs`

Advisory:

- `phalcom-semantic/src/advisory/parameters.rs`
- `phalcom-semantic/src/advisory/agreement.rs`
- `phalcom-semantic/src/advisory/summary.rs`
- `phalcom-semantic/src/advisory/solver.rs`
- `phalcom-semantic/src/advisory/flow.rs`
- `phalcom-semantic/src/advisory/query.rs`
- `phalcom-semantic/src/advisory/workspace.rs`
- `phalcom-semantic/src/advisory/mod.rs`

Editor presentation:

- `phalcom-semantic/src/editor.rs`
- `phalcom-semantic/src/presentation.rs`

Tests:

- `phalcom-semantic/tests/semantic/integration/mod.rs`
- `phalcom-semantic/tests/semantic/integration/source_index.rs`
- `phalcom-semantic/tests/semantic/integration/advisory_analysis.rs`
- `phalcom-semantic/tests/semantic/integration/editor.rs`
- `phalcom-semantic/tests/semantic/integration/presentation.rs`
- `phalcom-semantic/tests/semantic/integration/workspace.rs`
- `phalcom-lsp/src/inlay_hints.rs`
- `phalcom-lsp/tests/stage6_inlay_hints.rs`
- `phalcom-lsp/tests/professional_semantic_presentation.rs`

---

# Task 0 — Install the spec/plan and freeze characterization tests

**Purpose:** Pin the current failure modes before changing shared representations.

**Files:**
- Create: `docs/impl/semantic/semantic-correctness/part-3/phalcom_canonical_declaration_contracts_tech_spec.md`
- Create: `docs/impl/semantic/semantic-correctness/part-3/phalcom_canonical_declaration_contracts_implementation_plan.md`
- Create: `phalcom-semantic/tests/semantic/integration/contracts.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/mod.rs`
- Modify: `phalcom-lsp/tests/stage6_inlay_hints.rs`

**Interfaces:**
- Consumes: current `SemanticWorkspaceSession`, `SemanticSnapshot`, source index, advisory query API.
- Produces: failing characterization tests used by later tasks.

- [ ] **Step 1: Install both design documents at their in-repository paths.**

Copy the delivered technical spec and this plan verbatim. Record the grounded HEAD at the top of both.

- [ ] **Step 2: Register the new semantic integration module.**

In `phalcom-semantic/tests/semantic/integration/mod.rs`, add:

```rust
mod contracts;
```

Do not remove existing integration modules.

- [ ] **Step 3: Add characterization fixture for a partial callable with an annotated parameter.**

In `contracts.rs`, use the existing integration support pattern to analyze:

```phalcom
class Probe {
    run(value: String) {
        unknownThing()
    }
}
```

This deliberately keeps the return type unavailable while the parameter contract is known. Assert the existing limitation explicitly first: `snapshot.callable_signatures.get(&run_id)` is currently absent because the complete-signature gate blocks publication. Then add the target assertions as ignored/commented expectation only if the test framework requires the suite to stay green.

Preferred final test name:

```rust
fn partial_callable_preserves_known_parameter_contract()
```

The final post-Task-3 assertions must be:

```rust
let signature = snapshot
    .callable_signatures
    .get(&run_id)
    .expect("partial canonical signature must exist");
assert_eq!(signature.parameters.len(), 1);
assert!(signature.parameters[0].contract.is_known());
assert!(signature.return_contract.is_unknown());
```

- [ ] **Step 4: Add source-attachment characterization.**

Add a fixture with:

```phalcom
class Probe {
    run(value: String) {
        value
    }
}
```

Locate the formal binding for `value` and the source binding for the parameter. The final assertion after Task 5 will require their attachment to the exact parameter binding site and will reject `analysis.body_range` as the binding's declaration range.

- [ ] **Step 5: Add LSP regression for annotated parameter duplication.**

In `phalcom-lsp/tests/stage6_inlay_hints.rs`, add/extend a test with:

```phalcom
class Probe {
    run(value: String) {
        value
    }
}
```

Final expected result:

```rust
assert!(
    hints.iter().all(|hint| {
        // no hint is placed directly after the explicitly annotated `value`
        hint.position != value_name_end
    })
);
```

Also add an unannotated method parameter fixture and assert exactly one hint at that parameter position.

- [ ] **Step 6: Run characterization suites.**

Run:

```bash
cargo test -p phalcom-semantic --test semantic integration::contracts -- --nocapture
cargo test -p phalcom-lsp --test stage6_inlay_hints -- --nocapture
```

Expected before implementation: at least the newly enabled target assertions fail for the known complete-signature/source/hint issues.

- [ ] **Step 7: Commit the characterization baseline.**

```bash
git add \
  docs/impl/semantic/semantic-correctness/part-3/phalcom_canonical_declaration_contracts_tech_spec.md \
  docs/impl/semantic/semantic-correctness/part-3/phalcom_canonical_declaration_contracts_implementation_plan.md \
  phalcom-semantic/tests/semantic/integration/contracts.rs \
  phalcom-semantic/tests/semantic/integration/mod.rs \
  phalcom-lsp/tests/stage6_inlay_hints.rs
git commit -m "test(semantic): characterize declaration contract gaps"
```

---

# Task 1 — Add canonical parameter identity and declaration contract types

**Files:**
- Create: `phalcom-semantic/src/contract.rs`
- Modify: `phalcom-semantic/src/identity.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/contracts.rs`

**Interfaces:**
- Produces:
  - `CallableParameterId`
  - `ContractType`
  - `ContractBasis`
  - `TypeContract`

- [ ] **Step 1: Write unit/integration tests for identity stability.**

Add:

```rust
#[test]
fn callable_parameter_identity_is_callable_plus_index() {
    let callable = fixture_callable_id("Probe", "run");
    let left = CallableParameterId::new(callable.clone(), 0);
    let same = CallableParameterId::new(callable.clone(), 0);
    let next = CallableParameterId::new(callable, 1);

    assert_eq!(left, same);
    assert_ne!(left, next);
}
```

Use the existing test helpers for `CallableId`; do not invent a second fixture identity format.

- [ ] **Step 2: Add `CallableParameterId` to `identity.rs`.**

Add exactly:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CallableParameterId {
    pub callable: CallableId,
    pub index: u32,
}

impl CallableParameterId {
    pub fn new(callable: CallableId, index: u32) -> Self {
        Self { callable, index }
    }
}
```

Place it adjacent to `CallableId`/other callable-owned identities.

- [ ] **Step 3: Create `contract.rs`.**

Add:

```rust
use crate::diagnostic::SemanticSourceSpan;
use crate::types::evidence::UnknownReason;
use crate::types::parameter::TypeTerm;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractType {
    Known(TypeTerm),
    Dynamic,
    Unknown(UnknownReason),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ContractBasis {
    Unspecified,
    SourceAnnotation,
    InitializerInference,
    BodyInference,
    NativeSignature,
    DeclarationSemantics,
    ConstructorSemantics,
    ContextualTyping,
    PatternDecomposition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeContract {
    pub ty: ContractType,
    pub basis: ContractBasis,
    pub source: Option<SemanticSourceSpan>,
}
```

Implement the constructors and queries named by the technical spec. `unknown(reason)` MUST use `ContractBasis::Unspecified`.

- [ ] **Step 4: Add pure conversion helpers from resolved declaration knowledge.**

Add a helper that converts `TypeKnowledge` to `ContractType` without pretending that an `Unknown` is a missing declaration object:

```rust
pub fn contract_type_from_knowledge(
    knowledge: &TypeKnowledge,
) -> ContractType
```

Rules:

```text
Known(ev) → Known(TypeTerm::Canonical(ev.ty()))
Dynamic(_) → Dynamic
Unknown(r) → Unknown(r.clone())
```

Do not derive `ContractBasis` inside this function; the caller knows whether the input came from source annotation, native metadata, or inference.

- [ ] **Step 5: Export the new module/types from `lib.rs`.**

Add:

```rust
pub mod contract;
pub use contract::{ContractBasis, ContractType, TypeContract};
pub use identity::CallableParameterId;
```

Respect existing module/export ordering.

- [ ] **Step 6: Run focused tests.**

```bash
cargo test -p phalcom-semantic --test semantic integration::contracts -- --nocapture
cargo test -p phalcom-semantic contract --lib
```

Expected: new identity/contract tests pass; characterization failures from later tasks remain.

- [ ] **Step 7: Commit.**

```bash
git add phalcom-semantic/src/contract.rs \
        phalcom-semantic/src/identity.rs \
        phalcom-semantic/src/lib.rs \
        phalcom-semantic/tests/semantic/integration/contracts.rs
git commit -m "feat(semantic): add canonical declaration contracts"
```

---

# Task 2 — Make canonical callable and field signatures partial

**Files:**
- Modify: `phalcom-semantic/src/signature.rs`
- Modify: native/source signature construction call sites found by `rg "CallableParameterSemantic|CallableSemanticSignature|FieldSemanticSignature" phalcom-semantic -n`
- Modify: `phalcom-semantic/tests/semantic/integration/contracts.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/native_conformance.rs` where constructor signatures are asserted.

**Interfaces:**
- Consumes: `CallableParameterId`, `TypeContract`.
- Produces:
  - partial `CallableSemanticSignature`;
  - `DeclarationContractSet`.

- [ ] **Step 1: Add failing construction tests for partial signatures.**

Construct a semantic signature whose parameter is known and return is unknown:

```rust
let parameter = CallableParameterSemantic::new(
    CallableParameterId::new(callable.clone(), 0),
    "value",
    TypeContract::source_annotation(
        ContractType::Known(TypeTerm::Canonical(string_ty)),
        source_span,
    ),
);

let signature = CallableSemanticSignature {
    callable: callable.clone(),
    owner: callable.owner.clone(),
    side: callable.side,
    selector: callable.selector.clone(),
    generics: None,
    parameters: vec![parameter].into_boxed_slice(),
    return_contract: TypeContract::unknown(UnknownReason::UnannotatedDeclaration),
    source: None,
    implementation: ImplementationKind::Source,
    native_id: None,
    effects: EffectSpec::Unknown,
    raises: RaisesSpec::Unknown,
    flow: ReturnFlowSpec::Value,
    lifecycle: NativeLifecycleSpec::UNKNOWN,
};

assert!(!signature.is_complete());
assert!(signature.parameter_contract_at(0).unwrap().is_known());
```

- [ ] **Step 2: Refactor `CallableParameterSemantic`.**

Replace:

```rust
pub index: u32,
pub ty: TypeTerm,
```

with:

```rust
pub id: CallableParameterId,
pub contract: TypeContract,
```

Change constructor to:

```rust
pub fn new(
    id: CallableParameterId,
    local_name: impl Into<Box<str>>,
    contract: TypeContract,
) -> Self
```

Add:

```rust
pub fn index(&self) -> u32 {
    self.id.index
}
```

- [ ] **Step 3: Refactor callable return field.**

Replace:

```rust
pub return_type: TypeTerm,
```

with:

```rust
pub return_contract: TypeContract,
```

Replace `parameter_type_at` with:

```rust
pub fn parameter_contract_at(&self, index: usize) -> Option<&TypeContract>;
```

Add `is_complete()` that checks every parameter contract and return contract but does not control publication.

- [ ] **Step 4: Refactor `FieldSemanticSignature`.**

Replace:

```rust
pub ty: TypeTerm,
```

with:

```rust
pub contract: TypeContract,
```

- [ ] **Step 5: Add `DeclarationContractSet`.**

In `signature.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationContractSet {
    pub declaration: DeclarationId,
    pub callables: BTreeMap<CallableId, CallableSemanticSignature>,
    pub fields: BTreeMap<FieldId, FieldSemanticSignature>,
}
```

Add `new`, `callable`, `field`, and iterators.

- [ ] **Step 6: Migrate native signature constructors.**

For every current native `CallableSemanticSignature`, wrap canonical types using:

```rust
TypeContract::native(TypeTerm::Canonical(ty))
```

or the equivalent constructor introduced in Task 1.

Do not alter native selector identity/effects/raises/flow/lifecycle.

- [ ] **Step 7: Compile and fix direct field references mechanically.**

Run:

```bash
cargo check -p phalcom-semantic
```

Replace semantic-signature accesses only. Do not change dispatch authority yet; Task 3 performs that cutover.

- [ ] **Step 8: Run tests.**

```bash
cargo test -p phalcom-semantic --test semantic integration::contracts -- --nocapture
cargo test -p phalcom-semantic --test semantic integration::native_conformance -- --nocapture
```

- [ ] **Step 9: Commit.**

```bash
git add phalcom-semantic/src/signature.rs phalcom-semantic/src phalcom-semantic/tests/semantic/integration
git commit -m "refactor(semantic): represent partial callable contracts"
```

---

# Task 3 — Make declaration contracts the authority and dispatch a projection

**Files:**
- Modify: `phalcom-semantic/src/checker/declaration.rs`
- Modify: `phalcom-semantic/src/dispatch.rs`
- Modify: `phalcom-semantic/src/surface.rs`
- Modify: `phalcom-semantic/src/db/key.rs`
- Modify: `phalcom-semantic/src/db/product.rs`
- Modify: `phalcom-semantic/src/db/fingerprint.rs`
- Modify: `phalcom-semantic/src/db/query.rs`
- Modify: `phalcom-semantic/src/db/mod.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/contracts.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/workspace.rs`

**Interfaces:**
- Produces:
  - `QueryKey::DeclarationContracts`;
  - `query_declaration_contracts`;
  - `dispatch_signature_from_semantic`;
  - `query_callable_signature` independent of surface completeness.

- [ ] **Step 1: Write DB regression: partial signature query is Ready.**

Build a source declaration whose parameter is annotated but whose body keeps the return unresolved:

```phalcom
class Probe {
    run(value: String) {
        unknownThing()
    }
}
```

After session update:

```rust
let signature = snapshot
    .callable_signatures
    .get(&callable)
    .expect("canonical signature published");
assert!(signature.parameters[0].contract.is_known());
assert!(signature.return_contract.is_unknown());
```

This is the Task-0 characterization test promoted to a required passing test.

- [ ] **Step 2: Add `DeclarationContracts` query key.**

In `db/key.rs` add:

```rust
DeclarationContracts(DeclarationId),
```

Update all exhaustive matches for module/declaration ownership and display/debug categorization.

- [ ] **Step 3: Add typed product storage.**

In `db/product.rs` / `db/mod.rs`, add:

```rust
SemanticProduct::DeclarationContracts(Arc<DeclarationContractSet>)
```

and:

```rust
pub fn as_declaration_contracts(
    &self,
) -> Option<&Arc<DeclarationContractSet>>
```

Follow the exact accessor style used by `DeclarationSurface` and `CallableSignature`.

- [ ] **Step 4: Add fingerprints.**

In `db/fingerprint.rs`, add:

```rust
pub fn declaration_contracts_input_fingerprint(...);
pub fn declaration_contracts_product_fingerprint(...);
```

Hash semantic contract content:

- declaration ID;
- callable IDs/selectors/sides;
- parameter IDs/labels/rest;
- `ContractType`;
- `ContractBasis`;
- return contract;
- field IDs/mutability/contracts;
- generic signature.

Do not use editor-only whitespace/trivia as semantic contract fingerprint input. Source annotation ranges belong in source/presentation fingerprints unless range identity already participates in the existing declaration source input fingerprint.

- [ ] **Step 5: Split `register_class_surface` into lowering + projection.**

In `checker/declaration.rs`, introduce:

```rust
pub fn lower_class_contracts(
    ctx: &mut CheckingContext<'_>,
    class_def: &ClassDef,
) -> DeclarationContractSet
```

Move the current source annotation resolution, constructor return semantics, generic signature resolution, parameter label/rest normalization, getter/setter/index contract construction, and field annotation resolution into this function.

For each `ParameterDef`:

```rust
let id = CallableParameterId::new(callable_id.clone(), index as u32);
let contract = if let Some(annotation) = &parameter.annotation {
    let (knowledge, _) = ctx.resolve_type_annotation(method_resolver, annotation);
    TypeContract {
        ty: contract_type_from_knowledge(&knowledge),
        basis: ContractBasis::SourceAnnotation,
        source: Some(SemanticSourceSpan::new(
            ctx.current_module.clone(),
            annotation.range,
        )),
    }
} else {
    TypeContract::unknown(UnknownReason::NoTypeEvidence)
};
```

For unannotated return slots use `UnknownReason::UnannotatedDeclaration`.

For constructor returns use `ContractBasis::ConstructorSemantics`; when projected into formal knowledge preserve `EvidenceOrigin::ConstructorSemantics`.

For setters preserve `Unit` return semantics.

- [ ] **Step 6: Add dispatch projection.**

In `dispatch.rs`, add:

```rust
pub fn dispatch_signature_from_semantic(
    signature: &CallableSemanticSignature,
) -> CallableSignature
```

Map `TypeContract` to declaration-level `TypeKnowledge`.

Required mapping for known source annotations:

```text
Known + SourceAnnotation → assumed / DeveloperAnnotation
Known + DeclarationSemantics → established / DeclarationSemantics
Known + ConstructorSemantics → established / ConstructorSemantics
Known + NativeSignature → established / NativeSignature
Unknown(r) → Unknown(r)
Dynamic → Dynamic(ExplicitEscape or preserved declaration dynamic reason)
```

Do not use `EvidenceOrigin::CallableSignature` merely because the value passes through a callable signature.

- [ ] **Step 7: Rebuild `register_class_surface` as a convenience projection.**

Its target structure is:

```rust
pub fn register_class_surface(
    ctx: &mut CheckingContext<'_>,
    class_def: &ClassDef,
) {
    let contracts = lower_class_contracts(ctx, class_def);
    let surface = DeclarationSurface::from_contracts(&contracts);
    ctx.register_surface(contracts.declaration.clone(), surface);
}
```

If `DeclarationSurface::from_contracts` is instead implemented as a free function in `surface.rs`, use that exact function consistently.

- [ ] **Step 8: Add `query_declaration_contracts`.**

In `db/query.rs`:

```rust
pub fn query_declaration_contracts(
    db: &mut SemanticDb,
    decl_id: DeclarationId,
    unit: Arc<ParsedModuleUnit>,
    linked_interface: Arc<LinkedModuleInterface>,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    resolver: &dyn TypeResolver,
    declarations: &DeclarationTypeTable,
) -> QueryOutcome<Arc<DeclarationContractSet>>
```

Dependencies:

```text
DeclarationShell(decl_id)
LinkedInterface(module)
```

Use `lower_class_contracts` as the only source declaration lowering path.

- [ ] **Step 9: Change `query_declaration_surface`.**

Replace its direct call to `register_class_surface` with:

```text
ensure/query DeclarationContracts
        ↓
project DeclarationContractSet
        ↓
publish DeclarationSurface
```

The surface query depends on `DeclarationContracts(decl_id)`.

- [ ] **Step 10: Replace `query_callable_signature`.**

Delete the complete-only conversion:

```rust
semantic_signature_from_surface(...)
```

and replace the query body with:

```rust
let contracts_key = QueryKey::DeclarationContracts(callable.owner.clone());
// validate current dependency
let contracts = db.product(&contracts_key)
    .and_then(|p| p.as_declaration_contracts())
    .ok_or(...)?;

let signature = contracts
    .callable(&callable)
    .cloned()
    .ok_or(...)?;

publish CallableSignature(signature) with dependency on contracts_key
```

A partial signature is `Ready`, not `Blocked`.

- [ ] **Step 11: Delete `semantic_signature_from_surface`.**

Remove its import from `session.rs` and all callers.

- [ ] **Step 12: Reverse inferred-return publication direction.**

In `session.rs::refresh_inferred_callable_results`:

Before:

```text
dispatch.update_callable_return_type
→ semantic_signature_from_surface
→ callable_signatures.insert
```

After:

```text
callable_signatures.get/update canonical return contract
→ TypeContract::inferred(..., ContractBasis::BodyInference)
→ rebuild/update dispatch projection from canonical signature
```

Add to `CallableSignatureTable`:

```rust
pub fn update_return_contract(
    &mut self,
    callable: &CallableId,
    contract: TypeContract,
) -> bool
```

Add to `SurfaceDispatchResolver` a projection update accepting `&CallableSemanticSignature`, or reconstruct the whole projected `CallableSignature` and replace that selector entry.

- [ ] **Step 13: Update query dependency recording.**

`record_consumed_callable_signature` must no longer gate `SemanticDependency::CallableSignature` on `has_complete_types()`. Every source callable signature is a valid query dependency after this task.

- [ ] **Step 14: Run tests.**

```bash
cargo test -p phalcom-semantic --test semantic integration::contracts -- --nocapture
cargo test -p phalcom-semantic --test semantic integration::workspace -- --nocapture
cargo test -p phalcom-semantic --test semantic integration::native_conformance -- --nocapture
cargo check -p phalcom-semantic
```

Expected: partial callable contract test passes.

- [ ] **Step 15: Commit.**

```bash
git add phalcom-semantic/src phalcom-semantic/tests/semantic/integration
git commit -m "refactor(semantic): make declaration contracts canonical"
```

---

# Task 4 — Publish exact parameter and annotation source metadata

**Files:**
- Modify: `phalcom-semantic/src/source_index/scope.rs`
- Modify: `phalcom-semantic/src/source_index/builder.rs`
- Modify: `phalcom-semantic/src/source_index/mod.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/source_index.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/contracts.rs`

**Interfaces:**
- Produces:
  - `CallableParameterSourceInfo`;
  - `SourceBindingInfo.annotation_range`;
  - exact parameter `binding_site`;
  - `CallableSourceInfo.return_annotation_range`.

- [ ] **Step 1: Add source-index tests first.**

For:

```phalcom
class Probe {
    run(label value: String) -> Int {
        1
    }
}
```

assert:

```rust
let callable = structure.callable_sources.get(&run_id).unwrap();
let parameter = &callable.parameters[0];

assert_eq!(parameter.id, CallableParameterId::new(run_id.clone(), 0));
assert_eq!(parameter.name_range, parsed_param.name_range);
assert_eq!(parameter.label_range, parsed_param.label_range);
assert_eq!(parameter.annotation_range, parsed_param.annotation.as_ref().map(|a| a.range));
assert_eq!(callable.return_annotation_range, method.return_annotation.as_ref().map(|a| a.range));
assert!(structure.bindings.contains_key(&parameter.binding_site));
```

- [ ] **Step 2: Add `CallableParameterSourceInfo` to `scope.rs`.**

Use the exact struct from the technical spec.

- [ ] **Step 3: Replace callable range arrays/boolean.**

Change:

```rust
parameter_name_ranges: Arc<[SourceRange]>,
has_explicit_return_annotation: bool,
```

to:

```rust
parameters: Arc<[CallableParameterSourceInfo]>,
return_annotation_range: Option<SourceRange>,
```

- [ ] **Step 4: Add `annotation_range` to `SourceBindingInfo`.**

Update every constructor in `source_index/builder.rs`.

Default to `None` for bindings without source type annotations.

- [ ] **Step 5: Make `declare` return the binding site.**

If the current builder `declare(...)` does not already return `SourceSiteId`, change it to return the canonical first-declaration site it registers.

Do not allocate a second site for parameter metadata.

- [ ] **Step 6: Populate callable parameter metadata during `visit_callable`.**

For each `ParameterDef` in declaration order:

```rust
let parameter_id =
    CallableParameterId::new(callable.clone(), index as u32);

let binding_site = self.declare(
    scope,
    parameter.name.clone(),
    parameter_kind,
    parameter.name_range,
    parameter.annotation.as_ref().map(|a| a.range),
    true,
);

parameter_sources.push(CallableParameterSourceInfo {
    id: parameter_id,
    binding_site,
    range: parameter.range,
    name_range: parameter.name_range,
    label_range: parameter.label_range,
    annotation_range: parameter.annotation.as_ref().map(|a| a.range),
});
```

Adapt the exact `declare` argument order to the builder, but keep one authoritative call.

For index-set `put`, assign the next parameter index after index parameters.

- [ ] **Step 7: Propagate annotation range through let/destructure declaration.**

Change `declare_pattern` to accept the owning annotation range:

```rust
fn declare_pattern(
    &mut self,
    ...,
    annotation_range: Option<SourceRange>,
)
```

For every leaf of an annotated pattern, store the same `annotation_range`.

For unannotated patterns pass `None`.

- [ ] **Step 8: Export the new source metadata type.**

Update `source_index/mod.rs` re-exports.

- [ ] **Step 9: Update source fingerprints.**

`ModuleSourceIndex::fingerprint` / presentation fingerprint must include:

- parameter IDs;
- annotation presence/range where presentation-sensitive;
- return annotation presence/range;
- binding annotation presence/range.

Do not turn pure range movement into an unnecessary semantic contract invalidation.

- [ ] **Step 10: Run tests.**

```bash
cargo test -p phalcom-semantic --test semantic integration::source_index -- --nocapture
cargo test -p phalcom-semantic --test semantic integration::contracts -- --nocapture
```

- [ ] **Step 11: Commit.**

```bash
git add phalcom-semantic/src/source_index \
        phalcom-semantic/tests/semantic/integration/source_index.rs \
        phalcom-semantic/tests/semantic/integration/contracts.rs
git commit -m "feat(semantic): attach exact parameter source metadata"
```

---

# Task 5 — Make formal body entry consume canonical parameter contracts

**Files:**
- Modify: `phalcom-semantic/src/checker/binding.rs`
- Modify: `phalcom-semantic/src/checker/analysis.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/body.rs`
- Modify: `phalcom-semantic/src/checker/flow/state.rs`
- Modify: `phalcom-semantic/src/db/query.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/contracts.rs`

**Interfaces:**
- Produces:
  - `BindingRole`;
  - `ResolvedBindingContract`;
  - parameter body bindings that preserve source basis and parameter identity.

- [ ] **Step 1: Add failing binding-state assertions.**

For:

```phalcom
class Probe {
    run(value: String) {
        value
    }
}
```

final assertions:

```rust
let state = find_binding_state(&analysis, "value");
assert_eq!(
    state.role,
    BindingRole::CallableParameter(
        CallableParameterId::new(run_id.clone(), 0)
    )
);
assert_eq!(
    state.contract.as_ref().unwrap().basis,
    ContractBasis::SourceAnnotation
);
assert_eq!(
    state.current.origin(),
    Some(EvidenceOrigin::DeveloperAnnotation)
);
assert_eq!(state.range, parameter_name_range);
assert_ne!(state.range, analysis.body_range);
```

- [ ] **Step 2: Replace `BindingContractOrigin`.**

In `checker/binding.rs`, remove:

```rust
BindingContractOrigin
BindingContract
```

and add:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindingRole {
    Local,
    CallableParameter(CallableParameterId),
    ContextualBlockParameter,
    PatternBinding,
    ForBinding,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedBindingContract {
    pub ty: TypeId,
    pub basis: ContractBasis,
    pub source: Option<SourceRange>,
}
```

Change `BindingSeed.contract` to:

```rust
Option<ResolvedBindingContract>
```

and add:

```rust
pub role: BindingRole,
```

- [ ] **Step 3: Replace reconciliation origin checks.**

Every:

```rust
matches!(contract.origin, BindingContractOrigin::SourceAnnotation)
```

becomes:

```rust
contract.basis == ContractBasis::SourceAnnotation
```

Update incident summaries and flow invariant diagnostics to refer to `basis`.

- [ ] **Step 4: Add role to `BindingState` / flow state.**

In `checker/analysis.rs` and the underlying flow binding state, retain `BindingRole`.

Add `FlowInvariantFailure::DivergentBindingRole { binding, left, right }` in `checker/flow/state.rs`; update `checker/incident.rs` and `checker/context.rs::publish_flow_join_failure` to report it as an internal semantic invariant violation. `join_with_hierarchy` must compare role alongside contract and mutability for the same `BindingId`.

- [ ] **Step 5: Add canonical-contract lowering at checker entry.**

In `checker/context.rs`, replace:

```rust
bind_callable_parameter(name, current, range)
```

with an API shaped like:

```rust
pub fn bind_callable_parameter(
    &mut self,
    parameter: &CallableParameterSemantic,
    range: SourceRange,
) -> BindingDeclarationResult
```

Implementation:

1. derive `BindingRole::CallableParameter(parameter.id.clone())`;
2. resolve `ContractType::Known(TypeTerm::Canonical(ty))` to `ResolvedBindingContract`;
3. for source annotation basis, seed `current` with `TypeKnowledge::Unknown(NoTypeEvidence)` and let normal reconciliation derive `DeveloperAnnotation` assumption, OR explicitly use the same pure helper used by reconciliation;
4. preserve dynamic/unknown contract states;
5. use the parameter's actual source range.

Do not mark known source annotations as `EvidenceOrigin::CallableSignature`.

- [ ] **Step 6: Keep contextual block parameter semantics separate.**

`bind_contextual_block_parameter` uses:

```rust
role: BindingRole::ContextualBlockParameter
basis: ContractBasis::ContextualTyping
current: assumed(..., EvidenceOrigin::ContextualDerivation)
```

- [ ] **Step 7: Migrate let/pattern callers.**

Map current binding origins:

```text
SourceAnnotation       → basis SourceAnnotation, role Local/Pattern as appropriate
InferredInitializer    → basis InitializerInference
ContextualBlockParameter → role ContextualBlockParameter, basis ContextualTyping
PatternBinding         → role PatternBinding, basis PatternDecomposition
CallableParameter      → removed; role carries this dimension
```

- [ ] **Step 8: Change body analysis signature input.**

In `checker/body.rs`, remove production dependence on:

```rust
signature_consumed_by_body(
    dispatch: &SurfaceDispatchResolver,
    callable: &CallableId
) -> Option<(CallableId, CallableSignature)>
```

Introduce canonical body signature input:

```rust
pub struct BodyCallableContract<'a> {
    pub body_callable: &'a CallableId,
    pub signature_callable: &'a CallableId,
    pub signature: &'a CallableSemanticSignature,
}
```

or an owned equivalent if query lifetimes require it.

Keep constructor fallback normalization explicit: an instance-side constructor body may consume the class-side canonical constructor signature, but the signature is canonical rather than reconstructed from dispatch.

- [ ] **Step 9: Change `query_callable_body_with_formal_inputs`.**

Always ensure the canonical callable signature query for source callables. Remove:

```rust
if signature.has_complete_types() { ... }
```

Pass the `CallableSemanticSignature` to `analyze_callable_body_with_fields`.

- [ ] **Step 10: Bind body parameters from canonical signature.**

Replace:

```rust
for param in &sig.parameters {
    ctx.bind_callable_parameter(
        param.local_name.clone(),
        param.ty.clone(),
        body_range,
    );
}
```

with:

```rust
for parameter in signature.parameters.iter() {
    let range = parameter
        .source
        .as_ref()
        .map(|source| source.range)
        .unwrap_or(body_range);
    ctx.bind_callable_parameter(parameter, range);
}
```

After Task 4, source callables MUST have parameter source metadata, so add a debug assertion/test that the fallback is not used for normal source parameters.

- [ ] **Step 11: Derive expected return from `return_contract`.**

Known proper return contract → `CallableReturnContract`.

Unknown return contract → no expected return.

Dynamic → existing dynamic-boundary policy.

Preserve setter/constructor special return handling.

- [ ] **Step 12: Update semantic dependency recording.**

`record_consumed_callable_signature` records `SemanticDependency::CallableSignature` for all canonical source callables.

- [ ] **Step 13: Run focused and formal suites.**

```bash
cargo test -p phalcom-semantic --test semantic integration::contracts -- --nocapture
cargo test -p phalcom-semantic --test semantic integration::compiler_capabilities -- --nocapture
cargo test -p phalcom-semantic --test semantic -- --nocapture
```

- [ ] **Step 14: Commit.**

```bash
git add phalcom-semantic/src/checker \
        phalcom-semantic/src/db/query.rs \
        phalcom-semantic/tests/semantic/integration/contracts.rs
git commit -m "refactor(semantic): bind parameters from canonical contracts"
```

---

# Task 6 — Attach parameter formal bindings by canonical identity

**Files:**
- Modify: `phalcom-semantic/src/source_index/mod.rs`
- Modify: `phalcom-semantic/src/presentation.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/source_index.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/presentation.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/contracts.rs`

**Interfaces:**
- Consumes: `BindingRole::CallableParameter`, `CallableParameterSourceInfo`.
- Produces: identity-first `BindingId → SourceSiteId` attachment for parameters.

- [ ] **Step 1: Add a source-attachment regression.**

After analyzing:

```phalcom
class Probe {
    run(value: String) {
        value
    }
}
```

find the `BindingState` and source parameter:

```rust
let parameter_site = callable_source.parameters[0].binding_site.clone();
let attached = attachment
    .source_site_for_binding(binding_state.binding)
    .expect("formal parameter binding attached");
assert_eq!(attached, &parameter_site);
```

- [ ] **Step 2: Change `CallableSourceAttachment::from_analysis_with_incidents`.**

Before the existing range/order fallback, handle parameter-role bindings:

```rust
if let BindingRole::CallableParameter(parameter_id) = &state.role {
    let source = scopes
        .callable_sources
        .get(&parameter_id.callable)
        .and_then(|callable| {
            callable
                .parameters
                .iter()
                .find(|p| p.id == *parameter_id)
        });

    match source {
        Some(source) => {
            formal_bindings.insert(
                state.binding,
                source.binding_site.clone(),
            );
            continue;
        }
        None => {
            incidents.push(SourceAttachmentError::MissingBinding { ... });
            continue;
        }
    }
}
```

Constructor body/public callable normalization must use the parameter identity stored in `BindingRole`; do not guess by source name.

- [ ] **Step 3: Restrict range fallback to non-parameter bindings.**

The existing same-name/same-range path remains for locals where necessary.

Add a debug assertion that a `CallableParameter` reaching the generic fallback is a bug.

- [ ] **Step 4: Update formal projection tests.**

`FormalSemanticProjection` should find parameter facts at the exact parameter source site and preserve the binding contract basis.

- [ ] **Step 5: Run tests.**

```bash
cargo test -p phalcom-semantic --test semantic integration::source_index -- --nocapture
cargo test -p phalcom-semantic --test semantic integration::presentation -- --nocapture
cargo test -p phalcom-semantic --test semantic integration::contracts -- --nocapture
```

Expected: no normal parameter attachment incident for the fixture.

- [ ] **Step 6: Commit.**

```bash
git add phalcom-semantic/src/source_index/mod.rs \
        phalcom-semantic/src/presentation.rs \
        phalcom-semantic/tests/semantic/integration
git commit -m "fix(semantic): attach parameter facts by identity"
```

---

# Task 7 — Replace advisory parameter slots and add formal baselines

**Files:**
- Modify: `phalcom-semantic/src/advisory/parameters.rs`
- Modify: `phalcom-semantic/src/advisory/summary.rs`
- Modify: `phalcom-semantic/src/advisory/solver.rs`
- Modify: `phalcom-semantic/src/advisory/flow.rs`
- Modify: `phalcom-semantic/src/advisory/query.rs`
- Modify: `phalcom-semantic/src/advisory/workspace.rs`
- Modify: `phalcom-semantic/src/advisory/mod.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/advisory_analysis.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/contracts.rs`

**Interfaces:**
- Produces:
  - advisory maps keyed by `CallableParameterId`;
  - `AdvisoryParameterState`;
  - formal baseline seeding.

- [ ] **Step 1: Add two failing advisory tests.**

Fixture A:

```phalcom
class User {
    use(name: String) {
        name
    }
}
```

No callers.

Final assertion:

```text
advisory parameter baseline = String
effective = String
```

Fixture B:

```phalcom
class User {
    _name

    setName(name: String) {
        _name = name
    }
}
```

Final assertion:

```text
advisory field _name = String
```

- [ ] **Step 2: Replace `AdvisoryParameterSlot`.**

Delete the struct from `advisory/parameters.rs`.

Change:

```rust
BTreeMap<AdvisoryParameterSlot, AdvisoryFact>
```

to:

```rust
BTreeMap<CallableParameterId, AdvisoryFact>
```

through contributions, flow products, solver nodes, summaries, query/workspace accessors, fingerprints, and session assembly.

- [ ] **Step 3: Change call contribution construction.**

In `advisory/flow.rs::record_call_contributions`, replace:

```rust
AdvisoryParameterSlot::new(call.target.clone(), index as u32)
```

with:

```rust
CallableParameterId::new(call.target.clone(), index as u32)
```

Keep label/positional selector-slot mapping unchanged.

- [ ] **Step 4: Add `AdvisoryParameterState`.**

In `advisory/parameters.rs` add the struct from the spec.

Add constructor/helper:

```rust
pub fn resolve_parameter_state(
    parameter: CallableParameterId,
    baseline: AdvisoryFact,
    observed: Option<AdvisoryFact>,
    agreement: AdvisoryAgreement,
) -> AdvisoryParameterState
```

The actual effective-fact policy is completed in Task 8 after hierarchy-aware agreement exists.

- [ ] **Step 5: Build formal baseline map in `session.rs`.**

Before each callable advisory traversal, derive baseline facts by joining canonical parameter source identity to formal binding state:

```text
CallableParameterId
    → CallableSourceInfo.parameters[].binding_site
    → formal attachment / analysis binding role
    → BindingState.current
    → advisory_fact_from_formal(... FormalFact(...))
```

Prefer direct analysis role lookup:

```rust
analysis.bindings.values().find(|binding| {
    binding.role
        == BindingRole::CallableParameter(parameter_id.clone())
})
```

This avoids a source round trip for formal identity. Use the source binding site only to seed the advisory binding environment.

- [ ] **Step 6: Replace current unknown fallback.**

Current behavior equivalent to:

```rust
parameter_facts
    .get(&slot)
    .cloned()
    .unwrap_or_else(AdvisoryFact::unknown)
```

becomes:

```text
baseline = formal baseline or Unknown
observed = caller contributions
state = resolve_parameter_state(...)
seed binding with state.effective
```

Before Task 8, compatible equality may use exact equality; hierarchy refinement lands next.

- [ ] **Step 7: Change `AdvisoryCallableSummary.parameters`.**

Store `AdvisoryParameterState` (or identity/state tuples if existing summary layout requires it) rather than bare `(slot, fact)` pairs.

The public query must be able to retrieve:

```rust
summary.parameter(&CallableParameterId)
```

and inspect `baseline`, `observed`, `effective`, and `agreement`.

- [ ] **Step 8: Run tests.**

```bash
cargo test -p phalcom-semantic --test semantic integration::advisory_analysis -- --nocapture
cargo test -p phalcom-semantic --test semantic integration::contracts -- --nocapture
```

The no-caller `String` baseline and parameter-to-field `String` flow must now pass.

- [ ] **Step 9: Commit.**

```bash
git add phalcom-semantic/src/advisory \
        phalcom-semantic/src/session.rs \
        phalcom-semantic/tests/semantic/integration
git commit -m "feat(semantic): seed advisory parameters from formal facts"
```

---

# Task 8 — Add hierarchy-aware advisory refinement and incompatibility containment

**Files:**
- Modify: `phalcom-semantic/src/advisory/agreement.rs`
- Modify: `phalcom-semantic/src/advisory/parameters.rs`
- Modify: `phalcom-semantic/src/advisory/solver.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/advisory_analysis.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/contracts.rs`

**Interfaces:**
- Produces:
  - `AdvisoryAgreement::Incompatible`;
  - hierarchy-aware nominal refinement;
  - safe `effective` parameter fact.

- [ ] **Step 1: Add Animal/Dog tests.**

Fixture:

```phalcom
class Animal {}
class Dog is Animal {}

class Consumer {
    use(value: Animal) {
        value
    }
}

Consumer.new().use(Dog.new())
```

Final assertions:

```rust
assert_eq!(state.agreement, AdvisoryAgreement::MoreSpecific);
assert_eq!(state.baseline.shape, ValueShape::Instance(animal_id));
assert_eq!(state.observed.as_ref().unwrap().shape, ValueShape::Instance(dog_id));
assert_eq!(state.effective.shape, ValueShape::Instance(dog_id));
```

- [ ] **Step 2: Add String/Int incompatibility test.**

Fixture:

```phalcom
class Consumer {
    use(value: String) {
        value
    }
}

Consumer.new().use(1)
```

Final advisory assertions:

```rust
assert_eq!(state.agreement, AdvisoryAgreement::Incompatible);
assert_eq!(state.effective.shape, state.baseline.shape);
assert_ne!(
    state.effective.shape,
    ValueShape::bounded_union([
        ValueShape::Instance(string_id),
        ValueShape::Instance(int_id),
    ])
);
```

Also assert the formal argument mismatch diagnostic exists through the existing checker path.

- [ ] **Step 3: Extend `AdvisoryAgreement`.**

Add:

```rust
Incompatible,
```

Keep `Incomparable` for unsupported representation relations.

- [ ] **Step 4: Add hierarchy-aware comparison API.**

In `agreement.rs` add:

```rust
pub fn compare_against_formal(
    store: &TypeStore,
    hierarchy: &dyn TypeHierarchy,
    formal: &TypeKnowledge,
    advisory: &AdvisoryFact,
) -> AdvisoryAgreement
```

Nominal rules:

```text
formal Instance(Animal), advisory Instance(Animal) → Compatible
formal Animal, advisory Dog where Dog <: Animal    → MoreSpecific
formal String, advisory Int with no subtype path   → Incompatible
```

Retain existing tuple/list/union refinement behavior.

For shapes that cannot be translated safely to formal nominal relations, return `Incomparable`, not `Incompatible`.

- [ ] **Step 5: Define effective-state policy.**

In `resolve_parameter_state`:

```text
no observation        → baseline
formal unknown        → observation if known, else baseline
Compatible            → observation
MoreSpecific          → observation
Incompatible          → baseline
Incomparable          → baseline unless formal baseline is Unknown
Unknown               → known observation if baseline Unknown, else baseline
```

Retain the incompatible observation in `state.observed` for explanations/testing.

- [ ] **Step 6: Ensure solver joins observations before constraint resolution.**

Caller contributions may join to `Dog | Cat`. Run agreement against the joined observation and formal baseline; do not separately overwrite state based on last caller order.

- [ ] **Step 7: Run advisory + formal suites.**

```bash
cargo test -p phalcom-semantic --test semantic integration::advisory_analysis -- --nocapture
cargo test -p phalcom-semantic --test semantic integration::contracts -- --nocapture
cargo test -p phalcom-semantic --test semantic -- --nocapture
```

- [ ] **Step 8: Commit.**

```bash
git add phalcom-semantic/src/advisory \
        phalcom-semantic/src/session.rs \
        phalcom-semantic/tests/semantic/integration
git commit -m "feat(semantic): refine advisory observations against contracts"
```

---

# Task 9 — Publish canonical field signatures and preserve field advisory flow

**Files:**
- Modify: `phalcom-semantic/src/signature.rs`
- Modify: `phalcom-semantic/src/snapshot.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/src/editor.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/contracts.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/workspace.rs`

**Interfaces:**
- Produces: snapshot-level `FieldSignatureTable` populated from declaration contract sets.

- [ ] **Step 1: Add snapshot field-contract test.**

For:

```phalcom
class User {
    _declared: String
    _observed
}
```

assert the canonical field signature table contains both field identities:

```text
_declared contract = Known(String), SourceAnnotation
_observed contract = Unknown(UnannotatedDeclaration or NoTypeEvidence)
```

- [ ] **Step 2: Add `base_field_signatures` only if native bootstrap exposes canonical field signatures.**

Do not fabricate native field contracts. If current native bootstrap has no field signature report, initialize source snapshot field table independently and leave base table empty.

- [ ] **Step 3: Add `field_signatures` to `SemanticSnapshot`.**

Modify all constructors/builders:

```rust
pub field_signatures: Arc<FieldSignatureTable>,
```

Update `new`, `new_with_callable_analyses`, and every test constructor.

- [ ] **Step 4: Collect source field signatures during session publication.**

When declaration contract sets are ready, insert each field contract into `FieldSignatureTable`.

Do not derive them from `DeclarationSurface`.

- [ ] **Step 5: Keep field advisory facts separate.**

Do not write advisory field `String` back into `FieldSignatureTable` for an unannotated field.

For `_name = name: String` the snapshot should show:

```text
field_signatures[_name].contract = Unknown
advisory.field(_name) = String
```

This distinction is a regression assertion.

- [ ] **Step 6: Run tests.**

```bash
cargo test -p phalcom-semantic --test semantic integration::contracts -- --nocapture
cargo test -p phalcom-semantic --test semantic integration::workspace -- --nocapture
```

- [ ] **Step 7: Commit.**

```bash
git add phalcom-semantic/src/signature.rs \
        phalcom-semantic/src/snapshot.rs \
        phalcom-semantic/src/session.rs \
        phalcom-semantic/tests/semantic/integration
git commit -m "feat(semantic): publish canonical field contracts"
```

---

# Task 10 — Add one compiler-owned type-hint query

**Files:**
- Modify: `phalcom-semantic/src/editor.rs`
- Modify: `phalcom-semantic/src/presentation.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/editor.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/presentation.rs`

**Interfaces:**
- Produces:
  - `EditorTypeHintOwner`;
  - `EditorTypeHint`;
  - `EditorSemanticQuery::type_hints`.

- [ ] **Step 1: Add editor tests covering singular ownership.**

Fixture:

```phalcom
class Probe {
    _field

    run(annotated: String, inferred) {
        let local = 1
        _field = annotated
        inferred
    }
}
```

Expected hint owners:

```text
annotated → none
inferred  → exactly Parameter(run, 1)
local     → exactly Binding(...)
_field    → Field(_field) with advisory String
return    → Return(run) if unannotated and useful
```

Assert no duplicate insert offsets for the same owner/declaration.

- [ ] **Step 2: Add editor hint types.**

Use the technical-spec definitions:

```rust
pub enum EditorTypeHintOwner {
    Binding(SourceSiteId),
    Parameter(CallableParameterId),
    Field(FieldId),
    Return(CallableId),
}

pub struct EditorTypeHint {
    pub owner: EditorTypeHintOwner,
    pub declaration_range: SourceRange,
    pub insert_offset: usize,
    pub formal: Option<FormalPresentation>,
    pub advisory: Option<AdvisoryFact>,
}
```

- [ ] **Step 3: Implement `type_hints`.**

Algorithm:

```text
source = source_index.module(module)
presenter = TypePresenter(store)

1. lexical binding hints:
   - skip Import
   - skip MethodParameter / SetterParameter / IndexParameter
   - skip annotation_range.is_some()
   - attach formal binding fact + advisory binding fact

2. field hints:
   - skip explicit annotation
   - formal from field signature contract when renderable
   - advisory from advisory workspace field fact

3. callable parameter hints:
   - iterate CallableSourceInfo.parameters
   - skip annotation_range.is_some()
   - formal from canonical parameter contract or attached formal binding
   - advisory from AdvisoryParameterState.effective

4. return hints:
   - skip return_annotation_range.is_some()
   - formal from canonical return contract
   - advisory from callable return fact
   - preserve the current compiler-owned placement using:
     `source.structure.callable_body_ranges.get(&callable.id).map_or(callable.declaration_range.end, |range| range.end)`
```

Sort by `insert_offset`, then stable owner ordering.

- [ ] **Step 4: Formal presentation of contracts.**

Add a pure helper in `presentation.rs`:

```rust
pub fn present_contract(
    contract: &TypeContract,
    presenter: &TypePresenter<'_>,
) -> FormalPresentation
```

Known canonical term → `Known`.
Dynamic → `Dynamic`.
Unknown → `Unknown`.
`SelfType`/`Infer` retain current unknown/special presentation policy.

- [ ] **Step 5: Ensure explicit annotation suppression is semantic.**

The editor query MUST NOT accept `Program`.

Add a test that calls `type_hints` using only snapshot/module/source range.

- [ ] **Step 6: Run editor tests.**

```bash
cargo test -p phalcom-semantic --test semantic integration::editor -- --nocapture
cargo test -p phalcom-semantic --test semantic integration::presentation -- --nocapture
```

- [ ] **Step 7: Commit.**

```bash
git add phalcom-semantic/src/editor.rs \
        phalcom-semantic/src/presentation.rs \
        phalcom-semantic/src/lib.rs \
        phalcom-semantic/tests/semantic/integration/editor.rs \
        phalcom-semantic/tests/semantic/integration/presentation.rs
git commit -m "feat(semantic): publish canonical editor type hints"
```

---

# Task 11 — Cut `phalcom-lsp` inlay hints over to the editor query

**Files:**
- Modify: `phalcom-lsp/src/inlay_hints.rs`
- Modify: `phalcom-lsp/src/presentation.rs`
- Modify: `phalcom-lsp/tests/stage6_inlay_hints.rs`
- Modify: `phalcom-lsp/tests/professional_semantic_presentation.rs`

**Interfaces:**
- Consumes: `EditorSemanticQuery::type_hints`.
- Produces: LSP-only rendering/policy.

- [ ] **Step 1: Promote all Task-0 LSP assertions.**

Make these required:

```text
annotated method parameter → zero hint
unannotated method parameter → exactly one hint
annotated let → zero hint
unannotated field assigned from String parameter → String hint when policy permits
```

- [ ] **Step 2: Replace canonical `hints_for_request` data gathering.**

Current canonical path passes:

```rust
&request.document.parse.program
```

into `canonical_hints_for_request`.

Remove `Program` from canonical semantic hint computation.

Target:

```rust
let semantic_hints = snapshot
    .editor()
    .type_hints(
        module,
        SourceRange {
            start: visible_start,
            end: visible_end,
        },
    );
```

Map each `EditorTypeHint` to `InlayHint`.

- [ ] **Step 3: Delete canonical `ExplicitAnnotationIndex` use.**

Remove from the canonical request path:

```text
ExplicitAnnotationIndex
collect_pattern_names
collect_statement_annotations
collect_expr_annotations
collect_pack_annotations
collect_product_label_annotations
```

If a temporary legacy fallback still compiles because the retirement plan has not yet deleted it, isolate the legacy code so the canonical request path cannot call it. The final single-world cleanup should delete the dead legacy path.

- [ ] **Step 4: Delete direct canonical source traversal.**

Remove the canonical loops over:

```text
source.structure.bindings
field_sources
callable_sources
```

and helpers:

```text
canonical_formal_for_binding
canonical_formal_for_term
```

once `EditorTypeHint` provides the same facts.

- [ ] **Step 5: Preserve policy in LSP.**

For each semantic hint:

1. if formal is renderable (`Known`/`Dynamic`), render formal;
2. otherwise inspect advisory:
   - suppress `Unknown`;
   - suppress `Heuristic` under `HintPolicy::Stable`;
3. apply `suppress_obvious` only as LSP presentation policy using declaration range/text if desired.

Do not convert `suppress_obvious` into semantic inference.

- [ ] **Step 6: Fix formal return formatting.**

Use:

```rust
crate::presentation::inlay_type_label(&text, return_hint)
```

for both formal and advisory branches.

This fixes the current formal `: T` return-label inconsistency.

- [ ] **Step 7: Run LSP tests.**

```bash
cargo test -p phalcom-lsp --test stage6_inlay_hints -- --nocapture
cargo test -p phalcom-lsp --test professional_semantic_presentation -- --nocapture
cargo check -p phalcom-lsp
```

- [ ] **Step 8: Commit.**

```bash
git add phalcom-lsp/src/inlay_hints.rs \
        phalcom-lsp/src/presentation.rs \
        phalcom-lsp/tests/stage6_inlay_hints.rs \
        phalcom-lsp/tests/professional_semantic_presentation.rs
git commit -m "refactor(lsp): render canonical semantic type hints"
```

---

# Task 12 — Correct incremental dependencies and fingerprints

**Files:**
- Modify: `phalcom-semantic/src/db/key.rs`
- Modify: `phalcom-semantic/src/db/fingerprint.rs`
- Modify: `phalcom-semantic/src/db/query.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/workspace.rs`

**Interfaces:**
- Verifies contract edits invalidate the correct closure and no more.

- [ ] **Step 1: Add parameter annotation edit test.**

Revision 1:

```phalcom
class Probe {
    run(value: String) {
        value
    }
}
```

Revision 2:

```phalcom
class Probe {
    run(value: Int) {
        value
    }
}
```

Assert:

```text
DeclarationContracts(Probe) recomputed
CallableSignature(run) recomputed
CallableBody(run) recomputed
SourceFormalAttachment(run) refreshed if formal product changed
AdvisoryCallable(run) refreshed
AdvisoryModule(module) refreshed
```

Also include an unrelated callable in the same/another declaration and assert it is reused when its dependency product fingerprint is unchanged.

- [ ] **Step 2: Add presentation-only movement test.**

Move whitespace without changing annotation/contract semantics.

Assert canonical contract product fingerprint/reuse remains stable where the current DB design permits it, while source/presentation fingerprint changes.

- [ ] **Step 3: Check dependency graph.**

Ensure `query_callable_body_with_formal_inputs` records `CallableSignature` dependency regardless of completeness.

Ensure `query_declaration_surface` records `DeclarationContracts` rather than becoming a peer source of signature truth.

- [ ] **Step 4: Hash new advisory parameter state.**

Update advisory fingerprints to include:

```text
CallableParameterId
baseline
observed
effective
agreement
```

Deterministic ordering is required.

- [ ] **Step 5: Run workspace/incremental tests.**

```bash
cargo test -p phalcom-semantic --test semantic integration::workspace -- --nocapture
cargo test -p phalcom-semantic --test semantic incremental -- --nocapture
```

- [ ] **Step 6: Commit.**

```bash
git add phalcom-semantic/src/db \
        phalcom-semantic/src/session.rs \
        phalcom-semantic/tests/semantic/integration/workspace.rs
git commit -m "fix(semantic): track canonical contract dependencies"
```

---

# Task 13 — Delete obsolete reverse-authority and duplicate-representation paths

**Files:**
- Modify/Delete code in:
  - `phalcom-semantic/src/db/query.rs`
  - `phalcom-semantic/src/checker/body.rs`
  - `phalcom-semantic/src/checker/context.rs`
  - `phalcom-semantic/src/dispatch.rs`
  - `phalcom-semantic/src/advisory/parameters.rs`
  - `phalcom-lsp/src/inlay_hints.rs`

**Interfaces:**
- Removes transitional APIs after all consumers are migrated.

- [ ] **Step 1: Search for prohibited old symbols.**

Run:

```bash
rg \
  "semantic_signature_from_surface|BindingContractOrigin|AdvisoryParameterSlot|parameter_name_ranges|has_explicit_return_annotation|ExplicitAnnotationIndex" \
  phalcom-semantic phalcom-lsp
```

Expected after cleanup: no production references to these symbols.

- [ ] **Step 2: Remove `semantic_signature_from_surface`.**

Delete the function and any tests that only validate surface→semantic reconstruction.

Replace such tests with canonical-contract→surface projection tests.

- [ ] **Step 3: Remove complete-signature publication gates.**

Run:

```bash
rg "has_complete_types" phalcom-semantic/src
```

Any remaining use must be informational or a real consumer that does not gate `CallableSemanticSignature` existence/body dependencies.

If no legitimate consumer remains, remove `dispatch::CallableSignature::has_complete_types`.

- [ ] **Step 4: Remove body dispatch-signature fallback.**

Delete `signature_consumed_by_body` if no noncanonical caller remains.

Constructor normalization should now resolve a canonical signature ID directly.

- [ ] **Step 5: Remove advisory slot compatibility types.**

No type alias from `AdvisoryParameterSlot` to `CallableParameterId`.

- [ ] **Step 6: Remove LSP canonical AST annotation scanner.**

After single-world/legacy cleanup permits, delete the recursive scanner and unused AST imports from `inlay_hints.rs`.

- [ ] **Step 7: Run compile checks.**

```bash
cargo check --workspace
```

- [ ] **Step 8: Commit.**

```bash
git add phalcom-semantic phalcom-lsp
git commit -m "refactor(semantic): remove reverse declaration authority"
```

---

# Task 14 — End-to-end regression matrix

**Files:**
- Modify: `phalcom-semantic/tests/semantic/integration/contracts.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/advisory_analysis.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/editor.rs`
- Modify: `phalcom-lsp/tests/stage6_inlay_hints.rs`
- Modify: `phalcom-lsp/tests/professional_semantic_presentation.rs`

**Interfaces:**
- Validates every normative scenario in the spec.

- [ ] **Step 1: Add/confirm S1 — unannotated parameter + caller evidence.**

Expected formal unknown, advisory caller-derived known, one useful hint.

- [ ] **Step 2: Add/confirm S2 — annotated parameter without callers.**

Expected source contract, formal assumption, advisory baseline, zero parameter hint.

- [ ] **Step 3: Add/confirm S3 — Animal contract + Dog observation.**

Expected formal `Animal`, advisory effective `Dog`, `MoreSpecific`.

- [ ] **Step 4: Add/confirm S4 — incompatible call.**

Expected one formal mismatch; advisory effective does not union incompatible shape.

- [ ] **Step 5: Add/confirm S5 — parameter → field.**

Expected `_name` advisory `String`, parameter no hint, field hint allowed.

- [ ] **Step 6: Add/confirm S6 — partial callable.**

Expected canonical signature exists despite unknown return.

- [ ] **Step 7: Add/confirm S7 — annotated local with narrower initializer.**

Expected no ordinary hint; contract/current/advisory facts retained.

- [ ] **Step 8: Add/confirm S8/S9 — destructuring.**

Unannotated leaf hints remain; annotated leaf hints suppressed.

- [ ] **Step 9: Add/confirm S10 — setter/index parameters.**

Verify identity, contract basis, source metadata, body binding, advisory baseline, and hint suppression.

- [ ] **Step 10: Run all targeted suites.**

```bash
cargo test -p phalcom-semantic --test semantic -- --nocapture
cargo test -p phalcom-lsp --test stage6_inlay_hints -- --nocapture
cargo test -p phalcom-lsp --test professional_semantic_presentation -- --nocapture
```

- [ ] **Step 11: Commit.**

```bash
git add phalcom-semantic/tests phalcom-lsp/tests
git commit -m "test(semantic): cover canonical declaration evidence flow"
```

---

# Task 15 — Final verification and architecture gate

**Files:**
- Modify: existing semantic/LSP architecture boundary tests if present.
- Modify: `.agents/skills/phalcom-semantic-model/references/current-implementation-map.md` if that file still describes complete-only signatures or LSP annotation ownership.
- Modify: Part 3 checklist only after code passes.

- [ ] **Step 1: Verify prohibited symbols are gone.**

```bash
rg \
  "semantic_signature_from_surface|BindingContractOrigin|AdvisoryParameterSlot|ExplicitAnnotationIndex" \
  phalcom-semantic phalcom-lsp
```

Expected: zero production hits.

- [ ] **Step 2: Verify canonical authority direction mechanically.**

Search:

```bash
rg "CallableSemanticSignature" phalcom-semantic/src
rg "DeclarationContracts" phalcom-semantic/src
rg "get_callable\\(" phalcom-semantic/src/db phalcom-semantic/src/checker
```

Review every remaining `DeclarationSurface::get_callable` use. Dispatch resolution is allowed. Canonical signature publication/body contract acquisition from surfaces is not.

- [ ] **Step 3: Verify LSP boundary.**

```bash
rg \
  "collect_statement_annotations|collect_pattern_names|has_explicit_return_annotation|parameter_name_ranges" \
  phalcom-lsp/src/inlay_hints.rs
```

Expected: zero canonical semantic-path hits; after legacy deletion, zero hits total.

- [ ] **Step 4: Run formatting.**

```bash
cargo fmt --all -- --check
```

If it fails:

```bash
cargo fmt --all
cargo fmt --all -- --check
```

- [ ] **Step 5: Run semantic crate suite.**

```bash
cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-semantic --lib
```

Expected: PASS.

- [ ] **Step 6: Run LSP focused suite.**

```bash
cargo test -p phalcom-lsp --test stage6_inlay_hints
cargo test -p phalcom-lsp --test professional_semantic_presentation
cargo test -p phalcom-lsp --test semantic_boundary
```

Expected: PASS.

- [ ] **Step 7: Run workspace check and broader tests.**

```bash
cargo check --workspace
cargo test --workspace
```

Expected: PASS. If unrelated pre-existing failures exist, record exact test names/output and prove the targeted suites are green before classifying them as unrelated.

- [ ] **Step 8: Review incremental statistics assertions.**

Confirm the annotation edit test does not rebuild unrelated callables and that source-only movement does not invalidate semantic contract products unnecessarily.

- [ ] **Step 9: Update architecture documentation.**

Record the final ownership chain:

```text
DeclarationContracts
  → Callable/Field semantic signatures
  → dispatch projection
  → formal body analysis
  → formal source attachment
  → advisory baseline/observations
  → editor query
  → LSP
```

Remove descriptions that claim canonical signatures are complete-only or that LSP owns annotation suppression.

- [ ] **Step 10: Final commit.**

```bash
git add .
git commit -m "docs(semantic): close canonical contract consolidation"
```

---

# Verification Checklist Against the Technical Spec

The implementation is not complete until every row is checked.

| Spec requirement | Implementation task |
|---|---|
| canonical parameter identity | Tasks 1, 2 |
| role/provenance orthogonality | Task 5 |
| contracts separate from `TypeKnowledge` | Tasks 1, 5 |
| partial canonical signatures | Tasks 2, 3 |
| dispatch is projection | Task 3 |
| canonical-first inferred returns | Task 3 |
| exact parameter source metadata | Task 4 |
| actual parameter body range | Task 5 |
| identity-first source attachment | Task 6 |
| formal→advisory baseline | Task 7 |
| hierarchy-aware refinement | Task 8 |
| incompatible observation containment | Task 8 |
| parameter→field advisory flow | Tasks 7, 9 |
| field formal/advisory separation | Task 9 |
| semantic hint ownership | Task 10 |
| LSP annotation scanner removal | Task 11 |
| incremental dependency correctness | Task 12 |
| obsolete adapters deleted | Task 13 |
| S1–S10 acceptance matrix | Task 14 |
| architecture/prohibited-symbol gate | Task 15 |

---

# Self-Review Performed on This Plan

The plan was checked against the technical specification for:

1. **Spec coverage:** every normative goal G1–G12 and scenario S1–S10 maps to at least one implementation task.
2. **Authority direction:** no planned step allows advisory facts to become formal evidence; no final production step rebuilds canonical contracts from dispatch.
3. **Type consistency:** the same names are used throughout:
   - `CallableParameterId`
   - `ContractType`
   - `ContractBasis`
   - `TypeContract`
   - `DeclarationContractSet`
   - `BindingRole`
   - `ResolvedBindingContract`
   - `AdvisoryParameterState`
   - `EditorTypeHint`
4. **Migration closure:** every intentionally transitional path has a deletion task.
5. **Presentation ownership:** method/setter/index parameters are excluded from generic binding hints and owned by canonical parameter identity.
6. **Field semantics:** field advisory inference remains advisory and is not promoted into canonical field contract tables.
7. **Incrementality:** the new `DeclarationContracts` product is included in DB key/product/fingerprint/dependency work.
8. **Constructor behavior:** class-side constructor signature versus instance-side constructor body normalization remains explicit.
9. **No placeholder scan:** the implementation steps contain concrete target symbols, behavior, tests, and commands rather than unresolved placeholder instructions.

If the repository moves beyond grounded HEAD before execution, Task 0 must be repeated and line/function anchors amended before code changes begin.
