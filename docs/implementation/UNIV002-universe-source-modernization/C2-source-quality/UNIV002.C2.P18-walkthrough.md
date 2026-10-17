---
id: UNIV002.C2.P18
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# UNIV002.C2.P18 walkthrough — Arrow composition documentation

## Outcome

Added comments immediately above `Arrow.then` and `Arrow.compose` in
`packages/universe/src/functional/arrow.ph`:

```telos
/// Applies this arrow before `next`.
/// Applies `previous` before this arrow.
```

These statements match the nested calls in `then` and the delegation in
`compose`.

## Scope and evidence

- Only two comments changed. Method signatures, bodies, imports, module
  ownership, and generic identity are unchanged.
- Exact diff review and scoped `git diff --check` passed. No tests were run
  because no executable source changed.
- Independent comparison against MOD1 confirmed 174 matching `.ph` paths,
  with no additions or removals and seventeen changed hashes. The current
  `functional/arrow.ph` hash is
  `18f48d8445d28bc4ced75f63c37fc6658de93a02170c07548c061340c5cdd4b7`.
- Source commit: `2423d7dbdd845d853dc5696c77e4fa2971924550`.
- Shadow semantic linking remains `UNVERIFIED`; shadow runtime remains
  `BLOCKED`.

## Follow-up

P19 documents the input/output mapping of `Arrow.dimap` and the preserved
first tuple component of `Arrow.second`, without changing their implementations.
