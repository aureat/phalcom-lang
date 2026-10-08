---
id: COLL006.C1
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: patch-grade-implementation-plan
title: "Extended Collections Foundation and First-Generation Collection Family"
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
prepared_against_repository: aureat/phalcom-lang
prepared_against_branch: main
prepared_against_revision: ad1db231f5d866650e7ab169c377f28ffa602de9
prepared: 2026-10-08
source_records:
  - PROGRAM.md
  - STATUS.md
  - COLL006.C1-CHECKPOINT.md
  - COLL006.C1-master-implementation-record.md
---

# COLL006.C1 — Detailed Patch-Grade Implementation Plan

## 0. Purpose and authority

Current execution: P1/P2 are complete; P3 is partial and paused at a user-requested delivery. This document retains the original intended architecture and acceptance obligations. The checkpoint and P3 walkthrough/handoff own current state and executed evidence.

This plan converts the accepted `COLL006.C1` lifecycle architecture into an executable implementation sequence grounded in the live Telos/Phalcom repository at:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   ad1db231f5d866650e7ab169c377f28ffa602de9
```

The plan is subordinate to the accepted semantics in:

```text
PROGRAM.md
STATUS.md
COLL006.C1-CHECKPOINT.md
COLL006.C1-master-implementation-record.md
```

Those records remain the authority for the C1 acceptance objective, public collection ontology, literal defaults, and lifecycle state. This plan does **not** claim implementation or verification.

The current repository has advanced materially beyond the lifecycle records' original `800474d8...` audit baseline. In particular:

- first-class `Pack` runtime/static machinery now exists;
- `StaticValueStore` includes `StaticPack` backing/slice nodes;
- `PackObject` provides a strong root/slice GC precedent;
- semantic-to-core lowering already transports checked iteration/expansion decisions through `ModuleLoweringSemantics`;
- C4 P14 remains proposed/not started;
- the bracket literal pipeline remains List-specific;
- structural static materialization remains deferred for Tuple/Record/Data/Variant/Pack.

The implementation must therefore preserve the accepted C1 semantics while using these newer repository mechanisms rather than reconstructing older planned machinery.

---

# 1. Executive implementation verdict

`COLL006.C1` is implementable as one checkpoint with nine numbered plans, but five architectural clarifications must be made explicit before production work proceeds:

1. **Vector dimension identity must use exact Telos static `Int`, not host-width `i64`/`usize`.** The live type store still represents `TypeData::ConstInt(i64)`, while C4's ordinary `StaticInt` is arbitrary precision. P3 must converge that representation at the shared static-domain boundary rather than invent a Vector-only dimension scalar.

2. **Universe/native generic metadata must represent heterogeneous static-domain binders.** `phalcom-native-meta::KindSpec` currently models only `Type` and arrow kinds. `Vector<T,N>` requires a canonical `Type × Int -> Type` declaration shape without a Vector-specific arity/reflection hack.

3. **Partial annotations are a checker-flow feature, not merely syntax.** `Statement::Let` resolves a full canonical annotation before the initializer is analyzed. `Array<_>` and `Vector<Int,_>` therefore require an inference-bearing annotation elaboration path whose variables remain solver-local until the initializer completes.

4. **Sequence literal selection must be a semantic product projected into lowering.** The compiler must not reinterpret bracket syntax, expected types, phase, or Vector cardinality. The current positional-expansion pipeline provides the architectural precedent.

5. **Persistent collection nodes must participate in Telos GC reachability and allocation pressure.** Hiding an unbounded native `Arc` graph behind one public heap object would evade the current heap-count pressure model and complicate precise tracing. C1 should prefer private GC-visible persistent nodes unless measurements force a separately approved node arena.

These are not changes to the accepted public model. They are implementation prerequisites needed to preserve that model correctly.

---

# 2. Fixed semantic contract

The following semantics are **FIXED** for this implementation unless the checkpoint is explicitly amended.

## 2.1 Core sequence ontology

```text
List<T>
    runtime extent
    mutable elements
    mutable extent
    dynamically resizable
    explicitly requested after the cutover

Array<T>
    runtime extent
    mutable elements for ordinary runtime instances
    immutable extent after construction
    owned dense Value-slot storage
    const-capable when explicitly selected

Vector<T,N>
    static extent N
    immutable elements
    immutable extent
    owned dense Value-slot storage
    runtime- and const-capable
```

Array length is **not** part of type identity.

Vector length **is** part of type identity.

## 2.2 Bracket literal selection

```text
explicit literal head
        ↓
unique expected collection family
        ↓
evaluation-phase default
```

Target behavior after P4 cutover:

```telos
let a = [1, 2, 3]
// Array<Int>

const b = [1, 2, 3]
// Vector<Int, 3>

let c: List<_> = [1, 2, 3]
// List<Int>

let d: Array<_> = [1, 2, 3]
// Array<Int>

let e: Vector<_, _> = [1, 2, 3]
// Vector<Int, 3>

let f = <_>[1, 2, 3]
// Array<Int>

let g = <_, _>[1, 2, 3]
// Vector<Int, 3>
```

Phase defaulting must **not** resolve an overload ambiguity between multiple viable collection families.

## 2.3 Inference holes

`_` means:

> Allocate a fresh query-local inference variable in the target parameter's canonical kind/static domain and solve it from ordinary constraints.

It does **not** mean:

```text
Dynamic
Any
existential wildcard
missing argument
ignored parameter
fallback element type
```

No `InferVarId` may survive into:

- `TypeStore`;
- binding contracts;
- callable/public signatures;
- `CallableAnalysis`;
- metadata;
- reflection products;
- compiler lowering products;
- runtime type environments.

## 2.4 Const Array

A const Array remains classifier `Array<T>`.

It does not become `ConstArray<T>`.

The compiler should reject direct mutation when statically evident, and materialized runtime Array objects must reject mutation through aliases.

## 2.5 Reused language authorities

C1 must reuse:

```text
TypeStore / canonical relation engine
InferenceSession / InferenceCoordinator
C4 StaticValueStore
C4 static evaluator
C4 materialization authority
Collection nominal root
Iterator / Iterable
existing Telos hash / == semantics
existing semantic-to-lowering projection architecture
```

No parallel subsystem is permitted.

---

# 3. Live repository audit matrix

## 3.1 Front-end and semantic literal pipeline

| Concern | Live owner | Current state | C1 action |
|---|---|---|---|
| Literal AST | `phalcom-ast/src/ast.rs` | `Expr::ListLiteral(Box<ListLiteralExpr>)` | Rename/generalize to neutral sequence/bracket literal form |
| Literal parsing | `phalcom-ast/src/parser.rs::parse_list_literal` | `[...]` parsed intrinsically as List literal | Preserve syntax, remove family ownership |
| Explicit generic arguments | `GenericArgumentSyntax` | `Single`, `Expansion`, `Remainder`, `Value`; no hole | Add dedicated inference-hole case |
| Binding annotation flow | `phalcom-semantic/src/checker/statement.rs::check_statement` | Full annotation resolved before initializer | Add inference-bearing annotation elaboration |
| Literal synthesis | `phalcom-semantic/src/checker/expression.rs::synthesize_list_literal` | Hardcodes canonical List | Replace with semantic sequence resolution |
| Expected types | `checker/expected.rs::ExpectedType` | Already has `Proper` and solver-local `Inference` | Reuse; generalize collection extraction |
| Inference | `checker/inference.rs` | Kinded query-local inference variables already exist | Reuse exactly |

## 3.2 Semantic publication and lowering

| Concern | Live owner | Current state | C1 action |
|---|---|---|---|
| Body analysis products | `checker/context.rs`, `checker/analysis.rs::CallableAnalysis` | Publishes `for_iteration_lanes`, `positional_expansions`, etc. | Add checked sequence-literal resolution index |
| Lowering projection | `phalcom-core/src/modules/semantic_lowering.rs::ModuleLoweringSemantics` | Carries checked iteration/expansion products | Add sequence-literal lowering specs |
| Compiler literal codegen | `phalcom-core/src/compiler/lib/expr.rs` | `Expr::ListLiteral` directly selects List opcodes | Consume only semantic lowering resolution |
| Fixed literal opcode | `Bytecode::BuildList` | Allocates List directly | P2/P3 add direct target-family construction |
| Spread literal opcodes | `BeginListLiteral` / `ListLiteralAppend` / `FinishListLiteral` | Mutable List is staging and final value | Replace target-specific staging for Array/Vector |

## 3.3 Type/static infrastructure

| Concern | Live owner | Current state | C1 action |
|---|---|---|---|
| Applied types | `TypeData::Applied` | Heterogeneous `StaticArgument`s supported | Use for Array/Vector; no Vector-specific TypeData |
| Static integer term | `TypeData::ConstInt(i64)` | Machine-width canonical type-level Int term | P3 prerequisite: converge to exact static Int |
| Exact compile-time Int | `crate::scalar::StaticInt` | `Small(i64)` / arbitrary precision large form | Reuse as canonical exact integer scalar |
| Static values | `StaticValueData` | Tuple/Record/Data/Variant/Pack, no homogeneous sequence | P4 add one shared static sequence representation |
| Static materialization | `materialization.rs` | Structural values deferred | P4 must extend canonical C4 boundary, never bypass it |
| Static Pack | `StaticPackBacking` / `StaticPackData` | Root/backing/slice support live | Use as design precedent, not as Array/Vector identity |

## 3.4 Universe/runtime metadata

| Concern | Live owner | Current state | C1 action |
|---|---|---|---|
| Builtin identities | `phalcom-native-meta/src/universe.rs::UniverseKey` | No Array/Vector/etc. | Add identities and source paths as each native family lands |
| Generic builtin metadata | `UNIVERSE_TYPE_FORMS` | Type-only generic binders | Extend/converge for static-domain Vector N |
| Metadata kind spec | `phalcom-native-meta/src/types.rs::KindSpec` | `Type` / `Arrow` only | Add canonical static-domain representation or replace legacy dependency |
| Runtime reflection arity | `phalcom-core/src/typing/inspect.rs::class_constructor_arity` | Reads `UNIVERSE_TYPE_FORMS` | Ensure Vector reflection uses canonical metadata |
| Runtime class slots | `phalcom-core/src/universe/core_classes.rs` | Existing Collection/List/Map/Set/etc. | Extend per native family |
| Builtin source loading | `phalcom-modules/src/builtin.rs` | Explicit module-path includes | Register new collection source modules |
| Public collections package | `core/universe/src/collections/package.ph` | Explicit `expose` list | Expose new source modules |

## 3.5 Runtime representation and GC

| Concern | Live owner | Current state | C1 action |
|---|---|---|---|
| Value size | value representation tests/docs | exactly 16 B | Must remain 16 B |
| Heap Object size | heap/perf architecture | approximately 40 B ceiling after boxing fat variants | New arms must not raise the common slot ceiling |
| List | `heap/list.rs` | `Vec<Value>` | Preserve as resizable family |
| Map/Set | `heap/map.rs` | ordered vector + hash index + reentry lock | Factor mechanism carefully in P6 |
| Pack slice | `heap/pack.rs` | root `ObjRef` + ranges | Use as Slice lifetime precedent |
| GC | `heap/trace.rs` | exhaustive Object match | Every new arm must declare exact edges |
| Mutable key guard | `primitive/mod.rs::is_mutable_collection_key` | List/Map/Set/Bytes | Extend for new mutable/content-unstable families |

---

# 4. Required C1 architectural amendments

These should be added to the checkpoint consultation/amendment ledger before the corresponding implementation plan is declared complete.

## COLL006.C1.A5 — Exact Vector dimension representation

### Trigger

`Vector<T,N>` requires source-level static `Int` identity, while the live type store represents the built-in type-level integer term as:

```rust
TypeData::ConstInt(i64)
```

and the compile-time value layer already has an exact arbitrary-precision `StaticInt`.

### Decision

Vector dimension `N` is a canonical exact Telos `Int` static argument.

No `i64`, `u64`, or `usize` boundary may define semantic Vector type identity.

Preferred correction:

```text
shared exact StaticInt owner
        ↓
canonical ConstInt term representation
        ↓
ordinary StaticArgument::Term(TypeId)
        ↓
Vector<T,N>
```

The exact Rust representation may use an interned exact-int ID, shared `StaticInt`, or another canonical exact form, but it must be reusable by all static-Int generic applications.

### Runtime boundary

Only runtime value construction converts exact `N` to `usize`, with a checked host-addressability failure if the requested value cannot exist in the current process.

### Forbidden shortcut

Do not create:

```text
VectorExtent(u64)
VectorExtent(usize)
Vector-only BigInt path
```

as semantic identity.

---

## COLL006.C1.A6 — Heterogeneous Universe generic metadata

### Trigger

`phalcom-native-meta::KindSpec` cannot currently represent an `Int` static-domain generic binder.

### Decision

The canonical runtime/native metadata path must describe:

```text
Vector : Type × Int -> Type
```

without:

```text
Vector has arity 2 special case
class-name switch
fake Type binder for N
per-specialization runtime class
```

P3 must extend or supersede the legacy `KindSpec`/`UNIVERSE_TYPE_FORMS` limitation through the existing canonical static-domain metadata model.

Reflection tests must prove both parameter domains.

---

## COLL006.C1.A7 — Persistent node GC ownership

### Trigger

Bag/Bijection/RangeMap require structurally shared native nodes containing Telos `Value`s.

### Decision

Persistent nodes must be visible to:

```text
GC reachability
GC mark deduplication
allocation-pressure accounting
```

Preferred C1 representation:

```text
public collection root Object
        ↓ ObjRef
private boxed GC-visible node Object
        ↓ ObjRef children / Value payloads
```

If measurements show this representation is unacceptable, STOP AND CONSULT before creating a private arena or native Arc graph.

### Forbidden shortcut

Do not hide an arbitrarily large `Arc<Node>` graph below one counted Telos heap object without explicit collector/accounting integration.

---

## COLL006.C1.A8 — Sequence literal semantic/lowering product

### Decision

Collection-family selection is published by semantic analysis and projected through the standard semantic-to-core lowering boundary.

Required flow:

```text
AST neutral literal
    ↓
semantic ExpressionId resolution
    ↓
CallableAnalysis.sequence_literals
    ↓
ModuleLoweringSemantics.sequence_literals
    ↓
compiler emission
```

The compiler must fail internally if a semantically valid neutral sequence literal reaches code generation without a checked lowering product.

The compiler must never:

```text
inspect phase to select family
re-run element-type inference
re-run Vector N solving
interpret head arity semantically
fall back to List
```

---

## COLL006.C1.A9 — Ordered-hash factoring boundary

### Decision

P6 should first factor:

```text
Telos hash callback
Telos == callback
numeric SameValueZero fast path
re-entry locking discipline
mutable-key admission
```

from Map/Set lookup semantics.

Physical storage may then be shared where appropriate.

Do not begin by forcing Map, Set, MutableBag, IndexedDict, and OrderedSet into one oversized all-purpose storage object.

Preserving Map/Set behavior is a blocking gate before new ordered-hash families can be called implemented.

---

## COLL006.C1.A10 — C4 materialization dependency

### Trigger

Live `MaterializationDescriptor` still classifies structural static values as deferred.

### Decision

P4 may extend C4's static/materialization machinery with collection-specific structural values, but must remain inside C4's canonical ownership model.

P4 must not:

```text
materialize static Array/Vector directly from compiler AST
re-execute const initializer source
invent a collection-only constant pool
serialize collection static values through a private metadata schema
```

If full structural materialization/durable metadata remains unavailable at P4 entry, either:

1. implement the bounded missing capability in the C4-owned subsystem as an approved dependency, or
2. classify P4 as blocked.

Do not fake completion with runtime reconstruction.

---

# 5. Target end-to-end architecture

## 5.1 Runtime literal

```text
source
    [1, 2, 3]

parser
    BracketLiteral(head=None, ...)

semantic
    expected family?
    explicit head?
    evaluation phase?
    element constraints?
    extent constraints?
        ↓
    SequenceLiteralResolution {
        family,
        result_type,
        element_type,
        extent,
        construction
    }

semantic publication
    ExpressionId -> SequenceLiteralResolution

core projection
    SourceRange -> SequenceLiteralLoweringSpec

compiler
    evaluate children once, left-to-right
    emit direct selected-family construction

VM
    allocate exactly selected family
```

## 5.2 Partial annotation

```text
Vector<Int, _>
        ↓
annotation elaboration
        ↓
Applied(
    Vector,
    [Canonical(Int), Var(N)]
)
        ↓
initializer [1,2,3]
        ↓
constraints:
    element = Int
    N = 3
        ↓
solve
        ↓
materialize canonical TypeId
        ↓
binding contract = Vector<Int,3>
```

## 5.3 Static literal

```text
const x = [1,2,3]
        ↓
phase = Static
        ↓
family = Vector
        ↓
checked SequenceLiteralResolution
        ↓
StaticValueStore::Sequence
        ↓
C4 materialization descriptor
        ↓
module lowering
        ↓
immutable VectorObject
```

---

# 6. Cross-cutting invariants

Use these IDs in tests, walkthroughs, and checkpoint evidence.

## Literal / inference

- **C1-LIT-01** — AST bracket literal representation carries syntax, never concrete family semantics.
- **C1-LIT-02** — semantic analysis is the single family-selection authority.
- **C1-LIT-03** — explicit head outranks expected family; contradictory explicit/contextual family diagnoses.
- **C1-LIT-04** — unique contextual family outranks phase default.
- **C1-LIT-05** — phase default never chooses between viable overload candidates.
- **C1-INF-01** — every `_` is a fresh solver-local variable with the canonical destination classifier.
- **C1-INF-02** — no unsolved inference variable enters durable state.
- **C1-INF-03** — empty underconstrained literals remain underconstrained; no `Dynamic` fallback.
- **C1-INF-04** — binding annotations with holes are canonicalized only after initializer solving.

## Sequence representation

- **C1-SEQ-01** — Array length does not participate in type identity.
- **C1-SEQ-02** — Vector N participates in exact canonical type identity.
- **C1-SEQ-03** — Vector N is exact Telos Int, independent of host pointer width.
- **C1-SEQ-04** — direct Array/Vector literal construction never allocates an intermediate List.
- **C1-SEQ-05** — Vector has no runtime mutation primitive.
- **C1-SEQ-06** — ordinary Array has no structural resize primitive.
- **C1-SEQ-07** — const Array remains Array and rejects mutation through aliases.

## Runtime / GC

- **C1-REP-01** — `size_of::<Value>() == 16`.
- **C1-REP-02** — new Object arms do not raise the accepted common heap slot ceiling.
- **C1-GC-01** — every new native object has complete precise tracing.
- **C1-GC-02** — Slice roots its original backing object.
- **C1-GC-03** — persistent shared nodes survive through every reachable old/new root.
- **C1-GC-04** — persistent node allocation participates in pressure/accounting.
- **C1-REENT-01** — no Heap borrow survives a Telos hash/==/ordering callback.
- **C1-REENT-02** — callback failure cannot publish a partially mutated collection.

## Hash/equality

- **C1-HASH-01** — every mutable/content-unstable collection remains inadmissible as Map/Set key.
- **C1-HASH-02** — immutable collection hash uses Telos value hash laws, never Rust object identity.
- **C1-HASH-03** — frozen Array does not become key-admissible merely because one instance is frozen.

## Persistence

- **C1-PERS-01** — update creates a new root and never mutates storage reachable from an older root.
- **C1-PERS-02** — Bijection forward/reverse roots remain exact inverses.
- **C1-PERS-03** — Bag stores only positive multiplicities.
- **C1-PERS-04** — RangeMap stores pairwise non-overlapping intervals.
- **C1-PERS-05** — balanced-tree invariants remain valid after every update/rotation.

---

# 7. Dependency graph and implementation order

```text
P1  Contextual Literal + Inference Foundation
│
├───────────────┬────────────────┐
▼               ▼                ▼
P2 Array        P3 Vector        P6 Ordered Hash
│               │                │
│               │                ▼
│               │               P7 Persistent Hash
│               │
└───────┬───────┘
        ▼
P4 Static Sequences + Default Cutover
        │
        ▼
P5 Slice + Buffer

P1 ─────────────────────────────► P8 Interval + RangeMap

P2–P8 ──────────────────────────► P9 Convergence / Certification
```

Additional prerequisite gates:

```text
P3
 ├─ G0A exact static Int representation
 ├─ G0B heterogeneous Universe metadata
 └─ G0C exact spread-cardinality lowering contract

P4
 └─ C4 structural materialization capability

P7/P8
 └─ persistent node GC ownership/accounting decision
```

---

# 8. COLL006.C1.P1 — Contextual sequence literal and inference foundation

## 8.1 Goal

Neutralize the List-specific literal architecture and establish checked contextual selection/inference infrastructure **without changing the repository-wide bare-bracket default yet**.

At P1 completion, old bare brackets still behave as List in compatibility mode until P4.

## 8.2 Primary write set

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/tests/parser.rs

phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/expected.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/db/fingerprint.rs
relevant semantic tests

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/expr.rs
compiler/lowering tests
```

Bytecode/VM should change only as mechanically needed to preserve List behavior under the neutral AST; no Array/Vector runtime implementation belongs to P1.

## 8.3 Task sequence

### P1-T0 — Baseline and census

1. Record exact branch/SHA/dirty state.
2. Run focused parser, semantic literal, compiler list-literal, positional expansion, and collection corpus baselines.
3. Enumerate all matches for:
   ```text
   ListLiteral
   synthesize_list_literal
   BuildList
   BeginListLiteral
   ListLiteralAppend
   FinishListLiteral
   ```
4. Enumerate source/docs/tests whose *semantic expectation* is bare brackets -> List.
5. Do not migrate those semantics yet; create the P4 migration inventory.

**Exit:** no production edit before the literal ownership map is complete.

### P1-T1 — Neutral AST rename/refactor

Replace the family-owned AST shape conceptually with:

```rust
Expr::BracketLiteral(Box<BracketLiteralExpr>)

struct BracketLiteralExpr {
    head: Option<SequenceLiteralHeadSyntax>,
    elements: Vec<BracketLiteralElement>,
    range: SourceRange,
}
```

Private names may differ, but the AST must no longer say that `[...]` is intrinsically List.

Preserve existing element and expansion source spans.

**Gate:** all existing bare-bracket parser fixtures remain behaviorally unchanged.

### P1-T2 — Explicit literal-head grammar

Add syntax for:

```text
<T>[...]
<T,N>[...]
```

The parser must only recognize a leading `<...>` as a sequence-literal head when the balanced closing `>` is followed by `[`.

Do not make parser semantics depend on argument count beyond retaining syntax.

Do not disturb:

```text
type lambdas
generic applications
comparison tokens
qualified associated syntax
```

### P1-T3 — Generic inference-hole AST

Add a dedicated syntax case, conceptually:

```rust
GenericArgumentSyntax::InferenceHole { range: SourceRange }
```

Parser behavior:

```text
_ in authorized generic argument position -> InferenceHole
_ in pattern position                     -> Pattern::Wildcard
```

Do not encode the hole as `Value(Var("_"))`.

### P1-T4 — Inference-bearing annotation elaboration

Introduce a checker-only elaboration result:

```rust
enum AnnotationElaboration {
    Canonical(TypeId),
    Inference {
        context: InferenceContextId,
        term: InferenceTerm,
        holes: Box<[InferenceHoleOrigin]>,
    },
}
```

or an equivalent representation.

Authorized initial contexts:

```text
let / let mut / const annotation with initializer
sequence literal head
explicit constructor generics when existing call inference can solve them
```

Strict durable declaration signatures remain hole-free unless separately designed.

For each hole:

1. read destination binder metadata;
2. allocate a fresh solver variable with the exact `KindId`/domain;
3. retain source range/origin for diagnostics;
4. preserve declaration constraints;
5. materialize only after solve.

### P1-T5 — Binding flow integration

Refactor `Statement::Let` from:

```text
resolve canonical annotation
check initializer
```

to:

```text
elaborate annotation
check initializer against Proper or Inference expected type
solve if needed
materialize canonical type
publish binding contract
```

Preserve existing causal-invalidity and relation/explanation handling.

Negative cases:

```text
let x: Array<_> = []
let x: Vector<_, _> = []
```

remain underconstrained unless enough context exists.

### P1-T6 — Explicit evaluation phase

Add a scoped checker fact:

```rust
enum EvaluationPhase {
    Runtime,
    Static,
}
```

The exact storage may be a context field, expression-analysis input, or scoped guard, but it must propagate through nested static expression analysis.

At P1, phase exists but compatibility default remains List when no implemented explicit/contextual family is selected.

### P1-T7 — Sequence literal resolution product

Add:

```rust
enum SequenceLiteralFamily {
    List,
    Array,
    Vector,
}

struct SequenceLiteralResolution {
    family: SequenceLiteralFamily,
    result_type: TypeId,
    element_type: TypeId,
    extent: Option<TypeId>,
    construction: SequenceConstructionPlan,
}
```

and:

```rust
type SequenceLiteralResolutionIndex =
    BTreeMap<ExpressionId, SequenceLiteralResolution>;
```

Publish through `CheckingContext` and `CallableAnalysis`.

At P1:

- contextual List is executable;
- explicit/contextual future Array/Vector may be semantically recognized only if their declarations are available in the chosen staging approach, otherwise keep them behind tests/diagnostics until P2/P3;
- bare compatibility default stays List.

### P1-T8 — Lowering projection

Add:

```text
CallableAnalysis.sequence_literals
        ↓
ModuleLoweringSemantics.sequence_literals
```

Compiler behavior:

- retrieve exact checked resolution by lowering site;
- List resolution uses current List bytecode;
- missing checked resolution is an internal error;
- compiler performs no family inference.

Include sequence-literal resolution in semantic fingerprints if it can change executable output.

### P1-T9 — P1 closure

Required proof:

- no production `Expr::ListLiteral` remains;
- no compiler family selection from literal syntax remains;
- `_` is not Dynamic;
- unsolved holes are not publishable;
- existing bare List corpus remains green;
- semantic lowering carries a checked literal product;
- P4 migration inventory is recorded.

## 8.4 P1 acceptance tests

Positive:

```telos
let xs: List<Int> = [1,2,3]
let xs: List<_> = [1,2,3]
```

Hole behavior via test-only/future collection forms:

```text
hole gets Type kind for element binder
hole gets static-domain kind for static binder
two holes are distinct variables
shared constraints can unify holes
```

Negative:

```text
empty inferred element with no evidence
hole in forbidden durable signature
ambiguous collection family context
head/context family conflict
```

---

# 9. COLL006.C1.P2 — `Array<T>`

## 9.1 Goal

Implement the runtime fixed-after-construction mutable homogeneous sequence and direct contextual/explicit Array literal lowering.

## 9.2 Recommended representation

```rust
pub struct ArrayObject {
    elements: Box<[Value]>,
    frozen: bool,
}
```

The `frozen` state may be introduced in P2 even if only P4 creates frozen instances.

Array representation exposes:

```text
len
get
set with frozen guard
```

It exposes no structural resize operation.

## 9.3 Primary write set

Expected files include:

```text
phalcom-native-meta/src/universe.rs
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/value/mod.rs
phalcom-core/src/value/render.rs

phalcom-core/src/heap/array.rs                    # new
phalcom-core/src/primitive/array.rs               # new
phalcom-core/core/universe/src/collections/array.ph # new
phalcom-modules/src/builtin.rs
phalcom-core/core/universe/src/collections/package.ph

semantic core IDs / source declaration binding as required
compiler/bytecode/vm literal construction
collection/runtime tests
```

Exact registration files must be re-censused immediately before edits.

## 9.4 Tasks

### P2-T0 — Universe identity

Add canonical Array declaration identity and `Array < Collection`.

Publish:

```text
Array<T> : Type -> Type
```

with invariant T.

Update runtime class resolution/reflection and source module loading.

### P2-T1 — Heap representation

Add `Object::Array(ArrayObject)` only if the arm keeps `Object` within the current size ceiling.

Otherwise box the payload.

Add:

```text
Heap::alloc_array
Heap::array
Heap::array_mut
Value::class mapping
debug/raw rendering
GC trace over every element
```

### P2-T2 — Primitive floor

Minimal native floor:

```text
new / construction helper as required
rawLength
rawAt
rawSet
```

`rawSet`:

1. validates receiver;
2. normalizes/checks index;
3. rejects frozen Array;
4. replaces one existing slot;
5. never changes length.

No append/insert/remove/clear/replace-slice primitive.

### P2-T3 — Source API

`array.ph` should implement ordinary source-level behavior over the minimal primitives:

```text
size
first
last
get
[]
[]= / set
iterator
equality
toList
toSet
```

Use the existing canonical `Iterator`/`Iterable` traits.

### P2-T4 — Direct literal lowering

Add direct Array construction.

For fixed literals:

```text
evaluate n children
BuildArray(n)
```

must perform:

```text
one final dense element allocation
one Array heap object allocation
zero List allocations
```

For spread literals, use a compiler/VM-private dense builder backed by `Vec<Value>` or an equivalent staging buffer.

Do **not** allocate a List as builder/final staging.

### P2-T5 — Semantic Array resolution

Activate:

```text
<T>[...]
expected Array<T>
```

through the P1 semantic product.

Array element checking must consume contextual T directly.

Array result type is independent of final length.

### P2-T6 — Mutable-key policy

Add Array to mutable-key rejection.

A frozen Array remains inadmissible because classifier semantics are mutable and its structural equality can diverge from identity hash assumptions.

### P2-T7 — GC/resource/performance certification

Tests:

```text
empty/small/large Array
index read/write
negative/out-of-range behavior
no resize selector/primitive
GC retains element object handles
frozen guard is wired
Array length not in type identity
direct literal has no List intermediate
Object size ceiling
Value size 16 B
```

---

# 10. COLL006.C1.P3 — `Vector<T,N>`

## 10.1 Goal

Implement immutable fixed-dimensional Vector with exact static Int dimension identity and direct literal/cardinality inference.

## 10.2 Blocking entry gates

### P3-G0A — exact static Int

Before Vector type publication:

- eliminate the `ConstInt(i64)` semantic-width limitation for public static `Int` generic identity;
- reuse shared exact `StaticInt` semantics;
- prove metadata/fingerprints/relations remain canonical;
- preserve small-int fast representation if desired.

### P3-G0B — heterogeneous builtin generic metadata

Before Vector runtime reflection:

- represent Vector's second binder as static Int/domain metadata;
- prove `kind`, `remainingParameterCount`, application validation, metadata round-trip, and reflection do not treat N as `Type`;
- no Vector-specific arity hacks.

### P3-G0C — exact spread cardinality contract

Before allowing Vector spread:

- establish a semantic cardinality proof that corresponds to the **same runtime expansion lane**;
- if Vector expansion uses canonical iterator evidence, prove its canonical iterator yields exactly N dense elements;
- otherwise introduce a checked intrinsic dense-vector expansion lane.

Do not statically add N while runtime uses an unrelated user-overridable iterator with unconstrained yield count.

## 10.3 Recommended runtime representation

```rust
pub struct VectorObject {
    elements: Box<[Value]>,
}
```

Do not redundantly store N unless measurements require it; `Box<[Value]>::len()` is available.

No mutable element accessor exists.

## 10.4 Tasks

### P3-T1 — Canonical declaration

Publish:

```text
Vector<T,N>
T : Type
N : Int
N >= 0
```

using the ordinary heterogeneous `TypeData::Applied`/`StaticArgument` path.

No special Vector TypeData.

### P3-T2 — Runtime identity/heap

Add Vector universe class, heap arm, accessors, class mapping, GC trace, source module, package exposure.

Vector is immutable/value-semantic and may become hash-safe when element hash laws are satisfied.

### P3-T3 — Primitive/source API

Native floor:

```text
rawLength
rawAt
```

No `rawSet`.

Source behavior:

```text
size
first
last
get
[]
iterator
equality
hash
toArray
toList
```

Hash implementation must use Telos element hashes.

### P3-T4 — Cardinality inference

For:

```telos
<_, _>[a,b,c]
```

the checker solves:

```text
N = 3
```

For fixed elements plus exact spreads:

```text
N = fixed_count + Σ exact_spread_count
```

Use existing Pack/cardinality reasoning where mechanically possible.

Do not introduce a collection-specific linear equation solver if the general static inference system can express the equality.

### P3-T5 — Direct lowering

Fixed literal:

```text
evaluate children
BuildVector(n)
```

or generalized dense-build equivalent.

Runtime finalization asserts the number of produced elements equals semantically proven N.

No List staging.

### P3-T6 — Exact-N runtime boundary

If an explicit Vector type has an exact static N outside host-addressable allocation:

- the type may remain semantically well-formed;
- value construction fails cleanly at the resource/allocation boundary;
- type identity is not truncated.

### P3-T7 — Hash/equality/GC/perf tests

Required:

```text
Vector<Int,3> != type Vector<Int,4>
same type identity cross-module
N=0
N large
exact Int beyond i64 semantic formation
immutability
hash consistency
Map/Set key use for hashable contents
GC
direct literal no List
Object slot ceiling
```

---

# 11. COLL006.C1.P4 — Static sequences and atomic bracket-default cutover

## 11.1 Goal

Extend canonical static values/materialization for homogeneous sequences, support const Vector and const Array, migrate owned repository source, amend legacy literal decisions, and flip defaults atomically.

P4 begins only after P2 and P3 runtime families are stable.

## 11.2 Static representation

Add one reusable static sequence node, conceptually:

```rust
struct StaticSequenceData {
    result_type: TypeId,
    elements: Box<[StaticValueRef]>,
}
```

and:

```rust
StaticValueData::Sequence(StaticSequenceId)
```

`result_type` distinguishes:

```text
Array<Int> [1,2,3]
Vector<Int,3> [1,2,3]
```

while storage/interner machinery is shared.

Do not encode Array/Vector as Pack.

## 11.3 Tasks

### P4-T0 — C4 materialization gate

Verify current C4 materialization state.

If structural descriptors remain deferred, implement the bounded sequence materialization extension in the C4-owned path or classify P4 blocked.

### P4-T1 — Static evaluator integration

Static sequence evaluation consumes the checked `SequenceLiteralResolution`.

It evaluates element expressions once under the C4 static evaluator and interns a canonical sequence value.

No runtime constructor calls.

### P4-T2 — Materialization

Materialize:

```text
Static Vector -> immutable VectorObject
Static Array  -> ArrayObject { frozen = true }
```

No initializer AST execution at runtime.

### P4-T3 — Const mutation semantics

Semantic direct mutation:

```telos
const a: Array<Int> = ...
a[0] = 3
```

should diagnose.

Runtime alias:

```text
const Array -> Dynamic/Array alias -> rawSet
```

must fail through the frozen guard.

### P4-T4 — Migration census

Classify every repository-owned bare bracket use:

```text
needs List resizing
needs fixed runtime sequence
needs static immutable sequence
only iterates
test intentionally pins legacy List
```

Convert dynamic-List assumptions to explicit List context/constructor.

Do not infer List from downstream mutation.

### P4-T5 — Decision/spec amendment

Explicitly amend/supersede the accepted legacy decisions that define `[...]` as List construction.

No final cutover is accepted while current decision docs still contradict executable semantics.

### P4-T6 — Atomic default flip

Only after migration:

```text
runtime bare [...] -> Array
static bare [...]  -> Vector
```

Retain:

```text
expected List      -> List
expected Array     -> Array
expected Vector    -> Vector
```

### P4-T7 — Overload/ambiguity certification

Add hostile tests:

```text
f(Array<Int>)
f(Vector<Int,3>)
f([1,2,3])              -> ambiguous, not runtime-Array tie-break

f(<_>[1,2,3])           -> Array overload
f(<_,_>[1,2,3])         -> Vector overload
```

### P4-T8 — Cross-module static durability

Prove:

```text
const Vector export/import
const Array export/import
metadata round-trip
cold/incremental parity
no initializer re-execution
```

---

# 12. COLL006.C1.P5 — `Slice<T>` and `Buffer<T>`

## 12.1 Goal

Implement zero-copy contiguous read-only views and bounded owned mutable construction storage.

## 12.2 Slice representation

Preferred:

```rust
pub struct SliceObject {
    owner: ObjRef,
    start: usize,
    len: usize,
}
```

No raw interior pointer.

All v1 owners are heap objects:

```text
Array
Vector
Buffer
```

### Nested slices

Normalize:

```text
slice(slice(root,a,b), c,d)
    ->
slice(root, a+c, d-c)
```

to the original root.

GC traces one owner edge.

## 12.3 Slice semantics

- O(1) indexed read;
- read-only in v1;
- Array element replacements are visible;
- Vector is stable;
- Buffer slices may only cover initialized prefix values;
- define/avoid Buffer shrink operations that would invalidate an existing slice.

## 12.4 Buffer representation

```rust
pub struct BufferObject {
    logical_capacity: usize,
    elements: Vec<Value>,
}
```

Invariant:

```text
0 <= elements.len() <= logical_capacity
```

`Vec::capacity()` is not the public capacity.

Prefer reserving requested logical capacity when practical.

No uninitialized Telos `Value` slots are exposed.

## 12.5 P14 convergence

Coordinate with `LANG007.C4.P14`.

Preferred parser snapshot model:

```text
private owned Array<T>
        +
read-only Slice<T> exposure
```

Do not introduce another immutable runtime-sized sequence solely for parser snapshots.

## 12.6 Tests

```text
slice creation is zero-copy
slice-of-slice root normalization
backing survives forced GC with only Slice live
Array mutation observed through Slice
Vector read stability
Buffer initialized-prefix rules
capacity boundary
push at capacity fails
no implicit growth
Object size ceiling
```

Add Array, Buffer, and Slice to mutable/content-unstable key rejection.

---

# 13. COLL006.C1.P6 — Ordered mutable hash family

## 13.1 Goal

Factor the existing Map/Set hash/equality/re-entry mechanism safely, then implement:

```text
MutableBag<T>
IndexedDict<K,V>
OrderedSet<T>
```

without regressing existing Map/Set.

## 13.2 No-regression firewall

Before refactoring, lock tests for:

```text
Map insertion order
Map equality
Set equality
Set encounter order
numeric SameValueZero key handling
duplicate-key behavior
hash collision behavior
mutable-key rejection
reentrant read
reentrant structural mutation rejection
hash callback failure
== callback failure
GC
```

These tests must remain green after each extraction step.

## 13.3 Factor semantic key engine first

Create one internal authority for:

```text
hash(Value)
candidate equality(Value,Value)
numeric fast path
reentrant send enter/exit
key admission
callback error propagation
```

No Heap borrow may cross a Telos callback.

Physical ordered storage can remain MapObject-compatible initially.

## 13.4 MutableBag

Logical model:

```text
key -> positive exact multiplicity
total count
```

Required invariants:

```text
no zero entries
no negative entries
total == sum(counts)
```

Multiplicity must preserve exact Telos Int semantics, not `usize`.

## 13.5 IndexedDict

Use:

```text
ordered entry vector
key -> slot hash index
```

Required complexity:

```text
key lookup            expected O(1)
positional read       O(1)
middle insert/remove  O(n) acceptable
move                  O(n) acceptable
```

Order participates in equality.

Never overload integer key lookup with positional indexing:

```text
dict[k]       -> key semantics
entryAt(i)    -> position semantics
```

## 13.6 OrderedSet

Use ordered unique storage plus hash index.

Required:

```text
membership            expected O(1)
at(i)                 O(1)
indexOf               expected O(1) or indexed lookup
positional mutation   explicit
order-sensitive ==    yes
```

Current `Set` remains order-insensitive.

## 13.7 Mutable-key guard extension

At P6 closure, the central mutable-key classifier must cover all mutable/content-unstable collections introduced so far.

---

# 14. COLL006.C1.P7 — Persistent hash family

## 14.1 Goal

Implement a reusable structurally shared immutable hash substrate supporting:

```text
Bag<T>
Bijection<K,V>
```

## 14.2 Representation decision first

Before lookup/update code, prove the persistent-node lifetime model.

Preferred node class:

```rust
enum PersistentHashNode {
    Leaf { hash, key, value_or_count },
    Collision { hash, entries },
    Bitmap { bitmap, children },
}
```

Use 32-way bitmap trie/HAMT or CHAMP-class layout.

Private nodes should be GC-visible and boxed when needed to protect `Object` size.

## 14.3 Update algorithm

Order operations so reentrant user code completes before unpublished path-copy nodes are created:

```text
compute Telos hash
resolve necessary Telos == callbacks
determine update path
perform path-copy allocation
publish new public root
```

Callback failure publishes nothing.

## 14.4 Bag

Persistent Bag is:

```text
PersistentHashMap<T, positive multiplicity>
```

Properties:

```text
old root unchanged
new root shares unaffected nodes
count arithmetic exact
hash/equality based on multiplicities
```

## 14.5 Bijection

Representation:

```text
forward_root: persistent map K -> V
reverse_root: persistent map V -> K
```

Insertion:

1. check key conflict;
2. check value conflict;
3. build new forward root;
4. build new reverse root;
5. publish Bijection object.

`inverse` should swap roots in O(1), aside from allocating the new public wrapper if required.

No silent displacement.

## 14.6 Structural-sharing / GC tests

Minimum root test:

```text
root0
root1 = root0 + A
root2 = root1 + B
drop root1
force GC

root0 still correct
root2 still correct
shared nodes alive
unreachable unique root1 nodes reclaimed
```

Resource/performance tests should detect accidental O(n) full-copy updates.

---

# 15. COLL006.C1.P8 — `Bound`, `Interval`, persistent ordered storage, `RangeMap`

## 15.1 Goal

Implement generic interval semantics independently of numeric progression Range, then add a persistent balanced ordered map over disjoint intervals.

## 15.2 Source values

Preferred:

```telos
enum Bound<T> {
  Unbounded
  Included(_ value: T)
  Excluded(_ value: T)
}

data Interval<T>(
  lower: Bound<T>,
  upper: Bound<T>,
)
```

These should be ordinary source-level data where possible.

## 15.3 Canonical interval kernel

Implement one authority for:

```text
compareLower
compareUpper
isEmpty
contains
overlaps
intersection
touchesWithoutGap
unionIfContiguous
```

Do not duplicate endpoint logic in RangeMap operations.

If current comparison facilities cannot support generic ordered K without a language-wide ordering redesign, STOP AND CONSULT. C1 explicitly does not activate a new `Comparable`/`TotalOrder` foundation.

## 15.4 Persistent ordered tree

Preferred C1 baseline: persistent AVL tree.

Node fields conceptually:

```text
interval
value
left
right
height
```

Properties:

```text
lookup            O(log n)
insert/update     O(log n) path copy
rotations         preserve old roots
GC                precise
```

## 15.5 RangeMap invariants

- stored intervals are nonempty;
- intervals are pairwise non-overlapping;
- point lookup finds the containing interval;
- overlay/split operations preserve disjointness;
- callback errors publish nothing;
- adjacent equal-valued intervals should coalesce where exact interval union has no gap, if implementable without semantic ambiguity.

## 15.6 Tests

Property-based/randomized oracle comparisons against a simple sorted interval list are strongly recommended.

Verify AVL:

```text
balance factor
height metadata
in-order ordering
persistence
lookup equivalence
rotation cases
```

---

# 16. COLL006.C1.P9 — Convergence, API, migration closure, certification

## 16.1 Goal

Close the family as one coherent standard-library surface and certify all cross-layer invariants.

P9 is not allowed to redesign core storage. If representation is still unsettled, the owning earlier plan is incomplete.

## 16.2 `Entry<K,V>`

Replace/retire the current nongeneric mutable `Entry` shape with an immutable generic value, preferably:

```telos
data Entry<K, V>(
  _ key: K,
  _ value: V,
)
```

Use it for explicit mapping constructors/conversions.

Do not change Map iterator's labeled tuple item shape until compatibility and migration are explicitly handled.

## 16.3 Constructor convergence

Prefer Telos constructor conventions:

```text
@constructor
call(...)
```

where custom source construction warrants it.

Internal native helper names may remain implementation-oriented.

## 16.4 Conversion graph

Certify explicit allocation/view boundaries:

```text
List<T>   <-> Array<T>     allocating
Array<T>   -> Slice<T>     zero-copy
Vector<T,N>-> Slice<T>     zero-copy
Buffer<T>  -> Slice<T>     zero-copy

Array/List/Buffer -> Vector<T,N>  checked conversion
```

Do not introduce runtime existential/dynamic-N Vector merely for conversion convenience.

## 16.5 Documentation and decisions

Update:

```text
collection taxonomy
literal semantics
List explicit-resizeable guidance
Array vs Vector guidance
const Array semantics
Slice/Buffer
persistent collection laws
Range vs Interval distinction
legacy TDR-0025/TDR-0028 amendment
P14 convergence
```

## 16.6 Final certification

Close:

```text
parser
semantic inference
metadata
compiler lowering
runtime
GC
re-entry
property laws
performance contracts
migration
cold/incremental parity
docs/decisions
lifecycle records
```

---

# 17. File/symbol ownership matrix

This is a starting map, not permission to edit blindly. Re-run repository search before each plan.

| Layer | Primary current owners | C1 use |
|---|---|---|
| AST | `phalcom-ast/src/ast.rs` | neutral literal, inference-hole syntax |
| Parser | `phalcom-ast/src/parser.rs` | literal heads, hole grammar |
| Binding checking | `checker/statement.rs` | partial annotation solve-before-publication |
| Expression checking | `checker/expression.rs` | sequence literal resolution |
| Expected types | `checker/expected.rs` | family/element inference projections |
| Inference | `checker/inference.rs` | query-local holes |
| Analysis product | `checker/analysis.rs`, `checker/context.rs` | `sequence_literals` index |
| Type store | `types/store.rs` | exact static Int correction; Applied reuse |
| Annotation formation | `types/annotation.rs` | inference-bearing elaboration boundary |
| Static values | `static_value/store.rs` | StaticSequence |
| Materialization | `materialization.rs` | Array/Vector descriptor support |
| Semantic lowering | `core/modules/semantic_lowering.rs` | checked literal projection |
| Compiler | `core/compiler/lib/expr.rs` | target-specific bytecode only |
| Bytecode | `core/src/bytecode.rs` | Array/Vector builders |
| VM dispatch | `core/src/vm/dispatch.rs` | final allocation |
| Heap enum | `core/src/heap/object.rs` | new native objects/private nodes |
| GC | `core/src/heap/trace.rs` | exact edges |
| Class mapping | `core/src/value/mod.rs` | runtime class identity |
| Rendering | `core/src/value/render.rs` | raw/debug forms |
| Universe IDs | `phalcom-native-meta/src/universe.rs` | canonical class identities |
| Native type metadata | `phalcom-native-meta/src/types.rs` | static-domain generic binder |
| Core class slots | `core/src/universe/core_classes.rs` | native classes |
| Runtime typing | `core/src/typing/inspect.rs` | applied reflection |
| Source module loading | `phalcom-modules/src/builtin.rs` | new collection `.ph` modules |
| Collection package | `core/universe/src/collections/package.ph` | public exposure |
| Iterator law | `collections/iterator_laws.ph` | reused unchanged where possible |
| List surface | `collections/list.ph` | migration/reference behavior |
| Map surface | `collections/map.ph` | P6/P9 preservation/migration |
| Set surface | `collections/set.ph` | P6 preservation |

---

# 18. Memory and representation gates

## 18.1 Non-negotiable representation tests

Every runtime-family plan must retain:

```rust
assert_eq!(std::mem::size_of::<Value>(), 16);
```

and a deterministic bound for `Object`.

Use the repository's current measured floor/ceiling as the acceptance baseline rather than silently allowing a new fattest variant.

If a new payload would enlarge every SlotMap slot, box it.

## 18.2 Dense sequence memory model

Approximate v1 payload:

```text
Array/Vector elements = 16 * n bytes + allocator overhead
```

Array/Vector should avoid List spare capacity after finalization.

Fixed literals should produce exactly one final dense payload allocation.

## 18.3 Builder policy

A compiler-private growable `Vec<Value>` is acceptable construction staging.

A source-visible `List` is not.

## 18.4 Future specialization seam

Keep storage private enough to allow later:

```rust
enum DenseStorage {
    Generic(Box<[Value]>),
    Float64(Box<[f64]>),
    // future typed layouts
}
```

Do not implement unboxed specialization in C1.

Do not make layout representation a public ABI promise.

---

# 19. Compiler and inference optimization policy

## 19.1 Avoid monomorphization by Vector N

All:

```text
Vector<Int,3>
Vector<Int,4>
Vector<Float,3>
```

should share ordinary runtime declaration class machinery.

Static type specialization may influence compiler optimization but must not require one runtime class/method body per N.

## 19.2 Known-size construction

Fixed Array/Vector literals:

```text
evaluate n children
reserve/build exact n
freeze into Box<[Value]>
```

No repeated growth.

## 19.3 Spread construction

Array:

```text
growable private builder
```

Vector:

```text
growable private builder only after static final cardinality proof
final runtime count assertion
```

## 19.4 Static constants

Static Array/Vector literals should materialize from compile-time products rather than emit element-construction bytecode for every runtime initialization.

---

# 20. Hash/equality policy

| Family | Equality | Mutable/content stable? | Key-admissible target |
|---|---|---:|---|
| List | ordered structural | no | no |
| Array | ordered structural | no | no |
| Vector | ordered structural | yes | conditional |
| Slice | current content | backing-dependent | no |
| Buffer | ordered current content | no | no |
| Map | mapping | no | no |
| Set | membership | no | no |
| MutableBag | multiplicity | no | no |
| IndexedDict | ordered entries | no | no |
| OrderedSet | ordered unique sequence | no | no |
| Bag | multiplicity | yes | conditional |
| Bijection | bidirectional mapping | yes | conditional |
| RangeMap | canonical/extensional mapping | yes | conditional |

“Conditional” means all semantically hashed child values satisfy Telos stable hash laws.

Never substitute Rust `Hash`/object identity for Telos value hashing.

---

# 21. Verification strategy

No plan is complete because code compiles.

Each plan records **exact commands actually run** and results in the checkpoint/walkthrough.

## 21.1 Baseline command families

Use the current repository's exact target names discovered by `-- --list` before final certification.

Typical focused lanes:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic -- --list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core -- --list

cargo check -p phalcom-ast
cargo check -p phalcom-semantic
cargo check -p phalcom-core
cargo fmt --all -- --check
git diff --check
```

Use focused selectors during development; broad gates belong at plan barriers and P9.

Do not repeatedly run the entire workspace after every edit.

## 21.2 Required semantic tests

Literal/inference matrix:

```text
bare runtime
bare static
explicit Array head
explicit Vector head
expected List
expected Array
expected Vector
nested expected type
call argument
return expression
data component
empty literal
heterogeneous/common supertype
underconstrained hole
two independent holes
hole kind/domain mismatch
overload family ambiguity
```

## 21.3 Runtime/GC tests

For every new object:

```text
construction
class identity
basic operations
nested object values
forced GC
stale handle safety where applicable
debug rendering
negative type receiver
out-of-range behavior
```

Persistent collections additionally need old/new-root GC stress.

## 21.4 Cold/incremental parity

Required when new semantic products or metadata influence durable compilation:

```text
cold compile result
incremental edit
recompute
same final semantic/lowering product
no global invalidation where dependency graph is specific
```

## 21.5 Property testing

Use simple oracle structures for:

```text
MutableBag
IndexedDict
OrderedSet
Bag
Bijection
RangeMap
```

Generate operation sequences and compare observable state.

---

# 22. Performance contract gates

Required targets:

| Operation | Target |
|---|---:|
| Array index read/write | O(1) |
| Array fixed literal build | O(n), one final dense payload |
| List append | amortized O(1) |
| Vector index | O(1) |
| Vector fixed literal build | O(n), one final dense payload |
| Slice creation | O(1) |
| Slice index | O(1) |
| Buffer index/set/push within capacity | O(1) |
| MutableBag lookup/update | expected O(1) |
| IndexedDict key lookup | expected O(1) |
| IndexedDict positional read | O(1) |
| OrderedSet membership | expected O(1) |
| Bag/Bijection lookup | persistent hash depth |
| Bag/Bijection update | path-copy hash depth |
| RangeMap point lookup | O(log n) |
| RangeMap overlap/overlay | O(log n + k) |

Performance certification should prove architectural complexity and allocation shape, not chase unstable microbenchmark noise.

Blocking regressions include:

```text
Array/Vector literal internally allocates List
persistent update copies every entry
RangeMap lookup linearly scans all intervals
IndexedDict key lookup linearly scans order vector
Slice copies backing elements
new Object variant enlarges every heap slot materially
```

---

# 23. STOP AND CONSULT conditions

Retain all checkpoint STOP triggers and add the following implementation-specific ones.

STOP if:

1. exact static Int convergence requires changing public Int semantics rather than internal canonical representation;
2. `Vector<T,N>` can only be reflected by special-casing its name/arity;
3. sequence family selection cannot be projected without the compiler re-proving semantic facts;
4. phase defaulting leaks into overload candidate ranking;
5. partial annotations require placing `InferVarId` in `TypeStore`;
6. Vector exact-spread cardinality cannot be made consistent with runtime expansion semantics;
7. const Array alias safety would require a new public classifier;
8. C4 materialization cannot represent static sequence values without bypassing its authority;
9. Array/Vector/Buffer/Slice arms increase `Object` slot size beyond the current accepted ceiling;
10. persistent node graphs cannot participate in GC pressure/tracing without a new memory subsystem;
11. Map/Set behavior changes during ordered-hash extraction;
12. any hash/equality/ordering callback path holds a live Heap borrow;
13. RangeMap requires inventing a language-wide ordering trait to proceed;
14. an asymptotic contract cannot be met without changing public semantics;
15. the same serious semantic defect survives one well-grounded corrective attempt;
16. tests can be made green only by weakening an invariant or changing accepted semantics.

---

# 24. Forbidden shortcuts

Do not:

1. parse `_` as Dynamic;
2. infer `Dynamic` for empty sequence literals;
3. make Array extent a type argument;
4. make Vector N hidden runtime metadata;
5. store Vector N as `usize` semantic identity;
6. create runtime classes per Vector specialization;
7. use List as Array/Vector literal staging/final conversion;
8. use phase as overload tie-breaker;
9. re-infer collection family in compiler codegen;
10. make frozen Array hash-safe;
11. expose raw interior pointers for Slice;
12. let Buffer expose uninitialized Values;
13. silently grow Buffer capacity beyond its semantic bound;
14. hide persistent Value-bearing graphs from GC accounting;
15. implement Bag persistence by cloning an entire Map per update;
16. implement Bijection as two mutable Maps with partially publishable updates;
17. generalize numeric Range into Interval/RangeMap ordering in this checkpoint;
18. introduce another Iterable/Iterator/Collection hierarchy;
19. introduce `SetList`;
20. silently change Map iteration from labeled tuples to Entry before P9 compatibility work;
21. claim a plan verified without recorded executed evidence.

---

# 25. Lifecycle requirements per numbered plan

At the beginning of every plan:

1. re-read `COLL006.C1-CHECKPOINT.md`;
2. record starting branch/SHA/dirty state;
3. re-census mechanically relevant live source;
4. record baseline tests;
5. verify predecessor gates.

At completion of every plan:

1. add an as-built walkthrough;
2. update checkpoint plan ledger;
3. record exact verification evidence;
4. record incidents/deviations;
5. classify residual failures;
6. create a minimal next-plan handoff;
7. do not promote checkpoint completion prematurely.

Any new consultation must be appended to the checkpoint amendment ledger.

---

# 26. C1 final acceptance matrix

C1 may be promoted to `COMPLETE` only when all of the following are proven.

## Language

- [ ] neutral bracket AST
- [ ] explicit `<T>[...]`
- [ ] explicit `<T,N>[...]`
- [ ] runtime bare default Array
- [ ] static bare default Vector
- [ ] expected List/Array/Vector contextual selection
- [ ] overload ambiguity preserved
- [ ] `_` is true inference
- [ ] partial annotations solve before publication
- [ ] empty underconstrained literals reject cleanly

## Types/static

- [ ] Array length absent from type identity
- [ ] Vector exact N present in type identity
- [ ] Vector N arbitrary-precision semantic path
- [ ] heterogeneous Vector metadata correct
- [ ] static sequence canonicalization
- [ ] const Array
- [ ] const Vector
- [ ] cross-module materialization
- [ ] cold/incremental parity

## Runtime

- [ ] Array
- [ ] Vector
- [ ] Slice
- [ ] Buffer
- [ ] MutableBag
- [ ] IndexedDict
- [ ] OrderedSet
- [ ] Bag
- [ ] Bijection
- [ ] Interval
- [ ] RangeMap
- [ ] generic immutable Entry

## Correctness

- [ ] no List intermediate for Array/Vector literal
- [ ] Vector immutable
- [ ] Array fixed extent
- [ ] const Array alias-safe
- [ ] Slice lifetime-safe
- [ ] Buffer initialized-prefix/capacity-safe
- [ ] Map/Set preserved
- [ ] no Heap borrow across callbacks
- [ ] persistent old roots valid
- [ ] Bijection inverse exact
- [ ] Bag counts exact
- [ ] RangeMap intervals disjoint
- [ ] mutable-key classifier complete

## Memory/performance

- [ ] Value remains 16 B
- [ ] Object slot ceiling retained
- [ ] dense literal allocation shape proved
- [ ] Slice O(1)
- [ ] ordered hash expected O(1) lookup
- [ ] persistent hash path-copy
- [ ] RangeMap logarithmic lookup
- [ ] GC pressure sees persistent nodes

## Migration/process

- [ ] repository bare-List assumptions migrated
- [ ] legacy literal decisions amended
- [ ] P14 snapshot ownership reconciled
- [ ] public docs teach current semantics only
- [ ] all P1–P9 ledgers complete or explicitly superseded
- [ ] residual failures classified
- [ ] checkpoint state truthful

---

# 27. Immediate execution recommendation

The next executable artifact remains:

```text
COLL006.C1.P1 — Contextual Sequence Literal and Inference Foundation
```

P1 should be implemented in this order:

```text
T0 baseline/census
T1 neutral AST
T2 explicit head grammar
T3 inference-hole syntax
T4 inference-bearing annotation elaboration
T5 binding solve-before-publication
T6 evaluation phase
T7 semantic sequence resolution product
T8 lowering projection
T9 closure
```

**Do not** change the repository-wide bare-bracket default in P1.

The semantic epoch switch remains P4, after Array, Vector, and static sequence materialization exist and have focused evidence.

---

# 28. Final implementation guidance

The core design principle for C1 is:

> **Collection syntax chooses no runtime representation by itself. Semantic analysis resolves a canonical collection value/type, compiler lowering projects that already-proven decision, and the runtime merely executes the selected representation.**

This same separation must hold for:

```text
literal family
element type
Vector N
const/static identity
hash/equality callbacks
persistent sharing
interval ordering
```

The project already contains the correct architectural precedents:

- query-local kinded inference variables;
- canonical heterogeneous applied types;
- semantic products projected into `ModuleLoweringSemantics`;
- first-class Pack root/slice representation;
- exhaustive GC tracing;
- boxed fat heap variants protecting arena density;
- Map/Set callback re-entry discipline;
- canonical Iterator/Iterable protocols.

C1 should extend those mechanisms rather than build collection-specific analogues.

If implementation preserves those ownership boundaries, the nine-plan checkpoint is large but coherent. If any plan begins duplicating inference, static identity, runtime typing, iteration, or GC authority, that is evidence to stop and correct architecture rather than continue locally.
