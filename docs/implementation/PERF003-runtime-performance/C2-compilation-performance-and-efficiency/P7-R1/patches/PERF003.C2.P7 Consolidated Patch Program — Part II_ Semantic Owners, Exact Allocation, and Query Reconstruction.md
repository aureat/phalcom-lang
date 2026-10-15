
# 4. Patch Plan P7.PATCH.2 — Restore all remaining canonical semantic authority

## Objective

Repair every non-callable semantic owner currently defaulted, detached, fabricated, omitted, or reconstructed from the wrong authority.

This patch includes:

```text
accepted callable definitions
trait headers/surfaces
dispatch callable knowledge
witness plans
associated-type plans
derived conformance rules
trait dispatch
CheckedStaticBody/static applications
static definitions
static constants
static cases/signatures
module static bindings
compiler-required source-independent delegation/default-body facts
```

The third audit explicitly confirms that callable definitions, static owners, derived conformance, trait dispatch, and formal projection remain absent or fabricated in the loaded snapshot.

## 4.1 Persist accepted callable definitions as their own owner

A callable signature is not an accepted definition.

Add a dedicated record preserving:

```text
callable
origin:
    PrimaryDeclaration
    InherentImpl(stable impl)
    ConformanceWitness(stable impl)

source_member_index
accepted signature reference / exact signature identity
inherent_body_context?
compiler-required generated/delegation/default-body attributes
```

Import validation must cross-check:

```text
definition → callable signature
definition → referenced impl/conformance
definition → body analysis, if executable
definition → body context
```

Never create:

```rust
EffectiveCallableDefinition {
    origin: PrimaryDeclaration,
    source_member_index: 0,
    inherent_body_context: None,
    ...
}
```

unless those values are actually encoded source authority.

Rebuild `callable_definitions_by_module` from accepted definitions, not from signatures.

## 4.2 Eliminate residual source-AST dependency from canonical lowering

The audits found remaining canonical lowering paths that consult source data for:

```text
generated inherent delegation
trait-default detached method discovery
```

Those decisions must become semantic owner facts.

Add explicit accepted-definition or callable-analysis attributes sufficient for lowering to answer:

```text
Is this generated delegation?
Which field/target is delegated?
Is this a trait-default body?
Which accepted trait member/source-member identity produced it?
What detached callable is selected?
```

User-module lowering may still use user source.

Canonical imported Universe lowering may not silently reopen canonical syntax.

## 4.3 Restore trait authority without defaults

Trait artifact representation must preserve:

```text
trait identity
supertraits

associated-type requirements:
    exact domain/kind
    source/provenance required by compiler
    constraint/default information

required members
default members

per-member:
    callable/signature
    visibility
    source/default provenance
    semantic role
```

Do not substitute:

```text
KindId(0)
Public
default SourceRange
default_source = None
```

unless those are the actual source-built values.

Partial member lookup is an error. Missing a required referenced signature must fail import.

## 4.4 Restore dispatch `TypeKnowledge`

The dispatch surface must receive exact relocated callable knowledge.

Delete any artifact import path equivalent to:

```rust
TypeKnowledge::assumed(TypeId(0), EvidenceOrigin::Flow)
```

for restored compiler authority.

A strong test must load a generic/class/instance dispatch surface and compare source-built and imported:

```text
type
evidence category
selected callable
dispatch bucket membership
```

## 4.5 Make witness plans fully portable

Persist complete witness authority:

```text
requirement views
explicit selections
invalid explicit-member state
conditional selection/applicability
associated bindings
associated-type failures
completeness state
failure payloads
diagnostic/invalidity state
provenance
```

Do not map rich source states into generic placeholders such as:

```text
Incomplete -> failures = []
Unknown -> NoTypeEvidence
Blocked -> RecursiveFixpoint
Dynamic -> "dynamic"
InternalFailure -> "restored from artifact"
```

if the source owner contained more precise facts.

The loaded canonical workspace intentionally avoids rebuilding Universe witness plans, so witness completeness is a **load-time requirement**, not work that can be deferred.

## 4.6 Persist derived conformance authority

Persist `DerivedConformanceRule` semantic authority:

```text
source impl identity
trait template
target template
generic signature
guards/applicability
provenance
product fingerprint inputs
```

The **rule** is primary semantic authority.

The trait-dispatch bucket/index may be reconstructed if:

```text
its contents are a deterministic function of persisted authority
reconstruction does not re-run semantic resolution
reconstruction is allocation-accounted
source/import bucket membership parity is tested
```

If those conditions cannot be proven, persist the dispatch index itself.

## 4.7 Correct `StaticApplication` ownership

Keep the real owner:

```text
StaticBodyId + body-local ordinal
```

Do not copy checked static applications into `CallableAnalysis`.

Portable representation needs exact checked rows sufficient to resolve:

```text
StaticApplicationId
source expression/range if compiler-required
target definition
selected checked signature
argument-to-parameter correspondence
checked argument kind/value
```

Then a denotation references:

```text
(static body owner, application ordinal)
```

The loader resolves that reference into the imported `CheckedStaticBody`.

Two different bodies may both contain ordinal `0`. Tests must exercise this.

## 4.8 Complete source-independent static-body lowering

Current operational conversion is reported to have problems around:

```text
source-form checked bodies
nested static applications
KindedExpression arguments
first-application shortcut for TypeForm
```

Define one canonical source-independent operational lowering stage.

Preferred producer pipeline:

```text
checked source static body
        ↓
ValidatedOperationalTemplate
        ↓
artifact DTO
```

Every accepted source representation:

```text
Expression
Block
TypeForm
already Operational
```

must either:

```text
successfully lower to the portable operational form
```

or:

```text
fail publication with a precise unsupported-profile error
```

before the broad static coverage guard is relaxed.

## 4.9 Install operational templates into real static definitions

Do not return free-floating:

```text
ValidatedSemanticParts.operational_templates
```

and then discard them.

Rebuild the actual owner:

```text
StaticDefinitionProduct
    definition identity
    signature
    result/static domain
    parameter authority
    checked body owner
    operational root
    checked static applications
    dependency identity
    compiler/evaluator-required product metadata
```

The canonical image must contain a real `StaticDefinitionTable`.

## 4.10 Restore static constants

Persist the reduced canonical static-constant authority needed by compiler consumers:

```text
constant identity
checked initializer/result authority
outcome
residuals when required
canonical value
last-known-good value if semantically relevant
dependency identity/fingerprint
diagnostics needed by compiler semantics
```

Do not rerun Universe constant initializers from source during artifact load.

## 4.11 Restore static case catalogs/signatures

Populate their real snapshot tables and validate all referenced static/type owners.

They must not remain “DTO exists somewhere but snapshot owner is empty.”

## 4.12 Persist final module-static binding index

Persist the final result of module static-binding analysis rather than enough syntax facts to rerun the builder.

Conceptually preserve:

```text
symbol
source top-level/static body owner
body-local binding ordinal where relevant
phase
final binding state
declaration attachment
initializer attachment
import/name alias authority required for scope seeding
```

Do not reconstruct canonical bindings by source name/range matching.

## 4.13 Rebuild real module-bounded indexes

Delete or demote detached debug-key structures such as:

```text
BTreeMap<String, ...>
format!("{:?}", module)
```

as compiler authority.

P6 module-bounded readers must exist inside the actual owning structures, keyed by typed/stable module identity.

Required indexed families include the ones actually iterated by compiler consumers, including:

```text
declarations
accepted callable definitions
callable analyses where required
static definitions
static constants
module static bindings
data/enums/traits as required
impl/conformance owners
```

## PATCH.2 completion gate

The patch is complete when source-built versus imported parity exists for:

```text
primary callable definition
inherent callable definition
conformance-witness callable definition
trait with supertrait
non-public/default trait member
nontrivial associated-type domain
conditional/incomplete witness
nonempty associated plan and fingerprint
real dispatch type knowledge
derived conformance
trait dispatch
simple static definition
nested checked static application
static constant
module static binding
```

Only then may the corresponding clauses in `validate_semantic_world_export_coverage` be removed.

---

# 5. Patch Plan P7.PATCH.3 — Close the exact cumulative allocation transaction

## Objective

Make the P7 loader's stated allocation policy true from file acquisition through final loaded image publication.

The binary/resource audit found unaccounted native module allocations **before** the existing semantic guard, and additional latent gaps in `TypeStore`, semantic owner reconstruction, and query reconstruction.

## 5.1 Preserve the existing memory contract

Do not redefine the contract merely to make current code pass.

The intended invariant is:

```text
one cumulative artifact-load ledger
tracking allocator-requested owned allocation bytes

before every requested allocation:
    debit transaction

if debit cannot be made:
    do not issue allocation request

stage failure:
    parent transaction unchanged

successful child stage:
    commit debit to parent
```

This is not an RSS cap.

Track separately:

```text
requested bytes
retained bytes
peak RSS
```

## 5.2 Standard transaction pattern

Every stage should follow:

```rust
let mut child = transaction.child();

let value = build_stage(..., &mut child)?;

child.commit()?;
Ok(value)
```

Never debit the parent's successful-load budget directly throughout a fallible multi-row operation unless all earlier debits are intentionally committed even when the operation fails.

## 5.3 Fix native module-world materialization first

This path executes before the semantic guards and is therefore an immediate correctness defect.

Inventory:

```text
BTreeMap nodes
BTreeSet nodes
Vec allocations/reallocations
owned Strings
symbol/name clones
topology products
graph nodes
export/import products
Arc allocations
scratch
```

Move immutable loaded authority toward:

```text
sorted contiguous entries
budgeted arrays
budgeted Arc slices
typed binary-search indexes
```

Do not estimate `BTreeMap` node cost with a multiplier.

If a native tree must remain, the implementation needs an allocator-aware/fallible construction seam whose requests can be measured exactly.

## 5.4 Make `TypeStore::import_type_graph` transaction-aware

Its signature must receive the load transaction, directly or through a dedicated transactional builder.

Account:

```text
fresh store internals
type nodes
type-parameter relocation
record-row relocation
uniqueness indexes
scope validation
temporary context vectors
visited sets
boxed payloads
owned strings
interning structures
```

Remove local “estimated remaining bytes” counters as substitutes for the global ledger.

Local structural limits may remain as additional DoS protection, but they are not allocation accounting.

## 5.5 Make semantic validator scratch budget-aware

The semantic-world validator itself is currently blocked by `UnaccountedBoundary`.

Inventory every validator helper for:

```text
temporary vectors
sorting
duplicate indexes
visited sets
identity maps
diagnostic strings
cross-owner lookup tables
```

Where possible, validate against sorted portable arrays without building native maps.

The decoder guard:

```text
require_semantic_validation_allocation_accounting()
```

may be removed only when independent counting-allocator tests prove this stage.

## 5.6 Use frozen imported owner representations

Apply the `FlowGraph` precedent to the map-heavy owners introduced by PATCH.1/PATCH.2.

Examples:

```text
ExpressionAnalysisIndex
BindingAnalysisIndex
resolution indexes
ExplanationArena
CallableAnalysisTable
per-module callable index
static owner tables
query root table
dependency fingerprint table
```

Pattern:

```rust
enum Storage<K, V> {
    Mutable(...),
    Frozen(BudgetedSortedMap<K, V>),
}
```

Exact API is flexible.

Compiler consumers should observe one read interface.

## 5.7 Account query construction

Charge:

```text
query-owner decoding
query root tuples
product creation
cloned/owned product payloads
dependency vectors
Arc<[DependencyEdge]>
root index
dependency-fingerprint index
stable strings/paths if still owned
```

No ordinary `BTreeMap::insert` should hide outside the ledger in canonical loaded storage.

## 5.8 Preserve typed allocation errors

Never convert:

```rust
ArtifactError::Allocation(error)
```

into:

```rust
LimitExceeded(error.to_string())
IncompletePublication(error.to_string())
```

inside the constrained load transaction.

Use typed propagation:

```rust
match error {
    ArtifactError::Allocation(a) =>
        TeloscArtifactError::Allocation(a),

    other =>
        // formatting only outside the restricted boundary
}
```

Error construction on exhausted budget must request zero additional heap allocations.

## 5.9 Exact counting-allocator test methodology

For each stage:

```text
1. run under isolated counting allocator
2. obtain observed allocator-requested bytes N
3. run artifact transaction with budget N
4. require success
5. run with N - 1
6. require typed allocation rejection before unmatched request
7. verify parent ledger rollback
8. retry using fresh transaction
9. require success
```

Tests must target actual native materialization, not merely DTO decode.

Required stages:

```text
container/header/directory
module-world materialization
TypeStore relocation
full callable owner
static owners
trait/witness owners
query reconstruction
metadata native ownership
final publication Arc/products
entire disk load
```

## PATCH.3 guard-removal gate

Only after complete whole-load equality is demonstrated may the two global barriers be removed:

```text
require_semantic_validation_allocation_accounting
require_semantic_import_allocation_accounting
```

Removal must be the consequence of completed accounting, not an implementation step used to discover missing accounting.

---

# 6. Patch Plan P7.PATCH.4 — Make `QUERY_IMAGE` a closed incremental-authority layer

## Objective

Repair query export/import so the loaded canonical image has the same reusable root and aggregate dependency identity as the source-built image.

## 6.1 Define one owner-disposition function

Avoid partially overlapping switch statements across exporter/importer/database.

Introduce one conceptual authority:

```text
persisted_query_disposition(key)
    -> ReusableRoot
     | EvidenceOnly
     | OmittedSourceDependent
     | Unsupported
```

And one fingerprint adapter:

```text
fingerprint_for_persisted_owner(
    snapshot,
    modules,
    key
)
```

The same semantic function should underpin:

```text
query export
query import validation
canonical-image reconstruction
SemanticDb canonical dependency lookup
```

## 6.2 Reconstruct `dependency_evidence`

The audit found that the exporter writes dependency evidence but the loader ignores it.

For every evidence row:

```text
decode owner
verify disposition != OmittedSourceDependent
resolve canonical owner
recompute canonical product fingerprint
compare/validate wire evidence
reject duplicate/conflicting key
insert into canonical dependency_fingerprints
```

Evidence-only keys do **not** automatically become reusable root products.

They do become canonical dependency leaves usable by local incremental queries.

## 6.3 Fix supported query-kind drift

If:

```text
exporter says supported
wire enum says supported
stable-key decoder says supported
```

then `product_and_fingerprints` must have an owner adapter.

The audits specifically identify:

```text
ModuleStaticBindingCatalog
ModuleStaticBindings
```

as this class of mismatch.

Once PATCH.2 restores the semantic owners, add exact adapters.

Until then, do not misleadingly classify them as fully reusable.

## 6.4 Replace JSON stable-owner decoding if practical

Owned `serde_json::from_str` introduces:

```text
transient allocations
string/path allocations
duplicate validation logic
parse overhead
allocation-accounting complexity
```

Preferred P7 direction is a structured stable owner codec shared with the semantic/module artifact format.

Conceptually:

```text
StableQueryOwnerV2
    Module(...)
    Callable(...)
    Declaration(...)
    Impl(...)
    StaticDefinition(...)
    ...
```

If wire compatibility constraints require retaining V1 JSON decoding, decode V1 into the same validated structured representation and fully account its allocations.

## 6.5 Apply query-specific limits before reservation

Use:

```text
max_query_roots
max_query_dependencies_per_root
evidence-per-root limit
total-edge limit
```

at DTO decode.

Do not reserve to a generic one-million-entry semantic-family limit only to reject later under a 100k/10k query limit.

## 6.6 Replace quadratic reverse pruning

Current repeated full rescanning can degrade to Θ(V²) on a chain whose final leaf is omitted.

Replace with reverse adjacency.

Conceptually:

```text
candidate roots
     ↓
build reverse edges
     ↓
queue roots with unsupported/missing dependency
     ↓
pop root
mark removed
enqueue affected predecessors
```

Each root/edge should be visited O(1) times:

```text
O(V + E)
```

Canonical final emission ordering remains separately sorted.

## 6.7 Build frozen query indexes

Loaded canonical query storage should use immutable sorted storage for:

```text
CanonicalQueryEntry table
dependency fingerprint table
dependency edge slices
```

Workspace-local mutable query DB state may continue using native mutable maps.

## PATCH.4 completion gate

Required discriminators:

```text
reusable root round trip
evidence-only owner round trip
duplicate evidence rejection
wrong-owner evidence rejection
changed fingerprint rejection
source-dependent evidence rejection
ModuleStaticBindingCatalog reconstruction
ModuleStaticBindings reconstruction
malformed owner key rejection
exact N/N−1 query reconstruction allocation
10k/100k pruning chain scales linearly
canonical query fingerprints match source-built image
```