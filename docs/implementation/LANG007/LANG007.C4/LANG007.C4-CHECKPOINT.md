---
id: LANG007.C4
category: LANG
program: LANG007
checkpoint: LANG007.C4
kind: checkpoint-record
title: "Structural Static Upgrade"
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
requires:
  - LANG007.C1
  - LANG007.C2
  - LANG007.C3
prepared_against_revision: 8c3a98aab41ed8631e9f8746a6b29984cf77581d
prepared: 2026-10-04
active_plan: LANG007.C4.P4
latest_completed_plan: LANG007.C4.P3
next_plan: LANG007.C4.P4
active_plan_record: docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4-luna-patch-grade-implementation-plan.md
active_plan_prepared_against_revision: 03967716
last_updated: 2026-10-05
---

# LANG007.C4 — Checkpoint Record

## Structural Static Upgrade

## 0. Checkpoint state

```yaml
program: LANG007
checkpoint: LANG007.C4
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
active_plan: LANG007.C4.P4
latest_completed_plan: LANG007.C4.P3
next_plan: LANG007.C4.P4
active_plan_record: docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4-luna-patch-grade-implementation-plan.md
active_plan_prepared_against_revision: 03967716
next_action: >
  P3 is COMPLETE / IMPLEMENTED / FOCUSED_TESTED. P4 remains the primary active plan on local main at 419ec6e66ce90cba6003b7627eb4c99fa2345a7a; Record-specific AST/parser and FieldKey migration edits are in progress and unverified. P10 has completed its pre-S1 safe write-set in the isolated codex/lang007-c4-p10 worktree and has focused evidence for T1-T5, T7-T8, and the non-while T9 harnesses. Commit that work, then wait for a stable P4 frontend revision before synchronizing at S1. Preserve P3 Pack branches and do not start P10 T6 before S1.
```

This record is the single durable state authority for `LANG007.C4`.

Numbered plans describe intended work. Walkthroughs record what actually
shipped. Handoffs record what the next plan must inherit. This checkpoint
record owns the current C4 truth.

---

# 1. Objective and ownership boundary

## 1.1 Bounded acceptance objective

`LANG007.C4` upgrades Telos from C3's closed static/type-domain foundation into
a coherent structural and compile-time semantic system.

The checkpoint must establish:

```text
runtime bindings with explicit immutability/mutability
compile-time constants as a real static phase
first-class Pack structural transport
generalized heterogeneous record FieldKey
compiler-derived Product / OrderedProduct / UnorderedProduct / Sum structure
source-defined structural reflection over canonical semantic data
static control flow using ordinary Telos semantics
recursive generic static computation with bounded deterministic evaluation
normalized semantic residualization
cross-module durable static products and specialization
compiler materialization of static results without re-executing initializers
public ontology migration away from the old TypePack/RecordFields surface
tooling, documentation, incrementality, and performance closure
```

The architectural objective is not merely to add metaprogramming syntax.

The checkpoint must preserve a single semantic authority:

```text
existing Telos type/relation engine
existing Telos pattern/GADT engine
existing Telos row calculus
existing semantic declaration/query identity
existing compiler/runtime product substrate
        ↓
C4 static evaluation and structural reflection
```

C4 must not become a second VM, a second pattern system, a second structural
type system, or a second semantic identity graph.

## 1.2 C4 owns

C4 owns:

- `let` / `let mut` / `const` phase and mutability semantics;
- local definite-initialization proof and runtime fallback;
- immutable-field construction lifecycle;
- explicit stored-field declarations;
- semantic static values;
- bounded deterministic static evaluation;
- normalized semantic residual IR;
- first-class Pack semantics and efficient packing/spreading;
- generalized `FieldKey`;
- Product/Sum structural reflection;
- compiler-derived sealed structural capabilities;
- static structural `for`, `while`, `match`, `if`, `return`, `break`,
  `continue`;
- recursive generic static computation;
- static proof predicates including `Equivalent`, `Subtype`, and `Conforms`;
- durable cross-module static products;
- semantic-to-runtime materialization descriptors;
- public ontology migration;
- C4-specific tooling/docs/performance/incremental certification.

## 1.3 C4 does not own

C4 does not own:

- dynamically changing superclass/method-table semantics;
- a general compile-time VM;
- arbitrary compile-time I/O, filesystem, network, clock, randomness, fibers,
  or mutable runtime globals;
- compile-time execution of arbitrary native/runtime methods;
- a separate pattern resolver;
- a separate subtype/conformance engine;
- a second row solver;
- arbitrary cyclic runtime/static object graphs for reflection;
- public GADT proof-state metadata beyond ordinary structural variant
  reflection;
- general static functions/methods beyond the C4 static-definition model;
- a permanent compatibility layer for the pre-C4 TypePack/RecordFields public
  ontology.

## 1.4 Authoritative specifications and decisions

C4 semantics and structural-static rules are owned by:

- Decision: [TDR-0090](../../../decisions/accepted/0090-structural-static-evaluation-packs-products-sums-and-binding-semantics.md)
- [Bindings and Fields](../../../specs/bindings-and-fields.md)
- [Static Evaluation](../../../specs/static-evaluation.md)
- [Type-Level and Static Domains](../../../specs/type-level-domains.md)
- [Type Members](../../../specs/type-members.md)
- [Structural Products](../../../specs/structural-products.md)
- [Structural Sums](../../../specs/structural-sums.md)
- [Arguments and Parameters](../../../specs/callables/arguments.md)
- [VM Products](../../../specs/vm/products.md)

---

# 2. Plans ledger

The intended C4 decomposition is:

| Plan | Scope | Status | Completion | Verification | Outcome |
|---|---|---|---|---|---|
| `LANG007.C4.P1` | Binding, mutability, field lifecycle, corpus cutover | COMPLETE | IMPLEMENTED | BASELINE_BLOCKED | complete; all P1-owned behavior verified; G1/G2 closed to BASELINE_BLOCKED; all remaining failures classified D; unblocks P2 |
| `LANG007.C4.P2` | Static value model, static definitions, evaluator foundation, closed `const` | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | complete; T0–T11 implemented and verified; G1 end-to-end VM execution (16/16 pass), G2 C1–C3 compatibility (365 pass), G3 affected crates check pass; unblocks P3 |
| `LANG007.C4.P3` | First-class Pack and call/rest convergence | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | complete; T1–T9 implemented and verified; G1 semantic/static vertical, G2 runtime/rest/spread vertical, G3 fixed-call/resource invariants, G4 compatibility/stabilization gates pass; unblocks P4/P5 |
| `LANG007.C4.P4` | Generalized `FieldKey` and unordered-product calculus | IN_PROGRESS | PARTIAL | UNVERIFIED | active in local `main` working tree based at `419ec6e66ce90cba6003b7627eb4c99fa2345a7a`; Record-specific AST/parser, FieldKey/scalar, source-index, Universe, and focused test edits are uncommitted and have no post-edit verification; P4 remains primary `active_plan` |
| `LANG007.C4.P10` | Parallel structural classification, static-control, and convergence foundation | IN_PROGRESS | PARTIAL | UNVERIFIED | active in isolated worktree `codex/lang007-c4-p10` at `/Users/altunhasanli/.codex/worktrees/lang007-c4-p10/phalcom`, starting from `419ec6e66ce90cba6003b7627eb4c99fa2345a7a`; T1-T5 implemented/focused-tested, T7/T8 audit completed, and Pack plus non-loop evaluator T9 harnesses focused-tested; T6 and while-dependent T9 cases wait for S1; inherited fixed-call Pack runtime regression was interrupted after remaining idle for more than 60 seconds, so it is unverified; P4 remains primary `active_plan` |
| `LANG007.C4.P5` | Product/Sum reflection and compiler-derived structural capabilities | PROPOSED | NOT_STARTED | UNVERIFIED | planned |
| `LANG007.C4.P6` | Static control flow, structural iteration, matching, proof predicates | PROPOSED | NOT_STARTED | UNVERIFIED | planned |
| `LANG007.C4.P7` | Recursive generic static computation, residualization, memoization | PROPOSED | NOT_STARTED | UNVERIFIED | planned |
| `LANG007.C4.P8` | Incremental durability, metadata, cross-module static products, materialization | PROPOSED | NOT_STARTED | UNVERIFIED | planned |
| `LANG007.C4.P9` | Public ontology migration, tooling, documentation, performance certification, checkpoint closure | PROPOSED | NOT_STARTED | UNVERIFIED | planned |

Dependency direction:

```text
P1
 │
 ▼
P2
 │
 ├──────────────┐
 ▼              ▼
P3             P4
 │              │
 └──────┬───────┘
        ▼
       P5
        │
        ▼
       P6
        │
        ▼
       P7
        │
        ▼
       P8
        │
        ▼
       P9
```

`P3` and `P4` may proceed in parallel after P2 has established the shared
static-value/result-domain seams.

Do not reorder P5 before both P3 and P4 are semantically stable.

## 2.1 Parallel P4/P10 execution state

P4 remains the checkpoint's primary `active_plan`; P10 is a concurrent active
plan and does not change `latest_completed_plan` or the singleton `next_plan`
metadata. At P10 takeover, local `main` and `origin/main` both pointed to
`419ec6e66ce90cba6003b7627eb4c99fa2345a7a`, while the shared `main` working
tree held uncommitted P4 edits across the AST/parser, FieldKey/scalar,
source-index, Record Universe, and focused test files. P10's separate worktree
was created clean from that committed anchor, and only the named P10 plan was
copied into it. P10 has since implemented and focused-tested T1-T5, completed
the T7/T8 audits, and added focused Pack and non-loop evaluator resource
harnesses for T9. These P10 changes remain isolated until synchronization.

Before P10 T6 edits `Expr` or the parser, S1 requires a coherent P4 frontend
revision to be integrated or explicitly handed off, P10 pre-S1 work to be
committed, the branch to be synchronized, and the post-P4 AST/parser architecture
to be re-audited. P4 remains uncommitted in the main working tree, so S1 is not
yet open. P10's checkpoint changes must be merged semantically with any
concurrent P4 lifecycle updates. The inherited focused fixed-call Pack runtime
test was interrupted after it remained idle for over 60 seconds; it produced no
test result and is not counted as verification evidence.

---

# 3. Ratified checkpoint architecture

The following are accepted C4 design decisions.

They are architectural requirements, not claims that implementation already
exists.

## C4-I-01 — binding phase and runtime mutability are separate

Target surface:

```telos
let x = expr
let mut y = expr
let mut z
const c = staticExpr
```

Meaning:

```text
let
    Runtime + Immutable

let mut
    Runtime + Mutable

const
    Static
```

`let` and `const` require an initializer.

`let mut` may omit one for a simple mutable binding.

## C4-I-02 — uninitialized is storage state, not `None`

```telos
let mut x: Int
```

does not initialize `x` to `None`.

Reading the binding before initialization is invalid.

`None` remains an ordinary Telos value.

## C4-I-03 — local definite initialization is three-state

Canonical semantic states:

```text
Uninitialized
MaybeInitialized
DefinitelyInitialized
```

Read policy:

```text
DefinitelyInitialized
    -> safe; lowering may omit runtime check

Uninitialized
    -> guaranteed-invalid compile-time diagnostic

MaybeInitialized
    -> legal source; runtime checked read
```

Unprovable closure/global cases also use runtime checked reads rather than
forcing a whole-program proof.

## C4-I-04 — fields are immutable by default

```telos
class Counter {
    _name: String
    mut _count: Int
}
```

Bare stored fields are immutable.

`mut` explicitly marks mutable storage.

## C4-I-05 — immutable instance fields use construction lifecycle

An immutable instance field may be established exactly once during the
canonical construction lifecycle.

A declaration initializer counts as prior initialization.

Every successful normal construction path must establish every required
immutable instance field.

For each instance declaration default:

- evaluate it exactly once per object in the declaring owner's canonical
  instance initializer, before that owner's remaining user-authored
  constructor body. For a subclass, the first explicit `super.new(...)` call
  completes first; a root initializer runs defaults at entry;
- preserve declaration order within the owner;
- do not evaluate defaults in raw allocation;
- do not replay inherited defaults in a subclass initializer.

When a parent has an initialization lifecycle, a subclass initializer must
begin with exactly one explicit `super.new(...)` call. It runs the parent
initializer on the same instance, completing parent defaults/body before the
derived owner's defaults/body. Missing the required call is an invalid
construction path; C4 does not insert an implicit superclass chain.

## C4-I-06 — stored fields are explicit only

Assignment never creates field storage.

Reflection/layout/metadata derive from explicit declarations only.

## C4-I-07 — `mut` remains contextual unless a later ratified decision changes it

C4.P1 should reuse the parser's existing contextual `mut` mechanism rather than
reserve `mut` globally merely for `let mut`.

## C4-I-08 — static Telos is semantic execution, not a second VM

Canonical principle:

> Static Telos is ordinary Telos semantic code evaluated by a bounded,
> deterministic semantic evaluator over immutable static values.

It does not invoke arbitrary runtime/native behavior.

## C4-I-09 — static values are semantic, not runtime `Value`

C4 introduces a dedicated semantic static-value model.

It must not use:

```text
phalcom_core::Value
ObjRef
VM heap identity
runtime object identity
```

as its canonical semantic representation.

## C4-I-10 — normalized semantic residual IR

Unresolved open/generic static computation is represented by normalized
semantic residual IR.

Do not serialize raw AST or evaluator frames as the durable residual format.

## C4-I-11 — one Telos pattern language

Static matching reuses the existing resolved pattern semantics, variant
resolution, and GADT proof environment.

Only the minimum Pack/Product structural forms may be added.

No static-only pattern resolver.

## C4-I-12 — proof predicates remain Bool-shaped publicly

`Equivalent`, `Subtype`, and `Conforms` have public static result domain
`Bool`.

Internal outcomes remain richer:

```text
Proven
Refuted
Unknown
Blocked
Dynamic
Cancelled
BudgetExceeded
InternalFailure
```

Reduction rule:

```text
Proven   -> true
Refuted  -> false
Unknown  -> residual
Blocked  -> residual or diagnostic depending forcing context
Dynamic  -> residual where legal
```

Unknown is never coerced to false.

A closed context requiring a concrete result diagnoses unresolved proof.

## C4-I-13 — ordinary structural `Variant` metadata only

C4 public `Variant` reflection exposes structural variant information.

Existing GADT equality/refinement proof state remains matcher-internal.

Public GADT proof metadata may be designed later.

## C4-I-14 — structural capabilities are compiler-derived and sealed

`Product`, `OrderedProduct`, `UnorderedProduct`, and `Sum` are not forgeable
ordinary user traits.

Compiler-known structure is the source of truth.

## C4-I-15 — static `while` is an explicit semantic control-flow construct

Static evaluation must understand Telos `while` directly.

No method, selector, protocol, or intrinsic named `whileTrue` participates in
static-`while` semantics.

User-defined `whileTrue` remains an ordinary method.

## C4-I-16 — static materialization uses a semantic descriptor

Static initializer:

```text
source
  -> semantic static evaluation
  -> StaticValue
  -> MaterializationDescriptor
  -> compiler/runtime representation
```

The initializer is not rerun at runtime.

## C4-I-17 — cross-module static definitions export checked semantic meaning

Cross-module symbolic static definitions export:

- stable definition identity;
- checked semantic body/residual semantics;
- parameter/result domains;
- stable dependency fingerprints;
- required proof/capability environment.

Do not export evaluator continuations or raw AST as the semantic contract.

## C4-I-18 — immediate public ontology cutover

C4 uses a hard migration away from the public legacy:

```text
TypePack
LabeledTypePack
CompleteTypePack
RecordFields
```

where first-class Pack/Product concepts replace them.

Optimized internal C1–C3 representations may remain.

## C4-I-19 — Pack equality is structural semantic equality

Pack equality/hash includes:

- lane/shape identity;
- length;
- positional order;
- labels/order where structurally significant;
- recursively equal values.

Float follows Telos's global equality/hash semantics.

## C4-I-20 — Pack is specialized structural transport

Pack is a first-class Telos value optimized for:

- construction;
- variadic capture;
- spreading;
- lane decomposition;
- structural static iteration/reflection.

Implementation invariants:

```text
flat backing storage
O(1) length
O(1) rest/slice
O(1) first/rest view construction
O(n) equality/hash
amortized O(1) builder append
O(n) finalize/materialize
O(total args) fused multi-spread
no required Pack allocation for fixed calls
no recursive cons representation
```

## C4-I-21 — Pack views are one-step views

Source forms such as:

```telos
PositionalPackView::Entry(value, rest)
```

must not imply recursively materialized cons lists.

`rest` is an O(1) Pack slice/view over flat storage.

## C4-I-22 — reflected structure is shallow with semantic references

Product/Variant/Field reflection describes immediate structure.

Contained recursive types remain canonical `Type`/semantic references.

Reflection does not recursively expand contained type shapes.

## C4-I-23 — reduced structural static values are acyclic DAGs

Recursive semantic entities are represented by stable semantic references.

C4 does not require arbitrary cyclic `StaticValue` containment graphs.

## C4-I-24 — structural classification is below capability/reflection

Avoid query cycles:

```text
canonical declaration/type information
        ↓
structural classification
        ├─ sealed capability derivation
        └─ reflection shape derivation
```

Capability conformance and reflection must not recursively prove one another.

## C4-I-25 — structural equality/hash do not expand reflection

A `Type` inside a Pack/Product value uses canonical semantic type
equality/hash.

It does not compare/hash by recursively reflecting `Type::Shape`.

## C4-I-26 — recursion uses explicit evaluator frames

Recursive static computation does not use unbounded host-language recursion.

Exact repeated application keys are recursive cycles.

Changing-argument structural recursion remains legal until base case or budget.

## C4-I-27 — evaluator budgets/cancellation reuse existing query infrastructure

C4 does not invent hidden recursion/loop caps separate from existing semantic
budget/cancellation authority.

A static-value/residual growth budget may be added where required.

## C4-I-28 — uncertainty/cancellation is never cached as semantic success

Cancelled, blocked, budget-exceeded, or otherwise incomplete evaluation is not
memoized as a successful semantic reduction.

## C4-I-29 — field-key canonical order is language-defined

Target:

```telos
type FieldKey = Symbol | String | Int | Bool
```

Canonical ordering is a language semantic rule, not intern/discriminant order.

Ratified direction:

```text
Symbol < String < Int < Bool
```

Within kind:

```text
Symbol  canonical content/spelling
String  content
Int     numeric
Bool    false < true
```

## C4-I-30 — heterogeneous static structural loops specialize per iteration

A structural loop over heterogeneous entries refines the binder separately for
each static iteration.

It is not modeled as a lossy union binder when exact per-entry structure is
available.

## C4-I-31 — static String semantics are content semantics

Static String equality/hash is content/value based.

Compiler arena/intern identity is never observable Telos semantics.

## C4-I-32 — closed `const` must reduce

A closed static constant must produce a reduced static value.

A symbolic result remaining in a closed `const` is a compile-time failure.

---

# 4. Stable takeover map from C1–C3

C4 inherits and must preserve the following current authorities.

| Concept | Current authority | C4 use | Forbidden replacement |
|---|---|---|---|
| canonical type/static terms | `TypeId` / `ProperTypeId` / C3 static-domain machinery | static type values and proof inputs | second type-term universe |
| pack type terms | C1/C2 `TypeData::TypePack` family and pack substitution | optimized internal representation / bridge | mandatory public legacy ontology |
| rows | Record row IDs/solver/relation | generalized FieldKey migration | static-only row solver |
| pattern matching | `MatchResolution`, `PatternResolution`, variant candidate resolution | static match | second resolver |
| GADT branch proofs | `BranchProofEnvironment` | static match/refinement | public proof reimplementation |
| variant identity | `VariantId`, `VariantFamilyId`, enum/variant info | Sum reflection | name-only variant lookup |
| relation outcomes | rich relation/proof outcome enums | Equivalent/Subtype/Conforms | Bool-only internal engine |
| query budget/cancel | existing semantic query budget/token | static evaluator bound | hidden evaluator caps |
| incremental query DB | typed query keys/dependencies/fingerprints | static definitions/results | parallel static cache |
| field lifecycle | existing semantic field lifecycle table | P1 strengthening | compiler-only field proof |
| product runtime substrate | `phalcom-core::product` | Pack/Product materialization | generic DataObject-only storage |
| argument builder | private runtime/compiler pack builder | Pack/call convergence | public builder object |
| metadata durability | schema-v10 baseline from C3 | C4 schema evolution | raw session IDs |

Mechanical names may drift.

Architectural ownership may not silently move.

---

# 5. Planned stable C4 interfaces

The following are intended seams for later plans.

They are not implementation-complete until their owning plan walkthrough says
so.

## 5.1 P1 expected seams

```text
BindingPhase
RuntimeMutability
InitializationState
BindingReadSafety
runtime UninitializedBinding representation
checked / unchecked binding reads
explicit module binding category
explicit stored-field layout
immutable-field lifecycle enforcement
```

## 5.2 P2 expected seams

Established in P2 (T1–T4):

```text
StaticDomainSyntax, StaticValueDomain::{Kind, ValueType}
resolve_static_domain, validate_proper_type
TypeDefinitionDef, StaticParameterSyntax, StaticDefinitionBody
GenericArgumentSyntax::Value
StaticParameter, StaticDefinitionSignature, StaticDefinitionBodyRepr, StaticDefinition
StaticDefinitionProduct, StaticDefinitionTable, resolve_static_definition
QueryKey::StaticDefinition(DeclarationId), SemanticProduct::StaticDefinition
SemanticSnapshot::static_definitions, SemanticSnapshot::static_definition
StaticValueStoreId, StaticValueId, StaticValueRef, StaticValueStore
StaticValueData::{Unit, Bool, Int, Float, String, Symbol, Type, StaticTerm}
StaticInt, StaticFloat, StaticStringId, StaticSymbolId
StaticValueStore::semantic_eq, semantic_hash, content_fingerprint
SemanticSnapshot::static_values, SemanticWorkspaceSession::static_values
```

T5 adds:

```text
StaticValueData::{Tuple, Record, Data, Variant}
StaticTupleId, StaticRecordId, StaticDataValueId, StaticVariantValueId
StaticValueStore::{intern_tuple, intern_record, intern_data, intern_variant}
StaticValueStore::{static_value_type, check_value_type}
```

T6 now provides checked bodies, module binding handoff, separate body query ownership, and exact callable allowlisting:

```text
StaticIntrinsicOperation::NumberAdd
StaticIntrinsicRegistry::from_canonical_core / operation_for / resolve_ready_expressions
StaticOperationResolution / StaticOperationResolutionIndex
ModuleStaticBindingIndex / CheckedStaticBody / StaticBodyId
QueryKey::ModuleStaticBindings / QueryKey::CheckedStaticBody
```

Conceptually:

```rust
StaticValueId
StaticValueData
StaticEvalOutcome
StaticDefinition
StaticEvalContext
MaterializationDescriptor
```

T6 verification completed:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib checker::static_body::tests -- --nocapture
    3 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_ -- --nocapture
    19 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::static_definition_bodies -- --nocapture
    1 passed
```

The focused session test confirms checked const reads retain canonical
`SymbolId`, value-result bare-name resolution uses the checked lexical binding,
and `Number.+(_)` is projected only from the selected ordinary callable.
Static-body unit tests also cover runtime outer reads and writes as phase
errors. Broader query invalidation/import-alias coverage remains for T10/G2.
One expected crate warning family remains: deprecated `Statement::TypeAlias`
compatibility references.

Outcome direction:

```text
Reduced
Symbolic
Invalid
Recursive
BudgetExceeded
Cancelled
```

## 5.3 P3 expected seams

Planning is now patch-grade and repository-grounded. These remain intended interfaces until a P3 walkthrough records what actually shipped:

```text
proper first-class Pack Type distinct from C3 kinded TypePack terms
PackFromPacks / equivalent canonical bridge to existing C1–C3 pack algebra
PositionalPack / LabeledPack / CompletePack
PositionalPackView / LabeledPackView with Empty / Entry
StaticValue::Pack in the existing P2 StaticValueStore
StaticPack immutable backing + O(1) slice descriptors
StaticParameter canonical GenericParameterMode for value-domain variadics
runtime EmptyPack + immutable Pack root/slice representation
GC-safe root-referenced O(1) runtime slices
private ArgumentPackBuilder -> FinishPack only when a surface Pack is required
fixed calls remain direct and require no Pack/builder allocation
rest capture -> Pack across method/block/Fiber language-rest entry paths
Pack spread/forwarding -> direct lane append into one fused destination builder
explicit Tuple <-> Pack interoperability; no Tuple/Pack identity collapse
minimal stable ordinary Pack type metadata; StaticPack value persistence remains P8
```

## 5.4 P4 expected seams

```text
FieldKey
generalized row field keys
canonical FieldKey ordering
generalized record runtime shape/index
generalized unordered-product lookup
```

## 5.5 P5 expected seams

```text
Product
OrderedProduct
UnorderedProduct
Sum
ProductShape
OrderedShape
UnorderedShape
Variant
Variants = PositionalPack<Variant>
compiler-derived sealed structural evidence
```

## 5.6 P6 expected seams

```text
static semantic If / Match / For / While
EvalControl:
    Value
    Return
    Break
    Continue

Equivalent
Subtype
Conforms
```

Static `for` consumes structural semantics, not the runtime Iterator protocol.

## 5.7 P7 expected seams

```text
StaticResidualId
normalized semantic residual nodes
exact application-key cycle detection
static memo table
explicit evaluator frames
specialization of residuals
```

## 5.8 P8 expected seams

```text
static-definition query products
stable StaticValue/Residual metadata DAG
cross-module checked static definitions
const value dependency fingerprints
materialization descriptors consumed by compiler
```

## 5.9 P9 expected seams

```text
public Pack/Product ontology only
source-defined reflection helpers
tooling/docs/spec convergence
performance certification
checkpoint closure
```

---

# 6. Verification ledger

| Gate | Scope | Evidence | Result | What it proves |
|---|---|---|---|---|
| C4 planning baseline | repository + C3 takeover + C4 architecture | C4 requirements analysis and P1 repository audit | COMPLETE | planning is grounded; no implementation claim |
| C4.P1 plan readiness | P1 execution design | Luna patch-grade plan prepared against audited revision | COMPLETE | P1 has bounded execution plan; no implementation claim |
| C4.P1.T0 baseline control | audited revision 8c3a98aab41ed8631e9f8746a6b29984cf77581d | `cargo test -p phalcom-ast` (294 pass); focused semantic & modules suites run | BASELINE_RECORDED | starting revision frozen; pre-existing test failures cataloged |
| `LANG007.C4.P1.T7/T8/T10` focused continuation | explicit field layout, field lifecycle/default products, binding flow/runtime representation | semantic capability tests; core `field_layout`, `field_lifecycle`, `binding_init`, `migrated_layout_matches_pre_cutover_oracle`; incremental product tests | FOCUSED PASS | proves selected local semantics, codegen properties, and field layout parity across sides, order, mutability, counts, and superclass prefixes |
| `LANG007.C4.P1.G1` | semantic bindings/fields plus core corpus | semantic selectors pass; `language-corpus fields` passes 1; `language-corpus classes` and `classes_negative` pass; `language-corpus bindings` fails on `destructure_list_no_rest.ph` (classified D) | COMPLETE / BASELINE_BLOCKED | P1 field and class corpus lanes pass; the bindings corpus failure is classified D and remains outside P1 |
| `LANG007.C4.P1.G2` | six affected crates | AST and LSP pass; modules fails 1 catalog test (proven D); semantic reports 1,765 pass / 37 fail / 42 ignored (all 37 proven D); core P1-owned library test failures repaired and pass, 3 library baseline failures classified D, broader core test suite deferred per user instruction; REPL fails 1 test (proven D). All P1-owned failures are green. | COMPLETE / BASELINE_BLOCKED | broad stabilization complete to BASELINE_BLOCKED; all P1-owned behavior verified and green; residual failures classified D |
| `LANG007.C4.P2.T1` | `StaticValueDomain` + `resolve_static_domain` + `validate_proper_type` | `static_domains` 6/6 passed; `cargo check -p phalcom-semantic` clean | FOCUSED PASS | domain model is correct; Kind/ValueType distinction holds |
| `LANG007.C4.P2.T2` | generalized AST `TypeDefinitionDef` + parser | 288/288 AST tests passed | FOCUSED PASS | grammar and downstream consumers compile cleanly; no regressions |
| `LANG007.C4.P2.T3` | `StaticDefinition`, `StaticDefinitionTable`, `resolve_static_definition` | `static_definitions` 5/5; `type_annotations` 61/61; `incremental::db` 14/14; `check -p phalcom-semantic` and `check -p phalcom-core` clean | FOCUSED PASS | StaticDefinition is canonical product; TypeAliasInfo projection preserved; value params excluded from TypeStore |
| `LANG007.C4.P2.T4` | snapshot-safe primitive `StaticValueStore` and TypeStore bridge | `foundations::static_values` selected 1 test and passed, covering exact Int, Float representation/equality/hash, UTF-8 String, Symbol spelling, Type/StaticTerm admission, clone lineage, append stability, and foreign-store rejection | FOCUSED PASS | Primitive static values are semantic-only, append-stable, store-qualified, and retain signed-zero/NaN representation distinctions |
| `LANG007.C4.P2.T5` | immutable structural Tuple/Record/Data/Variant values, type derivation, and value assignability | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values_structural -- --nocapture` selected 1 test and passed | FOCUSED PASS | ordered tuple equality, order-independent P2 string-key record construction, shared DAGs, canonical data-component mapping, exact VariantId identity, rejection of wrong payload arity/foreign refs, ordinary type projection, and relation-engine mismatch are covered |
| `LANG007.C4.P2.T6 foundation slices` | canonical `Number.+(_)` identity allowlist, ready-expression projection, and exact local-binding reads | focused integration test `foundations::static_intrinsics` passed 1/1; focused unit test `static_intrinsic::tests::ready_analysis_projection_keeps_exact_callable_and_skips_invalid_sites` passed 1/1; `foundations::binding_init::read_after_assignment_is_proven` passed 1/1; incremental fingerprint regression passed 1/1; `cargo check -p phalcom-semantic` completed | FOCUSED PASS (slices only) | callable authorization retains exact identity; variable-read products retain the exact selected `BindingId`, included in callable body fingerprints |
| `LANG007.C4.P2.T6 checked-body analyzer` | ordinary checker-backed static body facts, result checking, runtime outer read/write phase errors, evaluator slots | `cargo test -p phalcom-semantic --lib checker::static_body::tests -- --nocapture` selected and passed 4; exact Number addition authorization in `foundations::static_` | FOCUSED PASS | phase access and exact per-expression operation facts come from ordinary checker identity |
| `LANG007.C4.P2.T6 module binding/session composition` | stable module const identity, linked import aliases, fresh body-local identities, snapshot composition | `cargo test -p phalcom-semantic --test semantic integration::static_definition_bodies -- --nocapture` selected and passed 3 | FOCUSED PASS | same-module and imported const reads use canonical SymbolId; checked bodies are visible in final definitions |
| `LANG007.C4.P2.T6 checked-body invalidation` | separate checked-body query, module input/product fingerprints, dependency edges and session reuse | integration revision changes provider const from Int to String; checked binding and checked-body input fingerprints changed, and the checked-body Arc was replaced | FOCUSED PASS | a changed provider binding product invalidates/recomputes the importing checked body |
| `LANG007.C4.P2.T7 evaluator foundation slice` | explicit work queue, ordered blocks, binding-ID local slots, mixed kinded/value argument layout, exact Number addition, selected if branches, lazy and/or, mutable locals, Symbol/Tuple/Record values, query budget, and cancellation | `cargo check -p phalcom-semantic` completed; `cargo test -p phalcom-semantic --lib checker::static_body::tests -- --nocapture` passed 4; `cargo test -p phalcom-semantic --test semantic integration::static_definition_bodies -- --nocapture` passed 3 | FOCUSED PASS | checker-authorized 1 + 2 reduces to 3; selected branches reduce; untaken unsupported comparisons are skipped while the taken one fails closed; mutable slot assignment updates the later checked read; Symbol literals, positional Tuple, and bare-label Record values construct; argument count/kind mismatches, budget exhaustion, and cancellation terminate evaluation |
| `LANG007.C4.P2.T8 static constant pipeline` | `StaticConstantId(SymbolId)`, exact initializer syntax site, static body check, SCC cycle detection, topological evaluation, content-recursive fingerprinting | `cargo test -p phalcom-semantic --test semantic foundations::static_constants` passed 9/9 (literal, computed, const-to-const dep, same-module isolation discriminator, transitive invalidation, self cycle, mutual cycle, runtime dep rejection, eval controls and LKG) | FOCUSED PASS | constant-owned identity avoids full module recomputation; editing constant A does not invalidate unaffected constant B in same module; cyclic peer evaluation safely halts with `StaticConstantCycle` without stack overflow; content fingerprint propagates transitive updates |
| `LANG007.C4.P2.T9 materialization vertical` | `MaterializationDescriptor`, `MaterializationCapability`, pure descriptor projection, M2 primitive VM materialization (Unit, Bool, Small/Large Int, Float exact bits, String, Symbol), deferred structural const handling | `cargo test -p phalcom-core --test core execution_static_const` passed 9/9; `cargo test -p phalcom-core --test core compiler_static_const` passed 7/7 | FOCUSED PASS | compiler emits direct constant-pool loads for primitive static constants without runtime initializer bytecode or slots; structural constants fail closed at runtime with `StaticConstStructuralDeferred`; LargeInt in chunk constants survives GC |
| `LANG007.C4.P2.G1 VM execution gate` | End-to-end VM execution gate for static constants | `cargo test -p phalcom-core --test core static_const` passed 16/16 | COMPLETE / FOCUSED_PASS | all execution and compiler static constant scenarios compile and run correctly on the VM |
| `LANG007.C4.P2.T10 incremental stabilization` | Incremental cold/warm parity, same-module constant isolation, acyclic SCC ordering, budget termination | `cargo test -p phalcom-semantic --test semantic static_constants` passed 9/9 | FOCUSED PASS | 100% cache hit rate on unchanged constants; topological invalidation propagates; cyclic queries terminate safely |
| `LANG007.C4.P2.G2 C1–C3 compatibility gate` | Regression verification for C1–C3 type alias, static definition, and algebraic data suites | `cargo test -p phalcom-semantic --test semantic type_alias` (2 pass); `cargo test -p phalcom-semantic --test semantic adts` (318 pass, 28 ignored, 0 fail); `cargo test -p phalcom-core --test core language::algebraic_data` (45 pass, 19 ignored, 0 fail) | COMPLETE / FOCUSED_PASS | zero regressions in C1–C3 static, type, and algebraic data behavior |
| `LANG007.C4.P2.G3 affected crates check` | Affected crate compilation check | `cargo check -p phalcom-semantic -p phalcom-core` completed with 0 errors | COMPLETE | all affected crates build cleanly |
| `LANG007.C4.P3` plan readiness | P3 repository grounding, architecture, implementation decomposition, coverage, stop conditions, and lifecycle contract | read-only live audit of `main` at `5a5bf8b218c2fbcfd785bfa41bb4eb7fec21c521` + `LANG007.C4.P3-requirements-analysis.md` + `LANG007.C4.P3-luna-patch-grade-implementation-plan.md` | COMPLETE (planning only) | P3 has a bounded executable patch-grade plan grounded in current source; proves no P3 implementation behavior and does not change P3 verification |

### 2026-10-04 — P2 T7.10 exact static-application authority stop

The active plan requires applications to consume an already-resolved target
signature and bind arguments by identity/order. Live `Expr` has no static
application node. `TypeAnnotationExpr::Application` resolves through
`resolve_type_form` to a canonical `TypeId`, while `ExpressionAnalysis` and
`CheckedStaticBody` publish no exact static-definition application product.
`StaticDefinitionProduct` carries the definition/signature and optional C3
`TypeAliasInfo`, but its alias projection is limited to Type-valued,
all-kinded definitions. The evaluator currently receives only one checked body,
its operation index, and arguments; it cannot push a nested exact target frame.

**Trigger:** shared-consultation protocol RED trigger 1/2/7/14 — the planned
application architecture is underspecified against live products, and adding
source-name resolution in the evaluator would create a second semantic
authority. No workaround or speculative architecture was implemented.

### Decision

The user selected the checked-product route: T7.10 adds an ordinary-checker
static-application product carrying exact target declaration/signature/domain
and argument identities. The evaluator must consume it without name lookup.
Value-parameter applications remain within T7 scope. Existing C3
`TypeAliasInfo` remains the optimized projection for compatible Type-valued
definitions.

### Plan amendment

T7.10 now includes an ordinary-checker publication seam for exact nested
static-application products. The product must be derived alongside TypeForm
resolution to ensure it agrees with the canonical TypeForm result. T7 tests
must cover target identity, mixed arguments, wrong domains, nested application,
and recursion outcomes. No static-definition identity or alias authority
changes. P2 remains `IN_PROGRESS / PARTIAL / UNVERIFIED` while the seam is
implemented and verified.

Future plan gates must be added only after actual execution.

### Follow-up — checked product seam and snapshot publication blocker

The worktree now contains `CheckedStaticApplication` on `CheckedStaticBody`,
linked from `SemanticDenotation::StaticApplication`. `TypeResolver` exposes
the product by resolved declaration identity; checked-body analysis records a
formal `SemanticDependency::StaticDefinition`, and the DB query has a replay
path for it. This does not yet implement evaluator application frames.

The focused regression added `type Literal: Int = 3` and
`type Applied: Int = Literal<>` to the established
`integration::static_definition_bodies` fixture. That addition makes the
existing `ReadBase` snapshot lookup fail: the published static-definition
table is empty. Removing the application restored the prior three-test pass.
A standalone fixture also yielded an empty table without an application, so
it is not a useful discriminator. No fallback source-name lookup was added.

### Resolution — checked-body dependency replay

The focused architectural consultation confirmed that the generic
`TrackingTypeResolver::resolve_type_name` added a `DeclarationShell` edge for
value-domain static definitions. Replay accepts declaration/alias products,
but these definitions publish neither. That invalid edge blocked checked-body
publication and left the prior empty snapshot. The localized
`resolve_static_definition_name` path preserves `LinkedName` and
`PublicExport`, resolves through canonical declaration identity, and records
only the `StaticDefinition` edge. It does not weaken query replay or add a
second resolver.

The focused `integration::static_definition_bodies` selector passed 3/3 after
the correction and again after evaluator coverage was added for zero-argument,
value-parameter, direct kinded, mixed kinded/value, nested nonrecursive, and
self-reentry applications. The mixed-parameter case was added in the latest
slice. T7 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`; the current next step
is to complete kinded-expression application and remaining resource-boundary
acceptance. Do not advance to T8 yet.

### Adviser decision — T7.11 canonical constructor authority

The ordinary checker owns Data/Variant constructor selection and payload
mapping. `CheckedStaticBody` should retain the body-local associated-resolution
facts and expose a compact checked-construction projection containing exact
target identity, result type, argument expression IDs, and stable component or
variant-field IDs. Data's current argument-index mapping must be converted to
`DataComponentId`s while canonical `DataInfo` is available. If accepted
Variant syntax permits reordered/labeled payloads, the checker must publish
exact `VariantFieldId` mapping; the evaluator must not infer it from source
labels or assume source order.

Canonical `DataInfo`/`VariantInfo` remain owned by snapshot semantic tables.
Pass borrowed canonical tables and the canonical `TypeStore` into evaluation;
do not copy catalogs into each body. The evaluator only consumes checked
construction facts and invokes T5's pure interners. Required discriminators:
reordered labeled Data payloads and exact Variant identity resistant to a
same-named case elsewhere. This decision is now the T7.11 implementation
contract.

### T7.11 focused implementation gate

Implemented the body-local checked construction product with exact target,
result type, argument expression IDs, and stable DataComponentId/VariantFieldId
payloads. `CheckedStaticBody` retains associated-resolution facts for its own
expressions. The evaluator receives borrowed snapshot TypeStore/Data/Enum
tables and uses the existing pure T5 Data/Variant interners.

The focused `integration::static_definition_bodies` selector passed 3/3. Its
reordered `Point { y: 2, x: 1 }` assertion matches canonical component
identity/order, and `LeftChoice::Same(3, 4)` differs from the same-named case
under `RightChoice`. The gate is focused evidence for T7.11 only. T7 remains
`IN_PROGRESS / PARTIAL / UNVERIFIED`; kinded-expression applications and
remaining budget/value-node boundaries remain open.

### T7.12 focused trace and cycle/resource gate

The evaluator exposes definition, exact application, and source-expression
trace frames. Trace growth uses the existing QueryBudget and checks
cancellation; there is no independent trace-depth ceiling. The focused
`integration::static_definition_bodies` selector passed 3/3 with mutual
A→B→A reentry, application step-budget exhaustion, and trace-path assertions.
The evaluator unit test continues to distinguish pre-cancelled execution.
`Recursive`, `BudgetExceeded`, and `Cancelled` remain distinct. T7 is still
`IN_PROGRESS / PARTIAL / UNVERIFIED`; kinded-expression application and
remaining resource/value-node acceptance remain open.

### Adviser decision — T7.10 kinded-expression result boundary

The checked product cannot generally project a kinded static application to a
`TypeId`: its result may depend on evaluated arguments or body control flow.
Evaluation therefore needs a tagged result across body-completion and
application-frame boundaries, with `StaticValue(StaticValueRef)` and
`KindedTerm(TypeId)` cases. The ordinary expression stack remains
`StaticValueRef`-only, and a kinded result may bind only a checked parameter
whose signature expects a kinded term. The existing direct checked-`TypeId`
argument path stays unchanged. T7 scope is limited to the checked
TypeForm/static-application path; unsupported TypeForm computation must fail
closed. No evaluator-side resolution or type synthesis is authorized.

The live evaluator borrows the canonical snapshot TypeStore immutably, while
`TypeView::materialize` requires mutable access. No snapshot-safe read-only
substitution API exists. Therefore a TypeForm result is supported only when
its checker-published canonical `TypeId` is independent of the callee's
kinded parameters, or is exactly one of those parameter nodes, in which case
the evaluator forwards the supplied canonical `TypeId` unchanged. Composite
results containing any callee kinded parameter fail closed. Other already
canonical results are admitted only when their kind matches the checked
result domain. Do not clone/mutate a parallel TypeStore or publish a
clone-qualified `TypeId`.

This is the T7.10 plan amendment. The focused
`integration::static_definition_bodies` selector passed 3/3 after adding an
independent TypeForm result, exact kinded-parameter forwarding with TypeId
identity preservation, nested applications through both paths, composite
parameter-dependent rejection, and a mismatched result-domain rejection.
Data constructor payload traversal now charges the step budget before
interning; a bounded regression confirms exhaustion leaves the Data value node
absent. `QueryBudget` has no separate value-node quota, so the plan's optional
value-node-limit case is not enabled. T7 focused acceptance is complete. The final selector passed 3/3 after adding
false/nested `if`, nested arithmetic, source-ordered writes, uninitialized
local read, and immutable reassignment rejection. Cancellation remains covered
by the pre-cancelled unit case; QueryBudget has no separate value-node quota.
The Data-construction budget regression confirms exhaustion occurs before the
Data value node is interned. T8 is now the next P2 task; P2 itself remains
`IN_PROGRESS / PARTIAL / UNVERIFIED`.

### 2026-10-04 — G2 stabilization closure

The required package audits and classification against clean predecessor `8c3a98aab41ed8631e9f8746a6b29984cf77581d` were completed:

- `phalcom-ast`: PASS. The library suite passed 63 tests; the migration tool
  passed 5; integration passed 282 with 1 ignored; trait syntax passed 12.
- `phalcom-modules`: FAIL (Category D). `bcat_08_root_alias_does_not_create_synthetic_declaration`
  cannot link `universe.collections.iterator_laws` because the test's partial
  interface map omits that dependency. This is a pre-existing builtin-catalog baseline failure.
- `phalcom-semantic`: FAIL (Category D only). All 27 Category B stale fixtures were
  migrated to accepted P1 immutable `let`, mutable field, or uninitialized `let mut` surface.
  Full package re-run certified: 1,765 passed, 37 failed, 42 ignored (in 591.72s).
  Every single one of the 37 residual failures was tested against clean predecessor
  `8c3a98aab41ed8631e9f8746a6b29984cf77581d` and confirmed to be a pre-existing Category D baseline failure.
- `phalcom-core`: FAIL (Category D only / partial run).
  - **Library target (`--lib`)**: 151 passed, 3 failed. P1-owned library failures `ic_add_method_invalidates`
    and `ic_override_after_caching` (stale `const` runtime storage migrated to `let`) and
    `canonical_range_lower_option_closes_number_at_is_none_call_edge` (migrated `lowerOpt` corpus
    search string) were repaired and pass in focused reruns. Three residual library baseline failures
    (`kernel_option_match_override_reroutes_every_combinator`, `captured_setter_rejects_an_ordinary_positional_rhs_lane`,
    `walk_orders_oldest_first_with_selector_shaped_names`) are classified Category D baselines.
  - **Integration tests (`--test core`)**: `field_layout` passed 4, 1 ignored (GATED `adt_lower_08_imported_field_layout_uses_declared_physical_slot`);
    `field_lifecycle` passed 2; `binding_init` passed 13; `compiler_installs_and_executes_canonical_constructor_projections` passed 1.
  - **Language corpus (`--test language-corpus`)**: Suite contains 63 tests. Execution was taking too long
    (individual suites take 60s to hundreds of seconds, e.g. classes took 793s), so the run was prematurely interrupted
    per user instruction. Partial results before interruption:
    - Passed: `corpus::absence_negative`, `corpus::classes_negative`, `corpus::fields` (1 passed in focused G1 run), `corpus::classes` (1 passed in focused G1 run, 793.41s).
    - Failed: `corpus::bytes_negative`, `corpus::blocks`, `corpus::bytes`, `corpus::collections_literals_negative`, `corpus::collections`.
    - Still in progress (>60s) when interrupted: `corpus::absence`, `corpus::arithmetic`, `corpus::bindings` (contains known Category D failure `destructure_list_no_rest.ph`), `corpus::booleans`, `corpus::classes`, `corpus::compile_errors`, `corpus::concurrency`, `corpus::concurrency_negative`.
  - Full execution and triage of the broader core test suite and remaining corpus lanes are deferred to subsequent checkpoint work per explicit user instruction ("Ignore core tests. We'll deal with it later — write into lifecycle records; language_corpus was taking too long, so it was prematurely interrupted").
- `phalcom-lsp`: PASS after the field-completion fixture was given an explicit
  `Client` field type; all package targets passed.
- `phalcom-repl`: FAIL (Category D). 28 tests passed and
  `repl_mi_06_all_reflection_children_importable` failed with `modifier.illegal_target`.
  Exact selector tested on predecessor `8c3a98aab41ed8631e9f8746a6b29984cf77581d` reproduced
  the exact same failure -> proven Category D baseline.

The G2 result is `COMPLETE / BASELINE_BLOCKED`. All P1-owned behavior is verified and green.
P1 is complete and unblocks P2.

`cargo fmt --all -- --check` also fails on formatting differences across many
existing repository files, including unmodified areas of touched files. No
repository-wide formatting was applied; `git diff --check` passes.

Do not record designed coverage as executed verification.

---

# 7. Deferred / baseline issue ledger

### Observed Baseline Test Failures (Revision 8c3a98aab41ed8631e9f8746a6b29984cf77581d)

1. **`semantic::capabilities::fields::field_facts_survive_constructor_and_general_writes`**
   - **Classification**: D (baseline / unrelated to current change)
   - **Observed**: Assertion `left == right` failed for binding `counter` (left: `None`, right: `Some(TypeId(779))`) in multi-class analysis fixture.
   - **Why non-blocking**: 17 other field capability tests pass; this test expects multi-class constructor type propagation in a single-file test harness.
   - **Action**: Do not fix during unrelated tasks; re-evaluate when field lifecycle tests are updated.

1. **`bcat_08_root_alias_does_not_create_synthetic_declaration` (phalcom-modules)**
   - **Classification**: D (baseline / unrelated)
   - **Observed**: `UnresolvedImport { module: ModuleId { project: Universe, path: collections.list }, path: "universe.collections.iterator_laws" }`.
   - **Why non-blocking**: Unit catalog link failure in Universe collections path, unrelated to binding/field AST.
   - **Action**: Record and leave undisturbed.

2. **`repl_mi_06_all_reflection_children_importable` (phalcom-repl)**
   - **Classification**: D (baseline / unrelated)
   - **Observed**: `modifier.illegal_target: top-level declarations cannot have visibility modifiers; use module exports (e.g. 'export')` while importing reflection child `module`.
   - **Why non-blocking**: Pre-existing repl import failure proven identical on clean starting revision `8c3a98aab41ed8631e9f8746a6b29984cf77581d`.
   - **Action**: Record and leave undisturbed.

3. **`bindings/destructure_list_no_rest.ph` (phalcom-core `language-corpus bindings`)**
   - **Classification**: D (baseline / unrelated)
   - **Observed**: Runtime attempt to invoke `unwrapOr(_)` on integer `1` in pattern lowering (`emit_element_read`).
   - **Why non-blocking**: Pre-existing pattern destructuring lowering defect, outside P1 binding/field lifecycle scope.
   - **Action**: Record and leave undisturbed.

4. **`phalcom-core` Library and Corpus Test Status**
   - **Classification**: D (baseline / unrelated)
   - **Observed Library Target (`--lib`)**: Three pre-existing library target failures:
     - `kernel_option_match_override_reroutes_every_combinator`: pre-existing generic call-entry failure recorded in C3 closure.
     - `captured_setter_rejects_an_ordinary_positional_rhs_lane`: pre-existing regression in captured-method shape validation.
     - `walk_orders_oldest_first_with_selector_shaped_names`: pre-existing order-dependent/flaky traceback handling test.
     - Note: P1-owned library failures `ic_add_method_invalidates`, `ic_override_after_caching`, and `canonical_range_lower_option_closes_number_at_is_none_call_edge` were repaired and pass.
   - **Integration Tests (`--test core`)**: `field_layout` (4 passed, 1 ignored GATED), `field_lifecycle` (2 passed), `binding_init` (13 passed), `compiler_installs_and_executes_canonical_constructor_projections` (1 passed).
   - **Partial Language Corpus (`--test language-corpus`)**: Suite contains 63 tests. Execution was taking too long (each lane takes 60s to hundreds of seconds, e.g. classes took 793s), so it was prematurely interrupted per user instruction.
     - Passed before interruption: `corpus::absence_negative`, `corpus::classes_negative`, `corpus::fields` (focused G1 pass), `corpus::classes` (focused G1 pass, 793.41s).
     - Failed before interruption: `corpus::bytes_negative`, `corpus::blocks`, `corpus::bytes`, `corpus::collections_literals_negative`, `corpus::collections`.
     - In progress (>60s) when interrupted: `corpus::absence`, `corpus::arithmetic`, `corpus::bindings` (contains known Category D failure `destructure_list_no_rest.ph`), `corpus::booleans`, `corpus::classes`, `corpus::compile_errors`, `corpus::concurrency`, `corpus::concurrency_negative`.
   - **Broader core suite status**: Per explicit user instruction ("Ignore core tests. We'll deal with it later — write into lifecycle records; language_corpus was taking too long, so it was prematurely interrupted"), execution and comprehensive triage of the broader core package and remaining corpus lanes are deferred to subsequent checkpoint work.
   - **Action**: Record and leave undisturbed.

5. **`phalcom-semantic` Baseline Failures (37 tests)**
   - **Classification**: D (baseline / unrelated)
   - **Observed**: 37 tests fail identically on clean starting revision `8c3a98aab41ed8631e9f8746a6b29984cf77581d`:
     - Export invalidation (2): `export_invalidation::test_previously_missing_public_name_appears_invalidates_consumer`, `export_invalidation::test_reexport_retargeting_invalidates_exact_consumer`
     - Record rows & aliases (2): `semantic::advanced::integration_matrix::test_matrix_1_record_row_subtyping_and_unification`, `semantic::capabilities::aliases::source_alias_cycle_and_malformed_target_are_quarantined`
     - Flow loops & control (3): `semantic::capabilities::flow_loops::break_and_continue_preserve_loop_exit_and_backedge_facts`, `semantic::capabilities::flow_loops::field_mutation_through_loop_preserves_contract`, `semantic::capabilities::flow_loops::loop_fixpoint_preserves_mutated_integer_and_abrupt_edges`
     - Generics & higher-order (3): `semantic::capabilities::generics::universe_future_preserves_payload_types_through_async_map_then_and_await`, `semantic::capabilities::higher_order::direct_and_explicit_call_on_same_block_publish_same_result`, `semantic::capabilities::higher_order::higher_order_block_call_propagates_captured_result`
     - Traits & expressions (5): `semantic::capabilities::traits::trait_default_replays_external_nominal_callable_dependencies`, `semantic::capabilities::traits::trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities`, `semantic::foundations::expression_composition::incompatible_open_record_expansions_have_explicit_diagnostic`, `semantic::foundations::expression_composition::map_expansion_does_not_fabricate_record_row`, `semantic::foundations::expression_engine::test_block_and_control_flow_typing`
     - Golden tests (2): `semantic::golden::golden_07_unknown_authority`, `semantic::golden::golden_12_higher_order_mutation`
     - Incremental DB & queries (17): `callable_dependencies::case_a`, `case_b`, `case_c`, `case_f`, `case_j`, `dependency_edit_remove_readd...`, `derived_conformance::trait_contract_edit...`, `generic_contracts::constructor_generic_bound_edit...`, `product_stability::body_only_edit...`, `pa8_recomputed_stable_intermediate...`, `previously_absent_name_addition...`, `range_only_body_edit...`, `stable_intermediate_callable_product...`, `record_rows::row_body_only_edit...`, `row_prefix_edit...`, `type_store_revisions::publication_effects...`, `stable_type_store_id...`
     - Integration & workspace (3): `imported_resolution::imported_static_domain_provider_edits...`, `workspace::alias_cycles_are_rejected_before_publication`, `workspace::imported_alias_cycle_is_rejected_before_publication`
   - **Why non-blocking**: Every selector verified on predecessor worktree and confirmed pre-existing; none are caused by P1 binding/mutability/field changes.
   - **Action**: Record and leave undisturbed.

The following are deliberate later-plan scope, not C4 baseline failures:

```text
general static evaluator                 -> P2
first-class Pack runtime/static value    -> P3
generalized FieldKey                     -> P4
Product/Sum reflection                   -> P5
static control flow/proof predicates     -> P6
recursive residualization/memoization    -> P7
durable static metadata/materialization  -> P8
ontology/tooling/performance closure     -> P9
public GADT proof metadata               -> post-C4 unless separately planned
```

---

# 8. Consultation / amendment ledger

## 2026-10-03/04 — C4 pre-implementation architectural freeze

### Scope

C4 structural/static upgrade design decisions O1–O16 and follow-up recursion /
Pack representation analysis.

### Decision

Ratified:

```text
O1
    immutable let/const require initializer;
    let mut may be uninitialized;
    uninitialized read traps;
    local definite-initialization proof removes checks where possible;
    Maybe retains runtime fallback.

O2
    constructor-lifecycle immutable-field initialization.

O3
    explicit fields only.

O4
    language-defined heterogeneous FieldKey ordering.

O5
    per-static-iteration specialization/refinement.

O6
    static String content semantics.

O7
    normalized semantic residual IR.

O8
    one Telos pattern language.

O9
    public Bool result; uncertainty residualizes and is never false.

O10
    ordinary structural Variant reflection only.

O11
    compiler-derived sealed structural capabilities.

O12
    explicit semantic while; no whileTrue role.

O13
    semantic materialization descriptor.

O14
    checked semantic definition + stable residual semantics cross-module.

O15
    immediate C4 public ontology migration.

O16
    structural semantic Pack equality.
```

Additional adopted invariants:

```text
Pack is flat/sliced structural transport, not recursive cons.
Pack rest/view is O(1).
Fixed calls do not allocate Pack.
Reflected structures remain shallow and use semantic Type references.
Reduced structural StaticValues form acyclic DAGs.
Capability/reflection derive from lower structural classification.
Type equality/hash does not recurse through Shape reflection.
Static recursive evaluation uses explicit frames and exact-key cycle detection.
```

### Architectural consequence

These rulings are the normative architecture for the C4 plans and guidance.

### Plan amendment

None. They precede P1 implementation.

## 2026-10-04 — T8 immutable-default runtime-path incident

### Scope

LANG007.C4.P1 T8.2 default initialization and T8.5 checked reads. The focused
runtime discriminator showed that semantic `default -> DefinitelyInitialized`
was not backed by ordinary instance construction: class declaration compilation
does not emit instance-default bytecode, while `@construct`/`@data` expand
defaults into constructor assignments.

### Decision

The existing C4-I-05 rule applies to every declared instance-field default,
not only derive attributes. T8 must evaluate each default once in its declaring
owner's canonical initializer. `@construct`/`@data` keep their generated
assignments as the sole default prologue; normal lowering must not duplicate
them. A plain class with defaults and no source constructor gets a canonical
zero-argument construction path only when it does not omit a required parent
initializer and it can satisfy all required immutable fields.

The class-side factory generated from `@constructor` continues to allocate
through internal `Class::allocate()` and invoke the hidden instance-side
initializer. Defaults run in the instance initializer. `Class::allocate()` /
`class_new_` and `NewInstance` remain raw allocation operations and do not
evaluate defaults; checked immutable reads trap while an instance is
incomplete.

For inheritance, preserve the established same-instance `super.new(...)`
ordering: the subclass initializer begins with exactly one explicit parent
initializer call; the parent initializer runs its own defaults and body, then
the subclass runs its own defaults and remaining body. Parent defaults are not
replayed by the child. A missing required explicit call is a diagnostic and an
invalid construction path. A derived `@construct`/`@data` path that cannot
express that call must fail closed rather than return a partially initialized
child.

P1 T8-spec is now complete. `docs/specs/objects/classes.md` owns the
source-visible class-field contract and is indexed in `docs/specs/README.md`.
The VM values and bytecode specifications now distinguish immutable-field
initialization failures from guest-visible `None`, define the checked field
load, and state that `NewInstance` allocates without evaluating defaults.

### Evidence and rationale

- `phalcom-core/src/compiler/lib/class_decl.rs` emits no bytecode for ordinary
  instance defaults.
- `phalcom-core/src/compiler/attributes.rs` emits defaults for `@construct`/
  `@data`; `lower_constructors` lowers `@constructor` to a class-side factory
  plus a same-instance initializer call.
- `phalcom-core/src/primitive/class.rs::class_new_` and VM `NewInstance` are
  allocation paths, not initializer-expression evaluators.
- `inheritance_super_construct_field_init_order_three_level.ph` establishes
  that explicit `super.new()` runs before child field assignments on the same
  instance; the default ordering follows this existing contract.

### Plan amendment

Required and completed. P1 T8 begins with T8-spec; its normative source and VM
specification sync is complete. T8a is now the first source task: publish the
canonical semantic constructor/initializer product and wire it through the
compiler entrypoint. T8 lifecycle validation and default lowering remain
gated on that product. The runtime default-path failure remains active work;
P1 is not complete.

## 2026-10-04 — T8 canonical constructor identity consultation

### Scope

Follow-up consultation for P1 T8 after confirming that semantic constructor
signatures and body traversal currently cover source-declared members, while
`@construct`/`@data` and default-only constructor paths are synthesized in the
compiler. Required by the fixed explicit-super rule: a subclass must resolve
its direct parent's canonical initializer even when that initializer is
generated.

### Decision

P1 must publish one canonical semantic constructor/initializer product for
source-declared, `@construct`, `@data`, and default-only construction paths.
Semantic super-call validation and compiler lowering must consume that same
published product. Do not validate only source constructors, synthesize a
second source index, or let the compiler independently derive constructor
identity/signature.

The product records or canonically references the declaring owner and stable
constructor identity; class-side factory callable and signature; paired
same-instance initializer identity/signature; ordered default `FieldId`s;
generated constructor-parameter-to-field mapping; constructor origin; and the
unique direct-parent initializer resolved from an explicit `super.new(...)`
call. Its dependencies/fingerprints cover owner fields/defaults, constructor
and derive declarations, superclass shape, and super-call arguments. Changes
or removals invalidate dependent child resolution and compiler projection.

The adopted construction contract remains unchanged: explicit `super` is
required, with no auto-chain; parent defaults and body complete before child
defaults and body; defaults are not evaluated by raw allocation. `@construct`
and `@data` retain their existing generated assignment behavior without a
duplicate default prologue.

### Evidence and rationale

- `CallableSignatureTable::get_for_body` and declaration signature helpers
  bridge source-declared constructor members only.
- Semantic session body traversal visits source class members; no semantic
  synthetic/default-only constructor projection exists.
- `FieldLifecycleTable` publishes field-default facts but no constructor
  identity.
- Core attribute expansion and `lower_constructors` synthesize constructors
  and hidden instance initializers after semantic analysis.
- Existing source-declared parent `@constructor` plus explicit `super.new()`
  is covered; generated/default-only parents have no canonical semantic target.

### Plan amendment

Level 2 amendment required and recorded in the active P1 plan. T8a publishes
semantic constructor/initializer products before T8 semantic lifecycle and
explicit-super validation, and wires the snapshot product through
`compile_analyzed` into semantic lowering. The compiler codegen/runtime stage
then consumes the same products and checked parent link. A stop condition forbids
continuation if compiler lowering cannot consume the product without
independent derivation. Required focused verification includes product
coverage for all four origins; explicit parent resolution for source,
`@construct`, `@data`, and default-only parents; generated field mapping/order;
incremental invalidation; existing three-level inheritance; once-only defaults;
and raw allocation behavior.

The current normative class specification already states the user-visible
default, ordering, explicit-super, raw-allocation, and generated-constructor
rules; no internal representation rule is added to it. No source or test code
was changed in this consultation. P1 remains `IN_PROGRESS / PARTIAL /
UNVERIFIED`.

## 2026-10-04 — T8a semantic-product-driven constructor projection consultation

### Scope

Follow-up to the T8a product publication and transport. Live source confirmed
that `compile_analyzed` transports the canonical products and explicit parent
initializer link, but `derive_construct` and `lower_constructors` still
materialize generated constructors from AST fields, names, and parameters.
That leaves the compiler consumer able to reconstruct facts T8a assigned to
semantic authority.

### Decision

Use semantic-product-driven constructor projection at the class compiler
boundary. Semantic products remain the authority; the compiler may build a
compiler-only AST/IR projection for the existing emitter, but it must not
create source declarations or feed synthesized declarations back into
semantic analysis.

- At class compilation, obtain the declaring owner's constructor products
  before constructor materialization.
- Replace field scanning in `derive_construct` with projection from the
  product's canonical `default_fields` and `parameter_fields` order. Source
  syntax may provide only the default expression associated with a canonical
  `FieldId`.
- `@data` continues deriving non-constructor methods, while its constructor
  comes from `ConstructorOrigin::Data`.
- Materialize default-only factory and initializer paths from their products.
- `lower_constructors` consumes product factory/initializer identities and
  signatures; it must not derive a selector from an AST name or parameters.
- Evaluate defaults once: generated construct/data assignments remain the
  only default prologue for those origins; source/default-only paths use the
  canonical initializer. Do not duplicate defaults.
- Keep the semantic resolved-parent link and canonical
  `CallableSignatureTable`; transport the existing table if needed rather
  than copying or rebuilding it.
- If the current `MethodDef` route cannot preserve product-owned identities,
  use a separate compiler-only lowering projection. Do not add identity or
  field-mapping fallbacks.

### Plan amendment

Required. Extend amended T8a's write-set to include
`phalcom-core/src/compiler/attributes.rs` and
`phalcom-core/src/compiler/lib/class_decl.rs` (plus compiler context only if
needed). Add consumer-level codegen regressions proving product-owned
constructor identity, field/default order, parameter mapping, and once-only
defaults for generated and default-only paths. Keep T8a open until the
class-construction consumer complies; the already-published semantic products
and `compile_analyzed` transport are partial prerequisites, not acceptance.

No further user-visible language decision is required. The existing explicit
`super` decision remains: a subclass must call the direct parent's canonical
initializer explicitly; the compiler must consume the semantic resolved link.
P1 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`.

---

## 2026-10-04 — T8a compiler consumer slice

### Implemented

Class construction now projects compiler-only factory and initializer members
from the semantic `ConstructorInitializationProduct` and canonical
`CallableSignatureTable`. Source constructors are paired by their published
source span; `@construct`, `@data`, and default-only paths are selected by
semantic origin. The compiler reads defaults by canonical `FieldId`, applies
generated parameter mappings in product order, installs the product-owned
factory/initializer selectors, and uses the semantic explicit-parent
initializer selector directly. Subclass defaults follow the resolved
same-instance parent initialization call.

`ModuleLoweringSemantics::new` also initializes the transported canonical
callable-signature table. Constructor compilation without the required
semantic product fails closed.

### Verification

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core` — PASS before
  the final exact-super-selector adjustment; the final focused test below
  rebuilt the changed compiler and test target successfully.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core compiler_installs_and_executes_canonical_constructor_projections -- --nocapture` — PASS, 1 selected. Exercises `@construct`, `@data`, default-only parent construction, source construction, installed selectors, field writes, runtime values, and parent-before-child default order.
- The earlier duplicate-encoding failure for the semantic parent initializer
  was corrected in `compiler/lib/expr.rs`; the rerun above passed.
- T9 focused LSP/REPL checks and TextMate JSON validation passed during this
  continuation (the handoff lists exact selectors). T7, G1, T8 lifecycle
  acceptance, T10, and G2 are not established by this slice.
- `git diff --check` over the scoped compiler source paths — PASS after the
  implementation edits. The pause records were updated and reviewed for stale
  status claims; they are untracked in this checkout, so Git's diff check does
  not include them.

This completes the amended T8a compiler-consumer implementation slice, not
T8's immutable-field lifecycle acceptance or P1. No broad stabilization gate
was run. The dirty shared worktree remains preserved.

## Earlier P1 closure checklist (superseded by current next action)

1. Run the focused T7 explicit-field semantic/parser/core tests; classify any
   failures A/B/C/D and record the actual result.
2. Inspect the live T8 lifecycle implementation, complete missing immutable
   write-once/constructor-exit/default-path behavior, and add/run the focused
   source and runtime regressions. Include semantic product/incremental checks
   for T8a not covered by the consumer test.
3. Run G1's bindings/fields semantic and language-corpus gates.
4. Confirm the T9 tooling results against the active plan and retain their
   exact evidence.
5. Complete T10's `let`/`let mut` and field-mutability invalidation checks,
   checked-read lowering inspection, and zero-allocation evidence.
6. Run G2 serially across the six affected crates and classify all failures.
7. Finish T11: update the P1 ledger and checkpoint verification/completion,
   then refresh the final walkthrough and handoff. Keep P2 blocked until P1 is
   complete.

## 2026-10-04 — P1 T7/T8/T10 focused continuation (pre-G2 pause)

### Implementation and focused evidence

- Constructor default projection now classifies fields by canonical
  `member_side`, so `@class` fields cannot leak into instance constructor
  default products. The semantic constructor-product regression includes a
  class-side field and asserts only the instance default is published.
- Class-side fixtures now declare static storage with `@class`; the shared
  state fixture mutates that storage through a class-side method.
- T7's live layout builder has no assignment-inference fallback. Explicit
  declaration order, immutable slot classification, and rejection of
  undeclared field layout writes passed in the focused core layout suite.
- Immutable lifecycle semantic tests, runtime default/read tests, and the
  explicit-super semantic tests passed. Binding flow reanalysis, field
  mutability invalidation, checked-read lowering, and the immediate-sentinel
  allocation check also passed.

### Verification

```text
semantic capabilities::bindings                 12 passed
semantic capabilities::fields                   27 passed
core field_layout                                4 passed, 1 ignored (GATED)
core field_lifecycle                             2 passed
core binding_init                                13 passed
semantic binding_flow_edit_rechecks_only_the_affected_callable 1 passed
semantic field_mutability_edit_invalidates_field_and_write_products 1 passed
AST c4_migrate test_migrate_implicit_fields      1 passed
```

`cargo fmt --all -- --check` remains red because formatter differences are
spread across many existing files, including files outside this scoped change;
no repository-wide formatting was applied. `git diff --check` passed for the
scoped work.

### Open acceptance and failure classification

- The T7 `migrated_layout_matches_pre_cutover_oracle` test now parses the legacy
  assignment-order fixture and compares it with the explicit compiled layouts;
  it also freezes retired `const` field identity/mutability and verifies the
  bare immutable declaration mapping. The focused test passed.
- The semantic portions of G1 passed. The core bindings corpus failed in
  `bindings/destructure_list_no_rest.ph`: runtime attempted `unwrapOr(_)` on
  integer `1` in `phalcom-core/src/compiler/lib/patterns.rs::emit_element_read`.
  This is classified D (unrelated baseline). The new `language-corpus fields`
  lane passed its explicit-field and explicit-super constructor fixture.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::classes -- --exact --nocapture` passed (1 passed, 793.41 seconds). The adjacent `classes_negative` test also passed.
- Its first attempt found `ctor_inherited_bare_new.ph` omitted the now-required `super.new()`; the fixture was migrated and its direct execution retained the expected output.
- At this pre-G2 pause, G2 had not run. P1 remained
  `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED`; this historical continuation
  did not close the plan or authorize P2. See the current G2 result above.

## 2026-10-04 — P2 T6 checked-body query lifecycle consultation

### Trigger and observed code

T6 integration reaches a lifecycle boundary. `session.rs` resolves and queries
`StaticDefinitionProduct` around lines 3636–3659, before the final ordinary
checker pass has stabilized canonical dispatch surfaces. The existing
`query_static_definition` uses `QueryKey::StaticDefinition`, an input
fingerprint based only on declaration identity, and no semantic dependency
edges; a cache hit returns the earlier product. Enriching that query with body
facts after the later checker pass would therefore risk stale body products.

### Consultation decision

The architectural consultation recommends a distinct `CheckedStaticBody`
SemanticDb query keyed by `StaticBodyId` (the static definition's
`DeclarationId`). Its input fingerprint must include body/signature/result
domain and phase-readable outer-binding inputs. The ordinary checker records
precise semantic dependencies; its product fingerprint includes exact
selected callables, binding/slot maps, static-operation resolutions, result
facts, and diagnostics. The snapshot-facing `StaticDefinitionProduct` composes
the resolved signature/alias projection with this one cached body product; it
must not compute or cache a second body analysis.

### Plan amendment

The active P2 plan's T6 now requires the distinct checked-body query and
snapshot composition. It also requires ordinary module checking to publish a
`ModuleStaticBindingIndex` keyed by stable module constant identity. Static
bodies consume checked type/phase facts from this handoff, seed body-local
bindings, and record stable constant reads; T8 consumes the same facts. This
preserves the §5.4 canonical static-definition contract and the one-body-
authority rule while adapting to live session ordering. Exact-intrinsic and
binding-identity invariants are unchanged.

### Remaining boundary

The current `analyze_static_body` helper is locally focused-tested but still
standalone. Session construction of phase-readable static-constant inputs and
the query/dependency/fingerprint composition remain unimplemented. P2 stays
`IN_PROGRESS / PARTIAL / UNVERIFIED`.

### Follow-up consultation — stable module-constant checker facts

The ordinary `<main>` `CallableAnalysis` retains binding facts, but this does
not identify which immutable binding is a `const`, and its `BindingId` is local
to that body. Do not seed a static-definition checker from a copied `<main>`
`BindingId` or infer phase from mutability after the fact.

The follow-up architectural consultation recommends that ordinary module
checking publish a `ModuleStaticBindingIndex` keyed by stable `SymbolId` (or
equivalent `StaticConstantId`). Each entry carries the source `<main>` body and
`BindingId`, checked type/contract, source site, and a phase category recorded
at declaration checking. Static bodies seed local binding IDs from this
product and preserve their mapping to stable constant identity. They record
which stable constants they read; the checked-body query depends on the handoff
product, and T8 uses the same read set and facts for `StaticConstant` products.
Imported/aliased constants must follow canonical linked identities.

The T6 plan now includes this checker-produced handoff as a prerequisite to
session integration. This avoids rerunning name/type resolution and makes
constant phase authority explicit.

### Implementation slice

Added `ModuleStaticBindingIndex` in `phalcom-semantic/src/module_bindings.rs`
and snapshot publication from each module's final `<main>` `CallableAnalysis`.
The index is keyed by stable `SymbolId`; each fact retains the owning main
callable, the exact source `BindingId`, cloned checked `BindingState`, and a
phase category selected from the top-level AST declaration kind. This creates
the advised identity/type/phase handoff. It remains UNVERIFIED in this slice:
focused construction/phase tests and dependency invalidation are still needed,
and the checked-body query does not yet consume it.

### Implementation continuation — module binding consumption

The module handoff now also includes linked import aliases resolved through
`LinkedReadSpec::Binding(SymbolId)`. Static-body analysis has a new entry point
that seeds checked visible module bindings with fresh body-local IDs, marks
only declaration-time `const` IDs as readable, retains their stable
`SymbolId` mapping, records stable constant reads, and carries the module
binding product fingerprint. `CheckingContext::declare_checked_binding`
copies checked current/contract/consistency facts without rerunning contract
reconciliation. This keeps source-body `BindingId`s out of the static body and
avoids a second type-proof path.

The distinct `CheckedStaticBody` query now owns the checked result under its
own `QueryKey`, stores source/signature/module-binding input identity and a
complete output fingerprint, and records query edges for captured semantic
dependencies plus `ModuleStaticBindings`. The final session pass publishes the
module index, checks or reuses each checked body, and composes it into the
snapshot-facing `StaticDefinitionProduct`; the early definition query remains
definition-only. This integration remains UNVERIFIED: no focused construction,
invalidation, import, or body-analysis gate has run. Continue T6 with focused
regressions and review any query dependency replay issues before accepting it.
The early `StaticDefinition` query input fingerprint now includes the resolved
signature and source body, preventing same-identity edits from reusing its
prior definition product.

## 2026-10-05 — P3 patch-grade architecture planning audit

### Scope

Prepared `LANG007.C4.P3-luna-patch-grade-implementation-plan.md` from the detailed P3 requirements analysis and a fresh read-only audit of live `main` at `5a5bf8b218c2fbcfd785bfa41bb4eb7fec21c521`. This is a plan-readiness event, not a P3 implementation event.

### Live repository findings carried into the plan

- AST already owns the canonical `ExpansionMode::Positional/Labeled/Complete`, `PackItem`, rest modes, and reusable anonymous remainder syntax. No second pack-lane syntax model is authorized.
- C1–C3 `TypeData::TypePack`, `LabeledTypePack`, `CompleteTypePack`, concat forms, `TupleFromPacks`, and `GenericParameterMode` remain canonical optimized type-level authorities. A first-class Pack therefore requires a distinct proper-Type representation plus a bridge, not replacement.
- semantic `StaticParameter` currently loses source pack mode for value-domain parameters even though `StaticParameterSyntax` retains it. P3 will retain mode directly while preserving the P2 rule that only kinded parameters receive `TypeParameterId`.
- P2 `StaticValueStore` has the intentional first-class Pack hole and is ready for additive StaticPack backing/slice nodes; no second static-value store is planned.
- runtime `ArgumentPackBuilderObject` is already the correct mutable staging primitive. It remains private; dynamic `InvokePack` continues consuming it directly. `FinishTuplePack` remains Tuple-specific.
- builder duplicate-label membership is currently linear (`labels.contains`); P3 plans a small-fast/large-indexed strategy without changing label-error timing.
- method rest capture currently materializes Tuple for all four rest modes, block rest materializes Tuple, and Fiber rest currently materializes List. P3 plans one VM Pack-capture authority for every language-rest entry path.
- Tuple/Record product storage/layout/descriptor infrastructure is reusable, but Tuple identity is not. P3 plans a distinct runtime Pack kind/shape with immutable root ProductStorage and GC-safe root-referenced O(1) slices.
- Value tag space permits a preferred immediate `EmptyPack` representation subject to implementation-time exhaustive ABI/tag audit.
- ordinary type metadata already persists open Tuple and C3 symbolic pack forms. Because proper Pack types may cross module boundaries during P3, minimal stable Pack type metadata is included in P3; durable StaticPack value serialization remains P8.
- existing outgoing-pack tests already prove fixed calls avoid builder machinery. P3 makes this a blocking performance gate and does not route fixed calls through first-class Pack.

### Planned architecture

```text
source Pack/rest/spread syntax
        ↓
canonical semantic Pack declaration + proper Pack type
        ├─ bridge to existing C3 kinded pack terms
        └─ P2 StaticValue::Pack for compile-time values
        ↓
compiler lowering
        ├─ fixed calls: existing direct invoke, no Pack/builder
        ├─ dynamic calls: private builder -> InvokePack
        └─ first-class Pack construction: direct static build or private builder -> FinishPack
        ↓
runtime immutable Pack
        root ProductStorage + ordered labels
        O(1) slices retaining the root through GC
```

### Plan state consequence

P3 remains:

```text
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
```

The next action is P3 T0. When actual implementation begins, T0 must move the checkpoint row to `IN_PROGRESS / PARTIAL / UNVERIFIED`, record the real starting revision, and run only the named baseline discriminator.

---

# 9. Performance and resource invariants

C4 performance is part of correctness.

## 9.1 P1 binding reads

A semantically proven initialized local read must not gain a runtime
initialization branch.

Checked reads exist only where proof is absent/uncertain.

Uninitialized binding state must not allocate a heap object per binding.

## 9.2 Pack

Required asymptotics:

```text
lane length              O(1)
first/rest               O(1)
slice/rest               O(1)
builder append           amortized O(1)
builder finalize         O(n)
equality/hash            O(n)
multi-spread lowering    O(total arguments)
unordered canonicalize   O(n log n) acceptable
```

Fixed calls must not allocate first-class Pack values.

## 9.3 Reflection

Reflection must not recursively expand type graphs.

`VariantsOf` should be O(number of variants).

Lookup should be O(log n) or cached O(1) where appropriate.

## 9.4 Static evaluation

No host-stack recursion proportional to Telos recursion.

Cancellation/budget must remain responsive.

Memoization keys must be canonical semantic identities, not source text or
arena addresses.

## 9.5 Metadata

Persist DAG sharing/references.

Do not recursively serialize infinitely expandable semantic graphs.

---

# 10. Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| runtime/static binding phase | semantic declaration model | module/compiler/P2 | `is_const` reinterpretation |
| runtime mutability | semantic binding model | flow/compiler/tooling | direct `BindingKind::Let` assumptions |
| binding initialization | semantic `FlowState` | lowering/tooling | compiler CFG solver |
| field existence | explicit class declaration | semantics/compiler/reflection | assignment inference |
| field mutability | semantic field signature | compiler/metadata/tooling | layout default reconstruction |
| field lifecycle | semantic field lifecycle | compiler/runtime | syntactic constructor-only flag |
| static values | semantic static-value arena | evaluator/compiler/metadata | runtime `Value` |
| proof relations | existing type relation/conformance engines | static proof predicates | static-only relation engine |
| static patterns | existing resolved patterns/GADT proofs | static matcher | static-only pattern resolver |
| rows | canonical row calculus | FieldKey/records/reflection | static-only row system |
| Pack semantic type/value | C4 Pack model + optimized runtime substrate | calls/rest/reflection | recursive cons list |
| structural classification | compiler/semantic declaration/type authority | sealed capability + reflection | capability↔reflection recursion |
| static residual | normalized semantic residual IR | specialization/metadata | raw AST/evaluator frames |
| static query/memo | semantic incremental DB/evaluator memo | compiler/importers | hidden second cache |
| runtime materialization | semantic descriptor + compiler lowering | VM/runtime | rerunning initializer |
| checkpoint state | this file | plans/guidance/handoffs | competing implementation-state file |

---

# 11. Checkpoint update obligations

## At each plan start

The implementer must:

```text
locate this checkpoint
confirm the plan row
mark active_plan
record starting revision when useful
confirm predecessor walkthrough/handoff
classify inherited failures
```

## During a plan

Update only at durable events:

- stable interface established;
- ratified invariant implemented;
- consultation/amendment;
- deferred failure classification;
- coherent verification gate.

Do not log every edit/test.

## At plan completion

Update:

```text
plan status/completion/verification
final durable invariants
verification evidence
residual failures
consultations/amendments
latest_completed_plan
active_plan
next_plan
next_action
```

A plan is not administratively complete until this checkpoint is current.

---

# 12. Checkpoint acceptance direction

`LANG007.C4` is not accepted when only P1 syntax or P2 static constants work.

C4 closure eventually requires coherent evidence for:

```text
runtime/static binding cutover
true uninitialized binding semantics
immutable-field lifecycle
explicit fields
semantic StaticValue model
closed const reduction
first-class efficient Pack
generalized FieldKey
Product/Sum reflection
sealed structural capabilities
static structural control flow
one pattern/GADT authority
Bool-shaped proof predicates with preserved uncertainty
recursive static computation
normalized residual IR
memoization and cycle control
incremental static queries
cross-module semantic static definitions
materialization without runtime re-execution
public ontology hard cutover
tooling/docs/spec convergence
performance/resource invariants
cold/incremental parity
```

Final C4 completion must remain distinct from per-plan completion and from
release-wide certification.

---

# 13. Current next action

`LANG007.C4.P2` remains **COMPLETE / IMPLEMENTED / FOCUSED_TESTED** and is the latest completed plan. Its completed static-value/evaluator/materialization/incremental gates are inherited prerequisites and must not be reopened by P3 without new causal evidence.

`LANG007.C4.P3` is active and remains **IN_PROGRESS / PARTIAL / UNVERIFIED**.

### P3 planning readiness

The following are now prepared against live `main` revision `5a5bf8b218c2fbcfd785bfa41bb4eb7fec21c521`:

```text
LANG007.C4.P3-requirements-analysis.md
LANG007.C4.P3-luna-patch-grade-implementation-plan.md
```

The patch-grade plan freezes these architectural directions for execution:

- a proper first-class Pack type distinct from C3 kinded pack terms, with one explicit C3 bridge;
- StaticPack in P2's existing StaticValueStore with immutable backing and O(1) slice descriptors;
- canonical static-parameter mode retention for value-domain variadic capture;
- a distinct runtime Pack root/slice representation reusing product storage/layout but not Tuple identity;
- preferred immediate `EmptyPack`, subject to implementation-time tag/ABI audit;
- private mutable argument builder retained for dynamic calls and first-class Pack finalization only when needed;
- one Pack rest-capture authority replacing current method Tuple, block Tuple, and verified Fiber List rest materializations;
- direct Pack lane spreading/forwarding without Iterator or Tuple/List conversion;
- explicit Tuple interoperability only; Record remains non-Pack/unordered;
- minimal stable ordinary Pack type metadata in P3, while durable StaticPack value persistence remains P8;
- fixed-call no-builder/no-Pack behavior as a blocking P3 performance invariant.

### Current P3 State and Next Action

P3 takeover and baseline work are complete. T1's semantic source-form
continuation and T4, T5, and T6 have focused implementation evidence recorded
below. The user resolved T7's source-name conflict by preserving the existing
C3 domain names and omitting colliding proper-type aliases. The focused
full-Universe rest-capture integration now passes with the proper Pack aliases
`PositionalPack<T>`, `LabeledPack<T>`, and `CompletePack<T>`. T7 source-view
testing also passes for empty and nonempty lanes and direct root-backed repeated
slices, but source equality fails for a root Pack versus an equivalent visible
slice. T8 and T9 remain unstarted.

The user resolved incident `INC-P3-1` by directing canonical typed Pack
remainder formation in the semantic layer. That T1 continuation now passes its
focused semantic regressions and is reflected in the normative structural
products specification. Incident `INC-P3-2` is resolved: preserve C3's
`TypePack`, `LabeledTypePack`, and `CompleteTypePack` kinded domains, and do
not define proper-type aliases under those names. The corresponding source
aliases remain `PositionalPack<T>`, `LabeledPack<T>`, and `CompletePack<T>`.
The full-Universe integration selector
`core_outgoing_packs_completion::p3_language_rest_capture_converges_on_pack`
passed 1/1 after removing the aliases. T7 remains active for the remaining
view/native/equality source checks; keep T8/T9 and broad gates open.


## P3 T4 construction repair — 2026-10-05

**Scope/state:** the reported Pack construction failure is corrected locally;
this repair is `IMPLEMENTED / FOCUSED_TESTED`. P3 remains `IN_PROGRESS / PARTIAL`;
T5–T9 and broader gates have not been completed by this repair.

**Inline advisory decision:** the handoff conflated ordinary data-constructor
dispatch with exact product-type materialization. `VM::get_or_bind_data_descriptor`
requires a registered data declaration, whereas intrinsic Pack intentionally
binds an existing runtime class without allocating a data descriptor. Standalone
fixed construction independently reproduced `<class Pack> does not understand
'call(_,_,label)'`. Registering Pack as ordinary data or discarding exact-type
errors would violate the structural construction contract.

**Established correction:** semantic analysis normalizes known canonical Pack
constructor results to proper Pack types while retaining their evidence strength.
Lowering records checked canonical Pack constructor sites and excludes them from
ordinary data construction. Static analyzed builds consume `BuildStaticPack`
and propagate exact-type materialization errors. Standalone fixed builds use
`BuildPack`, selected only through the linked builtin canonical runtime class
after shadowing resolution. Expanded construction uses one private builder and
one `FinishPack`. Ordinary fixed sends and `FinishTuplePack` keep their existing
lanes. No Pack spelling check authorizes specialization.

**Plan clarification:** recorded under P3 T4.4, covering the existing standalone
compiler's lack of a semantic snapshot. No change to Pack semantics, ownership,
GC policy, or canonical identity was adopted. Advisory/debugging work was
completed inline at the user's instruction.

**Executed verification:**

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs_completion::p3_ -- --nocapture --test-threads=1`
  selected six tests and passed. Covers fixed, expanded and empty construction,
  analyzed static product projection/execution, analyzed spread execution,
  one-builder/one-finalizer bytecode, ordinary fixed calls, Tuple finalization,
  and lexical Pack-name shadowing. The initial shadowing fixture used invalid
  closure syntax; correcting the fixture was mechanical and changed no oracle.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib pack_builder -- --nocapture`
  selected two tests and passed, covering exact label reservation and the
  duplicate-index threshold crossover.
- Scoped diff whitespace validation passed.

**Deferred:** full outgoing-pack legacy acceptance, product optimization,
language corpus, workspace certification, and remaining P3 tasks. No new
behavioral failure remained in the repair's focused tests. The pasted handoff's
constructor-probe baseline classification is inherited and was not reverified.

**Superseded next action:** T4 acceptance was resumed and T5/T6 focused work
has since been completed; see the continuation record below. P3 remains open.

## P3 T4-T6 continuation and T7 semantic formation incident — 2026-10-05

**Durable state:** T1 semantic source formation, T4 construction, T5 rest
capture, and T6 Pack spreading are `IMPLEMENTED / FOCUSED_TESTED` for the
covered paths. P3 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`. T7 is active
after resolving the source-name contract in `INC-P3-2`; T8/T9 remain
incomplete.

**T5 implementation:** method positional/labeled/split/complete rest, block
rest, and Fiber closure rest now converge on Pack capture in the live diff.
Owned staging helpers avoid making a second copy of the call-frame buffers.
The focused regression covers empty and populated lanes, labeled encounter
order, ordinary and Pack forwarding, block rest, and Fiber rest. A source census
found no additional language-rest binding path requiring conversion.

**T6 implementation:** Pack positional, labeled, and complete spreads preserve
their visible lanes; explicit Tuple conversion and Pack round-trip preserve
Tuple lane structure; multi-spread construction and calls each use one builder.
Duplicate-label validation from an earlier Pack spread occurs before evaluating
the later explicit value. The existing fixed-call bytecode discriminator from
the T4 record remains the evidence for the no-builder fast path.

**Executed focused verification:**

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs_completion::p3_language_rest_capture_converges_on_pack -- --nocapture` — 1 passed.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs_completion::p3_pack_spreading_preserves_lanes_and_fuses_builders -- --nocapture` — 1 passed.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs_completion::p3_duplicate_from_earlier_pack_spread_prevents_later_value -- --nocapture` — 1 passed.

### INC-P3-1 — typed Pack remainder source-form ownership

**Plan/task:** `LANG007.C4.P3.T7`, depending on T1's typed-remainder formation.

**Trigger:** live semantic behavior conflicts with the T1/T7 source contract;
resolving it may change canonical type formation in the semantic layer.

**Observed:** T7 requires public aliases and one-step view remainders in the
forms `Pack<*: T>` and `Pack<**: T>`. A temporary ordinary-project import probe
reported `AnnotationUnsupported` on those source forms. The live generic
application lowering paths in `phalcom-semantic/src/types/annotation.rs`
explicitly reject `GenericArgumentSyntax::Remainder` outside their existing
structural/RecordFields handling; no canonical Pack type-formation branch was
found. The temporary failing probe and only the experimental T7 source edits
were removed, preserving the pre-existing Pack source worktree contents.

**Root-cause hypothesis:** inherited T1 work established Pack type storage,
parser syntax, and metadata, but did not complete the source-level lowering
needed for canonical Pack lane remainders in ordinary type-form contexts. The
additional unresolved `Type` name in the probe is secondary; adding an import
would not address the explicit remainder rejection.

**Decision (user, 2026-10-05):** complete canonical typed Pack remainder
formation in the semantic layer and reopen T1 source-form acceptance. Preserve
`TypeData::Pack` as the proper Pack authority; add capture-safe scoped Pack
formation for generic type bodies and materialize it to the same proper Pack
type after substitution. Canonical declaration identity selects this path;
ordinary generic applications continue to reject remainder syntax.

**Plan amendment:** T1.2 now explicitly includes source-form lowering in both
resolved and scoped type formation, plus tests for all three remainder modes,
generic type-lambda substitution, and rejection on a non-Pack constructor. The
owning structural-products specification now defines proper Pack remainders.
**Verification:** the parser remainder test and four matching semantic tests
passed. The core Pack rest integration selector later passed after resolving
`INC-P3-2`. **Status:** T1 source formation is implemented and focused-tested.

### INC-P3-2 — C3 `CompleteTypePack` name conflicts with the proper Pack alias

**Identity:** `LANG007.C4.P3.T7`, Phalcom repository, shared `main` worktree.

**Escalation trigger:** accepted plan material conflicts with the live C3
source namespace and semantic identity (protocol triggers 1, 2, and 16).

**Required invariant:** C3 `TypePack`, `LabeledTypePack`, and
`CompleteTypePack` remain canonical kinded domains; proper Pack aliases must
not silently replace or shadow those identities.

**Expected architecture:** plan §5.1 proposes proper aliases named
`PositionalTypePack`, `LabeledTypePack`, and `CompleteTypePack`, while §7.3 and
T7 require `PositionalPack<T>`, `LabeledPack<T>`, and `CompletePack<T>` over
canonical `Pack` remainders and preserve C3 kinded binders.

**Observed:** the live `tuple.ph` defines and exports the intrinsic C3
`CompleteTypePack`; `pack.ph` imports it for the `Pack<***Es>` implementation.
Adding `type CompleteTypePack = CompletePack<Dynamic>` causes full Universe
bootstrap to fail with `Duplicate binding 'CompleteTypePack'`. The three core
aliases are otherwise present in the active source diff.

**Minimal reproduction:**

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs_completion::p3_language_rest_capture_converges_on_pack -- --nocapture
```

The test fails during canonical Universe interface loading before executing
its assertion, with the duplicate binding above and previous binding range
`311..332`. The existing C3 import is at `collections/pack.ph:1`; the
conflicting alias is in the same module. No attempt was made to rebind or
rename the C3 anchor.

**Decision:** the user directed preserving C3 names and dropping conflicting
proper-type aliases. The three proper Pack source aliases are
`PositionalPack<T>`, `LabeledPack<T>`, and `CompletePack<T>`; C3
`TypePack`/`LabeledTypePack`/`CompleteTypePack` remain kinded domains.
**Resolution:** removed the three attempted `*TypePack` aliases from
`pack.ph`, updated plan §5.1, and resumed T7. The full-Universe focused source
integration `core_outgoing_packs_completion::p3_language_rest_capture_converges_on_pack`
passed 1/1. T7 remains active until its view, native-floor, equality/hash, and
rendering checks are complete.

### T7 source view/equality continuation — 2026-10-05

**View evidence:**
`core_outgoing_packs_completion::p3_pack_source_views_decompose_one_step_over_shared_roots`
passed 1/1 in full-Universe project context. Empty variants, positional and
labeled Entry payloads, label encounter order, and repeated positional/labeled
rest slices were exercised. Repeated and nested positional remainders point
directly to the original Pack root with the expected offsets.

**Open correctness failure:**
`core_outgoing_packs_completion::p3_pack_source_equality_hash_and_rendering_use_visible_lanes`
failed at `root == slice`. `root` is `Pack(1, 2, right: 3, left: 4)`; `slice`
is formed from `Pack(0, 1, 2, hidden: 99, right: 3, left: 4)` by dropping the
first positional and labeled entries. The earlier root-to-root equality
assertion passed. Hash equality, lane/order inequality, nested equality/hash,
and the final rendering assertion occur later in that test and are therefore
not verified. No cause was established and no corrective edit was attempted.
The test-only reproducer currently remains uncommitted in
`phalcom-core/tests/core/collections/outgoing_packs_completion.rs`.

**Resolution of open equality failure:**
The root-versus-slice equality failure was diagnosed and corrected at the canonical
Pack slicing seam in `phalcom-core/src/primitive/pack.rs`.
`pack_raw_positional_rest_slice` and `pack_raw_labeled_rest_slice` previously
zeroed out the other lane's length; they now correctly preserve `lab_len` and
`pos_len` respectively, ensuring that slicing one lane leaves the other lane's
elements intact on the slice view.
Following this fix, `core_outgoing_packs_completion::p3_pack_source_equality_hash_and_rendering_use_visible_lanes`
passed 1/1, verifying visible lane equality, hash equality, order sensitivity,
nested Pack equality/hash, and canonical rendering.

---

## P3 Completion and Verification — 2026-10-05

`LANG007.C4.P3` is **COMPLETE / IMPLEMENTED / FOCUSED_TESTED**.

### Durable invariants established by P3

1. **First-class Pack type formation:**
   `TypeData::Pack` models ordered positional and Symbol-labeled lanes with
   `Closed`, `Typed(T)`, or `Unconstrained` remainder states. Intrinsic source
   aliases are `PositionalPack<T>`, `LabeledPack<T>`, and `CompletePack<T>`.
   C3 kinded domains `TypePack`, `LabeledTypePack`, and `CompleteTypePack` are
   preserved.
2. **Static Pack value & parameter mode hashing:**
   `StaticValueData::Pack` stores compile-time Pack values with lineage,
   structural equality, and stable hashing. `StaticParameter.mode` is hashed into
   `static_definition_product_fingerprint` to ensure variadic/mode changes
   correctly invalidate downstream definitions and applications.
3. **Runtime Pack representation & slicing:**
   `ValueTag::EmptyPack` (tag 12) is the immediate representation for `Pack()`.
   Non-empty Packs use heap `Object::PackRoot` or `Object::PackSlice`.
   Positional and labeled rest slicing preserves the opposing lane and points
   directly to the underlying root backing.
4. **Bytecode & builder convergence:**
   `FinishPack` (112), `BuildStaticPack` (113), and `BuildPack` (114) provide
   direct Pack construction without leaky builder abstractions. Ordinary fixed
   calls and static Tuple construction emit direct bytecodes with zero Pack
   machinery.
5. **Call-site rest capture & forwarding:**
   Method positional/labeled/split/complete rest, block rest, and Fiber rest
   converge on first-class `Pack`. Pack forwarding reads lanes directly without
   Tuple/List adaptation.
6. **One-step structural views:**
   `Pack.telos` provides canonical decomposition into `PackLaneEmpty` and
   `PackEntry` variants sharing root backing.

### Verification gates evidence

- **Gate G1 (Semantic type/static Pack vertical):**
  - `cargo test -p phalcom-ast pack_type_remainder`: 1 passed.
  - `cargo test -p phalcom-type-meta pack`: 3 passed.
  - `cargo test -p phalcom-semantic --test semantic foundations::pack_types`: 4 passed.
  - `cargo test -p phalcom-semantic --test semantic foundations::static_values_pack`: 1 passed.
  - `cargo test -p phalcom-semantic --test semantic foundations::static_definitions`: 6 passed.
  - `cargo test -p phalcom-semantic --test semantic incremental::pack_types`: 4 passed.
- **Gate G2 (Runtime Pack/rest/spread vertical):**
  - `cargo test -p phalcom-core --test core core_pack`: 5 passed.
  - `cargo test -p phalcom-core --test core memory_pack_gc`: 3 passed.
  - `cargo test -p phalcom-core --test core core_outgoing_packs_completion::p3_`: 11 passed.
- **Gate G3 (Fixed-call / Resource / Performance proof):**
  - `core_outgoing_packs_completion::static_calls_and_static_tuple_emit_no_pack_machinery`: 1 passed (confirmed zero Pack bytecodes emitted for fixed calls).
- **Gate G4 (Focused compatibility/stabilization):**
  - `cargo check -p phalcom-ast -p phalcom-type-meta -p phalcom-semantic -p phalcom-core`: passed cleanly.
  - `cargo test -p phalcom-semantic --test semantic type_alias`: 2 passed.
  - `cargo test -p phalcom-semantic --test semantic adts`: 318 passed (28 gated).
  - `cargo test -p phalcom-semantic --test semantic foundations::static_values_structural`: 1 passed.
  - `cargo test -p phalcom-semantic --test semantic static_constants`: 9 passed.
  - `cargo test -p phalcom-core --test core language::algebraic_data`: 45 passed (19 gated).
  - `cargo test -p phalcom-core --test core execution_static_const`: 9 passed.

### Next plan transition

`LANG007.C4.P3` is closed. Next plan is `LANG007.C4.P4` (Generalized `FieldKey`
and unordered-product calculus) proceeding concurrently in a separate worktree.
