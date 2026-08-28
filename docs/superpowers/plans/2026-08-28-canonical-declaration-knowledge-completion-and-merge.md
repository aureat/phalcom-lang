# Canonical Declaration Knowledge Completion and Merge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish the canonical declaration-knowledge cutover on `codex/canonical-declaration-knowledge`, prove that callable/parameter/field/native/editor type information has one compiler-owned authority, make the repository merge gates trustworthy, and merge PR #6 into `main` only after an exact branch SHA passes the full semantic/LSP verification matrix.

**Architecture:** `CallableSignatureTable` and `FieldSignatureTable` are declaration-owned semantic authorities. Source declarations and native metadata publish canonical signatures first. `DeclarationSurface` / `SurfaceDispatchResolver` are derived lookup projections used for selector/member resolution, never semantic reconstruction authorities. Formal checker products are authoritative when established; advisory products may fill editor/runtime-shape gaps only when formal knowledge is unavailable and must never flow backward into formal declaration truth. `phalcom-lsp` consumes protocol-neutral compiler presentation/query products from one immutable `SemanticSnapshot` and must not reimplement semantic inference.

**Tech Stack:** Rust 2024, Cargo workspace, `phalcom-semantic`, `phalcom-lsp`, `phalcom-modules`, GitHub Actions, canonical semantic integration harness `phalcom-semantic/tests/semantic.rs`, LSP integration tests, VS Code extension E2E.

**Spec:** Repository-grounded completion of PR #6 (`Canonical declaration knowledge consolidation`), constrained by `.agents/skills/semantic-analysis-development/references/current-architecture.md`, `.agents/skills/phalcom-semantic-model/references/current-implementation-map.md`, `docs/impl/semantic/semantic-correctness/part-4/phalcom-lsp-semantic-retirement-closure.md`, and `docs/impl/semantic/semantic-completeness/part-1/01.5-canonical-generic-type-semantics-and-declaration-model.md`.

**Verified baseline before this plan commit:**

```text
main                                  fb29fe01e48fe7167bcb0a2ef8025aec81a545f9
codex/canonical-declaration-knowledge da65512e38a31c7ccc65574691c5ed787ebabee0
PR                                    #6, open draft, base=main
Phase 4 GREEN run                     33158151673
Phase 4 GREEN failure                 canonical body-signature lookup not installed at all intended authority sites
```

The plan commit itself advances the branch; all implementation tasks must re-fetch the branch and `main` before making current-state claims.

## Global Constraints

- [ ] Never reconstruct declaration semantics from `DeclarationSurface`, `SurfaceDispatchResolver`, LSP syntax scans, rendered strings, or advisory products when a canonical signature product exists.
- [ ] Keep dispatch as the selector/member lookup mechanism. This plan removes dispatch **authority**, not dispatch resolution.
- [ ] Constructor declaration identity remains class-side. Constructor body analysis may remain instance-side internally, but every declaration/parameter/public signature lookup must normalize through canonical callable identity.
- [ ] `CallableParameterId` is the sole parameter identity. Do not reintroduce `{ callable, index }` duplicates, source-range ordering, or parameter-name identity.
- [ ] `Unknown`, `Dynamic`, `Self`, `Unit`, `Never`, inferred return knowledge, and source annotations remain distinct semantic states; do not collapse them for presentation convenience.
- [ ] Formal knowledge wins over advisory knowledge. Advisory facts are fallback presentation/runtime-shape observations only.
- [ ] Every behavior/ownership change starts from a failing regression or an already-captured valid RED regression. Do not weaken a semantic assertion to get green.
- [ ] Use the existing `phalcom-semantic --test semantic` integration harness for semantic behavior unless a lower-level unit test is specifically more appropriate.
- [ ] Do not create new top-level test binaries without registering them. `phalcom-semantic` and `phalcom-lsp` both have non-default test wiring constraints.
- [ ] Remove temporary self-modifying transformation workflows/scripts after the phase they serve. Permanent acceptance workflows must be read-only.
- [ ] Hosted verification commands that use stable Rust must override repository rustflags with `RUSTFLAGS=""`, because `.cargo/config.toml` currently injects nightly-only `-Zthreads=2` and `target-cpu=native`.
- [ ] PR #6 stays draft until Tasks 1–9 are complete and the branch is synchronized with current `main`.
- [ ] Never merge by an unpinned PR head. Final merge must use the exact SHA that passed the acceptance workflow.

---

# 1. Current Implementation Map and Remaining Defects

| Area | Current canonical product | Remaining defect | Primary files |
|---|---|---|---|
| Source callable declaration | `CallableSemanticSignature` / `CallableSignatureTable` | Phase 4 reverse reads still exist in `session.rs` | `phalcom-semantic/src/session.rs`, `signature.rs`, `checker/declaration_signature.rs` |
| Constructor parameter identity | `CallableParameterId` | Phase 3 complete; preserve regressions | `advisory/flow.rs`, `session.rs`, `tests/canonical_parameter_advisory.rs` |
| Inferred callable result | `CallableSemanticSignature::inferred_return` | Fixed-point still consults/mutates dispatch before canonical signature | `session.rs`, `dispatch.rs` |
| Native callable declaration | `CallableSemanticSignature` exists in import report | Native importer constructs dispatch first, then reconstructs canonical signature from surface | `types/native.rs` |
| Source fields | `FieldSemanticSignature` / `FieldSignatureTable` types exist | Table is not populated/published; fields still go source syntax -> dispatch directly | `signature.rs`, `checker/declaration.rs`, `session.rs`, `snapshot.rs` |
| Advisory field/callable presentation | `AdvisoryWorkspace` | Some formal baselines still come from dispatch/body reconstruction | `session.rs`, advisory modules |
| Protocol-neutral callable presentation | `CallablePresentation` | LSP adapters still reach stale/raw canonical fields and duplicate formatting logic | `presentation.rs`, `editor.rs` |
| Inlay hints | compiler snapshot | Field hints are advisory-only; callable parameter/return code uses stale signature member shapes | `phalcom-lsp/src/inlay_hints.rs` |
| Signature help | compiler snapshot | Uses stale `parameter.ty` / `signature.return_type` access and locally recomposes formal/advisory policy | `phalcom-lsp/src/signature_help.rs` |
| Hover | compiler target/query | Signature rendering still has LSP-local composition paths | `phalcom-lsp/src/hover.rs`, `backend.rs` |
| Merge CI | GitHub Actions | Stable jobs invoke plain `cargo`; repo toolchain/rustflags cause nightly override/SIGILL on hosted CPU | `.github/workflows/ci.yml`, `.cargo/config.toml`, `rust-toolchain.toml` |

Execution order is deliberate:

```text
Task 1  Complete Phase 4 callable authority
Task 2  Make native declarations canonical-first
Task 3  Publish canonical source field signatures
Task 4  Route field checking/advisory through canonical field facts
Task 5  Consolidate protocol-neutral presentation
Task 6  Migrate LSP hover/signature-help/inlays to canonical presentation
Task 7  Add mechanical authority/boundary audits
Task 8  Repair hosted CI portability
Task 9  Remove temporary phase machinery and update architecture docs
Task 10 Synchronize with main and run exact-SHA acceptance
Task 11 Review PR #6 and merge exact verified head
Task 12 Verify main after merge and close temporary RED artifacts
```

---

# 2. Task 1 — Complete Phase 4: Callable Signatures Own Return and Constructor Semantics

**Deliverable:** `CallableSignatureTable` is the only semantic source for declared/published callable return knowledge and constructor classification. Dispatch remains a derived projection used by call/member lookup.

**Files:**
- Modify: `.github/scripts/canonical_callable_phase4.py`
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/tests/semantic/incremental/declaration_authority.rs`
- Verify: `phalcom-semantic/tests/canonical_parameter_advisory.rs`
- Delete after GREEN: `.github/scripts/canonical_callable_phase4.py`
- Delete after GREEN: `.github/workflows/canonical-callable-phase4-green.yml`

**Existing RED evidence:** The Phase 4 authority regression compiled under nightly and failed specifically because advisory formal return knowledge read `DeclarationSurface` instead of `CallableSignatureTable`. Do not recreate a different RED test merely for ceremony; preserve and turn that exact regression green.

## Step 1.1 — Re-fetch and diagnose the failed transformation guard

- [ ] Re-fetch branch and ensure Phase 4 production edits were not committed by failed run `33158151673`.
- [ ] Inspect the four intended canonical lookup sites in `session.rs`:
  1. constructor field-lifecycle classification;
  2. advisory formal-return seeding;
  3. inferred-return fixed-point candidate selection;
  4. body recheck declaration-signature lookup.
- [ ] Do **not** fix the workflow by changing `text.count("callable_signatures.get_for_body") < 4` to `< 3`.
- [ ] Replace the raw count guard with named, structural guards. The transformation script should validate each intended semantic replacement independently:

```python
required_after = {
    "field lifecycle constructor authority": "callable_signatures\n                                                            .get_for_body(&analysis.callable)",
    "advisory formal return authority": ".get_for_body(&analysis.callable)\n            .map(|signature|",
    "inferred return authority": "let signature = callable_signatures.get_for_body(callable)?;",
    "body recheck authority": "let declared_signature = callable_signatures\n                        .get_for_body(&callable)",
}
for label, snippet in required_after.items():
    if snippet not in text:
        raise SystemExit(f"missing canonical authority site: {label}")
```

If formatting makes one exact snippet inappropriate, validate the smallest unique semantic fragment for that site; still keep four named checks.

## Step 1.2 — Apply the production direction change

- [ ] In `build_advisory_workspace`, add `callable_signatures: &CallableSignatureTable` as an explicit input.
- [ ] Change formal-call-result projection from:

```rust
let signature = dispatch
    .get_surface(&callable.owner)?
    .get_callable(callable.side, &callable.selector)?;
```

into canonical lookup:

```rust
let signature = callable_signatures.get(callable)?;
let return_knowledge = signature.published_return_knowledge();
```

- [ ] Preserve receiver-relative `Self` specialization by passing `return_knowledge` through `advisory_shape_from_formal_for_receiver` when a receiver exists.
- [ ] Change constructor transfer classification to:

```rust
let is_constructor = callable_signatures
    .get(callable)
    .is_some_and(CallableSemanticSignature::is_constructor);
```

Use the existing method form that compiles; do not rederive constructor-ness from selector spelling or attributes.

- [ ] Seed advisory formal returns from `callable_signatures.get_for_body(&analysis.callable).published_return_knowledge()`; remove `advisory_return_fact`, which reconstructs the baseline from body exits.
- [ ] Change constructor field-lifecycle filtering to `callable_signatures.get_for_body(&analysis.callable).is_constructor()`.
- [ ] Change inferred-return fixed-point logic so canonical publication happens first:

```rust
let Some(signature) = callable_signatures.get_mut(&signature_id) else {
    continue;
};
if signature.inferred_return.as_ref() == Some(&summary) {
    continue;
}
signature.inferred_return = Some(summary.clone());
changed_callables.insert(callable.clone());

// Projection refresh only; failure here must not suppress canonical publication.
let _ = dispatch.update_callable_return_type(&signature_id, summary);
```

- [ ] Replace the handwritten class-side fallback in body rechecking with `callable_signatures.get_for_body(&callable)`.
- [ ] Remove the now-unused `AdvisoryParameterSlot` import from `session.rs`.

## Step 1.3 — Strengthen the authority regression

- [ ] Keep the existing `callable_signature_query_never_reconstructs_semantics_from_dispatch_surface` test.
- [ ] Make the Phase 4 source-level guard assert that the specific forbidden patterns are absent from the relevant `session.rs` functions, not that the word `dispatch` is globally absent. Dispatch lookup is still valid.
- [ ] Explicitly forbid these reverse-authority patterns:

```text
resolve_formal_call_result -> dispatch.get_surface(...).get_callable(...)
advisory_transfer_target -> CallableSemanticKind::Constructor from dispatch
refresh_inferred_callable_results -> dispatch return_type used to decide canonical unknownness
constructor lifecycle -> dispatch kind used to classify constructor
```

## Step 1.4 — Verify Phase 4

Run from repository root:

```bash
cargo +stable fmt --all -- --check
RUSTFLAGS="" cargo +stable check -p phalcom-semantic
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic \
  semantic::incremental::declaration_authority::advisory_and_return_refresh_read_canonical_signatures_not_dispatch_surfaces -- --exact
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic \
  canonical_parameter_advisory::constructor_argument_transfer_uses_public_canonical_parameter_identity -- --exact
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic
git diff --check
```

- [ ] All pass.
- [ ] Review `git diff` and confirm production semantic changes are limited to the intended authority seam plus regression code.

## Step 1.5 — Commit and remove transformation machinery

Commit production first:

```bash
git add phalcom-semantic/src/session.rs \
        phalcom-semantic/tests/semantic/incremental/declaration_authority.rs
git commit -m "refactor(semantic): make callable signatures authoritative"
```

Then remove the temporary self-modifying machinery:

```bash
git rm .github/scripts/canonical_callable_phase4.py \
       .github/workflows/canonical-callable-phase4-green.yml
git commit -m "ci: remove completed phase 4 transformation driver"
```

**Stop condition:** Do not begin Task 2 until the complete semantic suite is green on the post-cleanup branch SHA.

---

# 3. Task 2 — Phase 5: Make Native Metadata Publish Canonical Signatures First

**Deliverable:** `register_native_surfaces` builds one `CallableSemanticSignature` directly from native metadata, stores it in `NativeSurfaceImportReport`, and derives dispatch `CallableSignature` only by projecting that canonical signature.

**Files:**
- Modify: `phalcom-semantic/src/types/native.rs`
- Reuse: `phalcom-semantic/src/checker/declaration_signature.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/declaration_knowledge.rs`
- Modify: `phalcom-semantic/tests/semantic/incremental/declaration_authority.rs`

## Step 2.1 — Add RED tests

Add two tests.

First, a behavior test in `integration/declaration_knowledge.rs` that selects a known native surface and asserts:

```rust
let signature = snapshot
    .callable_signatures
    .get(&callable)
    .expect("native callable has canonical signature");

assert_eq!(signature.implementation, ImplementationKind::NativePrimitive);
assert!(signature.native_id.is_some());
assert_eq!(signature.callable, callable);
```

Then compare the dispatch projection to the canonical projection:

```rust
let projected = phalcom_semantic::checker::declaration_signature::project_semantic_signature(signature);
let dispatch = snapshot
    .surfaces
    .get(&callable.owner)
    .and_then(|surface| surface.get_callable(callable.side, &callable.selector))
    .expect("native dispatch projection");
assert_eq!(&projected, dispatch);
```

If `project_semantic_signature` is currently crate-private, put this equality test inside `types/native.rs` as a unit test rather than widening visibility solely for the test.

Second, add an ownership guard that fails while `register_native_surfaces` contains canonical reconstruction from:

```text
surface.get_callable(...)
signature.parameters -> CallableParameterSemantic
signature.return_type -> DeclaredTypeFact
```

## Step 2.2 — Build native canonical parameters directly

- [ ] Keep `ParameterTupleSpec` metadata as source truth.
- [ ] For positional native params, preserve the current generated local names (`other` for first positional, `arg` thereafter) because native metadata does not currently carry local parameter names.
- [ ] For labeled params, set both local name and `external_label` from `labeled.label`.
- [ ] `RestParameterSpec` has no positional-vs-labeled discriminator. Represent it canonically as `RestMode::Complete`; do not invent `Positional` or `Labeled` semantics.
- [ ] Build each parameter with canonical identity immediately:

```rust
let id = CallableParameterId::new(callable_id.clone(), index as u32);
let declared_type = DeclaredTypeFact::from_knowledge_with_basis(
    &knowledge,
    DeclaredTypeBasis::NativeSignature,
);
let parameter = CallableParameterSemantic::new(id, local_name, declared_type);
```

Then apply `.with_label(...)` and `.with_rest(RestMode::Complete)` where required.

## Step 2.3 — Build native canonical return and metadata directly

Construct:

```rust
let semantic = CallableSemanticSignature {
    callable: callable_id.clone(),
    owner: callable_id.owner.clone(),
    side,
    selector: selector.clone(),
    generics: None,
    parameters: parameters.into_boxed_slice(),
    declared_return: DeclaredTypeFact::from_knowledge_with_basis(
        &ret_knowledge,
        DeclaredTypeBasis::NativeSignature,
    ),
    inferred_return: None,
    source: None,
    implementation: ImplementationKind::NativePrimitive,
    native_id: Some(record.id()),
    effects: record.effects(),
    raises: record.raises(),
    flow: record.flow(),
    lifecycle: record.lifecycle(),
};
```

- [ ] Push `semantic.clone()` into `report.callable_signatures`.
- [ ] Derive dispatch from `project_semantic_signature(&semantic)` and add that projection to `surfaces_by_decl`.
- [ ] Delete the post-surface reconstruction block entirely.

## Step 2.4 — Verify native canonical-first behavior

```bash
cargo +stable fmt --all -- --check
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic \
  semantic::integration::declaration_knowledge
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic \
  semantic::incremental::declaration_authority
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic
git diff --check
```

Commit:

```bash
git add phalcom-semantic/src/types/native.rs \
        phalcom-semantic/tests/semantic/integration/declaration_knowledge.rs \
        phalcom-semantic/tests/semantic/incremental/declaration_authority.rs
git commit -m "refactor(semantic): publish native callable signatures canonically"
```

---

# 4. Task 3 — Phase 6A: Publish Canonical Source Field Signatures

**Deliverable:** Every source field declaration publishes a `FieldSemanticSignature` even when its declared type is unknown. `SemanticSnapshot` owns a `FieldSignatureTable` parallel to `CallableSignatureTable`.

**Files:**
- Modify: `phalcom-semantic/src/signature.rs`
- Create: `phalcom-semantic/src/checker/field_signature.rs`
- Modify: `phalcom-semantic/src/checker/mod.rs`
- Modify: `phalcom-semantic/src/checker/declaration.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/src/snapshot.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Modify tests: `phalcom-semantic/tests/semantic/integration/declaration_knowledge.rs`
- Add test module only if needed: `phalcom-semantic/tests/semantic/integration/field_knowledge.rs`, registered in `integration/mod.rs`

## Step 3.1 — Write RED field-publication tests

Use source containing both annotated and unannotated fields:

```phalcom
class Box {
    value: Int
    inferred
}
```

Assert:

```rust
let annotated = FieldId::new(owner.clone(), "value", DispatchSide::Instance);
let unknown = FieldId::new(owner.clone(), "inferred", DispatchSide::Instance);

let annotated_sig = snapshot.field_signatures.get(&annotated).expect("annotated field signature");
let unknown_sig = snapshot.field_signatures.get(&unknown).expect("partial field signature");

assert!(annotated_sig.declared_type.is_known());
assert!(!unknown_sig.declared_type.is_known());
```

Also add a class-side field case using the current `@class`/static placement syntax accepted by the parser and assert the `FieldId.side` is `DispatchSide::Class`.

Expected RED: `SemanticSnapshot` does not have `field_signatures` and source registration does not build a field table.

## Step 3.2 — Add canonical field construction

Create `checker/field_signature.rs` with one source-to-semantic builder. Keep syntax/range concerns there; do not put them in `FieldSignatureTable`.

The central interface should be:

```rust
pub(crate) fn semantic_signature_for_field(
    ctx: &mut CheckingContext<'_>,
    owner: &DeclarationId,
    field: &phalcom_ast::ast::FieldDef,
    side: DispatchSide,
    resolver: &dyn TypeResolver,
) -> FieldSemanticSignature
```

It must:

```rust
let field_id = FieldId::new(owner.clone(), field.name.clone(), side);
let declared_type = match field.annotation.as_ref() {
    Some(annotation) => {
        let (knowledge, _) = ctx.resolve_type_annotation(resolver, annotation);
        DeclaredTypeFact::from_knowledge_with_basis(
            &knowledge,
            DeclaredTypeBasis::SourceAnnotation,
        )
    }
    None => DeclaredTypeFact::unknown(UnknownReason::UnannotatedDeclaration),
};

FieldSemanticSignature {
    field: field_id,
    owner: owner.clone(),
    side,
    name: field.name.clone().into_boxed_str(),
    mutable: true,
    declared_type,
    source: Some(SemanticSourceSpan::new(ctx.current_module.clone(), field.name_range)),
}
```

Use the actual field mutability semantics already represented by the AST/language if a non-mutable field form exists. If all current class fields are mutable implementation fields, keep `mutable: true`; do not infer mutability from naming convention.

## Step 3.3 — Add field projection helper

Add next to callable projection:

```rust
pub(crate) fn project_field_semantic_signature(
    signature: &FieldSemanticSignature,
) -> TypeKnowledge {
    signature.declared_type.to_knowledge()
}
```

This helper is intentionally small. Visibility remains source/member metadata and may still be attached while installing the projection.

## Step 3.4 — Populate a field table during source registration

- [ ] Add `FieldSignatureTable` alongside `CallableSignatureTable` in workspace update state.
- [ ] Seed it from a `base_field_signatures` table if native/core metadata later provides canonical fields; otherwise initialize the base table empty now.
- [ ] When source class members are registered, build the `FieldSemanticSignature` once, insert it into the field table, then project its `declared_type` to the dispatch/member surface.
- [ ] Remove the direct source annotation -> `surface.add_field_with_visibility` authority path in `register_class_surface`.

Do not duplicate annotation resolution in `session.rs`. The field builder is the one source declaration lowering boundary.

## Step 3.5 — Publish field signatures in snapshots

Add:

```rust
pub field_signatures: Arc<FieldSignatureTable>,
```

to `SemanticSnapshot`, and thread it through:
- `SemanticSnapshot::new`
- `SemanticSnapshot::new_with_callable_analyses`
- fallback/last-known-good snapshot construction in `SemanticWorkspaceSession`
- normal publication in `session.rs`

Keep ordering parallel to `callable_signatures` to make ownership obvious.

## Step 3.6 — Verify field publication

```bash
cargo +stable fmt --all -- --check
RUSTFLAGS="" cargo +stable check -p phalcom-semantic
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic \
  semantic::integration::declaration_knowledge
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic
git diff --check
```

Commit:

```bash
git add phalcom-semantic/src/signature.rs \
        phalcom-semantic/src/checker/field_signature.rs \
        phalcom-semantic/src/checker/mod.rs \
        phalcom-semantic/src/checker/declaration.rs \
        phalcom-semantic/src/session.rs \
        phalcom-semantic/src/snapshot.rs \
        phalcom-semantic/src/lib.rs \
        phalcom-semantic/tests/semantic/integration/declaration_knowledge.rs
git commit -m "feat(semantic): publish canonical field signatures"
```

---

# 5. Task 4 — Phase 6B: Make Field Checking and Advisory Projection Consume Canonical Field Facts

**Deliverable:** Field initializer diagnostics and advisory field baselines consume `FieldSignatureTable`; `DeclarationSurface` is only a member lookup projection.

**Files:**
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/declaration.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Modify as required: `phalcom-semantic/src/advisory/*`
- Tests: `phalcom-semantic/tests/semantic/integration/declaration_knowledge.rs`
- Tests: `phalcom-semantic/tests/semantic/integration/advisory_analysis.rs`
- Authority guard: `phalcom-semantic/tests/semantic/incremental/declaration_authority.rs`

## Step 4.1 — Add RED mismatch/authority tests

Use:

```phalcom
class Box {
    value: String = 1
}
```

Assert all of the following simultaneously:
- canonical field signature remains `String`;
- initializer fact remains `Int`/its actual formal result;
- `FieldMismatch` diagnostic is emitted;
- advisory field observation does not replace canonical declared type;
- dispatch field projection still says `String`.

Add an unknown-field case:

```phalcom
class Box {
    value = 1
}
```

Assert the canonical declared field type remains `Unknown`; any advisory `Int` observation is advisory-only and does not mutate `FieldSignatureTable`.

## Step 4.2 — Give checking context read access to canonical field table

Thread `&FieldSignatureTable` through the formal checking context at the same ownership level as callable signatures. Avoid a mutable field-signature borrow during body checking; declaration publication is complete before consumers read it.

The formal field lookup should become conceptually:

```rust
fn canonical_field_knowledge(
    &self,
    field: &FieldId,
) -> Option<TypeKnowledge> {
    self.field_signatures
        .get(field)
        .map(|signature| signature.declared_type.to_knowledge())
}
```

Dispatch may still be used to resolve a field **identity** by receiver/name; after identity is known, type knowledge comes from `FieldSignatureTable`.

## Step 4.3 — Change initializer checking

In `check_class_field_initializers`, replace type retrieval through `ctx.get_field(...)` if that method ultimately reads surface type state. Build the `FieldId` from owner/name/side and retrieve canonical declared knowledge from the field table.

Keep diagnostic behavior unchanged:

```rust
check_field_initializer_against_declared(ctx, field, &declared);
```

## Step 4.4 — Seed advisory fields from canonical declared facts

For each field:
- if canonical declared knowledge is established, project it into advisory shape as the formal baseline;
- if canonical declared knowledge is `Dynamic`, retain the dynamic boundary semantics and do not pretend the advisory shape is formal;
- if canonical declared knowledge is `Unknown`, allow runtime-shape observations to populate only advisory products.

Never write advisory joins back into `FieldSignatureTable`.

## Step 4.5 — Add field authority guard

The guard should permit `dispatch`/surface use to resolve `FieldId`, visibility, and inheritance, but forbid source/checker code from obtaining a field’s formal type by reading `member_surface.fields[...]` after canonical field publication.

## Step 4.6 — Verify

```bash
cargo +stable fmt --all -- --check
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic \
  semantic::integration::declaration_knowledge
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic \
  semantic::integration::advisory_analysis
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic \
  semantic::incremental::declaration_authority
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic
git diff --check
```

Commit:

```bash
git add phalcom-semantic/src/checker/context.rs \
        phalcom-semantic/src/checker/declaration.rs \
        phalcom-semantic/src/session.rs \
        phalcom-semantic/src/advisory \
        phalcom-semantic/tests/semantic/integration/declaration_knowledge.rs \
        phalcom-semantic/tests/semantic/integration/advisory_analysis.rs \
        phalcom-semantic/tests/semantic/incremental/declaration_authority.rs
git commit -m "refactor(semantic): route field semantics through canonical signatures"
```

---

# 6. Task 5 — Phase 7A: Consolidate Protocol-Neutral Declaration Presentation

**Deliverable:** `phalcom-semantic::presentation` provides the complete, single formatting policy for callable and field declarations. LSP adapters do not inspect canonical signature internals to decide formal-vs-advisory precedence.

**Files:**
- Modify: `phalcom-semantic/src/presentation.rs`
- Modify: `phalcom-semantic/src/editor.rs`
- Modify: `phalcom-semantic/src/snapshot.rs`
- Modify tests: `phalcom-semantic/tests/semantic/integration/presentation.rs`

## Step 5.1 — Add presentation RED tests for declaration states

Extend presentation integration tests to cover:
- annotated parameter -> formal type text;
- unannotated parameter with advisory observation -> formal `Unknown` plus separately available advisory text;
- annotated return -> formal text;
- inferred unannotated return -> published formal/inferred text;
- constructor return -> correctly specialized/presented `Self`/owner instance semantics;
- annotated field -> formal text;
- unannotated field -> formal `Unknown` with optional advisory text.

The protocol-neutral model should preserve both channels rather than pre-concatenating them.

## Step 5.2 — Add field presentation type

Add:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldPresentation {
    pub field: FieldId,
    pub owner_name: Box<str>,
    pub name: Box<str>,
    pub type_: FormalPresentation,
    pub mutable: bool,
    pub documentation: Option<Arc<str>>,
}
```

and:

```rust
impl FieldPresentation {
    pub fn from_signature(
        signature: &FieldSemanticSignature,
        presenter: &TypePresenter<'_>,
    ) -> Self { ... }
}
```

## Step 5.3 — Make callable presentation use canonical published-return API

`CallablePresentation::from_signature` must call `signature.published_return_knowledge()` rather than locally selecting `inferred_return` over `declared_return` in multiple adapters.

Preserve `Unknown`/`Dynamic` explicitly.

## Step 5.4 — Expose declaration presentation through editor query

Add read-only helpers to `EditorSemanticQuery`:

```rust
pub fn callable_presentation(
    &self,
    callable: &CallableId,
) -> Option<CallablePresentation>

pub fn field_presentation(
    &self,
    field: &FieldId,
) -> Option<FieldPresentation>
```

These may join canonical source metadata (`SourceCallableKind`, names/ranges/docs) with signature products but must not perform type inference.

## Step 5.5 — Verify semantic presentation

```bash
cargo +stable fmt --all -- --check
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic \
  semantic::integration::presentation
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic
git diff --check
```

Commit:

```bash
git add phalcom-semantic/src/presentation.rs \
        phalcom-semantic/src/editor.rs \
        phalcom-semantic/src/snapshot.rs \
        phalcom-semantic/tests/semantic/integration/presentation.rs
git commit -m "feat(semantic): expose canonical declaration presentation"
```

---

# 7. Task 6 — Phase 7B: Make Hover, Signature Help, and Inlay Hints Agree

**Deliverable:** For the same declaration and snapshot, hover, signature help, and inlay hints use identical canonical type text and formal/advisory precedence. The IDE no longer displays duplicate type information for one semantic site.

**Files:**
- Modify: `phalcom-lsp/src/inlay_hints.rs`
- Modify: `phalcom-lsp/src/signature_help.rs`
- Modify: `phalcom-lsp/src/hover.rs`
- Modify: `phalcom-lsp/src/backend.rs`
- Modify only if needed: `phalcom-lsp/src/presentation.rs`
- Add/modify tests under registered `phalcom-lsp/tests/`
- Extend: `phalcom-lsp/tests/semantic_boundary.rs`

## Step 6.1 — Add one end-to-end declaration fixture

Use a fixture that contains:

```phalcom
class Probe {
    value: String

    run(_ formal: Int, observed, label named: String) -> String {
        observed
        "done"
    }
}

let probe = Probe.new(...)
probe.run(1, 2, label: "x")
```

Adapt constructor syntax to a currently valid fixture; the important surface is mixed known/unknown parameters, field, and return.

For each semantic declaration, assert:
- hover formal type text;
- signature-help type text;
- inlay type text where an inlay is appropriate;
- no duplicate hint at explicitly annotated source sites;
- formal known text suppresses advisory fallback text;
- advisory text appears only when formal display is unavailable and policy permits it.

## Step 6.2 — Fix `inlay_hints.rs` to current canonical API

Replace stale accesses:

```text
parameter.index       -> parameter.index()
parameter.ty          -> parameter.declared_type / presentation helper
signature.return_type -> signature.published_return_knowledge() / presentation helper
```

For fields, stop passing `formal=None` unconditionally. Retrieve `snapshot.field_signatures.get(&field.id)` and project its declared type through `TypePresenter` / `FieldPresentation`.

Keep `ExplicitAnnotationIndex` only as a syntax-owned suppression index. It may answer “does source already spell a type here?” but must never decide the type itself.

## Step 6.3 — Fix `signature_help.rs`

Stop reading stale raw fields such as `parameter.ty` and `signature.return_type`. Prefer `EditorSemanticQuery::callable_presentation` or `CallablePresentation` as input.

The LSP renderer should become a pure adapter:

```rust
pub fn render_signature_help(
    presentation: &phalcom_semantic::CallablePresentation,
    advisory: Option<&phalcom_semantic::AdvisoryCallableSummary>,
    active_parameter: usize,
) -> SignatureHelp
```

If an advisory parameter is shown because `presentation.parameters[i].type_ == Unknown`, label it clearly as advisory in tooltip/secondary text rather than replacing the formal type in the canonical signature string.

## Step 6.4 — Fix hover composition

Hover method signature text must be sourced from `CallablePresentation`, not independently formatted from AST/dispatch. Preserve Phaldoc harvesting and keyword hover because those are syntax/documentation concerns, not type inference.

The typed signature should present formal unknownness explicitly. For example:

```text
run(_ formal: Int, _ observed: Unknown, label named: String) -> String
```

If an advisory observation for `observed` is available, display it as a distinct advisory line/section, not by replacing `Unknown` in the formal signature.

## Step 6.5 — Enforce single type label per inlay site

`push_canonical_hint` already prefers formal text over advisory text. Preserve that invariant and add a test that a site cannot produce both `: Formal` and a second advisory type hint.

Explicit source annotation suppresses an inlay entirely; hover/signature help still show the canonical formal type.

## Step 6.6 — Verify LSP presentation

Run focused LSP tests first, then complete suites:

```bash
cargo +stable fmt --all -- --check
RUSTFLAGS="" cargo +stable check -p phalcom-lsp
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic \
  semantic::integration::presentation
RUSTFLAGS="" cargo +stable test -p phalcom-lsp
RUSTFLAGS="" cargo +stable test -p phalcom-semantic
git diff --check
```

Commit:

```bash
git add phalcom-lsp/src/inlay_hints.rs \
        phalcom-lsp/src/signature_help.rs \
        phalcom-lsp/src/hover.rs \
        phalcom-lsp/src/backend.rs \
        phalcom-lsp/src/presentation.rs \
        phalcom-lsp/tests
git commit -m "refactor(lsp): render declaration types from canonical semantic presentation"
```

---

# 8. Task 7 — Add Permanent Mechanical Authority and Boundary Audits

**Deliverable:** Future changes cannot silently recreate a second semantic authority in dispatch, native import, fields, or LSP presentation.

**Files:**
- Modify: `phalcom-semantic/tests/semantic/incremental/declaration_authority.rs`
- Modify: `phalcom-lsp/tests/semantic_boundary.rs`
- Possibly add: `phalcom-semantic/tests/semantic/incremental/declaration_projection.rs`

## Step 7.1 — Semantic forbidden-pattern audit

The test should scan only the files/functions where the forbidden direction is meaningful. Do not forbid all `dispatch.get_surface` use; resolution legitimately needs it.

Assert these invariants:

```text
checker/declaration_signature.rs:
  semantic_signature_for_member does not read DeclarationSurface

types/native.rs:
  canonical native signature is not reconstructed from surface.get_callable

checker/declaration.rs / field_signature.rs:
  formal field declaration does not read a surface field type as source truth

session.rs:
  advisory return baseline comes from CallableSignatureTable
  constructor semantic classification comes from CallableSignatureTable
  inferred-return canonical update happens before dispatch projection update
  advisory never mutates callable/field formal signatures
```

## Step 7.2 — LSP boundary audit

Extend `phalcom-lsp/tests/semantic_boundary.rs` to forbid semantic type reconstruction patterns such as:

```text
resolve_type_annotation
TypeStore mutation
DeclarationSurface type reads used for hover/signature type text
request-time semantic inference helpers
```

Allow syntax parsing for call-site shape, explicit-annotation suppression, keyword hover, and Phaldoc harvesting.

## Step 7.3 — Verify guards against a deliberate mutation

Before committing, temporarily reintroduce one forbidden string in a scratch edit and prove the focused authority test fails; then revert the scratch edit and rerun green. This validates the guard itself rather than merely asserting it exists.

## Step 7.4 — Commit

```bash
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic \
  semantic::incremental::declaration_authority
RUSTFLAGS="" cargo +stable test -p phalcom-lsp --test semantic_boundary
git diff --check

git add phalcom-semantic/tests/semantic/incremental/declaration_authority.rs \
        phalcom-lsp/tests/semantic_boundary.rs
git commit -m "test(semantic): enforce canonical declaration authority boundaries"
```

---

# 9. Task 8 — Repair Hosted CI So Merge Status Is Meaningful

**Deliverable:** PR CI runs the toolchain it says it runs and does not inherit developer-machine `target-cpu=native` flags on GitHub-hosted runners.

**Files:**
- Modify: `.github/workflows/ci.yml`
- Keep unless local developer policy changes: `.cargo/config.toml`
- Keep: `rust-toolchain.toml` (`nightly-2026-07-10`)
- Modify/replace: `.github/workflows/canonical-declaration-verify.yml`

**Verified defect:** Current CI installs stable but executes plain `cargo`, while the repository toolchain file pins `nightly-2026-07-10`. `.cargo/config.toml` injects `-Zthreads=2 -C target-cpu=native`; hosted builds have crashed with `SIGILL` under `target-cpu=native`. This is an infrastructure failure and must be fixed before PR status is used as merge evidence.

## Step 8.1 — Make normal CI explicitly stable and neutralize developer rustflags

At workflow level add:

```yaml
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUSTFLAGS: ""
```

Then change commands:

```yaml
# test
- run: cargo +stable build --workspace --all-targets
- run: cargo +stable test --workspace --all-targets

# LSP/VS Code
- run: cargo +stable build -p phalcom-lsp

# fmt
- run: cargo +stable fmt --all -- --check

# clippy
- run: cargo +stable clippy --workspace --all-targets -- -D warnings
```

This makes the “stable” jobs actually stable and overrides `[build].rustflags` from `.cargo/config.toml` for hosted CI.

Keep the pinned nightly file for local/compiler work that needs it; do not change the project toolchain merely to make CI labels accurate.

## Step 8.2 — Isolate Miri toolchain

Miri remains a nightly-only lane. Give the Miri step an explicit neutral rustflag environment:

```yaml
- run: RUSTFLAGS="" cargo +nightly miri test -p phalcom-ast
```

If latest-nightly Miri cannot install/run reproducibly, pin the Miri action to a known date independently; do not make normal semantic merge verification depend on `target-cpu=native`.

## Step 8.3 — Make formatting/clippy status intentional

Current `continue-on-error` comments describe soft launch. Do not silently convert these to hard requirements in the same semantic PR unless the branch is already clean.

For final canonical-declaration acceptance, however, the dedicated verification workflow must run hard `fmt` and semantic/LSP checks regardless of the general CI soft-launch policy.

## Step 8.4 — Convert `canonical-declaration-verify.yml` into a read-only acceptance gate

Remove its “temporary branch verification” semantics and make it verify every push to `codex/canonical-declaration-knowledge` until merge.

Required steps:

```yaml
- uses: dtolnay/rust-toolchain@stable
  with:
    components: rustfmt
- run: cargo +stable fmt --all -- --check
- run: git diff --check
- run: RUSTFLAGS="" cargo +stable check -p phalcom-semantic -p phalcom-lsp -p phalcom-modules
- run: RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic
- run: RUSTFLAGS="" cargo +stable test -p phalcom-lsp
- run: RUSTFLAGS="" cargo +stable test -p phalcom-modules
```

Add named focused acceptance steps for:
- declaration authority;
- canonical parameter identity;
- constructor parameter transfer;
- native canonical projection;
- field canonical publication;
- semantic presentation;
- LSP semantic boundary / declaration presentation.

The workflow must not edit/commit/push source.

## Step 8.5 — Verify CI repair

Push the CI commit and require:
- general CI `Test (stable)` reaches and runs tests instead of SIGILL;
- VS Code lane builds the LSP server;
- dedicated canonical declaration workflow passes its semantic/LSP matrix.

Commit:

```bash
git add .github/workflows/ci.yml \
        .github/workflows/canonical-declaration-verify.yml
git commit -m "ci: make canonical declaration merge gates portable"
```

---

# 10. Task 9 — Cleanup and Documentation Closure Before Review

**Deliverable:** Branch contains permanent architecture/tests only; no temporary RED/GREEN transformation drivers remain; docs describe the actual implementation.

**Files:**
- Remove any remaining temporary phase scripts/workflows
- Update: `.agents/skills/semantic-analysis-development/references/current-architecture.md`
- Update: `.agents/skills/phalcom-semantic-model/references/current-implementation-map.md`
- Create: `docs/impl/semantic/semantic-correctness/part-4/canonical-declaration-knowledge-closure.md`
- Update PR #6 description

## Step 9.1 — Search for temporary machinery

```bash
git ls-files '.github/scripts/*phase*' '.github/workflows/*phase*'
git grep -n 'Phase 3 RED\|Phase 4 RED\|Phase 4 GREEN\|transformation driver'
```

- [ ] Remove branch-only self-modifying drivers.
- [ ] Keep only read-only verification workflows that remain useful through merge.

## Step 9.2 — Update architecture docs

Document these final facts:

```text
CallableSignatureTable: canonical callable declaration/type authority
FieldSignatureTable: canonical field declaration/type authority
DeclarationSurface / SurfaceDispatchResolver: lookup/dispatch projection only
CallableParameterId: sole parameter identity
AdvisoryWorkspace: non-authoritative runtime-shape/editor fallback
EditorSemanticQuery + presentation: protocol-neutral editor read layer
phalcom-lsp: no semantic reconstruction
```

Also document constructor duality explicitly:
- public declaration/signature/parameters class-side;
- body analysis may route through instance-side execution identity;
- `get_for_body()` is the only compatibility bridge and must not be generalized to arbitrary class-side fallback.

## Step 9.3 — Write closure record

`canonical-declaration-knowledge-closure.md` should contain:
- branch/PR;
- architecture before/after;
- completed phases 1–7;
- permanent regressions;
- accepted dispatch role;
- known intentionally deferred work;
- final verification workflow name;
- merge acceptance rule.

Do not call unfinished typing features “complete.” Close only the declaration-knowledge authority refactor.

## Step 9.4 — Update PR #6 and mark review scope

Replace the draft body with:
- problem statement;
- architecture invariant;
- phase summary;
- test matrix;
- CI portability repair note;
- exact closure doc link.

Keep the PR draft until Task 10 finishes.

Commit docs:

```bash
git add .agents/skills/semantic-analysis-development/references/current-architecture.md \
        .agents/skills/phalcom-semantic-model/references/current-implementation-map.md \
        docs/impl/semantic/semantic-correctness/part-4/canonical-declaration-knowledge-closure.md
git commit -m "docs(semantic): record canonical declaration authority"
```

---

# 11. Task 10 — Synchronize With `main` and Produce an Exact Verified Candidate SHA

**Deliverable:** One branch SHA contains all intended work, is not behind current `main`, and passes the complete read-only acceptance workflow.

## Step 10.1 — Freeze feature edits

- [ ] Re-fetch `main` and `codex/canonical-declaration-knowledge`.
- [ ] Record both SHAs in the PR body/closure note.
- [ ] From this point, only conflict resolution, verification fixes, review fixes, or documentation corrections are allowed.

## Step 10.2 — Synchronize current `main`

If `main` has advanced beyond the branch merge base:
- [ ] merge current `main` into `codex/canonical-declaration-knowledge` rather than silently testing against an old base;
- [ ] resolve conflicts according to current single-world architecture;
- [ ] rerun all focused semantic/LSP tests touched by conflict resolution;
- [ ] push the synchronization commit.

Do not merge PR #6 while GitHub reports the branch behind current `main` if that would cause untested merge-result behavior.

## Step 10.3 — Run the final local/branch verification matrix

Required commands:

```bash
cargo +stable fmt --all -- --check
git diff --check
RUSTFLAGS="" cargo +stable check -p phalcom-semantic -p phalcom-modules -p phalcom-lsp
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic
RUSTFLAGS="" cargo +stable test -p phalcom-semantic
RUSTFLAGS="" cargo +stable test -p phalcom-modules
RUSTFLAGS="" cargo +stable test -p phalcom-lsp
```

Then focused gates:

```bash
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic \
  semantic::incremental::declaration_authority
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic \
  canonical_parameter_advisory
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic \
  semantic::integration::declaration_knowledge
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic \
  semantic::integration::presentation
RUSTFLAGS="" cargo +stable test -p phalcom-lsp --test semantic_boundary
```

Use the concrete final test paths introduced by Tasks 2–6 as additional focused commands; do not omit them because the broad suite also ran.

## Step 10.4 — Verify repository ownership mechanically

Run searches and inspect every hit:

```bash
git grep -n 'semantic_signature_from_surface' -- phalcom-semantic
git grep -n 'CallableSemanticKind::Constructor' -- phalcom-semantic/src/session.rs phalcom-semantic/src/types/native.rs
git grep -n 'surface.get_callable' -- phalcom-semantic/src/types/native.rs
git grep -n 'AdvisoryParameterSlot' -- phalcom-semantic
git grep -n 'sort_by_key.*declaration_range' -- phalcom-semantic/src/session.rs
git grep -n 'parameter\.ty\|signature\.return_type' -- phalcom-lsp/src
```

Expected final interpretation:
- no canonical semantic reconstruction hits;
- no obsolete advisory parameter identity;
- no range-based parameter reconstruction;
- no stale LSP signature field API;
- dispatch-only lookup uses remain and are reviewed as legitimate.

## Step 10.5 — Obtain exact-SHA GitHub acceptance

- [ ] Record final branch head `CANDIDATE_SHA`.
- [ ] Wait for `Canonical Declaration Verification` run whose `head_sha == CANDIDATE_SHA`.
- [ ] Verify every required job/step is `success`.
- [ ] Record workflow run ID and candidate SHA in `canonical-declaration-knowledge-closure.md` or a final PR comment.
- [ ] If any code/doc commit occurs afterward, discard the old acceptance and rerun on the new SHA.

## Step 10.6 — Mark PR #6 ready

Only after exact-SHA acceptance:
- [ ] change PR #6 from draft to ready;
- [ ] confirm base is still `main`;
- [ ] confirm GitHub mergeability is true;
- [ ] confirm no unresolved review threads;
- [ ] confirm branch compare is not behind `main`.

---

# 12. Task 11 — Review and Merge PR #6 Using the Verified Head

**Deliverable:** PR #6 is merged into `main` with GitHub rejecting the operation if the verified head moved.

## Step 11.1 — Perform final diff review

Review changed files grouped by ownership:

```text
Canonical data model:
  declaration_type.rs
  identity.rs
  signature.rs

Source/native publishers:
  checker/declaration_signature.rs
  checker/field_signature.rs
  checker/declaration.rs
  types/native.rs

Consumers/projections:
  session.rs
  dispatch.rs
  snapshot.rs
  presentation.rs
  editor.rs

LSP adapters:
  inlay_hints.rs
  signature_help.rs
  hover.rs
  backend.rs

Permanent regressions:
  declaration_authority.rs
  parameter_identity.rs
  declaration_knowledge.rs
  presentation.rs
  semantic_boundary.rs
```

For each group, confirm data flows only left-to-right:

```text
source/native metadata
        ↓
canonical declaration signatures
        ↓
formal checker / advisory projection / dispatch lookup projection
        ↓
protocol-neutral presentation
        ↓
LSP
```

No arrow may point back upward.

## Step 11.2 — Merge with expected head SHA

Use the exact verified `CANDIDATE_SHA` in the GitHub merge call. Prefer ordinary merge commit to preserve the already-reviewed phase history unless repository policy explicitly requires squash/rebase.

Conceptually:

```text
merge PR #6
merge_method = merge
expected_head_sha = CANDIDATE_SHA
```

If GitHub rejects because the head moved, do not retry with the new SHA until Task 10 exact-SHA verification has been repeated.

## Step 11.3 — Verify merge result

Immediately re-fetch:
- PR #6: `state=closed`, `merged=true`;
- `main` branch head;
- merge commit parents.

Verify the merge commit has:
- one parent equal to pre-merge `main`;
- one parent equal to `CANDIDATE_SHA`.

Record the merge commit SHA in the closure record/PR comment.

---

# 13. Task 12 — Post-Merge Main Verification and Temporary Artifact Closure

**Deliverable:** `main` itself is proven healthy after merge; temporary RED PRs/branches do not remain as active work.

## Step 12.1 — Verify `main`

Require a post-merge CI/verification run against the merge commit. At minimum:

```bash
RUSTFLAGS="" cargo +stable check -p phalcom-semantic -p phalcom-modules -p phalcom-lsp
RUSTFLAGS="" cargo +stable test -p phalcom-semantic --test semantic
RUSTFLAGS="" cargo +stable test -p phalcom-lsp
```

If GitHub Actions runs automatically on `main`, verify the run references the merge SHA. Do not claim completion from the pre-merge branch run alone.

## Step 12.2 — Close/delete temporary Phase RED artifacts

- [ ] Ensure Phase 3 RED PRs #7/#8 remain closed unmerged.
- [ ] Close Phase 4 RED PR #9 unmerged if it is still open.
- [ ] Delete temporary RED branches if no longer useful for forensic history; PR history remains on GitHub.
- [ ] Delete `codex/canonical-declaration-knowledge` only after `main` verification succeeds and there is no follow-up review work to preserve.

## Step 12.3 — Final completion statement

The work is complete only when all of these are true:

```text
[ ] PR #6 merged
[ ] main contains exact verified candidate as merge parent
[ ] main post-merge verification green
[ ] CallableSignatureTable is callable declaration authority
[ ] FieldSignatureTable is field declaration authority
[ ] Native metadata publishes canonical signature before dispatch
[ ] Dispatch contains no reverse semantic authority path
[ ] CallableParameterId is sole parameter identity
[ ] Hover/signature help/inlays agree on formal declaration text
[ ] Formal known facts suppress advisory duplicates
[ ] Advisory observations never mutate formal signatures
[ ] Permanent authority/boundary tests are green
[ ] Temporary self-modifying phase drivers are gone
[ ] Architecture/closure docs match implementation
```

---

# 14. Commit / Review Boundaries

Use these as review-sized commits; do not collapse unrelated semantic and CI changes into one commit while developing:

```text
1. refactor(semantic): make callable signatures authoritative
2. ci: remove completed phase 4 transformation driver
3. refactor(semantic): publish native callable signatures canonically
4. feat(semantic): publish canonical field signatures
5. refactor(semantic): route field semantics through canonical signatures
6. feat(semantic): expose canonical declaration presentation
7. refactor(lsp): render declaration types from canonical semantic presentation
8. test(semantic): enforce canonical declaration authority boundaries
9. ci: make canonical declaration merge gates portable
10. docs(semantic): record canonical declaration authority
11. merge(main): synchronize current main if required
```

Review each commit against the invariant it claims. If a task exposes an unrelated correctness bug, add a focused regression and either fix it in a separate commit or record it as deferred; do not hide it inside a broad refactor.

# 15. Execution Guidance

Recommended execution mode is `superpowers:subagent-driven-development` with one worker/task at a time and review after each task. Tasks 1–4 are sequential because they change authority/data ownership. Tasks 5–7 depend on those canonical products. Task 8 can be implemented independently after Task 1, but its final verification must run against the completed branch. Tasks 9–12 are strictly sequential merge closure.

When executing this plan, start by re-fetching `main`, the feature branch, PR #6, and the latest Phase 4 workflow result; the SHA values recorded above are a baseline, not an assumption that the repository has stopped moving.
