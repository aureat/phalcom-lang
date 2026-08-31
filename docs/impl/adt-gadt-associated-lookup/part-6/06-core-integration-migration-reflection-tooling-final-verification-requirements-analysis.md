# Phalcom ADT/GADT + Associated Lookup
## Part 06 — Core Integration, Migration, Reflection, Tooling Completion, and Final Verification
### Requirements Analysis

**Status:** Requirements analysis / ratified-decision consolidation  
**Series:** ADT/GADT + Associated Lookup, Part 6 of 6  
**Repository:** `aureat/phalcom-lang`  
**Repository branch inspected:** `feat/adts`  
**Repository commit inspected:** `26166385f9c1bf35f6e9eb969385fc8a162f2f56`  
**Commit subject:** `ci: apply and verify ordered match orchestration`  
**Verified on:** 2026-08-31  
**Intended repository path:** `docs/impl/adt-gadt-associated-lookup/part-6/06-core-integration-migration-reflection-tooling-final-verification-requirements-analysis.md`

---

# 1. Purpose

Parts 01–05 establish Phalcom's ADT/GADT/associated-family language model and executable elimination semantics. Part 06 is the convergence phase: it removes remaining transitional exceptions, makes core sum types participate in the same semantic model as user declarations, completes reflection and source tooling over the semantic products, proves cross-module/incremental stability, synchronizes documentation, and deletes obsolete architecture.

Part 06 is not another semantic redesign phase. In particular, it must preserve the Part-05 boundary:

```text
phalcom-semantic
    owns declaration identity, associated resolution, type proofs,
    pattern resolution, GADT compatibility, usefulness, exhaustiveness

semantic-to-backend projection
    freezes executable consequences

phalcom-core
    executes exact identities and payload slots

phalcom-lsp / CLI presentation
    project compiler-owned semantic products
```

The completion criterion is stronger than “all features work.” The target is one coherent implementation world:

```text
one declaration world
one canonical type world
one associated-family world
one pattern/proof world
one source-identity world
one lowering projection
one runtime identity bridge
```

---

# 2. Source Basis and Precedence

This analysis is grounded in:

1. ratified project decisions in the Part-06 design conversation;
2. the landed/uploaded Part 05.2 technical specification and implementation plan;
3. Part 05.1 match/pattern/exhaustiveness specification and plan;
4. Part 04 runtime representation/lowering specification and plan;
5. Part 03 associated resolution/family/generic specialization specification and plan;
6. the connected `feat/adts` repository snapshot for concrete files and current seams.

When concrete Rust names differ after Part 05.2 finishes, implementation must mechanically adapt to the landed structures rather than introducing duplicate models.

The connected repository currently confirms several important facts:

```text
phalcom-ast/src/ast.rs
    EnumDef already has `attributes: Vec<Attribute>`
    BuiltinAttr already contains Native

phalcom-semantic/src/types/store.rs
    TypeStore hash-conses canonical TypeData
    TypeData already contains Applied, ExactCase, Family
    exact_case_type(...) validates and interns ExactCase

phalcom-semantic/src/identity.rs
    stable VariantId / VariantFamilyId / VariantFieldId / VariantConstructorId exist
    SemanticTargetId currently includes Variant, but not VariantFamily or VariantField

phalcom-semantic/src/source_index/*
    source identity is compiler-owned and protocol-neutral

phalcom-semantic/src/presentation.rs
    TypePresenter is already the protocol-neutral type presentation seam

phalcom-core/src/value/option.rs
    Option is currently an allocation-free immediate encoding

phalcom-core/src/vm/adt.rs
    runtime variant/case primitives and per-case behavior classes already exist

phalcom-core/src/modules/semantic_lowering.rs
    semantic-to-codegen projection is already the compiler authority boundary

phalcom-lsp/src/{hover,completion,diagnostics,backend}.rs
    LSP presentation/feature seams already exist and should remain consumers,
    not alternate semantic engines
```

---

# 3. Ratified Part-06 Decisions

## D06-01 — `Option` and `Result` are canonical `@native` enums

Canonical source-facing declarations are conceptually:

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
```

`@native` describes implementation provenance and/or physical representation. It does **not** grant alternate enum semantics.

Both declarations must publish ordinary semantic products:

```text
DeclarationId
EnumInfo
VariantId
VariantFamilyId
VariantFieldId
VariantConstructorId
VariantInfo
ExactCase
associated surfaces
source occurrences
reflection metadata
```

`Option` retains its immediate runtime representation. `Result` is also a native core ADT, but Part 06 does not require a particular optimized physical encoding. Physical Result optimization is implementation policy and may be deferred to Part 07.

Required invariant:

> Native core ADTs are builtin declarations, not builtin semantic exceptions.

## D06-03 — Core ADT eligibility

The core types that should participate in ADT-style closed-case semantics are:

```text
Option       yes
Result       yes
Ordering     yes
Bool         no; remains primitive
Unit/Nil     no; remain primitive
```

The criterion is semantic, not cardinality:

> A core type joins the ADT model when it has named source-level cases that should participate in variant identity, exact-case reasoning, associated lookup, matching, exhaustiveness, reflection, and tooling.

Part 06 must not force every finite primitive domain into enum representation.

## D06-04 — `ExactCase` is an official semantic type concept

`TypeData::ExactCase` is not temporary checker state. It is part of the canonical semantic type model and may be exposed through type reflection and presentation.

However, Part 06 does **not** ratify new source annotation grammar for exact-case types. A diagnostic/hover/reflection presentation may expose an exact case without making that rendering parseable as a user-authored type annotation.

## D06-05 — Formal type and observed exact-case proof remain separate

For:

```phalcom
const x: Option<Int> = Option::Some(1)
```

Phalcom may retain:

```text
formal/declared contract:
    Option<Int>

current observed knowledge:
    exact Option::Some case specialized to Option<Int>
```

The exact observation must not overwrite the declaration contract. Mutation/control-flow joins may change observed knowledge while the formal type remains stable.

This rule is especially important for GADT elimination and typed dispatch.

## D06-06 — `.class` returns the actual runtime case behavior class

Per-case runtime behavior classes are real runtime class objects and are reflectable. For an ADT value, `.class` may return the case behavior class.

They are nevertheless distinct from semantic exact-case identity:

```text
runtime case behavior class
    real runtime Class
    reflectable
    used by ordinary `.` dispatch

ExactCase
    canonical semantic type
    used by typing/refinement/dispatch proof

VariantId
    stable declaration identity
    used by matching/source identity
```

A case behavior class does not acquire a source `DeclarationId` merely because it is reflectable. Pattern navigation continues to the `@variant` declaration.

The project may reconsider an enum-root `.class` policy later; Part 06 implements the ratified current behavior.

## D06-07 — Reflection uses dedicated semantic metaobjects

Reflection must expose source-semantic concepts through dedicated reflection objects rather than reusing constructor callables, families, hidden compiler IDs, or generic maps.

Required conceptual metaobjects include:

```text
Enum
Variant
VariantFamily
VariantField
ExactCaseType / VariantType
VariantConstructor metadata where needed
```

Runtime class reflection remains ordinary class reflection and is not a substitute for semantic variant reflection.

## D06-08 — `::` does not gain a reflection meaning

Associated lookup remains exactly the language mechanism established in Parts 01–05.

For an enum that has only:

```phalcom
@variant Some(_ value: T)
```

`Option::Some` must not suddenly mean “variant reflection descriptor.” It remains exact getter associated lookup and therefore is invalid unless a getter-shaped member exists.

Reflection uses a separate ordinary API, conceptually:

```phalcom
Option.variants
Option.variant(...)
Option.variantFamily(...)
```

Exact public API spelling is finalized in the Part-06 reflection task, but no spelling may redefine `::`.

## D06-09 — Specialized exact cases are canonical semantic types

Specialized exact cases are not ephemeral presentation-only views.

Example conceptual facts:

```text
Variant declaration:
    VariantId(Option::Some(_))

canonical specialization A:
    ExactCase(Option::Some(_), Option<Int>)

canonical specialization B:
    ExactCase(Option::Some(_), Option<String>)
```

Equivalent specialization must normalize to the same canonical `TypeId` in one type store, exactly as other canonical type forms do.

Reflection may lazily reify a canonical exact specialized type, including:

```text
exact VariantId
specialized enum owner type
specialized payload field types
specialized result type
GADT specialization/equalities represented as semantic metadata where applicable
```

This does **not** imply:

```text
new DeclarationId per specialization
new RuntimeVariantId per specialization
new runtime case behavior class per specialization
runtime generic type token on every value
```

The VM remains generically erased. Static typed multiple dispatch may distinguish specialized exact cases when proof is available; a fully erased dynamic value cannot reconstruct generic arguments that were never retained.

## D06-10 — Reflection acquisition is access-filtered

Reflection honors the existing independent visibility axes:

```text
name visibility
construction visibility
payload visibility
```

The compiler's semantic database still knows the complete closed enum universe for exhaustiveness. User reflection sees only what the acquisition context is authorized to observe/use.

Acquired reflection values behave as capabilities and do not re-run lexical visibility on every subsequent operation.

## D06-11 — Family-pattern source identity is the family plus exact candidates

A source occurrence such as:

```phalcom
Animal::Dog*
Animal::Dog(...)
Animal::Dog(x, ..., named: y)
```

has primary semantic identity:

```text
VariantFamilyId(Dog)
```

and retains the exact candidate `VariantId`s resolved by semantic analysis.

Exact patterns continue to target one exact `VariantId`.

Go-to-definition for a family reference may return every declaration contributing to that variant family; it must not choose an arbitrary first member.

## D06-12 — Ordinary rename of a variant base renames the family

Given overloaded variants:

```phalcom
@variant Dog
@variant Dog()
@variant Dog(_ name: String)
```

Rename on `Dog` is a `VariantFamilyId` rename and updates the complete family plus all exact/family/contextual references that resolve to it.

Moving one exact selector into another family is a separate structural refactoring, not ordinary Rename.

External payload labels and local payload names are distinct rename identities.

## D06-13 — Generated pattern syntax is semantically chosen

Editor-generated match cases use contextual shorthand only where expected-pattern semantics prove it unambiguous. Otherwise generation uses qualification.

Residual-space completion ranks uncovered exact cases above `_` when they are spellable. If a reachable uncovered case cannot legally be named, `_` is the appropriate generated/suggested surface.

## D06-14 — Runtime reflection consumes projected metadata

The VM must not keep the complete live `SemanticSnapshot` solely to implement reflection.

Compilation/materialization projects the semantic metadata required at runtime into a compact immutable reflection product.

Conceptually:

```text
SemanticSnapshot
    ↓ projection
RuntimeReflectionMetadata
    ↓ materialization/linking
RuntimeEnumId / RuntimeVariantId
    ↓
reflection objects
```

## D06-15 — Persistent semantic identity is stable, never arena/runtime-local

Persistent metadata/snapshots/artifacts must not treat these as durable language identities:

```text
TypeId
VariantTypeId
RuntimeEnumId
RuntimeVariantId
CaseDiscriminant
SourceSite local ordinal without snapshot identity
pointer / ObjRef / arena address
```

Persistent keys derive from stable project/module/declaration identities and exact selector/field/family structure, then remap to compact local handles when loaded.

## D06-16 — Performance work belongs to Part 07

Part 06 contains no performance-baseline or match-optimization work unless a correctness implementation is unusably pathological.

The following move to Part 07/backlog:

```text
match jump tables
decision DAG factoring
shared-prefix factoring
payload extraction reuse
candidate-set compression
performance benchmarking/baselines
specialized Result physical optimization
```

---

# 4. Requirements by Capability

Requirements use stable IDs so the implementation plan and verification report can trace them.

## 4.1 Core migration requirements

### R06-CORE-01 — Native enum declarations

The core surface must contain canonical `@native enum` declarations for `Option`, `Result`, and `Ordering` (or structurally equivalent compiler-owned source products if the core declaration source is generated).

### R06-CORE-02 — Native has no alternate enum semantic path

Enum declaration analysis must consume native and user enums through the same `EnumInfo`/`VariantInfo` construction path after source/native binding authorization is established.

### R06-CORE-03 — Canonical core variant identities

`Option::Some`, `Option::None`, `Result::Ok`, `Result::Error`, and `Ordering` cases must have ordinary stable `VariantId`s and family identities.

### R06-CORE-04 — Core exact cases

Construction and known singleton acquisition must produce ordinary canonical `ExactCase` types.

### R06-CORE-05 — Core associated lookup/family behavior

Core native enum variants must participate in the same `::` exact/family resolution as user enums.

### R06-CORE-06 — Core matching/exhaustiveness

Core native enums must enter `PatternSpace` through ordinary enum semantics. Part-05.2's temporary core Option semantic/runtime adapter must be removable without source-name matching returning.

### R06-CORE-07 — Preserve Option immediate representation

`Option` remains allocation-free/immediate under the current VM representation, including nested `Some` behavior and GC visibility.

### R06-CORE-08 — Result native implementation independence

`Result` may use ordinary ADT case objects or a native representation. Whatever representation is chosen must implement the same runtime case interface and not affect semantic identity.

### R06-CORE-09 — Bool remains primitive

No Part-06 migration may manufacture `VariantId(True/False)` solely for exhaustiveness.

## 4.2 Canonical exact-case/type requirements

### R06-TYPE-01 — ExactCase canonicalization

`TypeStore::exact_case_type` remains the sole canonical constructor for exact cases. Equivalent enum application + exact variant identity yields one canonical `TypeId` within a store.

### R06-TYPE-02 — Specialization fidelity

Exact-case specialization preserves the canonical specialized enum result and candidate-specific field types, including GADT result specialization.

### R06-TYPE-03 — Contract/evidence separation

Binding/declaration products keep formal declared type separate from observed/refined exact-case knowledge.

### R06-TYPE-04 — Typed dispatch compatibility

The relation/dispatch machinery must accept canonical `ExactCase` as a legitimate type discriminator. Static dispatch may distinguish `ExactCase(V, E<Int>)` from `ExactCase(V, E<String>)` when those are distinct canonical types and the proof is available.

### R06-TYPE-05 — No runtime generic reification requirement

Typed dispatch support must not force every ADT instance to store generic type arguments at runtime.

### R06-TYPE-06 — Canonical presentation

`TypePresenter` must provide one deterministic exact-case presentation used by hover, diagnostics, reflection debug text, and tests. The rendering is not automatically source annotation syntax.

## 4.3 Reflection requirements

### R06-REFL-01 — Dedicated semantic metaobjects

Reflection exposes enums, variants, families, fields, and exact-case types through typed metaobjects.

### R06-REFL-02 — No compiler-private IDs in language API

The public API must not expose `VariantId`, `TypeId`, `RuntimeVariantId`, `CaseDiscriminant`, or lowering-site keys as user values.

### R06-REFL-03 — Enum reflection

An enum descriptor must expose at least:

```text
name / declaration identity presentation
variants
variant count
family lookup
variant lookup by exact selector
native? / implementation provenance if public policy exposes it
```

### R06-REFL-04 — Variant reflection

A variant descriptor must expose at least:

```text
enum owner
exact selector
family
singleton/constructor shape
payload fields
result type template
visibility metadata subject to access policy
case behavior class relationship
```

### R06-REFL-05 — Exact-case type reflection

A specialized exact-case descriptor exposes:

```text
variant declaration
specialized enum type
specialized payload field types
specialized result type
canonical type identity within the reflection context
```

### R06-REFL-06 — Family reflection

A family descriptor exposes family base, exact members, selector shapes, and specialized member views without pretending the family is one exact variant.

### R06-REFL-07 — `.class` behavior

ADT values return their runtime case behavior class from `.class`; runtime class reflection may connect back to variant metadata, but the class does not become the semantic exact-case type.

### R06-REFL-08 — Visibility

Reflection acquisition obeys name/construct/payload visibility independently. Reflection must not provide a construction backdoor.

### R06-REFL-09 — Projected runtime metadata

Runtime reflection is backed by a compact projection produced from formal semantic products, not by re-resolving source or retaining the whole snapshot.

## 4.4 Source index and navigation requirements

### R06-IDX-01 — Target universe extension

`SemanticTargetId` gains structural equivalents of:

```text
VariantFamily(VariantFamilyId)
VariantField(VariantFieldId)
```

while exact variants continue to use `Variant(VariantId)`.

### R06-IDX-02 — Exact pattern attachment

Qualified and contextual exact variant pattern tokens attach to the same `VariantId`.

### R06-IDX-03 — Family pattern attachment

`Base*` and selector-family pattern base tokens attach to `VariantFamilyId` and retain resolved candidates for hover/tooling.

### R06-IDX-04 — Payload-label attachment

An explicit external payload label in a variant pattern attaches to the exact candidate field identity/identities. If one source family pattern maps a label to multiple candidate fields, the source product must model that multi-target relation explicitly rather than pick one arbitrarily.

### R06-IDX-05 — Go-to-definition

Navigation targets source declarations, not runtime behavior classes or display strings.

### R06-IDX-06 — Cross-module identity

Imported/re-exported enum/variant/family occurrences resolve to the declaration identities in the defining module/package.

## 4.5 Hover requirements

### R06-HOVER-01 — Exact variant hover

Hover can show exact selector, specialized constructor/value type, enum result, payloads, and variant kind.

### R06-HOVER-02 — GADT proof hover

Where pattern success establishes case equalities, hover may include the branch-local established equalities using semantic explanation/proof products.

### R06-HOVER-03 — Family-pattern hover

Family pattern hover shows family identity, selector constraint, and reachable candidate summary.

### R06-HOVER-04 — No runtime-class substitution

Type hover reports semantic exact-case/root types; `.class` hover may separately report runtime case behavior class.

## 4.6 Completion/code-action requirements

### R06-COMP-01 — Pattern-domain completion

Completion inside pattern position uses the expected pattern space, not a global variant-name scan.

### R06-COMP-02 — Residual-space ranking

After previous arms, uncovered reachable cases are ranked first.

### R06-COMP-03 — GADT reachability

Impossible GADT cases are not suggested.

### R06-COMP-04 — Visibility-aware spelling

Unspellable reachable cases remain in exhaustiveness but are not offered as illegal explicit completions; `_` is offered where necessary.

### R06-COMP-05 — Add missing cases

A code action consumes `CoverageWitness`/residual-space semantic products and generates only missing reachable cases.

### R06-COMP-06 — Generate match

A code action can generate a complete match skeleton from a closed enum/GADT/closed union, choosing contextual versus qualified spelling according to D06-13.

## 4.7 Rename requirements

### R06-RENAME-01 — Family-base rename

Rename on a variant base is keyed by `VariantFamilyId` and changes all exact variants in that family plus all references resolving to the family/members.

### R06-RENAME-02 — No cross-enum collision

Unrelated same-name families are untouched.

### R06-RENAME-03 — Payload label versus local name

External selector labels and local payload names/bindings are independently renameable.

### R06-RENAME-04 — Contextual references

Contextual shorthand occurrences are renamed because their semantic target matches, not because their text matches.

## 4.8 Semantic highlighting requirements

### R06-HL-01 — Semantic token distinctions

At minimum tooling can distinguish:

```text
enum/type
variant exact member
variant family reference
pattern binding
wildcard
payload selector label
type parameter
```

### R06-HL-02 — No regex/name guessing

Classification is projected from AST/source-index/semantic identity.

## 4.9 Diagnostic presentation requirements

### R06-DIAG-01 — Match diagnostic rendering

CLI/LSP render semantic match diagnostics and causal explanations without recomputing the proof.

### R06-DIAG-02 — Missing-space rendering

Non-exhaustive diagnostics show precise missing cases/witnesses where available and opaque residual guidance otherwise.

### R06-DIAG-03 — GADT contradiction rendering

Impossible-case diagnostics can explain contradictory case equalities.

### R06-DIAG-04 — Contextual ambiguity rendering

Ambiguous shorthand reports viable owner/family interpretations and asks for qualification.

## 4.10 Incremental/cross-module requirements

### R06-INCR-01 — Added/removed variant invalidation

Changing a closed enum's variant set invalidates dependent exhaustiveness products, completion residuals, reflection metadata, and affected source tooling.

### R06-INCR-02 — Case-signature/GADT invalidation

Changing payload/result specialization invalidates constructor typing, matching, reflection, hover, and typed dispatch dependents.

### R06-INCR-03 — Visibility invalidation

Changing name/construct/payload visibility invalidates affected associated/reflection/tooling products.

### R06-INCR-04 — Re-export/import identity stability

Cross-module/package re-exports preserve canonical declaration identity instead of generating duplicate semantic variants.

## 4.11 Persistence/metadata requirements

### R06-PERSIST-01 — Stable semantic keys

Persisted artifacts encode stable declaration/selector/family/field keys and remap them to local compact IDs.

### R06-PERSIST-02 — No runtime discriminants as semantic IDs

Physical discriminants and runtime IDs remain artifact/runtime implementation details.

### R06-PERSIST-03 — Reflection metadata reproducibility

Equivalent source semantics produce deterministic reflection metadata independent of hash-map traversal and source-range-only edits.

## 4.12 Documentation/cleanup requirements

### R06-DOC-01 — Authoritative language docs

Update authoritative specs for ADTs, GADTs, associated lookup, match, wildcard/or/family patterns, exhaustiveness, visibility, native core ADTs, and reflection.

### R06-DOC-02 — Remove contradictory legacy docs

Class-like enum-variant or legacy Option semantics must be updated, archived as historical, or removed from authoritative paths.

### R06-CLEAN-01 — Delete source-name ADT semantics

No semantic/compiler/runtime path may reconstruct case meaning from strings such as `"Some"` or `"None"`.

### R06-CLEAN-02 — Delete duplicate family/pattern resolution

Only `phalcom-semantic` owns source associated/pattern/GADT/exhaustiveness reasoning.

### R06-CLEAN-03 — Delete transitional core Option identity adapter

Once core Option is a canonical enum declaration and runtime registration consumes it, temporary semantic compatibility identities from Part 05.2 must be removed or collapsed into the general native-enum registration path.

### R06-CLEAN-04 — LSP remains presentation/protocol layer

No LSP feature may parse pattern semantics, resolve variant families independently, or infer missing match cases without compiler semantic products.

## 4.13 Robustness/final proof requirements

### R06-TEST-01 — Capability-organized conformance suite

Tests are organized by language capability (`constructors`, `families`, `gadts`, `matching`, etc.), not by implementation part numbers.

### R06-TEST-02 — Generated/fuzz semantic testing

Generated closed enums, exact-case unions, nested/or/family patterns, and GADT specializations exercise coverage/usefulness without panics or soundness divergence.

### R06-TEST-03 — Vertical end-to-end proof

At least one GADT scenario verifies parser → semantics → exact case → match proof → lowering → runtime → reflection → hover → definition → rename → incremental update.

---

# 5. Required Architecture Changes

The requirements imply six major architecture changes.

## 5.1 Native enum declaration convergence

Today the language already has `@native` attribute infrastructure and dedicated `EnumDef.attributes`. Part 06 should therefore make core declarations visible as ordinary enum syntax/products and teach core-surface binding to authorize their native implementation.

Do **not** build a parallel `NativeEnumInfo` model.

Recommended shape:

```text
source/native core declaration
    ↓ parse
EnumDef(attributes includes @native)
    ↓ declaration analysis
EnumInfo / VariantInfo
    ↓ ordinary semantic consumers

separate native implementation registry
    DeclarationId / VariantId -> implementation strategy
```

## 5.2 Runtime representation strategy boundary

The runtime needs a general way to ask how a semantically ordinary native enum is physically represented.

Conceptually:

```rust
pub enum RuntimeAdtRepresentation {
    General,
    NativeOption,
    NativeResult, // only if actually required
}
```

The exact enum is optional; the required architectural property is that special representation is selected by stable semantic identity/registered native declaration, not by pattern/compiler source strings.

`runtime_variant_of`, `case_payload_len`, `case_payload_at`, singleton loading, and construction must be the common case interface.

## 5.3 Canonical reflection projection

Introduce semantic reflection descriptors/projectors owned by `phalcom-semantic`, then a compact runtime projection in `phalcom-core`.

Do not make the VM read checker arenas directly.

## 5.4 Source identity completion

Extend source target identity for variant families and fields, then attach Part-05 pattern products to precise source token ranges.

This is the prerequisite for definition, rename, semantic highlighting, and family-pattern hover.

## 5.5 Protocol-neutral tooling products

Complex editor features should be expressed as semantic/presentation products before LSP conversion.

Examples:

```text
PatternCompletionContext
MissingCaseEditPlan
GeneratedMatchPlan
VariantHoverPresentation
FamilyPatternPresentation
RenameSemanticSet
```

`phalcom-lsp` converts these into LSP protocol types; it does not rebuild the reasoning.

## 5.6 Stable persistent key layer

Snapshot-local/arena-local IDs remain excellent internal handles. Part 06 needs a deliberate persistent key projection for reflection artifacts, serialized metadata, and any long-lived source index cache.

---

# 6. Reflection Requirements: Recommended Semantic Ontology

Part 06 should distinguish declaration metadata from specialized type metadata.

```text
EnumDescriptor
    declaration identity
    generic parameters
    variants/families

VariantDescriptor
    one VariantId
    exact selector
    template fields/result
    family

VariantFamilyDescriptor
    one VariantFamilyId
    exact member declarations

VariantFieldDescriptor
    one VariantFieldId
    local name / external label
    template declared type

ExactCaseTypeDescriptor
    one canonical specialized ExactCase TypeId in current semantic world
    VariantDescriptor
    specialized enum type
    specialized field types
    specialized result
```

This solves the D06-09 problem cleanly:

```text
Some<Int> reflection
    is a canonical exact-case type descriptor

not
    a new variant declaration

not
    a new runtime class
```

The public API may use language-level wrappers around these descriptors, but compiler IDs remain internal.

---

# 7. Runtime Class Versus Semantic Type

Because D06-06 selects case behavior classes as `.class`, Part 06 must explicitly prevent accidental conflation.

For a value known statically as an exact `Some<Int>`:

```text
value.class
    -> runtime case behavior Class

semantic type reflection
    -> ExactCaseTypeDescriptor(Some, Option<Int>)

variant reflection
    -> VariantDescriptor(Option::Some(_))
```

All three are meaningful and different.

Required negative tests must prove:

```text
.class identity is not used for match semantics
runtime behavior class is not inserted as VariantId
exact-case type is not reconstructed from class name
source navigation never targets the runtime-generated class
```

---

# 8. Typed Multiple Dispatch Implications

Part 06 does not need to introduce new exact-case annotation syntax, but it must preserve architecture necessary for typed multiple dispatch.

Static dispatch may discriminate canonical types such as:

```text
ExactCase(Some, Option<Int>)
ExactCase(Some, Option<String>)
Option<Int>
```

when the call site's static proof contains those types.

Runtime-only dispatch over an erased value knows at least the exact runtime variant but does not necessarily know erased generic arguments. Part 06 must not manufacture false runtime generic evidence to make static overload distinctions available dynamically.

This is a critical soundness requirement.

---

# 9. Core `Result` Representation Analysis

`Result<T,E>` can and should be a built-in/native primitive **semantically** in exactly the same sense as `Option`: a canonical native core ADT declaration with ordinary enum identities.

Physical representation differs.

`Option` can preserve its current nested `Some` depth encoding in the universal `Value`. `Result` has two arbitrary payload-bearing cases, so a universal immediate encoding is not automatically available from the current representation contract.

Part 06 therefore requires:

```text
semantic unification: mandatory
native implementation binding: mandatory if Result is core-native
special compact physical optimization: not mandatory; Part 07
```

A correct Part-06 implementation may use the general ADT object representation for `Result` while still treating it as `@native` if native construction/behavior integration requires it. The important rule is that no semantic consumer can tell whether the physical representation is general or specialized.

---

# 10. Tooling Dependency Graph

The editor work should be sequenced by semantic dependency rather than LSP feature popularity:

```text
source index identity completion
        ↓
exact/family/field occurrence attachment
        ↓
protocol-neutral variant/pattern presentation
        ↓
hover + definition + semantic highlighting
        ↓
semantic rename
        ↓
PatternSpace/residual query surface
        ↓
completion + missing-case action + generate-match
```

Doing completion before source identity and residual-space products would encourage LSP-side semantic reconstruction and should be avoided.

---

# 11. Incremental Dependency Analysis

Part 05 already records enum/associated/type dependencies for matching. Part 06 adds downstream products whose fingerprints must be derived from the same semantic truth.

Examples:

```text
add enum variant
    invalidates:
        EnumInfo fingerprint
        dependent match proof
        residual completion
        reflection enum metadata
        generated-match plan

change payload external label
    invalidates:
        VariantInfo
        source target ranges/labels
        family selector compatibility
        rename/definition metadata
        reflection field metadata

change GADT result type
    invalidates:
        case environment
        exact-case specialization
        match reachability
        hover proof information
        typed dispatch candidates
```

Do not create a giant `SemanticDependency::Part06` catch-all. Use existing declaration/type/associated dependencies or add narrowly defined reflection/source product dependencies only where the existing graph cannot express the read.

---

# 12. Persistence Requirements Analysis

Current `TypeId` and `VariantTypeId` are store-relative compact handles. This is correct inside one snapshot but unsuitable as long-lived public identity.

Part 06 should introduce or formalize stable keys conceptually equivalent to:

```text
StableDeclarationKey
StableVariantKey {
    owner: StableDeclarationKey,
    selector: Selector,
}
StableVariantFamilyKey {
    owner: StableDeclarationKey,
    base: SelectorBase,
}
StableVariantFieldKey {
    variant: StableVariantKey,
    index: u32,
}
StableExactCaseKey {
    variant: StableVariantKey,
    enum_type: StableTypeKey,
}
```

Exact implementation may reuse existing `DeclarationId`/stable module keys where they already satisfy persistence constraints.

Persistent reflection artifacts remap these keys into runtime-local IDs during materialization.

---

# 13. Explicit Non-Goals

Part 06 does not implement or ratify:

```text
new match semantics
pattern guards
runtime GADT proof objects
user-level MatchError
runtime selector-family pattern matching
new exact-case source annotation grammar
new structural family source annotation grammar
arbitrary higher-rank polymorphism
specialized runtime class per generic exact case
per-instance runtime generic tokens
match jump tables / DAG optimization
performance benchmarking or baseline program
native/AOT representation optimization
Bool-as-ADT migration
```

Those are separate language/backend decisions or Part-07 work.

---

# 14. Proposed Work Breakdown

Part 06 should be executed as four coherent phases.

## 06.A — Core Migration and Legacy Removal

```text
native Option/Result/Ordering declaration integration
canonical core VariantIds / exact cases / associated surfaces
runtime representation registration by semantic identity
remove temporary Option semantic bridge
remove legacy core ADT semantic exceptions
canonical exact-case specialization audit
```

## 06.B — Reflection and Runtime Metadata Completion

```text
semantic reflection ontology
canonical exact-case reflection
runtime metadata projection
runtime class ↔ variant metadata bridge
visibility-aware reflection acquisition
stable metadata keys
```

## 06.C — Source Index / LSP / Developer Tooling Completion

```text
VariantFamily/VariantField source targets
pattern occurrence attachment
go to definition
hover
semantic highlighting
rename
residual-space completion
add-missing-cases
Generate Match
family-pattern tooling
rich diagnostics presentation
```

## 06.D — Cross-System Verification, Documentation, Cleanup, Final Audit

```text
cross-module/package tests
incremental invalidation tests
persistence/determinism checks
robustness/fuzz tests
capability-organized semantic/runtime/tooling suite
authoritative language docs sync
compatibility/deletion pass
vertical architecture proof
Part-07 optimization handoff
```

---

# 15. Acceptance Matrix

Part 06 is complete only if these vertical scenarios succeed without semantic exceptions.

## 15.1 Native Option

```phalcom
const x: Option<Int> = Option::Some(42)

const y = match x {
    Some(value) => value
    None => 0
}
```

Must verify:

```text
native declaration -> ordinary EnumInfo
Some/None -> canonical VariantId
constructor -> canonical ExactCase
formal x contract remains Option<Int>
branch observes exact case
match uses normal semantic candidates
VM uses IsVariant/GetVariantPayload-compatible native representation
reflection exposes enum/variant/exact-case metadata
hover/definition/rename use semantic targets
```

## 15.2 Native Result

```phalcom
const result: Result<Int, Error> = Result::Ok(42)

match result {
    Ok(value) => value
    Error(error) => recover(error)
}
```

Must use exactly the same semantic model as source-defined ADTs regardless of physical encoding.

## 15.3 GADT vertical proof

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

Verify:

```text
parser / AST
EnumInfo / CaseTypeEnvironment
constructor typing / canonical ExactCase
pattern candidate resolution
GADT compatibility and branch proof
exhaustiveness
branch result typing
semantic explanation
lowering projection
IsVariant / GetVariantPayload
VM execution
exact-case reflection
pattern hover
go to definition
semantic rename identity
incremental update after case-result edit
```

---

# 16. Risk Analysis

## Risk A — Reflection accidentally becomes a second semantic universe

Mitigation: reflection descriptors are projections over compiler-owned canonical identities and types. No lookup by runtime class name or selector string.

## Risk B — `.class` visibility causes exact-case/class conflation

Mitigation: tests and APIs explicitly distinguish runtime `Class` from `ExactCaseTypeDescriptor` and `VariantDescriptor`.

## Risk C — Core native types retain permanent exceptions

Mitigation: native implementation registry is downstream of ordinary enum semantic construction; architecture searches reject `Some`/`None` semantic string magic.

## Risk D — Tooling recomputes residual cases

Mitigation: expose protocol-neutral semantic residual/completion products from `phalcom-semantic`; LSP only maps them to edits/items.

## Risk E — ExactCase specialization is canonical but not persistently identifiable

Mitigation: keep canonical `TypeId` snapshot-local and add stable key projection for persistent metadata; never serialize raw arena IDs.

## Risk F — Result optimization expands scope

Mitigation: Part 06 requires semantic/native declaration convergence, not a new immediate encoding. Performance/representation optimization is explicitly Part 07.

---

# 17. Requirements Traceability Summary

The implementation plan must cover all requirement families:

```text
CORE       01–09
TYPE       01–06
REFL       01–09
IDX        01–06
HOVER      01–04
COMP       01–06
RENAME     01–04
HL         01–02
DIAG       01–04
INCR       01–04
PERSIST    01–03
DOC        01–02
CLEAN      01–04
TEST       01–03
```

No requirement family is optional for a Part-06 completion claim.

---

# 18. Final Requirements Statement

Part 06 succeeds when the following sentence is literally true of the repository:

> `Option`, `Result`, `Ordering`, user ADTs, GADTs, associated variant families, exact cases, patterns, runtime case identities, reflection, source navigation, editor tooling, and incremental analysis all consume the same canonical declaration/type/proof products, with physical representation and protocol presentation isolated behind explicit projections.

Anything that still has to ask “is this `Some`?” by source spelling, recover a variant from `.class`, rebuild a family from AST text in `phalcom-core`/`phalcom-lsp`, or serialize a runtime/arena ID as language identity is unfinished Part-06 work.
