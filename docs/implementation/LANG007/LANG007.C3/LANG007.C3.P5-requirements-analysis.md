---
id: LANG007.C3.P5
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: requirements-analysis
title: "LANG007.C3.P5 — Type-System Coherence, Structural Semantics, Dynamic Applications, and Iterator/Conformance Closure"
status: PROPOSED
completion: ANALYSIS_COMPLETE
verification: SOURCE_AUDITED
prepared: 2026-10-01
repository: aureat/phalcom-lang
prepared_against_revision: 6dcdcec08e55c2eb67eb60fff959cfb71fc3e939
depends_on:
  - LANG007.C3.P4
  - LANG007.C3.P4.R1
supersedes: null
---

# LANG007.C3.P5 — Requirements Analysis

## Type-System Coherence, Structural Semantics, Dynamic Applications, and Iterator/Conformance Closure

> **Purpose.** This document is the requirements-analysis handoff for the next
> LANG007.C3 plan. It is not an implementation plan. It records the current
> repository facts, specification conflicts, causal hypotheses, architecture
> constraints, implementation obligations, acceptance gates, and unresolved
> decisions that the P5 planner must resolve before producing an executable
> patch-grade plan.

---

## 0. Executive disposition

P5 must not be framed as “finish the remaining iterator tests” or “fix the
remaining P4 failures.” The current blockers expose four deeper seams that
intersect but have different semantic owners:

1. **Structural Tuple/Record semantics are inconsistent at the specification
   level.** `docs/specs/README.md` assigns structural Tuple/Record typing to
   `docs/specs/structural-products.md`, while
   `docs/specs/tuple-and-record-typing.md` also specifies closed-by-default,
   explicit-open, optional-member, and typed/unconstrained remainder semantics.
   The live relation still implements classic immutable Record width subtyping.
   P5 must resolve the specification defect before changing the relation.

2. **The type-formation model cannot presently express a known generic head
   whose argument is explicitly Dynamic.** `List<Dynamic>` collapses to a
   whole-form `TypeFormResolution::Dynamic` before an applied type can be
   formed. `TypeData::Applied`, metadata `TypeNode::Applied`, and runtime
   applied-type arguments all require canonical static terms or record rows;
   none can carry a Dynamic argument slot. This is upstream of the current
   `RangeIterator::toList` inferred-result blocker.

3. **Iterator bootstrap mixes compiler specialization asymmetry with genuinely
   underconstrained Universe source.** One witness-specialization path
   materializes declared parameter/result terms but does not visibly
   materialize the canonical inferred return. Independently, multiple iterator
   adapters use `<I as Iterator>::Item` or call `I.next` without declaring
   `I |= Iterator`, and several result-generic adapters provide no callable
   contract from which `U` can be inferred. The former is a high-confidence
   causal hypothesis requiring a focused proof; the latter is a verified
   source-contract defect.

4. **Rest/module-export conformance transport is not yet localized to runtime.**
   Existing probes do not produce a callable or call-specialization recipe, so
   they stop before evidence transport can be tested. P5 must establish
   semantic specialization first and only then test forwarding.

P5 should therefore be a **coherence and closure plan**: settle the language
model, make the representations match it, restore generic/conformance
correctness, and then certify natural Iterator/Iterable behavior and the
remaining evidence-transport seam.

The architectural direction recommended by this analysis is:

```text
Structural values:
    closed means exact
    openness is explicit
    optionality is a presence property
    captured rows and anonymous remainders are distinct

Dynamic generic applications:
    preserve known outer structure where source explicitly establishes it
    without turning Dynamic into a proper TypeId

Generic conformance:
    specialize the complete canonical callable contract at the use site
    and require source declarations to state every projection/conformance
    obligation they semantically depend on
```

The structural and nested-Dynamic recommendations still require explicit P5
architectural rulings where the current specification is contradictory or
silent.

---

## 1. Audit basis and evidence discipline

### 1.1 Repository baseline

This analysis was prepared against:

```text
repository: aureat/phalcom-lang
revision:   6dcdcec08e55c2eb67eb60fff959cfb71fc3e939
```

The current LANG007.C3 checkpoint remains:

```text
status:       IN_PROGRESS
completion:   PARTIAL
verification: UNVERIFIED
active plan:  LANG007.C3.P4.R1
```

P4.R1 has completed/focused-tested T1 through T5. T6/G5 is blocked and T7 has
not been reached. P4 and C3 are not closed.

### 1.2 Evidence classes

| Tag | Meaning | Planning consequence |
|---|---|---|
| **VERIFIED** | Directly observed in current source/spec/checkpoint at the pinned revision | May be treated as plan input |
| **SPEC-DEFECT** | Current specification sources conflict or semantic ownership is ambiguous | Resolve before dependent implementation |
| **CAUSAL-HYPOTHESIS** | Source inspection strongly suggests a cause but this audit did not execute the fix/proof | Reproduce/prove before production edit |
| **ARCH-RECOMMENDATION** | Coherent design proposed by this analysis but not yet a ratified language rule | Requires explicit adoption |
| **OPEN-DECISION** | Representation/semantics remain unsettled | Must become an early task/stop gate |

The P5 planner must preserve these distinctions.

---

## 2. Fixed predecessor invariants

P5 inherits accepted P4.R1 architecture. These are not reopened by default.

### P5-FIXED-001 — One canonical callable-result authority

`CallableSemanticSignature` remains the canonical callable contract. Ordinary
body-bearing callables and body-bearing trait defaults publish inferred results
through that same authority.

P5 must not add:

- a trait-default inferred-result table;
- a dispatch-local result cache;
- a consumer-owned result state;
- a second callable signature authority;
- runtime result inference.

### P5-FIXED-002 — Provider-owned inference

The provider body is analyzed in its canonical declaration environment.
Caller expected types may constrain a call site but must not define or mutate
the provider's canonical result.

### P5-FIXED-003 — Symbolic canonical results remain legal

`Self`, declaration generic parameters, and associated projections may remain
symbolic in the canonical result contract and are specialized through existing
use-site materialization/projection machinery.

### P5-FIXED-004 — No general result fixpoint

P4.R1 rejected a general recursive inferred-result fixpoint.

Required behavior remains:

```text
acyclic result dependency
    -> ordinary publication

cycle with sufficient explicit result seed
    -> may resolve through existing contracts

unseeded inferred-result SCC
    -> deterministic cycle disposition/diagnostic

stale inferred cycle after edit
    -> bounded one-shot barrier and one reanalysis
       not iterative result widening
```

### P5-FIXED-005 — Fingerprints remain semantic

A changed inferred result changes the callable interface fingerprint and
invalidates dependent consumers. A body edit preserving the inferred result
does not change the result-interface fingerprint. The same principle applies
to explicit-result bodies.

Relevant predecessor selectors include:

```text
semantic::incremental::callable_dependencies::
  inferred_return_refresh_preserves_stable_products_and_counts_final_disposition_once

semantic::incremental::product_stability::
  inferred_cross_module_result_updates_dependents_and_matches_cold_analysis

semantic::integration::metadata::
  cross_module_inferred_callable_result_round_trips_as_compiler_inferred_metadata
```

### P5-FIXED-006 — Conformance evidence is selected semantically and projected

Compiler/core lowering projects already-selected Exact/Derived evidence.
Runtime must not rediscover conformance from `ImplId`, selector spelling, or
source-rule scans.

---

## 3. Finding A — structural Tuple/Record semantics are not singular

**Classification: SPEC-DEFECT / VERIFIED.**

### 3.1 Current authority map

`docs/specs/README.md` declares `docs/specs/` authoritative, states that
semantic ownership determines precedence, and treats conflicts between current
specifications as defects requiring explicit reconciliation.

Its ownership map assigns:

```text
Structural Tuple/Record typing and closed structural type operations
    -> docs/specs/structural-products.md
```

### 3.2 `structural-products.md`

The current file is marked:

```text
Status: Normative
Specification role:
    Authoritative specification for structural Tuple/Record formation
    and closed type-level operations
```

It defines:

- `TypePack`, `LabeledTypePack`, and `RecordFields` as distinct domains;
- Tuple shape as positional plus labeled lanes;
- Record as `Record(R)` where `R : RecordFields`;
- closed and open rows;
- row-tail-preserving mapping;
- `Unit = Tuple<>`;
- `EmptyRecord = Record<>`;
- closed structural operations such as `At` and `Map`.

### 3.3 `tuple-and-record-typing.md`

This current document specifies a richer model:

```text
closed by default
explicit remainders open a lane/record
tuple positional and labeled lanes open independently
typed remainders
unconstrained remainders
bare ... as structural remainder
required vs optional labeled members
optional presence distinct from Option<T>
no optional positional members
tuple/record structural subtyping based on open/closed shape
```

Representative distinction:

```phalcom
{ name?: String }
```

is not the same contract as:

```phalcom
{ name: Option<String> }
```

### 3.4 Live relation still reflects older width subtyping

`phalcom-semantic/src/types/relation.rs` dispatches Record/Record relation to
`check_record_row_subtype`, whose shape is target-required-field based:

```text
for every field required by the supertype row
    find that field in the subtype row
    compare field types
```

This is compatible with traditional immutable Record width subtyping.

Historical implementation plans also describe Records as “width + covariant
depth,” explaining the current code, but historical plans do not override the
current spec corpus.

### P5-REQ-010 — Single normative structural owner

P5 must leave one unambiguous normative owner for:

- closedness;
- structural openness;
- optional presence;
- positional/labeled lane openness;
- homogeneous remainder constraints;
- unconstrained remainder semantics;
- structural subtype laws.

### P5-REQ-011 — Relation changes follow the ruling

No Record/Tuple subtype implementation change should precede the structural
semantic ruling.

**Architectural recommendation:**

```phalcom
{ a: Int, b: String } </: { a: Int }

{ a: Int, b: String } <: { a: Int, ... }
```

That is: closed means exact; width acceptance requires explicit openness.

---

## 4. Finding B — current AST supports captured rows but not the full structural model

**Classification: VERIFIED.**

A prior shorthand description that “the AST cannot express open rows” is too
broad.

### 4.1 Already supported

Current AST/semantic paths support a named/captured Record tail:

```phalcom
{ id: Int, **R }
```

`TypeAnnotationExpr::Record` carries fields and a tail, and semantic lowering
already understands that form.

Tuple type syntax also has:

```text
TypeTupleItem::Element(...)
TypeTupleItem::Expansion(TypePackExpansionSyntax)
```

for expansion of a specific pack operand.

### 4.2 Missing representation

Current `RecordTypeField` and canonical `RecordRowField` carry a name and type
but no first-class presence mode.

Current row tail is:

```text
Closed
Parameter(TypeParameterId)
Projection(TypeId)
```

which models closed rows and correlated symbolic tails, not anonymous
admissible remainders.

Tuple `Expansion` means “expand this particular pack/type form,” not
“arbitrarily many additional elements constrained by T.”

### 4.3 Captured row versus anonymous remainder

These are semantically distinct.

Captured:

```phalcom
{ id: Int, **R }
```

means one correlated row entity `R` participates in inference, substitution,
lacks constraints, projection, and reassembly.

Anonymous typed remainder:

```phalcom
{ id: Int, **: JsonValue }
```

admits unknown additional labels constrained by a value type. No reusable row
identity is captured.

Anonymous unconstrained remainder:

```phalcom
{ id: Int, ... }
```

admits extra fields without asserting a common value type.

### P5-REQ-012 — First-class member presence

Introduce a first-class labeled-member presence axis:

```text
Required
Optional
```

and preserve it through:

- parser/AST;
- annotation lowering;
- canonical structural representation;
- normalization;
- relation;
- mapping/projection where applicable;
- metadata if durable;
- source index/presentation/LSP.

Do not encode optional presence as `Option<T>`, `T | Unit`, or Dynamic.

### P5-REQ-013 — First-class anonymous remainder constraints

Represent at least:

```text
closed
unconstrained anonymous remainder
typed anonymous remainder C
captured/correlated row tail R
projection-derived row tail
```

without collapsing these states.

### P5-REQ-014 — Tuple lane independence

The final representation must support independent positional and labeled lane
openness and, if retained by the spec, the complete-lane shorthand.

### P5-REQ-015 — Ellipsis remains context-sensitive, not a fabricated pack

Structural `...` must not be implemented by pretending the Ellipsis singleton
value is a `TypePack`, `RecordFields`, `Any`, or Dynamic.

---

## 5. Finding C — current Record-row representation is too narrow for optional/anonymous openness

**Classification: VERIFIED + OPEN-DECISION.**

Current canonical row data is:

```rust
RecordRowData {
    fields: Box<[RecordRowField]>,
    tail: RecordRowTail,
}
```

with sorted unique fields and tail:

```text
Closed
Parameter
Projection
```

It does not encode optional presence, homogeneous anonymous remainder type, or
unconstrained anonymous remainder.

### P5-OPEN-001 — Record shape representation

P5 must deliberately choose whether to:

1. enrich `RecordRowData` with presence and anonymous remainder cases; or
2. introduce a higher structural-shape representation while keeping the row
   solver focused on correlated rows.

The implementation plan must state which identities are canonical, what
canonical equivalence means, and how `Record::Map`/projection preserve shape.

---

## 6. Finding D — packs and rows are not ordinary subtype-lattice members

**Classification: VERIFIED.**

`docs/specs/generic-constraints.md` defines subtype constraints over
`ProperTypeExpression` and distinguishes `Type`, `TypePack`,
`LabeledTypePack`, `CompleteTypePack`, `RecordFields`, and static domains.

`phalcom-semantic/src/types/relation.rs::check_subtype_impl` correctly gates
ordinary subtyping:

```text
identity may succeed first

otherwise:
    if kind(sub) != Type or kind(sup) != Type
        refute ordinary subtype relation
```

The same implementation nevertheless contains pointwise match arms for
`TypePack`, `LabeledTypePack`, and `CompleteTypePack`; for unequal non-Type
terms these are behind the proper-Type gate and are misleading.

### P5-REQ-020 — Preserve the proper-Type boundary

Do not weaken `ProperTypeId`/`KindId::TYPE` semantics to make pack machinery
convenient.

### P5-REQ-021 — Use domain-native operations

Packs, rows, and static terms use domain-appropriate:

```text
canonical equality
normalization
concatenation/reassembly
unification
projection
domain-specific inference
```

not ordinary `<:`.

### P5-REQ-022 — Remove or quarantine misleading pack subtype arms

Either delete unreachable ordinary-subtype pack arms or move the intended logic
behind an explicitly kinded domain relation.

---

## 7. Finding E — `List<Dynamic>` becomes whole-form Dynamic before application

**Classification: VERIFIED.**

`INC-C3-27.md` records the current T6 blocker.

In `phalcom-semantic/src/types/annotation.rs`, generic application resolution
currently does, in effect:

```text
resolve origin
resolve every argument
Ready(ty) -> collect
Dynamic   -> return whole-form Dynamic immediately
other terminal state -> propagate
only then call apply_type_form(...)
```

Therefore:

```phalcom
List<Dynamic>
```

never reaches `store.apply_type_form(List, ...)`.

`resolve_type_annotation` then maps the form to:

```text
TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape)
```

### 7.1 Direct consequence for `RangeIterator::toList`

The current direct probe shows:

```text
List<Dynamic>.new()
    -> Dynamic(ExplicitEscape)
    -> no constructor callable selected
    -> result binding Unknown(NoTypeEvidence)
    -> append unresolved/Unknown
    -> terminal body result Unknown
    -> canonical inferred_return absent
```

The first failure is type formation, not P4.R1 result publication.

### 7.2 Existing type identity cannot simply absorb Dynamic

`TypeData::Applied` is:

```text
origin: TypeId
arguments: StaticArgument[]
```

and `StaticArgument` is currently:

```text
Term(TypeId)
RecordRow(RecordRowId)
```

Metadata `StaticArgumentRef` and runtime `RuntimeStaticArgument` likewise only
carry canonical term or row categories.

Dynamic currently lives in `TypeKnowledge`, not `TypeData`.

---

## 8. Required semantic ruling for nested Dynamic

**Classification: OPEN-DECISION + ARCH-RECOMMENDATION.**

P5 must answer:

> Does explicit `List<Dynamic>` statically preserve the fact that the outer
> constructor is `List`, even though the element slot is behind a Dynamic
> boundary?

**Recommendation: yes.**

This does not mean Dynamic becomes a proper type. It means type-form structure
can be partially known.

### Required distinctions

The compiler must not collapse:

```text
Dynamic
List<Dynamic>
List<Any>
List<Int>
List<T> where T is canonical
List<X> where X is unresolved/Unknown
future existential/erased forms
```

### P5-REQ-030 — Nested Dynamic semantics must be explicit

Make a normative decision before editing generic type application.

### P5-REQ-031 — Preserve Dynamic/proper-type distinction

Under the recommended model, do not add an ordinary Dynamic `TypeId` solely to
fit `List<Dynamic>` into the current `Applied` node.

### P5-REQ-032 — Add a partial type-form/application product if adopted

A conceptual representation is:

```rust
enum TypeFormKnowledge {
    Known(TypeId),
    PartiallyKnownApplication {
        origin: TypeId,
        arguments: Box<[TypeFormArgumentKnowledge]>,
    },
    Dynamic(DynamicReason),
    Unknown(UnknownReason),
    // other terminal states
}
```

The exact Rust API is open. The semantic requirement is that a partially known
application is neither a fabricated `Known(TypeId)` nor whole-form Dynamic.

### P5-OPEN-002 — Ownership of partial application knowledge

P5 must decide whether the canonical owner is:

- `TypeFormResolution`;
- `TypeKnowledge`;
- a new formal type-shape product;
- a generalized static-argument envelope;
- another explicitly owned layer.

Downstream consumers must consume a canonical product rather than re-deriving
the distinction from source spelling.


---

## 9. Cross-layer obligations for partially-known applications

A local change in `resolve_type_form` is insufficient.

If P5 adopts known-head/dynamic-slot semantics, the following consumers must be
audited explicitly.

### P5-REQ-033 — Constructor/call specialization

For:

```phalcom
List<Dynamic>.new()
```

the compiler must be able to select the `List` constructor contract while
preserving the Dynamic obligation of the element slot.

Existing probes already establish an important generic rule:

```text
dependent result:
    T -> T with a Dynamic premise
    => Dynamic result

fixed result:
    generic premise Dynamic, result Int
    => Known Int result
```

P5 must preserve this distinction.

### P5-REQ-034 — Member lookup and associated projections

A known outer head may support head-owned member discovery where the member is
independent of the exact dynamic slot. A projection that semantically depends
on the dynamic slot must not fabricate a canonical static result.

P5 must define behavior for:

```text
head-only member
argument-dependent member
associated projection depending on dynamic argument
variance-dependent compatibility
```

### P5-REQ-035 — Substitution and materialization

Dynamic argument information must not disappear during:

- `TypeView` materialization;
- `TypeSubstitution`;
- receiver specialization;
- witness specialization;
- callable result specialization;
- generic inference;
- projection normalization.

### P5-REQ-036 — Assignability and conformance

P5 must explicitly define what can be proven for a partially-known application.

At minimum:

```text
List<Dynamic> == List<Any>    must not be fabricated
List<Dynamic> <: List<Any>    must not be assumed from Dynamic alone
List<Int> relation            follows declared generic variance
Dynamic obligations           remain visible when proof depends on the slot
head identity                 may remain usable for head-owned lookup
```

### P5-REQ-037 — Presentation

Diagnostics/tooling must distinguish:

```text
Dynamic
List<Dynamic>
List<Any>
Unknown/unresolved application
```

without collapsing them into the same formal state.

---

## 10. Metadata and runtime consequences

**Classification: VERIFIED representation gap + OPEN-DECISION.**

P4.R1 prohibited schema changes without evidence. P5 now has evidence that the
existing exact static-argument schema cannot represent a durable
known-head/dynamic-slot application.

That does not automatically authorize a schema change. P5 must decide whether
the partially-known application is:

```text
transient analysis knowledge
```

or:

```text
a durable contract/type-form crossing module/runtime boundaries
```

### P5-OPEN-003 — Durability boundary

Decide whether `List<Dynamic>` must survive in:

- exported callable signatures;
- imported metadata;
- incremental products/fingerprints;
- reflected runtime descriptors;
- runtime type recipes/environments.

### P5-REQ-038 — No lossy persistence

If durable, do not serialize the form as bare `List`, `List<Any>`, whole-form
Dynamic, or Unknown unless the language explicitly defines that as canonical
equivalence.

### P5-REQ-039 — Exact runtime type and Dynamic boundary remain separate

`docs/specs/objects/data-objects.md` requires preservation of exact applied
runtime type where exact type is semantically established.

P5 must distinguish:

```text
already-established exact applied type crossing a Dynamic boundary
```

from:

```text
source explicitly using Dynamic as a generic argument during static type
formation
```

The former does not by itself settle the latter.

---

## 11. Finding F — Universe collection APIs contain historical Dynamic erasure

**Classification: VERIFIED.**

Current `phalcom-core/core/universe/src/collections/iterator_laws.ph` uses
Dynamic containers in several terminal operations:

```text
group         -> Map<Dynamic, Dynamic> + List<Dynamic>
partition     -> List<Dynamic>, List<Dynamic>
toSet         -> Set<Dynamic>
toMap         -> Map<Dynamic, Dynamic>
toMap merging -> Map<Dynamic, Dynamic>
toList        -> List<Dynamic>
```

### P5-REQ-050 — Audit intentional versus historical Dynamic

Each affected use must be classified:

```text
INTENTIONAL_DYNAMIC_BOUNDARY
HISTORICAL_ERASURE
DEPENDENT_ON_UNRESOLVED_API_DESIGN
```

### P5-REQ-051 — Prefer existing associated contracts where sufficient

Where `Self::Item` already supplies the exact element contract, the library
should not erase it without reason.

Conceptual targets include:

```text
Iterable::toList
    -> List<Self::Item>

partition
    -> (List<Self::Item>, List<Self::Item>)

toSet
    -> Set<Self::Item>
```

subject to existing Set constraints and source syntax.

For `RangeIterator::toList`, the result should derive from the canonical
Iterator Item contract, not from a compiler special case.

### P5-REQ-052 — Library cleanup cannot evade the language question

Even if every internal `List<Dynamic>` use were made more precise, user code
may still write:

```phalcom
List<Dynamic>
```

P5 must define it.

### P5-OPEN-004 — `group`/`toMap` precision

`group` and `toMap` require key/value relationships not derivable from
`Self::Item` alone unless additional callable/entry contracts are stated.

P5 should not turn this into a full collection API redesign. It should either
express the precise contract using existing language machinery or preserve a
documented intentional Dynamic boundary.

---

## 12. Finding G — witness specialization is asymmetric around inferred return

**Classification: CAUSAL-HYPOTHESIS with strong source evidence.**

### 12.1 Requirement-side instantiation

`phalcom-semantic/src/traits.rs::TraitSurfaceMember::instantiate` specializes:

- parameter declared types;
- declared return;
- canonical `inferred_return`.

The inferred result is materialized through `TypeView` in the same environment.

### 12.2 Candidate-side specialization

The current
`phalcom-semantic/src/impls.rs::specialize_witness_signature` source path
materializes:

- parameter declared types;
- declared return;
- generic constraint terms.

The audited implementation does not visibly materialize
`signature.inferred_return`.

### 12.3 Compatibility consumes the inferred return

`witness_signature_return_type` explicitly uses `inferred_return` when the
declared return is unknown. `check_witness_compatibility_with_visibility` then
compares the candidate result covariantly to the instantiated requirement.

### 12.4 Causal hypothesis

A generic inherent unannotated witness such as:

```phalcom
iterator { self }
```

can therefore be compared in two different environments:

```text
requirement result
    -> instantiated to exact receiver

candidate inferred result
    -> potentially still owner-relative
```

This is a high-confidence explanation for at least part of the current
`ImplConformanceIncomplete` bootstrap failure, but no production edit was made
during this audit.

### P5-REQ-060 — Focused proof before fix

Before changing production code, reproduce with:

```text
generic receiver
unannotated inherent witness
canonical inferred result containing Self and/or owner parameter
trait requirement instantiated to exact receiver
```

The test must expose the current candidate/requirement result views.

### P5-REQ-061 — Specialize the complete canonical contract

If confirmed, specialize canonical `inferred_return` in the witness view under
the same environment as the rest of the callable contract.

This is use-site materialization, not re-inference and not a second result
owner.

### P5-REQ-062 — Keep witness checking strict

Required negative controls:

- genuinely incompatible inferred result;
- incompatible declared result;
- strengthened generic constraints;
- wrong rest role;
- wrong dispatch side;
- unresolved/dynamic result where a static proof is required.

---

## 13. Finding H — multiple Iterator adapters are ill-formed under their own projections

**Classification: VERIFIED source-contract defect.**

`docs/specs/generic-constraints.md` requires generic code to be checked under
declared constraints. Qualified associated projection consumes conformance
evidence; it does not create it.

Current `iterator_laws.ph` includes adapters that call `_upstream.next` and/or
project `<I as Iterator>::Item` without consistently declaring
`I |= Iterator`.

Examples include:

```text
FilterIterator<I>
SkipIterator<I>
SkipWhileIterator<I>
TakeIterator<I>
TakeWhileIterator<I>
InspectIterator<I>
```

and result-generic adapters whose bodies also depend on Iterator behavior.

By contrast, `FlatMapIterator`, `FlattenIterator`, `ChainIterator`,
`StrictZipIterator`, and `IndexedIterator` already carry meaningful bounds.

### P5-REQ-070 — Every associated projection must be justified

Add missing source constraints at the earliest semantically appropriate owner:

- class;
- constructor;
- impl;
- public factory/default method;
- or the minimum combination required by current scoping.

Avoid redundant duplication where an enclosing contract already proves the
obligation.

### P5-REQ-071 — Projection syntax is not proof syntax

Do not interpret:

```phalcom
<I as Iterator>::Item
```

as implicit:

```phalcom
where I |= Iterator
```

### P5-REQ-072 — Body behavior and generic domain must agree

If an adapter body invokes `I.next`, its generic contract must make that
operation valid even when the associated Item projection is absent from the
same declaration.

---

## 14. Finding I — mapper result generic `U` is underconstrained

**Classification: VERIFIED source shape + CAUSAL-HYPOTHESIS for exact diagnostic count.**

Current defaults include:

```phalcom
map(_ f)       { MappedIterator.new(self, f) }
filterMap(_ f) { FilterMapIterator.new(self, f) }
mapWhile(_ f)  { MapWhileIterator.new(self, f) }
```

while the constructors accept an untyped mapper and the class introduces `U`.
Nothing in those constructor contracts establishes `U`.

This is a direct inference hole and is a plausible cause of some or all of the
current three `type.generic.underconstrained` bootstrap diagnostics.

### P5-REQ-080 — Make `U` inferable from callable return

Conceptually:

```phalcom
map<U>(_ f: (Self::Item) -> U) -> MappedIterator<Self, U>
```

and:

```phalcom
MappedIterator<I, U>.new(
    _ upstream: I,
    _ mapper: (<I as Iterator>::Item) -> U
)
    where I |= Iterator
```

must provide a real inference path.

`filterMap` and `mapWhile` should express the appropriate `Option<U>`-shaped
mapper result if that is their intended API contract.

### P5-REQ-081 — Type predicate/callback adapters where semantics are known

Audit:

```text
filter
skipWhile
takeWhile
inspect
```

and provide typed callable contracts where this follows from their intended API.

### P5-REQ-082 — No generic defaulting

Do not repair underconstrained `U` by assigning:

```text
Dynamic
Any
Self::Item
first observed runtime value
```

unless a separate language rule explicitly defines that behavior.

---

## 15. Existing Iterator corrections that must be preserved

**Classification: VERIFIED predecessor work.**

P5 is not a redesign of already-corrected adapter semantics.

### Chain

Current source carries:

```text
I |= Iterator
J |= Iterator
<J as Iterator>::Item == <I as Iterator>::Item
Item = <I as Iterator>::Item
```

### FlatMap

The mapper consumes outer Item and returns an Iterable source; resulting Item
comes from the nested Iterable.

### Flatten

Outer Item must conform to Iterable and resulting Item projects through that
nested Iterable.

### StrictZip

```text
Item = (
    <I as Iterator>::Item,
    <J as Iterator>::Item
)
```

### Indexed

```text
Item = (Int, <I as Iterator>::Item)
```

Earlier focused evidence:

```text
iterator_adapter_fixture_preserves_item_projections_and_chain_equality
```

must remain green.

### P5-REQ-090 — Preserve accepted adapter contracts

P5 may add missing constraints/callable contracts around these types but must
not regress accepted Item/projection/equality semantics.

### P5-REQ-091 — Preserve the `toMap` iteration correction

The previously corrected `toMap` behavior must remain single-advance per
logical element. Add a post-bootstrap regression because Iterator default
changes could accidentally reintroduce the prior double-advance behavior.

---

## 16. Natural Range chain as primary end-to-end oracle

The highest-value integration program remains:

```phalcom
(0..).iter.take(10).toList
```

Current verified stages:

```text
(0..).iter
    -> Known RangeIterator
    -> Exact Range -> Iterable::iter evidence

.take(10)
    -> inherent RangeIterator::take
    -> Known TakeIterator<RangeIterator>

.toList
    -> no selected callable
    -> TakeIterator<RangeIterator> Iterator conformance incomplete
```

### P5-REQ-100 — Restore the complete chain without special cases

Expected successful path:

```text
Range
  -> RangeIterator

RangeIterator.take
  -> TakeIterator<RangeIterator>

TakeIterator<RangeIterator>
  -> exact Iterator conformance

Iterator
  -> guarded derived Iterable conformance

Iterable::toList default
  -> selected canonical callable

toList body/result
  -> canonical result consistent with adopted Dynamic/collection semantics
```

No stage may be hardcoded for Range, RangeIterator, TakeIterator, or `toList`.

### P5-REQ-101 — Assert semantic products, not just output

Where products are exposed, verify:

- receiver type at each stage;
- selected callable;
- Exact versus Derived evidence;
- nested proof identity for derived Iterable;
- specialized result contract;
- lowering into executable trait invocation;
- runtime result.

---

## 17. Universe bootstrap diagnostics are a hard gate

The checkpoint records:

```text
20 ImplConformanceIncomplete
3  type.generic.underconstrained
```

These stop current T6-B tests before their focused assertions.

### P5-REQ-110 — Do not bypass the precondition

Do not:

- ignore the diagnostics;
- skip Universe bootstrap in focused core tests;
- downgrade conformance errors;
- install fake evidence;
- use an artificially reduced Universe as final acceptance evidence.

### P5-REQ-111 — Maintain a diagnostic attribution ledger

For each bootstrap diagnostic record:

```text
source declaration/impl
requirement or generic parameter
diagnostic code
causal category:
  witness specialization
  missing source bound
  underconstrained callable result
  other
```

The gate passes only after each prior diagnostic is eliminated or explicitly
reclassified with evidence.

---

## 18. Finding J — rest/module-export evidence transport is not yet localized to runtime

**Classification: VERIFIED.**

The P4 audit remediation established correct nested derived-evidence slot
projection.

However, the remaining probe:

```text
p4_specialized_rest_dispatch_preserves_conformance_environment
```

currently reaches an unknown/Dynamic send with:

```text
callable = None
call_specialization = None
```

A bounded caller-to-rest-target probe likewise has no specialization recipe,
and the module-export probe has an empty recipe map.

Therefore there is no evidence recipe to forward yet.

### P5-REQ-120 — Semantic reachability before runtime transport

First establish:

```text
resolved callable
call specialization
non-empty conformance evidence recipe
```

Only then test rest/module-export forwarding.

### P5-REQ-121 — No speculative VM fix

A runtime patch is authorized only if a focused test proves:

```text
semantic specialization contains evidence
lowered recipe contains evidence
runtime forwarding loses/misroutes evidence
```

### P5-REQ-122 — Runtime remains proof-free

Any transport fix must carry/remap already-selected evidence. The VM must not
perform trait search or reconstruct conformance from source declarations.

---

## 19. Priority-3 residual ownership rule

Existing classification contains:

```text
30 CURRENT REGRESSION
4  TRUE CLEAN-PREDECESSOR BASELINE
2  INTENTIONAL P4 MIGRATION
4  FIXTURE INTERFERENCE — corrected
```

### P5-REQ-130 — Causal ownership only

P5 owns a residual only when focused tracing ties it to:

- structural exact/open/optional semantics;
- structural remainder representation;
- partial Dynamic generic application;
- canonical inferred-result specialization;
- generic source-contract inference;
- conformance/evidence specialization;
- P5 metadata/fingerprint/tooling changes;
- rest/module-export evidence transport.

Everything else remains separately classified unless C3 closure requires it.

### P5-REQ-131 — Broad red suites do not define broad scope

Workspace/aggregate failures are acceptance signals, not automatic P5
requirements.

---

## 20. Structural semantic laws required by P5

Assuming the recommended exact-closed model is ratified, direct law tests must
cover:

### Record

```phalcom
{ a: A, b: B } </: { a: A }
{ a: A, b: B } <:  { a: A, ... }
```

Also:

- open source cannot prove a closed exact target if extra fields may exist;
- typed target remainder accepts an extra field only when the field type
  satisfies the remainder constraint;
- optional field may be absent;
- optional field, if present, must satisfy its declared type;
- optional presence differs from required `Option<T>`;
- unconstrained source remainder cannot prove a typed optional field when it
  might contain that label with incompatible type;
- a typed source remainder may prove it only when the final ratified relation
  permits that type.

### Tuple

Test independently:

- fixed positional prefix;
- fixed labeled members;
- positional openness;
- labeled openness;
- optional labeled presence;
- typed/unconstrained remainder constraints;
- source-open versus target-closed negative cases.

If `***: T` is retained as shorthand for opening both lanes with the same
constraint, normalization must establish that equivalence.

### Canonicalization

Directly test:

```text
equivalent syntax -> same canonical structural meaning
captured **R != anonymous remainder
typed remainder T != Any
... != Dynamic
... != a pack value
optional presence survives normalization
Record key order obeys canonical Record identity
Tuple labeled order obeys the owning spec
```

### P5-REQ-140 — Property/law tests are mandatory

Cover representative reflexivity, transitivity, exact/open monotonicity, lane
independence, presence, canonicalization, row correlation, and negative cases.
Corpus examples alone are insufficient.

---

## 21. Required representation matrix

The P5 implementation plan must fill and close a matrix equivalent to this:

| Semantic concept | Source syntax | AST | Canonical semantic product | Metadata | Runtime/reflection | Relation |
|---|---|---|---|---|---|---|
| Closed Record | `{a:A}` | existing | row/shape | existing/adapted | existing | exact if ratified |
| Captured row | `{a:A, **R}` | existing | row tail parameter/projection | existing open-row machinery | current path | correlated-row rules |
| Optional field | `{a?:A}` | missing | **new presence representation** | if durable | presentation decision | presence-aware |
| Typed anonymous remainder | `{a:A, **:T}` | missing | **new remainder representation** | if durable | likely static/tooling | remainder-aware |
| Unconstrained remainder | `{a:A, ...}` | missing | **new remainder representation** | if durable | likely static/tooling | open |
| Tuple positional remainder | `(*:T)` | missing as anonymous remainder | **new** | if durable | tooling | lane-aware |
| Tuple labeled remainder | `(**:T)` | missing as anonymous remainder | **new** | if durable | tooling | lane-aware |
| Known applied type | `List<Int>` | existing | `TypeData::Applied` | existing | existing | ordinary variance |
| Bare Dynamic | `Dynamic` | existing | `TypeKnowledge::Dynamic` | existing slot state | dynamic boundary | not proper subtype term |
| Partial dynamic application | `List<Dynamic>` | currently collapses | **open decision** | **open decision** | **open decision** | boundary-aware |

A P5 plan that cannot fill this matrix is not implementation-ready.


---

## 22. Required implementation work packages

This section defines dependency-oriented work packages. It is not yet a
patch-grade implementation sequence; the P5 planner may split/merge work only
if it preserves the dependency graph and stop gates.

### T1 — Baseline and lifecycle pin

Required outputs:

- pinned starting SHA;
- current checkpoint/P4.R1 state;
- exact active blockers;
- current Universe diagnostic ledger;
- exact focused selector ledger;
- explicit statement that P4/C3 is not complete.

Exit:

```text
baseline is reproducible
predecessor invariants are enumerated
no stale handoff state is treated as current doctrine
```

### T2 — Structural specification reconciliation

Required work:

- reconcile `structural-products.md` and `tuple-and-record-typing.md`;
- ratify closed/exact versus width semantics;
- ratify optional member rules;
- ratify typed/unconstrained anonymous remainder rules;
- ratify Tuple lane openness;
- update authority map/cross-references.

Exit:

```text
one normative structural semantic model
```

No structural relation implementation should precede this gate.

### T3 — Structural source and canonical representation

Required work:

- parser/AST support for optional presence;
- parser/AST support for anonymous remainders;
- preserve existing captured row syntax;
- canonical field presence;
- canonical anonymous remainder constraints;
- normalization/canonicalization;
- source index/presentation identities.

Exit:

```text
every ratified structural syntax has an unambiguous semantic product
```

### T4 — Nested Dynamic architectural ruling and representation

Required work:

- formal semantics of `List<Dynamic>`;
- distinguish bare Dynamic and partial application;
- choose representation owner;
- choose durability boundary;
- direct formation controls.

Exit:

```text
downstream consumers no longer need to infer semantics from source spelling
```

### T5 — Structural relation and normalization

Depends on T2/T3.

Required work:

- exact/open relation;
- optional-presence relation;
- typed/unconstrained remainder relation;
- Tuple lane relation;
- mapping/projection preservation;
- remove/quarantine ordinary pack subtype dead paths;
- direct property/law suite.

Exit:

```text
structural laws are green
```

### T6 — Partial-Dynamic consumer integration

Depends on T4.

Audit/update:

- constructor resolution;
- generic callable application;
- member lookup;
- result specialization;
- substitution/materialization;
- projection normalization;
- assignability/variance;
- conformance interaction;
- diagnostics;
- metadata/runtime if durable.

Exit:

```text
Dynamic / List<Dynamic> / List<Any> / List<Int> controls remain distinct
dependent-versus-fixed generic result behavior is correct
```

### T7 — Generic witness and Iterator source-contract correctness

Subtasks:

1. reproduce inferred-witness specialization hypothesis;
2. if confirmed, materialize canonical inferred return in witness view;
3. add missing `I |= Iterator` source contracts;
4. type mapper/predicate contracts;
5. make result generic `U` inferable without defaulting;
6. retain negative well-formedness/compatibility diagnostics.

Exit:

```text
focused witness tests green
adapter declarations are well-formed under their own constraints
underconstrained mapper-result tests resolved
```

### T8 — Universe Iterator/Iterable restoration

Required work:

- rerun bootstrap diagnostic ledger;
- classify every prior 20+3 diagnostic;
- audit terminal Dynamic APIs;
- make precise source changes where justified;
- preserve accepted adapter semantics.

Exit:

```text
Universe bootstrap clean for all P5-owned diagnostics
```

Do not enter end-to-end acceptance by bypassing the global precondition.

### T9 — Natural collection integration

Primary source:

```phalcom
(0..).iter.take(10).toList
```

Also verify:

- `toMap` single-advance regression;
- Chain Item equality negative;
- skip/take invalid runtime inputs;
- existing adapter Item projections;
- eligible Derived Iterable proof and lowering.

Exit:

```text
natural chain is semantic + lowering + runtime green
```

### T10 — Metadata, incremental, and tooling closure

Required only for new durable products:

- optional presence metadata;
- anonymous remainder metadata;
- partial-Dynamic application metadata;
- schema validation;
- import/cold round-trip;
- fingerprint semantics;
- incremental invalidation;
- source index;
- navigation/completion/semantic-token/presentation updates.

Exit:

```text
no module/incremental/tooling path erases a P5 semantic distinction
```

### T11 — Rest/module-export evidence transport

Required order:

1. resolve semantic callable;
2. establish specialization;
3. inspect non-empty evidence recipe;
4. lower recipe;
5. execute forwarding path;
6. localize any evidence loss.

Exit:

```text
a required conformance environment is proven to survive forwarding
or semantic analysis proves that no such environment should exist
```

### T12 — Law/property and focused regression matrix

Run:

- structural laws;
- nested-Dynamic controls;
- witness negative controls;
- P4.R1 canonical result regressions;
- accepted iterator-adapter controls;
- conformance-lowering controls.

### T13 — Priority-3 causal disposition

For each residual:

```text
P5-causal
    -> fix and verify

not P5-causal
    -> preserve classification

uncertain
    -> focused trace, not broad speculative patch
```

### T14 — Broad certification

Only after focused gates:

- relevant semantic aggregates;
- collection corpus;
- affected core suites;
- workspace checks/tests under closure policy;
- Clippy;
- formatter gate;
- baseline failure ledger.

### T15 — Documentation and C3 closure handoff

Required outputs:

- effective normative specs;
- updated checkpoint;
- P5 walkthrough;
- exact certification evidence;
- remaining baseline ledger;
- no stale incident/handoff language presented as current architecture.

---

## 23. Dependency graph

The recommended dependency graph is:

```text
T1
 |
 v
T2 -----------------------------+
 |                              |
 v                              v
T3                             T4
 |                              |
 v                              v
T5                             T6
  \                            /
   \                          /
    +----------> T7 <--------+
                 |
                 v
                T8
                 |
                 v
                T9
                 |
          +------+------+
          |             |
          v             v
         T10           T11
          \             /
           +-----+-----+
                 |
                 v
                T12
                 |
                 v
                T13
                 |
                 v
                T14
                 |
                 v
                T15
```

Notes:

- T3 and T4 can proceed independently after T2's semantic ownership baseline.
- T7 can begin its focused witness reproducer before all structural work is
  complete, but final Universe acceptance must use the final canonical
  contracts.
- T8 must not be declared complete merely because the natural chain's immediate
  adapter starts resolving; the full bootstrap ledger must be dispositioned.
- T10 is conditional on which new products are durable, but at minimum existing
  incremental/fingerprint invariants must be re-run after P5.
- T11 must not be moved ahead of specialization-recipe proof.

---

## 24. Acceptance gates

### G0 — Baseline/lifecycle

Pass criteria:

- pinned revision;
- current P4.R1 state accurately recorded;
- blockers reproduced/source-verified;
- no C3 completion claim.

### G1 — Structural authority and representation

Pass criteria:

- one effective normative structural owner;
- closed/open rule explicit;
- optional presence explicit;
- anonymous remainder semantics explicit;
- captured row distinct;
- `ProperTypeId` boundary unchanged.

### G2 — Structural laws

Minimum controls:

```text
closed exact negative
explicit open positive
open source -> closed target negative
typed remainder positive/negative
optional absent positive
optional present compatible positive
optional present incompatible negative
Tuple positional-only openness
Tuple labeled-only openness
complete-lane equivalence if retained
captured row != anonymous remainder
ordinary subtype rejects non-Type pack domains
```

### G3 — Nested Dynamic

Minimum formation controls:

```text
Dynamic
List<Dynamic>
List<Any>
List<Int>
List<T>
List<unresolved-name>
```

and generic-result controls:

```text
dependent generic result + Dynamic premise
fixed generic result + Dynamic premise
```

If the product is durable, also require metadata/import/runtime representation
tests appropriate to the chosen durability boundary.

### G4 — Witness specialization

Focused test proves:

```text
canonical provider inferred result remains declaration-owned
candidate use-site result is materialized
requirement is materialized
compatible relation succeeds
negative mismatch still fails
```

### G5 — Adapter source contracts

Pass criteria:

- every associated projection is justified by declared evidence;
- Iterator-dependent body operations are legal under declared bounds;
- mapper `U` is inferable from callable contracts;
- no Any/Dynamic/defaulting shortcut.

### G6 — Universe bootstrap

The prior:

```text
20 ImplConformanceIncomplete
3 type.generic.underconstrained
```

are eliminated or individually reclassified outside P5 causality with evidence.

### G7 — Natural chain

`(0..).iter.take(10).toList` passes semantic, lowering, and runtime assertions.

### G8 — Collection regressions

Pass:

- accepted adapter projection fixture;
- Chain equality controls;
- `toMap` single-advance/value controls;
- skip/take runtime validation;
- eligible/ineligible derived Iterable controls.

### G9 — Persistence/incremental/tooling

When new durable representations exist:

- encode/decode/validation round-trip;
- imported behavior equals local behavior;
- cold equals incremental;
- fingerprint changes iff semantic contract changes;
- presentation does not collapse distinctions.

### G10 — Rest/evidence transport

Pass criteria:

- actual call specialization exists;
- non-empty conformance recipe exists;
- lowering retains recipe;
- runtime forwarding retains recipe;
- no VM proof search.

### G11 — Focused C3 matrix

All P5-specific and predecessor-invariant selectors relevant to modified
subsystems are green.

### G12 — Broad C3 closure

Pass criteria:

- relevant corpus aggregates dispositioned;
- workspace certification completed under project policy;
- known baselines explicitly reported;
- checkpoint/spec/walkthrough agree.

---

## 25. Focused test matrix

### Existing P4.R1 controls to preserve

```text
trait_default_inferred_return_is_canonical_and_independent_of_caller_expectation

branch_derived_tail_type_is_published_to_unannotated_callable_signature

semantic::incremental::callable_dependencies::
  inferred_return_refresh_preserves_stable_products_and_counts_final_disposition_once

semantic::incremental::product_stability::
  inferred_cross_module_result_updates_dependents_and_matches_cold_analysis

semantic::integration::metadata::
  cross_module_inferred_callable_result_round_trips_as_compiler_inferred_metadata
```

### Existing Iterator/derived-evidence controls

```text
iterator_adapter_fixture_preserves_item_projections_and_chain_equality

language::iteration::
  known_list_iterator_source_receiver_gets_guarded_iterable_to_list_evidence

language::traits::
  ordinary_trait_lowering_preserves_exact_and_derived_dispatch_evidence
```

The latter T6 selectors are currently blocked by the global Universe semantic
precondition in the active checkpoint. Historical green evidence does not count
as a post-P5 pass.

### Existing rest/evidence seam

```text
language::generic_conformance::
  p4_specialized_rest_dispatch_preserves_conformance_environment
```

This probe must first assert callable/specialization recipe existence before a
runtime failure can be attributed to evidence transport.

### New required P5 coverage

Names may differ, but coverage must include equivalents of:

```text
structural_closed_record_is_exact
structural_explicit_open_record_accepts_extra_fields
structural_optional_presence_is_not_option_value_type
structural_anonymous_remainder_is_not_captured_row
structural_tuple_lane_openness_is_independent
ordinary_subtype_rejects_pack_domains

nested_dynamic_application_preserves_outer_head
nested_dynamic_application_is_not_any
nested_dynamic_dependent_result_remains_dynamic
nested_dynamic_fixed_result_remains_known
nested_dynamic_metadata_roundtrip        # only if durable

generic_inherent_witness_specializes_canonical_inferred_result
generic_inherent_witness_wrong_result_remains_incompatible

iterator_adapter_projections_require_iterator_bound
mapped_iterator_result_generic_is_inferred_from_mapper_return
filter_map_result_generic_is_inferred_from_option_mapper
map_while_result_generic_is_inferred_from_option_mapper

range_take_to_list_preserves_full_evidence_chain
```

---

## 26. Forbidden shortcuts

### Structural

- Do not preserve old closed-record width subtyping merely because current
  tests encode it if the ratified specification adopts exact closed structures.
- Do not encode optional presence as `Option<T>`.
- Do not encode optional presence as `T | Unit`.
- Do not encode optional presence as Dynamic.
- Do not fabricate a row variable for anonymous openness.
- Do not collapse captured-row identity into anonymous remainder.
- Do not interpret `...` as Any or Dynamic.
- Do not reinterpret the Ellipsis singleton as a hidden TypePack/RecordFields
  term.

### Pack/static-domain

- Do not weaken `ProperTypeId`.
- Do not make pack terms ordinary subtype operands because they are stored
  under `TypeId`.
- Do not convert canonical equality into `<:` without a language rule.

### Dynamic application

- Do not add `TypeData::Dynamic` solely to fit nested Dynamic.
- Do not map Dynamic to Any.
- Do not map Unknown to Dynamic.
- Do not fabricate a synthetic Dynamic `TypeId`.
- Do not erase `List<Dynamic>` to bare `List`.
- Do not special-case `List`, `RangeIterator`, `TakeIterator`, or `toList`.
- Do not add a source return annotation merely to hide unresolved body
  semantics.

### Callable result/conformance

- Do not add a second inferred-result owner.
- Do not perform consumer-side result inference.
- Do not add a general inferred-result fixpoint.
- Do not weaken witness compatibility to silence bootstrap errors.
- Do not treat associated projection syntax as implicit conformance evidence.
- Do not invent/default mapper result `U`.

### Bootstrap/tests

- Do not bypass Universe semantic errors.
- Do not downgrade conformance errors for gate passage.
- Do not claim a selector passed when it failed before focused assertions.
- Do not substitute historical green evidence for a post-P5 run.

### Runtime evidence

- Do not patch VM forwarding before a semantic specialization recipe exists.
- Do not add VM proof search.
- Do not reconstruct conformance from raw implementation identity.

### Scope

- Do not absorb every Priority-3 failure without causality.
- Do not use a broad workspace failure as a substitute for localization.

---

## 27. Intrinsic/source-owned versus compiler-owned semantics

The architectural objective is not to make Tuple, Record, Iterator, or Dynamic
compiler magic. The source layer should expose the semantic façade while the
compiler owns only the closed foundational machinery.

### Source/intrinsic façade

Current direction includes:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data CompleteTypePack
intrinsic type data RecordFields

intrinsic data Tuple<***Es>(***Es)
intrinsic data Record<**Fs> { **Fs }
```

Iterator/Iterable relationships should likewise remain ordinary source
contracts wherever expressible.

### Compiler-owned closed machinery

Appropriate compiler ownership includes:

```text
canonical kind/domain identity
pack normalization/concatenation
record-row solver and correlation
structural shape normalization
anonymous remainder relation
presence relation
proper-type subtyping
type-member projection normalization
partially-known type-form/application knowledge
canonical callable-result publication
use-site callable specialization
trait witness checking
evidence construction/projection
incremental dependency/fingerprint machinery
```

### What should fall out of the type system

Once these layers are coherent, there should be no compiler-specific typing
rule for:

```text
RangeIterator
TakeIterator
MappedIterator
ChainIterator
toList
```

Their behavior should derive from generic constraints, associated Item
contracts, constructors, trait conformance, the derived Iterator=>Iterable
rule, and canonical callable result specialization.

---

## 28. Open decisions that P5 must close

### P5-DEC-01 — Closed structural exactness

Choose whether closed Tuple/Record types are exact or retain classic width
subtyping.

**Recommendation:** exact closed structures; width acceptance only through
explicit openness.

### P5-DEC-02 — Anonymous remainder representation

Choose canonical representation for:

```text
closed
unconstrained anonymous remainder
typed anonymous remainder
captured row tail
projection-derived tail
```

### P5-DEC-03 — Optional member representation

Choose where presence lives and how it participates in identity, mapping,
metadata, relation, and tooling.

### P5-DEC-04 — Nested Dynamic semantics

Choose whether explicit nested Dynamic preserves outer generic structure.

**Recommendation:** preserve known outer head in a partially-known application
product without making Dynamic a proper type.

### P5-DEC-05 — Partial application durability

Decide whether partial-Dynamic application knowledge is:

```text
analysis-only
module-contract durable
runtime-reflectable
```

### P5-DEC-06 — Metadata schema impact

If structural presence/remainders or partial-Dynamic products are exported,
decide whether metadata schema changes are required and define validation/
compatibility behavior.

### P5-DEC-07 — Universe terminal precision

Classify `toList`, `partition`, `toSet`, `group`, and `toMap` Dynamic uses and
choose the smallest precise source surface needed for P5.

### P5-DEC-08 — Rest/module-export root cause

Only after specialization exists, decide whether the remaining failure belongs
to:

```text
source resolution
semantic call specialization
lowering recipe generation
runtime forwarding
```

---

## 29. Stop/escalation triggers

The P5 implementer must stop and escalate rather than improvise if:

1. exact/open structural work unexpectedly requires changing nominal class
   subtyping or unrelated family width semantics;
2. anonymous remainder implementation requires treating a row variable as an
   existential and would lose correlation/lacks laws;
3. optional presence requires public metadata changes not authorized by the
   plan;
4. nested Dynamic appears to require making Dynamic a proper `TypeId`;
5. `List<Dynamic>` resolution would require a List-specific branch;
6. witness repair would require mutating canonical provider signatures per
   conformance target;
7. Iterator bootstrap remains red after confirmed witness/source-contract
   repairs with a materially new cause;
8. a generic can only be resolved by defaulting rather than a declared
   contract;
9. rest/module-export runtime work still lacks a semantic recipe;
10. incremental parity would require a second semantic cache/authority;
11. broad certification exposes unrelated failures that materially expand
    P5 scope.

---

## 30. Performance and incremental requirements

### P5-REQ-150 — No new whole-graph semantic fixpoint

Do not turn structural normalization or partial-Dynamic propagation into a
global iterative repair loop.

### P5-REQ-151 — Canonical fingerprints

New structural/partial products must have deterministic fingerprints derived
from canonical semantic content, not source rendering, hash-map iteration, or
allocation identity.

### P5-REQ-152 — Explicit dependencies

Consumers depending on presence, remainders, partial-Dynamic arguments,
specialized inferred results, or evidence recipes must record the owning
semantic dependency so relevant edits invalidate them and irrelevant edits do
not.

### P5-REQ-153 — Measure conformance-refresh impact

If witness specialization changes conformance plan inputs, verify it remains a
projection/materialization correction and does not reintroduce broad
conformance-to-inference feedback.

---

## 31. Definition of P5 completion

### Semantic model

- [ ] structural spec ownership is singular;
- [ ] closed/open semantics explicit;
- [ ] optional presence explicit;
- [ ] anonymous remainder semantics explicit;
- [ ] captured rows remain distinct;
- [ ] nested Dynamic semantics explicit;
- [ ] pack/static-domain relation boundary preserved.

### Representation

- [ ] every ratified structural syntax has canonical semantic representation;
- [ ] partial-Dynamic application has a canonical owner if adopted;
- [ ] no distinction exists only in source spelling;
- [ ] durable distinctions survive metadata if required;
- [ ] tooling presents distinctions without collapse.

### Generic/conformance

- [ ] witness-specialization hypothesis proven or disproven;
- [ ] confirmed defect repaired through use-site materialization;
- [ ] Iterator adapter bounds are well-formed;
- [ ] mapper-result generics are inferable;
- [ ] negative witness compatibility remains strict.

### Universe

- [ ] prior bootstrap diagnostics causally dispositioned;
- [ ] P5-owned bootstrap errors gone;
- [ ] Dynamic collection uses classified;
- [ ] accepted adapter semantics retained;
- [ ] `toMap` correction retained.

### End-to-end

- [ ] `(0..).iter.take(10).toList` fully established;
- [ ] Exact/Derived evidence survives lowering;
- [ ] final result matches adopted collection/Dynamic semantics;
- [ ] runtime behavior correct.

### Evidence transport

- [ ] rest/module-export probe has an actual specialization recipe;
- [ ] proven transport defects corrected without VM proof search.

### Incremental/tooling

- [ ] cold/incremental products agree;
- [ ] fingerprints change only with semantic contracts;
- [ ] metadata validation/round-trip complete if representation changed;
- [ ] affected LSP/source-index surfaces updated.

### Certification

- [ ] focused P4.R1 regression selectors remain green;
- [ ] P5 law/property matrix green;
- [ ] relevant collection/core aggregates dispositioned;
- [ ] broad workspace certification completed under project policy;
- [ ] residual baselines explicitly recorded;
- [ ] checkpoint, walkthrough, and normative specs agree.

---

## 32. Required handoff to the P5 planner

The next agent should perform an independent verification pass, but it should
not repeat the entire historical audit.

Minimum re-verification:

```text
1. current main SHA and checkpoint state
2. effective structural spec ownership
3. live Record relation law
4. AST/RecordRow representation
5. nested-Dynamic early-return path
6. TypeData/metadata/runtime static-argument categories
7. specialize_witness_signature vs TraitSurfaceMember::instantiate
8. Iterator adapter bounds and mapper signatures
9. Universe bootstrap diagnostic ledger
10. rest/module-export specialization products
```

Then the implementation plan must translate the work packages into patch-grade
tasks containing:

```text
file/symbol scope
preconditions
exact invariants
focused selectors
negative controls
stop conditions
commit boundaries
checkpoint update points
```

The implementation plan must not begin by opportunistically patching the
compiler. Specification/representation rulings come first.

---

## 33. Source and symbol index

### Specification

```text
docs/specs/README.md
    specification authority and ownership map

docs/specs/structural-products.md
    structural Tuple/Record owner and closed operations

docs/specs/tuple-and-record-typing.md
    closed/open/optional/remainder model requiring reconciliation

docs/specs/generic-constraints.md
    proper-type subtype relation, generic constraints, projection obligations

docs/specs/objects/data-objects.md
    exact applied data-object identity/runtime observability
```

### LANG007 lifecycle and incidents

```text
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md

docs/implementation/LANG007/LANG007.C3/
  LANG007.C3.P4.R1-canonical-inferred-result-contracts.md

docs/implementation/LANG007/LANG007.C3/INC-C3-27.md

docs/implementation/LANG007/LANG007.C3/
  LANG007-C3-completion-audit-5f872da0.md

docs/implementation/LANG007/LANG007.C3/
  LANG007.C3.P4-audit-remediation/
    03-nested-derived-conformance-runtime-layout.md
    04-semantic-regression-classification.md
```

### AST / structural representation

```text
phalcom-ast/src/ast.rs
    TypeAnnotationExpr::Record
    RecordTypeField
    RecordRowTail
    TypeTupleItem
    TypePackExpansionSyntax

phalcom-semantic/src/types/row.rs
    RecordRowField
    RecordRowData
    RecordRowTail

phalcom-semantic/src/types/store.rs
    TypeData
    StaticArgument
    TupleTypeElement
```

### Type formation / relation

```text
phalcom-semantic/src/types/annotation.rs
    TypeFormResolution
    resolve_type_form
    resolve_type_annotation
    TypeFormationSite

phalcom-semantic/src/types/relation.rs
    check_subtype_bounded
    check_subtype_impl
    check_record_row_subtype
    check_assignability_bounded
```

### Callable result / trait conformance

```text
phalcom-semantic/src/traits.rs
    TraitSurfaceMember::instantiate
    InstantiatedTraitRequirement

phalcom-semantic/src/impls.rs
    specialize_witness_signature
    witness_signature_return_type
    check_witness_compatibility
    check_witness_compatibility_with_visibility
    ConformanceWitnessPlan
```

### Metadata/runtime typing

```text
phalcom-type-meta/src/type_node.rs
    TypeNode
    StaticArgumentRef
    RecordRowRef

phalcom-semantic/src/metadata/export.rs

phalcom-core/src/typing/overlay.rs
    RuntimeOverlayTypeNode
    RuntimeStaticArgument
    RuntimeRecordRow
```

### Universe

```text
phalcom-core/core/universe/src/collections/iterator_laws.ph

phalcom-core/core/universe/src/collections/range.ph
    RangeIterator::take
    RangeIterator::toList
```

### Runtime evidence forwarding

```text
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/bytecode.rs

phalcom-core/tests/core/language/generic_conformance.rs
    p4_specialized_rest_dispatch_preserves_conformance_environment

phalcom-core/tests/core/language/traits.rs
    ordinary_trait_lowering_preserves_exact_and_derived_dispatch_evidence

phalcom-core/tests/core/language/iteration.rs
```

---

## 34. Final requirements summary

P5 should be accepted for implementation only if its plan explicitly covers:

1. Reconcile structural Tuple/Record specification authority.
2. Ratify exact-closed versus width semantics before relation edits.
3. Represent optional presence as a first-class structural property.
4. Represent anonymous typed/unconstrained remainders independently from
   captured rows.
5. Keep packs/rows/static domains out of ordinary proper-type subtyping.
6. Define coherent semantics for `List<Dynamic>` and equivalent nested Dynamic
   applications.
7. If known-head semantics are adopted, represent dynamic argument slots
   without fabricating a proper Dynamic type.
8. Carry the adopted representation through every affected consumer and
   persistence boundary.
9. Prove and, if confirmed, repair inferred-result specialization for generic
   witnesses without adding a second result authority.
10. Make Iterator adapter projections/body operations well-formed under
    explicit constraints.
11. Make mapper result generics inferable through typed callable contracts.
12. Eliminate or disposition the current 20 conformance-incomplete and 3
    underconstrained Universe diagnostics without bypassing bootstrap.
13. Preserve accepted Chain/FlatMap/Flatten/Zip/Indexed/toMap corrections.
14. Restore `(0..).iter.take(10).toList` through ordinary type, conformance,
    derived-evidence, lowering, and runtime machinery.
15. Establish a real specialization/evidence recipe before diagnosing
    rest/module-export forwarding as runtime evidence loss.
16. Close structural/Dynamic semantics with direct laws and
    metadata/incremental/tooling parity where durable.
17. Own only causally related Priority-3 residuals.
18. Complete broad certification and converge checkpoint, walkthrough, and
    normative specs before claiming C3 closure.

The central requirement is that P5 close C3 by making the semantic model more
coherent, not by accumulating local exceptions around the remaining tests.
