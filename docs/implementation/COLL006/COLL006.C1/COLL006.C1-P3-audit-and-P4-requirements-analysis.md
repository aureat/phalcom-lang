# COLL006.C1 — P3 Implementation Audit and P4 Requirements Analysis

**Program:** COLL006  
**Checkpoint:** COLL006.C1  
**Audit target:** `aureat/phalcom-lang`, branch `feat/telos-collection-upgrade`  
**Audited live branch head:** `298084a44968402fc47d826ee9f99df1c9205e6a` — `feat(COLL006): stage sequence products by inference owner`  
**Audit date:** 2026-10-08  
**Status recommendation:** **P3 remains `IN_PROGRESS / PARTIAL`; P4 remains `PROPOSED / NOT_STARTED`**  
**Purpose:** audit the current P3 implementation and lifecycle record, identify correctness and completeness gaps, reconstruct C1-wide constraints, and establish implementation-grade requirements for P4 without beginning P4 while P3 acceptance is incomplete.

---

## 1. Executive assessment

P3 has advanced materially beyond the initial partial Vector delivery. The current branch contains the exact-static-integer prerequisite, heterogeneous static generic application support sufficient for several concrete and symbolic Vector forms, source-owned `Vector<T, N: Int>` identity, neutral sequence-literal checking, direct Vector lowering and immutable runtime storage, bounded exact spread proofs, symbolic identity-spread copying, GC integration, structural equality/hash, and substantial focused semantic/runtime coverage.

That progress is real. It is not, however, sufficient to close P3.

The current implementation has **four blocking correctness/lifecycle conditions** before P4 can legitimately begin:

1. **Inference-owned sequence publication is not integrated.**  
   Commit `298084a` adds `checker/sequence_publication.rs`, but the helper is not wired into `CheckingContext`, sequence synthesis, declaration owners, call/generic inference, union inference, static-expression extraction, probes, or final artifact publication. Production sequence synthesis still writes directly to `ctx.sequence_literals`, and static checking still snapshots that map before final expected-type inference settlement. The new transaction model is currently scaffolding, not active semantics.

2. **Negative concrete Vector extents are not rejected at canonical type formation.**  
   `TypeStore::apply_static_form_with_arena` validates application shape/kind but has no Vector-specific nominal well-formedness validation. Scoped application also lacks the rule. As a result, the required invariant “closed `Vector<T, N>` is well formed only when a concrete `N >= 0`” is not centralized at the canonical formation boundary.

3. **`Array<T>`'s public API still leaks `Dynamic` and permits heterogeneity.**  
   The authored Universe declaration still exposes `rawAt -> Dynamic`, `rawSet(..., Dynamic)`, and public methods without homogeneous return/replacement annotations. This is not merely an ergonomics defect: it weakens the semantic guarantee that `Array<T>` reads as `T` and only accepts replacements assignable to `T`.

4. **Vector structural hashing is not safe for mutable descendants under current Map/Set key admission.**  
   Map/Set reject a mutable collection when the **top-level key** is mutable, but a `Vector` containing an `Array`, `List`, `Map`, `Set`, or other content-unstable descendant is accepted. Vector hash is structural and therefore can change after the descendant mutates. That can invalidate hash-table lookup invariants. P3 explicitly requires the mutable-descendant hash behavior to be decided and tested; it remains unresolved.

In addition, P3 still lacks required metadata/reflection/import-export and cold/incremental parity evidence, general Iterable spread coverage, several wrong-kind/arity/scoped-formation cases, and runtime evaluation/failure edge tests.

There is also a **branch-convergence prerequisite** before P4 migration work should begin. The audited branch is currently **5 commits ahead and 14 commits behind `main`**, with merge base `61c8734f752c8ffc252b6d4d4ba378c2bfe2b2b1`. The upstream changes overlap several files that P4 necessarily touches, including semantic expression/statement checking, parser code, Universe primitives, native-surface generation, runtime primitives, and the C1 planning documents. A P4 bare-bracket census or static-materialization design performed against the stale branch would not be a reliable repository-wide migration inventory.

The correct sequencing is therefore:

```text
finish P3 semantic invariants
        ↓
finish P3 runtime/durability evidence
        ↓
reconcile P3 lifecycle records
        ↓
converge feat/telos-collection-upgrade with current main
        ↓
rerun P3 acceptance gates on converged tree
        ↓
freeze P4 migration census + decision amendment
        ↓
implement P4 static sequences/materialization
        ↓
atomic bracket-default cutover
```

P4 is nonetheless sufficiently specified at the C1 architectural level to prepare a detailed requirements analysis now. The remainder of this document does so while treating all P3 dependencies as entry gates rather than assuming them away.

---

# Part I — Audit basis and repository state

## 2. Sources and authority

This audit used four classes of evidence.

### 2.1 Supplied lifecycle records

The supplied records establish the intended P3 contract and the user's stated remaining work:

- `COLL006.C1.P3-implementation-plan.md`
- `COLL006.C1.P3-handoff.md`
- `COLL006.C1.P3-walkthrough.md`
- `COLL006/PROGRAM.md`
- `COLL006/STATUS.md`

Their key architectural boundary is consistent:

> P1/P2/P3 leave bare brackets defaulting to List; P4 owns static sequence evaluation/materialization and the atomic default cutover.

The supplied P3 handoff also explicitly leaves negative extent formation, general inference-owned publication, Array API typing, durable metadata/incremental evidence, runtime edge cases, and wider semantic coverage open.

### 2.2 Live branch

The live branch is authoritative for implementation truth:

```text
repository: aureat/phalcom-lang
branch:     feat/telos-collection-upgrade
head:       298084a44968402fc47d826ee9f99df1c9205e6a
```

Relevant recent commits:

| Commit | Meaning |
|---|---|
| `5fee70c9` | P2 anchor |
| `e3dfe492` | initial substantial P3 Vector foundation |
| `e5e4ac20` | P3 sequence-construction continuation |
| `298084a4` | publication-lifecycle helper scaffolding |

### 2.3 C1 architecture records

The live C1 master/detailed records were reviewed, especially:

- `COLL006.C1-CHECKPOINT.md`
- `COLL006.C1-master-implementation-record.md`
- `COLL006.C1-detailed-implementation-plan.md`
- `COLL006.C1.P3-requirements-analysis.md`
- `COLL006.C1.P3-inference-publication-plan.md`

These records are important because P4 is not an isolated feature. It is the convergence phase between:

```text
P1 contextual literal/inference foundation
P2 Array
P3 Vector
       ↓
P4 static sequences + bracket default cutover
       ↓
P5 Slice / Buffer
```

P4 must preserve assumptions consumed later by P5–P9.

### 2.4 Live code surfaces

The audit sampled and cross-checked the main implementation boundaries:

```text
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/checker/sequence_literal.rs
phalcom-semantic/src/checker/sequence_publication.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/static_body.rs

phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/materialization.rs
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/static_definition.rs

phalcom-core/core/universe/src/collections/array.ph
phalcom-core/core/universe/src/collections/vector.ph
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/modules/static_constant.rs
phalcom-core/src/primitive/array.rs
phalcom-core/src/primitive/vector.rs
phalcom-core/src/primitive/mod.rs
phalcom-core/src/heap/array.rs
phalcom-core/src/heap/vector.rs

phalcom-semantic/tests/semantic/foundations/coll006_c1_p3_static_generics.rs
phalcom-semantic/tests/semantic/foundations/coll006_c1_p3_publication.rs
phalcom-core/tests/core/collections/vector.rs
```

No fresh local Cargo execution is claimed by this audit. Command results quoted below are recorded repository evidence, not commands executed by this auditor.

---

## 3. Branch and lifecycle state

### 3.1 Live branch has advanced beyond the supplied handoff

The supplied handoff describes a continuation from `e3dfe492` and lists inference-owned publication as future work. The live branch has since gained:

```text
e5e4ac20 feat(COLL006): continue P3 sequence construction
298084a4 feat(COLL006): stage sequence products by inference owner
```

Therefore, the supplied handoff is useful historical context but cannot be treated as the current code-state ledger.

### 3.2 `STATUS.md` is materially stale

The live branch `docs/implementation/COLL006/STATUS.md` still says:

```text
P3 is paused at the user's request.
...
The runtime test command was aborted during compilation.
```

That is contradicted by the later P3 walkthrough/checkpoint continuation record, which records:

```text
COLL006 semantic filter: 27 passed
core Vector suite:       8 passed
core Array suite:       10 passed
native surface:          current at 406 records
workspace all-targets:   passed
```

This is not cosmetic. The repository's declared lifecycle contract says `STATUS.md` summarizes program state while the checkpoint owns detailed truth. A summary that still reports a superseded aborted-runtime state makes the program index unreliable.

**Required P3 closure action:** update `STATUS.md` from the final accepted P3 checkpoint, not from the old `e3dfe492` snapshot.

### 3.3 The checkpoint does not yet account for `298084a`

The P3 publication supplement is still described as proposed/not implemented in the lifecycle narrative, while `298084a` has added a helper implementation. Conversely, because the helper is unintegrated, it would be equally incorrect to mark the publication task implemented.

The correct lifecycle classification at the audited head is approximately:

```text
P3 inference-owned publication:
    architecture:   specified
    helper substrate: partially implemented
    production integration: not implemented
    verification:   not established after 298084a
```

P3 closure should preserve this distinction rather than turning the existence of `sequence_publication.rs` into an implementation-complete claim.

### 3.4 No current CI evidence for `298084a`

The GitHub connector returned no workflow runs associated with the audited head. The combined-status endpoint was not accessible to the connector, so no claim about external status checks can be made.

The focused P3 command ledger predates the latest publication helper commit. The current head therefore needs at minimum a post-`298084a` compile/focused-test gate before even its unchanged runtime behavior can be considered reverified.

### 3.5 Branch divergence is now material to P4

Relative to current `main`:

```text
merge base: 61c8734f752c8ffc252b6d4d4ba378c2bfe2b2b1
feature branch: 5 commits ahead
feature branch: 14 commits behind
```

Overlapping paths changed both upstream and on the collection branch include:

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1-detailed-implementation-plan.md
phalcom-ast/src/parser.rs
phalcom-core/src/primitive/mod.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/src/value/render.rs
phalcom-modules/src/builtin.rs
phalcom-native-surface/src/generated.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/statement.rs
```

These are not incidental P4 files. Parser semantics, phase-sensitive checking, static definitions, Universe exposure, native metadata, and repository migration are all P4-critical. A migration census against the unconverged branch is therefore provisional by definition.

---

# Part II — C1 architectural reconstruction

## 4. C1's actual ownership

C1 is a multi-plan first-generation collection program, not merely an Array/Vector feature set.

Its sequence-related layers are:

| Layer | C1 owner | Semantic responsibility |
|---|---|---|
| literal syntax / neutral AST | P1 | parser describes syntax, never hard-wires family |
| contextual selection / holes | P1 | expected-type + explicit-head family selection; `_` inference |
| runtime fixed sequence | P2 | `Array<T>` |
| exact immutable sequence | P3 | `Vector<T,N>` |
| static values + defaults | P4 | const Array/Vector + runtime/static bare default cutover |
| contiguous views/builders | P5 | `Slice<T>`, `Buffer<T>` |
| mutable ordered hash family | P6 | `MutableBag`, `IndexedDict`, `OrderedSet` |
| persistent hash family | P7 | `Bag`, `Bijection` |
| intervals/ordered persistence | P8 | `Interval`, `RangeMap` |
| convergence/certification | P9 | complete API/migration/performance closeout |

This scope matters for P4 in two ways.

First, P4 is the last phase allowed to change what ordinary `[...]` means. P5 onward should consume stable sequence literal semantics, not participate in another default migration.

Second, P4 must not absorb later collection architecture. `Slice`, `Buffer`, persistent storage, ordered hashes, and generalized collection conversion APIs remain later owners.

---

## 5. Ratified literal selection precedence

The C1 architecture establishes the following precedence:

```text
1. explicit literal head
2. unique explicit/contextual expected family
3. phase default
```

Examples after P4:

```telos
<Int>[1, 2, 3]
// Array<Int>, explicit one-argument head

<Int, _>[1, 2, 3]
// Vector<Int, 3>, explicit two-argument head

let x: List<Int> = [1, 2, 3]
// List<Int>, contextual family overrides runtime default

let x: Array<Int> = [1, 2, 3]
// Array<Int>

let x: Vector<Int, 3> = [1, 2, 3]
// Vector<Int, 3>

let x = [1, 2, 3]
// Array<Int>, runtime phase default

const x = [1, 2, 3]
// Vector<Int, 3>, static phase default
```

A phase default is **not an overload-resolution tie-breaker**. Where both Array and Vector are viable overload candidates, an unheaded bracket literal remains ambiguous.

This invariant is already encoded in the program architecture and must be certified explicitly in P4.

---

## 6. C4 is an authority, not an optional implementation detail

C1 consumes C4 for:

```text
StaticValueStore
static evaluator
static constant identity
query budgets / cancellation
static-value typing
materialization descriptor ownership
cross-module static constant identity
incremental/static query ownership
```

The live static-value store currently supports:

```rust
StaticValueData::{
    Unit,
    Bool,
    Char,
    Int,
    Float,
    String,
    Symbol,
    Type,
    StaticTerm,
    Tuple,
    Record,
    Data,
    Variant,
    Pack,
}
```

There is no homogeneous static sequence node.

The static evaluator has explicit work items for Tuple, Record, data/variant construction, static applications, control flow, and primitive operations. `Expr::BracketLiteral` currently falls into `UnsupportedExpression`.

The materialization layer currently supports primitive descriptors and classifies all structural values as deferred:

```rust
pub enum MaterializationDescriptor {
    Unit,
    Bool(bool),
    Char(char),
    Int(StaticInt),
    Float(StaticFloat),
    String(Arc<str>),
    Symbol(Arc<str>),
    DeferredStructural(DeferredStructuralKind),
    Unsupported(String),
}
```

and runtime constant materialization rejects `DeferredStructural`.

Therefore P4 must extend the **existing C4-owned path**. It must not:

- re-run static initializer AST in the compiler or VM;
- serialize runtime `Value`s as compile-time identity;
- create a collection-only static arena;
- encode Array/Vector as Pack;
- bypass `StaticConstantProduct`;
- add a second evaluator for bracket literals.

---

# Part III — P3 implementation audit

## 7. P3 requirement-by-requirement status

The user's seven remaining items remain the correct closure structure. Their live status is:

| Remaining P3 item | Live status | Audit verdict |
|---|---|---|
| 1. centralized negative extent rejection | not implemented | **BLOCKER** |
| 2. inference-owned publication | helper scaffold only | **BLOCKER** |
| 3. Array API typing | not implemented | **BLOCKER/HIGH** |
| 4. semantic coverage completion | partial | **OPEN** |
| 5. runtime edge coverage | partial | **OPEN; hash policy BLOCKING** |
| 6. durable identity | incomplete | **OPEN** |
| 7. close P3 | impossible yet | **OPEN** |

---

# 8. Finding P3-F01 — negative concrete Vector extents are not rejected centrally

**Severity:** BLOCKER  
**Category:** type formation / canonical identity  
**Requirement:** concrete negative `N` must be rejected whenever a closed Vector application becomes publishable.

The core application path in `phalcom-semantic/src/types/store.rs::apply_static_form_with_arena` validates:

- origin kind;
- application arity;
- destination parameter kind/domain;
- static argument category;

and then interns `TypeData::Applied`.

It does not enforce the nominal invariant:

```text
if origin == canonical Vector
and N is concrete ConstInt
then N >= 0
```

Likewise, scoped application formation in `types/annotation.rs` can retain an applied Vector with a concrete negative term.

The problem cannot be repaired in `sequence_literal.rs`: an explicit type annotation or imported/scoped application can form Vector without constructing a literal.

## 8.1 Required architecture

Introduce one canonical nominal-application well-formedness boundary, conceptually:

```rust
fn validate_nominal_application(
    origin: TypeId,
    arguments: &[StaticArgument],
    store: &TypeStore,
) -> Result<(), TypeApplicationError>
```

The exact API may differ, but the invariant must be centralized.

The Vector rule is bounded:

```text
canonical Vector<T, N>
    N symbolic                  -> valid formation
    N == 0                      -> valid
    N > 0                       -> valid
    N < 0 and concrete          -> reject
```

Do **not** impose runtime allocation bounds at type formation. Exact huge positive `N` remains a valid type identity even when a value of that size cannot be materialized.

## 8.2 Required producer coverage

The validator must be reached from all closed/publication-producing paths:

```text
ordinary source application
partial application completion
scoped application materialization
alias beta-reduction / substitution
inference materialization
imported metadata reconstruction
associated/type-member application if it can produce Vector
```

Raw store interning should remain a low-level canonicalization primitive rather than becoming a fallible semantic validator.

## 8.3 Tests

Required hostile cases:

```telos
Vector<Int, -1>             // reject
Vector<Int, 0>              // accept
Vector<Int, 999...huge>     // type identity accepted

type Bad = Vector<Int, -1>  // reject through alias formation

type V<N: Int> = Vector<Int, N>
// symbolic declaration acceptable

V<-1>                       // reject when concretized

let x: Vector<Int, -1> = ...
// reject before literal lowering

// equivalent scoped/import/substitution cases
```

Also verify that imported metadata cannot publish a negative closed Vector even if the generic metadata graph is structurally well-formed.

---

# 9. Finding P3-F02 — the new sequence publication transaction is inactive

**Severity:** BLOCKER  
**Category:** semantic publication / inference lifetime

Commit `298084a` adds:

```text
checker/sequence_publication.rs
```

with:

```rust
PublicationOwner(InferenceContextId, InferenceFrameId)
SequencePublication {
    owners,
    pending,
    expression_dependencies,
}
```

and operations:

```text
register
probe
stage
local_resolution
discard_expression
locally_ready
reject
dispose
flush
```

This is directionally consistent with the approved P3 supplement.

It is not connected to the checker.

## 9.1 Evidence of non-integration

`CheckingContext` still owns only the committed:

```rust
sequence_literals: SequenceLiteralResolutionIndex
```

There is no `SequencePublication` field.

`sequence_literal.rs::synthesize` still performs:

```rust
if valid && let Some(expression) = ctx.current_expression_id() {
    ctx.sequence_literals.insert(expression, SequenceLiteralResolution { ... });
}
```

That means locally valid sequence construction facts become compiler-visible immediately.

`expression.rs::analyze_expression` adds the outer `ExpectedType::Inference` subtype constraint **after** `analyze_expression_inner` returns. Therefore the sequence literal can already have been inserted before the outer owner receives its final constraint.

`context.rs::finish_inference_context` still disposes solver state without issuing a sequence-publication acceptance/rejection receipt.

`statement.rs`, `call.rs`, borrowed generic child flows, and union generic scopes do not call the new helper.

`static_body.rs` currently snapshots:

```rust
let sequence_literals = context
    .sequence_literals
    .iter()
    ...
    .collect();
```

before the later `solved_expected_type` inference solve.

This is the exact extraction-order defect described by the publication supplement.

## 9.2 Why the current focused publication tests are insufficient

The branch contains source tests such as:

```text
late_generic_failure_discards_dependent_sequence_and_preserves_independent_sibling
successful_generic_and_partial_annotations_commit_sequences
```

The previously recorded `coll006` semantic suite passed before the latest helper commit.

Those tests are useful discriminators, but they cannot establish that the new transaction abstraction is live. A narrow source case may pass because a particular surrounding call/statement path already isolates or drops analysis products for unrelated reasons.

P3 needs tests that specifically exercise each publication owner path named by the approved supplement.

## 9.3 Required integration contract

A sequence resolution must have exactly one of these lifetimes:

```text
locally formed
    ↓
pending under owner dependencies
    ↓
all required owners accepted
    ↓
committed executable product
```

or:

```text
locally formed
    ↓
pending
    ↓
any required owner rejected/cancelled/blocked/underconstrained
    ↓
discarded
```

No pending entry may reach:

```text
CheckedStaticExpression
CheckedStaticBody
callable artifact
module lowering
compiler sequence index
metadata / fingerprint output
```

## 9.4 Required owner integrations

At minimum:

| Owner path | Required behavior |
|---|---|
| explicit literal-head inference | local owner settles only after head materialization + sequence checks |
| declaration partial annotation | accept only after final annotation materialization and initializer reconciliation |
| ordinary generic call | call owner settles after final argument/result constraints |
| borrowed/deferred generic child | child local readiness does not bypass parent acceptance |
| union shared inference | argument product survives real branch reuse but commits only after aggregate owner acceptance |
| static expected owner | settle before copying sequence index |
| flow probe | fresh pending products; no mutation of parent publication state |
| callable/static finalization | reject unresolved pending state; export committed map only |

## 9.5 Review of the helper itself

Even after wiring, `SequencePublication` deserves hardening tests.

Potential hazards:

- `stage()` silently replaces a pending payload for the same `ExpressionId`;
- `register()` uses `or_insert`, so a later discovered parent is ignored;
- a dependency on an unregistered owner is treated as rejected immediately;
- `discard_expression()` can remove only pending state, not an already committed product;
- rejected state carries no reason even though the architecture document specified `Rejected(reason)`;
- the helper currently has no dedicated unit tests in the `298084a` commit.

These are not necessarily wrong choices, but their preconditions must be made explicit and tested.

For example, if “all dependencies are fully captured before `stage`” is an invariant, duplicate staging and late-parent registration should be asserted as internal errors rather than silently tolerated.

---

# 10. Finding P3-F03 — Array API typing is semantically incomplete

**Severity:** HIGH / P3 acceptance blocker  
**Category:** public API typing

Current `array.ph` begins:

```telos
native class Array<T> is Collection {
  internal native rawLength -> Int
  internal native rawAt(_ index: Int) -> Dynamic
  internal native rawSet(_ index: Int, _ value: Dynamic) -> Dynamic
```

and exposes unannotated public reads/replacements:

```telos
at(_ i) { ... }
get(_ index) { ... }
[_ index] { ... }

at(_ i, put) { ... }
[_ i]=(_ val) { ... }
```

This defeats the type parameter's public semantic purpose.

## 10.1 Required public contract

P3 must preserve current bounds behavior while making the homogeneous contract explicit.

A coherent surface is:

```telos
// permissive read: current Array#at behavior
at(_ i: Int) -> T | None

// Option-returning read
get(_ i: Int) -> Option<T>

// strict read for implementation/subscript
private checkedAt(_ i: Int) -> T
// throws IndexError when out of range

[_ i: Int] -> T

// replacement
at(_ i: Int, put value: T) -> Array<T>
[_ i: Int]=(_ value: T) -> Array<T>   // or assignment protocol's established result contract
```

The exact naming of the strict helper is flexible.

If `T | None` is not accepted in this authored native surface, P3 must resolve the representation through the existing type system rather than lying with `Dynamic`.

## 10.2 Native floor

It is acceptable for the irreducible raw native boundary to traffic in `Value` internally, but authored native metadata should not advertise a `Dynamic` replacement parameter if the public semantic contract is homogeneous.

The implementation must preserve:

- negative-index normalization;
- existing out-of-range behavior;
- frozen-array write rejection;
- no resize operation.

## 10.3 Required tests

```telos
let a: Array<Int> = <Int>[1, 2]
let i: Int = a[0]             // type Int

a[0] = 3                      // valid
a[0] = "x"                    // semantic rejection

a.at(999)                     // T | None under chosen contract
a.get(999)                    // Option<T>
a[999]                        // IndexError
```

Regenerate and `--check` the native surface only after the authored signature is corrected.

---

# 11. Finding P3-F04 — mutable-descendant Vector hashing can violate Map/Set laws

**Severity:** BLOCKER until semantic policy is decided  
**Category:** runtime hash/equality law

P3 now permits Vector as a structural Map/Set key. The current top-level mutable-key guard rejects:

```text
List
Array
Map
Set
Bytes
...
```

when those values themselves are used as keys.

It does not recursively reject a `Vector` containing one of those values.

Vector hash is structural: it calls element hash operations. Therefore this sequence is potentially unsound:

```telos
let child: Array<Int> = <Int>[1, 2]
let key: Vector<Array<Int>, 1> = <Array<Int>, 1>[child]

let map = Map.new()
map[key] = "value"

child[0] = 9

// key's structural hash may now differ from the bucket chosen at insertion.
map[key]
```

Container immutability is not deep value immutability.

## 11.1 P3 must make one explicit policy choice

Acceptable directions include:

1. **deep key-admission stability check**  
   Reject Vector keys whose reachable hash-relevant descendants are mutable/content-unstable.

2. **conditional hashability semantics**  
   Vector is hashable only when element values/types satisfy a stable-hashability contract. This is architecturally larger and may depend on a future hashability trait, so it is probably too broad for P3.

3. **conservative Vector-key prohibition for now**  
   Do not admit Vector as Map/Set key until a stable content policy exists.

What is not acceptable is the current implicit rule:

```text
immutable outer container => stable structural hash
```

because it is false for mutable descendants.

P3 should not invent a language-wide hashability redesign. If the correct bounded rule cannot be implemented without such a redesign, record a consultation and choose a conservative key-admission policy.

---

# 12. Finding P3-F05 — semantic formation coverage is still incomplete

**Severity:** MEDIUM/HIGH  
**Category:** semantic test completeness

The current tests are strong on:

- exact integer representation;
- signed/huge static integer identity;
- symbolic binders;
- mixed holes;
- explicit/contextual Vector forms;
- extent mismatch;
- Array/List runtime-sized spread rejection;
- family conflict;
- symbolic identity Vector spread.

Remaining required coverage includes:

```text
wrong first argument kind:      <3, 3>[...]
wrong second argument domain:   <Int, String>[...]
wrong arity in all application paths
partial/scoped application formation
alias/substitution formation
general Iterable spread into Vector
generic-return contextual construction
nested inference contexts
empty Vector with inferred N
empty Vector with underconstrained T
large exact N type identity
concrete zero extent
cold/incremental change N=3 -> N=4
exact spread -> runtime-unknown spread edit
```

General Iterable deserves its own test rather than assuming Array/List rejection proves the protocol case.

---

# 13. Finding P3-F06 — bounded symbolic Vector construction is coherent but narrow

**Severity:** INFO / coverage caution

The current symbolic construction rule is intentionally bounded:

```text
Vector identity spread:
    one spread
    source is canonical Vector
    source extent == result extent
    no additional concrete elements
```

This is represented as:

```rust
SequenceConstructionPlan::VectorIdentitySpread {
    source_range,
    extent,
}
```

The compiler revalidates:

- result extent;
- exact expansion mode;
- source site;
- lone-spread shape;

and emits `CopyVector`.

This is a good fail-closed boundary.

The implementation correctly does **not** pretend it can prove symbolic arithmetic such as:

```text
N + 1
N + M
```

However, the semantic checker should ensure every rejected multiple-symbolic case receives a deterministic diagnostic. At least one internal branch currently sets `valid = false` for a second symbolic extent without immediately producing a dedicated explanation. That is a diagnostics-quality issue worth covering before P3 closure.

---

# 14. Finding P3-F07 — lowering is mostly defensive, but large-count boundaries need explicit tests

**Severity:** MEDIUM  
**Category:** lowering/resource boundary

Projection in `semantic_lowering.rs` converts a concrete extent to `u32` using a checked conversion:

```rust
TypeData::ConstInt(value) => u32::try_from(value.to_bigint()).ok()
```

Compiler Vector construction then fails closed if no host-addressable concrete extent exists.

This is correct separation:

```text
type identity can remain arbitrary precision
runtime allocation count must be host/addressable
```

There is one weaker path in semantic synthesis:

```rust
SequenceConstructionPlan::Fixed {
    element_count: literal.elements.len() as u32,
}
```

That cast should be changed to a checked conversion or guarded by a semantic resource limit. A source file with more than `u32::MAX` literal operands is unrealistic in practice, but the code should not encode silent truncation in a proof product.

Required boundary tests should distinguish:

```text
huge Vector type formation                 -> valid
attempted huge concrete materialization    -> controlled failure
no integer truncation                      -> proven
zero extent                                -> works
```

---

# 15. Finding P3-F08 — evaluation-order and failure-cleanup coverage is not yet adequate

**Severity:** MEDIUM  
**Category:** VM semantics

The compiler's fixed Vector path compiles ordinary operands in source order and `BuildVector` reverses popped stack values to restore source ordering.

The builder path also visits AST elements in source order.

That looks correct by inspection, but P3 requires behavioral proof.

Add an effect probe such as:

```telos
let log = List.new()

fn mark(_ n: Int) {
  log.append(n)
  n
}

let v = <Int, 3>[mark(1), mark(2), mark(3)]
// assert log == [1,2,3], each once
```

Include spreads whose operand expression has a side effect.

Failure coverage must include:

```text
ordinary operand throws after earlier elements were staged
spread operand throws
spread iteration/projection fails
construction is cancelled/suspended if that path is legal
GC occurs during operand evaluation
```

The private builder must never become guest-visible and must be reclaimable after frame unwinding.

Current tampered-cardinality tests prove defensive invariant rejection; they do not substitute for source-level failure cleanup.

---

# 16. Finding P3-F09 — exact spread currently copies through a temporary Vec

**Severity:** LOW/PERFORMANCE  
**Category:** runtime implementation

VM exact Vector expansion currently performs approximately:

```rust
vector.elements().to_vec()
```

before appending the values into the builder.

This is semantically safe, but it allocates a temporary proportional to the spread size.

P3's main storage requirement is that finalization move the builder payload rather than clone the completed sequence, which the implementation satisfies. Therefore this is not a P3 correctness blocker.

It should, however, be recorded as a possible P9/performance refinement. A future lane could append from rooted storage without an intermediate `Vec` if GC/re-entrancy constraints allow it.

---

# 17. Finding P3-F10 — durable identity evidence is incomplete

**Severity:** HIGH  
**Category:** metadata/reflection/incrementality

P3 still needs proof that:

```text
Vector generic parameter 0 => kind Type
Vector generic parameter 1 => static value domain Int
Vector<Int,3> != Vector<Int,4>
large/signed exact terms preserve metadata identity
imports/re-exports preserve the same nominal Vector owner
reflection reports the static-domain parameter correctly
```

Incremental tests must edit actual governing facts, not merely re-run an unchanged query:

```text
<Int,3>[1,2,3]
    ↓ edit
<Int,4>[1,2,3,4]
```

and:

```text
exact Tuple/Vector spread
    ↓ edit
runtime-sized Array/List/general Iterable spread
```

Then compare cold and incremental:

- canonical formatted result type;
- construction plan;
- exact spread facts;
- diagnostics;
- relevant fingerprints/dependencies.

Do not compare raw `TypeId` numbers across independently built stores.

---

# 18. Finding P3-F11 — lifecycle records are not reconciled

**Severity:** HIGH for process integrity  
**Category:** implementation lifecycle

Before P3 acceptance:

```text
P3 implementation plan
P3 inference-publication supplement
P3 walkthrough
P3 handoff
C1 checkpoint
COLL006 STATUS
COLL006 PROGRAM
```

must agree on:

- final head/anchor;
- which publication tasks actually landed;
- final verification commands;
- remaining approved deferrals;
- whether Array API typing is closed;
- negative extent rule owner;
- hashability/deep mutable policy;
- no P4 implementation yet.

The current state has a known contradiction between `STATUS.md` and later walkthrough/checkpoint evidence.

---

# Part IV — P3 acceptance gate before P4

## 19. Minimum P3 closure checklist

P4 must not begin until all of the following are true.

### Type formation

- [ ] concrete negative Vector extent rejected centrally;
- [ ] zero extent accepted;
- [ ] arbitrary-precision positive extent retains exact identity;
- [ ] aliases/scoped applications/substitution/materialization cannot bypass validation;
- [ ] wrong-kind and wrong-arity matrix complete.

### Inference/publication

- [ ] sequence products staged rather than directly committed when they depend on unresolved owners;
- [ ] declaration owner integrated;
- [ ] explicit-head owner integrated;
- [ ] generic call/deferred child owner integrated;
- [ ] union shared owner integrated;
- [ ] flow-probe isolation tested;
- [ ] static extraction occurs after final owner settlement;
- [ ] unresolved/cancelled/blocked owner exports no sequence product;
- [ ] no provisional owner/gate state enters durable artifacts.

### Array

- [ ] public reads carry homogeneous `T`/`Option<T>` semantics;
- [ ] public replacement argument is `T`;
- [ ] wrong-type replacement rejected semantically;
- [ ] bounds behavior unchanged;
- [ ] native surface regenerated and checked.

### Vector runtime

- [ ] operand evaluation once and in order;
- [ ] zero extent runtime;
- [ ] large extent/resource boundary;
- [ ] source failure cleanup;
- [ ] GC during construction;
- [ ] missing symbolic proof rejected;
- [ ] inconsistent symbolic proof rejected;
- [ ] deep mutable-descendant key/hash policy ratified and tested.

### Durability

- [ ] native metadata parameter kinds/domains;
- [ ] reflection;
- [ ] import/re-export;
- [ ] metadata round-trip;
- [ ] cold/incremental parity for N edits;
- [ ] cold/incremental parity for spread-exactness edits.

### Closure

- [ ] focused semantic gate;
- [ ] focused Vector runtime gate;
- [ ] focused Array runtime gate;
- [ ] native surface `--check`;
- [ ] workspace all-targets check;
- [ ] scoped formatting/diff review;
- [ ] lifecycle records reconciled;
- [ ] current branch integrated with `main` or a documented convergence commit established;
- [ ] focused P3 gates rerun on the converged tree.

---

# Part V — P4 requirements analysis

# 20. P4 objective

P4 owns one coherent language transition:

> Introduce first-class homogeneous static sequence values using the canonical C4 static-value/evaluator/materialization infrastructure; support static/const `Vector` and frozen static/const `Array`; migrate repository source that depends on legacy bare-bracket List semantics; explicitly amend the legacy accepted List-literal decisions; then atomically switch unconstrained runtime brackets to Array and unconstrained static brackets to Vector.

P4 is not just “make const Vector work.”

Its observable acceptance surface is:

```telos
let runtimeDefault = [1, 2, 3]
// Array<Int>

const staticDefault = [1, 2, 3]
// Vector<Int, 3>

let dynamic: List<_> = [1, 2, 3]
// List<Int>

const explicitArray: Array<_> = [1, 2, 3]
// static Array<Int>, runtime materialization is frozen Array

let explicitArray2 = <_>[1, 2, 3]
// Array<Int>

let explicitVector = <_, _>[1, 2, 3]
// Vector<Int, 3>
```

---

# 21. P4 non-goals

P4 must not:

- redesign Array or Vector runtime storage;
- introduce `ConstArray` as a distinct type/class;
- make Array statically dimensioned;
- make List a default again in any phase;
- implement Slice/Buffer;
- implement a new persistent collection substrate;
- invent a general hashability trait;
- implement arbitrary symbolic Vector arithmetic;
- bypass C4 with a collection-specific constant evaluator;
- use runtime `Value` as static identity;
- encode sequence static values as Pack;
- change parser syntax to decide Array versus Vector;
- use phase defaulting to resolve overload ambiguity;
- re-run const initializer source at runtime;
- silently preserve TDR-0025/TDR-0028 text while shipping contradictory behavior.

---

# 22. P4 entry gates

## P4-G0 — P3 accepted

All requirements in §19 are mandatory.

## P4-G1 — branch convergence

Before repository census or broad migration:

1. integrate current `main` into the feature line using the repository's chosen workflow;
2. resolve overlapping semantic/native/builtin/docs changes intentionally;
3. rerun the accepted P3 focused gates;
4. update the P4 planning anchor to the converged commit.

P4 must not perform its final literal census on a branch known to be 14 commits behind.

## P4-G2 — decision authority confirmed

The current accepted decisions include:

```text
docs/decisions/accepted/0025-list-literal-syntax.md
docs/decisions/accepted/0028-collections-representation-and-literals.md
```

They explicitly describe `[...]` as List construction.

P4 needs an explicit amendment/superseding decision before final cutover acceptance.

The implementation can prepare infrastructure before that document lands, but it must not merge final contradictory executable semantics while the accepted decision remains unchanged.

## P4-G3 — C4 materialization boundary confirmed

At P4 start, re-read:

```text
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/materialization.rs
phalcom-semantic/src/static_constant.rs
phalcom-core/src/modules/static_constant.rs
```

If upstream C4 work has already generalized structural materialization, reuse it.

If not, P4 is authorized to add the smallest reusable sequence extension to the C4-owned path.

---

# 23. P4 semantic invariant set

The implementation plan should copy these invariants verbatim.

| ID | Invariant |
|---|---|
| P4-I01 | parser AST remains collection-family neutral |
| P4-I02 | semantic analysis is the only Array/List/Vector family selector |
| P4-I03 | explicit literal head outranks phase default |
| P4-I04 | unique expected family outranks phase default |
| P4-I05 | runtime unconstrained default is Array |
| P4-I06 | static unconstrained default is Vector |
| P4-I07 | phase default never resolves an overload ambiguity |
| P4-I08 | static sequence evaluation consumes a checked sequence resolution; it does not rediscover family/type/cardinality |
| P4-I09 | every static sequence element is evaluated exactly once in source order |
| P4-I10 | static sequence identity lives in `StaticValueStore`, not runtime VM storage |
| P4-I11 | Array and Vector may share static storage machinery but never nominal identity |
| P4-I12 | const Array materializes as the ordinary Array runtime class with a frozen storage state |
| P4-I13 | const Vector materializes as the ordinary immutable Vector runtime class |
| P4-I14 | runtime materialization never executes initializer AST |
| P4-I15 | one constant is materialized once per canonical runtime owner identity and reused through imports/re-exports |
| P4-I16 | direct mutation of a statically known const Array is diagnosed |
| P4-I17 | mutation through an alias/dynamic path still fails at the frozen runtime guard |
| P4-I18 | an unsupported nested structural child makes runtime materialization unavailable; it does not corrupt static identity or trigger a fallback evaluator |
| P4-I19 | no unsolved inference variable/gate is serialized into a static sequence |
| P4-I20 | cold and incremental analysis agree on sequence family, result type, static value, diagnostics, and materialization capability |
| P4-I21 | every repository-owned source location that relied on bare List semantics is explicitly classified before default cutover |
| P4-I22 | final cutover is atomic across runtime/static semantics, corpus migration, decisions, and regression tests |

---

# 24. Static sequence representation

The existing detailed C1 design is sound:

```rust
struct StaticSequenceData {
    result_type: TypeId,
    elements: Box<[StaticValueRef]>,
}

StaticValueData::Sequence(StaticSequenceId)
```

## 24.1 Why `result_type` belongs in the node

These two static values can contain identical child references but have different semantics:

```text
Array<Int>      [1,2,3]
Vector<Int,3>   [1,2,3]
```

Therefore child contents alone cannot be the interning key.

The canonical `result_type` preserves:

- family identity;
- element type;
- Vector extent;
- any future source-level nominal distinction.

This follows existing C4 precedent: static Data/Variant nodes already retain canonical result type identity.

## 24.2 Store invariants

`intern_sequence` should require:

```text
all child refs belong to this StaticValueStore lineage
result_type is a proper canonical type
result_type's nominal origin is canonical Array or Vector
all child static values are type-compatible with the homogeneous element type
if Vector extent is concrete:
    extent is nonnegative
    extent == elements.len()
```

The final two checks are defensive store invariants, not replacements for the checker.

For a static value instance, a materialized Vector's actual sequence cardinality must be concrete. A generic static definition may contain symbolic checked terms, but it should not intern a runtime-materializable sequence value whose cardinality remains unproved.

## 24.3 Equality and hashing

Static sequence equality must include family/result identity.

Conceptually:

```text
same result type
and same length
and child semantic equality elementwise
```

Do not make:

```text
static Array<Int>[1,2] == static Vector<Int,2>[1,2]
```

merely because payloads match.

## 24.4 Not Pack

Pack has different semantics:

- heterogeneous positional/labeled lanes;
- slice/backing identity;
- call/rest expansion use;
- own type domain.

A homogeneous sequence should not be forced into Pack in order to reuse its arena implementation.

Reuse coding patterns, not ontology.

---

# 25. Static evaluator integration

Current C4 evaluator support is explicit by expression kind. `BracketLiteral` is currently unsupported.

P4 should add a dedicated work path such as:

```rust
Work::FinishSequence {
    expression: ExpressionId,
    count: usize,
}
```

or equivalent.

## 25.1 Product authority

The evaluator must query:

```text
CheckedStaticExpression.sequence_literals
CheckedStaticBody.sequence_literals
```

by the exact checked `ExpressionId`.

It must not infer family from:

- number of literal-head arguments;
- expected type;
- static phase;
- runtime class names;
- child value types.

Those questions were already answered by semantic checking.

## 25.2 Evaluation order

For a checked fixed sequence:

```text
push FinishSequence
push children in reverse work-stack order
work executes source child 0, child 1, ...
```

matching existing Tuple evaluation strategy.

For checked exact expansions, the evaluator must honor the semantic construction plan rather than dispatching arbitrary runtime iteration.

This needs an explicit design for static expansion:

```text
closed static Tuple expansion
static Vector expansion
other statically representable exact lanes
```

P4 should reuse P3's exact lane proof. It must not call runtime iterator methods from the static evaluator.

## 25.3 Failure behavior

If:

- sequence resolution is missing;
- result type is not closed;
- a required spread proof is missing;
- an element evaluation is unsupported/cancelled/budget-exhausted;
- construction cardinality contradicts its checked proof;

then the evaluator fails closed under the existing static-eval outcome taxonomy.

It must not construct a plausible sequence from source syntax.

---

# 26. Static materialization design

P4 must distinguish:

```text
static semantic value
runtime materialization capability
```

A static sequence may be valid even if one of its child values currently lacks runtime structural materialization.

## 26.1 Descriptor requirement

The existing descriptor is self-contained and VM-independent. Sequence materialization must preserve that property.

One bounded shape is conceptually:

```rust
enum StaticSequenceMaterializationKind {
    ArrayFrozen,
    Vector,
}

MaterializationDescriptor::Sequence {
    kind: StaticSequenceMaterializationKind,
    elements: Arc<[MaterializationDescriptor]>,
}
```

The exact representation can be improved during implementation.

Important constraints:

- no `StaticValueRef` may be handed to the runtime VM as durable backing identity;
- descriptor construction recursively determines child materialization capability;
- failure to materialize one child does not invalidate the static semantic value.

## 26.2 Scope boundary for nested structural children

P4 must not accidentally implement all of C4's general structural materializer just because a sequence may contain arbitrary `T`.

Therefore:

```text
static evaluation:
    permit any child that C4 can represent statically

runtime materialization:
    recursively materialize child categories supported by the canonical materializer
    otherwise preserve a clear Deferred/Unsupported capability
```

At minimum P4 should certify:

- primitive elements;
- nested homogeneous sequences.

If P4's product contract requires `Vector<Record<...>>` or `Array<Data>` to be usable at runtime immediately, then P4 is blocked on a broader C4 structural materialization extension and should say so explicitly rather than adding hidden one-off logic.

---

# 27. Runtime materialization

## 27.1 Vector

A materialized const Vector is simply an ordinary runtime `VectorObject`:

```text
immutable boxed dense storage
same canonical Vector class
same equality/hash/iteration behavior
```

There must be no separate `ConstVectorObject`.

## 27.2 Array

A materialized const Array is an ordinary `ArrayObject` with:

```text
frozen = true
```

The live `ArrayObject` already has frozen-state support, and `rawSet` already rejects frozen writes. P4 should use that mechanism.

There must be no `ConstArray` classifier.

## 27.3 Materialization once

`VM::ensure_static_constant_materialized` already caches by canonical owner module/name.

Sequence materialization must fit this existing path so that:

```text
imported const
renamed import
re-export
multiple reads
```

resolve to the same owner-held materialized runtime value rather than repeatedly allocating and re-running semantics.

This is a crucial P4 test.

---

# 28. Const Array mutation semantics

P4 must enforce two layers because static typing alone cannot protect aliases.

## 28.1 Direct semantic rejection

For a statically known const binding:

```telos
const a: Array<Int> = [1, 2]
a[0] = 3
```

the checker should diagnose mutation of the const/frozen value where the existing binding/static semantics provide enough evidence.

This is not the same as ordinary `let` binding immutability. The relevant fact is the value's const materialization/frozen provenance.

## 28.2 Runtime alias defense

For example:

```telos
const a: Array<Int> = [1, 2]
let dynamic: Dynamic = a
dynamic[0] = 3
```

or any other path that erases static provenance, the existing frozen `rawSet` guard must still reject the mutation.

P4 acceptance requires both layers.

---

# 29. Phase-default cutover

Current `sequence_literal.rs` uses:

```rust
contextual_family.unwrap_or(SequenceLiteralFamily::List)
```

P4 must replace this with phase-sensitive selection only after migration readiness.

Conceptually:

```rust
match contextual_family {
    Some(family) => family,
    None => match ctx.phase {
        EvaluationPhase::Runtime => SequenceLiteralFamily::Array,
        EvaluationPhase::Static  => SequenceLiteralFamily::Vector,
    },
}
```

The exact code should preserve explicit-head handling before this branch.

## 29.1 Atomicity requirement

Do not land a long-lived state where:

```text
runtime bare brackets => Array
static bare brackets  => List
```

or:

```text
runtime defaults changed
repository List-dependent source not migrated
```

Infrastructure can be developed before cutover, but the observable default change should be one coordinated change set with:

- source migration;
- decision amendment;
- regression fixtures;
- docs;
- native/static support already green.

---

# 30. Overload ambiguity

Required hostile example:

```telos
f(_ a: Array<Int>) { ... }
f(_ v: Vector<Int,3>) { ... }

f([1,2,3])
```

This must remain ambiguous.

The checker must not say:

```text
runtime phase => choose Array overload
```

because phase default applies only after semantic context has failed to provide a unique family, not as an overload preference among viable contextual families.

Explicit headed forms disambiguate:

```telos
f(<_>[1,2,3])       // Array
f(<_,_>[1,2,3])     // Vector
```

Also test explicit contextual List overloads.

---

# 31. Repository migration census

The C1 detailed plan says P1 should create a P4 migration inventory. No dedicated current inventory artifact was found in the C1 directory or through repository search.

P4 must therefore treat the census as real work, not assume P1 completed it.

## 31.1 Census should be AST/semantic, not regex-only

Text search is useful discovery, but punctuation-based grep cannot reliably distinguish:

```text
bracket literal
subscript expression
subscript method declaration
type syntax
documentation code fence
test fixture
generated code
```

A robust census should use the parser/AST where practical and supplement with textual documentation search.

## 31.2 Classifications

Every repository-owned bare bracket literal must be classified:

| Classification | Required migration |
|---|---|
| dynamically resizes / append/remove expected | make List explicit |
| fixed runtime sequence | Array default is acceptable; update assertions/types if needed |
| static immutable sequence | Vector default is intended |
| only iterates/reads | review whether Array behavior is source compatible |
| intentionally tests legacy List semantics | rewrite as explicit List test or retire as superseded |
| ambiguous | manual review before cutover |

Do not classify a use as “needs List” merely because some downstream code happens to mutate it. Determine whether resizing is semantically required.

## 31.3 Scope

At minimum:

```text
Universe source
standard library
examples
language corpus
semantic tests
core runtime tests
compiler tests
LSP/parser fixtures
docs/spec examples
decision examples
benchmarks where source semantics matter
```

Generated artifacts should be regenerated from authored source rather than hand-migrated.

---

# 32. Decision/spec migration

P4 must amend or supersede:

```text
TDR-0025 — List literals `[a,b,c]` desugar to List construction sends
TDR-0028 — Collections: native representation, shared protocol, literal surface
```

The new normative statement should encode:

```text
[...] is a contextual homogeneous sequence literal.

explicit one-argument head -> Array
explicit two-argument head -> Vector

unique expected List/Array/Vector -> that family

otherwise:
    runtime phase -> Array
    static phase -> Vector

phase is not an overload tie-breaker.
List remains available explicitly as the dynamically resizable homogeneous sequence.
```

Update the decisions index and current syntax/collection specs in the same closure tranche.

---

# 33. Cross-module durability

P4 must prove both semantic and runtime identity through modules.

Required cases:

```telos
// provider
export const v = [1,2,3]
export const a: Array<Int> = [1,2,3]

// importer
import provider { v, a }

// renamed import
import provider { v as values }

// re-export
export values
```

Assertions:

- static evaluator runs provider initializer once;
- static value identity survives semantic snapshot/export path as designed;
- runtime materialization belongs to canonical provider owner;
- importer reads reuse materialized value;
- Vector remains Vector;
- Array remains Array and frozen;
- Vector extent remains exact;
- no initializer AST is available/needed at runtime import use.

---

# 34. Incremental correctness requirements

P4 needs more than “edit rebuild succeeds.”

Minimum edit matrix:

| Before | After | Required invalidation |
|---|---|---|
| runtime `[1,2]` | runtime `[1,2,3]` | Array sequence value/type facts update |
| static `[1,2]` | static `[1,2,3]` | Vector N 2 -> 3, static value changes |
| explicit List context | remove context | family changes List -> Array at runtime |
| runtime binding -> const | phase changes Array default -> Vector default |
| `const Array<_>` | unannotated `const` | family changes Array -> Vector |
| exact Vector spread | Array spread | accepted -> static-cardinality diagnostic |
| valid const Array | invalid child static expression | no stale materialization descriptor |
| invalid -> valid -> invalid | all transitions | no stale static sequence/product reuse |

Compare cold and incremental:

```text
formatted canonical result type
static value semantic equality/shape
diagnostics
sequence construction proof
materialization capability
exported symbol/static constant identity
```

Do not compare raw arena IDs across unrelated stores.

---

# 35. P4 file/symbol impact map

This is a requirements map, not a frozen implementation plan.

| Area | Likely owner/symbol |
|---|---|
| static value representation | `phalcom-semantic/src/static_value/store.rs` |
| static materialization descriptor | `phalcom-semantic/src/materialization.rs` |
| static evaluator bracket execution | `phalcom-semantic/src/static_eval/machine.rs` |
| checked static sequence products | `CheckedStaticExpression::sequence_literals`, `CheckedStaticBody::sequence_literals` |
| phase default | `checker/sequence_literal.rs::synthesize` |
| static-expression extraction correctness | `checker/static_body.rs` |
| const mutation semantic diagnostics | checker binding/write/subscript-set path |
| frozen Array runtime | `heap/array.rs`, `primitive/array.rs` |
| Vector materialization | `heap/vector.rs`, runtime static constant materializer |
| backend descriptor projection | semantic lowering/static constant lowering |
| runtime constant caching | `phalcom-core/src/modules/static_constant.rs` |
| native authored Array typing | must already be closed in P3 |
| decisions | `docs/decisions/accepted/0025...`, `0028...` plus superseding record |
| current syntax/collection specs | repository current-spec owners |
| repository migration | all AST-discovered bare bracket uses |
| incremental tests | semantic incremental/static constant suites |
| runtime module parity | core modules/parity/static constant tests |

---

# 36. Proposed P4 execution decomposition

This is intentionally a requirements decomposition, not a patch-grade plan.

```text
P4-T0  convergence + authoritative census baseline
P4-T1  static sequence store representation
P4-T2  static evaluator integration
P4-T3  sequence materialization descriptor
P4-T4  runtime Vector/frozen-Array materialization
P4-T5  const mutation semantic/runtime defenses
P4-T6  cross-module + incremental durability
P4-T7  repository bare-bracket migration
P4-T8  decision/spec amendment
P4-T9  atomic runtime/static default flip
P4-T10 overload ambiguity + full regression certification
P4-T11 lifecycle reconciliation and P5 handoff
```

Dependencies:

```text
T0
├── T1 -> T2 -> T3 -> T4 -> T5 -> T6
└── T7 -> T8
             \       /
              \     /
                T9
                 ↓
                T10
                 ↓
                T11
```

The key property is that migration/decision work can proceed in parallel with static infrastructure, but the observable default flip waits for both branches.

---

# 37. P4 verification matrix

## 37.1 Static semantic values

- [ ] empty static Vector;
- [ ] nonempty primitive Vector;
- [ ] explicit static Array;
- [ ] inferred static Vector N;
- [ ] nested homogeneous sequence;
- [ ] sequence static semantic equality;
- [ ] Array versus Vector static identity differs;
- [ ] child unsupported for runtime materialization remains a valid static semantic value if C4 supports it;
- [ ] cancellation/budget does not publish partial sequence.

## 37.2 Materialization

- [ ] Vector produces `Object::Vector`;
- [ ] Array produces `Object::Array` with `frozen == true`;
- [ ] no initializer AST execution;
- [ ] read twice gives same owner-held runtime value;
- [ ] import/re-export reuses provider materialization;
- [ ] nested supported sequences materialize recursively;
- [ ] unsupported child fails closed without fallback execution;
- [ ] GC traces materialized children.

## 37.3 Const mutation

- [ ] direct const Array subscript write diagnosed;
- [ ] direct `at(_, put:)` diagnosed where statically knowable;
- [ ] Dynamic alias reaches frozen runtime guard;
- [ ] mutable ordinary runtime Array remains writable;
- [ ] List remains mutable/resizable;
- [ ] Vector remains immutable.

## 37.4 Defaults

- [ ] runtime no-context bare literal => Array;
- [ ] static no-context bare literal => Vector;
- [ ] expected List => List in both relevant phases;
- [ ] expected Array => Array;
- [ ] expected Vector => Vector;
- [ ] explicit `<_>` => Array;
- [ ] explicit `<_,_>` => Vector;
- [ ] empty bare static literal underconstrained T is diagnosed unless context supplies T;
- [ ] runtime overload Array/Vector ambiguity remains ambiguity;
- [ ] explicit heads disambiguate.

## 37.5 Migration

- [ ] every prior List-dependent repository source site classified;
- [ ] resizing sites made explicitly List;
- [ ] no semantic test accidentally relies on previous default;
- [ ] docs/examples migrated;
- [ ] accepted decisions no longer contradict behavior.

## 37.6 Incremental/cross-module

- [ ] family edit cold/incremental parity;
- [ ] cardinality edit cold/incremental parity;
- [ ] phase edit cold/incremental parity;
- [ ] import/re-export const sequence;
- [ ] stale descriptor not reused after initializer edit;
- [ ] unaffected sibling products retain expected reuse.

---

# 38. P4 performance/resource requirements

P4 is primarily semantic, but static sequences can create new compile-time and runtime allocation pressure.

Required safeguards:

1. Static evaluator charges query budget proportional to sequence element work.
2. `StaticValueStore` interning must avoid accidental quadratic comparison where large sequences are common; ordinary hash interning is acceptable for C1.
3. Runtime materializer must use checked capacity/allocation boundaries.
4. Exact arbitrary-precision Vector N must not be silently narrowed merely because a runtime value is requested.
5. Materialization should allocate each final runtime sequence once; do not construct Array/List as staging for Vector.
6. Constant caching must prevent repeated whole-sequence allocation on each read/import.
7. Large-static-sequence tests should distinguish semantic-size limits, query budget, and runtime allocation failure.

P9 remains the owner of broad performance certification. P4 should add only regressions required to prevent pathological implementation choices in its own path.

---

# 39. P4 diagnostics requirements

Diagnostics should distinguish at least:

```text
static sequence evaluator unsupported child/materialization
const Array direct mutation
runtime frozen Array mutation
static Vector extent/cardinality conflict inherited from P3
underconstrained empty static Vector element type
ambiguous Array/Vector overload context
legacy source migration failures, where applicable
```

Do not add duplicate error codes simply to label “P4.”

Use existing diagnostic taxonomy where the semantic meaning already exists.

---

# 40. P4 failure conditions requiring stop/consultation

The implementer should stop and request an architectural ruling if any of the following becomes true:

1. implementing static sequences appears to require a second evaluator;
2. runtime materialization requires serializing live VM `Value` or heap pointers;
3. arbitrary structural sequence elements require silently implementing all deferred C4 structural materialization;
4. const Array would need a new nominal class rather than ordinary frozen Array;
5. phase defaulting would need to bias overload resolution;
6. imported const sequence identity cannot be preserved without duplicating the static constant owner model;
7. P4 requires changing Vector static extent semantics established by P3;
8. `main` convergence invalidates the P1/P2/P3 family-selection architecture;
9. legacy TDR amendment cannot be obtained before cutover;
10. the repository census contains an ambiguous high-risk List-dependent use that cannot be migrated mechanically without behavior choice.

---

# Part VI — Priority code-review findings

## 41. Consolidated finding register

| ID | Severity | Finding | Owner |
|---|---|---|---|
| P3-F01 | BLOCKER | negative Vector extent not centralized | P3 |
| P3-F02 | BLOCKER | publication helper unintegrated; direct product publication remains | P3 |
| P3-F03 | HIGH | Array public API uses Dynamic/untyped replacement | P3 |
| P3-F04 | BLOCKER | Vector mutable-descendant structural hash instability | P3 |
| P3-F05 | HIGH | wrong-kind/arity/scoped/general Iterable coverage incomplete | P3 |
| P3-F06 | INFO | symbolic construction intentionally bounded; diagnostics need complete proof | P3 |
| P3-F07 | MEDIUM | `usize -> u32` fixed-count cast should be checked | P3 |
| P3-F08 | MEDIUM | effect-order/failure cleanup/GC-during-construction coverage incomplete | P3 |
| P3-F09 | LOW | exact Vector spread allocates temporary `Vec` | P9/perf |
| P3-F10 | HIGH | metadata/reflection/import/incremental evidence incomplete | P3 |
| P3-F11 | HIGH | lifecycle records contradict current continuation state | P3 |
| C1-F01 | HIGH | feature branch 14 commits behind main with P4-overlap files | pre-P4 |
| P4-F01 | BLOCKER if ignored | C4 structural materialization remains deferred | P4 gate |
| P4-F02 | HIGH | no durable P4 bare-bracket migration inventory found | P4 |
| P4-F03 | HIGH | accepted TDR-0025/0028 still define brackets as List | P4 |
| P4-F04 | HIGH | static evaluator currently rejects BracketLiteral | P4 |
| P4-F05 | HIGH | no StaticValueData homogeneous sequence node exists | P4 |

---

# Part VII — Recommended next action

## 42. What the P3 implementer should do next

Do **not** start P4 code.

The highest-value order is:

1. finish `SequencePublication` T2–T5 integration exactly as the approved supplement specifies;
2. add the canonical negative-extent formation rule;
3. correct Array API typing and regenerate native metadata;
4. decide/fix Vector mutable-descendant hash/key policy;
5. finish missing semantic/runtime/durability tests;
6. run P3 focused gates;
7. reconcile checkpoint/plan/walkthrough/handoff/STATUS;
8. converge the branch with current `main`;
9. rerun the P3 acceptance gates on the converged tree.

Only then freeze a P4 implementation plan.

---

# 43. P4 readiness decision

**Current readiness: NOT READY TO IMPLEMENT.**

P4 requirements are sufficiently understood to plan, but P4's implementation preconditions are not satisfied.

The strongest reason is architectural rather than procedural:

> P4 will make bare bracket literals ubiquitous producers of Array/Vector semantic and static construction products. Beginning that cutover while inference-owned product publication is still unsound would multiply the blast radius of the P3 defect across ordinary runtime literals, constants, imports, incremental snapshots, and backend materialization.

The correct strategy is to close product lifetime correctness first, then expand the feature's default reach.

---

# Appendix A — Important live code excerpts

## A.1 Current P3 bare default is still intentionally List

`phalcom-semantic/src/checker/sequence_literal.rs`:

```rust
let selected = match literal.head.as_ref().map(|head| head.arguments.len()) {
    Some(1) => SequenceLiteralFamily::Array,
    Some(2) => SequenceLiteralFamily::Vector,
    ...
    None => contextual_family.unwrap_or(SequenceLiteralFamily::List),
};
```

This is correct for P3 and must change only in P4.

## A.2 Sequence product still publishes directly

```rust
if valid && let Some(expression) = ctx.current_expression_id() {
    ctx.sequence_literals.insert(
        expression,
        SequenceLiteralResolution { ... },
    );
}
```

This is the live publication defect that `298084a` has not yet replaced.

## A.3 Static body extraction occurs before final expected solve

`phalcom-semantic/src/checker/static_body.rs` currently collects `context.sequence_literals`, then later solves `ExpectedType::Inference`.

This ordering must be reversed/transactional before P3 closure.

## A.4 Checked large Vector extent conversion is fail-closed

`phalcom-core/src/modules/semantic_lowering.rs`:

```rust
expected_extent: literal.extent.and_then(|extent| match snapshot.store.get(extent) {
    TypeData::ConstInt(value) => u32::try_from(value.to_bigint()).ok(),
    _ => None,
}),
```

This appropriately separates exact semantic integer identity from host-addressable construction count.

## A.5 Compiler validates symbolic identity spread proof

`phalcom-core/src/compiler/lib/expr.rs` verifies:

```text
proof result extent
proof source expansion
proof source range
literal has exactly one expansion
```

before emitting `CopyVector`.

This is a strong boundary worth preserving.

## A.6 Static evaluator has no bracket-literal case

`phalcom-semantic/src/static_eval/machine.rs` handles primitive literals, tuples, records, checked constructors, static applications, etc., then rejects unmatched expressions as:

```rust
_ => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
```

P4 must add checked sequence execution here, not elsewhere.

## A.7 Runtime structural constants are currently rejected

`phalcom-core/src/modules/static_constant.rs`:

```rust
MaterializationDescriptor::DeferredStructural(_)
| MaterializationDescriptor::Unsupported(_) => {
    return Err(RuntimeError::Internal(
        format!("static constant {} has no supported runtime materialization", spec.id)
    ).into());
}
```

P4 must extend the canonical descriptor/materializer boundary for homogeneous sequences.

---

# Appendix B — P4 completion criteria

P4 may be marked complete only when all statements below are true:

```text
[ ] P3 is accepted and lifecycle records agree.
[ ] feature branch is converged with current main and P3 gates pass after convergence.
[ ] StaticValueStore has one canonical homogeneous sequence representation.
[ ] Array and Vector static identity remain nominally distinct.
[ ] Static evaluator consumes checked sequence products.
[ ] Static evaluator evaluates operands once, in order.
[ ] Static Vector materializes as ordinary immutable Vector.
[ ] Static Array materializes as ordinary frozen Array.
[ ] Direct const Array mutation is diagnosed.
[ ] Alias-based const Array mutation is stopped by the runtime frozen guard.
[ ] Runtime constant materialization does not re-run initializer AST.
[ ] Cross-module import/re-export preserves owner/materialization identity.
[ ] Repository bare-bracket use has a durable classification inventory.
[ ] All dynamic-resize dependencies are migrated to explicit List.
[ ] TDR-0025/TDR-0028 are amended/superseded.
[ ] Runtime bare brackets default to Array.
[ ] Static bare brackets default to Vector.
[ ] Explicit/contextual List remains List.
[ ] Explicit/contextual Array remains Array.
[ ] Explicit/contextual Vector remains Vector.
[ ] Phase default does not resolve overload ambiguity.
[ ] Cold/incremental parity is demonstrated for family, phase, N, and spread-exactness edits.
[ ] static sequence fingerprints/dependencies invalidate correctly.
[ ] native/generated artifacts are current.
[ ] focused semantic/static/runtime/module/incremental gates pass.
[ ] scoped diff review finds no unrelated migration.
[ ] C1 checkpoint, P4 plan, walkthrough, handoff, PROGRAM, and STATUS agree.
```

---

# Appendix C — Audit limitations

This report is a repository/code/document audit, not a fresh local execution report.

It inspected the connected GitHub branch and its current source/commit history. It did not clone and execute the workspace locally. The command results described as passing are the results recorded in the repository's P3 lifecycle ledger. No GitHub Actions run was available for the current `298084a` head through the connected integration, and the combined status endpoint was not readable by the integration.

Those limitations do not affect the structural findings that:

- the publication helper is not referenced by production checker state;
- sequence synthesis still writes directly to the committed map;
- static sequence products are copied before final expected inference settlement;
- canonical Vector application lacks negative-extent validation;
- Array's authored public surface remains dynamically typed;
- the C4 static evaluator has no bracket-literal execution case;
- the C4 static store has no sequence node;
- structural materialization remains deferred;
- the branch is behind current `main`;
- the live COLL006 `STATUS.md` is stale relative to the later P3 continuation evidence.

These are direct source/lifecycle observations at the audited head.
