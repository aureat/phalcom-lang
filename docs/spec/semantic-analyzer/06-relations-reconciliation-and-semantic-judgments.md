# Phalcom Semantic Analyzer Specification
## 06 — Relations, Reconciliation, and Semantic Judgments

**Status:** Normative semantic-analyzer specification.

**Purpose:** Specify how formal semantic relations report outcomes and how consumers translate those outcomes into binding consistency, diagnostics, expression status, call validity, and inference behavior without losing information.

---

## 1. Relations are semantic computations, not predicates

A subtype or assignability check is not always a boolean proposition computable to `true` or `false` within the analyzer.

The analyzer may encounter:

```text
proved assignable
proved refuted
dynamic boundary
blocked analysis
uncertain relation
cancelled computation
budget exhaustion
internal failure
```

A relation API therefore returns a structured result.

A helper that reduces this to:

```text
Refuted => false
everything else => true
```

is semantically invalid because it treats inability to prove as proof.

---

## 2. Core judgment shape

Conceptually:

```text
judge(actual, expected, context)
    -> RelationOutcome
```

Representative outcomes:

```text
Assignable
Refuted(RelationFailure)
DynamicBoundary(DynamicReason)
Blocked(BlockReason)
Uncertain(UncertaintyReason)
Cancelled
BudgetExceeded(BudgetReport)
InternalFailure(Incident)
```

Exact enum partitioning may differ, but these distinctions must remain representable through consumer boundaries.

---

## 3. Assignable

`Assignable` means the formal checker established that the actual proposition satisfies the required relation.

A successful relation can validate a contract or call argument.

It does not necessarily upgrade the epistemic strength of the actual value.

Example:

```text
actual = Assumed(Int)
expected = Number
relation = Assignable
```

The analyzer has proved a relation over an assumed premise; the actual remains assumed.

---

## 4. Refuted

`Refuted` means the checker established that the required relation does not hold.

A refutation should retain the real operands and enough failure structure to explain the failed judgment.

It produces a contradiction between independently preserved facts.

For binding reconciliation:

```text
current Established(Int)
contract String
relation Refuted(Int <: String)
```

does not mean:

```text
current = String
```

or:

```text
current = Unknown
```

It means the binding has current `Int` knowledge and a refuted contract relation.

---

## 5. Dynamic boundary

A dynamic boundary means the formal relation cannot or should not be resolved statically because runtime semantics take over.

Consumers should propagate dynamic semantics explicitly.

A dynamic boundary is neither an assignability proof nor a refutation.

Depending on the operation, it may lead to:

```text
AnalysisStatus::DynamicBoundary
TypeKnowledge::Dynamic
runtime-check obligation
```

or an equivalent representation.

---

## 6. Blocked

Blocked means the relation depends on semantic information that cannot currently be established due to a formal analysis condition such as recursive fixpoint or unavailable modeled semantics.

Consumers must not convert blocked into success.

They may preserve independent facts while recording the blocked status.

---

## 7. Uncertain

Uncertain represents a relation for which the checker cannot currently prove either compatibility or incompatibility under the relevant formal algorithm.

The policy for uncertain must be explicit at the consuming operation.

It must not be casually mapped to an unrelated block reason such as `RecursiveFixpoint` unless that is the actual cause.

---

## 8. Cancellation, budget, and internal failure

These outcomes describe execution of the analyzer.

They must propagate distinctly because they have different retry, diagnostics, telemetry, and user-facing consequences.

```text
Cancelled
BudgetExceeded
InternalFailure
```

must not be disguised as type mismatch, suppression, or ordinary uncertainty.

---

## 9. Consumer interpretation

The relation engine answers the relation question. The caller decides what that answer means for the surrounding semantic construct.

That separation should be explicit.

### 9.1 Binding initialization

```text
actual initializer knowledge
        ↓
binding contract
        ↓
relation outcome
        ↓
BindingConsistency
AnalysisStatus/diagnostic ownership
causal invalidity
```

### 9.2 Reassignment

Same relation machinery, but state-transfer consequences differ because a write updates current flow knowledge.

### 9.3 Call argument

```text
argument knowledge
        ↓
parameter contract
        ↓
relation
        ↓
call validity/status
diagnostic
```

The argument's actual knowledge remains intact.

### 9.4 Return checking

```text
return expression knowledge
        ↓
callable return contract
        ↓
relation
        ↓
body/callable validity
```

### 9.5 Generic inference

Some relations become solver constraints rather than immediate proper-type judgments. Terminal solver results must preserve the same philosophy: real failure, blockage, cancellation, and budget outcomes remain explicit.

---

## 10. Reconciliation

Binding reconciliation is a pure semantic operation over:

```text
persistent contract
current knowledge
relation result
```

Its purpose is to classify the relationship without mutating the independent inputs.

Conceptually:

```text
reconcile(contract, current)
    -> BindingReconciliation
```

Possible semantic results include:

```text
Unconstrained
Validated
Assumed(basis)
Refuted(real failure)
DynamicBoundary
Blocked(real reason)
Cancelled
BudgetExceeded
InternalFailure
Uncertain
```

The concrete `BindingConsistency` type may choose to store only a subset if the remaining terminal outcome is carried in a sibling `AnalysisStatus`. What is prohibited is losing the distinction entirely.

---

## 11. Assumption through contracts

A contract may produce assumed current knowledge only in an explicitly eligible no-evidence case.

Conceptually:

```text
contract = Int
current  = Unknown(NoTypeEvidence)
eligible = true
        ↓
current  = Assumed(Int)
basis    = DeveloperBindingContract
```

This is not a relation proof. It is a separate semantic transition justified by the language's contract policy.

A blocked or syntax-invalid current result is not eligible.

---

## 12. One owning diagnostic per judgment site

A refuted relation normally creates one owning diagnostic at the semantic judgment site.

Downstream nodes carry causal invalidity rather than recreating the same diagnostic.

Examples of owning sites:

```text
binding initialization
assignment
argument
return
generic constraint
kind application
```

The relation failure should retain enough evidence to render the diagnostic without inventing placeholder types or source identities.

---

## 13. Relation evidence

Relation failures should preserve the actual judgment.

For example:

```text
Refuted {
    actual: CellNum,
    expected: Int,
    relation: Assignability,
    ...
}
```

or an equivalent structured representation.

The analyzer must not use sentinel values such as `Never` or `Unit` to fill missing operands in an error object.

If a structural failure has no single ordinary pair of type operands, it should have a structural failure variant.

---

## 14. Relation results and expected context

Checking an expression under expected type `E` should:

1. analyze/synthesize the expression;
2. run the appropriate relation against `E`;
3. retain actual knowledge;
4. apply diagnostic/status effects from the relation.

It must not implement checking by assigning `E` to the expression first and then trivially proving it compatible.

---

## 15. Relation results and fixed independent knowledge

A failure in one relation does not erase unrelated established facts.

Example:

```text
resolved callable = foo
fixed return      = Int
argument actual   = String
parameter         = Number
relation          = Refuted
```

Call product:

```text
knowledge  = Established(Int)
status     = Invalid(C)
invalidity = One(C)
```

The failed argument relation affects validity, not the independently fixed return proposition.

---

## 16. Propagation protocol

Every consumer of a relation outcome should have an explicit mapping.

A normative conceptual mapping is:

| Relation outcome | Consumer obligation |
|---|---|
| `Assignable` | continue as successful relation |
| `Refuted` | preserve actual facts, own/attach contradiction |
| `DynamicBoundary` | propagate dynamic semantics |
| `Blocked` | propagate blocked reason |
| `Uncertain` | propagate explicit uncertainty policy |
| `Cancelled` | propagate cancellation |
| `BudgetExceeded` | propagate budget outcome |
| `InternalFailure` | propagate internal failure |

The exact surrounding `AnalysisStatus` may differ by operation, but no outcome may be silently treated as `Assignable`.

### 16.1 Consumer matrix

| Relation outcome | Binding declaration / assignment | Call argument / setter / subscript / operator | Return | Generic constraint |
|---|---|---|---|---|
| `Assignable` | validate contract; preserve actual fact | continue; preserve operands | accept path | add satisfied evidence |
| `Refuted` | own mismatch; retain actual/current recovery fact | owning operation creates one cause; independent result may survive | own return mismatch | retain real conflict evidence |
| `DynamicBoundary` | mark runtime-dependent consistency | propagate dynamic semantics | preserve dynamic boundary | do not claim static satisfaction |
| `Blocked` | leave consistency unresolved | propagate blocked reason | callable path incomplete | solver blocked with dependency |
| `Uncertain` | apply explicit uncertainty policy | preserve uncertainty; no fabricated success | summary remains uncertain | retain underconstrained/ambiguous state |
| `Cancelled` | do not commit unfinished transition | propagate cancellation | callable analysis cancelled | cancel solver transaction |
| `BudgetExceeded` | do not claim validation | propagate budget report | partial result only when independent | preserve budget report |
| `InternalFailure` | contain incident; no source mismatch | propagate/contain internal failure | callable incomplete | abort affected solver product |

Every cell also preserves upstream causal invalidity and the relation's explanation evidence. Result knowledge survives only when its derivation is independent of the failed or unfinished relation.

### 16.2 Complete-outcome consumption

A consumer must consume the complete `RelationOutcome`. Calling a relation only for incidental side effects and discarding its status, evidence, operands, cause, or terminal payload is non-conforming unless the consumer proves those fields have no observable consequence for that operation.

---

## 17. Relation purity and diagnostics

Where possible, relation computation should be pure with respect to diagnostics.

A relation returns evidence; the semantic operation that requested it owns diagnostic creation.

This makes:

- testing simpler;
- reuse safer;
- generic solver integration cleaner;
- relation algorithms independent of source presentation;
- one-diagnostic ownership easier to enforce.

---

## 18. Recursive relations and budgets

Subtype/assignability algorithms involving recursive or structural types may require recursion guards or budgets.

When limits are hit, the outcome is an explicit non-success result.

The implementation must not return `Assignable` merely to break recursion, unless the type theory explicitly proves the relation through a coinductive rule.

Similarly, it must not report a fabricated refutation where the actual issue is exhausted analysis resources.

---

## 19. External behavior contract

Consumers can rely on:

- all semantically distinct terminal outcomes surviving relation boundaries;
- refutation retaining actual operands;
- successful relation not upgrading assumed evidence;
- dynamic relation being distinguishable from static proof;
- cancellation/budget/internal failure remaining operational outcomes;
- reconciliation preserving contract and current knowledge separately;
- independent result knowledge surviving unrelated relation failure;
- diagnostics being owned at the judgment that fails.

---

## 20. Conformance tests

The test suite should cover every relation outcome at at least one direct consumer boundary.

A branch-coverage matrix should include:

```text
binding initialization
assignment
argument checking
return checking
generic constraint/inference
```

Not every terminal state must have a source-level syntax fixture if it is not naturally source-triggerable. Internal tests may inject relation outcomes through the lowest legitimate semantic API.

The critical rule is that every variant be exercised through at least one consumer that would otherwise be capable of collapsing it.
