
# PERF003.C2.P7 — Consolidated Patch Program

## 0. Purpose and architectural ruling

This patch program synthesizes the six independent P7 audit slices into a single implementation sequence. It deliberately does **not** treat every reported finding as an independent bug. Most findings reduce to four architectural failures:

1. **The persisted semantic image is not yet owner-authoritative.** Several compiler-consumed semantic products are absent, represented only partially, reconstructed from the wrong owner, or replaced by fabricated defaults.
2. **The artifact load transaction is not allocation-closed.** Exact-budget machinery exists, but important native materialization paths still allocate outside the one cumulative transaction.
3. **The portable query/container layer does not yet have canonical identity and reconstruction closure.** Some bytes are accepted outside canonical layout, some query evidence is emitted but ignored, and some fingerprinting is incomplete.
4. **The current acceptance suite describes the intended product more strongly than it proves it.** Many tests never reach their named execution frontier because artifact construction/import is deliberately stopped earlier.

The most important positive finding must also be preserved: the **P5/P6 immutable-parent overlay architecture is sound**. Once a complete loaded `CanonicalUniverseFormalImage` exists, workspace sessions already reuse canonical parent tables rather than flattening or copying the Universe into every workspace.

Therefore this program does **not** propose a redesign of canonical-image consumption. The repair target is:

```text
source-built canonical Universe
             │
             │ exact compiler-semantic equivalence
             ▼
persisted P7 canonical compiler image
             │
             │ immutable parent
             ▼
small user/project/package overlay
```

P7 is still a **compiler image**, not P8's serialized executable Universe and not P9's automatic freshness/incremental-cache manager. Artifact freshness against unavailable source cannot be solved merely by inspecting a nonzero source digest; the container audit explicitly records this as a later release boundary.

---

# 1. Global implementation rules

These rules apply to every patch below.

### 1.1 No fabricated compiler authority

If a source-built canonical owner contains compiler-consumed semantic information, artifact load may do only one of two things:

```text
A. transport it losslessly and relocate it; or

B. reconstruct it deterministically from explicitly transported
   canonical primary authority, with a parity proof.
```

It may **not**:

```text
guess
default
look up by source name
look up by source range alone
substitute TypeId(0)
manufacture an empty proof
infer definition origin from a signature
rerun canonical checking from source
```

This rule directly addresses the reported fabricated callable definitions, assumed dispatch knowledge, synthetic trait facts, lossy witness states, empty static families, and empty callable-analysis table.

### 1.2 Guard removal is per contract, never global

Treat every current fail-closed clause as a separate feature gate.

A guard may be removed only when:

```text
producer owner
    ↓
wire representation
    ↓
structural validation
    ↓
type/static relocation
    ↓
native materialization
    ↓
real owning snapshot/table
    ↓
compiler consumer
```

has a passing parity discriminator.

Removing all guards merely to expose the next failure is prohibited.

### 1.3 Root closure precedes projection

Any new portable field containing or transitively owning a:

```text
TypeId
KindId
RecordRowId
TypeParameterId
static-domain handle
generic-conformance product
signature/type knowledge
```

must register its roots **before** `TypeExportContext::close_graph`.

Projection must never discover an unregistered type root.

### 1.4 Imported canonical structures are immutable

Imported owners should use immutable/frozen storage wherever mutable native `HashMap`/`BTreeMap`/`BTreeSet` allocation prevents exact load-budget accounting.

Source construction may remain optimized for mutation.

Artifact reconstruction should be optimized for:

```text
dense construction
fallibility
exact allocation accounting
stable iteration
binary-search or indexed lookup
structural sharing
```

### 1.5 Completeness means closure, not population

A table being non-empty is not proof that it is complete.

Separate:

```text
ArtifactStructuralCompleteness
CanonicalUniverseAcceptanceProfile
```

The first must be general and cardinality-independent.

The second may assert Universe-specific expected contents.

---

# 2. Patch Plan P7.PATCH.0 — Make the artifact contract executable

## Objective

Before materially enlarging the wire schema, replace the current partly aspirational persistence manifest with an **executable owner/field contract** and repair several narrow correctness defects that would otherwise be persisted into the final artifact.

This is the architectural foundation patch.

## Why this patch comes first

The audits found that the manifest labels substantial `CallableAnalysis` fields as `Wire`, while `CallableAnalysisRecordV1` contains only a subset and the importer rejects nonempty analyses.

Without a mechanical definition of “implemented,” subsequent DTO work can again produce a schema that appears complete while omitting a consumer or relocation step.

## 2.1 Replace binary `Wire` bookkeeping with a proof matrix

Conceptually introduce an implementation-state model separate from architectural disposition.

Illustrative shape:

```rust
enum ArtifactDisposition {
    RequiredWire,
    DerivedAfterRelocation,
    OmittedByPolicy,
}

enum ArtifactImplementationState {
    Missing,
    Partial,
    Implemented,
}

struct ArtifactFieldContract {
    owner: SemanticOwnerKind,
    field: &'static str,

    disposition: ArtifactDisposition,
    state: ArtifactImplementationState,

    exporter: Option<&'static str>,
    wire_field: Option<&'static str>,
    validator: Option<&'static str>,
    importer: Option<&'static str>,
    installed_owner: Option<&'static str>,
    consumer: Option<&'static str>,
    parity_test: Option<&'static str>,
}
```

The exact Rust representation is flexible. The invariant is not:

```text
manifest says Wire
```

but:

```text
RequiredWire
AND exporter exists
AND wire member exists
AND root closure exists
AND validation exists
AND relocation exists
AND materializer exists
AND owning table receives the product
AND at least one real consumer parity test exists
```

## 2.2 Make contract drift testable

Add tests that fail if a required field is classified `Implemented` while any required stage is absent.

For `CallableAnalysis`, the test should enumerate the live native field census and compare it against the artifact contract.

Do the same at `SemanticSnapshot` owner-family level.

Do **not** try to derive this entirely through Rust reflection. Explicit exhaustive match/test tables are acceptable and often clearer for a compiler artifact ABI.

## 2.3 Fix incremental fingerprint correctness before persisting more owners

The second Slice-2 audit identifies existing semantic fingerprint defects: `binding_write`, `family_applications`, and important callable-reference/associated-resolution dimensions are missing from `callable_body_product_fingerprint`.

Repair these before artifact parity testing.

Required changes:

```text
ExpressionAnalysis
    include binding_write

CallableAnalysis.family_applications
    hash every executable selection dimension

CallableReferenceResolution
    BoundFamily:
        hash family_type
        hash selector/spec
        hash members

    Associated:
        recursively hash complete AssociatedResolution

AssociatedResolution
    exhaustively hash every resolution kind
    not only DataExpansionField
```

Create one authoritative helper:

```rust
fn hash_associated_resolution<H: Hasher>(
    value: &AssociatedResolution,
    h: &mut H,
) {
    match &value.kind {
        // exhaustive semantic hashing
    }
}
```

Reuse it anywhere nested associated resolutions contribute to callable identity.

### Required tests

Each test changes exactly one compiler-relevant semantic choice while holding all other fields stable and asserts:

```text
old fingerprint != new fingerprint
```

At minimum:

```text
binding_write A → B
family application target/spec
associated exact value/callable/static invoke
bound-family family_type
bound-family selector
nested associated callable-reference resolution
```

## 2.4 Repair associated-type-plan fingerprint construction

The audit identifies a specific stale-fingerprint defect: the importer creates an empty associated plan, computes its fingerprint, and later mutates its binding map without recomputing the fingerprint.

Remove mutation-after-fingerprint construction.

Preferred shape:

```rust
let bindings = import_associated_bindings(...)?;

let associated_plan =
    ConformanceAssociatedTypePlan::from_validated_bindings(
        impl_id,
        bindings,
    )?;
```

The constructor must compute the fingerprint from final contents.

Avoid:

```rust
let mut plan = ConformanceAssociatedTypePlan::empty(id);
plan.bindings = imported_bindings; // invalid architecture
```

## 2.5 Tighten structural limits immediately

Before V2 callable rows become dense, add explicit per-owner bounds.

The audit specifically identifies unbounded `expression_count`.

Add limits for:

```text
expressions / callable
bindings / callable
explanation nodes / callable
diagnostics / callable
diagnostic causes / callable
semantic dependencies / callable
match products / callable
pattern products / callable
generic conformance bindings / call
candidate lists / expression
flow nodes / callable
flow edges / callable
```

The decoder must reject counts **before reservation**.

## 2.6 Close discriminants and duplicate ownership

Any raw numeric artifact tag must use checked conversion:

```rust
let kind = WitnessStatusV1::try_from(raw)
    .map_err(|_| ArtifactError::InvalidEnumTag { ... })?;
```

Never:

```rust
match raw {
    0 => A,
    1 => B,
    _ => SomeValidFallback,
}
```

Validation must reject:

```text
duplicate trait owner
duplicate witness owner
duplicate callable owner
duplicate local ordinal
duplicate associated binding
wrong-owner local reference
foreign impl reference
unresolved stable callable
```

before insertion into a native owner table.

## 2.7 Split completeness

Replace the semantic role currently played by broad table non-emptiness with:

```rust
fn validate_artifact_structural_completeness(
    snapshot: &SemanticSnapshot,
    contract: &ArtifactContract,
) -> Result<(), ArtifactError>;

fn validate_canonical_universe_profile(
    snapshot: &SemanticSnapshot,
) -> Result<(), ArtifactError>;
```

The first proves owner closure.

The second is allowed to say, for example, that the canonical Universe is expected to contain callable analyses or static definitions.

A future standalone package with no traits must not be deemed structurally incomplete because the Universe happens to contain traits.

## PATCH.0 completion gate

Do not start removing semantic-family coverage clauses yet.

PATCH.0 is complete when:

```text
field/owner contract is executable
fingerprint discriminators pass
associated-plan fingerprint is immutable/correct
dense-owner limits exist
raw discriminants are closed
duplicate/wrong-owner validation is explicit
structural completeness is conceptually separated from Universe profile
```

---

# 3. Patch Plan P7.PATCH.1 — Complete the canonical `CallableAnalysis` owner graph

## Objective

Turn callable persistence from a sparse execution summary into a relocatable, immutable representation of the compiler authority actually stored in `CallableAnalysis`.

This is the largest single semantic patch.

The audits consistently identify callable analysis as the dominant semantic blocker. The current wire has only a partial skeleton and no full importer.

## 3.1 Use an owner-centric schema

Do not extend the existing count-plus-side-table model indefinitely.

Introduce a coherent next record conceptually resembling:

```text
CallableAnalysisRecordV2
    callable
    body_source_owner
    body_range
    status

    expressions[]
    bindings[]

    flow_graph
    entry_flow
    normal_exits[]
    throw_exits[]
    unreachable
    return_validation

    for_iteration_lanes[]
    positional_expansions[]
    invalid_positional_expansions[]
    sequence_literals[]

    associated_resolutions[]
    callable_reference_resolutions[]
    family_applications[]
    match_resolutions[]
    pattern_site_resolutions[]

    diagnostic_causes[]
    diagnostics[]
    explanations[]
    return_explanation?

    dependencies[]
    semantic_dependencies[]

    constructor_super_initializer?
```

`dependency_fingerprint` should remain derived after relocation.

Internal incidents remain an omission-by-policy only when canonical publication verifies they are empty.

## 3.2 Establish one local-ordinal authority

Use one canonical ordinal mapping pass per callable.

Portable handles must be of the logical form:

```text
(stable callable owner, local ordinal)
```

for:

```text
Expression
Binding
Explanation
DiagnosticCause
FlowNode
FlowEdge
```

Never serialize native arena/store identifiers.

The exporter should generate one `CallableAnalysisOrdinalMap` and reuse it throughout the callable projection.

The importer must build the inverse mapping transactionally.

## 3.3 Persist real expression rows

The audit shows that `expression_count` plus sparse denotations is insufficient.

Introduce an expression row containing all compiler-consumed semantic facts, conceptually:

```text
CallableExpressionV2
    ordinal
    range
    knowledge

    callable?
    call_specialization?

    conditional_dispatch?
    conditional_capability?

    trait_dispatch?
    trait_dispatch_candidates[]

    contextual_realization?
    denotation?

    status
    causal_invalidity
    explanation?

    binding_read_safety?
    binding_read?
    binding_write?
```

`CallResolutionId` or any other source/query handle lacking a canonical artifact owner should remain fail-closed until its true authority is defined.

Do not fill it with a synthetic ordinal simply to make the struct constructible.

## 3.4 Persist real binding rows

Conceptually:

```text
CallableBindingV2
    ordinal
    name
    parameter?
    range

    contract
    current_knowledge
    denotation?

    consistency
    causal_invalidity

    mutable
    initialization
    version

    explanation?
    storage_origin
    formal_attachment
```

Do not maintain three contradictory authorities for:

```text
binding row
binding origin side table
formal source attachment side table
```

Either fold origin/attachment into the binding record or mechanically prove 1:1 correspondence during validation.

## 3.5 Make formal attachment semantics explicit

Do not infer attachment meaning from “not found in `source_index`.”

Imported P7 images deliberately omit the broad source index, so absence there cannot distinguish an implicit receiver from a missing required source declaration.

Persist an explicit classification equivalent to:

```text
SourceDeclaredFormal {
    canonical_site_or_attachment,
}

GeneratedOrImplicit {
    reason,
}

ModuleValue {
    symbol,
    import_provenance?,
}
```

Exact names are flexible.

Semantic meaning is not.

For:

```text
BodyLocal + explicit source parameter
```

an attachment is mandatory.

For:

```text
implicit receiver/generated binding
```

absence must be explicitly authorized by owner data.

For:

```text
ModuleValue
```

the canonical module symbol is the storage authority.

## 3.6 Preserve synthetic `<main>` ownership correctly

A synthetic top-level body can have:

```text
body source owner = module
```

while containing body-local bindings.

Do not:

```text
invent a normal callable signature for <main>
identify it by textual selector
rebase source sites using callable count/order
collapse two top-level lambdas with same parameter spelling
```

Tests must specifically include two local bindings with identical textual names and overlapping numeric local IDs in different owner contexts.

## 3.7 Finish generic call specialization

Current transport preserves static arguments but rejects nonempty conformance evidence.

Persist every semantic conformance-source variant actually present in `CallSpecialization`, including the audit-identified categories:

```text
Exact
Forwarded
Derived
CompilerDerived
```

Preserve:

```text
stable owners
source/provenance relation
relevant fingerprints
antecedent evidence
type/static roots
```

`DynamicType` remains a real explicit category. It must not be collapsed into “missing binding” or a fake type.

## 3.8 Complete resolution families

Persist and relocate:

```text
for_iteration_lanes
positional_expansions
known_invalid_positional_expansions
sequence_literals
associated_resolutions
callable_reference_resolutions
family_applications
match_resolutions
pattern_site_resolutions
conditional dispatch
conditional capability
trait dispatch
trait dispatch candidates
contextual realizations
```

These are compiler products, not hints for re-solving semantics.

## 3.9 Preserve flow and return authority

The existing frozen flow graph is good groundwork.

Attach it to the reconstructed callable owner and additionally preserve:

```text
entry flow
normal return flow facts
throw flow facts
unreachable facts
return-contract validation
```

Do not reduce body exits to one `exits_unreachable` boolean.

## 3.10 Preserve explanation/diagnostic closure

Represent the explanation graph with owner-local ordinals.

Validation must establish:

```text
all explanation references exist
all diagnostic causes exist
return_explanation exists when present
DAG/cycle invariants match native model
sharing is preserved
no wrong-owner references
```

Invalid semantic states must remain invalid.

Do not reconstruct “Complete” by deleting the evidence explaining why source analysis was incomplete.

## 3.11 Dependencies and post-relocation fingerprint

Persist:

```text
dependencies
semantic_dependencies
```

After relocation and full owner construction:

```rust
let fingerprint =
    callable_body_product_fingerprint(&restored_analysis);
```

Compare it against source-built semantic identity in parity tests.

Do not adopt an arbitrary wire fingerprint as authority if the product can be recomputed from the restored owner.

## 3.12 Prepare frozen imported storage

Before implementing the final importer, remove representation leakage such as:

```rust
impl Deref<Target = HashMap<...>> for CallableAnalysisTable
```

from consumer contracts.

Migrate consumers to explicit APIs:

```text
get
contains
iter
keys
values
len
is_empty
iter_module
```

Then support:

```text
SourceMutable(...)
ImportedFrozen(...)
```

internally without requiring an unbudgeted shadow `HashMap`.

Apply the same model to:

```text
expressions
bindings
resolution indexes
explanations
outer callable table
per-module callable index
```

## 3.13 Transactional owner construction

The final importer for one callable should look conceptually like:

```text
child transaction
    validate counts
    validate local ordinal closure
    relocate type/static/stable owners
    allocate frozen backing
    materialize bindings
    materialize expressions
    materialize flow and return facts
    materialize resolution families
    materialize explanations/diagnostics
    materialize dependencies
    rebuild formal projection contribution
    construct CallableAnalysis
    recompute callable product fingerprint
    validate consumer invariants
commit child
```

Any error must leave the parent transaction unchanged.

The partial call-specialization importer reported as charging directly against its caller should also be converted to child-transaction semantics.

## PATCH.1 guard-removal gate

Only after the following pass may the callable-specific export/import guards be removed:

```text
full field parity modulo relocated IDs
two owners with overlapping local ordinals
expression read/write parity
binding parity
generic-conformance parity
match/pattern parity
conditional + trait dispatch parity
contextual-realization parity
explanation DAG parity
diagnostic parity
formal attachment parity with broad source_index absent
post-relocation fingerprint parity
source-built vs loaded lowering parity
exact-fit/N−1 callable allocation tests
```

At that point remove only:

```text
callable-analysis branch of validate_semantic_world_export_coverage
ensure_complete_callable_analysis_payload
```

The global allocation `UnaccountedBoundary` must remain until PATCH.3 is complete.