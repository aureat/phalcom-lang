# Annotations — Design by Contract (`@requires`/`@ensures`/`@invariant`)

- Status: **Proposed** (experimental; not ratified)
- Date: 2026-07-11
- Depends on: [annotations-core.md](annotations-core.md) (the `@` desugar pass)
- Related: ADR-0008 (error handling — only `Error` subclasses throwable), classes.md §2 (private, non-inherited fields), ADR-0013 (escaping upvalues)

## Context

Contracts are declarative logic assertions (predicates over state) embedded in
imperative methods — the Eiffel paradigm. They are the **method-table-macro**
tier: pure body-weaves, no layout impact, buildable on the current AST.

## Decision

`@requires`/`@ensures`/`@invariant` are body-weaving attributes expanded by the
core pass. `PreconditionError`/`PostconditionError`/`InvariantError` are
`Error` subclasses (ADR-0008).

### `@requires` / `@ensures` — per-method body weave

```phalcom
@requires(amount > 0)
@ensures(_balance == old(_balance) + amount)
deposit(amount) { _balance = _balance + amount }
```

Rewrites `MethodDef.body` in three AST-level steps:

1. **`old(...)` hoist** — `old` is a reserved pseudo-selector meaningful **only**
   inside `@ensures`. Each `old(sub)` is hoisted to `let __old_N = sub;` **before**
   the body; the occurrence becomes `Var("__old_N")`. Eager by-value snapshot —
   an escaping upvalue (ADR-0013) can't make the postcondition read post-state.
2. **Precondition prologue** — per `@requires(c)`, prepend
   `c.ifFalse { PreconditionError.raise("…") }`. Params/fields are in scope
   because the check sits inside the method.
3. **Postcondition epilogue** — bind the body's last expr to `let __result = …;`,
   append each `@ensures(c).ifFalse { PostconditionError.raise }`, end with
   `Var("__result")` to preserve implicit return (classes.md §4).

Multiple `@requires`/`@ensures` are **order-independent** (all conjoined) — a
property of the derive model, not the checking order.

### `@invariant` — whole-class weave

```phalcom
@invariant => _balance >= 0
```

Collected first, then folded over every **public, non-static, non-constructor**
method: synthesize private `__check_invariant()` and wrap each qualifying body
with a call on entry and before the return value. Constructors check on exit
only (object not yet built on entry).

### Contracts are reflectable, not erased (D-contract-1)

The predicate blocks are **also** stored on the `MethodObject` as metadata
(`Symbol → [Block]` side table), not only inlined. This unlocks:

- **Contract-based property testing** — a harness generates inputs satisfying
  `@requires`, runs the method, asserts `@ensures` (Clojure `spec`-instrument-and-gen
  / QuickCheck). Highest value in a language with no static checker; slots into
  the forge golden-`.ph` corpus + `verify_invariants()` workflow.
- **Introspection/docs** — `Method>>contracts`, matching Eiffel's reflectable
  assertions.

Cost: one map per class; predicates survive weaving.

### Gradual-typing direction, reserved not built (D-contract-2)

`deposit(amount @ Number)` desugars to `@requires(amount.is(Number))` — type
annotations *are* contracts (Findler–Felleisen: contracts are the runtime
semantics of gradual types). Contracts are therefore Phalcom's only typing story.
Not built now; reflectable predicates + a blame field are the substrate it needs,
so D-contract-1 must not preclude it.

## Hazards (resolved)

- **Contract inheritance ⊗ private fields.** classes.md §2 makes fields private,
  non-inherited — so a per-class `@invariant` **can only** reference this class's
  own fields. Field privacy makes local-only invariants automatic, not a
  restriction to enforce. Full Eiffel conjunction-down-the-chain is a **one-line**
  upgrade: end `__check_invariant` with `super.__check_invariant()`. Ship
  local-only (no super-send); combined stays available.
- **Early `return` skips the epilogue.** Fall-through return is handled by the
  `__result` bind; explicit `return x` bypasses it. v1: the pass rewrites each
  `return x` site to run postconditions first. (Alternative — weave at the
  compiler return-emit point — deferred.)
- **`old(...)` ⊗ mutable aliasing.** `old(_items).size` captures the *reference*;
  a later mutation makes `_items.size == old(_items).size + 1` always false.
  **Decision:** `old` is restricted to value-typed / `@data` operands and
  **rejected at expansion time** on mutable references, with a precise span.
  Silent-wrong is worse than rejected.

## What this precludes

Erasing contracts (the naive plan) would preclude property testing and
gradual-typing — so we keep them reflectable. Shipping local-only invariants
precludes nothing (combined is strictly-more-info).
