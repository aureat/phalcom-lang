# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `LANG007`
- Checkpoint: `LANG007.C4`
- Plan: `LANG007.C4.P7`
- Task/Gate: `T10 / G10`
- Repository: `aureat/phalcom-lang`
- Branch: `main`
- Starting revision: `cb8a068c`
- Current revision: `cb8a068c` plus uncommitted P7 work

## 2. Escalation trigger

RED triggers 2, 11, 12, and 18: source Pack construction needs a checked
variadic-lane product that the current Data/Variant constructor path does not
publish. Repairing the checker constructor selection and/or allowing symbolic
locals would cross the planned T10 evaluator-only file list and has distinct
semantic-authority and work-frame consequences.

## 3. Required invariant

P7-I6 (residuals are semantic IR), I10 (cold exact equals specialization),
I14 (deterministic structure), and the T10 amendment in `INC-007`: an ordinary
Pack has a checked proper value type and ordered positional/Symbol-labeled
lanes; C3 kinded `TypePack` is separate. No evaluator-only reconstruction of
constructor resolution or unchecked alternate Pack syntax is allowed.

## 4. Expected architecture

The ordinary checker supplies a canonical Pack constructor result and source
argument-to-lane mapping. The evaluator builds a Pack residual when only some
entries are known; later specialization interns an exact Pack through
`StaticValueStore`. Both paths agree on the value and checked result domain.

## 5. Observed behavior

The T10 Pack residual now carries `result_type`, fingerprints it, and can
specialize mixed known/open entries through canonical `StaticValueStore::intern_pack`.
A focused hand-built residual passes no-new-facts identity and exact-value
parity. Source Pack construction is not yet operational in a static body.

## 6. Minimal reproduction

While developing the focused `partially_known_pack_preserves_ordered_lanes_and_specializes`
test, adding either of these definitions to its workspace fixture and using
`RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic partially_known_pack_preserves_ordered_lanes_and_specializes -- --nocapture`
gave the following results (the failing source fixture was removed from the
passing test after classification):

```telos
type PartialPack<N: Int>: Pack<***: Int> { Pack(N, named: 7) }
```

The checked body reports `GenericInferenceConflict` at `Pack(N, named: 7)`.
Moving the construction out of the result's immediate expected-type context:

```telos
type PartialPack<N: Int>: Pack<***: Int> {
    let p = Pack(N, named: 7)
    p
}
```

checks without diagnostics but evaluating with an open `N` returns
`Invalid(UnsupportedExpression)`, before it can publish a Pack residual.
An exact `Pack<*: Int, **: Int>` result and a bare `Pack` result were also
classified as kinded `Type` results by the current static-domain checker;
they cannot stand in for the declared proper value domain. The accepted
Pack specification remains unchanged.

Expected: the ordinarily checked source expression reduces when exact and
retains the positional and labeled Pack lanes when partially known.

## 7. Relevant code path

```text
checker/expression.rs::synthesize_unqualified_call
  -> ordinary data-constructor argument mapping
  -> CheckedStaticConstruction (one DataComponent per argument only)
static_definition.rs::CheckedStaticBody.static_constructions
  -> static_eval/machine.rs::Expr::UnqualifiedCall
  -> finish_checked_construction (Data or Variant only)
```

The canonical Pack declaration is variadic (`intrinsic data Pack<***Es>(***Es)`),
while the current `CheckedStaticConstruction` expects a fixed Data/Variant
payload identity for every source argument.

## 8. Scoped implementation diff

`static_eval/residual.rs` adds the proper Pack result domain;
`static_eval/specialize.rs` specializes the two lanes;
`static_eval/machine.rs` interns completed Packs through `StaticValueStore`;
`db/static_fingerprint.rs` includes the proper result type;
focused semantic tests cover domain, store identity, and exact-value parity.
No checker or work-frame ownership change was made for this incident.

## 9. Evidence

- `integration::static_recursive_evaluation` passed 20/20, including the
  standalone partial Pack test; `foundations::static_residuals` passed 13/13.
- Source fixtures above produced the specified checker diagnostic and
  evaluator terminal, respectively, before they were removed.
- The checker publishes constructor payload IDs for fixed Data/Variant
  components; the evaluator requires this checked product for
  `Expr::UnqualifiedCall` and interns only Data/Variant in that path.
- `StaticValueStore::intern_pack` preserves labeled encounter order and uses
  `StaticSymbolId`, unlike Data-component payloads.

## 10. Implementer's causal hypothesis

`Pack`'s variadic argument shape cannot be encoded by the fixed-component
`CheckedStaticConstruction`; without a checked Pack-lane selection, the
evaluator rejects the call before reducing or residualizing it. The direct
contextual constructor additionally conflicts with the current inference of
the typed two-lane remainder.

## 11. Alternative hypotheses

The direct constructor conflict may be a separate ordinary-checker inference
bug, while the absence of a Pack construction product independently explains
the otherwise accepted local-binding fixture's evaluator failure. General
symbolic-local frame support might permit open local values but would still
need canonical Pack source selection and exact execution.

## 12. Attempts already made

- Discriminated direct contextual construction from a local inferred call:
  the latter checks cleanly but still cannot execute in the static evaluator.
- Implemented only the already-authorized Pack residual result-domain and
  specialization slice; the exact canonical value and no-new-facts tests pass.
- No checker-inference or symbolic-local architecture change was attempted.

## 13. Why local authority is insufficient

Choosing a new checked Pack-lane constructor product versus deferring source
Pack acceptance changes T10's checked-source contract and task scope. Letting
the evaluator infer lanes from raw AST would violate P7-I6. Changing local
residual storage would alter evaluator work-frame and ownership semantics.

## 14. Decision requested

Should P7.T10 be amended to publish a canonical checked Pack-lane construction
product in the ordinary checker (and then resolve the contextual call conflict)
so cold/open source Pack construction satisfies G10, or should source Pack
construction be explicitly gated to a follow-up plan with G10 limited to
lower-level residual specialization? If P7 owns it, confirm that the checker
owns source lane/type selection and the evaluator only consumes that product.

## User decision — 2026-10-13

Implement checked source Pack construction in P7.T10/G10. Extend the ordinary
checker with the canonical Pack-lane product; the static evaluator consumes it
without guessing constructor identity or source lanes. Resolve the ordinary
contextual constructor conflict at its owning checker path. Keep exact source
execution and open specialization parity in G10's acceptance.

## Focused implementation outcome — 2026-10-13

The ordinary checker now records canonical `PackPositional` and
`PackLabeled(Symbol)` payload identities on `CheckedStaticConstruction` for
resolved intrinsic Pack calls. A typed open Pack result context lets the
ordinary call infer exact variadic lanes before applying the canonical result
type relation. Invalid contexts publish no static construction. The evaluator
consumes the checked mapping, interns fully known Pack values, and builds a
properly typed Pack residual for partially known arguments. Direct exact Pack
results are proven subtypes of declared open Pack results through the existing
bounded relation, as for the established Variant result-domain boundary.

An open Pack remainder no longer publishes an empty `FiniteExact` loop item
domain. Its sound uniform/unknown domain retains the checked pattern site;
an open loop source defers through the enclosing `Apply`, and exact Pack
iteration reduces through the existing O(1) cursor. Labeled cursor reads use
`StaticValueStore::pack_labeled_value_at` on the shared Pack backing rather
than allocating the whole labeled lane on each entry. Focused source tests
prove ordered mixed lanes, wrong-element rejection, no-new-facts identity,
cold/open parity, and residual-node budget behavior. Pack semantic product
fingerprinting and dependent static-constant cold/incremental parity also
pass focused tests. T10's other structural algorithms and full G10 remain
open; T0, T11–T13 and wider certification remain open.
