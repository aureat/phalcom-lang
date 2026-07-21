# `@ensures` — postcondition weave, and `old(...)`

- Status: **Implemented**
- Unit: U-ANNOT-CONTRACTS
- Evidence: `phalcom-core/src/compiler/attributes.rs` — `EnsuresExpander`
  (L234-321), registered at L642; `rewrite_old_calls` (L323); `rewrite_returns`
  (L386); `as_old_call` (L140); `build_check_stmt` (L1689).
- Tier: **Compile / weave** — pure AST→AST, `runtime: false`, no VM change.
- Depends on: [README.md](README.md) (the tier model, phase order) ·
  [annotations-contracts.md](../experimental/annotations-contracts.md) (the contract design)
- Related:
  [requires.md](requires.md) (the precondition sibling; `CompileMode` table) ·
  [invariant.md](invariant.md) (the outermost weave)

## Surface

`@ensures(pred)` injects a predicate check at every method exit. Legal on
**methods, getters, and setters** (`legal_targets`, L236-238); anywhere else is
`attr.illegal_target`.

```phalcom
class Order {
  @ensures(old(_count) + 1 == _count)
  add(item) { ... }
}
```

## As built

The weave is a three-part rewrite (L240-320):

1. **Hoist `old(...)`.** Each `old(sub)` in the predicate is replaced by a fresh
   `__old_N` variable, and `let __old_N = sub` is prepended to the body — so the
   snapshot is taken **before** the body runs (`rewrite_old_calls`, L323).
2. **Rewrite every `return`.** `rewrite_returns` (L386) turns each `return v` into
   a block: `{ let __result = v; <checks>; return __result }`. This is what makes
   the postcondition fire on **all** exit paths, not just the last statement. The
   rewrite recurses into nested block statements (L425).
3. **Append checks at the tail** if the body does not end in an explicit `return`
   (L291-316): the trailing expression is bound to `let __result`, the checks run,
   then `__result` is re-emitted as the body's value.

Each check is the same shape [requires.md](requires.md) uses, via
`build_check_stmt`: `pred.ifFalse { PostconditionError.new("<msg>").raise() }`.

### `old(...)` is a parse shape, not a method

`old` is never a real binding. The parser has no bare-call grammar (calls always
need a receiver), so `old(sub)` parses as an *invocation of the variable* `old` —
`MethodCall{ object: Var("old"), method: "call", args: [sub] }`. `as_old_call`
(L140) matches that exact shape. Matching the shape rather than a method literally
named `old` is what lets `old(...)` compile at all.

### `old(self)` is rejected

`contract.old_on_mutable` (L347): the `old(...)` operand must not be the whole
receiver. Capturing `self`/`super` aliases the live, mutable object — the snapshot
would be the same reference the method goes on to mutate, so it can never observe
pre-mutation state.

Anything else is accepted (a field read, a getter call, arithmetic). Phalcom is
dynamically typed with no flow analysis, so whether a given sub-expression's
*runtime value* is itself a mutable heap reference cannot be checked here — only
the unambiguous whole-receiver case is. Same floor-not-proof limit as the
truthiness ban ([ADR-0021](../../../adr/accepted/0021-no-truthiness-enforcement.md)).

### Stripping

`@ensures` is woven **only in `Debug`** (L252-254) — both `Release` and `Unchecked`
strip it. See the `CompileMode` table in [requires.md](requires.md). Purity
validation (`validate_purity`, L245) still runs unconditionally, same rationale as
`@requires`.

## Not built

- **`result` as the binding name.** [annotations-contracts.md](../experimental/annotations-contracts.md)
  and the old library sketch both say `@ensures` binds **`result`**. As built the
  binding is **`__result`**, and it is not injected into the predicate's scope by
  name — the predicate is woven verbatim into a body where `__result` happens to be
  in scope. A predicate written as `@ensures(result.isConfirmed)` compiles to a
  read of an undefined `result`, not the return value. **Divergence: the documented
  `result` surface does not work.**
- **`Contract.ensure(_)`** — as built the check is `pred.ifFalse { ... }`; no
  `Contract` module exists.
- **Reflectable contract metadata** (`MethodObject::contracts`) — plumbed
  (`ExpandCtx::strip_metadata`) but never emitted. See [requires.md](requires.md).
- **`@ensures` on constructors** — `Target::Construct` is not a legal target, and
  constructors carry no member-level attributes at all (L1620).
- **Exception exits.** The weave rewrites `return` and the fall-through tail. A body
  that exits by *raising* runs no postcondition check — correct (the postcondition
  is not claimed to hold on the throwing path), but it means "all exit paths" means
  all *normal* exit paths.
