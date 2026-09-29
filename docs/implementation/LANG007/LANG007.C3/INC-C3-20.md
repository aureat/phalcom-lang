# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C3
- Plan: LANG007.C3.P4
- Task/Gate: T13 — guarded conformance evidence for ordinary trait dispatch
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `d99ee4d3b9ede47acab171c12221677ca8fa6adc`
- Current revision: `c687bf9e25e72b42beb4da0e708db47288e0eda8` (continuation baseline; incident started at the revision below)
- Commits made for this incident: none

## 1a. Adopted disposition — 2026-09-29

The architect ruling is adopted. The bounded return-product census established
Outcome B: unannotated trait-default body returns are not currently published
as a canonical return-signature contract consumable by trait dispatch. This is
not a claim about ordinary class callables, whose inferred-return publication
exists. T13 is not authorized to add trait-default return inference/publication.
The `Iterator.take` body is additionally blocked by `RecursiveFixpoint`. Keep the natural
iterator-chain discriminator blocked and use the amended known-receiver
source-expression gate for T13 guarded-dispatch acceptance. Priority 3 is
unchanged; P4/C3 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`.

## 2. Escalation trigger

The required actual-expression gate cannot publish guarded
`Iterable::toList` evidence because the receiver chain has no known semantic
type before dispatch. Addressing this appears to require canonical
return-signature inference/publication for unannotated trait-default bodies,
beyond the authorized selector/applicability correction. This crosses
semantic authority and a cross-layer contract, so implementation is stopped
pending direction.

## 3. Required invariant

- Applicability and proof come from the existing canonical generic
  conformance resolver; no dispatch-local proof system or type guessing.
- Runtime lowering consumes selected semantic evidence and does not re-resolve
  proofs.
- Do not change VM dispatch, language behavior, ABI, `TypeStore::union`, the
  fixture/library source, or `TakeIterator` specifically.
- Keep T13/P4/C3 `IN_PROGRESS / PARTIAL / UNVERIFIED`; Priority 3 is unchanged.

## 4. Expected architecture

For `(0..).iter.take(10).toList`, semantic analysis should publish an existing
`TraitDispatchSite::Evidenced` selection for canonical `Iterable::toList`, with
guarded Derived conformance evidence for the receiver. The compiler and VM
consume that product through their existing paths.

## 5. Observed behavior

The focused inline-method test has no semantic diagnostics and finds the
actual source expression in `Caller.inspect`. `(0..).iter` has an Evidenced
selection for `Iterable::iter`, backed by the exact `universe:collections.range`
impl (ImplLocalId 3). That selection's callable signature return is
`Unknown(UnannotatedDeclaration)`.

Products for `(0..).iter.take(10)` and
`(0..).iter.take(10).toList` are `Ready` but have
`Unknown(UnannotatedDeclaration)`, `call=None`, `denotation=None`, and
`trait_dispatch=None`. No `Iterable::toList` product is published. This is not
a test-host issue: the expression is in a callable body and its source range
maps correctly.

The source declarations are unannotated trait-default bodies:
`Iterable.iter { self.iterator }` at
`phalcom-core/core/universe/src/collections/iterator_laws.ph:34` and
`Iterator.take(_ n) { TakeIterator.new(self, n) }` at line 17. Adjacent
`Iterable.iterator -> Self::Iterator` at line 32 is annotated.

## 6. Minimal reproduction

Exact command:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-core --test core \
  unbounded_take_to_list_keeps_guarded_iterable_dispatch_evidence -- --nocapture
```

Observed output:

```text
running 1 test
... panicked: the actual take/toList chain must publish an evidenced Iterable::toList dispatch
test ...unbounded_take_to_list_keeps_guarded_iterable_dispatch_evidence ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored
```

Expected: an expression product for the actual `.take(10).toList` chain with
the `Iterable::toList` requirement and guarded Derived evidence.

No runtime fixture or `corpus::collections` aggregate was run.

## 7. Relevant code path

```text
Iterable.iter { self.iterator }
  -> exact RangeIterator / Iterable::iter selection is published
  -> selected signature return remains Unknown(UnannotatedDeclaration)
  -> chained take/toList receiver knowledge remains Unknown
  -> no canonical receiver/trait proof query publishes Iterable::toList
```

Relevant symbols/files:

- `phalcom-core/core/universe/src/collections/iterator_laws.ph`: `Iterator::take`,
  `Iterable::iterator`, `Iterable::iter`, `Iterable::toList`, and blanket
  `impl<I> Iterable for I where I |= Iterator`.
- `phalcom-semantic/src/trait_dispatch.rs`: selector candidate discovery and
  `resolve_trait_evidenced_candidates` through
  `query_generic_conformance_proof`.
- `phalcom-semantic/src/checker/context.rs`: publication of
  `TraitDispatchSite::Evidenced`.
- `phalcom-core/src/modules/semantic_lowering.rs`: projection of already
  selected Exact/Derived evidence into iteration lowering; no proof search.
- `phalcom-core/src/compiler/lib/expr.rs` and `phalcom-core/src/vm/dispatch.rs`:
  existing selected-default lowering and VM dispatch, not changed for this
  incident.

## 8. Scoped implementation diff

The uncommitted T13 diff adds guarded/derived candidate discovery via the
canonical conformance query, records proof dependencies, surfaces coherence
conflicts, and adds exact, concrete-derived, symbolic, conflict, and
actual-expression regressions. The two existing Exact/Derived iteration
selection constructors in `semantic_lowering.rs` clone the already selected
evidence into `TraitDispatchSelection`; this is data projection only. No
runtime fixture/library code or VM behavior was changed. Unrelated dirty files
remain untouched.

## 9. Evidence

Focused passing evidence:

- `RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core` — passed (existing `apply_type_form_with_arena` dead-code warning only).
- `trait_dispatch_query_selects_an_exact_trait_default` — 1 passed.
- `trait_dispatch_query_accepts_guarded_derived_evidence_without_broadening_ineligible_receivers` — 1 passed.
- `symbolic_assumed_iterator_dispatches_guarded_derived_iterable_default` — 1 passed.
- `trait_dispatch_does_not_hide_canonical_coherence_conflict_behind_another_selector_candidate` — 1 passed.

Focused failing evidence:

- Historical pre-ruling run of
  `unbounded_take_to_list_keeps_guarded_iterable_dispatch_evidence` selected 1
  and failed because no `Iterable::toList` evidence product exists; actual
  chain and Unknown knowledge are recorded above. The test is now retained as
  ignored with a blocked reason and is not acceptance evidence.

The first command variant included `--exact` and selected zero tests; it is not
counted as evidence. The non-exact focused selector above selected one test.

## 10. Implementer's causal hypothesis

The chain loses a known receiver type at `Iterable.iter`: although the exact
trait-default selection is recorded, its source return signature remains
`Unknown(UnannotatedDeclaration)`. The following `take` and `toList`
expressions therefore cannot be resolved against a known receiver, so the
canonical guarded conformance query is not reached for `toList`.

## 11. Alternative hypotheses

- The test expression was outside callable-body analysis. Eliminated by
  hosting it in `Caller.inspect` and filtering to the inline entry module; the
  matching source ranges are present.
- The trait-dispatch resolver omits a valid guarded conformance for a known
  receiver. The failing expression has no known receiver, so this run does not
  establish that hypothesis.
- Body-inferred return signatures may already exist in another canonical
  product but not be consumed in this chain. This has not been investigated
  and must be checked before choosing a remedy.

## 12. Attempts already made

- Added canonical `GenericConformanceEvidence` to existing Exact and Derived
  iteration selections by cloning the already selected evidence. This removed
  the two core construction compile errors without proof search;
  `cargo check -p phalcom-core` passed.
- Moved the actual-expression regression to module scope and wrapped the
  expression in an inline method so callable expression analysis owns it. The
  test still fails with the Unknown receiver chain above.
- Temporary expression-range logging was used for a diagnostic run and
  removed immediately afterward.

## 13. Why local authority is insufficient

Selector applicability requires a known receiver, and the ruling forbids
dispatch-local type inference or guessing. The evidence points to the result
signature of an unannotated trait-default body, upstream of candidate
applicability. Changing how that signature is inferred/published may alter
canonical semantic products, dependency/invalidation behavior, and downstream
consumers; that scope was not authorized by the adopted T13 ruling.

## 14. Decision disposition

The ruling answers the original request: Outcome B is established for
unannotated trait-default bodies, so T13 must not add canonical inferred-return
publication for that category. Ordinary class callable inference is outside
this disposition. Trait-default return publication requires a separate
semantic design/remediation item. T13's actual
expression gate is instead amended to exercise `receiver.toList` where the
receiver's concrete Iterator type is already canonically known, plus a known
ineligible receiver negative control. The natural iterator chain remains
blocked, not deleted or rewritten.

Do not resolve the natural chain by guessing in trait dispatch, editing library
or fixture semantics, special-casing `TakeIterator`, propagating dispatch
through Unknown, or changing VM search behavior.

## 20. Known-receiver gate evidence — 2026-09-29

The amended source-level positive uses `[1, 2].iterator`, whose explicit
canonical `List<T>::iterator -> ListIterator<T>` signature supplies the
concrete receiver type. The expression `receiver.toList` publishes canonical
`Iterable::toList` as `TraitDispatchSite::Evidenced`; evidence is Derived from
the guarded `Iterator => Iterable` rule, retains the nested Exact Iterator
proof, and the callable records both the guarded rule and nested conformance
dependencies. A source-level Int receiver is the negative control and receives
no Derived `Iterable::toList` evidence.

Focused selectors, each selected 1 and passed:

- `known_list_iterator_source_receiver_gets_guarded_iterable_to_list_evidence`
- `known_non_iterator_source_receiver_does_not_get_derived_iterable_dispatch`

The natural-chain selector selected 1 but was ignored with
`BLOCKED by INC-C3-20`. No runtime fixture, `corpus::collections`, workspace
test, or broad certification gate was run for this amendment. No production
semantic/compiler/VM code, Universe source, fixture source, or Priority 3
classification changed.
