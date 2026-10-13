---
id: UNIV002.C2.P20
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# UNIV002.C2.P20 walkthrough — Iso.inverse documentation

## Outcome

Added one comment above `Iso.inverse` in
`packages/universe/src/functional/optics.ph`:

```telos
/// Exchanges the forward and backward callbacks.
```

The method constructs its result with the existing callbacks exchanged. The
comment does not claim those callbacks obey round-trip laws.

## Scope and evidence

- Only one comment changed. The method body, signature, Product constraints,
  reflection import, module ownership, and generic identity are unchanged.
- Exact diff review and scoped `git diff --check` passed. No tests were run
  because no executable source changed.
- Independent comparison against MOD1 confirmed 174 matching `.ph` paths,
  with no additions or removals and eighteen changed hashes. The current
  `functional/optics.ph` hash is
  `57f4a4b83a914259e7d4f77beb32fa25792b5f3d211eee7b7e19753db8b7e44e`.
- Source commit: `a66c51f84f50dc2476a36836cdd057e7849cb492`.
- Shadow semantic linking remains `UNVERIFIED`; shadow runtime remains
  `BLOCKED`.

## Follow-up

P21 documents the existing forward/reverse composition order of `Iso.compose`
and the conversion/focus-update/restore sequence in `Iso.modify`.
