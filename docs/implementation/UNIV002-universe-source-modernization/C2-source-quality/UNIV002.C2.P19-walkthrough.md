---
id: UNIV002.C2.P19
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# UNIV002.C2.P19 walkthrough — Arrow mapping documentation

## Outcome

Added comments immediately above `Arrow.dimap` and `Arrow.second` in
`packages/universe/src/functional/arrow.ph`:

```telos
/// Applies `before`, then this arrow, then `after`.
/// Applies this arrow to the second tuple component and preserves the first.
```

The wording follows the method bodies' direct call order and tuple mapping.

## Scope and evidence

- Only two comments changed. Signatures, bodies, imports, module ownership,
  and generic identities are unchanged.
- Exact diff review and scoped `git diff --check` passed. No tests were run
  because no executable source changed.
- Independent comparison against MOD1 confirmed 174 matching `.ph` paths,
  with no additions or removals and seventeen changed hashes. The current
  `functional/arrow.ph` hash is
  `b11292f4feb08e4ba26099183922d35d5871aef09ae8e53899354cbe3b136baf`.
- Source commit: `5c7f8a1e2675cb486f860f3d22877ef782c06b55`.
- Shadow semantic linking remains `UNVERIFIED`; shadow runtime remains
  `BLOCKED`.

## Follow-up

P20 documents the callback exchange in `Iso.inverse` in
`packages/universe/src/functional/optics.ph`, without changing its
implementation or claiming that supplied callbacks obey round-trip laws.
