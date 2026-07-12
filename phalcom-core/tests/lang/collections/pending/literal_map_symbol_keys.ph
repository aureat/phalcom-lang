// area: collections
// spec: lexical-structure.md §6; ADR-0032 §1, §3.1; DEC-COLL-B
// status: PENDING — graduates when the `#IDENT` symbol-literal lexer token lands (U-LEX)
// The §6 disambiguation, pair parsing, and the `{k:v}` -> native `Map` runtime
// wiring all shipped (U-COLL / U-COLLTYPES; see `collections/map_literal_construction.ph`
// for the construction-side PASS case). This fixture is blocked on a DIFFERENT,
// narrower gap: there is no `#IDENT` bare-symbol-literal token in the lexer, so
// `m.at(#a)` fails to lex ("Invalid token") — see DEFERRED.md. This pins the
// intended surface once that token lands: bare-identifier keys are symbols
// (`{a: 1}` ≡ key `#a`), retrieved via `at(#a)`.

let m = {a: 1, b: 2}
System.print(m.at(#a))
System.print(m.at(#b))
