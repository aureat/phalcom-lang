# IMPLEMENTATION INCIDENT — beta-reduction arena cloning

## 1. Identity

- Program: `LANG007`
- Checkpoint: `LANG007.C3`
- Plan: `LANG007.C3.P2.R1`
- Task/Gate: `T9`, `CL-35`, `CV-P2-21`
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Incident starting revision: `9c150747262275b34e52cb7b6642d7d3ceb4ad4d`
- Current revision at ruling: `d24f7097be5cc5fa353309ea1ecbcaadea50e867`

## 2. Escalation trigger

Mandatory trigger 11 from the repository implementation contract: changing
type-lambda arena access during beta reduction may affect canonical lambda
identity and captured/nested application semantics. The current call path has
two plausible no-copy designs with materially different representation and
failure behavior.

## 3. Required invariant

`INV-24` / `CV-P2-21`: one beta reduction must not copy work proportional to
the complete accumulated `TypeLambdaArena`. This is also explicit in the
canonical P2 plan's T3 resource requirement.

## 4. Expected architecture

Reduce against the active lambda arena without cloning the complete arena,
while preserving canonical application, nested lambda capture, partial
application, and checked failure behavior. The P2 plan permits a move-out and
restore approach only if those semantics remain correct.

## 5. Observed behavior

`TypeStore::apply_static_form` clones `self.lambda_arena`, invokes
`beta_reduce` on the clone, and installs that clone back. This copies the full
lambda arena for each lambda application. The clone is present in the live tree
at the clean plan-start revision, so it is not an incidental change from the
current corrective patch.

## 6. Minimal reproduction

Static code-path reproduction:

```text
TypeStore::apply_static_form
  -> self.lambda_arena.clone()
  -> TypeLambdaArena::beta_reduce
```

Expected: no full accumulated-arena clone per beta reduction.

Observed: the clone is explicit at `phalcom-semantic/src/types/store.rs` in
`apply_static_form`.

## 7. Relevant code path

```text
TypeStore::apply_static_form
  -> TypeLambdaArena::beta_reduce
     -> subst_scoped_to_canonical
        -> store.apply_type_form for a nested scoped application
```

The nested call uses the lambda arena held by `TypeStore`. The currently
suggested move-out approach would temporarily remove that arena while the
nested application is being normalized. A safe replacement therefore must
preserve access to existing lambda IDs and ensure new IDs are canonical in the
same arena.

## 8. Scoped implementation diff

No change has been made to the beta-reduction path. The current uncommitted
patch includes the T4/T5/T6/T7/T8 work and a trait-header invalidation fix; none
of those edits changes `TypeStore::apply_static_form` or lambda arena ownership.

## 9. Evidence

- Direct live-source inspection found `self.lambda_arena.clone()` in
  `TypeStore::apply_static_form`.
- `TypeLambdaArena::beta_reduce` recursively canonicalizes scoped `Applied`
  forms through `store.apply_type_form`.
- `store.apply_type_form` can itself reduce a lambda, so the arena must remain
  available across nested reduction.
- The active corrective plan requires no complete arena copy at T9 and marks
  this as `CL-35` / `CV-P2-21`.

## 10. Implementer's causal hypothesis

The clone currently provides a mutable working arena while `TypeStore` remains
available to nested canonicalization. Removing it safely likely requires
threading the active arena through nested application/materialization or
introducing an equivalent local mutation boundary.

## 11. Architectural ruling

The architect ruled `ARCHITECTURE_RESOLVED / IMPLEMENTATION_REQUIRED /
UNVERIFIED`, with resume authorized. Use exactly one active `TypeLambdaArena`
through each recursive beta-reduction or lambda-specialization operation.
Public `TypeStore` operations may move the arena out and restore it on every
ordinary `Result` path. Private active-arena-aware application helpers must
thread that same arena through every nested application.

Do not use a transaction or journal: interning is monotonic and checked
failures currently commit any interned nodes. Do not defer `INV-P2-22` / `INV-24`.
Apply the same active-arena protocol to `substitute_type_lambda`, the second
whole-arena clone site. `TypeStore` remains canonical `TypeId` authority;
`TypeLambdaArena` remains scoped/lambda interning authority.

## 12. Implementation and attempts

Before the ruling, no correction attempt was made. The nested
`ScopedTypeData::Applied` call path showed that a direct `mem::take` substitution
could make existing lambda IDs unavailable to recursive application. The
approved solution now explicitly threads one active arena through those paths.

## 13. Required focused regressions

Retain the existing capture, shadowing, higher-kinded application, and partial
application tests. Add coverage for:

1. A freshly specialized nested lambda immediately applied in the same outer
   reduction, yielding the expected canonical type.
2. Repeated full application and repeated partial application producing
   canonical identities.
3. A checked wrong-kind failure restoring the arena while preserving existing
   lambda IDs and permitting a later valid application.
4. Free-capture specialization with a nested lambda/application, repeated
   canonical specialization, and successful subsequent application.

The focused source audit must find no `self.lambda_arena.clone()` in
`apply_static_form` or `substitute_type_lambda`. Local immutable node copies are
allowed; copying the complete accumulated arena is not.

## 14. Status

```yaml
status: ARCHITECTURE_RESOLVED
implementation: REQUIRED
verification: UNVERIFIED
resume_authorized: true
strategy: one-active-TypeLambdaArena
mechanism: active-arena threading with bounded move-out/restore at TypeStore boundary
transaction_or_journal: REJECTED
defer_INV_24: REJECTED
```

## 15. Implementation result (2026-09-24)

Implemented and focused-tested under the architect's ruling. `TypeStore` moves its
single lambda arena out at public operation boundaries and restores it on every ordinary
return. Private active-arena-aware application helpers carry that same arena through
recursive beta reduction, canonicalization, substitution, and free-capture lambda
specialization. No transaction, journal, rollback, or panic recovery was introduced.

The four new regressions and existing lambda regression group pass as part of
`foundations::type_annotations` (49/49): fresh captured nested lambda application,
repeated full/partial canonical identity, checked wrong-kind restoration with existing
IDs preserved, and nested-lambda free-capture specialization followed by application.
Both former clone sites are absent; a source search over `phalcom-semantic/src/types`
finds no whole `TypeLambdaArena` clone.

Final incident status:

```yaml
status: IMPLEMENTED
implementation: COMPLETE
verification: FOCUSED_TESTED
resume_authorized: true
strategy: one-active-TypeLambdaArena
mechanism: active-arena threading with bounded move-out/restore at TypeStore boundary
transaction_or_journal: REJECTED
defer_INV_24: REJECTED
```
