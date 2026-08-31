# Phalcom ADT/GADT + Associated Lookup
## Part 06 — Core Integration, Migration, Reflection, Tooling Completion, and Final Verification
### Repository-Grounded Implementation Plan

> **For agentic workers:** use test-driven development for behavioral changes, keep `phalcom-semantic` as the only static semantic authority, and run the final verification matrix before claiming completion. Tasks use checkbox (`- [ ]`) syntax for execution tracking.

**Goal:** Converge the completed ADT/GADT/associated-family/match architecture into one whole-language implementation: migrate `Option`, `Result`, and `Ordering` onto canonical `@native enum` semantics without losing optimized/native representation; finish canonical exact-case reflection; project runtime reflection metadata; complete source identity, hover, definition, completion, rename, semantic highlighting and match code actions; verify cross-module/incremental/persistent behavior; synchronize authoritative docs; and delete all remaining transitional semantic paths.

**Architecture:** `phalcom-semantic` owns declaration/type/proof/source identity and protocol-neutral tooling products. Core native enum declarations produce ordinary `EnumInfo`/`VariantInfo`. `phalcom-core` binds stable semantic identities to representation strategies, lowering facts and projected reflection metadata. `phalcom-lsp` converts compiler-owned source/presentation/tooling products into protocol messages and edits; it never resolves variants, GADTs, families or exhaustiveness independently.

**Planning repository:** `aureat/phalcom-lang`  
**Planning branch inspected:** `feat/adts`  
**Planning HEAD inspected:** `26166385f9c1bf35f6e9eb969385fc8a162f2f56`  
**Commit subject:** `ci: apply and verify ordered match orchestration`  
**Spec:** `docs/impl/adt-gadt-associated-lookup/part-6/06-core-integration-migration-reflection-tooling-final-verification-technical-spec.md`

---

# 0. Hard Invariants

- [ ] `phalcom-semantic` remains the sole source-level authority for enum/variant/family/generic/GADT/match meaning.
- [ ] `@native` affects implementation binding/representation, not enum semantics.
- [ ] `Option`, `Result`, and `Ordering` publish ordinary canonical enum semantic products.
- [ ] `Bool` remains primitive; do not create artificial `VariantId`s for booleans.
- [ ] Part 06 does not ratify exact-case source annotation syntax.
- [ ] Formal declared type and observed exact-case evidence remain distinct.
- [ ] Specialized exact cases are canonical `TypeId`s but do not create specialized declarations/runtime classes/runtime IDs.
- [ ] Generic ADT runtime representation remains erased.
- [ ] `.class` returns the actual runtime case behavior class under the current ratified policy.
- [ ] Runtime case behavior class is not a semantic `ExactCase` or `VariantId`.
- [ ] `::` remains associated lookup only; reflection has a separate API.
- [ ] Runtime reflection consumes a compact projected metadata product, not the full `SemanticSnapshot`.
- [ ] Family-pattern source identity is `VariantFamilyId` plus exact semantic candidates.
- [ ] Ordinary Rename on a variant base renames the entire variant family.
- [ ] Completion/code actions consume `PatternSpace`/`MatchResolution`/coverage products, not LSP-side exhaustiveness logic.
- [ ] Persistent artifacts never use raw `TypeId`, `RuntimeVariantId`, `CaseDiscriminant`, pointer identity, or display strings as durable semantic identity.
- [ ] Match execution semantics from Part 05.2 are not redesigned in Part 06.
- [ ] Performance measurement and backend optimization are deferred to Part 07.

---

# 1. File Responsibility Map

The exact landed Part-05.2 tree must be reconciled in Task 0. The currently verified repository seams are:

## Front end / core source

```text
phalcom-ast/src/ast.rs
    EnumDef.attributes already exists
    BuiltinAttr::Native already exists

phalcom-ast/src/parser.rs
    enum/class attribute parsing

core source modules discovered during preflight
    canonical @native Option / Result / Ordering declarations
```

## Semantic declaration/type world

```text
phalcom-semantic/src/enum_semantics.rs
    EnumInfo / VariantInfo / visibility / exact-case templates

phalcom-semantic/src/identity.rs
    VariantId / VariantFamilyId / VariantFieldId / VariantConstructorId
    SemanticTargetId

phalcom-semantic/src/types/store.rs
    canonical Applied / ExactCase / Family / union/type interning

phalcom-semantic/src/core_surface/source.rs
    source/native binding records; currently class-oriented

phalcom-semantic/src/core_surface/identity.rs
    canonical core declaration identity seam

phalcom-semantic/src/presentation.rs
    TypePresenter and protocol-neutral presentation

phalcom-semantic/src/source_index/*
    source semantic identities and occurrences

phalcom-semantic/src/match_semantics.rs
phalcom-semantic/src/checker/pattern_space.rs
    Part-05 proof/residual products
```

## Runtime/compiler

```text
phalcom-core/src/value/option.rs
    current immediate Option representation

phalcom-core/src/adt.rs
phalcom-core/src/vm/adt.rs
    runtime enum/variant registry and case primitives

phalcom-core/src/modules/semantic_lowering.rs
    formal semantic -> executable projection

phalcom-core/src/modules/artifact.rs
phalcom-core/src/modules/materialize.rs
    compiled/runtime metadata materialization

phalcom-core/src/value/mod.rs
phalcom-core/src/primitive/class.rs
    `.class` / class reflection surfaces
```

## LSP

```text
phalcom-lsp/src/hover.rs
phalcom-lsp/src/completion.rs
phalcom-lsp/src/diagnostics.rs
phalcom-lsp/src/backend.rs
```

During preflight locate exact definition/rename/semantic-token/code-action handlers with `rg` and map planned responsibilities to their real files.

## Recommended new focused modules

Create only if no equivalent landed module exists:

```text
phalcom-semantic/src/reflection.rs
    protocol-neutral semantic enum/variant/family/exact-case reflection products

phalcom-semantic/src/tooling/patterns.rs
    protocol-neutral pattern completion / missing-case / generate-match plans

phalcom-core/src/modules/reflection_metadata.rs
    SemanticSnapshot -> compact runtime reflection projection

phalcom-core/src/reflection/adt.rs
    runtime reflection materialization/access for enum/variant/exact-case metadata
```

Do not create duplicate reflection/source/tooling engines if existing modules already own these responsibilities.

---

# Task 0 — Reconcile the Final Part-05.2 Tree

**Purpose:** Part 06 is a convergence phase and must begin from the actual final 05.2 implementation, not planning names.

**Files:** inspection only initially.

- [ ] **Step 0.1: Record repository state.**

Run:

```bash
git status --short
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git log -1 --oneline
```

Record unrelated working-tree changes. Do not reset/stash/discard developer work.

- [ ] **Step 0.2: Verify Part-05.2 completion products.**

Run:

```bash
rg -n 'MatchLoweringSpec|ExecutablePattern|MatchInvariantFailure|MissingMatchLoweringSemantics' phalcom-core phalcom-semantic
rg -n 'runtime_variant_of|case_payload_at|option_case|core.*Option|Option.*VariantId' phalcom-core phalcom-semantic
rg -n 'constructor == "Some"|constructor == "None"|emit_class_test\(value_slot, constructor' phalcom-core/src/compiler
```

Record the final core Option compatibility architecture and any remaining transitional path.

- [ ] **Step 0.3: Run the Part-05.2 verification suite.**

Use the exact landed plan. At minimum:

```bash
cargo check -p phalcom-ast -p phalcom-semantic -p phalcom-core
cargo test -p phalcom-semantic --test semantic -- --nocapture
cargo test -p phalcom-core --tests -- --nocapture
```

Part 06 must not classify a pre-existing Part-05.2 failure as its own regression.

- [ ] **Step 0.4: Inventory core Option/Result/Ordering declarations and native implementations.**

Run:

```bash
rg -n 'class Option|enum Option|@native.*Option|\bOption\b' core phalcom-core phalcom-semantic 2>/dev/null
rg -n 'class Result|enum Result|@native.*Result|\bResult\b' core phalcom-core phalcom-semantic 2>/dev/null
rg -n 'class Ordering|enum Ordering|\bOrdering\b' core phalcom-core phalcom-semantic 2>/dev/null
```

Adapt path root if core sources use another directory.

- [ ] **Step 0.5: Inventory tooling handlers.**

```bash
rg -n 'goto.*definition|definition\(' phalcom-lsp/src
rg -n 'rename|prepareRename' phalcom-lsp/src
rg -n 'semantic.*token|semanticTokens' phalcom-lsp/src
rg -n 'code.?action|CodeAction' phalcom-lsp/src
```

Write a mechanical name map into the implementation report.

---

# Phase 06.A — Core Migration and Legacy Removal

# Task 1 — Add Native Enum Core-Surface Extraction and Binding

**Files:**
- Modify: `phalcom-semantic/src/core_surface/source.rs`
- Modify: `phalcom-semantic/src/core_surface/identity.rs`
- Modify: the core-surface aggregate/module that currently stores `SourceClassRecord`
- Test: existing core-surface semantic test module; create focused enum test if absent

**Consumes:** existing `EnumDef.attributes`, `BuiltinAttr::Native`, ordinary enum AST.

**Produces:** source/native declaration records for enums without separate enum semantics.

- [ ] **Step 1.1: Write failing extraction test.**

Parse:

```phalcom
@native
enum Maybe<T> {
    @variant None
    @variant Some(_ value: T)
}
```

Assert core-surface extraction records:

```text
DeclarationId(Maybe)
native declaration implementation role
both variant syntax nodes remain ordinary enum members
```

- [ ] **Step 1.2: Generalize class-only core source record ownership.**

Do not shoehorn enums into `SourceClassRecord` if that obscures kind. Prefer a declaration record enum conceptually:

```rust
pub enum SourceDeclarationRecord {
    Class(SourceClassRecord),
    Enum(SourceEnumRecord),
}
```

or the repository's equivalent.

- [ ] **Step 1.3: Reuse existing `SourceNativeBindingRole`.**

`@native` on an enum should mean `DeclarationImplementation` at the declaration boundary.

- [ ] **Step 1.4: Reject unauthorized native enum implementation binding through the same policy used for other native declarations.**

Do not make arbitrary user `@native enum` privileged solely because the parser recognizes the attribute.

- [ ] **Step 1.5: Run tests.**

```bash
cargo test -p phalcom-semantic core_surface -- --nocapture
cargo check -p phalcom-semantic
```

- [ ] **Step 1.6: Commit.**

```text
feat(core): recognize native enum declarations
```

---

# Task 2 — Publish Canonical `@native Option` Through Ordinary Enum Semantics

**Files:**
- Modify: actual core declaration source containing `Option`
- Modify only as required: core source/bootstrap registration
- Modify: `phalcom-semantic/src/core_surface/identity.rs`
- Test: create/extend `phalcom-semantic/tests/semantic/adts/native_core.rs`

- [ ] **Step 2.1: Add semantic red tests before migration.**

Assert core Option exposes ordinary:

```text
EnumInfo
VariantId(Some(_))
VariantId(None)
VariantFamilyId(Some)
VariantFamilyId(None)
VariantFieldId(Some.value)
VariantConstructorId(Some(_))
```

- [ ] **Step 2.2: Express the authoritative core declaration as `@native enum Option<T>`.**

Canonical semantic shape:

```phalcom
@native
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}
```

Preserve existing core methods/behavior by moving them onto enum/root/case surfaces according to their intended semantics rather than leaving a parallel class declaration.

- [ ] **Step 2.3: Route the declaration through ordinary enum analysis.**

No core-specific `EnumInfo` synthesis is allowed after this step.

- [ ] **Step 2.4: Verify exact constructor/singleton typing.**

Tests must prove:

```text
Option::Some(1) -> ExactCase(Some(_), Option<Int>)
expected Option<Int> + Option::None -> ExactCase(None, Option<Int>) observed knowledge
```

while declaration contracts remain root `Option<Int>` where annotated.

- [ ] **Step 2.5: Verify associated family resolution.**

Core `Option::Some::*` / exact constructor lookup must use the same associated family resolver as a source enum with identical shape.

- [ ] **Step 2.6: Run focused tests.**

```bash
cargo test -p phalcom-semantic --test semantic native_core -- --nocapture
cargo test -p phalcom-semantic --test semantic associated -- --nocapture
```

- [ ] **Step 2.7: Commit.**

```text
feat(core): model Option as native enum
```

---

# Task 3 — Publish Canonical `@native Result` Through Ordinary Enum Semantics

**Files:**
- Modify: actual core declaration source containing `Result`
- Modify native bootstrap/binding only as required
- Test: `phalcom-semantic/tests/semantic/adts/native_core.rs`

- [ ] **Step 3.1: Add semantic red tests.**

Require:

```text
Result<T,E>
Result::Ok(_ value: T)
Result::Error(_ error: E)
```

with ordinary `VariantId`, family, field and constructor identities.

- [ ] **Step 3.2: Convert the authoritative declaration to canonical `@native enum Result<T,E>`.**

Do not encode a special physical representation in `phalcom-semantic`.

- [ ] **Step 3.3: Add specialization tests.**

```phalcom
const x: Result<Int, Error> = Result::Ok(1)
```

must create the canonical exact case for `Ok` specialized to `Result<Int, Error>`.

- [ ] **Step 3.4: Add matching/family parity test against an equivalent user enum.**

The semantic products should differ only in stable declaration identity/native metadata, not shape semantics.

- [ ] **Step 3.5: Run focused tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic native_core -- --nocapture
```

```text
feat(core): model Result as native enum
```

---

# Task 4 — Migrate `Ordering` and Explicitly Keep `Bool` Primitive

**Files:**
- Modify: current core `Ordering` declaration/registration
- Modify only if needed: enum/core identity table
- Test: `phalcom-semantic/tests/semantic/adts/native_core.rs`
- Test: boolean semantic regression tests

- [ ] **Step 4.1: Reconcile current Ordering surface.**

Record exact existing case names/behavior; do not invent replacements if the language already has ratified names.

- [ ] **Step 4.2: Convert Ordering to ordinary `@native enum` semantics.**

Its named cases must receive ordinary singleton `VariantId`s.

- [ ] **Step 4.3: Add negative Bool architecture test.**

Assert boolean literals/types continue to use primitive Bool semantics and no `Bool::True/False` enum declaration is introduced.

- [ ] **Step 4.4: Run tests and commit.**

```text
feat(core): unify Ordering with native ADT semantics
```

---

# Task 5 — Generalize Native ADT Runtime Representation Registration

**Files:**
- Modify: `phalcom-core/src/adt.rs`
- Modify: `phalcom-core/src/vm/adt.rs`
- Modify: bootstrap/materialization seam for core declarations
- Modify only if needed: `phalcom-core/src/value/option.rs`
- Test: create `phalcom-core/tests/native_adt_runtime.rs`

**Goal:** Native representation is registered by canonical semantic enum/variant identity.

- [ ] **Step 5.1: Write red runtime tests from canonical Option VariantIds.**

Given semantic IDs from core enum products, verify runtime registration maps immediate `None`/`Some` values to those exact cases.

- [ ] **Step 5.2: Introduce representation strategy metadata/hook.**

Use the narrowest repository-compatible form equivalent to:

```text
General
NativeOption
NativeResult (only if current runtime needs it)
```

Do not switch on selector/source name in generic VM operations.

- [ ] **Step 5.3: Route `runtime_variant_of` through registered native representation.**

Immediate Option checks are allowed inside the Option representation adapter, but the returned ID must be the runtime ID linked to canonical semantic `VariantId`.

- [ ] **Step 5.4: Route payload operations through the representation seam.**

Preserve exact one-layer `Some` peeling.

- [ ] **Step 5.5: Preserve `.class`.**

Immediate core Option cases must resolve to their registered case behavior classes.

- [ ] **Step 5.6: Add Result runtime parity.**

If Result uses general `AdtCaseObject`, prove the native semantic declaration works with general representation. If a native representation already exists, adapt it through the same interface.

- [ ] **Step 5.7: Run tests.**

```bash
cargo test -p phalcom-core --test native_adt_runtime -- --nocapture
cargo test -p phalcom-core option -- --nocapture
cargo test -p phalcom-core adt -- --nocapture
```

- [ ] **Step 5.8: Commit.**

```text
refactor(runtime): bind native ADT representations by semantic identity
```

---

# Task 6 — Remove the Part-05.2 Core Option Semantic Compatibility Layer

**Files:**
- Modify/delete: actual temporary core Option semantic identity bridge discovered in Task 0
- Modify: runtime/bootstrap registration as required
- Test: `phalcom-core/tests/option_match_compat.rs` or renamed capability test
- Test: `phalcom-semantic/tests/semantic/adts/native_core.rs`

- [ ] **Step 6.1: Write an architecture test proving Option pattern candidates originate from normal `EnumSemanticTable`.**

- [ ] **Step 6.2: Remove temporary manually synthesized Option `VariantId`s/tables if present.**

- [ ] **Step 6.3: Preserve the physical Option adapter.**

Do not delete `Value`'s immediate representation merely because the semantic adapter is removed.

- [ ] **Step 6.4: Run collision tests.**

A user enum named `Some`/`None` must remain independent.

- [ ] **Step 6.5: Search.**

```bash
rg -n 'core.*Option.*VariantId|Option.*compat|isSome|isNone|constructor == "Some"|constructor == "None"' phalcom-semantic phalcom-core/src/compiler
```

Classify every remaining hit.

- [ ] **Step 6.6: Commit.**

```text
refactor(core): retire Option semantic compatibility bridge
```

---

# Task 7 — Audit Canonical Exact-Case Specialization

**Files:**
- Modify only if defects found: `phalcom-semantic/src/types/store.rs`
- Modify: substitution/relation/inference files only if inconsistent
- Test: `phalcom-semantic/tests/semantic/adts/exact_cases.rs`
- Test: foundations type-store tests

- [ ] **Step 7.1: Add canonicalization tests.**

Prove repeated construction of:

```text
ExactCase(Some(_), Option<Int>)
```

returns the same `TypeId` within one store.

- [ ] **Step 7.2: Add specialization distinction test.**

`Some<Int>` exact case and `Some<String>` exact case are different canonical types but share one `VariantId`.

- [ ] **Step 7.3: Add partial/application normalization test.**

Equivalent canonical enum applications must produce equal exact cases regardless of application spine construction.

- [ ] **Step 7.4: Add GADT exact-case test.**

`Expr::Int` yields exact enum result `Expr<Int>` through case environment/result specialization.

- [ ] **Step 7.5: Do not add source syntax.**

No parser/type-annotation work belongs to this task.

- [ ] **Step 7.6: Run tests and commit only if code changes are required.**

```bash
cargo test -p phalcom-semantic exact_case -- --nocapture
cargo test -p phalcom-semantic --test semantic adts::exact_cases -- --nocapture
```

```text
test(types): lock canonical exact-case specialization
```

---

# Task 8 — Verify Formal Contract vs Observed Exact-Case Evidence

**Files:**
- Modify if needed: binding/flow/type-evidence checker files
- Test: `phalcom-semantic/tests/semantic/adts/exact_cases.rs`
- Test: flow/refinement suites

- [ ] **Step 8.1: Add immutable binding test.**

```phalcom
const x: Option<Int> = Option::Some(1)
```

Assert declaration/formal contract remains `Option<Int>` while observed knowledge may be exact Some.

- [ ] **Step 8.2: Add mutable assignment test.**

Observed exact case changes after reassignment without changing formal declaration type.

- [ ] **Step 8.3: Add match branch refinement test.**

Branch exact-case proof must not mutate outer declaration type.

- [ ] **Step 8.4: Add API-return widening test.**

Declared return type remains caller contract.

- [ ] **Step 8.5: Run tests and commit.**

```text
test(types): preserve contract and exact-case evidence separation
```

---

# Phase 06.B — Reflection and Runtime Metadata Completion

# Task 9 — Define Protocol-Neutral Semantic Reflection Products

**Files:**
- Create recommended: `phalcom-semantic/src/reflection.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Test: create `phalcom-semantic/tests/semantic/adts/reflection.rs`

- [ ] **Step 9.1: Write product-shape tests first.**

Require reflection lookup for one enum to expose enum/variant/family/field identities without runtime IDs.

- [ ] **Step 9.2: Add semantic reflection structs.**

Implement repository-style equivalents of:

```text
EnumReflection
VariantReflection
VariantFamilyReflection
VariantFieldReflection
ExactCaseTypeReflection
```

- [ ] **Step 9.3: Build from existing canonical tables.**

Do not duplicate payload/generic/GADT semantics. Reflection methods project `EnumSemanticTable`, `TypeStore`, associated products, and source metadata.

- [ ] **Step 9.4: Keep visibility context explicit.**

Provide internal complete reflection for compiler tooling and an access-filtered acquisition API for language/runtime reflection.

- [ ] **Step 9.5: Run tests and commit.**

```bash
cargo test -p phalcom-semantic --test semantic adts::reflection -- --nocapture
```

```text
feat(semantic): add ADT reflection projections
```

---

# Task 10 — Add Canonical Specialized Exact-Case Type Reflection

**Files:**
- Modify: semantic reflection module from Task 9
- Modify: `phalcom-semantic/src/presentation.rs`
- Test: `phalcom-semantic/tests/semantic/adts/reflection.rs`
- Test: presentation/type-model tests

- [ ] **Step 10.1: Add same-specialization canonical reflection test.**

Repeated reflection of the same canonical exact-case `TypeId` must yield equivalent/canonical descriptor identity under the chosen reflection cache model.

- [ ] **Step 10.2: Add distinct-specialization test.**

Some/Option Int vs String have distinct exact-case type descriptors but same variant declaration descriptor.

- [ ] **Step 10.3: Project specialized field/result types.**

Use canonical substitution/case environment; do not reparse declarations.

- [ ] **Step 10.4: Extend TypePresenter.**

Choose one deterministic exact-case rendering. Clearly document it as presentation unless/until source annotation grammar is separately ratified.

- [ ] **Step 10.5: Add GADT reflection test.**

Exact case for `Expr::Int` reflects specialized result `Expr<Int>` and declaration case constraints without runtime proof objects.

- [ ] **Step 10.6: Run and commit.**

```text
feat(types): reflect canonical exact-case specializations
```

---

# Task 11 — Audit ExactCase in Typed Dispatch/Relation

**Files:**
- Inspect/modify: `phalcom-semantic/src/types/relation.rs`
- Inspect/modify: dispatch candidate/selection modules
- Test: dispatch/type relation suites
- Test: `phalcom-semantic/tests/semantic/adts/exact_cases.rs`

- [ ] **Step 11.1: Add relation tests.**

Require:

```text
ExactCase(Some, Option<Int>) <: Option<Int>
ExactCase(Some, Option<String>) !<: Option<Int>
```

according to existing nominal generic relation rules.

- [ ] **Step 11.2: Add typed candidate distinction test.**

Where current typed-dispatch infrastructure permits exact type targets, prove distinct exact specializations are not collapsed before dispatch selection.

- [ ] **Step 11.3: Add erased dynamic-boundary negative test.**

Do not infer `Int` vs `String` from `RuntimeVariantId(Some)` alone.

- [ ] **Step 11.4: Reuse canonical relation/inference.**

No special “variant overload resolver” is allowed.

- [ ] **Step 11.5: Commit if changes required.**

```text
test(dispatch): preserve exact-case specialization identity
```

---

# Task 12 — Project Runtime Reflection Metadata

**Files:**
- Create recommended: `phalcom-core/src/modules/reflection_metadata.rs`
- Modify: `phalcom-core/src/modules/mod.rs`
- Modify: `phalcom-core/src/modules/compile.rs`
- Modify: `phalcom-core/src/modules/artifact.rs`
- Modify: `phalcom-core/src/modules/materialize.rs`
- Test: create `phalcom-core/tests/reflection_metadata.rs`

- [ ] **Step 12.1: Write failing projection tests.**

Assert metadata contains stable enum/variant/family/field structure but no raw `RuntimeVariantId`/`CaseDiscriminant` as semantic identity.

- [ ] **Step 12.2: Add compact immutable module reflection metadata.**

Project only runtime-reflection-required information.

- [ ] **Step 12.3: Define stable keys or reuse existing stable declaration/module identities.**

Stable variant keys include exact selector; family keys include base; field keys include variant + declaration index.

- [ ] **Step 12.4: Add deterministic fingerprint.**

Exclude source-range-only movement and rendered prose from semantic metadata identity.

- [ ] **Step 12.5: Attach to compiled module/artifact.**

Do not retain `Arc<SemanticSnapshot>` solely for runtime reflection.

- [ ] **Step 12.6: Run tests and commit.**

```bash
cargo test -p phalcom-core --test reflection_metadata -- --nocapture
```

```text
feat(runtime): project ADT reflection metadata
```

---

# Task 13 — Materialize Runtime Enum/Variant Reflection Objects

**Files:**
- Create/modify reflection runtime module discovered during preflight
- Likely create: `phalcom-core/src/reflection/adt.rs`
- Modify: heap/object/tracing only if new heap objects are required
- Modify: primitive/API registration for reflection methods
- Test: create `phalcom-core/tests/reflection_runtime.rs`

- [ ] **Step 13.1: Choose representation consistent with existing reflection objects.**

Reuse existing Type/Class/metaobject machinery where appropriate, but do not reduce semantic variant metadata to a generic dictionary.

- [ ] **Step 13.2: Implement Enum/Variant/Family/Field metaobjects.**

Public fields/methods expose language values, never compiler IDs.

- [ ] **Step 13.3: Implement visibility-aware acquisition.**

Acquisition context is checked once; stored descriptor/capability behaves stably afterwards.

- [ ] **Step 13.4: Implement enum APIs.**

Structural equivalents of:

```text
variants
variantCount
variant(selector:)
variantFamily(named:)
```

- [ ] **Step 13.5: Implement variant APIs.**

At minimum owner, selector, family, fields, shape, result type, case class.

- [ ] **Step 13.6: Add GC/root tests if heap-backed.**

- [ ] **Step 13.7: Run and commit.**

```bash
cargo test -p phalcom-core --test reflection_runtime -- --nocapture
```

```text
feat(runtime): expose ADT semantic reflection
```

---

# Task 14 — Make Runtime Case Behavior Classes Reflectable and Link Them to Variants

**Files:**
- Modify: `phalcom-core/src/adt.rs`
- Modify: `phalcom-core/src/vm/adt.rs`
- Modify: `phalcom-core/src/value/mod.rs`
- Modify: `phalcom-core/src/primitive/class.rs`
- Test: `phalcom-core/tests/reflection_runtime.rs`

- [ ] **Step 14.1: Rename misleading “hidden” documentation where it implies non-reflectability.**

Use “runtime case behavior class.” Preserve the rule that it has no source declaration ID.

- [ ] **Step 14.2: Add reverse reflection bridge.**

`ClassId -> RuntimeVariantId -> semantic runtime reflection metadata` for case classes.

- [ ] **Step 14.3: Verify `.class`.**

Source-defined and native Option/Result/Ordering cases return the expected case behavior class.

- [ ] **Step 14.4: Add negative semantic tests.**

Matching/source navigation/type reflection must not use the case class as their identity.

- [ ] **Step 14.5: Commit.**

```text
feat(reflection): expose runtime ADT case classes
```

---

# Task 15 — Persistence and Stable Semantic Reflection Keys

**Files:**
- Modify: semantic snapshot/metadata serialization modules discovered in preflight
- Modify: reflection metadata projection
- Test: serialization/fingerprint/incremental suites

- [ ] **Step 15.1: Inventory persisted semantic IDs.**

```bash
rg -n 'serialize|serde|snapshot|metadata|fingerprint|RuntimeVariantId|CaseDiscriminant|VariantTypeId|TypeId' phalcom-semantic phalcom-core
```

- [ ] **Step 15.2: Define stable variant/family/field/exact-case keys.**

Reuse `DeclarationId`/stable project/module keys where sufficient.

- [ ] **Step 15.3: Add round-trip/remapping tests.**

A metadata artifact reconstructed into a different runtime/type-store world maps stable semantic keys to new compact IDs without identity drift.

- [ ] **Step 15.4: Add source-range-only stability test.**

Changing ranges/doc whitespace must not change range-free reflection identity.

- [ ] **Step 15.5: Commit.**

```text
feat(metadata): stabilize ADT reflection identities
```

---

# Phase 06.C — Source Index / LSP / Developer Tooling Completion

# Task 16 — Extend `SemanticTargetId` with Variant Family and Field Identities

**Files:**
- Modify: `phalcom-semantic/src/identity.rs`
- Modify all exhaustive `SemanticTargetId` matches
- Test: source-index identity tests

- [ ] **Step 16.1: Add failing identity tests.**

- [ ] **Step 16.2: Add:**

```rust
SemanticTargetId::VariantFamily(VariantFamilyId)
SemanticTargetId::VariantField(VariantFieldId)
```

- [ ] **Step 16.3: Update target hashing/order/fingerprint consumers.**

- [ ] **Step 16.4: Do not expose runtime IDs.**

- [ ] **Step 16.5: Run and commit.**

```text
feat(source-index): add variant family and field targets
```

---

# Task 17 — Attach Rich Pattern Semantic Targets to Source Occurrences

**Files:**
- Modify: `phalcom-semantic/src/source_index/builder.rs`
- Modify: `phalcom-semantic/src/source_index/mod.rs`
- Modify: `phalcom-semantic/src/source_index/occurrence.rs`
- Modify semantic match product only if a required source range/identity was not retained
- Test: source-index tests; create `phalcom-semantic/tests/semantic/adts/tooling.rs`

- [ ] **Step 17.1: Add exact qualified/contextual tests.**

Both `Option::Some(x)` and contextual `Some(x)` target the same exact `VariantId`.

- [ ] **Step 17.2: Attach whole-family/selector-family base token.**

Target `VariantFamilyId` and retain exact candidate IDs in a queryable formal attachment.

- [ ] **Step 17.3: Attach payload labels.**

Exact pattern label -> `VariantFieldId`.

- [ ] **Step 17.4: Model multi-candidate label targets explicitly.**

For family patterns where one label maps to candidate-specific field IDs, retain all deterministic targets rather than arbitrarily selecting one.

- [ ] **Step 17.5: Attach owner token separately.**

Owner qualification targets the enum declaration.

- [ ] **Step 17.6: Run and commit.**

```bash
cargo test -p phalcom-semantic --test semantic adts::tooling -- --nocapture
```

```text
feat(source-index): attach ADT pattern identities
```

---

# Task 18 — Complete Go-to-Definition for Variants, Families, and Payload Labels

**Files:**
- Modify exact LSP definition handler located in Task 0 (often `phalcom-lsp/src/backend.rs` plus source-index query helpers)
- Modify: `phalcom-semantic/src/source_index/*` query API if needed
- Test: LSP definition integration tests

- [ ] **Step 18.1: Add red exact variant navigation tests.**

Qualified and contextual pattern occurrence -> exact `@variant` source declaration.

- [ ] **Step 18.2: Add family definition test.**

`Dog*` -> all declaration sites in the `Dog` family.

- [ ] **Step 18.3: Add payload-label definition test.**

External label -> payload parameter declaration.

- [ ] **Step 18.4: Add cross-module/re-export test.**

Navigation crosses to defining module.

- [ ] **Step 18.5: Add negative case-class assertion.**

Variant syntax never navigates to runtime case behavior class.

- [ ] **Step 18.6: Run and commit.**

```text
feat(lsp): navigate ADT patterns by semantic identity
```

---

# Task 19 — Add Protocol-Neutral Variant/Pattern Hover Presentations

**Files:**
- Modify: `phalcom-semantic/src/presentation.rs`
- Modify semantic reflection/pattern presentation module
- Modify: `phalcom-lsp/src/hover.rs`
- Test: semantic presentation tests
- Test: LSP hover tests

- [ ] **Step 19.1: Add semantic presentation products.**

Cover exact variants, singleton/constructor distinction, family patterns, payload fields, and branch GADT equalities.

- [ ] **Step 19.2: Reuse `TypePresenter`.**

No exact-case formatter in LSP.

- [ ] **Step 19.3: Add exact specialized hover test.**

Known `Some<Int>` exact case is presented distinctly from root `Option<Int>` while formal contract can also be shown where relevant.

- [ ] **Step 19.4: Add family-pattern hover test.**

Show selector constraint + candidate count/list under a presentation budget.

- [ ] **Step 19.5: Add GADT proof hover test.**

Use Part-05 explanation/proof products.

- [ ] **Step 19.6: Run and commit.**

```text
feat(lsp): present ADT pattern and exact-case hover
```

---

# Task 20 — Add Semantic Highlighting for ADT/Pattern Roles

**Files:**
- Modify semantic token handler located in Task 0
- Modify occurrence kinds/modifiers if necessary
- Test: semantic token integration tests

- [ ] **Step 20.1: Add source fixture:**

```phalcom
Result::Error(_, reason: message)
```

Assert distinct classifications for `Result`, `Error`, `_`, `reason`, `message`.

- [ ] **Step 20.2: Add family-pattern classification.**

- [ ] **Step 20.3: Use semantic/AST occurrence roles, not textual regex.**

- [ ] **Step 20.4: Run and commit.**

```text
feat(lsp): classify ADT pattern semantic tokens
```

---

# Task 21 — Implement Semantic Variant-Family Rename

**Files:**
- Modify LSP rename handler found in Task 0
- Modify: `phalcom-semantic/src/source_index/*` reverse-target queries
- Add semantic rename validation helper if current architecture has one
- Test: rename integration tests

- [ ] **Step 21.1: Add family-base rename fixture with overloads.**

Rename `Dog` and assert all exact variant declarations/references/family patterns change.

- [ ] **Step 21.2: Add unrelated-family negative fixture.**

Another enum's `Dog` is untouched.

- [ ] **Step 21.3: Add contextual shorthand fixture.**

`Dog(x)` updates because of target identity.

- [ ] **Step 21.4: Add external payload label rename.**

Validate selector collisions before producing edits.

- [ ] **Step 21.5: Add local payload name independent rename.**

Do not automatically change external label when they differ.

- [ ] **Step 21.6: Run and commit.**

```text
feat(lsp): rename variant families semantically
```

---

# Task 22 — Publish Pattern Completion Context from `phalcom-semantic`

**Files:**
- Create recommended: `phalcom-semantic/src/tooling/mod.rs`
- Create recommended: `phalcom-semantic/src/tooling/patterns.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Reuse: match/pattern-space/exhaustiveness modules
- Test: `phalcom-semantic/tests/semantic/adts/tooling.rs`

- [ ] **Step 22.1: Add residual completion tests first.**

For:

```phalcom
match option {
    Some(x) => ...
    /* cursor */
}
```

semantic context ranks `None` as remaining exact case.

- [ ] **Step 22.2: Define a protocol-neutral `PatternCompletionContext`.**

Contain expected type, residual summary, legal/spellable candidates, and wildcard recommendation.

- [ ] **Step 22.3: Reuse Part-05 residual/witness products.**

If current `MatchResolution` does not retain enough cursor-position residual information, extend semantic products narrowly; do not recompute in LSP.

- [ ] **Step 22.4: Filter impossible GADT cases.**

- [ ] **Step 22.5: Handle inaccessible cases.**

Keep them in residual truth but exclude illegal spelling and recommend `_` if needed.

- [ ] **Step 22.6: Run and commit.**

```text
feat(semantic): publish pattern completion contexts
```

---

# Task 23 — Integrate Residual-Space Completion in LSP

**Files:**
- Modify: `phalcom-lsp/src/completion.rs`
- Modify analysis service query plumbing if needed
- Test: completion integration tests

- [ ] **Step 23.1: Add exact missing-case completion tests.**

- [ ] **Step 23.2: Use semantic `PatternCompletionContext`.**

No enum/variant discovery from AST names.

- [ ] **Step 23.3: Implement D06-13 spelling policy.**

Contextual shorthand if provably unambiguous; qualified otherwise.

- [ ] **Step 23.4: Add GADT and visibility tests.**

- [ ] **Step 23.5: Run and commit.**

```text
feat(lsp): complete match cases from residual space
```

---

# Task 24 — Add “Add Missing Match Cases” Semantic Plan and Code Action

**Files:**
- Modify semantic tooling module
- Modify LSP code-action handler found in Task 0
- Test: semantic tooling + LSP code action tests

- [ ] **Step 24.1: Add semantic plan tests.**

Consume `CoverageWitness`/residual space and return structured missing cases.

- [ ] **Step 24.2: Define `MissingCaseEditPlan` or equivalent.**

It contains semantic pattern skeletons, not raw LSP edits.

- [ ] **Step 24.3: Generate only reachable missing cases.**

GADT impossible cases omitted.

- [ ] **Step 24.4: Handle unspellable residual with wildcard.**

- [ ] **Step 24.5: Convert plan to LSP edits.**

Use existing formatting/indent utilities.

- [ ] **Step 24.6: Run and commit.**

```text
feat(lsp): add missing match cases action
```

---

# Task 25 — Add “Generate Match” Semantic Plan and Code Action

**Files:**
- Modify semantic tooling module
- Modify LSP code-action handler
- Test: semantic tooling + LSP code action tests

- [ ] **Step 25.1: Add source expression tests for Option, Result, source enum and GADT.**

- [ ] **Step 25.2: Define `GeneratedMatchPlan`.**

Include scrutinee source span/reference, ordered reachable cases, payload binding suggestions, qualification policy.

- [ ] **Step 25.3: Avoid local-name collisions.**

Use source scope products.

- [ ] **Step 25.4: Preserve visibility/GADT constraints.**

- [ ] **Step 25.5: Render LSP edit.**

- [ ] **Step 25.6: Run and commit.**

```text
feat(lsp): generate exhaustive matches from semantics
```

---

# Task 26 — Finish Match/Associated Diagnostic Presentation

**Files:**
- Modify protocol-neutral diagnostics presenter if one exists
- Modify: `phalcom-lsp/src/diagnostics.rs`
- Modify CLI diagnostics renderer only for presentation mapping
- Test: diagnostic snapshots/golden tests

- [ ] **Step 26.1: Add non-exhaustive witness rendering tests.**

- [ ] **Step 26.2: Add GADT contradiction rendering tests.**

- [ ] **Step 26.3: Add contextual ambiguity rendering tests.**

- [ ] **Step 26.4: Source causal information from explanation DAG.**

Do not create UI-side proof messages from scratch.

- [ ] **Step 26.5: Verify CLI and LSP use equivalent protocol-neutral content.**

- [ ] **Step 26.6: Commit.**

```text
feat(diagnostics): present ADT match proofs consistently
```

---

# Phase 06.D — Cross-System Verification, Documentation, Cleanup, Final Audit

# Task 27 — Cross-Module and Package Conformance

**Files:**
- Create/extend semantic multi-module fixtures
- Create/extend core runtime project fixtures
- Create/extend LSP workspace fixtures

- [ ] **Step 27.1: Build a three-module fixture.**

```text
A declares enum
B constructs/reifies
C matches/reflects/navigates
```

- [ ] **Step 27.2: Add package facade/re-export fixture.**

- [ ] **Step 27.3: Assert identities are defining-declaration identities.**

- [ ] **Step 27.4: Assert payload field layout works across modules.**

- [ ] **Step 27.5: Assert definition/hover cross modules.**

- [ ] **Step 27.6: Run and commit.**

```text
test: cover cross-module ADT convergence
```

---

# Task 28 — Incremental Invalidation Conformance

**Files:**
- Modify/add `phalcom-semantic/tests/semantic/incremental/*`
- Add tooling incremental tests if framework supports them

- [ ] **Step 28.1: Add variant-set edit test.**

Add `C` to enum; existing A/B match becomes non-exhaustive and completion/code action sees C.

- [ ] **Step 28.2: Add GADT result edit test.**

Changing result specialization invalidates branch proof/hover/exact-case reflection.

- [ ] **Step 28.3: Add visibility edit test.**

Completion/reflection acquisition changes while exhaustiveness universe remains complete.

- [ ] **Step 28.4: Add payload external-label edit test.**

Pattern/source index/rename products invalidate.

- [ ] **Step 28.5: Reject a giant Part06 dependency.**

Fix exact dependency reads/fingerprints.

- [ ] **Step 28.6: Run and commit.**

```text
test: verify ADT tooling incremental invalidation
```

---

# Task 29 — Fuzz / Generated Robustness Suite

**Files:**
- Create repository-appropriate property/generated test module under semantic tests or fuzz harness
- Add deterministic seed corpus

- [ ] **Step 29.1: Generate closed enum universes.**

- [ ] **Step 29.2: Generate exact-case unions and nested/or/wildcard patterns.**

- [ ] **Step 29.3: Generate selector-family overload sets.**

- [ ] **Step 29.4: Generate bounded GADT specialization sets.**

- [ ] **Step 29.5: Assert properties.**

```text
no checker panic
accepted exhaustive match => empty residual
redundant alternative => no new modeled value
canonical exact-case construction idempotent
semantic target attachment deterministic
lowered execution agrees with model for generated executable cases
```

- [ ] **Step 29.6: Store minimized regressions as named tests.**

- [ ] **Step 29.7: Commit.**

```text
test: fuzz ADT match and identity invariants
```

---

# Task 30 — Reorganize Final Conformance Tests by Language Capability

**Files:**
- Reorganize `phalcom-semantic/tests/semantic/adts/*`
- Update module registrations and coverage ledger
- Reorganize/add core/LSP tests where implementation-part naming remains

- [ ] **Step 30.1: Converge semantic modules toward:**

```text
declarations.rs
constructors.rs
associated_lookup.rs
families.rs
generics.rs
gadts.rs
exact_cases.rs
matching.rs
exhaustiveness.rs
or_patterns.rs
selector_patterns.rs
visibility.rs
unions.rs
reflection.rs
modules.rs
diagnostics.rs
tooling.rs
```

- [ ] **Step 30.2: Move tests mechanically; do not change semantics during moves.**

- [ ] **Step 30.3: Run full semantic suite after each coherent move.**

- [ ] **Step 30.4: Update coverage ledger with actual passing coverage only.**

- [ ] **Step 30.5: Commit.**

```text
test: organize ADT conformance by language capability
```

---

# Task 31 — Synchronize Authoritative Language Documentation

**Files:**
- Modify authoritative ADT/GADT/associated/match docs under `docs/spec`
- Modify language reflection docs or create focused reflection spec as appropriate
- Update docs index/roadmap/ledger
- Mark/remove stale legacy enum/Option documentation

- [ ] **Step 31.1: Inventory contradictory docs.**

```bash
rg -n 'sealed.*variant|class.*variant|Some|None|MatchError|\.class.*variant|associated.*runtime' docs
```

Review matches; do not blindly replace text.

- [ ] **Step 31.2: Document native core ADTs.**

Explain `@native` semantic neutrality and Option/Result/Ordering.

- [ ] **Step 31.3: Document ExactCase status.**

Official semantic concept; do not claim authoring grammar that is not ratified.

- [ ] **Step 31.4: Document runtime case class vs semantic case/type.**

- [ ] **Step 31.5: Document reflection ontology and `::` separation.**

- [ ] **Step 31.6: Document pattern/tooling semantics where user-facing.**

- [ ] **Step 31.7: Run docs link/lint checks if repository has them.**

- [ ] **Step 31.8: Commit.**

```text
docs: finalize ADT GADT and reflection specification
```

---

# Task 32 — Compatibility and Transitional API Audit

**Files:** repository-wide based on findings.

- [ ] **Step 32.1: Inventory transitional components.**

```bash
rg -n 'legacy|compat|temporary|not lowered yet|not supported yet|Option.*special|Some.*special|None.*special' phalcom-* core docs
```

- [ ] **Step 32.2: Classify each:**

```text
keep permanently
remove now
deprecate explicitly
move to narrow compatibility boundary
```

- [ ] **Step 32.3: Delete obsolete semantic IDs/tables/resolvers.**

- [ ] **Step 32.4: Delete obsolete compiler errors/staging guards.**

- [ ] **Step 32.5: Delete stale legacy runtime family/variant paths only where new semantics fully replace them and unrelated `>>` behavior is not affected.**

- [ ] **Step 32.6: Run broad tests and commit.**

```text
refactor: remove ADT transition scaffolding
```

---

# Task 33 — Final Architectural Deletion Pass

- [ ] **Step 33.1: Search for source-string variant identity in compiler/runtime.**

```bash
rg -n 'constructor == "Some"|constructor == "None"|selector.*==.*"Some"|selector.*==.*"None"' phalcom-core phalcom-semantic
```

Allowed matches must be declaration fixtures/native registration constants only, not semantic selection.

- [ ] **Step 33.2: Search for `.class` general ADT matching.**

```bash
rg -n 'class.*Variant|variant.*class|emit_class_test' phalcom-core/src/compiler
```

Review every hit.

- [ ] **Step 33.3: Search for illicit semantic work in core/LSP.**

```bash
rg -n 'SelectorPattern::matches|CaseTypeEnvironment|PatternSpace|ExhaustivenessResult|derive_case_environment' phalcom-core/src phalcom-lsp/src
```

Expected: no source-level resolver logic; type imports in protocol-neutral DTO plumbing must be justified.

- [ ] **Step 33.4: Search for runtime proof state.**

```bash
rg -n 'Gadt.*Runtime|runtime.*equality.*proof|CaseTypeEnvironment' phalcom-core/src
```

- [ ] **Step 33.5: Search for persistent local IDs in metadata serialization.**

```bash
rg -n 'RuntimeVariantId|CaseDiscriminant|VariantTypeId|TypeId' phalcom-core/src/modules phalcom-semantic/src | grep -E 'serial|metadata|persist|artifact|snapshot'
```

Inspect, do not mechanically delete legitimate snapshot-local fields.

- [ ] **Step 33.6: Document every intentionally retained exception in final report.**

No unexplained transition debt is allowed.

---

# Task 34 — Final Vertical Architecture Proof

**Files:** create/extend end-to-end fixtures across semantic/core/LSP.

- [ ] **Step 34.1: GADT evaluator scenario.**

Use:

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}

fn eval<T>(_ expr: Expr<T>) -> T {
    match expr {
        Int(value) => value
        Bool(value) => value
    }
}
```

- [ ] **Step 34.2: Verify formal pipeline.**

```text
EnumInfo
CaseTypeEnvironment
constructor ExactCase
match candidate identities
GADT branch proof
exhaustiveness
match result T
```

- [ ] **Step 34.3: Verify execution pipeline.**

```text
semantic lowering
IsVariant
GetVariantPayload
runtime execution
no proof object
```

- [ ] **Step 34.4: Verify tooling pipeline.**

```text
exact pattern source targets
hover proof
variant definition
family-aware rename where applicable
completion excludes impossible cases
```

- [ ] **Step 34.5: Verify reflection pipeline.**

```text
Variant descriptor
ExactCaseType descriptor
runtime case class
all distinct but linked
```

- [ ] **Step 34.6: Verify incremental edit.**

Change one GADT result specialization and prove affected products recompute.

- [ ] **Step 34.7: Native Option/Result parallel vertical scenarios.**

Prove semantic parity despite different physical representation.

- [ ] **Step 34.8: Commit.**

```text
test: prove ADT architecture end to end
```

---

# Task 35 — Final Verification Matrix

Do not claim Part 06 complete without fresh command output.

## 35.1 Repository state

- [ ] Run:

```bash
git status --short
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git log --oneline --decorate -30
```

## 35.2 Formatting

- [ ] Run:

```bash
cargo fmt --all -- --check
```

Required: exit 0.

## 35.3 Front end

- [ ] Run:

```bash
cargo check -p phalcom-ast
cargo test -p phalcom-ast -- --nocapture
```

## 35.4 Semantic layer

- [ ] Run:

```bash
cargo check -p phalcom-semantic
cargo test -p phalcom-semantic --lib -- --nocapture
cargo test -p phalcom-semantic --test semantic -- --nocapture
```

## 35.5 Core/runtime

- [ ] Run:

```bash
cargo check -p phalcom-core
cargo test -p phalcom-core --lib -- --nocapture
cargo test -p phalcom-core --tests -- --nocapture
```

## 35.6 LSP/tooling

- [ ] Run the full LSP suite using the repository's actual test targets:

```bash
cargo check -p phalcom-lsp
cargo test -p phalcom-lsp -- --nocapture
```

If LSP has multiple integration targets, enumerate and run all of them.

## 35.7 Part-05 regression

- [ ] Re-run all landed 05.1/05.2 focused match suites.

At minimum structural equivalents of:

```text
match semantics
match lowering
match runtime
family patterns
GADT runtime
pattern contexts
Option/native core matching
```

## 35.8 Native core semantic checklist

Verify:

- [ ] Option is an ordinary `@native enum` semantic declaration.
- [ ] Result is an ordinary `@native enum` semantic declaration.
- [ ] Ordering is an ordinary core ADT semantic declaration.
- [ ] Bool remains primitive.
- [ ] Option/Result/Ordering variants use ordinary `VariantId`/families/fields.
- [ ] Exact cases are canonical `TypeId`s.
- [ ] Option immediate representation remains correct.
- [ ] no semantic core Option compatibility table remains.

## 35.9 Reflection checklist

- [ ] dedicated enum/variant/family/field/exact-case metaobjects exist.
- [ ] `::` has no reflection fallback.
- [ ] `.class` returns case behavior class.
- [ ] case class is linked to, but distinct from, variant/exact-case metadata.
- [ ] specialized exact cases reflect canonically.
- [ ] runtime reflection does not keep/re-resolve full semantic snapshot.
- [ ] visibility policy is tested.

## 35.10 Tooling checklist

- [ ] exact/contextual patterns attach same VariantId.
- [ ] family patterns attach VariantFamilyId + candidates.
- [ ] payload labels attach field identity/target set.
- [ ] go-to-definition works across modules.
- [ ] hover uses TypePresenter/semantic proof products.
- [ ] completion uses residual semantic space.
- [ ] impossible GADT cases are omitted.
- [ ] missing-case and generate-match actions use semantic plans.
- [ ] family rename is semantic.
- [ ] external label/local name rename are distinct.
- [ ] semantic highlighting is not name-heuristic.
- [ ] diagnostics use explanation DAG/witnesses.

## 35.11 Incremental/persistence checklist

- [ ] variant-set edit invalidates exhaustiveness/tooling/reflection.
- [ ] GADT result edit invalidates exact-case/proofs/tooling.
- [ ] visibility edit invalidates reflection/completion without corrupting exhaustiveness.
- [ ] stable metadata survives remapping to new local IDs.
- [ ] runtime discriminants never become persistent semantic identity.

## 35.12 Architecture searches

- [ ] Run Task-33 searches again on final HEAD and record exact output.

## 35.13 Documentation

- [ ] Authoritative docs agree with implementation.
- [ ] No authoritative document claims variants are source classes.
- [ ] No authoritative document claims Option has separate semantics.
- [ ] ExactCase presentation is not accidentally documented as parseable syntax unless separately ratified.
- [ ] Performance/optimization work is explicitly handed to Part 07.

---

# Task 36 — Final Implementation Report and Part-07 Handoff

The implementing agent must report:

```text
starting HEAD
final HEAD
branch
commits created
files changed
actual Part-05.2 name-map deviations
native Option migration architecture
native Result migration architecture
Ordering migration
Option physical representation status
ExactCase canonicalization/reflection status
.class/case behavior reflection status
runtime reflection metadata architecture
source-index target extensions
hover/definition/completion/rename/highlighting/code-action status
incremental/persistence results
legacy paths deleted
all verification commands + exact outcomes
remaining known limitations
```

The Part-07 handoff should contain only optimization/performance work discovered during Part 06, for example:

```text
match decision DAG/jump tables
native Result representation optimization
reflection metadata size optimization
source-index query performance
large-enum exhaustiveness performance
```

Do not transfer semantic correctness/tooling incompleteness to Part 07 and still call Part 06 complete.

---

# Part-06 Completion Statement

Part 06 is complete when core and user ADTs are semantically indistinguishable except for explicit native implementation provenance; exact cases are canonical, reflectable and dispatch-safe; runtime case classes remain a distinct reflectable behavioral layer; reflection/source tooling consume formal compiler products; all transitional ADT semantics are deleted; and the complete architecture passes vertical, cross-module, incremental and persistence verification.

---

# Appendix A — Requirements Traceability

The Part-06 requirements analysis maps to implementation tasks as follows:

| Requirement family | Primary tasks | Verification gate |
| --- | --- | --- |
| `R06-CORE-01..09` | 1–6 | 35.8 + native vertical scenarios in 34 |
| `R06-TYPE-01..06` | 7–11 | 35.8–35.9 + GADT vertical scenario |
| `R06-REFL-01..09` | 9–15 | 35.9 |
| `R06-IDX-01..06` | 16–18 | 35.10 |
| `R06-HOVER-01..04` | 19 | 35.10 |
| `R06-COMP-01..06` | 22–25 | 35.10 |
| `R06-RENAME-01..04` | 21 | 35.10 |
| `R06-HL-01..02` | 20 | 35.10 |
| `R06-DIAG-01..04` | 26 | 35.10 |
| `R06-INCR-01..04` | 27–28 | 35.11 |
| `R06-PERSIST-01..03` | 12, 15, 28 | 35.11 |
| `R06-DOC-01..02` | 31 | 35.13 |
| `R06-CLEAN-01..04` | 6, 32, 33 | 35.12 |
| `R06-TEST-01..03` | 27–30, 34 | 35.3–35.7 |

A clean Part-06 completion claim requires every row above to be complete. A passing runtime suite does not substitute for unfinished reflection/tooling, and a passing LSP suite does not substitute for unresolved semantic/native migration debt.
