# Phalcom ADT/GADT + Associated Lookup
## Part 06 — Core Integration, Migration, Reflection, Tooling Completion, and Final Verification

**Status:** Technical specification / convergence architecture contract  
**Series:** ADT/GADT + Associated Lookup, Part 6 of 6  
**Repository:** `aureat/phalcom-lang`  
**Planning repository branch inspected:** `feat/adts`  
**Planning repository commit inspected:** `26166385f9c1bf35f6e9eb969385fc8a162f2f56`  
**Commit subject:** `ci: apply and verify ordered match orchestration`  
**Verified on:** 2026-08-31  
**Intended repository path:** `docs/impl/adt-gadt-associated-lookup/part-6/06-core-integration-migration-reflection-tooling-final-verification-technical-spec.md`

---

# 1. Executive Summary

Parts 01–05 establish the language semantics and execution architecture for Phalcom ADTs, GADTs, associated variant families, exact-case types, and exhaustive matching. Part 06 finishes the architecture by removing transitional exceptions and making those semantics authoritative everywhere else in the implementation.

Part 06 is defined by one convergence rule:

> **There is one semantic model of an enum case. Core/native declarations, user declarations, reflection, source tooling, compiler lowering, and runtime metadata all project from that model instead of recreating it.**

The completed architecture is:

```text
source/core declaration
    ↓
canonical declaration products
    DeclarationId
    EnumInfo
    VariantId / VariantFamilyId / VariantFieldId
    VariantConstructorId
    CaseTypeEnvironment
    ↓
canonical type world
    Nominal / Applied / ExactCase / Family / Union
    ↓
associated + match proof products
    AssociatedResolution
    MatchResolution
    PatternSpace / CoverageWitness
    ↓                 ↓
source index          backend projection
semantic targets      executable identities / payload slots
    ↓                 ↓
editor tooling        runtime registry / bytecode
                      ↓
                reflection metadata projection
```

The primary semantic migration is core `Option`, `Result`, and `Ordering`. They become canonical `@native enum` declarations and publish exactly the same semantic products as user enums. `@native` changes implementation provenance and representation, never type/match/associated semantics.

`Option` keeps its current immediate representation. `Result` may use ordinary ADT runtime representation in Part 06; any specialized compact representation is deferred to Part 07 unless already required by the runtime. `Bool` remains a primitive finite domain and does not acquire artificial enum variant identities.

Part 06 also finalizes exact-case canonicalization and reflection. A specialized case such as the exact `Some` case of `Option<Int>` is a real canonical semantic type, not a display-only view, while remaining distinct from both the single source `VariantId` and the one erased runtime variant descriptor. This supports future/static typed multiple dispatch without requiring per-instance runtime generic tokens.

Reflection exposes dedicated semantic metaobjects. `::` remains associated lookup and never becomes reflection syntax. Runtime `.class` returns the real per-case behavior class under the ratified current policy, but that class is not the semantic `ExactCase` and does not replace `VariantId` for matching/source identity.

The source index is extended with family and variant-field identities. LSP features consume these compiler-owned products for definition, hover, semantic highlighting, rename, residual-space completion, “add missing cases,” and “generate match.” The LSP must not implement pattern/family/GADT semantics itself.

Part 06 concludes with cross-module, incremental, persistence, fuzz/robustness, documentation, and architectural deletion passes. Performance measurement and match/backend optimization are explicitly handed to Part 07.

---

# 2. Normative Inputs and Precedence

Part 06 consumes these inputs:

```text
Part 05.2
    executable pattern projection
    match lowering
    shared pattern runtime engine
    core Option compatibility bridge
    legacy ADT matcher retirement

Part 05.1
    formal pattern semantics
    exact/family pattern candidate identity
    PatternSpace
    GADT match proof
    exhaustiveness / CoverageWitness
    source identity preservation

Part 04
    runtime enum/variant registry
    runtime case representation
    case behavior classes
    semantic-to-lowering projection
    IsVariant / GetVariantPayload

Part 03 / 03.5
    associated resolution
    exact/family value denotation
    structural family type
    generic specialization

Part 02
    enum/variant declaration products
    ExactCase
    CaseTypeEnvironment
    visibility axes
```

Precedence is:

1. ratified project/user decisions;
2. this Part-06 specification for migration/reflection/tooling/completion;
3. Part 05.2 for executable pattern authority;
4. Part 05.1 for pattern proof/exhaustiveness meaning;
5. Part 04 for runtime identity and representation boundaries;
6. Parts 03/03.5 for associated-family semantics;
7. Part 02 for declaration/exact-case/GADT identity;
8. current implementation for exact file/symbol names.

Part 06 may delete transitional code from earlier parts. It may not reinterpret their language semantics.

---

# 3. Planning Baseline and Mandatory Implementation Preflight

The connected repository was inspected at:

```text
branch: feat/adts
HEAD:   26166385f9c1bf35f6e9eb969385fc8a162f2f56
subject: ci: apply and verify ordered match orchestration
```

The user's Part-05.2 technical specification and implementation plan are normative inputs even if the implementation continues to move after this planning snapshot.

Before Part-06 implementation begins, the agent must reconcile the final Part-05.2 tree and record:

```text
actual final 05.2 HEAD
actual MatchResolution / executable pattern product names
actual core Option compatibility bridge names
actual runtime ADT registry/case primitive names
actual source index associated/pattern attachment state
actual LSP retirement/single-semantic-world state
actual core source declaration files and Result/Ordering status
```

If the landed tree has structurally equivalent renamed symbols, mechanically adapt this document. Do not create duplicate systems to match planning names.

---

# 4. Existing Repository Seams Part 06 Must Reuse

## 4.1 Enum attributes already support `@native`

`phalcom-ast/src/ast.rs` already provides:

```rust
pub struct EnumDef {
    ...
    pub attributes: Vec<Attribute>,
    ...
}
```

and `BuiltinAttr::Native` / `@native` already exists.

Therefore Part 06 does not introduce a new annotation grammar merely to mark core ADTs.

## 4.2 Canonical exact cases already exist

`phalcom-semantic/src/types/store.rs` already contains:

```rust
TypeData::Applied { ... }
TypeData::ExactCase { variant: VariantTypeId, enum_type: TypeId }
TypeData::Family(FamilyTypeId)
```

and `TypeStore::exact_case_type` validates ownership and hash-conses the result through the canonical store.

Part 06 strengthens how these exact cases are reflected, presented, persisted, and used by tooling/typed dispatch; it does not replace the representation.

## 4.3 Stable variant identities already exist

`phalcom-semantic/src/identity.rs` already publishes:

```text
VariantId
VariantFamilyId
VariantFieldId
VariantConstructorId
```

`SemanticTargetId` currently includes `Variant(VariantId)` but not family/field targets. Part 06 extends that target universe.

## 4.4 Source index is compiler-owned

`phalcom-semantic/src/source_index/*` explicitly owns source identity and avoids LSP/protocol types. This is the correct location for pattern/family/field attachment.

## 4.5 Presentation is protocol-neutral

`phalcom-semantic/src/presentation.rs::TypePresenter` is already the canonical pure type rendering seam. Part 06 extends this rather than formatting exact cases independently in LSP diagnostics/hover.

## 4.6 Option is physically immediate today

`phalcom-core/src/value/option.rs` stores nested `Some` depth in `Value` metadata and exposes `option_case()` to peel exactly one layer. It does not allocate an ordinary ADT wrapper.

Part 06 preserves this representation.

## 4.7 Runtime ADT case interface already exists

`phalcom-core/src/vm/adt.rs` already owns:

```text
register enum/variant runtime descriptors
runtime_variant_of
value_is_variant
case_payload_len
case_payload_at
case_behavior_class
```

This is the general runtime interface native enum representations must implement.

## 4.8 Semantic-to-codegen projection already exists

`phalcom-core/src/modules/semantic_lowering.rs` already projects formal `SemanticSnapshot` products into immutable backend structures. Runtime reflection metadata should follow the same authority direction.

---

# 5. Ratified Decisions

## 5.1 Native core ADTs

The canonical intended source surface is:

```phalcom
@native
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}

@native
enum Result<T, E> {
    @variant Ok(_ value: T)
    @variant Error(_ error: E)
}

@native
enum Ordering {
    @variant Less
    @variant Equal
    @variant Greater
}
```

Exact names/legacy compatibility aliases for `Ordering` must be reconciled against the current core library before implementation.

Normative meaning of `@native`:

```text
YES:
    implementation supplied/bound by compiler/runtime
    representation may differ from general source enum
    documentation exposes that implementation provenance

NO:
    alternate EnumInfo construction
    alternate VariantId namespace
    special pattern semantics
    special exhaustiveness semantics
    alternate associated lookup
    alternate reflection identity
```

## 5.2 Primitive non-ADT core types

`Bool` remains primitive. Part 06 may continue to use finite-domain reasoning for boolean patterns/control flow without manufacturing `Bool::True` / `Bool::False` variants.

`Unit`/private `Nil` likewise remain outside the enum migration.

## 5.3 ExactCase status

`ExactCase` is a canonical semantic type.

Part 06 does not ratify source annotation syntax for it. The compiler may present exact-case types through a canonical rendering and reflection object even if users cannot yet write that spelling as a type annotation.

## 5.4 Formal contract vs observed proof

Formal declaration type and current type evidence remain separate. This specification forbids code that “simplifies” a binding's declared `Option<T>` contract into its current exact variant.

## 5.5 `.class`

The runtime case behavior class is reflectable and is returned by `.class` under the current decision.

This supersedes only the “hidden from reflection” interpretation of earlier wording. It does **not** turn the case behavior class into a source declaration or semantic exact-case type.

## 5.6 Reflection syntax boundary

`::` never means reflection. Reflection is ordinary API over semantic metaobjects.

## 5.7 Canonical specialized exact cases

A specialized exact case is canonical in the type store and lazily reifiable as a type reflection object.

For ordinary generic ADTs:

```text
VariantId(Some(_))                       one declaration identity
ExactCase(Some(_), Option<Int>)           one canonical TypeId
ExactCase(Some(_), Option<String>)        another canonical TypeId
RuntimeVariantId(Some(_))                 one erased runtime case identity
runtime case behavior class               one erased runtime behavior class
```

For a GADT, the exact-case template is governed by the declared result specialization and `CaseTypeEnvironment`, not by blindly copying enum generic parameters.

## 5.8 Reflection visibility/capability

Reflection acquisition applies access rules. Exhaustiveness remains based on complete semantic truth. Once metadata/capability is legitimately acquired, it behaves as a stable value rather than rechecking lexical access on every operation.

## 5.9 Variant-family source identity and rename

Family-pattern occurrences target `VariantFamilyId` and retain exact candidates. Ordinary Rename on a family base renames the complete family, including exact member declarations and contextual references that resolve to that family.

## 5.10 Runtime reflection projection

Runtime reflection consumes projected metadata; the VM does not retain the entire semantic snapshot as its reflection database.

## 5.11 Persistence

Persistent metadata uses stable declaration/selector/type structure rather than runtime/store-local compact IDs.

## 5.12 Part-07 boundary

Performance measurement, representation optimization, and decision-tree/jump-table work are not Part 06.

---

# 6. Global Part-06 Invariants

## I06-1 — One enum semantic pipeline

After native implementation binding, user enums and core native enums enter the same declaration semantic pipeline.

## I06-2 — Native representation is downstream of semantics

The semantic analyzer does not ask whether an enum uses `NativeOption` representation to determine variant identity, type relation, associated lookup, or exhaustiveness.

## I06-3 — ExactCase remains canonical

All exact cases are obtained through canonical type-store construction and remain ordinary proper types.

## I06-4 — Specialization is not declaration identity

`Some<Int>` and `Some<String>` are distinct specialized semantic types but share one `VariantId` declaration.

## I06-5 — Specialization is not runtime identity

Generic exact-case specialization does not imply a per-specialization runtime case descriptor/class/token.

## I06-6 — Runtime class is not semantic type

A reflectable case behavior class cannot substitute for `ExactCase` or `VariantId` in semantic APIs.

## I06-7 — Reflection has no hidden semantic resolver

Reflection metadata is projected from canonical products. It does not discover variant/family meaning from runtime class names or source strings.

## I06-8 — LSP has no hidden pattern resolver

Hover/definition/rename/completion/code actions consume source-index and formal semantic products.

## I06-9 — Exhaustiveness sees inaccessible reachable cases

Reflection/completion visibility filtering must not remove inaccessible cases from the closed semantic universe.

## I06-10 — Persistent identity is reconstruction-safe

Serialized artifacts may be loaded into a new store/runtime world without depending on previous arena/VM IDs.

## I06-11 — Part 05 remains authoritative

Part 06 tooling may summarize, render, or generate from `PatternSpace`/`CoverageWitness`; it may not implement competing coverage/GADT logic.

---

# 7. Core Native Enum Model

## 7.1 Source declaration ownership

The preferred architecture is to keep canonical core enum declarations in the core source surface (or generated source-equivalent input) and bind them to native implementations through the existing source/native binding infrastructure.

Recommended flow:

```text
core source
    @native enum Option<T> ...
        ↓
phalcom-ast EnumDef
        ↓
core-surface extraction / native binding authorization
        ↓
normal enum semantic declaration analysis
        ↓
EnumInfo / VariantInfo / AssociatedSurface
```

Do not create:

```text
CoreOptionVariant
NativeVariantId
BuiltinExactCase
BuiltinPatternSpace
```

as competing semantic models.

## 7.2 `@native` validation

Part 06 should validate that `@native` enum declarations correspond to an authorized native implementation registered by core/runtime integration.

A user module should not silently gain privileged VM behavior merely by spelling:

```phalcom
@native enum Foo { ... }
```

The existing native binding policy should be extended to enums so that `@native` is accepted as implementation binding only in authorized core/native sources.

Diagnostics must distinguish:

```text
native attribute syntactically recognized
native implementation binding not authorized / not found
native declaration shape mismatches registered implementation
```

## 7.3 Native shape contract

A registered native implementation must validate its semantic declaration shape.

For Option:

```text
Some(_): constructor, exactly one payload field
None: singleton/getter-shaped case, zero payload
```

For Result:

```text
Ok(_): constructor, one payload
Error(_): constructor, one payload
```

For Ordering:

```text
three singleton cases
```

A mismatched core source declaration is an integration error, not permission for runtime to invent a new semantic shape.

---

# 8. Runtime Representation Strategy for Native ADTs

## 8.1 Required abstraction

The runtime registry needs enough information to route general case operations through a native representation implementation.

Recommended conceptual model:

```rust
pub enum RuntimeAdtRepresentation {
    General,
    NativeOption,
    NativeResult,
}
```

The exact Rust shape may instead be a strategy table or native case hooks. The normative requirement is the operation boundary, not the enum name.

Required operations:

```text
construct exact VariantId/RuntimeVariantId
load singleton
runtime_variant_of(value)
case_payload_len(value)
case_payload_at(value, slot)
case_behavior_class(value)
```

## 8.2 Option

The current immediate Option representation remains unchanged unless a correctness bug requires it.

`runtime_variant_of` must identify immediate `Some`/`None` through the runtime registration associated with canonical semantic Option variant identities, not hard-coded compiler source names.

`case_payload_at(Some(v), 0)` peels exactly one Option layer.

## 8.3 Result

Part 06 requires native semantic declaration integration. It does not require a new immediate Result value encoding.

If the runtime already has a Result-specific representation, adapt it to the same case interface. Otherwise the initial implementation may materialize Result through ordinary `AdtCaseObject` while retaining native declaration/behavior binding.

## 8.4 Ordering

If `Ordering` already has immediate/native values, map them through the same runtime case identity interface. If it is currently class/object based, preserve behavior while migrating semantic identity first.

## 8.5 `.class`

For every runtime representation strategy, `.class` must resolve to the case behavior class registered for the exact `RuntimeVariantId`.

Even immediate Option values therefore expose the appropriate runtime `Some`/`None` behavior class.

---

# 9. Core Migration Handoff from Part 05.2

Part 05.2 may contain a narrow compatibility bridge that gives immediate Option typed variant identity for `IsVariant`/`GetVariantPayload`.

Part 06 must replace its temporary semantic identity source with normal core enum declaration products.

The resulting runtime compatibility is no longer conceptually:

```text
special core Option semantic case table
    ↓
immediate value bridge
```

but:

```text
ordinary EnumInfo/VariantId for @native Option
    ↓
RuntimeAdtRegistry native representation binding
    ↓
immediate value bridge
```

A representation adapter may remain permanently. A semantic adapter must not.

---

# 10. Exact-Case Canonicalization and Specialization

## 10.1 Current canonical representation

The existing semantic representation is authoritative:

```rust
TypeData::ExactCase {
    variant: VariantTypeId,
    enum_type: TypeId,
}
```

`VariantTypeId` compactly references stable `VariantId` within the store. `enum_type` is the canonical specialized enum result.

## 10.2 Canonical identity rule

Within one `TypeStore`:

```text
same VariantId
+ same canonical enum_type TypeId
= same ExactCase TypeId
```

No independent “specialized variant interner” is required unless later profiling demonstrates a need; the existing `TypeData` interner already defines canonical identity.

## 10.3 Exact-case template

`VariantInfo.exact_case_template` and `CaseTypeEnvironment` remain declaration-side templates/evidence. Application/specialization resolves them into canonical exact-case types.

For an ordinary ADT:

```text
Some<T> conceptually produces
    ExactCase(Some(_), Option<T>)
```

For a GADT:

```text
Expr::Int case produces
    ExactCase(Int(_), Expr<Int>)
```

regardless of a generic scrutinee binder `T` before elimination.

## 10.4 AppliedType analogy

Part 06 should make exact-case reflection behave analogously to canonical `Applied` types:

```text
canonical type handle
    ↓
lazy reflection descriptor
    ↓
stable presentation / metadata
```

This is an analogy in reflection/canonicalization architecture, not a requirement to rewrite `ExactCase` into `TypeData::Applied`.

## 10.5 Typed dispatch

The dispatch/relation system may use `ExactCase` as a precise static type.

Example conceptual overload set:

```text
f(ExactCase(Some, Option<Int>))
f(ExactCase(Some, Option<String>))
f(Option<Int>)
```

Static selection can use the exact specialization when available.

At a dynamic boundary where only erased runtime identity remains, dispatch cannot assume the generic argument from the runtime case alone.

---

# 11. Formal Contract and Observed Evidence

Part 06 must preserve the checker architecture in which declaration contract and current proof are distinct.

Required examples:

```phalcom
const x: Option<Int> = Option::Some(1)
```

may have:

```text
binding formal type = Option<Int>
flow knowledge       = ExactCase(Some, Option<Int>)
```

For mutable state:

```phalcom
var x: Option<Int> = Option::Some(1)
x = Option::None
```

flow knowledge changes while the formal type remains `Option<Int>`.

For an API:

```phalcom
fn get() -> Option<Int> { ... }
```

callers receive the declared result contract unless formal interprocedural evidence establishes more.

Reflection APIs should make this distinction explicit when they expose both declared and observed types to tooling.

---

# 12. Reflection Architecture

## 12.1 Two reflection domains

Part 06 distinguishes:

```text
runtime object/class reflection
    actual runtime Class identity / behavior

semantic type/declaration reflection
    Enum / Variant / Family / Field / ExactCaseType
```

They may link to each other but are not interchangeable.

## 12.2 Protocol-neutral semantic reflection products

Recommended module:

```text
phalcom-semantic/src/reflection.rs
or
phalcom-semantic/src/reflection/{mod,enum,types}.rs
```

Recommended conceptual products:

```rust
pub struct EnumReflection { ... }
pub struct VariantReflection { ... }
pub struct VariantFamilyReflection { ... }
pub struct VariantFieldReflection { ... }
pub struct ExactCaseTypeReflection { ... }
```

These structs may retain canonical semantic IDs internally because they are compiler products. User-visible runtime objects must not expose those raw IDs.

## 12.3 Enum reflection product

Required fields conceptually:

```rust
pub struct EnumReflection {
    pub declaration: DeclarationId,
    pub generic_parameters: Box<[TypeParameterReflection]>,
    pub variants: Box<[VariantId]>,
    pub families: Box<[VariantFamilyId]>,
    pub native: bool,
    pub source: Option<SemanticSourceSpan>,
}
```

## 12.4 Variant reflection product

Required fields:

```rust
pub struct VariantReflection {
    pub variant: VariantId,
    pub family: Option<VariantFamilyId>,
    pub shape: VariantShape,
    pub fields: Box<[VariantFieldId]>,
    pub result_template: TypeId,
    pub exact_case_template: TypeId,
    pub visibility: VariantVisibility,
    pub case_behavior: Box<[CallableId]>,
}
```

Exact representation may avoid duplicating data already accessible from `EnumSemanticTable`; the requirement is a stable projection API.

## 12.5 Specialized exact-case reflection

Required product:

```rust
pub struct ExactCaseTypeReflection {
    pub ty: TypeId,
    pub variant: VariantId,
    pub enum_type: TypeId,
    pub fields: Box<[SpecializedVariantFieldReflection]>,
    pub result_type: TypeId,
}
```

This descriptor is created lazily from a canonical `ExactCase` and canonical declaration data.

It is not a new declaration record.

## 12.6 Variant family reflection

A family descriptor retains:

```text
VariantFamilyId
base name
member VariantIds in canonical/source declaration order as required
specialized exact member views for an owner specialization
```

A family source pattern may additionally have a query-specific candidate subset, but that subset belongs to pattern resolution/source tooling rather than the declaration's family descriptor itself.

---

# 13. Public Reflection Surface

Part 06 must implement an ergonomic public reflection API, but the semantic distinctions are more important than exact method spelling.

Recommended initial surface:

```phalcom
Option.variants
Option.variantCount
Option.variant(selector: #Some(_))
Option.variantFamily(named: #Some)
```

Variant descriptor conceptually:

```phalcom
variant.enum
variant.selector
variant.family
variant.fields
variant.singleton?
variant.constructor?
variant.resultType
variant.caseClass
```

Field descriptor:

```phalcom
field.localName
field.externalLabel
field.type
```

Exact-case type descriptor:

```phalcom
exactCase.variant
exactCase.enumType
exactCase.fields
exactCase.resultType
```

The implementation may adjust naming to existing reflection conventions. It must not change the ontology.

## 13.1 Associated lookup remains separate

These remain program associated operations:

```phalcom
Option::Some::(_)
Option::Some::*
```

They are not reflection descriptors.

## 13.2 Constructor acquisition through reflection

If reflection exposes a constructor capability, acquisition must respect construction visibility. Metadata saying “this variant is constructible in some contexts” must not automatically hand every caller an unrestricted constructor callable.

---

# 14. Reflection Visibility

The three axes remain independent.

## 14.1 Name visibility

Controls whether user code can explicitly acquire/name the variant descriptor through normal reflection lookup by name/selector.

## 14.2 Construction visibility

Controls constructor capability acquisition/invocation. It does not remove the case from exhaustiveness or necessarily hide existence from authorized reflection.

## 14.3 Payload visibility

Controls reflective access to payload field metadata/value extraction according to the language's payload visibility policy.

## 14.4 Compiler/tooling truth

Compiler semantic products retain complete information required for:

```text
exhaustiveness
impossible/reachable case reasoning
diagnostics
source maintenance
```

Tooling filters suggestions by what can legally be written while retaining complete semantic reasoning internally.

---

# 15. Runtime Reflection Metadata Projection

## 15.1 Why projection is required

Keeping a whole `SemanticSnapshot` alive in the VM would couple runtime lifetime to checker arenas, inflate artifacts, and obstruct future AOT compilation.

Part 06 therefore introduces a compact reflection projection, analogous to `ModuleLoweringSemantics` but intended for runtime introspection rather than code generation.

## 15.2 Recommended product

Recommended module:

```text
phalcom-core/src/modules/reflection_metadata.rs
```

Conceptual shape:

```rust
pub struct ModuleReflectionMetadata {
    pub enums: Box<[RuntimeEnumReflectionSpec]>,
}

pub struct RuntimeEnumReflectionSpec {
    pub stable_owner: StableDeclarationKey,
    pub name: Box<str>,
    pub native: bool,
    pub variants: Box<[RuntimeVariantReflectionSpec]>,
}

pub struct RuntimeVariantReflectionSpec {
    pub stable_variant: StableVariantKey,
    pub selector: Selector,
    pub family: Option<StableVariantFamilyKey>,
    pub shape: VariantShape,
    pub fields: Box<[RuntimeVariantFieldReflectionSpec]>,
}
```

Exact type metadata can refer to a compact/stable runtime type metadata table rather than embed semantic `TypeId`s directly.

## 15.3 Materialization

During module/program materialization:

```text
stable semantic reflection key
    -> RuntimeEnumId / RuntimeVariantId
    -> runtime case behavior class
```

Runtime reflection objects retain the stable/metadata relation without exposing the compact IDs as public values.

---

# 16. Runtime Case Behavior Class Reflection

The earlier term “hidden case behavior class” should be replaced in Part-06 docs/code comments where it implies invisibility.

Preferred terminology:

```text
runtime case behavior class
```

It is:

```text
runtime-visible
reflectable
returned by `.class`
subclass of enum root behavior class
not a source declaration
not a semantic ExactCase
```

Recommended reverse bridge:

```text
ClassId -> RuntimeVariantId -> semantic reflection metadata
```

This permits class reflection to answer “which enum variant behavior class is this?” without using the class display name.

---

# 17. Source Identity Completion

## 17.1 Extend target identity

Add to `SemanticTargetId`:

```rust
VariantFamily(VariantFamilyId),
VariantField(VariantFieldId),
```

Exact constructor/value references continue to target `Variant(VariantId)` for source navigation unless a later source-identity design introduces a distinct constructor declaration target.

## 17.2 Pattern occurrence attachment

Part-05.1 `PatternResolution`/`ResolvedVariantPattern` products preserve enough data to attach:

```text
owner token        -> DeclarationId
exact base token   -> VariantId
family base token  -> VariantFamilyId
payload label      -> VariantFieldId or explicit multi-target field set
pattern binding    -> Binding source site
```

Contextual shorthand produces the exact same targets as qualification.

## 17.3 Multi-candidate family field labels

For:

```phalcom
Dog(x, ..., named: y)
```

several candidate variants may bind `named:` to different `VariantFieldId`s.

Do not force `SemanticTargetId` to lie by picking one field. Introduce a source attachment/product capable of carrying a deterministic set of field targets for such a token, while rename/definition policy operates on the semantic relationship intended by the query.

## 17.4 Family definition

A `VariantFamilyId` may have multiple declaring `@variant` sites. Source index must support one target identity with multiple definition sites.

---

# 18. Go-to-Definition

Definition is a pure query over compiler-owned source identity.

Required behavior:

```phalcom
Some(x)
```

on `Some` -> exact `@variant Some(...)` declaration.

```phalcom
Animal::Dog*
```

on `Dog` -> all variant declarations belonging to `Dog` family.

```phalcom
Result::Error(_, reason: message)
```

on `reason` -> payload parameter declaration identified by `VariantFieldId`.

No definition query may target:

```text
runtime case behavior class
RuntimeVariantId
class display name synthesized as Enum::selector
```

unless the query is explicitly on a runtime reflection class object rather than source variant syntax.

---

# 19. Hover

## 19.1 Exact variant

Recommended protocol-neutral presentation:

```text
Option::Some<T>(_ value: T) -> Option<T>
variant constructor
```

For a specialized expression/pattern:

```text
Option::Some(_)
exact case: ExactCase<Option::Some(_), Option<Int>>
payload: Int
```

The exact display syntax may be refined, but `TypePresenter` remains the authority.

## 19.2 Singleton

```text
Option::None<T> -> Option<T>
singleton variant
```

## 19.3 GADT case

```text
Expr::Int(_ value: Int) -> Expr<Int>

pattern proof:
    T = Int
```

Only show branch equality when the semantic match product proves it at the hovered site.

## 19.4 Family pattern

```text
variant family pattern: Animal::Dog
constraint: callable selector pattern Dog(...)
reachable candidates: 4
```

Optionally enumerate candidates within a bounded presentation budget.

---

# 20. Pattern Completion

## 20.1 Semantic query input

Completion must be able to request a compiler-owned pattern completion context containing:

```text
expected pattern domain
residual space at cursor arm position
accessible/spellable exact variants/families
GADT-compatible candidates
candidate payload field metadata
```

## 20.2 Suggested product

Recommended semantic product:

```rust
pub struct PatternCompletionContext {
    pub expected: TypeKnowledge,
    pub residual: PatternSpaceSummary,
    pub candidates: Box<[PatternCompletionCandidate]>,
    pub wildcard_recommended: bool,
}
```

This may be computed on demand from retained `MatchResolution`/pattern-space summaries; it must not reproduce the proof in LSP.

## 20.3 Ranking

Ranking order:

1. exact uncovered spellable cases;
2. useful family/selector patterns that cover uncovered space;
3. `_` when opaque/unspellable residual remains;
4. lower-priority general pattern constructs.

Impossible GADT cases are omitted.

---

# 21. Add Missing Cases Code Action

The semantic layer produces a `MissingCaseEditPlan` or equivalent containing source-independent case descriptions.

LSP then renders edits using current formatting/indentation.

Requirements:

```text
only uncovered reachable cases
no impossible GADT cases
respect union narrowing
inaccessible/unspellable case -> wildcard strategy when required
no duplicate existing arms
contextual shorthand only when unambiguous
```

Coverage witnesses remain the source of truth.

---

# 22. Generate Match Code Action

Given a typed expression of a closed enum/GADT/closed union, semantic tooling may produce a complete match skeleton plan.

Example:

```phalcom
match result {
    Ok(value) => {
        
    }
    Error(error) => {
        
    }
}
```

Names for payload bindings should derive from declared local field names where available and avoid lexical collisions using existing name-generation utilities.

Qualification policy follows D06-13.

---

# 23. Rename

## 23.1 Family rename

Standard Rename on any variant base token resolves to `VariantFamilyId` and edits:

```text
all @variant declarations in that family
qualified exact references
contextual exact references
whole-family references
selector-family patterns
exact associated constructor/family references sharing that family base
```

Only occurrences whose semantic identity belongs to that family are edited.

## 23.2 Payload external label rename

Rename of an external label updates selector identity-affecting declarations/usages and must run semantic conflict validation because selector collisions may result.

## 23.3 Local payload name rename

Local declaration name is ordinary local/declaration rename and does not alter external selector identity unless both names are syntactically the same declaration field and the language's rename UX explicitly requests both.

---

# 24. Semantic Highlighting

Part 06 adds/uses token classes or modifiers sufficient to distinguish:

```text
enum declaration/type reference
variant exact member
variant family reference
payload selector label
pattern binding
wildcard
type parameter
```

The LSP token emitter consumes AST/source-index semantic classifications.

For:

```phalcom
Result::Error(_, reason: message)
```

expected classification is:

```text
Result      enum/type
Error       variant
_           wildcard
reason      payload selector label
message     pattern binding
```

---

# 25. Diagnostic Presentation

Part 05 already owns the semantic error/proof. Part 06 owns final presentation integration.

## 25.1 Non-exhaustive

```text
match is not exhaustive

uncovered:
    Option::None
```

## 25.2 GADT impossible

```text
pattern is impossible

Expr::Bool requires:
    T = Bool

scrutinee establishes:
    T = Int
```

## 25.3 Ambiguous contextual variant

```text
contextual variant `Error` is ambiguous

candidates:
    NetworkResult::Error(_)
    ParseResult::Error(_)

qualify the variant explicitly
```

Protocol-neutral diagnostic presentation should consume explanation DAG nodes and semantic witness products rather than recompute reasons.

---

# 26. Generic Specialization Audit

Part 06 performs an end-to-end consistency audit, not a new generic solver.

For:

```phalcom
Option<Int>::Some(1)
```

and:

```phalcom
const x: Option<Int> = ...
match x {
    Some(v) => ...
    None => ...
}
```

all of these must share canonical substitution/specialization rules:

```text
constructor invocation
associated exact lookup
family reification
ExactCase construction
pattern candidate field specialization
reflection exact-case specialization
hover type display
typed dispatch relation
```

Any feature-specific generic substitution helper that produces materially independent semantics is a Part-06 cleanup target.

---

# 27. GADT End-to-End Audit

The declaration/elimination boundary remains:

```text
VariantInfo.CaseTypeEnvironment
    declaration evidence/template

match compatibility solver
    proves compatibility for scrutinee specialization

branch proof environment
    branch-local evidence

runtime
    proof erased
```

Reflection may describe declared case constraints/result specialization, but runtime values do not carry proof objects.

Tooling must never infer branch equalities from runtime case class identity.

---

# 28. Visibility End-to-End Audit

Test all three axes across:

```text
associated lookup
construction
family capture
matching
exhaustiveness
reflection
completion
hover
definition
rename
cross-module access
```

Required invariants:

```text
construction-private case may still be matchable if matching visibility allows it
construction-private case still participates in exhaustiveness
name-inaccessible reachable case still participates in exhaustiveness
completion never suggests illegal spelling
reflection cannot become a construction/payload access backdoor
```

---

# 29. Cross-Module and Package Semantics

Part 06 must move beyond single-module fixtures.

Required scenario:

```text
module A
    declares enum

module B
    constructs / reifies family

module C
    matches / reflects / hovers / navigates

package facade
    re-exports enum/type
```

All operations preserve the defining `DeclarationId`/variant identities.

Re-export does not clone `VariantId` or create a new variant family universe.

---

# 30. Incremental Analysis

## 30.1 Variant set edit

Adding a variant changes dependent exhaustiveness/residual-space/reflection/tooling products.

## 30.2 Selector/field edit

Changing external label/shape invalidates family resolution, pattern resolution, reflection metadata, navigation, and rename conflict checks.

## 30.3 GADT result edit

Changing case result specialization invalidates compatibility proofs and exact-case specializations downstream.

## 30.4 Visibility edit

Changes affected reflection/completion/associated acquisition without erasing cases from semantic exhaustiveness.

## 30.5 Fingerprints

Fingerprint semantic content, not rendered prose or source ranges alone.

---

# 31. Stable Metadata and Serialization

## 31.1 Local handles

These remain snapshot/runtime-local:

```text
TypeId
VariantTypeId
RuntimeEnumId
RuntimeVariantId
CaseDiscriminant
SourceSiteLocalId
```

## 31.2 Stable keys

Persistent artifacts use stable equivalents of:

```rust
StableVariantKey { owner, selector }
StableVariantFamilyKey { owner, base }
StableVariantFieldKey { variant, index }
StableExactCaseKey { variant, enum_type: StableTypeKey }
```

If existing project/module/declaration stable identities already cover portions of this structure, reuse them.

## 31.3 Determinism

Reflection/source metadata ordering must be deterministic from semantic declaration order/canonical selector order, not hash-map traversal.

Source-range-only edits do not change range-free semantic identity.

---

# 32. Documentation Synchronization

Part 06 must update authoritative language documentation for:

```text
ADTs and native ADTs
GADTs
variant families
associated lookup
singleton vs nullary constructors
ExactCase semantic type
match / wildcard / or-pattern
selector/family patterns
static totality / exhaustiveness
visibility axes
reflection ontology
runtime class vs semantic variant/type distinction
core Option / Result / Ordering semantics
```

Old docs describing class-style enum variants or core Option as an unrelated semantic mechanism must be removed from authoritative locations or explicitly marked historical.

Documentation must distinguish:

```text
ratified source syntax
presentation-only type rendering
reflection API
internal compiler notation
```

so Part 06 does not accidentally ratify exact-case source annotation syntax through hover examples.

---

# 33. Final Conformance Test Organization

Tests should converge on capability names:

```text
phalcom-semantic/tests/semantic/adts/
    declarations.rs
    constructors.rs
    associated_lookup.rs
    families.rs
    generics.rs
    gadts.rs
    exact_cases.rs
    matching.rs
    exhaustiveness.rs
    or_patterns.rs
    selector_patterns.rs
    visibility.rs
    unions.rs
    reflection.rs
    modules.rs
    diagnostics.rs
    tooling.rs

phalcom-core/tests/
    adt_runtime.rs
    adt_end_to_end.rs
    native_adt_runtime.rs
    reflection_runtime.rs
    match_runtime.rs
    match_gadt_runtime.rs
    ...

phalcom-lsp tests
    hover / definition / completion / rename /
    semantic tokens / code actions / diagnostics
```

Avoid Part-number test modules as the final language organization.

---

# 34. Generated/Fuzz Robustness

Pattern/exhaustiveness is suitable for generated property tests.

Generate bounded structures:

```text
closed enums with 0..N payload shapes
exact-case unions
nested variants
or-pattern trees
wildcards
selector-family overload sets
GADT result specializations
```

Properties:

```text
checker does not panic
accepted exhaustive match residual is empty
reported redundant alternatives consume no new generated values
lowered execution agrees with generated reference case model
exact-case canonicalization is idempotent
source target attachment never picks arbitrary same-name declaration
```

Use deterministic seeds in CI and retain minimized regression fixtures for discovered failures.

---

# 35. Architectural Deletion Pass

Before completion, repository searches must establish absence of semantic violations.

Search classes include:

```text
constructor == "Some"
constructor == "None"
variant lookup by display/source string in phalcom-core
match variant by .class
LSP family/pattern resolution by AST spelling
runtime GADT equality/proof state
duplicate enum/family tables
semantic ID reconstructed from formatted type/selector text
core Option semantic branch independent of EnumInfo
persistent RuntimeVariantId / CaseDiscriminant as source identity
```

Not every occurrence of strings `Some`/`None` is forbidden: core declaration/tests/documentation/native representation code can name them. The forbidden condition is semantic dispatch/identity determined from those strings rather than canonical core identities.

---

# 36. Final Vertical Architecture Scenario

The final architecture proof should include:

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}

fn eval<T>(_ expr: Expr<T>) -> T {
    match expr {
        Int(value) => value
        Bool(value) => value
    }
}
```

and verify:

```text
parser
AST
EnumInfo
VariantId
CaseTypeEnvironment
constructor specialization
ExactCase canonicalization
contextual pattern resolution
PatternSpace
GADT compatibility
branch equality
exhaustiveness
match result typing
explanation DAG
source index exact targets
semantic lowering
IsVariant
GetVariantPayload
VM execution
runtime case class
variant reflection
exact-case type reflection
hover
go-to-definition
rename target identity
incremental invalidation
stable metadata projection
```

Also run parallel native-core scenarios for `Option` and `Result` to prove representation independence.

---

# 37. Explicit Non-Goals / Part-07 Handoff

Part 06 explicitly defers:

```text
performance benchmark suite
match jump tables
decision DAG factoring
shared-prefix factoring
payload extraction reuse
family candidate compression
native/AOT ADT layout optimization
Result immediate/niche representation optimization
specialization-driven runtime generic tagging
```

The Part-07 handoff should contain any correctness-preserving optimization opportunities discovered during Part 06 without pulling them into the completion gate.

---

# 38. Completion Criteria

Part 06 is complete only when all of the following hold:

```text
Option is a canonical @native enum semantic declaration
Result is a canonical @native enum semantic declaration
Ordering participates in canonical ADT semantics
Bool remains primitive
core native variants use ordinary VariantId/ExactCase/family products
Option immediate representation survives without semantic Option exceptions
Result physical representation is semantically invisible
ExactCase specialization is canonical and reflectable
formal declared type remains distinct from observed exact-case evidence
.class returns reflectable runtime case behavior class
runtime case class is not semantic variant/type identity
reflection has dedicated enum/variant/family/field/exact-case metaobjects
:: has no reflection overload
reflection is visibility-aware
runtime reflection uses projected metadata
SemanticTargetId/source products represent variant families and fields
qualified/contextual pattern references share semantic targets
family patterns retain family + candidate identities
go-to-definition uses semantic source declarations
hover presents exact/family/GADT information from semantic products
completion uses expected/residual PatternSpace
missing-case and Generate Match actions use semantic witnesses
rename is family-semantic and distinguishes external labels/local names
semantic highlighting is semantic/AST-driven
match diagnostics render explanation DAG/witnesses
cross-module/package identity is stable
incremental changes invalidate all dependent products
persistent metadata uses stable keys, not local IDs
capability-organized conformance suite is complete
generated robustness tests cover proof machinery
legacy/transitional ADT paths are deleted
language documentation is synchronized
Part 07 receives optimization/performance backlog only
```

---

# 39. Final Architecture Statement

After Part 06, the phrase “native enum” must describe only where implementation comes from, never how language semantics are obtained.

The final Phalcom architecture is:

```text
semantic declaration/type/proof identity
    is authoritative

runtime representation
    may be specialized

runtime class
    may be case-specific and reflectable

reflection
    projects semantic metadata

source tooling
    projects semantic source identity/proofs

compiler/VM
    execute frozen semantic consequences
```

This is the convergence condition for the ADT/GADT/associated-family series.
