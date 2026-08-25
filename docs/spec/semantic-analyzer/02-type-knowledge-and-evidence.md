# Phalcom Semantic Analyzer Implementation Specification
## 02 — Type Knowledge, Evidence, Authority, and Provenance

**Status:** Normative semantic implementation specification.

**Purpose:** Specify how the analyzer represents knowledge about runtime value types, how that knowledge obtains authority, how its strength changes, and how provenance is preserved through derived facts.

---

## 1. Type knowledge is not a type

`TypeId` represents a type. `TypeKnowledge` represents the analyzer's epistemic relationship to a proposition about a value's type.

Conceptually:

```text
TypeKnowledge =
    Known(type, status, origin, provenance)
  | Unknown(reason)
  | Dynamic(reason)
```

A known `Int` and an unknown value are not two different types. An assumed `Int` and an established `Int` also share the same canonical `TypeId`. Evidence state belongs outside type identity.

This separation is required for canonical type interning, generic substitution, relation checking, reflection, and incremental semantic identity.

---

## 2. Established knowledge

`Established(T)` means the formal checker possesses sufficient checker-owned evidence to publish the proposition that the value has type `T`.

Established evidence may arise from semantic rules such as:

- literal semantics;
- exact constructor semantics;
- exact callable-result contracts after successful identity resolution;
- formally modeled intrinsic/native signatures when those signatures are trusted compiler contracts;
- field/member semantics whose declaring surface is formally resolved;
- structural or nominal derivation from established premises;
- generic inference whose return-influencing variables are solved only from established support;
- flow joins in which every reachable incoming fact remains established.

The specific evidence origin matters even when the epistemic strength is the same.

For example:

```text
Established(CellNum, ConstructorSemantics)
Established(Int, CallableSignature)
Established(Int, GenericInference)
```

are all established but have different explanations.

### 2.1 Established is privileged

Creating established knowledge is part of the trusted semantic core.

A parser, annotation resolver, presentation layer, advisory analysis, LSP feature, or generic helper that merely has a `TypeId` is not automatically authorized to claim `Established`.

The implementation should make unauthorized construction difficult. Exact Rust visibility is non-normative, but the architecture must ensure there are deliberate semantic operations through which established facts are created.

---

## 3. Assumed knowledge

`Assumed(T)` means `T` is available as a usable formal premise, but the checker has not independently established the runtime proposition.

Assumptions are legitimate. They are not errors and should participate in ordinary static checking.

Typical sources include:

- explicit developer binding contracts when no independent value evidence exists;
- callable parameter contracts at body entry;
- contextual block/callable parameter contracts;
- other explicit language contracts designated as assumption-capable.

The analyzer may reject a later operation that contradicts an assumption. It may not present the assumption as checker-established runtime precision.

### 3.1 Assumption eligibility

Not every absence of established evidence permits an assumption.

An assumption can fill a genuine *no-value-evidence* gap. It must not hide:

- syntax errors;
- unresolved names;
- unsupported analyzer paths;
- recursive-fixpoint blockage;
- suppressed dependencies;
- generic inference conflicts;
- underconstrained inference after contextual constraints have been exhausted;
- cancellation;
- budget exhaustion;
- internal failures.

This prevents a developer annotation from laundering an analyzer failure into apparent semantic success.

---

## 4. Unknown knowledge

`Unknown(reason)` means no usable formal type proposition is currently available.

Unknown is not a generic fallback. The reason is semantically meaningful because it tells consumers why knowledge is absent and whether a later phase or edit could change that fact.

Representative categories include:

```text
NoTypeEvidence
UnresolvedName
UncheckedExpression
MissingInitializer
UnderconstrainedTypeVariable
InferenceConflict
SuppressedDependency
RecursiveFixpoint / blocked reason
```

Exact enum organization may evolve.

### 4.1 Unknown must be honest

A checker coverage gap remains a coverage gap. It does not become `Unit`, `Object`, the first generic argument, the expected type, or the developer annotation.

### 4.2 Unknown reason preservation

Aggregation should preserve semantically meaningful unknown reasons where practical. A summary operation must not replace every non-concrete result with a generic `UncheckedExpression` reason.

If multiple reasons converge, the implementation may use a deterministic aggregate class, explanation node, or stable precedence rule. Arbitrary first-input selection should not determine semantics.

---

## 5. Dynamic knowledge

`Dynamic(reason)` means the analyzer deliberately crosses a boundary where static type proof is not available or not required and runtime semantics are authoritative.

Dynamic is not failure.

Therefore:

```text
Dynamic != Unknown
```

A dynamic result may be completely expected and semantically valid. It should not be reclassified as `Unknown` merely because both variants lack a concrete `TypeId`.

Dynamic boundaries may influence call checking, diagnostics, reflection, and LSP presentation differently from unresolved static knowledge.

---

## 6. Evidence status and evidence origin

Evidence strength and evidence origin answer different questions.

```text
status: how strongly is this proposition justified?
origin: why/how was this proposition obtained?
```

Representative status:

```text
Established
Assumed
```

Representative origins include:

```text
LiteralSemantics
ConstructorSemantics
CallableSignature
NativeSignature
GenericInference
DeveloperAnnotation
ContextualDerivation
Flow
```

The exact variant names may differ, but semantic distinctions must be preserved.

### 6.1 Required provenance roles

The following transformations are normative:

| Semantic fact | Epistemic strength | Semantic origin / basis |
|---|---|---|
| literal result | Established | literal/syntax semantics |
| exact `@constructor` result | Established | constructor semantics |
| exact ordinary callable return | Established | callable signature/contract |
| trusted native result | Established | native signature |
| solved generic result from established support | Established | generic inference |
| solved generic result from any return-influencing assumed support | Assumed | generic inference |
| source annotation as declaration claim | Assumption-capable contract | developer annotation |
| callable parameter inside body | Assumed unless independently stronger | callable signature / callable parameter contract |
| contextual block parameter | Assumed | contextual parameter contract |
| branch merge | weakest justified strength | flow |

The purpose is not naming consistency alone. Downstream explanation, auditing, optimization eligibility, and diagnostic interpretation can depend on origin.

---

## 7. Declaration syntax versus semantic role

One of the most important provenance transformations occurs at callable boundaries.

Given:

```phalcom
foo(value: Int) {
    value
}
```

the syntax `Int` originates as a developer-written annotation. During declaration analysis it becomes part of the callable's formal signature. During body analysis, the body consumes that signature as a contract.

Conceptually:

```text
source annotation
      ↓
resolved type form
      ↓
callable signature contract
      ↓
body-entry parameter assumption
```

The parameter read inside the body should therefore explain its usable type through the callable parameter contract, not merely by pointing back to arbitrary developer syntax.

This distinction becomes important for generated declarations, native signatures, protocol requirements, inherited contracts, and tooling that asks *why is this parameter known here?*

---

## 8. Knowledge transformation rules

### 8.1 Mapping types

A transformation that maps a known type to another canonical type must preserve status, origin, and provenance unless the semantic operation itself creates new evidence.

For example, substituting equivalent canonical type identities is not permission to promote `Assumed` to `Established`.

### 8.2 Flow joins

For reachable flow states:

```text
Established(A) + Established(B)
    -> Established(join(A, B)), origin Flow

Established(A) + Assumed(B)
    -> Assumed(join(A, B)), origin Flow

Assumed(A) + Assumed(B)
    -> Assumed(join(A, B)), origin Flow

Known(...) + Unknown(reason)
    -> Unknown(appropriate joined reason)

Known(...) + Dynamic(reason)
    -> Dynamic(appropriate dynamic reason)
```

These rules express the certainty order:

```text
Established > Assumed > no formal known proposition
```

where `Unknown` and `Dynamic` are different non-known categories rather than simply weaker known states.

### 8.3 Repetition does not create proof

Two assumptions that agree do not become established:

```text
Assumed(Int) + Assumed(Int) != Established(Int)
```

Likewise, a subtype relation successfully proven using an assumed premise does not upgrade the premise.

---

## 9. Expected context is not evidence

Suppose:

```phalcom
let users: List<User> = []
```

The annotation may provide enough context to type the otherwise context-sensitive empty list as `List<User>`. The resulting fact is justified by a real language typing rule that combines expression form with context.

By contrast, the analyzer must not implement checking by simply constructing:

```text
actual expression = Assumed(List<User>) because expected type was List<User>
```

Expected context is an input to derivation. It becomes evidence only through a semantic rule that proves or legitimately assumes something.

This distinction is specified further in `04-expression-analysis-and-contextual-typing.md`.

---

## 10. Contracts do not overwrite facts

For:

```phalcom
let n: Number = 42
```

the correct state is:

```text
contract = Number
current  = Established(Int)
relation = Validated(Int <: Number)
```

The annotation does not widen the value fact from `Int` to `Number`.

For:

```phalcom
let n: String = 42
```

the correct state is:

```text
contract = String
current  = Established(Int)
relation = Refuted(Int <: String)
```

Again, the contract does not overwrite the fact.

If the initializer has genuinely no formal evidence and the declaration is eligible to supply an assumption, then:

```text
contract = Int
current  = Assumed(Int)
basis    = developer binding contract
```

The difference between these cases must remain visible.

---

## 11. Generic inference and evidence support

A solved substitution and an established result are not the same thing.

Given:

```phalcom
class Box<T> {
    @class
    id(value: T) -> T { value }
}
```

then:

```phalcom
let x = Box.id(42)
```

may infer:

```text
T = Int
support = Established
result  = Established(Int, GenericInference)
```

But inside:

```phalcom
run(value: Int) {
    let x = Box.id(value)
}
```

if `value: Int` is only a body-entry contract assumption:

```text
T = Int
support = Assumed
result  = Assumed(Int, GenericInference)
```

The mathematical uniqueness of `T = Int` does not confer evidence authority.

Support is monotone within one inference session:

```text
Established support may weaken to Assumed
Assumed support never upgrades to Established
```

Only return-influencing inference variables determine the epistemic strength of a specialized generic result.

---

## 12. Provenance must be bounded

Phalcom requires enough provenance to explain semantic results and failures, but the hot semantic state must not become an unbounded proof graph.

The implementation should retain compact references such as:

- origin classification;
- relevant source span or semantic identity;
- parent explanation IDs;
- actual constraint origin for inference;
- support classification;
- resolved callable/declaration identity.

Rich explanation can live in a dedicated arena or graph. Every `TypeKnowledge` does not need a full duplicated proof tree.

---

## 13. Relationship to optimization and compilation

This specification does not mandate particular optimizations, but it establishes an important future boundary:

```text
Established facts
    may be eligible for proof-requiring compiler transformations

Assumed facts
    remain static-checking premises
    but are not automatically eligible for transformations that require checker-established runtime truth
```

Any optimization subsystem that consumes type evidence must state which evidence strengths it accepts.

---

## 14. External semantic contract

Consumers may rely on the following:

1. A concrete type can be observed without losing whether it is established or assumed.
2. Unknown and dynamic remain distinguishable.
3. Developer annotations cannot overwrite independently established current facts.
4. Generic inference cannot upgrade assumed support to established.
5. Constructor, callable, native, generic, contextual, and developer origins remain distinguishable where semantically relevant.
6. Flow merging never strengthens epistemic certainty.
7. Unsupported analysis remains honestly unknown/blocked rather than fabricated.
8. Formal knowledge is not contaminated by advisory observations.

---

## 15. Conformance tests

Tests should cover both algebra and composition.

Algebra tests:

```text
join(Established(Int), Assumed(Number)) -> Assumed(...)
join(Known, Unknown)                    -> Unknown
join(Known, Dynamic)                    -> Dynamic
map_type(Assumed(T))                    -> Assumed(mapped T)
```

Composition tests should assert full products. A test of a refuted annotation is incomplete if it checks only the final `TypeId`; it must check contract, current knowledge, evidence strength, consistency, causal state, and any downstream behavior whose correctness depends on those facts.

---

## Source basis

This specification is derived from the Part 1 Formal Semantic Epistemic Foundation specification and its Corrections and Amendments. The amendments take precedence on generic failure evidence, inference support, suppression-cause representation, and semantic fingerprinting. Repository implementation notes were re-grounded against `aureat/phalcom-lang` `main` at `c3b82e4b88469ef9fc79aa65a03e0bed95dc908d`; such notes are non-normative and may be updated as the code evolves.
