---
id: COLL006.C1.P4
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: implementation-plan
title: "Static Homogeneous Sequences and Atomic Bracket-Default Cutover"
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
depends_on:
  - COLL006.C1.P3
  - LANG007.C4.P3
follows: COLL006.C1.P3
supersedes: null
prepared: 2026-10-08
prepared_against_branch: feat/telos-collection-upgrade
prepared_against_revision: 298084a44968402fc47d826ee9f99df1c9205e6a
integration_main_observed: c2dd635ee34f2cef3e64e86c87a5e77992f29fa7
requirements:
  - COLL006.C1.P4-implementation-requirements-analysis.md
---

# COLL006.C1.P4 — Static Homogeneous Sequences and Atomic Bracket-Default Cutover

## Luna patch-grade implementation plan

---

# 0. Executor contract

This plan is an implementation contract for `COLL006.C1.P4`.

The implementer must:

- execute the tasks in the dependency order recorded here;
- use the live repository as mechanical truth while preserving the architecture and semantics fixed by this plan;
- consume the existing semantic/static/runtime authorities rather than inventing collection-specific duplicates;
- keep `COLL006.C1-CHECKPOINT.md` as the single durable checkpoint state record;
- use focused tests during BUILD mode and broaden only at named gates;
- preserve unrelated user/worktree changes;
- classify unrelated failures rather than repairing them opportunistically;
- stop and request architectural guidance when a STOP / CONSULT trigger fires;
- produce a P4 walkthrough and P4 handoff at completion;
- never promote P4 to complete merely because source compiles or the worktree is clean.

The implementer may adapt private helper names, local file decomposition, imports,
and mechanically equivalent APIs when the live tree has drifted.

The implementer may **not** change:

```text
collection-family semantics
static/runtime default semantics
semantic ownership of family selection
C4 ownership of static evaluation/materialization
Array/Vector public identity
Vector exact extent semantics
const Array classifier semantics
compiler fail-closed ownership boundary
migration acceptance requirements
```

without consultation.

Testing is evidence gathering, not ritual.

---

# 0.1 Planning reconciliation

This plan reconciles four bodies of evidence:

1. the canonical `COLL006.C1` checkpoint;
2. the current partial P3 handoff and publication supplement;
3. the existing coarse C1 master/detailed P4 direction;
4. the independent P3 audit and P4 implementation-requirements analysis.

The coarse C1 plan correctly established the major P4 shape:

```text
shared static homogeneous sequence representation
checked static evaluator integration
Vector materialization
frozen Array materialization
const mutation protection
repository literal migration
runtime/static default cutover
```

The deeper audit adds implementation requirements that are now **mandatory**:

```text
StaticValueData::Sequence must participate in:
    static_value_type
    semantic_eq
    semantic_hash
    content_fingerprint

sequence materialization must:
    use the existing StaticConstantId / RuntimeStaticConstantRef owner path
    root intermediate heap Values across GC
    never execute initializer AST at runtime
    publish no partial owner value on failure

const Array safety must:
    distinguish object mutation from binding rebinding
    reject direct proven const mutation semantically
    retain the runtime frozen guard as alias defense

literal cutover must:
    occur only after migration census + source edits + normative decision update
    make bare lowering fail closed when checked semantic family is unavailable
    not allow compiler phase guessing

P4 must not silently define const List semantics.
```

## 0.2 User-approved symbolic identity-spread boundary (2026-10-09)

Decision: keep static homogeneous sequences closed. A checked
`VectorIdentitySpread` whose canonical extent is symbolic remains
`StaticEvalOutcome::Symbolic`; the static evaluator does not specialize the
extent or publish a sequence root. Once the extent is closed as an exact static
integer, the identity-spread checks and sequence construction may reduce it.

This amends T3.5 and CV-09. It preserves C4 ownership of static evaluation and
materialization, the closed `StaticValueData::Sequence` invariant, and the
checker-owned identity proof. It does not authorize symbolic arithmetic,
type-parameter substitution in the evaluator, or a partial Sequence value.

These are not optional embellishments. They close correctness gaps in the earlier
high-level P4 outline.

## 0.3 Adviser amendment — empty static bracket element evidence (2026-10-09)

Before the T10 default cutover, `const empty = []` continues to select the
legacy `List` family. The literal still has no element-type evidence, so the
enclosing closed-constant check must emit the existing
`GenericInferenceUnderconstrained` diagnostic and publish no static value.
After T10, the same source selects `Vector<T, 0>`; its extent is known but `T`
remains underconstrained, so it must still diagnose and publish no value. A
typed control such as `const empty: Vector<Int, 0> = []` remains valid.

Do not introduce phase-aware family selection before T10 and do not diagnose
inside `sequence_literal::synthesize`: contextual/inference owners must first
get their normal chance to solve the element type. The existing underconstraint
diagnostic is emitted at the static initializer boundary only when an empty
bare bracket still has `Unknown(NoTypeEvidence)` after that opportunity. No
new diagnostic code or decision record is required. This amends T3 and CV-35;
the post-cutover case is verified at G6.

## 0.4 Adviser amendment — inherited failures at G2 (2026-10-09)

G2 may hand off to T4 as `BASELINE_BLOCKED` when all P4-active G2
discriminators pass and each remaining failure is positively matched to an
inherited baseline failure. Do not call the full filter passed. For the current
tree, the three computed-Record-key `RecordKeyNotStatic` failures in
`foundations::static_constants` match the pre-P4 15-pass/3-fail result and do
not execute the new empty-sequence boundary. Record their names and both result
counts in the checkpoint's baseline ledger. T4 may proceed with that explicit
classification; unrelated Record-key behavior remains outside P4.

---

# 1. Goal

Implement canonical static homogeneous Array/Vector values and their runtime
materialization, migrate owned source away from legacy bare-List assumptions,
then atomically change the semantic defaults so:

```telos
let x = [1, 2, 3]
// Array<Int>

const y = [1, 2, 3]
// Vector<Int, 3>
```

while explicit/contextual `List`, `Array`, and `Vector` behavior remains coherent,
checked, incremental, GC-safe, and compiler-authoritative through semantic
products.

---

# 2. Checkpoint acceptance objective

`COLL006.C1.P4` is one numbered plan inside `COLL006.C1`.

It does not close the full collection checkpoint.

It advances C1 by completing:

```text
C1 objective 4   runtime bare brackets -> Array
C1 objective 5   static bare brackets -> Vector
C1 objective 10  const Array evaluation + runtime immutability
C1 objective 11  Vector const semantics
C1 objective 12  canonical static homogeneous sequence values/materialization
C1 objective 18  migration from legacy bare-bracket List semantics
```

P4 must leave P5 with stable assumptions:

```text
List<T>        = explicit dynamically resizable sequence
Array<T>       = fixed-extent runtime sequence, mutable ordinary instances
Vector<T,N>    = immutable exact-static-extent sequence
const Array    = Array classifier + frozen materialization
const Vector   = Vector classifier + immutable materialization
bare runtime   = Array
bare static    = Vector
```

P4 is **not** allowed to begin the semantic cutover while P3 is still partial.

---

# 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: feat/telos-collection-upgrade
revision: 298084a44968402fc47d826ee9f99df1c9205e6a
main observed: c2dd635ee34f2cef3e64e86c87a5e77992f29fa7
merge base: 61c8734f752c8ffc252b6d4d4ba378c2bfe2b2b1
feature relative to main:
    5 commits ahead
    14 commits behind
```

The branch is therefore divergent at plan preparation.

P4 implementation must not treat this revision as a permanently frozen patch base.

Re-read the named live paths before editing.

Adapt mechanical drift locally.

Treat architectural drift as an escalation condition.

## 3.1 Verified current facts

At the planning anchor:

- `P1` is complete.
- `P2` is complete.
- `P3` is `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.
- bare brackets still default to `List`.
- explicit/contextual `Array` exists.
- explicit/contextual `Vector<T,N>` exists.
- `VectorIdentitySpread` exists as a checked semantic construction proof.
- runtime Vector storage is immutable.
- runtime Array storage already carries a frozen bit and rejects frozen writes.
- `CheckedStaticExpression` already retains `sequence_literals`.
- `CheckedStaticBody` already retains `sequence_literals`.
- the C4 static evaluator does not yet execute bracket literals as static homogeneous sequences.
- `StaticValueStore` has structural Tuple/Record/Data/Variant/Pack nodes but no homogeneous Sequence node.
- `MaterializationDescriptor` supports primitives and marks structural values deferred.
- `StaticConstantProduct::materialization_descriptor` already projects reduced static values through `MaterializationDescriptor::from_static_value`.
- module lowering already publishes `StaticConstantLoweringSpec`.
- imports/re-exports already use `RuntimeStaticConstantRef`.
- `VM::ensure_static_constant_materialized` already caches one owner-held runtime value.
- compiler same-module static-constant reads currently handle primitive descriptors directly and reject `DeferredStructural`.
- compiler bracket lowering currently has a compatibility fallback when no semantic lowering is attached:
  - explicit one-argument head -> Array;
  - bare headless literal -> List;
  - explicit two-argument head -> fail closed.
- that headless fallback must not become a runtime-Array phase guess during P4.

---

# 4. Required reads before implementation

Read in this order.

## 4.1 Workflow

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/commit-and-push-discipline.md
```

## 4.2 Checkpoint and predecessor

```text
docs/implementation/COLL006/PROGRAM.md
docs/implementation/COLL006/STATUS.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md

docs/implementation/COLL006/COLL006.C1/COLL006.C1.P3-implementation-plan.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P3-walkthrough.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P3-handoff.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P3-inference-publication-plan.md
```

## 4.3 P4 requirements

```text
COLL006.C1.P4-implementation-requirements-analysis.md

docs/implementation/COLL006/COLL006.C1/COLL006.C1-master-implementation-record.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1-detailed-implementation-plan.md
```

## 4.4 Static semantics / materialization

```text
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/materialization.rs
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/static_constant_demand.rs
phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/checker/static_body.rs
phalcom-semantic/src/db/fingerprint.rs
```

## 4.5 Sequence semantics

```text
phalcom-semantic/src/checker/sequence_literal.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/row_inference.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/application.rs
```

## 4.6 Runtime/lowering

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/static_constant.rs
phalcom-core/src/modules/materialize.rs
phalcom-core/src/compiler/lib/expr.rs

phalcom-core/src/heap/array.rs
phalcom-core/src/heap/vector.rs
phalcom-core/src/primitive/array.rs
phalcom-core/src/primitive/vector.rs
```

## 4.7 Tests

```text
phalcom-semantic/tests/semantic/README.md
phalcom-core/tests/README.md

current COLL006 semantic tests
current Array tests
current Vector tests
static_values_pack.rs
static_values_structural.rs
static_constants.rs
compiler/static_const.rs
execution/static_const.rs
module import/re-export static-constant tests
```

## 4.8 Normative literal/binding/static decisions

Re-read the current accepted source of truth for:

```text
TDR-0025
TDR-0028
TDR-0090
static evaluation spec
binding/field spec
type/static domain spec
generic constraint spec
collection/literal syntax specs
```

Do not trust historical paths if the decision index says they have moved or been
superseded.

---

# 5. Normative authority

## 5.1 Fixed C1 collection semantics

```text
List<T>
    homogeneous
    runtime extent
    dynamically resizable
    explicitly selected after cutover

Array<T>
    homogeneous
    runtime extent
    fixed after construction
    ordinary runtime elements mutable
    explicitly selectable as a const sequence

Vector<T,N>
    homogeneous
    exact static extent N
    immutable in all phases
```

Literal rules:

```text
<T>[...]        -> Array<T>
<T,N>[...]      -> Vector<T,N>

unique List context    -> List
unique Array context   -> Array
unique Vector context  -> Vector

otherwise:
    runtime -> Array
    static  -> Vector
```

Precedence:

```text
explicit family
    >
unique contextual family
    >
phase default
```

Phase default is never an overload tie-breaker.

## 5.2 Binding/static authority

```text
let      runtime immutable binding
let mut  runtime mutable/rebindable binding
const    static compile-time binding
```

Binding mutability and referenced-object mutability are separate.

A const Array is still classifier:

```text
Array<T>
```

not:

```text
ConstArray<T>
```

## 5.3 C4 authority

P4 consumes and extends:

```text
StaticValueStore
StaticValueRef
StaticEvaluator
StaticEvalOutcome
StaticConstantId
static query/dependency/cycle semantics
semantic static-value equality/hash
content fingerprints
MaterializationDescriptor
RuntimeStaticConstantRef
canonical owner runtime constant cache
QueryBudget
CancellationToken
```

P4 must not create alternatives to these authorities.

## 5.4 Non-normative historical collection documents

Older collection documents that describe:

```text
Array<T,N>
Vector<T>
```

are repository archaeology, not P4 semantics.

The current C1 checkpoint owns the collection taxonomy.

---

# 6. Takeover state

## 6.1 Stable interface map

| Concept | Current symbol/path | Owner | P4 invariant |
|---|---|---|---|
| Neutral bracket AST | `Expr::BracketLiteral`, `BracketLiteralExpr` | AST | parser never selects family |
| Family checker | `checker/sequence_literal.rs::synthesize` | semantic checker | sole family/cardinality authority |
| Family enum | `SequenceLiteralFamily` | semantic checker | `List/Array/Vector` only |
| Checked literal result | `SequenceLiteralResolution` | semantic checker | exact result/element/family/extent/construction |
| Exact spread lane | `SequenceExpansionResolution` | semantic checker | no runtime-sized Vector proof |
| Symbolic identity lane | `SequenceConstructionPlan::VectorIdentitySpread` | semantic checker | lone exact native Vector-storage spread |
| Checked static expression | `CheckedStaticExpression.sequence_literals` | C4/checker | static evaluator consumes, never re-solves |
| Checked static body | `CheckedStaticBody.sequence_literals` | C4/checker | same |
| Static value store | `StaticValueStore` | C4 | append-only canonical static identity |
| Static value enum | `StaticValueData` | C4 | P4 adds Sequence additively |
| Static value type projection | `StaticValueStore::static_value_type` | C4 | Sequence retains exact result type |
| Static equality/hash | `semantic_eq`, `semantic_hash` | C4 | Sequence integrated fully |
| Incremental value fingerprint | `content_fingerprint` | C4 | Sequence recursive content included |
| Descriptor | `MaterializationDescriptor` | C4 | self-contained; no VM/checker identity |
| Static constant product | `StaticConstantProduct` | C4 | descriptor derives from reduced static value |
| Lowering static constant | `StaticConstantLoweringSpec` | core lowering | consumes descriptor |
| Runtime owner ref | `RuntimeStaticConstantRef` | core module runtime | preserves canonical owner through aliases |
| Owner cache | `ModuleObject.static_constants` | VM | canonical materialized runtime value |
| Runtime materializer | `VM::ensure_static_constant_materialized` | VM | no partial owner publication |
| Array storage | `ArrayObject` | heap | has frozen guard |
| Vector storage | `VectorObject` | heap | immutable |
| Array raw write | `array_raw_set` / `ArrayObject::set` | VM/native | frozen alias defense |
| Compiler bracket lowering | `Expr::BracketLiteral` arm | compiler | checked semantic family required for nontrivial family semantics |
| Same-module const read | `Expr::Var` static-constant branch | compiler | currently primitive direct-materialization only |

## 6.2 P3 must close before P4

P4 inherits P3 only after the following are complete and recorded:

```text
canonical negative Vector extent formation
inference-owned sequence publication transaction
homogeneous Array public typing
Vector metadata/reflection/import-export verification
cold/incremental Vector extent/spread verification
runtime effect/failure edge cases required by P3
deep mutable-descendant hash/key policy
P3 lifecycle closure
```

These are predecessor obligations.

P4 must not absorb them silently.

---

# 7. Architecture

## 7.1 End-to-end authority flow

```text
source bracket syntax
        │
        ▼
ordinary semantic checker
        │
        ├─ family
        ├─ element type
        ├─ exact Vector extent
        ├─ construction plan
        └─ exact expansion proofs
        │
        ▼
SequenceLiteralResolution
        │
        ├──────── runtime lowering ───────────────► Array / Vector bytecode
        │
        └──────── static checker product
                         │
                         ▼
                  StaticEvaluator
                         │
                         ▼
              StaticValueData::Sequence
                         │
                         ├─ static_value_type
                         ├─ semantic_eq
                         ├─ semantic_hash
                         └─ content_fingerprint
                         │
                         ▼
              MaterializationDescriptor
                         │
                         ▼
            StaticConstantLoweringSpec
                         │
                         ▼
            canonical runtime owner
                         │
              RuntimeStaticConstantRef
                         │
                         ▼
        ModuleObject.static_constants
              │                    │
              ▼                    ▼
       VectorObject        frozen ArrayObject
```

## 7.2 No parallel family reasoning

The only layer that may decide:

```text
List
Array
Vector
N
exact spread eligibility
```

is semantic analysis.

Static evaluator, compiler, and VM consume exact checked products.

## 7.3 No runtime initializer execution

Runtime constant materialization consumes only self-contained descriptor data.

It must not consume:

```text
AST
ExpressionId
InferenceContextId
TypeSolver state
SequenceLiteralResolution directly
StaticValueRef
```

## 7.4 Static value vs runtime mutability

The compile-time `StaticValueData::Sequence` is always immutable canonical data.

Its runtime projection may be:

```text
Vector -> immutable VectorObject
Array  -> frozen ArrayObject
```

The static arena never models a mutable Array instance.

## 7.5 Exact compiler fallback rule after cutover

For an analyzed source program:

```text
compiler receives SequenceLiteralLoweringSpec
```

For a context-free/standalone compiler call with no semantic lowering:

- explicit one-argument `<T>[...]` may retain the existing syntactic Array compatibility path only if current compiler policy intentionally permits explicit-family syntax without analysis;
- explicit two-argument Vector continues to fail closed because exact semantic extent proof is required;
- **headless bare `[...]` must not default to Array or List in the compiler**.

After P4:

```text
headless bracket + missing semantic lowering
    -> MissingSequenceLiteralLowering
```

because the compiler does not own evaluation phase or contextual family selection.

If existing public APIs require standalone headless literal compilation, route those APIs
through semantic analysis. Do not encode phase inference in compiler syntax.

---

# 8. Ownership boundaries

## 8.1 P4 owns

```text
StaticValueData::Sequence
static sequence store/interning
static sequence type/equality/hash/fingerprint
static evaluator bracket execution
static Array/Vector descriptor projection
Vector runtime static materialization
frozen Array runtime static materialization
sequence-specific same-module/runtime const-read support
direct proven const Array mutation diagnostic
migration census
owned source migration
legacy literal decision supersession/amendment
runtime/static phase default cutover
P4-focused incremental/GC/durability evidence
P4 lifecycle records
```

## 8.2 P4 does not own

```text
remaining P3 inference publication work
remaining P3 Vector formation/typing work
general structural Tuple/Record/Data/Variant/Pack runtime materialization
general local-const architecture redesign
general deep immutability
general ownership/borrow checking
new Hashable trait architecture
const List semantics
mutable Slice
Buffer
persistent collections
ordered-hash refactor
general compile-time VM
general static constructor execution
general phase-aware compiler fallback
```

## 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| literal family | `SequenceLiteralResolution` | compiler, evaluator | compiler/static evaluator family selection |
| Vector N | canonical TypeStore + checked resolution | evaluator/compiler/runtime checks | source recount as semantic authority |
| static sequence identity | `StaticValueStore` | constant query/materialization | collection-specific static arena |
| static sequence type | `result_type` retained by Sequence | relation/materialization/tests | recompute family from phase |
| static constant owner | `StaticConstantId` | lowering/VM | compiler symbol-only owner |
| runtime static object | owner module cache | linked reads/exports | per-import copies as authority |
| const Array runtime mutability | frozen `ArrayObject` | all mutation paths | alias-copy immutability scheme |
| direct const mutation provenance | semantic binding/static identity | diagnostics | source spelling / compiler symbol set |
| phase default | semantic checking context | sequence synthesis | compiler/parser phase inference |
| migration classification | P4 census artifact | source edits/review | undocumented ad-hoc edits |

---

# 9. Global invariants

`INV-*` IDs are normative for this plan.

## Static semantic identity

**INV-01** — One canonical C4 `StaticValueStore` owns static sequence identity.

**INV-02** — A static sequence retains its exact checked ordinary `result_type`.

**INV-03** — `Array<T>[contents]` and `Vector<T,N>[contents]` are not the same static value identity merely because contents match.

**INV-04** — Static sequence child refs must belong to the same static-value-store lineage.

**INV-05** — A static Vector's exact semantic extent equals its logical element count.

**INV-06** — Host-size conversion never defines or truncates semantic Vector extent identity.

## Static laws / incrementality

**INV-07** — Sequence semantic equality is recursive, order-sensitive, and result-type-sensitive.

**INV-08** — Equal static sequences have equal semantic hashes.

**INV-09** — Static constant content fingerprints recursively include sequence result identity and ordered child content.

**INV-10** — Store-local allocation IDs do not affect semantic equality/hash/content fingerprints.

## Evaluator

**INV-11** — Static evaluator executes only checker-authorized sequence construction.

**INV-12** — Static evaluator never selects collection family.

**INV-13** — Static evaluator never invokes runtime Iterable protocol to prove Vector cardinality.

**INV-14** — Child evaluation is exactly once and source ordered.

**INV-15** — Cancelled/budgeted/invalid evaluation publishes no successful static constant value.

## Materialization

**INV-16** — Runtime materialization receives a self-contained descriptor only.

**INV-17** — Runtime materialization never executes initializer AST.

**INV-18** — Static Vector materializes as native immutable Vector storage.

**INV-19** — Static Array materializes as ordinary Array classifier with frozen mutation state.

**INV-20** — Failed materialization does not insert a partial value into the canonical owner cache.

**INV-21** — Intermediate heap children remain GC-rooted during recursive sequence materialization.

**INV-22** — Import/re-export identity uses existing canonical `RuntimeStaticConstantRef`.

## Const Array

**INV-23** — Direct provable const Array mutation receives a semantic diagnostic.

**INV-24** — Alias/Dynamic/reflection mutation attempts remain protected by the runtime frozen guard.

**INV-25** — Const Array does not become a new public type.

**INV-26** — Frozen Array instance status does not make the Array family generally hash-stable/hashable.

## Literal cutover

**INV-27** — Explicit family outranks contextual family; unique context outranks phase default.

**INV-28** — Phase default never resolves overload ambiguity.

**INV-29** — Runtime headless bracket default is Array only in semantic analysis.

**INV-30** — Static headless bracket default is Vector only in semantic analysis.

**INV-31** — Compiler does not infer phase-default family for a headless bracket.

**INV-32** — All owned source that semantically requires dynamic List behavior is explicit before default flip.

**INV-33** — Legacy normative List-literal rules are amended/superseded before final P4 acceptance.

## Scope

**INV-34** — P4 does not silently define `const List<T>`.

**INV-35** — P4 does not generalize arbitrary runtime constructors into static calls merely because bracket literals are statically evaluable.

---

# 10. Non-goals

Do not implement during P4:

- frozen List;
- immutable List;
- `ConstArray<T>`;
- `ConstVector<T,N>`;
- deep immutable type qualifiers;
- a new static collection trait;
- a second static evaluator;
- generic structural C4 materialization for Tuple/Record/Data/Variant/Pack;
- arbitrary compile-time invocation of Array/Vector source constructors;
- existential/dynamic-N Vector;
- dependent arithmetic beyond P3's approved exact lanes;
- a broad mutation-effect system;
- a general binding-phase refactor unless direct const mutation provenance cannot be represented otherwise;
- P5 Slice/Buffer work;
- P6+ collection work;
- release-wide performance certification.

---

# 11. Expected impact map

## 11.1 Expected semantic source

```text
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/static_value/mod.rs        # only if exports require it
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/materialization.rs
phalcom-semantic/src/checker/sequence_literal.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/context.rs          # only for const provenance if needed
phalcom-semantic/src/checker/binding.rs          # only for const provenance if needed
phalcom-semantic/src/checker/static_body.rs
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/db/fingerprint.rs           # only if Sequence isn't covered automatically through store fingerprint
```

## 11.2 Expected core source

```text
phalcom-core/src/modules/static_constant.rs
phalcom-core/src/modules/semantic_lowering.rs    # descriptor projection usually already generic
phalcom-core/src/modules/materialize.rs           # expected reuse; change only if required
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/heap/array.rs                    # frozen-construction helper
phalcom-core/src/heap/vector.rs                    # expected reuse
phalcom-core/src/heap/*                            # only GC/root helper if existing APIs require
```

## 11.3 Expected tests

```text
phalcom-semantic/tests/semantic/foundations/static_values_sequence.rs      # new
phalcom-semantic/tests/semantic/foundations/static_constants.rs            # extend
phalcom-semantic/tests/semantic/incremental/*                               # focused sequence incremental cases
current COLL006 semantic test module                                       # extend

phalcom-core/tests/core/compiler/static_const.rs                            # extend
phalcom-core/tests/core/execution/static_const.rs                           # extend
current Array runtime tests                                                 # extend
current Vector runtime tests                                                # extend
module import/re-export tests                                               # extend
GC/memory tests                                                             # extend if existing home is better
```

Exact module filenames may adapt to the live test tree.

## 11.4 Expected documentation/state

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md
docs/implementation/COLL006/PROGRAM.md
docs/implementation/COLL006/STATUS.md

docs/implementation/COLL006/COLL006.C1/
    COLL006.C1.P4-bracket-migration-census.md
    COLL006.C1.P4-walkthrough.md
    COLL006.C1.P4-handoff.md

accepted decision/spec files governing bracket literals
collection/literal docs/examples affected by semantic cutover
```

## 11.5 Unexpected-touch rule

Mechanical adjacent changes are permitted.

STOP AND CONSULT before entering:

```text
generic relation engine
module identity architecture
GC root architecture
compiler phase architecture
general static-definition call semantics
general binding product schema
metadata serialization schema
```

for a material architectural reason not already covered here.

---

# 12. Implementer decision authority

# 12.1 FIXED

The implementer must preserve:

- C1 family semantics;
- `Array<T>` runtime extent;
- `Vector<T,N>` static extent;
- Vector immutability;
- explicit const Array support;
- no `ConstArray`;
- C4 static-value authority;
- checked semantic family authority;
- existing static constant owner/reference path;
- runtime frozen Array alias defense;
- no phase-default overload tie-break;
- default cutover occurs after migration;
- headless compiler fallback does not become independent Array defaulting;
- no implicit const List.

# 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- `StaticSequenceId`/`StaticSequenceData` private names;
- storage vector/boxed slice choices inside the static arena;
- private descriptor helper decomposition;
- exact recursive materialization helper signature;
- private temporary root structure used during materialization;
- test module names;
- local diagnostic wording;
- whether sequence descriptor stores child descriptors inline or behind immutable shared slices;
- whether Array has `new_frozen`, `from_vec_frozen`, or equivalent private constructor;
- helper module for const-receiver provenance.

# 12.3 VERIFY-FIRST

| Assumption | Verify in | If false |
|---|---|---|
| P3 is COMPLETE/IMPLEMENTED at P4 start | checkpoint + P3 walkthrough/handoff | do not start P4 |
| feature branch is converged with intended integration target | git/GitHub history | converge first; rerun P3 gates |
| `CheckedStaticExpression.sequence_literals` still exists | `static_constant.rs` | mechanically adapt |
| `CheckedStaticBody.sequence_literals` still exists | `static_definition.rs` | mechanically adapt |
| evaluator has direct `EvaluatorBody` accessors | `static_eval/machine.rs` | mechanically adapt |
| sequence literal product contains exact expansion plan | checker analysis | STOP if lost |
| Array still has runtime frozen guard | heap/primitive Array | STOP if removed/redefined |
| module static constants still use owner cache | module runtime | STOP if ownership changed |
| descriptor remains self-contained | `materialization.rs` | preserve self-containment |
| local const runtime materialization remains generally limited | C4 current source/tests | do not invent P4-only local-const architecture |
| compiler same-module const branch still handles descriptors directly | compiler expr | adapt T6 |
| legacy decisions still encode bare List | decision index/files | amend/supersede before cutover |
| `const List` remains unspecified | C1 current records | STOP if user/spec has since ruled otherwise |
| standalone headless compiler fallback still exists | compiler expr | remove/tighten in T10 |
| static evaluator budget is charged per work item | machine/query budget | ensure sequence loops charge bounded work |
| VM recursive allocation needs explicit rooting for host-held Values | heap/GC architecture | use existing root seam; STOP if nonlocal architecture required |

---

# 13. Global STOP / CONSULT triggers

Stop editing the affected issue and prepare a consultation packet when:

1. P3 is not actually complete.
2. P3 sequence products can still outlive rejected inference owners.
3. `Vector<T,N>` cannot be validated through canonical ordinary static application.
4. static sequence support requires a second static-value store.
5. static evaluator would need to re-run type/family inference.
6. static evaluator would need runtime Iterable dispatch to prove Vector length.
7. runtime materialization would need AST/checker state.
8. const Array safety would require changing its public classifier.
9. const Array safety would require copying on arbitrary aliases.
10. direct const mutation provenance can only be added through a broad redesign of general binding identity.
11. supporting nested sequence constants requires general Tuple/Record/Data/Variant/Pack materialization as a prerequisite.
12. const List appears necessary to satisfy accepted P4 semantics.
13. phase default must influence overload candidate ranking to make tests pass.
14. compiler must independently know runtime/static phase to compile a headless bracket.
15. GC-safe recursive materialization cannot use an existing or bounded rooting mechanism.
16. module import/re-export support would require a second static-constant ownership system.
17. static sequence fingerprints cannot be deterministic without changing general static identity.
18. source migration requires changing public List/Array/Vector semantics rather than making use explicit.
19. accepted decisions/specs conflict with the checkpoint's ratified C1 semantics.
20. a semantic test oracle must be weakened to proceed.
21. the same nontrivial semantic failure survives one serious correction.
22. a materially unexpected subsystem becomes required.

A triggered consultation cannot be self-waived.

---

# 14. Debugging budget

## 14.1 Mechanical failures

Allow at most three coherent correction cycles.

Each cycle:

```text
inspect exact failure
identify concrete cause
make one coherent local correction
rerun smallest discriminator
```

Examples:

```text
missing enum arm
private method visibility
borrow checker cleanup
test module registration
renamed helper
```

## 14.2 Semantic failures

Before editing:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

One serious corrective attempt per underlying semantic issue.

If the same issue persists: STOP AND CONSULT.

## 14.3 Architectural failures

Zero speculative architecture fixes.

Consult immediately.

## 14.4 No unchanged reruns

Never rerun an unchanged failure unless code, fixture, environment, or hypothesis
relevant to that failure changed.

---

# 15. Testing surface analysis

| Coverage ID | Dimension | Invariants | Required case |
|---|---|---|---|
| CV-01 | static store canonical success | 01–06 | Array/Vector sequence interning |
| CV-02 | foreign child | 04 | child from another store rejected |
| CV-03 | Vector cardinality | 05–06 | 0, exact, mismatch, huge |
| CV-04 | static type projection | 02–03 | exact Array/Vector result type retained |
| CV-05 | equality/hash | 07–10 | equal/different family/order/content |
| CV-06 | fingerprint | 09–10 | child/family/length edits invalidate |
| CV-07 | evaluator fixed | 11–15 | fixed Array/Vector bracket |
| CV-08 | evaluator spread | 11–15 | closed Tuple / Vector exact spread |
| CV-09 | identity spread | 11–15 | symbolic Vector identity proof |
| CV-10 | evaluator failure | 15 | cancellation/budget/tampered proof |
| CV-11 | descriptor | 16 | self-contained recursive sequence descriptor |
| CV-12 | deferred nested structure | 16 | sequence child structurally deferred |
| CV-13 | Vector materialization | 17–18 | immutable native Vector |
| CV-14 | Array materialization | 17,19–20 | frozen ordinary Array |
| CV-15 | GC during materialization | 20–21 | nested heap-backed elements survive forced GC |
| CV-16 | owner reuse | 20,22 | repeated/imported/re-exported runtime reads |
| CV-17 | no runtime initializer | 17 | no source initializer bytecode execution |
| CV-18 | direct const mutation | 23 | `const Array` subscript write diagnosed |
| CV-19 | alias mutation | 24 | alias reaches runtime FrozenMutation |
| CV-20 | const classifier | 19,25 | frozen value still reports Array |
| CV-21 | hash policy | 26 | const Array not admitted merely due frozen state |
| CV-22 | runtime phase default | 27–31 | bare runtime -> Array |
| CV-23 | static phase default | 27–31 | bare const -> Vector |
| CV-24 | explicit/context precedence | 27 | explicit/context families unchanged |
| CV-25 | overload ambiguity | 28 | phase cannot pick Array/Vector overload |
| CV-26 | compiler fail-closed | 31 | headless bracket missing lowering errors |
| CV-27 | List migration | 32 | dynamic List sites remain List explicitly |
| CV-28 | static List boundary | 34 | no implicit const List model |
| CV-29 | constructor boundary | 35 | bracket support does not authorize arbitrary runtime constructor |
| CV-30 | incremental phase edit | 09,27–30 | `let` <-> `const` changes family correctly |
| CV-31 | incremental element edit | 09 | value change invalidates dependents |
| CV-32 | incremental extent edit | 05,09 | Vector N changes canonically |
| CV-33 | incremental spread edit | 11–15 | exact proof removal does not leave stale product |
| CV-34 | module durability | 22 | selective/renamed/qualified/re-export |
| CV-35 | empty literal | 27–30 | phase-owned family; T underconstraint preserved before and after cutover |
| CV-36 | runtime source order | 14 | default flip does not regress child effects |
| CV-37 | normative migration | 33 | accepted decision no longer says bare List |

---

# 16. Verification execution budget

# 16.1 BUILD MODE

Use narrow discriminators.

Do not run workspace-wide tests after each task.

Cargo commands run serially.

# 16.2 STABILIZE MODE

Run at named gates:

```text
static sequence store laws
static evaluator/static constant suite
core static const runtime
Array/Vector runtime
COLL006 semantic filter
module import/export
incremental sequence cases
```

# 16.3 CERTIFY MODE

P4 does **not** require automatic full release certification.

At P4 finalization, run:

```text
focused P4 semantic/runtime/incremental gates
native surface --check if native authored metadata changed
cargo check --workspace --all-targets
scoped formatting/diff hygiene
```

Only run workspace tests/clippy/release if:

```text
the user explicitly requests it
or
checkpoint policy is amended to require it
```

P9 owns broader C1 release/performance certification.

# 16.4 Verification ladder

```text
T0 exact regression/reproducer
T1 direct feature tests
T2 owning module suite
T3 adjacent cross-layer integration
T4 broad crate/package check
T5 release/workspace certification
```

Do not automatically climb after PASS.

---

# 17. Baseline / unrelated failure policy

Classify:

```text
A — definitely caused by P4
B — probably caused by P4
C — unclear
D — unrelated/baseline
```

A/B are active.

C gets one bounded classification pass.

D is recorded and left alone.

Inherited P3/C1 baseline records remain valid until reclassified by evidence.

Do not convert historical failures into P4 work without causal evidence.

Do not weaken assertions.

---

# 18. Exact implementation sequence

The implementation order is strict unless this plan explicitly says otherwise.

```text
PRECONDITION
    P3 COMPLETE + feature branch converged
        │
        ▼
T0  P4 takeover / baseline / decision-preflight
        │
        ▼
T1  Static sequence arena representation
        │
        ▼
T2  Static sequence semantic laws + fingerprints
        │
        ▼
G1  Static-value foundation gate
        │
        ▼
T3  Static evaluator bracket execution
        │
        ▼
G2  Static-evaluation gate
        │
        ▼
T4  Materialization descriptor
        │
        ▼
T5  VM recursive materialization + frozen Array
        │
        ▼
T6  Compiler/runtime static-constant read convergence
        │
        ▼
G3  Runtime materialization gate
        │
        ▼
T7  Direct const Array mutation provenance/diagnostic
        │
        ▼
G4  Const-safety gate
        │
        ▼
T8  Repository bracket census
        │
        ▼
T9  Explicit List migration + normative decision/spec amendment
        │
        ▼
G5  Pre-cutover migration gate
        │
        ▼
T10 Atomic semantic default cutover + compiler fallback tightening
        │
        ▼
G6  Literal semantics/ambiguity gate
        │
        ▼
T11 Incremental + import/re-export + GC hostile matrix
        │
        ▼
G7  P4 stabilization gate
        │
        ▼
T12 Lifecycle closure + P5 handoff
```

Do not move T10 earlier.

---

# 19. Tasks

# T0 — P4 takeover, predecessor gate, branch convergence, focused baseline

## Purpose

Establish a valid P4 implementation base and refuse to start on stale/incomplete
P3 state.

## Preconditions

- user has authorized P4 implementation;
- P3 records are available;
- current branch/worktree identity is known.

## Consumes

```text
P3 final walkthrough
P3 final handoff
C1 checkpoint
current feature branch
current intended integration target
```

## Produces

```text
P4 starting revision
converged branch state
verified P3 completion
focused baseline ledger
verified C4 seam map
verified decision supersession path
```

## Files and symbols

**Read:**

```text
COLL006.C1-CHECKPOINT.md
P3 plan/walkthrough/handoff/publication supplement
static_value/store.rs
static_eval/machine.rs
materialization.rs
modules/static_constant.rs
compiler/lib/expr.rs
accepted literal decisions
```

**Modify:**

```text
COLL006.C1-CHECKPOINT.md
```

only to mark P4 active after all preconditions pass.

## Required implementation shape

1. Verify P3 is:

```text
status: COMPLETE
completion: IMPLEMENTED
verification: truthful accepted state
```

2. Verify all P3 remaining-priority items from the 2026-10-08 handoff are either:

```text
completed
or
explicitly superseded by an approved consultation
```

3. Converge `feat/telos-collection-upgrade` with the intended current base.

4. Rerun the final P3 focused gates **after convergence**.

5. Record any new branch-conflict decisions.

6. Verify exact current C4 APIs.

7. Verify exact current accepted decision files governing brackets.

8. Confirm current compiler headless fallback.

## Forbidden approaches

- no P4 source changes while P3 is partial;
- no carrying a stale pre-convergence P3 PASS forward as post-convergence proof;
- no opportunistic unrelated merge cleanup;
- no default flip.

## Test changes required

None.

## Tests to run now

Use the final P3 handoff's accepted commands.

At minimum, based on current plan state:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_vector
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_array
RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-native-surface-gen -- --root . --check
```

Use the actual final P3 gate set if it has changed.

## Tests explicitly deferred

P4-specific tests.

## Acceptance

```text
P3 accepted
branch converged
P3 focused gates still pass or failures classified
C4 seam map current
P4 marked active
```

## Local STOP / CONSULT triggers

- P3 still partial;
- convergence materially changes semantic ownership;
- current C4 materialization has been redesigned;
- literal decisions already changed in a conflicting direction.

## Checkpoint update

Mandatory.

Record P4 start anchor and P3 accepted predecessor.

---

# T1 — Add canonical StaticSequence storage

## Purpose

Introduce one homogeneous static structural node without changing evaluation or
runtime behavior.

## Preconditions

- T0 complete.
- P3 result types are canonical and accepted.

## Consumes

```text
StaticValueStore
StaticValueRef
TypeStore canonical Array/Vector declarations
StaticInt exact extent terms
```

## Produces

Conceptually:

```rust
structural_id!(StaticSequenceId);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct StaticSequenceData {
    result_type: TypeId,
    elements: Box<[StaticValueRef]>,
}

enum StaticValueData {
    ...
    Sequence(StaticSequenceId),
}
```

plus:

```rust
intern_sequence(...)
get_sequence(...)
```

or exact live-tree equivalents.

## Files and symbols

**Read:**

```text
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/core_surface/*
```

**Modify:**

```text
phalcom-semantic/src/static_value/store.rs
exports if required
```

**Create tests:**

```text
phalcom-semantic/tests/semantic/foundations/static_values_sequence.rs
```

and register with the current semantic test module tree.

## Required implementation shape

### T1.1 Add structural ID/data

Use the existing `structural_id!` model.

Sequence data retains:

```text
result_type
ordered children
```

### T1.2 Extend store state

Add:

```text
sequence nodes
sequence interner
```

with clone behavior consistent with store lineage.

### T1.3 `intern_sequence`

Required checks:

1. every child ref belongs to the same store;
2. `result_type` exists in supplied `TypeStore`;
3. `result_type` is a proper type;
4. nominal origin is canonical Array or Vector;
5. Vector result has a closed exact static Int N if the sequence value is being interned as a fully reduced value;
6. N is nonnegative;
7. N equals logical element count;
8. count comparison uses exact semantic integer logic, not truncating cast.

Array carries no static extent in its nominal type.

### T1.4 Do not duplicate element type checking

The store validates static representation integrity.

It does not redo:

```text
generic inference
assignability
overload resolution
```

### T1.5 Add accessors

Return immutable borrowed logical contents.

## Forbidden approaches

- no Pack encoding;
- no separate `StaticArray`/`StaticVector` stores;
- no VM `Value`;
- no host-size semantic N;
- no mutable static sequence backing;
- no semantic family selection inside store.

## Test changes required

Coverage:

```text
CV-01
CV-02
CV-03
```

Required tests:

```text
Array<Int> + [1,2] succeeds
Vector<Int,2> + [1,2] succeeds
same value interns canonically
Array/Vector remain distinct
foreign child rejected
Vector count mismatch rejected
Vector 0 succeeds
negative malformed closed N rejected defensively
large exact N is compared without truncation
store clone preserves old refs
```

## Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic static_values_sequence
```

If the exact module path filter differs, first list tests and use the actual
non-zero selector.

## Tests explicitly deferred

Evaluator/runtime/cutover tests.

## Acceptance

Static sequence storage exists and all representation invariants pass.

## Local STOP / CONSULT triggers

- correct Vector identity cannot be recovered from ordinary TypeStore application;
- store would need generic inference;
- TypeId lifetime cannot safely be retained like existing Data/Variant result types.

## Checkpoint update

Only if T1 establishes a materially new durable static interface.

Recommended: record `StaticValueData::Sequence` as established after G1, not immediately.

---

# T2 — Integrate Sequence into all canonical static-value laws

## Purpose

Make static Sequence a complete C4 static value, not merely an internable node.

## Preconditions

- T1 passing.

## Consumes

```text
StaticValueData::Sequence
StaticSequenceData
existing recursive structural-value operations
```

## Produces

Sequence support in:

```text
static_value_type
semantic_eq
semantic_hash
content_fingerprint
domain_of policy
debug/exhaustive visitors
```

## Files and symbols

**Modify:**

```text
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/materialization.rs only for exhaustive temporary classification if compilation requires
phalcom-semantic/src/db/fingerprint.rs only if needed
tests
```

## Required implementation shape

### T2.1 Type projection

```text
Sequence -> retained result_type
```

Validate type remains present/proper.

Do not reconstruct from children.

### T2.2 Semantic equality

Equal iff:

```text
same canonical result_type
same length
pairwise semantic_eq children
```

### T2.3 Semantic hash

Hash:

```text
Sequence semantic tag
result type identity
length
child semantic hashes
```

Do not hash SequenceId.

### T2.4 Content fingerprint

Fingerprint:

```text
Sequence content tag
result type identity
length
child content fingerprints
```

Deterministic across store allocation order.

### T2.5 Domain policy

Unless C4 has generalized the API:

```text
Sequence -> StructuralValueNeedsSemanticProducts
```

for APIs that intentionally do not derive structural value domains without
semantic products.

### T2.6 Exhaustiveness audit

Search every `StaticValueData` match across workspace.

Classify each arm:

```text
must support Sequence
should classify Sequence deferred
intentionally unreachable/unsupported
```

## Forbidden approaches

- no raw derived `Hash` as semantic hash;
- no store-local IDs in content fingerprint;
- no family erasure;
- no Array/Vector equality across classifier identity.

## Test changes required

Coverage:

```text
CV-04
CV-05
CV-06
```

Tests:

```text
exact type projection
Array vs Vector unequal
same Vector equal
different order unequal
different child unequal
equal sequences equal hash
equal sequences equal fingerprint
different result family changes fingerprint
store allocation order does not alter logical fingerprint
```

## Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic static_values_sequence
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values
```

Use the smallest existing static-value filter that selects real tests.

## Tests explicitly deferred

Static evaluator and runtime.

## Acceptance

All first-class static-value operations have deliberate Sequence semantics.

## Local STOP / CONSULT triggers

- content fingerprint needs a new global TypeId durability model;
- semantic hash cannot preserve existing C4 laws;
- structural type projection would require a second type relation implementation.

---

# G1 — Static-value foundation gate

## Purpose

Prove T1/T2 before evaluator integration.

## Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic static_values_sequence
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

Confirm non-zero test selection.

## Expected

```text
new Sequence laws PASS
existing structural static-value tests unchanged
semantic crate check PASS
```

Do not broaden after PASS.

---

# T3 — Static evaluator execution of checked bracket literals

## Purpose

Make already-checked Array/Vector bracket construction executable in the C4
static evaluator.

## Preconditions

- G1 PASS.
- `CheckedStaticExpression.sequence_literals` and `CheckedStaticBody.sequence_literals` contain committed-only P3 products.

## Consumes

```text
SequenceLiteralResolutionIndex
SequenceLiteralResolution
SequenceConstructionPlan
SequenceExpansionResolution
StaticValueStore::intern_sequence
```

## Produces

```text
EvaluatorBody::sequence_literals()
Expr::BracketLiteral static execution
fixed sequence construction
exact spread construction
VectorIdentitySpread construction
```

## Files and symbols

**Read/modify:**

```text
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/static_definition.rs
checker analysis types
```

**Tests:**

```text
current COLL006 semantic tests
static constant tests
new focused static sequence evaluator tests
```

## Required implementation shape

### T3.1 Add body accessor

```rust
fn sequence_literals(&self)
    -> &SequenceLiteralResolutionIndex
```

for all `EvaluatorBody` variants.

### T3.2 Add bracket expression arm

At `Expr::BracketLiteral`:

1. resolve expression ID;
2. lookup checked `SequenceLiteralResolution`;
3. missing product => fail closed;
4. reject unsupported family for static evaluation;
5. schedule child work;
6. execute exact construction plan;
7. call `intern_sequence(result_type, children, ...)`.

### T3.3 Fixed lane

For:

```text
SequenceConstructionPlan::Fixed
```

evaluate child expressions exactly once in lexical order.

Validate observed result count against checked plan.

### T3.4 Expanded lane

Consume `exact_expansions`.

Mandatory initial static expansion sources:

```text
closed positional Tuple static value
static Vector Sequence value
```

Do not call runtime iterator methods.

If another P3 exact lane exists at P4 start, verify its static-value representation
before enabling it.

### T3.5 Vector identity spread

For:

```text
VectorIdentitySpread { source_range, extent }
```

require:

```text
checked exact expansion = VectorStorage
source static value = Sequence with Vector result type
source extent == proof extent
source logical count == extent
destination result type agrees
```

If `extent` is not a closed `TypeData::ConstInt`, return
`StaticEvalOutcome::Symbolic` before evaluating the spread or publishing a
Sequence root. Do not specialize the extent in the evaluator. For a closed
extent, perform every check above, then construct the logical child list.

No symbolic arithmetic.

### T3.6 Static Array

Intern with exact checked:

```text
Array<T>
```

result type.

### T3.7 Static Vector

Intern with exact checked:

```text
Vector<T,N>
```

result type.

### T3.8 Static List boundary

If semantic checking selects `List` in a static evaluator context:

```text
do not construct StaticValueData::Sequence(List)
```

Use the current C4 unsupported-static-expression/result mechanism plus a precise
diagnostic if the checker supports one at this boundary.

No frozen List is invented.

### T3.9 Budget/cancellation

Every sequence construction loop must participate in the existing query budget.

Do not hide an unbounded O(n) loop behind one work item if the current evaluator
budget semantics require per-element charging.

### T3.10 Failure atomicity

Failure may leave append-only child static values in the store.

It may not publish a successful root outcome.

## Forbidden approaches

- no family inference;
- no N inference;
- no runtime Iterator protocol;
- no AST recount as authoritative type semantics;
- no List fallback;
- no runtime constructor invocation;
- no second evaluator.

## Test changes required

Coverage:

```text
CV-07
CV-08
CV-09
CV-10
CV-28
CV-29
CV-35
```

Required tests:

```text
explicit static Array literal reduces
explicit static Vector literal reduces
nested sequence reduces
typed empty Vector<Int,0> reduces
pre-cutover `const empty = []` remains List-family, diagnoses unresolved T, and publishes no static value
post-T10 the same source selects Vector<T,0>, diagnoses unresolved T, and publishes no static value
typed control `const empty: Vector<Int,0> = []` remains valid
fixed children evaluated source-order/exactly-once where observable static operations allow
closed Tuple spread
static Vector spread
symbolic identity spread
tampered/missing proof fails closed
cancellation
zero/small budget
static List does not silently freeze
Array.new/Vector.new do not become statically authorized merely because bracket literals are supported
```

## Tests to run now

Use focused static evaluator/constant filters.

At minimum:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic static_constants
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006
```

Add/use a narrower `static_sequence` filter if the new tests are organized that way.

## Tests explicitly deferred

Runtime materialization.

## Acceptance

Checked Array/Vector bracket literals reduce to canonical static Sequence values.

## Local STOP / CONSULT triggers

- evaluator requires phase/family reconstruction;
- evaluator needs runtime collection constructors;
- committed-only P3 publication invariant is false;
- static spread needs a second cardinality engine.

## Checkpoint update

After G2, record static sequence evaluation as established.

---

# G2 — Static-evaluation gate

## Purpose

Prove the static semantic vertical before materialization.

## Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic static_sequence
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic static_constants
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006
```

Use actual registered non-zero filters.

## Expected

```text
explicit const/static Array and Vector reduce
existing primitive/structural static evaluation remains green
bare default still List until T10
```

The final condition is intentional.

---

# T4 — Add sequence materialization descriptors

## Purpose

Project reduced static sequence values into self-contained backend/runtime
materialization data.

## Preconditions

- G2 PASS, or G2 `BASELINE_BLOCKED` under §0.4 with every P4-active G2
  discriminator passing and all remaining failures matched to inherited D
  failures.

## Consumes

```text
StaticValueData::Sequence
StaticSequenceData
existing MaterializationDescriptor
```

## Produces

Conceptually:

```rust
enum SequenceMaterializationKind {
    Vector,
    FrozenArray,
}

enum MaterializationDescriptor {
    ...
    Sequence {
        kind: SequenceMaterializationKind,
        elements: Arc<[MaterializationDescriptor]>,
    },
    ...
}
```

Equivalent immutable descriptor graph representation is acceptable.

## Files and symbols

**Modify:**

```text
phalcom-semantic/src/materialization.rs
static-value materialization tests
```

## Required implementation shape

### T4.1 Family classification

Determine kind from retained result type at descriptor projection time.

Use canonical semantic family identity.

Selected bounded path: retain `StaticSequenceFamily::{Array,Vector}` on
`StaticSequenceData`, deriving it from the canonical result type during
`intern_sequence` and rejecting any noncanonical family there. This is immutable
validated projection metadata, not a second mutable authority, and avoids
adding `TypeStore` to C4 descriptor call sites. Sequence descriptors receive a
sequence-specific materialization capability; they are not classified as
primitive values.

Do not use element count.

If `MaterializationDescriptor::from_static_value` currently lacks TypeStore access
needed to distinguish Array/Vector from `result_type`, choose one bounded solution:

Preferred:

```text
store StaticSequenceData retains explicit checked family in addition to result_type
```

only if this does not duplicate mutable semantic authority.

Alternative:

```text
extend descriptor projection API to receive canonical TypeStore
```

if C4 descriptor call sites can do so coherently.

The implementer must not guess from type formatting or names.

If this decision materially changes C4 descriptor API across unrelated consumers,
STOP AND CONSULT.

### T4.2 Recursive children

Recursively project children.

### T4.3 Deferred nested structural values

If any child is:

```text
Tuple
Record
Data
Variant
Pack
```

and general structural materialization remains deferred, the sequence's runtime
materialization must remain explicitly deferred/unsupported.

Do not implement those families inside P4.

### T4.4 Capability

Use a truthful capability category.

Do not call it primitive.

If adding:

```text
MaterializationCapability::Structural
```

is clean and bounded, use it.

Do not imply all structural kinds are materializable if they are not.

## Forbidden approaches

- no StaticValueRef in descriptor;
- no TypeStore pointer;
- no ExpressionId;
- no checker proof in runtime descriptor;
- no implicit general C4.P8 completion.

## Test changes required

Coverage:

```text
CV-11
CV-12
```

Tests:

```text
Vector primitive children -> Sequence(Vector)
Array primitive children -> Sequence(FrozenArray)
nested materializable sequence -> recursive descriptor
sequence containing deferred Tuple -> deferred outer capability
descriptor clone is self-contained
```

## Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic static_values_sequence
```

The descriptor regression lives in `static_values_sequence`; the repository has
no separate `static_materialization` test module/filter.

## Acceptance

Every successfully reducible sequence has a truthful materialization descriptor
or a truthful deferred result.

## Local STOP / CONSULT triggers

- canonical family cannot be determined without broad descriptor architecture change;
- nested materialization forces all C4 structural families into scope.

---

# T5 — VM recursive sequence materialization

## Purpose

Turn supported Sequence descriptors into runtime Vector/frozen Array objects
without AST execution or GC hazards.

## Preconditions

- T4 passing.

## Consumes

```text
MaterializationDescriptor::Sequence
VM static constant materializer
VectorObject
ArrayObject
owner module static constant cache
```

## Produces

```text
runtime Vector materialization
runtime frozen Array materialization
recursive child materialization
GC-safe temporary roots
```

## Files and symbols

**Read/modify:**

```text
phalcom-core/src/modules/static_constant.rs
phalcom-core/src/heap/array.rs
phalcom-core/src/heap/vector.rs
existing GC/root helper modules as required
```

## Required implementation shape

### T5.1 Factor recursive descriptor materializer

Prefer:

```rust
fn materialize_descriptor(
    &mut self,
    descriptor: &MaterializationDescriptor,
    roots: &mut <existing rooted temporary mechanism>,
) -> PhResult<Value>
```

or equivalent.

`ensure_static_constant_materialized` remains the owner-cache publication boundary.

### T5.2 Vector

Materialize children, then construct native immutable Vector storage.

No intermediate List or Array object.

### T5.3 Array

Materialize children and create Array in frozen state before publishing the Value.

Add a private construction helper if necessary:

```rust
ArrayObject::from_vec_frozen(...)
```

Preferred over:

```text
allocate mutable public Array
publish
freeze later
```

### T5.4 GC rooting

Every materialized heap child that survives across another allocation must be
reachable from the GC root set.

Reuse an existing VM rooting mechanism.

If none exists and a new GC-root architecture is needed: STOP.

### T5.5 Owner publication

Existing shape:

```text
if owner cache contains value -> reuse
materialize descriptor
insert owner cache
return RuntimeStaticConstantRef
```

must remain.

Do not insert before full materialization succeeds.

### T5.6 Resource failure

Allocation failure returns controlled runtime error.

No panic/truncation/partial publication.

## Forbidden approaches

- unrooted `Vec<Value>` across allocations unless current GC explicitly treats it as a root;
- runtime AST execution;
- List staging;
- mutable-public Array then freeze after publication;
- duplicate owner cache;
- copying through import descriptors.

## Test changes required

Coverage:

```text
CV-13
CV-14
CV-15
CV-16
CV-17
CV-20
```

Add low-level descriptor materialization tests first, then source-level tests in T6.

Required direct tests:

```text
Sequence(Vector) -> Object::Vector
Sequence(FrozenArray) -> Object::Array and is_frozen
nested sequence
heap String/LargeInt children survive forced GC during/after construction
failure leaves owner cache absent
repeated ensure returns same RuntimeStaticConstantRef/value
```

## Tests to run now

Use targeted core tests only.

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib modules::static_constant
```

and the new exact focused core filter if integration tests are added.

Do not run broad `core` yet.

## Acceptance

Descriptor-to-runtime materialization is correct and GC-safe.

## Local STOP / CONSULT triggers

- existing heap GC cannot root temporary child graph without architectural changes;
- Array cannot be created frozen before publication;
- runtime classifier requires a new const-specific class.

---

# T6 — Converge same-module and linked static-constant reads for sequences

## Purpose

Make runtime use of static Sequence constants actually work through existing
canonical static-constant ownership.

This task prevents a semantic-only success where:

```text
const x = static sequence
```

reduces, but:

```text
let y = x
```

still fails as `StaticConstStructuralDeferred`.

## Preconditions

- T5 direct materializer passes.

## Consumes

```text
StaticConstantLoweringSpec
MaterializationDescriptor::Sequence
VM::ensure_static_constant_materialized
RuntimeStaticConstantRef
compiler same-module static const read branch
linked import/export static constant path
```

## Produces

```text
same-module runtime read of sequence const
linked read of sequence const
export/re-export read of sequence const
```

## Files and symbols

**Read/modify:**

```text
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/modules/static_constant.rs
phalcom-core/src/modules/materialize.rs          # expected reuse; modify only if needed
compiler/execution static_const tests
module linkage tests
```

## Required implementation shape

### T6.1 Preserve primitive behavior unless refactor is clearly beneficial

Primitive direct constant emission already works.

Do not redesign it merely to make all descriptors uniform.

### T6.2 Sequence same-module reads

For:

```text
MaterializationDescriptor::Sequence
```

use the canonical static constant owner/materialization mechanism.

Preferred semantic result:

```text
one owner-held runtime sequence object
same-module read references that canonical materialized value
imports/re-exports reference the same canonical owner
```

The exact bytecode/constant-pool mechanism is mechanically flexible.

The compiler must not rebuild a Vector/Array from descriptor children independently
at each read site.

### T6.3 Linked reads

Existing:

```text
RuntimeLinkedRead::StaticConstant(RuntimeStaticConstantRef)
```

should work unchanged after T5.

### T6.4 Exports/re-exports

Existing:

```text
RuntimeExportRef::StaticConstant
```

should work unchanged.

### T6.5 Runtime initializer absence

Inspect module initializer chunk.

No bracket child expression or static computation should execute at runtime for
the const.

## Forbidden approaches

- per-read fresh Sequence construction;
- per-import copies as ownership;
- global slot for const sequence;
- runtime initializer bytecode;
- symbol spelling as static identity.

## Test changes required

Coverage:

```text
CV-16
CV-17
CV-34
```

Required tests:

Module-level sources:

```telos
const V = <Int,3>[1,2,3]
let v1 = V
let v2 = V

const A: Array<Int> = <Int>[4,5,6]
let a1 = A
let a2 = A
```

Also:

```text
provider export
selective import
renamed import
qualified access
re-export
```

Assertions:

```text
correct classifier/content
canonical source owner retained
runtime initializer absent
runtime value survives GC
```

Internal object-identity reuse may be asserted where the current static constant
owner contract exposes it.

## Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core static_const_sequence
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core static_const
```

Use actual test filters; confirm non-zero selection.

## Tests explicitly deferred

Literal default cutover.

## Acceptance

Explicit/contextual const Array/Vector can be read at runtime in same-module and
linked-module cases with no initializer re-execution.

## Local STOP / CONSULT triggers

- same-module compiler cannot reference canonical owner value without new cross-layer identity architecture;
- linked import path constructs copies;
- local const support becomes a prerequisite for module const support.

---

# G3 — Runtime materialization gate

## Purpose

Prove static Sequence from semantic reduction through runtime use.

## Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic static_sequence
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core static_const
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_vector
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_array
```

Expected:

```text
explicit/contextual const Array/Vector work
bare default remains legacy List until T10
```

Do not flip defaults yet.

---

# T7 — Direct const Array mutation provenance and diagnostic

## Purpose

Provide compile-time diagnostics for direct provable mutation of a const Array
while preserving the runtime frozen guard for aliases.

## Preconditions

- G3 PASS.

## Consumes

```text
BindingId
BindingState / module binding facts
BindingKind/BindingPhase
static constant identity
synthesize_set_index_expr
Array nominal type
```

## Produces

A stable semantic check:

```text
direct receiver denotes static const
AND
receiver type is Array
AND
operation is mutating Array setter
    -> diagnostic
```

## Files and symbols

**Read:**

```text
phalcom-ast/src/ast.rs BindingKind/BindingPhase
phalcom-semantic/src/module_bindings.rs
phalcom-semantic/src/checker/binding.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/expression.rs::synthesize_set_index_expr
Array source/native setter identity
diagnostic enum
```

**Modify:**

Smallest semantic owner necessary.

## Required implementation shape

### T7.1 Provenance requirement

Use stable binding identity.

Do not use source name/spelling.

Accepted bounded options, in preference order:

1. existing binding metadata already exposes static phase at P4 start — use it;
2. add a checker-local `BindingId -> StaticConstantId/phase` provenance index seeded when static bindings are declared/imported;
3. extend `BindingState` with declaration phase only if the phase is genuinely a stable general semantic binding fact and all fingerprint/publication consequences remain bounded.

Option 3 is not automatically authorized if it cascades broadly.

### T7.2 Subscript write

In `synthesize_set_index_expr`, after receiver analysis establishes:

```text
binding_read
receiver type
```

and before mutating callable application is accepted:

```text
if direct static const Array receiver:
    diagnostic
    return invalid typed assignment result
```

### T7.3 Named mutator

If the public Array named replacement method has an exact canonical callable
identity available in the ordinary call path, apply the same guard there.

Do not build a general effect system.

### T7.4 Alias defense

For:

```telos
const A: Array<Int> = <Int>[1,2]
let alias = A
alias[0] = 3
```

semantic alias propagation is optional for P4.

Runtime FrozenMutation is mandatory.

### T7.5 Diagnostic

Add/use a code whose meaning is const object mutation, not rebinding.

Do not reuse wording that recommends `let mut`.

## Forbidden approaches

- compiler-only diagnostic;
- AST search for `const`;
- variable-name sets as semantic authority;
- weakening frozen runtime guard;
- full mutation-effect analysis.

## Test changes required

Coverage:

```text
CV-18
CV-19
CV-20
CV-21
```

Tests:

```text
direct module const Array subscript write -> semantic error
imported direct const Array write -> semantic error if canonical binding provenance is available
ordinary let Array mutation remains legal
let mut Array mutation remains legal
const Vector mutation remains rejected by lack of mutator/immutable API
alias to const Array reaches runtime FrozenMutation
const Array reports Array class
const Array not admitted as stable key merely because frozen
```

## Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_array
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core static_const
```

## Acceptance

Direct const Array mutation is diagnosed, aliases remain runtime-safe, ordinary
Array semantics are unchanged.

## Local STOP / CONSULT triggers

- binding phase cannot be represented without broad general binding redesign;
- mutator identity is too dynamic to guard without effect architecture;
- tests require const Array to become a different classifier.

---

# G4 — Const-safety gate

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_array
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core static_const
```

Expected:

```text
direct const mutation semantic rejection
alias runtime rejection
ordinary Array mutation unchanged
```

---

# T8 — Repository-wide bare-bracket migration census

## Purpose

Know exactly what will change before changing the default.

## Preconditions

- static/runtime P4 infrastructure works;
- branch is still converged/current.

## Consumes

```text
current parser/AST
current repository source
current tests/docs
```

## Produces

Durable artifact:

```text
docs/implementation/COLL006/COLL006.C1/
    COLL006.C1.P4-bracket-migration-census.md
```

## Required implementation shape

### T8.1 Use AST-aware classification

Preferred:

- repository/parser tooling;
- a small one-off analysis helper if needed;
- parser AST traversal.

Text search is discovery assistance only.

### T8.2 Scope

Classify bare bracket literals in:

```text
phalcom-core/core/universe/**/*.ph
packages/**/*.ph
examples/**/*.ph
test source fixtures
language corpus
benchmarks/source fixtures
semantic fixture strings
compiler/runtime fixture strings
spec/examples directly representing current semantics
accepted decisions affected by the cutover
```

### T8.3 Classifications

Use exactly:

```text
LIST_RESIZING
LIST_IDENTITY
FIXED_RUNTIME
STATIC_IMMUTABLE
READ_ITERATE_ONLY
LEGACY_TEST
AMBIGUOUS
```

For every entry record:

```text
path
source range/line
classification
reason
required migration
owner/test impact
```

### T8.4 Mutation interpretation

This alone does not prove List:

```telos
xs[i] = value
```

Array supports element replacement.

These prove/strongly imply List:

```text
append
prepend
insert changing size
remove changing size
pop
clear/removeAll
explicit List type identity
APIs requiring List
```

## Forbidden approaches

- regex-only final inventory;
- assuming every old bracket literal needs `List`;
- changing source during census;
- default flip during census.

## Tests to run now

No compilation is required for the census artifact itself.

If a temporary analysis script is created, test only that it classifies known
fixtures correctly.

## Acceptance

Zero in-scope bare literal sites remain `UNCLASSIFIED`.

Every `AMBIGUOUS` site has a manual decision before T9 starts.

## Local STOP / CONSULT triggers

- a major source family depends on semantics not expressible by List/Array/Vector;
- a classification requires changing public APIs rather than selecting an existing family.

## Checkpoint update

Record census completion and total counts by class.

---

# T9 — Migrate List-dependent source and amend normative literal decisions/specs

## Purpose

Make the repository safe for the new default before changing executable semantics.

## Preconditions

- T8 complete.
- no unresolved `AMBIGUOUS` census entries.

## Consumes

Migration census.

## Produces

```text
explicit List at every LIST_RESIZING/LIST_IDENTITY site
reviewed handling of READ_ITERATE_ONLY
rewritten LEGACY_TEST expectations
accepted decision/spec update ready/landed
```

## Files and symbols

From census only.

Do not broad-edit unrelated bracket syntax.

## Required implementation shape

### T9.1 LIST_RESIZING

Convert to explicit List using the canonical supported source form.

Prefer existing constructors/contextual annotations.

Do not invent a migration-only literal syntax.

### T9.2 LIST_IDENTITY

Make List nominal requirement explicit.

### T9.3 FIXED_RUNTIME

Leave bare, intentionally expecting future Array.

### T9.4 STATIC_IMMUTABLE

Leave bare, intentionally expecting future Vector.

### T9.5 READ_ITERATE_ONLY

Choose Array/List/Vector based on ownership and phase.

Record rationale in census.

### T9.6 LEGACY_TEST

Rewrite tests to assert new semantics or explicit legacy List use.

Do not preserve old expected behavior by compiler hacks.

### T9.7 Normative decision update

Amend/supersede current accepted List-literal decisions.

Required target normative rule:

```text
[...] is a contextual homogeneous sequence literal.

explicit:
    <T>[...]     Array
    <T,N>[...]   Vector

unique expected:
    List / Array / Vector

otherwise:
    runtime -> Array
    static  -> Vector

phase default never resolves overload ambiguity
```

Update current spec/examples consistently.

## Forbidden approaches

- flip defaults before source migration passes;
- convert every bare literal to List;
- leave normative accepted decision saying bare brackets intrinsically construct List;
- rewrite historical archaeology that is intentionally preserved as superseded history.

## Tests to run now

Run only affected package/corpus tests for migration slices.

Then at the end:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_array
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_vector
```

Bare defaults are still old until T10, so migrated List-dependent sites should
already be explicit and green.

## Acceptance

```text
all dynamic-List-required owned sites explicit
legacy tests ready for new default
normative decision/spec no longer blocks cutover
```

## Local STOP / CONSULT triggers

- migration needs a new collection API;
- large corpus semantics are ambiguous after manual review;
- decision process cannot accept the new rule before executable cutover.

---

# G5 — Pre-cutover migration gate

Before T10:

1. re-run census and prove no new/unclassified owned bare literals appeared;
2. verify every List-required site is explicit;
3. verify accepted decision/spec update is landed or accepted according to repository policy;
4. verify G3/G4 remain valid after migration.

No default flip if G5 fails.

---

# T10 — Atomic semantic default cutover and compiler fallback tightening

## Purpose

Change the language default only after all supporting semantics, materialization,
safety, migration, and normative rules are ready.

## Preconditions

- G5 PASS.

## Consumes

```text
EvaluationPhase
SequenceLiteralResolution
migrated source
updated normative decisions
```

## Produces

```text
runtime bare [...] -> Array
static bare [...]  -> Vector
headless compiler missing-lowering -> fail closed
```

## Files and symbols

**Modify:**

```text
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/sequence_literal.rs
phalcom-core/src/compiler/lib/expr.rs
focused semantic/compiler tests
```

Parser should require no semantic-family change.

## Required implementation shape

### T10.1 Semantic default helper

Conceptually:

```rust
fn default_family(ctx: &CheckingContext<'_>) -> SequenceLiteralFamily {
    match ctx.evaluation_phase() {
        EvaluationPhase::Runtime => SequenceLiteralFamily::Array,
        EvaluationPhase::Static => SequenceLiteralFamily::Vector,
    }
}
```

Use the actual existing EvaluationPhase API.

The binding checker owns the phase of each initializer. Check a `const`
initializer under `EvaluationPhase::Static` before annotation and expression
analysis, then restore the caller phase before declaration/flow processing.
Runtime `let`/`let mut` bindings inherit their caller phase. This keeps the
ordinary module binding product aligned with the canonical static-constant
product without introducing a second type projection.

### T10.2 Selection order

Preserve:

```text
explicit head
    else unique contextual family
    else phase default
```

### T10.3 Static Vector extent

Static bare default must use the same existing Vector cardinality path.

No new extent code.

### T10.4 Empty literal

Runtime:

```text
[] -> Array family, T may remain underconstrained
```

Static:

```text
[] -> Vector family, N=0, T may remain underconstrained
```

Do not invent element type.

### T10.5 Compiler fallback tightening

Current context-free compiler fallback must not become:

```text
headless -> Array
```

because compiler does not own phase.

Change headless no-lowering branch to:

```text
MissingSequenceLiteralLowering
```

Retain only syntax-explicit compatibility paths that do not require contextual or
phase reasoning, if current public compiler architecture still needs them.

### T10.6 No overload tie-break

Current language specifications state that parameter/result types do not create
same-selector overloads (`docs/specs/objects/impl.md` and
`docs/specs/objects/traits.md`). Therefore the requested Array/Vector,
List/Array, List/Vector, and three-family type-overload matrix is not expressible
in the current language. Do not introduce a new overload model for P4.

CV-25's replacement proof is a live ambiguous trait-dispatch case whose two
same-selector candidates require distinct List and Vector argument families.
The unbound bracket argument receives the runtime Array default, but the
dispatch result must retain both candidates, emit `TraitDispatchAmbiguous`, and
publish no selected target. This verifies that literal family selection cannot
repair an unrelated ambiguous dispatch result. If parameter-type overloading is
specified in the future, add the pairwise family matrix with that design.

Do not change call candidate ranking to force default selection.

## Forbidden approaches

- parser family selection;
- compiler phase detection;
- overload preference;
- source syntax counting as Vector type authority;
- implicit List fallback;
- changing explicit family semantics.

## Test changes required

Coverage:

```text
CV-22
CV-23
CV-24
CV-25
CV-26
CV-27
CV-28
CV-35
CV-36
```

Required matrix:

```telos
let x = [1,2,3]                    // Array<Int>
const y = [1,2,3]                  // Vector<Int,3>
let l: List<_> = [1,2,3]           // List<Int>
let a: Array<_> = [1,2,3]          // Array<Int>
let v: Vector<_,_> = [1,2,3]       // Vector<Int,3>
const a: Array<_> = [1,2,3]        // static Array
let x = <_>[1,2,3]                 // Array
let x = <_,_>[1,2,3]               // Vector
```

Ambiguity (current-language replacement; see T10.6):

```text
ambiguous same-selector trait candidates requiring List vs Vector arguments;
runtime-default Array argument does not select either candidate
```

Compiler:

```text
headless no lowering -> error
explicit Vector no checked proof -> error
```

## Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_array
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_vector
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core static_const
```

Then run directly affected migrated corpus/package filters.

## Acceptance

The new default is active with no compiler/parser duplicate authority and no
known owned List-semantic regressions.

## G6 execution result — 2026-10-09

**PASS / FOCUSED_TESTED.** The `const` phase was made explicit at the binding
owner after G6 coverage exposed that ordinary module binding facts still saw a
bare const bracket under the runtime phase. The regression confirms module
`const values = [1, 2]` publishes `Vector<Int, 2>` while runtime `let values =
[1, 2]` publishes `Array<Int>`; static-constant evaluation and ordinary module
binding facts now agree.

| Coverage | Passing evidence |
|---|---|
| CV-22 runtime default | `bare_runtime_sequence_literal_defaults_to_array`; `core_array` (11 passed) |
| CV-23 static default | `test_bare_static_sequence_defaults_to_exact_vector`; `top_level_const_and_let_bindings_use_their_phase_default_families` |
| CV-24 explicit/context precedence | `test_contextual_array_literal`, `test_contextual_list_literal_remains_available`, `test_explicit_array_literal`, `vector_lowering_uses_checked_extent_and_direct_family_bytecodes` |
| CV-25 ambiguity | `ambiguous_trait_dispatch_does_not_use_runtime_bracket_default_to_select_a_candidate` |
| CV-26 compiler fail-closed | `context_free_bracket_compilation_requires_checked_lowering`; explicit Array compatibility is covered by the same test |
| CV-27 List migration | G5 census and migration review; `test_contextual_list_literal_remains_available` |
| CV-28 static List boundary | `test_static_sequence_list_and_missing_proof_fail_closed` |
| CV-35 empty literal | `test_unconstrained_empty_static_sequence_remains_diagnostic_after_cutover` |
| CV-36 runtime source order | `bare_array_literal_preserves_source_order_of_element_effects` |

Gate commands/results:

```text
cargo test -p phalcom-semantic --test semantic coll006         49 passed
cargo test -p phalcom-core --test core core_array               11 passed
cargo test -p phalcom-core --test core core_vector               8 passed
cargo test -p phalcom-core --test core static_const             23 passed
cargo test -p phalcom-semantic --test semantic static_constants 21 passed, 3 failed
```

The three `static_constants` failures are the existing D4 computed-Record-key
`RecordKeyNotStatic` baseline cases; all static-sequence cases in that filter
passed. G2 remains `BASELINE_BLOCKED` by D4. T11/G7/T12, workspace all-target
check, native-surface recheck, full workspace tests, clippy, and release
certification remain outside this G6 stop point. The G5 migration census is a
historical 495-site snapshot; T12 should refresh it to include the additional
T10 regression literals added after G5.

## Local STOP / CONSULT triggers

- overload analyzer must be modified to prefer phase;
- standalone compiler API fundamentally requires headless semantic default without analysis;
- migrated source still depends on implicit List.

---

# G6 — Literal semantics and ambiguity gate

## Purpose

Prove the atomic cutover itself.

## Required evidence

Map each:

```text
CV-22
CV-23
CV-24
CV-25
CV-26
CV-27
CV-28
CV-35
CV-36
```

to named passing tests.

At minimum:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_array
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_vector
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core static_const
```

Do not broaden after PASS until T11 changes incremental/module/GC tests.

---

# T11 — Incremental, module durability, GC, and hostile boundary matrix

## Purpose

Prove P4's nonlocal correctness after the cutover.

## Preconditions

- G6 PASS.

## Consumes

Complete P4 behavior.

## Produces

Focused durability evidence.

## Files and symbols

Tests primarily.

Production changes only when a focused failure is causally attributed to P4.

## Required implementation/testing shape

### T11.1 Incremental static content

Cases:

```text
const x=[1,2] -> [1,2,3]
const x=[1,2] -> [1,9]
const Array annotation removed -> default Vector
const bare changed from const to let -> Vector -> Array
let bare changed to const -> Array -> Vector
Vector spread source loses exactness
Vector extent changes
invalid -> valid
valid -> invalid
```

Assert:

```text
cold/incremental semantic parity
no stale sequence product
no stale descriptor
correct fingerprint invalidation
unaffected constant reuse where expected
```

### T11.2 Module identity

Provider/consumer/re-export matrix:

```text
selective import
rename
qualified access
re-export
two consumers
```

Check canonical static owner.

### T11.3 GC

Force GC:

```text
before sequence runtime read
after owner materialization
during nested descriptor child construction if test hook permits
while only owner module cache roots the value
after import/re-export references established
```

### T11.4 Runtime alias mutation

Reprove frozen Array mutation through aliases.

### T11.5 Source effects

Reprove runtime default Array/explicit Vector literal child order/exactly-once
using existing P2/P3 runtime tests plus new bare-default cases.

### T11.6 Resource boundaries

Include:

```text
zero length
large but supported concrete length
host count conversion boundary
controlled allocation failure where existing test hooks exist
```

Do not add artificial global limits.

### T11.7 Static List

Reprove no accidental frozen List/static Sequence(List).

### T11.8 Arbitrary constructor boundary

Reprove static bracket support did not authorize unrelated runtime constructors.

## Forbidden approaches

- broad source edits during stabilization;
- weakening fingerprints;
- retaining LKG as current executable product after failure;
- test-only production special cases.

## Tests to run now

Focused:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic static_constants
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core static_const
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_array
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_vector
```

Use narrower exact filters first; run the larger incremental filter only at the
coherent stabilization gate.

If native authored signatures changed:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-native-surface-gen -- --root . --check
```

## Acceptance

All P4 hostile/incremental/module/GC obligations pass or have truthful classified
baseline failures that do not invalidate P4 acceptance.

## Local STOP / CONSULT triggers

- cold/incremental disagreement reflects semantic authority split;
- GC requires new root architecture;
- module owner identity differs by import path;
- successful prior revision LKG is accidentally used as current sequence executable product.

---

# G7 — P4 stabilization gate

## Purpose

Collect the complete focused acceptance evidence.

## Run

Exact final commands depend on live registered tests.

Required categories:

```text
semantic COLL006
static sequence store/evaluator/constants
semantic incremental sequence cases
core Array
core Vector
core static constants
module import/export static constants
native surface --check if applicable
workspace all-target check
scoped format/diff hygiene
```

Expected command floor:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic static_sequence
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic static_constants

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_array
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_vector
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core static_const

RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
git diff --check
```

Use `cargo fmt` only according to repository scoped-format policy; do not reformat
unrelated inherited drift.

No workspace tests/clippy/release gate is implied.

---

# T12 — Lifecycle closure and P5 handoff

## Purpose

Make repository state truthful and resumable.

## Preconditions

- G7 complete.
- no unresolved P4-owned A/B failures.
- any C failures are nonblocking and recorded.
- all consultations recorded.

## Produces

```text
P4 COMPLETE/IMPLEMENTED/<truthful verification>
checkpoint updated
PROGRAM/STATUS updated
migration census finalized
P4 walkthrough
P4 handoff
P5 entry assumptions
```

## Required records

### Walkthrough

Create:

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P4-walkthrough.md
```

Include:

- final revision;
- static Sequence architecture;
- exact descriptor/runtime path;
- const Array provenance solution;
- migration totals by census class;
- normative decision supersession;
- default flip;
- compiler fallback tightening;
- tests added;
- exact commands/results;
- consultations/deviations;
- deferred tests;
- baseline failures;
- verification classification.

### Handoff

Create:

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P4-handoff.md
```

P5 inherits:

```text
bare runtime brackets = Array
bare static brackets = Vector
explicit List preserved
Array ordinary mutability
const Array frozen runtime behavior
Vector immutable runtime/static behavior
static sequence C4 integration
```

Do not ask P5 to rediscover these.

## Checkpoint update

Plans ledger:

```text
P4 status
completion
verification
outcome
```

Set next action to P5 only if P4 acceptance is actually satisfied.

## Tests to run now

No new tests solely for documentation changes.

Review:

```sh
git diff --check
```

and confirm artifact links/record references are valid.

## Acceptance

P4 lifecycle truth matches implemented behavior and evidence.

---

# 20. Verification gates summary

| Gate | Proves | Must pass before |
|---|---|---|
| G1 | static Sequence storage/laws/fingerprint | evaluator work considered stable |
| G2 | checked static Array/Vector evaluation | descriptor/runtime work considered stable |
| G3 | runtime materialization + runtime reads | const safety/migration |
| G4 | const Array mutation safety | migration/cutover |
| G5 | source + normative migration readiness | default flip |
| G6 | new literal defaults + ambiguity/compiler authority | hostile/incremental stabilization |
| G7 | focused P4 acceptance | lifecycle completion |

---

# 21. Final focused acceptance mapping

At completion the walkthrough must contain an actual result for every coverage ID.

| Coverage | Required proof location |
|---|---|
| CV-01–06 | static-value sequence tests |
| CV-07–10 | static evaluator/constant tests |
| CV-11–12 | materialization descriptor tests |
| CV-13–17 | core static-constant/runtime tests |
| CV-18–21 | semantic const mutation + Array runtime tests |
| CV-22–29 | COLL006 literal/ambiguity/compiler tests |
| CV-30–33 | incremental semantic tests |
| CV-34 | module import/re-export tests |
| CV-35–36 | literal boundary/runtime effect tests |
| CV-37 | accepted decision/spec + migration census review |

No row may be silently omitted.

If a coverage obligation is not applicable because live architecture superseded
the mechanism, document why and map it to the replacement proof.

---

# 22. Performance/resource evidence

P4 is not the final performance certification plan, but it has architecture-level
resource requirements.

## 22.1 Static store

Target:

```text
intern_sequence expected O(n)
semantic_eq O(n) logical traversal
semantic_hash O(n)
content_fingerprint O(n)
```

No full generic inference per child inside static store.

## 22.2 Evaluator

Sequence work is proportional to logical children.

No hidden unbudgeted expansion.

## 22.3 Materialization

Target per materialized sequence node:

```text
one logical child pass
one final dense sequence allocation
no intermediate List
no initializer execution
owner cache prevents repeated full materialization
```

## 22.4 Runtime literal performance

P4 default flip must reuse P2/P3 direct lowering.

No:

```text
bare -> List -> convert Array
const -> Array -> convert Vector
```

## 22.5 Evidence

Focused tests/disassembly/heap assertions are sufficient for P4.

Microbenchmark certification remains P9 unless an obvious regression appears.

---

# 23. Checkpoint bookkeeping

## At P4 start

Record:

```text
P3 final revision/status
P4 start revision
branch convergence state
active_plan = P4
```

## During P4

Update checkpoint only for:

- G1 static Sequence authority established;
- G2 static evaluator established;
- G3 materialization established;
- const safety consultation/decision;
- migration census completion;
- normative decision supersession;
- G6 cutover completion;
- meaningful baseline/deferred failure;
- architectural consultation.

Do not append a chronological log for every edit.

## At completion

Record:

```text
P4 final revision
final status/completion/verification
coverage/gate results
migration counts
accepted decision references
residual failures
next action P5
```

---

# 24. Walkthrough deliverable

`COLL006.C1.P4-walkthrough.md` must answer:

1. What exact source behavior changed?
2. What static-value representation landed?
3. How is Array distinguished from Vector statically?
4. How are equality/hash/fingerprint defined?
5. How does static bracket evaluation use checked products?
6. How does runtime materialization avoid AST re-execution?
7. How are temporary Values rooted during recursive materialization?
8. How is const Array frozen?
9. How is direct const Array mutation diagnosed?
10. How do aliases remain safe?
11. How do same-module/import/re-export static reads resolve?
12. How many bare literals were classified in each migration category?
13. Which sites became explicit List?
14. Which decisions/specs were superseded/amended?
15. What exact tests ran and with what selected counts/results?
16. What was deliberately deferred?
17. What baseline failures remain?
18. Why is P5 safe to proceed?

---

# 25. Handoff deliverable

`COLL006.C1.P4-handoff.md` must provide:

```text
current branch/revision
P4 final state
stable interfaces
stable invariants
default family behavior
static sequence interfaces
runtime materialization interfaces
const Array safety behavior
migration status
decision/spec status
residual failures
P5 objective
first commands for P5
things P5 must not redesign
```

P5 must not be told to revisit P4 architecture unless a new concrete failure
invalidates it.

---

# 26. Completion truth table

Do not conflate:

```text
StaticSequence enum added
    != static sequence evaluator complete

static evaluator complete
    != runtime materialization complete

runtime materialization complete
    != source migration complete

source migration complete
    != default flip complete

default flip complete
    != incremental/GC/durability verified

focused gates passed
    != release certified
```

Valid final P4 classification will normally be:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

unless actual evidence justifies a different repository-defined state.

Do not claim `RELEASE_COMPLETE` from this plan's default verification budget.

---

# 27. Plan self-review

This plan is valid only if, before implementation begins, the implementer can
answer YES to each:

- [ ] P3 is actually complete.
- [ ] branch convergence has occurred.
- [ ] no unresolved inference-owned sequence publication gap remains.
- [ ] exact negative Vector formation is closed.
- [ ] Array homogeneous source typing is closed.
- [ ] current static-value store still owns structural values.
- [ ] current checked static products carry committed sequence resolutions.
- [ ] current runtime static constant owner path still exists.
- [ ] Array frozen guard still exists.
- [ ] accepted literal decisions can be amended/superseded.
- [ ] no plan task requires const List.
- [ ] every invariant has a task owner.
- [ ] every material semantic risk has coverage.
- [ ] default flip is after migration.
- [ ] compiler headless fallback is explicitly handled.
- [ ] no broad release test is accidentally required after every task.
- [ ] no duplicate source of semantic truth is planned.
- [ ] walkthrough/handoff/checkpoint obligations are explicit.

If any item is NO because of architectural drift rather than an unfinished
mechanical detail, STOP AND CONSULT.

---

# 28. One-page implementer execution card

Use this as the compact execution reminder.

```text
DO NOT START UNTIL P3 COMPLETE.

0. converge branch; rerun P3 gates
1. add StaticValueData::Sequence
2. integrate type/equality/hash/fingerprint
3. run static-value gate
4. teach StaticEvaluator to consume SequenceLiteralResolution
5. run static-eval gate
6. add self-contained Sequence materialization descriptor
7. materialize Vector / frozen Array with GC-rooted child construction
8. make runtime const sequence reads work through canonical owner path
9. run runtime materialization gate
10. add direct const Array mutation diagnostic; keep frozen alias guard
11. run const-safety gate
12. census every owned bare bracket literal
13. migrate List-required source explicitly
14. amend/supersede old bare-List decisions/specs
15. rerun census; no unclassified/List-dependent implicit sites
16. ONLY NOW flip runtime default -> Array, static default -> Vector
17. make compiler headless missing-lowering fail closed
18. run literal/ambiguity gate
19. run incremental/import/GC hostile matrix
20. run focused stabilization + workspace all-target check
21. update checkpoint/PROGRAM/STATUS
22. write walkthrough + handoff
23. activate P5 only if P4 acceptance is truthful
```

The most important anti-shortcut rule:

```text
Never implement the cutover first and repair static semantics/migration afterward.
```

The default flip is the smallest late-stage operation in P4, not the starting
point.
