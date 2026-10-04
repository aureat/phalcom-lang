# LANG007.C4.P2 — Static Value Model, Static Definitions, Evaluator Foundation, and Closed `const`
## Detailed Implementation Requirements Analysis

**Project:** Telos / `aureat/phalcom-lang`<br>
**Checkpoint:** `LANG007.C4` — Structural Static Upgrade<br>
**Plan:** `LANG007.C4.P2`<br>
**Repository baseline audited for P2:** `main @ c2a938c7c195b1098891ecf0cd6940e89e32ee17`<br>
**Baseline commit:** `fix(lang007): advance C4 P1 field and binding closure`<br>
**Analysis date:** 2026-10-04<br>
**P1 state at supplied records:** mostly implemented / focused-tested, but not administratively closed; T7 parity/G1/G2/T11 remained open in the supplied checkpoint/handoff.<br>
**Purpose:** implementation-grade requirements analysis for C4.P2, grounded in the latest pushed P1 implementation state and the ratified C4 architecture.

---

# 1. Executive conclusion

C4.P2 is the semantic foundation of the remaining C4 checkpoint.

It should **not** be implemented as ordinary constant folding. It introduces a compiler-owned semantic execution layer with three responsibilities:

1. **represent ordinary compile-time values** without using runtime `Value` or VM heap identity;
2. **represent and check reusable static definitions** whose result may be a Type, a C3 kinded static term, or an ordinary compile-time value such as `Int`, `Bool`, `String`, `Symbol`, Tuple/Record/data;
3. **evaluate closed static computations deterministically under semantic budget/cancellation authority**, while publishing products that P3–P8 can extend rather than replace.

P1 gives P2 the correct phase boundary:

```text
source `const`
    ↓
ModuleBindingSemantics::StaticConstant
    ↓
compiler emits no runtime slot / initializer
    ↓
runtime use currently fails closed as StaticConstUnmaterialized
```

That barrier must remain.

However, P1's compiler-side `global_static_constants` / `static_const_scopes` symbol sets are **not a suitable P2 semantic identity or value authority**. They are a lowering barrier and scope backstop. P2 must publish canonical semantic products keyed by stable semantic identity and make compiler lowering consume those products.

The current pushed repository exposes several architectural requirements that are more consequential than the original pre-P1 C4 analysis suggested:

1. **`type` syntax is still structurally a C3 type alias.** `TypeAliasDef.body` is `TypeAnnotation`, and the parser currently requires `=` and parses the result `KindSyntax` before generic parameters. P2 needs a real syntax/AST/semantic migration.

2. **C3 kinds are insufficient for C4 ordinary-value static domains.** `T: Type` and `S: Shape` can be modeled by `KindId`, but `N: Int`, `Flag: Bool`, and result `: Int`/`: Bool` mean “compile-time ordinary value whose Telos type is Int/Bool,” not “term whose kind is Int/Bool.”

3. **Generic C4 static definitions need ordinary-value parameters.** Ratified forms such as:
   ```telos
   type IsZero<N: Int>: Bool = N == 0
   type VariantsOf<T: Sum, Name: Symbol>: Variants = ...
   ```
   cannot be represented solely by current `GenericSignature` / `TypeParameterData.kind: KindId`.

4. **Static operations must be keyed by semantic operation identity, not AST spelling.** Current binary/unary operators are ordinary semantic dispatch, including overload/reflection behavior. P2 may reduce `40 + 2` only because semantic analysis selected an explicitly static-safe core operation.

5. **StaticValue IDs need snapshot-safe lifetime semantics.** The DB reuses query products incrementally. A raw `StaticValueId(u32)` cannot safely outlive/refer across snapshots unless the owning store has persistent/stable lineage or the reference is store-qualified.

6. **Ordinary static Int must follow Telos Int semantics.** Telos Int is exact/unbounded at the surface; C3 `TypeData::ConstInt(i64)` and VM immediate `i64` are not sufficient as the canonical C4 ordinary static Int representation.

Recommended P2 architecture:

```text
source static declaration
        ↓
checked StaticDefinition / StaticConstant product
        ↓
StaticValueDomain requirements
        ↓
ordinary semantic expression resolution
        ↓
exact StaticOperationResolution for reducible operations
        ↓
bounded explicit-frame StaticEvaluator
        ↓
StaticEvalOutcome
        ├─ Reduced(StaticValueId)
        ├─ Symbolic(...)      # seam only; closed const rejects
        ├─ Invalid(...)
        ├─ Recursive(...)
        ├─ BudgetExceeded(...)
        └─ Cancelled
        ↓
semantic MaterializationDescriptor
        ↓
ModuleLoweringSemantics projection
        ↓
P8 completes durable/general runtime materialization
```

P2 must leave P3/P4 these seams:

```text
StaticValue can gain Pack without redesign.
Static domains/values can later carry FieldKey/ProductShape/Variant values.
```

It must leave P6/P7 these seams:

```text
same evaluator can add match/for/while/control
same evaluator can add recursion/memoization/residual IR
```

And P8 this seam:

```text
P2 products have semantic identity and content fingerprints,
but no ObjRef, VM Symbol ID, raw AST residual, or evaluator continuation
is part of durable meaning.
```

---

# 2. Audit basis

## 2.1 Supplied records

The analysis used:

```text
LANG007.C4-CHECKPOINT.md
LANG007.C4-requirements-analysis.md
LANG007.C4.P1-handoff.md
LANG007.C4.P1-walkthrough.md
```

The supplied state records P1 as still partial/baseline-blocked and says P2 must not begin until P1 closure. This analysis is therefore planning-ready, not authorization to bypass P1 acceptance.

## 2.2 Current repository baseline

The live pushed `main` used for P2 audit is:

```text
c2a938c7c195b1098891ecf0cd6940e89e32ee17
```

This supersedes the original C4 audit baseline:

```text
8c3a98aab41ed8631e9f8746a6b29984cf77581d
```

for implementation facts.

## 2.3 Principal paths audited

```text
phalcom-ast/src/{ast.rs,parser.rs}

phalcom-semantic/src/
    semantic_shard.rs
    snapshot.rs
    type_alias.rs
    declarations.rs
    types/{store.rs,kind.rs,annotation.rs,parameter.rs,type_member.rs,
           denotation.rs,outcome.rs}
    checker/{analysis.rs,typed_expr.rs,statement.rs,expression.rs,
             control.rs,context.rs,flow/**}
    db/{key.rs,product.rs,query.rs,state.rs,fingerprint.rs}
    metadata/export.rs

phalcom-modules/src/{interface.rs,linker.rs}

phalcom-core/src/
    modules/{semantic_lowering.rs,compile.rs,linkage.rs}
    compiler/lib/{mod.rs,expr.rs,scope.rs,error.rs}
    bytecode.rs
    value/**
    vm/dispatch.rs

phalcom-core/tests/core/compiler/static_const.rs

phalcom-type-meta/src/header.rs
```

Numeric/runtime decisions were also checked because static numeric equality/hash and exact Int representation are P2 language-semantics concerns.

---

# 3. P1 takeover state P2 must consume

P2 must not redesign these P1 seams.

## 3.1 Binding surface

```rust
pub enum BindingKind {
    Let,
    LetMut,
    Const,
}
```

Meaning:

```text
Let     Runtime + Immutable
LetMut  Runtime + Mutable
Const   Static
```

## 3.2 Module binding semantics

```rust
pub enum ModuleBindingSemantics {
    Runtime { mutable: bool },
    StaticConstant,
    Declaration,
}
```

This is already the correct phase category.

## 3.3 Top-level binding identity

`phalcom-modules::linker::SymbolId` is the stable module-owned binding identity:

```text
module + name
```

The source index already targets top-level consts as module bindings.

**Recommendation:** use `SymbolId`, or a thin newtype over it, as top-level `StaticConstantId`.

Do not use:

```text
VM Symbol
source range
compiler slot
HashSet membership
```

as identity.

## 3.4 Compiler const barrier

P1 currently tracks:

```text
global_static_constants: HashSet<Symbol>
static_const_scopes: Vec<HashSet<Symbol>>
```

and runtime reference fails as:

```text
CompilerError::StaticConstUnmaterialized
```

These remain useful transition/assertion mechanisms.

They are **not** canonical value storage.

After P2:

```text
semantic static product = value authority
compiler symbol sets = scope/barrier/backstop or removable adapter
```

## 3.5 Initialization/runtime sentinel

P2 does not alter:

```text
InitializationState
BindingReadSafety
ValueTag::UninitializedBinding
GetLocalChecked
GetUpvalueChecked
GetGlobalChecked
```

Static evaluator locals are semantic slots, not VM values.

## 3.6 Constructor/field products

P1 established semantic constructor/initializer products and explicit-field lifecycle. Static data/materialization must consume canonical existing declaration/constructor identities and must not rederive them from source names/AST.

---

# 4. P2 exact scope

## 4.1 P2 should make these forms real

```telos
const answer = 40 + 2

const cfg = {
    enabled: true,
    retries: 3,
}

type PairType<A: Type, B: Type> =
    (A, B)

type IsZero<N: Int>: Bool =
    N == 0

type Choose<Flag: Bool, A: Type, B: Type>: Type =
    if Flag { A } else { B }

type Example<N: Int>: Int {
    let mut x = N
    x = x + 1
    x
}
```

## 4.2 P2 supports at minimum

```text
literal static values
Type/static-term values already represented by C3
static-definition parameters
non-recursive static-definition application
static const references
local static let / let mut
local assignment
lexical blocks with trailing value
ordinary if
boolean short-circuit
selected closed arithmetic/comparison/equality
Tuple/Record construction
phase-safe data/variant construction
phase-safe projection/index where semantically selected
closed const evaluation
semantic materialization descriptors
budgets/cancellation
semantic query/fingerprint integration
```

## 4.3 Deliberately deferred

```text
first-class Pack                                  P3
generalized FieldKey                             P4
Product/Sum reflection                           P5
static match                                     P6
static structural for                            P6
static while                                     P6
return/break/continue final semantics             P6
general pattern/destructuring evaluation          P6
proof predicates                                  P6
general recursive static definitions              P7
normalized residual IR                            P7
memoization/specialization cache                  P7
durable static-value metadata                     P8
artifact-only static definition import execution  P8
general/durable runtime materializer               P8
checkpoint tooling/public ontology closure         P9
```

## 4.4 P2 binding-pattern boundary

Recommended P2 rule:

```telos
const x = ...
```

supported.

General static destructuring:

```telos
const (a, b) = ...
```

deferred to P6, where static pattern evaluation can consume the canonical pattern matcher.

Likewise block-local P2 `let`/`let mut` should initially use simple name binders unless existing canonical pattern projections can be reused without adding a P2-only pattern engine.

---

# 5. Fundamental domain distinction: C3 static terms vs C4 static values

C3 already has a mature kinded static-term system:

```text
KindId
KindData::StaticDomain
TypeId
TypeData::ConstInt
TypeData::StaticData
StaticArgument
```

It represents:

```text
Type
TypePack
RecordRow
type data Shape
type data Endian
compiler/type-level const terms
```

P2 must preserve it.

C4 adds:

```text
ordinary Telos values evaluated at compile time
```

Examples:

```text
true
42
3.14
"abc"
#name
static Tuple/Record/data value
```

These are not automatically `TypeData`.

## 5.1 Required canonical domain model

Conceptually:

```rust
pub enum StaticValueDomain {
    /// Canonical kinded semantic term.
    Kind(KindId),

    /// Ordinary compile-time value whose Telos value type
    /// must satisfy this proper Type.
    ValueType(TypeId),
}
```

Names can differ; the semantic distinction cannot.

## 5.2 Examples

```text
T: Type
    -> Kind(Type)

P: TypePack
    -> Kind(TypePack)

S: Shape
where Shape is `type data`
    -> Kind(StaticDomain(Shape))

N: Int
    -> ValueType(Int)

Flag: Bool
    -> ValueType(Bool)

result : Bool
    -> ValueType(Bool)

result : Type
    -> Kind(Type)
```

## 5.3 Why KindId alone fails

Ordinary `Int` is a proper type, not a `KindData::StaticDomain`.

`N: Int` does not mean “N has kind Int.” It means “N is a compile-time ordinary value whose Telos type is Int.”

## 5.4 Why TypeId alone also fails

`T: Type` is a kinded semantic term binder, not an ordinary runtime value whose type happens to be Type.

The C3 proper-type/kind boundary must remain explicit.

---

# 6. Reuse the existing type-member syntax distinction

The AST already has:

```rust
pub enum TypeMemberParameterDomainSyntax {
    Kind(KindSyntax),
    ConstType(TypeAnnotation),
}

pub enum TypeMemberResultSyntax {
    Kind(KindSyntax),
    Type(TypeAnnotation),
}
```

This is a strong precedent.

Even though semantic `TypeMemberDomain` is currently only:

```rust
Kind(KindId)
```

the parser already distinguishes kind-domain from constant-value-type domain.

## Requirement

Extract/generalize a shared syntax concept, e.g.:

```rust
pub enum StaticDomainSyntax {
    Kind(KindSyntax),
    Value(TypeAnnotation),
}
```

and semantic:

```rust
StaticValueDomain
```

Use it for:

```text
static definitions
static-definition parameters
future generalized type members
future structural reflection members
```

Do not create separate conceptual domain models for each subsystem.

---

# 7. Static-definition generic parameters need a new semantic layer

Current generic parameters fundamentally assume:

```text
parameter -> KindId
```

This works for:

```telos
T: Type
*P: TypePack
S: Shape
```

but not:

```telos
N: Int
Flag: Bool
Name: Symbol
```

## 7.1 Do not globally mutate C3 TypeParameterData

That would destabilize:

```text
type inference
substitution
trait refs/conformance
type lambdas
pack binders
metadata
```

## 7.2 Recommended model

Conceptually:

```rust
pub struct StaticParameterId {
    definition: DeclarationId,
    index: u32,
}

pub struct StaticParameter {
    pub id: StaticParameterId,
    pub name: Box<str>,
    pub mode: GenericParameterMode,
    pub domain: StaticValueDomain,

    // Only for the C3 kinded lane:
    pub type_parameter: Option<TypeParameterId>,
}

pub struct StaticDefinitionSignature {
    pub parameters: Box<[StaticParameter]>,
    pub constraints: ...,
}
```

### Kinded lane

```telos
T: Type
```

retains a canonical `TypeParameterId` and uses existing C3 substitution.

### Value lane

```telos
N: Int
```

uses a static parameter identity and receives a `StaticValue` at application.

## 7.3 Mixed signatures

Must support:

```telos
type Select<N: Int, T: Type, S: Shape>: Type = ...
```

without losing source order/domain.

## 7.4 Pack parameter modes

`*P`, `**P`, `***P` remain C3 kind/pack modes at P2. P3 introduces public first-class Pack value semantics.

---

# 8. Static-definition application arguments

Current `TypeAnnotationExpr::Application` carries `Vec<GenericArgumentSyntax>`, where a single argument is a `TypeAnnotation`.

That is primarily C3 type/static-term application.

P2 static definitions may accept ordinary values.

## 8.1 Required semantic application model

Conceptually:

```rust
pub enum StaticApplicationArgument {
    KindedTerm(StaticArgument),
    Value(StaticValueId),
}

pub struct StaticDefinitionApplication {
    pub definition: DeclarationId,
    pub arguments: Box<[StaticApplicationArgument]>,
}
```

Do not force arbitrary ordinary compile-time values into `TypeData::Applied`.

## 8.2 C3 ConstInt bridge

Existing C3 integer static arguments can be converted at the application boundary to an ordinary P2 `StaticValue::Int` when the destination parameter is `ValueType(Int)`.

Do not expose two semantically different integer constant systems to users.

## 8.3 Future Bool/String/Symbol

P2 should make the semantic argument carrier general now, because P5 reflection needs Symbol-valued parameters. Avoid an Int-only ABI.

---

# 9. `type` syntax migration

Current live model:

```rust
pub struct TypeAliasDef {
    pub name: String,
    pub result_kind: Option<KindSyntax>,
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    pub body: TypeAnnotation,
    ...
}
```

Current parse order is effectively result-kind before generic parameters.

C4 requires:

```telos
type Optional<T> =
    T | None

type IsZero<N: Int>: Bool =
    N == 0

type Count<*P>: Int {
    ...
}
```

## 9.1 Body AST

Required:

```rust
pub enum StaticDefinitionBody {
    Expression(Expr),
    Block(LexicalBlock),
}
```

Use **`LexicalBlock`**, not `Expr::Block`.

The live AST's `Expr::Block` denotes a first-class Closure. Reusing it for static-definition body would introduce the wrong semantics.

## 9.2 Canonical grammar

```text
type
    name
    generic-parameters?
    result-domain?
    where-clause?
    body

result-domain := ":" StaticDomainSyntax

body :=
    "=" expression
  | lexical-block
```

Canonical examples:

```telos
type F<T>: Int = expr
type F<T>: Int { ... }
```

No `=` before block.

## 9.3 Legacy aliases

```telos
type Name = TypeForm
type Name<T> = TypeForm
```

remain legal.

Implicit result domain:

```text
Kind(Type)
```

and normalize through the same StaticDefinition authority.

## 9.4 Existing old result-domain order

Census current source before hard removal. Prefer a bounded hard C4 cutover rather than indefinite dual grammar.

---

# 10. One static-definition authority; TypeAliasInfo becomes compatibility projection

Current `TypeAliasInfo` is TypeId-centric:

```text
declaration
kind
generic_signature
form: TypeId
structural_form
dependencies
source
```

It remains valuable for C1–C3.

It cannot be authoritative for:

```telos
type IsZero<N: Int>: Bool = ...
```

## Required authority

Conceptually:

```rust
pub struct StaticDefinition {
    pub declaration: DeclarationId,
    pub signature: StaticDefinitionSignature,
    pub result_domain: StaticValueDomain,
    pub body: CheckedStaticBody,
    pub dependencies: Arc<[SemanticDependency]>,
    pub source: SemanticSourceSpan,
    pub fingerprint: ProductFingerprint,
}
```

For Type-valued definitions representable by C3 machinery:

```text
StaticDefinition
    ↓
TypeAliasInfo compatibility projection
```

not two independent source-derived semantic products.

Do not route proven C3 aliases through a general evaluator if existing type-lambda/type-normalization machinery can remain the optimized implementation of that StaticDefinition subset.

---

# 11. Stable identities

## 11.1 Static definitions

Use:

```text
DeclarationId
```

or thin `StaticDefinitionId(DeclarationId)`.

Never source range/body hash/rendered name.

## 11.2 Top-level const

Use:

```text
SymbolId(module, name)
```

or `StaticConstantId(SymbolId)`.

## 11.3 Local const

Use owner + `BindingId`, e.g.:

```text
CallableId + BindingId
or
StaticDefinitionId + local binding identity
```

No independent global DB key is required for evaluator-local constants.

---

# 12. Semantic DB products

Add at minimum:

```rust
QueryKey::StaticDefinition(DeclarationId)
QueryKey::StaticConstant(SymbolId)
```

and product variants:

```rust
StaticDefinitionProduct
StaticConstantProduct
```

Conceptual constant product:

```rust
pub struct StaticConstantProduct {
    pub id: StaticConstantId,
    pub declared_domain: Option<StaticValueDomain>,
    pub value_domain: StaticValueDomain,
    pub outcome: StaticEvalOutcome,
    pub materialization: Option<MaterializationDescriptorId>,
    pub dependencies: Arc<[SemanticDependency]>,
    pub source: SemanticSourceSpan,
}
```

Const A depending on B must record the query dependency.

Avoid durable static-application query keys in P2 unless needed; P7 owns memo/application specialization. In particular, do not key a durable DB query by raw snapshot-local `StaticValueId`.

---
