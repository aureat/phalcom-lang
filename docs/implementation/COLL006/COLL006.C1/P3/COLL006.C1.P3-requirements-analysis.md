# COLL006.C1.P3 — Implementation Architecture, Repository Audit, and Execution Plan

**Target:** `Vector<T, N>` — canonical statically dimensioned, immutable sequence  
**Repository:** [`aureat/phalcom-lang`](https://github.com/aureat/phalcom-lang)  
**Branch:** `feat/telos-collections-upgrade`  
**Date:** 8 October 2026  
**Implementation stage:** P1 and P2 completed; P3 pending

## 1. Executive assessment

**P3 should proceed as a bounded implementation of `Vector<T, N>`, but its first milestone must repair and verify the canonical static-generic infrastructure inherited from P1.**

The repository inspection confirms that P1 and P2 have established the principal architectural foundations: neutral bracket-literal syntax, contextual collection-family resolution, inference-bearing annotations, semantic lowering products, canonical `Array<T>` identity, fixed-extent runtime storage, and direct Array construction.

However, implementing Vector is substantially more demanding than adding another collection class. Its distinguishing property is that **the sequence's extent is part of its canonical type identity**, rather than a runtime attribute.

```telos
let a: Array<Int> = [1, 2, 3]
let b: Array<Int> = [1, 2, 3, 4]

// Same nominal type:
// Array<Int>

let c: Vector<Int, 3> = [1, 2, 3]
let d: Vector<Int, 4> = [1, 2, 3, 4]

// Distinct canonical types:
// Vector<Int, 3>
// Vector<Int, 4>
```

This requires correct integration across the entire compilation pipeline:

```text
Source syntax
    ↓
Canonical static-domain and generic argument formation
    ↓
Contextual sequence-family selection
    ↓
Element inference and exact cardinality proof
    ↓
Canonical inference solving
    ↓
Published semantic sequence resolution
    ↓
Compiler lowering
    ↓
VM construction
    ↓
Immutable runtime Vector object
```

The most important architectural principle is:

> `Vector<T, N>` must be an ordinary application of Telos's canonical generic type system, with `N` represented as a static integer term. Vector must not become a privileged second type system implemented through compiler-specific length metadata.

### 1.1 Audit classification

| Area | Assessment | P3 implication |
|---|---|---|
| P1 neutral bracket syntax | Established | Preserve without redesign |
| P1 query-local inference | Established, incomplete for static-domain arguments | Correct at the generic elaboration boundary |
| P1 semantic product | Established | Extend to publish exact cardinality evidence |
| P2 canonical Array identity | Established | Follow the canonical Universe registration pattern |
| P2 fixed runtime storage | Established | Use analogous storage with structural immutability |
| P2 direct construction | Established | Extend lowering without intermediate collections |
| P2 private spread builder | Established | Reuse construction mechanics where appropriate |
| Static integer generic arguments | **Critical prerequisite** | Verify and repair before Vector runtime implementation |
| Universe static-domain bootstrap | **Critical prerequisite** | Resolve generically, not through Vector exceptions |
| Exact spread-cardinality analysis | Not established by P2 | Introduce semantic proof infrastructure |
| Vector structural equality and hashing | P3 responsibility | Implement in Telos source |
| Const Vector materialization | P4 responsibility | Do not implement prematurely |
| Bare bracket default migration | P4 responsibility | Preserve existing behavior |
| Lifecycle records | Inconsistent with completed P2 | Reconcile before implementation begins |

The uploaded requirements analysis independently identifies the two most important prerequisites: concrete static generic arguments must elaborate as canonical values rather than ordinary expression types, and Universe bootstrap must correctly recognize static-domain binders. Pasted text Pasted text

**Verification distinction:** The P2 handoff reports 15 focused semantic tests, 10 focused runtime tests, and a successful workspace check. These are recorded implementation results, not fresh executions performed during this review. The review establishes architectural findings through remote source inspection; runtime claims beyond the inspected code retain their reported verification status.

## 2. Repository baseline and evidence

The remote branch was inspected at the following revision:

| Reference | Commit | Meaning |
|---|---|---|
| `main` | `61c8734f` | Baseline before P1 |
| P1 | `fb41492d` | Contextual sequence literals and inference foundation |
| P2 / branch HEAD | `5fee70c9` | Runtime fixed-extent Array implementation |

The collection branch is **two commits ahead of `main`**. The implementation plan should be pinned to `5fee70c9202af964de4c4250433bab8c806a0da4` until the implementer establishes a newer, verified takeover revision.

Primary repository records:

- [C1 checkpoint](https://github.com/aureat/phalcom-lang/blob/5fee70c9/docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md)
- [C1 detailed implementation plan](https://github.com/aureat/phalcom-lang/blob/5fee70c9/docs/implementation/COLL006/COLL006.C1/COLL006.C1-detailed-implementation-plan.md)
- [C1 master implementation record](https://github.com/aureat/phalcom-lang/blob/5fee70c9/docs/implementation/COLL006/COLL006.C1/COLL006.C1-master-implementation-record.md)
- [P2 implementation walkthrough](https://github.com/aureat/phalcom-lang/blob/5fee70c9/docs/implementation/COLL006/COLL006.C1/COLL006.C1.P2-walkthrough.md)
- [P2 handoff](https://github.com/aureat/phalcom-lang/blob/5fee70c9/docs/implementation/COLL006/COLL006.C1/COLL006.C1.P2-handoff.md)

### 2.1 Important discrepancy between planning records

The earlier P3 requirements analysis correctly identifies the type-formation and inference problems. However, the repository's C1 detailed plan defines **two additional architectural obligations that must not be lost**.

First, although `TypeId` is the correct representation of a canonical static argument, the current payload `TypeData::ConstInt(i64)` is insufficient for Telos's arbitrary-precision `Int` semantics.

The requirements analysis recommends retaining `ConstInt` as the canonical representation. That is correct at the identity-model level, but it does not settle the integer-width limitation. Pasted text

Second, exact spread cardinality cannot be inferred solely from the source collection's type when the runtime expansion uses an unconstrained, potentially overridden iterator.

Consequently, P3 needs three foundational guarantees:

1. **Exact static integer identity:** `N` cannot be semantically truncated to `i64`, `u64`, or `usize`.
2. **Canonical heterogeneous generic formation:** `Vector<T, N>` must preserve the distinct kinds of `T` and `N` across source, semantic metadata, and reflection.
3. **Proof-preserving expansion:** A runtime spread must emit precisely the positional elements whose count semantic analysis proved.

The detailed C1 plan identifies these as `P3-G0A`, `P3-G0B`, and `P3-G0C`. Its proposed amendments `COLL006.C1.A5` and `A6` should be reconciled with the checkpoint before implementation changes the shared type/metadata model.

**Architectural ruling:** Preserve canonical Universe source as the authority for declarations. Metadata must accurately project that source-owned identity, not independently redefine it.

---

## 3. P1/P2 audit — findings and corrective actions

### 3.1 Findings summary

The following classifications distinguish source-confirmed defects from unverified correctness risks.

| ID | Severity | Finding | Classification | Required disposition |
|---|---|---|---|---|
| A01 | Critical | Partial annotation treats static value `3` as type `Int` | **Confirmed defect** | Repair before P3 Vector inference |
| A02 | Critical | Complete source generic application rejects static value arguments | **Confirmed unsupported path** | Repair canonical type formation |
| A03 | Critical | Universe bootstrap rejects static-domain binder syntax | **Confirmed unsupported path** | Repair bootstrap kind resolution |
| A04 | Critical | Canonical static integer terms are limited to `i64` | **Confirmed representation limitation** | Reconcile and implement exact integer identity |
| A05 | Critical | Exact-sized Vector spread could use overridable iteration | **Architectural soundness risk** | Require proof-preserving expansion |
| A06 | High | Sequence lowering product omits `extent` | **Confirmed gap** | Extend compiler-facing semantic product |
| A07 | High | Compiler has collection-family fallback without lowering metadata | **Confirmed compatibility path; unsafe for Vector** | Restrict or eliminate for Vector |
| A08 | High | Invalid sequence synthesis can publish a fallback resolution | **Confirmed code path** | Publish only established semantic facts |
| A09 | High | Homogeneous Array mutation API uses `Dynamic` | **Confirmed typing weakness** | Correct or document as bounded P2 follow-up |
| A10 | Medium | Array spread finalization clones its entire builder | **Confirmed implementation inefficiency** | Use move-based finalization for Vector; consider shared correction |
| A11 | Medium | Bracket-literal explanation identifies the family as `List` | **Confirmed diagnostic/provenance defect** | Derive explanation from checked resolution |
| A12 | Medium | Contextual static-domain kind fallback uses ordinary `kind_of` | **Potential defect** | Prove with generic declaration tests; repair if reproduced |
| A13 | High | Structural Vector hashing can become unstable through mutable elements | **Cross-family semantic risk** | Establish hash-stability policy and regression tests |
| A14 | Medium | P2 tests do not demonstrate all failure, aliasing, and GC conditions | **Coverage gap** | Add targeted regression gates |
| A15 | Medium | Lifecycle headers and takeover text remain stale | **Confirmed documentation inconsistency** | Reconcile records before P3 starts |

### 3.2 A01 — Static generic values are elaborated as expression types

**Location:** [`phalcom-semantic/src/checker/annotation_elaboration.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-semantic/src/checker/annotation_elaboration.rs#L162-L202)

The current implementation contains the equivalent of:

```rust
GenericArgumentSyntax::Value(expr) => {
    let typed = analyze_expression(
        ctx,
        expr,
        &ExpectedType::None,
    );

    if let Some(ty) = typed.knowledge.ty() {
        static_args.push(
            InferenceStaticArgument::Term(
                InferenceTerm::Canonical(ty)
            )
        );
    }
}
```

For:

```telos
let values: Vector<_, 3> = [1, 2, 3]
```

the expression `3` has ordinary expression type `Int`.

But the second generic argument must be the **canonical static value** `3`, not the type `Int`.

The required distinction is:

```text
Int
    kind: Type

ConstInt(3)
    kind: StaticDomain(Int)
```

The correction must be parameter-directed:

```rust
// Conceptual interface; not a prescribed new public API.
fn elaborate_generic_argument(
    syntax: &GenericArgumentSyntax,
    parameter_kind: KindId,
    context: GenericElaborationContext,
) -> GenericArgumentOutcome
```

It must interpret ordinary type arguments, static values, and inference holes according to the destination parameter's declared domain.

This helper should be shared by complete type formation and inference-bearing annotation elaboration. Creating a separate `Vector` integer-literal parser or independent static expression interpreter would repair the symptom at the wrong architectural layer.

There is an additional defensive requirement: the current partial elaborator falls back to `KindId::TYPE` when it cannot recover a parameter kind. P3 must fail closed rather than allocate a potentially wrong-kind inference variable.

**Acceptance condition:** `Vector<_, 3>` contains a type-kind variable in the first slot and a canonical static integer term in the second; `Vector<Int, _>` creates a static-integer-domain variable, not a type variable.

### 3.3 A02 — Complete source type applications cannot form `Vector<Int, 3>`

**Location:** [`phalcom-semantic/src/types/annotation.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-semantic/src/types/annotation.rs#L3180-L3240)

The non-variadic generic-application resolver currently accepts `GenericArgumentSyntax::Single(...)` and rejects other forms, including `GenericArgumentSyntax::Value(...)`.

This means the canonical type store can represent heterogeneous applications, but the source resolver cannot yet form the necessary expression.

**Do not patch the Vector declaration itself.** Extend canonical generic application so each fixed parameter is elaborated in its declared domain.

The repair must also support symbolic arguments:

```telos
class Proposition<T, N: Int> {}

class Evidence<N: Int> {
  // A type position may refer to the static binder N.
  // Proposition<Int, N>
}
```

The implementation must resolve a static-bound identifier `N` through the established static/type-level binding environment. Evaluating `N` as an ordinary runtime variable would be incorrect.

### 3.4 A03 — Universe bootstrap cannot predeclare static-domain parameters

**Location:** [`phalcom-semantic/src/declarations.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-semantic/src/declarations.rs#L403-L455)

The current bootstrap calls:

```rust
resolve_kind_syntax(store, syntax)
```

Its early kind resolver rejects `KindSyntax::StaticDomain`.

Thus the desired declaration:

```telos
native class Vector<T, N: Int> is Collection
```

cannot simply be inserted into the Universe and expected to bootstrap successfully.

The correct repair is a narrow, source-authoritative static-domain bootstrap mechanism. It should recognize intrinsic static domains, including the canonical `Int` domain, without requiring the complete declaration resolver to be available before nominal shells exist.

A reasonable internal design is:

```text
Canonical Universe binder syntax
          ↓
Bootstrap-resolvable domain identity
          ↓
Canonical StaticDomainId
          ↓
KindId::StaticDomain(...)
          ↓
GenericSignature
          ↓
Canonical nominal constructor form
```

Ordinary source declarations must eventually use the same canonical domain identity.

### 3.5 A04 — Static integer width violates the intended Vector ontology

**Locations:**

- [`phalcom-semantic/src/types/store.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-semantic/src/types/store.rs#L263-L267)
- [`phalcom-semantic/src/scalar.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-semantic/src/scalar.rs)
- [`phalcom-type-meta/src/type_node.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-type-meta/src/type_node.rs#L110-L171)

The semantic type store currently uses:

```rust
TypeData::ConstInt(i64)
```

and:

```rust
pub fn const_int(&mut self, value: i64) -> TypeId
```

Meanwhile, the static-value subsystem already implements an exact integer representation:

```rust
pub enum StaticInt {
    Small(i64),
    Large(Arc<BigInt>),
}
```

The metadata representation also contains `ConstInt(i64)` variants.

This is a genuine conflict with the existing C1 detailed plan, which requires exact static `Int` semantics.

The correct design retains the canonical `TypeId` abstraction while widening the **underlying integer term** through a shared exact-integer representation.

Conceptually:

```rust
enum TypeData {
    // ...
    ConstInt(ExactStaticInt),
}
```

`ExactStaticInt` is illustrative; reuse the existing normalized `StaticInt` representation where dependency direction allows it. Do not introduce a second concept of integer equality or canonicalization.

This work must include semantic metadata encoding/decoding, stable identities, reflection, type relations, fingerprints, and type-member static arguments. It must not stop at changing a single enum field.

A distinction is essential:

```text
Vector<Int, 100000000000000000000>
```

may be a perfectly meaningful canonical type even when no process can allocate a value with that many elements.

**Type formation must retain exact `N`; runtime construction performs a separately checked resource/addressability conversion.**

### 3.6 A05 — Exact static spread cardinality must match runtime behavior

The current spread compiler supports a Tuple/Unit positional fast path and a general iterator path.

**Locations:**

- [`phalcom-core/src/compiler/lib/expr.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-core/src/compiler/lib/expr.rs#L103-L170)
- [`phalcom-core/src/vm/dispatch.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-core/src/vm/dispatch.rs#L198-L242)
- [`phalcom-semantic/src/checker/iteration.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-semantic/src/checker/iteration.rs#L323-L371)

Suppose:

```telos
let source: Vector<Int, 3> = <Int, 3>[1, 2, 3]

let result = <Int, _>[0, *source]
```

Static analysis can derive `N = 4` from the source's type.

But if `*source` executes an overridable `iterator` implementation, the runtime might produce fewer or more than three elements. The static proof would no longer describe the actual construction.

This is a **soundness issue**, not merely an optimization concern.

P3 should define an exact positional expansion mode that is inseparable from its proof. For built-in Vector, the preferred implementation is a checked intrinsic dense-storage expansion using the Vector's native storage length and element order, rather than unconstrained virtual iteration.

A private expansion contract could be modeled as:

```rust
enum CheckedSequenceExpansion {
    ExactTuplePositionals {
        count: u32,
    },

    ExactVectorStorage {
        extent: TypeId,
    },

    GeneralIterable,
}
```

This is a proposed shape, not a requirement to introduce precisely this enum. The fundamental requirement is that **a statically exact proof selects a runtime lane that preserves that proof**.

A defensive final-length check is still necessary, but it cannot replace the semantic and lowering guarantees.

### 3.7 A06–A08 — Semantic lowering and invalid-state publication

Three related issues deserve a coordinated correction.

#### Compiler-facing lowering omits the extent

**Location:** [`phalcom-core/src/modules/semantic_lowering.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-core/src/modules/semantic_lowering.rs#L578-L582)

The semantic resolution already contains:

```rust
pub extent: Option<TypeId>
```

However, the lowering specification contains only:

```rust
pub struct SequenceLiteralLoweringSpec {
    pub family: SequenceLiteralFamily,
    pub construction: SequenceConstructionPlan,
}
```

P3 cannot meet the compiler-authority invariant while this remains unchanged.

The lowering specification must carry the checked Vector extent, plus the expansion execution plan required to preserve cardinality.

The compiler may convert a proved concrete extent to a machine count for bytecode emission. It may not independently determine the semantic extent.

#### Missing lowering metadata causes family reconstruction

**Location:** [`phalcom-core/src/compiler/lib/expr.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-core/src/compiler/lib/expr.rs#L1404-L1417)

The compiler currently contains:

```rust
let family = if let Some(lowering) = self.lowering() {
    // Consume checked semantic resolution.
    spec.family
} else if list_expr.head.is_some() {
    SequenceLiteralFamily::Array
} else {
    SequenceLiteralFamily::List
};
```

This appears to serve a context-free interpreter compatibility path. Nevertheless, it is not safe for Vector.

For example, in the absence of lowering metadata:

```telos
<Int, 3>[1, 2, 3]
```

would be classified as Array by this fallback.

**Required distinction:**

- A fully checked compilation must always consume its semantic product.
- A context-free compatibility compiler must not invent a Vector type or reinterpret an explicit Vector head as Array.
- If the context-free path cannot obtain the required checked product, it must report a supported compilation-boundary error.

Ideally, the compiler's required-semantic mode should be explicit rather than inferred from whether an optional lowering object happens to exist.

#### Invalid semantic results can still produce fallback resolutions

**Location:** [`phalcom-semantic/src/checker/expression.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-semantic/src/checker/expression.rs#L2740-L2777)

When a bracket literal's type is unknown, the implementation can construct a fallback type using `Never` and still insert a sequence resolution into the context.

Although later compilation may reject the diagnostic state, this is undesirable for a semantic product advertised as a checked construction authority.

For P3, **publication must require a solved, valid result**:

```text
Family resolved
    ∧ Element type established
    ∧ Extent proof established
    ∧ Mandatory inference obligations discharged
    ∧ Construction lane valid
        ↓
Publish checked Vector resolution
```

A failed or underconstrained Vector literal must not publish a fabricated `Vector<Never, N>` or an unchecked cardinality.

Preserve diagnostic recovery separately from executable semantic products.

### 3.8 A09 — P2 does not fully enforce homogeneity through its source API

**Locations:**

- [`collections/array.ph`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-core/core/universe/src/collections/array.ph)
- [`primitive/array.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-core/src/primitive/array.rs)

The current Array source declares:

```telos
native class Array<T> is Collection {
  internal native rawAt(_ index: Int) -> Dynamic
  internal native rawSet(
    _ index: Int,
    _ value: Dynamic
  ) -> Dynamic
}
```

Its public element-replacement methods also leave their value parameters untyped.

This weakens the meaning of `Array<T>`.

A typed Array should not allow:

```telos
let values: Array<Int> = <Int>[1, 2, 3]

// Should fail semantic checking:
values[0] = "incorrect"
```

The runtime's storage being `Value`-based is not a justification for making the source API dynamically typed.

The appropriate model is:

```telos
// Illustrative desired signatures.
internal native rawAt(_ index: Int) -> T
internal native rawSet(
  _ index: Int,
  _ value: T
) -> Unit
```

Public checked accessors must distinguish out-of-range behavior from valid `T` values. In particular, a primitive's internal `None` sentinel must not be misrepresented as a valid arbitrary `T`.

The precise `at`/`get` API must be reconciled with the existing collection convention before changing public behavior.

**Disposition:** Record this as a P2 corrective finding. Repair it before relying on the same API pattern for Vector. At minimum, P3 must not reproduce the weakness.

The Vector source API should expose `T`-typed reads and typed iteration, not a public sequence of `Dynamic` values.

### 3.9 A10 — Array builder finalization copies the collection

**Location:** [`phalcom-core/src/vm/dispatch.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-core/src/vm/dispatch.rs#L4033-L4044)

Current finalization uses:

```rust
Some(builder) => builder.clone().finish()
```

The entire growable builder is cloned before becoming an Array.

This has avoidable consequences:

- O(n) copying at finalization.
- Additional transient memory proportional to the collection size.
- Continued residency of the original builder until reclamation.
- More GC/rooting complexity than a move-based finalization needs.

This is an implementation-quality problem, not evidence that the Array result is functionally incorrect.

A preferable heap operation would take ownership of the private builder payload exactly once:

```rust
// Conceptual operation.
fn finish_dense_literal(
    &mut self,
    builder: ObjRef,
    target: DenseSequenceTarget,
) -> Result<ObjRef, RuntimeError>
```

Its implementation must preserve GC roots while transitioning ownership and must never expose an emptied builder to guest code.

For Vector, finalization must additionally receive or retrieve the semantically proved extent and verify it before publishing the object.

**Do not copy P2's clone-on-finalization implementation into Vector merely for consistency.**

### 3.10 A11 — Semantic explanations still identify Array literals as List

**Location:** [`phalcom-semantic/src/checker/expression.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-semantic/src/checker/expression.rs#L105-L120)

The expression-analysis explanation currently constructs:

```rust
Expr::BracketLiteral(_) => {
    ExplanationStep::CollectionSynthesis {
        kind: CollectionKind::List,
        // ...
    }
}
```

This is already inaccurate for P2 Array literals.

The explanation, diagnostic presentation, and advisory layers should use the same checked family resolution as compiler lowering.

The source syntax is neutral; therefore, reporting `List` for all bracket literals contradicts P1's authority model.

**Required fix:** Derive the explanation's collection identity from the established semantic product. If the family remains unresolved, report unresolved knowledge rather than assuming List.

### 3.11 A12 — Static-domain kind classification needs a focused probe

**Location:** [`phalcom-semantic/src/types/annotation.rs`](https://github.com/aureat/phalcom-lang/blob/5fee70c9/phalcom-semantic/src/types/annotation.rs#L593-L655)

The contextual kind resolver contains an intrinsic-domain lookup, but its fallback uses:

```rust
let kind = store.kind_of(ty);
```

The same file provides a helper:

```rust
static_domain_kind_of_form(store, form)
```

which explicitly maps canonical `Int` in a domain position to the built-in static integer domain.

This inconsistency is suspicious but not conclusively a runtime defect: the earlier intrinsic lookup may already handle the ordinary case.

The implementer must first test:

```telos
class Sized<T, N: Int> {}
```

and inspect the published parameter kinds. If the fallback is exercised incorrectly, repair classification through the canonical helper rather than adding a Vector-specific exemption.

### 3.12 A13 — Immutability is not automatically deep hash stability

Vector's own storage is immutable, but its elements can reference mutable objects.

Consider the conceptual sequence:

```telos
let mutable = <Int>[1, 2]
let key = <Array<Int>, 1>[mutable]
```

Even if `key` cannot change its single stored reference, the referenced Array can change its contents.

If Vector's hash recursively incorporates element hashes, and an element's hash is content-sensitive and mutable, the Vector's hash can change after insertion into a hash table.

This tension already exists in other immutable structural containers, particularly Tuple. It should not be treated as a Vector-only defect.

**P3 ruling:** Implement structural hashing in Telos and allow ordinary immutable, hash-stable Vector keys. Do not add Vector to the mutable-collection rejection list. However, do not claim that arbitrary recursively mutable contents are automatically safe as hash keys.

The immediate acceptance matrix should cover Vectors containing stable primitive values and nested immutable values. Cases containing mutable descendants must be tested and evaluated against the existing Map/Set key policy.

A general recursive hash-stability policy, including cycles and custom user equality/hash methods, would be a cross-family semantic decision. Do not introduce it silently under P3.

### 3.13 A14 — P2 test coverage is narrower than its architectural contract

The P2 runtime suite contains useful tests for:

- fixed size and indexing;
- mutation without resizing;
- absence of structural resize selectors;
- ordered equality;
- live-view iteration;
- mutable-key rejection;
- spread construction;
- direct bytecode emission;
- GC retention of contained objects.

However, the inspected focused tests do not establish exhaustive coverage of:

| Unproved or insufficiently exercised case | Why it matters |
|---|---|
| Wrong-typed Array element replacement | Source homogeneity |
| Frozen Array mutation rejection | P4 safety prerequisite |
| GC triggered during spread iteration | Builder/scratch rooting |
| User iterator throws during construction | Failure cleanup |
| Reentrant iterator or element callbacks | VM borrow and root safety |
| Huge spread cardinality | Overflow and resource handling |
| Nested mutable key behavior | Hash consistency |
| Negative-index boundary extremes | Integer normalization |
| Missing semantic lowering metadata | Compiler fail-closed behavior |
| Invalid literal still publishing resolution | Durable semantic correctness |

This does not negate P2's recorded focused-test result. It means the verification classification should remain `FOCUSED_TESTED`, not be promoted to comprehensive certification.

### 3.14 A15 — Lifecycle record cleanup is required

The checkpoint's plan ledger correctly identifies P1 and P2 as completed, and its final takeover section names P3 as the next action.

However:

- `PROGRAM.md` still declares `PROPOSED / NOT_STARTED / UNVERIFIED`.
- `STATUS.md` has similarly stale front matter and text describing P1/P2 as future work.
- The checkpoint retains pre-P1 baseline statements describing `Expr::ListLiteral` as current implementation.
- Several passages say no numbered plans have been published or that P1 should be authored.

The correct current-state summary is:

```yaml
program: COLL006
checkpoint: COLL006.C1

checkpoint_status: IN_PROGRESS
checkpoint_completion: PARTIAL
checkpoint_verification: FOCUSED_TESTED

latest_completed_plan: COLL006.C1.P2
active_plan: none
next_plan: COLL006.C1.P3

completed_plans:
  - COLL006.C1.P1
  - COLL006.C1.P2
```

Treat original pre-P1 grounding as historical evidence rather than silently deleting it. Keep the checkpoint as the single authority for C1 progress.

---

## 4. Ratified P3 semantic contract

The implementer should begin with a compact, authoritative specification.

### 4.1 Ontology

`Vector<T, N>` is a homogeneous, ordered, owned sequence with **immutable elements and exact static extent**.

Its canonical generic shape is:

```text
Vector:
    (Type, StaticDomain(Int))
        -> Type
```

Its semantic obligations are:

| Property | Requirement |
|---|---|
| Element type | `T: Type` |
| Extent argument | `N` is a canonical static `Int` term |
| Extent identity | `N` participates in canonical applied-type identity |
| Concrete extent | Must be nonnegative |
| Symbolic extent | Permitted under `N: Int`; value construction still requires proof |
| Physical storage | Contiguous `Value` slots in v1 |
| Storage mutation | Forbidden |
| Structural resizing | Forbidden |
| Runtime representation | Does not need to store `T` or `N` as separate instance metadata |
| Element access | Typed as `T` |
| Iteration | Reuses canonical `Iterable` / `Iterator` |
| Equality | Ordered structural equality, same collection family |
| Hashing | Ordered structural hashing, consistent with equality |
| Runtime literal construction | Direct Vector allocation |
| Static evaluation/materialization | Deferred to P4 |

### 4.2 Required source behavior

```telos
let a = <Int, 3>[1, 2, 3]

let b = <Int, _>[1, 2, 3]

let c = <_, 3>[1, 2, 3]

let d = <_, _>[1, 2, 3]

// Each resolves to Vector<Int, 3>.
```

Contextual forms:

```telos
let a: Vector<Int, 3> = [1, 2, 3]

let b: Vector<Int, _> = [1, 2, 3]

let c: Vector<_, 3> = [1, 2, 3]

let d: Vector<_, _> = [1, 2, 3]
```

All eight forms must resolve through the same inference and type-formation authority.

### 4.3 Empty Vector behavior

```telos
let empty = <Int, _>[]
// Vector<Int, 0>
```

This succeeds because the element type is supplied explicitly.

Conversely:

```telos
let empty = <_, _>[]
```

must fail with an underconstrained element-type diagnostic.

The extent `N = 0` is known; `T` is not. Solving one parameter cannot justify fabricating a solution for the other.

### 4.4 Extent mismatch

```telos
let values: Vector<Int, 2> = [1, 2, 3]
```

must produce a semantic extent mismatch.

It must not construct a three-element object and attempt to cast it to `Vector<Int, 2>`.

### 4.5 Symbolic extent

A symbolic signature should be representable:

```telos
class Evidence<N: Int> {
  // Methods may refer to Vector<Int, N>.
}
```

P3 does not need a general integer-refinement theorem solver.

A concrete negative extent must be rejected. An abstract `N: Int` may remain a symbolic parameter, but constructing an actual Vector requires sufficient cardinality evidence to discharge the applicable extent obligation.

This is the bounded interpretation developed in the attached analysis. Pasted text

### 4.6 Family precedence

Retain the C1 rule:

```text
1. Explicit literal family
2. Unique concrete contextual family
3. Evaluation-phase default
```

Consequently:

```telos
let x: Array<Int> = <Int, 3>[1, 2, 3]
```

must report an incompatibility. The explicit two-argument head identifies Vector and cannot be reinterpreted as Array.

### 4.7 P4 compatibility boundary

During P3:

```telos
let x = [1, 2, 3]
// Still List<Int>.
```

P3 must not introduce the final bare-bracket defaults.

The following remain P4 responsibilities: bare runtime Array default, bare static Vector default, canonical static sequence values, constant Array/Vector materialization, and repository-wide literal migration.

---

# 5. Proposed P3 implementation architecture

## 5.1 Authority graph

The implementation must retain strict boundaries between syntax, static type formation, semantic inference, lowering, and execution.

```text
                         TELOS SOURCE
                              │
             ┌────────────────┴─────────────────┐
             │                                  │
      Vector<Int, 3>                      <Int, _>[...]
             │                                  │
             ▼                                  ▼
    Canonical type formation             Neutral bracket AST
             │                                  │
             │                                  ▼
             │                          Family resolution
             │                                  │
             └────────────────┬─────────────────┘
                              │
                              ▼
                   Canonical generic model
                    T : Type
                    N : StaticDomain(Int)
                              │
                              ▼
                     Semantic inference
                     ├─ Element type T
                     ├─ Cardinality proof
                     ├─ Static extent N
                     ├─ Generic constraints
                     └─ Residual obligations
                              │
                              ▼
                    Checked semantic product
                     ├─ Family = Vector
                     ├─ Result = Vector<T,N>
                     ├─ Extent = canonical N
                     └─ Expansion lane proofs
                              │
                              ▼
                       Compiler projection
                     ├─ Fixed construction
                     └─ Exact-spread construction
                              │
                              ▼
                          BYTECODE
                     ├─ BuildVector
                     └─ Private Vector builder
                              │
                              ▼
                             VM
                     ├─ Validate final extent
                     └─ Allocate VectorObject
                              │
                              ▼
                    IMMUTABLE RUNTIME VALUE
                      Box<[Value]>
```

The VM must never become an authority for deciding the type-level meaning of `N`.

## 5.2 Recommended semantic decomposition

The existing `synthesize_bracket_literal` should be refactored rather than expanded into a substantially larger special-purpose function.

Its current responsibilities include family selection, explicit-head interpretation, element synthesis, spread resolution, generic-hole solving, canonical type construction, and publication.

P3 would otherwise introduce additional complexity into an already dense function.

A preferable decomposition is:

```rust
// Conceptual semantic architecture.

resolve_sequence_family(...)
    -> SequenceFamilySelection

elaborate_sequence_generic_arguments(...)
    -> SequenceGenericArguments

analyze_sequence_elements(...)
    -> SequenceElementEvidence

analyze_sequence_cardinality(...)
    -> SequenceCardinalityEvidence

solve_sequence_application(...)
    -> SequenceTypeSolution

publish_sequence_resolution(...)
    -> SequenceLiteralResolution
```

These are responsibility boundaries, not mandated Rust symbol names.

The decomposition should preserve a single canonical inference session where the initializer and annotation jointly constrain `T` and `N`.

Avoid a nested ad hoc Vector extent solver whose result is later forced into the general inference session.

## 5.3 Cardinality evidence model

Cardinality should be a semantic property, not an interpretation of the compiler's construction strategy.

A useful starting abstraction is:

```rust
enum StaticCardinality {
    Exact(TypeId),
    RuntimeUnknown,
}
```

For additional diagnostic precision, internal analysis may also distinguish failed cardinality evidence from merely unavailable evidence.

The rules are:

```text
Ordinary literal element:
    +1

Unit positional expansion:
    +0

Closed Tuple positional expansion:
    +number of positional entries

Vector<T,N> intrinsic exact expansion:
    +N

Array<T> expansion:
    runtime unknown

List<T> expansion:
    runtime unknown

Arbitrary Iterable<T> expansion:
    runtime unknown
```

Importantly, **labeled Tuple entries do not automatically contribute to positional spread cardinality**. The proof must use the same positional lane that runtime expansion uses.

An unknown contribution makes the overall Vector cardinality unknown, regardless of whether the runtime source exposes a `size` property.

### Concrete and symbolic arithmetic

For a fully concrete literal:

```telos
let a = <Int, _>[1, 2, 3]
```

the equation is straightforward:

```text
N = 1 + 1 + 1
N = 3
```

For:

```telos
let a = <Int, _>[0, *(1, 2), 3]
```

the equation is:

```text
N = 1 + 2 + 1
N = 4
```

For a symbolic source:

```telos
// Within a generic scope where N: Int:
let result: Vector<Int, N> = <Int, N>[*source]
```

where `source: Vector<Int, N>`, the extent relationship can potentially be proved by direct symbolic equality, provided the runtime expansion is intrinsically exact.

However:

```telos
// Within a generic scope:
let result = <Int, _>[0, *source]
```

requires the term:

```text
N + 1
```

If the current canonical static-term engine cannot represent and normalize this symbolic expression, P3 must not silently substitute a runtime length or invent a Vector-specific arithmetic solver.

The bounded solution is to support concrete cardinality sums and already-provable symbolic equalities. More general symbolic arithmetic remains subject to the shared static-expression architecture.

## 5.4 Cardinality and construction must be separate

The existing:

```rust
enum SequenceConstructionPlan {
    Fixed { element_count: u32 },
    Expanded,
}
```

answers a construction question:

> Will the VM consume a known number of individual stack operands, or must it expand one or more sources?

It does not answer:

> Is the final sequence cardinality statically known?

An Array with spreads may have `Expanded` construction and runtime-unknown cardinality.

A Vector with spreads may have `Expanded` construction and **exact static cardinality**.

Therefore, preserve separate concepts:

```rust
struct SequenceLiteralResolution {
    // Existing fields...
    family: SequenceLiteralFamily,
    result_type: TypeId,
    element_type: TypeId,

    // Canonical exact static term for Vector.
    extent: Option<TypeId>,

    // Physical construction strategy.
    construction: SequenceConstructionPlan,

    // Proposed extension:
    cardinality: SequenceCardinalityEvidence,
    expansions: Box<[SequenceExpansionEvidence]>,
}
```

The exact final structure should avoid redundant copies of the same semantic fact. The important requirement is that every checked Vector expansion has auditable cardinality evidence and a corresponding lowering mode.

## 5.5 Runtime storage model

The recommended representation remains:

```rust
pub struct VectorObject {
    elements: Box<[Value]>,
}
```

This is structurally different from Array:

```rust
pub struct ArrayObject {
    elements: Box<[Value]>,
    frozen: bool,
}
```

Vector must not require a frozen flag because there is no mutable Vector state to freeze.

It must not expose:

```rust
fn set(...)
fn push(...)
fn append(...)
fn insert(...)
fn remove(...)
```

and should not provide a general mutable heap accessor.

A shared private read-only dense-storage utility is acceptable. A shared mutable Array/Vector object with behavior flags is not.

### Runtime type erasure

For v1, runtime instances need not contain a separate descriptor for `T` or `N`.

The canonical type system determines these during checking. Runtime storage records actual elements.

Every Vector construction path must preserve:

```text
actual_element_count == semantically_proved_N
```

This invariant belongs to the construction contract.

---

# 6. Detailed implementation plan

The recommended implementation is organized into nine stages. They are sequenced by architectural dependencies rather than by which Rust module is easiest to edit.

## Stage P3.0 — Takeover audit, record reconciliation, and prerequisite probes

**Objective:** Establish a trustworthy branch baseline and prove the generic-static boundaries before creating Vector runtime infrastructure.

### Work

1. Confirm the branch HEAD, working-tree cleanliness, and P1/P2 commit ancestry.
2. Re-run P1/P2 focused semantic and runtime tests.
3. Run the full workspace build check.
4. Record the verified P2 interface map.
5. Correct the lifecycle records' stale implementation-state text.
6. Reconcile proposed amendments A5/A6 with the checkpoint's accepted architecture.
7. Introduce minimal synthetic tests for static-domain generic parameters.

### Required synthetic probes

```telos
class Sized<T, N: Int> {}

class Example {
  @class test() {
    // Probe complete application:
    let a: Sized<Int, 3>

    // Probe partial forms independently:
    // Sized<_, 3>
    // Sized<Int, _>
    // Sized<_, _>
  }
}
```

The implementer should use appropriate test fixtures that supply initializer evidence where inference is required. These examples are type-formation targets, not a promise that uninitialized `let` declarations are valid.

The tests must inspect canonical binder kinds and argument representations, rather than merely asserting that parsing succeeds.

### Entry-gate decisions

| Gate | Requirement |
|---|---|
| G0.1 | P1/P2 focused tests pass on the exact takeover revision |
| G0.2 | Static-domain binder syntax is recognized semantically |
| G0.3 | Exact static integer representation decision is recorded |
| G0.4 | Metadata ownership for heterogeneous binders is resolved |
| G0.5 | Vector spread execution semantics are defined |
| G0.6 | Existing Array/List default behavior is unchanged |

**Exit:** The implementer has confirmed the root causes and resolved the necessary architectural amendments. No Vector-specific workaround has been introduced.

---

## Stage P3.1 — Exact static integers and canonical heterogeneous type formation

**Objective:** Finish the general type-system capabilities that make `Vector<T, N>` an ordinary Telos type.

This is P3's most important foundational stage.

### Task P3.1.1 — Unify exact static integer identity

Extend the shared static term representation so canonical type-level `Int` values support the same mathematical integer range as Telos's static-value model.

The change must cover:

- `TypeData::ConstInt`;
- `TypeStore::const_int` and related constructors;
- static integer interning and normalization;
- generic type application;
- inference-term materialization;
- equality and hashing;
- stable metadata serialization;
- type-member static arguments;
- type and semantic fingerprints;
- reflection and diagnostic rendering.

The exact integer model should remain normalized:

```text
Small integer
    -> efficient immediate representation

Large integer
    -> exact arbitrary-precision representation

Equal mathematical values
    -> equal canonical static terms
```

A metadata encoding may use a portable canonical integer representation, but it must not collapse distinct integers or depend on host pointer width.

### Task P3.1.2 — Repair complete static generic argument formation

Modify the non-variadic generic application path to elaborate arguments according to their actual parameter kinds.

Expected result:

```telos
Sized<Int, 3>
```

forms:

```text
Sized<
    Int,
    ConstInt(3)
>
```

not:

```text
Sized<
    Int,
    Int
>
```

Likewise, when `N` is an in-scope static binder:

```telos
Sized<Int, N>
```

must retain the symbolic static term associated with that binder.

### Task P3.1.3 — Repair partial annotation elaboration

Use the same parameter-directed interpretation for mixed explicit arguments and inference holes.

The second parameter's declared domain must determine the hole kind.

Examples:

```text
Sized<_, 3>
    T := fresh Type variable
    N := canonical 3

Sized<Int, _>
    T := Int
    N := fresh static-Int variable

Sized<_, _>
    T := fresh Type variable
    N := fresh static-Int variable
```

No unsolved hole may enter durable canonical type identity.

### Task P3.1.4 — Repair Universe bootstrap

Provide static-domain resolution during canonical source predeclaration without lying about the binder's kind.

Correct:

```text
T : Type
N : StaticDomain(Int)
```

Incorrect:

```text
T : Type
N : Type
```

The bootstrap fix must preserve the source declaration as the canonical authority.

### Task P3.1.5 — Reconcile native metadata and reflection

The existing `KindSpec` supports `Type` and arrow kinds, but not the required static integer domain.

Extend metadata projection if necessary so native registration, generic parameter reflection, and canonical source agree.

This must not introduce an independently authored Vector generic signature that can drift from the Universe declaration.

### Required verification

| Test | Expected result |
|---|---|
| `Sized<Int, 3>` | Valid canonical application |
| `Sized<Int, 4>` | Distinct from `Sized<Int, 3>` |
| `Sized<String, 3>` | Distinct from `Sized<Int, 3>` |
| `Sized<_, 3>` | Correct first-slot inference kind |
| `Sized<Int, _>` | Correct second-slot inference kind |
| `Sized<_, _>` | Two correctly kinded variables |
| `Sized<Int, Int>` | Static-domain kind error |
| `Sized<3, Int>` | First argument kind error |
| Exact integer beyond `i64::MAX` | No semantic truncation |
| Exact integer below `i64::MIN` | No semantic truncation |
| Cross-module metadata round-trip | Identity preserved |
| Repeated canonical formation | Same canonical identity |
| Unsolved inference publication | Rejected |

**Exit gate:** `Vector<T, N>` requires no special representation in `TypeStore`.

---

## Stage P3.2 — Canonical Vector Universe identity

**Objective:** Establish Vector as a normal canonical nominal collection family.

### Required source declaration

```telos
from universe.collections.iterator_laws import (
  Iterator as CanonicalIterator,
  Iterable as CanonicalIterable
)

native class Vector<T, N: Int> is Collection {
  // Native read-only storage floor.
}
```

The declaration must use the actual supported Telos source syntax. Do not create a special built-in type form to bypass static-domain syntax.

### Required integration

| Subsystem | Required addition |
|---|---|
| Native Universe identities | `UniverseKey::Vector` |
| Built-in modules | `universe.collections.vector` |
| Universe package | Export Vector |
| Semantic core IDs | `CoreDeclarationIds.vector` |
| Type helpers | Canonical Vector application helper where useful |
| Runtime class registry | `vector_class` |
| Class hierarchy | Vector derives from nominal `Collection` |
| Reflection | Correct generic parameter kinds |
| Native surface | Minimal Vector primitive registration |
| Tests | Canonical identity and class-tower verification |

### Concrete negative extent validation

P3 must reject source-formed concrete negative Vector extents, for example:

```telos
Vector<Int, -1>
```

This requires a type-formation validation boundary.

Because the current constraint language does not support arbitrary arithmetic predicates, the initial implementation can use a narrow canonical nominal well-formedness rule for Vector's concrete extent.

That validation must be centralized and invoked wherever a closed Vector application becomes publishable. It must not exist separately in the parser, annotation resolver, literal checker, and compiler.

If the existing declaration-constraint infrastructure provides an appropriate general mechanism, prefer that mechanism. Otherwise, record the bounded intrinsic formation rule explicitly.

A symbolic `N: Int` remains acceptable until the relevant value-construction proof is required.

### Type identity tests

```text
Vector<Int, 3> == Vector<Int, 3>
Vector<Int, 3> != Vector<Int, 4>
Vector<Int, 3> != Vector<String, 3>
Vector<Int, 0> is valid
Vector<Int, -1> is rejected
```

No runtime object needs to be allocated for these tests.

**Exit gate:** Vector's canonical source identity, generic signature, kind, and reflection are established independently of literal construction.

---

## Stage P3.3 — Exact cardinality and inference integration

**Objective:** Make Vector cardinality a proof-producing semantic operation.

### Task P3.3.1 — Introduce exact cardinality analysis

Analyze every bracket-literal element and expansion while preserving:

- element-type evidence;
- positional source identity;
- exact or unknown cardinality;
- the runtime lane that will realize the expansion.

### Task P3.3.2 — Distinguish exact and runtime-sized sources

Accept exact positional cardinality from established closed Tuple/Unit products and dimensioned Vectors.

Reject runtime-unknown spreads from Array, List, and general Iterable sources in Vector construction.

Do not infer static `N` from a runtime `size` method, even when that method is O(1).

### Task P3.3.3 — Solve `N` through the canonical inference session

For:

```telos
let values: Vector<Int, _> = [1, 2, 3]
```

the semantic checker must introduce the equivalent constraint:

```text
N == ConstInt(3)
```

into the existing query-local inference graph.

For:

```telos
let values: Vector<Int, 2> = [1, 2, 3]
```

the result is a contradiction between canonical static terms:

```text
ConstInt(2) != ConstInt(3)
```

The diagnostic must describe an extent mismatch.

### Task P3.3.4 — Preserve mandatory residual obligations

Do not publish a solved Vector type if the extent equality remains unproved.

In particular, distinguish:

```text
I can render a symbolic N.
```

from:

```text
I proved this construction contains exactly N elements.
```

The first does not imply the second.

### Task P3.3.5 — Support all explicit/contextual combinations

Test both literal-head arguments and contextual annotations with:

```text
T explicit, N explicit
T explicit, N inferred
T inferred, N explicit
T inferred, N inferred
```

The implementation must establish the same canonical result in equivalent cases.

### Task P3.3.6 — Refactor invalid-state handling

An underconstrained or invalid literal must produce diagnostics without publishing a plausible but unproved Vector construction product.

### Required tests

```telos
<Int, _>[]
// Vector<Int, 0>

<_, _>[]
// Error: T underconstrained.

<Int, 2>[1, 2, 3]
// Error: extent mismatch.

<Int, _>[1, *(2, 3), 4]
// Vector<Int, 4>.
```

For a statically dimensioned source:

```telos
let left = <Int, 2>[2, 3]
let right = <Int, _>[1, *left]

// Vector<Int, 3>.
```

For a runtime-sized source:

```telos
let source = <Int>[1, 2, 3]

let invalid = <Int, _>[0, *source]
// Error: exact static cardinality unavailable.
```

**Exit gate:** The semantic checker can prove or reject every required cardinality case without VM execution.

---

## Stage P3.4 — Sequence semantic product and lowering contract

**Objective:** Make the checked Vector construction executable without rediscovering its semantics.

### Required semantic product

The result should contain, directly or through checked associated products:

```rust
SequenceLiteralResolution {
    family: SequenceLiteralFamily::Vector,

    result_type: vector_type,
    element_type: element_type,

    extent: Some(canonical_static_n),

    construction: checked_construction,

    // Additional exact expansion evidence.
}
```

### Required compiler-facing projection

The compiler-facing product should preserve:

```text
Family identity
Canonical result type or required type recipe
Canonical extent evidence
Checked construction strategy
Exact-spread lowering modes
Source mappings
```

The compiler may receive a checked host-representable count derived from the semantic extent. Such a count is a lowering projection, not a new semantic identity.

### Required invariant

```text
semantic extent
    ↓ checked projection
bytecode expected extent
    ↓ runtime verification
actual Vector storage length
```

There must be no alternate path:

```text
AST element count
    ↓
compiler independently decides Vector N
```

### Additional corrections

During this stage, fix the P2/P1 explanation-family mismatch and ensure invalid resolutions are not projected to the compiler.

Update semantic fingerprints so changes in `N`, exactness, or spread execution mode invalidate the appropriate products.

**Exit gate:** The compiler can distinguish a checked Vector literal from Array/List solely through semantic products.

## Stage P3.5 — Runtime representation, native primitives, and source API

**Objective:** Introduce a native immutable Vector representation while keeping collection semantics source-authored.

### Task P3.5.1 — Implement `VectorObject`

Create a dedicated runtime representation:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct VectorObject {
    elements: Box<[Value]>,
}

impl VectorObject {
    pub fn from_boxed_slice(elements: Box<[Value]>) -> Self {
        Self { elements }
    }

    pub fn elements(&self) -> &[Value] {
        &self.elements
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn get(&self, index: usize) -> Option<Value> {
        self.elements.get(index).copied()
    }
}
```

This is a representative interface, not a literal patch prescription. Constructor visibility should be limited to trusted VM/heap construction boundaries.

**No general-purpose mutable accessor should exist.**

The representation must not carry:

```text
frozen: bool
capacity: usize
element_type: TypeId
extent: TypeId
```

The first two are unnecessary because the collection is permanently immutable and fixed-sized. The latter two belong to the semantic type system.

### Task P3.5.2 — Integrate Vector throughout the object model

Follow the established Array integration pattern.

Required areas include:

| Repository area | Responsibility |
|---|---|
| `phalcom-core/src/heap/vector.rs` | Immutable storage |
| `phalcom-core/src/heap/object.rs` | `Object::Vector` |
| `phalcom-core/src/heap/mod.rs` | Allocation and exports |
| `phalcom-core/src/heap/accessors.rs` | Read-only Vector access |
| `phalcom-core/src/heap/trace.rs` | Precise element tracing |
| `phalcom-core/src/universe/core_classes.rs` | Canonical runtime class |
| `phalcom-core/src/universe/invariants.rs` | Census and object-model invariants |
| `phalcom-core/src/value/mod.rs` | Class dispatch |
| `phalcom-core/src/value/render.rs` | Debug rendering |
| `phalcom-core/src/primitive/mod.rs` | Native primitive registration |
| `phalcom-core/src/primitive/vector.rs` | Minimal read-only primitive implementation |

Every exhaustive `Object` match must be audited.

Do not limit integration to the files that fail compilation. Object identity, sizing, GC, class dispatch, debug rendering, reflection, and native surface generation are all relevant.

### Task P3.5.3 — Provide the minimal native floor

The recommended native operations are:

```telos
internal native rawLength -> Int
internal native rawAt(_ index: Int) -> T
```

The read primitive must have a rigorously defined bounds contract.

If it returns `T`, an invalid index cannot be represented by returning `None` as though that were an arbitrary valid `T`. Use one of the existing checked-access conventions, or ensure the source-owned public method establishes valid bounds before invoking an internal unchecked accessor.

No mutation primitive may be registered.

### Task P3.5.4 — Implement Vector behavior in Telos source

Recommended v1 public API:

| Operation | Semantics |
|---|---|
| `size` | Element count |
| `first` | Optional first element |
| `last` | Optional last element |
| `at(_)` | Checked or optional indexed access, following established convention |
| `get(_)` | Optional indexed access |
| `[_]` | Indexed access |
| `iterator` | Typed, ordered iteration |
| `==(_)` | Same-family structural equality |
| `!=(_)` | Negation of equality |
| `hash` | Ordered structural hash |
| `toString` | Source-authored rendering |
| `toArray` | Copy into mutable Array |
| `toList` | Copy into resizable List |

The exact signatures should preserve the repository's existing conventions where those conventions are sound.

The principal typing requirements are:

```text
Vector<T,N> indexed element -> T

Vector<T,N> iterator item -> T

Vector<T,N> does not expose mutation
```

### Task P3.5.5 — Implement `VectorIterator<T, N>`

A natural representation is:

```telos
class VectorIterator<T, N: Int> {
  // Privately stored source:
  // Vector<T, N>

  // Privately stored cursor:
  // Int

  // Constructor and next implementation.
}
```

Use the canonical `Iterator` and `Iterable` protocols already established in the Universe.

For constructor style, prefer Telos's `@constructor call(...)` convention where supported and appropriate, rather than introducing a new `new`-centric API merely because Array currently uses one.

The iterator must retain the Vector as a GC root and return elements in storage order.

### Task P3.5.6 — Structural equality

Vector equality should use:

```text
same runtime collection family
AND same element count
AND pairwise ordered element equality
```

It must not compare Vector to List, Array, or Tuple as though they were the same collection family.

Because a Vector instance does not carry reified generic parameters, runtime equality should not require reading a separate stored `N`.

### Task P3.5.7 — Structural hash

Implement structural hashing in Telos source, following the Tuple precedent:

```text
initial family-sensitive accumulator
    ↓
fold ordered element hashes
    ↓
final hash
```

Required law:

```text
a == b  =>  a.hash == b.hash
```

The implementation must not cache the hash in P3.

Cross-family hash collisions are permissible; hash equality does not imply value equality.

### Task P3.5.8 — Map/Set key integration

Vector must not be included in `is_mutable_collection_key` simply because it is a collection.

Test ordinary stable Vector keys in both Map and Set.

Record the recursive mutable-content caveat described in A13 without silently changing the entire keyed-collection contract.

### Task P3.5.9 — Constructor scope

P3 does not require a public:

```telos
Vector.call(*items)
```

if that operation cannot statically derive a correct return dimension.

Likewise, avoid a generic `filled` constructor that requires runtime reification of an otherwise erased `N`.

P3's required constructors are compiler-controlled literal constructions and any strictly typed internal construction operations necessary to support them.

Broader public factories and conversions belong to the C1 API convergence work unless their typing is already expressible without special mechanisms.

**Exit gate:** A directly allocated Vector has correct runtime identity, read-only behavior, typed iteration, equality, hash, and GC tracing.

---

## Stage P3.6 — Direct Vector bytecode and exact-spread construction

**Objective:** Lower checked Vector literals directly to immutable Vector storage.

### Task P3.6.1 — Fixed literals

For:

```telos
let values = <Int, 3>[1, 2, 3]
```

the compiler should emit the equivalent of:

```text
Evaluate 1
Evaluate 2
Evaluate 3
BuildVector(3)
```

A reasonable first-generation instruction is:

```rust
Bytecode::BuildVector(u16)
```

The compiler must verify that its operand count corresponds to the checked construction product.

The bytecode's machine-width operand is not the semantic representation of `N`.

If a valid fixed literal exceeds the direct instruction's operand capacity, use an appropriate private builder path or an explicitly supported larger-count lowering strategy. Do not silently truncate the count.

### Task P3.6.2 — Private Vector builder

For expanded literals:

```telos
let source = <Int, 2>[2, 3]
let values = <Int, _>[1, *source]
```

the compiler should use a private growable construction buffer.

Conceptually:

```text
BeginVectorLiteral
    ↓
Append element 1
    ↓
Expand source's checked positional lane
    ↓
FinishVectorLiteral(expected extent = 3)
    ↓
VectorObject
```

A growable `Vec<Value>` is acceptable during construction.

The final published Vector remains fixed and immutable.

### Task P3.6.3 — Exact Vector positional expansion

Implement the proof-preserving expansion mode established in Stage P3.3.

For an exact Vector source, the compiler must select the checked intrinsic positional lane rather than an arbitrary overridable iterator.

For a closed Tuple source, it must select the correct positional product lane.

For an unknown Iterable source, Vector construction must already have been rejected semantically.

### Task P3.6.4 — Move-based finalization

Prefer consuming the builder's owned storage rather than cloning it.

An in-place heap transition may be particularly attractive if it can safely replace the private builder object with the final Vector object while preserving the object handle and collector invariants.

The implementation must ensure:

- no temporary unrooted elements exist across allocation or collection;
- builder values are traced until ownership transfers;
- conversion executes only after the expected extent check;
- failed construction cannot expose a partially initialized Vector;
- the private builder cannot be observed or mutated by guest code.

### Task P3.6.5 — Final cardinality assertion

The finalization boundary must check:

```text
actual payload length == checked expected extent
```

A mismatch indicates a violated compiler/runtime invariant.

It must not be presented as an ordinary source-level Vector-size error when semantic analysis had already certified the construction.

The implementation should report an internal invariant failure with sufficient diagnostic context to identify the originating literal.

### Task P3.6.6 — Rooting and reentrancy

Preserve the existing expansion compiler's scratch-rooting discipline.

In particular, executing user code while producing a literal must not leave the builder, source, iterator, or current item reachable only through an untraced Rust temporary.

Test callbacks that allocate, trigger GC, suspend execution where supported, or throw exceptions.

### Required bytecode assertions

For a fixed literal:

```text
Contains BuildVector
Does not contain BuildList
Does not contain BuildArray
```

For an expanded literal:

```text
Contains private Vector builder construction
Contains checked exact expansion
Contains Vector finalization
Does not allocate an intermediate List
Does not allocate an intermediate Array
```

**Exit gate:** Every valid P3 Vector literal constructs its target collection directly from checked semantic evidence.

---

## Stage P3.7 — Diagnostics, reflection, and incremental correctness

**Objective:** Ensure the feature is understandable, inspectable, and safe under incremental recompilation.

### Diagnostics

Provide deterministic diagnostic behavior for:

| Scenario | Required diagnostic meaning |
|---|---|
| `<Int, 2>[1, 2, 3]` | Extent mismatch |
| `Vector<Int, -1>` | Negative concrete extent |
| `<_, _>[]` | Element type underconstrained |
| `<Int, _>[*someArray]` | Cardinality not statically established |
| `<Int, _>[*someList]` | Cardinality not statically established |
| `<Int, _>[*someIterable]` | Cardinality not statically established |
| `<String, _>[1, 2]` | Element type incompatibility |
| `<Int, String>[...]` | Second generic argument has incorrect domain |
| `<3, 3>[...]` | First generic argument is not a type |
| Explicit Vector head under Array context | Collection-family conflict |
| Unresolved symbolic extent proof | Inference underconstraint or unproved obligation |
| Missing checked lowering product | Internal compilation boundary error |

Dedicated diagnostic codes such as `VectorExtentMismatch`, `VectorExtentNegative`, and `VectorExtentNotStatic` are appropriate if they fit the repository's existing diagnostic taxonomy.

Do not introduce redundant codes merely for cosmetic distinction.

### Reflection

Verify that reflection and exported metadata preserve:

```text
Vector:
    parameter 0: Type
    parameter 1: static Int domain
```

Reflection must not report the second parameter as an ordinary type parameter.

### Incremental invalidation

Test that changing:

```telos
<Int, 3>[1, 2, 3]
```

to:

```telos
<Int, 4>[1, 2, 3, 4]
```

changes all relevant semantic products and dependent type identities.

Likewise, changing an expansion source from exact to runtime-unknown must invalidate any previously accepted Vector construction.

The fingerprint must include facts governing generated behavior, including the cardinality and exact expansion modes.

**Exit gate:** Diagnostics, metadata, and incremental products agree with the canonical semantic result.

---

## Stage P3.8 — Comprehensive verification

**Objective:** Verify not only that Vector works, but that its fundamental invariants cannot be violated by the compiler, runtime, or incremental pipeline.

### 6.8.1 Semantic test matrix

| Category | Cases | Required result |
|---|---|---|
| Generic kinding | `T: Type`, `N: Int` | Correct kinds |
| Static values | Concrete `3`, symbolic `N` | Canonical static terms |
| Exact integer width | `N` beyond `i64` | Exact type identity |
| Type identity | Different `T` or `N` | Distinct canonical types |
| Zero extent | `<Int, _>[]` | `Vector<Int, 0>` |
| Empty underconstraint | `<_, _>[]` | Error |
| Negative extent | `Vector<Int, -1>` | Error |
| Explicit heads | Four explicit/hole combinations | Correct inference |
| Contextual typing | Four explicit/hole combinations | Correct inference |
| Extent mismatch | Expected 2, constructed 3 | Error |
| Fixed cardinality | Three ordinary elements | `N = 3` |
| Tuple spread | Exact positional arity | Correct `N` |
| Vector spread | Exact source dimension | Correct `N` |
| Multiple exact spreads | Composite sum | Correct `N` |
| Array spread | Runtime extent only | Error |
| List spread | Runtime extent only | Error |
| Generic Iterable spread | No static cardinality | Error |
| Symbolic equality | Already-provable `N == N` | Accepted where supported |
| Symbolic arithmetic | Unproved `N + 1` | Not silently accepted |
| Residual obligations | Mandatory extent constraint unresolved | No published Vector result |
| Element incompatibility | Wrong element type | Error |
| Family conflict | Explicit Array versus contextual Vector | Error |
| Inference escape | Unsolved `_` | No durable canonical publication |

### 6.8.2 Runtime test matrix

| Category | Required proof |
|---|---|
| Runtime class | Value is an instance of canonical Vector |
| Storage | Ordered contiguous fixed slots |
| Immutability | No element mutation |
| Resize | No structural resize methods |
| Indexing | First, last, negative, and out-of-range behavior |
| Iteration | Correct order and typed elements |
| Iterator lifetime | Iterator retains source |
| Equality | Reflexivity and symmetry |
| Equality order | Permutations compare unequal |
| Cross-family comparison | Vector differs from Array/List/Tuple |
| Hash | Equal values produce equal hashes |
| Map/Set | Stable Vector keys work |
| GC | Vector traces every contained reference |
| Nested GC | Nested Vectors retain descendants |
| Failure cleanup | Failed builder is not leaked to guest code |
| Finalization | No builder payload cloning |
| Direct allocation | No intermediate Array/List |
| Large extent | Checked allocation/resource boundary |
| Cardinality mismatch | Internal invariant failure |

### 6.8.3 Compiler and VM adversarial tests

The following tests are particularly important.

**Test 1 — Missing semantic metadata**

Compile an explicit Vector literal through a route lacking the required checked sequence product.

Expected: no Array fallback and no invented Vector identity.

**Test 2 — Incorrect expected extent**

Construct a synthetic internal lowering test with inconsistent cardinality, using an appropriate private test facility.

Expected: the compiler or VM rejects the violated invariant rather than publishing an incorrectly dimensioned Vector.

**Test 3 — Overridden iterator**

Use a Vector source whose ordinary iteration behavior can be customized, if the object model permits such customization.

Expected: exact Vector literal expansion follows the ratified proof-preserving positional lane; it must not accidentally use an arbitrary iterator count.

**Test 4 — GC during element evaluation**

Construct a Vector whose element expressions allocate and trigger GC.

Expected: previously constructed elements remain alive throughout construction.

**Test 5 — Nested exact expansions**

Construct a Vector from multiple Tuple and Vector spreads.

Expected: element order and computed cardinality agree exactly.

**Test 6 — Large static dimensions**

Form a Vector type whose `N` is too large for a runtime allocation.

Expected: canonical type identity remains exact; any attempted materialization fails at the appropriate resource boundary without overflow or truncation.

### 6.8.4 P1/P2 regression requirements

P3 must preserve:

```text
let x = [1, 2, 3]
// List<Int> until P4.

let y = <Int>[1, 2, 3]
// Array<Int>.

let z: Array<Int> = [1, 2, 3]
// Array<Int>.
```

Also verify:

- List remains dynamically resizable.
- Array retains fixed extent and mutable elements.
- Array/List remain rejected as mutable Map/Set keys.
- Array's existing direct lowering continues to work.
- Existing Array spread behavior remains valid.
- Existing Tuple/Pack expansion remains correct.
- Existing inference and static-domain tests continue to pass.

### 6.8.5 Suggested verification commands

At minimum:

```bash
RUSTFLAGS='' cargo test \
  -p phalcom-semantic \
  --test semantic coll006

RUSTFLAGS='' cargo test \
  -p phalcom-core \
  --test core core_array

RUSTFLAGS='' cargo test \
  -p phalcom-core \
  --test core core_vector

RUSTFLAGS='' cargo check \
  --workspace --all-targets

git diff --check
```

The Vector test filter is proposed; use the test name actually registered by the implementation.

Run additional focused metadata, static integer, object-model, GC, and incremental tests corresponding to the changed subsystems.

A successful build is not evidence that static generic arguments or exact-spread invariants are correct.

**Exit gate:** P3 has positive, negative, compiler, runtime, metadata, GC, and incremental evidence for each required invariant.

---

## Stage P3.9 — Lifecycle closure and P4 handoff

**Objective:** Close P3 truthfully and leave an executable takeover contract for P4.

### Required records

Update:

```text
docs/implementation/COLL006/
├── PROGRAM.md
├── STATUS.md
└── COLL006.C1/
    ├── COLL006.C1-CHECKPOINT.md
    ├── COLL006.C1.P3-implementation-plan.md
    ├── COLL006.C1.P3-walkthrough.md
    └── COLL006.C1.P3-handoff.md
```

The proposed P3 plan filename may be adapted to the repository's established naming convention.

### Walkthrough requirements

Record:

- implemented semantic interfaces;
- generic-static formation corrections;
- canonical integer representation changes;
- runtime object integration;
- exact-spread design;
- compiler and bytecode implementation;
- test commands and actual results;
- known deviations;
- architectural consultations and decisions;
- unresolved issues and their precise ownership.

### P4 handoff requirements

P4 must receive a stable interface for:

```text
Vector<T,N> canonical type formation
Vector literal semantic resolution
Exact extent proof
Runtime Vector allocation
Vector GC representation
Vector equality/hash semantics
Direct Vector lowering
Static-phase sequence checking
```

Do not state that const Vector construction is complete unless the static-value evaluation and materialization mechanisms have actually been implemented.

### Completion criteria

P3 is complete only when its required code and tests pass, its architecture remains consistent with C1 decisions, and all unresolved issues are either corrected or explicitly classified as approved deferred work.

---

# 7. Explicit P3 invariants

These should be copied into the implementation plan and used as review gates.

| ID | Invariant |
|---|---|
| `P3-TYPE-01` | Vector has exactly two generic parameters, `T: Type` and `N: Int` static domain. |
| `P3-TYPE-02` | `N` participates in canonical applied-type identity. |
| `P3-TYPE-03` | Canonical `N` preserves exact Telos integer semantics. |
| `P3-TYPE-04` | Concrete negative Vector extents are rejected. |
| `P3-TYPE-05` | Inference holes use their declared parameter kinds. |
| `P3-TYPE-06` | No unsolved hole enters durable type or compiler products. |
| `P3-TYPE-07` | Vector generic identity is source-authoritative, not native-metadata-authoritative. |
| `P3-EXT-01` | Every materialized Vector contains exactly its proved `N` elements. |
| `P3-EXT-02` | Extent inference uses the canonical inference solver. |
| `P3-EXT-03` | Runtime-unknown cardinality cannot establish `N`. |
| `P3-EXT-04` | Every exact spread uses an execution lane consistent with its proof. |
| `P3-EXT-05` | Unresolved mandatory cardinality obligations prevent publication. |
| `P3-IMM-01` | Vector exposes no element mutation. |
| `P3-IMM-02` | Vector exposes no structural resizing. |
| `P3-REP-01` | Final storage is immutable, fixed contiguous `Value` slots. |
| `P3-REP-02` | Runtime Vector does not require per-instance `T`/`N` descriptors. |
| `P3-COMP-01` | Compiler consumes checked family and extent evidence. |
| `P3-COMP-02` | Compiler never independently reconstructs semantic `N`. |
| `P3-COMP-03` | Vector construction never stages through List or Array. |
| `P3-COMP-04` | Missing required lowering products fail closed. |
| `P3-GC-01` | Vector and private builder elements remain precisely traced. |
| `P3-GC-02` | No unrooted temporary values survive across GC-capable operations. |
| `P3-EQ-01` | Equality is same-family, ordered, and structural. |
| `P3-HASH-01` | Equal Vectors have equal structural hashes. |
| `P3-INCR-01` | Changes in family, extent, and exact expansion semantics invalidate dependent products. |
| `P3-BOUND-01` | P3 does not perform the P4 bare-literal default cutover. |
| `P3-BOUND-02` | P3 does not introduce a private static sequence evaluator or materialization system. |

---

# 8. STOP AND CONSULT conditions

The implementer must inherit the checkpoint's existing stop conditions and add the following P3-specific triggers.

| Trigger | Reason |
|---|---|
| Exact static `Int` requires a Vector-only numeric representation | Violates canonical static identity |
| Metadata cannot preserve exact `N` without truncation | Violates type identity |
| Universe bootstrap requires declaring `N: Type` | Violates declared parameter semantics |
| Static value arguments require a second type resolver | Fragments generic formation |
| `_` requires fallback to `Dynamic` | Violates inference semantics |
| A Vector-specific solver is needed for elementary extent equality | Violates inference ownership |
| Exact spread cardinality cannot be reconciled with runtime expansion | **Soundness blocker** |
| A symbolic extent is accepted without discharging mandatory obligations | Unsound Vector construction |
| Source and runtime disagree about Vector's generic shape | Canonical identity split |
| Vector must reuse a mutable Array object | Weakens structural immutability |
| Runtime construction requires List/Array staging | Violates direct construction |
| A public factory requires unapproved runtime generic reification | Expands the language contract |
| Negative extent checking requires a new general arithmetic theorem solver | Unbounded scope expansion |
| Vector hashability requires changing global Map/Set semantics | Cross-family architectural decision |
| Const Vector needs a separate P3 static-value representation | Violates P4/C4 ownership |
| Tests can pass only by weakening an established C1 invariant | Invalid completion |

**Particularly important:** The static integer and metadata changes identified by C1 amendments A5/A6 should be recorded as approved checkpoint architecture before the implementer performs broad cross-subsystem edits. They are not ordinary Vector-local implementation details.

---

# 9. Recommended implementation ordering and dependency gates

The proposed execution structure is:

```text
P3.0  Repository takeover and architectural gates
 │
 ▼
P3.1  Exact static Int + generic static application
 │
 ▼
P3.2  Canonical Vector<T,N> formation and Universe identity
 │
 ▼
P3.3  Exact cardinality and solver integration
 │
 ▼
P3.4  Checked semantic/lowering contract
 │
 ├─────────────────────────────┐
 ▼                             ▼
P3.5                         P3.6
Runtime Vector                Direct compiler/VM lowering
 │                             │
 └──────────────┬──────────────┘
                ▼
P3.7  Diagnostics, metadata and incremental correctness
                │
                ▼
P3.8  Full focused verification matrix
                │
                ▼
P3.9  Lifecycle closure and P4 handoff
```

The runtime representation and compiler lowering can be developed in parallel after their shared semantic contract is established, provided they agree on the same checked extent and construction invariants.

Do **not** begin by implementing `VectorObject` and then attempt to repair static generic formation around it. That order would encourage local special cases and obscure the most consequential soundness problems.

---

# 10. Final architectural recommendations

## 10.1 The attached P3 requirements analysis is fundamentally correct, but incomplete

Its central conclusions remain valid:

- Vector must use canonical static arguments.
- Extent inference belongs to semantic checking.
- Query-local inference must solve `N`.
- Exact cardinality cannot be inferred from runtime-sized collections.
- Immutable storage should be structurally distinct from Array.
- Compiler lowering must consume semantic evidence.
- Static evaluation and materialization remain P4 responsibilities.

The analysis correctly identifies P1's static-argument elaboration defect and the Universe bootstrap problem. Pasted text

However, the remote repository reveals additional work that must become part of the actual plan.

## 10.2 Five findings deserve the highest implementation priority

**First — exact integer semantics.** `ConstInt(i64)` is not a sufficient final representation for a dimension parameter whose domain is Telos `Int`. Preserve `TypeId` identity while correcting its underlying exact integer representation.

**Second — generic source formation.** Repair complete and partial static arguments through the same canonical parameter-directed machinery. `Vector<Int, 3>` must be an ordinary source-level type application.

**Third — proof-preserving spreads.** This is potentially the most dangerous overlooked issue. Knowing that a source has type `Vector<T, N>` does not prove that an arbitrary user-overridable iterator yields exactly `N` items. The expansion operation itself must preserve the semantic cardinality proposition.

**Fourth — semantic product integrity.** Stop publishing fallback or invalid sequence resolutions. Extend the compiler-facing product with the checked extent and exact expansion information. Remove unsafe Vector family reconstruction from compiler compatibility paths.

**Fifth — source API typing.** P2's use of `Dynamic` for Array element access and replacement should not become Vector's model. A homogeneous generic sequence requires `T`-typed public operations, even when its native storage contains erased `Value` slots.

## 10.3 Recommended scope discipline

P3 should correct the P1 defects that prevent canonical Vector formation and the P2 architectural seams directly required for safe Vector construction.

It should not turn into an unrestricted rewrite of Array, List, Tuple, Map, Set, or the entire collection hierarchy.

The distinction matters:

| Required within P3 | Better treated as a separate corrective follow-up |
|---|---|
| Static generic argument correctness | General Array API redesign beyond type soundness |
| Exact canonical integer identity | Unrelated numeric-library changes |
| Vector Universe bootstrap | Broad native-metadata redesign unrelated to static domains |
| Vector cardinality proof | General dependent arithmetic constraints |
| Exact Vector spread execution | Wholesale Iterable protocol redesign |
| Vector immutability and typed API | Full immutable-collection trait hierarchy |
| Checked Vector lowering | General compiler architecture refactoring |
| Vector structural hash | Global deep-hashability policy redesign |
| Vector GC correctness | Unrelated collector optimization |
| P1/P2 regression coverage | Full C1 certification, which belongs to P9 |

### Final verdict

**COLL006.C1.P3 is implementable within a single bounded implementation plan, provided that the plan explicitly includes the shared static-generic repairs and exact cardinality guarantees as entry gates.**

The repository is sufficiently advanced to support this approach. P1 and P2 established useful architecture that should be preserved, not replaced. But P3 must not be considered a routine immutable-collection extension: it is the first major consumer of Telos's heterogeneous static generic parameters in ordinary collection construction.

The correct completion criterion is not merely that:

```telos
let x = <Int, 3>[1, 2, 3]
```

compiles and runs.

It is that the compiler can establish, preserve, and execute the proposition:

\[
x : \operatorname{Vector}(\operatorname{Int},3)
\quad\land\quad
|\operatorname{elements}(x)|=3
\]

**without a parallel type-identity mechanism, an unproved cardinality assumption, an intermediate collection, or a mutable escape path.**

That is the architectural standard against which P3 should be implemented and audited.