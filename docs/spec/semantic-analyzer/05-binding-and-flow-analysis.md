# Phalcom Semantic Analyzer Specification
## 05 — Binding and Flow Analysis

**Status:** Normative semantic-analyzer specification.

**Purpose:** Specify the semantic state of bindings and the behavior of declaration, initialization, reassignment, branch merge, loop analysis, widening, and flow publication.

---

## 1. Binding semantics are multi-dimensional

A binding is not represented semantically by a single type.

The conceptual binding state is:

```text
BindingState
├── BindingId
├── source name / declaration metadata
├── persistent contract: Option<BindingContract>
├── current flow knowledge: TypeKnowledge
├── consistency: BindingConsistency
├── mutability
├── denotation
├── causal invalidity
├── flow version/generation where useful
└── explanation/provenance
```

The central distinction is:

```text
persistent contract != current flow knowledge
```

The contract describes what values are permitted to be assigned. Current knowledge describes what the analyzer currently knows the runtime value to be.

---

## 2. Binding identity

`BindingId` identifies one semantic local binding.

A lexical name lookup resolves to a binding identity. The same source name can correspond to different identities in different scopes.

The same `BindingId` across incoming control-flow states denotes the same binding. Therefore declaration-stable properties associated with that identity—especially its persistent contract and mutability—must not silently disagree at a flow join.

Such disagreement indicates an analyzer invariant failure or malformed flow construction, not ordinary user-level type uncertainty.

`BindingId` is body-analysis identity. `SourceSiteId` is source-site identity. Their correspondence must be published explicitly and canonically; equality of spelling, source range, or nearest declaration is not a valid attachment algorithm. Identity lifetimes and attachments are specified in `10-semantic-identity-source-sites-and-attachments.md`.

---

## 3. Persistent contracts

A binding contract contains at least:

```text
contract type
contract origin
```

Representative origins:

```text
SourceAnnotation
InferredInitializer
CallableParameter
ContextualParameter
other explicit language contract
```

### 3.1 Explicit annotation contract

```phalcom
let x: Number = 1
```

creates a persistent `Number` contract.

### 3.2 Inferred initializer contract

Under current Phalcom policy, an unannotated binding whose initializer yields usable known type information may acquire a monomorphic inferred contract:

```phalcom
let x = 1
x = "text"
```

The inferred initializer contract preserves current monomorphic reassignment behavior without pretending that the developer wrote an explicit type annotation.

If Phalcom later adopts type-changing unannotated bindings, this policy can change by ceasing to create `InferredInitializer` contracts. The contract/current representation itself remains valid.

### 3.3 No contract

A binding with genuinely no persistent contract remains unconstrained by prior current values.

Transient flow knowledge is not implicitly promoted into a new contract.

---

## 4. Current flow knowledge

Current knowledge represents the most precise sound value-type fact available at the current program point.

Examples:

```text
Established(Int)
Assumed(Number)
Unknown(NoTypeEvidence)
Dynamic(...)
```

A current fact may be narrower than the persistent contract.

The analyzer should preserve narrower facts because they enable precise downstream dispatch and diagnostics.

---

## 5. Binding consistency

Consistency expresses the relation between current knowledge and persistent contract.

Conceptually:

```text
Unconstrained
Validated
Assumed(basis)
Refuted(failure evidence)
DynamicBoundary
Blocked(reason)
...
```

The exact representation may preserve terminal relation states separately from binding consistency; what matters is that relation outcomes are not destroyed.

Consistency is not a replacement type.

---

## 6. Initialization state machine

For:

```phalcom
let x: T = initializer
```

the analyzer performs:

```text
resolve persistent contract T
        ↓
analyze initializer under legitimate expected context
        ↓
retain initializer's actual knowledge
        ↓
relate actual knowledge to contract
        ↓
create binding state
```

### 6.1 Compatible established initializer

```text
contract    = Number
initializer = Established(Int)
relation    = Assignable
```

produces:

```text
current     = Established(Int)
consistency = Validated
```

### 6.2 Refuted established initializer

```text
contract    = String
initializer = Established(Int)
relation    = Refuted
```

produces:

```text
current     = Established(Int)
consistency = Refuted(...)
invalidity  = One(C)
```

The contradiction does not replace current knowledge.

### 6.3 Genuine no-evidence initializer

If initializer knowledge is an explicitly assumption-eligible no-evidence state and an explicit contract exists:

```text
current     = Assumed(contract type)
consistency = Assumed(developer binding contract)
```

A checker coverage failure is not eligible for this conversion.

---

## 7. `let`, `const`, and mutability

Binding mutability is a declaration property.

`const` without required initialization must produce the appropriate diagnostic and must not fabricate a usable value type.

Illegal writes to immutable bindings:

- own an immutable-write diagnostic;
- do not mutate the binding's current state;
- do not silently proceed as though the write succeeded.

The binding's previous current fact remains the current fact because runtime mutation is not semantically accepted.

---

## 8. Reassignment state machine

For:

```phalcom
x = rhs
```

the analyzer performs:

```text
resolve BindingId
        ↓
verify mutability
        ↓
analyze RHS
        ↓
check RHS against persistent contract, if any
        ↓
update current fact according to write semantics
        ↓
update consistency / invalidity
```

### 8.1 Contracted mutable binding

If `x` has a persistent contract, reassignment is checked against that contract.

The previous current type is not the assignment rule.

### 8.2 Unconstrained mutable binding

If `x` has no persistent contract, previous current knowledge must not be treated as a persistent expected type merely because it is available.

Any type-changing behavior is determined by the language's binding policy, not by accidental reuse of flow knowledge as context.

### 8.3 Refuted mutable write

When a mutable write violates the persistent contract, recovery should preserve the actual new RHS knowledge as the current runtime-value fact while recording contradiction/invalidity, provided the language's error-recovery model treats the write as the value that would occur.

This allows subsequent analysis to reflect the program actually written rather than pretending the invalid assignment never affected value flow.

Immutable-write recovery is different: the write is not semantically permitted to mutate the binding.

### 8.4 Binding transition matrix

| Persistent contract | RHS/current knowledge | Relation outcome | Resulting current knowledge | Consistency | Operation status / cause |
|---|---|---|---|---|---|
| none | known/assumed | not required | actual RHS knowledge | unconstrained | preserve RHS status/cause |
| present | known | `Assignable` | actual RHS knowledge | validated | ready plus upstream causes |
| present | known | `Refuted` | actual RHS knowledge for permitted mutable write | refuted | invalid with owning cause |
| present | assumption-eligible unknown | assumption rule applies | `Assumed(contract)` | assumed | ready or causally non-clean as justified |
| present | non-eligible unknown | `Uncertain`/blocked | preserve honest unknown | unresolved | propagate structured outcome |
| present | dynamic | `DynamicBoundary` | dynamic | runtime-dependent | dynamic boundary |
| any | blocked/cancelled/budget/internal | matching terminal outcome | retain only independent prior/result facts | unresolved | propagate terminal outcome unchanged |

Declaration initialization and reassignment may refine individual cells where their language semantics differ. They must preserve the same contract/current/consistency/status dimensions and may not collapse a terminal outcome into successful assignment.

---

## 9. Lexical scope and flow state

Lexical scope owns:

```text
name -> BindingId
declaration metadata needed for lookup
```

Flow state owns:

```text
BindingId -> current semantic state
```

The analyzer must avoid independent mutable current-value stores in both scope and flow.

A read-side publication index can cache or mirror facts but must be derived from the single semantic owner.

---

## 10. Branch flow

For:

```phalcom
if condition {
    ...
} else {
    ...
}
```

each branch starts from a compatible incoming state and produces an outgoing state.

The merge:

1. keeps only bindings that are reachable/meaningful under the language's scope rules;
2. verifies declaration-stable invariants for matching `BindingId`s;
3. joins current type knowledge;
4. joins denotation conservatively;
5. joins causal invalidity;
6. recomputes or conservatively joins contract consistency;
7. preserves mutability exactly.

---

## 11. Type-knowledge join

The flow join must be deterministic and epistemically monotone.

Representative semantics:

```text
Established(Int) + Established(Float)
    -> Established(Int | Float)

Established(Int) + Assumed(Number)
    -> Assumed(Int | Number)

Established(Int) + Unknown(R)
    -> Unknown(joined R)

Established(Int) + Dynamic(D)
    -> Dynamic(joined D)
```

The implementation must not choose the first known branch merely because it is easy.

If the type system represents unions canonically, union order must not depend on map/hash iteration order.

---

## 12. Contract invariants at join

For the same `BindingId`, incoming persistent contracts must represent the same semantic contract.

This state:

```text
branch A: BindingId(7), contract Int
branch B: BindingId(7), contract String
```

cannot legitimately mean:

```text
joined contract = None
```

because `None` means the binding is unconstrained, which is a weaker and potentially fail-open state.

Instead, contract disagreement is an internal semantic invariant failure. The analyzer must preserve fail-closed behavior and enough information to diagnose or report the internal inconsistency.

Likewise, mutability disagreement for the same binding identity is not ordinary flow uncertainty.

---

## 13. Consistency after join

Joined current knowledge may differ from every individual incoming current fact.

Therefore consistency must describe the relation between the **joined** current state and the persistent contract.

It cannot simply copy a branch's previous consistency.

For example:

```text
contract Number
branch A current Established(Int), validated
branch B current Established(Float), validated
```

may join to:

```text
current Established(Int | Float)
```

and the analyzer must establish that the joined fact remains compatible with `Number`.

---

## 14. Loops and fixpoints

Loop analysis conceptually performs:

```text
entry state
    ↓
body transfer
    ↓
back-edge state
    ↓
join with loop header
    ↓
repeat until stable
```

If exact convergence is too expensive or impossible under configured limits, the analyzer may widen.

Widening is a semantic transformation of current flow knowledge, not permission to replace current knowledge with the declared contract.

After widening, binding invariants must be re-established:

```text
contract preserved
mutability preserved
current knowledge widened
consistency reconciled against widened current
causal invalidity joined
```

A widening operation that changes current knowledge while leaving stale consistency is incomplete.

---

## 15. Loop non-convergence

If analysis cannot reach a stable state within the configured analysis strategy, it fails explicitly:

```text
Blocked(RecursiveFixpoint)
BudgetExceeded(...)
```

or another precise terminal outcome.

It does not fabricate:

```text
current = declaration type
```

merely to obtain convergence.

---

## 16. Flow denotation

Denotation should join conservatively.

If all reachable states agree on a semantically meaningful denotation, it may be retained. If they disagree in a way for which no meaningful joined denotation exists, the merged denotation becomes unknown/absent rather than arbitrarily selecting one branch.

This is separate from type join.

---

## 17. Causal invalidity in flow

Causal invalidity joins independently from type knowledge.

A branch that is semantically analyzable but causally invalid can contribute a precise type to a merge while marking the merged state causally dependent.

Example:

```text
branch A: Established(Int), One(C1)
branch B: Established(Float), Clean
```

may yield:

```text
Established(Int | Float), One(C1)
```

not suppression.

---

## 18. Flow summaries

A flow summary is a semantic boundary, not merely a debugging convenience.

If downstream consumers can distinguish:

```text
Established(Int)
Assumed(Int)
Unknown(...)
Dynamic(...)
contract origin
consistency
causal state
```

then any summary used for dependency tracking or semantic product identity must preserve the required distinctions directly or through another guaranteed part of the product.

A summary that stores only:

```text
BindingId -> TypeId
```

is insufficient when epistemic strength can affect downstream behavior.

---

## 19. External behavior guarantees

Consumers may rely on:

- binding contracts remaining stable across ordinary writes and flow;
- current knowledge being allowed to remain narrower than a contract;
- refuted contracts preserving actual current knowledge;
- unannotated inferred contracts being distinguishable from explicit annotations;
- previous current values not acting as hidden persistent contracts;
- branch joins never strengthening epistemic certainty;
- same-identity contract disagreements failing closed;
- loop widening preserving and rechecking binding invariants;
- immutable illegal writes not mutating flow state;
- causal invalidity joining independently from knowledge.

---

## 20. Required regression families

### Initialization

- compatible annotation preserves narrower established initializer;
- incompatible annotation preserves initializer and marks `Refuted`;
- genuine no-evidence + annotation creates assumption;
- checker coverage gap + annotation does not become assumption.

### Assignment

- validates against persistent contract;
- does not validate against old current type;
- invalid mutable write preserves actual RHS current fact for recovery;
- invalid immutable write preserves old state.

### Branches

- established + established;
- established + assumed;
- known + unknown;
- known + dynamic;
- denotation disagreement;
- causal invalidity aggregation;
- divergent same-ID contract fails closed.

### Loops

- stable loop reaches fixpoint;
- widening preserves contract and recomputes consistency;
- non-convergence becomes explicit blocked/budget outcome;
- loop-only local bindings do not leak incorrectly.

### Incremental flow identity

- epistemic status changes alter semantic identity where observable;
- incidental ordering changes do not.
