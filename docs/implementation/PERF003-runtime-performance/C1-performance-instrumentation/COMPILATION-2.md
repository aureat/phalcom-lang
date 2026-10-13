# Investigation Slice 9 — Canonical Universe as an Immutable Dependency Image

This slice exposes what is probably the **clearest ownership/lifetime defect found in the investigation**:

> **The process already computes a complete semantic analysis of the entire canonical Universe once while constructing `CanonicalUniverseProgram`, uses that snapshot to compile the canonical program, and then discards the semantic snapshot. Immediately afterward, ordinary one-shot fixture compilation reconstructs a substantial subset of the same Universe semantics from source.**

That is a much stronger finding than merely saying the existing semantic baseline is incomplete.

The measured behavior aligns directly with it: the first canonical compiler product costs about **81.27 s**, after which the process-shared product is warm; separate tiny-fixture compilation still sits around **45 s**. Pasted text Pasted text

---

## 1. H01 — The existing canonical semantic baseline is substantial, but deliberately partial

### `SOURCE-CONFIRMED`

`CanonicalUniverseSemanticBaseline` currently retains:

```rust
pub struct CanonicalUniverseSemanticBaseline {
    provider: UniverseSourceProvider,
    linked: Arc<LinkedProgram>,
    store: TypeStore,

    base_declarations: DeclarationTypeTable,
    base_hierarchy: MapTypeHierarchy,
    base_dispatch: SurfaceDispatchResolver,
    base_callable_signatures: CallableSignatureTable,

    base_data_semantics: DataSemanticTable,
    base_data_products: Vec<Arc<DataDeclarationProduct>>,

    base_enum_semantics: EnumSemanticTable,
    base_enum_products: Vec<Arc<EnumDeclarationProduct>>,

    base_associated_surfaces: AssociatedFamilyTable,
    base_associated_surface_products: Vec<Arc<AssociatedSurface>>,

    base_type_member_declarations: BTreeMap<...>,
    base_type_member_products: Vec<Arc<...>>,

    base_enum_requirements: EnumRequirementTable,
    base_enum_requirement_products: Vec<...>,

    canonical_alias_sources: BTreeMap<...>,
    canonical_native_diagnostics: BTreeMap<...>,
}
```

Its build is nontrivial. It already walks canonical source to establish source-authored:

| Product | Baseline |
|---|---:|
| nominal declarations | yes |
| hierarchy/supertype templates | yes |
| declaration surfaces | yes |
| callable signatures | yes |
| inherent implementation contributions | yes |
| data semantics | yes |
| enum semantics/requirements | yes |
| associated surfaces | yes |
| type members | yes |
| aliases/native diagnostics | yes |
| trait headers | **no** |
| trait behavioral surfaces | **no** |
| conformance index | **no** |
| conformance witness plans | **no** |
| derived conformance rules | **no** |
| trait-dispatch index | **no** |
| field signatures | **no complete baseline** |
| constructor paths | **no** |
| callable body analyses | **no** |
| generic proof query cache | **no** |
| semantic query graph | **no** |
| static semantic products | **incomplete** |

This explains why it reduced some startup work without eliminating the 45-second floor.

---

## 2. H02 — Every session still begins with an empty semantic database

### `SOURCE-CONFIRMED`

Session creation now correctly uses the process-shared baseline:

```rust
pub fn with_workspace(
    workspace: WorkspaceId
) -> Self {
    canonical_universe_semantic_baseline()
        .instantiate(workspace)
}
```

But instantiation does:

```rust
let db =
    SemanticDb::with_workspace(workspace);

let store =
    Arc::new(
        baseline.store.fork_from_baseline()
    );
```

`SemanticDb::with_workspace()` creates:

```rust
query_states: BTreeMap::new(),
query_keys_by_module: BTreeMap::new(),
products: BTreeMap::new(),
last_known_good: BTreeMap::new(),
index: DependencyIndex::new(),
scheduler: QueryScheduler::new(),
...
```

So the distinction is:

```text
semantic values                 process-shared partly
semantic query knowledge        not shared
dependency graph                not shared
proof query results             not shared
trait/conformance products      not shared
body products                   not shared
```

That is why session creation can start from canonical tables and still perform huge amounts of canonical query work.

---

## 3. H03 — Even baselined products are republished into every fresh DB

### `SOURCE-CONFIRMED`

Fresh semantic publication does things like:

```rust
for (declaration, info)
    in self.base_declarations.iter()
{
    query_declaration_shell(...);
}
```

followed by:

```rust
for (declaration, _)
    in self.base_declarations.iter()
{
    query_bootstrap_declaration_surface(...);
}
```

then:

```rust
for (_, signature)
    in self.base_callable_signatures.iter()
{
    query_bootstrap_callable_signature(...);
}
```

and:

```rust
for (class_decl, _)
    in self.base_declarations.iter()
{
    query_bootstrap_hierarchy_edge(...);
}
```

plus publication of:

```text
base_data_products
base_enum_products
base_associated_surface_products
base_type_member_products
base_enum_requirement_products
```

Several of those base-product loops occur at multiple points in the update.

So current baseline reuse is closer to:

```text
reuse immutable value
→ insert/revalidate it inside every fresh query database
```

than:

```text
reuse already-known immutable query result
```

That is an important distinction.

---

# 4. H04 — The old 40-module source injection exists specifically because the baseline stops too early

### `SOURCE-CONFIRMED`

The inline path documents its rationale:

```rust
// Inline evaluation has an implicit Universe runtime.
// Its semantic workspace must therefore include
// the same source-authored conformance declarations,
// rather than treating native member surfaces as a
// substitute for canonical trait evidence.
```

Standalone/package compilation makes the same point:

```rust
// Include the source-authored collection and scalar
// conformance modules so semantic products such as
// canonical `for` lowering are derived from their
// declared contracts...
```

This is crucial.

The 40 seeded modules are not arbitrary baggage that can simply be deleted.

They are being injected because the partial baseline does not contain enough source-authored formal semantics to answer questions such as:

```text
Iterator / Iterable contracts
exact conformance evidence
witness selection
generic conformance prerequisites
trait defaults
conditional/derived conformance
for-loop lowering evidence
```

Therefore the architectural repair is:

> **Promote those canonical semantic facts into the reusable canonical dependency image.**

Not:

> Remove the modules and hope native surfaces are enough.

---

# 5. H05 — The full semantic image already exists transiently

### `SOURCE-CONFIRMED` — **major finding**

`build_canonical_universe_program()` constructs the complete canonical source graph:

```rust
let linked =
    ModuleLinker::new(
        project_universe.clone(),
        interfaces,
    )
    .link_all(
        ModuleId::universe_root(),
        &resolved,
    )?;
```

It gathers every canonical parsed source:

```rust
let sources =
    source_index
        .units
        .iter()
        .map(|unit|
            (unit.id.clone(), unit.clone()))
        .collect();
```

Then it runs the normal semantic analyzer:

```rust
let semantic =
    phalcom_semantic::analyze_workspace(
        SemanticWorkspaceInput::new(
            linked.clone(),
            sources.clone(),
            0,
        )
    )?;
```

This is **full canonical Universe semantic analysis**.

The resulting snapshot necessarily contains the products missing from the lightweight baseline, including the finalized:

```text
trait headers/surfaces
conformance index
witness plans
derived conformance rules
trait-dispatch products
callable analyses
field/static products
constructor products
diagnostics
```

It is then handed to:

```rust
let analyzed = AnalyzedProgram {
    ...
    semantic: semantic.snapshot,
    ...
};

let program =
    ProgramCompiler::
        compile_analyzed_for_canonical_bootstrap(
            &analyzed
        )?;
```

And after `build_canonical_universe_program()` returns, the semantic snapshot is gone.

---

# 6. `CanonicalUniverseProgram` retains the wrong half of that computation for downstream compilation

Its current shape is:

```rust
pub struct CanonicalUniverseProgram {
    program: Arc<CompiledProgram>,
    source_index: Arc<NativeSourceIndex>,
    root_reachable: Arc<[ModuleId]>,
    bootstrap_order: Arc<[ModuleId]>,
}
```

It retains the **runtime/compiler projection**.

It does not retain the **full formal semantic result** that created that projection.

This makes sense for VM bootstrapping in isolation.

But it is exactly the wrong lifetime boundary when the same process subsequently compiles user code against Universe.

The process has effectively done:

```text
Canonical Universe source
        │
        ▼
full semantic analysis ─────────────┐
        │                           │
        ▼                           │ discarded
canonical executable program        X
        │
        ▼
VM bootstrap

then:

fixture source
        │
        ▼
partial canonical baseline
+
40 Universe source modules
        │
        ▼
redo large subset of
canonical semantics
```

The missing retained arrow is obvious.

---

# 7. H06 — `run_inline()` makes this missed reuse particularly stark

Current test helper order is:

```rust
let mut vm = universe_vm(); // VM::new()

let program =
    ProgramCompiler::compile_entry_selection(
        EntrySelection::Inline(...)
    )?;
```

`VM::new()` begins with:

```rust
let canonical =
    canonical_universe_program()?;
```

and its own comment says:

```rust
// Canonical source/native verification,
// linking, semantic analysis, and lowering
// are process-shared.
```

On first use, this is the expensive canonical product measured at roughly **81 s**. Pasted text

Then `VM::new()` returns.

Only then does:

```rust
compile_entry_selection(Inline)
```

start a new semantic path.

Thus the process order is literally:

```text
compute complete Universe semantics
discard semantic snapshot
finish VM construction
start fixture compilation
reconstruct substantial Universe semantics
```

This is probably the single clearest lifecycle defect discovered so far.

---

# 8. H07 — This is not the same as the existing `CanonicalUniverseSemanticBaseline`

There are effectively **two canonical semantic constructions** today.

### Lightweight semantic baseline

Owned by `phalcom-semantic`:

```text
CanonicalUniverseSemanticBaseline
```

Purpose:

```text
session bootstrap prefix
```

It constructs selected formal products without doing the complete ordinary semantic analysis.

### Full canonical compiler analysis

Owned indirectly by `phalcom-core`:

```text
canonical_universe_program()
    → analyze_workspace(all canonical sources)
```

Purpose:

```text
canonical executable program
```

It computes far richer semantics.

The second is exactly the information downstream sessions want, but it is not exposed as a reusable dependency semantic image.

---

# 9. H08 — There is also duplicate canonical linking ownership

### `SOURCE-CONFIRMED`

The semantic baseline calls:

```rust
canonical_universe_linked_program(&provider)
```

which has its own:

```rust
static CANONICAL_LINKED_UNIVERSE:
    OnceLock<Arc<LinkedProgram>>
```

But `build_canonical_universe_program()` in `phalcom-core` independently:

```rust
let mut interfaces = BTreeMap::new();
...
let mut resolved = BTreeMap::new();
...
let linked =
    ModuleLinker::new(...)
        .link_all(...)?;
```

So the repository maintains two canonical linked-product construction paths across crate boundaries.

Both are process-cached indirectly, but they do not share one canonical linked object.

This is another sign that canonical ownership is split at the wrong abstraction layer.

It is likely not a major recurring 45-second cost because both are process-level operations, but it complicates identity and prevents a clean “one canonical semantic universe” architecture.

---

# 10. H09 — The `TypeStore` design is already suitable for a complete semantic image

### `SOURCE-CONFIRMED`

This is good news.

`TypeStore::fork_from_baseline()` explicitly guarantees:

```rust
/// All node IDs allocated in `self` are preserved
/// with identical numeric values in the fork.
///
/// The fork receives a fresh `TypeStoreId`.
```

Implementation:

```rust
pub fn fork_from_baseline(&self) -> Self {
    let mut fork = self.clone();

    fork.id =
        TypeStoreId(
            NEXT_TYPE_STORE_ID
                .fetch_add(1, Ordering::Relaxed)
        );

    fork
}
```

The main arenas and interners use persistent `im` structures.

Therefore the full canonical semantic analysis can conceptually finish with:

```text
canonical TypeStore prefix
```

and every workspace gets:

```text
canonical prefix
+
workspace-local appended types
```

without renumbering canonical `TypeId`s.

This solves one of the hardest problems an immutable dependency image would otherwise have.

---

# 11. The full image must own the **final** canonical store prefix

This is important.

Current flow is approximately:

```text
partial baseline store
        │
        ▼
fork
        │
        ▼
full canonical semantic analysis
        │
        ├─ allocates trait types
        ├─ conformance-related forms
        ├─ generic products
        ├─ projections
        └─ other canonical semantic nodes
```

If downstream work forks from the *old* partial baseline, those later canonical IDs are not inherently part of its canonical prefix.

The future architecture should instead be:

```text
partial bootstrap construction
        │
        ▼
complete canonical semantic analysis
        │
        ▼
freeze FINAL canonical TypeStore
        │
        ├─ fixture/session fork A
        ├─ fixture/session fork B
        └─ persistent workspace fork C
```

Then all canonical semantic products and all session forks agree on the numeric canonical prefix.

---

# 12. H10 — Query identities are also surprisingly well suited to a canonical base layer

### `SOURCE-CONFIRMED`

`QueryKey` contains semantic identity:

```rust
DeclarationShell(DeclarationId)
DeclarationSurface(DeclarationId)
TraitHeader(DeclarationId)
TraitSurface(DeclarationId)
ConformanceDispatch(ImplId)
GenericConformance(GenericConformanceKey)
CallableDefinition(CallableId)
HierarchyEdge(DeclarationId)
FieldSignature(FieldId)
ConstructorInitialization(DeclarationId)
CallableSignature(CallableId)
CallableBody(CallableId)
...
```

Notably, query keys do **not** include:

```text
WorkspaceId
TypeStoreId
SnapshotId
```

`WorkspaceId` belongs to the containing `SemanticDb`, not to canonical semantic identity.

Likewise product fingerprints are content-oriented. For example callable signature product fingerprinting deliberately excludes source-sensitive material where appropriate.

This is exactly what one would want for:

```text
immutable canonical query base
+
workspace-local mutable query overlay
```

---

# 13. Do not solve this by cloning a complete canonical `SemanticDb`

A tempting implementation would be:

```rust
let db = canonical_db.clone();
db.workspace = workspace;
```

That would be architecturally wrong.

`SemanticDb` contains:

```text
workspace identity
revision
transaction journal
scheduler state
last-known-good state
revision event sets
mutable dependency graph
```

Those are session-local.

The reusable unit should instead resemble:

```rust
struct CanonicalSemanticQueryImage {
    products:
        Arc<BTreeMap<QueryKey, CanonicalQueryProduct>>,

    fingerprints:
        Arc<BTreeMap<QueryKey, ProductFingerprint>>,

    dependencies:
        Arc<CanonicalDependencyIndex>,
}
```

Then:

```rust
struct SemanticDb {
    canonical:
        Arc<CanonicalSemanticQueryImage>,

    workspace: WorkspaceId,
    revision: SemanticRevision,

    local_products: ...,
    local_dependencies: ...,
    local_scheduler: ...,
    ...
}
```

Lookup becomes conceptually:

```text
local product?
    yes → local

otherwise canonical product?
    yes → immutable canonical

otherwise compute
```

---

# 14. Canonical dependencies do not need ordinary reverse invalidation

For production Telos:

```text
canonical Universe image
```

is immutable for the process.

Therefore a workspace query may record:

```text
fixture callable
    depends on
Universe::Iterator requirement
```

with the canonical requirement's product fingerprint.

But there is no need for the workspace to maintain:

```text
Universe requirement
    → every downstream dependent
```

for invalidation caused by ordinary user edits, because the canonical requirement cannot change.

This can keep the local dependency graph considerably smaller.

For development/test scenarios with source-overridden Universe, the repository already distinguishes:

```rust
if provider.is_canonical() {
    shared image
} else {
    build private state
}
```

The same policy can extend to the full image.

---

# 15. H11 — The correct reusable artifact is not a full `SemanticSnapshot`

A `SemanticSnapshot` currently includes workspace-specific and editor-specific material:

```text
SnapshotId
generation
presentation sources
SourceSemanticIndex
FormalSemanticProjection
AdvisoryWorkspace
module query products
incremental structure shards
```

Slice 8 established that many of those should not even exist on an executable-only critical path.

So simply retaining:

```rust
Arc<SemanticSnapshot>
```

inside `CanonicalUniverseProgram` would be an expedient experiment, but not the final architecture.

The correct artifact is closer to:

```rust
pub struct CanonicalUniverseFormalImage {
    pub store: Arc<TypeStore>,

    pub declarations: Arc<...>,
    pub hierarchy: Arc<...>,
    pub dispatch: Arc<...>,

    pub callable_signatures: Arc<...>,
    pub field_signatures: Arc<...>,
    pub constructor_initializations: Arc<...>,

    pub trait_headers: Arc<...>,
    pub trait_surfaces: Arc<...>,

    pub conformance_index: Arc<...>,
    pub conformance_witness_plans: Arc<...>,
    pub derived_conformance_rules: Arc<...>,
    pub trait_dispatch_index: Arc<...>,

    pub data_semantics: Arc<...>,
    pub enum_semantics: Arc<...>,
    pub associated_surfaces: Arc<...>,

    pub static_products: Arc<...>,

    pub canonical_diagnostics: Arc<...>,

    pub query_image:
        Arc<CanonicalSemanticQueryImage>,
}
```

This is a **formal dependency image**, not an IDE snapshot.

---

# 16. Canonical callable bodies require a nuanced policy

The full canonical analysis naturally produces `CallableAnalysis` for Universe bodies.

Should the image retain all of them?

Probably internally, yes, at least initially for exact reuse and validation.

But downstream user compilation should **not iterate the complete Universe body-analysis table by default**.

There are two distinct needs:

```text
canonical-program compilation
    → does need canonical executable body analyses

user-program compilation
    → generally needs finalized canonical signatures,
      conformance evidence, dispatch products,
      generic contracts, etc.
```

Once canonical inferred returns and other body-derived public products have been settled, ordinary fixture analysis should treat them as dependency facts.

This follows the same principle as compiled dependency metadata:

> Consumers need the exported semantic result of dependencies, not repeated body checking of the dependencies.

---

# 17. H12 — This is effectively Telos's equivalent of a compiler dependency metadata image

The closest architecture analogy is not merely “cache the standard library.”

Conceptually the artifact is:

```text
canonical module graph
+
canonical type universe
+
public/internal semantic declarations
+
trait/conformance evidence
+
finalized callable contracts
+
dependency query fingerprints
+
runtime metadata identity
```

That is analogous in purpose to the role played by dependency metadata in mature compilers.

For Telos this artifact can initially remain **in-memory/process-shared**.

Serialization to disk is a separate optimization and should not be mixed into the first correction.

---

# 18. H13 — Table cloning must also be eliminated, or the image will still have an O(Universe) startup floor

Even with a complete image, doing:

```rust
session.declarations =
    image.declarations.clone();
```

is only cheap if the container itself shares structure.

Several current semantic tables use ordinary `HashMap` / `BTreeMap` internally.

A complete image followed by full std-map cloning would turn:

```text
reanalyze Universe
```

into:

```text
copy Universe tables
```

which is much better but still creates a fixed dependency-size startup term.

The stronger design is:

```text
immutable canonical base
+
workspace overlay
```

For example:

```rust
struct LayeredCallableSignatures {
    canonical:
        Arc<CallableSignatureTable>,

    local:
        HashMap<CallableId, CallableSemanticSignature>,

    removed_or_shadowed:
        HashSet<CallableId>,
}
```

Equivalent layered structures are appropriate for:

```text
declarations
hierarchy
dispatch
signatures
conformance indexes
```

or those structures can migrate to persistent collections.

The desired session initialization cost should approach:

\[
O(1)
\]

with respect to unchanged canonical Universe size.

---

# 19. Metadata export must respect this same base/overlay architecture

Slice 5 showed the current exporter does:

```text
iterate whole declaration table
× scan whole callable table
```

If the compiler gains a layered formal image but metadata exporter immediately calls:

```text
for every canonical declaration
for every canonical callable
```

then part of the 45-second fixed term simply moves downstream.

Therefore canonical semantic reuse and root-driven metadata export are coupled architectural fixes.

Ideal user-program behavior is:

```text
canonical Universe metadata already installed as pool 0

user program metadata:
    fixture/public local roots
    +
    explicit references to canonical identities
```

rather than:

```text
re-export canonical Universe into every program bundle
```

---

# 20. H14 — The newer `ProgramCompilationSession` validates the overlay direction

The persistent fixture path now starts from:

```rust
SemanticWorkspaceSession::new()
```

and applies only:

```rust
WorkspaceSourceBatchMutation::
    SetOverlayForModule { fixture, ... }
```

The module session does not secretly seed all 40 old support modules.

Its dependency discovery is ordinary authored-import transitive discovery.

This is architecturally the right shape:

```text
canonical immutable environment
+
one fixture overlay
```

rather than:

```text
fixture
+
40 canonical sources pretending to be
workspace-local source dependencies
```

The current semantic baseline is not yet rich enough to make that model maximally effective, but the module/session architecture is already pointing in the correct direction.

---

# 21. The target model

The desired cold fixture pipeline should eventually be:

```text
PROCESS ONCE
════════════════════════════════════

canonical Universe sources
        │
        ▼
canonical module graph
        │
        ▼
complete formal semantic analysis
        │
        ├── final canonical TypeStore prefix
        ├── declarations/signatures
        ├── trait contracts
        ├── conformance/witness products
        ├── dispatch/hierarchy
        ├── static/data/enum products
        ├── semantic query image
        ├── canonical metadata
        └── canonical executable program

                 │ immutable/process-shared
                 ▼

PER FIXTURE
════════════════════════════════════

fixture source
        │
        ▼
parse + interface
        │
        ▼
module overlay
        │
        ▼
fork canonical type prefix
        │
        ▼
local semantic queries only
        │
        ├── consult immutable canonical facts
        └── compute fixture-local facts
        │
        ▼
executable-minimum formal snapshot
        │
        ▼
root-driven local metadata
        │
        ▼
lower fixture
```

The 40-module semantic workspace disappears.

The 148-module presentation Universe disappears from executable publication.

Universe body checking disappears from the per-fixture path.

Canonical witness planning disappears from the per-fixture path.

Canonical query publication disappears from the per-fixture path.

---

# 22. What should remain fixture-local

A full canonical image does **not** mean “no semantic work.”

The fixture must still perform semantic work whose truth depends on the fixture:

| Work | Fixture-local? |
|---|---:|
| parse/interface of fixture | yes |
| fixture declarations/signatures | yes |
| fixture callable bodies | yes |
| calls into canonical signatures | yes, caller side |
| generic inference at fixture call sites | yes |
| local conformance declarations | yes |
| canonical conformance lookup | lookup only |
| local/canonical conformance overlap checks | yes where semantically required |
| local inferred returns | yes |
| static constants declared by fixture | yes |
| runtime lowering attachments | yes |
| local executable metadata roots | yes |

This is the appropriate source-dependent component observed in the 92-KB fixture.

The fixed canonical dependency work should be amortized.

---

# 23. Correctness constraints for the image

A canonical semantic image has stricter invariants than a cache.

| Invariant | Requirement |
|---|---|
| **Type identity** | Canonical type IDs have identical numeric identity in every fork |
| **Store isolation** | Every mutable workspace fork has distinct `TypeStoreId` |
| **Canonical immutability** | No workspace mutation may mutate canonical products |
| **Semantic identity** | `DeclarationId`, `CallableId`, `ImplId`, requirement IDs remain identical |
| **Conformance evidence** | Canonical source witness selection must equal full-source analysis |
| **Dispatch** | Canonical member/conditional dispatch must be identical |
| **Overlay composition** | User/local declarations may extend local state without changing canonical state |
| **Diagnostics parity** | Current executable diagnostics remain identical |
| **Provider versioning** | Image valid only for the exact canonical source/provider revision |
| **Override tests** | Noncanonical Universe provider must use a private image or full analysis |
| **Metadata identity** | canonical metadata references remain stable across program compilation |

The key acceptance condition is not “tests pass.”

It should be:

```text
full-source semantic canonical result
==
canonical-image-backed downstream result
```

for every externally observable formal product.

---

# 24. Recommended staged implementation order

A safe implementation should **not** attempt all layers simultaneously.

1. **Retain the already-produced full canonical formal result.** As an observational/prototype step, stop throwing away the canonical semantic snapshot after `canonical_universe_program()` projection and establish exact product census/parity.

2. **Extract a workspace-independent `CanonicalUniverseFormalImage`.** Move ownership into `phalcom-semantic`, where semantic types belong, and make core's canonical program builder consume it rather than independently owning the full semantic analysis.

3. **Fork sessions from the final canonical store prefix and formal products.** Remove the need for the 40-source support closure, initially leaving full workspace/editor publication enabled to isolate semantic correctness.

4. **Introduce an immutable canonical query base.** Stop bootstrapping the same canonical declarations/signatures/data products into every empty `SemanticDb`.

5. **Apply the Slice 8 executable-publication split.** Then `run_inline` stops building advisory, formal editor projection, presentation Universe index, etc.

6. **Fix metadata root selection/canonical-pool references.** Otherwise the compiler still traverses/export the canonical universe after semantic reuse succeeds.

That ordering gives clean parity gates after each structural change.

---

# 25. Instrumentation required before claiming the improvement

The canonical image should expose concrete counters:

```text
canonical_formal_image.build_count
canonical_formal_image.build_time

canonical_formal_image.type_count
canonical_formal_image.declarations
canonical_formal_image.callable_signatures
canonical_formal_image.trait_headers
canonical_formal_image.trait_requirements
canonical_formal_image.conformances
canonical_formal_image.witness_plans
canonical_formal_image.derived_rules
canonical_formal_image.callable_analyses

canonical_query_image.products
canonical_query_image.dependency_edges

session.canonical_query_hits
session.canonical_query_republications
session.canonical_table_materializations
session.canonical_table_entries_copied

fixture.universe_sources_in_semantic_workspace
fixture.universe_callable_bodies_analyzed
fixture.universe_trait_surfaces_built
fixture.universe_conformance_plans_built
fixture.universe_witness_plans_built
```

The long-term expected values for a normal inline fixture should approach:

```text
universe_sources_in_semantic_workspace = 0

universe_callable_bodies_analyzed = 0
universe_trait_surfaces_built     = 0
universe_conformance_plans_built  = 0
universe_witness_plans_built      = 0

canonical_query_republications     = 0
```

while canonical lookup/hit counters can remain high.

---

# 26. Updated diagnosis of the 45-second floor

At this point the causal story is much more coherent.

The current pipeline has **three distinct canonical reuse failures**:

| Layer | Current reuse problem |
|---|---|
| module/interface | fresh resolver bypasses canonical interface cache and can rebuild 148-module declaration catalog |
| formal semantics | complete canonical semantic snapshot exists transiently but is discarded; fixture reanalyzes canonical support sources |
| semantic query DB | even baselined canonical facts are republished into every fresh DB |
| presentation | executable compile builds complete canonical source/navigation index |
| metadata | user program re-exports broad canonical semantic tables despite canonical metadata pool already existing |

These are all manifestations of the same design problem:

> **Canonical Universe identity is recognized globally, but canonical Universe derived products are owned by several local consumers rather than by one immutable dependency-semantic image.**

---

# 27. Strength of the performance conclusion

### `DIRECTLY MEASURED`

The process has a very expensive first canonical compiler-product construction and a separate ~45-second tiny-fixture compilation floor. Pasted text Pasted text

### `SOURCE-CONFIRMED`

The full canonical semantic snapshot is computed during `CanonicalUniverseProgram` construction and not retained in the resulting product.

### `SOURCE-CONFIRMED`

One-shot inline compilation then injects 40 canonical source modules specifically to reconstruct source-authored trait/conformance evidence.

### `STRONG HYPOTHESIS`

Promoting the complete canonical formal result to an immutable dependency image should remove a **large fraction** of the ~45-second fixed semantic term.

### `NOT ESTABLISHED`

It is not yet justified to say that canonical-image reuse alone reduces ~45 seconds to a particular number. Even after it, the current executable path would still pay for source indexing, advisory publication, duplicate formal projection, metadata over-export, fixture-local semantic inefficiencies, and O0 test-profile amplification.

---

## Next slice

The next bounded slice should be **Slice 10: correctness architecture for canonical base + workspace overlay**.

That slice should determine exactly how declarations, dispatch, conformance indexes, witness plans, aliases, static products, and query dependencies compose when a workspace adds local semantics on top of an immutable Universe image—especially shadowing, local conformances against canonical traits/types, overlap checking across base+overlay, TypeStore lineage, and noncanonical Universe overrides.

That is the last major correctness question before we can turn the investigation into a concrete remediation program.