# Specification — List Literal Syntax `[a, b, c]`

> **Status:** **Proposal.** The one genuinely *un*specified list concern
> ([`core-classes.md`](./core-classes.md) §6, DEFERRED #6): surface literal syntax
> for `List`. This doc is the design ratified by
> [ADR-0028](../../../adr/0028-list-literal-syntax.md) (**Proposed** — awaiting
> sub-decision ratification); parser/compiler work follows in U-LEX. Grounded in
> [ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md) (native `List`),
> [ADR-0021](../../../adr/0021-no-truthiness-enforcement.md) (no truthiness),
> [ADR-0016](../../../adr/0016-hand-written-lexer-and-recursive-descent-parser.md)
> (lexer/parser). Inherits the baseline pin from [`README.md`](./README.md).
>
> **Owner:** U-LEX (surface syntax) + a new ADR (the literal decision).

## 1. Problem

Today a list can only be built imperatively — `List.new().add(a).add(b)`
([`core.ph`](../../../../phalcom-core/core/core.ph) `List#add`). The catalog
(`object-model.md` §3) lists `[1, 2]` → `List` as surface syntax, but the lexer has
no `[`/`]` handling and the parser has no list-literal production, so `[a, b, c]`
does not parse. `[` is currently an **unused** token — no indexing syntax exists
(element access is the `at(_)` message), so there is no grammar conflict to resolve.

## 2. Surface grammar

```
list-literal := '[' (expr (',' expr)* ','?)? ']'
```

- `[]` — the empty list.
- `[a]`, `[a, b, c]` — one or more elements.
- **Trailing comma** permitted: `[a, b,]` ≡ `[a, b]`.
- Elements are arbitrary expressions; **nesting** is ordinary (`[[1, 2], [3]]`).
- Elements evaluate **left-to-right, eagerly** (§4).

Precedence: a list literal is a **primary** expression (same tier as a parenthesized
expression or a block literal), so `[a] + [b]` parses as `([a]) + ([b])`.

## 3. Desugaring

`[a, b, c]` lowers to the existing floor/`.ph` protocol — **no new runtime
semantics**, no truthiness, ordinary sends:

```
[a, b, c]   ≡   List.new().add(a).add(b).add(c)
[]          ≡   List.new()
```

Two lowering strategies (the ADR picks one):

| Strategy | Mechanism | Trade-off |
|---|---|---|
| **A — desugar to sends** (recommended) | parser emits the `List.new().add(_)…` AST | zero new bytecode; reuses `list_class_new`/`rawPush`; `add` returns `self` so the chain composes |
| **B — dedicated `BuildList(n)` opcode** | compiler pushes *n* values, one opcode builds the `List` | avoids *n* message sends per literal; needs a new `Bytecode` variant + VM handler + disassembler row |

Strategy **A** ships within U-LEX with no floor change (the derivability default,
[ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md) §1). Strategy **B**
is a later performance optimization if literal-heavy code shows the send overhead —
it does **not** change the surface or the value produced.

## 4. Semantics

- **Evaluation order:** elements evaluate left-to-right before the `List` is
  returned (matches call-argument order).
- **Value:** a fresh, mutable `List` (`ListObject`) — same object `List.new()`
  yields; identity per literal (two `[1]` literals are `!=` by identity but `==` by
  structure once `List#==` lands, U-CORE-5).
- **Absence:** `[]` is an empty list, **not** `None` — a list is never absence
  ([ADR-0021](../../../adr/0021-no-truthiness-enforcement.md)).

## 5. Non-goals (this spec)

- **Index sugar `x[i]` / `x[i] = v`.** Element access stays the `at(_)` / `at(_, put:)`
  message. Whether to add subscript sugar is a *separate* proposal; keep `[…]`
  meaning *construction only* here.
- **Map/Set/Tuple/Range literals** (`{a: 1}`, `Set(…)`, `(a, b)`, `1..5`) — see
  [`map-and-set.md`](./map-and-set.md) and [`tuple-and-range.md`](./tuple-and-range.md).

## 6. Open sub-decisions (for the ADR)

| # | Question | Recommendation |
|---|---|---|
| LL-1 | Desugar-to-sends (A) vs dedicated opcode (B). | **A** now; **B** only if profiled. |
| LL-2 | Add subscript sugar `x[i]` in the same unit? | **No** — keep `[…]` construction-only; propose subscripting separately. |
| LL-3 | Trailing comma allowed? | **Yes** — matches nothing to break and eases codegen/formatting. |

## 7. Test strategy

Golden fixtures (`tests/lang/collections/`): `[]` → empty; `[1,2,3].size` → `3`;
`[1,2,3].at(1)` → `2`; nested `[[1],[2]].at(0).at(0)` → `1`; trailing comma parity;
left-to-right evaluation order (elements with observable side effects).
