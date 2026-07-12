# 29. List literals `[a, b, c]` desugar to `List` construction sends

- Status: Accepted (ratified with the collections umbrella [ADR-0032](0032-collections-representation-and-literals.md), 2026-07-12)
- Date: 2026-07-12
- Related: [ADR-0032](0032-collections-representation-and-literals.md) (collections umbrella),
  `docs/spec/v0.2/core/list-literal-syntax.md` (the design this ADR ratifies),
  `docs/spec/v0.2/core/core-classes.md` §6, `docs/spec/v0.2/object-model.md` §3,
  [ADR-0020](0020-kernel-list-native-array-protocol.md) (native `List`),
  [ADR-0021](0021-no-truthiness-enforcement.md) (no truthiness),
  [ADR-0016](0016-hand-written-lexer-and-recursive-descent-parser.md) (lexer/parser),
  [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) (frozen floor);
  DEFERRED #6

## Context

`object-model.md` §3 lists `[1, 2]` → `List` as surface syntax, but the lexer has no
`[`/`]` handling and the parser no list-literal production, so `[a, b, c]` does not
parse. A list can today only be built imperatively —
`List.new().add(a).add(b)` (`core.ph` `List#add`). This is the last genuinely
*un*specified `List` concern (`core-classes.md` §6, DEFERRED #6); everything else in
the collection family now has a spec.

`[` is currently an **unused** token: element access is the `at(_)` message, not
subscripting, so there is no existing grammar conflict to resolve. The native `List`
(`ListObject`, ADR-0020) and its `.ph` protocol already exist — what is missing is
*only* the surface sugar and its lowering.

The frozen floor (ADR-0019) sets the default: a new capability is `.ph`/desugaring
unless it fails the derivability test. List construction is fully expressible over
the existing floor (`List.new` + `rawPush`/`add`), so the literal must **not** add a
primitive unless a measured performance need justifies it.

## Decision

**A list literal is surface sugar that lowers to the existing `List` construction
protocol — no new runtime semantics, no truthiness, ordinary sends.**

1. **Grammar.** `list-literal := '[' (expr (',' expr)* ','?)? ']'`. A list literal is
   a **primary** expression (same tier as a parenthesized expression or block
   literal), so `[a] + [b]` parses as `([a]) + ([b])`.

2. **Desugaring (LL-1 — desugar to sends).** The parser emits the equivalent
   construction AST; the compiler adds **no** new bytecode:
   ```
   [a, b, c]  ≡  List.new().add(a).add(b).add(c)
   []         ≡  List.new()
   ```
   `List#add` returns `self`, so the chain composes. A dedicated `BuildList(n)`
   opcode is **explicitly deferred** to a future performance ADR — it would change
   neither the surface nor the produced value, only the send overhead, and is
   warranted only if profiling shows literal-heavy code is send-bound.

3. **Trailing comma (LL-3).** Permitted: `[a, b,]` ≡ `[a, b]`.

4. **Construction-only (LL-2).** `[…]` means *construction*. Subscript sugar
   (`x[i]` / `x[i] = v`) is **not** adopted here — element access stays the `at(_)` /
   `at(_, put:)` message. Whether to add subscripting is a separate proposal.

5. **Semantics.** Elements evaluate **left-to-right, eagerly**, before the `List` is
   returned (matching call-argument order). The value is a fresh **mutable** `List`
   (identity per literal). `[]` is an empty list, **never** `None` — a list is not
   absence (ADR-0021).

## Consequences

- **Zero floor change.** Ships within U-LEX as parser/compiler desugaring; the
  ADR-0019 census is untouched (R-INV-0.1 stays green). The frozen-floor default is
  honored.
- **`[` gains meaning without conflict.** The unused token becomes list construction;
  because there is no subscript syntax, no disambiguation is needed today.
- **Consistency with the collection family.** Map/Set/Tuple/Range literals (`{a: 1}`,
  `Set(…)`, `(a, b)`, `1..5`) are left to their own literal proposals; this ADR fixes
  only `[…]` so each collection's surface is decided in its own unit (ADR-0020's
  one-unit-per-collection rule). The `(a, b)` tuple literal in particular must be
  disambiguated from a parenthesized expression by the comma — flagged there, not
  here.
- **A later opcode stays open.** Choosing desugar-to-sends now does not foreclose
  `BuildList(n)`; it can replace the lowering transparently.
- **Owner.** U-LEX (surface syntax); the `List` runtime it targets already exists.

## Alternatives considered

- **Dedicated `BuildList(n)` opcode now.** Avoids *n* message sends per literal
  (push *n* values, one opcode builds the `List`). Rejected **for now**: it adds a
  `Bytecode` variant + VM handler + disassembler row for a benefit no profile yet
  demands, and the frozen-floor ethos prefers desugaring until measured otherwise.
  Deferred, not foreclosed (consequence above).
- **Make `[…]` also mean subscripting (`x[i]`).** Familiar from C-family languages,
  but conflates construction with access, introduces the `[` construction-vs-index
  ambiguity, and competes with the existing `at(_)` message. Rejected — keep `[…]`
  construction-only; propose subscripting separately if wanted.
- **No literal; keep `List.new().add(…)`.** Zero work, but leaves a catalog-promised
  surface (`object-model.md` §3) unbuilt and list-heavy code verbose. Rejected as a
  non-decision.
- **Filesystem-style / typed literals.** Out of scope; a list literal is untyped and
  heterogeneous like the underlying `List`.
