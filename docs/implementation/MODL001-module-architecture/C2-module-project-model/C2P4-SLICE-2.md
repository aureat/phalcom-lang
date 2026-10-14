# Slice 2 Report — Canonical Semantic Evidence Ownership

## Scope

This slice traced the current semantic path from an authored import through:

```text
ImportSiteId
    ↓
ImportResolutionProduct
    ↓
source-index import binding
    ↓
canonical target identity
    ↓
LinkedTypeResolver / checker value seeding
    ↓
callable/query semantic products
    ↓
incremental fingerprints and snapshot publication
```

The incident requires a canonical per-import evidence owner, stable key, conservative join/default, and incremental parity. Pasted text

### Repository state verified

`feat/telos-collections` currently points to `da436a6c…`. That is a documentation-only commit recording the P4 architecture incident/draft; its parent is the reported code revision `e5988ad…`. The source architecture analyzed below is therefore still the architecture described by the incident.

---

# 1. Slice ruling

The semantic evidence owner **must not be** any of these existing identities/products by themselves:

| Candidate | Ruling | Why |
|---|---|---|
| `ImportSiteId` | Insufficient | Identifies an import **statement**, not an individual selected binding. |
| `SourceSiteId` | Insufficient | Explicitly snapshot-local; unsuitable as the durable cross-product/incremental identity. |
| `ImportBindingId` | Wrong layer | It is an executable linked-read slot identity—the exact coupling P4 must eliminate. |
| `SymbolId` / `DeclarationId` | Insufficient | Identifies the remote declaration, not the distinct local import binding. Two aliases/import sites may target the same symbol. |
| local name string | Unsound | Shadowing, aliases, reordering, and distinct imports make spelling non-canonical. |
| `SemanticTargetId` | Insufficient | Canonical target identity, but not local import identity. |
| `ModuleStaticBindingIndex` | Wrong semantics | Models module **value** bindings and `StaticConstant`/`Runtime`, not dependency phase. |
| source occurrences | Presentation evidence only | Roles do not distinguish interface use from runtime-value requirements. |

The correct architecture needs a new stable identity in **`phalcom-modules`**, conceptually:

```rust
pub struct ImportBindingLocalId(pub u32);

pub struct SourceImportBindingId {
    pub site: ImportSiteId,
    pub local: ImportBindingLocalId,
}
```

or equivalent.

For:

```telos
from foo import A, B as Bee, C
```

there is one:

```text
ImportSiteId
```

but three:

```text
SourceImportBindingId(site, #0) → A
SourceImportBindingId(site, #1) → B as Bee
SourceImportBindingId(site, #2) → C
```

This is the missing stable key P4 requires.

---

# 2. Why `ImportSiteId` cannot be the phase key

`phalcom-modules/src/identity.rs` defines:

```rust
pub struct ImportSiteId {
    pub importer: ModuleId,
    pub local: ImportSiteLocalId,
}
```

and describes it as the identity of an authored **import site**.

The interface layer generates one site per `ImportSurface`. A selective import with several items remains one site.

The semantic source-index builder confirms the mismatch directly.

In `phalcom-semantic/src/source_index/builder.rs::visit_imports`:

```text
selective import statement
    │
    ├── one ImportSiteId
    │
    ├── item A   → one SourceSiteId
    ├── item B   → one SourceSiteId
    └── item C   → one SourceSiteId
```

Each selected item can have a different phase:

```telos
from foo import TypeA, makeB

class X {
  value: TypeA       // InterfaceOnly
}

run {
  makeB()            // Runtime
}
```

A phase attached to the enclosing `ImportSiteId` cannot express this.

The phase unit must therefore be **the imported local binding**, not the import declaration.

---

# 3. `SourceSiteId` already solves half the problem—but cannot be the persistent key

The source index already carries useful information.

`ImportBindingOrigin` is:

```rust
pub struct ImportBindingOrigin {
    pub local_binding: SourceSiteId,
    pub remote_target: SemanticTargetId,
}
```

For selective imports, `visit_imports` creates a separate local source binding for every item and records:

```text
local SourceSiteId
        ↓
ImportBindingOrigin
        ↓
canonical remote SemanticTargetId
```

This is exactly the association P4 needs semantically.

But `SourceSiteId` is explicitly documented as **snapshot-local**:

```text
SourceSiteId
    owner: SourceOwner
    local: SourceSiteLocalId
```

and its documentation says it must not be reused after replacing a source snapshot without carrying the `SnapshotId`.

Therefore the source index should **attach** the durable import identity rather than become its owner.

Recommended shape:

```rust
pub struct ImportBindingOrigin {
    pub local_binding: SourceSiteId,

    // New stable compiler/module identity.
    pub import_binding: SourceImportBindingId,

    pub remote_target: SemanticTargetId,
}
```

That creates the necessary bridge:

```text
SourceSiteId                    snapshot/presentation identity
     │
     ▼
SourceImportBindingId           stable import-binding identity
     │
     ▼
SymbolId / DeclarationId        canonical remote identity
```

Those three identities solve different problems and should remain distinct.

---

# 4. The provisional linker should own the authoritative binding table

Slice 1 concluded that P4 requires a semantic/provisional linked product separate from the executable link.

Slice 2 makes the shape of that product considerably clearer.

The provisional module link should own something conceptually like:

```rust
pub struct SymbolicImportBinding {
    pub id: SourceImportBindingId,

    /// Name visible in the importing module.
    pub local_name: Box<str>,

    /// Canonical remote target.
    pub target: SymbolicImportTarget,

    pub source_range: SourceRange,
}

pub enum SymbolicImportTarget {
    Module(ModuleId),
    Binding(SymbolId),
}
```

and:

```rust
pub struct SemanticLinkedModule {
    pub imports:
        BTreeMap<SourceImportBindingId, SymbolicImportBinding>,

    pub imports_by_local_name:
        BTreeMap<Box<str>, SourceImportBindingId>,

    // interface/export information...
}
```

This table should become the canonical semantic import namespace.

That change fixes a second major architectural defect discovered in this slice.

---

# 5. `LinkedTypeResolver` currently resolves semantic names through runtime slots

`phalcom-semantic/src/resolver.rs::LinkedTypeResolver` currently owns:

```rust
linked: Arc<LinkedProgram>
```

and resolves a selective imported type by following:

```text
local name
    ↓
LinkedModule.bindings.imports
    ↓
ImportBindingId
    ↓
LinkedModule.linked_reads[slot]
    ↓
LinkedReadSpec::Binding(SymbolId)
    ↓
DeclarationId
```

Likewise a module alias is resolved through:

```text
ImportBindingId → LinkedReadSpec::Module
```

This is precisely the coupling that prevents a genuine interface-only import:

> **Semantic type resolution currently requires the runtime-read layout to exist.**

The incident identified the broader coupling between semantic imports, runtime edges, read slots and materialization. Pasted text

The staged design should change this to:

```text
local name
    ↓
SemanticLinkedModule.imports_by_local_name
    ↓
SourceImportBindingId
    ↓
SymbolicImportBinding.target
    ↓
canonical DeclarationId / ModuleId
```

No `ImportBindingId` should be necessary for semantic resolution.

`ImportBindingId` should exist only after executable finalization.

---

# 6. This gives P4 a clean identity model

After this change there are four deliberately distinct identities:

| Identity | Owner | Meaning |
|---|---|---|
| `ImportSiteId` | modules/source resolution | One authored import declaration |
| `SourceImportBindingId` | symbolic linker | One local name introduced by that import |
| `SymbolId` / `DeclarationId` | canonical semantic model | Remote canonical declaration |
| `ImportBindingId` | executable linker | Runtime-read slot after finalization |

That separation is the right architecture.

In particular:

```text
SourceImportBindingId != SymbolId
```

because these should remain distinguishable:

```telos
from foo import Thing as A
from foo import Thing as B
```

Both bindings target the same:

```text
foo::Thing
```

but they are separate source imports and can independently contribute evidence, diagnostics and incremental invalidation.

---

# 7. Phase cannot be inferred inside `TrackingTypeResolver`

This is the most important semantic finding of Slice 2.

`TrackingTypeResolver::resolve_type_name` currently records query dependencies such as:

```text
LinkedName
PublicExport
DeclarationShell
TraitHeader
```

It therefore looks like an attractive place to add:

```rust
record_import_use(import, DependencyPhase::InterfaceOnly)
```

That would be **wrong**.

The same underlying type resolver is used from semantic contexts with different runtime meanings.

For example, `phalcom-semantic/src/checker/expression.rs` contains the variable-expression path:

```rust
Expr::Var { value, range } => {
    if let Some(fact) = ctx.lookup_local(value) {
        // ordinary runtime local/imported value
        ...
    } else if let Some(decl) = ctx.resolve_type_name(value) {
        if let Some(info) = ctx.declaration_info(&decl) {
            TypedExpression::established(
                info.class_object_type,
                ...
            )
        }
        ...
    }
}
```

The second branch has performed a **type-name resolution**, but its result represents a runtime class/type-form value.

So:

```text
resolve_type_name(...)
```

does **not** imply:

```text
InterfaceOnly
```

The semantic meaning comes from the *consumer*.

---

# 8. Evidence ownership must be split between resolution and use

The correct contract is:

```text
resolver
    owns:
        what did this name resolve to?
        which import binding supplied it?

semantic use site
    owns:
        why is that resolved thing being consumed?
        does execution require its runtime representation?
```

Conceptually the resolver needs to expose provenance:

```rust
pub struct ResolvedImportedName {
    pub target: DeclarationId,
    pub import_binding: Option<SourceImportBindingId>,
}
```

The exact API does not have to replace every `TypeResolver` method with this structure. A side lookup can work equally well:

```rust
resolver.import_origin(
    current_module,
    root,
) -> Option<SourceImportBindingId>
```

provided it uses the same symbolic import authority and never re-resolves text independently.

What must **not** happen is:

```text
resolve declaration first
        +
later search source/imports trying to guess which import supplied it
```

That would introduce a second resolver.

---

# 9. Use classification needs an explicit semantic context

The semantic recorder should receive an explicit requirement such as:

```rust
pub enum ImportUseRequirement {
    InterfaceOnly,
    Runtime,
}
```

but calls to it should occur from use-site-aware semantic APIs.

A practical implementation could expose:

```rust
fn record_import_use(
    &self,
    binding: SourceImportBindingId,
    requirement: ImportUseRequirement,
    reason: ImportUseReason,
);
```

with reasons retained for diagnostics/testing, for example:

```rust
pub enum ImportUseReason {
    TypeAnnotation,
    GenericConstraint,
    TraitConstraint,
    SuperclassSemanticReference,

    ValueRead,
    CallTarget,
    ClassObjectValue,
    RuntimeTypeForm,
    RuntimeDeclarationMaterialization,

    Unsupported,
}
```

The **reason** need not participate in the lattice. It is valuable proof/debug information.

The authoritative reduction remains:

```text
InterfaceOnly < Runtime
```

---

# 10. Do not derive runtime use by scanning `ExpressionAnalysis::binding_read`

There is already a good runtime-use signal:

```rust
ExpressionAnalysis {
    binding_read: Option<BindingId>,
    binding_write: Option<BindingId>,
    ...
}
```

A normal variable read sets `binding_read`.

But it is insufficient as the sole authority.

The unqualified-call path in `checker/expression.rs` begins with:

```rust
if let Some(fact) = ctx.lookup_local(&call.name) {
    ...
    apply_resolved_callable(...)
}
```

That is clearly a runtime consumption of the imported value, but this path does not have the same simple `Expr::Var` → `typed.binding_read` structure.

Therefore a post-analysis algorithm such as:

```rust
for expr in analysis.expressions {
    if let Some(binding) = expr.binding_read {
        mark_runtime(binding);
    }
}
```

would be unsafe.

### Better boundary

When imported values are seeded into the checker, preserve their provenance:

```text
body-local BindingId
        ↓
SourceImportBindingId
```

For example:

```rust
import_binding_origins:
    BTreeMap<BindingId, SourceImportBindingId>
```

Then an operation that **semantically consumes** a lexical value can record runtime use at that point.

A raw lookup used only for explanation/introspection should not count as a runtime use.

That likely argues for APIs along the lines of:

```rust
lookup_local(...)
consume_local_value(...)
consume_local_callable(...)
```

or an equivalent explicit-use hook.

---

# 11. Current value seeding loses exactly the provenance P4 needs

`CheckingContext::seed_linked_module_import_values` currently does:

```rust
for (local_name, import_id) in &linked_module.bindings.imports {
    let LinkedReadSpec::Binding(symbol) =
        linked_module.linked_reads[import_id];

    ...
    self.declare_checked_binding(
        local_name,
        &fact.state,
        fact.phase,
    );
}
```

There are two problems.

First, it gets semantic imported values from the **runtime linked-read layout**.

Second, `declare_checked_binding` allocates a fresh body-local `BindingId`, while `seed_linked_module_import_values` discards the returned identity and therefore does not retain:

```text
BindingId → source import binding
```

P4 should change both.

The new path should be approximately:

```text
SemanticLinkedModule.symbolic_imports
          +
remote ModuleStaticBindingIndex
          ↓
seed imported checker value
          ↓
fresh body-local BindingId
          ↓
retain BindingId → SourceImportBindingId
```

This preserves ordinary checker architecture while removing its dependency on executable slots.

---

# 12. Do not reuse `ModuleBindingPhase`

`module_bindings.rs` already has:

```rust
pub enum ModuleBindingPhase {
    StaticConstant,
    Runtime,
}
```

That is a completely different axis.

It describes whether a **module-owned value declaration** is static or runtime.

P4's phase describes whether the **importing module needs the dependency at runtime**:

```rust
DependencyPhase::InterfaceOnly
DependencyPhase::Runtime
```

For example, a type declaration may be imported purely semantically even though its target module contains runtime declarations.

Likewise a static constant can still create a runtime dependency under some execution/materialization model.

These enums must remain separate.

---

# 13. Source-index occurrences are useful verification, not phase authority

The current source index already knows substantial identity information.

For ordinary occurrences, it has:

```rust
OccurrenceRole {
    Declaration,
    Read,
    Write,
    Call,
    Reference,
}
```

and exact targets.

For imported expression variables, `expression_target()` explicitly consults:

```rust
scopes.import_origin(&binding)
```

and can recover the canonical remote target.

That makes the source index useful for:

```text
cross-checking
LSP presentation
debug assertions
tests
diagnostics
```

It should **not** become P4's primary phase classifier.

Why?

Because `Reference` is not equivalent to `InterfaceOnly`, `Read` is not sufficient to enumerate all generated runtime needs, and generated declaration materialization may have no direct source occurrence at all.

The incident explicitly requires coverage of generated declaration references and every emitted runtime read. Pasted text

---

# 14. Incremental semantics changes the evidence architecture

A transient shared recorder is not enough.

The existing semantic system can reuse query products across revisions. If an unchanged query is reused, its semantic body does not necessarily execute again.

Therefore this architecture would be wrong:

```text
run checker
   ↓
transient global ImportUseRecorder
   ↓
aggregate current events
```

On incremental reuse, evidence could silently disappear.

P4 evidence must be **retained with incremental query state**.

The existing architecture already provides the necessary foundation:

```text
QueryKey::ResolvedImport(ImportSiteId)
QueryKey::LinkedName(ModuleId, String)
...
CallableAnalysis.semantic_dependencies
query fingerprints
SemanticDb dependency replay
immutable SemanticSnapshot
```

But there is currently no import-use-phase product.

---

# 15. Recommended incremental representation

Rather than polluting every semantic product with ad hoc fields, I recommend a small query-owned metadata product associated with every semantic query that can consume an imported binding:

```rust
pub struct QueryImportUseEvidence {
    pub module: ModuleId,
    pub uses: Arc<[ImportUseFact]>,
    pub complete: bool,
    pub fingerprint: ProductFingerprint,
}

pub struct ImportUseFact {
    pub binding: SourceImportBindingId,
    pub target: SemanticTargetId,
    pub requirement: ImportUseRequirement,
    pub reason: ImportUseReason,
}
```

Conceptually the semantic DB retains:

```text
QueryKey
   ├── ordinary semantic product
   ├── dependency edges
   └── import-use evidence
```

When the query is reused:

```text
semantic product reused
import-use evidence reused
```

When the query is invalidated:

```text
both refresh together
```

When only phase-relevant source usage changes while the externally observable semantic signature happens to remain identical, the **import-use evidence fingerprint still changes**.

That last property matters.

Example:

```telos
// v1
fn f(x: ImportedType) -> Unit { ... }

// v2
fn f(x: LocalAliasOfSameType) -> Unit { ... }
```

The callable's externally observed type could theoretically stay equivalent while dependency requirements change.

Import-phase evidence therefore needs its **own content fingerprint**, rather than piggybacking only on the ordinary semantic product fingerprint.

---

# 16. Module-level phase aggregation should be a separate semantic product

After query-local evidence is retained, semantic analysis can publish:

```rust
pub struct ModuleImportPhaseProduct {
    pub module: ModuleId,
    pub bindings:
        BTreeMap<SourceImportBindingId, ImportPhaseSummary>,

    pub coverage: PhaseEvidenceCoverage,
}
```

with approximately:

```rust
pub struct ImportPhaseSummary {
    pub target: SemanticTargetId,
    pub observed: bool,
    pub phase: DependencyPhase,
}
```

and explicit coverage:

```rust
pub enum PhaseEvidenceCoverage {
    Complete,
    Incomplete,
}
```

The executable linker consumes **this product**, not individual checker events.

---

# 17. Finalization rule should be deliberately conservative

Given the P4 draft's stated policy that absent/unknown evidence remains runtime, the decision can be expressed precisely:

```rust
fn executable_phase(
    summary: Option<&ImportPhaseSummary>,
    coverage: PhaseEvidenceCoverage,
) -> DependencyPhase {
    if coverage != PhaseEvidenceCoverage::Complete {
        return DependencyPhase::Runtime;
    }

    let Some(summary) = summary else {
        return DependencyPhase::Runtime;
    };

    if !summary.observed {
        return DependencyPhase::Runtime;
    }

    summary.phase
}
```

A binding becomes `InterfaceOnly` only when:

```text
coverage complete
AND
at least one supported semantic use observed
AND
every observed use is interface-only
AND
no generated runtime requirement exists
```

Anything else is runtime.

This solves the “unused import” ambiguity conservatively too:

```text
unused selected import → Runtime
```

unless a later architectural decision explicitly authorizes dead-import erasure.

P4 does not need that optimization.

---

# 18. `SnapshotStatus::Complete` should not automatically equal phase completeness

The snapshot architecture already has:

```rust
SnapshotStatus::Complete
SnapshotStatus::Partial { blocked_modules }
```

and individual callable analysis has:

```rust
CallableAnalysisStatus::Complete
Partial
Blocked
Cancelled
BudgetExceeded
InternalFailure
```

These are useful gates.

However I would **not automatically equate**:

```text
SnapshotStatus::Complete
```

with:

```text
all phase-relevant owners analyzed
```

The semantic session supports incremental/deep-module worklists. A snapshot may be semantically valid for its requested analysis surface without necessarily proving every executable body relevant to an executable image.

P4 therefore needs an explicit coverage contract.

For executable finalization:

```text
all phase-relevant semantic owners for runtime candidate modules
must either:

1. have current complete evidence, or
2. cause the affected imports to remain Runtime.
```

This avoids making phase safety depend on undocumented assumptions about semantic-session eagerness.

---

# 19. `ResolvedImport` remains useful, but at the wrong granularity

`QueryKey::ResolvedImport(ImportSiteId)` and `ImportResolutionProduct` are already solid incremental products.

They track:

```text
authored import statement
        ↓
canonical target module
        +
resolution topology/fingerprint
```

They should remain.

P4 should build on them:

```text
ImportSiteId
    ↓
ImportResolutionProduct
    ↓
target ModuleId

SourceImportBindingId
    ↓
SymbolicImportBinding
    ↓
target SymbolId / ModuleId
```

The new child identity does **not** replace `ImportSiteId`.

It adds the missing name-binding layer beneath it.

---

# 20. Revised evidence pipeline

The resulting architecture is now quite concrete:

```text
AST import declaration
        │
        ▼
ImportSiteId
        │
        ├──── ImportResolutionProduct
        │         target module
        │
        ▼
SourceImportBindingId(s)
        │
        ▼
SymbolicImportBinding
  local name + canonical target
        │
        ├────────────────────┐
        ▼                    ▼
Source index            Semantic resolver
SourceSiteId            canonical resolution
        │                    │
        └─────────┬──────────┘
                  ▼
       semantic use-site owner
                  │
        explicit use requirement
        InterfaceOnly / Runtime
                  │
                  ▼
       QueryImportUseEvidence
                  │
        incremental retention
                  ▼
       ModuleImportPhaseProduct
                  │
                  ▼
       executable-link finalizer
             /           \
            /             \
runtime dependency     runtime read
     graph                slots
            \             /
             strict DAG
```

This resolves the main evidence-ownership ambiguity raised by the incident.

---

# 21. Required P4 architectural amendments from Slice 2

| Area | Required change |
|---|---|
| Stable key | Introduce item-level `SourceImportBindingId` under `ImportSiteId`. |
| Symbolic linker | Own local-name → import-binding → canonical-target mapping independent of runtime slots. |
| `ImportBindingOrigin` | Add the stable import-binding identity alongside snapshot-local `SourceSiteId`. |
| `LinkedTypeResolver` | Stop using `ImportBindingId`/`LinkedReadSpec` as semantic import authority. |
| Checker imported values | Seed from symbolic imports; retain `BindingId → SourceImportBindingId`. |
| Type resolution | Expose import provenance; do not classify every `resolve_type_name` as interface-only. |
| Runtime value use | Record at semantic consumption points, not by occurrence-role/post-hoc `binding_read` scanning alone. |
| Incremental storage | Retain import-use facts with query state/products; transient recording is insufficient. |
| Aggregation | Publish a module-level phase product with explicit coverage. |
| Unknown/partial | Runtime. |
| Unused imports | Runtime for P4. |
| Whole-module/re-export | Runtime by fixed policy. |
| Generated runtime requirements | Must join the same evidence stream before finalization. |

---

# 22. Slice 2 conclusion

The incident correctly states that the missing problem is not merely classification—it is the lack of a canonical evidence owner and source-to-runtime contract. Pasted text

The evidence model can now be narrowed substantially:

> **`phalcom-modules` should own stable source-import-binding identity and symbolic target resolution. `phalcom-semantic` should own phase evidence for semantic uses of those exact bindings. Semantic query state must retain that evidence incrementally. A module-level semantic product joins it conservatively, and only that product may drive executable runtime projection.**

The two most significant newly identified implementation hazards are:

```text
1. ImportSiteId is too coarse:
   selective items need independent stable identities.

2. Type resolution is not synonymous with interface use:
   the same resolution path can produce a runtime class object.
```

Those findings materially tighten the blocked P4 plan.

## Next slice

**Slice 3 should audit phase-classification completeness itself:** enumerate every semantic path capable of consuming an imported declaration—annotations, generic constraints, trait/conformance formation, superclass references, aliases, ADTs, static definitions, callable bodies, class-object expressions, callable references, generated constructors, contextual realization and semantic-to-core lowering—and determine exactly which are `InterfaceOnly`, which are necessarily `Runtime`, and which must initially conservatively remain `Runtime`.