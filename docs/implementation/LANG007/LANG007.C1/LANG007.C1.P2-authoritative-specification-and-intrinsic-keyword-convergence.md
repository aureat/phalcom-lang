---
id: LANG007.C1.P2
category: LANG
program: LANG007
checkpoint: LANG007.C1
kind: implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on:
  - LANG007.C1.P1 complete, implemented, and focused-tested
follows: LANG007.C1.P1
supersedes: null
repository: aureat/phalcom-lang
branch: main
planning_revision: e99ad77923893afa00b46141c8cf88932cad8f78
planning_revision_subject: "test(semantic): preserve class-side generic call products"
prepared: 2026-09-22
activation_revision: e99ad77923893afa00b46141c8cf88932cad8f78
activation_working_tree: uncommitted scoped P1 semantic/test and lifecycle records plus this P2 plan
executor: GPT-5.6 Luna High/Extra High
scope_note: post-P1 decision/specification convergence plus retirement of the obsolete @intrinsic source/presentation attribute; intrinsic remains the canonical keyword
---

# LANG007.C1.P2 — Authoritative Specification and `intrinsic` Keyword Convergence

## 0. Executor contract

This is a Luna-oriented patch-grade plan for the second numbered plan in `LANG007.C1`.

P2 starts **only after `LANG007.C1.P1` is fully delivered**. That start lock
has now been satisfied: the repository contains the completed P1
implementation, truthful C1 checkpoint state, P1 walkthrough, and P1 handoff.
This plan is active at T0; substantive specification/source edits remain within
its ordered scope.

### Activation record

- Activation revision: `e99ad77923893afa00b46141c8cf88932cad8f78`
- Working tree: uncommitted scoped P1 semantic/test changes, P1 lifecycle
  records, and this P2 plan; these changes are preserved intentionally.
- P1 closure: `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`
- P2 state: `IN_PROGRESS / PARTIAL / UNVERIFIED`

The P2 start lock is satisfied before substantive P2 edits begin.

The implementer:

- may adapt mechanical paths, helper names, and test filters to the final P1 tree;
- must consume the final P1 walkthrough/handoff rather than reconstructing P1 from history;
- must preserve the completed P1 semantic architecture;
- must not reopen the type-level domain, canonical identity, projection, `At`, `Map`, open-row, inference, metadata, source-index, or no-VM decisions established by P1;
- must treat the explicit user ruling in this plan as authoritative: **`intrinsic` is a source keyword / implementation kind, and the source attribute `@intrinsic` is retired and must be removed from current language authority and source-like presentation**;
- must not revert `ImplementationKind::Intrinsic`;
- must not replace `intrinsic type ...` with `native type ...` merely because older TDR/spec text still says so;
- must not delete or rename internal `NativeIntrinsicId` metadata merely because the source attribute is removed;
- must preserve decision history by recording a new amendment decision rather than rewriting older accepted decisions to pretend they never said `@intrinsic`;
- must use the final P1 implementation as evidence, not as permission to silently promote adjacent deferred language features;
- must follow the narrow verification budget below;
- must classify and defer unrelated failures;
- must STOP AND CONSULT when a fixed semantic, ownership, identity, or authority boundary would need to change;
- must keep the shared C1 checkpoint record current;
- must produce the required P2 walkthrough and P2 handoff.

Testing is evidence gathering. Do not run broad compiler/runtime suites merely because specifications changed.

### P2 start lock

Before any substantive P2 edit, all of the following must be true:

```text
LANG007.C1.P1 status:       COMPLETE
LANG007.C1.P1 completion:   IMPLEMENTED
LANG007.C1.P1 verification: FOCUSED_TESTED

LANG007.C1 checkpoint:
    records P1 as complete
    records actual P1 final revision
    records P1 gate evidence
    identifies P2 as next action

required files:
    LANG007.C1.P1-walkthrough.md
    LANG007.C1.P1-handoff.md
```

If any of those is missing, **STOP P2**. Finishing P1 is not P2 scope.

## 1. Goal

Converge Phalcom's accepted decision registry, normative language specifications, source/presentation surface, and current LANG007 lifecycle records on the fully implemented C1 semantics, with `intrinsic` as the canonical fourth implementation-kind keyword, no live `@intrinsic` source attribute, one mathematically coherent multi-domain type-member calculus, and no accidental promotion of deferred generic/type-level features.

## 2. Checkpoint acceptance objective

`LANG007.C1` owns the bounded objective:

> Phalcom has one coherent, implemented, specified, durable compiler model for `TypePack`, `LabeledTypePack`, typed type-member arguments, canonical type-member identity, symbolic projections, Tuple `Positionals`/`Labeled`/`At`, structural `Map<F>`, open-row mapping, inference/materialization, stable metadata/source identity, canonical Universe anchors, and zero-VM structural type computation.

P1 is assumed to have implemented and focused-tested that vertical.

P2 is **not corrective P1 implementation**. P2 advances the same checkpoint by making the authoritative decision/specification corpus and source-like presentation exactly describe the completed P1 architecture and the new `intrinsic` keyword ruling.

P2 is one of several C1 plans, not automatic checkpoint closure. Unless the checkpoint is explicitly amended, `LANG007.C1.P3` remains the successor architecture-baseline/acceptance-matrix plan.

P2 is accepted when:

1. the `intrinsic` keyword is ratified as `ImplementationKind::Intrinsic`;
2. `@intrinsic` is no longer a builtin source attribute and no current normative spec or source-like presentation emits it;
3. internal `NativeIntrinsicId` metadata remains available where it represents compiler/runtime optimization or semantic-operation metadata distinct from source implementation kind;
4. a new accepted TDR records the keyword decision as an amendment to the older `@intrinsic` model without rewriting decision history;
5. the decision registry lists all accepted LANG007 TDRs and the new intrinsic-keyword amendment;
6. every P1 semantic invariant has one primary normative specification owner;
7. `type-level-domains.md`, `type-members.md`, and `structural-products.md` exactly describe P1's implemented calculus at representation-independent semantic level;
8. neighboring generic/associated/trait/impl/Universe/value/VM/callable specs delegate consistently and do not duplicate authority;
9. `T[I]` and `T::At<I>` are specified as one canonical type-subscript member identity, with `At` only a named presentation;
10. `I: Int` is specified as `Const(Int)` only in declaration forms whose grammar admits that domain; C1 does not silently become general nominal or trait const generics;
11. the initial trait-associated surface remains unparameterized and preserves its P1-supported result-domain boundary; C1 does not silently become GAT support;
12. structural `Map<F>` is completely specified for `TypePack`, `LabeledTypePack`, Tuple, `RecordRow`, and Record, including symbolic/open-row cases and the algebraic laws below;
13. zero Tuple/Unit and empty Record static/runtime distinctions are consistent across language and VM specs;
14. valid symbolic projections remain symbolic, concrete invalidity remains invalid, and operational compiler states do not become language types;
15. structural operations do not implicitly distribute over unions and do not use runtime `Dynamic` as an applicability escape hatch;
16. domain-native structural members still do not make anonymous Tuple/Record shapes ordinary inherent-`impl` targets;
17. no runtime `TypePack`/`RecordRow` object model or type-member VM dispatch is introduced;
18. repository-wide contradiction/deletion searches find no live authoritative `@intrinsic` rule or LANG007 semantic contradiction;
19. focused code tests prove the narrow `@intrinsic` retirement / `intrinsic` preservation patch;
20. lifecycle records truthfully describe P2 and identify P3 or the amended next action.

## 3. Repository grounding

Prepared against:

```text
repository:          aureat/phalcom-lang
branch:              main
planning revision:   e99ad77923893afa00b46141c8cf88932cad8f78
revision subject:    test(semantic): preserve class-side generic call products
parent P1 landing:   479f8fb82fa58730fb2c40c2652b5e07016549ec
planning date:       2026-09-22
```

### 3.1 Current implementation facts verified during planning

At the planning revision:

1. `phalcom-common/src/modifier.rs` already defines `ImplementationKind::{Phalcom, Native, Foreign, Intrinsic}` and exposes `is_intrinsic()` / `"intrinsic"` presentation.
2. `phalcom-ast/src/token.rs` reserves `intrinsic` as a keyword and describes it as introducing compiler-intrinsic declarations.
3. `phalcom-ast/tests/parser.rs` contains coverage for `intrinsic class`, `intrinsic data`, `intrinsic type` aliases, intrinsic fields/methods, and intrinsic structural type-member declarations.
4. canonical Universe `Tuple` and `Record` source already use `intrinsic type ...` for the LANG007 structural type members.
5. `BuiltinAttr::Intrinsic` still exists in `phalcom-ast/src/ast.rs`, and `BuiltinAttr::parse("intrinsic")` still classifies `@intrinsic` as a builtin attribute.
6. `phalcom-core/src/compiler/attributes.rs` still installs a `MetadataExpander` for `BuiltinAttr::Intrinsic` and contains target handling for it.
7. `phalcom-semantic/src/core_surface/presentation.rs` still renders internal native intrinsic metadata as source-like `@intrinsic(BoolNot)`.
8. `phalcom-semantic/tests/semantic/integration/native_conformance.rs::test_presentation_ir_and_virtual_source` still requires that obsolete rendering.
9. `NativeIntrinsicId` remains an internal native-surface metadata channel used by `BoolAnd`, `BoolOr`, and `BoolNot`; this is not the source attribute and must survive.
10. `docs/specs/source-declarations.md` still states that `@intrinsic` is metadata-only.
11. `docs/specs/type-members.md` still states that `@intrinsic` remains builtin metadata and uses the older `native type` model.
12. TDR-0082 still says `@primitive` and `@intrinsic` are metadata-only Universe attributes.
13. TDR-0088 preserves that older distinction and uses `@intrinsic` + `native type` examples.
14. accepted LANG007 TDR files 0083–0088 exist, but `docs/decisions/README.md` still reports 82 accepted TDRs and ends the accepted table at TDR-0082.
15. `type-level-domains.md`, `type-members.md`, and `structural-products.md` already exist and are indexed as normative owners.
16. neighboring generic/associated/trait/impl/Universe/value/VM specs already contain partial LANG007 convergence.
17. `type-members.md` currently says `At<I>` is the canonical named member and subscript is its alias, while TDR-0086 says the canonical semantic identity is the type-level subscript member and `At` is the named presentation alias.
18. current associated/trait specs discuss non-`Type` result domains more broadly than original P1 scope required; P2 must align this with final P1 rather than equating representational capability with source-language admission.
19. current C1 checkpoint/guidance still preserve the older `native`/`@intrinsic` distinction and stale lifecycle state.

### 3.2 Current lifecycle facts

At planning time the C1 checkpoint still lists P1 as proposed/not-started/unverified, no P1 walkthrough exists, and no P1 handoff exists. T0 must consume the final state, not this snapshot.

### 3.3 Drift instruction

> Re-read every primary path named in this plan at P2 start. Adapt mechanical drift locally. Treat changes to semantic ownership, identity, source authority, implementation-kind meaning, P1 feature boundary, or TDR authority as STOP/CONSULT conditions.

## 4. Required reads before implementation

Read in this order.

### 4.1 Repository/workflow authority

```text
AGENTS.md
docs/implementation/README.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/commit-and-push-discipline.md
```

### 4.2 C1 takeover records

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1-GUIDANCE.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P1-semantic-domains-and-compiler-intrinsic-structural-type-operations.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P1-walkthrough.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P1-handoff.md
```

### 4.3 Decision authority

```text
docs/decisions/README.md
docs/decisions/accepted/0081-tuple-and-record-are-transparent-structural-value-products.md
docs/decisions/accepted/0082-declaration-modifiers-and-identity-based-access-domains.md
docs/decisions/accepted/0083-type-level-domains-and-typed-compile-time-parameters.md
docs/decisions/accepted/0084-canonical-type-level-terms-and-inference-lifetimes.md
docs/decisions/accepted/0085-parameterized-type-members-and-symbolic-projections.md
docs/decisions/accepted/0086-type-level-subscript-is-tuple-at.md
docs/decisions/accepted/0087-structural-map-is-shape-preserving.md
docs/decisions/accepted/0088-native-type-operations-use-closed-compiler-normalization.md
```

### 4.4 Normative specification owners

Primary:

```text
docs/specs/README.md
docs/specs/type-level-domains.md
docs/specs/type-members.md
docs/specs/structural-products.md
docs/specs/source-declarations.md
```

Neighboring:

```text
docs/specs/generic-constraints.md
docs/specs/associated-types.md
docs/specs/objects/impl.md
docs/specs/objects/traits.md
docs/specs/universe.md
docs/specs/objects/values.md
docs/specs/vm/products.md
docs/specs/vm/values-and-objects.md
docs/specs/callables/INDEX.md
docs/specs/callables/dispatch.md
docs/specs/callables/selectors.md
docs/specs/callables/references.md
```

### 4.5 Narrow code surface for `@intrinsic` retirement

```text
phalcom-common/src/modifier.rs
phalcom-ast/src/token.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/tests/parser.rs
phalcom-core/src/compiler/attributes.rs
phalcom-semantic/src/core_surface/presentation.rs
phalcom-semantic/tests/semantic/integration/native_conformance.rs
phalcom-native-meta/src/primitive.rs
phalcom-native-surface/src/lib.rs
phalcom-native-macros/src/lib.rs
phalcom-native-surface-gen/src/main.rs
```

Native-meta/native-surface files are read-only boundary evidence unless final P1 mechanically moved ownership.

### 4.6 P1 implementation evidence

Read final P1 paths named by the walkthrough/handoff for canonical type-member identity/signatures, projection/result-domain ownership, intrinsic type-operation evaluator, Const(Int), pack terms, mapped row tails, inference/materialization, stable metadata, source-index identity, and Universe publication.

P2 must describe the semantic contract without hard-coding private Rust layouts.

## 5. Normative authority

### 5.1 Explicit user ruling

FIXED:

```text
intrinsic is a source keyword.
intrinsic is represented as ImplementationKind::Intrinsic.
@intrinsic is retired as a source/builtin attribute.
P2 continues the intrinsic keyword architecture; it does not revert it.
```

### 5.2 New amendment TDR

P2 must encode this ruling in a new accepted TDR. At the planning revision the next free accepted file number is `0089`.

Expected decision:

```text
TDR-0089
    intrinsic is a first-class implementation-kind keyword;
    @intrinsic is retired from source language / source-like presentation
```

It must amend the relevant portions of TDR-0082 and TDR-0088.

Do not rewrite TDR-0082 or TDR-0088 to erase history. If 0089 is occupied at P2 start, use the next canonical accepted number and update links/registry atomically.

### 5.3 Accepted LANG007 semantics

Subject to the intrinsic amendment:

- TDR-0083 owns type-level domains and `Const(τ)`.
- TDR-0084 owns stable canonical terms versus ephemeral inference variables.
- TDR-0085 owns parameterized domain-aware type members and symbolic projections.
- TDR-0086 owns one canonical type-subscript identity for `T[I]` and `T::At<I>`.
- TDR-0087 owns shape-preserving structural `Map<F>`.
- TDR-0088 continues to own the closed compiler-normalization/no-VM boundary, with source-form wording amended by the new TDR.

### 5.4 Final P1 implementation

Completed P1 code/tests are as-built evidence. They do not override the explicit user ruling or accepted semantics. If final P1 materially disagrees, P2 must not document a defect as language law; STOP AND CONSULT.

### 5.5 Future/non-normative proposals

P2 does not promote:

```text
general nominal const generics
trait const generics
generic associated types
parameterized trait-associated members
open/user-defined type families
arbitrary compile-time Phalcom execution
const arithmetic
pack split/concatenation inference
variadic callable inference
Symbol/Range type-level indexing
union-distributive structural members
runtime pack/row objects
anonymous structural inherent impls
type-member VM bytecodes
general pack reflection APIs
```

## 6. Takeover state

P2 assumes P1 established the complete semantic/compiler substrate.

### 6.1 Stable interface map

| Concept | Expected owner | P2 invariant |
|---|---|---|
| declaration implementation kind | `phalcom-common::modifier::ImplementationKind` | includes `Intrinsic` |
| `intrinsic` keyword | AST lexer/parser/token layer | canonical source keyword |
| builtin attributes | `phalcom-ast::BuiltinAttr` | `@intrinsic` removed after P2 |
| compiler attribute expansion | compiler attributes registry | no intrinsic attribute expander |
| internal native intrinsic IDs | `phalcom-native-meta::NativeIntrinsicId` | survives; not source attr |
| source-like presentation | semantic core-surface presentation | legal current syntax only |
| canonical type members | final P1 member registry | identity semantic, not text |
| member signatures | final P1 signature owner | one parameter/result contract |
| canonical projections | final P1 projection owner | member identity + args + domain |
| structural evaluator | final P1 semantic normalizer | deterministic, no VM |
| pack domains | final P1 type/kind store | distinct TypePack/LabeledTypePack/RecordRow |
| compile-time member const | final P1 const-term model | `I: Int` means Const(Int) |
| Tuple shape | final P1 structural type model | positional + ordered labeled lane |
| Record shape | row domain | key identity order-independent |
| open-row Map | final P1 row projection/normalizer | tail mapped symbolically |
| stable metadata | type-meta + P1 exporter | no query-local IDs |
| source member identity | P1 source index / Universe | `[]` and `At` same target |
| runtime boundary | core/VM | no type-member execution |
| decision history | accepted TDRs + registry | amendment, no historical rewrite |
| normative authority | `docs/specs/**` | one primary owner per fact |
| C1 state | C1 checkpoint | single durable lifecycle authority |

### 6.2 Source/presentation model

After P2:

```text
ImplementationKind ::=
    Phalcom
  | Native
  | Foreign
  | Intrinsic
```

Conceptually:

```text
Phalcom
    ordinary Phalcom implementation/body semantics

Native
    category-appropriate host/native binding implementation

Foreign
    bounded foreign implementation category

Intrinsic
    implementation supplied directly by canonical compiler/language intrinsic
    semantics for a declaration category that admits intrinsic implementation
```

`intrinsic` is not visibility.

For structural type members:

```phalcom
intrinsic type Positionals: TypePack
intrinsic type Labeled: LabeledTypePack
intrinsic type At<I: Int>: Type
intrinsic type Map<F: Type -> Type>: Type
```

the implementation is semantic/type-system normalization and performs no VM call.

### 6.3 Internal `NativeIntrinsicId`

Keep separate:

```text
ImplementationKind::Intrinsic
    source declaration implementation kind

NativeIntrinsicId
    internal semantic/optimization identity for selected native surfaces
```

A native Bool operation may remain `Native + NativeIntrinsicId::BoolNot` without source-spelled `@intrinsic(BoolNot)`.

## 7. Architecture

P2 has four dependency-ordered slices:

```text
explicit user ruling
    ↓
new accepted amendment TDR + registry
    ↓
retire @intrinsic builtin/source-like presentation
    ↓
converge normative specification graph
    ↓
cross-corpus contradiction/deletion audit
    ↓
C1 lifecycle / P3 handoff
```

### 7.1 Formal type-level model

Kinds:

```text
κ ::= Type
    | RecordRow
    | TypePack
    | LabeledTypePack
    | (κ₁, ..., κₙ) -> κ
```

Parameter domains:

```text
δ ::= Kind(κ) | Const(τ)
```

Judgments:

```text
Γ ⊢ A : κ
Γ ⊢ c :const τ
```

Products:

```text
P : TypePack
L : LabeledTypePack
R : RecordRow

Tuple(P,L) : Type
Record(R)  : Type
```

Projection:

```text
Projection(
    subject,
    canonical member identity,
    canonical member-local arguments,
    result domain,
    qualification/evidence context where applicable
)
```

Language normalization classes:

```text
Reduced
Stuck
Invalid
```

Implementation may have extra operational states; they do not become language type meanings.

### 7.2 Canonical `At`

Resolve current contradiction in favor of TDR-0086:

```text
canonical semantic member:
    type-level Subscript

presentations:
    T[I]
    T::At<I>
```

Thus:

```text
MemberIdentity(T[I]) = MemberIdentity(T::At<I>)
```

Universe may author `At<I>` as a discoverable source anchor if P1 does so, but identity remains the one canonical subscript member.

### 7.3 Map algebra

Preserve:

```text
MapPack([T₀,...,Tₙ],F)
    = [F<T₀>,...,F<Tₙ>]

MapLabeled([(l₀,T₀),...,(lₙ,Tₙ)],F)
    = [(l₀,F<T₀>),...,(lₙ,F<Tₙ>)]

Tuple(P,L)::Map<F>
    = Tuple(MapPack(P,F), MapLabeled(L,F))

MapRow({lᵢ:Tᵢ},F)
    = {lᵢ:F<Tᵢ>}

MapRow({lᵢ:Tᵢ | R},F)
    = {lᵢ:F<Tᵢ> | MapRow(R,F)}

Record(R)::Map<F>
    = Record(MapRow(R,F))
```

with `F : Type -> Type` and ordinary type-form application for every `F<T>`.

Laws:

```text
shape(S::Map<F>) = shape(S)

labels(T::Map<F>) = labels(T)

(T::Map<F>)::At<I>
    ≡ F<T::At<I>>
    when I is valid

Lookup(Record(R)::Map<F>, K)
    ≡ F<Lookup(Record(R), K>>
    for existing K
```

No implicit union distribution.

### 7.4 Tuple indexing

For arity `N`:

```text
j = i       if i >= 0
    N + i   if i < 0
```

valid iff `0 <= j < N`.

Linear order:

```text
positionals ++ labeled payloads
```

Concrete OOB is `Invalid`.

### 7.5 Zero product/static-runtime boundary

Preserve:

```text
Tuple([],[]) ≡ Unit ≡ ()
Record({}) ≠ Unit          // static
```

VM may share canonical zero-product physical realization without erasing static Record identity.

### 7.6 Associated-type boundary

General type-member theory is broader than the initial trait surface.

C1/P2:

```text
general type-member theory:
    parameter/result-domain capable

domain-native structural members:
    use P1 member-local parameters/results

trait-associated initial surface:
    remains unparameterized
    preserves the P1-supported result-domain boundary
```

No GAT inference from generalized internals.

Unless final P1 explicitly and deliberately implemented a broader associated-result source feature, non-`Type` associated result declarations are not promoted by C1. If P1 unexpectedly did so, STOP AND CONSULT.

### 7.7 Const-generic boundary

`Const(τ)` exists semantically, but C1 requires source use for member-local type-member arguments such as `At<I: Int>`.

Do not claim all class/trait/method generic binders support const parameters.

## 8. Ownership boundaries

### 8.1 Owns

P2 owns:

- new accepted intrinsic-keyword amendment TDR;
- accepted registry synchronization;
- removal of builtin/source-like `@intrinsic`;
- preservation/presentation of `ImplementationKind::Intrinsic`;
- authoritative C1 spec convergence;
- `At`/subscript identity correction;
- explicit const/associated/inherent-type-member source boundaries;
- complete representation-independent structural type theory;
- current C1 lifecycle synchronization;
- focused tests/searches;
- P2 walkthrough/handoff.

### 8.2 Does not own

P2 does not own:

- unfinished P1 implementation;
- type-member evaluator redesign;
- `ImplementationKind` redesign;
- `NativeIntrinsicId` removal/rename;
- native primitive registry redesign;
- general intrinsic optimizer architecture;
- general const generics or GATs;
- user type families;
- pack split inference;
- new structural operations;
- union distribution;
- runtime pack/row objects;
- VM type-member bytecodes;
- broad reflection/LSP redesign;
- historical LANG006/STDL/SEMA record rewriting;
- docs-tree migration;
- workspace release certification.

### 8.3 Source-of-truth table

| Fact | Canonical owner after P2 | Consumers | Forbidden duplicate |
|---|---|---|---|
| source implementation kind | `ImplementationKind` + source declarations spec | parser/semantic/presentation | attribute-derived kind |
| intrinsic keyword decision | new accepted TDR + source spec | all declaration categories | `@intrinsic` rule |
| internal native intrinsic ID | `NativeIntrinsicId` | native/runtime/compiler | source attribute syntax |
| kind/domain grammar | type-level domains spec | members/generics/products | constraints redefinition |
| const parameter domain | type-level domains spec | admitted member grammars | `Int` as kind |
| const source admission | owning declaration grammar | parser/resolver | “all binders” inference |
| type-member identity | type-members spec + P1 registry | resolver/meta/index | name-only lookup |
| projection semantics | type-members spec | associated/structural uses | feature-local ontology |
| trait evidence | associated-types spec | constraints/checker | intrinsic registry |
| Tuple/Record algebra | structural-products spec | normalizer/tests | runtime class |
| row identity | RecordRow semantics | Record Map | labeled pack |
| subscript/At identity | type-members + structural-products | parser/index | two members |
| intrinsic type execution | P1 semantic normalizer + spec | checker | VM/native callable |
| static zero product | structural-products spec | checker/reflection | VM layout |
| VM product realization | vm/products spec | runtime/compiler | static identity |
| lifecycle state | C1 checkpoint | plans/records | parallel state file |

## 9. Global invariants

`INV-01 — P2 begins only after P1 is COMPLETE / IMPLEMENTED / FOCUSED_TESTED and walkthrough/handoff exist.`

`INV-02 — intrinsic is a first-class keyword and ImplementationKind::Intrinsic remains canonical.`

`INV-03 — @intrinsic is not a builtin source attribute after P2 and does not appear in current normative/source-like presentation.`

`INV-04 — NativeIntrinsicId is distinct internal metadata and is preserved.`

`INV-05 — intrinsic is implementation category/origin, not visibility/access authority.`

`INV-06 — intrinsic type members normalize in semantic/compiler code with zero VM dispatch.`

`INV-07 — @primitive is unaffected merely because @intrinsic is removed.`

`INV-08 — TDR history is preserved by amendment, not rewritten.`

`INV-09 — Type, RecordRow, TypePack, and LabeledTypePack remain distinct.`

`INV-10 — TypePack/LabeledTypePack are ordered; RecordRow identity is key-oriented/order-independent.`

`INV-11 — I: Int means Const(Int) in an admitted compile-time binder; Int is not a kind.`

`INV-12 — C1 does not imply general nominal/trait const generics.`

`INV-13 — type-member identity is declaration/domain identity, not spelling identity.`

`INV-14 — projection identity includes canonical member identity and declared result domain.`

`INV-15 — T[I] and T::At<I> are one canonical type-subscript member; At is a named presentation alias.`

`INV-16 — valid unresolved projections remain symbolic; Stuck != Invalid.`

`INV-17 — Dynamic is not a fallback for unresolved/invalid structural operations.`

`INV-18 — Tuple order is positional lane then labeled payload lane.`

`INV-19 — negative At and concrete bounds remain P1 semantics.`

`INV-20 — Map<F> requires F : Type -> Type and uses ordinary type application.`

`INV-21 — Tuple Map preserves arity, lane boundary, labels, label order, component order.`

`INV-22 — Record/RecordRow Map preserves keys and maps open symbolic tails.`

`INV-23 — TypePack/LabeledTypePack Map preserve their own identities/order.`

`INV-24 — structural members do not implicitly distribute over unions.`

`INV-25 — Tuple([],[]) is Unit; empty Record remains a distinct static Record type.`

`INV-26 — general type-member theory does not automatically expose ordinary user-defined inherent type members.`

`INV-27 — generalized projections do not imply GATs or parameterized trait-associated members.`

`INV-28 — no runtime TypePack/LabeledTypePack/RecordRow objects are introduced.`

`INV-29 — domain-native members do not make anonymous structural types ordinary inherent impl targets.`

`INV-30 — no type-member VM bytecode/runtime solver is introduced.`

`INV-31 — every C1 semantic fact has one primary normative owner; neighbors delegate.`

`INV-32 — normative specs describe semantics, not P1 Rust arena/ID layout.`

`INV-33 — parseable virtual source uses legal current syntax and never emits retired @intrinsic.`

`INV-34 — internal native-intrinsic metadata, if surfaced in non-source tooling, is clearly distinguished from source ImplementationKind::Intrinsic.`

`INV-35 — historical records outside active LANG007.C1 are not rewritten merely to remove old @intrinsic prose.`

## 10. Non-goals

Do not expand P2 into:

```text
P1 corrective implementation
general const/trait const generics
GATs
parameterized associated requirements
open user type families
arbitrary compile-time execution
const/pack arithmetic
pack split inference
variadic callable lowering
Symbol/Range type indexing
union-distributed At/Map
Map fusion
new structural operations
runtime pack/row models
VM type-member bytecodes
native primitive redesign
NativeIntrinsicId renaming/removal
general reflection/LSP redesign
historical docs migration
wiki/design cleanup
unrelated LANG006 corrective work
release certification
```

## 11. Expected impact map

### 11.1 Decision records

Create, if still next free:

```text
docs/decisions/accepted/0089-intrinsic-is-a-first-class-implementation-keyword.md
```

Modify:

```text
docs/decisions/README.md
```

Do not rewrite TDR-0082/TDR-0088 bodies.

### 11.2 Narrow production source cleanup

Expected:

```text
phalcom-ast/src/ast.rs
    remove BuiltinAttr::Intrinsic and builtin parse/name registration

phalcom-core/src/compiler/attributes.rs
    remove @intrinsic MetadataExpander / target handling

phalcom-semantic/src/core_surface/presentation.rs
    stop rendering @intrinsic(...)
    render ImplementationKind::Intrinsic as keyword when source-like
    preserve internal NativeIntrinsicId separately
```

### 11.3 Tests

```text
phalcom-ast/tests/parser.rs
phalcom-semantic/tests/semantic/integration/native_conformance.rs
```

Add a small builtin-attribute regression if existing parser tests cannot directly prove `BuiltinAttr::parse("intrinsic") == None`.

### 11.4 Normative specs

Expected audit/modify:

```text
docs/specs/README.md
docs/specs/source-declarations.md
docs/specs/type-level-domains.md
docs/specs/type-members.md
docs/specs/structural-products.md
docs/specs/generic-constraints.md
docs/specs/associated-types.md
docs/specs/objects/impl.md
docs/specs/objects/traits.md
docs/specs/universe.md
docs/specs/objects/values.md
docs/specs/vm/products.md
docs/specs/vm/values-and-objects.md
docs/specs/callables/INDEX.md
docs/specs/callables/dispatch.md
```

`callables/selectors.md` and `callables/references.md` are audit-only unless conflicting.

### 11.5 Current C1 records

Expected:

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1-GUIDANCE.md
```

Create at completion:

```text
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P2-walkthrough.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P2-handoff.md
```

### 11.6 Unexpected-touch rule

Mechanical exhaustive-match/test-helper edits are allowed. STOP AND CONSULT before modifying native intrinsic identity, P1 type-member representation, VM dispatch, conformance evidence, generic syntax, or unrelated historical implementation families.

## 12. Implementer decision authority

### 12.1 FIXED

Do not change:

1. intrinsic remains keyword.
2. `ImplementationKind::Intrinsic` remains.
3. `@intrinsic` builtin source attribute is removed.
4. internal `NativeIntrinsicId` remains separate.
5. P1 semantic architecture is inherited.
6. new amendment TDR preserves history.
7. one canonical subscript identity serves `[]` and `At`.
8. no general const generics.
9. no GATs.
10. pack/row domain separation.
11. open-row Map maps tail.
12. Map uses ordinary `Type -> Type` application.
13. no union distribution.
14. zero Tuple Unit / empty Record static distinction.
15. no runtime pack/row objects.
16. no structural inherent impl expansion.
17. no VM type-member execution.
18. specs remain representation-independent.
19. historical completed records remain historical.

### 12.2 MECHANICALLY FLEXIBLE

May adapt:

- implementation-keyword rendering helper;
- exact placement of builtin-attribute regression;
- spec wording/section order;
- non-source label for internal intrinsic ID (`intrinsic-id`, `compiler-intrinsic`, equivalent);
- exact `rg` patterns;
- private imports/helpers;
- final P1 mechanical path drift;
- exact TDR number if 0089 is consumed.

### 12.3 VERIFY-FIRST

| Assumption | Verify | If false |
|---|---|---|
| P1 complete/focused-tested | checkpoint + walkthrough/handoff | STOP |
| final P1 matches C1 decisions except intrinsic amendment | handoff/tests/code | STOP on material conflict |
| `ImplementationKind::Intrinsic` exists | modifier.rs | STOP if removed |
| `intrinsic` remains keyword | token/parser | STOP if removed |
| Tuple/Record use `intrinsic type` | live Universe | adapt move; STOP semantic revert |
| BuiltinAttr::Intrinsic remains | AST | skip if already removed |
| @intrinsic expander remains | compiler attrs | skip if already removed |
| virtual source emits @intrinsic | presentation/tests | preserve already-fixed result if not |
| NativeIntrinsicId has consumers | native-meta/surface/runtime | preserve |
| 0089 next free | decisions tree | use next free |
| registry omits 0083..0088 | README | update only missing |
| no general const generics in P1 | P1 evidence | STOP if scope changed |
| no GAT surface in P1 | P1 evidence | STOP if scope changed |
| associated-result boundary remains P1-bounded | P1 evidence | follow explicit accepted P1; otherwise narrow |
| no docs checker | CI/scripts | use focused target if present |
| P3 remains next | checkpoint | adapt lifecycle if amended |

## 13. Global STOP / CONSULT triggers

STOP if:

1. P1 is not fully complete.
2. final P1 architecture conflicts with accepted C1 theory.
3. keeping intrinsic keyword requires changing P1 semantic ownership.
4. removing BuiltinAttr::Intrinsic breaks another accepted source feature.
5. @intrinsic is required by a canonical non-presentation compiler/runtime contract.
6. removal appears to require deleting/changing NativeIntrinsicId semantics.
7. TDR number conflicts.
8. older TDRs cannot be cleanly amended without changing another accepted rule.
9. P1 deliberately added general const generics/GATs/non-Type associated source semantics beyond assumed scope.
10. P1 makes one subscript/At identity impossible without semantic change.
11. two specs claim incompatible primary ownership.
12. spec contradiction requires changing implementation semantics.
13. virtual source cannot remove @intrinsic without losing required identity and no structured channel exists.
14. a second implementation-kind/intrinsic source of truth would be introduced.
15. passing evidence would require weakening an invariant.
16. P2 requires broad compiler/runtime work beyond the expected cleanup.
17. the same nontrivial semantic/spec conflict survives one serious correction.
18. scope drifts into historical-doc mass cleanup.

## 14. Debugging budget

Mechanical failures: up to three coherent inspect/correct/retest cycles.

Semantic/spec conflict: record `Observed / Hypothesis / Evidence / Prediction / Discriminating search-test`; allow one serious correction, then STOP.

Architectural failure: zero speculative architecture attempts.

Never rerun unchanged failing evidence.

## 15. Testing surface analysis

| ID | Dimension | Invariants | Required evidence |
|---|---|---|---|
| CV-01 | keyword preservation | INV-02 | intrinsic parses to Intrinsic kind |
| CV-02 | attr retirement | INV-03 | `BuiltinAttr::parse("intrinsic")` no longer builtin |
| CV-03 | structural anchor | INV-02,06 | intrinsic type members remain accepted |
| CV-04 | source presentation | INV-03,33 | virtual source has no @intrinsic |
| CV-05 | internal metadata | INV-04 | native intrinsic expectations remain valid |
| CV-06 | presentation disambiguation | INV-04,34 | internal ID != source implementation kind |
| CV-07 | decision history | INV-08 | amendment TDR; old bodies historical |
| CV-08 | registry | INV-08 | accepted registry complete/count correct |
| CV-09 | domain separation | INV-09,10 | specs distinguish packs/row |
| CV-10 | const boundary | INV-11,12 | Const(Int), no general const-generic claim |
| CV-11 | member identity | INV-13..15 | one subscript/At identity |
| CV-12 | symbolic normalization | INV-16,17 | Reduced/Stuck/Invalid; no Dynamic fallback |
| CV-13 | indexing | INV-18,19 | negative/OOB/lanes specified |
| CV-14 | mapper kind | INV-20 | Map<Option> valid, Map<Int> invalid |
| CV-15 | Tuple Map | INV-21 | shape/lane/labels/order |
| CV-16 | Record/open-row Map | INV-22 | symbolic tail transformed |
| CV-17 | pack Map | INV-23 | domain order/labels retained |
| CV-18 | zero product | INV-25 | Unit vs empty Record static/runtime |
| CV-19 | union boundary | INV-24 | no distribution |
| CV-20 | associated/GAT boundary | INV-27 | no accidental GAT |
| CV-21 | inherent/impl boundary | INV-26,29 | no structural impl expansion |
| CV-22 | runtime boundary | INV-28,30 | no VM type-member execution |
| CV-23 | ownership | INV-31 | one primary spec owner |
| CV-24 | representation independence | INV-32 | no Rust layout as language law |
| CV-25 | deletion search | INV-03,35 | no active @intrinsic authority |
| CV-26 | lifecycle | INV-01 | checkpoint/walkthrough/handoff truthful |
| CV-27 | hygiene | all | scoped diff + diff-check |

## 16. Verification execution budget

### Modes

BUILD: exact searches, intrinsic parser tests, presentation test, narrow checks only.

STABILIZE: focused intrinsic tests, P1 type-member suite, affected crate checks, contradiction/deletion searches, `git diff --check`.

CERTIFY: workspace/release gates deferred unless explicitly required.

### Verification ladder

```text
T0 exact intrinsic regression
T1 focused AST/semantic feature tests
T2 owning crate checks
T3 narrow P1 type-member integration
T4 broad crate only if evidence demands
T5 workspace/release deferred
```

### Mandatory during BUILD after coherent code cleanup

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test parser intrinsic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::native_conformance::test_presentation_ir_and_virtual_source
```

Use `-- --list` once if exact semantic path drifted.

### Mandatory at G2

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test parser intrinsic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::native_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

Run serially.

### Mandatory final STABILIZE

Use final P1 handoff to select exact focused type-member suite; expected planning-revision form:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_members
git diff --check
```

Also rerun T5 deletion/contradiction searches.

### Only if evidence demands

Broader `phalcom-ast` or semantic suite only if narrow code cleanup shows cross-layer failure.

### Explicitly deferred / DO NOT RUN during BUILD

```text
workspace test
workspace clippy
full language corpus
full core runtime suite
all LSP tests
extension tests
performance suites
```

## 17. Baseline / unrelated failure policy

Classify:

```text
A definitely caused by P2
B probably caused by P2
C unclear
D unrelated/baseline
```

A/B fix; C one bounded classification pass then defer if nonblocking; D record/defer immediately. Do not weaken P1 tests or repair unrelated code to make broad suites green.


# 18. Tasks

## T0 — P1 Takeover, Authority Rebase, and P2 Activation

### Purpose

Establish the actual post-P1 repository state, prove that P2 is legally ready to start, consume P1's durable architecture rather than rediscovering it, and activate P2 in the C1 checkpoint.

### Preconditions

- `LANG007.C1.P1` is `COMPLETE`.
- P1 completion is `IMPLEMENTED`.
- P1 verification is at least `FOCUSED_TESTED`.
- P1 walkthrough exists.
- P1 handoff exists.
- The C1 checkpoint records the actual P1 final state.

### Consumes

```text
P1 final revision
P1 walkthrough
P1 handoff
C1 checkpoint/guidance
TDR-0081..0088
live normative specs
live intrinsic keyword implementation
```

### Produces

A verified P2 starting state:

```text
actual local HEAD
working-tree state
P1 completion proof
next accepted TDR number
live intrinsic keyword / @intrinsic census
final P1 semantic ownership map
P2 marked active in checkpoint
```

### Files and symbols

**Read:**

All required reads from §4.

**Modify:**

```text
docs/implementation/LANG007/LANG007.C1/LANG007.C1-CHECKPOINT.md
docs/implementation/LANG007/STATUS.md
```

Modify `PROGRAM.md` only if its current checkpoint/plan map is stale after P1 completion.

### Required implementation shape

1. Establish live state:

   ```sh
   git status --short
   git branch --show-current
   git rev-parse HEAD
   git log -5 --oneline
   ```

2. Read the final P1 walkthrough and handoff exactly once unless a specific ambiguity requires returning to them.

3. Confirm P1's actual focused acceptance evidence from the checkpoint/walkthrough. Do not rerun P1's whole acceptance matrix merely to start P2.

4. Verify the next accepted TDR number from the live `docs/decisions/accepted/` tree and decision registry.

5. Run narrow authority/census searches:

   ```sh
   rg -n '@intrinsic' \
     docs/specs \
     docs/decisions/accepted \
     phalcom-ast phalcom-common phalcom-semantic phalcom-core

   rg -n 'ImplementationKind::Intrinsic|BuiltinAttr::Intrinsic|intrinsic type' \
     phalcom-common phalcom-ast phalcom-semantic phalcom-core

   rg -n 'TypePack|LabeledTypePack|RecordRow|TypeMember|TupleFromPacks|RecordRowMap' \
     phalcom-semantic/src phalcom-type-meta/src
   ```

6. Build a short current-state matrix with only P2-relevant facts:
   - current keyword representation;
   - current source attribute representation;
   - current virtual-source rendering;
   - final P1 type-member owner;
   - final P1 projection owner;
   - final P1 result-domain representation;
   - final P1 const-term representation;
   - final P1 row-tail Map representation;
   - final P1 metadata/source target ownership.

7. Update the checkpoint plan ledger and mark P2:

   ```yaml
   status: IN_PROGRESS
   completion: PARTIAL
   verification: UNVERIFIED
   ```

8. Record the actual starting revision.

### Forbidden approaches

- no P1 implementation work;
- no speculative cleanup of incomplete P1 residue;
- no broad repository scan beyond named ownership seams;
- no re-running broad P1 suites;
- no reset, clean, restore, or broad formatting;
- no spec edits before the P1 completion lock is satisfied.

### Test changes required

None.

### Tests to run now

No Cargo tests.

### Tests explicitly deferred

All code verification until T2/G2.

### Acceptance

T0 passes only when:

```text
P1 is technically and administratively complete;
P2 has a trustworthy live starting revision;
unrelated working-tree state is preserved;
P2 is active in the checkpoint;
the current intrinsic/@intrinsic delta is enumerated;
the final P1 semantic owners are known;
the next accepted TDR number is verified.
```

### Local STOP / CONSULT triggers

- P1 is not complete.
- P1 walkthrough/handoff is missing or materially stale.
- Final P1 architecture contradicts the fixed P2 type theory.
- `ImplementationKind::Intrinsic` or the `intrinsic` keyword was removed/reverted.
- Final P1 deliberately expanded const generics, GATs, or other source semantics beyond the P2 assumptions in a way that changes normative scope.
- The checkpoint claims P1 success while the walkthrough records an unresolved blocking semantic failure.

### Checkpoint update

Mandatory at task completion: record the actual P2 start revision and inherited P1 state.

---

## T1 — Ratify the `intrinsic` Keyword and Repair the Accepted Decision Registry

### Purpose

Create one durable accepted decision for the user-ratified `intrinsic` keyword model and synchronize the accepted decision registry without rewriting historical TDR bodies.

### Preconditions

T0 PASS.

### Consumes

- explicit user ruling;
- TDR-0082;
- TDR-0088;
- completed P1 implementation;
- verified next free accepted TDR number.

### Produces

- one accepted amendment TDR;
- a synchronized accepted decision registry/count;
- one clear decision edge from the old `@intrinsic` model to the new `intrinsic` keyword model.

### Files and symbols

**Create:**

Expected if still free:

```text
docs/decisions/accepted/0089-intrinsic-is-a-first-class-implementation-keyword.md
```

**Modify:**

```text
docs/decisions/README.md
```

**Read only:**

```text
docs/decisions/accepted/0082-declaration-modifiers-and-identity-based-access-domains.md
docs/decisions/accepted/0088-native-type-operations-use-closed-compiler-normalization.md
```

### Required implementation shape

The new TDR must state at least:

```text
1. `intrinsic` is a lexical source keyword.
2. `intrinsic` maps to `ImplementationKind::Intrinsic`.
3. `@intrinsic` is retired as a builtin/source attribute.
4. `@primitive` is unaffected by this decision.
5. Intrinsic implementation kind is independent of visibility/access authority.
6. Target legality is declaration-category-specific.
7. LANG007 intrinsic type members are implemented by compiler/semantic
   normalization and never through VM/runtime dispatch.
8. `native` and `intrinsic` are distinct implementation kinds.
9. Internal `NativeIntrinsicId` metadata may remain attached to native surfaces;
   it is not the retired source attribute and does not imply
   `ImplementationKind::Intrinsic`.
10. Semantic behavior is keyed by canonical declaration/member identity rather
    than by presentation spelling.
11. Source-like reflection/presentation must not synthesize retired
    `@intrinsic(...)` syntax.
```

Decision relations:

```text
Amends:
    TDR-0082:
        the ruling that @intrinsic is the live metadata-only source form

    TDR-0088:
        the source-form/native-type-member wording using
        @intrinsic + native type

Preserves:
    TDR-0082:
        identity-based access and visibility semantics
        native/foreign distinction
        names do not imply implementation authority

    TDR-0088:
        closed compiler-side type normalization
        deterministic/side-effect-free calculus
        no VM type-member execution
```

The TDR should explain why the keyword architecture is coherent:

```text
implementation kind
    is declaration-level implementation provenance/category

internal intrinsic ID
    is optional implementation metadata for a compiler/runtime-recognized
    semantic operation

these are orthogonal facts even though both use the word "intrinsic"
```

The TDR should explicitly reject:

```text
reintroducing @intrinsic as an alias
treating intrinsic as visibility
treating intrinsic as native synonym
deriving ImplementationKind::Intrinsic from NativeIntrinsicId
using intrinsic spelling as canonical semantic identity
```

Update `docs/decisions/README.md` so that:

- TDR-0083 through TDR-0088 are listed if still missing;
- the new amendment TDR is listed;
- the accepted-count prose is correct;
- ordering follows ratification rules;
- every link resolves to the accepted file.

### Forbidden approaches

- do not rewrite TDR-0082/TDR-0088 bodies to erase history;
- do not move old decisions to retired status solely because one ruling is amended;
- do not claim `NativeIntrinsicId` is superseded;
- do not make `intrinsic` and `primitive` synonyms;
- do not broaden the record into a general compiler-directive feature;
- do not state that all declaration categories necessarily support intrinsic implementation.

### Test changes required

None.

### Tests to run now

No Cargo tests.

Focused decision checks:

```sh
rg -n 'TDR-008[3-9]' docs/decisions/README.md
rg -n 'Amends:|TDR-0082|TDR-0088|@intrinsic|ImplementationKind::Intrinsic' \
  docs/decisions/accepted/0089-*.md
```

Use the live accepted number if not `0089`.

### Tests explicitly deferred

Code tests to T2/G2.

### Acceptance

- new accepted TDR precisely records the user ruling;
- old accepted TDR bodies remain historical evidence;
- registry count/rows are synchronized;
- no new decision wording suggests `@intrinsic` remains a live source form;
- internal native intrinsic IDs are explicitly preserved as a separate concern.

### Local STOP / CONSULT triggers

- another accepted TDR lands at the planned number while P2 is in progress;
- registry rules require a materially different amendment/supersession relationship;
- defining `Intrinsic` as an implementation kind conflicts with another currently accepted declaration-origin rule not already covered by the amendment;
- the live parser accepts `intrinsic` in declaration categories whose semantics cannot be described without a new language decision.

### Checkpoint update

Record the new TDR ID and durable intrinsic-keyword amendment in the C1 consultation/amendment/invariant section.

---

# 19. Verification gates

## G1 — Decision Authority Gate

### Purpose

Prove P2 has one durable accepted authority for the intrinsic-keyword change before source/presentation/spec migration continues.

Run:

```sh
git diff -- docs/decisions
rg -n '@intrinsic|ImplementationKind::Intrinsic|intrinsic' docs/decisions/accepted/0089-*.md
rg -n '0083|0084|0085|0086|0087|0088|0089' docs/decisions/README.md
```

Use the live new TDR number if different.

Expected:

```text
one explicit amendment decision exists;
registry contains the complete accepted LANG007 decision sequence;
accepted count is correct;
TDR-0082/TDR-0088 historical bodies are not rewritten.
```

Do not broaden after PASS.

On failure, classify before editing.

---

## T2 — Retire the `@intrinsic` Builtin Attribute and Correct Source-Like Presentation

### Purpose

Make production source handling and source-like semantic presentation agree with the new keyword authority while preserving internal native intrinsic metadata.

### Preconditions

G1 PASS.

### Consumes

- new intrinsic-keyword TDR;
- `ImplementationKind::Intrinsic`;
- current `BuiltinAttr`;
- compiler attribute registry;
- core-surface presentation;
- `NativeIntrinsicId`.

### Produces

```text
no BuiltinAttr::Intrinsic
no compiler @intrinsic metadata expander
no @intrinsic(...) source-like rendering
intrinsic keyword preserved
ImplementationKind::Intrinsic preserved
internal NativeIntrinsicId preserved
focused regressions
```

### Files and symbols

**Read/modify:**

```text
phalcom-ast/src/ast.rs
    BuiltinAttr
    BuiltinAttr::ALL
    BuiltinAttr::name
    BuiltinAttr::parse

phalcom-core/src/compiler/attributes.rs
    builtin attribute expander registration
    builtin attribute target handling

phalcom-semantic/src/core_surface/presentation.rs
    MethodPresentation
    render_markdown
    render_virtual_source

phalcom-ast/tests/parser.rs
phalcom-semantic/tests/semantic/integration/native_conformance.rs
```

**Read only unless a mechanical compile consequence requires otherwise:**

```text
phalcom-common/src/modifier.rs
phalcom-native-meta/src/primitive.rs
phalcom-native-surface/src/lib.rs
phalcom-native-macros/src/lib.rs
phalcom-native-surface-gen/src/main.rs
```

### Required implementation shape

#### A. Remove builtin `@intrinsic`

Remove:

```text
BuiltinAttr::Intrinsic
BuiltinAttr::Intrinsic from BuiltinAttr::ALL
BuiltinAttr::Intrinsic name mapping
BuiltinAttr::parse("intrinsic") -> Builtin
compiler MetadataExpander for the intrinsic attribute
compiler target-special cases whose only purpose is @intrinsic metadata
```

After this task, `@intrinsic` must not receive builtin compiler semantics.

If arbitrary user-defined attributes are syntactically legal, `@intrinsic` may mechanically lex/parse as an ordinary user attribute name before normal user-attribute resolution. That is acceptable only if it receives no builtin privilege or special semantic association. Do not add a compatibility alias.

#### B. Preserve keyword architecture

Do not remove or weaken:

```text
TokenKind::Intrinsic
ImplementationKind::Intrinsic
intrinsic declaration modifier parsing
canonical Universe intrinsic type-member declarations
```

Do not translate intrinsic declarations back to `native`.

#### C. Render source implementation kind from the implementation-kind fact

`render_virtual_source()` or its final equivalent must render source implementation keywords from `ImplementationKind`, not from internal intrinsic metadata.

Use one local mapping equivalent to:

```text
Phalcom   -> ""
Native    -> "native "
Foreign   -> "foreign "
Intrinsic -> "intrinsic "
```

subject to the declaration category represented by that presentation IR.

Delete source-like:

```text
@intrinsic(BoolNot)
```

A native Bool primitive carrying:

```text
ImplementationKind::Native
NativeIntrinsicId::BoolNot
```

must remain source-presented as `native`, not be rewritten as `intrinsic`.

#### D. Keep internal native intrinsic metadata structured

`MethodPresentation.intrinsic` or its final equivalent may remain if tooling needs it, because it denotes internal `NativeIntrinsicId`.

For non-source display such as markdown badges, avoid a bare label that implies the source implementation kind.

Use a mechanically appropriate descriptive presentation such as:

```text
native · intrinsic-id:BoolNot · pure
```

or:

```text
native · compiler-intrinsic(BoolNot) · pure
```

The exact typography is flexible; the semantic separation is fixed.

#### E. Tests

Update/add focused coverage proving:

```text
intrinsic declaration keyword still works
intrinsic structural type members still parse
BuiltinAttr::parse("intrinsic") is no longer builtin
virtual source contains no @intrinsic(...)
native Bool internal NativeIntrinsicId still exists/validates
source implementation rendering follows ImplementationKind
```

A direct AST unit test is preferred for `BuiltinAttr::parse("intrinsic") == None` if parser behavior cannot isolate that fact cleanly.

### Forbidden approaches

- no removal of `ImplementationKind::Intrinsic`;
- no removal/rename of `NativeIntrinsicId`;
- no conversion of intrinsic type members to `native type`;
- no broad attribute-framework rewrite;
- no reflection redesign;
- no new replacement attribute such as `@compilerIntrinsic`;
- no inference of source implementation kind from internal native metadata;
- no string matching on `BoolNot`, `Map`, `At`, etc. to reconstruct source implementation kind.

### Test changes required

Cover:

```text
CV-01
CV-02
CV-03
CV-04
CV-05
CV-06
```

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test parser intrinsic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::native_conformance::test_presentation_ir_and_virtual_source
```

If the exact semantic filter drifted, inspect `-- --list` once and use the live path.

### Tests explicitly deferred

```text
full phalcom-ast suite
full semantic suite
full core runtime suite
workspace tests
workspace clippy
```

### Acceptance

T2 passes when:

```text
intrinsic keyword tests remain green;
@intrinsic no longer has builtin attribute authority;
parseable virtual source no longer renders @intrinsic;
internal NativeIntrinsicId evidence remains intact;
no broader compiler/runtime architecture changed.
```

### Local STOP / CONSULT triggers

- removing the attribute breaks canonical internal intrinsic-ID binding;
- a consumer requires parseable `@intrinsic` to recover canonical semantic identity;
- `ImplementationKind::Intrinsic` is not propagated through a required source-like presentation seam and fixing it would require broad semantic redesign;
- parser-supported intrinsic declaration categories expose semantics that cannot be locally classified;
- native-surface validation depends on the source attribute rather than internal metadata.

### Checkpoint update

After focused evidence passes, record `@intrinsic` source retirement and intrinsic keyword preservation as implemented durable facts.

---

## G2 — Intrinsic Source-Surface Gate

### Purpose

Prove the narrow production source/presentation migration before normative prose is converged.

Run serially:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test parser intrinsic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::native_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

Then:

```sh
rg -n 'BuiltinAttr::Intrinsic|@intrinsic\(' \
  phalcom-ast/src phalcom-core/src phalcom-semantic/src \
  phalcom-ast/tests phalcom-semantic/tests
```

Expected:

```text
no BuiltinAttr::Intrinsic;
no compiler @intrinsic builtin expander;
no source-like @intrinsic(...) renderer/test expectation;
ImplementationKind::Intrinsic remains;
TokenKind::Intrinsic remains;
NativeIntrinsicId remains in its internal metadata layer;
focused tests/checks pass.
```

Do not broaden after PASS.

On failure, classify before editing.

---

## T3 — Converge Declaration, Type-Member, and Universe Specifications on `intrinsic`

### Purpose

Replace the obsolete `native` + `@intrinsic` source model in current normative authority with the ratified intrinsic-keyword model, while preserving the distinction between source implementation kind and internal native intrinsic metadata.

### Preconditions

G2 PASS.

### Consumes

- new intrinsic-keyword amendment TDR;
- completed P1 type-member architecture;
- keyword/source cleanup from T2.

### Produces

One coherent declaration/type-member/Universe implementation-origin specification.

### Files and symbols

**Modify:**

```text
docs/specs/source-declarations.md
docs/specs/type-members.md
docs/specs/universe.md
docs/specs/README.md
```

**Audit/modify only if required:**

```text
docs/specs/callables/dispatch.md
docs/specs/callables/INDEX.md
docs/specs/callables/reflection.md
```

### Required implementation shape

#### `source-declarations.md`

Specify a four-way implementation category:

```text
ImplementationKind ::=
    Phalcom
  | Native
  | Foreign
  | Intrinsic
```

Define:

```text
intrinsic
    declaration implementation is supplied directly by canonical
    compiler/language intrinsic semantics through the mechanism appropriate
    to that declaration category
```

Preserve:

```text
Native != Intrinsic
Foreign != Intrinsic
ImplementationKind != Visibility
names/selectors do not imply implementation kind
```

Remove `@intrinsic` from builtin attributes.

Preserve `@primitive` only according to its independent existing metadata semantics.

State explicitly that `intrinsic` being a lexical modifier does not mean every declaration category necessarily has a meaningful intrinsic implementation contract. Target legality is defined by each owning declaration specification/compiler rule.

#### `type-members.md`

Use the final accepted source surface, expected:

```phalcom
intrinsic type Positionals: TypePack
intrinsic type Labeled: LabeledTypePack
intrinsic type At<I: Int>: Type
intrinsic type Map<F: Type -> Type>: Type
```

Describe intrinsic type members as the closed compiler/semantic normalization category.

They do not:

```text
create a runtime method
materialize runtime Tuple/Record values
send a message
enter a VM frame
invoke arbitrary source code
enter ordinary native-call dispatch
```

Remove live `@intrinsic` claims.

#### `universe.md`

Specify this ownership flow:

```text
canonical Universe source declaration
    owns authored source/provenance/navigation identity

ImplementationKind::Intrinsic
    records source declaration implementation category

canonical semantic member identity
    owns type-level operation meaning

compiler semantic normalizer
    supplies intrinsic type-member implementation

NativeIntrinsicId (where used)
    separate internal metadata for selected native surfaces
```

Native metadata must not become a second source declaration database.

#### `README.md`

Ensure the ownership table points to the current declaration/type-member/domain specs and does not preserve obsolete `@intrinsic` source semantics.

### Forbidden approaches

- no `native`/`intrinsic` synonymy;
- no replacement attribute for `@intrinsic`;
- no claim that all declaration forms permit intrinsic implementation;
- no VM/runtime implementation for intrinsic type members;
- no user-visible source syntax based on internal `NativeIntrinsicId`.

### Test changes required

None.

### Tests to run now

No Cargo tests.

Run:

```sh
rg -n '@intrinsic' \
  docs/specs/source-declarations.md \
  docs/specs/type-members.md \
  docs/specs/universe.md \
  docs/specs/README.md

rg -n 'intrinsic type|ImplementationKind|native|foreign' \
  docs/specs/source-declarations.md \
  docs/specs/type-members.md \
  docs/specs/universe.md
```

The first search must be empty.

### Tests explicitly deferred

All compiler/runtime feature tests.

### Acceptance

The current declaration/type-member authority can be understood without reading historical decisions and says unambiguously:

```text
intrinsic is keyword/implementation kind;
@intrinsic is retired;
intrinsic type-member implementation is compiler-semantic;
internal NativeIntrinsicId is separate.
```

### Local STOP / CONSULT triggers

- target legality for currently parsed intrinsic declarations cannot be specified without a new semantic decision;
- final P1 uses another canonical source spelling;
- Universe provenance cannot remain source-owned without promoting internal native metadata to source authority.

### Checkpoint update

Only if T3 establishes a durable clarification beyond the new TDR already recorded.

---

## T4 — Converge the Formal Type-Level and Structural Product Specification Graph

### Purpose

Make the authoritative C1 type theory mathematically complete, representation-independent, compositionally coherent, and exactly bounded to the fully implemented P1 feature.

### Preconditions

T3 accepted.

### Consumes

- completed P1 semantic products/tests;
- TDR-0083 through TDR-0088;
- new intrinsic amendment;
- current type-level/structural specs.

### Produces

A coherent normative graph for:

```text
type-level domains
compile-time const domains
type members
projection identity
At/subscript
Map
Tuple/Record static structure
generic/associated boundaries
structural impl boundary
zero-product/static-runtime boundary
```

### Files and symbols

**Primary modify:**

```text
docs/specs/type-level-domains.md
docs/specs/type-members.md
docs/specs/structural-products.md
docs/specs/generic-constraints.md
docs/specs/associated-types.md
docs/specs/objects/traits.md
docs/specs/objects/impl.md
```

**Consistency audit; modify only if contradictory or incomplete:**

```text
docs/specs/objects/values.md
docs/specs/vm/products.md
docs/specs/vm/values-and-objects.md
docs/specs/callables/INDEX.md
docs/specs/callables/dispatch.md
docs/specs/callables/selectors.md
docs/specs/callables/references.md
```

### Required implementation shape

#### A. Domain theory

One canonical grammar:

```text
κ ::= Type
    | RecordRow
    | TypePack
    | LabeledTypePack
    | (κ₁,...,κₙ) -> κ

ParameterDomain ::= Kind(κ) | Const(τ)
```

State:

```text
proper type
    has kind Type

Option
    has kind Type -> Type

Option<Int>
    has kind Type

TypePack
    ordered sequence of proper type forms

LabeledTypePack
    ordered sequence of unique (label, proper type) entries;
    label order is semantic

RecordRow
    key-oriented, order-independent-for-identity row;
    may be open
```

No runtime objecthood follows from kind/domain existence.

#### B. Const boundary

Clarify:

```phalcom
I: Int
```

means:

```text
Domain(I) = Const(Int)
```

only in a declaration form that admits a compile-time value parameter.

It does not mean:

```text
Kind(I) = Int
```

Do not use broad class/trait generic examples that imply general const generics.

Use:

```phalcom
intrinsic type At<I: Int>: Type
```

as the canonical C1 source example.

#### C. Member taxonomy versus source admission

Preserve a future-compatible semantic taxonomy, but distinguish it from admitted source forms.

Conceptually:

```text
semantic categories:
    inherent/declaration-owned
    associated/trait-owned
    domain-native/intrinsic

C1 newly requires:
    domain-native intrinsic structural members

C1 does not thereby expose:
    arbitrary user-defined inherent type members
    parameterized trait-associated members
```

#### D. Associated types

Preserve:

```text
trait ownership
conformance-supplied binding
associated declaration identity
evidence-based normalization
```

The C1 associated surface remains unparameterized.

Do not infer GAT support from generalized projection internals.

Unless final P1 explicitly implemented, tested, and accepted broader non-`Type` associated-result source syntax, keep the associated result-domain surface at its P1-supported boundary. If final P1 proves otherwise, STOP AND CONSULT rather than silently promoting it.

#### E. Canonical subscript/At identity

Fix current contradiction in favor of TDR-0086:

```text
canonical semantic member:
    type-level Subscript

source presentations:
    T[I]
    T::At<I>
```

`At` is a standardized named presentation alias, not another semantic member.

If Universe source authors `At<I>` for documentation/discovery, state that it anchors the canonical subscript member identity.

#### F. Indexing law

Define:

```text
linear(Tuple(P,L)) = P ++ eraseLabels(L)
```

For concrete arity `N`:

```text
j = i       if i >= 0
    N + i   if i < 0
```

valid iff:

```text
0 <= j < N
```

Concrete OOB is statically `Invalid`.

No Symbol or Range type-level indexing in C1.

#### G. Map algebra

Normatively include every law from §7.3.

Explicit symbolic forms:

```phalcom
P::Map<F>  // P : TypePack
L::Map<F>  // L : LabeledTypePack
R::Map<F>  // R : RecordRow
```

Open row tail must be transformed:

```text
{fields | R}::Map<F>
    =
{map(fields,F) | R::Map<F>}
```

Do not leave `R` unchanged.

Every payload transformation uses ordinary type application `F<T>`.

No automatic union distribution.

#### H. Normalization classes

Use:

```text
Reduced
Stuck
Invalid
```

as language-level classes.

Operational conditions such as cancellation, budget exhaustion, recursion/cycle machinery, or internal failure are implementation states and do not become source-language types.

Never approximate valid symbolic projections as `Dynamic`, `Never`, or an unrelated unknown.

#### I. Zero product

Preserve:

```text
Tuple([],[]) ≡ Unit ≡ ()
Record({}) ≠ Unit  // static type identity
```

Explain that VM representation may share zero-product realization.

#### J. Structural impl boundary

Domain-native members do not make:

```phalcom
impl (Int, String) {
    ...
}
```

legal.

Structural member coincidence does not create trait conformance.

### Forbidden approaches

- no Rust enum/arena/ID layouts in normative language definitions;
- no duplicate full definitions of one law in several specs;
- no speculative const-generic or GAT grammar;
- no open-type-family framing;
- no independent ad hoc descriptions of each Map domain;
- no `LabeledTypePack == RecordRow`;
- no static empty Record collapse to Unit;
- no union distribution;
- no runtime dispatch explanation for type-level operations.

### Test changes required

None.

### Tests to run now

No Cargo tests.

Use focused semantic searches and peer-owner comparisons only.

### Tests explicitly deferred

All runtime/compiler tests; P1 already owns implementation proof.

### Acceptance

A language implementer can derive every P1-observable type-level result from the normative specs without reading Rust source, while the specs do not imply unsupported const-generic/GAT/type-family features.

### Local STOP / CONSULT triggers

- P1 behavior cannot be described by the accepted domain calculus;
- associated-type source scope is materially broader than planned and already independently ratified;
- a structural algebra law conflicts with accepted TDRs or P1 evidence;
- VM/static specs disagree on empty Record semantics;
- defining source admission requires new syntax semantics not already accepted.

### Checkpoint update

Record only durable ownership/boundary clarifications.

---

## G3 — Core Specification Coherence Gate

### Purpose

Prove the central normative owners agree before repository-wide authority cleanup.

Run:

```sh
rg -n '@intrinsic' docs/specs

rg -n 'native type (Positionals|Labeled|At|Map)|intrinsic type (Positionals|Labeled|At|Map)' \
  docs/specs docs/decisions/accepted

rg -n 'canonical named member|presentation alias|T\[I\]|T::At' \
  docs/specs/type-members.md \
  docs/specs/structural-products.md \
  docs/decisions/accepted/0086-type-level-subscript-is-tuple-at.md

rg -n 'Const\(Int\)|I: Int|TypePack|LabeledTypePack|RecordRow' \
  docs/specs/type-level-domains.md docs/specs/generic-constraints.md

rg -n 'generic associated|parameterized associated|member-local parameters|result domain' \
  docs/specs/type-members.md docs/specs/associated-types.md docs/specs/objects/traits.md
```

Expected:

- zero live `@intrinsic` in normative specs;
- intrinsic type-member examples use keyword syntax;
- subscript/At wording agrees with TDR-0086;
- no general const-generic implication;
- no accidental GAT implication;
- domain ownership is singular and consistent.

On contradiction, diagnose before editing.

---

## T5 — Repository-Wide Authority, Deletion, and Deferred-Surface Audit

### Purpose

Prove P2 did not leave a contradictory active specification, obsolete source-attribute seam, duplicate semantic owner, or accidentally ratified deferred feature.

### Preconditions

G3 PASS.

### Consumes

All P2 edits.

### Produces

A complete contradiction/deletion/ownership matrix for the P2 walkthrough and P3 handoff.

### Files and symbols

No planned production edits unless the audit finds a contradiction in an expected current owner.

Historical implementation/design/wiki/archive records are evidence only and normally remain unchanged.

### Required implementation shape

Run and classify each search.

#### A. Active source attribute deletion

```sh
rg -n '@intrinsic' \
  docs/specs \
  phalcom-ast/src \
  phalcom-common/src \
  phalcom-core/src \
  phalcom-semantic/src
```

Expected current-authority/source result:

```text
no builtin/source-like @intrinsic usage
```

An internal comment that refers to the retired spelling only for a migration diagnostic is acceptable only if it clearly says it is retired. Prefer no active code references unless needed for an explicit migration error.

#### B. Keyword preservation

```sh
rg -n 'ImplementationKind::Intrinsic|TokenKind::Intrinsic|intrinsic type' \
  phalcom-common \
  phalcom-ast \
  phalcom-semantic \
  phalcom-core/core/universe
```

Expected: keyword and implementation kind remain.

#### C. Internal metadata preservation

```sh
rg -n 'NativeIntrinsicId|intrinsic = Bool' \
  phalcom-native-meta \
  phalcom-native-surface \
  phalcom-native-macros \
  phalcom-native-surface-gen \
  phalcom-core/src/primitive
```

Expected: internal native intrinsic metadata remains.

#### D. Decision registry

Verify:

```text
accepted count
TDR-0083..0088 rows
new amendment TDR row
ratification ordering
links
```

#### E. Domain ownership

```sh
rg -n 'TypePack|LabeledTypePack|RecordRow|Const\(' docs/specs
```

Classify substantive definitions:

```text
primary owner
delegating restatement
contradiction
```

Only contradictions or harmful duplication require edits.

#### F. At/subscript identity

```sh
rg -n 'At<I>|At<|type subscript|Subscript|presentation alias|canonical.*member' docs/specs
```

No active statement may imply two semantic members or reverse the canonical identity.

#### G. Map completeness

```sh
rg -n 'Map<F>|MapPack|MapLabeled|MapRow|open row|open tail|union' docs/specs
```

Verify explicit coverage for:

```text
TypePack
LabeledTypePack
Tuple
closed RecordRow
open RecordRow
Record
empty Tuple
empty Record
symbolic mapping
mapper kind
At/Map composition
Record lookup/Map composition
union non-distribution
```

#### H. Deferred-surface audit

```sh
rg -n 'const gener|GAT|generic associated|type famil|compile-time execution|runtime TypePack|structural impl' docs/specs
```

Every occurrence must either:

```text
define an explicitly admitted existing feature
or
state a deferral/non-goal/boundary
```

#### I. Runtime boundary

```sh
rg -n 'type-member.*VM|VM.*type-member|runtime.*Map|runtime.*At' docs/specs
```

Distinguish runtime Tuple collection/index behavior from type-level projection. No type-level VM execution is permitted.

#### J. Representation independence

Search for private P1 implementation names in the primary normative specs if needed:

```sh
rg -n 'TypeId|InferVarId|RecordRowVarId|TupleFromPacks|TypeMemberId|ProjectionId' \
  docs/specs/type-level-domains.md \
  docs/specs/type-members.md \
  docs/specs/structural-products.md
```

Implementation names may appear in clearly labeled non-normative implementation notes only if truly necessary. Prefer zero.

### Forbidden approaches

- no mass replacement across historical docs;
- no editing archive/wiki/design merely to make an unscoped search clean;
- no broad prose reformatting;
- no deleting internal intrinsic metadata because its name appears in the census;
- no converting historical implementation records into current authority.

### Test changes required

None.

### Tests to run now

All searches above plus:

```sh
git diff --check
```

If a repository markdown/link checker exists at P2 start, run its focused docs target only.

### Tests explicitly deferred

Cargo tests until final G4, except if an audit uncovers an actual code regression requiring a discriminator.

### Acceptance

Every active-authority result is classified; current specs are contradiction-free; the obsolete source attribute is gone from current authority/source-like output; deferred features remain deferred; historical references are explicitly recognized as history rather than opportunistically rewritten.

### Local STOP / CONSULT triggers

- two accepted TDRs remain in contradiction after the new amendment;
- a current spec requires a semantic change to P1;
- removing all live `@intrinsic` source semantics requires deleting internal native intrinsic metadata;
- a supposedly deferred feature is independently ratified elsewhere and changes C1's required boundary;
- a primary spec has to claim Rust representation details to state semantics.

### Checkpoint update

Record only residual/history classifications that P3 needs.

---

## T6 — Final Focused Verification, Lifecycle Closure, and P3 Handoff

### Purpose

Prove the narrow source cleanup and normative convergence, update durable C1 state, and leave P3 a compact takeover record requiring no semantic rediscovery.

### Preconditions

T5 PASS.

### Consumes

- all P2 edits;
- P1 final evidence;
- G1/G2/G3 evidence;
- T5 contradiction/deletion matrix.

### Produces

```text
P2 COMPLETE / IMPLEMENTED / FOCUSED_TESTED
updated C1 checkpoint
updated program/status/guidance
P2 walkthrough
P2 handoff
explicit P3 next action
```

### Files and symbols

**Modify:**

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1-GUIDANCE.md
```

**Create:**

```text
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P2-walkthrough.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P2-handoff.md
```

### Required implementation shape

#### A. Checkpoint

Record:

- actual P1 inherited completion;
- P2 plan status/completion/verification;
- new intrinsic-keyword amendment TDR;
- `@intrinsic` source retirement;
- stable distinction between source `Intrinsic` implementation kind and internal `NativeIntrinsicId`;
- final spec ownership map;
- focused verification commands/results;
- intentionally deferred broad verification;
- residual/baseline failures;
- consultations/amendments;
- next action (`LANG007.C1.P3` unless checkpoint changed).

Update stale C1 invariant text equivalent to:

```text
native implementation origin and @intrinsic semantic identity remain distinct
```

to:

```text
Native and Intrinsic are distinct ImplementationKind values.
@intrinsic is retired as a source/builtin attribute.
NativeIntrinsicId is separate internal operation metadata and does not determine
source ImplementationKind.
```

Do not rewrite P1 plan history.

#### B. Guidance

Update the do-not-redesign list so future agents do not resurrect `@intrinsic`, collapse Native/Intrinsic, or delete internal native intrinsic IDs.

#### C. Walkthrough

Include:

```text
actual starting/final revisions
new amendment TDR
registry repair
production @intrinsic retirement changes
preserved ImplementationKind::Intrinsic / NativeIntrinsicId distinction
final spec ownership graph
semantic contradictions corrected
deviations from plan
consultations
tests added/changed
tests actually run
searches actually run
tests deliberately deferred
residual historical references outside current authority
verification classification
next action
```

#### D. Handoff

For P3 include only:

```text
current revision
P1/P2 completion state
canonical TDR chain
intrinsic keyword model
source/internal intrinsic distinction
type-level domain owner map
type-member/projection owner map
At identity
Map algebra
zero-product rule
associated/const/GAT boundaries
stable source/test takeover map
deferred broad verification
known residual failures
must-read files
P3 objective
first commands
do-not-redesign list
```

### Forbidden approaches

- no `RELEASE_COMPLETE` claim without broad release evidence;
- no deletion of unrelated historical references;
- no P3 implementation work inside P2;
- no competing implementation-state document.

### Tests to run now

Final stabilization, serially:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test parser intrinsic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::native_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_members
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
git diff --check
```

Use final P1 handoff to substitute exact live semantic test filters if renamed.

Rerun the T5 deletion/contradiction searches once against the final diff.

### Tests explicitly deferred

```text
workspace test
workspace clippy
full language corpus
unrelated VM/core suites
LSP-wide suites
extension tests
performance benchmarks
```

### Acceptance

T6 passes only when every required coverage obligation has evidence and lifecycle records are current.

### Local STOP / CONSULT triggers

- a final P1 type-member focused regression fails semantically;
- the final authority census still finds a live contradiction whose repair changes language semantics;
- P3 cannot take over without reopening a P2 semantic decision.

### Checkpoint update

Mandatory final update.

---

## G4 — Final P2 Focused Acceptance Gate

### Purpose

Prove the actual P2 patch and normative corpus are coherent without pretending to release-certify the workspace.

Run the T6 focused commands and final T5 searches.

Required evidence map:

| Coverage | Evidence |
|---|---|
| `CV-01..CV-06` | parser/presentation/native intrinsic focused tests |
| `CV-07..CV-08` | amendment TDR + registry audit |
| `CV-09..CV-24` | normative ownership/contradiction matrix |
| `CV-25` | final `@intrinsic` deletion search |
| `CV-26` | checkpoint/walkthrough/handoff |
| `CV-27` | scoped diff review + `git diff --check` |

Do not broaden after PASS.

On failure, classify before editing.

---

# 20. Final focused acceptance

P2 may be marked:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

only when all conditions below hold.

## Decision authority

- [ ] intrinsic-keyword amendment TDR is accepted in the repository.
- [ ] accepted decision registry is synchronized through the new TDR.
- [ ] TDR-0083 through TDR-0088 are represented in the registry.
- [ ] historical TDR-0082/TDR-0088 bodies remain historical evidence.

## Source / presentation

- [ ] `ImplementationKind::Intrinsic` remains.
- [ ] `intrinsic` token/parser support remains.
- [ ] canonical Universe structural type members use `intrinsic`.
- [ ] `BuiltinAttr::Intrinsic` no longer exists.
- [ ] compiler builtin attribute expander for `@intrinsic` no longer exists.
- [ ] parseable virtual source emits no `@intrinsic`.
- [ ] internal `NativeIntrinsicId` remains intact.
- [ ] native/internal intrinsic metadata is not misrepresented as source implementation kind.

## Type theory

- [ ] kind/domain grammar is complete and singular.
- [ ] `Const(Int)` versus kind distinction is explicit.
- [ ] source admission does not imply general nominal/trait const generics.
- [ ] type-member categories, identity, parameter domains, and result domains are coherent.
- [ ] initial associated source boundary is explicit; no accidental GAT promotion.
- [ ] `T[I]` / `T::At<I>` identity matches TDR-0086.
- [ ] Reduced/Stuck/Invalid are distinct.
- [ ] Dynamic is not symbolic-projection fallback.
- [ ] valid abstract pack/row/member terms remain semantically meaningful.

## Structural completeness

- [ ] Tuple lane/label identity is explicit.
- [ ] Record key identity is explicit.
- [ ] negative `At` and concrete bounds are explicit.
- [ ] TypePack/LabeledTypePack/Tuple Map laws are explicit.
- [ ] closed/open RecordRow and Record Map laws are explicit.
- [ ] Map/At composition law is explicit.
- [ ] Record lookup/Map composition law is explicit.
- [ ] mapper kind requirement is explicit.
- [ ] symbolic pack/row Map is explicit.
- [ ] no implicit union distribution.
- [ ] zero Tuple is Unit.
- [ ] empty Record remains statically Record.
- [ ] VM zero-product optimization cannot erase static identity.

## Boundaries

- [ ] no runtime pack/row classes are implied.
- [ ] no anonymous structural inherent impl surface is implied.
- [ ] no VM type-member execution.
- [ ] no arbitrary user type-family system.
- [ ] no general const generics.
- [ ] no GATs.
- [ ] no historical-doc mass rewrite.
- [ ] internal native intrinsic metadata is not source syntax.

## Evidence

- [ ] focused parser tests pass.
- [ ] focused presentation/native-conformance tests pass.
- [ ] focused P1 type-member suite passes.
- [ ] affected crate checks pass.
- [ ] deletion/contradiction searches pass or are explicitly classified.
- [ ] `git diff --check` passes.
- [ ] checkpoint is current.
- [ ] walkthrough exists.
- [ ] handoff exists.

---

# 21. Performance/resource evidence

P2 adds no new runtime algorithmic hot path.

No benchmark suite is required.

P2 must preserve P1's semantic performance envelope:

```text
concrete Tuple At:
    O(1) after shape access

concrete TypePack/LabeledTypePack/Tuple/Record Map:
    O(n) in payload cardinality

symbolic projection formation/normalization:
    bounded and memoizable under the P1 architecture

type-member evaluation:
    zero VM execution
```

The `@intrinsic` retirement must not add per-call/runtime lookup.

Do not add performance ceremony to this primarily authority/presentation/specification patch.

---

# 22. Checkpoint bookkeeping

## At plan start

- read the C1 checkpoint;
- confirm final P1 entry;
- add/activate P2;
- record actual start revision;
- record inherited baseline changes if any.

## During plan

Update only for:

- acceptance of the intrinsic amendment TDR;
- G2 establishment of the source-keyword/attribute boundary;
- a durable specification-ownership clarification;
- consultation/amendment;
- meaningful deferred failure;
- coherent verification gate.

Do not log every prose edit or search.

## At plan completion

Record:

```text
P2 status
P2 completion
P2 verification
new decision ID
final intrinsic keyword invariant
final specification owner map
tests/searches actually run
broad gates deliberately deferred
residual failures
consultations/amendments
next action
```

The checkpoint remains the single current C1 state authority.

---

# 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P2-walkthrough.md
```

Required content:

1. plan identity and final state;
2. actual start/final revisions;
3. objective and outcome;
4. amendment TDR created;
5. registry correction;
6. production `@intrinsic` retirement changes;
7. preserved `ImplementationKind::Intrinsic` / `NativeIntrinsicId` distinction;
8. final normative specification ownership graph;
9. semantic contradictions corrected;
10. deliberate deviations from plan;
11. consultations/adopted decisions;
12. tests added or changed;
13. tests actually run;
14. authority/deletion searches actually run;
15. tests deliberately deferred;
16. residual historical/baseline references;
17. verification classification;
18. next action.

Never claim a test/search gate passed without completed evidence.

---

# 24. Handoff deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P2-handoff.md
```

Keep it compact. It must contain:

```text
current revision
P1/P2 completion state
canonical TDR chain
intrinsic keyword model
source implementation kind vs NativeIntrinsicId distinction
type-level domain owner map
type-member/projection owner map
canonical At/subscript identity
Map algebra
zero-product rule
associated/const/GAT boundaries
stable source/test takeover map
deferred broad verification
known residual failures
must-read files
P3 objective
first commands
do-not-redesign list
```

Do not duplicate this whole plan.

---

# 25. Completion truth table

Do not conflate:

```text
new TDR written
decision registry updated
@intrinsic builtin source attribute removed
intrinsic keyword preserved
virtual source corrected
normative prose updated
contradiction/deletion searches pass
focused parser tests pass
focused semantic tests pass
affected crates check
P2 lifecycle complete
C1 checkpoint complete
release certified
```

P2 completing does not imply workspace release certification.

If P3 remains part of C1, P2 completion also does not imply C1 completion.

---

# 26. Git / commit / push discipline

The planning request does not itself authorize commits or pushes.

At execution time:

- if commits/pushes are not explicitly authorized, do not commit or push;
- if they are authorized, use coherent milestones rather than edit-by-edit commits.

Recommended coherent commit boundaries if authorized:

```text
1. decision:
   intrinsic-keyword amendment TDR
   + accepted registry synchronization
   + checkpoint amendment entry

2. source surface:
   remove @intrinsic builtin/presentation support
   + focused tests
   + checkpoint G2 evidence

3. specification convergence:
   primary/neighbor normative specs
   + checkpoint ownership updates

4. plan completion:
   final checkpoint/status/guidance
   + P2 walkthrough
   + P2 handoff
```

Combine adjacent milestones if the actual diff is small and one combined commit is more coherent.

Before every authorized commit:

```sh
git status --short
git diff -- <scoped paths>
```

Then:

- exclude unrelated work;
- make the checkpoint reflect the state represented by the commit;
- run only the focused gate appropriate to that milestone;
- stage only the coherent work unit.

Do not:

```text
reset
clean
restore unrelated work
amend
rebase
squash
force-push
--no-verify
change branches unexpectedly
create a new branch without instruction
```

Do not push a known-broken intermediate state.

---

# 27. Plan self-review

Before delivery, this plan was checked against the following.

## Architecture

- one canonical owner exists for source implementation kind;
- source `intrinsic` and internal `NativeIntrinsicId` are explicitly separated;
- old TDR history is preserved through amendment rather than rewrite;
- one owner exists for each type-level domain;
- one owner exists for general type-member semantics;
- associated evidence remains separately trait-owned;
- structural algebra is independent from VM representation;
- no duplicate canonical resolver/identity database is planned;
- P1 semantic architecture is inherited, not redesigned.

## Luna executability

- P1 completion is an objective hard precondition;
- fixed/flexible/verify-first decisions are explicit;
- exact current paths are named;
- the production code cleanup is narrow;
- semantic/spec ownership is explicit;
- STOP/CONSULT triggers are objective;
- debugging loops are bounded;
- historical cleanup is explicitly forbidden;
- broad tests are explicitly deferred.

## Mathematical and type-theoretic completeness

The plan covers:

```text
kind/domain grammar
proper type vs higher-kinded type form
Const(τ)
ordered TypePack
ordered LabeledTypePack
unordered/open RecordRow
Tuple/Record formation
canonical type-member identity
projection subject/member/args/result domain
member-local parameter domains
Reduced/Stuck/Invalid
negative Tuple indexing
canonical subscript/At identity
complete Tuple linear component order
Type -> Type Map requirement
TypePack/LabeledTypePack/Tuple/RecordRow/Record Map
open-row tail transformation
Map/At composition
Record lookup/Map composition
Unit/empty Record distinction
union non-distribution
Dynamic boundary
associated/GAT boundary
const-generic boundary
inherent type-member/source-admission boundary
structural impl boundary
stable symbolic terms
runtime/VM boundary
intrinsic keyword/internal metadata distinction
```

## Testing

- code tests are tied to the narrow production-source change;
- specification verification uses contradiction/deletion searches;
- final P1 type-member regression protects against migration breakage;
- broad Cargo gates are not required by reflex;
- prose-only tasks do not trigger compiler testing;
- baseline failure policy is explicit.

## Lifecycle

- checkpoint activation is mandatory;
- durable amendment is recorded;
- checkpoint is updated only at meaningful events;
- walkthrough/handoff are mandatory;
- P3 receives a compact takeover record;
- no competing implementation-state file is introduced.

## Remaining VERIFY-FIRST facts

Execution must still verify only:

```text
P1 final revision/evidence
exact final P1 paths
next free accepted TDR number
exact intrinsic declaration target matrix
exact semantic test filters
whether a docs/link checker now exists
whether P3 remains C1's next action
```

None of these gives Luna authority to redesign semantics.

---

# 28. Compact implementer directive

```text
Do not start P2 until P1 is fully complete and its walkthrough/handoff exist.

Keep `intrinsic` as the source keyword and fourth ImplementationKind.

Ratify that direction with a new amendment TDR. Preserve TDR-0082/TDR-0088
as historical evidence; do not rewrite them to pretend @intrinsic never existed.

Remove the obsolete builtin/source-like @intrinsic attribute and virtual-source
rendering, but preserve internal NativeIntrinsicId metadata used by native
compiler/runtime operation metadata.

Then make the normative corpus exactly describe completed P1:
distinct Type/RecordRow/TypePack/LabeledTypePack domains, Const(Int),
canonical member identity, one Subscript/At identity, symbolic projections,
complete structural Map including open rows, Unit/empty-Record distinction,
no union distribution, no Dynamic escape hatch, no general const generics,
no GATs, no structural impl expansion, and no VM type-member execution.

Use focused parser/presentation/type-member tests and targeted
contradiction/deletion searches. Do not release-test the workspace.

Update the C1 checkpoint, create the P2 walkthrough/handoff, and leave P3 a
semantically closed decision/specification corpus that requires no redesign.
```
