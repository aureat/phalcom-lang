---
id: LANG005.C6.P4
kind: implementation-handoff
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
revision: b4207def6241cfd6cfc84d0f85202161d0a4219c
prepared: 2026-09-17
---

# LANG005.C6.P4 handoff

P4's semantic and staged-Universe implementation is complete. Verification is
`BASELINE_BLOCKED` only because the pre-existing positive cursor iteration
corpus fails at clean `HEAD`; the canonical P4 gates and the named passing
legacy lanes are green.

## Canonical products

- `Iterator`: `universe.collections.iterator_laws::Iterator`, with `Item`,
  `next -> Option<Self::Item>`, and `iterator { self }`.
- `Iterable`: `universe.collections.iterator_laws::Iterable`, with `Item`,
  `Iterator`, and `iterator -> Self::Iterator`.
- Contract: `Self::Iterator |= Iterator` plus
  `<Self::Iterator as Iterator>::Item == Self::Item`.
- Derived rule: `impl<I> Iterable for I where I |= Iterator`, in
  `phalcom-core/core/universe/src/collections/iterator_laws.ph`.
- Rule index/product: `DerivedConformanceRuleIndex` and
  `DerivedConformanceRule` in `phalcom-semantic/src/derived_conformance.rs`.
- Proof integration: `prove_constraint` / `evaluate_conformance_goal` in
  `phalcom-semantic/src/generic_evidence.rs`.
- Runtime shape: existing `RuntimeTypeEnvironment` and
  `RuntimeConformanceEnvironment`; the VM receives finite recipes and does no
  proof search.

## Transition state

P4 uses U-B staging. `collections.iterator_laws` is registered by the canonical
Universe source provider and exported by `collections/package.ph`. The legacy
`Iterable` runtime class, cursor selectors, collection inheritance, lazy
adapters, and `for` lowering are deliberately unchanged.

## P5 objective

Migrate production collections, `Range`, `Map` item behavior, lazy stateful
iterators, and `for` lowering to the canonical evidence-driven contracts, then
retire the legacy cursor protocol and runtime `Iterable` superclass only after
the public-name transition is separately planned and verified.

## Must-read files

- `LANG005.C6-CHECKPOINT.md`
- `LANG005.C6.P4-walkthrough.md`
- `docs/specs/iterator.md`
- `phalcom-core/core/universe/src/collections/iterator_laws.ph`
- `phalcom-semantic/src/generic_evidence.rs`
- `phalcom-semantic/src/derived_conformance.rs`
- `phalcom-semantic/tests/semantic/incremental/derived_conformance.rs`

## Baseline/deferred state

The positive `language-corpus::iteration` failure is reproduced at clean
`b4207def` and is deferred as a baseline runtime issue. Do not use it as proof
that the P4 law is broken. Full collection corpus and production migration are
P5 scope.

## Do not redesign

- Do not replace `Iterator => Iterable` with a compiler special case.
- Do not introduce supertraits for this law.
- Do not alter associated requirement identity or add a second projection normalizer.
- Do not move proof search into the VM.
- Do not reintroduce the cursor protocol as the public semantic iteration law.
- Do not perform collection/`for` migration while taking over P4.

## First recommended P5 commands

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::derived_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus iteration_disasm
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::derived_conformance
```
