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

A bounded read-only census found that source AST plus expression ranges can
identify a direct explicit-return or tail-expression root in simple cases, but
cannot establish general result contribution. Expression products do not
retain parent/child or value-provenance links. Syntax-recursively collecting
calls can pull in side effects, predicates, or other non-result calls; an
aggregate dependency graph also cannot trace a returned local to the call that
produced it. Nested operators, branches, and flow/binding values therefore
cannot be handled soundly by a range join alone.

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
  `phalcom-semantic/src/checker/statement.rs`: tail/explicit return fact
  construction (`body.rs:336–378`, `statement.rs:162–241`).
- `phalcom-semantic/src/checker/analysis.rs`: `NormalReturnFact` and
  `CallableAnalysis` data shapes (`analysis.rs:42–65, 333–340, 429–454`).
- `phalcom-semantic/src/session.rs`: `refresh_inferred_callable_results`
  consumes exit summaries (`session.rs:10112–10115`) and aggregate dependencies
  for rechecks (`session.rs:10217–10223`).

## 8. Scoped implementation diff

No CV-08 source or test diff exists. The concurrent CV-09 slice changes only
generic-call status handling and focused tests; it does not alter return facts,
callable dependencies, result publication, or recursion handling.

## 9. Evidence

- Existing body dependency tracking aggregates callable resolution throughout
  body analysis rather than storing a normal-return-specific edge.
- Explicit returns and tail expressions have a typed `ExpressionId` at their
  analysis sites, but drop it when creating `NormalReturnFact`.
- `ExpressionAnalysis` retains ID, range, selected callable, and knowledge but
  has no parent-child or value-provenance links.
- `CallableAnalysis.dependencies` also includes body-level lookups unrelated
  to the normal result; it is used for recheck scheduling, not as result proof.
- AST/range matching is only a heuristic beyond direct returned-call roots:
  nested expression and local/branch flow dependence is not represented.
- The adopted INC-C3-23 ruling prohibits a general result fixpoint and requires
  diagnostics to correspond to unresolved result cycles.

## 10. Implementer's causal hypothesis

A diagnostic based on aggregate dependencies or recursive AST call collection
can report a cycle that does not affect the inferred result. A sound general
implementation needs result provenance propagated with values through
expression composition and flow/binding analysis, then retained per normal
exit and included in relevant product fingerprints.

## 11. Alternative hypotheses

- A root returned-call identity can classify the exact direct/mutual examples,
  but does not cover calls whose values contribute through nested expressions
  or locals.
- If CV-08 is intentionally bounded to direct returned-call cycles, that
  limitation must be explicit and accepted as the diagnostic contract.
- If CV-08 requires general result-dependency SCCs, richer propagated
  provenance is required and materially expands the body-product contract.
- Aggregate body dependencies may be useful for scheduling/invalidation but
  are not, by themselves, proof of result dependence.

## 12. Attempts already made

- No CV-08 production attempt was made after the architect ruling.
- The aggregate dependency graph was rejected before implementation because it
  admits false result-cycle edges.
- A read-only source census checked AST, ranges, per-expression call products,
  normal exits, and publication inputs. Direct returned-call roots can be
  recognized, but nested value flow cannot be soundly recovered from current
  products. No edits, builds, or tests were made for CV-08.

## 13. Why local authority is insufficient

The minimal sound general design appears to require stable callable result
dependencies propagated through expression composition and flow/binding
values, recorded per normal exit, and incorporated into callable-product
fingerprints. That changes semantic body-product and incremental-invalidation
contracts. The adopted ruling requires sound SCC classification but does not
authorize that broader provenance architecture or define whether CV-08 may be
limited to direct returned-call cycles. This crosses the mandatory
consultation boundary for semantic authority, product representation, and
incremental invalidation.

## 14. Decision requested

For CV-08, which scope is authorized?

1. Limit T5 to classification of direct returned-call cycles and explicitly
   defer nested/local result-dependency cycles; or
2. Expand the semantic body product to carry stable result dependencies per
   normal exit, propagated through nested value-producing expressions and
   flow/binding values and included in product fingerprints, then build the
   classification-only SCC graph from that evidence.

If option 2 is required, specify the ownership and propagation contract for
branch joins, operators, and returned locals. The existing AST/range and
aggregate dependency products are insufficient proof.

### Adopted disposition — Option 1

Option 1 is adopted for CV-08 edge authority. The supported edge is only the
canonical selected `CallableId` of a normal-return root expression that is
itself a call (tail call or explicit `return call`). Do not infer edges by
syntax walking or source-range joins, and do not attach edges for locals,
nested calls, branches, operators, or other value provenance. Include this
direct-result identity in the callable body product fingerprint. After
ordinary inferred-result publication stabilizes, classify unresolved omitted-
result SCCs and emit deterministic diagnostics; do not add a result fixpoint.
Explicit declared result contracts remain authoritative seeds.

This settles the edge-source/representation question raised here; it does not
settle incremental retraction of inferred-only signatures when a previously
acyclic body edit creates a direct-result cycle. That separate issue is tracked
in [`INC-C3-25.md`](INC-C3-25.md). T5/G4 and P4 remain incomplete while that
incremental parity question is open.
