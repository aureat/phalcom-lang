---
id: LANG007.C3.P4.R1
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: corrective-implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on:
  - LANG007.C3.P4
follows: LANG007.C3.P4
supersedes: null
repository: aureat/phalcom-lang
prepared: 2026-09-29
prepared_against_revision: 9cc5e40de9895c19d4cccbd8382c6e723b426eb9
title: "Canonical Inferred Result Contracts"
---

# LANG007.C3.P4.R1 — Canonical Inferred Result Contracts

> Luna patch-grade corrective plan under supervisor review; authorized by
> `INC-C3-21.md`. This is separate from P4 T13: it does not certify G5/G6 or
> close P4/C3.

## 0. Executor contract

- T1 is a bounded, read-only source census. Do not edit production code until
  the supervisor reviews its source map and causal hypothesis.
- Reuse the existing canonical callable-signature and inferred-result
  authority. Do not add a trait-default result table/cache/registry, a parallel
  signature authority, dispatch-local body analysis, or consumer-side inference.
- Preserve explicit result annotations, bodyless declaration behavior, caller
  independence, symbolic generic/`Self` results, and the existing substitution
  authority.
- Keep the checkpoint current at meaningful gates. Run only focused selectors
  named here; preserve unrelated staged/unstaged/untracked work.
- Stop and consult on any trigger in §12. The user-owned `AM`
  `docs/specs/tuple-and-record-typing.md` is outside scope: do not edit, stage,
  or commit it.

## 1. Objective

For a body-bearing callable with no result annotation—including a trait
default—analyze the provider body in its canonical declaration generic/`Self`
environment, infer one symbolic result, and publish it through the ordinary
callable-signature authority. Calls, chains, trait defaults,
override/conformance, metadata, fingerprints, and incremental consumers must
use that same contract.

An explicit result annotation remains authoritative and the body is checked
against it. Bodyless declarations remain unchanged. No runtime result
inference or general recursive fixpoint inference is added.

## 2. Accepted architecture (FIXED)

1. Ordinary callables and trait defaults share one canonical result
   representation/publication path.
2. Caller expected types may constrain a call site but never set or mutate the
   provider contract. Trait applicability, conformance proof, and dispatch
   selection remain distinct from result inference.
3. `Self`, declaration generic parameters, and associated projections may be
   canonical symbolic results, specialized through existing use-site
   substitution. Durable products contain no `InferVarId`, `RecordRowVarId`,
   query-local proof IDs, or inference-local terms.
4. Acyclic inference dependencies are allowed. Do not add recursive widening or
   general fixpoint inference. Unseeded semantic cycles require a deterministic
   diagnostic; sufficient explicit result contracts may seed a resolvable
   cycle.
5. Causally classify `Iterator.take`'s `RecursiveFixpoint`. Its source body
   `TakeIterator.new(self, n)` is not syntactically self-recursive. Do not
   require an annotation unless a genuine semantic cycle is proven; repair an
   artificial analysis/staging dependency at its owner.
6. Reuse existing signature metadata representation. No schema change without
   evidence and architect approval.
7. Interface fingerprints change when inferred result contracts change. A
   body edit preserving the inferred result does not change the result
   interface fingerprint (body/code fingerprints may change). The same rule
   applies to explicit-result body edits preserving the declared result.
8. Update the general callable/method/typechecking normative owner, not the
   iterator spec. Resolve any material conflict before changing normative text.
9. Restore the natural collection chain only after both required stages have
   canonical results; then rerun only the directly blocked
   `corpus::collections` selector in T7.

## 3. Checkpoint boundary

P4/C3 stays `IN_PROGRESS / PARTIAL / UNVERIFIED`. R1 does not certify P4 G5
collection aggregates, G6, workspace, or release acceptance. Do not change
`TypeStore::union`, flow joins, row solving, compiler/VM behavior, runtime ABI,
generic runtime authority, or dispatch selection to make a result appear known.
Priority 3 failures remain separate unless R1 causality is demonstrated.

## 4. Bounded live-tree anchors (VERIFY-FIRST)

T1 verifies exact owners before any edit. Starting points:

- `phalcom-semantic/src/session.rs`:
  `InferredCallableRefreshInputs`, `refresh_inferred_callable_results`,
  `publishable_inferred_return`, `DownstreamCallableBodyState`,
  `declaration_signature_source_fingerprints`. Class method signatures enter
  the table before body analysis near 4430; trait-default bodies enter
  `callable_analyses` near 5693; refresh skips a body near 9913 when
  `get_for_body` finds no table entry.
- `phalcom-semantic/src/signature.rs`:
  `CallableSemanticSignature` and `CallableSignatureTable`.
- `phalcom-semantic/src/traits.rs`: `build_trait_surface` stores a
  `CallableSemanticSignature` in each `TraitSurfaceMember`; `instantiate`
  currently specializes `declared_return` only. This existing surface is a
  projection, not authorization for a second inferred-result authority.
- `phalcom-semantic/src/db/query.rs` and
  `phalcom-semantic/src/checker/body.rs`: trait-default body analysis consumes
  a `TraitSurface` signature/dependency; verify how consumers observe the
  canonical result and invalidate it.
- `phalcom-semantic/src/checker/body.rs` and
  `checker/declaration_signature.rs`: body analysis and explicit/unannotated
  result setup.
- `checker/declaration_signature.rs::project_semantic_signature` currently
  projects `declared_return`; trait call resolution passes
  `TraitSurfaceMember.signature` through this projection.
- `phalcom-semantic/src/checker/inference.rs` and `checker/call.rs`: symbolic
  results and use-site substitution; verify the actual owner.
- `phalcom-semantic/src/db/fingerprint.rs`: signature result fingerprint
  (around 492) and trait-surface member signature fingerprint (around 1525).
- `phalcom-semantic/src/metadata/export.rs`: exporter receives
  `CallableSignatureTable` and reads inferred results for callable records
  (around 137 and 1953); it does not receive `TraitSurface`.
- Semantic integration tests and the core collection corpus harness: T1 finds
  exact test modules and selectors.

Expand this map only when a traced dependency proves another owner is involved.

## 5. Invariants and required coverage

| ID | Obligation |
|---|---|
| INV-01 | Ordinary callables and trait defaults publish one canonical result through the same authority. |
| INV-02 | Provider contract is independent of caller expected type and invocation order. |
| INV-03 | Explicit annotations remain authoritative; incompatible bodies fail; bodyless declarations stay unchanged. |
| INV-04 | Generic, `Self`, and supported associated results remain durable symbolic forms until ordinary substitution; no local IDs escape. |
| INV-05 | Calls/chains, dispatch eligibility, override/conformance, metadata, and incremental consumers agree on the canonical contract. |
| INV-06 | Changed result changes interface fingerprint; result-preserving body edit does not. |
| INV-07 | No general recursive fixpoint; genuine unseeded cycles diagnose; artificial staging cycles are fixed at their owner. |
| INV-08 | R1 does not recategorize Priority 3 or certify T13/G5/G6. |
| CV-01 | Ordinary-vs-default parity and caller-context independence. |
| CV-02 | Explicit result, incompatible body, and bodyless controls. |
| CV-03 | `Self`, generic, and associated symbolic results specialize correctly without local IDs. |
| CV-04 | Overrides/conformance consume result contracts through existing authority. |
| CV-05 | Result changes invalidate dependents; same-result and explicit-result body edits keep interface fingerprint stable. |
| CV-06 | Cross-module metadata roundtrip reuses existing signature representation. |
| CV-07 | Cold/incremental agreement for changed and same-result bodies and a dependent consumer. |
| CV-08 | Direct/mutual cycle controls establish explicit-seed policy and deterministic diagnostic. |
| CV-09 | `Iterator.take` `RecursiveFixpoint` causally attributed; natural chain reaches typed receivers and canonical Derived evidence. |

## 6. Tasks and gates

### T1 — Bounded source census (read-only)

Trace one ordinary inferred result from body analysis through signature
publication, dependency/fingerprint, substitution, metadata, and consumers.
Compare an omitted-result trait default along the same path. Identify whether
the gap is producer branch, body-analysis scheduling, later publication, or
consumption. Record exact live functions/types, test modules/selectors, and the
smallest ordinary/default discriminator. No production edits or broad tests.

**Deliverable:** compact source map and causal hypothesis to the supervisor;
update §4 with verified anchors if needed. Supervisor review is a prerequisite
for T2.

**T1 outcome (2026-09-29, read-only):** ordinary class signatures enter
`CallableSignatureTable` before body analysis. Trait default signatures are
stored in `TraitSurfaceMember`; their bodies are analyzed and retained in
`callable_analyses`, but `refresh_inferred_callable_results` skips them when
`get_for_body` finds no table entry. The analysis records a `TraitSurface`
dependency; no inferred result is published into the trait member signature.
`project_semantic_signature` projects `declared_return`, and trait call
consumers project `TraitSurfaceMember.signature`.
The metadata exporter consumes the callable table, while trait-surface and
callable-signature fingerprints are separate. This confirms a
producer/publication gap, not an architecture blocker. No paired omitted-result
ordinary/default publication test was found; no files or tests were changed in
T1.

Existing selectors: ordinary
`capabilities::callable_publication::publication::branch_derived_tail_type_is_published_to_unannotated_callable_signature`;
trait-surface
`capabilities::traits::trait_surface_publishes_all_behavior_shapes_before_default_body_analysis`;
incremental
`incremental::traits::trait_body_edit_reuses_surface_and_unrelated_default_but_refreshes_changed_body`
(currently annotated defaults). Minimal new control: an ordinary class method
and omitted-result trait default each return the same known literal; compare
the ordinary inferred/published result with the default body's normal-exit
summary and canonical trait signature, without caller expected-type evidence.

### T2 — Shared publication and parity

Extend the existing canonical producer so body-bearing trait defaults without
explicit results publish through the same signature path as ordinary inferred
results, preserving declaration identity and canonical generic/`Self` context.
T1 verified that `project_semantic_signature` currently returns
`declared_return`, and trait consumers project `TraitSurfaceMember.signature`.
Determine and implement the narrow existing-authority path by which those
consumers observe the canonical result; do not merely populate a table entry
that no trait call reads. Keep body checking constrained by `declared_return`;
the inferred result must not become its own body-analysis premise.
The existing `TraitSurfaceMember` may expose a projection, but it must consume
the same canonical signature/result; do not create independently mutable
inferred-result copies or a trait-specific cache. Ensure existing trait call,
body-analysis, and dispatch-eligibility consumers observe the contract through
that authority and dependencies track the signature they actually consume.
Do not add inference in `trait_dispatch.rs` or compiler lowering. If this
consumer integration requires a second result owner or creates a dependency
cycle that cannot be resolved within the ruling, stop and consult rather than
duplicating state. Add paired
ordinary/default semantic tests and prove caller expected type cannot seed a
provider result. Use T1's exact test location/selector.

**G1:** CV-01 passes in focused semantic tests; filters select real tests. No
core corpus selector run.

### T3 — Explicit, symbolic, and consumer controls

Test explicit compatible result, incompatible body rejection, unchanged
bodyless declaration, `Self` result (including trait default), generic and
supported associated symbolic results, normal call-site substitution, and
override/conformance consumption. Inspect durable products for forbidden
inference-local/query-local identities.

**G2:** CV-02, CV-03, and the INC-C3-22 conformance lifecycle matrix pass in
focused semantic tests. G2 includes canonical inferred/explicit requirement
projection, compatible/incompatible witnesses, authoritative plan replacement,
stable plan identity, stale client/dispatch retirement, focused cold/incremental
parity, same-result plan reuse, and no result/conformance feedback. The full
matrix is recorded in `INC-C3-22.md`; none of these obligations may be waived
by passing the original CV-04 discriminator alone. The conformance-specific
incremental cases belong to G2; CV-05 through CV-07 remain T4's broader
signature, metadata, and dependent-consumer acceptance. No runtime/ABI tests.

### T4 — Dependencies, fingerprints, metadata, incremental behavior

Use existing dependency machinery. Changed result must invalidate dependents;
same-result inferred body edit and explicit-result body edit preserving the
declared result must keep the result interface fingerprint stable. Round-trip a
cross-module result using existing metadata. Add cold-vs-incremental comparisons
for changed and same-result bodies with a dependent consumer. No new schema or
cache; if existing form is insufficient, stop and consult.

**G3:** CV-05 through CV-07 pass in T1-identified narrow selectors, with exact
selected/passed counts recorded.

**Ruling amendment (INC-C3-22, 2026-09-29):** T3/G2 additionally owns the
conformance-plan lifecycle and focused conformance-client invalidation matrix
described above. This does not move or remove T4's broader CV-05–CV-07
signature-interface, cross-module metadata, and general dependent-consumer
acceptance.

### T5 — Recursion and `Iterator.take` attribution

Test direct/mutual omitted-result cycles without seeds and with sufficient
explicit contracts. Unseeded semantic cycles must use a deterministic
diagnostic; seeded cycles use declared canonical results. Trace
`Iterator.take` from Universe body through result dependency construction and
scheduling, distinguishing semantic recursion from staged-analysis cycle. If
artificial, correct the dependency owner and preserve acyclic/no-fixpoint
policy. Do not force an annotation, alter flow/loop logic, or change
`TypeStore::union`.

**Ruling amendment (INC-C3-23, 2026-09-30):** If no existing diagnostic
precisely represents an unseeded callable-result cycle, add one owned by
callable-result inference/publication validation. Emit one deterministic
diagnostic per unresolved result SCC while leaving results unknown. SCC
detection is classification-only: no recursive result fixpoint, caller seed,
or per-call-site errors. For a selected untyped formal, analyze without an
expected type, preserve the argument's own status, and do not manufacture a
`RecursiveFixpoint`; keep genuine blockers and unresolved required generics
fail-closed. Preserve `Iterator.take` runtime validation and do not annotate
its source declarations. INC-C3-24 records the unresolved live-data question:
how to derive sound result-specific SCC edges without using aggregate body
dependencies.

**G4:** CV-08 and `Iterator.take` causal attribution recorded with minimal
selector and before/after cause. Verify untyped-formal generic application and
genuine-blocker controls, canonical `Iterator.take` Known/Ready body and normal
publication, and unchanged invalid-argument runtime behavior (or record the
precise pre-fixture baseline blocker). No corpus aggregate. New
recursion/fixpoint architecture is STOP AND CONSULT.

### T6 — Restore the natural chain

Only after T2–T5 produce canonical contracts for both relevant stages, restore
the blocked `(0..).iter.take(10).toList` discriminator. Verify typed receiver
products and existing canonical Derived evidence; do not special-case library
names or fabricate dispatch evidence.

**G5:** exact restored-chain selector passes. No corpus/workspace run.

### T7 — Normative rule and bounded closure

Update the general callable/method/typechecking spec owner found in T1:
omitted-result inference for body-bearing callables, explicit result authority,
canonical generic/`Self` context, caller independence, recursion boundary, and
bodyless behavior. Do not amend the iterator spec or imply global static typing
or runtime inference. A material conflict with normative authority is STOP AND
CONSULT.

After the chain passes, run only the directly blocked selector:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::collections -- --nocapture
```

Confirm it selects the intended test(s); if its exact live name differs, use
T1's verified selector and record it. Do not run
`corpus::collections_literals_negative`, full core/semantic suites, P4 G5
aggregates, G6, workspace tests, or release gates.

**G-R1-final:** chain and direct `corpus::collections` selector pass; scoped code/spec
diff and records agree. P4/C3 remains partial/unverified pending its separately
authorized remaining gates.

## 7. Flexible decisions

**Mechanically flexible:** private helper decomposition, exact test module,
equivalent existing APIs, and diagnostic wording within existing policy, after
T1 confirms live ownership.

**Verify-first:** ordinary returns use the listed session refresh/signature
table; trait defaults can use canonical generic/`Self` environment; signature
representation durably encodes symbolic results; exact blocked collection
selector is the one recorded by the checkpoint. Correct mechanical drift
locally. If a second authority, new environment, schema/identity expansion, or
broader test gate is required, STOP AND CONSULT.

## 8. Verification budget and lifecycle

- BUILD through T5: exact affected semantic selectors only, serially, using
  pinned toolchain and `RUSTFLAGS='' RUSTC_WRAPPER=''`; no broad suites.
- Confirm test filters select actual tests. For semantic failure record
  `Observed / Hypothesis / Evidence / Prediction / Discriminating test` before
  one serious correction. Same failure after one serious attempt: stop/consult.
- Enter STABILIZE only after G1–G5; `G-R1-final` is limited to the direct
  collection selector above. Do not infer P4 G6 certification from R1 results.
- Update the checkpoint at meaningful gates. At completion create
  `LANG007.C3.P4.R1-walkthrough.md` and
  `LANG007.C3.P4.R1-handoff.md`. R1 may be
  `IMPLEMENTED / FOCUSED_TESTED`; P4/C3 must remain
  `IN_PROGRESS / PARTIAL / UNVERIFIED`.

## 9. STOP AND CONSULT

Stop before editing if the work requires a second signature/result source;
caller-dependent provider contracts; recursive widening/fixpoint; runtime,
VM, dispatch-local, or compiler body inference; serializing local inference
identity; changing generic `Self`/associated identity or override rules;
metadata schema redesign; weakening `ProperTypeId`, `TypeStore::union`, row,
or flow-join invariants; a material spec/test-oracle conflict; or scope beyond
the adopted ruling. The packet must provide exact task/invariant, minimal
reproducer, symbols/call path, observed facts, causal hypothesis and
alternatives, attempt made, trigger, and precise decision requested.
