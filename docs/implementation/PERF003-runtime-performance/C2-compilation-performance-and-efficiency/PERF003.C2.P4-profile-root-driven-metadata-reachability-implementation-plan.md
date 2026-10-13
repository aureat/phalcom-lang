# PERF003.C2.P4 — Profile/Root-Driven Metadata Reachability Architecture
## Luna Patch-Grade Detailed Implementation Plan

**Program:** `PERF003`  
**Checkpoint:** `PERF003.C2`  
**Plan:** `PERF003.C2.P4`  
**Package:** Package D — Profile/Root-Driven Metadata Reachability Architecture  
**Assumption:** `PERF003.C2.P1`–`P3` are complete, verified, and closed before this plan starts.  
**Repository:** `aureat/phalcom-lang` / `phalcom`  
**Primary branch context:** `perf/universe-characterization`  
**Status at authoring:** `PROPOSED / NOT_STARTED / UNVERIFIED`

## Accepted prerequisite amendment — 2026-10-13

The [user's architectural ruling](PERF003.C2.P4-architectural-ruling.md)
authorizes T5 continuation and adds two bounded T6 prerequisites:
`P4.PR-A` intrinsic source-index category repair and `P4.PR-B` versioned
heterogeneous Record metadata (fixed keys, remainder domains, scoped/row forms,
fingerprints, legacy decoding). T1 defaults and end-to-end assertions stand.
Export failures must preserve retained entity/root and recursive type context.
T6 certification and T7–T9 remain gated on usable, faithful canonical
RuntimePublic metadata. The ruling's acceptance table is binding.

Execution was explicitly stopped by the user after a partial T5 cutover and
focused-tested PR-A classification repair. PR-B remains investigation only.
Consume the current `PERF003.C2.P4-handoff.md` and CHECKPOINT before resuming;
the patch is uncommitted over `e74a9213` and is not full P4 certification.

**Continuation authorization, 2026-10-13:** The user resumed the P4
prerequisites needed for P5 acceptance and approved decode-time normalization
of pre-v13 metadata: introduce canonical Symbol key-domain nodes, relocate all
affected global/scoped node references, and validate compatibility and
round-trip fidelity. This amends the historical stop above; the older handoff
still records the earlier stopping point. No P4 completion follows merely from
this authorization.

**Identity clarification:** A legacy bundle need not contain a Symbol type node
and its header does not encode the Universe Symbol declaration/version. The
user selected a loader-supplied validated canonical Symbol identity for the
normalization. Standalone decode of affected legacy Record metadata without
that context returns a typed need-for-context error; old bundles without
affected Record data continue to decode normally. Never infer Universe
identity from the producer version or a source-text spelling.

**Fingerprint clarification:** Retain old structural fingerprints as
version-tagged legacy provenance when normalizing v12 Record graphs. They
must not compare equal to v13 semantic-export fingerprints until the semantic
authority re-exports the normalized graph. `phalcom-type-meta` must not
independently reconstruct the semantic exporter's hashing algorithm.

---

# 1. Purpose

Implement the metadata architecture required by `PERF003.C2` so that compilation no longer serializes or scans whole-world semantic metadata when only a bounded executable/public subset is required.

The target model is:

```text
SemanticSnapshot
    +
MetadataProfile
    +
explicit metadata roots
    +
canonical metadata authority
        ↓
MetadataReachabilityClosure
        ↓
reachable local metadata export only
        +
stable references to canonical metadata
        ↓
runtime metadata pools
        ↓
cross-pool semantic resolution through stable identities
```

This package eliminates three classes of waste and architectural ambiguity:

1. **profile-insensitive metadata export**;
2. **quadratic declaration × callable ownership scans**;
3. **duplication of canonical Universe metadata into ordinary user bundles**.

P4 must preserve runtime reflection semantics, stable semantic identity, type identity, generic metadata, runtime type-form roots, and VM materialization behavior.

---

# 2. Normative outcome

After P4:

```text
RuntimeMinimal metadata work
    ∝ executable metadata roots

RuntimePublic metadata work
    ∝ local public surface
    + executable metadata roots
    + reachable local dependencies

Canonical Universe metadata
    exported once
    installed once
    not duplicated into each ordinary program

Declaration → callable/field association
    indexed
    not O(declarations × all callables)

Runtime reflection
    resolves local and canonical metadata
    by stable semantic identity
```

The package is complete only when metadata work is structurally bounded by retained roots rather than aggregate snapshot size.

---

# 3. Current source-grounded problems

## 3.1 `MetadataProfile` is currently descriptive, not operational

Current owner:

```text
phalcom-type-meta/src/header.rs
```

Current enum:

```rust
pub enum MetadataProfile {
    RuntimeMinimal,
    RuntimePublic,
    ToolingDebug,
    Proof,
}
```

Current exporter:

```text
phalcom-semantic/src/metadata/export.rs
MetadataExporter
MetadataExporter::build_bundle
```

`MetadataExporter` stores a `MetadataProfile`, but the profile is currently used materially only when constructing `SemanticMetadataHeader`.

The exporter still traverses aggregate semantic tables:

```text
declarations
aliases
callables
fields
```

rather than selecting and traversing only retained entities.

This violates the existing metadata contract documented under the runtime typing metadata specification:

```text
Metadata export is root-driven.
It must never serialize the entire TypeStore merely because nodes exist there.
```

## 3.2 Metadata export contains an O(D × C) ownership scan

Current code in:

```text
phalcom-semantic/src/metadata/export.rs
MetadataExporter::build_bundle
```

contains the equivalent of:

```rust
for (decl_id, info) in decls.iter() {
    ...
    if let Some(callables) = self.callables {
        for (callable, _) in callables.iter() {
            if !matches!(
                &callable.owner,
                CallableOwnerId::Declaration(owner) if owner == decl_id
            ) {
                continue;
            }
            ...
        }
    }
}
```

This is directly observable through:

```text
telos_observability::PerfCounter::MetadataCallableOwnerComparisons
```

The target must be indexed owner lookup.

## 3.3 Runtime/compiler metadata profile is hard-coded

Current ordinary compilation uses:

```text
MetadataProfile::RuntimePublic
```

inside:

```text
phalcom-core/src/modules/compile.rs
ProgramCompiler
```

There is no explicit compiler metadata-retention input.

This is architecturally distinct from:

```text
SemanticProductRequest::runtime_minimal()
```

Semantic product publication and serialized/runtime metadata retention are separate axes and must remain separate.

## 3.4 RuntimePublic metadata is currently incomplete

The RuntimePublic specification requires public field semantic signatures.

`SemanticSnapshot` already publishes:

```rust
pub field_signatures: Arc<FieldSignatureTable>
```

but normal compiler metadata construction currently supplies:

```text
fields: None
```

to `MetadataExporter`.

P4 must make the profile contract real rather than optimize an incomplete dump.

## 3.5 Canonical metadata is installed separately but user bundles remain self-contained

Runtime metadata pools already exist:

```text
phalcom-core/src/typing/handle.rs
MetadataPoolId
RuntimeTypeRef
RuntimeSemanticHandle
```

and:

```text
phalcom-core/src/typing/registry.rs
RuntimeTypingRegistry
```

Canonical Universe metadata is installed during VM bootstrap before normal user metadata.

However, ordinary bundles still behave largely as independent metadata worlds.

P4 must turn the canonical pool into an explicit external metadata authority.

## 3.6 Reflection still contains same-pool assumptions and linear pool scans

Examples exist in:

```text
phalcom-core/src/typing/inspect.rs
```

including:

```text
declaration_record_for_class
base_constructor_arity
method/signature lookup paths
```

Some paths scan all pools and all declaration records.

Other paths assume a `TypeNode::Nominal` declaration record lives in the same bundle/pool as the type node.

That assumption is incompatible with cross-pool canonical reuse.

---

# 4. Architectural invariants

## INV-P4-01 — Profiles are retention policy, not labels

`MetadataProfile` must determine which semantic entities are roots and which metadata sections may be retained.

A profile must never be a header-only annotation.

## INV-P4-02 — Semantic analysis request and metadata retention remain separate

Do not derive metadata retention implicitly from `SemanticProductRequest`.

Allowed:

```text
semantic request: runtime-minimal
metadata profile: RuntimePublic
```

and:

```text
semantic request: runtime-minimal
metadata profile: RuntimeMinimal
```

must both be valid compiler configurations.

## INV-P4-03 — Reachability is root-driven

Production metadata export may not discover retained RuntimeMinimal entities by scanning the entire semantic world and filtering afterward.

The root set must be explicit and testable.

## INV-P4-04 — Stable semantic identity is the cross-pool authority

Persistent/compiler-facing cross-pool identity must use stable semantic identities and compatibility fingerprints.

VM-local numeric `MetadataPoolId` must not become durable artifact identity.

## INV-P4-05 — Child node IDs remain pool-local unless a schema migration explicitly changes them

Current `TypeNodeId`, `KindNodeId`, `GenericSignatureRecordId`, and similar indexes are pool-local.

Do not casually convert the entire metadata schema to pool-qualified node IDs.

P4 should first eliminate duplicated high-level canonical records through stable identity lookup.

## INV-P4-06 — Canonical metadata is immutable external authority

Canonical Universe metadata must be installed once and referenced as an immutable metadata authority.

User metadata must not shadow or mutate canonical metadata.

## INV-P4-07 — Reflection authority is distinct from metadata presence

A RuntimeMinimal artifact may execute using canonical metadata while still not granting arbitrary public reflection.

Preserve:

```text
TypingContextData
TypingCapabilities
TypingCapabilities::for_profile
```

semantics.

## INV-P4-08 — RuntimePublic remains semantically complete

RuntimePublic must retain the documented public reflection surface:

```text
public declaration forms and kinds
generic signatures/parameters/constraints
public superclass templates
public callable signatures
public field signatures
transparent public aliases
required public presentation names
runtime callable mappings where materialized
```

## INV-P4-09 — Metadata fingerprints describe retained artifact semantics

Unreachable compiler scratch state must not perturb retained metadata fingerprints.

Retained public/executable semantic changes must perturb the appropriate fingerprint.

## INV-P4-10 — No quadratic declaration-owner reconstruction

Owner relationships must be explicit indexes.

---

# 5. Ownership boundaries

| Responsibility | Owner |
|---|---|
| Stable metadata schema | `phalcom-type-meta` |
| `MetadataProfile` | `phalcom-type-meta/src/header.rs` |
| Semantic metadata graph export | `phalcom-semantic/src/metadata/*` |
| Metadata reachability | `phalcom-semantic/src/metadata/reachability.rs` |
| Callable/field owner indexes | `phalcom-semantic/src/signature.rs` |
| Executable/public root selection | `phalcom-core/src/modules/compile.rs` |
| Canonical metadata production | `phalcom-core/src/modules/canonical_universe.rs` / canonical `CompiledProgram` |
| Runtime metadata loading | `phalcom-core/src/typing/loader.rs` |
| Runtime metadata registry/indexes | `phalcom-core/src/typing/registry.rs` |
| Runtime reflection lookup | `phalcom-core/src/typing/inspect.rs` |
| Program metadata installation | `phalcom-core/src/modules/materialize.rs` |
| Performance metrics | `telos-observability` |

---

# 6. Package task graph

```text
P4.T0  Baseline characterization
   ↓
P4.T1  Compiler metadata request/profile plumbing
   ↓
P4.T2  Metadata root model + retention policy
   ↓
P4.T3  Reachability closure engine
   ↓
P4.T4  Owner indexes
   ↓
P4.T5  Reachability-driven exporter cutover
   ↓
P4.T6  Canonical metadata external-authority model
   ↓
P4.T7  Runtime stable-identity indexes
   ↓
P4.T8  Remove same-pool reflection assumptions
   ↓
P4.T9  Dependency compatibility / conflict policy
   ↓
P4.T10 Observability and cardinality guards
   ↓
P4.T11 Full parity, benchmark, lifecycle closure
```

Do not begin T6–T9 before T2–T5 establish correct local retention.

---

# 7. P4.T0 — Establish post-P3 baseline

## Purpose

Measure the post-P3 metadata cost before edits.

P4 must not reuse older pre-P3 measurements as current evidence.

## Required measurements

Measure at minimum:

```text
cold canonical compiler process
warm one-shot tiny fixture
warm retained-session tiny fixture
small multi-module project
RuntimePublic build
explicit RuntimeMinimal build once T1 exists
```

Capture:

```text
MetadataExport phase duration
MetadataExportDeclarations
MetadataExportAliases
MetadataExportCallables
MetadataExportFields
MetadataCallableOwnerComparisons
metadata bundle record counts
encoded metadata size
whole compile wall time
```

Also capture:

```text
total snapshot declaration count
total snapshot callable count
total field count
local-module declaration count
local-module callable count
canonical-module declaration count
canonical-module callable count
```

## Exit criterion

A checked-in or lifecycle-recorded baseline exists for comparison.

---

# 8. P4.T1 — Add explicit compiler metadata-retention configuration

## Purpose

Make metadata profile an explicit compiler input.

## Primary files

```text
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/compile_session.rs
phalcom-core/src/modules/mod.rs
```

Potential supporting owner:

```text
phalcom-core/src/modules/artifact.rs
```

if compilation options already belong there after P3.

## Required shape

Introduce a repository-consistent request/configuration object.

Representative form:

```rust
#[derive(Clone, Debug)]
pub struct ProgramCompilationOptions {
    pub metadata_profile: MetadataProfile,
}
```

or:

```rust
pub struct CompilerProductRequest {
    pub semantic_products: SemanticProductRequest,
    pub metadata_profile: MetadataProfile,
}
```

The exact public API may vary, but these invariants are mandatory:

```text
existing compile APIs default to RuntimePublic
tests/benchmarks can explicitly request RuntimeMinimal
metadata profile is not inferred from semantic publication request
profile flows to MetadataExporter through one explicit path
```

## Forbidden approaches

Do not:

```text
replace every RuntimePublic literal with RuntimeMinimal
derive metadata profile from perf-observability feature
derive metadata profile from source size
use cfg(test) to create a special metadata path
```

## Tests

Add focused compiler option tests proving:

```text
default compile → RuntimePublic metadata header
explicit RuntimeMinimal → RuntimeMinimal metadata header
semantic runtime-minimal + metadata RuntimePublic is accepted
```

---

# 9. P4.T2 — Introduce explicit metadata roots and retention policy

## Purpose

Define which semantic entities enter the metadata closure.

## Create

```text
phalcom-semantic/src/metadata/reachability.rs
```

## Modify

```text
phalcom-semantic/src/metadata/mod.rs
```

## Required model

Introduce explicit roots:

```rust
#[derive(Clone, Debug, Default)]
pub struct MetadataRootSet {
    pub runtime_types: BTreeSet<TypeId>,
    pub declarations: BTreeSet<DeclarationId>,
    pub aliases: BTreeSet<DeclarationId>,
    pub callables: BTreeSet<CallableId>,
    pub fields: BTreeSet<FieldId>,
}
```

Introduce retention context/policy:

```rust
pub struct MetadataRetentionRequest<'a> {
    pub profile: MetadataProfile,
    pub roots: &'a MetadataRootSet,
}
```

A trait-based policy is acceptable if useful:

```rust
pub trait MetadataRetentionPolicy {
    fn profile(&self) -> MetadataProfile;
    fn roots(&self) -> &MetadataRootSet;
}
```

but the selected roots must remain concretely enumerable.

## Core root construction

`phalcom-core/src/modules/compile.rs` must construct roots from authoritative compiler facts.

### RuntimeMinimal roots

Seed from existing executable semantic facts such as:

```text
runtime_root_specs
runtime_call_root_specs
runtime_trait_root_specs
runtime type-form constants
runtime product/data exact-type roots
runtime generic/conformance environment requirements
explicit validators/contracts required by existing runtime semantics
```

### RuntimePublic roots

Add:

```text
public exported local declarations
public callable semantic surfaces
public field semantic surfaces
transparent public aliases
public superclass/type/generic surfaces
```

Use canonical export/visibility authorities.

Do not infer publicness from naming conventions.

## Canonical filtering

Canonical Universe entities should normally be classified as external rather than inserted as local declaration/callable/field roots.

---

# 10. P4.T3 — Implement `MetadataReachabilityClosure`

## Purpose

Convert roots into the complete retained top-level semantic entity set.

## Required structure

Representative:

```rust
#[derive(Clone, Debug, Default)]
pub struct MetadataReachabilityClosure {
    pub declarations: BTreeSet<DeclarationId>,
    pub aliases: BTreeSet<DeclarationId>,
    pub callables: BTreeSet<CallableId>,
    pub fields: BTreeSet<FieldId>,
    pub runtime_types: BTreeSet<TypeId>,
}
```

## Algorithm

Use a deterministic worklist.

```text
seed roots
  ↓
enqueue entity
  ↓
visit direct semantic dependencies
  ↓
enqueue unseen dependencies
  ↓
repeat until fixed point
```

The closure engine must not repeatedly scan all semantic tables.

## Required semantic dependencies

### Declaration

Retain dependencies needed to serialize:

```text
type form
kind
generic signature
generic parameters
generic constraints
superclass template
profile-visible owned callables
profile-visible owned fields
algebraic/static-domain metadata where applicable
```

### Callable

Retain:

```text
generic signature
parameter type facts
result type fact
stable owner identity
```

### Field

Retain:

```text
field type fact
stable owner identity
```

### Alias

Retain:

```text
generic signature
target type form
```

### Runtime type

Type/kind child recursion remains owned by existing `MetadataExporter::export_type_form` / kind export.

Do not duplicate type-graph traversal in two authorities.

#### Accepted T5 dependency-closure amendment — 2026-10-13

The exporter reports nominal declaration references encountered while
recursively exporting the current closure's runtime type roots. For
`RuntimePublic`, the compiler adds newly discovered declarations to the root
set and recomputes the entity closure until the exporter reports no further
local declarations. The closure remains authoritative for profile visibility,
canonical external elision, and declaration/member expansion; it does not walk
`TypeData` children itself. `RuntimeMinimal` retains its existing declaration
policy and does not expand nominal references into declaration metadata.

Use a disposable exporter configured with the same semantic inputs for
discovery, so its partial output cannot affect final bundle node ordering. Only
fully exportable type roots contribute dependencies. Failed probes are retried
by the final selected-bundle export under the retained entity context, which
remains responsible for reporting projection errors and type provenance.
Aggregate closure work metrics across fixed-point rounds and publish retained
cardinalities from the final closure.

## Determinism

Closure iteration/output ordering must be deterministic.

Prefer:

```text
BTreeSet
stable IDs
stable work queue order
```

over insertion-order-dependent maps.

---

# 11. P4.T4 — Add callable and field owner indexes

## Purpose

Eliminate global owner scans.

## Modify

```text
phalcom-semantic/src/signature.rs
```

## CallableSignatureTable

Current shape includes:

```text
by_id
module_callables
```

Add:

```rust
owner_callables: HashMap<CallableOwnerId, BTreeSet<CallableId>>
```

Add API:

```rust
pub fn iter_owner<'a>(
    &'a self,
    owner: &CallableOwnerId,
) -> impl Iterator<Item = (&'a CallableId, &'a CallableSemanticSignature)> + 'a
```

or equivalent repository-consistent API.

### Mutation invariants

Update all of:

```text
insert
remove
remove_module
clone/equality behavior
```

so:

```text
by_id
module_callables
owner_callables
```

remain consistent.

## FieldSignatureTable

Assess the same need.

If RuntimePublic field reachability would otherwise require global scans, add:

```rust
owner_fields: HashMap<DeclarationId, BTreeSet<FieldId>>
```

and:

```rust
iter_owner(&DeclarationId)
```

## Tests

Add focused table invariants:

```text
insert maintains indexes
remove removes all index entries
remove_module removes all owner/module references
clone preserves indexes
owner lookup returns exact side/owner members
```

---

# 12. P4.T5 — Cut `MetadataExporter` over to the closure

## Purpose

Stop exporting aggregate semantic tables unconditionally.

## Modify

```text
phalcom-semantic/src/metadata/export.rs
```

## API migration

Current:

```rust
build_bundle(runtime_roots)
```

must gain an explicit request/closure.

Representative:

```rust
pub fn build_bundle(
    mut self,
    request: &MetadataExportRequest,
) -> Result<SemanticMetadataBundle, MetadataExportError>
```

or:

```rust
pub fn build_bundle_from_closure(
    mut self,
    closure: &MetadataReachabilityClosure,
    runtime_roots: &[RuntimeMetadataRoot],
) -> Result<SemanticMetadataBundle, MetadataExportError>
```

The API must make it impossible for production RuntimeMinimal compilation to accidentally fall back to whole-world iteration.

## Export loops

Convert:

```text
all declarations
all aliases
all callables
all fields
```

to:

```text
closure.declarations
closure.aliases
closure.callables
closure.fields
```

Look up each item by canonical ID.

## Project identity validation

Current pre-export identity validation walks all supplied tables.

Replace with closure-scoped validation.

Validation remains mandatory for every retained stable identity.

## Callable association

Declaration export must use owner indexes.

Target:

```text
MetadataCallableOwnerComparisons
≈ actual retained owner/callable relationships
```

not:

```text
D × C
```

## RuntimePublic fields

Wire:

```text
SemanticSnapshot::field_signatures
```

into normal RuntimePublic export.

Do not leave `fields: None`.

## Legacy direct exporter tests

Where tests intentionally construct `MetadataExporter` directly without a full snapshot, provide a small explicit root/closure path.

Do not retain a silent “export all” production default merely to avoid updating tests.

A clearly named test/helper path such as:

```text
build_complete_test_bundle
```

is acceptable if it is not used by compiler production paths.

---

# 13. P4.T6 — Introduce canonical metadata external authority

## Purpose

Stop copying canonical Universe declaration/callable/field metadata into ordinary program bundles.

## Source authority

Use the canonical compiled program:

```text
phalcom-core/src/modules/canonical_universe.rs
canonical_universe_program()
CompiledProgram::semantic_metadata
```

The canonical metadata bundle should be treated as the canonical RuntimePublic metadata authority.

## Required compiler model

Ordinary metadata export receives an external authority descriptor.

Representative:

```rust
pub struct ExternalMetadataAuthority {
    pub identity: MetadataDependencyIdentity,
    pub interface_fingerprint: Fingerprint128,
}
```

A full persisted dependency format is not required yet, but the compiler must know:

```text
canonical entities do not belong in this local bundle
canonical authority expected at runtime
```

## Canonical elision rule

For ordinary user compilation:

```text
canonical declaration records → not emitted locally
canonical callable records → not emitted locally
canonical field records → not emitted locally
```

unless a specific schema/runtime requirement proves a local record is required.

Every exception must be documented and counted.

## Type graph rule

A local bundle may still contain:

```rust
TypeNode::Nominal {
    declaration: StableDeclarationRef
}
```

for canonical declarations.

Stable nominal identity may refer to declaration metadata owned by another pool.

This is the preferred P4 mechanism before redesigning the type graph schema.

---

# 14. P4.T7 — Add stable-identity metadata indexes to the runtime registry

## Purpose

Make cross-pool lookup bounded and authoritative.

## Modify

```text
phalcom-core/src/typing/registry.rs
phalcom-core/src/typing/loader.rs
```

## Add pool-qualified record references

Representative:

```rust
pub struct RuntimeDeclarationMetadataRef {
    pub pool: MetadataPoolId,
    pub record: DeclarationRecordId,
}
```

Existing runtime handle types should be reused where already suitable.

Add indexes:

```rust
declaration_metadata:
    HashMap<StableDeclarationRef, RuntimeDeclarationMetadataRef>

callable_metadata:
    HashMap<StableCallableRef, RuntimeCallableSignatureRef>

field_metadata:
    HashMap<StableFieldRef, RuntimeFieldSignatureRef>
```

## Registration

When registering a pool:

```text
validate bundle
allocate/register pool
index declarations
index callables
index fields
detect conflicts
```

## Lookup APIs

Provide:

```rust
lookup_declaration_metadata(&StableDeclarationRef)
lookup_callable_metadata(&StableCallableRef)
lookup_field_metadata(&StableFieldRef)
```

These APIs become the runtime authority for stable semantic metadata lookup across pools.

---

# 15. P4.T8 — Remove same-pool reflection assumptions

## Purpose

Allow local metadata type nodes to refer by stable identity to canonical declaration metadata.

## Audit/modify

At minimum:

```text
phalcom-core/src/typing/inspect.rs
phalcom-core/src/typing/reify.rs
phalcom-core/src/typing/identity.rs
phalcom-core/src/typing/environment.rs
phalcom-core/src/modules/materialize.rs
```

## Required distinction

### Pool-local references

These remain same-pool:

```text
TypeNodeId
KindNodeId
ScopedTypeNodeId
GenericSignatureRecordId
CallableRecordId
FieldRecordId
```

when embedded as numeric indexes.

### Stable identities

These may resolve to another metadata pool:

```text
StableDeclarationRef
StableCallableRef
StableFieldRef
StableImplRef
other stable semantic owners
```

## Concrete required correction

Current logic analogous to:

```text
TypeNode::Nominal(declaration)
    → search current loaded.bundle.declarations
```

must become:

```text
TypeNode::Nominal(declaration)
    → registry.lookup_declaration_metadata(declaration)
    → pool-qualified declaration record
```

`base_constructor_arity` is a concrete example.

## Reflection lookup

Replace pool scans such as:

```text
for pool in registry.pools()
    for declaration in pool.declarations
```

with registry indexes.

---

# 16. P4.T9 — Canonical dependency compatibility and duplicate identity policy

## Purpose

Make cross-pool metadata safe and deterministic.

## Compatibility

A local program compiled against canonical world A must not silently run against incompatible canonical world B.

Use existing metadata header facts:

```text
semantic_model_version
native_surface_schema_version
identity_scheme
interface_fingerprint
source_fingerprint where semantically relevant
```

Introduce a narrow compiler/runtime dependency descriptor.

Representative:

```rust
pub enum MetadataDependencyIdentity {
    CanonicalUniverse {
        interface_fingerprint: Fingerprint128,
        semantic_model_version: u32,
        native_surface_schema_version: u32,
    },
}
```

Do **not** serialize `MetadataPoolId(0)` as dependency identity.

## Conflict policy

When loading a pool:

```text
same stable identity
+
same compatible semantic record
    → optionally tolerate during migration

same stable identity
+
incompatible semantic record
    → structured load error
```

Do not silently prefer first-loaded/last-loaded metadata.

## Canonical authority precedence

The expected normal state after P4 is:

```text
canonical Universe declaration/callable/field records
    canonical pool only

local declaration/callable/field records
    local pool only
```

---

# 17. P4.T10 — Observability and structural regression guards

## Modify

```text
telos-observability/src/metric.rs
telos-observability/src/phase.rs
```

only as needed.

Reuse existing metadata phases/counters where possible.

## Required new or clarified metrics

Add counters for:

```text
MetadataRootsSelected
MetadataEntitiesVisited
MetadataDependencyEdgesTraversed

MetadataDeclarationsRetained
MetadataDeclarationsRejectedByProfile

MetadataAliasesRetained
MetadataCallablesRetained
MetadataFieldsRetained

MetadataCanonicalDeclarationsElided
MetadataCanonicalCallablesElided
MetadataCanonicalFieldsElided

MetadataExternalLookups
MetadataExternalLookupFailures
MetadataIdentityConflicts
```

Keep existing:

```text
MetadataCallableOwnerComparisons
MetadataDeclarationsExported
MetadataAliasesExported
MetadataCallablesExported
MetadataFieldsExported
```

## Structural assertions

Tiny RuntimeMinimal fixture:

```text
whole-snapshot declaration sweep = 0
whole-snapshot callable sweep = 0
canonical declaration records emitted locally = 0
canonical callable records emitted locally = 0
canonical field records emitted locally = 0
```

RuntimePublic tiny fixture:

```text
retained local records ≈ public local API + runtime roots
owner comparisons ≈ retained owner edges
```

---

# 18. P4.T11 — Testing and certification

## 18.1 Unit-level semantic tests

Add or update tests for:

```text
MetadataRootSet construction
MetadataReachabilityClosure dependency traversal
callable owner index
field owner index
profile-specific retention
deterministic closure ordering
identity validation only on retained records
```

Suggested paths:

```text
phalcom-semantic/tests/semantic/metadata_reachability.rs
phalcom-semantic/tests/semantic/callable_owner_index.rs
phalcom-semantic/tests/semantic/field_owner_index.rs
```

## 18.2 Metadata profile matrix

Test:

| Case | Expected |
|---|---|
| RuntimeMinimal | execution roots only |
| RuntimePublic | RuntimeMinimal + public semantic API |
| ToolingDebug | RuntimePublic + debug/tooling records according to existing contract |
| Proof | ToolingDebug + allowed proof sections |
| RuntimeMinimal + proof-only section | reject or omit per existing policy |
| RuntimePublic + proofs section | reject per existing policy |

Do not invent new profile semantics in P4.

Consume existing metadata specifications.

## 18.3 Cross-pool runtime tests

Create focused tests under:

```text
phalcom-core/tests/core/compiler/cross_pool_metadata.rs
phalcom-core/tests/core/reflection/
```

Required scenarios:

### Case A — canonical nominal referenced by local type graph

```text
local callable parameter:
    UniverseGeneric<Int>
```

Expected:

```text
local pool owns callable
canonical pool owns canonical declaration metadata
reflection resolves generic declaration metadata correctly
```

### Case B — local public declaration with canonical superclass/template

Expected:

```text
local declaration metadata in local pool
canonical superclass metadata in canonical pool
declared supertype reflection succeeds
```

### Case C — local public callable with canonical parameter/result types

Expected:

```text
local callable metadata retained
canonical type identity preserved
no duplicated canonical callable/declaration records
```

### Case D — local public fields

Expected:

```text
RuntimePublic field semantic metadata present
RuntimeMinimal field metadata absent unless execution-required
```

### Case E — canonical method reflection

Expected:

```text
method semantic lookup resolves through canonical pool
```

### Case F — local method reflection

Expected:

```text
method semantic lookup resolves through local pool
```

### Case G — metadata identity conflict

Load incompatible duplicate stable declaration.

Expected:

```text
structured failure
```

### Case H — canonical dependency mismatch

Expected:

```text
structured compatibility failure before runtime use
```

## 18.4 RuntimeMinimal reflection authority test

Prove:

```text
RuntimeMinimal:
    executable runtime types work
    arbitrary local public reflection is unavailable

RuntimePublic:
    public declaration/signature discovery works
```

Canonical metadata being loaded must not escalate RuntimeMinimal reflection capability.

## 18.5 Type identity tests

Cross-pool type identity must remain semantic, not raw handle identity.

Test:

```text
canonical type represented by canonical pool handle
same nominal identity represented inside local composite graph
```

and ensure:

```text
typing::identity::equivalent
typing::inspect::subtype
typing::reify
```

retain correct behavior.

## 18.6 Metadata validation tests

Ensure `phalcom-type-meta` bundle validation remains correct for each local bundle independently.

Do not make validation dependent on VM state.

Cross-pool dependency compatibility belongs in core/runtime loading, not logical schema validation.

## 18.7 Deferred `corpus::booleans` incident

Re-run the checkpoint’s metadata-related deferred failure.

Classify result:

```text
failure disappears because problematic metadata is unreachable
    → valid P4 resolution

failure persists because required retained metadata is unrepresentable
    → separate schema/export correctness defect
```

Do not suppress a required metadata defect by stripping required records.

---

# 19. Performance acceptance

## Structural gate

The primary P4 performance gate is structural.

For a tiny RuntimeMinimal fixture:

```text
metadata work must scale with runtime roots
not total semantic snapshot cardinality
```

Required:

```text
0 canonical declaration records exported locally
0 canonical callable records exported locally
0 canonical field records exported locally
0 D×C callable-owner sweep
```

## Benchmark target

Retain the historical target:

```text
>90% reduction in ordinary tiny-fixture user metadata bundle size
```

as a benchmark target, not universal correctness law.

A library intentionally exposing a large public API is allowed to retain a larger bundle.

## Time target

Metadata export wall time should become proportional to reachable metadata rather than canonical Universe size.

Record:

```text
before/after MetadataExport duration
before/after whole compile duration
record-count deltas
encoded-size deltas
```

---

# 20. Detailed patch sequence

## Patch 1 — Compiler metadata request seam

Modify:

```text
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/compile_session.rs
```

Add explicit metadata profile input.

Keep RuntimePublic default.

Build/tests only.

## Patch 2 — Reachability types

Create:

```text
phalcom-semantic/src/metadata/reachability.rs
```

Add:

```text
MetadataRootSet
MetadataRetentionRequest
MetadataReachabilityClosure
```

No exporter cutover yet.

## Patch 3 — Root construction in core

Modify:

```text
phalcom-core/src/modules/compile.rs
```

Build RuntimeMinimal and RuntimePublic root sets from authoritative compiler products.

Add tests proving root cardinalities.

## Patch 4 — Callable/field owner indexes

Modify:

```text
phalcom-semantic/src/signature.rs
```

Add indexes and invariant tests.

Do not touch exporter yet if practical.

## Patch 5 — Reachability-driven exporter

Modify:

```text
phalcom-semantic/src/metadata/export.rs
```

Replace aggregate loops with closure iteration.

Make identity validation closure-driven.

Wire fields.

Add observability counters.

## Patch 6 — Local/canonical metadata partition

Modify:

```text
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/canonical_universe.rs
```

Teach local export that canonical stable owners are external authorities.

Assert canonical records are not emitted locally.

## Patch 7 — Runtime stable-identity indexes

Modify:

```text
phalcom-core/src/typing/registry.rs
phalcom-core/src/typing/loader.rs
```

Index declaration/callable/field stable identities on pool registration.

Add conflict detection.

## Patch 8 — Cross-pool reflection cutover

Modify:

```text
phalcom-core/src/typing/inspect.rs
phalcom-core/src/typing/reify.rs
phalcom-core/src/typing/identity.rs
phalcom-core/src/typing/environment.rs
```

Remove same-bundle declaration metadata assumptions.

Use registry stable-identity lookup.

## Patch 9 — Compatibility contract

Modify:

```text
phalcom-core/src/modules/artifact.rs
phalcom-core/src/modules/materialize.rs
```

or the current compiled-program metadata dependency owner.

Validate canonical dependency compatibility.

Do not persist numeric runtime pool IDs.

## Patch 10 — Certification + lifecycle docs

Update:

```text
docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/CHECKPOINT.md
```

Create:

```text
PERF003.C2.P4-walkthrough.md
PERF003.C2.P4-handoff.md
```

Record:

```text
baseline
patch heads
focused tests
broad tests
performance evidence
cardinality evidence
known residuals
P5 handoff
```

---

# 21. Verification ladder

Testing is evidence gathering, not ritual.

Run narrow gates after each patch; run broad gates only after architecture stabilizes.

## G0 — build

```sh
cargo check -p phalcom-type-meta
cargo check -p phalcom-semantic
cargo check -p phalcom-core --features perf-observability
```

## G1 — owner-index invariants

Focused semantic tests for callable/field owner indexing.

Expected:

```text
exact membership
remove correctness
module removal correctness
clone correctness
```

## G2 — reachability semantics

Run metadata reachability tests.

Expected:

```text
RuntimeMinimal contains only required closure
RuntimePublic contains public closure
unreachable entities excluded
canonical entities classified external
```

## G3 — exporter parity

For cases where old and new RuntimePublic outputs should be semantically equivalent, compare normalized metadata.

Ignore only ordering differences if ordering is not semantically specified.

Prefer deterministic ordering so raw equality is possible.

## G4 — cross-pool runtime

Run:

```text
cross_pool_metadata
reflection/type metadata tests
generic reflection tests
runtime type reification tests
```

Expected:

```text
canonical reflection succeeds
local reflection succeeds
cross-pool type identity preserved
```

## G5 — profile authority

Expected:

```text
RuntimeMinimal does not grant public discovery
RuntimePublic does
```

## G6 — cardinality

Tiny fixture:

```text
MetadataCallableOwnerComparisons no longer scales as D×C
canonical records emitted locally = 0
retained counts proportional to roots/local API
```

## G7 — broad compiler/runtime parity

Run the repository’s relevant core, corpus, reflection and semantic integration suites.

Attribute failures against a clean predecessor where necessary.

Do not automatically classify every broad failure as P4-owned.

## G8 — benchmark

Re-run T0 workloads.

Publish before/after table.

---

# 22. STOP / CONSULT triggers

## SC-P4-01 — Type graph requires true persisted cross-pool child references

If canonical duplication cannot be eliminated without allowing local `TypeNode` child edges to directly reference canonical pool nodes, stop before redesigning `phalcom-type-meta`.

That is a schema architecture decision.

## SC-P4-02 — RuntimePublic publicness has no authoritative semantic source

If public declaration/member visibility cannot be recovered from existing module/interface/surface products without reconstructing visibility from source, stop and resolve visibility ownership.

Do not infer from names.

## SC-P4-03 — Local/canonical stable identity collision is expected by design

If legitimate compilation routinely produces two different semantic records with the same `StableDeclarationRef`, stop.

The stable identity scheme or package identity model is insufficient.

## SC-P4-04 — Canonical metadata is not safely reusable across ordinary program VMs

If canonical metadata contains VM-local mutable references or program-specific runtime state, stop.

Metadata bundles must remain immutable semantic data.

## SC-P4-05 — RuntimeMinimal execution depends on broad declaration discovery

If executable semantics currently depend on scanning arbitrary metadata declarations, stop and identify the true runtime contract.

Do not retain the whole world merely to preserve an accidental scan.

## SC-P4-06 — Metadata retention changes source-language semantics

If reducing metadata changes method dispatch, static semantics, type checking, conformance selection or compilation output beyond reflection/metadata availability, stop.

P4 must not become a semantic-analysis package.

---

# 23. Forbidden approaches

Do not:

```text
1. switch all builds to RuntimeMinimal merely for speed;
2. use MetadataPoolId(0) as persisted canonical identity;
3. add a second metadata identity system parallel to Stable*Ref;
4. serialize the full TypeStore and prune after encoding;
5. scan all callables per declaration;
6. infer publicness from naming conventions;
7. silently shadow canonical records with local duplicates;
8. make phalcom-type-meta validation depend on VM registry state;
9. move compiler/runtime-owned policy into phalcom-type-meta;
10. weaken RuntimePublic reflection semantics to meet performance targets;
11. use test-only bypasses as the production implementation;
12. absorb P5 backend indexing work into P4.
```

---

# 24. Non-goals

P4 does not own:

```text
P5 semantic-lowering aggregate scan elimination
trait requirement-slot indexing
inherent-impl domain indexing
general backend module indexing
persistent .telpkg package format
serialized compiled Universe image
VM heap snapshotting
general package dependency artifact graph
new reflection capabilities
new type semantics
new conformance semantics
```

P4 should leave a clean metadata authority boundary that future package artifacts can reuse.

---

# 25. Required deliverables

At closure, produce:

```text
code implementation
focused tests
cross-pool reflection tests
profile matrix tests
cardinality tests
performance measurements
PERF003.C2.P4-walkthrough.md
PERF003.C2.P4-handoff.md
updated CHECKPOINT.md
```

Walkthrough must include:

```text
final architecture
changed files/symbols
before/after measurements
metadata cardinalities
bundle size comparison
cross-pool authority model
known residuals
P5 recommendation
```

---

# 26. P4 acceptance checklist

P4 is complete only if every item below is true.

## Architecture

- [ ] metadata profile is explicit compiler input;
- [ ] `MetadataProfile` materially controls retention;
- [ ] metadata roots are explicit;
- [ ] reachability closure exists and is deterministic;
- [ ] RuntimeMinimal does not perform whole-world metadata discovery;
- [ ] RuntimePublic preserves its documented reflection surface;
- [ ] public fields are retained under RuntimePublic;
- [ ] canonical metadata is treated as external authority;
- [ ] numeric pool IDs are not used as durable artifact identity.

## Complexity

- [ ] declaration → callable lookup is indexed;
- [ ] declaration → field lookup is indexed if required;
- [ ] `MetadataCallableOwnerComparisons` is proportional to retained relationships;
- [ ] no local export sweep over canonical declarations;
- [ ] no local export sweep over canonical callables;
- [ ] no local export sweep over canonical fields.

## Runtime

- [ ] canonical metadata pool resolves canonical records;
- [ ] local metadata pool resolves local records;
- [ ] stable declaration lookup is pool-independent;
- [ ] stable callable lookup is pool-independent;
- [ ] stable field lookup is pool-independent;
- [ ] same-pool declaration-record assumptions removed;
- [ ] cross-pool type identity preserved;
- [ ] duplicate stable identity conflicts are detected;
- [ ] canonical dependency mismatch is detected.

## Profiles

- [ ] RuntimeMinimal executes correctly;
- [ ] RuntimeMinimal does not expose arbitrary public reflection;
- [ ] RuntimePublic exposes public reflection correctly;
- [ ] ToolingDebug/Proof behavior remains compatible with existing policy.

## Evidence

- [ ] focused tests pass;
- [ ] cross-pool tests pass;
- [ ] metadata validation tests pass;
- [ ] broad compiler/runtime suites are classified;
- [ ] tiny-fixture bundle size materially reduced;
- [ ] metadata export time materially reduced;
- [ ] cardinality counters prove structural elimination.

---

# 27. Closure state

Successful completion should update the checkpoint to approximately:

```text
PERF003.C2.P1  CLOSED
PERF003.C2.P2  CLOSED
PERF003.C2.P3  CLOSED
PERF003.C2.P4  CLOSED
PERF003.C2.P5  NEXT / PROPOSED
```

P4 closure does **not** imply `PERF003.C2` overall is closed.

The final P4 reprofile determines whether the next dominant cost is:

```text
P5 backend/module aggregate indexing
P7 publication/lifecycle residuals
artifact persistence/cold-start work
or a newly exposed hotspot
```

---

# 28. Target end state

```text
                 ┌─────────────────────────────┐
                 │ Canonical Universe semantic │
                 │ image                       │
                 └──────────────┬──────────────┘
                                │
                                ▼
                 ┌─────────────────────────────┐
                 │ Canonical RuntimePublic     │
                 │ metadata bundle             │
                 └──────────────┬──────────────┘
                                │
                                ▼
                         Metadata Pool C
                                │
                                │ stable semantic identity
                                │
        ┌───────────────────────┴───────────────────────┐
        │                                               │
        ▼                                               ▼
┌────────────────────┐                        ┌────────────────────┐
│ local source/API   │                        │ executable roots   │
└─────────┬──────────┘                        └──────────┬─────────┘
          │                                              │
          └─────────────────────┬────────────────────────┘
                                ▼
                     MetadataRootSet(profile)
                                │
                                ▼
                    MetadataReachabilityClosure
                                │
                                ▼
                    Local metadata export only
                                │
                                ▼
                         Metadata Pool L
                                │
                                ▼
                     RuntimeTypingRegistry
               stable identity → pool-qualified record
                                │
                                ▼
                       reflection / reification
```

The central success condition is:

> A tiny user program must no longer cause metadata export to behave as if the entire canonical Universe belongs to that user program.
