# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C4
- Plan: LANG007.C4.P7
- Task/Gate: P7.T3 / P7.G3
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `3f8cd713`
- Current revision: `3f8cd713` (working tree contains in-progress P7 changes)

## 2. Escalation trigger

Trigger 2: a core plan premise is disproved by live repository evidence. The
existing static evaluator does not specialize composite TypeForms such as
`Boxed<T>` with the active definition's kinded argument.

## 3. Required invariant

P7-I4 requires exact repetition of definition, canonical arguments, and
relevant proof environment to be the recursion cycle. Changing-argument
recursion must proceed until a base case or budget. P7-I9 keeps TypeId identity
session-local and leaves durable identity to P8. P7-I10 requires cold exact
evaluation to agree with residual specialization.

## 4. Expected architecture

P7.T3 requires exact application keys and changing-argument recursion. The plan
uses `F<0> -> F<1> -> ...` as a step-budget discriminator and treats composite
TypeForm arguments as semantic application arguments. P7 extends the existing
evaluator and must not add a second source of type inference or a VM evaluator.

## 5. Observed behavior

The source below is accepted by the parser and checker, but evaluation reports
an exact recursive cycle after `Grow<T>` receives a composite kinded argument.
The cycle payload shows the first `Grow` application with `TypeId(146)` and then
two identical `Grow` keys with `TypeId(1213)`. Thus the composite application
does not produce a new canonical argument under the active parameter
substitution.

The P6 handoff also records that composite TypeForm substitution intentionally
fails closed in the current immutable-snapshot evaluator contract.

## 6. Minimal reproduction

```telos
data Boxed<T> { value: T }
type Grow<T: Type>: Type { Grow<Boxed<T>> }
type Start: Type = Grow<Int>
```

Command:

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic static_recursive_evaluation
```

Observed output (abridged):

```text
changing arguments must continue until the step budget, got Recursive(...)
... Grow<Kinded(TypeId(146))>
... Grow<Kinded(TypeId(1213))>
... Grow<Kinded(TypeId(1213))>
cycle_start: 2
```

Expected behavior: either the canonical argument changes on each recursive
application and evaluation reaches the step budget, or the source is rejected
as unsupported before it can be treated as an exact cycle. It must not silently
classify an un-specialized generic TypeForm as an exact repeated argument.

## 7. Relevant code path

```text
EvaluateTypeForm
  -> complete_checked_type_form
  -> checked KindedExpression argument
  -> EnterStaticApplication
  -> StaticApplicationKey(TypeId)
  -> active_application_index
```

`complete_checked_type_form` currently forwards a bare kinded parameter and
rejects a composite form containing one. `TypeSubstitution` can recursively
transform canonical TypeStore forms, but the evaluator currently receives an
immutable `&TypeStore`, so ownership/publication of newly interned TypeIds must
be decided before using it as a fix.

## 8. Scoped implementation diff

The current P7 working diff replaces definition-only active recursion state
with exact application keys and adds a residual store, residual outcomes,
proof-goal products, iteration hardening, and focused residual tests. The
in-progress application-key code is not yet complete or accepted. No attempt has
been made to add composite TypeForm substitution.

## 9. Evidence

- `P7.T3` requires argument-sensitive recursion and step-budget behavior.
- The live `complete_checked_type_form` implementation contains an explicit
  fail-closed boundary for composite substitution.
- The live P6 handoff says composite TypeForm substitution must fail closed.
- The minimal evaluator test reports repeated `TypeId(1213)` keys after the
  first changed kinded argument.
- `TypeSubstitution::apply` mutates/interns into a `TypeStore`; evaluator
  consumers currently hold an immutable canonical TypeStore reference.

## 10. Implementer's causal hypothesis

Exact cycle detection is functioning as designed for the keys it receives, but
the key is built from an argument whose composite TypeForm has not been
specialized under the active `T -> TypeId` binding. The current evaluator
boundary therefore cannot faithfully realize the P7 changing-argument
recursion contract for generic composite TypeForms.

## 11. Alternative hypotheses

- The checker may be publishing the same unresolved TypeId for `Boxed<T>` in
  each recursive body, independent of evaluator substitution.
- The exact static application carrier may require a normalized TypeTerm or
  residual identity instead of a materialized TypeId for generic composites.
- P7 may intend to defer composite type argument substitution, in which case
  its changed-argument acceptance criterion needs a narrower source form.

## 12. Attempts already made

- Added exact application key construction and exact active-key lookup.
- Added a focused `Grow<T>` test using `Grow<Boxed<T>>`.
- The test failed with repeated `TypeId(1213)` rather than budget exhaustion.
- No speculative substitution or TypeStore ownership patch was attempted.

## 13. Why local authority is insufficient

A fix may require changing the evaluator's TypeStore ownership/publication
contract, representing generic composite arguments as residual terms, or
amending P7's acceptance boundary. These choices affect canonical identity,
P8 durability, and the static evaluator API, so they are architectural.

## 14. Decision requested

Should P7 add evaluator-local canonical specialization for composite TypeForms,
and if so, what ownership and TypeId publication model preserves P7-I9 and the
existing P6 immutable-snapshot boundary? Or should P7 narrow its changed-argument
recursion acceptance to source forms already expressible by the current
canonical TypeStore contract and amend the plan accordingly?

## 15. Architect decision and amendment — 2026-10-12

The architect ruled that changed-argument recursive type application is part of
the accepted P7 behavior and that P7 needs a Level 2 amendment. Use the active
session-owned mutable `TypeStore` as the sole canonicalization owner; do not
create a fresh evaluator TypeStore. At the workspace-update boundary, pass
`Arc::make_mut(&mut self.store)` to the evaluator. The session store is already
a baseline fork, while copy-on-write keeps previously published snapshots
immutable. Generated IDs therefore remain in the existing session lineage and
continue to use the current `ReducedKinded(TypeId)`, static-value, and product
contracts.

P7 may recursively specialize checked composite TypeForms through the existing
`TypeSubstitution` and canonical type formation APIs. Preserve kind checks,
query budget charging before traversal/interning, cancellation, exact-cycle
identity, and snapshot/store-local `TypeId` scope. Do not publish a parallel
clone-qualified type store. This amends P6/T7.10's immutable-evaluator
fail-closed rule for P7 only; no separate result-transfer architecture is
needed.

The required amendment is recorded in the active P7 plan and checkpoint.
Implementation may resume on composite specialization. T3 acceptance is still
open until focused evidence is acquired.
