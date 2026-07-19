// area: collections
// spec: lexical-structure.md §6; ADR-0032 §1, §3.1; DEC-COLL-B; selectors.md §2
// status: PASS
// Graduated by U-LEX-HASH, which lands two coupled fixes: the `#IDENT`
// name-symbol lexer token (`m.at(#a)` now lexes) and `value.rs::value_eq`'s
// missing `(Value::Symbol, Value::Symbol)` arm (without it, `#a` — a freshly
// interned key distinct from the map's stored `Symbol.new("a")` key — would
// still fail to retrieve). Proves the intended surface: bare-identifier keys
// are symbols (`{a: 1}` ≡ key `#a`), retrieved via `at(#a)`.

const m = {a: 1, b: 2}
System.print(m.at(#a))
System.print(m.at(#b))
