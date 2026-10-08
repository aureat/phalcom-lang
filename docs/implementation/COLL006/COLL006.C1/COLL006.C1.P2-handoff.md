---
id: COLL006.C1.P2
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: handoff
title: "Array<T> Runtime Fixed-Extent Sequence — Handoff to P3 / P4"
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# COLL006.C1.P2 — Handoff to COLL006.C1.P3 and P4

## 1. Repository anchor

- **Worktree Branch:** `feat/telos-collections-upgrade`
- **Completed Plans:** `COLL006.C1.P1`, `COLL006.C1.P2`
- **Next Plans:** `COLL006.C1.P3` (`Vector<T, N>`) and `COLL006.C1.P4` (Static Sequences & Default Cutover)

## 2. Completed prerequisites for P3 and P4

- Canonical `Array<T>` declaration, runtime class `array_class`, and heap representation `ArrayObject` exist.
- `ArrayObject` uses `Box<[Value]>` storage with permanent extent and mutable elements.
- Minimal native floor is established: `rawLength`, `rawAt(_)`, `rawSet(_,_)`, `toString`.
- Telos-authored sequence protocol in `universe.collections.array` conforms to `Iterator` / `Iterable`.
- Key policy rejects `Array` as a mutable hash key.
- Semantic checker recognizes `<T>[...]`, `<_>[...]`, and contextual `Array<...>` as selecting `SequenceLiteralFamily::Array`.
- Compiler directly lowers fixed Array literals to `BuildArray(u16)` without `List` staging.
- Compiler lowers dynamic spread Array literals using private `ArrayLiteralBuilderObject` finalized to `ArrayObject`.
- Bare unconstrained runtime literals `[...]` remain `List` until P4.
- `frozen` flag on `ArrayObject` exists as a representation seam for P4 const materialization.

## 3. Stable interface / takeover map

| Concern | Established Interface |
|---|---|
| AST bracket literal | `phalcom_ast::ast::Expr::BracketLiteral(Box<BracketLiteralExpr>)` |
| AST literal head | `BracketLiteralHeadSyntax { arguments, range }` |
| Canonical Universe Key | `phalcom_native_meta::UniverseKey::Array` |
| Runtime Class | `CoreClasses.array_class` |
| Heap Object Variant | `phalcom_core::heap::Object::Array(ArrayObject)` |
| Array Storage | `phalcom_core::heap::ArrayObject` (`elements: Box<[Value]>`, `frozen: bool`) |
| Private Spread Builder | `phalcom_core::heap::ArrayLiteralBuilderObject` |
| Direct Array Bytecode | `Bytecode::BuildArray(u16)` |
| Builder Bytecodes | `BeginArrayLiteral`, `ArrayLiteralAppend`, `FinishArrayLiteral`, `ArrayTryExpandTuplePositionals` |
| Semantic Family | `SequenceLiteralFamily::Array` in `SequenceLiteralResolution` |

## 4. Invariants for P3 (`Vector<T, N>`)

1. **Static dimension:** `Vector<T, N>` requires static type argument `N: Int` (`N >= 0`), unlike `Array<T>` whose length is runtime state.
2. **Two-head syntax:** `<T, N>[...]` is the explicit syntax reserved for `Vector`. The parser already parses it; P3 will activate it in `synthesize_bracket_literal`.
3. **Immutability:** Vector is immutable; no element mutation or structural mutation.
4. **No List staging:** Like Array, Vector must construct directly without staging through List.
5. **Spread restriction:** Only statically exact spreads are admitted into Vector.

## 5. Invariants for P4 (Static Sequences & Default Cutover)

1. **Atomic cutover:** P4 owns switching bare runtime `[...]` from List to Array, and bare static `[...]` to Vector.
2. **Const Array:** P4 will activate `const Array<T>` materialization using the `frozen` field on `ArrayObject`.

## 6. Recommended next actions

- For P3 implementer:
  - Add canonical `Vector<T, N>` Universe declaration and `UniverseKey::Vector`.
  - Add `VectorObject` in `phalcom-core/src/heap`.
  - In `synthesize_bracket_literal`, activate `<T, N>[...]` and contextual `Vector<T, N>` selection.
  - Implement static cardinality verification for Vector literals.
  - Add direct `BuildVector(u16)` lowering and runtime execution.
