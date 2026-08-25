# Phalcom Semantic Analyzer Implementation Specification
## 08 — Callable Analysis, Body Entry, Return Summaries, and Publication

**Status:** Normative semantic implementation specification.

**Purpose:** Specify how callable signatures become body-entry contracts, how body analysis accumulates semantic products, how return information is summarized, and how exact callable semantics are projected back to call sites.

---

## 1. Callable analysis connects declaration semantics to expression semantics

A callable has two related but distinct semantic views:

```text
callable contract/signature
```

and:

```text
callable body analysis
```

The signature describes the callable's externally visible contract.

The body analysis describes what the implementation does and what facts were established or assumed while checking it.

Neither should overwrite the other.

---

## 2. Callable pipeline

Conceptually:

```text
declaration syntax
      ↓
resolved declaration surface
      ↓
exact callable signature
      ↓
body-entry environment
      ↓
body flow/expression analysis
      ↓
return-path collection
      ↓
normal return summary
      ↓
callable diagnostics/status/dependencies
      ↓
published CallableAnalysis
```

The exact query graph may split these phases into separate products.

---

## 3. Signature as contract

The callable signature contains formal information such as:

```text
owner
dispatch side
selector
generic parameters and kinds
parameter contracts
return contract
declared generic constraints
constructor/native/intrinsic metadata
```

A return annotation is a contract of the callable. It is not automatically proof that every arbitrary expression has that type.

At an exact call site, however, a trusted exact callable contract can establish a fixed call-result proposition according to the call-result rules below.

---

## 4. Body-entry parameters

A parameter enters the callable body through its callable contract.

Given:

```phalcom
foo(value: Int) {
    value
}
```

body entry conceptually creates:

```text
binding contract = Int
current knowledge = Assumed(Int)
assumption basis  = CallableParameterContract
origin            = CallableSignature
```

unless a language feature supplies stronger independent evidence.

The important transformation is:

```text
developer-written annotation
        ↓
callable signature
        ↓
body-entry contract
```

The body's reason for knowing `value: Int` is not merely “the developer wrote `Int`”; it is “this callable's parameter contract is `Int`.”

---

## 5. Body-entry return contract

The callable body receives a return **contract/context**, not an arbitrary value fact.

Return expressions are checked against that contract while retaining their actual knowledge.

The implementation should not model the return contract as though a value of that type already exists in flow.

---

## 6. Body-local semantic products

Callable analysis owns or publishes:

- expressions;
- bindings;
- call resolutions;
- flow graph/summaries;
- diagnostics;
- explanations;
- semantic dependencies.

Expression retrieval should normally be indexed inside the callable semantic product rather than modeled as one independent database query per AST expression unless profiling later justifies finer granularity.

This keeps query cardinality bounded while still supporting rich LSP reads.

---

## 7. Return-path collection

The analyzer records semantic knowledge from normal return paths.

This includes explicit returns and language-defined tail-expression returns where applicable.

Abrupt paths such as throw/unreachable should be modeled according to real `Never`/bottom semantics rather than as missing information.

---

## 8. Normal return summary

Return summarization operates over full `TypeKnowledge`, not merely `Option<TypeId>`.

The summary must preserve the distinctions:

```text
Known
Unknown(reason)
Dynamic(reason)
```

and within known:

```text
Established
Assumed
```

Representative behavior:

```text
Established(Int) + Established(Float)
    -> Established(Int | Float)

Established(Int) + Assumed(Number)
    -> Assumed(Int | Number)

Known + Unknown
    -> Unknown

Known + Dynamic
    -> Dynamic
```

The summary must not implement:

```text
if no concrete TypeId => Unknown(UncheckedExpression)
```

because that converts legitimate dynamic returns and classified unknown reasons into false generic unknowns.

---

## 9. Callable analysis status

Callable analysis has a body-level completion state, conceptually including:

```text
Complete
Partial
Blocked
Cancelled
BudgetExceeded
```

or an equivalent richer model.

This status summarizes body-analysis completion and is distinct from whether any individual expression has known type information.

A callable can have a useful partial product even when analysis does not fully complete.

---

## 10. Exact call-result promotion

At a call site, exact resolved callable identity can convert a trusted callable return contract into value knowledge.

This promotion is an explicit semantic operation.

The origin depends on the callable semantics.

### 10.1 Ordinary callable

```text
exact resolved callable
concrete fixed return contract Int
        ↓
Established(Int, CallableSignature)
```

assuming no other rule weakens the result.

### 10.2 Constructor

```text
exact constructor dispatch
constructor semantics establish Self
        ↓
Established(concrete receiver instance type, ConstructorSemantics)
```

Constructor provenance should not be flattened to ordinary callable-signature provenance.

### 10.3 Trusted native/intrinsic

```text
trusted native signature
        ↓
Established(return, NativeSignature)
```

or equivalent origin.

### 10.4 Generic specialized return

```text
successful generic solution
        ↓
materialized return
        ↓
support classification
        ↓
Established/Assumed(..., GenericInference)
```

This path is distinct from fixed return promotion.

---

## 11. Independent result knowledge and call invalidity

A call has multiple semantic judgments:

```text
callable identity
argument mapping
argument/parameter relations
generic constraints
return derivation
```

A failed argument relation does not necessarily destroy the return proposition.

For an exact fixed return:

```text
result knowledge = Established(Int)
call status      = Invalid(C)
invalidity       = One(C)
```

may be correct.

For a return whose type depends on failed generic inference, the result may instead be unknown.

The implementation must determine dependency rather than applying a blanket policy.

---

## 12. Call dependencies

A callable body that resolves a call depends semantically on the products that determined:

```text
resolved callable identity
callable signature/contract
generic constraints
relevant declaration hierarchy/surface
```

Those dependencies must be represented in the semantic database rather than inferred indirectly from source proximity.

This is important for invalidation when a callee's signature or constraints change.

---

## 13. Constructor/factory propagation

Consider:

```phalcom
class CellNum {
  @constructor
  new() {}

  @class
  of() {
    CellNum.new()
  }
}
```

The body of `of()` can establish `CellNum` from constructor semantics. Its return summary can then make the exact `CellNum.of()` call contract/result available to callers according to signature/body inference policy.

The explanation chain should distinguish:

```text
CellNum.new() -> ConstructorSemantics
CellNum.of()  -> callable return knowledge derived from body/signature
caller        -> exact callable result
```

rather than flattening every step to the same origin.

---

## 14. Return contracts versus inferred body returns

A declared return contract and an inferred body return are independent semantic facts.

For:

```phalcom
foo() -> Number { 1 }
```

body analysis establishes `Int` for the returned expression and validates:

```text
Int <: Number
```

The callable's external contract remains `Number` unless the language explicitly publishes narrower inferred signatures.

The body explanation may retain the narrower fact for diagnostics and internal reasoning.

---

## 15. Missing or incomplete body analysis

If body analysis is blocked, cancelled, budget-exhausted, or incomplete, callable publication must be honest.

A separately declared exact signature may still exist and be usable by callers if language semantics allow calls to depend on the signature independently of body verification.

The system must distinguish:

```text
callable contract is known
```

from:

```text
callable body has been completely verified
```

This distinction is essential for recursive declarations, native methods, separate compilation, and incremental analysis.

---

## 16. Callable-level diagnostics

Diagnostics generated in the body remain owned by their local semantic judgments.

The callable product aggregates them for publication.

Return-summary generation should not invent a generic diagnostic merely because one path was invalid if the owning expression already produced the relevant root cause.

---

## 17. Explanation publication

Callable analysis should preserve enough explanation data to answer:

- why a parameter is assumed to have its contract type;
- why a constructor result is `Self`;
- why a call result is established or assumed;
- which generic constraints determined a specialization;
- why a return path contradicts a declared contract;
- why analysis was blocked/cancelled/budgeted.

Explanation references must correspond to real semantic identities and evidence, not fabricated placeholder nodes.

---

## 18. External behavior guarantees

Consumers may rely on:

- body parameters being represented as callable-contract assumptions;
- return contracts remaining contextual contracts rather than current value facts;
- normal-return summaries preserving `Dynamic` and classified `Unknown`;
- callable status being distinct from return knowledge;
- exact fixed call results being promoted through an explicit semantic rule;
- constructor/native/generic provenance remaining distinguishable;
- independent fixed returns surviving unrelated invalid generic/argument judgments;
- body and signature products having explicit dependency relationships.

---

## 19. Required regression families

### Body entry

- annotated parameter becomes assumed body-entry knowledge with callable-parameter basis;
- contextual parameter remains contextual assumption;
- no accidental `Established` promotion.

### Return checking

- narrower established expression validates broader return contract without widening expression fact;
- incompatible return preserves actual expression knowledge and owns diagnostic;
- dynamic return summary remains dynamic;
- classified unknown reason survives summary.

### Call result

- ordinary fixed return origin;
- constructor result origin;
- trusted native result origin;
- generic established result;
- generic assumed result;
- fixed generic-independent return under invalid inference.

### Callable status

- cancellation and budget at body level remain distinguishable;
- partial body publication does not masquerade as complete verification.

---

## Source basis

This specification is derived from the Part 1 Formal Semantic Epistemic Foundation specification and its Corrections and Amendments. The amendments take precedence on generic failure evidence, inference support, suppression-cause representation, and semantic fingerprinting. Repository implementation notes were re-grounded against `aureat/phalcom-lang` `main` at `c3b82e4b88469ef9fc79aa65a03e0bed95dc908d`; such notes are non-normative and may be updated as the code evolves.
