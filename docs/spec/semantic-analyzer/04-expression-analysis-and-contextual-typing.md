# Phalcom Semantic Analyzer Implementation Specification
## 04 — Expression Analysis and Contextual Typing

**Status:** Normative semantic implementation specification.

**Purpose:** Specify the common expression-analysis pipeline, synthesis versus checking, contextual expectations, nested-result propagation, and publication of expression semantics.

---

## 1. Expression analysis is a result-rich operation

Expression analysis is conceptually:

```text
analyze(expression, expected-context, flow, semantic environment)
    -> ExpressionSemanticResult
```

The result is richer than a type.

It may contain:

```text
knowledge
analysis status
causal invalidity
denotation
resolved callable
explanation
constraints
semantic dependencies
```

The analyzer should not require a later publication step to infer information that was already known during expression processing.

---

## 2. Synthesis and checking

Phalcom uses bidirectional typing concepts.

Synthesis:

```text
Γ ; Φ ⊢ e ⇒ K
```

means the analyzer derives knowledge `K` from the expression and environment.

Checking:

```text
Γ ; Φ ⊢ e ⇐ E
```

means the analyzer analyzes `e` under an expected semantic context `E`.

In implementation, both modes may share one API and one result type. The key distinction is semantic:

> Expected context constrains or guides the derivation; it is not automatically evidence about the value.

---

## 3. Expected-context model

Expected context should preserve at least:

```text
none
proper expected type + origin
inference term + origin
```

Representative origins include:

```text
binding annotation
assignment contract
call parameter
callable return contract
collection element
tuple/record field
block/callable context
generic expected result
```

The origin matters because a context can have different semantic effects and explanation requirements even when the expected `TypeId` is the same.

---

## 4. Context may guide synthesis

Some expression forms are intentionally context-sensitive.

Example:

```phalcom
let users: List<User> = []
```

Without context, `[]` may not uniquely establish an element type.

With the binding contract:

```text
expected = List<User>
```

the collection typing rule may derive:

```text
[] : List<User>
```

provided the language rule explicitly permits contextual typing of empty collections.

The semantic evidence comes from the combination of:

```text
expression form + valid context rule
```

not from pretending the expected type was already the actual value fact.

---

## 5. Context must not overwrite independently synthesized facts

Example:

```phalcom
let x: Number = 1
```

The literal independently establishes:

```text
1 : Int
```

The annotation provides expected context `Number`.

Checking validates:

```text
Int <: Number
```

but the expression result remains:

```text
Established(Int)
```

It is not rewritten to `Number`.

If the relation is refuted:

```phalcom
let x: String = 1
```

the literal remains `Established(Int)` and the owning judgment becomes invalid.

---

## 6. Context origin must be legitimate

Expected context may originate only from a real semantic contract or contextual rule.

For assignment:

```phalcom
x = rhs
```

the RHS may be checked against `x`'s persistent assignment contract.

The analyzer must not manufacture an assignment expectation from the previous current flow fact when the binding has no persistent contract.

Otherwise this sequence:

```phalcom
let x = <unconstrained or contract-free value>
...
x = []
```

could accidentally let the old current value determine the type of a context-sensitive RHS, turning transient flow knowledge into a hidden persistent contract.

This violates the binding model.

---

## 7. Expression-result preservation

Every expression-specific analyzer should preserve the independent dimensions returned by its children.

A generic shape is:

```text
analyze children
      ↓
collect knowledge / status / invalidity / denotation
      ↓
perform expression-specific semantic operation
      ↓
derive own result
      ↓
join causes and terminal outcomes
      ↓
publish
```

A parent should not blindly copy child invalid status. It should determine whether:

- the child supplied enough semantic information for the parent operation;
- the parent owns a new failure;
- the parent is merely causally dependent;
- the parent is genuinely suppressed.

---

## 8. Variable reads

A local-variable read obtains current knowledge from flow state.

Conceptually:

```text
binding identity
      ↓
FlowState.current
      ↓
variable expression knowledge
```

The variable read also receives appropriate denotation and causal invalidity.

If the binding is inconsistent with its declaration but still has precise current knowledge, the read remains analyzable:

```text
binding.current     Established(CellNum)
binding.invalidity  One(C1)
        ↓
read x
        ↓
knowledge           Established(CellNum)
status              Ready
invalidity          One(C1)
```

The read is not automatically suppressed.

---

## 9. Calls and method dispatch

A call expression consists of several semantically distinct stages:

```text
analyze receiver/callee
        ↓
resolve callable identity
        ↓
map arguments to parameters
        ↓
analyze/check arguments
        ↓
perform generic inference if required
        ↓
derive return knowledge
        ↓
combine status/invalidity
        ↓
publish call resolution + result
```

Resolution and return derivation may produce knowledge independent of argument validity.

For example, an exact non-generic callable can have a fixed return type even if one argument relation is refuted.

The call result should therefore keep:

```text
knowledge = exact return fact
status    = Invalid(C)
```

rather than discarding the return type.

---

## 10. Constructor calls

`@constructor` has semantic meaning beyond an ordinary method declaration.

When exact constructor dispatch succeeds, the constructor return derives from constructor semantics—typically `Self` instantiated for the receiver/class—not merely from a textual return annotation.

The expression product should preserve that origin for explanation.

This matters even if constructor and ordinary callable returns currently share code paths.

---

## 11. Member access, operators, indexing, and protocol desugaring

Expression forms that lower to callable or protocol semantics should preserve the resolved semantic identity and the evidence path that established the result.

A convenience lowering must not reduce a rich relation or dispatch result to a boolean if terminal outcomes can occur.

The final expression result should distinguish:

```text
dispatch succeeded with fixed return
dispatch blocked
dynamic dispatch boundary
dispatch invalid due to relation
resolution unavailable
```

---

## 12. Branch expressions

Branch result synthesis uses the shared type-knowledge join rather than choosing a representative branch.

Conceptually:

```text
if cond {
    A
} else {
    B
}
```

produces:

```text
join(knowledge(A), knowledge(B))
join(causal(A), causal(B))
appropriate aggregate status
```

A known branch does not dominate a reachable unknown branch.

An assumed branch prevents the merged result from being stronger than assumed.

---

## 13. Collections and structural literals

List, set, tuple, map, record, and related expression forms must fail honestly when required component information is unavailable.

They may not fill missing information with real language sentinels such as:

```text
Unit
Never
Object
```

unless the language typing rule genuinely yields that type.

Contextual synthesis is allowed only through explicit context rules.

---

## 14. Return expressions

A return expression may be checked against the callable's persistent return contract.

The return contract should be represented as contract/context, not as arbitrary current `TypeKnowledge`.

The expression itself retains its actual knowledge.

The relation between actual return knowledge and the callable contract determines diagnostic/status consequences.

This separation becomes especially important when the returned expression is narrower than the declared return type or when the declared contract is incompatible.

---

## 15. Terminal outcomes in nested expression analysis

Nested analysis may encounter:

```text
Blocked
Cancelled
BudgetExceeded
DynamicBoundary
InternalFailure
```

These outcomes are not equivalent to successful checking.

An expression-analysis API must preserve them to its caller.

A helper whose effective behavior is:

```text
Refuted => false
everything else => true
```

is semantically insufficient because it erases the reason analysis did not prove the relation.

Consumers need structured outcomes.

---

## 16. Causal propagation through argument and receiver dependencies

Calls, operators, indexing, and member access should aggregate causal invalidity from semantically used receiver and argument values.

However, the parent becomes suppressed only if a required result is missing because of upstream invalidity.

If the receiver remains `Established(CellNum)` with causal invalidity, method dispatch remains possible.

---

## 17. Denotation and value type are distinct

An expression may denote:

```text
runtime value
class object
type object / reflected type
callable
field
module/member
```

depending on the language model.

The expression analyzer should preserve semantic denotation independently from type knowledge where the distinction is relevant.

A class object having a runtime type is not the same proposition as the nominal type denoted by that class object.

---

## 18. Expression explanations

Expression explanation should correspond to the operation actually performed.

Representative explanation steps include:

```text
literal semantics
binding read
method/call resolution
constructor semantics
generic specialization
flow join
contextual collection typing
```

A generic fallback explanation such as “literal” for unrelated expression forms is not semantically adequate for a mature presentation layer.

Explanation quality is not allowed to change the formal result, but it should faithfully expose the result's real status, origin, and semantic parents.

---

## 19. Publication contract

A published `ExpressionAnalysis` or equivalent read model must expose enough information for downstream consumers to answer:

- what type knowledge is available?
- what is its evidence strength/origin?
- did analysis complete?
- is the node invalid or merely causally dependent on invalid source?
- was analysis suppressed or blocked?
- what does the expression denote?
- what callable was resolved?
- what explanation supports the result?

If some information lives indirectly in an arena, the product must carry stable references sufficient to retrieve it.

---

## 20. External behavior examples

### Narrower value under broader contract

```phalcom
let n: Number = 1
```

Expected:

```text
initializer knowledge = Established(Int)
binding contract       = Number
binding current        = Established(Int)
consistency            = Validated
```

### Refuted contract

```phalcom
let n: String = 1
```

Expected:

```text
initializer knowledge = Established(Int)
binding current        = Established(Int)
consistency            = Refuted
owning diagnostic      = one mismatch
```

### Context-sensitive empty collection

```phalcom
let users: List<User> = []
```

Expected:

```text
[] analyzed under List<User> context
result derived by contextual collection rule
no fake expected-value evidence
```

### Downstream analysis through invalid binding

```phalcom
let x: Int = CellNum.new()
let y = x.cellOnly()
```

Expected downstream expression:

```text
knowledge  Established(return type)
status     Ready
invalidity One(binding mismatch cause)
callable   resolved CellNum.cellOnly
```

---

## 21. Conformance tests

Expression tests should assert the complete semantic product relevant to the behavior. Tests that only compare `knowledge.ty()` are insufficient for status, causality, provenance, and terminal propagation bugs.

Coverage should include:

- synthesis with no context;
- checking with compatible context;
- checking with incompatible context;
- contextual empty literals;
- assignment with and without persistent contract;
- variable read through invalid binding;
- exact calls with invalid arguments;
- generic calls;
- dynamic boundaries;
- terminal relation outcomes;
- branch knowledge joins;
- unsupported expression forms failing closed.

---

## Source basis

This specification is derived from the Part 1 Formal Semantic Epistemic Foundation specification and its Corrections and Amendments. The amendments take precedence on generic failure evidence, inference support, suppression-cause representation, and semantic fingerprinting. Repository implementation notes were re-grounded against `aureat/phalcom-lang` `main` at `c3b82e4b88469ef9fc79aa65a03e0bed95dc908d`; such notes are non-normative and may be updated as the code evolves.
