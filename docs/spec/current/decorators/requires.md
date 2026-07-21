# `@requires` — precondition weave

- Status: **Implemented**
- Unit: U-ANNOT-CONTRACTS
- Evidence: `phalcom-core/src/compiler/attributes.rs` — `RequiresExpander`
  (L170-232), registered at L641; `validate_purity` (L161); `build_check_stmt`
  (L1689). Fixtures: `phalcom-core/tests/lang/errors/annotation_construct_own_fields.ph`
  (uses the same weave shape), `tests/lang/compile-errors/annotation_unknown_error.ph`
  (registry legality).
- Tier: **Compile / weave** — pure AST→AST, `runtime: false`, no VM change.
- Depends on: [README.md](README.md) (the tier model, phase order) ·
  [annotations-contracts.md](../experimental/annotations-contracts.md) (the contract design)
- Related:
  [ensures.md](ensures.md) (the postcondition sibling; `old(...)` lives there) ·
  [invariant.md](invariant.md) (the outermost weave) ·
  [error-handling.md](../error-handling.md) (`Error#raise()`)

## Surface

`@requires(pred)` injects a predicate check at method entry. It is legal on
**methods, getters, and setters** (`legal_targets`, attributes.rs L172-174) — and
on nothing else; anywhere else is `attr.illegal_target`.

```phalcom
class Order {
  @requires(qty > 0)
  place(qty) { ... }
}
```

Multiple `@requires` on one member weave in **declaration order**, each as its own
check, all prepended to the body's prologue.

## As built

The weave is one statement per predicate, prepended to the member's body
(attributes.rs L216-228). `build_check_stmt` (L1689) produces:

```phalcom
pred.ifFalse { PreconditionError.new("<msg>").raise() }
```

- The check is a `ifFalse { ... }` send on the predicate — **not** a `Contract.require(...)`
  call. There is no `Contract` module on HEAD.
- `Error#raise()` takes no message argument, so the message is baked into a
  constructed instance first (`PreconditionError.new(msg).raise()`), per
  `build_check_stmt`'s own comment.
- The message is `Precondition failed for method `<name>`: <span-start>` — it cites
  the predicate's **source offset**, not the predicate text.

### Purity

`validate_purity` (L161) rejects a predicate containing mutating or side-effecting
operations with `contract.impure_predicate`. `is_pure_expr` (L103) is a
conservative syntactic check: assignments, `SetProperty`, `SetIndex`, sends named
`add`/`remove`/`put`, and any selector ending in `=` are impure; everything else
recurses structurally. It is a floor, not a proof — the same
"floor-not-proof" limitation as the truthiness ban
([ADR-0021](../../../adr/accepted/0021-no-truthiness-enforcement.md)).

**Purity runs in every `CompileMode`**, including modes that strip the guard. This
is deliberate (attributes.rs L184-190): purity is a compile-time soundness floor,
not a runtime guard, so stripping the *guard* must not silently skip catching an
impure predicate. The implementer flagged this as a judgment call — the plan does
not state it explicitly.

### `old(...)` is rejected

A `@requires` predicate containing an `old(...)` call is
`contract.old_in_precondition` (L219, and again at L194 on the stripped path).
`old(...)` is meaningful only inside `@ensures`.

### Stripping — `CompileMode`

`CompileMode` (attributes.rs L26-36) is selected on the CLI (`--release` /
`--unchecked`, default `Debug`) and threaded through `ExpandCtx`:

| Mode | `@requires` guard | `@ensures` guard | `@invariant` guard | Metadata (default) |
|------|------|------|------|------|
| `Debug` (default) | woven | woven | woven | retained |
| `Release` | **woven** | stripped | stripped | retained (opt out `--strip-contract-metadata`) |
| `Unchecked` | **stripped** | stripped | stripped | stripped by default |

`@requires` is the **only** contract guard that survives `Release` (L191-198) — a
precondition guards the caller's contract, which is exactly what you keep in a
release build.

## Not built

- **`Contract.require(_)`** — [annotations-contracts.md](../experimental/annotations-contracts.md)
  and the old stdlib sketch both weave to a `Contract.require(pred)` call. As built
  it is `pred.ifFalse { ... }` and no `Contract` module exists.
- **Reflectable contract metadata.** `ExpandCtx::strip_metadata` (L53) exists and is
  threaded through, and `CompileMode`'s own doc-comment table names
  `MethodObject::contracts` as the second, independent stripping axis — but **no
  metadata is ever emitted**. The flag is plumbed to a consumer that does not exist.
  A spec/code divergence worth surfacing, not smoothing over.
- **`@requires` on constructors.** `Target::Construct` is not in `legal_targets`, and
  `expand_class_attributes`'s member loop hands `Construct` members an empty
  attribute vector unconditionally (L1620, L1649) — a constructor cannot carry any
  member-level attribute at all. `@invariant` still weaves into constructor bodies
  (class-level, see [invariant.md](invariant.md)).
- **Span-precise diagnostics.** Every error in this module is
  `CompilerError::Message(String)` — a flat string, no miette span. The message
  interpolates `arg.range().start` (a raw offset) instead of rendering the span.
