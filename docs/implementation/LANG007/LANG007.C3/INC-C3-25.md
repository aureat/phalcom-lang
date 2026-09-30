# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C3
- Plan: LANG007.C3.P4.R1
- Task/Gate: T5 / CV-08 — incremental direct-result SCC classification
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `1823ac4f9084ccf34d77b01cc463ca5181ce3fe2`
- Current revision: `1823ac4f9084ccf34d77b01cc463ca5181ce3` (with the scoped, uncommitted CV-08 implementation/test diff)

## 2. Escalation trigger

The required incremental cold-parity case exposes a semantic publication and
invalidation ambiguity. Resolving it may require retraction and reanalysis of
already-published inferred results, beyond the adopted ruling's
classification-only SCC diagnostic.

## 3. Required invariant

- Explicit declared result contracts remain authoritative seeds.
- Unseeded direct-result cycles remain unknown and receive one deterministic
  diagnostic per SCC.
- A direct-result edge must come only from the canonical selected `CallableId`
  on a normal-return root call; aggregate body dependencies are not result
  proof.
- Incremental add/remove of a direct edge must publish/retire the diagnostic
  and match cold analysis, without a recursive result fixpoint.

## 4. Expected architecture

Under the adopted Option 1 disposition in [`INC-C3-24.md`](INC-C3-24.md),
CV-08 uses only direct root-call result edges stored on each
`NormalReturnFact`, included in the body product fingerprint, and an SCC
classification after ordinary inferred-result publication stabilizes. It does
not authorize a result fixpoint, general provenance, or compiler/VM inference.

## 5. Observed behavior

The focused incremental test starts with two methods whose side-effect calls
are followed by independent literal results. That state has no direct result
edges and no CV-08 diagnostic. It then changes both tails to direct calls to
each other. Both recomputed normal-return facts contain the expected opposite
canonical `CallableId`, but both facts remain `Known(Int)` with
`EvidenceOrigin::CallableSignature`; both signatures still have an omitted
declared result and retain an inferred `Known(Int)`. The snapshot contains no
CV-08 diagnostic, and the test's required Unknown assertion fails.

The direct and mutual cold fixtures do produce Unknown results and one
deterministic CV-08 diagnostic. The mismatch is specific to the incremental
transition from previously inferred, independently known results into an
unseeded direct cycle.

## 6. Minimal reproduction

Exact command:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic direct_result_cycle_diagnostics_track_incremental_edges_and_match_cold_analysis -- --nocapture
```

Selected 1 test; failed 1. Stage B facts showed:

```text
left.normal_returns[0].direct_result_dependency = Some(Probe.right)
right.normal_returns[0].direct_result_dependency = Some(Probe.left)
both normal-return facts = Known(TypeId(68), Established, CallableSignature)
both declared_return = Unknown(UnannotatedDeclaration)
both inferred_return = Some(Known(TypeId(68), Established, CallableSignature))
CV08 diagnostics = []
```

The assertion that `left`'s published result is Unknown failed. Update stats
reported two callable bodies recomputed and two callable signatures recomputed
(with zero callables counted as recomputed and two reused by the legacy
counter). A cold direct-cycle fixture is already covered by the passing
focused semantic test and yields Unknown plus one CV-08 diagnostic.

Expected: after the A-to-B edit, both omitted-result signatures and their
normal-return products are Unknown and the same one-SCC diagnostic appears in
incremental and cold snapshots. After removing one direct edge, the diagnostic
retires and both snapshots agree again.

## 7. Relevant code path

```text
checker::body / checker::statement
  -> root-call TypedExpression.callable copied into NormalReturnFact
  -> callable_body_product_fingerprint hashes the direct CallableId
  -> refresh_inferred_callable_results publishes normal-return summaries
  -> post-refresh SCC classifier updates diags_by_module
  -> SemanticSnapshot freezes diagnostics and callable products
```

Relevant files/symbols:

- `phalcom-semantic/src/checker/analysis.rs`: `NormalReturnFact` and the
  root-call syntax guard.
- `phalcom-semantic/src/checker/body.rs` and `checker/statement.rs`: tail and
  explicit-return fact construction.
- `phalcom-semantic/src/db/fingerprint.rs`: `hash_normal_return_fact`.
- `phalcom-semantic/src/session.rs`: `refresh_inferred_callable_results` and
  post-refresh `refresh_unresolved_callable_result_cycle_diagnostics`.
- `phalcom-semantic/src/diagnostic.rs`: `DiagnosticCode::CallableResultCycle`.
- `phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs`:
  incremental add/remove and cold-parity discriminator.

## 8. Scoped implementation diff

The uncommitted CV-08 diff adds direct root-call dependency capture on normal
returns, hashes it in the callable body product, adds the CV-08 diagnostic and
post-refresh SCC classifier, and adds focused semantic controls. It does not
change the inferred-result refresh policy to retract previously published
inferred results. No temporary diagnostic instrumentation remains. Unrelated
pre-existing workspace/audit files are not part of this incident diff.

## 9. Evidence

Focused selectors completed in this continuation:

- `recursive_inference_fails_honestly_without_inventing_unit_or_nominal_type` —
  1/1 pass; mutual unseeded cold cycle is Unknown and reports one SCC
  diagnostic with source-first anchoring and a related-member label.
- `direct_unseeded_result_cycle_is_reported_deterministically_without_fabricating_a_type`
  — 1/1 pass; direct cold self-cycle remains Unknown and diagnostic identity,
  message, anchor, and labels are repeatable.
- `non_result_recursive_calls_do_not_form_callable_result_cycles` — 1/1 pass;
  side-effect recursion with independent results has no diagnostic, and a
  returned local has no direct edge and remains Unknown without CV-08.
- `explicit_mutual_recursive_publication_uses_declared_results` — 1/1 pass;
  declared results remain Int and suppress CV-08.
- `direct_result_cycle_diagnostics_track_incremental_edges_and_match_cold_analysis`
  — selected 1, failed 1 at the known-to-cycle incremental transition described
  above.

An explicit self-seed assertion was added to the existing
`explicit_recursive_publication_uses_declared_result` test but has not been
rerun since that assertion was added. No corpus, workspace, invalid-`n`, or
broad test gate was run.

## 10. Implementer's causal hypothesis

The edited bodies are reanalyzed while their previous inferred Int signatures
are still available. The new direct calls therefore receive Known(Int), and
their normal-return summaries republish that same inferred value. The SCC
classifier currently filters to omitted-result signatures whose published
knowledge is Unknown, so these stale-revision-derived Known signatures are
excluded even though the current direct-result graph contains an unseeded SCC.

## 11. Alternative hypotheses

- The prior inferred result may be intentionally retained as a valid seed
  across a body edit; if so, the cold-parity expectation needs a precise
  semantic contract.
- The inferred result may be stale and must be retracted when the new
  direct-result graph proves an unseeded SCC; doing so may require a bounded
  reanalysis of SCC members and affected consumers.
- A different non-fixpoint invalidation sequence may already be authorized by
  the result-publication architecture, but that ownership rule is not stated in
  INC-C3-24.

## 12. Attempts already made

- Added direct root-call edges and fingerprint coverage as authorized. Cold
  direct/mutual cycles and non-result false-positive controls passed.
- Added an incremental known-results-to-cycle-to-acyclic test. It confirmed
  that the edge identities are present, but the first stage-B result check
  failed: existing inferred Known(Int) results persisted and no diagnostic was
  published.
- No correction to result publication, reanalysis scheduling, or signature
  ownership has been attempted.

## 13. Why local authority is insufficient

INC-C3-24 adopts classification-only SCC diagnostics after inferred-result
publication stabilizes. It does not say whether classification may invalidate
inferred-only signature products that were previously valid but become
self-supported after an incremental edit, nor whether members/consumers may be
reanalyzed once after that invalidation. Clearing those results without
rechecking leaves body products with stale Known facts; rechecking may change
the publication lifecycle and risk introducing a recursive iteration policy.
This is a semantic publication and incremental-invalidation decision, not a
local graph or diagnostic wording choice.

## 14. Decision requested

For an omitted-result direct-call SCC discovered after an incremental edit,
may the implementation perform a one-shot invalidation of inferred-only
signature results for the SCC followed by a single reanalysis that leaves the
cycle Unknown, while preventing the same stabilized refresh from republishing
those results? If so, specify the bounded affected-consumer rule and how this
remains distinct from a recursive result fixpoint. If not, identify the
alternative contract that satisfies incremental add/remove cold parity while
keeping declared results authoritative and preserving the no-fixpoint policy.
