---
id: LANG007.C3.P5
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - LANG007.C3.P4
  - LANG007.C3.P4.R1
follows: LANG007.C3.P4.R1
supersedes: null
repository: aureat/phalcom-lang
prepared: 2026-10-01
prepared_against_revision: 6dcdcec08e55c2eb67eb60fff959cfb71fc3e939
title: "Type-System Coherence, Structural Semantics, Dynamic Generic Applications, and Iterator/Conformance Closure"
---

# LANG007.C3.P5 — Type-System Coherence, Structural Semantics, Dynamic Generic Applications, and Iterator/Conformance Closure

## 0. Executor contract

This is a **Luna patch-grade implementation plan** for GPT-5.6 Luna High/Extra
High execution under architectural supervision. It is deliberately explicit
about semantic ownership and deliberately narrow about testing.

The implementer:

- may adapt private helper names, local decomposition, and mechanical signature
  drift to the live repository;
- must preserve the architecture, ownership boundaries, and numbered
  invariants in this plan;
- must not improvise material type-system architecture;
- must not use local special cases to make P5 acceptance tests pass;
- must follow the BUILD / STABILIZE / CERTIFY verification budget below rather
  than running broad tests after every edit;
- must classify unrelated failures instead of absorbing them into P5;
- must STOP AND CONSULT when any global or task-local escalation trigger fires;
- must keep `LANG007.C3-CHECKPOINT.md` current at durable gates;
- must produce the required P5 walkthrough and handoff;
- must preserve user-owned or unrelated worktree changes;
- must not rewrite predecessor history to make P5 appear cleaner than it is.

Testing is evidence gathering. A test is run because it discriminates between
specific hypotheses or proves a named gate, not merely because source changed.

### 0.1 Architectural rulings adopted by this plan

The requirements analysis left several decisions open. This implementation plan
closes them as follows.

**RULING P5-A — closed structural products are exact.**

A closed structural Tuple or Record describes exactly its declared lanes or
fields. Width acceptance is available only through explicit openness.

```phalcom
{ a: Int, b: String } </: { a: Int }
{ a: Int, b: String } <:  { a: Int, ... }
```

**RULING P5-B — optionality is member presence, not value optionality.**

```phalcom
{ name?: String }
```

means `name` may be absent; if present it has type `String`. It is not
equivalent to a required `name: Option<String>` field.

**RULING P5-C — captured rows and anonymous remainders are distinct.**

`**R` retains correlated row identity. Anonymous `...` / `**: T` / tuple-lane
remainder syntax admits additional structure but introduces no reusable row or
pack identity.

**RULING P5-D — anonymous structural remainder state is canonical shape data.**

Record anonymous remainder state is represented in the canonical Record-row
shape, but as an explicit non-correlated remainder category distinct from
`Parameter` and `Projection`. Tuple anonymous remainder state is represented in
the canonical Tuple shape, independently for positional and labeled lanes.
Anonymous openness is not represented by inventing a row/pack variable.

**RULING P5-E — Dynamic remains outside the proper-Type term domain, but an
applied proper type may contain an explicit Dynamic argument slot.**

`Dynamic` itself does not become `TypeData::Dynamic` and does not receive a
synthetic `TypeId`. Instead, the heterogeneous generic/static argument envelope
gains a canonical **Dynamic type-argument marker** that is legal only for a
generic parameter whose kind is `Type`.

Therefore:

```text
Dynamic              -> TypeKnowledge::Dynamic
List<Dynamic>        -> proper applied List type whose argument slot is Dynamic
List<Any>            -> ordinary applied type with Any TypeId argument
List<Int>            -> ordinary applied type with Int TypeId argument
unresolved List<X>   -> unresolved/Unknown; never converted to Dynamic
```

The applied `List<Dynamic>` node is a canonical formal type shape, but any
operation whose proof/result depends on that dynamic slot must retain a Dynamic
boundary rather than manufacturing static proof.

**RULING P5-F — durable type forms preserve Dynamic argument slots.**

If `List<Dynamic>` occurs in an exported callable/member contract, metadata and
runtime/reflection type descriptions preserve the Dynamic slot. The metadata
schema is extended additively where possible; older exact forms remain
decodable. Dynamic-reason/provenance is not part of canonical type identity.

**RULING P5-G — use-site specialization consumes a knowledge-bearing argument
environment.**

Existing exact `TypeSubstitution` remains the canonical fast path for exact
TypeId bindings. P5 introduces or extends a use-site materialization path able
to bind a type parameter to either:

```text
Known(TypeId)
Dynamic
```

When a type expression is materialized:

- exact parameter binding produces a `TypeId`;
- a bare parameter bound to Dynamic produces `TypeKnowledge::Dynamic`;
- an applied type containing such a parameter produces a canonical applied
  `TypeId` containing a Dynamic argument slot;
- a fixed result independent of Dynamic remains Known.

This knowledge-bearing path must not become a second declaration signature
authority.

**RULING P5-H — the current Record row solver remains the owner of correlated
rows.**

Anonymous remainder constraints extend structural shape semantics but do not
gain row-variable identity, lack sets, or solver correlation.

**RULING P5-I — the current witness-specialization asymmetry is VERIFY-FIRST.**

The plan does not assume the inferred-return omission is the sole cause of the
20 conformance diagnostics. T7 must first prove the causal path. If confirmed,
the fix is use-site materialization of `inferred_return`; compatibility rules
remain strict.

**RULING P5-J — rest/module-export runtime code is not modified until semantic
specialization and a non-empty evidence recipe are proven.**

---

## 1. Goal

Complete LANG007.C3's remaining type-system closure by reconciling structural
Tuple/Record semantics, representing explicit Dynamic inside generic
applications without making Dynamic a proper type, repairing generic witness
and Iterator source contracts, restoring the natural Iterator/Iterable chain,
and closing the remaining conformance-evidence forwarding seam without
introducing duplicate semantic authorities or compiler/library special cases.

---

## 2. Checkpoint acceptance objective

P5 is a **corrective closure and semantic-coherence plan** for LANG007.C3.

It is intended to make C3 closable, but P5 itself may only mark C3 complete
after its final broad gate proves the checkpoint's required acceptance surface.
The plan does not assume that all historical semantic regressions are P5-owned.

P5 advances C3 by closing four gaps that currently prevent credible
type-theoretic completion:

1. one normative and implemented structural product relation;
2. coherent nested-Dynamic generic formation and specialization;
3. well-formed generic witness/Iterator contracts with diagnostic-clean
   Universe bootstrap;
4. end-to-end conformance evidence through natural collection chains and
   forwarding boundaries.

---

## 3. Repository grounding

Prepared against:

```text
repository:          aureat/phalcom-lang
branch:              main
revision:            6dcdcec08e55c2eb67eb60fff959cfb71fc3e939
relevant predecessor:
  LANG007.C3.P4
  LANG007.C3.P4.R1
checkpoint state:
  IN_PROGRESS / PARTIAL / UNVERIFIED
```

Verified planning facts at that revision:

- P4.R1 T1–T5 are implemented/focused-tested; T6/G5 is blocked.
- P4.R1 preserves one canonical `CallableSemanticSignature` result authority.
- current T6 records 20 `ImplConformanceIncomplete` and 3
  `type.generic.underconstrained` Universe bootstrap diagnostics;
- `List<Dynamic>` currently returns whole-form
  `TypeFormResolution::Dynamic` before `apply_type_form`;
- `TypeData::Applied` carries `StaticArgument::{Term, RecordRow}`;
- metadata `StaticArgumentRef` and runtime `RuntimeStaticArgument` carry the
  same two categories;
- current AST supports captured Record row tails (`**R`) and tuple pack
  expansion, but not optional structural member presence or anonymous typed /
  unconstrained remainders;
- current ordinary subtype entry is correctly restricted to kind `Type`;
- current relation still contains misleading pack relation arms behind that
  proper-Type gate;
- current Record relation implements target-required-field width behavior;
- `TraitSurfaceMember::instantiate` materializes canonical inferred return;
- `specialize_witness_signature` was audited as materializing declared
  parameters/return and constraints without visibly materializing
  `inferred_return`;
- `witness_signature_return_type` consumes canonical inferred return when the
  declaration has no explicit result;
- current `iterator_laws.ph` contains both missing `I |= Iterator` contracts
  and untyped mapper contracts that leave result generic `U` underconstrained;
- Chain/FlatMap/Flatten/StrictZip/Indexed Item semantics already have accepted
  predecessor corrections;
- the remaining rest/module-export probe reaches no callable or call
  specialization, so it does not yet prove runtime evidence loss.

> **Re-read the named live paths before editing. Adapt mechanical drift locally.
> Treat architectural drift as an escalation condition.**

If `main` has advanced from the pinned revision, first diff the relevant paths.
Mechanical drift may be adapted. A change to any ruling/owner/invariant in this
plan requires STOP AND CONSULT.

---

## 4. Required reads before implementation

Read these before editing production code:

```text
AGENTS.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P4-walkthrough.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P4-handoff.md
docs/implementation/LANG007/LANG007.C3/
  LANG007.C3.P4.R1-canonical-inferred-result-contracts.md
docs/implementation/LANG007/LANG007.C3/INC-C3-21.md
docs/implementation/LANG007/LANG007.C3/INC-C3-22.md
docs/implementation/LANG007/LANG007.C3/INC-C3-24.md
docs/implementation/LANG007/LANG007.C3/INC-C3-25.md
docs/implementation/LANG007/LANG007.C3/INC-C3-26.md
docs/implementation/LANG007/LANG007.C3/INC-C3-27.md
docs/implementation/LANG007/LANG007.C3/
  LANG007-C3-completion-audit-5f872da0.md
docs/implementation/LANG007/LANG007.C3/
  LANG007.C3.P4-audit-remediation/
    03-nested-derived-conformance-runtime-layout.md
    04-semantic-regression-classification.md

docs/specs/README.md
docs/specs/structural-products.md
docs/specs/tuple-and-record-typing.md
docs/specs/generic-constraints.md
docs/specs/objects/data-objects.md

phalcom-ast/src/ast.rs
phalcom-semantic/src/types/row.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/metadata/export.rs
phalcom-type-meta/src/type_node.rs
phalcom-core/src/typing/overlay.rs
phalcom-core/core/universe/src/collections/iterator_laws.ph
phalcom-core/core/universe/src/collections/range.ph

phalcom-semantic/tests/semantic/foundations/type_annotations.rs
phalcom-semantic/tests/semantic/advanced/record_rows.rs
phalcom-core/tests/core/language/iteration.rs
phalcom-core/tests/core/language/traits.rs
phalcom-core/tests/core/language/generic_conformance.rs
```

Read exact test-module wiring/readmes for any new test file before creating it.
Avoid broad repository exploration unless one of these paths proves a plan
premise false.

---

## 5. Normative authority

### 5.1 Normative behavior adopted by P5

| Authority | P5 dependency |
|---|---|
| `docs/specs/README.md` | one semantic concept has one primary owner; conflicting current specs must be reconciled |
| `docs/specs/structural-products.md` | Tuple/Record formation, pack/row domains, closed structural operations |
| `docs/specs/tuple-and-record-typing.md` | source model for explicit openness, remainders, optional labeled members; P5 reconciles it into the primary owner |
| `docs/specs/generic-constraints.md` | `<:` applies to proper Type terms; associated projections consume declared evidence; generic bodies are checked under declared constraints |
| `docs/specs/objects/data-objects.md` | exact applied type identity, runtime-observable type information, Dynamic boundary does not erase already-established exact type |
| adopted INC-C3-21/22/24/25 rulings | canonical inferred-result authority, conformance refresh, bounded SCC barrier, no general result fixpoint |
| P5 rulings in §0.1 | exact closed structural products; presence semantics; anonymous remainder identity rules; Dynamic applied-argument semantics; durability and use-site materialization |

### 5.2 Current implementation is evidence, not authority

The following are implementation facts P5 is explicitly allowed to change:

- closed Record width relation;
- lack of optional presence;
- lack of anonymous structural remainder representation;
- `List<Dynamic>` whole-form Dynamic collapse;
- static-argument envelope without Dynamic marker;
- candidate witness specialization omitting inferred result if T7 confirms;
- underconstrained Iterator adapter source declarations.

### 5.3 Future/non-normative proposals not adopted here

P5 does **not** adopt:

- general gradual typing beyond explicit Dynamic boundaries;
- implicit existential type syntax;
- generalized pack subtyping;
- deep immutability;
- arbitrary higher-kinded dynamic arguments;
- a new row-polymorphism language beyond the already-supported correlated row
  and the anonymous remainder forms required here;
- runtime trait proof search;
- full redesign of `group`/`toMap` collection APIs.

---

## 6. Takeover state

### 6.1 Stable predecessor interfaces

| Concept | Current symbol/path | Owner | P5 invariant |
|---|---|---|---|
| canonical callable contract | `CallableSemanticSignature` / `signature.rs` | semantic declaration analysis | one result authority |
| inferred callable result | `CallableSemanticSignature::inferred_return` | provider analysis | caller cannot mutate provider result |
| trait requirement instantiation | `TraitSurfaceMember::instantiate` / `traits.rs` | semantic trait surface | use-site specialization only |
| witness compatibility | `check_witness_compatibility*` / `impls.rs` | semantic conformance | strict parameter/result/generic compatibility |
| exact conformance evidence | `ConformanceEvidence` / `impls.rs` | semantic conformance index | runtime consumes selected evidence |
| derived evidence lowering | `TraitDispatchSite::Evidenced` → core invocation spec | semantic lowering/core | no proof search in compiler/VM |
| proper subtype boundary | `check_subtype_impl` / `relation.rs` | semantic type relation | only kind `Type` participates after identity fast path |
| correlated Record row | `RecordRowData`, `RecordRowTail::{Parameter,Projection}` | semantic row domain | preserves row identity/correlation |
| generic applied type | `TypeData::Applied` | semantic TypeStore | canonical head + heterogeneous static arguments |
| metadata applied type | `TypeNode::Applied` | `phalcom-type-meta` | durable structural type identity |
| runtime applied type | `RuntimeOverlayTypeNode::Applied` | core typing | reflection/runtime type shape |
| Iterator associated Item | `trait Iterator { type Item }` | Universe source + semantic conformance | adapter result semantics derive from contract |
| Iterator=>Iterable law | generic `impl<I> Iterable for I where I |= Iterator` | Universe source/conformance | no adapter-specific derived rule |
| nested evidence slot layout | P4 audit remediation | compiler/core projection | canonical generic evidence slot keys |
| SCC result barrier | P4.R1 T5 | semantic incremental | one-shot, no fixpoint |

### 6.2 Stable accepted tests/results to retain

P5 must preserve the predecessor semantics covered by:

```text
trait_default_inferred_return_is_canonical_and_independent_of_caller_expectation

branch_derived_tail_type_is_published_to_unannotated_callable_signature

semantic::incremental::callable_dependencies::
  inferred_return_refresh_preserves_stable_products_and_counts_final_disposition_once

semantic::incremental::product_stability::
  inferred_cross_module_result_updates_dependents_and_matches_cold_analysis

semantic::integration::metadata::
  cross_module_inferred_callable_result_round_trips_as_compiler_inferred_metadata

iterator_adapter_fixture_preserves_item_projections_and_chain_equality

language::traits::
  ordinary_trait_lowering_preserves_exact_and_derived_dispatch_evidence
```

Historical pass evidence is not a substitute for post-P5 verification where P5
touches the same path.

---

## 7. Architecture

P5 has four canonical semantic flows.

### 7.1 Structural type flow

```text
source structural annotation
    ↓
AST:
  fixed members/elements
  presence
  captured expansions
  anonymous remainder syntax
    ↓
semantic formation:
  canonical fixed fields/elements
  correlated row/pack expansions
  anonymous remainder constraints
    ↓
TypeStore canonical shape
    ↓
normalization / relation / projections
    ↓
metadata + presentation when durable
```

Closed exactness and openness are semantic shape facts. Relation consumes them;
relation does not infer openness from field-count mismatch.

### 7.2 Dynamic generic application flow

```text
source: List<Dynamic>
    ↓
annotation formation
    ↓
origin = List TypeId
argument = StaticArgument::DynamicType
    ↓
TypeData::Applied {
  origin: List,
  arguments: [DynamicType]
}
    ↓
known outer type form
    ↓
knowledge-bearing use-site specialization
      ├─ fixed result independent of T -> Known(TypeId)
      ├─ bare T result              -> Dynamic
      └─ nested F<T> result         -> canonical F<Dynamic> TypeId
```

The Dynamic argument marker is not itself a TypeId and is valid only in a
Type-kind argument slot.

### 7.3 Callable/conformance flow

```text
provider body
    ↓
canonical CallableSemanticSignature
    ↓
conformance plan selects candidate
    ↓
use-site specialization of complete callable contract
    ↓
strict witness compatibility
    ↓
ConformanceEvidence
```

P5 may improve specialization but may not change result ownership.

### 7.4 Runtime evidence flow

```text
semantic ConformanceEvidence
    ↓
stable lowering fact / recipe
    ↓
compiler projection
    ↓
bytecode/runtime environment
    ↓
VM invocation
```

Rest/module-export forwarding may carry an existing environment; it may not
reconstruct proof.

---

## 8. Ownership boundaries

### 8.1 Owns

P5 owns:

- effective structural Tuple/Record exact/open/optional/remainder semantics;
- AST/canonical representation necessary for those semantics;
- structural relation and normalization changes;
- Dynamic generic argument representation and all affected use-site
  specialization;
- metadata/runtime type-shape persistence needed by durable new forms;
- generic witness inferred-result specialization if T7 confirms the hypothesis;
- Iterator adapter source contract well-formedness;
- mapper result generic inference;
- P5-causal Universe bootstrap diagnostics;
- natural Range→TakeIterator→Iterable→toList chain;
- actual rest/module-export conformance transport only after semantic recipe
  creation is proven;
- P5-causal residual regressions;
- checkpoint/walkthrough/handoff for this plan.

### 8.2 Does not own

P5 does not own:

- redesign of nominal class inheritance;
- family width/member semantics;
- new general subtype relations for packs/static domains;
- a general gradual type calculus;
- mutation/deep immutability semantics;
- new generic defaulting;
- unrelated flow/field/golden regressions without causal evidence;
- runtime trait search;
- wholesale collection API redesign;
- performance work unrelated to P5 paths;
- unrelated formatting debt except files touched by P5.

### 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| structural closed/open/presence/remainder shape | TypeStore structural representation | relation, projections, metadata, tooling | parser-only flags or relation-local reconstruction |
| correlated Record row identity | `RecordRowData` + row solver | inference, projections, relation | anonymous fake row variable |
| explicit Dynamic generic argument | applied-type static argument envelope | formation, specialization, metadata/runtime type shape | `TypeData::Dynamic`, Dynamic→Any rewrite |
| callable inferred result | `CallableSemanticSignature` | calls, conformance, metadata, incremental | witness-local or trait-local result cache |
| use-site argument knowledge | specialization/materialization environment | call/witness/member specialization | mutating provider signature |
| conformance proof | semantic `ConformanceEvidence` | lowering/compiler/runtime | VM resolver |
| Iterator Item | associated type contract | adapter methods, derived Iterable | method-name/type special case |
| bootstrap diagnostic disposition | checkpoint ledger | planner/certification | silent ignore list |

---

## 9. Global invariants

```text
INV-01 — closed structural Tuple/Record types are exact.

INV-02 — openness is explicit semantic shape data; field/element surplus alone
         never implies openness.

INV-03 — optionality is member presence and is distinct from Option<T>,
         T|Unit, Dynamic, and field omission in a closed exact type.

INV-04 — captured rows/packs preserve correlation; anonymous remainders have no
         solver identity.

INV-05 — Record anonymous remainder and Tuple positional/labeled remainder
         constraints survive normalization and canonicalization.

INV-06 — Ellipsis value semantics and structural-remainder syntax are
         contextually distinct; structural ... is not a hidden pack value.

INV-07 — only kind Type participates in ordinary value subtyping after identity;
         packs/rows/static-domain terms do not acquire ordinary <:.

INV-08 — Dynamic itself is not a proper TypeId.

INV-09 — a proper applied type may contain StaticArgument::DynamicType only in
         a Type-kind parameter position.

INV-10 — Dynamic, List<Dynamic>, List<Any>, List<Int>, and unresolved List<X>
         remain distinguishable.

INV-11 — substituting a Dynamic type argument into a bare parameter produces a
         Dynamic knowledge result; substituting it under an applied constructor
         produces that constructor with a Dynamic argument slot.

INV-12 — a generic result independent of a Dynamic argument remains statically
         Known.

INV-13 — generic relation/conformance cannot manufacture proof from a Dynamic
         argument; it preserves a Dynamic boundary when proof depends on it.

INV-14 — canonical callable inferred results remain provider-owned.

INV-15 — use-site witness specialization may materialize inferred_return but
         never re-infers or mutates the provider contract.

INV-16 — no general recursive inferred-result fixpoint is introduced.

INV-17 — associated projection syntax consumes declared evidence; it does not
         implicitly add a conformance bound.

INV-18 — every Iterator adapter body/Item projection is valid under its declared
         source generic contract.

INV-19 — result generic U is inferred from typed callable contracts, never
         defaulted.

INV-20 — accepted Chain/FlatMap/Flatten/StrictZip/Indexed Item semantics remain
         unchanged.

INV-21 — toMap advances the underlying iterator exactly once per logical entry.

INV-22 — the natural Range chain succeeds through ordinary type/conformance
         machinery with no Range/List/Take/toList special case.

INV-23 — semantic analysis selects conformance evidence; compiler/runtime only
         projects/transports it.

INV-24 — rest/module-export runtime code is changed only after a non-empty
         semantic/lowering evidence recipe is demonstrated.

INV-25 — metadata/incremental/tooling preserve every durable P5 distinction.

INV-26 — fingerprints depend on canonical semantic content, not source spelling
         or allocation/hash iteration.

INV-27 — broad unrelated failures do not expand P5 scope without causal evidence.

INV-28 — a test failing before its focused assertion is BLOCKED/FAIL, never
         acceptance evidence.
```

Tasks and coverage IDs below reference these invariants.

---

## 10. Non-goals

P5 does not:

- introduce a general structural intersection/union row calculus;
- introduce optional positional tuple members;
- add implicit existential generic arguments;
- give Dynamic a nominal/proper type identity;
- make Any stand in for Dynamic;
- add pack subtyping;
- redesign callable syntax;
- redesign Option/Result;
- redesign Set/Map constraints beyond source typing necessary for P5;
- redesign the Iterator API unrelated to well-formedness/precision;
- optimize runtime collection iteration;
- replace current conformance evidence architecture;
- solve all Priority-3 regressions;
- certify a release before focused semantic gates pass.

---

## 11. Expected impact map

### 11.1 Expected source areas

- `docs/specs/README.md` — structural authority map/cross-reference cleanup.
- `docs/specs/structural-products.md` — normative exact/open/presence/remainder
  semantics.
- `docs/specs/tuple-and-record-typing.md` — subordinate/synchronized syntax
  specification or de-duplication.
- `phalcom-ast/src/ast.rs` — optional presence and anonymous remainder AST.
- parser sources under `phalcom-ast` — parse new structural forms.
- `phalcom-semantic/src/types/row.rs` — Record presence/remainder.
- `phalcom-semantic/src/types/store.rs` — Tuple shape and Dynamic static argument.
- `phalcom-semantic/src/types/annotation.rs` — lowering of structural shapes and
  Dynamic generic arguments.
- `phalcom-semantic/src/types/relation.rs` — exact/open structural relation;
  Dynamic applied-argument relation; dead pack relation cleanup.
- `phalcom-semantic/src/types/environment.rs`,
  `specialization.rs`, or nearest existing owner — knowledge-bearing use-site
  type argument/materialization path.
- `phalcom-semantic/src/traits.rs` / `impls.rs` — witness specialization and
  conformance integration.
- `phalcom-type-meta/src/type_node.rs`,
  semantic metadata export/import/validation — durable new shape categories.
- `phalcom-core/src/typing/overlay.rs` and nearest metadata materializer —
  runtime type-shape preservation.
- `phalcom-core/core/universe/src/collections/iterator_laws.ph` — adapter
  bounds/callable contracts and precise terminal types.
- `phalcom-core/core/universe/src/collections/range.ph` — only if existing
  `RangeIterator::toList` source remains an intentional specialized helper;
  no compiler special case.
- semantic source-index/presentation modules if new syntax/types require them.
- compiler/core lowering/runtime forwarding only if T11 proves recipe loss.

### 11.2 Expected tests

- `phalcom-ast/tests/parser.rs`
- `phalcom-semantic/tests/semantic/foundations/type_annotations.rs`
- `phalcom-semantic/tests/semantic/foundations/type_model.rs`
- `phalcom-semantic/tests/semantic/advanced/record_rows.rs`
- new or existing structural relation law module under
  `phalcom-semantic/tests/semantic/foundations/`
- semantic metadata/incremental suites
- `phalcom-core/tests/core/language/iteration.rs`
- `phalcom-core/tests/core/language/traits.rs`
- `phalcom-core/tests/core/language/generic_conformance.rs`
- affected collection corpus fixtures.

### 11.3 Expected documentation/state

- `LANG007.C3-CHECKPOINT.md`
- P5 plan ledger entry if repository uses one
- `LANG007.C3.P5-walkthrough.md`
- `LANG007.C3.P5-handoff.md`
- reconciled normative specs.

### 11.4 Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem if doing so
would change semantic ownership, representation, runtime identity, incremental
authority, or language surface beyond the P5 rulings.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

- INV-01 through INV-28;
- the one canonical callable result authority;
- exact-closed structural semantics;
- first-class optional presence;
- distinction between captured and anonymous remainder;
- Dynamic remaining outside the TypeId term domain;
- use of a Dynamic argument marker rather than Dynamic→Any or
  `TypeData::Dynamic`;
- strict witness compatibility;
- explicit Iterator evidence requirements;
- no generic `U` defaulting;
- semantic proof authority / runtime transport boundary;
- P4.R1 bounded SCC architecture;
- final natural chain as an acceptance oracle.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- helper/function/type names for new private APIs;
- whether a small private enum is nested or file-level;
- whether closed Tuple internal storage is migrated immediately or through a
  helper wrapper, provided canonical semantics are singular;
- exact parser helper decomposition;
- exact metadata additive variant names;
- exact test file placement if the repository's live module organization has a
  clearer established owner;
- private conversion helpers between exact substitution and knowledge-bearing
  materialization;
- mechanical signature changes caused by live repository drift.

### 12.3 VERIFY-FIRST

| Assumption | Verify at | If false |
|---|---|---|
| `main` still contains P4.R1 T1–T5 architecture | checkpoint + listed source | adapt mechanical drift; consult if architecture changed |
| `structural-products.md` is still primary owner | specs README | STOP if authority changed materially |
| current Record relation remains width-based | `relation.rs` + focused test | adjust T5 baseline; do not force obsolete patch |
| optional/remainder AST still absent | AST/parser | reuse existing representation if semantically equivalent |
| Dynamic generic argument still collapses whole form | `annotation.rs` | use live canonical path; STOP if Dynamic already became proper TypeId |
| metadata static args still Term/RecordRow only | type meta + importer/exporter | adapt additive work; consult if schema already has different canonical representation |
| witness candidate inferred result remains unmaterialized | `impls.rs` | T7 must re-localize; do not patch assumption |
| 20+3 bootstrap diagnostics still reproduce/classify similarly | focused Universe preflight | create new ledger; STOP only if cause changed architecture |
| rest probe still lacks call specialization | semantic/core probe | if recipe now exists, continue directly to transport localization |

---

## 13. Global STOP / CONSULT triggers

Stop editing and prepare a consultation packet if any of these occurs:

1. a P5 ruling is impossible without changing the public language model;
2. a core premise in §3/§12.3 is false in a way that changes semantic ownership;
3. optional/remainder support requires treating anonymous openness as a
   correlated row/pack variable;
4. nested Dynamic appears to require a proper Dynamic `TypeId`;
5. Dynamic generic arguments require a List/Map/Range-specific compiler branch;
6. knowledge-bearing specialization would require mutating canonical provider
   signatures per call/conformance;
7. witness compatibility can pass only by weakening covariance,
   contravariance, generic constraints, visibility, rest role, or dispatch side;
8. associated projection legality can pass only by implicit evidence synthesis;
9. mapper `U` can pass only through defaulting;
10. the same nontrivial semantic failure persists after one serious correction;
11. structural metadata cannot be made backward-validating without a breaking
    schema/loader policy not described here;
12. runtime evidence forwarding must be changed before a semantic recipe exists;
13. VM trait/conformance search appears necessary;
14. an invariant must be weakened to pass a test;
15. two fixes have materially different architecture;
16. a materially different subsystem becomes required;
17. a workaround/special-case architecture is emerging;
18. broad certification exposes a soundness failure whose cause is outside this
    plan but blocks C3 closure;
19. current normative specs conflict with the P5 rulings after T2 reconciliation;
20. scope becomes materially more cross-cutting than this plan.

A triggered consultation cannot be self-waived.

---

## 14. Debugging budget

### 14.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress.

One cycle is:

```text
inspect
identify concrete cause
make one coherent correction
rerun the smallest discriminating command
```

Examples: parser token/range bug, missing enum match, metadata serde compile
error, stale test import.

### 14.2 Semantic failures

Before changing production code record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic failure.

If the same failure persists: STOP AND CONSULT.

Do not stack speculative fixes.

### 14.3 Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

Never rerun an unchanged failing test unless relevant source/test/fixture state
changed.

---

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| CV-01 | closed structural negative | INV-01/02 | `{a:A,b:B} </: {a:A}` |
| CV-02 | explicit open structural success | INV-01/02 | `{a:A,b:B} <: {a:A,...}` |
| CV-03 | optional absence | INV-03 | closed source omits optional target member |
| CV-04 | optional presence type | INV-03 | present optional field compatible/incompatible |
| CV-05 | captured vs anonymous | INV-04 | `**R` canonical identity differs from `...` |
| CV-06 | typed remainder | INV-05 | extra field checked against remainder type |
| CV-07 | tuple lane independence | INV-05 | open positional lane does not open labeled lane and vice versa |
| CV-08 | structural canonicalization | INV-01–06 | equivalent syntax canonicalizes; ordering laws preserved |
| CV-09 | proper-Type boundary | INV-07 | unequal pack/static terms rejected as ordinary subtype operands |
| CV-10 | nested Dynamic formation | INV-08–10 | `List<Dynamic>` keeps List head and differs from Dynamic/Any |
| CV-11 | Dynamic argument kind safety | INV-09 | Dynamic marker rejected for non-Type kind parameter |
| CV-12 | dependent Dynamic result | INV-11/13 | `T -> T`, T=Dynamic yields Dynamic |
| CV-13 | nested dependent result | INV-11 | `T -> Box<T>`, T=Dynamic yields `Box<Dynamic>` |
| CV-14 | fixed result | INV-12 | generic result Int remains Known under Dynamic premise |
| CV-15 | metadata round trip | INV-10/25/26 | durable `List<Dynamic>` and structural presence/remainders survive export/import |
| CV-16 | witness specialization | INV-14/15 | generic inherent inferred result materializes at exact receiver |
| CV-17 | witness negative | INV-15 | incompatible inferred result still fails |
| CV-18 | projection legality | INV-17/18 | missing `I |= Iterator` is rejected; repaired source is valid |
| CV-19 | mapper U inference | INV-18/19 | map/filterMap/mapWhile infer U from callable return |
| CV-20 | predecessor adapter semantics | INV-20 | Chain/FlatMap/Flatten/Zip/Indexed fixture remains green |
| CV-21 | toMap iteration | INV-21 | no double advance |
| CV-22 | natural Range chain | INV-22/23 | `(0..).iter.take(10).toList` full semantic/lowering/runtime chain |
| CV-23 | bootstrap diagnostic closure | INV-17–22/28 | prior 20+3 ledger dispositioned |
| CV-24 | rest evidence recipe | INV-23/24 | source produces non-empty specialization evidence recipe |
| CV-25 | rest/module export transport | INV-23/24 | lowering/runtime preserve established environment |
| CV-26 | incremental parity | INV-25/26 | cold/import/incremental products agree |
| CV-27 | result lifecycle regression | INV-14/16/26 | P4.R1 inferred-result selectors stay green |
| CV-28 | failure classification | INV-27/28 | unrelated Priority-3/baseline failures preserved, not laundered |

No GC/concurrency dimensions are added because P5 does not create new runtime
object lifetime or concurrent state.

---

## 16. Verification execution budget

Use:

```text
RUSTUP_TOOLCHAIN=nightly-2026-07-10
RUSTFLAGS=''
RUSTC_WRAPPER=''
```

where predecessor commands required that environment.

### 16.1 Modes

#### BUILD MODE

Run only exact/new discriminators required by the active task. Prefer one test
or one narrowly filtered module.

#### STABILIZE MODE

Run directly affected semantic/core modules and predecessor regression controls
after a coherent work package is complete.

#### CERTIFY MODE

Run broad crate/workspace gates only at T14/G12.

### 16.2 Verification ladder

```text
T0 — exact reproducer/new regression
T1 — directly affected feature tests
T2 — owning subsystem/module suite
T3 — adjacent cross-layer integration
T4 — broad crate/package verification
T5 — workspace/release verification
```

### 16.3 Mandatory during BUILD

- parser exact selectors while changing syntax;
- direct structural relation law selectors while changing relation;
- direct nested-Dynamic formation/materialization selectors;
- direct witness reproducer;
- exact Universe adapter source-contract selectors;
- exact natural chain only after bootstrap gate is clean;
- exact evidence recipe probe before any VM forwarding edit.

### 16.4 Mandatory at named gates

- G1 structural authority/representation;
- G2 structural laws;
- G3 nested Dynamic formation/materialization;
- G4 witness specialization;
- G5 adapter source contracts;
- G6 Universe bootstrap;
- G7 natural chain;
- G9 metadata/incremental/tooling;
- G10 evidence forwarding;
- G11 focused regression matrix;
- G12 broad closure.

### 16.5 Mandatory during STABILIZE

After T9/T10/T11, run:

- affected `phalcom-semantic` module filters;
- affected `phalcom-core --test core` language filters;
- metadata/incremental filters when those products changed;
- `corpus::collections` only after focused natural-chain success.

### 16.6 Only if evidence demands

- full parser suite before parser gate if focused syntax changes interact with
  recovery or snapshots;
- full metadata crate if schema additions affect shared serializer;
- full core library if a runtime forwarding helper changes common dispatch;
- Priority-3 focused selector only after a P5 causal trace implicates it.

### 16.7 Explicitly deferred / DO NOT RUN during BUILD

Do not run during ordinary task iteration:

```text
cargo test --workspace
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
broad unrelated language corpora
full semantic suite after every task
```

Reason: the checkpoint already has classified unrelated/baseline failures and
historical broad-gate noise. Broad verification is meaningful only after P5's
focused semantic products are coherent.

---

## 17. Baseline / unrelated failure policy

Classify encountered failures:

```text
A — definitely caused by P5
B — probably caused by P5
C — unclear
D — clearly unrelated/baseline
```

Policy:

- A/B: active responsibility.
- C: one bounded classification pass. If still unclear and nonblocking, record
  in checkpoint and continue.
- D: record/retain existing classification and continue immediately.
- Do not spend adviser compute on D unless it blocks a named P5 acceptance gate.
- Do not weaken/skip assertions to make broad gates green.
- A focused P5 test that stops at an unrelated global precondition is BLOCKED,
  not PASS.
- Preserve the existing P4 Priority-3 classification unless new evidence
  changes causality.

---

# 18. Tasks

## T1 — Pin live baseline and activate P5 state

### Purpose

Establish an execution-grade baseline from the live repository and convert the
requirements analysis into a durable active-plan checkpoint state without
editing production semantics.

### Preconditions

- P4.R1 T1–T5 remain accepted/focused-tested.
- No later commit has already superseded P5's architecture.
- Worktree/user changes are inventoried and protected.

### Consumes

- pinned planning revision;
- current checkpoint;
- P4/P4.R1 walkthrough/handoff/incidents;
- existing bootstrap diagnostic evidence.

### Produces

- live starting SHA;
- worktree preservation note;
- current 20+3 or replacement diagnostic ledger;
- current state of structural relation, Dynamic formation, witness
  specialization, Iterator source contracts, and rest probe;
- checkpoint entry marking P5 active.

### Files and symbols

**Read:**

```text
LANG007.C3-CHECKPOINT.md
P4/R1 plan/walkthrough/handoff/incidents
relation.rs::check_subtype_impl/check_record_row_subtype
annotation.rs::resolve_type_form
store.rs::TypeData/StaticArgument
traits.rs::TraitSurfaceMember::instantiate
impls.rs::specialize_witness_signature/witness_signature_return_type
iterator_laws.ph
generic_conformance.rs rest probe
```

**Modify:**

```text
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
```

**Create:** none.

**Tests/commands:**

Run only the smallest existing preflight commands necessary to reproduce the
active T6 blockers and capture diagnostic identities/counts. Prefer the exact
existing T6 selectors/check command already recorded in the checkpoint.

### Required implementation shape

Record the baseline as a table:

```text
Finding
Live symbol/path
Observed state
Matches plan premise? yes/no
Action
```

For the 20+3 bootstrap diagnostics, record source location and diagnostic code;
do not fix yet.

### Forbidden approaches

- no production source edits;
- no spec edits;
- no broad workspace tests;
- no reclassification without new evidence.

### Test changes required

None.

### Tests to run now

Only the current T6 semantic precondition/selector necessary to verify that the
blocker is still live.

### Tests explicitly deferred

All new P5 tests; workspace gates.

### Acceptance

T1 passes when every VERIFY-FIRST assumption in §12.3 is marked true or
mechanically updated, and no architectural drift is unresolved.

### Local STOP / CONSULT triggers

- canonical inferred-result authority changed;
- Dynamic already became a proper TypeId;
- structural spec owner materially changed;
- current blocker moved to a materially different subsystem.

### Checkpoint update

Yes. Record P5 ACTIVE, live SHA, diagnostic ledger, and next action T2.

---

## T2 — Reconcile and ratify structural Tuple/Record normative semantics

### Purpose

Make the structural product specification singular and executable before any
relation or representation patch.

### Preconditions

- T1 baseline accepted.
- No production structural edit has begun.

### Consumes

- P5-A through P5-D;
- specs authority rules;
- current structural-products and tuple-and-record documents.

### Produces

One normative model covering:

```text
closed exactness
explicit Record openness
Tuple positional/labeled lane openness
required/optional labeled presence
typed/unconstrained anonymous remainders
captured row/pack distinction
canonical ordering/equivalence
subtype laws
```

### Files and symbols

**Read:**

```text
docs/specs/README.md
docs/specs/structural-products.md
docs/specs/tuple-and-record-typing.md
docs/specs/generic-constraints.md
```

**Modify:**

```text
docs/specs/README.md
docs/specs/structural-products.md
docs/specs/tuple-and-record-typing.md
```

**Create:** none unless the repository's spec ownership convention requires a
new dedicated syntax-only page; do not create one by default.

### Required implementation shape

`structural-products.md` remains the primary semantic owner.

`tuple-and-record-typing.md` becomes either:

- a syntax/examples companion that explicitly defers semantics to
  `structural-products.md`; or
- a synchronized subordinate document with no independent conflicting law.

The normative model must state at least:

```text
Record closed target:
  exact declared key set modulo optional absence

Record open target:
  fixed members + anonymous/captured remainder rule

optional member:
  may be absent
  if present must satisfy declared type

Tuple:
  positionals and labeled lanes have separate closed/open state

captured remainder:
  exact correlated remainder identity

anonymous typed remainder:
  any additional member/element must satisfy constraint

anonymous unconstrained remainder:
  additional structure admitted without a common static type proof
```

Also state that ordinary `<:` is only between proper Type terms.

### Forbidden approaches

- no “implementation currently does X, therefore X is normative” reasoning;
- no second structural semantic owner;
- no unresolved TODO wording for exact/open behavior;
- no production code edits.

### Test changes required

None; tests follow the ratified text in T3/T5.

### Tests to run now

None. Documentation-only task.

### Tests explicitly deferred

All parser/semantic tests.

### Acceptance

A reviewer can answer every structural law in CV-01..CV-09 from one effective
normative owner without reconciling contradictory files.

### Local STOP / CONSULT triggers

- user-ratified syntax conflicts with P5-A..D;
- existing accepted ADR/PDR makes closed width subtyping normative;
- optional/remainder syntax has changed materially since audit.

### Checkpoint update

Yes. Record the ratified structural law and effective spec owner.

---

## T3 — Add structural presence and anonymous remainder source/canonical representation

### Purpose

Represent every T2-ratified structural shape without fabricating row/pack
identity.

### Preconditions

- T2 normative model accepted.

### Consumes

- exact/open/presence/remainder laws;
- existing captured Record row and pack-expansion AST;
- existing `RecordRowData` row solver semantics.

### Produces

- AST for optional labeled members;
- AST for anonymous Record and Tuple remainders;
- canonical member presence enum;
- canonical Record anonymous remainder state;
- canonical Tuple positional/labeled remainder state;
- formation validation for typed remainder proper types;
- source index/presentation traversal for new AST.

### Files and symbols

**Read:**

```text
phalcom-ast/src/ast.rs
parser modules that build TypeAnnotationExpr::Record/TypeTupleItem
phalcom-semantic/src/types/row.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/presentation.rs
```

**Modify:**

same paths as required.

**Create:**

Prefer no new production module unless representation complexity justifies a
small `types/structural_shape.rs`; if created, it must remain a semantic
representation helper, not a second row solver.

**Tests:**

```text
phalcom-ast/tests/parser.rs
phalcom-semantic/tests/semantic/foundations/type_annotations.rs
```

### Required implementation shape

#### Record field presence

Introduce a semantic enum equivalent to:

```rust
enum StructuralMemberPresence {
    Required,
    Optional,
}
```

and add it to canonical Record fields.

Default existing source fields to `Required`.

#### Record remainder

Extend canonical Record shape/tail with a non-correlated anonymous remainder,
for example:

```rust
enum RecordRowTail {
    Closed,
    Parameter(TypeParameterId),
    Projection(TypeId),
    Anonymous(AnonymousRecordRemainder),
}

enum AnonymousRecordRemainder {
    Unconstrained,
    Typed(TypeId),
}
```

Exact names may differ.

Rules:

- `Typed(TypeId)` requires kind `Type`;
- anonymous remainder has no `TypeParameterId`;
- row inference/lacks logic must branch explicitly and never bind/solve the
  anonymous case as a row variable.

#### Tuple shape

Evolve canonical exact Tuple representation to carry lane remainder state.
Equivalent conceptual shape:

```rust
struct TupleType {
    elements: Box<[TupleTypeElement]>,
    positional_remainder: AnonymousTupleRemainder,
    labeled_remainder: AnonymousTupleRemainder,
}

enum AnonymousTupleRemainder {
    Closed,
    Unconstrained,
    Typed(TypeId),
}
```

Existing closed tuples canonicalize with both remainder lanes `Closed`.
Existing symbolic pack-driven `TupleFromPacks` remains distinct.

Optional presence is legal only on labeled tuple members if that is what T2
ratified; parser must reject optional positional members.

#### AST

Do not reuse `TypePackExpansionSyntax` for anonymous remainder if doing so would
manufacture an operand. Add an explicit anonymous remainder AST category or
enum case.

### Forbidden approaches

- no fake `R`/`Ps` variable for anonymous openness;
- no Option encoding for presence;
- no `...` → Any/Dynamic;
- no loss of existing captured row/pack syntax;
- no relation implementation yet beyond compilation-required exhaustive
  matches.

### Test changes required

Add parser/formation tests for:

- required/optional Record field;
- optional labeled Tuple member;
- optional positional rejection;
- Record `...`;
- Record typed remainder;
- Tuple positional/labeled/complete anonymous remainder;
- captured `**R` remains unchanged;
- duplicate remainder/misordered syntax diagnostics;
- typed remainder rejects non-Type kind operand.

### Tests to run now

Focused parser selectors for each new syntax family and focused
`type_annotations` formation tests only.

### Tests explicitly deferred

Structural subtype laws; metadata; natural chain; workspace gates.

### Acceptance

Every T2 structural source form has exactly one AST and canonical semantic
shape. Captured and anonymous remainder forms are structurally distinguishable.

### Local STOP / CONSULT triggers

- parser syntax is materially different from T2;
- one canonical representation would require changing row correlation
  semantics;
- tuple anonymous remainder cannot be represented without treating it as an
  exact TypePack value.

### Checkpoint update

Yes. Record the durable canonical structural representation.

---

## T4 — Add canonical Dynamic generic argument slots and knowledge-bearing materialization

### Purpose

Make `List<Dynamic>` and equivalent source forms retain a known generic head
without making Dynamic a TypeId.

### Preconditions

- T1 baseline confirms the early whole-form Dynamic collapse.
- P5-E through P5-G remain viable.

### Consumes

- `TypeData::Applied`;
- `StaticArgument`;
- generic application kind metadata;
- existing exact substitution/materialization APIs;
- TypeKnowledge Dynamic semantics.

### Produces

- canonical `StaticArgument::DynamicType` or equivalent;
- application formation that accepts Dynamic only for `Type`-kind parameters;
- `TypeData::Applied` with dynamic slots;
- knowledge-bearing use-site argument environment/materializer;
- direct presentation/formatting for nested Dynamic;
- no changes to bare Dynamic TypeKnowledge ownership.

### Files and symbols

**Read:**

```text
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/application.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/specialization.rs
phalcom-semantic/src/types/view/materialization path if separate
phalcom-semantic/src/types/evidence.rs
phalcom-semantic/src/presentation.rs
```

**Modify:** nearest canonical owners above.

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/type_annotations.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs
```

### Required implementation shape

#### Static argument marker

Add a heterogeneous application argument case conceptually equivalent to:

```rust
StaticArgument::DynamicType
```

It has no `TypeId` and no DynamicReason in identity.

Kind application validates it as admissible only when the corresponding
parameter kind is exactly `Type`.

#### Formation

Refactor generic argument formation so that:

```text
Ready(TypeId) -> StaticArgument::Term(TypeId)
Dynamic       -> StaticArgument::DynamicType   only for Type-kind slot
Record row    -> existing RecordRow argument
Unknown/etc.  -> preserve existing terminal state
```

Do not early-return whole-form Dynamic merely because one argument is explicit
Dynamic.

#### Materialization outcome

Introduce a private/public semantic result sufficient for specialization:

```text
Known(TypeId)
Dynamic(DynamicReason or obligation)
terminal existing states as needed
```

Exact naming may reuse `TypeKnowledge`.

Applying a knowledge-bearing environment recursively must implement:

```text
T [T := Dynamic]
    -> Dynamic

Box<T> [T := Dynamic]
    -> Known(Box<Dynamic>)

Int [T := Dynamic]
    -> Known(Int)
```

The exact TypeSubstitution fast path must remain unchanged for all-exact
bindings.

### Forbidden approaches

- no `TypeData::Dynamic`;
- no synthetic Dynamic TypeId;
- no Dynamic→Any;
- no Unknown→Dynamic;
- no List-specific branch;
- no mutation of declaration-owned signatures.

### Test changes required

Create focused tests for CV-10..CV-14, including non-Type kind rejection.

### Tests to run now

Only direct type formation/materialization tests.

### Tests explicitly deferred

Constructor/member dispatch, metadata/runtime persistence, Iterator chain.

### Acceptance

`List<Dynamic>` is a canonical applied Type shape distinct from bare Dynamic,
and exact/fixed/dependent materialization controls satisfy INV-08..INV-13.

### Local STOP / CONSULT triggers

- current application machinery cannot accept heterogeneous argument categories
  without replacing its public generic model;
- Dynamic marker leaks into `ProperTypeId` as a standalone term;
- applying Dynamic to a parameter cannot distinguish bare-T from nested-F<T>
  without duplicating type inference.

### Checkpoint update

Yes. Record the Dynamic argument representation and materialization contract.

---

## T5 — Implement exact/open/optional structural relation and remove misleading pack subtype paths

### Purpose

Make structural relation implement the T2 law over the T3 canonical shape
while preserving the proper-Type boundary.

### Preconditions

- T2 and T3 accepted.
- Direct structural law tests exist before production relation edits.

### Consumes

- Record presence/remainder canonical shape;
- Tuple lane remainder canonical shape;
- existing bounded relation budgets/cancellation;
- existing captured row semantics.

### Produces

- exact closed Record relation;
- explicit-open Record relation;
- optional member relation;
- typed/unconstrained remainder rules;
- Tuple lane-aware relation;
- canonical failure outcomes;
- no ordinary pack subtype logic reachable/advertised.

### Files and symbols

**Read/modify:**

```text
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/row.rs
phalcom-semantic/src/types/store.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/advanced/record_rows.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs
new/existing structural relation law module
```

### Required implementation shape

#### Record relation

Treat target shape as contract:

1. compare fixed required target members:
   - required target must be provably present in source;
   - value type relation is covariant;
2. optional target:
   - closed source may omit;
   - if source provably contains label, its type must be compatible;
   - source anonymous remainder that may contain the label can satisfy the
     optional target only when its typed remainder proves compatibility;
   - unconstrained source remainder cannot prove compatibility for that possible
     field;
3. extra source fields:
   - closed target rejects;
   - captured/open target routes through existing row-tail relation;
   - typed anonymous target remainder checks every known source extra and any
     source remainder proof;
   - unconstrained target remainder accepts extra field presence without
     fabricating a common value type;
4. source openness cannot prove a closed target unless the source shape proves
   there can be no extra members.

Do not collapse “cannot prove because open/Dynamic” into a false static proof.

#### Tuple relation

Compare fixed positional/labeled members according to current variance/order
rules, then lane remainder state independently.

No optional positional members.

#### Pack relation cleanup

After identity fast path, unequal pack/static-domain terms must be rejected
before match arms. Remove ordinary pointwise TypePack/LabeledTypePack subtype
branches or move any useful comparison to a clearly named domain-equality/
compatibility helper outside ordinary `<:`.

### Forbidden approaches

- no retaining implicit closed width subtyping;
- no Option encoding for optional member;
- no treating anonymous remainder as captured row;
- no weakening `KindId::TYPE` gate;
- no unordered labeled Tuple rewrite unless the spec says so.

### Test changes required

Implement CV-01..CV-09 as direct relation laws, including transitive
representative cases and negative controls.

### Tests to run now

Run only the new structural law selectors and directly affected existing
record-row relation selectors.

### Tests explicitly deferred

Full semantic suite; metadata; core runtime.

### Acceptance

G2's direct structural relation matrix is green and existing correlated
row-polymorphism tests remain green or are updated only where T2 intentionally
changed the closed-width oracle.

### Local STOP / CONSULT triggers

- exactness breaks a non-structural nominal/family relation;
- captured row solver must be redesigned;
- optional/open relation requires unsound proof from unconstrained source
  remainder.

### Checkpoint update

Yes. Record exact/open structural relation as implemented/focused-tested.


## T6 — Integrate Dynamic argument slots through application consumers and persistence

### Purpose

Carry the T4 Dynamic-argument representation through generic member/call
specialization, relation/conformance, metadata, incremental identity, and
runtime/reflection type descriptions.

### Preconditions

- T4 accepted.
- T5 proper-Type boundary remains intact.
- No consumer-specific workaround has been added.

### Consumes

- canonical applied types containing Dynamic argument markers;
- knowledge-bearing materialization path;
- existing generic variance/substitution machinery;
- metadata/static argument schemas;
- runtime type overlay/materializer.

### Produces

- constructor/member selection for known outer head with Dynamic slots;
- dependent versus fixed result behavior through real callable specialization;
- generic applied relation preserving Dynamic boundary obligations;
- durable metadata representation;
- runtime/reflection type representation;
- cold/import/incremental identity for dynamic-slot applied types.

### Files and symbols

**Read:**

```text
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/types/application.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/specialization.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/normalize*.rs
phalcom-semantic/src/metadata/export.rs
semantic metadata import/materialization/validation path
phalcom-type-meta/src/type_node.rs
phalcom-core/src/typing/overlay.rs
core metadata/runtime type materialization path
incremental fingerprint/dependency code for type forms/signatures
```

**Modify:** only owners proven necessary by the source trace.

**Tests:**

```text
semantic foundations/type annotations
semantic generic call/application tests
semantic metadata tests
semantic incremental product stability tests
core typing integration/reflection tests if runtime type observation exists
```

### Required implementation shape

#### Generic/member specialization

For a receiver with an applied form such as:

```text
List<Dynamic>
```

member lookup may use the known `List` declaration/head. Generic
specialization receives argument bindings:

```text
T -> Dynamic
```

through the knowledge-bearing environment.

A member signature result:

```text
Int
```

remains Known.

A result:

```text
T
```

becomes Dynamic.

A result:

```text
Option<T>
```

becomes `Option<Dynamic>` if the Option application itself is statically
known and the only uncertainty is the Dynamic slot.

Parameters specialized from T similarly become Dynamic obligations at the call
boundary; they must not become Any.

#### Relation

When comparing applications of the same generic declaration:

- exact argument pairs use existing variance rules;
- a relation step that depends on a Dynamic argument returns a Dynamic boundary
  outcome rather than Proven or an invented refutation;
- a declaration's variance remains authoritative;
- no Dynamic slot implies blanket covariance.

Different nominal heads retain existing nominal/supertype behavior.

#### Metadata

Extend the heterogeneous static argument schema additively, conceptually:

```rust
enum StaticArgumentRef {
    Term(TypeNodeId),
    RecordRow(RecordRowRef),
    DynamicType,
}
```

Extend exporter/importer/validator/fingerprint logic.

For structural optional/remainder metadata introduced by T3/T5:

- retain old closed exact Tuple/Record forms where representationally identical;
- add new shape variants/fields only for semantics old nodes cannot encode;
- older metadata must remain valid under existing compatibility policy.

Do not serialize Dynamic provenance/reason as type identity.

#### Runtime/reflection

Extend runtime static argument representation equivalently:

```rust
RuntimeStaticArgument::DynamicType
```

A reflected applied type may report `List<Dynamic>` while values crossing an
ordinary Dynamic variable still retain their exact runtime type under existing
data-object rules.

These are distinct cases and tests should assert both.

#### Incremental

Structural fingerprints include:

- presence;
- anonymous remainder category/type;
- Tuple lane remainder state;
- Dynamic generic argument marker.

A semantically identical rewrite must preserve the interface fingerprint;
changing one of these contract dimensions must invalidate dependents.

### Forbidden approaches

- no bare Dynamic member dispatch treated as List dispatch;
- no Dynamic→Any relation;
- no schema rewrite that discards existing stable declaration identity;
- no source-string fingerprinting;
- no runtime-only type reconstruction.

### Test changes required

Implement CV-10..CV-15 and CV-26 controls:

```text
List<Dynamic>.new selects List constructor
fixed result remains Known
T result becomes Dynamic
Box<T> result becomes Box<Dynamic>
List<Dynamic> != List<Any>
metadata round trip
cold/import equivalence
incremental interface change on Dynamic-slot edit
```

Also add malformed metadata validation for a Dynamic marker in a non-Type kind
slot if metadata carries enough kind context to validate it.

### Tests to run now

During BUILD:

- direct generic call/member selectors;
- one metadata round-trip selector;
- one incremental cold-parity selector.

Do not run full metadata/incremental filters until G9.

### Tests explicitly deferred

Natural Iterator chain; workspace gates; rest forwarding.

### Acceptance

All Dynamic-slot consumers agree on one representation and no path collapses
the slot to Any, whole-form Dynamic, or Unknown without a semantic reason.

### Local STOP / CONSULT triggers

- metadata cannot represent the distinction additively under current versioning;
- generic specialization requires duplicating declaration signatures;
- relation needs a new general gradual subtype lattice rather than bounded
  Dynamic outcomes.

### Checkpoint update

Yes. Record durable Dynamic applied-type contract and metadata status.

---

## T7 — Prove and repair generic witness specialization; make Iterator adapter contracts well-formed

### Purpose

Remove the compiler/source contract causes behind the current Iterator
conformance bootstrap failures without weakening conformance semantics.

### Preconditions

- T1 diagnostic ledger exists.
- T4/T6 specialization architecture is available if Dynamic argument
  materialization is relevant.
- P4.R1 inferred-result invariants remain green.

### Consumes

- `TraitSurfaceMember::instantiate`;
- `specialize_witness_signature`;
- `witness_signature_return_type`;
- current Iterator adapter source;
- generic constraint/projection well-formedness rules.

### Produces

- focused proof or disproof of inferred-return specialization hypothesis;
- confirmed fix if needed;
- explicit Iterator bounds for every adapter that projects/calls Iterator;
- typed mapper/predicate/callback contracts;
- result generic `U` inference path;
- updated diagnostic ledger attribution.

### Files and symbols

**Read/modify if confirmed:**

```text
phalcom-semantic/src/impls.rs
  specialize_witness_signature
  witness_signature_return_type
  check_witness_compatibility_with_visibility

phalcom-semantic/src/traits.rs
  TraitSurfaceMember::instantiate
```

**Modify source contracts:**

```text
phalcom-core/core/universe/src/collections/iterator_laws.ph
```

**Tests:**

```text
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/capabilities/generic_conformance*.rs
existing Iterator semantic fixture owner
```

Adapt exact file placement to existing test module ownership.

### Required implementation shape

#### Phase A — witness reproducer before production edit

Create a focused semantic fixture:

```phalcom
trait IterLike {
  type Item
  iterator -> Self
}

class Adapter<I> {
  iterator { self }
}

impl<I> IterLike for Adapter<I> {
  type Item = Int
}
```

or an equivalent repository-native fixture in which:

- candidate method has no explicit result;
- canonical inferred result contains `Self` and/or owner generic;
- instantiated requirement has exact receiver result.

Assert both views before compatibility.

If candidate inferred result is not specialized and that is the observed
incompatibility/blocker, the hypothesis is confirmed.

#### Phase B — candidate specialization

Materialize `candidate.inferred_return` using the same receiver/conformance
environment as candidate declared terms.

Do not write the specialized result back into the canonical signature table.

For exact-only bindings, reuse existing `TypeView`/`TypeSubstitution` behavior.
For Dynamic-bearing bindings, use T4/T6 knowledge-bearing materialization.

#### Phase C — Iterator source bounds

Audit every adapter declaration/body.

At minimum, source requiring `I |= Iterator` includes the adapters that:

```text
call _upstream.next
project <I as Iterator>::Item
```

Place the bound at the smallest source owner that makes every declaration body
well-formed and every associated type binding justified. Prefer one enclosing
generic contract over redundant repetition where Phalcom scoping carries the
bound.

Do not alter already-accepted Chain/FlatMap/Flatten/StrictZip/Indexed semantics.

#### Phase D — callable contracts for inference

Make result generics inferable.

Target shape, adapted to actual syntax:

```phalcom
map<U>(_ f: (Self::Item) -> U) -> MappedIterator<Self, U>

filterMap<U>(_ f: (Self::Item) -> Option<U>)
    -> FilterMapIterator<Self, U>

mapWhile<U>(_ f: (Self::Item) -> Option<U>)
    -> MapWhileIterator<Self, U>
```

Constructors must carry equivalent mapper contracts and required `I |= Iterator`
bounds.

Type predicate/callback adapters where semantically known:

```text
filter / skipWhile / takeWhile: (Item) -> Bool
inspect: (Item) -> Unit or current callback result contract if intentionally ignored
```

Do not invent a return type if current callable semantics intentionally permit
a broader callback; verify source conventions first.

### Forbidden approaches

- no conformance compatibility weakening;
- no explicit return annotation solely to avoid inferred-return specialization;
- no implicit bound synthesized from `<I as Iterator>::Item`;
- no `U = Dynamic`/Any/Self::Item default;
- no Iterator-specific compiler branch.

### Test changes required

CV-16..CV-20.

Add negative controls proving:

- wrong inferred witness result remains incompatible;
- a projection without declared Iterator evidence remains invalid;
- conflicting mapper returns remain underconstrained/incompatible rather than
  silently defaulted.

### Tests to run now

1. exact witness reproducer;
2. exact witness negative;
3. exact adapter source-contract fixture;
4. existing accepted adapter projection fixture.

Do not run the whole Universe/core corpus yet.

### Tests explicitly deferred

Full bootstrap gate until T8; natural chain until G6 passes.

### Acceptance

- witness hypothesis is either disproved with a new causal ledger entry or
  confirmed/fixed with strict negative controls;
- every Iterator adapter is source-well-formed;
- `U` is inferable from callable contracts;
- accepted adapter Item semantics remain green.

### Local STOP / CONSULT triggers

- witness incompatibility persists after complete use-site materialization;
- fixing source bounds requires changing generic constraint semantics;
- mapper result cannot be expressed in current callable syntax without a
  language extension;
- accepted adapter semantics would need to change.

### Checkpoint update

Yes. Record witness disposition, source contract changes, and focused selector
results.

---

## T8 — Restore Universe Iterator/Iterable bootstrap and classify terminal collection Dynamic

### Purpose

Eliminate the current global semantic precondition blockers and ensure Universe
collection defaults expose the most precise contracts already expressible by
the type system.

### Preconditions

- T7 accepted.
- T6 Dynamic applications work where still intentionally used.

### Consumes

- T1 diagnostic ledger;
- repaired witness specialization if confirmed;
- repaired adapter bounds/callable contracts;
- existing `Iterator`/`Iterable` associated types and derived law.

### Produces

- updated zero-or-dispositioned bootstrap diagnostic ledger;
- precise `toList`, `partition`, `toSet` contracts where directly derivable;
- intentional Dynamic classification for `group`/`toMap` where precision needs
  additional API design;
- semantic precondition green for T9.

### Files and symbols

**Read/modify:**

```text
phalcom-core/core/universe/src/collections/iterator_laws.ph
phalcom-core/core/universe/src/collections/range.ph
related List/Set/Map source declarations only if required to express valid
generic contracts
```

**Tests:**

existing Universe bootstrap/semantic module tests and exact previously blocked
T6 selectors.

### Required implementation shape

#### Diagnostic ledger

For every prior 20+3 entry:

```text
old location/code
new state
cause
fix task
focused evidence
```

No aggregate count-only closure.

#### Terminal APIs

Prefer associated Item contracts where sufficient.

Expected P5 targets:

```phalcom
Iterable::toList -> List<Self::Item>

partition(where predicate)
    -> (List<Self::Item>, List<Self::Item>)

toSet -> Set<Self::Item>
```

subject to actual Set generic constraints. If `Set<T>` requires extra
conformance not guaranteed by `Iterable`, add the necessary method constraint
or preserve Dynamic with explicit classification; do not fake evidence.

`RangeIterator::toList`, if retained as an inherent optimization/helper, must
return the type implied by RangeIterator's canonical Item. It must not be the
sole way the natural chain obtains a list type.

For `group` and `toMap`:

- if existing source contracts can express key/value relationships without
  inventing new language machinery, type them;
- otherwise keep the Dynamic boundary and record it as
  `INTENTIONAL_DYNAMIC_BOUNDARY` / deferred API precision.

### Forbidden approaches

- no suppressing global diagnostics;
- no removing failing impls from bootstrap;
- no lowering diagnostics to warning;
- no hardcoded collection method result type in compiler;
- no speculative `Set<Self::Item>` if Set requirements are not established.

### Test changes required

- bootstrap ledger assertions where supported;
- direct `toList` result contract;
- `partition` and `toSet` type controls;
- preserve `toMap` runtime/typing behavior.

### Tests to run now

- the exact Universe preflight that previously reported 20+3;
- directly affected semantic Iterator/Iterable selectors;
- previously blocked T6 eligible/ineligible evidence selectors once the
  precondition is green.

### Tests explicitly deferred

Natural Range chain runtime until G6; broad collection corpus.

### Acceptance

All P5-owned bootstrap diagnostics are gone. Any residual is individually
classified and does not block the semantic precondition required by T9.

### Local STOP / CONSULT triggers

- residual conformance failures reveal a new compiler architecture defect;
- precise terminal typing requires redesign of Set/Map generic laws;
- fixing `toList` would bypass derived Iterable evidence.

### Checkpoint update

Yes. Record G6 bootstrap result and Dynamic terminal classification.

---

## T9 — Restore the natural Range → TakeIterator → Iterable → toList chain

### Purpose

Prove end-to-end that P5's type/conformance architecture composes naturally in
source, lowering, and runtime execution.

### Preconditions

- G6 Universe bootstrap is green.
- T8 terminal `toList` contract is established.
- no focused test bypasses global preflight.

### Consumes

- canonical RangeIterator result;
- TakeIterator Iterator conformance;
- derived Iterator=>Iterable law;
- trait default canonical inferred/declared result;
- lowering evidence projection;
- runtime iterator behavior.

### Produces

One end-to-end acceptance oracle:

```phalcom
(0..).iter.take(10).toList
```

with semantic and runtime assertions.

### Files and symbols

**Read:**

```text
phalcom-core/tests/core/language/iteration.rs
phalcom-core/tests/core/language/traits.rs
semantic lowering paths referenced by existing exact/derived evidence test
Range/Iterator Universe sources
```

**Modify:**

tests and only production code causally implicated by the chain after T8.

### Required implementation shape

Assert these stages separately in one or more focused tests:

```text
Range expression
    -> canonical Range

.iter
    -> selected Iterable::iter with Exact Range evidence
    -> result RangeIterator

.take(10)
    -> selected inherent RangeIterator::take
    -> result TakeIterator<RangeIterator>

TakeIterator<RangeIterator>
    -> exact Iterator evidence
    -> associated Item matches RangeIterator Item

.toList
    -> GuardedRule/Derived Iterable evidence
       with nested exact Iterator proof
    -> selected Iterable::toList callable
    -> specialized list result

lowering
    -> same Exact/Derived selection projected into TraitInvocationSpec

runtime
    -> finite list [0..9] under existing Range semantics
```

Use the live Range start/end semantics; do not hardcode expected values if
current range syntax is inclusive/exclusive differently. The test should derive
the expected sequence from current accepted Range behavior.

### Forbidden approaches

- no direct installation of `toList` on TakeIterator solely for test passage;
- no Range/Take/List compiler branch;
- no fake conformance target installation;
- no runtime proof search.

### Test changes required

CV-22 plus existing exact/derived lowering controls.

Also run CV-21 `toMap` and invalid `take`/`skip` runtime controls once the
shared iterator defaults are executable.

### Tests to run now

Exact/narrow core selectors:

```text
range_take_to_list_preserves_full_evidence_chain
ordinary_trait_lowering_preserves_exact_and_derived_dispatch_evidence
toMap single-advance regression
skip/take invalid-argument regression
```

Then, and only then, the directly blocked `corpus::collections` selector.

### Tests explicitly deferred

Workspace tests/clippy/format; unrelated corpora.

### Acceptance

G7/G8 are green with no special-case production code.

### Local STOP / CONSULT triggers

- semantic chain is correct but lowering changes evidence identity;
- lowering is correct but runtime lacks an environment required by the selected
  trait default;
- failure requires modifying derived conformance rule semantics.

### Checkpoint update

Yes. Record the full semantic/lowering/runtime chain and collection regression
results.

---

## T10 — Complete metadata, incremental, source-index, and LSP persistence for P5 types

### Purpose

Ensure P5's new structural and Dynamic-applied forms remain semantically stable
across modules, edits, metadata, and editor projections.

### Preconditions

- T3/T5 structural representation/relation accepted.
- T4/T6 Dynamic application representation accepted.
- no pending representation changes.

### Consumes

- canonical structural presence/remainder;
- Dynamic static argument marker;
- existing metadata TypeNode graph;
- incremental fingerprint/dependency architecture;
- source index/presentation conventions.

### Produces

- metadata schema representation for new durable forms;
- import/export/validation;
- cold/incremental equivalence;
- source index and presentation/LSP support;
- interface fingerprint tests.

### Files and symbols

**Read/modify as needed:**

```text
phalcom-type-meta/src/type_node.rs
phalcom-type-meta schema/fingerprint tests
phalcom-semantic/src/metadata/export.rs
semantic metadata import/validation/materialization
semantic incremental fingerprint/dependency modules
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/presentation.rs
LSP navigation/completion/token projection paths affected by TypeAnnotation AST
phalcom-core/src/typing/overlay.rs and metadata runtime materializer
```

### Required implementation shape

#### Structural metadata

Preserve existing old nodes for representationally identical closed forms where
possible.

Add new metadata shape only when needed to encode:

```text
optional member presence
anonymous Record remainder
Tuple positional/labeled anonymous remainder
```

Captured row tails retain existing stable parameter identity.

Do not encode an anonymous remainder as a fake stable parameter ref.

#### Dynamic argument metadata

Add Dynamic argument marker to static argument refs and validate it only in
Type-kind positions.

#### Fingerprints

Include semantic categories, not syntax spelling. Examples:

```text
{a:Int, ...} and syntactic equivalent
    -> same fingerprint

{a:Int} vs {a:Int, ...}
    -> different

{a:Int} vs {a?:Int}
    -> different

List<Any> vs List<Dynamic>
    -> different
```

#### Incremental dependencies

An edit changing a public structural remainder/presence or Dynamic generic slot
must invalidate dependent consumers.

A body-only edit preserving the same canonical callable result continues to
reuse P4.R1 interface products.

#### Tooling

Navigation on fixed member types should still resolve their source type names.
Completion/presentation should print/retain `?` and anonymous remainder syntax.
Semantic tokenization must distinguish the structural markers using existing
token classes; do not invent a new token protocol unless necessary.

### Forbidden approaches

- no source-text metadata identity;
- no anonymous row parameter;
- no full metadata graph rebuild workaround for incremental correctness;
- no editor-only parallel type model.

### Test changes required

CV-15, CV-26, CV-27 plus direct LSP/source-index tests for new syntax.

### Tests to run now

BUILD:

- one metadata schema/unit test per new variant;
- one semantic metadata round trip;
- one cold/incremental edit test per structural/Dynamic dimension;
- focused source-index/LSP selectors.

STABILIZE at G9:

```text
cargo test -p phalcom-type-meta
cargo test -p phalcom-semantic integration::metadata
cargo test -p phalcom-semantic incremental
```

Use exact repository command syntax/environment.

### Tests explicitly deferred

Workspace certification.

### Acceptance

No durable P5 distinction is lost across metadata/import/incremental/tooling,
and P4.R1 result lifecycle selectors remain green.

### Local STOP / CONSULT triggers

- schema compatibility requires destructive version migration;
- incremental correctness appears to require a second semantic authority;
- LSP would need to parse source text instead of consuming semantic/AST facts.

### Checkpoint update

Yes. Record schema/version disposition, cold parity, and tooling gates.


## T11 — Prove and close rest/module-export conformance evidence transport

### Purpose

Resolve the remaining P4 forwarding seam by first proving semantic
specialization/evidence creation and only then localizing any compiler/runtime
transport loss.

### Preconditions

- T7 witness/conformance specialization accepted.
- T9 exact/derived evidence lowering works on an ordinary non-forwarded call.
- Existing rest/module-export probe state rechecked.

### Consumes

- semantic selected callable;
- call specialization product;
- canonical conformance evidence recipe;
- existing lowering/runtime environment transport;
- P4 nested evidence-slot repair.

### Produces

One of two legitimate outcomes:

```text
A. semantic specialization now exists and evidence transport is proven correct;
or
B. a specific lowering/runtime forwarding defect is reproduced and repaired.
```

A third outcome—still no semantic specialization after one serious semantic
correction—requires consultation.

### Files and symbols

**Read first:**

```text
phalcom-core/tests/core/language/generic_conformance.rs
  p4_specialized_rest_dispatch_preserves_conformance_environment

docs/implementation/LANG007/LANG007.C3/
  LANG007.C3.P4-audit-remediation/
  03-nested-derived-conformance-runtime-layout.md

semantic call resolution/specialization products
semantic_lowering.rs exact/derived evidence projection
phalcom-core/src/bytecode.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/send.rs
```

**Modify:**

- semantic test/probe first;
- production source only after the first losing layer is identified.

### Required implementation shape

#### Stage 1 — strengthen the probe

At the forwarding call expression assert:

```text
callable.is_some()
call_specialization.is_some()
specialization conformance recipe is non-empty
expected trait/evidence slot identity is present
```

If any assertion fails, debug semantic resolution/specialization; do not touch
VM forwarding.

#### Stage 2 — lowering

Assert the lowered call/invocation product contains the same semantic recipe
identity and slot mapping.

If semantic has evidence and lowering does not, repair projection in compiler
lowering. Do not search for conformance again.

#### Stage 3 — runtime forwarding

Only if both semantic and lowering contain the recipe, execute the forwarding
path and inspect:

```text
type environment id
conformance environment id
slot count/key mapping
forwarded call frame
```

Repair only the first transport point that drops or replaces the environment.

Rest-family fallback and module-export forwarding must use the same already
selected evidence semantics as ordinary calls.

### Forbidden approaches

- no VM proof search;
- no `ImplId` → evidence reconstruction;
- no installing a global active conformance target;
- no rest-specific fake trait implementation;
- no changing selector routing to bypass the intended forwarding path.

### Test changes required

CV-24 and CV-25.

Add controls:

- ordinary direct generic evidenced call remains green;
- forwarded call with no required evidence does not allocate/install a fake
  conformance environment;
- repeated forwarding reuses equivalent runtime environment according to
  existing environment interning rules.

### Tests to run now

Run the strengthened exact rest selector only.

If module-export is a separate established probe, run its exact selector only
after rest localization or if rest is already correct.

If runtime common dispatch code changes, run the narrow existing dispatch
environment tests plus the exact forwarding selector.

### Tests explicitly deferred

Full core library/workspace tests until T14.

### Acceptance

The test proves a non-empty semantic recipe and either proves end-to-end
transport or identifies/fixes the exact transport layer. Runtime remains
proof-free.

### Local STOP / CONSULT triggers

- semantic recipe is still absent after one serious correction;
- runtime needs to inspect source impls/traits to rebuild evidence;
- rest and module-export require materially different conformance architecture.

### Checkpoint update

Yes. Record which layer owned the failure and exact evidence.

---

## T12 — Run focused P5 law/regression matrix and repair only P5-causal breakage

### Purpose

Stabilize all directly affected semantics before broad certification.

### Preconditions

- T2–T11 completed to their focused acceptance.
- No known architectural consultation is unresolved.

### Consumes

- CV-01..CV-27;
- predecessor P4.R1 controls;
- current causal ledger.

### Produces

- focused acceptance table;
- any narrow P5-caused corrections;
- stable basis for Priority-3 classification and broad gates.

### Files and symbols

No predetermined production edits. Tests and source are modified only when a
focused failure is classified A/B.

### Required implementation shape

Run the focused matrix in coherent groups:

```text
structural syntax/formation
structural relation laws
Dynamic generic formation/materialization
metadata/incremental durability
witness specialization
Iterator adapter contracts
Universe bootstrap
natural collection chain
exact/derived lowering
rest/module-export transport
P4.R1 result lifecycle regressions
```

For each failure, record A/B/C/D before editing.

Do not rerun a passing group after unrelated edits unless the edited subsystem
can causally affect it.

### Forbidden approaches

- no broad repair sweep;
- no snapshot/oracle update without normative justification;
- no converting P4.R1 expected behavior to P5 behavior unless P5 explicitly
  owns that semantic dimension.

### Test changes required

Only corrections to test fixtures made invalid by intentional P5 semantic
migration, with rationale recorded in checkpoint/walkthrough.

### Tests to run now

All G1–G10 focused commands, grouped to minimize redundant compilation.

### Tests explicitly deferred

Workspace-wide test/clippy/format until T14.

### Acceptance

Every CV-01..CV-27 obligation has a PASS or a formally accepted
NOT-APPLICABLE/deferred result consistent with plan scope. No P5-caused failure
remains open.

### Local STOP / CONSULT triggers

- a normative oracle conflict appears;
- an invariant must be weakened;
- same semantic failure persists after one serious correction.

### Checkpoint update

Yes. Record focused matrix and any accepted fixture migrations.

---

## T13 — Disposition Priority-3 and other residual regressions by causality

### Purpose

Prevent broad certification from conflating P5 regressions with historical
failures while still taking responsibility for failures caused by P5.

### Preconditions

- T12 focused matrix green.

### Consumes

- `04-semantic-regression-classification.md`;
- current checkpoint;
- any failures encountered in P5 affected module filters.

### Produces

An updated residual ledger with each encountered failure marked:

```text
P5_CAUSAL_FIXED
P5_CAUSAL_OPEN          # blocks T14; normally not allowed to proceed
UNCHANGED_CURRENT_REGRESSION
CLEAN_PREDECESSOR_BASELINE
INTENTIONAL_P5_MIGRATION
FIXTURE_INTERFERENCE_CORRECTED
UNRELATED_NEW_REGRESSION
```

### Files and symbols

**Read/modify:**

```text
docs/implementation/LANG007/LANG007.C3/
  LANG007.C3.P4-audit-remediation/
  04-semantic-regression-classification.md

docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
```

Do not rewrite historical columns; add current P5 disposition/evidence.

### Required implementation shape

A residual becomes P5-owned only with a trace connecting it to:

- structural shape/relation;
- Dynamic applied arguments;
- specialization/materialization;
- witness/conformance;
- Iterator source contracts;
- P5 metadata/incremental changes;
- evidence forwarding.

One bounded trace for C-classification is allowed.

### Forbidden approaches

- no “P5 touched semantic code, therefore every semantic failure is P5”;
- no hiding a P5 regression as baseline;
- no broad production fixes for D failures.

### Tests to run now

Only exact selectors necessary to classify C failures that block T14.

### Tests explicitly deferred

None needed after classification except broad gates.

### Acceptance

No encountered failure has ambiguous ownership that would make broad gate
results misleading.

### Local STOP / CONSULT triggers

A blocking failure remains C after one bounded causal trace.

### Checkpoint update

Yes. Persist residual ledger before broad certification.

---

## T14 — Broad checkpoint certification

### Purpose

Run the bounded broad verification required to claim P5/C3 closure and
truthfully classify any remaining failures.

### Preconditions

- T12 focused matrix green.
- T13 residual ownership clear.
- worktree contains only intended P5 changes plus preserved user changes.

### Consumes

- all focused gate evidence;
- baseline/residual ledger.

### Produces

- crate/workspace verification record;
- lint/format record;
- final release/checkpoint classification.

### Files and symbols

No planned production edits. If a broad failure is A/B, return to the owning
task once. Do not continue broad certification through a known P5 regression.

### Required verification order

Run in this order, stopping on P5-causal failure:

1. affected parser/AST owning suite;
2. `phalcom-type-meta` tests;
3. affected semantic module filters;
4. full `phalcom-semantic` test suite if project closure policy requires it;
5. affected `phalcom-core --test core` filters;
6. collection corpus aggregate;
7. `cargo check --workspace --all-targets`;
8. `cargo test --workspace` once;
9. `cargo clippy --workspace --all-targets -- -D warnings`;
10. `cargo fmt --all -- --check`.

Use the repository's required nightly/tool environment.

Do not repeat an unchanged broad command merely to seek a different result.

### Forbidden approaches

- no broad-test loops;
- no unrelated cleanup while certifying;
- no disabling tests/warnings;
- no rewriting baseline classifications without evidence.

### Test changes required

None unless a broad failure proves a P5 test integration mistake; production
semantic changes return to the appropriate task.

### Acceptance

All P5-caused gates are green. Any remaining D/baseline failure is documented
with exact selector/evidence and is consistent with checkpoint closure policy.
If checkpoint policy requires a fully green workspace and a baseline blocks it,
C3 remains not fully certified; report truthfully.

### Local STOP / CONSULT triggers

- a soundness-critical A/B broad failure;
- a C failure blocks closure after bounded classification;
- broad results contradict focused products without an obvious mechanical test
  harness cause.

### Checkpoint update

Yes. Record exact commands/results and certification status.

---

## T15 — Converge specs, checkpoint, walkthrough, and handoff

### Purpose

Make repository state durable and ensure future agents receive one coherent
account of P5 rather than reconstructing it from incidents and test logs.

### Preconditions

- T14 disposition complete.

### Consumes

- implemented architecture;
- deviations/consultations;
- exact verification evidence;
- residual ledger.

### Produces

```text
LANG007.C3.P5-walkthrough.md
LANG007.C3.P5-handoff.md
final LANG007.C3-CHECKPOINT.md update
final normative spec convergence
```

### Files and symbols

**Modify:**

```text
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
docs/specs/README.md
docs/specs/structural-products.md
docs/specs/tuple-and-record-typing.md
```

only if post-implementation wording needs exact symbol/test references.

**Create:**

```text
docs/implementation/LANG007/LANG007.C3/
  LANG007.C3.P5-walkthrough.md

docs/implementation/LANG007/LANG007.C3/
  LANG007.C3.P5-handoff.md
```

### Required implementation shape

The walkthrough must separate:

```text
architectural rulings
actual representation
actual source contracts
actual tests run
deferred/unrelated failures
certification level
```

The handoff must state what is now stable and what the next checkpoint/plan may
assume.

If G12 supports C3 closure, set checkpoint status/completion/verification
accordingly. If any closure criterion remains blocked, keep C3 truthfully
partial/unverified and set the exact next action.

### Forbidden approaches

- no claiming workspace certification if command was incomplete;
- no claiming a test passed if it failed before assertion;
- no hiding consultations/deviations;
- no stale P4 incident as current architecture.

### Tests to run now

None unless a documentation-only source census tool is part of repository
policy.

### Acceptance

Specs, code description, checkpoint, walkthrough, and handoff agree on the same
semantic model and verification state.

### Local STOP / CONSULT triggers

Documentation reveals an unresolved architecture contradiction not noticed
during implementation.

### Checkpoint update

Final P5 update.

---

# 19. Verification gates

## G1 — Structural authority and source representation

**Purpose:** prove one effective structural spec and one AST/canonical
representation for presence/remainders.

Run focused parser/formation selectors after T3. Expected command shape:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-ast --test parser structural_type -- --nocapture

RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic \
  semantic::foundations::type_annotations::structural -- --nocapture
```

If repository filters do not have the shown common prefix, run the individual
new test names rather than broadening.

Expected:

```text
optional labeled member parsed
anonymous Record remainder parsed
typed remainder parsed
Tuple lane remainders parsed
captured row syntax unchanged
optional positional rejected
canonical forms distinct as required
```

Do not broaden after PASS.

---

## G2 — Structural relation laws

**Purpose:** prove INV-01..INV-07.

Run the exact new law selectors, preferably under one dedicated module:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic \
  semantic::foundations::structural_shape_relations -- --nocapture
```

Expected at minimum:

```text
CV-01..CV-09 PASS
existing captured-row algebra controls PASS
```

On failure, classify semantic law versus representation before editing.

---

## G3 — Nested Dynamic application/materialization

**Purpose:** prove INV-08..INV-13.

Run:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic \
  semantic::foundations::type_annotations::nested_dynamic_application_preserves_outer_head -- --exact --nocapture

RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic \
  nested_dynamic_dependent_result -- --nocapture

RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic \
  nested_dynamic_fixed_result -- --nocapture
```

Expected:

```text
Dynamic != List<Dynamic> != List<Any>
Dynamic marker only accepted in Type-kind slot
T result -> Dynamic
Box<T> result -> Box<Dynamic>
Int result -> Int
```

---

## G4 — Generic witness specialization

**Purpose:** prove complete use-site specialization without changing provider
ownership.

Run exact selector created in T7:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic \
  generic_inherent_witness_specializes_canonical_inferred_result \
  -- --nocapture
```

Then negative:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic \
  generic_inherent_witness_wrong_result_remains_incompatible \
  -- --nocapture
```

Expected:

```text
provider canonical inferred result unchanged
candidate specialized view exact
compatible proof produced
wrong result rejected
```

---

## G5 — Iterator adapter source-contract gate

**Purpose:** prove INV-17..INV-20.

Run:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic \
  iterator_adapter_fixture_preserves_item_projections_and_chain_equality \
  -- --nocapture

RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic \
  mapped_iterator_result_generic_is_inferred_from_mapper_return \
  -- --nocapture
```

Also run exact FilterMap/MapWhile inference controls and missing-bound negative.

Expected: all pass without generic defaulting.

---

## G6 — Universe bootstrap gate

**Purpose:** remove the global semantic precondition blocker.

Run the exact preflight/check recorded by T1 plus the current previously blocked
T6 selectors.

Expected ledger:

```text
ImplConformanceIncomplete:
  P5-owned = 0

type.generic.underconstrained:
  P5-owned = 0

residual:
  individually classified and nonblocking
```

If the old counts still occur unchanged after T7/T8, STOP AND CONSULT rather
than broadening fixes.

---

## G7 — Natural Range chain gate

**Purpose:** prove INV-22/23.

Run:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-core --test core \
  language::iteration::range_take_to_list_preserves_full_evidence_chain \
  -- --exact --nocapture
```

Expected:

```text
Range -> RangeIterator
RangeIterator.take -> TakeIterator<RangeIterator>
exact Iterator evidence
derived Iterable evidence with nested exact proof
selected Iterable::toList
known/specified list result
lowered evidence identity preserved
runtime finite result correct
```

---

## G8 — Collection regression gate

Run exact existing/new selectors for:

```text
iterator_adapter_fixture_preserves_item_projections_and_chain_equality
toMap single-advance/value semantics
Chain unequal-item negative
skip invalid n
take invalid n
known eligible/ineligible Derived Iterable receiver controls
```

Then:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-core --test core corpus::collections -- --nocapture
```

Do not run `corpus::collections` before G7 passes.

---

## G9 — Persistence, incremental, and tooling

Run:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-type-meta

RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic integration::metadata -- --nocapture

RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic incremental -- --nocapture
```

Run exact new LSP/source-index selectors.

Expected:

```text
structural presence/remainders round-trip
List<Dynamic> round-trips
cold == imported == incremental
semantic fingerprint changes only when contract changes
P4.R1 canonical inferred-result lifecycle remains green
tooling prints/navigates new forms
```

---

## G10 — Rest/module-export evidence transport

First run:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-core --test core \
  language::generic_conformance::p4_specialized_rest_dispatch_preserves_conformance_environment \
  -- --exact --nocapture
```

The test must now assert recipe existence before runtime behavior.

Expected:

```text
callable present
call specialization present
non-empty conformance recipe
lowered recipe equivalent
runtime forwarded environment equivalent
final trait call succeeds
```

Run a module-export counterpart only if it exists or T11 creates it from the
already-audited probe.

---

## G11 — P4.R1 and P5 focused acceptance matrix

Run the exact predecessor result lifecycle selectors:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic \
  trait_default_inferred_return_is_canonical_and_independent_of_caller_expectation \
  -- --nocapture

RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic \
  branch_derived_tail_type_is_published_to_unannotated_callable_signature \
  -- --nocapture

RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic \
  semantic::incremental::callable_dependencies::inferred_return_refresh_preserves_stable_products_and_counts_final_disposition_once \
  -- --exact --nocapture

RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic \
  semantic::incremental::product_stability::inferred_cross_module_result_updates_dependents_and_matches_cold_analysis \
  -- --exact --nocapture

RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic \
  semantic::integration::metadata::cross_module_inferred_callable_result_round_trips_as_compiler_inferred_metadata \
  -- --exact --nocapture
```

Also run G1–G10 new focused groups if any subsequent task modified their owner.

Expected: all P5-owned focused obligations green.

---

## G12 — Broad checkpoint certification

Run once in T14, in order:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo check --workspace --all-targets

RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test --workspace

RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo clippy --workspace --all-targets -- -D warnings

RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo fmt --all -- --check
```

If repository policy requires a semantic/core aggregate before workspace test,
run it first and record separately.

Expected:

- no P5-caused failures;
- any predecessor/baseline failure exactly classified;
- final checkpoint status reflects actual broad result, not intent.

Do not repeat unchanged failing broad commands.

---

# 20. Final focused acceptance

At P5 completion, fill this table in the walkthrough with actual results.

| Coverage ID | Required evidence/test | Completion criterion |
|---|---|---|
| CV-01 | closed structural negative | PASS |
| CV-02 | explicit open positive | PASS |
| CV-03 | optional absence | PASS |
| CV-04 | optional present compatible/incompatible | PASS |
| CV-05 | captured vs anonymous identity | PASS |
| CV-06 | typed remainder | PASS |
| CV-07 | Tuple lane independence | PASS |
| CV-08 | structural canonicalization | PASS |
| CV-09 | proper-Type boundary | PASS |
| CV-10 | `List<Dynamic>` formation distinction | PASS |
| CV-11 | Dynamic argument kind safety | PASS |
| CV-12 | dependent Dynamic result | PASS |
| CV-13 | nested dependent `Box<Dynamic>` result | PASS |
| CV-14 | fixed generic result remains Known | PASS |
| CV-15 | metadata round trip | PASS |
| CV-16 | witness specialization | PASS or hypothesis explicitly disproved with replacement cause |
| CV-17 | witness incompatibility negative | PASS |
| CV-18 | projection/bound legality | PASS |
| CV-19 | mapper `U` inference | PASS |
| CV-20 | accepted adapter semantics | PASS |
| CV-21 | `toMap` single advance | PASS |
| CV-22 | natural Range chain | PASS |
| CV-23 | bootstrap ledger closure | PASS |
| CV-24 | forwarding evidence recipe exists | PASS |
| CV-25 | forwarding transport | PASS |
| CV-26 | cold/import/incremental equivalence | PASS |
| CV-27 | P4.R1 result lifecycle regressions | PASS |
| CV-28 | unrelated failure classification | COMPLETE |

Run only the stabilization gates needed by modified subsystems before T14.
Do not automatically run workspace certification twice.

---

# 21. Performance/resource evidence

P5 is correctness-led, not a performance plan. No new benchmark suite is
required.

However, collect bounded resource evidence where architecture could regress
analysis cost.

### 21.1 Required observations

For one representative incremental test after T10 record:

```text
number of callable bodies recomputed
number of conformance plans recomputed/refreshed
whether structural/Dynamic type edit invalidated only dependents
whether unrelated callable body remained reused
```

P4.R1's same-result reuse behavior must remain intact.

For metadata/type interning, no per-expression/per-call duplicate Dynamic
applied-type nodes should be observed; canonical interning should deduplicate
equivalent `List<Dynamic>` forms.

### 21.2 No performance ceremony

Do not add benchmarks unless evidence reveals a regression large enough to
block checkpoint acceptance. If broad recomputation becomes materially worse,
STOP AND CONSULT before adding caches.

---

# 22. Checkpoint bookkeeping

## 22.1 At plan start

- mark `LANG007.C3.P5` active;
- record live SHA/worktree preservation state;
- record inherited P4.R1 completion;
- record initial bootstrap diagnostic ledger;
- set next action T2.

## 22.2 During plan

Update checkpoint only for:

- T2 structural normative ruling;
- T3 canonical structural representation;
- T4 Dynamic argument/materialization representation;
- T5 structural relation gate;
- T7 witness/source-contract disposition;
- T8 bootstrap closure;
- T9 natural chain;
- T10 durable metadata/incremental/tooling gate;
- T11 forwarding ownership/repair;
- consultation/amendment;
- deferred/baseline failure;
- G11 focused matrix;
- G12 broad certification.

Do not update after every mechanical edit.

## 22.3 At plan completion

Record:

```text
plan status/completion/verification
final revision(s)
implemented invariants
exact gates/commands/results
consultations/amendments
residual baseline failures
broad certification status
whether C3 is COMPLETE or remains PARTIAL
next action
```

---

# 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C3/
  LANG007.C3.P5-walkthrough.md
```

It must contain:

1. final result and revision range;
2. architectural rulings P5-A..P5-J;
3. final structural representation;
4. final Dynamic generic argument/materialization representation;
5. witness specialization finding and actual fix/disproof;
6. Iterator source contract changes;
7. terminal collection Dynamic classification;
8. natural Range chain evidence;
9. metadata/incremental/tooling changes;
10. rest/module-export causal layer and correction;
11. important code changes by file/symbol;
12. deviations from this plan and consultation decisions;
13. tests added;
14. tests actually run with exact results;
15. tests explicitly deferred;
16. baseline/residual classification;
17. performance/resource observations;
18. final verification classification;
19. residual risks.

The walkthrough must distinguish source written, focused tested, aggregate
tested, and workspace certified.

---

# 24. Handoff deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C3/
  LANG007.C3.P5-handoff.md
```

It must contain:

- current final revision/worktree state;
- whether C3 is closed;
- stable source/spec interfaces inherited by the next agent;
- INV-01..INV-28 or a concise stable subset with references;
- canonical structural representation;
- canonical Dynamic generic argument semantics;
- callable/witness specialization invariant;
- Iterator/Iterable source contracts;
- metadata/runtime compatibility notes;
- evidence forwarding result;
- residual baseline failures;
- next checkpoint objective;
- must-read files;
- first recommended commands;
- things not to redesign/re-explore:
  - no Dynamic proper TypeId;
  - no closed width structural semantics unless a future normative change says so;
  - no second callable-result owner;
  - no VM proof search;
  - no generic `U` defaulting;
  - no pack subtyping.

---

# 25. Completion truth table

Do not conflate these states.

| State | Meaning | P5 metadata wording |
|---|---|---|
| source written | production/docs edits exist | `IMPLEMENTED` only for named task if code path exists |
| tests added | regressions/fixtures exist | not equivalent to verification |
| targeted tests passed | T0/T1 focused evidence | `FOCUSED_TESTED` |
| affected suite passed | owning module/feature aggregate green | `AFFECTED_SUITE_TESTED` or repository equivalent |
| checkpoint accepted | all required P5/C3 gates dispositioned | checkpoint completion may become `COMPLETE` |
| release/workspace certified | G12 broad policy passed | `VERIFIED` only if repository policy equates this to verification |

If workspace test is incomplete, lost, or blocked by a baseline, say exactly
that. Never infer a total from partial command output.

---

# 26. Plan self-review

Before implementation begins, the architect/implementer should confirm the
following checklist. This delivered plan has been written to satisfy it.

- [x] Every major invariant has an implementation owner.
- [x] Structural spec conflict is resolved before relation code.
- [x] Dynamic remains outside the standalone TypeId domain.
- [x] Nested Dynamic has one cross-layer representation strategy.
- [x] Use-site specialization is separated from canonical result ownership.
- [x] Iterator source defects are not laundered into compiler permissiveness.
- [x] Every important semantic risk has focused coverage.
- [x] Negative controls exist for the soundness-sensitive paths.
- [x] No duplicate source of truth is planned.
- [x] Luna has objective global and local STOP/CONSULT triggers.
- [x] Mechanical/semantic debugging loops are bounded.
- [x] BUILD verification is intentionally narrow.
- [x] Broad workspace tests are deferred to certification.
- [x] Baseline/unrelated failure policy is explicit.
- [x] Checkpoint/walkthrough/handoff obligations are explicit.
- [x] P4.R1 canonical inferred-result invariants are preserved.
- [x] Existing Iterator adapter corrections are preserved.
- [x] The natural chain is the primary end-to-end oracle.
- [x] Rest forwarding cannot trigger speculative VM edits.
- [x] Metadata/incremental/tooling durability is included.
- [x] No placeholders remain.
- [x] Task dependencies form a valid implementation order.

## 26.1 Final executor summary

The intended execution order is:

```text
T1  baseline
T2  normative structural reconciliation
T3  structural representation
T4  Dynamic argument/materialization representation
T5  structural relation
T6  Dynamic consumer + metadata/runtime integration
T7  witness + Iterator source contracts
T8  Universe bootstrap
T9  natural collection chain
T10 metadata/incremental/tooling closure
T11 evidence forwarding
T12 focused matrix
T13 residual causality
T14 broad certification
T15 documentation/handoff
```

The implementer should optimize for **semantic locality**: establish one
canonical fact at its owner, make consumers project it, and stop whenever a fix
would require a parallel authority or a special case for the current failing
fixture.
