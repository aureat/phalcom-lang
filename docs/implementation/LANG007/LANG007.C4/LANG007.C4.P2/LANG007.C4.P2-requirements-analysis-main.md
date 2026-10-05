# LANG007.C4.P2 — Static Value Model, Static Definitions, Evaluator Foundation, and Closed `const`
## Detailed Implementation Requirements Analysis

**Project:** Telos / `aureat/phalcom-lang`<br>
**Checkpoint:** `LANG007.C4` — Structural Static Upgrade<br>
**Plan:** `LANG007.C4.P2`<br>
**Repository baseline audited for P2:** `main @ c2a938c7c195b1098891ecf0cd6940e89e32ee17`<br>
**Baseline commit:** `fix(lang007): advance C4 P1 field and binding closure`<br>
**Analysis date:** 2026-10-04<br>
**P1 state at attached records:** mostly implemented / focused-tested, but not administratively closed; T7 parity/G1/G2/T11 remained open in the supplied checkpoint/handoff.<br>
**Purpose:** implementation-grade requirements analysis for C4.P2, grounded in the latest pushed P1 implementation state and the ratified C4 architecture.

---

# 1. Executive conclusion

C4.P2 is the semantic foundation of the entire remaining C4 checkpoint.

Its implementation should **not** be understood as “add constant folding.” It introduces a new, compiler-owned semantic execution layer with three distinct responsibilities:

1. **represent ordinary compile-time values** without using runtime `Value`/heap identity;
2. **represent and check reusable static definitions** whose result may be a Type, a C3 static term, or an ordinary compile-time value such as `Int`, `Bool`, `String`, `Symbol`, Tuple/Record/data;
3. **evaluate closed static computations deterministically and under semantic budget/cancellation authority**, while publishing products that later P3–P8 can extend rather than replace.

The current pushed P1 implementation gives P2 a good phase boundary:

```text
source `const`
    ↓
ModuleBindingSemantics::StaticConstant
    ↓
compiler emits no runtime slot / initializer
    ↓
runtime use currently fails closed as StaticConstUnmaterialized
```

That barrier is correct and must be preserved.

However, **the P1 compiler-side `global_static_constants` / `static_const_scopes` symbol sets are not a suitable P2 semantic identity or value authority**. They are a lowering barrier and scope backstop. P2 must publish canonical semantic products keyed by stable semantic identity and make compiler lowering consume those products.

The live repository also exposes four architectural issues that are more consequential than the original pre-P1 C4 analysis suggested:

1. **`type` syntax is still structurally a C3 type alias.** `TypeAliasDef.body` is `TypeAnnotation`, and the current parser requires `=` and parses the result `KindSyntax` before generic parameters. P2 must perform a real syntax/AST/semantic migration, not merely add an evaluator beside the existing alias model.

2. **C3 kinds are insufficient for C4 ordinary-value static domains.** `T: Type` and `S: Shape` can be modeled by `KindId`, but `N: Int`, `Flag: Bool`, and a result `: Int`/`: Bool` are different: they mean “compile-time value whose ordinary Telos type is `Int`/`Bool`,” not “term whose kind is Int/Bool.” P2 needs one explicit static-domain requirement model that distinguishes:
   ```text
   kinded semantic term
   vs
   ordinary compile-time value of a proper Type
   ```

3. **Generic C4 static definitions need value-domain parameters.** The ratified examples:
   ```telos
   type IsZero<N: Int>: Bool = N == 0
   type VariantsOf<T: Sum, Name: Symbol>: Variants = ...
   ```
   cannot be implemented solely by the current `GenericSignature` / `TypeParameterData.kind: KindId` model. P2 must establish a static-definition parameter model that can mix kinded parameters and ordinary compile-time-value parameters without corrupting C3 generic/type semantics.

4. **Static operation evaluation must be keyed by semantic operation identity, not AST operator spelling.** Current binary operators are ordinary semantic dispatch, including overload/reflection behavior. P2 may evaluate `40 + 2` only because semantic analysis selected an explicitly static-safe core operation. It must not interpret every `BinaryOp::Add` as integer addition and thereby bypass overload resolution or execute arbitrary runtime/user behavior at compile time.

The recommended P2 architecture is:

```text
source static declaration
        ↓
checked StaticDefinition / StaticConstant product
        ↓
StaticValueDomain requirements
        ↓
ordinary semantic expression resolution
        ↓
explicit StaticOperationResolution for reducible operations
        ↓
bounded explicit-frame StaticEvaluator
        ↓
StaticEvalOutcome
        ├─ Reduced(StaticValueId)
        ├─ Symbolic(...)            # structural seam; closed const rejects
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

P2 must leave P3/P4 two clean seams:

```text
StaticValue can gain first-class Pack nodes without redesign.
StaticValue/static domains can carry future FieldKey/ProductShape/Variant values.
```

It must leave P6/P7 two clean seams:

```text
same evaluator can add match/for/while/return/break/continue
same evaluator can add recursion/memoization/residual IR
```

And it must leave P8 one clean durability seam:

```text
all P2 products have semantic identity and content fingerprints,
but no runtime ObjRef, process-local symbol ID, raw AST residual,
or evaluator continuation is part of their durable meaning.
```

---

# 2. Audit basis

## 2.1 Attached C4 records

This analysis used the supplied:

```text
LANG007.C4-CHECKPOINT.md
LANG007.C4-requirements-analysis.md
LANG007.C4.P1-handoff.md
LANG007.C4.P1-walkthrough.md
```

The attached checkpoint says C4 is `IN_PROGRESS / PARTIAL`, P1 is still the active plan, and P2 is next. The supplied P1 records establish the static-const barrier, initialization/runtime work, module binding category, and the explicit instruction that P2 attach `StaticValue`, initializer semantics, declared result domain, dependency fingerprints, and materialization to semantic const identity rather than a runtime slot.

## 2.2 Live repository baseline

The repository was re-audited at the current pushed `main`:

```text
c2a938c7c195b1098891ecf0cd6940e89e32ee17
```

This is substantially newer than the original C4 analysis baseline:

```text
8c3a98aab41ed8631e9f8746a6b29984cf77581d
```

Therefore this document treats the live `c2a938...` source as implementation truth and uses the older C4 analysis only for ratified architecture and prior reasoning.

## 2.3 Principal live paths inspected

At minimum:

```text
phalcom-ast/src/
    ast.rs
    parser.rs

phalcom-semantic/src/
    semantic_shard.rs
    snapshot.rs
    type_alias.rs
    declarations.rs
    static_data.rs / static_case.rs where applicable
    types/
        store.rs
        kind.rs
        annotation.rs
        parameter.rs
        type_member.rs
        type_lambda.rs
        denotation.rs
        outcome.rs
    checker/
        analysis.rs
        typed_expr.rs
        statement.rs
        expression.rs
        control.rs
        context.rs
        flow/*
    db/
        key.rs
        product.rs
        query.rs
        state.rs
        fingerprint.rs
    metadata/export.rs

phalcom-modules/src/
    interface.rs
    linker.rs

phalcom-core/src/
    modules/
        semantic_lowering.rs
        compile.rs
        linkage.rs
    compiler/lib/
        mod.rs
        expr.rs
        scope.rs
        error.rs
    bytecode.rs
    value/*
    vm/dispatch.rs

phalcom-core/tests/core/compiler/static_const.rs

phalcom-type-meta/src/header.rs
```

Relevant current numeric/value specifications and implementation were also checked because StaticValue equality/hash and numeric representation are P2 correctness requirements.

---

# 3. P1 takeover state that P2 must consume

P2 must not redesign the following P1 seams.

## 3.1 Binding surface

Current canonical AST:

```rust
pub enum BindingKind {
    Let,
    LetMut,
    Const,
}
```

Current helper direction:

```text
Let     runtime immutable
LetMut  runtime mutable
Const   static phase
```

P2 extends `Const`; it does not reinterpret it.

## 3.2 Module binding semantics

Current module interface has:

```rust
pub enum ModuleBindingSemantics {
    Runtime { mutable: bool },
    StaticConstant,
    Declaration,
}
```

This is the correct phase-level module category.

P2 should attach semantic value/body/result products to the static-constant identity; it must not add another boolean such as `is_compile_time`.

## 3.3 Top-level binding identity

`phalcom-modules::linker::SymbolId` is the semantic identity of one module-owned binding:

```rust
SymbolId {
    module: ModuleId,
    name: ...
}
```

The source index already maps top-level `const` to a module binding target.

**Recommendation:** use `SymbolId` directly, or a thin newtype around it, as the stable in-process identity of a module static constant.

Do not use:

```text
VM Symbol
source range
compiler local slot
HashSet membership
```

as static-constant identity.

## 3.4 Compiler barrier

P1 currently tracks:

```text
global_static_constants: HashSet<Symbol>
static_const_scopes: Vec<HashSet<Symbol>>
```

and a runtime reference currently fails with:

```text
CompilerError::StaticConstUnmaterialized
```

This remains useful as a P2 transition/assertion mechanism.

It is **not** the canonical value store.

After P2:

```text
semantic static product
    is value authority

compiler HashSet
    may remain scope/barrier information
    or be replaced by semantic lowering attachments
```

The compiler must never evaluate the initializer.

## 3.5 Semantic initialization and runtime sentinel

P2 does not alter:

```text
InitializationState
BindingReadSafety
ValueTag::UninitializedBinding
GetLocalChecked / GetUpvalueChecked / GetGlobalChecked
```

Static evaluator local slots are separate semantic storage.

Do not reuse `ValueTag::UninitializedBinding` inside `phalcom-semantic`.

## 3.6 Constructor/field products

P1 added substantial constructor semantic products and explicit-field lifecycle behavior.

P2 does not own these.

Static data/materialization code that constructs data values must consume existing canonical data/constructor products and must not reintroduce AST-derived constructor identity.

---

# 4. P2 objective and exact scope

## 4.1 P2 should make these forms semantically real

```telos
const answer = 40 + 2

const enabled = true

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

type Example<T>: Int {
    let mut n = 1
    n = n + 1
    n
}
```

## 4.2 P2 supports

At minimum:

```text
literal static values
Type/static-term values already representable by C3
static-definition parameters
non-recursive static-definition application
static const references
local static let / let mut
local assignment
lexical blocks with trailing value
ordinary if
short-circuit boolean control
selected closed arithmetic/comparison/equality operations
Tuple/Record formation
phase-safe data/variant formation
phase-safe projection/index operations where semantically selected
closed const evaluation
semantic materialization descriptor formation
budget/cancellation
semantic query/fingerprint integration
```

## 4.3 P2 deliberately does not support yet

Owned later:

```text
first-class Pack value/runtime semantics                P3
generalized FieldKey                                   P4
Product/Sum reflection                                 P5
static match                                           P6
static structural for                                  P6
static while                                           P6
return/break/continue control semantics                 P6
general structural pattern/destructuring evaluation    P6
proof predicates Equivalent/Subtype/Conforms           P6
general recursive static definitions                   P7
normalized residual IR                                 P7
static memoization/specialization cache                 P7
durable static-value metadata schema                    P8
artifact-only cross-module static definition execution P8
full/general runtime materializer                       P8
public ontology retirement/tooling closure              P9
```

## 4.4 Important bounded syntax decision

P2 should keep `const` binding patterns conservative.

Recommended P2 rule:

```telos
const x = ...
```

supported.

General static destructuring such as:

```telos
const (a, b) = ...
```

should remain rejected/deferred until P6 integrates static pattern semantics through the existing matcher.

Likewise, P2 block-local `let`/`let mut` should initially require simple name bindings unless a pre-existing non-duplicated semantic projection can destructure StaticValue without creating a P2-only pattern engine.

This avoids violating C4's “one Telos pattern language” rule.

---

# 5. The most important P2 architecture distinction: kinded static terms vs ordinary compile-time values

C3 already has a mature kinded static-term system.

Examples:

```text
Type
TypePack
RecordRow
type data Shape
type data Endian
ConstInt term used by C3 type-level operations
```

These live in or through:

```text
KindId
KindData::StaticDomain
TypeId
TypeData::ConstInt
TypeData::StaticData
StaticArgument
```

P2 must preserve that system.

But C4 introduces another category:

```text
ordinary Telos values known/evaluated at compile time
```

Examples:

```text
true
42
3.14
"abc"
#name
runtime-data value whose entire construction is static
```

These are **not** automatically `TypeData`.

## 5.1 Required canonical domain model

P2 needs one domain-requirement representation, conceptually:

```rust
pub enum StaticValueDomain {
    /// A canonical kinded semantic term.
    ///
    /// Examples:
    /// Type
    /// TypePack
    /// RecordRow
    /// a C3 type-data domain such as Shape
    Kind(KindId),

    /// An ordinary compile-time value whose Telos value type must
    /// satisfy this proper type.
    ///
    /// Examples:
    /// Int
    /// Bool
    /// String
    /// Symbol
    /// Variant / ProductShape later in C4
    ValueType(TypeId),
}
```

Names may differ.

The distinction is mandatory.

## 5.2 Examples

```text
T: Type
    -> Kind(KindId::TYPE)

P: TypePack
    -> Kind(KindId::TYPE_PACK)

S: Shape
where Shape is `type data`
    -> Kind(KindData::StaticDomain(Shape))

N: Int
    -> ValueType(Int)

Flag: Bool
    -> ValueType(Bool)

result : Bool
    -> ValueType(Bool)

result : Type
    -> Kind(Type)
```

## 5.3 Why `KindId` alone is insufficient

The current contextual kind resolver accepts a reference as a kind only when its resolved type form has a `KindData::StaticDomain` kind.

Ordinary `Int` is a proper type.

It does not mean:

```text
kind = Int
```

Therefore forcing:

```telos
N: Int
```

through `KindSyntax::StaticDomain`
would incorrectly require the declaration `Int` itself to define a static-domain kind.

## 5.4 Why `TypeId` alone is also insufficient

Representing everything as:

```rust
ValueType(TypeId)
```

would lose the crucial difference between:

```text
a value whose type is Type
and
a semantic Type term classified by Kind::Type
```

`T: Type` is a meta-level term binder, not an ordinary runtime object value of class `Type`.

C4 must preserve C3's kinded term semantics.

---

# 6. Reuse the type-member syntax distinction instead of inventing a third model

The current AST already contains a useful distinction for type members:

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

Even though the semantic `TypeMemberDomain` is currently only:

```rust
Kind(KindId)
```

the parser architecture already understands:

```text
kind-domain parameter/result
vs
ordinary constant-value type parameter/result
```

## Requirement

P2 should extract/generalize this concept into a shared static-domain syntax/model used by:

```text
type definitions
future generalized type members
future reflection members
static-definition parameters
static-definition result requirements
```

Do not create:

```text
TypeDefinitionResultDomain
TypeMemberResultDomain
ReflectionResultDomain
```

as three independent semantic enums.

A shared syntax might conceptually be:

```rust
pub enum StaticDomainSyntax {
    Kind(KindSyntax),
    Value(TypeAnnotation),
}
```

and the semantic form:

```rust
StaticValueDomain
```

P5 can later migrate `TypeMemberDomain` onto the same authority.

---

# 7. Static-definition generic parameters: P2 needs a new semantic parameter layer

This is a major implementation requirement.

The current generic parameter model assumes each generic parameter is classified by `KindId`.

That works for:

```telos
T: Type
*P: TypePack
S: Shape       // if Shape is type data
```

It does not work for:

```telos
N: Int
Flag: Bool
Name: Symbol
```

## 7.1 Do not globally mutate C3 `TypeParameterData` into an ordinary value-parameter system

That would destabilize:

```text
type inference
substitution
trait references
generic conformance
type lambdas
pack binders
metadata
```

for no reason.

## 7.2 Recommended P2 model

Introduce definition-owned static parameters:

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

    // Present only for the C3-kind/type-term lane.
    pub type_parameter: Option<TypeParameterId>,
}

pub struct StaticDefinitionSignature {
    pub parameters: Box<[StaticParameter]>,
    pub constraints: ...,
}
```

Names are illustrative.

### Kinded parameter lane

```telos
T: Type
```

continues to have a canonical `TypeParameterId`.

Type-formation logic can therefore continue using the C3 substitution/inference machinery.

### Ordinary value lane

```telos
N: Int
```

gets an evaluator/static parameter identity and does **not** pretend to be a TypeStore type variable.

Its body uses a StaticValue argument.

## 7.3 Mixed definitions

P2 must support ordered signatures such as:

```telos
type Select<N: Int, T: Type, S: Shape>: Type = ...
```

without losing argument position or domain.

## 7.4 Pack-mode parameters

Existing:

```text
*P
**P
***P
```

remain C3 kinded/pack-domain parameters where they are legal.

P3 later gives the public first-class Pack value model.

Do not reinterpret pack binder modes as ordinary value parameters during P2.

---

# 8. Static-definition application and generic arguments

Current `TypeAnnotationExpr::Application` contains:

```rust
Vec<GenericArgumentSyntax>
```

where `GenericArgumentSyntax::Single` is itself a `TypeAnnotation`.

That is a C3 type/static-term application model.

P2 needs C4 static applications that may pass ordinary static values.

Examples eventually include:

```telos
IsZero<3>
Choose<true, Int, String>
VariantsOf<T, #Number>
```

## 8.1 Do not force ordinary value arguments into `TypeData::Applied`

`TypeData::Applied` is the C3 kinded/type-term application representation.

An ordinary compile-time string/data/Bool argument is not automatically a TypeStore term.

## 8.2 Required semantic application model

Conceptually:

```rust
pub enum StaticApplicationArgument {
    KindedTerm(StaticArgument),  // C3 term/row/etc.
    Value(StaticValueId),        // C4 ordinary compile-time value
}

pub struct StaticDefinitionApplication {
    pub definition: DeclarationId,
    pub arguments: Box<[StaticApplicationArgument]>,
}
```

This application object is query/evaluator semantic state.

It is not runtime call state.

## 8.3 C3 `ConstInt` compatibility bridge

Current generic/type-member syntax already represents some integer constants as C3 kinded terms.

For a C4 parameter:

```telos
N: Int
```

an existing `ConstInt(3)` syntax/term may be converted at the static-definition application boundary into:

```text
StaticValue::Int(3)
```

provided its semantic value is exactly an Int and no identity is lost.

Do not keep two independently observable integer constant semantics.

## 8.4 Future Bool/String/Symbol arguments

P2 should design the application argument layer to accept ordinary `StaticValueId` arguments from day one even if the first parser slice initially exercises Int.

P5 reflection depends on Symbol-valued static arguments.

Avoid an Int-only static-parameter ABI that P5 must replace.

---

# 9. `type` syntax migration

The current live AST is:

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

The current parser order is effectively:

```text
type Name : ResultKind <Generics> where ... = TypeForm
```

C4's ratified forms are:

```telos
type Optional<T> =
    T | None

type IsZero<N: Int>: Bool =
    N == 0

type Count<*P>: Int {
    ...
}
```

## 9.1 Required AST

Replace/generalize the alias-specific body:

```rust
pub enum StaticDefinitionBody {
    Expression(Expr),
    Block(LexicalBlock),
}
```

Use the existing `LexicalBlock`.

Do **not** use `Expr::Block` for the definition block because `Expr::Block` is a first-class Closure value in the live AST.

This distinction is already explicit in `if` and `match` parsing.

Recommended definition node:

```rust
pub struct TypeDefinitionDef {
    pub name: String,
    pub name_range: SourceRange,
    pub generic_parameters: Vec<...>,
    pub result_domain: Option<StaticDomainSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    pub body: StaticDefinitionBody,
    ...
}
```

The source-level statement may remain named `TypeAlias` temporarily for migration compatibility, but semantic naming should become `StaticDefinition` / `TypeDefinition`.

## 9.2 Required grammar order

Canonical P2 grammar should be:

```text
type
    name
    generic-parameters?
    result-domain?
    where-clause?
    body
```

with:

```text
result-domain := ":" StaticDomainSyntax

body :=
    "=" expression
  | lexical-block
```

Therefore:

```telos
type F<T>: Int = expression
type F<T>: Int { ... }
```

is canonical.

No `=` before a block body.

## 9.3 Legacy aliases

Historical:

```telos
type Name = TypeForm
type Name<T> = TypeForm
```

remain legal.

Their implicit result domain is:

```text
Kind(Type)
```

They should normalize through the same StaticDefinition semantic product.

## 9.4 Existing old result-domain ordering

The live parser currently accepts/result-kind-before-generics behavior.

P2 should perform a corpus census before hard removal.

Preferred C4 policy is a hard canonical cutover if current source use is small enough.

Do not preserve two grammar orders indefinitely.

---

# 10. Type-valued alias compatibility: one authority, not two

Current `TypeAliasInfo` stores:

```rust
declaration
kind
generic_signature
form: TypeId
structural_form
dependencies
source
```

This is valuable and deeply integrated with C1–C3 type formation/metadata.

It is too narrow to be the authoritative model of:

```telos
type IsZero<N: Int>: Bool = ...
```

## 10.1 Required authority

Introduce:

```rust
StaticDefinition
```

as the authoritative P2 definition product.

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

## 10.2 TypeAliasInfo becomes a compatibility projection

For a definition whose result domain is `Kind(Type)` and whose checked body can be represented by existing C3 type-term machinery:

```text
StaticDefinition
    ↓
TypeAliasInfo compatibility projection
```

not:

```text
source
   ├─ independently create TypeAliasInfo
   └─ independently create StaticDefinition
```

Two independent products would drift under substitution/fingerprinting.

## 10.3 Do not route all old aliases through general runtime-like evaluation

Existing TypeStore/type-lambda normalization is efficient and already correct for pure Type-valued aliases.

P2 should preserve it as an optimized semantic implementation of one StaticDefinition subset.

The general evaluator is needed when the body is ordinary expression/control/value semantics.

---

# 11. Static definitions must have stable semantic identity

Use:

```text
DeclarationId
```

as the canonical identity of a `type` static definition.

Do not invent an identity from:

```text
source range
body hash
rendered name
TypeId application
```

A thin `StaticDefinitionId(DeclarationId)` newtype is fine for type safety.

This is especially important for P7 recursion and P8 metadata.

---

# 12. Top-level static constant identity

Use the module binding identity:

```text
SymbolId(module, name)
```

or a thin:

```rust
StaticConstantId(SymbolId)
```

This gives P2:

```text
stable module ownership
export/import compatibility
source-index target compatibility
query key suitability
```

Do not use the compiler's VM `Symbol`.

---

# 13. Local `const` identity

Local consts occur inside callable/static scopes and cannot use module `SymbolId`.

Recommended identity:

```text
owner semantic body identity
+
BindingId
```

For runtime callable body:

```rust
LocalStaticConstantId {
    callable: CallableId,
    binding: BindingId,
}
```

For static-definition body:

```rust
StaticLocalId {
    definition: DeclarationId,
    local_index / binding identity
}
```

Local constants do not need independent global DB query keys.

Their evaluated values can live in the owning analysis/evaluation frame.

---

# 14. Semantic DB query products

The current `QueryKey` has no static-definition or constant key.

The current `SemanticProduct` has no StaticDefinition/StaticConstant product.

P2 should add at minimum:

```rust
QueryKey::StaticDefinition(DeclarationId)
QueryKey::StaticConstant(SymbolId)
```

and:

```rust
SemanticProduct::StaticDefinition(Arc<StaticDefinition>)
SemanticProduct::StaticConstant(Arc<StaticConstantProduct>)
```

## 14.1 StaticConstantProduct

Conceptually:

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

A product may retain an Invalid/Blocked outcome for diagnostics according to existing DB conventions, but there must be no successful value when evaluation did not reduce.

## 14.2 Dependency edges

Constant A referencing constant B:

```text
StaticConstant(A)
    depends on
StaticConstant(B)
```

A type definition referencing another definition:

```text
StaticDefinition(A)
    depends on
StaticDefinition(B)
```

Evaluation of an application depends on the definitions and constants it actually consumes.

## 14.3 Do not add durable application query keys yet unless needed

P7 owns memoization/application-key recursion specialization.

P2 may execute non-recursive applications within the enclosing query/evaluator.

If P2 adds an application cache, its key must not use raw snapshot-local `StaticValueId`s as durable DB identity.

---

# 15. Static-value store architecture

## 15.1 Required properties

The static store must provide:

```text
immutable semantic values
deterministic equality/hash
structural sharing
no observable arena identity
snapshot-safe references
bounded construction
future extension to Pack/FieldKey/ProductShape/Variant
```

## 15.2 Recommended value model

Conceptually:

```rust
pub struct StaticValueId(u32);

pub enum StaticValueData {
    Unit,
    Bool(bool),
    Int(CanonicalInt),
    Float(CanonicalFloat),
    String(StaticStringId),
    Symbol(StaticSymbolId),

    // Existing C3 semantic term bridges:
    Type(TypeId),
    StaticTerm(TypeId),

    Tuple(StaticProductId),
    Record(StaticRecordValueId),
    Data(StaticDataValueId),
    Variant(StaticVariantValueId),

    // P3 adds first-class Pack; P2 should reserve the architecture,
    // not implement partial Pack semantics.
}
```

Exact splitting into sub-arenas is mechanically flexible.

## 15.3 No runtime dependencies

`phalcom-semantic` StaticValue must not contain:

```text
phalcom_core::Value
ObjRef
VM ClassId
VM Symbol(u32)
heap address
runtime ProductStorage pointer
```

## 15.4 Static value identity is not language identity

Even if the store is hash-consed:

```text
StaticValueId equality
```

may be an internal fast path.

The language semantics remain value/content semantics.

Never expose:

```text
arena ID
interner index
allocation address
```

through `==`, `===`, reflection, hashing, or metadata.

---

# 16. StaticValueStore lifetime must be snapshot-safe

This requirement is easy to miss and critical.

Semantic DB products are reused incrementally.

A query product containing:

```rust
StaticValueId(17)
```

is only valid if the `StaticValueStore` that owns 17 is still the same semantic store lineage.

## 16.1 Rejected architecture

```text
each semantic revision:
    create empty StaticValueStore
    IDs restart at 0

query DB:
    reuses old StaticConstantProduct containing StaticValueId
```

This creates stale/wrong value identity.

## 16.2 Recommended architecture

Mirror the TypeStore/snapshot ownership model:

```text
semantic workspace/session
    owns evolving/interned StaticValueStore

analysis revision
    extends or persistently clones previous store

SemanticSnapshot
    publishes Arc<StaticValueStore>

query products
    reference IDs valid in that store lineage
```

Alternative:

```text
store-qualified StaticValueRef { store_id, value_id }
```

is acceptable if it integrates naturally.

## 16.3 P8 relationship

P8 owns **durable artifact encoding**.

P2 still owns in-process snapshot correctness.

“P8 will serialize it later” is not a reason to allow stale P2 query IDs.

---

# 17. Integer semantics

Do not use C3 `TypeData::ConstInt(i64)` as the canonical C4 ordinary static Int value.

Telos `Int` semantics are exact/unbounded at the language level and the runtime already promotes outside immediate `i64` representation.

Therefore:

```rust
StaticValueData::Int
```

must use a canonical exact integer representation.

Recommended:

```text
BigInt or a semantically equivalent canonical exact integer
```

P2 may optimize small integers internally.

It must preserve exact arithmetic and only materialize to:

```text
immediate Int
or
runtime LargeInt
```

at the compiler/runtime boundary.

## 17.1 Dependency impact

`phalcom-semantic` currently does not depend on `num-bigint`.

P2 may need to add an exact-integer dependency or move/share a canonical numeric semantic type into an appropriate lower crate.

Do not import `phalcom-core` merely to reuse runtime LargeInt.

---

# 18. Float semantics

Telos Float is IEEE-754 binary64.

Public equality semantics include:

```text
NaN != NaN
+0.0 == -0.0
```

This means a naïve:

```rust
#[derive(Eq, Hash)]
struct CanonicalFloat(u64);
```

does not automatically implement Telos semantic equality.

## 18.1 Static storage vs language equality

It is acceptable for the static store to preserve exact f64 bits for materialization.

But semantic equality/hash operations must obey Telos rules.

## 18.2 Hash-consing problem

If StaticValueStore itself interns by semantic equality:

```text
+0.0 and -0.0
```

would collapse and lose the sign bit needed for later materialization.

Therefore recommended separation:

```text
representation identity:
    exact IEEE bits

language equality:
    Telos Float == semantics

language hash:
    canonical equal-value hash
```

The store may intern floats by exact representation bits while structural
language equality compares semantically.

## 18.3 NaN

Because public NaN is not equal to itself, `StaticValue` cannot simply derive
Rust `Eq` and claim it is language equality.

Use explicit semantic comparison/hash APIs.

Internal arena keys may use exact bits.

---

# 19. String semantics

Ratified C4 rule:

```text
static String equality is content equality
```

Store:

```text
UTF-8 content / Arc<str> / content-interned ID
```

Never:

```text
VM string ObjRef
compiler arena address
```

`===` identity-sensitive String operations should not be reduced unless language semantics define an allocation-independent result.

P2 should reject/residualize identity-sensitive runtime allocation semantics.

---

# 20. Symbol semantics

Static Symbol values need stable semantic spelling/selector identity.

Do not use VM:

```text
Symbol(u32)
```

as semantic identity.

Use:

```text
canonical spelling / stable semantic symbol representation
```

Compiler materialization interns it into the destination VM later.

This is essential for P5/P8.

---

# 21. Tuple, Record, Data, Variant static values

P2 should establish structural ordinary-value nodes now because later reflection values are ordinary static ADTs.

## 21.1 Tuple

Static Tuple:

```text
ordered fixed components
```

Use child `StaticValueId`s.

Do not depend on runtime TupleObject.

The zero-arity language semantics must follow current Tuple/Unit law exactly.

If runtime/spec says zero-arity Tuple normalizes to Unit, static semantics must match.

## 21.2 Record

P2 only needs the current pre-P4 key domain.

Do not implement generalized FieldKey early.

Use current canonical record semantic field identity/order and design the node so P4 can replace/generalize key representation without replacing StaticValueStore.

## 21.3 Data

Static ordinary runtime `data` values should be represented by canonical semantic declaration/specialization plus structural payload.

Conceptually:

```rust
StaticDataValue {
    declaration / specialized type,
    constructor identity,
    components: ...
}
```

Do not infer fields from source syntax.

Consume canonical DataInfo/Data declaration products.

## 21.4 Variant

Use exact:

```text
VariantId
```

plus static payload.

Do not identify by name Symbol alone.

This prepares P5 reflection while preserving repeated variant families.

---

# 22. C3 `StaticTerm` bridge

P2 must not duplicate C3 static/type data in StaticValue.

Recommended rule:

```text
C3 kinded semantic term
    represented by TypeId
    wrapped/referenced as StaticValue::StaticTerm(TypeId)
```

Examples:

```text
type data constructor term
C3 ConstInt term
TypePack term
associated static-domain term
```

A proper Type uses:

```text
StaticValue::Type(TypeId)
```

so domain checks can distinguish:

```text
proper Type
other static term
ordinary compile-time value
```

---

# 23. Static-definition body representation

## 23.1 Checked body product, not raw AST authority

A StaticDefinition may retain source AST handles/ranges for execution during P2, but its semantic product must also contain the checked resolution products needed to avoid re-resolving meaning.

Conceptually:

```rust
pub struct CheckedStaticBody {
    pub source: StaticDefinitionBodySource,
    pub expressions: ExpressionAnalysisIndex,
    pub bindings: ...,
    pub static_operations: StaticOperationResolutionIndex,
    pub associated_resolutions: ...,
    pub call_resolutions: ...,
    pub diagnostics: ...,
}
```

Exact fields should reuse current analysis products.

## 23.2 Do not serialize raw AST as durable residual

P2 may keep AST references in-process because P7/P8 have not frozen residual/export.

But no design decision should make raw AST the semantic residual format.

---

# 24. Reuse ordinary semantic expression checking without pretending a static definition is a runtime method

Current expression checking is rich and already owns:

```text
dispatch selection
operator resolution
associated resolution
generic specialization
type knowledge
denotations
flow
if branch checking
match resolution
binding facts
```

P2 should reuse these semantics.

## 24.1 Rejected shortcut: synthetic runtime callable as permanent identity

Creating a fake method such as:

```text
$static_F
```

solely to reuse `CallableAnalysis` risks:

```text
runtime callable identity pollution
incorrect dispatch/tooling exposure
wrong metadata category
```

## 24.2 Recommended design

Add a semantic body-owner abstraction:

```rust
enum SemanticBodyOwner {
    Callable(CallableId),
    StaticDefinition(DeclarationId),
    StaticConstant(StaticConstantId), // where a body analysis is needed
}
```

and refactor only the checker context/explanation/dependency points that truly assume `CallableId`.

Publish:

```rust
StaticDefinitionAnalysis
```

instead of pretending it is a runtime callable.

## 24.3 Practical bounded alternative

If fully generalizing CheckingContext is disproportionately invasive, a private query-local synthetic body identity may be used internally only if:

```text
it never enters dispatch
it never enters runtime metadata
it never becomes a public CallableId
it is deterministically tied to the StaticDefinition identity
```

This should be treated as an implementation adapter, not semantic identity.

---

# 25. Static operation identity

This is a hard P2 correctness invariant.

Current binary expressions are semantically resolved through normal dispatch.

Therefore:

```telos
a + b
```

can mean user-defined behavior.

P2 must not do:

```rust
match binary.op {
    Add => eval_numeric_add(...)
}
```

without consulting semantic resolution.

## 25.1 Required model

During static body checking, classify exact semantically selected operations:

```rust
pub enum StaticIntrinsicOperation {
    IntAdd,
    IntSubtract,
    IntMultiply,
    IntDivide,
    IntFloorDivide,
    IntModulo,
    IntPower,
    IntShiftLeft,
    IntShiftRight,
    IntBitAnd,
    IntBitOr,
    IntBitXor,
    IntNegate,
    ...
    FloatAdd,
    ...
    NumericCompare,
    BoolNot,
    ...
}
```

Exact inventory should be deliberately scoped to P2.

Publish:

```rust
StaticOperationResolution {
    expression: ExpressionId,
    operation: StaticIntrinsicOperation,
}
```

## 25.2 Selection authority

Select the static intrinsic only from canonical semantic identity such as:

```text
exact core CallableId
intrinsic semantic operation ID
explicit compiler-owned static intrinsic registry
```

Never from:

```text
selector spelling alone
method name alone
receiver name string
AST operator token alone
```

## 25.3 User overloads

If semantic dispatch selected a user implementation of `+`:

```text
P2 does not execute it
```

Evaluation of that path returns:

```text
Invalid(static.runtime_call_not_allowed)
```

or future P7 symbolic behavior where appropriate.

C4 explicitly does not add const methods/functions.

---

# 26. Static operation registry

Create one compiler-owned registry mapping exact semantic callable/operation identity to a static evaluator operation.

Conceptually:

```text
CoreDeclarationIds / canonical callable IDs
    ↓
StaticIntrinsicRegistry
    ↓
StaticIntrinsicOperation
```

This registry should be initialized from semantic/core declaration identity, not runtime primitive function pointers.

It should be testable without constructing a VM.

---

# 27. Short-circuit and untaken-branch semantics

A crucial rule:

> An unsupported operation in an untaken branch does not make a closed static computation invalid.

Example:

```telos
const x =
    if false {
        runtimeOnlyOperation()
    } else {
        1
    }
```

must reduce to 1 if the source is otherwise semantically valid.

## 27.1 Do not implement an eager “static eligibility scan”

It is incorrect to walk every expression in a static body before evaluation and reject every runtime-only call.

Static execution eligibility is path-sensitive.

## 27.2 Semantic checking vs static execution

Semantic checker:

```text
resolves/types both source branches as ordinary source semantics require
```

Evaluator:

```text
executes only selected branch
encounters/rejects runtime-only operation only if path executes it
```

## 27.3 Boolean operators

`and` / `or` must preserve Telos short-circuit semantics.

Do not evaluate both operands merely because current ordinary expression analysis visits both for type checking.

---

# 28. Static evaluator control machine

Use an explicit machine from P2 onward.

Recommended modules:

```text
phalcom-semantic/src/static_eval/
    mod.rs
    value.rs
    domain.rs
    definition.rs
    context.rs
    frame.rs
    eval.rs
    control.rs
    operation.rs
    diagnostic.rs
    materialize.rs
```

Later:

```text
P6:
    pattern.rs
    reflect.rs / iteration.rs as needed

P7:
    memo.rs
    residual.rs
    specialization.rs
```

## 28.1 Control enum

Reserve:

```rust
pub enum EvalControl {
    Value(StaticValueId),
    Return(StaticValueId),
    Break,
    Continue,
}
```

P2 may reject explicit Return/Break/Continue until P6.

Do not force P6 to replace the evaluator state machine merely to add control.

## 28.2 Explicit frames

A frame should retain:

```text
definition/application identity
current expression/statement continuation
static parameters
lexical static local slots
result-domain expectation
semantic checked-body reference
budget
cancellation token
diagnostic trace/call chain
```

## 28.3 No unbounded host recursion

Nested non-recursive definition applications should still be represented as evaluator frames.

P7 then adds recursive application semantics without converting recursive Telos into Rust stack recursion.

---

# 29. P2 recursion policy

P7 owns general recursion.

P2 must still fail safely on cycles.

## 29.1 Required P2 behavior

If evaluation encounters a static definition already active in its application chain:

```text
return StaticEvalOutcome::Recursive
```

P2 may conservatively reject all definition recursion, even with changed arguments.

P7 later refines this into:

```text
exact-key cycle
vs
changing-argument legal recursion
```

## 29.2 Do not silently turn recursion into budget exhaustion

Keep `Recursive` distinct from:

```text
BudgetExceeded
Invalid
```

This preserves the outcome architecture P7 expects.

---

# 30. Static evaluator local state

Static-definition block locals are semantic evaluator slots.

## 30.1 `let`

```telos
let x = ...
```

immutable evaluator local.

Initializer required.

## 30.2 `let mut`

```telos
let mut x = ...
let mut x: Int
```

mutable evaluator local.

P1 ordinary semantics allow missing initializer.

Inside static evaluation:

```text
uninitialized slot is evaluator state
not runtime ValueTag
```

An executed read before initialization is a static evaluation error.

There is no runtime checked-read fallback inside a computation that is supposed to complete at compile time.

## 30.3 Assignment

Assignment updates the evaluator-local value after ordinary semantic type checking has already validated the write.

Do not rerun the type relation in the evaluator unless needed as a defensive assertion.

## 30.4 Flow precision

The evaluator follows one concrete static path.

It does not need P1's MaybeInitialized runtime fallback for its own local slots after a condition has reduced.

---

# 31. P2 block result semantics

Use the same lexical-block trailing-expression convention as existing semantic control flow.

The live checker already has executable-region analysis that can produce a branch/trailing value.

P2 static blocks should consume that semantic model.

Do not define a different “last statement rules for static block.”

---

# 32. `if` semantics

P2 supports ordinary `if`.

Required:

```text
evaluate condition
require reduced Bool
evaluate exactly one branch
preserve lexical local scope
produce branch value
```

If condition cannot reduce in a closed const:

```text
closed const fails
```

A generic static definition is not evaluated until it has arguments; its checked body may therefore contain parameter-dependent `if`.

---

# 33. P2 intentionally does not implement static `while`

The live parser currently still represents ordinary `while` using historical `whileTrue` lowering in one path.

C4 has ratified that static `while` must be an explicit semantic control-flow construct and must never recognize `whileTrue`.

Therefore:

```text
P2:
    no static while

P6:
    migrate/preserve explicit while semantic identity
```

P2 must not build support around current `whileTrue`.

If an executed P2 static body contains that method call:

```text
runtime/static-invalid operation
```

until P6 changes the semantic representation.

---

# 34. P2 intentionally does not implement `match`

Although the AST already has resolved match semantics, C4 assigns static match and pattern/GADT reuse to P6.

P2 should not implement a simplified match evaluator.

Doing so would create exactly the duplicate pattern engine C4 forbids.

---

# 35. References available to static evaluation

An evaluated static expression may resolve a reference to:

```text
static parameter
evaluator-local let/let mut/const
module static constant
imported source-workspace static constant
Type declaration/value
C3 static-domain declaration/term
checked static definition
compiler-owned static intrinsic operation
```

It may not read:

```text
runtime local let/let mut
method parameter
self runtime instance
runtime field
mutable runtime global
runtime closure capture
runtime object
```

unless a later static semantic feature explicitly defines a compile-time representation.

## 35.1 Optimizer knowledge is irrelevant

This must fail:

```telos
let x = 42
const y = x
```

even if optimizer can prove x == 42.

Staticness is declared semantic phase, not optimization fact.

---

# 36. Static constant evaluation query

For a top-level `const`:

```text
QueryKey::StaticConstant(SymbolId)
```

should:

1. locate the declaration/initializer by stable source index/shard identity;
2. resolve declared value type/domain if present;
3. check initializer semantics;
4. evaluate with `StaticEvalContext`;
5. validate final value against declared domain;
6. publish value/materialization/dependencies/fingerprint;
7. publish diagnostics.

No compiler bytecode is involved.

---

# 37. Source-shard/incremental input changes

Current `ModuleSemanticStructureShard` retains:

```text
alias_sources
callable body fingerprints
field fingerprints
...
```

but no dedicated static-constant input map.

P2 should add targeted source inputs such as:

```text
static_definition_sources: DeclarationId -> ...
static_definition_fingerprints: DeclarationId -> u64

static_constant_sources: SymbolId -> LetBinding/static declaration source
static_constant_fingerprints: SymbolId -> u64
```

Exact names flexible.

## 37.1 Why this matters

Without targeted fingerprints:

```text
editing any unrelated module source
```

may unnecessarily invalidate every static computation, or static initializer value changes may fail to invalidate the right query.

P2 should integrate with existing fine-grained DB practice.

---

# 38. Static value changes must affect product fingerprints

This is a non-negotiable C4 requirement.

If:

```telos
const Timeout = 10
```

changes to:

```telos
const Timeout = 20
```

then a dependent static definition/application must invalidate even if:

```text
type remains Int
```

Therefore:

```text
StaticConstant product fingerprint
```

must include semantic value content or a canonical value-content fingerprint.

Do not fingerprint only:

```text
value type
source declaration identity
result domain
```

---

# 39. Static-definition fingerprints

Fingerprint semantic contract:

```text
definition identity
parameter domains/modes
where constraints that P2 supports
result domain
checked body semantic operation/resolution content
semantic dependencies
```

Do not make semantically irrelevant source-range movement part of the durable product fingerprint.

Input/source fingerprints may change with text; product fingerprints should reflect semantic meaning.

---

# 40. Cancellation and incomplete query outcomes

Current DB already separates:

```text
Ready
Blocked
Cancelled
BudgetExceeded
Failed
```

P2 must preserve this.

## 40.1 Never publish cancellation as a static value

Cancelled evaluation is not:

```text
Reduced(None)
Reduced(Unit)
Unknown static value
```

## 40.2 Never reuse stale successful constant as current compiler truth

Editor last-known-good data may remain available through existing DB policy for presentation.

Compiler/current-generation semantic truth must not materialize the previous constant value after the current initializer becomes invalid/cancelled/budget-exceeded.

---

# 41. Query budget integration

Current `QueryBudget` has:

```text
Steps
RelationPairs
SccIterations
TypeDepth
DiagnosticNotes
```

P2 should use it directly.

## 41.1 Charge at least

```text
each evaluated expression
each executed statement
each static-definition application
each nontrivial intrinsic static operation
each constructed structural element
each projection/index traversal proportional to work
```

## 41.2 No hidden MAX_CONST_STEPS

Do not add:

```text
const MAX_STATIC_LOOP = ...
const MAX_CONST_RECURSION = ...
```

with independent semantics.

## 41.3 Static value growth

The current budget has no explicit memory/value-node dimension.

P2 should decide one of:

### Preferred

Add:

```text
BudgetKind::StaticValueNodes
max_static_value_nodes
static_value_nodes_created
```

or semantically equivalent.

### Minimum acceptable

Charge `Steps` proportional to every created static node/element and prove that no single operation can allocate unbounded value structure for one step.

The preferred explicit value-growth dimension gives much better diagnostics and memory-DoS control.

---

# 42. Cancellation

Use the existing `CancellationToken`.

Check:

```text
at frame transitions
at definition applications
during large tuple/record/data builders
during large arithmetic operations if implementation can be long-running
```

For exact BigInt operations, very large exponent/shift/multiply may require resource checks beyond expression count.

Do not assume “one arithmetic AST node == bounded cost.”

---

# 43. Numeric resource policy

Because Telos Int is exact/unbounded, P2 static arithmetic can allocate large integers.

Examples:

```telos
const x = 2 ** 100000000
const y = 1 << 100000000
```

must not trivially exhaust compiler memory.

P2 must reuse or establish the language/compiler numeric resource policy.

At minimum:

```text
budget before large power/shift
checked size estimation
deterministic BudgetExceeded / numeric-limit diagnostic
no host OOM as semantic outcome
```

Do not silently fall back to Float.

---

# 44. StaticEvalOutcome

Define the terminal model now:

```rust
pub enum StaticEvalOutcome {
    Reduced(StaticValueId),
    Symbolic(StaticResidualId),
    Invalid(StaticEvalError),
    Recursive(StaticCycle),
    BudgetExceeded(BudgetReport),
    Cancelled,
}
```

## 44.1 P2 `Symbolic`

P7 owns real residual IR.

P2 may use:

```text
Symbolic placeholder / unsupported-open outcome
```

only as a structural API seam.

A closed `const` with Symbolic is an error.

Do not create raw-AST residuals “temporarily.”

## 44.2 Open static definitions

A generic StaticDefinition is a checked definition, not a failed Symbolic evaluation merely because parameters are unbound.

Evaluate it when concrete static arguments are supplied.

This avoids requiring P7 residualization just to declare a generic static definition.

---

# 45. Result-domain validation

After an application/const reduces:

```text
StaticValue
    ↓
domain_of(value)
    ↓
check against declared StaticValueDomain
```

## 45.1 Kind lane

For:

```text
Kind(expected)
```

the value must be a canonical semantic term of that kind.

Use existing kind checking.

## 45.2 ValueType lane

For:

```text
ValueType(expected_type)
```

the static value has an ordinary Telos value type.

Validate using the existing assignability/subtyping relation.

Do not hard-code:

```text
if result is Bool variant -> Bool okay
```

per primitive.

This lets future:

```text
ProductShape
Variants
user static ordinary data
union result types
```

use the same relation machinery.

---

# 46. Type-valued static values

`StaticValue::Type(TypeId)` must preserve the proper-type boundary.

Do not allow:

```text
non-Type C3 StaticTerm
```

to satisfy result:

```telos
: Type
```

merely because both carry `TypeId`.

Use:

```text
ProperTypeId / kind(TypeId) == Type
```

as current architecture requires.

---

# 47. Static intrinsic operation coverage for P2

Recommended minimum.

## 47.1 Bool

```text
not
and
or
==
!=
```

Short-circuit syntax/semantics preserved.

## 47.2 Int

Exact semantic forms of:

```text
+
-
*
/
floor/integer division as language specifies
%
**
<<
>>
&
|
^
unary -
comparisons
<=>
equality
```

Do not implement operations whose exact current runtime semantic contract has not been audited.

## 47.3 Float

Only operations whose current language contract is deterministic and exactly reproducible in semantic code.

Need test:

```text
NaN
infinities
signed zero
division behavior
comparison
```

## 47.4 String

P2 may support:

```text
literal
content equality
possibly concatenation
```

only through a static intrinsic identity.

Do not support arbitrary String runtime methods.

## 47.5 Symbol

```text
literal
semantic equality
```

Do not use VM interner identity.

## 47.6 Type/C3 terms

Reuse existing semantic type/static operations.

Do not call runtime type reflection.

---

# 48. Avoid duplicating numeric/runtime semantic kernels where possible

A danger is implementing:

```text
runtime Int +
and
static Int +
```

with drifting edge behavior.

Ideal direction:

```text
shared pure numeric semantic kernel
    ├─ runtime primitive wrapper
    └─ static evaluator wrapper
```

provided the shared layer can remain below `phalcom-core` and does not introduce a semantic→core dependency.

If extracting the entire numeric kernel is too large for P2, P2 must at minimum copy the **spec semantics**, add edge parity tests, and record the duplication for later consolidation.

Do not call VM primitives directly.

---

# 49. Static equality

P2 needs explicit semantic equality for StaticValue.

It should dispatch by static value domain/shape, not runtime method invocation.

Required:

```text
Unit
Bool
Int
Float
String
Symbol
Type
StaticTerm
Tuple
Record
Data
Variant
```

For structural values:

```text
recursive semantic equality over acyclic DAG nodes
```

For Type:

```text
canonical semantic Type equality
```

Never:

```text
reflect Type::Shape and compare recursively
```

---

# 50. Materialization descriptor

P2 defines semantic materialization independent of VM heap objects.

Conceptually:

```rust
pub enum MaterializationNode {
    Unit,
    Bool(bool),
    Int(CanonicalInt),
    FloatBits(u64),
    String(StaticStringId),
    Symbol(StaticSymbolId),
    Type(TypeId),
    Tuple(Box<[MaterializationId]>),
    Record(...),
    Data { declaration, specialization, fields: ... },
    Variant { variant: VariantId, payload: ... },
}
```

Prefer an indexed DAG/pool:

```text
MaterializationDescriptorId
```

rather than deeply recursive Rust trees.

## 50.1 Descriptor requirements

```text
deterministic
no ObjRef
no VM Symbol
no heap address
no initializer AST
no evaluator continuation
```

## 50.2 Descriptor formation

It should be a pure projection:

```text
StaticValue -> MaterializationDescriptor
```

It must not re-run static computation.

---

# 51. P2 vs P8 materialization boundary

The original C4 decomposition gives P8 the full durable/compiler materialization closure.

P2 must nevertheless establish a vertical seam.

Recommended P2 contract:

1. every materializable P2 StaticValue can produce a semantic descriptor;
2. `ModuleLoweringSemantics` can carry static-constant descriptor/spec attachments;
3. the compiler never sees/evaluates the initializer AST for a const;
4. P2 may materialize a minimal same-module primitive subset if cheap;
5. complex/durable/artifact-crossing materialization remains P8.

## 51.1 Open scope choice to ratify before P2 plan

Two coherent choices:

### P2-M1 — semantic-only descriptor vertical

```text
const evaluates
descriptor publishes
compiler can inspect attachment
runtime consumption of const remains feature-gated until P8
```

Pros:
- strict decomposition.

Cons:
- `const answer = 42; print(answer)` remains unusable after P2.

### P2-M2 — recommended minimal same-module materialization

P2 additionally lowers:

```text
Unit
Bool
Int
Float
String
Symbol
possibly Type
```

from descriptor to existing runtime constant/bytecode forms.

Tuple/Record/Data may follow if straightforward.

P8 owns:

```text
durable metadata
artifact imports
large DAG pools
general structural static values
cross-module materialization
final schema
```

This gives P2 an end-to-end user-visible const vertical without taking P8's durability work.

**Recommendation: P2-M2**, if implementation remains descriptor-driven and bounded.

---

# 52. ModuleLoweringSemantics integration

Current core semantic projection has a mature:

```rust
ModuleLoweringSemantics
```

that already carries:

```text
bindings
binding read safeties
type-form values
constructor products
associated resolutions
matches
iteration lanes
...
```

P2 should add static projections here.

Conceptually:

```rust
pub struct StaticConstantLoweringSpec {
    pub constant: StaticConstantId,
    pub materialization: MaterializationDescriptorId / projected descriptor,
    pub value_type/domain: ...,
}

pub struct ModuleLoweringSemantics {
    ...
    pub static_constants: BTreeMap<SymbolId, StaticConstantLoweringSpec>,
    pub local_static_constants: ...,
}
```

Compiler lowering consumes this.

It must not query/evaluate semantic source itself.

---

# 53. P1 compiler const sets after P2

Current:

```text
global_static_constants
static_const_scopes
```

should be demoted to:

```text
scope/redeclaration/backstop knowledge
```

or replaced by semantic lowering attachments.

`is_static_constant(name)` must not become the value resolution API.

Recommended lowering rule:

```text
reference resolved by semantic attachment
    ↓
static constant spec/materialization

missing attachment for known static const
    ↓
fail closed
```

Do not fall back to runtime global lookup.

---

# 54. Semantic denotation extensions

Current `SemanticDenotation` includes:

```text
TypeForm(TypeId)
Kind(KindId)
StaticDomain(StaticDomainId)
StaticTerm(TypeId)
AssociatedValue(...)
BehavioralFamily(...)
```

P2 should extend it with static-definition/constant references where useful.

Recommended:

```rust
StaticConstant(StaticConstantId)
StaticDefinition(DeclarationId)
StaticValue(StaticValueId) // only if snapshot-store lifetime is rigorously safe
```

Prefer stable identity denotations for source references and keep actual values in query/store products.

This reduces stale-ID propagation through general expression products.

---

# 55. Top-level const reference typing

Once:

```telos
const Answer = 42
```

has a StaticConstantProduct, an expression referencing `Answer` must have:

```text
ordinary value type knowledge = Int
denotation = StaticConstant(AnswerId)
phase = static source value
```

Compiler/static evaluator can then distinguish:

```text
type of value
from
source static identity
```

No runtime slot is needed.

---

# 56. Static constants referencing constants

P2 must support dependency chains:

```telos
const A = 10
const B = A + 2
const C = B * 3
```

Requirements:

```text
canonical query dependency edges
single evaluation per query/revision
value fingerprint propagation
cycle detection
no source-order recomputation hacks
```

Whether forward references are legal should preserve existing module name-resolution law.

P2 must VERIFY-FIRST; do not invent new forward-reference semantics solely for constants.

---

# 57. Constant cycles

Example:

```telos
const A = B
const B = A
```

must terminate deterministically.

Return:

```text
Recursive / blocked recursive static dependency
```

with diagnostic chain.

Do not stack overflow.

Do not publish one old value as current.

---

# 58. Cross-module source-workspace constants

P8 owns durable compiled-artifact static metadata.

P2 can still support source-workspace dependency if current linked/source semantic graph provides provider source.

Required distinction:

```text
same workspace/source available
    may evaluate provider StaticConstant query

artifact-only imported constant without P8 value metadata
    fail closed / unsupported until P8
```

Do not materialize imported const by reading a runtime global slot.

---

# 59. Static-definition source indexing

Once type-definition bodies become Expr/LexicalBlock, source index must traverse them.

Need:

```text
parameter declaration sites
local let/let mut sites
type/value references
static definition application references
associated references
ordinary call references for diagnostics/navigation even if runtime-static-invalid
```

P9 owns final tooling polish, but P2 source identity cannot be absent.

---

# 60. Static-definition result application in expression/type contexts

P2 must resolve the same `type F<...>` declaration differently by its declared result domain.

## 60.1 Kind(Type) result

```telos
type Pair<A: Type, B: Type> = (A, B)
```

application yields a Type static value and remains usable in ordinary type-form positions.

## 60.2 ValueType(Bool) result

```telos
type IsZero<N: Int>: Bool = N == 0
```

application is a compile-time Bool value, not a Type form.

## 60.3 Parser/semantic implication

The existing AST name `Expr::TypeForm` is too narrow if it becomes the sole value-space representation of all `type` definition applications.

P2 has two options:

### Option A — generalized static form expression

Introduce:

```rust
Expr::StaticForm(...)
```

or a generalized application node.

### Option B — keep syntax node, generalize semantic denotation

Retain `Expr::TypeForm(TypeAnnotation)` as a source parser carrier, but semantic resolution determines whether the target is:

```text
Type value
StaticTerm
StaticDefinition application yielding ordinary value
```

This is mechanically smaller but leaves a misleading AST name.

**Recommendation:** use a generalized static-form abstraction if it can be introduced without destabilizing existing parser/type-formation paths. Otherwise record `Expr::TypeForm` as a compatibility parser carrier and keep the semantic model correctly generalized.

---

# 61. Static definition parameters in body name resolution

A body reference:

```telos
type IsZero<N: Int>: Bool =
    N == 0
```

must resolve `N` as:

```text
static value parameter
```

not runtime local and not TypeParameterId.

A body reference:

```telos
type Pair<T: Type>: Type =
    T
```

resolves as:

```text
kinded static term parameter
```

Both need one lexical static environment.

Conceptually:

```rust
enum StaticBindingValue {
    KindedTerm(TypeId / static argument),
    Value(StaticValueId),
}
```

---

# 62. Where clauses

Current generic where clauses are Type/trait/kind constraint oriented.

P2 should preserve existing where-clause semantics for kinded Type parameters.

Do not invent arbitrary value predicates in `where` for ordinary value-domain parameters unless already ratified.

For P2:

```text
value-domain parameter constraints
    come from declared ValueType domain
```

Body expressions can perform value comparisons.

Future richer static constraints may be ordinary static definitions rather than type-system where relations.

---

# 63. `TypeMemberDomain` interaction

Current semantic:

```rust
pub enum TypeMemberDomain {
    Kind(KindId),
}
```

Current AST already has ConstType/result Type syntax.

P2 should create the canonical shared `StaticValueDomain`.

Two implementation strategies:

## Strategy A — generalize TypeMemberDomain in P2

```rust
pub enum TypeMemberDomain {
    Kind(KindId),
    ValueType(TypeId),
}
```

or alias it to shared domain.

Pros:
- one domain model immediately;
- P5 reflection members do not require another migration.

Cons:
- may expand P2 into existing type-member application validation paths.

## Strategy B — shared domain introduced, type members adapt in P5

P2 StaticDefinition uses shared domain; current type members retain adapter:

```text
TypeMemberDomain::Kind
```

until P5.

Pros:
- smaller P2.

Cons:
- temporary adapter.

**Recommendation:** introduce the shared semantic type in P2 and migrate TypeMemberDomain if the change is mechanical; otherwise preserve an explicit adapter with a P5 removal marker. Do not invent a second conceptual domain.

---

# 64. Static evaluator and runtime/native calls

Executed paths must reject:

```text
ordinary user method calls
ordinary user getters/setters
native VM primitives not registered as static intrinsic
runtime object construction requiring runtime identity
I/O
filesystem/network/process
clock/random
fibers
scheduler/thread state
mutable runtime globals
runtime field reads
runtime closure execution
arbitrary toString/interpolation
identity-dependent ===
```

Diagnostics must identify the actual semantic target selected.

---

# 65. Data constructors in static evaluation

Construction is not “calling arbitrary user code” if the constructor is a compiler-known pure structural data/variant constructor and all arguments are static.

Use canonical semantic constructor products.

Do not execute user constructor methods.

Distinguish:

```text
data/variant structural constructor
    allowed static formation

class @constructor / arbitrary method body
    not P2 static execution
```

This prepares structural reflection values without creating compile-time object semantics for arbitrary classes.

---

# 66. Record construction

P2 supports current canonical fixed Record construction where keys are statically known under current pre-P4 rules.

Computed runtime-only keys are not static fixed shape.

P4 generalizes FieldKey.

Do not preempt P4 with ad hoc String/Int/Bool record key support in P2.

---

# 67. Projection/index

Allow only when semantic operation identity is static-safe and deterministic.

Examples:

```text
Tuple fixed index
Record fixed field/key
static data component
C3 type/static term projection
```

Do not run arbitrary `[]` overloads merely because syntax is IndexExpr.

Current semantic analysis records call/associated resolution; consume it.

---

# 68. Static definition declaration vs evaluation

A generic definition:

```telos
type IsZero<N: Int>: Bool = N == 0
```

must be checkable without evaluating `N`.

Therefore P2 has two stages:

```text
formation/check:
    validate body semantics and static parameter resolution
    publish StaticDefinition

application:
    bind concrete static arguments
    evaluate body
    validate result domain
```

Do not demand that every generic definition reduce during declaration.

That would prematurely require P7 residualization.

---

# 69. StaticDefinition checked-body eligibility

Do not eagerly reject all runtime-only operations at definition declaration time merely because they might be unreachable for some static argument.

Example:

```telos
type F<N: Int>: Int =
    if N == 0 {
        1
    } else {
        runtimeOnly()
    }
```

Declaring F may be semantically valid.

Applying:

```text
F<0>
```

can reduce.

Applying:

```text
F<1>
```

fails static evaluation.

This is consistent with C4's untaken-branch rule.

A stricter language policy could reject such definitions, but that has not been ratified.

P2 should use path-sensitive execution failure.

---

# 70. Diagnostics

Recommended stable families.

## Static constants

```text
const.non_static_dependency
const.static_evaluation_failed
const.symbolic_result
const.result_domain_mismatch
const.result_not_materializable
const.recursive_dependency
```

## Static definitions

```text
static_definition.invalid_body
static_definition.result_domain_mismatch
static_definition.argument_domain_mismatch
static_definition.recursive_application
static_definition.unsupported_control
```

## Static evaluation

```text
static.invalid_operation
static.runtime_call_not_allowed
static.runtime_binding_not_allowed
static.runtime_field_not_allowed
static.identity_not_available
static.budget_exceeded
static.value_budget_exceeded
static.cancelled          # usually internal/editor state, not persistent user error
```

Diagnostics should include:

```text
definition/const call chain
source operation
selected semantic target where applicable
static dependency that caused failure
```

---

# 71. Diagnostic causality

Avoid cascades.

If the static body is already semantically invalid:

```text
type mismatch / unresolved name / invalid call
```

do not also emit generic:

```text
static_evaluation_failed
```

unless it adds non-duplicated context.

Use existing causal invalidity/suppression infrastructure.

---

# 72. Materialization errors are not static evaluation errors

Distinguish:

```text
value failed to compute
vs
value computed but current compiler/runtime cannot materialize it yet
```

This matters during P2/P8 staging.

Outcome:

```text
Reduced(value)
```

may coexist with:

```text
materialization unsupported
```

for a later-plan value kind.

Do not lose the valid static value merely because runtime lowering is not yet complete.

---

# 73. Compiler error migration

Current P1:

```text
StaticConstUnmaterialized
```

means “P2 value does not exist.”

After P2, split as necessary:

```text
StaticConstantMissingSemanticValue     internal invariant failure
StaticConstantMaterializationUnsupported
```

The old error may remain as a defensive fallback but should no longer be the normal result of a successfully evaluated primitive const.

---

# 74. StaticValue and Materialization fingerprints

Static value fingerprint must be by canonical semantic contents.

For:

```text
String
Symbol
Type
Data
Variant
```

never hash:

```text
arena ID
VM symbol index
ObjRef
source range
```

For Float, define representation/equality/hash intentionally as discussed above.

---

# 75. Metadata boundary

Current type metadata schema is v10.

P2 should **not** finalize the C4 metadata schema.

P8 owns schema publication for:

```text
ordinary static constants
residual definitions
FieldKey
Pack structural values
general static member result domains
```

P2 may create internal staging records/tests.

Do not bump schema merely to serialize an unstable P2 StaticValue IR unless cross-module P2 acceptance genuinely requires it.

---

# 76. Cross-module imports before P8

For live source workspace:

```text
semantic DB can evaluate provider source
```

For precompiled artifact:

```text
no durable ordinary static value metadata yet
```

P2 should fail explicitly rather than pretending imported static const is runtime global storage.

P8 closes this.

---

# 77. Static value memory model

Prefer:

```text
small immutable nodes
hash-consing/share
child IDs
compact strings
specialized product sub-arenas if needed
```

Avoid:

```text
clone whole tuple/record/data on each operation
nested HashMap<String, StaticValue>
deep recursive Box trees
```

Static values should form an acyclic DAG.

---

# 78. Evaluator environment representation

Do not use:

```rust
HashMap<String, StaticValueId>
```

for every local read.

Semantic checking already resolves binding identity.

Use indexed/static binding slots:

```text
StaticLocalId -> slot index
```

Frame environment:

```text
Vec<StaticSlot>
```

or small structured arrays.

This matters for P6/P7 compile-time algorithms.

---

# 79. StaticSlot

Conceptually:

```rust
enum StaticSlot {
    Uninitialized,
    Initialized(StaticValueId),
}
```

Mutability is known from checked binding product.

No VM sentinel.

No Option conflation if `None` is itself a legitimate static value.

---

# 80. Static argument environment

Separate:

```text
definition parameters
lexical locals
```

so recursive P7 applications can create clean frames.

Parameter environment should be immutable per frame.

---

# 81. Explicit evaluator trace

For diagnostics and recursion:

```text
StaticEvalTraceFrame {
    definition / constant identity,
    application/source site,
}
```

Keep source spans diagnostic-only.

Do not include them in semantic application equality/key unless source identity genuinely changes semantics.

---

# 82. P2 operation determinism

Every P2 reducible operation must be deterministic under:

```text
same compiler semantic model
same static arguments
same dependencies
```

No dependence on:

```text
wall clock
hash randomization
thread schedule
allocation order
VM interner state
locale
filesystem
environment variables
```

---

# 83. Error determinism

Where multiple errors are possible, use source/evaluation order.

Do not let HashMap iteration decide which static error surfaces first.

This is especially important for Record/data builders and dependency traces.

---

# 84. Source order and structural order

P2 evaluation should preserve source evaluation order for expression side effects/errors even though static values are immutable.

Example Record fields:

```text
evaluate field expressions in source order
then canonicalize structural storage if semantic record representation does so
```

Do not reorder evaluation merely because Record shape is unordered.

---

# 85. Static exceptions/throw

P2 should not implement general `throw`/exception handling as static execution unless explicitly planned.

If executed:

```text
static.unsupported_control
```

or a deliberate P2 static error.

P6 control scope does not currently include throw/catch.

Do not accidentally invoke runtime exception machinery.

---

# 86. `return`, `break`, `continue`

Reserve `EvalControl` cases.

P2 block bodies may parse these ordinary statements.

Executed P2 behavior should be a precise “not supported in P2 static control” diagnostic unless explicit return is deliberately included.

P6 owns final semantics.

---

# 87. No const functions or const methods

Do not add:

```telos
const fn
const method
@const
```

to solve evaluator reuse.

Reusable static source abstraction is:

```telos
type F<...>: Domain = ...
type F<...>: Domain { ... }
```

That boundary is ratified C4 architecture.

---

# 88. File-level implementation impact

Likely primary P2 source changes.

## AST/parser

```text
phalcom-ast/src/ast.rs
    TypeAliasDef / new TypeDefinitionDef
    StaticDefinitionBody
    shared StaticDomainSyntax
    static generic parameter syntax if generalized
    possibly generalized static application expression syntax

phalcom-ast/src/parser.rs
    canonical type declaration grammar
    generic-before-result-domain order
    = expression vs lexical block
    static domain parsing
    static argument parsing
```

## Semantic static foundation

New likely module:

```text
phalcom-semantic/src/static_eval/
    mod.rs
    value.rs
    domain.rs
    definition.rs
    operation.rs
    context.rs
    frame.rs
    eval.rs
    control.rs
    diagnostic.rs
    materialize.rs
```

## Existing semantic integration

```text
phalcom-semantic/src/lib.rs

phalcom-semantic/src/semantic_shard.rs
    static definition/constant source maps + input fingerprints

phalcom-semantic/src/snapshot.rs
    StaticValueStore / static product accessors

phalcom-semantic/src/type_alias.rs
    compatibility projection from StaticDefinition

phalcom-semantic/src/declarations.rs
    declaration shell / static-definition relation

phalcom-semantic/src/types/
    denotation.rs
    annotation.rs
    parameter.rs
    type_member.rs (shared domain or adapter)
    outcome.rs        (static value budget dimension if adopted)

phalcom-semantic/src/checker/
    context.rs
    analysis.rs / new static analysis product
    expression.rs
    statement.rs
    control.rs
    typed_expr.rs
    associated.rs as needed

phalcom-semantic/src/db/
    key.rs
    product.rs
    query.rs
    state.rs
    fingerprint.rs
```

## Modules

```text
phalcom-modules/src/interface.rs
    likely no conceptual new category; may expose declared static contract metadata if needed

phalcom-modules/src/linker.rs
    consume SymbolId identity; no runtime-slot semantics
```

## Core lowering

```text
phalcom-core/src/modules/semantic_lowering.rs
    static constant lowering/materialization specs

phalcom-core/src/modules/compile.rs
    transport semantic products

phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/error.rs
    replace P1 unmaterialized barrier with semantic descriptor consumption
```

If P2-M2:

```text
phalcom-core constant/chunk/materialization helpers
value/runtime creation only at compiler/runtime materialization boundary
```

## Tooling/source identity

Minimum:

```text
phalcom-semantic/src/source_index/*
phalcom-lsp hover/navigation only where new body/parameters otherwise become invisible
```

P9 owns full tooling polish.

---

# 89. Existing code that P2 must not duplicate

Reuse:

```text
TypeStore
KindId / KindData
StaticDomainId
TypeData::StaticData
TypeData::ConstInt as C3 term where applicable
StaticArgument
TypeAliasTable compatibility
relation/assignability engine
Callable/Expression semantic resolution machinery
AssociatedSurface
DataInfo / EnumInfo / VariantId
QueryBudget
CancellationToken
SemanticDb query/dependency/fingerprint system
ModuleLoweringSemantics projection architecture
source index semantic identity
```

---

# 90. Existing code that P2 must not treat as authoritative static evaluation

Do not reuse as semantic execution:

```text
phalcom-core VM
Bytecode evaluator
runtime primitive function pointers
Compiler constant folding
runtime Symbol interner
runtime Value
ObjRef
ProductStorage
runtime call dispatch
```

They are materialization/runtime consumers only.

---

# 91. Test architecture

P2 needs tests at several layers.

## 91.1 AST/parser

Required:

```text
legacy type Alias = TypeForm
type F<T> = expression
type F<T>: Int = expression
type F<T>: Int { ... }
no '=' before block body
generics appear before result domain
N: Int parsed as ordinary static-value domain, not StaticDomain kind
T: Type parsed as kind domain
S: Shape resolves as C3 static-domain kind
mixed parameters
malformed static domain
```

Also regression proving `{ ... }` static definition block is `LexicalBlock`, not Closure `Expr::Block`.

## 91.2 StaticValue

Primitive values:

```text
Unit
Bool
small Int
large Int beyond i64
Float finite
+0.0 / -0.0
NaN variants
Infinity
String UTF-8
Symbol
Type
StaticTerm
```

Structural:

```text
Tuple
Record
Data
Variant
nested combinations
DAG sharing
```

## 91.3 Domain checking

```text
Kind(Type) accepts Type value
Kind(Type) rejects ordinary Int
Shape static-domain accepts Shape term
ValueType(Int) accepts static Int
ValueType(Bool) rejects Int
subtype/union declared result contract if supported
```

## 91.4 Static parameter application

```text
type IsZero<N: Int>: Bool
IsZero<0>
IsZero<1>

mixed Type + Int parameter definition
Symbol parameter path
C3 type-data domain parameter path
wrong argument domain
wrong argument count
```

## 91.5 Evaluator control

```text
expression body
block trailing result
let
let mut assignment
uninitialized static local read
if true
if false
nested if
and/or short-circuit
untaken forbidden runtime call does not fail
taken forbidden runtime call fails
```

## 91.6 Semantic operation identity

Critical negative tests:

```text
user overload of +
    must not run statically

same selector name as core intrinsic
    must not grant static authority

native runtime method +
    not whitelisted
    must not run

core canonical Int + selected
    does run
```

## 91.7 Const

```text
primitive const reduces
const depends on const
const value edit invalidates dependent
runtime let dependency rejected
runtime global dependency rejected
cycle diagnosed
budget exhaustion
cancellation
closed symbolic result rejected
no runtime slot
no initializer bytecode
```

If P2-M2:

```text
runtime read of primitive const materializes descriptor result
initializer not re-executed
```

## 91.8 Type alias compatibility

Existing C1–C3 alias corpus must retain:

```text
same TypeId semantics
same substitution
same metadata behavior at P2 scope
same inference
same alias cycles/errors
```

Do not accept P2 if simple old type aliases regress.

---

# 92. Incremental tests

Mandatory.

## 92.1 Constant value-only edit

```text
const A = 1 -> const A = 2
```

Expected:

```text
StaticConstant(A) recomputes
dependent static definition/constant recomputes
unrelated callable/declaration products reuse
```

## 92.2 Equivalent source edit

Whitespace/range-only edit:

```text
semantic static product fingerprint stable
```

subject to existing input fingerprint policy.

## 92.3 Definition body edit

```text
type F<N:Int>:Int = N + 1
->
type F<N:Int>:Int = N + 2
```

Definition product changes; dependents invalidate.

## 92.4 Unrelated module edit

Unrelated static constant should reuse.

## 92.5 Invalid current revision

Valid const -> invalid initializer.

Compiler must not use last-known-good value for current build.

## 92.6 Cancellation

Cancelled query must not publish Ready reduced value.

---

# 93. Cold/incremental parity

Even though P8 owns checkpoint-wide durability, P2 must prove in-process parity for its new products:

```text
cold semantic analysis
vs
incremental edit sequence ending in same source
```

must produce equivalent:

```text
StaticDefinition semantics
StaticConstant value
StaticValue content
materialization descriptor
diagnostics
```

No reliance on insertion order.

---

# 94. Performance requirements

## 94.1 Static value storage

No deep cloning per operation.

Structural values should share child IDs.

## 94.2 Evaluator locals

Indexed slots.

No string hash lookup on each local read/write.

## 94.3 TypeStore

No whole TypeStore clone per application.

## 94.4 AST

No AST clone per evaluation.

## 94.5 Definition checking

Checked body should be reused across applications.

Do not re-run semantic dispatch/type analysis for every concrete value argument unless specialization genuinely requires it.

## 94.6 Static operations

Budget checks should be cheap on ordinary small computations.

## 94.7 Dependency chains

Const dependency lookup should use semantic query cache.

No module-wide rescans.

---

# 95. Large-value stress tests

At least:

```text
large exact Int arithmetic
1000-element Tuple/Record literal within budget
nested static data
long constant dependency chain
many repeated identical subvalues to prove sharing
large String within configured resource policy
```

Then adversarial budget cases just over the limit.

P3 later adds large Pack stress.

---

# 96. Security/resource concerns

Compile-time evaluation is an attacker-controlled computation surface.

P2 must defend against:

```text
huge exponentiation
huge shifts
large structural literal allocation
deep nested expression
dependency cycles
definition call depth
pathological strings
repeated failed static calls
```

Use deterministic semantic limits.

No process-wide mutable cache keyed by untrusted rendered strings.

---

# 97. Threading/concurrency

Semantic DB may execute queries concurrently/editor-latest-wins.

StaticValueStore interning must be compatible with the existing semantic analysis threading model.

Do not introduce an unsynchronized global static-value interner.

Preferred:

```text
session/snapshot-owned store under same synchronization discipline as TypeStore
```

---

# 98. StaticValueStore and TypeStore coupling

StaticValue nodes containing TypeId are only meaningful with the corresponding TypeStore semantic universe.

Snapshot must retain both consistently.

Do not deserialize/copy a StaticValueId into another TypeStore without reification/mapping.

P8 stable metadata solves cross-artifact identity.

---

# 99. Structural values and recursive types

A static value may contain:

```text
Type(Node)
```

where Node is recursive.

That is a reference, not recursive static-value containment.

P2 must preserve the C4 invariant:

```text
reduced StaticValue graph is acyclic
recursive semantic entities are referenced by canonical IDs
```

---

# 100. Static Data cycles

P2 must not create arbitrary self-referential static ordinary data values.

There is no P2 source construct that should make:

```text
value contains itself by identity
```

possible.

If future language semantics introduce recursive constant graphs, that requires a separate design.

---

# 101. Equality/hash recursion safety

Because StaticValue structural containment is acyclic:

```text
structural equality/hash terminates
```

Use iterative or bounded traversal where deep structures could overflow host stack.

Do not assume input nesting is shallow.

---

# 102. Explanation/tooling integration

Static diagnostics should integrate with existing explanation/causal infrastructure where possible.

LSP should be able to show:

```text
const Answer: Int = 42
```

and eventually:

```text
static value: 42
```

but P2 should not require rendering enormous values in hover.

Use summaries/budget caps.

P9 finalizes UX.

---

# 103. P2 open decisions requiring ratification before patch-grade planning

## P2-O1 — materialization breadth

Choose:

```text
M1 semantic descriptor only until P8
or
M2 minimal same-module primitive materialization in P2
```

Recommendation: **M2**.

## P2-O2 — canonical static domain semantic name

Choose one shared model equivalent to:

```text
Kind(KindId)
ValueType(TypeId)
```

This is architectural, not cosmetic.

Recommendation: ratify this distinction before coding parser/generic signatures.

## P2-O3 — static-definition parameter representation

Choose whether to:

```text
introduce StaticDefinitionSignature/StaticParameter
```

(recommended)

or globally generalize GenericSignature.

Recommendation: **separate static-definition signature with compatibility links to TypeParameterId for kinded parameters**.

This minimizes C3 risk.

## P2-O4 — type-definition AST naming

Choose:

```text
rename TypeAliasDef -> TypeDefinitionDef
```

or retain compatibility name in AST.

Recommendation: new semantic name is mandatory; AST rename can be mechanical but preferable during P2 if migration cost is manageable.

## P2-O5 — generic static argument syntax

Need final source representation for ordinary static Bool/String/Symbol arguments.

Int can bridge existing C3 literal argument syntax, but P5 requires Symbol.

Recommendation: establish general static-value argument syntax in P2, even if only a subset is executable initially.

## P2-O6 — destructuring scope

Recommendation:

```text
name-only static const/local binders in P2
general static patterns in P6
```

This preserves one pattern engine.

---

# 104. VERIFY-FIRST items for the P2 patch plan

Before coding, the implementer must verify at the final P1-closed revision:

1. P1 final revision and clean checkpoint state.
2. Whether the P1 compiler static-const sets changed after `c2a938`.
3. Exact TypeAlias declaration query path; it may have evolved during P1 closure.
4. Exact module binding identity/source-index behavior for `TopLevelLetMut` and `TopLevelConst`.
5. Existing alias cycle detection and whether `DeclarationShell` query owns aliases.
6. Whether `TypeMemberParameterDomainSyntax::ConstType` has any active semantic consumer.
7. Current TypeStore snapshot lifetime/reuse strategy to mirror it safely for StaticValueStore.
8. Current numeric-limit semantics and reusable lower-level numeric crate options.
9. Exact current `Int` parser/runtime promotion path for large literals.
10. Exact compiler constant-pool/large-int rooting rules before P2-M2 materializes large Int.
11. Existing return/trailing-block semantics for LexicalBlock.
12. Current module import visibility rules for top-level bindings/forward references.
13. Whether any current source uses old type-result-domain-before-generics grammar.
14. Current metadata tests that assume all TypeAliasInfo are independently authoritative.

Architectural differences require consultation; mechanical drift may be adapted.

---

# 105. Rejected shortcuts

Do not:

1. use the VM as compile-time evaluator;
2. reuse `phalcom_core::Value` as StaticValue;
3. store VM `Symbol(u32)` in StaticValue;
4. make every static value a TypeData;
5. reinterpret `N: Int` as a KindId named Int;
6. globally convert C3 generic type parameters into runtime-value parameters;
7. implement static `+` by AST token alone;
8. execute user/native methods during static evaluation;
9. eagerly reject forbidden calls in untaken branches;
10. use Closure `Expr::Block` as type-definition lexical block;
11. use raw source AST as residual IR;
12. use StaticValueId alone as cross-snapshot/durable identity without store ownership;
13. hash constant products only by type;
14. materialize const by rerunning initializer bytecode;
15. use compiler `HashSet<Symbol>` as static value authority;
16. implement a simplified P2 match/destructurer;
17. implement static while by recognizing `whileTrue`;
18. create a hidden independent static evaluator step limit;
19. collapse cancellation/budget/recursion into Invalid;
20. serialize static arena IDs in metadata.

---

# 106. Recommended implementation staging

A later Luna patch-grade plan should likely stage P2 approximately as follows.

## P2.T0 — final P1 takeover and static source census

- verify P1 COMPLETE state;
- freeze revision;
- census `type` forms/current result-domain grammar;
- census const declarations/uses;
- inspect numeric/static argument usage.

## P2.T1 — shared static domain syntax/semantic model

- `StaticDomainSyntax`;
- `StaticValueDomain::{Kind,ValueType}`;
- result/parameter resolution;
- tests.

## P2.T2 — static-definition AST/grammar

- expression body;
- LexicalBlock body;
- generics-before-result-domain;
- legacy alias compatibility;
- source-index traversal.

## P2.T3 — StaticDefinition signature/product

- mixed static parameters;
- compatibility TypeParameter links;
- StaticDefinition query/product;
- TypeAliasInfo adapter;
- fingerprints.

## P2.T4 — StaticValueStore

- primitive exact values;
- snapshot-safe ownership;
- strings/symbols;
- Type/StaticTerm bridge;
- equality/hash tests;
- exact BigInt/Float semantics.

## P2.T5 — structural static values

- Tuple/Record/data/variant nodes;
- DAG sharing;
- value-domain typing;
- no Pack yet.

## P2.T6 — checked static body and operation registry

- body-owner integration;
- semantic operation resolution;
- core static intrinsic registry;
- forbidden runtime calls.

## P2.T7 — explicit-frame evaluator

- expression evaluation;
- lexical locals;
- assignment;
- if/short-circuit;
- nested non-recursive definition application;
- recursive outcome.

## P2.T8 — top-level/local const query integration

- StaticConstant query;
- dependency graph;
- reference denotation;
- value fingerprint;
- cycles;
- runtime binding rejection.

## P2.T9 — materialization descriptor and semantic lowering projection

- descriptor DAG;
- ModuleLoweringSemantics attachments;
- optional M2 primitive materializer;
- no initializer AST lowering.

## P2.T10 — budgets/cancellation/incremental

- value-node budget;
- numeric resource controls;
- cancellation;
- cold/incremental parity;
- value-only invalidation tests.

## P2.G1 — closed evaluator vertical

Prove:
```text
const
static type definition
ordinary static application
if/local mutation
forbidden runtime path
```

## P2.G2 — C1–C3 compatibility

Prove old type aliases/static domains/type packs still behave.

## P2.G3 — affected crate stabilization

Then completion records/handoff to P3/P4.

Exact task decomposition belongs to the patch-grade plan.

---

# 107. Acceptance matrix

P2 should not be marked implemented merely because `const 40 + 2` evaluates.

## Syntax/AST

- [ ] expression-bodied static definitions parse;
- [ ] block-bodied definitions parse without `=`;
- [ ] block body is LexicalBlock, not Closure;
- [ ] canonical generic/result-domain order works;
- [ ] historical Type aliases still parse;
- [ ] `N: Int` and `T: Type` are semantically distinct domains.

## Static domains

- [ ] one shared `Kind | ValueType` semantic model;
- [ ] C3 type-data domains continue to resolve;
- [ ] ordinary value types are not treated as kinds;
- [ ] mixed parameter definitions work.

## Static values

- [ ] no `phalcom-core` dependency;
- [ ] exact/unbounded Int;
- [ ] correct Float edge semantics;
- [ ] String content semantics;
- [ ] stable Symbol semantics;
- [ ] Type/StaticTerm bridge;
- [ ] structural value DAG;
- [ ] snapshot-safe StaticValueId ownership.

## Static definitions

- [ ] stable DeclarationId authority;
- [ ] checked body product;
- [ ] TypeAliasInfo compatibility derived/coherent;
- [ ] result domain checked;
- [ ] non-recursive application works;
- [ ] recursion fails distinctly.

## Static evaluator

- [ ] explicit frames;
- [ ] locals/assignment;
- [ ] ordinary if;
- [ ] short-circuit;
- [ ] untaken invalid static operation is not executed;
- [ ] semantic operation identity controls static intrinsics;
- [ ] arbitrary user/native calls never execute.

## Constants

- [ ] semantic StaticConstant product keyed by module identity;
- [ ] compiler HashSets are no longer value authority;
- [ ] initializer evaluated semantically;
- [ ] no runtime initializer bytecode;
- [ ] runtime binding dependencies rejected;
- [ ] const-to-const dependencies recorded;
- [ ] cycles terminate;
- [ ] value changes invalidate dependents.

## Outcomes/resources

- [ ] Reduced/Invalid/Recursive/Budget/Cancelled remain distinct;
- [ ] closed Symbolic const rejected;
- [ ] QueryBudget integrated;
- [ ] CancellationToken integrated;
- [ ] large value/numeric resource behavior bounded.

## Materialization seam

- [ ] deterministic descriptor;
- [ ] descriptor independent of VM objects;
- [ ] ModuleLoweringSemantics receives descriptor/spec;
- [ ] initializer AST is never re-evaluated by compiler;
- [ ] P2-M1/M2 scope explicitly ratified.

## Compatibility

- [ ] existing type aliases remain correct;
- [ ] C3 static domains remain correct;
- [ ] C3 TypePack/static-argument semantics remain correct;
- [ ] no P3 Pack ontology prematurely introduced;
- [ ] no P6 pattern/control duplication;
- [ ] no P7 raw residual workaround.

---

# 108. Exit criteria

C4.P2 is complete only when:

1. `const` is actually evaluated in `phalcom-semantic`, not merely blocked from runtime lowering;
2. top-level static constants have canonical semantic identity and query products;
3. constant value contents participate in dependency fingerprints;
4. StaticValue is independent of runtime `Value`, ObjRef, VM symbols, and runtime allocation identity;
5. StaticValueStore IDs are safe under semantic snapshot/query reuse;
6. exact Int and Float semantics are explicitly correct;
7. static String/Symbol semantics are stable/content-based as ratified;
8. expression-bodied and lexical-block-bodied `type` static definitions parse and check;
9. historical Type aliases still function through a coherent compatibility projection;
10. static result domains distinguish kinded terms from ordinary compile-time values;
11. generic static definitions can bind at least the C4-required ordinary value domains such as `Int` and the architecture supports Bool/String/Symbol without replacement;
12. a non-recursive concrete static-definition application evaluates;
13. ordinary `if`, simple local `let`/`let mut`, assignment, and trailing-result blocks evaluate;
14. operation reduction is selected by canonical semantic operation identity, never AST spelling alone;
15. runtime/user/native calls are not executed at compile time;
16. untaken branches are not evaluated;
17. static evaluator uses explicit frames and cannot host-stack recurse indefinitely;
18. recursion, budget exhaustion, cancellation, and invalid computation remain distinct;
19. existing QueryBudget/CancellationToken are the evaluator authority;
20. materialization is expressed as a semantic descriptor and the compiler never reruns initializer AST;
21. static descriptor/specs flow through semantic lowering rather than compiler-side static evaluation;
22. P2 does not implement static match/for/while, Pack, FieldKey, structural reflection, or residual IR as ad hoc shortcuts;
23. focused incremental/cold parity tests pass;
24. C1–C3 alias/static-domain regression gates pass;
25. the P2 walkthrough/handoff clearly freezes the interfaces P3/P4 inherit.

---

# 109. Highest-risk implementation areas

Ranked.

## Risk 1 — generic ordinary-value static parameters

`N: Int` requires architecture beyond current KindId generic parameters.

A bad shortcut here would infect P5 reflection and P7 residualization.

## Risk 2 — StaticValueStore lifetime under incremental reuse

Stale arena IDs can silently produce wrong compile-time values.

This is worse than a visible compiler error.

## Risk 3 — operation identity / arbitrary dispatch

Evaluating by syntax can accidentally make any overloaded method executable at compile time, effectively creating the hidden compile-time VM C4 explicitly rejects.

## Risk 4 — type alias compatibility

The repository has substantial C1–C3 logic built around TypeAliasInfo/TypeId.

P2 should generalize authority without unnecessarily replacing proven type normalization.

## Risk 5 — exact numeric semantics/resource policy

Compile-time BigInt and IEEE Float behavior must match Telos and remain bounded.

## Risk 6 — incremental const value fingerprints

If value content is omitted, cross-definition results become stale even though types remain unchanged.

## Risk 7 — body AST distinction

Using Closure `Expr::Block` for static definition body would embed the wrong language semantics from day one.

---

# 110. Final recommended P2 model

```text
SOURCE
────────────────────────────────────────────

const Answer: Int = 40 + 2

type IsZero<N: Int>: Bool =
    N == 0

type Pair<A: Type, B: Type>: Type =
    (A, B)

type Example<N: Int>: Int {
    let mut x = N
    if x == 0 {
        x = 1
    }
    x
}


DOMAIN MODEL
────────────────────────────────────────────

StaticValueDomain
    ├─ Kind(KindId)
    │    ├─ Type
    │    ├─ TypePack
    │    ├─ RecordRow
    │    └─ type-data domain Shape/...
    │
    └─ ValueType(TypeId)
         ├─ Int
         ├─ Bool
         ├─ String
         ├─ Symbol
         └─ later ProductShape / Variant / ...


DEFINITION MODEL
────────────────────────────────────────────

DeclarationId
    ↓
StaticDefinition
    ├─ StaticDefinitionSignature
    │    └─ StaticParameter[]
    │         ├─ kinded TypeParameter bridge
    │         └─ ordinary static-value parameter
    ├─ result_domain
    ├─ CheckedStaticBody
    ├─ dependencies
    └─ fingerprint

Type-valued C3 aliases
    = compatibility projection of this authority


VALUE MODEL
────────────────────────────────────────────

StaticValueStore
    ├─ Unit
    ├─ Bool
    ├─ exact Int
    ├─ Float representation
    ├─ String content
    ├─ Symbol semantic spelling
    ├─ Type(TypeId)
    ├─ StaticTerm(TypeId)
    ├─ Tuple
    ├─ Record
    ├─ Data
    └─ Variant

No runtime Value / ObjRef / VM Symbol.


EVALUATION
────────────────────────────────────────────

checked semantic expression
    ↓
exact StaticOperationResolution
    ↓
explicit-frame evaluator
    ├─ local slots
    ├─ concrete static params
    ├─ if/short-circuit
    ├─ nested non-recursive definitions
    ├─ QueryBudget
    └─ CancellationToken
    ↓
StaticEvalOutcome


CONSTANT
────────────────────────────────────────────

SymbolId(module, name)
    ↓
QueryKey::StaticConstant
    ↓
StaticConstantProduct
    ├─ StaticValueId
    ├─ domain/type
    ├─ dependencies
    ├─ value-content fingerprint
    └─ MaterializationDescriptor
    ↓
ModuleLoweringSemantics
    ↓
compiler materialization consumer

Initializer is never rerun.


LATER PLANS
────────────────────────────────────────────

P3 adds Pack StaticValue/runtime transport.
P4 adds FieldKey.
P5 adds Product/Sum reflection values.
P6 adds match/for/while/proof predicates.
P7 adds recursive evaluation/memo/residual IR.
P8 freezes durable metadata and general materialization.
```

This architecture keeps the C4 promise intact:

> **Static Telos remains ordinary Telos semantic meaning executed by a bounded semantic evaluator, not a second runtime.**
