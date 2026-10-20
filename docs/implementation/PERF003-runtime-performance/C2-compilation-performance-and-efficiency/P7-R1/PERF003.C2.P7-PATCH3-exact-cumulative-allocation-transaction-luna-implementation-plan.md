---
id: PERF003.C2.P7.PATCH3
category: PERF
program: PERF003
checkpoint: PERF003.C2
parent_plan: PERF003.C2.P7
kind: corrective-allocation-closure-implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
target_branch: perf/universe-characterization
prepared_against_revision: d26de3a2789e9f6dd6829f7d71388fa4fcc801c6
audit_baseline_revision: 2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a
patch0_implementation_revision: 8284893769d121dc2fe33ed0c866a402ff48bd35
executor_profile: GPT-5.6 Luna High / Extra High
---

# PERF003.C2.P7.PATCH3 — Close the Exact Cumulative Artifact-Load Allocation Transaction

> **Execution role:** patch-grade implementation plan for P7 allocation/resource correctness.  
> **Primary invariant:** one whole-load cumulative ledger of allocator-requested owned allocation bytes, with independent stage/section caps, precharge-before-request, child rollback, and zero hidden native allocation in the persisted-image load path.  
> **Guard policy:** semantic validation/import `UnaccountedBoundary` barriers remain fail-closed until their respective accounting proofs pass.  
> **Semantic scope:** PATCH3 changes physical materialization/accounting, not semantic authority. PATCH1/PATCH2 own missing semantic owners; PATCH4 owns query authority semantics; PATCH5 owns canonical container bytes; PATCH7 owns final release acceptance.

---

# 0. Executive directive

PATCH3 exists because the P7 loader currently **claims a cumulative allocation limit that is not yet true for all native work**.

The loader already has useful exact-budget foundations:

```text
AllocationBudget / ChildAllocationBudget
precharged Vec growth
precharged owned strings
precharged Box<T>
precharged Arc<T>
BudgetedSortedMap / BudgetedSortedSet
budgeted container directory
budgeted O(V + E) query-closure scratch
budgeted frozen FlowGraph
budgeted/frozen callable AnalysisMap
budgeted/frozen CallableAnalysisTable
final LoadedUniverseArtifact Arc charge
```

But the current branch still contains allocation leaks:

```text
MODULE_WORLD native materialization
    ordinary BTreeMap/BTreeSet
    owned name/string clones
    nested module products

SEMANTIC_WORLD validation
    unconditional UnaccountedBoundary
    component validators with uncharged scratch/errors

TypeStore import
    no AllocationBudget parameter
    native maps/sets/vectors
    local estimated-byte counter in scoped validation

semantic owner materialization
    remaining source-mutable native map owners
    incomplete frozen imported backing

QUERY_IMAGE reconstruction
    serde_json owner decoding
    format!/String errors
    Arc::new / Arc::from
    ordinary BTreeMap output authority
    cloned products

canonical query/formal image
    native BTreeMap/BTreeSet scratch/authority
    dynamic formatted errors
    unbudgeted CanonicalUniverseFormalImage Arc

core load error mapping
    ArtifactError repeatedly converted with e.to_string()
```

PATCH3 must **not** solve this by changing the contract.

The fixed contract is:

```text
for every owned allocation request attributable to one load:
    debit exact requested layout before issuing request

if debit fails:
    do not issue the allocation request

if allocator request itself fails:
    return typed allocation failure
    restore the helper/stage-local debit

if a stage fails:
    do not commit its child ledger to the parent

if a stage succeeds:
    commit all requested bytes, including temporary scratch

no refunds within a successful load
```

This tracks **allocator-requested bytes**, not RSS, allocator usable capacity, logical object size, encoded bytes, or retained bytes.

---

# 1. Repository grounding

## 1.1 Current branch

```text
repository: aureat/phalcom-lang
branch: perf/universe-characterization
HEAD: d26de3a2789e9f6dd6829f7d71388fa4fcc801c6
commit: feat(perf): add callable query owner identity
```

The branch is materially ahead of the October 15 audit baseline.

After PATCH0 implementation `8284893769d121dc2fe33ed0c866a402ff48bd35`, current history includes work that freezes the callable owner graph, bounds callable validation rows, completes more root coverage, adds owner rows, and introduces callable query-owner identity.

This plan is therefore based on the **current branch state**, not a frozen reproduction of `2939e9d`.

---

## 1.2 PATCH0 status

PATCH0 is:

```text
COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

It changed semantic fingerprints only and deliberately left the allocation policy/guards unchanged.

PATCH3 must preserve PATCH0.

---

# 2. Current assets to preserve and reuse

## 2.1 `AllocationBudget`

`phalcom-type-meta/src/allocation.rs` already defines the correct basic semantics:

```rust
pub struct AllocationBudget {
    remaining: usize,
    requested: usize,
}

pub struct ChildAllocationBudget {
    budget: AllocationBudget,
}
```

with exact helpers for:

```text
reserve_bytes / reserve_layout / reserve_array
vector
grow_vector_exact
string
boxed_str
boxed
arc
sorted_map_builder
sorted_set_builder
```

Growth already charges the **complete replacement request**, not the capacity delta.

That is correct.

---

## 2.2 Budgeted immutable ordered storage

Approved physical representation:

```text
BudgetedSortedMap<K,V> -> Vec<(K,V)>
BudgetedSortedSet<K>   -> Vec<K>
```

with:

```text
sort_unstable
adjacent duplicate validation
binary-search lookup
bounded range
no hidden native B-tree node allocations
```

Do not replace this with a guessed `BTreeMap` charge.

---

## 2.3 PATCH1 callable physical backing

Current branch already has the desired source/import split:

```text
AnalysisMap<K,V>
    Source(BTreeMap<K,V>)
    Frozen(Arc<BudgetedSortedMap<K,V>>)

CallableAnalysisTable
    Source(HashMap + BTreeSet module index)
    Frozen(charged sorted rows + charged module-range index)
```

PATCH3 should extend this pattern to other imported authority.

Do not create a second competing frozen-map abstraction for callable products.

---

## 2.4 Query closure validation

Current:

```text
validate_query_image_closure_with_allocation_budget
```

already uses:

```text
budgeted borrowed-key Vec
budgeted colors
budgeted iterative DFS stack
O(V + E) closure validation
child transaction
```

Preserve this improvement.

PATCH3 query work is now primarily:

```text
owner decoding
new product ownership
dependency edge storage
persistent root/fingerprint maps
final query image publication
error allocation
```

---

# 3. Source-proven live defects

## 3.1 Module materialization remains an immediate allocation violation

Current:

```rust
load_module_world_with_allocation_budget(...)
    -> transaction.child(...)
    -> load_module_world_inner(...)
```

is good staging.

But `load_module_world_inner` still builds native:

```rust
let mut interfaces = BTreeMap::new();
let mut modules = BTreeMap::new();
let mut graphs = ModuleGraphs::default();
let mut nodes = BTreeMap::new();
let mut children = BTreeMap::new();
let mut exposed = BTreeMap::new();
```

plus nested:

```text
declaration maps
export maps
exposed-child sets
linked-export maps
global/import binding maps
graph maps
owned strings
ModuleComponent(Box<str>)
metadata clones
```

These allocations occur **before** semantic guards.

This is PATCH3's first production priority.

---

## 3.2 Module allocation failures are still converted to `String`

Current module materialization contains the equivalent of:

```rust
ArtifactError::LimitExceeded(message.into())
```

for `AllocationError`.

This:

```text
erases the typed cause
allocates an error String
can allocate after budget exhaustion
```

Repair early.

---

## 3.3 Core loader still stringifies lower errors

Current core load code repeatedly does:

```rust
CanonicalImageError::IncompletePublication(e.to_string())
```

for container/module/semantic/query/metadata stages.

That makes repaired typed lower errors ineffective.

PATCH3 requires a structured mapper that preserves:

```rust
ArtifactError::Allocation(AllocationError)
```

without heap formatting inside the constrained transaction.

---

## 3.4 Semantic validator remains correctly fail-closed

Current:

```rust
SemanticWorldV2::validate_with_allocation_budget(...)
```

still calls:

```rust
require_semantic_validation_allocation_accounting()?;
```

which returns:

```rust
AllocationError::UnaccountedBoundary
```

The guard is correct.

Remove it only after validator exact-accounting proof.

---

## 3.5 Semantic world allocation errors are still misclassified

Current `world.rs` converts allocation errors through an `invalid(...)` helper.

PATCH3 requires direct:

```rust
ArtifactError::Allocation(error)
```

or an equivalent allocation-free structured propagation path.

---

## 3.6 Semantic import remains correctly fail-closed

Current:

```rust
validate_and_import_semantic_world_with_allocation_budget(...)
```

still calls:

```rust
require_semantic_import_allocation_accounting()?;
```

before inner materialization.

The inner path still begins with:

```rust
TypeStore::import_type_graph(&world.type_graph, limits)?
```

without the shared transaction.

The import guard must remain until this and all reachable semantic owner construction is closed.

---

## 3.7 TypeStore import is not transaction-aware

Current type import owns native:

```text
store vectors
parameter-to-ID index
parameter-variance index
relocation vectors
uniqueness BTreeSets
owner-slot BTreeSet
other interning/index state
```

without P7 accounting.

`validate_scoped_roots` additionally uses:

```rust
seen: BTreeSet<(ScopedTypeId, Vec<usize>)>
context.to_vec()
Vec::with_capacity(...)
```

and a local estimated `remaining_bytes`.

That local estimate may remain as an independent DoS/work limit.

It is **not** allocation accounting.

---

## 3.8 Query reconstruction remains open

Current V1 owner decoding uses:

```rust
let prefix = format!("{tag}:");
serde_json::from_str(payload)
```

Current product reconstruction creates ordinary:

```text
Arc::new
BTreeMap
String/path objects
Vec collections
deep/native owner clones
```

Current final query reconstruction uses:

```rust
let mut entries = BTreeMap::new();
let mut fingerprints = BTreeMap::new();
Arc::from(edges.into_boxed_slice())
```

These are unclosed.

---

## 3.9 Canonical query image is native-map-backed

Current:

```rust
pub struct CanonicalSemanticQueryImage {
    entries: BTreeMap<QueryKey, CanonicalQueryEntry>,
    ...
}
```

and artifact-loaded validation builds additional:

```text
BTreeSet pending
Vec leaves
format! diagnostics
```

PATCH3 must make artifact-loaded query authority physically frozen/budgeted.

---

## 3.10 Final formal image is not allocation-aware

Current:

```rust
CanonicalUniverseFormalImage::from_loaded_artifact(...)
```

validates and finally calls:

```rust
Arc::new(Self { ... })
```

without the root allocation transaction.

It also stores:

```rust
dependency_fingerprints: BTreeMap<QueryKey, ProductFingerprint>
```

as native ordered authority.

PATCH3 must close this final boundary.

---

# 4. Non-negotiable allocation semantics

## 4.1 What counts

A successful load's cumulative requested bytes include owned requests for:

```text
disk file buffer
container package strings/directory
DTO arrays/strings
module native authority
type/static relocation
semantic owner materialization
validation scratch
query owner decoding
query products/edges/indexes
metadata native products
snapshot/index ownership
formal image publication
LoadedUniverseArtifact publication
temporary scratch
replacement buffers during growth
```

If a Vec requests a 4096-byte replacement block, charge 4096 bytes, not the capacity delta.

---

## 4.2 What does not count

```text
deallocation
borrowed input bytes for load_universe_telosc_from_bytes
stack storage
allocator-internal usable-size rounding
RSS
logical bytes not requested from allocator
Arc::clone
borrows
```

`load_universe_telosc(path, ...)` owns its file buffer and charges it.

`load_universe_telosc_from_bytes(&[u8], ...)` borrows input and does not charge the input bytes themselves.

Document the expected total difference.

---

## 4.3 Scratch is not refunded on successful load

Temporary allocations count even if dropped before publication.

The budget is cumulative requested bytes, not retained bytes.

---

## 4.4 Failed stage rollback

Every multi-step fallible stage uses a child transaction.

Failure means:

```text
parent.requested unchanged
parent.remaining unchanged
no partial publication
```

---

## 4.5 Budget rejection precedes allocator request

If:

```text
remaining = N - 1
next request = N
```

return:

```text
AllocationError::Exhausted
```

without issuing the allocator request.

The test allocator must independently prove this.

---

# 5. Patch ownership boundaries

PATCH3 owns:

```text
AllocationBudget helpers
stage transaction discipline
allocation-safe typed errors
module native materialization backing
TypeStore artifact import accounting
semantic validation accounting
semantic native materialization accounting
frozen imported backing needed by allocation
query physical allocation
metadata allocation verification
formal-image publication allocation
counting-allocator proof
allocation guard removal
```

PATCH3 does not own:

```text
callable semantic completeness             -> PATCH1
remaining trait/static/definition semantics -> PATCH2
query dependency-evidence semantics         -> PATCH4
query persistence/disposition policy        -> PATCH4
strict binary canonicality                  -> PATCH5
final package/performance release           -> PATCH7
```

Never erase semantic fields to make an owner easier to allocate.

---

# 6. Target stage architecture

```text
root AllocationBudget(max_total_allocation_bytes)
│
├─ FILE ACQUISITION child
│
├─ CONTAINER child
│
├─ MODULE DTO child
│
├─ MODULE MATERIALIZATION child
│
├─ SEMANTIC DTO + VALIDATION child
│
├─ SEMANTIC MATERIALIZATION child
│    ├─ TypeStore
│    ├─ static graph
│    └─ semantic owners/indexes
│
├─ QUERY DTO child
│
├─ QUERY MATERIALIZATION child
│
├─ METADATA child
│
├─ FORMAL IMAGE PUBLICATION child
│
└─ LOADED ARTIFACT PUBLICATION child
```

Each child has an independent stage/section cap plus the remaining whole-load cap.

No whole-load reset occurs between stages.

---

# 7. Task sequence

# T0 — Takeover, provenance, and live allocation census

Run:

```bash
git branch --show-current
git rev-parse HEAD
git status --short
git log -12 --oneline
```

Record concurrent PATCH1/PATCH2 state.

Build a durable census with columns:

```text
stage
entry function
owned allocation sites
allocation-free operations
budget-aware operations
unaccounted operations
local cap
parent/child behavior
counting-allocator test
status
```

Mandatory rows:

```text
disk file read
container parse
MODULE_WORLD DTO decode
module native materialization
SEMANTIC_WORLD DTO decode
semantic validation
TypeStore import
static import
callable import
trait/witness import
other semantic import
QUERY_IMAGE DTO decode
query closure validation
query owner decoding
query product construction
query persistent storage
metadata DTO/bundle
canonical query image
canonical formal image
LoadedUniverseArtifact
```

No row may be marked accounted merely because its entry function accepts `AllocationBudget`.

---

## T0 baseline commands

Verify target names before execution:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-type-meta \
  --test allocation_budgeted_map -- --test-threads=1

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-compiler-artifact \
  --test container_allocation_test -- --test-threads=1

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-compiler-artifact \
  --test query_allocation_test -- --test-threads=1

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic \
  --test artifact-allocation -- --test-threads=1
```

Zero selected tests is failure to establish evidence.

---

# T1 — Extend exact allocation primitives only where needed

File:

```text
phalcom-type-meta/src/allocation.rs
```

Likely missing helper categories:

```text
exact shared Arc slice
exact boxed slice where unavoidable
budgeted immutable Vec owner handoff
budgeted bitset/dense scratch
budgeted work queue
budgeted copied nested identity/string
frozen map from already sorted entries
```

Do not add abstractions without a live caller.

---

## T1 conversion audit

Every load-path use of:

```rust
into_boxed_slice()
Arc::from(...)
String::into_boxed_str()
Vec::from(...)
collect()
to_owned()
clone()
```

must be classified:

```text
A. proven allocation-free ownership move
B. explicitly precharged fallible allocation
C. replaced
D. unresolved -> guard remains
```

Never assume a standard-library conversion is allocation-free without proof.

---

## T1 helper tests

For every new helper:

```text
observed allocator requests == ledger requested
exact N succeeds
N - 1 rejects before allocator request
allocator failure restores helper debit
layout overflow returns Overflow before request
```

---

# T2 — Preserve typed allocation errors end to end

This should land before large representation work.

## T2.1 Module and semantic lower layers

Replace:

```rust
ArtifactError::LimitExceeded("...".into())
```

for allocation failures with:

```rust
ArtifactError::Allocation(error)
```

at minimum in:

```text
phalcom-modules/src/compiler_artifact.rs
phalcom-compiler-artifact/src/semantic/world.rs
```

Preserve current typed query-decoder behavior.

---

## T2.2 Core mapping

Remove `e.to_string()` from the constrained load path.

Use a typed mapping seam:

```rust
fn map_artifact_load_error(
    error: ArtifactError,
    total_limit: usize,
) -> TeloscArtifactError {
    match error {
        ArtifactError::Allocation(error) =>
            map_allocation_error(error, total_limit),
        other =>
            TeloscArtifactError::Artifact(other),
    }
}
```

Exact enum naming is flexible.

The property is not.

---

## T2.3 Allocation-free validation failure representation

Audit:

```text
format!
to_string()
String::from
"...".into() into String error payload
ReferentialIntegrity(String)
IncompletePublication(String)
```

inside measured stages.

Prefer structured allocation-free error variants with scalar/enum payloads.

Formatting belongs at the presentation boundary after the constrained operation returns.

---

## T2 acceptance

Force `Exhausted` at representative module/semantic/query layers and assert:

```text
typed cause survives
no additional allocator request is caused by error translation
```

---

# T3 — Close native module-world materialization

This is the first major production representation repair.

## T3.1 Forbidden approach

Do not precharge native B-tree node estimates:

```text
entries * size_of::<(K,V)>()
private std node constants
guessed overhead factor
```

---

## T3.2 Introduce artifact-frozen module authority

Use the callable/flow precedent.

Conceptual:

```rust
enum CanonicalMapStorage<K,V> {
    Source(BTreeMap<K,V>),
    Frozen(Arc<BudgetedSortedMap<K,V>>),
}
```

or owner-specific equivalent wrappers.

Compiler consumers receive only:

```text
get
contains_key
iter
keys
values
len
is_empty
range when required
```

Source-side mutation remains source-side.

---

## T3.3 Migrate the complete nested module graph

Audit and convert artifact-created ownership for:

```text
CanonicalExternalModuleWorld.interfaces
LinkedProgram.modules
ModuleTopology.nodes
ModuleTopology.children
ModuleGraphs reference rows
ModuleGraphs semantic rows
ModuleGraphs runtime rows
UnlinkedModuleInterface.declarations
UnlinkedModuleInterface.exports
UnlinkedModuleInterface.exposed_children
LinkedModuleInterface.exports
ModuleBindingLayout.local_globals
ModuleBindingLayout.imports
metadata attribute ownership
runtime-dependency arrays
linked-read arrays
```

An outer frozen map with native maps inside is not closure.

---

## T3.4 Strings and module identities

Current `ModuleComponent::from_identifier` creates `Box<str>` and its error cases can allocate strings.

Introduce an artifact-load validated constructor that:

```text
validates borrowed spelling allocation-free
allocates final Box<str> through AllocationBudget
```

Audit every:

```text
String clone
Box<str> clone
ModulePath clone
ModuleId clone
symbol/name clone
import path clone
metadata clone
```

A charged outer pair does not pay for nested strings.

---

## T3.5 Prefer ownership transfer where safe

If the DTO already owns a final-compatible allocation, consume/move rather than clone.

But do not rely on hidden shrinking/reallocation from:

```text
String -> Box<str>
Vec -> Box<[T]>
```

without exact proof.

---

## T3.6 Graph validation

Preserve budgeted dense runtime-DAG scratch.

Prefer dense module-index arrays/bitsets/worklists for other validation.

Portable module IDs are dense; exploit them.

---

## T3 tests

Create an isolated module materialization allocation test using:

```text
many modules
declarations
exports/re-exports
imports
linked reads
globals/import bindings
topology children
package exposure
long names
metadata
reference/semantic/runtime edges
```

Required:

```text
observed == stage ledger
N succeeds
N - 1 Exhausted before unmatched request
malformed/duplicate input does not commit parent
retry succeeds
source/frozen module reads match
fingerprints match
runtime DAG match
```


# T4 — Make `TypeStore::import_type_graph` transaction-aware

## T4.1 New shared-budget API

Introduce:

```rust
TypeStore::import_type_graph_with_allocation_budget(
    graph,
    limits,
    transaction,
)
```

Keep the existing nonshared API only as a local-budget wrapper if useful.

The P7 loader must call the shared transaction path.

---

## T4.2 Full TypeStore allocation census

Account at minimum:

```text
type nodes
kind nodes
type parameters
record rows
families
variants
static domains
static constructors
scoped types
type lambdas

all relocation arrays

current parameter owner/index table
parameter variance index
content/interning indexes
owned declaration/selector/string payloads
nested argument arrays
boxed payloads
scope-validation scratch
uniqueness scratch
```

Do not stop after relocation vectors.

---

## T4.3 Imported canonical TypeStore architecture

The loaded canonical TypeStore is immutable baseline authority.

Preferred shape:

```text
canonical dense value arrays
    immutable, budgeted

canonical lookup/interner indexes
    frozen sorted/dense storage

local workspace fork
    immutable canonical prefix
    mutable user-local overlay
```

Do not reconstruct a huge source-mutable hash/table solely so user compilation can intern later types.

`fork_from_baseline` should preserve the canonical parent/prefix and allocate only user-local overlays.

This protects P5/P6 locality.

---

## T4.4 Parameter indexes

Replace import-time native map mutation for:

```text
(owner, index) -> TypeParameterId
(owner, index) -> Variance
```

with budgeted sorted/dense final indexes.

Validate duplicates with adjacent scans.

Do not clone the frozen canonical index into a mutable one when creating a user session.

---

## T4.5 Uniqueness validation

Current `collect::<BTreeSet<_>>()` uniqueness checks must leave the loaded path.

Use:

```text
budgeted Vec of relocated IDs
sort_unstable
adjacent duplicate scan
```

or a dense bitset where the ID range is exact.

No B-tree nodes.

---

## T4.6 Replace scoped-root estimated accounting

Current design:

```rust
seen: BTreeSet<(ScopedTypeId, Vec<usize>)>
context.to_vec()
Vec::with_capacity(context.len() + 1)
remaining_bytes -= estimated_amount
```

must not remain the memory accounting mechanism.

Keep structural work/depth limits separately.

Preferred compact model:

```rust
struct BinderFrame {
    parent: Option<u32>,
    width: u32,
}

struct ScopedValidationState {
    scoped: ScopedTypeId,
    frame: u32,
}
```

Store context frames once in a budgeted arena rather than cloning whole context vectors into every visited state.

For visited-state tracking prefer, in order:

```text
dense index / bitset if possible
budgeted open-addressed scratch backed by Vec
pre-reserved sorted scratch if complexity remains bounded
```

Do not reintroduce native `BTreeSet`.

---

## T4 tests

Required type fixtures:

```text
nested lambdas
deep binder references
multiple current parameter versions
variance overrides
record rows
family types
variants
static domains
constructors
scoped applications
duplicate relocated nodes
invalid binder references
large valid type graph
```

Required:

```text
observed requests == ledger
N success
N - 1 Exhausted before unmatched request
invalid scope rolls back
duplicate owner rolls back
fresh retry succeeds
canonical prefix/fork semantics preserved
```

---

# T5 — Close semantic-world validation allocation

This task removes the first global semantic allocation barrier only after proof.

Entry:

```rust
SemanticWorldV2::validate_with_allocation_budget
```

---

## T5.1 Validator census

Audit every delegate reachable from `validate_inner`:

```text
type graph
structural contracts
static values
checked definitions
witness plans
callable signatures
callable analyses
traits
dispatch surfaces
constructor/static/associated authority
duplicate owner checks
cross-owner reference checks
cycle/closure checks
```

Classify each:

```text
AllocationFree
BudgetAware
Unaccounted
```

No unknown rows at guard removal.

---

## T5.2 Validate wire arrays directly

Where wire records are canonically sorted:

```text
use adjacent scans
binary search
dense ordinals
```

rather than building maps/sets.

Temporary scratch is:

```text
budgeted Vec
budgeted bitset
budgeted stack
budgeted queue
budgeted borrowed-key vector
```

No hidden:

```text
BTreeMap
BTreeSet
HashMap
HashSet
collect::<Vec<_>>()
format!
```

---

## T5.3 Error paths

All validation failure construction inside this stage must be either:

```text
allocation-free structured error
```

or:

```text
explicitly budgeted
```

Prefer structured errors.

Do not allocate prose after exhaustion.

---

## T5.4 Guard removal

Remove:

```rust
require_semantic_validation_allocation_accounting()
```

only when a valid nonempty `SemanticWorldV2` proves:

```text
observed requests == validator child debit
exact N succeeds
N - 1 returns typed Exhausted before allocator request
malformed world does not commit child
```

This guard may be removed while semantic **import** remains guarded.

That is the intended staged rollout.

---

# T6 — Close semantic native materialization

This task follows PATCH1/PATCH2 owner-shape stabilization.

Do not finalize an imported physical backing for an owner whose semantic contract is unresolved.

---

## T6.1 Reuse callable frozen work

Use the current:

```text
AnalysisMap
CallableAnalysisTable::from_budgeted_entries
frozen FlowGraph
```

Do not add competing callable containers.

PATCH3's job is to certify and complete nested allocation coverage.

---

## T6.2 Owner families requiring allocation-safe imported storage

At minimum:

```text
declarations
hierarchy
aliases
callable signatures
accepted callable definitions
callable analyses
field signatures
data semantics
enum semantics
trait headers
trait surfaces
associated families
type-member declarations
conformance heads
witness plans
derived conformance rules
trait dispatch
constructor initialization
enum requirements
static case catalogs
static case signatures
static definitions
checked static bodies
static constants
module static bindings
formal projection
module-bounded owner indexes
```

An Arc around a struct containing uncharged maps is not closure.

---

## T6.3 Standard dual-storage pattern

Use owner-specific variants similar to:

```rust
enum OwnerStorage<K,V> {
    Source(...existing mutable backing...),
    Frozen(Arc<BudgetedSortedMap<K,V>>),
}
```

or dense indexed storage where identities are ordinals.

Consumers use one read contract.

Frozen imported authority is immutable.

---

## T6.4 Nested payload accounting

Charge the complete ownership graph:

```text
Arc owner
Vec fields
Box<[T]>
Box<str>
generic bindings
conformance evidence
trait member arrays
match/pattern arrays
diagnostic/explanation arrays
checked static application arguments
flow summaries
static operational rows
```

Outer buffer charging never covers nested ownership.

---

## T6.5 Snapshot assembly

Build `SemanticSnapshot` from already sealed owner tables.

Charge the final snapshot Arc.

Do not create detached debug indexes such as:

```text
BTreeMap<String, ...>
format!("{:?}", module)
```

PATCH2 owns replacing those with real typed indexes; PATCH3 certifies their allocation.

---

## T6 tests

Use family-specific allocation/parity fixtures first.

Each migrated family must prove:

```text
observed == ledger
N / N-1
failure rollback
source/import read parity
frozen mutation rejection where applicable
module-bounded iteration parity
fingerprint parity where defined
```

Then add aggregate semantic import proof.

---

## T6 guard removal

Remove:

```rust
require_semantic_import_allocation_accounting()
```

only after:

```text
TypeStore complete
static import complete
PATCH1 owner import complete
PATCH2 owner import complete
snapshot/index construction complete
error paths allocation-safe
aggregate semantic import exact proof passes
```

Do not remove it after only TypeStore closure.

---

# T7 — Close query reconstruction physical allocation

PATCH4 owns query semantics.

PATCH3 owns exact allocation of the resulting query authority.

---

## T7.1 Keep existing O(V + E) closure validator

Do not regress to global rescans or native set scratch.

---

## T7.2 Owner decoding

Current V1 owner strings + JSON allocate.

Acceptable coordination:

### Preferred

PATCH4 introduces structured stable query owners; PATCH3 accounts that structured decoder.

### Temporary V1 path

If JSON remains:

```text
avoid format!("{tag}:")
use borrowed prefix checks
thread AllocationBudget into owner decode
budget every String/Vec/stable-owner object
preserve typed malformed-owner errors
```

No uncontrolled `serde_json::from_str`.

---

## T7.3 Product construction

Change:

```rust
product_and_fingerprints(...)
```

to receive the allocation budget.

Prefer:

```text
Arc::clone existing canonical products
```

over deep copies.

When a new wrapper is required:

```text
charge wrapper Arc
charge nested arrays/maps/strings
```

---

## T7.4 Resolved-import query product

Current path builds:

```text
BTreeMap<String, ModuleId>
import.path().to_string()
```

This must become budgeted/frozen physical storage or reuse a stable canonical path identity without string materialization if PATCH4 permits.

Do not redesign its semantic owner in PATCH3.

---

## T7.5 Dependency edges

Do not use unexamined:

```rust
Arc::from(edges.into_boxed_slice())
```

Use an exact fallible shared-slice helper or another final representation whose requests are explicit.

---

## T7.6 Query entry storage

Artifact-loaded query entries must not end in:

```rust
BTreeMap<QueryKey, CanonicalQueryEntry>
```

unless there is an allocator-visible fallible exact construction seam.

Preferred:

```rust
enum CanonicalQueryEntries {
    Source(BTreeMap<QueryKey, CanonicalQueryEntry>),
    Frozen(Arc<BudgetedSortedMap<QueryKey, CanonicalQueryEntry>>),
}
```

Preserve reader APIs:

```text
entry
entries
len
is_empty
product
product_fingerprint
```

---

## T7.7 Dependency fingerprint storage

Apply the same source/frozen model to:

```text
BTreeMap<QueryKey, ProductFingerprint>
```

inside the canonical formal image.

Do not build a temporary budgeted vector and then copy into a B-tree.

---

## T7.8 Loaded-query validation

Current loaded query constructor allocates:

```text
BTreeSet pending
Vec leaves
format! errors
```

Validate before sealing using:

```text
dense dependency indices
budgeted colors/indegrees
already sorted root rows
structured errors
```

Do not revalidate the graph with a second hidden allocation algorithm.

---

## T7 tests

Fixture must include:

```text
mixed query owner kinds
hundreds/thousands of roots
high-degree root
long chain
many edges
query wrappers
type-member selector strings
resolved imports
```

Prove:

```text
owner decode charged
products charged
edges charged
root table charged
fingerprint table charged
query image Arc charged
observed == ledger
N / N-1
rollback
reader parity
```

PATCH4 separately proves dependency-evidence correctness.

---

# T8 — Metadata and final publication closure

## T8.1 Metadata

Audit and independently measure:

```text
MetadataSectionV1::decode_with_allocation_budget
decode_bundle_with_allocation_budget
nested token/container/string ownership
maps/lists
final metadata Arc
```

A budget parameter on the entry function is not proof.

---

## T8.2 Allocation-aware formal-image constructor

Introduce an artifact-specific constructor accepting the budget:

```rust
pub fn from_loaded_artifact_with_allocation_budget(
    world,
    modules,
    snapshot,
    queries,
    dependency_fingerprints,
    allocations,
) -> Result<Arc<Self>, CanonicalImageError>
```

Exact name flexible.

Required behavior:

```text
validation allocation-closed
dependency fingerprint backing already frozen
no hidden map conversion
final Arc charged
```

---

## T8.3 `build_sources`

Artifact-loaded image has no canonical source.

Empty source storage should allocate zero.

Do not charge fictitious bytes.

---

## T8.4 Outer loaded artifact

The current `LoadedUniverseArtifact` Arc already uses the budget helper.

Preserve it and verify it in the whole-load counter.

---

# T9 — Normalize stage child transactions

Every fallible shared-budget stage should look like:

```rust
let mut child = transaction.child(local_limit);

let value = stage_inner(..., &mut child)?;

transaction.commit_child(child)?;
Ok(value)
```

Avoid splitting one stage between parent and child without documented reason.

---

## Recommended local caps

| Stage | Local cap |
|---|---|
| disk file acquisition | `max_artifact_bytes` and whole-load remaining |
| container parse | container/artifact-derived local bound |
| section DTO decode | `max_decoded_section_bytes` |
| module materialization | `max_decoded_section_bytes` until dedicated cap exists |
| semantic validation | `max_decoded_section_bytes` |
| semantic materialization | `max_decoded_section_bytes` or explicit semantic-native cap |
| query reconstruction | `max_decoded_section_bytes` |
| metadata | `max_metadata_bytes` |
| final publication | whole-load remaining; fixed-small stage |

Section caps do not replace the root total.

The root total never resets.

---

# T10 — Independent counting allocator proof harness

The counting allocator is test-only verification, never production accounting.

---

## T10.1 Isolation

Use:

```text
dedicated test binary
single active measured thread
fixture built before measurement
tracking enabled immediately before target
tracking disabled immediately after return
no tracing/log formatting in measured region
```

Global allocator tests must not share a parallel integration binary with unrelated tests.

---

## T10.2 Requested-byte measurement

Count:

```rust
alloc(layout)        -> layout.size()
alloc_zeroed(layout) -> layout.size()
realloc(..., n)      -> n
dealloc(...)         -> 0
```

This is the contract.

---

## T10.3 Tripwire mode

Add a test-only request threshold.

If next allocator request would exceed threshold:

```text
record unexpected allocator request
fail/refuse request
```

For logical `N - 1` test:

```text
threshold = N - 1
```

Expected:

```text
AllocationError::Exhausted
tripwire not triggered
```

This proves the ledger rejected before the unmatched native request.

---

## T10.4 Exact stage algorithm

For each stage:

```text
1. run with very large logical budget
2. observe native requested bytes N
3. assert stage ledger requested == N
4. run with budget N -> success
5. run with budget N - 1 + allocator tripwire -> Exhausted
6. assert tripwire not triggered
7. assert failed child did not commit parent
8. retry with fresh budget N -> success
```

Zero-allocation stages must prove `N == 0`.

No fake minimum debit.

---

# T11 — Required certification matrix

| Gate | Stage | Proof |
|---|---|---|
| A3.01 | allocation helpers | exact request equality + failure restore |
| A3.02 | disk file acquisition | owned file buffer charged |
| A3.03 | container | directory/string/scratch equality |
| A3.04 | module DTO | DTO equality |
| A3.05 | module materialization | complete nested native authority equality |
| A3.06 | semantic DTO | decode equality |
| A3.07 | semantic validator | guard-removal proof |
| A3.08 | TypeStore | relocation/store equality |
| A3.09 | callable owner | frozen owner equality |
| A3.10 | other semantic owners | family-level equality |
| A3.11 | aggregate semantic import | snapshot materialization equality |
| A3.12 | query DTO | DTO equality |
| A3.13 | query reconstruction | owner/product/edge/table equality |
| A3.14 | metadata | native ownership equality |
| A3.15 | formal image | validation + Arc equality |
| A3.16 | LoadedUniverseArtifact | final Arc equality |
| A3.17 | full in-memory load | all owned requests except borrowed bytes |
| A3.18 | full disk load | equality including file buffer |
| A3.19 | aggregate total N/N-1 | each stage fits; total rejects/succeeds correctly |
| A3.20 | local stage N/N-1 | root has headroom; stage cap rejects |
| A3.21 | malformed rollback | no parent commit |
| A3.22 | corrected retry | success after prior failed load |

---

# 8. Guard-removal ledger

## Guard A — semantic validation

```text
require_semantic_validation_allocation_accounting
```

Remove only after:

```text
semantic DTO decode exact
all validator delegates classified
scratch exact/allocation-free
error construction safe
valid nonempty semantic world N/N-1 passes
malformed rollback passes
```

It may be removed before semantic import guard.

---

## Guard B — semantic import

```text
require_semantic_import_allocation_accounting
```

Remove only after:

```text
TypeStore exact
static import exact
PATCH1 owner import exact
PATCH2 owner import exact
snapshot/index exact
semantic import errors safe
aggregate semantic import N/N-1 passes
```

No shortcut.

---

# 9. Whole-load proof

Final PATCH3 proof:

```text
independent allocator observed requested bytes
==
root AllocationBudget.requested()
```

for both:

```text
fresh successful in-memory load
fresh successful disk load
```

Disk total includes owned file-buffer allocation.

In-memory total excludes caller-owned input bytes.

Record both explicitly.

---

# 10. Aggregate exhaustion discriminator

Construct a valid artifact with measured stage requests:

```text
container = A
modules = B
semantic = C
query = D
metadata = E
publication = F
```

Every local cap is greater than its stage request.

Set root total:

```text
A + B + C + D + E + F - 1
```

Require:

```text
typed Exhausted
first crossing request is not issued
no stage-limit misclassification
no publication
```

Set exact total:

```text
A + B + C + D + E + F
```

Require success.

---

# 11. Section-local exhaustion discriminator

Root total has generous headroom.

Set one local cap:

```text
N - 1
```

Require:

```text
stage child Exhausted
parent does not commit failed child
other-stage headroom irrelevant
```

This proves independent local limits.

---

# 12. Disk versus borrowed-buffer accounting

## Disk path

`load_universe_telosc(path, ...)` allocates an owned byte buffer.

Charge it.

Measure it.

---

## In-memory path

`load_universe_telosc_from_bytes(&[u8], ...)` borrows caller storage.

Do not charge the input buffer.

Only parse/materialization allocations count.

The two totals should differ by owned acquisition behavior.

Do not call this a mismatch.

---

# 13. Typed error contract

Required causes remain distinct:

```text
AllocationError::Exhausted
AllocationError::Overflow
AllocationError::Failed
AllocationError::UnaccountedBoundary
```

`UnaccountedBoundary` is an intentional temporary development barrier.

At PATCH3 completion no supported P7 load path should return it.

Do not collapse allocation causes into prose.

---

# 14. Performance constraints

Allocation closure must not destroy compiler/load complexity.

Preserve:

```text
module build          O(N log N) or better
query closure         O(V + E)
frozen map lookup     O(log N)
TypeStore ID lookup   O(1)
module-local iteration bounded to module
canonical parent attach without full copy
```

Avoid:

```text
linear scan for every hot lookup
quadratic ordered insertion into final vectors
deep canonical product clones
copying frozen canonical tables into local mutable tables
```

Recommended builder:

```text
reserve contiguous rows
append
sort_unstable
adjacent duplicate validation
seal
```

---

# 15. Metrics terminology

Keep distinct:

## Requested bytes

```text
sum of allocator request sizes
```

This is the enforced budget.

## Retained bytes

Memory still owned by final canonical image.

Performance metric only.

## Peak RSS

OS/process resident memory.

Performance metric only.

Do not state that the requested-byte budget is an RSS cap.

---

# 16. Semantic parity requirements

For every source/frozen physical backing pair compare:

```text
get
contains
iter
keys
values
len
module-bounded readers
ordering where contractual
fingerprints
dependency identity
visibility/exposure
runtime DAG meaning
compiler consumer reads
```

PATCH3 must reuse PATCH1/PATCH2 semantic fixtures.

Do not create alternate semantic truth.

---

# 17. Coordination with current PATCH1 work

At `d26de3a2`, PATCH1 already provides frozen callable infrastructure.

PATCH3 must reuse:

```text
AnalysisMap
CallableAnalysisTable frozen backing
FlowGraph frozen backing
callable ordinal ownership work
```

If PATCH1 changes concurrently, re-read/rebase before finalizing nested allocation proof.

No duplicate representation.

---

# 18. Coordination with PATCH2

PATCH2 owns final semantic shapes for:

```text
accepted definitions
traits
witnesses
derived conformance
trait dispatch
static owners
module static bindings
formal projection
```

PATCH3 may prepare physical wrappers.

It must not invent semantic defaults.

If rich owner allocation is difficult, change the physical backing—not the semantics.

---

# 19. Coordination with PATCH4

PATCH4 decides:

```text
persisted query owner kinds
dependency_evidence behavior
stable query owner codec
fingerprint authority
```

PATCH3 decides:

```text
how decoded owners/products/edges/tables allocate
```

If PATCH4 replaces JSON identity, use that structured codec.

If JSON remains temporarily, account it.

---

# 20. Coordination with PATCH5

PATCH5 owns:

```text
contiguous canonical file layout
decoded_len correctness
capability bits
section ordering
section buffer copies
publication byte semantics
```

PATCH3 only owns the allocation consequences of parsing/materialization.

Keep reviews separable.

---

# 21. Rejected shortcuts

Forbidden:

```text
size_of::<(K,V)> * entries as BTreeMap charge
private std B-tree node mirroring
guessed overhead multipliers
encoded bytes as native allocation estimate
per-section reset of whole-load budget
successful scratch refunds
ignoring temporary allocations
ignoring error allocation
production global allocator accounting
nightly allocator hook as production policy
custom map without fallible allocation semantics
build BTreeMap then copy into frozen storage
deep clone to satisfy old APIs
e.to_string() inside constrained load
remove UnaccountedBoundary to find the next leak
cargo check as allocation proof
helper-only tests as whole-load proof
```

---

# 22. STOP / CONSULT conditions

Stop if:

1. a compiler consumer genuinely requires mutable access to imported canonical authority;
2. a required final representation cannot be constructed fallibly with visible requests;
3. TypeStore requires full mutable canonical hash reconstruction and no overlay seam is viable;
4. freezing changes observable semantic ordering;
5. a helper depends on private standard-library allocation layout;
6. the counting allocator observes a different layout from the planned debit;
7. allocation-free error transport requires public ABI redesign outside PATCH3 authority;
8. PATCH1/PATCH2 owner contracts change during implementation;
9. query semantics must be decided by PATCH4;
10. the issue is actually PATCH5 binary canonicality.

Escalation packet:

```text
revision
stage
native type
requested operation
allocator request trace
ledger trace
semantic read requirements
candidate physical representations
performance tradeoff
decision required
```

---

# 23. Test harness placement

Recommended:

```text
phalcom-type-meta/tests/allocation_budgeted_map.rs
    primitive/helper exactness

phalcom-compiler-artifact/tests/container_allocation_test.rs
    container

phalcom-compiler-artifact/tests/query_allocation_test.rs
    query DTO

phalcom-modules/tests/artifact_materialization_allocation.rs
    full module native materialization

phalcom-semantic/tests/semantic/artifact_allocation.rs
    validator/frozen helper proof

phalcom-semantic/tests/artifact_type_import_allocation.rs
    isolated TypeStore import if necessary

phalcom-semantic/tests/artifact_query_reconstruction_allocation.rs
    complete query native reconstruction

phalcom-core/tests/core/modules/canonical_artifact_allocation.rs
    whole load
```

Exact filenames may follow current test registration conventions.

Keep global counting allocators isolated.

---

# 24. Verification ladder

## During build

```bash
cargo check -p <owning crate>
```

plus the smallest exact test.

## Allocation verification

Use isolated test binaries with:

```bash
--test-threads=1
```

## Semantic verification

Every backing change also runs source/import parity and fingerprint/consumer tests.

## Final PATCH3 verification

At minimum rerun the existing four allocation suites plus all newly added module/type/query/core allocation binaries.

Broad workspace/release certification remains PATCH7 unless API migration requires it sooner.

---

# 25. Failure atomicity

A failed load must leave:

```text
no CanonicalUniverseFormalImage publication
no LoadedUniverseArtifact publication
no poisoned singleton/global image
no committed failed-stage child debit
no retained partial owner affecting retry
```

A bad artifact followed by a corrected artifact must succeed on retry.

---

# 26. PATCH3 completion checklist

## Contract

```text
[ ] one root cumulative ledger
[ ] independent local limits
[ ] precharge before every request
[ ] no successful-load scratch refund
[ ] failed child rollback
```

## Errors

```text
[ ] lower allocation errors remain typed
[ ] core does not stringify allocation failures
[ ] exhausted error propagation causes zero extra requests
```

## Modules

```text
[ ] outer module authority frozen/charged
[ ] nested interfaces/link/topology/graphs charged
[ ] owned module strings charged
[ ] exact module N/N-1 test
```

## Validator

```text
[ ] all delegates classified
[ ] scratch charged/allocation-free
[ ] exact validator N/N-1
[ ] semantic validation guard removed only after proof
```

## TypeStore

```text
[ ] shared budget passed into import
[ ] all relocation arrays charged
[ ] canonical indexes frozen/charged
[ ] scoped validation uses global ledger
[ ] no estimated local bytes substitute
[ ] exact TypeStore N/N-1
```

## Semantic import

```text
[ ] PATCH1 owners allocation-closed
[ ] PATCH2 owners allocation-closed
[ ] snapshot/index ownership charged
[ ] aggregate semantic import exact
[ ] semantic import guard removed only after proof
```

## Query

```text
[ ] owner decode charged
[ ] products charged
[ ] edge storage charged
[ ] root table frozen/charged
[ ] fingerprint table frozen/charged
[ ] query image Arc charged
[ ] exact query N/N-1
```

## Metadata/publication

```text
[ ] metadata exact proof
[ ] formal-image validation allocation-closed
[ ] formal-image Arc charged
[ ] outer artifact Arc charged
```

## Whole load

```text
[ ] in-memory observed == ledger
[ ] disk observed == ledger
[ ] aggregate total N/N-1
[ ] local stage N/N-1
[ ] rollback
[ ] corrected retry
[ ] no supported load returns UnaccountedBoundary
```

---

# 27. Suggested implementation commits

Recommended reviewable sequence:

```text
fix(perf): preserve typed artifact allocation failures
feat(perf): extend exact artifact allocation helpers
feat(perf): freeze module artifact materialization
test(perf): certify module materialization allocation
feat(perf): budget type graph import
test(perf): certify type relocation allocation
feat(perf): close semantic validator allocation
test(perf): certify semantic validation budget
feat(perf): close imported semantic owner allocation
feat(perf): freeze loaded query authority
test(perf): certify query reconstruction allocation
feat(perf): budget canonical image publication
test(perf): certify whole artifact load allocation
docs(perf): close P7 PATCH3 allocation ledger
```

Adjust around concurrent PATCH1/PATCH2 ownership work.

Do not hide all representation changes in one commit.

---

# 28. Walkthrough requirement

Create:

```text
PERF003.C2.P7.PATCH3-walkthrough.md
```

Required sections:

1. start/final revision;
2. exact allocation contract;
3. before/after stage census;
4. helper changes;
5. typed error changes;
6. module frozen authority;
7. TypeStore changes;
8. validator changes;
9. semantic owner physical storage;
10. query physical storage;
11. metadata/publication;
12. counting allocator methodology;
13. stage N/N-1 matrix;
14. in-memory whole-load equality;
15. disk whole-load equality;
16. guard removals and proof commands;
17. retained/requested/RSS metrics;
18. residual PATCH4/PATCH5/PATCH7 work.

---

# 29. Handoff requirement

Create:

```text
PERF003.C2.P7.PATCH3-handoff.md
```

Must report:

```text
branch
HEAD
commits
clean/dirty state

semantic validation guard:
    PRESENT / REMOVED
    exact proof

semantic import guard:
    PRESENT / REMOVED
    exact proof

allocation census:
    ACCOUNTED rows
    OPEN rows

whole in-memory observed:
whole in-memory ledger:

whole disk observed:
whole disk ledger:

typed allocation error test:
aggregate total N/N-1:
stage-local N/N-1:
retry:

PATCH1/PATCH2 dependencies:
PATCH4 physical-allocation handoff:
PATCH7 final acceptance dependency:
```

No COMPLETE claim while any supported row remains unaccounted.

---

# 30. Reviewer questions

A reviewer must answer **yes** to all:

1. Does every owned allocator request have a preceding exact debit?
2. Can a budget failure occur before the allocator request?
3. Do successful temporary allocations remain charged?
4. Does failed child work leave parent unchanged?
5. Are native B-tree estimates absent?
6. Are imported persistent ordered owners allocator-visible?
7. Are nested strings/boxes/arcs accounted separately?
8. Does TypeStore import receive the shared transaction?
9. Is scoped validation exact rather than estimated?
10. Is validator accounting independently certified before guard removal?
11. Is semantic import independently certified before guard removal?
12. Is query owner decoding accounted?
13. Are shared dependency buffers accounted?
14. Is loaded query storage frozen rather than copied into a B-tree?
15. Is the formal image allocated under the ledger?
16. Are allocation errors typed through core?
17. Does exhausted error propagation allocate zero additional bytes?
18. Does full-load allocator observation equal the ledger?
19. Does `N - 1` reject before an unmatched request?
20. Can a failed load be retried?
21. Is semantic parity unchanged by physical backing?
22. Are requested bytes, retained bytes and RSS reported separately?

Any “no” means PATCH3 is not closed.

---

# 31. Final expected state

At PATCH3 completion:

```text
.telosc
  │
  ▼
root exact allocation transaction
  │
  ├─ file acquisition
  ├─ container
  ├─ module DTO
  ├─ frozen module authority
  ├─ semantic DTO
  ├─ allocation-closed semantic validation
  ├─ frozen TypeStore / relocation
  ├─ frozen semantic owners
  ├─ query DTO
  ├─ frozen query authority
  ├─ metadata
  ├─ canonical formal image
  └─ loaded artifact
         │
         ▼
   publish only on success
```

Core proof:

```text
successful load:

independent allocator requested
==
AllocationBudget.requested()
```

and:

```text
budget N - 1:

typed Exhausted
AND
no unmatched allocator request
AND
no partial publication
```

Only then are the two `UnaccountedBoundary` barriers obsolete because the path they protect is actually allocation-closed.

P7 still remains `IN_PROGRESS` until the remaining semantic, query, container, source-free package, determinism and performance gates close.
