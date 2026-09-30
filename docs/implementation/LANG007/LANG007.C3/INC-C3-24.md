# IMPLEMENTATION INCIDENT — INC-C3-24

## 1. Identity

- Program: `LANG007`
- Checkpoint: `LANG007.C3`
- Plan: `LANG007.C3.P4.R1`
- Task/Gate: `T5 / G4 / CV-08`
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `d8902f4c`
- Current revision: `f10f3d4909492cac5c93411709572b7674e364a8` (the committed CV-09 slice is separate and not part of this incident)

## 2. Escalation trigger

RED trigger 1/4/8/12: live body-analysis products do not expose a sound
result-dependency edge. Adding or deriving that edge may change the canonical
callable-body product contract, which is architectural rather than a local
diagnostic implementation detail.

## 3. Required invariant

- An unseeded direct or mutual recursive result cycle remains unknown; infer no
  `Unit`, nominal type, or other fabricated result.
- Do not add recursive result inference or a general fixpoint.
- Diagnose only a genuine result-dependency SCC, not every recursive call or
  body-level callable dependency.
- Keep `NormalReturnFact::publication_knowledge` fail-closed for blocked bodies.
- Explicit declared result contracts remain authoritative seeds.

## 4. Expected architecture

The adopted ruling for INC-C3-23 assigns deterministic unseeded-cycle
diagnostics to callable-result inference/publication validation. SCC discovery
is classification-only. The diagnostic should be one per unresolved SCC,
anchored deterministically at a declaration, with no per-call-site errors.

## 5. Observed behavior

The existing unseeded direct and mutual recursion controls preserve unknown
results and do not fabricate a type. Before the ruling, they had no diagnostic;
the controls did not establish a sound implementation-time edge source for
result SCC detection.

`CallableAnalysis.dependencies` is an aggregate body-consumption set populated
when callable members/calls are resolved. It can contain calls that do not
contribute to the callable's returned result. For example, using that graph
alone could misclassify a body that makes a recursive-looking call and then
returns a known independent value.

`NormalReturnFact` currently retains knowledge, flow, status, and causal
invalidity, but no source expression identity. `CallableAnalysis.expressions`
retains per-expression selected callable identities, but the return facts do
not connect a returned value to its root expression. Therefore the live
products do not directly provide the result-relevant edges required by the
ruling.

## 6. Minimal reproduction

Existing semantic control shape:

```phalcom
class Probe {
  @class
  f() { g() }

  @class
  g() { f() }
}
```

Focused existing selector:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::callable_publication_capabilities::recursive_inference_fails_honestly_without_inventing_unit_or_nominal_type \
  -- --exact
```

The test verifies both selected recursive call results remain unknown or
dynamic. It does not yet assert the newly required diagnostic; no CV-08
production change has been attempted.

## 7. Relevant code path

```text
checker/call.rs resolves calls and records body-level callable dependencies
  -> checker/context.rs aggregate `dependencies` set
  -> checker/body.rs and checker/statement.rs build NormalReturnFact
  -> checker/analysis.rs publishes CallableAnalysis
  -> session.rs refresh_inferred_callable_results publishes fail-closed summaries
```

Key locations:

- `phalcom-semantic/src/checker/context.rs`: callable dependency insertion and
  resolution products (including the `resolved_callables` expression map).
- `phalcom-semantic/src/checker/body.rs` and
  `phalcom-semantic/src/checker/statement.rs`: normal-return fact construction.
- `phalcom-semantic/src/checker/analysis.rs`: `NormalReturnFact` and
  `CallableAnalysis` data shapes.
- `phalcom-semantic/src/session.rs`: `refresh_inferred_callable_results`.

## 8. Scoped implementation diff

No CV-08 source or test diff exists. The concurrent CV-09 slice changes only
generic-call status handling and focused tests; it does not alter return facts,
callable dependencies, result publication, or recursion handling.

## 9. Evidence

- Existing body dependency tracking aggregates callable resolution throughout
  body analysis rather than storing a normal-return-specific edge.
- `NormalReturnFact` has no `ExpressionId` or selected result dependency.
- `ExpressionAnalysis` can identify a selected callable for an expression, but
  this is not currently linked from each normal return.
- The adopted INC-C3-23 ruling prohibits a general result fixpoint and requires
  diagnostics to correspond to unresolved result cycles.

## 10. Implementer's causal hypothesis

A diagnostic based on the current aggregate dependency set can report a cycle
that does not affect the inferred result. A sound implementation needs an
explicit, result-relevant edge source or a narrowly defined classification
rule tied to each value-producing return.

## 11. Alternative hypotheses

- For the required direct/mutual controls, a root returned-call identity may be
  sufficient if the architect approves that bounded rule.
- More general recursive result dependencies through operators, branches, or
  nested value-producing expressions may require richer provenance than the
  current body product carries.
- Aggregate body dependencies may be useful for scheduling/invalidation but
  are not, by themselves, proof of result dependence.

## 12. Attempts already made

- No CV-08 production attempt was made after the architect ruling.
- The aggregate dependency graph was rejected before implementation because it
  admits false result-cycle edges.

## 13. Why local authority is insufficient

Choosing the representation and scope of a result-dependency edge changes the
semantic body-product contract used by publication validation. The plan and
adopted ruling require sound SCC classification but do not specify whether to
extend `NormalReturnFact`, derive edges from source/analysis products, or limit
the accepted rule to directly returned calls. This crosses the repository's
mandatory consultation boundary for semantic authority and cross-layer
contracts.

## 14. Decision requested

For CV-08, may T5 extend the callable-body product with per-normal-return
result-expression provenance (using the existing `ExpressionId`/resolved-call
products) and build a classification-only SCC graph from result-relevant
returns? If so, is a direct returned-call edge the authorized scope for the
required direct/mutual controls, or must the implementation preserve
dependencies through nested value-producing expressions before emitting any
cycle diagnostic?

Until this is resolved, CV-08 production changes remain paused. CV-09 may
continue independently. T5/G4 and P4 remain incomplete.
