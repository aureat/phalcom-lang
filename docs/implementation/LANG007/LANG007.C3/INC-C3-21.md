# IMPLEMENTATION INCIDENT AND ADOPTED RULING

## 1. Identity

- Incident: INC-C3-21 — canonical result contract for unannotated trait defaults
- Program: LANG007
- Checkpoint: LANG007.C3
- Related plan: LANG007.C3.P4 T13; accepted implementation scope is the separate companion plan LANG007.C3.P4.R1, not a T13 amendment
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `5b629c8f5f04b275cafa7726bb3f4ce5a60e5173`
- Ruling received against current `main`: `9cc5e40de9895c19d4cccbd8382c6e723b426eb9`
- Status: ADOPTED; bounded implementation authorized under `LANG007.C3.P4.R1`

## 2. Escalation trigger

The follow-on requires deciding whether semantic authority should publish body-inferred return contracts for unannotated trait-default callables. That affects a canonical semantic product and its consumers, generic and `Self` substitution, recursion/fixpoint behavior, dependency invalidation, and metadata/incremental compatibility. This triggers the mandatory cross-layer/semantic-authority consultation conditions; local implementation must not choose the architecture.

## 3. Required invariants

- Trait dispatch consumes a canonical known receiver and the existing conformance-proof authority; it must not guess receiver types or create a dispatch-local inference authority.
- Compiler/lowering consumes selected semantic products and must not re-solve return inference or trait applicability.
- Preserve the distinction between `Unknown(UnannotatedDeclaration)` and a proven canonical result; do not turn unknown into `Dynamic` or a permissive dispatch candidate.
- The adopted INC-C3-20 ruling remains authoritative for T13: T13 does not add trait-default return inference, and G5/G6 remain unpassed/incomplete absent a separately authorized scope.

## 4. Architecture question at escalation time

INC-C3-20 established Outcome B for T13: unannotated trait-default body returns are not currently published as a canonical return-signature contract consumable by trait dispatch. The known-receiver `receiver.toList` gate closes T13's focused dispatch matrix without changing that fact.

At escalation time the unresolved choice was whether unannotated trait-default body returns should acquire a canonical inferred-result contract or require explicit result signatures. The architect has now resolved that question; the adopted contract is recorded in §15 and implemented only through the companion plan. This packet's earlier observations and alternatives remain historical evidence, not current unresolved decisions.

## 5. Observed behavior

Historical focused evidence recorded in [INC-C3-20](INC-C3-20.md): `Iterable::iter` is selected exactly, but its unannotated default return is `Unknown(UnannotatedDeclaration)`. The chained `(0..).iter.take(10)` and `.toList` products remain unknown, with no canonical `Iterable::toList` dispatch product. `Iterator::take` also encounters `RecursiveFixpoint`. The natural chain remains blocked/ignored and is not T13 acceptance evidence.

The amended known-receiver source gate using `[1, 2].iterator` publishes guarded Derived `Iterable::toList` evidence, retains the nested Exact Iterator proof/dependencies, and passes its ineligible-Int negative control. The Exact/Derived selections also survive compiler lowering unchanged. These results isolate the deferred question to result-contract publication; they do not prove a solution for unannotated trait-default returns.

## 6. Minimal reproduction

Historical pre-ruling command (not rerun for this packet):

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-core --test core \
  unbounded_take_to_list_keeps_guarded_iterable_dispatch_evidence -- --nocapture
```

It selected one test and failed because the actual `.take(10).toList` chain did not publish an evidenced `Iterable::toList` dispatch. The selector is now retained as blocked/ignored under INC-C3-20, not as passing evidence.

## 7. Relevant code path

```text
Iterable.iter { self.iterator }
  -> exact RangeIterator / Iterable::iter selection is published
  -> selected signature return is Unknown(UnannotatedDeclaration)
  -> chained take/toList receiver knowledge remains Unknown
  -> no canonical receiver/proof query publishes Iterable::toList
```

Relevant live files/symbols already inspected for INC-C3-20:

- `phalcom-core/core/universe/src/collections/iterator_laws.ph`: `Iterator::take`, `Iterable::iterator`, `Iterable::iter`, `Iterable::toList`, and the guarded blanket `Iterable` implementation.
- `phalcom-semantic/src/trait_dispatch.rs::resolve_trait_evidenced_candidates` and its canonical `query_generic_conformance_proof` path.
- `phalcom-semantic/src/checker/context.rs`: publication of `TraitDispatchSite::Evidenced`.
- `phalcom-core/src/modules/semantic_lowering.rs`: projection of already-selected evidence; no proof search.
- `phalcom-core/src/compiler/lib/expr.rs` and `phalcom-core/src/vm/dispatch.rs`: existing selected-default lowering/dispatch, not changed for this issue.

See INC-C3-20 §§5–14 for the detailed source-range, product, and attempt record; this packet does not claim a fresh source audit.

## 8. Scoped implementation diff

None. This is a decision packet only. The earlier T13 focused-matrix changes are recorded in the checkpoint/walkthrough and already pushed. The unrelated user edit to `docs/specs/tuple-and-record-typing.md` remains untouched.

## 9. Evidence

- INC-C3-20 recorded the historical failing natural-chain selector and the `Unknown(UnannotatedDeclaration)` products.
- The known-receiver positive and ineligible-receiver negative selectors each passed 1/1; the ordinary trait-lowering Exact/Derived preservation selector passed 1/1.
- Explicit `cargo check -p phalcom-semantic` and `cargo check -p phalcom-core` passed with an existing dead-code warning.
- The current checkpoint records the §13 focused matrix as satisfied, but G5 collection aggregates are unpassed/uncertified and G6 is incomplete.
- No tests or source changes were made for INC-C3-21.

## 10. Implementer's causal hypothesis

The immediate chain lacks a canonical result contract after `Iterable::iter`; downstream receiver typing therefore cannot reach ordinary canonical trait-applicability resolution. `Iterator::take` adds a separate recursive/fixpoint concern. This is a bounded hypothesis from the recorded products, not proof that return inference is the only or preferred remedy.

## 11. Alternative hypotheses

- A body-result product may already exist but not be published or consumed by the signature used for trait defaults.
- The language may intentionally require explicit result contracts for trait-default methods when downstream static chaining depends on them.
- The `RecursiveFixpoint` blocker may require a distinct design even if non-recursive default returns can be inferred.

These alternatives were not resolved by T13's known-receiver matrix.

## 12. Attempts already made

- T13 used the architect-authorized known-receiver positive and negative controls to test guarded dispatch independently of return inference; both passed.
- T13 did not implement trait-default return inference, modify Universe declarations, special-case `TakeIterator`, change compiler/VM behavior, or rerun the blocked chain/collection aggregates.

## 13. Why local authority is insufficient

Choosing inference versus explicit contracts changes accepted semantic behavior and may add a canonical result product consumed across semantic analysis, incremental invalidation, metadata/snapshots, compiler lowering, and library declarations. Recursive trait-default bodies and generic/`Self` substitution make the ownership and convergence rules nonlocal. The T13 ruling explicitly defers this to a separate semantic design/remediation item.

## 14. Decision requested (historical)

The original consultation requested a choice between canonical inferred result
contracts and explicit result signatures, plus ownership and lifecycle rules.
The architect response supersedes this request and is recorded in §15.

## 15. Adopted architect ruling and bounded authorization

The architect adopted canonical inferred result contracts for body-bearing
callables with omitted result annotations, including trait defaults. Analyze
the provider body in its canonical declaration generic/`Self` environment and
publish one canonical symbolic result through the ordinary callable-signature
authority. Explicit annotations remain authoritative; bodyless declarations
remain unchanged. Caller expected types cannot set provider contracts, and
callable result inference remains distinct from trait applicability,
conformance proof, and dispatch selection.

`Self`, generic parameters, and associated projections may remain symbolic and
are specialized at call sites through existing substitution. Durable result
products must exclude inference-local variables and query-local proof
identities. Acyclic dependencies are allowed, but general recursive
fixpoint/widening is not. Unseeded semantic cycles require a deterministic
diagnostic; artificial staged-analysis cycles should be repaired at their
dependency owner.

Reuse the existing callable signature representation for metadata. No schema,
runtime, compiler/VM/ABI, dispatch-local inference, override-rule,
`TypeStore::union`, row-solver, or flow-join changes are authorized. Update the
general callable/method/typechecking normative owner, not the iterator spec;
stop if current normative authority materially conflicts before editing.

The architect requires causal attribution of `Iterator.take`'s
`RecursiveFixpoint`: `TakeIterator.new(self, n)` is not syntactically
self-recursive. Do not require an annotation merely to hide an artificial
dependency cycle. Restore `(0..).iter.take(10).toList` only after both relevant
stages publish canonical contracts, then rerun only the directly blocked
`corpus::collections` selector. No broad G5, G6, workspace, or release
certification is included.

Implementation identity, authorized under the ruling, is `LANG007.C3.P4.R1` in
[`LANG007.C3.P4.R1-canonical-inferred-result-contracts.md`](LANG007.C3.P4.R1-canonical-inferred-result-contracts.md).
T1 is read-only; production edits require supervisor review of its source map
and causal hypothesis. P4/C3 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`.

## 16. T1 source census and supervisor disposition

The read-only T1 census found that ordinary class callable signatures are
inserted into `CallableSignatureTable` before body analysis
(`session.rs` near 4430). Trait-default signatures are built into
`TraitSurfaceMember` (`traits.rs` near 477), while their bodies are analyzed
into `callable_analyses` (`session.rs` near 5693). The shared
`refresh_inferred_callable_results` loop skips a body when
`callable_signatures.get_for_body(callable)` has no entry (`session.rs` near
9913); that is the concrete current publication gap.

Trait-body analysis records a `TraitSurface` dependency and consumes that
surface signature; `project_semantic_signature` currently projects
`declared_return`, and `TraitSurfaceMember::instantiate` specializes only that
declared result. Callable result fingerprints include `inferred_return`, while
trait-surface fingerprints include member signatures. Metadata export reads
inferred return from `CallableSignatureTable` and has no `TraitSurface` input.
The ordinary publication test exists, as do trait-surface and incremental
trait-body controls, but the incremental default control is explicitly
annotated; no paired omitted-result default publication test was found.

Supervisor review confirms the gap is in publication/consumer projection, not
body-result availability. This is not an architecture blocker by itself.
Proceed with P4.R1 T2 only if all consumers observe the single canonical
callable-signature result and no parallel inferred-result state is introduced.
T1 made no code changes and ran no tests; P4/C3 and the P4 G5/G6 dispositions
are unchanged.
