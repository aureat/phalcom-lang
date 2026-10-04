# LANG007.C4 — Full Implementation Requirements Analysis

**Checkpoint:** `LANG007.C4`<br>
**Repository:** `aureat/phalcom-lang`<br>
**Audited branch:** `main`<br>
**Audited revision:** `8c3a98aab41ed8631e9f8746a6b29984cf77581d`<br>
**Audit date:** 2026-10-03<br>
**Status at audited revision:** design largely ratified; coherent C4 implementation not started<br>
**Purpose:** implementation decomposition, repository-grounded requirements analysis, risk/open-question ledger, and certification plan

---

## 1. Executive conclusion

C4 should be implemented as **nine plans**. Fewer than nine forces unrelated semantic authorities into oversized implementation units; more than nine would mostly fragment cross-cutting work that must land atomically to remain correct. The nine-plan decomposition is:

| Plan | Scope | Primary deliverable |
|---|---|---|
| **C4.P1** | Binding/field phase and mutability cutover | `let` immutable runtime, `let mut` mutable runtime, `const` static; immutable-by-default fields; corpus migration |
| **C4.P2** | Static-definition and evaluator foundation | General static result domains, `StaticValue`, bounded semantic evaluator, block/expression `type` bodies, fully reduced closed `const` |
| **C4.P3** | First-class Pack and call/rest convergence | Runtime/static Pack, Pack views, Pack typing, rest capture to Pack, preserved zero-allocation fixed-call path |
| **C4.P4** | Generalized unordered-product calculus | Canonical `FieldKey`, row/type/runtime shape migration, generalized tails/lookup/relations/reflection order |
| **C4.P5** | Product/Sum reflection and compiler-derived capabilities | `Product`/`OrderedProduct`/`UnorderedProduct`/`Sum`, `ProductShape`, `Variant(s)`, static reflection publication |
| **C4.P6** | Static control, structural matching, iteration, and proof predicates | Static `match`/`for`/`while`, branch proofs, structural patterns, iteration semantics, proof operations |
| **C4.P7** | Recursive generic static computation and residualization | Recursive type computation, memoization, cycle handling, normalized symbolic residual IR, specialization |
| **C4.P8** | Incremental durability, metadata, cross-module constants, and materialization | Query products/dependencies, schema upgrade, exported static values/residuals, constant materialization descriptors |
| **C4.P9** | Tooling, compatibility retirement, performance and release certification | LSP/editor/REPL/tooling, docs/codemods cleanup, benchmarks, cold/incremental parity, final invariant audit |

This differs deliberately from the handoff's preliminary P1–P6 staging. The handoff correctly identified the feature families, but the live repository shows three boundaries that deserve independent implementation plans:

1. **binding/field migration must be isolated from evaluator construction** because it changes existing runtime behavior and the full source corpus before any new static execution is trustworthy;
2. **recursive/symbolic generic computation must be isolated from the core closed evaluator** because its normalization, cycle, specialization, and residual semantics are substantially more complex than closed `const` reduction;
3. **incremental/metadata/materialization durability must be isolated from source-level reflection/control** because stable identity, schema compatibility, dependency invalidation, and compiler materialization are a different correctness boundary and must be certified cross-module.

The repository is well positioned for C4. The required semantic authorities mostly already exist: canonical `TypeStore` interning, C3 static domains and heterogeneous static arguments, exact-case/GADT machinery, variant-family identity, match resolution/projection, proof-aware conformance results, query budgets/cancellation, incremental semantic products, and shared runtime product storage. The implementation must **converge these systems rather than add parallel ones**.

The highest-risk areas are not parser syntax. They are:

- changing the meaning of `const` from runtime immutable binding to a compile-time evaluation boundary;
- preventing static evaluation from becoming a hidden VM or runtime-dispatch interpreter;
- preserving the fixed-call no-Pack-allocation fast path while making Pack first-class;
- changing unordered keys from string/Symbol-centric storage to stable heterogeneous `FieldKey` identity across semantic rows, runtime shapes, fingerprints, metadata, and reflection;
- publishing Product/Sum reflection as ordinary Telos static data without making compiler structural facts forgeable by user-written trait impls;
- residualizing generic static computation without serializing raw source AST or leaking store-local identities;
- ensuring exported constant **values**, not only their types, participate in incremental fingerprints and cross-module invalidation.

---

## 2. Sources and audit basis

### 2.1 Attached design artifacts

The analysis treats the following attached files as the C4 design record, with the handoff taking precedence when examples conflict with later ratified terminology:

- `LANG007.C4-structural-static-upgrade-handoff.md` — canonical handoff, open-decision ledger, invariants, preliminary staging, completion criteria.
- `Products Source.md` — proposed Pack/Product/UnorderedProduct source definitions and reflection vocabulary.
- `Static For Loop.md` — intended static iteration ergonomics and shape-specific semantics.
- `Static Matching.md` — intended Pack/structural static algorithms and matching examples.
- `Sum Source.md` — Sum/Variant reflection model and source-defined utility examples.

### 2.2 Live repository verification

The audit verified that repository HEAD is exactly the handoff's prepared revision:

```text
8c3a98aab41ed8631e9f8746a6b29984cf77581d
```

Therefore the handoff is not stale at the commit level. It is still necessary to distinguish **design examples that predate final naming/syntax choices** from live implementation facts.

### 2.3 Major live implementation seams audited

The repository audit covered, at minimum:

```text
phalcom-ast/src/{ast.rs,parser.rs,token.rs,lexer.rs}
phalcom-semantic/src/
    checker/{statement.rs,expression.rs,iteration.rs,row_inference.rs}
    types/{store.rs,row.rs,type_member.rs,projection.rs,substitution.rs,
           relation.rs,outcome.rs,...}
    {data_semantics.rs,enum_semantics.rs,match_semantics.rs,identity.rs,
     impls.rs,derived_conformance.rs,trait_dispatch.rs,type_alias.rs}
    db/{key.rs,query.rs,fingerprint.rs}
    metadata/{export.rs,...}
    source_index/{builder.rs,occurrence.rs}
phalcom-type-meta/src/{header.rs,bundle.rs,...}
phalcom-modules/src/{interface.rs,graph.rs,...}
phalcom-core/src/
    compiler/{lib/mod.rs,lib/expr.rs,lib/product_opt.rs,attributes.rs,...}
    modules/{linkage.rs,semantic_lowering.rs}
    heap/pack_builder.rs
    product/{mod.rs,shape.rs,layout.rs,storage.rs,view.rs,...}
    vm/dispatch.rs
    bytecode.rs
phalcom-core/core/universe/src/collections/{tuple.ph,record.ph,...}
phalcom-lsp/src/{semantic_tokens.rs,hover.rs,completion.rs,...}
phalcom-repl/src/{common.rs,highlighter.rs,completer.rs,...}
tools/vsphalcom/syntaxes/phalcom.tmLanguage.json
```

The audit also checked relevant predecessor checkpoints where current code embodies earlier architectural commitments, especially LANG005 product convergence, trait/conformance proof-state closure, and LANG007 C1–C3 static-domain/type-pack work.

---

## 3. Ratified C4 language requirements

These are implementation requirements, not suggestions. They should not be reopened accidentally by local implementation convenience.

### 3.1 Binding and phase semantics

```telos
let x = runtimeExpression
let mut y = runtimeExpression
const z = staticExpression
```

| Form | Evaluation phase | Reassignable | Required property |
|---|---|---:|---|
| `let` | runtime | no | ordinary runtime expression |
| `let mut` | runtime | yes | ordinary runtime expression |
| `const` | compile time | no | initializer must statically reduce for a closed binding |

Staticness is **declared**, not inferred from optimizer knowledge. A runtime `let` initialized with a literal does not become a constant merely because the optimizer can fold it.

A runtime use of `const` consumes/materializes the already-computed value. It must never rerun the initializer at runtime.

### 3.2 Field mutability

Target class spelling:

```telos
class Counter {
    _name: String
    mut _count: Int
}
```

The existing semantic bit `FieldDef.mutable` remains useful. The source default changes from mutable to immutable.

### 3.3 Static computation

C4's static evaluator is a sibling semantic evaluation strategy over ordinary Telos semantic constructs. It is **not**:

- a compile-time VM;
- bytecode execution during semantic analysis;
- arbitrary runtime method dispatch;
- arbitrary native call execution.

Parameterized `type` declarations are C4's reusable source-defined static abstraction. C4 **does not** add const functions or const methods.

Canonical forms:

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
    let mut n = 0
    for _ in P {
        n += 1
    }
    n
}
```

A block-bodied `type` definition omits `=`.

### 3.4 Static control

Static evaluation supports ordinary Telos forms, when operands are statically eligible:

- `if`;
- `match`;
- `for`;
- `while`;
- local `let`;
- local `let mut`;
- assignment to evaluator-local mutable slots;
- `return`;
- `break`;
- `continue`;
- destructuring/pattern bindings.

Untaken branches are not evaluated.

### 3.5 Pack

Pack is a first-class immutable ordered two-lane product:

```telos
intrinsic data Pack<***Es>(***Es)
```

It has:

- positional lane;
- labeled lane;
- ordering within each lane;
- Symbol-only ordered labels;
- lane-sensitive identity;
- no duplicate labels;
- transparent value semantics.

`Pack()` is distinct from Unit/`()`, Tuple, and `{}`/Record.

Rest semantics:

```text
*rest   -> positional Pack
**rest  -> labeled Pack
***rest -> complete Pack
```

A semantic argument Pack does **not** imply a runtime allocation for ordinary calls.

### 3.6 Product model

Public structural capabilities:

```text
Product
OrderedProduct
UnorderedProduct
```

Ordered constituents are **entries**; labeled entries have **labels**. Unordered constituents are **fields**; fields have **keys**.

Canonical shape vocabulary:

```text
ProductShape::Ordered(OrderedShape)
ProductShape::Unordered(UnorderedShape)
```

Nominal product identity is never erased by equal payload shape.

### 3.7 Unordered keys

Canonical field-key domain:

```telos
type FieldKey =
    Symbol | String | Int | Bool
```

This applies to unordered structural identity and reflection. Ordered Pack/Tuple/call labels remain Symbols.

### 3.8 Sum model

Enumerated data exposes:

```text
Sum
Variant(name: Symbol, shape: ProductShape)
Variants = PositionalPack<Variant>
```

A Symbol identifies a **variant family**, not necessarily a unique variant. Repeated names remain distinct reflected entries in declaration order.

### 3.9 Reflection

Reflection should be ordinary immutable Telos static data wherever feasible, rather than opaque compiler handles. Examples include:

```text
Pack<Type>
ProductShape
OrderedShape
UnorderedShape
Variant
Variants = Pack<Variant>
```

Compiler-generated structural facts remain authoritative and unforgeable even though their reflected values are ordinary data.

---

## 4. Repository baseline: exact C4 delta

### 4.1 Binding semantics are currently the inverse of the target

The live AST says:

```rust
pub enum BindingKind {
    /// A mutable `let` binding.
    Let,
    /// An immutable `const` binding.
    Const,
}
```

`LetBinding` documentation also states that missing initializer yields `None` for `let` and is rejected for `const`.

The current assumption propagates beyond AST:

- semantic checker flow slots mark `BindingKind::Let` mutable;
- source index publishes mutability from the same test;
- compiler locals/globals use `matches!(binding.kind, BindingKind::Let)`;
- product optimizer uses the same distinction;
- module interfaces reduce the distinction to `DeclarationSurface.is_const`;
- top-level linkage distinguishes mutable global storage;
- LSP hover/token docs and REPL/editor keyword inventories describe the old meaning;
- compiler-generated AST creates `BindingKind::Let` scratch/owner bindings and therefore must be audited rather than mechanically rewritten.

**Requirement:** represent evaluation phase and runtime mutability as separate semantic facts even if the surface AST keeps a compact `BindingKind::{Let, LetMut, Const}` enum.

Recommended semantic normalization:

```rust
enum BindingPhase {
    Runtime,
    Static,
}

enum RuntimeMutability {
    Immutable,
    Mutable,
}
```

with surface mapping:

```text
Let    -> Runtime + Immutable
LetMut -> Runtime + Mutable
Const  -> Static  + Immutable
```

No consumer should ask “is this const?” when it actually means “is this immutable?” or “does this have runtime storage?”.

### 4.2 There is no `mut` token today

`Token::Mut` does not exist at the audited revision. `mut` must be added to:

- token enum and lexer keyword recognition;
- binding grammar after `let`;
- class-field grammar before a field;
- parser recovery/expected-token sets;
- semantic-token classification;
- keyword hover/completion inventories;
- REPL keyword tables/highlighters/completers;
- VS Code TextMate grammar;
- fuzz dictionaries/goldens where maintained.

### 4.3 Field semantics currently use `const` as the immutable marker

`FieldDef.mutable` is documented as:

```text
bare field       -> mutable = true
const field      -> mutable = false
```

The field layout/lifecycle machinery is valuable and should remain authoritative. The parser/default flips; later consumers should continue reading `FieldDef.mutable`.

A critical live behavior exists for **implicit fields inferred from assignment**: compiler code explicitly treats them as always mutable. This is not compatible with “immutable by default” without an explicit C4 policy decision; see Open Question O3.

### 4.4 `type` definitions cannot yet represent C4 bodies

Current `TypeAliasDef` is conceptually:

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

Two limitations matter:

1. the result is represented as a **kind syntax**, while C4 allows result domains such as `Bool`, `Int`, `Variants`, `ProductShape`, etc.;
2. the body is a `TypeAnnotation`, not an ordinary expression or statement block.

C4 therefore needs a generalized static-definition syntax/model. It should not permanently fork a second expression language.

### 4.5 Static values are broader than `TypeData`

C3 already established canonical static-domain machinery including `KindData::StaticDomain`, `TypeData::ConstInt`, `TypeData::StaticData`, and heterogeneous `StaticArgument`. Those must be reused.

However C4 `const` values include ordinary static values such as Bool, Float, String, Symbol, Pack, Tuple, Record, transparent data, and Sum values. Forcing all of those into `TypeData` would pollute the type arena and conflate “canonical kinded static term” with “ordinary compile-time value”.

C4 therefore needs a dedicated `StaticValue`/`StaticValueId` arena that can embed/reference `TypeId` where appropriate.

### 4.6 `TypeMemberDomain` is currently kind-only

The live `types/type_member.rs` models type-member result domains as:

```rust
pub enum TypeMemberDomain {
    Kind(KindId),
}
```

Its owner/operation enum is also centered on the current public ontology:

```text
Tuple
Record
TypePack
LabeledTypePack
CompleteTypePack
RecordRow
```

and operations such as `TuplePositionals`, `RecordFields`, `TypePackMap`, etc.

C4 reflection members may return non-kind static domains and ordinary static ADT values. This is a key architectural seam: the existing associated/static member pipeline must generalize from “returns a kinded type-level term” to “returns a statically typed semantic value/domain”.

### 4.7 Public TypePack/RecordFields facades are live Universe API

The Universe currently publishes intrinsic source facades such as:

```telos
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data CompleteTypePack
intrinsic type data RecordFields
```

Tuple/Record intrinsic members refer to them, and semantic projection normalizers recognize these identities.

C4's public model instead says:

```text
PositionalTypePack = PositionalPack<Type>
LabeledTypePack    = LabeledPack<Type>
CompleteTypePack   = CompletePack<Type>
```

This must be handled as a **public-surface migration over stable internal optimized terms**, not a forced immediate deletion of `TypeData::TypePack`, `LabeledTypePack`, `CompleteTypePack`, row IDs, and pack concatenation nodes.

### 4.8 Record rows are string-keyed today

The live semantic row core is effectively:

```rust
pub struct RecordRowField<T = TypeId> {
    pub name: Box<str>,
    pub ty: T,
    pub presence: StructuralMemberPresence,
}
```

`RecordRowData` canonicalizes by sorting `name` and binary-searches by `&str`.

C4 requires a canonical key object with:

```text
Symbol | String | Int | Bool
```

This migration affects at least:

- AST for record type fields/computed keys;
- semantic row interning;
- row inference/lacks constraints;
- substitution and rigidization;
- relations/subtyping;
- pattern resolution/coverage for records;
- structural fingerprints;
- type-member/reflection lookup;
- metadata export/import;
- runtime record shapes and lookup;
- diagnostics/formatting.

### 4.9 Runtime Record product shapes are also Symbol-keyed

The runtime product registry currently stores Record shape labels as `Symbol`:

```rust
pub struct RecordProductShape {
    pub presentation_labels: Box<[Symbol]>,
    pub logical_labels: Box<[Symbol]>,
    pub presentation_to_logical: Box<[u32]>,
}
```

and caches:

```rust
HashMap<Symbol, u32>
```

Therefore heterogeneous `FieldKey` is not only a semantic-row change; it crosses the VM product metadata boundary.

The AST already uses shared `ProductLabel` syntax for Tuple and Record, including computed `[expr]:` labels. C4 makes the domains context-sensitive:

```text
Tuple / Pack / call label -> Symbol only
Record field key          -> FieldKey
```

The compiler must stop applying a universal `GuardSymbol`-style assumption to Record key formation while preserving Symbol validation on ordered labels/calls.

### 4.10 Match/GADT authority already exists and must be reused

`match_semantics.rs` and checker/lowering products already own:

- `MatchResolution`;
- `MatchArmResolution`;
- `PatternResolution`;
- `ResolvedVariantPattern`;
- `ResolvedVariantCandidate`;
- pattern binding resolutions;
- usefulness/exhaustiveness;
- exact variant identities;
- `BranchProofEnvironment` and GADT branch equalities.

Static matching should consume these products. Building a new static-only pattern resolver would duplicate the hardest correctness logic in the compiler.

### 4.11 Variant families already model repeated variant names

The semantic identity layer has `VariantId` and `VariantFamilyId`; `EnumInfo` retains declared variants and family grouping. `VariantInfo` retains exact payload shape, constructor signature, result template, exact-case template, and case environment.

This maps directly to C4's rule that a reflected Symbol can name zero/one/many structural variants. Sum reflection should be derived from this table, not from source-name scans.

### 4.12 Proof-state granularity predecessor blocker is already closed

Older LANG005 planning identified a hard pre-C4 requirement: applicability must distinguish proven/applicable, disproven/not-applicable, unknown, blocked, dynamic, cancellation, and budget failure. The current repository has this richer model (`ImplApplicabilityResult`, `GenericConformanceProof`, `TraitDispatchResolution`, etc.).

This is **not a remaining blocker** at the audited revision. C4 should consume the rich terminals and never collapse them.

### 4.13 Query budgets and cancellation are mature

`QueryBudget` and `CancellationToken` already exist. Default budget dimensions include steps, relation pairs, SCC iterations, type depth, and diagnostic notes.

The static evaluator should integrate with this authority. Do not hide separate recursion/iteration counters that produce different terminal semantics.

### 4.14 Incremental DB products are mature, but C4 has no query keys yet

`db/key.rs`/`db/query.rs` provide typed query keys, dependency recording, product fingerprints, reuse validation, blocked/failed states, and cancellation/budget terminals.

There is currently no canonical `StaticDefinition` or `ConstantBinding` query product. C4 needs these so constant-value changes and static-definition changes invalidate dependents correctly.

### 4.15 Runtime products are already converged

`phalcom-core::product` already has:

- interned product shape metadata;
- physical product layout registry;
- `ProductStorage`;
- anonymous Tuple/Record descriptors;
- scalar replacement/allocation sinking for Tuple/Record/data/variant products;
- shared construction boundaries.

Pack should reuse this substrate where materialized, while keeping Pack's nominal runtime identity distinct from Tuple.

### 4.16 Argument Pack assembly already exists privately

`ArgumentPackBuilderObject` already holds:

```rust
positionals: Vec<Value>
labels: Vec<Symbol>
labeled_values: Vec<Value>
pending_label: Option<usize>
```

Dynamic outgoing calls use pack-building bytecodes; fixed calls already have tests that assert no `NewArgumentPack`/`InvokePack` path.

Current rest finishing still routes through Tuple finalization (`FinishTuplePack` / `finish_tuple` paths). C4 should add immutable Pack finalization/rest capture while keeping the builder VM-private.

The current duplicate-label check uses linear `labels.contains(...)`. For large dynamically assembled Packs this can become quadratic. C4 should preserve simple small-N behavior but add a thresholded transient index or equivalent if benchmarks show the new first-class Pack surface increases large-pack frequency.

### 4.17 Metadata is schema v10

`TYPE_METADATA_SCHEMA_VERSION` is 10. v10 already covers structural presence and anonymous tuple/record remainders, with prior revisions covering C3 static-domain information.

C4 requires a schema revision for, at minimum:

- generalized `FieldKey`;
- exported ordinary compile-time constants;
- stable Pack/static structural values where they cross modules;
- generalized static-member result domains;
- normalized residuals if exported unresolved static definitions require them.

---

## 5. Design-artifact inconsistencies and stale examples

These do **not** invalidate the ratified design, but they should be corrected before implementation plans are treated as copy-paste source specs.

### 5.1 Pack view constructor names are stale in `Static Matching.md`

Canonical `Products Source.md` uses:

```telos
PositionalPackView::Entry(...)
LabeledPackView::Entry(...)
```

`Static Matching.md` still contains examples using `TypePackView::Item` and `LabeledTypePackView::Item`. Those are stale. C4 should use the lane-specific Candidate A design only.

### 5.2 Complete Pack projection names are inconsistent

Canonical source defines:

```telos
CompletePack<T>::PositionalEntries
CompletePack<T>::LabeledEntries
```

but `Static Matching.md` uses `P::Positionals` and `P::Labeled` in an example. The final names should follow `Products Source.md` unless deliberately reopened.

### 5.3 Old Record reflection terminology remains in matching examples

`Static Matching.md` uses `RecordProduct` and `RecordField` in one example, while the ratified vocabulary is `UnorderedProduct`, `ProductField`, `FieldInfo`, and `Presence`.

### 5.4 `FieldPresence` vs `Presence`

`Static For Loop.md` comments refer to `FieldPresence`; `Products Source.md` defines `Presence`. `Presence` is the canonical spelling in the attached source model.

### 5.5 `T::Shape` examples omit the `ProductShape` wrapper

`Products Source.md` defines:

```telos
trait Product {
    type Shape: ProductShape
}
```

Therefore a generic `T: Product` match should structurally see `ProductShape::Ordered(...)` / `ProductShape::Unordered(...)`, not directly `OrderedShape(...)` unless the subject is `T::OrderedShape` under `T: OrderedProduct`.

Several matching examples should be updated accordingly.

### 5.6 Block-bodied `type` examples in `Sum Source.md` use stale `= { ... }`

The handoff explicitly ratifies:

```text
expression body -> `=`
block body      -> no `=`
```

Some Sum utilities are shown as:

```telos
type Foo<...>: X = {
    ...
}
```

Those should become:

```telos
type Foo<...>: X {
    ...
}
```

### 5.7 `VariantsNamed` example has an unbound `variant`

The source example patterns directly as `for Variant(name, _) in Vs` but later appends `variant`, which was never bound. The handoff's later form—bind `variant`, then `match variant`—is correct and avoids requiring an as-pattern feature merely for this utility.

### 5.8 Free-standing static `for` examples need contextual qualification

Several examples show a top-level `for T in SomeType`. C4 has ratified static `for` **inside static evaluation contexts** (`const` blocks and static `type` bodies). It has not ratified arbitrary top-level compile-time executable statements. Documentation should make those examples contextual/pseudocode unless top-level static statements are separately designed.

### 5.9 Runtime structural iteration is not automatically implied

The design requires Product values/types to support **structural static iteration**. A source example such as:

```telos
for value in pair { ... }
```

must not accidentally create a new runtime `Iterable` protocol requirement for all Products unless that is separately specified. In C4 static contexts, iteration is evaluator/compiler structural semantics and does not imply runtime method dispatch.

---

## 6. Required architecture

### 6.1 One semantic phase model

Do not let every subsystem reinterpret `BindingKind`. Normalize early:

```text
surface spelling
    ↓
phase + runtime mutability + storage class
    ↓
semantic/local/module products
    ↓
compiler lowering
```

A `const` must have no ordinary runtime initialization edge.

### 6.2 One static value model

Recommended shape:

```rust
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct StaticValueId(u32);

pub enum StaticValue {
    Unit,
    Bool(bool),
    Int(/* canonical integer */),
    Float(/* canonical semantic float representation */),
    String(StaticStringId),
    Symbol(StableSymbol),
    Type(TypeId),
    StaticTerm(TypeId),
    Pack(StaticPackId),
    Tuple(StaticProductValueId),
    Record(StaticProductValueId),
    Data(StaticDataValueId),
    Variant(StaticVariantValueId),
}
```

Requirements:

- no `phalcom_core::Value` dependency;
- no `ObjRef` or VM class/object identity;
- deterministic equality and hashing;
- explicit Float equality/hash rules matching Telos semantic equality;
- stable String/Symbol export encoding;
- canonical or shareable immutable result storage;
- transient evaluator builders for efficient repeated append;
- direct reference to canonical `TypeId`/C3 static terms where correct.

### 6.3 One evaluator, explicit control machine

User loops/recursion must not become unbounded host recursion. The evaluator should use explicit frames/work items and control transfer:

```rust
enum EvalControl {
    Value(StaticValueId),
    Return(StaticValueId),
    Break,
    Continue,
}
```

The frame model should retain:

- source span/static-definition identity;
- lexical static environment;
- mutable local slot array;
- specialization/proof environment;
- budget/cancellation handle;
- diagnostic trace frame.

### 6.4 One structural decomposition authority

Product subtyping, argument acceptance, typed overload matching, static structural matching, rest capture, Sum family selection, and proof predicates should converge on common shape decomposition/relation primitives.

Conceptually:

```text
canonical structural subject
    ↓
decompose ordered/unordered/sum family
    ↓
constraint or pattern match
    ↓
Proven / Refuted / Symbolic-or-terminal + captures + proof environment
    ↓
consumer policy
```

### 6.5 Reflection is semantic data, capability evidence is compiler-owned

Do not conflate:

1. the **reflected value** (`ProductShape`, `Variants`, etc.), which should be ordinary static data; and
2. the **fact that a declaration is structurally a Product/Sum**, which is compiler-derived semantic evidence and must not be forgeable.

### 6.6 Runtime Pack must reuse product storage without collapsing nominal identity

A materialized Pack may use the same layout/storage substrate as Tuple but must carry Pack's own runtime descriptor/class identity. The compiler may scalar-replace it or keep it as an argument window when unobservable.

### 6.7 FieldKey must have two representations: semantic-stable and runtime-efficient

Do not serialize/interchange VM `Symbol(u32)` as a stable key. A useful split is:

```rust
// semantic / metadata-stable
pub enum FieldKey {
    Symbol(StableSymbol),
    String(StaticString),
    Int(CanonicalInt),
    Bool(bool),
}

// runtime key representation may use interned/boxed forms optimized for lookup,
// but must convert through a canonical semantic encoding at metadata boundaries.
```

The canonical total order must be version-stable and defined explicitly before metadata fingerprints are finalized.

---

## 7. Open-decision ledger

The following questions are genuinely open or require explicit ratification before the affected plan lands. Recommendations below are implementation recommendations, not silent language-law changes.

| ID | Decision | Blocking plan(s) | Recommendation |
|---|---|---|---|
| **O1** | May `let` / `let mut` omit an initializer? | P1 | Require initializer for immutable `let`; optionally preserve no-initializer only for `let mut` if uninitialized-as-`None` remains intentional |
| **O2** | Exact one-time initialization rule for immutable fields | P1 | Permit writes only in constructor lifecycle states already tracked; require definite initialization before readable/complete object state |
| **O3** | Implicit field declaration by assignment | P1 | Prefer removing/diagnosing implicit declaration in C4; second-best is explicitly document it as legacy mutable-only exception. Do not infer immutable fields from arbitrary assignment |
| **O4** | Canonical cross-kind `FieldKey` total order | P4/P8 | Define a spec-level tag order plus within-kind order; never use Rust enum discriminants or process-local Symbol IDs |
| **O5** | Heterogeneous Product loop binder typing | P6 | Prefer per-iteration refinement for static loops; do not collapse key/value correlation to a broad union if evaluator can preserve exact element facts |
| **O6** | Static String equality / `===` / interning/materialization | P2/P8 | Semantic equality by contents; identity-sensitive `===` only if runtime allocation identity provably cannot affect result. Export UTF-8 contents, not arena IDs |
| **O7** | Symbolic residual IR | P7/P8 | Define normalized semantic residual nodes; do not persist raw AST or evaluator frames |
| **O8** | Exact structural type-pattern grammar | P6 | Reuse existing pattern grammar and Pack/row remainder syntax wherever possible; add only forms needed to represent canonical captures/tails |
| **O9** | `Equivalent` / `Subtype` / `Conforms` public signatures and symbolic behavior | P6/P7 | Return Bool only when proof/refutation is semantic; generic uncertainty must residualize or surface a non-Bool terminal internally, never guess false |
| **O10** | Public GADT refinement metadata on `Variant` | P5/P6 | Keep branch proof match-only in C4 unless a concrete source use requires public refinement reflection; avoid bloating `Variant` prematurely |
| **O11** | Unforgeable Product/Sum capability mechanism | P5 | Use compiler-owned derived capability evidence/associated surfaces, not an ordinary user-writable impl path |
| **O12** | Ordinary `while` semantic AST preservation | P6 | Preserve explicit source-control identity before static evaluation; do not recognize arbitrary user `.whileTrue` calls as static loops |
| **O13** | Runtime materialization of arbitrary `const` data/variant values | P2/P8 | Define a semantic materialization descriptor independent of VM objects; compiler/runtime chooses physical construction |
| **O14** | Cross-module symbolic static definition policy | P7/P8 | Export normalized definition/residual semantics only if required; prefer importing definition semantic products and specializing locally over shipping evaluator-internal state |
| **O15** | Compatibility lifetime of public `TypePack` / `RecordFields` facades | P3/P4/P9 | Establish a bounded compatibility window or same-checkpoint source migration. Do not leave two indefinitely first-class public ontologies |
| **O16** | First-class Pack equality/hash and Float-containing Pack semantics | P3 | Delegate element equality/hash to ordinary Telos semantic value rules; lane/order/labels participate in identity |

### 7.1 Decisions that should be settled before coding P1

O1–O3 must be decided before the binding/field corpus rewrite. A codemod cannot preserve behavior if the new language has no answer for currently valid uninitialized `let` or implicit-field assignment forms.

### 7.2 Decisions that may be deferred until their owning plan

O4–O16 can remain open until immediately before the named plan, provided preceding plans do not encode an accidental answer in serialized identity or public syntax.

---
## 8. Exact plan decomposition and dependency graph

### 8.1 Why C4 needs exactly nine plans

The handoff's P1–P6 staging is a useful first partition, but repository inspection shows that several of those buckets combine changes with different authorities, failure modes, and certification boundaries. In particular:

- binding/field semantics can and should be migrated before the evaluator exists;
- the static evaluator substrate is independent from first-class Pack runtime representation;
- generalized `FieldKey` changes the row calculus **and** runtime Record shape authority and therefore cannot safely hide inside a generic “product reflection” plan;
- Product/Sum reflection depends on Pack and unordered-product shape facts but has a separate unforgeable-capability problem;
- static control/matching can reuse existing semantic products only after reflection and evaluator primitives exist;
- recursion/residualization/memoization is a separate termination/incrementality problem from executing closed static programs;
- metadata/export/materialization must freeze only after the semantic forms stabilize;
- tooling, compatibility retirement, documentation, and final performance/cold-incremental certification are checkpoint-wide closure work and should not contaminate foundational implementation plans.

Therefore the implementation should be planned as the following **nine** units:

| Plan | Scope | Primary authority | Depends on | Produces |
|---|---|---|---|---|
| **C4.P1** | binding + field semantic cutover and corpus migration | AST/checker/compiler/lifecycle | none | correct `let` / `let mut` / `const` phase surface and immutable-by-default fields |
| **C4.P2** | static-value model, static definition syntax, closed evaluator foundation, `const` | `phalcom-semantic` | P1 | deterministic semantic evaluator for closed non-iterative/non-recursive static programs |
| **C4.P3** | first-class Pack and invocation/rest convergence | semantic products + `phalcom-core` product/call substrate | P1, P2 value hooks | runtime/static Pack, views, rest capture, fixed-call preservation |
| **C4.P4** | generalized unordered-product / `FieldKey` calculus | row/type system + runtime Record shape | P2 value hooks | `Symbol|String|Int|Bool` keyed rows, tails, lookup, reflection-ready shapes |
| **C4.P5** | Product/Sum reflection and compiler-derived capabilities | data/enum semantics + associated surfaces | P3, P4 | `ProductShape`, derived Product/Ordered/Unordered, `Variant`/`Variants`, `Sum` |
| **C4.P6** | static control, structural matching, iteration, proof predicates | static evaluator + match/GADT + structural decomposition | P2–P5 | static `match`/`for`/`while`, branch proofs, structural patterns, proof operations |
| **C4.P7** | recursive generic static computation, residualization, memoization | evaluator + specialization + query budgets | P6 | bounded recursive `type` computation and canonical symbolic residuals |
| **C4.P8** | durability, incrementality, metadata, compiler materialization | semantic DB + type metadata + compiler lowering | semantic forms from P2–P7 stable | cross-module/exported constants/static definitions and frozen runtime materialization |
| **C4.P9** | public ontology/tooling migration and checkpoint certification | whole workspace | P1–P8 | compatibility retirement, editor/REPL/docs, benchmark gates, cold/incremental parity, C4 closeout |

This is not merely a preferred schedule. Each row has an independent architectural invariant and a meaningful “done” boundary. Splitting any row further would create parser-only or representation-only islands; merging adjacent rows would make rollback, review, and performance attribution materially harder.

### 8.2 Dependency DAG

```text
C4.P1  binding / field transition
  │
  ├───────────────┐
  ▼               │
C4.P2 static evaluator foundation
  │               │
  ├──────────┐    │
  ▼          ▼    │
C4.P3 Pack   C4.P4 FieldKey/unordered products
  │          │
  └────┬─────┘
       ▼
C4.P5 Product/Sum reflection
       │
       ▼
C4.P6 static control + structural matching
       │
       ▼
C4.P7 recursion + residualization + memoization
       │
       ▼
C4.P8 incrementality + metadata + materialization
       │
       ▼
C4.P9 migration + tooling + performance + checkpoint closure
```

P3 and P4 are deliberately parallelizable after P2 establishes the shared static-value and result-domain seams. P8 should add temporary serialization/query hooks as earlier plans require tests, but the **schema bump and durable public encoding should be finalized only after P7 has frozen residual/value forms**.

### 8.3 Review rule for all plans

Every plan must deliver a vertical slice through all layers that are semantically relevant to that plan. “Parser accepts it” or “TypeStore can represent it” is never sufficient. Each plan must explicitly account for:

```text
source syntax
→ AST / source identity
→ semantic formation/checking
→ canonical representation
→ substitution/specialization where applicable
→ query dependency/fingerprint
→ lowering or explicit compile-time erasure
→ diagnostics
→ tests
→ performance / allocation behavior
```

---

# 9. C4.P1 — Binding, mutability, field lifecycle, and corpus cutover

## 9.1 Objective

Make the language's binding and field semantics match C4 **without yet requiring general static evaluation**:

```telos
let x = runtimeExpression       // runtime, immutable
let mut y = runtimeExpression   // runtime, mutable
const z = staticExpression      // compile-time, immutable; P2 evaluates it

class Counter {
    _name: String               // immutable by default
    mut _count: Int             // mutable
}
```

P1 must separate two dimensions that are currently conflated:

```text
binding phase:       Runtime | Static
runtime mutability:  Immutable | Mutable
```

`const` is no longer “the immutable spelling of a runtime local.” It is a **phase boundary**. The implementation should make this distinction explicit enough that P2 can attach evaluation semantics without another workspace-wide rename.

## 9.2 Entry decisions

**Must settle O1, O2, O3 before the source corpus rewrite.**

Recommended decisions for implementation quality:

1. `let` requires an initializer because an immutable runtime binding without a value has no useful one-time assignment model and preserving current implicit `None` would create a surprising exception.
2. `let mut` may omit an initializer **only if** the language intentionally preserves the historical `None` initialization behavior; otherwise require initializers uniformly. This must be explicit, not inherited accidentally.
3. immutable fields may be initialized exactly through the existing constructor lifecycle authority, with flow analysis proving legal first writes and rejecting later writes.
4. implicit field declaration by assignment should be retired or isolated as a documented mutable-only legacy form. It must never silently create an immutable field whose initialization semantics bypass declaration analysis.

## 9.3 Repository areas to modify

Primary:

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/error.rs

phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/db/fingerprint.rs

phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/product_opt.rs
phalcom-core/src/compiler/attributes.rs
phalcom-core/src/modules/linkage.rs

phalcom-modules/src/interface.rs
phalcom-modules/src/linker.rs / graph consumers as applicable
```

Tooling/corpus touched as part of the same atomic cutover:

```text
phalcom-lsp/src/semantic_tokens.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/completion.rs
phalcom-repl/src/common.rs
phalcom-repl/src/highlighter.rs
phalcom-repl/src/completer.rs
phalcom-core/bin/gen-core-table/**
tools/vsphalcom/syntaxes/phalcom.tmLanguage.json
fuzz/phalcom.dict
phalcom-core/core/**/*.ph
examples/**/*.ph
tests/fixtures/**/*.ph
```

## 9.4 AST and semantic representation

Use source syntax identity in the AST:

```rust
pub enum BindingKind {
    Let,
    LetMut,
    Const,
}
```

Then normalize for semantic consumers:

```rust
pub enum BindingPhase {
    Runtime,
    Static,
}

pub enum RuntimeMutability {
    Immutable,
    Mutable,
}

impl BindingKind {
    fn phase(self) -> BindingPhase { ... }
    fn runtime_mutability(self) -> Option<RuntimeMutability> { ... }
}
```

The exact Rust names can differ, but **do not force every consumer to repeat**:

```rust
matches!(kind, BindingKind::LetMut)
```

when it really wants “may assign at runtime.” Centralized normalization prevents the old `BindingKind::Let => mutable` assumption from surviving in obscure code paths.

## 9.5 Module-interface correction

`phalcom-modules::DeclarationSurface { is_const: bool }` currently uses `const` as a value-namespace immutability flag. Under C4 that bit is insufficient to describe source semantics.

P1 should determine what downstream module/linker consumers actually require and migrate to an explicit declaration binding class, conceptually:

```rust
pub enum ModuleBindingSemantics {
    RuntimeImmutable,
    RuntimeMutable,
    StaticConstant,
}
```

If linker consumers only care about assignment mutability, retain a compact representation but **do not call a phase fact `is_const`**. P8 can attach the exported constant value/product; P1 merely prevents interface identity from lying about phase.

## 9.6 Parser work

1. Add `Token::Mut` and keyword lexing.
2. Parse `let mut pattern ...` only in binding position.
3. Reject stray `mut` where no mutable declaration form exists.
4. Rework field declaration grammar from:

   ```text
   _field       mutable
   const _field immutable
   ```

   to:

   ```text
   _field       immutable
   mut _field   mutable
   ```

5. Preserve `FieldDef.mutable` as the sole post-parse mutability fact.
6. Update parser recovery so `let mut`, `mut _field`, and malformed `const` initializers produce targeted diagnostics rather than cascades.
7. Ensure `mut` remains available in the exact contexts intended; do not make every identifier named `mut` parse as a keyword-like property name unless the language's keyword access rules already do so.

## 9.7 Semantic checker work

Audit and replace all assumptions around:

```text
BindingKind::Let
BindingKind::Const
is_mutable
ConstWithoutInitializer
AssignToImmutable
same-scope redeclaration
closure capture mutability
loop-pattern binding mutability
destructuring binding flow
```

Required laws:

- plain `let` cannot be reassigned;
- `let mut` can be reassigned subject to ordinary type/flow rules;
- `const` cannot participate in runtime flow as a local initialized by emitted code;
- destructured names inherit the declaration's phase/mutability consistently;
- closure capture of a `let` is immutable; capture of `let mut` preserves existing mutable-capture semantics;
- P1 may represent a `const` initializer as “pending static evaluation” rather than validating it yet, but it must not compile it as runtime initialization.

## 9.8 Field lifecycle work

The compiler already tracks constructor field validity and immutable-write restrictions. Reuse it rather than inventing a second C4 field-state machine.

Audit:

- explicit constructors;
- `@construct` generated constructors;
- default field expressions;
- superclass constructor delegation;
- inherited fields;
- class-side/static fields;
- generated setters / `@set`;
- delegated accessors;
- reads before initialization;
- implicit-field fallback if retained.

The new default flips almost every declared field. A field syntax migration that compiles but causes generated constructors/setters to fail is a P1 failure.

## 9.9 Source codemod

Behavior-preserving binding rewrite:

```text
old const  → new let
old let    → new let mut
```

but never via naive two-pass textual replacement. Use token-aware or parser-assisted rewrite:

```text
old const token → sentinel or structured edit
old let token   → let mut
sentinel        → let
```

Field rewrite is context-sensitive:

```text
old const _field → _field
old bare _field  → mut _field
```

Only class-field declaration nodes may receive the second rewrite; product fields, parameters, record fields, and ordinary identifiers beginning with `_` must not.

Generated source in compiler tests and macro/attribute expansion helpers must be migrated too. For example, compiler-generated scratch locals must use runtime `let`/`let mut`, never `const` merely because the generated value is not reassigned.

## 9.10 Diagnostics

P1 should establish stable families:

```text
binding.initializer_required
binding.immutable_assignment
binding.redeclared
binding.invalid_mut_modifier
field.mutable_required
field.immutable_reassignment
field.initialization_required
field.read_before_initialization
```

Do not overload old “const assignment” wording after `const` changes phase.

## 9.11 Tests

Minimum matrix:

- parser token/snapshot tests for `mut`;
- `let` immutable local assignment rejection;
- `let mut` assignment success;
- `const` parsed distinctly and never treated as a runtime-mutable/immutable local variant;
- destructuring `let` / `let mut`;
- closure capture and mutation;
- loop binders and shadowing;
- top-level runtime immutable/mutable bindings;
- module import/export namespace semantics;
- REPL snippets;
- declared fields, field defaults, constructor initialization, derived constructors, inheritance, setters, delegation;
- implicit-field policy exact regression tests;
- corpus parse and compile after codemod;
- generated compiler source paths.

## 9.12 Performance requirements

P1 should be performance-neutral. In particular:

- no new runtime wrapper for immutable bindings;
- no generic mutability map lookup on every local load/store if current locals use indexed slots;
- no extra constructor field-state work at runtime when semantic/lowering proof can retain existing efficient bytecodes;
- no global source rescans in LSP solely to recognize `mut`.

## 9.13 Exit criteria

P1 is complete only when:

1. the entire source corpus uses the new spelling;
2. no production code equates `BindingKind::Let` with mutability;
3. all runtime assignment checks key off normalized mutability;
4. `const` no longer emits ordinary runtime initializer semantics;
5. declared fields are immutable by default and `mut` is the only declared mutable spelling;
6. constructor/lifecycle tests prove the chosen immutable initialization rule;
7. module/interface naming no longer encodes obsolete `is_const == immutable runtime binding` semantics;
8. editor/REPL keyword surfaces recognize `mut`;
9. workspace tests remain green before P2 adds static evaluation.

---

# 10. C4.P2 — Static value model, static definitions, evaluator foundation, and closed `const`

## 10.1 Objective

Introduce the semantic execution substrate that makes C4 static computation possible **without** implementing recursion, structural loops, or residualization yet.

P2 should make these forms real:

```telos
const answer = 40 + 2

const cfg = {
    enabled: true,
    retries: 3,
}

type IsZero<N: Int>: Bool =
    N == 0

type PairType<A: Type, B: Type> =
    (A, B)
```

And parse block-bodied static definitions so later plans can extend their statement set:

```telos
type Example<T>: Int {
    let mut n = 1
    n
}
```

At P2 exit, blocks need local binding/assignment/trailing result and ordinary `if`; P6 adds `match`/`for`/`while` structural control and richer control transfer. P7 adds recursion and symbolic residuals.

## 10.2 Static value arena

Do **not** represent every constant as `TypeData`. Add a semantic value store, conceptually:

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum StaticValueData {
    Unit,
    Bool(bool),
    Int(CanonicalInt),
    Float(CanonicalFloat),
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

The exact split among `StaticTerm`, `Data`, and `Variant` should reuse C3 static-algebraic identity where it fits. Required properties:

- no dependency on `phalcom-core::Value`;
- no `ObjRef`;
- deterministic equality/hash;
- canonical or canonicalizable final immutable values;
- compact sharing for strings/products/data payloads;
- stable logical encoding for P8 metadata;
- direct embedding/reference of canonical `TypeId` terms where appropriate;
- no observable arena index identity.

### 10.2.1 Float canonicalization

The store must define equality/hash for floating values consistently with Telos semantic equality. Do not derive raw Rust `Eq/Hash` over `f64` bits and accidentally make NaN/signed-zero behavior diverge from language semantics. A `CanonicalFloat` wrapper or normalized semantic representation is preferable.

### 10.2.2 Strings and symbols

Static strings should be content-addressed or otherwise content-equal. Static symbols need a stable semantic spelling/identity bridge distinct from VM `Symbol(u32)`. P8 serializes spellings/stable IDs; compiler materialization interns them into the destination VM only at lowering/materialization time.

## 10.3 Static result domains

Current `TypeAliasDef.result_kind: Option<KindSyntax>` and `TypeMemberDomain::Kind(KindId)` are too narrow.

Introduce a generalized notion of static result requirement that can express:

```text
Type
Bool
Int
Symbol
Variant
Variants
ProductShape
any declared static data domain
```

Architecturally this should build on C3's static-domain registry rather than invent a parallel kind universe. A possible semantic shape is:

```rust
pub enum StaticResultDomain {
    Kind(KindId),          // e.g. Type or other kind-valued result
    Domain(TypeId),        // proper/static data domain value type as appropriate
    StaticDomain(DeclarationId),
}
```

Do not freeze this illustrative enum literally if C3 already offers a more canonical generalized argument/domain representation. The invariant is that result checking must be **domain-based, not hard-coded per primitive**.

## 10.4 AST changes for `type` definitions

Generalize type alias bodies:

```rust
pub enum StaticDefinitionBody {
    Expression(Expr),
    Block(StaticBlockSyntax), // preferably reuse ordinary Block/Statement syntax
}
```

Prefer reusing ordinary Telos `Expr`/`Statement` AST nodes rather than creating `StaticExpr` and `StaticStatement` duplicates. If type annotation syntax currently contains forms that expression syntax cannot represent, adapt parsing/lowering at the static-definition boundary while retaining one semantic expression model.

Required source grammar:

```telos
type F<T> = expression

type F<T>: Int = expression

type F<T>: Int {
    statements
    trailingResult
}
```

**No `=` before a block body.**

The historical meaning of non-parameterized `type Alias = TypeExpression` must continue to work and should normalize through the same definition machinery with default result domain `Type`.

## 10.5 Static definition semantic product

Do not keep executable static definitions only as AST pointers. Produce a semantic product with stable definition identity and checked body/reference information, conceptually:

```rust
pub struct StaticDefinition {
    pub declaration: DeclarationId,
    pub generic_signature: GenericSignature,
    pub result_domain: StaticResultDomain,
    pub body: CheckedStaticBody,
    pub dependencies: Box<[StaticDependency]>,
    pub fingerprint: ProductFingerprint,
    pub source: SemanticSourceSpan,
}
```

`CheckedStaticBody` may retain expression IDs/semantic products rather than clone AST. P8 determines durable export encoding; P2 only needs query-stable in-process identity.

## 10.6 Explicit evaluator machine

Use an explicit evaluation stack rather than relying on arbitrary Rust recursion:

```rust
pub enum EvalControl {
    Value(StaticValueId),
    Return(StaticValueId),
    Break,
    Continue,
}
```

P2 can support only `Value`/`Return` paths if loops are not yet enabled, but the frame model should reserve the complete control protocol now so P6 does not replace the engine.

Logical modules:

```text
phalcom-semantic/src/static_eval/
    mod.rs
    value.rs
    context.rs
    frame.rs
    eval.rs
    control.rs
    normalize.rs
    diagnostic.rs
    materialize.rs      # semantic descriptor only; P8 completes compiler bridge
```

Later P6/P7 add or expand:

```text
    pattern.rs
    reflect.rs
    memo.rs
    residual.rs
```

## 10.7 Static environment and local mutation

Evaluator frames should contain typed/static local slots:

```text
immutable local: initialized once
mutable local: evaluator slot updated in place
```

Do not intern every intermediate assignment. Intern/canonicalize final immutable values and reusable structural nodes; keep evaluator-local transient state cheap.

References resolve to one of:

- evaluator local;
- generic/static parameter;
- static `const` binding;
- Type/static-domain declaration/value;
- compiler-provided intrinsic static operation;
- forbidden runtime binding.

A runtime `let` referenced by a `const` must fail even if its optimizer-known value is a literal.

## 10.8 Primitive static operations

P2 should support a deliberately closed table of language-defined semantic operations:

- literal construction;
- arithmetic on supported numeric static values;
- comparisons;
- boolean `and`/`or`/`not` with short-circuiting;
- semantic equality where phase-safe;
- tuple/record construction for currently representable key forms;
- transparent/static data construction where semantic declaration products prove the constructor;
- static projection/index where deterministically defined;
- `if` and block sequencing;
- Type/static-domain application that C3 already normalizes without runtime calls.

Crucially, an AST `MethodCall` or `UnqualifiedCall` is **not** automatically static because a runtime method happens to implement `+`, indexing, `toString`, etc. Static operations must be selected by semantic operation identity.

## 10.9 Forbidden operations

Reject at semantic evaluation boundary:

```text
runtime method/function execution
native primitive calls not explicitly exposed as static semantics
I/O
filesystem/network/process
clock/randomness
fibers/scheduler/thread state
runtime object identity
mutable runtime globals
runtime allocation identity
arbitrary user toString/interpolation
```

An unsupported operation in an untaken `if` branch must not fail a closed constant if the branch is semantically not evaluated.

## 10.10 `const` query and outcome model

Add a static-evaluation outcome model now even though P7 will add meaningful symbolic residuals:

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

At P2:

- closed `const` **must** end in `Reduced`;
- `Symbolic` for a closed const is `const.symbolic_result`;
- recursion may simply be rejected/recognized as unsupported until P7, but the outcome must not collapse into generic invalidity;
- `BudgetExceeded` and `Cancelled` are preserved separately.

Add a semantic query/product for constants, e.g. `QueryKey::ConstantBinding(StableBindingId)`, if the DB's key identity can already support it cleanly. P8 completes persistence/export.

## 10.11 Budget and cancellation integration

Every evaluator entry consumes the existing `QueryBudget` / `CancellationToken` authority.

Charge at least:

- each expression evaluation;
- nontrivial static operation;
- static construction proportional to element count;
- definition application;
- block statement execution.

Do not use a hidden independent `MAX_CONST_STEPS`. If a value-growth budget is needed, add a named dimension/reportable counter rather than silently reusing type-depth.

Cancellation should be checked at bounded intervals in long straight-line builders as well as at frame transitions.

## 10.12 Constant materialization descriptor

P2 should define the semantic side of runtime materialization without teaching `phalcom-semantic` about VM objects:

```rust
pub enum ConstantMaterialization {
    Immediate(...),
    String(...),
    Symbol(...),
    Pack(...),
    Tuple(...),
    Record(...),
    Data { declaration: ..., arguments: ..., fields: ... },
    Variant { variant: VariantId, payload: ... },
}
```

This is conceptual. Prefer a DAG/indexed pool to recursive heap-heavy trees for large constants. P8 lowers it into chunk constants / product builders / static runtime descriptors.

## 10.13 Diagnostics

```text
const.non_static_dependency
const.static_evaluation_failed
const.symbolic_result
const.result_not_materializable

static.invalid_operation
static.runtime_call_not_allowed
static.runtime_binding_not_allowed
static.identity_not_available
static.budget_exceeded
static.cancelled             # normally internal/editor suppression, not user-facing
static.result_domain_mismatch

type.static_body_invalid
type.result_domain_mismatch
```

Diagnostics should include the static definition/constant call chain and point to the runtime dependency or operation that made evaluation illegal.

## 10.14 Tests

P2 unit/integration cases:

- all primitive literal static values;
- arithmetic and comparison;
- short-circuit prevents evaluation of forbidden RHS;
- `if` evaluates only selected branch;
- local immutable and mutable evaluator slots;
- block trailing result;
- runtime binding dependency rejection;
- runtime method/native call rejection;
- Type-valued alias compatibility;
- non-Type result domains (`Bool`, `Int`, `Symbol` where available);
- tuple/record/static data construction;
- deterministic equality/hash;
- String/Float edge behavior according to O6/O16 decisions;
- cancellation;
- budget exhaustion;
- closed const symbolic rejection;
- no compiler/VM execution during semantic tests.

## 10.15 Performance requirements

- static values must be arena/interned or otherwise compact, not deeply cloned per operation;
- evaluator frames should be indexed vectors/small maps, not nested `HashMap<String, _>` lookups on every expression;
- no TypeStore clone per static definition application;
- no AST clone per evaluation;
- constant DAG sharing for repeated subvalues;
- cancellation/budget accounting should be branch-predictable and not dominate trivial aliases.

## 10.16 Exit criteria

P2 exits when:

1. `const` values are evaluated in `phalcom-semantic`, not emitted as runtime initializer code;
2. static values are independent of VM object identity;
3. expression- and block-bodied static `type` definitions parse and check;
4. non-Type static result domains work;
5. a closed set of deterministic primitive operations executes;
6. runtime calls/dependencies fail explicitly;
7. budgets/cancellation are integrated;
8. compiler consumers receive a semantic materialization descriptor, not AST to re-evaluate;
9. no const functions/methods exist;
10. the evaluator architecture can be extended by P6/P7 without replacement.

---

# 11. C4.P3 — First-class Pack and call/rest convergence

## 11.1 Objective

Make `Pack` a real immutable ordered two-lane value/type while preserving the compiler's existing no-allocation fixed-call path.

Canonical public semantics:

```telos
intrinsic data Pack<***Es>(***Es)

type PositionalPack<T: Type> = Pack<*: T>
type LabeledPack<T: Type> = Pack<**: T>
type CompletePack<T: Type> = Pack<***: T>

type PositionalTypePack = PositionalPack<Type>
type LabeledTypePack = LabeledPack<Type>
type CompleteTypePack = CompletePack<Type>
```

`Pack()` is distinct from `()`/Unit and `{}`.

## 11.2 Non-negotiable representation law

There are three different concepts that must not be collapsed:

```text
semantic argument Pack
runtime materialized Pack value
VM-private transient argument builder/window
```

A call semantically has an argument Pack; an ordinary fixed call should still compile to direct argument passing with **zero Pack allocation**.

## 11.3 Public source model and compatibility

Introduce the canonical Pack declarations and aliases in Universe source. Internal optimized domains may remain:

```rust
TypeData::TypePack
TypeData::LabeledTypePack
TypeData::CompleteTypePack
TypeData::TypePackConcat
TypeData::LabeledTypePackConcat
```

but public source reflection and aliases should resolve as Pack specializations.

P3 must explicitly map:

```text
PositionalPack<Type>  ↔ internal TypePack
LabeledPack<Type>     ↔ internal LabeledTypePack
CompletePack<Type>    ↔ internal CompleteTypePack
```

without forcing eager conversion or duplicating equality semantics.

The compatibility strategy for old `intrinsic type data TypePack`-style facades is O15/P9, but P3 must prevent **new** public APIs from depending on them as conceptually independent domains.

## 11.4 Static Pack representation

Use flat canonical storage:

```rust
pub struct StaticPackData {
    pub positional: Box<[StaticValueId]>,
    pub labels: Box<[StableSymbol]>,
    pub labeled_values: Box<[StaticValueId]>,
}
```

or an equivalent interning-friendly layout. Requirements:

- lane separation is structural;
- order within each lane is structural;
- labels are `Symbol` only;
- duplicate labels invalid;
- O(1) lane lengths;
- efficient whole-Pack hash/equality;
- slice/range views can share backing storage.

For evaluator append-heavy source algorithms such as:

```telos
let mut result = Pack()
for T in P {
    result = Pack(*result, T)
}
```

implement builder fusion/copy avoidance. A naive immutable copy on every append yields O(n²) compile-time behavior and would make the showcased source-defined algorithms pathological.

## 11.5 Runtime Pack representation

Preferred approach: reuse the existing product layout/storage machinery with a Pack-specific anonymous/product descriptor or dedicated heap object carrying the same substrate.

Requirements:

- runtime nominal identity remains Pack, not Tuple;
- layout uses compact labels + values;
- GC tracing comes from `ProductStorage` layout maps;
- equality/hash uses Pack semantics, not pointer identity;
- materialized Pack supports positional/labeled lane projection efficiently;
- empty Pack has a canonical representation if useful, but must not canonicalize to Unit.

Do not introduce recursive cons objects solely to implement `::View`.

## 11.6 Pack views

Canonical source:

```telos
intrinsic data PositionalPackView<T> {
    Empty
    Entry(_ value: T, _ rest: PositionalPack<T>)
}

intrinsic data LabeledPackView<T> {
    Empty
    Entry(_ label: Symbol, _ value: T, _ rest: LabeledPack<T>)
}
```

Implement `rest` as a semantic/runtime slice wherever possible. View construction should be O(1) and should not copy the remaining elements.

The implementation should fix stale design references to `Item`, `TypePackView`, `Positionals`, and `Labeled` at the same time; canonical names are `Entry`, `PositionalPackView`, `LabeledPackView`, `PositionalEntries`, `LabeledEntries`.

## 11.7 Call builder convergence

Keep `ArgumentPackBuilderObject` VM-private. It is a mutable staging object, not the language Pack value.

Add a finalization path conceptually:

```text
ArgumentPackBuilderObject
    ├─ finish invocation window / InvokePack
    └─ finish immutable Pack value
```

The builder currently performs duplicate label detection with a linear `contains` scan. For potentially large dynamic Packs, avoid O(n²) assembly. Options:

- maintain a temporary `HashSet<Symbol>` only beyond a small inline threshold;
- sort/check only when finalization semantics permit (order must still be preserved separately);
- use a small-vector + hash spill strategy.

Do not penalize common 0–4 label calls with an always-allocated hash table.

## 11.8 Rest capture migration

C4 target:

```text
*rest   → Positional Pack
**rest  → Labeled Pack
***rest → Complete Pack
empty   → Pack()
```

Audit every path that currently finalizes rest capture through Tuple, including `FinishTuplePack` and helpers around `finish_tuple`.

This migration spans:

- formal parameter binding;
- dynamic invoke pack assembly;
- forwarding;
- closure invocation if it has independent binder logic;
- native/runtime call boundaries consuming rest;
- pattern/rest capture if the same runtime helper is shared.

Do not make Record implicitly acceptable as `**args`. Record remains an unordered product; labeled Pack is ordered and Symbol-labeled. Explicit conversion may be introduced separately if desired.

## 11.9 Pack expansion and exact shape preservation

Pack expansion must preserve lane/order/labels exactly:

```telos
type Types = Pack<Int, Int, String, String>
const elements: Types = ...
ConstructData(*elements)
```

For labeled shape constraints, remember the language rule discussed during C4 design: `**args: T` means each labeled argument's value has type `T`; constraining the entire labeled shape requires a pack/row form such as `**args: **ConfigArgs` or the final grammar-equivalent. P3 parser/type relation tests must ensure these are not conflated.

## 11.10 Type relations

Reuse existing pack canonicalization, concatenation, substitution, and lane-ownership rules from C1–C3. P3 should add the public Pack semantics **over** them, not replace proven solvers.

Required cases:

- exact fixed Pack equality;
- lane-sensitive inequality;
- label-sensitive equality;
- constrained homogeneous Pack families;
- complete Pack decomposition;
- concatenation/expansion;
- duplicate label invalidity after specialization;
- open typed/unconstrained/captured ordered tails where already represented internally;
- substitution and generic specialization.

## 11.11 Compiler/lowering work

Certify two paths:

### Fixed call

```text
arguments already statically shaped
→ normal direct/specialized invoke
→ no NewArgumentPack
→ no immutable Pack heap allocation
```

### Dynamic/expanded call

```text
computed/expanded argument shape
→ VM-private builder
→ InvokePack or equivalent
→ only materialize language Pack if program semantics require a first-class value/rest capture
```

Add bytecode/compiler assertions so future refactors cannot silently regress fixed calls into Pack allocation.

## 11.12 Tests

- `Pack()` distinct from Unit and empty Record;
- positional only, labeled only, complete Packs;
- duplicate labels;
- equality/hash and lane ordering;
- nested Packs;
- `::View` O(1)-semantics behavior;
- type Pack aliases equal public Pack specializations;
- expansion and forwarding;
- all three rest modes;
- zero-rest result is Pack;
- Pack rest passed onward without Tuple conversion;
- dynamic computed label path;
- fixed call bytecode contains no builder/Pack allocation;
- product scalar-replacement can eliminate temporary Pack where legal;
- large append/filter static Pack algorithms remain near-linear.

## 11.13 Performance gates

Measure before/after:

- ordinary fixed call throughput and allocations;
- fixed generic specialized call;
- one expanded positional call;
- one expanded labeled call;
- rest-heavy forwarding;
- construction/view of 10/100/1000-entry static Packs;
- builder duplicate-label behavior for many labels;
- static filter/map Pack algorithm.

A material regression on ordinary fixed calls blocks P3 completion.

## 11.14 Exit criteria

P3 exits when Pack is first-class in source, semantic static values, type relations, runtime representation, views, and rest capture while ordinary fixed calls remain allocation-free with respect to Pack.

---

# 12. C4.P4 — Generalized FieldKey and unordered-product calculus

## 12.1 Objective

Generalize unordered products from string/Symbol-named rows to stable `FieldKey` keys:

```telos
type FieldKey =
    Symbol | String | Int | Bool
```

and establish the complete reflected shape/tail/lookup calculus:

```telos
Presence
FieldInfo
ProductField
FieldLookup
UnorderedTail
UnorderedShape
UnorderedShapeView
```

This plan deliberately excludes Product/Sum capability derivation, which is P5.

## 12.2 Canonical FieldKey representation

Implement a semantic key enum independent of VM identity:

```rust
pub enum FieldKey {
    Symbol(StableSymbol),
    String(StaticStringIdOrValue),
    Int(CanonicalInt),
    Bool(bool),
}
```

Required traits/operations:

- semantic equality;
- deterministic hash;
- total canonical order;
- stable metadata encode/decode;
- display/diagnostic formatting;
- static-value conversion;
- runtime conversion/materialization.

### 12.2.1 Canonical order

O4 must be ratified before fingerprints/metadata are frozen. Recommended form:

```text
kind tag order: Symbol < String < Int < Bool   # illustrative; choose and document
within Symbol: stable spelling/semantic symbol order
within String: Unicode/UTF-8 code-unit or defined scalar-value order
within Int: mathematical integer order
within Bool: false < true
```

The exact kind ordering is less important than being language-defined and version-stable. It must not depend on:

- Rust enum discriminant accident;
- `HashMap` iteration;
- VM interner numeric IDs;
- source field order.

## 12.3 Semantic row migration

Current `RecordRowField { name: Box<str>, ... }` becomes keyed by canonical `FieldKey`.

Audit all code using:

```text
field.name
binary_search_by(|field| field.name...)
BTreeMap<String, ...>
HashMap<Box<str>, ...>
StableRecordRowLack { field/name ... }
row solver keys
record pattern field names
metadata RecordFieldRef
fingerprint sorting
formatting
```

Preserve the current optionality correction: optional-source cannot satisfy a required-target merely because the value types match.

## 12.4 Row inference and lacks constraints

Generalizing keys is not a mechanical type change. Row inference currently creates constraints keyed by strings. Update:

- row variable lacks sets;
- duplicate detection;
- unification/extension;
- open-tail solving;
- substitution;
- occurs/cycle checks;
- rigidification/generalization;
- diagnostic witnesses.

A computed static key that reduces to the same `FieldKey` as another field must create a duplicate/collision diagnostic **after specialization**, never last-write-wins behavior.

## 12.5 Type syntax and key expressions

Ordered product labels remain Symbol. Unordered product fields may use general key syntax.

Parser/AST should distinguish the contexts even if they share some `ProductLabel` syntax today. Required forms should include canonical static keys, likely:

```telos
{
    name: String,          // #name Symbol key
    ["mime"]: String,
    [42]: Handler,
    [true]: Enabled,
}
```

and tail forms:

```telos
{
    id: Int,
    **[String]: Handler,
}
```

The exact bracket grammar belongs to O8 where structural patterns overlap, but P4 must ensure expression/type field key syntax can represent all four kinds.

## 12.6 Static versus runtime computed keys

Define three cases clearly:

1. **compile-time key expression reduces to concrete FieldKey** → fixed field;
2. **generic/static key remains symbolic but domain-constrained** → residual/captured shape where semantic identity can preserve correlation;
3. **runtime key expression only known by key domain** → cannot become a fixed static field; model conservatively as an open typed/unconstrained tail or reject in a context requiring a closed structural type.

Do not treat an arbitrary runtime String expression as if it produced one statically named field.

## 12.7 Unordered tails

Canonical public model:

```telos
intrinsic data UnorderedTail {
    Closed
    Typed(_ keyType: Type, _ valueType: Type)
    Unconstrained(_ keyType: Type)
    Captured(_ rest: UnorderedShape)
}
```

Map existing record-row remainder concepts onto this public model without losing solver correlation.

Laws:

- `Closed` contributes no unknown fields;
- `Typed(K,V)` permits zero or more additional fields whose keys conform to `K` and values conform to `V`;
- `Unconstrained(K)` constrains key domain but not a common value type;
- `Captured(rest)` preserves exact/correlated remainder identity;
- anonymous open tails are **not iterable finite collections**;
- fixed fields remain iterable even when a tail is open.

## 12.8 Lookup semantics

Implement one canonical unordered lookup returning:

```telos
FieldLookup::Required(Type)
FieldLookup::Optional(Type)
FieldLookup::Absent
FieldLookup::Unknown
```

Lookup must account for fixed field presence and tails. Examples:

- fixed required key → Required;
- fixed optional key → Optional;
- absent fixed key + Closed tail → Absent;
- absent fixed key + compatible open tail → Unknown or a more precise Optional-like result only if semantics proves a value type; canonical source currently selects `Unknown` for insufficient fixed knowledge;
- incompatible key domain in typed tail → Absent when provable.

Keep this lookup primitive shared with structural relation/matching code.

## 12.9 Runtime Record shape migration

This is a major obligation revealed by repository audit. `RecordProductShape` currently stores `Symbol` arrays and a `HashMap<Symbol,u32>` lookup cache. Generalize it to runtime FieldKey coordinates.

Potential architecture:

```rust
pub enum RuntimeFieldKey {
    Symbol(Symbol),
    String(ObjRefOrInternedStringHandle),
    Int(CanonicalRuntimeIntKey),
    Bool(bool),
}
```

However, avoid making runtime shape identity depend on heap object addresses. Strings used as structural keys need a stable/interned key representation suitable for hashing and GC ownership. A VM-owned string-key interner or immutable key pool may be more appropriate than raw String objects.

The runtime shape registry should retain:

```text
presentation keys
canonical logical keys
presentation → logical permutation
lookup cache
```

with canonical logical order defined by the same FieldKey ordering as semantic reflection.

## 12.10 Record value semantics and hashing

Record identity remains unordered structurally. Two Records with identical FieldKey→value mappings but different source/presentation order should remain structurally equal and may share logical layout while preserving presentation if the language exposes it separately.

Do not accidentally make new key kinds order-sensitive because they enter `ProductShape` arrays.

## 12.11 Structural relation convergence

Refactor shared helpers so these all use the same key/tail logic:

- record/unordered subtyping;
- argument acceptance where an unordered shape is relevant;
- record patterns;
- static field lookup;
- width typing;
- optional presence;
- tail constraints;
- captured remainder compatibility.

Avoid a P4-only “reflection lookup” whose optional/tail semantics drift from subtyping.

## 12.12 Metadata preparation

P4 should define stable metadata nodes but P8 performs the final schema release. Required durable key encoding:

```rust
enum FieldKeyRef {
    Symbol(StableSymbolRef),
    String(Box<str>),
    Int(CanonicalIntRef),
    Bool(bool),
}
```

No process-local symbol IDs or runtime object references.

## 12.13 Tests

- each FieldKey kind in closed Records/types;
- mixed-key record canonical order;
- source reorder equality/subtyping/fingerprint equality;
- duplicate same-kind keys;
- duplicates arising after generic/static specialization;
- Symbol `#"x"` vs String `"x"` remain distinct keys;
- Bool vs Int must remain distinct even if implementation languages equate true/1 in some contexts;
- optional required/optional relation matrix;
- width subtyping;
- typed/unconstrained/captured tails;
- lookup Required/Optional/Absent/Unknown;
- row inference/lacks with non-Symbol keys;
- record patterns;
- runtime construction/access/hash/equality;
- metadata round-trip staging tests;
- deterministic iteration/reflection order.

## 12.14 Performance gates

- canonical fixed fields sorted once at interning/formation time;
- O(log n) binary search or O(1) cached lookup for fixed rows;
- avoid repeated String allocation during key comparison;
- row solver key hashing should be cheap and canonical;
- runtime shape lookup cache should support all FieldKey kinds without linear scans;
- large 1k/10k-field synthetic shape formation benchmark to catch accidental O(n²) duplicate/order logic.

## 12.15 Exit criteria

P4 exits when generalized keys work through syntax, static formation, row inference, relations, runtime Record shape/storage lookup, reflection-ready shape construction, deterministic ordering, and metadata staging, with optional/tail semantics preserved.

---
# 13. C4.P5 — Product/Sum reflection and compiler-derived structural capabilities

## 13.1 Objective

Expose structural reflection as ordinary static Telos data while deriving the fact that a declaration is a Product or Sum from compiler-owned declaration semantics.

Canonical public ontology:

```text
data
├── Product
│   ├── OrderedProduct
│   └── UnorderedProduct
│
└── Sum
    └── Variants = PositionalPack<Variant>
        └── Variant(name: Symbol, shape: ProductShape)
```

This plan does **not** add a runtime “reflection object graph.” Reflection results are static immutable data formed from canonical semantic products.

## 13.2 Product reflection source surface

Canonical source should converge on:

```telos
intrinsic data ProductKind {
    Ordered
    Unordered
}

intrinsic data ProductShape {
    Ordered(_ shape: OrderedShape)
    Unordered(_ shape: UnorderedShape)
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

The traits are independent. Do not add trait inheritance merely to encode implementation derivation.

## 13.3 Product declaration derivation

Use the existing `DataInfo` / `DataShape::{Tuple,Record}` / `DataProductShapeTemplate` authorities.

For an ordered product data declaration:

```text
compiler fact:
    Product
    OrderedProduct

associated static results:
    Kind = ProductKind::Ordered
    Shape = ProductShape::Ordered(...)
    OrderedShape = ...
    Entries / PositionalEntries / LabeledEntries
```

For an unordered product data declaration:

```text
compiler fact:
    Product
    UnorderedProduct

associated static results:
    Kind = ProductKind::Unordered
    Shape = ProductShape::Unordered(...)
    UnorderedShape = ...
```

This applies equally to:

- intrinsic `Pack`;
- intrinsic `Tuple`;
- intrinsic `Record`;
- user-authored product `data` declarations;
- generic product data after specialization.

Nominal type identity remains separate from reflected payload shape.

## 13.4 Unforgeable capability mechanism

O11 is a hard design decision. The source traits exist so generic code can state constraints, but ordinary user code must not be able to write a lying implementation such as:

```telos
impl Product for SomeNonProduct { ... } // must not manufacture compiler structural facts
```

Repository options to investigate/choose deliberately:

### Preferred: compiler-derived sealed structural capability

Treat these traits as compiler-known derived capabilities whose conformance evidence is synthesized only from canonical `data` declaration shape. Their associated results are computed from semantic tables, not arbitrary source bodies.

This can reuse the repository's conformance/evidence infrastructure for consumption while marking the producing authority as compiler-derived/sealed.

### Acceptable alternative: intrinsic associated surface without ordinary conformance

If the trait solver cannot cleanly distinguish compiler-derived evidence, expose the constraints through a compiler-owned capability class/special conformance path while preserving the source spelling. The critical invariant is that user-authored `impl` cannot enter the same evidence bucket.

### Rejected

- name-based “if trait is called Product, trust it” logic;
- ordinary user impls plus runtime validation;
- checking whether associated `Shape` happens to look plausible;
- a second reflection-only nominal hierarchy disconnected from trait constraints.

## 13.5 Generalize type-member/static-member result domains

Current `TypeMemberDomain::Kind(KindId)` cannot represent:

```text
T::Shape : ProductShape
T::Kind : ProductKind
T::Variants : Variants
P::View : PositionalPackView<T>
shape::Lookup<K> : FieldLookup
```

Generalize associated intrinsic/static member declaration and normalization to use the same static result-domain machinery introduced in P2.

Do not implement each C4 member as a special “returns TypeId” escape hatch. The associated-member system needs to transport `StaticValueId`/semantic static terms in addition to proper types.

## 13.6 OrderedShape formation

Form ordered reflection from canonical semantic pack/type-shape facts:

```telos
intrinsic data OrderedShape(
    _ entries: CompleteTypePack,
    _ positionalTail: PositionalTail,
    _ labeledTail: LabeledTail,
)
```

Rules:

- fixed entries preserve lane order and labels;
- typed/unconstrained/captured tails remain explicit;
- a user-defined ordered data declaration can have the same `OrderedShape` as a Tuple or Pack without becoming the same type;
- declaration identity is never encoded into `OrderedShape` merely to preserve nominality.

## 13.7 UnorderedShape reflection integration

Consume P4's canonical `UnorderedShape` rather than reconstructing fields from source syntax.

`T::UnorderedShape` must reflect the specialized semantic row, including:

- fixed FieldKeys;
- field Type values;
- required/optional presence;
- typed/unconstrained/captured tail.

Source field ordering is not reflection ordering. Reflection uses canonical FieldKey order.

## 13.8 Sum reflection

Canonical source:

```telos
intrinsic data Variant(
    _ name: Symbol,
    _ shape: ProductShape,
)

type Variants = PositionalPack<Variant>

type VariantNames = PositionalPack<Symbol>

trait Sum {
    type Variants: Variants
}
```

Derive `Sum` from enumerated `data` declarations using existing enum semantic authority.

## 13.9 Variant family identity

Existing `VariantId` and `VariantFamilyId` already model the crucial C4 law:

```text
one Symbol may denote multiple structural variants
```

Reflection should build declaration-ordered `Variant` values from exact canonical variants:

```text
VariantFamilyId(#Number)
    ├── VariantId(Number(Int))   → Variant(#Number, OrderedShape(...Int...))
    └── VariantId(Number(Float)) → Variant(#Number, OrderedShape(...Float...))
```

Never collapse by symbol name. `VariantsOf<T,#Number>` is a filter returning zero/one/many entries.

## 13.10 Specialized Sum reflection

For:

```telos
data Result<T, E> {
    Ok(_ value: T)
    Err(_ error: E)
}
```

`Result<T,E>::Variants` may contain symbolic Type values in payload shapes. `Result<Int,String>::Variants` must reflect the specialized shapes with Int/String.

Use the same substitution/materialization authority that constructor/match semantic products already use. Do not re-read source AST and substitute names textually.

## 13.11 Nullary variants

Every Variant carries a `ProductShape`, including nullary variants. Define one canonical nullary payload shape consistent with the declaration's structural form and existing enum constructor semantics. Do not invent “no shape” as a third branch.

## 13.12 GADT variants

P5 must reflect the canonical specialized payload shape of a GADT variant but should **not** expose branch-proof/refinement internals unless O10 is explicitly reopened and accepted.

Branch equalities remain a semantic match proof authority. Reflection can say what product payload a Variant accepts; it need not serialize all proof environments.

## 13.13 Source-defined reflection utilities

The compiler should provide low-level facts; higher algorithms should be ordinary Telos where practical:

```telos
type VariantsNamed<Vs: Variants, Name: Symbol>: Variants {
    let mut result = Pack()

    for variant in Vs {
        match variant {
            Variant(name, _) =>
                if name == Name {
                    result = Pack(*result, variant)
                }
        }
    }

    result
}
```

Then:

```telos
type VariantsOf<T: Sum, Name: Symbol>: Variants =
    VariantsNamed<T::Variants, Name>
```

Likewise `VariantCount`, `HasVariant`, `VariantNamesOf`, Pack map/filter utilities should be source-defined once P6 static control is available.

P5 may bootstrap these as compiler intrinsics only temporarily for vertical tests; P9 should ensure the final surface uses ordinary source where intended.

## 13.14 Reflection value caching

Reflection is deterministic and commonly repeated. Cache canonical reflection per specialized semantic subject:

```text
(TypeId, reflection operation) → StaticValueId
```

or integrate through static member normalization memoization. Avoid recreating large `Variants`/`OrderedShape`/`UnorderedShape` values on each reference.

Cache keys must use canonical specialized TypeIds/arguments, not source text.

## 13.15 Diagnostics

```text
reflection.member_not_applicable
reflection.invalid_structural_capability
reflection.unavailable_shape
product.derived_capability_conflict      # internal/source diagnostic if user attempts forbidden impl
sum.derived_capability_conflict
variant.no_matching_shape
variant.ambiguous_shape
```

User attempts to manually implement sealed/derived structural capabilities should receive a precise diagnostic, not a downstream coherence mystery.

## 13.16 Tests

Product matrix:

- intrinsic Pack/Tuple/Record;
- user ordered data;
- user unordered data;
- generic data specialization;
- expansion/tail-containing product templates;
- same shape/different nominal declarations;
- user cannot forge Product/OrderedProduct/UnorderedProduct;
- associated member result domains return ordinary static data.

Sum matrix:

- ordinary distinct-name enum;
- repeated variant symbol with different ordered shapes;
- repeated symbol with ordered and unordered shapes;
- declaration order preserved;
- generic specialization;
- GADT payload reflection;
- nullary variants;
- user cannot forge Sum;
- `Variants` is ordinary Pack and supports Pack view/iteration once P6 is present.

## 13.17 Performance gates

- reflection formation linear in actual fixed shape size;
- no source AST scanning per reflection query;
- variant family lookup uses existing family index, not O(total variants) name scans for construction/matching;
- cached specialized reflection avoids repeated allocation;
- large 1k-variant family benchmark;
- large product reflection benchmark;
- no effect on ordinary runtime dispatch for code not using reflection.

## 13.18 Exit criteria

P5 exits when every canonical product/sum declaration receives unforgeable compiler-derived capability evidence and all reflected shapes/variants are ordinary static data built from existing semantic authorities, with repeated variant names preserved exactly.

---

# 14. C4.P6 — Static control flow, structural iteration, matching, and proof predicates

## 14.1 Objective

Extend the P2 evaluator from closed primitive expressions/blocks to the full ratified static control subset:

```text
match
for
while
return
break
continue
structural destructuring
reflection iteration
proof predicates
```

while consuming existing match/GADT/structural authorities instead of implementing parallel semantics.

## 14.2 Static match architecture

The required pipeline is:

```text
source Pattern
    ↓ semantic checking once
PatternResolution / MatchResolution / BranchProofEnvironment
    ↓
static evaluator OR runtime lowering
```

Static evaluation must **not** re-resolve variant names, re-run overload family selection from source text, or independently infer GADT equalities.

At evaluation time it should consume exact semantic projections:

```rust
pub fn eval_match(
    scrutinee: StaticValueId,
    resolution: &MatchResolution,
    ...
) -> StaticEvalOutcome
```

The exact API can differ; the key is that source resolution is upstream.

## 14.3 Selected-arm execution

For a reduced scrutinee:

1. traverse checked arms in semantic order;
2. apply the already-resolved pattern projection to the static value;
3. bind static locals/captures;
4. install the arm's proof/refinement environment;
5. evaluate only that arm body;
6. do not evaluate untaken branches.

This is essential for both correctness and compile-time performance.

## 14.4 Structural type patterns

C4 wants direct type matching analogous to runtime nominal destructuring, for example:

```telos
match T {
    TupleOfCallables<First, *Rest> => ...
}
```

and declaration-independent reflection:

```telos
match T::Shape {
    ProductShape::Ordered(OrderedShape(...)) => ...
}
```

O8 must finalize parser spellings for open/captured tails. Implement the smallest grammar that maps cleanly to existing pattern AST/semantic resolution. Avoid a second “type pattern” AST if ordinary patterns over static Type values can carry the semantics.

## 14.5 Pack structural patterns

Support canonical forms such as:

```telos
match P {
    Pack() => ...
    Pack(_head, *rest) => ...
    Pack(debug: DebugType, **_) => ...
    Pack([#*]: T, **rest) => ...
}
```

`#*` inside bracket-key syntax is an actual Symbol label, not expansion punctuation.

Captured rest is a concrete Pack and therefore iterable; anonymous typed/unconstrained tails are shape descriptions and are not.

## 14.6 Static `for`: no runtime iterator protocol

Ordinary runtime `for` lowers through `iterate(_)` / `iteratorValue(_)`. Static structural `for` must not invoke those methods.

For static subjects, semantic checking should classify iteration source into a closed structural iteration mode, conceptually:

```rust
pub enum StaticIterationKind {
    OrderedValuePositionals,
    OrderedValueLabeled,
    OrderedTypePositionals,
    OrderedTypeLabeled,
    UnorderedValueFields,
    UnorderedTypeFields,
    PositionalPackValues,
    LabeledPackEntries,
    // possibly Sum Variants via PositionalPack path, not a separate kind
}
```

Prefer deriving the actual mode from Product/Pack shape facts rather than hard-coded source type names.

## 14.7 Ordered product iteration rules

Static ordered **value**:

```telos
for value in product { ... }
```

iterates positional values only.

```telos
for [label]: value in product { ... }
```

iterates labeled entries only.

Static ordered **type**:

```telos
for EntryType in T { ... }
for [label]: EntryType in T { ... }
```

iterates fixed Type entries in the corresponding lane.

Open anonymous tails are not expanded into imaginary iterations.

## 14.8 Unordered product iteration rules

Static unordered value:

```telos
for [key]: value in recordLikeValue { ... }
```

iterates actual fields in deterministic canonical key order unless the language explicitly chooses presentation order for concrete values. The handoff currently requires canonical deterministic static order; keep that authority.

Static unordered type:

```telos
for [key]: FieldInfo(T, presence) in T { ... }
```

iterates **fixed** fields only, in canonical FieldKey order. An open tail remains as tail metadata and is not iterated.

## 14.9 Heterogeneous binder typing

O5 must be settled here.

Recommended static-evaluator behavior: preserve **per-iteration exact refinement** rather than widening the body binder once to a union/common supertype where possible.

For:

```telos
for value in (1, "x") { ... }
```

conceptually evaluate body twice with:

```text
iteration 0: value : Int, value = 1
iteration 1: value : String, value = "x"
```

This is a static unrolling/refinement model, not a runtime loop ABI. It preserves key/value and label/value correlation for heterogeneous products and enables useful source-defined static algorithms.

Semantic diagnostics inside the body must still be sound across every reachable iteration. If an expression requires `Int`, it cannot be accepted merely because one iteration is Int unless pattern/control refinement excludes the String iteration.

## 14.10 Refutable loop patterns

Loop patterns are destructuring binders, not filters.

If a pattern is refutable for any statically known element, either:

- reject the loop pattern as not irrefutable for the iteration domain; or
- surface the same pattern-mismatch semantics ordinary binding has, if that is the language's chosen loop law.

Do not silently skip nonmatching entries. Users can write explicit `match` to filter.

## 14.11 Static `while`

C4 allows static while, budgeted.

Because historical parser lowering may encode ordinary while through `.whileTrue`, P6 must preserve/introduce explicit semantic loop identity before evaluator execution.

Acceptable implementation:

- parser AST adds/retains `WhileExpr`/`WhileStatement` and runtime compiler later lowers it to the existing send/jump implementation;
- or semantic indexing publishes a dedicated `WhileResolution` keyed to the source construct before desugaring.

Rejected:

```text
if selector == "whileTrue" then treat as static while
```

because a user method with that selector is runtime dispatch and forbidden.

## 14.12 Control transfer

Use explicit evaluator control results:

```rust
Value(v)
Return(v)
Break
Continue
```

Rules:

- `return` exits the current static definition, not the whole compiler query;
- `break` / `continue` are valid only in active static loop frames;
- control state never leaks into cached final values;
- `return` from nested blocks does not become runtime non-local return semantics.

## 14.13 Proof predicates

Finalize O9 and expose compiler relation authority through deterministic static operations, likely:

```telos
Equivalent<A, B>
Subtype<A, B>
Conforms<T, Trait>
```

Implementation rules:

- consume existing relation/conformance engines;
- preserve rich internal outcomes (`Proven`, `Refuted`/NotApplicable, `Unknown`, `Blocked`, `Dynamic`, `Cancelled`, `BudgetExceeded`);
- return `true` only on proof;
- return `false` only on semantic refutation;
- generic uncertainty in a reusable `type` definition becomes symbolic/residual in P7, not false;
- closed `const` requiring a Bool cannot silently coerce Unknown to false;
- relation budgets feed the same parent static query budget/report.

## 14.14 Match proof environment integration

When a GADT/static match arm establishes equalities or exact-case facts, evaluator-local type/static operations in that arm must see the same `BranchProofEnvironment` the ordinary semantic checker used to validate the arm.

Do not rebuild a parallel substitution environment from bound variable names. Use canonical type parameter/variant identities.

## 14.15 Reflection fusion

Avoid materializing large reflection values when the only consumer immediately destructures/iterates them and the optimization is semantics-preserving.

Potential fusions:

```text
P::View → match
T::Variants → for
T::UnorderedShape::View → match
```

This is an optimization only; semantic behavior must be identical to constructing the ordinary immutable reflection value. Implement after correctness, with benchmarks.

## 14.16 Diagnostics

```text
static.match_unresolved
static.iteration_not_structural
static.iteration_open_tail_not_finite
static.refutable_loop_pattern
static.break_outside_loop
static.continue_outside_loop
static.return_outside_definition
static.while_condition_not_bool
static.proof_unknown
static.proof_blocked
static.proof_dynamic
static.proof_budget_exceeded
static.pattern_projection_failed
```

Existing match exhaustiveness/usefulness diagnostics should remain owned by match semantics and should not be duplicated under `static.*` merely because the match is evaluated statically.

## 14.17 Tests

Match:

- simple static enum/data match;
- overloaded variant family exact selection;
- repeated Symbol family;
- GADT branch refinement;
- untaken invalid arm not executed;
- exhaustiveness/usefulness parity with runtime match;
- Pack structural patterns and rest capture;
- ProductShape/UnorderedShape reflection patterns.

For:

- positional/labeled Pack;
- Tuple and user ordered data;
- Record and user unordered data;
- Type-level iteration;
- heterogeneous binder refinement;
- fixed fields only for open unordered tails;
- captured concrete tail iteration;
- refutable loop pattern behavior;
- break/continue/return.

While:

- terminating loop;
- break/continue;
- budget exhaustion;
- ordinary method named `whileTrue` remains forbidden runtime call.

Proofs:

- Proven true;
- Refuted false;
- Unknown/Blocked/Dynamic remain non-Bool terminal/residual;
- budget/cancellation propagation.

## 14.18 Performance gates

- iteration over flat Packs/fields is O(n);
- no recursive cons allocation for views;
- no repeated pattern candidate resolution per evaluator iteration;
- no source AST variant-family scans;
- branch proof installation uses persistent/small environments rather than cloning the entire TypeStore;
- large static for loops remain bounded and cancellation responsive;
- reflection fusion benchmark where valuable.

## 14.19 Exit criteria

P6 exits when static control forms execute through P2's semantic evaluator, all structural matching/variant/GADT decisions are reused from canonical semantic products, structural iteration has deterministic finite semantics, and proof predicates preserve relation uncertainty correctly.

---

# 15. C4.P7 — Recursive generic static computation, symbolic residualization, and memoization

## 15.1 Objective

Make parameterized `type` definitions a practical reusable static programming abstraction rather than only a macro-like closed evaluator entry.

Required capabilities:

- recursive type/static definitions;
- changing-argument recursion;
- exact cycle detection;
- specialization;
- symbolic results when generic inputs prevent reduction;
- memoization;
- normalized residual IR;
- bounded evaluation and cancellation;
- deterministic diagnostics.

## 15.2 Definition application key

Memoization/cycle detection key must include:

```text
stable definition identity
+
canonical static argument identities
+
relevant specialization/proof environment identity
```

Do not key by source text, AST pointer, TypeStore numeric allocation history, or pretty-printed arguments.

Conceptually:

```rust
pub struct StaticApplicationKey {
    definition: DeclarationId,
    arguments: Box<[StaticArgumentIdentity]>,
    environment: StaticSpecializationEnvironmentKey,
}
```

The environment key should include only semantic facts that can change the result. Avoid hashing entire checker contexts.

## 15.3 Recursion semantics

Exact in-progress key repetition indicates a cycle:

```text
F<A> → ... → F<A>
```

Changing arguments may proceed:

```text
Count<Pack<A,B,C>>
→ Count<Pack<B,C>>
→ Count<Pack<C>>
→ Count<Pack()>
```

until base case, cycle, or budget exhaustion.

Do not apply a simplistic “definition appears twice on stack = recursion error.” That would reject the primary use cases.

## 15.4 Host stack safety

Definition calls must push evaluator frames on the explicit P2 machine. Recursive source computation cannot consume unbounded Rust stack.

Cycle traces should record compact application frames and be printable without retaining giant static values unnecessarily.

## 15.5 Residualization requirement

Generic definitions may be semantically valid even when not fully reducible before specialization:

```telos
type IsNumeric<T: Type>: Bool =
    Subtype<T, Number>
```

For an unconstrained `T`, the compiler may need a symbolic result until a later specialization supplies an exact Type.

Introduce a normalized residual IR, not raw AST:

```rust
pub enum StaticResidualData {
    Parameter(StaticParameterId),
    Apply {
        definition: DeclarationId,
        arguments: Box<[StaticTerm]>,
    },
    Intrinsic {
        operation: StaticIntrinsicId,
        arguments: Box<[StaticTerm]>,
    },
    If {
        condition: StaticTerm,
        then_term: StaticTerm,
        else_term: StaticTerm,
    },
    Match { ...canonical semantic pattern refs... },
    Product { ... },
    // only forms genuinely required to preserve unresolved semantics
}
```

This enum is illustrative. Keep the residual language **smaller than the full source AST** and semantic-normalized.

## 15.6 Residual normalization laws

Normalize aggressively but soundly:

- substitute known static arguments;
- fold reduced primitive operations;
- select `if` branch when condition reduced;
- select static match arm when scrutinee reduced;
- eliminate identity Pack concatenations;
- canonicalize product/Pack constructors;
- normalize relation proofs when enough facts become known;
- preserve unresolved branch/control only when semantically required;
- avoid duplicate nested Apply nodes for equivalent applications.

The same specialization of the same residual should yield the same canonical result regardless of cold/incremental analysis history.

## 15.7 Residual versus error

Distinguish:

```text
Symbolic because a generic argument is unknown        → valid residual
Unknown because a semantic engine lacks proof yet      → residual/terminal according to operation semantics
Blocked by unresolved dependency                       → blocked terminal, not arbitrary residual if dependency is erroneous/incomplete
Dynamic boundary                                       → not statically reducible; usually invalid for a static result unless language explicitly permits residual dynamic facts
BudgetExceeded                                         → execution context terminal, not cacheable semantic residual
Cancelled                                              → execution context terminal
```

Do not encode BudgetExceeded or Cancelled into a residual that later looks like a valid program term.

## 15.8 Memoization layers

Use two scopes deliberately:

### Evaluator-local application memo

Cheap and always appropriate for repeated calls during one root evaluation.

```text
StaticApplicationKey → Reduced/Symbolic deterministic semantic result
```

### Semantic DB product cache

Use for definition/constant root products and possibly application results only where cross-query reuse demonstrably improves incremental compilation. Avoid exploding the global DB key space for every tiny recursive call before profiling.

P8 finalizes durable query integration.

## 15.9 Do not cache context-sensitive terminal states

Do not persist as semantic successes:

- Cancelled;
- BudgetExceeded under a particular budget;
- transient Blocked caused by an incomplete dependency snapshot.

Internal deterministic Invalid/Recursive results may be cacheable if their dependencies and diagnostics are stable, but review invalidation semantics carefully.

## 15.10 Budget accounting

P7 adds charges for:

- definition application;
- recursive frame push;
- pattern candidate application if not already charged in P6;
- loop iteration;
- residual node creation proportional to size;
- memo entry growth.

Consider an explicit `static_value_nodes` / `residual_nodes` limit if adversarial generic code can build exponentially large symbolic terms under the existing step budget.

Budget diagnostics must state:

```text
which budget
limit
used
root static definition
application chain
summarized arguments
```

## 15.11 Dependency recording

Every static application records semantic dependencies on:

- referenced constants;
- referenced static definitions;
- type/trait/data/enum declarations whose reflection is used;
- relation/conformance facts;
- associated static members;
- imported metadata products;
- intrinsic semantic operation versions if the DB has a suitable authority key.

Do not record evaluator implementation details as dependencies.

## 15.12 Specialization integration

When generic type/application arguments become exact, normalize residuals through the ordinary specialization path rather than re-parsing/re-checking source from scratch.

Required property:

```text
cold exact application
==
specialize previously symbolic application
```

in result identity, diagnostics, and fingerprint.

## 15.13 Recursive source-defined algorithms to certify

At minimum:

```telos
type Count<*P>: Int =
    match P {
        Pack() => 0
        Pack(_, *rest) => 1 + Count<*rest>
    }
```

and iterative equivalent:

```telos
type Count<*P>: Int {
    let mut count = 0
    for _ in P { count += 1 }
    count
}
```

Both should produce equivalent results with predictable complexity.

Other useful probes:

- `ContainsLabel`;
- `AllSubtype`;
- Pack map/filter;
- recursive UnorderedShape view walk;
- `VariantsNamed`;
- nested ProductShape transform.

## 15.14 Diagnostics

```text
static.recursive
static.cycle
static.budget_exceeded
static.residual_too_large
static.dynamic_dependency
static.blocked_dependency
static.specialization_failed
static.application_domain_mismatch
```

Cycle diagnostic example structure:

```text
Static evaluation is recursive for Count<...>
  Count<Pack<Int>>
  → Count<Pack<Int>>   # exact repeated key
cycle begins here
```

For changing arguments that merely exhaust the budget, report budget exhaustion rather than falsely claiming an exact cycle.

## 15.15 Tests

- direct exact cycle;
- mutual exact cycle;
- decreasing recursion succeeds;
- changing nonterminating recursion hits budget, not cycle;
- memoized repeated subproblem evaluates once logically;
- symbolic relation residual;
- specialization reduces residual;
- cold exact == specialize symbolic;
- cancellation mid-recursion;
- large residual growth budget;
- dependency edit invalidates root;
- irrelevant edit reuses cached deterministic result;
- proof environment participates in key when result changes;
- alpha/source formatting changes do not alter semantic residual identity where declaration identity is unchanged.

## 15.16 Performance gates

Benchmark:

- recursive Count at 10/100/1000 entries;
- repeated-subproblem algorithm with/without memoization baseline;
- symbolic residual formation for deep generic structures;
- specialization of cached residual vs cold re-evaluation;
- cancellation latency;
- memory growth under deliberately expanding recursion.

P7 should prevent accidental exponential behavior in common Pack map/filter/fold patterns.

## 15.17 Exit criteria

P7 exits when reusable generic static definitions can recurse, residualize, specialize, memoize, terminate predictably under shared budgets/cancellation, and produce canonical deterministic results independent of analysis history.

---
# 16. C4.P8 — Incremental durability, metadata, cross-module static products, and compiler materialization

## 16.1 Objective

Turn P2–P7 semantic behavior into a durable compiler architecture:

- stable query products;
- precise incremental dependencies and fingerprints;
- exported ordinary `const` values;
- imported static definitions/reflection facts;
- metadata schema evolution from current v10;
- deterministic constant/static-value serialization;
- compiler lowering from frozen semantic results;
- runtime materialization without re-running static initializers;
- cold/incremental equivalence.

P8 is where C4 stops being “works in one semantic session” and becomes a sound multi-module incremental language feature.

## 16.2 Query products

Add explicit root products for C4. Conceptually:

```rust
pub enum QueryKey {
    // existing...
    StaticDefinition(DeclarationId),
    ConstantBinding(StableBindingId),
    // application-level key only if profiling justifies global reuse
}
```

Products should publish:

### Static definition product

```text
definition identity
checked generic signature
result domain
normalized body/semantic body reference
referenced static dependencies
fingerprint
diagnostics/status
```

### Constant product

```text
binding identity
result domain/value type
Reduced StaticValueId / durable value descriptor
materialization descriptor
fingerprint
dependencies
diagnostics/status
```

Do not make `const` value identity implicit in an expression-typing cache whose fingerprint only contains the ordinary runtime type.

## 16.3 Exported constant invalidation law

Changing:

```telos
export const limit = 10
```

to:

```telos
export const limit = 20
```

must invalidate importers even though both values have type `Int`.

Therefore the exported semantic/interface fingerprint must include **constant value identity/content**, not just binding name/type/immutability.

Similarly, changing an exported reflected structure or static-definition body in a way that changes specialization results must invalidate dependents according to semantic product fingerprints.

## 16.4 Module interface semantics

P1 removes the misleading `is_const`-as-immutability assumption. P8 completes the module contract:

```text
runtime immutable binding
runtime mutable binding
static constant binding + durable semantic value
static definition
ordinary type/data/trait declarations
```

The linker should not need to execute source code to learn a `const` value. Imported constants arrive as metadata/semantic products and are materialized only if runtime code uses them.

## 16.5 Metadata schema revision

Current schema is v10. C4 should bump the schema once with a coherent additive/transition design if practical rather than serial bumps per sub-plan.

Required additions likely include:

1. `FieldKeyRef` for generalized unordered fields;
2. ordinary `StaticValueRef` / static value pool;
3. first-class Pack static values and shape/value encodings;
4. static result domains beyond `Kind`;
5. exported `const` binding values/fingerprints;
6. static-definition semantic descriptors needed by importers;
7. Product/Sum reflection capability/associated result descriptors if not recomputable solely from existing declaration metadata;
8. normalized residuals **only if** they must cross module boundaries;
9. any generalized runtime record-shape key metadata consumed by compiler lowering.

## 16.6 Stable static value encoding

Prefer an indexed immutable pool/DAG:

```rust
pub enum StaticValueRef {
    Unit,
    Bool(bool),
    Int(IntRef),
    Float(FloatRef),
    String(StringRef),
    Symbol(StableSymbolRef),
    Type(TypeRef),
    StaticTerm(TypeRef),
    Pack(StaticPackRef),
    Tuple(StaticProductRef),
    Record(StaticProductRef),
    Data(StaticDataValueRef),
    Variant(StaticVariantValueRef),
}
```

Requirements:

- stable across sessions;
- no `TypeId`, `KindId`, VM `Symbol`, `ObjRef`, arena offset, `StaticValueId`, or `RecordRowId` serialized as semantic identity;
- deterministic ordering of pools;
- deduplication where it materially reduces metadata;
- recursion/cycles impossible or represented explicitly if future value forms require them—C4 immutable structural values should normally be DAGs;
- decode validates indices/domains and fails closed on malformed metadata.

## 16.7 Type/static reference reuse

Reuse the existing metadata indirection for `TypeRef`, `StaticDomainRef`, `StaticDataRef`, `StaticArgumentRef`. Do not embed a second type serializer inside `StaticValueRef`.

A static Pack of Type values should reference the same canonical imported type identities used everywhere else.

## 16.8 FieldKey metadata

Finalize O4's canonical ordering and encode:

```text
Symbol → stable semantic symbol representation
String → UTF-8/content value
Int    → canonical arbitrary/fixed-width integer representation matching language Int semantics
Bool   → bool
```

On import, reconstruct semantic `FieldKey` values, then intern rows using canonical order. Round-trip must be independent of source order and producer process symbol numbering.

## 16.9 Residual export policy

O14 should be decided before schema freeze.

Preferred policy:

- export the checked static definition and stable declaration/generic products needed to evaluate/specialize in the importer;
- export a residual only when it is itself a canonical public semantic product whose reuse avoids re-evaluation and has stable dependency semantics;
- never serialize evaluator frames, raw AST pointers, transient local slots, budget state, cancellation state, or memo hash maps.

If the existing metadata already exports enough type-definition body semantics to reconstruct applications, keep residuals session-local in C4.

## 16.10 Product/Sum capability import

Derived structural capabilities must remain unforgeable across metadata boundaries. Import should trust only metadata produced for canonical declaration categories/shape facts, not arbitrary serialized “implements Product=true” user claims.

Validation should bind capability evidence to:

```text
declaration identity
+
algebraic declaration shape/product vs sum category
+
associated reflected shape fingerprint
```

or recompute derived capability facts from imported canonical `DataInfo`/`EnumInfo` equivalents.

## 16.11 Semantic lowering boundary

By the time runtime compiler lowering begins:

```text
all required consts               Reduced
required static type computations normalized
selected match/variant decisions frozen where statically required
reflection values either erased or represented as semantic constants
runtime materialization descriptors available
```

`phalcom-core` must **not** opportunistically invoke the static evaluator because it encounters a `const` reference or a type computation.

This keeps semantic and codegen invalidation/diagnostics deterministic.

## 16.12 Constant materialization lowering

Compiler consumes semantic descriptor and chooses efficient runtime form:

### Immediate primitives

Emit existing chunk constant/immediate representation.

### String/Symbol

Intern/load through canonical runtime constant mechanisms; do not preserve semantic arena IDs.

### Tuple/Record/Pack

Use static product/Pack construction descriptors, prebuilt immutable pools, or compact bytecode constants according to current VM architecture. Preserve ProductStorage/layout optimization.

### Data/Variant

Use canonical data/variant constructor descriptors and ProductStorage, but **do not execute user constructors/methods**. A compile-time transparent data value is materialized structurally from its proven declaration/value payload.

### Large constants

Prefer one module/chunk constant pool object/DAG materialization over duplicating construction bytecode at every use site.

## 16.13 Runtime lifecycle of `const`

A module `const` initializer never runs during module execution. Runtime use loads/materializes the frozen semantic result.

Questions to settle in implementation:

- eager materialization at module load vs lazy first-use;
- sharing of immutable materialized constants within a VM/module;
- GC rooting of module constants;
- whether equal constants across modules may share storage as an optimization.

These are physical choices only. Observable semantics are value-based and the initializer does not rerun.

## 16.14 Runtime identity-sensitive operations

If a compile-time structural constant is materialized more than once physically, runtime identity operators must still obey the language's defined identity semantics. This reinforces O6/P2's rule: static evaluation may use exact identity only where allocation identity cannot change observable result.

If Telos `===` on strings/products observes allocation identity, either materialize a stable shared object per constant binding or declare such identity checks outside the static-valid subset. Do not leave this accidental.

## 16.15 Incremental dependency tests

Required edits:

- change exported const value, same type → importer invalidated;
- change const initializer, same reduced value → ideally semantic fingerprint unchanged after normalization if architecture supports it; at minimum result remains correct;
- change unused local static definition → unrelated importer reused;
- change Product field shape → reflected dependents invalidated;
- reorder unordered fields without semantic shape change → reflection/result fingerprints should remain stable except presentation-specific products if intentionally exposed;
- change variant declaration order → `Variants` dependents invalidated because declaration order is semantic reflection order;
- change variant payload type → dependents invalidated;
- generic static definition edit → only application dependents invalidated through definition product;
- cancellation must not poison future query reuse.

## 16.16 Cold/incremental parity

For every durable C4 product, certify:

```text
cold build result
==
incremental build after equivalent edit sequence
```

Compare:

- semantic value identity/content;
- diagnostics and source attribution;
- reflection order;
- relation/proof terminal state;
- metadata output/fingerprint where deterministic;
- lowered runtime behavior.

This is a completion gate, not optional optimization testing.

## 16.17 Metadata compatibility strategy

Because schema v10 consumers exist, choose explicitly:

- strict version bump with unsupported-old/new diagnostic;
- or versioned decoder supporting v10 + v11 during transition.

Do not silently reinterpret v10 string record keys as generalized keys without an explicit import mapping such as `old name -> FieldKey::Symbol(...)` matching the historical source semantics.

## 16.18 Security/robustness of metadata decode

Imported metadata is structured input. Validate:

- pool indices;
- duplicate Pack labels;
- duplicate unordered FieldKeys;
- canonical ordering or canonicalize safely;
- domain/result mismatches;
- declaration references;
- impossible Product/Sum capability combinations;
- value depth/size budgets to avoid malicious metadata blowups.

## 16.19 Performance gates

Benchmark:

- metadata size for type-heavy project before/after C4;
- load/decode time;
- 1k exported constants;
- large Pack/Record constant decode;
- incremental edit of one const with many importers;
- repeated imported static-definition specialization;
- module runtime startup with unused large const (to guide lazy/eager materialization);
- runtime repeated access to same const.

Avoid material metadata bloat from repeating full reflection values that can be reconstructed from declaration facts.

## 16.20 Exit criteria

P8 exits when C4 semantic products survive module boundaries and incremental recompilation with stable identity, constants never rerun their initializers at runtime, metadata contains no process-local identities, and compiler lowering consumes frozen semantic descriptors rather than re-solving static semantics.

---

# 17. C4.P9 — Public ontology migration, tooling, documentation, performance certification, and checkpoint closure

## 17.1 Objective

Finish C4 as a language checkpoint rather than leaving a technically working semantic subsystem surrounded by stale source APIs, tools, docs, compatibility facades, and unmeasured performance risk.

P9 is **not** a dumping ground for correctness work omitted from earlier plans. P1–P8 must already be locally correct. P9 performs whole-system migration, retirement, user-facing integration, stress/performance certification, and final checkpoint evidence.

## 17.2 Public structural ontology cutover

The pre-C4 public model exposes:

```text
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data CompleteTypePack
intrinsic type data RecordFields
```

C4 target is:

```text
Pack / PositionalPack / LabeledPack / CompletePack
Product / OrderedProduct / UnorderedProduct
OrderedShape / UnorderedShape
FieldKey / FieldInfo / Presence
Sum / Variant / Variants
```

P9 must complete O15:

### Preferred end state

- old compiler-optimized kinds/nodes remain internal where valuable;
- Universe/public docs no longer teach them as separate conceptual data structures;
- aliases such as `PositionalTypePack = PositionalPack<Type>` are the source-visible model;
- old public `RecordFields` reflection surface is replaced by `UnorderedShape`/fixed fields;
- compatibility names, if temporarily retained, are explicitly deprecated aliases/facades with a removal checkpoint.

### Prohibited end state

Two equally documented public systems where some APIs return old `TypePack`/`RecordFields` and others return new Pack/Product shapes indefinitely.

## 17.3 Universe source migration

Audit and migrate:

```text
phalcom-core/core/universe/src/collections/tuple.ph
phalcom-core/core/universe/src/collections/record.ph
related type/static-domain modules
core exports
future/type.telos or equivalent staged surfaces
```

Add canonical C4 source declarations for:

- Pack and Pack aliases/views;
- OrderedTail/OrderedShape;
- FieldKey/Presence/FieldInfo/ProductField/FieldLookup;
- UnorderedTail/UnorderedShape/View;
- ProductKind/ProductShape and capability traits;
- Variant/Variants/VariantNames/Sum;
- source-defined reflection algorithms once P6/P7 can evaluate them.

Keep intrinsic declarations minimal: compiler facts and operations, not algorithms easily written in Telos.

## 17.4 Repair attached design artifact inconsistencies in canonical docs

P9 documentation/source examples must use one vocabulary:

| Stale | Canonical |
|---|---|
| `TypePackView::Item` | `PositionalPackView::Entry` |
| `LabeledTypePackView::Item` | `LabeledPackView::Entry` |
| `P::Positionals` | `P::PositionalEntries` |
| `P::Labeled` | `P::LabeledEntries` |
| `RecordProduct` | `UnorderedProduct` |
| `RecordField` | `ProductField` |
| `FieldPresence` | `Presence` |
| `type F = { ... }` block form | `type F { ... }` |
| raw `OrderedShape` matched against `T::Shape` | `ProductShape::Ordered(OrderedShape(...))` or use `T::OrderedShape` |

Source utilities must bind variables they use; e.g. fix the stale `VariantsNamed` example that appends `variant` without binding it.

## 17.5 Formatter

If the repository has formatter/printer paths for these nodes, update:

- `let mut` spacing;
- `mut _field`;
- block-bodied `type` definitions with no `=`;
- generalized FieldKey bracket forms;
- Pack structural patterns and expansions;
- Product/Sum reflection examples.

Round-trip parse/format/parse tests should preserve AST semantics.

## 17.6 LSP

Update semantic/editor behavior, not just token coloring.

### Semantic tokens

- `mut` keyword;
- static `const` remains keyword;
- block-bodied type syntax;
- generalized key expressions reuse normal expression tokenization inside brackets;
- derived Product/Sum associated members classify as type/static members consistently.

### Hover

Update keyword docs:

```text
let      runtime immutable
let mut  runtime mutable
const    compile-time constant
mut field mutable field marker
```

Hover for `const` should surface reduced static type/value when reasonably compact, with truncation for large structures.

Reflection members should expose result domain/type.

### Completion

- suggest `mut` after `let` and in field declaration context, not indiscriminately;
- new Universe reflection types/members;
- remove/deprioritize deprecated old structural facades according to O15;
- generalized FieldKey contexts where completion is meaningful.

### Definition/references/rename

Static definitions and constants retain stable declaration identity. Reflection-derived members should navigate to source declarations/intrinsic Universe declarations as appropriate, not synthetic mystery locations.

## 17.7 REPL

Decide REPL handling for `const` explicitly:

- evaluate through semantic static evaluator before execution;
- persist constant binding/result across REPL submissions according to existing module/session model;
- reject runtime dependency in const;
- show useful static diagnostics;
- support `let`/`let mut` mutation semantics.

Update all independent keyword arrays/highlighters/completers and eliminate duplicated keyword lists where practical to reduce future drift.

## 17.8 VS Code/TextMate and generated keyword tables

Update generator source of truth first, regenerate outputs, then update/manual-test grammar.

Audit at least:

```text
phalcom-core/bin/gen-core-table/**
tools/vsphalcom/src/generated/core-table.json
tools/vsphalcom/syntaxes/phalcom.tmLanguage.json
tools/vsphalcom/manual-test/**
```

Do not hand-edit generated tables without regenerating.

## 17.9 Fuzzing/property tests

Add corpus/fuzz tokens/forms for:

- `mut`;
- block-bodied `type`;
- Pack constructions/expansions;
- generalized FieldKey literals;
- nested structural patterns;
- repeated variant families;
- open/captured tails.

Property tests suitable for C4:

- unordered shape source-order permutation preserves canonical semantic shape;
- Pack split/view/recompose identity;
- metadata encode/decode identity;
- cold/incremental semantic equivalence;
- reflection specialization commutes with direct specialized formation where expected;
- fixed-call lowering contains no Pack allocation markers.

## 17.10 Documentation supersession

Explicitly supersede pre-C4 docs that say:

```text
let = mutable
const = immutable runtime
bare field = mutable
const field = immutable
TypePack/RecordFields are public fundamental ontology
```

Do not leave accepted ADR/spec pages silently contradictory. If historical decisions remain for provenance, mark them superseded and link to C4 authoritative semantics.

Update:

- language specs;
- Universe/type-system specs;
- VM product/call docs;
- module metadata docs;
- LSP docs;
- wiki/raw generated sources if repository policy requires;
- examples/tutorials.

## 17.11 Compatibility/deprecation policy

For any retained old public name, document:

```text
old name
new equivalent
whether source alias or compiler-only compatibility
warning behavior
planned removal checkpoint
metadata compatibility behavior
```

Avoid compatibility layers in semantic hot paths. Prefer parse/source aliases that normalize immediately to canonical C4 forms.

## 17.12 Whole-workspace performance certification

P9 owns final benchmark comparison against the pre-C4 baseline.

Required scenarios:

### Runtime hot paths

- ordinary fixed method/function call;
- fixed labeled call;
- runtime tuple/record construction/access;
- ordinary code with no `const`/reflection;
- module startup.

### Compile-time/static paths

- simple alias/static definition;
- 100/1000-entry Pack map/filter/count;
- large unordered shape relation;
- 1k variants reflection/filter;
- recursive static computation with memo reuse;
- metadata load;
- incremental one-file edit;
- editor analysis latency/cancellation.

### Allocation checks

Track:

- fixed calls: no Pack allocation;
- static views: no O(n) rest copies;
- reflection cache reuse;
- constant materialization sharing;
- metadata decode temporary allocations.

## 17.13 Regression thresholds

Set repository-appropriate explicit thresholds before declaring completion. At minimum:

- no material statistically significant regression in fixed-call throughput/allocation;
- no superlinear Pack view/iteration behavior;
- no superlinear unordered fixed-field lookup/formation beyond intentionally sorted O(n log n) canonicalization;
- bounded cancellation latency for static evaluation;
- metadata size growth explained by actually exported C4 products, not duplicated internal caches.

If the project lacks stable benchmark infrastructure, add focused microbenchmarks and record before/after numbers in the C4 walkthrough/checkpoint.

## 17.14 Final workspace certification

Run the complete relevant suite, not only focused C4 tests:

```text
cargo test / nextest according to repository convention
cargo clippy according to repository gate
cargo fmt/check
semantic focused suites
core runtime tests
module/linker tests
metadata schema compatibility tests
LSP tests
REPL tests
VS Code grammar/manual/golden tests as available
fuzz smoke/property tests
benchmarks
```

Also run grep/audit guards for stale concepts:

```text
BindingKind::Let mutability assumptions
old const-field prose/syntax
TypePackView::Item
RecordProduct / RecordField public terms
FieldPresence
old TypePack/RecordFields public docs
finish_tuple on rest-capture paths
runtime evaluator/VM calls from static_eval
```

## 17.15 Checkpoint artifacts

Produce:

- final C4 implementation plans P1–P9;
- implementation walkthrough(s);
- C4 checkpoint status/evidence file;
- performance evidence;
- metadata schema migration note;
- compatibility/deprecation note;
- open/deferred ledger with every non-C4 item explicitly justified.

## 17.16 Exit criteria

C4.P9 and C4 are complete only when:

1. public source/docs teach one Pack/Product/Sum ontology;
2. old binding/field semantics are gone or explicitly historical;
3. all tools understand the new syntax and static semantics;
4. metadata/incremental/runtime behavior is certified end-to-end;
5. fixed-call/runtime baseline performance is protected;
6. no compile-time VM/runtime-dispatch dependency exists;
7. no intentionally open C4 decision remains undocumented;
8. every deferred feature—especially const functions/methods—is explicitly outside the checkpoint;
9. completion criteria in Section 25 are all satisfied.

---
# 18. Cross-plan repository impact matrix

This matrix is a planning aid, not an exhaustive file list. Implementers should re-run repository searches at the start of each plan because paths may move after preceding refactors.

| Repository area | P1 | P2 | P3 | P4 | P5 | P6 | P7 | P8 | P9 |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| `phalcom-ast/src/token.rs`, `lexer.rs` | ● |  |  | ○ |  | ○ |  |  | ○ |
| `phalcom-ast/src/ast.rs`, `parser.rs` | ● | ● | ○ | ● | ○ | ● | ○ |  | ○ |
| semantic binding/body checker | ● | ● | ○ | ○ | ○ | ● | ● | ○ | ○ |
| `types/store.rs` / kind/static terms |  | ● | ● | ● | ○ | ○ | ● | ● | ○ |
| `types/row.rs`, row solver/inference |  |  |  | ● | ○ | ● | ○ | ● | ○ |
| `types/type_member.rs`, projection/normalization |  | ● | ● | ○ | ● | ● | ● | ● | ○ |
| data semantics |  | ○ | ○ | ○ | ● | ● | ○ | ● | ○ |
| enum/match/GADT semantics |  | ○ |  |  | ● | ● | ● | ● | ○ |
| conformance/trait proof engine |  | ○ |  |  | ● | ● | ● | ● | ○ |
| semantic DB/query/fingerprint | ○ | ● | ○ | ○ | ○ | ○ | ● | ● | ○ |
| metadata export/import |  | ○ | ○ | ○ | ○ | ○ | ○ | ● | ● |
| module interfaces/linkage | ● | ○ |  |  | ○ |  | ○ | ● | ○ |
| core Universe source | ○ | ○ | ● | ● | ● | ● | ● | ○ | ● |
| compiler expression/lowering | ● | ○ | ● | ● | ○ | ○ | ○ | ● | ○ |
| VM argument pack |  |  | ● |  |  |  |  | ○ | ○ |
| VM product shape/layout/storage |  |  | ● | ● | ○ |  |  | ● | ○ |
| VM data/variant materialization |  | ○ | ○ | ○ | ○ |  |  | ● | ○ |
| product optimizer | ● | ○ | ● | ○ | ○ |  |  | ● | ● |
| LSP | ● | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ● |
| REPL/editor grammar | ● | ○ |  |  |  |  |  | ○ | ● |
| docs/specs/ADRs | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ● |

Legend: **●** primary plan ownership, **○** integration/touchpoint.

## 18.1 Repository searches to repeat before implementation

```bash
rg -n "BindingKind::(Let|Const)|BindingKind|is_mutable|ConstWithoutInitializer|AssignToImmutable" \
  phalcom-ast phalcom-semantic phalcom-core phalcom-modules phalcom-lsp phalcom-repl

rg -n "FieldDef|mutable:|const _|implicit.*field|field.*lifecycle" \
  phalcom-ast phalcom-semantic phalcom-core

rg -n "TypeAliasDef|result_kind|TypeMemberDomain" \
  phalcom-ast phalcom-semantic phalcom-type-meta

rg -n "TypePack|LabeledTypePack|CompleteTypePack|RecordFields|RecordRow" \
  phalcom-ast phalcom-semantic phalcom-core phalcom-type-meta phalcom-modules phalcom-lsp

rg -n "RecordRowField|RecordTypeField|presentation_labels|logical_labels|HashMap<Symbol" \
  phalcom-semantic phalcom-core phalcom-type-meta

rg -n "ArgumentPackBuilderObject|FinishTuplePack|finish_tuple|NewArgumentPack|InvokePack|PackExpand" \
  phalcom-core

rg -n "VariantFamilyId|VariantInfo|EnumInfo|ResolvedVariantCandidate|BranchProofEnvironment|MatchResolution" \
  phalcom-semantic

rg -n "QueryBudget|CancellationToken|BudgetExceeded|QueryKey|ProductFingerprint" \
  phalcom-semantic

rg -n "whileTrue|WhileLet|parse_while|Statement::For|ForStatement" \
  phalcom-ast phalcom-semantic phalcom-core
```

---

# 19. Canonical diagnostic architecture

C4 adds many failure modes. Diagnostics should be grouped by semantic authority and avoid one generic “not compile-time constant” error for every case.

## 19.1 Binding/field

```text
binding.initializer_required
binding.immutable_assignment
binding.invalid_mut_modifier
binding.redeclared
field.mutable_required
field.immutable_reassignment
field.initialization_required
field.read_before_initialization
```

## 19.2 Constant/static evaluation

```text
const.non_static_dependency
const.static_evaluation_failed
const.symbolic_result
const.result_not_materializable

static.invalid_operation
static.runtime_call_not_allowed
static.runtime_binding_not_allowed
static.identity_not_available
static.recursive
static.cycle
static.budget_exceeded
static.residual_too_large
static.result_domain_mismatch
static.application_domain_mismatch
static.dynamic_dependency
static.blocked_dependency
```

## 19.3 Type/static definition

```text
type.static_body_invalid
type.result_domain_mismatch
type.static_application_failed
```

## 19.4 Pack/product/unordered

```text
pack.duplicate_label
pack.invalid_expansion
pack.invalid_lane
pack.result_shape_mismatch

field.invalid_key
field.duplicate_key
field.duplicate_key_after_specialization
field.invalid_tail
reflection.member_not_applicable
reflection.invalid_structural_capability
```

## 19.5 Sum/variant

```text
variant.no_matching_shape
variant.ambiguous_shape
variant.invalid_reflection
sum.invalid_derived_capability
```

## 19.6 Static control

```text
static.iteration_not_structural
static.iteration_open_tail_not_finite
static.refutable_loop_pattern
static.break_outside_loop
static.continue_outside_loop
static.return_outside_definition
static.while_condition_not_bool
static.pattern_projection_failed
```

## 19.7 Proof terminal rendering

Do not hide rich semantic states behind false or a generic error. Diagnostic notes should distinguish:

```text
Unknown        not enough static information
Blocked        another semantic dependency could not complete
Dynamic        proof crosses a runtime/dynamic boundary
Cancelled      editor/query interruption; usually suppress/restart
BudgetExceeded bounded static work exhausted
InternalFailure compiler semantic failure
```

For user-facing closed `const`, these states prevent successful reduction; for generic static definitions, P7 may turn appropriate Unknown/static-parameter dependence into a residual instead.

## 19.8 Diagnostic trace policy

Static evaluation traces should be concise and bounded by the existing diagnostic note cap. Include:

```text
root const/static definition
application chain
selected source spans
summarized static arguments
cycle start if any
budget kind and usage if any
```

Do not dump full thousand-entry Packs into errors. Implement stable truncation/summarization.

---

# 20. End-to-end verification matrix

## 20.1 Lexing/parsing

Certify:

- `mut` tokenization;
- `let`, `let mut`, `const`;
- immutable/mutable field declarations;
- expression and block static type bodies;
- Pack literal/construction/expansion syntax;
- generalized FieldKey syntax;
- open/captured tail syntax;
- structural match patterns;
- no ambiguity among `*`, `**`, `***`, and bracket-symbol keys such as `[#*]`;
- parser recovery around missing delimiters/colon/initializer.

## 20.2 Binding/flow

- immutable `let` assignment errors;
- mutable flow for `let mut`;
- destructuring;
- closure capture;
- top-level/module bindings;
- same-scope redeclaration;
- field definite initialization and mutation lifecycle;
- implicit field policy.

## 20.3 Static value correctness

- primitive values;
- semantic equality/hash;
- Float edge cases;
- strings/symbols;
- Tuple/Record/Pack distinction;
- nominal transparent data;
- variants;
- nested constant values;
- materialization round-trip.

## 20.4 Static evaluation

- literals/operators;
- short-circuiting;
- `if`;
- blocks/local mutation;
- result domains;
- forbidden runtime dependency/call;
- selected-branch-only execution;
- cancellation;
- budget exhaustion.

## 20.5 Pack

- empty/nonempty;
- positional/labeled/complete;
- duplicate labels;
- type aliases;
- views;
- expansion;
- rest capture all modes;
- forwarding;
- exact shape preservation;
- equality/hash;
- static map/filter/fold;
- runtime materialization;
- fixed-call no-allocation certification.

## 20.6 Unordered products

- all FieldKey kinds;
- optional presence;
- width subtyping;
- source reorder normalization;
- open typed/unconstrained/captured tails;
- fixed lookup outcomes;
- row inference/lacks;
- runtime Record construction/access;
- duplicate after specialization;
- metadata stable order.

## 20.7 Product reflection

- intrinsic and user-defined ordered/unordered products;
- generic specialization;
- tails;
- nominal identity not erased;
- unforgeable capability evidence;
- associated static result domains.

## 20.8 Sum reflection

- ordinary variants;
- repeated names;
- mixed payload shapes under same name;
- declaration order;
- generic specialization;
- GADT cases;
- nullary cases;
- source-defined Variants filters/utilities.

## 20.9 Static match/control

- Pack patterns;
- ProductShape patterns;
- repeated variant family selection;
- GADT branch proofs;
- static ordered/unordered iteration;
- heterogeneity correlation;
- open tail non-iteration;
- return/break/continue;
- static while;
- refutable loop pattern rule.

## 20.10 Recursive computation

- direct/mutual cycles;
- decreasing recursion;
- nonterminating changing recursion hits budget;
- memoized subproblems;
- residual formation;
- specialization;
- cold exact vs residual-specialized equality.

## 20.11 Incremental/metadata

For every feature above, where exported/imported:

- metadata round trip;
- stable fingerprint;
- cold/imported parity;
- incremental add/edit/delete/rename;
- cancellation recovery;
- no process-local IDs in serialized bytes.

## 20.12 Runtime/compiler

- no const initializer runtime execution;
- constant materialization correctness;
- GC tracing of materialized Pack/Record/data/variant constants;
- no fixed-call Pack allocation;
- product optimizer/scalar replacement remains correct;
- direct/static dispatch decisions remain frozen correctly.

---

# 21. Performance and complexity requirements

C4 is unusually susceptible to compile-time performance regressions because it introduces source-defined structural algorithms. Complexity needs to be part of the design, not a post-hoc optimization pass.

## 21.1 Target complexity table

| Operation | Target |
|---|---:|
| Pack lane length | O(1) |
| Pack first/rest view | O(1) |
| Pack fixed-index access, if exposed | O(1) or O(log n) depending representation; never O(n) scan per repeated view |
| Pack equality/hash | O(n) |
| Pack builder append/finalize | amortized O(1) append, O(n) finalize |
| Pack source-defined repeated append under evaluator | effective O(n), not O(n²), via builder fusion |
| unordered fixed field canonicalization | O(n log n) acceptable |
| unordered lookup after intern | O(log n) or O(1) cached |
| duplicate key/label detection | O(n) or O(n log n), not O(n²) at large n |
| `VariantsOf` source filter | O(v) |
| reflection fetch after cache | O(1) key lookup + result access |
| exact static memo lookup | expected O(1) / bounded map |
| static loop | O(iterations × body semantic work) |
| metadata decode | O(serialized nodes + canonicalization cost) |

## 21.2 Avoid hidden quadratic sources

Specifically review:

- Pack append by immutable reconstruction;
- `ArgumentPackBuilderObject` duplicate-label `contains` per insertion;
- repeated `Vec::position` reflection lookup;
- unordered shape duplicate detection before sorting;
- recursive view creation that copies rest;
- repeated formatting/hashing of giant static arguments for memo keys;
- repeated substitution of entire generic environments per iteration;
- variant name filters that repeatedly scan all variants inside another loop.

## 21.3 Memo key size

Large static arguments should have canonical intern IDs/fingerprints so application memo lookup does not re-hash every nested element on every recursive call. Interned `StaticValueId` identity can be used **within a semantic session** after content canonicalization; durable fingerprints use stable content/declaration identity.

## 21.4 Value-growth budgeting

Steps alone may not stop code that doubles a Pack each iteration. Add a named growth guard if measurements/abuse tests show need:

```text
static value nodes
Pack entries constructed
residual nodes
or aggregate semantic bytes estimate
```

The budget must be observable in `BudgetReport`, not an unexplained “too complex” error.

## 21.5 Cold/editor performance

Static evaluation can run during editor analysis. Requirements:

- cooperative cancellation at loop/application/build boundaries;
- no coarse global mutex around static value interning;
- query-local evaluator frames;
- shared immutable canonical stores where repository architecture supports it;
- avoid eagerly evaluating static definitions that no semantic consumer requests.

## 21.6 Runtime performance invariant

The strongest runtime invariant is:

> **C4 must not make an ordinary fixed call allocate or construct a first-class Pack solely because invocation is semantically described as a Pack.**

This requires bytecode/optimizer tests, not only benchmark observation.

---

# 22. Risk register and mitigation

| Risk | Severity | Evidence | Mitigation / owning plan |
|---|---|---|---|
| binding codemod changes behavior | High | current `let`/`const` meanings are opposite target | settle O1–O3; parser-assisted migration; P1 |
| `const` becomes compile-time VM | Critical architectural | temptation to reuse runtime operations | evaluator lives in semantic; closed intrinsic operation table; P2 |
| Pack allocation regresses all calls | Critical performance | existing dynamic builder machinery makes materialization easy | preserve fixed call path + bytecode assertions; P3 |
| public TypePack and Pack become dual ontologies | High design debt | C3 public facades already live | explicit alias/compatibility plan; P3/P9 |
| FieldKey treated as metadata-only | High correctness | runtime Record shape is Symbol-keyed | migrate semantic + runtime key authorities; P4 |
| optional row semantics regress | High correctness | recent fix exists | reuse row authority; relation matrix regression tests; P4 |
| Product/Sum traits forgeable | High soundness | normal trait impl system exists | compiler-derived sealed evidence; P5 |
| repeated variant names collapsed | High correctness | public Symbol family semantics intentional | use VariantFamilyId/exact VariantId; P5/P6 |
| static match duplicates GADT resolver | High maintenance/soundness | mature MatchResolution exists | consume semantic products only; P6 |
| static `for` invokes runtime protocol | High architecture | runtime `for` currently lowers to iterator sends | classify structural static iteration separately; P6 |
| `.whileTrue` mistaken for static while | High semantics | historical lowering | preserve explicit control identity; P6 |
| Unknown proof becomes false | High soundness | rich proof states exist | residual/terminal distinction; P6/P7 |
| host stack overflow from source recursion | High robustness | recursive type definitions allowed | explicit evaluator frames; P7 |
| memoization becomes stale across proofs | High correctness | branch/specialization facts can affect result | include relevant canonical environment in key; P7 |
| residual IR becomes second language/AST clone | Medium/high complexity | residualization needed | small normalized semantic IR only; P7 |
| const value edit not invalidating importer | Critical incrementality | existing module surface `is_const` carries no value | export value fingerprints/query dependencies; P8 |
| process-local IDs leak into metadata | Critical durability | TypeId/Symbol/arena IDs abound internally | stable refs/content encodings; P8 |
| large const eagerly hurts startup | Medium performance | structural constants may be large | profile lazy materialization/pooling; P8 |
| stale docs/tooling cause two language versions | Medium/high UX | extensive old ADRs/examples/tool keyword tables | explicit supersession and full tooling audit; P9 |

---

# 23. Deferred and explicitly non-goal work

C4 should remain disciplined. The following are **not** required merely because static evaluation exists:

- const functions;
- const methods;
- arbitrary runtime function execution at compile time;
- compile-time VM/bytecode interpreter;
- filesystem/network/process build scripts;
- procedural macros;
- runtime mutation of Pack;
- making Record implicitly convertible to labeled Pack for call expansion;
- dynamic mutation of class/method tables;
- public GADT proof-environment reflection unless O10 is explicitly reopened;
- replacing all optimized internal `TypeData::TypePack`/row nodes with literal source-level Pack/Record values;
- runtime iteration protocol changes solely to support static structural `for`;
- a universal metaprogramming/reflection object API beyond the ratified Product/Sum structural facts;
- const-function compatibility shims hidden behind `type` syntax.

Any implementation proposal that requires one of these to make a core C4 feature work should trigger an architecture review first.

---

# 24. Recommended implementation sequence and review checkpoints

## 24.1 P1 first: semantic cutover in a green tree

Land binding/field syntax and behavior-preserving corpus migration as one coherent transition. Do not mix the initial static evaluator into the same review; otherwise failures become hard to attribute between changed runtime binding semantics and new compile-time semantics.

Checkpoint evidence:

```text
new syntax everywhere
runtime behavior preserved where intended
no const runtime initializer behavior
field lifecycle green
workspace green
```

## 24.2 P2 spike before broad static feature work

Implement the smallest end-to-end constant:

```telos
const x = 1 + 2
```

and one non-Type definition:

```telos
type IsOne<N: Int>: Bool = N == 1
```

Prove semantic evaluation, domain checking, query dependency, and materialization descriptor before adding many operators.

## 24.3 P3/P4 in parallel after static-value seams stabilize

P3 probe:

```telos
const p = Pack(1, 2, debug: true)
```

plus runtime `*rest` → Pack and no-allocation fixed call.

P4 probe:

```telos
type T = {
    name: String,
    ["mime"]: String,
    [7]: Int,
    [true]: Bool,
}
```

prove stable canonical order and runtime access before broad row migration.

## 24.4 P5 reflection spike

```telos
data Value {
    Number(_ value: Int)
    Number(_ value: Float)
}

type NumberVariants: Variants =
    VariantsOf<Value, #Number>
```

This validates derived Sum, overloaded family preservation, ProductShape payload reflection, Variants Pack, and static member result domains.

## 24.5 P6 structural control spike

Use the handoff's strongest first vertical probe:

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

This simultaneously certifies block static definitions, Pack/type Pack bridging, local mutation, structural `for`, Int result domain, and const application.

Add a repeated-variant GADT/static match probe in the same plan to verify semantic product reuse.

## 24.6 P7 recursion/residual spike

Recursive equivalent of Count plus a symbolic proof predicate:

```telos
type Count<*P>: Int =
    match P {
        Pack() => 0
        Pack(_, *rest) => 1 + Count<*rest>
    }

type IsNumber<T: Type>: Bool =
    Subtype<T, Number>
```

Certify exact recursion, changing arguments, residualization, specialization, and memoization.

## 24.7 P8 durability spike

Two modules:

```telos
// a
export const C = Pack(1, 2, 3)

// b
import C from a
const N = Count<*C>
```

Then change only C's contents with same apparent runtime type and prove importer invalidation/cold-incremental parity.

## 24.8 P9 closure only after all semantic gates are green

Do not start deleting compatibility facades or mass-updating docs until the replacement semantics are stable. Conversely, do not declare C4 complete while old public ontology remains the normal documented surface.

---

# 25. C4 completion criteria

LANG007.C4 is complete only when **all** of the following are true.

## 25.1 Binding and fields

- [ ] `let` is runtime immutable everywhere.
- [ ] `let mut` is the only ordinary mutable runtime binding form.
- [ ] `const` is a static phase boundary, not a runtime immutable local.
- [ ] no semantic/compiler consumer assumes plain `let` is mutable.
- [ ] fields are immutable by default.
- [ ] `mut` marks mutable declared fields.
- [ ] immutable field initialization/lifecycle semantics are ratified and enforced.
- [ ] implicit field behavior is explicitly resolved.
- [ ] source corpus and generated sources use new syntax.

## 25.2 Static evaluation

- [ ] evaluator lives in `phalcom-semantic` or equivalent semantic layer, not VM.
- [ ] no arbitrary runtime dispatch fallback exists.
- [ ] static values do not depend on VM object identity.
- [ ] closed `const` always reduces completely or diagnoses failure.
- [ ] static result domains support ordinary static data, not only Type/Kind.
- [ ] expression- and block-bodied `type` definitions work.
- [ ] local evaluator mutation/control is deterministic.
- [ ] QueryBudget/cancellation are integrated.
- [ ] const functions/methods are absent and documented as deferred.

## 25.3 Pack

- [ ] Pack is first-class and distinct from Tuple/Unit/Record.
- [ ] both lanes preserve independent order.
- [ ] labels are Symbol and unique.
- [ ] views are canonical and O(1)-style slices, not recursive copied tails.
- [ ] public TypePack aliases are Pack specializations.
- [ ] `*rest`, `**rest`, `***rest` capture Packs.
- [ ] empty rest is Pack().
- [ ] dynamic builder remains private.
- [ ] ordinary fixed calls require no Pack allocation.

## 25.4 Unordered products

- [ ] FieldKey supports Symbol/String/Int/Bool end-to-end.
- [ ] canonical total order is specified and stable.
- [ ] semantic rows and runtime Record shapes use generalized keys.
- [ ] optionality/width typing remain correct.
- [ ] typed/unconstrained/captured tails work.
- [ ] open tails are not iterated as finite fields.
- [ ] lookup returns Required/Optional/Absent/Unknown correctly.
- [ ] specialization collisions diagnose rather than overwrite.

## 25.5 Product/Sum reflection

- [ ] Product/OrderedProduct/UnorderedProduct derive automatically from product data.
- [ ] Sum derives automatically from enumerated data.
- [ ] ordinary user impls cannot forge these capabilities.
- [ ] ProductShape reflection preserves payload structure without erasing nominal declaration identity.
- [ ] Variants is an ordinary PositionalPack<Variant>.
- [ ] repeated variant names remain distinct entries.
- [ ] Variant family selection uses canonical exact variants/families.
- [ ] generic specialization is reflected canonically.

## 25.6 Static control and matching

- [ ] static match consumes existing Pattern/Match/GADT semantic products.
- [ ] no second variant resolver/proof engine exists.
- [ ] static for iterates structural subjects directly, not runtime iterator methods.
- [ ] heterogeneous iteration rule is ratified and sound.
- [ ] static while has explicit semantic control identity.
- [ ] return/break/continue work within evaluator frames.
- [ ] proof predicates preserve Unknown/Blocked/Dynamic distinctions.

## 25.7 Recursion/residualization

- [ ] recursive static definitions use explicit evaluator frames.
- [ ] exact application cycles are detected.
- [ ] changing arguments may recurse until base/budget.
- [ ] generic unresolved computations residualize canonically.
- [ ] residuals are semantic IR, not raw AST/evaluator state.
- [ ] memoization keys use stable definition + canonical arguments + relevant environment.
- [ ] context-sensitive cancellation/budget terminals are not cached as semantic results.

## 25.8 Incrementality/metadata/lowering

- [ ] metadata schema encodes generalized FieldKey and exported static values.
- [ ] stable metadata contains no session-local IDs.
- [ ] exported const value changes invalidate dependents.
- [ ] imported static definitions/constants behave like cold local analysis.
- [ ] compiler lowering consumes frozen static products.
- [ ] runtime never reruns const initializer.
- [ ] materialized constants use canonical runtime product/data/variant machinery.
- [ ] cold/incremental parity tests pass.

## 25.9 Tooling/docs/performance

- [ ] formatter/LSP/REPL/editor grammar understand new syntax.
- [ ] public docs use canonical Pack/Product/Sum terminology.
- [ ] stale TypePack/RecordFields ontology is migrated or has a bounded explicit compatibility exit.
- [ ] old binding/field docs are marked superseded.
- [ ] attached design-example naming inconsistencies are repaired in authoritative docs.
- [ ] fixed-call performance/allocation baseline shows no material regression.
- [ ] static structural algorithms meet complexity expectations.
- [ ] metadata/editor latency benchmarks are acceptable.
- [ ] full workspace certification is green.

---

# 26. Final requirements verdict

C4 is implementable on the current repository architecture without a compile-time VM and without discarding the C1–C3 structural/type machinery. In fact, the repository already contains most of the hard **authorities** C4 needs: canonical TypeStore terms, heterogeneous static arguments, stable declaration identity, rich conformance proof terminals, GADT/match products, variant families, query budgets/cancellation, incremental products, and converged runtime ProductStorage.

The difficult work is **convergence**, not invention:

```text
current runtime immutable `const`
    → true compile-time const

current mutable `let`
    → immutable `let` + explicit `let mut`

current TypePack facades
    → Pack-based public ontology over retained optimized internal nodes

current string/Symbol record rows
    → stable FieldKey unordered-product calculus across semantic + runtime layers

current type-member Kind-only results
    → general static result domains

current enum/match authorities
    → Sum/Variant static reflection and matching reuse

current runtime argument builder
    → first-class Pack only where observable, with fixed-call fast path retained
```

The plan set above is intentionally nine units. That count is the smallest decomposition that keeps each major semantic/representation authority independently reviewable and performance-certifiable, while avoiding artificial parser/semantic/runtime micro-plans that cannot deliver a complete language capability on their own.

The decisions most important to settle **before implementation starts in earnest** are O1–O3 (initializer/field/implicit-field migration). O11 (unforgeable Product/Sum capability evidence) should be designed before P5 begins. O12 (while semantic identity) must be resolved before P6. O4 (FieldKey order) must be frozen before P4/P8 identity and metadata become durable. The remaining decisions can be resolved immediately before their owning plans without blocking P1/P2 groundwork.

The principal architectural guardrail remains:

> **Static Telos is ordinary Telos semantic code evaluated by a bounded deterministic semantic evaluator over immutable static values; it is not a second VM, a second pattern engine, or a second structural type system.**

If implementation pressure suggests adding a second Pack ontology, separate row relation, name-based variant resolver, runtime-method compile-time dispatch, or source-AST residual serializer, that is a signal to stop and reconnect the work to the existing canonical authority rather than proceed with the shortcut.
