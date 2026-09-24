# IMPLEMENTATION INCIDENT — beta-reduction arena cloning

## 1. Identity

- Program: `LANG007`
- Checkpoint: `LANG007.C3`
- Plan: `LANG007.C3.P2.R1`
- Task/Gate: `T9`, `CL-35`, `CV-P2-21`
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting/current revision: `9c150747262275b34e52cb7b6642d7d3ceb4ad4d` (local implementation changes are uncommitted)

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

## 11. Alternative hypotheses

1. A move-out/restore implementation is sufficient if nested lambda reduction
   can be routed through the moved arena without changing stable IDs.
2. A local arena transaction/journal can avoid whole-arena copying while
   preserving rollback/commit behavior.
3. The current copy boundary is required by an implicit identity/capture
   invariant, in which case the P2 plan needs a material amendment rather than
   an unreviewed workaround.

## 12. Attempts already made

No correction attempt was made. The nested `ScopedTypeData::Applied` call path
shows that a direct `mem::take` substitution could make existing lambda IDs
unavailable to recursive application; experimenting with that architecture
would cross the plan's explicit semantic boundary.

## 13. Why local authority is insufficient

The change may alter the canonical owner of newly interned lambda IDs, the
availability of captured nested lambdas during recursive application, and the
failure/commit behavior of beta reduction. These are stable-identity and
representation consequences, not a local borrow-checker adjustment.

## 14. Decision requested

Choose the safe architecture for satisfying `INV-24`: should beta reduction
thread one active arena through nested application, use a local transaction
mechanism, or should the plan be amended to defer this invariant? Please also
specify the minimum identity/capture regression cases required before the
implementer resumes this portion of T9.
