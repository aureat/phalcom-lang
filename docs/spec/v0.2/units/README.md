# Implementation Units — v0.2

Per-unit implementation specifications. Each **family** is a folder; each unit is
`<n>-<name>.md`. These describe *how a slice of the language was (or will be) built* —
grounded in the ADRs and the normative spec one level up.

## Families

### [`U-CORE/`](U-CORE/) — the core-library track
Kernel reflection, value protocols, collections, and errors, built in Phalcom over the
frozen primitive floor. Foundational rulings/census for this track live in
[`../core/`](../core/) (decisions, floor-census, forward-compat, invariant-requirements).

| Unit | Spec | Status |
|---|---|---|
| U-CORE-1 | [1-kernel-reflection](U-CORE/1-kernel-reflection.md) | ✅ landed (`03764e3`/`b1109c2`) — `hash`, `isA`, `Behavior` reflection, `Method < Function` |
| U-CORE-2 | [2-bool-and-option-residue](U-CORE/2-bool-and-option-residue.md) | mostly landed (`0da64d6`); verify/harden |
| U-CORE-3 | [3-callable-reflection](U-CORE/3-callable-reflection.md) | dispatch-ready — **next** |
| U-CORE-4 | [4-value-tostring](U-CORE/4-value-tostring.md) | dispatch-ready |
| U-CORE-5 | [5-collection-contract](U-CORE/5-collection-contract.md) | dispatch-ready |
| U-CORE-6 | [6-errors](U-CORE/6-errors.md) | dispatch-ready |

### [`U/`](U/) — the language-spine units (forge track)
As-built specifications for the landed spine units: what each implemented and how,
translated from the `forge/` planning record (which stays in place as the source).

| Unit | Spec | Realizes |
|---|---|---|
| U0 | [0-stabilization](U/0-stabilization.md) | verification substrate (`verify.sh`, golden corpus, invariants) |
| U1 | [1-heap-and-value](U/1-heap-and-value.md) | ADR-0009 handle/arena heap · ADR-0010 tagged `Value` |
| U2 | [2-metaclass-tower](U/2-metaclass-tower.md) | ADR-0002 parallel rule · ADR-0003 `Behavior` · `verify_invariants` |
| U3 | [3-selector-dispatch](U/3-selector-dispatch.md) | ADR-0012 label-encoded selectors · IC-ready dispatch |
| U4 | [4-blocks-and-closures](U/4-blocks-and-closures.md) | ADR-0013 upvalues/frame tokens · ADR-0006 `Function` root |
| U5 | [5-control-flow-inliner](U/5-control-flow-inliner.md) | ADR-0018 sacred-selector inliner + deopt guard |
| U6 | [6-option-and-bindings](U/6-option-and-bindings.md) | ADR-0007 Option · ADR-0014 let/var · ADR-0021 no-truthiness |
| U7 | [7-fields-and-construct](U/7-fields-and-construct.md) | ADR-0011 slot layout · `construct` · ADR-0017 static fields |
| U8 | [8-dnu-and-perform](U/8-dnu-and-perform.md) | ADR-0012 `doesNotUnderstand`/`perform` · `Message` |
| U9 | [9-variadics](U/9-variadics.md) | ADR-0012amd rest params `*xs` · `(*)` selector |
| U10 | [10-non-local-return](U/10-non-local-return.md) | ADR-0013 `ReturnNonLocal` + frame-token unwind |
| U11 | [11-bool-tower](U/11-bool-tower.md) | ADR-0004 abstract `Bool` + `True`/`False` |
| U-FE | [fe-front-end](U/fe-front-end.md) | ADR-0016 hand-written lexer + recursive-descent parser |
| U-LEX | [lex-lexical-delta](U/lex-lexical-delta.md) | block comments, digit separators, ADR-0022 `\(expr)` |
| U-LIST | [list-kernel-list](U/list-kernel-list.md) | ADR-0019/0020 native-array `List` |
| U-STD | [std-combinators](U/std-combinators.md) | pure-`.ph` Option/List combinators |

## Convention
- `<n>-<name>.md` — numeric prefix orders the units within a family.
- A unit spec cites the ADR(s) and spec section it realizes, and records the commit(s)
  where it landed once built.
