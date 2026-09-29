# IMPLEMENTATION INCIDENT

## 1. Identity

- Incident: INC-C3-21 — canonical result contract for unannotated trait defaults
- Program: LANG007
- Checkpoint: LANG007.C3
- Related plan: LANG007.C3.P4 T13; this is a separate follow-on design scope, not a T13 amendment yet
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting/current revision: `5b629c8f5f04b275cafa7726bb3f4ce5a60e5173`
- Status: architecture decision requested; no production implementation authorized by this packet

## 2. Escalation trigger

The follow-on requires deciding whether semantic authority should publish body-inferred return contracts for unannotated trait-default callables. That affects a canonical semantic product and its consumers, generic and `Self` substitution, recursion/fixpoint behavior, dependency invalidation, and metadata/incremental compatibility. This triggers the mandatory cross-layer/semantic-authority consultation conditions; local implementation must not choose the architecture.

## 3. Required invariants

- Trait dispatch consumes a canonical known receiver and the existing conformance-proof authority; it must not guess receiver types or create a dispatch-local inference authority.
- Compiler/lowering consumes selected semantic products and must not re-solve return inference or trait applicability.
- Preserve the distinction between `Unknown(UnannotatedDeclaration)` and a proven canonical result; do not turn unknown into `Dynamic` or a permissive dispatch candidate.
- The adopted INC-C3-20 ruling remains authoritative for T13: T13 does not add trait-default return inference, and G5/G6 remain unpassed/incomplete absent a separately authorized scope.

## 4. Expected architecture / unresolved contract

INC-C3-20 established Outcome B for T13: unannotated trait-default body returns are not currently published as a canonical return-signature contract consumable by trait dispatch. The known-receiver `receiver.toList` gate closes T13's focused dispatch matrix without changing that fact.

The separate scope needs a language/semantic decision: are unannotated trait-default body returns intended to acquire a canonical inferred-result contract usable by downstream chained calls, or must such methods have explicit return signatures for that contract? No representation, inference algorithm, or consumer change is proposed here.

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

## 14. Decision requested

For the separate follow-on scope, should Phalcom publish canonical inferred result contracts for unannotated trait-default bodies, or should downstream static chaining require explicit return signatures?

If inference is intended, specify the owning semantic product/layer and the required rules for generic/`Self` substitution, recursive/fixpoint bodies such as `Iterator::take`, dependency recording and cold/incremental invalidation, and metadata/snapshot compatibility. If explicit signatures are intended, specify the contract for library declarations such as `Iterable::iter` and `Iterator::take` and whether their return annotations are required for this chaining behavior.

Please also decide whether this should become a new numbered corrective plan under LANG007.C3 or a separate semantic-design work item. No implementation should resume until the decision and scope are recorded.
