# Telos Analyzer and Compiler Performance Architecture
## Unified Investigation Report, Architectural Resolution, and Implementation Handoff

---

## 0. Purpose of this handoff

This document consolidates the performance investigation into Telos/Phalcom one-shot compilation and turns the findings into an implementation program.

The primary observed problem is a **large, approximately source-size-independent compilation floor of roughly 45 seconds for tiny fixtures** in the historically measured cold `run_inline` path. The investigation has progressively localized this cost away from parsing, VM execution, TypeStore copying, and runtime bytecode lowering and toward a series of architectural lifetime failures:

> **Canonical Universe products already exist at several compiler layers, but those products stop at inconsistent ownership boundaries. Downstream layers repeatedly reconstruct semantic, presentation, query, interface, and metadata products from canonical source or flattened tables.**

The desired outcome is not merely to reduce one benchmark. The objective is to establish an analyzer/compiler architecture in which:

```text
canonical Universe semantics
    are computed once per semantic world

workspace semantics
    analyze only workspace-owned changes

query validation
    treats canonical dependencies as immutable

source/editor products
    reuse canonical persistent shards

compiler projection
    operates from explicit executable/reflection roots

metadata
    does not traverse the entire canonical semantic universe

incremental compilation
    retains precise dependency-driven recomputation

one-shot compilation
    is a short-lived instance of the same architecture,
    not a separate cold whole-workspace compiler
```

The implementation agent should treat this as an **architectural performance program**, not as a collection of isolated micro-optimizations.

---

# 1. Epistemic labels

Maintain these distinctions throughout implementation and reporting.

- **`DIRECTLY MEASURED`** — observed timing/counter/profile evidence.
- **`SOURCE-CONFIRMED`** — mechanism directly established from source.
- **`STRONG HYPOTHESIS`** — strongly supported explanation whose wall-time contribution has not yet been isolated.
- **`PLAUSIBLE HYPOTHESIS`** — credible but weakly measured.
- **`NOT ESTABLISHED`** — do not present as fact.
- **`FALSIFIED`** — investigation found evidence against the hypothesis.

Do not collapse `SOURCE-CONFIRMED` into `DIRECTLY MEASURED`. A bad algorithm may be obviously present in source while its contribution to the 45-second floor remains unmeasured.

---

# 2. Historical measured baseline

The following measurements are the original performance evidence that motivated the investigation. Reproduce equivalent measurements against the current revision before claiming exact improvements; repository state may have changed.

## 2.1 Ordinary core fixtures

For 14 ordinary/non-complexity fixtures:

```text
total test wall                    765.24 s
ProgramCompiler compilation       675.82 s
VM::new()                           85.92 s
vm.run_compiled                      3.31 s
other                                ~0.19 s
```

Approximately **88.3% of total wall time** was spent in one-shot compilation.

### `DIRECTLY MEASURED`

---

## 2.2 Tiny fixtures show a ~45-second fixed floor

Examples:

| Source size | Compilation |
|---:|---:|
| 345 B | 46.044 s |
| 485 B | 44.909 s |
| 687 B | 44.716 s |
| 722 B | 45.560 s |
| 894 B | 47.247 s |
| 1,041 B | 46.691 s |
| 1,102 B | 47.342 s |
| 1,635 B | 44.639 s |
| 1,823 B | 44.707 s |
| 1,846 B | 46.820 s |
| 1,977 B | 44.446 s |
| 2,951 B | 44.820 s |
| 7,494 B | 45.081 s |

Mean:

```text
~45.62 s
```

Median:

```text
~45.56 s
```

### `DIRECTLY MEASURED`

This is the central signal: tiny inputs paid nearly the same compilation cost.

---

## 2.3 Large fixture adds source-dependent cost

A ~92,609-byte randomized fixture measured approximately:

```text
compile      82.796 s
execution     2.503 s
```

This strongly suggests:

```text
T_compile ≈ large fixed cost + source-dependent component
```

### `DIRECTLY MEASURED`

---

## 2.4 Runtime-heavy RangeMap case is a separate problem

A roughly 4.8 KB RangeMap runtime-complexity fixture measured:

```text
compile       48.122 s
execution    469.539 s
```

The runtime problem is real, but independent of the fixed compiler floor.

Do not mix runtime data-structure performance remediation with this compiler program.

---

# 3. Important paths and files

Start a fresh repository pass by locating the current equivalents of these files.

## Core compilation

```text
phalcom-core/tests/support/vm.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/compile_session.rs
phalcom-core/src/modules/canonical_universe.rs
phalcom-core/src/modules/semantic_lowering.rs
```

## Semantic session and immutable publication

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/canonical_baseline.rs
phalcom-semantic/src/workspace.rs
```

## Query architecture

```text
phalcom-semantic/src/db/mod.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/state.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/dependency.rs
```

## Major semantic aggregates

```text
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/data_semantics.rs
phalcom-semantic/src/enum_semantics.rs
phalcom-semantic/src/enum_requirements.rs
phalcom-semantic/src/associated.rs
```

## Presentation/advisory layers

```text
phalcom-semantic/src/source_index/
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/advisory/
```

## Build profile

```text
Cargo.toml
```

Also locate the current `MetadataExporter` implementation and its runtime profile specification before modifying compiler metadata behavior.

---

# 4. Current one-shot architecture

Historically, the tiny fixture path effectively looks like:

```text
run_inline
    │
    ├─ VM::new()
    │
    └─ ProgramCompiler::compile_entry_selection(Inline)
            │
            ├─ parse fixture
            │
            ├─ build synthetic module/project
            │
            ├─ load selected Universe source modules
            │
            ├─ resolve imports
            │
            ├─ link module set
            │
            ├─ analyze_workspace
            │      │
            │      ├─ SemanticWorkspaceSession::new()
            │      ├─ SemanticWorkspaceSession::update()
            │      ├─ semantic fixed points
            │      ├─ source index
            │      ├─ advisory
            │      ├─ formal projection
            │      └─ SemanticSnapshot
            │
            └─ ProgramCompiler projection/lowering/metadata
```

The inline compiler explicitly seeds canonical source modules matching roughly:

```rust
["collections", ..]
| ["scalar", ..]
| ["text", ..]
| ["option", "unit"]
```

At the investigated revision this was exactly **40 Universe semantic source modules**, plus the synthetic user module.

The critical fact is:

> These canonical modules are being treated as ordinary source-owned semantic input because the reusable baseline does not contain the complete semantic products that those modules establish.

---

# 5. Major finding: this is primarily a lifetime/ownership defect

The compiler already has many correct ingredients:

```text
process-cached canonical parsed source
process-cached canonical interfaces
canonical linked Universe cache
canonical partial semantic baseline
persistent TypeStore backing
incremental SemanticDb
WorkspaceModuleSession
ProgramCompilationSession
immutable SemanticSnapshot
persistent source-index module roots
persistent formal-projection module roots
runtime-module-local lowering
```

Yet downstream systems repeatedly downgrade derived products to raw inputs.

Typical pattern:

```text
derived canonical product exists
        ↓
API boundary does not retain/expose it
        ↓
consumer receives raw source / flat table
        ↓
consumer reconstructs equivalent product
```

This pattern appears across:

```text
module interfaces
semantic products
query products
source indexes
formal projection
advisory summaries
metadata export
```

This is the central architectural diagnosis.

---

# 6. What has been falsified

Do not waste the primary optimization program on the following theories.

## 6.1 TypeStore whole-copying is not the 45-second root cause

### `FALSIFIED`

`TypeStore::fork_from_baseline()` preserves the canonical prefix using persistent `im` structures.

It assigns a new store identity but does not deep-copy the entire canonical type universe.

A future image still needs explicit prefix-lineage correctness, but a full TypeStore deep copy is not the dominant observed mechanism.

---

## 6.2 Universe source parsing is not repeated cold on every fixture

### `FALSIFIED`

The builtin source machinery has process-level parsed-source caching.

Canonical ASTs are generally reused.

The problem is repeated **processing of those ASTs**, not reparsing every source string from scratch.

---

## 6.3 The 40 Universe modules are not all bytecode-compiled for every fixture

### `FALSIFIED`

`AnalyzedProgram.runtime_modules` for inline compilation contains the user runtime module.

Compiler lowering skips non-runtime modules.

Thus the ~45-second floor is not “compile the Universe to bytecode again.”

It occurs mostly before or around projection/publication.

---

## 6.4 Full canonical Universe relinking inside semantic update is not the main repeated path

### `FALSIFIED`

There is a process-shared canonical linked Universe.

Do not conflate that with separate resolver/interface reconstruction bugs.

---

# 7. Source-confirmed fixed-cost mechanisms

---

## 7.1 Forty canonical Universe modules are deeply analyzed as workspace source

### `SOURCE-CONFIRMED`
### Priority: **Critical**

On cold publication:

```rust
semantic_work_modules =
    current_modules.clone()
```

when there is no previous snapshot and no narrowed deep frontier.

For the inline path that means approximately:

```text
40 canonical Universe modules
+ 1 fixture module
```

all enter deep semantic work.

This brings along:

```text
declaration work
signature work
hierarchy work
callable body checking
conformance settlement
inferred returns
module values
diagnostics
advisory
formal products
```

This is the largest architectural error.

The canonical Universe should be an immutable dependency semantic world, not ordinary workspace source.

---

# 8. Resolver/interface reconstruction

## 8.1 Fresh `ModuleResolver` bypasses canonical interface cache

### `SOURCE-CONFIRMED`
### Priority: High

A fresh resolver has its own local interface cache.

Canonical provider API paths can use the process-shared `BuiltinInterfaceBuilder` cache, but the resolver reaches a path equivalent to:

```text
build_from_parsed(...)
```

rather than the process-cached canonical interface.

The first absolute Universe import can therefore rebuild the Universe source declaration catalog.

That catalog scans every provider node and builds source interfaces.

This can happen despite parsed ASTs already being cached.

---

## 8.2 Approximate static interface-build lower bound

At the investigated state, a tiny fixture could trigger roughly:

```text
initial fixture interface                       1
resolver root interface                         1
Universe declaration catalog                  148
SemanticWorkspaceInput topology                41
semantic unlinked-interface query              41
snapshot/module-product reconstruction         41
-------------------------------------------------
lower-bound-ish total                         273
```

This is a static source-derived count, not a measured runtime count.

### `SOURCE-CONFIRMED mechanism`
### Magnitude: unmeasured

---

## 8.3 `SemanticWorkspaceInput::new()` builds interfaces then discards them

### `SOURCE-CONFIRMED`

Current compatibility constructor behavior is effectively:

```rust
let mut unlinked = BTreeMap::new();

for source in sources {
    let interface =
        InterfaceBuilder::build(...);

    unlinked.insert(...);
}

let topology =
    ModuleTopology::from_parts(..., &unlinked, ...);

Self {
    ...
    interfaces: Arc::new(BTreeMap::new()),
}
```

The interface work is performed for topology and then thrown away.

That guarantees later reconstruction.

This should be fixed independently and early.

---

# 9. Query-state reconstruction

## 9.1 Fresh `SemanticDb` has no canonical query state

### `SOURCE-CONFIRMED`
### Priority: Critical

A new DB begins with empty:

```text
query states
typed products
last-known-good query products
dependency index
scheduler state
revision event sets
```

The session therefore republishes canonical baseline products into each fresh DB.

---

## 9.2 Canonical dependencies currently require current workspace revision

The DB's reuse/dependency machinery assumes a dependency is usable only if its local query row is current:

```text
dependency QueryState exists
AND
validated_revision == current DB revision
AND
fingerprint matches
```

That assumption is correct for mutable workspace products.

It is incorrect for immutable canonical products.

This forces unnecessary canonical query republication.

---

# 10. Multiple formal body-analysis waves

### `SOURCE-CONFIRMED`
### Priority: High after canonical-image cutover

Cold semantic publication can analyze callable bodies in multiple waves:

```text
trait default bodies
class bodies
conformance witness bodies
inherent impl bodies
downstream reverse-dependency reanalysis
cycle reanalysis
inferred-return fixed point
conformance-plan-client reanalysis
late module-value revalidation
```

Not every callable executes every wave, but the architecture permits repeated work.

---

## 10.1 Module values force a systematic second eligibility state

Earlier body analysis runs with approximately:

```text
module_values = None
```

Late final validation runs with:

```text
module_values = Some(...)
```

The formal input fingerprint includes whether a module-value environment exists.

Therefore:

```text
false → true
```

cannot be a cache hit.

Bodies eligible for the final pass are necessarily reanalyzed.

This should eventually become dependency-sensitive rather than unconditional.

---

# 11. Conformance subsystem amplification

The seeded Universe set is conformance-heavy.

At the investigated state, source-level census found roughly:

```text
82 impl headers
73 explicit trait conformances
```

with especially large families around:

```text
Iterator
Iterable
```

---

## 11.1 Pairwise overlap detection

### `SOURCE-CONFIRMED`

`ConformanceIndex::overlap_conflicts` performs pairwise candidate comparisons.

For ~73 conformance heads:

```text
theoretical outer pairs ≈ 2,628
```

Iterator + Iterable alone produced a large same-trait pair population.

This work belongs once in the canonical image for canonical↔canonical pairs.

Workspace compilation should only need:

```text
local ↔ local
local ↔ canonical
```

---

## 11.2 Witness plans are built provisionally and finally

### `SOURCE-CONFIRMED`

Conformance witness publication has at least:

```text
provisional pass
final pass
```

Requirement instantiation/normalization is duplicated between outer publication and inner plan construction.

---

## 11.3 Global signature scan per trait requirement

### `SOURCE-CONFIRMED`
### Algorithmically severe

Witness selection contains behavior equivalent to:

```rust
for requirement in requirements {
    callable_signatures.iter().find_map(...)
}
```

So each requirement searches the entire signature table for its conformance-owned implementation.

For the investigated Iterator/Iterable surface alone, approximate requirement visits were on the order of:

```text
~1,302 per pass
~2,604 across provisional + final passes
```

and each visit may scan a large global signature table.

Correct architecture:

```text
ImplId
  → selector/side
      → candidate callable
```

or an equivalent owner/selector index.

---

## 11.4 Exact conformance proof memo is too narrow

### `SOURCE-CONFIRMED`

Generic conformance proof constructs a short-lived proof context.

Exact proof can scan the conformance index linearly.

Identical propositions can therefore be recomputed across invocations.

This should eventually become an indexed/query-level operation.

---

# 12. Source semantic index expands beyond semantic input

### `SOURCE-CONFIRMED`
### Priority: Critical

This was one of the strongest findings.

Cold `build_source_semantic_index` does not limit itself to the semantic source set.

It enumerates all canonical Universe provider nodes for presentation/navigation.

At the investigated state this meant approximately:

```text
~148 Universe provider modules
+ synthetic user module
≈ 149 source-index modules
```

So even though only ~40 Universe modules were deeply analyzed, the presentation/source system expanded to almost the entire Universe.

---

# 13. Source index repeats identical full AST traversals

### `SOURCE-CONFIRMED`
### Priority: Very High

For each source module, three functions independently instantiate the same type-target collector and walk the entire program:

```text
resolve_type_reference_targets
resolve_type_member_projection_targets
resolve_type_parameter_source_targets
```

The collector computes all three categories, but each function returns only one.

For ~149 cold source-index modules:

```text
149 × 3 = 447
```

full target-collector AST traversals before other source-index passes.

---

## 13.1 Each conformance can trigger another whole-program traversal

A conformance prepass calls:

```text
resolve_type_reference_targets(
    module,
    &source.program,
    resolver
)
```

inside a loop over source conformance statements.

Using only the known ~73 seeded conformances as a conservative floor:

```text
447 mandatory traversals
+ ≥73 conformance-triggered traversals
= ≥520 module-program collector traversals
```

The true count can be higher because presentation-only Universe modules may also contain conformances.

This is a clear algorithmic defect.

Correct product:

```rust
struct ResolvedSourceTypeTargets {
    nominal: ...,
    type_members: ...,
    type_parameters: ...,
}
```

computed once per module.

---

# 14. Source-index attachment scans

### `SOURCE-CONFIRMED`

`ModuleSourceIndex` construction includes nested searches where formal targets scan all expression sites and sometimes all occurrences.

Examples structurally resemble:

```text
formal target
  × all expression sites

formal target
  × all occurrences
```

These should use direct indexes:

```text
SourceSiteId → SourceSite
range/owner → occurrences
```

This matters more for large-source scaling than the fixed floor, but canonical presentation reuse removes the Universe portion entirely.

---

# 15. Advisory is effectively a second whole-workspace analysis

### `SOURCE-CONFIRMED`
### Priority: Critical

After formal semantic checking and source-index construction, session publication unconditionally builds an `AdvisoryWorkspace`.

It iterates retained semantic sources and performs another dataflow-style analysis involving:

```text
field facts
targets
callable bodies
top-level statements
expressions
bindings
parameters
return facts
interprocedural summaries
```

Then an `AdvisorySolver` runs a worklist/fixed point over callable dependencies.

There is therefore:

```text
outer workspace advisory fixed point
    containing
inner callable-summary worklist fixed point
```

---

## 15.1 Advisory performs repeated global selection scans

Per module, advisory logic may scan the whole callable-analysis set and filter by module.

This has a shape close to:

```text
modules × all-callables
```

per outer advisory iteration.

Pregroup analyses by module.

---

## 15.2 Advisory range lookups can scan occurrences repeatedly

Several helpers resolve exact source ranges by scanning occurrence/expression structures.

Index them.

---

## 15.3 Canonical advisory products should be image-owned

The crucial architectural fix is not to micro-optimize canonical advisory analysis.

The canonical Universe advisory result should be built once into the canonical image.

Workspace advisory should operate only on local shards and dependency-affected local products.

---

# 16. Formal projection

### `SOURCE-CONFIRMED`

`FormalSemanticProjection` is already architecturally promising:

```rust
im::OrdMap<
    ModuleId,
    Arc<ModuleFormalProjection>
>
```

It supports persistent module roots.

This should be reused directly by the canonical image.

---

## 16.1 Duplicate projection construction

Current snapshot construction builds a projection from callable analyses and the session later replaces it with the source-index-aware projection.

This is pure duplicate work and an API smell.

Snapshot construction should accept the authoritative projection directly.

---

# 17. Snapshot publication

### `SOURCE-CONFIRMED`

Current code explicitly acknowledges whole-root materialization:

```text
snapshot_full_map_materializations += 8
snapshot_roots_replaced += 8
type_store_snapshot_deep_copies = 0
advisory_flattening_work = 0
```

This is useful evidence:

- TypeStore is already shallow.
- Several semantic aggregates are not.

The canonical-image program should eventually complete this persistent-root migration.

---

# 18. Compiler lowering itself is largely runtime-module-local

### `SOURCE-CONFIRMED`

The compiler skips non-runtime modules during actual bytecode lowering.

That is good architecture.

However, **compiler semantic projection before lowering is not equally local**.

---

# 19. Metadata export leaks back to whole-snapshot scope

### `SOURCE-CONFIRMED`
### Priority: High after semantic cutover

The compiler constructs `MetadataExporter` with whole semantic tables and calls something equivalent to:

```rust
MetadataProfile::RuntimePublic
```

with runtime roots.

But root selection is not truly restrictive.

The exporter iterates broad tables including:

```text
all declarations
all callable signatures
aliases
static domains
```

---

## 19.1 Declaration × callable scan

For each declaration, metadata code may scan the full callable table to find declaration-owned callables.

This is roughly:

```text
O(D × C)
```

and those callables are then also independently exported.

Add a declaration-owner callable index.

---

## 19.2 `MetadataProfile` is not sufficiently operational

The investigated implementation used profile primarily as metadata/header identity rather than as a strong root/filter policy.

The intended behavior should be explicit:

```text
RuntimeMinimal
    executable semantic metadata only

RuntimePublic
    executable metadata
    + reflection-visible public metadata
```

Do not treat `RuntimePublic` as “serialize every semantic table.”

---

## 19.3 Canonical runtime metadata already has a pool

The VM already maintains a canonical Universe semantic metadata pool.

A normal user program should not blindly regenerate broad Universe metadata into its own metadata pool.

Eventually introduce cross-pool canonical references or equivalent deduplication.

Do not implement this before the canonical semantic-image boundary is stable.

---

# 20. Build-profile amplification

### `SOURCE-CONFIRMED`

Root test profile historically had approximately:

```toml
[profile.test]
inherits = "dev"
opt-level = 0
codegen-units = 256
lto = false
```

while package wildcard dependency overrides used:

```toml
[profile.test.package."*"]
opt-level = 3
```

Cargo wildcard package overrides target dependencies, not workspace members.

Therefore the Telos compiler workspace crates themselves ran at roughly:

```text
opt-level = 0
debug assertions = enabled
overflow checks = enabled
```

during tests.

This can dramatically amplify:

```text
HashMap/BTreeMap work
iterator-heavy code
small helper functions
cloning
hashing
recursive type operations
fixed-point loops
```

### Important

This is a **multiplier**, not the architectural root cause.

Do not “solve” the issue by changing the test profile to `opt-level=3`.

Run an A/B test because it gives valuable diagnostic information.

Suggested experiment:

```text
test compiler at opt-level 0
test compiler at opt-level 1
test compiler at opt-level 2
test compiler at opt-level 3
```

Prefer separate target directories.

Interpretation:

```text
45s → 5s
    optimization level is a huge multiplier,
    architecture still does unnecessary work.

45s → 30s
    both algorithmic structure and optimization matter.

45s → 40s
    architecture dominates overwhelmingly.
```

---

# 21. Current persistent compilation path proves work is avoidable

`ProgramCompilationSession` already retains:

```text
SemanticWorkspaceSession
WorkspaceModuleSession
stable fixture module identity
```

and applies source overlays to the same module/session.

This is the correct architectural direction.

The language corpus already uses this persistent path in some places.

However many core tests still call cold:

```text
run_inline(...)
```

which invokes fresh one-shot compilation.

Therefore always benchmark **both populations**:

```text
cold one-shot compile
persistent repeated compile
```

Do not claim global improvement using only the amortized language corpus.

---

# 22. Central architectural resolution

The optimized architecture should have four explicit lifetime domains.

```text
┌───────────────────────────────────────────────┐
│ 1. Canonical semantic foundation             │
│                                               │
│ Minimal/current bootstrap state sufficient    │
│ to analyze the complete Universe once.        │
└───────────────────────┬───────────────────────┘
                        │ one process/world build
                        ▼
┌───────────────────────────────────────────────┐
│ 2. Canonical Universe Formal Image            │
│                                               │
│ Complete immutable formal Universe semantics  │
│ + query products                              │
│ + module products                             │
│ + source/presentation shards                  │
└───────────────────────┬───────────────────────┘
                        │ immutable base
                        ▼
┌───────────────────────────────────────────────┐
│ 3. Workspace semantic overlay                 │
│                                               │
│ Only local/editable source modules            │
│ Local DB states and dependency graph          │
│ Persistent aggregate roots                    │
└───────────────────────┬───────────────────────┘
                        │ explicit roots
                        ▼
┌───────────────────────────────────────────────┐
│ 4. Executable/compiler projection             │
│                                               │
│ Runtime modules                               │
│ Reachable semantic metadata                   │
│ Optional reflection roots                     │
└───────────────────────────────────────────────┘
```

---

# 23. The current baseline should become the bootstrap foundation

Do not rewrite bootstrap semantics first.

Current `CanonicalUniverseSemanticBaseline` is partial, but it is already proven sufficient to run the ordinary semantic analyzer over the complete Universe.

Use it initially as:

```text
Canonical Semantic Foundation
```

whether or not the Rust type is renamed immediately.

The build sequence should be:

```text
existing partial baseline
        ↓
SemanticWorkspaceSession::from_baseline(...)
        ↓
analyze complete canonical Universe
        ↓
successful committed SemanticSnapshot
        ↓
freeze CanonicalUniverseFormalImage
```

This avoids bootstrap recursion.

---

# 24. Complete Universe analysis already exists

`phalcom-core/src/modules/canonical_universe.rs` already performs essentially:

```text
load all Universe sources
link complete Universe
analyze_workspace(...)
compile canonical runtime program
```

Therefore:

> The ordinary semantic engine is already capable of constructing the complete Universe semantics.

Do not build a second semantic analyzer specifically for the image.

Move/reuse that existing complete semantic publication at the correct ownership layer.

---

# 25. `CanonicalUniverseFormalImage`

The image should live in `phalcom-semantic`, not `phalcom-core`.

Conceptually:

```rust
pub struct CanonicalUniverseFormalImage {
    world:
        CanonicalSemanticWorldId,

    snapshot:
        Arc<SemanticSnapshot>,

    query_image:
        Arc<CanonicalSemanticQueryImage>,

    linked:
        Arc<LinkedProgram>,

    interfaces:
        Arc<CanonicalInterfaceSet>,

    sources:
        Arc<CanonicalSourceSet>,
}
```

The exact wrappers can evolve.

The important point is that the authoritative complete `SemanticSnapshot` is already almost exactly the semantic image payload.

---

# 26. Freeze after successful semantic transaction commit

Do not freeze intermediate checker state.

Correct boundary:

```text
begin semantic transaction
        ↓
ordinary semantic publication
        ↓
all products completed
        ↓
commit
        ↓
freeze image
```

At freeze time:

```text
Computing       == 0
Cancelled       == 0
BudgetExceeded  == 0
```

Audit `Blocked` and `Failed` query states rather than silently dropping them.

---

# 27. Canonical query image

The mutable `SemanticDb` should not itself be frozen wholesale.

Extract the immutable useful portion.

Conceptually:

```rust
pub struct CanonicalQueryEntry {
    pub input_fingerprint:
        InputFingerprint,

    pub product_fingerprint:
        ProductFingerprint,

    pub product:
        Arc<SemanticProduct>,

    pub dependencies:
        Arc<[CanonicalDependency]>,
}
```

and:

```rust
pub struct CanonicalSemanticQueryImage {
    entries:
        BTreeMap<QueryKey, CanonicalQueryEntry>,
}
```

Do not freeze:

```text
scheduler
transaction journal
mutable reverse invalidation graph
current revision
revision recomputation sets
revision validation event sets
mutable metrics
```

---

# 28. Semantic DB needs canonical fallback semantics

Introduce an explicit concept of product origin:

```rust
enum SemanticProductOrigin {
    Canonical,
    Workspace,
}
```

Recommended core API:

```rust
fn product_origin(
    &self,
    key: &QueryKey,
) -> Option<SemanticProductOrigin>;

fn product(
    &self,
    key: &QueryKey,
) -> Option<&Arc<SemanticProduct>>;

fn ready_product_fingerprint(
    &self,
    key: &QueryKey,
) -> Option<ProductFingerprint>;

fn is_current(
    &self,
    key: &QueryKey,
) -> bool;

fn local_query_state(
    &self,
    key: &QueryKey,
) -> Option<&QueryState>;
```

---

# 29. `query_state()` must remain local/mutable

Do not fake a canonical image entry as:

```text
QueryState::Ready
```

inside the workspace DB.

Existing code frequently does:

```rust
if db.query_state(&key).is_some() {
    db.discard_for_recompute(&key);
}
```

Canonical products must never be discardable through workspace mutation APIs.

Correct distinction:

```text
query_state / local_query_state
    mutable workspace product only

product / fingerprint / currentness
    workspace first, canonical fallback
```

---

# 30. Canonical currentness

For workspace products:

```text
validated_revision == current revision
```

remains correct.

For canonical products:

```text
exists in this immutable semantic world
```

is sufficient.

Conceptually:

```rust
fn is_current(
    &self,
    key: &QueryKey,
) -> bool {
    if let Some(local) =
        self.local_query_state(key)
    {
        return local.validated_revision()
            == Some(self.revision());
    }

    self.canonical
        .contains_ready(key)
}
```

---

# 31. Dependency recording

Current:

```text
local dependent
    → canonical dependency
```

must record the canonical product fingerprint without publishing a local canonical query row.

Conceptually:

```rust
match db.product_origin(&dependency) {
    Workspace => {
        require current local state;
        recorder.record(
            dependency,
            local_fingerprint,
        );
    }

    Canonical => {
        recorder.record(
            dependency,
            canonical_fingerprint,
        );
    }
}
```

---

# 32. Do not create workspace reverse edges from immutable canonical dependencies

A workspace body can record:

```text
LocalBody
    → Universe::Iterator.next
```

as a forward fingerprint dependency.

But ordinary workspace invalidation does not need:

```text
Universe::Iterator.next
    → LocalBody
```

inside its mutable reverse index because that canonical provider cannot change.

This reduces DB size and invalidation work.

---

# 33. Remove current Universe “assume current” shortcuts

Current dependency replay contains cases roughly equivalent to:

```rust
if !inputs.sources.contains_key(
        callable.module()
    )
    && callable.module().project
        == ProjectIdentity::Universe
{
    return QueryOutcome::Ready(());
}
```

for several dependency classes.

These are architectural placeholders.

Replace them with:

```rust
if db.is_current(&key) {
    return QueryOutcome::Ready(());
}
```

backed by the canonical query image.

That converts an assumption into verified semantic ownership.

---

# 34. Arena lineage invariants

---

## 34.1 TypeStore

The complete canonical image's final `TypeStore` must become the prefix for all workspaces.

Important:

```text
partial baseline TypeStore
    is NOT enough

full canonical semantic analysis
    may allocate additional TypeIds
```

Therefore the child workspace must fork from:

```text
image.final_type_store
```

not the pre-image baseline store.

Add explicit lineage identity such as:

```rust
struct CanonicalTypePrefixId(u64);
```

A workspace must fail closed if canonical semantic products are used with a store not forked from that prefix.

---

## 34.2 StaticValueStore

`StaticValueRef` already embeds:

```text
StaticValueStoreId
```

and cloned `StaticValueStore`s intentionally preserve lineage identity.

Therefore:

```text
image owns final canonical StaticValueStore
workspace clones/COW-extends it
```

Do not create:

```rust
StaticValueStore::new()
```

when instantiating a session from the full image.

---

# 35. Canonical project ownership makes overlay semantics simpler

Universe and user project identities are structurally disjoint.

Ordinary local code cannot simply replace a canonical declaration by spelling the same name.

Similarly, inherent impl authorization prevents arbitrary foreign-type extension.

Thus normal workspace semantics do not require a general “shadow canonical entry” system.

Conceptually:

```text
canonical identities
    immutable

workspace identities
    mutable

resolution
    can reference either
```

This substantially simplifies the overlay.

---

# 36. Conformance is the important exception

Storage may be layered, but conformance candidate semantics must observe the union.

Incorrect:

```text
try local conformances
if none:
    try canonical conformances
```

That creates precedence not present in the language.

Correct:

```text
candidates =
    canonical source heads
  ∪ workspace source heads
  ∪ canonical applicable derived rules
  ∪ workspace applicable derived rules

then apply existing coherence semantics
```

---

# 37. Incremental cross-layer coherence

Once the canonical image certifies:

```text
canonical ↔ canonical
```

coherence once, a workspace only needs fresh checking for:

```text
workspace ↔ workspace
workspace ↔ canonical
```

This converts a repeated global pairwise problem into an incremental boundary problem.

Do not lose this opportunity by flattening everything into an indistinguishable mutable map.

---

# 38. Canonical derived rules remain executable against local types

A canonical generic/guarded conformance rule can apply to a newly defined workspace type.

Therefore distinguish:

```text
immutable canonical facts
immutable canonical rules
workspace facts
```

The image freezes the rule definition.

It does not precompute every future subject the rule might apply to.

---

# 39. Workspace session lifecycle after image cutover

A normal session should conceptually begin as:

```rust
SemanticWorkspaceSession {
    canonical:
        image.clone(),

    db:
        SemanticDb::with_canonical(
            workspace,
            image.query_image.clone(),
        ),

    store:
        image.store.fork_from_baseline(),

    static_values:
        image.static_values.clone(),

    sources:
        BTreeMap::new(),

    semantic_structure_shards:
        BTreeMap::new(),

    last_snapshot:
        None,

    ...
}
```

---

# 40. Do not install the canonical snapshot as `last_snapshot`

This is critical.

Do **not** do:

```rust
session.last_snapshot =
    Some(image.snapshot.clone());
```

The image snapshot belongs to a different workspace identity and source lifecycle.

Doing so would make incremental logic believe all canonical Universe modules had previously been workspace sources and would reintroduce enormous invalidation/frontier work.

Correct model:

```text
canonical base:
    image

workspace previous snapshot:
    None

workspace local source set:
    empty
```

---

# 41. Do not seed local semantic lifecycle structures with canonical modules

Keep these local:

```text
sources
source_fingerprints
semantic_structure_shards

generic header dependency graph
declaration/signature dependency graph
alias dependency graph

module diagnostics
workspace revision effects
```

The canonical image may retain equivalent canonical products for reading, but those are not mutable workspace lifecycle inputs.

---

# 42. First compatibility cutover: initialize aggregates from the image

The first version does **not** need to solve all structural sharing.

Current code often does:

```rust
let mut declarations =
    self.base_declarations.clone();
```

or:

```rust
previous_snapshot
    .map_or_else(
        || self.base_hierarchy.clone(),
        |snapshot| (*snapshot.hierarchy).clone(),
    )
```

Initial image cutover can safely become:

```rust
let mut declarations =
    (*self.canonical.snapshot.declarations)
        .clone();
```

and:

```rust
let mut hierarchy =
    previous_snapshot
        .as_ref()
        .map_or_else(
            || {
                (*self
                    .canonical
                    .snapshot
                    .hierarchy)
                    .clone()
            },
            |snapshot| {
                (*snapshot.hierarchy).clone()
            },
        );
```

This still copies compatibility tables.

That is acceptable for the first parity patch.

The major win is that it eliminates canonical source semantic reconstruction.

---

# 43. Do not add dozens of new `base_*` fields

Avoid:

```text
base_trait_headers
base_trait_surfaces
base_conformance_index
base_derived_rules
base_callable_analyses
base_static_constants
...
```

This merely recreates `SemanticSnapshot` by hand.

One image root should be authoritative.

---

# 44. Persistent-root migration after semantic cutover

Once parity is established, eliminate the remaining canonical table copies.

The codebase already has excellent precedents:

```text
TypeStore
    im::HashMap / im::Vector

SourceSemanticIndex
    im::OrdMap<ModuleId, Arc<ModuleSourceIndex>>

FormalSemanticProjection
    im::OrdMap<ModuleId, Arc<ModuleFormalProjection>>
```

Apply the same model to large semantic aggregates.

---

# 45. Prefer module-sharded persistence

Many current tables maintain redundant module indices specifically to support:

```text
remove_module(...)
```

That is a strong architectural hint.

Instead of:

```rust
HashMap<CallableId, Signature>
+
HashMap<ModuleId, Set<CallableId>>
```

prefer:

```rust
im::OrdMap<
    ModuleId,
    Arc<ModuleCallableSignatures>
>
```

Lookup APIs can remain unchanged.

Workspace modification then clones/replaces only an affected module shard.

---

# 46. High-value tables for persistence

Prioritize:

```text
DeclarationTypeTable
CallableSignatureTable
FieldSignatureTable
SurfaceDispatchResolver
MapTypeHierarchy

TraitHeaderTable
TraitSurfaceTable

ConformanceIndex
conformance witness plans
DerivedConformanceRuleIndex
TraitDispatchIndex

DataSemanticTable
EnumSemanticTable
EnumRequirementTable
AssociatedFamilyTable

type-member maps
static-domain maps
static case tables
static definition/constant tables

AdvisoryWorkspace root maps
```

Do not simultaneously rewrite checker APIs into generic “table view” traits unless profiling proves necessary.

The checker currently takes concrete tables pervasively; preserving those public shapes while changing internal storage has much lower blast radius.

---

# 47. Module-system architecture after cutover

The module layer should make the same ownership distinction as semantics.

Desired model:

```text
CanonicalModuleWorld
    immutable Universe interfaces
    immutable linked canonical graph
    canonical source identities
    canonical import/export products

WorkspaceModuleSession
    workspace/project modules only
    mutable overlays only
```

A workspace import such as:

```text
from universe.collections ...
```

should resolve against the canonical external module world.

That does **not** mean the imported Universe source module becomes a mutable workspace source.

---

# 48. One-shot compilation should use the production module-session path

Current richer architecture:

```text
WorkspaceModuleSession
    ↓
WorkspaceModuleUpdate
    ↓
SemanticWorkspaceSession::
    update_module_workspace_with_cancel
```

already carries:

```text
interfaces
import products
import-site ownership
topology
reverse imports
module delta
graph-changed state
```

By contrast `SemanticWorkspaceInput::new()` is a compatibility path that reconstructs missing information.

One-shot compilation should become a short-lived instance of the production persistent path:

```rust
let mut session =
    ProgramCompilationSession::one_shot(...);

session.compile(...)
```

The fact that it lives for one invocation does not justify a second compiler architecture.

---

# 49. Remove the 40-source workaround only after parity

The filters for:

```text
collections
scalar
text
option.unit
```

must remain until the full image and query layer demonstrably provide all required semantics.

Then delete the workaround from:

```text
Inline
Standalone module
Package/project discovery
```

in one clearly gated cutover.

---

# 50. Presentation/source architecture after image cutover

The canonical image should own:

```text
canonical SourceSemanticIndex
canonical FormalSemanticProjection
canonical presentation source text
canonical advisory products
```

Workspace publication should process only local source modules.

For editor consumers, provide a composed view.

For executable compilation, do not materialize a combined 149-module source index unless a consumer explicitly needs it.

---

# 51. Compiler metadata architecture

After semantics is fixed, implement root-driven metadata.

Introduce a concept such as:

```rust
pub struct MetadataExportPlan {
    pub profile:
        MetadataProfile,

    pub executable_roots:
        Arc<[SemanticRoot]>,

    pub reflection_roots:
        Arc<[SemanticRoot]>,

    pub canonical_pool:
        Option<CanonicalMetadataPoolRef>,
}
```

Then compute a reachability closure.

---

# 52. `RuntimeMinimal` and `RuntimePublic`

Required semantics:

```text
RuntimeMinimal
    only executable semantics required by runtime

RuntimePublic
    RuntimeMinimal
    +
    public reflection-visible semantics
```

Neither should mean:

```text
iterate every declaration
iterate every callable
serialize all aliases
serialize all static domains
```

unless those objects are actually reachable under that profile.

---

# 53. Local algorithmic optimization package

After lifetime architecture is corrected, address the remaining scaling defects.

## High priority

### Source target collector

Collapse:

```text
resolve_type_reference_targets
resolve_type_member_projection_targets
resolve_type_parameter_source_targets
```

into one traversal.

---

### Conformance source lookup

Do not rerun the whole type-target traversal per conformance.

Use the already-computed module target product.

---

### Witness selection

Replace:

```text
requirement
    × global callable-signature scan
```

with an owner/selector/side index.

---

### Exact conformance lookup

Index by at least:

```text
trait declaration
target nominal family
generic/covering candidate bucket
```

Do not linearly scan the entire conformance index.

---

### Proof memoization

Promote reusable exact generic-conformance propositions to appropriate session/query-level cache products.

---

### Source attachment lookup

Add:

```text
SourceSiteId → SourceSite
range/owner → occurrence slice
```

---

### Advisory

Pregroup callables by module.

Replace range-based full scans with indexes.

---

### Lowering

Replace expression × full reference scans with indexed reference resolution.

---

### Conditional inherent implementation lookup

Add:

```text
ImplId → InherentImplDomain
```

rather than global scans.

---

### Trait requirement slot map

Compute once per semantic/lowering context rather than once per invocation.

---

# 54. Module-value double body analysis

Eventually replace unconditional late reanalysis with true dependencies.

The correct end state is one of:

```text
A. module-value facts available before ordinary body analysis

B. body product split into
   module-value-independent core
   +
   module-value-dependent refinement

C. dependency-track actual module-value reads
   and reanalyze only consumers
```

Do not simply omit `module_values` from the fingerprint.

That would make the cache unsound.

---

# 55. Implementation program

Use multiple implementation plans.

Do **not** deliver everything as one monolithic patch.

---

## Program P0 — Measurement lock and reproducibility

Before major changes:

1. Reproduce cold tiny-fixture compilation.
2. Reproduce retained-session compilation.
3. Capture current source/provider counts.
4. Record current branch/revision.
5. Perform test-profile opt-level A/B.
6. Add sufficiently narrow instrumentation.

Required timing hierarchy:

```text
compile.total

module_resolution.total
module_resolution.canonical_interface
module_resolution.catalog

semantic.session_init
semantic.total

semantic.structural
semantic.signatures
semantic.bodies
semantic.conformance
semantic.source_index
semantic.advisory
semantic.formal
semantic.snapshot

compiler_projection.total
compiler_projection.metadata
compiler_projection.lowering
```

---

## Program P1 — Reconstruction hygiene

Low-risk independent fixes:

### P1.1

Canonical `ModuleResolver` should use the canonical provider/interface cache instead of rebuilding from parsed source.

### P1.2

`SemanticWorkspaceInput::new()` must retain interfaces it already built.

### P1.3

Remove duplicate formal projection construction.

### P1.4

Where straightforward, stop rebuilding interface/module products already supplied by `WorkspaceModuleSession`.

Acceptance:

```text
canonical InterfaceBuilder builds
per warmed fixture
    → 0 or near-zero

Universe declaration catalog rebuild
per fixture
    → 0
```

---

## Program P2 — Complete canonical formal image

Create:

```text
phalcom-semantic/src/canonical_image.rs
```

Implement:

```text
existing bootstrap foundation
    ↓
ordinary full Universe semantic publication
    ↓
freeze snapshot
    ↓
freeze query image
```

No workspace cutover yet.

Acceptance:

```text
canonical image build count = 1/process/world
```

and complete semantic parity against the existing full Universe analysis.

---

## Program P3 — Canonical-aware SemanticDb and session instantiation

Implement:

```text
canonical query image
product origin
canonical-aware product lookup
canonical-aware fingerprint lookup
is_current
canonical-aware dependency recording
```

Add:

```text
SemanticWorkspaceSession::from_image(...)
```

but initially permit compatibility aggregate cloning.

Acceptance:

```text
canonical query products republished
    == 0
```

for image-backed workspace semantic dependencies.

---

## Program P4 — Module/session cutover and removal of canonical source seeding

Move one-shot compilation onto the production module-session architecture.

Treat Universe as immutable external module world.

Remove the 40-module source seed only after differential parity.

Acceptance:

```text
canonical Universe modules in workspace sources
    == 0

canonical deep semantic modules
    == 0

canonical callable bodies analyzed
    == 0

canonical conformance plans rebuilt
    == 0
```

This is expected to produce the **primary fixed-floor collapse**.

---

## Program P5 — Canonical presentation/advisory/formal reuse

Ensure:

```text
SourceSemanticIndex
FormalSemanticProjection
AdvisoryWorkspace
```

start from canonical persistent roots and replace only local module shards.

Acceptance:

```text
canonical source-index modules rebuilt
    == 0

canonical formal modules rebuilt
    == 0

canonical advisory modules solved
    == 0
```

---

## Program P6 — Persistent semantic aggregate roots

Convert whole-copy compatibility structures to structurally shared/module-sharded representations.

Acceptance:

```text
canonical table entries copied
during workspace instantiation
    == 0
```

and body-only edits replace only affected shards.

---

## Program P7 — Root-driven compiler metadata

Implement explicit metadata export plan and reachability.

Add owner indexes.

Integrate canonical metadata pool references where necessary.

Acceptance:

```text
metadata work proportional to
runtime/reflection roots,
not complete semantic universe
```

---

## Program P8 — Local semantic/compiler algorithms

Address:

```text
source-index collector duplication
conformance indexing
witness indexing
proof memoization
module-value rechecking
source-site lookup
advisory selection
lowering reference lookup
conditional-impl indexing
```

This package targets residual latency and large-source scaling.

---

# 56. Dependency ordering

```text
P0 measurements
    │
    ▼
P1 reconstruction hygiene
    │
    ▼
P2 canonical formal image
    │
    ▼
P3 canonical DB/session base
    │
    ▼
P4 module/compiler cutover
    │
    ├───────────────┐
    ▼               ▼
P5 presentation   P6 persistent roots
    │               │
    └──────┬────────┘
           ▼
P7 metadata/export
           │
           ▼
P8 local algorithms
```

P1 can land independently.

P2–P4 form the central architectural checkpoint.

P5–P8 can then become increasingly independent.

---

# 57. Differential correctness suite

Do not use runtime-output tests alone.

Compare the existing source-seeded path and image-backed path.

Required cases include:

| Scenario | Required parity |
|---|---|
| local type conforms to canonical trait | same evidence/witness |
| canonical guarded rule applies to local type | same proof |
| local conformance overlaps canonical generic head | same coherence failure |
| canonical type conforms to locally owned trait | same authorization/proof |
| illegal foreign conformance | same diagnostic |
| local subclass of canonical class | same subtype relation |
| local generic subclass of canonical generic class | same specialization |
| canonical trait default used from local conformance | same witness/call |
| local code invokes canonical generic member | same specialization |
| local static value depends on canonical static product | same value |
| canonical associated type used locally | same projection |
| canonical static domain/case used locally | same semantics |
| noncanonical test Universe provider | isolated semantic world |
| canonical image TypeStore mismatch | fail closed |
| static-value lineage mismatch | fail closed |

Compare:

```text
diagnostics
declarations
signatures
hierarchy
dispatch
inferred returns
conformance evidence
witness plans
derived conformance proofs
associated types
static semantics
formal results
lowering-relevant semantic selections
runtime result
```

Ignore literal equality only for intentionally world/session-local identities such as:

```text
WorkspaceId
SnapshotId
TypeStoreId
```

when semantic equivalence under the correct prefix mapping is established.

---

# 58. Instrumentation invariants

Add counters that answer architectural questions directly.

```text
canonical_image.build_count
canonical_image.build_time

resolver.canonical_interface_builds
resolver.universe_catalog_builds

semantic.workspace_modules_deep_analyzed
semantic.canonical_modules_deep_analyzed

semantic.canonical_query_republications

source_index.canonical_modules_rebuilt
source_index.workspace_modules_rebuilt
source_index.target_collector_walks

advisory.outer_iterations
advisory.canonical_modules_processed
advisory.workspace_modules_processed
advisory.callable_body_visits
advisory.solver_steps
advisory.solver_requeues

formal.canonical_modules_rebuilt
formal.workspace_modules_rebuilt

conformance.canonical_heads
conformance.workspace_heads
conformance.canonical_canonical_overlap_checks
conformance.local_canonical_overlap_checks
conformance.local_local_overlap_checks
conformance.exact_candidate_probes

snapshot.full_map_materializations
snapshot.canonical_entries_copied

metadata.declarations_considered
metadata.callables_considered
metadata.declaration_callable_probes
metadata.runtime_roots
metadata.reflection_roots
metadata.canonical_refs
```

After the central cutover, structural assertions should include:

```text
canonical semantic source modules analyzed = 0
canonical query republications             = 0
canonical source-index rebuilds            = 0
canonical advisory solves                  = 0
canonical formal rebuilds                  = 0
canonical/canonical coherence rechecks     = 0
```

After persistent-root work:

```text
canonical table entries copied             = 0
```

---

# 59. Benchmark matrix

Always maintain at least these benchmark classes.

## Cold one-shot tiny source

Tests fixed floor.

---

## Cold one-shot medium source

Tests local semantic proportionality.

---

## Cold one-shot large source

Tests scaling after the fixed floor disappears.

---

## Persistent no-op recompile

Expected:

```text
near-zero semantic recomputation
```

---

## Persistent body-only edit

Should avoid:

```text
module relink
declaration rebuild
unrelated signatures
unrelated bodies
global advisory
```

---

## Signature edit

Should invalidate the exact dependent closure.

---

## Conformance edit

Tests:

```text
local/canonical coherence
trait-dispatch invalidation
witness dependencies
```

---

## Project graph/import edit

Tests module-layer invalidation independently from semantic body work.

---

# 60. What not to do

This section is mandatory.

---

## Do not treat `opt-level=3` as the fix

Use it as a diagnostic experiment.

The compiler should not require optimization merely to avoid traversing hundreds of canonical modules for a 300-byte source file.

---

## Do not merely cache final compiled fixture programs

That would hide the architectural issue and provide little benefit for novel user programs.

Cache canonical reusable semantic products instead.

---

## Do not optimize TypeStore deep copying

The dominant whole-store-copy hypothesis has already been falsified.

Preserve its persistent prefix architecture and add explicit lineage correctness.

---

## Do not remove the 40 canonical source modules before the image is semantically complete

The current partial baseline does not contain all required trait/conformance/body/static/query products.

Premature deletion will create semantic regressions.

---

## Do not add another large family of `base_*` fields

The complete `SemanticSnapshot` already owns the semantic product set.

Use one canonical image root.

---

## Do not set the canonical image snapshot as a workspace's previous snapshot

That confuses dependency semantic availability with editable workspace source lifecycle.

---

## Do not seed workspace `sources` or `semantic_structure_shards` with canonical Universe modules

That recreates whole-workspace scheduling.

---

## Do not implement local-first conformance precedence

Canonical and local conformance candidates form one coherence domain.

---

## Do not flatten canonical + local semantic tables on every update merely to preserve APIs

A temporary compatibility copy is acceptable for the first parity patch.

It is not the final architecture.

Move aggregates to persistent/module-sharded storage afterward.

---

## Do not rewrite the entire checker around new layered traits in the first image patch

Concrete table APIs are pervasive.

First fix ownership/lifetime.

Then optimize internal storage.

---

## Do not treat presentation/LSP products as mandatory executable-source work

Canonical presentation products can be image-owned.

Workspace presentation work should be local/incremental.

---

## Do not silently skip canonical dependency validation

Replace current Universe-specific shortcuts with canonical query-image currentness.

---

## Do not mutate the process-global image for test Universe overrides

A noncanonical provider gets its own private semantic world/image.

---

## Do not conflate the semantic image with a frozen VM runtime image

These are different architectural problems.

Semantic image:

```text
types
signatures
traits
conformances
query products
source products
static semantic values
```

Runtime VM image:

```text
heap identity
GC roots
closures
Rc<Callable>
VM-local caches
resources
fibers
mutable runtime objects
```

Solve them separately.

---

## Do not serialize the semantic image to disk first

Begin with:

```rust
OnceLock<Arc<CanonicalUniverseFormalImage>>
```

inside the process.

Persistence/versioning can follow once the in-memory ownership model is correct.

---

## Do not weaken `RuntimePublic` reflection behavior to obtain speed

Make root selection correct.

Do not silently change language-observable metadata semantics.

---

## Do not weaken diagnostics to satisfy parity

If image-backed and source-backed analysis disagree, investigate the missing product/invariant.

---

# 61. Future compiled Universe artifact

The in-memory image is deliberately the precursor to a persisted compiled semantic artifact.

Correct progression:

```text
Phase 1

Universe source
    ↓
full semantic publication
    ↓
CanonicalUniverseFormalImage
    ↓
workspace overlays
```

Then:

```text
Phase 2

CanonicalUniverseFormalImage
    ↓
versioned serialization format
    ↓
Universe semantic artifact
```

Then:

```text
Phase 3

toolchain startup
    ↓
validate/load artifact
    ↓
zero source bootstrap
    ↓
workspace overlays
```

Do not design persisted `TypeId` stability, artifact versioning, or cross-build invalidation until the in-memory semantic image boundaries are proven.

---

# 62. Expected performance model after remediation

Current tiny compile approximately behaves like:

```text
T =
  canonical module/interface reconstruction
+ canonical semantic source work
+ canonical conformance/body settlement
+ whole-Universe presentation indexing
+ advisory fixed point
+ formal projection
+ snapshot materialization
+ whole-snapshot metadata
+ local fixture work
```

Target:

```text
T =
  O(1) canonical image access
+ workspace module resolution
+ workspace semantic work
+ workspace presentation delta where requested
+ root-driven metadata
+ runtime lowering
```

The fixed cost should approach:

```text
small image/session initialization
+
actual local compilation work
```

rather than scaling with canonical Universe size.

---

# 63. Current priority ranking

## Tier A — fixed-floor architecture

1. Canonical Universe modules treated as ordinary workspace semantic source.
2. Cold source-index expansion across essentially the whole Universe.
3. Missing full reusable canonical semantic image.
4. Fresh DB requiring canonical product republication.
5. Resolver bypassing canonical interface cache.
6. Whole-snapshot metadata traversal.
7. Canonical advisory/formal recomputation.

---

## Tier A/B — multiplicative semantic work

8. Late module-value second body analysis.
9. Conformance witness settlement.
10. Inferred-return fixed point.
11. Exact conformance proof scans.
12. Pairwise coherence.

---

## Tier B — secondary reconstruction/scaling

13. `SemanticWorkspaceInput` discarded interfaces.
14. Duplicate formal projection.
15. snapshot whole-map materializations.
16. source structural fingerprint reconstruction.
17. range/occurrence nested scans.
18. lowering global filtered scans.

---

# 64. Recommended first implementation milestone

The first meaningful milestone should be:

> **A complete canonical semantic image can be built once using the ordinary semantic pipeline, `phalcom-core` consumes that image for canonical Universe compilation, and a fresh workspace can read canonical query products without republishing them.**

Do not yet remove the 40 source modules until this milestone passes differential semantic tests.

The second milestone is:

> **Workspace compilation no longer contains canonical Universe source modules and no canonical formal/advisory/source-index work occurs per fixture.**

The third milestone is:

> **Workspace creation/update no longer copies canonical aggregate tables.**

Only then optimize the smaller local algorithms.

---

# 65. Suggested fresh-agent execution order

A fresh implementer should begin with repository confirmation rather than immediately editing.

### Step 1 — Verify the current architecture

Search current source for:

```text
compile_entry_selection
ProgramCompilationSession
SemanticWorkspaceSession
CanonicalUniverseSemanticBaseline
canonical_universe_program
["collections", ..]
["scalar", ..]
["text", ..]
["option", "unit"]
build_source_semantic_index
build_advisory_workspace
FormalSemanticProjection
MetadataExporter
RuntimePublic
```

Confirm whether any of the investigated paths changed.

---

### Step 2 — Reproduce counters/timings

Do not rely on historical 45-second timing as the current baseline.

Capture exact current revision and configuration.

---

### Step 3 — Land reconstruction hygiene

Resolver/interface/input/projection fixes.

Keep these small and separately reviewable.

---

### Step 4 — Implement image construction behind an unused API

No production cutover.

Prove the image can compile the canonical Universe itself.

---

### Step 5 — Freeze and integrate canonical query products

Add `is_current` and product-origin semantics.

Write focused unit tests before changing one-shot compilation.

---

### Step 6 — Add image-backed workspace mode behind a test seam

Compile the same fixtures through:

```text
old seeded source mode
image-backed mode
```

and compare semantic products.

---

### Step 7 — Resolve every parity difference

Do not paper over them.

Classify each as:

```text
missing canonical image product
incorrect arena lineage
incorrect DB currentness
cross-layer conformance bug
module/import ownership bug
presentation-only difference
actual pre-existing bug
```

---

### Step 8 — Remove source seeding

Only after parity.

Immediately remeasure cold tiny fixtures.

This is the most important performance checkpoint.

---

### Step 9 — Move canonical presentation/advisory/formal products to reusable roots

Measure again.

---

### Step 10 — Convert flat semantic aggregates to persistent module roots

Measure session initialization and snapshot publication separately.

---

### Step 11 — Repair metadata export

Do not begin by optimizing bytecode lowering.

---

### Step 12 — Execute local algorithmic package

At this point profiles should identify the true residual hot paths rather than canonical reconstruction noise.

---

# 66. Success criteria

The program is not complete merely when one benchmark becomes “fast.”

A successful architecture should demonstrate all of the following:

```text
canonical Universe semantic analysis
    once per canonical semantic world

canonical interfaces
    reused

canonical query products
    immutable/read-through

workspace source set
    excludes canonical modules

workspace deep analysis
    proportional to workspace dependency closure

canonical source/presentation shards
    reused

workspace snapshot roots
    structurally shared

conformance semantics
    correct across canonical/local boundary

metadata export
    root/profile-driven

one-shot and persistent compilation
    use the same compiler architecture

test-profile optimization level
    changes multiplier, not algorithmic work counts
```

Performance counters should prove that a tiny one-shot fixture performs **zero canonical semantic recomputation**.

---

# 67. Final architectural thesis

The key issue is not that Telos semantics are intrinsically expensive.

The repository has repeatedly implemented the right reusable abstractions, but the abstractions currently terminate at different layers:

```text
modules cache canonical parsing/interfaces,
but resolver reconstructs some interfaces;

semantics has a canonical baseline,
but it ends before complete formal semantics;

SemanticDb supports incrementality,
but starts empty for canonical facts;

source index/formal projection have persistent shards,
but cold publication rebuilds canonical presentation;

compiler lowering is runtime-local,
but metadata projection scans global semantic tables.
```

The architectural solution is to **complete the lifetime chain**:

```text
canonical source
    ↓ once
canonical module products
    ↓ once
canonical formal semantics
    ↓ once
canonical query products
    ↓ immutable base
workspace-local semantic overlay
    ↓ explicit executable/reflection roots
compiler projection
```

Once this chain is preserved, the source-size-independent ~45-second floor should have no architectural reason to exist.

The implementation agent should therefore optimize in this order:

```text
OWNERSHIP
    ↓
REUSE
    ↓
INCREMENTAL BOUNDARIES
    ↓
PERSISTENT DATA STRUCTURES
    ↓
ALGORITHMIC INDEXING
    ↓
MICRO-OPTIMIZATION
```

Reversing that order risks spending substantial effort accelerating work the compiler should not be performing at all.