# Phalcom Semantic Analyzer Specification
## 03 — Analysis Status, Causality, Invalidity, and Recovery

**Status:** Normative semantic-analyzer specification.

**Purpose:** Specify how semantic operations report completion or failure independently from type knowledge, how diagnostic causes are owned and propagated, and how the analyzer preserves useful semantics in invalid programs.

---

## 1. Three independent dimensions

Every analyzed expression can conceptually carry three independent dimensions:

```text
TypeKnowledge
AnalysisStatus
CausalInvalidity
```

These answer different questions.

`TypeKnowledge` asks:

> What formal knowledge, if any, is available about the value's type?

`AnalysisStatus` asks:

> What happened when the analyzer attempted this semantic operation?

`CausalInvalidity` asks:

> Does this result depend on source whose semantic validity has already been compromised?

None may be derived mechanically from another.

The implementation must not assume:

```text
known type        => Ready
unknown type      => Invalid
causal invalidity => Suppressed
Invalid           => Unknown
Dynamic           => analysis failure
```

Those implications are false in important Phalcom programs.

---

## 2. Analysis status model

The analyzer supports terminal or published statuses equivalent to:

```text
Ready
Invalid(cause)
Suppressed(non-clean cause summary)
Blocked(reason)
DynamicBoundary(reason)
Cancelled
BudgetExceeded(report)
InternalFailure(incident/reason)
```

Exact representation may change provided the semantic distinctions remain observable.

### 2.1 Ready

`Ready` means the analyzer possessed the required premises and completed the semantic operation.

It does **not** mean the surrounding program is globally valid.

A `Ready` expression may carry causal invalidity from an upstream contradiction.

### 2.2 Invalid

`Invalid(C)` means the operation itself owns a semantic contradiction or invalid language use represented by diagnostic cause `C`.

An invalid expression may still retain independently known type information.

### 2.3 Suppressed

`Suppressed(S)` means the operation could not perform its own semantic judgment because a required semantic premise was unavailable due to an upstream invalid cause.

Suppression is intentionally narrow. It exists to avoid diagnostic cascades while accurately saying that analysis was prevented.

Suppression must not be used merely because a dependency is causally invalid.

### 2.4 Blocked

`Blocked(reason)` means analysis cannot currently complete because a semantic prerequisite or fixpoint condition is unavailable for a reason distinct from a source-level contradiction.

Examples may include recursive fixpoint blockage or opaque native semantics depending on the surrounding architecture.

### 2.5 Dynamic boundary

`DynamicBoundary(reason)` means static analysis intentionally yields to runtime behavior at this operation.

This is distinct from both `TypeKnowledge::Dynamic` and ordinary failure. The two may coexist where that represents the operation accurately.

### 2.6 Cancelled

`Cancelled` means analysis was stopped by cancellation. It is not a type judgment and must not be reported as a contradiction, recursive block, or generic uncertainty.

### 2.7 Budget exceeded

`BudgetExceeded(report)` means the configured analysis resource limit was exhausted.

The analyzer may retain independent semantic facts already established before exhaustion, but must not claim that unfinished dependent judgments succeeded.

### 2.8 Internal failure

`InternalFailure(...)` represents analyzer/infrastructure failure rather than invalid user code. It must not be translated into a plausible source-level type explanation.

---

## 3. Causal invalidity

Causal invalidity is a compact hot-state summary of dependency on owning semantic failures.

Conceptually:

```text
CausalInvalidity =
    Clean
  | One(DiagnosticCauseId)
  | Multiple
```

`Multiple` intentionally represents a cardinality class rather than an unbounded set.

The full explanation graph may retain richer root-cause relationships. Hot expression/binding state need only answer whether no root, one root, or multiple roots are relevant and, in the one-root case, which root permits suppression linkage.

### 3.1 Monotonicity

Within one analysis path, causal invalidity can accumulate:

```text
Clean + One(C1) -> One(C1)
One(C1) + One(C1) -> One(C1)
One(C1) + One(C2) -> Multiple
Multiple + anything -> Multiple
```

It does not spontaneously return to `Clean` unless the semantic path itself is rebuilt from clean inputs in a new analysis.

### 3.2 Status/cause coherence

The three result dimensions remain independent, but their representations must agree:

- `Invalid(C)` owns diagnostic cause `C`.
- The causal summary for that result must semantically include `C`: `One(C)` when it is the only root, otherwise `Multiple`.
- The explanation/diagnostic graph must retain the exact owning cause even when the hot causal summary is `Multiple`.
- `Clean` cannot describe the same owning judgment as `Invalid(C)`.
- `Suppressed(S)` requires a missing premise whose absence is explained by non-clean upstream cause summary `S`.
- Source-range overlap or an unrelated diagnostic must not determine expression status or cause ownership.

### 3.3 Legal result combinations

| Knowledge | Status | Legal | Meaning |
|---|---|---:|---|
| `Established(T)` | `Ready` | yes | exact analysis completed |
| `Established(T)` | `Invalid(C)` | yes | result independently known; local judgment refuted |
| `Established(T)` | `Suppressed(S)` | conditional | the known result is independent, but a separate required judgment lacked an upstream-invalid premise |
| `Unknown(R)` | `Ready` | yes | operation completed honestly without a concrete proposition |
| `Unknown(R)` | `Invalid(C)` | yes | owning contradiction and no independent result |
| `Dynamic(D)` | `DynamicBoundary(D)` | yes | runtime authority is intentional |
| known knowledge | `Cancelled` / `BudgetExceeded` | conditional | knowledge was independently established before unfinished dependent work |

The conditional rows require an explicit independence argument. They must not be produced by copying a contract, expected type, or stale cached result into the knowledge field.

---

## 4. Diagnostic ownership

A semantic contradiction has an owning judgment.

Examples:

```text
binding initializer vs binding contract
assignment RHS vs binding contract
call argument vs parameter contract
return expression vs callable return contract
kind application vs required kind
generic constraint vs candidate substitution
```

The owning judgment creates the diagnostic cause.

Downstream expressions do not create duplicate errors merely because they depend on a value whose origin is invalid.

This separates:

```text
diagnostic ownership
```

from:

```text
causal dependency
```

and is the basis for useful error recovery.

Cause identity must be allocated explicitly by the semantic diagnostic mechanism. It must not be inferred by scanning for a diagnostic whose source range overlaps an expression.

---

## 5. The invalid-but-analyzable rule

The central recovery law is:

> Invalid source does not erase independently available semantic premises.

Canonical example:

```phalcom
class CellNum {
  @constructor
  new() {}

  cellOnly() -> Int { 1 }
}

class Probe {
  @class
  run() {
    let x: Int = CellNum.new()
    let y = x.cellOnly()
  }
}
```

### 5.1 Initializer

`CellNum.new()` resolves exactly and establishes `CellNum`.

```text
knowledge  = Established(CellNum)
status     = Ready
invalidity = Clean
```

### 5.2 Binding reconciliation

The binding contract is `Int`; the current value fact is `CellNum`.

The assignability judgment is refuted and owns cause `C1`.

```text
x.contract     = Int
x.current      = Established(CellNum)
x.consistency  = Refuted(...)
x.invalidity   = One(C1)
```

The current fact remains available.

### 5.3 Binding read

Reading `x` copies the current value knowledge and the causal dependency.

```text
knowledge  = Established(CellNum)
status     = Ready
invalidity = One(C1)
```

The read is not itself invalid merely because the binding declaration is inconsistent.

### 5.4 Downstream dispatch

`x.cellOnly()` has the exact receiver fact required to perform dispatch.

```text
receiver       = Established(CellNum)
resolved call  = CellNum.cellOnly
result         = Established(Int)
status         = Ready
invalidity     = One(C1)
```

This is the canonical demonstration that:

```text
causal invalidity != suppression
```

---

## 6. Genuine suppression

Suppression is correct when an upstream failure removed a premise that the child semantic operation requires.

Conceptually:

```text
upstream operation owns C1
        ↓
required receiver/callee/type proposition unavailable
        ↓
dependent operation cannot run its own judgment
        ↓
status = Suppressed(One(C1))
```

Examples may include:

- an unresolved or invalid construct prevents determining what callable is being invoked;
- a required structural component has no usable semantic result because analysis of that component was suppressed;
- a dependent expression requires a value fact that genuinely disappeared rather than merely becoming causally invalid.

The implementation should distinguish this from ordinary propagation of `Unknown`: suppression means *the dependency on an invalid root is why the operation could not be analyzed*.

---

## 7. Invalid operations can retain known results

A call can be invalid while its result type is independently fixed.

For example, if an exact callable identity has a concrete return contract independent of its generic variables or bad argument:

```text
call target       known
fixed return      Int
argument relation Refuted
```

then the call may publish:

```text
knowledge  = Established(Int)
status     = Invalid(C1)
invalidity = One(C1)
```

The invalid call is not rewritten to `Unknown` merely to reflect the diagnostic.

This rule generalizes to other constructs where result semantics are independent of the failed judgment.

---

## 8. Terminal status propagation

Terminal outcomes must propagate according to their actual meaning.

A relation or inference consumer may perform additional surrounding bookkeeping, but it must not silently coerce:

```text
Cancelled       -> Blocked
BudgetExceeded  -> success
InternalFailure -> ordinary uncertainty
DynamicBoundary -> Assignable
```

A consumer that has independently known result knowledge may preserve that knowledge while propagating the terminal status.

This yields a product such as:

```text
knowledge = Established(Int)
status    = BudgetExceeded(...)
```

when the result itself is independently known but some other required analysis did not finish.

---

## 9. Status production and publication

Status should be determined at the semantic operation that owns the judgment or terminal event.

Publication must not reconstruct status by rules such as:

```text
if causal_invalidity != Clean => Suppressed
if knowledge has no TypeId    => Unknown/Invalid
if a diagnostic overlaps range => Invalid
```

Those are lossy heuristics.

The internal expression-result carrier must therefore preserve enough status information for nested transfer and final publication. Whether that is represented directly on a `TypedExpression`, in a parallel result object, or in another semantically equivalent structure is an implementation choice.

The requirement is end-to-end preservation.

---

## 10. Causality through compound expressions

A compound expression aggregates causal invalidity from the inputs that semantically contribute to its result or operation.

Examples include:

```text
receiver
arguments
branch values
collection elements
record fields
operator operands
index target/index
```

Aggregation must be value/dependency-based, not source-range-based.

A compound expression that successfully analyzes using causally invalid children can remain `Ready` while carrying the joined causal invalidity.

If a child is suppressed and its missing result is required, the parent may become suppressed using that upstream cause.

---

## 11. Branches and causal joins

When branches merge:

```text
then invalidity + else invalidity -> joined invalidity
```

The causal join is independent of type-knowledge join and analysis-status join.

For example:

```text
then: Established(Int), Ready, One(C1)
else: Established(Float), Ready, Clean
```

may yield:

```text
knowledge  = Established(Int | Float)
status     = Ready
invalidity = One(C1)
```

The fact that one reachable value came through invalid source does not make the merge unanalyzable.

---

## 12. Diagnostics versus explanations

Diagnostics communicate user-facing invalid judgments.

Explanations communicate why semantic facts or failures hold.

A downstream `Ready` expression with causal invalidity should be explainable as:

```text
result type established by exact callable return
receiver fact derived from binding x
binding x is causally invalid due to C1
```

without manufacturing another mismatch diagnostic.

This separation is necessary for precise hover information and non-cascading error reporting.

---

## 13. Cause identity and incremental semantics

Raw `DiagnosticCauseId` values are local allocator identities.

They are useful within a snapshot for linking:

```text
Invalid(C1)
Suppressed(One(C1))
diagnostic root C1
```

They are not semantic cache identity.

A semantically identical analysis whose allocator numbers shift from `C1` to `C2` should retain the same semantic product fingerprint if causal shape and substantive diagnostic meaning are unchanged.

Fingerprinting rules are defined in `09-semantic-products-incrementality-and-fingerprints.md`.

---

## 14. External semantic contract

Consumers may rely on these behaviors:

- `Ready` does not imply `CausalInvalidity::Clean`.
- Non-clean causal invalidity does not imply `Suppressed`.
- `Invalid` identifies an owning failure rather than merely downstream contamination.
- `Suppressed` means a required premise was unavailable because of upstream invalidity.
- `Blocked`, `Cancelled`, `BudgetExceeded`, `DynamicBoundary`, and `InternalFailure` remain distinguishable.
- Invalid expressions may retain established or assumed type knowledge when that knowledge is independently justified.
- Diagnostic root ownership is not reconstructed from source-range coincidence.
- Downstream analysis continues wherever required semantic premises survive.

---

## 15. Conformance scenarios

At minimum, regression coverage should include:

1. Invalid annotation + established initializer + downstream method dispatch remains analyzable.
2. Invalid call argument + fixed concrete return retains the return type while status is invalid.
3. Genuine missing receiver premise produces suppression.
4. `Ready + One(C)` is preserved through at least one compound expression.
5. Multiple independent causes aggregate to `Multiple`.
6. Cancellation remains cancellation at an expression consumer boundary.
7. Budget exhaustion remains budget exhaustion.
8. Dynamic boundary does not become unknown.
9. Cause renumbering alone does not change semantic product identity.
