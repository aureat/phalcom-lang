# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `LANG007`
- Checkpoint: `LANG007.C3`
- Plan: `LANG007.C3.P4.R1`
- Task/Gate: `T5 / G4` — recursion policy and `Iterator.take` attribution
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `33b66a7b72772724ad7fdfb225db1031a1306936`
- Current revision: `b40c8e773bd1793fb1dbf4fca5871fef079c3f1e`

## 2. Escalation trigger

Mandatory triggers: **#11**, because plausible fixes have materially different
language-semantic consequences; and **#16**, because R1 requires an existing
deterministic diagnostic for unseeded cycles while the live behavior emits no
diagnostic. The `Iterator.take` observation also crosses generic-call
application and canonical inferred-result publication, beyond a local
mechanical correction.

## 3. Required invariant

Preserve R1 `INV-02`, `INV-03`, and `INV-07`: caller expectations do not define
provider results; declared results remain authoritative; no general recursive
fixpoint is introduced; genuine unseeded cycles must follow the adopted
deterministic policy; and artificial staging cycles, if demonstrated, are
corrected at their owner. `Iterator.take` is not to receive an annotation just
to hide a suspected cycle. Do not alter `TypeStore::union`, flow/loop logic,
row solving, compiler/VM behavior, runtime ABI, or dispatch selection.

## 4. Expected architecture

R1 T5 requires direct and mutual cycle controls both without a seed and with
sufficient explicit result contracts. It also requires causal attribution of
`Iterator.take`'s `RecursiveFixpoint`: distinguish semantic recursion from a
staged-analysis dependency, preserve an acyclic/no-fixpoint policy, and repair
an artificial cycle only at its owning dependency boundary. The natural
collection chain remains outside this gate until T2–T5 establish both required
canonical contracts.

## 5. Observed behavior

The direct and mutual unseeded fixtures each produce recursive call products
with `Unknown(UnannotatedDeclaration)`, `AnalysisStatus::Ready`, and an empty
diagnostic list. Thus they do not currently provide the plan's requested
deterministic diagnostic. The new explicit direct and mutual `Int`-contract
fixtures publish the declared canonical `Int` in both signature and recursive
call products, without error diagnostics.

In the canonical Universe, the `Iterator.take` body product exists before
`refresh_inferred_callable_results`. Its `TakeIterator.new(self, n)` expression
has a known, specialized result, but is marked `Blocked(RecursiveFixpoint)`.
The constructor's second parameter has no declared type; generic application
analyzes the argument without an expectation and records that blocked status.
That blocked normal-return fact publishes as `Unknown(InferenceBlocked)`, so
the enclosing trait-default result cannot be published. This evidence does
not establish a semantic callable/query cycle.

## 6. Minimal reproduction

For the `Iterator.take` probe, a temporary one-off test ran the canonical
Universe analyzer against the inline entry `let marker = 1` (no `.iter.take`
chain). Exact command:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-core --test core \
  language::iteration::__t5_diagnostic_take_body_products -- --exact --nocapture
```

The temporary test selected 1 and passed 1. It was removed after collecting
the products; its diagnostic instrumentation is not in the current tree.
Captured products:

```text
callable: Universe::collections.iterator_laws::Iterator.take(_), instance
source range: [749,788)
TakeIterator.new(self, n): range [761,786), Known(TypeId(1291)),
  status Blocked(RecursiveFixpoint), call=None
specialization: TakeIterator.I -> TypeId(776), matching self's TypeId(776)
self: Known(TypeId(776)), Ready
n: Unknown(NoTypeEvidence), Ready
callable analysis: Complete; return validation: NotApplicable
normal-return summary: Unknown(InferenceBlocked)
published signature: declared Unknown(UnannotatedDeclaration), inferred None
body diagnostics: none
```

The direct/mutual unseeded controls selected and passed 1/1 each, but observed
no diagnostics. The new explicit-seed direct/mutual controls selected and
passed 1/1 each. Their exact selectors are listed in §9; the test additions
are commit `b40c8e77`.

## 7. Relevant code path

```text
iterator_laws.ph::Iterator.take
  -> TakeIterator.new(self, n)
  -> checker/call.rs:1586-1590 sees constructor parameter.ty() == None
  -> argument analyzed without expectation; call status Blocked(RecursiveFixpoint)
  -> checker/analysis.rs::NormalReturnFact::publication_knowledge
  -> Unknown(InferenceBlocked)
  -> session.rs::refresh_inferred_callable_results cannot publish a known result
```

Relevant locations:

- `phalcom-core/core/universe/src/collections/iterator_laws.ph:23` —
  unannotated `Iterator.take(_ n) { TakeIterator.new(self, n) }`.
- `phalcom-core/core/universe/src/collections/iterator_laws.ph:659` —
  constructor `new(_ upstream: I, _ n)`; the `n` parameter is untyped and the
  body retains runtime validation for numeric/integer range.
- `phalcom-semantic/src/checker/call.rs:1585-1590` — generic application path
  for a missing formal parameter type; it records `RecursiveFixpoint`.
- `phalcom-semantic/src/checker/analysis.rs:343-355` — blocked normal-return
  status is converted to `Unknown(InferenceBlocked)` for publication.
- `phalcom-semantic/src/session.rs:6495` — inferred-result refresh runs after
  the initial callable-body analysis containing this product.

## 8. Scoped implementation diff

No production or diagnostic instrumentation remains. Commit `b40c8e77`
contains only focused semantic tests for explicit-result direct and mutual
recursive callables. The temporary
`phalcom-core/tests/core/language/iteration.rs` probe was removed. The unrelated
modified workspace file, supplied audit report, and
`docs/__unstructured__/Packs.md` were not staged or changed by this incident.

## 9. Evidence

- Temporary canonical-Universe probe: `Iterator.take` body is present before
  result refresh; `TakeIterator.new(self,n)` has a known specialized result but
  blocked status; `self` is Known/Ready; untyped `n` is Unknown/Ready.
- Source at `checker/call.rs:1586-1590` marks a generic call blocked when the
  selected formal parameter has no type, after analyzing the argument without
  an expected type.
- Source at `checker/analysis.rs:352` maps a blocked return fact to
  `Unknown(InferenceBlocked)`.
- Existing direct unseeded selector
  `semantic::capabilities::callable_publication::recursive_publication_does_not_fabricate_a_result`
  passed 1/1; observed unknown recursive call, Ready status, no diagnostics.
- Existing mutual unseeded selector
  `semantic::capabilities::callable_publication::publication::recursive_inference_fails_honestly_without_inventing_unit_or_nominal_type`
  passed 1/1; both calls unknown, Ready, no diagnostics.
- New direct explicit-result selector
  `semantic::capabilities::callable_publication::explicit_recursive_publication_uses_declared_result`
  passed 1/1.
- New mutual explicit-result selector
  `semantic::capabilities::callable_publication::publication::explicit_mutual_recursive_publication_uses_declared_results`
  passed 1/1.

Exact commands for these four controls, in that order:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::callable_publication::recursive_publication_does_not_fabricate_a_result -- --exact --nocapture
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::callable_publication::publication::recursive_inference_fails_honestly_without_inventing_unit_or_nominal_type -- --exact --nocapture
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::callable_publication::explicit_recursive_publication_uses_declared_result -- --exact
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::callable_publication::publication::explicit_mutual_recursive_publication_uses_declared_results -- --exact
```

## 10. Implementer's causal hypothesis

The `RecursiveFixpoint` at `Iterator.take` is a fail-closed generic-call
application status caused by the missing type on the constructor's `n`
parameter, not evidence of recursive callable/query scheduling. Because the
body's normal-return status is blocked, result publication suppresses the
otherwise known specialized constructor type.

## 11. Alternative hypotheses

- The current policy may intentionally require a declared parameter type
  before a generic application can be considered complete, even when the
  result specialization is known; in that case the unknown provider result may
  be expected and the plan must be reconciled.
- The specialized result may be sufficient to publish a callable result while
  the untyped argument remains separately unknown; changing this would alter
  generic-call/result-publication semantics and may affect other call sites.
- The runtime validation in `TakeIterator.new` may be an intentional part of
  accepting non-`Int` inputs and raising `ArgumentError`. Adding `Int` to the
  trait or constructor parameter could change that public behavior rather than
  merely repair a cycle.
- The unseeded-cycle contract may intentionally represent unresolved recursive
  results as `Unknown(UnannotatedDeclaration)` without an error diagnostic; if
  so, the R1 CV-08 diagnostic requirement needs amendment rather than a newly
  invented diagnostic.

## 12. Attempts already made

- Ran the temporary canonical-Universe diagnostic probe against the minimal
  inline entry. It showed a known specialized call result but blocked status
  at missing-formal-type handling; the probe was removed afterward.
- Ran exact existing unseeded direct and mutual cycle selectors. Both retained
  unknown results and emitted no diagnostics.
- Added focused explicit-seed direct and mutual controls. Both publish `Int`
  and pass 1/1; no production behavior changed.
- No source annotation, generic-call rule, diagnostic, result publication, or
  scheduler change was attempted.

## 13. Why local authority is insufficient

The plan and current implementation differ on whether unseeded cycles should
produce a diagnostic. Separately, deciding whether an untyped argument should
block a generic call whose result specialization is known is a language-level
inference/publication contract. Source annotations and changing generic-call
status handling have materially different consequences, especially because
`TakeIterator.new` currently validates its argument at runtime. Neither choice
can be made as a local mechanical fix without changing accepted behavior or
weakening an evidence boundary.

## 14. Decision requested

1. For CV-08, is the intended rule that an unseeded direct/mutual callable
   cycle emits a deterministic diagnostic, or is the current stable
   `Unknown(UnannotatedDeclaration)` result with no diagnostic intentional? If
   a diagnostic is required, identify the existing diagnostic/code path to use
   or authorize a new diagnostic and its owner.
2. For CV-09, should a generic constructor call with a known specialized result
   but an untyped formal argument remain blocked from result publication? If
   yes, clarify whether the `RecursiveFixpoint` reason is intended and whether
   `Iterator.take` must declare a parameter contract despite its runtime
   validation. If no, specify the permitted generic-call/result-publication
   policy that preserves argument-checking and runtime semantics without
   introducing a recursive fixpoint.

Until these decisions are recorded, T5/G4 remains partial, no production
changes are authorized on this seam, and T6/natural-chain acceptance remains
not started.
