---
id: MODL001.C2.P4-ARCHITECTURE-RULING
program: MODL001
checkpoint: MODL001.C2
plan: MODL001.C2.P4
kind: architectural-ruling
status: APPROVED
decision: PROCEED_AFTER_PLAN_AMENDMENT
date: 2026-10-13
repository: aureat/phalcom-lang
branch: feat/telos-collections
branch_head_reviewed: b679028c8ff634e17828003572bd2a6e1a0d79ad
incident_baseline: e5988ad80a04769740462de98c51d58b56bd314d
---

# MODL001.C2.P4 — Architectural Ruling and Implementer Directive

## Phase-aware source dependencies: phase ownership, staged link products, P2/P3 ordering, executable finalization, and P4 unblock decision

## 0. Decision

**The architecture incident is sustained, the original one-stage P4 implementation premise is rejected, and the architectural hold is now resolved by this ruling.**

P4 may proceed **after** the implementer records this decision in the checkpoint and amends the P4 implementation plan so that its tasks and acceptance criteria conform to the staged architecture below.

The required architecture is:

```text
source + module interfaces
        │
        ▼
SEMANTIC / SYMBOLIC LINK
canonical import/declaration identities
semantic/reference graph
NO finalized runtime read layout
NO runtime initialization order
        │
        ▼
CANONICAL SEMANTIC ANALYSIS
per-import use evidence
generated executable/runtime requirements
incremental completeness
        │
        ▼
EXECUTABLE LINK FINALIZATION
runtime dependency projection
runtime linked-read layout
strict runtime DAG validation
deterministic initialization order
        │
        ▼
COMPILATION / MATERIALIZATION
consume finalized slots and requirements only
must not discover new external runtime dependencies
        │
        ▼
P3 RUNTIME PREPARATION / INITIALIZATION / EXECUTION
```

The central ruling is:

> **Semantic analysis owns the proof of whether an imported binding is interface-only or runtime. `phalcom-modules` owns stable import identity and the staged linking/finalization products. `phalcom-core` orchestrates the stages and consumes the finalized executable product. The bytecode compiler may validate finalized runtime requirements but must not invent them.**

The phase lattice remains:

```text
InterfaceOnly < Runtime
```

`Runtime` dominates. Missing, incomplete, unsupported, ambiguous, or stale evidence is `Runtime`.

No user-facing import-phase syntax is added in P4.

---

# 1. Why this ruling is required

The incident was valid because the live architecture currently makes the runtime decision before the information required to make that decision exists.

Current executable flow is effectively:

```text
source
  ↓
ModuleLinker::link_all
  ↓
collect imports
  ↓
create runtime edges
  ↓
allocate linked-read identities
  ↓
validate runtime DAG / initialization order
  ↓
semantic analysis
  ↓
compiler lowering
```

That ordering is sufficient only while every selective import is conservatively treated as runtime.

P4 intends to remove runtime edges for imports proven to be semantic/interface-only. Once that optimization exists, the current order becomes unsound because semantic analysis has not yet established:

- whether an imported name is used only in type/declaration formation;
- whether it is used as a runtime value;
- whether a type-looking use later requires a runtime class object;
- whether semantic elaboration selects an external callable, witness, declaration, or runtime type representation;
- whether lowering will require a linked read even though no ordinary source `Read` occurrence exists.

The implementation must therefore separate **semantic identity linking** from **executable runtime linking**.

This is not optional refactoring. It is the correctness boundary that makes phase-aware dependency removal safe.

---

# 2. Status of the incident

## 2.1 Incident trigger

The incident trigger was correct:

1. P4 required semantic usage evidence.
2. The current linker validates the runtime graph before semantic analysis.
3. Semantic resolution currently depends on structures coupled to runtime read layout.
4. Generated runtime declaration references may appear after source-name resolution.
5. Therefore the draft could not prove that deleting a runtime edge would also delete every executable runtime read requiring that edge.

The implementation was right to stop.

## 2.2 Is this an existing bug?

The current system is primarily **over-conservative**: selective imports are runtime by default. That can reject semantic-only cycles and creates unnecessary runtime closure, but usually avoids missing runtime dependencies.

P4 would make the system precise by removing some of those edges. That precision cannot be introduced until runtime-read completeness is authoritative.

The architectural bug is the conflation of:

```text
semantic import identity
runtime dependency edge
runtime linked-read slot
runtime initialization ordering
```

into one early linker product.

P4 must repair that conflation.

---

# 3. Normative P2/P3 ordering decision

This section settles the ambiguity between the historical implementation-plan ordering and the corrected compiler/runtime pipeline ordering.

## 3.1 Keep the plan history

Do **not** renumber or reorder `MODL001.C2.P2`, `P3`, and `P4` merely because P4 changes the internal pipeline.

Historical implementation-plan order remains:

```text
P1 → P2 → P3 → P4
```

P4 is a corrective/refinement plan implemented after the P2/P3 substrate already exists.

The fact that P4 follows P3 in the implementation program does **not** mean phase finalization runs after P3 runtime initialization.

These are different orderings:

- **plan ordering** = when architectural increments were designed/implemented;
- **compiler pipeline ordering** = how one program is transformed at runtime/compile time.

Do not conflate them.

## 3.2 Amend the P2 pipeline contract

The P2 document currently describes a sequence equivalent to:

```text
resolve
→ build interfaces
→ link
→ validate semantic graph
→ validate runtime DAG
→ compile
```

That ordering is no longer sufficient once semantic analysis can prove selected imports interface-only.

The corrected semantic meaning of P2 is:

```text
resolve complete reachable source universe
→ build interfaces
→ establish canonical symbolic imports/exports
→ build reference + semantic graph substrate
→ expose semantic-linked program to semantic analysis
```

P2 remains the owner of:

- module/import/export identity;
- symbolic resolution;
- `SymbolId` / `DeclarationId` authority;
- reference graph infrastructure;
- semantic graph infrastructure;
- runtime graph types and algorithms;
- executable-link finalization machinery at the module layer.

But P2 must no longer imply that the **final runtime graph is valid before semantic phase evidence exists**.

## 3.3 Amend the P3 input contract

P3 remains runtime materialization/initialization/execution.

Its conceptual input is no longer merely “a `LinkedProgram`”.

P3 must consume only a **finalized executable link product** whose:

- runtime dependency graph is complete;
- runtime graph is acyclic;
- linked-read layout is complete;
- initialization order is computed;
- every external runtime read has canonical authority;
- no interface-only import has an executable linked-read slot solely because it was imported.

Therefore the corrected runtime sequence is:

```text
SemanticLinkedProgram
    ↓
semantic analysis
    ↓
ExecutableLinkedProgram / FinalizedLinkedProgram
    ↓
CompiledProgram
    ↓
P3 materialize_program(...)
    ↓
allocate runtime module objects
    ↓
materialize linked reads / exports / declarations
    ↓
initialize finalized DAG
    ↓
execute entry
```

P3 must never accept the provisional semantic-link product as executable input.

## 3.4 P4 sits between P2 semantics and P3 execution in the live pipeline

P4's runtime-pipeline role is:

```text
P2 symbolic/semantic linking
        ↓
semantic analysis
        ↓
P4 executable finalization
        ↓
P3 runtime materialization/execution
```

Again, this does **not** require changing P4's historical `depends_on` metadata. It requires amending P2/P3 descriptions so the products are correctly staged.

---

# 4. Phase ownership — fixed decision

## 4.1 `phalcom-modules` owns identity and transport, not semantic proof

`phalcom-modules` must own:

- stable source import-site identity;
- stable selected-binding identity;
- canonical symbolic target identity;
- semantic/symbolic link product;
- graph types;
- executable finalization input/output transport types;
- runtime dependency projection;
- runtime DAG validation;
- runtime read-slot assignment.

It must **not** depend on `phalcom-semantic`.

Do not introduce a crate cycle.

## 4.2 `phalcom-semantic` owns phase evidence

`phalcom-semantic` owns the answer to:

> Why is this imported target being consumed, and does this use require runtime representation?

It must publish retained semantic products that identify:

- the exact imported binding;
- the canonical target;
- the strongest observed requirement;
- evidence/reason categories;
- coverage/completeness;
- incremental fingerprint/currentness.

Semantic analysis—not the linker, source occurrence index, or bytecode compiler—is the authority for the phase proof.

## 4.3 `phalcom-core` owns orchestration and executable semantic projection

`phalcom-core` already orchestrates program discovery, semantic analysis, semantic-to-runtime projection, compilation, and materialization.

It must orchestrate the corrected sequence:

```text
ModuleLinker::link_semantic(...)
→ SemanticWorkspaceSession
→ phase/runtime-requirement extraction
→ ModuleLinker::finalize_executable(...)
→ ProgramCompiler
→ VM materialization
```

Exact method names are flexible.

The boundary is not.

## 4.4 Compiler ownership is validation-only

The bytecode compiler may:

- look up an already-finalized runtime read slot;
- emit `GetLinked` using that slot;
- assert that a required external declaration has finalized runtime authority;
- fail with an internal compiler invariant if the finalized product is incomplete.

The bytecode compiler must not:

- append new runtime linked reads to a finalized module;
- create a runtime graph edge;
- decide an import's phase;
- repair a missing runtime dependency by dynamically adding a slot;
- reinterpret source spelling to find another target.

---

# 5. Stable import binding identity — fixed decision

`ImportSiteId` is statement-granular and therefore insufficient as the phase key.

For:

```telos
from foo import A, B as Bee, C
```

there is one import statement but three independently classifiable local bindings.

P4 must introduce a stable item-level identity in `phalcom-modules`.

Recommended shape:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImportBindingLocalId(pub u32);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceImportBindingId {
    pub site: ImportSiteId,
    pub local: ImportBindingLocalId,
}
```

Names may vary, but the semantics may not.

The identity layers are:

| Identity | Meaning |
|---|---|
| `ImportSiteId` | one authored import/re-export declaration |
| `SourceImportBindingId` | one local binding introduced by that declaration |
| `SourceSiteId` | snapshot-local source-index occurrence/binding identity |
| `SymbolId` / `DeclarationId` | canonical remote semantic target |
| `ImportBindingId` | finalized executable runtime read slot identity |

Do not collapse these layers.

`SourceSiteId` is snapshot-local and must not become the durable phase key.

`ImportBindingId` must not exist for a pure interface-only selective binding unless another executable requirement explicitly needs a runtime slot.

---

# 6. Required staged products

Use distinct Rust types. Do not rely on a boolean such as `validated: bool` if the type system can prevent stage confusion.

Exact private names are flexible; semantic distinctions are fixed.

## 6.1 Stage A — semantic/symbolic linked program

Recommended conceptual shape:

```rust
pub struct SemanticLinkedProgram {
    pub universe: Arc<ProjectUniverse>,
    pub modules: BTreeMap<ModuleId, SemanticLinkedModule>,
    pub graphs: SemanticLinkGraphs,
    pub entry: ModuleId,
}
```

A module contains:

```rust
pub struct SemanticLinkedModule {
    pub interface: LinkedModuleInterface,
    pub local_globals: BTreeMap<Box<str>, GlobalBindingId>,
    pub imports: SymbolicImportTable,
}
```

with entries conceptually like:

```rust
pub struct SymbolicImportBinding {
    pub id: SourceImportBindingId,
    pub local_name: Box<str>,
    pub target: SymbolicImportTarget,
    pub range: SourceRange,
}

pub enum SymbolicImportTarget {
    Module(ModuleId),
    Binding(SymbolId),
}
```

Properties:

- every source import remains canonically resolvable;
- selected imports have stable item identity;
- aliases map to the same canonical remote declaration identity;
- semantic cycles are representable;
- no claim is made that a selected binding has a runtime slot;
- no final runtime DAG exists;
- no initialization order exists;
- no P3 runtime API accepts this type.

A provisional/reference runtime graph may exist internally only as conservative diagnostics/debug state. It is not executable authority.

## 6.2 Stage B — semantic phase/runtime requirement product

Semantic analysis publishes a retained product per module.

Recommended conceptual shape:

```rust
pub struct ModuleImportPhaseProduct {
    pub module: ModuleId,
    pub bindings: BTreeMap<SourceImportBindingId, ImportPhaseSummary>,
    pub coverage: PhaseEvidenceCoverage,
    pub fingerprint: ProductFingerprint,
}

pub struct ImportPhaseSummary {
    pub target: SemanticTargetId,
    pub phase: DependencyPhase,
    pub reasons: Box<[ImportUseReason]>,
    pub observed: bool,
}
```

Coverage must be explicit:

```rust
pub enum PhaseEvidenceCoverage {
    Complete,
    Incomplete,
}
```

A richer reason for incompleteness is acceptable.

Do **not** infer phase completeness merely from a generic snapshot status.

Semantic evidence can be complete only when all phase-relevant owners for the module are current and accounted for.

## 6.3 Stage C — executable linked program

Recommended conceptual shape:

```rust
pub struct ExecutableLinkedProgram {
    pub universe: Arc<ProjectUniverse>,
    pub modules: BTreeMap<ModuleId, ExecutableLinkedModule>,
    pub graphs: ModuleGraphs,
    pub entry: ModuleId,
    pub initialization_order: Vec<ModuleId>,
}
```

`ExecutableLinkedModule` contains:

```rust
pub struct ExecutableLinkedModule {
    pub interface: LinkedModuleInterface,
    pub bindings: ExecutableBindingLayout,
    pub linked_reads: Vec<LinkedReadSpec>,
    pub runtime_dependencies: Vec<ModuleId>,
}
```

This is the only linked product permitted to enter `CompiledProgram` / P3 materialization.

If retaining `LinkedProgram` as the final type minimizes churn, that is acceptable **only if** the provisional semantic product has a different type and cannot enter executable APIs.

---

# 7. Semantic resolver must stop depending on runtime read slots

Current `LinkedTypeResolver` resolves selected imported types through a chain equivalent to:

```text
local name
→ ModuleBindingLayout.imports
→ ImportBindingId
→ linked_reads[slot]
→ LinkedReadSpec::Binding(SymbolId)
→ DeclarationId
```

This is exactly the coupling P4 must remove.

Semantic resolution must instead use:

```text
local name
→ SourceImportBindingId
→ SymbolicImportBinding
→ canonical SymbolId / ModuleId
→ DeclarationId
```

No `ImportBindingId` is required.

No finalized runtime slot is required.

This migration is a prerequisite for safely withholding runtime slots from interface-only imports.

---

# 8. Semantic provenance and recording

## 8.1 Resolver reports identity/provenance; consumer reports phase

Do not make `resolve_type_name()` automatically record `InterfaceOnly`.

The same type resolver participates in both:

```telos
fn f(x: ImportedType) { }
```

and:

```telos
ImportedType
```

The first can be interface-only; the second materializes the runtime class object.

The semantic resolver therefore owns:

```text
what did this name resolve to?
which SourceImportBindingId supplied it?
```

The semantic use site owns:

```text
why is the target being consumed?
does that use require runtime representation?
```

A suitable resolver result is conceptually:

```rust
pub struct ResolvedImportedName {
    pub declaration: DeclarationId,
    pub import_binding: Option<SourceImportBindingId>,
}
```

A side lookup is acceptable if it uses the same canonical symbolic import authority.

Do not independently re-resolve source text.

## 8.2 Preserve runtime-binding provenance in checker scopes

When imported values are seeded into `CheckingContext`, retain:

```rust
BTreeMap<BindingId, SourceImportBindingId>
```

or equivalent.

This allows:

- ordinary `Expr::Var` reads;
- unqualified calls;
- callable references;
- writes/errors;
- associated operations;
- other body-local consumers

to attribute use back to the exact imported binding.

`ExpressionAnalysis::binding_read` alone is not complete enough.

## 8.3 Evidence must be query-retained

Do not implement a transient process-global `ImportUseRecorder`.

Incremental query reuse can skip recomputation. If evidence exists only as a side effect of executing a query, reused queries will silently lose phase evidence.

Phase evidence must be retained and reused atomically with the semantic query/product that established it.

Conceptual form:

```rust
pub struct QueryImportUseEvidence {
    pub module: ModuleId,
    pub uses: Arc<[ImportUseFact]>,
    pub complete: bool,
    pub fingerprint: ProductFingerprint,
}
```

The exact storage mechanism may be:

- an explicit query product;
- a sidecar carried by an existing product;
- another retained incremental structure.

The invariants are:

1. evidence survives query reuse;
2. evidence retracts when its owner invalidates;
3. evidence has content-sensitive fingerprinting;
4. cold and incremental aggregation produce identical module phase products.

---

# 9. Phase classification decision

## 9.1 Interface-only source uses

The following may contribute `InterfaceOnly` when canonical semantic analysis proves no executable representation is required by that use:

- parameter type annotations;
- return type annotations;
- field/member type annotations;
- type alias RHS;
- generic type arguments in declaration/type formation;
- generic bounds;
- `where` constraints;
- trait/protocol references used purely for semantic constraints;
- associated type projections used only for type formation;
- ADT/data/enum payload type declarations;
- callable signature types.

These are **use-level** classifications, not declaration-kind classifications.

## 9.2 Runtime source uses

The following contribute `Runtime`:

- ordinary imported value reads;
- imported callable invocation;
- imported callable/family references;
- imported class/type declaration used as a runtime expression;
- executable `Expr::TypeForm`;
- associated runtime lookup/invocation;
- runtime pattern/class checks;
- runtime decorators/attributes;
- imported static constants used as executable values;
- whole-module imports;
- direct re-exports.

## 9.3 Superclass references are Runtime

This is fixed.

An imported superclass participates in semantic hierarchy formation, but current class lowering also emits a runtime read of the superclass class object before `Bytecode::Class`.

Therefore:

```text
SemanticEdgeKind::Superclass
```

does not imply:

```text
DependencyPhase::InterfaceOnly
```

The same relation also contributes an executable requirement equivalent to:

```text
RuntimeDependencyReason::RuntimeDeclarationReference
```

until a future accepted runtime representation proves otherwise.

Do not optimize external superclass edges to interface-only in P4.

## 9.4 Static constants remain Runtime in P4

A cross-module static constant used only during static evaluation may eventually deserve a distinct compile-time dependency phase.

P4 does not introduce that third phase.

Until a first-class compile-time artifact dependency model exists:

```text
cross-module static constant dependency → Runtime
```

conservatively.

## 9.5 Unused selective imports remain Runtime in P4

Absence of observed use is not proof.

For this plan:

```text
unused / unobserved / unsupported selected import → Runtime
```

A later cleanup plan may introduce proven-unused erasure after the evidence/completeness system is mature.

---

# 10. Generated executable requirements are first-class evidence

P4 must not classify only user-authored identifier occurrences.

Semantic elaboration and runtime semantic projection can introduce executable requirements that have no ordinary source `Read` occurrence.

The finalizer must include them.

At minimum audit and represent the following families.

## 10.1 Runtime declaration materialization

Examples:

- imported superclass class object;
- imported nominal runtime pattern;
- contextual realization requiring a declaration class object;
- runtime type-form materialization.

Use:

```text
RuntimeDependencyReason::RuntimeDeclarationReference
```

or an equivalent retained reason.

The existing enum case is suitable; the missing piece is a production authority that emits it before final executable graph validation.

## 10.2 Runtime type/reification requirements

`ProgramCompiler` currently projects semantic types into runtime metadata roots and `RuntimeTypeRecipe`s.

If that projection requires a nominal runtime owner from another module, that requirement must participate in runtime closure before finalization.

Do not assume:

```text
source annotation == erased forever
```

The source annotation can be `InterfaceOnly` while a separately generated reification requirement upgrades the dependency to `Runtime`.

## 10.3 Selected external callable owners

If semantic analysis chooses an executable callable owned by another module, executable closure must contain the required owner even if the source spelling did not create an ordinary imported-value read.

Reduce semantic-only callable identity to a modules/core finalization transport requirement without creating a `phalcom-modules -> phalcom-semantic` dependency.

## 10.4 Conformance/witness/runtime trait environment owners

Pure trait references can be interface-only.

Selected executable conformance/witness implementations are separate runtime requirements.

Do not upgrade a trait-definition module merely because its trait name appeared in a constraint if runtime evidence is actually owned elsewhere.

Conversely, do not omit the actual witness/implementation owner's runtime requirement.

## 10.5 Runtime decorators/attributes

Non-compiler-only attributes synthesize executable construction/attach behavior.

Those uses are runtime even if they are outside ordinary callable-body analysis.

## 10.6 Patterns and contextual realization

Executable pattern class checks and contextual declaration realization can require external runtime declaration identity.

They must be represented before executable finalization.

---

# 11. Finalization input must combine two lanes

The runtime graph is not merely a filtered import graph.

Finalization must combine:

```text
A. source import phase evidence
   SourceImportBindingId → InterfaceOnly | Runtime

B. generated executable requirements
   declaration/module/callable/witness/runtime-type owner → Runtime
```

Then project one authoritative executable closure.

Conceptually:

```text
runtime requirements =
    fixed whole-module imports
  ∪ fixed direct re-exports
  ∪ selected imports finalized Runtime
  ∪ runtime declaration requirements
  ∪ external selected callable owners
  ∪ runtime witness/conformance owners
  ∪ runtime type/reification owners
  ∪ any conservative unsupported requirement
```

Only then:

```text
runtime requirements
    ↓
RuntimeDependencyGraph
    ↓
validate acyclic
    ↓
linked-read slots
    ↓
runtime dependency list
    ↓
initialization order
```

The exact order of “edge creation” and “slot assignment” inside finalization is mechanically flexible, provided they are projections of the same requirement authority and published atomically.

---

# 12. Executable read layout and graph must have one authority

This is a hard invariant.

For every external executable read:

```text
GetLinked(slot)
```

there must be:

1. a finalized read slot;
2. a canonical target;
3. a runtime requirement;
4. a corresponding runtime dependency owner;
5. a materialization path;
6. an initialization ordering proof.

Conversely, a selected import finalized `InterfaceOnly` must not receive a `GetLinked` slot solely because the import exists.

Do not implement P4 by only deleting runtime graph edges while retaining all current `linked_reads`.

That would be architecturally invalid.

---

# 13. `ImportBindingId` is executable-only

This is now fixed.

`ImportBindingId` represents an executable linked-read slot.

It is not:

- semantic import identity;
- source import identity;
- declaration identity;
- phase evidence identity.

An interface-only selected import may have:

```text
SourceImportBindingId
SymbolId / DeclarationId
semantic name-resolution entry
```

and **no**:

```text
ImportBindingId
LinkedReadSpec
RuntimeLinkedRead
```

unless another runtime requirement independently needs an executable read of that target.

---

# 14. Compiler late-discovery path must be closed

`phalcom-core/src/compiler/lib/expr.rs::emit_declaration_class_reference` currently contains fallback behavior capable of appending a new runtime linked read for an external/canonical declaration after the linked program has already been created.

That behavior is incompatible with finalized executable linking.

For the `ProgramCompiler` / finalized-program path:

> **The compiler must never mutate `linked_reads` to repair missing executable authority.**

Required behavior:

```text
external declaration requested
        ↓
lookup finalized runtime read mapping
        ├─ found → emit GetLinked
        └─ missing → internal compiler/finalization invariant error
```

The missing mapping means the semantic/runtime-requirement closure was incomplete.

Do not silently recover.

If a standalone/legacy compiler entry point outside the finalized-program architecture must temporarily retain legacy fallback behavior, isolate it explicitly and prove it cannot be reached from `ExecutableLinkedProgram`/`CompiledProgram`. Do not let that compatibility path weaken P4's program-level invariant.

---

# 15. Conservative finalization algorithm

For each selective `SourceImportBindingId`:

```text
if module coverage != Complete:
    Runtime

else if binding summary missing:
    Runtime

else if binding not observed:
    Runtime

else if any observed source use == Runtime:
    Runtime

else if any generated executable requirement maps to the binding/target:
    Runtime

else:
    InterfaceOnly
```

Additionally:

```text
whole-module import:
    Runtime

direct re-export:
    Runtime

cross-module static constant dependency:
    Runtime for P4

unsupported/unclassified:
    Runtime
```

The finalizer may deduplicate module-level runtime dependencies after per-binding classification.

Do not let deduplication destroy reason/provenance needed for diagnostics and fingerprints.

---

# 16. Completeness is explicit

The statement:

```text
"we did not see a runtime use"
```

is not proof.

`InterfaceOnly` requires:

```text
all relevant phase owners were analyzed
AND
all their evidence is current
AND
all use categories are supported
AND
all generated executable requirements are included
AND
no runtime use exists
```

Therefore `ModuleImportPhaseProduct` requires an explicit coverage/completeness field.

A generic semantic snapshot status is insufficient unless it is formally strengthened to imply this exact property.

P4 should prefer an explicit phase-coverage product rather than silently overloading another status.

---

# 17. Incremental ownership and invalidation

The following edits must invalidate or recompute the relevant phase evidence/finalized executable projection:

- type-only use → runtime value use;
- runtime value use → type-only use;
- import alias changes;
- selected import item addition/removal;
- imported target/re-export ownership changes;
- callable body edits changing runtime use;
- declaration signature edits changing semantic-only use;
- superclass change;
- runtime attribute change;
- pattern target change;
- semantic selection of another callable/witness;
- runtime type-reification requirement change.

The phase product requires a content-sensitive fingerprint separate from an unrelated semantic output if the unrelated output can remain equivalent while import phase changes.

Cold and incremental results must match for:

```text
per-binding phase
runtime edges
linked-read slots
runtime cycle diagnostics
initialization order
```

---

# 18. Required source-area changes

## 18.1 `phalcom-modules`

Primary areas:

```text
phalcom-modules/src/identity.rs
phalcom-modules/src/linker.rs
phalcom-modules/src/graph.rs
phalcom-modules/src/interface.rs
phalcom-modules/src/lib.rs
```

Required outcomes:

- item-level source import binding identity;
- symbolic import table;
- semantic-linked product;
- executable-linked product/finalizer;
- runtime graph construction after phase input;
- deterministic slot assignment after final classification;
- strict DAG validation only for executable product.

## 18.2 `phalcom-semantic`

Primary areas:

```text
phalcom-semantic/src/resolver.rs
phalcom-semantic/src/module_bindings.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/scope.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/static_body.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/module_product.rs
```

Required outcomes:

- semantic resolution no longer requires runtime `ImportBindingId`;
- imported checker bindings retain `SourceImportBindingId` provenance;
- use-site phase recording;
- retained incremental phase evidence;
- explicit module coverage;
- generated semantic/runtime requirement export where semantic authority exists.

## 18.3 `phalcom-core`

Primary areas:

```text
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/materialize.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/patterns.rs
```

Required outcomes:

- corrected orchestration order;
- generated runtime requirements closed before finalization;
- compile modules from `ExecutableLinkedProgram`;
- compiler does not append late linked reads;
- P3 materializer receives only finalized executable products;
- superclass/type-form/contextual/pattern/runtime-metadata requirements are represented.

---

# 19. Required implementation sequence

The previous P4 T1/T2/T3 outline should be amended to the following order. Task names may remain compatible with repository conventions, but dependencies must follow this sequence.

## T0 — Record the architectural decision and amend P2/P3/P4 contracts

Before production source edits:

1. record this ruling in `MODL001.C2/CHECKPOINT.md`;
2. change P4 from architectural hold to approved/active according to repository lifecycle conventions;
3. amend P4 architecture/tasks to match this ruling;
4. amend P2's pipeline language so final runtime DAG validation is post-semantic for phase-aware programs;
5. amend P3's input contract so runtime materialization consumes only finalized executable links;
6. preserve the historical plan dependency order.

Acceptance:

- no document still directs an implementer to validate the final runtime DAG before semantic phase evidence exists;
- no document implies a provisional semantic link is executable.

## T1 — Introduce stable selected-import identity and symbolic link product

Implement:

- `SourceImportBindingId` equivalent;
- one identity per selective item;
- symbolic import table keyed by stable item identity;
- canonical local name → symbolic import lookup;
- semantic-linked program type.

Keep existing conservative executable behavior available during transition if necessary.

Do not yet remove runtime edges unless the new finalizer is authoritative.

Acceptance:

- semantic selected imports are resolvable without `ImportBindingId`;
- aliases and multiple items in one selective statement have distinct stable identities.

## T2 — Migrate semantic resolution off executable linked-read slots

Replace `LinkedTypeResolver` and checker seeding dependence on:

```text
ImportBindingId → LinkedReadSpec
```

with:

```text
SourceImportBindingId → SymbolicImportTarget
```

Retain checker `BindingId → SourceImportBindingId` provenance.

Acceptance:

- semantic analysis runs against the semantic-linked product;
- deleting an executable runtime slot would no longer make a type-only import semantically unresolvable.

## T3 — Implement retained phase evidence

Add:

- use-site `InterfaceOnly`/`Runtime` recording;
- reason taxonomy;
- query-retained evidence;
- module aggregation;
- explicit completeness;
- phase fingerprinting.

Initially cover the known classification matrix.

Unknown categories remain Runtime.

Acceptance:

- focused tests prove type annotation = InterfaceOnly;
- bare value read/call = Runtime;
- mixed use joins Runtime;
- edit parity works cold/incremental.

## T4 — Close generated executable/runtime requirements

Before executable finalization, audit and publish requirements for at least:

- superclass materialization;
- class-object/type-form runtime values;
- contextual declaration realization;
- runtime pattern/class checks;
- runtime decorators/attributes;
- runtime type/reification roots;
- selected external callable owners;
- runtime conformance/witness owners;
- cross-module static constant dependency (conservative Runtime).

Acceptance:

- `RuntimeDependencyReason::RuntimeDeclarationReference` or equivalent has a real production producer;
- generated requirements are visible before DAG validation;
- no required family is left solely to bytecode emission.

## T5 — Implement executable finalizer

Input:

```text
SemanticLinkedProgram
+ ModuleImportPhaseProduct(s)
+ generated runtime requirements
```

Output:

```text
ExecutableLinkedProgram
```

Finalizer:

1. classify each selected binding;
2. project runtime requirements;
3. build runtime graph with reasons;
4. assign executable read slots from the same authority;
5. validate strict DAG;
6. compute initialization order;
7. publish atomically.

Acceptance:

- interface-only selected import has semantic identity but no executable slot;
- runtime selected import has slot and edge;
- semantic cycles can remain legal while runtime graph is acyclic;
- actual runtime cycle still fails.

## T6 — Cut compiler/materializer to finalized authority

Change program compilation/materialization to consume only `ExecutableLinkedProgram`.

Remove program-level late runtime-read insertion.

Any missing finalized external runtime mapping is an internal error.

Acceptance:

- every emitted linked read is covered by finalized authority;
- materializer never sees a provisional link;
- no compiler path mutates finalized read layout.

## T7 — Focused verification matrix

Required focused cases:

1. selected type annotation only → InterfaceOnly;
2. selected generic/trait constraint only → InterfaceOnly;
3. same selected binding type + value use → Runtime;
4. imported callable invocation → Runtime;
5. imported class object expression → Runtime;
6. executable type form → Runtime;
7. external superclass → RuntimeDeclarationReference;
8. runtime pattern/class check → Runtime;
9. runtime attribute/decorator → Runtime;
10. unknown/unsupported evidence → Runtime;
11. unused selected binding → Runtime in P4;
12. whole-module import → Runtime;
13. direct re-export → Runtime;
14. semantic-only cycle accepted with runtime DAG acyclic;
15. actual runtime cycle rejected;
16. phase edit incremental parity;
17. multiple selected items from one statement classify independently;
18. aliasing retains canonical remote identity;
19. finalized compiler emits no uncovered `GetLinked`;
20. missing finalized mapping fails closed.

## T8 — Re-run the Universe iteration split proving fixture

Only after T1–T7 pass.

Prove the intended S4 shape:

```text
protocol/default module
    ─runtime→ adapter module

adapter module
    ─interface-only→ protocol/default module
```

with one canonical protocol identity.

Then prove the paired negative control in which the adapter performs a runtime value read back into the protocol module and the cycle becomes a real runtime cycle and is rejected.

Do not perform production iteration cutover as part of P4.

---

# 20. Tests that are specifically required to prove the stage boundary

Add tests that make stage misuse mechanically visible.

## 20.1 Provisional product cannot execute

A `SemanticLinkedProgram` must not be accepted by:

- `ProgramCompiler` executable compilation entry points;
- P3 materialization;
- runtime initialization;
- any API that requires initialization order.

Prefer a type mismatch over a runtime boolean check.

## 20.2 Interface-only binding has no runtime slot

Fixture:

```telos
from b import T

fn f(x: T) -> T { x }
```

Assert:

```text
canonical declaration target retained
phase = InterfaceOnly
no ImportBindingId for T
no LinkedReadSpec for T
no runtime edge solely for T
```

## 20.3 External superclass has runtime authority

Fixture:

```telos
from b import Base

class Child is Base {}
```

Assert:

```text
semantic superclass edge exists
phase/runtime requirement = Runtime
RuntimeDeclarationReference edge exists
runtime read slot exists
compiler uses finalized slot
```

## 20.4 Compiler cannot repair finalization

Construct or mutate a test executable product so a required runtime declaration mapping is missing.

Compiler must fail with an internal invariant.

It must not append `linked_reads`.

---

# 21. Diagnostics and failure semantics

User-facing runtime cycle diagnostics remain based on the finalized runtime graph.

They should distinguish reasons where practical:

```text
selective runtime value import
whole-module import
re-export
runtime declaration reference
```

A semantic-only cycle must not be reported as a runtime initialization cycle.

A missing finalized runtime mapping during compilation is not a source circular-import diagnostic; it is an internal compiler/link-finalization defect.

Do not turn it into a permissive fallback.

---

# 22. LSP/source-index boundary

P4 is not an LSP redesign.

The source index may consume or present:

- stable canonical import target;
- phase evidence;
- source occurrence information.

But source occurrences are not phase authority because:

- `Reference` is not automatically interface-only;
- `Read` is not a complete list of runtime uses;
- generated runtime requirements may have no source occurrence.

Do not create an LSP-specific phase classifier.

If LSP changes are mechanically required because link product type names change, keep them projection-only.

---

# 23. Fixed non-goals

P4 must not expand into:

- `import type` syntax;
- separate type/value namespaces;
- lazy runtime module loading;
- partial module visibility;
- runtime SCC execution semantics;
- generalized unused-import elimination;
- a three-phase Interface/CompileTime/Runtime dependency model;
- runtime static-constant artifact redesign;
- compiler-wide declaration identity redesign;
- new trait runtime model;
- iteration API redesign;
- production Universe cutover.

If one of these appears necessary for correctness, stop and report a new architecture incident.

---

# 24. STOP / CONSULT triggers after this ruling

The previous broad architectural questions are now decided. Do not stop merely because staged linking requires touching several crates.

Stop only if live evidence disproves one of these settled premises:

1. canonical semantic name resolution cannot be migrated off runtime slots without changing language semantics;
2. semantic query products cannot retain import-use evidence under reuse without a fundamental incremental DB redesign;
3. a generated runtime requirement cannot be identified before bytecode compilation even after semantic-to-core projection;
4. P3 materialization or another executable API fundamentally requires runtime initialization order before semantic analysis;
5. finalized read layout cannot be made immutable for the program compiler;
6. source import item identity cannot remain stable enough across one semantic snapshot/revision to support incremental replacement;
7. the only sound fix requires new user-facing syntax or partial-module runtime semantics.

When escalating, provide:

- exact symbol/path;
- exact violated invariant;
- minimal fixture;
- whether the failure is semantic identity, completeness, finalization, or runtime representation;
- one proposed containment strategy.

---

# 25. Acceptance invariants

P4 is architecturally complete only when all of the following hold.

### INV-P4-A — Stage separation

A semantic-linked program and executable-linked program are distinct states, preferably distinct Rust types.

### INV-P4-B — Stable per-binding identity

Each selective imported item has a stable identity independent of runtime slot allocation.

### INV-P4-C — Semantic phase ownership

Only semantic products can prove a selected binding interface-only.

### INV-P4-D — Consumer-owned classification

Resolver identity lookup does not itself imply InterfaceOnly; semantic use sites classify consumption.

### INV-P4-E — Explicit completeness

Missing/incomplete coverage cannot produce InterfaceOnly.

### INV-P4-F — Runtime dominance

Any runtime source use or generated executable requirement joins the binding/module requirement to Runtime.

### INV-P4-G — Single executable projection

Runtime graph and linked-read layout derive from the same finalized requirement authority.

### INV-P4-H — Compiler cannot discover dependencies

Program bytecode lowering cannot add an external runtime dependency or linked read.

### INV-P4-I — Strict executable DAG

Only the finalized runtime graph is DAG-validated; actual runtime cycles remain errors.

### INV-P4-J — Semantic cycles remain representable

Semantic/interface cycles may survive when the finalized runtime projection is acyclic.

### INV-P4-K — P3 stage isolation

Runtime materialization/initialization accepts only finalized executable products.

### INV-P4-L — Incremental equivalence

Cold and incremental analysis agree on phase, runtime closure, read layout, diagnostics, and order.

---

# 26. Required documentation amendments

Before implementation begins, amend at least:

```text
docs/implementation/MODL001-module-architecture/C2-module-project-model/CHECKPOINT.md
docs/implementation/MODL001-module-architecture/C2-module-project-model/P2-module-linking-graphs-and-lsp.md
docs/implementation/MODL001-module-architecture/C2-module-project-model/P3-module-runtime-initialization-and-execution.md
docs/implementation/MODL001-module-architecture/C2-module-project-model/MODL001.C2.P4-phase-aware-source-dependencies.md
```

Required wording changes:

### P2

Replace any normative implication that the final runtime DAG is validated before semantic analysis with staged-link wording.

### P3

State that `CompiledProgram`/runtime materialization contains or consumes a finalized executable link product, never a provisional semantic link.

### P4

Remove the architecture hold and replace the ambiguous phase/finalization portions with this ruling.

P4's detailed implementation plan may use different private type names, but it must preserve the boundaries and invariants in this document.

### Checkpoint

Record:

- incident resolved by staged-link ruling;
- semantic phase owner;
- stable item-level import identity requirement;
- generated runtime requirement closure;
- P2/P3 pipeline amendment;
- P4 allowed to proceed after T0 documentation alignment.

---

# 27. Implementer authority after this ruling

The implementer may decide without further consultation:

- exact Rust type names;
- file placement;
- whether finalization is a `ModuleLinker` method, separate finalizer object, or free function;
- whether reason evidence is stored as a set, compact enum list, or diagnostic-only side table;
- internal map/set choices;
- exact incremental query decomposition;
- migration order of individual semantic checker call sites;
- temporary compatibility adapters while T1–T6 are incomplete.

The implementer may **not** decide to change:

- phase ownership;
- conservative fallback;
- stage separation;
- stable selected-item identity;
- generated runtime requirement closure;
- compiler validation-only role;
- strict runtime DAG semantics;
- P3 finalized-input requirement;
- whole-module/re-export Runtime policy;
- superclass Runtime policy for P4;
- unused-import Runtime policy for P4;
- no-new-syntax scope.

---

# 28. Final architectural answer to the incident

The incident asked, in effect:

> Where should phase be decided, and how can semantic analysis depend on linking without forcing the final runtime graph to exist first?

The answer is:

```text
1. Link canonical names first, without allocating executable import slots.
2. Run canonical semantic analysis against those symbolic imports.
3. Let semantic consumers publish retained per-binding phase evidence.
4. Add generated executable requirements from semantic-to-core projection.
5. Finalize one executable runtime closure from both evidence lanes.
6. Allocate linked-read slots from that closure.
7. Validate the final runtime DAG and compute initialization order.
8. Compile and materialize using those immutable finalized mappings.
```

This is the architecture P2 was conceptually aiming toward, but the live implementation collapsed its semantic and runtime products too early.

P4's job is to establish that missing stage boundary.

**Decision: approved. Proceed after T0 plan/checkpoint amendments.**
