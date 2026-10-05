---
id: LANG007.C4.structural-static-upgrade-handoff
category: LANG
program: LANG007
checkpoint: LANG007.C4
kind: handoff
status: PROPOSED
completion: DESIGN_RATIFIED_IMPLEMENTATION_NOT_STARTED
verification: REPOSITORY_AUDITED_NOT_IMPLEMENTED
title: "LANG007.C4 — Structural Static Evaluation, Packs, Products, Sums, and Binding Semantics"
repository: aureat/phalcom-lang
branch: main
prepared_against_revision: 8c3a98aab41ed8631e9f8746a6b29984cf77581d
prepared: 2026-10-03
---

# LANG007.C4 — Structural Static Evaluation, Packs, Products, Sums, and Binding Semantics

## Handoff for continued analysis and complete implementation

> **Historical note (2026-10-05):** This handoff document was prepared on 2026-10-03
> against `8c3a98aab41ed8631e9f8746a6b29984cf77581d` to launch C4 implementation.
> Implementation has since progressed through completed plans P1, P2, and P3, with
> P4 active. Current lifecycle truth is maintained in `LANG007.C4-CHECKPOINT.md`.
>
> **Repository:** `aureat/phalcom-lang`  
> **Branch:** `main`  
> **Prepared against:** `8c3a98aab41ed8631e9f8746a6b29984cf77581d`  
> **Checkpoint:** `LANG007.C4`  
> **Status:** historical handoff launching C4 implementation; current state is in `LANG007.C4-CHECKPOINT.md`

---

# 0. Executive conclusion

C4 should not be approached as a collection of isolated syntax features. The ratified design creates one coherent structural/static subsystem:

```text
runtime bindings
    let
    let mut

compile-time value bindings
    const

source-defined static computation
    type F<...> = expression
    type F<...> { statements; trailing_result }

static evaluator
    ordinary Telos semantic expressions
    deterministic + effect-free
    match / for / while / local mutation / recursion
    no compile-time VM
    no arbitrary runtime method execution

structural values and types
    Pack
    Product
        OrderedProduct
        UnorderedProduct
    Sum
        Variants
            Variant(name, ProductShape)

reflection
    ordinary immutable Telos data
    source-defined algorithms wherever possible
```

The current repository is unusually well-positioned for this upgrade. C3 already established canonical static domains, heterogeneous static arguments, metadata schema v10, and incremental semantic products. The type system already has canonical `TypeStore` interning, record-row interning, explicit query budgets and cancellation, symbolic relation outcomes, and mature GADT/exact-case infrastructure. Runtime product representation already converged Tuple, Record, and nominal data onto shared product storage/layout mechanisms. Variant families already represent overloaded variants sharing a base symbol. Match analysis already owns variant candidate resolution, pattern projection, exhaustiveness/usefulness, and GADT branch proofs.

C4 should **reuse those authorities rather than create parallel mechanisms**.

The highest-risk implementation areas are:

1. changing `const` from “runtime immutable binding” to “statically evaluated constant binding” while introducing `let mut` and making plain `let` runtime immutable;
2. giving parameterized `type` declarations executable expression/block bodies without turning the compiler into a second VM;
3. making `Pack` a real immutable first-class product while preserving the current no-allocation fast path for ordinary calls;
4. replacing public Record-specific row terminology with generalized unordered-product reflection while migrating the compiler from string-only row keys to `FieldKey = Symbol | String | Int | Bool`;
5. deriving `Product`, `OrderedProduct`, `UnorderedProduct`, and `Sum` reflection from ordinary `data` declarations without allowing user code to forge compiler semantic facts;
6. reusing existing match/GADT/variant-family authorities for static matching and overloaded Sum variants;
7. designing static recursion, memoization, residualization, budgets, and incremental dependencies so generic type computation is predictable, bounded, and fast;
8. preserving cold/incremental parity and stable metadata identity for exported constants and static reflection products.

The next agent should resist two tempting shortcuts:

- **Do not execute bytecode or ordinary methods at compile time.**
- **Do not redesign the public source ontology around the compiler’s current `TypePack` / `RecordRow` implementation types.**

C4 is specifically the checkpoint where those internal representations become implementation details beneath the more coherent public Pack/Product/Sum model.

---

# 1. Authority and status of this handoff

## 1.1 Ratified language semantics

These were explicitly selected during the C4 design discussion and should be treated as the source-language target unless the user reopens them.

- `let` is a runtime immutable binding.
- `let mut` is a runtime mutable binding.
- `const` is a compile-time constant binding.
- Class fields are immutable by default; `mut` marks a mutable field.
- Static evaluation is a sibling evaluation strategy for Telos semantic expressions, not compile-time runtime dispatch.
- Parameterized `type` declarations are C4’s reusable static-computation abstraction.
- **No const functions or const methods in C4.**
- Expression-bodied `type` declarations use `=`.
- Block-bodied `type` declarations omit `=`.
- Static `match`, `for`, and `while` are ordinary Telos control forms evaluated statically.
- Static blocks support local `let`, `let mut`, assignment, `return`, `break`, and `continue`.
- Pack is a first-class immutable ordered two-lane product.
- Pack views use the lane-specific Candidate A design.
- Public product capabilities are `Product`, `OrderedProduct`, `UnorderedProduct`.
- Ordered-product constituents are **entries**; labeled entries have **labels**.
- Unordered-product constituents are **fields**; fields have **keys**.
- General field keys are `Symbol | String | Int | Bool`.
- Product reflection uses `ProductShape::{Ordered, Unordered}`.
- Enumerated data uses terminology `Sum`, `Variant`, and `Variants`.
- A variant Symbol may name multiple structural variants.
- `Variants` is an ordinary `PositionalPack<Variant>`.
- Reflection should be ordinary source-visible static data where possible.
- Product values and product types support structural static iteration according to their shape.
- Open tails describe possible structure and are not finite iterable sequences.

## 1.2 Implementation details still open

Several decisions remain implementation- or edge-semantics-level and are listed explicitly in Part 17. The next agent should investigate and propose them before coding the affected layer. Do not silently turn a plausible implementation choice into a new language rule.

---

# 2. Repository baseline and exact C4 delta

The following findings were inspected directly against HEAD `8c3a98a`.

## 2.1 Algebraic data is already unified

`phalcom-ast/src/ast.rs` already converges ordinary product and sum declarations on `AlgebraicDef`, with `AlgebraicShapeSyntax::{Product, Sum, IntrinsicFacade}` and product payload forms split into Tuple/Record shape syntax.

This is a strong foundation. C4 should not reintroduce separate declaration pipelines for products and sums. Public terminology may say Sum/Variant while existing internal `SumShapeSyntax`, `AlgebraicCaseSyntax`, `EnumInfo`, etc. can be migrated incrementally where renaming has no semantic benefit.

## 2.2 C3 static-domain infrastructure is mature

The closed C3 handoff reports and the live tree confirms the important seams:

```text
KindData::StaticDomain
TypeData::ConstInt
TypeData::StaticData
StaticArgument
associated static-domain identity
metadata schema v10
```

`phalcom-type-meta` schema v10 contains real `StaticDomainRef`, `StaticDataRef`, and heterogeneous `StaticArgumentRef`.

Do not create a second static-domain registry for C4.

## 2.3 Type storage still exposes the old public ontology internally

`phalcom-semantic/src/types/store.rs` currently has specialized nodes including:

```rust
TypeData::Tuple(...)
TypeData::Record(...)

TypeData::TypePack(...)
TypeData::LabeledTypePack(...)
TypeData::CompleteTypePack { ... }
TypeData::TypePackConcat(...)
TypeData::LabeledTypePackConcat(...)
TypeData::TupleFromPacks { ... }
TypeData::CallableFromPacks { ... }
```

and `KindData` retains separate `TypePack`, `LabeledTypePack`, and `CompleteTypePack` kinds.

C4 does **not** require immediately deleting these optimized internal nodes. It does require that they stop defining the public source ontology.

Public semantics are:

```text
PositionalTypePack = PositionalPack<Type>
LabeledTypePack    = LabeledPack<Type>
CompleteTypePack   = CompletePack<Type>
```

Internal specialized nodes may remain as compact canonical representations so long as source equality, reflection, substitution, metadata, and specialization implement the Pack-based model.

## 2.4 Record rows are currently string-keyed

`phalcom-semantic/src/types/row.rs` currently uses fields approximately shaped as:

```rust
RecordRowField {
    name: Box<str>,
    ty: TypeId,
    presence: StructuralMemberPresence,
}
```

with canonical sorting and binary search by name.

C4 requires:

```text
FieldKey = Symbol | String | Int | Bool
```

This is a real compiler-wide migration. A canonical field-key representation will be needed in semantic rows, metadata, structural relations, patterns, fingerprints, reflection, and lowering.

## 2.5 Optional-field semantics already exist

Current rows encode:

```rust
StructuralMemberPresence::{Required, Optional}
```

and the recent repository work fixed the optional-source → required-target subtyping error. Preserve that authority. C4 generalizes keys/reflection; it should not reopen optionality semantics or reintroduce optional tuple entries.

## 2.6 Current binding semantics are opposite the C4 target

Current source/compiler semantics are:

```text
let   = mutable runtime binding
const = immutable runtime binding
```

This assumption is encoded across:

- `phalcom-ast`;
- `phalcom-semantic/src/checker/statement.rs`;
- `phalcom-semantic/src/source_index/builder.rs`;
- `phalcom-core/src/compiler/lib/mod.rs`;
- `phalcom-core/src/compiler/lib/product_opt.rs`;
- compiler assignment diagnostics;
- module interface/linking;
- LSP/editor tooling.

The compiler currently has logic equivalent to:

```rust
let mutable = matches!(binding.kind, BindingKind::Let);
```

Current `const` is not a static-evaluation boundary.

Therefore the C4 binding change is semantic and cross-cutting, not a lexer rename.

## 2.7 Current field syntax is also opposite the final target

The current implementation is roughly:

```text
bare _field   => mutable
const _field  => immutable
```

`FieldDef.mutable` already exists, and compiler lifecycle checks know how to enforce immutable fields.

C4 target:

```text
_field      => immutable
mut _field  => mutable
```

There is currently no `Token::Mut`.

The storage/lifecycle foundation is useful; C4 primarily changes grammar/defaults and must ensure every later consumer reads `FieldDef.mutable` rather than re-inferring mutability from spelling.

## 2.8 Match/GADT analysis should be reused

`phalcom-semantic/src/match_semantics.rs` already owns:

- `MatchResolution`;
- `MatchArmResolution`;
- `PatternResolution`;
- `ResolvedVariantPattern`;
- `ResolvedVariantCandidate`;
- `BranchProofEnvironment`;
- usefulness/exhaustiveness;
- exact candidate identities.

A static evaluator must not build a second variant resolver or second GADT proof engine. The semantic checker should resolve/refine patterns once; static evaluation should consume those semantic products.

## 2.9 Variant overload families already match the new Sum semantics

`phalcom-semantic/src/identity.rs` already has:

```rust
VariantId
VariantFamilyId
```

where `VariantFamilyId` explicitly groups overloaded variants sharing a base name.

`EnumInfo` / `VariantInfo` retain exact variants, payload shape, constructor signatures, exact-case templates, and case environments.

This maps directly onto the public rule:

```text
#Name
    -> Variant(shape A)
    -> Variant(shape B)
    -> ...
```

`VariantsOf<T, #Name>` should be driven from these canonical variant products, never by source-name matching.

## 2.10 Query budgets and cancellation already exist

`phalcom-semantic/src/types/outcome.rs` provides a reusable `QueryBudget` with defaults including:

```text
steps:          10_000
relation pairs: 1_000
SCC iterations: 100
type depth:     128
diagnostic notes: 32
```

plus `BudgetReport` and an epoch-aware cooperative `CancellationToken`.

C4 static evaluation should integrate with this authority instead of introducing hidden recursion counters or unbounded Rust recursion.

## 2.11 Incremental semantic DB already has stable query products

`phalcom-semantic/src/db/key.rs` / `db/query.rs` provide typed query keys, product fingerprints, dependency recording, reuse validation, blocked/failed states, and cold/incremental reuse.

C4 should add proper static-definition/constant query products rather than using a side cache disconnected from semantic dependencies.

## 2.12 Runtime product convergence already exists

`phalcom-core` already has shared product shape/layout/storage registries, product scalar replacement, static Tuple/Record construction, and nominal data/variant payload integration.

First-class Pack should reuse this substrate rather than creating a completely independent collection representation.

## 2.13 Argument-pack building already exists privately

`phalcom-core/src/heap/pack_builder.rs` has:

```rust
ArgumentPackBuilderObject {
    positionals: Vec<Value>,
    labels: Vec<Symbol>,
    labeled_values: Vec<Value>,
    pending_label: Option<usize>,
}
```

with duplicate-label checks.

Current dynamic outgoing calls already use Pack assembly bytecodes. This builder is deliberately VM/compiler-private and should remain so.

Current rest capture still finalizes through `finish_tuple`. C4 must change rest capture to immutable Pack without forcing ordinary fixed calls through materialized Pack allocation.

## 2.14 Type-member result domains need generalization

Current `phalcom-semantic/src/types/type_member.rs` has:

```rust
pub enum TypeMemberDomain {
    Kind(KindId),
}
```

C4 reflection members may return static values/data domains rather than only proper type/kind results. This is a key integration seam.

## 2.15 Metadata baseline is schema v10

`phalcom-type-meta` schema v10 already preserves static domains, structural presence, anonymous remainders, and heterogeneous static arguments.

C4 likely requires a schema revision for generalized `FieldKey`, exported ordinary constants, first-class Pack semantics, and any new static residual/result forms that cross module boundaries.

---

# 3. Ratified binding semantics

Canonical source:

```telos
let x = runtimeExpression

let mut y = runtimeExpression

const z = staticExpression
```

| Form | Phase | Reassignable | Static requirement |
|---|---|---:|---|
| `let` | runtime | no | none |
| `let mut` | runtime | yes | none |
| `const` | compile time | no | initializer must statically reduce |

Staticness is declared, never inferred from optimizer knowledge.

```telos
let x = 3
const y = x + 1
```

is invalid because `x` is runtime.

A `const` referenced by runtime code is materialized from its compile-time result. The initializer is not rerun at runtime.

## 3.1 Class fields

Target:

```telos
class Counter {
    _name: String
    mut _count: Int
}
```

`FieldDef.mutable` should remain the semantic bit.

The next agent must audit constructor lifecycle, `@construct`, read-before-write, delegation setters, inheritance, implicit fields, and class-side fields before finalizing the exact one-time initialization rule for immutable fields.

---

# 4. Binding and field corpus migration

The user explicitly wants a fast regex-oriented replacement of source code. Preserve behavior first.

Current meaning:

```text
old let   = mutable runtime
old const = immutable runtime
```

Behavior-preserving new spelling:

```text
old let   -> let mut
old const -> let
```

A safe staged/token-aware rewrite is:

```text
old const binding token -> temporary sentinel
old let binding token   -> let mut
sentinel                -> let
```

Do not perform naive sequential global replacements.

For fields:

```text
old const _field        -> _field
old bare mutable _field -> mut _field
```

This is context-sensitive. A lexer/token scanner or parser-assisted codemod is strongly preferred even if actual replacement is regex/string based.

### Rust-side migration

Recommended AST:

```rust
enum BindingKind {
    Let,
    LetMut,
    Const,
}
```

but avoid making every semantic consumer treat these as mere mutability variants. Distinguish:

```text
evaluation phase:
    Runtime
    Static

runtime mutability:
    Immutable
    Mutable
```

`const` is primarily a phase boundary.

Audit all occurrences of:

```text
BindingKind
BindingKind::Let
BindingKind::Const
is_mutable
ConstWithoutInitializer
AssignToImmutable
TopLevelBindingKind
SourceBindingKind
DeclarationSurface.is_const
```

across AST, semantic checker, source index, modules, compiler, optimizer, LSP, fixtures, and generated source.

Compiler-generated scratch locals must not accidentally become `const`.

---

# 5. Static evaluation

## 5.1 Core law

> **Static evaluation is an alternate evaluation strategy for a defined subset of Telos semantic expressions. It is not compile-time message dispatch.**

For `+`, the evaluator executes Telos addition semantics directly. It does not boot the runtime, look up a `+` method, invoke it, and capture the result.

Runtime lowering remains independent.

## 5.2 Static contexts

```telos
const x = expression
```

and:

```telos
type Name<...>: Domain =
    expression
```

or:

```telos
type Name<...>: Domain {
    statements
    trailingResult
}
```

No `=` precedes a block body.

## 5.3 No const functions

C4 explicitly does not implement const functions/methods.

Parameterized `type` declarations are the reusable source-defined static abstraction.

## 5.4 Allowed semantic forms

Subject to static eligibility:

- literals;
- static references;
- Type/static-domain values;
- Tuple/Pack/Record construction;
- transparent data construction;
- statically resolved Sum construction;
- product expansion/composition;
- projection;
- static indexing/lookup;
- arithmetic/comparison/boolean operators;
- semantic equality;
- `if`;
- `match`;
- `for`;
- `while`;
- local `let`;
- local `let mut`;
- assignment to local evaluator state;
- `return`;
- `break`;
- `continue`;
- destructuring;
- structural reflection;
- parameterized/recursive `type` application;
- compiler relation/proof predicates.

## 5.5 Forbidden forms

No:

- arbitrary runtime methods/functions;
- arbitrary native calls;
- general runtime dynamic dispatch;
- I/O;
- filesystem/network/process access;
- clock/randomness;
- runtime mutable state;
- runtime heap identity;
- fiber/thread/scheduler observation.

Static interpolation must use deterministic language-defined rendering, not arbitrary user `toString`.

## 5.6 Identity

Compile-time values have semantic identity, not VM object identity.

Exact identity operators are statically valid only where runtime allocation identity cannot affect the result.

---

# 6. Static evaluator architecture

Implement in `phalcom-semantic`, not the VM.

Suggested logical modules:

```text
static_eval/
    value
    context
    frame
    eval
    control
    pattern
    reflect
    normalize
    memo
    materialize
    diagnostic
```

## 6.1 Ordinary const values should not all become `TypeData`

C4 constants include ordinary values:

```text
Bool
Int
Float
String
Symbol
Type
Pack
Tuple
Record
transparent data
Sum values
```

Use a semantic `StaticValue` / `StaticValueId` arena, conceptually:

```rust
enum StaticValue {
    Unit,
    Bool(bool),
    Int(...),
    Float(...),
    String(StaticStringId),
    Symbol(StableSymbol),
    Type(TypeId),
    StaticTerm(TypeId),
    Pack(StaticPackId),
    Tuple(StaticProductId),
    Record(StaticProductId),
    Data(StaticDataValueId),
    Variant(StaticVariantValueId),
}
```

Requirements:

- no `phalcom-core::Value` dependency;
- no `ObjRef`;
- deterministic equality/hash;
- stable export encoding;
- compact sharing of structural values;
- direct bridge to `TypeId` for Type/static-domain terms.

## 6.2 Use an explicit evaluation machine

User recursion and loops must not become unbounded host-stack recursion.

Use explicit frames/work stack and explicit control transfer:

```rust
enum EvalControl {
    Value(StaticValueId),
    Return(StaticValueId),
    Break,
    Continue,
}
```

This centralizes budgets, cancellation, stack traces, and loop control.

## 6.3 Evaluator-local mutation

Static `let mut` uses mutable evaluator slots. Do not create observable mutable static objects and do not intern every intermediate assignment.

Final immutable results may be canonicalized/interned.

## 6.4 Never fall back to runtime dispatch

If current parser lowering has erased a semantic construct into a method-send shape, preserve/reintroduce enough semantic AST to evaluate it directly.

This is particularly important for legacy `while` desugaring.

---

# 7. `type` definition parsing and semantic model

Current `TypeAliasDef` has:

```rust
result_kind: Option<KindSyntax>
body: TypeAnnotation
```

This is insufficient.

C4 needs conceptually:

```rust
enum StaticDefinitionBody {
    Expression(...),
    Block(...),
}
```

and a generalized result domain.

Examples:

```telos
type Optional<T> =
    T | None
```

```telos
type IsSubtype<A: Type, B: Type>: Bool =
    Subtype<A, B>
```

```telos
type Count<*P>: Int {
    let mut count = 0
    for _ in P {
        count += 1
    }
    count
}
```

A result annotation such as `: Int` is not merely a kind. Normalize type-definition result requirements through a general static-domain abstraction, likely something conceptually like:

```rust
enum StaticResultDomain {
    Kind(KindId),
    Const(TypeId),
}
```

or reuse the existing generalized C3 generic/static-domain machinery.

Default result domain remains `Type`.

Do not create a permanently separate “static expression language” if ordinary Telos `Expr`/`Statement` can serve both phases.

---

# 8. Evaluation outcomes, recursion, budget, cancellation

Publish distinct outcomes:

```rust
enum StaticEvalOutcome {
    Reduced(StaticValueId),
    Symbolic(StaticResidualId),
    Invalid(StaticEvalError),
    Recursive(StaticCycle),
    BudgetExceeded(BudgetReport),
    Cancelled,
}
```

A closed `const` must finish `Reduced`.

Generic `type` computation may remain `Symbolic`.

## 8.1 Reuse QueryBudget

Use existing `QueryBudget` and `CancellationToken`.

Charge at least:

- expression evaluation;
- type application;
- loop iteration;
- nontrivial pattern candidate attempt;
- structural construction proportional to work.

Add static-value growth budgets only if needed.

## 8.2 Recursion key

Memoize type computation by:

```text
stable definition identity
+
canonical static argument identities
+
relevant specialization environment
```

Exact in-progress repetition means recursion/cycle.

Changing arguments may continue until a base case or budget.

## 8.3 Avoid caching context-sensitive terminal states

Cache deterministic semantic results. Be cautious about caching `Cancelled` and `BudgetExceeded` because those depend on execution context.

## 8.4 Diagnostics

Recursion/budget diagnostics should include:

- static definition;
- source call chain;
- summarized static arguments;
- budget kind/limit/used;
- cycle start.

Use the existing diagnostic note cap.

---

# 9. Static match and structural patterns

Static matching must consume the existing canonical pattern resolution/GADT proof products.

Desired architecture:

```text
source Pattern
    ↓
semantic PatternResolution
    ↓
runtime lowering OR static evaluation
```

When a static scrutinee selects an arm:

- bind static locals;
- install branch proof/equalities;
- evaluate only the selected arm;
- do not evaluate untaken branches.

For unresolved generic scrutinees, retain a canonical residual and normalize after specialization.

Direct type matching should mirror runtime nominal destructuring where possible:

```telos
match value {
    TupleOfCallables(first, *rest) => ...
}
```

```telos
match T {
    TupleOfCallables<First, *Rest> => ...
}
```

`T::Shape` remains the declaration-independent reflection route.

---

# 10. Static `for` / `while`

There is no separate `static for`.

## Ordered product value

```telos
for value in product {
    ...
}
```

iterates positional values.

```telos
for [label]: value in product {
    ...
}
```

iterates labeled entries.

## Ordered product type

For `T: OrderedProduct`:

```telos
for EntryType in T {
    ...
}
```

iterates fixed positional Type entries.

```telos
for [label]: EntryType in T {
    ...
}
```

iterates fixed labeled Type entries.

## Unordered product value

```telos
for [key]: value in recordLikeValue {
    ...
}
```

iterates actual fields. Static evaluation uses canonical deterministic key order.

## Unordered product type

```telos
for [key]: FieldInfo(FieldType, presence) in T {
    ...
}
```

iterates fixed fields in canonical `FieldKey` reflection order.

## Variants

Because `Variants = PositionalPack<Variant>`:

```telos
for Variant(name, shape) in T::Variants {
    ...
}
```

uses ordinary positional Pack iteration.

Repeated names remain distinct entries.

## Tails

`Typed` and `Unconstrained` tails are not iterable; they describe possible structure.

Captured concrete remainder Packs/shapes may be explicitly iterated.

## Refutable loop patterns

A loop pattern binds/destructures; it does not silently filter. Use explicit `match` for refutable conditions.

## `while`

Static `while` is legal and budgeted.

Audit parser desugaring before implementation; do not execute runtime `.whileTrue` semantics statically.

---

# 11. Pack and ordered-product source model

```telos
intrinsic data Pack<***Es>(***Es)

intrinsic data Tuple<***Es>(***Es)

intrinsic data Record<**Fs> {
    **Fs
}

type PositionalPack<T: Type> =
    Pack<*: T>

type LabeledPack<T: Type> =
    Pack<**: T>

type CompletePack<T: Type> =
    Pack<***: T>

type PositionalTypePack =
    PositionalPack<Type>

type LabeledTypePack =
    LabeledPack<Type>

type CompleteTypePack =
    CompletePack<Type>
```

Pack is immutable, first-class, ordered within each lane, lane-sensitive, label-sensitive, and has transparent value semantics.

`Pack()` is distinct from `()` and `{}`.

Pack labels are Symbols.

Duplicate labels are invalid.

## 11.1 Selected Pack-view design

Only Candidate A:

```telos
intrinsic data PositionalPackView<T> {
    Empty

    Entry(
        _ value: T,
        _ rest: PositionalPack<T>,
    )
}

intrinsic data LabeledPackView<T> {
    Empty

    Entry(
        _ label: Symbol,
        _ value: T,
        _ rest: LabeledPack<T>,
    )
}

type PositionalTypePackView =
    PositionalPackView<Type>

type LabeledTypePackView =
    LabeledPackView<Type>
```

Reflection:

```telos
impl<T: Type> PositionalPack<T> {
    intrinsic type View:
        PositionalPackView<T>
}

impl<T: Type> LabeledPack<T> {
    intrinsic type View:
        LabeledPackView<T>
}
```

Implement views as flat-storage slices where possible.

## 11.2 Ordered tails

```telos
intrinsic data OrderedTail<Rest> {
    Closed

    Typed(
        _ valueType: Type,
    )

    Unconstrained

    Captured(
        _ rest: Rest,
    )
}

type PositionalTail =
    OrderedTail<PositionalTypePack>

type LabeledTail =
    OrderedTail<LabeledTypePack>

intrinsic data OrderedShape(
    _ entries: CompleteTypePack,
    _ positionalTail: PositionalTail,
    _ labeledTail: LabeledTail,
)
```

---

# 12. First-class Pack runtime requirements

## 12.1 Preserve fixed-call fast path

An invocation semantically has an argument Pack. Runtime implementation does not need to materialize it.

Fixed call:

```text
stack/register argument window
-> direct/specialized invoke
-> no Pack allocation
```

## 12.2 Reuse private builder

Keep `ArgumentPackBuilderObject` private.

Add immutable Pack finalization.

## 12.3 Rest capture target

C4 rest semantics:

```text
*rest   -> positional Pack
**rest  -> labeled Pack
***rest -> complete Pack
empty   -> Pack()
```

Audit every `finish_tuple` used for rest capture/argument forwarding.

Record should not implicitly become a labeled Pack for invocation expansion. Explicit conversion is required. Record composition remains independent.

## 12.4 Runtime representation

Prefer reuse of product layout/storage.

Requirements:

- immutable;
- compact label storage;
- O(1) lane lengths;
- no duplicate labels;
- semantic equality/hash preserve lane and order;
- GC trace map from existing layout machinery.

## 12.5 Static Pack representation

Use canonical flat semantic storage and slice/range views, not VM objects and not recursive cons allocation.

---

# 13. Product and unordered-product reflection

Canonical terminology:

```text
ordered product
    entries
        positional entries
        labeled entries
            label

unordered product
    fields
        key
        presence
```

## 13.1 Product reflection

```telos
intrinsic data ProductKind {
    Ordered
    Unordered
}

intrinsic data ProductShape {
    Ordered(
        _ shape: OrderedShape,
    )

    Unordered(
        _ shape: UnorderedShape,
    )
}

trait Product {
    type Kind: ProductKind
    type Shape: ProductShape
}

trait OrderedProduct {
    type OrderedShape: OrderedShape
    type Entries: CompleteTypePack
    type PositionalEntries: PositionalTypePack
    type LabeledEntries: LabeledTypePack
}

trait UnorderedProduct {
    type UnorderedShape: UnorderedShape
}
```

Traits are independent; no inheritance.

Compiler derives:

```text
ordered product data
    Product
    OrderedProduct

unordered product data
    Product
    UnorderedProduct
```

User-defined data participates automatically.

Do not allow ordinary user conformance to forge these structural capabilities.

## 13.2 Unordered source

```telos
type FieldKey =
    Symbol | String | Int | Bool

intrinsic data Presence {
    Required
    Optional
}

intrinsic data FieldInfo(
    _ type: Type,
    _ presence: Presence,
)

intrinsic data ProductField(
    _ key: FieldKey,
    _ type: Type,
    _ presence: Presence,
)

intrinsic data FieldLookup {
    Required(_ type: Type)
    Optional(_ type: Type)
    Absent
    Unknown
}

intrinsic data UnorderedTail {
    Closed

    Typed(
        _ keyType: Type,
        _ valueType: Type,
    )

    Unconstrained(
        _ keyType: Type,
    )

    Captured(
        _ rest: UnorderedShape,
    )
}

type FixedFields =
    Record<**[FieldKey]: FieldInfo>

intrinsic data UnorderedShape(
    _ fields: FixedFields,
    _ tail: UnorderedTail,
)

intrinsic data UnorderedShapeView {
    Empty(
        _ tail: UnorderedTail,
    )

    Field(
        _ field: ProductField,
        _ rest: UnorderedShape,
    )
}

impl UnorderedShape {
    intrinsic type View:
        UnorderedShapeView

    intrinsic type Tail:
        UnorderedTail

    intrinsic type Lookup<K: FieldKey>:
        FieldLookup
}
```

## 13.3 FieldKey representation

Compiler needs a stable canonical key representation with a language-defined total order.

Do not order by process-local Symbol IDs or accidental Rust enum discriminants.

The exact cross-kind order remains an explicit open decision.

## 13.4 Typed tails and computed keys

`{ **[String]: Handler }` means additional String keys map to Handler.

Static computed FieldKey -> fixed field.

Runtime key expression known only by key domain -> conservative/open tail.

Collision after specialization -> diagnostic, never overwrite.

---

# 14. Sum and Variant reflection

Core source:

```telos
intrinsic data Variant(
    _ name: Symbol,
    _ shape: ProductShape,
)

type Variants =
    PositionalPack<Variant>

type VariantsView =
    PositionalPackView<Variant>

type VariantNames =
    PositionalPack<Symbol>

trait Sum {
    type Variants: Variants
}
```

There is no `SumShape`: `Variants` is the complete sum reflection.

A Symbol is a family name, not a unique variant.

Build reflection from existing `EnumInfo` / `VariantInfo` / `VariantFamilyId`.

Source-defined selection:

```telos
type VariantsNamed<
    Vs: Variants,
    Name: Symbol,
>: Variants {
    let mut result = Pack()

    for variant in Vs {
        match variant {
            Variant(name, _) =>
                if name == Name {
                    result = Pack(
                        *result,
                        variant,
                    )
                }
        }
    }

    result
}

type VariantsOf<
    T: Sum,
    Name: Symbol,
>: Variants =
    VariantsNamed<T::Variants, Name>
```

Zero/one/many results are represented naturally by a Pack.

Higher utilities such as `VariantCount`, `HasVariant`, `VariantNamesOf`, mapping/filtering, etc. should usually be Telos source.

Do not add `VariantLookup::Found/Absent`.

---

# 15. One canonical structural matching authority

C4 should converge these consumers on shared structural decomposition/relation primitives:

```text
product subtyping
argument acceptance
typed overload resolution
Sum variant-family selection
static structural matching
GADT branch refinement
rest capture
proof predicates
```

Architectural shape:

```text
canonical shape decomposition
    ↓
constraint / pattern matching
    ↓
proof/refutation/symbolic result + captures
    ↓
consumer-specific policy
```

Do not let each subsystem reimplement optionality, tails, label/key matching, or variant-family candidate selection.

---

# 16. Incrementality, metadata, lowering, diagnostics, performance

## 16.1 Query integration

Suggested query products:

```rust
QueryKey::StaticDefinition(DeclarationId)
QueryKey::ConstantBinding(StableBindingId)
```

Application-level memoization can remain evaluator-local unless project-wide query reuse demonstrates value.

Record dependencies on constants, type definitions, reflection declarations, relations, trait facts, and imported metadata.

Changing an exported constant value must invalidate dependents even if its ordinary value type does not change.

## 16.2 Metadata

C4 probably needs schema v11+ for:

- generalized FieldKey;
- exported ordinary static constants;
- first-class Pack semantics where exported;
- generalized reflection result domains;
- static residuals if they cross module boundaries.

Stable metadata must never serialize `TypeId`, VM `Symbol(u32)`, `ObjRef`, or arena offsets as identity.

## 16.3 Compiler lowering

By runtime codegen:

- constants are fully evaluated;
- required type computations are normalized;
- pattern/reflection decisions are frozen;
- compiler lowering does not invoke the static evaluator opportunistically.

Introduce a semantic constant-materialization descriptor and let compiler/VM choose efficient physical construction.

## 16.4 Diagnostics

Recommended diagnostic families:

```text
binding.immutable_assignment
binding.initializer_required
field.mutable_required

const.non_static_dependency
const.static_evaluation_failed
const.symbolic_result
const.result_not_materializable

static.invalid_operation
static.runtime_call_not_allowed
static.recursive
static.budget_exceeded
static.result_domain_mismatch

type.static_body_invalid
type.result_domain_mismatch

reflection.member_not_applicable

pack.duplicate_label
pack.invalid_expansion

field.invalid_key
field.duplicate_key_after_specialization

variant.ambiguous_shape
variant.no_matching_shape
```

Cancellation normally restarts/suppresses editor query output rather than becoming a user error.

## 16.5 Performance requirements

- flat Pack storage;
- O(1) views via slices;
- canonical sorted unordered fields;
- binary search/hash lookup;
- evaluator builder fusion for repeated immutable append patterns;
- memoized type computation;
- no TypeStore/lambda-arena cloning per static call;
- value-size budgets;
- reflection fusion (`View -> match`, `Variants -> for`);
- no Pack allocation for ordinary fixed calls;
- retain existing product scalar replacement.

---

# 17. Explicitly open decisions

These must be resolved deliberately.

## O1 — initializer rules for `let` / `let mut`

Current `let` may omit initializer and reads `None`. C4 has not finally ratified whether new plain `let` or `let mut` may omit an initializer.

Likely direction: immutable `let` requires initializer; `let mut` may preserve no-initializer behavior. Confirm before coding.

## O2 — immutable field construction rule

Decide precise one-time/construction assignment semantics using existing lifecycle analysis.

## O3 — implicit field declaration by assignment

Current language historically permits implicit fields. Decide how that interacts with immutable-by-default fields and `mut`.

## O4 — canonical FieldKey total order

Must be version-stable before metadata/fingerprints.

## O5 — heterogeneous product-value loop binder typing

For:

```telos
for value in (1, "x") { ... }
```

decide union/common-supertype/per-iteration refinement/homogeneity requirements.

Likewise preserve key/value correlation for heterogeneous unordered values.

## O6 — static String exact identity/materialization

Specify static equality, `===`, interning, interpolation, metadata.

## O7 — canonical symbolic residual IR

Avoid serializing raw source AST if possible. Define normalized semantic residuals.

## O8 — exact structural type-pattern grammar

Semantics are clear; parser spellings for open/captured tails and type patterns need finalization.

## O9 — proof-predicate public surface

Finalize `Equivalent`, `Subtype`, `Conforms` signatures and symbolic behavior.

## O10 — GADT refinement reflection

Decide whether public `Variant` needs refinement metadata in C4 or whether branch proof remains match-only.

## O11 — compiler-derived Product/Sum capability mechanism

Choose an implementation that ordinary user impls cannot forge.

## O12 — ordinary `while` semantic AST

Audit current desugaring and preserve semantic identity for static evaluation.

---

# 18. Recommended staged C4 implementation

## P1 — binding transition + static evaluator substrate

Deliver:

- `mut` token/parser;
- `BindingKind::{Let,LetMut,Const}`;
- runtime mutability transition;
- field syntax transition;
- behavior-preserving source codemod;
- `StaticValue`;
- evaluator outcomes;
- budgets/cancellation;
- primitive static expressions;
- expression/block `type` bodies;
- no const functions.

## P2 — first-class Pack + argument/rest convergence

Deliver:

- Pack source/universe declaration;
- runtime/static Pack representations;
- views;
- Pack type relations;
- rest capture Pack migration;
- dynamic Pack finalization;
- no-allocation fixed-call certification.

## P3 — generalized unordered-product calculus

Deliver:

- FieldKey compiler representation;
- parser bracket/computed keys;
- metadata;
- canonical ordering;
- typed/unconstrained/captured tails;
- lookup;
- generalized relation rules;
- reflection/iteration.

## P4 — Product/Sum reflection + derived capabilities

Deliver:

- Product traits/reflection;
- compiler derivation;
- user/intrinsic products;
- Sum/Variant/Variants;
- overloaded family reflection;
- source-defined utilities.

## P5 — static control + structural matching + recursion

Deliver:

- static match;
- static for;
- static while;
- return/break/continue;
- recursive type computation;
- memoization;
- symbolic residualization;
- proof predicates;
- direct structural type patterns.

## P6 — convergence/certification

Deliver:

- metadata schema upgrade;
- exported const fingerprints;
- incremental products;
- compiler materialization;
- optimizer integration;
- LSP/formatter/highlighter;
- docs supersession;
- public TypePack/RecordRows migration;
- benchmarks;
- cold/incremental parity.

---

# 19. Vertical certification matrix

Every major feature must be traced through:

```text
lexer/token
    ↓
parser
    ↓
AST
    ↓
source index / stable identity
    ↓
semantic declaration
    ↓
TypeStore / StaticValue store
    ↓
substitution / specialization
    ↓
inference
    ↓
relations / GADT refinement
    ↓
static evaluator
    ↓
incremental query product
    ↓
metadata export/import
    ↓
semantic lowering
    ↓
compiler materialization
    ↓
runtime representation
    ↓
reflection
    ↓
formatter/LSP/diagnostics
```

Parser success is not checkpoint completion.

---

# 20. Test and performance certification

## Bindings

Test immutable/mutable assignment, captures, destructuring, top-level globals, exports/imports, REPL, generated locals, and same-scope redeclaration.

## Fields

Test immutable default, `mut`, constructor writes, read-before-write, derived constructors, setters/delegation, inheritance, class-side fields, implicit-field policy.

## Static evaluation

Test literals, operators, short-circuiting, untaken invalid branches, loops, control transfer, recursion, cycle detection, budget exhaustion, cancellation, symbolic residuals, and closed-const rejection.

## Packs

Test empty Pack distinction, both lanes, duplicate labels, views, rest slices, equality/hash, subtyping, expansion, all rest modes, forwarding, and no-allocation fixed-call paths.

## Unordered products

Test all FieldKey kinds, optionality, width, typed key tails, collisions, canonical ordering, metadata roundtrip.

## Products/Sums

Test intrinsic and user-defined ordered/unordered products, nominal identity preservation, repeated variant names, overloaded variant shape resolution, GADT variants.

## Incrementality

For exported constants/static definitions, compare cold build and incremental edit products/diagnostics.

## Benchmarks

Measure:

- ordinary fixed calls;
- dynamic expanded calls;
- type-heavy generic compilation;
- recursive Pack algorithms;
- large unordered shapes;
- large variant families;
- static evaluator memoization;
- metadata load;
- incremental editor latency.

---

# 21. Instructions for the next agent

Before editing:

```bash
git status --short
git rev-parse HEAD
git log -8 --oneline

rg -n "BindingKind::(Let|Const)|BindingKind" \
  phalcom-ast phalcom-semantic phalcom-core phalcom-modules phalcom-lsp

rg -n "ConstWithoutInitializer|AssignToImmutable|is_mutable|mutable:" \
  phalcom-ast phalcom-semantic phalcom-core phalcom-modules phalcom-lsp

rg -n "TypePack|LabeledTypePack|CompleteTypePack|RecordRow|RecordFields" \
  phalcom-ast phalcom-semantic phalcom-core phalcom-type-meta \
  phalcom-modules phalcom-lsp

rg -n "finish_tuple|ArgumentPackBuilderObject|NewArgumentPack|InvokePack|RestMode" \
  phalcom-core

rg -n "VariantFamilyId|VariantInfo|ResolvedVariantCandidate|BranchProofEnvironment" \
  phalcom-semantic

rg -n "QueryBudget|CancellationToken|BudgetExceeded|RecursiveFixpoint" \
  phalcom-semantic
```

Then:

1. rebase all path/line assumptions against the actual HEAD;
2. create the official C4 checkpoint/requirements files;
3. preserve a strict “ratified vs open” ledger;
4. settle only the open decisions needed for the next implementation unit;
5. implement vertical slices, not parser-only islands;
6. keep static evaluation in `phalcom-semantic`;
7. make `phalcom-core` consume frozen semantic products;
8. benchmark fixed-call and semantic-query hot paths throughout;
9. do not add a second pattern resolver, row relation, variant-family table, or static-domain registry;
10. do not implement const functions in this checkpoint.

---

# 22. Non-negotiable C4 invariants

**C4-INV-01 — Staticness is semantic, not optimizer-derived.**

**C4-INV-02 — There is no compile-time VM.**

**C4-INV-03 — Static evaluation never falls back to arbitrary runtime dispatch.**

**C4-INV-04 — A successful closed `const` is fully reduced.**

**C4-INV-05 — Generic type computation may residualize.**

**C4-INV-06 — Runtime/static matching share one pattern and proof authority.**

**C4-INV-07 — Pack is semantically distinct from Tuple and Unit.**

**C4-INV-08 — Semantic argument Packs do not imply runtime allocation.**

**C4-INV-09 — Record is one intrinsic UnorderedProduct, not the definition of unordered reflection.**

**C4-INV-10 — Product shape reflection never erases nominal data identity.**

**C4-INV-11 — Variant Symbol denotes a family and may map to multiple variants.**

**C4-INV-12 — Unordered reflection order is deterministic but not semantic Record ordering.**

**C4-INV-13 — Open structural tails are not finite iterable sequences.**

**C4-INV-14 — Static values never depend on VM object identity.**

**C4-INV-15 — Stable metadata never serializes process/session IDs as identity.**

**C4-INV-16 — Static work is bounded and cooperatively cancellable.**

**C4-INV-17 — Public source ontology leads; current TypePack/RecordRow nodes are implementation details.**

**C4-INV-18 — Compiler-derived structural capabilities cannot be forged by ordinary impls.**

**C4-INV-19 — Runtime use of a const materializes its result; it does not rerun its initializer.**

**C4-INV-20 — C4 does not introduce const functions.**

---

# 23. Recommended first end-to-end spikes

First:

```telos
type Count<*P>: Int {
    let mut n = 0

    for _ in P {
        n += 1
    }

    n
}

const x = Count<*PositionalTypePack<Int, String, Bool>>
```

This validates block-bodied type parsing, static result domains, Pack/TypePack integration, static local mutation, static `for`, type application, `const`, canonical Int result, query dependencies, and diagnostics.

Second:

```telos
data Value {
    Number(_ value: Int)
    Number(_ value: Float)
}

type NumberVariants: Variants =
    VariantsOf<Value, #Number>
```

This validates Sum reflection, repeated variant families, Variants Pack, source-defined filtering, and ProductShape reflection.

Third: exercise positional/labeled/complete runtime rest capture and certify that a normal fixed call emits no argument-Pack allocation.

These are architectural probes, not special cases.

---

# 24. Completion criteria

LANG007.C4 is complete only when:

1. the source corpus uses new binding/field syntax;
2. no semantic consumer assumes plain `let` is mutable;
3. `const` is statically evaluated and never runtime-initialized;
4. parameterized/block-bodied `type` definitions execute through the bounded semantic evaluator;
5. static match/for/while share existing semantic authorities;
6. Pack is first-class across runtime, type system, static evaluation, and reflection;
7. rest captures use Pack;
8. ordinary fixed calls retain their no-Pack fast path;
9. generalized FieldKey unordered products work through parsing, relations, metadata, and reflection;
10. Product capabilities derive for intrinsic and user-defined data;
11. Sum/Variant reflection preserves overloaded Symbol families;
12. metadata round-trips exported C4 structures;
13. cold and incremental analysis agree;
14. recursion/budget/cancellation tests pass;
15. formatter/LSP/highlighting understand new syntax;
16. stale public TypePack/RecordFields surface is migrated or has an explicit compatibility exit plan;
17. old binding/static-domain docs are marked superseded;
18. performance benchmarks show no material fixed-call or ordinary-analysis regression;
19. no runtime VM path is required for static evaluation;
20. every intentionally deferred feature, especially const functions, is documented.

---

# 25. Final guidance

The implementation should preserve the simplification that motivated C4:

> **Telos should not grow a second miniature compile-time language.**

Static code is ordinary Telos code evaluated by a deterministic semantic evaluator over static values.

Reflection likewise should not become a catalog of opaque compiler handles when ordinary immutable Telos structures suffice:

```text
Pack<Type>
ProductShape
UnorderedShape
Variants = Pack<Variant>
```

The compiler should provide canonical facts efficiently, prove structural relationships correctly, and erase/materialize reflection as appropriate.

If an implementation step appears to require a second Pack ontology, a second pattern resolver, a second row relation, a second variant-family table, or a compile-time VM, stop and re-evaluate the architecture. C4 should converge the mature systems already present in the repository.
