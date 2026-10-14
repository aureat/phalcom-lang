# IMPLEMENTATION INCIDENT

## 1. Identity

- Program / checkpoint / plan: LANG007 / LANG007.C4 / LANG007.C4.P7
- Task / gate: T8–T10 / G8–G10
- Repository / branch: `aureat/phalcom-lang` / `main`
- Starting revision: `dbcdcd13`; observed continuation revision: `cb8a068c` plus scoped working-tree changes.

## 2. Escalation trigger

Repository RED triggers 8, 11, and 16: the residual representation and the
checker/evaluator contract for unevaluated branches need an architectural
decision before claiming the audit's explicit `If`/`Match` acceptance. The
plan permits a narrower `Apply` fallback for imperative source bodies.

## 3. Required invariant

P7-I1, I2, I6, I10, and I14: one evaluator and canonical pattern resolver;
no source AST or suspended evaluator frame in a residual; exact evaluation
equals open evaluation followed by specialization; untaken branches do not run.

## 4. Expected architecture

P7 §§14–15 and T8–T9 prefer normalized branch terms where representable,
but explicitly allow the enclosing `Apply` for imperative branches. The
read-only P7 audit §5 requires operational `If` and `Match` residuals, including
canonical arm semantics and proof contexts.

## 5. Observed behavior

`StaticResidualData` defines `If` and `Match` payloads, but
`specialize_residual_term` does not consume them. `StaticResidualMatchArm` holds
an already-evaluated `body: StaticEvalTerm`; the evaluator cannot materialize
untaken effectful branch bodies into such terms without executing them.
The current checked definition and match resolution remain available in the
same semantic session. A canonical enclosing `Apply` can replay them safely
after exact specialization, but does not provide a standalone `Match` term.

## 6. Minimal reproduction

```
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic open_value_condition_defers_to_the_enclosing_application
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic open_match_scrutinee_defers_without_losing_arm_order
```

Before the scoped continuation, both returned `Invalid(UnsupportedExpression)`
for valid open value parameters at `AfterIf` / `AfterMatchScrutinee`. After
the bounded fallback correction, both return an `Apply` residual, specialize
to the selected result, and pass cold-exact parity (1/1 each). The former
also proves a recursive untaken branch is not evaluated. These tests do not
establish standalone `If` or `Match` residual normalization.

## 7. Relevant code path

```
CheckedStaticBody + MatchResolutionIndex
  -> StaticEvaluator::run (AfterIf / AfterMatchScrutinee)
  -> residual_for_root (Apply capturing the definition and arguments)
  -> specialize_apply -> evaluate_specialized_application
  -> ordinary checked match / proof and selected branch
```

`residual.rs` defines `If`/`Match`; `specialize.rs` rejects both with
`UnsupportedExpression`. `pattern.rs` is the canonical runtime matcher.

## 8. Scoped implementation diff

`machine.rs` defers unresolved condition/scrutinee to the enclosing checked
application; `static_recursive_evaluation.rs` proves direct open/exact parity
and arm selection. No new branch-IR ownership or pattern authority was added.

## 9. Evidence

- Existing `If`/`Match` store constructors intern terms without deferred
  branch execution or proof-context transport.
- The checked-body execution path evaluates only the selected branch.
- Both discriminating tests failed before the fallback and passed after it.
- The P7 plan explicitly allows fallback for unrepresentable imperative bodies.

## 10. Implementer's causal hypothesis

`Apply` is sufficient for correct session-local specialization of imperative
control flow, but direct control residuals require checker-owned semantic
branch terms or another approved way to defer their evaluation. Reusing raw
AST or evaluator continuations would violate P7-I6.

## 11. Alternative hypotheses

- Restrict direct `If`/`Match` to already-reduced side-effect-free terms and
  leave all other control inside `Apply`.
- Define an additional canonical deferred-branch semantic product, owned by
  the checker, for the residual store to reference and specialize.

These alternatives change the accepted residual family contract and P8's
downstream freeze point differently.

## 12. Attempts already made

Only the plan-approved `Apply` fallback was implemented and verified. No
speculative direct control residual design was attempted.

## 13. Why local authority is insufficient

Choosing whether `Apply` satisfies G8/G9 for generic control, or whether a
new checked branch representation is mandatory, changes the residual identity,
P8 interface, and plan acceptance contract.

## 14. Decision requested

For P7 acceptance, is a normalized enclosing `Apply` sufficient for checked
static `if`/`match` bodies that cannot safely become pre-evaluated branch
terms? If explicit `If`/`Match` residuals are required, identify the owner and
minimal semantic carrier for unevaluated branch bodies and trusted branch
proofs without storing raw AST or a continuation.

## User ruling — 2026-10-13

Accept the checked enclosing `Apply` for control bodies whose branches cannot
be represented without executing untaken code. Standalone `If`/`Match` stays
gated rather than introducing a deferred-branch IR solely to satisfy the
existing enum. Amend G8/G9 to verify cold/open parity, ordered canonical
matching, branch proof transport, and untaken-branch immunity through this
route. Do not certify unsupported direct `If`/`Match` constructors as operational.
