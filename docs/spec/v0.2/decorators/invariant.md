# `@invariant` — class-level invariant weave

- Status: **Implemented**
- Unit: U-ANNOT-CONTRACTS (+ [ADR-0052](../../../adr/accepted/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md) receiver-scoped guard)
- Evidence: `phalcom-core/src/compiler/attributes.rs` — `InvariantExpander`
  (L446-460), registered at L643; the real weave in `expand_class_attributes`
  (L1661-1681) via `weave_invariant_checks`; predicate collection at L1550-1554.
- Tier: **Compile / weave** — the outermost weave (Eiffel `invariant → post → pre`).
- Depends on: [README.md](README.md) · [annotations-contracts.md](../experimental/annotations-contracts.md)
- Related: [requires.md](requires.md) (`CompileMode` table) · [ensures.md](ensures.md)

## What it does

`@invariant(pred)` on a **class** wraps every public non-static member with an
entry+exit check of `pred`.

```phalcom
@invariant(_balance >= 0)
class Account {
  var _balance
  withdraw(n) { _balance = _balance - n }
}
```

`InvariantExpander::expand` is a **deliberate no-op** (L452-459): the registry row
exists only so `attr.unknown`/`attr.illegal_target` fire. The actual weave needs the
*whole class* (every member, not just the one the attribute is attached to), so it
runs once from `expand_class_attributes` itself.

As built (L1661-1681):

- Legal target is **`Class` only** (L448-449). A standalone `invariant` clause in a
  class body parses directly into `ClassDef::invariants` and merges with any
  `@invariant` predicates (L1535, L1553).
- Woven into non-static **methods, getters, setters** (entry + exit) and
  **constructors** (exit only — the entry check is skipped, since an invariant
  cannot hold pre-construction, L1674-1677).
- Check shape is the shared `build_check_stmt` (L1689):
  `pred.ifFalse { InvariantError.new("<msg>").raise() }`.
- Purity is validated unconditionally (L1551, L1585-1589) →
  `contract.impure_predicate`.

### Stripping

Woven **only in `Debug`** (L1661) — `Release` and `Unchecked` strip it. See the
`CompileMode` table in [requires.md](requires.md).

### ADR-0052 receiver-scoped guard

[ADR-0052](../../../adr/accepted/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)
rules the re-entrancy guard **receiver-scoped**, and rules per-receiver decorator
state Layout-confined (no side table keyed on receiver).

## Not built

- **The re-entrancy guard itself.** ADR-0052 Fix 1 is cited by `InvariantExpander`'s
  own doc-comment, but the weave at L1661-1681 emits a plain entry/exit check with
  **no guard flag** — a self-send from inside a woven method re-checks the
  invariant mid-mutation. **Divergence: the ADR is ratified, the guard is absent.**
- **Static members / private members.** The weave skips `is_static` members; it does
  not otherwise distinguish "public" — every non-static method/getter/setter is
  woven regardless of naming convention.
- **Reflectable metadata** — see [requires.md](requires.md).
