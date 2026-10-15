# PERF003.C2.P7 — Audit Slice 3 Report

**Audit slice:** 3  
**Date:** 2026-10-15  
**Repository:** `aureat/phalcom-lang`  
**Branch:** `perf/universe-characterization`  
**Audited remote HEAD:** `2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a` — `feat(perf): extend artifact loading foundations`  
**Primary scope:** loaded semantic image → immutable canonical workspace parent → compiler-reader/source-denial closure; accepted callable definitions; static owner families; formal projection and P6 indexes; query-image reconstruction; residual source dependencies; standalone-package compilation acceptance.

---

# 1. Executive conclusion

Slice 3 confirms that the **workspace overlay half of the architecture is substantially better than the raw artifact importer**.

Once a genuinely complete `CanonicalUniverseFormalImage` exists, `SemanticWorkspaceSession::from_image` and the first local publication correctly preserve major canonical tables through immutable parent references rather than copying their contents. The current canonical-image integration tests already prove pointer identity for inherited static definitions, static constants, callable definitions, module-static bindings, constructor initializations, trait headers/surfaces, conformance authority, aliases, dispatch surfaces, and other parent-backed products.

That is an important positive result: **P5/P6's parent-image consumption model is viable** and should not be replaced with a flattened imported workspace snapshot.

However, the actual `.telosc` artifact path still cannot construct the immutable parent authority that this overlay model expects. Slice 3 found several independent blockers beyond the callable-analysis owner graph from Slice 2:

1. **Accepted callable definitions are not persisted.** Import currently manufactures one `PrimaryDeclaration` definition from every callable signature, assigning `source_member_index = 0` and `inherent_body_context = None`. This destroys exact inherent-impl/conformance-witness provenance and can mislead lowering/query reconstruction.
2. **Operational static templates are imported and then orphaned.** The loader decodes them into `ValidatedSemanticParts.operational_templates`, but the reconstructed `SemanticSnapshot.static_definitions` remains empty and `phalcom-core` discards the standalone vector when it creates the formal image.
3. **Required static owner families remain completely empty** in the reconstructed snapshot: static case catalogs/signatures, static definitions, static constants, and module static bindings.
4. **Derived-conformance and trait-dispatch authority remain empty** even though user compilation consumes them for generic evidence and call lowering.
5. **Formal projection remains empty**, while the compiler depends on it for exact body-local binding attachments.
6. **Query-image reconstruction is incomplete in two separate ways:**
   - `ModuleStaticBindingCatalog` and `ModuleStaticBindings` are exported as reusable query roots but have no loaded-owner adapter.
   - the exported `dependency_evidence` set is validated but then silently discarded on load.
7. **Query reconstruction is not allocation-closed.** It allocates ordinary `BTreeMap` nodes, products, JSON-decoded owner objects, strings, and other structures outside the cumulative allocation ledger.
8. **The current “P7 end-to-end” test is aspirational, not executed acceptance.** It is blocked during artifact construction by the fail-closed semantic export guard. Even when enabled it tests only an inline `print(42)` compilation—not a standalone package and not runtime execution.
9. **There are residual syntax/source dependencies in compiler lowering** for inherent delegation and trait-default detached-method discovery. These must be replaced by owner-held semantic authority for any canonical module that may be lowered without source.
10. **The standalone-package image path itself is structurally present and well-directed.** `EntrySelection::Package` loads only user/package source, resolves Universe through `image.modules()`, and creates `SemanticWorkspaceSession::from_image`. This is a solid base for the final P7 package acceptance test.

The resulting closure rule is:

> **Completing `CallableAnalysis` is necessary, but P7 will still fail until the artifact reconstructs the exact canonical parent tables that `SemanticWorkspaceSession::from_image` already knows how to consume.**

The implementer should therefore treat the loaded snapshot as a **canonical immutable semantic parent**, not as a reduced compilation summary.

---

# 2. Scope audited in this slice

The audit traced these paths:

```text
SEMANTIC_WORLD
    ↓
validate_and_import_semantic_world*
    ↓
SemanticSnapshot
    ↓
CanonicalUniverseFormalImage::from_loaded_artifact
    ↓
SemanticDb::with_canonical_image
    ↓
SemanticWorkspaceSession::from_image
    ↓
local workspace publication with canonical parents
    ↓
ProgramAnalyzer::{analyze_entry_selection_with_image,
                  discover_and_analyze_with_image}
    ↓
semantic lowering / metadata roots / compiler
```

Files inspected include:

- `phalcom-semantic/src/snapshot.rs`
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/impls.rs`
- `phalcom-semantic/src/presentation.rs`
- `phalcom-semantic/src/module_bindings.rs`
- `phalcom-semantic/src/static_constant.rs`
- `phalcom-semantic/src/static_definition.rs`
- `phalcom-semantic/src/artifact/export.rs`
- `phalcom-semantic/src/artifact/import.rs`
- `phalcom-semantic/src/artifact/query.rs`
- `phalcom-semantic/src/artifact/operational.rs`
- `phalcom-semantic/src/canonical_image.rs`
- `phalcom-semantic/src/db/mod.rs`
- `phalcom-core/src/modules/canonical_artifact.rs`
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/modules/compile.rs`
- `phalcom-core/tests/core/modules/canonical_artifact.rs`
- `phalcom-semantic/tests/semantic/integration/canonical_image.rs`
- `phalcom-semantic/tests/semantic/artifact_allocation.rs`

No GitHub combined-status entries or PR workflow runs are associated with the audited HEAD.

---

# 3. Severity summary

| ID | Severity | Finding |
|---|---|---|
| S3-01 | **CRITICAL** | Artifact import fabricates every callable definition as `PrimaryDeclaration`, losing inherent-impl/conformance-witness origin, source-member identity, and body context. |
| S3-02 | **CRITICAL** | Imported operational static templates are orphaned: they are never attached to reconstructed `StaticDefinitionProduct`s and are discarded by the core loader. |
| S3-03 | **CRITICAL** | Required static owner families are deliberately empty in the loaded snapshot: static cases, static definitions, static constants, and module-static bindings. |
| S3-04 | **CRITICAL** | `derived_conformance_rules` and `trait_dispatch_index` are empty in the loaded snapshot despite being compiler inputs for generic/trait call lowering. |
| S3-05 | **HIGH** | `formal_projection` is empty, but compiler lowering requires exact formal binding attachments; broad `source_index` is intentionally omitted. |
| S3-06 | **HIGH** | `ModuleStaticBindingCatalog` and `ModuleStaticBindings` are classified/exported as supported reusable query roots but `product_and_fingerprints` has no loader adapter for them. |
| S3-07 | **CRITICAL** | Exported `QUERY_IMAGE.dependency_evidence` is never reconstructed. Canonical aggregate dependency fingerprints are silently lost. |
| S3-08 | **HIGH** | Query-image materialization allocates ordinary `BTreeMap`s and products outside the shared `AllocationBudget`, so the cumulative load ledger remains incomplete even after semantic import is repaired. |
| S3-09 | **HIGH** | `ReconstructedModuleOwnerIndexes` are string/debug-keyed, cover only declarations/data/enums, allocate natively, and are discarded by the core loader; they do not satisfy P6/T8 reader indexes. |
| S3-10 | **HIGH** | `callable_definitions_by_module` is reconstructed from signatures instead of accepted definitions, so even cardinality can be wrong when signatures and definitions are not 1:1. |
| S3-11 | **HIGH** | Inherent delegation lowering still reads `snapshot.sources` to determine generated delegation and target field. |
| S3-12 | **HIGH** | Trait-default detached-method discovery still scans trait source AST and source ranges. Source-free canonical lowering cannot reconstruct those entries. |
| S3-13 | **MEDIUM-HIGH** | `project_anonymous_products` is source-AST based. This is acceptable for user modules but cannot be used as a hidden fallback for canonical imported modules. |
| S3-14 | **HIGH** | Module-static-binding construction still has source-driven builders and name/range matching; the imported canonical image must persist the final `ModuleStaticBindingIndex` rather than rederive it. |
| S3-15 | **HIGH** | Source-free completeness is primarily non-empty/cardinality based and does not establish definition/signature/index parity or exact owner coverage. |
| S3-16 | **HIGH** | Current canonical artifact test cannot reach load/compile because artifact construction is intentionally blocked; its later assertions are not current evidence. |
| S3-17 | **HIGH** | Existing artifact acceptance covers only `EntrySelection::Inline("print(42)")`; no persisted-image standalone package (`parse`, `similarity`, `slotmap`) is compiled and run. |
| S3-18 | **MEDIUM** | P7 artifact capability is intentionally compiler-only (`portable_executable = false`). A runtime package test must distinguish “compiler source-free” from P8's future source-free executable Universe. |
| S3-19 | **POSITIVE / architectural** | `SemanticWorkspaceSession::from_image` and first publication correctly reuse canonical parent tables with pointer identity and no canonical entry copying. |
| S3-20 | **POSITIVE / architectural** | `EntrySelection::Package` and standalone-module paths already resolve Universe as an external module world from the image and analyze only local source. |

---

# 4. The canonical parent overlay architecture is sound

The most important positive finding is in `phalcom-semantic/src/session.rs`.

`SemanticWorkspaceSession::from_image` does **not** flatten the canonical Universe snapshot into mutable local maps. It attaches immutable parents:

```rust
base_declarations:
    DeclarationTypeTable::with_parent(snapshot.declarations.clone()),

base_hierarchy:
    MapTypeHierarchy::with_parent(snapshot.hierarchy.clone()),

base_dispatch:
    SurfaceDispatchResolver::with_parent(snapshot.dispatch.clone()),

base_callable_signatures:
    CallableSignatureTable::with_parent(snapshot.callable_signatures.clone()),

base_field_signatures:
    FieldSignatureTable::with_parent(snapshot.field_signatures.clone()),

base_data_semantics:
    DataSemanticTable::with_parent(snapshot.data_semantics.clone()),

base_enum_semantics:
    EnumSemanticTable::with_parent(snapshot.enum_semantics.clone()),

base_associated_surfaces:
    AssociatedFamilyTable::with_parent(snapshot.associated_surfaces.clone()),

base_type_member_declarations:
    ParentMap::with_parent(snapshot.type_member_declarations.clone()),

base_enum_requirements:
    EnumRequirementTable::with_parent(snapshot.enum_requirements.clone()),
```

During the first local publication, additional families are seeded from the canonical snapshot:

```text
callable definitions         → ParentMap parent
conformance index            → immutable parent
derived conformance rules    → immutable parent
static definitions           → StaticDefinitionTable parent
static constants             → StaticConstantTable parent
module static bindings       → ParentMap parent
conformance witness plans    → copied immutable Arc products
```

The existing canonical-image integration test verifies pointer identity across multiple workspace revisions for many of these products.

## Finding S3-19 — positive architecture

**Do not redesign this into a giant copied workspace snapshot.**

The artifact loader should reconstruct the exact immutable canonical parent once. Local sessions should continue layering workspace products over it.

That preserves:

- P5 sharing;
- P6 local-complexity behavior;
- canonical identity stability;
- no repeated Universe semantic construction;
- constant-time parent attachment for many families.

---

# 5. Accepted callable definitions are a hard semantic owner

## 5.1 Live authority

`EffectiveCallableDefinition` contains:

```rust
pub struct EffectiveCallableDefinition {
    pub callable: CallableId,
    pub origin: CallableDefinitionOrigin,
    pub source_member_index: usize,
    pub signature: CallableSemanticSignature,
    pub inherent_body_context: Option<InherentBodyContext>,
}
```

with:

```rust
pub enum CallableDefinitionOrigin {
    PrimaryDeclaration,
    InherentImpl(ImplId),
    ConformanceWitness(ImplId),
}
```

These are not source decorations.

Core lowering uses them to distinguish:

- target-owned primary bodies;
- inherent implementation members;
- conformance witness methods;
- detached methods;
- covering/generic body context.

## 5.2 Current import is fabricated

`phalcom-semantic/src/artifact/import.rs` currently does approximately:

```rust
for rec in &world.callable_signatures {
    let sig = import_callable_signature(...)?;

    callable_definitions_map.insert(
        sig.callable.clone(),
        EffectiveCallableDefinition {
            callable: sig.callable.clone(),
            origin: CallableDefinitionOrigin::PrimaryDeclaration,
            source_member_index: 0,
            signature: sig.clone(),
            inherent_body_context: None,
        },
    );

    callable_signatures.insert(sig);
}
```

This assumes:

```text
callable signature == accepted callable definition
```

That invariant is false.

## Finding S3-01 — CRITICAL

The artifact needs an explicit accepted-definition DTO.

Minimum semantic payload:

```text
Stable callable
Definition origin:
    PrimaryDeclaration
    InherentImpl(stable impl)
    ConformanceWitness(stable impl)
source_member_index
accepted signature reference / exact signature authority
inherent_body_context?
compiler-required delegation/default-body attributes
```

Import must validate the relation between:

- definition owner;
- callable signature;
- impl/conformance authority;
- body analysis where applicable.

It must never infer origin from the callable owner shape alone.

---

# 6. The fabricated definition can break more than lowering

The loaded query system reconstructs `QueryKey::CallableDefinition` products from:

```rust
snapshot.callable_definitions.get(callable)
```

and computes the definition fingerprint from:

- signature fingerprint;
- origin;
- impl identity;
- `source_member_index`;
- `inherent_body_context`.

Therefore fabricated `PrimaryDeclaration / 0 / None` definitions do not merely produce a less precise backend view; they alter the formal callable-definition product.

This can make:

- canonical query reconstruction incorrect;
- dependency fingerprints incorrect;
- local incremental dependencies observe the wrong canonical leaf;
- conformance/inherent body semantics diverge.

The definition owner must be repaired **before** callable-definition query roots can be certified.

---

# 7. `callable_definitions_by_module` is reconstructed from the wrong authority

The importer currently builds:

```rust
for (_, sig) in callable_signatures.iter() {
    callable_definitions_by_module
        .entry(sig.callable.owner.module().clone())
        .or_default()
        .push(sig.callable.clone());
}
```

The index is called:

```text
callable_definitions_by_module
```

but its source is the signature table.

## Finding S3-10 — HIGH

The index must be derived from **accepted definitions**, not signatures.

Reasons:

1. A signature may exist without an executable accepted definition.
2. Bodyless/abstract/native/trait-surface contracts need not imply an accepted source definition.
3. Conformance and generated definitions have different owner semantics.
4. P6 local module iteration is supposed to iterate exactly the backend-lowerable definitions.

Correct construction:

```rust
for (callable, definition) in callable_definitions.iter() {
    by_module[callable.module()].push(callable);
}
sort/dedup/validate
```

It should then be compared against the source-built snapshot's `with_callable_definitions` result.

---

# 8. Current “module owner indexes” do not satisfy P6

`ReconstructedModuleOwnerIndexes` currently contains:

```rust
pub struct ReconstructedModuleOwnerIndexes {
    pub declarations_by_module: BTreeMap<String, Vec<StableDeclarationRef>>,
    pub data_by_module: BTreeMap<String, Vec<StableDeclarationRef>>,
    pub enums_by_module: BTreeMap<String, Vec<StableDeclarationRef>>,
}
```

Construction uses:

```rust
let key = format!("{:?}", shell.declaration.module);
```

or equivalent for data/enums.

## Finding S3-09 — HIGH

This object should not be treated as P6/T8 completion evidence.

Problems:

- keys are debug-formatted strings instead of typed `ModuleId` / stable module refs;
- it covers only three owner families;
- native `BTreeMap`/`Vec` allocation is outside the exact semantic load model;
- it is returned in `ValidatedSemanticParts`;
- `phalcom-core::load_universe_telosc_from_bytes` never consumes it;
- it therefore has zero effect on the loaded compiler image.

P6 reader complexity should be implemented **inside the actual snapshot/table owners**, not as a detached debug index.

Required module-bounded readers include at least:

- declarations;
- callable definitions;
- callable analyses where required by canonical reader;
- static definitions;
- static constants;
- module-static bindings;
- data/enum/traits as needed by lowering;
- accepted impl/conformance owners.

---

# 9. Static operational templates are currently orphaned

Export currently walks:

```rust
snapshot.static_definitions.iter()
```

and extracts:

```rust
product.checked_body.root
```

only when that root is:

```rust
StaticDefinitionBodyRepr::Operational(template)
```

Those templates are encoded in:

```text
checked_definitions
```

Import decodes them into:

```rust
ValidatedSemanticParts {
    operational_templates: Vec<OperationalTemplate>,
    ...
}
```

But the reconstructed snapshot is created with:

```rust
static_definitions:
    Arc::new(StaticDefinitionTable::new())
```

and the core artifact loader uses only:

```rust
parts.snapshot
```

when creating `CanonicalUniverseFormalImage`.

`parts.operational_templates` is not attached anywhere.

## Finding S3-02 — CRITICAL

The current operational-template work is **dead-end loader state**.

A successfully decoded operational static definition is lost before the image is published.

This must be repaired by reconstructing the actual canonical static-definition owner:

```text
StaticDefinitionProduct
    definition:
        id
        signature
        body representation / source-independent operational body
        source provenance as required
    diagnostics as required
    alias_info if required
    checked_body:
        owner
        operational root
        compiler/evaluator-required checked facts
```

or by explicitly redesigning `StaticDefinitionProduct` to allow a source-free imported operational-body representation.

A free-floating vector is not enough.

---

# 10. Static definition transport is structurally incomplete

`CheckedStaticDefinitionsV1` currently encodes:

```text
owner
parameter_count
local_count
operational root
```

That is useful evaluator code, but it is not the full `StaticDefinition` semantic product.

The reconstructed local workspace expects `snapshot.static_definitions` to be a real table with a canonical parent.

## Finding S3-03A — CRITICAL

The static-definition artifact must preserve enough owner data to answer all ordinary consumers without source:

- definition identity;
- signature and result domain;
- parameter identities/categories;
- operational checked body;
- dependency identity/fingerprint as required;
- aliases/type projection if relevant;
- checked static applications/constructions when semantically referenced by denotations;
- exact static-body ownership.

Do not treat the operational tree alone as the static-definition table.

---

# 11. Static constants are entirely absent from loaded authority

`StaticConstantProduct` contains:

```text
stable constant id
declaration/initializer ranges
checked initializer
outcome
residuals
value
last-known-good value
diagnostics
```

Backend lowering currently uses successful products through:

```rust
project_static_constants(module, snapshot)
```

and materializes:

```text
StaticConstantLoweringSpec
    id
    MaterializationDescriptor
```

The imported snapshot currently has:

```rust
StaticConstantTable::new()
```

## Finding S3-03B — CRITICAL

At minimum, P7's canonical compiler image must restore every successful Universe constant needed by:

- imported static definition evaluation;
- compile-time type/static operations;
- runtime materialization of constants;
- module-value checking.

If P7 intentionally omits re-evaluable source/checker state for a constant, the imported product must still contain the **canonical reduced result and dependency identity** necessary for compiler consumers.

Do not re-run constant initializers from source on load.

---

# 12. Module-static binding authority is required, not optional

`ModuleStaticBindingIndex` is the bridge from top-level module declarations into ordinary checker/static-body consumption.

It carries:

```rust
ModuleBindingFact {
    symbol,
    source_body,
    source_binding,
    phase,
    state,
    declaration_range,
    initializer_range,
}
```

The live source builder:

```rust
ModuleStaticBindingIndex::from_module_analysis(
    module,
    program,
    analysis,
)
```

uses the AST only to identify source binding/phase and then pairs it with final checker state.

The imported image currently sets:

```rust
module_static_bindings:
    ParentMap::default()
```

## Finding S3-03C / S3-14 — CRITICAL/HIGH

Persist the **final index**, not enough inputs to rerun `from_module_analysis`.

For canonical Universe import, the artifact must encode:

- stable `SymbolId`;
- source `<main>` body owner;
- body-local binding ordinal relocated through the callable owner;
- static vs runtime phase;
- final `BindingState`;
- declaration/initializer attachment ranges if compiler-required;
- visible-name/import aliases required for scope seeding.

Name/range matching is acceptable only during source construction. It must not be part of artifact reconstruction.

---

# 13. Query image declares module-static queries reusable but cannot load them

`portable_query_kind` exports:

```rust
QueryKey::ModuleStaticBindingCatalog(_)
    → PortableQueryKindV1::ModuleStaticBindingCatalog

QueryKey::ModuleStaticBindings(_)
    → PortableQueryKindV1::ModuleStaticBindings
```

`resolve_query_key` also decodes both portable kinds.

But `product_and_fingerprints` has no case for either.

It terminates with:

```rust
_ => Err(incomplete(
    "query kind has no loaded semantic owner adapter"
))
```

## Finding S3-06 — HIGH

Once the semantic artifact actually restores module binding roots, `QUERY_IMAGE` can contain a root that the loader advertises as supported and then cannot materialize.

This is a concrete internal contract mismatch:

```text
exporter: supported
wire enum: supported
key decoder: supported
loader product adapter: unsupported
```

Repair by adding exact owner adapters and shared product/input fingerprint functions, or temporarily classify those kinds as non-reusable until the semantic owner exists.

The former is preferred because P7 requires these canonical products.

---

# 14. Query dependency evidence is silently discarded

The query artifact has two concepts:

1. `supported_roots`
2. `dependency_evidence`

The exporter deliberately emits `dependency_evidence` from the canonical aggregate dependency fingerprint map even for products that are not reusable query roots.

Validation checks the rows:

```rust
for evidence in &image.dependency_evidence {
    if evidence.key.kind.profile_support()
        == OmittedSourceDependent
    {
        reject
    }
}
```

But `reconstruct_query_image_inner` never reads `image.dependency_evidence` after validation.

It builds returned fingerprints only from:

```rust
image.supported_roots
```

## Finding S3-07 — CRITICAL

This defeats the purpose of aggregate dependency evidence.

`SemanticDb` explicitly consults:

```rust
canonical_image.dependency_fingerprint(key)
```

when:

- recording local query dependencies;
- deciding whether canonical dependencies are immutable leaves;
- validating observed dependency fingerprints.

If aggregate evidence-only keys are dropped, a local workspace can no longer record/validate dependencies against those canonical owners.

### Required repair

Reconstruct:

```text
BTreeMap<QueryKey, ProductFingerprint>
```

from both:

- supported root products; and
- every `dependency_evidence` row.

For evidence-only rows:

1. decode the key;
2. locate its canonical snapshot aggregate owner;
3. compute the correct canonical fingerprint through a shared adapter;
4. validate stable aggregate ownership;
5. insert it into `dependency_fingerprints`;
6. reject duplicates/conflicts.

Do not convert it into a reusable `CanonicalQueryEntry` unless the kind is eligible.

---

# 15. Aggregate fingerprint adapters are too narrow

`aggregate_structural_fingerprint` currently supports only:

```text
CallableSignature
FieldSignature
DeclarationSurface
TypeMemberDeclaration
```

Everything else returns `None`.

Yet the artifact/query design includes a wider set of reusable/aggregate owner categories.

## Finding S3-07A — HIGH

Once dependency evidence is actually reconstructed, the canonical image needs a single authoritative adapter for every persisted aggregate-only dependency kind.

Do not maintain multiple partially overlapping fingerprint switch statements in:

- query exporter;
- query importer;
- canonical-image validation;
- SemanticDb bootstrap.

Prefer one shared:

```text
fingerprint_for_persisted_owner(snapshot, modules, key)
```

with a declared disposition:

```text
ReusableRoot(product + input fp)
EvidenceOnly(product fp)
OmittedSourceDependent
Unsupported
```

This reduces drift.

---

# 16. Query reconstruction remains outside the exact allocation ledger

`reconstruct_query_image_inner` uses the budget for:

- temporary root vector;
- dependency edge vectors;
- final outer `Arc`.

But then constructs:

```rust
let mut entries = BTreeMap::new();
let mut fingerprints = BTreeMap::new();
```

and inserts one row per loaded root.

It also calls owner adapters that allocate:

- `ResolvedImportsProduct` BTreeMaps;
- enum variant Vecs;
- Arcs;
- stable-ID decoded strings/boxed arrays;
- error strings;
- other product internals.

## Finding S3-08 — HIGH

Even after the semantic-world `UnaccountedBoundary` is removed, query reconstruction will remain an unaccounted allocation boundary.

The allocation test currently validates **query closure validation scratch**, not full `CanonicalSemanticQueryImage` reconstruction.

### Recommended architecture

As in Slice 2:

- frozen immutable sorted root table;
- frozen dependency-fingerprint table;
- budgeted dense storage;
- allocation-free binary search / typed lookup;
- no hidden BTree reconstruction on load.

Workspace `SemanticDb` can continue using native mutable maps because it is not part of artifact materialization. The imported canonical image should use frozen storage.

---

# 17. Query-owner decoding itself allocates outside the ledger

Portable query owner keys are JSON strings.

`resolve_query_key` uses owned deserialization into stable identity structures.

That can allocate:

- strings;
- path arrays;
- selector strings;
- boxed names.

These allocations are not charged by the artifact `AllocationBudget`.

## Finding S3-08A — MEDIUM-HIGH

Either:

1. account JSON owner decoding explicitly; or
2. replace the query owner key encoding with the same structured stable identity codec already used by semantic/module sections.

A structured portable owner record would also reduce:

- parse overhead;
- duplicate string validation;
- JSON dependency;
- transient allocations.

This is not necessarily a blocker to semantic correctness, but it is relevant to P7's exact cumulative allocation contract.

---

# 18. Derived conformance and trait dispatch remain absent

The loaded snapshot currently explicitly constructs:

```rust
derived_conformance_rules:
    DerivedConformanceRuleIndex::new()

trait_dispatch_index:
    TraitDispatchIndex::default()
```

The export guard simultaneously rejects nonempty source values for both.

Core compilation consumes them.

Examples:

```rust
snapshot.derived_conformance_rules.get_rule(impl_id)
```

is used while creating generic evidence and runtime call environments.

## Finding S3-04 — CRITICAL

Persisting only explicit conformance heads and reduced witness callables is not sufficient.

The imported canonical image needs exact:

- derived-conformance rules;
- guards/applicability data;
- generic signature;
- source impl identity;
- trait/target templates;
- fingerprint authority;
- dispatch buckets / reconstructible trait dispatch index.

### Derived vs reconstructed index

A useful distinction:

- **derived conformance rule** is semantic authority → persist.
- **trait dispatch lookup index** may be a derived performance index → rebuild in O(N), if rebuilding does not re-solve semantics and is allocation-accounted.

Do not serialize redundant lookup buckets if a deterministic bounded rebuild from persisted heads/rules/surfaces is cheaper and semantically exact.

---

# 19. Witness plans remain only partially portable

The current importer does reconstruct some `ConformanceWitnessPlan` fields, but Slice 1 already established that required:

- requirement views;
- invalid explicit members;
- nontrivial failures;
- conditional applicability;
- other plan authority

are guarded out.

Core session code inherits canonical witness plans and skips rebuilding Universe plans:

```rust
if db.canonical_image().is_some()
    && contribution.source_module.project.is_universe()
{
    continue;
}
```

This is exactly the behavior P7 wants—but only if imported plans are complete.

## Implication

A partially reconstructed plan cannot be repaired later by a local user workspace session. Universe plan construction is intentionally bypassed.

Therefore witness-plan completeness is a **load-time invariant**, not deferred work.

---

# 20. Formal projection is currently empty but is a compiler reader

Loaded snapshot:

```rust
formal_projection:
    FormalSemanticProjection::default()
```

Core lowering does:

```rust
snapshot.formal_projection
    .binding_attachment_range(callable, binding)
```

to project parameter/local declaration attachments.

Slice 2 identified the dangerous fallback:

```text
missing formal attachment
+ parameter exists
+ broad source_index has no parameter site
→ treat as implicit receiver
```

Since broad `source_index` is intentionally empty after artifact load, this can hide an incomplete imported formal projection.

## Finding S3-05 — HIGH

Rebuild or persist the **compiler-required formal projection subset** before publishing the image.

The broad source semantic index should stay absent.

Required attachment authority should be derivable from:

- callable owner;
- relocated binding ordinal;
- parameter identity;
- persisted formal declaration attachment;
- explicit “generated/implicit receiver has no source declaration” disposition.

No source lookup is necessary.

---

# 21. Residual source dependency: inherent delegation

`semantic_lowering.rs` currently detects generated delegation through:

```rust
snapshot.sources
    .get(&impl_id.module)
    .and_then(|source|
        source.program.statements.get(impl_id.local.0 as usize)
    )
    .and_then(|statement| ...)
```

It also extracts:

```text
delegation.target_field
```

from source AST.

## Finding S3-11 — HIGH

This is backend-required accepted-definition authority and must not be rediscovered from source.

Extend the accepted callable definition / impl member product with something like:

```text
ImplementationMemberKind:
    Body
    GeneratedDelegation {
        target_field
    }
```

or an equivalent semantic lowering descriptor.

Then:

```text
is_generated_delegation
delegation_target_field
```

come directly from the accepted definition.

This also removes repeated AST walking during ordinary compilation.

---

# 22. Residual source dependency: trait default methods

Current detached-method construction scans:

```rust
snapshot.sources[module].program.statements
```

for trait declarations, enumerates trait members, matches source ranges against the semantic trait surface, and constructs:

```text
DetachedMethodSource::TraitDefault {
    trait_declaration,
    source_member_index
}
```

## Finding S3-12 — HIGH

The trait surface/default-body authority should already know:

- which requirement has a default implementation;
- the exact callable/body identity;
- the source member ordinal if backend mapping requires it.

Persist/reconstruct that owner relation directly.

Do not match semantic members back to source by range.

This is especially important because source ranges are provenance, not stable semantic identity.

---

# 23. Anonymous product lowering is source-backed

At the end of module semantic lowering:

```rust
let anonymous_products = snapshot
    .sources
    .get(module)
    .map(|source|
        project_anonymous_products(
            module,
            snapshot,
            &source.program,
            runtime_type_roots
        )
    )
    .unwrap_or_default();
```

## Finding S3-13 — MEDIUM-HIGH

This is acceptable today for **local user/package modules**, because P7 does not remove user source.

It is not acceptable as a fallback for imported canonical Universe modules.

The final source-denied test must prove:

```text
runtime_modules excludes canonical Universe modules
```

or, where canonical modules need backend lowering, their anonymous-product lowering plan is persisted/derived without source.

Do not accidentally broaden `runtime_modules` later without revisiting this assumption.

---

# 24. User-module metadata source index is not automatically a P7 bug

`compile.rs` seeds runtime metadata roots by scanning:

```rust
analyzed.semantic.source_index.module(module)
```

for class/data declarations.

In the loaded-image program path:

```rust
runtime_modules =
    linked.modules.keys()
        .filter(|id| !id.project.is_universe())
```

so this source index pertains to current user/package modules, whose source is present and analyzed locally.

## Assessment

This is **not** a P7 source-denial violation by itself.

The invariant to keep is:

> A P7 loaded Universe image removes Universe source dependency, not application/package source dependency.

However, canonical external metadata roots must still come from the canonical metadata/image, never from missing Universe source indexes.

---

# 25. Standalone package image path already exists

`ProgramAnalyzer::analyze_entry_selection_with_image` supports:

```rust
EntrySelection::Package(pkg_dir)
```

It:

1. canonicalizes the package directory;
2. requires `package.ph`;
3. requires executable `main.ph`;
4. loads a standalone package project;
5. constructs `entry_id = ...::main`;
6. invokes `discover_and_analyze_with_image`.

That function constructs:

```rust
ModuleResolver::with_external_modules(
    &universe,
    &source_provider,
    image.modules().clone(),
)
```

then:

```rust
ModuleLinker::new(...)
    .with_external_modules(image.modules().clone())
```

and:

```rust
SemanticWorkspaceSession::from_image(..., image)
```

Universe modules are excluded from local runtime-module lowering.

## Finding S3-20 — positive architecture

The high-level entry path required for the user's P7 end state is already present.

The missing work is not a new package loader. It is making the image complete enough that this path can actually run against `.telosc`.

---

# 26. Standalone single-module behavior is also sensible

`analyze_standalone_module_with_image` permits dependencies only when they are absolute Universe imports.

Any non-Universe dependency produces:

```text
StandaloneImportRequiresPackageContext
```

This cleanly separates:

- a single source file using the standard Universe;
- a standalone package that can own local dependencies.

This is a useful final acceptance discriminator.

---

# 27. Current canonical artifact test is aspirational, not passing evidence

`phalcom-core/tests/core/modules/canonical_artifact.rs` contains an ambitious test:

```rust
fn build_and_load_universe_telosc_compiles_source_free_inline_user_entry()
```

It asserts:

1. canonical image is semantically valid;
2. `build_universe_telosc` succeeds;
3. build is deterministic;
4. atomic write succeeds;
5. disk load succeeds;
6. memory load succeeds;
7. source-free snapshot is complete;
8. major semantic owners are restored;
9. `loaded.compile_user_entry(Inline("print(42)"))` succeeds.

This test is well-shaped as a future acceptance scaffold.

But at audited HEAD it cannot pass because `build_universe_telosc` reaches the deliberate callable-analysis/static-owner export blockers.

## Finding S3-16 — HIGH

Do not cite the existence of this test as “P7 end-to-end coverage.”

The currently exercised evidence stops before the artifact is built.

A test body is not coverage until the branch can run through its downstream assertions.

---

# 28. Current test does not meet the requested standalone-package closure

Even when the current test becomes green, its application is:

```text
print(42)
```

through `EntrySelection::Inline`.

It proves very little about:

- standalone package discovery;
- package modules;
- package-relative imports;
- generic call environment;
- trait/conformance evidence;
- static definitions/constants;
- collection lowering;
- package-level module bindings;
- `parse`;
- `similarity`;
- `slotmap`.

## Finding S3-17 — HIGH

P7 needs a separate **feature-dense package acceptance**.

Recommended hierarchy:

### Gate A — inline smoke

```text
print(42)
```

Useful for minimum image load.

### Gate B — standalone single module

Imports canonical Universe only.

### Gate C — tiny standalone package

At least:

```text
package.ph
main.ph
one sibling package/module
```

### Gate D — real package feature matrix

Compile and run representative packages/tests from:

- `packages/parse`
- `packages/similarity`
- `packages/slotmap`

using the loaded `.telosc` image.

At least one fixture should combine those semantics rather than only compiling each package in isolation.

---

# 29. P7 compiler-only capability vs. “run a package”

The artifact summary deliberately asserts:

```rust
!built.summary.capabilities.portable_executable
```

and the writer rejects the portable-executable bit.

That is architecturally consistent with the P7/P8 boundary:

```text
P7 → canonical compiler image
P8 → portable/runtime executable Universe authority
```

## Finding S3-18 — MEDIUM

The phrase “P7 can run a standalone package” must be interpreted carefully.

P7 can and should prove:

```text
user/package compilation consumes Universe compiler authority
from .telosc without Universe source
```

The **runtime** may still obtain its executable Universe through the existing pre-P8 mechanism.

Therefore acceptance telemetry should distinguish:

```text
Universe parser invocations during compiler image load/compile = 0
Universe semantic analysis during compiler image load/compile = 0
Universe metadata export during compiler image load/compile = 0

legacy VM/runtime Universe setup
    → separately counted and explicitly outside P7 artifact capability
```

Otherwise a “package ran successfully” test can accidentally hide continued source-backed compiler work.

---

# 30. Source-denied acceptance must make source access impossible

A strong P7 test should not merely assert counters after a normal run.

Run the consumer process with Universe source physically unavailable to the compiler path.

Examples:

```text
TELOSC artifact copied to temp installation
Universe source directory moved/renamed
source provider replaced with deny/panic implementation
fresh process launched
```

Then compile:

```text
EntrySelection::Package(...)
```

If compiler code accidentally falls back to:

- `UniverseSourceProvider`;
- `snapshot.sources`;
- canonical source `OnceLock`;
- source AST reconstruction;

the test must fail immediately.

---

# 31. The local workspace parent tests are valuable and should be retained

`ordinary_image_session_keeps_universe_external_across_replacement` is one of the strongest current architectural tests.

It verifies:

- local snapshots contain only local source;
- Universe remains external;
- local module session contains no Universe source or linked modules;
- canonical derived rules remain parent-owned;
- canonical conformance index remains parent-owned;
- canonical aliases/static definitions/callable definitions/module bindings survive;
- canonical static constants survive;
- pointer identity is retained;
- local mutation cannot overwrite canonical parent entries;
- type/static-value stores extend canonical prefixes.

## Recommendation

After `.telosc` loading works, duplicate this test using:

```text
load_universe_telosc_from_bytes(...)
→ loaded.formal_image
→ SemanticWorkspaceSession::from_image(...)
```

instead of the source-built in-memory canonical image.

That will prove the persisted image satisfies the same parent contract.

---

# 32. Static table parent inheritance is already correctly wired

Slice 3 specifically checked whether imported static products, once reconstructed, would be lost on first local publication.

They would not.

First publication initializes:

```rust
StaticDefinitionTable::with_parent(
    canonical_snapshot.static_definitions.clone()
)

StaticConstantTable::with_parent(
    canonical_snapshot.static_constants.clone()
)

ParentMap::with_parent(
    canonical_snapshot.module_static_bindings.clone()
)
```

This is correct.

## Consequence

The main static problem is **artifact reconstruction**, not session overlay mechanics.

Implementer time should be spent on:

- wire owner coverage;
- imported static tables;
- allocation accounting;
- query owner adapters;

not redesigning `SemanticWorkspaceSession`.

---

# 33. Canonical conformance inheritance is also mostly correctly wired

On first local update:

```text
conformance index          → canonical parent
derived conformance rules → canonical parent
witness plans             → cloned Arc map of immutable plans
```

The session explicitly skips rebuilding Universe conformance plans when an image is attached.

This is desirable.

## Consequence

The imported image must provide complete canonical conformance authority. There is intentionally no later source-backed repair pass.

---

# 34. Canonical callable analyses are not copied into local snapshots

On first update from an image:

```rust
let mut callable_analyses =
    previous_snapshot.map_or_else(HashMap::new, ...)
```

Since `previous_snapshot` is initially `None`, the canonical image's callable analyses are not copied into the new user workspace snapshot.

## Assessment

This is **not automatically a bug**.

It is consistent with the architecture if:

- canonical body analyses remain owned by the immutable image;
- user compilation does not lower canonical Universe modules;
- canonical formal/query products provide whatever immutable dependency evidence local checking needs.

This avoids deep-copying all Universe body analysis into every workspace.

## Required proof

Add an invariant test:

```text
image callable analyses: N > 0
first local snapshot local callable analyses:
    only user/package modules
Universe analyses are absent from local table
yet all user compiler/semantic queries succeed
```

Do not “fix” this by copying canonical body analyses.

---

# 35. Formal/query authority must compensate for non-copied canonical bodies

Because canonical callable bodies stay outside the local snapshot, any local query that depends on a body-derived canonical fact must obtain it through:

- canonical query image;
- canonical aggregate dependency fingerprint;
- canonical signatures/evidence;
- canonical parent product tables.

This makes S3-07 (lost dependency evidence) especially important.

A missing canonical body copy is fine.

A missing body copy **plus missing aggregate dependency evidence** is not.

---

# 36. Completeness predicate remains too weak for closure certification

`has_source_free_artifact_contents` checks broad conditions such as:

```text
nonempty callable analyses
all callable statuses complete
some body facts exist
nonempty signatures
nonempty definitions
nonempty static tables
nonempty module products
...
```

It does not prove:

- every signature that requires a definition has one;
- no fabricated definition exists;
- definition origin parity;
- every body-required formal binding is attached;
- static definition operational row parity;
- every static constant/module binding owner is covered;
- derived rule count/owner parity;
- trait-dispatch reconstruction equivalence;
- module bounded indexes agree with primary owners;
- query dependency evidence coverage.

## Finding S3-15 — HIGH

Treat `has_source_free_artifact_contents` as a **cheap final sanity predicate**, not as artifact certification.

Add explicit family-level parity validators before setting:

```text
SnapshotStatus::Complete
```

---

# 37. Required owner parity checks

Before promoting an imported snapshot:

## Callable definitions

```text
stable definition owner set
== expected exported definition owner set
```

and:

```text
definition.signature.callable == definition.callable
origin owner closure valid
source member ordinal in accepted impl/default descriptor
body context valid
```

## Static definitions

```text
exported static definition IDs
== imported table IDs
```

Every operational body owner must resolve to exactly one definition.

## Static constants

```text
constant IDs / values / materialization descriptors
preserved
```

## Module bindings

```text
one canonical module row per exported module
stable symbol uniqueness
binding owner relocation valid
```

## Derived rules / trait dispatch

```text
rule owners preserved
reconstructed dispatch bucket fingerprints match source
```

## P6 indexes

For every primary table:

```text
index[module]
== sorted primary owners belonging to module
```

---

# 38. Query adapter coverage should be mechanically checked

The current `PortableQueryKindV1` support drift is exactly the sort of defect a compile-time/static test can catch.

Add a table-driven test:

```text
for every PortableQueryKindV1:
    profile_support
    export key adapter
    decode key adapter
    loaded product adapter
    fingerprint adapter
    dependency-evidence adapter
```

Allowed states:

```text
SupportedReusableRoot
    → all adapters MUST exist

SupportedEvidenceOnly
    → key + evidence-fingerprint adapters MUST exist

OmittedSourceDependent
    → loader MUST reject as evidence/root

Unsupported
    → exporter MUST never emit
```

This would have caught `ModuleStaticBindings`.

---

# 39. Query dependency evidence requires negative tests

Add tests that create:

```text
Reusable root A
    depends on canonical evidence-only owner B
```

Then prove after load:

```rust
loaded.formal_image
    .dependency_fingerprint(&B)
    == source_fingerprint_B
```

and:

```rust
SemanticDb::with_canonical_image(...)
    .record_dependency(..., B)
```

succeeds without constructing B as a reusable local query product.

Also test:

- duplicate evidence key;
- wrong aggregate owner;
- altered fingerprint;
- source-dependent evidence kind;
- evidence/root fingerprint disagreement.

---

# 40. Query allocation tests currently cover only preflight closure

`artifact_allocation.rs` has useful allocator-instrumented coverage for:

- query closure scratch;
- flow graph reconstruction;
- exact-fit/N-1 behavior;
- allocation-free error reporting for selected boundaries.

It does not measure:

```text
reconstruct_query_image_with_allocation_budget
```

end to end.

## Required test

Use the counting allocator around the full reconstruction:

```text
valid populated query image
complete source-free snapshot fixture
measure global allocator requested bytes
compare with AllocationBudget requested bytes
```

Required result:

```text
all load-owned allocations are accounted
```

or explicitly document any platform allocator boundary excluded by policy.

---

# 41. Static operational template allocation path also needs audit

`import_checked_static_definitions` currently returns ordinary:

```text
Vec<OperationalTemplate>
```

with recursively owned operational nodes.

Once those templates are attached to canonical static definition owners, the allocation cost becomes retained canonical image memory.

Before removing the semantic import allocation guard, measure:

- template vectors;
- nested operation boxes;
- strings/member names;
- constructor selectors;
- argument arrays;
- final Arc/static owner wrappers.

A recursive decoder can be bounded by depth and entry limits yet still violate the exact transaction ledger if retained allocations are not charged.

---

# 42. Writer/source parity requirement for static operational forms

Source-built static definitions may contain richer `CheckedStaticBody` data than the operational tree.

The artifact should define exactly what the loaded compiler/evaluator needs.

Required source/import comparator should verify:

```text
definition signature
result domain
operational behavior
parameter/local layout
static application targets
construction targets
dependency reads
materialized outcome
```

rather than comparing source-only AST/checker caches.

This is one place where artifact parity should be **semantic parity**, not byte-for-byte owner struct equality.

---

# 43. Avoid persisting broad source structures to solve lowering gaps

Several current gaps could be superficially “fixed” by serializing:

```text
snapshot.sources
source_index
semantic_structure_shards
```

That would violate the P7 design and erase much of the intended performance benefit.

The correct move is to identify the tiny semantic authority currently extracted from source and attach it to its proper owner.

Examples:

| Current source lookup | Correct artifact owner |
|---|---|
| inherent delegation kind/target field | accepted callable/impl definition |
| trait default body/member ordinal | trait member/default definition |
| formal parameter source attachment | callable analysis/formal projection |
| top-level binding phase/identity | module static binding index |
| static body executable plan | static definition checked operational owner |
| runtime static constant materialization | static constant product/value |

---

# 44. P6 local-complexity contract after load

The image path should retain the current local-complexity behavior:

```text
compiler lowering of user module M
    should inspect:
        user facts for M
        directly referenced canonical parents/indexes
    not:
        every Universe callable
        every Universe declaration
        every Universe static constant
```

The current snapshot APIs support this direction:

```text
callable_analyses_in_module
callable_definitions_in_module
module_constants
module static binding lookup
dispatch/type indexes
```

But the imported image must rebuild actual typed indexes, not detached debug maps.

---

# 45. Performance concern: loaded module indexes should be frozen, not rebuilt per session

A complete `.telosc` load is allowed one-time index reconstruction.

Each subsequent `SemanticWorkspaceSession::from_image` should only attach handles.

Required measurement:

```text
load artifact:
    O(canonical owner count) once

create new workspace session:
    O(1) / O(number of parent handles)

compile package:
    O(local package + referenced canonical lookups)
```

The existing observability expectation:

```text
CanonicalAggregateEntriesCopied = 0
CanonicalParentHandlesAttached = 10
```

should remain true after the source-built image is replaced by a persisted one.

---

# 46. The final P7 test must use a fresh process

The current canonical image is a process-global `OnceLock` when built from source.

An in-process test can accidentally pass because source-built canonical objects already exist in memory.

Required test topology:

```text
process A:
    build universe.telosc
    exit

process B:
    Universe source inaccessible
    load universe.telosc
    compile standalone package
    run compiled package
```

Process B must never have initialized:

```text
canonical_universe_formal_image()
canonical_universe_linked_program()
UniverseSourceProvider canonical source cache
```

This is mandatory evidence for P7.

---

# 47. Recommended feature-dense package acceptance

The user's requested end state explicitly names `parse`, `similarity`, and `slotmap`.

A useful final package fixture should exercise:

```text
package imports
generic data
enum inference
trait/conformance calls
iteration
collections
static constants / static definitions
match/pattern lowering
module-level bindings
call specialization
```

and then consume:

```text
parse
similarity
slotmap
```

through ordinary package imports.

The exact package source can be small; the point is to force the compiler through the relevant canonical semantic families.

---

# 48. Distinguish compile success from run success

Final evidence should record two separate outcomes:

```text
A. loaded-image compilation succeeds
B. produced program executes successfully
```

Do not conflate them.

P7 failure categories should identify whether the issue is:

```text
artifact load
semantic workspace
linking
compiler lowering
metadata
program execution
legacy runtime Universe setup
```

This will also make the P7/P8 boundary visible.

---

# 49. Revised implementer execution order after Slice 3

The combined Slice 1–3 dependency order should now be:

```text
A. Fix source semantic correctness defects
   1. callable body fingerprint completeness
   2. any owner fingerprint drift

B. Complete callable-analysis owner graph
   1. root census
   2. frozen/budgeted owner storage
   3. DTO + relocation + formal attachments
   4. checked StaticApplication cross-owner relation

C. Persist accepted callable definitions
   1. exact origin
   2. impl/conformance owner
   3. source member ordinal
   4. inherent body context
   5. delegation/default execution attributes

D. Complete static owner authority
   1. static case catalogs
   2. static case signatures
   3. static definitions
   4. attach operational templates
   5. static constants
   6. module static binding indexes

E. Complete conformance authority
   1. witness plan fields
   2. derived conformance rules
   3. deterministic trait-dispatch reconstruction/index

F. Reconstruct compiler reader indexes
   1. formal projection subset
   2. module-local primary-owner indexes
   3. no broad source index

G. Repair QUERY_IMAGE
   1. module-static query adapters
   2. reconstruct dependency_evidence
   3. unify owner fingerprint adapters
   4. budgeted frozen query storage

H. Close semantic-load allocation ledger
   1. all retained semantic owners
   2. all query owners
   3. error-path allocations
   4. exact-fit/N-1 rollback

I. Remove fail-closed writer/import guards

J. Prove source/import parity

K. Build and write universe.telosc

L. Fresh-process source-denied tests
   1. inline
   2. standalone module
   3. standalone package
   4. parse/similarity/slotmap feature matrix

M. Re-run P5/P6 performance gates
```

---

# 50. Work that should NOT be interleaved prematurely

Do not spend significant time tuning:

- container compression;
- artifact mmap;
- metadata micro-optimizations;
- cache freshness;
- final CLI artifact discovery;

until the canonical owner graph can be built and loaded correctly.

Those are valuable later, but the critical path remains semantic authority.

Likewise, do not remove export guards one family at a time if the resulting artifact can pass serialization but cannot publish a complete source-free snapshot.

---

# 51. Proposed concrete implementation work packages

## Work Package 1 — CallableDefinition transport

Files:

```text
phalcom-compiler-artifact/src/semantic/world.rs
phalcom-semantic/src/artifact/export.rs
phalcom-semantic/src/artifact/import.rs
phalcom-semantic/src/impls.rs
```

Acceptance:

```text
source/import exact origin parity
inherent body context parity
conformance witness parity
module definition index parity
query CallableDefinition fingerprint parity
```

## Work Package 2 — Static owner family

Files:

```text
static_definition.rs
static_constant.rs
module_bindings.rs
artifact/operational.rs
artifact/export.rs
artifact/import.rs
semantic/world.rs
```

Acceptance:

```text
static definitions nonempty after load
operational templates reachable through definition owners
static constants materialize
module bindings seed a user checker
no Universe source requested
```

## Work Package 3 — Conformance completion

Acceptance:

```text
witness plan exact parity
derived rule parity
trait dispatch lookup parity
generic user call compiles from imported image
```

## Work Package 4 — Query evidence repair

Acceptance:

```text
every SupportedReusableRoot kind has adapter
dependency_evidence survives load
SemanticDb records aggregate canonical dependencies
malformed/stale evidence rejected
```

## Work Package 5 — Source-independent backend reader

Acceptance:

```text
no canonical snapshot.sources read
for delegation/default/formal binding authority
```

---

# 52. Required negative tests

A correct P7 loader must reject artifacts that:

- contain a callable definition whose origin owner is absent;
- classify a conformance witness as a primary declaration;
- use an out-of-range impl member ordinal;
- attach an operational template to an absent/wrong static definition;
- contain duplicate static definition owners;
- contain a module binding whose body-local binding ordinal is absent;
- contain a static constant referring to absent static values;
- contain a derived conformance rule with absent trait/target/type roots;
- advertise a module-static query root with no corresponding semantic owner;
- carry dependency evidence whose owner fingerprint disagrees with the snapshot;
- carry evidence for a source-dependent query kind;
- omit a formal attachment for an explicit source parameter;
- attempt to rely on `source_index` or source AST during imported canonical lowering.

---

# 53. Required positive tests

Before P7 closure:

```text
CallableDefinition:
    primary
    inherent
    covering inherent
    conformance witness

Static:
    simple constant
    operational definition
    nested static application
    static construction
    imported module static constant

Conformance:
    exact
    forwarded generic
    derived
    compiler-derived

Compiler:
    local binding read/write
    trait default
    delegation
    match/pattern
    for iteration
    sequence
    generic call

Package:
    local sibling import
    Universe import
    parse
    similarity
    slotmap
```

---

# 54. Source-free package acceptance telemetry

Record at minimum:

```text
Universe source files opened             = 0
Universe parse invocations               = 0
Universe interfaces rebuilt              = 0
Universe linker modules rebuilt          = 0
Universe semantic callable analyses      = 0
Universe static definitions rechecked    = 0
Universe constants re-evaluated          = 0
Universe metadata export work            = 0

artifact load bytes
artifact retained allocation bytes
artifact load wall time

user/package source files parsed
user semantic work
user modules lowered
```

This is the evidence that P7 produced a real compiler cache rather than another bootstrap route.

---

# 55. Current branch testing assessment

## Strong existing tests

The branch already has useful coverage for:

- source-built canonical image parity;
- canonical parent pointer reuse;
- local replacement without Universe source incorporation;
- artifact container corruption;
- capability/header validation;
- selected allocation boundaries;
- flow graph frozen reconstruction;
- callable-analysis helper records;
- native catalog fingerprint rejection.

## Missing decisive tests

Still absent as passing evidence:

```text
real semantic-world export
real semantic-world import
complete loaded snapshot
query evidence import
source-built vs artifact-built canonical parent parity
standalone package compile from artifact
standalone package run from artifact-backed compiler
parse package
similarity package
slotmap package
fresh process with Universe source unavailable
```

No CI status/workflow run is attached to the audited HEAD.

---

# 56. Key distinction for implementer guidance

There are now three different concepts that should not be conflated:

## 1. Source-built canonical image

Already relatively mature.

It contains all semantic authority and is used by current image-session integration tests.

## 2. Persisted P7 canonical compiler image

Incomplete.

It must reconstruct enough of concept (1) to be semantically equivalent for compiler consumers, while omitting source/presentation-only data.

## 3. P8 executable Universe image

Not P7.

It removes the remaining VM/runtime Universe compilation path.

P7 should target exact compiler equivalence between (1) and (2), not prematurely expand into (3).

---

# 57. Closure criteria after Slice 3

P7 should not be declared complete until all of these are true:

- [ ] real Universe `CallableAnalysis` owner graph exports/imports;
- [ ] accepted callable definitions survive exactly;
- [ ] static case/definition/constant/module-binding authority survives;
- [ ] operational templates are attached to actual static owners;
- [ ] conformance witness/derived/dispatch authority survives;
- [ ] compiler-required formal projection is reconstructed;
- [ ] no compiler-required broad source index is needed;
- [ ] query reusable-root adapter set is closed;
- [ ] dependency evidence survives import;
- [ ] loaded query image is allocation-accounted;
- [ ] semantic import is allocation-accounted;
- [ ] imported snapshot passes field-level parity, not merely nonempty checks;
- [ ] source-built and artifact-built workspace overlays behave equivalently;
- [ ] fresh process loads `.telosc` without Universe source;
- [ ] standalone package compiles;
- [ ] representative package executes;
- [ ] `parse`, `similarity`, and `slotmap` acceptance runs succeed;
- [ ] canonical compiler work counters remain zero on loaded path;
- [ ] P5 zero-copy/parent-sharing gates remain intact;
- [ ] P6 module-local complexity gates remain intact;
- [ ] deterministic artifact build succeeds repeatedly.

---

# 58. Recommended Slice 4

The next audit slice should focus on **allocation, retained-memory, and validation closure across the entire load transaction**, now that the semantic owner/source-denial graph is mapped.

Recommended Slice 4 scope:

```text
container acquisition/decode
MODULE_WORLD
SEMANTIC_WORLD
TypeStore relocation/interning
StaticValueStore
frozen semantic owners
QUERY_IMAGE reconstruction
METADATA decode
final Arc publication
failure rollback
error-path allocations
```

It should answer:

> Which exact allocations remain outside `AllocationBudget`, which data structures need frozen representations, where peak memory is duplicated, and what exact-fit/N-1 tests are still missing before `UnaccountedBoundary` can be removed?

That will turn the current broad allocation blocker into an actionable closure map.

---

# 59. Final Slice 3 verdict

**Status: P7 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`.**

Slice 3 strengthens rather than weakens the closure case:

- the **consumer architecture is promising**;
- the **artifact authority is still materially incomplete**.

The good news is that no major redesign of `SemanticWorkspaceSession::from_image` appears necessary. The branch already has the correct immutable-parent model for a compiled Universe.

The remaining work is to make `.telosc` reconstruct that parent truthfully:

```text
complete callable owner graph
+ accepted definition provenance
+ static owner authority
+ conformance authority
+ formal compiler attachments
+ query evidence
+ budgeted immutable indexes
```

Once those are present, the existing `EntrySelection::Package` path is a credible route to the user's intended P7 end state.

Until then, a green serializer/codec test would not constitute P7 closure.
