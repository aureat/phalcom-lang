---
id: LANG007.C4.P4
category: LANG
program: LANG007
checkpoint: LANG007.C4
plan: LANG007.C4.P4
kind: implementation-plan
title: "Generalized FieldKey and Unordered-Product Calculus"
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
started_against_revision: 03967716867684a513cffd6a044af36366859b18
worktree_branch: codex/lang007-c4-p4
worktree_path: /Users/altunhasanli/.codex/worktrees/lang007-c4-p4/phalcom
integrated_branch: main
integrated_revision: 80e4e854841e71b8292df27a8932f109f6ae2ca5
started: 2026-10-05
continued_against_revision: bf17519547d621ff50b399bfb3697c8c8821001c
continued_branch: main
continued_worktree_path: /Users/altunhasanli/dev/phalcom/phalcom
continued: 2026-10-05
depends_on:
  - LANG007.C4.P2
parallel_with:
  - LANG007.C4.P3
blocks:
  - LANG007.C4.P5
follows: LANG007.C4.P2
supersedes: null
repository: aureat/phalcom-lang
branch: main
requirements_analysis: LANG007.C4.P4-requirements-analysis
requirements_repository_revision_audited: b9f72cfe8ee5338485b716702cd9560776fce3f7
prepared: 2026-10-05
---

# LANG007.C4.P4 — Generalized `FieldKey` and Unordered-Product Calculus

## Luna patch-grade implementation plan

# 0. Executor contract

This plan is written for a capable implementation model operating under bounded architectural authority. It converts the accepted `LANG007.C4.P4` requirements analysis into an executable patch sequence. It defines intended work; actual progress and verification are recorded in the C4 checkpoint and the plan's completion records.

The implementer:

- may adapt mechanical details to the live repository after re-reading the named paths;
- must preserve the semantic ownership, identity, ordering, evaluation-order, and P3/P4 boundaries defined below;
- must not invent a second row solver, second FieldKey encoding, second static evaluator, second pattern-key resolver, or runtime-only structural identity;
- must keep Record keys and ordered Tuple/Pack/call labels deliberately distinct;
- must use the existing type relation, row, static-value, query-budget, cancellation, incremental, product-storage, and pattern authorities rather than reproducing them;
- must use sparse discriminating tests during BUILD, coherent gates during STABILIZE, and only the named broader checks during CERTIFY;
- must classify unrelated failures rather than repairing them opportunistically;
- must preserve dirty P3 work and never overwrite, stash, discard, or rebase it as a side effect of P4;
- must STOP AND CONSULT when any global or task-local escalation trigger fires;
- must keep the C4 checkpoint record truthful throughout execution;
- must produce the P4 walkthrough and P4 handoff before marking the plan complete.

Testing is evidence gathering, not a ritual. A source edit does not itself justify a broad test run.

---

# 1. Goal

Implement one canonical heterogeneous Record field-key model—`Symbol | String | Int | Bool`—and migrate Telos's existing unordered-product stack to it end-to-end: source syntax, canonical rows, row solving/inference/subtyping/lookup, static Records, runtime Record shapes and construction, equality/hash/access/spread, Record patterns, stable metadata staging, incrementality, and large-shape performance, while preserving Symbol-only Tuple/Pack/call labels and all established optional-row semantics.

---

# 2. Checkpoint acceptance objective

`LANG007.C4` requires generalized heterogeneous Record field keys and a reusable unordered-product calculus before P5 can build compiler-derived `UnorderedProduct` structure and reflection.

P4 is therefore a **migration plus soundness/performance hardening plan**, not checkpoint closure. It must establish the canonical low-level structural facts that P5 will consume later:

```text
FieldKey scalar
    ↓
canonical RecordRowData + key-aware tail
    ↓
existing row solver / relation / lookup
    ↓
StaticValue::Record
    ↓
semantic lowering projection
    ↓
RecordProductShape + runtime Record behavior
    ↓
resolved Record patterns
    ↓
lossless metadata staging / fingerprints
```

P4 is complete only when the four key kinds behave coherently across this vertical and the P3 ordered-label boundary is demonstrably intact.

P4 does **not** close C4. P5 remains blocked until both P3 and P4 are semantically stable.

---

# 3. Repository grounding

Prepared from the P4 requirements analysis audited against:

```text
repository: aureat/phalcom-lang
branch:     main
audited:    b9f72cfe8ee5338485b716702cd9560776fce3f7
date:       2026-10-05
```

The attached P3 interim records report the same pushed `b9f72cfe` head, while also recording uncommitted P3 lifecycle/test changes. P3 is still `IN_PROGRESS / PARTIAL / UNVERIFIED`; its root-versus-slice source equality failure is outside P4 unless P4 directly regresses that path.

Repository facts inherited from the requirements audit:

1. `RecordRowData`, row interning, row solver, lacks constraints, optionality, captured tails, substitution, instantiation, and record subtype relation already exist and are the semantic authorities to migrate.
2. Canonical row field identity is currently `Box<str>` and the solver/relation stack uses string-keyed searches.
3. Record value syntax already has computed labels, but type syntax and patterns are still effectively Symbol/name-oriented.
4. Anonymous row tails currently constrain only value type; P4 must add key-domain information.
5. P2 `StaticValueStore` has structural Record support but Record keys remain string-based.
6. P3 has already introduced first-class Pack type/static/runtime/product branches. These shared-file edits are preservation constraints.
7. Runtime Record shapes already separate presentation order from logical canonical order. P4 must retain that distinction.
8. Runtime Record shape/build/view machinery is Symbol-only today.
9. Current source Record equality performs repeated lookup and is quadratic.
10. Stable type metadata and runtime type overlays currently serialize/store Record field names as strings.
11. Record patterns are string/Symbol-name based.
12. `phalcom-core` already consumes semantic products, so the canonical pure semantic `FieldKey` can be reused or losslessly projected into runtime Record shapes without inventing a second semantic key definition.

At T0, re-read the named live paths and compare the actual starting revision with `b9f72cfe`. Mechanical drift is expected. Any architectural drift affecting Pack labels, row identity, static-value identity, product shape ownership, or metadata ownership is an escalation condition.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as a STOP / CONSULT condition.

---

# 4. Required reads before implementation

Read these before editing P4 source:

```text
AGENTS.md

docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-GUIDANCE.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-requirements-analysis.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-structural-static-upgrade-handoff.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4-requirements-analysis.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4.P3/LANG007.C4.P3-interim-handoff.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P3/LANG007.C4.P3-interim-walkthrough.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P3/LANG007.C4.P3-luna-patch-grade-implementation-plan.md

docs/specs/structural-products.md
docs/specs/type-level-domains.md
docs/specs/vm/products.md
docs/specs/vm/values-and-objects.md
```

Re-audit, at minimum:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/error.rs

phalcom-semantic/src/types/row.rs
phalcom-semantic/src/types/row_solver.rs
phalcom-semantic/src/types/store.rs
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
phalcom-semantic/src/static_eval/**
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/match_semantics.rs

phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/validate.rs
phalcom-type-meta/src/header.rs

phalcom-core/src/product/shape.rs
phalcom-core/src/product/mod.rs
phalcom-core/src/product/view.rs
phalcom-core/src/heap/record.rs
phalcom-core/src/heap/record_literal_builder.rs
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

Also inspect the exact existing Record/row/parser/core test conventions before creating new files. Prefer extending established focused modules where doing so keeps selectors narrow; otherwise create P4-specific modules named consistently with the repository's current test harness.

Avoid broad exploration unless evidence makes it necessary.

---

# 5. Normative authority

## 5.1 Accepted language semantics

The following are fixed P4 semantics:

```telos
type FieldKey = Symbol | String | Int | Bool
```

Canonical cross-kind order is:

```text
Symbol < String < Int < Bool
```

Within a kind:

```text
Symbol  exact spelling/content, deterministic lexical order
String  exact content, deterministic lexical order
Int     mathematical numeric order, arbitrary precision
Bool    false < true
```

Additional fixed behavior:

- `#x` and `"x"` are distinct keys.
- `1` and `true` are distinct keys; `0` and `false` are distinct keys.
- no Unicode normalization or locale-sensitive ordering is introduced.
- Record structural identity/equality/hash ignore presentation/source order.
- duplicate Record keys are invalid on every construction route; no last-write-wins semantics.
- Tuple labels, Pack labels, call labels, selector slots, rest labeled lanes, and ordered-product reflection labels remain `Symbol`.
- Record-to-call / Record-to-ordered-labeled-lane expansion admits only `FieldKey::Symbol`; no String/Int/Bool coercion.
- existing required/optional Record field rules remain authoritative.
- optional Tuple entries are not reintroduced.
- existing `RecordRowId` / `RecordRowSolver` / relation engine remain the only row authorities.
- query budget/cancellation remain the resource authority.
- P2 static evaluation remains the static key-expression authority; P4 does not add an evaluator.
- unresolved symbolic key computation is not embedded as a P4-specific residual graph in `RecordRowData`.
- P5 owns public `Product` / `UnorderedProduct` capabilities and reflection.
- P8 owns final durable metadata/schema publication.
- P9 owns final public ontology retirement/tooling closure.

## 5.2 Patch-grade design choices frozen by this plan

The requirements analysis intentionally left implementation-level choices. This plan freezes them as follows.

### D-P4-01 — `FieldKey` text storage

Use `Arc<str>` for `FieldKey::Symbol` and `FieldKey::String` unless the live repository proves a concrete ownership constraint that makes `Box<str>` materially safer. `Arc<str>` is chosen because the same immutable key content is expected to cross row, solver, static-value, lowering, runtime-shape, diagnostic, and metadata projections, and cheap clones reduce repeated text copies.

`Arc` identity is never semantic. Equality/order/hash/fingerprint use content only.

### D-P4-02 — exact integer scalar ownership

Move or re-export the existing normalized `StaticInt` implementation into a lower pure semantic scalar module so both `StaticValueData::Int` and `FieldKey::Int` use one exact arbitrary-precision representation. Preserve compatibility re-exports if needed to keep the P2/P3 patch surface bounded. Do not create a second BigInt normalization path.

### D-P4-03 — anonymous tail default key domain

Freeze:

```text
**: V  == unknown extra FieldKey keys, values constrained by V
...    == unknown extra FieldKey keys, unconstrained values
```

Symbol-only anonymous tails are explicit:

```telos
**[Symbol]: V
```

### D-P4-04 — typed-tail lookup precision

For an absent fixed key queried against `Typed(K, V)`:

```text
key proven outside K -> Absent
key proven inside K  -> Optional(V)
membership uncertain -> Unknown
```

`Optional(V)` means “may be absent; if present, V-constrained,” not “syntactically declared optional only.”

### D-P4-05 — runtime Record lookup

Use binary search over canonical `logical_keys` as the baseline. Do not retain/add a heterogeneous hash cache merely by analogy with Symbol-only shapes. A cache may be introduced only if T8 performance evidence demonstrates a need and it preserves borrowed, allocation-free key comparison.

### D-P4-06 — runtime shape key storage

Use one owned canonical key array plus a compact presentation permutation:

```rust
RecordProductShape {
    logical_keys: Box<[FieldKey]>,
    presentation_to_logical: Box<[u32]>,
}
```

Mechanically equivalent integer index widths are allowed if validated against repository limits. Do not own duplicate presentation and logical copies of String/BigInt keys.

### D-P4-07 — metadata staging

P4 defines a lossless `FieldKeyRef` representation and deterministic stable encoding/fingerprint semantics. P4 does **not** silently publish an incompatible final wire/schema version.

If the active metadata schema cannot encode non-Symbol keys compatibly:

```text
active legacy export of Symbol key       -> permitted via exact Symbol projection
active export of String/Int/Bool key     -> fail closed with explicit diagnostic/error
P4 staging representation                -> retains full lossless FieldKeyRef
final durable schema publication         -> P8
```

No stringification, Symbol coercion, BigInt truncation, field omission, or process-local IDs are permitted.

## 5.3 Normative versus implementation versus future scope

| Layer | P4 normative | Current implementation from audit | Future/non-P4 |
|---|---|---|---|
| key identity | `FieldKey` tagged content value | string/Symbol-oriented | P5 reflection consumes it |
| row identity | canonical FieldKey-sorted row | `Box<str>` sorted row | no replacement solver |
| tails | key domain + optional value domain | value domain only | P7 residual computation |
| static Record | FieldKey keys | string keys | P8 durable StaticValue serialization |
| runtime Record | content-owned keys | Symbol-only | P9 final API naming/tooling |
| ordered labels | Symbol only | Symbol | unchanged by P4 |
| metadata | lossless FieldKeyRef staging | string-name nodes | P8 final schema release |
| capabilities | none added | n/a | P5 `UnorderedProduct` |

---

# 6. Takeover state

P4 inherits the following stable authorities.

| Concept | Current symbol/path | Owner | P4 invariant |
|---|---|---|---|
| canonical row | `RecordRowId`, `RecordRowData` in `types/row.rs` | semantic type system | migrate key coordinate; do not replace |
| row solving | `RecordRowSolver` in `types/row_solver.rs` | semantic type system | preserve payload inference/cycles/budgets |
| optionality | `StructuralMemberPresence` and relation logic | semantic relation | semantics unchanged |
| captured tails | `RecordRowTail::{Parameter,Projection}` | semantic type system | preserve correlation |
| static values | `StaticValueStore`, P2/P3 structural values | semantic static layer | add/migrate Record keys only; preserve Pack |
| exact int | current `StaticInt` | semantic static scalar | one shared exact-int authority |
| pattern resolution | existing Record/match checker products | semantic checker | migrate resolved key coordinate |
| product runtime | `ProductStorage`, `ProductShape` | core runtime | Record-specific generalized keys; Pack preserved |
| Record shape | `RecordProductShape` | core runtime | logical/presentation separation retained |
| Pack shape | `PackProductShape` | P3 | Symbol labels remain untouched |
| call labels | existing Symbol argument-pack machinery | P3/runtime | only Symbol Record keys may cross boundary |
| query resources | existing `QueryBudget` / cancellation | semantic query infrastructure | reused for new relation work |
| stable metadata | schema-v10-era type metadata | type-meta/P8 later | stage lossless key representation; no lossy wire change |
| checkpoint truth | `LANG007.C4-CHECKPOINT.md` | lifecycle | P4 state must remain truthful alongside P3 |

P3 work already landed on shared files including `product/shape.rs`, `product/mod.rs`, `product/view.rs`, `compiler/lib/expr.rs`, `vm/dispatch.rs`, `static_value/store.rs`, `types/store.rs`, `type_lambda.rs`, and type metadata. P4 must preserve all Pack arms and P3 fixed-call/no-surface-Pack invariants.

---

# 7. Architecture

## 7.1 Authority flow

The canonical direction is:

```text
source Record key syntax
    ↓ ordinary parsing / checked static-key resolution
FieldKey
    ↓
RecordRowData + RecordRowTail(key domain, value domain)
    ↓
RecordRowSolver / relation / unordered lookup
    ├───────────────┐
    ↓               ↓
StaticValue Record  checked lowering facts
    ↓               ↓
static evaluator    compiler
                    ↓
             runtime Record shape
                    ↓
            VM / source Record API
```

Metadata is a projection of the same semantic key content:

```text
FieldKey -> FieldKeyRef -> stable fingerprint / staged metadata
```

No downstream subsystem may reconstruct FieldKey meaning from source spelling or runtime object identity.

## 7.2 Canonical `FieldKey`

Target conceptual form:

```rust
pub enum FieldKey {
    Symbol(Arc<str>),
    String(Arc<str>),
    Int(StaticInt),
    Bool(bool),
}
```

Required pure operations:

```rust
impl FieldKey {
    pub fn canonical_cmp(&self, other: &Self) -> Ordering;
    pub fn stable_fingerprint_into(&self, sink: &mut ...);
    pub fn display(&self) -> ...;
    pub fn kind(&self) -> FieldKeyKind;
}
```

Process-local `Hash` may exist for hash maps, but durable fingerprints must use an explicit tagged encoding.

Stable conceptual encoding:

```text
Symbol: tag 0 + byte length + exact UTF-8 bytes
String: tag 1 + byte length + exact UTF-8 bytes
Int:    tag 2 + sign + canonical magnitude bytes
Bool:   tag 3 + 0/1
```

## 7.3 Record source key model

Record keys become context-specific. Do not globally widen ordered-product labels.

Preferred AST boundary:

```rust
pub enum RecordKeySyntax {
    Symbol { ... },
    Computed { expr: Box<Expr>, ... },
}
```

Record type fields and Record pattern entries use Record-key syntax. Tuple/Pack/call syntax keeps the existing ordered `ProductLabel`/Symbol path.

Record type key expressions are static. A fixed row field exists only after the key expression reduces to one concrete `FieldKey` through P2's checked static authority.

For Record value literals:

```text
computed key reduces statically -> fixed FieldKey when semantic analysis can prove it
runtime key expression          -> never fabricated as a fixed row field
```

A runtime key contributes only conservative open-tail typing using existing type inference/join policy. If the live checker lacks a sound way to represent multiple dynamic key/value contributions without inventing a new row system, consult rather than fabricating fixed keys.

## 7.4 Canonical rows and tails

Target:

```rust
pub struct RecordRowField<T = TypeId> {
    pub key: FieldKey,
    pub ty: T,
    pub presence: StructuralMemberPresence,
}

pub struct RecordRowData {
    pub fields: Box<[RecordRowField]>, // canonical FieldKey order, unique
    pub tail: RecordRowTail,
}
```

Anonymous tails gain a key type:

```rust
pub enum AnonymousRecordRemainder {
    Unconstrained { key_type: TypeId },
    Typed { key_type: TypeId, value_type: TypeId },
}
```

Mechanical names may differ; both coordinates are mandatory.

Canonical formation is centralized:

```text
input fields
  -> sort once by FieldKey::canonical_cmp
  -> detect adjacent duplicates
  -> validate tail key domain against FieldKey
  -> intern canonical row
```

No shadow `name` field remains in canonical row identity.

## 7.5 Key-domain membership

Concrete key membership in an anonymous tail is decided by ordinary type relations:

```text
FieldKey::Symbol(...) -> Symbol type
FieldKey::String(...) -> String type
FieldKey::Int(...)    -> Int type
FieldKey::Bool(...)   -> Bool type

relation(type_of_key, tail.key_type)
```

Conceptual result:

```rust
enum KeyDomainRelation {
    ProvenInside,
    ProvenOutside,
    UnknownOrBlocked,
}
```

This is a thin projection over the existing relation engine, not a second subtype system.

## 7.6 Unordered lookup

P4 establishes one internal lookup semantic, conceptually:

```rust
enum UnorderedFieldLookup {
    Required(TypeId),
    Optional(TypeId),
    Absent,
    Unknown,
}
```

It is consumed by expected-field checking, pattern checking, relation helpers where appropriate, and later P5/P6 reflection/matching. P5 must not reimplement lookup.

## 7.7 Row solver/relation algorithm

Because rows are canonical sorted sequences, correspondence must use merge-style traversal where practical:

```text
left fixed prefix  ─┐
                    ├─ O(n + m) correspondence
right fixed prefix ─┘
```

Payload equations, captured-tail identity, occurs/cycle handling, query budgets, and cancellation remain existing authorities.

Subtyping keeps current optionality/width rules but extends tail reasoning with key domains.

## 7.8 Static Record

`StaticValueData::Record` moves from string keys to canonical `FieldKey` keys.

`StaticValueStore::field_key_of(value)` must be a pure store-to-content conversion:

```text
Static Symbol -> spelling -> FieldKey::Symbol
Static String -> content  -> FieldKey::String
Static Int    -> exact int -> FieldKey::Int
Static Bool   -> bool      -> FieldKey::Bool
other         -> NotAFieldKey
```

Returned keys contain no StaticValueStore IDs.

Static Record interning sorts canonically and fingerprints FieldKey stable bytes + child fingerprints.

## 7.9 Runtime Record key conversion

Runtime `Value -> FieldKey` conversion owns the heap boundary:

```text
Symbol Value    -> resolve spelling -> Symbol(content)
String object   -> copy/share immutable content -> String(content)
Small/Large Int -> exact semantic int -> Int(...)
Bool            -> Bool
other           -> record.invalid_field_key
```

No `ObjRef` or VM Symbol ID survives in structural key identity.

Reverse observation (`FieldKey -> Value`) may allocate String/LargeInt values for `keyAt`/diagnostic observation; this is an accepted correctness floor. Do not add a permanently rooted materialization cache without evidence.

## 7.10 Runtime Record shape

Preferred shape:

```rust
pub struct RecordProductShape {
    pub logical_keys: Box<[FieldKey]>,
    pub presentation_to_logical: Box<[u32]>,
}
```

`logical_keys` is canonical FieldKey order. `presentation_to_logical` preserves source/runtime encounter order.

Baseline fixed-shape lookup is `O(log n)` binary search in `logical_keys` with borrowed, allocation-free comparison.

## 7.11 Evaluation order

For computed Record fields:

```text
evaluate key
validate/convert to FieldKey
only then evaluate field value
```

A Record-specific guard/conversion path may be added. Tuple/Pack keeps `GuardSymbol` semantics.

Duplicate detection must preserve existing source evaluation laws: once a duplicate is known at the point the language specifies rejection, no later field value whose evaluation should be skipped may execute.

## 7.12 Record spread and P3 boundary

```text
Record -> Record
    preserve every FieldKey kind

Tuple/Pack labeled -> Record
    Symbol label -> FieldKey::Symbol

Record -> call / Pack labeled lane / Tuple labeled lane
    Symbol key -> forward
    String/Int/Bool -> reject
```

Known non-Symbol shapes should be diagnosed semantically when possible; dynamic shapes are runtime-guarded.

## 7.13 Record equality/hash

Equality/hash walk canonical logical order, never presentation order.

Target complexity:

```text
Record equality: O(n)
Record hash:     O(n)
```

Equal maps with different presentation order compare equal and hash equally.

## 7.14 Record patterns

Pattern keys statically reduce to canonical FieldKey before lowering. Resolved pattern products carry FieldKey, not source text. Rest exclusion compares exact FieldKey identity. Compiler lowering consumes resolved semantic keys and never re-parses/re-resolves source spelling.

## 7.15 Metadata staging

P4 introduces a lossless stable key representation, conceptually:

```rust
pub enum FieldKeyRef {
    Symbol(Box<str>),
    String(Box<str>),
    Int(CanonicalIntRef),
    Bool(bool),
}
```

Its fingerprint encoding is explicit and deterministic. Legacy metadata `name = "x"` can only map to `FieldKeyRef::Symbol("x")`.

The active wire schema must fail closed for non-Symbol keys if it cannot encode them compatibly. P4 must not smuggle heterogeneous keys through strings.

---

# 8. Ownership boundaries

## 8.1 Owns

P4 owns:

- pure semantic `FieldKey` identity, order, fingerprint, and display;
- the shared exact-int scalar placement needed by FieldKey;
- Record-specific key syntax for values/types/patterns;
- typed anonymous Record key-domain tails;
- canonical row migration from string name to FieldKey;
- row solver/lacks/inference/substitution/instantiation migration;
- key-domain-aware record subtyping and unordered lookup;
- StaticValue Record FieldKey migration;
- static-value-to-FieldKey conversion;
- runtime Record FieldKey conversion and shape/storage lookup;
- Record construction/access/equality/hash/spread behavior;
- Symbol-only call/Pack/Tuple labeled-boundary enforcement;
- Record pattern FieldKey resolution/lowering;
- FieldKey metadata staging and stable fingerprints;
- targeted incrementality and large-shape evidence.

## 8.2 Does not own

P4 does not own:

- Tuple, Pack, call, selector, or constructor-label generalization to FieldKey;
- P3 Pack equality/root-slice repair unless P4 causes a direct regression;
- replacing Record with Map or adopting arbitrary Map key semantics;
- mutable Record semantics;
- a new row/unordered solver;
- Product/OrderedProduct/UnorderedProduct/Sum capability derivation;
- public P5 shape reflection;
- static structural loops;
- general static residualization or recursive static computation;
- final cross-module StaticValue persistence;
- final metadata schema release;
- removal of internal legacy `RecordFields` kind;
- broad LSP/REPL/public-ontology polish;
- final C4 performance certification.

## 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| concrete key identity | semantic `FieldKey` | rows/static/runtime/metadata | VM/store-local key enum with different semantics |
| key ordering | `FieldKey::canonical_cmp` | row/runtime/metadata/P5 | per-layer rank/order logic |
| exact integer key | shared exact-int semantic scalar | StaticValue/FieldKey/runtime conversion | second BigInt normalization |
| fixed row fields/presence | `RecordRowData` | relation/inference/pattern/P5 | reflection-only map |
| anonymous tail domain | `AnonymousRecordRemainder` | relation/lookup/metadata | P5-only tail model |
| captured row correlation | existing `RecordRowTail` | solver/relation | flattened key/value unions |
| row solving | `RecordRowSolver` | inference/generics | unordered-specific solver |
| key membership | ordinary type relation via thin helper | relation/lookup | type-name checks |
| fixed lookup | P4 unordered lookup API | checker/pattern/P5/P6 | bespoke reflection lookup |
| static Record identity | `StaticValueStore` + FieldKey | evaluator/compiler | runtime Record object |
| runtime logical shape | `RecordProductShape` using FieldKey | VM/access/equality | heap-key identity |
| ordered labels | existing Symbol authorities | Tuple/Pack/calls | FieldKey widening |
| stable key encoding | `FieldKeyRef`/FieldKey stable encoder | metadata/incremental | stringify-everything encoding |
| public structural capability | P5 | generic constraints/reflection | P4 trait synthesis |
| final metadata publication | P8 | cross-module durability | P4 silent schema break |
| checkpoint state | C4 checkpoint | all plans | plan-local competing truth |

---

# 9. Global invariants

The following IDs are normative and must be referenced by tasks/tests.

```text
P4-INV-01  FieldKey is exactly Symbol | String | Int | Bool.
P4-INV-02  FieldKey identity is content/value based and independent of VM, store, range, and arena identity.
P4-INV-03  Canonical kind order is Symbol < String < Int < Bool.
P4-INV-04  Symbol and String with identical text are distinct.
P4-INV-05  Int and Bool are distinct.
P4-INV-06  Int keys use exact arbitrary-precision numeric identity/order.
P4-INV-07  Canonical Record rows sort once by FieldKey order and reject adjacent duplicates.
P4-INV-08  Duplicate Record keys are invalid on every construction route; no overwrite semantics.
P4-INV-09  Record presentation/source order is not structural type/equality/hash identity.
P4-INV-10  Required/optional Record semantics remain unchanged.
P4-INV-11  Open source rows do not satisfy closed targets by key-domain widening.
P4-INV-12  Anonymous Record tails carry key-domain information.
P4-INV-13  Captured tails preserve exact key/value correlation.
P4-INV-14  Anonymous open tails are not finite iterable field sequences.
P4-INV-15  Tuple labels remain Symbol.
P4-INV-16  Pack labels remain Symbol.
P4-INV-17  Call labels remain Symbol.
P4-INV-18  Non-Symbol Record keys cannot be forwarded as ordered/call labels.
P4-INV-19  Record-to-Record expansion preserves all FieldKey kinds.
P4-INV-20  Tuple/Pack-to-Record labeled expansion maps Symbol to FieldKey::Symbol only.
P4-INV-21  StaticValue::Record stores canonical FieldKey, not store-local text IDs.
P4-INV-22  Runtime Record structural identity contains no heap object identity or VM Symbol ID.
P4-INV-23  Runtime String key comparison is content-based without per-comparison allocation.
P4-INV-24  Fixed runtime Record lookup is O(log n) or better.
P4-INV-25  Record equality/hash ignore presentation order.
P4-INV-26  Record equality/hash do not expand structural reflection.
P4-INV-27  Record patterns resolve to canonical FieldKey before lowering.
P4-INV-28  Existing row solver remains the only row inference authority.
P4-INV-29  Existing relation/query budget/cancellation remain authoritative.
P4-INV-30  Symbolic static key computation does not create a P4-only residual engine.
P4-INV-31  Stable metadata never serializes VM/store-local key IDs.
P4-INV-32  Legacy string-name metadata can only mean Symbol key.
P4-INV-33  Non-representable heterogeneous metadata fails closed until a compatible durable schema exists.
P4-INV-34  P3 Pack type/static/runtime/product behavior remains intact.
P4-INV-35  EmptyRecord remains the zero-field immediate Record.
P4-INV-36  Product/Sum sealed capability derivation remains P5 scope.
P4-INV-37  Final public RecordFields ontology retirement remains P9 scope.
P4-INV-38  One canonical FieldKey order is used by rows, static Records, runtime logical shapes, fingerprints, and P5 inputs.
P4-INV-39  A key collision discovered after static reduction/specialization is diagnosed.
P4-INV-40  Invalid runtime Record key is rejected before its associated field value executes when the key guard is reached first.
P4-INV-41  Record-specific syntax/semantics never widen the shared ordered `ProductLabel` contract.
P4-INV-42  Runtime-computed keys never masquerade as fixed row fields.
P4-INV-43  P2 checked static semantics, not evaluator-side name lookup, determine fixed computed type/pattern keys.
P4-INV-44  Typed-tail fixed-key lookup returns Optional(V) only when key membership is proven inside the tail key domain.
P4-INV-45  Runtime Record logical key storage owns each content-heavy key once per shape, modulo shape interning.
P4-INV-46  Metadata staging is lossless even when active wire export must reject non-Symbol keys.
P4-INV-47  Computed Record key validation precedes value evaluation and preserves duplicate/error timing.
P4-INV-48  Row fixed-prefix correspondence is merge-based or otherwise demonstrably non-quadratic.
P4-INV-49  Changing only source field order does not change canonical row identity/fingerprint.
P4-INV-50  P4 lifecycle bookkeeping never erases or falsifies concurrent P3 state.
```

---

# 10. Non-goals

Do not expand P4 into:

- generalized heterogeneous labels for ordered products;
- user-definable FieldKey kinds;
- Map/Record unification;
- Record mutation;
- structural capability derivation;
- reflection API design beyond the internal lookup/shape facts P5 needs;
- general static expression residualization;
- a new metadata release/version unless separately ratified;
- whole-workspace cleanup of every `name` string that is not a Record field key;
- broad formatter/clippy churn;
- broad language-corpus repair;
- P3 closure work unrelated to a direct P4 regression.

---

# 11. Expected impact map

## 11.1 Expected source areas

### AST/parser

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/error.rs
phalcom-ast/tests/**
```

### Semantic scalar/rows/checker

```text
phalcom-semantic/src/field_key.rs                 # preferred new owner
phalcom-semantic/src/scalar.rs                    # or equivalent lower exact-int owner
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
phalcom-semantic/src/checker/call.rs               # Record tail/boundary audit
phalcom-semantic/src/checker/associated.rs         # row-tail audit
phalcom-semantic/src/checker/gadt_proof.rs         # row-field audit
phalcom-semantic/src/impls.rs                      # row-tail audit
```

### Static values/evaluator

```text
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/static_eval/**
phalcom-semantic/src/db/fingerprint.rs             # if row/static products fingerprint here
```

### Match semantics

```text
phalcom-semantic/src/match_semantics.rs
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/checker/coverage/**
```

### Metadata

```text
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/validate.rs
phalcom-type-meta/src/header.rs                    # only if staging requires a compatible change
phalcom-type-meta/tests/schema_compat.rs

phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/export.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
```

### Core runtime/compiler

```text
phalcom-core/src/product/shape.rs
phalcom-core/src/product/mod.rs
phalcom-core/src/product/view.rs
phalcom-core/src/heap/record.rs
phalcom-core/src/heap/record_literal_builder.rs
phalcom-core/src/heap/object.rs                    # exhaustive audit
phalcom-core/src/heap/trace.rs                     # builder/lifetime audit
phalcom-core/src/bytecode.rs                       # Record-only key guard if needed
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

## 11.2 Expected focused tests

Prefer these conceptual modules; conform names to live harness conventions at T0:

```text
phalcom-ast/tests/**record*field*key*

phalcom-semantic/tests/semantic/foundations/field_keys.rs
phalcom-semantic/tests/semantic/foundations/record_rows_field_keys.rs
phalcom-semantic/tests/semantic/foundations/static_values_record_keys.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
phalcom-semantic/tests/semantic/incremental/**record_rows**

phalcom-type-meta/tests/**field_key** or existing schema_compat.rs

phalcom-core/tests/core/collections/record_field_keys.rs
phalcom-core/tests/core/patterns/**record**
phalcom-core/tests/core/collections/outgoing_packs_completion.rs   # P3 boundary regressions only
```

## 11.3 Expected documentation/state

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4-luna-patch-grade-implementation-plan.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4-walkthrough.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4-handoff.md
```

Normative specs should be updated only where P4 changes user-visible Record key/tail/runtime behavior. Final P5/P8/P9-owned documentation must not be prematurely written as if those features exist.

## 11.4 Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary. STOP AND CONSULT before entering a materially different subsystem when that entry would change semantic authority, identity, ownership, lifetime, metadata versioning, ordered-label behavior, or checkpoint scope.

---

# 12. Implementer decision authority

## 12.1 FIXED

The implementer must not change:

- the four FieldKey variants;
- canonical key order;
- content-based Symbol/String identity and their distinction;
- exact arbitrary-precision Int identity;
- Int/Bool distinction;
- anonymous-tail default key domain (`FieldKey`);
- typed-tail lookup precision (`Optional(V)` on proven membership);
- existing row solver as the row authority;
- required/optional semantics;
- captured-tail correlation semantics;
- P2 static evaluator ownership;
- Symbol-only Tuple/Pack/call labels;
- no-coercion boundary for Record-to-call/ordered lanes;
- one-copy logical runtime shape architecture (`logical_keys + permutation`);
- binary-search baseline lookup;
- presentation-order preservation separate from structural identity;
- FieldKeyRef lossless staging plus fail-closed legacy export;
- P5/P8/P9 scope boundaries;
- evaluation-order rule: key validate before value;
- no duplicate overwrite semantics.

## 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- exact helper/type names other than public/ratified names;
- `field_key.rs` versus another clearly lower pure semantic module;
- `Arc<str>` plumbing details and compatibility constructors;
- exact index integer width in runtime shape after bounds validation;
- whether Record key conversion is a method on VM, product module, or dedicated core helper, provided there is one authority;
- exact bytecode opcode/helper name for Record-only key validation;
- exact test file/module layout;
- private enum names for lookup/membership outcomes;
- staging type placement inside `phalcom-type-meta`;
- whether source `record.ph` equality/hash use new floor primitives or a native helper, provided complexity/semantics match this plan;
- temporary compatibility re-exports for moved `StaticInt`.

## 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| `phalcom-core` can depend on/reuse semantic `FieldKey` without layering cycle | Cargo manifests + current semantic-lowering dependencies | STOP if a new cross-crate cycle would be required; propose lossless shared-lower-layer placement |
| moving/re-exporting `StaticInt` can preserve P2/P3 API | `static_value/store.rs`, module exports | adapt with compatibility re-export; STOP if duplicate exact-int semantics would otherwise be required |
| Record AST can be split without widening Tuple/Pack label AST | AST/parser call sites | adapt parser shape; STOP if shared AST semantics force ordered-label widening |
| type/pattern computed keys can consume checked P2 static facts | annotation/checker/static-eval seams | STOP if evaluator-side lookup/resolution would be needed |
| anonymous tail key domain can be represented by existing `TypeId` | row/type-lambda representations | STOP if a new kind/domain system is required |
| row relation can express key-domain membership through existing relation outcomes | `types/relation.rs` | STOP if a parallel subtype path would be needed |
| runtime shape can own semantic key content safely | `product/shape.rs`, GC/lifetime boundaries | STOP if ObjRef-backed structural identity appears necessary |
| binary search can use borrowed key comparison | `FieldKey` representation + product shape lookup | adapt helper signatures; do not allocate per comparison |
| current metadata schema cannot necessarily encode heterogeneous keys | type-meta header/codec | stage FieldKeyRef + fail closed; STOP before any incompatible silent schema change |
| P3 shared files still preserve Symbol-only Pack labels | P3 live paths | preserve mechanically; STOP on architecture drift |
| lifecycle convention supports or defines parallel-plan bookkeeping | lifecycle convention + checkpoint schema | follow it; if singleton metadata would overwrite live P3 truth, STOP on bookkeeping and consult |

---

# 13. Global STOP / CONSULT triggers

A triggered consultation cannot be self-waived.

STOP AND CONSULT if:

1. `RecordRowData` cannot carry FieldKey without a second row identity arena.
2. Anonymous tail key domains require a fundamentally new kind/static-domain system rather than a `TypeId` constraint.
3. Exact String/Symbol/Int/Bool key content cannot be extracted from P2 static values without store-local identity leakage.
4. Runtime Record shape cannot own/reuse content keys without heap identity or GC-lifetime coupling.
5. Heterogeneous Record construction would require widening Tuple/Pack/call labels.
6. Active metadata cannot stage FieldKey without either lossy conversion or an unratified incompatible schema release.
7. Existing optionality/width semantics materially conflict with the preserved rules.
8. Row solving would have to be replaced rather than migrated.
9. Static key expressions would require evaluator-side name/type resolution instead of checked semantic products.
10. Record patterns would require a second key/type resolver.
11. A proposed optimization changes observable presentation order.
12. Equality/hash would need to equate Symbol and String spellings.
13. Large-row evidence reveals unavoidable quadratic behavior in a canonical owner and fixing it changes architecture materially.
14. P3 has changed ProductShape/Pack/label ownership enough to invalidate the planned Record-specific split.
15. A new cross-crate dependency cycle is required to share FieldKey.
16. Record runtime key identity appears to require `ObjRef`, `Symbol(u32)`, `StaticStringId`, or `StaticSymbolId`.
17. A computed-key typing fix begins inventing fixed keys from runtime expressions.
18. A test oracle would require weakening any `P4-INV-*` invariant.
19. The same semantic failure persists after one serious evidence-backed correction.
20. A workaround/special-case architecture starts replacing a missing canonical seam.
21. The current worktree contains uncommitted P3 edits that P4 would overwrite or ambiguously modify and no isolated clean worktree is available.
22. Checkpoint lifecycle rules cannot represent P4 progress without falsifying P3 state.
23. Scope expands materially beyond the expected impact map.

---

# 14. Debugging budget

## 14.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress:

```text
inspect
  -> identify concrete mechanical cause
  -> make one coherent correction
  -> rerun smallest discriminating command
```

Examples: missed enum arm, signature drift, test-module export, moved helper path, borrow mismatch.

## 14.2 Semantic failures

Before changing code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic failure. If it persists, STOP AND CONSULT.

## 14.3 Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

Never rerun an unchanged failing test unless relevant code/configuration/evidence changed.

---

# 15. Testing surface analysis

| Coverage ID | Dimension | Invariants | Required evidence |
|---|---|---|---|
| CV-01 | scalar identity | 01–06 | all four kinds; same-text/tag distinctions; huge Int |
| CV-02 | canonical ordering | 03,06,07,38 | cross-kind + within-kind order |
| CV-03 | parser/source | 01,12,15–17,41 | mixed Record keys/tails/patterns; Tuple grammar unchanged |
| CV-04 | row canonicalization | 07–09,39,49 | order-independent RowId; duplicates; optionality retained |
| CV-05 | tail formation | 11–14,44 | typed/unconstrained key domains; invalid domain rejected |
| CV-06 | solver/lacks | 04–06,12,13,28,48 | lacks across all key kinds; captured tails; merge |
| CV-07 | relation/optionality | 10–14,29,44 | required/optional/width/tail matrix |
| CV-08 | lookup | 10–14,44 | Required/Optional/Absent/Unknown cases |
| CV-09 | static Record | 01–08,21,38 | all key kinds, equality/hash/fingerprint, duplicates |
| CV-10A | static Record value key bridge | 30,39 | static value->FieldKey; invalid value behavior |
| CV-10B | checked type/pattern key bridge | 30,39,43 | stable binding/product identity; dependencies, specialization, invalid/nonstatic behavior |
| CV-11 | runtime conversion | 01–06,22,23,40 | all key kinds, invalid key timing, huge Int |
| CV-12 | runtime shape | 09,22–24,38,45 | logical order, presentation permutation, lookup |
| CV-13 | runtime construction | 08,19,20,35,40,47 | fixed/dynamic/spread/empty/duplicate behavior |
| CV-14 | equality/hash | 09,25,26,38 | reordered equal mappings equal/hash equal; tag distinctions |
| CV-15 | call/Pack boundary | 15–20,34,41 | Symbol-only forwarding; no Pack regression |
| CV-16 | patterns | 01–06,27,43 | all key kinds; rest exclusion; semantic resolved key |
| CV-17 | metadata | 02,06,31–33,46 | FieldKeyRef round-trip/fingerprint; fail-closed legacy export |
| CV-18 | incrementality | 34,38,49 | key change invalidates; reorder stable; Pack unaffected |
| CV-19 | evaluation order | 08,40,47 | invalid/duplicate key prevents later value side effect where required |
| CV-20 | performance | 23,24,45,48 | 1k/10k formation, lookup, equality, solver merge; no quadratic path |
| CV-21 | query resources | 29 | budget/cancellation behavior unchanged through key-domain relation |
| CV-22 | P2 compatibility | 21,30,43 | static values/evaluator existing tests remain green |
| CV-23 | P3 compatibility | 15–20,34,41 | Pack labels/spreads/fixed-call path unchanged |
| CV-24 | lifecycle | 50 | P3 state preserved; P4 evidence recorded truthfully |

---

# 16. Verification execution budget

## 16.1 Modes

### BUILD MODE

Use exact new regressions and directly owning unit/integration selectors only. No broad language corpus. No whole workspace tests. No reflexive crate-wide tests after each task.

### STABILIZE MODE

Run P4's four coherent gates and directly affected compatibility selectors.

### CERTIFY MODE

Run affected crate checks and any specifically named P4 package-level verification. Do not silently promote to workspace/release certification.

## 16.2 Verification ladder

```text
T0 — exact reproducer/new regression
T1 — directly affected feature tests
T2 — owning subsystem/module suite
T3 — adjacent cross-layer integration
T4 — affected crate/package check
T5 — workspace/release verification (not required by P4 unless separately requested)
```

## 16.3 Mandatory during BUILD

- new exact regression for the task's semantic boundary;
- `cargo check` only for the directly affected crate(s) when compilation feedback is useful;
- P3 regression selector only when editing a P3-shared path affecting Pack/call behavior;
- `git diff --check` after coherent patch slices, not after every edit.

## 16.4 Mandatory at named gates

- G1 semantic FieldKey/row gate;
- G2 static/runtime Record vertical gate;
- G3 pattern/call/P3 interoperability gate;
- G4 metadata/incremental/resource gate.

## 16.5 Mandatory during STABILIZE

- affected crate checks for AST, semantic, type-meta, and core as actually touched;
- exact P2 static-value compatibility selectors;
- exact P3 Pack/call selectors that overlap shared code;
- relevant existing Record/row tests not superseded by new P4-specific tests.

## 16.6 Only if evidence demands

- broader semantic package tests;
- broader core package tests;
- metadata schema compatibility suites beyond touched codecs;
- predecessor comparison worktrees for newly observed failures not already classified.

## 16.7 Explicitly deferred / DO NOT RUN during BUILD

```text
cargo test --workspace
workspace clippy
full phalcom-core test package
full phalcom-semantic package
phalcom-core language-corpus
broad LSP/REPL suites
```

The checkpoint records that the broad language corpus is extremely slow and contains known unrelated failures. It is not a P4 focused gate absent a separate request.

---

# 17. Baseline / unrelated failure policy

Classify every non-passing verification result:

```text
A — definitely caused by P4
B — probably caused by P4
C — unclear
D — clearly unrelated/baseline
```

- A/B: active P4 responsibility.
- C: one bounded classification pass. If nonblocking and still unclear, record it and continue only when acceptance is not compromised.
- D: record and continue immediately.
- Never weaken assertions, skip tests, or add compatibility hacks merely to make a broad gate green.
- Existing checkpoint baseline failures remain D unless P4 evidence shows a causal change.
- The P3 root-versus-slice equality failure remains a P3 issue unless P4 modifies the failing path and changes the behavior.
- Record new deferred failures in the checkpoint ledger with exact selector, observed result, causal classification, and action.

Known checkpoint baselines include the previously classified module/REPL/core/semantic failures; do not spend P4 debugging budget rediscovering them unless they block a P4 acceptance selector.

---

# 18. Tasks

## T0 — Takeover, baseline, and P3 coexistence audit

### Purpose

Freeze the real P4 starting revision, protect concurrent P3 work, confirm that the requirements-audited architecture still matches live source, identify exact focused test selectors, and establish checkpoint/lifecycle state without claiming implementation behavior.

### Preconditions

- P2 remains complete and its static-value/evaluator seams are available.
- P4 requirements analysis is accepted.
- P3 may still be in progress.

### Consumes

- C4 checkpoint record;
- P3 interim handoff/walkthrough and active plan;
- P4 requirements analysis;
- lifecycle convention;
- live repository state.

### Produces

- exact P4 starting revision and worktree state;
- confirmed strategy for avoiding P3 dirty-worktree collisions;
- revalidated shared-file ownership map;
- exact focused test module/selector names used by later tasks;
- truthful P4 lifecycle start record.

### Files and symbols

**Read:** all paths in Sections 3–4, especially P3-shared files.

**Modify:** checkpoint/plan lifecycle records only if the lifecycle convention says P4 may be marked active at this point.

**Create:** no implementation source.

### Required implementation shape

1. Record:

   ```sh
   git status --short
   git rev-parse HEAD
   git log -1 --oneline
   ```

2. Compare live head against `b9f72cfe8ee5338485b716702cd9560776fce3f7`.
3. If the current primary worktree contains uncommitted P3 edits:
   - do not stash, reset, discard, amend, or absorb them into P4;
   - use a clean isolated worktree/branch at the intended P4 base, or wait until P3 changes are durably committed;
   - if the repository workflow forbids an isolated worktree and P4 must edit the same shared files, STOP AND CONSULT.
4. Re-read P3 Pack/product/static-value branches. Confirm:
   - `ProductShape::Pack` still exists;
   - Pack labels remain Symbol;
   - static Pack labels remain their P3 symbol representation;
   - fixed calls still bypass surface Pack/builder allocation;
   - P3 runtime slices/root ownership has not been refactored into a shared key model that changes P4 assumptions.
5. Inspect current row/Record tests and record exact module selectors for:
   - row formation/subtyping;
   - Record static values;
   - Record runtime construction/access/equality;
   - Record patterns;
   - P3 Pack/call compatibility.
6. Read lifecycle convention and determine how parallel P3/P4 progress is represented. Do not overwrite `active_plan: P3` merely to start P4 if that would make checkpoint metadata false.
7. Add a checkpoint planning/start note only after lifecycle representation is understood.

### Forbidden approaches

- no source implementation edits;
- no broad suite runs;
- no cleanup of P3 dirty state;
- no speculative refactor of shared product helpers;
- no overwrite of singleton lifecycle fields when P3 is still active unless the lifecycle convention explicitly directs it.

### Test changes required

None.

### Tests to run now

Use only read-only discovery plus the smallest known compatibility discriminators. If the current P3 selectors still exist, run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values_structural -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::outgoing_packs_completion::p3_pack_spreading_preserves_lanes_and_fuses_builders -- --nocapture
```

If those exact selectors drifted, discover the equivalent live selector before execution; do not broaden to a whole package merely to find it.

Do **not** rerun the known failing P3 root-vs-slice equality reproducer as a P4 baseline unless P4 will touch that exact equality path.

### Tests explicitly deferred

Everything else.

### Acceptance

T0 is complete when:

- starting revision/worktree are recorded;
- P3 dirty state is protected;
- live architecture still supports the P4 plan or a consultation has occurred;
- exact focused test selectors are known;
- lifecycle bookkeeping can represent P4 truthfully;
- no implementation claim has been made.

### Local STOP / CONSULT triggers

- any P3 architectural drift invalidates Section 7;
- no clean way exists to isolate P4 from uncommitted P3 edits;
- lifecycle rules cannot represent parallel P3/P4 progress truthfully;
- a new crate layering issue appears around semantic FieldKey reuse.

### Checkpoint update

Record only the actual starting revision, P4 start/planning state, and any coexistence/consultation decision. Do not promote verification.

---

## T1 — Canonical `FieldKey` scalar and Record-specific source syntax

### Purpose

Create the pure semantic FieldKey authority, unify exact integer representation, and introduce Record-specific value/type/pattern/tail syntax without yet migrating canonical rows.

### Preconditions

T0 complete; shared-file assumptions verified.

### Consumes

- existing `StaticInt` implementation;
- AST `ProductLabel`, Record literal/type/pattern syntax;
- P2 static-value scalar semantics;
- existing primitive `Symbol`, `String`, `Int`, `Bool` type identities.

### Produces

- canonical `FieldKey` type and operations;
- lower pure exact-int scalar owner/re-export;
- context-specific Record key AST;
- parser forms for heterogeneous Record type keys and key-domain tails;
- parser forms for heterogeneous Record patterns;
- source-visible `FieldKey` union alias in the canonical structural/Record module if not already present;
- no global ordered-label widening.

### Files and symbols

**Read:**

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/error.rs
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/types/annotation.rs
phalcom-core/core/universe/src/collections/record.ph
```

**Modify/Create:**

```text
phalcom-semantic/src/field_key.rs          # preferred
phalcom-semantic/src/scalar.rs             # or equivalent pure scalar module
phalcom-semantic/src/lib.rs / module exports
phalcom-semantic/src/static_value/store.rs # exact-int re-export only in this task
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/error.rs
phalcom-ast/tests/**
phalcom-core/core/universe/src/collections/record.ph  # FieldKey alias only if this is canonical owner
```

### Required implementation shape

#### T1.1 Extract/re-export exact integer scalar

Move the implementation, not the semantics, of `StaticInt` into a lower module usable by both static values and FieldKey. Preserve existing constructors, normalization, equality, hash, and exact BigInt behavior.

Preferred compatibility shape:

```rust
// lower module
pub enum StaticInt {
    Small(i64),
    Large(Arc<BigInt>),
}

// old module path may re-export temporarily
pub use crate::scalar::StaticInt;
```

Do not create `FieldKeyInt` or a second BigInt wrapper.

#### T1.2 Add canonical FieldKey

Implement explicit, manually reviewed semantic comparison. Do not rely on enum declaration order as the sole contract.

Required operations:

- kind/tag;
- exact equality;
- process-local `Hash` compatible with equality;
- `canonical_cmp`;
- stable tagged fingerprint encoding;
- deterministic diagnostic display/escaping;
- constructors from owned/borrowed content as mechanically useful.

`Arc<str>` is chosen storage. No interner ID participates in semantics.

#### T1.3 Split Record key syntax from ordered labels

Introduce Record-specific key syntax for:

```telos
{ name: String }
{ ["mime"]: String }
{ [42]: Handler }
{ [true]: Enabled }
```

and patterns:

```telos
{ name: x }
{ ["mime"]: x }
{ [42]: x }
{ [true]: x }
```

Bare identifiers remain Symbol keys.

Do not change Tuple/Pack/call label grammar or AST semantics.

#### T1.4 Add tail key-domain grammar

Accept:

```telos
{ **[String]: Handler }
{ **[Int]: Value }
{ **[FieldKey]: Value }
{ **[String]: ... }
{ **: Value }
{ ... }
{ **R }
```

Normalize syntax so semantic resolution can distinguish:

```text
captured tail **R
anonymous key+value constrained
anonymous key constrained/value unconstrained
anonymous default-FieldKey + value constrained
anonymous default-FieldKey + unconstrained value
```

Do not yet alter `RecordRowData` in this task.

#### T1.5 Source `FieldKey` type alias

If no canonical source alias exists, add the ordinary Telos alias in the module that owns Record structural vocabulary:

```telos
type FieldKey = Symbol | String | Int | Bool
```

Do not create a compiler-only pseudo-kind named FieldKey. The semantic type used for tail validation must be the ordinary canonical type/union meaning.

If making this alias available would require ad hoc name injection or a new prelude mechanism, STOP AND CONSULT rather than special-casing the parser/type resolver.

### Forbidden approaches

- deriving semantic order accidentally from enum variant order;
- storing Symbol IDs, StaticStringId, StaticSymbolId, source ranges, or TypeId in FieldKey;
- duplicating `StaticInt` semantics;
- globally widening `ProductLabel`;
- treating String syntax as Symbol;
- accepting Float or arbitrary Object as a FieldKey;
- using locale/Unicode normalization for key order.

### Test changes required

Create/extend focused tests for CV-01, CV-02, CV-03:

- all four FieldKey variants;
- cross-kind order;
- exact same-kind order;
- huge positive/negative Int;
- `#x != "x"`, `1 != true`, `0 != false`;
- stable fingerprint distinction;
- Record type parser forms;
- optional heterogeneous fields;
- tail forms;
- pattern forms;
- malformed bracket/tail syntax;
- Tuple/Pack/call label grammar unchanged.

### Tests to run now

Use the new exact selectors, conceptually:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::field_keys -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast record_field_key -- --nocapture
```

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast -p phalcom-semantic
```

Do not run runtime/core tests yet. Intermediate semantic row representation is intentionally not migrated.

### Tests explicitly deferred

Row solver, relation, runtime, metadata, patterns lowering.

### Acceptance

- FieldKey scalar semantics pass all CV-01/CV-02 tests.
- parser/AST can represent required Record forms without changing ordered-label AST semantics.
- existing P2 static values still compile using the shared exact-int scalar.
- no row identity has been partially migrated in a way that leaves mixed string/FieldKey authorities.

### Local STOP / CONSULT triggers

- exact-int extraction would require changing P2 numeric semantics;
- Record syntax cannot be separated from ordered labels without global widening;
- source `FieldKey` alias needs a new name-resolution special case.

### Checkpoint update

Record only if the FieldKey scalar becomes a durable implemented interface and the focused scalar/parser tests actually pass.

---

## T2 — Canonical row/store/tail migration

### Purpose

Migrate the canonical Record row coordinate from string names to FieldKey and add key-domain-aware anonymous tails through TypeStore, scoped rows, substitution, and instantiation.

### Preconditions

T1 FieldKey/parser infrastructure complete and focused-tested.

### Consumes

- `FieldKey`;
- canonical `RecordRowData`/`RecordRowId`;
- `RecordRowTail` and `AnonymousRecordRemainder`;
- TypeStore row interning;
- scoped/type-lambda row forms;
- substitution/instantiation machinery.

### Produces

- `RecordRowField.key: FieldKey`;
- canonical FieldKey sort/duplicate detection;
- anonymous remainder key type + optional value type;
- FieldKey-based row find/lookup primitives;
- migrated scoped/inference/stable row field and lacks representations needed by later tasks;
- one checked row-formation path used by rebuild/substitution/specialization.

### Files and symbols

**Read/Modify:**

```text
phalcom-semantic/src/types/row.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/rigid.rs          # if live row forms use it
phalcom-semantic/src/checker/row_inference.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/metadata/export.rs       # bounded legacy-schema compile bridge; T8 owns lossless schema staging
```

### Required implementation shape

#### T2.1 Canonical field migration

Replace canonical identity:

```rust
name: Box<str>
```

with:

```rust
key: FieldKey
```

Remove, rather than preserve, a shadow canonical `name` field. Source spelling/provenance may remain outside canonical row identity for diagnostics.

#### T2.2 One canonical row constructor

Ensure all row formation/rebuild paths call the same checked canonicalizer:

```text
sort by FieldKey::canonical_cmp
  -> reject exact duplicates
  -> preserve presence and payload TypeId
  -> validate tail key domain
  -> intern
```

Binary search is the fixed-field lookup baseline.

Required identity discriminator:

```telos
{ a: Int, ["b"]: String }
{ ["b"]: String, a: Int }
```

must produce the same canonical row identity.

`#x` and `"x"` may coexist.

#### T2.3 Anonymous tail key domain

Migrate the remainder to carry `key_type` for both unconstrained-value and typed-value cases.

Defaults created from syntax:

```text
**: V -> key_type = FieldKey
...   -> key_type = FieldKey
```

`**[K]: V` uses resolved `K`.

Validate `K` using ordinary type semantics: it must be a proper type provably admissible as a FieldKey domain according to the existing type-boundary policy. Do not check type names.

#### T2.4 Captured tails

Keep `RecordRowTail::Parameter` and `Projection` correlation semantics unchanged. Do not flatten captured rows into unions of key/value types.

#### T2.5 Scoped/generic representations

Migrate every row field/lack coordinate that participates in generic application or substitution:

```text
ScopedRecordField
InferenceRecordField
RecordRowTerm
RecordRowLacks
StableRecordRowLack
row failure witnesses where mechanically required
```

Key-domain types that reference scoped binders must use the same capture-safe substitution machinery as value types.

#### T2.6 Record type-key static resolution seam

Wire parsed fixed Record type keys to canonical FieldKey **only** through a checker-owned static-resolution projection that consumes P2 checked/static facts.

Required result shape conceptually:

```text
Reduced concrete FieldKey -> may enter RecordRowData
Symbolic/open static result -> remain in established static-definition/specialization authority; do not intern a concrete row yet
Runtime/non-static result   -> record.key_not_static
Invalid static value kind   -> record.invalid_field_key / key_not_static
```

Literal fast paths are allowed only if they are semantically equivalent projections of the checked static literal and do not create a second evaluator/name resolver.

When specialization finally yields a concrete key, canonical row formation reruns duplicate detection, satisfying P4-INV-39.

#### T2.6 consultation amendment — 2026-10-05

The mandatory consultation confirmed that the live P2 static-constant product is
not currently available to the type-form path. `StaticConstantProduct` carries a
stable `StaticConstantId(SymbolId)`, checked initializer, reduced value, and
query dependencies, but session type-form processing precedes same-module static
constant publication. `TypeResolver` and `CheckingContext::resolve_type_form`
also expose no checked expression/static-value projection for Record keys.

For the current T2 slice:

- bare Symbol keys continue to project directly to `FieldKey::Symbol`;
- a computed key may be projected only when its AST is a supported static
  literal and conversion exactly preserves P2 literal semantics;
- computed names, calls, operators, const references, and generic/symbolic
  results must not be evaluated or inserted into `RecordRowData` by annotation
  resolution; no placeholder field identity is permitted;
- do not add expression/name lookup to `TypeResolver` or create a second key
  evaluator.

This first amendment bounded the implementation but left T2 acceptance blocked
on the type-form key seam. The follow-up architecture consultation below
supersedes that sequencing consequence: T2 may complete its structural
row/tail substrate while const-reference, general-expression, and
generic/symbolic type/pattern keys continue to fail closed. The deferred seam is
assigned to T7.0 after the static Record value bridge and runtime Record tasks.

#### T2.7 metadata compatibility compile bridge — 2026-10-05

A read-only architecture consultation confirmed that T2 must not migrate the
shared `phalcom-type-meta` schema. T8 owns lossless `FieldKeyRef` staging,
fingerprints, codec/compatibility audit, and its tests. To keep the semantic
crate compiling during T2, the bounded `phalcom-semantic/src/metadata/export.rs`
write-set is added:

- legacy `name` fields accept `FieldKey::Symbol` only;
- legacy anonymous remainders are exported only when their key domain is the
  exact nominal `Symbol` type; typed remainders preserve their value type;
- String/Int/Bool keys and all other remainder key domains return an explicit
  metadata export error;
- apply the same boundary to static row arguments and scoped Record,
  OpenRecord, and RecordRow forms;
- do not modify `phalcom-type-meta`, stringify heterogeneous keys, or drop a
  remainder key/value type.

This is a compile-integration amendment only. It does not satisfy T8 metadata
acceptance or close T2's separate static-constant key-resolution seam. The
implementation incident is recorded in
`LANG007.C4.P4-T2-implementation-incident-2026-10-05.md`.

#### T2.8 acceptance and dependency amendment — 2026-10-05

A follow-up architecture consultation resolved the T2/T3/T4 dependency loop.
T2 owns the canonical `FieldKey` row/tail substrate and direct supported-literal
projection. For this structural gate, const references, general expressions,
and generic/symbolic keys must fail closed and must never enter
`RecordRowData`; they do not block T2 completion once this behavior is tested.

- T3's precondition is the completed, focused-tested canonical row/tail
  substrate. T3 does not depend on type-annotation expression evaluation.
- T4 owns static Record **value** evaluation and value-to-FieldKey conversion;
  it does not resolve Record type annotations or pattern keys.
- Split CV-10 into CV-10A (static Record value conversion/construction) and
  CV-10B (checked type/pattern key resolution, dependencies, specialization,
  and post-specialization duplicate detection).
- Add T7.0 after T4, T5, and T6 and before T7 pattern lowering. T7.0 wires type
  and pattern key consumers to one checker-owned projection. It must use the
  binding selected by ordinary checking, stable `SymbolId`/
  `StaticConstantId`, the current `StaticConstantProduct`, and targeted
  dependency invalidation. It must not add evaluator/name lookup to
  `TypeResolver`, intern symbolic keys, or create a pattern-only evaluator.
- Keep CV-10B and P4/P4-INV-43 open until T7.0 is implemented and verified.

T7.0 must cover literal parity; const/operation conversion; invalid/nonstatic
diagnostics with no row insertion; const-only targeted invalidation;
generic/symbolic specialization; post-specialization collisions; and shared
type/pattern projection without a duplicate evaluator. If checked facts cannot
provide this authority without a parallel evaluator or resolver, stop and
consult before implementation.

### Forbidden approaches

- partial migration leaving some canonical row identities as strings;
- `key + optional name` dual identity;
- source-order interning;
- mutating interned rows in place;
- type-name checks for `Symbol`/`String`/etc.;
- converting captured tails to anonymous key/value unions;
- inserting unresolved symbolic key expressions into `RecordRowData`;
- evaluator-side source-name lookup.

### Test changes required

Cover CV-04 and CV-05, plus the T2 deferred-key fail-closed boundary:

- mixed canonical sort;
- same fields/different source order same `RecordRowId`;
- duplicates for each kind;
- Symbol/String same text coexist;
- Int/Bool distinct;
- huge Int;
- optional presence retained;
- each tail syntax normalizes correctly;
- invalid key-domain type rejected;
- scoped key-domain substitution;
- supported static literal key projects to FieldKey;
- const-reference, general-expression, and symbolic/generic keys fail closed
  and cannot enter a fixed canonical row;
- post-specialization collision is covered by T7.0 / CV-10B.

### Tests to run now

Use new focused semantic selectors only, e.g.:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::record_rows_field_keys::canonical_ -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::record_rows_field_keys::tail_ -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_annotations::record_type_static_literal_keys_reduce_to_canonical_field_keys -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_annotations::record_type_names_and_general_expressions_fail_closed_before_row_interning -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_annotations::anonymous_record_tail_key_domains_default_and_substitute_correctly -- --nocapture
```

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

### Tests explicitly deferred

Full solver/relation matrix; runtime; patterns; metadata.

### Acceptance

- all canonical/scoped row coordinate types use FieldKey where they denote Record keys;
- anonymous tails have key domains;
- one canonical checked row formation path owns ordering/duplicates;
- captured-tail identity remains unchanged;
- supported literal type keys project directly without a second evaluator;
- unsupported computed/symbolic type keys fail closed and cannot bypass P2
  static semantics; their checked projection is deferred to T7.0.

### Local STOP / CONSULT triggers

- key-domain types require a new kind system;
- substitution cannot carry key-domain `TypeId` without new architecture;
- static type-key reduction lacks a checker-owned P2 seam and would require evaluator-side resolution;
- a row identity split into legacy/new stores appears necessary.

### Checkpoint update

Record the canonical row/tail interface only after focused T2 evidence passes.

---

## T3 — Row solver, inference, relation, and unordered lookup convergence

### Purpose

Make the existing row calculus fully FieldKey-aware, remove avoidable quadratic fixed-prefix scans, and establish the shared unordered lookup semantic later consumed by patterns and P5/P6.

### Preconditions

T2 canonical row/tail substrate complete and focused-tested. The deferred
type/pattern expression-key projection is assigned to T7.0 and is not a T3
precondition.

### Consumes

- canonical FieldKey rows/tails;
- existing `RecordRowSolver`;
- lacks constraints;
- record subtype relation;
- inference terms/bounds;
- query budget/cancellation.

### Produces

- FieldKey lacks/failure witnesses;
- merge-based fixed-prefix unification/correspondence;
- key-domain-aware tail compatibility;
- shared `UnorderedFieldLookup` semantic;
- expected Record field typing using the same lookup;
- runtime-computed Record literal keys represented conservatively rather than as fixed keys.

### Files and symbols

```text
phalcom-semantic/src/types/row_solver.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/checker/row_inference.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/impls.rs
```

### Required implementation shape

#### T3.1 FieldKey lacks

Migrate all lacks coordinates and row solver diagnostics to exact FieldKey.

Distinct constraints:

```text
lacks(#x)
lacks("x")
```

must remain independent.

#### T3.2 Merge-based solver correspondence

Rows are canonical. Replace repeated `.find()`/`.any()` correspondence over fixed prefixes with two-pointer merge traversal where the algorithm compares two canonical field sets.

Payload terms continue using existing generic solver logic.

Do not alter occurs/cycle detection semantics.

#### T3.3 Key-domain relation helper

Add one thin helper that maps a concrete FieldKey to its ordinary semantic type and asks the existing relation engine whether it is within `key_type`.

It must preserve rich outcomes:

```text
proven inside
proven outside
unknown/blocked/cancelled/budget etc.
```

Do not coerce uncertainty to false.

#### T3.4 Tail subtype rules

Implement/retain:

```text
Closed <: compatible open target
Typed(Ks,Vs) <: Typed(Kt,Vt)       when Ks <: Kt and Vs <: Vt
Typed(Ks,Vs) <: Unconstrained(Kt)  when Ks <: Kt
Unconstrained(Ks) <: Unconstrained(Kt) when Ks <: Kt
Unconstrained(_) </: Typed(_, _)
open source </: Closed target
```

Captured tails continue through existing correlation/proof machinery.

#### T3.5 Optional absent-field rule

For target optional `k:T` absent from source fixed fields:

```text
source Closed                         -> absence acceptable
source Typed(K,V), k outside K        -> absence acceptable
source Typed(K,V), k inside K         -> require V <: T
source Typed(K,V), membership unknown -> preserve blocked/unknown policy
source Unconstrained(K), k outside K  -> absence acceptable
source Unconstrained(K), may contain k-> cannot prove value compatibility
captured unresolved                   -> preserve existing blocked behavior
```

Do not regress the prior corrected law that optional source does not satisfy required target.

#### T3.6 Shared unordered lookup

Implement one API with semantics:

```text
fixed Required(T) -> Required(T)
fixed Optional(T) -> Optional(T)
absent + Closed -> Absent
absent + Typed(K,V), outside -> Absent
absent + Typed(K,V), inside -> Optional(V)
absent + Typed(K,V), uncertain -> Unknown
absent + Unconstrained(K), outside -> Absent
absent + Unconstrained(K), otherwise -> Unknown
captured exact row -> recurse through same lookup
captured unresolved -> Unknown/blocked evidence
```

#### T3.7 Runtime-computed Record literal typing

Audit existing computed Record key typing. Required rule:

- if checker/P2 static authority reduces the key to concrete FieldKey, use fixed field;
- otherwise do not fabricate one;
- if the key expression's ordinary type is proven within FieldKey, represent it conservatively through the existing anonymous-tail machinery using the narrowest sound key/value domain the existing inference/join infrastructure can produce;
- if multiple dynamic entries cannot be soundly combined without introducing new row-correlation machinery, widen through existing type-join policy (up to FieldKey / existing common value type) rather than creating a new solver;
- if no sound existing join exists, STOP AND CONSULT instead of inventing a parallel representation.

### Forbidden approaches

- new unordered solver;
- key-name stringification inside solver;
- repeated nested scans on canonical fixed prefixes;
- loss of captured-tail correlation;
- Bool-only internal relation decisions;
- treating Unknown as Refuted;
- inventing fixed row fields from runtime expressions.

### Test changes required

Cover CV-06, CV-07, CV-08, CV-20, CV-21:

- lacks all four kinds;
- same-text Symbol/String distinction;
- row extension/lacks propagation;
- two-row merge;
- captured tail and row variable cycle behavior unchanged;
- required/optional matrix;
- extra field vs closed/typed/unconstrained tails;
- key-domain compatible/incompatible cases;
- all tail-to-tail rules;
- optional absent-field matrix;
- lookup Required/Optional/Absent/Unknown;
- query budget/cancellation preserved;
- 1k/10k solver merge discriminator proving no obvious quadratic fixed-prefix path.

### Tests to run now

Use focused new semantic selectors, e.g.:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::record_rows_field_keys::solver_ -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::record_rows_field_keys::subtype_ -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::record_rows_field_keys::lookup_ -- --nocapture
```

Run the exact existing optional-record regression selector identified in T0.

Do not run full semantic package yet.

### Tests explicitly deferred

Static/runtime Record vertical, patterns, metadata.

### Acceptance

- solver/lacks/relation use FieldKey end-to-end;
- fixed-prefix correspondence is non-quadratic by architecture;
- existing optionality rules remain intact;
- key-domain tail semantics pass the matrix;
- one lookup API exists for later consumers;
- query budget/cancellation still terminate relation work.

### Local STOP / CONSULT triggers

- preserving optionality requires ad hoc exceptions per key kind;
- relation cannot classify concrete-key membership without a second subtype path;
- row solver needs replacement rather than migration;
- dynamic-key literal typing has no sound projection into existing tail/join semantics.

### Checkpoint update

Record T3 as a durable semantic gate only after G1-level row semantics are locally green; otherwise keep task partial.

---

## T4 — Static Record and static-key bridge

### Purpose

Migrate static Record identity to FieldKey and make P2 static evaluation the canonical bridge for Record key values without contaminating Pack/static IDs.

### Preconditions

T1 FieldKey/exact-int and T2/T3 canonical row semantics available.

### Consumes

- `StaticValueStore` structural values;
- P2 static evaluator;
- FieldKey stable encoding;
- checked Record construction facts;
- P3 `StaticValueData::Pack` branches as preservation constraints.

### Produces

- `StaticRecordField { key: FieldKey, value: StaticValueRef }` or equivalent;
- canonical `intern_record(Vec<(FieldKey, StaticValueRef)>)`;
- `StaticValueStore::field_key_of` pure conversion;
- static Record equality/hash/fingerprint using canonical key order;
- evaluator support for computed static Record keys;
- no Pack label/generalization changes.

### Files and symbols

```text
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/static_eval/**
phalcom-semantic/src/db/fingerprint.rs        # as required
phalcom-semantic/src/checker/static_body.rs  # only if checked construction facts need projection
```

### Required implementation shape

#### T4.1 Static Record representation

Replace string key fields with FieldKey. `intern_record` must:

1. validate child refs belong to the store lineage;
2. sort by `FieldKey::canonical_cmp`;
3. reject exact duplicates;
4. intern the canonical field sequence.

Source order is irrelevant to static Record identity.

#### T4.2 Static conversion

Implement one pure conversion:

```rust
field_key_of(StaticValueRef) -> Result<FieldKey, NotAFieldKey>
```

Resolve String/Symbol IDs to owned content before returning. Returned FieldKey must survive independently of those IDs.

#### T4.3 Static evaluator construction

For a static Record field:

```text
evaluate key expression under existing checked static authority
  -> convert reduced value to FieldKey
  -> if invalid, fail closed
  -> evaluate value according to ordinary source/evaluator order
  -> canonical Record intern at completion
```

No runtime VM conversion is used.

#### T4.4 Fingerprints

Fingerprint static Records in canonical FieldKey order:

```text
record tag
field count
for each canonical field:
    FieldKey stable bytes
    child content fingerprint
```

No StaticStringId/StaticSymbolId is written to durable fingerprints.

#### T4.5 Pack isolation

Audit every changed exhaustive match. `StaticPackBacking.labels` remain P3 Symbol IDs. Do not rewrite Pack equality/hash/fingerprint to FieldKey.

### Forbidden approaches

- store-local IDs as key identity;
- source-order static equality/hash;
- runtime conversion calls;
- broadening `StaticPack` labels;
- lossy BigInt conversion;
- accepting arbitrary static values as FieldKeys.

### Test changes required

Cover CV-09, CV-10A, CV-19, CV-22, CV-23:

- intern each key kind and mixed keys;
- same mapping/different input order same semantic equality/hash/fingerprint;
- duplicate each kind rejected;
- Symbol/String same text distinct;
- Int/Bool distinct;
- exact huge Int;
- `field_key_of` valid/invalid matrix;
- evaluator computed keys for static Record values (not type annotations or
  pattern keys);
- invalid key evaluation timing;
- P2 existing static structural-value tests unchanged;
- P3 static Pack tests unchanged.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values_record_keys -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values_pack -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

Also rerun the exact P2 `static_values_structural` selector from the checkpoint if it remains separate.

### Tests explicitly deferred

Core runtime, patterns, metadata durability.

### Acceptance

- Static Record canonical identity is FieldKey-based and store-ID independent;
- static evaluator can construct mixed-key Records through P2 checked evaluation;
- stable fingerprints are content-based;
- Pack static behavior is unchanged.

### Local STOP / CONSULT triggers

- P2 static evaluator cannot expose checked key expression results without new name resolution;
- static Record and row key semantics diverge;
- preserving Pack branches requires shared-key generalization.

### Checkpoint update

Record the Static Record FieldKey seam and focused evidence if complete.

---

## T5 — Runtime Record shape, key conversion, construction, and lookup

### Purpose

Generalize runtime Record shape/storage/build/access from Symbol-only keys to canonical content-owned FieldKey while preserving presentation order, EmptyRecord behavior, GC safety, and fast lookup.

### Preconditions

T1 FieldKey scalar and T3 semantic lookup/type rules established. T4 static Record may proceed before or alongside this task, but runtime work must not depend on unfinished static metadata.

### Consumes

- semantic `FieldKey`;
- `ProductStorage` and current Record shape registry;
- current Record presentation/logical mapping;
- compiler lowering specs for analyzed anonymous products;
- VM Record literal builder/BuildRecord paths;
- runtime String/Int/LargeInt/Symbol/Bool value representations.

### Produces

- runtime FieldKey conversion authority;
- one-copy canonical `RecordProductShape`;
- O(log n) key lookup baseline;
- generalized fixed/dynamic Record construction;
- heterogeneous Record key observation/access floor;
- Record-specific key validation preserving source order;
- no change to Pack/Tuple/call label representation.

### Files and symbols

```text
phalcom-core/src/product/shape.rs
phalcom-core/src/product/mod.rs
phalcom-core/src/product/view.rs
phalcom-core/src/heap/record.rs
phalcom-core/src/heap/record_literal_builder.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/primitive/record.rs
phalcom-core/src/typing/overlay.rs
phalcom-core/src/typing/environment.rs
phalcom-core/src/typing/inspect.rs
```

### Required implementation shape

#### T5.1 Runtime conversion authority

Create one conversion seam, mechanically placed where current VM/product ownership fits:

```rust
fn value_to_field_key(&self, value: Value) -> Result<FieldKey, RecordKeyError>
fn field_key_to_value(&mut self, key: &FieldKey) -> Result<Value, VMError>
```

`value_to_field_key` rules:

- Symbol: resolve canonical spelling/content;
- String: read immutable content and own/share semantic content;
- immediate Int: exact integer;
- LargeInt object: exact BigInt content;
- Bool: bool;
- all others: structured Record-key type error.

No heap reference survives conversion.

`field_key_to_value` is observation/materialization only. It may allocate String/LargeInt values.

#### T5.2 RecordProductShape

Migrate shape storage to:

```rust
logical_keys: Box<[FieldKey]>
presentation_to_logical: Box<[u32]>
```

or a mechanically equivalent compact integer index.

Formation:

1. consume presentation-order `(FieldKey, Value)` entries or lowering spec;
2. canonical-sort key/index pairs by FieldKey order;
3. reject adjacent duplicate keys;
4. derive `presentation_to_logical`;
5. materialize ProductStorage in canonical logical order;
6. retain presentation permutation;
7. use the existing shape registry/interning authority.

Do not own two copies of String/BigInt keys merely to preserve presentation order.

#### T5.3 Lookup

Implement borrowed binary search against `logical_keys`:

```text
O(log n)
no String allocation
no BigInt materialization/allocation
```

If the existing shape cache is tightly typed to Symbol, remove or bypass it for Record rather than creating a lossy interning layer. Only retain an O(1) heterogeneous cache if it can use FieldKey content safely and later evidence shows benefit.

#### T5.4 Analyzed/static Record lowering spec

Migrate Record lowering facts from string labels to exact FieldKeys. Prefer:

```rust
Record {
    logical_keys: Box<[FieldKey]>,
    source_to_logical: Box<[u32]>,
}
```

The compiler consumes checked semantic keys. It must not re-parse source label text or re-sort by runtime Symbol spelling.

Validate permutation length/range/uniqueness in `finish_record_from_spec` or equivalent fail-closed boundary.

#### T5.5 Dynamic Record literal builder

Replace Symbol-only builder entries with FieldKey entries after key validation:

```rust
Vec<(FieldKey, Value)>
```

or an equivalent representation that does not root raw String/LargeInt key objects as structural identity.

Convert/validate key before inserting and before evaluating the associated value when the compiler's current evaluation sequence reaches the key guard first.

If a new bytecode guard is useful, it is Record-specific (conceptually `GuardFieldKey`). Do not alter Tuple/Pack `GuardSymbol`.

#### T5.6 Fixed BuildRecord path

BuildRecord should consume runtime key Values, convert to FieldKey, reject invalid/duplicate keys, and finish a canonical Record.

The name of the bytecode may remain unchanged.

#### T5.7 Access/observation floor

Generalize the runtime Record primitive surface so a heterogeneous key can be observed and used for lookup.

Preferred direction:

```text
rawKeyAt(index)         -> runtime Symbol/String/Int/Bool
rawValueAt(index)       -> presentation-order value if existing surface requires it
rawLookup(key)          -> direct canonical lookup
rawLogicalValueAt(i)    -> optional internal floor for O(n) equality/hash
```

A legacy `rawLabelAt` may remain only for Symbol-specific compatibility callers and must reject/fail on non-Symbol keys rather than misrepresent them.

Final public naming cleanup can remain P9; semantic correctness cannot.

#### T5.8 EmptyRecord

Preserve EmptyRecord immediate zero-field representation exactly. No new heap allocation for empty Record.

#### T5.9 Runtime type overlay

Migrate runtime Record field typing overlays from name strings to a heterogeneous stable key representation. Do not collapse non-Symbol fields in runtime type inspection.

### Forbidden approaches

- `ObjRef` in shape identity;
- VM `Symbol(u32)` in canonical Record shape;
- per-comparison String allocation;
- duplicate owned presentation/logical key arrays;
- global `ProductKey` abstraction that forces Pack/Tuple widening;
- last-write-wins duplicate behavior;
- source string re-resolution in compiler/runtime;
- evaluating a field value before rejecting an already-invalid key when current source order requires the guard first.

### Test changes required

Cover CV-11, CV-12, CV-13, CV-19, CV-20:

Construction:

- fixed Symbol/String/Int/Bool keys;
- mixed keys;
- huge Int;
- empty Record unchanged;
- dynamic computed key;
- invalid key rejects before field-value side effect;
- duplicate computed key fails;
- same-text Symbol/String coexist;
- Int/Bool distinct.

Access/shape:

- get each key kind;
- missing key;
- presentation iteration remains source/runtime encounter order;
- internal logical order follows FieldKey order;
- binary lookup matches all keys;
- 1k/10k mixed key formation and repeated lookup complete without quadratic scan structure.

Lowering:

- analyzed Record spec contains exact FieldKeys;
- compiler/runtime consume semantic key facts without re-resolution;
- malformed permutation/spec fails closed in direct unit coverage if such validation already has a test seam.

### Tests to run now

Create/use a focused core module, e.g.:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::record_field_keys::construction_ -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::record_field_keys::lookup_ -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::record_field_keys::evaluation_order_ -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

Because `product/shape.rs`, `compiler/lib/expr.rs`, and `vm/dispatch.rs` overlap P3, also run the exact P3 spread/fixed-call selector identified in T0.

### Tests explicitly deferred

Record equality/hash source surface, patterns, metadata/incremental broad gate.

### Acceptance

- runtime Record shape owns exact FieldKey content, not heap IDs;
- presentation and logical order remain separate;
- fixed lookup is O(log n) baseline and comparison allocation-free;
- all four key kinds construct/access correctly;
- invalid/duplicate key timing is correct;
- EmptyRecord remains immediate;
- P3 Pack path still passes its focused regression.

### Local STOP / CONSULT triggers

- sharing semantic FieldKey creates a crate dependency cycle;
- Record shape requires heap refs for String/LargeInt keys;
- current ProductShape abstraction forces a shared key type with Pack/Tuple;
- preserving presentation order requires duplicating content-heavy key arrays with no alternative;
- compiler cannot consume semantic FieldKey lowering facts without source re-resolution.

### Checkpoint update

Record runtime FieldKey/shape seam and focused evidence only after T5 selectors pass.

---

## T6 — Record equality/hash/spread and P3 ordered-label boundary

### Purpose

Complete Record structural behavior over canonical FieldKeys, make equality/hash linear in field count, generalize Record-to-Record spreading, and enforce the Symbol-only boundary when Records feed calls/Pack/Tuple labeled lanes.

### Preconditions

T5 runtime shape/access available; T3 semantic key-domain facts available.

### Consumes

- canonical logical Record shape;
- source `record.ph` behavior;
- current Record/Tuple/Pack/call spread helpers;
- P3 `ArgumentPackBuilder` and Pack spread paths;
- compiler semantic shape information when statically known.

### Produces

- O(n) Record equality/hash in canonical logical order;
- heterogeneous Record-to-Record spread;
- Symbol-only Record-to-call/ordered-lane enforcement;
- no String->Symbol coercion;
- explicit split between unordered Record entry helpers and ordered Symbol-lane helpers.

### Files and symbols

```text
phalcom-core/core/universe/src/collections/record.ph
phalcom-core/src/product/mod.rs
phalcom-core/src/product/view.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/heap/pack_builder.rs              # only if boundary helper lives here
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/expression.rs
phalcom-core/tests/core/collections/outgoing_packs_completion.rs
```

### Required implementation shape

#### T6.1 Equality

Replace nested presentation-key search with canonical logical walk.

Minimum semantic algorithm:

```text
if field counts differ -> false
for logical i in 0..n:
    canonical keys must be equal
    corresponding values compare by Telos ==
```

Do not make presentation order observable in equality.

Implementation options:

- source `record.ph` over logical floor accessors;
- native key-shape comparison plus source value comparison;
- another equivalent architecture that is O(n) and uses canonical keys.

Do not add a reflection dependency.

#### T6.2 Hash

Hash canonical logical sequence:

```text
Record domain tag
field count
for each logical field:
    key semantic hash
    value semantic hash
```

Hash must be consistent with equality across different presentation order.

#### T6.3 Record-to-Record spread

Preserve every FieldKey kind and normal duplicate detection.

Dynamic spread must append existing FieldKey content directly to the Record destination builder/shape path. It must not materialize keys as strings and reparse them.

#### T6.4 Tuple/Pack-to-Record

Ordered Symbol labels project as:

```text
Symbol -> FieldKey::Symbol
```

No other change to ordered products.

#### T6.5 Record-to-call/Pack/Tuple labeled lanes

At semantic checking time, when the Record shape is known and includes any String/Int/Bool key, reject labeled spread into a call or ordered labeled lane with a specific diagnostic such as `record.spread_non_symbol_call_key`.

For open/dynamic shapes that cannot prove all keys are Symbol, preserve the existing language policy for dynamic calls but insert/retain runtime enforcement:

```text
for each Record entry:
    FieldKey::Symbol -> append as Symbol label
    otherwise        -> fail; never coerce
```

If the checker can prove a tail key domain is a subtype of Symbol, allow it. If it can prove a non-Symbol possibility, reject statically. If uncertain under existing relation policy, use the established dynamic/runtime fallback rather than treating Unknown as false/safe.

#### T6.6 Split shared helpers

Audit helpers such as generic “labeled lane” snapshots/compilers. Any helper that currently conflates Record fields with ordered labels must split into:

```text
unordered Record FieldKey entry path
ordered Symbol-labeled lane path
```

Do not create a global `Label = FieldKey` alias.

### Forbidden approaches

- presentation-order equality/hash;
- nested O(n²) Record equality;
- reflection-based equality/hash;
- String/Int/Bool call-label coercion;
- generic helper widening that changes P3 Pack labels;
- constructing an intermediate surface Pack merely to spread/call;
- changing fixed-call direct invocation.

### Test changes required

Cover CV-14, CV-15, CV-19, CV-20, CV-23:

Equality/hash:

- same mapping/different source order equal and same hash;
- same text Symbol/String differs;
- Int/Bool differs;
- value mismatch differs;
- large reordered Records compare linearly by algorithm and complete under synthetic gate.

Spread/boundary:

- Record->Record preserves all key kinds;
- Tuple/Pack Symbol lanes -> Record become Symbol FieldKeys;
- all-Symbol Record `**record` call spread works;
- known String/Int/Bool Record call spread rejected semantically;
- dynamic non-Symbol Record fails at runtime;
- no String->Symbol coercion;
- Pack labeled spread unchanged;
- Tuple labeled behavior unchanged;
- fixed-call no-builder/no-surface-Pack invariant unchanged;
- duplicate from an earlier spread prevents later value evaluation where existing language order requires it.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::record_field_keys::equality_hash_ -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::record_field_keys::spread_ -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::outgoing_packs_completion::p3_pack_spreading_preserves_lanes_and_fuses_builders -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::outgoing_packs_completion::p3_duplicate_from_earlier_pack_spread_prevents_later_value -- --nocapture
```

Also run the exact fixed-call disassembly/bytecode selector recorded in T0 if shared compiler code changed.

### Tests explicitly deferred

Patterns and metadata gate.

### Acceptance

- Record equality/hash are canonical and O(n);
- heterogeneous Record spread works;
- non-Symbol keys never become ordered/call labels;
- P3 Pack/fixed-call behavior is unchanged;
- helper ownership clearly separates ordered and unordered products.

### Local STOP / CONSULT triggers

- enforcing Symbol-only calls would require changing P3 argument representation;
- source Record equality cannot access canonical logical data without a new runtime API whose design changes public semantics materially;
- static call-spread uncertainty has no existing dynamic fallback policy.

### Checkpoint update

Record P3/P4 interoperability gate facts once focused tests pass.

---

## T7 — Record patterns and structural consumers

### Purpose

Generalize Record pattern keys to FieldKey, make pattern checking consume the shared unordered lookup authority, and lower exact resolved keys without source-name reconstruction.

### Preconditions

T3 unordered lookup established; T5 runtime lookup established.

### Consumes

- Record pattern AST from T1;
- shared `UnorderedFieldLookup`;
- existing pattern/match resolution;
- runtime Record key lookup/rest reconstruction;
- current pattern compiler/lowering.

### Produces

- resolved Record pattern fields carrying FieldKey;
- static key reduction for pattern keys via P2 checked authority;
- exact heterogeneous Record destructuring;
- rest exclusion by exact FieldKey;
- compiler lowering from resolved semantic key facts;
- P6-ready pattern products with no re-resolution requirement.

### Files and symbols

```text
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/match_semantics.rs
phalcom-semantic/src/checker/coverage/**
phalcom-core/src/compiler/lib/patterns.rs
phalcom-core/src/product/view.rs
phalcom-core/src/primitive/record.rs
```

### Required implementation shape

#### T7.0 Checked type/pattern key projection

This task is scheduled after T4, T5, and T6, before pattern lowering. It closes
the deferred T2.6 type-annotation seam and supplies the same checked projection
to Record patterns. T4's evaluator conversion for static Record values is a
separate producer and must not be reused as a second expression resolver.

The projection must:

- consume ordinary-checker binding selection and its stable source identity;
- query the current `StaticConstantProduct` by `StaticConstantId(SymbolId)`;
- convert concrete static value content to owned `FieldKey` values;
- make the existing publishing query own projection and record exact
  `StaticConstant` dependencies;
- defer genuinely symbolic results to P7 without creating a `FieldKey` or row
  placeholder;
- rerun canonical duplicate detection after concrete projection;
- share one checked projection between Record type and pattern keys, without
  adding evaluation/name lookup to `TypeResolver` or a pattern-only evaluator.

If existing checked products cannot provide stable identity and dependency edges
without a new parallel authority, stop and consult before changing ownership.

#### T7.0 principal-architect ruling and amended implementation contract — 2026-10-05

The principal architect approved T7.0 with material plan amendment. The ruling
resolves the owner, cold-start ordering, exact dependency, republication, and
mixed-cycle questions raised by the implementation stop. T7.0 is resumed under
the contract below. The checkpoint records the consultation and current plan
state; this subsection is the executable T7.0 amendment.

**Fixed architecture**

- The existing semantic query that publishes the type, signature, or resolved
  pattern owns computed-key projection and its dependencies. `TypeFormationSite`
  remains context only.
- Do not add `QueryKey::RecordKeyProjection` or an equivalent per-key query or
  product in P4. Owner-relative `RecordKeyUseId` is permitted for diagnostics,
  provenance, and local indexing; it is not a query key. Derive it from stable
  owner identity plus deterministic traversal ordinal/path, never a source range
  or ownerless `ExpressionId`.
- Introduce an early, type-free module static-binding catalog backed by linked
  namespace/import authority. It identifies stable `SymbolId`, binding phase,
  declaration/source identity, and syntax locations, but carries no `TypeId`,
  `BindingState`, or checked contract. Imported aliases retain their canonical
  remote `SymbolId`; ordinary checker binding selection remains authoritative.
- `StaticConstant(K)` owns formation of K's declared/expected type, then checks
  its initializer and publishes its `StaticConstantProduct`. The final
  `ModuleStaticBindingIndex` is a later checked projection/snapshot consumer,
  not a prerequisite for static-constant semantics.
- One shared ordinary-checker-based checked-static-expression/projector serves
  type keys and, in T7.1, pattern keys. Reuse P2 `StaticEvaluator` and its
  `StaticValueRef`; `StaticValueStore::field_key_of` converts the value to an
  owned concrete `FieldKey`. No evaluation or textual name resolution is added
  to `TypeResolver` or pattern lowering.
- The owner query records every exact selected
  `SemanticDependency::StaticConstant(StaticConstantId(symbol_id))`, alongside
  other dependencies from ordinary checking. A module-wide binding/interface
  dependency does not replace those exact edges.
- Projection yields `Concrete(FieldKey)`, `DeferredToP7`, or `Invalid`. A
  deferred symbolic key is never a `FieldKey`, row entry, or AST placeholder.
  Closed Record formation requires concrete keys; canonical row formation and
  duplicate detection occur only after projection, through the existing mutable
  `TypeStore`/row authority.
- A dependency cycle where closed static-constant formation needs a computed
  Record key that directly or transitively requires that constant is invalid.
  Diagnose it via active semantic-query evaluation/`Computing` state and
  transient key-projection cause. Do not define fixed points, source-order
  semantics, initializer-first speculation, symbolic placeholders, or LKG
  semantic inputs. Publish a diagnosed invalid/recursive semantic product, not
  an indefinitely blocked or internal-failure query.
- Recompute changed owner products and canonical rows through normal exact
  dependency invalidation. Never mutate an interned row or retain an obsolete
  `TypeId` as current semantic truth. Cold and incremental results agree.

**Mechanically flexible:** private helper names, catalog decomposition, the
owner-local representation of checked key facts, and equivalent query/session
plumbing may follow the live tree. Keep the established owner query and exact
identity chain intact.

**Verify first:** inspect live linked namespace/import products, all query
publication paths for type-bearing owners, the static-constant expected-type
path, and `Computing`/dependency revalidation behavior before choosing private
APIs. Adapt mechanical drift locally. If evidence requires a different owner,
identity, lifetime, canonicalization, or cross-layer contract, stop and escalate
under `shared-consultation-escalation-protocol.md`.

##### T7.0A Early catalog and self-sufficient static-constant queries

Add/publish a pre-type `ModuleStaticBindingCatalog` (or mechanically named
product) from parsed declarations and existing linked namespace/import
resolution. Preserve local and imported stable symbol identities and enough
source/declaration identity to recover the const declaration and initializer.
Keep the existing checked `ModuleStaticBindingIndex` as the later ordinary
module result.

Make `QueryKey::StaticConstant(K)` recover K through that catalog, form its
annotated expected type itself when present, then check/evaluate the initializer.
An unannotated constant continues to obtain its value type from ordinary
initializer checking. Checker contexts that read a constant must be able to use
the published constant's checked type without consulting final module-body
`BindingState`.

Acceptance for this slice: catalog publication is type-free; linked imports map
to the remote symbol; constant semantic publication no longer waits on final
`ModuleStaticBindings`; annotated constants may recursively demand constants
needed by their declared types; runtime `let` bindings remain ineligible.

##### T7.0B Shared checked static expression and key projector

Factor the reusable ordinary-checker result of a static expression out of the
constant-initializer-only shape. Preserve checked expression products, binding
selections, static operations/applications/constructions, associated
resolutions, local slots, result type, diagnostics, and complete semantic
dependencies. Use the same representation for const initializers and computed
key expressions.

Extend/factor P2 `StaticEvaluator` to evaluate this already-checked expression;
do not create another evaluator. The projector maps each selected `BindingId`
through the checker-selected binding to `SymbolId`, then to
`StaticConstantId`, forces/pulls the ready product, evaluates through P2, and
calls `field_key_of`. It performs no name lookup. A literal fast path is allowed
only behind this shared entry point. Return `Concrete`, `DeferredToP7`, or
`Invalid`; no symbolic key reaches row construction.

Acceptance for this slice: shared checking/evaluation works for static String,
Symbol, Int, and Bool keys and supported operations; invalid/unreduced values
fail through the existing `field_key_of` domain; ordinary runtime bindings are
rejected; dependency collection retains all selected constants and other
checker-produced dependencies.

##### T7.0C Existing type-owner integration

Find each existing query that publishes a type-bearing fact and make that
query own projection before canonical row formation. This includes declaration
shell/surface, trait header/surface where applicable, field signature,
callable signature, type-member declaration, static-case signature,
associated-static-domain declaration, and a constant's declared type. The
publishing query—not `TypeFormationSite`—supplies stable owner identity, captures
projection facts, merges dependencies, and forms the final type using the normal
mutable `TypeStore` row constructor.

Project all required keys before row formation. Run duplicate checking only
after reduction, using canonical row rules. Do not publish provisional rows,
patch rows in place, or move evaluation into `TypeResolver`.

Acceptance for this slice: a noncyclic const-key type forms cold; canonical
duplicates caused by reduction are rejected; a changed key recomputes and
republishes the owning type/signature through the existing owner query.

##### T7.0D Exact forcing, invalidation, republication, and mixed cycles

The semantic query/session orchestrator forces not-ready selected constants
during owner computation, tracks the active evaluation stack/`Computing` state,
and retains transient cause information for recursive entry through key
projection. Classify a cycle containing this edge and a closed `StaticConstant`
as an invalid static-formation cycle. Prefer a dedicated diagnostic such as
`StaticConstantFormationCycle`, or reuse the established cycle diagnostic with
a clear computed-key/type-formation note. Do not add a persistent second cycle
graph.

Keep exact consumer-to-constant dependency edges. Use normal reverse dependency
invalidation and fingerprint revalidation: unchanged constant products may let
consumers validate; changed products cause only exact consumers to reproject,
reform canonical rows, and republish dependent products. Old interned rows stay
immutable. Previous-snapshot/LKG values cannot satisfy a current computation.

Acceptance for this slice: direct and transitive mixed cycles diagnose
deterministically without hanging; an edit that introduces a cycle cannot keep
a previously successful type/signature as current truth; edits to K invalidate
consumers that selected K without substituting whole-module invalidation.

##### T7.0E Focused acceptance gate

Run the exact focused semantic gates added for T7.0 after A–D form a coherent
vertical slice. Confirm each selector actually runs tests. Cover:

- stable owner/site identity and exact local/imported `StaticConstant` edges;
- static String/Symbol/Int/Bool values and supported operations, invalid key
  values, and rejection of runtime `let`;
- duplicate detection after reduction, including `Symbol(#x)` versus
  `String("x")` remaining distinct;
- key edit `#a` to `#b`, same-result source edit, targeted invalidation,
  canonical row/type/signature republication, and cold/incremental parity;
- direct and transitive mixed cycles, deterministic diagnostic, no hang, no
  stale LKG semantic reuse, and no placeholder in a canonical row;
- symbolic deferral without `FieldKey`, `RecordRowData`, or AST placeholder.

T7.0E passes only when cold and incrementally updated sessions agree on key,
Record type, diagnostics, and downstream signature/body visibility, and all
mixed-cycle requirements above are demonstrated. **Do not begin T7.1 until this
gate passes.** T7.0 is not complete merely because the code compiles.

#### T7.1 Static pattern keys


Pattern forms:

```telos
{ name: x }
{ ["mime"]: x }
{ [42]: x }
{ [true]: x }
```

must resolve to concrete FieldKey before canonical pattern resolution. Runtime key expressions are not fixed structural pattern keys.

Consume the shared T7.0 checked static-key projection from the ordinary
`CallableBody` checker. Record exact `SemanticDependency::StaticConstant(id)`
edges on `QueryKey::CallableBody(callable)`, then require a concrete `FieldKey`
before producing `ResolvedRecordFieldPattern`. Preserve the `FieldKey` through
coverage, destructuring, and lowering, and use exact identity for rest exclusion.
Do not create a pattern-only evaluator or invalidate a callable signature solely
because a body pattern changed.

#### T7.2 Semantic pattern product

`ResolvedRecordFieldPattern` (or equivalent) carries FieldKey and source range/provenance separately.

Expected-field typing uses `UnorderedFieldLookup`:

- Required/Optional fixed fields preserve current match typing rules;
- typed tail may provide Optional(V) when membership proven;
- absent/unknown cases preserve existing diagnostic/refinement policy.

#### T7.3 Runtime destructuring

Compiler lowering carries exact FieldKey projection and calls canonical Record lookup. It must not emit a Symbol unconditionally.

#### T7.4 Rest pattern

For:

```telos
{ [42]: x, **rest }
```

exclude exactly `FieldKey::Int(42)` and preserve all remaining keys/presentation behavior according to ordinary Record semantics.

`#x` exclusion does not remove `"x"`.

#### T7.5 P6 seam

Publish the resolved Record pattern facts in the same semantic products P6's static matcher can consume. No static matcher implementation belongs here.

### Forbidden approaches

- pattern label stringification;
- `Value::symbol` for every Record pattern key;
- pattern-only key resolver/evaluator;
- key-kind coercion;
- Map pattern semantic merging merely because keys overlap;
- P6 static matcher implementation.

### Test changes required

Cover CV-16, CV-10B, and relevant CV-08:

- match each key kind;
- same-text Symbol/String distinction;
- Int/Bool distinction;
- missing fixed key;
- optional/open-tail typing;
- typed-tail lookup precision;
- invalid/non-static pattern key diagnostic;
- runtime destructuring;
- rest excludes exact heterogeneous key only;
- lowered pattern consumes resolved key even if source spelling/provenance is rearranged in test scaffolding.

### Tests to run now

Use the focused semantic/core pattern modules identified in T0, e.g.:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic record_pattern_field_keys -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core record_pattern_field_keys -- --nocapture
```

Then rerun the exact existing Record-pattern baseline selector from T0.

### Tests explicitly deferred

Metadata/incremental/resource certification.

### Acceptance

- resolved pattern keys are FieldKeys;
- semantic lookup is shared with row/checker logic;
- compiler uses resolved keys, not source strings;
- rest reconstruction excludes exact keys;
- P6 can consume the product later without re-resolution.

### Local STOP / CONSULT triggers

- existing pattern products cannot carry FieldKey without duplicating row identity or creating a second resolver;
- pattern rest reconstruction requires changing general Map pattern semantics;
- P2 checked static-key seam cannot be shared with pattern resolution.

### Checkpoint update

Record the resolved-pattern FieldKey seam after focused semantic/runtime pattern tests pass.

---

## T8 — Metadata staging, incrementality, compatibility, and performance hardening

### Purpose

Make P4 durable enough for downstream P5/P8 work without prematurely publishing P8's final schema: add lossless FieldKey metadata staging, deterministic fingerprints, targeted invalidation, runtime overlay typing, large-shape asymptotic evidence, exhaustive audits, and affected-crate compatibility checks.

### Preconditions

T1–T7 implementation slices complete and focused-tested.

### Consumes

- all P4 canonical facts;
- current type metadata codec/header;
- incremental query/fingerprint infrastructure;
- P2/P3 compatibility selectors;
- checkpoint baseline ledger.

### Produces

- `FieldKeyRef` and exact integer metadata representation;
- deterministic stable key fingerprint path;
- legacy Symbol-name migration semantics where decoder path exists;
- fail-closed active export for non-representable heterogeneous keys;
- runtime type overlay FieldKey support;
- incremental key/tail invalidation evidence;
- 1k/10k algorithmic resource evidence;
- exhaustive enum/visitor audit;
- affected crate checks.

### Files and symbols

```text
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/validate.rs
phalcom-type-meta/src/header.rs
phalcom-type-meta/tests/schema_compat.rs
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/export.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
phalcom-semantic/tests/semantic/incremental/**
phalcom-core/src/typing/overlay.rs
phalcom-core/src/typing/environment.rs
phalcom-core/src/typing/inspect.rs
all exhaustive matches over Record row/static/product shapes
```

### Required implementation shape

#### T8.1 FieldKeyRef

Define lossless stable tags for all four variants and exact integer sign/magnitude (or equivalent canonical BigInt form).

Stable encoding must not depend on Rust enum discriminant order by accident. Use explicit tags.

#### T8.2 Active schema boundary

Audit current metadata codec/versioning.

Preferred P4 behavior:

- in-memory/staged metadata nodes can carry `FieldKeyRef` losslessly;
- legacy Symbol-only wire decode maps name to `FieldKeyRef::Symbol`;
- legacy Symbol-only export remains valid if exact compatibility is preserved;
- heterogeneous row export through a wire format that cannot represent it returns a specific error/diagnostic;
- P4 does not bump/publish final C4 metadata schema merely to make tests pass.

If a compatible additive tagged representation already exists and introducing FieldKeyRef does not constitute a final schema release, use it. If not, keep the staged model and fail-closed boundary.

#### T8.3 Runtime overlay typing

Migrate `RuntimeRecordField` or equivalent from string name to heterogeneous stable key. Type inspection must preserve all key kinds.

#### T8.4 Fingerprints/incrementality

Any fingerprint previously writing `field.name` must write FieldKey stable encoding in canonical order.

Required discriminators:

1. `#x -> "x"` changes row/product fingerprint and invalidates dependents.
2. fixed field source reorder does not change canonical row identity/fingerprint.
3. value type change invalidates normally.
4. typed-tail key domain change invalidates row/dependents.
5. unrelated declaration/body change does not invalidate the row product.
6. P4 Record key change does not invalidate unrelated P3 Pack shape/static products.

#### T8.5 Exhaustive audit

Search all matches/usages of:

```text
RecordRowField.name
RecordFieldRef.name
RuntimeRecordField.name
StaticRecordField key/name
ProductShape::Record
ProductShape::Pack
RecordRowTail / AnonymousRecordRemainder
RecordPatternEntry label/name
RecordProductShape labels
```

Classify each remaining string/name as:

- legitimate non-Record identity;
- source/provenance-only display;
- compatibility wire field;
- missed P4 migration.

Do not perform blind global replacements.

#### T8.6 Performance/resource evidence

Required algorithmic evidence:

```text
row canonicalization       O(n log n)
duplicate scan             O(n)
row fixed-prefix merge     O(n + m)
row lookup                 O(log n)
runtime shape formation    O(n log n)
runtime lookup             O(log n)
Record equality/hash       O(n)
```

Run 1k and 10k mixed-key synthetic cases for:

- row formation;
- row solver disjoint/overlap merge;
- runtime shape formation;
- repeated lookup;
- equality of reordered large Records.

Do not impose a fragile wall-clock microbenchmark threshold. Acceptance is structural algorithm inspection plus a synthetic size gate that catches obvious accidental quadratic behavior/hangs.

If instrumentation already exists for operation counts, use it. Do not add public benchmarking infrastructure solely for P4.

#### T8.7 Compatibility

Rerun:

- P2 static-value/closed const focused selectors touched by exact-int/static-value changes;
- P3 static Pack/runtime Pack/spread/fixed-call selectors touched by shared files;
- existing Record row optionality/subtyping selectors;
- existing Record runtime behavior selectors unaffected by key-kind expansion.

#### T8.8 Documentation/spec sync

Update normative Record/structural product/VM specs only for implemented P4 behavior:

- FieldKey domain/order;
- Record fixed key/tail syntax;
- ordered-label boundary;
- runtime key categories/evaluation order;
- presentation vs structural order;
- metadata fail-closed staging if user-visible/tooling-visible.

Do not document P5 capabilities or P8 final schema as implemented.

### Forbidden approaches

- stringifying heterogeneous keys in metadata;
- schema version bump without explicit lifecycle/architecture authority;
- process-local IDs in fingerprints;
- source-order fingerprinting;
- performance “fixes” that add per-Record hash maps without evidence;
- blind global replacement of every `name` field;
- using broad failing suites as a substitute for focused P4 evidence.

### Test changes required

Cover CV-17, CV-18, CV-20, CV-22, CV-23:

Metadata:

- FieldKeyRef round-trip all variants;
- exact huge Int;
- Symbol/String distinction;
- deterministic stable fingerprint;
- legacy Symbol migration where applicable;
- non-Symbol active export fails closed when schema cannot carry it;
- no runtime/store IDs.

Incremental:

- key kind/content edit invalidation;
- source reorder stability;
- tail domain invalidation;
- unrelated edit stability;
- Pack product isolation.

Performance:

- 1k/10k cases listed above.

Compatibility:

- P2 and P3 focused selectors.

### Tests to run now

Run G4's focused components, then affected crate checks:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta field_key -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::metadata -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic record_row_field_key_incremental -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic record_field_key_scaling -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::record_field_keys::large_ -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast -p phalcom-type-meta -p phalcom-semantic -p phalcom-core
```

Exact selector names must follow the live modules created/identified in T0; do not broaden merely because an illustrative selector differs mechanically.

### Tests explicitly deferred

- full language corpus;
- full workspace tests/clippy;
- LSP/REPL broad suites;
- P8 final durable static metadata certification;
- P9 checkpoint-wide performance certification.

### Acceptance

- metadata staging is lossless and legacy export is fail-closed where necessary;
- all key-bearing fingerprints are stable/content-based;
- targeted invalidation behaves correctly;
- large-shape architecture shows no accidental quadratic canonical path;
- affected crates check;
- P2/P3 focused compatibility remains green or unrelated failures are classified;
- no missed canonical string-key Record identity remains.

### Local STOP / CONSULT triggers

- active metadata cannot stage FieldKey without lossy semantics or an incompatible final schema publication;
- 10k discriminators reveal quadratic behavior whose fix requires changing canonical ownership;
- P3 compatibility failure indicates ordered-label architecture was altered;
- incremental identity depends on process-local key IDs.

### Checkpoint update

Record G4 evidence, deferred schema boundary, performance evidence, and any classified failures.

---

## T9 — Stabilization, lifecycle closure, walkthrough, and handoff

### Purpose

Run the complete P4 focused acceptance gates, review the final scoped diff against invariants, classify all failures, update lifecycle truth, and deliver durable P4 completion records without overstating checkpoint or release certification.

### Preconditions

T1–T8 complete; no unresolved P4 A/B failures; no open consultation blocking acceptance.

### Consumes

- all P4 implementation slices;
- G1–G4 evidence;
- checkpoint baseline ledger;
- P3 current state at closure time.

### Produces

- final P4 verification classification;
- updated P4 plan lifecycle state;
- updated C4 checkpoint row/next action;
- `LANG007.C4.P4-walkthrough.md`;
- `LANG007.C4.P4-handoff.md`;
- explicit P5 unblock only if **both** P3 and P4 are semantically stable according to checkpoint rules.

### Files and symbols

```text
full P4 scoped diff
LANG007.C4-CHECKPOINT.md
LANG007.C4.P4-luna-patch-grade-implementation-plan.md
LANG007.C4.P4-walkthrough.md
LANG007.C4.P4-handoff.md
```

### Required implementation shape

1. Run G1–G4 serially.
2. Classify every failure A/B/C/D before editing.
3. Re-run a gate only after relevant changes.
4. Audit the final diff for every `P4-INV-*` invariant and every expected shared P3 seam.
5. Run `git diff --check` on the scoped final diff.
6. Record exact commands/results, including tests intentionally deferred.
7. Set P4 lifecycle state according to actual evidence:
   - source implemented but gate incomplete != COMPLETE;
   - focused gates passed != workspace/release certified;
   - metadata staging/fail-closed legacy path != P8 final schema release.
8. Inspect P3 state before setting the checkpoint next action:
   - if P3 is complete/stable too, P5 may become the next plan;
   - if P3 remains incomplete, record P4 complete but keep P5 blocked and retain P3 closure as prerequisite.
9. Do not rewrite P3 history/state to make the dependency graph look sequential.

### Forbidden approaches

- marking P4 complete because source compiles;
- treating designed tests as executed evidence;
- upgrading C4 checkpoint to complete;
- claiming final metadata durability/P8 completion;
- claiming P5 unblocked while P3 is not stable;
- running broad corpus merely for ceremonial closure;
- omitting known deferred failures from handoff.

### Test changes required

None unless a gate exposes a P4-owned missing acceptance case.

### Tests to run now

G1–G4 exactly as defined below, plus:

```sh
git diff --check
```

Run any final `cargo check` only once per affected-crate set after the implementation is stable.

### Tests explicitly deferred

Everything outside the P4 acceptance budget unless causal evidence requires it.

### Acceptance

T9 completes only when:

- G1–G4 have actual recorded outcomes;
- every required CV obligation is mapped to passing evidence or explicitly unresolved;
- no unresolved A/B P4 failures remain;
- final diff preserves all P4 invariants;
- checkpoint/plan status is truthful;
- walkthrough and handoff exist;
- P5 dependency state reflects actual P3 + P4 stability.

### Local STOP / CONSULT triggers

- a gate exposes a semantic contradiction rather than a mechanical defect;
- lifecycle state cannot represent “P4 complete while P3 still active” truthfully;
- final diff reveals a duplicate semantic authority introduced during implementation.

### Checkpoint update

Mandatory final update. Record exact P4 result, verification level, residual risks/deferred failures, metadata boundary, P3 dependency state, and next action.

---

# 19. Verification gates

## G1 — FieldKey + row semantic gate

### Purpose

Prove the semantic core independently of runtime Record implementation.

### Required coverage

```text
CV-01 scalar identity
CV-02 canonical ordering
CV-03 syntax
CV-04 row canonicalization
CV-05 tail formation
CV-06 solver/lacks
CV-07 relation/optionality
CV-08 unordered lookup
CV-21 query resources
```

G1 also verifies that unsupported const-reference, general-expression, and
symbolic/generic type keys fail closed and do not enter canonical rows. CV-10B
acceptance remains deferred until T7.0.

### Run

Use the exact live selectors established by T0. Expected shape:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast record_field_key -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::field_keys -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::record_rows_field_keys -- --nocapture
```

Also run the exact existing optional-row/subtyping regression selector recorded in T0.

### Expected

- zero P4-owned failures;
- canonical identity independent of source order;
- tail key-domain matrix correct;
- no runtime/core dependency needed for semantic correctness.

Do not broaden after PASS.

---

## G2 — Static/runtime Record vertical gate

### Purpose

Prove one FieldKey semantics from static value through runtime Record construction/access/equality/hash.

### Required coverage

```text
CV-09 static Record
CV-10A static Record value key bridge
CV-11 runtime conversion
CV-12 runtime shape
CV-13 runtime construction
CV-14 equality/hash
CV-19 evaluation order
```

### Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values_record_keys -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::record_field_keys -- --nocapture
```

If equality/hash live in a distinct established module, add that exact focused selector rather than a broad package run.

### Expected

- all key kinds work;
- EmptyRecord unchanged;
- presentation/logical separation proven;
- reordered mappings equal/hash equally;
- invalid/duplicate key timing proven.

Do not broaden after PASS.

---

## G3 — Pattern/call/P3 interoperability gate

### Purpose

Prove generalized unordered Record behavior does not leak into ordered labels and that patterns consume canonical keys.

### Required coverage

```text
CV-15 call/Pack boundary
CV-16 patterns
CV-10B checked type/pattern key projection
CV-22 P2 compatibility as shared checker/static seams require
CV-23 P3 compatibility
```

### Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic record_pattern_field_keys -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core record_pattern_field_keys -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::outgoing_packs_completion::p3_pack_spreading_preserves_lanes_and_fuses_builders -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::outgoing_packs_completion::p3_duplicate_from_earlier_pack_spread_prevents_later_value -- --nocapture
```

Also run the exact fixed-call/no-Pack P3 selector if compiler call lowering changed.

### Expected

- heterogeneous Record patterns pass;
- Record rest excludes exact FieldKeys;
- all-Symbol Record call spread passes;
- non-Symbol Record call/ordered spread rejects;
- Pack labels and fixed-call behavior remain unchanged.

Do not broaden after PASS.

---

## G4 — Metadata/incremental/resource/affected-crate gate

### Purpose

Prove lossless metadata staging, deterministic invalidation, scalable algorithms, and compilation of all directly affected crates.

### Required coverage

```text
CV-17 metadata
CV-18 incrementality
CV-20 performance/resource
CV-22 P2 compatibility
CV-23 P3 compatibility
CV-24 lifecycle evidence capture
```

### Run

Use exact module names created/identified by T0/T8. Expected command set:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta field_key -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::metadata -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic record_row_field_key_incremental -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic record_field_key_scaling -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::record_field_keys::large_ -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast -p phalcom-type-meta -p phalcom-semantic -p phalcom-core
```

Compatibility selectors to include if not already covered in G2/G3:

```text
P2 static_values_structural / static constants selector(s)
P3 static_values_pack
P3 runtime Pack collection/memory selector(s) affected by shared exhaustive matches
```

### Expected

- no lossy metadata path;
- targeted invalidation matrix passes;
- source reorder is fingerprint-stable;
- 1k/10k synthetic gates complete with canonical non-quadratic algorithms;
- affected crates compile;
- no P2/P3 regression attributable to P4.

Do not run the broad language corpus after PASS.

---

# 20. Final focused acceptance

At completion, map actual executed evidence to every coverage obligation.

| Coverage ID | Minimum accepting evidence | Required result |
|---|---|---|
| CV-01 | FieldKey scalar focused test | PASS |
| CV-02 | ordering/fingerprint focused test | PASS |
| CV-03 | AST/parser Record-key/tail/pattern tests | PASS |
| CV-04 | canonical row identity/duplicate tests | PASS |
| CV-05 | anonymous/captured tail tests | PASS |
| CV-06 | solver/lacks tests | PASS |
| CV-07 | subtyping/optionality matrix | PASS |
| CV-08 | shared unordered lookup tests | PASS |
| CV-09 | Static Record tests | PASS |
| CV-10A | static Record value key conversion/construction | PASS |
| CV-10B | checked type/pattern key resolution, invalidation, specialization | PASS |
| CV-11 | runtime conversion tests | PASS |
| CV-12 | runtime shape/lookup/presentation tests | PASS |
| CV-13 | construction/spread/duplicate tests | PASS |
| CV-14 | equality/hash tests | PASS |
| CV-15 | call/Pack boundary tests | PASS |
| CV-16 | Record pattern tests | PASS |
| CV-17 | FieldKeyRef + fail-closed metadata tests | PASS |
| CV-18 | incremental discriminators | PASS |
| CV-19 | side-effect/evaluation-order tests | PASS |
| CV-20 | 1k/10k resource discriminators + algorithm audit | PASS |
| CV-21 | budget/cancellation regression | PASS |
| CV-22 | P2 focused compatibility | PASS or unrelated classified |
| CV-23 | P3 focused compatibility | PASS or unrelated classified |
| CV-24 | checkpoint/walkthrough/handoff truth audit | PASS |

A missing required coverage row keeps P4 `PARTIAL / UNVERIFIED` regardless of source completeness.

---

# 21. Performance/resource evidence

P4 has explicit algorithmic goals and therefore requires evidence.

## 21.1 Required complexity contracts

```text
FieldKey comparison              payload-dependent, no hidden allocation
row canonicalization             O(n log n)
row duplicate detection          O(n) after sort
row fixed lookup                 O(log n)
row solver fixed-prefix merge    O(n + m)
runtime shape formation          O(n log n)
runtime fixed lookup             O(log n)
Record equality                  O(n)
Record hash                      O(n)
```

## 21.2 Memory contracts

- one content-owned canonical key per logical shape entry, not one per presentation + logical copy;
- no per-Record instance hash map solely for generalized keys;
- no heap-object rooting for structural key identity;
- no permanent String/LargeInt observation cache absent evidence;
- no Pack representation expansion caused by P4.

## 21.3 Required evidence

For 1k and 10k mixed-key synthetic shapes:

- canonical row formation;
- row solver disjoint and overlapping merge;
- runtime shape formation;
- repeated fixed lookup;
- equality of same mappings with different presentation order.

Acceptance does not require a microbenchmark threshold. It requires:

1. code-path inspection establishing the complexity shape;
2. synthetic scale tests that would expose obvious nested scans/hangs;
3. no unexplained severe regression relative to the same algorithm at 1k vs 10k.

If wall-clock timing is noisy, do not turn timing into semantics. Prefer operation-count instrumentation only if an existing internal seam supports it cheaply.

---

# 22. Checkpoint bookkeeping

## 22.1 At plan start

- record real starting revision;
- record worktree/isolation decision;
- represent P4 start according to lifecycle convention without erasing P3 state;
- keep P4 verification `UNVERIFIED`.

## 22.2 During plan

Update checkpoint only for:

- a durable new interface/invariant that later tasks inherit;
- a consultation/amendment;
- a coherent verification gate;
- a new deferred/baseline failure classification;
- a change in P3/P4 dependency state.

Do not record every mechanical edit.

## 22.3 At plan completion

Update:

- P4 plans ledger row;
- completion/verification classification;
- actual final revision;
- stable P4 interfaces;
- G1–G4 evidence;
- performance/resource result;
- metadata staging/fail-closed boundary;
- deferred failures;
- consultation decisions;
- P3 current state;
- whether P5 is now unblocked;
- next action.

If P3 remains incomplete, P4 may be complete while P5 remains blocked.

---

# 23. Walkthrough deliverable

Create:

```text
LANG007.C4.P4-walkthrough.md
```

It must contain:

- final P4 result and lifecycle classification;
- exact starting/final revisions;
- implemented FieldKey architecture;
- exact row/tail/solver/relation changes;
- static Record bridge;
- runtime Record shape/conversion/access changes;
- equality/hash/spread/call-boundary behavior;
- pattern migration;
- metadata staging boundary;
- incrementality/performance evidence;
- important implementation deviations from this plan;
- all consultation decisions;
- tests added;
- tests actually run with results;
- tests intentionally deferred;
- baseline/unrelated failures encountered;
- residual risks;
- P3 state and P5 dependency status.

The walkthrough records what actually shipped, not what this plan intended.

---

# 24. Handoff deliverable

Create:

```text
LANG007.C4.P4-handoff.md
```

It must contain:

- current repository revision/worktree state;
- P4 lifecycle state;
- stable FieldKey/row/lookup/runtime/metadata interfaces;
- invariants downstream plans must preserve;
- exact P5 prerequisites supplied by P4;
- P3 state and whether P5 is unblocked;
- metadata work deliberately deferred to P8;
- ontology/tooling work deliberately deferred to P9;
- deferred failures;
- must-read files;
- first recommended commands for next implementer;
- things not to redesign/re-explore, especially FieldKey order, Symbol-only ordered labels, row authority, and typed-tail semantics.

---

# 25. Completion truth table

Do not conflate these states:

| Evidence | Meaning | Does it complete P4? |
|---|---|---:|
| FieldKey source written | implementation slice exists | No |
| parser/row tests added | coverage designed | No |
| T1–T7 focused selectors pass | main behavior implemented | No |
| G1–G3 pass | semantic/runtime interoperability mostly proven | No |
| G4 pass + affected crates check | P4 focused certification complete | Potentially, after lifecycle review |
| walkthrough/handoff written | durable record complete | Required |
| full workspace green | release-level evidence | Not required by P4 |
| final metadata schema published | P8 concern | Not a P4 completion criterion |
| C4 checkpoint complete | P9/whole-C4 outcome | No |

P4 may be `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` (or the lifecycle vocabulary actually used at closure) while C4 remains `IN_PROGRESS / PARTIAL`.

---

# 26. Plan self-review

Before delivery/execution, verify:

- [x] one canonical FieldKey authority is defined;
- [x] all four variants and total order are fixed;
- [x] exact integer identity is shared, not duplicated;
- [x] Record syntax is separated from ordered labels;
- [x] anonymous tails have key domains;
- [x] captured tails retain correlation;
- [x] existing row solver remains authority;
- [x] key-domain lookup and relation reuse ordinary type semantics;
- [x] static key resolution reuses P2 checked static semantics;
- [x] static Record and runtime Record use the same key meaning;
- [x] runtime shape contains no heap identity;
- [x] presentation order and canonical structural order remain distinct;
- [x] lookup/equality/solver performance goals are explicit;
- [x] P3 ordered-label and fixed-call boundaries are explicit;
- [x] Record patterns consume resolved keys;
- [x] metadata is lossless or fails closed;
- [x] P8/P9 boundaries are preserved;
- [x] every major invariant has task ownership and test coverage;
- [x] debugging loops are bounded;
- [x] global/local consultation triggers are objective;
- [x] broad tests are intentionally excluded from BUILD;
- [x] lifecycle truth can preserve P3/P4 parallelism;
- [x] T0–T9 form a valid dependency order;
- [x] no placeholder architectural choice remains unfrozen.

---

# 27. Task dependency summary

```text
T0  takeover / coexistence audit
 │
 ▼
T1  FieldKey scalar + syntax
 │
 ▼
T2  canonical rows + tails
 │
 ▼
T3  solver / relation / lookup
 ├─────────────┐
 ▼             ▼
T4 static      T5 runtime shape/construction
 │             │
 └──────┬──────┘
        ▼
T6 equality/hash/spread + P3 boundary
        │
        ▼
T7 patterns
        │
        ▼
T8 metadata/incremental/performance/compat
        │
        ▼
T9 gates + lifecycle closure
```

T4 and T5 may proceed in parallel only after T3's FieldKey/row semantics are stable enough that both consume the same authority. T6 waits for runtime shape and static/semantic key behavior to converge. T7 should use the already-established shared lookup rather than preceding it.

---

# 28. Final execution directive

Implement P4 as a vertical migration of **one unordered-product key authority**, not as independent syntax, type-system, static-value, runtime, pattern, and metadata features.

The critical architectural sentence for the implementer is:

> A Record field key has one semantic identity (`FieldKey`), one canonical order, one row calculus, and multiple lossless projections. Ordered labels remain Symbol and are not generalized by P4.

When in doubt, preserve that ownership direction and consult rather than adding a compatibility shortcut that creates another source of truth.
