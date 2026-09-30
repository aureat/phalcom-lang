# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C3
- Plan: LANG007.C3.P4.R1
- Task/Gate: T6 / G5 — restore the natural iterator chain
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Initial incident revision: `d0f611b317268bb9f113b9cf264c4c9c188b1d77`
- First post-ruling implementation revision: `c138c4c3b229e817f90695a21277b59141980a05`
- The current post-ruling evidence and its pending follow-up are recorded in §16 and INC-C3-27.

## 2. Escalation trigger

The required semantic oracle appears incorrect for the specified source, and
the plan conflicts with the accepted trait-dispatch rule: `RangeIterator`
declares compatible inherent `take` and `toList` methods, while the test
requires the `toList` call to select the trait default and carry Derived
`Iterable` evidence. Shared consultation is mandatory before changing the
source surface, dispatch behavior, or T6 acceptance.

## 3. Required invariant

- Preserve the accepted rule that a compatible concrete inherent member
  takes precedence over a trait default.
- Publish one canonical inferred result for each body-bearing callable; do not
  fabricate a trait-dispatch selection or special-case library names.
- Do not weaken result knowledge or dispatch evidence to make the gate pass.
- Keep R1 and P4/C3 `IN_PROGRESS / PARTIAL / UNVERIFIED` until their separate
  acceptance gates complete.

## 4. Expected architecture

R1 CV-09/T6 requires the exact `(0..).iter.take(10).toList` chain to expose
typed receiver products and existing canonical Derived evidence. The ignored
core test specifically searches the `.toList` expression for evidenced
`Iterable::toList` dispatch with `DerivedConformanceOrigin::GuardedRule`.
T6 prohibits library-name special cases and fabricated dispatch evidence.

The accepted trait rule says selection is an explicit/compatible concrete
witness first, then a trait default. An inherent member that owns the selector
therefore prevents that same call from being evidence for the trait default.

## 5. Observed behavior

In the analyzed fixture, `(0..).iter` is `Known/Assumed` as `RangeIterator`
(TypeId(228)) and has an Exact `Range` → `Iterable::iter` selection. The next
expression, `.take(10)`, is `Unknown(UnannotatedDeclaration)` with
`Blocked(UnknownType(NoTypeEvidence))`; it resolves to concrete
`RangeIterator::take(_)` and has no trait-dispatch site. The `.toList`
expression is `Unknown(UnannotatedDeclaration)`, `Ready`, and has no
trait-dispatch site.

`range.ph` declares both inherent methods on `RangeIterator`: `toList` at
lines 271–281 and `take` at line 284. The test's expected
`Iterable::toList` Derived selection is therefore inconsistent with the
accepted precedence rule for this receiver. This does not establish that
`RangeIterator`'s inferred method results are correct; both remain unknown in
the observed chain.

## 6. Minimal reproduction

Exact focused command:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core unbounded_take_to_list_keeps_guarded_iterable_dispatch_evidence -- --ignored --nocapture
```

The filter selected one ignored test. The unmodified test first fails its
global `!semantic.has_errors()` assertion due the previously recorded
pre-fixture Universe bootstrap diagnostics: 20 `ImplConformanceIncomplete`
and 3 `GenericInferenceUnderconstrained` diagnostics across
`iterator_laws`, `list`, `map`, and `set` modules (INC-C3-23 §§6/9).

For one diagnostic isolation pass, only that test's global assertion was
temporarily bypassed and then restored. The chain-specific lookup still failed
because no evidenced `Iterable::toList` product exists at the `.toList`
expression. No production or library source was changed.

Expected by the current plan/test: known chain receiver products and an
existing canonical GuardedRule Derived `Iterable::toList` selection at the
natural chain's `.toList` call.

## 7. Relevant code path

```text
(0..).iter
  -> Range's Iterable::iter (Exact evidence)
  -> RangeIterator
  -> inherent RangeIterator::take(_) (result Unknown)
  -> inherent RangeIterator::toList (result Unknown; no trait selection)
```

Relevant anchors:

- `docs/implementation/LANG007/LANG007.C3/LANG007.C3.P4.R1-canonical-inferred-result-contracts.md`:
  T6 and G5, lines 313–320; CV-09, line 153.
- `docs/specs/objects/traits.md`: concrete-witness precedence, line 614.
- `phalcom-core/core/universe/src/collections/range.ph`: inherent methods,
  lines 271–284; `Range` conformance, lines 291–294.
- `phalcom-core/tests/core/language/iteration.rs`: ignored discriminator and
  its `Iterable::toList`/Derived assertions, lines 27–83.
- `phalcom-semantic/src/checker/analysis.rs`: `ExpressionAnalysis.knowledge`,
  `trait_dispatch`, and per-expression `status`.

## 8. Scoped implementation diff

At the initial escalation revision, no T6 source or test change remained; the
incident, checkpoint, and plan amendment were the T6 worktree changes. After
the ruling, commit `c138c4c3` added the `TakeIterator` import to `range.ph`, a
shared non-generic-call correction for an untyped `Unknown(NoTypeEvidence)`
formal, and direct/natural-stage selectors. Current post-ruling evidence is in
§16. Temporary diagnostic probes were removed; no corpus aggregate was run.

## 9. Pre-ruling evidence at initial escalation

- The focused ignored selector selected exactly one test.
- The first unmodified run stopped at the global semantic-error assertion on
  the known 20 + 3 Universe bootstrap diagnostics.
- In the temporary diagnostic isolation pass, `.iter` was Known/Assumed
  `RangeIterator` with Exact `Range` → `Iterable::iter` evidence;
  `.take(10)` was Unknown/Blocked and selected inherent `RangeIterator::take`;
  `.toList` was Unknown/Ready and had no dispatch evidence.
- The live `RangeIterator` source owns inherent `take` and `toList` members.
- The trait spec explicitly places compatible concrete witnesses ahead of
  trait defaults.
- At the initial escalation revision, no T6 source or test diff remained.
  This is historical state and is superseded by the post-ruling record in §16.

## 10. Implementer's causal hypothesis

The old test's recorded blocker (“`Iterable.iter` remains Unknown”) is stale:
the live `.iter` product is now Known/Assumed. The next stage does not use the
generic `Iterator.take` default; ordinary dispatch resolves the concrete
`RangeIterator::take` declaration, whose inferred result is still Unknown.
Likewise, ordinary dispatch sees the concrete `RangeIterator::toList`
declaration, so the test's expected trait-default evidence cannot arise for
this call under the accepted precedence rule.

## 11. Alternative hypotheses

- T6's intended acceptance may be two separate proofs: canonical result types
  for the natural Range chain, and Derived `Iterable::toList` evidence on a
  receiver without a compatible inherent `toList` (the existing
  `ListIterator` control already exercises this shape).
- The Range library may intentionally own both operations, in which case the
  natural-chain oracle should assert the concrete selections and their
  canonical result products rather than trait-default evidence.
- T6 may intend to change the Range library surface so these operations use
  trait defaults; that would be an observable library/ownership change outside
  the current test-only correction and needs explicit authorization.
- The unknown results for concrete `RangeIterator` methods may reveal a
  remaining canonical body-result inference gap in R1, separate from the
  incorrect Derived-evidence expectation.

## 12. Attempts already made

- Ran the exact ignored selector with the original assertion: it stopped at
  global Universe diagnostics before inspecting the natural-chain product.
- Temporarily bypassed only the broad global-error assertion to isolate the
  chain: the expected evidenced `Iterable::toList` product was still absent;
  observed per-expression products are recorded in §5. The assertion was
  restored and the temporary edit removed.
- No production, library, or dispatch correction was attempted.

## 13. Why local authority is insufficient

The active plan requires Derived trait-default evidence for a call whose
receiver has a compatible inherent method, but the accepted normative rule
requires the inherent member to win. Making the planned assertion true would
require either changing the oracle, changing the library's member ownership,
or changing accepted dispatch semantics. Those choices have different
observable/API consequences. The remaining Unknown result products also need
to be separated from this oracle conflict before deciding whether R1's
canonical inference implementation needs another bounded correction.

## 14. Decision requested

How should T6/G5 reconcile the exact natural Range chain with the accepted
inherent-before-default rule?

1. Preserve the rule and revise the discriminator to assert canonical
   `RangeIterator` method products on the natural chain, while proving
   GuardedRule Derived `Iterable::toList` evidence on a receiver with no
   compatible inherent `toList`; or
2. Change the Range library surface so the natural chain selects the trait
   default, with explicit guidance about the intended ownership and whether
   that public behavior change is in scope.

Also clarify whether the still-Unknown results of inherent
`RangeIterator::take`/`toList` are an R1 inference defect to fix through the
existing canonical result contract, or should be split into a separately
scoped investigation. Do not authorize weakening inherent precedence or
fabricating dispatch evidence.

## 15. Adopted architectural ruling

**Disposition: INC-C3-26 Option 1 adopted; T6 resumes under a split-proof
acceptance.** Preserve inherent-before-default precedence and do not change
the Range library surface. The earlier concern that the natural chain's final
`toList` could never carry Derived evidence is not established: its actual
receiver cannot be known until `RangeIterator::take` has a canonical result.
Re-evaluate final `.toList` dispatch only after that result is Known.

T6-A treats the currently Unknown body-bearing `RangeIterator::take` and
`RangeIterator::toList` results as R1 defects to investigate through existing
canonical callable-result machinery. Probe each method body, normal-return
product, generic constructor call where applicable, and `CallableSignature`
publication directly. No per-method special case, annotation workaround,
Dynamic fallback, or dispatch redirection is authorized. Then assert the
natural chain's known intermediate products and actual ordinary member
selection, with cold parity.

T6-B separately proves canonical GuardedRule Derived `Iterable::toList`
evidence on a known Iterator receiver without a compatible winning inherent
member. Reuse the existing known-`ListIterator` test if it covers evidence
identity, nested Exact Iterator evidence, dependencies, and lowering; add only
missing coverage. G5 requires both halves and no corpus/workspace gate.

The checkpoint records adoption, and the R1 plan's T6/CV-09/G5 wording
reflects this split. Post-ruling implementation and current blockers are
recorded below; G5 remains incomplete.

## 16. Post-ruling implementation and verification state — 2026-09-30

Commit `c138c4c3` (`fix(core): infer RangeIterator.take result`) is on `main`
and `origin/main`. It contains exactly three implementation files:

- `phalcom-core/core/universe/src/collections/range.ph`: import the existing
  `TakeIterator` surface used by the inherent `RangeIterator::take` body.
- `phalcom-semantic/src/checker/call.rs`: do not perform assignability
  checking against an untyped `Unknown(NoTypeEvidence)` formal; other unknown
  reasons and all actual argument analysis remain intact.
- `phalcom-core/tests/core/language/iteration.rs`: direct take body/signature
  and natural `.iter`/`.take` stage assertions.

The direct `RangeIterator::take` and natural-stage selectors pass 1/1. The
natural source proves Exact `Range -> Iterable::iter`, inherent
`RangeIterator::take`, and a Ready/Known `TakeIterator<RangeIterator>` result.
The selector includes a later `.toList` expression but does not yet assert its
product.

The direct `RangeIterator::toList` probe found an upstream Dynamic type-form
boundary rather than a demonstrated callable-result publication failure:
`List<Dynamic>.new()` becomes `Dynamic(ExplicitEscape)` during type-form
resolution, so the mutable binding, `append`, terminal normal return, and
canonical inferred return remain Unknown/absent. A temporary nested-generic
fixture confirmed that a dependent `Box<T>` result with `T` supplied by a
Dynamic actual also becomes Dynamic; the probe was removed after 1/1. Whether
an explicit nested Dynamic argument should preserve a Known outer applied type
is unresolved. See pending architect consultation INC-C3-27. No production
change was made for this result.

After `.take`, the actual natural-chain receiver is
`TakeIterator<RangeIterator>`. Its final `.toList` currently has no selected
callable or trait-dispatch product because the existing `Iterator` conformance
is incomplete at `Universe::collections.iterator_laws`, impl local 20. The
known-ListIterator Derived selector and the existing Exact/Derived lowering
selector each selected 1 test but stopped at their global semantic-error
precondition on the recorded Universe conformance diagnostics; their focused
assertions were not reached. Neither is current T6-B acceptance evidence.

No conformance, trait dispatch, TypeForm representation, or precondition was
changed. No corpus or workspace gate was run. T6/G5 remains incomplete, R1 and
P4/C3 remain `IN_PROGRESS / PARTIAL / UNVERIFIED`, and Priority 3 is unchanged.
