---
id: LANG007.C4
category: LANG
program: LANG007
checkpoint: LANG007.C4
kind: checkpoint-record
title: "Structural Static Upgrade"
status: IN_PROGRESS
completion: NOT_STARTED
verification: UNVERIFIED
requires:
  - LANG007.C1
  - LANG007.C2
  - LANG007.C3
prepared_against_revision: 8c3a98aab41ed8631e9f8746a6b29984cf77581d
prepared: 2026-10-04
active_plan: LANG007.C4.P1
latest_completed_plan: none
next_plan: LANG007.C4.P2
---

# LANG007.C4 — Checkpoint Record

## Structural Static Upgrade

## 0. Checkpoint state

```yaml
program: LANG007
checkpoint: LANG007.C4
status: IN_PROGRESS
completion: NOT_STARTED
verification: UNVERIFIED
active_plan: LANG007.C4.P1
latest_completed_plan: none
next_plan: LANG007.C4.P2
next_action: >
  Resume LANG007.C4.P1 after the T8a compiler-consumer slice. First run the
  outstanding T7 explicit-field gates and classify their results. Then inspect
  and finish T8 immutable-field lifecycle/codegen coverage, run G1, complete
  T10 incremental/performance evidence, run G2, and close T11 records. T8a's
  focused runtime consumer test passed; its broader semantic/incremental gates
  and all P1 closure gates remain outstanding. Do not begin P2.
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

---

# 2. Plans ledger

The intended C4 decomposition is:

| Plan | Scope | Status | Completion | Verification | Outcome |
|---|---|---|---|---|---|
| `LANG007.C4.P1` | Binding, mutability, field lifecycle, corpus cutover | IN_PROGRESS | PARTIAL | UNVERIFIED | active |
| `LANG007.C4.P2` | Static value model, static definitions, evaluator foundation, closed `const` | PROPOSED | NOT_STARTED | UNVERIFIED | planned |
| `LANG007.C4.P3` | First-class Pack and call/rest convergence | PROPOSED | NOT_STARTED | UNVERIFIED | planned |
| `LANG007.C4.P4` | Generalized `FieldKey` and unordered-product calculus | PROPOSED | NOT_STARTED | UNVERIFIED | planned |
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

Conceptually:

```rust
StaticValueId
StaticValueData
StaticEvalOutcome
StaticDefinition
StaticEvalContext
MaterializationDescriptor
```

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

```text
first-class Pack
PositionalPack
LabeledPack
CompletePack
one-step PackView
flat/sliced runtime/static backing
rest capture -> Pack
spread lowering -> fused Pack segments
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

Future plan gates must be added only after actual execution.

Do not record designed coverage as executed verification.

---

# 7. Deferred / baseline issue ledger

### Observed Baseline Test Failures (Revision 8c3a98aab41ed8631e9f8746a6b29984cf77581d)

1. **`semantic::capabilities::fields::field_facts_survive_constructor_and_general_writes`**
   - **Classification**: D (baseline / unrelated to current change)
   - **Observed**: Assertion `left == right` failed for binding `counter` (left: `None`, right: `Some(TypeId(779))`) in multi-class analysis fixture.
   - **Why non-blocking**: 17 other field capability tests pass; this test expects multi-class constructor type propagation in a single-file test harness.
   - **Action**: Do not fix during unrelated tasks; re-evaluate when field lifecycle tests are updated.

2. **`bcat_08_root_alias_does_not_create_synthetic_declaration` (phalcom-modules)**
   - **Classification**: D (baseline / unrelated)
   - **Observed**: `UnresolvedImport { module: ModuleId { project: Universe, path: collections.list }, path: "universe.collections.iterator_laws" }`.
   - **Why non-blocking**: Unit catalog link failure in Universe collections path, unrelated to binding/field AST.
   - **Action**: Record and leave undisturbed.

### Design-deferred items

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

## Remaining P1 closure steps at pause

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

The next action is:

```text
Resume LANG007.C4.P1 by running the outstanding T7 explicit-field focused
gates, then finish T8 lifecycle/codegen acceptance and its semantic/incremental
checks. T8a compiler-consumer implementation is focused-tested. Continue with
G1, T10, G2, and T11 in plan order; do not begin P2.
```

Required artifacts:

```text
LANG007.C4.P1-requirements-analysis.md
LANG007.C4.P1-luna-patch-grade-implementation-plan.md
```

P1 implementation must begin by following its T0 takeover requirements:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Then re-read:

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/luna-implementer-prompt.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/commit-and-push-discipline.md

LANG007.C3-CHECKPOINT.md
LANG007.C3.P5-handoff.md

LANG007.C4-CHECKPOINT.md
LANG007.C4-GUIDANCE.md
LANG007.C4.P1-requirements-analysis.md
LANG007.C4.P1-luna-patch-grade-implementation-plan.md
```

Do not begin P2 until P1's static/runtime declaration boundary and field
lifecycle are durably implemented and handed off.
