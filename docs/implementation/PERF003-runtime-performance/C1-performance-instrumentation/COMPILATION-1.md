# Implementation Requirements — Architecturally Correct Compilation-Speed Fixes

## 1. Executive conclusion

The investigation is far enough along to stop treating the ~45-second floor as an unidentified hotspot. The empirical behavior is clear: ordinary fixtures from 345 bytes through 7.5 KB all compile in roughly 44–47 seconds, with compilation accounting for the overwhelming majority of wall time. Pasted text The source-level diagnosis is also now coherent: **one-shot compilation repeatedly reconstructs canonical, semantic, presentation, advisory, and metadata products whose cost is controlled by the Universe/workspace rather than the user program**. This is the semantic-workspace amplification thesis from the original investigation, but substantially narrowed and strengthened. Pasted text

The architecturally correct fix is **not**:

- simply switch tests to `ProgramCompilationSession`;
- enable `opt-level=3`;
- cache one or two functions;
- remove sorting;
- special-case `run_inline`;
- memoize a few scans while retaining the same lifecycle.

Those can improve numbers without fixing the compilation model.

The correct target architecture is:

```text
                     ┌────────────────────────────────────┐
                     │ Canonical Universe Compilation Image│
                     │                                    │
                     │ parsed/interface/link products      │
                     │ declarations / signatures           │
                     │ hierarchy / dispatch                │
                     │ conformance evidence                │
                     │ reusable query products             │
                     │ optional presentation shards        │
                     │ canonical metadata pool             │
                     └─────────────────┬──────────────────┘
                                       │ immutable dependency
                                       │
                         ┌─────────────▼──────────────┐
                         │ User compilation overlay   │
                         │                            │
                         │ changed/user modules only  │
                         │ query delta                │
                         │ affected semantics only    │
                         └─────────────┬──────────────┘
                                       │
                          demand-driven product graph
                                       │
                 ┌─────────────────────┼─────────────────────┐
                 ▼                     ▼                     ▼
          executable product     diagnostics product   tooling/presentation
          + minimal metadata     when requested        when requested
```

A 500-byte program should be compiled **as a 500-byte overlay on an immutable Universe**, not as a request to reconstruct a substantial fraction of the Universe compiler state.

The right external model is close to what mature query compilers pursue: computations are keyed and memoized, dependencies are explicit, and unchanged queries are not executed merely to recover products known to be unchanged. Rust's incremental query architecture, for example, explicitly separates determining that a query remains valid from recomputing its value, and preserves query dependency state across compilations. [Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation.html?utm_source=chatgpt.com)

---

# 2. This should be implemented as several coordinated plans, not one giant patch

I would **not** implement all findings under one implementation plan. There are at least five independently reviewable architectural boundaries:

| Package | Purpose | Risk |
|---|---|---:|
| **A. Canonical Universe dependency image** | Stop reconstructing canonical module/interface/semantic state | High |
| **B. Consumer-specific semantic publication** | Stop executable compilation from eagerly building tooling/advisory products | High |
| **C. Semantic/conformance incrementalization** | Eliminate repeated body/witness/proof work | Very high |
| **D. Metadata reachability architecture** | Implement actual profile/root-driven metadata export | High |
| **E. Local complexity corrections** | Remove `N×M`, repeated AST walks, repeated lookup scans | Medium |

Instrumentation should span all five, but source changes should remain independently attributable.

The sequencing matters. If you optimize loops before correcting the lifecycle, you risk making expensive global work faster instead of eliminating it.

---

# 3. Package A — Canonical Universe must become a true compilation dependency

This is the highest-level structural requirement.

Today there is a canonical semantic baseline, but it ends too early. A fresh compilation still reconstructs interfaces, query rows, conformances, presentation products, and other derived state.

The implementation needs a stronger immutable artifact. I would give it an explicit conceptual identity such as:

```rust
pub struct CanonicalUniverseCompilationImage {
    pub identity: UniverseImageIdentity,

    // Module layer
    pub parsed_modules: Arc<CanonicalParsedModules>,
    pub interfaces: Arc<CanonicalInterfaceSet>,
    pub declaration_catalog: Arc<UniverseSourceDeclarationCatalog>,
    pub import_products: Arc<CanonicalImportProducts>,
    pub topology: Arc<ModuleTopology>,
    pub linked: Arc<LinkedProgram>,

    // Formal semantic layer
    pub type_store: Arc<TypeStore>,
    pub declarations: Arc<DeclarationTypeTable>,
    pub hierarchy: Arc<MapTypeHierarchy>,
    pub dispatch: Arc<SurfaceDispatchResolver>,
    pub callable_signatures: Arc<CallableSignatureTable>,
    pub algebraic_products: Arc<CanonicalAlgebraicProducts>,

    // Evidence layer
    pub conformances: Arc<ConformanceIndex>,
    pub witness_plans: Arc<ConformanceWitnessPlans>,
    pub trait_dispatch: Arc<TraitDispatchIndex>,
    pub generic_conformance_roots: Arc<...>,

    // Reusable query state
    pub semantic_roots: Arc<CanonicalSemanticQueryImage>,

    // Optional consumer products
    pub presentation: Option<Arc<CanonicalPresentationImage>>,
    pub metadata: Arc<CanonicalMetadataImage>,
}
```

The names need not be these exact names. The ownership boundary is the requirement.

## 3.1 Canonical products must be immutable

A user session must **never mutate canonical products**.

Use:

```text
canonical immutable root
          │
          └── persistent/forked user overlay
```

rather than:

```text
clone canonical mutable table
          │
          └── mutate independent copy
```

`TypeStore` already demonstrates the intended persistent-data-structure direction. Extend that principle to semantic products where practical.

## 3.2 Give the image a stable identity

Every image should carry something equivalent to:

```rust
struct UniverseImageIdentity {
    schema_version: u32,
    source_fingerprint: Fingerprint,
    compiler_semantic_version: Fingerprint,
}
```

Anything retaining canonical query products must invalidate them whenever:

- Universe source changes;
- semantic rules change;
- metadata schema changes where relevant;
- builtin/native surface changes.

Cross-session reuse must be **sound**, not merely fast.

---

# 4. Package A1 — Eliminate the resolver's 148-interface rebuild

This should be one of the first implementation patches because the fix is conceptually straightforward.

Current bad boundary:

```text
ModuleResolver
    ↓
load_interface(Universe)
    ↓
BuiltinInterfaceBuilder::build_from_parsed
    ↓
fresh Universe root
    ↓
UniverseSourceDeclarationCatalog::build
    ↓
InterfaceBuilder × all Universe modules
```

Instead:

```text
ModuleResolver
    ↓
CanonicalUniverseCompilationImage.interfaces
CanonicalUniverseCompilationImage.declaration_catalog
```

### Requirements

`ModuleResolver` must not privately reconstruct canonical Universe interfaces.

Its Universe path should consume the canonical provider/cache:

```rust
if module_id.project == ProjectIdentity::Universe {
    return canonical.interfaces.get(module_id);
}
```

or the equivalent source-provider API.

`UniverseSourceDeclarationCatalog` must become process-shared/canonical and constructed once.

### Structural acceptance gate

After canonical warm-up, one ordinary inline compile must report:

```text
UniverseSourceDeclarationCatalog::build        0
canonical Universe InterfaceBuilder::build     0
```

Anything else means the ownership boundary is still wrong.

---

# 5. Package A2 — Stop routing one-shot compilation through `SemanticWorkspaceInput::new`

The current convenience constructor is a compatibility path. It reconstructs interfaces for topology, discards them, and forces downstream reconstruction.

That API should not be the production path for `ProgramCompiler::compile_entry_selection`.

Instead one-shot compilation should internally create an ephemeral **real module session**:

```text
Canonical Universe image
        ↓
WorkspaceModuleSession / equivalent
        ↓
insert synthetic user module
        ↓
exact interface/import/topology products
        ↓
SemanticWorkspaceSession
```

The externally visible operation can remain one-shot:

```rust
ProgramCompiler::compile_entry_selection(...)
```

but internally it should use the same canonical/persistent architecture as `ProgramCompilationSession`.

This is important:

> **The optimization must live underneath the one-shot API, not only inside the test harness.**

Changing `run_inline()` to share a session would hide the defect rather than repair `ProgramCompiler`.

---

# 6. Preserve interfaces/import products all the way through the pipeline

Once a product has been computed, downstream APIs must accept it directly.

Current pattern:

```text
build interface
    ↓
derive topology
    ↓
discard interface
    ↓
query rebuilds interface
    ↓
snapshot rebuilds interface
```

Required pattern:

```text
build/fetch interface once
         │
         ├─ topology
         ├─ semantic DB
         └─ snapshot/module product
```

This suggests an input model closer to:

```rust
pub struct SemanticWorkspaceInput {
    pub sources: Arc<ModuleSources>,
    pub interfaces: Arc<ModuleInterfaces>,
    pub import_products: Arc<ImportProducts>,
    pub topology: Arc<ModuleTopology>,
    pub reverse_imports: Arc<ReverseImportIndex>,
    pub delta: WorkspaceDelta,
    ...
}
```

No downstream layer should use raw AST source as a reason to reconstruct an interface that is already present.

---

# 7. Package A3 — Treat the canonical Universe as the previous semantic revision

This is one of the most important conceptual changes.

Today a one-shot session effectively sees:

```text
previous snapshot = None
```

and therefore concludes:

```text
everything currently visible is new
```

That is semantically false.

The user module is new.

The canonical Universe is not.

The correct semantic ancestry is:

```text
Revision U:
    canonical Universe

Revision U+1:
    canonical Universe
    + synthetic user module
```

not:

```text
Revision 0:
    40 Universe source modules
    + user module
    all appearing simultaneously for the first time
```

This single change should eliminate many cold-global decisions.

### Required invariant

For a new standalone user compilation:

```text
changed canonical modules = 0
added user modules        = 1
```

The semantic engine must receive that fact directly.

---

# 8. Expand the canonical baseline to include currently reconstructed evidence

The existing baseline is useful but incomplete.

At minimum investigate moving the following into canonical immutable state:

| Product | Current problem | Required state |
|---|---|---|
| declarations | already canonical | retain |
| hierarchy | canonical | retain |
| callable signatures | canonical | retain |
| dispatch surfaces | canonical | retain |
| algebraic products | canonical | retain |
| **conformance index** | rebuilt | canonical |
| **witness plans** | rebuilt | canonical |
| **trait dispatch index** | rebuilt | canonical |
| **derived conformance rules** | rebuilt | canonical |
| **generic proof roots** | session-local | reusable immutable roots |
| **Universe callable-body products** | repeatedly analyzed | canonical where semantics permits |
| **formal module projection shards** | reconstructed | canonical/lazy |
| **source presentation shards** | reconstructed | canonical/lazy |

The goal is **not** to serialize every conceivable semantic object.

The goal is to ensure unchanged source-authored Universe semantics are represented as immutable dependencies instead of source that has to re-enter semantic analysis.

---

# 9. Package A4 — Cross-session query reuse needs explicit architecture

A fresh `SemanticDb` currently has no previous query graph. That defeats a large portion of the incremental design.

You do not necessarily need to make the entire user `SemanticDb` process-global.

A cleaner design is:

```text
CanonicalSemanticQueryImage
    immutable query products
    immutable dependency roots/fingerprints

User SemanticDb
    overlay rows
    user dependency graph
    references canonical roots
```

Think of canonical queries as **external green inputs**.

A user query depending on:

```text
Universe::Iterator signature
```

should not need to republish/recompute that signature query.

It should depend on:

```text
CanonicalQueryRoot(IteratorSignature, fingerprint=X)
```

whose value and fingerprint already exist.

Rust's red-green architecture is useful precedent here: unchanged dependency nodes can be treated as valid without executing the associated query body again. [Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation.html?utm_source=chatgpt.com)

### Do not simply clone the canonical `SemanticDb`

That risks:

- mutable query state leaking across compilations;
- wrong module revision ownership;
- user-specific dependency edges becoming global;
- accidental retention of user ASTs.

The right abstraction is an **immutable base + mutable overlay**.

---

# 10. Package B — Introduce explicit compilation consumer/product profiles

This is arguably the second-biggest architectural requirement.

Currently compilation eagerly publishes products for several different consumers:

```text
runtime execution
compiler lowering
diagnostics
editor/source navigation
advisory analysis
reflection metadata
```

They should not be one mandatory pipeline.

Introduce an explicit capability/product request.

For example:

```rust
pub struct CompilationProductRequest {
    pub executable: bool,
    pub diagnostics: DiagnosticLevel,
    pub reflection: MetadataProfile,
    pub presentation: PresentationLevel,
    pub advisory: AdvisoryLevel,
}
```

Possible presets:

```rust
CompilationProductRequest::runtime_minimal()
CompilationProductRequest::runtime_public()
CompilationProductRequest::check()
CompilationProductRequest::tooling()
CompilationProductRequest::proof()
```

Then each downstream product becomes demand-driven.

---

# 11. Runtime compilation must not build source presentation for the whole Universe

The current cold source index expands to approximately:

```text
148 Universe modules + user fixture
```

That is unacceptable for ordinary execution compilation.

Split the concept.

## 11.1 Core semantic source data

Only data actually required for:

- semantic correctness;
- runtime lowering;
- diagnostics explicitly requested.

For user/runtime modules only.

Something like:

```rust
CoreSourceSemanticIndex
```

## 11.2 Presentation index

For:

- go-to-definition;
- references;
- semantic source sites;
- IDE navigation;
- tooling/source rendering.

Something like:

```rust
PresentationSourceIndex
```

Canonical Universe presentation shards can be precomputed once and shared by tooling consumers.

### Required behavior

`runtime_minimal` compilation:

```text
presentation Universe rebuild count = 0
```

`tooling` compilation:

```text
canonical presentation shards reused
user presentation shards built incrementally
```

---

# 12. Collapse the three type-target AST traversals

Current APIs independently traverse the same AST for:

```text
type references
type-member projections
type-parameter source targets
```

Yet the collector already produces all three.

Replace them with one product:

```rust
pub struct ResolvedTypeSourceTargets {
    pub nominal:
        BTreeMap<SourceRange, DeclarationId>,

    pub member_projections:
        BTreeMap<SourceRange, TypeMemberDeclarationId>,

    pub type_parameters:
        TypeParameterSourceTargets,
}

pub fn resolve_type_source_targets(
    ...
) -> ResolvedTypeSourceTargets;
```

Then make all consumers read from that result.

### Acceptance gate

For each rebuilt module:

```text
TypeReferenceTargetCollector full-program executions = 1
```

not three, not `3 + impl_count`.

---

# 13. Never rescan an entire module once per conformance

The source-index conformance path currently invokes full-program type resolution inside a per-`impl` loop.

Fix this structurally:

```rust
let targets =
    resolve_type_source_targets(module, program, resolver);

for impl_def in conformances {
    let trait_decl =
        targets.nominal.get(&impl_def.trait_ref.range);
}
```

The per-conformance operation should be indexed lookup.

Expected complexity:

```text
old:
O(I × P)

new:
O(P + I log T)
```

or effectively `O(P + I)` with an appropriate hash/index representation.

---

# 14. Add source-site/range indexes instead of repeated scans

The presentation/advisory paths repeatedly do:

```text
all expression sites
→ find SourceSiteId

all occurrences
→ filter by SourceRange
```

Build indexes once:

```rust
pub struct ModuleSourceLookupIndex {
    by_site_id: HashMap<SourceSiteId, SourceSite>,
    occurrences_by_range:
        BTreeMap<SourceRange, SmallVec<[OccurrenceId; 2]>>,
    expression_sites_by_range:
        BTreeMap<SourceRange, SmallVec<[SourceSiteId; 2]>>,
}
```

Then formal attachment and advisory lookup become logarithmic/direct.

This removes the observed:

```text
O(F × E)
O(F × O)
O(E × O)
```

patterns.

---

# 15. Package B2 — Advisory analysis must leave the mandatory runtime path

This should be treated as a consumer-boundary issue before optimizing the solver.

The investigation found advisory analysis running a whole-workspace fixed point over the 41 semantic modules after formal semantic checking.

The first implementation question must be:

> Does any executable correctness or runtime lowering result actually require `AdvisoryWorkspace`?

The current source investigation has not identified such a dependency.

If none exists, executable compilation should request:

```rust
AdvisoryLevel::None
```

and advisory production should be used for:

- IDE/tooling;
- rich diagnostics;
- explicit analysis APIs.

### If diagnostics require some advisory information

Do not re-enable the entire workspace fixed point.

Define narrower levels:

```text
None
LocalDiagnostics
WorkspaceDiagnostics
ToolingComplete
```

and compute only the requested product.

---

# 16. If advisory remains, make it incrementally module-indexed

Independently of demand-driven publication, fix these structural problems:

### Pre-group callables

Not:

```rust
for module in modules {
    all_analyses.iter().filter(module)
}
```

but:

```rust
analyses_by_module: BTreeMap<ModuleId, Arc<[CallableAnalysis]>>
```

### Recompute only affected SCCs

The advisory dependency solver should consume a change set rather than seed every callable each update.

### Retain summaries

Per-callable advisory summaries should be immutable/query-owned products.

### Separate module top-level from callable fixed point

Do not rerun module-level flow if only one callable summary changes unless an explicit dependency says it must.

---

# 17. Package B3 — Formal projection should be lazy or consumer-specific

Current architecture builds the source-aware formal projection and then the snapshot constructor builds another full projection that is immediately overwritten.

First, eliminate the redundant build unconditionally.

Change:

```rust
SemanticSnapshot::new_with_callable_analyses(...)
```

so it does **not** derive expensive products internally.

Prefer:

```rust
SemanticSnapshot::new(SemanticSnapshotParts {
    callable_analyses,
    formal_projection,
    source_index,
    ...
})
```

A snapshot constructor should freeze products, not secretly reconstruct them.

Second, determine whether executable compilation needs `FormalSemanticProjection`.

If it is only a presentation/query projection, request it only in tooling modes.

If runtime lowering requires some formal data, create the minimal runtime-facing form rather than the complete presentation projection.

### Hard gate

A product must never be:

```text
construct
→ store temporarily
→ immediately replace
```

inside successful production compilation.

---

# 18. Package C — Fix callable-body staging instead of accepting systematic reanalysis

One of the strongest semantic findings was:

```text
body analysis with module_values=None
        ↓
later module environment exists
        ↓
fingerprint changes
        ↓
body analyzed again
```

That systematic second analysis needs redesign.

There are two architecturally defensible approaches.

## Option 1 — Establish the module-value environment before callable-body checking

Preferred where possible.

Ordering becomes:

```text
declarations/signatures
        ↓
module binding/export environment
        ↓
callable body analysis
```

Then each body sees its final environment on its first authoritative analysis.

## Option 2 — Split body analysis into dependency-sensitive queries

If module-value facts genuinely require earlier body information, separate stable work:

```text
BodyStructure
BodyNameResolution
BodyTypeAnalysis
BodyModuleValueResolution
```

Only the portion that actually depends on module-value bindings should be revisited.

Do **not** include:

```text
has_module_value_environment: bool
```

as a blanket fingerprint field that invalidates every eligible body regardless of whether it ever reads a module value.

### Required correctness invariant

A body that does not read module values must have an identical reusable semantic result before and after unrelated module-value publication.

---

# 19. Package C2 — Make inferred-result propagation SCC-driven

The current iterative process can repeatedly reanalyze unknown-return callables along dependency chains.

Build an explicit callable-result dependency graph:

```text
CallableId → callables whose public/inferred result affects it
```

Then:

1. compute SCCs;
2. topologically process acyclic components once;
3. fixed-point only cyclic SCCs;
4. publish changed contract fingerprints;
5. invalidate only dependent SCCs whose observable contract changed.

A callable being *recomputed* is not sufficient reason to consider its result *changed*.

This distinction is fundamental.

Required condition:

```rust
if new_public_result_fingerprint == old_public_result_fingerprint {
    do_not_propagate_change();
}
```

Rust's incremental model emphasizes this exact property: a recomputed query whose result is unchanged can remain effectively green for dependents. [Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation.html?utm_source=chatgpt.com)

---

# 20. Package C3 — Conformance witness lookup needs real indexes

The current witness planner performs, per requirement:

```rust
callable_signatures.iter().find_map(...)
```

over a global callable table.

Create:

```rust
pub struct ConformanceWitnessIndex {
    explicit:
        HashMap<
            (ImplId, CallableSide, Selector),
            CallableId
        >
}
```

Build/update it once when conformance-owned signatures are published.

Then explicit witness lookup becomes direct.

### Required complexity change

From:

\[
O(S \times \sum R_i)
\]

to:

\[
O(\sum R_i)
\]

for explicit witness retrieval, aside from type compatibility checking.

---

# 21. Cache conformance requirement instantiation

The planner currently instantiates requirement views multiple times during provisional/final planning.

Represent normalized instantiated requirements as a stable product:

```rust
ConformanceRequirementView {
    impl_id,
    trait_ref,
    target,
    requirements: Arc<[InstantiatedRequirement]>
}
```

Fingerprint it from:

```text
trait surface
target head
trait arguments
callable result contracts relevant to defaults
```

Then witness resolution and witness-plan production consume the same product.

Do not rematerialize generic substitutions multiple times in one revision.

---

# 22. Separate structural witness plans from return-sensitive completion

The two whole witness-plan passes are semantically understandable but too broad.

Refactor into:

```text
StructuralConformancePlan
    explicit witness ownership
    default candidate ownership
    requirement identities
    generic substitutions

FinalWitnessTyping
    specialized signatures
    inferred return compatibility
```

Only the second portion should depend on inferred callable returns.

Then:

```text
provisional full plan
→ final full plan
```

becomes:

```text
stable structural plan once
→ finalize changed requirements after return settlement
```

This is a far better incremental boundary.

---

# 23. Use exact provisional→final plan changes

The final planner already knows which provisional plans actually changed.

Do not discard that set and later compare against an empty previous snapshot, thereby making all cold plans appear changed.

The update should propagate:

```rust
let changed_final_plans =
    finalize_conformance_plans(...);
```

directly into dependency invalidation.

For a canonical baseline + user overlay:

```text
unchanged canonical witness plans = green
new/affected user witness plans   = changed
```

---

# 24. Package C4 — Index `ConformanceIndex` for its actual queries

Current exact proof can linearly scan the conformance index.

At minimum maintain:

```rust
ConformanceIndex {
    by_impl: ...,

    by_trait:
        HashMap<TraitDeclarationId, ...>,

    by_trait_and_family:
        HashMap<
            (TraitDeclarationId, TypeFamilyKey),
            ...
        >,

    exact_heads:
        HashMap<CanonicalConformanceHead, ...>,
}
```

Then:

```text
query_exact
overlap candidates
dispatch candidates
```

must begin from a narrow bucket.

### Coherence checking

Instead of comparing all eligible conformances pairwise:

```text
all conformances × all later conformances
```

partition first by:

```text
trait declaration
target family/head discriminator
```

and run unification only within candidate buckets.

---

# 25. Make generic conformance proof an actual memoized semantic query

Currently proof contexts memoize only during an individual proof invocation.

Promote:

```rust
GenericConformanceKey
```

to the true semantic query key.

Callers should ask:

```rust
db.generic_conformance(key)
```

not:

```text
compute proof
then record that proof as a dependency
```

The desired semantics are:

```text
expression 1 requests proof X
    → compute X

expression 2 requests proof X
    → reuse X

second body-analysis pass requests proof X
    → reuse X if dependencies unchanged
```

This is exactly the kind of query-level memoization mature compilers use. Rust's trait solver likewise relies on caching goal evaluation because repeated proof search is intrinsically expensive. [Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/solve/caching.html?utm_source=chatgpt.com)

---

# 26. Package D — Implement metadata profiles for real

This is not optional technical debt. Current behavior does not match the architectural intent of `MetadataProfile`.

Today `RuntimePublic` effectively behaves as:

```text
label bundle RuntimePublic
+
walk broad declaration/callable universe
```

The correct design is:

```text
profile
   ↓
root selection
   ↓
reachability closure
   ↓
semantic records
```

Implement:

```rust
pub trait MetadataRetentionPolicy {
    fn declaration_is_root(&self, ...);
    fn callable_is_root(&self, ...);
    fn alias_is_root(&self, ...);
    fn retain_dependency(&self, ...);
}
```

or an equivalent explicit export planner.

---

# 27. Define metadata roots before emitting records

For example:

### `RuntimeMinimal`

Roots:

- runtime type constants used by executable code;
- callable ABI contracts needed by execution;
- validators;
- runtime specialization/conformance data;
- required stable IDs.

### `RuntimePublic`

Adds:

- public user declarations;
- public callable signatures;
- publicly reflectable aliases;
- public superclass/type information.

### `ToolingDebug`

Adds source/tooling products.

Then compute semantic reachability.

Do not iterate every declaration merely because it exists in `SemanticSnapshot`.

---

# 28. Add owner indexes for metadata

Even after root filtering, remove:

```text
for each declaration:
    scan every callable
```

Extend `CallableSignatureTable` with:

```rust
callables_by_declaration:
    HashMap<DeclarationId, BTreeSet<CallableId>>

callables_by_impl:
    HashMap<ImplId, BTreeSet<CallableId>>
```

or expose a generalized owner index:

```rust
HashMap<CallableOwnerId, Arc<[CallableId]>>
```

This will also help other compiler phases currently filtering global tables.

---

# 29. Reuse canonical Universe metadata rather than exporting it into every program

The VM already has canonical Universe metadata installed independently.

The correct long-term model is:

```text
metadata pool 0:
    canonical Universe metadata

metadata pool N:
    program-local metadata
    + references to pool 0
```

This requires an explicit cross-pool reference format.

Do **not** just delete Universe records and assume IDs remain valid.

Required design work includes:

- stable external metadata references;
- pool-qualified type/declaration handles;
- serialization representation;
- runtime lookup semantics;
- lifecycle/GC ownership;
- compatibility/version checks.

Once implemented, a tiny program's metadata bundle should contain only its local closure plus references to canonical metadata.

---

# 30. Package E — Fix global filtered scans with module/owner indexes

Several backend loops are:

```rust
global_table.iter()
    .filter(|x| x.module == runtime_module)
```

Create persistent module indexes in the owning tables.

Examples:

```text
ModuleId → callable analyses
ModuleId → callable definitions
ModuleId → data declarations
ModuleId → enum declarations
ModuleId → source attachments
```

Then runtime lowering of one fixture should scale with the fixture.

These are not Tier-1 fixes before the lifecycle work, but they are architecturally clean and useful for the 92-KB source-dependent portion.

---

# 31. Cache global trait requirement-slot maps

Do not rebuild:

```text
trait requirement → slot
```

per trait invocation.

This should be an immutable semantic product:

```rust
TraitRequirementSlotIndex
```

owned by the snapshot/canonical semantic image and reused by lowering.

---

# 32. Index inherent implementation domains

Replace:

```text
scan every conditional member
→ find matching ImplId
```

with:

```rust
ImplId → InherentImplDomain
```

This is a straightforward ownership/index correction.

---

# 33. Build profiles are a diagnostic/operational fix, not the architecture fix

The project should introduce a dedicated performance-test profile, for example:

```toml
[profile.perf-test]
inherits = "test"
opt-level = 2
debug = 0
```

or an equivalent CI strategy.

Why:

- correctness tests can remain `opt-level=0`;
- compiler runtime performance gates should not benchmark unoptimized compiler code;
- release/runtime expectations need an optimized executable.

But **do not claim success because 45 seconds becomes 10 seconds after enabling optimization**.

The structural acceptance criterion is that compilation stops doing irrelevant Universe-scale work.

---

# 34. Snapshot architecture requirement: constructors must not derive expensive products

`SemanticSnapshot` should become a pure immutable assembly point.

Bad:

```rust
SemanticSnapshot::new(...)
    ├─ compute formal projection
    ├─ scan internal incidents
    └─ derive assorted products

snapshot.with_formal_projection(actual_projection)
```

Preferred:

```rust
SemanticSnapshot::from_parts(
    SemanticSnapshotParts {
        declarations,
        signatures,
        analyses,
        source_index,
        formal_projection,
        advisory,
        ...
    }
)
```

Each product should have a clear producer/query.

This matters not only for performance but for dependency ownership and testability.

---

# 35. Full-map publication should become persistent/sharded

The existing `snapshot_full_map_materializations += 8` is a warning that product ownership remains partly monolithic.

For every root classify it as one of:

```text
canonical immutable
persistent module map
changed-module shard
small scalar/index
truly whole-workspace derived root
```

Prefer:

```rust
im::OrdMap<ModuleId, Arc<ModuleShard>>
```

or another persistent indexed structure for large module-partitionable products.

The important invariant is not “use `im` everywhere.”

It is:

> Updating one module must not require materializing every unchanged module merely to publish an immutable snapshot.

---

# 36. Canonical/user identity and aliasing requirements

A deeper implementation plan must explicitly handle these correctness issues.

### Stable IDs

Canonical:

```text
ModuleId
DeclarationId
CallableId
TypeId/type-family identity
ImplId
```

must preserve stable identity when referenced from user overlays.

### TypeStore lineage

User type-store forks must preserve canonical prefix identity.

The existing persistent `TypeStore` design appears suitable; do not regress to copying.

### Snapshot isolation

Two user compilations must not see each other's:

- user types;
- aliases;
- callable analyses;
- query state;
- diagnostics;
- source indexes.

### Canonical Arc ownership

Canonical products may be process-shared indefinitely, but must never contain user ASTs or user-module references.

---

# 37. Exact required invariants

Every implementation package should be gated against these.

## Semantic parity

One-shot and retained-session compilation of the same source must produce equivalent:

- diagnostics;
- callable signatures;
- type results;
- conformance witnesses;
- compiled behavior;
- metadata according to requested profile.

## Determinism

Warm reuse must not change:

- diagnostic ordering;
- stable semantic IDs;
- metadata fingerprints;
- emitted bytecode.

## Error isolation

A failed user compilation must not corrupt canonical state.

## Cancellation

A cancelled user update must leave the reusable baseline valid.

## Concurrency

If canonical images are process-shared, simultaneous compilations must be read-only with respect to them.

---

# 38. Structural performance acceptance gates

I would make these mandatory because wall-clock gates alone do not prove architectural repair.

For a warmed tiny one-shot fixture:

| Counter | Required result |
|---|---:|
| canonical Universe source reparses | **0** |
| canonical Universe interface rebuilds | **0** |
| `UniverseSourceDeclarationCatalog::build` | **0** |
| deep semantic updates for canonical Universe modules | **0** |
| body analyses for unchanged canonical Universe callables | **0** |
| source-index presentation modules rebuilt | **0** in runtime-minimal mode |
| advisory workspace iterations | **0** in runtime-minimal mode |
| redundant snapshot formal projection | **0** |
| metadata full-world declaration sweep | **0** |
| metadata declaration × global-callable scans | **0** |
| user interface build | ideally **1** |
| full type-target collector runs/user module | **1** |
| canonical conformance proofs recomputed | **0** unless genuinely input-sensitive |

These counters are substantially more meaningful than “compile became faster.”

---

# 39. Behavioral performance gates

Once instrumentation exists, establish gates relative to the current measured baseline.

I would not initially hard-code a final millisecond target, because we do not yet have optimized timings.

But require:

### Gate 1 — Eliminate the fixed Universe floor

For warm same-process compilation:

```text
345-byte source
485-byte source
7.5-KB source
```

should no longer all cluster near one large Universe-dominated constant.

Compilation should start correlating primarily with user/affected source.

### Gate 2 — One-shot versus retained session

After canonical warm-up, first user one-shot compile and first update in `ProgramCompilationSession` should have the **same fundamental product frontier**.

Persistent sessions can still win by reusing previous user revisions.

But one-shot should no longer be catastrophically slower because it reconstructs the canonical Universe.

### Gate 3 — Optimized compiler runtime

Measure both:

```text
test/O0 diagnostic profile
optimized performance profile
```

Architecture must pass structural counters under both.

---

# 40. Implementation sequencing

I recommend this order.

## Phase 1 — Remove catastrophic lifecycle reconstruction

Implement together or in tightly coordinated plans:

1. canonical `UniverseSourceDeclarationCatalog`;
2. resolver canonical-interface reuse;
3. one-shot compiler through the real module-session input architecture;
4. preserve interfaces/import products/topology rather than rebuilding them;
5. canonical semantic state as the baseline revision.

This is likely the largest immediate win with comparatively low semantic-risk versus the deeper solver changes.

---

## Phase 2 — Separate execution from presentation/advisory products

Implement:

1. `CompilationProductRequest`;
2. runtime-minimal source index;
3. no canonical presentation reconstruction for executable compilation;
4. lazy/advisory/tooling products;
5. eliminate redundant formal projection.

This should collapse another major chunk of the fixed floor.

---

## Phase 3 — Conformance and callable semantic settlement

Implement:

1. conformance witness indexes;
2. indexed conformance lookup;
3. memoized `GenericConformanceKey` queries;
4. cached instantiated requirement products;
5. structural/final witness-plan separation;
6. SCC-oriented inferred-return settlement;
7. eliminate unconditional module-value second body analysis.

This is the highest semantic-risk package and deserves very strong parity tests.

---

## Phase 4 — Root-driven metadata

Implement:

1. actual metadata-profile retention;
2. root selection;
3. reachability;
4. owner indexes;
5. canonical metadata external references.

This can be implemented after semantic lifecycle improvements because it has a relatively self-contained acceptance surface.

---

## Phase 5 — Remaining local complexity

Then eliminate:

- global module-filtering scans;
- repeated trait-slot construction;
- inherent-impl scans;
- range/occurrence scans;
- structural-fingerprint temporary strings;
- avoidable BTreeMap rebuilding;
- unnecessary cloning.

At this stage these optimizations will affect genuinely necessary work rather than polishing globally redundant work.

---

# 41. What **not** to combine

I would specifically keep these separate review units:

### Do not combine metadata redesign with semantic-query redesign

Both touch IDs/fingerprints and a bug would be hard to localize.

### Do not combine advisory removal with conformance changes

If diagnostics move unexpectedly, you want to know whether advisory product selection or formal semantics caused it.

### Do not combine test-profile changes with performance acceptance patches

Otherwise improvements become impossible to attribute.

### Do not “solve” one-shot performance by replacing `run_inline` alone

That is a harness optimization, not a compiler fix.

---

# 42. Tests required before optimization patches are accepted

The implementation needs more than benchmarks.

### One-shot/persistent equivalence

For representative fixtures:

```text
compile_entry_selection(source)
```

versus:

```text
ProgramCompilationSession::compile_fixture(source)
```

must agree semantically.

### Canonical isolation

Compile:

```text
A
B
A
```

in the same process and prove the two `A` results are equivalent and not contaminated by `B`.

### Failure recovery

```text
valid A
invalid B
valid C
```

must leave canonical/session state correct.

### Conformance stress

Cover:

- explicit witnesses;
- defaults;
- conditional conformances;
- associated types;
- inferred witness returns;
- overlapping generic heads;
- recursive generic evidence.

### Metadata parity

For every `MetadataProfile`, compare old/new observable reflection semantics where the old behavior was intended.

### Presentation isolation

Runtime-minimal compilation must work without creating presentation/advisory products.

### Incremental invalidation

Modify exactly one:

- callable body;
- callable signature;
- conformance;
- alias;
- module import;
- top-level value.

Assert which products are recomputed.

---

# 43. Observability must become an architectural regression guard

`telos-observability` should not merely measure durations.

It should record **work cardinality**, because regression can hide behind faster hardware.

Persist metrics such as:

```text
canonical_interfaces_rebuilt
semantic_modules_recomputed
callable_body_analyses
callable_body_analyses_by_reason

source_index_module_rebuilds
source_ast_full_traversals

conformance_requirement_instantiations
witness_signature_probes
exact_conformance_proof_calls
exact_conformance_unique_keys

formal_projection_modules_rebuilt
formal_projection_sites

metadata_declarations_visited
metadata_callables_visited
metadata_owner_probes

advisory_outer_iterations
advisory_body_flows
```

Then CI can enforce architectural expectations.

Example:

```text
tiny_runtime_fixture:
    canonical_interfaces_rebuilt == 0
    canonical_body_analyses == 0
    advisory_outer_iterations == 0
    metadata_world_sweep == 0
```

That is far more robust than asserting `< 500 ms`.

---

# 44. The deepest architectural correction

The source investigation points to one fundamental mistake repeated in multiple forms:

> **The compiler treats previously derived canonical products as if downstream consumers primarily own raw source and should be free to reconstruct whatever derived representation they need.**

That produces boundaries like:

```text
cached parsed source
→ rebuild interface

semantic tables
→ rebuild ownership projection

callable analyses
→ rebuild formal projection

source AST
→ independently rebuild three target maps

canonical semantic world
→ republish into empty query database

canonical metadata
→ export semantic Universe into user pool
```

The better invariant is:

> **A derived product has one authoritative producer, an explicit identity/fingerprint, a defined lifetime, and downstream consumers consume that product rather than reconstructing it from a lower-level representation.**

That principle should guide the implementation plans.

---

# 45. Target end-state

The healthy warm one-shot path should eventually look roughly like:

```text
ProgramCompiler::compile_entry_selection(Inline)
│
├─ fetch CanonicalUniverseCompilationImage
│      O(1) shared roots
│
├─ parse user fixture
├─ build user interface
├─ resolve user imports against canonical module products
├─ create one-module overlay
│
├─ semantic update
│   ├─ canonical Universe = immutable green dependency
│   ├─ analyze affected user declarations/signatures
│   ├─ analyze affected user bodies
│   ├─ solve only new/affected conformance goals
│   └─ publish user shards
│
├─ requested products
│   ├─ runtime-minimal source semantics
│   ├─ no advisory unless requested
│   ├─ no global presentation unless requested
│   └─ no redundant formal projection
│
├─ metadata export
│   ├─ user/profile roots
│   ├─ reachable closure
│   └─ references canonical Universe metadata
│
└─ lower runtime user module
```

That architecture makes the desired complexity obvious:

\[
T_{\text{warm one-shot}}
\approx
T_{\text{user source}}
+
T_{\text{affected semantic closure}}
+
T_{\text{requested outputs}}
\]

instead of:

\[
T_{\text{warm one-shot}}
\approx
T_{\text{Universe}}
+
T_{\text{presentation Universe}}
+
T_{\text{semantic Universe}}
+
T_{\text{metadata Universe}}
+
T_{\text{user}}
\]

That is the standard I would use for the implementation program.

### Bottom line

The highest-leverage implementation should start with **canonical Universe ownership + one-shot overlay compilation**, immediately followed by **consumer-specific publication**. After those boundaries are repaired, optimize conformance/body settlement and metadata reachability. The dozens of smaller `N×M` scans should be fixed, but they should not lead the work: the dominant architectural defect is that one-shot compilation repeatedly reconstructs products that should already exist as immutable canonical dependencies.

If you want, the next step should be to turn this requirements analysis into **several patch-grade implementation plans with explicit dependencies and acceptance gates**, rather than one monolithic performance plan.