# PERF003.C2.P5 — Canonical Publication and Persistent Image Boundary
## Patch-Grade Detailed Implementation Plan

**Program:** `PERF003`  
**Checkpoint:** `PERF003.C2`  
**Normalized plan:** `PERF003.C2.P5`  
**Repository:** `aureat/phalcom-lang` (`phalcom`)  
**Branch:** `perf/universe-characterization`  
**Prepared against committed branch head:** `e74a92132bab6a7b7682a047c05dc01ab4db8b28`  
**Prepared:** 2026-10-13  
**Plan status:** `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`
**Latest implemented source revision:** `eeca76f3` (see CHECKPOINT.md for task census and current acceptance evidence).
**Primary successor:** `PERF003.C2.P6` — user-only backend/local complexity closure  
**Artifact successor:** `PERF003.C2.P7` — `.telosc` container + persisted semantic compiler image  

---

# 1. Purpose

`PERF003.C2.P5` establishes the **explicit canonical compiler-image publication boundary** that must exist before the Universe can be safely persisted as `universe.telosc`.

P1–P3 already changed the semantic architecture from repeated whole-Universe reconstruction toward:

```text
immutable canonical Universe authority
    +
local workspace overlay
```

P4 is establishing:

```text
root-driven local metadata
    +
immutable canonical metadata authority
    +
stable cross-pool semantic identity
```

P5 must now make the **canonical in-memory compiler world itself explicit, consumer-aware, immutable, compositional, and serialization-ready**.

The target is not an on-disk artifact yet. P5 must instead make the in-memory authority clean enough that P7 can encode it without:

- re-running semantic analysis;
- reconstructing omitted products;
- serializing mutable database internals;
- depending on process-local IDs;
- copying the complete canonical semantic world into every local session;
- implicitly defining artifact contents by `Default == Full`;
- rebuilding editor/advisory products merely because the compiler needs executable semantics.

The P5 end state is:

```text
canonical module authority
    +
canonical semantic authority
    +
explicit canonical product contract
    +
eligible immutable query roots
    +
canonical RuntimePublic metadata authority
    +
stable compatibility/provenance contract
        ↓
serialization-ready canonical compiler image boundary
        ↓
P7 portable .telosc projection
```

P5 does **not** write `.telosc`.

---

# 2. Current repository state and plan-entry ruling

## 2.1 Verified branch state

The current committed branch head inspected for this plan is:

```text
e74a92132bab6a7b7682a047c05dc01ab4db8b28
```

This revision advances P4 beyond the earlier `a82dfe32` snapshot.

The current committed P4 state includes:

```text
P4.T1  ProgramCompilationOptions metadata-profile plumbing
        IMPLEMENTED / focused seam tested

P4.T2  MetadataRootSet / MetadataRetentionRequest
        IMPLEMENTED

P4.T3  MetadataReachabilityClosure
        IMPLEMENTED

P4.T4  callable/field owner indexes
        IMPLEMENTED

P4.T10 metadata observability counter definitions
        IMPLEMENTED
```

The production cutover remains incomplete:

```text
P4.T5  reachability-driven MetadataExporter cutover
        NOT IMPLEMENTED

P4.T6  canonical metadata external authority
        BLOCKED

P4.T7  runtime stable-identity indexes
        NOT IMPLEMENTED

P4.T8  cross-pool reflection cutover
        NOT IMPLEMENTED

P4.T9  canonical metadata dependency compatibility
        NOT IMPLEMENTED

P4.T11 focused compiler/reflection/runtime verification
        NOT COMPLETE
```

P4 is currently recorded as:

```text
IN_PROGRESS / PARTIAL / BASELINE_BLOCKED
```

## 2.2 Active P4 incident

P4 has a mandatory architecture incident:

```text
PERF003.C2.P4-canonical-authority-incident.md
```

The required canonical RuntimePublic authority:

```text
canonical_universe_program()
    → CompiledProgram::semantic_metadata
```

cannot currently be built because canonical bootstrap fails before compiler projection:

```text
NativeSourceIndex::build
    ↓
ProductKind source-index rejection
```

Observed error:

```text
intrinsic runtime data ProductKind is not a canonical Tuple/Record/Pack source declaration
```

Separately, retained-session aggregate metadata export still fails on:

```text
legacy metadata cannot represent a Record remainder outside the Symbol key domain
```

These are distinct failures.

## 2.3 P5 entry decision

P5 may be **planned now** and its source-independent preparation tasks may begin after explicit authorization, but **P5 must not be accepted as implemented or complete while P4's canonical metadata authority is unavailable**.

The following are hard prerequisites for P5 integration closure:

```text
1. P4 T5 production exporter cutover is complete.
2. RuntimePublic field metadata is wired.
3. Canonical/local metadata partitioning is real in production.
4. Runtime stable-identity lookup and cross-pool reflection are functional.
5. Canonical metadata compatibility/dependency identity exists.
6. A canonical RuntimePublic metadata authority can actually be constructed.
7. P4 focused compiler/reflection gates pass or are explicitly classified.
```

The `ProductKind` bootstrap repair is **not automatically P5 work**.

If the P4 ruling assigns it as a separately bounded prerequisite, that prerequisite must close before P5's canonical metadata integration gate.

---

# 3. Numbering authority

The repository checkpoint still contains historical plan numbering in which:

```text
P5 = backend scans
P7 = consumer-specific publication
```

The user-approved normalized forward program is:

```text
P1  Canonical module / one-shot overlay boundary
P2  Complete canonical formal image + immutable queries
P3  Conformance/callable incrementalization
P4  Root-driven metadata + cross-pool authority
P5  Canonical publication + persistent image boundary
P6  User-only backend/local complexity closure
P7  .telosc schema + persisted semantic compiler image
P8  Portable executable Universe image
P9  Artifact-first loader/cache/toolchain certification
```

This plan uses the **normalized P5**.

Historical lifecycle evidence must not be deleted or rewritten as if the older numbering never existed. Before P5 execution is formally activated, `CHECKPOINT.md` should gain an explicit numbering amendment or normalized-plan section.

---

# 4. Normative P5 outcome

At P5 closure, the canonical Universe compiler world must satisfy:

```text
Canonical Universe source
    ↓
source-backed artifact-build path
    ↓
explicit canonical compiler product request
    ↓
complete immutable canonical semantic authority
    ├── canonical module products
    ├── canonical type/static arenas
    ├── canonical declarations/hierarchy/dispatch
    ├── canonical callable/signature/analysis products required downstream
    ├── canonical traits/conformance/evidence products required downstream
    ├── canonical data/enum/static semantics
    ├── explicitly eligible immutable query roots
    └── canonical RuntimePublic metadata authority
            ↓
ordinary workspace
    ├── readonly parent references
    └── user-owned local deltas only
```

A warm ordinary workspace must not require:

```text
canonical semantic update
canonical source presentation rebuild
canonical advisory fixed point
canonical aggregate deep copy
canonical metadata export
canonical query reconstruction
```

P5 must leave one remaining fundamental cold problem intentionally visible:

```text
canonical image is still source-built on first process access
```

That problem belongs to P7/P9 persistence and loader cutover.

---

# 5. Architectural invariants

## INV-P5-01 — Canonical image contents are explicit

`SemanticProductRequest::default()` must not be the durable definition of the canonical compiler image.

The canonical build must use an explicit, named contract.

Conceptually:

```rust
SemanticProductRequest::canonical_compiler_image()
```

or an equivalent dedicated request type.

The exact API may differ, but canonical-image contents must be reviewable without knowing the implementation of `Default`.

---

## INV-P5-02 — Canonical semantic completeness is compiler-driven, not tooling-driven

The canonical compiler image must contain every product needed to:

```text
type-check novel user source
resolve canonical imports
resolve generic constraints
perform conformance/evidence queries
lower user code
construct runtime metadata references
compile calls against canonical APIs
```

It does **not** follow that it must contain:

```text
all editor presentation products
all advisory products
all navigation/source-map products
all debug products
```

Those families require explicit classification.

---

## INV-P5-03 — Full tooling mode remains supported

P5 must not optimize runtime/compiler publication by breaking `SemanticProductRequest::full()` semantics.

The system must continue to support:

```text
Full tooling publication
Runtime compiler publication
Canonical compiler-image publication
```

as distinct contracts.

---

## INV-P5-04 — Canonical image is not local publication history

Preserve the P2 rule:

```text
canonical image != last_snapshot
canonical image != last_known_good
```

`SemanticWorkspaceSession::from_image(...)` must continue to begin with:

```text
last_snapshot: None
last_known_good: None
```

Canonical authority is a parent lifetime domain, not revision zero of the mutable workspace.

---

## INV-P5-05 — Ordinary sessions must not own copied canonical aggregates

P5 must move session ownership toward:

```text
Arc<CanonicalUniverseFormalImage>
    +
local mutable semantic deltas
```

rather than:

```text
clone canonical table A
clone canonical table B
clone canonical table C
...
```

A clone that duplicates O(Universe) map entries is not an acceptable parent model merely because the value type contains `Arc`s.

---

## INV-P5-06 — Canonical parents are immutable

Local mutation, invalidation, rollback, removal, cancellation, and snapshot replacement must never mutate canonical aggregate state.

Any parented table abstraction must distinguish:

```text
canonical parent entries
local overlay entries
```

and must not silently copy-on-write the canonical world during the first local mutation.

---

## INV-P5-07 — Stable semantic identity and process-local identity remain separate

P5 may preserve in-process canonical arena prefixes for efficiency.

P5 must **not** declare raw runtime/process IDs portable.

Not durable identity:

```text
WorkspaceId
TypeStoreId
StaticValueStoreId
MetadataPoolId
ClassId
ObjRef
raw arena addresses
```

P7 will introduce relocatable artifact IDs and validation.

---

## INV-P5-08 — Query persistence eligibility remains explicit and fail-closed

P2 already rejects query products that are not proven immutable, closed, current, and relocatable.

P5 must preserve that model.

Do not replace it with:

```text
serialize SemanticDb
serialize all Ready rows
serialize mutable reverse dependency state
```

---

## INV-P5-09 — P4 owns metadata authority semantics

P5 consumes the canonical metadata authority produced by P4.

P5 must not duplicate or redefine:

```text
stable declaration identity
stable callable identity
stable field identity
cross-pool lookup policy
metadata conflict policy
canonical metadata dependency compatibility
```

---

## INV-P5-10 — P6 owns residual backend whole-world complexity

P5 may add indexes/views required to make canonical/local ownership possible.

P5 must not expand into an unbounded rewrite of:

```text
phalcom-core/src/modules/semantic_lowering.rs
```

or every backend aggregate scan.

P6 owns the measured user-only backend complexity closure.

---

# 6. Primary source ownership

| Responsibility | Primary source owner |
|---|---|
| Semantic product request | `phalcom-semantic/src/request.rs` |
| Canonical semantic image | `phalcom-semantic/src/canonical_image.rs` |
| Ordinary semantic overlay session | `phalcom-semantic/src/session.rs` |
| Immutable published semantic snapshot | `phalcom-semantic/src/snapshot.rs` |
| Canonical query read-through | `phalcom-semantic/src/db/mod.rs` |
| Query-key/product definitions | `phalcom-semantic/src/db/*` |
| Declaration aggregate | `phalcom-semantic/src/declarations.rs` |
| Callable/field aggregate | `phalcom-semantic/src/signature.rs` |
| Hierarchy aggregate | `phalcom-semantic/src/types/relation.rs` (`MapTypeHierarchy`) |
| Dispatch aggregate | semantic dispatch/surface resolver owner used by `SemanticSnapshot::dispatch` |
| Data / enum / trait / associated aggregates | corresponding `phalcom-semantic/src/*` owners |
| Canonical module products | `phalcom-modules/src/canonical_products.rs` |
| External canonical module session | `phalcom-modules/src/session.rs` / linker boundary |
| Runtime compiler entry/options | `phalcom-core/src/modules/compile.rs` |
| Retained compiler session | `phalcom-core/src/modules/compile_session.rs` |
| Canonical VM-independent compiler product | `phalcom-core/src/modules/canonical_universe.rs` |
| Canonical RuntimePublic metadata authority | P4-owned core metadata path |
| Performance counters/phases | `telos-observability` |
| Lifecycle | `docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/*` |

---

# 7. Current source facts P5 must address

## 7.1 Canonical image uses implicit full publication

Current canonical construction in:

```text
phalcom-semantic/src/canonical_image.rs
build(...)
```

creates:

```rust
SemanticWorkspaceInput {
    ...
    product_request: SemanticProductRequest::default(),
}
```

Current `Default` is:

```text
SemanticProductRequest::full()
```

which means:

```text
PresentationLevel::Full
AdvisoryLevel::Full
```

Therefore canonical bootstrap is still implicitly coupled to full tooling-oriented publication.

---

## 7.2 Runtime compiler sessions set their request too late to affect cold canonical construction

`ProgramCompilationSession::standalone()` does:

```rust
SemanticWorkspaceSession::new()
    .with_product_request(SemanticProductRequest::runtime_minimal())
```

but `SemanticWorkspaceSession::new()` first obtains:

```text
canonical_universe_formal_image()
```

Therefore the canonical source build has already run before the local runtime-minimal request is attached.

P5 must remove this ambiguity.

---

## 7.3 `from_image()` still clones canonical aggregate tables

Current `SemanticWorkspaceSession::from_image()` copies canonical snapshot aggregates into session fields including:

```text
base_declarations
base_hierarchy
base_dispatch
base_callable_signatures
base_data_semantics
base_enum_semantics
base_associated_surfaces
base_type_member_declarations
base_enum_requirements
```

and derives:

```text
alias_forms
```

from the canonical alias table.

These copies are not the 45-second cold bottleneck; warm session creation has previously measured roughly 9–12 ms.

They are nevertheless the wrong ownership boundary for a persisted dependency world.

---

## 7.4 Update code repeatedly seeds complete mutable aggregate state from base tables

Current update paths include patterns such as:

```rust
let mut declarations = self.base_declarations.clone();
let mut hierarchy = ... self.base_hierarchy.clone();
let mut dispatch = ... self.base_dispatch.clone();
let mut callable_signatures = ... self.base_callable_signatures.clone();
let mut data_semantics = ... self.base_data_semantics.clone();
let mut enum_semantics = ... self.base_enum_semantics.clone();
```

P5 must ensure these become **cheap overlay construction**, not repeated O(Universe) map duplication.

---

## 7.5 Canonical query freezing is already deliberately selective

`CanonicalSemanticQueryImage::freeze(...)` explicitly rejects:

```text
open-world proof/conformance queries
callable-body queries
checked static bodies
unsupported query families
mutable dependency closures
cycles without a proven contract
workspace-origin rows
stale rows
failed/blocked rows
```

and retains only independently approved structural products.

This is the correct architectural starting point.

P5 should make the portability classification explicit and reusable by P7 rather than replacing it.

---

# 8. Package task graph

```text
P5.T0  Entry gate, lifecycle normalization, and measurement census
   ↓
P5.T1  Explicit canonical compiler-image product contract
   ↓
P5.T2  Product-family classification and request-model cleanup
   ↓
P5.T3  Canonical bootstrap cutover to explicit request
   ↓
P5.T4  Complete consumer-specific publication semantics
   ↓
P5.T5  Introduce immutable canonical aggregate-parent architecture
   ↓
P5.T6  Convert workspace/session aggregates to parent + local delta
   ↓
P5.T7  Snapshot/read API hardening for parented canonical aggregates
   ↓
P5.T8  Formal canonical persistence inventory and capability descriptor
   ↓
P5.T9  Query persistence classification extraction
   ↓
P5.T10 Bind P4 canonical metadata authority into the compiler-image boundary
   ↓
P5.T11 Observability + structural cardinality gates
   ↓
P5.T12 Semantic/tooling/compiler parity and performance verification
   ↓
P5.T13 Lifecycle closure and P6/P7 handoff
```

T1–T4 may be developed before the P4 bootstrap incident is resolved if they do not depend on canonical metadata construction.

T10 and final P5 acceptance are blocked on P4.

---

# 9. P5.T0 — Entry gate, lifecycle normalization, and measurement census

## Purpose

Establish a clean execution boundary so P5 does not inherit ambiguous P4 state or stale numbering.

## Required work

### A. Reconcile lifecycle numbering

Modify:

```text
docs/implementation/PERF003-runtime-performance/
  C2-compilation-performance-and-efficiency/CHECKPOINT.md
```

Record explicitly:

```text
normalized P5 = canonical publication + persistent image boundary
normalized P6 = backend/local complexity
historical P5/P7 labels retained as legacy execution history
```

Do not delete historical evidence.

### B. Record P4 prerequisite state

P5's activation record must link:

```text
PERF003.C2.P4-canonical-authority-incident.md
```

and state whether the incident was resolved by:

```text
bounded prerequisite repair
or
separate plan
or
approved detached continuation
```

Do not infer a ruling.

### C. Capture current P5 baseline counters

Before changing canonical publication, measure in one process:

```text
A. first canonical_universe_formal_image()
B. second canonical_universe_formal_image()
C. first SemanticWorkspaceSession::new() after image warm
D. second warm SemanticWorkspaceSession::new()
```

Break canonical build into at least:

```text
canonical baseline acquisition
CanonicalModuleProducts::capture
topology construction
semantic update
source-index/presentation publication
advisory publication
snapshot publication
validate_publication
query freeze
from_image attachment
```

### D. Add aggregate-copy census

Before T5/T6, record:

```text
entries cloned into base_declarations
entries cloned into base_hierarchy
entries cloned into base_dispatch
entries cloned into base_callable_signatures
entries cloned into base_data_semantics
entries cloned into base_enum_semantics
entries cloned into base_associated_surfaces
entries cloned into base_type_member_declarations
entries cloned into base_enum_requirements
alias forms copied
```

The goal is structural comparison after the parent conversion.

## Exit criteria

```text
P5 lifecycle entry is unambiguous
P4 blockers are explicitly recorded
cold/warm phase measurements exist
canonical aggregate-copy cardinalities exist
```

---

# 10. P5.T1 — Define an explicit canonical compiler-image product contract

## Purpose

Remove `Default == Full` as the canonical image specification.

## Primary file

```text
phalcom-semantic/src/request.rs
```

## Required API

Introduce an explicit constructor or dedicated contract.

Minimum acceptable form:

```rust
impl SemanticProductRequest {
    pub const fn canonical_compiler_image() -> Self {
        // Explicitly reviewed product contract.
    }
}
```

A richer dedicated type is acceptable if product axes need to expand:

```rust
pub struct CanonicalCompilerProductRequest {
    pub source_semantics: ...,
    pub presentation: ...,
    pub advisory: ...,
    ...
}
```

## Initial safety rule

The first commit may deliberately make:

```text
canonical_compiler_image() == full()
```

if necessary to introduce an explicit seam before narrowing.

That commit is useful only as a migration step.

P5 cannot close until the request has been classified and unnecessary canonical products have been removed.

## Required tests

```text
default() remains full tooling-compatible
runtime_minimal() remains unchanged
canonical_compiler_image() is explicit
canonical request is not inferred from local consumer request
```

## Forbidden

Do not:

```text
change Default to runtime_minimal
make canonical bootstrap inherit whichever request a later session chooses
use cfg(test) to select a cheaper canonical request
```

---

# 11. P5.T2 — Classify every canonical product family

## Purpose

Determine the minimal **complete compiler image**, not merely the smallest snapshot that passes current tests.

## Required classification

Every major `SemanticSnapshot` and canonical query/module family must be classified as one of:

```text
A. REQUIRED_COMPILER_AUTHORITY
B. REQUIRED_RUNTIME_METADATA_AUTHORITY
C. REQUIRED_PORTABLE_QUERY_ROOT
D. OPTIONAL_TOOLING_PRESENTATION
E. OPTIONAL_DEBUG_SOURCE_MAP
F. REBUILDABLE_LOCAL_DERIVATION
G. PROCESS_LOCAL_ONLY
H. WORKSPACE_LOCAL_ONLY
I. NEVER_PERSIST
```

## Required inventory

At minimum classify:

```text
CanonicalModuleProducts
ProjectUniverse/module topology
unlinked interfaces
linked interfaces
import-resolution products
parsed canonical modules
TypeStore canonical prefix
StaticValueStore canonical prefix
declarations
type aliases
hierarchy
surfaces
dispatch
callable definitions
callable signatures
field signatures
callable analyses
constructor initializations
trait headers
trait surfaces
conformance index
conformance witness plans
derived conformance rules
trait dispatch index
data semantics
enum semantics
enum requirements
associated surfaces
type-member declarations
associated static domains
static case catalogs
static case signatures
static definitions
static constants
module static bindings
semantic graph
formal projection
source semantic index
presentation source text
advisory workspace
diagnostics/internal incidents
semantic structure shards
canonical frozen query products
canonical RuntimePublic metadata
```

## Consumer matrix

For every `REQUIRED_*` family, identify exact consumers:

```text
semantic update
generic checking
conformance/evidence
lowering
metadata export
canonical program projection
runtime materialization
tooling/editor
P7 relocation
P8 portable executable generation
```

## Gate

A product may be removed from canonical compiler publication only after:

```text
all current downstream consumers are identified
and
a differential test demonstrates no loss of compiler semantics
```

---

# 12. P5.T3 — Cut canonical bootstrap over to the explicit request

## Purpose

Make canonical construction use the reviewed contract.

## Modify

```text
phalcom-semantic/src/canonical_image.rs
```

Current:

```rust
product_request: SemanticProductRequest::default(),
```

Target:

```rust
product_request: SemanticProductRequest::canonical_compiler_image(),
```

or equivalent.

## Required behavior

Canonical build must remain:

```text
complete
deterministic
single-publication
retryable on failed construction
non-reentrant
immutable after publication
```

Preserve:

```text
OnceLock
INITIALIZE mutex
recursion guard
successful-publication-only behavior
```

## Completeness gate

Strengthen `validate_publication(...)` so it validates **compiler-image completeness**, not the accidental contents of Full mode.

Validation should check required product families established in T2.

Do not validate optional tooling products unless the canonical request includes them.

## Test matrix

At minimum:

```text
canonical request builds successfully
full request builds successfully
canonical request has every required compiler product
canonical request excludes each intentionally omitted optional family
canonical request and full request produce identical compiler-relevant semantic identities
```

---

# 13. P5.T4 — Complete consumer-specific publication semantics

## Purpose

Finish the historically displaced consumer-publication work without preserving stale P7 numbering.

Current request architecture has:

```text
PresentationLevel::{None, Full}
AdvisoryLevel::{None, Full}
```

but the existing `PresentationLevel::None` behavior is already more nuanced than its name: local compiler-required source semantic products are still published while canonical presentation shards are suppressed.

P5 should make those semantics explicit and maintainable.

## Required design decision

Separate, either conceptually or in the type system:

```text
compiler-required source semantic index
editor/presentation source products
advisory products
```

Do not treat them as one boolean "presentation" concept if that forces canonical compiler images to retain editor-only work.

A representative model is:

```rust
pub enum SourceSemanticLevel {
    None,
    CompilerRequired,
    Full,
}

pub enum PresentationLevel {
    None,
    Full,
}

pub enum AdvisoryLevel {
    None,
    Full,
}
```

The exact public shape may differ. The mandatory property is independent classification.

## Required publication behavior

### Runtime compiler request

```text
local compiler-required source semantic products = yes
canonical presentation source shards = no
advisory workspace = no
```

### Canonical compiler-image request

Publish only products demonstrated necessary for:

```text
novel downstream user compilation
canonical lowering/projection
artifact construction
```

### Full tooling request

Preserve:

```text
full source/navigation presentation
full advisory workspace
tooling compatibility
```

## Transition tests

Retained sessions must cover:

```text
Full → Runtime → Full
Full → Canonical-like coverage where supported
Runtime → Full
```

and prove stale omitted products do not leak across request transitions.

## Non-goal

Do not make local session request changes mutate the process-wide canonical image.

---

# 14. P5.T5 — Introduce immutable canonical aggregate-parent architecture

## Purpose

Remove deep canonical table ownership from each local semantic session.

## Current problem

`SemanticWorkspaceSession::from_image()` currently initializes fields by cloning canonical aggregates.

The session also repeatedly clones those base tables to seed update-local mutable aggregate state.

## Required structural model

Move to:

```text
CanonicalUniverseFormalImage
    owns canonical aggregate Arc<T>
        ↓ readonly parent
local table
    owns only local entries
        ↓
composite lookup/iteration
```

The preferred migration is to make aggregate containers themselves parent-aware, preserving most downstream APIs.

Representative pattern:

```rust
pub struct DeclarationTypeTable {
    parent: Option<Arc<DeclarationTypeTable>>,
    local_entries: HashMap<DeclarationId, DeclarationTypeInfo>,
    ...
}
```

with:

```rust
DeclarationTypeTable::with_parent(parent: Arc<Self>)
get(id)               // local, then parent
contains(id)
iter()                 // deterministic parent + local view
iter_local()
remove_local(...)
remove_module_local(...)
```

The exact implementation may use a shared generic overlay helper where appropriate.

## Why parent-aware containers are preferred

They avoid changing every consumer from:

```rust
snapshot.declarations.get(...)
```

to a special external composite object.

They also allow existing immutable `SemanticSnapshot` fields to remain:

```text
Arc<DeclarationTypeTable>
Arc<CallableSignatureTable>
...
```

while changing ownership semantics underneath.

## Mandatory rules

```text
parent is immutable
local mutations never copy parent entries
local remove cannot delete parent entries
duplicate stable IDs across parent/local are rejected or asserted
iteration ordering is deterministic
Clone of an empty/small local overlay is O(local), not O(canonical)
```

---

# 15. P5.T6 — Convert canonical-seeded aggregates to parent + local delta

## Purpose

Apply the parent model to the specific tables currently copied by `SemanticWorkspaceSession`.

## Required migration order

Do not convert all tables in one patch.

### Slice 1 — declaration/signature/hierarchy core

Convert:

```text
DeclarationTypeTable
CallableSignatureTable
FieldSignatureTable if applicable to base ownership
MapTypeHierarchy
```

Primary files:

```text
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
```

### Slice 2 — dispatch/surface families

Convert the aggregate behind:

```text
SurfaceDispatchResolver
declaration surfaces
trait headers/surfaces where canonical parent copies exist
```

### Slice 3 — data/enum/associated families

Convert:

```text
DataSemanticTable
EnumSemanticTable
EnumRequirementTable
AssociatedFamilyTable
type-member declaration aggregate
```

### Slice 4 — residual canonical lookup products

Audit:

```text
alias_forms
type aliases
static-domain products
static case products
module static bindings
other copied baseline maps
```

Use canonical read-through rather than reproducing canonical entries locally.

## Session field target

`SemanticWorkspaceSession` should gain an explicit parent:

```rust
canonical_image: Arc<CanonicalUniverseFormalImage>
```

or an equivalent readonly authority handle.

The collection of `base_*` deep-copy fields should be removed or reduced to cheap overlay factories.

Conceptually:

```rust
fn new_declaration_overlay(&self) -> DeclarationTypeTable {
    DeclarationTypeTable::with_parent(self.canonical_image.snapshot().declarations.clone())
}
```

## Update-loop target

Replace:

```rust
let mut declarations = self.base_declarations.clone();
```

with:

```rust
let mut declarations = self.new_declaration_overlay();
```

and similarly for the other canonical aggregates.

On later local revisions, cloning the previous local overlay is acceptable if it copies only local state and shared parent handles.

---

# 16. P5.T7 — Harden snapshot/read APIs for parented aggregates

## Purpose

Ensure canonical-parent composition is semantically transparent to consumers while remaining observable for performance.

## Required APIs

Each parented table should expose clear semantics for:

```text
exact lookup
merged iteration
local-only iteration
module-scoped local iteration
canonical-only iteration where diagnostics/testing need it
entry count:
    local
    canonical
    merged
```

Avoid ambiguous `len()` if callers need to distinguish local work from merged semantic visibility.

Recommended:

```rust
len()            // merged semantic size, if efficient
local_len()
parent_len()
iter()
iter_local()
```

## Duplicate/coherence policy

Stable IDs should normally be disjoint because canonical Universe and local projects occupy distinct project identities.

If the same semantic ID appears in both parent and local state:

```text
STOP
```

unless an explicitly reviewed shadowing model exists.

Do not silently let local entries override canonical entries.

## Snapshot invariants

A local `SemanticSnapshot` may expose a merged semantic world for read consumers, but publication ownership must remain:

```text
canonical facts = parent authority
local facts     = workspace publication
```

This distinction must remain inspectable for P6 and P7.

---

# 17. P5.T8 — Define the formal canonical persistence inventory

## Purpose

Give P7 a normative, executable definition of what is projected into the semantic artifact.

P5 still does not encode bytes.

## Required code/document boundary

Introduce either:

```text
CanonicalCompilerImageCapabilities
CanonicalPersistenceInventory
CanonicalProductFamily
```

or an equivalent exhaustive representation.

Representative:

```rust
pub enum CanonicalPersistenceClass {
    RequiredSemantic,
    RequiredMetadata,
    EligibleQuery,
    OptionalPresentation,
    OptionalSourceMap,
    Rebuildable,
    ProcessLocal,
    NeverPersist,
}
```

This need not mirror every struct field one-for-one, but it must make omissions explicit.

## Required manifest information

The canonical image boundary must expose or derive:

```text
canonical semantic world identity
canonical module source/content fingerprint
module/interface identity
semantic model ABI
stable-identity ABI/version input
native surface ABI/version input
canonical type-prefix extent
canonical static-value lineage extent
query image census/capabilities
metadata profile + metadata interface fingerprint
optional tooling capabilities
```

Do not use:

```text
CARGO_PKG_VERSION alone
raw TypeStoreId
raw MetadataPoolId
pointer identity
```

as portable compatibility identity.

P7 will convert the contract into durable header fields and digests.

---

# 18. P5.T9 — Extract explicit query persistence classification

## Purpose

Turn P2's correct but embedded freeze-policy match into a reusable persistence contract.

## Current authority

```text
phalcom-semantic/src/canonical_image.rs
CanonicalSemanticQueryImage::freeze
```

currently directly matches query families into retained/rejected classes.

## Required refactor

Extract a function or enum such as:

```rust
pub enum CanonicalQueryPersistence {
    EligibleStructural,
    OpenWorld,
    SessionLocal,
    Unsupported,
}
```

with:

```rust
fn canonical_query_persistence(key: &QueryKey) -> CanonicalQueryPersistence
```

or family-level equivalent.

`freeze(...)` must continue to enforce:

```text
committed/current row
canonical origin
typed product family match
dependency closure retained
matching dependency fingerprints
acyclic contract
no open-world negative dependency
```

## P7 handoff

P7 should consume the same classification when deciding which query DTOs may be encoded.

There must not be two independent lists:

```text
one list for in-memory freeze
another list for artifact serialization
```

that can silently drift.

---

# 19. P5.T10 — Bind P4 canonical metadata authority into the compiler-image boundary

## Purpose

Make canonical metadata one explicit component of the image P7 will eventually persist.

## Hard prerequisite

P4 must supply:

```text
constructible canonical RuntimePublic metadata
stable declaration/callable/field indexes
cross-pool reflection
canonical dependency identity
compatibility/conflict validation
```

## Layering rule

Do not create a dependency from:

```text
phalcom-semantic → phalcom-core
```

merely to place runtime metadata inside `CanonicalUniverseFormalImage`.

Semantic authority remains semantic-owned.

The core/compiler layer may expose a higher-level view that composes:

```text
CanonicalUniverseFormalImage
+
canonical CompiledProgram metadata authority
+
canonical module/compiler compatibility identity
```

Prefer a non-duplicating view over a second independently built semantic cache.

Representative shape:

```rust
pub struct CanonicalCompilerImageView<'a> {
    pub formal: &'a CanonicalUniverseFormalImage,
    pub metadata: &'a SemanticMetadataBundle,
    pub compatibility: &'a CanonicalCompatibilityIdentity,
}
```

Names are illustrative.

## Mandatory invariant

There must be exactly one canonical semantic world represented.

Do not independently rebuild a semantic snapshot for the core-level image.

---

# 20. P5.T11 — Observability and structural cardinality guards

## Purpose

Make architectural regressions visible before wall-clock regressions.

## Add/retain phases

The current canonical image phases include:

```text
CanonicalImageCacheLookup
CanonicalImageBuild
CanonicalImageBootstrap
CanonicalImageValidate
CanonicalImageQueryFreeze
```

Add subphase measurement where needed for:

```text
canonical module capture
canonical product publication
canonical source-semantic index
canonical presentation
canonical advisory
canonical snapshot publication
canonical aggregate-parent attachment
local overlay initialization
```

## Required counters

At minimum make it possible to assert:

```text
canonical source parses during warm session creation            = 0
canonical semantic updates during warm session creation         = 0
canonical aggregate entries deep-copied into warm session       = 0
canonical parent handles attached                               > 0
canonical presentation modules built in compiler-image mode     = expected bounded value
canonical advisory products built in compiler-image mode        = expected bounded value
canonical query rows examined during warm session creation      = 0
```

After T5/T6:

```text
CanonicalAggregateEntriesCopied = 0
```

for `SemanticWorkspaceSession::from_image()` is the preferred hard gate.

If some aggregate cannot yet be parented, record it explicitly rather than hiding it in a total duration.

---

# 21. P5.T12 — Verification matrix

P5 needs semantic, structural, tooling, and performance proof.

## 21.1 Request-contract tests

Test:

```text
Default == Full remains true unless explicitly amended
runtime_minimal contract unchanged
canonical_compiler_image contract explicit
canonical contract independent of local session request
```

---

## 21.2 Canonical product differential tests

Construct the canonical world with:

```text
full
canonical_compiler_image
```

Compare all compiler-authoritative families.

Differences are allowed only for families classified optional/rebuildable.

No compiler-required stable identity may differ.

---

## 21.3 Novel user-program differential suite

Compile novel user source against both canonical builds exercising:

```text
multiple Universe imports
re-exports
nominal types
generic types
generic callables
traits
associated types/members
conformance evidence
derived conformance
data
enums
static/type-level forms
field access
reflection where RuntimePublic permits
```

Required:

```text
same semantic success/failure
same selected declarations/callables
same conformance/evidence results
same lowering-relevant products
same runtime metadata identities
```

P5 need not prove source-free startup; that is P7/P9.

---

## 21.4 Full tooling compatibility

Run focused source-index/editor/advisory tests with Full publication.

Prove P5 did not optimize by deleting expected tooling behavior.

---

## 21.5 Parent-overlay semantic tests

For each converted table:

```text
canonical exact lookup works
local exact lookup works
merged iteration includes both
local removal does not affect canonical
local rollback does not affect canonical
local invalidation does not mutate canonical
clone copies local state only
duplicate canonical/local stable ID is rejected
module removal affects local module only
```

---

## 21.6 Session lifecycle tests

Create:

```text
canonical image
workspace A
workspace B
```

Mutate A.

Verify:

```text
canonical unchanged
B unchanged
A local state changes
A and B still resolve canonical identities identically
```

---

## 21.7 Query-image tests

Preserve P2 tests for:

```text
foreign world rejection
type-prefix compatibility
static-value lineage compatibility
open-world query rejection
mutable dependency rejection
cycle rejection
typed-product family validation
census accounting
```

Add tests proving T9 classification and `freeze()` agree.

---

## 21.8 P4 metadata integration tests

After P4 is unblocked:

```text
canonical metadata registered once
local bundle excludes canonical declaration records
local canonical references resolve by stable identity
cross-pool reflection succeeds
metadata pool numeric ordering is irrelevant
canonical compatibility mismatch rejects materialization
```

---

## 21.9 Performance matrix

Measure at minimum:

```text
cold canonical image construction
warm canonical image lookup
warm SemanticWorkspaceSession::new()
first tiny local compile
second retained tiny compile
multi-module user compile
Full tooling session
canonical compiler-image build
```

Report separately:

```text
wall time
phase time
structural counters
result status
test/build profile
thread count
revision
```

Do not compare a failing early fixture to a successful end-to-end compilation as equivalent performance evidence.

---

# 22. Expected implementation slices and commit boundaries

P5 should be reviewable.

## Slice A — explicit publication contract

```text
T0
T1
T2
T3
```

Expected files:

```text
request.rs
canonical_image.rs
focused tests
lifecycle docs
```

No aggregate ownership refactor yet.

---

## Slice B — consumer-specific publication completion

```text
T4
```

Expected files:

```text
request.rs
session.rs
source-index/formal publication helpers
advisory publication helpers
focused transition tests
```

Goal:

```text
canonical compiler image no longer pays unproven editor/advisory work
```

---

## Slice C — canonical parent core tables

```text
T5
T6 slice 1
T7 for converted tables
```

Start with:

```text
declarations
callable signatures
hierarchy
```

Do not convert every semantic family until this pattern is proven.

---

## Slice D — remaining semantic parent families

```text
T6 slices 2–4
T7 completion
```

Convert:

```text
dispatch
data
enum
associated
type-member
residual copied baseline maps
```

Each slice must carry copy-cardinality evidence.

---

## Slice E — persistence contract

```text
T8
T9
```

No byte encoding.

Deliver:

```text
exhaustive persistence classification
compatibility/capability boundary
single query eligibility authority
```

---

## Slice F — metadata composition and certification

```text
T10
T11
T12
T13
```

This slice is P4-dependent.

---

# 23. Required code-quality constraints

## 23.1 No giant semantic-state wrapper with hidden copying

A type named "overlay" is insufficient if construction clones the parent maps.

Tests/counters must prove physical canonical entry copying does not occur.

## 23.2 No dynamic trait-object indirection in hot semantic lookup without evidence

Prefer static table APIs and explicit parent handles unless measurement shows a need for another abstraction.

## 23.3 Deterministic iteration

Artifact-bound canonical inventories must not depend on randomized `HashMap` iteration.

Where deterministic merged iteration matters, sort or use stable indexes explicitly.

Do not globally replace all semantic `HashMap`s with `BTreeMap`s merely for future serialization; P7 can project into deterministic DTO ordering.

## 23.4 No process-global mutable canonical state

Canonical image state is:

```text
OnceLock published
immutable after publication
```

Do not add mutable global caches that can change artifact meaning after image publication.

## 23.5 No source-hidden assumptions yet

P5 source-backed construction may still use canonical source.

However, every P5 product required by P7 must have a clear source-independent representation contract.

If novel downstream compilation fundamentally requires raw canonical source bodies, STOP and define the missing typed-template contract.

---

# 24. STOP / CONSULT conditions

Stop implementation and escalate if any of the following occurs.

## 24.1 P4 authority remains unavailable at metadata-composition stage

Do not fake canonical metadata with:

```text
pool 0 assumptions
temporary local duplication
reflection-disabled test paths
```

P5.T10 remains blocked.

## 24.2 Canonical request cannot omit Full products without breaking legal compiler operations

Classify the actual dependency.

Do not simply restore Full and declare P5 complete.

Determine whether the required product is:

```text
compiler authority
or
an accidental dependency on a tooling representation
```

and fix ownership accordingly.

## 24.3 Parent-overlay conversion requires copying canonical maps on first mutation

The design has failed INV-P5-05/06.

Stop and redesign the table boundary.

## 24.4 Canonical/local ID collision is observed

Do not define implicit local shadowing.

Investigate stable identity/project ownership.

## 24.5 Open-world conformance needs a frozen negative result

Do not persist or parent-read an unsound proof result.

Preserve P2's rejection model and define an explicit dependency contract.

## 24.6 P7 would require raw process IDs

Do not bless those IDs as durable merely because canonical construction is deterministic.

Define relocation.

## 24.7 Source AST is still required for a semantic compiler operation intended to work from P7 artifact state

Record the missing portable typed-template product before P7 implementation.

Do not discover this only after the serializer is written.

---

# 25. Explicit non-goals

P5 does not implement:

```text
.telosc byte encoding
artifact section offsets
artifact filesystem cache
toolchain installation
content-addressed cache layout
VM bytecode portability
heap snapshots
initializer bytecode persistence
native machine code
general third-party package caching
all backend index optimization
all compiler performance work
```

Those belong to P6–P9 or successor checkpoints.

---

# 26. Acceptance gates

P5 is complete only if all of the following are true.

## G1 — P4 dependency closed

```text
canonical RuntimePublic metadata authority is constructible
cross-pool metadata model passes focused tests
```

## G2 — explicit canonical request

```text
canonical image no longer depends on SemanticProductRequest::default()
```

## G3 — canonical completeness

A canonical compiler-image build supports the same legal novel user compilation behavior as Full for compiler-relevant semantics.

## G4 — tooling preservation

Full mode preserves source/presentation/advisory behavior.

## G5 — no warm canonical analysis

After canonical image publication:

```text
canonical semantic update = 0
canonical source parse = 0
canonical query freeze = 0
```

for ordinary warm session creation.

## G6 — no warm canonical aggregate deep copy

Preferred structural gate:

```text
canonical aggregate entries copied in SemanticWorkspaceSession::from_image() = 0
```

Any exception requires explicit plan amendment.

## G7 — immutable parent safety

Local mutation/rollback/cancellation cannot mutate canonical aggregates or query roots.

## G8 — explicit persistence inventory

Every canonical product family needed by P7 has:

```text
persistence class
ownership
identity requirements
relocation requirement
consumer list
```

## G9 — single query eligibility authority

In-memory canonical query freeze and P7 persistence planning use one classification.

## G10 — evidence quality

All performance claims distinguish:

```text
cold canonical acquisition
warm lookup
session creation
semantic update
metadata export
backend lowering
VM bootstrap
```

and identify successful vs failed end-to-end tests.

---

# 27. Recommended focused commands

Exact selectors must be adjusted to final test names, but P5 should maintain a focused progression.

```sh
# Core semantic request/canonical-image checks
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-semantic --lib canonical_image
RUSTFLAGS='' cargo test -p phalcom-semantic --lib request

# Parent aggregate unit tests
RUSTFLAGS='' cargo test -p phalcom-semantic --lib declarations::tests
RUSTFLAGS='' cargo test -p phalcom-semantic --lib signature::tests
RUSTFLAGS='' cargo test -p phalcom-semantic --lib types::relation

# Incremental/session behavior
RUSTFLAGS='' cargo test -p phalcom-semantic --test incremental

# Core compiler integration
RUSTFLAGS='' cargo check -p phalcom-core --features perf-observability
RUSTFLAGS='' cargo test -p phalcom-core --lib modules::canonical_universe
RUSTFLAGS='' cargo test -p phalcom-core --lib metadata_profile_is_explicit_and_independent_of_semantic_publication

# P4-dependent compiler/reflection gates once unblocked
RUSTFLAGS='' cargo test -p phalcom-core --test core compiler::product_optimization
RUSTFLAGS='' cargo test -p phalcom-core --test core reflection::
```

Do not treat a command as required evidence merely because it appears here; final lifecycle records must capture the exact command that actually ran.

---

# 28. Observability success shape

The desired post-P5 in-process profile is approximately:

```text
FIRST PROCESS ACCESS
    source-backed canonical image build
    expensive, still present
    explicit compiler-image products only

WARM CANONICAL LOOKUP
    tiny

WARM WORKSPACE CREATION
    attach Arc canonical parents
    fork canonical TypeStore prefix
    attach immutable query image
    allocate empty/small local deltas
    no canonical aggregate map clone

LOCAL UPDATE
    user-owned semantic work
    canonical exact lookups/read-through
    no canonical presentation/advisory rebuild
```

P7 then transforms:

```text
source-backed canonical image build
```

into:

```text
universe.telosc read + validate + relocate
```

---

# 29. P6 handoff contract

P6 receives:

```text
immutable canonical parent aggregates
local-only mutable overlay aggregates
module/owner indexes from P3/P4
explicit merged/local iteration APIs
observability distinguishing canonical scans from local scans
```

P6 must then eliminate backend patterns where compiling one local module still performs:

```text
scan all canonical declarations
scan all canonical callable analyses
scan all canonical trait surfaces
scan all canonical lowering products
```

P5 should not preempt that work except where an index/view is required for parent composition.

---

# 30. P7 handoff contract

P7 receives an in-memory canonical compiler image with:

```text
explicit module authority
explicit semantic authority
explicit canonical metadata authority
explicit eligible query products
explicit persistence classes
explicit compatibility inputs
no dependence on local publication history
no requirement to serialize SemanticDb
no requirement to serialize raw VM/runtime handles
```

P7 then introduces:

```text
.telosc sectioned container
artifact-local structural IDs
relocatable semantic DTOs
deterministic encoding
digests
fresh-process load
source-unavailable semantic compilation proof
```

---

# 31. Lifecycle deliverables at P5 completion

Create/update:

```text
CHECKPOINT.md
PERF003.C2.P5-walkthrough.md
PERF003.C2.P5-handoff.md
```

The walkthrough must record:

```text
starting revision
ending revision
task-by-task implementation
exact tests run
exact failed/deferred tests
copy/cardinality counters
cold/warm timings
product-family differences between Full and canonical request
P4 dependency disposition
remaining P6/P7 work
```

The handoff must state explicitly:

```text
P5 does not persist universe.telosc yet
P5 does not eliminate source-built cold canonical acquisition
P5 does make the canonical compiler authority serialization-ready
```

---

# 32. Final P5 acceptance statement

The correct completion statement is:

> **The canonical Universe is now represented as an explicit, immutable compiler authority with a reviewed consumer-specific publication contract. Ordinary semantic sessions attach canonical aggregate parents and maintain user-owned local deltas instead of reconstructing or deep-copying the canonical world. Canonical query persistence eligibility and product-family persistence classes are explicit, and the P4 RuntimePublic metadata authority composes into the same canonical compiler-image boundary. The image remains source-built in P5; P7 is responsible for encoding and loading this authority as `universe.telosc`.**

Do **not** claim:

```text
compiler only compiles user code
```

at P5.

That statement requires P7 semantic persistence, P8 portable executable Universe state, and P9 artifact-first startup certification.

---

# Appendix A — Immediate implementation order

When P4 entry prerequisites are resolved, execute P5 in this order:

```text
1. P5.T0 lifecycle/measurement entry gate
2. P5.T1 explicit canonical request seam
3. P5.T2 exhaustive product classification
4. P5.T3 canonical bootstrap explicit request
5. P5.T4 source/presentation/advisory publication separation
6. P5.T5 parent-table abstraction
7. P5.T6 declaration/signature/hierarchy conversion
8. verify semantics + copy counters
9. P5.T6 dispatch/data/enum/associated conversion
10. P5.T7 snapshot/read hardening
11. P5.T8 persistence inventory/capabilities
12. P5.T9 query classification extraction
13. P5.T10 bind P4 metadata authority
14. P5.T11 observability gates
15. P5.T12 full focused parity/performance matrix
16. P5.T13 lifecycle closure and P6/P7 handoff
```

Do not begin with the largest aggregate migration.

Prove the parent pattern on declaration/signature/hierarchy tables first.

---

# Appendix B — Current P4/P5 boundary

The committed branch at `e74a9213` establishes:

```text
ProgramCompilationOptions.metadata_profile
compile_analyzed_with_options
compile_fixture_with_options
P4 metadata root/closure engine
P4 callable/field owner indexes
P4 metadata observability definitions
```

It does **not** yet establish:

```text
production reachability-driven metadata export
RuntimePublic field export
constructible canonical RuntimePublic metadata authority
runtime stable-identity metadata indexes
cross-pool reflection
canonical metadata dependency compatibility
```

Those remain P4 responsibilities.

P5 must consume their completed architecture, not silently redefine it.

---

# Appendix C — Main code references at plan preparation

Prepared against:

```text
e74a92132bab6a7b7682a047c05dc01ab4db8b28
```

Primary references:

```text
phalcom-semantic/src/request.rs
  SemanticProductRequest
  PresentationLevel
  AdvisoryLevel

phalcom-semantic/src/canonical_image.rs
  CanonicalUniverseFormalImage
  CanonicalSemanticQueryImage
  canonical_universe_formal_image
  build
  validate_publication

phalcom-semantic/src/session.rs
  SemanticWorkspaceSession
  from_image
  from_foundation_for_image
  update

phalcom-semantic/src/snapshot.rs
  SemanticSnapshot
  ModuleQueryProducts

phalcom-semantic/src/db/mod.rs
  SemanticDb::with_canonical_image
  canonical read-through

phalcom-semantic/src/declarations.rs
  DeclarationTypeTable

phalcom-semantic/src/signature.rs
  CallableSignatureTable
  FieldSignatureTable

phalcom-semantic/src/types/relation.rs
  MapTypeHierarchy

phalcom-modules/src/canonical_products.rs
  CanonicalModuleProducts

phalcom-core/src/modules/compile.rs
  ProgramCompilationOptions
  ProgramCompiler
  compile_analyzed_with_options
  project_analyzed_inner

phalcom-core/src/modules/compile_session.rs
  ProgramCompilationSession
  compile_fixture_with_options

phalcom-core/src/modules/canonical_universe.rs
  CanonicalUniverseProgram
  canonical_universe_program
  build_canonical_universe_program

phalcom-semantic/src/metadata/reachability.rs
  MetadataRootSet
  MetadataRetentionRequest
  MetadataReachabilityClosure
  compute_reachability_closure

phalcom-semantic/src/metadata/export.rs
  MetadataExporter::build_bundle
  P4 pending production cutover

phalcom-core/src/typing/registry.rs
phalcom-core/src/typing/inspect.rs
  P4 pending cross-pool runtime authority
```

---

# Appendix D — Evidence discipline

This plan distinguishes:

```text
committed source fact
reported test evidence
architectural requirement
proposed implementation shape
```

It does not claim that:

```text
P4 is complete
ProductKind bootstrap is repaired
canonical RuntimePublic metadata is currently constructible
the 44–45 second cold build improved at e74a9213
P5 has been tested
.telosc exists
source-free Universe compilation exists
```

Those claims require later implementation evidence.
