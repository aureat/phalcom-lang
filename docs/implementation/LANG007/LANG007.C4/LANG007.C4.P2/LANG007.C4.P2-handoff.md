---
id: LANG007.C4.P2-handoff
kind: handoff
plan: LANG007.C4.P2
checkpoint: LANG007.C4
created: 2026-10-04
for: next implementer (taking over for LANG007.C4.P3)
---

# LANG007.C4.P2 — Handoff (P2 Complete; P3 Next)

This handoff records established work through complete closure of `LANG007.C4.P2`.
All tasks (T0–T11) and gates (G1–G3) are IMPLEMENTED and FOCUSED_TESTED.
The next plan is `LANG007.C4.P3` (First-class Pack and call/rest convergence).

**Current plan status**: COMPLETE / IMPLEMENTED / FOCUSED_TESTED

---

## 1. Current revision and state

Branch: `main`.

```text
status:       COMPLETE
completion:   IMPLEMENTED
verification: FOCUSED_TESTED
active_plan:  LANG007.C4.P3
latest_done:  LANG007.C4.P2
next_task:    begin LANG007.C4.P3 (First-class Pack and call/rest convergence)
```

Task ledger:

```text
T0  COMPLETE — P2 takeover state; 0 stale result-before-generics; checkpoint initialized
T1  COMPLETE — StaticValueDomain, resolve_static_domain, validate_proper_type; 6/6 tests
T2  COMPLETE — TypeDefinitionDef, StaticParameterSyntax, StaticDefinitionBody, GenericArgumentSyntax::Value; 288/288 AST tests
T3  COMPLETE — StaticParameter, StaticDefinitionSignature, StaticDefinition, StaticDefinitionTable, resolve_static_definition; 5/5 tests
T4  COMPLETE — StaticValueStore, StaticValueId, StaticValueRef, primitive StaticValue variants; focused test passed
T5  COMPLETE — immutable structural Tuple/Record/Data/Variant values, type projection and relation check; focused test passed
T6  COMPLETE — checked static bodies, module binding handoff, checked-body query/dependencies, session composition, phase reads/writes, and exact intrinsic projection; focused gates passed
T7  COMPLETE — explicit-frame static evaluator foundation, control flow, checked applications, Data/Variant identity, budget/cancellation; focused tests passed
T8  COMPLETE — StaticConstantId(SymbolId), exact initializer span location, ordinary checker static body, SCC cycle detection, topological evaluation, content-recursive fingerprinting; 9/9 focused tests passed
T9  COMPLETE — MaterializationDescriptor, MaterializationCapability, M2 primitive VM materialization (Unit, Bool, Small/Large Int, Float exact bits, String, Symbol), deferred structural const handling; 16/16 tests passed
G1  COMPLETE — End-to-end VM execution gate passed (16/16 tests in core static_const)
T10 COMPLETE — Incremental stabilization, cold/warm parity, same-module constant isolation, budget termination (9/9 tests pass)
G2  COMPLETE — C1–C3 compatibility gate passed (365 tests passed across type_alias, adts, and core algebraic_data)
G3  COMPLETE — Affected crates check passed (cargo check -p phalcom-semantic -p phalcom-core clean)
T11 COMPLETE — Plan closure, records updated, and handoff to P3 authored
```

---

## 2. Established T1–T3 interfaces (do not redesign)

### 2.1 Static domain model (`phalcom-semantic/src/static_domain.rs`)

```rust
pub enum StaticValueDomain {
    Kind(KindId),
    ValueType(TypeId),
}
impl StaticValueDomain {
    pub fn is_kind(&self) -> bool;
    pub fn is_value_type(&self) -> bool;
    pub fn as_kind(&self) -> Option<KindId>;
    pub fn as_value_type(&self) -> Option<TypeId>;
    pub fn validate(&self, store: &TypeStore) -> bool;
    pub fn to_type_member_domain(&self) -> Option<TypeMemberDomain>;
    pub fn from_type_member_domain(domain: TypeMemberDomain) -> Self;
}

pub fn resolve_static_domain(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    syntax: &StaticDomainSyntax,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormationOutcome<StaticValueDomain>

pub fn validate_proper_type(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    ann: &TypeAnnotation,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormationOutcome<TypeId>
```

**Rules**:
- `Type` in syntax → `Kind(KindId::TYPE)`.
- Any named kind (e.g. `Shape`) → `Kind(k)` via `TypeStore::static_domain_kind`.
- Proper-type names (e.g. `Int`, `Bool`) → `ValueType(type_id)`.
- `is_kind()` is the gate for TypeParameterId allocation and C3 generic-signature inclusion.

### 2.2 AST generalized type definition (`phalcom-ast/src/ast.rs`)

```rust
// Parameters
pub struct StaticParameterSyntax {
    pub name: String,
    pub domain: Option<StaticDomainSyntax>,  // None => Type by default
    pub range: SourceRange,
}

// Body
pub enum StaticDefinitionBody {
    Expression(Box<Expr>),
    Block(LexicalBlock),
}

// Definition
pub struct TypeDefinitionDef {
    pub name: String,
    pub name_range: SourceRange,
    pub parameters: Vec<StaticParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    pub result_domain: Option<StaticDomainSyntax>,  // None => Kind(Type) default
    pub body: StaticDefinitionBody,
    pub implementation: ImplementationKind,
    pub implementation_range: Option<SourceRange>,
    pub range: SourceRange,
}

// Generic arguments
pub enum GenericArgumentSyntax {
    Type(TypeAnnotation),
    Value(Expr),          // new for value static arguments
}
```

**Grammar**: `type Name<P: Domain, ...>: ResultDomain = body` or
legacy `type Name = TypeAnnotation`.

### 2.3 Semantic static definition model (`phalcom-semantic/src/static_definition.rs`)

```rust
pub struct StaticParameter {
    pub name: String,
    pub domain: StaticValueDomain,
    pub index: u32,
    pub type_parameter_id: Option<TypeParameterId>,  // SOME only for Kind lane
    pub range: SourceRange,
}

pub struct StaticDefinitionSignature {
    pub parameters: Box<[StaticParameter]>,
    pub result_domain: StaticValueDomain,
    pub generic_signature: Option<GenericSignature>,  // kinded lane only
    pub where_clause: Option<WhereClauseSyntax>,
}

pub enum StaticDefinitionBodyRepr {
    TypeForm(TypeAnnotation),
    Expression(Expr),
    Block(LexicalBlock),
}

pub struct StaticDefinition {
    pub id: DeclarationId,        // identity is DeclarationId, not range
    pub signature: StaticDefinitionSignature,
    pub body: StaticDefinitionBodyRepr,
    pub source_range: SourceRange,
}

pub struct StaticDefinitionProduct {
    pub definition: Arc<StaticDefinition>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
    pub alias_info: Option<Arc<TypeAliasInfo>>,   // Some iff Type-valued, kinded-only, TypeForm body
}

pub struct StaticDefinitionTable { ... }  // BTreeMap<DeclarationId, Arc<StaticDefinitionProduct>>

pub fn resolve_static_definition(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    def: &TypeDefinitionDef,
    module_id: &ModuleId,
    dependencies: Box<[DeclarationId]>,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> StaticDefinitionProduct
```

**Rules** (FIXED — do not redesign):
- `TypeParameterId` allocated ONLY for `StaticValueDomain::Kind` parameters.
- Ordinary value parameters (e.g. `N: Int`, `Flag: Bool`) NEVER enter TypeStore parameter identity.
- Where-clause constraints mentioning value parameters are diagnosed with
  `DiagnosticCode::AnnotationUnsupported` and filtered out before `resolve_generic_signature`.
- `TypeAliasInfo` projected only when: result domain is `Kind(...)`, body is `TypeForm`,
  AND all parameters are kinded. Mixed or value-result definitions have `alias_info: None`.
- For type lambda bodies: `TypeAliasInfo.kind` stores the full constructor arrow kind;
  domain check uses `lambda.result_kind` (not the outer arrow kind).
- Definition identity is `DeclarationId`; source ranges are provenance only.

### 2.4 DB product integration

```rust
// phalcom-semantic/src/db/key.rs
QueryKey::StaticDefinition(DeclarationId)

// phalcom-semantic/src/db/product.rs
SemanticProduct::StaticDefinition(Arc<StaticDefinitionProduct>)
fn as_static_definition(&self) -> Option<&Arc<StaticDefinitionProduct>>
```

Routing in `db/mod.rs` and `session.rs`: `query_key_module` and
`query_key_module_for_worklist` handle `StaticDefinition` keys.

Fingerprints in `db/fingerprint.rs`:
- `static_definition_input_fingerprint`
- `static_definition_product_fingerprint`

### 2.5 Snapshot integration

```rust
// phalcom-semantic/src/snapshot.rs
pub struct SemanticSnapshot {
    ...
    pub static_definitions: Arc<StaticDefinitionTable>,
    ...
}
impl SemanticSnapshot {
    pub fn with_static_definitions(self, table: Arc<StaticDefinitionTable>) -> Self;
    pub fn static_definitions(&self) -> &StaticDefinitionTable;
    pub fn static_definition(&self, id: &DeclarationId) -> Option<&Arc<StaticDefinitionProduct>>;
}
```

In `session.rs`, `recompute_structural_shards` tracks `static_definitions_table`.
The old ad-hoc alias lowering loop has been replaced by unified
`resolve_static_definition` + `query_static_definition`. Definitions with
`alias_info` are still projected into `type_aliases` for C3 compatibility.

---

## 3. Invariants established (T1–T4 scope)

- **P2-INV-03**: kinded terms and ordinary compile-time values are distinct domains ✓
- **P2-INV-04**: C3 Type/static-term identity reused, not duplicated ✓
- **P2-INV-11**: definitions are declaration-identified ✓
- **P2-INV-12**: ordinary static value parameters do not masquerade as TypeParameters ✓
- **P2-INV-13**: existing Type aliases retain optimized C3 semantics ✓
- **P2-INV-14**: StaticDefinition is the generalized source-definition authority ✓
- **P2-INV-02**: StaticValue nodes contain no runtime `Value`/`ObjRef`/VM identity ✓
- **P2-INV-05**: ordinary static Int is exact/unbounded ✓
- **P2-INV-06**: Float representation bits are separate from Telos equality/hash ✓
- **P2-INV-07**: String equality uses content ✓
- **P2-INV-08**: Symbol identity uses spelling ✓
- **P2-INV-09**: published refs are store-qualified ✓
- **P2-INV-10**: store lineage is append-only across clones ✓

---

## 4. Completed T4 — StaticValueStore and primitive values

### T4 contract and implemented surface

```rust
// In phalcom-semantic/src/static_value/ (new module tree)

pub struct StaticValueStoreId(u64);   // global atomic counter, like TypeStoreId
pub struct StaticValueId(u32);        // arena index within one store

pub struct StaticValueRef {
    store: StaticValueStoreId,
    value: StaticValueId,
}

pub struct StaticValueStore {
    id: StaticValueStoreId,
    values: Vec<StaticValueData>,
    // interner maps for content-equal nodes (bool, small int, etc.)
    // string/symbol intern tables (content-keyed)
}

pub enum StaticValueData {
    Unit,
    Bool(bool),
    Int(StaticInt),
    Float(StaticFloat),
    String(StaticStringId),
    Symbol(StaticSymbolId),
    Type(TypeId),        // requires proper Type validation
    StaticTerm(TypeId),  // other C3 terms
    Tuple(StaticTupleId), Record(StaticRecordId), Data(StaticDataValueId), Variant(StaticVariantValueId)
}

pub enum StaticInt {
    Small(i64),
    Large(Arc<num_bigint::BigInt>),
}

pub struct StaticFloat { bits: u64 }
```

**Lifetime pattern** (mirrors TypeStore — FIXED):
```text
SemanticWorkspaceSession -> Arc<StaticValueStore>  (mutation via Arc::make_mut / persistent clone)
SemanticSnapshot         -> Arc<StaticValueStore>  (immutable)
```

**Required semantics**:
- `NaN != NaN` (Telos float equality; NOT bitwise equality)
- `+0.0 == -0.0` (Telos equality)
- Signed-zero bits **preserved** for materialization
- `StaticFloat` derives `PartialEq`/`Eq`/`Hash` by **bit identity** for store interning;
  Telos semantic equality is a **separate function** (not derived)
- `StaticInt::Small(i64)` normalized back from Large when representable
- Exact/unbounded — no semantic i64 ceiling
- No `phalcom_core::Value`, `ObjRef`, or VM `Symbol` anywhere in semantic types

**Required API**:
- `StaticValueStore::new() -> Self` (new unique `StaticValueStoreId`)
- `intern_bool`, `intern_int`, `intern_float`, `intern_string`, `intern_symbol`
- `intern_type(TypeId, &TypeStore) -> Result<StaticValueRef, KindError>`
  (validates proper type before admission)
- `intern_static_term(TypeId) -> StaticValueRef`
- `get(ref: StaticValueRef) -> &StaticValueData` (validates store identity)
- `domain_of(ref: StaticValueRef) -> StaticValueDomain`
- Float semantic equality helper: `static_float_eq(a: &StaticFloat, b: &StaticFloat) -> bool`
  (uses `+0 == -0`, `NaN != NaN` semantics — NOT bitwise)

**Snapshot integration** (same as TypeStore):
```rust
// phalcom-semantic/src/snapshot.rs
pub struct SemanticSnapshot {
    ...
    pub static_values: Arc<StaticValueStore>,
    ...
}
```

**Session integration**:
```rust
// phalcom-semantic/src/session.rs
// SemanticWorkspaceSession owns Arc<StaticValueStore>
// mutated via Arc::make_mut during recompute phases
```

**Cargo.toml addition** (`phalcom-semantic/Cargo.toml`):
```toml
num-bigint = { workspace = true }
```
(`num-bigint` is already a workspace dependency; just add the reference.)

### T4 forbidden
- Per-query arenas referenced by DB products
- Naked reusable raw IDs (always store-qualified refs for published products)
- Runtime `LargeInt`/`Value` reuse
- Derived bitwise `Float` `Eq` as Telos language `Eq`
- VM Symbol reuse

### T4 required tests (`phalcom-semantic/tests/semantic/foundations/static_values.rs`)
```text
unit value exists
bool true/false
small int normalize back from large
large int exact (> i64::MAX)
negative large int
float finite
float NaN != NaN (Telos equality)
float +0.0 == -0.0 (Telos equality)
float +0.0 and -0.0 preserve distinct bits
float infinity
string utf-8 content semantics
symbol spelling semantics
Type proper type validation
StaticTerm non-proper admitted
Type/StaticTerm distinction enforced
store clone preserves refs (IDs stable)
unrelated store rejects foreign refs
append preserves previous IDs
```

### T4 implementation result

Implemented in `phalcom-semantic/src/static_value/{mod.rs,store.rs}` and
published through `SemanticSnapshot::static_values`; the session owns the same
append-only store lineage. `TypeStore::nominal_type_id` supplies read-only
canonical built-in type lookup for domain reporting.

The domain lookup takes the paired type store:

```rust
domain_of(reference, &type_store) -> Result<StaticValueDomain, StaticValueStoreError>
```

This resolves primitive built-in domains through the existing `TypeStore`
authority. `content_fingerprint` preserves exact Float bits; `semantic_hash`
normalizes signed zero and NaN consistently with `semantic_eq`.

Focused gate: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::static_values -- --nocapture`
selected one test and passed. T4 is COMPLETE / IMPLEMENTED / FOCUSED_TESTED.

### Completed T5 — structural StaticValues and type projection

T5 adds store-local IDs for Tuple, Record, Data, and Variant structural nodes.
Tuple children preserve order. P2 Record keys remain strings, are sorted for
canonical construction, and reject duplicates. Data payloads are normalized
through exact canonical `DataComponentId`s and retain the data declaration and
constructor identities. Variants retain the exact `VariantId` and require the
corresponding exact-case result type. Each constructor accepts only refs from
the same store that already exist, allowing shared immutable DAGs without a
backpatch route to cycles.

`static_value_type` uses TypeStore for Tuple/Record formation and checks
Data/Enum metadata before projecting stored canonical result types.
`check_value_type` delegates to the existing assignability relation. The T5
focused test selected 1 test and passed:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values_structural -- --nocapture
```

`domain_of(reference, &TypeStore)` is the primitive-only T4 API. Structural
type-domain consumers must call `static_value_type` with snapshot Data/Enum
tables. This keeps the TypeStore-only function from guessing canonical
declaration products.

---

### T6 complete — checked static bodies and exact operations

T6 is COMPLETE / IMPLEMENTED / FOCUSED_TESTED. The checked-body architecture
and handoff decisions remain as recorded in the checkpoint consultation ledger.

Implemented:

- ModuleStaticBindingIndex publishes final ordinary-checker facts with stable
  SymbolId, owner body and BindingId, checked state, and static or runtime
  phase. Linked imported names retain the target symbol identity.
- analyze_static_body_with_module_bindings seeds fresh local identities,
  records stable const reads, and retains the module-index fingerprint.
- Ordinary expression analysis publishes both binding_read and binding_write;
  static bodies reject outer runtime reads and writes.
- CheckedStaticBody has its own SemanticDb key/product, source and dependency
  fingerprints, dependency edges, cache validation, and session composition.
  The public static-definition view references this checked product.
- The session composes bodies only for active source modules with ordinary
  binding products. Canonical Universe definitions without such a handoff
  remain uncomposed in this pass.
- In a value-result body, an ambiguous unqualified name that resolves to an
  existing seeded binding is checked as an ordinary variable. Kind-result
  type forms still use type-form checking.
- The selected canonical Number.+(_) operation appears in checked-body
StaticOperationResolution; selector spelling alone does not grant
authority.

Focused verification completed:

    RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic — passed
    RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib checker::static_body::tests -- --nocapture — 4 passed
    RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::static_definition_bodies -- --nocapture — 3 passed
    RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_ -- --nocapture — 19 passed
    RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::static_definition_bodies -- --nocapture — 3 passed

The three integration cases confirm checked-body session composition,
same-module and imported canonical const identity, exact operation projection,
and provider-change invalidation with body recomputation. The semantic check
reports inherited deprecated
Statement::TypeAlias compatibility warnings.

### T7 complete — bounded explicit-frame StaticEvaluator

T7 is IMPLEMENTED / FOCUSED_TESTED. The evaluator in
`phalcom-semantic/src/static_eval/` consumes `CheckedStaticBody`, uses an
explicit work stack, preserves mixed kinded/value parameter positions, and
fails closed for unsupported syntax. It supports the focused control, local,
application, constructor, TypeForm-result, trace, cancellation, and resource
cases summarized in the T7 closure at the end of the checkpoint and walkthrough.
The final focused command passed `integration::static_definition_bodies` 3/3.

`QueryBudget` has no separate value-node quota. The available discriminator
charges Data constructor payload work before interning and confirms the node is
absent after exhaustion. Cancellation is covered by pre-cancelled evaluation.
Do not broaden this into a new quota or TypeStore mutation model under T7.

### T8 architecture decision — constant-owned initializer source/result

The T8 consultation resolved that `query_static_constant(SymbolId)` owns a
constant-specific initializer source/check result. Do not fabricate a
`StaticDefinition` or key this product by `StaticBodyId(DeclarationId)`.
Extend `ModuleBindingFact` with the exact const declaration/initializer source
site when building it alongside ordinary `<main>` analysis. Use that site and
stable `SymbolId` to retrieve syntax from `ParsedModuleUnit`; do not perform a
fresh name lookup. `CallableAnalysis` facts alone do not provide the initializer
AST and expression-tree relation needed to build the evaluator body.

Check the exact initializer with the shared ordinary checker in a static-body
context, consuming canonical module bindings and the existing dispatch/semantic
products. This is another analysis invocation, but not another checker or
resolution authority. The new constant-owned checked input/result should retain
dependencies, evaluation outcome, and a targeted source fingerprint. Change one
of two const initializers in a module and prove only that constant's product
invalidates while the other is reused; verify checked reads retain stable
`SymbolId`s. This requires the appended T8 plan amendment.

### Next action

Read T8 in the active plan, the T8 amendment near its end, and the adviser
record in `LANG007.C4-CHECKPOINT.md`. Implement T8.1/T8.2 first: stable
`StaticConstantId(SymbolId)` identity and exact initializer source-site capture.
No T8 source implementation has started. Keep T9 blocked until T8 acceptance.

## 5. T7–T11 plan summary

### T6 — CheckedStaticBody and exact StaticOperationResolution

New module `phalcom-semantic/src/static_intrinsic.rs`:
- `StaticIntrinsicRegistry` built from canonical Universe callable identity
- `StaticOperationResolution { expression: ExpressionId, callable: CallableId, operation: StaticIntrinsicOperation }`

Static body checking uses the **normal semantic checker** in a static-phase environment:
- Static-readable: static parameters, `const` names, type names
- Phase error: runtime `let`, `let mut`, `self`, field access, runtime closure captures
- Authorization: exact `CallableId` identity; selector spelling alone grants NO authority

STOP if: exact selected callable identity is lost before body publication.

### T7 — Bounded explicit-frame StaticEvaluator

New module `phalcom-semantic/src/static_eval/`:
- `machine.rs`, `frame.rs`, `error.rs`
- No host-recursive guest computation (Rust call depth does not grow with guest nesting)
- Budget checkpoints before every expression step, frame push, expensive intrinsic
- Local slots from pre-published checked binding layout
- `StaticEvalOutcome`: `Reduced(StaticValueRef)`, `Symbolic`, `Invalid`, `Recursive`, `BudgetExceeded`, `Cancelled`
- P2 supports nested non-recursive applications; re-entry → `Recursive`

STOP if: implementation duplicates checker facts with AST-specific special cases.

### T8 — StaticConstant queries and the semantic DB

New `phalcom-semantic/src/static_constant.rs`:
- `QueryKey::StaticConstant(DeclarationId)` / `SemanticProduct::StaticConstant`
- Fingerprint: content-based (not value-ref ID)
- Dependency edges: `StaticConstant(A) -> StaticConstant(B)` and `-> StaticDefinition(F)`
- Cycles: deterministic via query/dependency authority (no stack overflow)
- LKG may remain for presentation; current product must not be stale compiler truth

STOP if: SemanticDb cannot model const cycles without a parallel cache.

### T9 — Materialization descriptors and M2 primitive runtime vertical

New `phalcom-semantic/src/materialization.rs`:
- `StaticValueRef -> MaterializationDescriptorRef`
- Descriptor capability: `P2PrimitiveMaterializable` vs `DeferredStructural`
- Compiler lookup: consume `ModuleLoweringSemantics` static constant map
- M2 primitive materialization: Unit/Bool/Int(small→i64, large→BigInt)/Float(exact bits)/String(content)/Symbol(spelling)
- LargeInt GC rooting behavioral test (behavioral, not just inspection)
- No initializer bytecode; no runtime const slot; no reinvocation

STOP if: BigInt materialization opens an unrooted lifetime.

### G1 — Closed evaluator/materialization vertical gate

Must pass before T10:
```telos
type IsZero<N: Int>: Bool = N == 0
type Choose<Flag: Bool, A: Type, B: Type>: Type = if Flag { A } else { B }
const Answer = 40 + 2
const Enabled = true
const Big = <integer beyond i64>
const Message = "hello"
const Key = #name
const A = 10
const B = A + 2
const Good = if false { runtimeOnly() } else { 1 }  // must reduce
```
Plus: user overload of `+` with matching selector MUST NOT execute statically.

### T10 — Resource, incrementality, performance

- Every evaluator path charges query budget
- BigInt adversarial preflight: huge shift, huge exponent, rapid multiplication
- StaticValue store lineage/sharing verified
- Evaluator: indexed locals, explicit frame vector, no body clone per application
- Incremental scenarios: unrelated edit reuses unaffected products; value change 10→20 invalidates dependent

### G2 — C1–C3 compatibility gate

Legacy aliases, generic aliases, type lambdas, Kind/StaticDomain, TypePack,
RecordRow, static type members, generic substitution/inference, alias cycles —
all must remain behaviorally unchanged.

### G3 — Affected-crate stabilization

```sh
cargo test -p phalcom-ast
cargo test -p phalcom-semantic
cargo test -p phalcom-modules
cargo test -p phalcom-core
# and when touched:
cargo test -p phalcom-type-meta
cargo test -p phalcom-lsp
cargo test -p phalcom-repl
```

Classify failures A/B/C/D. D failures → record and continue; never weaken assertions.

### T11 — Completion records

Update checkpoint to `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
Write final `LANG007.C4.P2-walkthrough.md` and replace this interim handoff
with the final `LANG007.C4.P2-handoff.md` (P3/P4 handoff content per plan §T11).

---

## 6. What must not be redesigned

| Item | Rule |
|---|---|
| `StaticValueDomain::Kind` vs `ValueType` distinction | FIXED |
| `TypeParameterId` allocated only for kinded parameters | FIXED |
| `DeclarationId` as definition identity (not range/hash) | FIXED |
| `TypeAliasInfo` projection rules (kinded-only, TypeForm, Kind result) | FIXED |
| Session's unified alias/static-definition loop | FIXED |
| `StaticValueStore` COW lineage model (mirrors TypeStore) | FIXED — to be implemented in T4 |
| Store-qualified `StaticValueRef` for published products | FIXED |
| No `phalcom_core::Value`/`ObjRef`/VM `Symbol` in semantic nodes | FIXED |
| Explicit-frame evaluator (no host-recursive guest eval) | FIXED |
| `QueryBudget`/`CancellationToken` as resource authority | FIXED |
| No initializer replay at materialization | FIXED |
| P8 owns structural/artifact materialization | FIXED |
| Content-based constant fingerprints (not store insertion order) | FIXED |

---

## 7. Baseline failure classification (inherited from P1)

All pre-existing failures are Category D. No new failures were introduced
in T0–T4. P1 walkthrough documents the baseline evidence.

---

## 8. Established T8–T9 interfaces (do not redesign)

### 8.1 Static constant pipeline (`phalcom-semantic/src/static_constant.rs`)

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticConstantId(pub SymbolId);

pub struct StaticConstantProduct {
    pub id: StaticConstantId,
    pub name: Arc<str>,
    pub outcome: StaticEvalOutcome,
    pub value: Option<StaticValueRef>,
    pub lkg_value: Option<StaticValueRef>,
    pub ty: TypeId,
    pub diagnostics: Vec<Diagnostic>,
    pub dependencies: Vec<SemanticDependency>,
}

pub type StaticConstantTable = BTreeMap<StaticConstantId, Arc<StaticConstantProduct>>;
```

`StaticConstantProduct::materialization_descriptor(&self, store: &StaticValueStore)` projects a pure `MaterializationDescriptor`.

### 8.2 Materialization descriptors (`phalcom-semantic/src/materialization.rs`)

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum MaterializationDescriptor {
    Unit,
    Bool(bool),
    SmallInt(i64),
    LargeInt(BigInt),
    Float(StaticFloat),
    String(Arc<str>),
    Symbol(Arc<str>),
    DeferredStructural(DeferredStructuralKind),
    Unsupported(String),
}
```

### 8.3 Compiler lowering projection (`phalcom-core/src/modules/semantic_lowering.rs`)

```rust
#[derive(Clone, Debug)]
pub struct StaticConstantLoweringSpec {
    pub id: StaticConstantId,
    pub name: Arc<str>,
    pub descriptor: MaterializationDescriptor,
}
```

In `phalcom-core/src/compiler/lib/expr.rs`, reads of `const` bindings emit direct `Bytecode::Constant` pool loads with zero runtime initializer overhead, no `DefineGlobal`, and no local slots. Structural constants return `CompilerError::StaticConstStructuralDeferred(name)`.

---

## 9. Takeover boundary — LANG007.C4.P3

`LANG007.C4.P2` is certified COMPLETE and FOCUSED_TESTED.

The next plan is **`LANG007.C4.P3` — First-class Pack and call/rest convergence**:
- Scope:
  - First-class `Pack` structural value and type representation in Telos;
  - Positional and labeled element transport without intermediate heap allocations;
  - Rest/splat syntax and semantic convergence (`...pack`);
  - Interoperability between tuples and packs.
- Read before starting:
  - `docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md`
  - `docs/implementation/LANG007/LANG007.C4/LANG007.C4.P2-walkthrough.md`
  - `docs/workflow/implementation-record-lifecycle-convention.md`

