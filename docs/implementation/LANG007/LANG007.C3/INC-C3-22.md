# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `LANG007`
- Checkpoint: `LANG007.C3`
- Plan: `LANG007.C3.P4.R1`
- Task/Gate: `T3 / CV-04` — inferred trait-default contract consumption by override/conformance
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `e3d2f55c677b5cc6ed43c35eafb05d6e40a58e3b`
- Current revision: `e3d2f55c677b5cc6ed43c35eafb05d6e40a58e3b` (T3 changes are uncommitted)

## 2. Escalation trigger

Mandatory trigger: the required consumer behavior appears to require changing
the session lifecycle and invalidation of already-analyzed dispatch clients.
This crosses the accepted incremental-semantics boundary; no such lifecycle
change is assumed authorized by T3.

## 3. Required invariant

Keep the single canonical inferred-result owner in `CallableSignatureTable`;
do not add a second result store, infer from caller expectations, or weaken
conformance proof rules. Any conformance/override consumer that uses an inferred
trait-default result must observe the canonical contract and leave dependent
dispatch products coherent.

## 4. Expected architecture

R1 T3/CV-04 requires override/conformance consumers to use the canonical
inferred trait-member contract through existing authority. The `TraitSurface`
remains a declaration-contract projection; it must not become an independently
writable inferred-result owner.

## 5. Observed behavior

The trait default’s `CallableSignatureTable` entry contains a known `String`
inferred result. The corresponding already-published
`ConformanceWitnessPlan.requirement_views` entry has no published result
(`None`). The failing fixture gives the class witness an explicit `Int` result.
This establishes that the plan’s requirement view did not consume the canonical
inferred result; it does not by itself establish which later dispatch products
would need recomputation.

## 6. Minimal reproduction

Exact command and selector:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic conformance_witness_uses_canonical_inferred_trait_default_contract -- --nocapture
```

Observed assertion:

```text
assertion `left == right` failed
  left: None
 right: Some(TypeId(74))
test result: FAILED. 0 passed; 1 failed
```

The test first verifies the canonical trait-default signature publishes
`String`, then checks the plan’s instantiated requirement view for that same
requirement.

Expected behavior: the conformance/override consumer must use the canonical
inferred trait-default contract, so the plan requirement view must represent
`String`; an `Int` witness cannot be treated as satisfying that contract.

## 7. Relevant code path

```text
ProgramAnalyzer session
  -> session.rs::publish_conformance_witness_plans (initial plan publication)
  -> impls.rs::instantiate_conformance_requirements from TraitSurface
  -> TraitSurfaceMember::instantiate
  -> trait-default body analysis
  -> session.rs::refresh_inferred_callable_results
  -> CallableSignatureTable publishes inferred String
```

Relevant locations:

- `phalcom-semantic/src/session.rs::publish_conformance_witness_plans` is first
  called around line 5696, before the trait-default body-analysis loop.
- `phalcom-semantic/src/session.rs::publish_conformance_witness_plans` builds
  requirement views from the `TraitSurface` around line 10175.
- `phalcom-semantic/src/impls.rs::build_conformance_witness_plan` also builds
  requirement views from the trait surface around line 1855.
- `phalcom-semantic/src/session.rs::refresh_inferred_callable_results` runs
  later, around line 6488, and publishes the inferred result to the canonical
  callable-signature table.
- No post-refresh conformance-plan publication was found in this bounded trace.

## 8. Scoped implementation diff

Uncommitted T3 work is limited to:

- `phalcom-semantic/src/checker/context.rs`: Assumed trait-default dispatch
  transiently reads the canonical callable signature after refresh and records
  its existing `CallableSignature` dependency.
- `phalcom-semantic/tests/semantic/capabilities/traits.rs`: focused inferred
  Self/generic/associated call-site test, Assumed-dispatch test, and this
  conformance-plan discriminator.

No conformance lifecycle change has been made. The unrelated user-owned staged
and modified `docs/specs/tuple-and-record-typing.md` remains untouched.

## 9. Evidence

Focused passing evidence:

- `trait_defaults_preserve_abstract_self_and_use_canonical_argument_checking` — 1/1
- `trait_default_can_use_abstract_associated_projection_and_exact_view_normalizes_it` — 1/1
- `bodyless_explicit_conformance_member_is_invalid_and_cannot_fall_back_to_default` — 1/1
- `trait_dispatch_query_selects_an_exact_trait_default` — 1/1
- `trait_default_inferred_self_generic_and_associated_results_specialize_at_calls` — 1/1
- `assumed_trait_default_dispatch_consumes_canonical_inferred_result` — 1/1

The conformance-plan discriminator above selected 1 test and failed at the
assertion shown in §6.

## 10. Implementer's causal hypothesis

The plan’s conformance requirement projection is produced before the
trait-default body’s inferred result is published. It therefore retains the
pre-inference surface contract and does not consume the later canonical table
result.

## 11. Alternative hypotheses

- The observed plan view could be intentionally defined to retain only the
  source-declared contract, with inferred results excluded from conformance
  compatibility; that would conflict with the current T3/CV-04 wording and needs
  an explicit ruling.
- A post-inference projection may be possible without changing proof strength,
  but already-analyzed dispatch clients may then require invalidation or
  reanalysis. The existing lifecycle’s intended contract for that transition is
  not established by the focused evidence.

## 12. Attempts already made

- Added and ran the focused discriminator. It confirmed the canonical table
  result is `String` while the plan view remains `None`.
- Inspected publication order and the two requirement-view construction sites.
- Made no conformance-plan, dependency, lifecycle, or proof changes.

## 13. Why local authority is insufficient

The semantic session publishes conformance plans before trait-default result
refresh, then analyzes dispatch clients against those plans. A post-refresh
update could change conformance completeness and dispatch eligibility; deciding
how that update invalidates or rechecks existing clients is an incremental
semantic lifecycle decision, not a local projection edit.

## 14. Decision requested

Where and when should inferred trait-default contracts be consumed for
override/conformance validation, and what existing-authority invalidation or
reanalysis step is authorized to reconcile dispatch clients already analyzed
against the pre-inference conformance plans—without creating a second result
owner or an unauthorized fixed point?
