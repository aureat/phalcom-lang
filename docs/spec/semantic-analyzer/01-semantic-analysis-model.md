# Phalcom Semantic Analyzer Implementation Specification
## 01 — Semantic Analysis Model

**Status:** Normative semantic implementation specification.

**Purpose:** Define the analyzer-wide semantic machine: the products it computes, the authority and ownership boundaries between subsystems, the internal information-flow model, and the externally observable behavior that all subsystem implementations must preserve.

**Scope:** This document specifies the conceptual implementation model and externally observable contract of Phalcom's compiler-owned semantic analyzer. It is intentionally more concrete than the language-level typing specification and intentionally less concrete than the Rust source. It defines what semantic information exists, which subsystem owns it, how it flows through analysis, and what consumers may rely on.

**Normative sources:** *Phalcom Semantic Correctness / Single-World Takeover — Part 1: Formal Semantic Epistemic Foundation* together with *Part 1 Corrections and Amendments*. Where those documents differ, the amendments take precedence.

**Repository grounding:** The implementation shape referenced by non-normative notes was reviewed against `aureat/phalcom-lang` through `main` at `c3b82e4b88469ef9fc79aa65a03e0bed95dc908d` (`fix(semantic): preserve generic evidence and advisory agreement`, 2026-08-25). File names and concrete types may evolve; the semantics specified here do not depend on those names.

---

## 1. Purpose of the semantic analyzer

Phalcom's semantic analyzer is not merely an accept/reject type checker. It is the compiler-owned system that interprets source programs into a persistent semantic model that can be consumed by compilation, diagnostics, explanation, semantic tooling, and later advisory analysis. Its central responsibility is to preserve the distinction between facts that have been established, assumptions supplied by language contracts, contextual constraints, contradictions, dynamic boundaries, incomplete analysis, and invalidity.

The analyzer therefore answers several different questions at once:

1. **What does this source construct denote?**
2. **What type knowledge is available for its runtime value, if any?**
3. **How strong is that knowledge, and what evidence supports it?**
4. **Did the analyzer complete the semantic operation?**
5. **Is the result causally dependent on an invalid upstream operation?**
6. **Which callable, declaration, field, binding, or type-level object was resolved?**
7. **Which semantic constraints or judgments were applied?**
8. **Which diagnostics own failures, and which later nodes merely depend on those failures?**
9. **Which parts of the result are semantically significant for incremental reuse?**

No single enum or `TypeId` answers all of these questions. The analyzer is correct only when these dimensions remain distinct through analysis and publication.

---

## 2. The semantic product model

At the expression level, the conceptual semantic product is:

```text
ExpressionSemanticResult
├── type knowledge
├── analysis status
├── causal invalidity
├── semantic denotation
├── resolved callable / dispatch identity, where applicable
├── evidence origin and provenance
├── constraint / judgment evidence
└── explanation dependencies
```

The concrete Rust implementation may package these fields in more than one internal object before publication. That representation is not normative. The normative requirement is that every semantically meaningful dimension produced during analysis survives until the published semantic product or an explicitly documented higher-level summary that preserves equivalent information.

At the binding level, the conceptual product is:

```text
BindingSemanticState
├── stable binding identity
├── persistent assignment contract, if any
├── current flow-sensitive type knowledge
├── contract/current consistency
├── mutability
├── current denotation
├── causal invalidity
├── version / flow generation where required
└── explanation / provenance references
```

At the callable level, the product is:

```text
CallableSemanticProduct
├── callable identity and signature
├── body-local expression products
├── body-local binding products
├── call resolutions
├── flow graph / flow summaries
├── return summary
├── diagnostics and causal ownership
├── explanations
├── semantic dependencies
└── callable analysis status
```

The workspace/snapshot layer publishes these products without changing their semantic meaning. Presentation and LSP layers may project or summarize them, but must not retroactively alter formal analysis.

---

## 3. Formal analysis and advisory analysis

Phalcom distinguishes compiler-owned **formal semantic knowledge** from advisory observations or heuristics.

Formal knowledge is eligible to participate in hard semantic judgments such as assignability, binding-contract validation, return checking, generic constraints, and compiler acceptance. Advisory information is observational. It may improve presentation or offer a more specific runtime-shape hypothesis, but it cannot become formal evidence merely because it agrees with a likely runtime shape.

The dependency direction is:

```text
source + language semantics
        ↓
formal semantic analyzer
        ↓
formal semantic products
        ↓
advisory projection / advisory comparison
        ↓
presentation / LSP
```

The reverse direction is prohibited:

```text
advisory observation
        ✗
formal TypeKnowledge
        ✗
hard compiler rejection
```

When formal and advisory information disagree, the formal product remains unchanged. Advisory disagreement can be reported or visualized only through an explicitly advisory product. A `Ready` formal result with concrete known type may be compared against an advisory shape; non-ready, unknown, or dynamic formal states are not upgraded by advisory information.

---

## 4. Semantic identity and ownership boundaries

### 4.1 Source identity

Declarations, callables, expressions, bindings, and source occurrences require semantic identities appropriate to their lifetime. Identity is not interchangeable with source range. Ranges are presentation coordinates; semantic identity determines which product, dependency, or flow entity is being discussed.

### 4.2 Type identity

`TypeId` denotes an interned formal type in the semantic type store. It is not an epistemic object. The type `Int` is the same type regardless of whether knowledge of `Int` is established, assumed, contextual, or advisory.

Therefore the following must never be represented as distinct canonical types:

```text
Established Int
Assumed Int
Expected Int
Advisory Int
```

Those distinctions belong outside the `TypeId`.

### 4.3 Flow ownership

Flow-sensitive current binding facts have one semantic owner: the flow state used by body analysis. Lexical scope maps names to binding identity and declaration metadata; it does not independently own a competing current value fact.

A publication index may mirror flow facts for read-side access, but a mirror is not allowed to become an independent semantic authority.

### 4.4 Query/database ownership

Incremental query products are semantic authority for the products they own. Mutable helper caches, presentation caches, advisory engines, and LSP-local mirrors are never alternate sources of truth for formal semantic results.

---

## 5. The analyzer pipeline

A callable body is conceptually analyzed in the following order:

```text
declaration/surface products
        ↓
exact callable signature
        ↓
body-entry environment
        ↓
initial flow state
        ↓
statement/expression transfer
        ↓
relations + calls + inference
        ↓
flow joins / loop fixpoint
        ↓
return-path summarization
        ↓
callable status + diagnostics + explanations
        ↓
published callable semantic product
```

This is a conceptual dependency order, not a required call stack.

### 5.1 Signature formation

Declaration and annotation processing produces callable contracts. This stage establishes what the callable promises, not what its implementation has already proved about each runtime invocation.

### 5.2 Body entry

The body consumes the callable contract. Parameters therefore enter the body as contract-backed assumptions unless independent checker evidence establishes more.

A source annotation may have originated from developer syntax, but once incorporated into a validated callable signature its semantic role at body entry is a callable parameter contract. Provenance must distinguish these stages.

### 5.3 Expression analysis

Expression analysis may synthesize a result from syntax and semantics, or check under expected context. Expected context can guide a derivation; it is not by itself value evidence.

### 5.4 Judgment application

Assignability, subtype, equality, kind, generic, and contract judgments return structured semantic outcomes. Consumers translate those outcomes into diagnostics, status, consistency, or inference results without erasing terminal distinctions.

### 5.5 Flow transfer

Statements change flow-sensitive current facts. Persistent contracts remain stable unless the language rule itself creates or replaces the binding.

### 5.6 Join and fixpoint

Branches join current facts conservatively. Loops iterate toward a fixpoint or an explicit bounded/non-success outcome. Flow operations may weaken knowledge but may not invent stronger evidence.

### 5.7 Publication

Publication exposes the result of analysis. It does not reconstruct facts that should have been preserved internally. In particular, expression status must not be guessed from type knowledge, diagnostic overlap, or causal invalidity after the fact.

---

## 6. Internal behavior versus external behavior

This specification uses two complementary contracts.

### 6.1 Internal semantic behavior

Internal behavior describes how semantic information must be represented and transformed so that the analyzer remains coherent. Examples include:

- expected types remain contextual constraints rather than becoming value facts;
- a binding contract and current flow fact are separate objects;
- causal invalidity propagates independently from analysis completion;
- generic inference records the actual constraint that failed;
- epistemic support weakens monotonically;
- flow joins cannot select an arbitrary incoming fact;
- semantic product fingerprints omit incidental allocator identities.

These requirements may constrain implementation architecture because violating them makes correct external behavior impossible or fragile.

### 6.2 External observable behavior

External behavior describes what a caller of the semantic model is entitled to observe. Examples include:

- a refuted annotation does not overwrite an established initializer type;
- an invalid call can retain an independently known result type;
- a downstream call may remain `Ready` even when its receiver carries causal invalidity;
- `Dynamic` is published as dynamic rather than unknown;
- cancellation is distinguishable from a type contradiction;
- an inferred generic result depending on assumed input is itself assumed;
- changing only a local diagnostic cause number does not make a semantic product different.

External behavior is the basis for source-level integration tests and LSP/compiler consumers.

---

## 7. Information preservation as the governing law

The analyzer should prefer retaining orthogonal facts over replacing one with another.

Consider:

```phalcom
let x: Int = CellNum.new()
```

The analyzer has at least two independent propositions:

```text
developer contract: x must satisfy Int
checker fact:       initializer is CellNum
```

A failed relation between those propositions produces a third fact:

```text
CellNum <: Int is refuted
```

Correct recovery preserves all three:

```text
contract      = Int
current       = Established(CellNum)
consistency   = Refuted(...)
invalidity    = One(C1)
```

Incorrect recovery destroys information by choosing one proposition as a substitute for another:

```text
current = Int      # annotation overwrote fact
current = Unknown  # contradiction erased fact
```

This information-preservation law applies throughout the analyzer:

- declarations do not overwrite facts;
- contradictions do not become types;
- expected context does not become evidence;
- dynamic boundaries do not become unknown;
- cancellation does not become blocked;
- advisory observations do not become formal knowledge;
- provenance does not collapse to whichever source span is easiest to retain.

---

## 8. Fail-closed incompleteness

Semantic completeness and semantic correctness are separate requirements.

Partially implemented syntax or an unavailable semantic prerequisite may yield:

```text
Unknown(reason)
Blocked(reason)
Dynamic(reason)
Cancelled
BudgetExceeded
```

as appropriate.

It must not yield a fabricated ordinary type merely to keep analysis moving.

Real language types such as `Unit`, `Never`, `Object`, or a first generic argument are never substitutes for missing analyzer information.

This gives Phalcom a safe completeness strategy:

```text
unsupported but honest  >  apparently precise but fabricated
```

A later completeness project can replace `Unknown(UncheckedExpression)` with a real rule without changing the epistemic model.

---

## 9. Recovery and useful analysis of invalid programs

Phalcom deliberately continues semantic analysis after recoverable contradictions when required premises remain available.

The canonical example is:

```phalcom
let x: Int = CellNum.new()
let y = x.cellOnly()
```

The binding declaration is invalid because its current value contradicts its persistent contract. Nevertheless the runtime-value fact remains `CellNum`, so dispatch on `x.cellOnly()` is still analyzable.

The correct conceptual result is:

```text
x:
    contract       Int
    current        Established(CellNum)
    consistency    Refuted
    invalidity     One(C1)

x.cellOnly():
    receiver       Established(CellNum)
    dispatch       CellNum.cellOnly
    result         Established(Int)
    status         Ready
    invalidity     One(C1)
```

The downstream operation is causally connected to invalid source, but it is not suppressed because the semantic premise required for dispatch is still present.

This distinction is fundamental to diagnostics, hover, completion, navigation, and future formal explanation.

---

## 10. Analyzer terminal states

Analysis can terminate without producing an ordinary successful judgment for reasons that are not type contradictions. At minimum, the implementation model preserves:

```text
Ready
Invalid
Suppressed
Blocked
DynamicBoundary
Cancelled
BudgetExceeded
InternalFailure
```

These states describe the semantic operation, not the type value.

A concrete known type may coexist with some non-ready states when the type is independently known. For example, a fixed callable return may remain known even if an argument relation makes the invocation invalid.

Consumers must not infer status solely from whether a `TypeId` exists.

---

## 11. Explanation and provenance

Explanations are derived from actual semantic evidence. They must not manufacture a cleaner story than the analyzer actually used.

An explanation should be able to distinguish:

```text
Int because literal semantics established it
CellNum because @constructor establishes Self
Result<Int, E> because generic inference solved T from established evidence
Int because callable body entry assumes the parameter contract
```

A future presentation layer may render these explanations differently, but it must not collapse the underlying origin or strength.

---

## 12. Incremental semantics

Incrementality is part of semantic correctness because cached products can otherwise publish stale meaning.

A semantic product fingerprint must change when downstream-observable semantics change. It must not change merely because incidental allocation or source presentation changed.

The analyzer therefore distinguishes:

```text
semantic identity
source/input identity
ephemeral allocator identity
```

The details are specified in `09-semantic-products-incrementality-and-fingerprints.md`.

---

## 13. Conformance expectations

An implementation conforming to this model must support tests at two layers.

Internal semantic tests should directly exercise:

- knowledge joins;
- flow joins;
- binding reconciliation;
- relation outcome propagation;
- inference failure/support;
- return summaries;
- semantic fingerprint equivalence.

Source-level composition tests should analyze real Phalcom programs and inspect published semantic products. These tests must assert more than final `TypeId` when the behavior under test includes status, causality, provenance, consistency, dispatch, or incremental identity.

A refactor is conforming when these contracts remain true even if internal types and helper functions change substantially.

---

## 14. Non-goals

This document does not specify:

- every AST variant Phalcom will eventually support;
- exact Rust struct layout;
- exact enum names where semantics remain equivalent;
- query granularity below the existing callable/declaration product boundaries unless semantic identity requires it;
- machine-code lowering;
- optimization rules that consume established evidence;
- complete Part 2/3 LSP migration.

The purpose is to specify the semantic machine, not freeze its current implementation syntax.

---

## Source basis

This specification is derived from the Part 1 Formal Semantic Epistemic Foundation specification and its Corrections and Amendments. The amendments take precedence on generic failure evidence, inference support, suppression-cause representation, and semantic fingerprinting. Repository implementation notes were re-grounded against `aureat/phalcom-lang` `main` at `c3b82e4b88469ef9fc79aa65a03e0bed95dc908d`; such notes are non-normative and may be updated as the code evolves.
