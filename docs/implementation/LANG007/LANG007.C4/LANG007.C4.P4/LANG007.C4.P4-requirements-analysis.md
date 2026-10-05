---
id: LANG007.C4.P4-requirements-analysis
category: LANG
program: LANG007
checkpoint: LANG007.C4
plan: LANG007.C4.P4
kind: requirements-analysis
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
repository: aureat/phalcom-lang
branch: main
repository_revision_audited: b9f72cfe8ee5338485b716702cd9560776fce3f7
analysis_date: 2026-10-05
depends_on:
  - LANG007.C4.P2
parallel_with:
  - LANG007.C4.P3
blocks:
  - LANG007.C4.P5
---

# LANG007.C4.P4 — Generalized `FieldKey` and Unordered-Product Calculus

## Detailed implementation requirements analysis

---

# 0. Executive conclusion

`LANG007.C4.P4` is a **type-system/runtime structural migration**, not a small Record syntax extension.

The accepted C4 target generalizes unordered product fields from the repository's current string/Symbol-only model to the closed semantic key domain:

```telos
type FieldKey =
    Symbol | String | Int | Bool
```

with one language-defined canonical order:

```text
Symbol < String < Int < Bool
```

and with canonical within-kind ordering:

```text
Symbol  by stable spelling/content
String  by exact content
Int     by mathematical integer value
Bool    false < true
```

The live repository at `b9f72cfe8ee5338485b716702cd9560776fce3f7` demonstrates that this change cuts through all of the following established authorities:

```text
Record source syntax
    ↓
record type syntax / row-tail syntax
    ↓
canonical RecordRowData
    ↓
row solver / lacks constraints / inference
    ↓
substitution / type lambdas / specialization
    ↓
record subtyping / width / optionality
    ↓
StaticValue::Record
    ↓
semantic lowering / exact anonymous-product specs
    ↓
Record runtime construction and dynamic builders
    ↓
RecordProductShape / lookup / ProductStorage
    ↓
Record equality / hash / access / patterns
    ↓
stable type metadata / runtime type overlays
```

The central architectural requirement is therefore:

> **Introduce one semantic, content-owned `FieldKey` value authority and make every unordered-product layer consume it or a lossless projection of it. Do not let each subsystem invent its own string/int/bool key encoding.**

P4 must preserve the mature C3 row calculus rather than replacing it. The row solver, optionality rules, width typing, captured row tails, query budgets, substitution machinery, and record relation engine remain the authority. Their key coordinate changes from `Box<str>` to canonical `FieldKey`, and anonymous tails acquire an explicit **key domain** in addition to their existing value constraint.

The runtime likewise should not invent a second semantic key model. `phalcom-core` already depends on semantic products and can consume the pure semantic `FieldKey` representation for Record shapes and lowering. Runtime values are converted to/from that content representation at the VM boundary; VM `Symbol` IDs, heap `ObjRef`s, string object identity, and `LargeInt` object identity must never become structural Record key identity.

P4 is allowed to proceed while P3 remains partially implemented. The checkpoint DAG intentionally permits P3 and P4 in parallel after P2. Current `main` already contains substantial pushed P3 work, including `PackProductShape`, first-class runtime Pack objects, Pack lowering, and Pack/static-value additions. P4 must preserve that work, but the unresolved P3 root-vs-slice equality issue is not a semantic prerequisite for generalized Record rows. P5 remains blocked until **both** P3 and P4 are stable.

The most important implementation hazards are:

1. accidentally generalizing Tuple/Pack/call labels from `Symbol` to `FieldKey`;
2. storing VM or StaticValueStore-local identities in canonical row keys;
3. treating a runtime-computed Record key as a statically fixed field;
4. failing to add a key domain to anonymous row tails;
5. duplicating row lookup logic in reflection instead of converging existing subtyping/inference/pattern machinery;
6. silently stringifying heterogeneous keys in metadata;
7. making Record equality/order depend on source/presentation order;
8. preserving current O(n²) row/Record scans after introducing large heterogeneous shapes;
9. allowing `#x`, `"x"`, `1`, and `true` to collide because of host-language equality/coercion;
10. broadening `**Record` call forwarding to non-Symbol keys, which would violate the P3 ordered/labeled-lane contract.

This requirements analysis recommends a vertical P4 implementation with the following conceptual sequence:

```text
canonical FieldKey scalar
    ↓
Record AST/type-key and key-domain tail syntax
    ↓
row/store/solver/inference/substitution migration
    ↓
shared unordered lookup + relation convergence
    ↓
StaticValue::Record migration
    ↓
semantic lowering projection
    ↓
runtime Record shape/build/access/equality/hash
    ↓
record pattern migration
    ↓
metadata staging + compatibility/performance gates
```

This document is a requirements and architecture analysis. It is **not** the patch-grade execution plan and does not claim that P4 implementation has started.

---

# 1. Lifecycle and repository grounding

## 1.1 Repository state audited

Prepared against:

```text
repository: aureat/phalcom-lang
branch:     main
head:       b9f72cfe8ee5338485b716702cd9560776fce3f7
date:       2026-10-05
```

The live branch is thirteen commits ahead of the P2 closure revision used for initial P3 planning:

```text
P2 closure / P3 planning base:
    5a5bf8b218c2fbcfd785bfa41bb4eb7fec21c521

current audited main:
    b9f72cfe8ee5338485b716702cd9560776fce3f7
```

Recent pushed P3 history includes first-class Pack runtime/type work, product-shape changes, scalable label membership, and lifecycle continuation records. P4 planning therefore must use the **post-P3-partial** source tree rather than the earlier P2-only baseline.

## 1.2 Lifecycle authority

The C4 checkpoint remains:

```text
status:       IN_PROGRESS
completion:   PARTIAL
verification: UNVERIFIED
```

Current plan state:

| Plan | Scope | Status | Completion | Verification |
|---|---|---|---|---|
| `LANG007.C4.P1` | binding/mutability/field lifecycle | COMPLETE | IMPLEMENTED | BASELINE_BLOCKED |
| `LANG007.C4.P2` | static values/evaluator/closed const | COMPLETE | IMPLEMENTED | FOCUSED_TESTED |
| `LANG007.C4.P3` | first-class Pack/call/rest convergence | IN_PROGRESS | PARTIAL | UNVERIFIED |
| `LANG007.C4.P4` | generalized FieldKey/unordered products | PROPOSED | NOT_STARTED | UNVERIFIED |

The C4 dependency graph deliberately places P3 and P4 in parallel:

```text
P2
 ├──── P3 ────┐
 └──── P4 ────┤
              ▼
              P5
```

Consequences:

- P4 depends on **P2**, not on P3 completion.
- P4 must adapt to P3's already-pushed product/runtime edits.
- P4 must not consume unfinished P3 behavior as an architectural prerequisite.
- P5 must not begin its final structural reflection/capability work until both P3 and P4 provide stable shape authorities.

## 1.3 P3 state relevant to P4

Current P3 work has materially changed shared files that P4 will touch:

```text
phalcom-core/src/product/shape.rs
phalcom-core/src/product/mod.rs
phalcom-core/src/product/view.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/vm/dispatch.rs
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-type-meta/*
```

P3 has established or partially established:

- proper `TypeData::Pack` and Pack-from-pack forms;
- P2 `StaticValueStore` extension with `StaticValueData::Pack`;
- runtime `Object::Pack`;
- `ProductShape::Pack`;
- immutable Pack root/slice representation;
- `EmptyPack`;
- Pack builders/finalizers;
- rest/call spread convergence in covered paths;
- stable Pack type metadata additions.

P4 must **preserve** those additions while changing Record-specific branches. It must never “simplify” `ProductShape` back to Tuple/Record-only assumptions or generalize Pack labels to heterogeneous keys.

---

# 2. Normative authority

## 2.1 Ratified C4 FieldKey rule

The checkpoint ratifies:

```telos
type FieldKey = Symbol | String | Int | Bool
```

Canonical cross-kind order:

```text
Symbol < String < Int < Bool
```

Within kind:

```text
Symbol  canonical spelling/content
String  exact content
Int     numeric order
Bool    false < true
```

This order is language semantics. It is not:

- Rust enum declaration order by accident;
- discriminant order;
- VM `Symbol(u32)` order;
- StaticValueStore string/symbol ID order;
- hash-map iteration order;
- heap object address order;
- source/presentation field order.

## 2.2 Ordered labels remain Symbol

P4 changes **unordered-product fields** only.

The following remain Symbol-labeled:

```text
Tuple labeled entries
Pack labeled entries
method/call labels
rest labeled lane
selector slots
nominal constructor parameter labels
ordered product reflection labels
```

This is a hard boundary.

Therefore:

```text
Record field key      : FieldKey
Tuple/Pack/call label : Symbol
```

`String("x")` as a Record key is not a call label. `Int(1)` as a Record key is not a Pack label.

## 2.3 Existing optionality semantics remain authoritative

P4 does not redesign optionality.

Preserve:

```text
Required
Optional
```

and the corrected relation law:

```text
{ k?: S }  </:  { k: T }
```

even when `S <: T`.

P4 also must not reintroduce optional tuple entries.

## 2.4 Existing row calculus remains authoritative

C4 explicitly requires reuse of:

- `RecordRowId`;
- `RecordRowData`;
- `RecordRowTail`;
- `RecordRowSolver`;
- row inference;
- lacks constraints;
- substitution/instantiation;
- record subtype relation;
- query budget/cancellation;
- canonical TypeStore interning.

P4 migrates the coordinate system; it does not create a parallel “UnorderedRowStore.”

## 2.5 Public reflection is downstream

The C4 design names future source-visible structures:

```telos
Presence
FieldInfo
ProductField
FieldLookup
UnorderedTail
UnorderedShape
UnorderedShapeView
```

P4 owns the low-level calculus and reflection-ready canonical facts.

P5 owns:

- compiler-derived `Product` / `UnorderedProduct` evidence;
- public `ProductShape`;
- public `UnorderedShape` associated reflection;
- formation/caching of those static reflection values.

Therefore P4 should provide a reusable **internal unordered-field lookup/decomposition API** with semantics equivalent to the future `FieldLookup`, but it must not prematurely implement P5's capability system.

## 2.6 P8/P9 boundaries

P8 owns final durable static products, residual metadata, broad cross-module static durability, and final metadata/schema closure.

P9 owns final public ontology retirement/tooling/docs/performance checkpoint closure.

P4 nevertheless must:

- define a stable `FieldKeyRef` encoding shape;
- ensure type metadata has a non-lossy route for heterogeneous record keys;
- stage fingerprint/round-trip tests;
- fail closed rather than stringify non-Symbol keys if a legacy metadata path cannot yet encode them.

P4 must not remove the optimized/internal `RecordFields` kind merely for naming cleanliness; public retirement belongs to P9.

---

# 3. Current implementation audit

## 3.1 AST: Record type keys are still identifiers

Live `phalcom-ast/src/ast.rs`:

```rust
pub struct RecordTypeField {
    pub name: String,
    pub ty: TypeAnnotation,
    pub optional: bool,
    pub range: SourceRange,
}
```

`TypeAnnotationExpr::Record` carries:

```rust
Record {
    fields: Vec<RecordTypeField>,
    tail: Option<RecordRowTailSyntax>,
    range: SourceRange,
}
```

Current parser behavior requires an identifier for a record type field.

Therefore source types can express:

```telos
{
    name: String,
    age?: Int,
}
```

but not yet:

```telos
{
    ["mime"]: String,
    [42]: Handler,
    [true]: Enabled,
}
```

This is a real P4 syntax/AST gap.

## 3.2 AST: Record values already admit computed labels syntactically

Record literals use `RecordLiteralField { label: ProductLabel, ... }`.

`ProductLabel` supports:

```rust
Static { ... }
Computed { expr: Box<Expr>, ... }
```

so value syntax already admits bracket-computed labels.

However this shared AST type is also used for ordered Tuple labels.

That sharing is now dangerous: P4 must allow `FieldKey` in Record context while continuing to require `Symbol` in Tuple/Pack/call contexts.

Recommended direction:

```text
shared parser spelling may remain similar
but semantic/AST context must distinguish:

RecordKeySyntax
    vs
OrderedProductLabelSyntax / ProductLabel
```

A dedicated Record key AST is preferable to globally widening `ProductLabel`.

## 3.3 Parser: Record tails have no key domain

Current record type tails support:

```text
**R
**: T
...
**: ...
```

but not the C4 form:

```telos
**[String]: Handler
```

The current anonymous remainder models only a value constraint.

P4 requires a key-domain dimension.

## 3.4 Canonical rows are string-keyed

Live `phalcom-semantic/src/types/row.rs`:

```rust
pub struct RecordRowField<T = TypeId> {
    pub name: Box<str>,
    pub ty: T,
    pub presence: StructuralMemberPresence,
}

pub struct RecordRowData {
    pub fields: Box<[RecordRowField]>, // sorted by name, unique
    pub tail: RecordRowTail,
}
```

Formation:

```text
sort by field.name
reject equal name
binary-search by &str
```

This is the primary semantic migration point.

## 3.5 Anonymous row tails only carry value constraints

Current:

```rust
pub enum AnonymousRecordRemainder {
    Unconstrained,
    Typed(TypeId),
}
```

This can express:

```text
unknown extra fields, unknown values
unknown extra fields, values <: V
```

but cannot express:

```text
unknown extra String keys only
unknown extra Int keys only
unknown extra K keys with V values
```

It is therefore insufficient for the ratified `UnorderedTail` model.

## 3.6 Row solver is string-keyed end-to-end

`phalcom-semantic/src/types/row_solver.rs` currently has:

```rust
RecordRowLacks {
    row: RecordRowVarId,
    field: Box<str>,
}
```

and failures such as:

```text
LacksViolation { field: Box<str>, ... }
IncompatibleFields { field: Box<str>, ... }
MissingField { field: Box<str> }
ExtraField { field: Box<str> }
DuplicateField(Box<str>)
```

The solver:

- sorts by `field.name`;
- compares names;
- searches matching fields with repeated `.find()` / `.any()` scans;
- records lacks by string.

This must migrate to `FieldKey`.

The existing canonical sort makes a more efficient two-pointer/merge implementation possible and desirable.

## 3.7 Scoped type lambdas and inference duplicate the string key coordinate

Current representations include:

```rust
ScopedRecordField {
    name: Box<str>,
    ...
}

InferenceRecordField {
    name: Box<str>,
    ...
}

StableRecordRowLack {
    field: Box<str>,
    ...
}
```

These are not optional consumers: if only `RecordRowData` changes, generic/scoped row applications, solver lifting, substitution, and incremental replay become inconsistent.

P4 must migrate these together.

## 3.8 Record subtyping repeatedly scans by string

`check_record_row_subtype` currently:

- finds target/source correspondence by `name`;
- preserves required/optional checks;
- handles closed/typed/unconstrained/captured tails;
- checks extra source fields against target tail;
- checks tail-to-tail compatibility.

The algorithm uses nested scans despite rows being sorted.

P4 must replace string equality with FieldKey equality and extend tail compatibility to key domains.

The existing optionality/width logic should remain recognizable; a wholesale relation rewrite is unnecessary.

## 3.9 Semantic Record literal checking discards non-static key identity

The live checker currently normalizes labels through a helper equivalent to:

```rust
product_label_name(label) -> Option<Box<str>>
```

Only selected static Symbol forms yield a name.

For `RecordLiteralEntry::Field`:

- known Symbol label → fixed row field;
- computed label → value expression is checked, but there is no fixed row key;
- duplicates/lacks/expected-field lookup are string-based.

This means the parser already accepts more key syntax than the semantic Record type system can represent.

P4 must close this semantic hole deliberately.

## 3.10 StaticValue Record is still string-keyed

P2's store, now extended by P3 Pack work, still has:

```rust
struct StaticRecordField {
    key: Box<str>,
    value: StaticValueRef,
}
```

and:

```rust
intern_record(Vec<(Box<str>, StaticValueRef)>)
```

which sorts by string and fingerprints key bytes.

This must become FieldKey-based.

Important: P3's `StaticPack` remains Symbol-labeled and must not be generalized.

## 3.11 `StaticInt` is usable but poorly located for FieldKey

`StaticInt` is already a store-independent exact integer representation:

```rust
pub enum StaticInt {
    Small(i64),
    Large(Arc<BigInt>),
}
```

Its semantics are appropriate for `FieldKey::Int`.

However it currently lives inside `static_value/store.rs`.

A canonical FieldKey module should not depend conceptually on store-local string/symbol IDs.

Recommended architecture:

```text
extract/re-export exact semantic scalar primitives
    CanonicalInt / StaticInt
into a lower pure semantic module

FieldKey
    uses exact integer content

StaticValueStore
    uses the same exact integer type
```

Do not create a second BigInt normalization implementation.

## 3.12 Runtime Record shape is Symbol-only

Current `RecordProductShape` owns Symbol arrays and a Symbol lookup cache.

`finish_record` currently accepts:

```rust
Vec<(Symbol, Value)>
```

and canonicalizes labels using resolved Symbol spelling.

`RecordView` exposes Symbol labels and Symbol lookup.

`primitive/record.rs::rawLabelAt` returns a Symbol value.

`RecordLiteralBuilderObject` stores:

```rust
Vec<(Symbol, Value)>
```

VM Record bytecodes reject a key unless `label.symbol_value()` succeeds.

Every one of those contracts must change for heterogeneous Record keys.

## 3.13 Current runtime shape already separates presentation and logical order

This is valuable and should be preserved.

Current Record construction conceptually retains:

```text
presentation_labels
logical_labels
presentation_to_logical
logical-value storage
```

P4 should preserve the distinction:

- **presentation order** may remain observable by Record iteration/key-at operations;
- **logical order** is canonical FieldKey order for storage, equality, reflection, fingerprints, and lookup.

Canonical order must not retroactively become “the source order of a Record.”

## 3.14 Record source equality is currently quadratic

Universe `record.ph` compares each own field against a nested scan of the other Record.

This is currently tolerable for small Symbol-only records but conflicts with P4's large-shape performance requirements.

Once both Records carry canonical logical key order, equality can be O(n):

```text
same count
same canonical key at each logical index
same corresponding value
```

without making presentation order semantic.

## 3.15 Stable metadata is string-keyed

Current `phalcom-type-meta` nodes contain:

```rust
RecordFieldRef {
    name: Box<str>,
    ...
}

ScopedRecordFieldRef {
    name: Box<str>,
    ...
}
```

Fingerprints write field names as strings.

Runtime type overlays similarly contain:

```rust
RuntimeRecordField {
    name: Box<str>,
    ...
}
```

A non-Symbol Record row cannot be faithfully exported through this model.

P4 must stage a real `FieldKeyRef`; stringification is forbidden.

## 3.16 Record patterns are Symbol-name-only

Current:

```rust
RecordPatternEntry {
    label: String,
    pattern: Pattern,
    ...
}
```

Semantic resolution calls row lookup by string.

Compiler lowering builds `Value::symbol(...)` keys.

Record rest construction excludes matched fields by string.

Generalized Records require generalized fixed pattern keys.

This work belongs in P4's unordered calculus because P6 static matching must later consume the same resolved Record pattern facts.

## 3.17 Shared lowering helpers create a P3/P4 collision hazard

Several compiler/runtime paths currently share “labeled” helpers across:

- Record;
- Tuple;
- Pack;
- call argument expansion.

P4 must split Record-specific key handling where necessary.

The critical rule is:

```text
Record spread into Record:
    FieldKey keys allowed

Tuple/Pack labeled lane:
    Symbol labels only

Record spread into call **:
    permitted only for entries whose keys are Symbol
    non-Symbol key -> reject, never coerce
```

This is one of the most important P4/P3 interoperability requirements.

---

# 4. Canonical semantic `FieldKey` architecture

## 4.1 One pure semantic value type

Recommended canonical owner:

```text
phalcom-semantic/src/field_key.rs
```

Conceptual API:

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FieldKey {
    Symbol(Arc<str>),
    String(Arc<str>),
    Int(CanonicalInt),
    Bool(bool),
}
```

Exact private storage may use `Box<str>` instead of `Arc<str>` if profiling favors it, but semantics must be content-owned.

Properties:

- no `Symbol` VM ID;
- no `ObjRef`;
- no `StaticStringId`;
- no `StaticSymbolId`;
- no StaticValueStore lineage;
- no source range;
- no arena address;
- no `TypeId` embedded in a concrete key.

The representation must be usable by:

- canonical rows;
- row solver;
- static Record values;
- semantic lowering projections;
- stable metadata exporter;
- diagnostics;
- runtime Record shape after lossless projection/reuse.

## 4.2 Symbol versus String are distinct variants

Required:

```text
FieldKey::Symbol("x") != FieldKey::String("x")
```

No implicit conversion.

Diagnostics should make this visually obvious:

```text
#x
"x"
```

rather than formatting both as `x`.

## 4.3 Int versus Bool are distinct variants

Required:

```text
FieldKey::Int(1) != FieldKey::Bool(true)
FieldKey::Int(0) != FieldKey::Bool(false)
```

Host-language equality conventions must not leak into Telos.

## 4.4 Int is arbitrary precision and normalized

The key `1` must have one semantic identity regardless of how the runtime value was represented.

Use the same normalization law as P2 static integers:

```text
small representable integer -> canonical small form
otherwise                   -> exact BigInt form
```

Comparisons are mathematical numeric comparisons.

Do not hash or order by decimal source spelling.

## 4.5 Exact String/Symbol content, no Unicode normalization

Recommended deterministic content ordering:

```text
lexicographic order of exact UTF-8 content
```

No NFC/NFD or locale transformation.

This matches Telos String content semantics and avoids hidden normalization cost.

For valid UTF-8 this yields a deterministic language-level order independent of process locale.

## 4.6 Total canonical order

Implement explicit comparison:

```text
kind_rank(Symbol) = 0
kind_rank(String) = 1
kind_rank(Int)    = 2
kind_rank(Bool)   = 3
```

Then compare payload according to kind.

Do not derive semantic ordering from enum declaration order alone. A derived Rust `Ord` is acceptable only if guarded by an explicit manual representation/rank contract and tests, but an explicit comparator is safer.

Recommended:

```rust
impl FieldKey {
    pub fn canonical_cmp(&self, other: &Self) -> Ordering;
}
```

All row canonicalization must call this authority.

## 4.7 Stable hash/fingerprint encoding

Define explicit stable bytes conceptually:

```text
Symbol: tag 0 + UTF-8 length + UTF-8 bytes
String: tag 1 + UTF-8 length + UTF-8 bytes
Int:    tag 2 + sign + canonical magnitude bytes
Bool:   tag 3 + 0/1
```

This stable encoding owns:

- semantic fingerprints;
- metadata structural fingerprints;
- incremental product fingerprints that include Record shape;
- deterministic test expectations.

Rust's `Hash` may remain useful for process-local maps, but it is not the durable fingerprint format.

## 4.8 Display

Provide one canonical diagnostic renderer:

```text
Symbol(#foo) -> #foo
String(foo)  -> "foo"
Int(42)      -> 42
Bool(true)   -> true
```

Escaping must be deterministic for String/Symbol spellings.

Every duplicate/missing/lacks diagnostic should consume this helper instead of reconstructing text.

---

# 5. Source syntax and AST requirements

## 5.1 Record key syntax must be context-specific

Do not widen the shared ordered-product label type globally.

Preferred normalized AST:

```rust
pub enum RecordKeySyntax {
    Symbol {
        symbol: SymbolLiteralKind,
        range: SourceRange,
    },
    Computed {
        expr: Box<Expr>,
        range: SourceRange,
    },
}
```

Then:

```rust
pub struct RecordLiteralField {
    pub key: RecordKeySyntax,
    pub value: Expr,
    pub range: SourceRange,
}
```

and record type fields use the same key syntax or an equivalent static-only key syntax:

```rust
pub struct RecordTypeField {
    pub key: RecordKeySyntax,
    pub ty: TypeAnnotation,
    pub optional: bool,
    pub range: SourceRange,
}
```

Tuple/Pack labels continue using Symbol-only `ProductLabel`.

A mechanically smaller implementation may retain `ProductLabel` in the AST but must introduce a Record-specific semantic normalization layer. What is forbidden is treating “ProductLabel now accepts FieldKey” as a global rule.

## 5.2 Required fixed Record type forms

P4 should accept:

```telos
{
    name: String,          // Symbol key #name
    ["mime"]: String,      // String key
    [42]: Handler,         // Int key
    [true]: Enabled,       // Bool key
}
```

Bare field identifiers are Symbol keys.

Explicit Symbol literals should also be representable where grammar already supports them.

## 5.3 Fixed key expressions in a type are static

Inside a Record **type**, bracketed key expression is not a runtime expression.

It must statically reduce to one `FieldKey`.

Examples:

```telos
{ ["x"]: Int }
{ [42]: String }
{ [constKey]: Value }   // only when constKey has a reduced static FieldKey value
```

A runtime `let` cannot be used to determine a structural type key.

P4 must reuse P2's static-value/checked-static authority. It must not introduce an ad hoc “evaluate simple key expression” interpreter in type annotation code.

## 5.4 Symbolic value-dependent keys

C4 architecture anticipates generic/static keys that may remain symbolic before specialization.

P4 must **not** solve this by inventing a second residual expression graph inside `RecordRowData`.

Recommended boundary:

```text
concrete/reduced FieldKey
    -> may enter canonical RecordRowData

symbolic static key
    -> retain through checked static-definition/static-expression authority
       and residualize/block as appropriate;
       P7 owns general normalized residual computation

runtime key
    -> never enters fixed row fields
```

Where an existing P2/P3 checked static application can reduce the key during specialization, the final materialization step must call the ordinary row constructor and re-check duplicate keys.

This guarantees collision-after-specialization diagnostics without introducing `FieldKeyExpr` as a parallel residual system.

## 5.5 Tail syntax

Required explicit typed-key tail:

```telos
{
    id: Int,
    **[String]: Handler,
}
```

Recommended generalized anonymous-tail grammar:

```telos
**[K]: V
**[K]: ...
**: V
...
```

Recommended meaning:

```text
**[K]: V  -> Typed(keyType=K, valueType=V)
**[K]: ...-> Unconstrained(keyType=K)
**: V     -> Typed(keyType=FieldKey, valueType=V)
...       -> Unconstrained(keyType=FieldKey)
```

Captured tail remains:

```telos
**R
```

where `R` has the row/RecordFields kind and preserves exact correlated structure.

### Why default anonymous tails should use `FieldKey`

Before P4 the only representable key universe was Symbol, so the distinction was unobservable.

After P4, the natural generalized meaning of “any additional Record field” is “any FieldKey.”

Users who need the old restricted domain can write:

```telos
**[Symbol]: V
```

This is the recommended language rule for the implementation plan to freeze. It is an architectural recommendation derived from the generalized Record model; if the language owner wants legacy anonymous tails to remain Symbol-only, that must be an explicit amendment before implementation because it affects subtyping and metadata.

## 5.6 Tail key domain validation

`K` in:

```telos
**[K]: V
```

must be a proper type provably within `FieldKey` or dynamically admissible under the language's normal type-boundary policy.

For canonical static type formation, prefer a compile-time diagnostic when `K` is proven incompatible:

```text
record.key_domain_not_field_key
```

Do not let an arbitrary `Object` key domain silently become a FieldKey tail.

---

# 6. Canonical row representation

## 6.1 Migrate field coordinate from name to key

Target:

```rust
pub struct RecordRowField<T = TypeId> {
    pub key: FieldKey,
    pub ty: T,
    pub presence: StructuralMemberPresence,
}
```

Canonical row:

```rust
pub struct RecordRowData {
    pub fields: Box<[RecordRowField]>, // canonical FieldKey order, unique
    pub tail: RecordRowTail,
}
```

Formation:

```text
sort once using FieldKey::canonical_cmp
reject adjacent duplicate key
intern canonical row
```

`find_field` becomes:

```rust
pub fn find_field(&self, key: &FieldKey) -> Option<TypeId>
```

using binary search.

## 6.2 Do not preserve a shadow `name` field

Avoid:

```rust
key: FieldKey,
name: Option<Box<str>>,
```

in canonical rows.

That creates dual identity for Symbol fields and invites consumers to keep using `name`.

Presentation/source text belongs in diagnostics/provenance, not canonical row identity.

## 6.3 Anonymous tails gain key type

Recommended:

```rust
pub enum AnonymousRecordRemainder {
    Unconstrained {
        key_type: TypeId,
    },
    Typed {
        key_type: TypeId,
        value_type: TypeId,
    },
}
```

or an equivalent compact representation.

The names are mechanically flexible; the two semantic coordinates are not.

## 6.4 Captured tails retain existing identities

Keep:

```rust
RecordRowTail::Parameter(TypeParameterId)
RecordRowTail::Projection(TypeId)
```

for correlated/captured rows.

Do not flatten a captured tail into:

```text
keyType union + valueType union
```

because doing so loses key/value correlation.

## 6.5 Row interning identity

Two closed rows with identical key→(type,presence) mappings but different source order must intern to the same canonical row.

Required:

```text
{ #a: Int, "b": String }
{ "b": String, #a: Int }
```

→ same canonical RecordRowId, assuming exact type/presence equality.

Source order is not type identity.

---

# 7. Row solver and inference requirements

## 7.1 Lacks constraints use FieldKey

Target:

```rust
pub struct RecordRowLacks {
    pub row: RecordRowVarId,
    pub field: FieldKey,
}
```

and:

```rust
StableRecordRowLack {
    parameter: TypeParameterId,
    field: FieldKey,
}
```

All failure witnesses use `FieldKey`.

## 7.2 Unification uses canonical key merge

Rows are already canonical sorted sequences.

Replace repeated:

```text
find by key
any by key
```

with a two-pointer merge where practical:

```text
left keys  ─┐
            ├─ O(n + m) correspondence / left-only / right-only
right keys ─┘
```

This provides:

- deterministic duplicate handling;
- less allocation;
- no accidental O(n²) on 10k-field records;
- simple support for heterogeneous ordering.

The row solver remains responsible for payload equations and tail variable binding.

## 7.3 Preserve payload inference authority

Only the key coordinate changes.

Do not add FieldKey-specific type-variable inference.

Field value types continue to use existing `InferenceTerm`, bounds, proof state, and `RecordRowSolver<V>` payload mechanisms.

## 7.4 Lacks propagation

When extending a row variable with key `K`, existing lacks propagation rules apply to exact `FieldKey` equality.

Distinct keys:

```text
#x
"x"
```

must generate independent constraints.

## 7.5 Anonymous key-domain tails are rigid shape constraints

An anonymous tail:

```text
Typed(K,V)
```

does not create an enumerable set of fields or a row variable.

It means:

```text
zero or more unknown fields
each key conforms to K
each value conforms to V
fixed row keys excluded by structural uniqueness
```

The solver must not invent concrete keys from it.

## 7.6 Generic/scoped representation migration

Migrate all of:

```text
ScopedRecordField
InferenceRecordField
RecordRowTerm
RecordRowLacks
StableRecordRowLack
row-solver failure witnesses
```

to `FieldKey`.

Scoped anonymous remainders must also carry scoped/canonical key-domain types as needed.

If a key-domain type can reference a type-lambda binder, the scoped remainder representation must be capture-safe in the same manner as its value type.

## 7.7 Substitution and specialization

Every path that rebuilds a row must call the same canonical checked formation routine.

This is essential for:

- canonical ordering after substitution;
- duplicate detection;
- tail key/value validation;
- collision diagnostics;
- deterministic IDs/fingerprints.

Do not mutate a canonical row's fields in place.

---

# 8. Structural relation semantics

## 8.1 Fixed-field matching

Fixed fields match by exact `FieldKey` equality.

Required examples:

```text
#x       matches #x
"x"      matches "x"
1        matches 1
true     matches true

#x       does not match "x"
1        does not match true
```

## 8.2 Required/optional rules remain unchanged

For a fixed target field:

```text
target Required(T)
    source must prove Required(S) with S <: T

target Optional(T)
    source may:
      - have Required(S), S <: T
      - have Optional(S), S <: T
      - omit it when the source proves it cannot violate T
```

Open-tail cases must be considered using the key domain.

## 8.3 Concrete key versus tail domain

Introduce one canonical helper conceptually:

```rust
enum KeyDomainRelation {
    ProvenInside,
    ProvenOutside,
    UnknownOrBlocked,
}
```

implemented through existing type relations:

```text
typeOf(FieldKeyValue) <: keyType
```

not by checking type names.

For a concrete key:

```text
#x    : Symbol
"x"   : String
42    : Int
true  : Bool
```

If membership is refuted, the tail cannot contain that key.

If membership is blocked/unknown, relation/lookup must preserve uncertainty.

## 8.4 Extra source fields against target typed tail

For source fixed field `(k:S)` absent from target fixed fields:

Target:

```text
Closed
    -> reject

Unconstrained(K)
    -> require key k to be accepted by K

Typed(K,V)
    -> require k accepted by K
       and S <: V

captured Parameter/Projection
    -> use existing correlation/proof machinery;
       block when unresolved rather than guessing
```

## 8.5 Tail-to-tail subtype rules

The semantic principle is set inclusion of possible fields.

Recommended rules:

```text
Closed <: any compatible open target

Typed(Ks, Vs) <: Typed(Kt, Vt)
    when Ks <: Kt and Vs <: Vt

Typed(Ks, Vs) <: Unconstrained(Kt)
    when Ks <: Kt

Unconstrained(Ks) <: Unconstrained(Kt)
    when Ks <: Kt

Unconstrained(_) </: Typed(_, _)

open source </: Closed target
```

Captured tails preserve identity/correlation rules already present.

Unknown relation outcomes remain blocked/uncertain; they are never coerced to false merely for convenience.

## 8.6 Optional target field absent from fixed source

For an optional target key `k:T` absent from source fixed fields:

- closed source tail → acceptable absence;
- source typed tail whose key domain excludes `k` → acceptable absence;
- source typed tail that may contain `k` → its value type must be compatible with `T`;
- source unconstrained compatible-key tail → cannot prove value compatibility, so relation fails/blocks according to existing relation policy;
- captured unresolved tail → preserve existing blocked behavior.

This is the key-domain generalization of the current optional-row fix.

## 8.7 Keep query budget/cancellation

Do not add an unbounded recursive helper for key-domain/tail comparison.

All nested type relation calls must reuse:

```text
QueryBudget
CancellationToken
visited pair/cycle state
```

from the current relation engine.

---

# 9. Canonical unordered-field lookup

## 9.1 One shared lookup semantic

P4 should establish an internal semantic lookup result equivalent to:

```telos
FieldLookup::Required(Type)
FieldLookup::Optional(Type)
FieldLookup::Absent
FieldLookup::Unknown
```

Exact Rust shape is flexible.

It should be consumed by:

- expected Record field typing;
- record-pattern checking;
- future P5 `UnorderedShape::Lookup`;
- structural matching;
- relation helpers where applicable;
- static structural operations later.

Do not let P5 implement a second lookup.

## 9.2 Fixed fields

```text
fixed Required(T) -> Required(T)
fixed Optional(T) -> Optional(T)
```

## 9.3 Closed tail

Absent fixed key + Closed:

```text
Absent
```

## 9.4 Typed anonymous tail

For `Typed(K,V)` and absent fixed key `k`:

```text
k proven outside K
    -> Absent

k proven inside K
    -> Optional(V)
       because the field may be absent, but if present its value is V-constrained

membership uncertain
    -> Unknown
```

This is more precise than collapsing every open tail to Unknown and fits the public `Optional(Type)` variant.

If the language owner wants `Optional(Type)` to mean only syntactically declared optional fields, then typed-tail lookup should return `Unknown`; that would be a less precise but sound policy. The recommended implementation uses `Optional(V)` because the public data shape naturally denotes “may be present with V.”

## 9.5 Unconstrained tail

For `Unconstrained(K)`:

```text
k outside K -> Absent
otherwise   -> Unknown
```

because there is no common value type.

## 9.6 Captured tails

If a captured tail can be resolved through canonical row identity, recursively use the same lookup.

If it remains a parameter/projection without exact shape, return Unknown/blocked semantic evidence rather than inventing a type.

---

# 10. Static Record requirements

## 10.1 StaticValue Record keys become FieldKey

Target:

```rust
struct StaticRecordField {
    key: FieldKey,
    value: StaticValueRef,
}
```

and:

```rust
intern_record(Vec<(FieldKey, StaticValueRef)>)
```

## 10.2 Canonicalization

`intern_record` must:

1. validate child refs;
2. canonical-sort by `FieldKey::canonical_cmp`;
3. reject duplicate exact FieldKeys;
4. intern canonical field sequence.

Static Record equality then remains order-independent naturally.

## 10.3 Fingerprint

Record fingerprint must write:

```text
FieldKey stable encoding
StaticValue child fingerprint
```

in canonical key order.

Do not hash a String/Symbol store-local ID.

## 10.4 Static key conversion

Provide a pure conversion conceptually:

```rust
StaticValueStore::field_key_of(value) -> Result<FieldKey, NotAFieldKey>
```

Rules:

```text
StaticValueData::Symbol -> FieldKey::Symbol(spelling content)
StaticValueData::String -> FieldKey::String(content)
StaticValueData::Int    -> FieldKey::Int(exact normalized integer)
StaticValueData::Bool   -> FieldKey::Bool
anything else           -> invalid FieldKey
```

The returned FieldKey is independent of the StaticValueStore lineage.

## 10.5 Static evaluator

When evaluating a static Record field key:

```text
evaluate key expression
    ↓
convert StaticValue -> FieldKey
    ↓
evaluate/retain field value according to source evaluation law
    ↓
intern canonical Record
```

P2 evaluator budget/cancellation remains in force.

Do not call runtime VM conversion.

## 10.6 P3 Pack isolation

`StaticPackBacking.labels` remain `StaticSymbolId`.

No P4 change should alter Pack label equality, ordering, or fingerprint semantics.

---

# 11. Runtime Record key representation

## 11.1 Prefer semantic FieldKey reuse

Because `FieldKey` is a pure content-owned semantic scalar and `phalcom-core` already consumes `phalcom-semantic`, the cleanest authority is:

```text
phalcom-semantic::FieldKey
        ↓ lossless use/projection
phalcom-core RecordProductShape
```

This avoids a second `RuntimeFieldKey` comparator/hash definition.

If a core-local wrapper is mechanically required, it must be a lossless projection with semantic comparison delegated to the canonical FieldKey rules.

## 11.2 Runtime Value -> FieldKey conversion

VM conversion:

```text
Symbol Value
    -> resolve spelling
    -> FieldKey::Symbol(content)

String object
    -> immutable string contents
    -> FieldKey::String(content)

Int immediate
    -> canonical exact Int

LargeInt object
    -> canonical exact Int

Bool immediate
    -> FieldKey::Bool

other
    -> structured runtime type error
```

No ObjRef survives in the key.

## 11.3 FieldKey -> runtime Value conversion

Needed for:

- Record key iteration;
- `keyAt`;
- diagnostics/source APIs.

Projection:

```text
Symbol -> intern spelling -> Symbol Value
String -> allocate/materialize String value
Int    -> immediate or LargeInt
Bool   -> Bool
```

This projection may allocate String/LargeInt values when keys are observed. That is acceptable as an initial correctness floor because structural key identity remains non-heap. Do not add a permanently rooted key-object cache without performance evidence.

## 11.4 Invalid computed key timing

Current compiler emits a key guard before the field value expression for computed product labels.

P4 must preserve the important evaluation law:

```text
evaluate key
validate FieldKey category
only then evaluate field value
```

Add/use a Record-specific guard such as conceptually:

```text
GuardFieldKey
```

Do **not** replace Tuple/Pack's `GuardSymbol`.

## 11.5 Duplicate keys never overwrite

All construction routes must reject duplicate structural Record keys.

Required routes:

- fixed literal;
- computed literal;
- Record expansion;
- semantic `BuildStaticRecord`;
- static-value materialization later;
- pattern rest reconstruction where applicable.

No last-write-wins Record semantics.

---

# 12. Runtime `RecordProductShape`

## 12.1 Recommended memory layout

Current shape stores both presentation and logical Symbol arrays.

With content-heavy String/BigInt keys, duplicating keys becomes more expensive.

Recommended P4 representation:

```rust
pub struct RecordProductShape {
    pub logical_keys: Box<[FieldKey]>,
    pub presentation_to_logical: Box<[u32]>,
}
```

Optional inverse mapping may be added only if required by hot presentation lookup.

Presentation key `i` is:

```text
logical_keys[presentation_to_logical[i]]
```

This keeps one owned copy of each structural key.

## 12.2 Canonical logical order

`logical_keys` must be sorted exactly by `FieldKey::canonical_cmp`.

This order owns:

- logical storage slots;
- structural shape equality;
- canonical reflection iteration later;
- O(log n) lookup;
- deterministic fingerprints.

## 12.3 Presentation order

`presentation_to_logical` preserves source/runtime encounter order.

Therefore two Records can have:

```text
same mapping
different presentation order
```

and still be structurally equal while potentially producing different presentation iteration order if that surface remains intentionally observable.

P4 must not silently change the existing Record presentation contract.

## 12.4 Lookup

Minimum required performance:

```text
O(log n) binary search in logical_keys
```

with no String allocation during comparison.

An O(1) cache is optional if measurements justify it.

The current `HashMap<Symbol,u32>` cache must not force P4 to intern String/Int/Bool keys into a second runtime key pool.

## 12.5 ProductShape registry

`ProductShape::Pack` introduced by P3 remains separate.

Required:

```text
ProductShape::Tuple  -> Symbol labels
ProductShape::Pack   -> Symbol labels
ProductShape::Record -> FieldKey keys
```

No generic `ProductKey` enum should erase this distinction merely to reduce code.

---

# 13. Runtime Record construction/lowering

## 13.1 Fixed `BuildRecord`

Current VM pops `(label,value)` pairs and requires Symbol.

P4 changes Record-only behavior:

```text
pop key Value
convert/validate FieldKey
pop value pairing
finish_record
```

The bytecode name can remain `BuildRecord`.

Tuple `BuildTuple` remains Symbol-labeled.

## 13.2 Dynamic Record literal builder

Current builder:

```rust
Vec<(Symbol, Value)>
```

becomes conceptually:

```rust
Vec<(FieldKey, Value)>
```

or stores raw key Values only if GC/lifetime/error-timing remain correct.

Preferred: convert to FieldKey before insertion so builder is independent of key object lifetimes.

The builder remains VM-private.

## 13.3 Static analyzed Record lowering

Current:

```rust
AnonymousProductConstructionKind::Record {
    presentation_labels: Box<[Box<str>]>,
    logical_labels: Box<[Box<str>]>,
    source_to_logical: Box<[u32]>,
}
```

must carry heterogeneous keys.

Preferred transport:

```rust
Record {
    logical_keys: Box<[FieldKey]>,
    source_to_logical: Box<[u32]>,
}
```

Source/presentation keys can be recovered through the permutation; duplicating them is unnecessary unless a specific compiler consumer needs them.

This spec is a lowering projection, not a second semantic authority.

## 13.4 `finish_record_from_spec`

Must:

- validate permutation length/bounds/uniqueness;
- use canonical logical keys from the checked semantic spec;
- build logical ProductStorage;
- retain exact runtime type recipe;
- never re-sort source labels using runtime spellings;
- never reinterpret a key string.

## 13.5 Record expansion

Record literal `**source` requires context-specific handling.

For source Record:

```text
all FieldKey entries preserved
```

For source Tuple/Pack labeled lane:

```text
Symbol labels become FieldKey::Symbol
```

Do not change Tuple/Pack lane types.

Map expansion remains governed by the current Record spread semantic policy; P4 should not silently make arbitrary runtime Map key sets structural Record types.

---

# 14. P3/P4 interoperability: call and ordered-label boundaries

## 14.1 Heterogeneous Record -> call labeled spread

Current calls can consume `**record` because all Record labels are Symbols.

After P4 this requires a boundary.

Rule:

```text
for each Record entry used as a call label:
    if key is Symbol:
        forward as call label
    else:
        reject
```

Known heterogeneous Record shape should be rejected semantically where possible.

Dynamic/unknown Record shape must be checked at runtime during expansion.

Never coerce:

```text
"timeout" -> #timeout
1         -> #1
```

## 14.2 Heterogeneous Record -> Pack/Tuple labeled construction

Same rule when a source is consumed as an **ordered labeled lane**:

```text
FieldKey::Symbol only
```

Record-to-Record expansion is the only generalized-key structural path.

## 14.3 Pack/Tuple -> Record

This direction is safe:

```text
Symbol label -> FieldKey::Symbol
```

No semantic information is lost.

## 14.4 Shared helper split

Any current helper named generically like:

```text
snapshot_labeled_lane
compile_product_label
```

must be audited for whether it means:

```text
ordered Symbol lane
```

or:

```text
unordered Record key/value entries
```

If it serves both, split the APIs.

This prevents P4 from mutating P3 call behavior accidentally.

---

# 15. Record access, equality, and hash

## 15.1 `rawLabelAt` can no longer promise Symbol

The source runtime floor must expose a heterogeneous key.

Preferred terminology:

```text
rawKeyAt
keyAt
```

returning `FieldKey`.

P9 may own final public naming cleanup, but P4 must provide a semantically correct heterogeneous observation path.

A compatibility `rawLabelAt` may exist only for Symbol-specific internal callers and must fail/reject non-Symbol fields rather than lie.

## 15.2 Fast lookup primitive

Current source `Record.get` linearly scans keys.

P4 should expose an internal native primitive backed by canonical shape lookup, conceptually:

```text
rawLookup(key)
rawLogicalIndexOf(key)
rawValueForKey(key)
```

Exact API is flexible.

Goal:

```text
Record.get(key) -> O(log n) baseline
```

without allocation for key comparison.

## 15.3 Equality

Recommended O(n) structural equality:

```text
if sizes differ -> false

for logical index i:
    compare canonical FieldKey
    compare corresponding value by Telos ==
```

Because both logical key arrays use the same total canonical order, no nested lookup is required.

Presentation order is ignored.

## 15.4 Hash

Hash in canonical logical order:

```text
record-domain tag
field count
for each logical field:
    FieldKey semantic hash
    value semantic hash
```

Equal Records therefore hash equally independent of presentation order.

Do not recursively reflect key/value types.

## 15.5 EmptyRecord

`EmptyRecord` remains the immediate zero-field Record.

No P4 heap allocation is required for empty Record.

---

# 16. Record patterns

## 16.1 Pattern AST

Generalize:

```rust
RecordPatternEntry {
    label: String,
    ...
}
```

to a static Record-key pattern form.

Required examples:

```telos
{ name: x }
{ ["mime"]: x }
{ [42]: x }
{ [true]: x }
```

Pattern keys must statically reduce to concrete FieldKey.

An arbitrary runtime key expression is not a fixed structural pattern key.

## 16.2 Semantic pattern resolution

`ResolvedRecordFieldPattern` should carry canonical `FieldKey`, not source string.

Row lookup must use the shared unordered lookup authority.

Pattern mismatch diagnostics render canonical FieldKey.

## 16.3 Rest pattern

For:

```telos
{ [42]: x, **rest }
```

the rest Record excludes exactly `FieldKey::Int(42)`.

Do not exclude a Symbol/String key with the same presentation text.

The reconstructed rest Record preserves remaining entries and normal Record presentation rules.

## 16.4 Compiler lowering

Executable Record pattern specs carry a lossless FieldKey projection.

The compiler materializes/looks up runtime keys through the same VM FieldKey conversion authority.

Map pattern keys remain Map semantics and must not be merged with Record structural keys merely because their literal sets overlap.

## 16.5 P6 dependency

P6's static matcher must consume these already-resolved FieldKey pattern products.

P6 must not re-resolve record key expressions.

---

# 17. Metadata and runtime typing staging

## 17.1 Stable metadata key type

Define:

```rust
pub enum FieldKeyRef {
    Symbol(Box<str>),
    String(Box<str>),
    Int(CanonicalIntRef),
    Bool(bool),
}
```

with explicit stable tags.

The exact Int metadata encoding should be canonical sign+magnitude or another exact deterministic BigInt encoding.

## 17.2 Record metadata fields

Eventually:

```rust
RecordFieldRef {
    key: FieldKeyRef,
    ty: TypeNodeId,
    presence: StructuralMemberPresenceRef,
}
```

and scoped equivalents.

Do not retain `name` as a second canonical key.

## 17.3 Anonymous remainder metadata

Metadata must eventually encode:

```text
Unconstrained(keyType)
Typed(keyType, valueType)
```

rather than the current:

```text
Unconstrained
Typed(valueType)
```

## 17.4 P4 versus P8 schema responsibility

P4 should stage the stable nodes and exporter/importer/fingerprint logic needed to prove the representation.

P8 owns the final checkpoint-wide durable schema release.

Until a schema path can encode heterogeneous keys:

```text
non-Symbol row export must fail closed
```

It must not:

- stringify;
- convert String to Symbol;
- truncate BigInt;
- serialize VM IDs;
- silently omit fields.

## 17.5 Legacy Symbol rows

A future versioned decoder may map legacy v10:

```text
RecordFieldRef.name = "x"
```

to:

```text
FieldKeyRef::Symbol("x")
```

This is the only sound legacy interpretation.

## 17.6 Runtime overlay typing

Migrate:

```rust
RuntimeRecordField {
    name: Box<str>,
    ...
}
```

to a heterogeneous stable key representation.

Runtime type environments must not collapse non-Symbol key information.

---

# 18. Incremental and fingerprint requirements

## 18.1 Row identity fingerprints

Any fingerprint that currently writes `field.name` must write the stable FieldKey encoding.

Source order must not affect canonical row fingerprint.

## 18.2 Static Record fingerprint

Likewise canonical key order.

## 18.3 Lowering product fingerprints

Anonymous product lowering specs must include exact FieldKey content and canonical permutation.

## 18.4 Targeted invalidation

Required incremental discriminators should prove:

- changing a field key changes the owning row/type product;
- changing `#x` to `"x"` invalidates even with same text;
- reordering fixed Record type fields does not change canonical type identity/fingerprint;
- changing a field value type invalidates normally;
- changing only unrelated declaration/body does not invalidate the row product;
- changing typed-tail key domain invalidates the row/type and dependents;
- a P4 key change does not invalidate unrelated P3 Pack shape products.

## 18.5 No process-local identity in durable hashes

Forbidden fingerprint inputs:

```text
Symbol(u32)
StaticStringId
StaticSymbolId
ObjRef
HashMap iteration sequence
pointer address
```

---

# 19. Diagnostics

Recommended diagnostic families:

```text
record.invalid_field_key
record.key_not_static
record.duplicate_field_key
record.key_domain_not_field_key
record.row_lacks_unproven
record.row_key_domain_mismatch
record.spread_non_symbol_call_key
record.pattern_key_not_static
record.metadata_key_unrepresentable
```

Diagnostics should show keys unambiguously:

```text
duplicate Record field key `"x"`
duplicate Record field key `#x`
```

not both as `x`.

For computed keys, include the source range of the key expression and, when known, the first conflicting field.

Collision after specialization must point to both originating key sites when provenance is available.

---

# 20. Performance and memory requirements

## 20.1 Semantic row formation

Target:

```text
canonicalization: O(n log n)
duplicate detection after sort: O(n)
lookup: O(log n)
```

Avoid a sort followed by repeated linear duplicate scans.

## 20.2 Row solver

Target correspondence for two fixed row prefixes:

```text
O(n + m)
```

after canonicalization.

Do not preserve the current nested `.find()` / `.any()` behavior for large rows.

## 20.3 Runtime Record formation

Target:

```text
validate/convert keys: O(n)
canonical sort/permutation: O(n log n)
storage materialization: O(n)
```

No permanent per-Record hash map is required if shape-level binary search meets lookup goals.

## 20.4 Runtime lookup

Baseline:

```text
O(log n)
```

allocation-free for comparison.

Optional shape-level O(1) cache may be retained if it supports borrowed heterogeneous lookup without per-query String/BigInt allocation.

## 20.5 Equality/hash

Target:

```text
Record equality: O(n) logical walk
Record hash:     O(n)
```

Do not use nested presentation scans.

## 20.6 Key storage

Avoid owning two copies of large String/BigInt keys per `RecordProductShape`.

Prefer:

```text
one logical key array
+ compact permutation(s)
```

## 20.7 Synthetic gates

Include 1k and 10k mixed-key row/shape formation tests/bench discriminators.

They do not need microbenchmark statistical infrastructure if deterministic operation-count or coarse regression evidence is easier, but they must detect accidental O(n²) algorithms.

---

# 21. Required invariants

The patch-grade plan should number and preserve at least the following.

```text
P4-INV-01
    FieldKey is exactly Symbol | String | Int | Bool.

P4-INV-02
    FieldKey semantic identity is content/value based and independent of VM,
    StaticValueStore, source range, and arena identity.

P4-INV-03
    Cross-kind canonical order is Symbol < String < Int < Bool.

P4-INV-04
    Symbol and String with identical text are distinct keys.

P4-INV-05
    Int and Bool are distinct keys.

P4-INV-06
    Int keys use exact arbitrary-precision numeric identity.

P4-INV-07
    Record canonical fields are sorted exactly once by FieldKey canonical order.

P4-INV-08
    Duplicate Record keys are invalid; no construction route overwrites.

P4-INV-09
    Record source/presentation order is not structural type/equality identity.

P4-INV-10
    RecordRow optionality semantics are unchanged.

P4-INV-11
    Open source rows cannot satisfy closed targets merely by key-domain widening.

P4-INV-12
    Anonymous Record tails carry key-domain information.

P4-INV-13
    Captured row tails preserve exact correlation and are not reduced to key/value unions.

P4-INV-14
    Anonymous open tails are not finite iterable field sequences.

P4-INV-15
    Tuple labels remain Symbol.

P4-INV-16
    Pack labels remain Symbol.

P4-INV-17
    Call labels remain Symbol.

P4-INV-18
    Non-Symbol Record keys cannot be forwarded as call labels.

P4-INV-19
    Record-to-Record expansion preserves all FieldKey kinds.

P4-INV-20
    Pack/Tuple-to-Record labeled expansion maps Symbol labels to Symbol FieldKeys only.

P4-INV-21
    StaticValue::Record uses canonical FieldKey, not store-local text IDs.

P4-INV-22
    Runtime Record shape identity contains no heap object identity.

P4-INV-23
    Runtime String key comparison uses content without per-comparison allocation.

P4-INV-24
    Record lookup is O(log n) or better on fixed shapes.

P4-INV-25
    Record equality/hash ignore presentation order.

P4-INV-26
    Record equality/hash do not use reflection expansion.

P4-INV-27
    Record patterns resolve to canonical FieldKeys before lowering.

P4-INV-28
    The existing row solver remains the only row inference authority.

P4-INV-29
    Existing type relation/query budget and cancellation remain authoritative.

P4-INV-30
    Symbolic static key computation does not create a P4-only residual engine.

P4-INV-31
    Stable metadata never serializes VM or store-local key IDs.

P4-INV-32
    Legacy string-key metadata can only mean Symbol key when migrated.

P4-INV-33
    Non-representable heterogeneous metadata fails closed until a stable schema path exists.

P4-INV-34
    P3 ProductShape::Pack and Pack runtime behavior remain untouched by Record key migration.

P4-INV-35
    EmptyRecord remains the zero-field immediate Record.

P4-INV-36
    Product/Sum sealed capability derivation remains P5 scope.

P4-INV-37
    Final public RecordFields ontology retirement remains P9 scope.

P4-INV-38
    FieldKey canonical order is used consistently by rows, static Records,
    runtime logical shapes, metadata fingerprints, and P5 reflection inputs.

P4-INV-39
    A static/generic key collision discovered after reduction/specialization is a diagnostic.

P4-INV-40
    Invalid runtime Record key is rejected before its field value expression executes
    when source evaluation reaches the key guard first.
```

---

# 22. Source-of-truth map

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| concrete FieldKey identity | `phalcom-semantic` FieldKey scalar | row/static/runtime/metadata | VM-specific key enum with different semantics |
| FieldKey order | FieldKey canonical comparator | row sort/runtime sort/reflection/fingerprints | per-layer order tables |
| exact integer key | shared semantic exact-int scalar | static values/FieldKey/runtime projection | second BigInt normalization |
| row field set/presence | `RecordRowData` | relation/inference/pattern/reflection | reflection-only row map |
| anonymous tail key/value domain | `AnonymousRecordRemainder` | relation/lookup/metadata | P5-only tail model |
| captured row correlation | existing RecordRowTail parameter/projection | solver/relation | flattened union |
| row solving | `RecordRowSolver` | inference/generics | unordered reflection solver |
| fixed lookup semantics | P4 shared unordered lookup | checker/pattern/P5/P6 | bespoke reflection lookup |
| Static Record identity | `StaticValueStore` + FieldKey | evaluator/compiler | runtime Record |
| runtime Record logical shape | `RecordProductShape` projection of FieldKey | VM/access/equality | hash-map-only identity |
| ordered labels | existing Symbol authorities | Tuple/Pack/calls | FieldKey widening |
| stable key encoding | `FieldKeyRef` | metadata/fingerprint | strings for every key |
| final Product capability | P5 | generic constraints/reflection | P4 trait synthesis |
| checkpoint state | C4 checkpoint file | all plans | plan-local state authority |

---

# 23. Expected repository impact map

## 23.1 AST/parser

Expected:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/error.rs
phalcom-ast/tests/**
```

Reasons:

- Record key AST;
- heterogeneous Record type keys;
- typed key-domain tails;
- heterogeneous Record pattern keys;
- context separation from ordered labels.

## 23.2 Semantic field-key/rows

Expected:

```text
phalcom-semantic/src/field_key.rs                  # likely new
phalcom-semantic/src/types/row.rs
phalcom-semantic/src/types/row_solver.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/rigid.rs                # verify live use
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/row_inference.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/checker/call.rs               # tail/RecordFields audit
phalcom-semantic/src/checker/associated.rs         # tail audit
phalcom-semantic/src/checker/gadt_proof.rs         # row-field audit
phalcom-semantic/src/impls.rs                      # tail audit
```

## 23.3 Static values/evaluator

Expected:

```text
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/static_eval/**
phalcom-semantic/src/db/fingerprint.rs             # if static/row products include key content
```

P3 Pack branches in `static_value/store.rs` are preservation constraints.

## 23.4 Match semantics

Expected:

```text
phalcom-semantic/src/match_semantics.rs
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/checker/coverage/**
```

Only Record key coordinates; variant/data labels remain their existing Symbol/name identities.

## 23.5 Metadata

Expected staging surface:

```text
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/validate.rs
phalcom-type-meta/src/header.rs                    # only if a versioned staging change is adopted
phalcom-type-meta/tests/schema_compat.rs

phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/export.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
```

## 23.6 Core runtime/compiler

Expected:

```text
phalcom-core/src/product/shape.rs
phalcom-core/src/product/mod.rs
phalcom-core/src/product/view.rs
phalcom-core/src/heap/record.rs
phalcom-core/src/heap/record_literal_builder.rs
phalcom-core/src/heap/object.rs                    # likely no new arm, exhaustive audits
phalcom-core/src/heap/trace.rs                     # builder audit
phalcom-core/src/bytecode.rs                       # GuardFieldKey if added
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/patterns.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/primitive/record.rs
phalcom-core/src/typing/overlay.rs
phalcom-core/src/typing/environment.rs
phalcom-core/src/typing/inspect.rs
phalcom-core/core/universe/src/collections/record.ph
```

## 23.7 P3 shared files requiring extra care

P4 touches several P3-active files.

Before implementation, re-read current P3 state for:

```text
product/shape.rs
product/mod.rs
product/view.rs
compiler/lib/expr.rs
vm/dispatch.rs
static_value/store.rs
type_lambda.rs
type metadata
```

Mechanical merge drift is expected.

Architectural drift affecting Pack ownership/labels is a consultation trigger.

---

# 24. Explicit non-goals

P4 does **not** own:

- changing Tuple labels to FieldKey;
- changing Pack labels to FieldKey;
- changing call labels to FieldKey;
- changing selector labels to FieldKey;
- arbitrary Map key semantics;
- replacing Record with Map;
- making Record mutable;
- removing `RecordFields` internal kind;
- Product/OrderedProduct/UnorderedProduct trait derivation;
- Sum/Variant reflection;
- public `ProductShape` capability system;
- static `for` over unordered reflection;
- general static residualization;
- recursive static computation;
- final cross-module StaticValue serialization;
- final metadata schema checkpoint release;
- broad LSP/REPL ontology migration;
- final C4 performance certification;
- P3 Pack equality bug repair unless P4 causes a direct regression.

---

# 25. Architectural questions resolved by this analysis

## 25.1 Should runtime FieldKey contain ObjRef for String/LargeInt?

**No.**

Record shape identity must outlive/ignore individual heap objects and cannot depend on GC object identity.

Use owned content.

## 25.2 Should semantic FieldKey use StaticStringId/StaticSymbolId?

**No.**

Those IDs are StaticValueStore-lineage-local and cannot be canonical row identity.

## 25.3 Should `ProductLabel` globally become FieldKey?

**No.**

That would incorrectly widen Tuple/Pack/call labels.

## 25.4 Should P4 create a new unordered solver?

**No.**

Migrate the current row solver.

## 25.5 Should runtime Record use source order as canonical key order?

**No.**

Retain presentation order separately; logical order is FieldKey canonical order.

## 25.6 Should static Record use source order for equality/fingerprint?

**No.**

Canonical key order.

## 25.7 Can a runtime-computed key become a fixed Record type field?

**No.**

Only statically reduced keys become fixed fields. Runtime keys produce conservative open-tail type information.

## 25.8 Can heterogeneous Record keys become call labels?

**Only Symbol keys.**

No coercion.

## 25.9 Should P4 encode symbolic key residuals directly in RecordRowData?

**No.**

Use P2 checked static evaluation where reducible; preserve unresolved symbolic computation for the established static-definition/residual architecture, with general residualization owned by P7.

## 25.10 Does P4 need to wait for P3 T7/T8 closure?

**No.**

P4 is parallel after P2. Runtime shared-file conflicts require coordination, not semantic dependency.

---

# 26. Open choices to freeze in the later patch-grade plan

The following are implementation-level choices that should be frozen before coding their respective slice.

## O-P4-1 — string storage in `FieldKey`

Candidates:

```text
Box<str>
Arc<str>
```

Recommendation: `Arc<str>` if row/metadata/runtime projections clone keys frequently; otherwise `Box<str>` is simpler. Semantics identical.

## O-P4-2 — exact integer scalar module location

Recommendation: extract/re-export the existing normalized `StaticInt` into a pure semantic scalar module rather than duplicate it.

## O-P4-3 — anonymous tail default key domain

Recommendation:

```text
**: V -> FieldKey keys
...   -> FieldKey keys
```

with explicit `**[Symbol]: V` for Symbol-only tails.

## O-P4-4 — typed-tail lookup precision

Recommendation:

```text
Typed(K,V), key proven inside K -> Optional(V)
```

rather than Unknown.

## O-P4-5 — runtime Record shape lookup

Recommendation: canonical binary search first. Retain/add hash cache only if benchmarks justify it.

## O-P4-6 — runtime shape key storage

Recommendation:

```text
logical_keys + presentation_to_logical
```

rather than duplicated presentation/logical owned keys.

## O-P4-7 — metadata staging/version

Recommendation: define lossless `FieldKeyRef` during P4; do not claim final schema release. If existing bundle format cannot represent non-Symbol keys without a version bump, fail closed on export until the versioned path is adopted rather than stringify.

---

# 27. STOP / consultation conditions for implementation planning

A later patch-grade plan should require STOP AND CONSULT if live implementation proves any of these false:

1. `RecordRowData` cannot carry FieldKey without introducing a second row identity arena.
2. anonymous tail key domains require a fundamentally new kind rather than a Type constraint.
3. P2 static values cannot expose exact String/Symbol/Int/Bool content without store-local identity leakage.
4. runtime Record shape cannot own key content without violating GC or object-size constraints.
5. supporting heterogeneous Record construction would require generalizing Pack/call labels.
6. metadata cannot stage FieldKey without silently breaking supported schema compatibility.
7. optionality/width semantics appear inconsistent with the rules preserved here.
8. row solver changes require replacing rather than adapting the existing solver.
9. static key expressions require evaluator-side name/type resolution rather than consuming checked P2 products.
10. Record pattern resolution would require a second key/type resolver.
11. a proposed optimization changes presentation-order behavior.
12. equality/hash would need to treat Symbol and String spellings as equivalent.
13. a large-row discriminator reveals an unavoidable quadratic canonical algorithm in an architectural owner.
14. P3 has since changed shared product representation in a way that invalidates the Record-shape design.

---

# 28. Test architecture

## 28.1 AST/parser coverage

Required:

```text
record type:
  { name: String }
  { ["mime"]: String }
  { [42]: Handler }
  { [true]: Bool }
  mixed fixed keys
  optional heterogeneous keys

tails:
  { **[String]: Handler }
  { **[Int]: Value }
  { **[FieldKey]: Value }
  { **[String]: ... }
  { **: Value }
  { ... }
  { **R }

patterns:
  { name: x }
  { ["mime"]: x }
  { [42]: x }
  { [true]: x }
```

Negative:

- malformed bracket key;
- missing colon;
- invalid tail syntax;
- ordered Tuple label grammar remains unchanged.

## 28.2 FieldKey scalar tests

Matrix:

```text
cross-kind order:
  Symbol < String < Int < Bool

within:
  #a < #b
  "a" < "b"
  -2 < -1 < 0 < 1 < huge BigInt
  false < true

distinct:
  #x != "x"
  1 != true
  0 != false

hash/fingerprint:
  stable across construction order
  exact BigInt
  exact UTF-8
```

## 28.3 Row formation

- mixed-key canonical sorting;
- same row different source order → same RecordRowId;
- duplicate each kind;
- `#x` and `"x"` allowed together;
- large Int key;
- optional presence preserved;
- typed/unconstrained key-domain tails;
- invalid key-domain type rejected.

## 28.4 Row solver/lacks

- lacks Symbol;
- lacks String;
- lacks Int;
- lacks Bool;
- distinct same-text Symbol/String;
- extension propagation;
- two-row merge;
- captured tail;
- row-variable cycle/occurs behavior unchanged;
- budget/cancellation unchanged.

## 28.5 Subtyping matrix

At minimum:

```text
required -> required
optional -/-> required
required -> optional
optional -> optional

width:
  extra fixed field vs closed
  extra fixed field vs Typed compatible key/value
  extra fixed field vs Typed incompatible key
  extra fixed field vs Unconstrained compatible key
  extra fixed field vs incompatible key domain

tails:
  Closed -> Typed
  Typed -> Typed
  Typed -> Unconstrained
  Unconstrained -> Typed (reject)
  Unconstrained -> Unconstrained
  open -> Closed (reject)

optional absent:
  closed
  typed tail excludes key
  typed tail includes key with compatible value
  typed tail includes key with incompatible value
  unconstrained compatible key
  captured unresolved
```

Repeat key cases across all four key kinds.

## 28.6 Static Record

- intern all key kinds;
- source order irrelevant;
- duplicate rejection;
- semantic equality;
- semantic hash;
- content fingerprint;
- static value → FieldKey conversion;
- invalid static key value;
- static value type projection.

## 28.7 Runtime Record

Construction:

- fixed Symbol/String/Int/Bool;
- mixed keys;
- huge Int;
- empty Record unchanged;
- dynamic computed key;
- invalid key rejects before field-value side effect;
- duplicate computed key fails;
- Record spread preserves heterogeneous keys;
- Tuple/Pack Symbol spread into Record.

Access:

- get each key kind;
- missing key;
- Symbol/String same text distinct;
- Int/Bool distinct;
- presentation iteration;
- canonical logical internal order.

Equality/hash:

- same mapping different source order equal;
- same hash;
- different key kind unequal;
- different value unequal.

## 28.8 Call/Pack boundary regression

- `**record` into call works for all-Symbol Record;
- known String-key Record call spread rejected;
- dynamic Record with non-Symbol key fails at runtime;
- no String→Symbol coercion;
- Pack labeled spread behavior unchanged;
- Tuple labeled behavior unchanged;
- fixed-call no-Pack invariant from P3 unchanged.

## 28.9 Record pattern coverage

- match each key kind;
- missing fixed key;
- optional/open row typing;
- rest excludes exact heterogeneous key;
- Symbol/String distinction;
- runtime destructuring;
- match lowering consumes resolved key, not source spelling.

## 28.10 Metadata staging

- FieldKeyRef encode/decode for all kinds;
- exact BigInt round-trip;
- Symbol/String distinction;
- deterministic fingerprint;
- legacy Symbol name → Symbol FieldKey migration test if decoder path is implemented;
- non-Symbol export fails closed if active schema cannot represent it;
- no runtime IDs in metadata.

## 28.11 Performance discriminators

- form 1k mixed keys;
- form 10k mixed keys;
- row lookup repeated;
- runtime Record lookup repeated;
- equality of large reordered Records;
- row solver merge of large disjoint/overlapping fixed prefixes.

The goal is to expose accidental O(n²), not to establish release-level benchmark thresholds.

---

# 29. Suggested coherent implementation decomposition for the later Luna plan

This is not yet the executable plan, but the requirements naturally decompose into these coherent slices.

## T0 — Takeover, baseline, and P3 coexistence audit

Freeze live revision; re-read P3 current shared product files; run only targeted existing row/Record/P3 discriminators.

## T1 — Canonical FieldKey scalar and source syntax

Deliver:

- pure semantic FieldKey;
- order/hash/display;
- Record key AST;
- type/pattern/tail syntax;
- no row migration yet beyond compile scaffolding.

## T2 — Canonical row/store/tail migration

Deliver:

- RecordRowField.key;
- anonymous tail key domain;
- TypeStore row formation;
- scoped rows;
- substitution/instantiation.

## T3 — Row solver/inference/relation convergence

Deliver:

- FieldKey lacks;
- merge unification;
- lookup calculus;
- subtyping/tail rules;
- expected field typing.

## T4 — Static Record and static-key bridge

Deliver:

- StaticValue Record FieldKey;
- key conversion;
- evaluator support for static Record keys;
- collision/fingerprint behavior.

## T5 — Runtime Record shape and construction

Deliver:

- FieldKey RecordProductShape;
- builder/VM conversion;
- BuildRecord/static lowering;
- O(log n) lookup;
- key access.

## T6 — Record equality/hash/spread + P3 boundary

Deliver:

- canonical logical equality/hash;
- heterogeneous Record spread;
- Symbol-only call/Pack lane enforcement;
- presentation preservation.

## T7 — Record patterns and structural consumers

Deliver:

- FieldKey pattern AST/resolution/lowering;
- rest exclusion;
- shared lookup use.

## T8 — Metadata staging, incrementality, performance, compatibility

Deliver:

- FieldKeyRef staging;
- fingerprints;
- runtime overlay typing;
- large-shape gates;
- C1–C3/P2/P3 compatibility checks.

## T9 — Lifecycle closure

Only after actual verification:

- P4 walkthrough;
- P4 handoff;
- checkpoint ledger update;
- unblock P5 when both P3 and P4 are semantically stable.

---

# 30. Acceptance gates recommended for the later plan

## G1 — FieldKey + row semantic gate

Must prove:

```text
syntax
canonical FieldKey
row formation
tails
solver/lacks
relation
lookup
```

No runtime dependency.

## G2 — Static/runtime Record vertical gate

Must prove:

```text
StaticValue Record
runtime mixed-key Record
construction
lookup
equality/hash
presentation/logical separation
```

## G3 — Pattern/call interoperability gate

Must prove:

```text
heterogeneous Record patterns
Record rest
Record spread
Symbol-only ordered/call labels
no P3 Pack regression
```

## G4 — Metadata/incremental/resource gate

Must prove:

```text
stable key fingerprint
metadata staging
targeted invalidation
large shape asymptotics
affected crate checks
```

Broad language corpus should not be mandatory during BUILD. The checkpoint already records that it is extremely slow and is not a P3/P4-focused gate unless separately requested.

---

# 31. Key architectural findings from the live audit

## F1 — P4 is wider than a `Box<str>` rename

The key string is embedded in:

- AST type fields;
- row IDs;
- row solver;
- stable lacks;
- inference;
- type lambdas;
- static records;
- relation;
- patterns;
- product lowering;
- runtime shapes;
- runtime type overlays;
- metadata fingerprints.

A partial migration would create contradictory shape identities.

## F2 — value syntax is ahead of type semantics

Computed Record labels already parse.

The compiler currently guards them as Symbol.

P4 can leverage the syntax shape but must replace the semantic/runtime key category only for Record.

## F3 — typed tails require a new dimension

Current anonymous row tail knows only value type.

`**[String]: Handler` requires key type + value type.

This is not representable by merely changing fixed field keys.

## F4 — the current row solver has an avoidable large-shape quadratic path

Canonical sorted rows are already available, but unification/subtyping frequently scans them repeatedly.

P4 should exploit canonical order rather than carrying O(n²) string-era algorithms into 10k-field heterogeneous shapes.

## F5 — StaticValueStore IDs cannot be row key identity

P2 String/Symbol IDs are store-local.

FieldKey must own content.

## F6 — runtime heap IDs cannot be shape identity

String/LargeInt are heap objects.

RecordProductShape must own stable content, not ObjRef.

## F7 — P3 introduced a stronger ordered/unordered separation

`ProductShape::Pack` now exists independently.

P4 should strengthen, not blur:

```text
Pack labels  = Symbol
Record keys  = FieldKey
```

## F8 — heterogeneous Record-to-call spread is a newly exposed semantic boundary

Previously every Record key was a Symbol, so `**record` naturally formed call labels.

P4 must add a Symbol-only filter/rejection boundary.

## F9 — current Record equality is unnecessarily O(n²)

Canonical logical FieldKey order makes O(n) equality straightforward while preserving presentation order separately.

## F10 — metadata staging cannot be postponed conceptually

Even if P8 owns the final schema release, P4 must define the lossless FieldKey metadata shape now. Otherwise P5/P8 would have to reverse-engineer P4 row identity later.

---

# 32. Risks

| Risk | Severity | Requirement |
|---|---:|---|
| Global ProductLabel widening breaks calls/Pack | Critical | Record-specific key path |
| Store/VM IDs enter FieldKey | Critical | content-owned semantic key |
| tail lacks key domain | Critical | two-coordinate anonymous tail |
| Symbol/String collapse | Critical | tagged identity + tests |
| Int/Bool collapse | Critical | tagged identity + tests |
| runtime String key uses ObjRef | Critical | copy/own content in shape |
| non-Symbol `**record` call forwarding | Critical | Symbol-only gate |
| metadata stringification | Critical | FieldKeyRef or fail closed |
| optionality regression | High | preserve existing matrix |
| row solver quadratic behavior | High | merge canonical prefixes |
| Record equality presentation-sensitive | High | logical canonical walk |
| static key ad hoc evaluator | High | P2 checked static authority |
| symbolic key invents residual engine | High | defer to P7 residual authority |
| P3 Pack branches lost in exhaustive edits | High | explicit P3 regression gate |
| runtime key materialization allocates on keyAt | Medium | acceptable floor; benchmark before cache |
| broad schema bump collides P8 | Medium | staging/fail-closed boundary |
| tooling assumes field.name | Medium | compile audit now, polish P9 |

---

# 33. Final planning judgment

P4 is implementable on the current architecture without replacing the row system or runtime product substrate.

The repository is actually well-positioned:

```text
already present:
    canonical RecordRowId / row interning
    optionality/presence
    captured/open row tails
    row solver
    subtype relation
    P2 StaticValueStore
    shared ProductStorage
    Record presentation/logical permutations
    exact BigInt runtime/static representations
    semantic metadata graph
    P3 expanded product-shape infrastructure
```

The missing abstraction is the **stable heterogeneous field-key scalar and key-domain-aware unordered tail**.

The strongest implementation architecture is:

```text
FieldKey
  = one semantic content value

RecordRowData
  = canonical sorted fixed FieldKeys + existing presence + key-aware tail

RecordRowSolver / relation
  = existing engines migrated to FieldKey and tail key domains

StaticValue::Record
  = same FieldKey canonical set

RecordProductShape
  = same semantic key content projected into canonical logical runtime shape
    + presentation permutation

metadata
  = explicit FieldKeyRef

P5 reflection
  = consumes these canonical P4 facts later
```

This architecture solves the generalized Record problem while preserving the core C4 principle:

> **one structural/type authority, multiple projections — not one implementation per consumer.**

P4 should be considered complete only when Symbol/String/Int/Bool keys work coherently through source syntax, row formation, inference/lacks, subtyping/lookup, static Record values, runtime construction/access/equality/hash, record patterns, P3 call-boundary interoperability, deterministic key order, metadata staging, and large-shape performance evidence.

It should **not** be considered complete merely because `RecordRowField.name` has been renamed to `key`.

---

# 34. Preparation checklist for a Luna patch-grade plan

Before authoring the executable P4 plan, re-read the live versions of:

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-GUIDANCE.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-requirements-analysis.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-structural-static-upgrade-handoff.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4.P3/LANG007.C4.P3-interim-handoff.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P3/LANG007.C4.P3-luna-patch-grade-implementation-plan.md

docs/specs/structural-products.md
docs/specs/type-level-domains.md
docs/specs/vm/products.md
docs/specs/vm/values-and-objects.md
```

Then re-audit:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs

phalcom-semantic/src/field_key.rs                 # if created
phalcom-semantic/src/types/row.rs
phalcom-semantic/src/types/row_solver.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/checker/row_inference.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/match_semantics.rs

phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/validate.rs
phalcom-type-meta/src/header.rs

phalcom-core/src/product/shape.rs
phalcom-core/src/product/mod.rs
phalcom-core/src/product/view.rs
phalcom-core/src/heap/record_literal_builder.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/patterns.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/primitive/record.rs
phalcom-core/src/typing/overlay.rs
phalcom-core/src/typing/environment.rs
phalcom-core/core/universe/src/collections/record.ph
```

The executable plan should freeze the chosen representation only after confirming no post-`b9f72cfe` P3 continuation has materially changed these shared seams.

---

# 35. Requirements-analysis completion state

This document establishes:

```text
requirements analysis: COMPLETE
implementation plan:   NOT YET AUTHORED
P4 implementation:     NOT_STARTED
P4 verification:       UNVERIFIED
checkpoint:            remains IN_PROGRESS / PARTIAL
```

No implementation or lifecycle promotion is implied by this analysis.
