// area: collections
// spec: lexical-structure.md §6; ADR-0032 §1, §3.1; DEC-COLL-B
// status: PENDING — graduates when U-COLLTYPES wires `{k:v}` to the native `Map`
// The §6 disambiguation and pair parsing ship in U-COLL; the runtime target is
// deferred, so this currently fails with the "map literal pending" diagnostic.
// This pins the intended surface: bare-identifier keys are symbols (`{a: 1}` ≡
// key `#a`), retrieved via `at(#a)`.

let m = {a: 1, b: 2}
System.print(m.at(#a))
System.print(m.at(#b))
