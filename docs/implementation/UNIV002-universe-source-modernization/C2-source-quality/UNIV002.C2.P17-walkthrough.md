---
id: UNIV002.C2.P17
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# UNIV002.C2.P17 walkthrough — Either transformation documentation

## Outcome

Added comments immediately above `Either.mapRight`, `Either.bimap`, and
`Either.swap` in `packages/universe/src/functional/either.ph`. They document
which callback applies to each variant and that `swap` reverses the variant
while retaining its payload.

## Scope and evidence

- Only three comments changed. Method signatures, bodies, imports, module
  ownership, representation, and identity are unchanged.
- The comments do not describe Left as an error or add callback, purity,
  exception, or timing guarantees.
- Exact diff review and scoped `git diff --check` passed. No tests were run
  because no executable source changed.
- Independent comparison against MOD1 confirmed 174 matching `.ph` paths,
  with no additions or removals and sixteen changed hashes. The current
  `functional/either.ph` hash is
  `508502608736ae31a51f550d1959d4e5f70b7931c2b077370fb56c8c5baf23d1`.
- Source commit: `9d03621c86ae1abab4a7f5db170953e1ead3e92d`.
- Shadow semantic linking remains `UNVERIFIED`; shadow runtime remains
  `BLOCKED`.

## Follow-up

P18 documents application order for `Arrow.then` and `Arrow.compose` in
`packages/universe/src/functional/arrow.ph`, without changing their
implementation.
