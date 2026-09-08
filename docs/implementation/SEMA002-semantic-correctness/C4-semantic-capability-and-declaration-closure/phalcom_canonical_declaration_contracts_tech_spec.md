# Phalcom Canonical Declaration Contracts and Evidence Flow — Technical Specification

**Status:** Proposed for implementation  
**Decision:** Option B — canonical contract consolidation  
**Repository:** `aureat/phalcom-lang`  
**Branch:** `main`  
**Grounded HEAD:** `9b30ec324d4361128f285154fe236e25746df750`  
**Grounded date:** 2026-08-28  
**Scope:** `phalcom-semantic` declaration contracts, callable parameters/returns, source attachment, formal body entry, advisory propagation, field observations, editor presentation, and the `phalcom-lsp` inlay-hint consumer.

---

## 1. Executive Decision

Phalcom will adopt one canonical declaration-contract model inside `phalcom-semantic`.

A source or native declaration is lowered once into a canonical contract product. Formal body analysis consumes that product. Advisory analysis may consume formal products one-way as non-authoritative baselines and combine them with runtime-shape observations. Source indexes attach canonical declaration and analysis identities to exact source sites. Editor queries compose those products. `phalcom-lsp` renders the query result and does not reconstruct declaration semantics from the AST.

The normative dataflow is:

```text
source/native declaration
        │
        ▼
canonical declaration contracts
        │
        ├──────────────► dispatch projection
        │
        ▼
formal body analysis
        │
        ├──────────────► diagnostics / explanations
        │
        ▼
formal → advisory projection
        │
        + call-site / flow observations
        ▼
advisory analysis
        │
        ▼
immutable SemanticSnapshot
        │
        ▼
protocol-neutral editor queries
        │
        ▼
phalcom-lsp protocol rendering
```

The reverse directions are forbidden:

```text
advisory ─X─► formal acceptance
LSP AST scan ─X─► semantic declaration truth
dispatch surface ─X─► reconstruct canonical declaration contracts
```

---

## 2. Problem Statement

The current implementation has multiple representations that each contain a subset of the same declaration information:

- `dispatch::CallableSignature` stores parameter and return `TypeKnowledge`.
- `signature::CallableSemanticSignature` stores parameter and return `TypeTerm`, but only exists for completely typed signatures.
- `checker::binding::BindingContractOrigin` mixes declaration role with contract provenance.
- callable body analysis reconstructs parameter bindings from the dispatch surface.
- source indexing separately represents callable parameters as lexical `SourceBindingInfo` records and positional `parameter_name_ranges`.
- advisory analysis keys parameter observations by another `(CallableId, index)` structure and seeds parameters primarily from call-site evidence.
- LSP inlay-hint code reparses the AST to decide whether declarations are explicitly annotated and traverses both lexical bindings and callable parameters.

This produces observable failures:

1. A source-annotated callable parameter can be formally resolved while still failing editor/source recognition.
2. Source annotation provenance is replaced by generic `CallableSignature` evidence at body entry.
3. a parameter body binding is currently created with the whole callable `body_range`, not the parameter source range.
4. `CallableSemanticSignature` disappears entirely when any parameter or return slot is unknown.
5. advisory parameter entry can remain `Unknown` even when formal analysis has a valid source contract.
6. `_field = parameter` cannot reliably infer the unannotated field's advisory shape from `parameter: String`.
7. annotated callable parameters can still receive inlay hints because they participate in both the generic binding pass and callable-parameter pass.
8. `phalcom-lsp` owns `ExplicitAnnotationIndex`, which is source-semantic metadata that belongs in `phalcom-semantic`.

The problem is not that Phalcom has formal, advisory, source, and presentation products. Those products answer different questions and should remain distinct. The defect is that ownership and derivation between them are incomplete.

---

## 3. Verified Current Baseline

The implementation at the grounded HEAD establishes the following constraints.

### 3.1 Canonical semantic signatures are complete-only

`phalcom-semantic/src/signature.rs` currently defines:

```rust
pub struct CallableParameterSemantic {
    pub index: u32,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub rest: RestMode,
    pub ty: TypeTerm,
    pub source: Option<SemanticSourceSpan>,
}

pub struct CallableSemanticSignature {
    ...
    pub parameters: Box<[CallableParameterSemantic]>,
    pub return_type: TypeTerm,
    ...
}
```

`phalcom-semantic/src/db/query.rs::semantic_signature_from_surface` returns `None` when `dispatch::CallableSignature::has_complete_types()` is false. The canonical query therefore cannot publish a partially known callable.

### 3.2 Binding contract origin mixes unrelated axes

`phalcom-semantic/src/checker/binding.rs` currently defines:

```rust
pub enum BindingContractOrigin {
    SourceAnnotation,
    InferredInitializer,
    CallableParameter,
    ContextualBlockParameter,
    PatternBinding,
}
```

`SourceAnnotation` answers where a constraint came from. `CallableParameter` answers what a binding is. They are not mutually exclusive.

### 3.3 Body analysis loses source provenance and range

`phalcom-semantic/src/checker/body.rs` obtains `dispatch::CallableSignature`, then performs:

```rust
for param in &sig.parameters {
    ctx.bind_callable_parameter(
        param.local_name.clone(),
        param.ty.clone(),
        body_range,
    );
}
```

`CheckingContext::bind_callable_parameter_with_causal` converts a known incoming value to:

```rust
TypeKnowledge::assumed(ty, EvidenceOrigin::CallableSignature)
```

and creates a binding contract with `BindingContractOrigin::CallableParameter`.

Thus a source annotation can resolve correctly in declaration analysis but become generic callable-signature evidence at body entry, and the formal binding receives the callable range rather than its parameter range.

### 3.4 Source metadata is too lossy

`phalcom-semantic/src/source_index/scope.rs` currently stores:

```rust
pub struct CallableSourceInfo {
    ...
    pub parameter_name_ranges: Arc<[SourceRange]>,
    pub has_explicit_return_annotation: bool,
}

pub struct SourceBindingInfo {
    ...
    pub declaration_range: SourceRange,
    ...
}
```

There is no canonical parameter source record that simultaneously identifies:

- the semantic parameter slot,
- lexical parameter binding site,
- parameter range,
- name range,
- external-label range,
- annotation range.

There is also no annotation range on ordinary source bindings.

### 3.5 Source attachment still has to reconcile formal bindings after analysis

`CallableSourceAttachment` maps formal `BindingId` values to source sites. The current attachment path may fall back to name/range matching or order-based rebasing. Callable parameters should not require this inference because their declaration identity is already structurally known.

### 3.6 Advisory parameter entry is observation-first

`phalcom-semantic/src/session.rs` builds `seed_bindings` for callable parameters from `parameter_facts` (joined caller contributions). If no observation exists it seeds `AdvisoryFact::unknown()` for that parameter binding.

The crate already provides `advisory_fact_from_formal` and `AdvisoryOrigin::FormalFact`, so the missing behavior is wiring rather than a new semantic authority.

### 3.7 Field write propagation already exists

`phalcom-semantic/src/advisory/analyzer.rs` evaluates assignment RHS facts and notifies `field_observer` for implicit field assignments. `advisory/flow.rs` accumulates them into `field_writes`. Therefore:

```phalcom
setName(name: String) {
    _name = name
}
```

does not require special-case "parameter-to-field" inference. Once `name` has a useful advisory entry fact, ordinary flow can propagate it to `_name`.

### 3.8 LSP annotation suppression is still reconstructed

`phalcom-lsp/src/inlay_hints.rs` defines `ExplicitAnnotationIndex`, walks the AST, and separately records binding, parameter, field, and return annotations.

The canonical hint path then traverses generic source bindings and callable parameters independently. Method/setter/index parameters are both lexical bindings and callable parameters, so this creates overlapping presentation ownership.

---

## 4. Goals

This work MUST provide all of the following.

### G1 — One canonical declaration-contract authority

Source/native declarations lower to a canonical contract product. Dispatch surfaces, formal body contexts, advisory baselines, and editor presentation consume this product.

### G2 — Partial callable contracts are always representable

A valid callable declaration has a canonical semantic signature even when individual slots are unknown, dynamic, unresolved, or not yet inferred.

```phalcom
process(value: String) {
    unknownThing()
}
```

must have a canonical product equivalent to:

```text
parameter[0] = Known(String, source annotation)
return       = Unknown(UnannotatedDeclaration)
```

### G3 — Declaration role and contract provenance are orthogonal

A parameter may simultaneously be:

```text
role  = callable parameter #0
basis = source annotation
```

No enum may require choosing between those two facts.

### G4 — Contracts and current value knowledge remain distinct

For:

```phalcom
let x: Number = 1
```

the system can retain:

```text
contract        = Number
formal current  = Int
advisory shape  = Int
```

The declaration contract constrains values. `TypeKnowledge` represents what formal analysis currently knows about a value.

### G5 — Source identity is exact

Callable parameters have canonical parameter identities and exact source metadata. Body bindings are explicitly associated with parameter identities. Normal execution MUST NOT rely on parameter name + range matching to recover this relationship.

### G6 — Formal authority is preserved

Formal results remain authoritative for checker acceptance. Advisory evidence never changes a formal type, discharges a proof, or suppresses a formal mismatch.

### G7 — Formal facts may seed advisory analysis one-way

If a parameter has a known formal contract and no narrower valid observation, advisory flow can use a shape projected from that formal contract.

### G8 — Call-site observations can refine, not corrupt, formal baselines

For:

```phalcom
consume(value: Animal) {}
consume(Dog.new())
```

the formal contract remains `Animal`; advisory effective shape may refine to `Dog`.

For an incompatible observation, advisory analysis must not silently publish `String | Int` against a formal `String` constraint.

### G9 — Field advisory inference emerges from ordinary flow

When an unannotated field receives a value whose effective advisory shape is known, that shape can propagate through the existing field write pipeline.

### G10 — Explicit annotations suppress ordinary declaration type hints

Suppression affects presentation only. It MUST NOT erase stronger formal or advisory facts used by hover, analysis, dispatch, or explanations.

### G11 — Semantic presentation ownership is singular

Each declaration has exactly one owner for an ordinary type hint:

- ordinary local/top-level binding → binding owner,
- method/setter/index parameter → callable parameter owner,
- closure parameter → lexical binding owner,
- field → field owner,
- callable return → return slot owner.

### G12 — LSP is a protocol renderer

`phalcom-lsp` must not parse source annotations to decide semantic hint ownership or explicitness.

---

## 5. Non-Goals

This work does NOT:

- redesign generic inference;
- redesign the type-relation/prover engine;
- promote advisory field observations into formal field contracts;
- change Phalcom source syntax;
- add closure parameter annotation syntax;
- redesign effect or termination analysis;
- redesign constructor identity;
- change the formal rule that source annotations are assumptions when the checker lacks independent evidence;
- merge formal and advisory lattices;
- make advisory disagreement a second hard diagnostic;
- complete every remaining `phalcom-lsp` retirement task unrelated to these declaration/presentation paths.

---

## 6. Terminology

### 6.1 Declaration contract

A persistent static requirement associated with a declaration slot.

Examples:

```text
parameter x: String  → String contract
field _x: Number     → Number contract
return -> Animal     → Animal contract
let x: Number        → Number binding contract
```

### 6.2 Formal knowledge

The checker's current epistemic statement about a value:

```rust
TypeKnowledge::Known(...)
TypeKnowledge::Unknown(...)
TypeKnowledge::Dynamic(...)
```

Formal knowledge has evidence status and derivation origin.

### 6.3 Advisory fact

A non-authoritative statement about likely or observed runtime value shape. It is useful for tooling and local/interprocedural prediction.

### 6.4 Contract basis

Why a contract slot has its current declaration constraint. This is not the same as `EvidenceOrigin`.

### 6.5 Binding role

What semantic role the lexical binding serves. This is not the same as contract basis.

---

## 7. Canonical Identity Changes

### 7.1 Add `CallableParameterId`

Add to `phalcom-semantic/src/identity.rs`:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CallableParameterId {
    pub callable: CallableId,
    pub index: u32,
}

impl CallableParameterId {
    pub fn new(callable: CallableId, index: u32) -> Self {
        Self { callable, index }
    }
}
```

This becomes the semantic identity used by:

- `CallableParameterSemantic`;
- source parameter metadata;
- checker parameter binding roles;
- advisory parameter contributions;
- advisory parameter summaries;
- editor parameter hint ownership.

`AdvisoryParameterSlot` is retired after migration.

Names are presentation metadata, not parameter identity.

---

## 8. Canonical Contract Model

Create `phalcom-semantic/src/contract.rs`.

### 8.1 `ContractType`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractType {
    Known(TypeTerm),
    Dynamic,
    Unknown(UnknownReason),
}
```

This is declaration-slot state, not `TypeKnowledge`.

### 8.2 `ContractBasis`

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ContractBasis {
    Unspecified,
    SourceAnnotation,
    InitializerInference,
    BodyInference,
    NativeSignature,
    DeclarationSemantics,
    ConstructorSemantics,
    ContextualTyping,
    PatternDecomposition,
}
```

`Unspecified` means the slot exists but no static constraint has yet been established. It is not evidence.

### 8.3 `TypeContract`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeContract {
    pub ty: ContractType,
    pub basis: ContractBasis,
    pub source: Option<SemanticSourceSpan>,
}
```

Required constructors:

```rust
impl TypeContract {
    pub fn unknown(reason: UnknownReason) -> Self;
    pub fn source_annotation(ty: ContractType, source: SemanticSourceSpan) -> Self;
    pub fn inferred(ty: TypeTerm, basis: ContractBasis) -> Self;
    pub fn declaration(ty: TypeTerm) -> Self;
    pub fn native(ty: TypeTerm) -> Self;

    pub fn known_term(&self) -> Option<&TypeTerm>;
    pub fn is_known(&self) -> bool;
    pub fn is_unknown(&self) -> bool;
    pub fn is_dynamic(&self) -> bool;
}
```

### 8.4 Contract-to-formal conversion

The contract module may provide pure conversion helpers, but it must not perform checker acceptance.

For a resolved proper source annotation, body entry can derive:

```text
TypeContract(SourceAnnotation, String)
        ↓
TypeKnowledge::assumed(String, DeveloperAnnotation)
```

For a declaration-semantic guarantee such as a setter `Unit` return:

```text
TypeContract(DeclarationSemantics, Unit)
        ↓
TypeKnowledge::established(Unit, DeclarationSemantics)
```

For a constructor result:

```text
TypeContract(ConstructorSemantics, Self)
        ↓
TypeKnowledge::established(Self, ConstructorSemantics)
```

For an unconstrained parameter:

```text
TypeContract(Unspecified, Unknown(NoTypeEvidence))
        ↓
TypeKnowledge::Unknown(NoTypeEvidence)
```

The conversion preserves why the fact is assumed/established rather than flattening everything to `CallableSignature`.

---

## 9. Canonical Signature Changes

Modify `phalcom-semantic/src/signature.rs`.

### 9.1 Parameter identity and contract

Replace:

```rust
pub index: u32,
pub ty: TypeTerm,
```

with:

```rust
pub id: CallableParameterId,
pub contract: TypeContract,
```

Keep `index()` as a convenience derived from `id.index`.

Target shape:

```rust
pub struct CallableParameterSemantic {
    pub id: CallableParameterId,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub rest: RestMode,
    pub contract: TypeContract,
    pub source: Option<SemanticSourceSpan>,
}
```

The parameter `source` is the parameter/name source span used by declaration-level consumers. Rich source-token metadata lives in `source_index`.

### 9.2 Return contract

Replace:

```rust
pub return_type: TypeTerm,
```

with:

```rust
pub return_contract: TypeContract,
```

Add compatibility-free query helpers:

```rust
pub fn parameter_contract_at(&self, index: usize) -> Option<&TypeContract>;
pub fn return_contract(&self) -> &TypeContract;
pub fn is_complete(&self) -> bool;
```

`is_complete()` is informative only. It MUST NOT control whether the canonical signature exists.

### 9.3 Field contracts

Replace `FieldSemanticSignature::ty: TypeTerm` with:

```rust
pub contract: TypeContract,
```

### 9.4 Declaration contract set

Add:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationContractSet {
    pub declaration: DeclarationId,
    pub callables: BTreeMap<CallableId, CallableSemanticSignature>,
    pub fields: BTreeMap<FieldId, FieldSemanticSignature>,
}
```

This is the canonical source/native declaration contract product for one class/declaration.

---

## 10. Query and Dispatch Ownership

### 10.1 Add a declaration-contract query product

Add a DB key:

```rust
QueryKey::DeclarationContracts(DeclarationId)
```

and typed product:

```rust
SemanticProduct::DeclarationContracts(Arc<DeclarationContractSet>)
```

The query graph becomes:

```text
DeclarationShell
LinkedInterface
Parsed source
       │
       ▼
DeclarationContracts
       │
       ├────────► CallableSignature(callable)
       │
       └────────► DeclarationSurface
                         │
                         ▼
                      dispatch
```

### 10.2 `query_callable_signature`

`query_callable_signature` must read the canonical callable from `DeclarationContracts`.

It no longer:

- reads a dispatch surface as semantic truth;
- calls `semantic_signature_from_surface`;
- blocks because another slot is incomplete.

If the callable declaration exists, the query returns its `CallableSemanticSignature`, including partial slots.

### 10.3 `DeclarationSurface` becomes a projection

`dispatch::CallableSignature` and `DeclarationSurface` may remain optimized dispatch structures, but they are projections of canonical contracts.

Add a pure projection path:

```rust
pub fn dispatch_signature_from_semantic(
    signature: &CallableSemanticSignature,
) -> CallableSignature;
```

The projection maps `TypeContract` to the declaration-level `TypeKnowledge` needed by existing dispatch machinery.

There must be no production path that reconstructs `CallableSemanticSignature` from `DeclarationSurface`.

### 10.4 Inferred return updates are canonical-first

The existing inferred-return refresh currently mutates the dispatch surface first and then reconstructs a canonical signature.

Reverse the direction:

```text
body return summary
      ↓
update CallableSemanticSignature.return_contract
      ↓
update CallableSignatureTable
      ↓
refresh dispatch projection
```

`SurfaceDispatchResolver::update_callable_return_type` becomes an internal projection update or is replaced by a method that accepts the updated canonical signature.

---

## 11. Binding Model

### 11.1 Binding role

Add to `checker/binding.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindingRole {
    Local,
    CallableParameter(CallableParameterId),
    ContextualBlockParameter,
    PatternBinding,
    ForBinding,
}
```

`BindingSeed` gains:

```rust
pub role: BindingRole,
```

`BindingState` gains the same role.

### 11.2 Replace overloaded origin

Replace `BindingContractOrigin` with a checker-resolved form that preserves canonical basis:

```rust
pub struct ResolvedBindingContract {
    pub ty: TypeId,
    pub basis: ContractBasis,
    pub source: Option<SourceRange>,
}
```

`ResolvedBindingContract` is explicitly a lowered checker product derived from canonical `TypeContract`; it is not a second declaration authority.

### 11.3 Reconciliation rule

The current special case:

```rust
BindingContractOrigin::SourceAnnotation
```

becomes:

```rust
ContractBasis::SourceAnnotation
```

The semantic rule is unchanged:

- independent established evidence that conflicts with the contract wins as checker evidence and produces a refutation;
- when actual value evidence is eligible to be assumed and the contract basis is `SourceAnnotation`, the annotation supplies an assumed current type;
- advisory evidence never enters this reconciliation.

### 11.4 Parameter body entry

For a source-annotated parameter:

```phalcom
foo(value: String) {}
```

the body binding must be:

```text
role:
    CallableParameter(foo(_), 0)

resolved contract:
    String
    basis = SourceAnnotation
    source = annotation/name source

current:
    assumed String
    EvidenceOrigin::DeveloperAnnotation
```

It MUST NOT be rewritten to `BindingContractOrigin::CallableParameter` or `EvidenceOrigin::CallableSignature`.

---

## 12. Source Metadata

Modify `source_index/scope.rs`.

### 12.1 Add `CallableParameterSourceInfo`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableParameterSourceInfo {
    pub id: CallableParameterId,
    pub binding_site: SourceSiteId,
    pub range: SourceRange,
    pub name_range: SourceRange,
    pub label_range: Option<SourceRange>,
    pub annotation_range: Option<SourceRange>,
}
```

### 12.2 Enrich callable source info

Replace:

```rust
pub parameter_name_ranges: Arc<[SourceRange]>,
pub has_explicit_return_annotation: bool,
```

with:

```rust
pub parameters: Arc<[CallableParameterSourceInfo]>,
pub return_annotation_range: Option<SourceRange>,
```

Return-hint placement is not redesigned by this work. The compiler editor query may preserve the current canonical placement policy using `callable_body_ranges`/`declaration_range`; the LSP must not rescan syntax to infer annotation presence.

### 12.3 Enrich binding source info

Add:

```rust
pub annotation_range: Option<SourceRange>,
```

to `SourceBindingInfo`.

For a destructuring declaration with one annotation:

```phalcom
let (x, y): (Int, String) = pair
```

each source leaf may retain the same declaration annotation range for presentation suppression.

### 12.4 Builder behavior

`source_index/builder.rs` must assign the same `CallableParameterId` used by the canonical signature:

```text
CallableId + declaration-order parameter index
```

The builder records the lexical parameter `binding_site` directly when declaring the parameter. It must not recover it in a later pass.

---

## 13. Formal Body Analysis

### 13.1 Consume canonical signature directly

`checker/body.rs` must stop using `signature_consumed_by_body` as a dispatch-signature lookup.

Replace it with a canonical body-contract input. Constructor public/body identity normalization may remain explicit, but the returned contract is a `CallableSemanticSignature`.

The DB query that invokes body analysis is responsible for obtaining the canonical signature product first.

### 13.2 Partial parameters

For each canonical parameter:

- known contract → bind a resolved parameter contract;
- unknown contract → bind `Unknown`, retaining `BindingRole::CallableParameter`;
- dynamic contract → bind dynamic;
- invalid/unresolved annotation → retain the unknown reason and causal diagnostics.

Body analysis proceeds even if the return slot is unknown.

### 13.3 Expected return

Only a known/dynamic return contract contributes expected-return checking context. An unknown return contract does not block body analysis.

### 13.4 Parameter source range

The body binding range is the parameter source/name range from the canonical signature/source product, never `body_range`.

---

## 14. Formal Source Attachment

`CallableSourceAttachment` remains a projection/index, not semantic inference.

### 14.1 Parameter attachments are identity-first

For bindings whose role is:

```rust
BindingRole::CallableParameter(parameter_id)
```

attachment uses:

```text
parameter_id
  → CallableParameterSourceInfo.binding_site
```

directly.

No name/range heuristic is allowed for this path.

### 14.2 Other bindings

Locals, pattern bindings, loop bindings, and closure parameters may continue to use the current exact source-site machinery while their identities remain body-local.

### 14.3 Incident semantics

`MissingBinding`/`AmbiguousBinding` remain valid incidents for genuinely unattached non-parameter bindings, but a normal source callable parameter should be mechanically attachable.

---

## 15. Advisory Parameter Model

### 15.1 Retire `AdvisoryParameterSlot`

Replace all advisory `(CallableId, index)` keys with `CallableParameterId`.

### 15.2 Add declaration-boundary state

Add to `advisory/parameters.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryParameterState {
    pub parameter: CallableParameterId,
    pub baseline: AdvisoryFact,
    pub observed: Option<AdvisoryFact>,
    pub effective: AdvisoryFact,
    pub agreement: AdvisoryAgreement,
}
```

`baseline` is the one-way formal projection.

`observed` is the joined call-site fact when available.

`effective` is the safe fact used to seed callable advisory flow.

### 15.3 Baseline construction

For a formally known parameter binding:

```rust
advisory_fact_from_formal(
    store,
    &binding.current,
    AdvisoryOrigin::FormalFact(...)
)
```

For unknown/dynamic/unavailable formal state, baseline is `Unknown`.

### 15.4 Observation refinement

The merge is constraint-aware.

Required cases:

| Formal baseline | Observation | Agreement | Effective |
|---|---|---|---|
| `String` | none | `Unknown` | `String` |
| `Animal` | `Dog` | `MoreSpecific` | `Dog` |
| `Animal` | `Dog | Cat` | compatible refinement | `Dog | Cat` |
| `String` | `Int` | `Incompatible` | `String` |
| unknown | `Dog` | `Unknown` | `Dog` |
| dynamic | `Dog` | `Unknown` | `Dog` or dynamic-policy result; never formalized |

### 15.5 Hierarchy-aware agreement

Extend `advisory/agreement.rs` with hierarchy-aware nominal comparison. `Instance(Dog)` is a refinement of formal `Animal` when `Dog <: Animal`.

Add an explicit `Incompatible` state rather than using `Incomparable` for a known nominal contradiction.

`Incomparable` remains for representations where no safe relation is implemented.

### 15.6 Formal authority

An incompatible advisory observation does not emit a second hard type diagnostic and does not alter the formal contract. The normal formal call checker owns mismatch diagnostics.

---

## 16. Advisory Flow and Field Propagation

`advisory/flow.rs` continues to use a binding environment keyed by `SourceSiteId`.

Before analyzing a callable, the workspace builder creates parameter states and seeds:

```text
parameter source binding site
    → AdvisoryParameterState.effective
```

Then existing expression evaluation applies:

```text
read parameter
    → effective advisory fact
assignment RHS
    → same advisory fact
field_observer
    → field_writes
workspace field fact
```

No special source-annotation-to-field rule is added.

An unannotated field receiving `String` from a `String`-contract parameter therefore acquires advisory `String`.

This remains advisory. Formal field inference policy is out of scope.

---

## 17. Snapshot Publication

The immutable snapshot must expose all canonical contract facts needed by downstream queries.

Add `FieldSignatureTable` to `SemanticSnapshot` if it is not already published.

The snapshot contract layer contains:

```text
CallableSignatureTable
FieldSignatureTable
```

both populated from canonical declaration contract sets.

The dispatch surface is a downstream projection.

Snapshot construction must publish a coherent set from the same DB revision:

```text
contracts
dispatch projection
callable analyses
source index / attachments
advisory workspace
```

A snapshot must never contain a new canonical contract table paired with a stale dispatch projection from another revision.

---

## 18. Editor Type-Hint Query

Extend `phalcom-semantic/src/editor.rs` with protocol-neutral type-hint products.

### 18.1 Types

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EditorTypeHintOwner {
    Binding(SourceSiteId),
    Parameter(CallableParameterId),
    Field(FieldId),
    Return(CallableId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorTypeHint {
    pub owner: EditorTypeHintOwner,
    pub declaration_range: SourceRange,
    pub insert_offset: usize,
    pub formal: Option<FormalPresentation>,
    pub advisory: Option<AdvisoryFact>,
}
```

Only unannotated declarations are returned by the ordinary type-hint query.

### 18.2 Query

```rust
impl<'a> EditorSemanticQuery<'a> {
    pub fn type_hints(
        &self,
        module: &ModuleId,
        visible: SourceRange,
    ) -> Vec<EditorTypeHint>;
}
```

Ownership rules:

- generic binding traversal excludes `MethodParameter`, `SetterParameter`, and `IndexParameter`;
- callable parameter traversal owns those parameter hints;
- closure/for/destructure/local bindings remain in generic binding traversal;
- fields use `FieldId`;
- returns use `CallableId`.

### 18.3 Explicit annotation suppression

The query reads canonical source metadata:

```text
SourceBindingInfo.annotation_range
CallableParameterSourceInfo.annotation_range
FieldSourceInfo.has_explicit_annotation / annotation range
CallableSourceInfo.return_annotation_range
```

It does not accept a `Program` solely for annotation suppression.

### 18.4 Formal/advisory precedence is presentation policy

The editor query returns both channels. `phalcom-lsp` may continue to prefer renderable formal facts over advisory facts and respect `HintPolicy::Stable`/`All`.

---

## 19. LSP Cutover

`phalcom-lsp/src/inlay_hints.rs` canonical request handling becomes:

```text
RequestContext
  → compiler snapshot
  → snapshot.editor().type_hints(module, visible_source_range)
  → policy/filter/render
  → Vec<lsp_types::InlayHint>
```

Delete from the canonical path:

- `ExplicitAnnotationIndex`;
- AST recursive annotation collection;
- generic canonical binding traversal;
- separate canonical callable parameter traversal;
- direct source-index composition of formal/advisory facts.

After legacy retirement permits it, delete obsolete compatibility paths as well.

Formal and advisory return labels must both use `presentation::inlay_type_label(..., return_hint)` so formal return hints render ` -> T`, not `: T`.

---

## 20. Required Semantic Scenarios

The following are acceptance requirements.

### S1 — Unannotated parameter with caller evidence

```phalcom
class A {
    use(x) { x }
}

A.new().use("hello")
```

Expected:

```text
formal parameter = Unknown
advisory observed/effective = String
parameter type hint may show String
```

### S2 — Annotated parameter, no callers

```phalcom
class A {
    use(x: String) { x }
}
```

Expected:

```text
contract basis = SourceAnnotation
formal current = assumed String / DeveloperAnnotation
advisory baseline/effective = String
no ordinary parameter inlay
```

### S3 — Annotated supertype, narrower caller observation

```phalcom
class Animal {}
class Dog is Animal {}

class Consumer {
    use(value: Animal) {
        value
    }
}

Consumer.new().use(Dog.new())
```

Expected:

```text
formal contract = Animal
observed = Dog
agreement = MoreSpecific
effective advisory = Dog
formal remains Animal
no parameter inlay because annotation is explicit
```

### S4 — Incompatible caller

```phalcom
class Consumer {
    use(value: String) {}
}

Consumer.new().use(1)
```

Expected:

```text
formal call diagnostic = argument mismatch
advisory observed = Int
agreement = Incompatible
effective parameter seed does not become String | Int
```

### S5 — Annotated parameter flows to unannotated field

```phalcom
class User {
    _name

    setName(name: String) {
        _name = name
    }
}
```

Expected:

```text
formal name = String
advisory name baseline/effective = String
field advisory _name = String
no name parameter hint
field may show inferred String hint
```

### S6 — Partial callable

```phalcom
class A {
    run(value: String) {
        unknownThing()
    }
}
```

Expected:

```text
canonical callable signature exists
parameter contract = String
return contract = Unknown(...)
body binding = String assumption
editor recognizes annotation
```

### S7 — Annotated local with more-specific initializer

```phalcom
let x: Number = 1
```

Expected:

```text
binding contract = Number
formal current may preserve Int evidence
advisory = Int
no ordinary binding inlay
```

### S8 — Unannotated destructuring

```phalcom
let pair = (1, "x")
let (x, y) = pair
```

Expected:

```text
x/y remain generic lexical binding hint owners
parameter-owner changes do not suppress destructuring hints
```

### S9 — Annotated destructuring

```phalcom
let (x, y): (Int, String) = pair
```

Expected:

```text
each leaf source binding records the declaration annotation range
no x/y ordinary inlays
formal facts remain queryable
```

### S10 — Setter/index parameters

Annotated and unannotated setter/index parameters follow the same identity, contract, attachment, advisory, and hint rules as ordinary method parameters.

---

## 21. Incremental Semantics

The DB dependency graph must reflect the new authority.

Required dependency direction:

```text
ParsedModule / LinkedInterface / DeclarationShell
            ↓
DeclarationContracts
       ┌────┴────────────┐
       ▼                 ▼
CallableSignature   DeclarationSurface
       │                 │
       └──────┬──────────┘
              ▼
          CallableBody
              │
              ▼
    SourceFormalAttachment
              │
              ▼
        AdvisoryCallable
              │
              ▼
         AdvisoryModule
```

A parameter annotation edit must invalidate:

- the owning declaration contract set;
- the affected callable signature;
- body analyses that consume that signature;
- downstream formal source attachment;
- advisory callable/module products;
- editor products derived from the new snapshot.

It must not require unrelated declarations or callables to recompute if their dependency fingerprints are unchanged.

Fingerprints MUST include:

- contract type state;
- contract basis;
- canonical parameter identity/index;
- relevant source annotation presence/range in presentation fingerprints;
- effective advisory parameter facts.

---

## 22. Diagnostics and Explanations

The contract model should improve explanation fidelity without redesigning the explanation engine.

A mismatch such as:

```phalcom
foo(value: String) {}
foo(1)
```

must be able to preserve:

```text
expected String
  because callable parameter #0 has a source annotation

actual Int
  because argument is an integer literal
```

`ContractBasis::SourceAnnotation` and the contract source span provide the expected-side provenance.

No diagnostic should need to inspect the AST to learn whether the expected type was source-declared.

---

## 23. Compatibility and Migration Policy

This is an architectural cutover, not a permanent dual model.

Temporary adapters are allowed only inside the implementation sequence and must have deletion steps in the same plan.

Specifically:

- `semantic_signature_from_surface` must be deleted when `DeclarationContracts` becomes authoritative.
- `CallableSignature::has_complete_types()` may remain as a diagnostic/convenience method only if it still has a real consumer; it must not gate canonical signature publication.
- `AdvisoryParameterSlot` must be deleted after migration to `CallableParameterId`.
- `ExplicitAnnotationIndex` must be deleted from the canonical LSP path.
- body analysis must not retain a fallback that reconstructs parameter contracts from dispatch once canonical signature queries are wired.

---

## 24. Test Strategy

Testing is layered.

### 24.1 Unit tests

Add focused tests for:

- `TypeContract` constructors and conversions;
- source-annotation assumption semantics;
- declaration role/basis independence;
- hierarchy-aware advisory refinement;
- incompatible advisory observations;
- dispatch projection of partial canonical contracts.

### 24.2 Semantic integration tests

Extend/create tests under:

```text
phalcom-semantic/tests/semantic/integration/
```

for:

- partial callable signatures;
- source parameter metadata;
- parameter-to-binding identity attachment;
- formal-to-advisory parameter seeding;
- field propagation;
- incremental invalidation;
- editor type-hint ownership.

### 24.3 LSP tests

Update `phalcom-lsp/tests/stage6_inlay_hints.rs` and professional presentation tests to assert:

- annotated bindings/parameters/fields/returns do not receive ordinary type hints;
- unannotated parameters receive one hint, not two;
- formal return hints use arrow formatting;
- an unannotated field can display advisory `String` after assignment from `name: String`.

### 24.4 Architecture tests

Add/extend boundary checks so the LSP cannot regain source-annotation semantic ownership.

At minimum, the final canonical `inlay_hints.rs` must not define or reference:

```text
ExplicitAnnotationIndex
collect_statement_annotations
collect_pattern_names
```

for canonical semantic hinting.

---

## 25. Success Criteria

The implementation is complete only when all are true:

1. `CallableSemanticSignature` exists for partial source signatures.
2. callable parameter identity is canonical and shared across signature, source, checker, advisory, and editor products.
3. source annotation basis survives into body binding state.
4. parameter body bindings use parameter source range, not callable `body_range`.
5. source parameter formal attachment is identity-first.
6. advisory parameter entry uses formal baselines where available.
7. compatible caller facts refine advisory shape.
8. incompatible observations do not union into a formally invalid shape.
9. annotated parameter → unannotated field propagation produces advisory field type.
10. explicit annotations suppress ordinary type hints without erasing semantic facts.
11. each declaration has one type-hint presentation owner.
12. `phalcom-lsp` no longer reconstructs annotation suppression from AST for the canonical path.
13. incremental dependency tests show parameter annotation edits invalidate only the necessary semantic closure.
14. all targeted semantic and LSP suites pass.
15. no temporary reverse-authority adapter remains.

---

## 26. Final Architecture

After this change the coexistence of several semantic products is intentional and ordered:

```text
Canonical identity/source structure
          │
          ▼
Canonical declaration contracts
          │
          ├──────────────► dispatch index
          │
          ▼
Formal callable analysis
          │
          ├──────────────► diagnostics/explanations
          │
          ▼
Formal source projection
          │
          ▼
Advisory baseline + observations
          │
          ▼
Editor query composition
          │
          ▼
LSP protocol rendering
```

The system does not need one giant semantic struct. It needs one authority for each question and explicit one-way derivations between authorities. This specification establishes those boundaries for declaration types, callable parameters, returns, advisory parameter flow, field observations, and type-hint presentation.
