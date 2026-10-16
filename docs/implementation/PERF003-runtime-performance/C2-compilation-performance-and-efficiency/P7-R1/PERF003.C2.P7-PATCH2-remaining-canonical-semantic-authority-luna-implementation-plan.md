---
id: PERF003.C2.P7.PATCH2
category: PERF
program: PERF003
checkpoint: PERF003.C2
parent_plan: PERF003.C2.P7
kind: corrective-implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
depends_on:
  - PERF003.C2.P7.PATCH0
  - PERF003.C2.P7.PATCH1
follows: PERF003.C2.P7.PATCH1
supersedes: null
target_branch: perf/universe-characterization
prepared_against_revision: 21a54705c33dae95ee9c77d86789b4911635e6b0
audit_baseline_revision: 2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a
executor_profile: GPT-5.6 Luna High / Extra High
---

# PERF003.C2.P7.PATCH2 — Restore the Remaining Canonical Semantic Authority

> **Execution role:** Luna-oriented patch-grade implementation plan.  
> **Parent acceptance objective:** `PERF003.C2.P7` — persisted semantic compiler image (`universe.telosc`).  
> **Patch objective:** replace every remaining fabricated, erased, detached, or source-dependent primary semantic owner needed by a loaded canonical compiler image with exact portable authority or an explicitly proved deterministic derivation.  
> **Lifecycle boundary:** this patch completes semantic-owner fidelity. It does **not** close the global allocation transaction, query-image reconstruction, container canonicality, loaded-image promotion, or full P7 package/performance acceptance.

---

# 0. Executor contract

This plan is intentionally implementation-directive. Luna may adapt mechanical details to the live tree, but it must not improvise the semantic authority model.

The implementer must:

- inspect the live revision before editing;
- preserve all accepted P5/P6/P7 architecture;
- treat source-built semantic products as authority;
- transport primary authority exactly;
- derive only products explicitly proven deterministic from transported primary authority;
- keep all unrelated fail-closed guards in place;
- avoid source syntax as a fallback for canonical imported modules;
- thread the shared allocation transaction through new import APIs where mechanically feasible, while leaving **global exact allocation certification to PATCH3**;
- update `PERF003.C2/CHECKPOINT.md` only at meaningful lifecycle events;
- produce a PATCH2 walkthrough and handoff;
- stop and consult on ownership, versioning, unsupported static-program, or conformance-proof ambiguity.

A green test obtained by defaulting missing facts is a plan violation.

---

# 1. Goal

After PATCH2, the semantic world must no longer contain these P7 defects:

```text
CallableSignature
    -> fabricated EffectiveCallableDefinition {
         PrimaryDeclaration,
         source_member_index = 0,
         inherent_body_context = None
       }

TraitSurface wire
    -> fabricated associated domain KindId(0)
    -> fabricated Public visibility
    -> fabricated default spans/source
    -> dropped exact member semantic signatures

DeclarationSurface wire
    -> fabricated callable TypeKnowledge(TypeId(0))

WitnessPlan wire
    -> collapsed completeness u8
    -> empty requirement_views
    -> missing conditional selections/applicability
    -> empty associated failures/diagnostics
    -> stale associated-plan fingerprint

derived_conformance_rules
    -> empty

trait_dispatch_index
    -> empty

checked static operational templates
    -> detached vector
    -> discarded before canonical snapshot publication

static_case_catalogs
static_case_signatures
static_definitions
static_constants
module_static_bindings
    -> empty

core lowering
    -> snapshot.sources lookup for generated inherent delegation
    -> snapshot.sources AST scan for trait-default detached methods
```

The target is:

```text
validated wire primary owners
    ↓
fresh relocation
    ↓
exact native semantic owners
    ↓
proved O(N) derived indexes
    ↓
source-independent compiler consumers
```

No canonical compiler consumer should need Universe AST merely to recover semantic decisions already made during the source build.

---

# 2. Repository grounding

## 2.1 Prepared state

Planning rechecked the remote branch directly.

```text
repository: aureat/phalcom-lang
branch: perf/universe-characterization

prepared head:
21a54705c33dae95ee9c77d86789b4911635e6b0
fix(perf): close callable analysis type roots

audit baseline:
2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a
feat(perf): extend artifact loading foundations
```

At planning time, `perf/universe-characterization` had no commits beyond `21a54705...`.

The six audits therefore remain directly applicable except where the one later commit extended callable-analysis root registration. PATCH2's primary-owner defects remain live.

## 2.2 Live implementation facts verified during planning

### `EffectiveCallableDefinition`

Current native owner:

```rust
pub enum CallableDefinitionOrigin {
    PrimaryDeclaration,
    InherentImpl(ImplId),
    ConformanceWitness(ImplId),
}

pub struct EffectiveCallableDefinition {
    pub callable: CallableId,
    pub origin: CallableDefinitionOrigin,
    pub source_member_index: usize,
    pub signature: CallableSemanticSignature,
    pub inherent_body_context: Option<InherentBodyContext>,
}

pub struct InherentBodyContext {
    pub generic_signature: Option<GenericSignature>,
    pub covering: Option<CoveringImplSubstitution>,
    pub source_target_head: TypeId,
}
```

`InherentBodyContext::signature_for_body` materially changes body checking through covering substitutions and source-relative `Self`. It is not diagnostic metadata.

`CoveringImplSubstitution` owns two inverse parameter maps:

```text
impl_to_decl
decl_to_impl
```

They must be transported canonically and validated as inverses.

### Accepted-definition import is currently false authority

Current artifact import manufactures one definition from each callable signature and sets:

```text
origin = PrimaryDeclaration
source_member_index = 0
inherent_body_context = None
```

This must be deleted, not decorated.

### Core lowering still reads Universe source

`phalcom-core/src/modules/semantic_lowering.rs` currently uses `snapshot.sources` to:

1. detect whether an inherent impl member is a generated delegation;
2. recover the delegation target field;
3. scan trait AST members to discover default implementations and their `source_member_index`.

PATCH2 owns removal of those canonical-source dependencies.

### Trait owner model

Current native `TraitSurfaceMember` contains:

```text
requirement
callable
signature: CallableSemanticSignature
visibility
source
default_present
default_source
```

`TraitAssociatedTypeRequirement` contains:

```text
requirement
name
domain: TypeMemberDomain
source
```

Current wire keeps only callable IDs and associated-type names.

The importer currently invents `KindId(0)`, `Public`, default spans, and `default_source = None`.

### Supertraits are not an independent native owner

The current trait exporter derives `super_traits` from the trait `GenericSignature` constraints whose subject is `Self`.

Therefore:

```text
generic_signature = primary authority
super_traits wire list = redundant derived check, if retained
```

PATCH2 must not create two independent supertrait authorities.

### Final dispatch surface is richer than the wire

Native `MemberSurface` contains:

```text
callables: CallableId -> TypeKnowledge
callable_signatures: Selector -> dispatch::CallableSignature
callable_visibility
fields / field_ids: TypeKnowledge
field_visibility
```

Current `DeclarationSurfaceRecordV1` carries callable IDs plus field signatures.

Current import gives every restored callable:

```rust
TypeKnowledge::assumed(TypeId(0), EvidenceOrigin::Flow)
```

That placeholder must disappear.

### Conditional inherent dispatch is a hidden authority gap

`SurfaceDispatchResolver` owns:

```text
surfaces
conditional_members
conditional_targets_by_owner
type_declarations
inherent_impl_domains
```

The source canonical baseline calls:

```text
register_surface(...)
register_conditional_members(...)
```

after `build_effective_surface`.

`ConditionalInherentMemberSet` is **not derivable from `DeclarationSurface` alone** and is not currently represented in `SemanticWorldV2`.

It contains checked authority such as:

```text
impl_id
domain
is_conditionally_applicable
callable
signature_template
visibility
source_member
is_requirement
```

PATCH2 must persist the conditional inherent authority required to rebuild `SurfaceDispatchResolver`.

### Witness-plan model

Native `ConformanceWitnessPlan` contains:

```text
impl_id
source_module
target_template
trait_ref_template
generic_signature
requirement_views
requirements
associated_type_plan
fingerprint
invalid_explicit_members
completeness
diagnostics
```

Current wire does not preserve that graph.

Native requirement selections include:

```text
ConformanceCallable
InherentCallable {
    callable,
    conditional_impl,
    applicability
}
ConditionalInherent {
    candidate,
    pending,
    fallback
}
DataComponent
TraitDefault
```

Current wire omits or rejects key variants/fields.

### Associated-type plan fingerprint bug

Current import:

```text
ConformanceAssociatedTypePlan::empty(impl_id)
then:
associated_type_plan.bindings = bindings
```

The fingerprint remains the empty-plan fingerprint.

PATCH2 must construct from final content and recompute.

### Derived conformance is primary

`DerivedConformanceRule` owns:

```text
impl_id
source_module
target_parameter
target_template
trait_ref
generic_signature
associated_type_plan
requirement_views
requirements
fingerprint
source
```

It is not reconstructible from ordinary conformance heads alone.

It must be wire authority.

### Trait dispatch is derived

`TraitDispatchIndex::build(conformance_index, trait_surfaces)` already:

- iterates lookup-eligible conformance heads;
- creates requirement contributions;
- sorts/deduplicates buckets;
- computes bucket and aggregate fingerprints.

Therefore PATCH2 must rebuild and verify `TraitDispatchIndex`; it must **not** serialize a second trait-dispatch truth.

### Static case owners

Native wire-required products exist:

```text
StaticDomainCaseCatalogProduct
StaticCaseSignatureProduct
```

They are currently omitted from `SemanticWorldV2` and loaded snapshot.

### Static definition owners

Native checked application identity is:

```text
StaticApplicationId {
    body: StaticBodyId,
    ordinal: u32
}
```

and the row is retained in `CheckedStaticBody`.

This owner decision is fixed.

Current operational wire:

```text
StaticApplication {
    target,
    arguments
}
```

drops the application ordinal and checked signature/category authority.

Current export only emits a static definition when:

```text
checked_body.root is already StaticDefinitionBodyRepr::Operational
```

even though `OperationalTemplate::from_checked_body` exists.

Current import returns detached `operational_templates`; the loaded snapshot's `static_definitions` remains empty.

### Operational lowering defect

`OperationalTemplate::from_checked_body` currently has two important unsafe/incomplete cases:

- TypeForm conversion can use the first checked static application as a shortcut;
- `KindedExpression` static-application arguments are unsupported.

PATCH2 must remove these restrictions or precisely fail canonical publication for a verified unsupported profile. Do not pick "first application".

### Static evaluator facts

The evaluator's source mode uses checked application rows to interpret:

```text
Kinded
Value
KindedExpression
```

arguments.

Operational mode evaluates a `StaticApplication` merely by target + child arguments.

To preserve checker authority in a source-free program, PATCH2 must make the canonical operational application refer to the owner-held checked application row.

### Static constants

`StaticConstantProduct` currently owns:

```text
id
declaration_range
initializer_range
checked: CheckedStaticConstantInitializer
outcome
residuals
value
lkg_value
diagnostics
```

`CheckedStaticConstantInitializer` contains a source AST expression plus:

```text
dependencies
static_constant_reads
record_key_static_constant_reads
result_type
input_fingerprint
```

Core lowering only projects successful values through:

```text
StaticConstantProduct::materialization_descriptor(static_values)
```

Query reuse consumes the checked input fingerprint and semantic dependencies.

Therefore the persisted compiler image needs the **reduced canonical authority and query/dependency summary**, not the initializer AST.

### Module static bindings

Native final owner:

```rust
pub struct ModuleBindingFact {
    pub symbol: SymbolId,
    pub source_body: CallableId,
    pub source_binding: BindingId,
    pub phase: ModuleBindingPhase,
    pub state: BindingState,
    pub declaration_range: SourceRange,
    pub initializer_range: Option<SourceRange>,
}

pub struct ModuleStaticBindingIndex {
    module: ModuleId,
    bindings: BTreeMap<SymbolId, ModuleBindingFact>,
    visible_names: BTreeMap<Box<str>, SymbolId>,
}
```

`from_module_analysis` reconstructs these facts by scanning source syntax and matching name/range against callable bindings.

Artifact import must not rerun that function.

`source_binding` is body-local. It must be encoded through PATCH1's owner-qualified binding ordinal, not raw `BindingId`.

---

# 3. Required reads before implementation

Read in this order.

## 3.1 Lifecycle and current P7 state

```text
AGENTS.md
docs/implementation/README.md

docs/implementation/PERF003-runtime-performance/
  C2-compilation-performance-and-efficiency/
    CHECKPOINT.md

docs/implementation/PERF003-runtime-performance/
  C2-compilation-performance-and-efficiency/P7/
    PERF003.C2.P7-telosc-persisted-semantic-compiler-image-detailed-implementation-plan.md
    PERF003.C2.P7-remaining-execution-plan.md
    PERF003.C2.P7-walkthrough.md
    PERF003.C2.P7-handoff.md
```

Read the current audit reports under P7-R1/reports.

## 3.2 PATCH0 and PATCH1 takeover artifacts

Read their current checkpoint/walkthrough/handoff state.

PATCH2 assumes PATCH1 has established:

- owner-qualified binding ordinals;
- portable `BindingState`;
- stable semantic dependency codec where shared;
- frozen/read APIs needed for imported binding-backed owners.

If those are not implemented yet, PATCH2 may develop wire DTOs synthetically but must not duplicate those contracts.

## 3.3 Primary source

```text
phalcom-semantic/src/impls.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/surface.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/derived_conformance.rs

phalcom-semantic/src/static_case.rs
phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/module_bindings.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/static_constant_demand.rs

phalcom-semantic/src/artifact/manifest.rs
phalcom-semantic/src/artifact/roots.rs
phalcom-semantic/src/artifact/export.rs
phalcom-semantic/src/artifact/import.rs
phalcom-semantic/src/artifact/operational.rs

phalcom-compiler-artifact/src/semantic/world.rs
phalcom-compiler-artifact/src/semantic/operational.rs

phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/query.rs

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/compile.rs
```

Read the exact current test files adjacent to those owners before adding a new test hierarchy.

---

# 4. Normative authority

Authority order:

1. user-ratified P7 consolidated patch program;
2. current PERF003.C2 checkpoint and P7 amendments;
3. native source semantic owner types;
4. actual compiler/evaluator consumers;
5. persistence manifest as an architectural disposition target;
6. audits as defect evidence;
7. future P8/P9 plans only as compatibility context.

Important correction:

> The current persistence manifest says `dispatch` is derived from surfaces. The live resolver also owns conditional inherent sets that are not present in `DeclarationSurface`. PATCH2 must make the manifest executable by naming the missing wire prerequisite rather than pretending surface rows alone are sufficient.

A manifest label never overrides live owner reality.

---

# 5. Patch boundary

## 5.1 PATCH2 owns

PATCH2 owns semantic fidelity for:

1. accepted callable definitions;
2. compiler-required definition body provenance;
3. trait headers/surfaces;
4. declaration dispatch surfaces;
5. conditional inherent dispatch authority;
6. conformance witness plans;
7. associated type witness plans;
8. derived conformance rules;
9. deterministic trait-dispatch rebuild;
10. static case catalogs;
11. static case signatures;
12. canonical static definition programs;
13. checked static applications under `CheckedStaticBody`;
14. static definition products;
15. static constant reduced authority;
16. final module static binding indexes;
17. source-independent core readers for delegation/default-method authority;
18. roots and cross-owner validation for all above;
19. per-family export-coverage guard removal.

## 5.2 PATCH2 does not own

- complete callable-analysis transport — PATCH1;
- exact whole-loader allocation closure — PATCH3;
- query-image evidence/root reconstruction — PATCH4;
- `.telosc` physical canonicality — PATCH5;
- final canonical-image promotion/module-products assembly — PATCH6;
- package/performance certification — PATCH7;
- executable VM image — P8;
- artifact discovery/freshness/incremental rebuild — P9.

## 5.3 PATCH2 may prepare but must not falsely certify allocation

Every new importer should accept the existing allocation transaction where its constructors already support charged allocation.

Do not claim global exact accounting.

If an owner requires native map-node allocation that PATCH3 will replace, expose a clean fallible construction seam now and leave the global `UnaccountedBoundary` in place.

---

# 6. Architecture

# 6.1 Primary-vs-derived rule

A product is **Wire** if reconstructing it requires making a semantic choice.

A product is **Derived** only if:

```text
validated wire owners
    -> one deterministic O(N) algorithm
    -> exact source parity
```

with no source syntax, name resolution, checker invocation, or heuristic.

For PATCH2:

| Owner | Disposition |
|---|---|
| EffectiveCallableDefinition | Wire |
| InherentBodyContext | Wire |
| definition body provenance/delegation target | Wire |
| TraitHeader | Wire |
| TraitSurface | Wire |
| DeclarationSurface | Wire |
| conditional inherent member sets/domains | Wire prerequisite |
| SurfaceDispatchResolver | Derived |
| ConformanceWitnessPlan | Wire |
| ConformanceAssociatedTypePlan | Wire |
| DerivedConformanceRule | Wire |
| TraitDispatchIndex | Derived |
| StaticDomainCaseCatalogProduct | Wire |
| StaticCaseSignatureProduct | Wire |
| canonical static definition program | Wire |
| StaticDefinitionProduct semantic authority | Wire |
| reduced static constant authority | Wire |
| ModuleStaticBindingIndex | Wire |
| callable_definitions_by_module | Derived from accepted definitions |
| trait requirement layout | Derived from exact trait members |
| broad source index | Omitted |

---

# 6.2 Accepted callable definitions

Introduce a portable owner record conceptually:

```text
CallableDefinitionRecord
    callable
    origin:
        PrimaryDeclaration
        InherentImpl(stable impl)
        ConformanceWitness(stable impl)

    source_member_index

    signature: complete CallableSemanticSignature

    inherent_body_context?:
        generic_signature?
        covering?:
            impl_to_decl[]
            decl_to_impl[]
        source_target_head

    body_provenance:
        OrdinarySourceBody
        GeneratedDelegation {
            target_field
        }
```

Exact type names are flexible.

## Signature ownership

Use an inline complete semantic signature or an exact canonical reference whose 1:1 validity is mechanically proven.

The safer existing precedent is the enum-requirement row, which carries its complete owner-held signature because a required callable signature is not guaranteed to live in the global table.

Do not infer definition semantics from `callable_signatures`.

## Covering substitution

Wire maps must be canonical sorted `(parameter, parameter)` pairs.

Validation:

```text
no duplicate source parameter
no duplicate target parameter
impl_to_decl and decl_to_impl are exact inverses
parameter owners match expected impl/declaration owners
parameter kinds agree
```

## `callable_definitions_by_module`

Derived **only** after accepted definitions import succeeds:

```rust
for (callable, definition) in callable_definitions {
    by_module[callable.module()].push(callable)
}
sort
dedup
```

Never derive it from signatures.

---

# 6.3 Definition body provenance and source-free lowering

The live code needs one additional semantic fact that `EffectiveCallableDefinition` does not currently retain:

```text
is this inherent member a generated delegation?
if yes, what canonical target field does it delegate to?
```

Move this fact into semantic authority when the inherent contribution is built.

Recommended native addition:

```rust
pub enum CallableDefinitionBodyProvenance {
    SourceBody,
    GeneratedDelegation {
        target_field: Box<str>,
    },
}
```

or an equivalent explicit field.

Populate it in `impls.rs` while handling `ImplMember::Delegation`.

Carry it through:

```text
InherentMemberContribution
    ↓
EffectiveCallableDefinition
    ↓
artifact record
    ↓
imported definition
    ↓
core lowering
```

Core must then delete the AST read at the current generated-delegation seam.

A generated delegation may legitimately lack a `CallableAnalysis`; body provenance is what authorizes its generated lowering.

Do not infer generated status from missing analysis.

---

# 6.4 Trait default body identity

Core currently scans trait AST and matches ranges to recover:

```text
trait default callable
source_member_index
```

Make the native trait surface own the required member ordinal.

Recommended addition:

```rust
TraitSurfaceMember {
    ...
    source_member_index: usize,
}
```

Populate it from `trait_def.members.iter().enumerate()`.

For property requirements that generate getter/setter requirements from one source property, both generated members may share the same source member index. Their canonical callable/requirement IDs disambiguate them.

Carry this ordinal on wire.

Core lowering then obtains trait defaults by iterating exact `TraitSurfaceMember`s:

```text
if member.default_present:
    DetachedMethodSource::TraitDefault {
        trait_declaration,
        source_member_index: member.source_member_index
    }
```

No AST scan.

---

# 6.5 Trait headers and surfaces

Replace the current lossy trait DTO with a complete owner record.

Conceptually:

```text
TraitRecord
    declaration

    header:
        generic_signature?
        source

    members[]:
        requirement
        callable
        complete CallableSemanticSignature
        visibility
        source
        source_member_index
        default_present
        default_source?

    associated_types[]:
        requirement
        name
        domain
        source

    diagnostics[]    # if canonical profile permits them

    derived_super_traits[]  # optional redundant integrity field only
```

## Supertrait rule

If the existing `super_traits` field remains for compatibility:

- export it deterministically from generic signature;
- validator recomputes it;
- mismatch rejects;
- importer does **not** treat it as a second authority.

## Requirement layout

Do not persist `TraitRequirementLayout` unless a measured reason requires it.

Rebuild:

```rust
surface.finalize_requirement_layout();
```

after exact members are installed.

Parity-test slot order/fingerprint.

---

# 6.6 Exact declaration dispatch surfaces

Replace callable-ID-only surface transport.

Conceptual rows:

```text
DeclarationSurfaceRecord
    declaration

    instance_callables[]:
        callable
        dispatch_signature
        return/type knowledge
        visibility

    class_callables[]:
        ...

    fields[]:
        field_id
        knowledge
        visibility

    constructor? / other final surface facts if present in live owner
```

Use the live `dispatch::CallableSignature` field census:

```text
selector
parameters
return_type
generics
kind
```

and the live callable parameter contract fields.

Do not synthesize `TypeKnowledge`.

Do not force all visibility to Public.

Where `CallableSemanticSignature` is already exact and the projection function is proven lossless for the dispatch product, it is permissible to derive `dispatch::CallableSignature` from that semantic signature **only if every surface callable has an exact transported semantic signature owner**.

Because generated/surface-only callables can violate that assumption, verify first. If not universal, persist the final dispatch signature directly.

## Fields

Likewise, use exact final field knowledge/visibility. Do not assume the separate `FieldSemanticSignature` table is sufficient unless parity tests prove it.

---

# 6.7 Conditional inherent dispatch authority

Add a wire family under the semantic primary section.

Conceptual:

```text
ConditionalInherentSetRecord
    target:
        Declaration
        ExactEnumCase

    members[]:
        impl_id
        domain:
            target
            head_type
            generic_signature?
            constraints[]
        is_conditionally_applicable
        callable
        signature_template
        visibility
        source_member
        is_requirement
```

Validation:

```text
set target matches every member domain target
member impl_id matches domain impl_id
callable owner compatible with target
source_member in portable integer range
no duplicate semantic member key
signature callable == member callable
generic constraints own valid relocated roots
```

Import:

```text
surface rows + conditional sets
    ↓
SurfaceDispatchResolver::register_surface
SurfaceDispatchResolver::register_conditional_members
    ↓
register declaration type mappings from exact structural owner data
```

Do not import `SurfaceDispatchResolver` itself as opaque state.

---

# 6.8 Complete witness authority

Replace `WitnessPlanV1`'s summary contract with field-complete proof transport.

## Requirement views

Persist every:

```text
TraitRequirementId
InstantiatedTraitRequirement {
    requirement
    callable
    signature
    visibility
    default_callable
}
```

## Requirement selections

Encode every native variant and every field.

Especially:

```text
InherentCallable {
    callable
    conditional_impl
    applicability
}

ConditionalInherent {
    candidate
    pending
    fallback
}
```

Do not reject them merely to retain the current schema.

## Completeness

Use a tagged portable enum with payloads matching:

```text
Complete
Incomplete { failures[] }
Unknown(reason)
Blocked(reason)
Dynamic(obligation)
Cancelled
BudgetExceeded(report)
InternalFailure(message)
```

No `u8`-to-generic-placeholder reconstruction.

## Invalid explicit members

Persist exact bool.

## Diagnostics

Preserve owner diagnostics required by semantic parity.

Canonical Universe publication may separately require an accepted diagnostic policy, but import must not erase nonempty source facts and then claim equality.

---

# 6.9 Associated type plan

Portable plan:

```text
AssociatedTypePlanRecord
    impl_id

    bindings[]:
        requirement
        value_template:
            Type(type)
            Dynamic
        source_impl
        source span

    failures[]:
        kind:
            Missing
            Duplicate
            Unknown
            Invalid
        name
        reason
        source
        requirement?

    diagnostics[]
```

Import must build final collections first.

Then:

```text
let plan = ConformanceAssociatedTypePlan {
    ...
    fingerprint: recompute_from_final_bindings_failures(...)
}
```

Prefer a new constructor:

```rust
ConformanceAssociatedTypePlan::from_parts(...)
```

so stale fingerprints cannot be created by mutation after `empty()`.

Delete the current `empty() + assign bindings` import pattern.

---

# 6.10 Witness fingerprint

Do not treat the existing wire `u64` as semantic authority.

Wire fingerprint may remain as an integrity check.

Import order:

```text
relocate all owner fields
build associated plan with fresh fingerprint
build complete witness plan
recompute witness product fingerprint
compare portable semantic checksum when contract defines one
publish
```

Add discriminator proving nonempty associated binding changes the imported fingerprint.

---

# 6.11 Derived conformance rules

Add explicit wire records.

Conceptually:

```text
DerivedConformanceRuleRecord
    impl_id
    source_module
    target_parameter
    target_template
    trait_ref
    generic_signature?
    associated_type_plan
    requirement_views[]
    requirements[]
    source
```

Do not serialize `fingerprint` as primary authority.

Import using final constructor:

```rust
DerivedConformanceRule::new(...)
```

so the native fingerprint is recomputed.

Insert into `DerivedConformanceRuleIndex`.

Validate:

```text
rule impl owner
target parameter owner
target template roots
trait declaration
requirement owners
selection owners
associated plan impl_id
```

---

# 6.12 Trait dispatch rebuild

After:

```text
conformance_index
trait_surfaces
```

are final, rebuild:

```rust
let trait_dispatch_index =
    TraitDispatchIndex::build(&conformance_index, &trait_surfaces);
```

Compare:

- lookup results;
- ordered bucket contents;
- bucket fingerprints;
- aggregate fingerprint;

against the source-built snapshot in parity tests.

Do not add trait-dispatch wire rows unless this proof fails because the build function lacks source-independent primary authority. If it fails for that reason, STOP AND CONSULT.

---

# 6.13 Static case catalogs and signatures

Add direct wire rows.

## Catalog

```text
StaticDomainCaseCatalogRecord
    domain
    cases[]:
        case id
        shape:
            Singleton
            Constructor { selector slots[] }
        source_range
```

## Signature

```text
StaticCaseSignatureRecord
    case id
    parameter_kinds[]
    source_range
```

Register all kind/domain roots before type graph closure.

Validate:

```text
case domain exists
case IDs unique
signature case exists in catalog
constructor arity == parameter_kinds length where applicable
```

Do not infer catalogs from names after load.

---

# 6.14 Canonical static definition program

This is the core static architecture decision.

A persisted compiler image cannot keep AST as compiler authority, and it cannot reduce a checked application to target name alone.

Introduce a **canonical source-independent static program** produced from `CheckedStaticBody`.

Recommended native concept:

```rust
pub struct CanonicalStaticProgram {
    pub owner: StaticBodyId,
    pub parameter_count: u32,
    pub local_count: u32,
    pub root: OperationalNode,

    pub applications: Arc<[CanonicalCheckedStaticApplication]>,

    pub parameter_bindings: ...,
    pub kinded_parameters: ...,
    pub static_constant_bindings: ...,
    pub static_constant_reads: ...,
    pub dependencies: ...,
    pub result: StaticValueDomain,

    pub input_fingerprint: InputFingerprint,
    pub module_binding_fingerprint: ProductFingerprint,
}
```

Exact auxiliary fields must follow the live evaluator/compiler reader census.

## Source-only vs canonical authority

A source build may retain `CheckedStaticBody` for incremental checking/diagnostics.

After checking, derive exactly one `CanonicalStaticProgram`.

The **compiler/evaluator product authority** for persistence and product fingerprinting becomes that canonical program.

Do not maintain two independently mutable executable authorities.

A reasonable `StaticDefinitionProduct` evolution is:

```text
definition metadata/signature/source range
canonical_program: Arc<CanonicalStaticProgram>
source_checked_body: Option<Arc<CheckedStaticBody>>  # source-only auxiliary
diagnostics
alias_info
```

Names flexible.

The loaded artifact sets `source_checked_body = None`.

If changing the owner this way conflicts materially with current P7/P8 contracts, STOP AND CONSULT.

### Adopted T7/T12 evaluator clarification — 2026-10-16

The architect decision for `INC-PATCH2-T7-T12-STATIC-EVAL` selects
source-independent canonical static execution in PATCH2; G4/G6 are not deferred.
The persistent executable frame is the pair:

```text
StaticDefinition.signature + CanonicalStaticProgram
```

`StaticBodyId` remains the durable body owner and qualifies body-local
application ordinals. `CheckedStaticBody` remains source-side checking,
incremental-recomputation, and diagnostic authority. A canonical program is
derived once from it and becomes definition execution authority after
canonicalization/import.

This clarification supersedes the tentative source-map fields in the concept
sketch above. Do not add `parameter_bindings`, `local_slots`,
`static_constant_bindings`, `ExpressionId` maps, or `BindingId` maps to the
canonical program or its wire representation. Operational parameter indexes,
local slots, stable static-constant symbols, and body-qualified application
ordinals are the execution layout. Validate them against the definition
signature and canonical program before execution. The signature remains the
authority for parameter domains, kinded `TypeParameterId`s, and result domain;
`kinded_parameters` is a sorted compact lane projection validated against that
signature.

The evaluator must have a first-class canonical definition frame. Its arguments
are ordinal-aligned `StaticArgument`s and its locals are sized by
`CanonicalStaticProgram.local_count`. Source evaluation may retain a
`CheckedStaticBody` path as a differential/auxiliary path, but production
definition execution must use the canonical program when present. Imported
execution and nested calls require `canonical_program`; missing `checked_body`
is normal. Canonical `ReadParameter` and local accesses must not synthesize or
fall back to `BindingId(index)`.

No schema expansion is authorized for frame maps. Preserve fail-closed guards
until canonical-frame validation, source/canonical and fresh-store evaluation
parity, source-denied execution, fingerprint parity, and malformed-layout
checks pass. Global PATCH3 allocation guards remain unchanged.

---

# 6.15 Checked static applications

Canonical row:

```text
CanonicalCheckedStaticApplication
    ordinal
    source_range
    target
    full checked StaticDefinitionSignature

    arguments[]:
        parameter_index
        category:
            KindedTerm { term }
            KindedExpression
            Value
```

The row owner is `CanonicalStaticProgram.owner == StaticBodyId`.

There is no portable source `ExpressionId`.

## Operational application node

Change the portable/native operational node to:

```text
StaticApplication {
    application_ordinal,
    arguments[]
}
```

The program owner qualifies the ordinal.

The evaluator resolves:

```text
program.applications[application_ordinal]
```

and obtains target/signature/argument categories from the checked row.

Do not keep `target` in the operational node as a second authority unless retained only as a validated redundant checksum.

## Argument cross-validation

Require:

```text
application arguments length matches checked row
parameter indexes valid and unique
row signature parameter categories agree
KindedTerm owns a relocated type root
KindedExpression child evaluates a kinded term
Value child evaluates a static value/residual as allowed
```

---

# 6.16 Repair operational canonicalization

`OperationalTemplate::from_checked_body` must become total for the canonical P7-supported checked language subset.

Required fixes:

1. do not use `static_applications.first()` for a TypeForm;
2. select the checked application by exact source expression/application relation;
3. lower nested `KindedExpression` recursively into a child operational node;
4. preserve evaluation order;
5. preserve application ordinal;
6. reject ambiguous or dangling expression→application mapping;
7. support every checked body representation accepted by canonical Universe:
   - TypeForm;
   - Expression;
   - Block;
   - Operational.

If a checked feature in the real Universe has no source-independent operational form, STOP AND CONSULT after producing the minimal owner/example.

Do not solve by serializing AST.

---

# 6.17 Static definition product transport

Wire record concept:

```text
StaticDefinitionRecord
    id
    signature
    source_range
    alias_info?
    canonical_program
    diagnostics if permitted by profile
```

Do not export only free-floating templates.

Import must insert a real `StaticDefinitionProduct` into `StaticDefinitionTable`.

`ValidatedSemanticParts.operational_templates` should disappear as a detached final product, or become purely temporary codec state consumed immediately into definition owners.

Core must never discard the only static program copy.

---

# 6.18 Static definition fingerprints

Current product fingerprint hashes the source representation variant and `Debug` of the checked body.

That cannot provide source/import semantic parity when import intentionally has no AST.

PATCH2 must define a representation-independent **semantic product fingerprint**.

Recommended:

```text
hash:
    definition identity
    complete semantic signature
    canonical program
    checked application table
    result/domain
    semantic dependencies
    alias info
```

Source build:

```text
CheckedStaticBody
    -> CanonicalStaticProgram
    -> semantic product fingerprint
```

Import:

```text
wire CanonicalStaticProgram
    -> relocated CanonicalStaticProgram
    -> same semantic product fingerprint
```

Source syntax remains part of an **input fingerprint**, not the persisted product meaning.

Do not normalize source/input and product fingerprints into one hash.

---

# 6.19 Static constants: persist reduced authority, not initializer AST

Refactor the persistent part of a static constant into a source-independent summary.

Conceptually:

```text
StaticConstantAuthority
    id
    declaration_range
    initializer_range

    input_fingerprint
    semantic_dependencies[]
    static_constant_reads[]
    record_key_static_constant_reads[]
    result_type?

    outcome
    value?
    lkg_value?
    diagnostics
```

Source builds may retain an optional source checked initializer for incremental recomputation.

Imported canonical products do not.

Add accessors so query/fingerprint consumers stop reaching through:

```text
product.checked.<field>
```

for data that is part of the persistent authority.

## P7 outcome profile

VERIFY FIRST on the real canonical Universe.

If all canonical constants are successfully reduced, P7 may require:

```text
outcome == Reduced(value)
```

and reject other outcomes at artifact publication.

If canonical Universe contains:

```text
ReducedKinded
Symbolic
Invalid
Recursive
BudgetExceeded
Cancelled
Unknown
Blocked
Dynamic
```

do not fabricate a representation.

Classify whether the payload already has stable source-independent codecs.

If a required outcome needs residual graph transport or new semantic architecture, STOP AND CONSULT.

## Materialization

Imported reduced values reference the relocated `StaticValueStore`.

`StaticConstantProduct::materialization_descriptor` must work identically.

No source evaluation on load.

---

# 6.20 Module static bindings

Wire the **final `ModuleStaticBindingIndex`**, not `ModuleStaticBindingCatalog` source inputs and not AST declarations.

Conceptual:

```text
ModuleStaticBindingIndexRecord
    module

    bindings[]:
        symbol
        source_body
        source_binding_ordinal
        phase
        complete BindingState
        declaration_range
        initializer_range?

    visible_names[]:
        local name
        symbol
```

`source_binding_ordinal` is interpreted under `source_body` using PATCH1's binding map.

Validation:

```text
module matches symbols
source_body belongs to compatible module
binding ordinal resolves
BindingState.binding == resolved binding
symbol unique
visible name unique
visible target exists
phase closed enum
```

Import does **not** call `from_module_analysis`.

---

# 6.21 Source-independent compiler reader

After accepted definitions and trait members carry explicit provenance:

delete canonical-source reads for:

```text
generated inherent delegation detection
delegation target field
trait-default detached method discovery
```

Core path becomes owner-driven.

### Inherent delegation

Use:

```text
definition.body_provenance
```

to decide generated delegation and target field.

### Trait default

Use:

```text
TraitSurfaceMember.default_present
TraitSurfaceMember.source_member_index
```

to publish `DetachedMethodSource::TraitDefault`.

### Anonymous products

Do **not** expand PATCH2 into source-independent arbitrary user AST lowering.

`project_anonymous_products` may remain source-backed for user modules.

Acceptance must prove canonical imported Universe modules are not silently re-lowered through that fallback during normal user compilation.

---

# 7. Cross-owner validation

Before snapshot insertion, validate all relationships.

## 7.1 Callable definitions

```text
definition.callable == definition.signature.callable
origin impl exists when applicable
inherent context exists iff required
covering maps inverse
source_member_index portable
generated delegation allowed only for inherent origin
delegation target field nonempty
no duplicate callable definition
```

## 7.2 Trait surfaces

```text
member requirement.owner == trait declaration
member callable/signature agree
member visibility closed enum
default_source present iff default_present where native invariant requires
associated requirement owner/index valid
associated domains relocate
generic signature owner valid
derived supertrait checksum matches generic signature
```

## 7.3 Dispatch surfaces

```text
surface declaration exists
callable identity agrees with selector/side
dispatch signature and callable knowledge agree
visibility exists
field ID belongs to declaration
no duplicate selector ambiguity beyond native semantics
conditional set target/member/domain agree
```

## 7.4 Witness/derived rules

```text
impl IDs unique
trait refs valid
requirement IDs belong to trait
requirement views exact
selection callable/impl targets valid
associated plan impl matches
fallback graph finite
completeness discriminants closed
failure requirement IDs valid
```

## 7.5 Static owners

```text
case catalog/signature closure
definition ID unique
canonical program owner == definition static body ID
application ordinals dense/unique
operational application refs resolve
checked signature target == definition target row
static value refs relocate
module binding references PATCH1 callable/binding owner
```

Validation must be bounded.

---

# 8. Invariants

### INV-01 — No fabricated semantic authority

No placeholder `TypeId(0)`, `KindId(0)`, `Public`, empty proof graph, default span, generic failure, or synthetic `PrimaryDeclaration` may be created solely because the wire omitted source truth.

### INV-02 — Definitions are definitions

`callable_definitions` is populated only from accepted-definition records.

### INV-03 — Module definition index is derived from definitions

Never from signatures.

### INV-04 — Body provenance is semantic authority

Generated delegation/default method behavior is retained before source omission.

### INV-05 — Trait member parity

Signature, visibility, source/default provenance, domain, and member identity survive relocation.

### INV-06 — Supertrait single authority

Generic signature constraints are primary; redundant supertrait lists are validation-only.

### INV-07 — Dispatch knowledge is exact

No arbitrary assumed type may stand in for final dispatch callable/field knowledge.

### INV-08 — Conditional inherent authority survives

User overlay dispatch must see the same conditional inherent domains/members as source-built canonical dispatch.

### INV-09 — Witness proof state is lossless

Every native selection/completeness/failure/applicability dimension is preserved.

### INV-10 — Fingerprints derive from final owners

Associated plans, witness plans, derived rules, static products, and derived indexes recompute fingerprints after relocation.

### INV-11 — Trait dispatch has one authority

Rebuild it from exact primary semantic owners; do not serialize a second independent index.

### INV-12 — Checked static applications belong to `StaticBodyId + ordinal`

Never callable analysis, never target-only identity.

### INV-13 — Static program is source-independent

No AST is required to evaluate an imported canonical static definition.

### INV-14 — Operational nodes reference checked authority

A static application operation resolves an owner-local checked row.

### INV-15 — No first-match static shortcuts

No `.first()` or name/range guess may select a checked static application.

### INV-16 — Static constants are not re-evaluated on load

Persist reduced semantic authority/dependencies.

### INV-17 — Module static bindings are final facts

Do not rerun `from_module_analysis`.

### INV-18 — Body-local binding identity remains qualified

Module binding source bindings use `(source_body, binding ordinal)`.

### INV-19 — Canonical source omission is real

Core lowering succeeds for canonical imported authority without `snapshot.sources` for delegation/default discovery.

### INV-20 — Broad source index stays omitted

PATCH2 does not reconstruct it.

### INV-21 — Derived products are O(N)

Trait dispatch, requirement layout, callable module index, resolver indexes rebuild in bounded deterministic work.

### INV-22 — No detached semantic products

Operational programs/static owners are reachable through their owning snapshot tables.

### INV-23 — Allocation architecture remains compatible

New importer APIs do not create deliberate hidden unbudgeted copies that PATCH3 cannot close.

### INV-24 — Guard discipline

Remove each owner-family guard only after its parity gate passes.

### INV-25 — P7 remains open

PATCH2 completion is not full artifact/load/package certification.

---

# 9. Decision authority

# 9.1 FIXED

Luna must not change:

1. accepted definitions are wire authority;
2. `EffectiveCallableDefinition` is not inferred from signatures;
3. generated delegation semantics move out of AST lookup;
4. trait-default source member identity moves out of AST lookup;
5. trait surface fields are preserved exactly;
6. generic signature is the primary supertrait authority;
7. final declaration surfaces contain real callable/field knowledge;
8. conditional inherent sets are required to rebuild dispatch;
9. witness plan proof state is field-complete;
10. associated plan fingerprint is computed after final bindings;
11. derived conformance rules are wire primary;
12. trait dispatch is deterministic derived state unless parity disproves that;
13. checked static applications remain `CheckedStaticBody`/static-program owned;
14. imported static definitions use source-independent canonical programs;
15. operational static applications refer to checked application rows;
16. AST is not serialized to solve static program persistence;
17. static constants are not re-evaluated on load;
18. final module static binding index is persisted;
19. module binding local IDs use PATCH1 owner-qualified binding identity;
20. global semantic allocation guards remain through PATCH2.

# 9.2 MECHANICALLY FLEXIBLE

Luna may adapt:

- DTO names/version suffixes;
- private helper organization;
- use of inline signature record vs validated reference;
- exact enum names for body provenance;
- exact canonical static program type name;
- whether static application rows are embedded in a program record or referenced from a body-local array;
- internal sorted-vector/map representation;
- exact test placement;
- exact builder method names.

# 9.3 VERIFY-FIRST

Verify before editing:

| Assumption | Evidence | Response if false |
|---|---|---|
| PATCH1 portable BindingState/ordinal contract exists | PATCH1 handoff/live code | do not duplicate; STOP if PATCH2 cannot reference it |
| PATCH2 semantic-world version policy | user decision, 2026-10-16 | use semantic-world schema 3; preserve outer container schema 2 and reject semantic-world schema 2 |
| all canonical trait surface members have stable source member ordinal | trait builder | add owner fact if straightforward |
| generated delegation can be identified during impl contribution construction | `ImplMember::Delegation` path | add body provenance there |
| final dispatch signature cannot universally be derived from global semantic signatures | surface census | if universal proof exists, derive instead of duplicate |
| `TraitDispatchIndex::build` parity is exact | source/import differential test | if not, consult |
| canonical static definitions can all canonicalize to operational programs | real Universe census | if not, consult |
| canonical static constants are reduced | real Universe census | if not, classify outcome codecs; consult if symbolic/residual required |
| product fingerprints can be made representation-independent without violating query contract | fingerprint tests/spec | consult if semantics unclear |
| conditional inherent sets are absent from current wire | world.rs | if concurrent work added them, reuse it |
| `source_member_index=0` for primary definitions is intentional source semantics | consumers | preserve; do not “fix” by guess |

---

# 10. Global STOP / CONSULT triggers

Stop when:

1. any required owner lacks a stable identity;
2. artifact schema compatibility requires a policy decision;
3. source member ordinal meaning is ambiguous;
4. a final dispatch fact cannot be reconstructed without re-running semantic selection;
5. `TraitDispatchIndex::build` does not reproduce source index from exact primaries;
6. a witness selection variant requires a new proof-owner design;
7. canonical Universe needs a static feature `OperationalTemplate::from_checked_body` cannot represent after one serious repair;
8. symbolic static constants require residual graph persistence;
9. static evaluator can only run imported programs by retaining AST;
10. a static application would need fabricated source `ExpressionId`;
11. module static binding cannot resolve a source binding through PATCH1 identity;
12. source-independent core lowering still requires canonical AST after provenance changes;
13. a second competing semantic authority is proposed;
14. a nontrivial semantic parity failure persists after one serious correction;
15. PATCH2 scope begins absorbing global allocation/query/container work.

Consultation packet:

```text
trigger
revision
owner type
wire type
exporter
importer
consumer
minimal source owner/example
observed mismatch
two plausible options
decision needed
```

---

# 11. Debugging budget

Mechanical failures: up to three coherent cycles with progress.

Semantic parity failures: one serious correction.

Architecture failures: zero speculative redesign cycles.

For semantic failures record:

```text
Observed
Expected invariant
Hypothesis
Evidence
Prediction
Minimal discriminator
```

Never repeatedly rerun a test unchanged.

---

# 12. Coverage design

| ID | Dimension | Required proof |
|---|---|---|
| CV-01 | primary definition | Primary origin/signature/member index parity |
| CV-02 | inherent definition | impl origin + complete body context |
| CV-03 | conformance definition | witness origin parity |
| CV-04 | covering context | inverse parameter map + Self specialization parity |
| CV-05 | generated delegation | target field survives with no source |
| CV-06 | module definition index | exact set from definitions |
| CV-07 | trait member | signature/visibility/source/default/member ordinal |
| CV-08 | associated type | exact domain/source |
| CV-09 | supertrait | generic-derived checksum parity |
| CV-10 | dispatch callable | exact TypeKnowledge/signature/visibility |
| CV-11 | dispatch field | exact knowledge/visibility |
| CV-12 | conditional inherent | domain/member lookup parity |
| CV-13 | witness views | exact instantiated requirement views |
| CV-14 | witness selection | every selection variant |
| CV-15 | completeness | every payload-bearing variant |
| CV-16 | associated plan | bindings/failures/diagnostics/fingerprint |
| CV-17 | invalid explicit member | bool parity |
| CV-18 | witness diagnostics | parity |
| CV-19 | derived rule | full owner parity |
| CV-20 | trait dispatch | deterministic rebuild/fingerprint parity |
| CV-21 | static catalog | domain/case shape parity |
| CV-22 | static case signature | kind parity |
| CV-23 | static program | TypeForm/Expression/Block canonicalization |
| CV-24 | static application | body+ordinal identity |
| CV-25 | nested kinded application | no source expression identity required |
| CV-26 | operational execution | source/import equal outcome |
| CV-27 | static definition fingerprint | representation-independent parity |
| CV-28 | static constant | reduced value/materialization parity |
| CV-29 | static constant dependency | input/dependencies/read summary parity |
| CV-30 | module binding | symbol/body/binding/state/phase parity |
| CV-31 | imported binding seed | user checker sees exact canonical module binding |
| CV-32 | source denial | delegation/default lowering without canonical source |
| CV-33 | root closure | all new type/static roots present |
| CV-34 | malformed owner | duplicates/wrong owner rejected |
| CV-35 | guard frontier | only completed owner-family coverage guards removed |
| CV-36 | detached templates | none survive as orphan final state |

---

# 13. Verification budget

## BUILD

Run one focused test target per coherent owner family.

Do not run workspace tests.

## STABILIZE

Run all PATCH2 family parity tests and source-independent lowering differential.

## CERTIFY

PATCH2 certification remains focused. PATCH7 owns full P7 certification.

Verification ladder:

```text
T0 exact new discriminator
T1 owner-family codec/export/import test
T2 semantic owning subsystem test
T3 semantic→core consumer parity
T4 affected crate suite only if evidence requires
T5 workspace/release deferred
```

Explicitly defer:

```text
cargo test --workspace
full corpus
full package matrix
complete full-loader allocation
cross-process artifact determinism
P5/P6 full certification
P8/P9
```

---

# 14. Tasks

# T0 — Live takeover and owner census

## Purpose

Anchor the exact live revision and convert PATCH2 scope into a current field/consumer/codec matrix.

## Produce

For every PATCH2 family:

```text
native owner
native fields
wire fields
root fields
export adapter
validation
import adapter
snapshot insertion
compiler/query consumers
fingerprint
current placeholder/default
test
```

Also record exact current `snapshot.sources` reads in compiler paths and classify:

```text
canonical-authority dependency
user-source-only acceptable dependency
incidental source-id fallback
```

## Commands

```bash
git branch --show-current
git rev-parse HEAD
git status --short
```

Read current PATCH1 handoff.

Run only existing focused semantic-world/trait/static tests necessary to establish baseline.

## Acceptance

No production edits required.

The checkpoint records PATCH2 start and inherited failures.

---

# T1 — Accepted callable definition transport and body provenance

## Purpose

Eliminate fabricated definitions and canonical AST dependence for generated inherent delegation.

## Files

```text
phalcom-semantic/src/impls.rs
phalcom-semantic/src/artifact/roots.rs
phalcom-semantic/src/artifact/export.rs
phalcom-semantic/src/artifact/import.rs
phalcom-compiler-artifact/src/semantic/world.rs
phalcom-semantic/src/db/query.rs
phalcom-core/src/modules/semantic_lowering.rs
```

## Source model changes

Add explicit body provenance to the accepted-definition pipeline.

At the `ImplMember::Delegation` branch in `build_inherent_impl_contribution`, capture:

```text
GeneratedDelegation { target_field }
```

on each generated `InherentMemberContribution`.

Ordinary members carry:

```text
SourceBody
```

Carry it into `EffectiveCallableDefinition`.

Do not recover it later from `impl_id.local` + AST.

## Wire

Add `callable_definitions` to `SemanticWorldV3`. Keep the outer Telosc
container schema at 2; the semantic-world section and payload use schema 3.
Schema-2 semantic-world payloads are intentionally rejected because they do not
carry the accepted callable-definition authority required by PATCH2.

Encode:

```text
callable
origin
source_member_index
complete semantic signature
body provenance
inherent body context
```

Covering maps sorted and inverse-validated.

## Import

Delete signature-derived definition fabrication.

Build exact definitions only from these records.

Build `callable_definitions_by_module` from imported definitions.

## Core

Replace current `snapshot.sources` delegation tests with `definition.body_provenance`.

## Tests

Required:

- CV-01–CV-06;
- `QueryKey::CallableDefinition` product/input fingerprint parity;
- generated delegation without callable analysis still lowers;
- same signature with different origin/member/context produces different query fingerprint.

## Guard

Do not remove the broad static/derived owner guard yet.

Callable-definition-specific manifest parity may be marked implemented only after tests pass.

## STOP

If `ConformanceWitness` definitions are not currently published by any source path, do not fabricate fixtures implying they are. Find the actual source owner path and test that path; if none exists but compiler expects it, consult.

---

# T2 — Exact trait surface transport and trait-default provenance

## Purpose

Replace lossy trait rows and remove trait-default AST scanning.

## Files

```text
phalcom-semantic/src/traits.rs
artifact/roots.rs
artifact/export.rs
artifact/import.rs
compiler-artifact/semantic/world.rs
core/modules/semantic_lowering.rs
```

## Source change

Add `source_member_index` to `TraitSurfaceMember`.

Change trait-building loops to enumerate source members.

For generated property getter/setter requirements, both use the property's source member index.

## Wire

Encode:

```text
header generic signature + source
members complete
associated type requirements complete
    full diagnostics, including callable-qualified explanation references and
    root-cause equivalence within the canonically ordered trait-surface family
source_member_index
```

Keep `super_traits` only as a derived integrity projection if format compatibility wants it.

## Import

No:

```text
KindId(0)
Public
default span
default_source None
```

unless they are exact source facts.

Finalize requirement layout after member insertion.

## Core

Delete trait AST scan for detached defaults.

Iterate trait surface members directly.

## Tests

CV-07–CV-09 and source-denied trait-default lowering.

Test property-generated getter/setter sharing one source member index.

---

# T3 — Exact declaration surfaces and conditional inherent dispatch

## Purpose

Make user overlay dispatch against imported Universe semantically identical.

## Files

```text
phalcom-semantic/src/surface.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/impls.rs
artifact/export.rs
artifact/import.rs
artifact/roots.rs
compiler-artifact/semantic/world.rs
```

## Wire

Replace callable-ID-only rows with full final dispatch rows.

Add conditional inherent set records.

Each callable row carries its final `TypeKnowledge` and visibility. Its
dispatch signature is reconstructed only from the canonical callable signature
projection after export verifies that the retained dispatch signature agrees
with that projection (apart from the separately transported final return
knowledge). Field rows carry final knowledge and visibility. Declared field
contract/mutability/source are optional together because some canonical native
surface fields have no `FieldSemanticSignature`; do not fabricate those facts.
Conditional member vector order is explicit and dense so reconstruction keeps
candidate order.

Root:

```text
dispatch TypeKnowledge
parameter knowledge
generic signature
conditional domain head types/constraints/signatures
```

## Import

Construct exact `DeclarationSurface`.

Then derive `SurfaceDispatchResolver`.

Register conditional sets/domains.

Register each surface owner's type form from the same imported
`DeclarationTypeTable` shell used by source publication.

Do not use `TypeId(0)`.

## Differential tests

From a source snapshot and fresh imported snapshot compare:

```text
resolve_dispatch_on_owner
callable signature
return knowledge
visibility
field knowledge
conditional member candidates
inherent_impl_domain lookup
```

Use:

- ordinary instance member;
- class member;
- generic member;
- field;
- conditional inherent impl;
- exact-case conditional target if present in fixtures.

## Acceptance

CV-10–CV-12.

---

# T4 — Complete witness-plan and associated-type-plan transport

## Purpose

Replace proof-state collapse with exact conformance authority.

## Files

```text
phalcom-semantic/src/impls.rs
phalcom-semantic/src/traits.rs
artifact/roots.rs
artifact/export.rs
artifact/import.rs
compiler-artifact/semantic/world.rs
```

## First refactor

Add a final-parts constructor for `ConformanceAssociatedTypePlan`.

Do not permit artifact importer to mutate a fingerprinted `empty()` object.

Recommended:

```rust
ConformanceAssociatedTypePlan::from_parts(
    impl_id,
    bindings,
    failures,
    diagnostics,
)
```

It computes fingerprint after content is installed.

## Wire completeness

Implement:

- requirement views;
- every requirement selection variant;
- conditional impl/applicability;
- conditional inherent pending/fallback;
- associated binding/failure/diagnostic;
- invalid explicit members;
- complete completeness payload;
- plan diagnostics.

## Validation

Reject unknown tags before materialization.

Validate all requirement owners against imported trait surface.

Validate fallback structure is finite/bounded.

## Fingerprint

Recompute associated plan, then witness plan.

Never accept copied fingerprint as primary.

## Tests

CV-13–CV-18.

One test must reproduce the old bug:

```text
nonempty associated bindings
old empty fingerprint != correct final fingerprint
imported fingerprint == source final fingerprint
```

---

# T5 — Derived conformance rules and deterministic trait dispatch

## Purpose

Restore generic/derived proof authority and its derived lookup index.

## Files

```text
phalcom-semantic/src/derived_conformance.rs
phalcom-semantic/src/trait_dispatch.rs
artifact/export.rs
artifact/import.rs
artifact/roots.rs
compiler-artifact/semantic/world.rs
```

## Derived rules

Add direct wire records using shared witness nested codecs.

Import with `DerivedConformanceRule::new`.

Build `DerivedConformanceRuleIndex`.

## Trait dispatch

After conformance heads and trait surfaces are complete:

```rust
TraitDispatchIndex::build(...)
```

Compare against source.

Do not serialize buckets.

## Tests

CV-19–CV-20.

Add one generic user-resolution test where derived conformance is required and source syntax is unavailable.

---

# T6 — Static case catalogs and signatures

## Purpose

Restore the small but required static-domain primary owner families first.

## Files

```text
phalcom-semantic/src/static_case.rs
artifact/roots.rs
artifact/export.rs
artifact/import.rs
compiler-artifact/semantic/world.rs
```

## Implement

Direct DTOs described in §6.13.

Install into real snapshot maps/tables.

## Tests

CV-21–CV-22.

Include duplicate case, missing catalog, bad arity/kind root malformed tests.

---

# T7 — Canonical static program and checked application authority

## Purpose

Replace detached operational templates with a real source-independent static compiler program.

## Files

```text
phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/checker/static_body.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/artifact/operational.rs
phalcom-semantic/src/artifact/export.rs
phalcom-semantic/src/artifact/import.rs
phalcom-semantic/src/artifact/roots.rs
phalcom-compiler-artifact/src/semantic/operational.rs
phalcom-compiler-artifact/src/semantic/world.rs
phalcom-semantic/src/db/fingerprint.rs
```

## Subtask T7.1 — Canonical program owner

Introduce the canonical static program model.

Migrate post-check publication so source-built static definition products always have a canonical program for supported bodies.

Source-only checked body remains optional auxiliary authority for source recomputation/diagnostics, not the serialized executable truth.

## Subtask T7.2 — Application rows

Create body-local checked application rows.

Encode full checked signature.

No source `ExpressionId`.

## Subtask T7.3 — Operational references

Change operational application node to body-local application ordinal.

Update evaluator to resolve row.

## Subtask T7.4 — Canonicalization correctness

Repair:

- TypeForm exact application matching;
- nested KindedExpression lowering;
- evaluation order;
- multiple/nested application cases.

Delete `first()` shortcut.

## Subtask T7.5 — Fingerprint

Make static definition **product** fingerprint canonical-program based.

Keep source representation in source/input fingerprint.

## Tests

CV-23–CV-27.

Required fixtures:

```text
TypeForm static body
Expression static body
Block static body
two applications in one body
nested application
Kinded argument
KindedExpression argument
Value argument
two bodies both application ordinal 0
```

Source program and imported program must evaluate to same outcome.

## STOP

If canonical Universe uses a checked static construct not representable operationally after this bounded work, consult. Do not serialize AST.

---

# T8 — Static definition owner transport

## Purpose

Attach canonical static programs to real `StaticDefinitionProduct`s.

## Files

```text
static_definition.rs
artifact/export.rs
artifact/import.rs
compiler-artifact/semantic/world.rs
snapshot.rs
```

## Wire

Definition record:

```text
id
signature
source range
alias info
canonical program
accepted diagnostics policy
```

## Export

Iterate every accepted `snapshot.static_definitions` owner.

Require canonical program.

No filter:

```text
if root is Operational
```

because canonicalization happened earlier.

## Import

Construct real products and insert into `StaticDefinitionTable`.

Eliminate orphan final `operational_templates`.

## Test

After isolated semantic-world import:

```text
snapshot.static_definitions.get(id).is_some()
product canonical program evaluates
no source unit available
```

CV-36 must pass.

---

# T9 — Static constant persistent authority

## Purpose

Restore static constants without AST or re-evaluation.

## Files

```text
static_constant.rs
static_constant_demand.rs
db/query.rs
db/fingerprint.rs
artifact/export.rs
artifact/import.rs
artifact/roots.rs
compiler-artifact/semantic/world.rs
core/modules/semantic_lowering.rs
```

## Subtask T9.1 — Persistent authority accessor

Refactor consumers away from directly requiring `product.checked` for:

```text
input_fingerprint
dependencies
static_constant_reads
record_key_static_constant_reads
result_type
```

Introduce a source-independent authority summary.

Source build populates it from `CheckedStaticConstantInitializer`.

## Subtask T9.2 — Universe outcome census

Before designing residual transport, inspect every canonical Universe constant outcome.

Record counts by variant.

### If all are Reduced

Adopt P7 canonical profile:

```text
canonical artifact requires Reduced
```

and add explicit preflight rejection for other variants.

### If not

Implement stable payload codecs for already source-independent variants.

STOP if Symbolic requires new residual graph persistence not already approved.

## Subtask T9.3 — Wire/import

Persist:

```text
stable constant ID
ranges
persistent authority summary
outcome
static value references
lkg value
diagnostics per policy
```

Relocate static values.

Construct real `StaticConstantProduct`.

No evaluation.

## Tests

CV-28–CV-29.

Source/import:

```text
is_successful
value
materialization_descriptor
product fingerprint
dependency set
query product behavior
```

must agree.

---

# T10 — Final ModuleStaticBindingIndex transport

## Purpose

Persist module-owned checker facts used by user overlays and static checking.

## Files

```text
phalcom-semantic/src/module_bindings.rs
artifact/export.rs
artifact/import.rs
artifact/roots.rs
compiler-artifact/semantic/world.rs
snapshot.rs
session.rs if reader integration requires it
```

## Dependency

Use PATCH1 portable binding record/reference.

Do not define another BindingState wire model.

## Wire

As §6.20.

Visible alias names are explicit.

## Import

Resolve:

```text
(source_body, source_binding_ordinal)
    -> imported callable analysis
    -> fresh BindingId
```

Validate imported `BindingState` matches.

Build `ModuleStaticBindingIndex` directly.

No `Program`, no `from_module_analysis`.

## Tests

CV-30–CV-31.

Include:

- local top-level runtime binding;
- static constant binding;
- imported alias with different local visible name;
- same numeric BindingId under two module bodies;
- missing source body;
- invalid binding ordinal.

---

# T11 — Assemble exact owner graph and deterministic derived indexes

## Purpose

Make isolated semantic import produce all PATCH2 owners in their true snapshot fields.

## Files

```text
artifact/import.rs
snapshot.rs
dispatch.rs
trait_dispatch.rs
presentation.rs only if shared interfaces require it
```

## Required construction order

Conceptually:

```text
1 type/static relocation
2 structural declarations/aliases/data/enums
3 signatures/fields/constructor initialization
4 exact trait headers/surfaces
5 exact declaration surfaces
6 exact conformance heads
7 exact accepted callable definitions
8 PATCH1 callable analyses
9 witness plans
10 derived conformance rules
11 static case catalogs/signatures
12 static definition products
13 static constant products
14 module static binding indexes

then derive:
15 callable_definitions_by_module
16 trait requirement layouts
17 SurfaceDispatchResolver
18 TraitDispatchIndex
19 PATCH1 formal projection contribution
```

Do not mark snapshot Complete here solely because these families exist; PATCH6 owns final promotion.

## Remove false defaults

Search and delete production occurrences of the known placeholder patterns from the artifact importer:

```text
PrimaryDeclaration / 0 / None fabricated from signature
TypeKnowledge::assumed(TypeId(0), ...)
KindId(0) for associated domain
visibility = Public as import default
default_source = None as import fallback
requirement_views = empty
invalid_explicit_members = false as import fallback
associated plan = empty then mutate
derived_conformance_rules = empty
trait_dispatch_index = default
static_case_* = empty
static_definitions = empty
static_constants = empty
module_static_bindings = empty
```

A legitimate exact source value may still equal zero/Public/None; it must come from wire authority, not a fallback.

## Tests

One isolated complete semantic-world fixture that has at least one nontrivial owner in every PATCH2 family.

Assert field values, not only nonemptiness.

---

# T12 — Source-independent compiler consumer differential

## Purpose

Prove restored authority is actually sufficient for compiler readers.

## Files

```text
phalcom-core/src/modules/semantic_lowering.rs
focused core semantic-lowering tests
```

## Imported canonical side

Remove canonical `snapshot.sources`.

Keep local user source where the user module genuinely owns syntax.

Compare source-built and imported canonical semantics for:

```text
inherent impl lowering
generated delegation target
conditional inherent resolution
trait default detached method
conformance witness detached method
trait requirement slots
derived conformance lookup
static constant projection
module binding visibility
```

Instrumentation or a source-denial test must prove canonical source is not consulted.

## Allowed source use

User module syntax remains allowed.

Do not rewrite `project_anonymous_products` solely because it reads the current user module.

## Acceptance

CV-32.

---

# T13 — Root/validator closure

## Purpose

Make the new primary owners structurally safe before removing coverage guards.

## Files

```text
artifact/roots.rs
compiler-artifact/semantic/world.rs
artifact/export.rs
artifact/import.rs
```

## Root census

Cover all new:

```text
TypeId
KindId
StaticDomainId
RecordRowId
TypeParameterId
GenericSignature
TypeKnowledge
inherent domain
associated binding
dispatch signature
static definition signature
static program terms
static case kinds
module BindingState
```

No projection discovers a new root after graph closure.

## Validator complexity

Pre-index owner IDs where needed.

Avoid repeated linear scans over large tables.

Malformed wire tests:

```text
duplicate definition
wrong impl owner
noninverse covering maps
trait member wrong owner
bad visibility tag
bad associated domain
witness requirement absent
fallback cycle/depth overflow
conditional member wrong target
static application dangling ordinal
static signature mismatch
module binding bad owner/ordinal
unknown enum tag
```

---

# T14 — Remove only PATCH2 coverage guards

## Purpose

Enable exact remaining semantic-owner export/import while preserving independent barriers.

## Preconditions

All PATCH2 gates pass.

## Current broad guard

The current exporter rejects when any of:

```text
static_case_catalogs
static_case_signatures
static_definitions
static_constants
module_static_bindings
derived_conformance_rules
trait_dispatch_index
```

is nonempty.

Do not simply delete the entire compound expression.

Replace it with family-specific completeness checks where useful.

Trait dispatch is now Derived, so its presence is validated by rebuild parity rather than direct wire requirement.

Witness-specific guard clauses are removed only after exact witness proof transport passes.

## Keep

- global semantic validation allocation `UnaccountedBoundary`;
- global semantic import allocation `UnaccountedBoundary`;
- any unsupported StaticApplication/constant outcome guard whose prerequisite did not pass;
- PATCH4 query blockers;
- PATCH6 promotion blockers.

## Frontier test

Run the narrow artifact builder far enough to prove PATCH2 owner coverage is no longer the failure.

Assert the **next expected independent typed/frontier error**.

Do not use `is_err()`.

---

# 15. Verification gates

# G1 — Accepted definition gate

Proof:

```text
Primary
Inherent
ConformanceWitness
covering context
definition query fingerprint
module index
generated delegation
```

No source lookup.

---

# G2 — Trait/dispatch gate

Proof:

```text
trait headers
trait members
associated domains
visibility
default provenance
requirement layout
final declaration dispatch signature/knowledge
conditional inherent lookup
```

No `TypeId(0)` placeholder.

---

# G3 — Witness/derived conformance gate

Proof:

```text
all selection variants
all completeness variants
associated failure payload
nonempty associated fingerprint
derived rule
trait dispatch rebuild parity
```

---

# G4 — Static case/static program gate

Proof:

```text
catalog/signature parity
all accepted source body forms canonicalize
nested KindedExpression
body-qualified application ordinal
source/import evaluator equality
static product fingerprint parity
```

---

# G5 — Static constants/module binding gate

Proof:

```text
no re-evaluation
materialization descriptor parity
dependency/fingerprint parity
final module binding index parity
visible alias parity
binding owner resolution via PATCH1 ordinal
```

---

# G6 — Source-independent lowering gate

Proof with canonical sources absent:

```text
generated delegation
trait default detached method
conditional dispatch
derived conformance
static constant projection
```

User source remains present only for user-owned module.

---

# G7 — Owner-graph structural gate

One rich semantic world fixture.

Every PATCH2 family exact.

Malformed cross-owner rows rejected.

---

# G8 — Guard frontier

PATCH2 guards removed.

Global PATCH3 allocation guard still blocks full load if not yet completed.

Test asserts exact frontier.

---

# 16. Final acceptance ledger

PATCH2 cannot close unless:

```text
[ ] callable definitions are explicit wire owners
[ ] definition origin exact
[ ] inherent body context exact
[ ] covering maps validated
[ ] generated delegation provenance exact
[ ] callable_definitions_by_module derives from definitions

[ ] trait header exact
[ ] trait member semantic signatures exact
[ ] visibility exact
[ ] associated domains exact
[ ] default provenance exact
[ ] source member ordinals exact
[ ] supertrait redundancy validated

[ ] declaration surface callable knowledge exact
[ ] declaration surface field knowledge exact
[ ] conditional inherent sets exact
[ ] SurfaceDispatchResolver rebuild parity

[ ] witness requirement_views exact
[ ] every selection variant exact
[ ] completeness payload exact
[ ] associated bindings/failures exact
[ ] associated fingerprint recomputed
[ ] invalid_explicit_members exact
[ ] witness diagnostics exact

[ ] derived conformance rules wire/import exact
[ ] TraitDispatchIndex rebuilt and parity-tested

[ ] static case catalogs exact
[ ] static case signatures exact

[ ] canonical static program exists
[ ] every real Universe static definition canonicalizes
[ ] StaticApplication identity = body + ordinal
[ ] operational node references checked row
[ ] KindedExpression nested application works
[ ] no first-application shortcut
[ ] imported StaticDefinitionProduct owns program
[ ] no orphan operational template
[ ] static definition product fingerprint source/import equivalent

[x] static constant outcome census recorded (11/11 canonical constants Reduced; 2026-10-16)
[ ] canonical constant authority source-independent
[ ] no constant re-evaluation on load
[ ] materialization descriptor parity

[ ] final ModuleStaticBindingIndex persisted
[ ] source binding resolved through PATCH1 owner-qualified ordinal
[ ] visible names/aliases exact
[ ] no AST rebuild

[ ] canonical core lowering no longer reads Universe AST for delegation/default discovery
[ ] all new roots closed
[ ] all cross-owner validators pass
[ ] PATCH2 family guards removed
[ ] global PATCH3 allocation guards retained
```

---

# 17. Performance/resource requirements

PATCH2 adds substantial semantic data. Measure, do not guess.

Record:

```text
callable definition count/bytes
trait member count/bytes
conditional inherent member count/bytes
witness plan bytes
derived conformance bytes
static catalog/signature bytes
canonical static program bytes
static application row bytes
static constant bytes
module binding bytes
```

Check duplication.

Red flags:

- complete semantic signatures copied many times unnecessarily;
- trait dispatch serialized despite deterministic rebuild;
- AST serialized;
- both source and imported maps retained in loaded owner;
- static operational tree plus a second independent target tree;
- BTree/Hash shadow structures built only for API compatibility.

PATCH3 will certify requested-byte accounting, but PATCH2 should not knowingly make it impossible.

---

# 18. Checkpoint bookkeeping

At start:

```text
active corrective patch: P7.PATCH2
starting revision
PATCH1 dependency status
known baseline failures
```

Meaningful intermediate events:

1. accepted definition/body provenance stable;
2. exact trait/dispatch contract stable;
3. witness/derived conformance stable;
4. canonical static program accepted;
5. static constants/module binding contract stable;
6. source-independent lowering gate passes;
7. PATCH2 coverage guards removed.

At completion:

```yaml
PATCH2:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED
```

only if the focused evidence genuinely passed.

P7 normally remains:

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
```

---

# 19. Walkthrough deliverable

Create according to the live P7 record layout:

```text
PERF003.C2.P7.PATCH2-walkthrough.md
```

Required content:

1. start/final revisions;
2. final semantic authority graph;
3. accepted definition schema;
4. body provenance design;
5. trait/dispatch schema;
6. conditional inherent authority;
7. witness/associated/derived conformance design;
8. trait-dispatch derivation proof;
9. static case schema;
10. canonical static program design;
11. checked application identity;
12. operational canonicalization repairs;
13. static constant persistence profile;
14. module static binding design;
15. source-dependent core reads removed;
16. roots/validators;
17. guards removed/retained;
18. tests actually run;
19. tests deferred;
20. size/resource observations;
21. residual blockers for PATCH3+.

---

# 20. Handoff deliverable

Create:

```text
PERF003.C2.P7.PATCH2-handoff.md
```

State exactly:

```text
revision
dirty/clean status
stable wire types
stable native owner changes
derived-index rules
static program interface
static constant outcome profile
module binding reference model
remaining global allocation guard
remaining query/container/promotion work
known baseline failures
first commands for PATCH3
```

Include these explicit do-not-redesign statements:

```text
EffectiveCallableDefinition is explicit authority.
Do not infer definitions from signatures.
GenericSignature is supertrait authority.
TraitDispatchIndex is derived, not a second wire authority.
Conditional inherent sets are required resolver input.
CheckedStaticApplication belongs to StaticBodyId + ordinal.
Do not serialize AST.
Do not re-evaluate static constants on artifact load.
Do not rebuild ModuleStaticBindingIndex from source.
Do not remove global allocation UnaccountedBoundary until PATCH3.
```

---

# 21. Completion truth table

| Claim | PATCH2 can establish? |
|---|---|
| remaining primary semantic owners wire-complete | yes |
| source/import semantic owner parity | yes |
| source-independent delegation/default lowering | yes |
| static compiler program source-independent | yes |
| witness/derived conformance exact | yes |
| trait dispatch deterministic rebuild | yes |
| exact whole-loader allocation | no — PATCH3 |
| complete query image | no — PATCH4 |
| canonical `.telosc` bytes | no — PATCH5 |
| final loaded canonical parent promoted | no — PATCH6 |
| package/runtime P7 closure | no — PATCH7 |
| P8 takeover | no |

---

# Appendix A — Current false-authority deletion checklist

Search the artifact importer and eliminate these *as fallbacks*:

```text
origin: PrimaryDeclaration
source_member_index: 0
inherent_body_context: None

TypeKnowledge::assumed(TypeId(0), EvidenceOrigin::Flow)

TypeMemberDomain::Kind(KindId(0))
visibility: Public
source: default span
default_source: None

requirement_views: BTreeMap::new()
invalid_explicit_members: false
conditional_impl: None
applicability: None
associated failures: []
plan diagnostics: []

ConformanceAssociatedTypePlan::empty(...)
then mutating bindings

DerivedConformanceRuleIndex::default()
TraitDispatchIndex::default()

StaticDefinitionTable::new()
StaticConstantTable::new()
empty static_case_catalogs
empty static_case_signatures
empty module_static_bindings

detached operational_templates not attached to owner
```

Exact source data may legitimately contain `None`, `Public`, zero, or empty collections. The violation is **inventing** those values because wire authority is absent.

---

# Appendix B — Accepted definition wire checklist

```text
callable
origin
source_member_index
full signature

body provenance:
    ordinary
    generated delegation target field

inherent body context:
    generic signature
    covering impl-to-decl
    covering decl-to-impl
    source_target_head

roots:
    all signature types
    all generic parameters/constraints
    source_target_head
```

---

# Appendix C — Trait/dispatch checklist

Trait:

```text
declaration
generic signature
header source

member:
    requirement
    callable
    semantic signature
    visibility
    source
    source_member_index
    default_present
    default_source

associated:
    requirement
    name
    TypeMemberDomain
    source
```

Dispatch surface:

```text
callable identity
dispatch signature
TypeKnowledge
visibility
field identity
field TypeKnowledge
field visibility
```

Conditional:

```text
target
impl
domain
conditional flag
callable
signature template
visibility
source member
requirement flag
```

---

# Appendix D — Witness checklist

```text
impl_id
source_module
target_template
trait_ref_template
generic_signature

requirement_views:
    requirement
    callable
    signature
    visibility
    default_callable

requirements:
    ConformanceCallable
    InherentCallable + conditional_impl + applicability
    ConditionalInherent + candidate + pending + fallback
    DataComponent + specialized_type
    TraitDefault

associated plan:
    bindings
    failures
    diagnostics
    recomputed fingerprint

invalid_explicit_members

completeness:
    Complete
    Incomplete + failures
    Unknown + reason
    Blocked + reason
    Dynamic + obligation
    Cancelled
    BudgetExceeded + report
    InternalFailure + message

diagnostics
recomputed witness fingerprint
```

---

# Appendix E — Static program checklist

Canonical program:

```text
owner StaticBodyId
parameter_count
local_count
operational root
checked applications[]
parameter bindings
kinded parameters
constant bindings/reads
dependencies
result
input fingerprint
module binding fingerprint
```

Application:

```text
ordinal
source range
target
checked static signature
argument categories and parameter indexes
```

Operational application:

```text
application ordinal
child operational arguments
```

Must not contain portable source `ExpressionId`.

---

# Appendix F — Static constant checklist

Persistent authority:

```text
id
declaration range
initializer range
input fingerprint
semantic dependencies
static constant reads
record-key static reads
result type
outcome
value
lkg value
diagnostics
```

Source-only auxiliary:

```text
AST initializer
full CheckedStaticExpression
```

Imported canonical product must not require source-only auxiliary state.

---

# Appendix G — Module static binding checklist

```text
module

fact:
    SymbolId
    stable source CallableId
    PATCH1 binding ordinal
    phase
    complete BindingState
    declaration range
    initializer range

visible name -> stable SymbolId
```

Import validates binding ordinal against the imported source body.

---

# Appendix H — Forbidden shortcuts

Plan violations:

```text
infer accepted definitions from signatures
PrimaryDeclaration/0/None fallback
TypeId(0) dispatch knowledge
KindId(0) associated domain
force Public visibility
drop default source
ignore supertrait/generic inconsistency
drop witness requirement views
collapse completeness into u8 + generic payload
empty conditional impl/applicability
mutate an empty associated plan after fingerprinting
serialize TraitDispatchIndex redundantly
leave derived conformance rules empty

serialize AST into .telosc
copy StaticApplication into CallableAnalysis
identify StaticApplication by target only
pick first static application
drop checked signature
fabricate ExpressionId for imported static program
ignore KindedExpression
keep operational templates detached

re-evaluate canonical static constants on load
rerun ModuleStaticBindingIndex::from_module_analysis
serialize raw body-local BindingId
use source name/range matching

read canonical Universe source for delegation/default lowering
remove global allocation guards
mark P7 complete
```

---

# Appendix I — Recommended execution sequence

```text
T0  live truth / census
 ↓
T1  accepted definitions + body provenance
 ↓
T2  trait surfaces + default provenance
 ↓
T3  declaration/conditional dispatch
 ↓
T4  witness/associated proof closure
 ↓
T5  derived rules + trait dispatch rebuild
 ↓
T6  static catalogs/signatures
 ↓
T7  canonical static program + applications
 ↓
T8  real static definition products
 ↓
T9  static constant authority
 ↓
T10 module static binding index
 ↓
T11 complete native owner graph
 ↓
T12 source-independent core differential
 ↓
T13 roots/validators
 ↓
T14 family-specific guard removal
```

---

# Appendix J — Plan self-review

## Architectural completeness

- [x] every audit-identified fabricated semantic fact has an owning correction;
- [x] newly discovered conditional inherent resolver authority is included;
- [x] primary vs derived ownership is explicit;
- [x] trait dispatch is not duplicated;
- [x] static application ownership matches live producer;
- [x] static program avoids AST persistence;
- [x] static constants avoid source re-evaluation;
- [x] module binding identity uses PATCH1 owner-local contract;
- [x] compiler source dependencies are explicitly removed.

## Luna executability

- [x] fixed/flexible/verify-first choices separated;
- [x] STOP conditions objective;
- [x] tasks are dependency-ordered;
- [x] each owner family has focused acceptance;
- [x] broad tests explicitly deferred;
- [x] downstream guards remain valid intermediate states.

## Lifecycle truth

- [x] PATCH2 is subordinate to PERF003.C2.P7;
- [x] checkpoint remains single durable state authority;
- [x] walkthrough required;
- [x] handoff required;
- [x] PATCH2 completion does not imply P7 closure.
