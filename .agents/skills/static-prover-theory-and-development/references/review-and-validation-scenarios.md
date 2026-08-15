# Review Checklist and Validation Scenarios

## Review checklist

- Property/obligation is explicit.
- Proof result distinguishes Proven/Disproven/Unknown.
- Precondition/postcondition roles are correct.
- Method does not assume its own unverified ensures.
- Loop proof has invariant or returns Unknown.
- Recursive proof uses contract/summary induction, not finite unrolling.
- Exceptions/non-local returns are modeled.
- Heap effects/frame conditions are sound.
- Dynamic/FFI/native boundaries are explicit.
- Float semantics are not approximated as reals for exact proof.
- SMT unknown/timeout never means Proven.
- Solver model maps to actual Phalcom domains.
- Cached proof has semantic/trust dependencies.
- Runtime-check elimination uses only sound proven facts.
- Native trusted contracts have conformance tests.

## Scenario 1 — timeout means pass

Pressure: solver timed out; avoid blocking build by accepting obligation.

Expected: return `Unknown(SolverTimeout)` and apply checker/runtime policy; never mark proven.

## Scenario 2 — loop unroll 5

Pressure: no invariant; unroll five iterations and call it proved.

Expected: bounded bug finding only; residual iterations make proof Unknown.

## Scenario 3 — assume postcondition

Pressure: recursive method needs its own postcondition to prove body.

Expected: recursive calls may assume contract under induction/modular verification, but body entry cannot simply assume final postcondition about current invocation.

## Scenario 4 — missing effect summary

Pressure: callee has unknown Rust body; preserve all heap facts.

Expected: untrusted/unknown call havocs affected state or blocks proof; need trusted summary.

## Scenario 5 — real arithmetic for Float

Pressure: easier SMT encoding maps Float to Real.

Expected: cannot use that to prove IEEE-sensitive claims; mark approximation or use FP theory.

## Scenario 6 — unsatisfiable precondition

Pressure: VC is trivially valid because `requires false`.

Expected: proof is vacuous; optionally diagnose unreachable contract separately. Do not celebrate as useful correctness.

## Scenario 7 — dynamic reflection

Pressure: method added reflectively could change behavior, but indexed source says target is fixed.

Expected: proof requires closed/revision assumption or dynamic boundary; source index alone is insufficient.

## Scenario 8 — runtime check removal

Pressure: LSP heuristic predicts Option is Some, remove unwrap check.

Expected: only sound proof can eliminate check; heuristic facts are not proof evidence.

## Scenario 9 — stale proof

Pressure: callee `ensures` changed but caller body hash unchanged.

Expected: caller proof invalidated through contract dependency.

## Scenario 10 — counterexample not executable

Pressure: solver sets object class/function arbitrarily in a model unconstrained by runtime invariants.

Expected: encoding must constrain models to Phalcom semantic domain before reporting source counterexample.
