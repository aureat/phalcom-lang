---
id: UNIV002.C2.P21
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# UNIV002.C2.P21 walkthrough — Iso composition and modification documentation

## Outcome

Added three comments in `packages/universe/src/functional/optics.ph`:

- `Iso.compose` documents that forward application runs this Iso before
  `other`, and restoration applies `other` before this Iso.
- `Iso.modify` documents conversion to the focus, applying `f`, then restoring
  a source value.

No callback law or guarantee about callback count, timing, purity, or
exceptions is claimed.

## Scope and evidence

- The exact source diff contains only the three planned comment lines. Method
  bodies, declarations, signatures, imports, Product constraints,
  reflection contract, module ownership, and generic identities are unchanged.
- Exact diff review and scoped `git diff --check` passed. No tests were run
  because the change is comments-only.
- Independent comparison against MOD1 confirmed 174 matching `.ph` paths,
  no additions or removals, and eighteen changed hashes. The current
  `functional/optics.ph` SHA-256 is
  `e0f95221731b0a0827c109afbba1510f20ce252013ee8d825e757582923e26d8`.
- Source commit: `cb385f1ecccacc38e4746e3b47d4ce5a943387e3`.
- Shadow semantic linking remains `UNVERIFIED`; shadow runtime remains
  `BLOCKED`.

## Follow-up

P22 documents the present/absent behavior of `Optional.set` and
`Optional.modify`, plus selected/unselected behavior of `Prism.modify`.
