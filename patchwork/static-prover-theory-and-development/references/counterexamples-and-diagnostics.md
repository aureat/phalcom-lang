# Counterexamples and Diagnostics

## From model to user explanation

Solver model may contain names like:

```text
x!0 = -1
H3 = ...
obj!7 = ...
```

Users need:

```text
precondition may fail when amount = -1
because this call permits a negative value here
```

Maintain mapping from logical variables to semantic IDs/source ranges.

## Minimal counterexamples

Prefer small relevant models. Constrain/simplify auxiliary variables and hide internal heap/version symbols.

## Path explanation

Record branch decisions leading to violation:

```text
1. x <= 0 at line ...
2. branch enters ...
3. assertion x > 0 fails at ...
```

## Unsat cores

When solver supports them, unsat cores can identify assumptions necessary for proof. Useful for explaining why a contract is redundant or which invariants establish a result.

## Proof traces

Do not expose massive solver proofs in normal diagnostics. Keep internal evidence for debugging/auditing and render a compact reason chain.

## Unknown diagnostics

Different unknown reasons need different advice:

```text
missing loop invariant -> annotate/provide invariant
solver timeout -> simplify contract/report resource limit
dynamic boundary -> add type/contract/runtime check
untrusted native call -> provide native summary
```

## Multiple spans

Contract diagnostics often need:

- declaration contract span;
- violating call/return span;
- provenance source of conflicting fact.

## Stability

Diagnostic codes should describe semantic obligation kind, not solver backend wording. Changing solver must not rewrite public error taxonomy.

---

## Deep treatment: explanations as first-class proof products

### Diagnostic data model

Do not wait for solver output to decide what to explain. Every obligation should carry:

```text
ObligationKind
PrimarySourceRange
SemanticEntityId
ExpectedPropertyOrigin
Path/assumption origins
Logical-variable mapping
Relevant contract/effect/native assumptions
```

A solver model then fills values into an existing explanation skeleton.

### Witness reconstruction pipeline

```text
solver model
  -> decode typed logical values
  -> validate semantic-domain invariants
  -> map symbols to SSA/semantic IDs
  -> collapse SSA versions to source variables at program points
  -> reconstruct branch path
  -> simplify heap/object details
  -> render source diagnostic
```

If decoding fails because the model uses an impossible runtime state, classify it as modeling/backend failure rather than blaming the user program.

### Counterexample minimization

SMT models are not necessarily minimal. Useful minimization strategies:

- ask solver for smaller integers/shorter strings with optimization when robust;
- iteratively fix/remove irrelevant variables while retaining satisfiability;
- show only variables that influence the violated obligation/path;
- collapse internal heap versions into before/after field values.

Do not spend unbounded time minimizing in the IDE; use budgets and present a correct larger witness if needed.

### Proven diagnostics

Proof success can also need explanation in tooling:

```text
assertion proven
  because x >= 1 from branch condition
  and length(xs) = 3 from verified callee contract
```

This is useful for debugging prover behavior and future “why is this check removable?” tooling. Keep it optional; normal users should not see giant proof traces.

### Unknown guidance

Map reasons to actionable next steps:

```text
MissingLoopInvariant -> identify loop and suggest invariant category
OpenWorldDispatch    -> require protocol contract/sealed assumption/runtime check
UntrustedNative      -> add system-reviewed native summary/conformance test
UnsupportedTheory    -> simplify/restate property or expand prover capability
SolverTimeout        -> show obligation/theory hotspot, not "property false"
MalformedSource      -> defer proof until semantic dependency is complete
```

### Diagnostic stability

Public diagnostic code should reflect semantic failure:

```text
proof.precondition_not_established
proof.postcondition_counterexample
proof.unknown.loop_invariant
proof.unknown.dynamic_dispatch
```

not backend strings such as `Z3_L_TRUE` or solver tactic names. Backend replacement should not churn user-facing taxonomy.
